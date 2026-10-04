use super::*;
use crate::{
    capability_v1::{apply_command_at_revision, CapabilityCommand, CAP_LOCAL_MAP, CAP_REAR_VIEW},
    continuous_input::InputSample,
    formal_runtime::{advance_owner_step_with_scene, advance_regulator_environment, FormalRuntime},
    player_rules::EffectivePlayerRules,
    world_v3::apply_effects_atomically,
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn stopped_runtime(root: PathBuf, scene_id: Option<&str>) -> FormalRuntime {
    let runtime = FormalRuntime::new_with_save_dir(root).unwrap();
    runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime
        .owner_handle
        .lock()
        .unwrap()
        .take()
        .unwrap()
        .join()
        .unwrap();
    if let Some(scene_id) = scene_id {
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        runtime.install_scene_registry(registry, scene_id).unwrap();
        let mut state = runtime.state.lock().unwrap();
        // Isolated production-scene consumer fixture, not campaign-completion evidence.
        state.route.current_world_id = state.world.world_id.clone();
    }
    runtime
}

fn root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "live-build-rules-{}-{}",
        std::process::id(),
        NEXT_DIR.fetch_add(1, Ordering::Relaxed)
    ))
}

fn equip(runtime: &FormalRuntime, item: &str) {
    runtime.grant_trusted_build_item(item).unwrap();
    runtime.equip_build_item(item).unwrap();
}

fn grant_capability(runtime: &FormalRuntime, capability_id: &str) {
    let mut state = runtime.state.lock().unwrap();
    let revision = state.world.revision;
    let effects = apply_command_at_revision(
        &mut state.capabilities,
        CapabilityCommand::Grant {
            capability_id: capability_id.into(),
        },
        revision,
    )
    .unwrap();
    apply_effects_atomically(&mut state.world, &effects).unwrap();
    let build = state.progression_v6.clone();
    state.install_build(build).unwrap();
}

fn measured_walk(runtime: &FormalRuntime, position: Vec3, expected_speed: f32) {
    // Same lock order as the simulation owner and public snapshot/save paths.
    let mut state = runtime.state.lock().unwrap();
    let scene = runtime.scene_runtime.lock().unwrap();
    state.world.player.position_m = position;
    let epoch = state.world.revision.world_epoch;
    let seq = state.world.last_input_seq + 1;
    let client_time = state.world.last_client_time_ms + 17;
    state.latest_sample = InputSample::new(epoch, seq, client_time, 1.0, 0.0).unwrap();
    state.latest_input_received_at = Instant::now();
    advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
    assert!(
        (state.world.player.velocity_mps.x_m - expected_speed).abs() < 0.000_001,
        "actual velocity {} != {expected_speed}",
        state.world.player.velocity_mps.x_m
    );
    let expected_x = position.x_m + expected_speed / 60.0;
    assert!(
        (state.world.player.position_m.x_m - expected_x).abs() < 0.000_002,
        "actual displacement must consume the current KCC rules"
    );
}

#[test]
fn equipment_refreshes_live_kcc_immediately_and_removes_sources_independently() {
    let runtime = stopped_runtime(root(), Some("mh_drowned_quay"));
    let start = Vec3::new(14.0, 0.0, 7.5).unwrap();
    let original = runtime.state.lock().unwrap().world_persistent_v1.clone();
    measured_walk(&runtime, start, 2.4);
    runtime
        .grant_trusted_build_item("water_movement_module")
        .unwrap();
    measured_walk(&runtime, start, 2.4);
    runtime.equip_build_item("water_movement_module").unwrap();
    measured_walk(&runtime, start, 3.2);
    equip(&runtime, "water_movement_charm");
    measured_walk(&runtime, start, 3.6);
    equip(&runtime, "terrain_penalty_ignore_boots");
    measured_walk(&runtime, start, 4.0);
    runtime.unequip_build_slot("boots").unwrap();
    measured_walk(&runtime, start, 3.6);
    runtime.unequip_build_slot("charm").unwrap();
    measured_walk(&runtime, start, 3.2);
    runtime.unequip_build_slot("water_module").unwrap();
    measured_walk(&runtime, start, 2.4);
    assert_eq!(
        runtime
            .state
            .lock()
            .unwrap()
            .world_persistent_v1
            .mist_harbor
            .pump,
        original.mist_harbor.pump
    );
}

