extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const MAPPING_RESOURCE = "res://data/gh_bio_floor_material.internal-use.json"
const TEXTURE_RESOURCE = "res://assets/props/grey_hive/floor_tiles.png"
const SOURCE_PATH = "assets/derived/asset-pipeline-v1/sprites/png/runtime2d.grey_hive.floor_tiles.v1.png"
const ASSET_ID = "runtime2d.grey_hive.floor_tiles.v1"
const ROW_ID = "approved_floor"
const OUTPUT_BYTES = 1315059
const OUTPUT_SHA256 = "4c93e192505e71d6ec4e414aeb5c9b76376b7407d677017b31b8eba5e24e744f"
const CANVAS_SIZE = Vector2i(1434, 980)
const CELL_SIZE = 2.0
const GRID_CELLS = Vector2i(12, 8)
const GRID_VERTICES = Vector2i(13, 9)
const MAX_PPM = 256.0
const PIXEL_CENTERS = [[187,184],[345,305],[185,426],[27,305]]
const CORNER_RGBA = [[61,55,59,252],[36,38,40,252],[59,62,64,253],[182,179,171,252]]
const UV_CONVENTION = "(sourcePixelCenter + 0.5) / canvasSize"

var attempted: bool = false
var floor_texture: ImageTexture
var last_lattice_vertex_count: int = 0
var last_lattice_vertices: Array[Vector2] = []

func build_packets(snapshot: Dictionary, scene_data: Dictionary, current_key: String, projection, wall_visual) -> Array:
	last_lattice_vertex_count = 0
	last_lattice_vertices.clear()
	if not _valid_context(snapshot, scene_data, current_key, wall_visual) or not _valid_projection(projection):
		return []
	_load_once()
	if floor_texture == null:
		return []
	var lattice: Array[Vector2] = []
	for z_index in range(GRID_VERTICES.y):
		for x_index in range(GRID_VERTICES.x):
			var world_point := Vector3(float(x_index) * CELL_SIZE, 0.0, float(z_index) * CELL_SIZE)
			var screen_point = projection.call("project", world_point)
			if not screen_point is Vector2 or not _finite_point(screen_point):
				return []
			lattice.append(screen_point)
	if lattice.size() != 117:
		return []
	last_lattice_vertices = lattice
	last_lattice_vertex_count = lattice.size()
	var uvs := _uv_corners()
	var packets: Array = []
	for z_index in range(GRID_CELLS.y):
		for x_index in range(GRID_CELLS.x):
			var top_left: int = z_index * GRID_VERTICES.x + x_index
			var points := PackedVector2Array([
				lattice[top_left],
				lattice[top_left + 1],
				lattice[top_left + GRID_VERTICES.x + 1],
				lattice[top_left + GRID_VERTICES.x]
			])
			if not _convex_quad(points):
				last_lattice_vertices.clear()
				last_lattice_vertex_count = 0
				return []
			var world_x: float = float(x_index) * CELL_SIZE
			var world_z: float = float(z_index) * CELL_SIZE
			var world_corners: Array[Vector3] = [
				Vector3(world_x, 0.0, world_z),
				Vector3(world_x + CELL_SIZE, 0.0, world_z),
				Vector3(world_x + CELL_SIZE, 0.0, world_z + CELL_SIZE),
				Vector3(world_x, 0.0, world_z + CELL_SIZE)
			]
			var bounds := _points_bounds(points)
			if not _valid_rect(bounds):
				last_lattice_vertices.clear()
				last_lattice_vertex_count = 0
				return []
			packets.append({
				"id": "gh_bio_floor:%02d:%02d" % [z_index, x_index],
				"grid_x": x_index,
				"grid_z": z_index,
				"world_corners": world_corners,
				"points": points,
				"uvs": uvs,
				"bounds_rect": bounds
			})
	if packets.size() != 96:
		last_lattice_vertices.clear()
		last_lattice_vertex_count = 0
		return []
	return packets

