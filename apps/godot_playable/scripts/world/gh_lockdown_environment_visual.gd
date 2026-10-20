extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const PROJECTION_SCRIPT = preload("res://scripts/world/projection.gd")
const MAP_RESOURCE = "res://data/gh_lockdown_environment.internal-use.json"
const SCENE_RESOURCE = "res://data/scenes/gh_lockdown.json"
const MAP_BYTES = 4130
const MAP_SHA256 = "fac1c2f5fb7f6cd7fee9a19e5473106af837bf86132c3bf80fb588c6b85f50c2"
const SCENE_BYTES = 10081
const SCENE_SHA256 = "718489a1ea4ec4622ccdcefc8459a1dc1bd4f66ccf26abc813b6a06519197850"
const FLOOR_PATH = "res://assets/props/grey_hive/floor_tiles.png"
const FLOOR_BYTES = 1315059
const FLOOR_SHA256 = "4c93e192505e71d6ec4e414aeb5c9b76376b7407d677017b31b8eba5e24e744f"
const FLOOR_SIZE = Vector2i(1434, 980)
const PIPE_PATH = "res://assets/props/return_station/service_pipes.png"
const PIPE_BYTES = 1049893
const PIPE_SHA256 = "7f3c67879398e89adc9026fc2aca5fbe7792b1dc360b73c737c1b2299eac96da"
const PIPE_SIZE = Vector2i(1414, 1047)
const PIPE_FRAME = Rect2i(0, 219, 388, 270)
const PIPE_SEED = Vector2i(100, 320)
const PIPE_CONTACT_A = Vector2i(50, 345)
const PIPE_CONTACT_B = Vector2i(325, 482)
const PIPE_PIXELS = 46171
const PIPE_WORLD_A = Vector3(14.5, 0.0, 3.0)
const PIPE_WORLD_B = Vector3(15.5, 0.0, 3.0)
const PIPE_SPAN = 1.0
const PIPE_MAX_ROTATION = 0.025
const PIPE_MAX_EXTENT_AT_48 = 80.0
const PIPE_SHADOW_RADII = Vector2(0.6, 0.16)
const PIPE_SHADOW_ALPHA = 0.18
const CLUTTER_PATH = "res://assets/props/grey_hive/clutter.png"
const CLUTTER_BYTES = 1857722
const CLUTTER_SHA256 = "4743030e58286fdac5c4e119941608763d1ceb1d3bbed79ca1d7361ceab97ebe"
const CLUTTER_SIZE = Vector2i(1428, 1061)
const CLUTTER_FRAME = Rect2i(40, 716, 347, 287)
const CLUTTER_SEED = Vector2i(150, 800)
const CLUTTER_CONTACT_A = Vector2i(10, 213)
const CLUTTER_CONTACT_B = Vector2i(329, 182)
const CLUTTER_PIXELS = 73359
const CLUTTER_DISCONNECTED_PIXELS = 168
const CLUTTER_WORLD_A = Vector3(7.625, 0.0, 11.5)
const CLUTTER_WORLD_B = Vector3(8.375, 0.0, 10.5)
const CLUTTER_SPAN = 1.25
const CLUTTER_MAX_ROTATION = 0.06
const CLUTTER_MAX_EXTENT_AT_48 = 96.0
const CLUTTER_SHADOW_RADII = Vector2(0.5, 0.55)
const CLUTTER_SHADOW_ALPHA = 0.16
const ALARM_BODY_PATH = "res://assets/props/grey_hive/lockdown_alarm_body_source.png"
const ALARM_BODY_BYTES = 903994
const ALARM_BODY_SHA256 = "3cf5731678cb776eb63a62e910be26e815af46f273c76ac4d91fb1916cbce669"
const ALARM_BODY_SIZE = Vector2i(1448, 1086)
const MAX_PPM = 256.0
const SHADOW_STEPS = 24
const FLOOR_UV_CORNERS = [[187, 184], [345, 305], [185, 426], [27, 305]]
const WALL_IDS = [
	"lockdown_wall_north", "lockdown_wall_south",
	"lockdown_wall_west_north", "lockdown_wall_west_south",
	"lockdown_wall_east_north", "lockdown_wall_east_south"
]
const OUTER_WALL_EDGES: Dictionary = {
	"lockdown_wall_south": [Vector2(0.0, 16.0), Vector2(24.0, 16.0)],
	"lockdown_wall_east_north": [Vector2(24.0, 0.5), Vector2(24.0, 6.5)],
	"lockdown_wall_east_south": [Vector2(24.0, 9.5), Vector2(24.0, 15.5)]
}

var _static_attempted: bool = false
var _static_ready: bool = false
var _context_ready: bool = false
var _canonical_scene: Dictionary = {}
var _floor_texture: ImageTexture
var _pipe_texture: ImageTexture
var _clutter_texture: ImageTexture
var _alarm_body_texture: ImageTexture
var _status: Dictionary = {
	"scene": "NOT_READY", "map": "NOT_READY", "references": "NOT_READY",
	"floor": "NOT_READY", "pipe": "NOT_READY", "clutter": "NOT_READY",
	"alarm_body": "NOT_READY"
}

func prepare(snapshot, scene_data, current_key, scene_hash) -> void:
	_context_ready = false
	if not _valid_identity(snapshot, scene_data, current_key) or typeof(scene_hash) != TYPE_STRING or scene_hash != SCENE_SHA256:
		return
	if _static_attempted:
		_context_ready = _static_ready and _deep_equal(scene_data, _canonical_scene)
		return
	_static_attempted = true
	var scene_raw: PackedByteArray = _read_exact(SCENE_RESOURCE, SCENE_BYTES, SCENE_SHA256)
	if scene_raw.is_empty():
		_status["scene"] = "PIN_OR_READ_FAILURE"
		return
	var scene_text: String = scene_raw.get_string_from_utf8()
	if scene_text.to_utf8_buffer() != scene_raw:
		_status["scene"] = "UTF8_FAILURE"
		return
	var parsed_scene: Variant = JSON.parse_string(scene_text)
	if not parsed_scene is Dictionary:
		_status["scene"] = "JSON_TYPE_FAILURE"
		return
	var parsed_scene_dictionary: Dictionary = parsed_scene
	if not _deep_equal(scene_data, parsed_scene_dictionary):
		_status["scene"] = "COMMITTED_SCENE_MISMATCH"
		return
	_canonical_scene = parsed_scene_dictionary
	_status["scene"] = "PINNED_AND_MATCHED"
	var map_raw: PackedByteArray = _read_exact(MAP_RESOURCE, MAP_BYTES, MAP_SHA256)
	if map_raw.is_empty():
		_status["map"] = "PIN_OR_READ_FAILURE"
		return
	var map_text: String = map_raw.get_string_from_utf8()
	if map_text.to_utf8_buffer() != map_raw:
		_status["map"] = "UTF8_FAILURE"
		return
	var parsed_map: Variant = JSON.parse_string(map_text)
	if not _valid_map(parsed_map):
		_status["map"] = "SEALED_MAP_MISMATCH"
		return
	_status["map"] = "PINNED_AND_MATCHED"
	if not _reference_maps_valid():
		_status["references"] = "PIN_OR_READ_FAILURE"
		return
	_status["references"] = "PINNED"
	_static_ready = true
	_context_ready = true
	_load_floor()
	_load_pipe()
	_load_clutter()
	_load_alarm_body()

