#![allow(clippy::needless_return)]

#[path = "../../../../server-rs/src/continuous_input.rs"]
mod continuous_input;
#[path = "../../../../server-rs/src/fixed_step.rs"]
mod fixed_step;
#[path = "../../../../server-rs/src/world_v3.rs"]
mod world_v3;

use continuous_input::{DropReason, InputFrame, InputReceiver, ReceiveOutcome};
use fixed_step::FixedStepScheduler;
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
        Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tauri::{ipc::Channel, AppHandle, Manager, State};
use world_v3::{ActorView, DoorView, Transform, Vec3, WorldStateV3, WorldView};

const RUNTIME_PROTOCOL: &str = "continuous-ipc";
const RUNTIME_VERSION: u32 = 1;
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionInfo {
    session_id: String,
    generation: u64,
    world_epoch: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OwnerSummary {
    session_id: String,
    generation: u64,
    world_epoch: u64,
    stop_requested_at_ms: f64,
    input_records: Vec<Value>,
    callback_send_records: Vec<Value>,
    diagnostics: Value,
    command_counts: Value,
    stopped_cleanly: bool,
}

#[derive(Debug)]
enum OwnerCommand {
    Input {
        frame: InputFrame,
        payload_bytes: usize,
        receipt_ns: u64,
        reply: Sender<Result<Value, String>>,
    },
    Control {
        kind: &'static str,
        receipt_ns: u64,
        reply: Sender<Result<Value, String>>,
    },
    Diagnostics {
        reply: Sender<Result<Value, String>>,
    },
    Stop {
        reply: Sender<Result<Value, String>>,
    },
}

struct ActiveSession {
    info: SessionInfo,
    tx: Sender<OwnerCommand>,
    join: Option<thread::JoinHandle<OwnerSummary>>,
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

fn runtime_error(code: &str) -> String {
    code.to_owned()
}

fn result_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../results/tauri-input-runtime-tp06b1.json")
}

fn make_world(epoch: u64) -> WorldStateV3 {
    let position = Vec3::new(0.0, 0.0, 0.0).expect("finite synthetic position");
    let velocity = Vec3::zero();
    let transform = Transform::new(position, 0.0).expect("finite synthetic transform");
    let actor_transform = Transform::new(
        Vec3::new(2.0, 0.0, 2.0).expect("finite actor position"),
        0.0,
    )
    .expect("finite actor transform");
    let actor = ActorView::new("tp06b1-synthetic-actor", "probe", actor_transform, velocity)
        .expect("non-empty synthetic actor");
    let door = DoorView::new("tp06b1-synthetic-door", true).expect("non-empty synthetic door");
    WorldStateV3::new(
        "tp06b1-synthetic",
        epoch,
        0,
        0,
        transform,
        velocity,
        vec![actor],
        vec![door],
    )
    .expect("valid synthetic world")
}

fn drop_reason_code(reason: DropReason) -> &'static str {
    match reason {
        DropReason::PayloadTooLarge => "payload_too_large",
        DropReason::InvalidPayload => "invalid_payload",
        DropReason::WrongProtocol => "wrong_protocol",
        DropReason::WrongVersion => "wrong_version",
        DropReason::WrongEpoch => "wrong_epoch",
        DropReason::Paused => "paused",
        DropReason::ClockRegression => "clock_regression",
        DropReason::DuplicateOrOldSequence => "duplicate_or_old_sequence",
        DropReason::GapOverflow => "gap_overflow",
        DropReason::SequenceOverflow => "sequence_overflow",
    }
}

fn outcome_value(outcome: ReceiveOutcome) -> Value {
    match outcome {
        ReceiveOutcome::Applied {
            applied_frames,
            ack_seq,
        } => json!({
            "status": "applied",
            "appliedFrames": applied_frames,
            "ackSeq": ack_seq,
        }),
        ReceiveOutcome::Buffered {
            pending_frames,
            ack_seq,
        } => json!({
            "status": "buffered",
            "pendingFrames": pending_frames,
            "ackSeq": ack_seq,
        }),
        ReceiveOutcome::Dropped { reason, ack_seq } => json!({
            "status": "dropped",
            "errorCode": drop_reason_code(reason),
            "ackSeq": ack_seq,
        }),
    }
}

