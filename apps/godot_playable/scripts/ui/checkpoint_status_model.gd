extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const Content = preload('res://scripts/ui/content_model.gd')
const CHECKPOINTS = {
	'return_station:rs_core_room':{'rs_core_checkpoint':''},
	'grey_hive:gh_entry_maintenance':{'gh_entry_checkpoint':''},
	'grey_hive:gh_power_room':{'gh_power_checkpoint':''},
	'grey_hive:gh_gate_a':{'gh_gate_a_checkpoint':''},
	'grey_hive:gh_central_shaft':{'gh_shaft_checkpoint':''},
	'grey_hive:gh_lockdown':{'gh_lockdown_checkpoint':''},
	'grey_hive:gh_deep_decon':{'gh_decon_safety_alcove_checkpoint':''},
	'grey_hive:gh_beacon':{'gh_beacon_checkpoint':''},
	'grey_hive:gh_exit':{'gh_exit_checkpoint':''},
	'clockworks:cw_boiler_chamber':{'cw_boiler_chamber_checkpoint':''},
	'clockworks:cw_conveyor_bridge':{'cw_conveyor_bridge_checkpoint':''},
	'clockworks:cw_entry_foundry':{'cw_entry_foundry_checkpoint':''},
	'clockworks:cw_forged_guard_arena':{'cw_forged_guard_arena_checkpoint':''},
	'clockworks:cw_furnace_heart':{'cw_furnace_heart_checkpoint':''},
	'clockworks:cw_gear_shaft':{'cw_gear_shaft_checkpoint':'','cw_gear_upper_dock_checkpoint':' · 上层停靠点'},
	'clockworks:cw_pressure_hall':{'cw_pressure_hall_checkpoint':''},
	'clockworks:cw_regulator_core':{'cw_regulator_core_checkpoint':''},
	'clockworks:cw_shutdown_exit':{'cw_shutdown_exit_checkpoint':''},
	'mist_harbor:mh_breakwater':{'mh_breakwater_checkpoint':''},
	'mist_harbor:mh_drowned_quay':{'mh_drowned_quay_checkpoint':''},
	'mist_harbor:mh_extraction':{'mh_extraction_checkpoint':''},
	'mist_harbor:mh_fog_pier':{'mh_fog_pier_checkpoint':''},
	'mist_harbor:mh_pump_station':{'mh_pump_station_checkpoint':''},
	'mist_harbor:mh_resonance_tower':{'mh_resonance_tower_checkpoint':''},
	'mist_harbor:mh_signal_yard':{'mh_signal_yard_checkpoint':''},
	'mist_harbor:mh_tidal_warehouse':{'mh_warehouse_checkpoint':''},
	'mist_harbor:mh_warden_arena':{'mh_warden_arena_checkpoint':''}
}
static func project(s, eligible) -> String:
	if typeof(eligible)!=TYPE_BOOL or not eligible or not P.snapshot(s) or s.get('entryToken')!=null:
		return ''
	var context = s.worldId+':'+s.sceneId
	if not CHECKPOINTS.has(context) or not Content.SCENE_NAMES.has(s.sceneId):
		return ''
	var id = s.checkpointId
	if id==null:
		return '本场景检查点：未设置'
	if typeof(id)==TYPE_STRING and CHECKPOINTS[context].has(id):
		return '当前检查点：'+Content.SCENE_NAMES[s.sceneId]+CHECKPOINTS[context][id]
	return '本场景检查点：状态未确认'
