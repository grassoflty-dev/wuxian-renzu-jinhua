#![allow(dead_code)]

use serde::Serialize;
use serde_json::{json, Value};
use std::{env, fs, hint::black_box, path::PathBuf, time::Instant};

const NS_PER_SECOND: u64 = 1_000_000_000;
const SIMULATION_SECONDS: u64 = 600;
const MAX_CATCH_UP: u32 = 5;
const PAUSE_WINDOW_NS: u64 = 5 * NS_PER_SECOND;
const SIXTY_HZ_BUDGET_US: f64 = 1_000_000.0 / 60.0;

#[derive(Clone, Copy, Debug, Serialize)]
struct FrameAdvance {
    executed_ticks: u32,
    dropped_ns: u64,
    paused: bool,
}

#[derive(Debug)]
struct FixedTick {
    dt_ns: u64,
    max_catch_up: u32,
    last_wall_ns: u64,
    accumulator_ns: u64,
    tick_count: u64,
    simulated_ns: u64,
    dropped_ns: u64,
    max_ticks_frame: u32,
    paused: bool,
}

impl FixedTick {
    fn new(hz: u32) -> Self {
        Self {
            dt_ns: NS_PER_SECOND / hz as u64,
            max_catch_up: MAX_CATCH_UP,
            last_wall_ns: 0,
            accumulator_ns: 0,
            tick_count: 0,
            simulated_ns: 0,
            dropped_ns: 0,
            max_ticks_frame: 0,
            paused: false,
        }
    }

    fn pause(&mut self, now_ns: u64) {
        self.paused = true;
        self.last_wall_ns = now_ns;
        self.accumulator_ns = 0;
    }

    fn resume(&mut self, now_ns: u64) {
        self.paused = false;
        self.last_wall_ns = now_ns;
        self.accumulator_ns = 0;
    }

