extends RefCounted

const MODULES_RESOURCE = "res://data/rs_bulkhead_modules.internal-use.json"
const MODULE_TEXTURE_PATH = "res://assets/props/return_station/bulkhead_tiles.png"
const MODULE_BYTES = 1673423
const MODULE_SHA256 = "c5a18cae9a3ddc95c8515f6b342cb7ca3be12c23ff502ab5b3f16000bb4c9a5b"
const SOURCE_PATH = "assets/derived/asset-pipeline-v1/sprites/png/runtime2d.grey_hive.wall_tiles.v1.png"
const SOURCE_BYTES = 1673423
const FRAME = Rect2i(0, 0, 355, 480)
const PAINTED_BOUNDS = Rect2i(41, 6, 299, 457)
const FOOT_A = Vector2(50.0, 322.0)
const FOOT_B = Vector2(300.0, 460.0)
const MAX_MODULES = 128
const MAX_EXTENT_PX_AT_48 = 96.0
const BULKHEAD_IDS = ["north_bulkhead", "south_bulkhead", "west_bulkhead_north", "west_bulkhead_south", "east_bulkhead_north", "east_bulkhead_south"]
const STRUCTURE_IDS = ["north_bulkhead", "south_bulkhead", "west_bulkhead_north", "west_bulkhead_south", "east_bulkhead_north", "east_bulkhead_south", "service_island_north", "service_island_south"]
const RUNS = [
	{"id":"north_bulkhead", "start":Vector3(0.0, 0.0, 0.25), "end":Vector3(24.0, 0.0, 0.25), "axis":"x"},
	{"id":"south_bulkhead", "start":Vector3(0.0, 0.0, 15.75), "end":Vector3(24.0, 0.0, 15.75), "axis":"x"},
	{"id":"west_bulkhead_north", "start":Vector3(0.25, 0.0, 0.5), "end":Vector3(0.25, 0.0, 5.0), "axis":"z"},
	{"id":"west_bulkhead_south", "start":Vector3(0.25, 0.0, 10.0), "end":Vector3(0.25, 0.0, 15.5), "axis":"z"},
	{"id":"east_bulkhead_north", "start":Vector3(23.75, 0.0, 0.5), "end":Vector3(23.75, 0.0, 5.0), "axis":"z"},
	{"id":"east_bulkhead_south", "start":Vector3(23.75, 0.0, 10.0), "end":Vector3(23.75, 0.0, 15.5), "axis":"z"}
]

var attempted: bool = false
var module_texture: ImageTexture
var terminal_texture: ImageTexture
var terminal_painted_bounds: Rect2i = Rect2i()