#[test]
fn terrain_equipment_never_opens_world_walls_quest_gates_or_deep_water() {
    let runtime = stopped_runtime(root(), Some("mh_pump_station"));
    let before = runtime.state.lock().unwrap().kcc.clone();
    equip(&runtime, "terrain_penalty_ignore_boots");
    equip(&runtime, "water_movement_module");
    let state = runtime.state.lock().unwrap();
    assert_eq!(state.kcc.bounds, before.bounds);
    assert_eq!(state.kcc.walls, before.walls);
    for position in [
        Vec3::new(-1.0, 0.0, 8.0).unwrap(),
        Vec3::new(0.25, 0.0, 8.0).unwrap(),
        Vec3::new(20.0, 0.0, 7.5).unwrap(),
    ] {
        assert!(!before.can_occupy(position, 0.35));
        assert!(!state.kcc.can_occupy(position, 0.35));
    }
    drop(state);
    let gate = stopped_runtime(root(), None);
    let blocked = Vec3::new(0.0, 0.0, 8.0).unwrap();
    assert!(!gate.state.lock().unwrap().kcc.can_occupy(blocked, 0.35));
    equip(&gate, "terrain_penalty_ignore_boots");
    assert!(!gate.state.lock().unwrap().kcc.can_occupy(blocked, 0.35));
}

#[test]
fn equipment_map_is_channel_filtered_and_never_fakes_exploration_or_capability_grants() {
    let runtime = stopped_runtime(root(), Some("mh_drowned_quay"));
    let original = {
        let state = runtime.state.lock().unwrap();
        (
            state.world.explored.clone(),
            state.world_persistent_v1.clone(),
            state.capabilities.clone(),
        )
    };
    assert!(runtime
        .snapshot()
        .unwrap()
        .capabilities
        .explored_map
        .rooms
        .is_empty());
    runtime
        .grant_trusted_build_item("map_fog_bypass_array")
        .unwrap();
    assert_eq!(
        runtime
            .snapshot()
            .unwrap()
            .capabilities
            .map_topology_authorized,
        Some(false)
    );
    runtime.equip_build_item("map_fog_bypass_array").unwrap();
    equip(&runtime, "map_topology_chip");
    let full = runtime.snapshot().unwrap().capabilities;
    assert_eq!(full.map_topology_authorized, Some(true));
    assert_eq!(full.explored_map.rooms.len(), 3);
    assert!(full.explored_map.objectives.is_empty());
    assert!(full.items.iter().all(|item| !item.granted));
    let knowledge = full.map_knowledge.unwrap();
    assert!(knowledge
        .regions
        .iter()
        .any(|region| region.channel == KnowledgeChannel::Terrain));
    assert!(knowledge
        .features
        .iter()
        .any(|item| item.channel == KnowledgeChannel::Connections));
    assert!(knowledge.regions.iter().all(|region| matches!(
        region.channel,
        KnowledgeChannel::Topology | KnowledgeChannel::Terrain
    )));
    assert!(knowledge
        .features
        .iter()
        .all(|item| item.channel == KnowledgeChannel::Connections));
    assert!(!full.rear_view.granted);
    runtime.unequip_build_slot("map_array").unwrap();
    let topology = runtime.snapshot().unwrap().capabilities;
    assert_eq!(topology.explored_map.rooms.len(), 3);
    assert!(topology
        .map_knowledge
        .unwrap()
        .regions
        .iter()
        .all(|region| region.channel == KnowledgeChannel::Topology));
    assert!(runtime
        .snapshot()
        .unwrap()
        .capabilities
        .map_knowledge
        .unwrap()
        .features
        .is_empty());
    runtime.unequip_build_slot("map_chip").unwrap();
    let empty = runtime.snapshot().unwrap().capabilities;
    assert_eq!(empty.map_topology_authorized, Some(false));
    assert!(empty.explored_map.rooms.is_empty());
    let state = runtime.state.lock().unwrap();
    assert_eq!(
        (
            state.world.explored.clone(),
            state.world_persistent_v1.clone(),
            state.capabilities.clone()
        ),
        original
    );
}

#[test]
fn local_map_selection_composes_with_equipment_and_survives_last_source_removal() {
    let runtime = stopped_runtime(root(), Some("mh_drowned_quay"));
    grant_capability(&runtime, CAP_LOCAL_MAP);
    assert!(runtime
        .snapshot()
        .unwrap()
        .capabilities
        .explored_map
        .rooms
        .is_empty());
    equip(&runtime, "map_fog_bypass_array");
    assert_eq!(
        runtime
            .snapshot()
            .unwrap()
            .capabilities
            .explored_map
            .rooms
            .len(),
        3
    );
    runtime
        .apply_capability_command(crate::formal_runtime::CapabilityCommandRequest::Select {
            capability_ids: vec![CAP_LOCAL_MAP.into()],
        })
        .unwrap();
    runtime.unequip_build_slot("map_array").unwrap();
    let selected = runtime.snapshot().unwrap().capabilities;
    assert_eq!(selected.map_topology_authorized, Some(true));
    assert_eq!(selected.explored_map.rooms.len(), 1);
    assert!(selected
        .explored_map
        .rooms
        .iter()
        .all(|room| room.room_id == "mh_dq_west_quay"));
    assert!(selected
        .map_knowledge
        .unwrap()
        .regions
        .iter()
        .all(|region| region.channel != KnowledgeChannel::Terrain));
}

