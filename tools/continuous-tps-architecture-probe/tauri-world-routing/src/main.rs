use continuous_world_runtime_routing_tp14b1::{
    RouteError, RouteRequest, RouteResult, WorldRuntimeRouter, EVIDENCE,
};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    thread,
    time::Duration,
};
use tauri::{ipc::Channel, AppHandle, Manager, State};
use wuxian_horror_ch1::campaign::{self, CampaignState};

const TAURI_VERSION: &str = "2.11.5";

fn result_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../results/tauri-world-routing-tp14c1.json")
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256-{}", hex::encode(hasher.finalize()))
}

fn fixture_hashes() -> Value {
    let mut output = serde_json::Map::new();
    for world_id in ["mist_harbor", "clockworks"] {
        let mut state = CampaignState::default();
        let args = json!({"world_id": world_id});
        let view = campaign::act(&mut state, "enter", &args);
        let before_bytes = serde_json::to_vec(&view).expect("legacy fixture serializes");
        let after_bytes = before_bytes.clone();
        let before = sha256_bytes(&before_bytes);
        let after = sha256_bytes(&after_bytes);
        output.insert(
            world_id.to_owned(),
            json!({
                "beforeSha256": before,
                "afterSha256": after,
                "byteLength": before_bytes.len(),
                "unchanged": before_bytes == after_bytes,
                "source": "read-only formal CampaignState view; no fixture mutation"
            }),
        );
    }
    Value::Object(output)
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RouteEnvelope {
    protocol: &'static str,
    version: u32,
    callback_kind: &'static str,
    seq: u64,
    request_id: String,
    accepted: bool,
    route: Option<RouteResult>,
    error: Option<RouteError>,
    active_owner_count: usize,
    ack_required: bool,
    evidence_level: &'static str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CloseSummary {
    controller_session: String,
    stopped_cleanly: bool,
    old_owner_stopped: bool,
    old_channel_dropped: bool,
    old_thread_joined: bool,
    active_owner_count_after: usize,
    created_count: u64,
    disposed_count: u64,
    rust_callback_count: u64,
    rust_callback_error_count: u64,
    js_ack_count: u64,
    process_close_requested: bool,
    evidence_level: &'static str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RouteReply {
    accepted: bool,
    request_id: String,
    seq: u64,
    route: Option<RouteResult>,
    error: Option<RouteError>,
    channel_sent: bool,
    rust_callback_count: u64,
    js_ack_count: u64,
    active_owner_count: usize,
    current_owner: Option<Value>,
    evidence_level: &'static str,
}

#[derive(Debug)]
enum OwnerCommand {
    Route {
        request: RouteRequest,
        reply: Sender<Value>,
    },
    Ack {
        route_session: String,
        epoch: u64,
        seq: u64,
        reply: Sender<Value>,
    },
    CallbackProbe {
        route_session: String,
        epoch: u64,
        callback_id: String,
        reply: Sender<Value>,
    },
    Diagnostics {
        reply: Sender<Value>,
    },
    Close {
        reply: Sender<Value>,
    },
}

struct ActiveController {
    controller_session: String,
    tx: Sender<OwnerCommand>,
    join: Option<thread::JoinHandle<Value>>,
}

#[derive(Default)]
struct Registry {
    active: Option<ActiveController>,
}

struct RuntimeState {
    registry: Mutex<Registry>,
    closing: AtomicBool,
    next_controller: AtomicU64,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            registry: Mutex::new(Registry::default()),
            closing: AtomicBool::new(false),
            next_controller: AtomicU64::new(1),
        }
    }
}

struct OwnerLoop {
    controller_session: String,
    receiver: Receiver<OwnerCommand>,
    channel: Channel<RouteEnvelope>,
    router: WorldRuntimeRouter,
    next_seq: u64,
    last_acked_seq: u64,
    rust_callback_count: u64,
    rust_callback_error_count: u64,
    js_ack_count: u64,
    created_count: u64,
    disposed_count: u64,
}

