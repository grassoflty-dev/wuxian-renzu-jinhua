extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const V = preload('res://scripts/world/presentation.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')

const WORLD_ID: String = 'grey_hive'
const SCENE_ID: String = 'gh_deep_decon'
const STEAM_ID: String = 'gh_decon_steam_jet_zone'
const MIST_ID: String = 'gh_decon_mist_zone'
const STEAM_POLYGON: Array = [[7.0,4.0],[10.0,4.0],[10.0,6.5],[7.0,6.5]]
const MIST_POLYGON: Array = [[14.0,9.0],[18.0,9.0],[18.0,14.0],[14.0,14.0]]

func _finite_vector2(value) -> bool:
	if typeof(value) != TYPE_VECTOR2:
		return false
	var point: Vector2 = value
	return is_finite(point.x) and is_finite(point.y)

func _finite_vector3(value) -> bool:
	if typeof(value) != TYPE_VECTOR3:
		return false
	var point: Vector3 = value
	return is_finite(point.x) and is_finite(point.y) and is_finite(point.z)

func _finite_color(value) -> bool:
	if typeof(value) != TYPE_COLOR:
		return false
	var color: Color = value
	return is_finite(color.r) and is_finite(color.g) and is_finite(color.b) and is_finite(color.a) and color.a >= 0.0 and color.a <= 1.0

func _valid_viewport(value) -> bool:
	if typeof(value) != TYPE_RECT2:
		return false
	var viewport_rect: Rect2 = value
	return is_finite(viewport_rect.position.x) and is_finite(viewport_rect.position.y) and is_finite(viewport_rect.size.x) and is_finite(viewport_rect.size.y) and viewport_rect.size.x > 0.0 and viewport_rect.size.y > 0.0

func _polygon_matches(value, expected: Array) -> bool:
	if not value is Array or value.size() != 4:
		return false
	var points: Array = value
	for index in range(4):
		var point_value: Variant = points[index]
		if not point_value is Array:
			return false
		var point: Array = point_value
		var expected_point: Array = expected[index]
		if point.size() != 2 or not P.number(point[0],float(expected_point[0]),float(expected_point[0])) or not P.number(point[1],float(expected_point[1]),float(expected_point[1])):
			return false
	return true

func _source_row_matches(row: Dictionary, hazard_id: String, hazard_kind: String, tag: String, polygon: Array, warning_ms: int, active_ms: int, damage: int, recovery_ms: int, interval_ms: int) -> bool:
	if row.size() != 4 or row.get('id') != hazard_id or row.get('kind') != hazard_kind:
		return false
	if not _polygon_matches(row.get('polygon'),polygon):
		return false
	var environment_value: Variant = row.get('environment')
	if not environment_value is Dictionary:
		return false
	var environment: Dictionary = environment_value
	return environment.size() == 6 and environment.has('tag') and environment.has('warningMs') and environment.has('activeMs') and environment.has('recoveryMs') and environment.has('damage') and environment.has('damageIntervalMs') and environment.get('tag') == tag and P.number(environment.get('warningMs'),warning_ms,warning_ms) and P.number(environment.get('activeMs'),active_ms,active_ms) and P.number(environment.get('recoveryMs'),recovery_ms,recovery_ms) and P.number(environment.get('damage'),damage,damage) and P.number(environment.get('damageIntervalMs'),interval_ms,interval_ms)

