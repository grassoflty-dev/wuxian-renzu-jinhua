extends Node2D
const P = preload('res://scripts/bridge/protocol.gd')
const V = preload('res://scripts/world/presentation.gd')
const Camera = preload('res://scripts/world/projection.gd')
const CenYaoIdle = preload('res://scripts/world/cenyao_idle_visual.gd')
const MhCwActor = preload('res://scripts/world/mh_cw_actor_visual.gd')
const MH_CW_ROSTER = {"cw_entry_foundry_forged_guard_01":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_entry_foundry","authoring_type":"enemy.clockworks.forged_guard"},"mh_breakwater_signal_wraith_02":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_breakwater","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_tidal_warehouse_drowned_01":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_tidal_warehouse","authoring_type":"enemy.mist_harbor.drowned"},"mh_signal_yard_signal_wraith_03":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_signal_yard","authoring_type":"enemy.mist_harbor.signal_wraith"},"cw_pressure_hall_forged_guard_01":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_pressure_hall","authoring_type":"enemy.clockworks.forged_guard"},"mh_signal_yard_signal_wraith_02":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_signal_yard","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_resonance_tower_signal_wraith_01":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_resonance_tower","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_tidal_warehouse_drowned_02":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_tidal_warehouse","authoring_type":"enemy.mist_harbor.drowned"},"mh_drowned_quay_drowned_03":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_drowned_quay","authoring_type":"enemy.mist_harbor.drowned"},"mh_drowned_quay_drowned_02":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_drowned_quay","authoring_type":"enemy.mist_harbor.drowned"},"mh_drowned_quay_drowned_01":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_drowned_quay","authoring_type":"enemy.mist_harbor.drowned"},"mh_fog_pier_drowned_02":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_fog_pier","authoring_type":"enemy.mist_harbor.drowned"},"mh_drowned_quay_drowned_04":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_drowned_quay","authoring_type":"enemy.mist_harbor.drowned"},"mh_signal_yard_signal_wraith_01":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_signal_yard","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_pump_station_drowned_03":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_pump_station","authoring_type":"enemy.mist_harbor.drowned"},"mh_tidal_warehouse_signal_wraith_01":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_tidal_warehouse","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_breakwater_signal_wraith_01":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_breakwater","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_fog_pier_drowned_01":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_fog_pier","authoring_type":"enemy.mist_harbor.drowned"},"cw_furnace_heart_forged_guard_01":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_furnace_heart","authoring_type":"enemy.clockworks.forged_guard"},"cw_conveyor_bridge_forged_guard_01":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_conveyor_bridge","authoring_type":"enemy.clockworks.forged_guard"},"mh_resonance_tower_signal_wraith_02":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_resonance_tower","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_pump_station_drowned_02":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_pump_station","authoring_type":"enemy.mist_harbor.drowned"},"cw_pressure_hall_forged_guard_02":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_pressure_hall","authoring_type":"enemy.clockworks.forged_guard"},"cw_furnace_heart_forged_guard_02":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_furnace_heart","authoring_type":"enemy.clockworks.forged_guard"},"cw_gear_shaft_forged_guard_02":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_gear_shaft","authoring_type":"enemy.clockworks.forged_guard"},"mh_pump_station_drowned_01":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_pump_station","authoring_type":"enemy.mist_harbor.drowned"},"cw_gear_shaft_forged_guard_01":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_gear_shaft","authoring_type":"enemy.clockworks.forged_guard"},"mh_resonance_tower_signal_wraith_03":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_resonance_tower","authoring_type":"enemy.mist_harbor.signal_wraith"},"cw_entry_foundry_forged_guard_02":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_entry_foundry","authoring_type":"enemy.clockworks.forged_guard"},"cw_conveyor_bridge_forged_guard_02":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_conveyor_bridge","authoring_type":"enemy.clockworks.forged_guard"},"mh_resonance_tower_signal_wraith_04":{"runtime_type":"enemy.mist_harbor.signal_wraith","world":"mist_harbor","key":"signal_wraith","scene":"mh_resonance_tower","authoring_type":"enemy.mist_harbor.signal_wraith"},"mh_tidal_warehouse_drowned_03":{"runtime_type":"runtime2d.enemy.mistharbor.drowned.v2","world":"mist_harbor","key":"drowned","scene":"mh_tidal_warehouse","authoring_type":"enemy.mist_harbor.drowned"},"cw_conveyor_bridge_forged_guard_03":{"runtime_type":"runtime2d.enemy.clockworks.forged_guard.v2","world":"clockworks","key":"forged_guard","scene":"cw_conveyor_bridge","authoring_type":"enemy.clockworks.forged_guard"}}
const GreyHiveActor = preload('res://scripts/world/grey_hive_actor_visual.gd')
const RegisteredProps = preload('res://scripts/world/registered_props_visual.gd')
const GreyHiveStateProps = preload('res://scripts/world/grey_hive_state_props_visual.gd')
const GhBioLockdownVisual = preload('res://scripts/world/gh_bio_lockdown_visual.gd')
const GhSupplyCrateVisual = preload('res://scripts/world/gh_supply_crate_visual.gd')
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
	'bio_pod_approved':{'key':'bio_pod','scene':'gh_bio_isolation','asset':'runtime2d.prop.bio_pod.v1','position':Vector3(8,0,7)},
	'medical_station_approved':{'key':'medical_station','scene':'gh_bio_isolation','asset':'runtime2d.prop.medical_station.v1','position':Vector3(16,0,5)},
	'lockdown_terminal_visual':{'key':'lockdown_terminal','scene':'gh_lockdown','asset':'runtime2d.prop.lockdown_terminal.v1','position':Vector3(4,0,8)}
}
var gh_bio_lockdown_visual
const CwPressureValveVisual = preload('res://scripts/world/cw_pressure_valve_visual.gd')
const CW_PRESSURE_VALVES = {
	'cw_pressure_valve_visual_01':Vector3(8,0,7),
	'cw_pressure_valve_visual_02':Vector3(12,0,7),
	'cw_pressure_valve_visual_03':Vector3(16,0,7)
}
var cw_pressure_valve_visual
const CwFurnaceVisual = preload('res://scripts/world/cw_furnace_visual.gd')
const CW_FURNACES = {
	'cw_entry_furnace':['cw_entry_foundry','furnace_v2','runtime2d.world.clockworks.furnace.v2','visual.props_back',Vector3(19,0,6)],
	'cw_boiler_furnace':['cw_boiler_chamber','furnace_v2','runtime2d.world.clockworks.furnace.v2','visual.props_back',Vector3(16,0,6)],
	'cw_boiler_furnace_variant':['cw_boiler_chamber','furnace_v1','runtime2d.world.clockworks.furnace.v1','visual.props_dynamic',Vector3(18,0,10)],
	'cw_heart_furnace_v1':['cw_furnace_heart','furnace_v1','runtime2d.world.clockworks.furnace.v1','visual.props_back',Vector3(12,0,5)],
	'cw_heart_furnace_v2':['cw_furnace_heart','furnace_v2','runtime2d.world.clockworks.furnace.v2','visual.props_dynamic',Vector3(16,0,8)]
}
var cw_furnace_visual
var camera = Camera.new()
var cenyao_idle = CenYaoIdle.new()
var registered_props = RegisteredProps.new()
var mh_cw_actor
var grey_hive_actor
var grey_hive_state_props
var prop_bodies_drawn: Dictionary = {}
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
var displayed = {}
var occluder_alpha = {}
var viewport_rect = Rect2()
var safe_rect_provider: Callable
var interaction_focus: Dictionary = {}
func _ready() -> void:
	baizhi_placeholder_visual = BaizhiPlaceholderVisual.new()
	cw_furnace_visual = CwFurnaceVisual.new()
	cw_pressure_valve_visual = CwPressureValveVisual.new()
	gh_supply_crate_visual = GhSupplyCrateVisual.new()
	grey_hive_state_props = GreyHiveStateProps.new()
	gh_bio_lockdown_visual = GhBioLockdownVisual.new()
	mh_cw_actor = MhCwActor.new()
	grey_hive_actor = GreyHiveActor.new()
	texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
	texture_repeat = CanvasItem.TEXTURE_REPEAT_DISABLED
