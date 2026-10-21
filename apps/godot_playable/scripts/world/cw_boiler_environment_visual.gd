extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const FurnaceVisualScript = preload('res://scripts/world/cw_furnace_visual.gd')

const WALLS = {
	'cw_boiler_chamber_wall_north': [[0.0,0.0],[24.0,0.0],[24.0,0.5],[0.0,0.5]],
	'cw_boiler_chamber_wall_south': [[0.0,15.5],[24.0,15.5],[24.0,16.0],[0.0,16.0]],
	'cw_boiler_chamber_wall_west_north': [[0.0,0.5],[0.5,0.5],[0.5,6.0],[0.0,6.0]],
	'cw_boiler_chamber_wall_west_south': [[0.0,9.5],[0.5,9.5],[0.5,15.5],[0.0,15.5]],
	'cw_boiler_chamber_wall_east_north': [[23.5,0.5],[24.0,0.5],[24.0,6.0],[23.5,6.0]],
	'cw_boiler_chamber_wall_east_south': [[23.5,9.5],[24.0,9.5],[24.0,15.5],[23.5,15.5]]
}
const OUTER_WALL_EDGES: Dictionary = {
	'cw_boiler_chamber_wall_south': [[0.0,16.0],[24.0,16.0]],
	'cw_boiler_chamber_wall_east_north': [[24.0,0.5],[24.0,6.0]],
	'cw_boiler_chamber_wall_east_south': [[24.0,9.5],[24.0,15.5]]
}
const WALL_INNER = {
	'cw_boiler_chamber_wall_north': [[0.0,0.5],[24.0,0.5]],
	'cw_boiler_chamber_wall_south': [[0.0,15.5],[24.0,15.5]],
	'cw_boiler_chamber_wall_west_north': [[0.5,0.5],[0.5,6.0]],
	'cw_boiler_chamber_wall_west_south': [[0.5,9.5],[0.5,15.5]],
	'cw_boiler_chamber_wall_east_north': [[23.5,0.5],[23.5,6.0]],
	'cw_boiler_chamber_wall_east_south': [[23.5,9.5],[23.5,15.5]]
}
const WALL_CONTACT = {
	'cw_boiler_chamber_wall_north': [[0.0,0.44],[24.0,0.44],[24.0,0.5],[0.0,0.5]],
	'cw_boiler_chamber_wall_south': [[0.0,15.5],[24.0,15.5],[24.0,15.56],[0.0,15.56]],
	'cw_boiler_chamber_wall_west_north': [[0.44,0.5],[0.5,0.5],[0.5,6.0],[0.44,6.0]],
	'cw_boiler_chamber_wall_west_south': [[0.44,9.5],[0.5,9.5],[0.5,15.5],[0.44,15.5]],
	'cw_boiler_chamber_wall_east_north': [[23.5,0.5],[23.56,0.5],[23.56,6.0],[23.5,6.0]],
	'cw_boiler_chamber_wall_east_south': [[23.5,9.5],[23.56,9.5],[23.56,15.5],[23.5,15.5]]
}
const SPRITES = {
	'cw_boiler_chamber_steel_floor': ['runtime2d.world.clockworks.floor_steel.v1','visual.floor',12.0,0.0,8.0],
	'cw_boiler_chamber_factory_tiles': ['runtime2d.world.clockworks.factory_tiles.v1','visual.floor',12.0,0.0,8.0],
	'cw_boiler_pressure_pipe_run': ['runtime2d.world.clockworks.pressure_pipes.v1','visual.props_back',11.0,0.0,4.0],
	'cw_boiler_furnace': ['runtime2d.world.clockworks.furnace.v2','visual.props_back',16.0,0.0,6.0],
	'cw_boiler_furnace_variant': ['runtime2d.world.clockworks.furnace.v1','visual.props_dynamic',18.0,0.0,10.0],
	'cw_boiler_pressure_pipe_branch': ['runtime2d.world.clockworks.pressure_pipes.v1','visual.props_dynamic',8.0,0.0,12.0]
}
const FURNACES = {
	'furnace_v2': [16.0,0.0,6.0],
	'furnace_v1': [18.0,0.0,10.0]
}
const FLOOR_BASE: Color = Color(52.0/255.0,58.0/255.0,60.0/255.0,1.0)
const FLOOR_SEAM: Color = Color(0.055,0.065,0.068,0.11)
const FLOOR_CONTACT: Color = Color(0.035,0.045,0.048,0.16)
const WALL_SIDE: Color = Color('#282e31')
const WALL_CAP: Color = Color('#4d5556')
const SHORT_SCENE_SHA: String = "04b00520e16435d159f1a459016083020fd40d11a2f8086d2f78dcbd0e483b39"
const SHORT_CANONICAL_SCENE: Dictionary = {"boundsM":{"depth":16.0,"width":24.0,"x":0,"z":0},"cameraZones":[],"checkpoints":[{"id":"cw_boiler_chamber_checkpoint","navNode":"cw_boiler_chamber_spawn_nav","position":[4.0,0.0,8.0]}],"collision":[{"id":"cw_boiler_chamber_wall_north","polygon":[[0.0,0.0],[24.0,0.0],[24.0,0.5],[0.0,0.5]]},{"id":"cw_boiler_chamber_wall_south","polygon":[[0.0,15.5],[24.0,15.5],[24.0,16.0],[0.0,16.0]]},{"id":"cw_boiler_chamber_wall_west_north","polygon":[[0.0,0.5],[0.5,0.5],[0.5,6.0],[0.0,6.0]]},{"id":"cw_boiler_chamber_wall_west_south","polygon":[[0.0,9.5],[0.5,9.5],[0.5,15.5],[0.0,15.5]]},{"id":"cw_boiler_chamber_wall_east_north","polygon":[[23.5,0.5],[24.0,0.5],[24.0,6.0],[23.5,6.0]]},{"id":"cw_boiler_chamber_wall_east_south","polygon":[[23.5,9.5],[24.0,9.5],[24.0,15.5],[23.5,15.5]]}],"doors":[],"hazards":[{"environment":{"activeMs":8000,"damage":8,"damageIntervalMs":1000,"exposure":{"damageThreshold":50,"gainPerSecond":25,"lossPerSecond":20,"maxUnits":100},"recoveryMs":3000,"tag":"heat","warningMs":1000},"id":"cw_heat_accumulation_staged","kind":"heat_zone","polygon":[[11.0,2.0],[17.0,2.0],[17.0,6.0],[11.0,6.0]]},{"environment":{"activeMs":700,"damage":10,"damageIntervalMs":700,"recoveryMs":3400,"tag":"pressure","warningMs":900},"id":"cw_boiler_vent_staged","kind":"steam_jet","polygon":[[11.0,10.0],[13.0,10.0],[13.0,13.0],[11.0,13.0]]}],"interactions":[{"cooldownMs":8000,"environmentControl":{"exposureReductionUnits":40,"suppressionMs":6000,"targetHazardIds":["cw_heat_accumulation_staged","cw_boiler_vent_staged"]},"event":null,"id":"cw_coolant_valve_staged","kind":"environment_control","position":[18.0,0.0,8.0],"rangeM":2.5}],"logic":{"traversal":[]},"navigation":{"links":[{"from":"cw_boiler_chamber_spawn_nav","to":"cw_boiler_chamber_center_nav"},{"from":"cw_boiler_chamber_center_nav","to":"cw_boiler_chamber_spawn_nav"}],"nodes":[{"id":"cw_boiler_chamber_spawn_nav","position":[3.0,0.0,8.0]},{"id":"cw_boiler_chamber_center_nav","position":[12.0,0.0,8.0]}]},"objectives":[],"occluders":[],"presentation":{"cameraProfile":"oblique_default","layers":[{"id":"visual.floor","zGroup":0},{"id":"visual.props_back","zGroup":10},{"id":"visual.props_dynamic","zGroup":20}],"sprites":[{"assetId":"runtime2d.world.clockworks.floor_steel.v1","id":"cw_boiler_chamber_steel_floor","layer":"visual.floor","position":[12.0,0.0,8.0]},{"assetId":"runtime2d.world.clockworks.factory_tiles.v1","id":"cw_boiler_chamber_factory_tiles","layer":"visual.floor","position":[12.0,0.0,8.0]},{"assetId":"runtime2d.world.clockworks.pressure_pipes.v1","id":"cw_boiler_pressure_pipe_run","layer":"visual.props_back","position":[11.0,0.0,4.0]},{"assetId":"runtime2d.world.clockworks.furnace.v2","id":"cw_boiler_furnace","layer":"visual.props_back","position":[16.0,0.0,6.0]},{"assetId":"runtime2d.world.clockworks.furnace.v1","id":"cw_boiler_furnace_variant","layer":"visual.props_dynamic","position":[18.0,0.0,10.0]},{"assetId":"runtime2d.world.clockworks.pressure_pipes.v1","id":"cw_boiler_pressure_pipe_branch","layer":"visual.props_dynamic","position":[8.0,0.0,12.0]}]},"sceneId":"cw_boiler_chamber","schemaVersion":1,"spawns":[{"entityType":null,"id":"cw_boiler_chamber_spawn","kind":"player","navNode":"cw_boiler_chamber_spawn_nav","position":[3.0,0.0,8.0]},{"entityType":"enemy.clockworks.furnace_hound","id":"cw_boiler_chamber_furnace_hound_01","kind":"enemy","navNode":null,"position":[10.0,0.0,4.0]},{"entityType":"enemy.clockworks.furnace_hound","id":"cw_boiler_chamber_furnace_hound_02","kind":"enemy","navNode":null,"position":[16.0,0.0,12.0]},{"entityType":"enemy.clockworks.furnace_hound","id":"cw_boiler_chamber_furnace_hound_03","kind":"enemy","navNode":null,"position":[20.0,0.0,5.0]}],"transitions":[{"id":"cw_boiler_return_to_conveyor_bridge","polygon":[[0.5,6.5],[2.5,6.5],[2.5,9.5],[0.5,9.5]],"spawnId":"cw_conveyor_bridge_spawn","toSceneId":"cw_conveyor_bridge"},{"id":"cw_boiler_to_gear_shaft","polygon":[[22.0,6.5],[24.0,6.5],[24.0,9.5],[22.0,9.5]],"spawnId":"cw_gear_shaft_spawn","toSceneId":"cw_gear_shaft"}],"triggers":[],"vfxMarkers":[],"worldId":"clockworks"}
const SHORT_FACES: Array[Dictionary] = [{"id":"north_east_end","owner":"cw_boiler_chamber_wall_north","a":[24.0,0.0],"b":[24.0,0.5],"low":0.0,"high":0.75},{"id":"south_east_end","owner":"cw_boiler_chamber_wall_south","a":[24.0,15.5],"b":[24.0,16.0],"low":0.0,"high":0.22},{"id":"west_north_jamb","owner":"cw_boiler_chamber_wall_west_north","a":[0.0,6.0],"b":[0.5,6.0],"low":0.0,"high":0.75},{"id":"west_south_step","owner":"cw_boiler_chamber_wall_west_south","a":[0.0,15.5],"b":[0.5,15.5],"low":0.22,"high":0.75},{"id":"east_north_jamb","owner":"cw_boiler_chamber_wall_east_north","a":[23.5,6.0],"b":[24.0,6.0],"low":0.0,"high":0.22}]

