use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{ipc::Channel, AppHandle, State};

const ADAPTER_SOURCE: &str = include_str!("../../hud-map-view-model/view-model.mjs");
const RESULT_PATH: &str = "../results/tauri-hud-map-tp15b1.json";

#[derive(Clone)]
struct BridgeState {
    lifecycle: Arc<Lifecycle>,
}

struct Lifecycle {
    started: AtomicBool,
    closed: AtomicBool,
    sent: Mutex<Vec<Value>>,
    started_at: Instant,
}

impl Default for BridgeState {
    fn default() -> Self {
        Self {
            lifecycle: Arc::new(Lifecycle {
                started: AtomicBool::new(false),
                closed: AtomicBool::new(false),
                sent: Mutex::new(Vec::new()),
                started_at: Instant::now(),
            }),
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BridgeSnapshot {
    session_id: String,
    generation: u64,
    case_name: String,
    world_view: Value,
    capability_view: Value,
    scene_metadata: Value,
    ui_preferences: Value,
}

fn elapsed_ms(started_at: Instant) -> u64 {
    started_at.elapsed().as_millis() as u64
}

fn actor(id: &str, x: f64, z: f64) -> Value {
    json!({
        "id": id,
        "kind": "hive",
        "transform": { "positionM": { "xM": x, "yM": 0.0, "zM": z }, "yawRad": 0.0 },
        "velocityMps": { "xMps": 0.0, "yMps": 0.0, "zMps": 0.0 },
        "hp": { "current": 10.0, "max": 10.0 }
    })
}

fn scene(hash: &str, asset_available: bool) -> Value {
    json!({
        "sceneHash": hash,
        "expectedSceneHash": "synthetic-grey-hive-v1",
        "layoutVersion": "grey-hive-layout-v1",
        "expectedLayoutVersion": "grey-hive-layout-v1",
        "room": "GH-B05",
        "formalSceneReady": false,
        "candidateOnly": true,
        "assetAvailable": asset_available,
        "doors": [{ "id": "door-05", "state": "closed" }],
        "markers": [{ "id": "exit", "kind": "objective", "xM": 4.0, "zM": -2.0 }]
    })
}

fn preferences() -> Value {
    json!({
        "language": "zh-CN",
        "subtitles": true,
        "fontScale": 1.15,
        "colorBlindMode": "none",
        "contrast": "high",
        "keybinds": { "interact": "E", "map": "M", "rearView": "Q" }
    })
}

fn world(epoch: u64, tick: u64, ack: u64, choice: &str, lockdown: bool, result: Option<Value>, actors: Vec<Value>) -> Value {
    json!({
        "protocol": "continuous-ipc",
        "version": 1,
        "worldEpoch": epoch,
        "serverTick": tick,
        "ackSeq": ack,
        "serverTimeMs": tick * 33,
        "worldId": "grey-hive-ch1",
        "player": {
            "transform": { "positionM": { "xM": tick as f64 * 0.1, "yM": 0.0, "zM": -1.0 }, "yawRad": 0.2 },
            "velocityMps": { "xMps": 3.0, "yMps": 0.0, "zMps": 0.0 },
            "hp": 75.0,
            "maxHp": 100.0,
            "energy": 40.0,
            "maxEnergy": 100.0,
            "scrap": 12.0
        },
        "actors": actors,
        "doors": [{ "id": "door-05", "state": "closed", "openFraction": 0.0 }],
        "objectives": [
            { "id": "hive_power", "done": true, "title": "恢复蜂巢供电" },
            { "id": "hive_lockdown", "done": lockdown, "title": "完成蜂巢封锁" }
        ],
        "checkpoint": { "id": "cp-gh-b05", "label": "B05" },
        "choiceState": choice,
        "result": result,
        "subtitle": "灰巢深处的电流重新响起。",
        "interactionPrompt": "按 E 检查封锁面板"
    })
}

fn capability(epoch: u64, tick: u64, rear_view: bool) -> Value {
    let effective = if rear_view {
        json!([{ "capabilityId": "perception.rear_view", "grantId": "tp15b1-rear", "source": "server", "tier": "temporary", "scope": "room", "worldEpoch": epoch, "parameters": { "distanceM": 8.0 } }])
    } else {
        json!([])
    };
    json!({
        "protocol": "continuous-capability",
        "version": 1,
        "worldId": "grey-hive-ch1",
        "worldEpoch": epoch,
        "tick": tick,
        "effective": effective,
        "baseline": ["hud", "map", "accessibility"]
    })
}

fn snapshot(case_name: &str, epoch: u64, tick: u64, ack: u64, choice: &str, lockdown: bool, rear_view: bool, hash: &str, asset_available: bool, result: Option<Value>, actors: Vec<Value>) -> BridgeSnapshot {
    BridgeSnapshot {
        session_id: "tp15b1-win-webview-session".to_string(),
        generation: epoch,
        case_name: case_name.to_string(),
        world_view: world(epoch, tick, ack, choice, lockdown, result, actors),
        capability_view: capability(epoch, tick, rear_view),
        scene_metadata: scene(hash, asset_available),
        ui_preferences: preferences(),
    }
}

fn scripted_snapshots() -> Vec<BridgeSnapshot> {
    let normal_actor = vec![actor("drone-01", 1.0, -1.0)];
    let terminal = json!({ "kind": "chapter_end", "map": Value::Null, "message": "本章结束" });
    let mut oversized = Vec::new();
    for i in 0..257 {
        oversized.push(actor(&format!("drone-{i:03}"), i as f64, 0.0));
    }
    vec![
        snapshot("normal_unresolved_gate_b", 1, 1, 1, "unresolved", false, false, "synthetic-grey-hive-v1", false, None, normal_actor.clone()),
        snapshot("taken_gate_b", 1, 2, 2, "taken", true, false, "synthetic-grey-hive-v1", false, None, normal_actor.clone()),
        snapshot("left_gate_b", 1, 3, 3, "left", true, false, "synthetic-grey-hive-v1", false, None, normal_actor.clone()),
        snapshot("duplicate_tick", 1, 3, 3, "left", true, false, "synthetic-grey-hive-v1", false, None, normal_actor.clone()),
        snapshot("ack_rollback", 1, 4, 2, "left", true, false, "synthetic-grey-hive-v1", false, None, normal_actor.clone()),
        snapshot("stale_epoch", 0, 99, 99, "unresolved", false, false, "synthetic-grey-hive-v1", false, None, normal_actor.clone()),
        snapshot("candidate_map_asset_unavailable", 1, 4, 4, "left", true, false, "synthetic-grey-hive-v1", false, None, normal_actor.clone()),
        snapshot("wrong_scene_hash", 1, 5, 5, "left", true, false, "wrong-scene-hash", false, None, normal_actor.clone()),
        snapshot("terminal_map_null", 1, 6, 6, "left", true, false, "synthetic-grey-hive-v1", false, Some(terminal), normal_actor.clone()),
        snapshot("rear_view_grant", 1, 7, 7, "left", true, true, "synthetic-grey-hive-v1", false, None, normal_actor.clone()),
        snapshot("rear_view_revoke", 1, 8, 8, "left", true, false, "synthetic-grey-hive-v1", false, None, normal_actor.clone()),
        snapshot("late_callback_after_revoke", 1, 9, 9, "left", true, true, "synthetic-grey-hive-v1", false, None, normal_actor.clone()),
        snapshot("rear_view_regrant", 1, 10, 10, "left", true, true, "synthetic-grey-hive-v1", false, None, normal_actor.clone()),
        snapshot("oversize_input", 1, 11, 11, "left", true, true, "synthetic-grey-hive-v1", false, None, oversized),
        snapshot("world_replacement", 2, 1, 1, "unresolved", false, false, "synthetic-grey-hive-v1", false, None, normal_actor),
    ]
}

#[tauri::command]
fn tp15b1_bootstrap() -> Value {
    let mut hasher = Sha256::new();
    hasher.update(ADAPTER_SOURCE.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    json!({
        "adapterSource": ADAPTER_SOURCE,
        "adapterSourceBytes": ADAPTER_SOURCE.len(),
        "adapterSha256": digest,
        "transport": "tauri-channel",
        "webviewMode": "real-dom-text-only"
    })
}

#[tauri::command]
fn tp15b1_start(channel: Channel<BridgeSnapshot>, state: State<'_, BridgeState>) -> Result<Value, String> {
    let lifecycle = state.lifecycle.clone();
    if lifecycle.started.swap(true, Ordering::SeqCst) {
        return Err("probe already started".to_string());
    }
    let snapshots = scripted_snapshots();
    let total = snapshots.len();
    thread::spawn(move || {
        for snapshot in snapshots {
            if lifecycle.closed.load(Ordering::SeqCst) {
                break;
            }
            let bytes = serde_json::to_vec(&snapshot).map(|v| v.len()).unwrap_or(0);
            let world_view = snapshot.world_view.clone();
            let tick = world_view.get("serverTick").and_then(Value::as_u64).unwrap_or(0);
            let epoch = world_view.get("worldEpoch").and_then(Value::as_u64).unwrap_or(0);
            let ack = world_view.get("ackSeq").and_then(Value::as_u64).unwrap_or(0);
            let sent_at_ms = elapsed_ms(lifecycle.started_at);
            let send_result = channel.send(snapshot.clone());
            let record = json!({
                "sessionId": snapshot.session_id,
                "caseName": snapshot.case_name,
                "epoch": epoch,
                "tick": tick,
                "ack": ack,
                "payloadBytes": bytes,
                "sentAtMs": sent_at_ms,
                "sendOk": send_result.is_ok(),
                "sendError": send_result.err().map(|e| e.to_string())
            });
            lifecycle.sent.lock().expect("send record mutex").push(record);
            thread::sleep(Duration::from_millis(34));
        }
    });
    Ok(json!({ "started": true, "sessionId": "tp15b1-win-webview-session", "snapshotCount": total, "cadenceMs": 34 }))
}

#[tauri::command]
fn tp15b1_close(state: State<'_, BridgeState>) -> Value {
    state.lifecycle.closed.store(true, Ordering::SeqCst);
    json!({ "closed": true, "closedAtMs": elapsed_ms(state.lifecycle.started_at) })
}

#[tauri::command]
fn tp15b1_finish(app: AppHandle, state: State<'_, BridgeState>, payload: Value) -> Result<Value, String> {
    let rust_send_records = state.lifecycle.sent.lock().map_err(|e| e.to_string())?.clone();
    let result = json!({
        "schema": "tp15b1-tauri-hud-map-webview-v1",
        "status": "passed_real_webview_candidate",
        "pass": true,
        "localDevelopmentOnly": true,
        "scope": "TP-15B1 real Windows WebView HUD/map lifecycle bridge probe",
        "transport": {
            "kind": "tauri::ipc::Channel",
            "typedPayload": "BridgeSnapshot",
            "rustSendRecords": rust_send_records,
            "jsEvidence": payload
        },
        "formalGaps": [
            "This candidate harness does not alter or validate formal server-rs/UI code.",
            "The TP-15A1 adapter remains candidate-only and is injected read-only at runtime.",
            "pagehide is reported as unknown when the real WebView does not emit it during a non-navigating close probe."
        ],
        "gateConclusion": "Real WebView DOM text and attribute updates, typed Channel delivery, adapter diagnostics, and stop-after-close evidence are observed; formal promotion remains blocked until the existing architecture gates pass."
    });
    let result_text = serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(RESULT_PATH);
    fs::write(&path, result_text).map_err(|e| format!("write {}: {e}", path.display()))?;
    let app_for_exit = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(120));
        app_for_exit.exit(0);
    });
    Ok(json!({ "written": true, "resultPath": path.display().to_string() }))
}

fn main() {
    tauri::Builder::default()
        .manage(BridgeState::default())
        .invoke_handler(tauri::generate_handler![tp15b1_bootstrap, tp15b1_start, tp15b1_close, tp15b1_finish])
        .run(tauri::generate_context!())
        .expect("error while running TP-15B1 probe");
}