func readiness() -> Dictionary:
	return _status.duplicate(true)

func matches_authored_row(row) -> bool:
	if not row is Dictionary:
		return false
	var row_dictionary: Dictionary = row
	var row_id: Variant = row_dictionary.get("id")
	if typeof(row_id) == TYPE_STRING and row_id == "lockdown_service_cables":
		return _matches_row(row_dictionary, "lockdown_service_cables", "runtime2d.grey_hive.pipes_cables.v1", "visual.props_back", [15.0, 0.0, 3.0])
	if typeof(row_id) == TYPE_STRING and row_id == "lockdown_approved_clutter":
		return _matches_row(row_dictionary, "lockdown_approved_clutter", "runtime2d.grey_hive.clutter.v1", "visual.props_back", [8.0, 0.0, 11.0])
	return false

func draw_floor(canvas, snapshot, scene_data, current_key, projection, viewport) -> bool:
	if not _valid_canvas(canvas) or not _valid_context(snapshot, scene_data, current_key) or not _valid_projection(projection, viewport) or _floor_texture == null:
		return false
	var plans: Array = []
	var colors: PackedColorArray = PackedColorArray([Color(0.62, 0.66, 0.64, 1.0), Color(0.62, 0.66, 0.64, 1.0), Color(0.62, 0.66, 0.64, 1.0), Color(0.62, 0.66, 0.64, 1.0)])
	var uvs: PackedVector2Array = PackedVector2Array()
	for source_corner: Array in FLOOR_UV_CORNERS:
		var source_x: float = float(source_corner[0])
		var source_y: float = float(source_corner[1])
		var uv: Vector2 = Vector2((source_x + 0.5) / float(FLOOR_SIZE.x), (source_y + 0.5) / float(FLOOR_SIZE.y))
		if not _finite_point(uv) or uv.x < 0.0 or uv.x > 1.0 or uv.y < 0.0 or uv.y > 1.0:
			return false
		uvs.append(uv)
	if uvs.size() != 4:
		return false
	for row_index in range(8):
		for column_index in range(12):
			var x0: float = float(column_index) * 2.0
			var z0: float = float(row_index) * 2.0
			var x1: float = x0 + 2.0
			var z1: float = z0 + 2.0
			var points: PackedVector2Array = PackedVector2Array()
			var world_points: Array[Vector3] = [Vector3(x0, 0.0, z0), Vector3(x1, 0.0, z0), Vector3(x1, 0.0, z1), Vector3(x0, 0.0, z1)]
			for world_point: Vector3 in world_points:
				var projected_variant: Variant = projection.call("project", world_point)
				if not projected_variant is Vector2:
					return false
				var projected_point: Vector2 = projected_variant
				if not _finite_point(projected_point):
					return false
				points.append(projected_point)
			if points.size() != 4 or not _valid_points_bounds(points):
				return false
			plans.append(points)
	if plans.size() != 96:
		return false
	for patch_variant: Variant in plans:
		if not patch_variant is PackedVector2Array:
			return false
		var patch_points: PackedVector2Array = patch_variant
		canvas.draw_polygon(patch_points, colors, uvs, _floor_texture)
	return true

