extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const ProjectionScript = preload("res://scripts/world/projection.gd")
const DEFINITION_RESOURCE = "res://data/cw_pressure_floor_material.internal-use.json"
const TEXTURE_RESOURCE = "res://assets/props/clockworks/pressure_hall_steel_floor_source.png"
const WORLD_ID = "clockworks"
const SCENE_ID = "cw_pressure_hall"
const ROW_ID = "cw_pressure_hall_steel_floor"
const ASSET_ID = "runtime2d.world.clockworks.floor_steel.v1"
const DEFINITION_BYTES_MAX = 16384
const SOURCE_BYTES = 1610704
const SOURCE_SHA256 = "0b4555021799ac849756e0f11c207d4e9309df19f4b8f68d0f4e8177f5204043"
const SOURCE_SIZE = Vector2i(1197, 1132)
const PATCH_SIZE = 2.0
const GRID = Vector2i(12, 8)
const SOURCE_PIXEL_EDGE_CORNERS = [[510, 200], [970, 460], [600, 740], [140, 480]]
const EXPECTED_COLLISION = [
	{"id": "cw_pressure_hall_wall_north", "polygon": [[0, 0], [24, 0], [24, 0.5], [0, 0.5]]},
	{"id": "cw_pressure_hall_wall_south", "polygon": [[0, 15.5], [24, 15.5], [24, 16], [0, 16]]},
	{"id": "cw_pressure_hall_wall_west_north", "polygon": [[0, 0.5], [0.5, 0.5], [0.5, 6], [0, 6]]},
	{"id": "cw_pressure_hall_wall_west_south", "polygon": [[0, 9.5], [0.5, 9.5], [0.5, 15.5], [0, 15.5]]},
	{"id": "cw_pressure_hall_wall_east_north", "polygon": [[23.5, 0.5], [24, 0.5], [24, 6], [23.5, 6]]},
	{"id": "cw_pressure_hall_wall_east_south", "polygon": [[23.5, 9.5], [24, 9.5], [24, 15.5], [23.5, 15.5]]}
]

var _definition_checked: bool = false
var _definition_valid: bool = false
var _definition: Dictionary = {}
var _floor_texture: ImageTexture

func is_definition_valid(value) -> bool:
	return _definition_matches(value)

func build_packet(snapshot, scene: Dictionary, current_key: String, projection) -> Dictionary:
	if not _valid_context(snapshot, scene, current_key, projection):
		return {}
	if not _load_definition():
		return {}
	return _build_valid_packet(snapshot, current_key, projection)

func is_packet_valid(packet, snapshot, scene: Dictionary, current_key: String, projection) -> bool:
	if not _valid_context(snapshot, scene, current_key, projection):
		return false
	if not _packet_shape_valid(packet):
		return false
	if not _load_definition():
		return false
	var expected := _build_valid_packet(snapshot, current_key, projection)
	return not expected.is_empty() and packet == expected

func visible_patches(packet, snapshot, scene: Dictionary, current_key: String, projection, viewport) -> Array:
	if not _valid_rect(viewport) or not is_packet_valid(packet, snapshot, scene, current_key, projection):
		return []
	var visible: Array = []
	for patch in packet.patches:
		if patch.screenBounds.intersects(viewport):
			visible.append(patch)
	return visible

func decode_source(bytes):
	if not bytes is PackedByteArray or bytes.size() != SOURCE_BYTES:
		return null
	var hash := HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256) != OK or hash.update(bytes) != OK:
		return null
	if hash.finish().hex_encode() != SOURCE_SHA256:
		return null
	var image := Image.new()
	if image.load_png_from_buffer(bytes) != OK:
		return null
	if image.get_width() != SOURCE_SIZE.x or image.get_height() != SOURCE_SIZE.y or image.get_format() != Image.FORMAT_RGBA8:
		return null
	return image

func draw_packet(canvas, packet, snapshot, scene: Dictionary, current_key: String, projection, viewport) -> bool:
	if not canvas is CanvasItem or not _valid_rect(viewport):
		return false
	if not is_packet_valid(packet, snapshot, scene, current_key, projection):
		return false
	var visible := visible_patches(packet, snapshot, scene, current_key, projection, viewport)
	if visible.is_empty():
		return true
	var texture := _load_texture()
	if texture == null:
		return false
	for patch in visible:
		canvas.draw_polygon(patch.screenPoints, patch.tint, patch.uvs, texture)
	return true

