//! Injected monotonic fixed-step scheduler for the future continuous runtime.
//!
//! This is a pure clock/accumulator component. It owns no thread or timer,
//! does not serialize clock handles, and has no Campaign, KCC, IPC, or input
//! implementation. A caller may consume the returned tick plan by sampling
//! held input and consuming edge input at its own protocol boundary.

use std::fmt;

pub const DEFAULT_FREQUENCY_HZ: u32 = 60;
pub const COMPARISON_FREQUENCY_HZ: u32 = 30;
pub const MAX_TICKS_PER_FRAME: u32 = 5;
const NANOS_PER_SECOND: u64 = 1_000_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchedulerError {
    InvalidFrequency(u32),
    ClockRegression { previous_ns: u64, current_ns: u64 },
    ArithmeticOverflow(&'static str),
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFrequency(hz) => write!(formatter, "unsupported fixed frequency: {hz} Hz"),
            Self::ClockRegression {
                previous_ns,
                current_ns,
            } => write!(
                formatter,
                "monotonic clock regressed from {previous_ns} ns to {current_ns} ns"
            ),
            Self::ArithmeticOverflow(field) => write!(formatter, "{field} overflowed"),
        }
    }
}

impl std::error::Error for SchedulerError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedStepConfig {
    frequency_hz: u32,
    fixed_dt_nanos: u64,
    max_ticks_per_frame: u32,
}

impl FixedStepConfig {
    pub fn new(frequency_hz: u32) -> Result<Self, SchedulerError> {
        if !matches!(frequency_hz, DEFAULT_FREQUENCY_HZ | COMPARISON_FREQUENCY_HZ) {
            return Err(SchedulerError::InvalidFrequency(frequency_hz));
        }
        Ok(Self {
            frequency_hz,
            fixed_dt_nanos: NANOS_PER_SECOND / u64::from(frequency_hz),
            max_ticks_per_frame: MAX_TICKS_PER_FRAME,
        })
    }

    pub const fn frequency_hz(self) -> u32 {
        self.frequency_hz
    }

    pub const fn fixed_dt_nanos(self) -> u64 {
        self.fixed_dt_nanos
    }

