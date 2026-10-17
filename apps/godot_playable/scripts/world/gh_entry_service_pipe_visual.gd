extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const ENTRY_WALL_SCRIPT = preload("res://scripts/world/gh_entry_wall_modules_visual.gd")
const PROJECTION_SCRIPT = preload("res://scripts/world/projection.gd")
const MAPPING_RESOURCE = "res://data/gh_entry_service_pipe.internal-use.json"
const TEXTURE_RESOURCE = "res://assets/props/return_station/service_pipes.png"
const SOURCE_PATH = "assets/derived/asset-pipeline-v1/sprites/png/runtime2d.grey_hive.pipes_cables.v1.png"
const ASSET_ID = "runtime2d.grey_hive.pipes_cables.v1"
const ROW_ID = "entry_pipes"
const OUTPUT_BYTES = 1049893
const OUTPUT_SHA256 = "7f3c67879398e89adc9026fc2aca5fbe7792b1dc360b73c737c1b2299eac96da"
const CANVAS_SIZE = Vector2i(1414, 1047)
const FRAME = Rect2i(0, 219, 388, 270)
const COMPONENT_BOUNDS = Rect2i(0, 0, 388, 270)
const COMPONENT_PIXELS = 46171
const SEED_SOURCE = Vector2i(100, 320)
const CONTACT_A_SOURCE = Vector2(50.0, 345.0)
const CONTACT_B_SOURCE = Vector2(325.0, 482.0)
const CONTACT_A_LOCAL = Vector2(50.0, 126.0)
const CONTACT_B_LOCAL = Vector2(325.0, 263.0)
const WORLD_CONTACT_A = Vector3(6.5, 0.0, 2.0)
const WORLD_CONTACT_B = Vector3(7.5, 0.0, 2.0)
const WORLD_POSITION = Vector3(7.0, 0.0, 2.0)
const SHADOW_RADII = Vector2(0.6, 0.16)
const SHADOW_ALPHA = 0.18
const MAX_PPM = 256.0
const MAX_ROTATION_RADIANS = 0.025
const MAX_EXTENT_PX_AT_48 = 80.0
const SHADOW_STEPS = 24
const PACKET_KEYS = [
	"id", "row_id", "kind", "context_key", "asset_id", "p", "texture",
	"source_rect", "destination_rect", "component_bounds", "component_pixel_count",
	"contact_a_source", "contact_b_source", "contact_a_local", "contact_b_local",
	"contact_a_world", "contact_b_world", "transform_origin", "transform_rotation",
	"transform_scale", "uniform_scale", "body_rect", "shadow_points", "shadow_rect",
	"shadow_center_world", "shadow_radii", "shadow_alpha"
]

var _attempted: bool = false
var _texture: ImageTexture

func build_packet(snapshot, scene_data, current_key, projection, wall_visual) -> Dictionary:
	if not _valid_context(snapshot, scene_data, current_key, wall_visual) or not _valid_projection(projection):
		return {}
	_load_once()
	if _texture == null or _texture.get_width() != FRAME.size.x or _texture.get_height() != FRAME.size.y:
		return {}
	var packet: Dictionary = _make_packet(current_key, projection)
	return packet if not packet.is_empty() and is_valid_packet(packet, snapshot, scene_data, current_key, projection, wall_visual) else {}

func matches_authored_row(row) -> bool:
	if not row is Dictionary or not _keys_are(row, ["assetId", "id", "layer", "position"]):
		return false
	if typeof(row.get("id")) != TYPE_STRING or typeof(row.get("assetId")) != TYPE_STRING or typeof(row.get("layer")) != TYPE_STRING:
		return false
	if row.get("id") != ROW_ID or row.get("assetId") != ASSET_ID or row.get("layer") != "visual.props_back":
		return false
	return _numbers_equal(row.get("position"), [7, 0, 2])

