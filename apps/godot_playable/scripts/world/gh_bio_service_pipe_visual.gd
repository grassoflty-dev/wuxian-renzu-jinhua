extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const MAPPING_RESOURCE = "res://data/gh_bio_service_pipe.internal-use.json"
const TEXTURE_RESOURCE = "res://assets/props/return_station/service_pipes.png"
const SOURCE_PATH = "assets/derived/asset-pipeline-v1/sprites/png/runtime2d.grey_hive.pipes_cables.v1.png"
const ASSET_ID = "runtime2d.grey_hive.pipes_cables.v1"
const ROW_ID = "bio_service_pipes"
const OUTPUT_BYTES = 1049893
const OUTPUT_SHA256 = "7f3c67879398e89adc9026fc2aca5fbe7792b1dc360b73c737c1b2299eac96da"
const CANVAS_SIZE = Vector2i(1414, 1047)
const FRAME = Rect2i(0, 219, 388, 270)
const EXPECTED_COMPONENT_BOUNDS = Rect2i(0, 0, 388, 270)
const EXPECTED_COMPONENT_PIXELS = 46171
const COMPONENT_SEED_SOURCE = Vector2i(100, 320)
const CONTACT_A_SOURCE = Vector2(50.0, 345.0)
const CONTACT_B_SOURCE = Vector2(325.0, 482.0)
const CONTACT_A_LOCAL = Vector2(50.0, 126.0)
const CONTACT_B_LOCAL = Vector2(325.0, 263.0)
const WORLD_CONTACT_A = Vector3(7.5, 0.0, 3.0)
const WORLD_CONTACT_B = Vector3(8.5, 0.0, 3.0)
const WORLD_POSITION = Vector3(8.0, 0.0, 3.0)
const SHADOW_RADII = Vector2(0.6, 0.16)
const SHADOW_ALPHA = 0.18
const MAX_PPM = 256.0
const MAX_ROTATION_RADIANS = 0.025
const MAX_EXTENT_PX_AT_48 = 80.0
const SHADOW_STEPS = 24

var _attempted: bool = false
var _texture: ImageTexture
var _selected_bounds: Rect2i = Rect2i()
var _selected_pixel_count: int = 0

