extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')

const WORLD_ID: String = 'mist_harbor'
const SCENE_ID: String = 'mh_fog_pier'
const WALKABLE_ID: String = 'mh_fog_pier_walkable_area'
const STONE_ID: String = 'approved_stone_water_module'
const STONE_ASSET: String = 'runtime2d.world.mistharbor.water_edge_stone.v1'
const RELIEF: float = 0.25
const CONTACT_INSET: float = 0.04
const SIDE_COLOR: Color = Color(38.0 / 255.0, 59.0 / 255.0, 70.0 / 255.0, 1.0)
const CAP_COLOR: Color = Color(60.0 / 255.0, 82.0 / 255.0, 92.0 / 255.0, 1.0)
const OUTLINE_COLOR: Color = Color(113.0 / 255.0, 132.0 / 255.0, 138.0 / 255.0, 0.50)
const CONTACT_COLOR: Color = Color(8.0 / 255.0, 21.0 / 255.0, 27.0 / 255.0, 0.14)

const WALL_LAYOUT: Array = [
	{
		'id': 'pier_wall_north',
		'polygon': [[0.0, 0.0], [24.0, 0.0], [24.0, 0.5], [0.0, 0.5]],
		'inner_line': [[0.0, 0.5], [24.0, 0.5]],
		'contact_polygon': [[0.0, 0.5], [24.0, 0.5], [24.0, 0.46], [0.0, 0.46]]
	},
	{
		'id': 'pier_wall_south',
		'polygon': [[0.0, 15.5], [24.0, 15.5], [24.0, 16.0], [0.0, 16.0]],
		'inner_line': [[0.0, 15.5], [24.0, 15.5]],
		'contact_polygon': [[0.0, 15.5], [24.0, 15.5], [24.0, 15.54], [0.0, 15.54]]
	},
	{
		'id': 'pier_wall_west_north',
		'polygon': [[0.0, 0.5], [0.5, 0.5], [0.5, 6.0], [0.0, 6.0]],
		'inner_line': [[0.5, 0.5], [0.5, 6.0]],
		'contact_polygon': [[0.5, 0.5], [0.5, 6.0], [0.46, 6.0], [0.46, 0.5]]
	},
	{
		'id': 'pier_wall_west_south',
		'polygon': [[0.0, 9.5], [0.5, 9.5], [0.5, 15.5], [0.0, 15.5]],
		'inner_line': [[0.5, 9.5], [0.5, 15.5]],
		'contact_polygon': [[0.5, 9.5], [0.5, 15.5], [0.46, 15.5], [0.46, 9.5]]
	},
	{
		'id': 'pier_wall_east_north',
		'polygon': [[23.5, 0.5], [24.0, 0.5], [24.0, 6.0], [23.5, 6.0]],
		'inner_line': [[23.5, 0.5], [23.5, 6.0]],
		'contact_polygon': [[23.5, 0.5], [23.5, 6.0], [23.54, 6.0], [23.54, 0.5]]
	},
	{
		'id': 'pier_wall_east_south',
		'polygon': [[23.5, 9.5], [24.0, 9.5], [24.0, 15.5], [23.5, 15.5]],
		'inner_line': [[23.5, 9.5], [23.5, 15.5]],
		'contact_polygon': [[23.5, 9.5], [23.5, 15.5], [23.54, 15.5], [23.54, 9.5]]
	}
]

func _finite_number(value: Variant) -> bool:
	var value_type: int = typeof(value)
	return (value_type == TYPE_INT or value_type == TYPE_FLOAT) and is_finite(float(value))

func _same_number(value: Variant, expected: float) -> bool:
	return _finite_number(value) and P.number(value, expected, expected)

func _finite_vector2(value: Variant) -> bool:
	return typeof(value) == TYPE_VECTOR2 and is_finite(value.x) and is_finite(value.y)

func _finite_vector3(value: Variant) -> bool:
	return typeof(value) == TYPE_VECTOR3 and is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _finite_rect(value: Variant) -> bool:
	if typeof(value) != TYPE_RECT2:
		return false
	var rect: Rect2 = value
	return _finite_vector2(rect.position) and _finite_vector2(rect.size) and rect.size.x > 0.0 and rect.size.y > 0.0 and _finite_vector2(rect.end)

func _same_point(value: Variant, expected: Variant) -> bool:
	if typeof(value) != TYPE_ARRAY or value.size() != 2 or typeof(expected) != TYPE_ARRAY or expected.size() != 2:
		return false
	return _same_number(value[0], float(expected[0])) and _same_number(value[1], float(expected[1]))

func _same_polygon(value: Variant, expected: Variant) -> bool:
	if typeof(value) != TYPE_ARRAY or typeof(expected) != TYPE_ARRAY or value.size() != expected.size():
		return false
	for index: int in range(expected.size()):
		if not _same_point(value[index], expected[index]):
			return false
	return true

