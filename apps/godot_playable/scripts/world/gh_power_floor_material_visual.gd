extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const ProjectionScript = preload("res://scripts/world/projection.gd")
const MAP_RESOURCE = "res://data/gh_power_floor_material.internal-use.json"
const SOURCE_RESOURCE = "res://assets/props/grey_hive/floor_tiles.png"
const SOURCE_BYTES = 1315059
const SOURCE_SHA256 = "4c93e192505e71d6ec4e414aeb5c9b76376b7407d677017b31b8eba5e24e744f"
const CANVAS_SIZE = Vector2(1434.0, 980.0)
const WORLD_ID = "grey_hive"
const SCENE_ID = "gh_power_room"
const GATE_SCENE_ID = "gh_gate_a"
const GATE_MAP_RESOURCE = "res://data/gh_gate_a_floor_material.internal-use.json"
const ROW_ID = "approved_floor_atlas"
const ASSET_ID = "runtime2d.grey_hive.floor_tiles.v1"
const MAX_PPM = 256.0
const PIXEL_CENTERS = [[187, 184], [345, 305], [185, 426], [27, 305]]
const WALLS = [
	["gh_power_wall_north", [[0, 0], [20, 0], [20, 0.5], [0, 0.5]]],
	["gh_power_wall_south", [[0, 13.5], [20, 13.5], [20, 14], [0, 14]]],
	["gh_power_wall_west_north", [[0, 0.5], [0.5, 0.5], [0.5, 5.5], [0, 5.5]]],
	["gh_power_wall_west_south", [[0, 8.5], [0.5, 8.5], [0.5, 13.5], [0, 13.5]]],
	["gh_power_wall_east_north", [[19.5, 0.5], [20, 0.5], [20, 5.5], [19.5, 5.5]]],
	["gh_power_wall_east_south", [[19.5, 8.5], [20, 8.5], [20, 13.5], [19.5, 13.5]]]
]

var definition_attempted_by_scene: Dictionary = {}
var definition_valid_by_scene: Dictionary = {}
var texture_attempted: bool = false
var floor_texture: ImageTexture

func is_definition_valid(value) -> bool:
	if not value is Dictionary or not value.get("installation") is Dictionary:
		return false
	var profile: Dictionary = _profile_for_scene(value.installation.get("sceneId"))
	return not profile.is_empty() and _definition_matches(value, profile)

func _definition_matches(value, profile: Dictionary) -> bool:
	if not value is Dictionary or not _keys_are(value, ["schema", "registeredSourceReuse", "approval", "GodotDisplay", "publicRelease", "asset", "selection", "installation"]):
		return false
	if typeof(value.get("schema")) != TYPE_STRING or value.get("schema") != profile.schema:
		return false
	if typeof(value.get("registeredSourceReuse")) != TYPE_BOOL or value.get("registeredSourceReuse") != true:
		return false
	if typeof(value.get("approval")) != TYPE_STRING or value.get("approval") != "INTERNAL_REGISTERED_SOURCE_REUSE":
		return false
	if typeof(value.get("GodotDisplay")) != TYPE_STRING or value.get("GodotDisplay") != "NOT_RUN":
		return false
	if typeof(value.get("publicRelease")) != TYPE_STRING or value.get("publicRelease") != "NOT_ESTABLISHED":
		return false
	var asset = value.get("asset")
	var selection = value.get("selection")
	var installation = value.get("installation")
	if not asset is Dictionary or not selection is Dictionary or not installation is Dictionary:
		return false
	if not _keys_are(asset, ["assetId", "path", "bytes", "sha256", "canvas", "format"]):
		return false
	if typeof(asset.get("assetId")) != TYPE_STRING or asset.get("assetId") != ASSET_ID:
		return false
	if typeof(asset.get("path")) != TYPE_STRING or asset.get("path") != SOURCE_RESOURCE:
		return false
	if not _int_equals(asset.get("bytes"), SOURCE_BYTES) or typeof(asset.get("sha256")) != TYPE_STRING or asset.get("sha256") != SOURCE_SHA256:
		return false
	if typeof(asset.get("format")) != TYPE_STRING or asset.get("format") != "RGBA8" or not _numbers_equal(asset.get("canvas"), [1434, 980]):
		return false
	if not _keys_are(selection, ["kind", "pixelCenterCorners", "uvConvention"]):
		return false
	if typeof(selection.get("kind")) != TYPE_STRING or selection.get("kind") != "affine_top_face":
		return false
	if typeof(selection.get("uvConvention")) != TYPE_STRING or selection.get("uvConvention") != "(sourcePixelCenter + 0.5) / canvasSize":
		return false
	if not _numbers_equal(selection.get("pixelCenterCorners"), PIXEL_CENTERS):
		return false
	if not _keys_are(installation, ["kind", "worldId", "sceneId", "rowId", "layer", "position", "boundsM", "groundY", "patchSizeM", "grid", "patchCount", "tint"]):
		return false
	if typeof(installation.get("kind")) != TYPE_STRING or installation.get("kind") != "visual_only_ground_material":
		return false
	if typeof(installation.get("worldId")) != TYPE_STRING or installation.get("worldId") != WORLD_ID:
		return false
	if typeof(installation.get("sceneId")) != TYPE_STRING or installation.get("sceneId") != profile.sceneId:
		return false
	if typeof(installation.get("rowId")) != TYPE_STRING or installation.get("rowId") != ROW_ID:
		return false
	if typeof(installation.get("layer")) != TYPE_STRING or installation.get("layer") != "visual.floor":
		return false
	if not _numbers_equal(installation.get("position"), [10, 0, 7]) or not _numbers_equal(installation.get("boundsM"), [0, 0, 20, 14]):
		return false
	if not P.number(installation.get("groundY")) or float(installation.get("groundY")) != 0.0:
		return false
	if not _numbers_equal(installation.get("patchSizeM"), [2, 2]) or not _numbers_equal(installation.get("grid"), [10, 7]):
		return false
	if not _int_equals(installation.get("patchCount"), 70) or not _numbers_equal(installation.get("tint"), [1, 1, 1, 1]):
		return false
	return true

