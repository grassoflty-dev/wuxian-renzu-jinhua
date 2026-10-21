extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const SCENE_SHA256 = '65e0cbf11183fe114259d680b0cd84a23a7f8619a25e628f4b30a7a84ce870a8'
const HEIGHT = 0.25
const SIDE_COLOR: Color = Color('#384246')
const CAP_COLOR: Color = Color('#4b5659')
const CONTACT_COLOR: Color = Color(17.0/255.0,26.0/255.0,29.0/255.0,0.14)
const CANONICAL_SCENE: Dictionary = {"boundsM":{"depth":16.0,"width":24.0,"x":0,"z":0},"cameraZones":[],"checkpoints":[{"id":"gh_decon_safety_alcove_checkpoint","navNode":"gh_decon_safe_nav","position":[3.0,0.0,12.0]}],"collision":[{"id":"deep_decon_wall_north","polygon":[[0.0,0.0],[24.0,0.0],[24.0,0.5],[0.0,0.5]]},{"id":"deep_decon_wall_south","polygon":[[0.0,15.5],[24.0,15.5],[24.0,16.0],[0.0,16.0]]},{"id":"deep_decon_wall_west_north","polygon":[[0.0,0.5],[0.5,0.5],[0.5,6.5],[0.0,6.5]]},{"id":"deep_decon_wall_west_south","polygon":[[0.0,9.5],[0.5,9.5],[0.5,15.5],[0.0,15.5]]},{"id":"deep_decon_wall_east_north","polygon":[[23.5,0.5],[24.0,0.5],[24.0,6.5],[23.5,6.5]]},{"id":"deep_decon_wall_east_south","polygon":[[23.5,9.5],[24.0,9.5],[24.0,15.5],[23.5,15.5]]},{"id":"deep_decon_safety_alcove_west","polygon":[[1.0,10.5],[1.5,10.5],[1.5,13.5],[1.0,13.5]]},{"id":"deep_decon_safety_alcove_south","polygon":[[1.0,13.0],[4.0,13.0],[4.0,13.5],[1.0,13.5]]}],"doors":[],"hazards":[{"environment":{"activeMs":700,"damage":8,"damageIntervalMs":700,"recoveryMs":2400,"tag":"pressure","warningMs":900},"id":"gh_decon_steam_jet_zone","kind":"steam_jet","polygon":[[7.0,4.0],[10.0,4.0],[10.0,6.5],[7.0,6.5]]},{"environment":{"activeMs":4000,"damage":6,"damageIntervalMs":1000,"recoveryMs":1500,"tag":"toxin","warningMs":1200},"id":"gh_decon_mist_zone","kind":"decon_mist","polygon":[[14.0,9.0],[18.0,9.0],[18.0,14.0],[14.0,14.0]]}],"interactions":[{"cooldownMs":8000,"environmentControl":{"exposureReductionUnits":0,"suppressionMs":6000,"targetHazardIds":["gh_decon_steam_jet_zone"]},"event":null,"id":"gh_decon_valve_01_static_marker","kind":"environment_control","position":[7.0,0.0,3.0],"rangeM":2.5},{"cooldownMs":8000,"environmentControl":{"exposureReductionUnits":0,"suppressionMs":6000,"targetHazardIds":["gh_decon_mist_zone"]},"event":null,"id":"gh_decon_valve_02_static_marker","kind":"environment_control","position":[17.0,0.0,13.0],"rangeM":2.5}],"logic":{"traversal":[]},"navigation":{"links":[{"from":"gh_decon_spawn_nav","to":"gh_decon_safe_nav"},{"from":"gh_decon_safe_nav","to":"gh_decon_valve_01_nav"},{"from":"gh_decon_valve_01_nav","to":"gh_decon_valve_02_nav"},{"from":"gh_decon_valve_02_nav","to":"gh_decon_exit_nav"}],"nodes":[{"id":"gh_decon_spawn_nav","position":[2.0,0.0,8.0]},{"id":"gh_decon_safe_nav","position":[3.0,0.0,12.0]},{"id":"gh_decon_valve_01_nav","position":[7.0,0.0,3.0]},{"id":"gh_decon_valve_02_nav","position":[17.0,0.0,13.0]},{"id":"gh_decon_exit_nav","position":[22.0,0.0,8.0]}]},"objectives":[],"occluders":[],"presentation":{"cameraProfile":"oblique_default","layers":[{"id":"visual.floor","zGroup":0},{"id":"visual.props_back","zGroup":10},{"id":"visual.props_dynamic","zGroup":20}],"sprites":[{"assetId":"runtime2d.grey_hive.floor_tiles.v1","id":"deep_decon_approved_floor","layer":"visual.floor","position":[12.0,0.0,8.0]},{"assetId":"runtime2d.grey_hive.wall_tiles.v1","id":"deep_decon_wall_tiles","layer":"visual.props_back","position":[12.0,0.0,1.0]},{"assetId":"runtime2d.grey_hive.pipes_cables.v1","id":"deep_decon_service_pipes","layer":"visual.props_back","position":[10.0,0.0,3.0]},{"assetId":"runtime2d.prop.supply_crates.v1","id":"deep_decon_safety_corner_crates","layer":"visual.props_back","position":[3.0,0.0,12.5]},{"assetId":"runtime2d.grey_hive.clutter.v1","id":"deep_decon_drainage_clutter","layer":"visual.props_back","position":[18.0,0.0,12.5]},{"assetId":"runtime2d.grey_hive.pipes_cables.v1","id":"deep_decon_valve_01_pipe_anchor","layer":"visual.props_back","position":[7.0,0.0,3.0]},{"assetId":"runtime2d.grey_hive.pipes_cables.v1","id":"deep_decon_valve_02_pipe_anchor","layer":"visual.props_back","position":[17.0,0.0,13.0]},{"assetId":"runtime2d.enemy.infected_maintenance_worker.v1","id":"deep_decon_worker_01_visual","layer":"visual.props_dynamic","position":[9.0,0.0,5.0]},{"assetId":"runtime2d.enemy.infected_maintenance_worker.v1","id":"deep_decon_worker_02_visual","layer":"visual.props_dynamic","position":[17.0,0.0,10.5]}]},"sceneId":"gh_deep_decon","schemaVersion":1,"spawns":[{"entityType":null,"id":"gh_deep_decon_spawn","kind":"player","navNode":"gh_decon_spawn_nav","position":[2.0,0.0,8.0]},{"entityType":"grey_hive.infected_maintenance_worker","id":"gh_decon_worker_01","kind":"enemy","navNode":"gh_decon_valve_01_nav","position":[9.0,0.0,5.0]},{"entityType":"grey_hive.infected_maintenance_worker","id":"gh_decon_worker_02","kind":"enemy","navNode":"gh_decon_valve_02_nav","position":[17.0,0.0,10.5]},{"entityType":"enemy.grey_hive.swarm","id":"gh_deep_decon_swarm_01","kind":"enemy","navNode":"gh_decon_valve_01_nav","position":[11.5,0.0,8.0]},{"entityType":"enemy.grey_hive.swarm","id":"gh_deep_decon_swarm_02","kind":"enemy","navNode":"gh_decon_exit_nav","position":[20.0,0.0,5.0]}],"transitions":[{"id":"gh_deep_decon_to_sentinel_arena","polygon":[[22.0,7.0],[24.0,7.0],[24.0,9.0],[22.0,9.0]],"spawnId":"gh_sentinel_arena_spawn","toSceneId":"gh_sentinel_arena"}],"triggers":[],"vfxMarkers":[],"worldId":"grey_hive"}
const WALL_PLAN: Array = [{"id":"deep_decon_wall_north","collision_index":0,"sides":[{"a":[0.5,0.5],"b":[23.5,0.5],"clip":true},{"a":[24.0,0.0],"b":[24.0,0.5],"clip":false}],"contact":[0.5,0.5,23.25,0.53]},{"id":"deep_decon_wall_south","collision_index":1,"sides":[{"a":[0.0,16.0],"b":[24.0,16.0],"clip":false},{"a":[24.0,15.5],"b":[24.0,16.0],"clip":false}],"contact":[]},{"id":"deep_decon_wall_west_north","collision_index":2,"sides":[{"a":[0.5,0.5],"b":[0.5,6.5],"clip":false},{"a":[0.0,6.5],"b":[0.5,6.5],"clip":false}],"contact":[0.5,0.53,0.53,6.5]},{"id":"deep_decon_wall_west_south","collision_index":3,"sides":[{"a":[0.5,9.5],"b":[0.5,15.5],"clip":true}],"contact":[0.5,9.5,0.53,15.25]},{"id":"deep_decon_wall_east_north","collision_index":4,"sides":[{"a":[24.0,0.5],"b":[24.0,6.5],"clip":false},{"a":[23.5,6.5],"b":[24.0,6.5],"clip":false}],"contact":[]},{"id":"deep_decon_wall_east_south","collision_index":5,"sides":[{"a":[24.0,9.5],"b":[24.0,15.5],"clip":false}],"contact":[]}]