func draw_wall(canvas, snapshot, scene_data, current_key, wall, projection, viewport) -> bool:
	if not _valid_canvas(canvas) or not _valid_context(snapshot, scene_data, current_key) or not _valid_projection(projection, viewport) or not wall is Dictionary:
		return false
	var wall_dictionary: Dictionary = wall
	var wall_id: Variant = wall_dictionary.get("id")
	if typeof(wall_id) != TYPE_STRING or not WALL_IDS.has(wall_id):
		return false
	var canonical_wall: Dictionary = _find_row(_canonical_scene.get("collision"), "id", wall_id)
	if canonical_wall.is_empty() or not _deep_equal(wall_dictionary, canonical_wall):
		return false
	var spec: Dictionary = _wall_spec(wall_id)
	if spec.is_empty():
		return false
	var polygon_variant: Variant = wall_dictionary.get("polygon")
	if not polygon_variant is Array:
		return false
	var polygon: Array = polygon_variant
	if polygon.size() != 4:
		return false
	var height_variant: Variant = spec.get("height")
	var inner_a_variant: Variant = spec.get("inner_a")
	var inner_b_variant: Variant = spec.get("inner_b")
	var inset_variant: Variant = spec.get("inset")
	if not P.number(height_variant, 0.01, 1.0) or not inner_a_variant is Vector2 or not inner_b_variant is Vector2 or not inset_variant is Vector2:
		return false
	var height: float = float(height_variant)
	var inner_a: Vector2 = inner_a_variant
	var inner_b: Vector2 = inner_b_variant
	var inset: Vector2 = inset_variant
	if not _finite_point(inner_a) or not _finite_point(inner_b) or not _finite_point(inset):
		return false
	var ground_a_variant: Variant = projection.call("project", Vector3(inner_a.x, 0.0, inner_a.y))
	var ground_b_variant: Variant = projection.call("project", Vector3(inner_b.x, 0.0, inner_b.y))
	var raised_a_variant: Variant = projection.call("project", Vector3(inner_a.x, height, inner_a.y))
	var raised_b_variant: Variant = projection.call("project", Vector3(inner_b.x, height, inner_b.y))
	if not ground_a_variant is Vector2 or not ground_b_variant is Vector2 or not raised_a_variant is Vector2 or not raised_b_variant is Vector2:
		return false
	var ground_a: Vector2 = ground_a_variant
	var ground_b: Vector2 = ground_b_variant
	var raised_a: Vector2 = raised_a_variant
	var raised_b: Vector2 = raised_b_variant
	var contact_points: PackedVector2Array = PackedVector2Array()
	var contact_world_points: Array[Vector3] = [Vector3(inner_a.x + inset.x, 0.0, inner_a.y + inset.y), Vector3(inner_b.x + inset.x, 0.0, inner_b.y + inset.y), Vector3(inner_b.x, 0.0, inner_b.y), Vector3(inner_a.x, 0.0, inner_a.y)]
	for world_point: Vector3 in contact_world_points:
		var projected_variant: Variant = projection.call("project", world_point)
		if not projected_variant is Vector2:
			return false
		var projected_point: Vector2 = projected_variant
		if not _finite_point(projected_point):
			return false
		contact_points.append(projected_point)
	var face_points: PackedVector2Array = PackedVector2Array([ground_a, ground_b, raised_b, raised_a])
	var outer_face_points: PackedVector2Array = PackedVector2Array()
	var draw_outer_face: bool = false
	var edge_variant: Variant = OUTER_WALL_EDGES.get(wall_id)
	if edge_variant != null and _outer_wall_bindings_match(canvas, snapshot, scene_data, current_key, projection, viewport):
		if not edge_variant is Array or edge_variant.size() != 2:
			return false
		var edge_points: Array = edge_variant
		if not edge_points[0] is Vector2 or not edge_points[1] is Vector2:
			return false
		var outer_a: Vector2 = edge_points[0]
		var outer_b: Vector2 = edge_points[1]
		if not _finite_point(outer_a) or not _finite_point(outer_b):
			return false
		var outer_ground_a_variant: Variant = projection.call("project", Vector3(outer_a.x, 0.0, outer_a.y))
		var outer_ground_b_variant: Variant = projection.call("project", Vector3(outer_b.x, 0.0, outer_b.y))
		var outer_raised_a_variant: Variant = projection.call("project", Vector3(outer_a.x, height, outer_a.y))
		var outer_raised_b_variant: Variant = projection.call("project", Vector3(outer_b.x, height, outer_b.y))
		if not outer_ground_a_variant is Vector2 or not outer_ground_b_variant is Vector2 or not outer_raised_a_variant is Vector2 or not outer_raised_b_variant is Vector2:
			return false
		var outer_ground_a: Vector2 = outer_ground_a_variant
		var outer_ground_b: Vector2 = outer_ground_b_variant
		var outer_raised_a: Vector2 = outer_raised_a_variant
		var outer_raised_b: Vector2 = outer_raised_b_variant
		outer_face_points = PackedVector2Array([outer_ground_a, outer_ground_b, outer_raised_b, outer_raised_a])
		if not _valid_points_bounds(outer_face_points):
			return false
		draw_outer_face = true
	var cap_points: PackedVector2Array = PackedVector2Array()
	for point_variant: Variant in polygon:
		if not point_variant is Array or point_variant.size() != 2:
			return false
		var point_array: Array = point_variant
		if not P.number(point_array[0], 0.0, 24.0) or not P.number(point_array[1], 0.0, 16.0):
			return false
		var projected_cap_variant: Variant = projection.call("project", Vector3(float(point_array[0]), height, float(point_array[1])))
		if not projected_cap_variant is Vector2:
			return false
		var projected_cap: Vector2 = projected_cap_variant
		if not _finite_point(projected_cap):
			return false
		cap_points.append(projected_cap)
	if contact_points.size() != 4 or face_points.size() != 4 or cap_points.size() != 4 or not _valid_points_bounds(contact_points) or not _valid_points_bounds(face_points) or not _valid_points_bounds(cap_points):
		return false
	if draw_outer_face and (not _nonzero_polygon_area(contact_points) or not _nonzero_polygon_area(face_points) or not _nonzero_polygon_area(outer_face_points) or not _nonzero_polygon_area(cap_points)):
		return false
	canvas.draw_colored_polygon(contact_points, Color(0.025, 0.03, 0.03, 0.12))
	canvas.draw_colored_polygon(face_points, Color(0.15, 0.21, 0.22, 1.0))
	if draw_outer_face:
		canvas.draw_colored_polygon(outer_face_points, Color(0.15, 0.21, 0.22, 1.0))
	canvas.draw_colored_polygon(cap_points, Color(0.23, 0.29, 0.3, 1.0))
	if wall_id == "lockdown_wall_north":
		_draw_alarm_body(canvas, projection, viewport)
	return true

func _outer_wall_bindings_match(canvas, snapshot, scene_data, current_key, projection, viewport) -> bool:
	if not _valid_canvas(canvas) or not _valid_identity(snapshot, scene_data, current_key) or not _valid_projection(projection, viewport):
		return false
	var required_properties: Array[String] = ["snapshot", "scene", "current_key", "camera", "viewport_rect"]
	var property_list: Array[Dictionary] = canvas.get_property_list()
	for required_name: String in required_properties:
		var found: bool = false
		for property: Dictionary in property_list:
			if property.get("name") == required_name:
				found = true
				break
		if not found:
			return false
	var actual_snapshot: Variant = canvas.get("snapshot")
	var actual_scene: Variant = canvas.get("scene")
	var actual_key: Variant = canvas.get("current_key")
	var actual_camera: Variant = canvas.get("camera")
	var actual_viewport: Variant = canvas.get("viewport_rect")
	var expected_viewport: Rect2 = viewport
	var camera_viewport: Variant = projection.get("viewport")
	return actual_snapshot is Dictionary and _deep_equal(actual_snapshot, snapshot) and actual_scene is Dictionary and _deep_equal(actual_scene, scene_data) and typeof(actual_key) == TYPE_STRING and actual_key == current_key and actual_camera == projection and actual_viewport is Rect2 and actual_viewport == expected_viewport and camera_viewport is Vector2 and camera_viewport == expected_viewport.size

func _nonzero_polygon_area(points: PackedVector2Array) -> bool:
	if points.size() < 3 or not _finite_points(points):
		return false
	var twice_area: float = 0.0
	for index: int in range(points.size()):
		twice_area += points[index].cross(points[(index + 1) % points.size()])
	return is_finite(twice_area) and absf(twice_area) > 0.0

func build_dressing(snapshot, scene_data, current_key, projection, viewport) -> Array:
	if not _valid_context(snapshot, scene_data, current_key) or not _valid_projection(projection, viewport):
		return []
	var packets: Array = []
	var canonical_presentation_variant: Variant = _canonical_scene.get("presentation")
	if not canonical_presentation_variant is Dictionary:
		return []
	var canonical_presentation: Dictionary = canonical_presentation_variant
	var sprites_variant: Variant = canonical_presentation.get("sprites")
	if not sprites_variant is Array:
		return []
	var sprites: Array = sprites_variant
	for row_variant: Variant in sprites:
		if not row_variant is Dictionary:
			return []
		var row: Dictionary = row_variant
		if not matches_authored_row(row):
			continue
		var packet: Dictionary = _make_dressing_packet(row, current_key, projection)
		if not packet.is_empty():
			packets.append(packet)
	if packets.size() > 2:
		return []
	return packets

