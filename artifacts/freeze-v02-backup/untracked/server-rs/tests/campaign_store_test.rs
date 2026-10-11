use serde_json::{json, Value};
use std::{
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use wuxian_horror_ch1::{
    campaign::{self, CampaignState},
    campaign_store::CampaignStore,
};

fn persistent_test_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after epoch")
        .as_nanos();
    let path = env::temp_dir()
        .join("yujie-campaign-store-tests")
        .join(format!("{label}-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&path).expect("create isolated test directory");
    // Deliberately do not remove this directory. It is a QA artifact for
    // inspecting pending files, timestamp archives, and the untouched source.
    path
}

fn save_path(label: &str) -> PathBuf {
    persistent_test_dir(label)
        .join("data")
        .join("campaign-v2.json")
}

fn default_raw_state() -> Value {
    serde_json::to_value(CampaignState::default()).expect("default state serializes")
}

fn write_json(path: &PathBuf, value: &Value) -> Vec<u8> {
    let bytes = serde_json::to_vec_pretty(value).expect("test JSON serializes");
    fs::create_dir_all(path.parent().expect("test save parent")).expect("create save parent");
    fs::write(path, &bytes).expect("write test save");
    bytes
}

fn find_new_archive(path: &PathBuf) -> Vec<PathBuf> {
    let parent = path.parent().expect("test save parent");
    fs::read_dir(parent)
        .expect("read test save parent")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|candidate| {
            candidate
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("campaign-v2-before-new-") && name.ends_with(".json"))
                .unwrap_or(false)
        })
        .collect()
}

#[test]
fn empty_and_old_version_payloads_never_replace_the_source() {
    for (label, payload) in [
        ("empty", json!({})),
        ("version-one", {
            let mut value = default_raw_state();
            value["version"] = json!(1);
            value
        }),
    ] {
        let path = save_path(label);
        let before = write_json(&path, &payload);
        let mut store = CampaignStore::load(path.clone());

        let view = store.view();
        assert!(view["save_warning"].as_str().is_some(), "{label} must warn");
        assert_eq!(view["read_only_recovery_required"], json!(true));
        assert!(store.flush().is_err(), "{label} flush must be blocked");
        assert_eq!(fs::read(&path).expect("read untouched source"), before);
        assert!(!store.pending_path_for_diagnostics().exists());

        let action_view = store.action("move", &json!({"dx": 1, "dy": 0}));
        assert!(
            action_view["save_warning"].as_str().is_some(),
            "{label} action must keep warning"
        );
        assert_eq!(
            fs::read(&path).expect("read untouched source after action"),
            before
        );
    }
}

#[test]
fn malformed_map_never_reaches_flush() {
    let mut state = CampaignState::default();
    campaign::act(&mut state, "enter", &json!({"world_id": "grey_hive"}));
    let mut broken = serde_json::to_value(state).expect("world state serializes");
    broken["map"]["tiles"][0] = json!(".");
    let path = save_path("broken-map");
    let before = write_json(&path, &broken);

    let mut store = CampaignStore::load(path.clone());
    let view = store.view();
    assert!(view["save_warning"].as_str().is_some());
    assert!(view["save_warning"]
        .as_str()
        .expect("map warning")
        .contains("tiles"));
    assert!(store.flush().is_err());
    assert_eq!(fs::read(&path).expect("read untouched broken map"), before);
}

#[test]
fn pending_write_failure_is_diagnostic_and_preserves_target() {
    let path = save_path("pending-failure");
    let before = write_json(&path, &default_raw_state());
    let pending = path.with_extension("json.pending");
    fs::create_dir(&pending).expect("make pending path unwritable");

    let mut store = CampaignStore::load(path.clone());
    let error = store.flush().expect_err("directory pending must fail");
    assert!(error.contains("pending"));
    assert!(pending.is_dir());
    assert_eq!(fs::read(&path).expect("read untouched target"), before);
    assert!(store.view()["save_warning"].as_str().is_some());
    assert_eq!(store.view()["pending_save"], json!(true));
}

#[test]
fn explicit_new_archives_raw_previous_bytes_before_saving() {
    let path = save_path("new-archive");
    let before = write_json(&path, &default_raw_state());
    let mut store = CampaignStore::load(path.clone());

    let result = store.action("new", &json!({}));
    assert_eq!(result["version"], json!(2));
    let archives = find_new_archive(&path);
    assert_eq!(archives.len(), 1, "new must create one timestamp archive");
    assert_eq!(
        fs::read(&archives[0]).expect("read timestamp archive"),
        before
    );
    assert!(path.is_file());
    assert!(store.view()["save_warning"].is_null());
}

#[test]
fn high_frequency_actions_are_throttled_but_checkpoints_save_now() {
    let path = save_path("throttle");
    let mut store = CampaignStore::load(path.clone());

    store.action("move", &json!({"dx": 1, "dy": 0}));
    assert!(path.is_file(), "first action establishes a save");
    let backup = path.with_extension("json.bak");
    assert!(
        !backup.exists(),
        "first save has no previous target to back up"
    );

    store.action("attack", &json!({"dx": 1, "dy": 0}));
    assert!(
        !backup.exists(),
        "immediate high-frequency action is throttled"
    );

    store.action("equip", &json!({"id": "echo"}));
    assert!(backup.is_file(), "checkpoint action saves immediately");
}

/// 更名兼容性：title 既是游戏标题，也是存档格式标识。老存档里写的还是
/// "余界：归航协议"（本仓库既有的 data/campaign-v2.json 就是这样），
/// 双接受之后必须能正常读入，并且写回时迁移到新标识。
#[test]
fn legacy_title_save_is_readable_and_migrates_to_the_new_name() {
    let path = save_path("legacy-title");
    let mut legacy = default_raw_state();
    legacy["title"] = json!("余界：归航协议");
    write_json(&path, &legacy);

    let mut store = CampaignStore::load(path.clone());
    let view = store.view();
    assert!(
        view["save_warning"].is_null(),
        "old title must still decode, got warning: {:?}",
        view["save_warning"]
    );
    assert_eq!(view["read_only_recovery_required"], json!(false));
    assert_eq!(view["title"], json!("无限人族进化"));

    // 读入即规范化：写回后存档里只剩新标识。
    store.flush().expect("legacy state must flush");
    let rewritten: Value = serde_json::from_slice(&fs::read(&path).expect("read rewritten save"))
        .expect("rewritten save is JSON");
    assert_eq!(rewritten["title"], json!("无限人族进化"));
    assert!(CampaignStore::load(path).view()["save_warning"].is_null());
}

#[test]
fn unknown_title_is_still_rejected_and_source_kept() {
    let path = save_path("unknown-title");
    let mut payload = default_raw_state();
    payload["title"] = json!("别的战役名");
    let before = write_json(&path, &payload);

    let mut store = CampaignStore::load(path.clone());
    let warning = store.view()["save_warning"]
        .as_str()
        .map(str::to_owned)
        .expect("unknown title must warn");
    assert!(warning.contains("title"), "warning was: {warning}");
    assert!(store.flush().is_err());
    assert_eq!(fs::read(&path).expect("read untouched source"), before);
}