func build_packet(snapshot, scene_data, current_key, projection) -> Dictionary:
	if not _valid_context(snapshot, scene_data, current_key, projection):
		return {}
	_load_definition_once(snapshot.sceneId)
	if not definition_valid_by_scene.get(snapshot.sceneId, false):
		return {}
	return _build_canonical_packet(snapshot, scene_data, current_key, projection)

func is_packet_valid(packet, snapshot, scene_data, current_key, projection) -> bool:
	if not _valid_context(snapshot, scene_data, current_key, projection) or not _packet_shape_is_native(packet):
		return false
	_load_definition_once(snapshot.sceneId)
	if not definition_valid_by_scene.get(snapshot.sceneId, false):
		return false
	var expected := _build_canonical_packet(snapshot, scene_data, current_key, projection)
	return not expected.is_empty() and packet == expected

func visible_patches(packet, snapshot, scene_data, current_key, projection, viewport) -> Array:
	if not _valid_rect(viewport) or not is_packet_valid(packet, snapshot, scene_data, current_key, projection):
		return []
	var visible: Array = []
	for patch in packet.patches:
		if _rect_intersects_inclusive(patch.screenBounds, viewport):
			visible.append(patch)
	return visible

func decode_source(bytes):
	if typeof(bytes) != TYPE_PACKED_BYTE_ARRAY or bytes.size() != SOURCE_BYTES:
		return null
	var hash := HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256) != OK or hash.update(bytes) != OK or hash.finish().hex_encode() != SOURCE_SHA256:
		return null
	var image := Image.new()
	if image.load_png_from_buffer(bytes) != OK or image.get_width() != 1434 or image.get_height() != 980 or image.get_format() != Image.FORMAT_RGBA8:
		return null
	return image

func draw_packet(canvas, packet, snapshot, scene_data, current_key, projection, viewport) -> bool:
	if not canvas is CanvasItem or not _valid_rect(viewport) or not is_packet_valid(packet, snapshot, scene_data, current_key, projection):
		return false
	var visible: Array = []
	for patch in packet.patches:
		if _rect_intersects_inclusive(patch.screenBounds, viewport):
			visible.append(patch)
	if visible.is_empty():
		return true
	if not _load_texture_once():
		return false
	for patch in visible:
		canvas.draw_polygon(patch.screenPoints, patch.tint, patch.uvs, floor_texture)
	return true