func _layout_for_id(wall_id: Variant) -> Dictionary:
	if typeof(wall_id) != TYPE_STRING:
		return {}
	var checked_wall_id: String = wall_id
	for layout: Dictionary in WALL_LAYOUT:
		if layout.get('id') == checked_wall_id:
			return layout
	return {}

func _valid_bounds(scene: Dictionary) -> bool:
	var bounds_value: Variant = scene.get('boundsM')
	if typeof(bounds_value) != TYPE_DICTIONARY:
		return false
	var bounds: Dictionary = bounds_value
	return bounds.size() == 4 and _same_number(bounds.get('x'), 0.0) and _same_number(bounds.get('z'), 0.0) and _same_number(bounds.get('width'), 24.0) and _same_number(bounds.get('depth'), 16.0)

func _valid_walkable(scene: Dictionary) -> bool:
	var polygons_value: Variant = scene.get('walkablePolygons')
	if typeof(polygons_value) != TYPE_ARRAY:
		return false
	var polygons: Array = polygons_value
	if polygons.size() != 1 or typeof(polygons[0]) != TYPE_DICTIONARY:
		return false
	var row: Dictionary = polygons[0]
	if row.size() != 2 or row.get('id') != WALKABLE_ID:
		return false
	var polygon_value: Variant = row.get('polygon')
	if typeof(polygon_value) != TYPE_ARRAY:
		return false
	var expected: Array = [[0.5, 0.5], [23.5, 0.5], [23.5, 15.5], [0.5, 15.5]]
	return _same_polygon(polygon_value, expected)

func _valid_stone_row(scene: Dictionary) -> bool:
	var presentation_value: Variant = scene.get('presentation')
	if typeof(presentation_value) != TYPE_DICTIONARY:
		return false
	var presentation: Dictionary = presentation_value
	var sprites_value: Variant = presentation.get('sprites')
	if typeof(sprites_value) != TYPE_ARRAY:
		return false
	var sprites: Array = sprites_value
	if sprites.size() > 64:
		return false
	var seen_ids: Dictionary = {}
	var candidate_count: int = 0
	for sprite_value: Variant in sprites:
		if typeof(sprite_value) != TYPE_DICTIONARY:
			return false
		var sprite: Dictionary = sprite_value
		var sprite_id: Variant = sprite.get('id')
		var asset_id: Variant = sprite.get('assetId')
		var layer: Variant = sprite.get('layer')
		var position_value: Variant = sprite.get('position')
		if typeof(sprite_id) != TYPE_STRING or sprite_id.is_empty() or typeof(asset_id) != TYPE_STRING or typeof(layer) != TYPE_STRING:
			return false
		if seen_ids.has(sprite_id):
			return false
		seen_ids[sprite_id] = true
		if typeof(position_value) != TYPE_ARRAY or position_value.size() != 3:
			return false
		for coordinate: Variant in position_value:
			if not _finite_number(coordinate):
				return false
		if sprite_id == STONE_ID or asset_id == STONE_ASSET:
			candidate_count += 1
			if sprite.size() != 4 or sprite_id != STONE_ID or asset_id != STONE_ASSET or layer != 'visual.floor':
				return false
			if not _same_number(position_value[0], 12.0) or not _same_number(position_value[1], 0.0) or not _same_number(position_value[2], 8.0):
				return false
	return candidate_count == 1

func _valid_collisions(scene: Dictionary) -> bool:
	var collision_value: Variant = scene.get('collision')
	if typeof(collision_value) != TYPE_ARRAY:
		return false
	var collisions: Array = collision_value
	if collisions.size() != WALL_LAYOUT.size():
		return false
	var seen_ids: Dictionary = {}
	for wall_value: Variant in collisions:
		if typeof(wall_value) != TYPE_DICTIONARY:
			return false
		var wall: Dictionary = wall_value
		var wall_id: Variant = wall.get('id')
		if wall.size() != 2 or typeof(wall_id) != TYPE_STRING or seen_ids.has(wall_id):
			return false
		var layout: Dictionary = _layout_for_id(wall_id)
		if layout.is_empty() or not _same_polygon(wall.get('polygon'), layout.get('polygon')):
			return false
		seen_ids[wall_id] = true
	return seen_ids.size() == WALL_LAYOUT.size()

func _valid_projection(projection: Variant) -> bool:
	if not projection is RefCounted or projection.get_script() != ProjectionScript:
		return false
	var origin_value: Variant = projection.get('origin')
	var center_value: Variant = projection.get('screen_center')
	var projection_viewport_value: Variant = projection.get('viewport')
	if not _finite_vector3(origin_value) or not _finite_vector2(center_value) or not _finite_vector2(projection_viewport_value):
		return false
	if projection_viewport_value.x <= 0.0 or projection_viewport_value.y <= 0.0:
		return false
	if not P.number(projection.get('ppm'), 0.000001, 256.0) or typeof(projection.get('initialized')) != TYPE_BOOL or projection.get('initialized') != true:
		return false
	return true

