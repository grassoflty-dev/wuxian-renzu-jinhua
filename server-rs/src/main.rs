//! 无限人族进化：原生三世界测试版入口。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::{json, Value};
use std::fs;
use wuxian_horror_ch1::formal_runtime::{
    CapabilityCommandRequest, CombatIntentRequest, FormalRuntime,
};

mod native_release_audit;
mod native_bundle_identity {
    include!(concat!(env!("OUT_DIR"), "/native_bundle_identity.rs"));
}

#[tauri::command]
fn native_build_identity() -> Value {
    let build_identity: Value = serde_json::from_str(
        native_bundle_identity::NATIVE_BUILD_IDENTITY_JSON,
    )
    .expect("build script generated valid native build identity JSON");
    json!({
        "schemaVersion": 1,
        "buildIdentity": build_identity,
        "sidecarSha256": native_bundle_identity::NATIVE_BUNDLE_SIDECAR_SHA256,
        "entry": {
            "path": native_bundle_identity::NATIVE_BUNDLE_ENTRY_PATH,
            "sizeBytes": native_bundle_identity::NATIVE_BUNDLE_ENTRY_SIZE_BYTES,
            "sha256": native_bundle_identity::NATIVE_BUNDLE_ENTRY_SHA256,
        },
        "fileCount": native_bundle_identity::NATIVE_BUNDLE_FILE_COUNT,
    })
}


