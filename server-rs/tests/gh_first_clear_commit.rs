#[path = "support/entry_readiness.rs"]
mod entry_readiness;
use entry_readiness::acknowledge_ready;

use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use wuxian_horror_ch1::{
    formal_runtime::{FormalInteractionResponse, FormalRuntime},
    save_v6,
    world_v3::WorldView,
};

const RS: &str = include_str!("fixtures/scene-runtime-v1/return_station.json");
const GH: &str = include_str!("fixtures/scene-runtime-v1/grey_hive.json");
const MH: &str = include_str!("fixtures/scene-runtime-v1/mist_harbor.json");
const CW: &str = include_str!("fixtures/scene-runtime-v1/clockworks.json");

fn root(label: &str) -> PathBuf {
    let token = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "gh-first-clear-{label}-{}-{token}",
        std::process::id()
    ))
}

fn load_gh(runtime: &FormalRuntime) {
    runtime
        .load_scene_registry(
            [RS, GH, MH, CW],
            "gh_test_power",
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .unwrap();
}

fn interact(runtime: &FormalRuntime, id: &str, request_id: &str) -> FormalInteractionResponse {
    let view = runtime.snapshot().unwrap();
    runtime
        .activate_scene_interaction(id, request_id, view.world_epoch)
        .unwrap()
}

fn gh_progress(view: &WorldView) -> &wuxian_horror_ch1::world_v3::WorldProgressProjection {
    view.progression
        .worlds
        .iter()
        .find(|progress| progress.world_id == "grey_hive")
        .expect("Grey Hive progression is projected")
}

fn complete_gh_first_clear(runtime: &FormalRuntime) -> Vec<FormalInteractionResponse> {
    [
        ("gh_power_console_test", "gh-first-power"),
        ("gh_lockdown_test", "gh-first-lockdown"),
        ("gh_extract_test", "gh-first-extraction"),
    ]
    .into_iter()
    .map(|(id, request_id)| {
        let response = interact(runtime, id, request_id);
        assert!(response.applied, "{id}: {:?}", response.error_code);
        response
    })
    .collect()
}

#[test]
fn extraction_before_power_and_lockdown_is_rejected_without_partial_state() {
    let path = root("premature-extraction");
    let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    load_gh(&runtime);
    let before = runtime.snapshot().unwrap();

    let rejected = interact(&runtime, "gh_extract_test", "gh-extract-too-early");
    assert!(!rejected.applied);
    assert_eq!(
        rejected.error_code.as_deref(),
        Some("E_SCENE_PROGRESS_PREREQUISITES_MISSING")
    );
    let after_rejection = runtime.snapshot().unwrap();
    assert_eq!(after_rejection.progression, before.progression);
    assert_eq!(
        after_rejection.capabilities.items,
        before.capabilities.items
    );
    assert_eq!(after_rejection.scene_id, before.scene_id);
    assert_eq!(after_rejection.world_epoch, before.world_epoch);
    assert_eq!(after_rejection.interactables, before.interactables);

    // A failed extraction did not consume the scene interaction/request id.
    assert!(interact(&runtime, "gh_power_console_test", "gh-first-power").applied);
    assert!(interact(&runtime, "gh_lockdown_test", "gh-first-lockdown").applied);
    assert!(interact(&runtime, "gh_extract_test", "gh-extract-too-early").applied);
    assert!(gh_progress(&runtime.snapshot().unwrap()).completed);

    drop(runtime);
    let _ = fs::remove_dir_all(path);
}

#[test]
fn production_event_sequence_commits_first_clear_once_and_duplicate_request_is_inert() {
    let path = root("first-clear-once");
    let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    load_gh(&runtime);
    let before = runtime.snapshot().unwrap();
    let capabilities_before: Vec<_> = before
        .capabilities
        .items
        .iter()
        .map(|item| (item.capability_id.clone(), item.granted))
        .collect();

    let power = interact(&runtime, "gh_power_console_test", "gh-first-power");
    assert!(power.applied);
    assert!(!gh_progress(&power.view).completed);
    let lockdown = interact(&runtime, "gh_lockdown_test", "gh-first-lockdown");
    assert!(lockdown.applied);
    assert!(!gh_progress(&lockdown.view).completed);
    let extraction = interact(&runtime, "gh_extract_test", "gh-first-extraction");
    assert!(extraction.applied);
    assert!(gh_progress(&extraction.view).completed);
    assert!(gh_progress(&extraction.view).first_completion);
    assert_eq!(
        extraction.view.progression.event_seq,
        before.progression.event_seq + 4,
        "three required event commits plus the atomic Complete route command"
    );
    assert_eq!(
        extraction
            .view
            .capabilities
            .items
            .iter()
            .map(|item| (item.capability_id.clone(), item.granted))
            .collect::<Vec<_>>(),
        capabilities_before,
        "first clear in this package does not choose or grant a capability"
    );

    let scene_events_before = runtime
        .presentation_events_since(extraction.view.world_epoch, 0)
        .unwrap();
    let duplicate = interact(&runtime, "gh_extract_test", "gh-first-extraction");
    assert!(!duplicate.applied);
    assert!(duplicate
        .error_code
        .as_deref()
        .is_some_and(|error| error.contains("DuplicateRequest")));
    assert_eq!(duplicate.view.progression, extraction.view.progression);
    assert_eq!(
        duplicate.view.capabilities.items,
        extraction.view.capabilities.items
    );
    assert_eq!(
        runtime
            .presentation_events_since(extraction.view.world_epoch, 0)
            .unwrap(),
        scene_events_before,
        "replaying the extraction request does not emit another scene event"
    );

    drop(runtime);
    let _ = fs::remove_dir_all(path);
}

#[test]
fn save_v5_continue_preserves_grey_hive_first_clear_and_required_events() {
    let path = root("save-continue-first-clear");
    {
        let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        load_gh(&runtime);
        complete_gh_first_clear(&runtime);
        runtime.save().unwrap();

        let saved = save_v6::read_save(&path).unwrap().save;
        let progress = saved
            .progression
            .progress
            .iter()
            .find(|progress| progress.world_id == "grey_hive")
            .unwrap();
        assert!(progress.completed);
        assert!(progress.first_completion);
        for event in ["hive_power", "hive_lockdown", "hive_extraction"] {
            assert!(progress
                .completed_events
                .iter()
                .any(|actual| actual.as_str() == event));
        }
    }

    {
        let continued = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        load_gh(&continued);
        let view = acknowledge_ready(&continued, continued.continue_saved().unwrap());
        let progress = gh_progress(&view);
        assert!(progress.completed);
        assert!(progress.first_completion);
        let saved = save_v6::read_save(&path).unwrap().save;
        let saved_progress = saved
            .progression
            .progress
            .iter()
            .find(|progress| progress.world_id == "grey_hive")
            .unwrap();
        assert_eq!(progress.completed, saved_progress.completed);
        assert_eq!(progress.first_completion, saved_progress.first_completion);
        assert_eq!(view.progression.event_seq, saved.progression.event_seq);
    }

    let _ = fs::remove_dir_all(path);
}