func is_valid_packet(packet, snapshot, scene_data, current_key, projection, wall_visual) -> bool:
	if not packet is Dictionary or not _keys_are(packet, PACKET_KEYS):
		return false
	if not _valid_context(snapshot, scene_data, current_key, wall_visual) or not _valid_projection(projection) or _texture == null:
		return false
	var expected: Dictionary = _make_packet(current_key, projection)
	if expected.is_empty():
		return false
	for key in PACKET_KEYS:
		if packet.get(key) != expected.get(key):
			return false
	return true

func draw_contact_shadow(canvas, packet, viewport: Rect2, snapshot, scene_data, current_key, projection, wall_visual) -> bool:
	if not canvas is CanvasItem:
		return false
	if not is_valid_packet(packet, snapshot, scene_data, current_key, projection, wall_visual):
		return false
	var points: PackedVector2Array = packet.shadow_points
	var bounds: Rect2 = packet.shadow_rect
	if not _finite_points(points) or not _finite_rect(bounds) or not _finite_rect(viewport):
		return false
	if not _rect_intersects(viewport, bounds):
		return true
	canvas.draw_colored_polygon(points, Color(0.045, 0.052, 0.058, SHADOW_ALPHA))
	return true

func draw_body(canvas, packet, snapshot, scene_data, current_key, projection, wall_visual) -> bool:
	if not canvas is CanvasItem:
		return false
	if not is_valid_packet(packet, snapshot, scene_data, current_key, projection, wall_visual):
		return false
	canvas.draw_set_transform(packet.transform_origin, packet.transform_rotation, packet.transform_scale)
	canvas.draw_texture_rect_region(packet.texture, packet.destination_rect, packet.source_rect, Color.WHITE, false, true)
	canvas.draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
	return true

func _valid_context(snapshot, scene_data, current_key, wall_visual) -> bool:
	if not P.snapshot(snapshot) or not scene_data is Dictionary or typeof(current_key) != TYPE_STRING:
		return false
	if snapshot.get("worldId") != "grey_hive" or snapshot.get("sceneId") != "gh_entry_maintenance":
		return false
	if scene_data.get("worldId") != "grey_hive" or scene_data.get("sceneId") != "gh_entry_maintenance":
		return false
	if not P.counter(snapshot.get("worldEpoch"), 1) or current_key != "grey_hive:gh_entry_maintenance:" + str(snapshot.worldEpoch):
		return false
	if not wall_visual is RefCounted or wall_visual.get_script() != ENTRY_WALL_SCRIPT:
		return false
	if not wall_visual.has_method("has_valid_layout") or not wall_visual.call("has_valid_layout", scene_data):
		return false
	var presentation = scene_data.get("presentation")
	if not presentation is Dictionary:
		return false
	var sprites = presentation.get("sprites")
	if not P.rows(sprites, 2048) or not P.unique(sprites, "id"):
		return false
	var candidates: Array = []
	for row in sprites:
		if row.get("id") == ROW_ID or row.get("assetId") == ASSET_ID:
			candidates.append(row)
	return candidates.size() == 1 and matches_authored_row(candidates[0])

func _valid_projection(projection) -> bool:
	if not projection is RefCounted or projection.get_script() != PROJECTION_SCRIPT:
		return false
	return projection.has_method("project") and P.number(projection.get("ppm"), 0.000001, MAX_PPM)