    pub const fn max_ticks_per_frame(self) -> u32 {
        self.max_ticks_per_frame
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickPlan {
    tick_indices: Vec<u64>,
    simulation_times_nanos: Vec<u64>,
    simulation_time_nanos: u64,
    dropped_time_nanos: u64,
    paused: bool,
}

impl TickPlan {
    fn paused_plan(simulation_time_nanos: u64) -> Self {
        Self {
            tick_indices: Vec::new(),
            simulation_times_nanos: Vec::new(),
            simulation_time_nanos,
            dropped_time_nanos: 0,
            paused: true,
        }
    }

    fn active(
        first_tick: u64,
        tick_count: usize,
        fixed_dt_nanos: u64,
        simulation_time_nanos: u64,
        dropped_time_nanos: u64,
    ) -> Result<Self, SchedulerError> {
        let mut tick_indices = Vec::with_capacity(tick_count);
        let mut simulation_times_nanos = Vec::with_capacity(tick_count);
        for offset in 0..tick_count {
            let tick_index = first_tick
                .checked_add(offset as u64)
                .ok_or(SchedulerError::ArithmeticOverflow("tick index"))?;
            let completed_ticks = tick_index
                .checked_add(1)
                .ok_or(SchedulerError::ArithmeticOverflow("simulation time"))?;
            let simulation_time = completed_ticks
                .checked_mul(fixed_dt_nanos)
                .ok_or(SchedulerError::ArithmeticOverflow("simulation time"))?;
            tick_indices.push(tick_index);
            simulation_times_nanos.push(simulation_time);
        }
        Ok(Self {
            tick_indices,
            simulation_times_nanos,
            simulation_time_nanos,
            dropped_time_nanos,
            paused: false,
        })
    }

    pub fn tick_count(&self) -> usize {
        self.tick_indices.len()
    }

    pub fn tick_indices(&self) -> &[u64] {
        &self.tick_indices
    }

    pub fn simulation_times_nanos(&self) -> &[u64] {
        &self.simulation_times_nanos
    }

    pub const fn simulation_time_nanos(&self) -> u64 {
        self.simulation_time_nanos
    }

    pub const fn dropped_time_nanos(&self) -> u64 {
        self.dropped_time_nanos
    }

    pub const fn paused(&self) -> bool {
        self.paused
    }
}

pub struct FixedStepScheduler {
    config: FixedStepConfig,
    last_observed_ns: u64,
    accumulator_nanos: u64,
    total_ticks: u64,
    total_dropped_time_nanos: u64,
    paused: bool,
}

impl FixedStepScheduler {
    pub fn new(initial_now_ns: u64) -> Self {
        Self::with_frequency(DEFAULT_FREQUENCY_HZ, initial_now_ns)
            .expect("the default fixed frequency must be valid")
    }

    pub fn with_frequency(frequency_hz: u32, initial_now_ns: u64) -> Result<Self, SchedulerError> {
        Ok(Self {
            config: FixedStepConfig::new(frequency_hz)?,
            last_observed_ns: initial_now_ns,
            accumulator_nanos: 0,
            total_ticks: 0,
            total_dropped_time_nanos: 0,
            paused: false,
        })
    }

    pub const fn config(&self) -> FixedStepConfig {
        self.config
    }

    pub const fn is_paused(&self) -> bool {
        self.paused
    }

    pub const fn last_observed_ns(&self) -> u64 {
        self.last_observed_ns
    }

    pub const fn accumulator_nanos(&self) -> u64 {
        self.accumulator_nanos
    }

    pub const fn total_ticks(&self) -> u64 {
        self.total_ticks
    }

    pub const fn simulation_time_nanos(&self) -> u64 {
        self.total_ticks.saturating_mul(self.config.fixed_dt_nanos)
    }

    pub const fn total_dropped_time_nanos(&self) -> u64 {
        self.total_dropped_time_nanos
    }

    fn validate_clock(&self, now_ns: u64) -> Result<(), SchedulerError> {
        if now_ns < self.last_observed_ns {
            return Err(SchedulerError::ClockRegression {
                previous_ns: self.last_observed_ns,
                current_ns: now_ns,
            });
        }
        Ok(())
    }

    /// Advances from an injected monotonic nanosecond reading.
    ///
    /// At most five fixed ticks are returned. If more are owed, the complete
    /// remaining accumulator is reported as `dropped_time_nanos` and cleared,
    /// so the scheduler never creates a later catch-up burst.
    pub fn advance(&mut self, now_ns: u64) -> Result<TickPlan, SchedulerError> {
        self.validate_clock(now_ns)?;
        if self.paused {
            self.last_observed_ns = now_ns;
            return Ok(TickPlan::paused_plan(self.simulation_time_nanos()));
        }

        let elapsed = now_ns - self.last_observed_ns;
        let accumulated = self
            .accumulator_nanos
            .checked_add(elapsed)
            .ok_or(SchedulerError::ArithmeticOverflow("accumulator"))?;
        let available_ticks = accumulated / self.config.fixed_dt_nanos;
        let planned_ticks = available_ticks.min(u64::from(self.config.max_ticks_per_frame));
        let consumed = planned_ticks
            .checked_mul(self.config.fixed_dt_nanos)
            .ok_or(SchedulerError::ArithmeticOverflow("consumed time"))?;
        let remainder = accumulated - consumed;
        let dropped_time_nanos = if available_ticks > u64::from(self.config.max_ticks_per_frame) {
            remainder
        } else {
            0
        };
        let next_total_ticks = self
            .total_ticks
            .checked_add(planned_ticks)
            .ok_or(SchedulerError::ArithmeticOverflow("total ticks"))?;
        let next_simulation_time = next_total_ticks
            .checked_mul(self.config.fixed_dt_nanos)
            .ok_or(SchedulerError::ArithmeticOverflow("simulation time"))?;
        let next_total_dropped = self
            .total_dropped_time_nanos
            .checked_add(dropped_time_nanos)
            .ok_or(SchedulerError::ArithmeticOverflow("dropped time"))?;
        let plan = TickPlan::active(
            self.total_ticks,
            planned_ticks as usize,
            self.config.fixed_dt_nanos,
            next_simulation_time,
            dropped_time_nanos,
        )?;

        self.last_observed_ns = now_ns;
        self.accumulator_nanos = if dropped_time_nanos == 0 {
            remainder
        } else {
            0
        };
        self.total_ticks = next_total_ticks;
        self.total_dropped_time_nanos = next_total_dropped;
        Ok(plan)
    }

    /// Starts a pause barrier at an injected monotonic reading.
    pub fn pause(&mut self, now_ns: u64) -> Result<(), SchedulerError> {
        self.validate_clock(now_ns)?;
        self.last_observed_ns = now_ns;
        self.accumulator_nanos = 0;
        self.paused = true;
        Ok(())
    }

    /// Resumes from a pause without replaying wall-clock time.
    pub fn resume(&mut self, now_ns: u64) -> Result<(), SchedulerError> {
        self.validate_clock(now_ns)?;
        if self.paused {
            self.last_observed_ns = now_ns;
            self.accumulator_nanos = 0;
            self.paused = false;
        }
        Ok(())
    }
}
