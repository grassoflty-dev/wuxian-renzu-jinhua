#![allow(clippy::needless_return)]

#[allow(dead_code)]
#[path = "../../../../server-rs/src/fixed_step.rs"]
mod fixed_step;
#[allow(dead_code)]
#[path = "../../../../server-rs/src/world_v3.rs"]
mod world_v3;

use continuous_runtime_owner_observability_tp17b1::{
    QueueError, QueueItem, RuntimeOwner, SharedBoundedEventQueue,
};
use fixed_step::FixedStepScheduler;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    mem::size_of,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tauri::{ipc::Channel, AppHandle, Manager, State};
use world_v3::{ActorView, DoorView, Transform, Vec3, WorldStateV3, WorldView};

const QUEUE_CAPACITY: usize = 8;
const PAYLOAD_LIMIT: usize = 64;
const SOAK_TICKS: u64 = 36_000;
const EVIDENCE: &str = "real_windows_webview_tauri_2_11_5_tp17c1";
static PROCESS_START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

fn monotonic_ns() -> u64 {
    PROCESS_START
        .get_or_init(Instant::now)
        .elapsed()
        .as_nanos()
        .min(u128::from(u64::MAX)) as u64
}

fn monotonic_ms() -> f64 {
    monotonic_ns() as f64 / 1_000_000.0
}

#[cfg(windows)]
#[repr(C)]
struct ProcessMemoryCountersEx {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
    private_usage: usize,
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
}

#[cfg(windows)]
#[link(name = "psapi")]
extern "system" {
    fn GetProcessMemoryInfo(
        process: *mut std::ffi::c_void,
        counters: *mut ProcessMemoryCountersEx,
        size: u32,
    ) -> i32;
}

fn memory_sample() -> Value {
    #[cfg(windows)]
    {
        let mut counters = ProcessMemoryCountersEx {
            cb: size_of::<ProcessMemoryCountersEx>() as u32,
            page_fault_count: 0,
            peak_working_set_size: 0,
            working_set_size: 0,
            quota_peak_paged_pool_usage: 0,
            quota_paged_pool_usage: 0,
            quota_peak_non_paged_pool_usage: 0,
            quota_non_paged_pool_usage: 0,
            pagefile_usage: 0,
            peak_pagefile_usage: 0,
            private_usage: 0,
        };
        let success = unsafe {
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut counters,
                size_of::<ProcessMemoryCountersEx>() as u32,
            ) != 0
        };
        if success {
            return json!({
                "workingSetBytes": counters.working_set_size as u64,
                "privateBytes": counters.private_usage as u64,
                "peakWorkingSetBytes": counters.peak_working_set_size as u64,
                "sampleApi": "Windows PSAPI GetProcessMemoryInfo",
                "evidenceLevel": "direct_process_api",
                "sampledAtMs": monotonic_ms()
            });
        }
        return json!({
            "workingSetBytes": null,
            "privateBytes": null,
            "peakWorkingSetBytes": null,
            "sampleApi": "Windows PSAPI GetProcessMemoryInfo failed",
            "evidenceLevel": "unavailable_api_error",
            "sampledAtMs": monotonic_ms()
        });
    }
    #[cfg(not(windows))]
    {
        json!({
            "workingSetBytes": null,
            "privateBytes": null,
            "peakWorkingSetBytes": null,
            "sampleApi": "Windows-only probe unavailable on this target",
            "evidenceLevel": "unavailable_non_windows",
            "sampledAtMs": monotonic_ms()
        })
    }
}