func build_packet(snapshot: Dictionary, scene_data: Dictionary, current_key: String, projection, wall_visual) -> Dictionary:
	if not _valid_context(snapshot, scene_data, current_key, wall_visual):
		return {}
	if not _valid_projection(projection):
		return {}
	_load_once()
	if _texture == null or _texture.get_width() != FRAME.size.x or _texture.get_height() != FRAME.size.y:
		return {}
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
	var body_rect: Rect2 = _transformed_bounds(transform_origin, rotation, transform_scale, _selected_bounds)
	if not _finite_rect(body_rect) or maxf(body_rect.size.x, body_rect.size.y) > MAX_EXTENT_PX_AT_48 * ppm / 48.0 + 0.01:
		return {}
	var shadow_points: PackedVector2Array = _make_shadow_points(projection)
	var shadow_rect: Rect2 = _points_bounds(shadow_points)
	if shadow_points.size() != SHADOW_STEPS or not _finite_rect(shadow_rect):
		return {}
	var source_rect: Rect2 = Rect2(Vector2.ZERO, Vector2(float(FRAME.size.x), float(FRAME.size.y)))
	return {
		"id": ROW_ID,
		"row_id": ROW_ID,
		"asset_id": ASSET_ID,
		"p": WORLD_POSITION,
		"texture": _texture,
		"source_rect": source_rect,
		"destination_rect": source_rect,
		"component_bounds": _selected_bounds,
		"component_pixel_count": _selected_pixel_count,
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

func matches_authored_row(row) -> bool:
	if not row is Dictionary or not _keys_are(row, ["assetId", "id", "layer", "position"]):
		return false
	if typeof(row.get("id")) != TYPE_STRING or typeof(row.get("assetId")) != TYPE_STRING or typeof(row.get("layer")) != TYPE_STRING:
		return false
	if row.get("id") != ROW_ID or row.get("assetId") != ASSET_ID or row.get("layer") != "visual.props_back":
		return false
	return _numbers_equal(row.get("position"), [8, 0, 3])

func draw_contact_shadow(canvas: CanvasItem, packet: Dictionary, viewport: Rect2) -> bool:
	var points = packet.get("shadow_points")
	var bounds = packet.get("shadow_rect")
	if not points is PackedVector2Array or points.size() != SHADOW_STEPS or not _finite_points(points) or not bounds is Rect2 or not _finite_rect(bounds):
		return false
	if not _rect_intersects(viewport, bounds):
		return true
	canvas.draw_colored_polygon(points, Color(0.045, 0.052, 0.058, SHADOW_ALPHA))
	return true

func draw_body(canvas: CanvasItem, packet: Dictionary) -> bool:
	var texture = packet.get("texture")
	var source_rect = packet.get("source_rect")
	var destination_rect = packet.get("destination_rect")
	var origin = packet.get("transform_origin")
	var rotation = packet.get("transform_rotation")
	var scale = packet.get("transform_scale")
	if not texture is Texture2D or texture != _texture or not source_rect is Rect2 or not destination_rect is Rect2:
		return false
	if source_rect != destination_rect or source_rect != Rect2(Vector2.ZERO, Vector2(float(FRAME.size.x), float(FRAME.size.y))):
		return false
	if not origin is Vector2 or not scale is Vector2 or not is_finite(rotation) or not _finite_point(origin) or not _finite_point(scale):
		return false
	if scale.x <= 0.0 or scale.y <= 0.0 or not is_equal_approx(scale.x, scale.y) or absf(rotation) > MAX_ROTATION_RADIANS or texture.get_width() != FRAME.size.x or texture.get_height() != FRAME.size.y:
		return false
	canvas.draw_set_transform(origin, rotation, scale)
	canvas.draw_texture_rect_region(texture, destination_rect, source_rect, Color.WHITE, false, true)
	canvas.draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
	return true

func _valid_context(snapshot: Dictionary, scene_data: Dictionary, current_key: String, wall_visual) -> bool:
	if not P.snapshot(snapshot):
		return false
	if typeof(snapshot.get("worldId")) != TYPE_STRING or snapshot.get("worldId") != "grey_hive" or typeof(snapshot.get("sceneId")) != TYPE_STRING or snapshot.get("sceneId") != "gh_bio_isolation":
		return false
	if typeof(scene_data.get("worldId")) != TYPE_STRING or scene_data.get("worldId") != "grey_hive" or typeof(scene_data.get("sceneId")) != TYPE_STRING or scene_data.get("sceneId") != "gh_bio_isolation":
		return false
	if not P.counter(snapshot.get("worldEpoch"), 1) or current_key != "grey_hive:gh_bio_isolation:" + str(snapshot.get("worldEpoch")):
		return false
	if wall_visual == null or not wall_visual is Object or not wall_visual.has_method("has_valid_layout") or not wall_visual.call("has_valid_layout", scene_data):
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
	if projection == null or not projection is Object or not projection.has_method("project"):
		return false
	return P.number(projection.get("ppm"), 0.000001, MAX_PPM)

func _load_once() -> void:
	if _attempted:
		return
	_attempted = true
	var mapping_file = FileAccess.open(MAPPING_RESOURCE, FileAccess.READ)
	if mapping_file == null or mapping_file.get_length() <= 0 or mapping_file.get_length() > 8192:
		return
	var mapping_bytes: PackedByteArray = mapping_file.get_buffer(mapping_file.get_length())
	mapping_file.close()
	var mapping_text: String = mapping_bytes.get_string_from_utf8()
	if mapping_text.to_utf8_buffer() != mapping_bytes or not _valid_mapping(JSON.parse_string(mapping_text)):
		return
	var image_file = FileAccess.open(TEXTURE_RESOURCE, FileAccess.READ)
	if image_file == null or image_file.get_length() != OUTPUT_BYTES:
		return
	var png_bytes: PackedByteArray = image_file.get_buffer(image_file.get_length())
	image_file.close()
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
	_selected_bounds = EXPECTED_COMPONENT_BOUNDS
	_selected_pixel_count = EXPECTED_COMPONENT_PIXELS
	_texture = candidate_texture

func _isolate_seed_component(source: Image) -> Image:
	var width: int = FRAME.size.x
	var height: int = FRAME.size.y
	var seed_x: int = COMPONENT_SEED_SOURCE.x - FRAME.position.x
	var seed_y: int = COMPONENT_SEED_SOURCE.y - FRAME.position.y
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
	if head != tail or selected_pixels != EXPECTED_COMPONENT_PIXELS or measured_bounds != EXPECTED_COMPONENT_BOUNDS:
		return null
	return isolated

func _valid_mapping(value) -> bool:
	if not value is Dictionary or not _keys_are(value, ["schema", "registeredSourceReuse", "approval", "GodotDisplay", "publicRelease", "sourceAsset", "runtimeAsset", "selection", "installation"]):
		return false
	if value.get("schema") != "wuxian.gh-bio-service-pipe.internal-use/v1" or value.get("registeredSourceReuse") != true or value.get("approval") != "INTERNAL_REGISTERED_SOURCE_REUSE" or value.get("GodotDisplay") != "NOT_RUN" or value.get("publicRelease") != "NOT_ESTABLISHED":
		return false
	var source_asset = value.get("sourceAsset")
	var runtime_asset = value.get("runtimeAsset")
	var selection = value.get("selection")
	var installation = value.get("installation")
	if not source_asset is Dictionary or not runtime_asset is Dictionary or not selection is Dictionary or not installation is Dictionary:
		return false
	if not _keys_are(source_asset, ["path", "assetId", "bytes", "sha256", "canvas", "format", "admission", "runtimeStatus"]):
		return false
	if source_asset.get("path") != SOURCE_PATH or source_asset.get("assetId") != ASSET_ID or source_asset.get("bytes") != OUTPUT_BYTES or source_asset.get("sha256") != OUTPUT_SHA256 or source_asset.get("format") != "RGBA8" or source_asset.get("admission") != "release_approved" or source_asset.get("runtimeStatus") != "ADOPT" or not _numbers_equal(source_asset.get("canvas"), [1414, 1047]):
		return false
	if not _keys_are(runtime_asset, ["path", "bytes", "sha256", "canvas", "format"]):
		return false
	if runtime_asset.get("path") != TEXTURE_RESOURCE or runtime_asset.get("bytes") != OUTPUT_BYTES or runtime_asset.get("sha256") != OUTPUT_SHA256 or runtime_asset.get("format") != "RGBA8" or not _numbers_equal(runtime_asset.get("canvas"), [1414, 1047]):
		return false
	if not _keys_are(selection, ["framePx", "seedPx", "componentBoundsPx", "connectedAlphaPixels", "contactsPx"]):
		return false
	if not _numbers_equal(selection.get("framePx"), [0, 219, 388, 270]) or not _numbers_equal(selection.get("seedPx"), [100, 320]) or not _numbers_equal(selection.get("componentBoundsPx"), [0, 219, 388, 270]) or not P.counter(selection.get("connectedAlphaPixels"), EXPECTED_COMPONENT_PIXELS, EXPECTED_COMPONENT_PIXELS) or not _numbers_equal(selection.get("contactsPx"), [[50, 345], [325, 482]]):
		return false
	if not _keys_are(installation, ["worldId", "sceneId", "rowId", "assetId", "layer", "positionM", "contactsM", "spanM", "geometry", "shadow"]):
		return false
	if installation.get("worldId") != "grey_hive" or installation.get("sceneId") != "gh_bio_isolation" or installation.get("rowId") != ROW_ID or installation.get("assetId") != ASSET_ID or installation.get("layer") != "visual.props_back" or not _numbers_equal(installation.get("positionM"), [8, 0, 3]) or not _numbers_equal(installation.get("contactsM"), [[7.5, 0, 3], [8.5, 0, 3]]) or not P.number(installation.get("spanM"), 1.0, 1.0):
		return false
	var geometry = installation.get("geometry")
	if not geometry is Dictionary or not _keys_are(geometry, ["transform", "maximumRotationRadians", "maxExtentPxAt48"]):
		return false
	if geometry.get("transform") != "positive-uniform-scale-small-rotation" or not P.number(geometry.get("maximumRotationRadians"), MAX_ROTATION_RADIANS, MAX_ROTATION_RADIANS) or not P.number(geometry.get("maxExtentPxAt48"), MAX_EXTENT_PX_AT_48, MAX_EXTENT_PX_AT_48):
		return false
	var shadow = installation.get("shadow")
	if not shadow is Dictionary or not _keys_are(shadow, ["centerM", "radiiM", "alpha"]):
		return false
	return _numbers_equal(shadow.get("centerM"), [8, 0, 3]) and _numbers_equal(shadow.get("radiiM"), [0.6, 0.16]) and P.number(shadow.get("alpha"), 0.0, SHADOW_ALPHA) and P.number(shadow.get("alpha"), SHADOW_ALPHA, SHADOW_ALPHA)

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