func _valid_context(snapshot, scene_data, current_key, projection) -> bool:
	if not _valid_projection(projection) or not P.snapshot(snapshot):
		return false
	if typeof(snapshot.get("worldId")) != TYPE_STRING or snapshot.get("worldId") != WORLD_ID:
		return false
	if typeof(snapshot.get("sceneId")) != TYPE_STRING or _profile_for_scene(snapshot.get("sceneId")).is_empty():
		return false
	var epoch = snapshot.get("worldEpoch")
	if not P.counter(epoch, 1):
		return false
	if typeof(current_key) != TYPE_STRING or current_key != WORLD_ID + ":" + snapshot.sceneId + ":" + str(epoch):
		return false
	if not scene_data is Dictionary or typeof(scene_data.get("worldId")) != TYPE_STRING or scene_data.get("worldId") != WORLD_ID:
		return false
	if typeof(scene_data.get("sceneId")) != TYPE_STRING or scene_data.get("sceneId") != snapshot.sceneId or not _valid_scene_layout(scene_data, _profile_for_scene(snapshot.sceneId)):
		return false
	var presentation = scene_data.get("presentation")
	if not presentation is Dictionary:
		return false
	var sprites = presentation.get("sprites")
	if not P.rows(sprites, 64) or not P.unique(sprites, "id"):
		return false
	var candidates: Array = []
	for row in sprites:
		if not row is Dictionary or not P.ident(row.get("id")) or not P.ident(row.get("assetId")) or not P.ident(row.get("layer")):
			return false
		var position = row.get("position")
		if not position is Array or position.size() != 3:
			return false
		for component in position:
			if not P.number(component):
				return false
		if row.get("id") == ROW_ID or row.get("assetId") == ASSET_ID:
			candidates.append(row)
	return candidates.size() == 1 and _matches_authored_row(candidates[0])

func _profile_for_scene(scene_id) -> Dictionary:
	if typeof(scene_id) != TYPE_STRING:
		return {}
	if scene_id == SCENE_ID:
		return {"sceneId": SCENE_ID, "mapResource": MAP_RESOURCE, "schema": "wuxian.gh-power-room-floor.internal-use/v1", "packetKind": "gh_power_room_floor_v1", "patchPrefix": "gh_power_floor", "wallIds": ["gh_power_wall_north", "gh_power_wall_south", "gh_power_wall_west_north", "gh_power_wall_west_south", "gh_power_wall_east_north", "gh_power_wall_east_south"]}
	if scene_id == GATE_SCENE_ID:
		return {"sceneId": GATE_SCENE_ID, "mapResource": GATE_MAP_RESOURCE, "schema": "wuxian.gh-gate-a-floor.internal-use/v1", "packetKind": "gh_gate_a_floor_v1", "patchPrefix": "gh_gate_a_floor", "wallIds": ["gh_gate_wall_north", "gh_gate_wall_south", "gh_gate_wall_west_north", "gh_gate_wall_west_south", "gh_gate_wall_east_north", "gh_gate_wall_east_south"]}
	return {}

func _valid_projection(projection) -> bool:
	if projection == null or not projection is RefCounted or not is_instance_valid(projection):
		return false
	if projection.get_script() != ProjectionScript:
		return false
	var origin = projection.get("origin")
	var center = projection.get("screen_center")
	var viewport = projection.get("viewport")
	var ppm = projection.get("ppm")
	if not origin is Vector3 or not _finite_vector3(origin):
		return false
	if not center is Vector2 or not _finite_vector2(center):
		return false
	if not viewport is Vector2 or not _finite_vector2(viewport) or viewport.x <= 0.0 or viewport.y <= 0.0:
		return false
	return P.number(ppm, 0.000001, MAX_PPM)

