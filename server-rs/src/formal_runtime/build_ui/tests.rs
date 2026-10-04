use super::*;
use crate::{effects::HazardTag, save_v6::SaveV6};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Barrier,
    },
};

static NEXT: AtomicU64 = AtomicU64::new(1);
fn root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "build-ui-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}
fn runtime(path: PathBuf) -> FormalRuntime {
    let runtime = FormalRuntime::new_with_save_dir(path).unwrap();
    runtime.pause().unwrap();
    runtime
}
fn equip(
    runtime: &FormalRuntime,
    id: &str,
    item: &str,
    expected: Option<&str>,
) -> BuildCommandRequest {
    let view = runtime.snapshot().unwrap();
    BuildCommandRequest {
        request_id: id.into(),
        world_epoch: view.world_epoch,
        expected_build_revision: view.build.unwrap().revision,
        action: BuildAction::Equip {
            item_id: item.into(),
            expected_item_id: expected.map(str::to_owned),
        },
    }
}
fn unequip(runtime: &FormalRuntime, id: &str, slot: &str, expected: &str) -> BuildCommandRequest {
    let mut request = equip(runtime, id, expected, None);
    request.action = BuildAction::Unequip {
        slot_id: slot.into(),
        expected_item_id: expected.into(),
    };
    request
}
fn apply(runtime: &FormalRuntime, request: BuildCommandRequest) -> CommandReceipt {
    let result = runtime.apply_build_command(request).unwrap();
    assert_eq!(result.world_epoch, result.snapshot.world_epoch);
    assert_eq!(result.server_tick, result.snapshot.server_tick);
    assert_eq!(
        result.authority_revision,
        result.snapshot.authority_revision
    );
    assert!(result.snapshot.build.is_some());
    result
}

#[test]
fn projection_is_authoritative_owned_only_and_release_admission_does_not_disable_runtime() {
    let runtime = runtime(root());
    let empty = runtime.snapshot().unwrap().build.unwrap();
    assert_eq!(empty.schema_version, 1);
    assert!(empty.items.is_empty() && empty.equipment.is_empty());
    runtime.grant_trusted_build_item("rear_view_lens").unwrap();
    runtime.grant_trusted_build_item("rear_view_lens").unwrap();
    {
        let mut state = runtime.state.lock().unwrap();
        let mut candidate = state.progression_v6.clone();
        candidate
            .inventory
            .insert("legacy_unrecognized_item".into(), 3);
        state.install_build_command(candidate).unwrap();
    }
    let view = runtime.snapshot().unwrap();
    let build = view.build.unwrap();
    assert_eq!(build.items.len(), 1);
    assert_eq!(build.items[0].item_id, "rear_view_lens");
    assert_eq!(build.items[0].quantity, 2);
    assert!(!build.items[0].release_eligible);
    assert!(!build.items[0].equipped);
    assert!(!build.items[0].label.is_empty() && !build.items[0].description.is_empty());
    assert!(
        apply(
            &runtime,
            equip(&runtime, "equip-lens", "rear_view_lens", None)
        )
        .applied
    );
    let projected = runtime.snapshot().unwrap().build.unwrap();
    assert!(projected.items[0].equipped);
    assert_eq!(projected.equipment[0].item_id, "rear_view_lens");
    assert!(runtime.snapshot().unwrap().capabilities.rear_view.granted);
    assert_eq!(
        runtime.build_snapshot().unwrap().0.inventory["legacy_unrecognized_item"],
        3
    );
}

#[test]
fn forged_or_unowned_ids_and_expected_occupant_mismatch_do_not_mutate() {
    let runtime = runtime(root());
    runtime
        .grant_trusted_build_item("heat_resistance_charm")
        .unwrap();
    let before = runtime.build_snapshot().unwrap();
    let snapshot = runtime.snapshot().unwrap();
    for (id, item, expected, code) in [
        (
            "unknown",
            "legacy_unknown",
            None,
            "E_BUILD_CONTENT_NOT_FOUND",
        ),
        ("unowned", "rear_view_lens", None, "E_BUILD_ITEM_NOT_OWNED"),
        (
            "wrong-slot",
            "heat_resistance_charm",
            Some("rear_view_charm"),
            "E_BUILD_SLOT_CHANGED",
        ),
    ] {
        let receipt = apply(&runtime, equip(&runtime, id, item, expected));
        assert!(!receipt.applied && !receipt.already_applied);
        assert_eq!(receipt.error_code.as_deref(), Some(code));
        assert_eq!(runtime.build_snapshot().unwrap(), before);
        assert_eq!(receipt.authority_revision, snapshot.authority_revision);
        assert_eq!(receipt.snapshot.build, snapshot.build);
    }
}