func _finite_vector2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _finite_vector3(value: Vector3) -> bool:
	return is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _finite_color(value: Color) -> bool:
	return is_finite(value.r) and is_finite(value.g) and is_finite(value.b) and is_finite(value.a)

func _finite_rect(value: Rect2) -> bool:
	return _finite_vector2(value.position) and _finite_vector2(value.size) and _finite_vector2(value.end) and value.size.x > 0.0 and value.size.y > 0.0

func _valid_canvas(value) -> bool:
	if not value is CanvasItem or not is_instance_valid(value):
		return false
	var canvas: CanvasItem = value
	return not canvas.is_queued_for_deletion() and canvas.is_inside_tree()

func _valid_projection(value) -> bool:
	if value == null or not value is RefCounted or not is_instance_valid(value) or value.get_script() != ProjectionScript:
		return false
	var ppm_value: Variant = value.get('ppm')
	var origin_value: Variant = value.get('origin')
	var center_value: Variant = value.get('screen_center')
	var projection_viewport_value: Variant = value.get('viewport')
	if not P.number(ppm_value,0.000001,256.0) or not origin_value is Vector3 or not center_value is Vector2 or not projection_viewport_value is Vector2:
		return false
	var ppm: float = float(ppm_value)
	var origin: Vector3 = origin_value
	var center: Vector2 = center_value
	var projection_viewport: Vector2 = projection_viewport_value
	return is_finite(ppm) and _finite_vector3(origin) and _finite_vector2(center) and _finite_vector2(projection_viewport) and projection_viewport.x > 0.0 and projection_viewport.y > 0.0

func _number_matches(value: Variant, expected: float) -> bool:
	return P.number(value,expected,expected)