func build_packets(snapshot: Dictionary, scene_data: Dictionary, current_key: String, projection, structure_visual) -> Array:
	if not _valid_context(snapshot, scene_data, current_key, structure_visual) or not is_finite(projection.ppm) or projection.ppm <= 0.0 or projection.ppm > 256.0:
		return []
	_load_once()
	if module_texture == null or terminal_texture == null:
		return []
	var packets: Array = []
	var scale_at_48: float = projection.ppm / 48.0
	for run in RUNS:
		var run_length: float = run.start.distance_to(run.end)
		var direction: Vector3 = (run.end - run.start).normalized()
		var cursor: float = 0.0
		var module_index: int = 0
		while cursor < run_length:
			var remaining: float = run_length - cursor
			var segment_length: float = 1.0 if remaining >= 1.0 else 0.5
			if not is_equal_approx(remaining, segment_length) and segment_length == 0.5:
				return []
			var world_start: Vector3 = run.start + direction * cursor
			var world_end: Vector3 = run.start + direction * (cursor + segment_length)
			var screen_start: Vector2 = projection.project(world_start)
			var screen_end: Vector2 = projection.project(world_end)
			var screen_axis: Vector2 = screen_end - screen_start
			var source_axis: Vector2 = FOOT_B - FOOT_A
			var partial_source_fraction: float = 0.5 if segment_length == 0.5 else 1.0
			var source_length: float = source_axis.length() * partial_source_fraction
			var source_contact_end: Vector2 = FOOT_A + source_axis * partial_source_fraction
			var ratio: float = screen_axis.length() / source_length
			var reflected: bool = run.axis == "z"
			var target_angle: float = screen_axis.angle()
			var source_angle: float = source_axis.angle()
			var reflected_source_angle: float = PI - source_angle if reflected else source_angle
			var rotation: float = target_angle - reflected_source_angle
			var transform_scale: Vector2 = Vector2(-ratio, ratio) if reflected else Vector2(ratio, ratio)
			var transform_origin: Vector2 = screen_start - _source_to_screen(FOOT_A, rotation, transform_scale)
			var endpoint_check: Vector2 = transform_origin + _source_to_screen(source_contact_end, rotation, transform_scale)
			var source_bounds: Rect2i = terminal_painted_bounds if segment_length == 0.5 else PAINTED_BOUNDS
			var painted_rect: Rect2 = _transformed_bounds(transform_origin, rotation, transform_scale, source_bounds)
			var expected_step: float = projection.ppm * segment_length * sqrt(1.25)
			if not is_finite(ratio) or ratio <= 0.0 or absf(endpoint_check.distance_to(screen_end)) > 0.01 or absf(screen_axis.length() - expected_step) > 0.01:
				return []
			if maxf(painted_rect.size.x, painted_rect.size.y) > MAX_EXTENT_PX_AT_48 * scale_at_48 + 0.01:
				return []
			var fade_group: String = "rs:bulkhead-module:%s:%03d" % [run.id, module_index]
			packets.append({
				"id": fade_group,
				"kind": "rs_bulkhead_module",
				"p": (world_start + world_end) * 0.5,
				"run_id": run.id,
				"index": module_index,
				"terminal": segment_length == 0.5,
				"fade_group": fade_group,
				"texture": terminal_texture if segment_length == 0.5 else module_texture,
				"source_rect": Rect2(FRAME),
				"transform_origin": transform_origin,
				"transform_rotation": rotation,
				"transform_scale": transform_scale,
				"body_rect": painted_rect
			})
			cursor += segment_length
			module_index += 1
			if packets.size() > MAX_MODULES:
				return []
	if packets.size() != 70:
		return []
	return packets

func filter_legacy_bulkhead_packets(packets: Array) -> Array:
	var result: Array = []
	for packet in packets:
		if not packet is Dictionary or not _is_legacy_bulkhead_packet(packet.get("id")):
			result.append(packet)
	return result

func front_fade_alphas(packets: Array, current_by_group: Dictionary, player_rect: Rect2, delta: float, reduced_motion: bool, structure_visual) -> Dictionary:
	var result: Dictionary = {}
	for packet in packets:
		if not packet is Dictionary or typeof(packet.get("fade_group")) != TYPE_STRING or not packet.get("body_rect") is Rect2:
			continue
		var group: String = packet.fade_group
		var body_rect: Rect2 = packet.body_rect
		var target: float = 0.25 if _rect_intersects(body_rect, player_rect) else 1.0
		result[group] = structure_visual.step_front_alpha(float(current_by_group.get(group, 1.0)), target, delta, reduced_motion)
	return result

func draw_module(canvas: CanvasItem, packet: Dictionary, ppm: float, alpha: float) -> bool:
	var texture = packet.get("texture")
	var origin = packet.get("transform_origin")
	var rotation = packet.get("transform_rotation")
	var scale = packet.get("transform_scale")
	var source_rect = packet.get("source_rect")
	if not texture is Texture2D or not origin is Vector2 or not scale is Vector2 or not source_rect is Rect2 or not is_finite(rotation) or not is_finite(ppm) or ppm <= 0.0 or not is_finite(alpha):
		return false
	canvas.draw_set_transform(origin, rotation, scale)
	canvas.draw_texture_rect_region(texture, source_rect, source_rect, Color(1.0, 1.0, 1.0, clampf(alpha, 0.25, 1.0)), false, true)
	canvas.draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
	return true

func _valid_context(snapshot: Dictionary, scene_data: Dictionary, current_key: String, structure_visual) -> bool:
	if snapshot.get("worldId") != "return_station" or snapshot.get("sceneId") != "rs_core_room" or scene_data.get("worldId") != "return_station" or scene_data.get("sceneId") != "rs_core_room":
		return false
	var epoch = snapshot.get("worldEpoch")
	if (typeof(epoch) != TYPE_INT and typeof(epoch) != TYPE_FLOAT) or not is_finite(float(epoch)) or float(epoch) < 1.0 or floorf(float(epoch)) != float(epoch):
		return false
	if current_key != "return_station:rs_core_room:" + str(epoch) or structure_visual == null:
		return false
	if not structure_visual.has_valid_layout("return_station", "rs_core_room", scene_data):
		return false
	var selected: Array = structure_visual.selected_structure_ids("return_station", "rs_core_room", scene_data)
	return selected == STRUCTURE_IDS

