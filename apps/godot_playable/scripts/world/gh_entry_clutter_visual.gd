extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const ENTRY_WALL_SCRIPT = preload("res://scripts/world/gh_entry_wall_modules_visual.gd")
const PROJECTION_SCRIPT = preload("res://scripts/world/projection.gd")
const MAPPING_RESOURCE = "res://data/gh_entry_clutter.internal-use.json"
const TEXTURE_RESOURCE = "res://assets/props/grey_hive/clutter.png"
const SOURCE_PATH = "assets/derived/asset-pipeline-v1/sprites/png/runtime2d.grey_hive.clutter.v1.png"
const ORIGINAL_PATH = "assets/source/approved/batch03/04_runtime2d.grey_hive.clutter.v1.png"
const ASSET_ID = "runtime2d.grey_hive.clutter.v1"
const ROW_ID = "entry_clutter"
const OUTPUT_BYTES = 1857722
const OUTPUT_SHA256 = "4743030e58286fdac5c4e119941608763d1ceb1d3bbed79ca1d7361ceab97ebe"
const CANVAS_SIZE = Vector2i(1428, 1061)
const FRAME = Rect2i(40, 716, 347, 287)
const COMPONENT_BOUNDS = Rect2i(0, 0, 347, 287)
const COMPONENT_PIXELS = 73359
const COMPONENT_SEED_SOURCE = Vector2i(150, 800)
const COMPONENT_SEED_LOCAL = Vector2i(110, 84)
const DISCONNECTED_SAMPLE_SOURCE = Vector2i(142, 716)
const CONTACT_A_SOURCE = Vector2(50.0, 929.0)
const CONTACT_B_SOURCE = Vector2(369.0, 898.0)
const CONTACT_A_LOCAL = Vector2(10.0, 213.0)
const CONTACT_B_LOCAL = Vector2(329.0, 182.0)
const WORLD_CONTACT_A = Vector3(13.625, 0.0, 10.5)
const WORLD_CONTACT_B = Vector3(14.375, 0.0, 9.5)
const WORLD_POSITION = Vector3(14.0, 0.0, 10.0)
const SHADOW_RADII = Vector2(0.5, 0.55)
const SHADOW_ALPHA = 0.16
const MAX_PPM = 256.0
const MAX_ROTATION_RADIANS = 0.06
const MAX_EXTENT_PX_AT_48 = 96.0
const SHADOW_STEPS = 24
const RAW_COMPONENT_BYTES = 398356
const RAW_COMPONENT_SHA256 = "0c86a9267836caa9c8424787ce9b91e8c4aaac4bbb9a54ec0d9bcfa4d92b40de"
const PACKET_KEYS = [
	"id", "row_id", "kind", "context_key", "asset_id", "p", "texture",
	"source_rect", "destination_rect", "component_bounds", "component_pixel_count",
	"contact_a_source", "contact_b_source", "contact_a_local", "contact_b_local",
	"contact_a_world", "contact_b_world", "transform_origin", "transform_rotation",
	"transform_scale", "uniform_scale", "ppm", "body_rect", "shadow_points", "shadow_rect",
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
	return _numbers_equal(row.get("position"), [14, 0, 10])

func is_valid_packet(packet, snapshot, scene_data, current_key, projection, wall_visual) -> bool:
	if not packet is Dictionary or not _keys_are(packet, PACKET_KEYS):
		return false
	if not _valid_context(snapshot, scene_data, current_key, wall_visual) or not _valid_projection(projection) or _texture == null:
		return false
	if not _packet_geometry_is_finite(packet):
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
	if points.size() != SHADOW_STEPS or not _finite_points(points) or not _finite_rect(bounds) or not _finite_rect(viewport):
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

func _packet_geometry_is_finite(packet: Dictionary) -> bool:
	if not packet.get("p") is Vector3 or not _finite_world(packet.p):
		return false
	for field in ["contact_a_source", "contact_b_source", "contact_a_local", "contact_b_local", "transform_origin", "transform_scale"]:
		if not packet.get(field) is Vector2 or not _finite_point(packet.get(field)):
			return false
	for field in ["contact_a_world", "contact_b_world", "shadow_center_world"]:
		if not packet.get(field) is Vector3 or not _finite_world(packet.get(field)):
			return false
	if not packet.get("source_rect") is Rect2 or not _finite_rect(packet.source_rect):
		return false
	if not packet.get("destination_rect") is Rect2 or not _finite_rect(packet.destination_rect):
		return false
	if not packet.get("body_rect") is Rect2 or not _finite_rect(packet.body_rect):
		return false
	if not packet.get("shadow_rect") is Rect2 or not _finite_rect(packet.shadow_rect):
		return false
	if not packet.get("shadow_points") is PackedVector2Array or packet.shadow_points.size() != SHADOW_STEPS or not _finite_points(packet.shadow_points):
		return false
	if not P.number(packet.get("transform_rotation"), -MAX_ROTATION_RADIANS, MAX_ROTATION_RADIANS) or not P.number(packet.get("uniform_scale"), 0.000001, 1.0e9) or not P.number(packet.get("ppm"), 0.000001, MAX_PPM) or not P.number(packet.get("shadow_alpha"), 0.0, 1.0):
		return false
	if not packet.get("shadow_radii") is Vector2 or not _finite_point(packet.shadow_radii):
		return false
	return true

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
	var expected_axis: Vector2 = Vector2(
		(WORLD_CONTACT_B.x - WORLD_CONTACT_A.x - (WORLD_CONTACT_B.z - WORLD_CONTACT_A.z)) * ppm,
		(WORLD_CONTACT_B.x - WORLD_CONTACT_A.x + WORLD_CONTACT_B.z - WORLD_CONTACT_A.z) * ppm * 0.5
	)
	if not _finite_point(expected_axis) or screen_axis.distance_to(expected_axis) > 0.01:
		return {}
	if not is_finite(source_length) or source_length <= 0.0 or not is_finite(screen_length) or screen_length <= 0.0:
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
		"kind": "gh_entry_clutter",
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
		"ppm": ppm,
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
	if not _rgba_equals8(source_image.get_pixel(DISCONNECTED_SAMPLE_SOURCE.x, DISCONNECTED_SAMPLE_SOURCE.y), [0, 0, 0, 1]):
		return
	var isolated: Image = _isolate_seed_component(source_image)
	if isolated == null or isolated.get_width() != FRAME.size.x or isolated.get_height() != FRAME.size.y or isolated.get_format() != Image.FORMAT_RGBA8:
		return
	if isolated.get_pixel(DISCONNECTED_SAMPLE_SOURCE.x - FRAME.position.x, DISCONNECTED_SAMPLE_SOURCE.y - FRAME.position.y).a > 0.0:
		return
	if not _rgba_equals8(isolated.get_pixel(int(CONTACT_A_LOCAL.x), int(CONTACT_A_LOCAL.y)), [32, 30, 30, 255]) or not _rgba_equals8(isolated.get_pixel(int(CONTACT_B_LOCAL.x), int(CONTACT_B_LOCAL.y)), [79, 58, 28, 251]) or not _rgba_equals8(isolated.get_pixel(COMPONENT_SEED_LOCAL.x, COMPONENT_SEED_LOCAL.y), [121, 101, 74, 253]):
		return
	var selected_bytes: PackedByteArray = isolated.get_data()
	if selected_bytes.size() != RAW_COMPONENT_BYTES:
		return
	var selected_hash: HashingContext = HashingContext.new()
	if selected_hash.start(HashingContext.HASH_SHA256) != OK or selected_hash.update(selected_bytes) != OK or selected_hash.finish().hex_encode() != RAW_COMPONENT_SHA256:
		return
	var candidate_texture: ImageTexture = ImageTexture.create_from_image(isolated)
	if candidate_texture == null or candidate_texture.get_width() != FRAME.size.x or candidate_texture.get_height() != FRAME.size.y:
		return
	_texture = candidate_texture

func _isolate_seed_component(source: Image) -> Image:
	var width: int = FRAME.size.x
	var height: int = FRAME.size.y
	var seed_x: int = COMPONENT_SEED_SOURCE.x - FRAME.position.x
	var seed_y: int = COMPONENT_SEED_SOURCE.y - FRAME.position.y
	if seed_x != COMPONENT_SEED_LOCAL.x or seed_y != COMPONENT_SEED_LOCAL.y or source.get_pixel(COMPONENT_SEED_SOURCE.x, COMPONENT_SEED_SOURCE.y).a <= 0.0:
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
		if tail > capacity:
			return null
	var measured_bounds: Rect2i = Rect2i(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
	if head != tail or selected_pixels != COMPONENT_PIXELS or measured_bounds != COMPONENT_BOUNDS:
		return null
	return isolated

func _valid_mapping(value) -> bool:
	if not value is Dictionary or not _keys_are(value, ["schema", "registeredSourceReuse", "approval", "consumerStatus", "publicRelease", "originalAsset", "sourceAsset", "runtimeAsset", "selection", "installation"]):
		return false
	if typeof(value.get("schema")) != TYPE_STRING or value.get("schema") != "wuxian.gh-entry-clutter.internal-use/v1" or typeof(value.get("registeredSourceReuse")) != TYPE_BOOL or value.get("registeredSourceReuse") != true or value.get("approval") != "INTERNAL_REGISTERED_SOURCE_REUSE" or value.get("consumerStatus") != "NOT_RUN" or value.get("publicRelease") != "NOT_ESTABLISHED":
		return false
	var original_asset = value.get("originalAsset")
	var source_asset = value.get("sourceAsset")
	var runtime_asset = value.get("runtimeAsset")
	var selection = value.get("selection")
	var installation = value.get("installation")
	if not original_asset is Dictionary or not source_asset is Dictionary or not runtime_asset is Dictionary or not selection is Dictionary or not installation is Dictionary:
		return false
	if not _keys_are(original_asset, ["path", "bytes", "sha256", "admission"]) or original_asset.get("path") != ORIGINAL_PATH or not P.counter(original_asset.get("bytes"), 1894441, 1894441) or original_asset.get("sha256") != "d9b7f424d0ac278c9029d7ddc2ad9559f08a3c56562770eefc7ed301d9094a6c" or original_asset.get("admission") != "release_approved":
		return false
	if not _keys_are(source_asset, ["path", "assetId", "bytes", "sha256", "canvas", "format", "admission", "runtimeStatus"]):
		return false
	if source_asset.get("path") != SOURCE_PATH or source_asset.get("assetId") != ASSET_ID or not P.counter(source_asset.get("bytes"), OUTPUT_BYTES, OUTPUT_BYTES) or source_asset.get("sha256") != OUTPUT_SHA256 or source_asset.get("format") != "RGBA8" or source_asset.get("admission") != "release_approved" or source_asset.get("runtimeStatus") != "ADOPT" or not _numbers_equal(source_asset.get("canvas"), [1428, 1061]):
		return false
	if not _keys_are(runtime_asset, ["path", "bytes", "sha256", "canvas", "format"]) or runtime_asset.get("path") != TEXTURE_RESOURCE or not P.counter(runtime_asset.get("bytes"), OUTPUT_BYTES, OUTPUT_BYTES) or runtime_asset.get("sha256") != OUTPUT_SHA256 or runtime_asset.get("format") != "RGBA8" or not _numbers_equal(runtime_asset.get("canvas"), [1428, 1061]):
		return false
	if not _keys_are(selection, ["framePx", "seedPx", "seedLocalPx", "componentBoundsLocalPx", "connectedAlphaPixels", "disconnectedFrameAlphaPixels", "contactsSourcePx", "contactsLocalPx", "localOriginConvention"]):
		return false
	if not _numbers_equal(selection.get("framePx"), [40, 716, 347, 287]) or not _numbers_equal(selection.get("seedPx"), [150, 800]) or not _numbers_equal(selection.get("seedLocalPx"), [110, 84]) or not _numbers_equal(selection.get("componentBoundsLocalPx"), [0, 0, 347, 287]) or not P.counter(selection.get("connectedAlphaPixels"), COMPONENT_PIXELS, COMPONENT_PIXELS) or not P.counter(selection.get("disconnectedFrameAlphaPixels"), 168, 168) or not _numbers_equal(selection.get("contactsSourcePx"), [[50, 929], [369, 898]]) or not _numbers_equal(selection.get("contactsLocalPx"), [[10, 213], [329, 182]]) or selection.get("localOriginConvention") != "frameTopLeft":
		return false
	if not _keys_are(installation, ["worldId", "sceneId", "rowId", "assetId", "layer", "positionM", "contactsM", "spanM", "geometry", "shadow", "registrationMeaning"]):
		return false
	if installation.get("worldId") != "grey_hive" or installation.get("sceneId") != "gh_entry_maintenance" or installation.get("rowId") != ROW_ID or installation.get("assetId") != ASSET_ID or installation.get("layer") != "visual.props_back" or not _numbers_equal(installation.get("positionM"), [14, 0, 10]) or not _numbers_equal(installation.get("contactsM"), [[13.625, 0, 10.5], [14.375, 0, 9.5]]) or not P.number(installation.get("spanM"), 1.25, 1.25) or installation.get("registrationMeaning") != "visualOnlyNoPhysicalDimensionsOrCollision":
		return false
	var geometry = installation.get("geometry")
	if not geometry is Dictionary or not _keys_are(geometry, ["transform", "maximumRotationRadians", "maxExtentPxAt48"]):
		return false
	if geometry.get("transform") != "positive-uniform-scale-small-rotation" or not P.number(geometry.get("maximumRotationRadians"), MAX_ROTATION_RADIANS, MAX_ROTATION_RADIANS) or not P.number(geometry.get("maxExtentPxAt48"), MAX_EXTENT_PX_AT_48, MAX_EXTENT_PX_AT_48):
		return false
	var shadow = installation.get("shadow")
	if not shadow is Dictionary or not _keys_are(shadow, ["centerM", "radiiM", "alpha"]):
		return false
	return _numbers_equal(shadow.get("centerM"), [14, 0, 10]) and _numbers_equal(shadow.get("radiiM"), [0.5, 0.55]) and P.number(shadow.get("alpha"), SHADOW_ALPHA, SHADOW_ALPHA)

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
		elif typeof(expected[index]) == TYPE_INT:
			if not P.counter(value[index], expected[index], expected[index]):
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
	var corners: Array = [
		Vector2(bounds.position),
		Vector2(bounds.position + Vector2i(bounds.size.x, 0)),
		Vector2(bounds.position + bounds.size),
		Vector2(bounds.position + Vector2i(0, bounds.size.y))
	]
	var first: Vector2 = origin + _source_to_screen(corners[0], rotation, scale.x)
	var lo: Vector2 = first
	var hi: Vector2 = first
	for corner in corners:
		var point: Vector2 = origin + _source_to_screen(corner, rotation, scale.x)
		lo = lo.min(point)
		hi = hi.max(point)
	return Rect2(lo, hi - lo)

func _points_bounds(points: PackedVector2Array) -> Rect2:
	if points.is_empty():
		return Rect2()
	var lo: Vector2 = points[0]
	var hi: Vector2 = points[0]
	for point in points:
		lo = lo.min(point)
		hi = hi.max(point)
	return Rect2(lo, hi - lo)

func _finite_point(point: Vector2) -> bool:
	return is_finite(point.x) and is_finite(point.y)

func _finite_world(point: Vector3) -> bool:
	return is_finite(point.x) and is_finite(point.y) and is_finite(point.z)

func _finite_points(points: PackedVector2Array) -> bool:
	for point in points:
		if not _finite_point(point):
			return false
	return true

func _finite_rect(rect: Rect2) -> bool:
	return _finite_point(rect.position) and _finite_point(rect.size) and rect.size.x > 0.0 and rect.size.y > 0.0 and _finite_point(rect.end)

func _rect_intersects(viewport: Rect2, bounds: Rect2) -> bool:
	if not _finite_rect(viewport) or not _finite_rect(bounds):
		return false
	return viewport.intersects(bounds)

func _rgba_equals8(color: Color, expected: Array) -> bool:
	return roundi(color.r * 255.0) == expected[0] and roundi(color.g * 255.0) == expected[1] and roundi(color.b * 255.0) == expected[2] and roundi(color.a * 255.0) == expected[3]