fn memory_number(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn merge_peak(peak: &mut Value, candidate: Value) {
    let peak_working = memory_number(peak, "workingSetBytes");
    let candidate_working = memory_number(&candidate, "workingSetBytes");
    let peak_private = memory_number(peak, "privateBytes");
    let candidate_private = memory_number(&candidate, "privateBytes");
    if candidate_working > peak_working || candidate_private > peak_private {
        *peak = candidate;
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionInfo {
    session_id: String,
    generation: u64,
    world_epoch: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueueItemInput {
    epoch: u64,
    tick: u64,
    event_id: String,
    enqueue_sequence: u64,
    payload_bytes: usize,
    #[serde(default)]
    payload: String,
}

#[derive(Debug)]
enum OwnerCommand {
    Enqueue {
        item: QueueItemInput,
        reply: Sender<Value>,
    },
    Drain {
        reply: Sender<Value>,
    },
    Pause {
        reply: Sender<Value>,
    },
    Resume {
        reply: Sender<Value>,
    },
    Advance {
        delta_ns: u64,
        reply: Sender<Value>,
    },
    QueueCloseProbe {
        reply: Sender<Value>,
    },
    Reentry {
        reply: Sender<Value>,
    },
    ReplaceWorld {
        world_epoch: u64,
        reply: Sender<Value>,
    },
    Lifecycle30 {
        reply: Sender<Value>,
    },
    RunSoak {
        ticks: u64,
        reply: Sender<Value>,
    },
    Ack {
        epoch: u64,
        ack_seq: u64,
        reply: Sender<Value>,
    },
    Diagnostics {
        reply: Sender<Value>,
    },
    Stop {
        reply: Sender<Value>,
    },
}

struct ActiveSession {
    info: SessionInfo,
    tx: Sender<OwnerCommand>,
    join: Option<thread::JoinHandle<Value>>,
}

#[derive(Default)]
struct Registry {
    active: Option<ActiveSession>,
}

struct RuntimeState {
    registry: Mutex<Registry>,
    closing: AtomicBool,
    next_session: AtomicU64,
    next_generation: AtomicU64,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            registry: Mutex::new(Registry::default()),
            closing: AtomicBool::new(false),
            next_session: AtomicU64::new(1),
            next_generation: AtomicU64::new(1),
        }
    }
}

fn result_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../results/tauri-runtime-observability-tp17c1.json")
}

fn make_world(epoch: u64) -> WorldStateV3 {
    let player_transform = Transform::new(Vec3::zero(), 0.0).expect("finite player transform");
    let actor_transform = Transform::new(
        Vec3::new(2.0, 0.0, 2.0).expect("finite actor position"),
        0.0,
    )
    .expect("finite actor transform");
    let actor = ActorView::new(
        "tp17c1-synthetic-actor",
        "probe",
        actor_transform,
        Vec3::zero(),
    )
    .expect("valid synthetic actor");
    let door = DoorView::new("tp17c1-synthetic-door", true).expect("valid synthetic door");
    WorldStateV3::new(
        "tp17c1-synthetic-world",
        epoch,
        0,
        0,
        player_transform,
        Vec3::zero(),
        vec![actor],
        vec![door],
    )
    .expect("valid synthetic world")
}

fn queue_error_value(error: QueueError) -> Value {
    let (code, detail) = match error {
        QueueError::Closed => ("E_QUEUE_CLOSED", json!({})),
        QueueError::StaleEpoch { expected, actual } => (
            "E_QUEUE_STALE_EPOCH",
            json!({"expectedEpoch": expected, "actualEpoch": actual}),
        ),
        QueueError::Full { capacity } => {
            ("E_QUEUE_FULL_REJECT_NEWEST", json!({"capacity": capacity}))
        }
        QueueError::PayloadTooLarge { limit, actual } => (
            "E_QUEUE_PAYLOAD_TOO_LARGE",
            json!({"limit": limit, "actual": actual}),
        ),
        QueueError::DuplicateSequence(sequence) => (
            "E_QUEUE_DUPLICATE_SEQUENCE",
            json!({"enqueueSequence": sequence}),
        ),
        QueueError::EmptyEventId => ("E_QUEUE_EMPTY_EVENT_ID", json!({})),
    };
    json!({"ok": false, "errorCode": code, "detail": detail})
}

fn queue_metrics_from_snapshot(owner: &RuntimeOwner) -> Value {
    let snapshot = owner.snapshot();
    json!({
        "length": snapshot.queue_length.value,
        "capacity": snapshot.queue_capacity.value,
        "highWater": snapshot.queue_high_water.value,
        "dropped": snapshot.queue_dropped.value,
        "rejected": snapshot.queue_rejected.value
    })
}

fn snapshot_payload(
    owner: &RuntimeOwner,
    callback_send_count: u64,
    callback_error_count: u64,
    js_ack_count: u64,
) -> Value {
    let owner_snapshot = owner.snapshot();
    json!({
        "snapshot": owner_snapshot,
        "queueMetrics": queue_metrics_from_snapshot(owner),
        "lifecycle": {
            "created": owner_snapshot.created_count,
            "disposed": owner_snapshot.disposed_count,
            "running": owner_snapshot.running
        },
        "channel": {
            "rustSendCount": callback_send_count,
            "rustSendErrorCount": callback_error_count,
            "jsAckCount": js_ack_count,
            "provenance": "wrapper_owned_counters"
        },
        "memory": memory_sample(),
        "evidenceLevel": EVIDENCE
    })
}

struct OwnerLoop {
    info: SessionInfo,
    receiver: Receiver<OwnerCommand>,
    channel: Channel<WorldView>,
    owner: RuntimeOwner,
    scheduler: FixedStepScheduler,
    world: WorldStateV3,
    synthetic_now_ns: u64,
    callback_send_count: u64,
    callback_error_count: u64,
    js_ack_count: u64,
    processed_event_count: u64,
}

impl OwnerLoop {
    fn send_world(&mut self, ack_seq: u64) {
        let result = self.channel.send(self.world.view(ack_seq));
        self.callback_send_count = self.callback_send_count.saturating_add(1);
        if result.is_err() {
            self.callback_error_count = self.callback_error_count.saturating_add(1);
        }
    }

    fn process_plan(&mut self, plan: &fixed_step::TickPlan, send_each_tick: bool) {
        for (index, simulation_time_ns) in plan
            .tick_indices()
            .iter()
            .zip(plan.simulation_times_nanos())
        {
            let tick = index.saturating_add(1);
            self.world
                .advance_clock(tick, simulation_time_ns / 1_000_000)
                .expect("synthetic world clock is monotonic");
            if send_each_tick {
                self.send_world(tick);
            }
        }
    }

    fn advance(&mut self, delta_ns: u64, send_each_tick: bool) -> Value {
        self.synthetic_now_ns = self.synthetic_now_ns.saturating_add(delta_ns);
        match self.scheduler.advance(self.synthetic_now_ns) {
            Ok(plan) => {
                self.process_plan(&plan, send_each_tick);
                json!({
                    "ok": true,
                    "tickCount": plan.tick_count(),
                    "droppedTimeNs": plan.dropped_time_nanos(),
                    "paused": plan.paused(),
                    "totalTicks": self.scheduler.total_ticks(),
                    "snapshot": snapshot_payload(&self.owner, self.callback_send_count, self.callback_error_count, self.js_ack_count)
                })
            }
            Err(error) => {
                json!({"ok": false, "errorCode": "E_SCHEDULER", "error": error.to_string()})
            }
        }
    }

    fn enqueue(&mut self, input: QueueItemInput) -> Value {
        let item = QueueItem {
            epoch: input.epoch,
            tick: input.tick,
            event_id: input.event_id,
            enqueue_sequence: input.enqueue_sequence,
            payload_bytes: input.payload_bytes,
            payload: input.payload,
        };
        match self.owner.enqueue(item) {
            Ok(()) => json!({
                "ok": true,
                "ack": self.owner.snapshot().processed_event_entries.value,
                "snapshot": snapshot_payload(&self.owner, self.callback_send_count, self.callback_error_count, self.js_ack_count)
            }),
            Err(error) => queue_error_value(error),
        }
    }

    fn drain(&mut self) -> Value {
        let processed = self.owner.drain_process();
        self.processed_event_count = self
            .processed_event_count
            .saturating_add(processed.len() as u64);
        json!({
            "ok": true,
            "processedEventIds": processed,
            "processedEventCount": self.processed_event_count,
            "snapshot": snapshot_payload(&self.owner, self.callback_send_count, self.callback_error_count, self.js_ack_count)
        })
    }

    fn queue_close_probe(&mut self) -> Value {
        let epoch = self.info.world_epoch;
        let _ = self.owner.exit_world();
        let rejected = self.owner.enqueue(QueueItem {
            epoch,
            tick: 9001,
            event_id: "after-close".to_owned(),
            enqueue_sequence: 9001,
            payload_bytes: 12,
            payload: "after-close".to_owned(),
        });
        let after_close = snapshot_payload(
            &self.owner,
            self.callback_send_count,
            self.callback_error_count,
            self.js_ack_count,
        );
        self.owner
            .enter_world(epoch, 1)
            .expect("owner reopens after close probe");
        json!({
            "ok": true,
            "closedEnqueue": rejected.err().map(queue_error_value).unwrap_or_else(|| json!({"ok": true})),
            "afterClose": after_close,
            "reopened": snapshot_payload(&self.owner, self.callback_send_count, self.callback_error_count, self.js_ack_count)
        })
    }

    fn replace_world(&mut self, world_epoch: u64) -> Value {
        let _ = self.owner.exit_world();
        self.info.world_epoch = world_epoch;
        self.owner
            .enter_world(world_epoch, 1)
            .expect("owner enters replacement world");
        self.scheduler = FixedStepScheduler::with_frequency(60, 0).expect("valid scheduler");
        self.synthetic_now_ns = 0;
        self.world = make_world(world_epoch);
        json!({
            "ok": true,
            "worldEpoch": world_epoch,
            "snapshot": snapshot_payload(&self.owner, self.callback_send_count, self.callback_error_count, self.js_ack_count)
        })
    }

    fn reentry_probe(&self) -> Value {
        let queue = SharedBoundedEventQueue::new(self.info.world_epoch, 2, PAYLOAD_LIMIT);
        let first = QueueItem {
            epoch: self.info.world_epoch,
            tick: 1,
            event_id: "reentry-first".to_owned(),
            enqueue_sequence: 1,
            payload_bytes: 10,
            payload: "first".to_owned(),
        };
        let second = QueueItem {
            epoch: self.info.world_epoch,
            tick: 2,
            event_id: "reentry-second".to_owned(),
            enqueue_sequence: 2,
            payload_bytes: 10,
            payload: "second".to_owned(),
        };
        let first_result = queue.try_enqueue(first);
        let mut callback_ids = Vec::new();
        let mut reentry_result = None;
        queue.drain_with(|item| {
            callback_ids.push(item.event_id);
            reentry_result = Some(queue.try_enqueue(second.clone()).map(|_| "accepted"));
        });
        json!({
            "firstAccepted": first_result.is_ok(),
            "callbackIds": callback_ids,
            "reentryAccepted": reentry_result.map(|result| result.is_ok()).unwrap_or(false),
            "queueMetrics": queue.metrics(),
            "lockReleasedBeforeCallback": true,
            "evidenceLevel": EVIDENCE
        })
    }

    fn lifecycle_30(&mut self) -> Value {
        let before = snapshot_payload(
            &self.owner,
            self.callback_send_count,
            self.callback_error_count,
            self.js_ack_count,
        );
        let epoch = self.info.world_epoch;
        let mut after_exit = Value::Null;
        for _ in 0..30 {
            self.owner.exit_world().expect("active owner exits");
            after_exit = snapshot_payload(
                &self.owner,
                self.callback_send_count,
                self.callback_error_count,
                self.js_ack_count,
            );
            self.owner
                .enter_world(epoch, 1)
                .expect("owner re-enters after cycle");
        }
        json!({
            "cycles": 30,
            "before": before,
            "afterFinalExit": after_exit,
            "reopened": snapshot_payload(&self.owner, self.callback_send_count, self.callback_error_count, self.js_ack_count),
            "evidenceLevel": EVIDENCE
        })
    }

    fn run_soak(&mut self, ticks: u64) -> Value {
        let before = snapshot_payload(
            &self.owner,
            self.callback_send_count,
            self.callback_error_count,
            self.js_ack_count,
        );
        let queue_rejected_before = self.owner.snapshot().queue_rejected.value.unwrap_or(0);
        let mut peak_memory = before["memory"].clone();
        let mut peak_snapshot = before["snapshot"].clone();
        let mut latencies = Vec::with_capacity(ticks as usize);
        self.scheduler = FixedStepScheduler::with_frequency(60, 0).expect("valid scheduler");
        self.synthetic_now_ns = 0;
        self.world = make_world(self.info.world_epoch);
        let started = Instant::now();
        let mut sent_this_run = 0_u64;
        for tick_index in 0..ticks {
            let tick_start = Instant::now();
            self.synthetic_now_ns = self.synthetic_now_ns.saturating_add(16_666_667);
            let plan = self
                .scheduler
                .advance(self.synthetic_now_ns)
                .expect("soak clock is monotonic");
            self.process_plan(&plan, false);
            let event_id = format!("soak-{tick_index}");
            let accepted = self
                .owner
                .enqueue(QueueItem {
                    epoch: self.info.world_epoch,
                    tick: tick_index,
                    event_id,
                    enqueue_sequence: tick_index.saturating_add(1),
                    payload_bytes: 16,
                    payload: "bounded-soak".to_owned(),
                })
                .is_ok();
            if accepted {
                let processed = self.owner.drain_process();
                self.processed_event_count = self
                    .processed_event_count
                    .saturating_add(processed.len() as u64);
            }
            self.owner.add_cache(&format!("cache-{tick_index}"));
            if tick_index % 1000 == 0 {
                self.owner.add_reward(&format!("reward-{tick_index}"));
            }
            if (tick_index + 1) % 60 == 0 {
                self.send_world(tick_index + 1);
                sent_this_run = sent_this_run.saturating_add(1);
            }
            latencies.push(tick_start.elapsed().as_nanos() as u64);
            if tick_index % 256 == 0 {
                let current = memory_sample();
                merge_peak(&mut peak_memory, current);
                peak_snapshot =
                    serde_json::to_value(self.owner.snapshot()).expect("snapshot serializes");
            }
        }
        let wall_time_ms = started.elapsed().as_secs_f64() * 1000.0;
        let after_tick_snapshot = snapshot_payload(
            &self.owner,
            self.callback_send_count,
            self.callback_error_count,
            self.js_ack_count,
        );
        let _ = self.owner.exit_world();
        let after_exit = snapshot_payload(
            &self.owner,
            self.callback_send_count,
            self.callback_error_count,
            self.js_ack_count,
        );
        let queue_rejected_after = after_exit["queueMetrics"]["rejected"].as_u64().unwrap_or(0);
        self.owner
            .enter_world(self.info.world_epoch, 1)
            .expect("owner reopens after soak");
        let mut sorted = latencies;
        sorted.sort_unstable();
        let percentile = |fraction: f64| -> u64 {
            if sorted.is_empty() {
                return 0;
            }
            let index = ((sorted.len() - 1) as f64 * fraction).round() as usize;
            sorted[index]
        };
        json!({
            "ticks": ticks,
            "before": before,
            "peak": {
                "snapshot": peak_snapshot,
                "memory": peak_memory
            },
            "afterTick": after_tick_snapshot,
            "afterExit": after_exit,
            "queueRejectedBefore": queue_rejected_before,
            "queueRejectedAfter": queue_rejected_after,
            "queueRejectedDelta": queue_rejected_after.saturating_sub(queue_rejected_before),
            "sentThisRun": sent_this_run,
            "wallTimeMs": wall_time_ms,
            "tickLatencyNs": {
                "count": sorted.len(),
                "p50": percentile(0.50),
                "p95": percentile(0.95),
                "p99": percentile(0.99),
                "max": sorted.last().copied().unwrap_or(0)
            },
            "scheduler": {
                "totalTicks": self.scheduler.total_ticks(),
                "totalDroppedTimeNs": self.scheduler.total_dropped_time_nanos()
            },
            "evidenceLevel": EVIDENCE
        })
    }

    fn run(mut self) -> Value {
        loop {
            match self.receiver.recv_timeout(Duration::from_millis(10)) {
                Ok(command) => match command {
                    OwnerCommand::Enqueue { item, reply } => {
                        let _ = reply.send(self.enqueue(item));
                    }
                    OwnerCommand::Drain { reply } => {
                        let _ = reply.send(self.drain());
                    }
                    OwnerCommand::Pause { reply } => {
                        let result = self
                            .scheduler
                            .pause(self.synthetic_now_ns)
                            .map(|_| json!({"ok": true, "paused": true}))
                            .unwrap_or_else(
                                |error| json!({"ok": false, "error": error.to_string()}),
                            );
                        let _ = reply.send(result);
                    }
                    OwnerCommand::Resume { reply } => {
                        let result = self
                            .scheduler
                            .resume(self.synthetic_now_ns)
                            .map(|_| json!({"ok": true, "paused": false}))
                            .unwrap_or_else(
                                |error| json!({"ok": false, "error": error.to_string()}),
                            );
                        let _ = reply.send(result);
                    }
                    OwnerCommand::Advance { delta_ns, reply } => {
                        let _ = reply.send(self.advance(delta_ns, true));
                    }
                    OwnerCommand::QueueCloseProbe { reply } => {
                        let _ = reply.send(self.queue_close_probe());
                    }
                    OwnerCommand::Reentry { reply } => {
                        let _ = reply.send(self.reentry_probe());
                    }
                    OwnerCommand::ReplaceWorld { world_epoch, reply } => {
                        let _ = reply.send(self.replace_world(world_epoch));
                    }
                    OwnerCommand::Lifecycle30 { reply } => {
                        let _ = reply.send(self.lifecycle_30());
                    }
                    OwnerCommand::RunSoak { ticks, reply } => {
                        let _ = reply.send(self.run_soak(ticks));
                    }
                    OwnerCommand::Ack {
                        epoch,
                        ack_seq,
                        reply,
                    } => {
                        self.js_ack_count = self.js_ack_count.saturating_add(1);
                        let _ = reply.send(json!({
                            "ok": true,
                            "epoch": epoch,
                            "ackSeq": ack_seq,
                            "jsAckCount": self.js_ack_count
                        }));
                    }
                    OwnerCommand::Diagnostics { reply } => {
                        let _ = reply.send(snapshot_payload(
                            &self.owner,
                            self.callback_send_count,
                            self.callback_error_count,
                            self.js_ack_count,
                        ));
                    }
                    OwnerCommand::Stop { reply } => {
                        let _ = self.owner.exit_world();
                        let summary = json!({
                            "sessionId": self.info.session_id,
                            "generation": self.info.generation,
                            "worldEpoch": self.info.world_epoch,
                            "stoppedCleanly": true,
                            "final": snapshot_payload(&self.owner, self.callback_send_count, self.callback_error_count, self.js_ack_count),
                            "rustSendCount": self.callback_send_count,
                            "rustSendErrorCount": self.callback_error_count,
                            "jsAckCount": self.js_ack_count,
                            "threadOwnerStopped": true,
                            "evidenceLevel": EVIDENCE
                        });
                        let _ = reply.send(summary.clone());
                        return summary;
                    }
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    let _ = self.owner.exit_world();
                    return json!({
                        "sessionId": self.info.session_id,
                        "stoppedCleanly": false,
                        "threadOwnerStopped": true,
                        "final": snapshot_payload(&self.owner, self.callback_send_count, self.callback_error_count, self.js_ack_count),
                        "evidenceLevel": EVIDENCE
                    });
                }
            }
        }
    }
}

fn spawn_session(
    state: &RuntimeState,
    channel: Channel<WorldView>,
    world_epoch: u64,
) -> ActiveSession {
    let session_number = state.next_session.fetch_add(1, Ordering::AcqRel);
    let generation = state.next_generation.fetch_add(1, Ordering::AcqRel);
    let info = SessionInfo {
        session_id: format!("tp17c1-session-{session_number}"),
        generation,
        world_epoch,
    };
    let thread_info = info.clone();
    let (tx, receiver) = mpsc::channel();
    let join = thread::spawn(move || {
        let mut owner = RuntimeOwner::new(world_epoch, QUEUE_CAPACITY, PAYLOAD_LIMIT);
        owner.enter_world(world_epoch, 1).expect("owner starts");
        OwnerLoop {
            info: thread_info,
            receiver,
            channel,
            owner,
            scheduler: FixedStepScheduler::with_frequency(60, 0).expect("valid scheduler"),
            world: make_world(world_epoch),
            synthetic_now_ns: 0,
            callback_send_count: 0,
            callback_error_count: 0,
            js_ack_count: 0,
            processed_event_count: 0,
        }
        .run()
    });
    ActiveSession {
        info,
        tx,
        join: Some(join),
    }
}

fn owner_sender(session_id: &str, state: &RuntimeState) -> Result<Sender<OwnerCommand>, String> {
    let registry = state
        .registry
        .lock()
        .map_err(|_| "E_REGISTRY_POISONED".to_owned())?;
    let active = registry
        .active
        .as_ref()
        .ok_or_else(|| "E_STALE_SESSION".to_owned())?;
    if active.info.session_id != session_id {
        return Err("E_STALE_SESSION".to_owned());
    }
    Ok(active.tx.clone())
}

fn stop_active(mut active: ActiveSession) -> Result<Value, String> {
    let (reply_tx, reply_rx) = mpsc::channel();
    active
        .tx
        .send(OwnerCommand::Stop { reply: reply_tx })
        .map_err(|_| "E_OWNER_DISCONNECTED".to_owned())?;
    let summary = reply_rx
        .recv_timeout(Duration::from_secs(10))
        .map_err(|_| "E_OWNER_STOP_TIMEOUT".to_owned())?;
    if let Some(join) = active.join.take() {
        join.join().map_err(|_| "E_OWNER_THREAD_PANIC".to_owned())?;
    }
    Ok(summary)
}

fn send_command(
    tx: Sender<OwnerCommand>,
    build: impl FnOnce(Sender<Value>) -> OwnerCommand,
) -> Result<Value, String> {
    let (reply_tx, reply_rx) = mpsc::channel();
    tx.send(build(reply_tx))
        .map_err(|_| "E_OWNER_DISCONNECTED".to_owned())?;
    reply_rx
        .recv_timeout(Duration::from_secs(30))
        .map_err(|_| "E_OWNER_COMMAND_TIMEOUT".to_owned())
}

fn write_report(report: Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(&report).map_err(|_| "E_REPORT_ENCODE".to_owned())?;
    fs::write(result_path(), format!("{text}\n")).map_err(|error| error.to_string())
}

#[tauri::command]
fn tp17c1_environment(user_agent: String) -> Value {
    let webview_version = user_agent
        .split(" Edg/")
        .nth(1)
        .or_else(|| user_agent.split(" Chrome/").nth(1))
        .map(|value| value.split_whitespace().next().unwrap_or(value).to_owned());
    json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "platform": "windows-webview2",
        "tauri": "2.11.5",
        "webviewUserAgent": user_agent,
        "webviewVersion": webview_version,
        "memory": memory_sample(),
        "processApi": "GetProcessMemoryInfo",
        "localDevelopmentOnly": true,
        "evidenceLevel": EVIDENCE
    })
}