    fn advance_to(&mut self, now_ns: u64) -> FrameAdvance {
        if now_ns < self.last_wall_ns {
            return FrameAdvance { executed_ticks: 0, dropped_ns: 0, paused: self.paused };
        }
        if self.paused {
            self.last_wall_ns = now_ns;
            return FrameAdvance { executed_ticks: 0, dropped_ns: 0, paused: true };
        }

        let elapsed_ns = now_ns - self.last_wall_ns;
        self.last_wall_ns = now_ns;
        self.accumulator_ns = self.accumulator_ns.saturating_add(elapsed_ns);

        let mut executed_ticks = 0;
        while self.accumulator_ns >= self.dt_ns && executed_ticks < self.max_catch_up {
            self.accumulator_ns -= self.dt_ns;
            executed_ticks += 1;
            self.tick_count += 1;
            self.simulated_ns += self.dt_ns;
        }

        let dropped_ns = if self.accumulator_ns >= self.dt_ns {
            let dropped = self.accumulator_ns - (self.accumulator_ns % self.dt_ns);
            self.accumulator_ns %= self.dt_ns;
            self.dropped_ns += dropped;
            dropped
        } else {
            0
        };
        self.max_ticks_frame = self.max_ticks_frame.max(executed_ticks);
        FrameAdvance { executed_ticks, dropped_ns, paused: false }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
struct ScheduleSummary {
    hz: u32,
    dt_ns: u64,
    expected_ticks: u64,
    actual_ticks: u64,
    expected_simulated_s: f64,
    actual_simulated_s: f64,
    simulated_duration_error_s: f64,
    wall_elapsed_s: f64,
    pause_window_s: f64,
    ticks_during_pause: u64,
    dropped_after_catch_up: u64,
    max_ticks_per_frame: u32,
    pass: bool,
}

fn frame_target_ns(frame: u64, hz: u32) -> u64 {
    frame.saturating_mul(NS_PER_SECOND) / hz as u64
}

fn run_ten_minute_schedule(hz: u32) -> ScheduleSummary {
    let total_frames = SIMULATION_SECONDS * hz as u64;
    let pause_at_frame = total_frames / 2;
    let mut runner = FixedTick::new(hz);
    let mut now_ns = 0;
    for frame in 1..=pause_at_frame {
        now_ns = frame_target_ns(frame, hz);
        runner.advance_to(now_ns);
    }

    runner.pause(now_ns);
    let ticks_before_pause = runner.tick_count;
    now_ns += PAUSE_WINDOW_NS;
    runner.advance_to(now_ns);
    let ticks_during_pause = runner.tick_count - ticks_before_pause;
    runner.resume(now_ns);

    for frame in (pause_at_frame + 1)..=total_frames {
        let active_ns = frame_target_ns(frame, hz);
        runner.advance_to(active_ns + PAUSE_WINDOW_NS);
    }
    now_ns = frame_target_ns(total_frames, hz) + PAUSE_WINDOW_NS;

    let expected_ticks = total_frames;
    let expected_simulated_s = SIMULATION_SECONDS as f64;
    let actual_simulated_s = runner.simulated_ns as f64 / NS_PER_SECOND as f64;
    let error = (actual_simulated_s - expected_simulated_s).abs();
    let wall_elapsed_s = now_ns as f64 / NS_PER_SECOND as f64;
    let pass = runner.tick_count == expected_ticks
        && ticks_during_pause == 0
        && error <= 1.0 / hz as f64
        && runner.max_ticks_frame <= MAX_CATCH_UP;
    ScheduleSummary {
        hz,
        dt_ns: runner.dt_ns,
        expected_ticks,
        actual_ticks: runner.tick_count,
        expected_simulated_s,
        actual_simulated_s,
        simulated_duration_error_s: error,
        wall_elapsed_s,
        pause_window_s: PAUSE_WINDOW_NS as f64 / NS_PER_SECOND as f64,
        ticks_during_pause,
        dropped_after_catch_up: runner.dropped_ns,
        max_ticks_per_frame: runner.max_ticks_frame,
        pass,
    }
}

#[derive(Debug, Serialize)]
struct CatchUpSummary {
    hz: u32,
    long_frame_s: f64,
    ticks_on_long_frame: u32,
    max_catch_up: u32,
    dropped_s: f64,
    ticks_while_paused: u64,
    ticks_first_frame_after_resume: u32,
    no_infinite_catch_up: bool,
    no_background_burst: bool,
    pass: bool,
}

fn run_catch_up_probe(hz: u32) -> CatchUpSummary {
    let mut runner = FixedTick::new(hz);
    let long_frame_ns = 250_000_000;
    let long = runner.advance_to(long_frame_ns);
    runner.pause(long_frame_ns);
    let before_pause = runner.tick_count;
    runner.advance_to(long_frame_ns + PAUSE_WINDOW_NS);
    let ticks_while_paused = runner.tick_count - before_pause;
    runner.resume(long_frame_ns + PAUSE_WINDOW_NS);
    let after_resume = runner.advance_to(long_frame_ns + PAUSE_WINDOW_NS + runner.dt_ns);
    let no_infinite_catch_up = long.executed_ticks <= MAX_CATCH_UP && runner.max_ticks_frame <= MAX_CATCH_UP;
    let no_background_burst = ticks_while_paused == 0 && after_resume.executed_ticks <= 1;
    CatchUpSummary {
        hz,
        long_frame_s: long_frame_ns as f64 / NS_PER_SECOND as f64,
        ticks_on_long_frame: long.executed_ticks,
        max_catch_up: MAX_CATCH_UP,
        dropped_s: long.dropped_ns as f64 / NS_PER_SECOND as f64,
        ticks_while_paused,
        ticks_first_frame_after_resume: after_resume.executed_ticks,
        no_infinite_catch_up,
        no_background_burst,
        pass: no_infinite_catch_up && no_background_burst && long.dropped_ns > 0,
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
enum InputKind {
    Press,
    Release,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct InputEvent {
    id: u32,
    at_ms: u64,
    kind: InputKind,
}

#[derive(Debug, Serialize)]
struct InputTickRecord {
    tick: u64,
    held: bool,
    consumed_edges: Vec<u32>,
}

#[derive(Debug)]
struct InputState {
    held: bool,
    pending_edges: Vec<u32>,
    consumed: Vec<u32>,
    dropped_while_paused: Vec<u32>,
    ignored_duplicate_presses: Vec<u32>,
    records: Vec<InputTickRecord>,
}

impl InputState {
    fn new() -> Self {
        Self {
            held: false,
            pending_edges: Vec::new(),
            consumed: Vec::new(),
            dropped_while_paused: Vec::new(),
            ignored_duplicate_presses: Vec::new(),
            records: Vec::new(),
        }
    }

    fn ingest(&mut self, event: InputEvent, paused: bool) {
        if paused {
            self.dropped_while_paused.push(event.id);
            return;
        }
        match event.kind {
            InputKind::Press if self.held => self.ignored_duplicate_presses.push(event.id),
            InputKind::Press => {
                self.held = true;
                self.pending_edges.push(event.id);
            }
            InputKind::Release if self.held => {
                self.held = false;
                self.pending_edges.push(event.id);
            }
            InputKind::Release => {}
        }
    }

    fn clear_for_pause(&mut self) {
        self.held = false;
        self.pending_edges.clear();
    }

    fn consume_tick(&mut self, tick: u64) {
        let edges = std::mem::take(&mut self.pending_edges);
        self.consumed.extend(edges.iter().copied());
        self.records.push(InputTickRecord { tick, held: self.held, consumed_edges: edges });
    }
}

#[derive(Debug, Serialize)]
struct InputSummary {
    events: Vec<InputEvent>,
    consumed_edge_ids: Vec<u32>,
    dropped_while_paused: Vec<u32>,
    ignored_duplicate_presses: Vec<u32>,
    held_ticks_before_pause: u64,
    held_ticks_after_resume: u64,
    max_active_frame_ms: f64,
    max_wall_frame_ms: f64,
    repeated_edge_consumed_once: bool,
    continuous_state_not_lost: bool,
    paused_input_dropped_without_burst: bool,
    pass: bool,
    key_frames: Vec<InputTickRecord>,
}

fn run_input_probe(hz: u32) -> InputSummary {
    let events = vec![
        InputEvent { id: 1, at_ms: 100, kind: InputKind::Press },
        InputEvent { id: 2, at_ms: 120, kind: InputKind::Press },
        InputEvent { id: 3, at_ms: 250, kind: InputKind::Release },
        InputEvent { id: 4, at_ms: 300, kind: InputKind::Press },
        InputEvent { id: 5, at_ms: 340, kind: InputKind::Release },
        InputEvent { id: 6, at_ms: 500, kind: InputKind::Press },
        InputEvent { id: 7, at_ms: 1500, kind: InputKind::Press },
        InputEvent { id: 8, at_ms: 2000, kind: InputKind::Release },
        InputEvent { id: 9, at_ms: 3100, kind: InputKind::Press },
        InputEvent { id: 10, at_ms: 3300, kind: InputKind::Release },
    ];
    let mut runner = FixedTick::new(hz);
    let mut input = InputState::new();
    let mut event_index = 0;
    let mut now_ns = 0;
    let mut max_active_frame_ns = 0;
    let mut max_wall_frame_ns = 0;

    let advance = |next_ns: u64, runner: &mut FixedTick, input: &mut InputState, event_index: &mut usize, now_ns: &mut u64, max_active_frame_ns: &mut u64, max_wall_frame_ns: &mut u64| {
        let frame_ns = next_ns - *now_ns;
        *max_wall_frame_ns = (*max_wall_frame_ns).max(frame_ns);
        if !runner.paused {
            *max_active_frame_ns = (*max_active_frame_ns).max(frame_ns);
        }
        while *event_index < events.len() && events[*event_index].at_ms * 1_000_000 <= next_ns {
            input.ingest(events[*event_index], runner.paused);
            *event_index += 1;
        }
        let result = runner.advance_to(next_ns);
        let first_tick = runner.tick_count - result.executed_ticks as u64;
        for offset in 0..result.executed_ticks {
            input.consume_tick(first_tick + offset as u64 + 1);
        }
        *now_ns = next_ns;
    };

    // Fine frames through 600 ms, one intentionally blocked 250 ms frame,
    // then a precise boundary at 1 s for the pause transition.
    let fine_frames = (hz as u64 * 6) / 10;
    for frame in 1..=fine_frames {
        advance(frame_target_ns(frame, hz), &mut runner, &mut input, &mut event_index, &mut now_ns, &mut max_active_frame_ns, &mut max_wall_frame_ns);
    }
    advance(now_ns + 250_000_000, &mut runner, &mut input, &mut event_index, &mut now_ns, &mut max_active_frame_ns, &mut max_wall_frame_ns);
    advance(NS_PER_SECOND, &mut runner, &mut input, &mut event_index, &mut now_ns, &mut max_active_frame_ns, &mut max_wall_frame_ns);

    let ticks_before_pause = runner.tick_count;
    runner.pause(now_ns);
    input.clear_for_pause();
    advance(1_500_000_000, &mut runner, &mut input, &mut event_index, &mut now_ns, &mut max_active_frame_ns, &mut max_wall_frame_ns);
    advance(2_000_000_000, &mut runner, &mut input, &mut event_index, &mut now_ns, &mut max_active_frame_ns, &mut max_wall_frame_ns);
    advance(3_000_000_000, &mut runner, &mut input, &mut event_index, &mut now_ns, &mut max_active_frame_ns, &mut max_wall_frame_ns);
    let ticks_while_paused = runner.tick_count - ticks_before_pause;

    runner.resume(now_ns);
    advance(3_100_000_000, &mut runner, &mut input, &mut event_index, &mut now_ns, &mut max_active_frame_ns, &mut max_wall_frame_ns);
    advance(3_300_000_000, &mut runner, &mut input, &mut event_index, &mut now_ns, &mut max_active_frame_ns, &mut max_wall_frame_ns);
    advance(3_500_000_000, &mut runner, &mut input, &mut event_index, &mut now_ns, &mut max_active_frame_ns, &mut max_wall_frame_ns);

    let held_ticks_before_pause = input.records.iter().filter(|record| record.tick <= ticks_before_pause && record.held).count() as u64;
    let held_ticks_after_resume = input.records.iter().filter(|record| record.tick > ticks_before_pause && record.held).count() as u64;
    let repeated_edge_consumed_once = input.consumed.iter().filter(|id| **id == 1).count() == 1
        && !input.consumed.contains(&2)
        && input.ignored_duplicate_presses == vec![2];
    let continuous_state_not_lost = held_ticks_before_pause > 0 && held_ticks_after_resume > 0 && input.consumed.contains(&6) && input.consumed.contains(&9);
    let paused_input_dropped_without_burst = input.dropped_while_paused == vec![7, 8] && ticks_while_paused == 0 && !input.consumed.contains(&7) && !input.consumed.contains(&8);
    let pass = repeated_edge_consumed_once && continuous_state_not_lost && paused_input_dropped_without_burst;
    let key_frames = input.records.iter().filter(|record| record.consumed_edges.len() > 0 || record.held).take(12).map(|record| InputTickRecord {
        tick: record.tick,
        held: record.held,
        consumed_edges: record.consumed_edges.clone(),
    }).collect();
    InputSummary {
        events,
        consumed_edge_ids: input.consumed,
        dropped_while_paused: input.dropped_while_paused,
        ignored_duplicate_presses: input.ignored_duplicate_presses,
        held_ticks_before_pause,
        held_ticks_after_resume,
        max_active_frame_ms: max_active_frame_ns as f64 / 1_000_000.0,
        max_wall_frame_ms: max_wall_frame_ns as f64 / 1_000_000.0,
        repeated_edge_consumed_once,
        continuous_state_not_lost,
        paused_input_dropped_without_burst,
        pass,
        key_frames,
    }
}

#[derive(Debug, Serialize)]
struct PerformanceSummary {
    profile: String,
    sample_count: usize,
    warmup_samples: usize,
    budget_ms_per_tick_at_60hz: f64,
    p50_us: f64,
    p95_us: f64,
    p99_us: f64,
    max_us: f64,
    p95_budget_share: f64,
    within_60hz_budget: bool,
    workload: String,
}

fn percentile(values: &[u128], percentile: f64) -> f64 {
    let index = ((values.len() as f64 * percentile).ceil() as usize).saturating_sub(1).min(values.len() - 1);
    values[index] as f64 / 1_000.0
}

fn run_performance_probe() -> PerformanceSummary {
    const WARMUP: usize = 100;
    const SAMPLES: usize = 6_000;
    let mut seed = 0.1234_f64;
    let mut work = 0.0_f64;
    for _ in 0..WARMUP {
        work = black_box(proxy_physics_tick(work, &mut seed));
    }
    let mut durations = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let started = Instant::now();
        work = black_box(proxy_physics_tick(work, &mut seed));
        durations.push(started.elapsed().as_nanos());
    }
    durations.sort_unstable();
    let max_us = *durations.last().unwrap_or(&0) as f64 / 1_000.0;
    let p95_us = percentile(&durations, 0.95);
    PerformanceSummary {
        profile: if cfg!(debug_assertions) { "debug".into() } else { "release".into() },
        sample_count: SAMPLES,
        warmup_samples: WARMUP,
        budget_ms_per_tick_at_60hz: SIXTY_HZ_BUDGET_US / 1_000.0,
        p50_us: percentile(&durations, 0.50),
        p95_us,
        p99_us: percentile(&durations, 0.99),
        max_us,
        p95_budget_share: p95_us / SIXTY_HZ_BUDGET_US,
        within_60hz_budget: p95_us <= SIXTY_HZ_BUDGET_US,
        workload: format!("AG-06-equivalent capsule proxy: 256 static AABB candidates, deterministic overlap/normal/integration loop; sink={work:.6}"),
    }
}

fn proxy_physics_tick(mut state: f64, seed: &mut f64) -> f64 {
    for collider in 0..256_u32 {
        *seed = (*seed * 1.618_033_988_75 + 0.017_271_8).fract();
        let x = (collider as f64 * 0.03125 + *seed) - 4.0;
        let z = (collider as f64 * 0.0175 - *seed) - 2.0;
        let distance_sq = x * x + z * z;
        if distance_sq < 9.0 {
            let normal = if distance_sq > 0.000001 { 1.0 / distance_sq.sqrt() } else { 0.0 };
            state = state.mul_add(0.999_7, normal * 0.000_3 + x * 0.000_01 + z * 0.000_01);
        }
    }
    state
}

fn result_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("results").join("tick-ag07.json")
}

fn run_report() -> Value {
    let schedules = vec![run_ten_minute_schedule(30), run_ten_minute_schedule(60)];
    let catch_up = vec![run_catch_up_probe(30), run_catch_up_probe(60)];
    let inputs = vec![run_input_probe(30), run_input_probe(60)];
    let schedule_pass = schedules.iter().all(|summary| summary.pass);
    let catch_up_pass = catch_up.iter().all(|summary| summary.pass);
    let input_pass = inputs.iter().all(|summary| summary.pass);
    let performance = run_performance_probe();
    let performance_pass = performance.within_60hz_budget;
    json!({
        "schema": "continuous_tick_probe_ag07_v1",
        "status": if schedule_pass && catch_up_pass && input_pass { "isolated_tick_probe_passed" } else { "isolated_tick_probe_failed" },
        "pass": schedule_pass && catch_up_pass && input_pass && performance_pass,
        "scope": "isolated scheduler/input/physics-budget proxy; not formal Campaign/Rust continuous-world p95",
        "environment": {
            "os_family": env::consts::FAMILY,
            "os": env::consts::OS,
            "arch": env::consts::ARCH,
            "rust_toolchain": "rustc/cargo 1.98.0 (recorded from verification host)",
            "platform": "x86_64-pc-windows-msvc"
        },
        "contract": {
            "fixed_tick_rates_hz": [30, 60],
            "simulation_duration_s": SIMULATION_SECONDS,
            "max_catch_up_ticks_per_frame": MAX_CATCH_UP,
            "pause_policy": "clear accumulator and held/pending input; background time is not replayed on resume",
            "input_policy": "edge events consume once; held state persists while active; pause-period events are dropped",
            "clock": "monotonic virtual nanoseconds for scheduler tests; Instant for proxy timing samples"
        },
        "source_audit": [
            {"file": "server-rs/src/campaign_main.rs", "lines": "17-24", "finding": "campaign_action accepts one action plus args and calls CampaignStore::action; no seq/client_time/input batch."},
            {"file": "server-rs/src/campaign.rs", "lines": "1-5", "finding": "current campaign documentation describes 160ms movement and 250ms tick cadence."},
            {"file": "server-rs/release-ui/js/app.js", "lines": "188-195", "finding": "campaign_view/action use Tauri invoke or HTTP request/response."},
            {"file": "server-rs/release-ui/js/app.js", "lines": "650-718", "finding": "held input selects a latest pending move; normal actions queue up to three; tick is coalesced to one pending flag while actionBusy."},
            {"file": "server-rs/release-ui/js/app.js", "lines": "730-745", "finding": "pause clears held and queued actions, then attempts a save; resume only restores focus."},
            {"file": "server-rs/release-ui/js/app.js", "lines": "862-864", "finding": "movement timer is 160ms and tick timer is 250ms; these are not fixed 30/60Hz Rust simulation ticks."},
            {"file": "server-rs/release-ui/js/app.js", "lines": "872-873", "finding": "blur/focus/visibility handlers clear held input, so lifecycle can discard held state."},
            {"file": "server-rs/release-ui/js/tps-renderer.js", "lines": "636-663", "finding": "TPS renderer advances local predicted movement and jump from render-frame dt."},
            {"file": "server-rs/release-ui/js/tps-renderer.js", "lines": "868-887", "finding": "Babylon render loop clamps frame dt to 1-80ms and calls updateMovement from the render loop."}
        ],
        "fixed_schedule": schedules,
        "catch_up_and_pause": catch_up,
        "input_edge_and_lifecycle": inputs,
        "performance": performance,
        "thresholds": {
            "simulated_duration_error_s_max": 1.0 / 30.0,
            "max_catch_up_ticks_per_frame": MAX_CATCH_UP,
            "pause_ticks": 0,
            "p95_proxy_budget_share_max": 1.0,
            "edge_duplicate_consumption": 0
        },
        "limitations": [
            "The scheduler uses a deterministic virtual monotonic clock; it does not prove Windows timer jitter or Tauri channel latency.",
            "The performance workload is an AG-06-equivalent capsule/AABB proxy, not the Grey Hive world or formal Rust Campaign.",
            "Debug and release timings are host-local Instant samples and must not be treated as a platform guarantee.",
            "Formal background lifecycle, save barrier, replay, and network/input protocol remain TP-01/TP implementation work."
        ]
    })
}

fn write_report(report: Value) {
    let path = result_path();
    let mut merged = if path.exists() {
        fs::read_to_string(&path).ok().and_then(|text| serde_json::from_str::<Value>(&text).ok()).unwrap_or_else(|| json!({}))
    } else {
        json!({})
    };
    if let (Some(target), Some(source)) = (merged.as_object_mut(), report.as_object()) {
        for (key, value) in source {
            if key != "performance" {
                target.insert(key.clone(), value.clone());
            }
        }
        let profile = report.get("performance").and_then(|value| value.get("profile")).and_then(Value::as_str).unwrap_or("unknown");
        let profiles_snapshot = {
            let profiles = target.entry("performance_profiles").or_insert_with(|| json!({}));
            if let Some(map) = profiles.as_object_mut() {
                map.insert(profile.to_string(), report["performance"].clone());
            }
            profiles.clone()
        };
        target.insert("performance".to_string(), json!({"current_profile": profile, "profiles": profiles_snapshot}));
    } else {
        merged = report;
    }
    fs::create_dir_all(path.parent().expect("result directory")).expect("create result directory");
    fs::write(path, serde_json::to_string_pretty(&merged).expect("serialize AG-07 report")).expect("write AG-07 report");
}

#[test]
fn ag07_fixed_tick_pause_input_and_budget_probe() {
    let report = run_report();
    write_report(report.clone());
    assert!(report["pass"].as_bool().unwrap_or(false), "AG-07 scheduler/input assertions failed: {report}");
}