func _make_packet(current_key: String, projection) -> Dictionary:
	var screen_start = projection.call("project", WORLD_CONTACT_A)
	var screen_end = projection.call("project", WORLD_CONTACT_B)
	if not screen_start is Vector2 or not screen_end is Vector2 or not _finite_point(screen_start) or not _finite_point(screen_end):
		return {}
	var source_axis: Vector2 = CONTACT_B_LOCAL - CONTACT_A_LOCAL
	var screen_axis: Vector2 = screen_end - screen_start
	var source_length: float = source_axis.length()
	var screen_length: float = screen_axis.length()
	var ppm: float = float(projection.get("ppm"))
	if not is_finite(source_length) or source_length <= 0.0 or not is_finite(screen_length) or screen_length <= 0.0 or absf(screen_length - ppm * sqrt(1.25)) > 0.01:
		return {}
	var uniform_scale: float = screen_length / source_length
	var rotation: float = screen_axis.angle() - source_axis.angle()
	if not is_finite(uniform_scale) or uniform_scale <= 0.0 or not is_finite(rotation) or absf(rotation) > MAX_ROTATION_RADIANS:
		return {}
	var transform_scale: Vector2 = Vector2(uniform_scale, uniform_scale)
	var transform_origin: Vector2 = screen_start - _source_to_screen(CONTACT_A_LOCAL, rotation, uniform_scale)
	var endpoint_check: Vector2 = transform_origin + _source_to_screen(CONTACT_B_LOCAL, rotation, uniform_scale)
	if not _finite_point(transform_origin) or not _finite_point(endpoint_check) or endpoint_check.distance_to(screen_end) > 0.01:
		return {}
	var body_rect: Rect2 = _transformed_bounds(transform_origin, rotation, transform_scale, COMPONENT_BOUNDS)
	if not _finite_rect(body_rect) or maxf(body_rect.size.x, body_rect.size.y) > MAX_EXTENT_PX_AT_48 * ppm / 48.0 + 0.01:
		return {}
	var shadow_points: PackedVector2Array = _make_shadow_points(projection)
	var shadow_rect: Rect2 = _points_bounds(shadow_points)
	if shadow_points.size() != SHADOW_STEPS or not _finite_rect(shadow_rect):
		return {}
	var frame_rect: Rect2 = Rect2(Vector2.ZERO, Vector2(float(FRAME.size.x), float(FRAME.size.y)))
	return {
		"id": ROW_ID,
		"row_id": ROW_ID,
		"kind": "gh_entry_service_pipe",
		"context_key": current_key,
		"asset_id": ASSET_ID,
		"p": WORLD_POSITION,
		"texture": _texture,
		"source_rect": frame_rect,
		"destination_rect": frame_rect,
		"component_bounds": COMPONENT_BOUNDS,
		"component_pixel_count": COMPONENT_PIXELS,
		"contact_a_source": CONTACT_A_SOURCE,
		"contact_b_source": CONTACT_B_SOURCE,
		"contact_a_local": CONTACT_A_LOCAL,
		"contact_b_local": CONTACT_B_LOCAL,
		"contact_a_world": WORLD_CONTACT_A,
		"contact_b_world": WORLD_CONTACT_B,
		"transform_origin": transform_origin,
		"transform_rotation": rotation,
		"transform_scale": transform_scale,
		"uniform_scale": uniform_scale,
		"body_rect": body_rect,
		"shadow_points": shadow_points,
		"shadow_rect": shadow_rect,
		"shadow_center_world": WORLD_POSITION,
		"shadow_radii": SHADOW_RADII,
		"shadow_alpha": SHADOW_ALPHA
	}

