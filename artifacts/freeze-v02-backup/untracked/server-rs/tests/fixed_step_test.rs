use std::time::Instant;

use wuxian_horror_ch1::fixed_step::{
    FixedStepScheduler, SchedulerError, COMPARISON_FREQUENCY_HZ, DEFAULT_FREQUENCY_HZ,
    MAX_TICKS_PER_FRAME,
};

const SECOND_NS: u64 = 1_000_000_000;
const ACTIVE_SECONDS: u64 = 600;
const PAUSE_SECONDS: u64 = 5;

fn run_active_window(frequency_hz: u32) -> (FixedStepScheduler, u64) {
    let mut scheduler =
        FixedStepScheduler::with_frequency(frequency_hz, 0).expect("supported frequency");
    let dt = scheduler.config().fixed_dt_nanos();
    let expected_ticks = ACTIVE_SECONDS * u64::from(frequency_hz);
    let half_ticks = expected_ticks / 2;
    let mut now_ns = 0;

    for tick in 0..half_ticks {
        now_ns += dt;
        let plan = scheduler.advance(now_ns).unwrap();
        assert_eq!(plan.tick_count(), 1);
        assert_eq!(plan.tick_indices(), &[tick]);
        assert_eq!(plan.simulation_times_nanos(), &[(tick + 1) * dt]);
    }
    scheduler.pause(now_ns).unwrap();
    let paused_now_ns = now_ns + PAUSE_SECONDS * SECOND_NS;
    let paused_plan = scheduler.advance(paused_now_ns).unwrap();
    assert!(paused_plan.paused());
    assert_eq!(paused_plan.tick_count(), 0);
    scheduler.resume(paused_now_ns).unwrap();

    now_ns = paused_now_ns;
    for tick in half_ticks..expected_ticks {
        now_ns += dt;
        let plan = scheduler.advance(now_ns).unwrap();
        assert_eq!(plan.tick_count(), 1);
        assert_eq!(plan.tick_indices(), &[tick]);
    }

    (scheduler, now_ns)
}

#[test]
fn default_and_comparison_rates_have_integer_fixed_steps() {
    let sixty = FixedStepScheduler::new(0);
    let thirty = FixedStepScheduler::with_frequency(COMPARISON_FREQUENCY_HZ, 0).unwrap();
    assert_eq!(sixty.config().frequency_hz(), DEFAULT_FREQUENCY_HZ);
    assert_eq!(sixty.config().fixed_dt_nanos(), 16_666_666);
    assert_eq!(thirty.config().fixed_dt_nanos(), 33_333_333);
    assert_eq!(sixty.config().max_ticks_per_frame(), MAX_TICKS_PER_FRAME);
    assert!(matches!(
        FixedStepScheduler::with_frequency(24, 0),
        Err(SchedulerError::InvalidFrequency(24))
    ));
}

#[test]
fn six_hundred_active_seconds_pause_for_five_seconds_without_replay() {
    for frequency_hz in [COMPARISON_FREQUENCY_HZ, DEFAULT_FREQUENCY_HZ] {
        let (scheduler, final_now_ns) = run_active_window(frequency_hz);
        let expected_ticks = ACTIVE_SECONDS * u64::from(frequency_hz);
        assert_eq!(scheduler.total_ticks(), expected_ticks);
        assert_eq!(scheduler.total_dropped_time_nanos(), 0);
        assert_eq!(scheduler.accumulator_nanos(), 0);
        assert_eq!(
            scheduler.simulation_time_nanos(),
            expected_ticks * scheduler.config().fixed_dt_nanos()
        );
        assert_eq!(
            final_now_ns,
            expected_ticks * scheduler.config().fixed_dt_nanos() + PAUSE_SECONDS * SECOND_NS
        );
    }
}

#[test]
fn long_frame_is_capped_and_excess_time_is_explicitly_dropped() {
    for frequency_hz in [COMPARISON_FREQUENCY_HZ, DEFAULT_FREQUENCY_HZ] {
        let mut scheduler = FixedStepScheduler::with_frequency(frequency_hz, 0).unwrap();
        let plan = scheduler.advance(250_000_000).unwrap();
        let dt = scheduler.config().fixed_dt_nanos();
        let expected_drop = 250_000_000 - u64::from(MAX_TICKS_PER_FRAME) * dt;
        assert_eq!(plan.tick_count(), MAX_TICKS_PER_FRAME as usize);
        assert_eq!(plan.tick_indices(), &[0, 1, 2, 3, 4]);
        assert_eq!(plan.dropped_time_nanos(), expected_drop);
        assert_eq!(scheduler.total_dropped_time_nanos(), expected_drop);
        assert_eq!(scheduler.accumulator_nanos(), 0);

        let recovery = scheduler.advance(250_000_000 + dt).unwrap();
        assert_eq!(recovery.tick_count(), 1);
        assert_eq!(recovery.tick_indices(), &[5]);
    }
}

