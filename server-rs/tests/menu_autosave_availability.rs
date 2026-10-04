//! Main-menu availability uses real runtime save files and production scenes.
//! Read probes must never migrate files or bypass a present invalid newer save.
use std::{fs, path::PathBuf, sync::atomic::{AtomicU64, Ordering}};
use wuxian_horror_ch1::{formal_runtime::FormalRuntime, production_scene_bootstrap as production, save_v5, save_v6};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Saves(PathBuf);
impl Saves {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("wuxian-menu-autosave-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        assert!(!path.exists()); Self(path)
    }
    fn runtime(&self) -> FormalRuntime {
        let runtime = FormalRuntime::new_with_save_dir(self.0.clone()).unwrap();
        production::install(&runtime).unwrap(); runtime
    }
}
impl Drop for Saves { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
fn start_paused(runtime: &FormalRuntime) {
    let prepared = runtime.reset_new().unwrap();
    runtime.scene_ready(prepared.entry_token.as_ref().unwrap(), true).unwrap();
}
fn files(root: &std::path::Path) -> Vec<(String, Vec<u8>)> {
    fn visit(root: &std::path::Path, at: &std::path::Path, out: &mut Vec<(String, Vec<u8>)>) {
        if let Ok(entries) = fs::read_dir(at) { for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() { visit(root, &path, out); }
            else { out.push((path.strip_prefix(root).unwrap().to_string_lossy().into_owned(), fs::read(path).unwrap())); }
        } }
    }
    let mut out = vec![]; visit(root, root, &mut out); out.sort(); out
}
#[test]
fn default_autosave_survives_new_return_and_fresh_runtime_continue() {
    let saves = Saves::new();
    { let runtime = saves.runtime(); start_paused(&runtime); runtime.return_to_hub().unwrap(); }
    let runtime = saves.runtime(); let before = files(&saves.0);
    assert!(runtime.has_save()); assert!(runtime.list_save_slots().is_empty());
    assert_eq!(files(&saves.0), before, "availability is read only");
    let restored = runtime.continue_saved().unwrap();
    assert_eq!(restored.scene_id, "rs_core_room"); assert!(restored.player.current_hp > 0);
    assert!(restored.entry_token.is_some());
}
#[test]
fn default_named_both_and_absence_keep_existing_availability() {
    let saves = Saves::new(); let runtime = saves.runtime();
    assert!(!runtime.has_save()); assert!(!runtime.has_default_save()); start_paused(&runtime);
    runtime.save_slot("named", "Named", true).unwrap();
    assert!(runtime.has_save()); assert_eq!(runtime.list_save_slots().len(), 1);
    assert!(!save_v6::save_path(&saves.0).exists()); assert!(!runtime.has_default_save());
    runtime.save().unwrap(); assert!(runtime.has_save()); assert!(runtime.has_default_save());
    let before = files(&saves.0); let before_world = runtime.snapshot().unwrap();
    let before_events = runtime.presentation_events_since(before_world.world_epoch, 0).unwrap();
    for _ in 0..3 { assert!(runtime.has_default_save()); assert!(runtime.has_save()); }
    assert_eq!(files(&saves.0), before); assert_eq!(runtime.snapshot().unwrap(), before_world);
    assert_eq!(runtime.presentation_events_since(before_world.world_epoch, 0).unwrap(), before_events);
    let restored = runtime.continue_slot("named").unwrap(); assert!(restored.player.current_hp > 0);
}
#[test]
fn corrupt_or_dead_files_never_advertise_a_living_save() {
    for dead in [false, true] {
        let saves = Saves::new(); let runtime = saves.runtime(); start_paused(&runtime);
        runtime.save().unwrap(); runtime.save_slot("broken", "Broken", true).unwrap();
        for (path, nested) in [(save_v6::save_path(&saves.0), false), (saves.0.join("slots/broken/slot-v6.json"), true)] {
            if dead {
                let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                if nested { value["save"]["save"]["player"]["currentHp"] = 0.into(); }
                else { value["save"]["player"]["currentHp"] = 0.into(); }
                fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
            } else { fs::write(path, b"invalid JSON").unwrap(); }
        }
        let before = files(&saves.0);
        assert!(!runtime.has_save()); assert!(runtime.list_save_slots().iter().all(|slot| !slot.valid));
        assert!(runtime.continue_saved().is_err()); assert!(runtime.continue_slot("broken").is_err());
        assert_eq!(files(&saves.0), before);
    }
}
#[test]
fn corrupt_winning_v6_cannot_advertise_valid_older_v5() {
    let saves = Saves::new(); let runtime = saves.runtime(); start_paused(&runtime); runtime.save().unwrap();
    let current = save_v6::read_save(&saves.0).unwrap(); save_v5::write_save(&saves.0, &current.save).unwrap();
    fs::write(save_v6::save_path(&saves.0), b"invalid winning v6").unwrap();
    let before = files(&saves.0); assert!(runtime.list_save_slots().is_empty());
    assert!(!runtime.has_save()); assert_eq!(runtime.continue_saved().unwrap_err(), "E_SAVE_CORRUPT");
    assert_eq!(files(&saves.0), before);
}
#[test]
fn invalid_v6_path_cannot_advertise_valid_older_v5() {
    let saves = Saves::new(); let runtime = saves.runtime(); start_paused(&runtime); runtime.save().unwrap();
    let current = save_v6::read_save(&saves.0).unwrap(); save_v5::write_save(&saves.0, &current.save).unwrap();
    fs::remove_file(save_v6::save_path(&saves.0)).unwrap(); fs::create_dir(save_v6::save_path(&saves.0)).unwrap();
    let before = files(&saves.0); assert!(!runtime.has_save()); assert!(runtime.continue_saved().is_err());
    assert_eq!(files(&saves.0), before);
}
#[test]
fn v5_availability_is_read_only_and_explicit_continue_still_migrates() {
    let saves = Saves::new(); let runtime = saves.runtime(); start_paused(&runtime); runtime.save().unwrap();
    let current = save_v6::read_save(&saves.0).unwrap(); save_v5::write_save(&saves.0, &current.save).unwrap();
    fs::remove_file(save_v6::save_path(&saves.0)).unwrap(); let before = files(&saves.0);
    assert!(runtime.has_save()); assert!(runtime.list_save_slots().is_empty());
    assert_eq!(files(&saves.0), before); assert!(!save_v6::save_path(&saves.0).exists());
    assert!(runtime.continue_saved().unwrap().player.current_hp > 0);
    assert!(save_v6::save_path(&saves.0).is_file());
    assert_eq!(fs::read(save_v5::save_path(&saves.0)).unwrap(), before[0].1);
}

