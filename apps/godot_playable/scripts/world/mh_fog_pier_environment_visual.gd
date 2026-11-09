extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')

const WORLD_ID: String = 'mist_harbor'
const SCENE_ID: String = 'mh_fog_pier'
const WALKABLE_ID: String = 'mh_fog_pier_walkable_area'
const STONE_ID: String = 'approved_stone_water_module'
const STONE_ASSET: String = 'runtime2d.world.mistharbor.water_edge_stone.v1'
const CAP_STONE_PATH: String = 'res://assets/props/mist_harbor/fog_pier_stone_source.png'
var cap_stone_attempted := false
var cap_stone: Texture2D
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
# FP_OUTER_DATA_BEGIN
const OUTER_SCENE_PATH: String = 'res://data/scenes/mh_fog_pier.json'
const OUTER_SCENE_BYTES: int = 7671
const OUTER_SCENE_SHA256: String = '1f12ceae6565dcfb8f3b9f497840f8027748e6109208e788ea6d7579fff00946'
const OUTER_EDGES: Array = [
	{'id': 'pier_wall_north', 'a': [24.0, 0.0], 'b': [24.0, 0.5]},
	{'id': 'pier_wall_south', 'a': [0.0, 16.0], 'b': [24.0, 16.0]},
	{'id': 'pier_wall_south', 'a': [24.0, 15.5], 'b': [24.0, 16.0]},
	{'id': 'pier_wall_west_north', 'a': [0.0, 6.0], 'b': [0.5, 6.0]},
	{'id': 'pier_wall_east_north', 'a': [24.0, 0.5], 'b': [24.0, 6.0]},
	{'id': 'pier_wall_east_north', 'a': [23.5, 6.0], 'b': [24.0, 6.0]},
	{'id': 'pier_wall_east_south', 'a': [24.0, 9.5], 'b': [24.0, 15.5]}
]
var _outer_scene_attempted: bool = false
var _outer_scene_cache: Dictionary = {}
# FP_OUTER_DATA_END

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

# FP_OUTER_FUNCTION_BEGIN
func _outer_load_canonical() -> bool:
	if _outer_scene_attempted:
		return not _outer_scene_cache.is_empty()
	_outer_scene_attempted = true
	var file: FileAccess = FileAccess.open(OUTER_SCENE_PATH, FileAccess.READ)
	if file == null or file.get_length() != OUTER_SCENE_BYTES:
		if file != null:
			file.close()
		return false
	var bytes: PackedByteArray = file.get_buffer(OUTER_SCENE_BYTES)
	file.close()
	if bytes.size() != OUTER_SCENE_BYTES:
		return false
	var hash_context: HashingContext = HashingContext.new()
	if hash_context.start(HashingContext.HASH_SHA256) != OK or hash_context.update(bytes) != OK:
		return false
	if hash_context.finish().hex_encode() != OUTER_SCENE_SHA256:
		return false
	var text: String = bytes.get_string_from_utf8()
	if text.to_utf8_buffer() != bytes:
		return false
	var decoded: Variant = JSON.parse_string(text)
	if not decoded is Dictionary:
		return false
	_outer_scene_cache = decoded
	return true
func _outer_same_json(value: Variant, expected: Variant, depth: int = 0) -> bool:
	if depth > 64:
		return false
	if typeof(expected) == TYPE_INT or typeof(expected) == TYPE_FLOAT:
		return _finite_number(value) and float(value) == float(expected)
	if typeof(value) != typeof(expected):
		return false
	if expected is Dictionary:
		if value.size() != expected.size():
			return false
		for key: Variant in expected:
			if not value.has(key) or not _outer_same_json(value[key], expected[key], depth + 1):
				return false
		return true
	if expected is Array:
		if value.size() != expected.size():
			return false
		for index: int in range(expected.size()):
			if not _outer_same_json(value[index], expected[index], depth + 1):
				return false
		return true
	return value == expected
func _outer_binding_matches(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, projection_value: Variant, viewport_value: Variant, snapshot_guard: Variant = null, scene_guard: Variant = null, camera_guard: Variant = null) -> bool:
	if not canvas_value is CanvasItem or not is_instance_valid(canvas_value) or not canvas_value.is_inside_tree() or not canvas_value.is_visible_in_tree() or canvas_value.is_queued_for_deletion():
		return false
	if typeof(snapshot_value) != TYPE_DICTIONARY or typeof(scene_value) != TYPE_DICTIONARY or typeof(key_value) != TYPE_STRING or not _finite_rect(viewport_value):
		return false
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	var viewport: Rect2 = viewport_value
	if viewport.position != Vector2.ZERO or snapshot.get('entryToken') != null or not _valid_context(snapshot, scene, key_value, projection_value, viewport):
		return false
	var canvas: CanvasItem = canvas_value
	if not _outer_same_json(canvas.get('snapshot'), snapshot) or not _outer_same_json(canvas.get('scene'), scene):
		return false
	if canvas.get('current_key') != key_value or canvas.get('scene_hash') != OUTER_SCENE_SHA256 or canvas.get('camera') != projection_value:
		return false
	var stored_viewport: Variant = canvas.get('viewport_rect')
	if typeof(stored_viewport) != TYPE_RECT2 or stored_viewport != viewport or canvas.get_viewport_rect() != viewport:
		return false
	if projection_value.get('viewport') != viewport.size:
		return false
	if snapshot_guard != null and not _outer_same_json(snapshot, snapshot_guard):
		return false
	if scene_guard != null and not _outer_same_json(scene, scene_guard):
		return false
	if not _outer_load_canonical() or not _outer_same_json(scene, _outer_scene_cache):
		return false
	if camera_guard != null:
		if projection_value.get('origin') != camera_guard.origin or projection_value.get('screen_center') != camera_guard.screen_center or projection_value.get('viewport') != camera_guard.viewport or projection_value.get('ppm') != camera_guard.ppm or projection_value.get('initialized') != camera_guard.initialized:
			return false
	return true
