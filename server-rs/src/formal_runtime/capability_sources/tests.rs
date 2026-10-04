use super::*;
use crate::{
    capability_v1::{apply_command_at_revision, CapabilityCommand, RegenerationConfig},
    effects::{CapabilityPermission, MapKnowledgeTag},
    formal_runtime::build_v6::PlayerProgressionV6,
    player_rules::{EffectivePlayerRules, KnowledgeLevel},
    world_v3::WorldRevision,
};

fn authority(ids: &[&str]) -> CapabilityState {
    let mut state = CapabilityState::new(
        "grey_hive",
        RegenerationConfig::new(1500, 3000, 2.5, 0.75).unwrap(),
    )
    .unwrap();
    for id in ids {
        apply_command_at_revision(
            &mut state,
            CapabilityCommand::Grant {
                capability_id: (*id).into(),
            },
            WorldRevision::new(1, 1, 1).unwrap(),
        )
        .unwrap();
    }
    state
}

fn build(state: &CapabilityState) -> PlayerProgressionV6 {
    let mut build = PlayerProgressionV6::default();
    build.sync_capabilities(state);
    build
}

#[test]
fn all_six_use_trusted_stable_sources_but_only_two_require_selection() {
    let mut state = authority(&CANONICAL);
    let build = build(&state);
    let (sources, rules) = build
        .resolve_rules_with_capabilities(&state, &state.rear_view_authorization)
        .unwrap();
    assert_eq!(sources.len(), 4);
    assert!(!sources.iter().any(|s| selected_policy(&s.source_id)));
    assert!(rules.perception.rear_view);
    assert_eq!(rules.map, EffectivePlayerRules::default().map);
    assert_eq!(
        rules.capability_permissions,
        BTreeSet::from([
            CapabilityPermission::EnemyVitalsBasic,
            CapabilityPermission::DelayedRegeneration,
            CapabilityPermission::AuthoredAirStep,
        ])
    );
    apply_command_at_revision(
        &mut state,
        CapabilityCommand::Select {
            capability_ids: vec![CAP_LOCAL_MAP.into(), CAP_ACOUSTIC_MAPPING.into()],
        },
        WorldRevision::new(1, 2, 2).unwrap(),
    )
    .unwrap();
    let (sources, rules) = build
        .resolve_rules_with_capabilities(&state, &state.rear_view_authorization)
        .unwrap();
    assert_eq!(sources.len(), 6);
    assert!(rules
        .capability_permissions
        .contains(&CapabilityPermission::AcousticMapping));
    for tag in [
        MapKnowledgeTag::Topology,
        MapKnowledgeTag::Terrain,
        MapKnowledgeTag::Connections,
    ] {
        assert_eq!(rules.map.level_for_tag(tag), KnowledgeLevel::ExploredOnly);
    }
    assert_eq!(
        rules.map.level_for_tag(MapKnowledgeTag::Objectives),
        KnowledgeLevel::Known
    );
    for tag in [
        MapKnowledgeTag::Enemies,
        MapKnowledgeTag::Hazards,
        MapKnowledgeTag::Loot,
        MapKnowledgeTag::Npcs,
        MapKnowledgeTag::Secrets,
    ] {
        assert_eq!(rules.map.level_for_tag(tag), KnowledgeLevel::None);
    }
    for source in &sources {
        assert_eq!(source.source_kind, EffectSourceKind::InnateCapability);
        assert_eq!(
            source.instance_id,
            format!("capability:{}", source.source_id)
        );
        assert_eq!(
            source.lifetime,
            if selected_policy(&source.source_id) {
                EffectLifetime::Selected
            } else {
                EffectLifetime::Permanent
            }
        );
    }
    assert!(sources.windows(2).all(|w| w[0].source_id < w[1].source_id));
    // Selecting a different granted capability removes only selection-dependent sources.
    apply_command_at_revision(
        &mut state,
        CapabilityCommand::Select {
            capability_ids: vec![CAP_AIR_STEP.into()],
        },
        WorldRevision::new(1, 3, 3).unwrap(),
    )
    .unwrap();
    let (after, rules) = build
        .resolve_rules_with_capabilities(&state, &state.rear_view_authorization)
        .unwrap();
    assert_eq!(after.len(), 4);
    assert!(!rules
        .capability_permissions
        .contains(&CapabilityPermission::AcousticMapping));
    assert_eq!(rules.map, EffectivePlayerRules::default().map);
    assert!(rules
        .capability_permissions
        .contains(&CapabilityPermission::DelayedRegeneration));
    assert!(rules
        .capability_permissions
        .contains(&CapabilityPermission::AuthoredAirStep));
}