#[test]
fn no_drop_frame_slicing_produces_the_same_tick_plan_and_clock() {
    for frequency_hz in [COMPARISON_FREQUENCY_HZ, DEFAULT_FREQUENCY_HZ] {
        let dt = 1_000_000_000 / u64::from(frequency_hz);
        let total_ticks = 600 * u64::from(frequency_hz);
        let final_now = total_ticks * dt;
        let mut one_tick = FixedStepScheduler::with_frequency(frequency_hz, 0).unwrap();
        for tick in 0..total_ticks {
            let plan = one_tick.advance((tick + 1) * dt).unwrap();
            assert_eq!(plan.tick_count(), 1);
        }

        let mut split = FixedStepScheduler::with_frequency(frequency_hz, 0).unwrap();
        let mut now = 0;
        let mut remaining = total_ticks;
        let frame_sizes = [1_u64, 2, 4, 3, 5];
        let mut frame = 0;
        while remaining > 0 {
            let size = frame_sizes[frame % frame_sizes.len()].min(remaining);
            now += size * dt;
            let plan = split.advance(now).unwrap();
            assert_eq!(plan.tick_count(), size as usize);
            assert_eq!(plan.dropped_time_nanos(), 0);
            remaining -= size;
            frame += 1;
        }

        assert_eq!(one_tick.total_ticks(), split.total_ticks());
        assert_eq!(
            one_tick.simulation_time_nanos(),
            split.simulation_time_nanos()
        );
        assert_eq!(one_tick.accumulator_nanos(), split.accumulator_nanos());
        assert_eq!(one_tick.last_observed_ns(), final_now);
        assert_eq!(split.last_observed_ns(), final_now);
    }
}

#[test]
fn pause_resume_is_a_repeated_barrier_and_does_not_replay_background_time() {
    let mut scheduler = FixedStepScheduler::new(0);
    let dt = scheduler.config().fixed_dt_nanos();
    assert_eq!(scheduler.advance(dt).unwrap().tick_count(), 1);

    scheduler.pause(dt).unwrap();
    scheduler.pause(dt + SECOND_NS).unwrap();
    assert!(scheduler.is_paused());
    assert_eq!(
        scheduler.advance(dt + 2 * SECOND_NS).unwrap().tick_count(),
        0
    );
    scheduler.resume(dt + 2 * SECOND_NS).unwrap();
    scheduler.resume(dt + 2 * SECOND_NS).unwrap();
    assert_eq!(
        scheduler
            .advance(2 * dt + 2 * SECOND_NS)
            .unwrap()
            .tick_count(),
        1
    );
    assert_eq!(scheduler.total_ticks(), 2);
    assert_eq!(scheduler.total_dropped_time_nanos(), 0);
}

#[test]
fn clock_regression_is_rejected_without_mutation() {
    let mut scheduler = FixedStepScheduler::new(0);
    let dt = scheduler.config().fixed_dt_nanos();
    scheduler.advance(dt - 1).unwrap();
    let before = (
        scheduler.last_observed_ns(),
        scheduler.accumulator_nanos(),
        scheduler.total_ticks(),
    );
    assert!(matches!(
        scheduler.advance(dt - 2),
        Err(SchedulerError::ClockRegression { .. })
    ));
    assert_eq!(
        before,
        (
            scheduler.last_observed_ns(),
            scheduler.accumulator_nanos(),
            scheduler.total_ticks()
        )
    );
}

#[test]
fn scheduler_overhead_probe_is_recorded_without_claiming_runtime_percentiles() {
    let start = Instant::now();
    let mut scheduler = FixedStepScheduler::new(0);
    let dt = scheduler.config().fixed_dt_nanos();
    for tick in 0..10_000_u64 {
        let plan = scheduler.advance((tick + 1) * dt).unwrap();
        std::hint::black_box(plan);
    }
    let elapsed = start.elapsed();
    println!("fixed_step_10000_ticks_elapsed_ns={}", elapsed.as_nanos());
    assert_eq!(scheduler.total_ticks(), 10_000);
}