func _same_json(value: Variant, expected: Variant, depth: int = 0) -> bool:
	if depth > 64: return false
	if typeof(expected) == TYPE_INT or typeof(expected) == TYPE_FLOAT: return P.number(value) and float(value) == float(expected)
	if typeof(value) != typeof(expected): return false
	if expected is Dictionary:
		if value.size() != expected.size(): return false
		for key: Variant in expected:
			if not value.has(key) or not _same_json(value[key],expected[key],depth+1): return false
		return true
	if expected is Array:
		if value.size() != expected.size(): return false
		for index: int in range(expected.size()):
			if not _same_json(value[index],expected[index],depth+1): return false
		return true
	return value == expected
func _v2(value: Variant) -> bool:
	return typeof(value) == TYPE_VECTOR2 and is_finite(value.x) and is_finite(value.y)
func _v3(value: Variant) -> bool:
	return typeof(value) == TYPE_VECTOR3 and is_finite(value.x) and is_finite(value.y) and is_finite(value.z)
func _rect_ok(value: Variant) -> bool:
	if typeof(value) != TYPE_RECT2: return false
	var rect: Rect2 = value
	return _v2(rect.position) and _v2(rect.size) and _v2(rect.end) and rect.size.x > 0.0 and rect.size.y > 0.0 and rect.position == Vector2.ZERO
