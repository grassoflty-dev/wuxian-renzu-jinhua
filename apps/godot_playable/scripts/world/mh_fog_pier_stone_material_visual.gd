extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const GroundScript = preload("res://scripts/world/mh_fog_pier_ground_visual.gd")
const ProjectionScript = preload("res://scripts/world/projection.gd")

const PACKET_KIND = "mh_fog_pier_stone_material_v1"
const WORLD_ID = "mist_harbor"
const SCENE_ID = "mh_fog_pier"
const DEFINITION_RESOURCE = "res://data/mh_fog_pier_stone_material.internal-use.json"
const TEXTURE_RESOURCE = "res://assets/props/mist_harbor/fog_pier_stone_source.png"
const ASSET_ID = "runtime2d.world.mistharbor.water_edge_stone.v1"
const OUTPUT_BYTES = 1983603
const OUTPUT_SHA256 = "e823434b3fbc420fc14270200f81e0a459e594597304b1323d336e532a924f9d"
const CANVAS_WIDTH = 1228
const CANVAS_HEIGHT = 1254
const MAX_DEFINITION_BYTES = 16384
const COLUMN_COUNT = 23
const ROW_COUNT = 5
const PATCH_COUNT = 115
const PATCH_TINT = Color(0.55, 0.67, 0.80, 1.0)
const PATCH_CORNER_UVS = [
	Vector2(820.0 / 1228.0, 240.0 / 1254.0),
	Vector2(1032.0 / 1228.0, 342.0 / 1254.0),
	Vector2(392.0 / 1228.0, 627.0 / 1254.0),
	Vector2(180.0 / 1228.0, 525.0 / 1254.0)
]
const TOP_LEVEL_KEYS = ["schema", "registeredSourceReuse", "approval", "GodotDisplay", "publicRelease", "asset", "selection", "installation"]
const ASSET_KEYS = ["assetId", "derivedRelative", "sourceRelative", "sourceSha256", "path", "bytes", "sha256", "canvas", "format"]
const SELECTION_KEYS = ["kind", "sourcePixelEdgeCorners", "uvConvention"]
const INSTALLATION_KEYS = ["kind", "worldId", "sceneId", "rowId", "layer", "position", "boundsM", "walkableId", "walkablePolygon", "groundY", "patchSizeM", "grid", "patchCount", "tint"]
const PACKET_KEYS = ["kind", "worldId", "sceneId", "worldEpoch", "currentKey", "ppm", "patches"]
const PATCH_KEYS = ["id", "row", "column", "worldCorners", "screenPoints", "screenBounds", "uvs", "tint"]

var _ground_gate: RefCounted
var _definition_attempted: bool = false
var _definition: Dictionary = {}
var _texture_attempted: bool = false
var _cached_texture: ImageTexture = null

func _init() -> void:
	_ground_gate = GroundScript.new()

func _finite_number(value) -> bool:
	return (typeof(value) == TYPE_INT or typeof(value) == TYPE_FLOAT) and is_finite(float(value))

func _same_number(value, expected: float) -> bool:
	return _finite_number(value) and P.number(value, expected, expected)

func _finite_vector2(value) -> bool:
	return typeof(value) == TYPE_VECTOR2 and is_finite(value.x) and is_finite(value.y)

func _finite_vector3(value) -> bool:
	return typeof(value) == TYPE_VECTOR3 and is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _finite_rect(value, positive: bool = false) -> bool:
	if typeof(value) != TYPE_RECT2:
		return false
	var rect: Rect2 = value
	if not _finite_vector2(rect.position) or not _finite_vector2(rect.size):
		return false
	return rect.size.x > 0.0 and rect.size.y > 0.0 if positive else rect.size.x >= 0.0 and rect.size.y >= 0.0

func _finite_color(value) -> bool:
	return typeof(value) == TYPE_COLOR and is_finite(value.r) and is_finite(value.g) and is_finite(value.b) and is_finite(value.a)

func _exact_keys(value: Dictionary, keys: Array) -> bool:
	if value.size() != keys.size():
		return false
	for key in keys:
		if not value.has(key):
			return false
	return true

func _numbers_match(value, expected: Array) -> bool:
	if typeof(value) != TYPE_ARRAY or value.size() != expected.size():
		return false
	for index in range(expected.size()):
		if not _same_number(value[index], float(expected[index])):
			return false
	return true