#[test]
fn legacy_v4_probe_never_migrates_and_explicit_continue_keeps_original() {
    let saves = Saves::new(); fs::create_dir_all(&saves.0).unwrap();
    let original = include_bytes!("fixtures/capability-save-profiles/legacy-v4.json");
    let path = wuxian_horror_ch1::save_v3::save_path(&saves.0); fs::write(&path, original).unwrap();
    let runtime = saves.runtime(); let before = files(&saves.0);
    assert!(runtime.has_default_save()); assert!(runtime.has_save());
    assert!(runtime.list_save_slots().iter().any(|slot| slot.slot_id == "legacy-save-v3" && slot.valid && slot.read_only));
    assert_eq!(files(&saves.0), before);
    assert!(runtime.continue_slot("legacy-save-v3").unwrap().player.current_hp > 0);
    assert_eq!(fs::read(path).unwrap(), original); assert!(save_v6::save_path(&saves.0).is_file());
}
#[test]
fn corrupt_newer_source_disables_legacy_virtual_slot_without_falling_back() {
    for version in [5, 6] {
        let saves = Saves::new(); fs::create_dir_all(&saves.0).unwrap();
        fs::write(wuxian_horror_ch1::save_v3::save_path(&saves.0), include_bytes!("fixtures/capability-save-profiles/legacy-v4.json")).unwrap();
        fs::write(if version == 5 { save_v5::save_path(&saves.0) } else { save_v6::save_path(&saves.0) }, b"corrupt newer source").unwrap();
        let runtime = saves.runtime(); let before = files(&saves.0);
        assert!(!runtime.has_default_save()); assert!(!runtime.has_save());
        let slots = runtime.list_save_slots(); assert_eq!(slots.len(), 1);
        assert!(!slots[0].valid); assert_eq!(slots[0].error_code.as_deref(), Some("E_SAVE_DEFAULT_UNAVAILABLE"));
        assert!(runtime.continue_saved().is_err()); assert!(runtime.continue_slot("legacy-save-v3").is_err());
        assert_eq!(files(&saves.0), before);
    }
}
#[test]
fn corrupt_default_does_not_hide_independent_valid_named_save() {
    let saves = Saves::new(); let runtime = saves.runtime(); start_paused(&runtime);
    runtime.save_slot("independent", "Independent", true).unwrap(); runtime.save().unwrap();
    fs::write(save_v6::save_path(&saves.0), b"corrupt default").unwrap();
    assert!(!runtime.has_default_save()); assert!(runtime.has_save());
    assert!(runtime.continue_slot("independent").unwrap().player.current_hp > 0);
}
#[cfg(unix)]
#[test]
fn dangling_newer_source_never_advertises_or_migrates_older_save() {
    for version in [5, 6] {
        let saves = Saves::new(); fs::create_dir_all(&saves.0).unwrap();
        fs::write(wuxian_horror_ch1::save_v3::save_path(&saves.0), include_bytes!("fixtures/capability-save-profiles/legacy-v4.json")).unwrap();
        let path = if version == 5 { save_v5::save_path(&saves.0) } else { save_v6::save_path(&saves.0) };
        std::os::unix::fs::symlink(saves.0.join("missing-target"), &path).unwrap();
        let runtime = saves.runtime();
        assert!(!runtime.has_default_save()); assert!(!runtime.has_save());
        assert!(fs::symlink_metadata(&path).unwrap().file_type().is_symlink());
        assert!(!saves.0.join("missing-target").exists());
        assert_eq!(fs::read_dir(&saves.0).unwrap().count(), 2);
    }
}

