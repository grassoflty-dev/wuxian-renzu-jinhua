extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const GHPalette = preload('res://scripts/world/gh_palette.gd')
const SCENE_RESOURCE = 'res://data/scenes/gh_sentinel_arena.json'
const SCENE_BYTES = 8241
const SCENE_SHA256 = '3e3e3a52dca003ec13c7b3698317f6a32627091eeda833a49896910e521f45bb'
# Visual relief only; never collision height or a support surface.
const RELIEF_M = 12.0 / 48.0
const NORTH_IMAGE_PATH = 'res://assets/props/return_station/bulkhead_tiles.png'
var north_image_attempted := false
var north_image: Texture2D
const WALLS = [
	{'id':'sentinel_arena_wall_north','polygon':[[0,0],[24,0],[24,0.5],[0,0.5]]},
	{'id':'sentinel_arena_wall_south','polygon':[[0,15.5],[24,15.5],[24,16],[0,16]]},
	{'id':'sentinel_arena_wall_west_north','polygon':[[0,0.5],[0.5,0.5],[0.5,6.5],[0,6.5]]},
	{'id':'sentinel_arena_wall_west_south','polygon':[[0,9.5],[0.5,9.5],[0.5,15.5],[0,15.5]]},
	{'id':'sentinel_arena_wall_east_north','polygon':[[23.5,0.5],[24,0.5],[24,6.5],[23.5,6.5]]},
	{'id':'sentinel_arena_wall_east_south','polygon':[[23.5,9.5],[24,9.5],[24,15.5],[23.5,15.5]]}
]
const PYLONS = [
	{'id':'sentinel_arena_pylon_collision_left','polygon':[[7,5],[8,5],[8,7],[7,7]]},
	{'id':'sentinel_arena_pylon_collision_right','polygon':[[16,5],[17,5],[17,7],[16,7]]}
]
# These are union boundaries, without rectangle-to-rectangle internal edges.
const OUTLINES = [
	[[0,0],[24,0],[24,6.5],[23.5,6.5],[23.5,0.5],[0.5,0.5],[0.5,6.5],[0,6.5]],
	[[0,9.5],[0.5,9.5],[0.5,15.5],[23.5,15.5],[23.5,9.5],[24,9.5],[24,16],[0,16]]
]
const FACES = [
	{'a':[24,0],'b':[24,6.5],'clip_end':false},
	{'a':[0.5,0.5],'b':[0.5,6.5],'clip_end':false},
	{'a':[0.5,0.5],'b':[23.5,0.5],'clip_end':true},
	{'a':[0,6.5],'b':[0.5,6.5],'clip_end':false},
	{'a':[23.5,6.5],'b':[24,6.5],'clip_end':false},
	{'a':[0.5,9.5],'b':[0.5,15.5],'clip_end':true},
	{'a':[24,9.5],'b':[24,16],'clip_end':false},
	{'a':[0,16],'b':[24,16],'clip_end':false}
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

# The caller must first succeed in draw_shell for this exact frame/context.
func owns_collision(row: Variant) -> bool:
	for expected: Dictionary in WALLS:
		if _same_json(row,expected):
			return true
	return false

func _known_collision(row: Variant) -> bool:
	if owns_collision(row):
		return true
	for expected: Dictionary in PYLONS:
		if _same_json(row,expected):
			return true
	return false

func _load_scene_once() -> void:
	if _scene_attempted:
		return
	_scene_attempted = true
	var file = FileAccess.open(SCENE_RESOURCE,FileAccess.READ)
	if file == null:
		return
	if file.get_length() != SCENE_BYTES:
		file.close()
		return
	var bytes: PackedByteArray = file.get_buffer(SCENE_BYTES)
	file.close()
	if bytes.size() != SCENE_BYTES:
		return
	var hash = HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256) != OK or hash.update(bytes) != OK or hash.finish().hex_encode() != SCENE_SHA256:
		return
	var source_text: String = bytes.get_string_from_utf8()
	if source_text.to_utf8_buffer() != bytes:
		return
	var decoded: Variant = JSON.parse_string(source_text)
	if decoded is Dictionary:
		_canonical_scene = decoded

func prepare(snapshot: Variant, scene: Variant, current_key: Variant, scene_hash: Variant) -> void:
	if not snapshot is Dictionary or not scene is Dictionary:
		return
	if not P.snapshot(snapshot) or snapshot.get('entryToken') != null or snapshot.get('worldId') != 'grey_hive' or snapshot.get('sceneId') != 'gh_sentinel_arena' or scene_hash != SCENE_SHA256:
		return
	if not P.counter(snapshot.get('worldEpoch'),1) or typeof(current_key) != TYPE_STRING or current_key != 'grey_hive:gh_sentinel_arena:'+str(snapshot.worldEpoch):
		return
	_load_scene_once()