func _number_matrix_match(value, expected: Array) -> bool:
	if typeof(value) != TYPE_ARRAY or value.size() != expected.size():
		return false
	for row_index in range(expected.size()):
		if not _numbers_match(value[row_index], expected[row_index]):
			return false
	return true

func is_definition_valid(value) -> bool:
	if typeof(value) != TYPE_DICTIONARY or not _exact_keys(value, TOP_LEVEL_KEYS):
		return false
	if typeof(value.get("registeredSourceReuse")) != TYPE_BOOL or value.registeredSourceReuse != true:
		return false
	if typeof(value.get("schema")) != TYPE_STRING or value.schema != "wuxian.mh-fog-pier-stone-material.internal-use/v1":
		return false
	if value.get("approval") != "INTERNAL_REGISTERED_SOURCE_REUSE" or value.get("GodotDisplay") != "NOT_RUN" or value.get("publicRelease") != "NOT_ESTABLISHED":
		return false
	var asset = value.get("asset")
	if typeof(asset) != TYPE_DICTIONARY or not _exact_keys(asset, ASSET_KEYS):
		return false
	if asset.get("assetId") != ASSET_ID or asset.get("derivedRelative") != "assets/derived/asset-pipeline-v1/sprites/png/runtime2d.world.mistharbor.water_edge_stone.v1.png":
		return false
	if asset.get("sourceRelative") != "batch07/02_runtime2d.world.mistharbor.water_edge_stone.v1.png" or asset.get("sourceSha256") != "dc09452bf3f55c57cca1a93bb09f169ef441af90321dff47d91c4be7e62d3a83":
		return false
	if asset.get("path") != TEXTURE_RESOURCE or not _same_number(asset.get("bytes"), float(OUTPUT_BYTES)):
		return false
	if asset.get("sha256") != OUTPUT_SHA256 or not _numbers_match(asset.get("canvas"), [CANVAS_WIDTH, CANVAS_HEIGHT]) or asset.get("format") != "RGBA8":
		return false
	var selection = value.get("selection")
	if typeof(selection) != TYPE_DICTIONARY or not _exact_keys(selection, SELECTION_KEYS):
		return false
	if selection.get("kind") != "affine_top_plane" or selection.get("uvConvention") != "sourcePixelEdge / canvasSize":
		return false
	if not _number_matrix_match(selection.get("sourcePixelEdgeCorners"), [[820, 240], [1032, 342], [392, 627], [180, 525]]):
		return false
	var installation = value.get("installation")
	if typeof(installation) != TYPE_DICTIONARY or not _exact_keys(installation, INSTALLATION_KEYS):
		return false
	if installation.get("kind") != "visual_only_ground_material" or installation.get("worldId") != WORLD_ID or installation.get("sceneId") != SCENE_ID:
		return false
	if installation.get("rowId") != "approved_stone_water_module" or installation.get("layer") != "visual.floor":
		return false
	if not _numbers_match(installation.get("position"), [12, 0, 8]) or not _numbers_match(installation.get("boundsM"), [0, 0, 24, 16]):
		return false
	if installation.get("walkableId") != "mh_fog_pier_walkable_area":
		return false
	if not _number_matrix_match(installation.get("walkablePolygon"), [[0.5, 0.5], [23.5, 0.5], [23.5, 15.5], [0.5, 15.5]]):
		return false
	if not _same_number(installation.get("groundY"), 0.0) or not _numbers_match(installation.get("patchSizeM"), [1, 3]):
		return false
	if not _numbers_match(installation.get("grid"), [COLUMN_COUNT, ROW_COUNT]) or not _same_number(installation.get("patchCount"), float(PATCH_COUNT)):
		return false
	if not _numbers_match(installation.get("tint"), [0.55, 0.67, 0.80, 1.0]):
		return false
	return true

func _valid_projection(projection) -> bool:
	if not projection is RefCounted or projection.get_script() != ProjectionScript:
		return false
	return P.number(projection.get("ppm"), 0.000001, 256.0)

func _valid_context(snapshot, scene, current_key, projection) -> bool:
	if not _valid_projection(projection):
		return false
	if _ground_gate == null or not is_instance_valid(_ground_gate) or _ground_gate.get_script() != GroundScript:
		return false
	var gate_packet = _ground_gate.call("build_packet", snapshot, scene, current_key, projection)
	if typeof(gate_packet) != TYPE_DICTIONARY or gate_packet.get("kind") != "mh_fog_pier_ground_v1":
		return false
	var cells = gate_packet.get("cells")
	return typeof(cells) == TYPE_ARRAY and cells.size() == GroundScript.CELL_COUNT

