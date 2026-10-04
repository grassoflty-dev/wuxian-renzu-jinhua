#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixedStepConfig {
    pub hz: u32,
    pub max_steps_per_frame: u32,
}

impl FixedStepConfig {
    pub fn new(hz: u32, max_steps_per_frame: u32) -> Result<Self, FixedStepError> {
        if hz == 0 || max_steps_per_frame == 0 {
            return Err(FixedStepError::InvalidConfig);
        }
        Ok(Self {
            hz,
            max_steps_per_frame,
        })
    }
    pub fn dt_s(self) -> f32 {
        1.0 / self.hz as f32
    }
}

#[derive(Clone, Debug)]
pub struct FixedStepClock {
    pub config: FixedStepConfig,
    accumulator_s: f32,
}

impl FixedStepClock {
    pub fn new(config: FixedStepConfig) -> Self {
        Self {
            config,
            accumulator_s: 0.0,
        }
    }
    pub fn push_frame(&mut self, frame_dt_s: f32) -> Result<u32, FixedStepError> {
        if !frame_dt_s.is_finite() || frame_dt_s < 0.0 {
            return Err(FixedStepError::InvalidDelta);
        }
        self.accumulator_s +=
            frame_dt_s.min(self.config.dt_s() * self.config.max_steps_per_frame as f32);
        let steps = (self.accumulator_s / self.config.dt_s()).floor() as u32;
        let steps = steps.min(self.config.max_steps_per_frame);
        self.accumulator_s -= self.config.dt_s() * steps as f32;
        Ok(steps)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixedStepError {
    InvalidConfig,
    InvalidDelta,
}
