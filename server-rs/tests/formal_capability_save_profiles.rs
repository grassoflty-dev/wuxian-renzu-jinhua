//! Persistence inputs are frozen old-wire fixtures, never synthesized by the current serializer.
use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use wuxian_horror_ch1::{
    effects::EffectSourceKind,
    formal_runtime::{build_v6::PlayerProgressionV6, FormalRuntime},
    save_slots, save_v3,
    save_v5::{self, SaveV5},
    save_v6::{self, SaveV6, EFFECT_SOURCES_VERSION},
    world_persistent_v1::WorldPersistentState,
};

const EMPTY: &str = include_str!("fixtures/capability-save-profiles/legacy-empty.json");
const MIXED: &str = include_str!("fixtures/capability-save-profiles/legacy-mixed.json");
const MATRIX: &str = include_str!("fixtures/capability-save-profiles/legacy-matrix.json");
const SLOT: &str = include_str!("fixtures/capability-save-profiles/legacy-slot.json");
const V5: &str = include_str!("fixtures/capability-save-profiles/legacy-v5.json");
const V4: &str = include_str!("fixtures/capability-save-profiles/legacy-v4.json");
static NEXT: AtomicU64 = AtomicU64::new(1);
struct Root(PathBuf);
impl Root {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "capability-profile-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn file(&self) -> PathBuf {
        save_v6::save_path(&self.0)
    }
    fn backup(&self) -> PathBuf {
        self.0
            .join("formal-save-v6.json.legacy-effect-sources-v1.bak")
    }
    fn put(&self, value: &Value) {
        fs::write(self.file(), serde_json::to_vec_pretty(value).unwrap()).unwrap();
    }
    fn load(&self, raw: &str) -> SaveV6 {
        fs::write(self.file(), raw).unwrap();
        save_v6::read_save(&self.0).unwrap()
    }
    fn slot(&self) -> PathBuf {
        self.0.join("slots/frozen/slot-v6.json")
    }
    fn put_slot(&self, raw: &[u8]) {
        fs::create_dir_all(self.slot().parent().unwrap()).unwrap();
        fs::write(self.slot(), raw).unwrap();
    }
    fn slot_backup(&self) -> PathBuf {
        self.slot()
            .with_file_name("slot-v6.json.legacy-effect-sources-v1.bak")
    }
}
impl Drop for Root {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn value(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap()
}
fn innate(save: &SaveV6) -> Vec<String> {
    save.effect_sources
        .iter()
        .filter(|s| s.source_kind == EffectSourceKind::InnateCapability)
        .map(|s| s.source_id.clone())
        .collect()
}
fn assert_preserved(old: &Value, current: &SaveV6) {
    let saved: SaveV5 = serde_json::from_value(old["save"].clone()).unwrap();
    let build: PlayerProgressionV6 = serde_json::from_value(old["progression"].clone()).unwrap();
    let persistent: WorldPersistentState = old
        .get("worldPersistentV1")
        .cloned()
        .map(|v| serde_json::from_value(v).unwrap())
        .unwrap_or_default();
    assert_eq!(current.save, saved);
    assert_eq!(current.progression, build);
    assert_eq!(current.world_persistent_v1, persistent);
    assert_eq!(current.effect_sources_version, EFFECT_SOURCES_VERSION);
    current.validate().unwrap();
    current.resolve_rules().unwrap();
}

#[test]
fn frozen_legacy_matrix_normalizes_only_sources_and_marker_in_memory() {
    let rows: Vec<Value> = serde_json::from_str(MATRIX).unwrap();
    for row in rows {
        let root = Root::new();
        root.put(&row["save"]);
        let original = fs::read(root.file()).unwrap();
        let current =
            save_v6::read_or_migrate(&root.0).unwrap_or_else(|e| panic!("{}: {e}", row["name"]));
        assert_preserved(&row["save"], &current);
        assert_eq!(
            innate(&current),
            serde_json::from_value::<Vec<String>>(row["innateIds"].clone()).unwrap(),
            "{}",
            row["name"]
        );
        let old_sources = row["save"]["effectSources"].as_array().unwrap();
        assert_eq!(
            serde_json::to_value(&current.effect_sources[..old_sources.len()]).unwrap(),
            row["save"]["effectSources"]
        );
        assert_eq!(fs::read(root.file()).unwrap(), original);
        assert!(!root.backup().exists());
        assert_eq!(save_v6::read_save(&root.0).unwrap(), current);
    }
}

#[test]
fn actual_old_empty_air_step_and_clockworks_captures_are_accepted() {
    for raw in [
        EMPTY,
        include_str!("fixtures/capability-save-profiles/legacy-authored-air-step.json"),
        include_str!("fixtures/capability-save-profiles/legacy-clockworks.json"),
    ] {
        let root = Root::new();
        let current = root.load(raw);
        assert_preserved(&value(raw), &current);
        assert_eq!(fs::read(root.file()).unwrap(), raw.as_bytes());
    }
}

#[test]
fn current_profile_round_trip_is_strict_and_idempotent_with_retained_old_bytes() {
    let root = Root::new();
    let current = root.load(MIXED);
    save_v6::write_save(&root.0, &current).unwrap();
    assert_eq!(fs::read(root.backup()).unwrap(), MIXED.as_bytes());
    let first = fs::read(root.file()).unwrap();
    assert_eq!(
        value(std::str::from_utf8(&first).unwrap())["effectSourcesVersion"],
        2
    );
    assert_eq!(save_v6::read_save(&root.0).unwrap(), current);
    save_v6::write_save(&root.0, &current).unwrap();
    assert_eq!(fs::read(root.file()).unwrap(), first);
    assert_eq!(fs::read(root.backup()).unwrap(), MIXED.as_bytes());
}

#[test]
fn existing_matching_retained_backup_is_reused_and_conflict_aborts_before_replacement() {
    for matching in [true, false] {
        let root = Root::new();
        let current = root.load(MIXED);
        let backup = if matching {
            MIXED.as_bytes()
        } else {
            b"unrelated retained backup"
        };
        fs::write(root.backup(), backup).unwrap();
        let result = save_v6::write_save(&root.0, &current);
        if matching {
            result.unwrap();
        } else {
            assert_eq!(result.unwrap_err(), "E_SAVE_MIGRATION_BACKUP_MISMATCH");
            assert_eq!(fs::read(root.file()).unwrap(), MIXED.as_bytes());
        }
        assert_eq!(fs::read(root.backup()).unwrap(), backup);
    }
}

#[test]
fn unwriteable_retained_backup_and_invalid_replacement_preserve_original() {
    let root = Root::new();
    let mut current = root.load(MIXED);
    fs::create_dir(root.backup()).unwrap();
    assert!(save_v6::write_save(&root.0, &current).is_err());
    assert_eq!(fs::read(root.file()).unwrap(), MIXED.as_bytes());
    current.effect_sources.clear();
    assert_eq!(
        save_v6::write_save(&root.0, &current).unwrap_err(),
        "E_SAVE_EFFECT_SOURCES_INVALID"
    );
    assert_eq!(fs::read(root.file()).unwrap(), MIXED.as_bytes());
}

fn source_mutations(original: &Value) -> Vec<Value> {
    let mut cases = vec![];
    let mut v = original.clone();
    v["effectSources"].as_array_mut().unwrap().remove(0);
    cases.push(v);
    let mut v = original.clone();
    let copy = v["effectSources"][0].clone();
    v["effectSources"].as_array_mut().unwrap().push(copy);
    cases.push(v);
    let mut v = original.clone();
    v["effectSources"].as_array_mut().unwrap().swap(0, 1);
    cases.push(v);
    let mut v = original.clone();
    v["effectSources"][0]["source_id"] = json!("forged");
    cases.push(v);
    let mut v = original.clone();
    v["effectSources"][0]["instance_id"] = json!("equipment:forged");
    cases.push(v);
    let mut v = original.clone();
    v["effectSources"][0]["lifetime"] = json!("permanent");
    cases.push(v);
    let mut v = original.clone();
    v["effectSources"][0]["effects"] = json!([]);
    cases.push(v);
    let mut v = original.clone();
    v["effectSources"] = json!([]);
    cases.push(v);
    cases
}

#[test]
fn neither_profile_accepts_missing_extra_duplicate_reordered_or_forged_sources() {
    let root = Root::new();
    let current = root.load(MIXED);
    for original in [value(MIXED), serde_json::to_value(current).unwrap()] {
        for bad in source_mutations(&original) {
            root.put(&bad);
            let bytes = fs::read(root.file()).unwrap();
            assert_eq!(
                save_v6::read_save(&root.0).unwrap_err(),
                "E_SAVE_EFFECT_SOURCES_INVALID"
            );
            assert_eq!(fs::read(root.file()).unwrap(), bytes);
        }
    }
}

#[test]
fn marker_absence_is_legacy_only_and_explicit_invalid_markers_never_fallback() {
    let root = Root::new();
    let current = root.load(MIXED);
    for marker in [
        Value::Null,
        json!(0),
        json!(1),
        json!(3),
        json!("2"),
        json!(true),
        json!(2.0),
        json!({}),
    ] {
        let mut bad = value(MIXED);
        bad["effectSourcesVersion"] = marker;
        root.put(&bad);
        assert_eq!(
            save_v6::read_save(&root.0).unwrap_err(),
            "E_SAVE_EFFECT_SOURCES_VERSION_UNSUPPORTED"
        );
    }
    let mut current_as_old = serde_json::to_value(current).unwrap();
    current_as_old
        .as_object_mut()
        .unwrap()
        .remove("effectSourcesVersion");
    root.put(&current_as_old);
    assert_eq!(
        save_v6::read_save(&root.0).unwrap_err(),
        "E_SAVE_EFFECT_SOURCES_INVALID"
    );
}

#[test]
fn raw_schema_validation_rejects_extra_missing_and_explicit_default_shapes() {
    let root = Root::new();
    let current = root.load(MIXED);
    for base in [value(MIXED), serde_json::to_value(current).unwrap()] {
        let mut cases = vec![];
        let mut no_sources = base.clone();
        no_sources.as_object_mut().unwrap().remove("effectSources");
        cases.push(no_sources);
        let mut bad = base.clone();
        bad["extra"] = json!(1);
        cases.push(bad);
        let mut bad = base.clone();
        bad["save"]["capabilities"]["unexpected"] = json!(1);
        cases.push(bad);
        let mut bad = base.clone();
        bad["save"]["capabilities"]
            .as_object_mut()
            .unwrap()
            .remove("lastNowMs");
        cases.push(bad);
        let mut bad = base.clone();
        bad["progression"]
            .as_object_mut()
            .unwrap()
            .remove("bloodline");
        cases.push(bad);
        let mut bad = base.clone();
        bad["effectSources"][0]["extra"] = json!(true);
        cases.push(bad);
        for bad in cases {
            root.put(&bad);
            assert!(save_v6::read_save(&root.0).is_err(), "accepted {bad}");
        }
    }
    let mut explicit_default = value(EMPTY);
    explicit_default["worldPersistentV1"] = json!({"mistHarbor":{"pump":{"state":"ready","drainCompleteAtWorldTimeMs":null},"exploredRegionIds":[]}});
    root.put(&explicit_default);
    assert_eq!(
        save_v6::read_save(&root.0).unwrap_err(),
        "E_SAVE_SCHEMA_INVALID"
    );
}

#[test]
fn saved_mirror_cannot_be_repaired_or_used_as_authority() {
    let root = Root::new();
    let current = root.load(MIXED);
    for base in [value(MIXED), serde_json::to_value(current).unwrap()] {
        let mut absent = base.clone();
        absent["progression"]["capabilities"] = json!([]);
        let mut reordered = base.clone();
        reordered["progression"]["capabilities"]
            .as_array_mut()
            .unwrap()
            .reverse();
        let mut forged = base.clone();
        forged["save"]["capabilities"]["grants"] = json!([]);
        forged["save"]["capabilities"]["selected"] = json!([]);
        for bad in [absent, reordered, forged] {
            root.put(&bad);
            assert_eq!(
                save_v6::read_save(&root.0).unwrap_err(),
                "E_SAVE_CAPABILITY_BUILD_MISMATCH"
            );
        }
    }
}

#[test]
fn existing_grant_selection_and_authored_route_provenance_checks_remain_required() {
    let root = Root::new();
    for current in [false, true] {
        let original = if current {
            serde_json::to_value(root.load(MIXED)).unwrap()
        } else {
            value(MIXED)
        };
        let mut duplicate = original.clone();
        let grant = duplicate["save"]["capabilities"]["grants"][0].clone();
        duplicate["save"]["capabilities"]["grants"]
            .as_array_mut()
            .unwrap()
            .push(grant);
        let mut selected = original.clone();
        selected["save"]["capabilities"]["selected"] =
            json!(["information.local_map_i", "information.local_map_i"]);
        let mut ungranted = original.clone();
        ungranted["save"]["capabilities"]["selected"] = json!(["unknown.capability"]);
        for bad in [duplicate, selected, ungranted] {
            root.put(&bad);
            assert_eq!(
                save_v6::read_save(&root.0).unwrap_err(),
                "E_SAVE_CAPABILITY_INVALID"
            );
        }
        for world in [1, 2] {
            let mut bad = original.clone();
            bad["save"]["progression"]["progress"][world]["completedEvents"] = json!([]);
            root.put(&bad);
            assert_eq!(
                save_v6::read_save(&root.0).unwrap_err(),
                "E_SAVE_CAPABILITY_ROUTE_INVALID"
            );
        }
    }
}

#[test]
fn bare_and_malformed_rear_view_metadata_does_not_create_an_innate_source() {
    let root = Root::new();
    let mut bare = value(EMPTY);
    bare["save"]["rearView"] = value(MIXED)["save"]["rearView"].clone();
    bare["save"]["capabilities"]["rearViewAuthorization"] = bare["save"]["rearView"].clone();
    root.put(&bare);
    assert!(innate(&save_v6::read_save(&root.0).unwrap()).is_empty());
    for malformed in [true, false] {
        let mut bad = value(MIXED);
        if malformed {
            bad["save"]["rearView"]["grantedAtRevision"] = Value::Null;
            bad["save"]["capabilities"]["rearViewAuthorization"]["grantedAtRevision"] = Value::Null;
        } else {
            bad["save"]["rearView"]["grantId"] = json!("spoofed");
        }
        root.put(&bad);
        assert!(!innate(&save_v6::read_save(&root.0).unwrap())
            .contains(&"perception.rear_view_i".into()));
    }
}

#[test]
fn old_slot_read_and_listing_do_not_write_and_explicit_overwrite_retains_exact_wrapper() {
    let root = Root::new();
    root.put_slot(SLOT.as_bytes());
    let (name, save) = save_slots::read_slot_v6(&root.0, "frozen").unwrap();
    assert_eq!(name, "Frozen legacy profile");
    assert_preserved(&value(SLOT)["save"], &save);
    let summaries = save_slots::list_slots(&root.0);
    assert_eq!(summaries.len(), 1);
    assert!(summaries[0].valid);
    assert_eq!(fs::read(root.slot()).unwrap(), SLOT.as_bytes());
    assert!(!root.slot_backup().exists());
    save_slots::overwrite_slot_v6(&root.0, "frozen", "Current", &save).unwrap();
    assert_eq!(fs::read(root.slot_backup()).unwrap(), SLOT.as_bytes());
    let current: Value = serde_json::from_slice(&fs::read(root.slot()).unwrap()).unwrap();
    assert_eq!(current["schemaVersion"], 3);
    assert_eq!(current["save"]["effectSourcesVersion"], 2);
    assert_eq!(
        save_slots::read_slot_v6(&root.0, "frozen").unwrap(),
        ("Current".into(), save.clone())
    );
    save_slots::overwrite_slot_v6(&root.0, "frozen", "Current again", &save).unwrap();
    assert_eq!(fs::read(root.slot_backup()).unwrap(), SLOT.as_bytes());
}

#[test]
fn slot_backup_conflict_or_directory_preserves_wrapper() {
    for directory in [false, true] {
        let root = Root::new();
        root.put_slot(SLOT.as_bytes());
        let (_, save) = save_slots::read_slot_v6(&root.0, "frozen").unwrap();
        if directory {
            fs::create_dir(root.slot_backup()).unwrap();
        } else {
            fs::write(root.slot_backup(), b"conflict").unwrap();
        }
        assert!(save_slots::overwrite_slot_v6(&root.0, "frozen", "Updated", &save).is_err());
        assert_eq!(fs::read(root.slot()).unwrap(), SLOT.as_bytes());
        if !directory {
            assert_eq!(fs::read(root.slot_backup()).unwrap(), b"conflict");
        }
    }
}

#[test]
fn slot_wrapper_and_nested_profile_fail_closed_before_normalization() {
    let root = Root::new();
    let original = value(SLOT);
    let mut cases = vec![];
    for (key, val) in [
        ("schemaVersion", json!(2)),
        ("slotId", json!("other")),
        ("displayName", json!("  ")),
        ("extra", json!(true)),
    ] {
        let mut bad = original.clone();
        bad[key] = val;
        cases.push(bad);
    }
    let mut missing = original.clone();
    missing.as_object_mut().unwrap().remove("updatedAtMs");
    cases.push(missing);
    let mut marker = original.clone();
    marker["save"]["effectSourcesVersion"] = Value::Null;
    cases.push(marker);
    let mut mirror = original.clone();
    mirror["save"]["progression"]["capabilities"] = json!([]);
    cases.push(mirror);
    for tampered in source_mutations(&original["save"]) {
        let mut bad = original.clone();
        bad["save"] = tampered;
        cases.push(bad);
    }
    for bad in cases {
        let bytes = serde_json::to_vec(&bad).unwrap();
        root.put_slot(&bytes);
        assert!(save_slots::read_slot_v6(&root.0, "frozen").is_err());
        assert!(!save_slots::list_slots(&root.0)[0].valid);
        assert_eq!(fs::read(root.slot()).unwrap(), bytes);
        assert!(!root.slot_backup().exists());
    }
}

#[test]
fn v5_file_and_slot_migrations_use_current_capability_sources_and_keep_originals() {
    let root = Root::new();
    fs::write(save_v5::save_path(&root.0), V5).unwrap();
    let save = save_v6::read_or_migrate(&root.0).unwrap();
    assert_eq!(innate(&save).len(), 6);
    assert_eq!(save.save, serde_json::from_str::<SaveV5>(V5).unwrap());
    assert_eq!(
        save.progression.inventory.get("old_inventory_token"),
        Some(&9)
    );
    assert_eq!(
        fs::read(save_v5::save_path(&root.0)).unwrap(),
        V5.as_bytes()
    );
    let dir = root.0.join("slots/v5");
    fs::create_dir_all(&dir).unwrap();
    let bytes = serde_json::to_vec_pretty(&json!({"schemaVersion":2,"slotId":"v5","displayName":"V5","updatedAtMs":7,"save":value(V5)})).unwrap();
    fs::write(dir.join("slot-v5.json"), &bytes).unwrap();
    let (_, slot) = save_slots::read_slot_v6(&root.0, "v5").unwrap();
    assert_eq!(slot, save);
    assert_eq!(fs::read(dir.join("slot-v5.json")).unwrap(), bytes);
}

#[test]
fn frozen_v4_file_and_slot_migrations_retain_original_files() {
    let root = Root::new();
    fs::write(save_v3::save_path(&root.0), V4).unwrap();
    save_v6::read_or_migrate(&root.0)
        .unwrap()
        .validate()
        .unwrap();
    assert_eq!(
        fs::read(save_v3::save_path(&root.0)).unwrap(),
        V4.as_bytes()
    );
    let dir = root.0.join("slots/v4");
    fs::create_dir_all(&dir).unwrap();
    let bytes = serde_json::to_vec_pretty(&json!({"schemaVersion":1,"slotId":"v4","displayName":"V4","updatedAtMs":7,"save":value(V4)})).unwrap();
    fs::write(dir.join("slot-v4.json"), &bytes).unwrap();
    save_slots::read_slot_v6(&root.0, "v4")
        .unwrap()
        .1
        .validate()
        .unwrap();
    assert_eq!(fs::read(dir.join("slot-v4.json")).unwrap(), bytes);
}

#[test]
fn corrupt_current_file_or_slot_never_falls_back_and_failed_continue_is_atomic() {
    let root = Root::new();
    fs::write(save_v5::save_path(&root.0), V5).unwrap();
    fs::write(
        root.file(),
        br#"{"schemaVersion":6,"effectSourcesVersion":2}"#,
    )
    .unwrap();
    let before = fs::read(root.file()).unwrap();
    assert!(save_v6::read_or_migrate(&root.0).is_err());
    let runtime = FormalRuntime::new_with_save_dir(root.0.clone()).unwrap();
    let live = serde_json::to_value(runtime.snapshot().unwrap()).unwrap();
    let build = runtime.build_snapshot().unwrap();
    assert!(runtime.continue_saved().is_err());
    assert_eq!(
        serde_json::to_value(runtime.snapshot().unwrap()).unwrap(),
        live
    );
    assert_eq!(runtime.build_snapshot().unwrap(), build);
    assert_eq!(fs::read(root.file()).unwrap(), before);
    let legacy: SaveV5 = serde_json::from_str(V5).unwrap();
    save_slots::create_slot(&root.0, "frozen", "Legacy V5", &legacy).unwrap();
    root.put_slot(br#"{"schemaVersion":3}"#);
    assert!(save_slots::read_slot_v6(&root.0, "frozen").is_err());
    assert_eq!(fs::read(root.slot()).unwrap(), br#"{"schemaVersion":3}"#);
}

#[test]
fn file_and_slot_size_limits_apply_before_profile_parsing() {
    let root = Root::new();
    let bytes = vec![b' '; 8 * 1024 * 1024 + 1];
    fs::write(root.file(), &bytes).unwrap();
    assert_eq!(save_v6::read_save(&root.0).unwrap_err(), "E_SAVE_TOO_LARGE");
    root.put_slot(&bytes);
    assert_eq!(
        save_slots::read_slot_v6(&root.0, "frozen").unwrap_err(),
        "E_SLOT_TOO_LARGE"
    );
}

#[cfg(unix)]
#[test]
fn retained_backup_symlinks_to_targets_are_rejected_without_replacing_original_bytes() {
    use std::os::unix::fs::symlink;
    let root = Root::new();
    let save = root.load(MIXED);
    symlink(root.file(), root.backup()).unwrap();
    assert_eq!(
        save_v6::write_save(&root.0, &save).unwrap_err(),
        "E_SAVE_MIGRATION_BACKUP_NOT_REGULAR"
    );
    assert_eq!(fs::read(root.file()).unwrap(), MIXED.as_bytes());
    assert!(fs::symlink_metadata(root.backup())
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(fs::read(root.backup()).unwrap(), MIXED.as_bytes());

    root.put_slot(SLOT.as_bytes());
    let (_, save) = save_slots::read_slot_v6(&root.0, "frozen").unwrap();
    symlink(root.slot(), root.slot_backup()).unwrap();
    assert_eq!(
        save_slots::overwrite_slot_v6(&root.0, "frozen", "Updated", &save).unwrap_err(),
        "E_SLOT_MIGRATION_BACKUP_NOT_REGULAR"
    );
    assert_eq!(fs::read(root.slot()).unwrap(), SLOT.as_bytes());
    assert!(fs::symlink_metadata(root.slot_backup())
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(fs::read(root.slot_backup()).unwrap(), SLOT.as_bytes());
}

#[test]
fn frozen_legacy_files_and_slots_reject_even_empty_or_valid_new_environment_fields() {
    let known = json!({"hazards": {"clockworks/cw_boiler_chamber/cw_heat_accumulation_staged": {
        "cycleStartedAtMs":0,"lastAdvancedAtMs":0,"nextDamageAtMs":0,
        "suppressedUntilMs":0,"exposureMilliunits":0}}, "controlCooldownUntilMs":{}});
    for environment in [Value::Null, json!({}), known] {
        let root = Root::new();
        let mut file = value(EMPTY);
        file["worldPersistentV1"]["environment"] = environment.clone();
        root.put(&file);
        let bytes = fs::read(root.file()).unwrap();
        assert_eq!(
            save_v6::read_save(&root.0).unwrap_err(),
            "E_SAVE_SCHEMA_INVALID"
        );
        assert_eq!(fs::read(root.file()).unwrap(), bytes);
        assert!(!root
            .file()
            .with_file_name("formal-save-v6.json.legacy-effect-sources-v1.bak")
            .exists());
        let mut slot = value(SLOT);
        slot["save"]["worldPersistentV1"]["environment"] = environment;
        let slot_bytes = serde_json::to_vec(&slot).unwrap();
        root.put_slot(&slot_bytes);
        assert!(save_slots::read_slot_v6(&root.0, "frozen").is_err());
        assert_eq!(fs::read(root.slot()).unwrap(), slot_bytes);
        assert!(!root.slot_backup().exists());
        assert!(!save_slots::list_slots(&root.0)[0].valid);
        assert_eq!(fs::read(root.slot()).unwrap(), slot_bytes);
    }
}

#[test]
fn frozen_legacy_files_and_slots_reject_new_clockworks_receipt_field_presence() {
    for (field, supplied) in [
        ("pressureValveIds", Value::Null), ("pressureValveIds", json!([])),
        ("pressureValveIds", json!(["cw_pressure_valve_01_staged"])),
        ("coreConsoleConfirmed", Value::Null), ("coreConsoleConfirmed", json!(false)),
        ("coreConsoleConfirmed", json!(true)),
        ("actorRosterVersion", Value::Null), ("actorRosterVersion", json!(0)),
        ("actorRosterVersion", json!(1)),
    ] {
        let root = Root::new();
        let mut file = value(EMPTY);
        file["worldPersistentV1"]["clockworks"][field] = supplied.clone();
        root.put(&file);
        let bytes = fs::read(root.file()).unwrap();
        assert_eq!(save_v6::read_save(&root.0).unwrap_err(), "E_SAVE_SCHEMA_INVALID");
        assert_eq!(fs::read(root.file()).unwrap(), bytes);
        assert!(!root.backup().exists());
        let mut slot = value(SLOT);
        slot["save"]["worldPersistentV1"]["clockworks"][field] = supplied;
        let slot_bytes = serde_json::to_vec(&slot).unwrap();
        root.put_slot(&slot_bytes);
        assert!(save_slots::read_slot_v6(&root.0, "frozen").is_err());
        assert_eq!(fs::read(root.slot()).unwrap(), slot_bytes);
        assert!(!root.slot_backup().exists());
        assert!(!save_slots::list_slots(&root.0)[0].valid);
        assert_eq!(fs::read(root.slot()).unwrap(), slot_bytes);
    }
}


#[test]
fn frozen_legacy_files_and_slots_reject_new_ordinary_actor_controller_presence() {
    for supplied in [Value::Null, json!({}), json!({"active": null}), json!({"terrainVariant": null}), json!({"terrainVariant": "terrain"})] {
        let root = Root::new();
        let old_actor = wuxian_horror_ch1::world_v3::ActorRuntime::spawn(
            "legacy-guard", "enemy.clockworks.forged_guard",
            wuxian_horror_ch1::world_v3::Vec3::new(1.0, 0.0, 1.0).unwrap(),
        ).unwrap();
        let mut actor = serde_json::to_value(old_actor).unwrap();
        actor["ordinary"] = supplied.clone();
        let mut file = value(EMPTY);
        file["save"]["genericActors"] = json!([actor.clone()]);
        root.put(&file);
        let bytes = fs::read(root.file()).unwrap();
        assert_eq!(save_v6::read_save(&root.0).unwrap_err(), "E_SAVE_SCHEMA_INVALID");
        assert_eq!(fs::read(root.file()).unwrap(), bytes);
        assert!(!root.backup().exists());
        let mut slot = value(SLOT);
        slot["save"]["save"]["genericActors"] = json!([actor]);
        let slot_bytes = serde_json::to_vec(&slot).unwrap();
        root.put_slot(&slot_bytes);
        assert!(save_slots::read_slot_v6(&root.0, "frozen").is_err());
        assert_eq!(fs::read(root.slot()).unwrap(), slot_bytes);
        assert!(!root.slot_backup().exists());
    }
}

#[test]
fn frozen_legacy_files_and_slots_reject_gate_b_actor_roster_marker_presence() {
    for supplied in [Value::Null, json!(0), json!(1)] {
        let root = Root::new();
        let mut file = value(EMPTY);
        file["worldPersistentV1"]["greyHive"]["gateBActorRosterVersion"] = supplied.clone();
        root.put(&file); let bytes = fs::read(root.file()).unwrap();
        assert_eq!(save_v6::read_save(&root.0).unwrap_err(), "E_SAVE_SCHEMA_INVALID");
        assert_eq!(fs::read(root.file()).unwrap(), bytes); assert!(!root.backup().exists());
        let mut slot = value(SLOT);
        slot["save"]["worldPersistentV1"]["greyHive"]["gateBActorRosterVersion"] = supplied;
        let slot_bytes = serde_json::to_vec(&slot).unwrap(); root.put_slot(&slot_bytes);
        assert!(save_slots::read_slot_v6(&root.0, "frozen").is_err());
        assert_eq!(fs::read(root.slot()).unwrap(), slot_bytes); assert!(!root.slot_backup().exists());
        assert!(!save_slots::list_slots(&root.0)[0].valid);
    }
}

#[test]
fn frozen_legacy_files_and_slots_reject_new_group_actor_state_presence() {
    for supplied in [Value::Null, json!({}), json!({"memberHp": []}), json!({"memberHp": [0,0,0,0]})] {
        let root = Root::new();
        let old_actor = wuxian_horror_ch1::world_v3::ActorRuntime::spawn(
            "legacy-guard", "enemy.clockworks.forged_guard",
            wuxian_horror_ch1::world_v3::Vec3::new(1.0, 0.0, 1.0).unwrap(),
        ).unwrap();
        let mut actor = serde_json::to_value(old_actor).unwrap();
        actor["group"] = supplied.clone();
        let mut file = value(EMPTY);
        file["save"]["genericActors"] = json!([actor.clone()]);
        root.put(&file);
        let bytes = fs::read(root.file()).unwrap();
        assert_eq!(save_v6::read_save(&root.0).unwrap_err(), "E_SAVE_SCHEMA_INVALID");
        assert_eq!(fs::read(root.file()).unwrap(), bytes);
        assert!(!root.backup().exists());
        let mut slot = value(SLOT);
        slot["save"]["save"]["genericActors"] = json!([actor]);
        let slot_bytes = serde_json::to_vec(&slot).unwrap();
        root.put_slot(&slot_bytes);
        assert!(save_slots::read_slot_v6(&root.0, "frozen").is_err());
        assert_eq!(fs::read(root.slot()).unwrap(), slot_bytes);
        assert!(!root.slot_backup().exists());
    }
}

#[test]
fn frozen_legacy_files_and_slots_reject_swarm_actor_roster_marker_presence() {
    for supplied in [Value::Null, json!(0), json!(1)] {
        let root = Root::new();
        let mut file = value(EMPTY);
        file["worldPersistentV1"]["greyHive"]["swarmActorRosterVersion"] = supplied.clone();
        root.put(&file); let bytes = fs::read(root.file()).unwrap();
        assert_eq!(save_v6::read_save(&root.0).unwrap_err(), "E_SAVE_SCHEMA_INVALID");
        assert_eq!(fs::read(root.file()).unwrap(), bytes); assert!(!root.backup().exists());
        let mut slot = value(SLOT);
        slot["save"]["worldPersistentV1"]["greyHive"]["swarmActorRosterVersion"] = supplied;
        let slot_bytes = serde_json::to_vec(&slot).unwrap(); root.put_slot(&slot_bytes);
        assert!(save_slots::read_slot_v6(&root.0, "frozen").is_err());
        assert_eq!(fs::read(root.slot()).unwrap(), slot_bytes); assert!(!root.slot_backup().exists());
        assert!(!save_slots::list_slots(&root.0)[0].valid);
    }
}

#[test]
fn frozen_legacy_files_and_slots_reject_tidebound_actor_roster_marker_presence() {
    for supplied in [Value::Null, json!(0), json!(1)] {
        let root = Root::new();
        let mut file = value(EMPTY);
        file["worldPersistentV1"]["mistHarbor"]["tideboundActorRosterVersion"] = supplied.clone();
        root.put(&file); let bytes = fs::read(root.file()).unwrap();
        assert_eq!(save_v6::read_save(&root.0).unwrap_err(), "E_SAVE_SCHEMA_INVALID");
        assert_eq!(fs::read(root.file()).unwrap(), bytes); assert!(!root.backup().exists());
        let mut slot = value(SLOT);
        slot["save"]["worldPersistentV1"]["mistHarbor"]["tideboundActorRosterVersion"] = supplied;
        let slot_bytes = serde_json::to_vec(&slot).unwrap(); root.put_slot(&slot_bytes);
        assert!(save_slots::read_slot_v6(&root.0, "frozen").is_err());
        assert_eq!(fs::read(root.slot()).unwrap(), slot_bytes); assert!(!root.slot_backup().exists());
        assert!(!save_slots::list_slots(&root.0)[0].valid);
    }
}

#[test]
fn frozen_legacy_files_and_slots_reject_controller_variant_key_presence() {
    for supplied in [Value::Null, json!(0), json!(1), json!(255)] {
        let root = Root::new();
        let old = wuxian_horror_ch1::world_v3::ActorRuntime::spawn(
            "legacy-wraith", "enemy.mist_harbor.signal_wraith",
            wuxian_horror_ch1::world_v3::Vec3::new(1.0, 0.0, 1.0).unwrap(),
        ).unwrap();
        let mut actor = serde_json::to_value(old).unwrap();
        actor["controllerVariant"] = supplied;
        let mut file = value(EMPTY);
        file["save"]["genericActors"] = json!([actor.clone()]);
        root.put(&file); let bytes = fs::read(root.file()).unwrap();
        assert_eq!(save_v6::read_save(&root.0).unwrap_err(), "E_SAVE_SCHEMA_INVALID");
        assert_eq!(fs::read(root.file()).unwrap(), bytes); assert!(!root.backup().exists());
        let mut slot = value(SLOT);
        slot["save"]["save"]["genericActors"] = json!([actor]);
        let slot_bytes = serde_json::to_vec(&slot).unwrap(); root.put_slot(&slot_bytes);
        assert!(save_slots::read_slot_v6(&root.0, "frozen").is_err());
        assert_eq!(fs::read(root.slot()).unwrap(), slot_bytes); assert!(!root.slot_backup().exists());
        assert!(!save_slots::list_slots(&root.0)[0].valid);
    }
}

#[test]
fn frozen_legacy_files_and_slots_reject_signal_controller_marker_presence() {
    for supplied in [Value::Null, json!(0), json!(1)] {
        let root = Root::new(); let mut file = value(EMPTY);
        file["worldPersistentV1"]["mistHarbor"]["signalWraithControllerVersion"] = supplied.clone();
        root.put(&file); let bytes = fs::read(root.file()).unwrap();
        assert_eq!(save_v6::read_save(&root.0).unwrap_err(), "E_SAVE_SCHEMA_INVALID");
        assert_eq!(fs::read(root.file()).unwrap(), bytes); assert!(!root.backup().exists());
        let mut slot = value(SLOT);
        slot["save"]["worldPersistentV1"]["mistHarbor"]["signalWraithControllerVersion"] = supplied;
        let slot_bytes = serde_json::to_vec(&slot).unwrap(); root.put_slot(&slot_bytes);
        assert!(save_slots::read_slot_v6(&root.0, "frozen").is_err());
        assert_eq!(fs::read(root.slot()).unwrap(), slot_bytes); assert!(!root.slot_backup().exists());
        assert!(!save_slots::list_slots(&root.0)[0].valid);
    }
}

#[test]
fn frozen_legacy_file_and_slot_reject_any_beacon_key_presence_without_rewriting_originals() {
    for supplied in [Value::Null, json!(0), json!({}), json!({"schemaVersion":1,"state":"uncollected","receipts":[]})] {
        let root = Root::new(); let mut file = value(EMPTY);
        file["worldPersistentV1"]["greyHive"]["beacon"] = supplied.clone();
        root.put(&file); let bytes = fs::read(root.file()).unwrap();
        assert_eq!(save_v6::read_save(&root.0).unwrap_err(), "E_SAVE_SCHEMA_INVALID");
        assert_eq!(fs::read(root.file()).unwrap(), bytes); assert!(!root.backup().exists());
        let mut slot = value(SLOT);
        slot["save"]["worldPersistentV1"]["greyHive"]["beacon"] = supplied;
        let slot_bytes = serde_json::to_vec(&slot).unwrap(); root.put_slot(&slot_bytes);
        assert!(save_slots::read_slot_v6(&root.0, "frozen").is_err());
        assert_eq!(fs::read(root.slot()).unwrap(), slot_bytes); assert!(!root.slot_backup().exists());
        assert!(!save_slots::list_slots(&root.0)[0].valid);
    }
}

#[test]
fn frozen_legacy_files_and_slots_reject_charge_controller_keys_and_new_sentinel_states() {
    for fault in ["null","empty","valid","chargeWindup","charge","stagger"] {
        let root=Root::new();let mut actor=json!({"actorId":"old","actorType":"sentinel","positionM":{"xM":1.0,"yM":0.0,"zM":1.0},"hp":100,"state":"chase","stateRemainingMs":0,"attackSerial":0,"active":true});
        match fault {"null"=>actor["chargeController"]=Value::Null,"empty"=>actor["chargeController"]=json!({}),"valid"=>actor["chargeController"]=json!({"schemaVersion":1,"cooldownRemainingMs":0,"chargeSerial":0,"attack":null}),_=>actor["state"]=json!(fault)}
        let mut file=value(EMPTY);file["save"]["actors"]=json!([actor.clone()]);root.put(&file);let bytes=fs::read(root.file()).unwrap();assert_eq!(save_v6::read_save(&root.0).unwrap_err(),"E_SAVE_SCHEMA_INVALID");assert_eq!(fs::read(root.file()).unwrap(),bytes);assert!(!root.backup().exists());
        let mut slot=value(SLOT);slot["save"]["save"]["actors"]=json!([actor]);let bytes=serde_json::to_vec(&slot).unwrap();root.put_slot(&bytes);assert!(save_slots::read_slot_v6(&root.0,"frozen").is_err());assert_eq!(fs::read(root.slot()).unwrap(),bytes);assert!(!root.slot_backup().exists());
    }
}