func _valid_context(snapshot, scene: Dictionary, current_key: String, projection) -> bool:
	if not P.context(snapshot) or not P.counter(snapshot.get("worldEpoch"), 1):
		return false
	if snapshot.get("worldId") != WORLD_ID or snapshot.get("sceneId") != SCENE_ID:
		return false
	if current_key != WORLD_ID + ":" + SCENE_ID + ":" + str(snapshot.get("worldEpoch")):
		return false
	if typeof(scene.get("worldId")) != TYPE_STRING or scene.get("worldId") != WORLD_ID:
		return false
	if typeof(scene.get("sceneId")) != TYPE_STRING or scene.get("sceneId") != SCENE_ID:
		return false
	if not _valid_projection(projection):
		return false
	return _valid_scene_layout(scene)

func _valid_projection(projection) -> bool:
	if projection == null or not projection is RefCounted or projection.get_script() != ProjectionScript:
		return false
	var ppm = projection.get("ppm")
	var origin = projection.get("origin")
	var center = projection.get("screen_center")
	return P.number(ppm, 0.0, 256.0) and float(ppm) > 0.0 and _finite_vector3(origin) and _finite_vector2(center)

func _valid_scene_layout(scene: Dictionary) -> bool:
	var bounds = scene.get("boundsM")
	if not bounds is Dictionary or not _keys_are(bounds, ["x", "z", "width", "depth"]):
		return false
	if not _number_equals(bounds.get("x"), 0.0) or not _number_equals(bounds.get("z"), 0.0):
		return false
	if not _number_equals(bounds.get("width"), 24.0) or not _number_equals(bounds.get("depth"), 16.0):
		return false
	var collision = scene.get("collision")
	if not collision is Array or collision.size() != EXPECTED_COLLISION.size() or not P.unique(collision, "id"):
		return false
	for expected in EXPECTED_COLLISION:
		var found = false
		for row in collision:
			if row.get("id") == expected.id:
				if not _keys_are(row, ["id", "polygon"]) or not _number_tree_equals(row.get("polygon"), expected.polygon):
					return false
				found = true
				break
		if not found:
			return false
	var presentation = scene.get("presentation")
	if not presentation is Dictionary:
		return false
	var sprites = presentation.get("sprites")
	if not sprites is Array or not P.rows(sprites, 2048) or not P.unique(sprites, "id"):
		return false
	var candidates: Array = []
	for row in sprites:
		if row.get("id") == ROW_ID or row.get("assetId") == ASSET_ID:
			candidates.append(row)
	return candidates.size() == 1 and _registered_row_matches(candidates[0])

func _registered_row_matches(row) -> bool:
	if not row is Dictionary or not _keys_are(row, ["id", "assetId", "layer", "position"]):
		return false
	if typeof(row.get("id")) != TYPE_STRING or row.get("id") != ROW_ID:
		return false
	if typeof(row.get("assetId")) != TYPE_STRING or row.get("assetId") != ASSET_ID:
		return false
	if typeof(row.get("layer")) != TYPE_STRING or row.get("layer") != "visual.floor":
		return false
	return _number_tree_equals(row.get("position"), [12, 0, 8])

func _load_definition() -> bool:
	if _definition_checked:
		return _definition_valid
	_definition_checked = true
	var file := FileAccess.open(DEFINITION_RESOURCE, FileAccess.READ)
	if file == null or file.get_length() <= 0 or file.get_length() > DEFINITION_BYTES_MAX:
		return false
	var bytes := file.get_buffer(file.get_length())
	file.close()
	var text := bytes.get_string_from_utf8()
	if text.to_utf8_buffer() != bytes:
		return false
	var parsed = JSON.parse_string(text)
	if not _definition_matches(parsed):
		return false
	_definition = parsed
	_definition_valid = true
	return true