fn command_counts_value(counts: &[(&str, u64)]) -> Value {
    let mut object = serde_json::Map::new();
    for (name, count) in counts {
        object.insert((*name).to_owned(), json!(count));
    }
    Value::Object(object)
}

fn owner_loop(
    info: SessionInfo,
    receiver: Receiver<OwnerCommand>,
    channel: Channel<WorldView>,
) -> OwnerSummary {
    let owner_start = Instant::now();
    let mut scheduler = FixedStepScheduler::with_frequency(60, 0).expect("60Hz is valid");
    let mut input_receiver = InputReceiver::new(info.world_epoch);
    let mut world = make_world(info.world_epoch);
    let mut input_records = Vec::new();
    let mut callback_send_records = Vec::new();
    let mut consumed_edge_ids = Vec::new();
    let mut expired_samples = 0_u64;
    let mut sample_count = 0_u64;
    let mut last_nonzero_held_ns = None;
    let mut max_tick_batch = 0_usize;
    let mut max_pending = 0_usize;
    let mut max_queued_edges = 0_usize;
    let mut command_counts = [
        ("input", 0_u64),
        ("pause", 0_u64),
        ("hidden", 0_u64),
        ("resume", 0_u64),
        ("diagnostics", 0_u64),
    ];
    let mut stopped_cleanly = false;

    let send_ticks = |scheduler_now_ns: u64,
                          receipt_now_ns: u64,
                          scheduler: &mut FixedStepScheduler,
                          input_receiver: &mut InputReceiver,
                          world: &mut WorldStateV3,
                          callback_send_records: &mut Vec<Value>,
                          consumed_edge_ids: &mut Vec<u64>,
                          expired_samples: &mut u64,
                          sample_count: &mut u64,
                          last_nonzero_held_ns: &mut Option<u64>,
                          max_tick_batch: &mut usize| {
        let plan = scheduler.advance(scheduler_now_ns).expect("owner monotonic clock");
        *max_tick_batch = (*max_tick_batch).max(plan.tick_count());
        if plan.tick_count() == 0 {
            return;
        }
        let samples = input_receiver
            .consume_tick_plan(&plan, receipt_now_ns)
            .expect("receiver tick plan monotonic");
        for sample in samples {
            *sample_count = sample_count.saturating_add(1);
            let expired = sample.held_axes().x() == 0.0
                && sample.held_axes().z() == 0.0
                && last_nonzero_held_ns
                    .is_some_and(|held_ns| receipt_now_ns.saturating_sub(held_ns) > continuous_input::HELD_EXPIRY_NS);
            if expired {
                *expired_samples = expired_samples.saturating_add(1);
            }
            for edge in sample.edge_events() {
                consumed_edge_ids.push(edge.id());
            }
            let tick = sample.tick_index().saturating_add(1);
            world
                .advance_clock(tick, receipt_now_ns / 1_000_000)
                .expect("world clock monotonic");
            let ack_seq = input_receiver.ack_seq();
            let payload = world.view(ack_seq);
            let payload_bytes = serde_json::to_vec(&payload)
                .expect("WorldView must serialize")
                .len();
            let send_start = monotonic_ms();
            let send_result = channel.send(payload);
            let send_end = monotonic_ms();
            callback_send_records.push(json!({
                "ownerSendStartMs": send_start,
                "ownerSendEndMs": send_end,
                "sendDurationMs": send_end - send_start,
                "epoch": info.world_epoch,
                "tick": tick,
                "ackSeq": ack_seq,
                "payloadBytes": payload_bytes,
                "sendOk": send_result.is_ok(),
                "sendError": send_result.err().map(|error| error.to_string()),
            }));
        }
    };

    loop {
        match receiver.recv_timeout(Duration::from_millis(2)) {
            Ok(command) => match command {
                OwnerCommand::Input {
                    frame,
                    payload_bytes,
                    receipt_ns,
                    reply,
                } => {
                    command_counts[0].1 = command_counts[0].1.saturating_add(1);
                    let outcome = input_receiver.receive(frame.clone(), receipt_ns);
                    if frame.held_axes().x() != 0.0 || frame.held_axes().z() != 0.0 {
                        last_nonzero_held_ns = Some(receipt_ns);
                    }
                    max_pending = max_pending.max(input_receiver.pending_len());
                    max_queued_edges = max_queued_edges.max(input_receiver.queued_edge_len());
                    input_records.push(json!({
                        "receiptNs": receipt_ns,
                        "receiptMs": receipt_ns as f64 / 1_000_000.0,
                        "payloadBytes": payload_bytes,
                        "epoch": info.world_epoch,
                        "seq": frame.seq(),
                        "outcome": outcome_value(outcome),
                    }));
                    let _ = reply.send(Ok(json!({
                        "status": "received",
                        "sessionId": info.session_id,
                        "generation": info.generation,
                        "epoch": info.world_epoch,
                        "seq": frame.seq(),
                        "ackSeq": input_receiver.ack_seq(),
                        "outcome": outcome_value(outcome),
                        "receiptNs": receipt_ns,
                        "payloadBytes": payload_bytes,
                    })));
                }
                OwnerCommand::Control {
                    kind,
                    receipt_ns,
                    reply,
                } => {
                    let index = match kind {
                        "pause" => 1,
                        "hidden" => 2,
                        "resume" => 3,
                        _ => 1,
                    };
                    command_counts[index].1 = command_counts[index].1.saturating_add(1);
                    let result = match kind {
                        "pause" => {
                            input_receiver.pause();
                            last_nonzero_held_ns = None;
                            let owner_now_ns = owner_start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
                            scheduler.pause(owner_now_ns).map(|_| json!({"status":"paused"})).map_err(|error| error.to_string())
                        }
                        "hidden" => {
                            input_receiver.hidden();
                            last_nonzero_held_ns = None;
                            let owner_now_ns = owner_start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
                            scheduler.pause(owner_now_ns).map(|_| json!({"status":"hidden"})).map_err(|error| error.to_string())
                        }
                        "resume" => {
                            input_receiver.resume();
                            let owner_now_ns = owner_start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
                            scheduler.resume(owner_now_ns).map(|_| json!({"status":"resumed"})).map_err(|error| error.to_string())
                        }
                        _ => Err("unknown_control".to_owned()),
                    };
                    let _ = reply.send(result.map(|mut value| {
                        if let Value::Object(object) = &mut value {
                            object.insert("epoch".to_owned(), json!(info.world_epoch));
                            object.insert("receiptNs".to_owned(), json!(receipt_ns));
                            object.insert("ackSeq".to_owned(), json!(input_receiver.ack_seq()));
                        }
                        value
                    }));
                }
                OwnerCommand::Diagnostics { reply } => {
                    command_counts[4].1 = command_counts[4].1.saturating_add(1);
                    let counters = input_receiver.counters();
                    let _ = reply.send(Ok(json!({
                        "sessionId": info.session_id,
                        "generation": info.generation,
                        "epoch": info.world_epoch,
                        "ackSeq": input_receiver.ack_seq(),
                        "pendingFrames": input_receiver.pending_len(),
                        "queuedEdges": input_receiver.queued_edge_len(),
                        "heldAxes": {"x": input_receiver.held_axes().x(), "z": input_receiver.held_axes().z()},
                        "paused": input_receiver.is_paused(),
                        "sampleCount": sample_count,
                        "expiredHeldSamples": expired_samples,
                        "consumedEdgeIds": consumed_edge_ids,
                        "maxPendingFrames": max_pending,
                        "maxQueuedEdges": max_queued_edges,
                        "counters": {
                            "acceptedFrames": counters.accepted_frames(),
                            "appliedFrames": counters.applied_frames(),
                            "bufferedFrames": counters.buffered_frames(),
                            "droppedFrames": counters.dropped_frames(),
                            "droppedEdges": counters.dropped_edges(),
                            "duplicateEdges": counters.duplicate_edges(),
                            "protocolRejections": counters.protocol_rejections(),
                            "versionRejections": counters.version_rejections(),
                            "epochRejections": counters.epoch_rejections(),
                            "pausedRejections": counters.paused_rejections(),
                            "clockRejections": counters.clock_rejections(),
                            "sequenceRejections": counters.sequence_rejections(),
                            "gapOverflowRejections": counters.gap_overflow_rejections(),
                        }
                    })));
                }
                OwnerCommand::Stop { reply } => {
                    stopped_cleanly = true;
                    let _ = reply.send(Ok(json!({
                        "status": "stopping",
                        "sessionId": info.session_id,
                        "generation": info.generation,
                        "epoch": info.world_epoch,
                    })));
                    break;
                }
            },
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
        let loop_scheduler_ns = owner_start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
        let loop_receipt_ns = monotonic_ns();
        send_ticks(
            loop_scheduler_ns,
            loop_receipt_ns,
            &mut scheduler,
            &mut input_receiver,
            &mut world,
            &mut callback_send_records,
            &mut consumed_edge_ids,
            &mut expired_samples,
            &mut sample_count,
            &mut last_nonzero_held_ns,
            &mut max_tick_batch,
        );
    }

    let counters = input_receiver.counters();
    OwnerSummary {
        session_id: info.session_id,
        generation: info.generation,
        world_epoch: info.world_epoch,
        stop_requested_at_ms: monotonic_ms(),
        input_records,
        callback_send_records,
        diagnostics: json!({
            "sampleCount": sample_count,
            "expiredHeldSamples": expired_samples,
            "consumedEdgeIds": consumed_edge_ids,
            "maxPendingFrames": max_pending,
            "maxQueuedEdges": max_queued_edges,
            "maxTickBatch": max_tick_batch,
            "schedulerTotalTicks": scheduler.total_ticks(),
            "schedulerTotalDroppedTimeNs": scheduler.total_dropped_time_nanos(),
            "pendingFramesAtStop": input_receiver.pending_len(),
            "queuedEdgesAtStop": input_receiver.queued_edge_len(),
            "ackSeqAtStop": input_receiver.ack_seq(),
            "counters": {
                "acceptedFrames": counters.accepted_frames(),
                "appliedFrames": counters.applied_frames(),
                "bufferedFrames": counters.buffered_frames(),
                "droppedFrames": counters.dropped_frames(),
                "droppedEdges": counters.dropped_edges(),
                "duplicateEdges": counters.duplicate_edges(),
                "protocolRejections": counters.protocol_rejections(),
                "versionRejections": counters.version_rejections(),
                "epochRejections": counters.epoch_rejections(),
                "pausedRejections": counters.paused_rejections(),
                "clockRejections": counters.clock_rejections(),
                "sequenceRejections": counters.sequence_rejections(),
                "gapOverflowRejections": counters.gap_overflow_rejections(),
            }
        }),
        command_counts: command_counts_value(&command_counts),
        stopped_cleanly,
    }
}

fn stop_active(mut active: ActiveSession) -> Result<OwnerSummary, String> {
    let (reply_tx, reply_rx) = mpsc::channel();
    active
        .tx
        .send(OwnerCommand::Stop { reply: reply_tx })
        .map_err(|_| runtime_error("owner_disconnected"))?;
    let _ = reply_rx
        .recv_timeout(Duration::from_secs(2))
        .map_err(|_| runtime_error("stop_timeout"))??;
    let join = active
        .join
        .take()
        .ok_or_else(|| runtime_error("owner_join_missing"))?;
    join.join().map_err(|_| runtime_error("owner_join_failed"))
}

fn current_sender(
    state: &RuntimeState,
    session_id: &str,
) -> Result<(Sender<OwnerCommand>, SessionInfo), String> {
    if state.closing.load(Ordering::Acquire) {
        return Err(runtime_error("closing"));
    }
    let registry = state.registry.lock().map_err(|_| runtime_error("registry_poisoned"))?;
    let active = registry.active.as_ref().ok_or_else(|| runtime_error("stale_session"))?;
    if active.info.session_id != session_id {
        return Err(runtime_error("stale_session"));
    }
    Ok((active.tx.clone(), active.info.clone()))
}

fn spawn_session(
    state: &RuntimeState,
    channel: Channel<WorldView>,
    world_epoch: u64,
) -> Result<ActiveSession, String> {
    let session_number = state.next_session.fetch_add(1, Ordering::Relaxed);
    let generation = state.next_generation.fetch_add(1, Ordering::Relaxed);
    let info = SessionInfo {
        session_id: format!("tp06b1-session-{session_number}"),
        generation,
        world_epoch,
    };
    let (tx, rx) = mpsc::channel();
    let thread_info = info.clone();
    let join = thread::Builder::new()
        .name(format!("tp06b1-owner-{generation}"))
        .spawn(move || owner_loop(thread_info, rx, channel))
        .map_err(|_| runtime_error("owner_spawn_failed"))?;
    Ok(ActiveSession {
        info,
        tx,
        join: Some(join),
    })
}

fn write_report(report: Value) -> Result<(), String> {
    let path = result_path();
    fs::write(
        &path,
        serde_json::to_vec_pretty(&report).map_err(|_| runtime_error("result_encode_failed"))?,
    )
    .map_err(|_| runtime_error("result_write_failed"))
}

#[tauri::command]
fn tp06b_start(
    channel: Channel<WorldView>,
    world_epoch: u64,
    state: State<'_, RuntimeState>,
) -> Result<SessionInfo, String> {
    if state.closing.load(Ordering::Acquire) {
        return Err(runtime_error("closing"));
    }
    let mut registry = state.registry.lock().map_err(|_| runtime_error("registry_poisoned"))?;
    if registry.active.is_some() {
        return Err(runtime_error("already_running"));
    }
    let active = spawn_session(&state, channel, world_epoch)?;
    let info = active.info.clone();
    registry.active = Some(active);
    Ok(info)
}

/// The fixed TP-06B1 transport command. It accepts a JSON string and invokes
/// the formal InputFrame::from_json parser before the owner/receiver sees it.
#[tauri::command]
fn tp06b_input(
    session_id: String,
    payload: String,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let payload_bytes = payload.as_bytes();
    let frame = InputFrame::from_json(payload_bytes).map_err(|error| match error {
        continuous_input::InputDecodeError::PayloadTooLarge => runtime_error("payload_too_large"),
        continuous_input::InputDecodeError::InvalidJson
        | continuous_input::InputDecodeError::InvalidFrame(_) => runtime_error("invalid_payload"),
    })?;
    let (tx, info) = current_sender(&state, &session_id)?;
    let receipt_ns = monotonic_ns();
    let (reply_tx, reply_rx) = mpsc::channel();
    tx.send(OwnerCommand::Input {
        frame,
        payload_bytes: payload_bytes.len(),
        receipt_ns,
        reply: reply_tx,
    })
    .map_err(|_| runtime_error("owner_disconnected"))?;
    reply_rx
        .recv_timeout(Duration::from_secs(2))
        .map_err(|_| runtime_error("input_timeout"))?
        .map(|mut value| {
            if let Value::Object(object) = &mut value {
                object.insert("sessionId".to_owned(), json!(info.session_id));
                object.insert("generation".to_owned(), json!(info.generation));
            }
            value
        })
}

fn control_command(
    session_id: String,
    kind: &'static str,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let (tx, _info) = current_sender(&state, &session_id)?;
    let receipt_ns = monotonic_ns();
    let (reply_tx, reply_rx) = mpsc::channel();
    tx.send(OwnerCommand::Control {
        kind,
        receipt_ns,
        reply: reply_tx,
    })
    .map_err(|_| runtime_error("owner_disconnected"))?;
    reply_rx
        .recv_timeout(Duration::from_secs(2))
        .map_err(|_| runtime_error("control_timeout"))?
}

#[tauri::command]
fn tp06b_pause(session_id: String, state: State<'_, RuntimeState>) -> Result<Value, String> {
    control_command(session_id, "pause", state)
}

#[tauri::command]
fn tp06b_hidden(session_id: String, state: State<'_, RuntimeState>) -> Result<Value, String> {
    control_command(session_id, "hidden", state)
}

#[tauri::command]
fn tp06b_resume(session_id: String, state: State<'_, RuntimeState>) -> Result<Value, String> {
    control_command(session_id, "resume", state)
}

#[tauri::command]
fn tp06b_diagnostics(session_id: String, state: State<'_, RuntimeState>) -> Result<Value, String> {
    let (tx, _info) = current_sender(&state, &session_id)?;
    let (reply_tx, reply_rx) = mpsc::channel();
    tx.send(OwnerCommand::Diagnostics { reply: reply_tx })
        .map_err(|_| runtime_error("owner_disconnected"))?;
    reply_rx
        .recv_timeout(Duration::from_secs(2))
        .map_err(|_| runtime_error("diagnostics_timeout"))?
}

#[tauri::command]
fn tp06b_replace(
    session_id: String,
    channel: Channel<WorldView>,
    world_epoch: u64,
    state: State<'_, RuntimeState>,
) -> Result<SessionInfo, String> {
    if state.closing.load(Ordering::Acquire) {
        return Err(runtime_error("closing"));
    }
    let old = {
        let mut registry = state.registry.lock().map_err(|_| runtime_error("registry_poisoned"))?;
        let active = registry.active.as_ref().ok_or_else(|| runtime_error("stale_session"))?;
        if active.info.session_id != session_id {
            return Err(runtime_error("stale_session"));
        }
        if active.info.world_epoch == world_epoch {
            return Err(runtime_error("invalid_epoch"));
        }
        registry.active.take().ok_or_else(|| runtime_error("stale_session"))?
    };
    let _old_summary = stop_active(old)?;
    let mut registry = state.registry.lock().map_err(|_| runtime_error("registry_poisoned"))?;
    if registry.active.is_some() {
        return Err(runtime_error("already_running"));
    }
    let active = spawn_session(&state, channel, world_epoch)?;
    let info = active.info.clone();
    registry.active = Some(active);
    Ok(info)
}

#[tauri::command]
fn tp06b_stop(session_id: String, state: State<'_, RuntimeState>) -> Result<Value, String> {
    let active = {
        let mut registry = state.registry.lock().map_err(|_| runtime_error("registry_poisoned"))?;
        let active = registry.active.as_ref().ok_or_else(|| runtime_error("stale_session"))?;
        if active.info.session_id != session_id {
            return Err(runtime_error("stale_session"));
        }
        registry.active.take().ok_or_else(|| runtime_error("stale_session"))?
    };
    let summary = stop_active(active)?;
    Ok(serde_json::to_value(summary).map_err(|_| runtime_error("summary_encode_failed"))?)
}

#[tauri::command]
fn tp06b_record_result(report: Value) -> Result<Value, String> {
    write_report(report)?;
    Ok(json!({"status":"recorded","path":"tools/continuous-tps-architecture-probe/results/tauri-input-runtime-tp06b1.json"}))
}

#[tauri::command]
fn tp06b_environment(user_agent: String) -> Value {
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
        "webviewVersion": webview_version,
        "userAgentBytes": user_agent.len()
    })
}

fn stop_current_if_any(state: &RuntimeState) -> Result<Option<OwnerSummary>, String> {
    let active = {
        let mut registry = state.registry.lock().map_err(|_| runtime_error("registry_poisoned"))?;
        registry.active.take()
    };
    active.map(stop_active).transpose()
}

#[tauri::command]
fn tp06b_finish(app: AppHandle, state: State<'_, RuntimeState>) -> Result<(), String> {
    state.closing.store(true, Ordering::Release);
    let _ = stop_current_if_any(&state)?;
    app.exit(0);
    Ok(())
}

fn main() {
    PROCESS_START.get_or_init(Instant::now);
    tauri::Builder::default()
        .manage(RuntimeState::default())
        .invoke_handler(tauri::generate_handler![
            tp06b_start,
            tp06b_input,
            tp06b_pause,
            tp06b_hidden,
            tp06b_resume,
            tp06b_diagnostics,
            tp06b_replace,
            tp06b_stop,
            tp06b_record_result,
            tp06b_environment,
            tp06b_finish,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(state) = window.try_state::<RuntimeState>() {
                    state.closing.store(true, Ordering::Release);
                    let _ = stop_current_if_any(&state);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running TP-06B1 Tauri application");
}