func draw_contact_shadow(canvas, packet, snapshot, scene_data, current_key, projection, viewport) -> bool:
	if not _valid_canvas(canvas) or not _valid_context(snapshot, scene_data, current_key) or not _valid_projection(projection, viewport) or not _valid_packet(packet, snapshot, scene_data, current_key, projection):
		return false
	var points_variant: Variant = packet.get("shadow_points")
	if not points_variant is PackedVector2Array:
		return false
	var points: PackedVector2Array = points_variant
	if points.size() != SHADOW_STEPS or not _finite_points(points):
		return false
	var bounds: Rect2 = _points_bounds(points)
	if not _finite_rect(bounds):
		return false
	var viewport_rect: Rect2 = viewport
	if bounds.intersects(viewport_rect):
		var alpha: float = float(packet.get("shadow_alpha"))
		canvas.draw_colored_polygon(points, Color(0.045, 0.052, 0.058, alpha))
	return true

func draw_dressing(canvas, packet, snapshot, scene_data, current_key, projection, viewport) -> bool:
	if not _valid_canvas(canvas) or not _valid_context(snapshot, scene_data, current_key) or not _valid_projection(projection, viewport) or not _valid_packet(packet, snapshot, scene_data, current_key, projection):
		return false
	var texture_variant: Variant = packet.get("texture")
	if not texture_variant is ImageTexture:
		return false
	var texture: ImageTexture = texture_variant
	var destination_variant: Variant = packet.get("destination_rect")
	var source_variant: Variant = packet.get("source_rect")
	var origin_variant: Variant = packet.get("transform_origin")
	var scale_variant: Variant = packet.get("transform_scale")
	var rotation_variant: Variant = packet.get("transform_rotation")
	if not destination_variant is Rect2 or not source_variant is Rect2 or not origin_variant is Vector2 or not scale_variant is Vector2 or not P.number(rotation_variant, -1.0, 1.0):
		return false
	var destination: Rect2 = destination_variant
	var source: Rect2 = source_variant
	var origin: Vector2 = origin_variant
	var transform_scale: Vector2 = scale_variant
	var rotation: float = float(rotation_variant)
	if not _finite_rect(destination) or not _finite_rect(source) or not _finite_point(origin) or not _finite_point(transform_scale) or transform_scale.x <= 0.0 or transform_scale.y <= 0.0:
		return false
	canvas.draw_set_transform(origin, rotation, transform_scale)
	canvas.draw_texture_rect_region(texture, destination, source, Color.WHITE, false, true)
	canvas.draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
	return true

func _valid_identity(snapshot, scene_data, current_key) -> bool:
	if not P.snapshot(snapshot) or not scene_data is Dictionary or typeof(current_key) != TYPE_STRING:
		return false
	var snapshot_dictionary: Dictionary = snapshot
	var scene_dictionary: Dictionary = scene_data
	if typeof(snapshot_dictionary.get("worldId")) != TYPE_STRING or typeof(snapshot_dictionary.get("sceneId")) != TYPE_STRING or snapshot_dictionary.get("worldId") != "grey_hive" or snapshot_dictionary.get("sceneId") != "gh_lockdown":
		return false
	if typeof(scene_dictionary.get("worldId")) != TYPE_STRING or typeof(scene_dictionary.get("sceneId")) != TYPE_STRING or scene_dictionary.get("worldId") != "grey_hive" or scene_dictionary.get("sceneId") != "gh_lockdown":
		return false
	if not snapshot_dictionary.has("entryToken") or snapshot_dictionary.get("entryToken") != null or not P.counter(snapshot_dictionary.get("worldEpoch"), 1):
		return false
	return current_key == "grey_hive:gh_lockdown:" + str(snapshot_dictionary.get("worldEpoch"))

func _valid_context(snapshot, scene_data, current_key) -> bool:
	if not _context_ready or not _static_ready or not _valid_identity(snapshot, scene_data, current_key) or not scene_data is Dictionary:
		return false
	var scene_dictionary: Dictionary = scene_data
	return _deep_equal(scene_dictionary, _canonical_scene)

func _valid_projection(projection, viewport) -> bool:
	if not projection is Object or projection.get_script() != PROJECTION_SCRIPT or not projection.has_method("project"):
		return false
	var initialized_variant: Variant = projection.get("initialized")
	var origin_variant: Variant = projection.get("origin")
	var center_variant: Variant = projection.get("screen_center")
	var size_variant: Variant = projection.get("viewport")
	var ppm_variant: Variant = projection.get("ppm")
	if typeof(initialized_variant) != TYPE_BOOL or not bool(initialized_variant) or not origin_variant is Vector3 or not center_variant is Vector2 or not size_variant is Vector2 or not P.number(ppm_variant, 0.000001, MAX_PPM) or not viewport is Rect2:
		return false
	var origin: Vector3 = origin_variant
	var center: Vector2 = center_variant
	var camera_size: Vector2 = size_variant
	var viewport_rect: Rect2 = viewport
	if not _finite_world(origin) or not _finite_point(center) or not _finite_point(camera_size) or camera_size.x <= 0.0 or camera_size.y <= 0.0 or not _finite_rect(viewport_rect):
		return false
	return absf(camera_size.x - viewport_rect.size.x) <= 0.01 and absf(camera_size.y - viewport_rect.size.y) <= 0.01

func _valid_canvas(canvas) -> bool:
	return canvas is CanvasItem and canvas.is_inside_tree() and not canvas.is_queued_for_deletion()

func _valid_map(value) -> bool:
	if not value is Dictionary:
		return false
	var mapping: Dictionary = value
	return mapping.get("schema") == "wuxian.gh-lockdown-environment.internal-use/v1" and typeof(mapping.get("registeredSourceReuse")) == TYPE_BOOL and mapping.get("registeredSourceReuse") and mapping.get("approval") == "INTERNAL_REGISTERED_SOURCE_REUSE" and mapping.get("GodotDisplay") == "NOT_RUN" and mapping.get("publicRelease") == "NOT_ESTABLISHED" and mapping.get("worldId") == "grey_hive" and mapping.get("sceneId") == "gh_lockdown"

func _reference_maps_valid() -> bool:
	return not _read_exact("res://data/gh_entry_floor_material.internal-use.json", 0, "6513a6a3c7b5702eea7c5e27ca61f9d19eee5926c546473f2648306b6e41f6ee").is_empty() and not _read_exact("res://data/gh_entry_service_pipe.internal-use.json", 1615, "0dcad760e164065f93ec6ce695ee719954ebae511c36f01c9360d3a563c087ac").is_empty() and not _read_exact("res://data/gh_entry_clutter.internal-use.json", 0, "c112001fb86be02767a015cd0a998cc39fc1ef37451ea35022c13b06155e7ca5").is_empty()