func _definition_matches(value) -> bool:
	if not value is Dictionary or not _keys_are(value, ["schema", "registeredSourceReuse", "approval", "GodotDisplay", "publicRelease", "asset", "selection", "installation"]):
		return false
	if not _string_equals(value.get("schema"), "wuxian.cw-pressure-floor-material.internal-use/v1"):
		return false
	if typeof(value.get("registeredSourceReuse")) != TYPE_BOOL or value.get("registeredSourceReuse") != true:
		return false
	if not _string_equals(value.get("approval"), "INTERNAL_REGISTERED_SOURCE_REUSE"):
		return false
	if not _string_equals(value.get("GodotDisplay"), "NOT_RUN") or not _string_equals(value.get("publicRelease"), "NOT_ESTABLISHED"):
		return false
	var asset = value.get("asset")
	if not asset is Dictionary or not _keys_are(asset, ["assetId", "derivedRelative", "sourceRelative", "sourceSha256", "path", "bytes", "sha256", "canvas", "format"]):
		return false
	if not _string_equals(asset.get("assetId"), ASSET_ID):
		return false
	if not _string_equals(asset.get("derivedRelative"), "assets/derived/asset-pipeline-v1/sprites/png/runtime2d.world.clockworks.floor_steel.v1.png"):
		return false
	if not _string_equals(asset.get("sourceRelative"), "batch07/05_runtime2d.world.clockworks.floor_steel.v1.png"):
		return false
	if not _string_equals(asset.get("sourceSha256"), "1a01e851654d9f5c98b5486cbe489f357ed99c55ffc601c492af63044d90b190"):
		return false
	if not _string_equals(asset.get("path"), TEXTURE_RESOURCE) or not _number_equals(asset.get("bytes"), SOURCE_BYTES):
		return false
	if not _string_equals(asset.get("sha256"), SOURCE_SHA256) or not _number_tree_equals(asset.get("canvas"), [1197, 1132]):
		return false
	if not _string_equals(asset.get("format"), "RGBA8"):
		return false
	var selection = value.get("selection")
	if not selection is Dictionary or not _keys_are(selection, ["kind", "sourcePixelEdgeCorners", "uvConvention"]):
		return false
	if not _string_equals(selection.get("kind"), "affine_top_plane"):
		return false
	if not _number_tree_equals(selection.get("sourcePixelEdgeCorners"), SOURCE_PIXEL_EDGE_CORNERS):
		return false
	if not _string_equals(selection.get("uvConvention"), "sourcePixelEdge / standaloneCanvasSize"):
		return false
	var installation = value.get("installation")
	if not installation is Dictionary or not _keys_are(installation, ["kind", "worldId", "sceneId", "rowId", "layer", "position", "boundsM", "groundY", "patchSizeM", "grid", "patchCount", "tint"]):
		return false
	if not _string_equals(installation.get("kind"), "visual_only_ground_material") or not _string_equals(installation.get("worldId"), WORLD_ID):
		return false
	if not _string_equals(installation.get("sceneId"), SCENE_ID) or not _string_equals(installation.get("rowId"), ROW_ID):
		return false
	if not _string_equals(installation.get("layer"), "visual.floor") or not _number_tree_equals(installation.get("position"), [12, 0, 8]):
		return false
	if not _number_tree_equals(installation.get("boundsM"), [0, 0, 24, 16]) or not _number_equals(installation.get("groundY"), 0):
		return false
	if not _number_tree_equals(installation.get("patchSizeM"), [2, 2]) or not _number_tree_equals(installation.get("grid"), [12, 8]):
		return false
	if not _number_equals(installation.get("patchCount"), 96) or not _number_tree_equals(installation.get("tint"), [1, 1, 1, 1]):
		return false
	return true

func _build_valid_packet(snapshot: Dictionary, current_key: String, projection) -> Dictionary:
	var patches: Array = []
	var uvs := _source_uvs()
	var tint := PackedColorArray([Color.WHITE, Color.WHITE, Color.WHITE, Color.WHITE])
	for row in range(GRID.y):
		for column in range(GRID.x):
			var x := float(column) * PATCH_SIZE
			var z := float(row) * PATCH_SIZE
			var corners: Array = [
				Vector3(x, 0.0, z),
				Vector3(x + PATCH_SIZE, 0.0, z),
				Vector3(x + PATCH_SIZE, 0.0, z + PATCH_SIZE),
				Vector3(x, 0.0, z + PATCH_SIZE)
			]
			var screen := PackedVector2Array()
			for corner in corners:
				var point = projection.call("project", corner)
				if not point is Vector2 or not _finite_vector2(point):
					return {}
				screen.append(point)
			var screen_bounds := _points_bounds(screen)
			if not _valid_rect(screen_bounds):
				return {}
			patches.append({
				"id": "cw_pressure_hall_steel_r%d_c%d" % [row, column],
				"row": row,
				"column": column,
				"worldCorners": corners,
				"screenPoints": screen,
				"screenBounds": screen_bounds,
				"uvs": uvs.duplicate(),
				"tint": tint.duplicate()
			})
	if patches.size() != 96:
		return {}
	return {
		"kind": "cw_pressure_hall_steel_floor_v1",
		"worldId": WORLD_ID,
		"sceneId": SCENE_ID,
		"worldEpoch": snapshot.get("worldEpoch"),
		"currentKey": current_key,
		"ppm": float(projection.get("ppm")),
		"patches": patches
	}