func _polygon_matches(value: Variant, expected: Array) -> bool:
	if not value is Array or value.size() != expected.size():
		return false
	for index in range(expected.size()):
		var actual_point: Variant = value[index]
		var expected_point: Variant = expected[index]
		if not actual_point is Array or not expected_point is Array or actual_point.size() != 2:
			return false
		var expected_x: float = float(expected_point[0])
		var expected_z: float = float(expected_point[1])
		if not _number_matches(actual_point[0],expected_x) or not _number_matches(actual_point[1],expected_z):
			return false
	return true

func _position_matches(value: Variant, expected: Array) -> bool:
	if not value is Array or value.size() != 3 or expected.size() != 3:
		return false
	for index in range(3):
		if not _number_matches(value[index],float(expected[index])):
			return false
	return true

func _valid_collisions(scene: Dictionary) -> bool:
	var collision_value: Variant = scene.get('collision')
	if not P.rows(collision_value,16) or collision_value.size() != WALLS.size():
		return false
	var seen: Dictionary = {}
	for row_value in collision_value:
		if not row_value is Dictionary:
			return false
		var row: Dictionary = row_value
		var id_value: Variant = row.get('id')
		if typeof(id_value) != TYPE_STRING or not WALLS.has(id_value) or seen.has(id_value):
			return false
		var expected_polygon: Array = WALLS[id_value]
		if not _polygon_matches(row.get('polygon'),expected_polygon):
			return false
		seen[id_value] = true
	return seen.size() == WALLS.size()

func _valid_sprites(scene: Dictionary) -> bool:
	var presentation_value: Variant = scene.get('presentation')
	if not presentation_value is Dictionary:
		return false
	var presentation: Dictionary = presentation_value
	var sprites_value: Variant = presentation.get('sprites')
	if not P.rows(sprites_value,16) or sprites_value.size() != SPRITES.size():
		return false
	var seen: Dictionary = {}
	for row_value in sprites_value:
		if not row_value is Dictionary:
			return false
		var row: Dictionary = row_value
		var id_value: Variant = row.get('id')
		if typeof(id_value) != TYPE_STRING or not SPRITES.has(id_value) or seen.has(id_value):
			return false
		var expected_value: Variant = SPRITES.get(id_value)
		if not expected_value is Array:
			return false
		var expected: Array = expected_value
		if expected.size() != 5 or row.get('assetId') != expected[0] or row.get('layer') != expected[1] or not _position_matches(row.get('position'),[expected[2],expected[3],expected[4]]):
			return false
		seen[id_value] = true
	return seen.size() == SPRITES.size()

func _valid_context(snapshot_value, scene_value, current_key_value, projection, viewport_value) -> bool:
	if not P.snapshot(snapshot_value) or not scene_value is Dictionary or typeof(current_key_value) != TYPE_STRING:
		return false
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	if snapshot.get('worldId') != 'clockworks' or snapshot.get('sceneId') != 'cw_boiler_chamber' or scene.get('worldId') != 'clockworks' or scene.get('sceneId') != 'cw_boiler_chamber':
		return false
	var epoch_value: Variant = snapshot.get('worldEpoch')
	if not P.counter(epoch_value,1) or current_key_value != 'clockworks:cw_boiler_chamber:'+str(epoch_value):
		return false
	if not P.counter(scene.get('schemaVersion'),1,1):
		return false
	var bounds_value: Variant = scene.get('boundsM')
	if not bounds_value is Dictionary:
		return false
	var bounds: Dictionary = bounds_value
	if bounds.size() != 4 or not _number_matches(bounds.get('x'),0.0) or not _number_matches(bounds.get('z'),0.0) or not _number_matches(bounds.get('width'),24.0) or not _number_matches(bounds.get('depth'),16.0):
		return false
	if not _valid_collisions(scene) or not _valid_sprites(scene) or not _valid_projection(projection) or not viewport_value is Rect2:
		return false
	var viewport: Rect2 = viewport_value
	return _finite_rect(viewport)

func _project_point(projection, x: float, y: float, z: float) -> Vector2:
	var world_point: Vector3 = Vector3(x,y,z)
	var projected_value: Variant = projection.call('project',world_point)
	if not projected_value is Vector2:
		return Vector2(NAN,NAN)
	var projected: Vector2 = projected_value
	return projected

func _project_polygon(projection, coordinates: Array, height: float) -> PackedVector2Array:
	var points: PackedVector2Array = PackedVector2Array()
	for coordinate_value in coordinates:
		if not coordinate_value is Array or coordinate_value.size() != 2:
			return PackedVector2Array()
		var coordinate: Array = coordinate_value
		var point: Vector2 = _project_point(projection,float(coordinate[0]),height,float(coordinate[1]))
		if not _finite_vector2(point):
			return PackedVector2Array()
		points.append(point)
	return points

func _ellipse_points(projection, center_x: float, center_z: float, radius_x: float, radius_z: float) -> PackedVector2Array:
	var points: PackedVector2Array = PackedVector2Array()
	for index in range(24):
		var angle: float = TAU*float(index)/24.0
		var world_x: float = center_x+cos(angle)*radius_x
		var world_z: float = center_z+sin(angle)*radius_z
		var point: Vector2 = _project_point(projection,world_x,0.0,world_z)
		if not _finite_vector2(point):
			return PackedVector2Array()
		points.append(point)
	return points

func _texture_ready(furnace_visual_value, key: String) -> bool:
	if furnace_visual_value == null or not furnace_visual_value is RefCounted or not is_instance_valid(furnace_visual_value) or furnace_visual_value.get_script() != FurnaceVisualScript:
		return false
	var textures_value: Variant = furnace_visual_value.get('textures')
	if not textures_value is Dictionary:
		return false
	var textures: Dictionary = textures_value
	var texture_value: Variant = textures.get(key)
	return texture_value is Texture2D and is_instance_valid(texture_value)

func _draw_plan_polygon(points: PackedVector2Array, color: Color) -> Dictionary:
	return {'kind':'polygon','points':points,'color':color}

func _draw_plan_line(points: PackedVector2Array, color: Color, width: float) -> Dictionary:
	return {'kind':'line','points':points,'color':color,'width':width}

func _plan_valid(plan: Dictionary) -> bool:
	var kind_value: Variant = plan.get('kind')
	var points_value: Variant = plan.get('points')
	var color_value: Variant = plan.get('color')
	if typeof(kind_value) != TYPE_STRING or not points_value is PackedVector2Array or not color_value is Color:
		return false
	var points: PackedVector2Array = points_value
	var color: Color = color_value
	if points.size() < (3 if kind_value == 'polygon' else 2) or not _finite_color(color):
		return false
	for point in points:
		if not _finite_vector2(point):
			return false
	if kind_value == 'line':
		var width_value: Variant = plan.get('width')
		if not P.number(width_value,0.000001,100000.0):
			return false
	return kind_value == 'polygon' or kind_value == 'line'

