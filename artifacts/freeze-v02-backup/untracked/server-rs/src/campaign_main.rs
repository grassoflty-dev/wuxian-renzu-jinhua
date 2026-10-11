#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::Value;
use std::sync::Mutex;
use tauri::Manager;
use wuxian_horror_ch1::campaign_store::CampaignStore;
use wuxian_horror_ch1::tp06a_heartbeat::{Tp06Error, Tp06SessionId, Tp06aRuntime};
use wuxian_horror_ch1::world_v3::WorldView;

struct Session(Mutex<CampaignStore>);
struct ContinuousRuntime(Tp06aRuntime);

#[tauri::command]
fn campaign_view(session: tauri::State<Session>) -> Result<Value, String> {
    let store = session.0.lock().map_err(|_| "会话繁忙，请重新启动游戏")?;
    Ok(store.view())
}

#[tauri::command]
fn campaign_action(
    session: tauri::State<Session>,
    action: String,
    args: Option<Value>,
) -> Result<Value, String> {
    let mut store = session.0.lock().map_err(|_| "会话繁忙，请重新启动游戏")?;
    Ok(store.action(&action, &args.unwrap_or_else(|| serde_json::json!({}))))
}

#[tauri::command]
fn campaign_save(session: tauri::State<Session>) -> Result<(), String> {
    session.0.lock().map_err(|_| "会话繁忙")?.flush()
}

#[tauri::command]
fn tp06a_start(
    runtime: tauri::State<ContinuousRuntime>,
    channel: tauri::ipc::Channel<WorldView>,
    world_epoch: u64,
) -> Result<Tp06SessionId, Tp06Error> {
    runtime.0.start(channel, world_epoch)
}

#[tauri::command]
fn tp06a_stop(
    runtime: tauri::State<ContinuousRuntime>,
    session_id: Tp06SessionId,
) -> Result<(), Tp06Error> {
    runtime.0.stop(session_id)
}

fn flush_on_close(window: &tauri::Window) {
    window.state::<ContinuousRuntime>().0.close();
    let session = window.state::<Session>();
    match session.0.lock() {
        Ok(mut store) => {
            if let Err(error) = store.flush() {
                // Closing must not turn a damaged save into a fresh file. The
                // store keeps the original and reports the recovery path.
                eprintln!("[campaign] 退出保存跳过：{error}");
            }
        }
        Err(_) => eprintln!("[campaign] 退出保存跳过：会话繁忙或已损坏"),
    };
}

#[tauri::command]
fn campaign_window(window: tauri::Window, width: u32, height: u32) -> Result<(), String> {
    let (width, height) = match (width, height) {
        (1280, 720) | (1920, 1080) | (2560, 1440) => (width, height),
        _ => return Err("不支持的分辨率".into()),
    };
    window
        .set_size(tauri::LogicalSize::new(width, height))
        .map_err(|e| e.to_string())
}

fn main() {
    let path = std::env::current_exe()
        .expect("executable path")
        .parent()
        .expect("executable directory")
        .join("data")
        .join("campaign-v2.json");
    tauri::Builder::default()
        .manage(Session(Mutex::new(CampaignStore::load(path))))
        .manage(ContinuousRuntime(Tp06aRuntime::default()))
        .invoke_handler(tauri::generate_handler![
            campaign_view,
            campaign_action,
            campaign_save,
            campaign_window,
            tp06a_start,
            tp06a_stop
        ])
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                flush_on_close(window);
            }
        })
        .run(tauri::generate_context!())
        .expect("无限人族进化启动失败");
}