#[test]
fn replay_is_idempotent_and_request_payload_reuse_fails() {
    let runtime = runtime(root());
    runtime.grant_trusted_build_item("rear_view_lens").unwrap();
    let request = equip(&runtime, "request-1", "rear_view_lens", None);
    let first = apply(&runtime, request.clone());
    assert!(first.applied && !first.already_applied);
    let duplicate = apply(&runtime, request.clone());
    assert!(!duplicate.applied && duplicate.already_applied);
    assert_eq!(duplicate.snapshot.build, first.snapshot.build);
    assert_eq!(duplicate.authority_revision, first.authority_revision);
    let mut conflict = request;
    conflict.action = BuildAction::Unequip {
        slot_id: "lens".into(),
        expected_item_id: "rear_view_lens".into(),
    };
    let receipt = apply(&runtime, conflict);
    assert_eq!(
        receipt.error_code.as_deref(),
        Some("E_BUILD_REQUEST_CONFLICT")
    );
    assert_eq!(receipt.snapshot.build, first.snapshot.build);
}

#[test]
fn replacement_removal_compose_with_bloodline_and_apply_measured_heat() {
    let runtime = runtime(root());
    runtime
        .grant_trusted_build_bloodline("internal_thermal_adaptation")
        .unwrap();
    for id in ["heat_resistance_charm", "rear_view_charm", "rear_view_lens"] {
        runtime.grant_trusted_build_item(id).unwrap();
    }
    let damage = || {
        let mut state = runtime.state.lock().unwrap();
        let before = state.world.player_hp;
        super::super::encounter_hazards::apply_hazard_damage(
            &mut state,
            12,
            HazardTag::Heat,
            "heat",
        );
        before - state.world.player_hp
    };
    assert_eq!(damage(), 6);
    assert!(
        apply(
            &runtime,
            equip(&runtime, "heat", "heat_resistance_charm", None)
        )
        .applied
    );
    assert_eq!(damage(), 5);
    assert!(
        apply(
            &runtime,
            equip(
                &runtime,
                "replace",
                "rear_view_charm",
                Some("heat_resistance_charm")
            )
        )
        .applied
    );
    assert_eq!(damage(), 6);
    assert!(apply(&runtime, equip(&runtime, "lens", "rear_view_lens", None)).applied);
    assert!(
        apply(
            &runtime,
            unequip(&runtime, "remove-charm", "charm", "rear_view_charm")
        )
        .applied
    );
    assert!(runtime.snapshot().unwrap().capabilities.rear_view.granted);
    assert!(
        apply(
            &runtime,
            unequip(&runtime, "remove-lens", "lens", "rear_view_lens")
        )
        .applied
    );
    assert!(!runtime.snapshot().unwrap().capabilities.rear_view.granted);
    assert_eq!(runtime.build_snapshot().unwrap().0.inventory.len(), 3);
}

#[test]
fn stale_revision_epoch_owner_failure_and_revision_exhaustion_are_atomic() {
    let runtime = runtime(root());
    runtime.grant_trusted_build_item("rear_view_lens").unwrap();
    let stale = equip(&runtime, "old", "rear_view_lens", None);
    runtime
        .grant_trusted_build_item("heat_resistance_lining")
        .unwrap();
    assert_eq!(
        apply(&runtime, stale).error_code.as_deref(),
        Some("E_BUILD_STALE_REVISION")
    );
    let mut wrong_epoch = equip(&runtime, "epoch", "rear_view_lens", None);
    wrong_epoch.world_epoch += 1;
    assert_eq!(
        apply(&runtime, wrong_epoch).error_code.as_deref(),
        Some("E_BUILD_STALE_EPOCH")
    );
    let owner_request = equip(&runtime, "owner", "rear_view_lens", None);
    runtime.state.lock().unwrap().last_owner_error = Some("fixture-owner-error".into());
    assert_eq!(
        apply(&runtime, owner_request).error_code.as_deref(),
        Some("E_BUILD_OWNER_UNAVAILABLE")
    );
    runtime.state.lock().unwrap().last_owner_error = None;
    for build_overflow in [false, true] {
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.revision.authority_revision = if build_overflow { 100 } else { u64::MAX };
            state.build_commands.revision = if build_overflow { MAX_SAFE_REVISION } else { 3 };
        }
        let before = runtime.build_snapshot().unwrap();
        let receipt = apply(
            &runtime,
            equip(&runtime, "overflow", "rear_view_lens", None),
        );
        assert!(!receipt.applied && !receipt.already_applied);
        assert_eq!(
            receipt.error_code.as_deref(),
            Some(if build_overflow {
                "E_BUILD_REVISION_EXHAUSTED"
            } else {
                "E_WORLD_REVISION: RevisionExhausted"
            })
        );
        assert_eq!(runtime.build_snapshot().unwrap(), before);
    }
}