func commit(s: Dictionary, data: Dictionary, hash: String) -> void:
	clear_interaction_focus()
	var key = s.worldId+':'+s.sceneId+':'+str(s.worldEpoch)
	if key!=current_key:
		camera.reset()
		events.clear()
		sound.clear()
		displayed.clear()
		occluder_alpha.clear()
		current_key = key
	snapshot = s
	scene = data
	scene_hash = hash
	last_snapshot_at = Time.get_ticks_msec()
	var ap = s.player.get('actionPresentation')
	if ap is Dictionary:
		var akey = str([s.serverTick,ap.get('requestId'),ap.get('elapsedMs'),ap.get('phase')])
		if akey!=action_key:
			action_key = akey
			action_at = last_snapshot_at
	queue_redraw()
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
	if snapshot.is_empty() or scene.is_empty():
		return
	var full = get_viewport_rect()
	var safe_rect = full
	if snapshot.worldId=='return_station' and snapshot.sceneId=='rs_core_room' and safe_rect_provider.is_valid():
		var candidate = safe_rect_provider.call()
		if candidate is Rect2:
			if is_finite(candidate.position.x) and is_finite(candidate.position.y) and is_finite(candidate.size.x) and is_finite(candidate.size.y) and candidate.size.x>0 and candidate.size.y>0:
				var clipped = candidate.intersection(full)
				if clipped.size.x>0 and clipped.size.y>0:
					safe_rect = clipped
	camera.update(snapshot,scene,full.size,maxf(0,delta),safe_rect)
	viewport_rect = full
	_update_occlusion(delta)
	queue_redraw()