#[test]
fn each_grant_maps_to_its_own_rule_and_never_changes_acquisition() {
    for id in CANONICAL {
        let mut state = authority(&[id]);
        if selected_policy(id) {
            apply_command_at_revision(
                &mut state,
                CapabilityCommand::Select {
                    capability_ids: vec![id.into()],
                },
                WorldRevision::new(1, 2, 2).unwrap(),
            )
            .unwrap();
        }
        let original = state.clone();
        let (sources, rules) = build(&state)
            .resolve_rules_with_capabilities(&state, &state.rear_view_authorization)
            .unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_id, id);
        let permission = match id {
            CAP_ENEMY_VITALS => Some(CapabilityPermission::EnemyVitalsBasic),
            CAP_REGENERATION => Some(CapabilityPermission::DelayedRegeneration),
            CAP_ACOUSTIC_MAPPING => Some(CapabilityPermission::AcousticMapping),
            CAP_AIR_STEP => Some(CapabilityPermission::AuthoredAirStep),
            _ => None,
        };
        assert_eq!(
            rules.capability_permissions,
            permission.into_iter().collect()
        );
        assert_eq!(rules.perception.rear_view, id == CAP_REAR_VIEW);
        assert_eq!(
            rules.map.topology != KnowledgeLevel::None,
            id == CAP_LOCAL_MAP
        );
        assert_eq!(rules.terrain, EffectivePlayerRules::default().terrain);
        assert_eq!(rules.hazards, EffectivePlayerRules::default().hazards);
        assert_eq!(rules.boundaries, EffectivePlayerRules::default().boundaries);
        assert_eq!(state, original);
    }
}

#[test]
fn capability_mirror_or_source_metadata_cannot_invent_grant_authority() {
    let state = authority(&[]);
    let mut forged = build(&state);
    forged.capabilities.push(CAP_AIR_STEP.into());
    assert_eq!(
        forged
            .resolve_rules_with_capabilities(&state, &state.rear_view_authorization)
            .unwrap_err(),
        "E_BUILD_CAPABILITY_AUTHORITY_MISMATCH"
    );
    let mut state = authority(&[CAP_REGENERATION]);
    state.selected.push(CAP_AIR_STEP.into());
    assert_eq!(
        build(&state)
            .resolve_rules_with_capabilities(&state, &state.rear_view_authorization)
            .unwrap_err(),
        "E_BUILD_CAPABILITY_AUTHORITY_INVALID"
    );
    state.selected = vec![CAP_REGENERATION.into(), CAP_REGENERATION.into()];
    assert!(build(&state)
        .resolve_rules_with_capabilities(&state, &state.rear_view_authorization)
        .is_err());
    state.selected.clear();
    state.grants[0].capability_id = "browser.forged_permission".into();
    assert!(build(&state)
        .resolve_rules_with_capabilities(&state, &state.rear_view_authorization)
        .is_err());
    let mut state = authority(&[CAP_REGENERATION]);
    state.grants.push(state.grants[0].clone());
    assert!(build(&state)
        .resolve_rules_with_capabilities(&state, &state.rear_view_authorization)
        .is_err());
}

#[test]
fn rear_view_requires_valid_authorization_but_equipment_is_independent() {
    let state = authority(&[CAP_REAR_VIEW]);
    let mut build = build(&state);
    let spoof =
        RearViewAuthorization::granted("browser.forged", WorldRevision::new(1, 2, 2).unwrap())
            .unwrap();
    assert!(
        !build
            .resolve_rules_with_capabilities(&state, &spoof)
            .unwrap()
            .1
            .perception
            .rear_view
    );
    build.grant_trusted_item("rear_view_lens").unwrap();
    build.equip("rear_view_lens").unwrap();
    let (sources, rules) = build
        .resolve_rules_with_capabilities(&state, &spoof)
        .unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].source_kind, EffectSourceKind::Equipment);
    assert!(rules.perception.rear_view);
    build.unequip("lens").unwrap();
    assert!(
        !build
            .resolve_rules_with_capabilities(&state, &spoof)
            .unwrap()
            .1
            .perception
            .rear_view
    );
    assert!(
        build
            .resolve_rules_with_capabilities(&state, &state.rear_view_authorization)
            .unwrap()
            .1
            .perception
            .rear_view
    );
}

#[test]
fn trusted_capability_catalog_rejects_unknown_missing_and_malformed_definitions() {
    let original: serde_json::Value = serde_json::from_str(CONTENT).unwrap();
    for mutation in 0..8 {
        let mut value = original.clone();
        match mutation {
            0 => {
                value["capabilities"]
                    .as_object_mut()
                    .unwrap()
                    .remove(CAP_AIR_STEP);
            }
            1 => {
                value["capabilities"]["arbitrary_capability"] =
                    value["capabilities"][CAP_AIR_STEP].clone();
            }
            2 => {
                value["capabilities"][CAP_AIR_STEP]["effects"][0]["grant_capability"]["tag"] =
                    serde_json::json!("free_flight")
            }
            3 => {
                value["capabilities"][CAP_AIR_STEP]["effects"][0]["grant_capability"]["stack"] =
                    serde_json::json!("add")
            }
            4 => value["capabilities"][CAP_AIR_STEP]["lifetime"] = serde_json::json!("permanent"),
            5 => value["capabilities"][CAP_AIR_STEP]["effects"] = serde_json::json!([]),
            6 => value["capabilities"][CAP_AIR_STEP]["uiCategory"] = serde_json::json!("unknown"),
            _ => value["schemaVersion"] = serde_json::json!(2),
        }
        assert!(
            catalog(&value.to_string()).is_err(),
            "accepted mutation {mutation}"
        );
    }
}

#[test]
fn matching_rear_view_records_without_a_grant_revision_do_not_authorize_a_source() {
    let mut state = authority(&[CAP_REAR_VIEW]);
    state.rear_view_authorization.granted_at_revision = None;
    let (sources, rules) = build(&state)
        .resolve_rules_with_capabilities(&state, &state.rear_view_authorization)
        .unwrap();
    assert!(sources.is_empty());
    assert!(!rules.perception.rear_view);
}