func _canvas_live(value: Variant) -> bool:
	return value is CanvasItem and is_instance_valid(value) and value.is_inside_tree() and value.is_visible_in_tree() and not value.is_queued_for_deletion()
func _projection_ok(value: Variant, viewport: Rect2) -> bool:
	if not value is RefCounted or not is_instance_valid(value) or value.get_script() != ProjectionScript: return false
	var size: Variant = value.get('viewport')
	var ppm: Variant = value.get('ppm')
	return typeof(value.get('initialized')) == TYPE_BOOL and value.get('initialized') and _v3(value.get('origin')) and _v2(value.get('screen_center')) and _v2(size) and size.x > 0.0 and size.y > 0.0 and P.number(ppm,0.0,256.0) and ppm > 0.0 and size == viewport.size
func _context_ok(snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant) -> bool:
	if not snapshot is Dictionary or not P.snapshot(snapshot) or not scene is Dictionary or snapshot.get('worldId') != 'grey_hive' or snapshot.get('sceneId') != 'gh_deep_decon' or snapshot.get('entryToken') != null: return false
	var epoch: Variant = snapshot.get('worldEpoch')
	return P.counter(epoch,1) and typeof(key) == TYPE_STRING and key == 'grey_hive:gh_deep_decon:'+str(epoch) and typeof(scene_hash) == TYPE_STRING and scene_hash == SCENE_SHA256 and _same_json(scene,CANONICAL_SCENE)
func _binding_ok(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, projection: Variant, viewport_value: Variant) -> bool:
	if not _canvas_live(canvas) or not _rect_ok(viewport_value) or not _context_ok(snapshot,scene,key,scene_hash): return false
	var viewport: Rect2 = viewport_value
	if not _projection_ok(projection,viewport): return false
	var actual_snapshot: Variant = canvas.get('snapshot')
	var actual_scene: Variant = canvas.get('scene')
	var actual_key: Variant = canvas.get('current_key')
	var actual_hash: Variant = canvas.get('scene_hash')
	var actual_camera: Variant = canvas.get('camera')
	var stored_viewport: Variant = canvas.get('viewport_rect')
	var actual_viewport: Rect2 = canvas.get_viewport_rect()
	return actual_snapshot is Dictionary and _same_json(actual_snapshot,snapshot) and actual_scene is Dictionary and _same_json(actual_scene,scene) and typeof(actual_key) == TYPE_STRING and actual_key == key and typeof(actual_hash) == TYPE_STRING and actual_hash == scene_hash and actual_camera == projection and typeof(stored_viewport) == TYPE_RECT2 and stored_viewport == viewport and actual_viewport == viewport
func _capture(snapshot: Dictionary, projection: RefCounted) -> Array:
	return [snapshot.worldEpoch,snapshot.serverTick,snapshot.authorityRevision,projection.origin,projection.screen_center,projection.viewport,projection.ppm]