#[tauri::command]
fn formal_snapshot(
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::WorldSnapshot, String> {
    runtime
        .snapshot()
        .map(wuxian_horror_ch1::world_v3::WorldSnapshot::from_view)
}

#[tauri::command]
fn formal_presentation_events(
    world_epoch: u64,
    after_event_id: u64,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<Vec<wuxian_horror_ch1::world_v3::PresentationEvent>, String> {
    runtime.presentation_events_since(world_epoch, after_event_id)
}

#[tauri::command]
fn formal_sound_cues(
    world_epoch: u64,
    after_event_id: u64,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<Vec<wuxian_horror_ch1::world_v3::SoundCueEvent>, String> {
    runtime.sound_cues_since(world_epoch, after_event_id)
}

#[tauri::command]
fn formal_new(
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    native_new_journey_stage("command_received");
    let view = match runtime.reset_new() {
        Ok(view) => {
            native_new_journey_stage("reset_new_returned");
            view
        }
        Err(error) => {
            native_new_journey_stage("reset_new_failed");
            return Err(error);
        }
    };
    let receipt = wuxian_horror_ch1::world_v3::CommandReceipt::from_view("new", view);
    native_new_journey_stage("receipt_constructed");
    Ok(receipt)
}

/// Opt-in native timing trace. Failure to write diagnostics never changes gameplay.
fn native_new_journey_stage(stage: &str) {
    let Some(directory) = std::env::var_os("WUXIAN_NATIVE_DIAGNOSTICS_DIR") else {
        return;
    };
    if directory.is_empty() {
        return;
    }
    let directory = std::path::PathBuf::from(directory);
    if fs::create_dir_all(&directory).is_err() {
        return;
    }
    use std::io::Write;
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("new-journey-stages.log"))
    {
        let at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or_default();
        let _ = writeln!(file, "{at_ms} pid={} {stage}", std::process::id());
    }
}

#[tauri::command]
fn formal_submit_input(
    sample: wuxian_horror_ch1::continuous_input::InputSample,
    combat: Vec<CombatIntentRequest>,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    let command_id = format!("input:{}:{}", sample.world_epoch, sample.seq);
    runtime
        .submit_input(sample, combat)
        .map(|view| wuxian_horror_ch1::world_v3::CommandReceipt::from_view(command_id, view))
}

#[tauri::command]
fn formal_submit_action(
    action: wuxian_horror_ch1::formal_runtime::ActionCommandRequest,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    let command_id = format!("action:{}", action.request_id);
    runtime
        .submit_action(action)
        .map(|view| wuxian_horror_ch1::world_v3::CommandReceipt::from_view(command_id, view))
}

#[tauri::command]
fn formal_interact(
    actor_id: String,
    request_id: String,
    world_epoch: Option<u64>,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::formal_runtime::FormalInteractionResponse, String> {
    wuxian_horror_ch1::scene_route_commands::interaction(
        &runtime, &actor_id, &request_id, world_epoch,
    )
}

#[tauri::command]
fn formal_scene_transition(
    id: String,
    request_id: String,
    world_epoch: u64,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    wuxian_horror_ch1::scene_route_commands::transition(&runtime, &id, &request_id, world_epoch)
}

#[tauri::command]
fn formal_scene_checkpoint(
    id: String,
    request_id: String,
    world_epoch: u64,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    wuxian_horror_ch1::scene_route_commands::checkpoint(&runtime, &id, &request_id, world_epoch)
}

#[tauri::command]
fn formal_environment_control(id: String, request_id: String, world_epoch: u64, runtime: tauri::State<'_, FormalRuntime>)
    -> Result<wuxian_horror_ch1::world_v3::CommandReceipt,String> {
    wuxian_horror_ch1::scene_route_commands::environment_control(&runtime,&id,&request_id,world_epoch)
}

#[tauri::command]
fn formal_scene_trigger(
    id: String,
    request_id: String,
    world_epoch: u64,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    wuxian_horror_ch1::scene_route_commands::trigger(&runtime, &id, &request_id, world_epoch)
}

#[tauri::command]
fn formal_world_gate(
    id: String,
    request_id: String,
    world_epoch: u64,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    wuxian_horror_ch1::scene_route_commands::world_gate(&runtime, &id, &request_id, world_epoch)
}

#[tauri::command]
fn formal_mission_terminal_status(
    id: String,
    request_id: String,
    world_epoch: u64,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    wuxian_horror_ch1::scene_route_commands::mission_terminal_status(
        &runtime,
        &id,
        &request_id,
        world_epoch,
    )
}

#[tauri::command]
fn formal_capability_terminal_status(
    id: String,
    request_id: String,
    world_epoch: u64,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    wuxian_horror_ch1::scene_route_commands::capability_terminal_status(
        &runtime,
        &id,
        &request_id,
        world_epoch,
    )
}

#[tauri::command]
fn formal_save_rest_terminal(
    id: String,
    request_id: String,
    world_epoch: u64,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    wuxian_horror_ch1::scene_route_commands::save_rest_terminal(
        &runtime,
        &id,
        &request_id,
        world_epoch,
    )
}

#[tauri::command]
fn formal_build_command(
    command: wuxian_horror_ch1::formal_runtime::build_ui::BuildCommandRequest,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    runtime.apply_build_command(command)
}

#[tauri::command]
fn formal_capability_command(
    command: CapabilityCommandRequest,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    runtime.apply_capability_command(command).map(|view| {
        wuxian_horror_ch1::world_v3::CommandReceipt::from_view("capability-command", view)
    })
}

#[tauri::command]
fn formal_choose_first_enhancement(
    capability_id: String,
    context: wuxian_horror_ch1::formal_runtime::EnhancementTerminalContext,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    runtime
        .choose_first_enhancement(&capability_id, &context)
        .map(|view| {
            wuxian_horror_ch1::world_v3::CommandReceipt::from_view(
                format!("enhancement:{capability_id}"),
                view,
            )
        })
}

#[tauri::command]
fn formal_has_save(runtime: tauri::State<'_, FormalRuntime>, default_only: Option<bool>) -> bool {
    if default_only.unwrap_or(false) { runtime.has_default_save() } else { runtime.has_save() }
}

#[tauri::command]
fn formal_save(
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    runtime
        .save()
        .map(|view| wuxian_horror_ch1::world_v3::CommandReceipt::from_view("save", view))
}

#[tauri::command]
fn formal_continue(
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    runtime
        .continue_saved()
        .map(|view| wuxian_horror_ch1::world_v3::CommandReceipt::from_view("continue", view))
}

#[tauri::command]
fn formal_pause(
    context: Option<wuxian_horror_ch1::formal_runtime::SessionContext>,
    command_sequence: Option<u64>,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    match context.as_ref() {
        Some(context) => runtime.pause_context_ordered(context,
            command_sequence.ok_or("E_LIFECYCLE_COMMAND_SEQUENCE_REQUIRED")?),
        None => runtime.pause(),
    }.map(|view| wuxian_horror_ch1::world_v3::CommandReceipt::from_view("pause", view))
}

#[tauri::command]
fn formal_scene_ready(
    token: wuxian_horror_ch1::world_v3::SceneEntryToken,
    remain_paused: bool,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    runtime.scene_ready(&token, remain_paused).map(|view|
        wuxian_horror_ch1::world_v3::CommandReceipt::from_view(format!("scene-ready:{}", token.generation), view))
}

#[tauri::command]
fn formal_resume(
    context: Option<wuxian_horror_ch1::formal_runtime::SessionContext>,
    command_sequence: Option<u64>,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    match context.as_ref() {
        Some(context) => runtime.resume_context_ordered(context,
            command_sequence.ok_or("E_LIFECYCLE_COMMAND_SEQUENCE_REQUIRED")?),
        None => runtime.resume(),
    }.map(|view| wuxian_horror_ch1::world_v3::CommandReceipt::from_view("resume", view))
}

#[tauri::command]
fn formal_return(
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    runtime
        .return_to_hub()
        .map(|view| wuxian_horror_ch1::world_v3::CommandReceipt::from_view("return", view))
}

#[tauri::command]
fn formal_list_save_slots(
    runtime: tauri::State<'_, FormalRuntime>,
) -> Vec<wuxian_horror_ch1::save_slots::SlotSummary> {
    runtime.list_save_slots()
}

#[tauri::command]
fn formal_save_slot(
    slot_id: String,
    display_name: String,
    create: bool,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    runtime
        .save_slot(&slot_id, &display_name, create)
        .map(|view| wuxian_horror_ch1::world_v3::CommandReceipt::from_view("save-slot", view))
}

#[tauri::command]
fn formal_continue_slot(
    slot_id: String,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    runtime
        .continue_slot(&slot_id)
        .map(|view| wuxian_horror_ch1::world_v3::CommandReceipt::from_view("continue-slot", view))
}

#[tauri::command]
fn formal_return_slot(
    slot_id: String,
    display_name: String,
    create: bool,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::world_v3::CommandReceipt, String> {
    runtime
        .return_to_hub_slot(&slot_id, &display_name, create)
        .map(|view| wuxian_horror_ch1::world_v3::CommandReceipt::from_view("return-slot", view))
}

fn export_compiled_release_audit(
    context: &tauri::Context<tauri::Wry>,
    output: &std::path::Path,
) -> Result<(), String> {
    use native_release_audit::{Input, NativeConfig, Role};
    use std::borrow::Cow;
    let mut keys: Vec<String> = context.assets().iter().map(|(key, _)| key.into_owned()).collect();
    keys.sort();
    if keys.len() > native_release_audit::MAX_FILES {
        return Err("E_AUDIT_LIMIT".into());
    }
    let web = keys.into_iter().map(|key| {
        let path = key.strip_prefix('/').ok_or("E_AUDIT_EMBEDDED_ASSET_KEY")?;
        let bytes = context.assets().get(&key.as_str().into())
            .ok_or("E_AUDIT_EMBEDDED_ASSET_DECODE")?;
        Ok(Input { logical_path: format!("web/{path}"), role: Role::WebAsset, bytes })
    });
    let scenes = wuxian_horror_ch1::production_scene_bootstrap::embedded_scene_json()?;
    let scene_ids = wuxian_horror_ch1::production_scene_bootstrap::EMBEDDED_SCENE_IDS;
    let scene_inputs = scene_ids.iter().zip(scenes).map(|(id, bytes)| Ok(Input {
        logical_path: format!("scenes/{id}.json"), role: Role::CompiledScene,
        bytes: Cow::Borrowed(bytes.as_bytes()),
    }));
    let icon = context.default_window_icon().ok_or("E_AUDIT_NATIVE_ICON_MISSING")?;
    let config = context.config();
    let native_config = NativeConfig {
        has_external_resources: config.bundle.resources.is_some(),
        has_external_binaries: config.bundle.external_bin.is_some(),
        embedded_frontend: matches!(config.build.frontend_dist,
            Some(tauri::utils::config::FrontendDist::Directory(_))),
        icon_width: icon.width(), icon_height: icon.height(),
    };
    let icon_input = Ok(Input { logical_path: "native/default-window-icon.rgba".into(),
        role: Role::WindowIconRgba, bytes: Cow::Borrowed(icon.rgba()) });
    native_release_audit::export(output,
        &std::env::current_exe().map_err(|_| "E_AUDIT_EXECUTABLE_PATH")?,
        native_build_identity(), native_config,
        web.chain(scene_inputs).chain(std::iter::once(icon_input)))?;
    Ok(())
}

fn main() {
    let mode = native_release_audit::parse_args(std::env::args_os().skip(1))
        .unwrap_or_else(|error| { eprintln!("{error}"); std::process::exit(2) });
    let context = tauri::generate_context!();
    if let native_release_audit::Mode::Export(output) = mode {
        if let Err(error) = export_compiled_release_audit(&context, &output) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    tauri::Builder::default()
        .setup(|app| {
            use tauri::Manager;
            let save_dir = std::env::var_os("WUXIAN_FORMAL_SAVE_DIR")
                .map(std::path::PathBuf::from)
                .map(Ok)
                .unwrap_or_else(|| app.path().app_data_dir())?;
            let runtime =
                FormalRuntime::new_with_save_dir(save_dir).map_err(std::io::Error::other)?;
            wuxian_horror_ch1::production_scene_bootstrap::install(&runtime)
                .map_err(std::io::Error::other)?;
            app.manage(runtime);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            native_build_identity,
            formal_snapshot,
            formal_presentation_events,
            formal_sound_cues,
            formal_new,
            formal_submit_input,
            formal_submit_action,
            formal_interact,
            formal_scene_transition,
            formal_scene_checkpoint,
            formal_scene_trigger,
            formal_environment_control,
            formal_world_gate,
            formal_mission_terminal_status,
            formal_capability_terminal_status,
            formal_save_rest_terminal,
            formal_capability_command,
            formal_build_command,
            formal_choose_first_enhancement,
            formal_has_save,
            formal_save,
            formal_continue,
            formal_pause,
            formal_resume,
            formal_scene_ready,
            formal_return,
            formal_list_save_slots,
            formal_save_slot,
            formal_continue_slot,
            formal_return_slot
        ])
        .run(context)
        .expect("error while running tauri application");
}