func draw_tiles(canvas: CanvasItem, packets: Array, viewport: Rect2) -> bool:
	if canvas == null or floor_texture == null or not _valid_rect(viewport) or packets.size() != 96:
		return false
	var seen: Dictionary = {}
	for packet in packets:
		if not _valid_packet(packet, seen):
			return false
	if seen.size() != 96:
		return false
	var colors := PackedColorArray([Color.WHITE, Color.WHITE, Color.WHITE, Color.WHITE])
	for packet in packets:
		var points: PackedVector2Array = packet.points
		if quad_intersects_viewport(points, viewport):
			canvas.draw_polygon(points, colors, packet.uvs, floor_texture)
	return true

func quad_intersects_viewport(points: PackedVector2Array, viewport: Rect2) -> bool:
	if points.size() != 4 or not _finite_points(points) or not _valid_rect(viewport):
		return false
	var bounds := _points_bounds(points)
	return _valid_rect(bounds) and bounds.intersects(viewport)

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
	return candidates.size() == 1 and _matches_authored_row(candidates[0])

func _matches_authored_row(row) -> bool:
	if not row is Dictionary or not _keys_are(row, ["assetId", "id", "layer", "position"]):
		return false
	if typeof(row.get("id")) != TYPE_STRING or typeof(row.get("assetId")) != TYPE_STRING or typeof(row.get("layer")) != TYPE_STRING:
		return false
	if row.get("id") != ROW_ID or row.get("assetId") != ASSET_ID or row.get("layer") != "visual.floor":
		return false
	return _numbers_equal(row.get("position"), [12, 0, 8])

func _valid_projection(projection) -> bool:
	if projection == null or not projection is Object or not projection.has_method("project"):
		return false
	return P.number(projection.get("ppm"), 0.000001, MAX_PPM)

func _load_once() -> void:
	if attempted:
		return
	attempted = true
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
	var hash := HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256) != OK or hash.update(png_bytes) != OK or hash.finish().hex_encode() != OUTPUT_SHA256:
		return
	var image := Image.new()
	if image.load_png_from_buffer(png_bytes) != OK or image.get_width() != CANVAS_SIZE.x or image.get_height() != CANVAS_SIZE.y or image.get_format() != Image.FORMAT_RGBA8:
		return
	for index in range(PIXEL_CENTERS.size()):
		var pixel: Color = image.get_pixel(PIXEL_CENTERS[index][0], PIXEL_CENTERS[index][1])
		var rgba := [roundi(pixel.r * 255.0), roundi(pixel.g * 255.0), roundi(pixel.b * 255.0), roundi(pixel.a * 255.0)]
		if rgba != CORNER_RGBA[index]:
			return
	floor_texture = ImageTexture.create_from_image(image)
	if floor_texture == null or floor_texture.get_width() != CANVAS_SIZE.x or floor_texture.get_height() != CANVAS_SIZE.y:
		floor_texture = null