func _valid_scene(scene: Dictionary) -> bool:
	if scene.get('worldId') != WORLD_ID or scene.get('sceneId') != SCENE_ID or not P.counter(scene.get('schemaVersion'),1,1):
		return false
	var bounds_value: Variant = scene.get('boundsM')
	if not bounds_value is Dictionary:
		return false
	var bounds: Dictionary = bounds_value
	if bounds.size() != 4 or not P.number(bounds.get('x'),0.0,0.0) or not P.number(bounds.get('z'),0.0,0.0) or not P.number(bounds.get('width'),24.0,24.0) or not P.number(bounds.get('depth'),16.0,16.0):
		return false
	var hazards_value: Variant = scene.get('hazards')
	if not hazards_value is Array or hazards_value.size() != 2:
		return false
	var hazards: Array = hazards_value
	var seen: Dictionary = {}
	for row_value in hazards:
		if not row_value is Dictionary:
			return false
		var row: Dictionary = row_value
		var id_value: Variant = row.get('id')
		if typeof(id_value) != TYPE_STRING or seen.has(id_value):
			return false
		seen[id_value] = true
		if id_value == STEAM_ID:
			if not _source_row_matches(row,STEAM_ID,'steam_jet','pressure',STEAM_POLYGON,900,700,8,2400,700):
				return false
		elif id_value == MIST_ID:
			if not _source_row_matches(row,MIST_ID,'decon_mist','toxin',MIST_POLYGON,1200,4000,6,1500,1000):
				return false
		else:
			return false
	return seen.size() == 2 and seen.has(STEAM_ID) and seen.has(MIST_ID)

func _snapshot_hazard_matches(row: Dictionary, hazard_id: String, hazard_kind: String, tag: String, polygon: Array) -> bool:
	if row.get('entityId') != hazard_id or row.get('kind') != hazard_kind or typeof(row.get('active')) != TYPE_BOOL or typeof(row.get('phaseActive')) != TYPE_BOOL or row.get('phaseActive') != true or not P.transform(row.get('transform')):
		return false
	if not _polygon_matches(row.get('polygonM'),polygon):
		return false
	var environment_value: Variant = row.get('environment')
	if not environment_value is Dictionary:
		return false
	var environment: Dictionary = environment_value
	if environment.size() != 4 or typeof(environment.get('phase')) != TYPE_STRING or typeof(environment.get('tag')) != TYPE_STRING or environment.get('tag') != tag:
		return false
	var accepted: Dictionary = V.environment(row)
	if accepted.is_empty():
		return false
	if accepted.phase == 'warning':
		return row.size() == 8 and P.counter(row.get('warningRemainingMs'),1,10000) and row.warningRemainingMs == accepted.remainingMs
	return row.size() == 7 and not row.has('warningRemainingMs')

func _valid_snapshot_hazards(snapshot: Dictionary) -> bool:
	var hazards_value: Variant = snapshot.get('hazards')
	if not hazards_value is Array or hazards_value.size() != 2:
		return false
	var hazards: Array = hazards_value
	var seen: Dictionary = {}
	for row_value in hazards:
		if not row_value is Dictionary:
			return false
		var row: Dictionary = row_value
		var id_value: Variant = row.get('entityId')
		if typeof(id_value) != TYPE_STRING or seen.has(id_value):
			return false
		seen[id_value] = true
		if id_value == STEAM_ID:
			if not _snapshot_hazard_matches(row,STEAM_ID,'steam_jet','pressure',STEAM_POLYGON):
				return false
		elif id_value == MIST_ID:
			if not _snapshot_hazard_matches(row,MIST_ID,'decon_mist','toxin',MIST_POLYGON):
				return false
		else:
			return false
	return seen.size() == 2 and seen.has(STEAM_ID) and seen.has(MIST_ID)

