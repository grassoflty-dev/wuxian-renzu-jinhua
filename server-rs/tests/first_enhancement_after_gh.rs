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
    capability_v1::{CAP_LOCAL_MAP, CAP_REAR_VIEW, CAP_REGENERATION},
    effects::{CapabilityPermission, EffectSource, MapKnowledgeProjection},
    formal_runtime::{CapabilityCommandRequest, FormalRuntime},
    player_rules::{EffectivePlayerRules, KnowledgeChannel, KnowledgeLevel},
    save_v6,
    world_v3::{RearViewAuthorization, WorldView},
};

const RS: &str = include_str!("fixtures/scene-runtime-v1/return_station.json");
const GH: &str = include_str!("fixtures/scene-runtime-v1/grey_hive.json");
const MH: &str = include_str!("fixtures/scene-runtime-v1/mist_harbor.json");
const CW: &str = include_str!("fixtures/scene-runtime-v1/clockworks.json");
const CHOICES: [&str; 3] = [CAP_LOCAL_MAP, CAP_REAR_VIEW, CAP_REGENERATION];

fn root(label: &str) -> PathBuf {
    let token = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "first-enhancement-{label}-{}-{token}",
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

fn interact(runtime: &FormalRuntime, id: &str, request_id: &str) {
    let view = runtime.snapshot().unwrap();
    let response = runtime
        .activate_scene_interaction(id, request_id, view.world_epoch)
        .unwrap();
    assert!(response.applied, "{id}: {:?}", response.error_code);
}

fn complete_grey_hive(runtime: &FormalRuntime) {
    interact(runtime, "gh_power_console_test", "first-enhancement-power");
    interact(runtime, "gh_lockdown_test", "first-enhancement-lockdown");
    interact(runtime, "gh_extract_test", "first-enhancement-extraction");
    let view = runtime.snapshot().unwrap();
    let grey_hive = view
        .progression
        .worlds
        .iter()
        .find(|progress| progress.world_id == "grey_hive")
        .unwrap();
    assert!(grey_hive.completed);
    assert!(grey_hive.first_completion);
}

fn granted_ids(runtime: &FormalRuntime) -> Vec<String> {
    runtime
        .snapshot()
        .unwrap()
        .capabilities
        .items
        .into_iter()
        .filter(|item| item.granted)
        .map(|item| item.capability_id)
        .collect()
}

fn expected_choice_source(capability_id: &str) -> EffectSource {
    let (lifetime, category, effects) = match capability_id {
        CAP_LOCAL_MAP => (
            "selected",
            "information",
            serde_json::json!([
                {"map_knowledge_level": {"tag": "topology", "level": "explored_only", "stack": "max"}},
                {"map_knowledge_level": {"tag": "terrain", "level": "explored_only", "stack": "max"}},
                {"map_knowledge_level": {"tag": "connections", "level": "explored_only", "stack": "max"}},
                {"map_knowledge_level": {"tag": "objectives", "level": "known", "stack": "max"}}
            ]),
        ),
        CAP_REAR_VIEW => (
            "permanent",
            "perception",
            serde_json::json!([
                {"perception": {"tag": "rear_view", "stack": "any"}}
            ]),
        ),
        CAP_REGENERATION => (
            "permanent",
            "body",
            serde_json::json!([
                {"grant_capability": {"tag": "delayed_regeneration", "stack": "any"}}
            ]),
        ),
        _ => panic!("unsupported public first-enhancement choice"),
    };
    serde_json::from_value(serde_json::json!({
        "source_kind": "innate_capability", "source_id": capability_id,
        "instance_id": format!("capability:{capability_id}"),
        "lifetime": lifetime, "ui_category": category, "effects": effects
    }))
    .unwrap()
}

fn expected_choice_rules(capability_id: &str) -> EffectivePlayerRules {
    let mut rules = EffectivePlayerRules::default();
    match capability_id {
        CAP_LOCAL_MAP => {
            rules.map.topology = KnowledgeLevel::ExploredOnly;
            rules.map.terrain = KnowledgeLevel::ExploredOnly;
            rules.map.connections = KnowledgeLevel::ExploredOnly;
            rules.map.objectives = KnowledgeLevel::Known;
        }
        CAP_REAR_VIEW => rules.perception.rear_view = true,
        CAP_REGENERATION => {
            rules
                .capability_permissions
                .insert(CapabilityPermission::DelayedRegeneration);
        }
        _ => panic!("unsupported public first-enhancement choice"),
    }
    rules
}

fn assert_choice_projection(runtime: &FormalRuntime, view: &WorldView, capability_id: &str) {
    let (build, sources, rules) = runtime.build_snapshot().unwrap();
    assert_eq!(build.capabilities, [capability_id.to_owned()]);
    assert_eq!(sources, vec![expected_choice_source(capability_id)]);
    assert_eq!(rules, expected_choice_rules(capability_id));
    assert_eq!(view.capabilities.items.len(), 6);
    assert_eq!(
        view.capabilities
            .items
            .iter()
            .filter(|item| item.granted)
            .map(|item| item.capability_id.as_str())
            .collect::<Vec<_>>(),
        [capability_id]
    );
    assert_eq!(
        view.capabilities
            .items
            .iter()
            .filter(|item| item.selected)
            .map(|item| item.capability_id.as_str())
            .collect::<Vec<_>>(),
        [capability_id]
    );
    assert_eq!(
        view.capabilities.map_topology_authorized,
        Some(capability_id == CAP_LOCAL_MAP)
    );
    let map = view
        .capabilities
        .map_knowledge
        .as_ref()
        .expect("resolved map authorization is projected");
    if capability_id == CAP_LOCAL_MAP {
        assert!(map.regions.iter().all(|region| matches!(
            region.channel,
            KnowledgeChannel::Topology | KnowledgeChannel::Terrain
        )));
        assert!(map.features.iter().all(|feature| matches!(
            feature.channel,
            KnowledgeChannel::Connections | KnowledgeChannel::Objectives
        )));
    } else {
        assert_eq!(map, &MapKnowledgeProjection::default());
        assert!(view.capabilities.explored_map.rooms.is_empty());
        assert!(view.capabilities.explored_map.connections.is_empty());
        assert!(view.capabilities.explored_map.objectives.is_empty());
    }
    if capability_id == CAP_REAR_VIEW {
        assert!(view.capabilities.rear_view.granted);
        assert_eq!(
            view.capabilities.rear_view.grant_id.as_deref(),
            Some(CAP_REAR_VIEW)
        );
        assert!(view.capabilities.rear_view.granted_at_revision.is_some());
    } else {
        assert_eq!(view.capabilities.rear_view, RearViewAuthorization::denied());
    }
    assert!(view.capabilities.enemy_vitals.is_empty());
}

#[test]
fn all_first_enhancement_choices_are_rejected_before_grey_hive_first_clear() {
    let path = root("before-clear");
    let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    load_gh(&runtime);
    let before = runtime.snapshot().unwrap();
    let before_build = runtime.build_snapshot().unwrap();

    for capability_id in CHOICES {
        assert_eq!(
            runtime.choose_first_enhancement(capability_id).unwrap_err(),
            "E_CAPABILITY_REQUIRES_GH_FIRST_CLEAR"
        );
        let after = runtime.snapshot().unwrap();
        assert_eq!(after.capabilities.items, before.capabilities.items);
        assert_eq!(after.capabilities.rear_view, before.capabilities.rear_view);
        assert_eq!(after.progression, before.progression);
        assert_eq!(runtime.build_snapshot().unwrap(), before_build);
    }

    drop(runtime);
    let _ = fs::remove_dir_all(path);
}

#[test]
fn grey_hive_first_clear_allows_one_explicit_choice_of_each_supported_option() {
    for capability_id in CHOICES {
        let path = root("one-choice");
        let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        load_gh(&runtime);
        complete_grey_hive(&runtime);
        assert!(
            granted_ids(&runtime).is_empty(),
            "first clear does not auto-grant an enhancement"
        );

        assert_eq!(
            runtime.choose_first_enhancement(capability_id).unwrap_err(),
            "E_ENHANCEMENT_REQUIRES_FORMAL_PAUSE"
        );
        runtime.pause().unwrap();
        let selected = runtime.choose_first_enhancement(capability_id).unwrap();
        assert!(selected
            .capabilities
            .items
            .iter()
            .any(|item| { item.capability_id == capability_id && item.granted && item.selected }));
        assert_eq!(granted_ids(&runtime), [capability_id.to_owned()]);
        assert_choice_projection(&runtime, &selected, capability_id);
        assert_choice_projection(&runtime, &runtime.snapshot().unwrap(), capability_id);
        let before_repeat_build = runtime.build_snapshot().unwrap();

        let before_repeat = runtime.snapshot().unwrap();
        let another_choice = CHOICES
            .into_iter()
            .find(|choice| *choice != capability_id)
            .unwrap();
        assert_eq!(
            runtime
                .choose_first_enhancement(another_choice)
                .unwrap_err(),
            "E_CAPABILITY_REJECTED: FirstEnhancementAlreadyChosen"
        );
        let after_repeat = runtime.snapshot().unwrap();
        assert_eq!(
            after_repeat.capabilities.items,
            before_repeat.capabilities.items
        );
        assert_eq!(
            after_repeat.capabilities.rear_view,
            before_repeat.capabilities.rear_view
        );
        assert_eq!(after_repeat.progression, before_repeat.progression);
        assert_eq!(runtime.build_snapshot().unwrap(), before_repeat_build);
        assert_choice_projection(&runtime, &after_repeat, capability_id);

        drop(runtime);
        let _ = fs::remove_dir_all(path);
    }
}

#[test]
fn save_v6_new_runtime_continue_preserves_each_first_clear_choice_source_rules_and_projection() {
    for capability_id in CHOICES {
        let path = root("save-continue");
        let expected_build;
        let expected_projection;
        {
            let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
            load_gh(&runtime);
            complete_grey_hive(&runtime);
            runtime.pause().unwrap();
            let chosen = runtime.choose_first_enhancement(capability_id).unwrap();
            assert_choice_projection(&runtime, &chosen, capability_id);
            expected_build = runtime.build_snapshot().unwrap();
            expected_projection = chosen.capabilities.clone();
            runtime.save().unwrap();

            let durable = save_v6::read_save(&path).unwrap();
            assert_eq!(durable.effect_sources_version, 2);
            assert_eq!(
                durable.effect_sources,
                vec![expected_choice_source(capability_id)]
            );
            assert_eq!(
                durable.resolve_rules().unwrap(),
                expected_choice_rules(capability_id)
            );
            let raw: serde_json::Value =
                serde_json::from_slice(&fs::read(save_v6::save_path(&path)).unwrap()).unwrap();
            assert_eq!(raw["effectSourcesVersion"], 2);
            assert_eq!(
                raw["effectSources"],
                serde_json::to_value(&durable.effect_sources).unwrap()
            );
            let saved = durable.save;
            assert_eq!(
                saved.capabilities.first_enhancement_choice.as_deref(),
                Some(capability_id)
            );
            assert_eq!(
                saved
                    .capabilities
                    .grants
                    .iter()
                    .filter(|grant| CHOICES.contains(&grant.capability_id.as_str()))
                    .count(),
                1
            );
            assert!(saved
                .progression
                .progress
                .iter()
                .find(|progress| progress.world_id == "grey_hive")
                .is_some_and(|progress| progress.completed && progress.first_completion));
            // Projection may expose only geometry present in saved exploration history.
            for room in &expected_projection.explored_map.rooms {
                assert!(saved.explored.rooms.iter().any(|known| known == room));
            }
            if capability_id == CAP_REAR_VIEW {
                assert_eq!(expected_projection.rear_view, saved.rear_view);
                assert_eq!(
                    expected_projection.rear_view,
                    saved.capabilities.rear_view_authorization
                );
            }
        }

        {
            let restarted = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
            load_gh(&restarted);
            let continued = acknowledge_ready(&restarted, restarted.continue_saved().unwrap());
            let grey_hive = continued
                .progression
                .worlds
                .iter()
                .find(|progress| progress.world_id == "grey_hive")
                .unwrap();
            assert!(grey_hive.completed && grey_hive.first_completion);
            assert!(continued.capabilities.items.iter().any(|item| {
                item.capability_id == capability_id && item.granted && item.selected
            }));
            assert_choice_projection(&restarted, &continued, capability_id);
            assert_eq!(restarted.build_snapshot().unwrap(), expected_build);
            assert_eq!(continued.capabilities, expected_projection);

            let before_repeat = continued.capabilities.items.clone();
            let another_choice = CHOICES.into_iter().find(|id| *id != capability_id).unwrap();
            assert_eq!(
                restarted
                    .choose_first_enhancement(another_choice)
                    .unwrap_err(),
                "E_ENHANCEMENT_REQUIRES_FORMAL_PAUSE"
            );
            restarted.pause().unwrap();
            assert_eq!(
                restarted
                    .choose_first_enhancement(another_choice)
                    .unwrap_err(),
                "E_CAPABILITY_REJECTED: FirstEnhancementAlreadyChosen"
            );
            assert_eq!(
                restarted.snapshot().unwrap().capabilities.items,
                before_repeat
            );
            assert_eq!(restarted.build_snapshot().unwrap(), expected_build);
        }
        let _ = fs::remove_dir_all(path);
    }
}

#[test]
fn first_clear_choice_requires_pause_without_reopening_other_paused_commands() {
    let path = root("pause-gate");
    let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    load_gh(&runtime);
    complete_grey_hive(&runtime);

    assert_eq!(
        runtime.choose_first_enhancement(CAP_LOCAL_MAP).unwrap_err(),
        "E_ENHANCEMENT_REQUIRES_FORMAL_PAUSE"
    );
    runtime.pause().unwrap();
    runtime.choose_first_enhancement(CAP_LOCAL_MAP).unwrap();

    let view = runtime.snapshot().unwrap();
    assert!(view
        .capabilities
        .items
        .iter()
        .any(|item| item.capability_id == CAP_LOCAL_MAP && item.granted && item.selected));
    assert_eq!(
        runtime
            .apply_capability_command(CapabilityCommandRequest::Select {
                capability_ids: vec![CAP_REAR_VIEW.into()],
            })
            .unwrap_err(),
        "E_RUNTIME_PAUSED"
    );

    drop(runtime);
    let _ = fs::remove_dir_all(path);
}