func _load_once() -> void:
	if _attempted:
		return
	_attempted = true
	var mapping_file = FileAccess.open(MAPPING_RESOURCE, FileAccess.READ)
	if mapping_file == null or mapping_file.get_length() < 1 or mapping_file.get_length() > 8192:
		return
	var mapping_length: int = int(mapping_file.get_length())
	var mapping_bytes: PackedByteArray = mapping_file.get_buffer(mapping_length)
	mapping_file.close()
	if mapping_bytes.size() != mapping_length:
		return
	var mapping_text: String = mapping_bytes.get_string_from_utf8()
	if mapping_text.to_utf8_buffer() != mapping_bytes or not _valid_mapping(JSON.parse_string(mapping_text)):
		return
	var image_file = FileAccess.open(TEXTURE_RESOURCE, FileAccess.READ)
	if image_file == null or image_file.get_length() != OUTPUT_BYTES:
		return
	var png_bytes: PackedByteArray = image_file.get_buffer(OUTPUT_BYTES)
	image_file.close()
	if png_bytes.size() != OUTPUT_BYTES:
		return
	var hash: HashingContext = HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256) != OK or hash.update(png_bytes) != OK or hash.finish().hex_encode() != OUTPUT_SHA256:
		return
	var source_image: Image = Image.new()
	if source_image.load_png_from_buffer(png_bytes) != OK or source_image.get_width() != CANVAS_SIZE.x or source_image.get_height() != CANVAS_SIZE.y or source_image.get_format() != Image.FORMAT_RGBA8:
		return
	var isolated: Image = _isolate_seed_component(source_image)
	if isolated == null or isolated.get_width() != FRAME.size.x or isolated.get_height() != FRAME.size.y or isolated.get_format() != Image.FORMAT_RGBA8:
		return
	if isolated.get_pixel(int(CONTACT_A_LOCAL.x), int(CONTACT_A_LOCAL.y)).a <= 0.0 or isolated.get_pixel(int(CONTACT_B_LOCAL.x), int(CONTACT_B_LOCAL.y)).a <= 0.0:
		return
	var candidate_texture: ImageTexture = ImageTexture.create_from_image(isolated)
	if candidate_texture == null or candidate_texture.get_width() != FRAME.size.x or candidate_texture.get_height() != FRAME.size.y:
		return
	_texture = candidate_texture