func _emit_plan(canvas: CanvasItem, plan: Dictionary) -> void:
	var kind_value: Variant = plan.get('kind')
	var points_value: Variant = plan.get('points')
	var color_value: Variant = plan.get('color')
	var points: PackedVector2Array = points_value
	var color: Color = color_value
	if kind_value == 'polygon':
		canvas.draw_colored_polygon(points,color)
	else:
		var width_value: Variant = plan.get('width')
		canvas.draw_polyline(points,color,float(width_value),true)

func _append_furnace_ground(plans: Array[Dictionary], projection, center: Vector3) -> bool:
	var shadow_points: PackedVector2Array = _ellipse_points(projection,center.x,center.z,0.78,0.25)
	if shadow_points.size() != 24:
		return false
	plans.append(_draw_plan_polygon(shadow_points,Color(0.025,0.027,0.028,0.16)))
	var warm_color: Color = Color('#a96735')
	var warm_radii: Array[Vector2] = [Vector2(1.45,0.58),Vector2(0.98,0.39),Vector2(0.52,0.22)]
	var warm_alphas: Array[float] = [0.018,0.027,0.038]
	for index in range(warm_radii.size()):
		var radius: Vector2 = warm_radii[index]
		var points: PackedVector2Array = _ellipse_points(projection,center.x,center.z,radius.x,radius.y)
		if points.size() != 24:
			return false
		var color: Color = warm_color
		color.a = warm_alphas[index]
		plans.append(_draw_plan_polygon(points,color))
	return true

func draw_floor(canvas_value, snapshot_value, scene_value, current_key_value, projection, viewport_value, furnace_visual_value) -> bool:
	if not _valid_canvas(canvas_value) or not _valid_context(snapshot_value,scene_value,current_key_value,projection,viewport_value):
		return false
	var canvas: CanvasItem = canvas_value
	var plans: Array[Dictionary] = []
	for row in range(4):
		for column in range(6):
			var x0: float = float(column*4)
			var x1: float = float(column*4+4)
			var z0: float = float(row*4)
			var z1: float = float(row*4+4)
			var coordinates: Array = [[x0,z0],[x1,z0],[x1,z1],[x0,z1]]
			var points: PackedVector2Array = _project_polygon(projection,coordinates,0.0)
			if points.size() != 4:
				return false
			var variation: float = float((column+2*row)%3-1)*0.012
			var color: Color = Color(FLOOR_BASE.r+variation,FLOOR_BASE.g+variation,FLOOR_BASE.b+variation,1.0)
			if not _finite_color(color):
				return false
			plans.append(_draw_plan_polygon(points,color))
	for wall_id_value in WALL_CONTACT:
		var wall_id: String = str(wall_id_value)
		var contact_coordinates_value: Variant = WALL_CONTACT.get(wall_id)
		if not contact_coordinates_value is Array:
			return false
		var contact_coordinates: Array = contact_coordinates_value
		var contact_points: PackedVector2Array = _project_polygon(projection,contact_coordinates,0.0)
		if contact_points.size() != 4:
			return false
		plans.append(_draw_plan_polygon(contact_points,FLOOR_CONTACT))
	var seam_width: float = 0.75*(float(projection.get('ppm'))/48.0)
	for seam_x in [4.0,8.0,12.0,16.0,20.0]:
		var vertical_points: PackedVector2Array = _project_polygon(projection,[[seam_x,0.0],[seam_x,16.0]],0.0)
		if vertical_points.size() != 2:
			return false
		plans.append(_draw_plan_line(vertical_points,FLOOR_SEAM,seam_width))
	for seam_z in [4.0,8.0,12.0]:
		var horizontal_points: PackedVector2Array = _project_polygon(projection,[[0.0,seam_z],[24.0,seam_z]],0.0)
		if horizontal_points.size() != 2:
			return false
		plans.append(_draw_plan_line(horizontal_points,FLOOR_SEAM,seam_width))
	for key_value in FURNACES:
		var key: String = str(key_value)
		if not _texture_ready(furnace_visual_value,key):
			continue
		var foot_value: Variant = FURNACES.get(key)
		if not foot_value is Array or foot_value.size() != 3:
			return false
		var foot: Vector3 = Vector3(float(foot_value[0]),float(foot_value[1]),float(foot_value[2]))
		if not _finite_vector3(foot) or not _append_furnace_ground(plans,projection,foot):
			return false
	for plan in plans:
		if not _plan_valid(plan):
			return false
	for plan in plans:
		_emit_plan(canvas,plan)
	return true