func _valid_mapping(value) -> bool:
	if not value is Dictionary or not _keys_are(value, ["schema", "registeredSourceReuse", "approval", "consumerStatus", "publicRelease", "sourceAsset", "runtimeAsset", "selection", "installation"]):
		return false
	if typeof(value.get("schema")) != TYPE_STRING or value.get("schema") != "wuxian.gh-bio-floor-material.internal-use/v1" or typeof(value.get("registeredSourceReuse")) != TYPE_BOOL or value.get("registeredSourceReuse") != true:
		return false
	if typeof(value.get("approval")) != TYPE_STRING or value.get("approval") != "INTERNAL_REGISTERED_SOURCE_REUSE" or typeof(value.get("consumerStatus")) != TYPE_STRING or value.get("consumerStatus") != "NOT_RUN" or typeof(value.get("publicRelease")) != TYPE_STRING or value.get("publicRelease") != "NOT_ESTABLISHED":
		return false
	var source_asset = value.get("sourceAsset")
	var runtime_asset = value.get("runtimeAsset")
	var selection = value.get("selection")
	var installation = value.get("installation")
	if not source_asset is Dictionary or not runtime_asset is Dictionary or not selection is Dictionary or not installation is Dictionary:
		return false
	if not _keys_are(source_asset, ["path", "assetId", "bytes", "sha256", "canvas", "format", "admission", "runtimeStatus"]):
		return false
	if typeof(source_asset.get("path")) != TYPE_STRING or source_asset.get("path") != SOURCE_PATH or typeof(source_asset.get("assetId")) != TYPE_STRING or source_asset.get("assetId") != ASSET_ID or not _int_equals(source_asset.get("bytes"), OUTPUT_BYTES) or typeof(source_asset.get("sha256")) != TYPE_STRING or source_asset.get("sha256") != OUTPUT_SHA256 or typeof(source_asset.get("format")) != TYPE_STRING or source_asset.get("format") != "RGBA8" or typeof(source_asset.get("admission")) != TYPE_STRING or source_asset.get("admission") != "release_approved" or typeof(source_asset.get("runtimeStatus")) != TYPE_STRING or source_asset.get("runtimeStatus") != "ADOPT" or not _numbers_equal(source_asset.get("canvas"), [1434, 980]):
		return false
	if not _keys_are(runtime_asset, ["path", "bytes", "sha256", "canvas", "format"]):
		return false
	if typeof(runtime_asset.get("path")) != TYPE_STRING or runtime_asset.get("path") != TEXTURE_RESOURCE or not _int_equals(runtime_asset.get("bytes"), OUTPUT_BYTES) or typeof(runtime_asset.get("sha256")) != TYPE_STRING or runtime_asset.get("sha256") != OUTPUT_SHA256 or typeof(runtime_asset.get("format")) != TYPE_STRING or runtime_asset.get("format") != "RGBA8" or not _numbers_equal(runtime_asset.get("canvas"), [1434, 980]):
		return false
	if not _keys_are(selection, ["kind", "pixelCenterCorners", "uvConvention", "interiorAlphaRange"]):
		return false
	if typeof(selection.get("kind")) != TYPE_STRING or selection.get("kind") != "affine_top_face" or typeof(selection.get("uvConvention")) != TYPE_STRING or selection.get("uvConvention") != UV_CONVENTION or not _numbers_equal(selection.get("pixelCenterCorners"), PIXEL_CENTERS) or not _numbers_equal(selection.get("interiorAlphaRange"), [251, 254]):
		return false
	if not _keys_are(installation, ["installationKind", "worldId", "sceneId", "rowId", "assetId", "layer", "anchorM", "cellSizeM", "gridCells", "cellCount", "vertexGrid", "groundY", "orientation"]):
		return false
	if typeof(installation.get("installationKind")) != TYPE_STRING or installation.get("installationKind") != "visual-only-flat-ground" or typeof(installation.get("worldId")) != TYPE_STRING or installation.get("worldId") != "grey_hive" or typeof(installation.get("sceneId")) != TYPE_STRING or installation.get("sceneId") != "gh_bio_isolation" or typeof(installation.get("rowId")) != TYPE_STRING or installation.get("rowId") != ROW_ID or typeof(installation.get("assetId")) != TYPE_STRING or installation.get("assetId") != ASSET_ID or typeof(installation.get("layer")) != TYPE_STRING or installation.get("layer") != "visual.floor":
		return false
	if not _numbers_equal(installation.get("anchorM"), [12, 0, 8]) or not _numbers_equal(installation.get("cellSizeM"), [2, 2]) or not _numbers_equal(installation.get("gridCells"), [12, 8]) or not _int_equals(installation.get("cellCount"), 96) or not _numbers_equal(installation.get("vertexGrid"), [13, 9]) or not P.number(installation.get("groundY")) or float(installation.get("groundY")) != 0.0 or typeof(installation.get("orientation")) != TYPE_STRING or installation.get("orientation") != "A:x,z;B:x+2,z;C:x+2,z+2;D:x,z+2":
		return false
	return true

