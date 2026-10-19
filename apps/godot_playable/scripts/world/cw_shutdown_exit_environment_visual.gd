extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const SCENE_PATH = 'res://data/scenes/cw_shutdown_exit.json'
const SCENE_BYTES = 5877
const SCENE_SHA256 = '4e24942874fb43b0111eea4c4ac6035fd4ccd65e5b45b845ca4063f63ab2692b'
# Shutdown cutaway appearance, following the existing CW steel palette.
# These are display heights only, never collision or traversable support heights.
const HIGH_M = 1.0
const LOW_M = 0.18
const MIN_AREA2 = 0.000001
const SIDE_COLOR = Color(0.18,0.22,0.24,1.0)
const CAP_COLOR = Color(0.25,0.29,0.31,1.0)
const WALLS = [
	{'id':'cw_shutdown_wall_north','polygon':[[0,0],[24,0],[24,0.5],[0,0.5]]},
	{'id':'cw_shutdown_wall_south','polygon':[[0,15.5],[24,15.5],[24,16],[0,16]]},
	{'id':'cw_shutdown_wall_west_north','polygon':[[0,0.5],[0.5,0.5],[0.5,6],[0,6]]},
	{'id':'cw_shutdown_wall_west_south','polygon':[[0,9.5],[0.5,9.5],[0.5,15.5],[0,15.5]]},
	{'id':'cw_shutdown_wall_east','polygon':[[23.5,0.5],[24,0.5],[24,15.5],[23.5,15.5]]}
]
const CAP_HEIGHTS = [HIGH_M,LOW_M,HIGH_M,HIGH_M,LOW_M]
# Complete +x/+z exterior union, including both 0.82m high step faces.
# a/b are XZ endpoints; clip_low_end applies only to the two low-cap corners.
const SIDES = [
	{'a':[0.5,0.5],'b':[23.5,0.5],'low':0.0,'high':HIGH_M,'clip_low_end':true},
	{'a':[23.5,0.5],'b':[24,0.5],'low':LOW_M,'high':HIGH_M,'clip_low_end':false},
	{'a':[24,0],'b':[24,0.5],'low':0.0,'high':HIGH_M,'clip_low_end':false},
	{'a':[0,16],'b':[24,16],'low':0.0,'high':LOW_M,'clip_low_end':false},
	{'a':[24,15.5],'b':[24,16],'low':0.0,'high':LOW_M,'clip_low_end':false},
	{'a':[0.5,0.5],'b':[0.5,6],'low':0.0,'high':HIGH_M,'clip_low_end':false},
	{'a':[0,6],'b':[0.5,6],'low':0.0,'high':HIGH_M,'clip_low_end':false},
	{'a':[0.5,9.5],'b':[0.5,15.5],'low':0.0,'high':HIGH_M,'clip_low_end':true},
	{'a':[0,15.5],'b':[0.5,15.5],'low':LOW_M,'high':HIGH_M,'clip_low_end':false},
	{'a':[24,0.5],'b':[24,15.5],'low':0.0,'high':LOW_M,'clip_low_end':false}
]

var _scene_attempted: bool = false
var _canonical_scene: Dictionary = {}

func _same_json(value: Variant, expected: Variant, depth: int = 0) -> bool:
	if depth > 64:
		return false
	if typeof(expected) == TYPE_INT or typeof(expected) == TYPE_FLOAT:
		return P.number(value) and float(value) == float(expected)
	if typeof(value) != typeof(expected):
		return false
	if expected is Dictionary:
		if value.size() != expected.size():
			return false
		for key: Variant in expected:
			if not value.has(key) or not _same_json(value[key],expected[key],depth+1):
				return false
		return true
	if expected is Array:
		if value.size() != expected.size():
			return false
		for index: int in range(expected.size()):
			if not _same_json(value[index],expected[index],depth+1):
				return false
		return true
	return value == expected

# Renderer must require draw_shell success in the same frame before using this.
func owns_collision(wall_value: Variant) -> bool:
	if not wall_value is Dictionary or wall_value.size() != 2:
		return false
	for expected: Dictionary in WALLS:
		if _same_json(wall_value,expected):
			return true
	return false

