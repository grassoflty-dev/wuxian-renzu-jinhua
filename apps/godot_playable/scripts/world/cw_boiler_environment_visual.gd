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
	var canvas: CanvasItem = canvas_value
	canvas.draw_colored_polygon(side_points,WALL_SIDE)
	canvas.draw_colored_polygon(cap_points,WALL_CAP)
	return true