func _load_definition_after_context() -> Dictionary:
	if _definition_attempted:
		return _definition
	_definition_attempted = true
	var file := FileAccess.open(DEFINITION_RESOURCE, FileAccess.READ)
	if file == null:
		return {}
	var length: int = file.get_length()
	if length <= 0 or length > MAX_DEFINITION_BYTES:
		file.close()
		return {}
	var raw_bytes: PackedByteArray = file.get_buffer(length)
	file.close()
	if raw_bytes.size() != length:
		return {}
	var text: String = raw_bytes.get_string_from_utf8()
	if text.to_utf8_buffer() != raw_bytes:
		return {}
	var parser := JSON.new()
	if parser.parse(text) != OK or typeof(parser.data) != TYPE_DICTIONARY:
		return {}
	if not is_definition_valid(parser.data):
		return {}
	_definition = parser.data
	return _definition

func _project_point(projection, point: Vector3):
	var screen = projection.call("project", point)
	return screen if _finite_vector2(screen) else null

func _patch_id(row: int, column: int) -> String:
	return "mh_fog_pier_stone_%02d_%02d" % [row, column]

func _source_uv(corner_index: int) -> Vector2:
	return PATCH_CORNER_UVS[corner_index]

func _build_patches(projection) -> Array:
	var patches: Array = []
	for row in range(ROW_COUNT):
		var z1: float = 0.5 + 3.0 * float(row)
		var z2: float = z1 + 3.0
		for column in range(COLUMN_COUNT):
			var x1: float = 0.5 + float(column)
			var x2: float = x1 + 1.0
			var world_corners: Array = [Vector3(x1, 0.0, z1), Vector3(x2, 0.0, z1), Vector3(x2, 0.0, z2), Vector3(x1, 0.0, z2)]
			var screen_points: Array = []
			var uvs: Array = []
			var screen_min: Vector2
			var screen_max: Vector2
			for corner_index in range(4):
				var screen = _project_point(projection, world_corners[corner_index])
				if screen == null:
					return []
				screen_points.append(screen)
				uvs.append(_source_uv(corner_index))
				if corner_index == 0:
					screen_min = screen
					screen_max = screen
				else:
					screen_min = screen_min.min(screen)
					screen_max = screen_max.max(screen)
			var screen_bounds := Rect2(screen_min, screen_max - screen_min)
			if not _finite_rect(screen_bounds, true):
				return []
			patches.append({
				"id": _patch_id(row, column),
				"row": row,
				"column": column,
				"worldCorners": world_corners,
				"screenPoints": screen_points,
				"screenBounds": screen_bounds,
				"uvs": uvs,
				"tint": [PATCH_TINT, PATCH_TINT, PATCH_TINT, PATCH_TINT]
			})
	return patches if patches.size() == PATCH_COUNT else []

func build_packet(snapshot, scene, current_key, projection) -> Dictionary:
	if not _valid_context(snapshot, scene, current_key, projection):
		return {}
	if _load_definition_after_context().is_empty():
		return {}
	var patches: Array = _build_patches(projection)
	if patches.size() != PATCH_COUNT:
		return {}
	return {
		"kind": PACKET_KIND,
		"worldId": WORLD_ID,
		"sceneId": SCENE_ID,
		"worldEpoch": snapshot.get("worldEpoch"),
		"currentKey": current_key,
		"ppm": float(projection.get("ppm")),
		"patches": patches
	}

func _patch_shape_valid(patch) -> bool:
	if typeof(patch) != TYPE_DICTIONARY or not _exact_keys(patch, PATCH_KEYS):
		return false
	if typeof(patch.get("id")) != TYPE_STRING or typeof(patch.get("row")) != TYPE_INT or typeof(patch.get("column")) != TYPE_INT:
		return false
	if typeof(patch.get("worldCorners")) != TYPE_ARRAY or patch.worldCorners.size() != 4:
		return false
	if typeof(patch.get("screenPoints")) != TYPE_ARRAY or patch.screenPoints.size() != 4:
		return false
	if typeof(patch.get("uvs")) != TYPE_ARRAY or patch.uvs.size() != 4:
		return false
	if typeof(patch.get("tint")) != TYPE_ARRAY or patch.tint.size() != 4 or not _finite_rect(patch.get("screenBounds"), true):
		return false
	for point in patch.worldCorners:
		if not _finite_vector3(point):
			return false
	for point in patch.screenPoints:
		if not _finite_vector2(point):
			return false
	for uv in patch.uvs:
		if not _finite_vector2(uv):
			return false
	for color in patch.tint:
		if not _finite_color(color):
			return false
	return true