func _layout(snapshot: Dictionary, scene: Dictionary, key: Variant, scene_hash: Variant) -> bool:
	if not P.snapshot(snapshot) or snapshot.get('entryToken') != null or snapshot.get('worldId') != 'grey_hive' or snapshot.get('sceneId') != 'gh_sentinel_arena' or scene_hash != SCENE_SHA256:
		return false
	if not P.counter(snapshot.get('worldEpoch'),1) or typeof(key) != TYPE_STRING or key != 'grey_hive:gh_sentinel_arena:'+str(snapshot.worldEpoch):
		return false
	if _canonical_scene.is_empty() or scene.size() != _canonical_scene.size():
		return false
	for field: Variant in _canonical_scene:
		if field != 'collision' and (not scene.has(field) or not _same_json(scene[field],_canonical_scene[field])):
			return false
	var rows: Variant = scene.get('collision')
	if not rows is Array or rows.size() != 8:
		return false
	var seen: Dictionary = {}
	for row: Variant in rows:
		if not _known_collision(row) or seen.has(row.id):
			return false
		seen[row.id] = true
	return true

func _v2(value: Variant) -> bool:
	return typeof(value) == TYPE_VECTOR2 and is_finite(value.x) and is_finite(value.y)

func _v3(value: Variant) -> bool:
	return typeof(value) == TYPE_VECTOR3 and is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _rect(value: Rect2) -> bool:
	return _v2(value.position) and _v2(value.size) and _v2(value.end) and value.size.x > 0 and value.size.y > 0

func _camera(camera: Variant, viewport: Rect2) -> bool:
	if not camera is RefCounted or not is_instance_valid(camera) or camera.get_script() != ProjectionScript or typeof(camera.initialized) != TYPE_BOOL or not camera.initialized:
		return false
	return P.number(camera.ppm) and camera.ppm > 0 and camera.ppm <= 256 and _v3(camera.origin) and _v2(camera.screen_center) and _v2(camera.viewport) and camera.viewport.x > 0 and camera.viewport.y > 0 and camera.viewport == viewport.size

func _project(points: Array, camera: Variant) -> PackedVector2Array:
	var result = PackedVector2Array()
	for point: Array in points:
		result.append(camera.project(Vector3(point[0],RELIEF_M,point[1])))
	return result

func _plan(camera: Variant) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for face: Dictionary in FACES:
		var a = Vector3(face.a[0],0,face.a[1])
		var b = Vector3(face.b[0],0,face.b[1])
		var top_a: Vector2 = camera.project(a+Vector3.UP*RELIEF_M)
		var top_b: Vector2 = camera.project(b+Vector3.UP*RELIEF_M)
		var original = PackedVector2Array([camera.project(a),camera.project(b),top_b,top_a])
		# The same-screen top point is (x+h-y,h,z+h-y). Exactly two
		# concave endpoints enter another canonical cap at this height.
		var low_b: Vector3 = b-(b-a).normalized()*RELIEF_M if face.clip_end else b
		var points = PackedVector2Array([camera.project(a),camera.project(low_b),top_b,top_a])
		result.append({'points':points,'original':original,'color':GHPalette.FACE,'width':0.0})
	for row: Dictionary in WALLS:
		var points = _project(row.polygon,camera)
		result.append({'points':points,'original':points,'color':GHPalette.CAP,'width':0.0})
	for boundary: Array in OUTLINES:
		var points = _project(boundary,camera)
		points.append(points[0])
		result.append({'points':points,'original':points,'color':GHPalette.EDGE,'width':1.2*camera.ppm/48.0})
	return result

func _valid_points(points: PackedVector2Array, line: bool) -> bool:
	if points.size() != (9 if line else 4):
		return false
	for point: Vector2 in points:
		if not _v2(point):
			return false
	if line:
		for index: int in range(1,points.size()):
			var length_squared: float = (points[index]-points[index-1]).length_squared()
			if not is_finite(length_squared) or length_squared <= 0.0:
				return false
		return true
	var area: float = 0.0
	for index: int in range(1,points.size()-1):
		area += (points[index]-points[0]).cross(points[index+1]-points[0])
	return is_finite(area) and absf(area) > 0.0

func _bounds(points: PackedVector2Array, width: float) -> Rect2:
	var box = Rect2(points[0],Vector2.ZERO)
	for point: Vector2 in points:
		box = box.expand(point)
	return box.grow(width*0.5)

func _intersects(a: Rect2, b: Rect2) -> bool:
	return a.end.x >= b.position.x and a.position.x <= b.end.x and a.end.y >= b.position.y and a.position.y <= b.end.y

