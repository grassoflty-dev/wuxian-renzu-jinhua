extends Node2D
const P = preload('res://scripts/bridge/protocol.gd')
const MhDrownedQuayEnvironmentVisual = preload('res://scripts/world/mh_drowned_quay_environment_visual.gd')
var mh_drowned_quay_environment_visual = MhDrownedQuayEnvironmentVisual.new()
const MhSignalYardEnvironmentVisual = preload('res://scripts/world/mh_signal_yard_environment_visual.gd')
var mh_signal_yard_environment_visual = MhSignalYardEnvironmentVisual.new()
const MhTidalWarehouseEnvironmentVisual = preload('res://scripts/world/mh_tidal_warehouse_environment_visual.gd')
var mh_tidal_warehouse_environment_visual = MhTidalWarehouseEnvironmentVisual.new()
const MhWarehouseCratesVisual = preload('res://scripts/world/mh_warehouse_crates_visual.gd')
var mh_warehouse_crates_visual = MhWarehouseCratesVisual.new()
const MhWardenArenaEnvironmentVisual = preload('res://scripts/world/mh_warden_arena_environment_visual.gd')
var mh_warden_arena_environment_visual = MhWardenArenaEnvironmentVisual.new()
const V = preload('res://scripts/world/presentation.gd')
const CombatOutcomeText = preload('res://scripts/ui/combat_feedback_model.gd')
const AcousticEventView = preload('res://scripts/world/event_view.gd')
const Camera = preload('res://scripts/world/projection.gd')
const MhFogPierGroundVisual = preload('res://scripts/world/mh_fog_pier_ground_visual.gd')
const MhFogPierStoneMaterialVisual = preload('res://scripts/world/mh_fog_pier_stone_material_visual.gd')
const MhFogPierLayeredFogVisual = preload('res://scripts/world/mh_fog_pier_layered_fog_visual.gd')
const MhFogPierEnvironmentVisual = preload('res://scripts/world/mh_fog_pier_environment_visual.gd')
const CenYaoIdle = preload('res://scripts/world/cenyao_idle_visual.gd')
const PlayerStatusUiArt = preload('res://scripts/ui/ui_art_texture.gd')
const CenYaoEastWalk = preload('res://scripts/world/cenyao_east_walk_visual.gd')
const MhCwActor = preload('res://scripts/world/mh_cw_actor_visual.gd')
const RsEnvironmentVisual = preload('res://scripts/world/rs_environment_visual.gd')
const BaizhiStaticBodyVisual = preload('res://scripts/world/baizhi_static_body_visual.gd')
const RsSourceOwnedVisual = preload('res://scripts/world/rs_source_owned_visual.gd')
const MhCwAmbientBackdropVisual = preload('res://scripts/world/mh_cw_ambient_backdrop_visual.gd')
const RsBulkheadModulesVisual = preload('res://scripts/world/rs_bulkhead_modules_visual.gd')
const RsServicePipeVisual = preload('res://scripts/world/rs_service_pipe_visual.gd')
const RsExteriorBackplateVisual = preload('res://scripts/world/rs_exterior_backplate_visual.gd')
const WholeSceneCoverageVisual = preload('res://scripts/world/whole_scene_coverage_visual.gd')
const GhEntryExteriorBackplateVisual = preload('res://scripts/world/gh_entry_exterior_backplate_visual.gd')
const GhEntryR2PlateCandidate = preload('res://scripts/world/gh_entry_r2_plate_candidate.gd')
const GhDeepDeconExteriorBackplateVisual = preload('res://scripts/world/gh_deep_decon_exterior_backplate_visual.gd')
const GhBeaconExteriorBackplateVisual = preload('res://scripts/world/gh_beacon_exterior_backplate_visual.gd')
const GhBioIsolationExteriorBackplateVisual = preload('res://scripts/world/gh_bio_isolation_exterior_backplate_visual.gd')
const GhLockdownExteriorBackplateVisual = preload('res://scripts/world/gh_lockdown_exterior_backplate_visual.gd')
const GhPowerExteriorBackplateVisual = preload('res://scripts/world/gh_power_exterior_backplate_visual.gd')
const GhExitExteriorBackplateVisual = preload('res://scripts/world/gh_exit_exterior_backplate_visual.gd')
const GhGateAExteriorBackplateVisual = preload('res://scripts/world/gh_gate_a_exterior_backplate_visual.gd')
const GhGateBExteriorBackplateVisual = preload('res://scripts/world/gh_gate_b_exterior_backplate_visual.gd')
const GhCentralShaftExteriorBackplateVisual = preload('res://scripts/world/gh_central_shaft_exterior_backplate_visual.gd')
const MhResonanceTowerExteriorBackplateVisual = preload('res://scripts/world/mh_resonance_tower_exterior_backplate_visual.gd')
const CwRegulatorCoreExteriorBackplateVisual = preload('res://scripts/world/cw_regulator_core_exterior_backplate_visual.gd')
const MhSignalYardExteriorBackplateVisual = preload('res://scripts/world/mh_signal_yard_exterior_backplate_visual.gd')
const CwConveyorBridgeExteriorBackplateVisual = preload('res://scripts/world/cw_conveyor_bridge_exterior_backplate_visual.gd')
const CwForgedGuardArenaExteriorBackplateVisual = preload('res://scripts/world/cw_forged_guard_arena_exterior_backplate_visual.gd')
const MhWardenArenaExteriorBackplateVisual = preload('res://scripts/world/mh_warden_arena_exterior_backplate_visual.gd')
const CwBoilerChamberExteriorBackplateVisual = preload('res://scripts/world/cw_boiler_chamber_exterior_backplate_visual.gd')
const CwFurnaceHeartExteriorBackplateVisual = preload('res://scripts/world/cw_furnace_heart_exterior_backplate_visual.gd')
const MhFogPierExteriorBackplateVisual = preload('res://scripts/world/mh_fog_pier_exterior_backplate_visual.gd')
const MhFogPierExteriorR2BackplateVisual = preload('res://scripts/world/mh_fog_pier_exterior_r2_backplate_visual.gd')
const CwEntryFoundryExteriorBackplateVisual = preload('res://scripts/world/cw_entry_foundry_exterior_backplate_visual.gd')
const CwGearShaftBackdropVisual = preload('res://scripts/world/cw_gear_shaft_backdrop_visual.gd')
const GhBioWallModulesVisual = preload('res://scripts/world/gh_bio_wall_modules_visual.gd')
const GhBioServicePipeVisual = preload('res://scripts/world/gh_bio_service_pipe_visual.gd')
const GhBioFloorMaterialVisual = preload('res://scripts/world/gh_bio_floor_material_visual.gd')
const GhEntryFloorMaterialVisual = preload('res://scripts/world/gh_entry_floor_material_visual.gd')
const RsFloorTreatmentVisual = preload('res://scripts/world/rs_floor_treatment_visual.gd')
const CwPressureFloorMaterialVisual = preload('res://scripts/world/cw_pressure_floor_material_visual.gd')
const CwConveyorEnvironmentVisual = preload('res://scripts/world/cw_conveyor_environment_visual.gd')
const CwConveyorBeltDustVisual = preload('res://scripts/world/cw_conveyor_belt_dust_visual.gd')
const CwGearShaftEnvironmentVisual = preload('res://scripts/world/cw_gear_shaft_environment_visual.gd')
const MhBreakwaterEnvironmentVisual = preload('res://scripts/world/mh_breakwater_environment_visual.gd')
const MhResonanceTowerEnvironmentVisual = preload('res://scripts/world/mh_resonance_tower_environment_visual.gd')
const MhExtractionEnvironmentVisual = preload('res://scripts/world/mh_extraction_environment_visual.gd')
const GhSentinelFloorMaterialVisual = preload('res://scripts/world/gh_sentinel_floor_material_visual.gd')
const GhSentinelEnvironmentVisual = preload('res://scripts/world/gh_sentinel_environment_visual.gd')
const GhShaftDustVisual = preload('res://scripts/world/gh_shaft_dust_visual.gd')
const GhDeepDeconFloorMaterialVisual = preload('res://scripts/world/gh_deep_decon_floor_material_visual.gd')
const GhDeepDeconEnvironmentVisual = preload('res://scripts/world/gh_deep_decon_environment_visual.gd')
const GhDeepDeconAlcoveVisual = preload('res://scripts/world/gh_deep_decon_alcove_visual.gd')
const GhGateAEnvironmentVisual = preload('res://scripts/world/gh_gate_a_environment_visual.gd')
const GhPowerFloorMaterialVisual = preload('res://scripts/world/gh_power_floor_material_visual.gd')
const GhPowerGateNeighborVisual = preload('res://scripts/world/gh_power_gate_neighbor_visual.gd')
const GhDenseServiceWingVisual = preload('res://scripts/world/gh_dense_service_wing_visual.gd')
const GhDenseRegionComposition = preload('res://scripts/world/gh_dense_region_composition.gd')
const PlayerActionVfxArtVisual = preload('res://scripts/world/player_action_vfx_art_visual.gd')
const GhEntryWallModulesVisual = preload('res://scripts/world/gh_entry_wall_modules_visual.gd')
const GhEntryServicePipeVisual = preload('res://scripts/world/gh_entry_service_pipe_visual.gd')
const GhLockdownEnvironmentVisual = preload('res://scripts/world/gh_lockdown_environment_visual.gd')
const GhEntryAmbientVisual = preload('res://scripts/world/gh_entry_ambient_visual.gd')
const GhBeaconEnvironmentVisual = preload('res://scripts/world/gh_beacon_environment_visual.gd')
const GhGateBEnvironmentVisual = preload('res://scripts/world/gh_gate_b_environment_visual.gd')
const GhGateBWestCableVisual = preload('res://scripts/world/gh_gate_b_west_cable_visual.gd')
const GhDeepDeconAirVisual = preload('res://scripts/world/gh_deep_decon_air_visual.gd')
const GhCentralShaftEnvironmentVisual = preload('res://scripts/world/gh_central_shaft_environment_visual.gd')
const GhShaftLiftVisual = preload('res://scripts/world/gh_shaft_lift_visual.gd')
const GhShaftLowerServiceVisual = preload('res://scripts/world/gh_shaft_lower_service_visual.gd')
const GhGateShaftPortalVisual = preload('res://scripts/world/gh_gate_shaft_portal_visual.gd')
const GhGateDeconVisual = preload('res://scripts/world/gh_gate_decon_visual.gd')
const GhBeaconExitNeighborVisual = preload('res://scripts/world/gh_beacon_exit_neighbor_visual.gd')
const GhRouteNeighborVisual = preload('res://scripts/world/gh_route_neighbor_visual.gd')
const GhUtilityLayoutVisual = preload('res://scripts/world/gh_utility_layout_visual.gd')
const GhDeconRegionVisual = preload('res://scripts/world/gh_decon_region_visual.gd')
const GhFloorRouteRegionVisual = preload('res://scripts/world/gh_floor_route_region_visual.gd')
const GhBioRegionVisual = preload('res://scripts/world/gh_bio_region_visual.gd')
const GhBeaconExitRegionVisual = preload('res://scripts/world/gh_beacon_exit_region_visual.gd')
const GhSwarmMemberTemporalVisual = preload('res://scripts/world/gh_swarm_member_temporal_visual.gd')
const GhSentinelTemporalVisual = preload('res://scripts/world/gh_sentinel_temporal_visual.gd')
const CwPrimeRegulatorTemporalVisual = preload('res://scripts/world/cw_prime_regulator_temporal_visual.gd')
const CwDroneHoundTemporalVisual = preload('res://scripts/world/cw_drone_hound_temporal_visual.gd')
const MhDrownedTideboundTemporalVisual = preload('res://scripts/world/mh_drowned_tidebound_temporal_visual.gd')
const MhWardenTemporalVisual = preload('res://scripts/world/mh_warden_temporal_visual.gd')
const MhShoreRouteVisual = preload('res://scripts/world/mh_shore_route_visual.gd')
const CwEarlyRouteVisual = preload('res://scripts/world/cw_early_route_visual.gd')
const CwLaterRouteVisual = preload('res://scripts/world/cw_later_route_visual.gd')
const MH_DROWNED_TIDEBOUND_READY = true # Both originals passed provisional purpose fit.
const MH_TIDEBOUND_IDS = {'mh_drowned_quay':['mh_drowned_quay_tidebound_01'], 'mh_breakwater':['mh_breakwater_tidebound_01','mh_breakwater_tidebound_02'], 'mh_resonance_tower':['mh_resonance_tower_tidebound_01']}
const CW_DRONE_HOUND_READY = true # Both original atlases passed provisional purpose fit.
const CW_DRONE_HOUND_SCENES = {
	'cw_pressure_hall': {'pressure_drone': 2}, 'cw_conveyor_bridge': {'pressure_drone': 2},
	'cw_gear_shaft': {'pressure_drone': 3}, 'cw_boiler_chamber': {'furnace_hound': 3},
	'cw_furnace_heart': {'furnace_hound': 3}}
const CW_REGULAR_GUARDS = {
	'cw_entry_foundry':['cw_entry_foundry_forged_guard_01','cw_entry_foundry_forged_guard_02'],
	'cw_pressure_hall':['cw_pressure_hall_forged_guard_01','cw_pressure_hall_forged_guard_02'],
	'cw_conveyor_bridge':['cw_conveyor_bridge_forged_guard_01','cw_conveyor_bridge_forged_guard_02','cw_conveyor_bridge_forged_guard_03'],
	'cw_gear_shaft':['cw_gear_shaft_forged_guard_01','cw_gear_shaft_forged_guard_02'],
	'cw_furnace_heart':['cw_furnace_heart_forged_guard_01','cw_furnace_heart_forged_guard_02']}
# The single produced atlas has passed its provisional purpose fit.
const SWARM_ATLAS_READY = true
const SENTINEL_ATLAS_READY = true
const PRIME_REGULATOR_ATLAS_READY = true
const GhExitSourceOwnedVisual = preload('res://scripts/world/gh_exit_source_owned_visual.gd')
const GhExitEnvironmentVisual = preload('res://scripts/world/gh_exit_environment_visual.gd')
const GhExitSealedBarrierVisual = preload('res://scripts/world/gh_exit_sealed_barrier_visual.gd')
var gh_beacon_environment_visual = GhBeaconEnvironmentVisual.new()
var gh_central_shaft_environment_visual = GhCentralShaftEnvironmentVisual.new()
var gh_shaft_lift_visual = GhShaftLiftVisual.new()
var gh_shaft_lower_service_visual = GhShaftLowerServiceVisual.new()
var gh_gate_shaft_portal_visual = GhGateShaftPortalVisual.new()
var gh_gate_decon_visual = GhGateDeconVisual.new()
var gh_beacon_exit_neighbor_visual = GhBeaconExitNeighborVisual.new()
var gh_route_neighbor_visual = GhRouteNeighborVisual.new()
var hive_route_frame: Dictionary = {}
var hive_route_frame_at: int = 0
var hive_route_cache: Dictionary = {}
var hive_route_drawn: int = 0
var hive_route_draw_failed: bool = false
var hive_route_pending_key: Array = []
var hive_route_pending_at: int = 0
var beacon_exit_frame: Dictionary = {}
var beacon_exit_frame_at: int = 0
var beacon_exit_cache: Dictionary = {}
var gh_utility_layout_visual = GhUtilityLayoutVisual.new()
var gh_decon_region_visual = GhDeconRegionVisual.new()
var gh_floor_route_region_visual = GhFloorRouteRegionVisual.new()
var gh_bio_region_visual = GhBioRegionVisual.new()
var gh_beacon_exit_region_visual = GhBeaconExitRegionVisual.new()
var gh_swarm_member_temporal_visual = GhSwarmMemberTemporalVisual.new()
var gh_sentinel_temporal_visual = GhSentinelTemporalVisual.new()
var cw_prime_regulator_temporal_visual = CwPrimeRegulatorTemporalVisual.new()
var cw_drone_hound_by_id: Dictionary = {}
var cw_drone_hound_sampled: Dictionary = {}
var mh_drowned_tidebound_by_id: Dictionary = {}
var mh_drowned_tidebound_sampled: Dictionary = {}
var mh_warden_temporal_visual = MhWardenTemporalVisual.new()
var mh_warden_sampled := false
var mh_shore_route_visual = MhShoreRouteVisual.new()
var cw_early_route_visual = CwEarlyRouteVisual.new()
var cw_later_route_visual = CwLaterRouteVisual.new()
var cw_later_owned := false
var cw_early_owned := false
var cw_early_bound_key := ''
var prime_regulator_sampled := false
var cw_regular_guard_temporal_by_id: Dictionary = {}
var cw_regular_guard_sampled: Dictionary = {}
var swarm_temporal_animate := false
var swarm_art_unready := false
var sentinel_temporal_animate := false
var sentinel_temporal_drawn := false
var sentinel_art_unready := false
var gh_exit_source_owned_visual = GhExitSourceOwnedVisual.new()
var exit_source_plan: Dictionary = {}
var exit_source_frame: Dictionary = {}
var exit_source_frame_at := 0
var exit_source_drawn := 0
var exit_source_failed := false
var utility_plan: Dictionary = {}
var utility_frame: Dictionary = {}
var utility_frame_at := 0
var utility_drawn_pieces := 0
var utility_draw_failed := false
var utility_bio_wall_drawn := 0
var utility_beacon_wall_drawn := 0
var gate_decon_frame: Dictionary = {}
var gate_decon_frame_at: int = 0
var lockdown_frame: Dictionary = {}
var lockdown_frame_at: int = 0
var shaft_lockdown_seal_frame: Dictionary = {}
var shaft_lockdown_seal_frame_at: int = 0
var gate_shaft_portal_frame: Dictionary = {}
var gate_shaft_portal_frame_at: int = 0
var gh_exit_environment_visual = GhExitEnvironmentVisual.new()
var gh_exit_sealed_barrier_visual = GhExitSealedBarrierVisual.new()
const MH_CW_ROSTER = {"mh_resonance_warden_staged_marker":{"runtime_type":"runtime2d.enemy.mistharbor.signal_wraith.v2","world":"mist_harbor","key":"signal_wraith","scene":"mh_warden_arena","authoring_type":"enemy.mist_harbor.resonance_warden"},"cw_prime_regulator":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_regulator_core","authoring_type":"enemy.clockworks.prime_regulator"},"cw_forged_guard_elite":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_forged_guard_arena","authoring_type":"enemy.clockworks.forged_guard_elite"},"cw_entry_foundry_forged_guard_01":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_entry_foundry","authoring_type":"enemy.clockworks.forged_guard"},"mh_breakwater_signal_wraith_02":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_breakwater","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_tidal_warehouse_drowned_01":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_tidal_warehouse","authoring_type":"enemy.mist_harbor.drowned"},"mh_signal_yard_signal_wraith_03":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_signal_yard","authoring_type":"enemy.mist_harbor.signal_wraith"},"cw_pressure_hall_forged_guard_01":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_pressure_hall","authoring_type":"enemy.clockworks.forged_guard"},"mh_signal_yard_signal_wraith_02":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_signal_yard","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_resonance_tower_signal_wraith_01":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_resonance_tower","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_tidal_warehouse_drowned_02":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_tidal_warehouse","authoring_type":"enemy.mist_harbor.drowned"},"mh_drowned_quay_drowned_03":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_drowned_quay","authoring_type":"enemy.mist_harbor.drowned"},"mh_drowned_quay_drowned_02":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_drowned_quay","authoring_type":"enemy.mist_harbor.drowned"},"mh_drowned_quay_drowned_01":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_drowned_quay","authoring_type":"enemy.mist_harbor.drowned"},"mh_fog_pier_drowned_02":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_fog_pier","authoring_type":"enemy.mist_harbor.drowned"},"mh_drowned_quay_drowned_04":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_drowned_quay","authoring_type":"enemy.mist_harbor.drowned"},"mh_signal_yard_signal_wraith_01":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_signal_yard","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_pump_station_drowned_03":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_pump_station","authoring_type":"enemy.mist_harbor.drowned"},"mh_tidal_warehouse_signal_wraith_01":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_tidal_warehouse","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_breakwater_signal_wraith_01":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_breakwater","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_fog_pier_drowned_01":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_fog_pier","authoring_type":"enemy.mist_harbor.drowned"},"cw_furnace_heart_forged_guard_01":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_furnace_heart","authoring_type":"enemy.clockworks.forged_guard"},"cw_conveyor_bridge_forged_guard_01":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_conveyor_bridge","authoring_type":"enemy.clockworks.forged_guard"},"mh_resonance_tower_signal_wraith_02":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_resonance_tower","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_pump_station_drowned_02":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_pump_station","authoring_type":"enemy.mist_harbor.drowned"},"cw_pressure_hall_forged_guard_02":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_pressure_hall","authoring_type":"enemy.clockworks.forged_guard"},"cw_furnace_heart_forged_guard_02":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_furnace_heart","authoring_type":"enemy.clockworks.forged_guard"},"cw_gear_shaft_forged_guard_02":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_gear_shaft","authoring_type":"enemy.clockworks.forged_guard"},"mh_pump_station_drowned_01":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_pump_station","authoring_type":"enemy.mist_harbor.drowned"},"cw_gear_shaft_forged_guard_01":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_gear_shaft","authoring_type":"enemy.clockworks.forged_guard"},"mh_resonance_tower_signal_wraith_03":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_resonance_tower","authoring_type":"enemy.mist_harbor.signal_wraith"},"cw_entry_foundry_forged_guard_02":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_entry_foundry","authoring_type":"enemy.clockworks.forged_guard"},"cw_conveyor_bridge_forged_guard_02":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_conveyor_bridge","authoring_type":"enemy.clockworks.forged_guard"},"mh_resonance_tower_signal_wraith_04":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_resonance_tower","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_tidal_warehouse_drowned_03":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_tidal_warehouse","authoring_type":"enemy.mist_harbor.drowned"},"cw_conveyor_bridge_forged_guard_03":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_conveyor_bridge","authoring_type":"enemy.clockworks.forged_guard"}}
const GreyHiveActor = preload('res://scripts/world/grey_hive_actor_visual.gd')
const GhGateBBruteTemporalVisual = preload('res://scripts/world/gh_gate_b_brute_temporal_visual.gd')
const MhSignalWraithTemporalVisual = preload('res://scripts/world/mh_signal_wraith_temporal_visual.gd')
const GhMaintenanceTemporalVisual = preload('res://scripts/world/gh_maintenance_temporal_visual.gd')
const GhSecurityTemporalVisual = preload('res://scripts/world/gh_security_temporal_visual.gd')
const RegisteredProps = preload('res://scripts/world/registered_props_visual.gd')
const GreyHiveStateProps = preload('res://scripts/world/grey_hive_state_props_visual.gd')
const GhBioLockdownVisual = preload('res://scripts/world/gh_bio_lockdown_visual.gd')
const GhBioClutterVisual = preload('res://scripts/world/gh_bio_clutter_visual.gd')
const GhSentinelCircleWarningVisual = preload('res://scripts/world/gh_sentinel_circle_warning_visual.gd')
const GhBioSampleCaseVisual = preload('res://scripts/world/gh_bio_sample_case_visual.gd')
const GhEntryClutterVisual = preload('res://scripts/world/gh_entry_clutter_visual.gd')
const GhSupplyCrateVisual = preload('res://scripts/world/gh_supply_crate_visual.gd')
const GhPowerEnvironmentVisual = preload('res://scripts/world/gh_power_environment_visual.gd')
const BaizhiPlaceholderVisual = preload('res://scripts/world/baizhi_placeholder_visual.gd')
var baizhi_placeholder_visual
const GH_CRATES = {
	'power_room_crates':['gh_power_room',Vector3(14,0,10)],
	'gate_a_service_crates':['gh_gate_a',Vector3(14,0,10)],
	'lockdown_supply_crates':['gh_lockdown',Vector3(20,0,11)],
	'bio_supply_crates':['gh_bio_isolation',Vector3(18,0,11.5)],
	'gate_b_supply_crates':['gh_gate_b',Vector3(5,0,11.5)],
	'deep_decon_safety_corner_crates':['gh_deep_decon',Vector3(3,0,12.5)],
	'beacon_service_crates':['gh_beacon',Vector3(5,0,11.5)],
	'exit_console_crates':['gh_exit',Vector3(18,0,11.5)]
}
var gh_supply_crate_visual
const Content = preload('res://scripts/ui/content_model.gd')
const GH_STORY_PLACEMENTS = {
	'exit_extraction_console_approved':{'key':'lockdown_terminal','scene':'gh_exit','asset':'runtime2d.prop.lockdown_terminal.v1','position':Vector3(8,0,8)},
	'bio_pod_approved':{'key':'bio_pod','scene':'gh_bio_isolation','asset':'runtime2d.prop.bio_pod.v1','position':Vector3(8,0,7)},
	'medical_station_approved':{'key':'medical_station','scene':'gh_bio_isolation','asset':'runtime2d.prop.medical_station.v1','position':Vector3(16,0,5)},
	'bio_log_terminal_approved':{'key':'lockdown_terminal','scene':'gh_bio_isolation','asset':'runtime2d.prop.lockdown_terminal.v1','position':Vector3(20,0,8)},
	'lockdown_terminal_visual':{'key':'lockdown_terminal','scene':'gh_lockdown','asset':'runtime2d.prop.lockdown_terminal.v1','position':Vector3(4,0,8)}
}
var gh_bio_lockdown_visual
const CwPressureValveVisual = preload('res://scripts/world/cw_pressure_valve_visual.gd')
const CwPressureEnvironmentVisual = preload('res://scripts/world/cw_pressure_environment_visual.gd')
const MhPumpEnvironmentVisual = preload('res://scripts/world/mh_pump_environment_visual.gd')
const MhPumpChannelFlowVisual = preload('res://scripts/world/mh_pump_channel_flow_visual.gd')
const CW_PRESSURE_VALVES = {
	'cw_pressure_valve_visual_01':Vector3(8,0,7),
	'cw_pressure_valve_visual_02':Vector3(12,0,7),
	'cw_pressure_valve_visual_03':Vector3(16,0,7)
}
var cw_pressure_valve_visual
const CwFurnaceVisual = preload('res://scripts/world/cw_furnace_visual.gd')
const CwFurnaceHeartEnvironmentVisual = preload('res://scripts/world/cw_furnace_heart_environment_visual.gd')
const CwFurnaceHeartFloorMaterialVisual = preload('res://scripts/world/cw_furnace_heart_floor_material_visual.gd')
const CwShutdownExitEnvironmentVisual = preload('res://scripts/world/cw_shutdown_exit_environment_visual.gd')
const CwBoilerEnvironmentVisual = preload('res://scripts/world/cw_boiler_environment_visual.gd')
const CwRegulatorEnvironmentVisual = preload('res://scripts/world/cw_regulator_environment_visual.gd')
const CwForgedGuardArenaEnvironmentVisual = preload('res://scripts/world/cw_forged_guard_arena_environment_visual.gd')
const CwEntryFoundryEnvironmentVisual = preload('res://scripts/world/cw_entry_foundry_environment_visual.gd')
const CW_FURNACES = {
	'cw_arena_furnace':['cw_forged_guard_arena','furnace_v2','runtime2d.world.clockworks.furnace.v2','visual.props_back',Vector3(18,0,5)],
	'cw_regulator_furnace_v2':['cw_regulator_core','furnace_v2','runtime2d.world.clockworks.furnace.v2','visual.props_back',Vector3(16,0,6)],
	'cw_regulator_furnace_v2_side':['cw_regulator_core','furnace_v2','runtime2d.world.clockworks.furnace.v2','visual.props_dynamic',Vector3(12,0,12)],
	'cw_entry_furnace':['cw_entry_foundry','furnace_v2','runtime2d.world.clockworks.furnace.v2','visual.props_back',Vector3(19,0,6)],
	'cw_boiler_furnace':['cw_boiler_chamber','furnace_v2','runtime2d.world.clockworks.furnace.v2','visual.props_back',Vector3(16,0,6)],
	'cw_boiler_furnace_variant':['cw_boiler_chamber','furnace_v1','runtime2d.world.clockworks.furnace.v1','visual.props_dynamic',Vector3(18,0,10)],
	'cw_heart_furnace_v1':['cw_furnace_heart','furnace_v1','runtime2d.world.clockworks.furnace.v1','visual.props_back',Vector3(12,0,5)],
	'cw_heart_furnace_v2':['cw_furnace_heart','furnace_v2','runtime2d.world.clockworks.furnace.v2','visual.props_dynamic',Vector3(16,0,8)]
}
var cw_furnace_visual
var cw_furnace_heart_environment_visual = CwFurnaceHeartEnvironmentVisual.new()
var cw_furnace_heart_floor_material_visual = CwFurnaceHeartFloorMaterialVisual.new()
var cw_shutdown_exit_environment_visual = CwShutdownExitEnvironmentVisual.new()
var cw_entry_foundry_environment_visual = CwEntryFoundryEnvironmentVisual.new()
var cw_boiler_environment_visual = CwBoilerEnvironmentVisual.new()
var cw_regulator_environment_visual = CwRegulatorEnvironmentVisual.new()
var cw_forged_guard_arena_environment_visual = CwForgedGuardArenaEnvironmentVisual.new()
var camera = Camera.new()
var gh_current_geometry_cache: Dictionary = {}
var protected_actor_captions: Dictionary = {}
var rs_gate_atlas: Texture2D = null
var rs_gate_atlas_attempted = false
var rs_gh_gate_atlas: Texture2D = null
var rs_gh_gate_atlas_attempted = false
var aim_reticle: Dictionary = {}
var visual_movement_context: Dictionary = {}
var visual_movement_facing := Vector2.ZERO
var walk_binding: Dictionary = {}
var walk_intent := false
var walk_seconds := 0.0
var cenyao_east_walk = CenYaoEastWalk.new()
var mh_fog_pier_ground_visual = MhFogPierGroundVisual.new()
var mh_fog_pier_stone_material_visual = MhFogPierStoneMaterialVisual.new()
var mh_fog_pier_layered_fog_visual = MhFogPierLayeredFogVisual.new()
var mh_fog_pier_environment_visual = MhFogPierEnvironmentVisual.new()
var rs_environment_visual = RsEnvironmentVisual.new()
var baizhi_static_body_visual = BaizhiStaticBodyVisual.new()
var rs_source_owned_visual = RsSourceOwnedVisual.new()
var rs_source_plan: Dictionary = {}
var mh_cw_ambient_backdrop_visual = MhCwAmbientBackdropVisual.new()
var rs_bulkhead_modules_visual = RsBulkheadModulesVisual.new()
var rs_service_pipe_visual = RsServicePipeVisual.new()
var rs_exterior_backplate_visual = RsExteriorBackplateVisual.new()
var whole_scene_coverage_visual = WholeSceneCoverageVisual.new()
var gh_entry_exterior_backplate_visual = GhEntryExteriorBackplateVisual.new()
var gh_entry_r2_plate_candidate = GhEntryR2PlateCandidate.new()
var gh_deep_decon_exterior_backplate_visual = GhDeepDeconExteriorBackplateVisual.new()
var gh_beacon_exterior_backplate_visual = GhBeaconExteriorBackplateVisual.new()
var gh_bio_isolation_exterior_backplate_visual = GhBioIsolationExteriorBackplateVisual.new()
var gh_lockdown_exterior_backplate_visual = GhLockdownExteriorBackplateVisual.new()
var gh_power_exterior_backplate_visual = GhPowerExteriorBackplateVisual.new()
var gh_exit_exterior_backplate_visual = GhExitExteriorBackplateVisual.new()
var gh_gate_a_exterior_backplate_visual = GhGateAExteriorBackplateVisual.new()
var gh_gate_b_exterior_backplate_visual = GhGateBExteriorBackplateVisual.new()
var gh_central_shaft_exterior_backplate_visual = GhCentralShaftExteriorBackplateVisual.new()
var mh_resonance_tower_exterior_backplate_visual = MhResonanceTowerExteriorBackplateVisual.new()
var cw_regulator_core_exterior_backplate_visual = CwRegulatorCoreExteriorBackplateVisual.new()
var mh_signal_yard_exterior_backplate_visual = MhSignalYardExteriorBackplateVisual.new()
var cw_conveyor_bridge_exterior_backplate_visual = CwConveyorBridgeExteriorBackplateVisual.new()
var cw_forged_guard_arena_exterior_backplate_visual = CwForgedGuardArenaExteriorBackplateVisual.new()
var mh_warden_arena_exterior_backplate_visual = MhWardenArenaExteriorBackplateVisual.new()
var cw_boiler_chamber_exterior_backplate_visual = CwBoilerChamberExteriorBackplateVisual.new()
var cw_furnace_heart_exterior_backplate_visual = CwFurnaceHeartExteriorBackplateVisual.new()
var mh_fog_pier_exterior_backplate_visual = MhFogPierExteriorBackplateVisual.new()
var mh_fog_pier_exterior_r2_backplate_visual = MhFogPierExteriorR2BackplateVisual.new()
var cw_entry_foundry_exterior_backplate_visual = CwEntryFoundryExteriorBackplateVisual.new()
var cw_gear_shaft_backdrop_visual = CwGearShaftBackdropVisual.new()
var gh_bio_wall_modules_visual = GhBioWallModulesVisual.new()
var gh_bio_service_pipe_visual = GhBioServicePipeVisual.new()
var gh_bio_floor_material_visual = GhBioFloorMaterialVisual.new()
var gh_entry_floor_material_visual = GhEntryFloorMaterialVisual.new()
var rs_floor_treatment_visual = RsFloorTreatmentVisual.new()
var cw_pressure_floor_material_visual = CwPressureFloorMaterialVisual.new()
var gh_sentinel_floor_material_visual = GhSentinelFloorMaterialVisual.new()
var gh_sentinel_environment_visual = GhSentinelEnvironmentVisual.new()
var gh_shaft_dust_visual = GhShaftDustVisual.new()
var gh_deep_decon_floor_material_visual = GhDeepDeconFloorMaterialVisual.new()
var gh_deep_decon_environment_visual = GhDeepDeconEnvironmentVisual.new()
var gh_deep_decon_alcove_visual = GhDeepDeconAlcoveVisual.new()
var gh_gate_a_environment_visual = GhGateAEnvironmentVisual.new()
var gh_power_floor_material_visual = GhPowerFloorMaterialVisual.new()
var gh_power_gate_neighbor_visual = GhPowerGateNeighborVisual.new()
var gh_dense_service_wing_visual = GhDenseServiceWingVisual.new()
var gh_dense_region_composition = GhDenseRegionComposition.new()
var dense_visibility_context: Dictionary = {}
var dense_frame_ready: bool = false
var shaft_frame_context: Dictionary = {}
var shaft_pending_context: Dictionary = {}
var shaft_frame_at: int = 0
# Bounded by the authority's existing 250 ms held-input freshness limit.
const SHAFT_PENDING_MAX_MS: int = 250
var shaft_draw_failed: bool = false
var shaft_previous_region: String = ''
var shaft_previous_key: String = ''
var player_action_vfx_art_visual = PlayerActionVfxArtVisual.new()
var gh_entry_wall_modules_visual = GhEntryWallModulesVisual.new()
var gh_entry_service_pipe_visual = GhEntryServicePipeVisual.new()
var gh_lockdown_environment_visual = GhLockdownEnvironmentVisual.new()
var gh_entry_ambient_visual
var gh_bio_clutter_visual = GhBioClutterVisual.new()
var gh_sentinel_circle_warning_visual = GhSentinelCircleWarningVisual.new()
var gh_bio_sample_case_visual = GhBioSampleCaseVisual.new()
var gh_entry_clutter_visual = GhEntryClutterVisual.new()
var gh_power_environment_visual = GhPowerEnvironmentVisual.new()
var cw_pressure_environment_visual = CwPressureEnvironmentVisual.new()
var cw_conveyor_environment_visual = CwConveyorEnvironmentVisual.new()
var cw_conveyor_belt_dust_visual = CwConveyorBeltDustVisual.new()
var gh_gate_b_environment_visual = GhGateBEnvironmentVisual.new()
var gh_gate_b_west_cable_visual = GhGateBWestCableVisual.new()
var gh_deep_decon_air_visual = GhDeepDeconAirVisual.new()
var mh_pump_environment_visual = MhPumpEnvironmentVisual.new()
var mh_pump_channel_flow_visual = MhPumpChannelFlowVisual.new()
var rs_front_alpha: Dictionary = {}
var gh_bio_wall_alpha: Dictionary = {}
var gh_entry_wall_alpha: Dictionary = {}
var gh_entry_wall_draw_failed: bool = false
var cenyao_idle = CenYaoIdle.new()
var registered_props = RegisteredProps.new()
var mh_cw_actor
var grey_hive_actor
var gh_gate_b_brute_temporal_visual = GhGateBBruteTemporalVisual.new()
var mh_wraith_temporal_by_id: Dictionary = {}
var gh_maintenance_temporal_visual = GhMaintenanceTemporalVisual.new()
var gh_security_temporal_visual = GhSecurityTemporalVisual.new()
var grey_hive_state_props
var prop_bodies_drawn: Dictionary = {}
var gh_vital_body_records: Dictionary = {}
var snapshot: Dictionary = {}
var scene: Dictionary = {}
var scene_hash = ''
var events: Array = []
var sound: Dictionary = {}
var text_scale = 1.0
var color_assistance = false
var reduced_motion = false
var last_snapshot_at = 0
var action_key = ''
var action_at = 0
signal frame_committed(key)
var reported_key = ''
var current_key = ''
var _prepared_environment_route = ''
const ENVIRONMENT_PREPARE_ROUTES = ['grey_hive:gh_sentinel_arena','mist_harbor:mh_extraction','mist_harbor:mh_resonance_tower','mist_harbor:mh_breakwater','clockworks:cw_gear_shaft','mist_harbor:mh_drowned_quay','clockworks:cw_forged_guard_arena','grey_hive:gh_lockdown','mist_harbor:mh_signal_yard','grey_hive:gh_central_shaft','grey_hive:gh_exit']
const ENVIRONMENT_PREPARE_LATE = ['grey_hive:gh_central_shaft','grey_hive:gh_exit']
var _gh_shaft_dust_window: Window = null
var _gh_shaft_dust_committed_generation: int = -1
var _cw_conveyor_belt_dust_window: Window = null
var _cw_conveyor_belt_dust_generation: int = -1
var _mh_pump_flow_window: Window = null
var _mh_pump_flow_committed_generation: int = -1
var cw_gear_shaft_environment_visual = CwGearShaftEnvironmentVisual.new()
var mh_breakwater_environment_visual = MhBreakwaterEnvironmentVisual.new()
var mh_resonance_tower_environment_visual = MhResonanceTowerEnvironmentVisual.new()
var mh_extraction_environment_visual = MhExtractionEnvironmentVisual.new()
var displayed = {}
var action_body_frame: Dictionary = {}
var action_badges: Dictionary = {}
var action_badges_attempted: Dictionary = {}
var action_effects: Dictionary = {}
var action_effects_attempted: Dictionary = {}
var occluder_alpha = {}
var viewport_rect = Rect2()
var safe_rect_provider: Callable
var rs_camera_fit_provider: Callable
var interaction_focus: Dictionary = {}
var mh_fog_pier_phase_seconds: float = 0.0
var mh_fog_pier_defer_labels: bool = false
var mh_fog_pier_deferred_labels: Array = []
var mh_warden_arena_last_draw_report: Dictionary = {}
func _ready() -> void:
	gh_entry_ambient_visual = GhEntryAmbientVisual.new()
	baizhi_placeholder_visual = BaizhiPlaceholderVisual.new()
	cw_furnace_visual = CwFurnaceVisual.new()
	cw_furnace_visual.prepare_resources()
	cw_pressure_valve_visual = CwPressureValveVisual.new()
	gh_supply_crate_visual = GhSupplyCrateVisual.new()
	gh_supply_crate_visual.prepare_resources()
	grey_hive_state_props = GreyHiveStateProps.new()
	grey_hive_state_props.prepare_resources()
	gh_bio_lockdown_visual = GhBioLockdownVisual.new()
	mh_cw_actor = MhCwActor.new()
	grey_hive_actor = GreyHiveActor.new()
	mh_warden_arena_environment_visual.prepare()
	texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
	texture_repeat = CanvasItem.TEXTURE_REPEAT_DISABLED
func _enter_tree() -> void:
	var dust_window: Variant = get_window()
	if not dust_window is Window or not is_instance_valid(dust_window):
		return
	_gh_shaft_dust_window = dust_window
	_cw_conveyor_belt_dust_window = dust_window
	_mh_pump_flow_window = dust_window
	var callback: Callable = Callable(self,'_shaft_dust_window_changed')
	var conveyor_dust_callback: Callable = Callable(self,'_cw_conveyor_belt_dust_window_changed')
	if not dust_window.visibility_changed.is_connected(callback):
		dust_window.visibility_changed.connect(callback)
	if not dust_window.focus_exited.is_connected(callback):
		dust_window.focus_exited.connect(callback)
	if not dust_window.size_changed.is_connected(callback):
		dust_window.size_changed.connect(callback)
	if not dust_window.visibility_changed.is_connected(conveyor_dust_callback):
		dust_window.visibility_changed.connect(conveyor_dust_callback)
	if not dust_window.focus_exited.is_connected(conveyor_dust_callback):
		dust_window.focus_exited.connect(conveyor_dust_callback)
	if not dust_window.size_changed.is_connected(conveyor_dust_callback):
		dust_window.size_changed.connect(conveyor_dust_callback)
	var pump_flow_callback: Callable = Callable(self,'_mh_pump_flow_window_changed')
	if not dust_window.visibility_changed.is_connected(pump_flow_callback):
		dust_window.visibility_changed.connect(pump_flow_callback)
	if not dust_window.focus_exited.is_connected(pump_flow_callback):
		dust_window.focus_exited.connect(pump_flow_callback)
	if not dust_window.size_changed.is_connected(pump_flow_callback):
		dust_window.size_changed.connect(pump_flow_callback)
func _exit_tree() -> void:
	_gh_shaft_dust_committed_generation = -1
	_cw_conveyor_belt_dust_generation = -1
	_mh_pump_flow_committed_generation = -1
	var dust_window: Variant = _gh_shaft_dust_window
	if dust_window is Window and is_instance_valid(dust_window):
		var callback: Callable = Callable(self,'_shaft_dust_window_changed')
		if dust_window.visibility_changed.is_connected(callback):
			dust_window.visibility_changed.disconnect(callback)
		if dust_window.focus_exited.is_connected(callback):
			dust_window.focus_exited.disconnect(callback)
		if dust_window.size_changed.is_connected(callback):
			dust_window.size_changed.disconnect(callback)
		var conveyor_dust_callback: Callable = Callable(self,'_cw_conveyor_belt_dust_window_changed')
		if dust_window.visibility_changed.is_connected(conveyor_dust_callback):
			dust_window.visibility_changed.disconnect(conveyor_dust_callback)
		if dust_window.focus_exited.is_connected(conveyor_dust_callback):
			dust_window.focus_exited.disconnect(conveyor_dust_callback)
		if dust_window.size_changed.is_connected(conveyor_dust_callback):
			dust_window.size_changed.disconnect(conveyor_dust_callback)
		var pump_flow_callback: Callable = Callable(self,'_mh_pump_flow_window_changed')
		if dust_window.visibility_changed.is_connected(pump_flow_callback):
			dust_window.visibility_changed.disconnect(pump_flow_callback)
		if dust_window.focus_exited.is_connected(pump_flow_callback):
			dust_window.focus_exited.disconnect(pump_flow_callback)
		if dust_window.size_changed.is_connected(pump_flow_callback):
			dust_window.size_changed.disconnect(pump_flow_callback)
	_gh_shaft_dust_window = null
	_cw_conveyor_belt_dust_window = null
	_mh_pump_flow_window = null
func _notification(what: int) -> void:
	if what == NOTIFICATION_APPLICATION_FOCUS_OUT or what == NOTIFICATION_VISIBILITY_CHANGED:
		_shaft_dust_window_changed()
		_cw_conveyor_belt_dust_window_changed()
		_mh_pump_flow_window_changed()
func _shaft_dust_window_changed() -> void:
	if snapshot.get('worldId') == 'grey_hive' and snapshot.get('sceneId') == 'gh_central_shaft' and current_key == 'grey_hive:gh_central_shaft:'+str(snapshot.get('worldEpoch')):
		queue_redraw()
func _shaft_dust_host_matches(host: Variant) -> bool:
	if not host is Node or not is_instance_valid(host) or not host.is_inside_tree() or host.is_queued_for_deletion():
		return false
	var owner_script: Variant = host.get_script()
	if not owner_script is Script or owner_script.resource_path != 'res://scripts/ui/v1_main.gd' or host.get('renderer') != self or not host.has_method('_eligible'):
		return false
	var owner_snapshot: Variant = host.get('snapshot')
	if not owner_snapshot is Dictionary or not P.snapshot(owner_snapshot) or not P.same(owner_snapshot,snapshot):
		return false
	return owner_snapshot.get('serverTick') == snapshot.get('serverTick') and owner_snapshot.get('authorityRevision') == snapshot.get('authorityRevision') and owner_snapshot.player.entityId == snapshot.player.entityId
func _shaft_dust_gate(bind_generation: bool = false) -> Dictionary:
	if not P.snapshot(snapshot) or snapshot.get('entryToken') != null or snapshot.player.currentHp <= 0 or snapshot.get('worldId') != 'grey_hive' or snapshot.get('sceneId') != 'gh_central_shaft' or current_key != 'grey_hive:gh_central_shaft:'+str(snapshot.worldEpoch):
		return {}
	if not scene is Dictionary or scene.get('worldId') != 'grey_hive' or scene.get('sceneId') != 'gh_central_shaft' or scene_hash not in [GhShaftDustVisual.SCENE_SHA,GhShaftDustVisual.LEGACY_SCENE_SHA]:
		return {}
	var host: Variant = get_parent()
	if not _shaft_dust_host_matches(host):
		return {}
	var generation: Variant = host.get('generation')
	if typeof(generation) != TYPE_INT or generation <= 0:
		return {}
	if bind_generation:
		_gh_shaft_dust_committed_generation = generation
		return {}
	if generation != _gh_shaft_dust_committed_generation:
		return {}
	var eligible: Variant = host.call('_eligible')
	if typeof(eligible) != TYPE_BOOL or not eligible or host.get('generation') != _gh_shaft_dust_committed_generation or not _shaft_dust_host_matches(host):
		return {}
	var live_window: Variant = get_window()
	if not live_window is Window or not is_instance_valid(live_window) or live_window != _gh_shaft_dust_window:
		return {}
	return {'worldId':snapshot.worldId,'sceneId':snapshot.sceneId,'worldEpoch':snapshot.worldEpoch,'serverTick':snapshot.serverTick,'authorityRevision':snapshot.authorityRevision,'player_id':snapshot.player.entityId,'presentation_allowed':eligible,'visible':visible and is_visible_in_tree() and live_window.visible and live_window.has_focus(),'reduced_motion':reduced_motion}
func _cw_conveyor_belt_dust_window_changed() -> void:
	if snapshot.get('worldId')=='clockworks' and snapshot.get('sceneId')=='cw_conveyor_bridge' and current_key=='clockworks:cw_conveyor_bridge:'+str(snapshot.get('worldEpoch')):
		queue_redraw()
func _cw_conveyor_belt_dust_host_matches(host: Variant) -> bool:
	if not host is Node or not is_instance_valid(host) or not host.is_inside_tree() or host.is_queued_for_deletion():
		return false
	var owner_script: Variant = host.get_script()
	if not owner_script is Script or owner_script.resource_path!='res://scripts/ui/v1_main.gd' or host.get('renderer')!=self or not host.has_method('_eligible'):
		return false
	var owner_snapshot: Variant = host.get('snapshot')
	return owner_snapshot is Dictionary and P.snapshot(owner_snapshot) and P.same(owner_snapshot,snapshot) and owner_snapshot.get('serverTick')==snapshot.get('serverTick') and owner_snapshot.get('authorityRevision')==snapshot.get('authorityRevision') and owner_snapshot.player.entityId==snapshot.player.entityId
func _cw_conveyor_belt_dust_gate(bind_generation: bool = false) -> Dictionary:
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.player.currentHp<=0 or snapshot.get('worldId')!='clockworks' or snapshot.get('sceneId')!='cw_conveyor_bridge' or not P.counter(snapshot.get('worldEpoch'),1) or current_key!='clockworks:cw_conveyor_bridge:'+str(snapshot.worldEpoch) or not scene is Dictionary or scene.get('worldId')!='clockworks' or scene.get('sceneId')!='cw_conveyor_bridge' or scene_hash!=CwConveyorBeltDustVisual.SCENE_SHA:
		return {}
	var host: Variant = get_parent()
	if not _cw_conveyor_belt_dust_host_matches(host):
		return {}
	var generation: Variant = host.get('generation')
	if typeof(generation)!=TYPE_INT or generation<=0:
		return {}
	if bind_generation:
		_cw_conveyor_belt_dust_generation = generation
		return {}
	if generation!=_cw_conveyor_belt_dust_generation:
		return {}
	var eligible: Variant = host.call('_eligible')
	var live_window: Variant = get_window()
	var final_generation: Variant = host.get('generation')
	if typeof(eligible)!=TYPE_BOOL or not eligible or not _cw_conveyor_belt_dust_host_matches(host) or typeof(final_generation)!=TYPE_INT or final_generation!=_cw_conveyor_belt_dust_generation or not live_window is Window or not is_instance_valid(live_window) or live_window!=_cw_conveyor_belt_dust_window or typeof(reduced_motion)!=TYPE_BOOL:
		return {}
	return {'worldId':snapshot.worldId,'sceneId':snapshot.sceneId,'worldEpoch':snapshot.worldEpoch,'serverTick':snapshot.serverTick,'authorityRevision':snapshot.authorityRevision,'player_id':snapshot.player.entityId,'presentation_allowed':eligible,'visible':visible and is_visible_in_tree() and live_window.visible and live_window.has_focus(),'reduced_motion':reduced_motion}
func _mh_pump_flow_window_changed() -> void:
	if snapshot.get('worldId')=='mist_harbor' and snapshot.get('sceneId')=='mh_pump_station' and current_key=='mist_harbor:mh_pump_station:'+str(snapshot.get('worldEpoch')):
		queue_redraw()
func _mh_pump_flow_host_matches(host: Variant) -> bool:
	if not host is Node or not is_instance_valid(host) or not host.is_inside_tree() or host.is_queued_for_deletion():
		return false
	var owner_script: Variant = host.get_script()
	if not owner_script is Script or owner_script.resource_path!='res://scripts/ui/v1_main.gd' or host.get('renderer')!=self or not host.has_method('_eligible'):
		return false
	var owner_snapshot: Variant = host.get('snapshot')
	return owner_snapshot is Dictionary and P.snapshot(owner_snapshot) and P.same(owner_snapshot,snapshot) and owner_snapshot==snapshot and owner_snapshot.get('serverTick')==snapshot.get('serverTick') and owner_snapshot.get('authorityRevision')==snapshot.get('authorityRevision') and owner_snapshot.player.entityId==snapshot.player.entityId
func _mh_pump_flow_gate(bind_generation: bool = false) -> Dictionary:
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.player.currentHp<=0 or snapshot.get('worldId')!='mist_harbor' or snapshot.get('sceneId')!='mh_pump_station' or not P.counter(snapshot.get('worldEpoch'),1) or current_key!='mist_harbor:mh_pump_station:'+str(snapshot.worldEpoch):
		return {}
	if not scene is Dictionary or scene.get('worldId')!='mist_harbor' or scene.get('sceneId')!='mh_pump_station' or scene_hash!=MhPumpChannelFlowVisual.SCENE_SHA:
		return {}
	var pump_state: Variant = snapshot.get('mistHarborPump')
	if not pump_state is Dictionary or pump_state.size()!=1 or typeof(pump_state.get('state'))!=TYPE_STRING or pump_state.get('state') not in ['ready','draining']:
		return {}
	var host: Variant = get_parent()
	if not _mh_pump_flow_host_matches(host):
		return {}
	var generation: Variant = host.get('generation')
	if typeof(generation)!=TYPE_INT or generation<=0:
		return {}
	if bind_generation:
		_mh_pump_flow_committed_generation = generation
		return {}
	if generation!=_mh_pump_flow_committed_generation:
		return {}
	var eligible: Variant = host.call('_eligible')
	var final_generation: Variant = host.get('generation')
	var live_window: Variant = get_window()
	if typeof(eligible)!=TYPE_BOOL or not eligible or not _mh_pump_flow_host_matches(host) or typeof(final_generation)!=TYPE_INT or final_generation!=_mh_pump_flow_committed_generation or not live_window is Window or not is_instance_valid(live_window) or live_window!=_mh_pump_flow_window or typeof(reduced_motion)!=TYPE_BOOL:
		return {}
	return {'worldId':snapshot.worldId,'sceneId':snapshot.sceneId,'worldEpoch':snapshot.worldEpoch,'serverTick':snapshot.serverTick,'authorityRevision':snapshot.authorityRevision,'player_id':snapshot.player.entityId,'presentation_allowed':eligible,'visible':visible and is_visible_in_tree() and live_window.visible and live_window.has_focus(),'reduced_motion':reduced_motion}
func _prepare_environment_route(route: String) -> void:
	match route:
		'grey_hive:gh_sentinel_arena': gh_sentinel_environment_visual.prepare(snapshot,scene,current_key,scene_hash)
		'mist_harbor:mh_extraction': mh_extraction_environment_visual.prepare(snapshot,scene,current_key,scene_hash)
		'mist_harbor:mh_resonance_tower': mh_resonance_tower_environment_visual.prepare(snapshot,scene,current_key,scene_hash)
		'mist_harbor:mh_breakwater': mh_breakwater_environment_visual.prepare(snapshot,scene,current_key,scene_hash)
		'clockworks:cw_gear_shaft': cw_gear_shaft_environment_visual.prepare(snapshot,scene,current_key,scene_hash)
		'mist_harbor:mh_drowned_quay': mh_drowned_quay_environment_visual.prepare(snapshot,scene,current_key,scene_hash)
		'clockworks:cw_forged_guard_arena': cw_forged_guard_arena_environment_visual.prepare(snapshot,scene,current_key,scene_hash)
		'grey_hive:gh_lockdown': gh_lockdown_environment_visual.prepare(snapshot,scene,current_key,scene_hash)
		'mist_harbor:mh_signal_yard': mh_signal_yard_environment_visual.prepare(snapshot,scene,current_key)
		'grey_hive:gh_central_shaft': gh_central_shaft_environment_visual.prepare(snapshot,scene,current_key,scene_hash)
		'grey_hive:gh_exit': pass # Source-owned Exit does not invoke the old sidecar-bound renderer.
# Preserve camera coordinates only for a validated physical handoff at the shared plane.
func _rebase_power_gate_camera(next: Dictionary, next_hash: String) -> bool:
	if not camera.initialized or not P.snapshot(snapshot) or not P.snapshot(next): return false
	if snapshot.worldId!=next.worldId or snapshot.worldId not in ['grey_hive','mist_harbor','clockworks'] or int(next.worldEpoch)!=int(snapshot.worldEpoch)+1: return false
	if current_key!=snapshot.worldId+':'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch) or snapshot.player.entityId!=next.player.entityId: return false
	if not P.transform(snapshot.player.get('transform')) or not P.transform(next.player.get('transform')): return false
	var from_id: String = snapshot.sceneId
	var to_id: String = next.sceneId
	var route=snapshot.get('hiveRouteNeighbor')
	if route is Dictionary and route.get('ready')==true and route.get('sourceSceneId')==from_id and route.get('targetSceneId')==to_id and route.get('worldEpoch')==snapshot.worldEpoch and route.get('targetSceneSourceSha256')==next_hash and P.token(route.get('token')) and P.token(next.get('entryToken')) and next.entryToken.generation==route.token.generation+1 and next.entryToken.worldEpoch==next.worldEpoch:
		var handoff=next.get('hiveRouteHandoff')
		if not handoff is Dictionary or handoff.get('sourceSceneId')!=to_id or handoff.get('targetSceneId')!=from_id or handoff.get('worldEpoch')!=next.worldEpoch or handoff.get('sourceSceneSourceSha256')!=next_hash or handoff.get('targetSceneSourceSha256')!=scene_hash: return false
		var targets: Array=MhShoreRouteVisual.EDGES.get(from_id,[]) if snapshot.worldId=='mist_harbor' else ((CwLaterRouteVisual.LATER_EDGES.get(from_id,[]) if CwLaterRouteVisual.LATER_EDGES.has(from_id) and (from_id!='cw_boiler_chamber' or to_id=='cw_gear_shaft') else CwEarlyRouteVisual.EDGES.get(from_id,[])) if snapshot.worldId=='clockworks' else GhRouteNeighborVisual.EDGES.get(from_id,[]))
		if targets.size()!=2: return false
		var east: bool=to_id==targets[1] and not targets[1].is_empty()
		if not east and (to_id!=targets[0] or targets[0].is_empty()): return false
		var width: float=20.0 if from_id in ['gh_entry_maintenance','gh_power_room'] else 24.0
		var offset: float=width if east else -width
		camera.origin.x-=offset
		return true
	if snapshot.worldId!='grey_hive': return false
	var old_offset: Vector3
	var new_offset: Vector3
	var seam_x: float
	var seam_min_z: float = 6.0
	var seam_max_z: float = 8.0
	if (from_id=='gh_beacon' and to_id=='gh_exit') or (from_id=='gh_exit' and to_id=='gh_beacon'):
		var neighbor=snapshot.get('beaconExitNeighbor')
		if not neighbor is Dictionary or neighbor.get('ready')!=true or neighbor.get('sourceSceneId')!=from_id or neighbor.get('targetSceneId')!=to_id or neighbor.get('worldEpoch')!=snapshot.worldEpoch or neighbor.get('targetSceneSourceSha256')!=next.get('utilitySceneSourceSha256') or not P.token(neighbor.get('token')) or not P.token(next.get('entryToken')): return false
		if next.entryToken.generation!=neighbor.token.generation+1 or next.entryToken.worldEpoch!=next.worldEpoch: return false
		var previous: Vector3=P.vector(snapshot.player.transform.positionM)
		var following: Vector3=P.vector(next.player.transform.positionM)
		old_offset=Vector3.ZERO if from_id=='gh_beacon' else Vector3(24,0,0)
		new_offset=Vector3.ZERO if to_id=='gh_beacon' else Vector3(24,0,0)
		var a=previous+old_offset;var b=following+new_offset
		if previous.y!=0.0 or following.y!=0.0 or a.z<6.5 or a.z>9.5 or b.z<6.5 or b.z>9.5: return false
		if from_id=='gh_beacon' and not (a.x<=24.0 and b.x>24.0): return false
		if from_id=='gh_exit' and not (a.x>=24.0 and b.x<24.0): return false
		camera.origin+=old_offset-new_offset
		return true
	if from_id=='gh_power_room' and to_id=='gh_gate_a':
		old_offset = Vector3(0,0,0)
		new_offset = Vector3(20,0,0)
		seam_x = 20.0
	elif from_id=='gh_gate_a' and to_id=='gh_power_room':
		old_offset = Vector3(20,0,0)
		new_offset = Vector3(0,0,0)
		seam_x = 20.0
	elif from_id=='gh_gate_a' and to_id=='gh_central_shaft':
		old_offset = Vector3(20,0,0)
		new_offset = Vector3(40,0,-1)
		seam_x = 40.0
	elif from_id=='gh_central_shaft' and to_id=='gh_gate_a':
		old_offset = Vector3(40,0,-1)
		new_offset = Vector3(20,0,0)
		seam_x = 40.0
	elif from_id=='gh_gate_b' and to_id=='gh_deep_decon':
		old_offset = Vector3(0,0,0)
		new_offset = Vector3(24,0,0)
		seam_x = 24.0
		seam_min_z = 7.0
		seam_max_z = 9.0
	elif from_id=='gh_deep_decon' and to_id=='gh_gate_b':
		old_offset = Vector3(24,0,0)
		new_offset = Vector3(0,0,0)
		seam_x = 24.0
		seam_min_z = 7.0
		seam_max_z = 9.0
	elif from_id=='gh_central_shaft' and to_id=='gh_lockdown':
		old_offset = Vector3(40,0,-1)
		new_offset = Vector3(64,0,-1)
		seam_x = 64.0
	elif from_id=='gh_lockdown' and to_id=='gh_central_shaft':
		old_offset = Vector3(64,0,-1)
		new_offset = Vector3(40,0,-1)
		seam_x = 64.0
	else: return false
	var previous: Vector3 = P.vector(snapshot.player.transform.positionM)
	var following: Vector3 = P.vector(next.player.transform.positionM)
	var previous_global = previous+old_offset
	var following_global = following+new_offset
	# The tolerance is two actual 0.35 m player radii, not a spawn-based seam.
	if absf(previous_global.x-seam_x)>0.7 or absf(following_global.x-seam_x)>0.7 or previous_global.distance_to(following_global)>0.7: return false
	if previous_global.z<seam_min_z or previous_global.z>seam_max_z or following_global.z<seam_min_z or following_global.z>seam_max_z: return false
	if seam_x in [24.0,40.0,64.0] and (previous.y!=0.0 or following.y!=0.0): return false
	camera.origin += old_offset-new_offset
	return true
func commit(s: Dictionary, data: Dictionary, hash: String) -> void:
	var route_view=s.get('hiveRouteNeighbor')
	var hive_pending_key_value: Array=[s.get('worldId'),s.get('sceneId'),s.get('worldEpoch'),hash,route_view.get('token') if route_view is Dictionary else null,s.get('hiveRouteHold'),s.get('hiveRouteBoundary'),s.get('denseOccupiedRegion'),s.get('currentVisibleRegions'),Content.progress(s,str(s.get('worldId','')))]
	if hive_pending_key_value!=hive_route_pending_key:
		hive_route_pending_key=hive_pending_key_value.duplicate(true);hive_route_pending_at=Time.get_ticks_msec();hive_route_draw_failed=false
	var route=s.get('hiveRouteNeighbor')
	if route is Dictionary and route.get('ready')==true and hive_route_cache.get('token')==route.get('token') and hive_route_cache.get('source')==s.get('sceneId') and hive_route_cache.get('epoch')==s.get('worldEpoch') and hive_route_cache.get('sourceHash')==hash and hive_route_cache.get('targetHash')==route.get('targetSceneSourceSha256') and hive_route_cache.get('region')==s.get('denseOccupiedRegion') and hive_route_cache.get('visible')==s.get('currentVisibleRegions') and hive_route_cache.get('progress')==Content.progress(s,str(s.get('worldId',''))):
		hive_route_cache.authorized=true
	var pair=s.get('beaconExitNeighbor')
	if pair is Dictionary and pair.get('ready')==true and beacon_exit_cache.get('token')==pair.get('token') and beacon_exit_cache.get('source')==s.get('sceneId') and beacon_exit_cache.get('epoch')==s.get('worldEpoch') and beacon_exit_cache.get('sourceHash')==hash and beacon_exit_cache.get('targetHash')==pair.get('targetSceneSourceSha256') and beacon_exit_cache.get('progress')==Content.progress(s,'grey_hive') and beacon_exit_cache.get('beacon')==s.get('greyHiveBeacon'):
		beacon_exit_cache.authorized=true
	var previous_dense: Dictionary = dense_visibility_context.duplicate()
	dense_visibility_context.clear()
	clear_interaction_focus()
	var key = s.worldId+':'+s.sceneId+':'+str(s.worldEpoch)
	if key!=current_key:
		_cw_regular_guard_reset_all()
		gh_swarm_member_temporal_visual.reset()
		gh_sentinel_temporal_visual.reset()
		gh_gate_b_brute_temporal_visual.reset()
		gh_maintenance_temporal_visual.reset()
		gh_security_temporal_visual.reset()
		if not _rebase_power_gate_camera(s,hash): camera.reset()
		if gh_entry_ambient_visual!=null:
			gh_entry_ambient_visual.reset()
		_clear_gh_vital_body_records()
		events.clear()
		sound.clear()
		displayed.clear()
		occluder_alpha.clear()
		rs_front_alpha.clear()
		gh_bio_wall_alpha.clear()
		gh_entry_wall_alpha.clear()
		mh_fog_pier_phase_seconds = 0.0
		mh_fog_pier_defer_labels = false
		mh_fog_pier_deferred_labels.clear()
		current_key = key
	gh_current_geometry_cache.clear()
	snapshot = s
	scene = data
	scene_hash = hash
	_cw_regular_guard_reconcile()
	if _dense_visibility_scope() and previous_dense.get('world')==s.worldId and previous_dense.get('scene')==s.sceneId and previous_dense.get('epoch')==s.worldEpoch and previous_dense.get('player')==s.player.entityId and previous_dense.get('hash')==hash and previous_dense.get('entities')==s.get('denseEntityRegions') and previous_dense.get('escapes')==s.get('densePartitionEscapeIds'):
		var next_region: Dictionary = gh_dense_service_wing_visual.current_region(s)
		if not next_region.is_empty() and next_region.id==previous_dense.get('region'):
			dense_visibility_context = {'world':s.worldId,'scene':s.sceneId,'epoch':s.worldEpoch,'tick':s.serverTick,'revision':s.authorityRevision,'region':next_region.id,'player':s.player.entityId,'hash':hash,'entities':s.denseEntityRegions.duplicate(true),'escapes':s.densePartitionEscapeIds.duplicate(true)}
	if P.snapshot(snapshot) and snapshot.get('entryToken') == null and snapshot.get('worldId') == 'grey_hive' and snapshot.get('sceneId') == 'gh_gate_b' and P.counter(snapshot.get('worldEpoch'),1) and scene.get('worldId') == 'grey_hive' and scene.get('sceneId') == 'gh_gate_b' and current_key == 'grey_hive:gh_gate_b:'+str(snapshot.get('worldEpoch')):
		gh_gate_b_west_cable_visual.prepare_resources()
	_mh_pump_flow_committed_generation = -1
	_mh_pump_flow_gate(true)
	_gh_shaft_dust_committed_generation = -1
	_shaft_dust_gate(true)
	_cw_conveyor_belt_dust_generation = -1
	_cw_conveyor_belt_dust_gate(true)
	var route_key = str(snapshot.get('worldId',''))+':'+str(snapshot.get('sceneId',''))
	var next_route: String = route_key if route_key in ENVIRONMENT_PREPARE_ROUTES else ''
	if _prepared_environment_route!=next_route and not _prepared_environment_route.is_empty():
		_prepare_environment_route(_prepared_environment_route)
	_prepared_environment_route = next_route
	if not next_route.is_empty() and not next_route in ENVIRONMENT_PREPARE_LATE:
		_prepare_environment_route(next_route)
	if snapshot.get('worldId')=='clockworks' and snapshot.get('sceneId')=='cw_entry_foundry':
		cw_entry_foundry_environment_visual.prepare(snapshot,scene,current_key,scene_hash)
	if snapshot.get('worldId')=='mist_harbor' and snapshot.get('sceneId')=='mh_tidal_warehouse':
		mh_tidal_warehouse_environment_visual.prepare_resources(snapshot,scene,current_key)
		mh_warehouse_crates_visual.prepare_resources(self,snapshot,scene,current_key,camera,viewport_rect)
	if next_route in ENVIRONMENT_PREPARE_LATE:
		_prepare_environment_route(next_route)
	_prepare_shaft_pending()
	last_snapshot_at = Time.get_ticks_msec()
	var ap = s.player.get('actionPresentation')
	if ap is Dictionary:
		var akey = str([s.serverTick,ap.get('requestId'),ap.get('elapsedMs'),ap.get('phase')])
		if akey!=action_key:
			action_key = akey
			action_at = last_snapshot_at
	queue_redraw()
# Visual-only locomotion facing; combat direction remains authoritative.
func _walk_matches(s: Dictionary) -> bool:
	return P.snapshot(s) and P.same(walk_binding,s) and walk_binding.get('player_id')==s.player.entityId and walk_binding.get('tick')==s.serverTick and walk_binding.get('revision')==s.authorityRevision
func _walk_allowed(s: Dictionary) -> bool:
	return walk_intent and not reduced_motion and _walk_matches(s) and s.player.currentHp>0 and not s.player.actionState in ['primaryAttack','pulse','guard','pierce','dash']
func set_visual_movement(movement: Vector2, s: Dictionary, eligible: bool) -> void:
	swarm_temporal_animate = eligible and not reduced_motion and P.snapshot(s) and P.snapshot(snapshot) and P.same(s,snapshot) and s.serverTick==snapshot.serverTick and s.authorityRevision==snapshot.authorityRevision
	sentinel_temporal_animate = swarm_temporal_animate
	if not sentinel_temporal_animate: gh_sentinel_temporal_visual.suspend()
	if not swarm_temporal_animate: gh_swarm_member_temporal_visual.reset()
	var valid = P.snapshot(s) and eligible and is_finite(movement.x) and is_finite(movement.y) and movement.length_squared()>0.01
	var east = false
	if valid:
		var projected = Vector2(movement.x-movement.y,(movement.x+movement.y)*0.5)
		east = projected.length_squared()>0.0 and posmod(int(floor(atan2(projected.y,projected.x)/(PI/4.0)+0.5)),8)==0
	var continuous = valid and east and walk_intent and P.same(walk_binding,s) and walk_binding.get('player_id')==s.player.entityId
	if not continuous: walk_seconds = 0.0
	walk_intent = valid and east
	if valid:
		walk_binding = {'worldId':s.worldId,'sceneId':s.sceneId,'worldEpoch':s.worldEpoch,'player_id':s.player.entityId,'tick':s.serverTick,'revision':s.authorityRevision}
	else:
		walk_binding.clear()
	if not P.snapshot(s):
		visual_movement_context.clear()
		visual_movement_facing = Vector2.ZERO
		return
	if not P.same(visual_movement_context,s):
		visual_movement_context = {"worldId":s.worldId,"sceneId":s.sceneId,"worldEpoch":s.worldEpoch}
		visual_movement_facing = Vector2.ZERO
	if eligible and is_finite(movement.x) and is_finite(movement.y) and movement.length_squared()>0.01:
		visual_movement_facing = movement.normalized()

func set_aim_reticle(screen: Vector2, s: Dictionary, enabled: bool) -> void:
	aim_reticle.clear()
	if not enabled or not P.snapshot(s) or not P.snapshot(snapshot) or not P.same(s,snapshot) or s.serverTick!=snapshot.serverTick or s.authorityRevision!=snapshot.authorityRevision:
		return
	if s.get('entryToken')!=null or s.player.currentHp<=0 or not is_finite(screen.x) or not is_finite(screen.y):
		return
	if scene.get('worldId')!=s.worldId or scene.get('sceneId')!=s.sceneId or current_key!=s.worldId+':'+s.sceneId+':'+str(s.worldEpoch):
		return
	aim_reticle = {'screen':screen,'context':{'worldId':s.worldId,'sceneId':s.sceneId,'worldEpoch':s.worldEpoch},'tick':s.serverTick,'revision':s.authorityRevision}
func clear_interaction_focus() -> void:
	interaction_focus.clear()
	queue_redraw()
func set_interaction_focus(target: Dictionary, s: Dictionary, eligible: bool, expected_generation: int) -> void:
	clear_interaction_focus()
	if not eligible or target.is_empty() or not P.counter(expected_generation,1):
		return
	if not P.snapshot(s) or not P.snapshot(snapshot):
		return
	if not P.same(s,snapshot) or s.serverTick!=snapshot.serverTick or s.authorityRevision!=snapshot.authorityRevision:
		return
	if s.get('entryToken')!=null or snapshot.get('entryToken')!=null or s.player.currentHp<=0 or snapshot.player.currentHp<=0:
		return
	if scene.is_empty() or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or current_key!=snapshot.worldId+':'+snapshot.sceneId+':'+str(snapshot.worldEpoch):
		return
	if target.size()!=5 or not target.has('generation') or not target.has('context') or not target.has('entityId') or not target.has('kind') or not target.has('label'):
		return
	if not P.counter(target.get('generation'),1) or target.get('generation')!=expected_generation:
		return
	var target_context = target.get('context')
	if not P.context(target_context) or not P.same(target_context,snapshot):
		return
	if not P.ident(target.get('entityId')):
		return
	if typeof(target.get('kind'))!=TYPE_STRING or typeof(target.get('label'))!=TYPE_STRING:
		return
	var kind: String = target.kind
	var label: String = target.label
	if kind.is_empty() or label.is_empty():
		return
	interaction_focus = {'target':target.duplicate(true),'generation':expected_generation,'serverTick':snapshot.serverTick,'authorityRevision':snapshot.authorityRevision}
	queue_redraw()
func _process(delta: float) -> void:
	if _walk_allowed(snapshot) and is_finite(delta) and delta>0.0:
		walk_seconds = fmod(walk_seconds+delta,4.0/6.0)
	else:
		walk_seconds = 0.0
	if snapshot.is_empty() or scene.is_empty():
		mh_fog_pier_phase_seconds = 0.0
		return
	if P.snapshot(snapshot) and snapshot.get('worldId')=='mist_harbor' and snapshot.get('sceneId')=='mh_fog_pier' and scene.get('worldId')=='mist_harbor' and scene.get('sceneId')=='mh_fog_pier' and current_key=='mist_harbor:mh_fog_pier:'+str(snapshot.get('worldEpoch')):
		mh_fog_pier_phase_seconds = mh_fog_pier_layered_fog_visual.advance_visual_time(mh_fog_pier_phase_seconds,delta,reduced_motion)
	else:
		mh_fog_pier_phase_seconds = 0.0
	if P.snapshot(snapshot) and snapshot.get('worldId')=='grey_hive' and snapshot.get('sceneId')=='gh_entry_maintenance' and scene.get('worldId')=='grey_hive' and scene.get('sceneId')=='gh_entry_maintenance' and current_key=='grey_hive:gh_entry_maintenance:'+str(snapshot.get('worldEpoch')):
		gh_entry_ambient_visual.advance_visual(delta,reduced_motion)
	var full = get_viewport_rect()
	var safe_rect = full
	if snapshot.worldId=='return_station' and snapshot.sceneId=='rs_core_room' and rs_camera_fit_provider.is_valid():
		var candidate = rs_camera_fit_provider.call()
		if candidate is Rect2:
			if is_finite(candidate.position.x) and is_finite(candidate.position.y) and is_finite(candidate.size.x) and is_finite(candidate.size.y) and candidate.size.x>0 and candidate.size.y>0:
				var clipped = candidate.intersection(full)
				if clipped.size.x>0 and clipped.size.y>0:
					safe_rect = clipped
	camera.update(snapshot,scene,full.size,maxf(0,delta),safe_rect,scene_hash)
	viewport_rect = full
	exit_source_plan.clear()
	if _exit_source_scope() and snapshot.get('utilitySceneSourceSha256')==scene_hash:
		exit_source_plan=gh_exit_source_owned_visual.prepare(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,gh_exit_environment_visual,gh_bio_lockdown_visual,gh_supply_crate_visual)
		if not exit_source_plan.is_empty() and _pair_visibility_scope():
			var region=gh_beacon_exit_region_visual.prepare(snapshot,scene,scene_hash,camera,gh_shaft_lift_visual)
			if region.is_empty(): exit_source_plan.clear()
			else: exit_source_plan.merge(region,true)
		if not exit_source_plan.is_empty():
			exit_source_plan.merge({'key':current_key,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':snapshot.player.entityId,'camera_origin':camera.origin,'camera_ppm':camera.ppm,'camera_center':camera.screen_center})
	utility_plan.clear()
	if _utility_new_scope() and snapshot.get('utilitySceneSourceSha256')==scene_hash:
		utility_plan=gh_utility_layout_visual.prepare(scene,scene_hash,Callable(camera,'project'),gh_shaft_lift_visual)
		if not utility_plan.is_empty() and _decon_visibility_scope():
			utility_plan=gh_decon_region_visual.prepare(snapshot,scene,scene_hash,camera,gh_shaft_lift_visual)
		if not utility_plan.is_empty() and snapshot.sceneId=='gh_bio_isolation':
			var original: Dictionary = utility_plan.original_bio
			var packets=gh_bio_wall_modules_visual.build_packets(snapshot,original,current_key,camera)
			var shadows=gh_bio_wall_modules_visual.build_shadow_packets(original,camera)
			if packets.size()!=72 or shadows.size()!=6: utility_plan.clear()
			elif _bio_visibility_scope():
				var region=gh_bio_region_visual.prepare(snapshot,scene,scene_hash,camera,gh_shaft_lift_visual)
				var atlas=gh_bio_region_visual.prepare_atlas(region,packets,shadows,gh_bio_wall_modules_visual,camera) if not region.is_empty() else {}
				if region.is_empty() or atlas.is_empty(): utility_plan.clear()
				else:
					utility_plan.merge(region,true)
					utility_plan['bio_atlas']=atlas
		if not utility_plan.is_empty() and snapshot.sceneId=='gh_beacon' and not gh_beacon_environment_visual.whole_static_ready(self,snapshot,utility_plan.original_beacon,current_key,camera,get_viewport_rect()): utility_plan.clear()
		if not utility_plan.is_empty() and _pair_visibility_scope():
			var region=gh_beacon_exit_region_visual.prepare(snapshot,scene,scene_hash,camera,gh_shaft_lift_visual)
			if region.is_empty(): utility_plan.clear()
			else: utility_plan.merge(region,true)
		if not utility_plan.is_empty():
			utility_plan.merge({'key':current_key,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':snapshot.player.entityId,'camera_origin':camera.origin,'camera_ppm':camera.ppm,'camera_center':camera.screen_center})
	rs_source_plan = rs_source_owned_visual.prepare(snapshot,scene,current_key,camera,rs_environment_visual,_rs_terminal_feet()) if _rs_environment_context() else {}
	_update_rs_front_alpha(maxf(0,delta))
	_update_gh_bio_wall_alpha(maxf(0,delta))
	_update_gh_entry_wall_alpha(maxf(0,delta))
	_update_occlusion(delta)
	queue_redraw()
func visible_point(p: Vector3) -> bool:
	return viewport_rect.grow(4).has_point(camera.project(p))
func _dense_visibility_scope() -> bool:
	return snapshot.get('worldId')=='grey_hive' and snapshot.get('sceneId') in ['gh_power_room','gh_gate_a']

func _decon_visibility_scope() -> bool:
	return snapshot.get('worldId')=='grey_hive' and snapshot.get('sceneId')=='gh_deep_decon' and snapshot.get('utilitySceneSourceSha256')==GhDeconRegionVisual.SHA

func _bio_visibility_scope() -> bool:
	return snapshot.get('worldId')=='grey_hive' and snapshot.get('sceneId')=='gh_bio_isolation' and (snapshot.get('utilitySceneSourceSha256')==GhBioRegionVisual.SHA or scene_hash==GhBioRegionVisual.SHA)

func _pair_visibility_scope() -> bool:
	return snapshot.get('worldId')=='grey_hive' and ((snapshot.get('sceneId')=='gh_beacon' and snapshot.get('utilitySceneSourceSha256')==GhBeaconExitRegionVisual.BEACON) or (snapshot.get('sceneId')=='gh_exit' and snapshot.get('utilitySceneSourceSha256')==GhBeaconExitRegionVisual.EXIT))

func _floor_route_visibility_scope() -> bool:
	return gh_floor_route_region_visual.scope(snapshot,scene_hash)

func _floor_route_region_plan() -> Dictionary:
	if not _floor_route_visibility_scope(): return {}
	if snapshot.get('shaftLowerService') is Dictionary and typeof(snapshot.get('denseOccupiedRegion'))==TYPE_STRING and snapshot.denseOccupiedRegion.begins_with('grey_hive.gh_central_shaft.lower_service_v2.'):
		return gh_shaft_lower_service_visual.region_plan(snapshot,scene,scene_hash)
	var progress: Dictionary=Content.progress(snapshot,'grey_hive')
	if not progress.get('completedEvents') is Array: return {}
	return gh_floor_route_region_visual.prepare(snapshot,scene,scene_hash,progress.completedEvents)

func _shaft_lower_region_active() -> bool:
	var id=snapshot.get('denseOccupiedRegion')
	return snapshot.get('sceneId')=='gh_central_shaft' and typeof(id)==TYPE_STRING and (id.begins_with('grey_hive.gh_central_shaft.lower_info_v1.') or id.begins_with('grey_hive.gh_central_shaft.lower_service_v2.')) and _floor_route_visibility_scope() and is_dense_region_available()

func _region_visibility_scope() -> bool:
	return _dense_visibility_scope() or _decon_visibility_scope() or _bio_visibility_scope() or _pair_visibility_scope() or _floor_route_visibility_scope()

func is_dense_region_available() -> bool:
	if _floor_route_visibility_scope():
		var plan=_floor_route_region_plan()
		return not plan.is_empty() and gh_floor_route_region_visual.contains(plan,P.vector(snapshot.player.transform.positionM))
	if _bio_visibility_scope():
		return _utility_plan_current() and utility_plan.get('region')==snapshot.get('denseOccupiedRegion') and snapshot.get('currentVisibleRegions')==[utility_plan.get('region')] and gh_bio_region_visual.contains(utility_plan,P.vector(snapshot.player.transform.positionM))
	if _pair_visibility_scope():
		var plan: Dictionary=utility_plan if snapshot.sceneId=='gh_beacon' else exit_source_plan
		var current: bool=_utility_plan_current() if snapshot.sceneId=='gh_beacon' else _exit_source_current()
		return current and plan.get('region')==snapshot.get('denseOccupiedRegion') and snapshot.get('currentVisibleRegions')==[plan.get('region')] and gh_beacon_exit_region_visual.contains(plan,P.vector(snapshot.player.transform.positionM))
	if _decon_visibility_scope():
		return _utility_plan_current() and utility_plan.get('region')==snapshot.get('denseOccupiedRegion') and snapshot.get('currentVisibleRegions')==[utility_plan.get('region')] and gh_decon_region_visual.contains(utility_plan,P.vector(snapshot.player.transform.positionM))
	if not _dense_visibility_scope(): return true
	if not P.snapshot(snapshot): return false
	if dense_visibility_context.get('world')!=snapshot.worldId or dense_visibility_context.get('scene')!=snapshot.sceneId or dense_visibility_context.get('epoch')!=snapshot.worldEpoch or dense_visibility_context.get('tick')!=snapshot.serverTick or dense_visibility_context.get('revision')!=snapshot.authorityRevision or dense_visibility_context.get('player')!=snapshot.player.entityId or dense_visibility_context.get('hash')!=scene_hash or dense_visibility_context.get('entities')!=snapshot.get('denseEntityRegions') or dense_visibility_context.get('escapes')!=snapshot.get('densePartitionEscapeIds'): return false
	var region: Dictionary = gh_dense_service_wing_visual.current_region(snapshot)
	return not region.is_empty() and region.id==dense_visibility_context.get('region')

func is_dense_visible_position(position: Vector3) -> bool:
	if _floor_route_visibility_scope():
		var plan=_floor_route_region_plan()
		return not plan.is_empty() and gh_floor_route_region_visual.contains(plan,position)
	if _bio_visibility_scope():
		return is_dense_region_available() and gh_bio_region_visual.contains(utility_plan,position)
	if _pair_visibility_scope():
		var plan: Dictionary=utility_plan if snapshot.sceneId=='gh_beacon' else exit_source_plan
		return is_dense_region_available() and gh_beacon_exit_region_visual.contains(plan,position)
	if _decon_visibility_scope():
		return is_dense_region_available() and gh_decon_region_visual.contains(utility_plan,position)
	if not _dense_visibility_scope(): return true
	if not is_dense_region_available(): return false
	return gh_dense_service_wing_visual.same_region(snapshot,position)

# Entity presentation requires the same authoritative region as the player.
func _dense_entity_allowed(id: Variant, position: Vector3) -> bool:
	if not _region_visibility_scope(): return true
	if not is_dense_region_available() or not position.is_finite() or typeof(id)!=TYPE_STRING: return false
	var regions = snapshot.get('denseEntityRegions')
	return regions is Dictionary and regions.get(id)==snapshot.get('denseOccupiedRegion')

func _dense_object_allowed(o: Dictionary) -> bool:
	if _decon_visibility_scope() or _bio_visibility_scope() or _pair_visibility_scope() or _floor_route_visibility_scope():
		if _floor_route_visibility_scope() and o.get('kind') in ['gh_entry_wall_module','gh_shaft_piece']: return true
		if _floor_route_visibility_scope() and o.get('kind')=='gh_shaft_fixture': return is_dense_region_available() and o.get('data',{}).get('fixture',{}).get('region')==snapshot.get('denseOccupiedRegion')
		if _floor_route_visibility_scope() and snapshot.sceneId=='gh_sentinel_arena' and snapshot.get('denseOccupiedRegion')=='grey_hive.gh_sentinel_arena.info_v1.combat_floor' and is_dense_region_available() and o.get('kind')=='prop':
			if o.get('id')=='sentinel_arena_pylon_left' and o.get('p')==Vector3(8,0,5): return true
			if o.get('id')=='sentinel_arena_pylon_right' and o.get('p')==Vector3(17,0,5): return true
		if o.get('kind') in ['player','gh_utility_piece','gh_exit_source_piece','gh_bio_region_atlas']: return true
		if not o.get('p') is Vector3: return false
		if _pair_visibility_scope() and o.get('kind')=='beacon_body':
			if typeof(o.get('id'))!=TYPE_STRING or not o.id.begins_with('beacon-body:'): return false
			return _dense_entity_allowed(o.id.trim_prefix('beacon-body:'),o.p)
		var data=o.get('data',{})
		if o.get('kind') in ['actor','door','interaction','npc']:
			if not data is Dictionary: return false
			return _dense_entity_allowed(data.get('doorId') if o.kind=='door' else data.get('entityId'),o.p)
		return is_dense_visible_position(o.p)
	if o.get('kind') in ['player','gh_dense_partition','gh_dense_boundary']: return true
	if not o.get('p') is Vector3: return false
	var kind = o.get('kind')
	var data = o.get('data',{})
	if kind in ['actor','door','interaction','npc']:
		if not data is Dictionary: return false
		var id = data.get('doorId') if kind=='door' else data.get('entityId')
		return _dense_entity_allowed(id,o.p)
	if kind=='prop': return data is Dictionary and _gh_crate_selected(data) and is_dense_visible_position(o.p)
	if kind in ['gh_console_body','registered_terminal','gh_story_prop']: return is_dense_visible_position(o.p)
	return false

func _dense_visible_partition(part: Dictionary, plan: Dictionary) -> Dictionary:
	if plan.is_empty() or not part.get('pieces') is Array: return {}
	var pieces: Array = []
	for piece in part.pieces:
		if not piece is Dictionary or not piece.get('world') is Rect2: return {}
		if gh_dense_region_composition._inside(plan.local,piece.world): pieces.append(piece)
	if pieces.is_empty(): return {}
	var result: Dictionary = part.duplicate()
	result.pieces = pieces
	return result

func is_dense_visible_interaction(row: Dictionary) -> bool:
	if not _region_visibility_scope(): return true
	if not P.transform(row.get('transform')): return false
	var position: Vector3 = P.vector(row.transform.positionM)
	return _dense_entity_allowed(row.get('entityId'),position)

func is_baizhi_region_authorized() -> bool:
	if not _bio_visibility_scope(): return true
	if not is_dense_region_available() or not P.rows(snapshot.get('npcs'),8) or not P.rows(snapshot.get('interactables'),2048): return false
	var npcs=snapshot.npcs.filter(func(n): return n is Dictionary and n.get('entityId')=='gh_bz_whitezhi_v1' and n.get('entityType')=='npc.baizhi')
	var markers=snapshot.interactables.filter(func(i): return i is Dictionary and i.get('entityId')=='gh_bz_first_contact' and i.get('kind')=='npc_dialogue')
	if npcs.size()!=1 or markers.size()!=1 or not npcs[0].get('position') is Array or npcs[0].position.size()!=3 or not P.transform(markers[0].get('transform')): return false
	for value in npcs[0].position:
		if not P.number(value): return false
	var p=Vector3(npcs[0].position[0],npcs[0].position[1],npcs[0].position[2])
	return p.is_finite() and p==P.vector(markers[0].transform.positionM) and _dense_entity_allowed(npcs[0].entityId,p) and _dense_entity_allowed(markers[0].entityId,p)

func actor_visible(id: String, allow_dead: bool = false) -> bool:
	if id=='player' or id==snapshot.get('player',{}).get('entityId'):
		return not snapshot.is_empty() and visible_point(P.vector(snapshot.player.transform.positionM))
	for a in snapshot.get('actors',[]):
		if a.entityId==id and (a.active or allow_dead):
			if a.get('signalOnly',false)==true: return false
			if _region_visibility_scope() and (not P.transform(a.get('transform')) or not _dense_entity_allowed(a.get('entityId'),P.vector(a.transform.positionM))): return false
			if a.entityType=='enemy.mist_harbor.signal_wraith' and (V.signal_positions(a,snapshot).is_empty() or not a.signalPerception.precise):
				return false
			var positions = V.actor_positions(a,snapshot)
			if allow_dead and a.entityType=='enemy.grey_hive.swarm':
				positions = []
				for member in V.swarm_members(a):
					positions.append(member.positionM)
			for p in positions:
				if visible_point(P.vector(p)) and not _occluded(P.vector(p)):
					return true
	return false
# Bind the cue only to the unique canonical RS marker and its live native row.
func _rs_gh_gate_active() -> bool:
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or not _rs_environment_context() or not camera.initialized:
		return false
	var authored = scene.get('interactions')
	var live = snapshot.get('interactables')
	if not P.rows(authored,2048) or not P.unique(authored,'id') or not P.rows(live,2048) or not P.unique(live,'entityId'):
		return false
	var marker = authored.filter(func(row): return row.get('id')=='rs_world_gate_marker')
	var target = live.filter(func(row): return row.get('entityId')=='rs_world_gate_marker')
	if marker.size()!=1 or target.size()!=1 or marker[0].get('kind')!='world_gate_marker' or not _gh_authored_position(marker[0].get('position'),Vector3(20,0,8)):
		return false
	return target[0].get('kind')=='world_gate' and target[0].get('active')==true and P.transform(target[0].get('transform')) and P.vector(target[0].transform.positionM)==Vector3(20,0,8)

func _draw_rs_gh_gate_pad() -> void:
	if not _rs_gh_gate_active():
		return
	var replacement = _rs_gate_badge('rs_world_gate_marker')
	if not replacement.is_empty():
		var candidate = {'id':'rs_world_gate_marker','p':Vector3(20,0,8),'kind':'interaction','data':replacement.row}
		if not _object_culling_state(candidate,replacement.at).culled: return
	var pad = [[19.4,7.7],[19.7,7.4],[20.3,7.4],[20.6,7.7],[20.6,8.3],[20.3,8.6],[19.7,8.6],[19.4,8.3]]
	_poly(pad,0.01,Color(0.52,0.35,0.15,0.28),true)
	_poly(pad,0.01,Color(0.78,0.60,0.32,0.75),false)

func _rs_gate_badge(id: String) -> Dictionary:
	var roles = {'rs_world_gate_marker':Vector3(20,0,8),'rs_mh_world_gate_marker':Vector3(20,0,12),'rs_cw_world_gate_marker':Vector3(20,0,4)}
	if not roles.has(id) or not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or not _rs_environment_context() or not camera.initialized or not is_finite(camera.ppm) or camera.ppm<=0.0 or viewport_rect!=get_viewport_rect(): return {}
	var host = get_parent()
	if not is_instance_valid(host) or host.get('renderer')!=self or host.get('snapshot')!=snapshot or not host.get('exit_entry_preview') is Dictionary or not host.exit_entry_preview.is_empty(): return {}
	var authored = scene.get('interactions')
	var live = snapshot.get('interactables')
	if not P.rows(authored,2048) or not P.unique(authored,'id') or not P.rows(live,2048) or not P.unique(live,'entityId'): return {}
	var marker = authored.filter(func(row): return row.get('id')==id)
	var target = live.filter(func(row): return row.get('entityId')==id)
	if marker.size()!=1 or target.size()!=1 or marker[0].get('kind')!='world_gate_marker' or not _gh_authored_position(marker[0].get('position'),roles[id]): return {}
	var row: Dictionary = target[0]
	if row.get('kind')!='world_gate' or row.get('active')!=true or not P.transform(row.get('transform')) or P.vector(row.transform.positionM)!=roles[id]: return {}
	if not rs_gate_atlas_attempted:
		rs_gate_atlas_attempted = true
		var path = 'res://assets/ui/map/map_region_ui_local_r1.png'
		if ResourceLoader.exists(path):
			var candidate = load(path)
			if candidate is Texture2D and candidate.get_width()==1254 and candidate.get_height()==1254:
				var image: Image = candidate.get_image()
				if image!=null and image.get_format()==Image.FORMAT_RGBA8: rs_gate_atlas = candidate
	var texture: Texture2D=rs_gate_atlas
	if id=='rs_world_gate_marker':
		if not rs_gh_gate_atlas_attempted:
			rs_gh_gate_atlas_attempted=true
			rs_gh_gate_atlas=rs_gate_atlas
			if not is_instance_valid(rs_gh_gate_atlas):
				var raw=Image.new()
				if raw.load('res://assets/ui/map/map_region_ui_local_r1.png')==OK and raw.get_width()==1254 and raw.get_height()==1254 and raw.get_format()==Image.FORMAT_RGBA8:
					rs_gh_gate_atlas=ImageTexture.create_from_image(raw)
		texture=rs_gh_gate_atlas
	if not is_instance_valid(texture): return {}
	var at: Vector2 = camera.project(roles[id])
	var side: float = (36.0 if id=='rs_world_gate_marker' else 20.0)*camera.ppm/48.0
	if not at.is_finite() or not is_finite(side) or side<=0.0: return {}
	var rect = Rect2(at-Vector2(side*0.5,side),Vector2(side,side))
	if not _rect_intersects(viewport_rect,rect): return {}
	return {'row':row,'at':at,'rect':rect,'texture':texture}

# Validate both original static groups before whole-scene reuse.
func _whole_power_ready() -> bool:
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_power_room': return false
	if not gh_power_floor_material_visual._valid_rect(viewport_rect): return false
	var floor_packet = gh_power_floor_material_visual.build_packet(snapshot,scene,current_key,camera)
	if floor_packet.is_empty() or not gh_power_floor_material_visual.is_packet_valid(floor_packet,snapshot,scene,current_key,camera): return false
	if not gh_power_floor_material_visual._load_texture_once(): return false
	for light in ['unknown','on','off']:
		if gh_power_environment_visual.build_packets(scene,snapshot,current_key,camera,light).is_empty(): return false
	return true
func _whole_bio_ready() -> bool:
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_bio_isolation': return false
	if not gh_bio_floor_material_visual._valid_rect(viewport_rect): return false
	var floor_packets = gh_bio_floor_material_visual.build_packets(snapshot,scene,current_key,camera,gh_bio_wall_modules_visual)
	if floor_packets.size()!=96 or gh_bio_floor_material_visual.floor_texture==null: return false
	var seen: Dictionary = {}
	for packet in floor_packets:
		if not gh_bio_floor_material_visual._valid_packet(packet,seen): return false
	if seen.size()!=96: return false
	var walls = gh_bio_wall_modules_visual.build_packets(snapshot,scene,current_key,camera)
	if walls.size()!=72: return false
	var shadows = gh_bio_wall_modules_visual.build_shadow_packets(scene,camera)
	return shadows.size()==6

func _whole_gate_a_ready() -> bool:
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null: return false
	var floor_packet = gh_power_floor_material_visual.build_packet(snapshot,scene,current_key,camera)
	if floor_packet.is_empty() or not gh_power_floor_material_visual.is_packet_valid(floor_packet,snapshot,scene,current_key,camera): return false
	if not gh_power_floor_material_visual._load_texture_once(): return false
	return gh_gate_a_environment_visual.whole_static_ready(self,snapshot,scene,current_key,camera,viewport_rect)

func _gh_entry_composition_ready() -> bool:
	if not P.snapshot(snapshot) or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_entry_maintenance': return true
	var floor_packets: Array=gh_entry_floor_material_visual.build_packets(snapshot,scene,current_key,camera)
	if floor_packets.size()!=70: return false
	var walls: Array=gh_entry_wall_modules_visual.build_packets(snapshot,scene,current_key,camera)
	if walls.size()!=63: return false
	var shadows: Array=gh_entry_wall_modules_visual.build_shadow_packets(snapshot,scene,current_key,camera,walls)
	return shadows.size()==5 and gh_entry_wall_modules_visual.suppressible_collision_ids(snapshot,scene,current_key,camera,walls,shadows,viewport_rect).size()==5

func _hive_full_scope() -> bool:
	return P.snapshot(snapshot) and (snapshot.worldId=='clockworks' and (CwEarlyRouteVisual.EDGES.has(snapshot.sceneId) or CwLaterRouteVisual.LATER_EDGES.has(snapshot.sceneId)) or snapshot.worldId=='mist_harbor' and MhShoreRouteVisual.EDGES.has(snapshot.sceneId) or snapshot.worldId=='grey_hive' and (snapshot.sceneId=='gh_sentinel_arena' or (snapshot.sceneId in ['gh_bio_isolation','gh_beacon'] and not gh_utility_layout_visual.is_new(str(snapshot.sceneId),scene_hash))))

func _cw_early_boiler_cues() -> bool:
	if not cw_boiler_environment_visual._valid_canvas(self) or not cw_boiler_environment_visual._valid_context(snapshot,scene,current_key,camera,viewport_rect): return false
	var plans: Array[Dictionary]=[]
	for key_value in cw_boiler_environment_visual.FURNACES:
		var key: String=str(key_value)
		if not cw_boiler_environment_visual._texture_ready(cw_furnace_visual,key): continue
		var position=cw_boiler_environment_visual.FURNACES.get(key)
		if not position is Array or position.size()!=3: return false
		var foot=Vector3(float(position[0]),float(position[1]),float(position[2]))
		if not foot.is_finite() or not cw_boiler_environment_visual._append_furnace_ground(plans,camera,foot): return false
	for plan in plans:
		if not cw_boiler_environment_visual._plan_valid(plan): return false
	for plan in plans: cw_boiler_environment_visual._emit_plan(self,plan)
	return true

func _cw_early_pressure_ready() -> bool:
	if snapshot.get('worldId')!='clockworks' or snapshot.get('sceneId')!='cw_pressure_hall': return true
	# The old Pressure image staging explicitly excludes pending entries. The
	# pending target uses its exact geometry/steel plan until that entry commits.
	if snapshot.get('entryToken')!=null: return true
	var floor_packet=cw_pressure_floor_material_visual.build_packet(snapshot,scene,current_key,camera)
	var source: Array=cw_pressure_environment_visual.build_packets(scene,snapshot,current_key,camera)
	if floor_packet.is_empty() or source.is_empty() or cw_pressure_floor_material_visual._load_texture()==null or not gh_shaft_lift_visual._load_once(): return false
	for owner in ['cw_pressure_pipe_run_north','cw_pressure_pipe_run_south']:
		for suffix in ['upper','lower','cross_a','cross_b','center']:
			var pieces=source.filter(func(p): return p is Dictionary and p.get('id')==owner+':'+suffix)
			if pieces.size()!=1 or pieces[0].get('role')!='pipe_detail' or pieces[0].get('source_id')!=owner or not pieces[0].get('points') is PackedVector2Array or pieces[0].points.size()!=2 or not P.number(pieces[0].get('width'),0.001,1000): return false
			if not pieces[0].points[0].is_finite() or not pieces[0].points[1].is_finite() or pieces[0].points[0].distance_squared_to(pieces[0].points[1])<=0.000001: return false
	var stage: Dictionary=cw_pressure_environment_visual.stage_short_union(source,self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	if stage.is_empty() or not cw_pressure_environment_visual.short_union_current(stage,source,self,snapshot,scene,current_key,scene_hash,camera,viewport_rect): return false
	var caps=cw_pressure_environment_visual.prepare_cap_images(source,scene,snapshot,current_key,camera,cw_pressure_floor_material_visual)
	var low=cw_pressure_environment_visual.prepare_low_face_images(stage.packets,scene,snapshot,current_key,camera)
	var ends=cw_pressure_environment_visual.prepare_end_face_images(stage.packets,scene,snapshot,current_key,camera,viewport_rect)
	if caps.is_empty() or low.is_empty() or ends.is_empty(): return false
	var outer=cw_pressure_environment_visual.prepare_outer_z_images(stage.packets,scene,snapshot,current_key,camera,viewport_rect)
	for id in ['cw_pressure_hall_wall_east_north:exterior_face','cw_pressure_hall_wall_east_south:exterior_face']:
		var packets=stage.packets.filter(func(p): return p is Dictionary and p.get('id')==id)
		if packets.size()>1: return false
		if packets.size()==1 and cw_pressure_environment_visual._short_visible(packets[0].points,viewport_rect)>0 and (outer.is_empty() or not outer.groups.has(id)): return false
	return true

func _hive_full_plan() -> Dictionary:
	if not _hive_full_scope(): return {}
	if snapshot.worldId=='clockworks':
		if CwLaterRouteVisual.LATER_EDGES.has(snapshot.sceneId) and snapshot.sceneId!='cw_boiler_chamber': return cw_later_route_visual.later_current(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,cw_pressure_floor_material_visual,gh_route_neighbor_visual,cw_gear_shaft_environment_visual)
		if not _cw_early_pressure_ready(): return {}
		return cw_early_route_visual.current(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,cw_pressure_floor_material_visual,gh_route_neighbor_visual)
	if snapshot.worldId=='mist_harbor': return mh_shore_route_visual.current(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,mh_fog_pier_stone_material_visual,gh_route_neighbor_visual)
	var progress: Dictionary=Content.progress(snapshot,'grey_hive')
	if progress.is_empty() or not progress.get('completedEvents') is Array: return {}
	return _gh_current_geometry(progress.completedEvents)

func _gh_current_geometry(events: Array) -> Dictionary:
	# This memo is limited to the current GH room and one engine process frame.
	# Re-run the source/catalog/material gates before considering a positive hit.
	var valid_camera = is_instance_valid(camera) and camera.initialized and camera.origin.is_finite() and camera.screen_center.is_finite() and is_finite(camera.ppm) and camera.ppm>0.0
	if not valid_camera:
		gh_current_geometry_cache.clear()
		return gh_route_neighbor_visual.current(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,events)
	var source = gh_route_neighbor_visual._catalog(snapshot.sceneId,scene_hash)
	var bounds = scene.get('boundsM')
	var admitted = source!=null and source.source_hash==scene_hash and source.current==scene and gh_shaft_lift_visual._load_once() and bounds is Dictionary and bounds.get('x')==0 and bounds.get('z')==0 and bounds.get('width')==24 and bounds.get('depth')==16
	if not admitted:
		gh_current_geometry_cache.clear()
		return gh_route_neighbor_visual.current(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,events)
	var frame = Engine.get_process_frames()
	var c = gh_current_geometry_cache
	if not c.is_empty() and c.frame==frame and c.key==current_key and c.hash==scene_hash and c.camera==camera and c.origin==camera.origin and c.center==camera.screen_center and c.ppm==camera.ppm and c.snapshot==snapshot and c.scene==scene and c.events==events and c.material==gh_shaft_lift_visual._data and c.floor==gh_shaft_lift_visual._floor and c.steel==gh_shaft_lift_visual._steel:
		return c.plan
	gh_current_geometry_cache.clear()
	var plan = gh_route_neighbor_visual.current(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,events)
	if not plan.is_empty():
		gh_current_geometry_cache = {'frame':frame,'key':current_key,'hash':scene_hash,'camera':camera,'origin':camera.origin,'center':camera.screen_center,'ppm':camera.ppm,'snapshot':snapshot.duplicate(true),'scene':scene.duplicate(true),'events':events.duplicate(true),'material':gh_shaft_lift_visual._data.duplicate(true),'floor':gh_shaft_lift_visual._floor,'steel':gh_shaft_lift_visual._steel,'plan':plan}
	return plan

func dense_region_ready() -> bool:
	if not _gh_entry_composition_ready(): return false
	if _floor_route_visibility_scope() and gh_floor_route_region_visual.requires_floor_region(snapshot) and not is_dense_region_available(): return false
	if _hive_full_scope() and _hive_full_plan().is_empty(): return false
	if _rs_environment_context() and not _rs_source_plan_current(): return false
	if not P.snapshot(snapshot): return false
	if snapshot.get('hiveRouteHoldMissing')==true: return false
	if snapshot.get('hiveRouteHold') is Dictionary and not _hive_route_hold_ready(): return false
	var route=snapshot.get('hiveRouteNeighbor')
	if route is Dictionary and route.get('ready')==true and not hive_route_preview_ready(): return false
	var boundary=snapshot.get('hiveRouteBoundary')
	if boundary is Dictionary and boundary.get('closed')==true and not _hive_route_boundary_ready(): return false
	if snapshot.get('beaconExitHoldMissing')==true: return false
	if snapshot.get('beaconExitHold') is Dictionary and not _beacon_exit_hold_ready(): return false
	if snapshot.get('beaconExitNeighbor') is Dictionary and (snapshot.beaconExitNeighbor.get('ready')==true or snapshot.beaconExitNeighbor.get('straddling')==true) and not beacon_exit_preview_ready():
		if snapshot.beaconExitNeighbor.get('ready')==true or not snapshot.get('beaconExitHold') is Dictionary or not _beacon_exit_hold_ready(): return false
	if _exit_source_scope() and (not _exit_source_compatible() or not _exit_source_frame_current()): return false
	if _utility_new_scope() and (not _utility_plan_compatible() or not _utility_frame_current()): return false
	if snapshot.worldId=='grey_hive' and snapshot.sceneId in ['gh_central_shaft','gh_lockdown']:
		var closed=_shaft_lockdown_gate_closed()
		if closed==null: return false
		if closed and (_shaft_lockdown_seal_plan().is_empty() or not _shaft_lockdown_seal_frame_current()): return false
	if _shaft_lockdown_portal_required() and not _shaft_lockdown_neighbor_ready(): return false
	if _lockdown_context() and (_lockdown_plan().is_empty() or not _lockdown_frame_current()): return false
	if _gate_decon_context():
		if _gate_decon_plan().is_empty() or not _gate_decon_frame_current(): return false
		if _gate_decon_portal_required() and (gate_decon_frame.get('portal')!=true or gh_gate_decon_visual.preview(snapshot,scene,current_key,camera,gh_shaft_lift_visual,gh_utility_layout_visual).is_empty()): return false
	if _gate_shaft_portal_required():
		if _gate_shaft_portal_plan().is_empty() or not _gate_shaft_portal_frame_current(): return false
	if shaft_visual_expected() or shaft_legacy_deferred():
		if _shaft_context_matches(shaft_frame_context): return true
		return _shaft_context_matches(shaft_pending_context) and Time.get_ticks_msec()-shaft_frame_at<=SHAFT_PENDING_MAX_MS
	if snapshot.get('worldId')!='grey_hive' or not snapshot.get('sceneId') in ['gh_power_room','gh_gate_a']: return true
	return dense_frame_ready and is_dense_region_available()

func _hive_route_held_plan() -> Dictionary:
	var h=snapshot.get('hiveRouteHold')
	if not h is Dictionary or h.size()!=6 or hive_route_cache.get('authorized')!=true or h.get('sourceSceneId')!=snapshot.sceneId or h.get('worldEpoch')!=snapshot.worldEpoch or h.get('sourceSceneSourceSha256')!=scene_hash or hive_route_cache.get('player')!=snapshot.player.entityId: return {}
	var old_plan: Dictionary={};var shift=Vector3.ZERO
	if hive_route_cache.get('source')==h.sourceSceneId and hive_route_cache.get('target')==h.targetSceneId and hive_route_cache.get('epoch')==h.worldEpoch and hive_route_cache.get('sourceHash')==h.sourceSceneSourceSha256 and hive_route_cache.get('targetHash')==h.targetSceneSourceSha256:
		if hive_route_cache.get('region')!=snapshot.get('denseOccupiedRegion') or hive_route_cache.get('visible')!=snapshot.get('currentVisibleRegions'): return {}
		old_plan=hive_route_cache.get('targetPlan',{})
	elif hive_route_cache.get('source')==h.targetSceneId and hive_route_cache.get('target')==h.sourceSceneId and int(hive_route_cache.get('epoch',-1))+1==h.worldEpoch and hive_route_cache.get('sourceHash')==h.targetSceneSourceSha256 and hive_route_cache.get('targetHash')==h.sourceSceneSourceSha256 and hive_route_cache.get('offset') is Vector3:
		old_plan=hive_route_cache.get('sourcePlan',{});shift=-hive_route_cache.offset
	else: return {}
	var plan=gh_route_neighbor_visual.reproject(old_plan,hive_route_cache.get('origin',Vector3.INF),hive_route_cache.get('center',Vector2.INF),float(hive_route_cache.get('ppm',0.0)),camera,shift)
	if plan.is_empty(): return {}
	plan['held']=h.duplicate(true)
	return plan

func _hive_route_source_regions() -> Array:
	if _dense_visibility_scope():
		var region: Dictionary=gh_dense_service_wing_visual.current_region(snapshot)
		if region.is_empty() or not region.get('rect') is Rect2: return []
		var rect: Rect2=region.rect
		if snapshot.sceneId=='gh_gate_a': rect.position.x-=20.0
		return [rect]
	if _decon_visibility_scope() or _bio_visibility_scope() or _pair_visibility_scope():
		if not _utility_plan_current() or utility_plan.get('region')!=snapshot.get('denseOccupiedRegion') or snapshot.get('currentVisibleRegions')!=[utility_plan.get('region')] or not utility_plan.get('parts') is Array: return []
		return utility_plan.parts.duplicate()
	var b=scene.get('boundsM')
	if not b is Dictionary or not P.number(b.get('x')) or not P.number(b.get('z')) or not P.number(b.get('width')) or not P.number(b.get('depth')) or b.width<=0 or b.depth<=0: return []
	return [Rect2(float(b.x),float(b.z),float(b.width),float(b.depth))]

func _hive_route_plan() -> Dictionary:
	if not P.snapshot(snapshot): return {}
	if snapshot.worldId=='clockworks':
		var neighbor=snapshot.get('hiveRouteNeighbor',{})
		if not neighbor is Dictionary: neighbor={}
		var later=CwLaterRouteVisual.LATER_EDGES.has(snapshot.sceneId) and (snapshot.sceneId!='cw_boiler_chamber' or neighbor.get('targetSceneId')=='cw_gear_shaft')
		var fresh=cw_later_route_visual.later_prepare(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,cw_pressure_floor_material_visual,gh_route_neighbor_visual,cw_gear_shaft_environment_visual) if later else cw_early_route_visual.prepare(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,cw_pressure_floor_material_visual,gh_route_neighbor_visual)
		if not fresh.is_empty() and _cw_early_pressure_ready(): return fresh
		return _hive_route_held_plan()
	if snapshot.worldId=='mist_harbor':
		var fresh=mh_shore_route_visual.prepare(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,mh_fog_pier_stone_material_visual,gh_route_neighbor_visual)
		if not fresh.is_empty(): return fresh
		return _hive_route_held_plan()
	var progress: Dictionary=Content.progress(snapshot,'grey_hive')
	if not progress.is_empty() and progress.get('completedEvents') is Array:
		var fresh=gh_route_neighbor_visual.prepare(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,progress.completedEvents,_hive_route_source_regions())
		if not fresh.is_empty(): return fresh
	return _hive_route_held_plan()

func hive_route_preview_ready() -> bool:
	var v=snapshot.get('hiveRouteNeighbor')
	if not v is Dictionary or hive_route_frame.get('arenaState')!=v.get('arenaShuttersOpen') or hive_route_frame.get('token')!=v.get('token') or hive_route_frame.get('hash')!=v.get('targetSceneSourceSha256') or hive_route_frame.get('sourceHash')!=scene_hash or hive_route_frame.get('epoch')!=snapshot.worldEpoch or hive_route_frame.get('player')!=snapshot.player.entityId or hive_route_frame.get('region')!=snapshot.get('denseOccupiedRegion') or hive_route_frame.get('visible')!=snapshot.get('currentVisibleRegions') or hive_route_frame.get('progress')!=Content.progress(snapshot,snapshot.worldId): return false
	return Time.get_ticks_msec()-hive_route_frame_at<=SHAFT_PENDING_MAX_MS

func _hive_route_hold_ready() -> bool:
	var h=snapshot.get('hiveRouteHold')
	if not h is Dictionary or hive_route_frame.get('epoch')!=snapshot.worldEpoch or hive_route_frame.get('player')!=snapshot.player.entityId or hive_route_frame.get('region')!=snapshot.get('denseOccupiedRegion') or hive_route_frame.get('visible')!=snapshot.get('currentVisibleRegions') or Time.get_ticks_msec()-hive_route_frame_at>SHAFT_PENDING_MAX_MS: return false
	if hive_route_frame.get('held')==h: return true
	return hive_route_cache.get('source')==h.get('sourceSceneId') and hive_route_cache.get('epoch')==h.get('worldEpoch') and hive_route_frame.get('token')==hive_route_cache.get('token') and not _hive_route_held_plan().is_empty()

func _hive_route_boundary_ready() -> bool:
	return hive_route_frame.get('boundary')==snapshot.get('hiveRouteBoundary') and hive_route_frame.get('sourceHash')==scene_hash and hive_route_frame.get('epoch')==snapshot.worldEpoch and hive_route_frame.get('player')==snapshot.player.entityId and Time.get_ticks_msec()-hive_route_frame_at<=SHAFT_PENDING_MAX_MS

func hive_route_frame_pending() -> bool:
	if hive_route_draw_failed or hive_route_pending_at<=0 or Time.get_ticks_msec()-hive_route_pending_at>SHAFT_PENDING_MAX_MS: return false
	var boundary=snapshot.get('hiveRouteBoundary')
	if boundary is Dictionary and boundary.get('closed')==true:
		var plan=mh_shore_route_visual.boundary(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,mh_fog_pier_stone_material_visual) if snapshot.worldId=='mist_harbor' else ((cw_later_route_visual.later_boundary(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,cw_pressure_floor_material_visual) if CwLaterRouteVisual.LATER_EDGES.has(snapshot.sceneId) and (snapshot.sceneId!='cw_boiler_chamber' or snapshot.get('hiveRouteBoundary',{}).get('targetSceneId')=='cw_gear_shaft') else cw_early_route_visual.boundary(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,cw_pressure_floor_material_visual)) if snapshot.worldId=='clockworks' else gh_route_neighbor_visual.boundary(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual))
		if plan.is_empty(): return false
	if snapshot.get('hiveRouteHoldMissing')==true: return false
	var v=snapshot.get('hiveRouteNeighbor')
	if snapshot.get('hiveRouteHold') is Dictionary or (v is Dictionary and v.get('ready')==true): return not _hive_route_plan().is_empty()
	return boundary is Dictionary and boundary.get('closed')==true

func gh_first_pair_portal(row: Dictionary) -> bool:
	if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or row.get('kind')!='scene_transition': return false
	return (snapshot.sceneId=='gh_entry_maintenance' and row.get('entityId')=='gh_entry_to_power_room') or (snapshot.sceneId=='gh_power_room' and row.get('entityId')=='gh_power_return_to_entry')

func _gh_first_pair_caption(row: Dictionary) -> String:
	if not gh_first_pair_portal(row) or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!=snapshot.sceneId or current_key!='grey_hive:'+snapshot.sceneId+':'+str(snapshot.worldEpoch): return ''
	var b=snapshot.get('hiveRouteBoundary')
	var target='gh_power_room' if snapshot.sceneId=='gh_entry_maintenance' else 'gh_entry_maintenance'
	if not b is Dictionary or b.get('sourceSceneId')!=snapshot.sceneId or b.get('targetSceneId')!=target or b.get('worldEpoch')!=snapshot.worldEpoch or b.get('sourceSceneSourceSha256')!=scene_hash or typeof(b.get('closed'))!=TYPE_BOOL or typeof(b.get('east'))!=TYPE_BOOL or b.east!=(snapshot.sceneId=='gh_entry_maintenance'): return ''
	if b.closed:
		return '通道准备中' if _hive_route_boundary_ready() else ''
	var neighbor=snapshot.get('hiveRouteNeighbor')
	if not neighbor is Dictionary or neighbor.get('ready')!=true or neighbor.get('sourceSceneId')!=snapshot.sceneId or neighbor.get('targetSceneId')!=target or neighbor.get('worldEpoch')!=snapshot.worldEpoch or not hive_route_preview_ready(): return ''
	return '步行前往动力室' if target=='gh_power_room' else '步行返回维护入口'

func hive_route_ordinary_suppressed(row: Dictionary) -> bool:
	if not P.snapshot(snapshot) or row.get('kind')!='scene_transition': return false
	if gh_first_pair_portal(row): return true
	if snapshot.worldId=='clockworks':
		var portals={'cw_entry_foundry':['','cw_to_pressure_hall'],'cw_pressure_hall':['cw_return_to_entry_foundry','cw_to_conveyor_bridge'],'cw_conveyor_bridge':['cw_bridge_to_pressure_hall','cw_bridge_to_boiler_chamber'],'cw_boiler_chamber':['cw_boiler_return_to_conveyor_bridge','cw_boiler_to_gear_shaft'],'cw_gear_shaft':['cw_gear_return_to_boiler',''],'cw_furnace_heart':['','cw_furnace_heart_to_arena'],'cw_forged_guard_arena':['cw_arena_return_to_furnace_heart','cw_arena_to_regulator_core'],'cw_regulator_core':['cw_regulator_core_return_to_arena','cw_regulator_core_to_shutdown_exit'],'cw_shutdown_exit':['cw_shutdown_exit_return_to_regulator_core','']}
		var b=snapshot.get('hiveRouteBoundary')
		return portals.has(snapshot.sceneId) and b is Dictionary and b.get('sourceSceneId')==snapshot.sceneId and b.get('worldEpoch')==snapshot.worldEpoch and typeof(b.get('east'))==TYPE_BOOL and row.get('entityId')==portals[snapshot.sceneId][1 if b.east else 0]
	if snapshot.worldId=='mist_harbor':
		var portals={'mh_fog_pier':['','mh_to_tidal_warehouse'],'mh_tidal_warehouse':['mh_warehouse_to_fog_pier','mh_warehouse_to_signal_yard'],'mh_signal_yard':['mh_signal_yard_to_warehouse','mh_signal_yard_to_quay'],'mh_drowned_quay':['mh_drowned_quay_to_signal_yard','mh_drowned_quay_to_breakwater'],'mh_breakwater':['mh_breakwater_to_drowned_quay','mh_breakwater_to_pump_station'],'mh_pump_station':['mh_pump_station_to_breakwater','mh_pump_station_to_tower'],'mh_resonance_tower':['mh_tower_to_pump_station','mh_tower_to_warden_arena'],'mh_warden_arena':['mh_warden_arena_to_tower','mh_warden_arena_to_extraction'],'mh_extraction':['mh_extraction_to_warden_arena','']}
		var b=snapshot.get('hiveRouteBoundary')
		return portals.has(snapshot.sceneId) and b is Dictionary and b.get('sourceSceneId')==snapshot.sceneId and b.get('worldEpoch')==snapshot.worldEpoch and typeof(b.get('east'))==TYPE_BOOL and row.get('entityId')==portals[snapshot.sceneId][1 if b.east else 0]
	if snapshot.worldId!='grey_hive': return false
	var ids={'gh_entry_maintenance':'gh_entry_to_power_room','gh_power_room':'gh_power_return_to_entry','gh_lockdown':'gh_lockdown_to_bio_isolation','gh_bio_isolation':'gh_bio_to_gate_b','gh_deep_decon':'gh_deep_decon_to_sentinel_arena','gh_sentinel_arena':'gh_sentinel_arena_to_beacon'}
	if snapshot.sceneId=='gh_sentinel_arena' and row.get('entityId')=='gh_sentinel_arena_return_to_deep_decon': return snapshot.get('hiveRouteBoundary',{}).get('east')==false
	if not ids.has(snapshot.sceneId) or row.get('entityId')!=ids[snapshot.sceneId]: return false
	var b=snapshot.get('hiveRouteBoundary')
	return b is Dictionary and b.get('sourceSceneId')==snapshot.sceneId and b.get('worldEpoch')==snapshot.worldEpoch

func _beacon_exit_plan() -> Dictionary:
	if not P.snapshot(snapshot): return {}
	if snapshot.get('beaconExitNeighbor') is Dictionary:
		var fresh=gh_beacon_exit_neighbor_visual.prepare(self,snapshot,scene,scene_hash,current_key,camera,viewport_rect,gh_shaft_lift_visual,gh_exit_source_owned_visual,gh_exit_environment_visual,gh_bio_lockdown_visual,gh_supply_crate_visual,gh_beacon_environment_visual,gh_utility_layout_visual,registered_props)
		if not fresh.is_empty(): return fresh
	return _beacon_exit_held_plan()

func _beacon_exit_held_plan() -> Dictionary:
	var h=snapshot.get('beaconExitHold')
	if not h is Dictionary or h.size()!=6 or beacon_exit_cache.get('authorized')!=true or h.get('sourceSceneId')!=snapshot.sceneId or h.get('worldEpoch')!=snapshot.worldEpoch or h.get('sourceSceneSourceSha256')!=scene_hash or beacon_exit_cache.get('player')!=snapshot.player.entityId: return {}
	var old_plan: Dictionary={};var shift=Vector3.ZERO
	if beacon_exit_cache.get('source')==h.sourceSceneId and beacon_exit_cache.get('target')==h.targetSceneId and beacon_exit_cache.get('epoch')==h.worldEpoch and beacon_exit_cache.get('sourceHash')==h.sourceSceneSourceSha256 and beacon_exit_cache.get('targetHash')==h.targetSceneSourceSha256:
		old_plan=beacon_exit_cache.get('targetPlan',{})
	elif beacon_exit_cache.get('source')==h.targetSceneId and beacon_exit_cache.get('target')==h.sourceSceneId and int(beacon_exit_cache.get('epoch',-1))+1==h.worldEpoch and beacon_exit_cache.get('sourceHash')==h.targetSceneSourceSha256 and beacon_exit_cache.get('targetHash')==h.sourceSceneSourceSha256:
		old_plan=beacon_exit_cache.get('sourcePlan',{})
		shift=Vector3(-24,0,0) if h.sourceSceneId=='gh_exit' else Vector3(24,0,0)
	else: return {}
	var plan=gh_beacon_exit_neighbor_visual.reproject(old_plan,beacon_exit_cache.get('origin',Vector3.INF),beacon_exit_cache.get('center',Vector2.INF),float(beacon_exit_cache.get('ppm',0.0)),camera,shift)
	if plan.is_empty(): return {}
	plan['held']=h.duplicate(true)
	return plan

func _beacon_exit_hold_ready() -> bool:
	var h=snapshot.get('beaconExitHold')
	if not h is Dictionary or beacon_exit_frame.get('epoch')!=snapshot.worldEpoch or beacon_exit_frame.get('player')!=snapshot.player.entityId or Time.get_ticks_msec()-beacon_exit_frame_at>SHAFT_PENDING_MAX_MS: return false
	if beacon_exit_frame.get('held')==h: return true
	# The immediately preceding frame already showed this same authorized
	# source-side strip; carry it only until the next matching held draw.
	return beacon_exit_cache.get('source')==h.get('sourceSceneId') and beacon_exit_cache.get('epoch')==h.get('worldEpoch') and beacon_exit_frame.get('token')==beacon_exit_cache.get('token') and not _beacon_exit_held_plan().is_empty()

func beacon_exit_preview_ready() -> bool:
	var v=snapshot.get('beaconExitNeighbor')
	if not v is Dictionary or beacon_exit_frame.get('token')!=v.get('token') or beacon_exit_frame.get('hash')!=v.get('targetSceneSourceSha256') or beacon_exit_frame.get('sourceHash')!=scene_hash or beacon_exit_frame.get('epoch')!=snapshot.get('worldEpoch') or beacon_exit_frame.get('player')!=snapshot.player.get('entityId'): return false
	if beacon_exit_frame.get('progress')!=Content.progress(snapshot,'grey_hive') or beacon_exit_frame.get('beacon')!=snapshot.get('greyHiveBeacon'): return false
	return Time.get_ticks_msec()-beacon_exit_frame_at<=SHAFT_PENDING_MAX_MS

func beacon_exit_ordinary_suppressed(row: Dictionary) -> bool:
	if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or row.get('kind')!='scene_transition': return false
	var expected='gh_beacon_to_exit' if snapshot.sceneId=='gh_beacon' else ('gh_exit_to_beacon' if snapshot.sceneId=='gh_exit' else '')
	if expected.is_empty() or row.get('entityId')!=expected: return false
	var v=snapshot.get('beaconExitNeighbor');var h=snapshot.get('beaconExitHold')
	return (v is Dictionary and v.get('sourceSceneId')==snapshot.sceneId and v.get('worldEpoch')==snapshot.worldEpoch) or (h is Dictionary and h.get('sourceSceneId')==snapshot.sceneId and h.get('worldEpoch')==snapshot.worldEpoch) or snapshot.get('beaconExitHoldMissing')==true

func _lockdown_context() -> bool:
	return P.snapshot(snapshot) and snapshot.worldId=='grey_hive' and snapshot.sceneId=='gh_lockdown'

func _lockdown_plan() -> Dictionary:
	if not _lockdown_context(): return {}
	var progression=snapshot.get('progression')
	if not progression is Dictionary or progression.get('currentWorldId')!='grey_hive' or not P.rows(progression.get('worlds'),16) or not P.unique(progression.worlds,'worldId'): return {}
	var progress: Dictionary = Content.progress(snapshot,'grey_hive')
	if progress.is_empty() or not P.counter(progress.get('visitId'),1) or not progress.get('completedEvents') is Array: return {}
	if not P.rows(scene.get('collision'),2048) or not P.unique(scene.collision,'id') or not P.rows(scene.get('interactions'),2048) or not P.unique(scene.interactions,'id'): return {}
	var seals=scene.get('collision',[]).filter(func(row): return row is Dictionary and row.get('id')=='gh_lockdown_bio_seal')
	var terminals=scene.get('interactions',[]).filter(func(row): return row is Dictionary and row.get('id')=='gh_lockdown_terminal')
	if seals.size()!=1 or terminals.size()!=1: return {}
	var terminal: Dictionary = terminals[0]
	if terminal.size()!=4 or terminal.get('kind')!='lockdown_terminal' or terminal.get('event')!='hive_lockdown' or not _gh_state_authored_position(terminal.get('position'),[4,0,8]): return {}
	var seal_open: bool = 'hive_lockdown' in progress.completedEvents
	if seal_open and not _gh_lockdown_open_seal(seals[0]): return {}
	return gh_gate_decon_visual.prepare(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,seal_open,gh_lockdown_environment_visual)

func _lockdown_frame_current() -> bool:
	if lockdown_frame.get('key')!=current_key or lockdown_frame.get('hash')!=scene_hash or lockdown_frame.get('player')!=snapshot.player.entityId: return false
	if lockdown_frame.get('epoch')!=snapshot.worldEpoch or snapshot.serverTick<lockdown_frame.get('tick',snapshot.serverTick) or snapshot.authorityRevision<lockdown_frame.get('revision',snapshot.authorityRevision): return false
	return Time.get_ticks_msec()-lockdown_frame_at<=SHAFT_PENDING_MAX_MS

func _shaft_lockdown_portal_required() -> bool:
	if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or not snapshot.sceneId in ['gh_central_shaft','gh_lockdown'] or not P.transform(snapshot.player.get('transform')): return false
	var p: Vector3 = P.vector(snapshot.player.transform.positionM)
	if not p.is_finite() or p.y!=0.0 or p.z<7.0 or p.z>9.0: return false
	if snapshot.sceneId=='gh_central_shaft' and (p.x<22.5 or p.x>24.0): return false
	if snapshot.sceneId=='gh_lockdown' and (p.x<0.0 or p.x>1.5): return false
	var progress: Dictionary = Content.progress(snapshot,'grey_hive')
	if progress.is_empty() or not progress.get('completedEvents') is Array: return true
	return 'hive_power' in progress.completedEvents

func _shaft_lockdown_neighbor_ready() -> bool:
	var v=snapshot.get('shaftLockdownNeighbor')
	var target: String = 'gh_lockdown' if snapshot.sceneId=='gh_central_shaft' else 'gh_central_shaft'
	if not v is Dictionary or v.size()!=7 or v.get('sourceSceneId')!=snapshot.sceneId or v.get('targetSceneId')!=target or not P.counter(v.get('worldEpoch')) or not P.counter(v.get('serverTick')) or not P.counter(v.get('authorityRevision')) or v.worldEpoch!=snapshot.worldEpoch or v.serverTick!=snapshot.serverTick or v.authorityRevision!=snapshot.authorityRevision or typeof(v.get('shaftDeferred'))!=TYPE_BOOL: return false
	if not v.get('targetSceneSourceSha256') is String: return false
	var catalog=gh_gate_decon_visual._catalog(target,v.targetSceneSourceSha256)
	if catalog==null or catalog.source_hash!=v.get('targetSceneSourceSha256') or not gh_shaft_lift_visual._load_once(): return false
	var b=catalog.current.get('boundsM')
	return b is Dictionary and b.get('x')==0 and b.get('z')==0 and b.get('width')==24 and b.get('depth')==16

func _shaft_lockdown_gate_closed() -> Variant:
	if snapshot.get('sceneId')=='gh_central_shaft':
		if not P.transform(snapshot.player.get('transform')): return null
		var player_position: Vector3 = P.vector(snapshot.player.transform.positionM)
		if not player_position.is_finite(): return null
		if player_position.y<0.0: return false
	var progression=snapshot.get('progression')
	if not progression is Dictionary or progression.get('currentWorldId')!='grey_hive' or not P.rows(progression.get('worlds'),16) or not P.unique(progression.worlds,'worldId'): return null
	var progress: Dictionary = Content.progress(snapshot,'grey_hive')
	if progress.is_empty() or not P.counter(progress.get('visitId'),1) or not progress.get('completedEvents') is Array: return null
	return 'hive_power' not in progress.completedEvents

func _shaft_lockdown_seal_plan() -> Dictionary:
	if _shaft_lockdown_gate_closed()!=true: return {}
	return gh_gate_decon_visual.shaft_lockdown_seal(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,gh_lockdown_environment_visual)

func _shaft_lockdown_seal_frame_current() -> bool:
	if shaft_lockdown_seal_frame.get('key')!=current_key or shaft_lockdown_seal_frame.get('hash')!=scene_hash or shaft_lockdown_seal_frame.get('player')!=snapshot.player.entityId: return false
	if shaft_lockdown_seal_frame.get('epoch')!=snapshot.worldEpoch or snapshot.serverTick<shaft_lockdown_seal_frame.get('tick',snapshot.serverTick) or snapshot.authorityRevision<shaft_lockdown_seal_frame.get('revision',snapshot.authorityRevision): return false
	return Time.get_ticks_msec()-shaft_lockdown_seal_frame_at<=SHAFT_PENDING_MAX_MS

func _utility_new_scope() -> bool:
	return P.snapshot(snapshot) and snapshot.worldId=='grey_hive' and gh_utility_layout_visual.is_new(str(snapshot.sceneId),scene_hash)

func _exit_source_scope() -> bool:
	return P.snapshot(snapshot) and snapshot.worldId=='grey_hive' and snapshot.sceneId=='gh_exit'

# Prepare the target without replacing the live canvas or camera.
func prepare_exit_preview(target: Dictionary, target_scene: Dictionary, target_hash: String) -> bool:
	if not P.snapshot(target) or target.get('exitEntryPreview')!=true or not P.token(target.get('entryToken')) or not P.same(target.entryToken,target): return false
	var viewport: Rect2=get_viewport_rect()
	if viewport.size.x<=0 or viewport.size.y<=0: return false
	var preview_camera=Camera.new()
	preview_camera.update(target,target_scene,viewport.size,0.0,viewport,target_hash)
	var key='grey_hive:gh_exit:'+str(target.worldEpoch)
	var plan=gh_exit_source_owned_visual.prepare(target,target_scene,key,target_hash,preview_camera,gh_shaft_lift_visual,gh_exit_environment_visual,gh_bio_lockdown_visual,gh_supply_crate_visual)
	if plan.is_empty() or plan.get('hash')!=target_hash: return false
	if target_hash==GhBeaconExitRegionVisual.EXIT:
		return not gh_beacon_exit_region_visual.prepare(target,target_scene,target_hash,preview_camera,gh_shaft_lift_visual,true).is_empty()
	return true

# A saved v2 pose is staged without installing it into the live canvas.
func prepare_shaft_service_preview(target: Dictionary, target_scene: Dictionary, target_hash: String) -> bool:
	if not P.snapshot(target) or target.get('shaftServiceEntryPreview')!=true or target.sceneId!='gh_central_shaft' or not P.token(target.get('entryToken')) or not P.same(target.entryToken,target): return false
	var identity=target.get('shaftLowerService')
	if not identity is Dictionary or identity.size()!=2 or identity.get('recipeVersion')!=2 or identity.get('sceneSourceSha256')!=target_hash: return false
	if target.get('shaftLiftDeferred')!=false or not gh_shaft_lift_visual._load_once(): return false
	var view: Dictionary=V.shaft_lift_view(target.get('shaftLift'),target,target_scene,target_hash)
	if view.is_empty() or view.legacyFloorActive: return false
	var viewport: Rect2=get_viewport_rect()
	if viewport.size.x<=0 or viewport.size.y<=0: return false
	var preview_camera=Camera.new()
	preview_camera.update(target,target_scene,viewport.size,0.0,viewport,target_hash)
	var regions: Array=[]
	for region in gh_shaft_lift_visual._data.get('regions',[]):
		if not region is Dictionary or not region.get('id') is String: return false
		regions.append(region.id)
	regions.append('well')
	var lower_plan: Dictionary=gh_shaft_lift_visual.prepare(target,target_scene,target_hash,Callable(preview_camera,'project'),'lower',regions)
	var upper_plan: Dictionary=gh_shaft_lift_visual.prepare(target,target_scene,target_hash,Callable(preview_camera,'project'),'upper',['upper','well'])
	if lower_plan.is_empty() or upper_plan.is_empty(): return false
	return not gh_shaft_lower_service_visual.prepare_candidate(Callable(preview_camera,'project'),identity,target_scene,[],preview_camera.ppm).is_empty()
# Preflight the entire candidate without installing or drawing it. The entry
# token remains the authority for the subsequent ready request.
func prepare_shaft_service_proposal() -> bool:
	if not P.snapshot(snapshot) or snapshot.sceneId!='gh_central_shaft' or not P.token(snapshot.get('entryToken')) or not P.same(snapshot.entryToken,snapshot): return false
	var proposal=snapshot.get('shaftLowerServiceProposal')
	if not proposal is Dictionary or proposal.size()!=2 or proposal.get('recipeVersion')!=2 or proposal.get('sceneSourceSha256')!=scene_hash: return false
	if _prepare_shaft_visual().is_empty(): return false
	return not gh_shaft_lower_service_visual.prepare_candidate(Callable(camera,'project'),proposal,scene,[],camera.ppm).is_empty()
func _exit_source_compatible() -> bool:
	if not _exit_source_scope() or snapshot.get('utilitySceneSourceSha256')!=scene_hash or exit_source_plan.is_empty() or exit_source_plan.get('hash')!=scene_hash or exit_source_plan.get('key')!=current_key or exit_source_plan.get('epoch')!=snapshot.worldEpoch or exit_source_plan.get('player')!=snapshot.player.entityId: return false
	if _pair_visibility_scope() and (exit_source_plan.get('region')!=snapshot.get('denseOccupiedRegion') or snapshot.get('currentVisibleRegions')!=[exit_source_plan.get('region')]): return false
	if not P.counter(exit_source_plan.get('tick')) or not P.counter(exit_source_plan.get('revision')) or snapshot.serverTick<exit_source_plan.tick or snapshot.authorityRevision<exit_source_plan.revision: return false
	var progress=Content.progress(snapshot,'grey_hive')
	return not progress.is_empty() and P.counter(progress.get('visitId'),1) and progress.visitId==exit_source_plan.get('visit') and progress.get('completedEvents') is Array and ('hive_extraction' in progress.completedEvents)==exit_source_plan.get('open')

func _exit_source_current() -> bool:
	return _exit_source_compatible() and exit_source_plan.tick==snapshot.serverTick and exit_source_plan.revision==snapshot.authorityRevision and exit_source_plan.get('camera_origin')==camera.origin and exit_source_plan.get('camera_ppm')==camera.ppm and exit_source_plan.get('camera_center')==camera.screen_center

func _exit_source_frame_current() -> bool:
	if _pair_visibility_scope() and (exit_source_frame.get('region')!=snapshot.get('denseOccupiedRegion') or exit_source_frame.get('region_open')!=exit_source_plan.get('region_open')): return false
	return _exit_source_compatible() and exit_source_frame.get('key')==current_key and exit_source_frame.get('hash')==scene_hash and exit_source_frame.get('epoch')==snapshot.worldEpoch and exit_source_frame.get('player')==snapshot.player.entityId and exit_source_frame.get('visit')==exit_source_plan.visit and exit_source_frame.get('open')==exit_source_plan.open and Time.get_ticks_msec()-exit_source_frame_at<=SHAFT_PENDING_MAX_MS

func _utility_plan_current() -> bool:
	return _utility_new_scope() and snapshot.get('utilitySceneSourceSha256')==scene_hash and not utility_plan.is_empty() and utility_plan.get('key')==current_key and utility_plan.get('hash')==scene_hash and utility_plan.get('epoch')==snapshot.worldEpoch and utility_plan.get('tick')==snapshot.serverTick and utility_plan.get('revision')==snapshot.authorityRevision and utility_plan.get('player')==snapshot.player.entityId and utility_plan.get('camera_origin')==camera.origin and utility_plan.get('camera_ppm')==camera.ppm and utility_plan.get('camera_center')==camera.screen_center

func _utility_plan_compatible() -> bool:
	if (_decon_visibility_scope() or _bio_visibility_scope() or _pair_visibility_scope()) and (utility_plan.get('region')!=snapshot.get('denseOccupiedRegion') or snapshot.get('currentVisibleRegions')!=[utility_plan.get('region')]): return false
	return _utility_new_scope() and snapshot.get('utilitySceneSourceSha256')==scene_hash and not utility_plan.is_empty() and utility_plan.get('key')==current_key and utility_plan.get('hash')==scene_hash and utility_plan.get('epoch')==snapshot.worldEpoch and utility_plan.get('player')==snapshot.player.entityId and P.counter(utility_plan.get('tick')) and P.counter(utility_plan.get('revision')) and snapshot.serverTick>=utility_plan.tick and snapshot.authorityRevision>=utility_plan.revision

func _utility_frame_current() -> bool:
	if (_decon_visibility_scope() or _bio_visibility_scope() or _pair_visibility_scope()) and utility_frame.get('region')!=snapshot.get('denseOccupiedRegion'): return false
	if _pair_visibility_scope() and utility_frame.get('region_open')!=utility_plan.get('region_open'): return false
	if utility_frame.get('key')!=current_key or utility_frame.get('hash')!=scene_hash or utility_frame.get('player')!=snapshot.player.entityId: return false
	if utility_frame.get('epoch')!=snapshot.worldEpoch or snapshot.serverTick<utility_frame.get('tick',snapshot.serverTick) or snapshot.authorityRevision<utility_frame.get('revision',snapshot.authorityRevision): return false
	return Time.get_ticks_msec()-utility_frame_at<=SHAFT_PENDING_MAX_MS

# A fully prepared new region is awaiting its first draw. It cannot authorize
# commands yet, but the ordinary redraw gap must not force a key-release cycle.
func _decon_frame_pending() -> bool:
	if not (_decon_visibility_scope() or _bio_visibility_scope()) or not is_dense_region_available(): return false
	if utility_frame.get('key')!=current_key or utility_frame.get('hash')!=scene_hash or utility_frame.get('player')!=snapshot.player.entityId or utility_frame.get('epoch')!=snapshot.worldEpoch: return false
	if typeof(utility_frame.get('region'))!=TYPE_STRING or utility_frame.get('region')==snapshot.get('denseOccupiedRegion'): return false
	if snapshot.serverTick<utility_frame.get('tick',snapshot.serverTick) or snapshot.authorityRevision<utility_frame.get('revision',snapshot.authorityRevision): return false
	return Time.get_ticks_msec()-utility_frame_at<=SHAFT_PENDING_MAX_MS

func _pair_frame_pending() -> bool:
	if not _pair_visibility_scope() or not is_dense_region_available(): return false
	var frame: Dictionary=utility_frame if snapshot.sceneId=='gh_beacon' else exit_source_frame
	var plan: Dictionary=utility_plan if snapshot.sceneId=='gh_beacon' else exit_source_plan
	var frame_at: int=utility_frame_at if snapshot.sceneId=='gh_beacon' else exit_source_frame_at
	if frame.get('key')!=current_key or frame.get('hash')!=scene_hash or frame.get('player')!=snapshot.player.entityId or frame.get('epoch')!=snapshot.worldEpoch or frame.get('region_open')!=plan.get('region_open'): return false
	if not P.counter(frame.get('tick')) or not P.counter(frame.get('revision')) or snapshot.serverTick<frame.tick or snapshot.authorityRevision<frame.revision: return false
	if typeof(frame.get('region'))!=TYPE_STRING or frame.get('region')==snapshot.get('denseOccupiedRegion'): return false
	return Time.get_ticks_msec()-frame_at<=SHAFT_PENDING_MAX_MS

func _gate_decon_context() -> bool:
	return P.snapshot(snapshot) and snapshot.worldId=='grey_hive' and snapshot.sceneId in ['gh_gate_b','gh_deep_decon']

func _gate_decon_plan() -> Dictionary:
	if not _gate_decon_context(): return {}
	if _utility_new_scope(): return utility_plan if _utility_plan_compatible() else {}
	if snapshot.sceneId=='gh_gate_b' and not gh_gate_b_environment_visual.whole_static_ready(self,snapshot,scene,current_key,camera,viewport_rect,grey_hive_state_props,gh_supply_crate_visual): return {}
	var barrier_open: Variant = null
	if snapshot.sceneId=='gh_gate_b':
		var progress: Dictionary = Content.progress(snapshot,'grey_hive')
		if progress.is_empty() or not progress.get('completedEvents') is Array or not snapshot.get('doors') is Array: return {}
		var barriers=scene.get('collision',[]).filter(func(row): return row is Dictionary and row.get('id')=='gh_gate_b_gate_barrier')
		var doors=snapshot.doors.filter(func(row): return row is Dictionary and row.get('doorId')=='gh_gate_b')
		if barriers.size()!=1 or doors.size()!=1: return {}
		barrier_open='hive_lockdown' in progress.completedEvents
		if barrier_open:
			if not _gh_gate_b_open_barrier(barriers[0]): return {}
		elif _gh_door_body(doors[0]).is_empty() or doors[0].get('open')!=false or doors[0].get('locked')!=true: return {}
	return gh_gate_decon_visual.prepare(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,barrier_open)

func _gate_decon_portal_required() -> bool:
	if not _gate_decon_context() or not P.transform(snapshot.player.get('transform')): return false
	var p: Vector3 = P.vector(snapshot.player.transform.positionM)
	if not p.is_finite() or p.y!=0.0 or p.z<7.0 or p.z>9.0: return false
	if snapshot.sceneId=='gh_gate_b' and (p.x<22.0 or p.x>24.0): return false
	if snapshot.sceneId=='gh_deep_decon' and (p.x<0.0 or p.x>2.0): return false
	var progress: Dictionary = Content.progress(snapshot,'grey_hive')
	if progress.is_empty() or not progress.get('completedEvents') is Array: return true
	return 'hive_lockdown' in progress.completedEvents

func _gate_decon_frame_current() -> bool:
	if gate_decon_frame.get('key')!=current_key or gate_decon_frame.get('hash')!=scene_hash or gate_decon_frame.get('player')!=snapshot.player.entityId: return false
	if gate_decon_frame.get('epoch')!=snapshot.worldEpoch or snapshot.serverTick<gate_decon_frame.get('tick',snapshot.serverTick) or snapshot.authorityRevision<gate_decon_frame.get('revision',snapshot.authorityRevision): return false
	return Time.get_ticks_msec()-gate_decon_frame_at<=SHAFT_PENDING_MAX_MS

func _gate_shaft_portal_required() -> bool:
	if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or not snapshot.sceneId in ['gh_gate_a','gh_central_shaft'] or not P.transform(snapshot.player.get('transform')): return false
	var p: Vector3 = P.vector(snapshot.player.transform.positionM)
	if not p.is_finite() or p.y!=0.0: return false
	var in_portal: bool = (p.x>=17.5 and p.x<=20.0 and p.z>=6.0 and p.z<=8.0) if snapshot.sceneId=='gh_gate_a' else (p.x>=0.0 and p.x<=1.5 and p.z>=7.0 and p.z<=9.0)
	if not in_portal: return false
	var progress: Dictionary = Content.progress(snapshot,'grey_hive')
	if progress.is_empty() or not progress.get('completedEvents') is Array: return true
	return 'hive_power' in progress.completedEvents

func _gate_shaft_portal_plan() -> Dictionary:
	return gh_gate_shaft_portal_visual.prepare(snapshot,scene,current_key,camera,gh_shaft_lift_visual)

func _gate_shaft_portal_frame_current() -> bool:
	if gate_shaft_portal_frame.get('key')!=current_key or gate_shaft_portal_frame.get('hash')!=scene_hash or gate_shaft_portal_frame.get('player')!=snapshot.player.entityId: return false
	if gate_shaft_portal_frame.get('epoch')!=snapshot.worldEpoch or snapshot.serverTick<gate_shaft_portal_frame.get('tick',snapshot.serverTick) or snapshot.authorityRevision<gate_shaft_portal_frame.get('revision',snapshot.authorityRevision): return false
	return Time.get_ticks_msec()-gate_shaft_portal_frame_at<=SHAFT_PENDING_MAX_MS

func _shaft_context_matches(value: Dictionary) -> bool:
	return value.get('key')==current_key and value.get('hash')==scene_hash and value.get('epoch')==snapshot.worldEpoch and value.get('tick')==snapshot.serverTick and value.get('revision')==snapshot.authorityRevision and value.get('player')==snapshot.player.entityId

func _prepare_shaft_pending() -> void:
	shaft_pending_context.clear()
	if not P.snapshot(snapshot) or shaft_frame_context.is_empty() or Time.get_ticks_msec()-shaft_frame_at>SHAFT_PENDING_MAX_MS: return
	if shaft_frame_context.get('key')!=current_key or shaft_frame_context.get('hash')!=scene_hash or shaft_frame_context.get('epoch')!=snapshot.worldEpoch or shaft_frame_context.get('player')!=snapshot.player.entityId: return
	if snapshot.serverTick<shaft_frame_context.get('tick',snapshot.serverTick) or snapshot.authorityRevision<shaft_frame_context.get('revision',snapshot.authorityRevision): return
	var mode: String = ''
	if shaft_legacy_deferred(): mode='deferred'
	elif shaft_visual_expected(): mode='active'
	if mode.is_empty() or shaft_frame_context.get('mode')!=mode: return
	if mode=='active' and _prepare_shaft_visual().is_empty(): return
	if mode=='deferred' and (not gh_central_shaft_environment_visual._valid_context(snapshot,scene,current_key) or not gh_central_shaft_environment_visual._valid_camera(camera,viewport_rect)): return
	shaft_pending_context={'key':current_key,'hash':scene_hash,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':snapshot.player.entityId,'mode':mode}

func shaft_visual_expected() -> bool:
	if not snapshot is Dictionary or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_central_shaft': return false
	if shaft_legacy_deferred(): return false
	if scene_hash==GhCentralShaftEnvironmentVisual.SCENE_SHA256: return true
	if snapshot.get('shaftLift') != null: return true
	if not scene is Dictionary or not scene.get('movingSupports') is Array: return false
	for support in scene.movingSupports:
		if support is Dictionary and support.get('id')=='gh_shaft_two_stop_lift': return true
	return false

func shaft_legacy_deferred() -> bool:
	if not P.snapshot(snapshot) or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_central_shaft': return false
	if typeof(snapshot.get('shaftLiftDeferred'))!=TYPE_BOOL or snapshot.shaftLiftDeferred!=true or snapshot.get('shaftLift')!=null or snapshot.get('supportScene')!=null: return false
	if current_key!='grey_hive:gh_central_shaft:'+str(snapshot.worldEpoch) or scene_hash!=GhCentralShaftEnvironmentVisual.SCENE_SHA256: return false
	if not scene is Dictionary or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId: return false
	var supports=scene.get('movingSupports')
	if not supports is Array or supports.size()!=1 or not supports[0] is Dictionary or supports[0].get('id')!='gh_shaft_two_stop_lift': return false
	var p=P.vector(snapshot.player.transform.positionM)
	return p.is_finite() and p.y>=0.0 and p.y<=3.0

func _prepare_shaft_visual() -> Dictionary:
	if not shaft_visual_expected() or not P.snapshot(snapshot) or not camera.initialized: return {}
	if typeof(snapshot.get('shaftLiftDeferred'))!=TYPE_BOOL or snapshot.shaftLiftDeferred!=false: return {}
	if current_key!='grey_hive:gh_central_shaft:'+str(snapshot.worldEpoch): return {}
	var view: Dictionary = V.shaft_lift_view(snapshot.get('shaftLift'),snapshot,scene,scene_hash)
	if view.is_empty() or view.legacyFloorActive or not gh_shaft_lift_visual._load_once(): return {}
	var p: Vector3 = P.vector(view.rider.positionM)
	var moving: bool = view.pose.phase in ['rising','falling']
	var inside_well: bool = p.x>=1.5 and p.x<=5.5 and p.z>=1.0 and p.z<=5.0
	var transit: bool = moving and inside_well
	var upper: bool = transit or p.y>=0.0
	var lower: bool = transit or p.y<0.0
	var upper_regions: Array = ['upper','well']
	var lower_regions: Array = []
	var service: bool = snapshot.get('shaftLowerService') is Dictionary
	if service and lower:
		for region in gh_shaft_lift_visual._data.get('regions',[]):
			if not region is Dictionary or not region.get('id') is String: return {}
			lower_regions.append(region.id)
		if transit or (inside_well and view.pose.phase=='lower_hold'): lower_regions.append('well')
	elif transit or inside_well:
		lower_regions=['well','front_landing_corridor'] if transit or view.pose.phase=='lower_hold' else ['well']
	elif lower:
		var candidates: Array = []
		for region in gh_shaft_lift_visual._data.get('regions',[]):
			if not region is Dictionary or not region.get('id') is String: return {}
			var rect: Rect2 = gh_shaft_lift_visual._rect(region.get('polygon'))
			if rect.size==Vector2.ZERO: return {}
			if p.x>=rect.position.x and p.x<=rect.end.x and p.z>=rect.position.y and p.z<=rect.end.y: candidates.append(region.id)
		if candidates.is_empty(): return {}
		candidates.sort()
		var selected: String = shaft_previous_region if shaft_previous_key==current_key and shaft_previous_region in candidates else candidates[0]
		shaft_previous_key=current_key;shaft_previous_region=selected
		lower_regions=[selected]
		if selected=='front_landing_corridor' and view.pose.phase=='lower_hold': lower_regions.append('well')
	var plans: Array = []
	if upper:
		var upper_plan: Dictionary = gh_shaft_lift_visual.prepare(snapshot,scene,scene_hash,Callable(camera,'project'),'upper',upper_regions)
		if upper_plan.is_empty(): return {}
		plans.append(upper_plan)
	if lower:
		var lower_plan: Dictionary = gh_shaft_lift_visual.prepare(snapshot,scene,scene_hash,Callable(camera,'project'),'lower',lower_regions)
		if lower_plan.is_empty(): return {}
		if service:
			lower_plan.depth=lower_plan.depth.filter(func(piece): return piece.get('sourceId')!='lower_east')
		if upper:
			lower_plan.ground=lower_plan.ground.filter(func(piece): return piece.floor!='cabin')
			lower_plan.depth=lower_plan.depth.filter(func(piece): return piece.floor!='cabin')
		plans.append(lower_plan)
		if service:
			var visible=snapshot.get('currentVisibleRegions') if snapshot.get('currentVisibleRegions') is Array else []
			var service_plan: Dictionary=gh_shaft_lower_service_visual.prepare_candidate(Callable(camera,'project'),snapshot.shaftLowerService,scene,visible,camera.ppm)
			if service_plan.is_empty(): return {}
			plans.append(service_plan)
	if plans.is_empty(): return {}
	var ground_count:=0
	var depth_count:=0
	for plan in plans:
		ground_count+=plan.ground.size()
		depth_count+=plan.depth.size()
	if ground_count==0 or depth_count==0: return {}
	return {'plans':plans,'upper':upper,'lower':lower,'transit':transit,'key':current_key,'hash':scene_hash,
		'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':snapshot.player.entityId}

func _draw() -> void:
	protected_actor_captions.clear()
	if not mh_warden_sampled: mh_warden_temporal_visual.reset()
	mh_warden_sampled=false
	for id in mh_drowned_tidebound_by_id.keys():
		if not mh_drowned_tidebound_sampled.has(id): mh_drowned_tidebound_by_id.erase(id)
	mh_drowned_tidebound_sampled.clear()
	for id in cw_drone_hound_by_id.keys():
		if not cw_drone_hound_sampled.has(id): cw_drone_hound_by_id.erase(id)
	cw_drone_hound_sampled.clear()
	if not prime_regulator_sampled: cw_prime_regulator_temporal_visual.reset()
	prime_regulator_sampled=false
	for id in cw_regular_guard_temporal_by_id.keys():
		if not cw_regular_guard_sampled.has(id): _cw_regular_guard_drop(id)
	cw_regular_guard_sampled.clear()
	dense_frame_ready = false
	shaft_frame_context.clear()
	shaft_pending_context.clear()
	shaft_draw_failed = false
	mh_warden_arena_last_draw_report = {'ground_drawn':false,'wall_ids':[],'bank_ids':[],'ambient_polygons':0}
	_clear_gh_vital_body_records()
	mh_fog_pier_deferred_labels.clear()
	mh_fog_pier_defer_labels = false
	prop_bodies_drawn.clear()
	draw_rect(get_viewport_rect(),Color('#0b121a'))
	if snapshot.is_empty() or scene.is_empty():
		_wraith_reset_all()
		return
	if viewport_rect == Rect2():
		_wraith_reset_all()
		return
	# Bind RS geometry and the camera in the same draw as the objects.
	if _rs_environment_context():
		rs_source_plan = rs_source_owned_visual.prepare(snapshot,scene,current_key,camera,rs_environment_visual,_rs_terminal_feet())
		if rs_source_plan.is_empty():
			if rs_source_owned_visual.probe_enabled and P.snapshot(snapshot) and camera.initialized and viewport_rect.has_area(): rs_source_owned_visual.probe_finish('required_plan_empty')
			return
		if rs_source_owned_visual.probe_enabled and not rs_source_owned_visual.probe_done and P.snapshot(snapshot) and camera.initialized and viewport_rect.has_area():
			rs_source_owned_visual.probe_active=true
			rs_source_owned_visual.probe_viewport=viewport_rect
	var gh_entry_owned: bool = snapshot.get('worldId')=='grey_hive' and snapshot.get('sceneId')=='gh_entry_maintenance'
	var gh_entry_floor_prepared: Array = []
	if gh_entry_owned:
		gh_entry_floor_prepared=gh_entry_floor_material_visual.build_packets(snapshot,scene,current_key,camera)
		var entry_walls: Array=gh_entry_wall_modules_visual.build_packets(snapshot,scene,current_key,camera)
		var entry_shadows: Array=gh_entry_wall_modules_visual.build_shadow_packets(snapshot,scene,current_key,camera,entry_walls) if entry_walls.size()==63 else []
		if gh_entry_floor_prepared.size()!=70 or entry_shadows.size()!=5 or gh_entry_wall_modules_visual.suppressible_collision_ids(snapshot,scene,current_key,camera,entry_walls,entry_shadows,viewport_rect).size()!=5: return
	var hive_full_owned: bool=_hive_full_scope()
	var hive_full_plan: Dictionary=_hive_full_plan() if hive_full_owned else {}
	cw_later_owned=hive_full_owned and snapshot.worldId=='clockworks' and CwLaterRouteVisual.LATER_EDGES.has(snapshot.sceneId) and snapshot.sceneId!='cw_boiler_chamber' and not hive_full_plan.is_empty()
	cw_early_owned=hive_full_owned and snapshot.worldId=='clockworks' and not hive_full_plan.is_empty()
	if cw_early_owned: cw_early_bound_key=current_key
	if hive_full_owned and hive_full_plan.is_empty():
		var pending_route=snapshot.get('hiveRouteNeighbor')
		if snapshot.worldId=='clockworks' and cw_early_bound_key!=current_key and not snapshot.get('hiveRouteHold') is Dictionary and not snapshot.get('hiveRouteHandoff') is Dictionary and not (pending_route is Dictionary and pending_route.get('ready')==true): hive_full_owned=false
		else: return
	var hive_route_plan: Dictionary=_hive_route_plan()
	var hive_route_view=snapshot.get('hiveRouteNeighbor')
	var hive_route_hold=snapshot.get('hiveRouteHold')
	var hive_route_boundary=snapshot.get('hiveRouteBoundary')
	var hive_boundary_plan: Dictionary={}
	if hive_route_boundary is Dictionary and hive_route_boundary.get('closed')==true:
		hive_boundary_plan=mh_shore_route_visual.boundary(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,mh_fog_pier_stone_material_visual) if snapshot.worldId=='mist_harbor' else ((cw_later_route_visual.later_boundary(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,cw_pressure_floor_material_visual) if CwLaterRouteVisual.LATER_EDGES.has(snapshot.sceneId) and (snapshot.sceneId!='cw_boiler_chamber' or snapshot.get('hiveRouteBoundary',{}).get('targetSceneId')=='cw_gear_shaft') else cw_early_route_visual.boundary(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual,cw_pressure_floor_material_visual)) if snapshot.worldId=='clockworks' else gh_route_neighbor_visual.boundary(snapshot,scene,current_key,scene_hash,camera,gh_shaft_lift_visual))
		if hive_boundary_plan.is_empty() and (snapshot.worldId!='clockworks' or hive_full_owned): return
	if snapshot.get('hiveRouteHoldMissing')==true or ((hive_route_hold is Dictionary or (hive_route_view is Dictionary and hive_route_view.get('ready')==true)) and hive_route_plan.is_empty()): return
	var hive_route_floor_ok: bool=true
	hive_route_drawn=0;hive_route_draw_failed=false
	_wraith_reconcile()
	_cw_regular_guard_reconcile(true)
	var brute_rows: Array = []
	var actor_rows = snapshot.get('actors')
	if actor_rows is Array: brute_rows=actor_rows.filter(func(row): return row is Dictionary and row.get('entityId')=='gh_gate_b_brute_01')
	if brute_rows.size()!=1 or not _gate_b_brute_matches(brute_rows[0]) or not actor_visible('gh_gate_b_brute_01'):
		gh_gate_b_brute_temporal_visual.reset()
	var shaft_deferred_mode: bool = shaft_legacy_deferred()
	var shaft_mode: bool = shaft_visual_expected()
	var shaft_plan: Dictionary = _prepare_shaft_visual() if shaft_mode else {}
	# Never draw the legacy solid floor over an admitted but unprepared well.
	if shaft_mode and shaft_plan.is_empty(): return
	var shaft_ground_ok: bool = true
	var shaft_legacy_walls: Array = []
	var shaft_legacy_floor_ok: bool = not shaft_deferred_mode
	var shaft_pipe_ok: bool = not shaft_deferred_mode and (not shaft_mode or not shaft_plan.upper)
	var dense_mode = _dense_visibility_scope()
	if dense_mode: dense_visibility_context.clear()
	var dense_region_plan: Dictionary = {}
	var utility_new: bool = _utility_new_scope()
	if _decon_visibility_scope() and not _utility_plan_current(): return
	if _bio_visibility_scope() and not is_dense_region_available(): return
	if _pair_visibility_scope() and not is_dense_region_available(): return
	if utility_new and not _utility_plan_current(): return
	var exit_source: bool = _exit_source_scope()
	if exit_source and not _exit_source_current(): return
	exit_source_drawn=0
	exit_source_failed=false
	var bio_visual_scene: Dictionary = utility_plan.original_bio if utility_new and snapshot.sceneId=='gh_bio_isolation' else scene
	var beacon_visual_scene: Dictionary = utility_plan.original_beacon if utility_new and snapshot.sceneId=='gh_beacon' else scene
	var beacon_wall_plans: Dictionary = {}
	if utility_new and snapshot.sceneId=='gh_beacon' and not _pair_visibility_scope():
		for wall in beacon_visual_scene.collision:
			var commands = gh_beacon_environment_visual.prepare_layout_wall(self,snapshot,scene,beacon_visual_scene,current_key,camera,wall,viewport_rect)
			if not commands is Array or commands.is_empty() or beacon_wall_plans.has(wall.id): return
			beacon_wall_plans[wall.id]=commands
		if beacon_wall_plans.size()!=6: return
	utility_drawn_pieces=0
	utility_draw_failed=false
	utility_bio_wall_drawn=0
	utility_beacon_wall_drawn=0
	if hive_full_owned and not hive_full_plan.is_empty():
		var ambient=mh_cw_ambient_backdrop_visual.prepare(snapshot,scene,current_key,camera,hive_full_plan)
		if not ambient.is_empty(): mh_cw_ambient_backdrop_visual.draw(self,ambient)
	var whole_rs = false
	var rs_owned: Dictionary = rs_source_plan if _rs_source_plan_current() else {}
	if rs_owned.is_empty():
		rs_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,viewport_rect)
	else:
		if not rs_owned.get('background',{}).is_empty(): rs_source_owned_visual.draw_piece(self,rs_owned.background)
		if rs_source_owned_visual.probe_active:
			for i in range(rs_owned.context.size()):
				rs_source_owned_visual.probe_phase='shell' if i<rs_source_owned_visual.probe_shell_count else 'context'
				rs_source_owned_visual.draw_piece(self,rs_owned.context[i])
		else:
			for piece in rs_owned.context: rs_source_owned_visual.draw_piece(self,piece)
		if rs_source_owned_visual.probe_active: rs_source_owned_visual.probe_phase=''
	# Optional, source-bound rear context; failure leaves the complete entry scene intact.
	if gh_entry_owned and camera.initialized:
		var entry_catalog=gh_route_neighbor_visual._catalog(snapshot.sceneId,scene_hash)
		if entry_catalog!=null and entry_catalog.source_hash==scene_hash and entry_catalog.current==scene and rs_source_owned_visual._load_once():
			var north=rs_source_owned_visual._facade_plane(camera,'x',0.5,0.5,19.5)
			var west=rs_source_owned_visual._facade_plane(camera,'z',0.5,0.5,13.5)
			if not north.is_empty() and not west.is_empty():
				for piece in north+west: rs_source_owned_visual.draw_piece(self,piece)
	var whole_gh = false
	var gh_background = false
	var whole_mh = false
	var whole_cw = false
	var whole_pressure = false
	var whole_fog_pier = false
	var whole_shaft = false
	var whole_decon = false
	var whole_boiler = false
	var whole_gear = false
	var whole_heart = false
	var whole_sentinel = false
	var whole_conveyor = false
	var whole_regulator = false
	var whole_forged = false
	var whole_warehouse = false
	var whole_signal = false
	var whole_drowned = false
	var whole_gate_a = false
	var whole_gate_b = false
	var whole_warden = false
	var whole_tower = false
	var whole_extraction = false
	var whole_pump = false
	var whole_beacon = false
	var whole_exit = false
	var whole_power = false
	var power_background = false
	var power_side_plan: Dictionary = {}
	var gate_a_background = false
	var gate_wall_plan: Dictionary = {}
	var whole_bio = false
	var whole_shutdown = false
	var warehouse_frame: Dictionary = mh_tidal_warehouse_environment_visual.build_frame(snapshot,scene,current_key,camera,viewport_rect)
	var gate_decon_mode: bool = _gate_decon_context()
	var beacon_exit_plan: Dictionary = _beacon_exit_plan()
	var gate_decon_plan: Dictionary = _gate_decon_plan() if gate_decon_mode else {}
	var gate_decon_floor_drawn: bool = false
	var gate_decon_preview_drawn: bool = not _gate_decon_portal_required()
	var gate_decon_preview_actual: bool = false
	var gate_decon_walls: Array = []
	var gate_decon_wall_ok: bool = true
	var gate_decon_functional_floor_ok: bool = true
	var lockdown_mode: bool = _lockdown_context()
	var lockdown_plan: Dictionary = _lockdown_plan() if lockdown_mode else {}
	var lockdown_floor_drawn: bool = false
	var lockdown_walls: Array = []
	var lockdown_wall_ok: bool = true
	var forged_floor_plan: Dictionary = {}
	if not gh_entry_owned and gh_entry_exterior_backplate_visual._valid_context(snapshot,scene,current_key,scene_hash) and camera.initialized:
		gh_background = whole_scene_coverage_visual.draw_master(self,'gh',viewport_rect,camera,Vector3(10,0,7))
	elif not hive_full_owned and mh_breakwater_environment_visual.matches_context(self,snapshot,scene,current_key,camera,viewport_rect):
		whole_mh = whole_scene_coverage_visual.draw_master(self,'mh',viewport_rect)
	elif not hive_full_owned and cw_entry_foundry_environment_visual._context_valid(snapshot,scene,current_key) and cw_entry_foundry_environment_visual._camera_valid(camera,viewport_rect):
		whole_cw = whole_scene_coverage_visual.draw_master(self,'cw',viewport_rect)
	elif not hive_full_owned and P.snapshot(snapshot) and snapshot.get('entryToken')==null and camera.initialized and not cw_pressure_environment_visual.build_packets(scene,snapshot,current_key,camera).is_empty():
		whole_pressure = whole_scene_coverage_visual.draw_master(self,'pressure',viewport_rect)
	elif not hive_full_owned and mh_fog_pier_environment_visual._valid_context(snapshot,scene,current_key,camera,viewport_rect):
		whole_fog_pier = whole_scene_coverage_visual.draw_master(self,'fog_pier',viewport_rect)
	elif not shaft_mode and not shaft_deferred_mode and gh_central_shaft_environment_visual._valid_context(snapshot,scene,current_key) and gh_central_shaft_environment_visual._valid_camera(camera,viewport_rect):
		whole_shaft = whole_scene_coverage_visual.draw_master(self,'shaft',viewport_rect)
	elif not gate_decon_mode and gh_deep_decon_environment_visual._binding_ok(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect):
		whole_decon = whole_scene_coverage_visual.draw_master(self,'decon',viewport_rect)
	elif not hive_full_owned and cw_boiler_environment_visual._valid_context(snapshot,scene,current_key,camera,viewport_rect):
		whole_boiler = whole_scene_coverage_visual.draw_master(self,'boiler',viewport_rect)
	elif not hive_full_owned and cw_gear_shaft_environment_visual._draw_context(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect):
		whole_gear = whole_scene_coverage_visual.draw_master(self,'gear',viewport_rect)
	elif not hive_full_owned and _heart_whole_ready():
		whole_heart = whole_scene_coverage_visual.draw_master(self,'heart',viewport_rect)
	elif not hive_full_owned and _sentinel_whole_ready():
		whole_sentinel = whole_scene_coverage_visual.draw_master(self,'sentinel',viewport_rect)
	elif not hive_full_owned and _conveyor_whole_ready():
		whole_conveyor = whole_scene_coverage_visual.draw_master(self,'conveyor',viewport_rect)
	elif not hive_full_owned and cw_regulator_environment_visual.reflections_ready(self,snapshot,scene,current_key,camera,viewport_rect,cw_furnace_visual):
		whole_regulator = whole_scene_coverage_visual.draw_master(self,'regulator',viewport_rect)
	elif not hive_full_owned and cw_forged_guard_arena_environment_visual._context(self,snapshot,scene,current_key,camera,viewport_rect):
		forged_floor_plan = cw_forged_guard_arena_environment_visual.prepare_floor(self,snapshot,scene,current_key,camera,viewport_rect,cw_furnace_visual,_cw_furnace_key(cw_forged_guard_arena_environment_visual.furnace_row(scene)))
		if not forged_floor_plan.is_empty():
			whole_forged = whole_scene_coverage_visual.draw_master(self,'forged_arena',viewport_rect)
	elif not hive_full_owned and mh_tidal_warehouse_environment_visual._canvas(self) and not warehouse_frame.is_empty() and mh_tidal_warehouse_environment_visual._frame_valid(warehouse_frame,snapshot,scene,current_key,camera,viewport_rect):
		whole_warehouse = whole_scene_coverage_visual.draw_master(self,'fog_pier',viewport_rect)
	elif not hive_full_owned and mh_signal_yard_environment_visual._canvas(self) and mh_signal_yard_environment_visual._view(snapshot,scene,current_key,camera,viewport_rect):
		whole_signal = whole_scene_coverage_visual.draw_master(self,'fog_pier',viewport_rect)
	elif not hive_full_owned and _drowned_whole_ready():
		whole_drowned = whole_scene_coverage_visual.draw_master(self,'fog_pier',viewport_rect)
	elif not hive_full_owned and cw_shutdown_exit_environment_visual.whole_static_ready(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect):
		whole_shutdown = whole_scene_coverage_visual.draw_master(self,'shutdown',viewport_rect)
	elif not dense_mode and _whole_power_ready():
		power_side_plan = gh_power_gate_neighbor_visual.prepare_power_sides(scene,camera)
		if not power_side_plan.is_empty():
			power_background = whole_scene_coverage_visual.draw_master(self,'sentinel',viewport_rect,camera,Vector3(20,0,7))
	elif not utility_new and not hive_full_owned and _whole_bio_ready():
		whole_bio = whole_scene_coverage_visual.draw_master(self,'sentinel',viewport_rect)
	elif not dense_mode and _whole_gate_a_ready():
		gate_wall_plan = gh_power_gate_neighbor_visual.prepare_gate_walls(scene,camera)
		if not gate_wall_plan.is_empty():
			gate_a_background = whole_scene_coverage_visual.draw_master(self,'sentinel',viewport_rect,camera,Vector3(0,0,7))
	elif not gate_decon_mode and gh_gate_b_environment_visual.whole_static_ready(self,snapshot,scene,current_key,camera,viewport_rect,grey_hive_state_props,gh_supply_crate_visual):
		whole_gate_b = whole_scene_coverage_visual.draw_master(self,'sentinel',viewport_rect)
	elif not hive_full_owned and mh_warden_arena_environment_visual.whole_static_ready(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect):
		whole_warden = whole_scene_coverage_visual.draw_master(self,'fog_pier',viewport_rect)
	elif not hive_full_owned and mh_resonance_tower_environment_visual.whole_static_ready(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect):
		whole_tower = whole_scene_coverage_visual.draw_master(self,'fog_pier',viewport_rect)
	elif not hive_full_owned and mh_extraction_environment_visual.whole_static_ready(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect):
		whole_extraction = whole_scene_coverage_visual.draw_master(self,'extraction',viewport_rect)
	elif not hive_full_owned and mh_pump_environment_visual.whole_static_ready(self,snapshot,scene,current_key,camera,viewport_rect):
		whole_pump = whole_scene_coverage_visual.draw_master(self,'fog_pier',viewport_rect)
	elif not utility_new and not hive_full_owned and gh_beacon_environment_visual.whole_static_ready(self,snapshot,scene,current_key,camera,viewport_rect):
		whole_beacon = whole_scene_coverage_visual.draw_master(self,'sentinel',viewport_rect)
	elif not exit_source and gh_exit_environment_visual.whole_static_ready(self,snapshot,scene,current_key,camera,viewport_rect):
		whole_exit = whole_scene_coverage_visual.draw_master(self,'sentinel',viewport_rect)
	var gh_r2_plate_drawn: bool = false if gh_entry_owned or whole_gh or gh_background else gh_entry_r2_plate_candidate.draw_plate(self,snapshot,scene,current_key,scene_hash,camera)
	if not gh_entry_owned and not whole_gh and not gh_background and not gh_r2_plate_drawn:
		gh_entry_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_decon and not gate_decon_mode:
		gh_deep_decon_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_beacon and not utility_new and not hive_full_owned:
		gh_beacon_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_bio and not utility_new and not hive_full_owned:
		gh_bio_isolation_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not lockdown_mode: gh_lockdown_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not power_background and not dense_mode:
		gh_power_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_exit and not exit_source:
		gh_exit_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not gate_a_background and not dense_mode:
		gh_gate_a_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_gate_b and not gate_decon_mode:
		gh_gate_b_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_shaft and not shaft_mode:
		gh_central_shaft_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_tower:
		mh_resonance_tower_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_regulator:
		cw_regulator_core_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_signal:
		mh_signal_yard_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_conveyor:
		cw_conveyor_bridge_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_forged:
		cw_forged_guard_arena_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_warden:
		mh_warden_arena_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_boiler:
		cw_boiler_chamber_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_heart:
		cw_furnace_heart_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_fog_pier and not mh_fog_pier_exterior_r2_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect):
		mh_fog_pier_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_cw:
		cw_entry_foundry_exterior_backplate_visual.draw_backplate(self,snapshot,scene,current_key,scene_hash,viewport_rect)
	if not whole_gear:
		cw_gear_shaft_backdrop_visual.draw_backdrop(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	displayed.clear()
	action_body_frame.clear()
	if not whole_rs and rs_owned.is_empty():
		rs_environment_visual.draw_south_floor_fascia(self,snapshot,scene,current_key,camera,viewport_rect)
	var b: Dictionary = scene.boundsM
	var floor_poly = [[b.x,b.z],[b.x+b.width,b.z],[b.x+b.width,b.z+b.depth],[b.x,b.z+b.depth]]
	var floor_color = {'return_station':Color('#233c47'),'grey_hive':Color('#263634'),'mist_harbor':Color('#203742'),'clockworks':Color('#40362c')}.get(snapshot.worldId,Color('#263634'))
	if not hive_full_owned and not gh_entry_owned and not gh_r2_plate_drawn and not whole_rs and rs_owned.is_empty() and not whole_gh and not whole_mh and not whole_cw and not whole_pressure and not whole_fog_pier and not whole_shaft and not shaft_mode and not whole_decon and not whole_boiler and not whole_gear and not whole_heart and not whole_sentinel and not whole_conveyor and not whole_regulator and not whole_forged and not whole_warehouse and not whole_signal and not whole_drowned and not whole_shutdown and not whole_power and not whole_bio and not whole_gate_a and not whole_gate_b and not whole_warden and not whole_tower and not whole_extraction and not whole_pump and not whole_beacon and not whole_exit and not dense_mode and not utility_new and not exit_source and gate_decon_plan.is_empty() and lockdown_plan.is_empty():
		_poly(floor_poly,0,floor_color,true)
	var rs_pre_feet: Array = []
	var rs_pre_back: Array = []
	var rs_material_plan: Dictionary = {}
	if _rs_environment_context():
		rs_pre_feet = _rs_terminal_feet()
		rs_pre_back = rs_environment_visual.build_packets(scene,snapshot.worldId,snapshot.sceneId,camera,rs_pre_feet,'back')
		rs_material_plan = rs_floor_treatment_visual.prepare_material(self,snapshot,scene,current_key,camera,viewport_rect,rs_pre_back)
	if not rs_owned.is_empty():
		if rs_source_owned_visual.probe_active: rs_source_owned_visual.probe_phase='floor'
		rs_source_owned_visual.draw_surface(self,rs_owned.floor)
		if rs_source_owned_visual.probe_active: rs_source_owned_visual.probe_phase=''
	elif not whole_rs:
		if rs_material_plan.is_empty():
			rs_floor_treatment_visual.draw_floor(self,snapshot,scene,current_key,camera,viewport_rect)
		else:
			rs_floor_treatment_visual.draw_material(self,rs_material_plan.floor,rs_material_plan.texture)
	_draw_rs_gh_gate_pad()
	if not whole_heart and not cw_later_owned:
		cw_furnace_heart_floor_material_visual.draw_floor(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	if whole_shutdown:
		cw_shutdown_exit_environment_visual._draw_ground_light_field(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	elif not cw_later_owned:
		cw_shutdown_exit_environment_visual.draw_floor(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	var warden_ground_drawn: bool = false if whole_warden or hive_full_owned else mh_warden_arena_environment_visual.draw_ground(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	mh_warden_arena_last_draw_report['ground_drawn'] = warden_ground_drawn
	if whole_extraction:
		mh_extraction_environment_visual._field_draw(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	elif not hive_full_owned:
		mh_extraction_environment_visual.draw_ground(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	if whole_tower:
		mh_resonance_tower_environment_visual._field_draw(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	elif not hive_full_owned:
		mh_resonance_tower_environment_visual.draw_ground(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	if not whole_mh and not hive_full_owned:
		mh_breakwater_environment_visual.draw_ground(self,snapshot,scene,current_key,camera,viewport_rect)
	if not whole_gear and not cw_later_owned:
		cw_gear_shaft_environment_visual.draw_floor(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	if not whole_drowned and not hive_full_owned:
		mh_drowned_quay_environment_visual.draw_ground(self,snapshot,scene,current_key,camera,viewport_rect)
	if cw_later_owned and snapshot.sceneId=='cw_forged_guard_arena':
		forged_floor_plan=cw_forged_guard_arena_environment_visual.prepare_floor(self,snapshot,scene,current_key,camera,viewport_rect,cw_furnace_visual,_cw_furnace_key(cw_forged_guard_arena_environment_visual.furnace_row(scene)))
	cw_forged_guard_arena_environment_visual.draw_floor(self,snapshot,scene,current_key,camera,viewport_rect,cw_furnace_visual,_cw_furnace_key(cw_forged_guard_arena_environment_visual.furnace_row(scene)),whole_forged or cw_later_owned,forged_floor_plan)
	if shaft_mode:
		for plan in shaft_plan.plans:
			if not gh_shaft_lift_visual.draw_ground(self,plan): shaft_ground_ok=false
	if not whole_shaft and not shaft_mode:
		var shaft_floor_drawn: bool = gh_central_shaft_environment_visual.draw_floor(self,snapshot,scene,current_key,camera,viewport_rect)
		if shaft_deferred_mode: shaft_legacy_floor_ok=shaft_floor_drawn
	if not shaft_mode or (shaft_plan.upper and not shaft_plan.transit):
		gh_shaft_dust_visual.draw_dust(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect,_shaft_dust_gate())
	if exit_source:
		if not gh_gate_decon_visual.draw_floor(self,exit_source_plan): exit_source_failed=true
	else:
		gh_exit_environment_visual.draw_floor(self,snapshot,scene,current_key,camera,viewport_rect,whole_exit)
	if not lockdown_plan.is_empty(): lockdown_floor_drawn=gh_gate_decon_visual.draw_floor(self,lockdown_plan)
	if not lockdown_floor_drawn: gh_lockdown_environment_visual.draw_floor(self,snapshot,scene,current_key,camera,viewport_rect)
	if not whole_cw and not cw_early_owned:
		cw_entry_foundry_environment_visual.draw_floor(self,snapshot,scene,current_key,camera,get_viewport_rect(),cw_furnace_visual)
	if whole_signal or hive_full_owned and snapshot.sceneId=='mh_signal_yard':
		mh_signal_yard_environment_visual._ground_ambient_draw(self,snapshot,scene,current_key,camera,viewport_rect,true)
	else:
		mh_signal_yard_environment_visual.draw_floor(self,snapshot,scene,current_key,camera,viewport_rect)

	var warehouse_consumed_ids: Array = []
	mh_tidal_warehouse_environment_visual.draw_floor(self,warehouse_frame,snapshot,scene,current_key,camera,viewport_rect,whole_warehouse or hive_full_owned)
	if hive_full_owned and snapshot.worldId=='mist_harbor' and snapshot.sceneId=='mh_tidal_warehouse':
		mh_warehouse_crates_visual.draw_north_backdrop(self,snapshot,scene,current_key,camera,viewport_rect)
	cw_regulator_environment_visual.draw_floor(self,snapshot,scene,current_key,camera,viewport_rect,cw_furnace_visual,whole_regulator or cw_later_owned)
	if not beacon_exit_plan.is_empty() and gh_beacon_exit_neighbor_visual.draw(self,beacon_exit_plan):
		var held=snapshot.get('beaconExitHold')
		beacon_exit_frame={'token':beacon_exit_plan.get('token'),'hash':beacon_exit_plan.get('hash'),'sourceHash':scene_hash,'epoch':snapshot.worldEpoch,'player':snapshot.player.entityId,'progress':Content.progress(snapshot,'grey_hive').duplicate(true),'beacon':snapshot.get('greyHiveBeacon').duplicate(true) if snapshot.get('greyHiveBeacon') is Dictionary else null,'held':held.duplicate(true) if held is Dictionary else null}
		beacon_exit_frame_at=Time.get_ticks_msec()
		if beacon_exit_plan.has('reciprocal'):
			var v: Dictionary=snapshot.beaconExitNeighbor
			beacon_exit_cache={'source':snapshot.sceneId,'target':v.targetSceneId,'epoch':snapshot.worldEpoch,'sourceHash':scene_hash,'targetHash':v.targetSceneSourceSha256,'player':snapshot.player.entityId,'token':v.token.duplicate(true),'targetPlan':{'floor':beacon_exit_plan.floor,'walls':beacon_exit_plan.walls},'sourcePlan':beacon_exit_plan.reciprocal,'origin':camera.origin,'center':camera.screen_center,'ppm':camera.ppm,'progress':Content.progress(snapshot,'grey_hive').duplicate(true),'beacon':snapshot.get('greyHiveBeacon').duplicate(true) if snapshot.get('greyHiveBeacon') is Dictionary else null,'authorized':v.ready or (held is Dictionary and held.get('currentPreviewMatches')==true)}
	if not hive_route_plan.is_empty():
		for piece in hive_route_plan.floor:
			if not gh_route_neighbor_visual.draw_piece(self,piece): hive_route_floor_ok=false
	if hive_full_owned:
		for piece in hive_full_plan.floor:
			if not gh_route_neighbor_visual.draw_piece(self,piece): hive_route_floor_ok=false
	if cw_early_owned and snapshot.sceneId=='cw_entry_foundry':
		cw_entry_foundry_environment_visual._draw_furnace_cue(self,snapshot,scene,current_key,camera,get_viewport_rect(),cw_furnace_visual)
	if not gate_decon_plan.is_empty():
		gate_decon_floor_drawn=gh_gate_decon_visual.draw_floor(self,gate_decon_plan)
		if _gate_decon_portal_required():
			var preview: Dictionary = gh_gate_decon_visual.preview(snapshot,scene,current_key,camera,gh_shaft_lift_visual,gh_utility_layout_visual)
			if not preview.is_empty():
				gate_decon_preview_drawn=gh_gate_decon_visual.draw_floor(self,preview)
				gate_decon_preview_actual=gate_decon_preview_drawn
	var gate_b_floor_ok: bool = gh_gate_b_environment_visual.draw_floor(self,snapshot,scene,current_key,camera,viewport_rect,grey_hive_state_props,gh_supply_crate_visual,whole_gate_b or gate_decon_floor_drawn)
	if gate_decon_mode and snapshot.sceneId=='gh_gate_b': gate_decon_functional_floor_ok=gate_b_floor_ok
	if not whole_boiler and not cw_early_owned:
		cw_boiler_environment_visual.draw_floor(self,snapshot,scene,current_key,camera,viewport_rect,cw_furnace_visual)
	elif cw_early_owned and snapshot.sceneId=='cw_boiler_chamber':
		_cw_early_boiler_cues()
	if not whole_conveyor and not cw_early_owned:
		cw_conveyor_environment_visual.draw_floor(self,snapshot,scene,current_key,camera,viewport_rect)
	if not whole_beacon and not utility_new and not hive_full_owned:
		gh_beacon_environment_visual.draw_floor(self,snapshot,scene,current_key,camera,viewport_rect)
	if utility_new and snapshot.sceneId=='gh_beacon':
		if not gh_gate_decon_visual.draw_floor(self,utility_plan): utility_draw_failed=true
	if not hive_full_owned: mh_pump_environment_visual.draw_land_material(self,snapshot,scene,current_key,camera,viewport_rect,whole_pump)
	var gh_power_floor_packet = gh_power_floor_material_visual.build_packet(snapshot,scene,current_key,camera)
	if not whole_power and not whole_gate_a and not dense_mode:
		gh_power_floor_material_visual.draw_packet(self,gh_power_floor_packet,snapshot,scene,current_key,camera,get_viewport_rect())
	var dense_neighbor_drawn = false
	if not gh_power_floor_packet.is_empty():
		dense_neighbor_drawn = gh_power_gate_neighbor_visual.draw_floor(self,snapshot,scene,current_key,camera,gh_power_floor_material_visual,not dense_mode)
	if dense_mode:
		var static_ready = true
		if snapshot.sceneId=='gh_power_room':
			for light in ['unknown','on','off']:
				if gh_power_environment_visual.build_packets(scene,snapshot,current_key,camera,light).is_empty(): static_ready = false
		if static_ready: dense_region_plan = gh_dense_region_composition.prepare(snapshot,scene,current_key,scene_hash,camera,gh_power_floor_material_visual,gh_power_gate_neighbor_visual,gh_dense_service_wing_visual,dense_neighbor_drawn)
		if not dense_region_plan.is_empty():
			gh_dense_region_composition.draw(self,dense_region_plan)
			dense_visibility_context = {'world':snapshot.worldId,'scene':snapshot.sceneId,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'region':dense_region_plan.region.id,'player':snapshot.player.entityId,'hash':scene_hash,'entities':snapshot.denseEntityRegions.duplicate(true),'escapes':snapshot.densePartitionEscapeIds.duplicate(true)}
	if _gate_shaft_portal_required() and (not dense_mode or not dense_region_plan.is_empty()):
		var portal_plan: Dictionary = _gate_shaft_portal_plan()
		if not portal_plan.is_empty() and gh_shaft_lift_visual.draw_ground(self,portal_plan):
			gate_shaft_portal_frame={'key':current_key,'hash':scene_hash,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':snapshot.player.entityId}
			gate_shaft_portal_frame_at=Time.get_ticks_msec()
	var gh_sentinel_floor_packet = gh_sentinel_floor_material_visual.build_packet(snapshot,scene,current_key,camera)
	if not whole_sentinel and not hive_full_owned:
		gh_sentinel_floor_material_visual.draw_packet(self,gh_sentinel_floor_packet,snapshot,scene,current_key,camera,get_viewport_rect())
	var gh_deep_decon_floor_packet = gh_deep_decon_floor_material_visual.build_packet(snapshot,scene,current_key,scene_hash,camera)
	var gh_deep_decon_floor_drawn: bool = false if whole_decon or gate_decon_floor_drawn else gh_deep_decon_floor_material_visual.draw_packet(self,gh_deep_decon_floor_packet,snapshot,scene,current_key,scene_hash,camera,get_viewport_rect())
	var cw_pressure_floor_packet = cw_pressure_floor_material_visual.build_packet(snapshot,scene,current_key,camera)
	var cw_pressure_floor_drawn: bool = false if whole_pressure or cw_early_owned else cw_pressure_floor_material_visual.draw_packet(self,cw_pressure_floor_packet,snapshot,scene,current_key,camera,get_viewport_rect())
	if cw_pressure_floor_drawn:
		cw_pressure_environment_visual.draw_floor_treatment(self,scene,snapshot,current_key,camera,get_viewport_rect())
	var mh_fog_pier_ground_packet = mh_fog_pier_ground_visual.build_packet(snapshot,scene,current_key,camera)
	if not whole_fog_pier and not hive_full_owned:
		mh_fog_pier_ground_visual.draw_packet(self,mh_fog_pier_ground_packet,snapshot,scene,current_key,camera,get_viewport_rect())
	var mh_fog_pier_stone_material_packet = mh_fog_pier_stone_material_visual.build_packet(snapshot,scene,current_key,camera)
	if not whole_fog_pier and not hive_full_owned:
		mh_fog_pier_stone_material_visual.draw_packet(self,mh_fog_pier_stone_material_packet,snapshot,scene,current_key,camera,get_viewport_rect())
	var mh_fog_pier_layered_packet: Dictionary = mh_fog_pier_layered_fog_visual.build_packet(snapshot,scene,current_key,camera,mh_fog_pier_phase_seconds,reduced_motion)
	var mh_fog_pier_fog_viewport: Rect2 = get_viewport_rect()
	var mh_fog_pier_back_drawn: bool = mh_fog_pier_layered_fog_visual.draw_phase(self,mh_fog_pier_layered_packet,snapshot,scene,current_key,camera,mh_fog_pier_phase_seconds,reduced_motion,mh_fog_pier_fog_viewport,'back')
	var gh_bio_floor_packets = gh_bio_floor_material_visual.build_packets(snapshot,bio_visual_scene,current_key,camera,gh_bio_wall_modules_visual)
	if not whole_bio and not utility_new and not hive_full_owned:
		gh_bio_floor_material_visual.draw_tiles(self,gh_bio_floor_packets,get_viewport_rect())
	if utility_new and snapshot.sceneId=='gh_bio_isolation':
		if not gh_gate_decon_visual.draw_floor(self,utility_plan): utility_draw_failed=true
	if not gh_r2_plate_drawn and not whole_gh:
		var gh_entry_floor_packets = gh_entry_floor_prepared if gh_entry_owned else gh_entry_floor_material_visual.build_packets(snapshot,scene,current_key,camera)
		gh_entry_floor_material_visual.draw_tiles(self,gh_entry_floor_packets,get_viewport_rect())
		gh_entry_ambient_visual.draw_floor(self,snapshot,scene,camera)
	if gh_supply_crate_visual!=null:
		gh_supply_crate_visual.draw_beacon_contact_shadow(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	if registered_props!=null:
		registered_props.draw_beacon_contact_shadow(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	var rs_active = _rs_environment_context()
	var rs_terminal_feet: Array = []
	var rs_back_packets: Array = []
	var rs_module_packets: Array = []
	var rs_service_pipe_packet: Dictionary = {}
	var gh_bio_wall_packets: Array = []
	var gh_bio_wall_shadow_packets: Array = []
	var gh_bio_wall_active: bool = false
	var gh_entry_wall_packets: Array = []
	var gh_entry_wall_shadow_packets: Array = []
	var gh_entry_wall_suppressed_ids: Array = []
	var gh_entry_wall_active: bool = false
	gh_entry_wall_draw_failed = false
	var gh_bio_service_pipe_packet: Dictionary = {}
	var gh_bio_clutter_packet: Dictionary = {}
	var gh_entry_clutter_packet: Dictionary = {}
	if rs_active:
		rs_terminal_feet = rs_pre_feet
		rs_back_packets = rs_pre_back
		rs_active = not rs_back_packets.is_empty()
	if rs_active:
		rs_module_packets = rs_bulkhead_modules_visual.build_packets(snapshot,scene,current_key,camera,rs_environment_visual) if rs_owned.is_empty() else []
		if not rs_module_packets.is_empty():
			rs_back_packets = rs_bulkhead_modules_visual.filter_legacy_bulkhead_packets(rs_back_packets)
		rs_service_pipe_packet = rs_service_pipe_visual.build_packet(snapshot,scene,current_key,camera,rs_environment_visual)
	gh_bio_wall_packets = gh_bio_wall_modules_visual.build_packets(snapshot,bio_visual_scene,current_key,camera) if not hive_full_owned else []
	if gh_bio_wall_packets.size() == 72:
		gh_bio_wall_shadow_packets = gh_bio_wall_modules_visual.build_shadow_packets(bio_visual_scene,camera)
		gh_bio_wall_active = gh_bio_wall_shadow_packets.size() == 6 and is_finite(viewport_rect.size.x) and is_finite(viewport_rect.size.y) and viewport_rect.size.x > 0.0 and viewport_rect.size.y > 0.0
	if not gh_bio_wall_active:
		gh_bio_wall_packets.clear()
		gh_bio_wall_shadow_packets.clear()
	gh_entry_wall_packets = gh_entry_wall_modules_visual.build_packets(snapshot,scene,current_key,camera) if not gh_r2_plate_drawn and not whole_gh else []
	if gh_entry_wall_packets.size() == 63:
		gh_entry_wall_shadow_packets = gh_entry_wall_modules_visual.build_shadow_packets(snapshot,scene,current_key,camera,gh_entry_wall_packets)
	if gh_entry_wall_shadow_packets.size() == 5:
		gh_entry_wall_suppressed_ids = gh_entry_wall_modules_visual.suppressible_collision_ids(snapshot,scene,current_key,camera,gh_entry_wall_packets,gh_entry_wall_shadow_packets,viewport_rect)
		gh_entry_wall_active = gh_entry_wall_suppressed_ids.size() == 5
	if not gh_entry_wall_active:
		gh_entry_wall_packets.clear()
		gh_entry_wall_shadow_packets.clear()
		gh_entry_wall_suppressed_ids.clear()
	var gh_entry_service_pipe_packet: Dictionary = gh_entry_service_pipe_visual.build_packet(snapshot,scene,current_key,camera,gh_entry_wall_modules_visual)
	gh_bio_service_pipe_packet = gh_bio_service_pipe_visual.build_packet(snapshot,bio_visual_scene,current_key,camera,gh_bio_wall_modules_visual)
	gh_bio_clutter_packet = gh_bio_clutter_visual.build_packet(snapshot,bio_visual_scene,current_key,camera,gh_bio_wall_modules_visual)
	var gh_bio_sample_case_packet: Dictionary = gh_bio_sample_case_visual.build_packet(snapshot,bio_visual_scene,current_key,camera,gh_bio_wall_modules_visual)
	gh_entry_clutter_packet = gh_entry_clutter_visual.build_packet(snapshot,scene,current_key,camera,gh_entry_wall_modules_visual)
	var gh_power_console_matches = 0
	var gh_power_console_body: Dictionary = {}
	var gh_power_light_state: String = 'unknown'
	if snapshot.get('worldId')=='grey_hive' and snapshot.get('sceneId')=='gh_power_room' and scene.get('worldId')=='grey_hive' and scene.get('sceneId')=='gh_power_room' and P.counter(snapshot.get('worldEpoch'),1) and current_key==snapshot.worldId+':'+snapshot.sceneId+':'+str(snapshot.worldEpoch):
		var gh_power_presentation = scene.get('presentation',{})
		if gh_power_presentation is Dictionary:
			var gh_power_sprites = gh_power_presentation.get('sprites')
			if P.rows(gh_power_sprites,2048):
				for gh_power_sprite in gh_power_sprites:
					if gh_power_sprite is Dictionary and gh_power_sprite.get('id')=='power_console_off_on':
						gh_power_console_matches += 1
						if gh_power_console_matches==1:
							gh_power_console_body = _gh_console_body(gh_power_sprite)
		if gh_power_console_matches==1 and not gh_power_console_body.is_empty() and gh_power_console_body.get('id')=='power_console_off_on' and gh_power_console_body.get('kind')=='gh_console_body' and gh_power_console_body.get('p') is Vector3 and gh_power_console_body.p==Vector3(4,0,7) and gh_power_console_body.get('data') is Dictionary:
			var validated_console_state = gh_power_console_body.data.get('state')
			if validated_console_state in ['on','off']:
				gh_power_light_state = validated_console_state
	var gh_power_packets = gh_power_environment_visual.build_packets(scene,snapshot,current_key,camera,gh_power_light_state)
	var gh_power_room_active = not gh_power_packets.is_empty()
	var gh_power_pipe_detail_drawn = gh_power_room_active and gh_power_environment_visual.has_pipe_detail(gh_power_packets)
	var cw_pressure_packets = cw_pressure_environment_visual.build_packets(scene,snapshot,current_key,camera)
	var cw_pressure_room_active = not cw_pressure_packets.is_empty()
	var mh_pump_packets = mh_pump_environment_visual.build_packets(scene,snapshot,current_key,camera)
	var mh_pump_room_active = not mh_pump_packets.is_empty()
	var gh_lockdown_wall_drawn_ids: Array = []
	var warden_wall_drawn_ids: Array[String] = []
	var gate_b_cable_receipt: Dictionary = {}
	var gh_sentinel_shell_drawn: bool = false if whole_sentinel or hive_full_owned else gh_sentinel_environment_visual.draw_shell(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	var cw_furnace_heart_shell_drawn: bool = false if whole_heart or cw_later_owned else cw_furnace_heart_environment_visual.draw_shell(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	var cw_shutdown_exit_shell_drawn: bool = false if whole_shutdown or cw_later_owned else cw_shutdown_exit_environment_visual.draw_shell(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	var gh_deep_decon_alcove_receipt: Dictionary = {}
	var gh_exit_barrier_packets: Array = []
	var exit_collision = scene.get('collision')
	if exit_collision is Array:
		var exit_barriers = exit_collision.filter(func(row): return row is Dictionary and row.get('id')=='exit_sealed_extraction_barrier')
		if exit_barriers.size()==1 and not exit_source:
			gh_exit_barrier_packets = gh_exit_sealed_barrier_visual.build_packets(snapshot,scene,current_key,scene_hash,camera,_gh_exit_open_barrier(exit_barriers[0]))
	# Canonical static geometry is display only. No spawns, routes, navigation or hidden objectives.
	for wall in ([] if dense_mode or shaft_mode or hive_full_owned else scene.get('collision',[])):
		if exit_source: continue # Exact static owners and state seal are already prepared together.
		if utility_new and wall is Dictionary and (snapshot.sceneId=='gh_deep_decon' or (snapshot.sceneId=='gh_bio_isolation' and not str(wall.get('id','')).begins_with('gh_bio_isolation_wall_')) or (snapshot.sceneId=='gh_beacon' and (_pair_visibility_scope() or not str(wall.get('id','')).begins_with('beacon_wall_')))): continue
		if lockdown_floor_drawn and wall is Dictionary and lockdown_plan.owners.has(wall.get('id')):
			if gh_gate_decon_visual.draw_owner(self,lockdown_plan,wall.id):
				lockdown_walls.append(wall.id)
				if wall.id.begins_with('lockdown_wall_'): gh_lockdown_wall_drawn_ids.append(wall.id)
			else: lockdown_wall_ok=false
			continue
		if gate_decon_floor_drawn and not utility_new and wall is Dictionary and gate_decon_plan.owners.has(wall.get('id')):
			if snapshot.sceneId=='gh_gate_b' and wall.id!='gh_gate_b_gate_barrier' and not gh_gate_b_environment_visual.draw_wall(self,snapshot,scene,current_key,camera,wall,viewport_rect,gh_gate_b_west_cable_visual,gate_b_cable_receipt,true): gate_decon_wall_ok=false
			if gh_gate_decon_visual.draw_owner(self,gate_decon_plan,wall.id): gate_decon_walls.append(wall.id)
			else: gate_decon_wall_ok=false
			continue
		if wall is Dictionary:
			var static_id = wall.get('id')
			if whole_gh and static_id in ['gh_entry_wall_north','gh_entry_wall_south','gh_entry_wall_west','gh_entry_wall_east_north','gh_entry_wall_east_south']: continue
			if whole_mh and static_id in ['breakwater_wall_north','breakwater_wall_south','breakwater_wall_west_north','breakwater_wall_west_south','breakwater_wall_east_north','breakwater_wall_east_south']: continue
			if whole_cw and static_id in ['cw_entry_foundry_wall_north','cw_entry_foundry_wall_south','cw_entry_foundry_wall_west_north','cw_entry_foundry_wall_west_south','cw_entry_foundry_wall_east_north','cw_entry_foundry_wall_east_south']: continue
			if whole_pressure and static_id in ['cw_pressure_hall_wall_north','cw_pressure_hall_wall_south','cw_pressure_hall_wall_west_north','cw_pressure_hall_wall_west_south','cw_pressure_hall_wall_east_north','cw_pressure_hall_wall_east_south']: continue
			if whole_fog_pier and static_id in ['pier_wall_north','pier_wall_south','pier_wall_west_north','pier_wall_west_south','pier_wall_east_north','pier_wall_east_south']: continue
			if whole_decon and static_id in ['deep_decon_wall_north','deep_decon_wall_south','deep_decon_wall_west_north','deep_decon_wall_west_south','deep_decon_wall_east_north','deep_decon_wall_east_south','deep_decon_safety_alcove_west','deep_decon_safety_alcove_south']: continue
			if whole_boiler and static_id in ['cw_boiler_chamber_wall_north','cw_boiler_chamber_wall_south','cw_boiler_chamber_wall_west_north','cw_boiler_chamber_wall_west_south','cw_boiler_chamber_wall_east_north','cw_boiler_chamber_wall_east_south']: continue
			if whole_gear and static_id in ['cw_gear_shaft_wall_north','cw_gear_shaft_wall_south','cw_gear_shaft_wall_west_north','cw_gear_shaft_wall_west_south','cw_gear_shaft_wall_east_north','cw_gear_shaft_wall_east_south']: continue
			if whole_heart and cw_furnace_heart_environment_visual.owns_collision(wall): continue
			if whole_conveyor and static_id in ['cw_conveyor_bridge_wall_north','cw_conveyor_bridge_wall_south','cw_conveyor_bridge_wall_west_north','cw_conveyor_bridge_wall_west_south','cw_conveyor_bridge_wall_east_north','cw_conveyor_bridge_wall_east_south']: continue
			if whole_regulator and static_id in ['cw_regulator_core_wall_north','cw_regulator_core_wall_south','cw_regulator_core_wall_west_north','cw_regulator_core_wall_west_south','cw_regulator_core_wall_east_north','cw_regulator_core_wall_east_south']: continue
			if whole_forged and static_id in ['cw_forged_guard_arena_wall_north','cw_forged_guard_arena_wall_south','cw_forged_guard_arena_wall_west_north','cw_forged_guard_arena_wall_west_south','cw_forged_guard_arena_wall_east_north','cw_forged_guard_arena_wall_east_south']: continue
			if whole_signal and static_id in ['yard_wall_north','yard_wall_south','yard_wall_west_north','yard_wall_west_south','yard_wall_east_north','yard_wall_east_south']: continue
			if whole_drowned and static_id in ['quay_wall_north','quay_wall_south','quay_wall_west_north','quay_wall_west_south','quay_wall_east_north','quay_wall_east_south']: continue
		if gh_r2_plate_drawn and wall is Dictionary and wall.get('id') in ['gh_entry_wall_north','gh_entry_wall_south','gh_entry_wall_west','gh_entry_wall_east_north','gh_entry_wall_east_south']:
			continue
		if (whole_shutdown or cw_shutdown_exit_shell_drawn) and cw_shutdown_exit_environment_visual.owns_collision(wall):
			continue
		if cw_furnace_heart_shell_drawn and cw_furnace_heart_environment_visual.owns_collision(wall):
			continue
		if (whole_sentinel or gh_sentinel_shell_drawn) and gh_sentinel_environment_visual.owns_collision(wall):
			continue
		if gh_entry_wall_active and wall is Dictionary and typeof(wall.get('id')) == TYPE_STRING and wall.id in gh_entry_wall_suppressed_ids:
			continue
		if gh_bio_wall_active and wall is Dictionary and gh_bio_wall_modules_visual.should_suppress_collision_fallback(wall.get('id'),gh_bio_wall_packets):
			continue
		if gh_power_room_active and wall is Dictionary and gh_power_environment_visual.is_wall_id(wall.get('id')):
			continue
		if cw_pressure_room_active and wall is Dictionary and cw_pressure_environment_visual.is_wall_id(wall.get('id')):
			continue
		if mh_pump_room_active and wall is Dictionary and mh_pump_environment_visual.is_wall_id(wall.get('id')):
			continue
		if wall is Dictionary and P.polygon(wall.get('polygon')):
			if whole_warden and mh_warden_arena_environment_visual.WALLS.has(wall.get('id')):
				continue
			if mh_warden_arena_environment_visual.draw_wall(self,snapshot,scene,current_key,scene_hash,wall,camera,viewport_rect):
				var warden_wall_id: Variant = wall.get('id')
				if typeof(warden_wall_id) == TYPE_STRING:
					warden_wall_drawn_ids.append(warden_wall_id)
					mh_warden_arena_last_draw_report['wall_ids'] = warden_wall_drawn_ids
				continue
			if rs_active and rs_environment_visual.is_structure_id(wall.get('id')):
				continue
			if mh_fog_pier_environment_visual.draw_wall(self,snapshot,scene,current_key,camera,wall,viewport_rect):
				continue
			if whole_beacon and gh_beacon_environment_visual.WALLS.has(wall.get('id')):
				continue
			if utility_new and snapshot.sceneId=='gh_beacon' and beacon_wall_plans.has(wall.get('id')):
				if gh_beacon_environment_visual._paint(self,beacon_wall_plans[wall.id]): utility_beacon_wall_drawn+=1
				else: utility_draw_failed=true
				continue
			if gh_beacon_environment_visual.draw_wall(self,snapshot,scene,current_key,camera,wall,viewport_rect):
				continue
			if cw_conveyor_environment_visual.draw_wall(self,snapshot,scene,current_key,camera,wall,viewport_rect):
				continue
			if cw_boiler_environment_visual.draw_wall(self,snapshot,scene,current_key,camera,wall,viewport_rect):
				continue
			if gh_gate_b_environment_visual.draw_wall(self,snapshot,scene,current_key,camera,wall,viewport_rect,gh_gate_b_west_cable_visual,gate_b_cable_receipt,whole_gate_b):
				continue
			if cw_regulator_environment_visual.draw_wall(self,snapshot,scene,current_key,camera,wall,viewport_rect):
				continue
			if cw_forged_guard_arena_environment_visual.draw_wall(self,snapshot,scene,current_key,camera,wall,viewport_rect):
				continue
			if _cw_arena_known_open_shutter(wall):
				continue
			if mh_drowned_quay_environment_visual.draw_wall(self,snapshot,scene,current_key,camera,wall,viewport_rect):
				continue
			if cw_entry_foundry_environment_visual.draw_wall(self,wall,snapshot,scene,current_key,camera,viewport_rect):
				continue
			if mh_tidal_warehouse_environment_visual.draw_wall(self,warehouse_frame,wall,snapshot,scene,current_key,camera,viewport_rect,mh_warehouse_crates_visual,whole_warehouse):
				continue
			if gh_central_shaft_environment_visual.draw_wall(self,wall,snapshot,scene,current_key,camera,viewport_rect):
				if shaft_deferred_mode and GhCentralShaftEnvironmentVisual.CANONICAL_WALLS.has(wall.get('id')): shaft_legacy_walls.append(wall.id)
				continue
			if whole_exit and gh_exit_environment_visual.EXTERIOR_IDS.has(wall.get('id')):
				continue
			if gh_exit_environment_visual.draw_wall(self,snapshot,scene,current_key,wall,camera,viewport_rect):
				continue
			if mh_signal_yard_environment_visual.draw_wall(self,snapshot,scene,current_key,camera,wall,viewport_rect):
				continue
			if gh_lockdown_environment_visual.draw_wall(self,snapshot,scene,current_key,wall,camera,viewport_rect):
				if typeof(wall.get('id')) == TYPE_STRING:
					gh_lockdown_wall_drawn_ids.append(wall.get('id'))
				continue
			if cw_gear_shaft_environment_visual.draw_wall(self,wall,snapshot,scene,current_key,scene_hash,camera,viewport_rect):
				continue
			if mh_breakwater_environment_visual.draw_wall(self,wall,snapshot,scene,current_key,camera,viewport_rect):
				continue
			if whole_tower:
				continue
			if mh_resonance_tower_environment_visual.draw_wall(self,wall,snapshot,scene,current_key,scene_hash,camera,viewport_rect):
				continue
			if whole_extraction:
				continue
			if mh_extraction_environment_visual.draw_wall(self,wall,snapshot,scene,current_key,scene_hash,camera,viewport_rect):
				continue
			if gate_a_background and not gate_wall_plan.is_empty() and gate_wall_plan.owners.has(wall.get('id')):
				if wall.get('id')!='gh_gate_a_closed_barrier' or not _gh_gate_a_open_barrier(wall):
					gh_power_gate_neighbor_visual.draw_owner(self,gate_wall_plan,wall.id)
				continue
			if whole_gate_a and not gh_gate_a_environment_visual._definition(wall.get('id')).is_empty():
				continue
			if gh_gate_a_environment_visual.draw_wall(self,snapshot,scene,current_key,camera,wall,viewport_rect):
				continue
			if _gh_gate_a_open_barrier(wall):
				continue
			if _gh_gate_b_open_barrier(wall):
				continue
			if _gh_lockdown_open_seal(wall):
				continue
			if _gh_exit_open_barrier(wall):
				continue
			if gh_exit_barrier_packets.size()==15 and gh_exit_sealed_barrier_visual.matches_barrier(wall):
				continue
			if gh_deep_decon_environment_visual.draw_wall(self,snapshot,scene,current_key,scene_hash,camera,wall,viewport_rect,gh_deep_decon_floor_material_visual,gh_deep_decon_floor_packet,gh_deep_decon_floor_drawn):
				continue
			if gh_deep_decon_alcove_visual.draw_or_consume(self,snapshot,scene,current_key,scene_hash,camera,wall,viewport_rect,gh_deep_decon_alcove_receipt,gh_deep_decon_floor_material_visual,gh_deep_decon_floor_packet,gh_deep_decon_floor_drawn):
				continue
			_poly(wall.polygon,0,Color('#4b5659'),true)
			_poly(wall.polygon,0,Color('#819092'),false)
	if snapshot.worldId=='grey_hive' and snapshot.sceneId in ['gh_central_shaft','gh_lockdown'] and _shaft_lockdown_gate_closed()==true:
		var seal: Dictionary = _shaft_lockdown_seal_plan()
		if not seal.is_empty() and gh_gate_decon_visual.draw_floor(self,seal):
			shaft_lockdown_seal_frame={'key':current_key,'hash':scene_hash,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':snapshot.player.entityId}
			shaft_lockdown_seal_frame_at=Time.get_ticks_msec()
	if lockdown_mode and lockdown_floor_drawn and lockdown_wall_ok and lockdown_walls.size()==lockdown_plan.owners.size():
		lockdown_frame={'key':current_key,'hash':scene_hash,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':snapshot.player.entityId}
		lockdown_frame_at=Time.get_ticks_msec()
	if gate_decon_mode and not utility_new and gate_decon_floor_drawn and gate_decon_functional_floor_ok and gate_decon_wall_ok and gate_decon_walls.size()==gate_decon_plan.owners.size() and gate_decon_preview_drawn:
		gate_decon_frame={'key':current_key,'hash':scene_hash,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':snapshot.player.entityId,'portal':gate_decon_preview_actual}
		gate_decon_frame_at=Time.get_ticks_msec()
	if gh_bio_wall_active:
		if _bio_visibility_scope():
			for points in utility_plan.bio_atlas.shadows:
				draw_colored_polygon(points,Color(0.015,0.025,0.03,gh_bio_wall_modules_visual.SHADOW_ALPHA))
		elif not gh_bio_wall_modules_visual.draw_contact_shadows(self,gh_bio_wall_shadow_packets,viewport_rect) and utility_new:
			utility_draw_failed=true
	if gh_entry_wall_active and not gh_entry_wall_modules_visual.draw_contact_shadows(self,gh_entry_wall_shadow_packets,viewport_rect,snapshot,scene,current_key,camera,gh_entry_wall_packets):
		gh_entry_wall_draw_failed = true
	if rs_active:
		_draw_rs_packets(rs_back_packets,'back',rs_material_plan,whole_rs,rs_owned)
	for region in ([] if dense_mode else scene.get('terrainRegions',[])):
		if hive_full_owned and snapshot.sceneId=='mh_pump_station' and region is Dictionary and region.get('id')=='mh_pump_east_channel_water': continue
		if region is Dictionary and P.polygon(region.get('polygon')):
			if mh_drowned_quay_environment_visual.draw_water(self,snapshot,scene,current_key,camera,region,viewport_rect):
				continue
			var tag = region.get('tag','')
			var pump = snapshot.get('mistHarborPump')
			var drained = pump is Dictionary and pump.get('state')=='drained' and region.get('id') in ['mh_pump_east_channel_water','mh_water_depth_region']
			var c = Color(0.16,0.4,0.5,0.16 if drained else 0.32) if 'water' in tag else Color(0.36,0.29,0.2,0.24)
			_poly(region.polygon,0,c,true)
	var cw_conveyor_belt_base_drawn: bool = cw_conveyor_environment_visual.draw_conveyor(self,snapshot,scene,current_key,camera,viewport_rect)
	if cw_conveyor_belt_base_drawn:
		var cw_conveyor_belt_dust_gate: Dictionary = _cw_conveyor_belt_dust_gate()
		if not cw_conveyor_belt_dust_gate.is_empty():
			cw_conveyor_belt_dust_visual.draw_dust(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect,cw_conveyor_belt_dust_gate)
	if not rs_active and not _region_visibility_scope():
		_supports()
		_surfaces()
	var gh_lockdown_dressing_packets: Array = []
	var gh_lockdown_dressing_candidates: Array = gh_lockdown_environment_visual.build_dressing(snapshot,scene,current_key,camera,viewport_rect)
	for gh_lockdown_packet_variant in gh_lockdown_dressing_candidates:
		if not gh_lockdown_packet_variant is Dictionary:
			continue
		var gh_lockdown_packet: Dictionary = gh_lockdown_packet_variant
		if gh_lockdown_environment_visual.draw_contact_shadow(self,gh_lockdown_packet,snapshot,scene,current_key,camera,viewport_rect):
			gh_lockdown_dressing_packets.append(gh_lockdown_packet)
	if mh_pump_room_active:
		var mh_pump_channel_material_drawn: bool = mh_pump_environment_visual._material_context_valid(self,snapshot,scene,current_key,camera,viewport_rect) if hive_full_owned else mh_pump_environment_visual.draw_channel_material(self,snapshot,scene,current_key,camera,viewport_rect)
		if mh_pump_channel_material_drawn:
			var mh_pump_channel_flow_gate: Dictionary = _mh_pump_flow_gate()
			if not mh_pump_channel_flow_gate.is_empty():
				mh_pump_channel_flow_visual.draw_flow(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect,mh_pump_channel_flow_gate)
		_draw_mh_pump_packets(mh_pump_packets,whole_pump or hive_full_owned)
	if not rs_service_pipe_packet.is_empty() and _rect_intersects(viewport_rect,rs_service_pipe_packet.get('shadow_rect',Rect2())):
		rs_service_pipe_visual.draw_contact_shadow(self,rs_service_pipe_packet)
	if not gh_bio_service_pipe_packet.is_empty() and _rect_intersects(viewport_rect,gh_bio_service_pipe_packet.get('shadow_rect',Rect2())):
		gh_bio_service_pipe_visual.draw_contact_shadow(self,gh_bio_service_pipe_packet,viewport_rect)
	if not gh_entry_service_pipe_packet.is_empty() and not gh_entry_service_pipe_visual.draw_contact_shadow(self,gh_entry_service_pipe_packet,viewport_rect,snapshot,scene,current_key,camera,gh_entry_wall_modules_visual):
		gh_entry_service_pipe_packet.clear()
	if not gh_bio_sample_case_packet.is_empty() and not gh_bio_sample_case_visual.draw_contact_shadow(self,gh_bio_sample_case_packet,viewport_rect):
		gh_bio_sample_case_packet.clear()
	if not gh_bio_clutter_packet.is_empty() and _rect_intersects(viewport_rect,gh_bio_clutter_packet.get('shadow_rect',Rect2())):
		gh_bio_clutter_visual.draw_contact_shadow(self,gh_bio_clutter_packet,viewport_rect)
	if not gh_entry_clutter_packet.is_empty() and not gh_entry_clutter_visual.draw_contact_shadow(self,gh_entry_clutter_packet,viewport_rect,snapshot,scene,current_key,camera,gh_entry_wall_modules_visual):
		gh_entry_clutter_packet.clear()
	var mh_fog_pier_mid_drawn: bool = mh_fog_pier_layered_fog_visual.draw_phase(self,mh_fog_pier_layered_packet,snapshot,scene,current_key,camera,mh_fog_pier_phase_seconds,reduced_motion,mh_fog_pier_fog_viewport,'mid')
	if not mh_fog_pier_layered_packet.is_empty() and mh_fog_pier_back_drawn and mh_fog_pier_mid_drawn:
		mh_fog_pier_defer_labels = true
	gh_deep_decon_air_visual.draw_air(self,snapshot,scene,current_key,camera,viewport_rect)
	if mh_tidal_warehouse_environment_visual.draw_bank(self,warehouse_frame,'warehouse_inner_fog_bank',snapshot,scene,current_key,camera,viewport_rect):
		warehouse_consumed_ids.append('warehouse_inner_fog_bank')
	var yard_drawn_bank_ids: Array = []
	if mh_signal_yard_environment_visual.draw_back_bank(self,snapshot,scene,current_key,camera,viewport_rect):
		yard_drawn_bank_ids.append('yard_north_fog_bank')
	mh_signal_yard_environment_visual.draw_air(self,snapshot,scene,current_key,camera,viewport_rect)
	var gh_central_shaft_pipe_packet: Dictionary = gh_central_shaft_environment_visual.build_pipe_packet(snapshot,scene,current_key,camera,viewport_rect)
	if not gh_central_shaft_pipe_packet.is_empty() and (not shaft_mode or shaft_plan.upper):
		var pipe_shadow_ok: bool = gh_central_shaft_environment_visual.draw_pipe_shadow(self,gh_central_shaft_pipe_packet,snapshot,scene,current_key,camera,viewport_rect)
		if shaft_mode or shaft_deferred_mode: shaft_pipe_ok=pipe_shadow_ok
	var quay_bank_drawn_ids: Array = []
	if mh_drowned_quay_environment_visual.draw_back_bank(self,snapshot,scene,current_key,camera,viewport_rect):
		quay_bank_drawn_ids.append('quay_north_fog_bank')
	var breakwater_bank_drawn_ids: Array[String] = mh_breakwater_environment_visual.draw_bank(self,'back',snapshot,scene,current_key,camera,viewport_rect)
	var resonance_tower_bank_drawn_ids: Array[String] = mh_resonance_tower_environment_visual.draw_back_bank(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	var extraction_bank_drawn_ids: Array[String] = mh_extraction_environment_visual.draw_bank(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	mh_extraction_environment_visual.draw_air(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	var warden_back_air_report: Dictionary = mh_warden_arena_environment_visual.draw_back_air(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	var warden_consumed_ids: Array = warden_back_air_report.get('bank_ids',[])
	mh_warden_arena_last_draw_report['bank_ids'] = warden_consumed_ids
	mh_warden_arena_last_draw_report['ambient_polygons'] = warden_back_air_report.get('ambient_polygons',0)
	var objects: Array = []
	if hive_full_owned:
		for index in range(hive_full_plan.walls.size()):
			var piece: Dictionary=hive_full_plan.walls[index]
			objects.append({'id':'hive-current:'+str(index),'p':piece.worldAnchor,'kind':'hive_route_piece','data':piece})
	if not hive_route_plan.is_empty():
		for index in range(hive_route_plan.walls.size()):
			var piece: Dictionary=hive_route_plan.walls[index]
			objects.append({'id':'hive-route:'+str(index),'p':piece.worldAnchor,'kind':'hive_route_piece','data':piece})
	if not hive_boundary_plan.is_empty():
		for index in range(hive_boundary_plan.walls.size()):
			var piece: Dictionary=hive_boundary_plan.walls[index]
			objects.append({'id':'hive-boundary:'+str(index),'p':piece.worldAnchor,'kind':'hive_route_piece','data':piece})
	if utility_new:
		for index in range(utility_plan.pieces.size()):
			var piece: Dictionary = utility_plan.pieces[index]
			objects.append({'id':'utility:'+str(index),'p':piece.anchor,'kind':'gh_utility_piece','data':piece})
	var exit_visible_props:=0
	if exit_source:
		for index in range(exit_source_plan.pieces.size()):
			var piece: Dictionary=exit_source_plan.pieces[index]
			objects.append({'id':'exit-source:'+str(index),'p':piece.anchor,'kind':'gh_exit_source_piece','data':piece})
		for packet in exit_source_plan.props:
			if _pair_visibility_scope() and not is_dense_visible_position(packet.p): continue
			draw_colored_polygon(packet.shadow,Color(0.025,0.035,0.038,packet.alpha))
			objects.append({'id':packet.id,'p':packet.p,'kind':'gh_exit_source_prop','data':packet})
			exit_visible_props+=1
	if shaft_mode:
		for plan_index in range(shaft_plan.plans.size()):
			var plan: Dictionary = shaft_plan.plans[plan_index]
			for piece_index in range(plan.depth.size()):
				var piece: Dictionary = plan.depth[piece_index]
				if not piece.get('worldAnchor') is Vector3:
					shaft_draw_failed=true
					continue
				objects.append({'id':'shaft_piece:'+str(plan_index)+':'+str(piece_index),'p':piece.worldAnchor,'kind':'gh_shaft_piece','data':{'plan':plan,'index':piece_index}})
			for fixture in plan.get('fixtures',[]):
				if not fixture is Dictionary or not fixture.get('worldAnchor') is Vector3: shaft_draw_failed=true; continue
				objects.append({'id':'shaft_fixture:'+str(plan_index)+':'+str(objects.size()),'p':fixture.worldAnchor,'kind':'gh_shaft_fixture','data':{'fixture':fixture,'texture':plan.get('atlas')}})
	if dense_mode and not dense_region_plan.is_empty():
		for piece in dense_region_plan.walls:
			var footprint: Rect2 = piece.world
			objects.append({'id':'dense_boundary:'+str(objects.size()),'p':Vector3(footprint.get_center().x,0,footprint.get_center().y),'kind':'gh_dense_boundary','data':{'points':piece.points,'uv':piece.uv,'texture':dense_region_plan.wall_texture}})
	if not gh_power_floor_packet.is_empty():
		var dense_texture = gh_power_gate_neighbor_visual._wall_texture()
		var active_parts: Array = gh_dense_service_wing_visual.prepare(snapshot,scene,current_key,scene,scene_hash,0.0,camera,dense_texture)
		for part in active_parts:
			var selected: Dictionary = _dense_visible_partition(part,dense_region_plan) if dense_mode else part
			if not selected.is_empty(): objects.append({'id':part.id,'p':part.p,'kind':'gh_dense_partition','data':selected})
		if dense_neighbor_drawn:
			var neighbor = gh_power_gate_neighbor_visual._neighbor(snapshot.sceneId)
			if neighbor!=null:
				var offset = 20.0 if snapshot.sceneId=='gh_power_room' else -20.0
				var neighboring_parts: Array = gh_dense_service_wing_visual.prepare(snapshot,scene,current_key,neighbor.current,neighbor.source_hash,offset,camera,dense_texture)
				for part in neighboring_parts:
					var selected: Dictionary = _dense_visible_partition(part,dense_region_plan) if dense_mode else part
					if not selected.is_empty(): objects.append({'id':'neighbor:'+part.id,'p':part.p,'kind':'gh_dense_partition','data':selected})
				dense_frame_ready = not active_parts.is_empty() and not neighboring_parts.is_empty()
	for barrier_packet in gh_exit_barrier_packets:
		gh_exit_sealed_barrier_visual.draw_shadow(self,barrier_packet,viewport_rect)
		objects.append({'id':barrier_packet.id,'p':barrier_packet.p,'kind':'gh_exit_barrier_module','data':barrier_packet})
	if not gh_bio_sample_case_packet.is_empty():
		objects.append({'id':gh_bio_sample_case_packet.id,'p':gh_bio_sample_case_packet.p,'kind':'gh_bio_sample_case','data':gh_bio_sample_case_packet})
	var gh_exit_dressing_packets: Array[Dictionary] = gh_exit_environment_visual.build_dressing(snapshot,scene,current_key,camera,viewport_rect)
	for gh_exit_shadow_packet: Dictionary in gh_exit_dressing_packets:
		gh_exit_environment_visual.draw_contact_shadow(self,gh_exit_shadow_packet,snapshot,scene,current_key,camera,viewport_rect)
	var gh_exit_grounding_ids: Array[String] = _gh_exit_body_grounding_ids()
	if exit_source:
		for body_id in gh_exit_grounding_ids:
			if _pair_visibility_scope():
				var rows=scene.get('presentation',{}).get('sprites',[]).filter(func(row): return row is Dictionary and row.get('id')==body_id)
				if rows.size()!=1 or not rows[0].get('position') is Array or rows[0].position.size()!=3 or not is_dense_visible_position(Vector3(rows[0].position[0],rows[0].position[1],rows[0].position[2])): continue
			if exit_source_plan.grounding.has(body_id): draw_colored_polygon(exit_source_plan.grounding[body_id],Color(0.025,0.035,0.038,0.16))
	else:
		gh_exit_environment_visual.draw_body_grounding(self,gh_exit_grounding_ids,snapshot,scene,current_key,camera,viewport_rect)
	if _bio_visibility_scope():
		for piece in utility_plan.bio_atlas.pieces:
			objects.append({'id':'bio-region:'+str(objects.size()),'p':piece.anchor,'kind':'gh_bio_region_atlas','data':piece})
	elif not whole_bio:
		for module_packet in gh_bio_wall_packets:
			objects.append({'id':module_packet.id,'p':module_packet.p,'kind':'gh_bio_wall_module','data':module_packet})
	for entry_wall_packet in gh_entry_wall_packets:
		objects.append({'id':entry_wall_packet.id,'p':entry_wall_packet.p,'kind':'gh_entry_wall_module','data':entry_wall_packet})
	if not whole_rs:
		for module_packet in rs_module_packets:
			objects.append({'id':module_packet.id,'p':module_packet.p,'kind':'rs_bulkhead_module','data':module_packet})
	for prop in scene.get('presentation',{}).get('sprites',[]):
		if not prop is Dictionary or not prop.get('position') is Array or prop.position.size()!=3 or typeof(prop.get('assetId'))!=TYPE_STRING:
			continue
		if gate_b_cable_receipt.size() == 1 and gate_b_cable_receipt.has('pipe_submitted') and typeof(gate_b_cable_receipt.get('pipe_submitted')) == TYPE_BOOL and gate_b_cable_receipt.get('pipe_submitted') == true and gh_gate_b_west_cable_visual.matches_authored_row(prop):
			continue
		if whole_decon and prop.get('id') in ['deep_decon_wall_tiles','deep_decon_service_pipes']:
			continue
		if whole_pressure and prop.get('id') in ['cw_pressure_pipe_run_north','cw_pressure_pipe_run_south']:
			continue
		if gh_power_room_active and gh_power_pipe_detail_drawn and prop.get('id')=='power_room_pipes':
			continue
		if cw_pressure_room_active and cw_pressure_environment_visual.has_pipe_detail(cw_pressure_packets,str(prop.get('id',''))):
			continue
		if rs_active and not rs_service_pipe_packet.is_empty() and rs_service_pipe_visual.matches_authored_row(prop):
			objects.append({'id':prop.id,'p':rs_service_pipe_packet.p,'kind':'rs_service_pipe','data':rs_service_pipe_packet})
			continue
		if not gh_bio_service_pipe_packet.is_empty() and gh_bio_service_pipe_visual.matches_authored_row(prop):
			objects.append({'id':prop.id,'p':gh_bio_service_pipe_packet.p,'kind':'gh_bio_service_pipe','data':gh_bio_service_pipe_packet})
			continue
		if not gh_entry_service_pipe_packet.is_empty() and gh_entry_service_pipe_visual.matches_authored_row(prop):
			objects.append({'id':prop.id,'p':gh_entry_service_pipe_packet.p,'kind':'gh_entry_service_pipe','data':gh_entry_service_pipe_packet})
			continue
		if not gh_bio_clutter_packet.is_empty() and gh_bio_clutter_visual.matches_authored_row(prop):
			objects.append({'id':prop.id,'p':gh_bio_clutter_packet.p,'kind':'gh_bio_clutter','data':gh_bio_clutter_packet})
			continue
		if not gh_entry_clutter_packet.is_empty() and gh_entry_clutter_visual.matches_authored_row(prop):
			objects.append({'id':prop.id,'p':gh_entry_clutter_packet.p,'kind':'gh_entry_clutter','data':gh_entry_clutter_packet})
			continue
		if gh_lockdown_environment_visual.matches_authored_row(prop):
			var gh_lockdown_added_packet: bool = false
			for gh_lockdown_candidate_variant in gh_lockdown_dressing_packets:
				if not gh_lockdown_candidate_variant is Dictionary:
					continue
				var gh_lockdown_candidate: Dictionary = gh_lockdown_candidate_variant
				if gh_lockdown_candidate.get('row_id') == prop.get('id'):
					objects.append({'id':prop.id,'p':gh_lockdown_candidate.p,'kind':'gh_lockdown_dressing','data':gh_lockdown_candidate})
					gh_lockdown_added_packet = true
					break
			if gh_lockdown_added_packet:
				continue
		if not gh_central_shaft_pipe_packet.is_empty() and gh_central_shaft_environment_visual.matches_pipe_row(prop):
			objects.append({'id':prop.id,'p':Vector3(prop.position[0],prop.position[1],prop.position[2]),'kind':'gh_central_pipe','data':gh_central_shaft_pipe_packet})
			continue
		if exit_source and prop.get('id') in ['exit_service_pipes','exit_approved_clutter']: continue
		var gh_exit_added_packet: bool = false
		for gh_exit_packet: Dictionary in gh_exit_dressing_packets:
			if gh_exit_packet.id == prop.get('id') and gh_exit_environment_visual.matches_authored_row(prop,gh_exit_packet.selection_key):
				objects.append({'id':prop.id,'p':gh_exit_packet.position,'kind':'gh_exit_dressing','data':gh_exit_packet})
				gh_exit_added_packet = true
				break
		if gh_exit_added_packet:
			continue
		var console_body: Dictionary = {}
		if prop.get('id')=='power_console_off_on':
			if gh_power_console_matches==1:
				console_body = gh_power_console_body
		else:
			console_body = _gh_console_body(prop)
		if not console_body.is_empty():
			objects.append(console_body)
			continue
		var asset: String = prop.assetId
		var story_prop = _gh_story_prop(prop)
		if not story_prop.is_empty():
			objects.append(story_prop)
			continue
		var terminal = _terminal_position(prop)
		if terminal is Vector3:
			objects.append({'id':prop.id,'p':terminal,'kind':'registered_terminal','data':prop})
			continue
		if prop.get('id') == 'lockdown_back_wall' and gh_lockdown_wall_drawn_ids.size() == 6:
			continue
		if 'enemy' in asset or 'actor' in asset or 'npc' in asset or 'beacon' in asset or 'door' in asset or '.gate_' in asset or 'terminal' in asset or 'floor' in asset or 'wall' in asset or 'fog' in asset or 'water' in asset:
			continue
		if prop.get('layer') in ['visual.props_back','visual.props_dynamic']:
			var p = Vector3(prop.position[0],prop.position[1],prop.position[2])
			objects.append({'id':prop.id,'p':p,'kind':'prop','data':prop})
	var ground_beacon = _ground_beacon_body()
	if not ground_beacon.is_empty():
		objects.append(ground_beacon)
	objects.append({'id':snapshot.player.entityId,'p':P.vector(snapshot.player.transform.positionM),'kind':'player','data':snapshot.player})
	for a in snapshot.actors:
		if not a.active or a.entityId==snapshot.player.entityId:
			continue
		if a.entityType=='enemy.grey_hive.swarm':
			for member in V.swarm_members(a):
				if member.active:
					objects.append({'id':member.memberId,'p':P.vector(member.positionM),'kind':'actor','data':a,'member':member})
			continue
		for n in range(V.actor_positions(a,snapshot).size()):
			var p = P.vector(V.actor_positions(a,snapshot)[n])
			objects.append({'id':a.entityId+':'+str(n),'p':p,'kind':'actor','data':a})
	for d in snapshot.doors:
		objects.append({'id':d.doorId,'p':P.vector(d.transform.positionM),'kind':'door','data':d})
	for i in snapshot.interactables:
		if i.active:
			objects.append({'id':i.entityId,'p':P.vector(i.transform.positionM),'kind':'interaction','data':i})
	var npc = baizhi_placeholder_visual.resolve(snapshot,scene) if baizhi_placeholder_visual!=null else {}
	if not npc.is_empty():
		objects.append({'id':npc.entityId,'p':Vector3(npc.position[0],npc.position[1],npc.position[2]),'kind':'npc','data':npc})
	if gh_power_room_active:
		var power_draw_packets = gh_power_packets.filter(func(packet): return packet is Dictionary and not gh_power_environment_visual.is_wall_id(packet.get('wall_id'))) if whole_power else gh_power_packets
		if dense_mode:
			power_draw_packets = []
			if not dense_region_plan.is_empty():
				for packet in gh_power_packets:
					if packet is Dictionary and not gh_power_environment_visual.is_wall_id(packet.get('wall_id')) and packet.get('footprint') is Rect2 and gh_dense_region_composition._inside(dense_region_plan.local,packet.footprint): power_draw_packets.append(packet)
		_draw_gh_power_packets(power_draw_packets,power_side_plan if power_background else {})
	if cw_pressure_room_active:
		var pressure_draw_packets = cw_pressure_packets.filter(func(packet): return packet is Dictionary and packet.get('role')=='contact_shadow') if whole_pressure else cw_pressure_packets
		_draw_cw_pressure_packets(pressure_draw_packets)
	objects.sort_custom(func(a,b):
		var ay = camera.project(a.p if shaft_mode else Vector3(a.p.x,0,a.p.z)).y
		var by = camera.project(b.p if shaft_mode else Vector3(b.p.x,0,b.p.z)).y
		return a.id<b.id if is_equal_approx(ay,by) else ay<by)
	var interaction_captions = _interaction_captions()
	swarm_art_unready=false
	sentinel_art_unready=false
	sentinel_temporal_drawn=false
	gh_swarm_member_temporal_visual.begin_frame()
	for o in objects:
		if _region_visibility_scope() and o.kind!='hive_route_piece' and not _dense_object_allowed(o): continue
		if shaft_mode and o.kind not in ['player','gh_shaft_piece','gh_shaft_fixture']:
			if (not shaft_plan.upper and not _shaft_lower_region_active()) or (shaft_plan.transit and o.kind in ['actor','npc','interaction']): continue
		var caption: String = interaction_captions.get(o.id,'')
		if rs_active and o.kind=='interaction':
			caption = ''
		_draw_object(o,caption)
	gh_swarm_member_temporal_visual.end_frame()
	var hive_expected: int=(hive_route_plan.get('walls',[]) as Array).size()+(hive_boundary_plan.get('walls',[]) as Array).size()+(hive_full_plan.get('walls',[]) as Array).size()
	if hive_route_floor_ok and not hive_route_draw_failed and hive_route_drawn==hive_expected and (not hive_route_plan.is_empty() or not hive_boundary_plan.is_empty()):
		hive_route_frame={'arenaState':hive_route_plan.get('arenaState'),'token':hive_route_plan.get('token'),'hash':hive_route_plan.get('targetHash'),'sourceHash':scene_hash,'epoch':snapshot.worldEpoch,'player':snapshot.player.entityId,'region':snapshot.get('denseOccupiedRegion'),'visible':snapshot.get('currentVisibleRegions').duplicate(true) if snapshot.get('currentVisibleRegions') is Array else null,'progress':Content.progress(snapshot,snapshot.worldId).duplicate(true),'held':hive_route_hold.duplicate(true) if hive_route_hold is Dictionary else null,'boundary':hive_route_boundary.duplicate(true) if not hive_boundary_plan.is_empty() else null}
		hive_route_frame_at=Time.get_ticks_msec()
		if hive_route_plan.has('reciprocal') and hive_route_view is Dictionary:
			hive_route_cache={'source':snapshot.sceneId,'target':hive_route_view.targetSceneId,'epoch':snapshot.worldEpoch,'sourceHash':scene_hash,'targetHash':hive_route_view.targetSceneSourceSha256,'player':snapshot.player.entityId,'region':snapshot.get('denseOccupiedRegion'),'visible':snapshot.get('currentVisibleRegions').duplicate(true) if snapshot.get('currentVisibleRegions') is Array else null,'token':hive_route_view.token.duplicate(true),'targetPlan':{'floor':hive_route_plan.floor,'walls':hive_route_plan.walls},'sourcePlan':hive_route_plan.reciprocal,'offset':hive_route_plan.offset,'origin':camera.origin,'center':camera.screen_center,'ppm':camera.ppm,'progress':Content.progress(snapshot,snapshot.worldId).duplicate(true),'authorized':hive_route_view.ready or (hive_route_hold is Dictionary and hive_route_hold.get('currentPreviewMatches')==true)}
	if not sentinel_temporal_drawn: gh_sentinel_temporal_visual.reset()
	if exit_source and not exit_source_failed and exit_source_drawn==exit_source_plan.pieces.size()+exit_visible_props:
		exit_source_frame={'key':current_key,'hash':scene_hash,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':snapshot.player.entityId,'visit':exit_source_plan.visit,'open':exit_source_plan.open,'region':exit_source_plan.get('region'),'region_open':exit_source_plan.get('region_open')}
		exit_source_frame_at=Time.get_ticks_msec()
	if utility_new and not utility_draw_failed and utility_drawn_pieces==utility_plan.pieces.size() and (snapshot.sceneId!='gh_bio_isolation' or (gh_bio_wall_active and (utility_bio_wall_drawn==utility_plan.bio_atlas.pieces.size() if _bio_visibility_scope() else utility_bio_wall_drawn==72))) and (snapshot.sceneId!='gh_deep_decon' or gate_decon_floor_drawn) and (snapshot.sceneId!='gh_beacon' or _pair_visibility_scope() or utility_beacon_wall_drawn==6):
		utility_frame={'key':current_key,'hash':scene_hash,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':snapshot.player.entityId,'region':utility_plan.get('region'),'region_open':utility_plan.get('region_open')}
		utility_frame_at=Time.get_ticks_msec()
		if gate_decon_mode and gate_decon_floor_drawn and gate_decon_functional_floor_ok and gate_decon_preview_drawn:
			gate_decon_frame={'key':current_key,'hash':scene_hash,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':snapshot.player.entityId,'portal':gate_decon_preview_actual}
			gate_decon_frame_at=Time.get_ticks_msec()
	if mh_tidal_warehouse_environment_visual.draw_bank(self,warehouse_frame,'warehouse_fog_bank_foreground',snapshot,scene,current_key,camera,viewport_rect):
		warehouse_consumed_ids.append('warehouse_fog_bank_foreground')
	gh_entry_ambient_visual.draw_air(self,snapshot,scene,camera)
	if gh_entry_wall_draw_failed:
		for wall in scene.get('collision',[]):
			if wall is Dictionary and typeof(wall.get('id')) == TYPE_STRING and wall.id in ['gh_entry_wall_north','gh_entry_wall_south','gh_entry_wall_west','gh_entry_wall_east_north','gh_entry_wall_east_south'] and P.polygon(wall.get('polygon')):
				_poly(wall.polygon,0,Color('#4b5659'),true)
				_poly(wall.polygon,0,Color('#819092'),false)
	if rs_active:
		var rs_front_packets = rs_environment_visual.build_packets(scene,snapshot.worldId,snapshot.sceneId,camera,rs_terminal_feet,'front')
		if not rs_module_packets.is_empty():
			rs_front_packets = rs_bulkhead_modules_visual.filter_legacy_bulkhead_packets(rs_front_packets)
		_draw_rs_packets(rs_front_packets,'front',{},whole_rs,rs_owned)
		_draw_rs_interaction_captions(objects,interaction_captions)
		if rs_source_owned_visual.probe_active: rs_source_owned_visual.probe_finish('draw_submitted')
	var mh_fog_pier_front_drawn: bool = mh_fog_pier_layered_fog_visual.draw_phase(self,mh_fog_pier_layered_packet,snapshot,scene,current_key,camera,mh_fog_pier_phase_seconds,reduced_motion,mh_fog_pier_fog_viewport,'front')
	mh_fog_pier_defer_labels = false
	var mh_fog_pier_queued_labels: Array = mh_fog_pier_deferred_labels
	mh_fog_pier_deferred_labels = []
	for mh_fog_pier_label in mh_fog_pier_queued_labels:
		_label(mh_fog_pier_label.p,mh_fog_pier_label.text,mh_fog_pier_label.color)
	breakwater_bank_drawn_ids.append_array(mh_breakwater_environment_visual.draw_bank(self,'front',snapshot,scene,current_key,camera,viewport_rect))
	var mh_fog_pier_suppressed_bank_ids: Array = mh_fog_pier_layered_fog_visual.suppressed_bank_ids(mh_fog_pier_layered_packet,snapshot,scene,current_key,camera,mh_fog_pier_phase_seconds,reduced_motion,[mh_fog_pier_back_drawn,mh_fog_pier_mid_drawn,mh_fog_pier_front_drawn])
	mh_fog_pier_suppressed_bank_ids.append_array(warehouse_consumed_ids)
	mh_fog_pier_suppressed_bank_ids.append_array(yard_drawn_bank_ids)
	mh_fog_pier_suppressed_bank_ids.append_array(quay_bank_drawn_ids)
	mh_fog_pier_suppressed_bank_ids.append_array(breakwater_bank_drawn_ids)
	mh_fog_pier_suppressed_bank_ids.append_array(resonance_tower_bank_drawn_ids)
	mh_fog_pier_suppressed_bank_ids.append_array(extraction_bank_drawn_ids)
	mh_fog_pier_suppressed_bank_ids.append_array(warden_consumed_ids)
	cw_boiler_environment_visual.draw_pressure_steam(self,snapshot,scene,current_key,camera,viewport_rect,reduced_motion)
	_hazards()
	_beacon()
	_boss_warning()
	_action()
	_event_draw()
	if not dense_mode and not shaft_mode: _occlusion()
	_atmosphere(mh_fog_pier_suppressed_bank_ids)
	if not shaft_mode or (shaft_plan.upper and not shaft_plan.transit):
		_world_ui()
		_draw_interaction_focus()
	_draw_aim_reticle()
	if not shaft_mode or (shaft_plan.upper and not shaft_plan.transit): _rear_view()
	if shaft_mode and shaft_ground_ok and shaft_pipe_ok and not shaft_draw_failed:
		shaft_frame_context={'key':shaft_plan.key,'hash':shaft_plan['hash'],'epoch':shaft_plan.epoch,'tick':shaft_plan.tick,'revision':shaft_plan.revision,'player':shaft_plan.player,'mode':'active'}
		shaft_frame_at=Time.get_ticks_msec()
	elif shaft_deferred_mode and shaft_legacy_floor_ok and shaft_pipe_ok and not shaft_draw_failed:
		var legacy_seen: Dictionary = {}
		for id in shaft_legacy_walls: legacy_seen[id]=true
		if shaft_legacy_walls.size()==GhCentralShaftEnvironmentVisual.CANONICAL_WALLS.size() and legacy_seen.size()==GhCentralShaftEnvironmentVisual.CANONICAL_WALLS.size():
			shaft_frame_context={'key':current_key,'hash':scene_hash,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':snapshot.player.entityId,'mode':'deferred'}
			shaft_frame_at=Time.get_ticks_msec()
	if reported_key!=current_key:
		reported_key = current_key
		call_deferred('_report_frame',current_key)

func _drowned_whole_ready() -> bool:
	if not mh_drowned_quay_environment_visual._context(self,snapshot,scene,current_key,camera,viewport_rect): return false
	for row in range(4):
		for column in range(6):
			var x0 = 0.5+column*4.0
			var z0 = 0.5+row*4.0
			if mh_drowned_quay_environment_visual._quad(camera,[x0,z0,minf(23.5,x0+4.0),minf(15.5,z0+4.0)],0).size()!=4: return false
	var ids = ['quay_wall_north','quay_wall_south','quay_wall_west_north','quay_wall_west_south','quay_wall_east_north','quay_wall_east_south']
	for id in ids:
		var rows = scene.collision.filter(func(row): return row is Dictionary and row.get('id')==id)
		if rows.size()!=1 or not rows[0].get('polygon') is Array or rows[0].polygon.size()!=4: return false
		var ground: Array[Vector3] = []
		var cap: Array[Vector3] = []
		for pair in rows[0].polygon:
			ground.append(Vector3(float(pair[0]),0,float(pair[1])))
			cap.append(Vector3(float(pair[0]),0.25,float(pair[1])))
		if mh_drowned_quay_environment_visual._project(camera,ground).size()!=4 or mh_drowned_quay_environment_visual._project(camera,cap).size()!=4: return false
		for i in range(4):
			var j = (i+1)%4
			var face: Array[Vector3] = [ground[i],ground[j],cap[j],cap[i]]
			if mh_drowned_quay_environment_visual._project(camera,face).size()!=4: return false
	return true
func _conveyor_whole_ready() -> bool:
	if snapshot.get('entryToken')!=null or not cw_conveyor_environment_visual._material_context_valid(self,snapshot,scene,current_key,camera,viewport_rect): return false
	var floor_commands: Array[Dictionary] = cw_conveyor_environment_visual._build_floor_commands(camera)
	if not cw_conveyor_environment_visual._commands_valid(floor_commands,32,true): return false
	for id in ['cw_conveyor_bridge_wall_north','cw_conveyor_bridge_wall_south','cw_conveyor_bridge_wall_west_north','cw_conveyor_bridge_wall_west_south','cw_conveyor_bridge_wall_east_north','cw_conveyor_bridge_wall_east_south']:
		var definition: Dictionary = cw_conveyor_environment_visual._wall_definition(id)
		if definition.is_empty(): return false
		var commands: Array[Dictionary] = cw_conveyor_environment_visual._build_wall_commands(camera,definition)
		if not cw_conveyor_environment_visual._commands_valid(commands,4,true): return false
	return true
func _sentinel_whole_ready() -> bool:
	if not is_inside_tree() or is_queued_for_deletion() or not is_visible_in_tree(): return false
	if not gh_sentinel_environment_visual._rect(viewport_rect) or not gh_sentinel_environment_visual._camera(camera,viewport_rect) or not gh_sentinel_environment_visual._layout(snapshot,scene,current_key,scene_hash): return false
	if gh_sentinel_floor_material_visual.build_packet(snapshot,scene,current_key,camera).is_empty(): return false
	var plan: Array[Dictionary] = gh_sentinel_environment_visual._plan(camera)
	if plan.size()!=16: return false
	var original_visible = false
	var emitted_visible = false
	for index in range(plan.size()):
		var command: Dictionary = plan[index]
		var line = index>=14
		var width: float = command.width
		if not is_finite(width) or (line and width<=0.0) or (not line and width!=0.0): return false
		if not gh_sentinel_environment_visual._valid_points(command.points,line) or not gh_sentinel_environment_visual._valid_points(command.original,line): return false
		var box: Rect2 = gh_sentinel_environment_visual._bounds(command.points,width)
		var original_box: Rect2 = gh_sentinel_environment_visual._bounds(command.original,width)
		if not gh_sentinel_environment_visual._rect(box) or not gh_sentinel_environment_visual._rect(original_box): return false
		original_visible = original_visible or gh_sentinel_environment_visual._intersects(original_box,viewport_rect)
		emitted_visible = emitted_visible or gh_sentinel_environment_visual._intersects(box,viewport_rect)
	return original_visible and emitted_visible
func _heart_whole_ready() -> bool:
	if not cw_furnace_heart_environment_visual._canvas_matches(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect) or not cw_furnace_heart_environment_visual._context(snapshot,scene,current_key,scene_hash): return false
	var plan: Array[Dictionary] = cw_furnace_heart_environment_visual._plan(camera)
	if plan.size()!=18: return false
	var vertices = 0
	var any_visible = false
	for command: Dictionary in plan:
		if not cw_furnace_heart_environment_visual._valid_polygon(command.original) or not cw_furnace_heart_environment_visual._valid_polygon(command.points): return false
		vertices += command.points.size()
		var visible: int = cw_furnace_heart_environment_visual._visible(command.points,viewport_rect)
		if visible<0: return false
		any_visible = any_visible or visible>0
	return vertices==74 and any_visible
func _rs_environment_context() -> bool:
	return snapshot.get('worldId')=='return_station' and snapshot.get('sceneId')=='rs_core_room' and scene.get('worldId')=='return_station' and scene.get('sceneId')=='rs_core_room' and current_key=='return_station:rs_core_room:'+str(snapshot.get('worldEpoch'))

func _rs_terminal_feet() -> Array:
	var result: Array = []
	for prop in scene.get('presentation',{}).get('sprites',[]):
		if not prop is Dictionary:
			continue
		var position = _terminal_position(prop)
		if position is Vector3:
			result.append({'id':prop.id,'position':position})
	return result

func _draw_gh_power_packets(packets: Array, side_plan: Dictionary = {}) -> void:
	var side_drawn: Dictionary = {}
	var power_pipe_attempted: bool = false
	var north_attempted: bool = false
	var north_drawn: bool = false
	var south_faces: Dictionary = gh_power_environment_visual.stage_south_low_faces(self,packets)
	for packet in packets:
		if not packet is Dictionary:
			continue
		var wall_id = packet.get('wall_id')
		if not side_plan.is_empty() and wall_id!='gh_power_wall_south' and side_plan.owners.has(wall_id) and packet.get('role') in ['wall_face','wall_cap','cap_edge','panel_seam','fastener']:
			if not side_drawn.has(wall_id):
				gh_power_gate_neighbor_visual.draw_owner(self,side_plan,wall_id)
				side_drawn[wall_id] = true
			continue
		if not side_plan.is_empty() and not south_faces.is_empty() and wall_id=='gh_power_wall_south' and packet.get('role') in ['wall_cap','cap_edge']:
			if not side_drawn.has(wall_id):
				gh_power_gate_neighbor_visual.draw_caps(self,side_plan,wall_id)
				side_drawn[wall_id]=true
			continue
		if not south_faces.is_empty() and packet.get('wall_id') == 'gh_power_wall_south':
			var south_id: Variant = packet.get('id')
			if south_id in south_faces.decorations:
				continue
			if south_faces.groups.has(south_id):
				for piece: Dictionary in south_faces.groups[south_id]:
					draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,south_faces.texture)
				continue
		if packet.get('wall_id')=='gh_power_wall_north' and packet.get('role') in ['wall_face','wall_cap','cap_edge','panel_seam','fastener']:
			if not north_attempted:
				north_attempted = true
				north_drawn = gh_power_environment_visual.draw_north_image(self,scene,snapshot,current_key,camera)
			if north_drawn:
				continue
		if packet.get('role')=='pipe_detail' and packet.get('source_id')=='power_room_pipes':
			if not power_pipe_attempted:
				power_pipe_attempted = true
				if gh_entry_service_pipe_visual!=null:
					gh_entry_service_pipe_visual.draw_power_body(self,snapshot,scene,current_key,scene_hash,camera)
			continue
		var points = packet.get('points')
		var color = packet.get('color')
		if not points is PackedVector2Array or not color is Color:
			continue
		var filled = packet.get('filled')==true
		if filled and points.size()>=3:
			draw_colored_polygon(points,color)
		elif not filled and points.size()>=2:
			var width = float(packet.get('width',1.0))
			if not is_finite(width) or width<=0.0:
				continue
			if points.size()==2:
				draw_line(points[0],points[1],color,width,true)
			else:
				var outline: PackedVector2Array = points.duplicate()
				outline.append(outline[0])
				draw_polyline(outline,color,width,true)

func _cw_early_pressure_pipe(packet: Dictionary) -> bool:
	var points=packet.get('points')
	if not points is PackedVector2Array or points.size()!=2 or not P.number(packet.get('width'),0.001,1000) or not gh_shaft_lift_visual._load_once() or not is_finite(camera.ppm) or camera.ppm<=0: return false
	var direction: Vector2=points[1]-points[0]
	if not direction.is_finite() or direction.length_squared()<=0.000001: return false
	var normal=Vector2(-direction.y,direction.x).normalized()*float(packet.width)*0.5
	var quad=PackedVector2Array([points[0]-normal,points[1]-normal,points[1]+normal,points[0]+normal])
	var repeat=direction.length()/(0.5*camera.ppm)
	var uv=PackedVector2Array([Vector2(0,0),Vector2(repeat,0),Vector2(repeat,1),Vector2(0,1)])
	var previous=texture_repeat;texture_repeat=CanvasItem.TEXTURE_REPEAT_ENABLED
	draw_polygon(quad,PackedColorArray([Color.WHITE]),uv,gh_shaft_lift_visual._steel)
	texture_repeat=previous
	return true

func _draw_cw_pressure_packets(packets: Array) -> void:
	var short_source: Array = packets
	var short_frame: Dictionary = cw_pressure_environment_visual.stage_short_union(short_source,self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	var short_checked: bool = false
	var short_allowed: bool = false
	var cap_images: Dictionary = {}
	var low_face_images: Dictionary = {}
	var end_face_images: Dictionary = {}
	var outer_z_images: Dictionary = {}
	if not short_frame.is_empty():
		packets = short_frame.packets
		short_allowed = cw_pressure_environment_visual.short_union_current(short_frame,short_source,self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
		short_checked = true
		if short_allowed:
			cap_images = cw_pressure_environment_visual.prepare_cap_images(short_source,scene,snapshot,current_key,camera,cw_pressure_floor_material_visual)
			low_face_images = cw_pressure_environment_visual.prepare_low_face_images(packets,scene,snapshot,current_key,camera)
			end_face_images = cw_pressure_environment_visual.prepare_end_face_images(packets,scene,snapshot,current_key,camera,viewport_rect)
			outer_z_images = cw_pressure_environment_visual.prepare_outer_z_images(packets,scene,snapshot,current_key,camera,viewport_rect)
	for packet in packets:
		if not packet is Dictionary:
			continue
		var points = packet.get('points')
		var color = packet.get('color')
		if not points is PackedVector2Array or not color is Color:
			continue
		if cw_early_owned and packet.get('role')=='pipe_detail':
			_cw_early_pressure_pipe(packet)
			continue
		if not short_frame.is_empty() and packet.get('pressure_short_union_v1')==true:
			if not short_checked:
				short_allowed = cw_pressure_environment_visual.short_union_current(short_frame,short_source,self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
				short_checked = true
			if not short_allowed:
				continue
		if packet.get('role')=='wall_face' and not outer_z_images.is_empty() and outer_z_images.groups.has(packet.get('id')):
			for piece: Dictionary in outer_z_images.groups[packet.id]:
				draw_polygon(piece.points,PackedColorArray([Color(1,1,1,color.a)]),piece.uv,outer_z_images.texture)
			continue
		if packet.get('role')=='wall_face' and not end_face_images.is_empty() and end_face_images.groups.has(packet.get('id')):
			var piece: Dictionary = end_face_images.groups[packet.id]
			draw_polygon(piece.points,PackedColorArray([Color(1,1,1,color.a)]),piece.uv,end_face_images.texture)
			continue
		if packet.get('role')=='wall_face' and not low_face_images.is_empty() and low_face_images.groups.has(packet.get('id')):
			for piece in low_face_images.groups[packet.id]:
				draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,low_face_images.texture)
			continue
		if packet.get('role')=='wall_cap' and not cap_images.is_empty() and cap_images.groups.has(packet.get('wall_id')):
			for piece in cap_images.groups[packet.wall_id]:
				draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,cap_images.texture)
			continue
		if cw_early_owned and snapshot.get('sceneId')=='cw_pressure_hall' and packet.get('role') in ['wall_face','wall_cap','wall_edge','panel_seam','fastener']:
			continue
		if packet.get('filled')==true and points.size()>=3:
			draw_colored_polygon(points,color)
		elif points.size()>=2:
			var width = float(packet.get('width',1.0))
			if not is_finite(width) or width<=0.0:
				continue
			if points.size()==2:
				draw_line(points[0],points[1],color,width,true)
			else:
				var outline: PackedVector2Array = points.duplicate()
				outline.append(outline[0])
				draw_polyline(outline,color,width,true)

func _draw_mh_pump_packets(packets: Array, suppress_static: bool = false) -> void:
	if suppress_static:
		packets = packets.filter(func(packet): return packet is Dictionary and not mh_pump_environment_visual.is_wall_id(packet.get('wall_id')))
	else:
		packets = mh_pump_environment_visual.compose_outer_walls(packets,self,snapshot,scene,current_key,camera,viewport_rect)
	for packet in packets:
		if not packet is Dictionary:
			continue
		var points = packet.get('points')
		var color = packet.get('color')
		if not points is PackedVector2Array or not color is Color:
			continue
		if packet.get('filled')==true and points.size()>=3:
			draw_colored_polygon(points,color)
		elif points.size()>=2:
			var width = float(packet.get('width',1.0))
			if not is_finite(width) or width<=0.0:
				continue
			if points.size()==2:
				draw_line(points[0],points[1],color,width,true)
			else:
				var outline: PackedVector2Array = points.duplicate()
				outline.append(outline[0])
				draw_polyline(outline,color,width,true)

func _rs_source_plan_current() -> bool:
	return not rs_source_plan.is_empty() and P.snapshot(snapshot) and rs_source_plan.get('key')==current_key and rs_source_plan.get('epoch')==snapshot.worldEpoch and rs_source_plan.get('tick')==snapshot.serverTick and rs_source_plan.get('revision')==snapshot.authorityRevision and rs_source_plan.get('player')==snapshot.player.entityId and rs_source_plan.get('camera_origin')==camera.origin and rs_source_plan.get('camera_ppm')==camera.ppm and rs_source_plan.get('camera_center')==camera.screen_center

func _rs_source_owner_packet(id: String) -> bool:
	for owner in RsSourceOwnedVisual.OWNERS:
		if id.ends_with(':'+owner) or id.contains(':'+owner+':'): return true
	return false

func _draw_rs_packets(packets: Array, phase: String, material_plan: Dictionary = {}, whole_rs: bool = false, owned: Dictionary = {}) -> void:
	for packet in packets:
		if not packet is Dictionary or packet.get('kind')!='polygon' or not packet.get('points') is PackedVector2Array:
			continue
		if whole_rs and (phase=='front' or not str(packet.get('id','')).begins_with('rs:terminal-')):
			continue
		if not owned.is_empty() and _rs_source_owner_packet(str(packet.get('id',''))):
			var id: String = packet.id
			if rs_source_owned_visual.probe_active and 'service_island_' in id: rs_source_owned_visual.probe_phase='island'
			if phase=='back' and owned.tops.has(id):
				rs_source_owned_visual.draw_surface(self,owned.tops[id])
			elif phase=='front' and owned.fronts.has(id):
				var fade_group = packet.get('fade_group',id)
				rs_source_owned_visual.draw_surface(self,owned.fronts[id],clampf(float(rs_front_alpha.get(fade_group,1.0)),0.25,1.0))
			if rs_source_owned_visual.probe_active: rs_source_owned_visual.probe_phase=''
			continue
		var points: PackedVector2Array = packet.points
		var color: Color = packet.color
		if phase=='back' and packet.get('id')=='rs:terminal-base:mission_terminal_art' and not material_plan.is_empty():
			rs_floor_treatment_visual.draw_material(self,material_plan.pad,material_plan.texture)
			continue
		if phase=='front':
			var fade_group = packet.get('fade_group',packet.get('id'))
			color.a *= clampf(float(rs_front_alpha.get(fade_group,1.0)),0.25,1.0)
			if packet.get('id') in ['rs:front:service_island_north:1','rs:front:service_island_south:1'] and packet.get('island_plinth_pieces') is Array and packet.get('island_plinth_texture') is Texture2D and is_instance_valid(packet.island_plinth_texture):
				for piece in packet.island_plinth_pieces:
					draw_polygon(piece.points,PackedColorArray([Color(1,1,1,color.a)]),piece.uv,packet.island_plinth_texture)
				continue
		if packet.get('filled')==true:
			draw_colored_polygon(points,color)
		else:
			draw_polyline(points,color,maxf(0.5,float(packet.get('width',1.0))),true)

# Screen-space placement only; default clipping and every world anchor remain intact.
func _rs_caption_baseline(original: Vector2, text: String, player_rect: Rect2, safe: Rect2, scale: float) -> Vector2:
	if not is_finite(original.x) or not is_finite(original.y) or not is_finite(scale) or scale<=0.0: return original
	var font=ThemeDB.fallback_font
	var font_size=int(15*text_scale)
	if font==null or font_size<=0: return original
	var width: float=font.get_string_size(text,HORIZONTAL_ALIGNMENT_LEFT,-1,font_size).x
	var ascent: float=font.get_ascent(font_size)
	var height: float=font.get_height(font_size)
	if not is_finite(width) or not is_finite(ascent) or not is_finite(height) or width<=0.0 or height<=0.0 or ascent<0.0 or ascent>height: return original
	var bounds=Rect2(original-Vector2(width*0.5,ascent),Vector2(width,height))
	# A clear caption is never moved, even if old viewport clipping applies.
	if not bounds.intersects(player_rect): return original
	if not player_rect.has_area() or not safe.has_area() or width>safe.size.x or height>safe.size.y: return original
	var gap: float=2.0*scale
	var above=Vector2(clampf(original.x,safe.position.x+width*0.5,safe.end.x-width*0.5),player_rect.position.y-gap-(height-ascent))
	var above_rect=Rect2(above-Vector2(width*0.5,ascent),Vector2(width,height))
	if safe.encloses(above_rect) and not above_rect.intersects(player_rect): return above
	var y=clampf(original.y,safe.position.y+ascent,safe.end.y-(height-ascent))
	var left=Vector2(player_rect.position.x-gap-width*0.5,y)
	var right=Vector2(player_rect.end.x+gap+width*0.5,y)
	var left_rect=Rect2(left-Vector2(width*0.5,ascent),Vector2(width,height))
	var right_rect=Rect2(right-Vector2(width*0.5,ascent),Vector2(width,height))
	var left_ok=safe.encloses(left_rect) and not left_rect.intersects(player_rect)
	var right_ok=safe.encloses(right_rect) and not right_rect.intersects(player_rect)
	if left_ok and right_ok: return left if original.distance_squared_to(left)<=original.distance_squared_to(right) else right
	if left_ok: return left
	if right_ok: return right
	return original

func _draw_rs_interaction_captions(objects: Array, captions: Dictionary) -> void:
	var scale: float = camera.ppm/48.0
	var safe:=Rect2()
	var player_rect:=Rect2()
	if P.snapshot(snapshot) and _rs_environment_context() and camera.initialized and is_finite(scale) and scale>0.0 and viewport_rect==get_viewport_rect() and safe_rect_provider.is_valid() and snapshot.player.currentHp>0:
		var matches: Array=objects.filter(func(o): return o is Dictionary and o.get('kind')=='player' and o.get('id')==snapshot.player.entityId and o.get('data')==snapshot.player and o.get('p')==P.vector(snapshot.player.transform.positionM))
		var candidate=safe_rect_provider.call()
		if matches.size()==1 and candidate is Rect2 and is_finite(candidate.position.x) and is_finite(candidate.position.y) and is_finite(candidate.size.x) and is_finite(candidate.size.y) and candidate.has_area():
			safe=candidate.intersection(viewport_rect)
			var foot: Vector2=camera.project(P.vector(snapshot.player.transform.positionM))
			var factor: float=(104.0/560.0)*scale
			if foot.is_finite() and safe.has_area(): player_rect=Rect2(foot-Vector2(168,551)*factor,Vector2(336,560)*factor)
	for object in objects:
		if not object is Dictionary or object.get('kind')!='interaction' or not captions.has(object.get('id')):
			continue
		var caption = captions.get(object.id,'')
		if typeof(caption)==TYPE_STRING and not caption.is_empty() and object.get('p') is Vector3:
			var original: Vector2=camera.project(object.p)-Vector2(0,20)*scale
			var old_original: Vector2=original
			var badge_placement=false
			if object.get('id')=='rs_world_gate_marker':
				var badge=_rs_gate_badge(object.id)
				if not badge.is_empty() and object.get('data')==badge.row and object.p==P.vector(badge.row.transform.positionM) and not _object_culling_state(object,badge.at).culled:
					var font=ThemeDB.fallback_font
					var font_size=int(15*text_scale)
					if font!=null and font_size>0:
						var descent: float=font.get_height(font_size)-font.get_ascent(font_size)
						var ascent: float=font.get_ascent(font_size)
						var height: float=font.get_height(font_size)
						var width: float=font.get_string_size(caption,HORIZONTAL_ALIGNMENT_LEFT,-1,font_size).x
						var proposed=Vector2(badge.at.x,badge.rect.position.y-4.0*scale-descent)
						var bounds=Rect2(proposed-Vector2(width*0.5,ascent),Vector2(width,height))
						if is_finite(descent) and descent>=0.0 and is_finite(ascent) and is_finite(height) and is_finite(width) and width>0.0 and height>0.0 and viewport_rect.encloses(bounds):
							original=proposed
							badge_placement=true
			var baseline: Vector2=_rs_caption_baseline(original,caption,player_rect,safe,scale) if player_rect.has_area() and safe.has_area() else original
			if badge_placement:
				var font=ThemeDB.fallback_font
				var font_size=int(15*text_scale)
				var width: float=font.get_string_size(caption,HORIZONTAL_ALIGNMENT_LEFT,-1,font_size).x
				var bounds=Rect2(baseline-Vector2(width*0.5,font.get_ascent(font_size)),Vector2(width,font.get_height(font_size)))
				if not viewport_rect.encloses(bounds): baseline=_rs_caption_baseline(old_original,caption,player_rect,safe,scale) if player_rect.has_area() and safe.has_area() else old_original
			_label(baseline,caption)
func _update_rs_front_alpha(delta: float) -> void:
	if not _rs_environment_context():
		rs_front_alpha.clear()
		return
	var feet = _rs_terminal_feet()
	var packets = rs_environment_visual.build_packets(scene,snapshot.worldId,snapshot.sceneId,camera,feet,'front')
	if packets.is_empty():
		rs_front_alpha.clear()
		return
	var factor: float = (104.0/560.0)*(camera.ppm/48.0)
	var foot: Vector2 = camera.project(P.vector(snapshot.player.transform.positionM))
	var player_rect = Rect2(foot-Vector2(168,551)*factor,Vector2(336,560)*factor)
	var module_packets: Array = rs_bulkhead_modules_visual.build_packets(snapshot,scene,current_key,camera,rs_environment_visual) if not _rs_source_plan_current() else []
	if not module_packets.is_empty():
		packets = rs_bulkhead_modules_visual.filter_legacy_bulkhead_packets(packets)
	var next_alpha: Dictionary = rs_environment_visual.front_fade_alphas(packets,rs_front_alpha,player_rect,delta,reduced_motion)
	if not module_packets.is_empty():
		var module_alpha: Dictionary = rs_bulkhead_modules_visual.front_fade_alphas(module_packets,rs_front_alpha,player_rect,delta,reduced_motion,rs_environment_visual)
		for fade_group in module_alpha:
			next_alpha[fade_group] = module_alpha[fade_group]
	rs_front_alpha = next_alpha

func _update_gh_bio_wall_alpha(delta: float) -> void:
	if snapshot.get('worldId') != 'grey_hive' or snapshot.get('sceneId') != 'gh_bio_isolation' or scene.get('worldId') != 'grey_hive' or scene.get('sceneId') != 'gh_bio_isolation' or current_key != 'grey_hive:gh_bio_isolation:' + str(snapshot.get('worldEpoch')):
		gh_bio_wall_alpha.clear()
		return
	if not snapshot.get('player') is Dictionary or not P.transform(snapshot.player.get('transform')):
		gh_bio_wall_alpha.clear()
		return
	var bio_scene: Dictionary = utility_plan.original_bio if _utility_plan_current() and snapshot.sceneId=='gh_bio_isolation' else scene
	var packets: Array = gh_bio_wall_modules_visual.build_packets(snapshot,bio_scene,current_key,camera)
	if packets.size() != 72:
		gh_bio_wall_alpha.clear()
		return
	var factor: float = (104.0/560.0)*(camera.ppm/48.0)
	var foot: Vector2 = camera.project(P.vector(snapshot.player.transform.positionM))
	var player_rect: Rect2 = Rect2(foot-Vector2(168,551)*factor,Vector2(336,560)*factor)
	if not is_finite(player_rect.position.x) or not is_finite(player_rect.position.y) or not is_finite(player_rect.size.x) or not is_finite(player_rect.size.y) or player_rect.size.x <= 0.0 or player_rect.size.y <= 0.0:
		gh_bio_wall_alpha.clear()
		return
	gh_bio_wall_alpha = gh_bio_wall_modules_visual.front_fade_alphas(packets,gh_bio_wall_alpha,player_rect,delta,reduced_motion)

func _update_gh_entry_wall_alpha(delta: float) -> void:
	if not gh_entry_wall_modules_visual.has_valid_context(snapshot,scene,current_key) or not snapshot.get('player') is Dictionary or not P.transform(snapshot.player.get('transform')):
		gh_entry_wall_alpha.clear()
		return
	var packets: Array = gh_entry_wall_modules_visual.build_packets(snapshot,scene,current_key,camera)
	if packets.size() != 63:
		gh_entry_wall_alpha.clear()
		return
	var factor: float = (104.0/560.0)*(camera.ppm/48.0)
	var foot: Vector2 = camera.project(P.vector(snapshot.player.transform.positionM))
	var player_rect: Rect2 = Rect2(foot-Vector2(168,551)*factor,Vector2(336,560)*factor)
	if not is_finite(player_rect.position.x) or not is_finite(player_rect.position.y) or not is_finite(player_rect.size.x) or not is_finite(player_rect.size.y) or player_rect.size.x <= 0.0 or player_rect.size.y <= 0.0:
		gh_entry_wall_alpha.clear()
		return
	var next_alpha: Dictionary = gh_entry_wall_modules_visual.front_fade_alphas(packets,gh_entry_wall_alpha,player_rect,delta,reduced_motion,snapshot,scene,current_key,camera)
	if next_alpha.size() != 63:
		gh_entry_wall_alpha.clear()
		return
	gh_entry_wall_alpha = next_alpha

func _poly(points: Array, height: float, color: Color, filled: bool) -> void:
	var p = PackedVector2Array()
	for a in points:
		p.append(camera.project(Vector3(float(a[0]),height,float(a[1]))))
	if p.size()<3:
		return
	if filled:
		draw_colored_polygon(p,color)
	else:
		p.append(p[0])
		draw_polyline(p,color,1.5,true)
func _ring(center: Vector3, radius: float, color: Color, start: float = 0, span: float = TAU, filled: bool = false) -> void:
	var points: Array = []
	if span<TAU-0.001:
		points.append([center.x,center.z])
	for n in range(49):
		var a = start+span*n/48.0
		points.append([center.x+sin(a)*radius,center.z+cos(a)*radius])
	_poly(points,center.y,color,filled)
func _corridor(p: Vector3, yaw: float, length: float, half_width: float, c: Color) -> void:
	var d = Vector2(sin(yaw),cos(yaw))*length
	var side = Vector2(cos(yaw),-sin(yaw))*half_width
	_poly([[p.x+side.x,p.z+side.y],[p.x+d.x+side.x,p.z+d.y+side.y],[p.x+d.x-side.x,p.z+d.y-side.y],[p.x-side.x,p.z-side.y]],p.y,c,false)
func _label(p: Vector2, text: String, c: Color = Color('#e7edf2')) -> void:
	if mh_fog_pier_defer_labels:
		mh_fog_pier_deferred_labels.append({'p':p,'text':text,'color':c})
		return
	var font = ThemeDB.fallback_font
	var size = int(15*text_scale)
	var width = font.get_string_size(text,HORIZONTAL_ALIGNMENT_LEFT,-1,size).x
	var bounds = Rect2(p-Vector2(width*0.5,font.get_ascent(size)),Vector2(width,font.get_height(size)))
	if viewport_rect.encloses(bounds):
		draw_string(font,p-Vector2(width*0.5,0),text,HORIZONTAL_ALIGNMENT_LEFT,-1,size,c)

func _caption_bounds(p: Vector2, text: String) -> Rect2:
	var font = ThemeDB.fallback_font
	var size = int(15*text_scale)
	if font==null or size<=0 or not p.is_finite(): return Rect2()
	var width = font.get_string_size(text,HORIZONTAL_ALIGNMENT_LEFT,-1,size).x
	return Rect2(p-Vector2(width*0.5,font.get_ascent(size)),Vector2(width,font.get_height(size)))

func _protected_actor_label(id: String, p: Vector2, text: String) -> void:
	_label(p,text)
	if mh_fog_pier_defer_labels: return
	var bounds = _caption_bounds(p,text)
	if bounds.size.x<=0.0 or bounds.size.y<=0.0 or not viewport_rect.encloses(bounds): return
	if not protected_actor_captions.has(id): protected_actor_captions[id] = []
	protected_actor_captions[id].append(bounds)

func _vital_caption_position(id: String, original: Vector2, text: String) -> Vector2:
	if mh_fog_pier_defer_labels or not protected_actor_captions.has(id): return original
	var bounds = _caption_bounds(original,text)
	if bounds.size.x<=0.0 or bounds.size.y<=0.0 or not viewport_rect.encloses(bounds): return original
	var hit = false
	var top = INF
	for protected_rect in protected_actor_captions[id]:
		if bounds.intersects(protected_rect): hit = true
		top = minf(top,protected_rect.position.y)
	if not hit or not is_finite(top): return original
	var candidate = original+Vector2(0,top-2.0*text_scale-bounds.end.y)
	var moved = _caption_bounds(candidate,text)
	if not viewport_rect.encloses(moved): return original
	for protected_rect in protected_actor_captions[id]:
		if moved.intersects(protected_rect): return original
	return candidate
func _interaction_captions() -> Dictionary:
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or current_key!=snapshot.worldId+':'+snapshot.sceneId+':'+str(snapshot.worldEpoch):
		return {}
	var rows = snapshot.get('interactables')
	if not P.rows(rows,2048) or not P.unique(rows,'entityId'):
		return {}
	var labels = {'world_gate':'世界入口','mission_terminal':'任务终端','capability_terminal':'能力终端','save_rest_terminal':'休息 / 保存','scene_transition':'前往下一场景','scene_checkpoint':'检查点','scene_trigger':'场景装置','environment_control':'环境控制','power_console':'恢复电力','door_panel':'门控制','lockdown_terminal':'隔离协议','facility_log':'设施记录','bio_log_terminal':'生物记录','gate_b_panel':'Gate B 控制','extraction_console':'撤离控制','beacon_collect':'收取信标','beacon_mount':'部署信标','beacon':'信标','pump_control':'泵站控制','valve_control':'阀门','coolant_valve':'冷却阀','terminal':'终端'}
	var rs_roles: Dictionary = {
		'rs_world_gate_marker':{'native_kind':'world_gate','authored_kind':'world_gate_marker','position':Vector3(20,0,8),'caption':'世界入口 · '+Content.WORLD_NAMES['grey_hive']},
		'rs_mh_world_gate_marker':{'native_kind':'world_gate','authored_kind':'world_gate_marker','position':Vector3(20,0,12),'caption':'世界入口 · '+Content.WORLD_NAMES['mist_harbor']},
		'rs_cw_world_gate_marker':{'native_kind':'world_gate','authored_kind':'world_gate_marker','position':Vector3(20,0,4),'caption':'世界入口 · '+Content.WORLD_NAMES['clockworks']},
		'rs_storage_inventory_marker':{'native_kind':'storage_inventory_marker','authored_kind':'storage_inventory_marker','position':Vector3(15,0,12),'caption':'背包'}
	}
	var rs_scope: bool = snapshot.worldId=='return_station' and snapshot.sceneId=='rs_core_room'
	var rs_authored_captions: Dictionary = {}
	if rs_scope:
		var authored_rows = scene.get('interactions')
		if P.rows(authored_rows,2048) and P.unique(authored_rows,'id'):
			for authored in authored_rows:
				if typeof(authored.get('id'))!=TYPE_STRING or typeof(authored.get('kind'))!=TYPE_STRING or not rs_roles.has(authored.id):
					continue
				var role: Dictionary = rs_roles[authored.id]
				var position = authored.get('position')
				if typeof(position)!=TYPE_ARRAY or position.size()!=3 or not P.number(position[0]) or not P.number(position[1]) or not P.number(position[2]) or authored.kind!=role.authored_kind:
					continue
				var authored_position = Vector3(float(position[0]),float(position[1]),float(position[2]))
				if authored_position==role.position:
					rs_authored_captions[authored.id] = role.caption
	var captions: Dictionary = {}
	for row in rows:
		if typeof(row.get('entityId'))==TYPE_STRING and typeof(row.get('kind'))==TYPE_STRING and typeof(row.get('active'))==TYPE_BOOL and row.active and P.transform(row.get('transform')):
			if beacon_exit_ordinary_suppressed(row): continue
			if gh_first_pair_portal(row):
				var route_caption=_gh_first_pair_caption(row)
				if not route_caption.is_empty(): captions[row.entityId]=route_caption
				continue
			var caption: String = labels.get(row.kind,'')
			if rs_scope and rs_roles.has(row.entityId):
				var role: Dictionary = rs_roles[row.entityId]
				if row.kind==role.native_kind and P.vector(row.transform.positionM)==role.position and rs_authored_captions.has(row.entityId):
					caption = rs_authored_captions[row.entityId]
			if not caption.is_empty():
				captions[row.entityId] = caption
	return captions
func _prop_placeholder_rect(at: Vector2, ppm: float) -> Rect2:
	if not is_finite(at.x) or not is_finite(at.y) or not is_finite(ppm) or ppm<=0:
		return Rect2()
	var scale = ppm/48.0
	var size = Vector2(32,24)*scale
	var origin = at-Vector2(16,24)*scale
	if not is_finite(scale) or scale<=0 or not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0 or not is_finite(origin.x) or not is_finite(origin.y):
		return Rect2()
	var rect = Rect2(origin,size)
	return rect if is_finite(rect.end.x) and is_finite(rect.end.y) else Rect2()
func _rect_intersects(a: Rect2, b: Rect2) -> bool:
	if not is_finite(a.position.x) or not is_finite(a.position.y) or not is_finite(a.size.x) or not is_finite(a.size.y) or a.size.x<=0 or a.size.y<=0 or not is_finite(b.position.x) or not is_finite(b.position.y) or not is_finite(b.size.x) or not is_finite(b.size.y) or b.size.x<=0 or b.size.y<=0:
		return false
	return a.intersects(b)
func _object_culling_state(o: Dictionary, at: Vector2) -> Dictionary:
	if o.get('kind')=='interaction' and o.get('id')=='rs_world_gate_marker':
		var badge=_rs_gate_badge(o.id)
		if not badge.is_empty() and o.get('data')==badge.row and o.get('p')==P.vector(badge.row.transform.positionM) and at==badge.at:
			return {'furnaceKey':'','culled':not _rect_intersects(viewport_rect,badge.rect),'bodyRect':badge.rect,'fallbackRect':badge.rect}
	if o.get('kind')=='gh_exit_dressing' and o.get('data') is Dictionary and o.data.get('body_rect') is Rect2:
		var exit_body_rect: Rect2 = o.data.body_rect
		var exit_fallback_rect: Rect2 = _prop_placeholder_rect(at,camera.ppm)
		return {'furnaceKey':'','culled':not (_rect_intersects(viewport_rect,exit_body_rect) or _rect_intersects(viewport_rect,exit_fallback_rect)),'bodyRect':exit_body_rect,'fallbackRect':exit_fallback_rect}
	if o.get('kind')=='gh_central_pipe' and o.get('data') is Dictionary and o.data.get('body_rect') is Rect2:
		var central_pipe_body_rect: Rect2 = o.data.body_rect
		var central_pipe_fallback_rect: Rect2 = _prop_placeholder_rect(at,camera.ppm)
		var central_pipe_body_visible: bool = _rect_intersects(viewport_rect,central_pipe_body_rect)
		var central_pipe_fallback_visible: bool = _rect_intersects(viewport_rect,central_pipe_fallback_rect)
		return {'furnaceKey':'','culled':not (central_pipe_body_visible or central_pipe_fallback_visible),'bodyRect':central_pipe_body_rect,'fallbackRect':central_pipe_fallback_rect}
	if o.get('kind')=='gh_entry_wall_module' and o.get('data') is Dictionary and o.data.get('body_rect') is Rect2:
		var entry_module_rect: Rect2 = o.data.body_rect
		return {'furnaceKey':'','culled':not _rect_intersects(viewport_rect,entry_module_rect),'bodyRect':entry_module_rect,'fallbackRect':Rect2()}
	if o.get('kind')=='gh_exit_barrier_module' and o.get('data') is Dictionary and o.data.get('body_rect') is Rect2:
		var exit_barrier_rect: Rect2 = o.data.body_rect
		return {'furnaceKey':'','culled':not _rect_intersects(viewport_rect,exit_barrier_rect),'bodyRect':exit_barrier_rect,'fallbackRect':Rect2()}
	if o.get('kind')=='gh_bio_wall_module' and o.get('data') is Dictionary and o.data.get('body_rect') is Rect2:
		var gh_module_rect: Rect2 = o.data.body_rect
		return {'furnaceKey':'','culled':not _rect_intersects(viewport_rect,gh_module_rect),'bodyRect':gh_module_rect,'fallbackRect':Rect2()}
	if o.get('kind')=='rs_bulkhead_module' and o.get('data') is Dictionary and o.data.get('body_rect') is Rect2:
		var module_rect: Rect2 = o.data.body_rect
		return {'furnaceKey':'','culled':not _rect_intersects(viewport_rect,module_rect),'bodyRect':module_rect,'fallbackRect':Rect2()}
	if o.get('kind')=='rs_service_pipe' and o.get('data') is Dictionary and o.data.get('body_rect') is Rect2:
		var pipe_rect: Rect2 = o.data.body_rect
		return {'furnaceKey':'','culled':not _rect_intersects(viewport_rect,pipe_rect),'bodyRect':pipe_rect,'fallbackRect':Rect2()}
	if o.get('kind')=='gh_bio_service_pipe' and o.get('data') is Dictionary and o.data.get('body_rect') is Rect2:
		var gh_pipe_rect: Rect2 = o.data.body_rect
		var gh_pipe_fallback: Rect2 = _prop_placeholder_rect(at,camera.ppm)
		var gh_pipe_body_visible: bool = _rect_intersects(viewport_rect,gh_pipe_rect)
		var gh_pipe_fallback_visible: bool = _rect_intersects(viewport_rect,gh_pipe_fallback)
		return {'furnaceKey':'','culled':not (gh_pipe_body_visible or gh_pipe_fallback_visible),'bodyRect':gh_pipe_rect,'fallbackRect':gh_pipe_fallback}
	if o.get('kind')=='gh_entry_service_pipe' and o.get('data') is Dictionary and o.data.get('body_rect') is Rect2:
		var gh_entry_pipe_rect: Rect2 = o.data.body_rect
		var gh_entry_pipe_fallback: Rect2 = _prop_placeholder_rect(at,camera.ppm)
		var gh_entry_pipe_body_visible: bool = _rect_intersects(viewport_rect,gh_entry_pipe_rect)
		var gh_entry_pipe_fallback_visible: bool = _rect_intersects(viewport_rect,gh_entry_pipe_fallback)
		return {'furnaceKey':'','culled':not (gh_entry_pipe_body_visible or gh_entry_pipe_fallback_visible),'bodyRect':gh_entry_pipe_rect,'fallbackRect':gh_entry_pipe_fallback}
	if o.get('kind')=='gh_bio_sample_case' and o.get('data') is Dictionary and o.data.get('body_rect') is Rect2:
		var sample_rect: Rect2 = o.data.body_rect
		return {'furnaceKey':'','culled':not _rect_intersects(viewport_rect,sample_rect),'bodyRect':sample_rect,'fallbackRect':Rect2()}
	if o.get('kind')=='gh_bio_clutter' and o.get('data') is Dictionary and o.data.get('body_rect') is Rect2:
		var gh_clutter_rect: Rect2 = o.data.body_rect
		var gh_clutter_fallback: Rect2 = _prop_placeholder_rect(at,camera.ppm)
		var gh_clutter_body_visible: bool = _rect_intersects(viewport_rect,gh_clutter_rect)
		var gh_clutter_fallback_visible: bool = _rect_intersects(viewport_rect,gh_clutter_fallback)
		return {'furnaceKey':'','culled':not (gh_clutter_body_visible or gh_clutter_fallback_visible),'bodyRect':gh_clutter_rect,'fallbackRect':gh_clutter_fallback}
	if o.get('kind')=='gh_entry_clutter' and o.get('data') is Dictionary and o.data.get('body_rect') is Rect2:
		var gh_entry_clutter_rect: Rect2 = o.data.body_rect
		var gh_entry_clutter_fallback: Rect2 = _prop_placeholder_rect(at,camera.ppm)
		var gh_entry_clutter_body_visible: bool = _rect_intersects(viewport_rect,gh_entry_clutter_rect)
		var gh_entry_clutter_fallback_visible: bool = _rect_intersects(viewport_rect,gh_entry_clutter_fallback)
		return {'furnaceKey':'','culled':not (gh_entry_clutter_body_visible or gh_entry_clutter_fallback_visible),'bodyRect':gh_entry_clutter_rect,'fallbackRect':gh_entry_clutter_fallback}
	if o.get('kind')=='gh_lockdown_dressing' and o.get('data') is Dictionary and o.data.get('body_rect') is Rect2:
		var gh_lockdown_dressing_rect: Rect2 = o.data.body_rect
		var gh_lockdown_dressing_fallback: Rect2 = _prop_placeholder_rect(at,camera.ppm)
		var gh_lockdown_dressing_body_visible: bool = _rect_intersects(viewport_rect,gh_lockdown_dressing_rect)
		var gh_lockdown_dressing_fallback_visible: bool = _rect_intersects(viewport_rect,gh_lockdown_dressing_fallback)
		return {'furnaceKey':'','culled':not (gh_lockdown_dressing_body_visible or gh_lockdown_dressing_fallback_visible),'bodyRect':gh_lockdown_dressing_rect,'fallbackRect':gh_lockdown_dressing_fallback}
	if o.get('kind')=='gh_console_body' and o.get('id')=='power_console_off_on' and o.get('p') is Vector3 and o.p==Vector3(4,0,7) and o.get('data') is Dictionary and grey_hive_state_props!=null and P.snapshot(snapshot) and snapshot.has('entryToken') and snapshot.get('entryToken')==null and snapshot.get('worldId')=='grey_hive' and snapshot.get('sceneId')=='gh_power_room' and P.counter(snapshot.get('worldEpoch'),1) and scene.get('worldId')=='grey_hive' and scene.get('sceneId')=='gh_power_room' and current_key=='grey_hive:gh_power_room:'+str(snapshot.get('worldEpoch')) and camera!=null:
		var gh_power_console_row: Dictionary = _gh_visual_row('power_console_off_on','runtime2d.prop.power_console.off_on.v1',[4,0,7])
		var gh_power_console_resolved: Dictionary = _gh_console_body(gh_power_console_row) if not gh_power_console_row.is_empty() else {}
		if not gh_power_console_resolved.is_empty() and gh_power_console_resolved==o:
			var gh_power_console_data: Dictionary = gh_power_console_resolved.get('data',{})
			var gh_power_console_state_value = gh_power_console_data.get('state')
			if typeof(gh_power_console_state_value)==TYPE_STRING and gh_power_console_state_value in ['off','on']:
				var gh_power_console_state: String = gh_power_console_state_value
				var gh_power_console_rect: Rect2 = grey_hive_state_props.canvas_rect('power_console',gh_power_console_state,at,camera.ppm)
				if _rect_intersects(viewport_rect,gh_power_console_rect):
					return {'furnaceKey':'','culled':false,'bodyRect':gh_power_console_rect,'fallbackRect':Rect2()}
	if o.get('kind')=='door' and o.get('data') is Dictionary and grey_hive_state_props!=null:
		var gate_body_selection: Dictionary = _gh_door_body(o.data)
		if not gate_body_selection.is_empty():
			var gate_body_rect: Rect2 = grey_hive_state_props.canvas_rect(gate_body_selection.key,gate_body_selection.state,at,camera.ppm)
			if _rect_intersects(viewport_rect,gate_body_rect):
				return {'furnaceKey':'','culled':false,'bodyRect':gate_body_rect,'fallbackRect':Rect2()}
	if o.get('kind')=='actor' and o.get('data') is Dictionary and grey_hive_actor!=null and o.get('p') is Vector3:
		var gh_actor_edge_key: String = _grey_hive_body_key(o.data)
		if not gh_actor_edge_key.is_empty() and o.p==P.vector(o.data.transform.positionM) and not _occluded(o.p):
			if gh_actor_edge_key=='sentinel' and SENTINEL_ATLAS_READY and gh_sentinel_temporal_visual.ready():
				var temporal_rect: Rect2=gh_sentinel_temporal_visual.canvas_rect(at,camera.ppm)
				if _rect_intersects(viewport_rect,temporal_rect):
					return {'furnaceKey':'','culled':false,'bodyRect':temporal_rect,'fallbackRect':Rect2()}
			var gh_actor_edge_rect: Rect2 = grey_hive_actor.canvas_rect(gh_actor_edge_key,at,camera.ppm)
			if _rect_intersects(viewport_rect,gh_actor_edge_rect):
				return {'furnaceKey':'','culled':false,'bodyRect':gh_actor_edge_rect,'fallbackRect':Rect2()}
	var furnace_key = ''
	if o.get('kind')=='prop' and o.get('data') is Dictionary:
		furnace_key = _cw_furnace_key(o.data)
	if furnace_key.is_empty():
		return {'furnaceKey':'','culled':not viewport_rect.grow(100).has_point(at),'bodyRect':Rect2(),'fallbackRect':Rect2()}
	var body_rect = cw_furnace_visual.canvas_rect(furnace_key,at,camera.ppm) if cw_furnace_visual!=null else Rect2()
	var fallback_rect = _prop_placeholder_rect(at,camera.ppm)
	var body_visible = _rect_intersects(viewport_rect,body_rect)
	var fallback_visible = _rect_intersects(viewport_rect,fallback_rect)
	return {'furnaceKey':furnace_key,'culled':not (body_visible or fallback_visible),'bodyRect':body_rect,'fallbackRect':fallback_rect}
func _gh_bio_log_art_caption_offset(o: Dictionary) -> float:
	var terminal_foot := Vector3(20,0,8)
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null:
		return 0.0
	if snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_bio_isolation' or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_bio_isolation':
		return 0.0
	if typeof(current_key)!=TYPE_STRING or current_key!='grey_hive:gh_bio_isolation:'+str(snapshot.worldEpoch):
		return 0.0
	if camera==null:
		return 0.0
	var ppm_type: int = typeof(camera.ppm)
	if ppm_type!=TYPE_INT and ppm_type!=TYPE_FLOAT:
		return 0.0
	var ppm: float = float(camera.ppm)
	if not is_finite(ppm) or ppm<=0.0:
		return 0.0
	if o.get('kind')!='interaction' or o.get('id')!='gh_bio_log_terminal' or not o.get('p') is Vector3 or o.get('p')!=terminal_foot:
		return 0.0
	var current_rows = snapshot.get('interactables')
	if not P.rows(current_rows,2048) or not P.unique(current_rows,'entityId'):
		return 0.0
	var current_matches: Array = current_rows.filter(func(row): return row.get('entityId')=='gh_bio_log_terminal')
	if current_matches.size()!=1:
		return 0.0
	var current_row: Dictionary = current_matches[0]
	if current_row.get('kind')!='bio_log_terminal' or typeof(current_row.get('active'))!=TYPE_BOOL or not current_row.active or not P.transform(current_row.get('transform')) or P.vector(current_row.transform.positionM)!=terminal_foot or o.get('data')!=current_row:
		return 0.0
	var authored_rows = scene.get('interactions')
	if not P.rows(authored_rows,2048) or not P.unique(authored_rows,'id'):
		return 0.0
	var authored_matches: Array = authored_rows.filter(func(row): return row.get('id')=='gh_bio_log_terminal')
	if authored_matches.size()!=1:
		return 0.0
	var authored_row: Dictionary = authored_matches[0]
	if typeof(authored_row.get('kind'))!=TYPE_STRING or authored_row.kind!='bio_log_terminal' or not authored_row.has('event') or authored_row.get('event')!=null or not _gh_authored_position(authored_row.get('position'),terminal_foot):
		return 0.0
	var presentation = scene.get('presentation')
	if not presentation is Dictionary:
		return 0.0
	var sprite_rows = presentation.get('sprites')
	if not P.rows(sprite_rows,2048) or not P.unique(sprite_rows,'id'):
		return 0.0
	var story_matches: Array = sprite_rows.filter(func(row): return row.get('id')=='bio_log_terminal_approved')
	if story_matches.size()!=1:
		return 0.0
	var resolved_story: Dictionary = _gh_story_prop(story_matches[0])
	if resolved_story.is_empty() or resolved_story.get('id')!='bio_log_terminal_approved' or resolved_story.get('kind')!='gh_story_prop' or resolved_story.get('p')!=terminal_foot or not resolved_story.get('data') is Dictionary or resolved_story.data.get('key')!='lockdown_terminal' or typeof(resolved_story.data.get('image_allowed'))!=TYPE_BOOL or not resolved_story.data.image_allowed:
		return 0.0
	var terminal_config = GhBioLockdownVisual.CONFIG.get('lockdown_terminal')
	if not terminal_config is Dictionary:
		return 0.0
	var configured_height = terminal_config.get('height_m')
	if typeof(configured_height)!=TYPE_INT and typeof(configured_height)!=TYPE_FLOAT:
		return 0.0
	var expected_height: float = float(configured_height)*ppm
	if not is_finite(expected_height) or expected_height<=0.0 or not prop_bodies_drawn.has('bio_log_terminal_approved'):
		return 0.0
	var cached_height = prop_bodies_drawn.get('bio_log_terminal_approved')
	if typeof(cached_height)!=TYPE_INT and typeof(cached_height)!=TYPE_FLOAT:
		return 0.0
	var cached_height_number: float = float(cached_height)
	if not is_finite(cached_height_number) or cached_height_number<=0.0 or cached_height_number!=expected_height:
		return 0.0
	var offset: float = expected_height+4.0*(ppm/48.0)
	if not is_finite(offset) or offset<=0.0:
		return 0.0
	return offset

func _gh_entry_tutorial_art_caption_offset(o: Dictionary, at: Vector2) -> float:
	var foot := Vector3(4,0,7)
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null:
		return 0.0
	if snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_entry_maintenance' or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_entry_maintenance':
		return 0.0
	if typeof(current_key)!=TYPE_STRING or current_key!='grey_hive:gh_entry_maintenance:'+str(snapshot.worldEpoch) or camera==null:
		return 0.0
	if typeof(camera.ppm)!=TYPE_INT and typeof(camera.ppm)!=TYPE_FLOAT:
		return 0.0
	var ppm: float = float(camera.ppm)
	if not is_finite(ppm) or ppm<=0.0 or o.get('kind')!='interaction' or o.get('id')!='gh_entry_tutorial_terminal' or not o.get('p') is Vector3 or o.get('p')!=foot:
		return 0.0
	var authored = scene.get('interactions')
	if not P.rows(authored,2048) or not P.unique(authored,'id'):
		return 0.0
	var authored_matches: Array = authored.filter(func(row): return row.get('id')=='gh_entry_tutorial_terminal')
	if authored_matches.size()!=1:
		return 0.0
	var authored_row: Dictionary = authored_matches[0]
	if authored_row.get('kind')!='terminal' or not authored_row.has('event') or authored_row.get('event')!=null or not _gh_authored_position(authored_row.get('position'),foot):
		return 0.0
	var current = snapshot.get('interactables')
	if not P.rows(current,2048) or not P.unique(current,'entityId'):
		return 0.0
	var current_matches: Array = current.filter(func(row): return row.get('entityId')=='gh_entry_tutorial_terminal')
	if current_matches.size()!=1:
		return 0.0
	var row: Dictionary = current_matches[0]
	if row.get('kind')!='terminal' or typeof(row.get('active'))!=TYPE_BOOL or not row.active or not P.transform(row.get('transform')) or P.vector(row.transform.positionM)!=foot or o.get('data')!=row:
		return 0.0
	var offset: float = 1.4*ppm+4.0*(ppm/48.0)
	if not is_finite(offset) or offset<=0.0 or gh_bio_lockdown_visual==null or not gh_bio_lockdown_visual.draw_body(self,'lockdown_terminal',at,ppm):
		return 0.0
	return offset

func _gh_facility_log_art_caption_offset(o: Dictionary, at: Vector2) -> float:
	var targets = {
		'gh_log_shaft_01': {'scene':'gh_central_shaft','foot':Vector3(3,0,8)},
		'gh_sys_sentinel_01': {'scene':'gh_sentinel_arena','foot':Vector3(6,0,3.5)}
	}
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or not targets.has(o.get('id')):
		return 0.0
	var target: Dictionary = targets[o.id]
	var foot: Vector3 = target.foot
	if snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!=target.scene or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!=target.scene:
		return 0.0
	if typeof(current_key)!=TYPE_STRING or current_key!='grey_hive:'+target.scene+':'+str(snapshot.worldEpoch) or camera==null:
		return 0.0
	if typeof(camera.ppm)!=TYPE_INT and typeof(camera.ppm)!=TYPE_FLOAT:
		return 0.0
	var ppm: float = float(camera.ppm)
	if not is_finite(ppm) or ppm<=0.0 or o.get('kind')!='interaction' or not o.get('p') is Vector3 or o.get('p')!=foot:
		return 0.0
	var authored = scene.get('interactions')
	if not P.rows(authored,2048) or not P.unique(authored,'id'):
		return 0.0
	var authored_matches: Array = authored.filter(func(row): return row.get('id')==o.id)
	if authored_matches.size()!=1:
		return 0.0
	var authored_row: Dictionary = authored_matches[0]
	if authored_row.get('kind')!='facility_log' or not authored_row.has('event') or authored_row.get('event')!=null or not _gh_authored_position(authored_row.get('position'),foot):
		return 0.0
	var current = snapshot.get('interactables')
	if not P.rows(current,2048) or not P.unique(current,'entityId'):
		return 0.0
	var current_matches: Array = current.filter(func(row): return row.get('entityId')==o.id)
	if current_matches.size()!=1:
		return 0.0
	var row: Dictionary = current_matches[0]
	if row.get('kind')!='facility_log' or typeof(row.get('active'))!=TYPE_BOOL or not row.active or not P.transform(row.get('transform')) or P.vector(row.transform.positionM)!=foot or o.get('data')!=row:
		return 0.0
	var offset: float = 1.4*ppm+4.0*(ppm/48.0)
	if not is_finite(offset) or offset<=0.0 or gh_bio_lockdown_visual==null or not gh_bio_lockdown_visual.draw_body(self,'lockdown_terminal',at,ppm):
		return 0.0
	return offset

# Keep every ambiguous or edge-touching packet on the original draw path.
func _shaft_service_wall_outside(data: Dictionary) -> bool:
	if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or snapshot.sceneId!='gh_central_shaft' or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or current_key!='grey_hive:gh_central_shaft:'+str(snapshot.worldEpoch) or scene_hash!=GhShaftLowerServiceVisual.SHA or not camera.initialized: return false
	if not snapshot.get('shaftLowerService') is Dictionary or snapshot.shaftLowerService.size()!=2 or snapshot.shaftLowerService.get('recipeVersion')!=2 or snapshot.shaftLowerService.get('sceneSourceSha256')!=scene_hash: return false
	if not is_inside_tree() or viewport_rect!=get_viewport_rect() or get_global_transform()!=Transform2D.IDENTITY or get_canvas_transform()!=Transform2D.IDENTITY: return false
	if not is_finite(viewport_rect.position.x) or not is_finite(viewport_rect.position.y) or not is_finite(viewport_rect.size.x) or not is_finite(viewport_rect.size.y) or not viewport_rect.has_area(): return false
	var plan=data.get('plan')
	var index=data.get('index')
	if not plan is Dictionary or plan.get('identity')!=snapshot.shaftLowerService or plan.get('suppressOldOwner')!='lower_east' or not plan.get('selection') is Dictionary or plan.selection.get('floor')!='lower' or not plan.get('depth') is Array or typeof(index)!=TYPE_INT or index<0 or index>=plan.depth.size(): return false
	var piece=plan.depth[index]
	if not piece is Dictionary or piece.get('role')!='service_wall' or piece.get('ownerRegions')!=['lower_service'] or not piece.get('points') is PackedVector2Array or not piece.get('uv') is PackedVector2Array or not piece.get('texture') is Texture2D or not is_instance_valid(piece.texture): return false
	var points: PackedVector2Array=piece.points
	var uv: PackedVector2Array=piece.uv
	if points.size()!=4 or uv.size()!=4: return false
	var lo=Vector2(INF,INF)
	var hi=Vector2(-INF,-INF)
	var area:=0.0
	var uv_area:=0.0
	for i in range(4):
		if not points[i].is_finite() or not uv[i].is_finite(): return false
		lo=lo.min(points[i]);hi=hi.max(points[i])
		area+=points[i].cross(points[(i+1)%4])
		uv_area+=uv[i].cross(uv[(i+1)%4])
	if not is_finite(area) or not is_finite(uv_area) or absf(area)<=0.000001 or absf(uv_area)<=0.000001: return false
	# Slack protects rasterization at the boundary; no geometry is changed.
	var slack:=2.0
	return hi.x<viewport_rect.position.x-slack or lo.x>viewport_rect.end.x+slack or hi.y<viewport_rect.position.y-slack or lo.y>viewport_rect.end.y+slack
func _draw_object(o: Dictionary, interaction_caption: String = '') -> void:
	if o.get('kind')=='hive_route_piece':
		if gh_route_neighbor_visual.draw_piece(self,o.data): hive_route_drawn+=1
		else: hive_route_draw_failed=true
		return
	if o.get('kind')=='gh_bio_region_atlas':
		var piece=o.get('data',{})
		if not piece is Dictionary or not piece.get('points') is PackedVector2Array or not piece.get('uv') is PackedVector2Array or not piece.get('texture') is Texture2D:
			utility_draw_failed=true
			return
		var alpha=float(gh_bio_wall_alpha.get(piece.get('fade_group'),1.0))
		draw_polygon(piece.points,PackedColorArray([Color(1,1,1,clampf(alpha,0.25,1.0))]),piece.uv,piece.texture)
		utility_bio_wall_drawn+=1
		return
	if o.get('kind')=='gh_exit_source_piece':
		if gh_exit_source_owned_visual.draw_piece(self,o.data): exit_source_drawn+=1
		else: exit_source_failed=true
		return
	if o.get('kind')=='gh_exit_source_prop':
		if gh_exit_source_owned_visual.draw_prop(self,o.data): exit_source_drawn+=1
		else: exit_source_failed=true
		return
	if o.get('kind')=='gh_utility_piece':
		if gh_utility_layout_visual.draw_piece(self,o.data): utility_drawn_pieces+=1
		else: utility_draw_failed=true
		return
	if o.get('kind')=='gh_shaft_fixture':
		var packet=o.get('data',{});var fixture=packet.get('fixture',{});var texture=packet.get('texture')
		if not fixture is Dictionary: shaft_draw_failed=true; return
		if fixture.get('kind')=='retained':
			if not gh_shaft_lower_service_visual._retained.draw_prop(self,fixture.get('packet',{})): shaft_draw_failed=true
			return
		if not texture is Texture2D or not fixture.get('destination') is Rect2 or not fixture.get('source') is Rect2: shaft_draw_failed=true; return
		draw_texture_rect_region(texture,fixture.destination,fixture.source,Color.WHITE,false,true)
		return
	if o.get('kind')=='gh_shaft_piece':
		if o.get('data') is Dictionary and _shaft_service_wall_outside(o.data): return
		if not gh_shaft_lift_visual.draw_piece(self,o.data.plan,o.data.index): shaft_draw_failed=true
		return
	if o.get('kind')=='gh_dense_boundary':
		draw_polygon(o.data.points,PackedColorArray([Color.WHITE]),o.data.uv,o.data.texture)
		return
	if o.get('kind')=='gh_dense_partition':
		gh_dense_service_wing_visual.draw(self,o.data)
		return
	var at = camera.project(o.p)
	var culling = _object_culling_state(o,at)
	if culling.culled:
		return
	var furnace_key: String = culling.furnaceKey
	var scale = camera.ppm/48.0
	var c = Color('#779ba2')
	match o.kind:
		'gh_entry_wall_module':
			var entry_module_alpha: float = float(gh_entry_wall_alpha.get(o.data.get('fade_group',''),1.0))
			if not gh_entry_wall_modules_visual.draw_module(self,o.data,entry_module_alpha,snapshot,scene,current_key,camera):
				gh_entry_wall_draw_failed = true
		'gh_bio_wall_module':
			var gh_module_alpha: float = float(gh_bio_wall_alpha.get(o.data.get('fade_group',''),1.0))
			var bio_wall_drawn: bool = gh_bio_wall_modules_visual.draw_module(self,o.data,gh_module_alpha)
			if _utility_new_scope() and snapshot.sceneId=='gh_bio_isolation':
				if bio_wall_drawn: utility_bio_wall_drawn+=1
				else: utility_draw_failed=true
		'rs_bulkhead_module':
			var module_alpha: float = float(rs_front_alpha.get(o.data.get('fade_group',''),1.0))
			rs_bulkhead_modules_visual.draw_module(self,o.data,camera.ppm,module_alpha)
		'rs_service_pipe':
			if not rs_service_pipe_visual.draw_body(self,o.data):
				draw_rect(_prop_placeholder_rect(at,camera.ppm),Color('#65716d'))
		'gh_bio_service_pipe':
			if not gh_bio_service_pipe_visual.draw_body(self,o.data):
				draw_rect(culling.fallbackRect,Color('#65716d'))
		'gh_entry_service_pipe':
			if not gh_entry_service_pipe_visual.draw_body(self,o.data,snapshot,scene,current_key,camera,gh_entry_wall_modules_visual):
				draw_rect(culling.fallbackRect,Color('#65716d'))
		'gh_bio_sample_case':
			gh_bio_sample_case_visual.draw_body(self,o.data)
		'gh_exit_barrier_module':
			gh_exit_sealed_barrier_visual.draw_body(self,o.data)
		'gh_bio_clutter':
			gh_bio_clutter_visual.draw_body(self,o.data)
		'gh_entry_clutter':
			if not gh_entry_clutter_visual.draw_body(self,o.data,snapshot,scene,current_key,camera,gh_entry_wall_modules_visual):
				draw_rect(culling.fallbackRect,Color('#65716d'))
		'gh_lockdown_dressing':
			if not gh_lockdown_environment_visual.draw_dressing(self,o.data,snapshot,scene,current_key,camera,viewport_rect):
				draw_rect(culling.fallbackRect,Color('#65716d'))
		'gh_central_pipe':
			if not gh_central_shaft_environment_visual.draw_pipe_body(self,o.data,snapshot,scene,current_key,camera,viewport_rect):
				if shaft_visual_expected() or shaft_legacy_deferred(): shaft_draw_failed=true
				draw_rect(culling.fallbackRect,Color('#65716d'))
		'gh_exit_dressing':
			if not gh_exit_environment_visual.draw_dressing(self,o.data,snapshot,scene,current_key,camera,viewport_rect):
				draw_rect(culling.fallbackRect,Color('#65716d'))
		'gh_story_prop':
			var body_drawn = false
			if o.data.image_allowed and gh_bio_lockdown_visual!=null:
				body_drawn = gh_bio_lockdown_visual.draw_body(self,o.data.key,at,camera.ppm)
			if body_drawn and o.get('id')=='bio_log_terminal_approved' and o.get('data') is Dictionary and o.data.get('key')=='lockdown_terminal' and o.get('p') is Vector3 and o.p==Vector3(20,0,8):
				prop_bodies_drawn['bio_log_terminal_approved'] = float(GhBioLockdownVisual.CONFIG['lockdown_terminal']['height_m'])*camera.ppm
			if not body_drawn:
				draw_rect(Rect2(at-Vector2(16,24)*scale,Vector2(32,24)*scale),Color('#65716d'))
		'registered_terminal':
			var terminal_key = 'rs_terminals' if o.get('id') in ['world_gate_terminal_art','save_rest_terminal_art'] and o.data.get('assetId')=='runtime2d.world.returnstation.terminals.v1' else 'rs_terminal'
			var storage_drawn: bool = o.get('id')=='storage_terminal_art' and o.data.get('assetId')=='runtime2d.world.returnstation.terminal.v1' and registered_props.draw_storage_body(self,at,camera.ppm)
			if not storage_drawn and not registered_props.draw_body(self,terminal_key,at,camera.ppm):
				draw_rect(Rect2(at-Vector2(16,24)*scale,Vector2(32,24)*scale),Color('#65716d'))
		'beacon_body':
			if registered_props.draw_body(self,o.data.key,at,camera.ppm):
				prop_bodies_drawn[o.id] = registered_props.canvas_height(o.data.key,camera.ppm)
		'gh_console_body':
			var drawn = grey_hive_state_props!=null and grey_hive_state_props.draw_body(self,'power_console',o.data.state,at,camera.ppm)
			var dot_height = grey_hive_state_props.body_height('power_console',camera.ppm)+4*scale if drawn else 28*scale
			draw_circle(at-Vector2(0,dot_height),5,Color('#61ddcf') if o.data.state=='on' else Color('#efbc6c'))
			if not drawn:
				draw_rect(Rect2(at-Vector2(16,24)*scale,Vector2(32,24)*scale),Color('#65716d'))
		'prop':
			if o.get('id')=='gate_a_cabling' and gh_entry_service_pipe_visual!=null and gh_entry_service_pipe_visual.draw_gate_a_body(self,snapshot,scene,current_key,scene_hash,camera):
				return
			if _gh_crate_selected(o.data) and gh_supply_crate_visual!=null and gh_supply_crate_visual.draw_body(self,at,camera.ppm):
				return
			if furnace_key!='' and cw_furnace_visual!=null and cw_furnace_visual.draw_body(self,furnace_key,at,camera.ppm):
				return
			if _cw_pressure_valve_selected(o.data) and cw_pressure_valve_visual!=null and cw_pressure_valve_visual.draw_body(self,at,camera.ppm):
				return
			if o.data.get('assetId')=='runtime2d.prop.power_console.off_on.v1':
				var powered = false
				for row in snapshot.progression.get('worlds',[]):
					if row is Dictionary and row.get('worldId')=='grey_hive' and row.get('completedEvents') is Array and 'hive_power' in row.completedEvents:
						powered = true
				draw_circle(at-Vector2(0,28)*scale,5,Color('#61ddcf') if powered else Color('#efbc6c'))
			var beacon_scene: Dictionary = utility_plan.original_beacon if _utility_plan_current() and snapshot.sceneId=='gh_beacon' else scene
			if gh_beacon_environment_visual.should_skip_mount_blockout(o,snapshot,beacon_scene,current_key,camera,viewport_rect,prop_bodies_drawn):
				return
			draw_rect(_prop_placeholder_rect(at,camera.ppm),Color('#65716d'))
		'player':
			c = Color('#85dfd0')
			var body_drawn = false
			if o.id==snapshot.player.entityId and o.data==snapshot.player and snapshot.player.currentHp>0:
				var visual_player: Dictionary = o.data
				if P.same(visual_movement_context,snapshot) and visual_movement_facing.length_squared()>0.01 and not snapshot.player.actionState in ['primaryAttack','pulse','guard','pierce','dash']:
					visual_player = o.data.duplicate()
					visual_player.facingX = visual_movement_facing.x
					visual_player.facingZ = visual_movement_facing.y
				if _walk_allowed(snapshot):
					body_drawn = cenyao_east_walk.draw_body(self,at,camera.ppm,walk_seconds)
				if not body_drawn:
					body_drawn = cenyao_idle.draw_body(self,at,visual_player,camera.ppm)
			if body_drawn and snapshot.player.actionState in ['guard','dash']:
				var factor: float=(104.0/560.0)*(camera.ppm/48.0)
				if is_finite(factor) and factor>0.0:
					action_body_frame={'worldId':snapshot.worldId,'sceneId':snapshot.sceneId,'worldEpoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':o.id,'foot':at,'rect':Rect2(at-CenYaoIdle.ANCHOR*factor,CenYaoIdle.CANVAS*factor)}
			if not body_drawn:
				draw_line(at,at-Vector2(0,34)*scale,c,9*scale,true)
			displayed[o.id] = true
		'actor':
			var a: Dictionary = o.data
			c = Color('#df9c79')
			var type: String = a.entityType
			if type=='enemy.mist_harbor.signal_wraith' and a.get('signalOnly',false)==true:
				_wraith_drop(a.entityId)
				if not a.signalPerception.precise: _ring(o.p,float(a.signalPerception.uncertaintyRadiusM),Color(0.6,0.5,0.8,0.5))
				_label(at-Vector2(0,30*scale),'信号' if a.signalPerception.precise else '不确定信号',Color('#b9a9de'))
				return
			if type=='enemy.mist_harbor.signal_wraith':
				c = Color('#b9a9de')
				var precise = a.get('signalPerception',{}).get('precise',false)
				if not precise:
					_wraith_drop(a.entityId)
					_ring(o.p,float(a.signalPerception.uncertaintyRadiusM),Color(0.6,0.5,0.8,0.5))
					_label(at-Vector2(0,30*scale),'不确定信号',c)
					return
			if type=='enemy.mist_harbor.signal_wraith':
				var approved_positions: Array = V.signal_positions(a,snapshot)
				if _mh_cw_body_key(a)!='signal_wraith' or not actor_visible(a.entityId) or approved_positions.size()!=1 or o.p!=P.vector(approved_positions[0]):
					_wraith_drop(a.entityId)
					return
			if 'pressure_drone' in type:
				if not _cw_drone_hound_body(a,o.p,at,'pressure_drone'):
					draw_line(at+Vector2(-16,-22)*scale,at+Vector2(16,-22)*scale,c,4,true)
					draw_circle(at-Vector2(0,22)*scale,9*scale,c)
			elif 'hound' in type:
				if not _cw_drone_hound_body(a,o.p,at,'furnace_hound'):
					draw_rect(Rect2(at-Vector2(25,18)*scale,Vector2(50,18)*scale),c)
			elif type=='enemy.grey_hive.swarm':
				if not _swarm_actor_matches(a) or not actor_visible(a.entityId) or not o.get('member') is Dictionary or not visible_point(o.p) or _occluded(o.p) or (_region_visibility_scope() and not is_dense_visible_position(o.p)):
					return
				var drawn=false
				if SWARM_ATLAS_READY:
					drawn=gh_swarm_member_temporal_visual.draw_body(self,snapshot,a,o.member,at,camera.ppm,events,swarm_temporal_animate)
				if not drawn:
					swarm_art_unready=true
					draw_circle(at,5*scale,c)
			else:
				var body_key = _grey_hive_body_key(a)
				if body_key=='sentinel' and not actor_visible(a.entityId) and (_occluded(o.p) or not _rect_intersects(viewport_rect,culling.bodyRect)):
					return
				var body_drawn = _mh_warden_body(a,o.p,at)
				if not body_drawn: body_drawn = _mh_drowned_tidebound_body(a,o.p,at)
				if _gate_b_brute_matches(a) and actor_visible(a.entityId) and o.p==P.vector(a.transform.positionM) and P.counter(snapshot.get('serverTimeMs')):
					body_drawn=gh_gate_b_brute_temporal_visual.draw_body(self,current_key,int(snapshot.worldEpoch),int(snapshot.serverTick),int(snapshot.serverTimeMs),a,_gate_b_brute_events(),at,camera.ppm,not reduced_motion)
				if not body_drawn and grey_hive_actor!=null and not body_key.is_empty() and (actor_visible(a.entityId) or (_rect_intersects(viewport_rect,culling.bodyRect) and not _occluded(o.p))):
					if body_key=='sentinel' and SENTINEL_ATLAS_READY:
						body_drawn=gh_sentinel_temporal_visual.draw_body(self,snapshot,a,scene_hash,at,camera.ppm,sentinel_temporal_animate)
						sentinel_temporal_drawn=body_drawn
						if not body_drawn: sentinel_art_unready=true
					elif body_key=='maintenance' and P.counter(snapshot.get('serverTimeMs')):
						body_drawn = gh_maintenance_temporal_visual.draw_body(self,current_key,int(snapshot.worldEpoch),int(snapshot.serverTick),int(snapshot.serverTimeMs),a,at,camera.ppm,not reduced_motion)
					elif body_key=='security' and P.counter(snapshot.get('serverTimeMs')):
						body_drawn = gh_security_temporal_visual.draw_body(self,current_key,int(snapshot.worldEpoch),int(snapshot.serverTick),int(snapshot.serverTimeMs),a,at,camera.ppm,not reduced_motion)
					if not body_drawn:
						body_drawn = grey_hive_actor.draw_body(self,body_key,at,camera.ppm)
					if body_drawn:
						_record_gh_vital_body(a,body_key,at)
				if not body_drawn and mh_cw_actor!=null and actor_visible(a.entityId):
					var mh_cw_key = _mh_cw_body_key(a)
					if not mh_cw_key.is_empty():
						if PRIME_REGULATOR_ATLAS_READY and _cw_regular_guard_matches(a) and o.p==P.vector(a.transform.positionM) and P.counter(snapshot.get('serverTimeMs')):
							if not cw_regular_guard_temporal_by_id.has(a.entityId): cw_regular_guard_temporal_by_id[a.entityId]=CwPrimeRegulatorTemporalVisual.new()
							cw_regular_guard_sampled[a.entityId]=true
							body_drawn=cw_regular_guard_temporal_by_id[a.entityId].draw_body(self,current_key,int(snapshot.worldEpoch),int(snapshot.serverTick),int(snapshot.serverTimeMs),a,at,camera.ppm,a.entityId,not reduced_motion)
						if PRIME_REGULATOR_ATLAS_READY and a.entityId=='cw_prime_regulator' and mh_cw_key=='forged_guard' and P.counter(snapshot.get('serverTimeMs')):
							prime_regulator_sampled=true
							body_drawn=cw_prime_regulator_temporal_visual.draw_body(self,current_key,int(snapshot.worldEpoch),int(snapshot.serverTick),int(snapshot.serverTimeMs),a,at,camera.ppm,'cw_prime_regulator',not reduced_motion)
						if mh_cw_key=='signal_wraith' and not reduced_motion:
							if not mh_wraith_temporal_by_id.has(a.entityId): mh_wraith_temporal_by_id[a.entityId] = MhSignalWraithTemporalVisual.new()
							body_drawn = mh_wraith_temporal_by_id[a.entityId].draw_body(self,snapshot,scene,a,true,_wraith_events(a.entityId),Callable(camera,'project'),camera.ppm)
						if not body_drawn:
							body_drawn = mh_cw_actor.draw_body(self,mh_cw_key,at,camera.ppm)
				if not body_drawn:
					if type=='enemy.mist_harbor.signal_wraith': return
					var boss = 'sentinel' in type or 'forged_guard' in type
					draw_rect(Rect2(at-Vector2(13,44 if boss else 30)*scale,Vector2(26,44 if boss else 30)*scale),c)
			if color_assistance:
				draw_line(at-Vector2(7,7),at+Vector2(7,7),Color.WHITE,2)
				draw_line(at-Vector2(-7,7),at+Vector2(-7,7),Color.WHITE,2)
			if a.entityId in ['cw_prime_regulator','cw_forged_guard_elite']:
				_protected_actor_label(a.entityId,at-Vector2(0,65*text_scale),'主调节者' if a.entityId=='cw_prime_regulator' else '铸造卫士')
			displayed[a.entityId] = true
		'door':
			var d: Dictionary = o.data
			c = Color('#76c3b4') if d.open else Color('#cc986e')
			var selection = _gh_door_body(d)
			var drawn = false
			if grey_hive_state_props!=null and not selection.is_empty():
				drawn = grey_hive_state_props.draw_body(self,selection.key,selection.state,at,camera.ppm)
			if not drawn:
				draw_rect(Rect2(at-Vector2(16,42)*scale,Vector2(32,42)*scale),Color(c,0.2 if d.open else 0.8))
			var caption_height = grey_hive_state_props.body_height(selection.key,camera.ppm)+4*scale if drawn else 46*scale
			_label(at-Vector2(0,caption_height),'开启' if d.open else ('锁定' if d.locked else '关闭'),c)
		'interaction':
			var badge = _rs_gate_badge(o.id)
			var badge_drawn = not badge.is_empty() and o.get('data')==badge.row and o.get('p')==P.vector(badge.row.transform.positionM) and at==badge.at
			if badge_drawn:
				draw_texture_rect_region(badge.texture,badge.rect,Rect2(520,936,216,216),Color.WHITE)
			var art_caption_offset: float = _gh_bio_log_art_caption_offset(o)
			if art_caption_offset<=0.0:
				art_caption_offset = _gh_entry_tutorial_art_caption_offset(o,at)
			if art_caption_offset<=0.0:
				art_caption_offset = _gh_facility_log_art_caption_offset(o,at)
			if art_caption_offset<=0.0 and not badge_drawn and not gh_first_pair_portal(o.data) and not (o.id=='rs_world_gate_marker' and _rs_gh_gate_active()):
				draw_rect(Rect2(at-Vector2(8,16)*scale,Vector2(16,16)*scale),Color('#72b9cb'))
			if not interaction_caption.is_empty():
				if art_caption_offset>0.0:
					_label(at-Vector2(0,art_caption_offset),interaction_caption)
				else:
					_label(at-Vector2(0,20)*scale,interaction_caption)
		'npc':
			if not is_finite(camera.ppm) or camera.ppm<=0 or not is_finite(at.x) or not is_finite(at.y) or not is_finite(1.7*camera.ppm+5*scale):
				return
			if not baizhi_static_body_visual.draw_body(self,o.data,camera) and (baizhi_placeholder_visual==null or not baizhi_placeholder_visual.draw_body(self,o.data,camera)):
				draw_circle(at-Vector2(0,23)*scale,10*scale,Color('#deded2'))
				draw_line(at,at-Vector2(0,23)*scale,Color('#deded2'),8*scale)
			_label(at-Vector2(0,1.7*camera.ppm+5*scale),'白芷')
func _supports() -> void:
	if cw_later_owned and snapshot.get('sceneId')=='cw_gear_shaft': return
	var poses = V.support(snapshot.get('supportScene'),snapshot,scene_hash)
	var cw_gear_drawn_support_ids: Array[String] = cw_gear_shaft_environment_visual.draw_supports(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)
	for field in ['standingDecks','movingSupports']:
		for d in scene.get(field,[]):
			if not d is Dictionary or not P.polygon(d.get('polygon')):
				continue
			if not poses.has(d.get('id')):
				continue
			var height = poses[d.id].heightM
			if field=='standingDecks' and (not P.number(d.get('heightM'),0,3) or abs(d.heightM-height)>0.0001):
				continue
			if P.number(height,0,3):
				if cw_gear_shaft_environment_visual.should_skip_legacy_support(self,d,field,poses,cw_gear_drawn_support_ids,snapshot,scene,current_key,scene_hash,camera,viewport_rect):
					continue
				_poly(d.polygon,height,Color('#6c7464'),true)
				_poly(d.polygon,height,Color('#c7cbb4'),false)
func _surfaces() -> void:
	for r in scene.get('terrainRegions',[]):
		if not r is Dictionary or r.get('tag')!='conveyor' or not P.polygon(r.get('polygon')) or not r.get('surfaceVelocityMps') is Array or r.surfaceVelocityMps.size()!=2 or not P.number(r.surfaceVelocityMps[0],-4,4) or not P.number(r.surfaceVelocityMps[1],-4,4):
			continue
		var d = Vector2(r.surfaceVelocityMps[0],r.surfaceVelocityMps[1])
		if d.length()<0.1 or d.length()>4:
			continue
		var center = Vector2.ZERO
		for p in r.polygon:
			center += Vector2(p[0],p[1])
		center /= r.polygon.size()
		var shift = 0.0 if reduced_motion else fmod(float(snapshot.serverTick)/60*d.length(),1.5)
		var from = Vector3(center.x+d.normalized().x*shift,0,center.y+d.normalized().y*shift)
		draw_line(camera.project(from),camera.project(from+Vector3(d.normalized().x,0,d.normalized().y)),Color('#e7c493'),3,true)
func _hazards() -> void:
	for h in snapshot.hazards:
		var e = V.environment(h)
		if _decon_visibility_scope():
			if e.is_empty() or not is_dense_region_available() or not P.polygon(h.get('polygonM')): continue
			var source=PackedVector2Array()
			for p in h.polygonM: source.append(Vector2(float(p[0]),float(p[1])))
			var c = Color('#ffc36b') if e.phase=='warning' else Color('#f07545') if e.phase=='active' else Color('#58d7cf') if e.phase=='suppressed' else Color('#95a8b0')
			for r in utility_plan.parts:
				var clip=PackedVector2Array([r.position,Vector2(r.end.x,r.position.y),r.end,Vector2(r.position.x,r.end.y)])
				for poly in Geometry2D.intersect_polygons(source,clip):
					var world_poly=[]
					for p in poly: world_poly.append([p.x,p.y])
					if P.polygon(world_poly): _poly(world_poly,0,Color(c,0.25 if e.phase=='active' else 0.12),true)
			if P.transform(h.get('transform')) and is_dense_visible_position(P.vector(h.transform.positionM)):
				_label(camera.project(P.vector(h.transform.positionM)),{'heat':'热区','pressure':'蒸汽','toxin':'消杀雾'}[e.tag]+' · '+{'warning':'预警','active':'危险','recovery':'间歇','suppressed':'已抑制'}[e.phase]+' '+str(snappedf(e.remainingMs/1000.0,0.1))+'秒',c)
			continue
		if not e.is_empty():
			var c = Color('#ffc36b') if e.phase=='warning' else Color('#f07545') if e.phase=='active' else Color('#58d7cf') if e.phase=='suppressed' else Color('#95a8b0')
			_poly(h.polygonM,0,Color(c,0.25 if e.phase=='active' else 0.12),true)
			_label(camera.project(P.vector(h.transform.positionM)),{'heat':'热区','pressure':'蒸汽','toxin':'消杀雾'}[e.tag]+' · '+{'warning':'预警','active':'危险','recovery':'间歇','suppressed':'已抑制'}[e.phase]+' '+str(snappedf(e.remainingMs/1000.0,0.1))+'秒',c)
		elif h.kind=='water_depth_slowdown' and h.active and snapshot.sceneId=='mh_drowned_quay' and h.entityId=='mh_water_depth_region':
			_ring(P.vector(snapshot.player.transform.positionM),0.4 if reduced_motion else 0.3+fmod(float(snapshot.serverTick)/108,1)*0.4,Color('#64a9c2'))
		elif h.kind=='moving_machinery' and snapshot.worldId=='clockworks' and snapshot.sceneId=='cw_regulator_core' and h.active and h.get('phaseActive')==true and P.polygon(h.get('polygonM')):
			var regulator = snapshot.actors.filter(func(a): return a.entityId=='cw_prime_regulator' or a.entityType=='runtime2d.enemy.clockworks.forged_guard.v2')
			if regulator.size()==1 and regulator[0].entityId=='cw_prime_regulator' and regulator[0].active and (h.get('warningRemainingMs')==null or P.counter(h.warningRemainingMs,0,10000)):
				_poly(h.polygonM,0,Color(0.8,0.5,0.2,0.2),true)
				_poly(h.polygonM,0,Color('#e7c493'),false)
		elif h.kind=='heat_zone' and h.entityId=='cw_regulator_furnace_heat_zone' and h.get('phaseActive')==true and P.polygon(h.get('polygonM')):
			var boss = false
			for a in snapshot.actors:
				if a.entityId=='cw_prime_regulator' and a.entityType=='runtime2d.enemy.clockworks.forged_guard.v2' and a.active:
					boss = true
			if boss and (h.get('warningRemainingMs')==null or P.counter(h.warningRemainingMs,0,600)):
				_poly(h.polygonM,0,Color(1,0.4,0.2,0.24 if h.active else 0.08),true)
func _warden_boss_context() -> Dictionary:
	var boss=snapshot.get('bossEncounter')
	if snapshot.get('worldId')!='mist_harbor' or snapshot.get('sceneId')!='mh_warden_arena' or not boss is Dictionary or boss.get('entityId')!='mh_resonance_warden_staged_marker' or boss.get('entityType')!='enemy.mist_harbor.resonance_warden' or not actor_visible(boss.entityId) or not P.counter(boss.get('currentHp'),1,100000) or not P.counter(boss.get('maxHp'),1,100000) or boss.currentHp>boss.maxHp or not P.counter(boss.get('phase'),1,2) or not boss.get('state') in ['idle','chase','decoy','windup','recovery','stagger'] or boss.get('temporaryVisual')!=true or boss.get('publicReleaseEligible')!=false or typeof(boss.get('mappedTrueSource'))!=TYPE_BOOL: return {}
	var actors=snapshot.actors.filter(func(a): return a.entityId==boss.entityId)
	if actors.size()!=1 or actors[0].entityType!='runtime2d.enemy.mistharbor.signal_wraith.v2' or not actors[0].active: return {}
	return {'boss':boss,'actor':actors[0]}

func _warden_authorized_warning(boss: Dictionary) -> Dictionary:
	var w=boss.get('warning')
	if not w is Dictionary or typeof(w.get('ordinaryVisible'))!=TYPE_BOOL or not P.counter(w.get('remainingMs'),1,60000) or not P.counter(w.get('attackSerial'),1) or not w.get('originM') is Array or w.originM.size()!=3 or not P.number(w.originM[0],0.5,23.5) or w.originM[1]!=0 or not P.number(w.originM[2],0.5,15.5) or not P.number(w.get('radiusM'),0.001,12) or not P.number(w.get('halfAngleRad'),0.001,PI+0.000001) or not P.number(w.get('directionRad'),-PI-0.000001,PI+0.000001) or not w.get('kind') in ['strike','pulse','decoy']: return {}
	if (w.kind=='decoy')!=(boss.state=='decoy') or w.kind!='decoy' and boss.state!='windup' or boss.mappedTrueSource and (w.kind=='decoy' or not V.mapping(snapshot)): return {}
	if not (w.ordinaryVisible or boss.mappedTrueSource and V.mapping(snapshot)): return {}
	return w

func _mh_warden_body(actor: Dictionary, world_position: Vector3, foot: Vector2) -> bool:
	if actor.get('entityId')!='mh_resonance_warden_staged_marker': return false
	var context=_warden_boss_context()
	if context.is_empty() or context.actor!=actor or not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or scene.get('worldId')!='mist_harbor' or scene.get('sceneId')!='mh_warden_arena' or current_key!='mist_harbor:mh_warden_arena:'+str(snapshot.worldEpoch) or actor.get('actorKind')!='enemy' or not P.transform(actor.get('transform')) or world_position!=P.vector(actor.transform.positionM) or not P.counter(snapshot.get('serverTimeMs')):
		mh_warden_temporal_visual.reset();return false
	if not P.rows(snapshot.get('actors'),2048) or not P.unique(snapshot.actors,'entityId') or not P.rows(scene.get('spawns'),512) or not P.unique(scene.spawns,'id'): mh_warden_temporal_visual.reset();return false
	var authored=scene.spawns.filter(func(row): return row is Dictionary and row.get('id')==actor.entityId)
	if authored.size()!=1 or authored[0].get('kind')!='enemy' or authored[0].get('entityType')!='enemy.mist_harbor.resonance_warden': mh_warden_temporal_visual.reset();return false
	var warning=_warden_authorized_warning(context.boss)
	var prep=not warning.is_empty() and warning.kind in ['strike','pulse']
	mh_warden_sampled=true
	if mh_warden_temporal_visual.draw_body(self,current_key+':'+actor.entityId,int(snapshot.serverTick),int(snapshot.serverTimeMs),world_position,foot,camera.ppm,prep,not reduced_motion): return true
	return mh_cw_actor!=null and mh_cw_actor.draw_body(self,'signal_wraith',foot,camera.ppm)

func _boss_warning() -> void:
	var sentinel = V.sentinel(snapshot)
	if not sentinel.is_empty() and sentinel.get('warning') is Dictionary:
		var w: Dictionary = sentinel.warning
		if w.shape=='circle':
			if not gh_sentinel_circle_warning_visual.draw_warning(self,snapshot,scene,current_key,camera,viewport_rect):
				_ring(P.vector(w.originM),w.radiusM,Color('#f0ad6a'))
		else:
			_corridor(P.vector(w.originM),w.directionRad,w.rangeM,w.radiusM,Color('#f0ad6a'))
		var warning_names = {'light_windup':'轻击蓄势','heavy_windup':'重击蓄势','charge_windup':'冲锋蓄势','charge':'冲锋中'}
		var warning_text = '哨卫 · '+sentinel.phase+' '+str(sentinel.remainingMs)+'ms'
		if warning_names.has(sentinel.phase) and sentinel.remainingMs>0:
			warning_text = '哨卫 · '+warning_names[sentinel.phase]+' · 约'+('%.1f' % (ceil(float(sentinel.remainingMs)/100.0)/10.0))+'秒'
		var warning_at = camera.project(P.vector(sentinel.positionM))-Vector2(0,65*text_scale)
		var sentinel_id = 'gh_sentinel_arena_sentinel_01'
		var same_actor = []
		if P.snapshot(snapshot) and P.rows(snapshot.get('actors'),2048) and P.unique(snapshot.actors,'entityId'):
			same_actor = snapshot.actors.filter(func(a): return a.get('entityId')==sentinel_id and a.get('active')==true and P.transform(a.get('transform')) and P.vector(a.transform.positionM)==P.vector(sentinel.positionM))
		if same_actor.size()==1 and displayed.has(sentinel_id) and actor_visible(sentinel_id):
			_protected_actor_label(sentinel_id,warning_at,warning_text)
		else:
			_label(warning_at,warning_text)
	var context=_warden_boss_context()
	if context.is_empty(): return
	var boss: Dictionary = context.boss
	_protected_actor_label(context.actor.entityId,camera.project(P.vector(context.actor.transform.positionM))-Vector2(0,65*text_scale),'共鸣守望者 · 阶段 '+str(int(boss.phase)))
	var w=_warden_authorized_warning(boss)
	if not w.is_empty():
		var center = Vector3(w.originM[0],0,w.originM[2])
		_ring(center,w.radiusM,Color('#f0ad6a'),w.directionRad-w.halfAngleRad if w.kind=='strike' else 0,w.halfAngleRad*2 if w.kind=='strike' else TAU)
		_label(camera.project(center)-Vector2(0,50),'回声' if w.kind=='decoy' else '共鸣预警 · 移出范围')
# Independent once-only bindings use the same full-image contract as the HUD.
func _action_badge_texture(kind: String) -> AtlasTexture:
	if kind not in ['guard','dash']: return null
	if action_badges_attempted.has(kind): return action_badges.get(kind)
	action_badges_attempted[kind]=true
	var icon=PlayerStatusUiArt.atlas('res://assets/ui/skill_family/'+kind+'.png',Rect2(0,0,1254,1254))
	if icon!=null and icon.atlas!=null and icon.atlas.get_size()==Vector2(1254,1254): action_badges[kind]=icon
	return action_badges.get(kind)

# World-space action art owns its own full-image texture/cache so a guard emblem or
# dash streak never needs the 24 px HUD badge slot. Same 1254x1254 full-image
# contract as _action_badge_texture, but never the badge AtlasTexture instance.
func _action_effect_texture(kind: String) -> Texture2D:
	if kind not in ['guard','dash']: return null
	if action_effects_attempted.has(kind): return action_effects.get(kind)
	action_effects_attempted[kind]=true
	# Prefer the alpha-cut VFX art (transparent background); the HUD skill icon is
	# an opaque sheet and only stays as a compatibility fallback.
	var icon=PlayerStatusUiArt.atlas('res://assets/ui/skill_family/vfx_'+kind+'_alpha_r2.png',Rect2(0,0,1254,1254))
	if icon==null or icon.atlas==null or icon.atlas.get_size()!=Vector2(1254,1254):
		icon=PlayerStatusUiArt.atlas('res://assets/ui/skill_family/vfx_'+kind+'_alpha_r1.png',Rect2(0,0,1254,1254))
	if icon==null or icon.atlas==null or icon.atlas.get_size()!=Vector2(1254,1254):
		icon=PlayerStatusUiArt.atlas('res://assets/ui/skill_family/'+kind+'.png',Rect2(0,0,1254,1254))
	if icon!=null and icon.atlas!=null and icon.atlas.get_size()==Vector2(1254,1254): action_effects[kind]=icon
	return action_effects.get(kind)

func _action_badge_slot(foot: Vector2, label: String) -> Rect2:
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or not camera.initialized or viewport_rect!=get_viewport_rect() or not safe_rect_provider.is_valid() or not P.number(text_scale,0.1,4.0): return Rect2()
	if scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or current_key!=snapshot.worldId+':'+snapshot.sceneId+':'+str(snapshot.worldEpoch): return Rect2()
	if not P.same(action_body_frame,snapshot) or action_body_frame.get('tick')!=snapshot.serverTick or action_body_frame.get('revision')!=snapshot.authorityRevision or action_body_frame.get('player')!=snapshot.player.entityId or action_body_frame.get('foot')!=foot or not action_body_frame.get('rect') is Rect2: return Rect2()
	var body: Rect2=action_body_frame.rect
	var provided=safe_rect_provider.call()
	if not provided is Rect2 or not provided.has_area() or not body.has_area(): return Rect2()
	if not is_finite(provided.position.x) or not is_finite(provided.position.y) or not is_finite(provided.size.x) or not is_finite(provided.size.y): return Rect2()
	var safe: Rect2=provided.intersection(viewport_rect)
	var font=ThemeDB.fallback_font
	var font_size=int(15*text_scale)
	if not safe.has_area() or font==null or font_size<=0: return Rect2()
	var width: float=font.get_string_size(label,HORIZONTAL_ALIGNMENT_LEFT,-1,font_size).x
	var ascent: float=font.get_ascent(font_size)
	var height: float=font.get_height(font_size)
	if not is_finite(width) or not is_finite(ascent) or not is_finite(height) or width<=0.0 or height<=0.0: return Rect2()
	var label_at=foot-Vector2(0,52*text_scale)
	var label_rect=Rect2(label_at-Vector2(width*0.5,ascent),Vector2(width,height))
	var edge: float=24.0*text_scale
	var gap: float=4.0*text_scale
	var size=Vector2(edge,edge)
	for pos in [Vector2(body.end.x+gap,body.get_center().y-edge*0.5),Vector2(body.position.x-gap-edge,body.get_center().y-edge*0.5),Vector2(body.get_center().x-edge*0.5,body.position.y-gap-edge)]:
		var slot=Rect2(pos,size)
		if safe.encloses(slot) and not slot.intersects(body) and not slot.intersects(label_rect): return slot
	return Rect2()
func _action() -> void:
	var p: Dictionary = snapshot.player
	var ap = p.get('actionPresentation')
	if not p.actionState in ['primaryAttack','pulse','guard','pierce','dash'] or not ap is Dictionary or not P.counter(ap.get('requestId'),1) or not ap.get('phase') in ['windup','active','recovery'] or not P.counter(ap.get('durationMs'),1,60000) or not P.counter(ap.get('elapsedMs'),0,60000) or ap.elapsedMs>=ap.durationMs or not P.number(ap.get('rangeM'),0,64) or not P.number(ap.get('lineHalfWidthM'),0,8) or Time.get_ticks_msec()-action_at>=minf(250,ap.durationMs-ap.elapsedMs):
		return
	var pos = P.vector(p.transform.positionM)
	var dir = Vector2(p.facingX,p.facingZ).normalized()
	if dir.length()<0.001:
		return
	var yaw = atan2(dir.x,dir.y)
	var c = Color('#a1deda')
	var art_drawn = false
	if p.actionState in ['primaryAttack','pulse','pierce']:
		var kind = 'basic' if p.actionState=='primaryAttack' else p.actionState
		var factor = 0.78 if p.actionState=='pulse' and ap.phase=='windup' and not reduced_motion else 1.0
		var opacity = clampf(1.0-float(ap.elapsedMs)/float(ap.durationMs),0.0,1.0) if ap.phase=='recovery' else 1.0
		art_drawn = player_action_vfx_art_visual.draw_effect(self,Callable(camera,'project'),pos,dir,kind,ap.phase,float(ap.rangeM),float(ap.lineHalfWidthM),factor,opacity)
	var label: String={'windup':'准备','active':'执行','recovery':'恢复'}[ap.phase]
	if p.actionState=='guard': label={'windup':'防御准备','active':'防御中','recovery':'防御恢复'}[ap.phase]
	elif p.actionState=='dash': label={'windup':'准备','active':'冲刺中','recovery':'冲刺恢复'}[ap.phase]
	var status_icon: AtlasTexture=null
	if p.actionState in ['guard','dash'] and ap.phase=='active':
		status_icon=_action_badge_texture(p.actionState)
		if status_icon!=null:
			var slot=_action_badge_slot(camera.project(pos),label)
			if slot.has_area(): draw_texture_rect(status_icon,slot,false,Color.WHITE)
	match p.actionState:
		'primaryAttack':
			if not art_drawn: _ring(pos,ap.rangeM,c,yaw-PI/3,PI*2/3)
		'guard':
			var guard_art: Texture2D = _action_effect_texture('guard') if not reduced_motion else null
			if guard_art!=null:
				# Body convention (see action_body_frame): 104 px tall at 48 ppm, so ppm is px/metre.
				var body_scale: float = camera.ppm
				var half_extent: float = maxf(0.6,float(ap.rangeM))*body_scale
				var fade: float = lerpf(1.0,0.35,clampf(float(ap.elapsedMs)/float(ap.durationMs),0.0,1.0))
				var center: Vector2 = camera.project(pos)
				if is_finite(half_extent) and half_extent>0.0 and is_finite(center.x) and is_finite(center.y):
					draw_texture_rect(guard_art,Rect2(center-Vector2(half_extent,half_extent),Vector2(half_extent,half_extent)*2.0),false,Color(1,1,1,fade))
			elif ap.phase=='active' and status_icon==null:
				var at = camera.project(pos)+Vector2(0,-28)
				var shield = PackedVector2Array([at+Vector2(-10,-10),at+Vector2(10,-10),at+Vector2(8,4),at+Vector2(0,12),at+Vector2(-8,4),at+Vector2(-10,-10)])
				draw_polyline(shield,c,2.5,true)
		'pierce':
			if not art_drawn: _corridor(pos,yaw,ap.rangeM,ap.lineHalfWidthM,c)
		'pulse':
			if not art_drawn: _ring(pos,ap.rangeM*(0.78 if ap.phase=='windup' and not reduced_motion else 1.0),c)
		'dash':
			var dash_art: Texture2D = _action_effect_texture('dash') if not reduced_motion else null
			if dash_art!=null:
				var reach: float = maxf(0.7,float(ap.rangeM))
				var from: Vector2 = camera.project(pos)
				var to: Vector2 = camera.project(pos-Vector3(dir.x,0,dir.y)*reach)
				var span: Vector2 = to-from
				var dash_fade: float = lerpf(1.0,0.3,clampf(float(ap.elapsedMs)/float(ap.durationMs),0.0,1.0))
				if span.length()>0.001 and is_finite(dash_fade):
					var side: Vector2 = span.normalized().orthogonal()*maxf(6.0,float(ap.lineHalfWidthM)*2.0*camera.ppm)*0.5
					var tint: Color = Color(1,1,1,dash_fade)
					var quad := PackedVector2Array([from+side,from-side,to-side,to+side])
					var uvs := PackedVector2Array([Vector2(0,0),Vector2(0,1),Vector2(1,1),Vector2(1,0)])
					draw_polygon(quad,PackedColorArray([tint,tint,tint,tint]),uvs,dash_art)
			elif ap.phase=='active' and status_icon==null:
				draw_line(camera.project(pos),camera.project(pos-Vector3(dir.x,0,dir.y)*0.7),c,4,true)
	_label(camera.project(pos)-Vector2(0,52*text_scale),label)
func _event_draw() -> void:
	for e in events:
		var pos = P.vector(e.positionM)
		if _region_visibility_scope() and not is_dense_visible_position(pos): continue
		var c = Color('#f1ca8c')
		if e.get('combatFeedback') is Dictionary:
			var outcome: String = e.combatFeedback.outcome
			var at = camera.project(pos)
			if outcome=='rejected':
				_label(at-Vector2(0,30),CombatOutcomeText.rejection_text(e.combatFeedback.get('reason')))
				continue
			if outcome=='blocked':
				_label(at-Vector2(0,24),'格挡',Color('#8de8ff'))
			elif outcome=='absorbed':
				_label(at-Vector2(0,24),'伤害吸收',Color('#a8b2bf'))
			elif outcome=='player_hurt':
				c = Color('#ff8c86')
			draw_line(at-Vector2(5,0),at+Vector2(5,0),c,2)
			draw_line(at-Vector2(0,5),at+Vector2(0,5),c,2)
		elif e.kind in ['EnemyAttackTelegraph','SentinelAttackWindup','SentinelHeavyWindup','SentinelChargeWindup']:
			if e.get('rangeM',0)>0:
				if e.get('attackKind')=='tide_swing' and P.number(e.get('halfAngleRad'),0.001,PI):
					_ring(pos,e.rangeM,c,e.directionRad-e.halfAngleRad,e.halfAngleRad*2)
				elif e.get('attackKind') in ['bite','slam']:
					_ring(pos,e.rangeM,c)
				else:
					_corridor(pos,e.directionRad,e.rangeM,e.radiusM,c)
			else:
				_ring(pos,e.radiusM,c)
		elif 'PressureWindup' in e.kind or 'PressureImpact' in e.kind:
			_ring(pos,e.radiusM,c)
		elif 'Motion' in e.kind:
			draw_circle(camera.project(pos),maxf(3,e.radiusM*camera.ppm*0.25),c)
		elif e.kind in ['EnemyStagger','EnemyDeath','SentinelDeath','ForgedGuardEliteDeath','EnemyAlert','EnemyMemberHit','EnemyMemberDisperse','EnemyAttackImpact','SignalBlink']:
			_label(camera.project(pos)-Vector2(0,35*text_scale),{'EnemyStagger':'硬直','EnemyDeath':'击败','SentinelDeath':'哨卫停用','ForgedGuardEliteDeath':'铸造卫士停用','EnemyAlert':'警觉','EnemyMemberHit':'命中','EnemyMemberDisperse':'散开','EnemyAttackImpact':'接触','SignalBlink':'信号位移'}.get(e.kind,e.kind),c)
func _occluder_fade_to(o: Dictionary) -> float:
	var value = o.get('fadeTo',0.25)
	return float(value) if P.number(value,0,1) else 0.25
func _occlusion() -> void:
	var player = P.vector(snapshot.player.transform.positionM)
	for o in scene.get('occluders',[]):
		if not o is Dictionary or not P.polygon(o.get('polygon')):
			continue
		var poly = PackedVector2Array()
		for p in o.polygon:
			poly.append(Vector2(p[0],p[1]))
		var covered = Geometry2D.is_point_in_polygon(Vector2(player.x,player.z),poly)
		var alpha = occluder_alpha.get(o.get('id'),_occluder_fade_to(o) if covered else 1.0)
		_poly(o.polygon,0,Color(0.3,0.38,0.4,clampf(alpha,0,1)),true)
# Revalidate the historical cue against its current owner and draw frame.
func _sound_cue_draw_current() -> bool:
	if not is_inside_tree() or not is_visible_in_tree() or is_queued_for_deletion() or not P.snapshot(snapshot): return false
	if scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or current_key!=snapshot.worldId+':'+snapshot.sceneId+':'+str(snapshot.worldEpoch) or not camera.initialized or not is_finite(camera.ppm) or camera.ppm<=0.0 or viewport_rect!=get_viewport_rect(): return false
	var host = get_parent()
	if not is_instance_valid(host) or host.is_queued_for_deletion() or host.get('renderer')!=self or host.get('snapshot')!=snapshot or not host.has_method('_eligible') or host.call('_eligible')!=true: return false
	var source = host.get('event_view')
	if not is_instance_valid(source) or source.get('epoch')!=snapshot.worldEpoch or source.get('sound')!=sound: return false
	return AcousticEventView.sound_current(sound,snapshot,Time.get_ticks_msec())
func _world_ui() -> void:
	if P.snapshot(snapshot) and snapshot.get('entryToken')==null and Content.WORLD_NAMES.has(snapshot.worldId) and Content.SCENE_NAMES.has(snapshot.sceneId) and scene.get('worldId')==snapshot.worldId and scene.get('sceneId')==snapshot.sceneId and current_key==snapshot.worldId+':'+snapshot.sceneId+':'+str(snapshot.worldEpoch):
		var tiers = snapshot.capabilities.get('enemyVitals')
		if P.rows(tiers,512) and P.unique(tiers,'entityId'):
			for v in tiers:
				if typeof(v.get('tier'))!=TYPE_STRING or not v.tier in ['healthy','wounded','severelyWounded','critical'] or not displayed.has(v.entityId) or not actor_visible(v.entityId):
					continue
				for a in snapshot.actors:
					var enemy = a.actorKind=='enemy' or a.actorKind=='sentinel' and a.entityType=='enemy.grey_hive.sentinel' and a.entityId=='gh_sentinel_arena_sentinel_01' and snapshot.worldId=='grey_hive' and snapshot.sceneId=='gh_sentinel_arena'
					if a.entityId==v.entityId and a.active and enemy and P.transform(a.transform):
						var vital_text = {'healthy':'健康','wounded':'受伤','severelyWounded':'重伤','critical':'濒危'}[v.tier]
						var vital_position = _gh_vital_caption_position(a,camera.project(P.vector(a.transform.positionM))-Vector2(0,55*text_scale))
						_label(_vital_caption_position(a.entityId,vital_position,vital_text),vital_text)
	if not sound.is_empty() and _sound_cue_draw_current():
		var dir = Vector2(sin(sound.directionRad)-cos(sound.directionRad),sin(sound.directionRad)+cos(sound.directionRad)).normalized()
		var at = camera.project(P.vector(snapshot.player.transform.positionM))+dir*70
		draw_line(at-dir*10,at+dir*10,Color('#bcd9e2'),3)
		var distance = '近处' if sound.distanceM<2 else '约 5 米' if sound.distanceM<7.5 else '约 '+str(int(round(sound.distanceM/10)*10))+' 米'
		_label(at+Vector2(0,-15),('当时听到的真实声源' if sound.kind=='warden_true_call' else '当时听到的信号')+' · 当时'+distance)
func _draw_aim_reticle() -> void:
	if aim_reticle.is_empty() or not P.snapshot(snapshot) or not P.same(aim_reticle.get('context',{}),snapshot) or aim_reticle.get('tick')!=snapshot.serverTick or aim_reticle.get('revision')!=snapshot.authorityRevision:
		return
	if snapshot.get('entryToken')!=null or snapshot.player.currentHp<=0 or not camera.initialized or not is_finite(camera.ppm) or camera.ppm<=0.0:
		return
	var screen: Vector2 = aim_reticle.screen
	if not viewport_rect.has_point(screen):
		return
	var player = P.vector(snapshot.player.transform.positionM)
	var ground: Vector3 = camera.inverse(screen,player.y)
	var b = scene.get('boundsM')
	if not b is Dictionary or not is_finite(ground.x) or not is_finite(ground.y) or not is_finite(ground.z):
		return
	if ground.x<b.x or ground.x>b.x+b.width or ground.z<b.z or ground.z>b.z+b.depth:
		return
	var at = camera.project(ground)
	if not is_finite(at.x) or not is_finite(at.y) or not viewport_rect.has_point(at):
		return
	var color = Color('#c6dce2')
	draw_arc(at,7.0,0.0,TAU,20,color,1.5,true)
	draw_line(at+Vector2(-11,0),at+Vector2(-4,0),color,1.5,true)
	draw_line(at+Vector2(4,0),at+Vector2(11,0),color,1.5,true)
	draw_line(at+Vector2(0,-11),at+Vector2(0,-4),color,1.5,true)
	draw_line(at+Vector2(0,4),at+Vector2(0,11),color,1.5,true)
func _draw_interaction_focus() -> void:
	if interaction_focus.is_empty() or interaction_focus.size()!=4:
		return
	var window = get_window()
	if not is_instance_valid(window) or not window.has_focus():
		return
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.player.currentHp<=0:
		return
	if scene.is_empty() or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or current_key!=snapshot.worldId+':'+snapshot.sceneId+':'+str(snapshot.worldEpoch):
		return
	var target = interaction_focus.get('target')
	if not target is Dictionary or target.size()!=5 or not target.has('generation') or not target.has('context') or not target.has('entityId') or not target.has('kind') or not target.has('label'):
		return
	if not P.counter(interaction_focus.get('generation'),1) or not P.counter(target.get('generation'),1) or interaction_focus.generation!=target.generation:
		return
	var target_context = target.get('context')
	if not P.context(target_context) or not P.same(target_context,snapshot):
		return
	if not P.ident(target.get('entityId')) or typeof(target.get('kind'))!=TYPE_STRING or typeof(target.get('label'))!=TYPE_STRING:
		return
	if target.kind.is_empty() or target.label.is_empty():
		return
	if not P.counter(interaction_focus.get('serverTick')) or not P.counter(interaction_focus.get('authorityRevision')) or interaction_focus.serverTick!=snapshot.serverTick or interaction_focus.authorityRevision!=snapshot.authorityRevision:
		return
	var rows = snapshot.get('interactables')
	if not P.rows(rows,2048) or not P.unique(rows,'entityId'):
		return
	var matches: Array = rows.filter(func(row): return row is Dictionary and row.get('entityId')==target.entityId and row.get('kind')==target.kind)
	if matches.size()!=1:
		return
	var row: Dictionary = matches[0]
	if typeof(row.get('active'))!=TYPE_BOOL or not row.active or not P.transform(row.get('transform')):
		return
	var position: Vector3 = P.vector(row.transform.positionM)
	if _region_visibility_scope() and not _dense_entity_allowed(row.get('entityId'),position): return
	if _occluded(position):
		return
	if not is_finite(camera.ppm) or camera.ppm<=0:
		return
	var projected: Vector2 = camera.project(position)
	if not is_finite(projected.x) or not is_finite(projected.y):
		return
	var scale = camera.ppm/48.0
	if not is_finite(scale) or scale<=0:
		return
	var rect = Rect2(projected-Vector2(12,20)*scale,Vector2(24,24)*scale)
	if not is_finite(rect.position.x) or not is_finite(rect.position.y) or not is_finite(rect.size.x) or not is_finite(rect.size.y):
		return
	if not viewport_rect.encloses(rect):
		return
	draw_rect(rect,Color('#D9AD58'),false,2.0)
func _atmosphere(suppressed_bank_ids: Array = []) -> void:
	if snapshot.worldId=='mist_harbor':
		for prop in scene.get('presentation',{}).get('sprites',[]):
			if prop is Dictionary and prop.get('assetId') in ['runtime2d.world.mistharbor.fog_bank.v1','runtime2d.world.mistharbor.fog_vfx.v1'] and prop.get('position') is Array and prop.position.size()==3:
				var bank_id = prop.get('id')
				var bank_position = prop.get('position')
				if mh_extraction_environment_visual.legacy_bank_consumed(self,prop,snapshot,scene,current_key,scene_hash,camera,viewport_rect,suppressed_bank_ids):
					continue
				if mh_resonance_tower_environment_visual.legacy_bank_consumed(self,prop,snapshot,scene,current_key,scene_hash,camera,viewport_rect,suppressed_bank_ids):
					continue
				if mh_breakwater_environment_visual.legacy_bank_consumed(self,prop,snapshot,scene,current_key,camera,viewport_rect,suppressed_bank_ids):
					continue
				if mh_drowned_quay_environment_visual.legacy_bank_consumed(self,prop,snapshot,scene,current_key,camera,viewport_rect,suppressed_bank_ids):
					continue
				if mh_signal_yard_environment_visual.should_suppress_bank(prop,snapshot,scene,current_key,camera,viewport_rect,suppressed_bank_ids):
					continue
				if mh_tidal_warehouse_environment_visual.legacy_bank_consumed(prop,snapshot,scene,current_key,camera,viewport_rect,suppressed_bank_ids):
					continue
				if mh_warden_arena_environment_visual.legacy_bank_consumed(prop,snapshot,scene,current_key,scene_hash,camera,viewport_rect,suppressed_bank_ids):
					continue
				if prop.get('assetId')=='runtime2d.world.mistharbor.fog_bank.v1' and typeof(bank_id)==TYPE_STRING and bank_id in suppressed_bank_ids and prop.get('layer')=='visual.props_back' and ((bank_id=='approved_fog_bank_north' and P.number(bank_position[0],13.0,13.0) and P.number(bank_position[1],0.0,0.0) and P.number(bank_position[2],2.5,2.5)) or (bank_id=='approved_fog_bank_east' and P.number(bank_position[0],20.0,20.0) and P.number(bank_position[1],0.0,0.0) and P.number(bank_position[2],11.0,11.0))):
					continue
				var at = camera.project(Vector3(prop.position[0],prop.position[1],prop.position[2]))
				draw_circle(at,80*camera.ppm/48,Color(0.6,0.7,0.74,0.055))
		var pump = snapshot.get('mistHarborPump')
		if snapshot.sceneId=='mh_pump_station' and pump is Dictionary and pump.get('state') in ['ready','draining','drained']:
			var at = camera.project(Vector3(17,0,7))
			if pump.state=='draining' and not reduced_motion:
				at.x += sin(float(snapshot.serverTick)/15)*0.8
			draw_circle(at,9,Color('#efbc6c') if pump.state=='ready' else Color('#61ddcf'))
			_label(at-Vector2(0,24),{'ready':'泵站就绪','draining':'排水中','drained':'排水完成'}[pump.state])

func _report_frame(key: String) -> void:
	if key==current_key:
		frame_committed.emit(key)

func _update_occlusion(delta: float) -> void:
	if snapshot.is_empty():
		return
	var player = P.vector(snapshot.player.transform.positionM)
	for o in scene.get('occluders',[]):
		if not o is Dictionary or not P.ident(o.get('id')) or not P.polygon(o.get('polygon')):
			continue
		var poly = PackedVector2Array()
		for p in o.polygon:
			poly.append(Vector2(p[0],p[1]))
		var target = _occluder_fade_to(o) if Geometry2D.is_point_in_polygon(Vector2(player.x,player.z),poly) else 1.0
		occluder_alpha[o.id] = move_toward(float(occluder_alpha.get(o.id,1.0)),target,delta*3.75)
func _rear_view() -> void:
	var grant = snapshot.capabilities.get('rearView')
	if not grant is Dictionary or typeof(grant.get('granted'))!=TYPE_BOOL or not grant.granted or not P.ident(grant.get('grantId')):
		return
	var revision = grant.get('grantedAtRevision')
	if not revision is Dictionary or not P.counter(revision.get('worldEpoch')) or not P.counter(revision.get('serverTick')) or not P.counter(revision.get('authorityRevision')) or revision.worldEpoch>snapshot.worldEpoch or revision.worldEpoch==snapshot.worldEpoch and (revision.serverTick>snapshot.serverTick or revision.serverTick==snapshot.serverTick and revision.authorityRevision>snapshot.authorityRevision):
		return
	var player = P.vector(snapshot.player.transform.positionM)
	var facing = Vector2(snapshot.player.facingX,snapshot.player.facingZ).normalized()
	if facing.length()<0.001:
		return
	var rect = Rect2(Vector2(18,viewport_rect.size.y-310*text_scale),Vector2(230,120)*text_scale)
	if not viewport_rect.encloses(rect):
		return
	draw_rect(rect,Color('#16202a'))
	_label(rect.position+Vector2(115,23)*text_scale,'后方视野 · 当前威胁')
	var count = 0
	for a in snapshot.actors:
		if not a.active or a.actorKind=='player':
			continue
		for p in V.actor_positions(a,snapshot):
			var pos = P.vector(p)
			var offset = Vector2(pos.x-player.x,pos.z-player.z)
			if offset.dot(facing)>=0 or not visible_point(pos) or _occluded(pos):
				continue
			var dot_position = rect.get_center()+Vector2(offset.x-offset.y,(offset.x+offset.y)*0.5).limit_length(4)*12*text_scale
			draw_circle(dot_position,4*text_scale,Color('#e6ba89'))
			count += 1
			if count>=32:
				return

func _beacon() -> void:
	var beacon = snapshot.get('greyHiveBeacon')
	if snapshot.worldId!='grey_hive' or not beacon is Dictionary or beacon.size()!=2 or not beacon.get('state') in ['uncollected','carried','mounted'] or typeof(beacon.get('legacyCompletedWithoutReceipt'))!=TYPE_BOOL or beacon.legacyCompletedWithoutReceipt and beacon.state!='uncollected':
		return
	if beacon.legacyCompletedWithoutReceipt:
		return
	if beacon.state=='carried':
		var at = camera.project(P.vector(snapshot.player.transform.positionM))+Vector2(15,-10)
		draw_circle(at,4,Color('#b9eef2'))
	else:
		if snapshot.sceneId!='gh_beacon':
			return
		var id = 'gh_beacon_deploy_marker' if beacon.state=='uncollected' else 'gh_beacon_storage_mount_marker'
		for i in snapshot.interactables:
			if i.entityId==id:
				var at = camera.project(P.vector(i.transform.positionM))
				var drawn_key = 'beacon-body:'+id
				if not prop_bodies_drawn.has(drawn_key):
					draw_rect(Rect2(at-Vector2(7,18 if beacon.state=='mounted' else 8),Vector2(14,18 if beacon.state=='mounted' else 8)),Color('#b9eef2'))
				var caption_height = maxf(30,prop_bodies_drawn.get(drawn_key,0)+10*text_scale)
				_label(at-Vector2(0,caption_height),'便携信标 · '+('已部署' if beacon.state=='mounted' else '待收取'))

func _gh_story_prop(prop: Dictionary) -> Dictionary:
	if snapshot.get('worldId')!='grey_hive' or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!=snapshot.get('sceneId') or not GH_STORY_PLACEMENTS.has(prop.get('id')):
		return {}
	var selected: Dictionary = GH_STORY_PLACEMENTS[prop.id]
	if snapshot.sceneId!=selected.scene or prop.get('assetId')!=selected.asset or prop.get('layer')!='visual.props_dynamic' or not _gh_authored_position(prop.get('position'),selected.position):
		return {}
	var sprites = scene.get('presentation',{}).get('sprites')
	if not P.rows(sprites,2048):
		return {}
	var same_id: Array = sprites.filter(func(row): return row is Dictionary and row.get('id')==prop.id)
	if same_id.size()!=1 or same_id[0]!=prop:
		return {}
	if prop.id=='exit_extraction_console_approved':
		if _exit_source_scope():
			if not _exit_source_current(): return {}
		elif not gh_exit_environment_visual.matches_terminal_row(prop,snapshot,scene,current_key,scene_hash): return {}
	var foot: Vector3 = selected.position
	var image_allowed = true
	if prop.id=='lockdown_terminal_visual':
		var authored = scene.get('interactions')
		if not P.rows(authored,2048):
			return {}
		var operation: Array = authored.filter(func(row): return row is Dictionary and row.get('id')=='gh_lockdown_terminal')
		if operation.size()!=1 or operation[0].get('kind')!='lockdown_terminal' or operation[0].get('event')!='hive_lockdown' or not _gh_authored_position(operation[0].get('position'),Vector3(4,0,8)):
			return {}
		var interactables = snapshot.get('interactables')
		if not P.rows(interactables,2048):
			return {}
		var current: Array = interactables.filter(func(row): return row is Dictionary and row.get('entityId')=='gh_lockdown_terminal')
		if current.size()!=1 or current[0].get('kind')!='lockdown_terminal' or typeof(current[0].get('active'))!=TYPE_BOOL or not P.transform(current[0].get('transform')):
			return {}
		foot = P.vector(current[0].transform.positionM)
		var progress = Content.progress(snapshot,'grey_hive')
		image_allowed = current[0].active and not progress.is_empty() and not 'hive_lockdown' in progress.completedEvents
	return {'id':prop.id,'p':foot,'kind':'gh_story_prop','data':{'key':selected.key,'image_allowed':image_allowed}}
func _gh_crate_selected(prop: Dictionary) -> bool:
	if snapshot.get('worldId')!='grey_hive' or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!=snapshot.get('sceneId') or not GH_CRATES.has(prop.get('id')):
		return false
	var selected: Array = GH_CRATES[prop.id]
	if snapshot.sceneId!=selected[0] or prop.get('assetId')!='runtime2d.prop.supply_crates.v1' or prop.get('layer')!='visual.props_back' or not _gh_authored_position(prop.get('position'),selected[1]):
		return false
	var sprites = scene.get('presentation',{}).get('sprites')
	if not P.rows(sprites,2048):
		return false
	var same_id: Array = sprites.filter(func(row): return row is Dictionary and row.get('id')==prop.id)
	return same_id.size()==1 and same_id[0]==prop
func _gh_exit_body_grounding_ids() -> Array[String]:
	var body_ids: Array[String] = []
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_exit' or not P.counter(snapshot.get('worldEpoch'),1) or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_exit' or current_key!='grey_hive:gh_exit:'+str(snapshot.get('worldEpoch')):
		return body_ids
	var presentation_value: Variant = scene.get('presentation')
	if not presentation_value is Dictionary:
		return body_ids
	var sprite_rows: Variant = presentation_value.get('sprites')
	if not P.rows(sprite_rows,2048):
		return body_ids
	var rows_by_id: Dictionary = {}
	var counts: Dictionary = {}
	for row_value: Variant in sprite_rows:
		if not row_value is Dictionary:
			return body_ids
		var row: Dictionary = row_value
		var row_id: Variant = row.get('id')
		if typeof(row_id)!=TYPE_STRING:
			return body_ids
		if row_id in ['exit_extraction_console_approved','exit_console_crates']:
			counts[row_id] = int(counts.get(row_id,0))+1
			rows_by_id[row_id] = row
	if counts.get('exit_extraction_console_approved',0)==1 and gh_bio_lockdown_visual!=null and is_instance_valid(gh_bio_lockdown_visual):
		var terminal_attempted: Variant = gh_bio_lockdown_visual.get('attempted')
		var terminal_textures: Variant = gh_bio_lockdown_visual.get('textures')
		var terminal_texture: Variant = terminal_textures.get('lockdown_terminal') if terminal_textures is Dictionary else null
		var terminal_row: Dictionary = rows_by_id['exit_extraction_console_approved']
		var terminal_body: Dictionary = _gh_story_prop(terminal_row)
		var terminal_data: Variant = terminal_body.get('data')
		var terminal_allowed: Variant = terminal_data.get('image_allowed') if terminal_data is Dictionary else null
		if typeof(terminal_attempted)==TYPE_BOOL and terminal_attempted and terminal_texture is Texture2D and is_instance_valid(terminal_texture) and terminal_texture.get_width()==1270 and terminal_texture.get_height()==1086 and typeof(terminal_allowed)==TYPE_BOOL and terminal_allowed and terminal_body.get('p') is Vector3:
			var terminal_position: Vector3 = terminal_body.get('p')
			var terminal_culling: Dictionary = _object_culling_state(terminal_body,camera.project(terminal_position))
			var terminal_culled: Variant = terminal_culling.get('culled')
			if typeof(terminal_culled)==TYPE_BOOL and not terminal_culled:
				body_ids.append('exit_extraction_console_approved')
	if counts.get('exit_console_crates',0)==1 and gh_supply_crate_visual!=null and is_instance_valid(gh_supply_crate_visual):
		var crate_attempted: Variant = gh_supply_crate_visual.get('attempted')
		var crate_texture: Variant = gh_supply_crate_visual.get('texture')
		var crate_row: Dictionary = rows_by_id['exit_console_crates']
		if typeof(crate_attempted)==TYPE_BOOL and crate_attempted and crate_texture is Texture2D and is_instance_valid(crate_texture) and crate_texture.get_width()==1238 and crate_texture.get_height()==1233 and _gh_crate_selected(crate_row):
			var crate_position_value: Variant = crate_row.get('position')
			if crate_position_value is Array and crate_position_value.size()==3:
				var crate_body: Dictionary = {'id':'exit_console_crates','p':Vector3(crate_position_value[0],crate_position_value[1],crate_position_value[2]),'kind':'prop','data':crate_row}
				var crate_culling: Dictionary = _object_culling_state(crate_body,camera.project(crate_body.p))
				var crate_culled: Variant = crate_culling.get('culled')
				if typeof(crate_culled)==TYPE_BOOL and not crate_culled:
					body_ids.append('exit_console_crates')
	return body_ids
func _gh_authored_position(value, expected: Vector3) -> bool:
	if not value is Array or value.size()!=3:
		return false
	for coordinate in value:
		if not P.number(coordinate):
			return false
	return Vector3(value[0],value[1],value[2])==expected
func _terminal_position(prop: Dictionary):
	if snapshot.worldId!='return_station' or snapshot.sceneId!='rs_core_room' or prop.get('layer')!='visual.props_back':
		return null
	var positions = {'mission_terminal_art':Vector3(8,0,4),'capability_terminal_art':Vector3(15,0,4),'storage_terminal_art':Vector3(15,0,11),'world_gate_terminal_art':Vector3(20,0,6),'save_rest_terminal_art':Vector3(8,0,11)}
	var asset_ids = {'mission_terminal_art':'runtime2d.world.returnstation.terminal.v1','capability_terminal_art':'runtime2d.world.returnstation.terminal.v1','storage_terminal_art':'runtime2d.world.returnstation.terminal.v1','world_gate_terminal_art':'runtime2d.world.returnstation.terminals.v1','save_rest_terminal_art':'runtime2d.world.returnstation.terminals.v1'}
	var prop_id = prop.get('id')
	if not positions.has(prop_id) or prop.get('assetId')!=asset_ids[prop_id] or not prop.get('position') is Array or prop.position.size()!=3:
		return null
	for value in prop.position:
		if not P.number(value):
			return null
	var position = Vector3(prop.position[0],prop.position[1],prop.position[2])
	return position if position==positions[prop_id] else null

func _ground_beacon_body() -> Dictionary:
	var beacon = snapshot.get('greyHiveBeacon')
	if snapshot.worldId!='grey_hive' or snapshot.sceneId!='gh_beacon' or not beacon is Dictionary or beacon.size()!=2 or not beacon.get('state') in ['uncollected','carried','mounted'] or typeof(beacon.get('legacyCompletedWithoutReceipt'))!=TYPE_BOOL or beacon.legacyCompletedWithoutReceipt or beacon.state=='carried':
		return {}
	var folded = beacon.state=='uncollected'
	var art_id = 'beacon_folded_approved' if folded else 'beacon_mounted_approved'
	var asset_id = 'runtime2d.prop.beacon.folded.v1' if folded else 'runtime2d.prop.beacon.deployed.v1'
	var art = scene.get('presentation',{}).get('sprites',[]).filter(func(row): return row is Dictionary and row.get('id')==art_id and row.get('assetId')==asset_id and row.get('layer')=='visual.props_dynamic')
	if art.size()!=1:
		return {}
	var marker_id = 'gh_beacon_deploy_marker' if folded else 'gh_beacon_storage_mount_marker'
	var markers = snapshot.interactables.filter(func(row): return row.entityId==marker_id)
	if markers.size()!=1 or not P.transform(markers[0].get('transform')):
		return {}
	return {'id':'beacon-body:'+marker_id,'p':P.vector(markers[0].transform.positionM),'kind':'beacon_body','data':{'key':'beacon_folded' if folded else 'beacon_deployed'}}

func _occluded(pos: Vector3) -> bool:
	for o in scene.get('occluders',[]):
		if not o is Dictionary or not P.polygon(o.get('polygon')) or float(occluder_alpha.get(o.get('id'),1.0))<=0.5:
			continue
		var poly = PackedVector2Array()
		for p in o.polygon:
			poly.append(Vector2(p[0],p[1]))
		if Geometry2D.is_point_in_polygon(Vector2(pos.x,pos.z),poly):
			return true
	return false

func _wraith_drop(id: String) -> void:
	if mh_wraith_temporal_by_id.has(id):
		mh_wraith_temporal_by_id[id].reset()
		mh_wraith_temporal_by_id.erase(id)

func _wraith_reset_all() -> void:
	for id in mh_wraith_temporal_by_id.keys(): _wraith_drop(id)

# Retire each instance independently before draw, including disappearance.
func _wraith_reconcile() -> void:
	var rows = snapshot.get('actors')
	if not rows is Array:
		_wraith_reset_all()
		return
	for id in mh_wraith_temporal_by_id.keys():
		var matches: Array = rows.filter(func(row): return row is Dictionary and row.get('entityId')==id)
		if matches.size()!=1:
			_wraith_drop(id)
			continue
		var a: Dictionary = matches[0]
		var perception = a.get('signalPerception')
		if not perception is Dictionary or perception.get('precise')!=true:
			_wraith_drop(id)
			continue
		if _mh_cw_body_key(a)!='signal_wraith' or not actor_visible(id): _wraith_drop(id)

# EventView.project has already authorized the event. Apply its same current
# expiry calculation and pass only this actor's supported, unexpired events.
func _cw_drone_hound_body(actor: Dictionary, world_position: Vector3, foot: Vector2, family: String) -> bool:
	if not CW_DRONE_HOUND_READY or not P.snapshot(snapshot) or snapshot.worldId!='clockworks' or scene.get('worldId')!='clockworks' or scene.get('sceneId')!=snapshot.sceneId or current_key!='clockworks:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch): return false
	var families: Dictionary = CW_DRONE_HOUND_SCENES.get(snapshot.sceneId,{})
	if not families.has(family) or actor.get('entityType')!='enemy.clockworks.'+family or actor.get('actorKind')!='enemy' or actor.get('active')!=true or not P.transform(actor.get('transform')) or not P.counter(snapshot.get('serverTimeMs')): return false
	var valid_id=false
	for index in range(1,int(families[family])+1):
		if actor.get('entityId')==snapshot.sceneId+'_'+family+'_%02d'%index: valid_id=true
	if not valid_id or world_position!=P.vector(actor.transform.positionM) or not actor_visible(actor.entityId): return false
	if not P.rows(snapshot.get('actors'),2048) or not P.unique(snapshot.actors,'entityId') or not P.rows(scene.get('spawns'),512) or not P.unique(scene.spawns,'id'): return false
	var live: Array = snapshot.actors.filter(func(row): return row.get('entityId')==actor.entityId)
	var authored: Array = scene.spawns.filter(func(row): return row.get('id')==actor.entityId)
	if live.size()!=1 or live[0]!=actor or authored.size()!=1 or authored[0].get('kind')!='enemy' or authored[0].get('entityType')!=actor.entityType: return false
	if not cw_drone_hound_by_id.has(actor.entityId): cw_drone_hound_by_id[actor.entityId]=CwDroneHoundTemporalVisual.new()
	cw_drone_hound_sampled[actor.entityId]=true
	return cw_drone_hound_by_id[actor.entityId].draw_body(self,family,current_key+':'+actor.entityId,int(snapshot.serverTick),int(snapshot.serverTimeMs),actor,foot,camera.ppm,_cw_drone_hound_events(actor.entityId,family),not reduced_motion)

func _cw_drone_hound_events(id: String, family: String) -> Array:
	# Only the already-authorized EventView.project output is considered here.
	var selected: Array = []
	var now: int = Time.get_ticks_msec()
	for event in events:
		if not event is Dictionary or event.get('actorId')!=id or event.get('worldEpoch')!=snapshot.worldEpoch: continue
		if not P.counter(event.get('eventId'),1) or not P.counter(event.get('serverTick')) or event.serverTick>snapshot.serverTick or not P.counter(event.get('durationMs'),1,60000): continue
		if not P.number(event.get('_duration'),0.001,60000) or not P.number(event.get('_initial_age'),0,60000) or not P.counter(event.get('_accepted')) or event._accepted>now: continue
		var attack: String = str(event.get('attackKind',''))
		if family=='pressure_drone' and attack!='pressure_shot' or family=='furnace_hound' and attack not in ['bite','leap']: continue
		var kind: String = str(event.get('kind',''))
		if kind not in ['EnemyAttackTelegraph','EnemyAttackImpact'] and not (family=='pressure_drone' and kind=='PressureShotMotion') and not (family=='furnace_hound' and kind=='FurnaceHoundLeapMotion' and attack=='leap'): continue
		var age: float = maxf((float(snapshot.serverTick)-float(event.serverTick))*1000.0/60.0,float(event._initial_age)+float(now-int(event._accepted)))
		if age<float(event._duration): selected.append(event)
	selected.sort_custom(func(a,b): return a.serverTick<b.serverTick or a.serverTick==b.serverTick and a.eventId<b.eventId)
	return selected

func _mh_drowned_tidebound_body(actor: Dictionary, world_position: Vector3, foot: Vector2) -> bool:
	if not MH_DROWNED_TIDEBOUND_READY or not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.worldId!='mist_harbor' or scene.get('worldId')!='mist_harbor' or scene.get('sceneId')!=snapshot.sceneId or current_key!='mist_harbor:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch): return false
	if actor.get('actorKind')!='enemy' or actor.get('active')!=true or not P.transform(actor.get('transform')) or not P.counter(snapshot.get('serverTimeMs')) or world_position!=P.vector(actor.transform.positionM) or not actor_visible(actor.entityId): return false
	var family=''
	if actor.get('entityType')=='runtime2d.enemy.mistharbor.drowned.v2' and _mh_cw_body_key(actor)=='drowned': family='drowned'
	elif actor.get('entityType')=='enemy.mist_harbor.tidebound' and actor.get('entityId') in MH_TIDEBOUND_IDS.get(snapshot.sceneId,[]): family='tidebound'
	if family.is_empty() or not P.rows(snapshot.get('actors'),2048) or not P.unique(snapshot.actors,'entityId') or not P.rows(scene.get('spawns'),512) or not P.unique(scene.spawns,'id'): return false
	var live: Array = snapshot.actors.filter(func(row): return row is Dictionary and row.get('entityId')==actor.entityId)
	var authored: Array = scene.spawns.filter(func(row): return row is Dictionary and row.get('id')==actor.entityId)
	if live.size()!=1 or live[0]!=actor or authored.size()!=1 or authored[0].get('kind')!='enemy' or authored[0].get('entityType')!='enemy.mist_harbor.'+family: return false
	if not mh_drowned_tidebound_by_id.has(actor.entityId): mh_drowned_tidebound_by_id[actor.entityId]=MhDrownedTideboundTemporalVisual.new()
	mh_drowned_tidebound_sampled[actor.entityId]=true
	return mh_drowned_tidebound_by_id[actor.entityId].draw_body(self,family,current_key+':'+actor.entityId,int(snapshot.serverTick),int(snapshot.serverTimeMs),actor,foot,camera.ppm,_mh_tidebound_events(actor.entityId) if family=='tidebound' else [],not reduced_motion)

func _mh_tidebound_events(id: String) -> Array:
	# Consume only the already-authorized EventView.project output.
	var selected: Array = []
	var now: int = Time.get_ticks_msec()
	for event in events:
		if not event is Dictionary or event.get('actorId')!=id or event.get('worldEpoch')!=snapshot.worldEpoch: continue
		if not P.counter(event.get('eventId'),1) or not P.counter(event.get('serverTick')) or event.serverTick>snapshot.serverTick or not P.counter(event.get('durationMs'),1,60000): continue
		if not P.number(event.get('_duration'),0.001,60000) or not P.number(event.get('_initial_age'),0,60000) or not P.counter(event.get('_accepted')) or event._accepted>now: continue
		var attack: String = str(event.get('attackKind',''))
		var kind: String = str(event.get('kind',''))
		if attack not in ['tide_swing','tide_charge'] or kind not in ['EnemyAttackTelegraph','EnemyChargeMotion','EnemyAttackImpact'] or kind=='EnemyChargeMotion' and attack!='tide_charge': continue
		var age: float = maxf((float(snapshot.serverTick)-float(event.serverTick))*1000.0/60.0,float(event._initial_age)+float(now-int(event._accepted)))
		if age<float(event._duration): selected.append(event)
	selected.sort_custom(func(a,b): return a.serverTick<b.serverTick or a.serverTick==b.serverTick and a.eventId<b.eventId)
	return selected

func _wraith_events(id: String) -> Array:
	var selected: Array = []
	var now: int = Time.get_ticks_msec()
	for event in events:
		if not event is Dictionary or event.get('actorId')!=id or event.get('worldEpoch')!=snapshot.worldEpoch: continue
		if not P.counter(event.get('eventId'),1) or not P.counter(event.get('serverTick')) or event.serverTick>snapshot.serverTick or not P.counter(event.get('durationMs'),1,60000): continue
		if not P.number(event.get('_duration'),0.001,60000) or not P.number(event.get('_initial_age'),0,60000) or not P.counter(event.get('_accepted')) or event._accepted>now: continue
		var supported: bool = event.get('kind')=='SignalBlink' or event.get('kind') in ['EnemyAttackTelegraph','SignalShotMotion','EnemyAttackImpact'] and event.get('attackKind')=='signal_shot'
		if not supported: continue
		var age: float = maxf((float(snapshot.serverTick)-float(event.serverTick))*1000.0/60.0,float(event._initial_age)+float(now-int(event._accepted)))
		if age<float(event._duration): selected.append(event)
	return selected
func _swarm_actor_matches(actor: Dictionary) -> bool:
	if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or not snapshot.sceneId in ['gh_deep_decon','gh_lockdown'] or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or current_key!='grey_hive:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch): return false
	if actor.get('entityType')!='enemy.grey_hive.swarm' or actor.get('actorKind')!='enemy' or actor.get('active')!=true or not actor.get('entityId') in [snapshot.sceneId+'_swarm_01',snapshot.sceneId+'_swarm_02'] or not P.transform(actor.get('transform')) or V.swarm_members(actor).is_empty(): return false
	if not P.rows(snapshot.get('actors'),2048) or not P.unique(snapshot.actors,'entityId') or not P.rows(scene.get('spawns'),512) or not P.unique(scene.spawns,'id'): return false
	var live=snapshot.actors.filter(func(row): return row.get('entityId')==actor.entityId)
	var authored=scene.spawns.filter(func(row): return row.get('id')==actor.entityId)
	return live.size()==1 and live[0]==actor and authored.size()==1 and authored[0].get('kind')=='enemy' and authored[0].get('entityType')=='enemy.grey_hive.swarm'

func _gate_b_brute_matches(actor: Dictionary) -> bool:
	if not P.snapshot(snapshot) or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_gate_b' or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_gate_b' or current_key!='grey_hive:gh_gate_b:'+str(snapshot.worldEpoch): return false
	if actor.get('entityId')!='gh_gate_b_brute_01' or actor.get('entityType')!='enemy.grey_hive.brute' or actor.get('actorKind')!='enemy' or actor.get('active')!=true or not P.transform(actor.get('transform')): return false
	if not P.rows(snapshot.get('actors'),2048) or not P.unique(snapshot.actors,'entityId') or not P.rows(scene.get('spawns'),512) or not P.unique(scene.spawns,'id'): return false
	var live: Array = snapshot.actors.filter(func(row): return row.get('entityId')=='gh_gate_b_brute_01')
	var authored: Array = scene.spawns.filter(func(row): return row.get('id')=='gh_gate_b_brute_01')
	return live.size()==1 and live[0]==actor and authored.size()==1 and authored[0].get('kind')=='enemy' and authored[0].get('entityType')=='enemy.grey_hive.brute'

func _gate_b_brute_events() -> Array:
	# These are EventView.project outputs, not the unfiltered event transport.
	# Recheck its existing expiry formula against this exact snapshot and time.
	var selected: Array = []
	var now: int = Time.get_ticks_msec()
	for event in events:
		if not event is Dictionary or event.get('actorId')!='gh_gate_b_brute_01' or event.get('worldEpoch')!=snapshot.worldEpoch: continue
		if not P.counter(event.get('eventId'),1) or not P.counter(event.get('serverTick')) or event.serverTick>snapshot.serverTick or not P.counter(event.get('durationMs'),1,60000): continue
		if not P.number(event.get('_duration'),0.001,60000) or not P.number(event.get('_initial_age'),0,60000) or not P.counter(event.get('_accepted')) or event._accepted>now: continue
		var supported: bool = event.get('kind')=='EnemyAttackTelegraph' and event.get('attackKind') in ['slam','charge'] or event.get('kind')=='EnemyChargeMotion' and event.get('attackKind')=='charge' or event.get('kind')=='EnemyAttackImpact' and event.get('attackKind') in ['slam','charge']
		if not supported: continue
		var age: float = maxf((float(snapshot.serverTick)-float(event.serverTick))*1000.0/60.0,float(event._initial_age)+float(now-int(event._accepted)))
		if age<float(event._duration): selected.append(event)
	return selected

func _grey_hive_body_key(actor: Dictionary) -> String:
	if snapshot.get('worldId')!='grey_hive' or scene.get('worldId')!='grey_hive' or snapshot.get('sceneId')!=scene.get('sceneId') or not P.ident(actor.get('entityId')) or typeof(actor.get('active'))!=TYPE_BOOL or not actor.active or not P.transform(actor.get('transform')):
		return ''
	var key = ''
	var authoring_type = ''
	if actor.get('actorKind')=='enemy' and actor.get('entityType')=='runtime2d.enemy.infected_maintenance_worker.v1':
		key = 'maintenance'
		authoring_type = 'grey_hive.infected_maintenance_worker'
	elif actor.get('actorKind')=='enemy' and actor.get('entityType')=='runtime2d.enemy.infected_security.v1':
		key = 'security'
		authoring_type = 'grey_hive.infected_security'
	elif actor.get('actorKind')=='sentinel' and actor.get('entityType')=='enemy.grey_hive.sentinel' and actor.entityId=='gh_sentinel_arena_sentinel_01' and snapshot.sceneId=='gh_sentinel_arena' and not V.sentinel(snapshot).is_empty():
		key = 'sentinel'
		authoring_type = 'enemy.grey_hive.sentinel'
	else:
		return ''
	var actors = snapshot.actors.filter(func(row): return row.get('entityId')==actor.entityId)
	if actors.size()!=1 or actors[0]!=actor:
		return ''
	var spawns = scene.get('spawns',[])
	if not P.rows(spawns,512):
		return ''
	var matches = spawns.filter(func(row): return row is Dictionary and row.get('id')==actor.entityId)
	if matches.size()!=1 or matches[0].get('kind')!='enemy' or matches[0].get('entityType')!=authoring_type:
		return ''
	return key

func _clear_gh_vital_body_records() -> void:
	gh_vital_body_records.clear()

func _gh_vital_record_rect_valid(body_rect: Rect2, foot: Vector2) -> bool:
	return is_finite(body_rect.position.x) and is_finite(body_rect.position.y) and is_finite(body_rect.size.x) and is_finite(body_rect.size.y) and is_finite(body_rect.end.x) and is_finite(body_rect.end.y) and body_rect.size.x>0.0 and body_rect.size.y>0.0 and is_finite(foot.x) and is_finite(foot.y) and body_rect.position.y<foot.y

func _record_gh_vital_body(actor: Dictionary, selected_key: String, projected_foot: Vector2) -> void:
	var actor_id: Variant = actor.get('entityId')
	if typeof(actor_id)==TYPE_STRING:
		gh_vital_body_records.erase(actor_id)
	if grey_hive_actor==null or camera==null or camera.get_script()!=Camera or not camera.initialized or not is_finite(camera.ppm) or camera.ppm<=0.0:
		return
	if snapshot.get('worldId')!='grey_hive' or scene.get('worldId')!='grey_hive' or snapshot.get('sceneId')!=scene.get('sceneId') or not P.ident(snapshot.get('sceneId')) or not P.counter(snapshot.get('worldEpoch'),1):
		return
	var expected_key: String = 'grey_hive:'+snapshot.get('sceneId')+':'+str(snapshot.get('worldEpoch'))
	if current_key!=expected_key:
		return
	var canonical_key: String = _grey_hive_body_key(actor)
	if canonical_key.is_empty() or canonical_key!=selected_key:
		return
	var actor_transform: Variant = actor.get('transform')
	if not P.transform(actor_transform):
		return
	var actual_foot: Vector2 = camera.project(P.vector(actor_transform.positionM))
	if not is_finite(actual_foot.x) or not is_finite(actual_foot.y) or actual_foot!=projected_foot:
		return
	var body_rect: Rect2 = gh_sentinel_temporal_visual.canvas_rect(actual_foot,camera.ppm) if canonical_key=='sentinel' and sentinel_temporal_drawn else grey_hive_actor.canvas_rect(canonical_key,actual_foot,camera.ppm)
	if not _gh_vital_record_rect_valid(body_rect,actual_foot) or typeof(actor_id)!=TYPE_STRING or not P.ident(actor_id):
		return
	gh_vital_body_records[actor_id] = {'current_key':current_key,'actor_id':actor_id,'selected_key':canonical_key,'projected_foot':actual_foot,'ppm':camera.ppm,'body_rect':body_rect}

func _gh_vital_caption_position(actor: Dictionary, fallback: Vector2) -> Vector2:
	if not is_finite(fallback.x) or not is_finite(fallback.y) or grey_hive_actor==null or camera==null or camera.get_script()!=Camera or not camera.initialized or not is_finite(camera.ppm) or camera.ppm<=0.0:
		return fallback
	if snapshot.get('worldId')!='grey_hive' or scene.get('worldId')!='grey_hive' or snapshot.get('sceneId')!=scene.get('sceneId') or not P.ident(snapshot.get('sceneId')) or not P.counter(snapshot.get('worldEpoch'),1):
		return fallback
	var expected_key: String = 'grey_hive:'+snapshot.get('sceneId')+':'+str(snapshot.get('worldEpoch'))
	if current_key!=expected_key:
		return fallback
	var actor_id: Variant = actor.get('entityId')
	if typeof(actor_id)!=TYPE_STRING or not P.ident(actor_id):
		return fallback
	var canonical_key: String = _grey_hive_body_key(actor)
	if canonical_key.is_empty():
		return fallback
	var record_value: Variant = gh_vital_body_records.get(actor_id)
	if not record_value is Dictionary:
		return fallback
	var record: Dictionary = record_value
	if record.size()!=6 or record.get('current_key')!=current_key or record.get('actor_id')!=actor_id or record.get('selected_key')!=canonical_key:
		return fallback
	var recorded_foot_value: Variant = record.get('projected_foot')
	var recorded_ppm_value: Variant = record.get('ppm')
	var recorded_rect_value: Variant = record.get('body_rect')
	if not recorded_foot_value is Vector2 or (typeof(recorded_ppm_value)!=TYPE_INT and typeof(recorded_ppm_value)!=TYPE_FLOAT) or not recorded_rect_value is Rect2:
		return fallback
	var recorded_foot: Vector2 = recorded_foot_value
	var recorded_ppm: float = float(recorded_ppm_value)
	var recorded_rect: Rect2 = recorded_rect_value
	if not is_finite(recorded_ppm) or recorded_ppm!=camera.ppm or not _gh_vital_record_rect_valid(recorded_rect,recorded_foot):
		return fallback
	var actor_transform: Variant = actor.get('transform')
	if not P.transform(actor_transform):
		return fallback
	var actual_foot: Vector2 = camera.project(P.vector(actor_transform.positionM))
	if not is_finite(actual_foot.x) or not is_finite(actual_foot.y) or recorded_foot!=actual_foot:
		return fallback
	var expected_rect: Rect2 = gh_sentinel_temporal_visual.canvas_rect(actual_foot,camera.ppm) if canonical_key=='sentinel' and sentinel_temporal_drawn else grey_hive_actor.canvas_rect(canonical_key,actual_foot,camera.ppm)
	if not _gh_vital_record_rect_valid(expected_rect,actual_foot) or recorded_rect!=expected_rect:
		return fallback
	if typeof(text_scale)!=TYPE_INT and typeof(text_scale)!=TYPE_FLOAT:
		return fallback
	var safe_text_scale: float = float(text_scale)
	if not is_finite(safe_text_scale) or safe_text_scale<=0.0:
		return fallback
	var fallback_font: Font = ThemeDB.fallback_font
	if fallback_font==null:
		return fallback
	var font_size: int = int(15.0*safe_text_scale)
	if font_size<=0:
		return fallback
	var descent: float = fallback_font.get_descent(font_size)
	var gap: float = 4.0*camera.ppm/48.0
	if not is_finite(descent) or descent<0.0 or not is_finite(gap) or gap<=0.0:
		return fallback
	var baseline_y: float = minf(fallback.y,expected_rect.position.y-descent-gap)
	if not is_finite(baseline_y):
		return fallback
	return Vector2(actual_foot.x,baseline_y)

func _cw_regular_guard_matches(actor:Dictionary) -> bool:
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.get('worldId')!='clockworks' or scene.get('worldId')!='clockworks' or scene.get('sceneId')!=snapshot.get('sceneId') or not CW_REGULAR_GUARDS.has(snapshot.sceneId): return false
	if current_key!='clockworks:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch) or not actor.get('entityId') in CW_REGULAR_GUARDS[snapshot.sceneId] or actor.get('entityType')!='runtime2d.enemy.clockworks.forged_guard.v2' or actor.get('actorKind')!='enemy' or actor.get('active')!=true or not P.transform(actor.get('transform')): return false
	if not P.rows(snapshot.get('actors'),2048) or not P.unique(snapshot.actors,'entityId') or not P.rows(scene.get('spawns'),512) or not P.unique(scene.spawns,'id'): return false
	var live=snapshot.actors.filter(func(row): return row is Dictionary and row.get('entityId')==actor.entityId)
	var authored=scene.spawns.filter(func(row): return row is Dictionary and row.get('id')==actor.entityId)
	return live.size()==1 and live[0]==actor and authored.size()==1 and authored[0].get('kind')=='enemy' and authored[0].get('entityType')=='enemy.clockworks.forged_guard'

func _cw_regular_guard_drop(id:String) -> void:
	if cw_regular_guard_temporal_by_id.has(id):
		cw_regular_guard_temporal_by_id[id].reset()
		cw_regular_guard_temporal_by_id.erase(id)
	cw_regular_guard_sampled.erase(id)

func _cw_regular_guard_reset_all() -> void:
	for id in cw_regular_guard_temporal_by_id.keys(): _cw_regular_guard_drop(id)

func _cw_regular_guard_reconcile(check_visibility:bool=false) -> void:
	var rows=snapshot.get('actors')
	if not rows is Array: _cw_regular_guard_reset_all();return
	for id in cw_regular_guard_temporal_by_id.keys():
		var matches=rows.filter(func(row): return row is Dictionary and row.get('entityId')==id)
		if matches.size()!=1 or not _cw_regular_guard_matches(matches[0]) or (check_visibility and not actor_visible(id)): _cw_regular_guard_drop(id)

func _mh_cw_body_key(actor: Dictionary) -> String:
	if _cw_regular_guard_matches(actor): return 'forged_guard'
	if not P.ident(actor.get('entityId')) or not MH_CW_ROSTER.has(actor.entityId) or actor.get('actorKind')!='enemy' or typeof(actor.get('active'))!=TYPE_BOOL or not actor.active or not P.transform(actor.get('transform')):
		return ''
	var binding: Dictionary = MH_CW_ROSTER[actor.entityId]
	if snapshot.get('worldId')!=binding.world or scene.get('worldId')!=binding.world or snapshot.get('sceneId')!=binding.scene or scene.get('sceneId')!=binding.scene or actor.get('entityType')!=binding.runtime_type:
		return ''
	var actors = snapshot.actors.filter(func(row): return row.get('entityId')==actor.entityId)
	if actors.size()!=1 or actors[0]!=actor:
		return ''
	var spawns = scene.get('spawns',[])
	if not P.rows(spawns,512):
		return ''
	var matches = spawns.filter(func(row): return row is Dictionary and row.get('id')==actor.entityId)
	if matches.size()!=1 or matches[0].get('kind')!='enemy' or matches[0].get('entityType')!=binding.authoring_type:
		return ''
	if binding.key=='signal_wraith':
		var positions = V.signal_positions(actor,snapshot)
		var perception = actor.get('signalPerception')
		if positions.size()!=1 or not perception is Dictionary or typeof(perception.get('precise'))!=TYPE_BOOL or not perception.precise:
			return ''
	return binding.key

# Current source association only; immutable textures never cache world/state authority.
func _gh_state_context(scene_id: String) -> bool:
	return snapshot.get('worldId')=='grey_hive' and scene.get('worldId')=='grey_hive' and snapshot.get('sceneId')==scene_id and scene.get('sceneId')==scene_id
func _gh_state_authored_position(v, expected: Array) -> bool:
	if not v is Array or v.size()!=3:
		return false
	for n in range(3):
		if not P.number(v[n],expected[n],expected[n]):
			return false
	return true
func _gh_visual_row(id: String, asset: String, position: Array) -> Dictionary:
	var sprites = scene.get('presentation',{}).get('sprites')
	if not P.rows(sprites,2048):
		return {}
	var matches = sprites.filter(func(row): return row is Dictionary and row.get('id')==id)
	if matches.size()!=1:
		return {}
	var row: Dictionary = matches[0]
	if row.get('assetId')!=asset or row.get('layer')!='visual.props_dynamic' or not _gh_state_authored_position(row.get('position'),position):
		return {}
	return row
func _gh_console_body(prop: Dictionary) -> Dictionary:
	if not _gh_state_context('gh_power_room') or prop.get('id')!='power_console_off_on':
		return {}
	var art = _gh_visual_row('power_console_off_on','runtime2d.prop.power_console.off_on.v1',[4,0,7])
	if art.is_empty() or prop!=art:
		return {}
	var interactions = scene.get('interactions')
	if not P.rows(interactions,2048):
		return {}
	var canonical = interactions.filter(func(row): return row is Dictionary and row.get('id')=='gh_power_console')
	if canonical.size()!=1 or canonical[0].get('kind')!='power_console' or canonical[0].get('event')!='hive_power':
		return {}
	var current = snapshot.get('interactables')
	if not P.rows(current,2048):
		return {}
	var matches = current.filter(func(row): return row is Dictionary and row.get('entityId')=='gh_power_console')
	if matches.size()!=1:
		return {}
	var entity: Dictionary = matches[0]
	if entity.get('kind')!='power_console' or typeof(entity.get('active'))!=TYPE_BOOL or not P.transform(entity.get('transform')):
		return {}
	var worlds = snapshot.get('progression',{}).get('worlds')
	if not P.rows(worlds,16) or not P.unique(worlds,'worldId'):
		return {}
	var gh = worlds.filter(func(row): return row.get('worldId')=='grey_hive')
	if gh.size()!=1:
		return {}
	var completed = gh[0].get('completedEvents')
	if not P.rows(completed,128):
		return {}
	var seen = {}
	for event in completed:
		if not P.ident(event) or seen.has(event):
			return {}
		seen[event] = true
	return {'id':art.id,'p':P.vector(entity.transform.positionM),'kind':'gh_console_body','data':{'state':'on' if seen.has('hive_power') else 'off'}}
func _cw_arena_known_open_shutter(wall: Dictionary) -> bool:
	# The native render-only projection carries the persistent elite first-kill truth.
	# Missing or invalid projection remains UNKNOWN: preserve the existing fallback.
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.get('worldId')!='clockworks' or snapshot.get('sceneId')!='cw_forged_guard_arena':
		return false
	if scene.get('worldId')!='clockworks' or scene.get('sceneId')!='cw_forged_guard_arena' or scene_hash!='38ae047e46e980f953377121c78713a79b69f84b994e2dd1984e1404fe77ccf5' or current_key!='clockworks:cw_forged_guard_arena:'+str(snapshot.worldEpoch):
		return false
	var expected = {
		'cw_arena_shutter_west_blocker': [[0.5,6],[1,6],[1,9.5],[0.5,9.5]],
		'cw_arena_shutter_east_blocker': [[23,6],[23.5,6],[23.5,9.5],[23,9.5]]
	}
	var id = wall.get('id')
	if typeof(id)!=TYPE_STRING or not expected.has(id) or wall.size()!=3 or wall.get('requiresActorFirstKill')!='cw_forged_guard_elite' or wall.get('polygon')!=expected[id]:
		return false
	var collision = scene.get('collision')
	if not P.rows(collision,2048) or not P.unique(collision,'id'):
		return false
	var matching = collision.filter(func(row): return row.get('id')==id)
	if matching.size()!=1 or matching[0]!=wall:
		return false
	return P.forged_guard_shutters_open(snapshot)==true

func _gh_gate_a_open_barrier(wall: Dictionary) -> bool:
	# Only the canonical closed strip follows the current native Gate A door.
	if not _gh_state_context('gh_gate_a') or not P.counter(scene.get('schemaVersion'),1,1) or not P.counter(snapshot.get('worldEpoch'),1) or current_key!='grey_hive:gh_gate_a:'+str(snapshot.worldEpoch):
		return false
	if wall.size()!=3 or wall.get('id')!='gh_gate_a_closed_barrier' or wall.get('requiresEvent')!='hive_power':
		return false
	var polygon = wall.get('polygon')
	var expected = [[9.5,0.5],[10.5,0.5],[10.5,13.5],[9.5,13.5]]
	if not polygon is Array or polygon.size()!=expected.size():
		return false
	for index in range(expected.size()):
		if not polygon[index] is Array or polygon[index].size()!=2:
			return false
		for axis in range(2):
			if not P.number(polygon[index][axis],expected[index][axis],expected[index][axis]):
				return false
	var collision = scene.get('collision')
	var doors = scene.get('doors')
	var current = snapshot.get('doors')
	if not P.rows(collision,2048) or not P.unique(collision,'id') or not P.rows(doors,2048) or not P.unique(doors,'id') or not P.rows(current,2048) or not P.unique(current,'doorId'):
		return false
	var barriers = collision.filter(func(row): return row.get('id')=='gh_gate_a_closed_barrier')
	var authored = doors.filter(func(row): return row.get('id')=='gh_gate_a')
	var native = current.filter(func(row): return row.get('doorId')=='gh_gate_a')
	if barriers.size()!=1 or barriers[0]!=wall or authored.size()!=1 or native.size()!=1:
		return false
	var door: Dictionary = authored[0]
	if door.size()!=4 or door.get('requiresEvent')!='hive_power' or door.get('assetId')!='runtime2d.prop.gate_a.closed_open.v1' or not _gh_state_authored_position(door.get('position'),[10,0,7]):
		return false
	var state: Dictionary = native[0]
	if typeof(state.get('open'))!=TYPE_BOOL or not state.open or typeof(state.get('locked'))!=TYPE_BOOL or state.locked or not P.transform(state.get('transform')):
		return false
	var position: Dictionary = state.transform.positionM
	return P.number(position.xM,10,10) and P.number(position.yM,0,0) and P.number(position.zM,7,7)

func _gh_gate_b_open_barrier(wall: Dictionary) -> bool:
	# Only the canonical closed strip follows the current native Gate B door.
	if not _gh_state_context('gh_gate_b') or not P.counter(scene.get('schemaVersion'),1,1) or not P.counter(snapshot.get('worldEpoch'),1) or current_key!='grey_hive:gh_gate_b:'+str(snapshot.worldEpoch):
		return false
	if wall.size()!=3 or wall.get('id')!='gh_gate_b_gate_barrier' or wall.get('requiresEvent')!='hive_lockdown':
		return false
	var polygon = wall.get('polygon')
	var expected = [[11.5,0.5],[12.5,0.5],[12.5,15.5],[11.5,15.5]]
	if not polygon is Array or polygon.size()!=expected.size():
		return false
	for index in range(expected.size()):
		if not polygon[index] is Array or polygon[index].size()!=2:
			return false
		for axis in range(2):
			if not P.number(polygon[index][axis],expected[index][axis],expected[index][axis]):
				return false
	var collision = scene.get('collision')
	var doors = scene.get('doors')
	var current = snapshot.get('doors')
	if not P.rows(collision,2048) or not P.unique(collision,'id') or not P.rows(doors,2048) or not P.unique(doors,'id') or not P.rows(current,2048) or not P.unique(current,'doorId'):
		return false
	var barriers = collision.filter(func(row): return row.get('id')=='gh_gate_b_gate_barrier')
	var authored = doors.filter(func(row): return row.get('id')=='gh_gate_b')
	var native = current.filter(func(row): return row.get('doorId')=='gh_gate_b')
	if barriers.size()!=1 or barriers[0]!=wall or authored.size()!=1 or native.size()!=1:
		return false
	var door: Dictionary = authored[0]
	if door.size()!=4 or door.get('requiresEvent')!='hive_lockdown' or door.get('assetId')!='runtime2d.prop.gate_b.v1' or not _gh_state_authored_position(door.get('position'),[12,0,8]):
		return false
	var state: Dictionary = native[0]
	if typeof(state.get('open'))!=TYPE_BOOL or not state.open or typeof(state.get('locked'))!=TYPE_BOOL or state.locked or not P.transform(state.get('transform')):
		return false
	var position: Dictionary = state.transform.positionM
	return P.number(position.xM,12,12) and P.number(position.yM,0,0) and P.number(position.zM,8,8)

func _gh_lockdown_open_seal(wall: Dictionary) -> bool:
	# The canonical seal follows the same current route event as native collision.
	if not _gh_state_context('gh_lockdown') or not P.counter(scene.get('schemaVersion'),1,1) or not P.counter(snapshot.get('worldEpoch'),1) or current_key!='grey_hive:gh_lockdown:'+str(snapshot.worldEpoch):
		return false
	if wall.size()!=3 or wall.get('id')!='gh_lockdown_bio_seal' or wall.get('requiresEvent')!='hive_lockdown':
		return false
	var polygon = wall.get('polygon')
	var expected = [[10,0.5],[11,0.5],[11,15.5],[10,15.5]]
	if not polygon is Array or polygon.size()!=expected.size():
		return false
	for index in range(expected.size()):
		if not polygon[index] is Array or polygon[index].size()!=2:
			return false
		for axis in range(2):
			if not P.number(polygon[index][axis],expected[index][axis],expected[index][axis]):
				return false
	var collision = scene.get('collision')
	var interactions = scene.get('interactions')
	if not P.rows(collision,2048) or not P.unique(collision,'id') or not P.rows(interactions,2048) or not P.unique(interactions,'id'):
		return false
	var seals = collision.filter(func(row): return row.get('id')=='gh_lockdown_bio_seal')
	var terminals = interactions.filter(func(row): return row.get('id')=='gh_lockdown_terminal')
	if seals.size()!=1 or seals[0]!=wall or terminals.size()!=1:
		return false
	var terminal: Dictionary = terminals[0]
	if terminal.size()!=4 or terminal.get('kind')!='lockdown_terminal' or terminal.get('event')!='hive_lockdown' or not _gh_state_authored_position(terminal.get('position'),[4,0,8]):
		return false
	var progression = snapshot.get('progression')
	if not progression is Dictionary or progression.get('currentWorldId')!='grey_hive' or not P.rows(progression.get('worlds'),16) or not P.unique(progression.worlds,'worldId'):
		return false
	var progress = Content.progress(snapshot,'grey_hive')
	return not progress.is_empty() and progress.visitId>0 and 'hive_lockdown' in progress.completedEvents

func _gh_exit_open_barrier(wall: Dictionary) -> bool:
	# The canonical extraction barrier follows the same current route event as native collision.
	if not _gh_state_context('gh_exit') or not P.counter(scene.get('schemaVersion'),1,1) or not P.counter(snapshot.get('worldEpoch'),1) or current_key!='grey_hive:gh_exit:'+str(snapshot.worldEpoch):
		return false
	if wall.size()!=3 or wall.get('id')!='exit_sealed_extraction_barrier' or wall.get('requiresEvent')!='hive_extraction':
		return false
	var polygon = wall.get('polygon')
	var expected = [[21.5,0.5],[22.5,0.5],[22.5,15.5],[21.5,15.5]]
	if not polygon is Array or polygon.size()!=expected.size():
		return false
	for index in range(expected.size()):
		if not polygon[index] is Array or polygon[index].size()!=2:
			return false
		for axis in range(2):
			if not P.number(polygon[index][axis],expected[index][axis],expected[index][axis]):
				return false
	var collision = scene.get('collision')
	var interactions = scene.get('interactions')
	if not P.rows(collision,2048) or not P.unique(collision,'id') or not P.rows(interactions,2048) or not P.unique(interactions,'id'):
		return false
	var seals = collision.filter(func(row): return row.get('id')=='exit_sealed_extraction_barrier')
	var terminals = interactions.filter(func(row): return row.get('id')=='gh_exit_extraction_console')
	if seals.size()!=1 or seals[0]!=wall or terminals.size()!=1:
		return false
	var terminal: Dictionary = terminals[0]
	if terminal.size()!=4 or terminal.get('kind')!='extraction_console' or terminal.get('event')!='hive_extraction' or not _gh_state_authored_position(terminal.get('position'),[8,0,8]):
		return false
	var progression = snapshot.get('progression')
	if not progression is Dictionary or progression.get('currentWorldId')!='grey_hive' or not P.rows(progression.get('worlds'),16) or not P.unique(progression.worlds,'worldId'):
		return false
	var progress = Content.progress(snapshot,'grey_hive')
	return not progress.is_empty() and progress.visitId>0 and 'hive_extraction' in progress.completedEvents

func _gh_door_body(d: Dictionary) -> Dictionary:
	var key = 'gate_a' if d.get('doorId')=='gh_gate_a' else 'gate_b' if d.get('doorId')=='gh_gate_b' else ''
	if key.is_empty() or not _gh_state_context('gh_'+key) or typeof(d.get('open'))!=TYPE_BOOL or typeof(d.get('locked'))!=TYPE_BOOL or not P.transform(d.get('transform')):
		return {}
	var asset = 'runtime2d.prop.gate_a.closed_open.v1' if key=='gate_a' else 'runtime2d.prop.gate_b.v1'
	var visual_id = 'gate_a_closed_open_visual' if key=='gate_a' else 'gate_b_visual_approved'
	var position = [10,0,7] if key=='gate_a' else [12,0,8]
	if _gh_visual_row(visual_id,asset,position).is_empty():
		return {}
	var doors = scene.get('doors')
	var current = snapshot.get('doors')
	if not P.rows(doors,2048) or not P.rows(current,2048):
		return {}
	var canonical = doors.filter(func(row): return row is Dictionary and row.get('id')==d.doorId)
	var matches = current.filter(func(row): return row is Dictionary and row.get('doorId')==d.doorId)
	if canonical.size()!=1 or matches.size()!=1 or matches[0]!=d or canonical[0].get('assetId')!=asset or canonical[0].get('requiresEvent')!=('hive_power' if key=='gate_a' else 'hive_lockdown'):
		return {}
	if key=='gate_b' and d.open:
		return {}
	return {'key':key,'state':'open' if d.open else 'closed'}

func _cw_pressure_valve_selected(prop: Dictionary) -> bool:
	if snapshot.get('worldId')!='clockworks' or snapshot.get('sceneId')!='cw_pressure_hall' or scene.get('worldId')!=snapshot.get('worldId') or scene.get('sceneId')!=snapshot.get('sceneId') or not CW_PRESSURE_VALVES.has(prop.get('id')):
		return false
	if prop.get('assetId')!='runtime2d.world.clockworks.pressure_pipe_valve.v1' or prop.get('layer')!='visual.props_dynamic' or not _gh_authored_position(prop.get('position'),CW_PRESSURE_VALVES[prop.id]):
		return false
	var sprites = scene.get('presentation',{}).get('sprites')
	if not P.rows(sprites,2048):
		return false
	var same_id: Array = sprites.filter(func(row): return row is Dictionary and row.get('id')==prop.id)
	return same_id.size()==1 and same_id[0]==prop

func _cw_furnace_key(prop: Dictionary) -> String:
	if prop.get('id')=='cw_arena_furnace' and not cw_forged_guard_arena_environment_visual.matches_context(snapshot,scene,current_key,scene_hash):
		return ''
	if snapshot.get('worldId')!='clockworks' or scene.get('worldId')!=snapshot.get('worldId') or scene.get('sceneId')!=snapshot.get('sceneId') or not CW_FURNACES.has(prop.get('id')):
		return ''
	var selected: Array = CW_FURNACES[prop.id]
	if snapshot.sceneId!=selected[0] or prop.get('assetId')!=selected[2] or prop.get('layer')!=selected[3] or not _gh_authored_position(prop.get('position'),selected[4]):
		return ''
	var sprites = scene.get('presentation',{}).get('sprites')
	if not P.rows(sprites,2048):
		return ''
	var same_id: Array = sprites.filter(func(row): return row is Dictionary and row.get('id')==prop.id)
	return selected[1] if same_id.size()==1 and same_id[0]==prop else ''