func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var manifest_file = FileAccess.open(MODULES_RESOURCE, FileAccess.READ)
	if manifest_file == null or manifest_file.get_length() > 8192:
		return
	var manifest_bytes: PackedByteArray = manifest_file.get_buffer(manifest_file.get_length())
	manifest_file.close()
	var manifest_text: String = manifest_bytes.get_string_from_utf8()
	if manifest_text.to_utf8_buffer() != manifest_bytes:
		return
	var manifest = JSON.parse_string(manifest_text)
	if not _valid_manifest(manifest):
		return
	var image_file = FileAccess.open(MODULE_TEXTURE_PATH, FileAccess.READ)
	if image_file == null or image_file.get_length() != MODULE_BYTES:
		return
	var png_bytes: PackedByteArray = image_file.get_buffer(image_file.get_length())
	image_file.close()
	var hash: HashingContext = HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256) != OK or hash.update(png_bytes) != OK or hash.finish().hex_encode() != MODULE_SHA256:
		return
	var image: Image = Image.new()
	if image.load_png_from_buffer(png_bytes) != OK or image.get_width() != 1418 or image.get_height() != 1028 or image.get_format() != Image.FORMAT_RGBA8:
		return
	if _painted_bounds(image) != PAINTED_BOUNDS or image.get_pixel(int(FOOT_A.x), int(FOOT_A.y)).a <= 0.0 or image.get_pixel(int(FOOT_B.x), int(FOOT_B.y)).a <= 0.0:
		return
	var clipped: Image = image.duplicate()
	var source_axis: Vector2 = FOOT_B - FOOT_A
	if not is_finite(source_axis.x) or source_axis.x <= 0.0:
		return
	for y in range(FRAME.position.y, FRAME.end.y):
		for x in range(FRAME.position.x, FRAME.end.x):
			var along: float = (float(x) + 0.5 - FOOT_A.x) / source_axis.x
			if along >= 0.5:
				clipped.set_pixel(x, y, Color(0.0, 0.0, 0.0, 0.0))
	var clipped_bounds: Rect2i = _painted_bounds(clipped)
	if clipped_bounds.size.x <= 0 or clipped_bounds.size.y <= 0:
		return
	terminal_painted_bounds = clipped_bounds
	module_texture = ImageTexture.create_from_image(image)
	terminal_texture = ImageTexture.create_from_image(clipped)

func _valid_manifest(value) -> bool:
	if not value is Dictionary or not _keys_are(value, ["schema", "registeredSourceReuse", "approval", "GodotDisplay", "publicRelease", "sourceAsset", "asset", "installation"]):
		return false
	if value.get("schema") != "wuxian.rs-bulkhead-modules.internal-use/v1" or value.get("registeredSourceReuse") != true or value.get("approval") != "INTERNAL_REGISTERED_SOURCE_REUSE" or value.get("GodotDisplay") != "NOT_RUN" or value.get("publicRelease") != "NOT_ESTABLISHED":
		return false
	var source = value.get("sourceAsset")
	var asset = value.get("asset")
	var installation = value.get("installation")
	if not source is Dictionary or not asset is Dictionary or not installation is Dictionary:
		return false
	if not _keys_are(source, ["path", "bytes", "sha256", "canvas", "format"]) or source.get("path") != SOURCE_PATH or source.get("bytes") != SOURCE_BYTES or source.get("sha256") != MODULE_SHA256 or not _number_array_equals(source.get("canvas"), [1418, 1028]) or source.get("format") != "RGBA8":
		return false
	if not _keys_are(asset, ["path", "bytes", "sha256", "format", "canvas", "frame", "paintedBounds", "groundFeetPx"]):
		return false
	if asset.get("path") != MODULE_TEXTURE_PATH or asset.get("bytes") != MODULE_BYTES or asset.get("sha256") != MODULE_SHA256 or asset.get("format") != "RGBA8" or not _number_array_equals(asset.get("canvas"), [1418, 1028]) or not _number_array_equals(asset.get("frame"), [0, 0, 355, 480]) or not _number_array_equals(asset.get("paintedBounds"), [41, 6, 299, 457]) or not _number_array_equals(asset.get("groundFeetPx"), [[50, 322], [300, 460]]):
		return false
	if not _keys_are(installation, ["worldId", "sceneId", "runIds", "cadenceM", "terminalM", "maxModules", "maxExtentPxAt48", "maxLateralFootprintM"]):
		return false
	return installation.get("worldId") == "return_station" and installation.get("sceneId") == "rs_core_room" and installation.get("runIds") == BULKHEAD_IDS and installation.get("cadenceM") == 1.0 and installation.get("terminalM") == 0.5 and installation.get("maxModules") == MAX_MODULES and installation.get("maxExtentPxAt48") == MAX_EXTENT_PX_AT_48 and installation.get("maxLateralFootprintM") == 0.5