func _outer_polygon_valid(points: PackedVector2Array) -> bool:
	if points.size() != 4:
		return false
	for point: Vector2 in points:
		if not _finite_vector2(point):
			return false
	var area2: float = 0.0
	for index: int in range(1, 3):
		var term: float = (points[index] - points[0]).cross(points[index + 1] - points[0])
		if not is_finite(term):
			return false
		area2 += term
	if not is_finite(area2) or area2 == 0.0:
		return false
	var winding: float = 0.0
	var min_point: Vector2 = points[0]
	var max_point: Vector2 = points[0]
	for index: int in range(4):
		var edge: Vector2 = points[(index + 1) % 4] - points[index]
		var next_edge: Vector2 = points[(index + 2) % 4] - points[(index + 1) % 4]
		var turn: float = edge.cross(next_edge)
		if not _finite_vector2(edge) or not _finite_vector2(next_edge) or edge == Vector2.ZERO or not is_finite(turn) or turn == 0.0:
			return false
		if index == 0:
			winding = turn
		elif (turn > 0.0) != (winding > 0.0):
			return false
		min_point = Vector2(minf(min_point.x, points[index].x), minf(min_point.y, points[index].y))
		max_point = Vector2(maxf(max_point.x, points[index].x), maxf(max_point.y, points[index].y))
	var bounds_size: Vector2 = max_point - min_point
	return _finite_vector2(min_point) and _finite_vector2(max_point) and _finite_vector2(bounds_size) and bounds_size.x > 0.0 and bounds_size.y > 0.0
func _outer_visibility(points: PackedVector2Array, viewport: Rect2) -> int:
	var corners: PackedVector2Array = PackedVector2Array([viewport.position, Vector2(viewport.end.x, viewport.position.y), viewport.end, Vector2(viewport.position.x, viewport.end.y)])
	var axes: Array[Vector2] = [Vector2.RIGHT, Vector2.DOWN]
	for index: int in range(4):
		var edge: Vector2 = points[(index + 1) % 4] - points[index]
		axes.append(Vector2(-edge.y, edge.x))
	var separated: bool = false
	for axis: Vector2 in axes:
		if not _finite_vector2(axis) or axis == Vector2.ZERO:
			return -1
		var polygon_min: float = INF
		var polygon_max: float = -INF
		var viewport_min: float = INF
		var viewport_max: float = -INF
		for point: Vector2 in points:
			var projected: float = (point - points[0]).dot(axis)
			if not is_finite(projected):
				return -1
			polygon_min = minf(polygon_min, projected)
			polygon_max = maxf(polygon_max, projected)
		for point: Vector2 in corners:
			var projected: float = (point - points[0]).dot(axis)
			if not is_finite(projected):
				return -1
			viewport_min = minf(viewport_min, projected)
			viewport_max = maxf(viewport_max, projected)
		if not is_finite(polygon_min) or not is_finite(polygon_max) or not is_finite(viewport_min) or not is_finite(viewport_max) or polygon_min >= polygon_max or viewport_min >= viewport_max:
			return -1
		if polygon_max <= viewport_min or viewport_max <= polygon_min:
			separated = true
	return 0 if separated else 1