func is_packet_valid(packet, snapshot, scene, current_key, projection) -> bool:
	if not _valid_context(snapshot, scene, current_key, projection):
		return false
	if _load_definition_after_context().is_empty():
		return false
	if typeof(packet) != TYPE_DICTIONARY or not _exact_keys(packet, PACKET_KEYS):
		return false
	if packet.get("kind") != PACKET_KIND or packet.get("worldId") != WORLD_ID or packet.get("sceneId") != SCENE_ID:
		return false
	if typeof(packet.get("worldEpoch")) != typeof(snapshot.get("worldEpoch")) or packet.get("worldEpoch") != snapshot.get("worldEpoch"):
		return false
	if typeof(packet.get("currentKey")) != TYPE_STRING or packet.get("currentKey") != current_key:
		return false
	var ppm: float = float(projection.get("ppm"))
	if not P.number(packet.get("ppm"), ppm, ppm):
		return false
	var patches = packet.get("patches")
	if typeof(patches) != TYPE_ARRAY or patches.size() != PATCH_COUNT:
		return false
	var expected_patches: Array = _build_patches(projection)
	if expected_patches.size() != PATCH_COUNT:
		return false
	for index in range(PATCH_COUNT):
		var patch = patches[index]
		if not _patch_shape_valid(patch) or patch != expected_patches[index]:
			return false
	return true

func _sha256(bytes: PackedByteArray) -> String:
	var hashing := HashingContext.new()
	if hashing.start(HashingContext.HASH_SHA256) != OK or hashing.update(bytes) != OK:
		return ""
	return hashing.finish().hex_encode().to_lower()

func decode_source(bytes) -> Image:
	if typeof(bytes) != TYPE_PACKED_BYTE_ARRAY or bytes.size() != OUTPUT_BYTES:
		return null
	if _sha256(bytes) != OUTPUT_SHA256:
		return null
	var image := Image.new()
	if image.load_png_from_buffer(bytes) != OK:
		return null
	if image.get_width() != CANVAS_WIDTH or image.get_height() != CANVAS_HEIGHT or image.get_format() != Image.FORMAT_RGBA8:
		return null
	return image

func _load_texture_after_visible_validation() -> Texture2D:
	if _texture_attempted:
		return _cached_texture
	_texture_attempted = true
	var file := FileAccess.open(TEXTURE_RESOURCE, FileAccess.READ)
	if file == null:
		return null
	if file.get_length() != OUTPUT_BYTES:
		file.close()
		return null
	var bytes: PackedByteArray = file.get_buffer(OUTPUT_BYTES)
	file.close()
	if bytes.size() != OUTPUT_BYTES:
		return null
	var image: Image = decode_source(bytes)
	if image == null:
		return null
	_cached_texture = ImageTexture.create_from_image(image)
	return _cached_texture

func draw_packet(canvas, packet, snapshot, scene, current_key, projection, viewport: Rect2) -> bool:
	if not canvas is CanvasItem:
		return false
	if not _finite_rect(viewport, true) or not is_packet_valid(packet, snapshot, scene, current_key, projection):
		return false
	var visible_patches: Array = []
	for patch in packet.patches:
		var bounds: Rect2 = patch.screenBounds
		if bounds.intersects(viewport):
			visible_patches.append(patch)
	if visible_patches.is_empty():
		return true
	var texture: Texture2D = _load_texture_after_visible_validation()
	if texture == null:
		return false
	var drawing_canvas: CanvasItem = canvas
	for patch in visible_patches:
		var points := PackedVector2Array(patch.screenPoints)
		var colors := PackedColorArray(patch.tint)
		var uvs := PackedVector2Array(patch.uvs)
		drawing_canvas.draw_polygon(points, colors, uvs, texture)
	return true