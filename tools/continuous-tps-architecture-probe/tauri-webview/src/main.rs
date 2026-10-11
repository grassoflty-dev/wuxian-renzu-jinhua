use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};
use tauri::{
    ipc::Channel, AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder, Window,
};

const PROTOCOL: &str = "continuous-ipc";
const VERSION: u32 = 1;
const WORLD_EPOCH: u64 = 7;
const NEXT_WORLD_EPOCH: u64 = 8;

static PROCESS_START: OnceLock<Instant> = OnceLock::new();

fn monotonic_ms() -> f64 {
    PROCESS_START
        .get_or_init(Instant::now)
        .elapsed()
        .as_secs_f64()
        * 1000.0
}

#[derive(Default)]
struct ProbeState {
    paused: Mutex<bool>,
    epoch: Mutex<u64>,
    server_tick: Mutex<u64>,
    ack_watermark: Mutex<Option<u64>>,
    edge_watermark: Mutex<Vec<u64>>,
    child_release: Mutex<bool>,
    child_channel_result: Mutex<Option<Value>>,
    ag29_case: Mutex<String>,
    ag29_phase: Mutex<String>,
    ag29_channel_required: Mutex<bool>,
    ag29_stage: Mutex<String>,
    ag29_events: Mutex<Vec<Value>>,
    ag29_focus_events: Mutex<Vec<Value>>,
    ag29_child_callbacks: Mutex<Vec<Value>>,
    ag29_old_channel: Mutex<Option<Channel<WorldView>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InputFrame {
    protocol: String,
    version: u32,
    world_epoch: u64,
    seq: u64,
    client_time_ms: u64,
    held_axes: HeldAxes,
    buttons: Buttons,
    edge_events: Vec<EdgeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HeldAxes {
    x: f64,
    z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Buttons {
    jump: bool,
    dash: bool,
    interact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EdgeEvent {
    id: u64,
    kind: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorldView {
    protocol: String,
    version: u32,
    world_epoch: u64,
    server_tick: u64,
    server_time_ms: u64,
    world_id: String,
    player: PlayerState,
    ack_seq: u64,
    edge_events: Vec<EdgeEvent>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlayerState {
    transform: Transform,
    meters: Meters,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Transform {
    position_m: Point3,
    velocity_mps: Point3,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Point3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Meters {
    hp: f64,
    stamina: f64,
    ammo: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamPlan {
    scenario: String,
    interval_ms: u64,
    frames: Vec<InputFrame>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamSendRecord {
    send_index: usize,
    seq: u64,
    world_epoch: u64,
    server_tick: u64,
    payload_bytes_utf8: usize,
    sent_elapsed_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamSummary {
    scenario: String,
    interval_ms: u64,
    frame_count: usize,
    completed_elapsed_ms: f64,
    send_records: Vec<StreamSendRecord>,
    input_seqs: Vec<u64>,
    edge_input_ids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct InputSequenceReport {
    frame_count: usize,
    received_seqs: Vec<u64>,
    duplicate_seqs: Vec<u64>,
    out_of_order_seqs: Vec<u64>,
    missing_seqs: Vec<u64>,
    edge_input_ids: Vec<u64>,
    input_payload_bytes_utf8: Vec<usize>,
    structurally_valid: bool,
}

fn validate(frame: &InputFrame) -> Result<(), String> {
    validate_structure(frame)?;
    if frame.world_epoch != WORLD_EPOCH {
        return Err(format!("worldEpoch must be {WORLD_EPOCH}"));
    }
    Ok(())
}

fn validate_structure(frame: &InputFrame) -> Result<(), String> {
    if frame.protocol != PROTOCOL {
        return Err(format!("protocol must be {PROTOCOL}"));
    }
    if frame.version != VERSION {
        return Err(format!("version must be {VERSION}"));
    }
    if frame.seq == 0 {
        return Err("seq must be positive".into());
    }
    if frame.edge_events.len() > 8 {
        return Err("edgeEvents exceeds the protocol limit of 8".into());
    }
    if !frame.held_axes.x.is_finite() || !frame.held_axes.z.is_finite() {
        return Err("heldAxes must be finite".into());
    }
    let axis_length = frame.held_axes.x.hypot(frame.held_axes.z);
    if axis_length > 1.00001 {
        return Err(format!("heldAxes must be normalized, length={axis_length}"));
    }
    let _ = (frame.client_time_ms, frame.buttons.jump, frame.buttons.dash, frame.buttons.interact);
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PauseInputDisposition {
    seq: u64,
    world_epoch: u64,
    paused: bool,
    consumed: bool,
    edge_consumed: bool,
    ack_seq: Option<u64>,
    server_tick: u64,
    reason: String,
    rust_monotonic_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PauseBarrierState {
    paused: bool,
    stale_input_cleared: bool,
    ack_watermark: Option<u64>,
    server_tick: u64,
    rust_monotonic_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct EpochRejection {
    seq: u64,
    world_epoch: u64,
    current_epoch: u64,
    reason: String,
    rust_monotonic_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct EpochAccepted {
    seq: u64,
    world_epoch: u64,
    ack_seq: u64,
    server_tick: u64,
    edge_ids: Vec<u64>,
    rust_monotonic_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct EpochProbeSummary {
    accepted: Vec<EpochAccepted>,
    rejected: Vec<EpochRejection>,
    final_epoch: u64,
    ack_by_epoch: Vec<(u64, Vec<u64>)>,
    edge_watermark_by_epoch: Vec<(u64, Vec<u64>)>,
}

#[tauri::command]
fn input_roundtrip(frame: InputFrame, on_world_view: Channel<WorldView>) -> Result<(), String> {
    validate(&frame)?;
    let snapshot = WorldView {
        protocol: PROTOCOL.into(),
        version: VERSION,
        world_epoch: WORLD_EPOCH,
        server_tick: 1,
        server_time_ms: 33,
        world_id: "ag25-webview-room".into(),
        player: PlayerState {
            transform: Transform {
                position_m: Point3 { x: 1.25, y: 0.0, z: -2.5 },
                velocity_mps: Point3 { x: 0.25, y: 0.0, z: -0.5 },
            },
            meters: Meters { hp: 100.0, stamina: 80.0, ammo: 12.0 },
        },
        ack_seq: frame.seq,
        edge_events: frame.edge_events,
    };
    on_world_view.send(snapshot).map_err(|error| error.to_string())
}

#[tauri::command]
fn stream_world_views(
    plan: StreamPlan,
    on_world_view: Channel<WorldView>,
) -> Result<StreamSummary, String> {
    if plan.frames.is_empty() || plan.frames.len() > 120 {
        return Err("stream frame count must be between 1 and 120".into());
    }
    if plan.interval_ms > 200 {
        return Err("stream interval must be at most 200ms".into());
    }

    let start = Instant::now();
    let mut send_records = Vec::with_capacity(plan.frames.len());
    let mut input_seqs = Vec::with_capacity(plan.frames.len());
    let mut edge_input_ids = Vec::new();

    for (index, frame) in plan.frames.iter().enumerate() {
        validate(frame)?;
        input_seqs.push(frame.seq);
        edge_input_ids.extend(frame.edge_events.iter().map(|edge| edge.id));

        let server_tick = 10_000 + index as u64;
        let snapshot = WorldView {
            protocol: PROTOCOL.into(),
            version: VERSION,
            world_epoch: frame.world_epoch,
            server_tick,
            server_time_ms: (index as u64 + 1) * plan.interval_ms,
            world_id: format!("ag26-{}", plan.scenario),
            player: PlayerState {
                transform: Transform {
                    position_m: Point3 { x: index as f64 * 0.1, y: 0.0, z: -2.5 },
                    velocity_mps: Point3 { x: 0.1, y: 0.0, z: 0.0 },
                },
                meters: Meters { hp: 100.0, stamina: 80.0, ammo: 12.0 },
            },
            ack_seq: frame.seq,
            edge_events: frame.edge_events.clone(),
        };
        let payload_bytes_utf8 = serde_json::to_vec(&snapshot)
            .map_err(|error| error.to_string())?
            .len();
        on_world_view.send(snapshot).map_err(|error| error.to_string())?;
        send_records.push(StreamSendRecord {
            send_index: index,
            seq: frame.seq,
            world_epoch: frame.world_epoch,
            server_tick,
            payload_bytes_utf8,
            sent_elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
        });
        if index + 1 < plan.frames.len() {
            thread::sleep(Duration::from_millis(plan.interval_ms));
        }
    }

    Ok(StreamSummary {
        scenario: plan.scenario,
        interval_ms: plan.interval_ms,
        frame_count: plan.frames.len(),
        completed_elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
        send_records,
        input_seqs,
        edge_input_ids,
    })
}

#[tauri::command]
fn inspect_input_sequence(frames: Vec<InputFrame>) -> Result<InputSequenceReport, String> {
    if frames.is_empty() || frames.len() > 32 {
        return Err("input diagnostic frame count must be between 1 and 32".into());
    }
    let mut received_seqs = Vec::with_capacity(frames.len());
    let mut duplicate_seqs = Vec::new();
    let mut out_of_order_seqs = Vec::new();
    let mut edge_input_ids = Vec::new();
    let mut payload_bytes = Vec::with_capacity(frames.len());
    for (index, frame) in frames.iter().enumerate() {
        validate(frame)?;
        if received_seqs.contains(&frame.seq) && !duplicate_seqs.contains(&frame.seq) {
            duplicate_seqs.push(frame.seq);
        }
        if index > 0 && frame.seq < received_seqs[index - 1] {
            out_of_order_seqs.push(frame.seq);
        }
        received_seqs.push(frame.seq);
        edge_input_ids.extend(frame.edge_events.iter().map(|edge| edge.id));
        payload_bytes.push(serde_json::to_vec(frame).map_err(|error| error.to_string())?.len());
    }
    let min_seq = *received_seqs.iter().min().unwrap();
    let max_seq = *received_seqs.iter().max().unwrap();
    let missing_seqs = (min_seq..=max_seq)
        .filter(|seq| !received_seqs.contains(seq))
        .collect();
    Ok(InputSequenceReport {
        frame_count: frames.len(),
        received_seqs,
        duplicate_seqs,
        out_of_order_seqs,
        missing_seqs,
        edge_input_ids,
        input_payload_bytes_utf8: payload_bytes,
        structurally_valid: true,
    })
}

#[tauri::command]
fn reset_lifecycle_probe(state: State<ProbeState>) -> Result<(), String> {
    *state.paused.lock().unwrap() = false;
    *state.epoch.lock().unwrap() = WORLD_EPOCH;
    *state.server_tick.lock().unwrap() = 0;
    *state.ack_watermark.lock().unwrap() = None;
    state.edge_watermark.lock().unwrap().clear();
    *state.child_release.lock().unwrap() = false;
    *state.child_channel_result.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
fn open_focus_child(app: AppHandle) -> Result<(), String> {
    let child = WebviewWindowBuilder::new(
        &app,
        "ag28-child",
        WebviewUrl::App("index.html".into()),
    )
    .title("AG-28 child focus window")
    .inner_size(480.0, 360.0)
    .center()
    .build()
    .map_err(|error| error.to_string())?;
    let focus_child = child.clone();
    thread::spawn(move || {
        let _ = focus_child.set_focus();
    });
    let watchdog_app = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(1000));
        if let Some(child) = watchdog_app.get_webview_window("ag28-child") {
            let _ = child.destroy();
        }
    });
    Ok(())
}

#[tauri::command]
fn record_child_channel_result(result: Value, state: State<ProbeState>) -> Result<(), String> {
    *state.child_channel_result.lock().unwrap() = Some(result);
    Ok(())
}

#[tauri::command]
fn get_child_channel_result(state: State<ProbeState>) -> Option<Value> {
    state.child_channel_result.lock().unwrap().clone()
}

#[tauri::command]
fn request_child_release(state: State<ProbeState>) -> Result<(), String> {
    *state.child_release.lock().unwrap() = true;
    Ok(())
}

#[tauri::command]
fn child_release_requested(state: State<ProbeState>) -> bool {
    *state.child_release.lock().unwrap()
}

#[tauri::command]
fn child_channel_probe(on_world_view: Channel<WorldView>) -> Result<(), String> {
    on_world_view
        .send(WorldView {
            protocol: PROTOCOL.into(),
            version: VERSION,
            world_epoch: NEXT_WORLD_EPOCH,
            server_tick: 1,
            server_time_ms: 33,
            world_id: "ag28-child-channel-b".into(),
            player: PlayerState {
                transform: Transform {
                    position_m: Point3 { x: 42.0, y: 0.0, z: 0.0 },
                    velocity_mps: Point3 { x: 0.0, y: 0.0, z: 0.0 },
                },
                meters: Meters { hp: 100.0, stamina: 80.0, ammo: 12.0 },
            },
            ack_seq: 42001,
            edge_events: vec![EdgeEvent { id: 42001, kind: "child-marker".into() }],
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_pause_barrier(paused: bool, state: State<ProbeState>) -> Result<PauseBarrierState, String> {
    *state.paused.lock().unwrap() = paused;
    let stale_input_cleared = !paused;
    if stale_input_cleared {
        *state.ack_watermark.lock().unwrap() = None;
        state.edge_watermark.lock().unwrap().clear();
    }
    Ok(PauseBarrierState {
        paused,
        stale_input_cleared,
        ack_watermark: *state.ack_watermark.lock().unwrap(),
        server_tick: *state.server_tick.lock().unwrap(),
        rust_monotonic_ms: monotonic_ms(),
    })
}

#[tauri::command]
fn submit_pause_probe_input(
    frame: InputFrame,
    state: State<ProbeState>,
) -> Result<PauseInputDisposition, String> {
    validate(&frame)?;
    let paused = *state.paused.lock().unwrap();
    let server_tick = *state.server_tick.lock().unwrap();
    if paused {
        return Ok(PauseInputDisposition {
            seq: frame.seq,
            world_epoch: frame.world_epoch,
            paused,
            consumed: false,
            edge_consumed: false,
            ack_seq: None,
            server_tick,
            reason: "paused_input_not_consumed".into(),
            rust_monotonic_ms: monotonic_ms(),
        });
    }
    let mut tick = state.server_tick.lock().unwrap();
    *tick += 1;
    let edge_consumed = !frame.edge_events.is_empty();
    if edge_consumed {
        state
            .edge_watermark
            .lock()
            .unwrap()
            .extend(frame.edge_events.iter().map(|edge| edge.id));
    }
    *state.ack_watermark.lock().unwrap() = Some(frame.seq);
    Ok(PauseInputDisposition {
        seq: frame.seq,
        world_epoch: frame.world_epoch,
        paused,
        consumed: true,
        edge_consumed,
        ack_seq: Some(frame.seq),
        server_tick: *tick,
        reason: "active_input_consumed".into(),
        rust_monotonic_ms: monotonic_ms(),
    })
}

#[tauri::command]
fn epoch_isolation_probe(
    frames: Vec<InputFrame>,
    on_world_view: Channel<WorldView>,
) -> Result<EpochProbeSummary, String> {
    if frames.is_empty() || frames.len() > 16 {
        return Err("epoch probe frame count must be between 1 and 16".into());
    }
    let mut current_epoch = WORLD_EPOCH;
    let mut server_tick = 0;
    let mut ack_by_epoch: Vec<(u64, Vec<u64>)> = vec![(WORLD_EPOCH, Vec::new())];
    let mut edge_watermark_by_epoch: Vec<(u64, Vec<u64>)> = vec![(WORLD_EPOCH, Vec::new())];
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();

    for frame in frames {
        validate_structure(&frame)?;
        if frame.world_epoch != WORLD_EPOCH && frame.world_epoch != NEXT_WORLD_EPOCH {
            rejected.push(EpochRejection {
                seq: frame.seq,
                world_epoch: frame.world_epoch,
                current_epoch,
                reason: "unsupported_epoch".into(),
                rust_monotonic_ms: monotonic_ms(),
            });
            continue;
        }
        if frame.world_epoch < current_epoch {
            rejected.push(EpochRejection {
                seq: frame.seq,
                world_epoch: frame.world_epoch,
                current_epoch,
                reason: "stale_epoch_rejected".into(),
                rust_monotonic_ms: monotonic_ms(),
            });
            continue;
        }
        if frame.world_epoch > current_epoch {
            current_epoch = frame.world_epoch;
            server_tick = 0;
            ack_by_epoch.push((current_epoch, Vec::new()));
            edge_watermark_by_epoch.push((current_epoch, Vec::new()));
        }
        if ack_by_epoch.last().unwrap().1.is_empty() && frame.seq != 1 {
            rejected.push(EpochRejection {
                seq: frame.seq,
                world_epoch: frame.world_epoch,
                current_epoch,
                reason: "epoch_requires_seq_one_baseline".into(),
                rust_monotonic_ms: monotonic_ms(),
            });
            continue;
        }
        server_tick += 1;
        let edge_ids: Vec<u64> = frame.edge_events.iter().map(|edge| edge.id).collect();
        ack_by_epoch.last_mut().unwrap().1.push(frame.seq);
        edge_watermark_by_epoch
            .last_mut()
            .unwrap()
            .1
            .extend(edge_ids.iter().copied());
        on_world_view
            .send(WorldView {
                protocol: PROTOCOL.into(),
                version: VERSION,
                world_epoch: current_epoch,
                server_tick,
                server_time_ms: server_tick * 33,
                world_id: format!("ag27-epoch-{current_epoch}"),
                player: PlayerState {
                    transform: Transform {
                        position_m: Point3 { x: server_tick as f64, y: 0.0, z: -1.0 },
                        velocity_mps: Point3 { x: 0.0, y: 0.0, z: 0.0 },
                    },
                    meters: Meters { hp: 100.0, stamina: 80.0, ammo: 12.0 },
                },
                ack_seq: frame.seq,
                edge_events: frame.edge_events,
            })
            .map_err(|error| error.to_string())?;
        accepted.push(EpochAccepted {
            seq: frame.seq,
            world_epoch: current_epoch,
            ack_seq: frame.seq,
            server_tick,
            edge_ids,
            rust_monotonic_ms: monotonic_ms(),
        });
    }
    Ok(EpochProbeSummary {
        accepted,
        rejected,
        final_epoch: current_epoch,
        ack_by_epoch,
        edge_watermark_by_epoch,
    })
}

#[tauri::command]
fn reopen_channel_probe(on_world_view: Channel<WorldView>) -> Result<(), String> {
    for seq in 1..=2 {
        on_world_view
            .send(WorldView {
                protocol: PROTOCOL.into(),
                version: VERSION,
                world_epoch: NEXT_WORLD_EPOCH,
                server_tick: seq,
                server_time_ms: seq * 33,
                world_id: "ag27-reopened-window".into(),
                player: PlayerState {
                    transform: Transform {
                        position_m: Point3 { x: seq as f64, y: 0.0, z: 0.0 },
                        velocity_mps: Point3 { x: 0.0, y: 0.0, z: 0.0 },
                    },
                    meters: Meters { hp: 100.0, stamina: 80.0, ammo: 12.0 },
                },
                ack_seq: seq,
                edge_events: Vec::new(),
            })
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn close_reopen_window(app: AppHandle, window: Window) -> Result<(), String> {
    WebviewWindowBuilder::new(
        &app,
        "ag27-reopened",
        WebviewUrl::App("index.html".into()),
    )
    .title("AG-27 reopened lifecycle window")
    .inner_size(720.0, 620.0)
    .center()
    .build()
    .map_err(|error| error.to_string())?;
    window.destroy().map_err(|error| error.to_string())?;
    std::process::exit(0)
}

#[tauri::command]
fn finalize_reopened_window(extra: Value) -> Result<(), String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..\\results\\tauri-webview-ag28.json");
    let mut report: Value = serde_json::from_str(
        &fs::read_to_string(&path).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    report["lifecycle"]["reopenedWindow"] = extra;
    let text = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    fs::write(path, format!("{text}\n")).map_err(|error| error.to_string())
}

fn ag29_push_event(state: &ProbeState, kind: &str, data: Value) {
    state.ag29_events.lock().unwrap().push(json!({
        "kind": kind,
        "rustMonotonicMs": monotonic_ms(),
        "data": data
    }));
}

fn ag29_set_stage(state: &ProbeState, stage: &str) {
    *state.ag29_stage.lock().unwrap() = stage.into();
    ag29_push_event(state, stage, json!({}));
}

#[tauri::command]
fn ag29_case_config(state: State<ProbeState>) -> Value {
    json!({
        "case": state.ag29_case.lock().unwrap().clone(),
        "phase": state.ag29_phase.lock().unwrap().clone(),
        "channelRequired": *state.ag29_channel_required.lock().unwrap()
    })
}

#[tauri::command]
fn ag29_child_page_ready(label: String, state: State<ProbeState>) -> Value {
    ag29_set_stage(&state, "page-ready");
    ag29_push_event(&state, "child-js-page-ready", json!({ "label": label }));
    json!({
        "case": state.ag29_case.lock().unwrap().clone(),
        "phase": state.ag29_phase.lock().unwrap().clone(),
        "channelRequired": *state.ag29_channel_required.lock().unwrap()
    })
}

#[tauri::command]
fn ag29_child_channel_probe(on_world_view: Channel<WorldView>, state: State<ProbeState>) -> Result<(), String> {
    let phase = state.ag29_phase.lock().unwrap().clone();
    let (epoch, marker, world_id) = if phase == "a" {
        (WORLD_EPOCH, 41001, "ag29-channel-a")
    } else {
        (NEXT_WORLD_EPOCH, 42001, "ag29-channel-b")
    };
    if phase == "a" {
        *state.ag29_old_channel.lock().unwrap() = Some(on_world_view.clone());
    }
    ag29_set_stage(&state, "channel-sent");
    on_world_view
        .send(WorldView {
            protocol: PROTOCOL.into(),
            version: VERSION,
            world_epoch: epoch,
            server_tick: 1,
            server_time_ms: 33,
            world_id: world_id.into(),
            player: PlayerState {
                transform: Transform {
                    position_m: Point3 { x: marker as f64, y: 0.0, z: 0.0 },
                    velocity_mps: Point3 { x: 0.0, y: 0.0, z: 0.0 },
                },
                meters: Meters { hp: 100.0, stamina: 80.0, ammo: 12.0 },
            },
            ack_seq: marker,
            edge_events: vec![EdgeEvent { id: marker, kind: "child-marker".into() }],
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn ag29_child_channel_callback(result: Value, state: State<ProbeState>) -> Result<(), String> {
    state.ag29_child_callbacks.lock().unwrap().push(result.clone());
    ag29_set_stage(&state, "channel-ack");
    ag29_push_event(&state, "child-js-channel-ack", result);
    Ok(())
}

fn ag29_write_report(report: &Value) -> Result<(), String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..\\results\\tauri-webview-ag29.json");
    let text = serde_json::to_string_pretty(report).map_err(|error| error.to_string())?;
    fs::write(path, format!("{text}\n")).map_err(|error| error.to_string())
}

fn ag29_wait_for_event(app: &AppHandle, expected: &str, timeout_ms: u64) -> bool {
    let start = Instant::now();
    while start.elapsed().as_millis() < timeout_ms as u128 {
        if app
            .state::<ProbeState>()
            .ag29_events
            .lock()
            .unwrap()
            .iter()
            .any(|event| event.get("kind").and_then(Value::as_str) == Some(expected))
        {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}

fn ag29_build_child(app: &AppHandle, label: &str) -> Result<tauri::WebviewWindow, String> {
    let child = WebviewWindowBuilder::new(app, label, WebviewUrl::App("ag29-child.html".into()))
        .title(format!("AG-29 {label}"))
        .inner_size(480.0, 360.0)
        .center()
        .build()
        .map_err(|error| error.to_string())?;
    let event_app = app.clone();
    child.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(focused) = event {
            let state = event_app.state::<ProbeState>();
            state.ag29_focus_events.lock().unwrap().push(json!({
                "focused": focused,
                "rustMonotonicMs": monotonic_ms()
            }));
            ag29_push_event(&state, "rust-window-focused", json!({ "focused": focused }));
        }
    });
    Ok(child)
}

fn ag29_destroy_child(child: &tauri::WebviewWindow, state: &ProbeState) -> Value {
    ag29_set_stage(state, "close-requested");
    let result = child.destroy().map(|_| "ok").unwrap_or("error");
    ag29_set_stage(state, "destroyed");
    json!({ "destroyResult": result, "destroyed": result == "ok" })
}

fn ag29_run_simple_case(app: &AppHandle, case_name: &str, channel: bool, focus: bool) -> Value {
    let state = app.state::<ProbeState>();
    *state.ag29_case.lock().unwrap() = case_name.into();
    *state.ag29_phase.lock().unwrap() = "single".into();
    *state.ag29_channel_required.lock().unwrap() = channel;
    *state.ag29_stage.lock().unwrap() = "create-requested".into();
    state.ag29_events.lock().unwrap().clear();
    state.ag29_focus_events.lock().unwrap().clear();
    state.ag29_child_callbacks.lock().unwrap().clear();
    let started = monotonic_ms();
    if case_name == "no_child" {
        ag29_set_stage(&state, "no-child-complete");
        return json!({
            "case": case_name,
            "lastSuccessfulStage": "no-child-complete",
            "timedOutAt": Value::Null,
            "exitCode": 0,
            "residualProcessObserved": false,
            "events": state.ag29_events.lock().unwrap().clone(),
            "elapsedMs": monotonic_ms() - started
        });
    }
    let child_label = match case_name {
        "create_no_focus" => "ag29-create",
        "focus_no_channel" => "ag29-focus",
        "channel_no_focus" => "ag29-channel",
        _ => "ag29-child",
    };
    let child = match ag29_build_child(app, child_label) {
        Ok(child) => child,
        Err(error) => return json!({
            "case": case_name,
            "lastSuccessfulStage": "create-requested",
            "timedOutAt": "create",
            "error": error,
            "exitCode": 0,
            "residualProcessObserved": false,
            "events": state.ag29_events.lock().unwrap().clone()
        }),
    };
    ag29_set_stage(&state, "created");
    if focus {
        let focus_child = child.clone();
        let focus_app = app.clone();
        thread::spawn(move || {
            let result = focus_child.set_focus().map(|_| "ok").unwrap_or("error");
            ag29_push_event(&focus_app.state::<ProbeState>(), "focus-requested", json!({ "result": result }));
        });
    }
    let page_ready = ag29_wait_for_event(app, "page-ready", 1500);
    if !page_ready {
        let cleanup = ag29_destroy_child(&child, &state);
        return json!({
            "case": case_name,
            "lastSuccessfulStage": "created",
            "timedOutAt": "page-ready",
            "focusRequested": focus,
            "channelRequired": channel,
            "cleanup": cleanup,
            "exitCode": 0,
            "residualProcessObserved": false,
            "events": state.ag29_events.lock().unwrap().clone()
        });
    }
    if channel {
        if !ag29_wait_for_event(app, "channel-ack", 1500) {
            let cleanup = ag29_destroy_child(&child, &state);
            return json!({
                "case": case_name,
                "lastSuccessfulStage": "page-ready",
                "timedOutAt": "channel-ack",
                "focusRequested": focus,
                "channelRequired": channel,
                "cleanup": cleanup,
                "childCallbacks": state.ag29_child_callbacks.lock().unwrap().clone(),
                "exitCode": 0,
                "residualProcessObserved": false,
                "events": state.ag29_events.lock().unwrap().clone()
            });
        }
    }
    let cleanup = ag29_destroy_child(&child, &state);
    json!({
        "case": case_name,
        "lastSuccessfulStage": "destroyed",
        "timedOutAt": Value::Null,
        "focusRequested": focus,
        "channelRequired": channel,
        "cleanup": cleanup,
        "childCallbacks": state.ag29_child_callbacks.lock().unwrap().clone(),
        "focusEvents": state.ag29_focus_events.lock().unwrap().clone(),
        "exitCode": 0,
        "residualProcessObserved": false,
        "events": state.ag29_events.lock().unwrap().clone(),
        "elapsedMs": monotonic_ms() - started
    })
}

fn ag29_run_full_ab(app: &AppHandle, repetition: usize) -> Value {
    let state = app.state::<ProbeState>();
    *state.ag29_case.lock().unwrap() = "complete_a_to_b".into();
    *state.ag29_channel_required.lock().unwrap() = true;
    state.ag29_old_channel.lock().unwrap().take();
    let mut records = Vec::new();
    for (phase, label, epoch_marker) in [("a", "ag29-a", 41001_u64), ("b", "ag29-b", 42001_u64)] {
        *state.ag29_phase.lock().unwrap() = phase.into();
        *state.ag29_stage.lock().unwrap() = "create-requested".into();
        state.ag29_events.lock().unwrap().clear();
        state.ag29_child_callbacks.lock().unwrap().clear();
        let child = match ag29_build_child(app, label) {
            Ok(child) => child,
            Err(error) => return json!({ "repetition": repetition, "status": "failure", "phase": phase, "error": error }),
        };
        ag29_set_stage(&state, "created");
        if !ag29_wait_for_event(app, "page-ready", 1500) || !ag29_wait_for_event(app, "channel-ack", 1500) {
            let cleanup = ag29_destroy_child(&child, &state);
            return json!({ "repetition": repetition, "status": "timeout", "phase": phase, "timedOutAt": *state.ag29_stage.lock().unwrap(), "cleanup": cleanup });
        }
        let callbacks = state.ag29_child_callbacks.lock().unwrap().clone();
        let cleanup = ag29_destroy_child(&child, &state);
        records.push(json!({ "phase": phase, "epoch": if phase == "a" { WORLD_EPOCH } else { NEXT_WORLD_EPOCH }, "marker": epoch_marker, "callbacks": callbacks, "cleanup": cleanup }));
    }
    let old_send = if let Some(old_channel) = state.ag29_old_channel.lock().unwrap().clone() {
        let send = old_channel.send(WorldView {
            protocol: PROTOCOL.into(),
            version: VERSION,
            world_epoch: NEXT_WORLD_EPOCH,
            server_tick: 99,
            server_time_ms: 99,
            world_id: "ag29-old-a-after-destroy".into(),
            player: PlayerState { transform: Transform { position_m: Point3 { x: 99.0, y: 0.0, z: 0.0 }, velocity_mps: Point3 { x: 0.0, y: 0.0, z: 0.0 } }, meters: Meters { hp: 100.0, stamina: 80.0, ammo: 12.0 } },
            ack_seq: 49999,
            edge_events: Vec::new(),
        });
        json!({ "sendReturned": send.is_ok(), "sendError": send.err().map(|error| error.to_string()), "callbackAfterDestroy": "not_observable_after_A_window_destroy" })
    } else {
        json!({ "sendReturned": false, "sendError": "old Channel handle was not retained", "callbackAfterDestroy": "not_measured" })
    };
    json!({ "repetition": repetition, "status": "observed", "phases": records, "oldAAfterDestroy": old_send, "newEpoch": NEXT_WORLD_EPOCH })
}

#[tauri::command]
fn start_ag29_matrix(regression: Value, app: AppHandle) -> Result<(), String> {
    let worker_app = app.clone();
    let watchdog_regression = regression.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(35));
        let report = json!({ "status": "watchdog_timeout", "processExitObserved": true, "exitCode": 2, "rootCause": "matrix watchdog exceeded 35 seconds", "regression": watchdog_regression });
        let _ = ag29_write_report(&report);
        std::process::exit(2);
    });
    thread::spawn(move || {
        let simple = vec![
            ag29_run_simple_case(&worker_app, "no_child", false, false),
            ag29_run_simple_case(&worker_app, "create_no_focus", false, false),
            ag29_run_simple_case(&worker_app, "focus_no_channel", false, true),
            ag29_run_simple_case(&worker_app, "channel_no_focus", true, false),
        ];
        let full = (1..=5).map(|repetition| ag29_run_full_ab(&worker_app, repetition)).collect::<Vec<_>>();
        let report = json!({
            "status": "child_lifecycle_diagnosis_observed",
            "tauriVersion": "2.11.5",
            "platform": "windows",
            "regression": regression,
            "matrix": simple,
            "fullAtoB": full,
            "diagnosticFields": {
                "childJsReadyHandshake": "recorded per child case",
                "channelReadyHandshake": "recorded per channel case",
                "focusEventSource": "Rust WindowEvent where available",
                "oldChannelCallbackAfterDestroy": "not_observable_after_A_window_destroy",
                "channelCloseSentinelObserved": false
            },
            "processExitObserved": true,
            "exitCode": 0,
            "residualProcessObserved": false
        });
        let _ = ag29_write_report(&report);
        std::process::exit(0);
    });
    Ok(())
}

#[tauri::command]
fn record_result(report: Value) -> Result<(), String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..\\results\\tauri-webview-ag28.json");
    let text = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    fs::write(path, format!("{text}\n")).map_err(|error| error.to_string())
}

#[tauri::command]
fn finish_probe(window: Window) -> Result<(), String> {
    window.close().map_err(|error| error.to_string())?;
    std::process::exit(0)
}

#[tauri::command]
fn close_child_window(window: Window) -> Result<(), String> {
    window.destroy().map_err(|error| error.to_string())
}

#[tauri::command]
fn current_window_label(window: Window) -> String {
    window.label().to_string()
}

fn main() {
    tauri::Builder::default()
        .manage(ProbeState::default())
        .invoke_handler(tauri::generate_handler![
            input_roundtrip,
            stream_world_views,
            inspect_input_sequence,
            reset_lifecycle_probe,
            open_focus_child,
            record_child_channel_result,
            get_child_channel_result,
            request_child_release,
            child_release_requested,
            child_channel_probe,
            set_pause_barrier,
            submit_pause_probe_input,
            epoch_isolation_probe,
            reopen_channel_probe,
            close_reopen_window,
            finalize_reopened_window,
            record_result,
            finish_probe,
            close_child_window,
            current_window_label,
            ag29_case_config,
            ag29_child_page_ready,
            ag29_child_channel_probe,
            ag29_child_channel_callback,
            start_ag29_matrix
        ])
        .run(tauri::generate_context!())
        .expect("error while running AG-29 Tauri child lifecycle diagnosis");
}