func draw_wall(canvas_value, snapshot_value, scene_value, current_key_value, projection, wall_value, viewport_value) -> bool:
	if not _valid_canvas(canvas_value) or not _valid_context(snapshot_value,scene_value,current_key_value,projection,viewport_value) or not wall_value is Dictionary:
		return false
	var wall: Dictionary = wall_value
	var id_value: Variant = wall.get('id')
	if typeof(id_value) != TYPE_STRING or not WALLS.has(id_value):
		return false
	var expected_polygon_value: Variant = WALLS.get(id_value)
	var inner_value: Variant = WALL_INNER.get(id_value)
	if not expected_polygon_value is Array or not inner_value is Array:
		return false
	var expected_polygon: Array = expected_polygon_value
	if not _polygon_matches(wall.get('polygon'),expected_polygon):
		return false
	var inner: Array = inner_value
	if inner.size() != 2:
		return false
	var high_wall: bool = id_value == 'cw_boiler_chamber_wall_north' or id_value == 'cw_boiler_chamber_wall_west_north' or id_value == 'cw_boiler_chamber_wall_west_south'
	var wall_height: float = 0.75 if high_wall else 0.22
	var first_value: Variant = inner[0]
	var second_value: Variant = inner[1]
	if not first_value is Array or not second_value is Array or first_value.size() != 2 or second_value.size() != 2:
		return false
	var first: Array = first_value
	var second: Array = second_value
	var first_x: float = float(first[0])
	var first_z: float = float(first[1])
	var second_x: float = float(second[0])
	var second_z: float = float(second[1])
	var ground_a: Vector2 = _project_point(projection,first_x,0.0,first_z)
	var ground_b: Vector2 = _project_point(projection,second_x,0.0,second_z)
	var top_a: Vector2 = _project_point(projection,first_x,wall_height,first_z)
	var top_b: Vector2 = _project_point(projection,second_x,wall_height,second_z)
	var side_points: PackedVector2Array = PackedVector2Array([ground_a,ground_b,top_b,top_a])
	if id_value == 'cw_boiler_chamber_wall_east_south' or id_value == 'cw_boiler_chamber_wall_west_south':
		# Clip only the lower corner hidden by the existing 0.22m south cap.
		var south_height: float = 0.22
		var clipped_ground: Vector2 = _project_point(projection,second_x,0.0,second_z-south_height)
		var cap_intersection: Vector2 = _project_point(projection,second_x,south_height,second_z)
		if not _finite_vector2(clipped_ground) or not _finite_vector2(cap_intersection):
			return false
		side_points = PackedVector2Array([ground_a,clipped_ground,cap_intersection])
		if high_wall:
			# West is taller than the south cap: retain its visible upper end.
			side_points.append(top_b)
		side_points.append(top_a)
	var cap_points: PackedVector2Array = _project_polygon(projection,expected_polygon_value,wall_height)
	if not _finite_vector2(ground_a) or not _finite_vector2(ground_b) or not _finite_vector2(top_a) or not _finite_vector2(top_b) or cap_points.size() != 4 or not _finite_color(WALL_SIDE) or not _finite_color(WALL_CAP) or not _finite_color(FLOOR_CONTACT):
		return false
	var outer_face_enabled: bool = false
	var outer_points: PackedVector2Array = PackedVector2Array()
	if OUTER_WALL_EDGES.has(id_value):
		outer_face_enabled = _outer_face_context_matches(canvas_value,snapshot_value,scene_value,current_key_value,projection,viewport_value)
		if outer_face_enabled:
			outer_points = _outer_face_points(projection,id_value,wall_height)
			if outer_points.size() != 4 or not _points_have_area(side_points) or not _points_have_area(outer_points) or not _points_have_area(cap_points):
				return false
	var canvas: CanvasItem = canvas_value
	var short_family: Dictionary = _short_stage(canvas_value,snapshot_value,scene_value,current_key_value,projection,wall,viewport_value,wall_height)
	canvas.draw_colored_polygon(side_points,WALL_SIDE)
	if outer_face_enabled:
		canvas.draw_colored_polygon(outer_points,WALL_SIDE)
	_short_emit(canvas_value,short_family,wall,snapshot_value,scene_value,current_key_value,projection,viewport_value)
	canvas.draw_colored_polygon(cap_points,WALL_CAP)
	return true

func _outer_face_context_matches(canvas: CanvasItem, snapshot_value, scene_value, current_key_value, projection, viewport_value) -> bool:
	if not P.snapshot(snapshot_value) or not snapshot_value is Dictionary or not scene_value is Dictionary or typeof(current_key_value) != TYPE_STRING or not viewport_value is Rect2:
		return false
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	var viewport: Rect2 = viewport_value
	if snapshot.get('entryToken') != null:
		return false
	var canvas_snapshot: Variant = canvas.get('snapshot')
	var canvas_scene: Variant = canvas.get('scene')
	var canvas_key: Variant = canvas.get('current_key')
	var canvas_camera: Variant = canvas.get('camera')
	var canvas_viewport: Variant = canvas.get('viewport_rect')
	var projection_viewport: Variant = projection.get('viewport')
	var initialized: Variant = projection.get('initialized')
	return canvas_snapshot is Dictionary and canvas_snapshot == snapshot and canvas_scene is Dictionary and canvas_scene == scene and typeof(canvas_key) == TYPE_STRING and canvas_key == current_key_value and canvas_camera == projection and canvas_viewport is Rect2 and canvas_viewport == viewport and projection_viewport is Vector2 and projection_viewport == viewport.size and typeof(initialized) == TYPE_BOOL and initialized == true

func _outer_face_points(projection, wall_id: String, wall_height: float) -> PackedVector2Array:
	var endpoints_value: Variant = OUTER_WALL_EDGES.get(wall_id)
	if not endpoints_value is Array or endpoints_value.size() != 2:
		return PackedVector2Array()
	var first_value: Variant = endpoints_value[0]
	var second_value: Variant = endpoints_value[1]
	if not first_value is Array or first_value.size() != 2 or not second_value is Array or second_value.size() != 2 or not P.number(first_value[0],0.0,24.0) or not P.number(first_value[1],0.0,16.0) or not P.number(second_value[0],0.0,24.0) or not P.number(second_value[1],0.0,16.0):
		return PackedVector2Array()
	var a: Vector2 = _project_point(projection,float(first_value[0]),0.0,float(first_value[1]))
	var b: Vector2 = _project_point(projection,float(second_value[0]),0.0,float(second_value[1]))
	var top_b: Vector2 = _project_point(projection,float(second_value[0]),wall_height,float(second_value[1]))
	var top_a: Vector2 = _project_point(projection,float(first_value[0]),wall_height,float(first_value[1]))
	return PackedVector2Array([a,b,top_b,top_a])

func _points_have_area(points: PackedVector2Array) -> bool:
	if points.size() < 3:
		return false
	var twice_area: float = 0.0
	for index in range(points.size()):
		var a: Vector2 = points[index]
		var b: Vector2 = points[(index+1)%points.size()]
		if not _finite_vector2(a) or not _finite_vector2(b):
			return false
		twice_area += a.x*b.y-b.x*a.y
	return is_finite(twice_area) and twice_area != 0.0

# Candidate region-centered +Y steam calibration; not an authored nozzle pose.
# No textures, local clocks, exposure-derived density or gameplay state writes.
const SteamPresentation = preload('res://scripts/world/presentation.gd')
const STEAM_REGION = [[11.0,10.0],[13.0,10.0],[13.0,13.0],[11.0,13.0]]