func _load_floor() -> void:
	var bytes: PackedByteArray = _read_exact(FLOOR_PATH, FLOOR_BYTES, FLOOR_SHA256)
	if bytes.is_empty():
		_status["floor"] = "PIN_OR_READ_FAILURE"
		return
	var image: Image = Image.new()
	if image.load_png_from_buffer(bytes) != OK or image.get_width() != FLOOR_SIZE.x or image.get_height() != FLOOR_SIZE.y or image.get_format() != Image.FORMAT_RGBA8:
		_status["floor"] = "PNG_CANVAS_OR_FORMAT_FAILURE"
		return
	for source_corner: Array in FLOOR_UV_CORNERS:
		var corner_alpha: int = roundi(image.get_pixel(int(source_corner[0]), int(source_corner[1])).a * 255.0)
		if corner_alpha < 251 or corner_alpha > 254:
			_status["floor"] = "TOP_FACE_SAMPLE_ALPHA_FAILURE"
			return
	var texture: ImageTexture = ImageTexture.create_from_image(image)
	if texture == null or texture.get_width() != FLOOR_SIZE.x or texture.get_height() != FLOOR_SIZE.y:
		_status["floor"] = "TEXTURE_CREATION_FAILURE"
		return
	_floor_texture = texture
	_status["floor"] = "READY"

func _load_pipe() -> void:
	var bytes: PackedByteArray = _read_exact(PIPE_PATH, PIPE_BYTES, PIPE_SHA256)
	if bytes.is_empty():
		_status["pipe"] = "PIN_OR_READ_FAILURE"
		return
	var image: Image = Image.new()
	if image.load_png_from_buffer(bytes) != OK or image.get_width() != PIPE_SIZE.x or image.get_height() != PIPE_SIZE.y or image.get_format() != Image.FORMAT_RGBA8:
		_status["pipe"] = "PNG_CANVAS_OR_FORMAT_FAILURE"
		return
	var isolated: Image = _isolate_component(image, PIPE_FRAME, PIPE_SEED, PIPE_PIXELS, -1, PIPE_CONTACT_A, PIPE_CONTACT_B)
	if isolated == null:
		_status["pipe"] = "COMPONENT_OR_CONTACT_FAILURE"
		return
	_pipe_texture = ImageTexture.create_from_image(isolated)
	if _pipe_texture == null or _pipe_texture.get_width() != PIPE_FRAME.size.x or _pipe_texture.get_height() != PIPE_FRAME.size.y:
		_pipe_texture = null
		_status["pipe"] = "TEXTURE_CREATION_FAILURE"
		return
	_status["pipe"] = "READY"

func _load_clutter() -> void:
	var bytes: PackedByteArray = _read_exact(CLUTTER_PATH, CLUTTER_BYTES, CLUTTER_SHA256)
	if bytes.is_empty():
		_status["clutter"] = "PIN_OR_READ_FAILURE"
		return
	var image: Image = Image.new()
	if image.load_png_from_buffer(bytes) != OK or image.get_width() != CLUTTER_SIZE.x or image.get_height() != CLUTTER_SIZE.y or image.get_format() != Image.FORMAT_RGBA8:
		_status["clutter"] = "PNG_CANVAS_OR_FORMAT_FAILURE"
		return
	var isolated: Image = _isolate_component(image, CLUTTER_FRAME, CLUTTER_SEED, CLUTTER_PIXELS, CLUTTER_DISCONNECTED_PIXELS, CLUTTER_CONTACT_A + CLUTTER_FRAME.position, CLUTTER_CONTACT_B + CLUTTER_FRAME.position)
	if isolated == null:
		_status["clutter"] = "COMPONENT_OR_CONTACT_FAILURE"
		return
	_clutter_texture = ImageTexture.create_from_image(isolated)
	if _clutter_texture == null or _clutter_texture.get_width() != CLUTTER_FRAME.size.x or _clutter_texture.get_height() != CLUTTER_FRAME.size.y:
		_clutter_texture = null
		_status["clutter"] = "TEXTURE_CREATION_FAILURE"
		return
	_status["clutter"] = "READY"

# Static wall-plane decoration only; no alarm, power or progression state.
# Called only after the canonical north wall has been successfully drawn.
func _draw_alarm_body(canvas, projection, viewport) -> void:
	if _alarm_body_texture == null:
		return
	var world_points: Array[Vector3] = [Vector3(3.6, 0.75, 0.5), Vector3(4.4, 0.75, 0.5), Vector3(4.4, 0.15, 0.5), Vector3(3.6, 0.15, 0.5)]
	var points: PackedVector2Array = PackedVector2Array()
	for world_point: Vector3 in world_points:
		var projected_variant: Variant = projection.call("project", world_point)
		if not projected_variant is Vector2:
			return
		var projected_point: Vector2 = projected_variant
		if not _finite_point(projected_point):
			return
		points.append(projected_point)
	if points.size() != 4 or not _valid_points_bounds(points):
		return
	var viewport_rect: Rect2 = viewport
	if not _points_bounds(points).intersects(viewport_rect):
		return
	var uvs: PackedVector2Array = PackedVector2Array([Vector2(0.0, 0.0), Vector2(1.0, 0.0), Vector2(1.0, 1.0), Vector2(0.0, 1.0)])
	var colors: PackedColorArray = PackedColorArray([Color.WHITE, Color.WHITE, Color.WHITE, Color.WHITE])
	canvas.draw_polygon(points, colors, uvs, _alarm_body_texture)

func _load_alarm_body() -> void:
	var bytes: PackedByteArray = _read_exact(ALARM_BODY_PATH, ALARM_BODY_BYTES, ALARM_BODY_SHA256)
	if bytes.is_empty():
		_status["alarm_body"] = "PIN_OR_READ_FAILURE"
		return
	var image: Image = Image.new()
	if image.load_png_from_buffer(bytes) != OK or image.get_width() != ALARM_BODY_SIZE.x or image.get_height() != ALARM_BODY_SIZE.y or image.get_format() != Image.FORMAT_RGBA8:
		_status["alarm_body"] = "PNG_CANVAS_OR_FORMAT_FAILURE"
		return
	var texture: ImageTexture = ImageTexture.create_from_image(image)
	if texture == null or texture.get_width() != ALARM_BODY_SIZE.x or texture.get_height() != ALARM_BODY_SIZE.y:
		_status["alarm_body"] = "TEXTURE_CREATION_FAILURE"
		return
	_alarm_body_texture = texture
	_status["alarm_body"] = "READY"

func _read_exact(path: String, expected_bytes: int, expected_sha: String) -> PackedByteArray:
	var file: FileAccess = FileAccess.open(path, FileAccess.READ)
	if file == null:
		return PackedByteArray()
	var length: int = int(file.get_length())
	if expected_bytes > 0 and length != expected_bytes:
		file.close()
		return PackedByteArray()
	if expected_bytes == 0 and (length < 1 or length > 32768):
		file.close()
		return PackedByteArray()
	var bytes: PackedByteArray = file.get_buffer(length)
	file.close()
	if bytes.size() != length or _sha256(bytes) != expected_sha:
		return PackedByteArray()
	return bytes