func _draw_outer_faces(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, projection_value: Variant, wall_data: Dictionary, viewport_value: Variant) -> void:
	if not _outer_binding_matches(canvas_value, snapshot_value, scene_value, key_value, projection_value, viewport_value):
		return
	var snapshot: Dictionary = snapshot_value
	var snapshot_guard: Dictionary = snapshot.duplicate(true)
	var scene_guard: Dictionary = scene_value.duplicate(true)
	var projection: RefCounted = projection_value
	var camera_guard: Dictionary = {'origin': projection.get('origin'), 'screen_center': projection.get('screen_center'), 'viewport': projection.get('viewport'), 'ppm': projection.get('ppm'), 'initialized': projection.get('initialized')}
	var staged: Array[Dictionary] = []
	for edge_value: Variant in OUTER_EDGES:
		if not edge_value is Dictionary:
			return
		var edge: Dictionary = edge_value
		var owner: Variant = edge.get('id')
		var a_value: Variant = edge.get('a')
		var b_value: Variant = edge.get('b')
		if typeof(owner) != TYPE_STRING or typeof(a_value) != TYPE_ARRAY or typeof(b_value) != TYPE_ARRAY or a_value.size() != 2 or b_value.size() != 2:
			return
		if not _finite_number(a_value[0]) or not _finite_number(a_value[1]) or not _finite_number(b_value[0]) or not _finite_number(b_value[1]):
			return
		var a: Vector3 = Vector3(float(a_value[0]), 0.0, float(a_value[1]))
		var b: Vector3 = Vector3(float(b_value[0]), 0.0, float(b_value[1]))
		var world: Array[Vector3] = [a, b, Vector3(b.x, RELIEF, b.z), Vector3(a.x, RELIEF, a.z)]
		var points: PackedVector2Array = _project_world_points(projection, world)
		if not _outer_polygon_valid(points):
			return
		var visibility: int = _outer_visibility(points, viewport_value)
		if visibility < 0:
			return
		staged.append({'id': owner, 'points': points, 'visible': visibility == 1})
	if staged.size() != OUTER_EDGES.size():
		return
	var emission: Array[PackedVector2Array] = []
	var wall_id: Variant = wall_data.get('id')
	for command: Dictionary in staged:
		if command.id == wall_id and command.visible:
			emission.append(command.points)
	if emission.is_empty() or not _outer_binding_matches(canvas_value, snapshot_value, scene_value, key_value, projection_value, viewport_value, snapshot_guard, scene_guard, camera_guard):
		return
	var drawing_canvas: CanvasItem = canvas_value
	for points: PackedVector2Array in emission:
		drawing_canvas.draw_colored_polygon(points, SIDE_COLOR)
# FP_OUTER_FUNCTION_END
func _cap_material_pieces(polygon: Array, projection) -> Array:
	if not cap_stone_attempted:
		cap_stone_attempted = true
		if ResourceLoader.exists(CAP_STONE_PATH):
			var loaded = load(CAP_STONE_PATH)
			if loaded is Texture2D:
				cap_stone = loaded
		if not is_instance_valid(cap_stone):
			var image = Image.new()
			if image.load(CAP_STONE_PATH)==OK and image.get_width()==1228 and image.get_height()==1254:
				cap_stone = ImageTexture.create_from_image(image)
	if not is_instance_valid(cap_stone) or cap_stone.get_width()!=1228 or cap_stone.get_height()!=1254 or polygon.size()!=4:
		return []
	var x0=float(polygon[0][0]); var z0=float(polygon[0][1])
	var x1=float(polygon[2][0]); var z1=float(polygon[2][1])
	if not is_finite(x0) or not is_finite(z0) or not is_finite(x1) or not is_finite(z1) or x0>=x1 or z0>=z1:
		return []
	var pieces: Array = []
	var x=x0
	while x<x1:
		var cx=floorf(x/4.0)*4.0
		var xe=minf(x1,cx+4.0)
		if xe<=x: return []
		var z=z0
		while z<z1:
			var cz=floorf(z/4.0)*4.0
			var ze=minf(z1,cz+4.0)
			if ze<=z: return []
			var points = _project_world_points(projection,[Vector3(x,RELIEF,z),Vector3(xe,RELIEF,z),Vector3(xe,RELIEF,ze),Vector3(x,RELIEF,ze)])
			if points.size()!=4: return []
			var area := 0.0
			for i in range(4):
				area += points[i].cross(points[(i+1)%4])
			if not is_finite(area) or absf(area)<=0.000001: return []
			var u0=(x-cx)/4.0; var u1=(xe-cx)/4.0; var v0=(z-cz)/4.0; var v1=(ze-cz)/4.0
			var uv:=PackedVector2Array()
			for f in [Vector2(u0,v0),Vector2(u1,v0),Vector2(u1,v1),Vector2(u0,v1)]:
				uv.append((Vector2(820,240)+Vector2(212,102)*f.x+Vector2(-640,285)*f.y)/Vector2(1228,1254))
			pieces.append({'points':points,'uv':uv})
			z=ze
		x=xe
	return pieces

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
	var cap_pieces = _cap_material_pieces(layout.polygon,projection)
	drawing_canvas.draw_colored_polygon(contact_points, CONTACT_COLOR)
	drawing_canvas.draw_colored_polygon(side_points, SIDE_COLOR)
	_draw_outer_faces(drawing_canvas, snapshot, scene, current_key, projection, wall_data, viewport) # FP_OUTER_CALL
	if cap_pieces.is_empty():
		drawing_canvas.draw_colored_polygon(cap_points, CAP_COLOR)
	else:
		for piece in cap_pieces:
			drawing_canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,cap_stone)
	drawing_canvas.draw_polyline(outline_points, OUTLINE_COLOR, outline_width, true)
	return true