func visible_point(p: Vector3) -> bool:
	return viewport_rect.grow(4).has_point(camera.project(p))
func actor_visible(id: String, allow_dead: bool = false) -> bool:
	if id=='player' or id==snapshot.get('player',{}).get('entityId'):
		return not snapshot.is_empty() and visible_point(P.vector(snapshot.player.transform.positionM))
	for a in snapshot.get('actors',[]):
		if a.entityId==id and (a.active or allow_dead):
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
func _draw() -> void:
	prop_bodies_drawn.clear()
	draw_rect(get_viewport_rect(),Color('#0b121a'))
	if snapshot.is_empty() or scene.is_empty():
		return
	displayed.clear()
	var b: Dictionary = scene.boundsM
	var floor_poly = [[b.x,b.z],[b.x+b.width,b.z],[b.x+b.width,b.z+b.depth],[b.x,b.z+b.depth]]
	var floor_color = {'return_station':Color('#233c47'),'grey_hive':Color('#263634'),'mist_harbor':Color('#203742'),'clockworks':Color('#40362c')}.get(snapshot.worldId,Color('#263634'))
	_poly(floor_poly,0,floor_color,true)
	# Canonical static geometry is display only. No spawns, routes, navigation or hidden objectives.
	for wall in scene.get('collision',[]):
		if wall is Dictionary and P.polygon(wall.get('polygon')):
			_poly(wall.polygon,0,Color('#4b5659'),true)
			_poly(wall.polygon,0,Color('#819092'),false)
	for region in scene.get('terrainRegions',[]):
		if region is Dictionary and P.polygon(region.get('polygon')):
			var tag = region.get('tag','')
			var pump = snapshot.get('mistHarborPump')
			var drained = pump is Dictionary and pump.get('state')=='drained' and region.get('id') in ['mh_pump_east_channel_water','mh_water_depth_region']
			var c = Color(0.16,0.4,0.5,0.16 if drained else 0.32) if 'water' in tag else Color(0.36,0.29,0.2,0.24)
			_poly(region.polygon,0,c,true)
	_supports()
	_surfaces()
	var objects: Array = []
	for prop in scene.get('presentation',{}).get('sprites',[]):
		if not prop is Dictionary or not prop.get('position') is Array or prop.position.size()!=3 or typeof(prop.get('assetId'))!=TYPE_STRING:
			continue
		var console_body = _gh_console_body(prop)
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
	objects.sort_custom(func(a,b):
		var ay = camera.project(Vector3(a.p.x,0,a.p.z)).y
		var by = camera.project(Vector3(b.p.x,0,b.p.z)).y
		return a.id<b.id if is_equal_approx(ay,by) else ay<by)
	var interaction_captions = _interaction_captions()
	for o in objects:
		_draw_object(o,interaction_captions.get(o.id,''))
	_hazards()
	_beacon()
	_boss_warning()
	_action()
	_event_draw()
	_occlusion()
	_atmosphere()
	_world_ui()
	_draw_interaction_focus()
	_rear_view()
	if reported_key!=current_key:
		reported_key = current_key
		call_deferred('_report_frame',current_key)
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
	var font = ThemeDB.fallback_font
	var size = int(15*text_scale)
	var width = font.get_string_size(text,HORIZONTAL_ALIGNMENT_LEFT,-1,size).x
	var bounds = Rect2(p-Vector2(width*0.5,font.get_ascent(size)),Vector2(width,font.get_height(size)))
	if viewport_rect.encloses(bounds):
		draw_string(font,p-Vector2(width*0.5,0),text,HORIZONTAL_ALIGNMENT_LEFT,-1,size,c)
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
			var caption: String = labels.get(row.kind,'')
			if rs_scope and rs_roles.has(row.entityId):
				var role: Dictionary = rs_roles[row.entityId]
				if row.kind==role.native_kind and P.vector(row.transform.positionM)==role.position and rs_authored_captions.has(row.entityId):
					caption = rs_authored_captions[row.entityId]
			if not caption.is_empty():
				captions[row.entityId] = caption
	return captions