func _keys_are(value: Dictionary, expected: Array) -> bool:
	var actual: Array = value.keys()
	var wanted: Array = expected.duplicate()
	actual.sort()
	wanted.sort()
	return actual == wanted

func _number_array_equals(value, expected: Array) -> bool:
	if not value is Array or value.size() != expected.size():
		return false
	for index in range(expected.size()):
		if expected[index] is Array:
			if not _number_array_equals(value[index], expected[index]):
				return false
		elif (typeof(value[index]) != TYPE_INT and typeof(value[index]) != TYPE_FLOAT) or not is_finite(float(value[index])) or float(value[index]) != float(expected[index]):
			return false
	return true

func _painted_bounds(image: Image) -> Rect2i:
	var min_x: int = FRAME.end.x
	var min_y: int = FRAME.end.y
	var max_x: int = -1
	var max_y: int = -1
	for y in range(FRAME.position.y, FRAME.end.y):
		for x in range(FRAME.position.x, FRAME.end.x):
			if image.get_pixel(x, y).a > 0.0:
				min_x = mini(min_x, x)
				min_y = mini(min_y, y)
				max_x = maxi(max_x, x)
				max_y = maxi(max_y, y)
	if max_x < min_x or max_y < min_y:
		return Rect2i()
	return Rect2i(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)

func _source_to_screen(point: Vector2, rotation: float, scale: Vector2) -> Vector2:
	return Vector2(point.x * scale.x, point.y * scale.y).rotated(rotation)

func _transformed_bounds(origin: Vector2, rotation: float, scale: Vector2, source_bounds: Rect2i) -> Rect2:
	var points: Array[Vector2] = []
	var right: float = float(source_bounds.position.x + source_bounds.size.x)
	var bottom: float = float(source_bounds.position.y + source_bounds.size.y)
	for point in [Vector2(source_bounds.position), Vector2(right, source_bounds.position.y), Vector2(right, bottom), Vector2(source_bounds.position.x, bottom)]:
		points.append(origin + _source_to_screen(point, rotation, scale))
	var min_point: Vector2 = points[0]
	var max_point: Vector2 = points[0]
	for point in points:
		min_point = Vector2(minf(min_point.x, point.x), minf(min_point.y, point.y))
		max_point = Vector2(maxf(max_point.x, point.x), maxf(max_point.y, point.y))
	return Rect2(min_point, max_point - min_point)

func _rect_intersects(a: Rect2, b: Rect2) -> bool:
	if a.size.x <= 0.0 or a.size.y <= 0.0 or b.size.x <= 0.0 or b.size.y <= 0.0:
		return false
	return a.intersects(b)

func _is_legacy_bulkhead_packet(packet_id) -> bool:
	if typeof(packet_id) != TYPE_STRING:
		return false
	for structure_id in BULKHEAD_IDS:
		for prefix in ["rs:top:" + structure_id, "rs:top-edge:" + structure_id, "rs:inset:" + structure_id, "rs:key-edge:" + structure_id, "rs:seam:" + structure_id, "rs:front:" + structure_id, "rs:fascia:" + structure_id]:
			if packet_id == prefix or packet_id.begins_with(prefix + ":"):
				return true
	return false