func _valid_context(snapshot_value, scene_value, current_key_value, camera_value, viewport_value) -> bool:
	if not snapshot_value is Dictionary or not scene_value is Dictionary or typeof(current_key_value) != TYPE_STRING:
		return false
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	if not P.snapshot(snapshot) or snapshot.get('entryToken') != null or snapshot.get('worldId') != WORLD_ID or snapshot.get('sceneId') != SCENE_ID or not P.counter(snapshot.get('worldEpoch'),1):
		return false
	if current_key_value != WORLD_ID + ':' + SCENE_ID + ':' + str(snapshot.worldEpoch) or not _valid_scene(scene) or not _valid_snapshot_hazards(snapshot):
		return false
	if not camera_value is RefCounted or not is_instance_valid(camera_value):
		return false
	var camera: RefCounted = camera_value
	if camera.get_script() != ProjectionScript:
		return false
	var origin_value: Variant = camera.get('origin')
	var center_value: Variant = camera.get('screen_center')
	var camera_viewport_value: Variant = camera.get('viewport')
	var ppm_value: Variant = camera.get('ppm')
	var initialized_value: Variant = camera.get('initialized')
	if not _finite_vector3(origin_value) or not _finite_vector2(center_value) or not _finite_vector2(camera_viewport_value) or not P.number(ppm_value,0.000001,256.0) or typeof(initialized_value) != TYPE_BOOL or initialized_value != true or not _valid_viewport(viewport_value):
		return false
	var camera_viewport: Vector2 = camera_viewport_value
	var viewport_rect: Rect2 = viewport_value
	return camera_viewport.x > 0.0 and camera_viewport.y > 0.0 and absf(camera_viewport.x-viewport_rect.size.x) <= 0.001 and absf(camera_viewport.y-viewport_rect.size.y) <= 0.001

func _rect_intersects(a: Rect2, b: Rect2) -> bool:
	return a.position.x <= b.end.x and a.end.x >= b.position.x and a.position.y <= b.end.y and a.end.y >= b.position.y

func _append_world_polygon(shapes: Array[Dictionary], world_points_value, camera: RefCounted, zone: Rect2, max_height: float, color: Color) -> bool:
	if not world_points_value is Array or world_points_value.size() < 3 or world_points_value.size() > 12 or not P.number(max_height,0.0,0.9) or not _finite_color(color):
		return false
	var world_points: Array = world_points_value
	var screen_points: PackedVector2Array = PackedVector2Array()
	var low: Vector2 = Vector2(INF,INF)
	var high: Vector2 = Vector2(-INF,-INF)
	for world_value in world_points:
		if not _finite_vector3(world_value):
			return false
		var world_point: Vector3 = world_value
		if world_point.x < zone.position.x or world_point.x > zone.end.x or world_point.z < zone.position.y or world_point.z > zone.end.y or world_point.y < 0.0 or world_point.y > max_height:
			return false
		var projected_value: Variant = camera.call('project',world_point)
		if not _finite_vector2(projected_value):
			return false
		var projected: Vector2 = projected_value
		screen_points.append(projected)
		low = low.min(projected)
		high = high.max(projected)
	var bounds: Rect2 = Rect2(low,high-low)
	if bounds.size.x <= 0.0 or bounds.size.y <= 0.0:
		return false
	shapes.append({'points':screen_points,'bounds':bounds,'color':color})
	return true

func _append_steam(shapes: Array[Dictionary], phase: String, camera: RefCounted) -> bool:
	if phase == 'suppressed':
		return true
	var height: float = 0.9 if phase == 'active' else 0.2 if phase == 'warning' else 0.12
	var alpha_cap: float = 0.10 if phase == 'active' else 0.035 if phase == 'warning' else 0.02
	var bases: Array[Vector2] = [Vector2(7.65,4.65),Vector2(8.5,5.25),Vector2(9.35,5.85)]
	var widths: Array[float] = [0.20,0.14,0.085]
	var heights: Array[float] = [1.0,0.72,0.48]
	var alpha_weights: Array[float] = [0.40,0.70,1.0]
	var zone: Rect2 = Rect2(7.0,4.0,3.0,2.5)
	var base_color: Color = Color('#b5cbd1')
	for base: Vector2 in bases:
		for layer in range(3):
			var half_width: float = widths[layer]
			var top_half_width: float = half_width*0.28
			var top_y: float = height*heights[layer]
			var drift: float = 0.08*heights[layer]
			var top_x: float = base.x+drift
			var top_z: float = base.y-drift
			var points: Array = [
				Vector3(base.x-half_width,0.01,base.y+half_width),
				Vector3(base.x+half_width,0.01,base.y-half_width),
				Vector3(base.x+half_width*0.85,top_y*0.30,base.y-half_width*0.85),
				Vector3(top_x+half_width*0.60,top_y*0.72,top_z-half_width*0.60),
				Vector3(top_x+top_half_width,top_y,top_z-top_half_width),
				Vector3(top_x-top_half_width,top_y,top_z+top_half_width),
				Vector3(top_x-half_width*0.60,top_y*0.72,top_z+half_width*0.60),
				Vector3(base.x-half_width*0.85,top_y*0.30,base.y+half_width*0.85)
			]
			var color: Color = base_color
			color.a = alpha_cap*alpha_weights[layer]
			if not _append_world_polygon(shapes,points,camera,zone,height,color):
				return false
	return true