func _draw_object(o: Dictionary, interaction_caption: String = '') -> void:
	var at = camera.project(o.p)
	if not viewport_rect.grow(100).has_point(at):
		return
	var scale = camera.ppm/48.0
	var c = Color('#779ba2')
	match o.kind:
		'gh_story_prop':
			var body_drawn = false
			if o.data.image_allowed and gh_bio_lockdown_visual!=null:
				body_drawn = gh_bio_lockdown_visual.draw_body(self,o.data.key,at,camera.ppm)
			if not body_drawn:
				draw_rect(Rect2(at-Vector2(16,24)*scale,Vector2(32,24)*scale),Color('#65716d'))
		'registered_terminal':
			if not registered_props.draw_body(self,'rs_terminal',at,camera.ppm):
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
			if _gh_crate_selected(o.data) and gh_supply_crate_visual!=null and gh_supply_crate_visual.draw_body(self,at,camera.ppm):
				return
			var furnace_key = _cw_furnace_key(o.data)
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
			draw_rect(Rect2(at-Vector2(16,24)*scale,Vector2(32,24)*scale),Color('#65716d'))
		'player':
			c = Color('#85dfd0')
			draw_circle(at,12*scale,c)
			var body_drawn = false
			if o.id==snapshot.player.entityId and o.data==snapshot.player and snapshot.player.currentHp>0:
				body_drawn = cenyao_idle.draw_body(self,at,o.data,camera.ppm)
			if not body_drawn:
				draw_line(at,at-Vector2(0,34)*scale,c,9*scale,true)
			var f = Vector3(float(o.data.facingX),0,float(o.data.facingZ))
			draw_line(at,camera.project(o.p+f*0.7),Color.WHITE,2,true)
			displayed[o.id] = true
		'actor':
			var a: Dictionary = o.data
			c = Color('#df9c79')
			var type: String = a.entityType
			if type=='enemy.mist_harbor.signal_wraith':
				c = Color('#b9a9de')
				var precise = a.get('signalPerception',{}).get('precise',false)
				if not precise:
					_ring(o.p,float(a.signalPerception.uncertaintyRadiusM),Color(0.6,0.5,0.8,0.5))
					_label(at-Vector2(0,30*scale),'不确定信号',c)
					return
			if 'pressure_drone' in type:
				draw_line(at+Vector2(-16,-22)*scale,at+Vector2(16,-22)*scale,c,4,true)
				draw_circle(at-Vector2(0,22)*scale,9*scale,c)
			elif 'hound' in type:
				draw_rect(Rect2(at-Vector2(25,18)*scale,Vector2(50,18)*scale),c)
			elif 'swarm' in type:
				draw_circle(at,5*scale,c)
			else:
				var body_key = _grey_hive_body_key(a)
				var body_drawn = false
				if grey_hive_actor!=null and not body_key.is_empty() and actor_visible(a.entityId):
					body_drawn = grey_hive_actor.draw_body(self,body_key,at,camera.ppm)
				if not body_drawn and mh_cw_actor!=null and actor_visible(a.entityId):
					var mh_cw_key = _mh_cw_body_key(a)
					if not mh_cw_key.is_empty():
						body_drawn = mh_cw_actor.draw_body(self,mh_cw_key,at,camera.ppm)
				if not body_drawn:
					var boss = 'sentinel' in type or 'forged_guard' in type
					draw_rect(Rect2(at-Vector2(13,44 if boss else 30)*scale,Vector2(26,44 if boss else 30)*scale),c)
			if color_assistance:
				draw_line(at-Vector2(7,7),at+Vector2(7,7),Color.WHITE,2)
				draw_line(at-Vector2(-7,7),at+Vector2(-7,7),Color.WHITE,2)
			if a.entityId in ['cw_prime_regulator','cw_forged_guard_elite']:
				_label(at-Vector2(0,65*text_scale),'主调节者' if a.entityId=='cw_prime_regulator' else '铸造卫士')
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
			draw_rect(Rect2(at-Vector2(8,16)*scale,Vector2(16,16)*scale),Color('#72b9cb'))
			if not interaction_caption.is_empty():
				_label(at-Vector2(0,20)*scale,interaction_caption)
		'npc':
			if not is_finite(camera.ppm) or camera.ppm<=0 or not is_finite(at.x) or not is_finite(at.y) or not is_finite(1.7*camera.ppm+5*scale):
				return
			if baizhi_placeholder_visual==null or not baizhi_placeholder_visual.draw_body(self,o.data,camera):
				draw_circle(at-Vector2(0,23)*scale,10*scale,Color('#deded2'))
				draw_line(at,at-Vector2(0,23)*scale,Color('#deded2'),8*scale)
			_label(at-Vector2(0,1.7*camera.ppm+5*scale),'白芷')
