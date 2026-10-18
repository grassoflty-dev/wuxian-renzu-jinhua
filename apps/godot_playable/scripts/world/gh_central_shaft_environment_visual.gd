extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const PROJECTION_SCRIPT = preload('res://scripts/world/projection.gd')
const SCENE_PATH = 'res://data/scenes/gh_central_shaft.json'
const SCENE_BYTES = 10328
const SCENE_SHA256 = '806ef2be417d59ab7041002cecfb26c1fc383db920157feb29c2669c343df5da'
const MAP_PATH = 'res://data/gh_central_shaft_environment.internal-use.json'
const MAP_BYTES = 3642
const MAP_SHA256 = '9888dcf1993dd3c45eda7036f7f7c3a143a6c6c079e2738dc0c132987cea403d'
const FLOOR_PATH = 'res://assets/props/grey_hive/floor_tiles.png'
const FLOOR_BYTES = 1315059
const FLOOR_SHA256 = '4c93e192505e71d6ec4e414aeb5c9b76376b7407d677017b31b8eba5e24e744f'
const WALL_PATH = 'res://assets/props/return_station/bulkhead_tiles.png'
const WALL_BYTES = 1673423
const WALL_SHA256 = 'c5a18cae9a3ddc95c8515f6b342cb7ca3be12c23ff502ab5b3f16000bb4c9a5b'
const PIPE_PATH = 'res://assets/props/return_station/service_pipes.png'
const PIPE_BYTES = 1049893
const PIPE_SHA256 = '7f3c67879398e89adc9026fc2aca5fbe7792b1dc360b73c737c1b2299eac96da'
const PIPE_FRAME = Rect2i(0, 219, 388, 270)
const PIPE_PIXELS = 46171
const PIPE_SEED = Vector2i(100, 320)
const PIPE_CONTACT_A = Vector2(50.0, 126.0)
const PIPE_CONTACT_B = Vector2(325.0, 263.0)
const PIPE_WORLD_A = Vector3(9.5, 0.0, 3.0)
const PIPE_WORLD_B = Vector3(10.5, 0.0, 3.0)
const PIPE_CENTER = Vector3(10.0, 0.0, 3.0)
const PIPE_MAX_ROTATION = 0.025
const PIPE_MAX_EXTENT_AT_48 = 80.0
const WALL_FRAME = Rect2i(0, 0, 355, 480)
const WALL_PAINTED = Rect2i(41, 6, 299, 457)
const WALL_FOOT_A = Vector2(50.0, 322.0)
const WALL_FOOT_B = Vector2(300.0, 460.0)
const WALL_MAX_EXTENT_AT_48 = 96.0
const FLOOR_CORNERS = [Vector2(187, 184), Vector2(345, 305), Vector2(185, 426), Vector2(27, 305)]
const CANONICAL_WALLS = {
	'shaft_wall_north': [[0.0,0.0],[24.0,0.0],[24.0,0.5],[0.0,0.5]],
	'shaft_wall_south': [[0.0,15.5],[24.0,15.5],[24.0,16.0],[0.0,16.0]],
	'shaft_wall_west_north': [[0.0,0.5],[0.5,0.5],[0.5,6.5],[0.0,6.5]],
	'shaft_wall_west_south': [[0.0,9.5],[0.5,9.5],[0.5,15.5],[0.0,15.5]],
	'shaft_wall_east_north': [[23.5,0.5],[24.0,0.5],[24.0,6.5],[23.5,6.5]],
	'shaft_wall_east_south': [[23.5,9.5],[24.0,9.5],[24.0,15.5],[23.5,15.5]],
	'shaft_support_column_a': [[8.0,5.0],[9.0,5.0],[9.0,7.0],[8.0,7.0]],
	'shaft_support_column_b': [[15.0,9.0],[16.0,9.0],[16.0,11.0],[15.0,11.0]]
}

var _ready_for_scene: bool = false
var _scene_hash: String = ''
var _canonical_scene: Dictionary = {}
var _floor_texture: Texture2D
var _wall_texture: Texture2D
var _pipe_texture: Texture2D