func _valid_packet(packet, seen: Dictionary) -> bool:
	if not packet is Dictionary or not _keys_are(packet, ["id", "grid_x", "grid_z", "world_corners", "points", "uvs", "bounds_rect"]):
		return false
	if typeof(packet.get("grid_x")) != TYPE_INT or typeof(packet.get("grid_z")) != TYPE_INT:
		return false
	var x_index: int = int(packet.grid_x)
	var z_index: int = int(packet.grid_z)
	if x_index < 0 or x_index >= GRID_CELLS.x or z_index < 0 or z_index >= GRID_CELLS.y:
		return false
	var cell_index: int = z_index * GRID_CELLS.x + x_index
	if seen.has(cell_index) or typeof(packet.get("id")) != TYPE_STRING or packet.get("id") != "gh_bio_floor:%02d:%02d" % [z_index, x_index]:
		return false
	var x: float = float(x_index) * CELL_SIZE
	var z: float = float(z_index) * CELL_SIZE
	var expected_world: Array[Vector3] = [
		Vector3(x, 0.0, z),
		Vector3(x + CELL_SIZE, 0.0, z),
		Vector3(x + CELL_SIZE, 0.0, z + CELL_SIZE),
		Vector3(x, 0.0, z + CELL_SIZE)
	]
	var world_corners = packet.get("world_corners")
	if not world_corners is Array or world_corners.size() != 4:
		return false
	for index in range(4):
		if not world_corners[index] is Vector3 or world_corners[index] != expected_world[index]:
			return false
	var points = packet.get("points")
	if not points is PackedVector2Array or points.size() != 4 or not _convex_quad(points):
		return false
	var uvs = packet.get("uvs")
	var expected_uvs := _uv_corners()
	if not uvs is PackedVector2Array or uvs.size() != 4:
		return false
	for index in range(4):
		if uvs[index] != expected_uvs[index]:
			return false
	var bounds = packet.get("bounds_rect")
	if not bounds is Rect2 or not _valid_rect(bounds) or bounds != _points_bounds(points):
		return false
	seen[cell_index] = true
	return true

func _uv_corners() -> PackedVector2Array:
	var points := PackedVector2Array()
	for pixel in PIXEL_CENTERS:
		points.append((Vector2(float(pixel[0]), float(pixel[1])) + Vector2(0.5, 0.5)) / Vector2(float(CANVAS_SIZE.x), float(CANVAS_SIZE.y)))
	return points

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
		elif not P.number(value[index]) or float(value[index]) != float(expected[index]):
			return false
	return true

func _int_equals(value, expected: int) -> bool:
	return P.counter(value, float(expected), float(expected))

func _convex_quad(points: PackedVector2Array) -> bool:
	if points.size() != 4 or not _finite_points(points):
		return false
	var direction: float = 0.0
	for index in range(4):
		var first: Vector2 = points[index]
		var second: Vector2 = points[(index + 1) % 4]
		var third: Vector2 = points[(index + 2) % 4]
		var cross: float = (second - first).cross(third - second)
		if not is_finite(cross) or absf(cross) <= 0.000001:
			return false
		if direction == 0.0:
			direction = signf(cross)
		elif signf(cross) != direction:
			return false
	return true

func _points_bounds(points: PackedVector2Array) -> Rect2:
	if points.is_empty():
		return Rect2()
	var minimum: Vector2 = points[0]
	var maximum: Vector2 = points[0]
	for point in points:
		minimum = Vector2(minf(minimum.x, point.x), minf(minimum.y, point.y))
		maximum = Vector2(maxf(maximum.x, point.x), maxf(maximum.y, point.y))
	return Rect2(minimum, maximum - minimum)

func _finite_points(points: PackedVector2Array) -> bool:
	for point in points:
		if not _finite_point(point):
			return false
	return true

func _finite_point(point: Vector2) -> bool:
	return is_finite(point.x) and is_finite(point.y)

func _valid_rect(rect: Rect2) -> bool:
	return _finite_point(rect.position) and _finite_point(rect.size) and rect.size.x > 0.0 and rect.size.y > 0.0 and is_finite(rect.end.x) and is_finite(rect.end.y)