#[test]
fn structurally_valid_but_unrestorable_default_is_unavailable_without_mutation() {
    for bad in ["unknown-scene", "world-mismatch", "outside-level"] {
        let saves = Saves::new(); let runtime = saves.runtime(); start_paused(&runtime); runtime.save().unwrap();
        let mut save = save_v6::read_save(&saves.0).unwrap();
        match bad {
            "unknown-scene" => {
                save.save.scene_id = "rs_does_not_exist".into();
                for scene in &mut save.save.scene_states { scene.scene_id = "rs_does_not_exist".into(); }
            },
            "world-mismatch" => {
                save.save.scene_id = "gh_entry_maintenance".into();
                for scene in &mut save.save.scene_states { scene.scene_id = "gh_entry_maintenance".into(); }
            },
            _ => save.save.player.position_m.x_m = 99999.0,
        }
        save_v6::write_save(&saves.0, &save).unwrap();
        let before_files = files(&saves.0); let before_world = runtime.snapshot().unwrap();
        assert!(save_v6::read_save(&saves.0).is_ok());
        assert!(!runtime.has_default_save(), "{bad}"); assert!(!runtime.has_save(), "{bad}");
        assert!(runtime.continue_saved().is_err(), "{bad}");
        assert_eq!(runtime.snapshot().unwrap(), before_world); assert_eq!(files(&saves.0), before_files);
    }
}