#[tauri::command]
fn tp17c1_start(
    channel: Channel<WorldView>,
    world_epoch: u64,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    if state.closing.load(Ordering::Acquire) {
        return Err("E_CLOSING".to_owned());
    }
    let mut registry = state
        .registry
        .lock()
        .map_err(|_| "E_REGISTRY_POISONED".to_owned())?;
    if registry.active.is_some() {
        return Err("E_ALREADY_RUNNING".to_owned());
    }
    let active = spawn_session(&state, channel, world_epoch);
    let info = active.info.clone();
    registry.active = Some(active);
    Ok(json!({
        "session": info,
        "queue": {"capacity": QUEUE_CAPACITY, "payloadLimit": PAYLOAD_LIMIT, "overflowPolicy": "reject_newest"},
        "memory": memory_sample(),
        "evidenceLevel": EVIDENCE
    }))
}

#[tauri::command]
fn tp17c1_enqueue(
    session_id: String,
    item: QueueItemInput,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let tx = owner_sender(&session_id, &state)?;
    send_command(tx, |reply| OwnerCommand::Enqueue { item, reply })
}

#[tauri::command]
fn tp17c1_drain(session_id: String, state: State<'_, RuntimeState>) -> Result<Value, String> {
    let tx = owner_sender(&session_id, &state)?;
    send_command(tx, |reply| OwnerCommand::Drain { reply })
}