func _valid_context(snapshot: Variant, scene: Variant, current_key: Variant, projection: Variant, viewport: Variant) -> bool:
	if typeof(snapshot) != TYPE_DICTIONARY or not P.snapshot(snapshot):
		return false
	var state: Dictionary = snapshot
	if state.get('worldId') != WORLD_ID or state.get('sceneId') != SCENE_ID or not P.counter(state.get('worldEpoch'), 1):
		return false
	if typeof(current_key) != TYPE_STRING or current_key != WORLD_ID + ':' + SCENE_ID + ':' + str(state.get('worldEpoch')):
		return false
	if typeof(scene) != TYPE_DICTIONARY:
		return false
	var scene_data: Dictionary = scene
	if not P.counter(scene_data.get('schemaVersion'), 1, 1) or scene_data.get('worldId') != WORLD_ID or scene_data.get('sceneId') != SCENE_ID:
		return false
	if not _valid_bounds(scene_data) or not _valid_walkable(scene_data) or not _valid_stone_row(scene_data) or not _valid_collisions(scene_data):
		return false
	if not _valid_projection(projection) or not _finite_rect(viewport):
		return false
	return true

func _lift_polygon(points: Variant, height: float) -> Array:
	if typeof(points) != TYPE_ARRAY:
		return []
	var lifted: Array = []
	for point_value: Variant in points:
		if typeof(point_value) != TYPE_ARRAY or point_value.size() != 2 or not _finite_number(point_value[0]) or not _finite_number(point_value[1]):
			return []
		var point: Array = point_value
		lifted.append(Vector3(float(point[0]), height, float(point[1])))
	return lifted


func _project_world_points(projection: Variant, points: Variant) -> PackedVector2Array:
	if not projection is RefCounted or projection.get_script() != ProjectionScript or typeof(points) != TYPE_ARRAY:
		return PackedVector2Array()
	var screen_points: PackedVector2Array = PackedVector2Array()
	for world_value: Variant in points:
		if not _finite_vector3(world_value):
			return PackedVector2Array()
		var screen_value: Variant = projection.call('project', world_value)
		if not _finite_vector2(screen_value):
			return PackedVector2Array()
		screen_points.append(screen_value)
	return screen_points

func draw_wall(canvas, snapshot, scene, current_key, projection, wall, viewport) -> bool:
	if not canvas is CanvasItem or not _valid_context(snapshot, scene, current_key, projection, viewport):
		return false
	if typeof(wall) != TYPE_DICTIONARY:
		return false
	var wall_data: Dictionary = wall
	var wall_id: Variant = wall_data.get('id')
	if typeof(wall_id) != TYPE_STRING or wall_data.size() != 2:
		return false
	var layout: Dictionary = _layout_for_id(wall_id)
	if layout.is_empty() or not _same_polygon(wall_data.get('polygon'), layout.get('polygon')):
		return false
	var inner_line_value: Variant = layout.get('inner_line')
	if typeof(inner_line_value) != TYPE_ARRAY or inner_line_value.size() != 2 or typeof(inner_line_value[0]) != TYPE_ARRAY or typeof(inner_line_value[1]) != TYPE_ARRAY:
		return false
	var inner_line: Array = inner_line_value
	var start: Array = inner_line[0]
	var finish: Array = inner_line[1]
	var side_world: Array = [
		Vector3(float(start[0]), 0.0, float(start[1])),
		Vector3(float(finish[0]), 0.0, float(finish[1])),
		Vector3(float(finish[0]), RELIEF, float(finish[1])),
		Vector3(float(start[0]), RELIEF, float(start[1]))
	]
	var contact_world: Array = _lift_polygon(layout.get('contact_polygon'), 0.0)
	var cap_world: Array = _lift_polygon(layout.get('polygon'), RELIEF)
	var contact_points: PackedVector2Array = _project_world_points(projection, contact_world)
	var side_points: PackedVector2Array = _project_world_points(projection, side_world)
	var cap_points: PackedVector2Array = _project_world_points(projection, cap_world)
	if contact_points.size() != 4 or side_points.size() != 4 or cap_points.size() != 4:
		return false
	var outline_points: PackedVector2Array = PackedVector2Array()
	for point: Vector2 in cap_points:
		outline_points.append(point)
	outline_points.append(cap_points[0])
	var ppm: float = float(projection.get('ppm'))
	var outline_width: float = ppm / 48.0
	if not is_finite(outline_width) or outline_width <= 0.0:
		return false
	var drawing_canvas: CanvasItem = canvas
	drawing_canvas.draw_colored_polygon(contact_points, CONTACT_COLOR)
	drawing_canvas.draw_colored_polygon(side_points, SIDE_COLOR)
	drawing_canvas.draw_colored_polygon(cap_points, CAP_COLOR)
	drawing_canvas.draw_polyline(outline_points, OUTLINE_COLOR, outline_width, true)
	return true
