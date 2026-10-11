use serde_json::Value;
use wuxian_horror_ch1::formal_runtime::FormalRuntime;

const TAURI_CONFIG: &str = include_str!("../tauri.conf.json");
const MAIN_RS: &str = include_str!("../src/main.rs");
const ENTRY_HTML: &str = include_str!("../release-ui/index.html");
const ENTRY_APP: &str = include_str!("../release-ui/js/app.js");
const BABYLON_MANIFEST: &str = include_str!("../../governance/dependencies/babylonjs.json");

#[test]
fn tauri_uses_only_the_formal_babylon_frontend() {
    let config: Value = serde_json::from_str(TAURI_CONFIG).unwrap();
    assert_eq!(config["build"]["frontendDist"], "./release-ui");

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
fn formal_page_loads_only_the_fixed_babylon_and_view_modules() {
    let expected = [
        "vendor/babylon.js",
        "js/tps-renderer.js",
        "js/capability-hud.js",
        "js/presentation-v3.js",
        "js/renderer-boot.js",
        "js/app.js",
    ];
    let positions: Vec<_> = expected
        .iter()
        .map(|path| ENTRY_HTML.find(&format!("src=\"{path}\"")).expect(path))
        .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    for legacy in ["three.min.js", "js/renderer.js", "js/hive-side2d.js"] {
        assert!(!ENTRY_HTML.contains(&format!("src=\"{legacy}\"")));
    }
}

#[test]
fn formal_app_has_no_legacy_ipc_or_network_fallback() {
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
            !ENTRY_APP.contains(command),
            "legacy IPC reference remains: {command}"
        );
    }
    for fallback in ["fetch(", "XMLHttpRequest", "window.location"] {
        assert!(
            !ENTRY_APP.contains(fallback),
            "fallback path remains: {fallback}"
        );
    }
    for command in [
        "formal_snapshot",
        "formal_new",
        "formal_submit_input",
        "formal_interact",
        "formal_save",
        "formal_continue",
        "formal_has_save",
    ] {
        assert!(
            ENTRY_APP.contains(command),
            "formal IPC reference missing: {command}"
        );
    }
    assert!(ENTRY_APP.contains("入口失败关闭"));
    assert!(ENTRY_APP.contains("formal_has_save"));
    assert!(ENTRY_APP.contains("formal_continue"));
    assert!(ENTRY_APP.contains("formal_save"));
    assert!(ENTRY_APP.contains("旧版 Grid 存档不会被读取"));
}

#[test]
fn new_game_resets_formal_runtime_with_a_fresh_epoch() {
    let runtime = FormalRuntime::new().unwrap();
    let before = runtime.snapshot().unwrap();
    let after = runtime.reset_new().unwrap();

    assert_eq!(after.world_id, "grey_hive");
    assert_eq!(after.world_epoch, before.world_epoch + 1);
    assert_eq!(after.server_tick, 0);
    assert_eq!(after.ack_seq, 0);
    assert_eq!(after.progression.current_world_id, "grey_hive");
}

#[test]
fn vendor_manifest_pins_exact_official_package_and_license() {
    let manifest: Value = serde_json::from_str(BABYLON_MANIFEST).unwrap();
    assert_eq!(manifest["name"], "babylonjs");
    assert_eq!(manifest["version"], "9.26.2");
    assert_eq!(manifest["license"], "Apache-2.0");
    assert_eq!(manifest["sourceTag"], "9.26.2");
    assert_eq!(
        manifest["integrity"],
        "sha512-UPvVLHT/6lus8j3gqrghhImfKXG2IPBaqBlmOJYWj6oTqVb/yZ/8MYh3+yHdX6Gl7oNvJvcyRrenm2K6xzOT2g=="
    );
}