func _sha256(bytes: PackedByteArray) -> String:
	var context: HashingContext = HashingContext.new()
	if context.start(HashingContext.HASH_SHA256) != OK or context.update(bytes) != OK:
		return ""
	return context.finish().hex_encode()

func _isolate_component(source: Image, frame: Rect2i, seed_source: Vector2i, expected_pixels: int, expected_disconnected: int, contact_a: Vector2i, contact_b: Vector2i) -> Image:
	if frame.size.x <= 0 or frame.size.y <= 0 or frame.position.x < 0 or frame.position.y < 0 or frame.end.x > source.get_width() or frame.end.y > source.get_height():
		return null
	var seed_x: int = seed_source.x - frame.position.x
	var seed_y: int = seed_source.y - frame.position.y
	if seed_x < 0 or seed_y < 0 or seed_x >= frame.size.x or seed_y >= frame.size.y or source.get_pixel(seed_source.x, seed_source.y).a <= 0.0:
		return null
	var capacity: int = frame.size.x * frame.size.y
	var visited: PackedByteArray = PackedByteArray()
	visited.resize(capacity)
	var queue: PackedInt32Array = PackedInt32Array()
	queue.resize(capacity)
	var seed_index: int = seed_y * frame.size.x + seed_x
	queue[0] = seed_index
	visited[seed_index] = 1
	var head: int = 0
	var tail: int = 1
	var selected_count: int = 0
	var total_alpha: int = 0
	var minimum: Vector2i = Vector2i(frame.size.x, frame.size.y)
	var maximum: Vector2i = Vector2i(-1, -1)
	if expected_disconnected >= 0:
		for y in range(frame.size.y):
			for x in range(frame.size.x):
				if source.get_pixel(frame.position.x + x, frame.position.y + y).a > 0.0:
					total_alpha += 1
	var isolated: Image = Image.create_empty(frame.size.x, frame.size.y, false, Image.FORMAT_RGBA8)
	if isolated == null:
		return null
	isolated.fill(Color(0.0, 0.0, 0.0, 0.0))
	while head < tail:
		var index: int = queue[head]
		head += 1
		var x: int = index % frame.size.x
		var y: int = floori(float(index) / float(frame.size.x))
		var source_x: int = frame.position.x + x
		var source_y: int = frame.position.y + y
		var color: Color = source.get_pixel(source_x, source_y)
		if color.a <= 0.0:
			return null
		isolated.set_pixel(x, y, color)
		selected_count += 1
		minimum.x = mini(minimum.x, x)
		minimum.y = mini(minimum.y, y)
		maximum.x = maxi(maximum.x, x)
		maximum.y = maxi(maximum.y, y)
		if x > 0:
			var left_index: int = index - 1
			if visited[left_index] == 0 and source.get_pixel(source_x - 1, source_y).a > 0.0:
				visited[left_index] = 1
				queue[tail] = left_index
				tail += 1
		if x + 1 < frame.size.x:
			var right_index: int = index + 1
			if visited[right_index] == 0 and source.get_pixel(source_x + 1, source_y).a > 0.0:
				visited[right_index] = 1
				queue[tail] = right_index
				tail += 1
		if y > 0:
			var up_index: int = index - frame.size.x
			if visited[up_index] == 0 and source.get_pixel(source_x, source_y - 1).a > 0.0:
				visited[up_index] = 1
				queue[tail] = up_index
				tail += 1
		if y + 1 < frame.size.y:
			var down_index: int = index + frame.size.x
			if visited[down_index] == 0 and source.get_pixel(source_x, source_y + 1).a > 0.0:
				visited[down_index] = 1
				queue[tail] = down_index
				tail += 1
	if selected_count != expected_pixels or minimum != Vector2i.ZERO or maximum != frame.size - Vector2i.ONE:
		return null
	if expected_disconnected >= 0 and total_alpha - selected_count != expected_disconnected:
		return null
	var local_a: Vector2i = contact_a - frame.position
	var local_b: Vector2i = contact_b - frame.position
	if local_a.x < 0 or local_a.y < 0 or local_b.x < 0 or local_b.y < 0 or local_a.x >= frame.size.x or local_a.y >= frame.size.y or local_b.x >= frame.size.x or local_b.y >= frame.size.y:
		return null
	if isolated.get_pixel(local_a.x, local_a.y).a <= 0.0 or isolated.get_pixel(local_b.x, local_b.y).a <= 0.0:
		return null
	return isolated