func _packet_shape_valid(packet) -> bool:
	if not packet is Dictionary or not _keys_are(packet, ["kind", "worldId", "sceneId", "worldEpoch", "currentKey", "ppm", "patches"]):
		return false
	if not _string_equals(packet.get("kind"), "cw_pressure_hall_steel_floor_v1"):
		return false
	if not _string_equals(packet.get("worldId"), WORLD_ID) or not _string_equals(packet.get("sceneId"), SCENE_ID):
		return false
	if not P.counter(packet.get("worldEpoch"), 1) or not _string_equals(packet.get("currentKey"), WORLD_ID + ":" + SCENE_ID + ":" + str(packet.get("worldEpoch"))):
		return false
	if not P.number(packet.get("ppm"), 0.0, 256.0) or float(packet.get("ppm")) <= 0.0:
		return false
	if not packet.get("patches") is Array or packet.patches.size() != GRID.x * GRID.y:
		return false
	for patch in packet.patches:
		if not patch is Dictionary or not _keys_are(patch, ["id", "row", "column", "worldCorners", "screenPoints", "screenBounds", "uvs", "tint"]):
			return false
		if typeof(patch.get("id")) != TYPE_STRING or typeof(patch.get("row")) != TYPE_INT or typeof(patch.get("column")) != TYPE_INT:
			return false
		if not patch.get("worldCorners") is Array or patch.worldCorners.size() != 4:
			return false
		for corner in patch.worldCorners:
			if not _finite_vector3(corner):
				return false
		if not patch.get("screenPoints") is PackedVector2Array or patch.screenPoints.size() != 4 or not _finite_points(patch.screenPoints):
			return false
		if not _valid_rect(patch.get("screenBounds")):
			return false
		if not patch.get("uvs") is PackedVector2Array or patch.uvs.size() != 4 or not _finite_points(patch.uvs):
			return false
		if not patch.get("tint") is PackedColorArray or patch.tint.size() != 4:
			return false
		for color in patch.tint:
			if not color is Color or not is_finite(color.r) or not is_finite(color.g) or not is_finite(color.b) or not is_finite(color.a):
				return false
	return true

func _source_uvs() -> PackedVector2Array:
	var result := PackedVector2Array()
	for pixel in SOURCE_PIXEL_EDGE_CORNERS:
		result.append(Vector2(float(pixel[0]) / float(SOURCE_SIZE.x), float(pixel[1]) / float(SOURCE_SIZE.y)))
	return result

func _points_bounds(points: PackedVector2Array) -> Rect2:
	if points.is_empty():
		return Rect2()
	var minimum := points[0]
	var maximum := points[0]
	for point in points:
		minimum = minimum.min(point)
		maximum = maximum.max(point)
	return Rect2(minimum, maximum - minimum)

func _valid_rect(value) -> bool:
	return value is Rect2 and _finite_vector2(value.position) and _finite_vector2(value.size) and value.size.x > 0.0 and value.size.y > 0.0

func _finite_vector2(value) -> bool:
	return value is Vector2 and is_finite(value.x) and is_finite(value.y)

func _finite_vector3(value) -> bool:
	return value is Vector3 and is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _finite_points(points: PackedVector2Array) -> bool:
	for point in points:
		if not _finite_vector2(point):
			return false
	return true

func _number_equals(value, expected: float) -> bool:
	return P.number(value) and float(value) == expected

func _number_tree_equals(value, expected) -> bool:
	if expected is Array:
		if not value is Array or value.size() != expected.size():
			return false
		for index in range(expected.size()):
			if not _number_tree_equals(value[index], expected[index]):
				return false
		return true
	return P.number(value) and float(value) == float(expected)

func _keys_are(value: Dictionary, expected: Array) -> bool:
	if value.size() != expected.size():
		return false
	for key in expected:
		if not value.has(key):
			return false
	return true

func _string_equals(value, expected: String) -> bool:
	return typeof(value) == TYPE_STRING and value == expected

func _load_texture() -> ImageTexture:
	if _floor_texture != null:
		return _floor_texture
	var file := FileAccess.open(TEXTURE_RESOURCE, FileAccess.READ)
	if file == null or file.get_length() != SOURCE_BYTES:
		return null
	var bytes := file.get_buffer(SOURCE_BYTES)
	file.close()
	var image = decode_source(bytes)
	if image == null:
		return null
	_floor_texture = ImageTexture.create_from_image(image)
	if _floor_texture == null or _floor_texture.get_width() != SOURCE_SIZE.x or _floor_texture.get_height() != SOURCE_SIZE.y:
		_floor_texture = null
	return _floor_texture