func prepare(snapshot_value: Variant, scene_value: Variant, key_value: Variant, scene_hash_value: Variant) -> bool:
	if snapshot_value is Dictionary and scene_value is Dictionary and typeof(key_value) == TYPE_STRING and typeof(scene_hash_value) == TYPE_STRING and _ready_for_scene and _scene_hash == SCENE_SHA256:
		var cached_snapshot: Dictionary = snapshot_value
		var cached_scene: Dictionary = scene_value
		var cached_key: String = key_value
		if scene_hash_value == SCENE_SHA256 and _valid_snapshot_key(cached_snapshot,cached_key) and _same_value(cached_scene,_canonical_scene):
			return true
	_ready_for_scene = false
	_scene_hash = ''
	_canonical_scene.clear()
	_floor_texture = null
	_wall_texture = null
	_pipe_texture = null
	if not snapshot_value is Dictionary or not scene_value is Dictionary or typeof(key_value) != TYPE_STRING or typeof(scene_hash_value) != TYPE_STRING:
		return false
	var snapshot_data: Dictionary = snapshot_value
	var scene_data: Dictionary = scene_value
	var current_key: String = key_value
	var supplied_hash: String = scene_hash_value
	if not _valid_snapshot_key(snapshot_data, current_key) or supplied_hash != SCENE_SHA256:
		return false
	var map_data: PackedByteArray = _read_exact(MAP_PATH, MAP_BYTES, MAP_SHA256)
	if map_data.size() != MAP_BYTES:
		return false
	var map_text: String = map_data.get_string_from_utf8()
	if map_text.to_utf8_buffer() != map_data:
		return false
	var map_value: Variant = JSON.parse_string(map_text)
	if not _valid_use_map(map_value):
		return false
	var scene_bytes: PackedByteArray = _read_exact(SCENE_PATH, SCENE_BYTES, SCENE_SHA256)
	if scene_bytes.size() != SCENE_BYTES:
		return false
	var scene_text: String = scene_bytes.get_string_from_utf8()
	if scene_text.to_utf8_buffer() != scene_bytes:
		return false
	var canonical_value: Variant = JSON.parse_string(scene_text)
	if not canonical_value is Dictionary:
		return false
	var canonical_scene: Dictionary = canonical_value
	if not _same_value(scene_data, canonical_scene) or not _valid_layout(canonical_scene):
		return false
	var floor_texture: Texture2D = _load_png(FLOOR_PATH, FLOOR_BYTES, FLOOR_SHA256, Vector2i(1434, 980))
	var wall_texture: Texture2D = _load_png(WALL_PATH, WALL_BYTES, WALL_SHA256, Vector2i(1418, 1028))
	var pipe_source: Image = _load_png_image(PIPE_PATH, PIPE_BYTES, PIPE_SHA256, Vector2i(1414, 1047))
	if floor_texture == null or wall_texture == null or pipe_source == null:
		return false
	var pipe_crop: Image = _isolate_pipe_component(pipe_source)
	if pipe_crop == null or pipe_crop.get_width() != PIPE_FRAME.size.x or pipe_crop.get_height() != PIPE_FRAME.size.y or pipe_crop.get_format() != Image.FORMAT_RGBA8:
		return false
	var pipe_texture: ImageTexture = ImageTexture.create_from_image(pipe_crop)
	if pipe_texture == null or pipe_texture.get_width() != PIPE_FRAME.size.x or pipe_texture.get_height() != PIPE_FRAME.size.y:
		return false
	_canonical_scene = canonical_scene
	_scene_hash = SCENE_SHA256
	_floor_texture = floor_texture
	_wall_texture = wall_texture
	_pipe_texture = pipe_texture
	_ready_for_scene = true
	return true