func _isolate_seed_component(source: Image) -> Image:
	var width: int = FRAME.size.x
	var height: int = FRAME.size.y
	var seed_x: int = SEED_SOURCE.x - FRAME.position.x
	var seed_y: int = SEED_SOURCE.y - FRAME.position.y
	if seed_x < 0 or seed_y < 0 or seed_x >= width or seed_y >= height or source.get_pixel(FRAME.position.x + seed_x, FRAME.position.y + seed_y).a <= 0.0:
		return null
	var capacity: int = width * height
	var visited: PackedByteArray = PackedByteArray()
	visited.resize(capacity)
	var queue: PackedInt32Array = PackedInt32Array()
	queue.resize(capacity)
	var seed_index: int = seed_y * width + seed_x
	var head: int = 0
	var tail: int = 1
	queue[0] = seed_index
	visited[seed_index] = 1
	var isolated: Image = Image.create_empty(width, height, false, Image.FORMAT_RGBA8)
	if isolated == null:
		return null
	isolated.fill(Color(0.0, 0.0, 0.0, 0.0))
	var min_x: int = width
	var min_y: int = height
	var max_x: int = -1
	var max_y: int = -1
	var selected_pixels: int = 0
	while head < tail:
		var index: int = queue[head]
		head += 1
		var x: int = index % width
		var y: int = floori(float(index) / float(width))
		var source_x: int = FRAME.position.x + x
		var source_y: int = FRAME.position.y + y
		var color: Color = source.get_pixel(source_x, source_y)
		if color.a <= 0.0:
			return null
		isolated.set_pixel(x, y, color)
		selected_pixels += 1
		min_x = mini(min_x, x)
		min_y = mini(min_y, y)
		max_x = maxi(max_x, x)
		max_y = maxi(max_y, y)
		if x > 0:
			var left_index: int = index - 1
			if visited[left_index] == 0 and source.get_pixel(source_x - 1, source_y).a > 0.0:
				visited[left_index] = 1
				queue[tail] = left_index
				tail += 1
		if x + 1 < width:
			var right_index: int = index + 1
			if visited[right_index] == 0 and source.get_pixel(source_x + 1, source_y).a > 0.0:
				visited[right_index] = 1
				queue[tail] = right_index
				tail += 1
		if y > 0:
			var up_index: int = index - width
			if visited[up_index] == 0 and source.get_pixel(source_x, source_y - 1).a > 0.0:
				visited[up_index] = 1
				queue[tail] = up_index
				tail += 1
		if y + 1 < height:
			var down_index: int = index + width
			if visited[down_index] == 0 and source.get_pixel(source_x, source_y + 1).a > 0.0:
				visited[down_index] = 1
				queue[tail] = down_index
				tail += 1
	var measured_bounds: Rect2i = Rect2i(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
	if head != tail or selected_pixels != COMPONENT_PIXELS or measured_bounds != COMPONENT_BOUNDS:
		return null
	return isolated

func _valid_mapping(value) -> bool:
	if not value is Dictionary or not _keys_are(value, ["schema", "registeredSourceReuse", "approval", "GodotDisplay", "publicRelease", "sourceAsset", "runtimeAsset", "selection", "installation"]):
		return false
	if value.get("schema") != "wuxian.gh-entry-service-pipe.internal-use/v1" or value.get("registeredSourceReuse") != true or value.get("approval") != "INTERNAL_REGISTERED_SOURCE_REUSE" or value.get("GodotDisplay") != "NOT_RUN" or value.get("publicRelease") != "NOT_ESTABLISHED":
		return false
	var source_asset = value.get("sourceAsset")
	var runtime_asset = value.get("runtimeAsset")
	var selection = value.get("selection")
	var installation = value.get("installation")
	if not source_asset is Dictionary or not runtime_asset is Dictionary or not selection is Dictionary or not installation is Dictionary:
		return false
	if not _keys_are(source_asset, ["path", "assetId", "bytes", "sha256", "canvas", "format", "admission", "runtimeStatus"]):
		return false
	if source_asset.get("path") != SOURCE_PATH or source_asset.get("assetId") != ASSET_ID or not P.counter(source_asset.get("bytes"), OUTPUT_BYTES, OUTPUT_BYTES) or source_asset.get("sha256") != OUTPUT_SHA256 or source_asset.get("format") != "RGBA8" or source_asset.get("admission") != "release_approved" or source_asset.get("runtimeStatus") != "ADOPT" or not _numbers_equal(source_asset.get("canvas"), [1414, 1047]):
		return false
	if not _keys_are(runtime_asset, ["path", "bytes", "sha256", "canvas", "format"]):
		return false
	if runtime_asset.get("path") != TEXTURE_RESOURCE or not P.counter(runtime_asset.get("bytes"), OUTPUT_BYTES, OUTPUT_BYTES) or runtime_asset.get("sha256") != OUTPUT_SHA256 or runtime_asset.get("format") != "RGBA8" or not _numbers_equal(runtime_asset.get("canvas"), [1414, 1047]):
		return false
	if not _keys_are(selection, ["framePx", "seedPx", "componentBoundsPx", "connectedAlphaPixels", "contactsPx"]):
		return false
	if not _numbers_equal(selection.get("framePx"), [0, 219, 388, 270]) or not _numbers_equal(selection.get("seedPx"), [100, 320]) or not _numbers_equal(selection.get("componentBoundsPx"), [0, 219, 388, 270]) or not P.counter(selection.get("connectedAlphaPixels"), COMPONENT_PIXELS, COMPONENT_PIXELS) or not _numbers_equal(selection.get("contactsPx"), [[50, 345], [325, 482]]):
		return false
	if not _keys_are(installation, ["worldId", "sceneId", "rowId", "assetId", "layer", "positionM", "contactsM", "spanM", "geometry", "shadow"]):
		return false
	if installation.get("worldId") != "grey_hive" or installation.get("sceneId") != "gh_entry_maintenance" or installation.get("rowId") != ROW_ID or installation.get("assetId") != ASSET_ID or installation.get("layer") != "visual.props_back" or not _numbers_equal(installation.get("positionM"), [7, 0, 2]) or not _numbers_equal(installation.get("contactsM"), [[6.5, 0, 2], [7.5, 0, 2]]) or not P.number(installation.get("spanM"), 1.0, 1.0):
		return false
	var geometry = installation.get("geometry")
	if not geometry is Dictionary or not _keys_are(geometry, ["transform", "maximumRotationRadians", "maxExtentPxAt48"]):
		return false
	if geometry.get("transform") != "positive-uniform-scale-small-rotation" or not P.number(geometry.get("maximumRotationRadians"), MAX_ROTATION_RADIANS, MAX_ROTATION_RADIANS) or not P.number(geometry.get("maxExtentPxAt48"), MAX_EXTENT_PX_AT_48, MAX_EXTENT_PX_AT_48):
		return false
	var shadow = installation.get("shadow")
	if not shadow is Dictionary or not _keys_are(shadow, ["centerM", "radiiM", "alpha"]):
		return false
	return _numbers_equal(shadow.get("centerM"), [7, 0, 2]) and _numbers_equal(shadow.get("radiiM"), [0.6, 0.16]) and P.number(shadow.get("alpha"), SHADOW_ALPHA, SHADOW_ALPHA)

func _keys_are(value: Dictionary, expected: Array) -> bool:
	var actual: Array = value.keys()
	var wanted: Array = expected.duplicate()
	actual.sort()
	wanted.sort()
	return actual == wanted

func _numbers_equal(value, expected: Array) -> bool:
	if not value is Array or value.size() != expected.size():
		return false
	for index in range(expected.size()):
		if expected[index] is Array:
			if not _numbers_equal(value[index], expected[index]):
				return false
		elif not P.number(value[index], float(expected[index]), float(expected[index])):
			return false
	return true

func _make_shadow_points(projection) -> PackedVector2Array:
	var points: PackedVector2Array = PackedVector2Array()
	for index in range(SHADOW_STEPS):
		var angle: float = TAU * float(index) / float(SHADOW_STEPS)
		var world_point: Vector3 = Vector3(WORLD_POSITION.x + cos(angle) * SHADOW_RADII.x, WORLD_POSITION.y, WORLD_POSITION.z + sin(angle) * SHADOW_RADII.y)
		var screen_point = projection.call("project", world_point)
		if not screen_point is Vector2 or not _finite_point(screen_point):
			return PackedVector2Array()
		points.append(screen_point)
	return points

func _source_to_screen(point: Vector2, rotation: float, scale: float) -> Vector2:
	return point.rotated(rotation) * scale

func _transformed_bounds(origin: Vector2, rotation: float, scale: Vector2, bounds: Rect2i) -> Rect2:
	var points: PackedVector2Array = PackedVector2Array()
	var left: float = float(bounds.position.x)
	var top: float = float(bounds.position.y)
	var right: float = float(bounds.position.x + bounds.size.x)
	var bottom: float = float(bounds.position.y + bounds.size.y)
	for local_point in [Vector2(left, top), Vector2(right, top), Vector2(right, bottom), Vector2(left, bottom)]:
		points.append(origin + local_point.rotated(rotation) * scale.x)
	return _points_bounds(points)

func _points_bounds(points: PackedVector2Array) -> Rect2:
	if points.size() < 3 or not _finite_points(points):
		return Rect2()
	var minimum: Vector2 = points[0]
	var maximum: Vector2 = points[0]
	for point in points:
		minimum = minimum.min(point)
		maximum = maximum.max(point)
	var result: Rect2 = Rect2(minimum, maximum - minimum)
	return result if _finite_rect(result) else Rect2()

func _rect_intersects(a: Rect2, b: Rect2) -> bool:
	return _finite_rect(a) and _finite_rect(b) and a.intersects(b)

func _finite_point(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _finite_points(points: PackedVector2Array) -> bool:
	for point in points:
		if not _finite_point(point):
			return false
	return true

func _finite_rect(value: Rect2) -> bool:
	return _finite_point(value.position) and _finite_point(value.size) and value.size.x > 0.0 and value.size.y > 0.0 and is_finite(value.end.x) and is_finite(value.end.y)