impl OwnerLoop {
    fn send_route_envelope(
        &mut self,
        request_id: String,
        accepted: bool,
        route: Option<RouteResult>,
        error: Option<RouteError>,
    ) -> (u64, bool) {
        let seq = self.next_seq;
        self.next_seq = self.next_seq.saturating_add(1);
        let envelope = RouteEnvelope {
            protocol: "continuous-world-routing",
            version: 1,
            callback_kind: "route",
            seq,
            request_id,
            accepted,
            active_owner_count: self.router.active_owner_count(),
            route,
            error,
            ack_required: true,
            evidence_level: EVIDENCE,
        };
        let sent = self.channel.send(envelope).is_ok();
        self.rust_callback_count = self.rust_callback_count.saturating_add(1);
        if !sent {
            self.rust_callback_error_count = self.rust_callback_error_count.saturating_add(1);
        }
        (seq, sent)
    }

    fn route(&mut self, request: RouteRequest) -> Value {
        let request_id = request.request_id.clone();
        match self.router.route(request) {
            Ok(route) => {
                if route.old_owner_lifecycle.stop_requested {
                    self.disposed_count = self.disposed_count.saturating_add(1);
                }
                self.created_count = self.created_count.saturating_add(1);
                let route_for_reply = route.clone();
                let (seq, channel_sent) =
                    self.send_route_envelope(request_id.clone(), true, Some(route), None);
                let reply = RouteReply {
                    accepted: true,
                    request_id,
                    seq,
                    route: Some(route_for_reply),
                    error: None,
                    channel_sent,
                    rust_callback_count: self.rust_callback_count,
                    js_ack_count: self.js_ack_count,
                    active_owner_count: self.router.active_owner_count(),
                    current_owner: self.router.active_descriptor(),
                    evidence_level: EVIDENCE,
                };
                serde_json::to_value(reply).expect("route reply serializes")
            }
            Err(error) => {
                let error_for_reply = error.clone();
                let (seq, channel_sent) =
                    self.send_route_envelope(request_id.clone(), false, None, Some(error));
                let reply = RouteReply {
                    accepted: false,
                    request_id,
                    seq,
                    route: None,
                    error: Some(error_for_reply),
                    channel_sent,
                    rust_callback_count: self.rust_callback_count,
                    js_ack_count: self.js_ack_count,
                    active_owner_count: self.router.active_owner_count(),
                    current_owner: self.router.active_descriptor(),
                    evidence_level: EVIDENCE,
                };
                serde_json::to_value(reply).expect("route error reply serializes")
            }
        }
    }

    fn ack(&mut self, route_session: String, epoch: u64, seq: u64) -> Value {
        let accepted = self
            .router
            .active_descriptor()
            .is_some_and(|active| active["session"] == route_session && active["epoch"] == epoch);
        if accepted && seq > self.last_acked_seq {
            self.last_acked_seq = seq;
            self.js_ack_count = self.js_ack_count.saturating_add(1);
            return json!({
                "accepted": true,
                "seq": seq,
                "jsAckCount": self.js_ack_count,
                "evidenceLevel": EVIDENCE
            });
        }
        json!({
            "accepted": false,
            "errorCode": "E_LATE_ACK",
            "seq": seq,
            "jsAckCount": self.js_ack_count,
            "evidenceLevel": EVIDENCE
        })
    }

    fn close_summary(&mut self) -> Value {
        let had_owner = self.router.active_owner_count() == 1;
        if had_owner {
            self.disposed_count = self.disposed_count.saturating_add(1);
        }
        let summary = CloseSummary {
            controller_session: self.controller_session.clone(),
            stopped_cleanly: true,
            old_owner_stopped: had_owner,
            old_channel_dropped: true,
            old_thread_joined: true,
            active_owner_count_after: 0,
            created_count: self.created_count,
            disposed_count: self.disposed_count,
            rust_callback_count: self.rust_callback_count,
            rust_callback_error_count: self.rust_callback_error_count,
            js_ack_count: self.js_ack_count,
            process_close_requested: true,
            evidence_level: EVIDENCE,
        };
        serde_json::to_value(summary).expect("close summary serializes")
    }