func _make_dressing_packet(row: Dictionary, current_key: String, projection) -> Dictionary:
	var row_id_variant: Variant = row.get("id")
	if typeof(row_id_variant) != TYPE_STRING:
		return {}
	var row_id: String = row_id_variant
	var texture: ImageTexture
	var frame: Rect2i
	var contact_a: Vector2i
	var contact_b: Vector2i
	var world_a: Vector3
	var world_b: Vector3
	var max_rotation: float
	var max_extent: float
	var shadow_radii: Vector2
	var shadow_alpha: float
	if row_id == "lockdown_service_cables" and _pipe_texture != null:
		texture = _pipe_texture
		frame = Rect2i(0, 0, PIPE_FRAME.size.x, PIPE_FRAME.size.y)
		contact_a = PIPE_CONTACT_A - PIPE_FRAME.position
		contact_b = PIPE_CONTACT_B - PIPE_FRAME.position
		world_a = PIPE_WORLD_A
		world_b = PIPE_WORLD_B
		max_rotation = PIPE_MAX_ROTATION
		max_extent = PIPE_MAX_EXTENT_AT_48
		shadow_radii = PIPE_SHADOW_RADII
		shadow_alpha = PIPE_SHADOW_ALPHA
	elif row_id == "lockdown_approved_clutter" and _clutter_texture != null:
		texture = _clutter_texture
		frame = Rect2i(0, 0, CLUTTER_FRAME.size.x, CLUTTER_FRAME.size.y)
		contact_a = CLUTTER_CONTACT_A
		contact_b = CLUTTER_CONTACT_B
		world_a = CLUTTER_WORLD_A
		world_b = CLUTTER_WORLD_B
		max_rotation = CLUTTER_MAX_ROTATION
		max_extent = CLUTTER_MAX_EXTENT_AT_48
		shadow_radii = CLUTTER_SHADOW_RADII
		shadow_alpha = CLUTTER_SHADOW_ALPHA
	else:
		return {}
	var texture_size: Vector2i = Vector2i(texture.get_width(), texture.get_height())
	if texture_size != frame.size or not matches_authored_row(row):
		return {}
	var contact_a_variant: Variant = projection.call("project", world_a)
	var contact_b_variant: Variant = projection.call("project", world_b)
	if not contact_a_variant is Vector2 or not contact_b_variant is Vector2:
		return {}
	var screen_a: Vector2 = contact_a_variant
	var screen_b: Vector2 = contact_b_variant
	if not _finite_point(screen_a) or not _finite_point(screen_b):
		return {}
	var source_a: Vector2 = Vector2(float(contact_a.x), float(contact_a.y))
	var source_b: Vector2 = Vector2(float(contact_b.x), float(contact_b.y))
	var source_axis: Vector2 = source_b - source_a
	var screen_axis: Vector2 = screen_b - screen_a
	var source_length: float = source_axis.length()
	var screen_length: float = screen_axis.length()
	var ppm_variant: Variant = projection.get("ppm")
	if not P.number(ppm_variant, 0.000001, MAX_PPM) or source_length <= 0.0 or not is_finite(source_length) or not is_finite(screen_length):
		return {}
	var world_dx: float = world_b.x - world_a.x
	var world_dz: float = world_b.z - world_a.z
	var expected_span: float = PIPE_SPAN if row_id == "lockdown_service_cables" else CLUTTER_SPAN
	if absf(Vector2(world_dx, world_dz).length() - expected_span) > 0.000001:
		return {}
	var expected_axis: Vector2 = Vector2((world_dx - world_dz) * float(ppm_variant), (world_dx + world_dz) * float(ppm_variant) * 0.5)
	if not _finite_point(expected_axis) or screen_axis.distance_to(expected_axis) > 0.02:
		return {}
	var uniform_scale: float = screen_length / source_length
	var rotation: float = screen_axis.angle() - source_axis.angle()
	if not is_finite(uniform_scale) or uniform_scale <= 0.0 or not is_finite(rotation) or absf(rotation) > max_rotation:
		return {}
	var transform_scale: Vector2 = Vector2(uniform_scale, uniform_scale)
	var transform_origin: Vector2 = screen_a - source_a.rotated(rotation) * uniform_scale
	if not _finite_point(transform_origin):
		return {}
	var endpoint_check: Vector2 = transform_origin + source_b.rotated(rotation) * uniform_scale
	if not _finite_point(endpoint_check) or endpoint_check.distance_to(screen_b) > 0.02:
		return {}
	var body_rect: Rect2 = _transformed_bounds(transform_origin, rotation, transform_scale, Vector2(frame.size))
	if not _finite_rect(body_rect) or maxf(body_rect.size.x, body_rect.size.y) > max_extent * float(ppm_variant) / 48.0 + 0.02:
		return {}
	var shadow_points: PackedVector2Array = _shadow_points(projection, Vector3(float(row.position[0]), 0.0, float(row.position[2])), shadow_radii)
	if shadow_points.size() != SHADOW_STEPS or not _finite_points(shadow_points):
		return {}
	var destination: Rect2 = Rect2(Vector2.ZERO, Vector2(frame.size))
	return {
		"id": row_id, "row_id": row_id, "kind": "gh_lockdown_dressing", "context_key": current_key,
		"asset_id": row.get("assetId"), "layer": row.get("layer"), "p": Vector3(float(row.position[0]), 0.0, float(row.position[2])),
		"texture": texture, "source_rect": destination, "destination_rect": destination,
		"transform_origin": transform_origin, "transform_rotation": rotation, "transform_scale": transform_scale,
		"uniform_scale": uniform_scale, "body_rect": body_rect, "fallback_rect": _prop_placeholder_at(projection, Vector3(float(row.position[0]), 0.0, float(row.position[2]))),
		"shadow_points": shadow_points, "shadow_radii": shadow_radii, "shadow_alpha": shadow_alpha,
		"camera_origin": projection.get("origin"), "camera_center": projection.get("screen_center"),
		"camera_viewport": projection.get("viewport"), "camera_ppm": float(ppm_variant)
	}

func _valid_packet(packet, snapshot, scene_data, current_key, projection) -> bool:
	if not packet is Dictionary or not _valid_context(snapshot, scene_data, current_key):
		return false
	var data: Dictionary = packet
	var row_id_variant: Variant = data.get("row_id")
	if typeof(row_id_variant) != TYPE_STRING or data.get("kind") != "gh_lockdown_dressing" or data.get("context_key") != current_key:
		return false
	var row: Dictionary = _find_sprite(_canonical_scene, row_id_variant)
	if row.is_empty() or not matches_authored_row(row) or data.get("asset_id") != row.get("assetId"):
		return false
	var expected_texture: ImageTexture = _pipe_texture if row_id_variant == "lockdown_service_cables" else _clutter_texture
	if expected_texture == null or data.get("texture") != expected_texture:
		return false
	if data.get("camera_origin") != projection.get("origin") or data.get("camera_center") != projection.get("screen_center") or data.get("camera_viewport") != projection.get("viewport") or not P.number(data.get("camera_ppm"), 0.000001, MAX_PPM) or absf(float(data.get("camera_ppm")) - float(projection.get("ppm"))) > 0.000001:
		return false
	var p_variant: Variant = data.get("p")
	var body_variant: Variant = data.get("body_rect")
	var fallback_variant: Variant = data.get("fallback_rect")
	var origin_variant: Variant = data.get("transform_origin")
	var scale_variant: Variant = data.get("transform_scale")
	var source_variant: Variant = data.get("source_rect")
	var destination_variant: Variant = data.get("destination_rect")
	var shadow_variant: Variant = data.get("shadow_points")
	if not p_variant is Vector3 or not _finite_world(p_variant) or not body_variant is Rect2 or not fallback_variant is Rect2 or not origin_variant is Vector2 or not scale_variant is Vector2 or not source_variant is Rect2 or not destination_variant is Rect2 or not shadow_variant is PackedVector2Array:
		return false
	var p: Vector3 = p_variant
	var body_rect: Rect2 = body_variant
	var fallback_rect: Rect2 = fallback_variant
	var transform_origin: Vector2 = origin_variant
	var transform_scale: Vector2 = scale_variant
	var source_rect: Rect2 = source_variant
	var destination_rect: Rect2 = destination_variant
	var shadow_points: PackedVector2Array = shadow_variant
	var expected_position: Variant = row.get("position")
	return p.x >= 0.0 and p.x <= 24.0 and p.z >= 0.0 and p.z <= 16.0 and _numbers_equal(expected_position, [p.x, 0.0, p.z]) and _finite_rect(body_rect) and _finite_rect(fallback_rect) and _finite_point(transform_origin) and _finite_point(transform_scale) and transform_scale.x > 0.0 and transform_scale.y > 0.0 and _finite_rect(source_rect) and _finite_rect(destination_rect) and shadow_points.size() == SHADOW_STEPS and _finite_points(shadow_points) and P.number(data.get("transform_rotation"), -0.1, 0.1) and P.number(data.get("uniform_scale"), 0.000001, 1.0e6) and P.number(data.get("shadow_alpha"), 0.0, 0.18)