func draw_floor(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not _valid_context(snapshot_value, scene_value, key_value) or not _valid_camera(camera_value, viewport_value) or not _valid_canvas(canvas_value) or _floor_texture == null:
		return false
	var camera: RefCounted = camera_value
	var viewport: Rect2 = viewport_value
	var patches: Array = []
	var uv_corners: PackedVector2Array = PackedVector2Array()
	for pixel in FLOOR_CORNERS:
		var uv: Vector2 = Vector2((pixel.x + 0.5) / 1434.0, (pixel.y + 0.5) / 980.0)
		if not _finite_v2(uv):
			return false
		uv_corners.append(uv)
	for row in range(8):
		for column in range(12):
			var x0: float = float(column) * 2.0
			var z0: float = float(row) * 2.0
			var world_points: Array[Vector3] = [Vector3(x0,0.0,z0),Vector3(x0+2.0,0.0,z0),Vector3(x0+2.0,0.0,z0+2.0),Vector3(x0,0.0,z0+2.0)]
			var points: PackedVector2Array = PackedVector2Array()
			for world_point in world_points:
				var projected: Vector2 = camera.project(world_point)
				if not _finite_v2(projected):
					return false
				points.append(projected)
			var bounds: Rect2 = _bounds(points)
			if not _valid_rect(bounds):
				return false
			patches.append({'points':points,'bounds':bounds})
	if patches.size() != 96 or uv_corners.size() != 4:
		return false
	var room_points: PackedVector2Array = PackedVector2Array([camera.project(Vector3(0,0,0)),camera.project(Vector3(24,0,0)),camera.project(Vector3(24,0,16)),camera.project(Vector3(0,0,16))])
	if not _valid_points(room_points):
		return false
	var colors: PackedColorArray = PackedColorArray([Color(0.62,0.66,0.64,1.0),Color(0.62,0.66,0.64,1.0),Color(0.62,0.66,0.64,1.0),Color(0.62,0.66,0.64,1.0)])
	for patch_value in patches:
		if not patch_value is Dictionary:
			return false
		var patch: Dictionary = patch_value
		var points: PackedVector2Array = patch.get('points')
		var bounds: Rect2 = patch.get('bounds')
		if not _valid_points(points) or not _valid_rect(bounds):
			return false
	for patch_value in patches:
		var patch: Dictionary = patch_value
		var points: PackedVector2Array = patch.points
		var bounds: Rect2 = patch.bounds
		if _intersects(viewport, bounds):
			canvas_value.draw_polygon(points, colors, uv_corners, _floor_texture)
	canvas_value.draw_colored_polygon(room_points, Color(0.467,0.569,0.545,0.025))
	return true

func draw_wall(canvas_value: Variant, wall_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not _valid_context(snapshot_value, scene_value, key_value) or not _valid_camera(camera_value, viewport_value) or not _valid_canvas(canvas_value) or not wall_value is Dictionary or _wall_texture == null:
		return false
	var wall: Dictionary = wall_value
	var wall_id_value: Variant = wall.get('id')
	if typeof(wall_id_value) != TYPE_STRING or not CANONICAL_WALLS.has(wall_id_value) or not _same_value(wall, _collision_by_id(_canonical_scene, wall_id_value)):
		return false
	var camera: RefCounted = camera_value
	var viewport: Rect2 = viewport_value
	if wall_id_value == 'shaft_wall_north':
		var north_plan: Array = _north_wall_plan(camera)
		var ground: PackedVector2Array = _project_polygon(wall.polygon, 0.0, camera)
		if north_plan.size() != 24 or not _valid_points(ground):
			return false
		for module_value in north_plan:
			if not module_value is Dictionary or not module_value.get('origin') is Vector2 or not module_value.get('scale') is Vector2 or not is_finite(module_value.get('rotation')) or not _valid_rect(module_value.get('bounds')):
				return false
		canvas_value.draw_colored_polygon(ground, Color('#394943'))
		for module_value in north_plan:
			var module: Dictionary = module_value
			canvas_value.draw_set_transform(module.origin, module.rotation, module.scale)
			canvas_value.draw_texture_rect_region(_wall_texture, Rect2(0,0,355,480), Rect2(WALL_FRAME), Color(0.72,0.76,0.73,1.0), false, true)
			canvas_value.draw_set_transform(Vector2.ZERO,0.0,Vector2.ONE)
		return true
	var height: float = 0.55 if wall_id_value in ['shaft_support_column_a','shaft_support_column_b'] else 0.35
	var polygon_value: Variant = wall.get('polygon')
	if not polygon_value is Array or polygon_value.size() != 4:
		return false
	var ground_points: PackedVector2Array = PackedVector2Array()
	var raised_points: PackedVector2Array = PackedVector2Array()
	for point_value in polygon_value:
		if not point_value is Array or point_value.size() != 2 or not P.number(point_value[0]) or not P.number(point_value[1]):
			return false
		var x: float = float(point_value[0])
		var z: float = float(point_value[1])
		var ground_point: Vector2 = camera.project(Vector3(x,0.0,z))
		var raised_point: Vector2 = camera.project(Vector3(x,height,z))
		if not _finite_v2(ground_point) or not _finite_v2(raised_point):
			return false
		ground_points.append(ground_point)
		raised_points.append(raised_point)
	var faces: Array = []
	for index in range(4):
		var next_index: int = (index + 1) % 4
		var face: PackedVector2Array = PackedVector2Array([ground_points[index],ground_points[next_index],raised_points[next_index],raised_points[index]])
		if not _valid_points(face):
			return false
		var color: Color = Color('#394943') if wall_id_value in ['shaft_wall_south','shaft_wall_west_north','shaft_wall_west_south'] else Color('#45554f')
		faces.append({'points':face,'color':color})
	var contact_bounds: Rect2 = _bounds(ground_points)
	var cap_bounds: Rect2 = _bounds(raised_points)
	if not _valid_points(ground_points) or not _valid_points(raised_points) or not _valid_rect(contact_bounds) or not _valid_rect(cap_bounds):
		return false
	for face_value in faces:
		if not face_value is Dictionary or not face_value.get('points') is PackedVector2Array or not face_value.get('color') is Color or not _valid_points(face_value.points):
			return false
	canvas_value.draw_colored_polygon(ground_points, Color(0.045,0.065,0.058,0.12))
	for face_value in faces:
		var face_row: Dictionary = face_value
		canvas_value.draw_colored_polygon(face_row.points, face_row.color)
	canvas_value.draw_colored_polygon(raised_points, Color('#687773'))
	return true

func matches_pipe_row(row_value: Variant) -> bool:
	if not row_value is Dictionary:
		return false
	var row: Dictionary = row_value
	if row.get('id') != 'shaft_cable_risers' or row.get('assetId') != 'runtime2d.grey_hive.pipes_cables.v1' or row.get('layer') != 'visual.props_back':
		return false
	return _number_array_equal(row.get('position'), [10.0,0.0,3.0])

func build_pipe_packet(snapshot_value: Variant, scene_value: Variant, key_value: Variant, camera_value: Variant, viewport_value: Variant) -> Dictionary:
	if not _valid_context(snapshot_value, scene_value, key_value) or not _valid_camera(camera_value, viewport_value) or _pipe_texture == null:
		return {}
	var camera: RefCounted = camera_value
	var sprites_value: Variant = _canonical_scene.get('presentation',{}).get('sprites')
	if not sprites_value is Array:
		return {}
	var row_count: int = 0
	for row_value in sprites_value:
		if not row_value is Dictionary:
			return {}
		if row_value.get('id') == 'shaft_cable_risers':
			row_count += 1
			if not matches_pipe_row(row_value):
				return {}
	if row_count != 1:
		return {}
	var screen_a: Vector2 = camera.project(PIPE_WORLD_A)
	var screen_b: Vector2 = camera.project(PIPE_WORLD_B)
	if not _finite_v2(screen_a) or not _finite_v2(screen_b):
		return {}
	var axis: Vector2 = screen_b - screen_a
	var source_axis: Vector2 = PIPE_CONTACT_B - PIPE_CONTACT_A
	var source_length: float = source_axis.length()
	var scale_value: float = axis.length() / source_length if source_length > 0.0 else 0.0
	var rotation: float = axis.angle() - source_axis.angle()
	var transform_scale: Vector2 = Vector2(scale_value,scale_value)
	var origin: Vector2 = screen_a - (PIPE_CONTACT_A * scale_value).rotated(rotation)
	var endpoint: Vector2 = origin + (PIPE_CONTACT_B * scale_value).rotated(rotation)
	var body_points: PackedVector2Array = _transformed_rect_points(origin, rotation, transform_scale, Rect2(0,0,388,270))
	var body_rect: Rect2 = _bounds(body_points)
	if not is_finite(scale_value) or scale_value <= 0.0 or not is_finite(rotation) or absf(rotation) > PIPE_MAX_ROTATION or not _finite_v2(origin) or not _finite_v2(endpoint) or endpoint.distance_to(screen_b) > 0.01 or not _valid_points(body_points) or not _valid_rect(body_rect) or maxf(body_rect.size.x,body_rect.size.y) > PIPE_MAX_EXTENT_AT_48 * camera.ppm / 48.0 + 0.01:
		return {}
	var shadow_points: PackedVector2Array = PackedVector2Array()
	for index in range(32):
		var angle: float = TAU * float(index) / 32.0
		var world: Vector3 = PIPE_CENTER + Vector3(cos(angle) * 0.6,0.0,sin(angle) * 0.16)
		var projected: Vector2 = camera.project(world)
		if not _finite_v2(projected):
			return {}
		shadow_points.append(projected)
	var shadow_rect: Rect2 = _bounds(shadow_points)
	if not _valid_rect(shadow_rect):
		return {}
	return {
		'id':'shaft_cable_risers','kind':'gh_central_pipe','context_key':key_value,'scene_hash':_scene_hash,
		'camera_origin':camera.origin,'camera_center':camera.screen_center,'camera_ppm':camera.ppm,
		'texture':_pipe_texture,'origin':origin,'rotation':rotation,'scale':transform_scale,
		'body_rect':body_rect,'body_points':body_points,'shadow_points':shadow_points,'shadow_rect':shadow_rect
	}

func draw_pipe_shadow(canvas_value: Variant, packet_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not _valid_canvas(canvas_value) or not _valid_pipe_packet(packet_value,snapshot_value,scene_value,key_value,camera_value,viewport_value):
		return false
	var packet: Dictionary = packet_value
	var viewport: Rect2 = viewport_value
	var shadow_points: PackedVector2Array = packet.shadow_points
	var shadow_rect: Rect2 = packet.shadow_rect
	if not _valid_points(shadow_points) or not _valid_rect(shadow_rect):
		return false
	if _intersects(viewport,shadow_rect):
		canvas_value.draw_colored_polygon(shadow_points,Color(0.035,0.045,0.04,0.18))
	return true

func draw_pipe_body(canvas_value: Variant, packet_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not _valid_canvas(canvas_value) or not _valid_pipe_packet(packet_value,snapshot_value,scene_value,key_value,camera_value,viewport_value):
		return false
	var packet: Dictionary = packet_value
	var viewport: Rect2 = viewport_value
	var body_rect: Rect2 = packet.body_rect
	if not _valid_rect(body_rect) or not _intersects(viewport,body_rect):
		return false
	canvas_value.draw_set_transform(packet.origin, packet.rotation, packet.scale)
	canvas_value.draw_texture_rect_region(packet.texture,Rect2(0,0,388,270),Rect2(0,0,388,270),Color.WHITE,false,true)
	canvas_value.draw_set_transform(Vector2.ZERO,0.0,Vector2.ONE)
	return true

func _valid_pipe_packet(packet_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not packet_value is Dictionary or not _valid_context(snapshot_value,scene_value,key_value) or not _valid_camera(camera_value,viewport_value):
		return false
	var packet: Dictionary = packet_value
	var camera: RefCounted = camera_value
	if packet.get('id') != 'shaft_cable_risers' or packet.get('kind') != 'gh_central_pipe' or packet.get('context_key') != key_value or packet.get('scene_hash') != _scene_hash or packet.get('texture') != _pipe_texture:
		return false
	if packet.get('camera_origin') != camera.origin or packet.get('camera_center') != camera.screen_center or not P.number(packet.get('camera_ppm'),0.000001,256.0) or float(packet.camera_ppm) != float(camera.ppm):
		return false
	if not packet.get('origin') is Vector2 or not packet.get('scale') is Vector2 or not packet.get('body_rect') is Rect2 or not packet.get('body_points') is PackedVector2Array or not packet.get('shadow_points') is PackedVector2Array or not packet.get('shadow_rect') is Rect2:
		return false
	return _finite_v2(packet.origin) and _finite_v2(packet.scale) and packet.scale.x > 0.0 and packet.scale.y > 0.0 and _valid_rect(packet.body_rect) and _valid_points(packet.body_points) and _valid_rect(packet.shadow_rect) and _valid_points(packet.shadow_points)

func _valid_context(snapshot_value: Variant, scene_value: Variant, key_value: Variant) -> bool:
	if not _ready_for_scene or _scene_hash != SCENE_SHA256 or not snapshot_value is Dictionary or not scene_value is Dictionary or typeof(key_value) != TYPE_STRING:
		return false
	var snapshot_data: Dictionary = snapshot_value
	var scene_data: Dictionary = scene_value
	var current_key: String = key_value
	return _valid_snapshot_key(snapshot_data,current_key) and _same_value(scene_data,_canonical_scene)

func _valid_snapshot_key(snapshot_data: Dictionary, current_key: String) -> bool:
	if not P.snapshot(snapshot_data) or snapshot_data.get('entryToken') != null:
		return false
	if typeof(snapshot_data.get('worldId')) != TYPE_STRING or snapshot_data.get('worldId') != 'grey_hive' or typeof(snapshot_data.get('sceneId')) != TYPE_STRING or snapshot_data.get('sceneId') != 'gh_central_shaft':
		return false
	if not P.counter(snapshot_data.get('worldEpoch'),1) or current_key != 'grey_hive:gh_central_shaft:' + str(snapshot_data.get('worldEpoch')):
		return false
	return true

func _valid_camera(camera_value: Variant, viewport_value: Variant) -> bool:
	if not camera_value is RefCounted or camera_value.get_script() != PROJECTION_SCRIPT or not viewport_value is Rect2:
		return false
	var camera: RefCounted = camera_value
	var viewport: Rect2 = viewport_value
	if typeof(camera.get('initialized')) != TYPE_BOOL or not camera.initialized or not camera.get('origin') is Vector3 or not camera.get('screen_center') is Vector2 or not camera.get('viewport') is Vector2:
		return false
	if not _finite_v3(camera.origin) or not _finite_v2(camera.screen_center) or not _finite_v2(camera.viewport) or not _valid_rect(viewport):
		return false
	return P.number(camera.get('ppm'),0.000001,256.0) and camera.viewport == viewport.size

func _valid_canvas(canvas_value: Variant) -> bool:
	return canvas_value is CanvasItem and canvas_value.is_inside_tree() and not canvas_value.is_queued_for_deletion()

func _valid_layout(scene_data: Dictionary) -> bool:
	if scene_data.get('worldId') != 'grey_hive' or scene_data.get('sceneId') != 'gh_central_shaft':
		return false
	var bounds_value: Variant = scene_data.get('boundsM')
	if not bounds_value is Dictionary or not _same_value(bounds_value,{'x':0,'z':0,'width':24,'depth':16}):
		return false
	var collisions_value: Variant = scene_data.get('collision')
	if not collisions_value is Array or collisions_value.size() != 8:
		return false
	var seen: Dictionary = {}
	for collision_value in collisions_value:
		if not collision_value is Dictionary:
			return false
		var collision: Dictionary = collision_value
		var id_value: Variant = collision.get('id')
		if typeof(id_value) != TYPE_STRING or not CANONICAL_WALLS.has(id_value) or seen.has(id_value) or not _same_value(collision.get('polygon'),CANONICAL_WALLS[id_value]):
			return false
		seen[id_value] = true
	if seen.size() != 8:
		return false
	var presentation_value: Variant = scene_data.get('presentation')
	if not presentation_value is Dictionary or not presentation_value.get('sprites') is Array:
		return false
	var sprite_ids: Dictionary = {}
	var selected: Dictionary = {'shaft_approved_floor':false,'shaft_back_wall':false,'shaft_cable_risers':false}
	for sprite_value in presentation_value.sprites:
		if not sprite_value is Dictionary:
			return false
		var sprite: Dictionary = sprite_value
		var id_value: Variant = sprite.get('id')
		if typeof(id_value) != TYPE_STRING or sprite_ids.has(id_value):
			return false
		sprite_ids[id_value] = true
		if selected.has(id_value):
			var expected_asset: String = 'runtime2d.grey_hive.floor_tiles.v1' if id_value == 'shaft_approved_floor' else ('runtime2d.grey_hive.wall_tiles.v1' if id_value == 'shaft_back_wall' else 'runtime2d.grey_hive.pipes_cables.v1')
			var expected_layer: String = 'visual.floor' if id_value == 'shaft_approved_floor' else 'visual.props_back'
			var expected_position: Array = [12,0,8] if id_value == 'shaft_approved_floor' else ([12,0,1] if id_value == 'shaft_back_wall' else [10,0,3])
			if sprite.get('assetId') != expected_asset or sprite.get('layer') != expected_layer or not _number_array_equal(sprite.get('position'),expected_position):
				return false
			selected[id_value] = true
	for row_id in selected:
		if not selected[row_id]:
			return false
	return true

func _valid_use_map(value: Variant) -> bool:
	if not value is Dictionary:
		return false
	var map_data: Dictionary = value
	return _same_value(map_data, _expected_use_map())

func _expected_use_map() -> Dictionary:
	return {
		'schema':'wuxian.gh-central-shaft-environment.internal-use/v1',
		'registeredSourceReuse':true,
		'approval':'INTERNAL_REGISTERED_SOURCE_REUSE',
		'GodotDisplay':'NOT_RUN',
		'publicRelease':'NOT_ESTABLISHED',
		'worldId':'grey_hive',
		'sceneId':'gh_central_shaft',
		'canonicalScene':{'path':SCENE_PATH,'bytes':SCENE_BYTES,'sha256':SCENE_SHA256},
		'floor':{
			'assetId':'runtime2d.grey_hive.floor_tiles.v1','path':FLOOR_PATH,'bytes':FLOOR_BYTES,'sha256':FLOOR_SHA256,
			'canvas':[1434,980],'format':'RGBA8',
			'selection':{'kind':'affine_top_face','pixelCenterCorners':[[187,184],[345,305],[185,426],[27,305]],'uvConvention':'(sourcePixelCenter + 0.5) / canvasSize','interiorAlphaRange':[251,254]},
			'rowId':'shaft_approved_floor','layer':'visual.floor','positionM':[12,0,8],'boundsXZWH':[0,0,24,16],
			'cellSizeM':[2,2],'grid':[12,8],'patchCount':96,'tint':[0.62,0.66,0.64,1],
			'tileability':'NOT_ESTABLISHED; observe actual seams, do not promise seamless'
		},
		'wall':{
			'rowId':'shaft_back_wall','assetId':'runtime2d.grey_hive.wall_tiles.v1','layer':'visual.props_back','positionM':[12,0,1],
			'path':WALL_PATH,'bytes':WALL_BYTES,'sha256':WALL_SHA256,'canvas':[1418,1028],'format':'RGBA8',
			'framePx':[0,0,355,480],'paintedBoundsPx':[41,6,299,457],'groundFeetPx':[[50,322],[300,460]],
			'texturedRuns':[{'id':'north','collisionId':'shaft_wall_north','startM':[0,0,0.25],'endM':[24,0,0.25],'cadenceM':1,'count':24}],
			'maxExtentPxAt48':96,'tint':[0.72,0.76,0.73,1],'rearOnly':true
		},
		'pipe':{
			'assetId':'runtime2d.grey_hive.pipes_cables.v1','path':PIPE_PATH,'bytes':PIPE_BYTES,'sha256':PIPE_SHA256,
			'canvas':[1414,1047],'format':'RGBA8',
			'selection':{'framePx':[0,219,388,270],'seedPx':[100,320],'componentBoundsPx':[0,219,388,270],'connectedAlphaPixels':46171,'contactsPx':[[50,345],[325,482]]},
			'rowId':'shaft_cable_risers','layer':'visual.props_back','positionM':[10,0,3],
			'contactsM':[[9.5,0,3],[10.5,0,3]],'spanM':1,'maximumRotationRadians':0.025,
			'maxExtentPxAt48':80,'shadowRadiiM':[0.6,0.16],'shadowAlpha':0.18
		},
		'cutaway':{
			'exteriorHeightM':0.35,'columnHeightM':0.55,'footprintPolicy':'exact canonical collision polygons only',
			'cap':'#687773','faceX':'#394943','faceZ':'#45554f','contactAlpha':0.12
		},
		'ambient':{
			'kind':'static floor-bound tonal wash','maxPolygons':3,'maxAlpha':0.035,
			'color':'#77918b','emitterOrGameplayState':false
		},
		'occlusion':'Unmodified canonical beam polygons/fadeTo and existing Renderer _update_occlusion/_occlusion only; no new occluder/fade/timer',
		'temporal':'NONE; identical new decoration for reduced_motion true/false',
		'registrationMeaning':'Exact existing pixel selections plus PROPOSED room-specific display contacts, not measured room dimensions; MAIN future admission required'
	}

func _same_value(left_value: Variant, right_value: Variant) -> bool:
	var left_type: int = typeof(left_value)
	var right_type: int = typeof(right_value)
	if (left_type == TYPE_INT or left_type == TYPE_FLOAT) and (right_type == TYPE_INT or right_type == TYPE_FLOAT):
		return P.number(left_value) and P.number(right_value) and float(left_value) == float(right_value)
	if left_type != right_type:
		return false
	if left_type == TYPE_DICTIONARY:
		var left_dictionary: Dictionary = left_value
		var right_dictionary: Dictionary = right_value
		if left_dictionary.size() != right_dictionary.size():
			return false
		for key in left_dictionary:
			if not right_dictionary.has(key) or not _same_value(left_dictionary[key],right_dictionary[key]):
				return false
		return true
	if left_type == TYPE_ARRAY:
		var left_array: Array = left_value
		var right_array: Array = right_value
		if left_array.size() != right_array.size():
			return false
		for index in range(left_array.size()):
			if not _same_value(left_array[index],right_array[index]):
				return false
		return true
	return left_value == right_value

func _number_array_equal(value: Variant, expected: Array) -> bool:
	if not value is Array or value.size() != expected.size():
		return false
	for index in range(expected.size()):
		if not P.number(value[index]) or float(value[index]) != float(expected[index]):
			return false
	return true

func _collision_by_id(scene_data: Dictionary, wanted_id: String) -> Variant:
	var rows_value: Variant = scene_data.get('collision')
	if not rows_value is Array:
		return null
	var found: Variant = null
	var count: int = 0
	for row_value in rows_value:
		if not row_value is Dictionary:
			return null
		var row: Dictionary = row_value
		if row.get('id') == wanted_id:
			found = row
			count += 1
	return found if count == 1 else null

func _north_wall_plan(camera: RefCounted) -> Array:
	if _wall_texture == null:
		return []
	var plan: Array = []
	var source_axis: Vector2 = WALL_FOOT_B - WALL_FOOT_A
	for index in range(24):
		var start: Vector2 = camera.project(Vector3(float(index),0.0,0.25))
		var finish: Vector2 = camera.project(Vector3(float(index + 1),0.0,0.25))
		var screen_axis: Vector2 = finish - start
		var ratio: float = screen_axis.length() / source_axis.length()
		var rotation: float = screen_axis.angle() - source_axis.angle()
		var origin: Vector2 = start - (WALL_FOOT_A * ratio).rotated(rotation)
		var endpoint: Vector2 = origin + (WALL_FOOT_B * ratio).rotated(rotation)
		var scale_value: Vector2 = Vector2(ratio,ratio)
		var painted_points: PackedVector2Array = _transformed_rect_points(origin,rotation,scale_value,Rect2(WALL_PAINTED))
		var painted_rect: Rect2 = _bounds(painted_points)
		if not _finite_v2(start) or not _finite_v2(finish) or not is_finite(ratio) or ratio <= 0.0 or not is_finite(rotation) or not _finite_v2(origin) or endpoint.distance_to(finish) > 0.01 or not _valid_points(painted_points) or not _valid_rect(painted_rect) or maxf(painted_rect.size.x,painted_rect.size.y) > WALL_MAX_EXTENT_AT_48 * camera.ppm / 48.0 + 0.01:
			return []
		plan.append({'origin':origin,'rotation':rotation,'scale':scale_value,'bounds':painted_rect})
	return plan if plan.size() == 24 else []

func _project_polygon(polygon_value: Variant, y: float, camera: RefCounted) -> PackedVector2Array:
	var points: PackedVector2Array = PackedVector2Array()
	if not polygon_value is Array or polygon_value.size() != 4:
		return points
	for point_value in polygon_value:
		if not point_value is Array or point_value.size() != 2 or not P.number(point_value[0]) or not P.number(point_value[1]):
			return PackedVector2Array()
		var projected: Vector2 = camera.project(Vector3(float(point_value[0]),y,float(point_value[1])))
		if not _finite_v2(projected):
			return PackedVector2Array()
		points.append(projected)
	return points

func _transformed_rect_points(origin: Vector2, rotation: float, scale_value: Vector2, rect: Rect2) -> PackedVector2Array:
	var points: PackedVector2Array = PackedVector2Array()
	var corners: Array[Vector2] = [rect.position,Vector2(rect.end.x,rect.position.y),rect.end,Vector2(rect.position.x,rect.end.y)]
	for corner in corners:
		var transformed: Vector2 = origin + (corner * scale_value).rotated(rotation)
		if not _finite_v2(transformed):
			return PackedVector2Array()
		points.append(transformed)
	return points

func _bounds(points: PackedVector2Array) -> Rect2:
	if points.size() < 3:
		return Rect2()
	var minimum: Vector2 = points[0]
	var maximum: Vector2 = points[0]
	for point in points:
		if not _finite_v2(point):
			return Rect2()
		minimum = minimum.min(point)
		maximum = maximum.max(point)
	var rect: Rect2 = Rect2(minimum,maximum-minimum)
	return rect if _valid_rect(rect) else Rect2()

func _intersects(left: Rect2, right: Rect2) -> bool:
	return _valid_rect(left) and _valid_rect(right) and left.intersects(right)

func _valid_rect(rect: Rect2) -> bool:
	return _finite_v2(rect.position) and _finite_v2(rect.size) and rect.size.x > 0.0 and rect.size.y > 0.0 and _finite_v2(rect.end)

func _valid_points(points: PackedVector2Array) -> bool:
	if points.size() < 3:
		return false
	for point in points:
		if not _finite_v2(point):
			return false
	return true

func _finite_v2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _finite_v3(value: Vector3) -> bool:
	return is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _read_exact(path: String, expected_bytes: int, expected_sha: String) -> PackedByteArray:
	var file: FileAccess = FileAccess.open(path,FileAccess.READ)
	if file == null:
		return PackedByteArray()
	var length: int = int(file.get_length())
	if length != expected_bytes:
		file.close()
		return PackedByteArray()
	var bytes: PackedByteArray = file.get_buffer(length)
	file.close()
	if bytes.size() != expected_bytes:
		return PackedByteArray()
	var digest: HashingContext = HashingContext.new()
	if digest.start(HashingContext.HASH_SHA256) != OK or digest.update(bytes) != OK:
		return PackedByteArray()
	if digest.finish().hex_encode() != expected_sha:
		return PackedByteArray()
	return bytes

func _load_png_image(path: String, expected_bytes: int, expected_sha: String, expected_size: Vector2i) -> Image:
	var bytes: PackedByteArray = _read_exact(path,expected_bytes,expected_sha)
	if bytes.size() != expected_bytes:
		return null
	var image: Image = Image.new()
	if image.load_png_from_buffer(bytes) != OK or image.get_width() != expected_size.x or image.get_height() != expected_size.y or image.get_format() != Image.FORMAT_RGBA8:
		return null
	return image

func _load_png(path: String, expected_bytes: int, expected_sha: String, expected_size: Vector2i) -> Texture2D:
	var image: Image = _load_png_image(path,expected_bytes,expected_sha,expected_size)
	if image == null:
		return null
	var texture: ImageTexture = ImageTexture.create_from_image(image)
	if texture == null or texture.get_width() != expected_size.x or texture.get_height() != expected_size.y:
		return null
	return texture

func _isolate_pipe_component(source: Image) -> Image:
	var width: int = PIPE_FRAME.size.x
	var height: int = PIPE_FRAME.size.y
	var seed_x: int = PIPE_SEED.x - PIPE_FRAME.position.x
	var seed_y: int = PIPE_SEED.y - PIPE_FRAME.position.y
	if seed_x < 0 or seed_y < 0 or seed_x >= width or seed_y >= height or source.get_pixel(PIPE_SEED.x,PIPE_SEED.y).a <= 0.0:
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
	var isolated: Image = Image.create_empty(width,height,false,Image.FORMAT_RGBA8)
	if isolated == null:
		return null
	isolated.fill(Color(0,0,0,0))
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
		var source_x: int = PIPE_FRAME.position.x + x
		var source_y: int = PIPE_FRAME.position.y + y
		var color: Color = source.get_pixel(source_x,source_y)
		if color.a <= 0.0:
			return null
		isolated.set_pixel(x,y,color)
		selected_pixels += 1
		min_x = mini(min_x,x)
		min_y = mini(min_y,y)
		max_x = maxi(max_x,x)
		max_y = maxi(max_y,y)
		if x > 0 and visited[index-1] == 0 and source.get_pixel(source_x-1,source_y).a > 0.0:
			visited[index-1] = 1
			queue[tail] = index-1
			tail += 1
		if x+1 < width and visited[index+1] == 0 and source.get_pixel(source_x+1,source_y).a > 0.0:
			visited[index+1] = 1
			queue[tail] = index+1
			tail += 1
		if y > 0 and visited[index-width] == 0 and source.get_pixel(source_x,source_y-1).a > 0.0:
			visited[index-width] = 1
			queue[tail] = index-width
			tail += 1
		if y+1 < height and visited[index+width] == 0 and source.get_pixel(source_x,source_y+1).a > 0.0:
			visited[index+width] = 1
			queue[tail] = index+width
			tail += 1
	if selected_pixels != PIPE_PIXELS or min_x != 0 or min_y != 0 or max_x != width-1 or max_y != height-1:
		return null
	if isolated.get_pixel(int(PIPE_CONTACT_A.x),int(PIPE_CONTACT_A.y)).a <= 0.0 or isolated.get_pixel(int(PIPE_CONTACT_B.x),int(PIPE_CONTACT_B.y)).a <= 0.0:
		return null
	return isolated