func _valid_scene_layout(scene_data: Dictionary, profile: Dictionary) -> bool:
	if not P.counter(scene_data.get("schemaVersion"), 1, 1):
		return false
	var bounds = scene_data.get("boundsM")
	if not bounds is Dictionary or not _keys_are(bounds, ["depth", "width", "x", "z"]):
		return false
	if not _numbers_equal([bounds.get("x"), bounds.get("z"), bounds.get("width"), bounds.get("depth")], [0, 0, 20, 14]):
		return false
	var collision = scene_data.get("collision")
	if not collision is Array or collision.size() != (7 if profile.sceneId == GATE_SCENE_ID else 6) or not P.unique(collision, "id"):
		return false
	for index in range(WALLS.size()):
		var row = collision[index]
		if not row is Dictionary or not _keys_are(row, ["id", "polygon"]):
			return false
		if typeof(row.get("id")) != TYPE_STRING or row.get("id") != profile.wallIds[index]:
			return false
		if not _numbers_equal(row.get("polygon"), WALLS[index][1]):
			return false
	if profile.sceneId == GATE_SCENE_ID:
		var barrier = collision[6]
		if not barrier is Dictionary or not _keys_are(barrier, ["id", "polygon", "requiresEvent"]):
			return false
		if typeof(barrier.get("id")) != TYPE_STRING or barrier.id != "gh_gate_a_closed_barrier":
			return false
		if typeof(barrier.get("requiresEvent")) != TYPE_STRING or barrier.requiresEvent != "hive_power":
			return false
		if not _numbers_equal(barrier.get("polygon"), [[9.5, 0.5], [10.5, 0.5], [10.5, 13.5], [9.5, 13.5]]):
			return false
	return true

func _matches_authored_row(row) -> bool:
	if not row is Dictionary or not _keys_are(row, ["assetId", "id", "layer", "position"]):
		return false
	if typeof(row.get("id")) != TYPE_STRING or row.get("id") != ROW_ID:
		return false
	if typeof(row.get("assetId")) != TYPE_STRING or row.get("assetId") != ASSET_ID:
		return false
	if typeof(row.get("layer")) != TYPE_STRING or row.get("layer") != "visual.floor":
		return false
	return _numbers_equal(row.get("position"), [10, 0, 7])

func _load_definition_once(scene_id: String) -> void:
	if definition_attempted_by_scene.get(scene_id, false):
		return
	definition_attempted_by_scene[scene_id] = true
	definition_valid_by_scene[scene_id] = false
	var profile: Dictionary = _profile_for_scene(scene_id)
	if profile.is_empty():
		return
	var file = FileAccess.open(profile.mapResource, FileAccess.READ)
	if file == null or file.get_length() <= 0 or file.get_length() > 16384:
		return
	var bytes: PackedByteArray = file.get_buffer(file.get_length())
	file.close()
	var text: String = bytes.get_string_from_utf8()
	if text.to_utf8_buffer() != bytes:
		return
	definition_valid_by_scene[scene_id] = _definition_matches(JSON.parse_string(text), profile)

func _load_texture_once() -> bool:
	if texture_attempted:
		return floor_texture != null
	texture_attempted = true
	var file = FileAccess.open(SOURCE_RESOURCE, FileAccess.READ)
	if file == null or file.get_length() != SOURCE_BYTES:
		return false
	var bytes: PackedByteArray = file.get_buffer(file.get_length())
	file.close()
	var image = decode_source(bytes)
	if image == null:
		return false
	floor_texture = ImageTexture.create_from_image(image)
	if floor_texture == null or floor_texture.get_width() != 1434 or floor_texture.get_height() != 980:
		floor_texture = null
	return floor_texture != null

func _build_canonical_packet(snapshot: Dictionary, scene_data: Dictionary, current_key: String, projection) -> Dictionary:
	var profile: Dictionary = _profile_for_scene(snapshot.sceneId)
	var patches: Array = []
	var uvs := _uv_corners()
	for row in range(7):
		for column in range(10):
			var x := float(column * 2)
			var z := float(row * 2)
			var corners: Array[Vector3] = [
				Vector3(x, 0.0, z),
				Vector3(x + 2.0, 0.0, z),
				Vector3(x + 2.0, 0.0, z + 2.0),
				Vector3(x, 0.0, z + 2.0)
			]
			var points := PackedVector2Array()
			for corner in corners:
				var point = projection.call("project", corner)
				if not point is Vector2 or not _finite_vector2(point):
					return {}
				points.append(point)
			if not _convex_quad(points):
				return {}
			var bounds := _points_bounds(points)
			if not _valid_rect(bounds):
				return {}
			patches.append({
				"id": "%s_r%d_c%d" % [profile.patchPrefix, row, column],
				"row": row,
				"column": column,
				"worldCorners": corners,
				"screenPoints": points,
				"screenBounds": bounds,
				"uvs": uvs.duplicate(),
				"tint": PackedColorArray([Color.WHITE, Color.WHITE, Color.WHITE, Color.WHITE])
			})
	if patches.size() != 70:
		return {}
	return {
		"kind": profile.packetKind,
		"worldId": WORLD_ID,
		"sceneId": profile.sceneId,
		"worldEpoch": snapshot.get("worldEpoch"),
		"currentKey": current_key,
		"ppm": projection.get("ppm"),
		"patches": patches
	}

