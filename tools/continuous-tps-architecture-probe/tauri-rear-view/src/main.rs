use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager, Window, WindowEvent};

const BABYLON_VENDOR: &str = include_str!("../../../../server-rs/release-ui/vendor/babylon.js");
static START: OnceLock<Instant> = OnceLock::new();

fn monotonic_ms() -> f64 {
    START.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
}

#[derive(Default)]
struct ProbeState {
    events: Mutex<Vec<Value>>,
    final_report: Mutex<Option<Value>>,
}

fn result_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..\\results\\tauri-rear-view-tp13b1.json");
    path
}

fn write_report(report: &Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(report).map_err(|e| e.to_string())?;
    fs::write(result_path(), format!("{text}\n")).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_babylon_source() -> Result<String, String> {
    Ok(BABYLON_VENDOR.to_string())
}

#[tauri::command]
fn record_events(state: tauri::State<'_, ProbeState>, events: Vec<Value>) -> Result<(), String> {
    let mut sink = state.events.lock().map_err(|e| e.to_string())?;
    for event in events {
        sink.push(json!({ "rustReceivedAtMs": monotonic_ms(), "event": event }));
    }
    Ok(())
}

#[tauri::command]
fn finish_probe(app: AppHandle, window: Window, state: tauri::State<'_, ProbeState>, report: Value) -> Result<(), String> {
    let events = state.events.lock().map_err(|e| e.to_string())?.clone();
    let mut final_report = report;
    final_report["rust"] = json!({
        "tauriVersion": "2.11.5",
        "vendorInjection": "include_str! runtime command; vendor file not copied",
        "localDevelopmentOnly": true,
        "monotonicMs": monotonic_ms(),
        "eventCount": events.len(),
        "events": events,
        "closeRequested": true,
        "processExitCode": 0
    });
    *state.final_report.lock().map_err(|e| e.to_string())? = Some(final_report.clone());
    write_report(&final_report)?;
    let _ = window.close();
    let app_for_exit = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(250));
        let state = app_for_exit.state::<ProbeState>();
        if let (Ok(events), Ok(mut stored)) = (state.events.lock(), state.final_report.lock()) {
            if let Some(report) = stored.as_mut() {
                report["rust"]["eventCount"] = json!(events.len());
                report["rust"]["events"] = json!(&*events);
                report["rust"]["windowDestroyedObserved"] = json!(events.iter().any(|event| event["event"]["kind"] == "window_destroyed"));
                let _ = write_report(report);
            }
        }
        app_for_exit.exit(0);
    });
    Ok(())
}

fn main() {
    START.get_or_init(Instant::now);
    let state = ProbeState::default();
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![get_babylon_source, record_events, finish_probe])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if matches!(event, WindowEvent::Destroyed) {
                        let state = app_handle.state::<ProbeState>();
                        if let Ok(mut events) = state.events.lock() {
                            events.push(json!({ "rustReceivedAtMs": monotonic_ms(), "event": { "kind": "window_destroyed" } }));
                        };
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("TP-13B1 Tauri runtime failed");
}
