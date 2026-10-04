use std::time::{SystemTime, UNIX_EPOCH};
use wuxian_horror_ch1::{formal_runtime::FormalRuntime, save_v6};

#[test]
fn loading_slot_return_cancels_exact_entry_without_overwrite_or_create() {
    let root = std::env::temp_dir().join(format!("entry-slot-cancel-{}-{}", std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime.pause().unwrap();
    runtime.save().unwrap();
    runtime.save_slot("living", "Living", true).unwrap();
    let save_bytes = std::fs::read(save_v6::save_path(&root)).unwrap();
    let slot_path = root.join("slots/living/slot-v6.json");
    let slot_bytes = std::fs::read(&slot_path).unwrap();
    for (slot, create) in [("living", false), ("never-create", true)] {
        let prepared = runtime.reset_new().unwrap();
        let token = prepared.entry_token.unwrap();
        assert_eq!(runtime.resume().unwrap_err(), "E_SCENE_ENTRY_NOT_READY");
        let returned = runtime.return_to_hub_slot(slot, "Must not write", create).unwrap();
        assert!(returned.entry_token.is_none());
        assert_eq!(returned.server_tick, prepared.server_tick);
        assert_eq!(runtime.scene_ready(&token, false).unwrap_err(), "E_SCENE_ENTRY_STALE_TOKEN");
        assert_eq!(std::fs::read(&slot_path).unwrap(), slot_bytes);
        assert_eq!(std::fs::read(save_v6::save_path(&root)).unwrap(), save_bytes);
        assert!(!root.join("slots/never-create").exists());
    }
    drop(runtime);
    std::fs::remove_dir_all(root).unwrap();
}