func _supports() -> void:
	var poses = V.support(snapshot.get('supportScene'),snapshot,scene_hash)
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
func _boss_warning() -> void:
	var sentinel = V.sentinel(snapshot)
	if not sentinel.is_empty() and sentinel.get('warning') is Dictionary:
		var w: Dictionary = sentinel.warning
		if w.shape=='circle':
			_ring(P.vector(w.originM),w.radiusM,Color('#f0ad6a'))
		else:
			_corridor(P.vector(w.originM),w.directionRad,w.rangeM,w.radiusM,Color('#f0ad6a'))
		_label(camera.project(P.vector(sentinel.positionM))-Vector2(0,65*text_scale),'哨卫 · '+sentinel.phase+' '+str(sentinel.remainingMs)+'ms')
	var boss = snapshot.get('bossEncounter')
	if snapshot.worldId!='mist_harbor' or snapshot.sceneId!='mh_warden_arena' or not boss is Dictionary or boss.get('entityId')!='mh_resonance_warden_staged_marker' or boss.get('entityType')!='enemy.mist_harbor.resonance_warden' or not actor_visible(boss.entityId) or not P.counter(boss.get('currentHp'),1,100000) or not P.counter(boss.get('maxHp'),1,100000) or boss.currentHp>boss.maxHp or not P.counter(boss.get('phase'),1,2) or not boss.get('state') in ['idle','chase','decoy','windup','recovery','stagger'] or boss.get('temporaryVisual')!=true or boss.get('publicReleaseEligible')!=false or typeof(boss.get('mappedTrueSource'))!=TYPE_BOOL:
		return
	var actors = snapshot.actors.filter(func(a): return a.entityId==boss.entityId)
	if actors.size()!=1 or actors[0].entityType!='runtime2d.enemy.mistharbor.signal_wraith.v2' or not actors[0].active:
		return
	_label(camera.project(P.vector(actors[0].transform.positionM))-Vector2(0,65*text_scale),'共鸣守望者 · 阶段 '+str(int(boss.phase)))
	var w = boss.get('warning')
	if not w is Dictionary or typeof(w.get('ordinaryVisible'))!=TYPE_BOOL or not P.counter(w.get('remainingMs'),1,60000) or not P.counter(w.get('attackSerial'),1) or not w.get('originM') is Array or w.originM.size()!=3 or not P.number(w.originM[0],0.5,23.5) or w.originM[1]!=0 or not P.number(w.originM[2],0.5,15.5) or not P.number(w.get('radiusM'),0.001,12) or not P.number(w.get('halfAngleRad'),0.001,PI+0.000001) or not P.number(w.get('directionRad'),-PI-0.000001,PI+0.000001) or not w.get('kind') in ['strike','pulse','decoy']:
		return
	if (w.kind=='decoy')!=(boss.state=='decoy') or w.kind!='decoy' and boss.state!='windup' or boss.mappedTrueSource and (w.kind=='decoy' or not V.mapping(snapshot)):
		return
	if w.ordinaryVisible or boss.mappedTrueSource and V.mapping(snapshot):
		var center = Vector3(w.originM[0],0,w.originM[2])
		_ring(center,w.radiusM,Color('#f0ad6a'),w.directionRad-w.halfAngleRad if w.kind=='strike' else 0,w.halfAngleRad*2 if w.kind=='strike' else TAU)
		_label(camera.project(center)-Vector2(0,50),'回声' if w.kind=='decoy' else '共鸣预警 · 移出范围')
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
	match p.actionState:
		'primaryAttack': _ring(pos,ap.rangeM,c,yaw-PI/3,PI*2/3)
		'guard':
			var at = camera.project(pos)+Vector2(0,-28)
			var shield = PackedVector2Array([at+Vector2(-10,-10),at+Vector2(10,-10),at+Vector2(8,4),at+Vector2(0,12),at+Vector2(-8,4),at+Vector2(-10,-10)])
			draw_polyline(shield,c,2.5,true)
		'pierce': _corridor(pos,yaw,ap.rangeM,ap.lineHalfWidthM,c)
		'pulse': _ring(pos,ap.rangeM*(0.78 if ap.phase=='windup' and not reduced_motion else 1.0),c)
		'dash': draw_line(camera.project(pos),camera.project(pos-Vector3(dir.x,0,dir.y)*0.7),c,4,true)
	_label(camera.project(pos)-Vector2(0,52*text_scale),{'windup':'准备','active':'执行','recovery':'恢复'}[ap.phase])
