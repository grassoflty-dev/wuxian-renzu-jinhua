#[path = "support/entry_readiness.rs"]
mod entry_readiness;
use entry_readiness::acknowledge_ready;

use serde_json::Value;
use wuxian_horror_ch1::formal_runtime::FormalRuntime;

const TAURI_CONFIG: &str = include_str!("../tauri.conf.json");
const MAIN_RS: &str = include_str!("../src/main.rs");
const ENTRY_HTML: &str = include_str!("../../apps/web/index.html");
const ENTRY_BRIDGE: &str = include_str!("../../apps/web/src/bridge/tauri-client.ts");
const WEB_PACKAGE: &str = include_str!("../../apps/web/package.json");
const PIXI_MANIFEST: &str = include_str!("../../governance/dependencies/pixijs.json");

#[test]
fn tauri_uses_only_the_formal_pixi_frontend() {
    let config: Value = serde_json::from_str(TAURI_CONFIG).unwrap();
    assert_eq!(config["build"]["frontendDist"], "../apps/web/dist");

    let start = MAIN_RS
        .find(".invoke_handler(tauri::generate_handler![")
        .expect("Tauri command allow-list exists");
    let handlers = MAIN_RS[start..]
        .split(".invoke_handler(tauri::generate_handler![")
        .nth(1)
        .unwrap()
        .split("])")
        .next()
        .unwrap();

    for command in [
        "formal_snapshot",
        "formal_new",
        "formal_submit_input",
        "formal_submit_action",
        "formal_presentation_events",
        "formal_interact",
        "formal_capability_command",
        "formal_choose_first_enhancement",
        "formal_has_save",
        "formal_save",
        "formal_continue",
    ] {
        assert!(
            handlers.contains(command),
            "missing formal command {command}"
        );
    }
    for legacy in [
        "api_new",
        "api_continue",
        "api_world_move",
        "api_zone_action",
        "api_scene_goto",
        "api_scene_back",
        "campaign_view",
        "tp06a_start",
    ] {
        assert!(
            !handlers.contains(legacy),
            "legacy handler remains registered: {legacy}"
        );
    }
}

#[test]
fn formal_page_uses_the_vite_pixi_entry() {
    assert!(ENTRY_HTML.contains("<script type=\"module\" src=\"/src/main.ts\"></script>"));
    let package: Value = serde_json::from_str(WEB_PACKAGE).unwrap();
    assert_eq!(package["dependencies"]["pixi.js"], "8.21.0");
    assert!(package["scripts"]["build"].as_str().unwrap().contains("vite build"));
    for legacy in [
        "three.min.js",
        "vendor/babylon.js",
        "js/tps-renderer.js",
        "js/renderer.js",
        "js/hive-side2d.js",
    ] {
        assert!(!ENTRY_HTML.contains(legacy));
    }
}

#[test]
fn formal_bridge_uses_only_formal_ipc() {
    for command in [
        "campaign_view",
        "campaign_action",
        "campaign_save",
        "tp06a_start",
        "tp06a_stop",
        "api_world_move",
        "api_zone_action",
    ] {
        assert!(
            !ENTRY_BRIDGE.contains(command),
            "legacy IPC reference remains: {command}"
        );
    }
    for command in [
        "formal_snapshot",
        "formal_new",
        "formal_submit_input",
        "formal_submit_action",
        "formal_presentation_events",
        "formal_has_save",
        "formal_continue",
        "formal_return",
    ] {
        assert!(
            ENTRY_BRIDGE.contains(command),
            "formal IPC reference missing: {command}"
        );
    }
    assert!(ENTRY_BRIDGE.contains("assertSnapshotV3"));
}

#[test]
fn new_game_resets_formal_runtime_with_a_fresh_epoch() {
    let runtime = FormalRuntime::new().unwrap();
    let before = runtime.snapshot().unwrap();
    let after = acknowledge_ready(&runtime, runtime.reset_new().unwrap());

    assert_eq!(after.world_id, "grey_hive");
    assert_eq!(after.world_epoch, before.world_epoch + 1);
    assert_eq!(after.server_tick, 0);
    assert_eq!(after.ack_seq, 0);
    assert_eq!(after.progression.current_world_id, "grey_hive");
}

#[test]
fn vendor_manifest_pins_pixi_package_and_license() {
    let manifest: Value = serde_json::from_str(PIXI_MANIFEST).unwrap();
    assert_eq!(manifest["package"], "pixi.js");
    assert_eq!(manifest["version"], "8.21.0");
    assert_eq!(manifest["license"], "MIT");
}