#[test]
fn two_concurrent_commands_from_one_projection_cannot_both_commit() {
    let runtime = Arc::new(runtime(root()));
    for item in ["rear_view_charm", "heat_resistance_charm"] {
        runtime.grant_trusted_build_item(item).unwrap();
    }
    let requests = [
        equip(&runtime, "concurrent-a", "rear_view_charm", None),
        equip(&runtime, "concurrent-b", "heat_resistance_charm", None),
    ];
    let barrier = Arc::new(Barrier::new(3));
    let handles = requests
        .into_iter()
        .map(|request| {
            let runtime = Arc::clone(&runtime);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                apply(&runtime, request)
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    let results = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|r| r.applied).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|r| r.error_code.as_deref() == Some("E_BUILD_STALE_REVISION"))
            .count(),
        1
    );
    assert_eq!(
        runtime.snapshot().unwrap().build.unwrap().equipment.len(),
        1
    );
}

#[test]
fn strict_wire_shape_limits_and_null_expected_slot_are_distinct() {
    let valid = serde_json::json!({"requestId":"test-1","worldEpoch":1,"expectedBuildRevision":0,"action":{"kind":"equip","itemId":"rear_view_lens","expectedItemId":null}});
    let parsed: BuildCommandRequest = serde_json::from_value(valid.clone()).unwrap();
    assert!(parsed.validate().is_ok());
    for mutate in [0, 1, 2, 3] {
        let mut value = valid.clone();
        match mutate {
            0 => {
                value["action"]
                    .as_object_mut()
                    .unwrap()
                    .remove("expectedItemId");
            }
            1 => {
                value["action"]["effects"] = serde_json::json!([]);
            }
            2 => {
                value["action"]["kind"] = serde_json::json!("grant");
            }
            _ => {
                value["releaseEligible"] = serde_json::json!(true);
            }
        }
        assert!(serde_json::from_value::<BuildCommandRequest>(value).is_err());
    }
    for id in ["", "UPPER", "a/b", &"a".repeat(97)] {
        let mut request = parsed.clone();
        request.request_id = id.into();
        assert!(request.validate().is_err());
    }
    let mut request = parsed;
    request.expected_build_revision = MAX_SAFE_REVISION + 1;
    assert!(request.validate().is_err());
}

#[test]
fn both_paused_and_running_equipment_follow_existing_runtime_policy() {
    let runtime = runtime(root());
    runtime.grant_trusted_build_item("rear_view_lens").unwrap();
    assert!(apply(&runtime, equip(&runtime, "paused", "rear_view_lens", None)).applied);
    runtime.resume().unwrap();
    assert!(
        apply(
            &runtime,
            unequip(&runtime, "running", "lens", "rear_view_lens")
        )
        .applied
    );
    assert!(!runtime.state.lock().unwrap().paused);
}

#[test]
fn receipt_eviction_does_not_allow_an_old_success_to_apply_twice() {
    let runtime = runtime(root());
    runtime.grant_trusted_build_item("rear_view_lens").unwrap();
    let first = equip(&runtime, "first", "rear_view_lens", None);
    assert!(apply(&runtime, first.clone()).applied);
    for n in 0..RECEIPT_LIMIT {
        let request = if n % 2 == 0 {
            unequip(&runtime, &format!("step-{n}"), "lens", "rear_view_lens")
        } else {
            equip(&runtime, &format!("step-{n}"), "rear_view_lens", None)
        };
        assert!(apply(&runtime, request).applied);
    }
    assert_eq!(
        runtime.state.lock().unwrap().build_commands.receipts.len(),
        RECEIPT_LIMIT
    );
    let before = runtime.build_snapshot().unwrap();
    assert_eq!(
        apply(&runtime, first).error_code.as_deref(),
        Some("E_BUILD_STALE_REVISION")
    );
    assert_eq!(runtime.build_snapshot().unwrap(), before);
}