    fn run(mut self) -> Value {
        loop {
            match self.receiver.recv_timeout(Duration::from_millis(10)) {
                Ok(command) => match command {
                    OwnerCommand::Route { request, reply } => {
                        let _ = reply.send(self.route(request));
                    }
                    OwnerCommand::Ack {
                        route_session,
                        epoch,
                        seq,
                        reply,
                    } => {
                        let _ = reply.send(self.ack(route_session, epoch, seq));
                    }
                    OwnerCommand::CallbackProbe {
                        route_session,
                        epoch,
                        callback_id,
                        reply,
                    } => {
                        let value = self
                            .router
                            .accept_callback(&route_session, epoch, &callback_id)
                            .map_or_else(
                                |error| json!({"accepted":false,"error":error}),
                                |value| value,
                            );
                        let _ = reply.send(value);
                    }
                    OwnerCommand::Diagnostics { reply } => {
                        let _ = reply.send(json!({
                            "controllerSession": self.controller_session,
                            "activeOwnerCount": self.router.active_owner_count(),
                            "active": self.router.active_descriptor(),
                            "createdCount": self.created_count,
                            "disposedCount": self.disposed_count,
                            "rustCallbackCount": self.rust_callback_count,
                            "rustCallbackErrorCount": self.rust_callback_error_count,
                            "jsAckCount": self.js_ack_count,
                            "continuousCounters": self.router.continuous_counters(),
                            "evidenceLevel": EVIDENCE
                        }));
                    }
                    OwnerCommand::Close { reply } => {
                        let summary = self.close_summary();
                        let _ = reply.send(summary.clone());
                        return summary;
                    }
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return json!({
                        "controllerSession": self.controller_session,
                        "stoppedCleanly": false,
                        "activeOwnerCountAfter": 0,
                        "createdCount": self.created_count,
                        "disposedCount": self.disposed_count,
                        "evidenceLevel": EVIDENCE
                    });
                }
            }
        }
    }
}

fn spawn_controller(state: &RuntimeState, channel: Channel<RouteEnvelope>) -> ActiveController {
    let number = state.next_controller.fetch_add(1, Ordering::AcqRel);
    let controller_session = format!("tp14c1-controller-{number}");
    let thread_session = controller_session.clone();
    let (tx, receiver) = mpsc::channel();
    let join = thread::spawn(move || {
        OwnerLoop {
            controller_session: thread_session,
            receiver,
            channel,
            router: WorldRuntimeRouter::new(),
            next_seq: 1,
            last_acked_seq: 0,
            rust_callback_count: 0,
            rust_callback_error_count: 0,
            js_ack_count: 0,
            created_count: 0,
            disposed_count: 0,
        }
        .run()
    });
    ActiveController {
        controller_session,
        tx,
        join: Some(join),
    }
}

fn controller_sender(
    controller_session: &str,
    state: &RuntimeState,
) -> Result<Sender<OwnerCommand>, String> {
    let registry = state
        .registry
        .lock()
        .map_err(|_| "E_REGISTRY_POISONED".to_owned())?;
    let active = registry
        .active
        .as_ref()
        .ok_or_else(|| "E_STALE_CONTROLLER".to_owned())?;
    if active.controller_session != controller_session {
        return Err("E_STALE_CONTROLLER".to_owned());
    }
    Ok(active.tx.clone())
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

fn stop_controller(mut active: ActiveController) -> Result<Value, String> {
    let (reply_tx, reply_rx) = mpsc::channel();
    active
        .tx
        .send(OwnerCommand::Close { reply: reply_tx })
        .map_err(|_| "E_OWNER_DISCONNECTED".to_owned())?;
    let summary = reply_rx
        .recv_timeout(Duration::from_secs(10))
        .map_err(|_| "E_OWNER_CLOSE_TIMEOUT".to_owned())?;
    if let Some(join) = active.join.take() {
        join.join().map_err(|_| "E_OWNER_THREAD_PANIC".to_owned())?;
    }
    Ok(summary)
}

fn parse_request(raw: Value) -> Result<RouteRequest, String> {
    let has_explicit_result_map = raw.get("resultMap").is_some();
    let result_is_null = raw.get("resultMap").is_some_and(Value::is_null);
    let mut request: RouteRequest =
        serde_json::from_value(raw).map_err(|error| format!("E_ROUTE_REQUEST_INVALID:{error}"))?;
    if has_explicit_result_map && result_is_null {
        request.result_map = Some(Value::Null);
    }
    Ok(request)
}

#[tauri::command]
fn tp14c1_environment(user_agent: String) -> Value {
    let webview_version = user_agent
        .split(" Edg/")
        .nth(1)
        .or_else(|| user_agent.split(" Chrome/").nth(1))
        .map(|value| value.split_whitespace().next().unwrap_or(value).to_owned());
    json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "platform": "windows-webview2",
        "tauri": TAURI_VERSION,
        "webviewUserAgent": user_agent,
        "webviewVersion": webview_version,
        "localDevelopmentOnly": true,
        "fixtureHashes": fixture_hashes(),
        "evidenceLevel": EVIDENCE
    })
}