func _capture_matches(snapshot: Dictionary, projection: RefCounted, captured: Array) -> bool:
	return captured.size() == 7 and snapshot.worldEpoch == captured[0] and snapshot.serverTick == captured[1] and snapshot.authorityRevision == captured[2] and projection.origin == captured[3] and projection.screen_center == captured[4] and projection.viewport == captured[5] and projection.ppm == captured[6]
func _owner_index(wall: Variant) -> int:
	if not wall is Dictionary or typeof(wall.get('id')) != TYPE_STRING: return -1
	for index: int in range(WALL_PLAN.size()):
		var plan: Dictionary = WALL_PLAN[index]
		if wall.get('id') == plan.id: return index if _same_json(wall,CANONICAL_SCENE.collision[plan.collision_index]) else -1
	return -1
func _project_world(projection: RefCounted, world: Array) -> PackedVector2Array:
	var points: PackedVector2Array = PackedVector2Array()
	for value: Variant in world:
		if not _v3(value): return PackedVector2Array()
		var point: Variant = projection.call('project',value)
		if not _v2(point): return PackedVector2Array()
		points.append(point)
	return points
func _color_ok(color: Color) -> bool:
	return is_finite(color.r) and is_finite(color.g) and is_finite(color.b) and is_finite(color.a) and color.r >= 0.0 and color.r <= 1.0 and color.g >= 0.0 and color.g <= 1.0 and color.b >= 0.0 and color.b <= 1.0 and color.a >= 0.0 and color.a <= 1.0
func _quad_ok(points: PackedVector2Array, color: Color) -> bool:
	if points.size() != 4 or not _color_ok(color): return false
	var lo: Vector2 = Vector2(INF,INF)
	var hi: Vector2 = Vector2(-INF,-INF)
	for point: Vector2 in points:
		if not _v2(point): return false
		lo = lo.min(point)
		hi = hi.max(point)
	if not _v2(lo) or not _v2(hi) or hi.x <= lo.x or hi.y <= lo.y: return false
	var area: float = 0.0
	for index: int in range(1,3):
		area += (points[index]-points[0]).cross(points[index+1]-points[0])
	if not is_finite(area) or area == 0.0: return false
	for index: int in range(4):
		var edge: Vector2 = points[(index+1)%4]-points[index]
		var next_edge: Vector2 = points[(index+2)%4]-points[(index+1)%4]
		var turn: float = edge.cross(next_edge)
		if not _v2(edge) or not _v2(next_edge) or edge == Vector2.ZERO or next_edge == Vector2.ZERO or not is_finite(turn) or turn == 0.0 or (turn > 0.0) != (area > 0.0): return false
	return true
func _visible(points: PackedVector2Array, viewport: Rect2) -> int:
	var axes: Array[Vector2] = [Vector2.RIGHT,Vector2.DOWN]
	for index: int in range(points.size()):
		var edge: Vector2 = points[(index+1)%points.size()]-points[index]
		axes.append(Vector2(-edge.y,edge.x))
	var corners: PackedVector2Array = PackedVector2Array([viewport.position,Vector2(viewport.end.x,viewport.position.y),viewport.end,Vector2(viewport.position.x,viewport.end.y)])
	var separated: bool = false
	for axis: Vector2 in axes:
		if not _v2(axis) or axis == Vector2.ZERO:
			return -1
		var polygon_min: float = INF
		var polygon_max: float = -INF
		var viewport_min: float = INF
		var viewport_max: float = -INF
		for point: Vector2 in points:
			var polygon_value: float = (point-points[0]).dot(axis)
			if not is_finite(polygon_value):
				return -1
			polygon_min = minf(polygon_min,polygon_value)
			polygon_max = maxf(polygon_max,polygon_value)
		for corner: Vector2 in corners:
			var viewport_value: float = (corner-points[0]).dot(axis)
			if not is_finite(viewport_value):
				return -1
			viewport_min = minf(viewport_min,viewport_value)
			viewport_max = maxf(viewport_max,viewport_value)
		if not is_finite(polygon_min) or not is_finite(polygon_max) or not is_finite(viewport_min) or not is_finite(viewport_max) or polygon_min >= polygon_max or viewport_min >= viewport_max:
			return -1
		if polygon_max <= viewport_min or viewport_max <= polygon_min:
			separated = true
	return 0 if separated else 1