#[test]
fn save_close_continue_reconstructs_projection_without_changing_profile_two() {
    let path = root();
    let runtime = runtime(path.clone());
    runtime.grant_trusted_build_item("rear_view_lens").unwrap();
    let request = equip(&runtime, "saved-equip", "rear_view_lens", None);
    assert!(apply(&runtime, request.clone()).applied);
    runtime.save().unwrap();
    let before = runtime.build_snapshot().unwrap();
    let save = crate::save_v6::read_save(&path).unwrap();
    let json = serde_json::to_value(&save).unwrap();
    assert_eq!(json["effectSourcesVersion"], 2);
    assert!(json.get("buildCommands").is_none());
    assert!(json.get("build").is_none());
    drop(runtime);
    let continued = self::runtime(path.clone());
    continued.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&continued, view)).unwrap();
    continued.pause().unwrap();
    assert_eq!(continued.build_snapshot().unwrap(), before);
    let projection = continued.snapshot().unwrap().build.unwrap();
    assert!(projection.items[0].equipped);
    assert_eq!(projection.equipment[0].item_id, "rear_view_lens");
    assert_eq!(projection.revision, 1);
    assert_eq!(
        apply(&continued, request).error_code.as_deref(),
        Some("E_BUILD_STALE_EPOCH")
    );
    assert!(
        apply(
            &continued,
            unequip(&continued, "after-load", "lens", "rear_view_lens")
        )
        .applied
    );
    continued.save().unwrap();
    let saved: SaveV6 = crate::save_v6::read_save(&path).unwrap();
    saved.validate().unwrap();
    assert_eq!(saved.effect_sources_version, 2);
    drop(continued);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn repeated_continue_and_slot_switch_rebase_epochs_and_reject_stale_build_commands() {
    let path = root();
    let runtime = runtime(path.clone());
    runtime.grant_trusted_build_item("rear_view_lens").unwrap();
    runtime.save().unwrap();
    runtime.save_slot("slot-a", "A", true).unwrap();
    runtime.save_slot("slot-b", "B", true).unwrap();
    runtime.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
    runtime.pause().unwrap();
    let first = equip(&runtime, "first-continue", "rear_view_lens", None);
    assert!(apply(&runtime, first.clone()).applied);
    runtime.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
    runtime.pause().unwrap();
    assert!(runtime.snapshot().unwrap().world_epoch > first.world_epoch);
    assert_eq!(
        apply(&runtime, first).error_code.as_deref(),
        Some("E_BUILD_STALE_EPOCH")
    );
    runtime.continue_slot("slot-a").map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
    runtime.pause().unwrap();
    let slot_request = equip(&runtime, "slot-a-command", "rear_view_lens", None);
    assert!(apply(&runtime, slot_request.clone()).applied);
    runtime.continue_slot("slot-b").map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
    runtime.pause().unwrap();
    assert!(runtime.snapshot().unwrap().world_epoch > slot_request.world_epoch);
    assert_eq!(
        apply(&runtime, slot_request).error_code.as_deref(),
        Some("E_BUILD_STALE_EPOCH")
    );
    let before_new = equip(&runtime, "before-new", "rear_view_lens", None);
    assert!(apply(&runtime, before_new.clone()).applied);
    runtime.reset_new().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
    runtime.pause().unwrap();
    runtime.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
    runtime.pause().unwrap();
    assert!(runtime.snapshot().unwrap().world_epoch > before_new.world_epoch);
    assert_eq!(
        apply(&runtime, before_new).error_code.as_deref(),
        Some("E_BUILD_STALE_EPOCH")
    );
    let before = runtime.build_snapshot().unwrap();
    runtime.state.lock().unwrap().build_commands.revision = MAX_SAFE_REVISION;
    assert_eq!(
        runtime.continue_saved().unwrap_err(),
        "E_BUILD_REVISION_EXHAUSTED"
    );
    assert_eq!(runtime.build_snapshot().unwrap(), before);
    assert_eq!(
        runtime.reset_new().unwrap_err(),
        "E_BUILD_REVISION_EXHAUSTED"
    );
    assert_eq!(runtime.build_snapshot().unwrap(), before);
    drop(runtime);
    let _ = std::fs::remove_dir_all(path);
}