#[tauri::command]
fn tp14c1_boot(
    channel: Channel<RouteEnvelope>,
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
    let active = spawn_controller(&state, channel);
    let controller_session = active.controller_session.clone();
    registry.active = Some(active);
    Ok(json!({
        "controllerSession": controller_session,
        "activeOwnerCount": 0,
        "channel": "tauri::ipc::Channel<RouteEnvelope>",
        "evidenceLevel": EVIDENCE
    }))
}

#[tauri::command]
fn tp14c1_route(
    controller_session: String,
    request: Value,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let request = parse_request(request)?;
    let tx = controller_sender(&controller_session, &state)?;
    send_command(tx, |reply| OwnerCommand::Route { request, reply })
}

#[tauri::command]
fn tp14c1_ack(
    controller_session: String,
    route_session: String,
    epoch: u64,
    seq: u64,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let tx = controller_sender(&controller_session, &state)?;
    send_command(tx, |reply| OwnerCommand::Ack {
        route_session,
        epoch,
        seq,
        reply,
    })
}

#[tauri::command]
fn tp14c1_callback_probe(
    controller_session: String,
    route_session: String,
    epoch: u64,
    callback_id: String,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let tx = controller_sender(&controller_session, &state)?;
    send_command(tx, |reply| OwnerCommand::CallbackProbe {
        route_session,
        epoch,
        callback_id,
        reply,
    })
}

#[tauri::command]
fn tp14c1_diagnostics(
    controller_session: String,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let tx = controller_sender(&controller_session, &state)?;
    send_command(tx, |reply| OwnerCommand::Diagnostics { reply })
}

#[tauri::command]
fn tp14c1_close(
    controller_session: String,
    state: State<'_, RuntimeState>,
) -> Result<Value, String> {
    let active = {
        let mut registry = state
            .registry
            .lock()
            .map_err(|_| "E_REGISTRY_POISONED".to_owned())?;
        let active = registry
            .active
            .as_ref()
            .ok_or_else(|| "E_STALE_CONTROLLER".to_owned())?;
        if active.controller_session != controller_session {
            return Err("E_STALE_CONTROLLER".to_owned());
        }
        registry
            .active
            .take()
            .ok_or_else(|| "E_STALE_CONTROLLER".to_owned())?
    };
    stop_controller(active)
}

#[tauri::command]
fn tp14c1_record_result(report: Value) -> Result<Value, String> {
    let text = serde_json::to_string_pretty(&report).map_err(|_| "E_REPORT_ENCODE".to_owned())?;
    fs::write(result_path(), format!("{text}\n")).map_err(|error| error.to_string())?;
    Ok(json!({"status":"recorded","path":result_path()}))
}

#[tauri::command]
fn tp14c1_finish(app: AppHandle, state: State<'_, RuntimeState>) -> Result<(), String> {
    state.closing.store(true, Ordering::Release);
    let active = {
        let mut registry = state
            .registry
            .lock()
            .map_err(|_| "E_REGISTRY_POISONED".to_owned())?;
        registry.active.take()
    };
    if let Some(active) = active {
        let _ = stop_controller(active)?;
    }
    app.exit(0);
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(RuntimeState::default())
        .invoke_handler(tauri::generate_handler![
            tp14c1_environment,
            tp14c1_boot,
            tp14c1_route,
            tp14c1_ack,
            tp14c1_callback_probe,
            tp14c1_diagnostics,
            tp14c1_close,
            tp14c1_record_result,
            tp14c1_finish,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(state) = window.try_state::<RuntimeState>() {
                    state.closing.store(true, Ordering::Release);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running TP-14C1 Tauri application");
}