#[test]
fn rear_view_projection_tracks_two_equipment_sources_and_independent_permanent_grant() {
    let runtime = stopped_runtime(root(), None);
    equip(&runtime, "rear_view_charm");
    equip(&runtime, "rear_view_lens");
    let enabled = runtime.snapshot().unwrap().capabilities;
    assert!(enabled.rear_view.granted);
    assert_eq!(
        enabled.rear_view.grant_id.as_deref(),
        Some("effective.rear_view")
    );
    assert!(enabled.items.iter().all(|item| !item.granted));
    runtime.unequip_build_slot("charm").unwrap();
    assert!(runtime.snapshot().unwrap().capabilities.rear_view.granted);
    runtime.unequip_build_slot("lens").unwrap();
    assert!(!runtime.snapshot().unwrap().capabilities.rear_view.granted);
    equip(&runtime, "rear_view_charm");
    grant_capability(&runtime, CAP_REAR_VIEW);
    runtime.unequip_build_slot("charm").unwrap();
    let permanent = runtime.snapshot().unwrap().capabilities;
    assert!(permanent.rear_view.granted);
    assert_eq!(permanent.rear_view.grant_id.as_deref(), Some(CAP_REAR_VIEW));
}

#[test]
fn rejected_equipment_leaves_live_projection_movement_and_build_unchanged() {
    let runtime = stopped_runtime(root(), Some("mh_drowned_quay"));
    equip(&runtime, "water_movement_module");
    let build = runtime.build_snapshot().unwrap();
    let view = runtime.snapshot().unwrap();
    let kcc = runtime.state.lock().unwrap().kcc.clone();
    assert_eq!(
        runtime
            .equip_build_item("terrain_penalty_ignore_boots")
            .unwrap_err(),
        "E_BUILD_ITEM_NOT_OWNED"
    );
    assert_eq!(
        runtime.unequip_build_slot("boots").unwrap_err(),
        "E_BUILD_SLOT_EMPTY"
    );
    assert_eq!(runtime.build_snapshot().unwrap(), build);
    assert_eq!(runtime.snapshot().unwrap(), view);
    assert_eq!(runtime.state.lock().unwrap().kcc, kcc);
}