func _pressure_steam_frame(snapshot: Dictionary, scene: Dictionary) -> Dictionary:
	var occluders: Variant = scene.get('occluders')
	var sources: Variant = scene.get('hazards')
	if not occluders is Array or not occluders.is_empty() or not P.rows(sources,64):
		return {}
	var source_count: int = 0
	for row_value: Variant in sources:
		if not row_value is Dictionary:
			return {}
		var row: Dictionary = row_value
		if row.get('id')!='cw_boiler_vent_staged' and row.get('kind')!='steam_jet':
			continue
		if row.size()!=4 or row.get('id')!='cw_boiler_vent_staged' or row.get('kind')!='steam_jet' or not _polygon_matches(row.get('polygon'),STEAM_REGION) or row.get('heightRangeM')!=null:
			return {}
		var config: Variant = row.get('environment')
		if not config is Dictionary or config.size()!=6 or config.get('tag')!='pressure':
			return {}
		for key: String in ['warningMs','activeMs','recoveryMs','damage','damageIntervalMs']:
			var expected: int = {'warningMs':900,'activeMs':700,'recoveryMs':3400,'damage':10,'damageIntervalMs':700}[key]
			if not P.counter(config.get(key),expected,expected):
				return {}
		source_count += 1
	if source_count!=1:
		return {}
	var native_count: int = 0
	var frame: Dictionary = {}
	for row_value: Variant in snapshot.hazards:
		if not row_value is Dictionary:
			return {}
		var row: Dictionary = row_value
		if row.get('entityId')!='cw_boiler_vent_staged' and row.get('kind')!='steam_jet':
			continue
		if row.get('entityId')!='cw_boiler_vent_staged' or row.get('kind')!='steam_jet' or not _polygon_matches(row.get('polygonM'),STEAM_REGION) or row.get('heightRangeM')!=null or not P.transform(row.get('transform')):
			return {}
		if P.vector(row.transform.positionM)!=Vector3(12,0,11.5) or not _number_matches(row.transform.get('yawRad'),0.0) or typeof(row.get('active'))!=TYPE_BOOL or typeof(row.get('phaseActive'))!=TYPE_BOOL:
			return {}
		var environment: Dictionary = SteamPresentation.environment(row)
		if environment.is_empty():
			return {}
		var maximum: int = {'warning':900,'active':700,'recovery':3400,'suppressed':6000}[environment.phase]
		if not P.counter(environment.remainingMs,1,maximum):
			return {}
		if environment.phase=='warning':
			if not P.counter(row.get('warningRemainingMs'),environment.remainingMs,environment.remainingMs):
				return {}
		elif row.get('warningRemainingMs')!=null:
			return {}
		native_count += 1
		frame = environment
	return frame if native_count==1 else {}

func _pressure_steam_polygon_valid(points: PackedVector2Array) -> bool:
	if points.size()!=24:
		return false
	for point: Vector2 in points:
		if not _finite_vector2(point):
			return false
	var signed_area: float = 0.0
	var winding: float = 0.0
	for index: int in range(24):
		var a: Vector2 = points[index]-points[0]
		var b: Vector2 = points[(index+1)%24]-points[0]
		var term: float = a.cross(b)
		var turn: float = (points[(index+1)%24]-points[index]).cross(points[(index+2)%24]-points[(index+1)%24])
		if not is_finite(term) or not is_finite(turn) or turn==0.0:
			return false
		if index==0:
			winding = turn
		elif (turn>0.0)!=(winding>0.0):
			return false
		signed_area += term
	return is_finite(signed_area) and signed_area != 0.0

# Convex polygon/viewport SAT. -1 invalid arithmetic, 0 offscreen, 1 positive overlap.
func _pressure_steam_visibility(points: PackedVector2Array, viewport: Rect2) -> int:
	var corners: PackedVector2Array = PackedVector2Array([viewport.position,Vector2(viewport.end.x,viewport.position.y),viewport.end,Vector2(viewport.position.x,viewport.end.y)])
	var axes: Array[Vector2] = [Vector2.RIGHT,Vector2.DOWN]
	for index: int in range(24):
		var edge: Vector2 = points[(index+1)%24]-points[index]
		axes.append(Vector2(-edge.y,edge.x))
	var separated: bool = false
	for axis: Vector2 in axes:
		if not _finite_vector2(axis) or axis==Vector2.ZERO:
			return -1
		var polygon_min: float = INF
		var polygon_max: float = -INF
		var view_min: float = INF
		var view_max: float = -INF
		for point: Vector2 in points:
			var distance: float = (point-points[0]).dot(axis)
			if not is_finite(distance):
				return -1
			polygon_min = minf(polygon_min,distance)
			polygon_max = maxf(polygon_max,distance)
		for point: Vector2 in corners:
			var distance: float = (point-points[0]).dot(axis)
			if not is_finite(distance):
				return -1
			view_min = minf(view_min,distance)
			view_max = maxf(view_max,distance)
		if polygon_max<=view_min or view_max<=polygon_min:
			separated = true
	return 0 if separated else 1

func _pressure_steam_canvas_matches(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, projection_value: Variant, viewport_value: Variant, reduced_motion_value: Variant) -> bool:
	if not _valid_canvas(canvas_value) or not viewport_value is Rect2 or typeof(reduced_motion_value)!=TYPE_BOOL:
		return false
	var canvas: CanvasItem = canvas_value
	var viewport: Rect2 = viewport_value
	var stored_viewport: Variant = canvas.get('viewport_rect')
	var stored_reduced_motion: Variant = canvas.get('reduced_motion')
	if not canvas.is_visible_in_tree() or not _finite_rect(viewport) or viewport.position!=Vector2.ZERO or not stored_viewport is Rect2 or typeof(stored_reduced_motion)!=TYPE_BOOL or stored_reduced_motion!=reduced_motion_value:
		return false
	var actual_viewport: Rect2 = canvas.get_viewport_rect()
	if not _finite_rect(actual_viewport) or actual_viewport!=viewport or stored_viewport!=viewport:
		return false
	return _outer_face_context_matches(canvas,snapshot_value,scene_value,current_key_value,projection_value,viewport_value)

func draw_pressure_steam(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, projection_value: Variant, viewport_value: Variant, reduced_motion_value: Variant) -> bool:
	if typeof(reduced_motion_value)!=TYPE_BOOL or not _valid_canvas(canvas_value) or not _valid_context(snapshot_value,scene_value,current_key_value,projection_value,viewport_value):
		return false
	if typeof(projection_value.get('initialized'))!=TYPE_BOOL or not projection_value.get('initialized') or snapshot_value.get('entryToken')!=null or snapshot_value.player.currentHp<=0:
		return false
	if not _pressure_steam_canvas_matches(canvas_value,snapshot_value,scene_value,current_key_value,projection_value,viewport_value,reduced_motion_value):
		return false
	var frame: Dictionary = _pressure_steam_frame(snapshot_value,scene_value)
	if frame.is_empty() or frame.phase!='active':
		return false
	# Native remaining time is the sole burst phase; reduced motion fixes one pose.
	var progress: float = 0.5 if reduced_motion_value else (700.0-float(frame.remainingMs))/700.0
	var radius_x: float = 0.20+0.08*progress
	var radius_z: float = 0.12+0.04*progress
	var steam_color: Color = Color('#d5d8d8')
	steam_color.a = 0.07*(1.0-0.6*progress)
	if not _finite_color(steam_color) or steam_color.a<=0.0 or steam_color.a>0.07:
		return false
	var plans: Array[PackedVector2Array] = []
	var any_visible: bool = false
	for layer: int in range(3):
		var height: float = 0.10+0.12*float(layer)+0.60*progress
		var points: PackedVector2Array = PackedVector2Array()
		for index: int in range(24):
			var angle: float = TAU*float(index)/24.0
			points.append(_project_point(projection_value,12.0+radius_x*cos(angle),height,11.5+radius_z*sin(angle)))
		if not _pressure_steam_polygon_valid(points):
			return false
		var visibility: int = _pressure_steam_visibility(points,viewport_value)
		if visibility<0:
			return false
		any_visible = any_visible or visibility==1
		plans.append(points)
	if not any_visible:
		return false
	if not _pressure_steam_canvas_matches(canvas_value,snapshot_value,scene_value,current_key_value,projection_value,viewport_value,reduced_motion_value):
		return false
	# Complete 72-point plan validated before the first of exactly three commands.
	var canvas: CanvasItem = canvas_value
	for points: PackedVector2Array in plans:
		canvas.draw_colored_polygon(points,steam_color)
	return true