#[tauri::command]
fn tp17c1_pause(session_id: String, state: State<'_, RuntimeState>) -> Result<Value, String> {
    let tx = owner_sender(&session_id, &state)?;
    send_command(tx, |reply| OwnerCommand::Pause { reply })
}

#[tauri::command]
fn tp17c1_resume(session_id: String, state: State<'_, RuntimeState>) -> Result<Value, String> {
    let tx = owner_sender(&session_id, &state)?;
    send_command(tx, |reply| OwnerCommand::Resume { reply })
}

#[tauri::command]
fn tp17c1_advance(
    session_id: String,
    delta_ns: u64,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let tx = owner_sender(&session_id, &state)?;
    send_command(tx, |reply| OwnerCommand::Advance { delta_ns, reply })
}

#[tauri::command]
fn tp17c1_queue_close_probe(
    session_id: String,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let tx = owner_sender(&session_id, &state)?;
    send_command(tx, |reply| OwnerCommand::QueueCloseProbe { reply })
}

#[tauri::command]
fn tp17c1_replace_world(
    session_id: String,
    world_epoch: u64,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let tx = owner_sender(&session_id, &state)?;
    send_command(tx, |reply| OwnerCommand::ReplaceWorld {
        world_epoch,
        reply,
    })
}

#[tauri::command]
fn tp17c1_lifecycle_30(
    session_id: String,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let tx = owner_sender(&session_id, &state)?;
    send_command(tx, |reply| OwnerCommand::Lifecycle30 { reply })
}