func _north_placements(camera: Variant) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	if OUTLINES[0]!=[[0,0],[24,0],[24,6.5],[23.5,6.5],[23.5,0.5],[0.5,0.5],[0.5,6.5],[0,6.5]] or FACES[2].a!=[0.5,0.5] or FACES[2].b!=[23.5,0.5] or not FACES[2].clip_end or WALLS[0].id!='sentinel_arena_wall_north' or WALLS[0].polygon!=[[0,0],[24,0],[24,0.5],[0,0.5]]:
		return result
	if not north_image_attempted:
		north_image_attempted = true
		if ResourceLoader.exists(NORTH_IMAGE_PATH):
			var loaded = load(NORTH_IMAGE_PATH)
			if loaded is Texture2D:
				north_image = loaded
		if not is_instance_valid(north_image):
			var image = Image.new()
			if image.load(NORTH_IMAGE_PATH)==OK and image.get_width()==1418 and image.get_height()==1028:
				north_image = ImageTexture.create_from_image(image)
	if not is_instance_valid(north_image) or north_image.get_width()!=1418 or north_image.get_height()!=1028:
		return result
	var foot_a := Vector2(50,322)
	var foot_b := Vector2(300,460)
	var source_axis := foot_b-foot_a
	for i in range(23):
		var a: Vector2 = camera.project(Vector3(0.5+i,0,0.5))
		var b: Vector2 = camera.project(Vector3(1.5+i,0,0.5))
		if not _v2(a) or not _v2(b):
			return []
		var delta := b-a
		var scale := delta.length()/source_axis.length()
		var angle := delta.angle()-source_axis.angle()
		var origin := a-foot_a.rotated(angle)*scale
		if not is_finite(scale) or scale<=0.0 or not is_finite(angle) or not _v2(origin) or (origin+foot_b.rotated(angle)*scale).distance_to(b)>0.01:
			return []
		result.append({'origin':origin,'angle':angle,'scale':scale})
	return result

func draw_shell(canvas: Variant, snapshot: Variant, scene: Variant, current_key: Variant, scene_hash: Variant, camera: Variant, viewport: Variant) -> bool:
	if not canvas is CanvasItem or not is_instance_valid(canvas) or canvas.is_queued_for_deletion() or not canvas.is_inside_tree() or not canvas.is_visible_in_tree():
		return false
	if not snapshot is Dictionary or not scene is Dictionary or not viewport is Rect2 or not _rect(viewport) or not _camera(camera,viewport) or not _layout(snapshot,scene,current_key,scene_hash):
		return false
	var plan: Array[Dictionary] = _plan(camera)
	if plan.size() != 16:
		return false
	var original_visible: bool = false
	var emitted_visible: bool = false
	for index: int in range(plan.size()):
		var command: Dictionary = plan[index]
		var line: bool = index >= 14
		var width: float = command.width
		if not is_finite(width) or (line and width <= 0.0) or (not line and width != 0.0):
			return false
		if not _valid_points(command.points,line) or not _valid_points(command.original,line):
			return false
		var box: Rect2 = _bounds(command.points,width)
		var original_box: Rect2 = _bounds(command.original,width)
		if not _rect(box) or not _rect(original_box):
			return false
		original_visible = original_visible or _intersects(original_box,viewport)
		emitted_visible = emitted_visible or _intersects(box,viewport)
	if not original_visible or not emitted_visible:
		return false
	var placements = _north_placements(camera)
	var use_north_image = placements.size()==23
	# No area epsilon and no per-cap/per-face dropping. Floating-point collapse
	# rejects the entire plan above. Every successful shell retains all six caps.
	for index in range(plan.size()):
		var command: Dictionary = plan[index]
		if use_north_image and index==2:
			for placement in placements:
				canvas.draw_set_transform(placement.origin,placement.angle,Vector2.ONE*placement.scale)
				canvas.draw_texture_rect_region(north_image,Rect2(0,0,355,480),Rect2(0,0,355,480))
			canvas.draw_set_transform(Vector2.ZERO,0.0,Vector2.ONE)
			continue
		if use_north_image and index==8:
			continue
		if use_north_image and index==14:
			var points: PackedVector2Array = command.points
			canvas.draw_polyline(PackedVector2Array([points[1],points[2],points[3],points[4]]),command.color,command.width,true)
			canvas.draw_polyline(PackedVector2Array([points[5],points[6],points[7],points[0]]),command.color,command.width,true)
			continue
		if command.width > 0:
			canvas.draw_polyline(command.points,command.color,command.width,true)
		else:
			canvas.draw_colored_polygon(command.points,command.color)
	return true