func _packet_shape_is_native(packet) -> bool:
	if not packet is Dictionary or not _keys_are(packet, ["kind", "worldId", "sceneId", "worldEpoch", "currentKey", "ppm", "patches"]):
		return false
	if typeof(packet.get("kind")) != TYPE_STRING or packet.get("kind") != _profile_for_scene(packet.get("sceneId")).get("packetKind"):
		return false
	if typeof(packet.get("worldId")) != TYPE_STRING or packet.get("worldId") != WORLD_ID:
		return false
	if typeof(packet.get("sceneId")) != TYPE_STRING or _profile_for_scene(packet.get("sceneId")).is_empty():
		return false
	if not P.counter(packet.get("worldEpoch"), 1) or typeof(packet.get("currentKey")) != TYPE_STRING:
		return false
	if not P.number(packet.get("ppm"), 0.000001, MAX_PPM):
		return false
	var patches = packet.get("patches")
	if not patches is Array or patches.size() != 70:
		return false
	for index in range(70):
		var patch = patches[index]
		if not patch is Dictionary or not _keys_are(patch, ["id", "row", "column", "worldCorners", "screenPoints", "screenBounds", "uvs", "tint"]):
			return false
		if typeof(patch.get("id")) != TYPE_STRING or typeof(patch.get("row")) != TYPE_INT or typeof(patch.get("column")) != TYPE_INT:
			return false
		if patch.get("row") != int(index / 10) or patch.get("column") != index % 10:
			return false
		var corners = patch.get("worldCorners")
		if not corners is Array or corners.size() != 4:
			return false
		for corner in corners:
			if not corner is Vector3 or not _finite_vector3(corner):
				return false
		var points = patch.get("screenPoints")
		if not points is PackedVector2Array or points.size() != 4 or not _convex_quad(points):
			return false
		if not patch.get("screenBounds") is Rect2 or not _valid_rect(patch.get("screenBounds")):
			return false
		var uvs = patch.get("uvs")
		if not uvs is PackedVector2Array or uvs.size() != 4 or not _finite_points(uvs):
			return false
		var tint = patch.get("tint")
		if not tint is PackedColorArray or tint.size() != 4:
			return false
		for color in tint:
			if not _finite_color(color):
				return false
	return true

func _uv_corners() -> PackedVector2Array:
	var result := PackedVector2Array()
	for pixel in PIXEL_CENTERS:
		result.append((Vector2(float(pixel[0]), float(pixel[1])) + Vector2(0.5, 0.5)) / CANVAS_SIZE)
	return result

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
	var direction := 0.0
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
	var minimum: Vector2 = points[0]
	var maximum: Vector2 = points[0]
	for point in points:
		minimum = Vector2(minf(minimum.x, point.x), minf(minimum.y, point.y))
		maximum = Vector2(maxf(maximum.x, point.x), maxf(maximum.y, point.y))
	return Rect2(minimum, maximum - minimum)

func _rect_intersects_inclusive(first: Rect2, second: Rect2) -> bool:
	return first.end.x >= second.position.x and first.position.x <= second.end.x and first.end.y >= second.position.y and first.position.y <= second.end.y

func _finite_points(points: PackedVector2Array) -> bool:
	for point in points:
		if not _finite_vector2(point):
			return false
	return true

func _finite_vector2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _finite_vector3(value: Vector3) -> bool:
	return is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _finite_color(value: Color) -> bool:
	return is_finite(value.r) and is_finite(value.g) and is_finite(value.b) and is_finite(value.a)

func _valid_rect(value) -> bool:
	if not value is Rect2 or not _finite_vector2(value.position) or not _finite_vector2(value.size):
		return false
	return value.size.x > 0.0 and value.size.y > 0.0 and is_finite(value.end.x) and is_finite(value.end.y)