func _load_scene_once() -> void:
	if _scene_attempted:
		return
	_scene_attempted = true
	var file: FileAccess = FileAccess.open(SCENE_PATH,FileAccess.READ)
	if file == null:
		return
	if file.get_length() != SCENE_BYTES:
		file.close()
		return
	var bytes: PackedByteArray = file.get_buffer(SCENE_BYTES)
	file.close()
	if bytes.size() != SCENE_BYTES:
		return
	var hash: HashingContext = HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256) != OK or hash.update(bytes) != OK or hash.finish().hex_encode() != SCENE_SHA256:
		return
	var source_text: String = bytes.get_string_from_utf8()
	if source_text.to_utf8_buffer() != bytes:
		return
	var decoded: Variant = JSON.parse_string(source_text)
	if decoded is Dictionary:
		_canonical_scene = decoded

func _context(snapshot: Dictionary, scene: Dictionary, key_value: Variant, hash_value: Variant) -> bool:
	if snapshot.get('worldId') != 'clockworks' or snapshot.get('sceneId') != 'cw_shutdown_exit' or snapshot.get('entryToken') != null:
		return false
	if not P.snapshot(snapshot) or hash_value != SCENE_SHA256:
		return false
	var epoch: Variant = snapshot.get('worldEpoch')
	if not P.counter(epoch,1) or typeof(key_value) != TYPE_STRING or key_value != 'clockworks:cw_shutdown_exit:'+str(epoch):
		return false
	_load_scene_once()
	if _canonical_scene.is_empty() or scene.size() != _canonical_scene.size():
		return false
	for field: Variant in _canonical_scene:
		if field != 'collision' and (not scene.has(field) or not _same_json(scene[field],_canonical_scene[field])):
			return false
	var rows: Variant = scene.get('collision')
	if not rows is Array or rows.size() != WALLS.size():
		return false
	var seen: Dictionary = {}
	for wall: Variant in rows:
		if not owns_collision(wall) or seen.has(wall.id):
			return false
		seen[wall.id] = true
	return true

func _finite_v2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _finite_v3(value: Vector3) -> bool:
	return is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _valid_rect(value: Rect2) -> bool:
	return _finite_v2(value.position) and _finite_v2(value.size) and _finite_v2(value.end) and value.size.x > 0.0 and value.size.y > 0.0

func _valid_camera(camera: Variant, viewport: Rect2) -> bool:
	if not camera is RefCounted or not is_instance_valid(camera) or camera.get_script() != ProjectionScript:
		return false
	if typeof(camera.initialized) != TYPE_BOOL or not camera.initialized or not camera.origin is Vector3 or not camera.screen_center is Vector2 or not camera.viewport is Vector2:
		return false
	return P.number(camera.ppm,0.0,256.0) and camera.ppm > 0.0 and _finite_v3(camera.origin) and _finite_v2(camera.screen_center) and _finite_v2(camera.viewport) and camera.viewport.x > 0.0 and camera.viewport.y > 0.0 and camera.viewport == viewport.size

func _project_world(points: Array[Vector3], camera: RefCounted) -> PackedVector2Array:
	var result: PackedVector2Array = PackedVector2Array()
	for point: Vector3 in points:
		if not _finite_v3(point):
			return PackedVector2Array()
		var projected: Vector2 = camera.project(point)
		if not _finite_v2(projected):
			return PackedVector2Array()
		result.append(projected)
	return result

func _plan(camera: RefCounted) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for face: Dictionary in SIDES:
		var a: Vector3 = Vector3(face.a[0],face.low,face.a[1])
		var b: Vector3 = Vector3(face.b[0],face.low,face.b[1])
		var top_a: Vector3 = Vector3(a.x,face.high,a.z)
		var top_b: Vector3 = Vector3(b.x,face.high,b.z)
		var original_world: Array[Vector3] = [a,b,top_b,top_a]
		var final_world: Array[Vector3] = original_world.duplicate()
		if face.clip_low_end:
			# The .18m adjacent cap hides only the lower terminal triangle.
			# Keep the endpoint at cap height and the entire upper face.
			var axis: Vector3 = Vector3.RIGHT if b.x > a.x else Vector3(0,0,1)
			final_world = [a,b-axis*LOW_M,Vector3(b.x,LOW_M,b.z),top_b,top_a]
		result.append({'original':_project_world(original_world,camera),'points':_project_world(final_world,camera),'color':SIDE_COLOR})
	for index: int in range(WALLS.size()):
		var cap_world: Array[Vector3] = []
		for point: Array in WALLS[index].polygon:
			cap_world.append(Vector3(point[0],CAP_HEIGHTS[index],point[1]))
		var cap_points: PackedVector2Array = _project_world(cap_world,camera)
		result.append({'original':cap_points,'points':cap_points,'color':CAP_COLOR})
	return result