func _wall_spec(wall_id: String) -> Dictionary:
	match wall_id:
		"lockdown_wall_north":
			return {"inner_a": Vector2(0.0, 0.5), "inner_b": Vector2(24.0, 0.5), "inset": Vector2(0.0, -0.04), "height": 0.9}
		"lockdown_wall_south":
			return {"inner_a": Vector2(0.0, 15.5), "inner_b": Vector2(24.0, 15.5), "inset": Vector2(0.0, 0.04), "height": 0.18}
		"lockdown_wall_west_north":
			return {"inner_a": Vector2(0.5, 0.5), "inner_b": Vector2(0.5, 6.5), "inset": Vector2(-0.04, 0.0), "height": 0.9}
		"lockdown_wall_west_south":
			return {"inner_a": Vector2(0.5, 9.5), "inner_b": Vector2(0.5, 15.5), "inset": Vector2(-0.04, 0.0), "height": 0.9}
		"lockdown_wall_east_north":
			return {"inner_a": Vector2(23.5, 0.5), "inner_b": Vector2(23.5, 6.5), "inset": Vector2(0.04, 0.0), "height": 0.18}
		"lockdown_wall_east_south":
			return {"inner_a": Vector2(23.5, 9.5), "inner_b": Vector2(23.5, 15.5), "inset": Vector2(0.04, 0.0), "height": 0.18}
	return {}

func _matches_row(row: Dictionary, expected_id: String, expected_asset: String, expected_layer: String, expected_position: Array) -> bool:
	if row.size() != 4 or typeof(row.get("id")) != TYPE_STRING or typeof(row.get("assetId")) != TYPE_STRING or typeof(row.get("layer")) != TYPE_STRING:
		return false
	if row.get("id") != expected_id or row.get("assetId") != expected_asset or row.get("layer") != expected_layer:
		return false
	return _numbers_equal(row.get("position"), expected_position)

func _numbers_equal(value, expected: Array) -> bool:
	if not value is Array:
		return false
	var actual: Array = value
	if actual.size() != expected.size():
		return false
	for index in range(expected.size()):
		if not P.number(actual[index]) or not P.number(expected[index]) or absf(float(actual[index]) - float(expected[index])) > 0.000001:
			return false
	return true

func _deep_equal(left, right) -> bool:
	var left_numeric: bool = P.number(left)
	var right_numeric: bool = P.number(right)
	if left_numeric or right_numeric:
		return left_numeric and right_numeric and absf(float(left) - float(right)) <= 0.000001
	if typeof(left) != typeof(right):
		return false
	if left is Dictionary:
		var left_dictionary: Dictionary = left
		var right_dictionary: Dictionary = right
		if left_dictionary.size() != right_dictionary.size():
			return false
		for key: Variant in left_dictionary.keys():
			if not right_dictionary.has(key) or not _deep_equal(left_dictionary.get(key), right_dictionary.get(key)):
				return false
		return true
	if left is Array:
		var left_array: Array = left
		var right_array: Array = right
		if left_array.size() != right_array.size():
			return false
		for index: int in range(left_array.size()):
			if not _deep_equal(left_array[index], right_array[index]):
				return false
		return true
	return left == right

func _find_row(rows_variant, key: String, value) -> Dictionary:
	if not rows_variant is Array:
		return {}
	var rows: Array = rows_variant
	var found: Dictionary = {}
	for row_variant: Variant in rows:
		if not row_variant is Dictionary:
			return {}
		var row: Dictionary = row_variant
		if row.get(key) == value:
			if not found.is_empty():
				return {}
			found = row
	return found

func _find_sprite(scene_data: Dictionary, row_id) -> Dictionary:
	var presentation_variant: Variant = scene_data.get("presentation")
	if not presentation_variant is Dictionary:
		return {}
	var presentation: Dictionary = presentation_variant
	return _find_row(presentation.get("sprites"), "id", row_id)

func _shadow_points(projection, center: Vector3, radii: Vector2) -> PackedVector2Array:
	var points: PackedVector2Array = PackedVector2Array()
	for step in range(SHADOW_STEPS):
		var angle: float = TAU * float(step) / float(SHADOW_STEPS)
		var world_point: Vector3 = Vector3(center.x + cos(angle) * radii.x, 0.0, center.z + sin(angle) * radii.y)
		var projected_variant: Variant = projection.call("project", world_point)
		if not projected_variant is Vector2:
			return PackedVector2Array()
		var projected: Vector2 = projected_variant
		if not _finite_point(projected):
			return PackedVector2Array()
		points.append(projected)
	return points

func _prop_placeholder_at(projection, world_position: Vector3) -> Rect2:
	var projected_variant: Variant = projection.call("project", world_position)
	if not projected_variant is Vector2:
		return Rect2()
	var at: Vector2 = projected_variant
	var ppm: float = float(projection.get("ppm"))
	return Rect2(at - Vector2(16.0, 24.0) * ppm / 48.0, Vector2(32.0, 24.0) * ppm / 48.0)

func _transformed_bounds(origin: Vector2, rotation: float, transform_scale: Vector2, size: Vector2) -> Rect2:
	var minimum: Vector2 = Vector2(INF, INF)
	var maximum: Vector2 = Vector2(-INF, -INF)
	var local_points: Array[Vector2] = [Vector2.ZERO, Vector2(size.x, 0.0), size, Vector2(0.0, size.y)]
	for local_point: Vector2 in local_points:
		var screen_point: Vector2 = origin + local_point.rotated(rotation) * transform_scale
		if not _finite_point(screen_point):
			return Rect2()
		minimum = minimum.min(screen_point)
		maximum = maximum.max(screen_point)
	return Rect2(minimum, maximum - minimum) if maximum.x > minimum.x and maximum.y > minimum.y else Rect2()

func _points_bounds(points: PackedVector2Array) -> Rect2:
	if points.is_empty():
		return Rect2()
	var minimum: Vector2 = Vector2(INF, INF)
	var maximum: Vector2 = Vector2(-INF, -INF)
	for point: Vector2 in points:
		if not _finite_point(point):
			return Rect2()
		minimum = minimum.min(point)
		maximum = maximum.max(point)
	return Rect2(minimum, maximum - minimum) if maximum.x > minimum.x and maximum.y > minimum.y else Rect2()

func _valid_points_bounds(points: PackedVector2Array) -> bool:
	return points.size() >= 3 and _finite_points(points) and _finite_rect(_points_bounds(points))

func _finite_point(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _finite_world(value: Vector3) -> bool:
	return is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _finite_points(points: PackedVector2Array) -> bool:
	for point: Vector2 in points:
		if not _finite_point(point):
			return false
	return true

func _finite_rect(value: Rect2) -> bool:
	return _finite_point(value.position) and _finite_point(value.size) and value.size.x > 0.0 and value.size.y > 0.0 and _finite_point(value.end)