#[tauri::command]
fn tp17c1_run_soak(
    session_id: String,
    ticks: u64,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    if ticks != SOAK_TICKS {
        return Err(format!("E_SOAK_TICK_BUDGET_EXPECTED_{SOAK_TICKS}"));
    }
    let tx = owner_sender(&session_id, &state)?;
    send_command(tx, |reply| OwnerCommand::RunSoak { ticks, reply })
}

#[tauri::command]
fn tp17c1_reentry(session_id: String, state: State<'_, RuntimeState>) -> Result<Value, String> {
    let tx = owner_sender(&session_id, &state)?;
    send_command(tx, |reply| OwnerCommand::Reentry { reply })
}

#[tauri::command]
fn tp17c1_snapshot(session_id: String, state: State<'_, RuntimeState>) -> Result<Value, String> {
    let tx = owner_sender(&session_id, &state)?;
    send_command(tx, |reply| OwnerCommand::Diagnostics { reply })
}

#[tauri::command]
fn tp17c1_ack(
    session_id: String,
    epoch: u64,
    ack_seq: u64,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let tx = owner_sender(&session_id, &state)?;
    send_command(tx, |reply| OwnerCommand::Ack {
        epoch,
        ack_seq,
        reply,
    })
}

#[tauri::command]
fn tp17c1_replace_channel(
    session_id: String,
    channel: Channel<WorldView>,
    world_epoch: u64,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let old = {
        let mut registry = state
            .registry
            .lock()
            .map_err(|_| "E_REGISTRY_POISONED".to_owned())?;
        let active = registry
            .active
            .as_ref()
            .ok_or_else(|| "E_STALE_SESSION".to_owned())?;
        if active.info.session_id != session_id {
            return Err("E_STALE_SESSION".to_owned());
        }
        registry
            .active
            .take()
            .ok_or_else(|| "E_STALE_SESSION".to_owned())?
    };
    let old_summary = stop_active(old)?;
    let mut registry = state
        .registry
        .lock()
        .map_err(|_| "E_REGISTRY_POISONED".to_owned())?;
    let active = spawn_session(&state, channel, world_epoch);
    let info = active.info.clone();
    registry.active = Some(active);
    Ok(json!({
        "oldSessionId": session_id,
        "newSession": info,
        "oldSummary": old_summary,
        "replacementBarrier": "old stop/join/channel drop completed before new owner insertion",
        "memory": memory_sample(),
        "evidenceLevel": EVIDENCE
    }))
}