# All fifteen original and final convex faces must remain fully renderable.
func _valid_polygon(points: PackedVector2Array) -> bool:
	if points.size() < 4 or points.size() > 5:
		return false
	for point: Vector2 in points:
		if not _finite_v2(point):
			return false
	var bounds: Rect2 = Rect2(points[0],Vector2.ZERO)
	for point: Vector2 in points:
		bounds = bounds.expand(point)
	if not _valid_rect(bounds):
		return false
	var area2: float = 0.0
	for index: int in range(1,points.size()-1):
		var triangle: float = (points[index]-points[0]).cross(points[index+1]-points[0])
		if not is_finite(triangle):
			return false
		area2 += triangle
	if not is_finite(area2) or absf(area2) <= MIN_AREA2:
		return false
	for index: int in range(points.size()):
		var edge: Vector2 = points[(index+1)%points.size()]-points[index]
		var next_edge: Vector2 = points[(index+2)%points.size()]-points[(index+1)%points.size()]
		var turn: float = edge.cross(next_edge)
		if not _finite_v2(edge) or edge == Vector2.ZERO or not is_finite(turn) or turn == 0.0 or (turn > 0.0) != (area2 > 0.0):
			return false
	return true

# Strict convex polygon/rectangle SAT: -1 invalid, 0 outside, 1 positive overlap.
# No clipping or generated fragments; at most seven axes per fixed face.
func _visible(points: PackedVector2Array, viewport: Rect2) -> int:
	var axes: Array[Vector2] = [Vector2.RIGHT,Vector2.DOWN]
	for index: int in range(points.size()):
		var edge: Vector2 = points[(index+1)%points.size()]-points[index]
		axes.append(Vector2(-edge.y,edge.x))
	var corners: PackedVector2Array = PackedVector2Array([viewport.position,Vector2(viewport.end.x,viewport.position.y),viewport.end,Vector2(viewport.position.x,viewport.end.y)])
	var origin: Vector2 = points[0]
	var separated: bool = false
	for axis: Vector2 in axes:
		if not _finite_v2(axis) or axis == Vector2.ZERO:
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
		if not is_finite(polygon_min) or not is_finite(polygon_max) or not is_finite(rect_min) or not is_finite(rect_max) or polygon_min >= polygon_max or rect_min >= rect_max:
			return -1
		if polygon_max <= rect_min or rect_max <= polygon_min:
			separated = true
	return 0 if separated else 1

func draw_shell(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, scene_hash_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not canvas_value is CanvasItem or not is_instance_valid(canvas_value) or not canvas_value.is_inside_tree() or not canvas_value.is_visible_in_tree() or canvas_value.is_queued_for_deletion():
		return false
	if not snapshot_value is Dictionary or not scene_value is Dictionary or not viewport_value is Rect2:
		return false
	var viewport: Rect2 = viewport_value
	if not _valid_rect(viewport) or not _valid_camera(camera_value,viewport) or not _context(snapshot_value,scene_value,key_value,scene_hash_value):
		return false
	var plan: Array[Dictionary] = _plan(camera_value)
	if plan.size() != 15:
		return false
	var drawable: Array[Dictionary] = []
	var final_vertex_count: int = 0
	for command: Dictionary in plan:
		var original: PackedVector2Array = command.original
		var points: PackedVector2Array = command.points
		if not _valid_polygon(original) or not _valid_polygon(points):
			return false
		final_vertex_count += points.size()
		var visible: int = _visible(points,viewport)
		if visible < 0:
			return false
		if visible > 0:
			drawable.append(command)
	if final_vertex_count != 62 or drawable.is_empty():
		return false
	# No command has been emitted until the complete geometry and SAT pass.
	for command: Dictionary in drawable:
		canvas_value.draw_colored_polygon(command.points,command.color)
	return true