func _append_mist(shapes: Array[Dictionary], phase: String, camera: RefCounted) -> bool:
	if phase == 'suppressed':
		return true
	var height: float = 0.45 if phase == 'active' else 0.15 if phase == 'warning' else 0.08
	var alpha_cap: float = 0.07 if phase == 'active' else 0.025 if phase == 'warning' else 0.015
	var lanes: Array[float] = [9.65,10.85,12.15,13.35]
	var starts: Array[float] = [14.35,15.45,16.65]
	var ends: Array[float] = [15.15,16.25,17.65]
	var heights: Array[float] = [0.72,1.0,0.78]
	var alpha_weights: Array[float] = [0.75,1.0,0.75]
	var z_half: Array[float] = [0.12,0.10,0.12]
	var zone: Rect2 = Rect2(14.0,9.0,4.0,5.0)
	var base_color: Color = Color('#80a99e')
	for lane_index in range(lanes.size()):
		var lane_z: float = lanes[lane_index]
		for segment in range(3):
			var x0: float = starts[segment]
			var x1: float = ends[segment]
			var segment_height: float = height*heights[segment]
			var half_depth: float = z_half[segment]
			var taper: float = minf(0.16,(x1-x0)*0.18)
			var points: Array = [
				Vector3(x0,0.015,lane_z),
				Vector3(x1,0.015,lane_z),
				Vector3(x1-taper*0.35,segment_height*0.30,lane_z+half_depth*0.30),
				Vector3(x1-taper,segment_height*0.75,lane_z+half_depth*0.65),
				Vector3((x0+x1)*0.5,segment_height,lane_z),
				Vector3(x0+taper,segment_height*0.75,lane_z-half_depth*0.65),
				Vector3(x0+taper*0.35,segment_height*0.30,lane_z-half_depth*0.30)
			]
			var color: Color = base_color
			color.a = alpha_cap*alpha_weights[segment]
			if not _append_world_polygon(shapes,points,camera,zone,height,color):
				return false
	return true

func draw_air(canvas_value, snapshot_value, scene_value, current_key_value, camera_value, viewport_value) -> int:
	if not canvas_value is CanvasItem or not is_instance_valid(canvas_value) or not _valid_context(snapshot_value,scene_value,current_key_value,camera_value,viewport_value):
		return 0
	var canvas: CanvasItem = canvas_value
	if not canvas.is_inside_tree() or canvas.is_queued_for_deletion():
		return 0
	var snapshot: Dictionary = snapshot_value
	var camera: RefCounted = camera_value
	var viewport_rect: Rect2 = viewport_value
	var phases: Dictionary = {}
	for row_value in snapshot.hazards:
		var row: Dictionary = row_value
		var environment: Dictionary = row.environment
		phases[row.entityId] = environment.phase
	var shapes: Array[Dictionary] = []
	if not _append_steam(shapes,phases[STEAM_ID],camera) or not _append_mist(shapes,phases[MIST_ID],camera):
		return 0
	var drawn: int = 0
	for shape: Dictionary in shapes:
		var bounds: Rect2 = shape.bounds
		if not _rect_intersects(viewport_rect,bounds):
			continue
		var points: PackedVector2Array = shape.points
		var color: Color = shape.color
		canvas.draw_colored_polygon(points,color)
		drawn += 1
	return drawn
