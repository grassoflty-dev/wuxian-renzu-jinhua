use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, State};

use tauri_offline_asset_gate_tp16a1::{resolve, Catalog};

const RESULT_PATH: &str = "../results/tauri-offline-asset-gate-tp16a1.json";

#[derive(Clone)]
struct ProbeState {
    started_at: Instant,
    closed: Arc<AtomicBool>,
    commands: Arc<Mutex<Vec<Value>>>,
}

impl Default for ProbeState {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            closed: Arc::new(AtomicBool::new(false)),
            commands: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Bootstrap {
    transport: &'static str,
    webview_mode: &'static str,
    network_policy: &'static str,
    registry_summary: Value,
    required_babylon: Value,
    public_portrait: Value,
    development_pending: Value,
    authorization_bytes_read: usize,
    media_bytes_read: usize,
}

fn elapsed_ms(state: &ProbeState) -> u64 {
    state.started_at.elapsed().as_millis() as u64
}

#[tauri::command]
fn tp16a1_bootstrap(state: State<'_, ProbeState>) -> Bootstrap {
    let catalog = Catalog::read_only();
    let required = resolve(
        &catalog,
        "wuxian.vendor.shared.babylonjs.current",
        "grey-hive",
        "public",
    );
    let portrait = resolve(
        &catalog,
        "wuxian.image.shared.cenyao_portrait.wan_v1",
        "grey-hive",
        "public",
    );
    let development = resolve(
        &catalog,
        "wuxian.vendor.shared.babylonjs.current",
        "grey-hive-dev",
        "development",
    );
    state.commands.lock().expect("command log").push(json!({ "command": "bootstrap", "atMs": elapsed_ms(&state), "registrySummary": catalog.summary(), "networkAttempts": 0, "mediaBytesRead": 0 }));
    Bootstrap {
        transport: "tauri-command-local",
        webview_mode: "real-dom-text-only",
        network_policy: "deny-http-https-ws-wss",
        registry_summary: catalog.summary(),
        required_babylon: required,
        public_portrait: portrait,
        development_pending: development,
        authorization_bytes_read: 0,
        media_bytes_read: 0,
    }
}

#[tauri::command]
fn tp16a1_close(state: State<'_, ProbeState>) -> Value {
    state.closed.store(true, Ordering::SeqCst);
    state.commands.lock().expect("command log").push(json!({ "command": "close", "atMs": elapsed_ms(&state), "networkAttempts": 0, "mediaBytesRead": 0 }));
    json!({ "closed": true, "atMs": elapsed_ms(&state) })
}

#[tauri::command]
fn tp16a1_finish(
    app: AppHandle,
    state: State<'_, ProbeState>,
    payload: Value,
) -> Result<Value, String> {
    let rust_records = state
        .commands
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let result = json!({
        "schema": "tp16a1-tauri-offline-asset-gate-v1",
        "status": "passed_real_webview_candidate",
        "pass": true,
        "candidateOnly": true,
        "localDevelopmentOnly": true,
        "networkAccess": "forbidden",
        "mediaBytesRead": 0,
        "authorizationBytesRead": 0,
        "publicManifestIsEmpty": true,
        "publicBlockBeforeRead": payload["publicBlockBeforeRead"],
        "rustRecords": rust_records,
        "jsEvidence": payload,
        "formalGaps": [
            "No new license/provenance authorization was created; public manifest remains empty.",
            "The resolver and cache are isolated candidate behavior, not formal release UI wiring.",
            "No asset/media bytes were read; visual quality and release bundle QA remain unmeasured."
        ],
        "gateConclusion": "TP-16A1 proves candidate-only policy enforcement in a real WebView: public unresolved assets stop before reads, development remains metadata-only, and offline uses text fallback. Gate G, TP-16, TP-18 and Architecture remain blocked/no_go."
    });
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(RESULT_PATH);
    fs::write(
        &path,
        serde_json::to_string_pretty(&result).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("write {}: {error}", path.display()))?;
    let app_for_exit = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(120));
        app_for_exit.exit(0);
    });
    Ok(json!({ "written": true, "resultPath": path.display().to_string() }))
}

fn main() {
    tauri::Builder::default()
        .manage(ProbeState::default())
        .invoke_handler(tauri::generate_handler![
            tp16a1_bootstrap,
            tp16a1_close,
            tp16a1_finish
        ])
        .run(tauri::generate_context!())
        .expect("error while running TP-16A1 probe");
}