func _short_equal(actual: Variant, expected: Variant) -> bool:
	if typeof(expected) in [TYPE_INT,TYPE_FLOAT]:
		return P.number(actual,float(expected),float(expected))
	if expected is Dictionary:
		if not actual is Dictionary or actual.size()!=expected.size():
			return false
		for key: Variant in expected:
			if not actual.has(key) or not _short_equal(actual[key],expected[key]):
				return false
		return true
	if expected is Array:
		if not actual is Array or actual.size()!=expected.size():
			return false
		for index: int in range(expected.size()):
			if not _short_equal(actual[index],expected[index]):
				return false
		return true
	return typeof(actual)==typeof(expected) and actual==expected

func _short_canonical(scene_value: Variant) -> bool:
	return scene_value is Dictionary and _short_equal(scene_value,SHORT_CANONICAL_SCENE)

func _short_bindings(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, projection_value: Variant, viewport_value: Variant) -> bool:
	if not _valid_canvas(canvas_value) or not canvas_value.is_visible_in_tree() or not P.snapshot(snapshot_value) or not scene_value is Dictionary:
		return false
	if not _short_canonical(scene_value) or snapshot_value.get('worldId')!='clockworks' or snapshot_value.get('sceneId')!='cw_boiler_chamber' or snapshot_value.get('entryToken')!=null:
		return false
	var epoch: Variant = snapshot_value.get('worldEpoch')
	if not P.counter(epoch,1) or typeof(key_value)!=TYPE_STRING or key_value!='clockworks:cw_boiler_chamber:'+str(epoch):
		return false
	if not _valid_projection(projection_value) or typeof(projection_value.get('initialized'))!=TYPE_BOOL or not projection_value.get('initialized'):
		return false
	if not viewport_value is Rect2:
		return false
	var viewport: Rect2 = viewport_value
	if not _finite_rect(viewport) or viewport.position!=Vector2.ZERO:
		return false
	var canvas_snapshot: Variant = canvas_value.get('snapshot')
	var canvas_scene: Variant = canvas_value.get('scene')
	var canvas_key: Variant = canvas_value.get('current_key')
	var canvas_hash: Variant = canvas_value.get('scene_hash')
	var canvas_camera: Variant = canvas_value.get('camera')
	var stored_viewport: Variant = canvas_value.get('viewport_rect')
	var actual_viewport: Variant = canvas_value.get_viewport_rect()
	var projection_viewport: Variant = projection_value.get('viewport')
	return canvas_snapshot is Dictionary and _short_equal(canvas_snapshot,snapshot_value) and canvas_scene is Dictionary and _short_equal(canvas_scene,scene_value) and canvas_key==key_value and canvas_hash==SHORT_SCENE_SHA and canvas_camera==projection_value and stored_viewport is Rect2 and stored_viewport==viewport and actual_viewport is Rect2 and actual_viewport==viewport and projection_viewport is Vector2 and projection_viewport==viewport.size and viewport.size.x>0.0 and viewport.size.y>0.0

func _short_capture(canvas_value: Variant, snapshot_value: Dictionary, key_value: String, projection_value: RefCounted) -> Array:
	return [snapshot_value.duplicate(true),key_value,canvas_value.get('scene_hash'),canvas_value,projection_value,projection_value.get('origin'),projection_value.get('screen_center'),projection_value.get('viewport'),projection_value.get('ppm')]

func _short_capture_matches(stamp_value: Variant, canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, projection_value: Variant, viewport_value: Variant) -> bool:
	if not stamp_value is Array or stamp_value.size()!=9 or not _short_bindings(canvas_value,snapshot_value,scene_value,key_value,projection_value,viewport_value):
		return false
	var stamp: Array = stamp_value
	return stamp[0] is Dictionary and _short_equal(stamp[0],snapshot_value) and stamp[1]==key_value and stamp[2]==SHORT_SCENE_SHA and stamp[3]==canvas_value and stamp[4]==projection_value and stamp[5]==projection_value.get('origin') and stamp[6]==projection_value.get('screen_center') and stamp[7]==projection_value.get('viewport') and stamp[8]==projection_value.get('ppm')

func _short_world(row_value: Variant) -> Array:
	if not row_value is Dictionary:
		return []
	var a: Variant = row_value.get('a')
	var b: Variant = row_value.get('b')
	var low: Variant = row_value.get('low')
	var high: Variant = row_value.get('high')
	if not a is Array or a.size()!=2 or not b is Array or b.size()!=2 or not P.number(a[0],0.0,24.0) or not P.number(a[1],0.0,16.0) or not P.number(b[0],0.0,24.0) or not P.number(b[1],0.0,16.0) or not P.number(low,0.0,1.0) or not P.number(high,0.0,1.0) or float(high)<=float(low):
		return []
	return [Vector3(float(a[0]),float(low),float(a[1])),Vector3(float(b[0]),float(low),float(b[1])),Vector3(float(b[0]),float(high),float(b[1])),Vector3(float(a[0]),float(high),float(a[1]))]