func _stage_plan(projection: RefCounted, viewport: Rect2) -> Dictionary:
	var commands: Array[Dictionary] = []
	for owner_value: Variant in WALL_PLAN:
		if not owner_value is Dictionary:
			return {}
		var owner: Dictionary = owner_value
		var owner_id: String = owner.get('id')
		var contact: Variant = owner.get('contact')
		if not contact is Array:
			return {}
		if contact.size() == 4:
			for value: Variant in contact:
				if not P.number(value,0.0,24.0):
					return {}
			var x0: float = contact[0]
			var z0: float = contact[1]
			var x1: float = contact[2]
			var z1: float = contact[3]
			var contact_points: PackedVector2Array = _project_world(projection,[Vector3(x0,0,z0),Vector3(x1,0,z0),Vector3(x1,0,z1),Vector3(x0,0,z1)])
			commands.append({'owner':owner_id,'kind':'contact','points':contact_points,'color':CONTACT_COLOR})
		elif not contact.is_empty():
			return {}
		var sides: Variant = owner.get('sides')
		if not sides is Array:
			return {}
		for side_value: Variant in sides:
			if not side_value is Dictionary or typeof(side_value.get('clip')) != TYPE_BOOL:
				return {}
			var a: Variant = side_value.get('a')
			var b: Variant = side_value.get('b')
			if not a is Array or not b is Array or a.size() != 2 or b.size() != 2 or not P.number(a[0],0.0,24.0) or not P.number(a[1],0.0,16.0) or not P.number(b[0],0.0,24.0) or not P.number(b[1],0.0,16.0):
				return {}
			var ground_a: Vector3 = Vector3(a[0],0,a[1])
			var ground_b: Vector3 = Vector3(b[0],0,b[1])
			var axis: Vector3 = Vector3(b[0]-a[0],0,b[1]-a[1])
			if axis.length_squared() == 0.0:
				return {}
			axis = axis.normalized()
			var top_a: Vector3 = Vector3(a[0],HEIGHT,a[1])
			var top_b: Vector3 = Vector3(b[0],HEIGHT,b[1])
			var lower_b: Vector3 = ground_b-axis*HEIGHT if side_value.clip else ground_b
			var side_points: PackedVector2Array = _project_world(projection,[ground_a,lower_b,top_b,top_a])
			commands.append({'owner':owner_id,'kind':'side','points':side_points,'color':SIDE_COLOR})
		var collision_index: Variant = owner.get('collision_index')
		if not P.counter(collision_index,0,CANONICAL_SCENE.collision.size()-1):
			return {}
		var collision: Dictionary = CANONICAL_SCENE.collision[int(collision_index)]
		var cap_world: Array = []
		for point_value: Variant in collision.polygon:
			if not point_value is Array or point_value.size() != 2 or not P.number(point_value[0],0.0,24.0) or not P.number(point_value[1],0.0,16.0):
				return {}
			cap_world.append(Vector3(point_value[0],HEIGHT,point_value[1]))
		var cap_points: PackedVector2Array = _project_world(projection,cap_world)
		commands.append({'owner':owner_id,'kind':'cap','points':cap_points,'color':CAP_COLOR})
	if commands.size() != 19:
		return {}
	var visible_commands: Array[Dictionary] = []
	for command: Dictionary in commands:
		if not _quad_ok(command.points,command.color):
			return {}
		var visibility: int = _visible(command.points,viewport)
		if visibility < 0:
			return {}
		if visibility == 1:
			visible_commands.append(command)
	return {'all':commands,'visible':visible_commands}

func draw_wall(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, scene_hash_value: Variant, projection_value: Variant, wall_value: Variant, viewport_value: Variant) -> bool:
	if not _binding_ok(canvas_value,snapshot_value,scene_value,current_key_value,scene_hash_value,projection_value,viewport_value):
		return false
	var owner_index: int = _owner_index(wall_value)
	if owner_index < 0:
		return false
	var snapshot: Dictionary = snapshot_value
	var projection: RefCounted = projection_value
	var viewport: Rect2 = viewport_value
	var captured: Array = _capture(snapshot,projection)
	var plan: Dictionary = _stage_plan(projection,viewport)
	if plan.is_empty() or not plan.has('all') or plan.all.size() != 19:
		return false
	var owner_id: String = WALL_PLAN[owner_index].id
	var commands: Array[Dictionary] = []
	for command: Dictionary in plan.visible:
		if command.owner == owner_id:
			commands.append(command)
	if commands.is_empty() or not _binding_ok(canvas_value,snapshot_value,scene_value,current_key_value,scene_hash_value,projection_value,viewport_value) or not _capture_matches(snapshot,projection,captured):
		return false
	var canvas: CanvasItem = canvas_value
	for command: Dictionary in commands:
		canvas.draw_colored_polygon(command.points,command.color)
	return true