#[tauri::command]
fn tp17c1_close(session_id: String, state: State<'_, RuntimeState>) -> Result<Value, String> {
    let active = {
        let mut registry = state
            .registry
            .lock()
            .map_err(|_| "E_REGISTRY_POISONED".to_owned())?;
        let active = registry
            .active
            .as_ref()
            .ok_or_else(|| "E_STALE_SESSION".to_owned())?;
        if active.info.session_id != session_id {
            return Err("E_STALE_SESSION".to_owned());
        }
        registry
            .active
            .take()
            .ok_or_else(|| "E_STALE_SESSION".to_owned())?
    };
    let summary = stop_active(active)?;
    Ok(json!({
        "closed": true,
        "summary": summary,
        "memory": memory_sample(),
        "channelCloseSentinel": "not_publicly_exposed_by_tauri_channel",
        "evidenceLevel": EVIDENCE
    }))
}

#[tauri::command]
fn tp17c1_record_result(report: Value) -> Result<Value, String> {
    write_report(report)?;
    Ok(json!({"status": "recorded", "path": result_path()}))
}

#[tauri::command]
fn tp17c1_finish(app: AppHandle, state: State<'_, RuntimeState>) -> Result<(), String> {
    state.closing.store(true, Ordering::Release);
    let active = {
        let mut registry = state
            .registry
            .lock()
            .map_err(|_| "E_REGISTRY_POISONED".to_owned())?;
        registry.active.take()
    };
    if let Some(active) = active {
        let _ = stop_active(active)?;
    }
    app.exit(0);
    Ok(())
}

fn main() {
    PROCESS_START.get_or_init(Instant::now);
    tauri::Builder::default()
        .manage(RuntimeState::default())
        .invoke_handler(tauri::generate_handler![
            tp17c1_environment,
            tp17c1_start,
            tp17c1_enqueue,
            tp17c1_drain,
            tp17c1_pause,
            tp17c1_resume,
            tp17c1_advance,
            tp17c1_queue_close_probe,
            tp17c1_replace_world,
            tp17c1_lifecycle_30,
            tp17c1_run_soak,
            tp17c1_reentry,
            tp17c1_snapshot,
            tp17c1_ack,
            tp17c1_replace_channel,
            tp17c1_close,
            tp17c1_record_result,
            tp17c1_finish,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(state) = window.try_state::<RuntimeState>() {
                    state.closing.store(true, Ordering::Release);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running TP-17C1 Tauri application");
}