#[test]
fn live_equipment_projection_and_movement_survive_disk_save_close_continue() {
    let root = root();
    let expected;
    {
        let runtime = stopped_runtime(root.clone(), Some("mh_drowned_quay"));
        for item in [
            "map_fog_bypass_array",
            "map_topology_chip",
            "water_movement_module",
            "rear_view_lens",
        ] {
            equip(&runtime, item);
        }
        measured_walk(&runtime, Vec3::new(14.0, 0.0, 7.5).unwrap(), 3.2);
        let capabilities = runtime.snapshot().unwrap().capabilities;
        expected = (
            capabilities.explored_map,
            capabilities.map_knowledge,
            runtime.state.lock().unwrap().world_persistent_v1.clone(),
        );
        runtime.save().unwrap();
    }
    {
        let runtime = stopped_runtime(root.clone(), Some("mh_drowned_quay"));
        let restored = runtime.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert_eq!(restored.capabilities.explored_map, expected.0);
        assert_eq!(restored.capabilities.map_knowledge, expected.1);
        assert!(restored.capabilities.rear_view.granted);
        assert_eq!(
            runtime.state.lock().unwrap().world_persistent_v1,
            expected.2
        );
        measured_walk(&runtime, Vec3::new(14.0, 0.0, 7.5).unwrap(), 3.2);
        runtime.unequip_build_slot("water_module").unwrap();
        measured_walk(&runtime, Vec3::new(14.0, 0.0, 7.5).unwrap(), 2.4);
        runtime.unequip_build_slot("map_array").unwrap();
        assert_eq!(
            runtime
                .snapshot()
                .unwrap()
                .capabilities
                .explored_map
                .rooms
                .len(),
            3
        );
        assert!(runtime
            .snapshot()
            .unwrap()
            .capabilities
            .map_knowledge
            .unwrap()
            .features
            .is_empty());
        runtime.unequip_build_slot("map_chip").unwrap();
        runtime.unequip_build_slot("lens").unwrap();
        runtime.save().unwrap();
    }
    {
        let runtime = stopped_runtime(root.clone(), Some("mh_drowned_quay"));
        let empty = runtime.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap().capabilities;
        assert_eq!(empty.map_topology_authorized, Some(false));
        assert!(empty.explored_map.rooms.is_empty());
        assert!(!empty.rear_view.granted);
        measured_walk(&runtime, Vec3::new(14.0, 0.0, 7.5).unwrap(), 2.4);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn trusted_heat_equipment_changes_actual_regulator_damage_and_removes_independently() {
    let runtime = stopped_runtime(root(), Some("cw_regulator_core"));
    {
        let mut state = runtime.state.lock().unwrap();
        let scene = runtime.scene_runtime.lock().unwrap();
        state.world.player.position_m = Vec3::new(18.0, 0.0, 5.0).unwrap();
        state.world.generic_actors[0].hp = 360;
        state.world.server_time_ms = 100;
        advance_regulator_environment(&mut state, scene.as_ref().unwrap()).unwrap();
        assert_eq!(state.world.player_hp, 100);
    }
    let pulse = |time, damage| {
        let mut state = runtime.state.lock().unwrap();
        let scene = runtime.scene_runtime.lock().unwrap();
        let before = state.world.player_hp;
        state.world.server_time_ms = time;
        advance_regulator_environment(&mut state, scene.as_ref().unwrap()).unwrap();
        assert_eq!(before - state.world.player_hp, damage);
    };
    pulse(700, 12);
    equip(&runtime, "heat_resistance_lining");
    pulse(1_700, 6);
    equip(&runtime, "heat_resistance_charm");
    pulse(2_700, 5); // 12 × 0.5 × 0.75 = 4.5, rounded by the authoritative damage path.
    runtime.unequip_build_slot("lining").unwrap();
    pulse(3_700, 9);
    runtime.unequip_build_slot("charm").unwrap();
    pulse(4_700, 12);
}

#[test]
fn map_uses_canonical_scene_geometry_and_command_receipts_keep_effective_channels() {
    let runtime = stopped_runtime(root(), Some("mh_drowned_quay"));
    equip(&runtime, "map_fog_bypass_array");
    let expected = runtime.snapshot().unwrap().capabilities;
    {
        let mut state = runtime.state.lock().unwrap();
        state.world.explored.rooms[0].outline_m[0].x_m = 999.0;
    }
    let canonical = runtime.snapshot().unwrap().capabilities;
    assert_eq!(canonical.map_knowledge, expected.map_knowledge);
    assert_eq!(canonical.explored_map.rooms, expected.explored_map.rooms);
    assert_eq!(
        runtime.pause().unwrap().capabilities.map_knowledge,
        expected.map_knowledge
    );
    assert_eq!(
        runtime.resume().unwrap().capabilities.map_knowledge,
        expected.map_knowledge
    );
}

#[test]
fn explored_terrain_projection_does_not_reveal_a_concave_unexplored_indentation() {
    let outer = polygon(&[
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 4.0],
        [3.0, 4.0],
        [3.0, 1.0],
        [1.0, 1.0],
        [1.0, 4.0],
        [0.0, 4.0],
    ]);
    let crossing = polygon(&[[0.5, 2.0], [3.5, 2.0], [3.5, 3.0], [0.5, 3.0]]);
    assert!(!convex_contains(&outer, &crossing));
    let rectangle = polygon(&[[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]]);
    assert!(convex_contains(&rectangle, &crossing));
}

#[test]
fn equipment_commands_advance_authority_revision_and_exhaustion_is_atomic() {
    let runtime = stopped_runtime(root(), None);
    let before = runtime.snapshot().unwrap().authority_revision;
    runtime.grant_trusted_build_item("rear_view_lens").unwrap();
    assert_eq!(runtime.snapshot().unwrap().authority_revision, before + 1);
    runtime.equip_build_item("rear_view_lens").unwrap();
    assert_eq!(runtime.snapshot().unwrap().authority_revision, before + 2);
    runtime
        .state
        .lock()
        .unwrap()
        .world
        .revision
        .authority_revision = u64::MAX;
    let build = runtime.build_snapshot().unwrap();
    let view = runtime.snapshot().unwrap();
    let kcc = runtime.state.lock().unwrap().kcc.clone();
    assert_eq!(
        runtime.unequip_build_slot("lens").unwrap_err(),
        "E_WORLD_REVISION: RevisionExhausted"
    );
    assert_eq!(runtime.build_snapshot().unwrap(), build);
    assert_eq!(runtime.snapshot().unwrap(), view);
    assert_eq!(runtime.state.lock().unwrap().kcc, kcc);
}

#[test]
fn rejected_interaction_receipts_preserve_all_equipment_map_channels() {
    let runtime = stopped_runtime(root(), Some("mh_pump_station"));
    equip(&runtime, "map_fog_bypass_array");
    let expected = runtime.snapshot().unwrap();
    for interaction in ["unknown_interaction", "mh_pump_control_primary"] {
        let rejected = runtime
            .activate_scene_interaction(interaction, interaction, expected.world_epoch)
            .unwrap();
        assert!(!rejected.applied);
        assert_eq!(rejected.view.capabilities, expected.capabilities);
        assert_eq!(
            rejected.view.authority_revision,
            expected.authority_revision
        );
    }
    let paused = runtime.pause().unwrap();
    let rejected = runtime
        .activate_scene_interaction("mh_pump_control_primary", "paused", expected.world_epoch)
        .unwrap();
    assert_eq!(rejected.error_code.as_deref(), Some("E_RUNTIME_PAUSED"));
    assert_eq!(rejected.view.capabilities, paused.capabilities);
    assert_eq!(rejected.view.authority_revision, paused.authority_revision);
}

#[test]
fn trusted_bloodline_lifecycle_changes_actual_heat_damage_with_equipment() {
    const STRONG: &str = "internal_thermal_adaptation";
    const LIGHT: &str = "internal_thermal_adaptation_light";
    let runtime = stopped_runtime(root(), Some("cw_regulator_core"));
    let original_capabilities = runtime.state.lock().unwrap().capabilities.clone();
    {
        let mut state = runtime.state.lock().unwrap();
        let scene = runtime.scene_runtime.lock().unwrap();
        state.world.player.position_m = Vec3::new(18.0, 0.0, 5.0).unwrap();
        state.world.generic_actors[0].hp = 360;
        state.world.server_time_ms = 100;
        advance_regulator_environment(&mut state, scene.as_ref().unwrap()).unwrap();
    }
    let pulse = |time, expected_damage| {
        let mut state = runtime.state.lock().unwrap();
        let scene = runtime.scene_runtime.lock().unwrap();
        let hp = state.world.player_hp;
        state.world.server_time_ms = time;
        advance_regulator_environment(&mut state, scene.as_ref().unwrap()).unwrap();
        assert_eq!(hp - state.world.player_hp, expected_damage);
    };
    pulse(700, 12);
    runtime.grant_trusted_build_bloodline(STRONG).unwrap();
    pulse(1_700, 6);
    equip(&runtime, "heat_resistance_lining");
    pulse(2_700, 3); // Two 50% sources leave 25%, never implicit immunity.
    runtime
        .replace_trusted_build_bloodline(STRONG, LIGHT)
        .unwrap();
    pulse(3_700, 5); // 12 * .5 * .75 rounds to 5 in the actual authoritative path.
    runtime.unequip_build_slot("lining").unwrap();
    pulse(4_700, 9);
    runtime.remove_trusted_build_bloodline(LIGHT).unwrap();
    pulse(5_700, 12);
    runtime.grant_trusted_build_bloodline(STRONG).unwrap();
    runtime.equip_build_item("heat_resistance_lining").unwrap();
    pulse(6_700, 3);
    runtime.remove_trusted_build_bloodline(STRONG).unwrap();
    pulse(7_700, 6); // Removing the bloodline retains the equipment source.
    runtime.unequip_build_slot("lining").unwrap();
    pulse(8_700, 12);
    assert_eq!(
        runtime.state.lock().unwrap().capabilities,
        original_capabilities
    );
}

#[test]
fn bloodline_authority_is_narrow_and_rejections_preserve_live_state() {
    const STRONG: &str = "internal_thermal_adaptation";
    let runtime = stopped_runtime(root(), Some("mh_drowned_quay"));
    let before = runtime.snapshot().unwrap().authority_revision;
    runtime.grant_trusted_build_bloodline(STRONG).unwrap();
    assert_eq!(runtime.snapshot().unwrap().authority_revision, before + 1);
    let rules = runtime.build_snapshot().unwrap().2;
    assert_eq!(
        rules
            .hazards
            .remaining_damage_bps(crate::effects::HazardTag::Heat),
        5000
    );
    assert_eq!(
        rules
            .hazards
            .remaining_damage_bps(crate::effects::HazardTag::Water),
        10000
    );
    assert_eq!(
        rules
            .hazards
            .remaining_damage_bps(crate::effects::HazardTag::Machinery),
        10000
    );
    assert_eq!(rules.map, EffectivePlayerRules::default().map);
    assert_eq!(rules.perception, EffectivePlayerRules::default().perception);
    assert_eq!(rules.terrain, EffectivePlayerRules::default().terrain);
    assert_eq!(rules.boundaries, EffectivePlayerRules::default().boundaries);
    measured_walk(&runtime, Vec3::new(14.0, 0.0, 7.5).unwrap(), 2.4);
    let build = runtime.build_snapshot().unwrap();
    let view = runtime.snapshot().unwrap();
    let kcc = runtime.state.lock().unwrap().kcc.clone();
    let rejected: [Box<dyn Fn() -> Result<(), String>>; 6] = [
        Box::new(|| runtime.grant_trusted_build_bloodline(STRONG)),
        Box::new(|| runtime.grant_trusted_build_bloodline("unknown_lineage")),
        Box::new(|| runtime.replace_trusted_build_bloodline("different_lineage", STRONG)),
        Box::new(|| runtime.replace_trusted_build_bloodline(STRONG, STRONG)),
        Box::new(|| runtime.replace_trusted_build_bloodline(STRONG, "unknown_lineage")),
        Box::new(|| runtime.remove_trusted_build_bloodline("different_lineage")),
    ];
    for command in rejected {
        assert!(command().is_err());
        assert_eq!(runtime.build_snapshot().unwrap(), build);
        assert_eq!(runtime.snapshot().unwrap(), view);
        assert_eq!(runtime.state.lock().unwrap().kcc, kcc);
    }
    runtime.remove_trusted_build_bloodline(STRONG).unwrap();
    assert_eq!(
        runtime.snapshot().unwrap().authority_revision,
        view.authority_revision + 1
    );
    let empty = runtime.build_snapshot().unwrap();
    assert!(runtime.remove_trusted_build_bloodline(STRONG).is_err());
    assert!(runtime
        .grant_trusted_build_bloodline("unknown_lineage")
        .is_err());
    assert_eq!(runtime.build_snapshot().unwrap(), empty);
}

#[test]
fn bloodline_commands_reject_revision_exhaustion_without_partial_install() {
    const STRONG: &str = "internal_thermal_adaptation";
    for operation in ["grant", "replace", "remove"] {
        let runtime = stopped_runtime(root(), None);
        if operation != "grant" {
            runtime.grant_trusted_build_bloodline(STRONG).unwrap();
        }
        runtime
            .state
            .lock()
            .unwrap()
            .world
            .revision
            .authority_revision = u64::MAX;
        let build = runtime.build_snapshot().unwrap();
        let view = runtime.snapshot().unwrap();
        let kcc = runtime.state.lock().unwrap().kcc.clone();
        let result = match operation {
            "grant" => runtime.grant_trusted_build_bloodline(STRONG),
            "replace" => {
                runtime.replace_trusted_build_bloodline(STRONG, "internal_thermal_adaptation_light")
            }
            _ => runtime.remove_trusted_build_bloodline(STRONG),
        };
        assert_eq!(result.unwrap_err(), "E_WORLD_REVISION: RevisionExhausted");
        assert_eq!(runtime.build_snapshot().unwrap(), build);
        assert_eq!(runtime.snapshot().unwrap(), view);
        assert_eq!(runtime.state.lock().unwrap().kcc, kcc);
    }
}

#[test]
fn capability_selection_refreshes_cached_map_rules_without_changing_acquisition_or_exploration() {
    use crate::{capability_v1::CAP_REGENERATION, formal_runtime::CapabilityCommandRequest};
    let runtime = stopped_runtime(root(), Some("mh_drowned_quay"));
    grant_capability(&runtime, CAP_LOCAL_MAP);
    grant_capability(&runtime, CAP_REGENERATION);
    let explored = runtime.state.lock().unwrap().world.explored.clone();
    let grants = runtime.state.lock().unwrap().capabilities.grants.clone();
    assert_eq!(
        runtime
            .snapshot()
            .unwrap()
            .capabilities
            .map_topology_authorized,
        Some(false)
    );
    let selected = runtime
        .apply_capability_command(CapabilityCommandRequest::Select {
            capability_ids: vec![CAP_LOCAL_MAP.into()],
        })
        .unwrap();
    assert_eq!(selected.capabilities.map_topology_authorized, Some(true));
    let cached_sources = runtime.state.lock().unwrap().effect_sources_v6.clone();
    assert_eq!(cached_sources, runtime.build_snapshot().unwrap().1);
    let unselected = runtime
        .apply_capability_command(CapabilityCommandRequest::Select {
            capability_ids: vec![CAP_REGENERATION.into()],
        })
        .unwrap();
    assert_eq!(unselected.capabilities.map_topology_authorized, Some(false));
    assert!(runtime
        .state
        .lock()
        .unwrap()
        .effective_rules_v6
        .capability_permissions
        .contains(&crate::effects::CapabilityPermission::DelayedRegeneration));
    equip(&runtime, "map_topology_chip");
    assert_eq!(
        runtime
            .snapshot()
            .unwrap()
            .capabilities
            .map_topology_authorized,
        Some(true)
    );
    runtime.unequip_build_slot("map_chip").unwrap();
    assert_eq!(
        runtime
            .snapshot()
            .unwrap()
            .capabilities
            .map_topology_authorized,
        Some(false)
    );
    assert_eq!(runtime.state.lock().unwrap().world.explored, explored);
    assert_eq!(runtime.state.lock().unwrap().capabilities.grants, grants);
    let before = runtime.snapshot().unwrap();
    let sources = runtime.build_snapshot().unwrap();
    assert!(runtime
        .apply_capability_command(CapabilityCommandRequest::Select {
            capability_ids: vec![crate::capability_v1::CAP_AIR_STEP.into()],
        })
        .is_err());
    assert_eq!(runtime.snapshot().unwrap(), before);
    assert_eq!(runtime.build_snapshot().unwrap(), sources);
    runtime
        .state
        .lock()
        .unwrap()
        .world
        .revision
        .authority_revision = u64::MAX;
    let before = runtime.snapshot().unwrap();
    assert_eq!(
        runtime
            .apply_capability_command(CapabilityCommandRequest::Select {
                capability_ids: vec![CAP_LOCAL_MAP.into()],
            })
            .unwrap_err(),
        "E_WORLD_REVISION: RevisionExhausted"
    );
    assert_eq!(runtime.snapshot().unwrap(), before);
    assert_eq!(runtime.build_snapshot().unwrap(), sources);
}

#[test]
fn unselected_acquired_regeneration_consumes_resolved_permission_in_the_actual_owner_tick() {
    use crate::{
        capability_v1::{RegenerationConfig, CAP_REGENERATION},
        formal_runtime::CapabilityCommandRequest,
    };
    let runtime = stopped_runtime(root(), None);
    grant_capability(&runtime, CAP_REGENERATION);
    grant_capability(&runtime, CAP_LOCAL_MAP);
    runtime
        .apply_capability_command(CapabilityCommandRequest::Select {
            capability_ids: vec![CAP_LOCAL_MAP.into()],
        })
        .unwrap();
    {
        let mut state = runtime.state.lock().unwrap();
        // Isolated owner-consumer fixture: no unrelated actor attacks or production tuning edits.
        state.world.sentinels.clear();
        state.world.generic_actors.clear();
        state.world.player_hp = 50;
        state.world.player_max_hp = 100;
        state.capabilities.regeneration = RegenerationConfig::new(1000, 3000, 60.0, 0.75).unwrap();
        state.last_damaged_at_ms = Some(state.world.server_time_ms);
        state.last_combat_at_ms = Some(state.world.server_time_ms);
        let eligible_at = state.world.server_time_ms + 3000;
        assert!(!state
            .capabilities
            .selected
            .iter()
            .any(|id| id == CAP_REGENERATION));
        let mut eligible_ticks = 0;
        for _ in 0..400 {
            let hp = state.world.player_hp;
            advance_owner_step_with_scene(&mut state, None).unwrap();
            let expected = if state.world.server_time_ms >= eligible_at && hp < 75 {
                1
            } else {
                0
            };
            assert_eq!(state.world.player_hp - hp, expected);
            eligible_ticks += usize::from(expected == 1);
        }
        assert_eq!(eligible_ticks, 25);
        assert_eq!(state.world.player_hp, 75);
        assert_eq!(state.capabilities.regeneration.hp_per_second, 60.0);
    }
}

#[test]
fn acoustic_bearings_follow_selected_source_policy_at_actual_authored_interaction() {
    use crate::{
        capability_v1::{CAP_ACOUSTIC_MAPPING, CAP_REGENERATION},
        formal_runtime::CapabilityCommandRequest,
    };
    for selected in [false, true] {
        let runtime = stopped_runtime(root(), Some("mh_tidal_warehouse"));
        grant_capability(&runtime, CAP_ACOUSTIC_MAPPING);
        grant_capability(&runtime, CAP_REGENERATION);
        runtime
            .apply_capability_command(CapabilityCommandRequest::Select {
                capability_ids: vec![if selected {
                    CAP_ACOUSTIC_MAPPING
                } else {
                    CAP_REGENERATION
                }
                .into()],
            })
            .unwrap();
        runtime.state.lock().unwrap().world.player.position_m = Vec3::new(16.0, 0.0, 10.0).unwrap();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let result = runtime
            .activate_scene_interaction("mh_west_beacon", "source-policy-beacon", epoch)
            .unwrap();
        assert!(result.applied, "{:?}", result.error_code);
        let cues = runtime.sound_cues_since(epoch, 0).unwrap();
        assert_eq!(cues.len(), 1);
        if selected {
            assert_eq!(cues[0].direction_rad, Some(std::f32::consts::FRAC_PI_2));
            assert_eq!(cues[0].distance_m, Some(1.0));
        } else {
            assert_eq!(cues[0].direction_rad, None);
            assert_eq!(cues[0].distance_m, None);
        }
    }
}

#[test]
fn permanent_rear_view_survives_scene_reconstruction_without_accepting_spoofed_records() {
    let runtime = stopped_runtime(root(), Some("mh_drowned_quay"));
    grant_capability(&runtime, CAP_REAR_VIEW);
    let registry = runtime
        .scene_registry
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .clone();
    for scene in ["mh_pump_station", "rs_core_room", "gh_entry_maintenance"] {
        let view = runtime
            .install_scene_registry(registry.clone(), scene)
            .unwrap();
        assert!(view.capabilities.rear_view.granted);
        assert_eq!(
            view.capabilities.rear_view.grant_id.as_deref(),
            Some(CAP_REAR_VIEW)
        );
        assert!(
            runtime
                .state
                .lock()
                .unwrap()
                .effective_rules_v6
                .perception
                .rear_view
        );
    }
    {
        let mut state = runtime.state.lock().unwrap();
        state.world.rear_view =
            RearViewAuthorization::granted("browser.forged", state.world.revision).unwrap();
        let build = state.progression_v6.clone();
        state.install_build(build).unwrap();
    }
    let view = runtime
        .install_scene_registry(registry, "rs_core_room")
        .unwrap();
    assert!(!view.capabilities.rear_view.granted);
    assert!(view
        .capabilities
        .items
        .iter()
        .any(|item| item.capability_id == CAP_REAR_VIEW && item.granted));
}

#[test]
fn legacy_capability_sources_restore_live_cache_and_nondefault_regeneration_without_inferred_selection(
) {
    use crate::{
        capability_v1::{CAP_ACOUSTIC_MAPPING, CAP_REGENERATION},
        formal_runtime::CapabilityCommandRequest,
    };
    const OLD: &str =
        include_str!("../../../tests/fixtures/capability-save-profiles/legacy-mixed.json");
    let dir = root();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(crate::save_v6::save_path(&dir), OLD).unwrap();
    let old: serde_json::Value = serde_json::from_str(OLD).unwrap();
    {
        let runtime = stopped_runtime(dir.clone(), None);
        runtime.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        let (sources, rules, capabilities) = {
            let state = runtime.state.lock().unwrap();
            (
                state.effect_sources_v6.clone(),
                state.effective_rules_v6.clone(),
                serde_json::to_value(&state.capabilities).unwrap(),
            )
        };
        let snapshot = runtime.build_snapshot().unwrap();
        assert_eq!((sources, rules), (snapshot.1, snapshot.2));
        for key in [
            "regeneration",
            "lastNowMs",
            "lastDamageAtMs",
            "lastCombatAtMs",
            "fractionalHeal",
            "handledRequestIds",
        ] {
            assert_eq!(
                capabilities[key], old["save"]["capabilities"][key],
                "changed {key}"
            );
        }
        assert!(runtime
            .state
            .lock()
            .unwrap()
            .effective_rules_v6
            .capability_permissions
            .contains(&crate::effects::CapabilityPermission::AcousticMapping));
        runtime
            .apply_capability_command(CapabilityCommandRequest::Select {
                capability_ids: vec![CAP_REGENERATION.into()],
            })
            .unwrap();
        assert_eq!(
            runtime
                .snapshot()
                .unwrap()
                .capabilities
                .map_topology_authorized,
            Some(false)
        );
        assert!(!runtime
            .state
            .lock()
            .unwrap()
            .effective_rules_v6
            .capability_permissions
            .contains(&crate::effects::CapabilityPermission::AcousticMapping));
        runtime.save().unwrap();
    }
    {
        let runtime = stopped_runtime(dir.clone(), None);
        let restored = runtime.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert_eq!(restored.capabilities.map_topology_authorized, Some(false));
        assert!(restored
            .capabilities
            .items
            .iter()
            .any(|item| item.capability_id == CAP_ACOUSTIC_MAPPING
                && item.granted
                && !item.selected));
        let state = runtime.state.lock().unwrap();
        assert!(!state
            .effective_rules_v6
            .capability_permissions
            .contains(&crate::effects::CapabilityPermission::AcousticMapping));
        assert!(state
            .effective_rules_v6
            .capability_permissions
            .contains(&crate::effects::CapabilityPermission::DelayedRegeneration));
        assert_eq!(state.capabilities.regeneration.hp_per_second, 1.75);
    }
    assert_eq!(
        std::fs::read(dir.join("formal-save-v6.json.legacy-effect-sources-v1.bak")).unwrap(),
        OLD.as_bytes()
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rear_view_survives_public_world_gate_and_scene_transition_without_opening_locked_worlds() {
    let runtime = stopped_runtime(root(), Some("rs_core_room"));
    {
        let mut state = runtime.state.lock().unwrap();
        // Return Station retains the campaign's current world; this fixture changes no unlocks.
        state.route.current_world_id = "grey_hive".into();
        state.world.player.position_m = Vec3::new(20.0, 0.0, 8.0).unwrap();
    }
    grant_capability(&runtime, CAP_REAR_VIEW);
    let before = runtime.snapshot().unwrap();
    assert_eq!(
        runtime
            .use_world_gate("rs_world_gate_to_mh", "rear-mh-locked", before.world_epoch)
            .unwrap_err(),
        "E_WORLD_GATE_GH_EXTRACTION_REQUIRED"
    );
    assert_eq!(runtime.snapshot().unwrap(), before);
    let sources = runtime.build_snapshot().unwrap().1;
    let entered = runtime
        .use_world_gate("rs_world_gate_to_gh", "rear-gh-enter", before.world_epoch).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
        .unwrap();
    assert_eq!(entered.scene_id, "gh_entry_maintenance");
    assert!(entered.capabilities.rear_view.granted);
    assert_eq!(
        entered.capabilities.rear_view.grant_id.as_deref(),
        Some(CAP_REAR_VIEW)
    );
    assert_eq!(runtime.build_snapshot().unwrap().1, sources);
    runtime.state.lock().unwrap().world.player.position_m = Vec3::new(19.0, 0.0, 7.0).unwrap();
    let moved = runtime
        .transition_scene(
            "gh_entry_to_power_room",
            "rear-power-enter",
            entered.world_epoch,
        ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
        .unwrap();
    assert_eq!(moved.scene_id, "gh_power_room");
    assert!(moved.capabilities.rear_view.granted);
    assert_eq!(
        moved.capabilities.rear_view.grant_id.as_deref(),
        Some(CAP_REAR_VIEW)
    );
    assert_eq!(runtime.build_snapshot().unwrap().1, sources);
    assert!(
        runtime
            .state
            .lock()
            .unwrap()
            .effective_rules_v6
            .perception
            .rear_view
    );
}