func _event_draw() -> void:
	for e in events:
		var pos = P.vector(e.positionM)
		var c = Color('#f1ca8c')
		if e.get('combatFeedback') is Dictionary:
			var outcome: String = e.combatFeedback.outcome
			var at = camera.project(pos)
			if outcome=='rejected':
				_label(at-Vector2(0,30),{'cooldown':'技能仍在冷却','insufficient_energy':'能量不足','action_busy':'当前动作尚未结束','guard_not_active':'防御已结束','dash_rejected':'当前无法冲刺','no_target':'未命中目标'}.get(e.combatFeedback.get('reason'),'当前无法执行该动作'))
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
						_label(camera.project(P.vector(a.transform.positionM))-Vector2(0,55*text_scale),{'healthy':'健康','wounded':'受伤','severelyWounded':'重伤','critical':'濒危'}[v.tier])
	if not sound.is_empty() and V.mapping(snapshot):
		var dir = Vector2(sin(sound.directionRad)-cos(sound.directionRad),sin(sound.directionRad)+cos(sound.directionRad)).normalized()
		var at = camera.project(P.vector(snapshot.player.transform.positionM))+dir*70
		draw_line(at-dir*10,at+dir*10,Color('#bcd9e2'),3)
		var distance = '近处' if sound.distanceM<2 else '约 5 米' if sound.distanceM<7.5 else '约 '+str(int(round(sound.distanceM/10)*10))+' 米'
		_label(at+Vector2(0,-15),('真实声源' if sound.kind=='warden_true_call' else '信号')+' · '+distance)
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
func _atmosphere() -> void:
	if snapshot.worldId=='mist_harbor':
		for prop in scene.get('presentation',{}).get('sprites',[]):
			if prop is Dictionary and prop.get('assetId') in ['runtime2d.world.mistharbor.fog_bank.v1','runtime2d.world.mistharbor.fog_vfx.v1'] and prop.get('position') is Array and prop.position.size()==3:
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
	var foot: Vector3 = selected.position
	var image_allowed = true
	if selected.key=='lockdown_terminal':
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
func _mh_cw_body_key(actor: Dictionary) -> String:
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