func _short_quad(points: PackedVector2Array, color: Color) -> bool:
	if points.size()!=4 or not _finite_color(color):
		return false
	var bounds: Rect2 = Rect2(points[0],Vector2.ZERO)
	for point: Vector2 in points:
		if not _finite_vector2(point):
			return false
		bounds = bounds.expand(point)
	if not _finite_rect(bounds):
		return false
	var origin: Vector2 = points[0]
	var area2: float = 0.0
	for index: int in range(1,points.size()-1):
		var triangle: float = (points[index]-origin).cross(points[index+1]-origin)
		if not is_finite(triangle):
			return false
		area2 += triangle
	if not is_finite(area2) or area2==0.0:
		return false
	var positive: bool = false
	for index: int in range(points.size()):
		var edge: Vector2 = points[(index+1)%points.size()]-points[index]
		var next_edge: Vector2 = points[(index+2)%points.size()]-points[(index+1)%points.size()]
		var turn: float = edge.cross(next_edge)
		if not _finite_vector2(edge) or edge==Vector2.ZERO or not is_finite(turn) or turn==0.0:
			return false
		if index==0:
			positive = turn>0.0
		elif (turn>0.0)!=positive:
			return false
	return (area2>0.0)==positive

func _short_visible(points: PackedVector2Array, viewport: Rect2) -> int:
	if points.size()!=4 or not _finite_rect(viewport):
		return -1
	var axes: Array[Vector2] = [Vector2.RIGHT,Vector2.DOWN]
	for index: int in range(points.size()):
		var edge: Vector2 = points[(index+1)%points.size()]-points[index]
		axes.append(Vector2(-edge.y,edge.x))
	var corners: PackedVector2Array = PackedVector2Array([viewport.position,Vector2(viewport.end.x,viewport.position.y),viewport.end,Vector2(viewport.position.x,viewport.end.y)])
	var origin: Vector2 = points[0]
	var separated: bool = false
	for axis: Vector2 in axes:
		if not _finite_vector2(axis) or axis==Vector2.ZERO:
			return -1
		var polygon_min: float = INF
		var polygon_max: float = -INF
		var rect_min: float = INF
		var rect_max: float = -INF
		for point: Vector2 in points:
			var projected: float = (point-origin).dot(axis)
			if not is_finite(projected):
				return -1
			polygon_min = minf(polygon_min,projected)
			polygon_max = maxf(polygon_max,projected)
		for corner: Vector2 in corners:
			var projected: float = (corner-origin).dot(axis)
			if not is_finite(projected):
				return -1
			rect_min = minf(rect_min,projected)
			rect_max = maxf(rect_max,projected)
		if not is_finite(polygon_min) or not is_finite(polygon_max) or not is_finite(rect_min) or not is_finite(rect_max) or polygon_min>=polygon_max or rect_min>=rect_max:
			return -1
		if polygon_max<=rect_min or rect_max<=polygon_min:
			separated = true
	return 0 if separated else 1

func _short_stage(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, projection_value: Variant, wall_value: Variant, viewport_value: Variant, wall_height: float) -> Dictionary:
	if not _short_bindings(canvas_value,snapshot_value,scene_value,key_value,projection_value,viewport_value) or not wall_value is Dictionary:
		return {}
	var owner_value: Variant = wall_value.get('id')
	if typeof(owner_value)!=TYPE_STRING or not WALLS.has(owner_value) or not _short_equal(wall_value,{'id':owner_value,'polygon':WALLS[owner_value]}):
		return {}
	var owner_found: bool = false
	for face: Dictionary in SHORT_FACES:
		if face.get('owner')==owner_value and P.number(face.get('high'),wall_height,wall_height):
			owner_found = true
	if not owner_found:
		return {}
	var snapshot: Dictionary = snapshot_value
	var key: String = key_value
	var projection: RefCounted = projection_value
	var viewport: Rect2 = viewport_value
	var stamp: Array = _short_capture(canvas_value,snapshot,key,projection)
	var all: Array[Dictionary] = []
	for face: Dictionary in SHORT_FACES:
		var world: Array = _short_world(face)
		if world.size()!=4:
			return {}
		var points: PackedVector2Array = PackedVector2Array()
		for world_value: Variant in world:
			if not world_value is Vector3 or not _finite_vector3(world_value):
				return {}
			var projected_value: Variant = projection.call('project',world_value)
			if not projected_value is Vector2 or not _finite_vector2(projected_value):
				return {}
			points.append(projected_value)
		if not _short_quad(points,WALL_SIDE):
			return {}
		var visibility: int = _short_visible(points,viewport)
		if visibility<0:
			return {}
		all.append({'owner':face.owner,'points':points,'color':WALL_SIDE,'visible':visibility==1})
	if all.size()!=5 or not _short_bindings(canvas_value,snapshot_value,scene_value,key_value,projection_value,viewport_value) or not _short_capture_matches(stamp,canvas_value,snapshot_value,scene_value,key_value,projection_value,viewport_value):
		return {}
	return {'all':all,'stamp':stamp}

func _short_emit(canvas_value: Variant, family_value: Variant, wall_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, projection_value: Variant, viewport_value: Variant) -> void:
	if not family_value is Dictionary or not wall_value is Dictionary:
		return
	var owner_value: Variant = wall_value.get('id')
	if typeof(owner_value)!=TYPE_STRING or not WALLS.has(owner_value) or not _short_equal(wall_value,{'id':owner_value,'polygon':WALLS[owner_value]}):
		return
	var all_value: Variant = family_value.get('all')
	var stamp: Variant = family_value.get('stamp')
	if not all_value is Array or all_value.size()!=5 or not viewport_value is Rect2:
		return
	var viewport: Rect2 = viewport_value
	var chosen: Dictionary = {}
	var chosen_count: int = 0
	for command_value: Variant in all_value:
		if not command_value is Dictionary:
			return
		var command: Dictionary = command_value
		var points_value: Variant = command.get('points')
		var color_value: Variant = command.get('color')
		var visible_value: Variant = command.get('visible')
		if not points_value is PackedVector2Array or not color_value is Color or typeof(visible_value)!=TYPE_BOOL:
			return
		var points: PackedVector2Array = points_value
		var color: Color = color_value
		if not _short_quad(points,color) or not _short_equal(color,WALL_SIDE):
			return
		var visibility: int = _short_visible(points,viewport)
		if visibility<0 or (visibility==1)!=visible_value:
			return
		if command.get('owner')==owner_value:
			chosen = command
			chosen_count += 1
	if chosen_count!=1 or not chosen.get('visible'):
		return
	if not _short_bindings(canvas_value,snapshot_value,scene_value,key_value,projection_value,viewport_value) or not _short_capture_matches(stamp,canvas_value,snapshot_value,scene_value,key_value,projection_value,viewport_value):
		return
	var canvas: CanvasItem = canvas_value
	var final_points: PackedVector2Array = chosen.get('points')
	var final_color: Color = chosen.get('color')
	canvas.draw_colored_polygon(final_points,final_color)
