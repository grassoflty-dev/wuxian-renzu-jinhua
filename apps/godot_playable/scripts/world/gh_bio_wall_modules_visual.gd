extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const MAPPING_RESOURCE = 'res://data/gh_bio_wall_modules.internal-use.json'
const MODULE_TEXTURE_PATH = 'res://assets/props/return_station/bulkhead_tiles.png'
const MODULE_BYTES = 1673423
const MODULE_SHA256 = 'c5a18cae9a3ddc95c8515f6b342cb7ca3be12c23ff502ab5b3f16000bb4c9a5b'
const FRAME = Rect2i(0, 0, 355, 480)
const PAINTED_BOUNDS = Rect2i(41, 6, 299, 457)
const FOOT_A = Vector2(50.0, 322.0)
const FOOT_B = Vector2(300.0, 460.0)
const MAX_MODULES = 96
const MAX_EXTENT_PX_AT_48 = 96.0
const SHADOW_ALPHA = 0.18
const FADE_TO = 0.25
const WALL_IDS = [
	'gh_bio_isolation_wall_north',
	'gh_bio_isolation_wall_south',
	'gh_bio_isolation_wall_west_north',
	'gh_bio_isolation_wall_west_south',
	'gh_bio_isolation_wall_east_north',
	'gh_bio_isolation_wall_east_south'
]
const RUNS = [
	{'id':'north', 'collision_id':'gh_bio_isolation_wall_north', 'start':Vector3(0.0,0.0,0.25), 'end':Vector3(24.0,0.0,0.25), 'axis':'x'},
	{'id':'south', 'collision_id':'gh_bio_isolation_wall_south', 'start':Vector3(0.0,0.0,15.75), 'end':Vector3(24.0,0.0,15.75), 'axis':'x'},
	{'id':'west_north', 'collision_id':'gh_bio_isolation_wall_west_north', 'start':Vector3(0.25,0.0,0.5), 'end':Vector3(0.25,0.0,6.5), 'axis':'z'},
	{'id':'west_south', 'collision_id':'gh_bio_isolation_wall_west_south', 'start':Vector3(0.25,0.0,9.5), 'end':Vector3(0.25,0.0,15.5), 'axis':'z'},
	{'id':'east_north', 'collision_id':'gh_bio_isolation_wall_east_north', 'start':Vector3(23.75,0.0,0.5), 'end':Vector3(23.75,0.0,6.5), 'axis':'z'},
	{'id':'east_south', 'collision_id':'gh_bio_isolation_wall_east_south', 'start':Vector3(23.75,0.0,9.5), 'end':Vector3(23.75,0.0,15.5), 'axis':'z'}
]
const COLLISION_POLYGONS = {
	'gh_bio_isolation_wall_north': [[0.0,0.0],[24.0,0.0],[24.0,0.5],[0.0,0.5]],
	'gh_bio_isolation_wall_south': [[0.0,15.5],[24.0,15.5],[24.0,16.0],[0.0,16.0]],
	'gh_bio_isolation_wall_west_north': [[0.0,0.5],[0.5,0.5],[0.5,6.5],[0.0,6.5]],
	'gh_bio_isolation_wall_west_south': [[0.0,9.5],[0.5,9.5],[0.5,15.5],[0.0,15.5]],
	'gh_bio_isolation_wall_east_north': [[23.5,0.5],[24.0,0.5],[24.0,6.5],[23.5,6.5]],
	'gh_bio_isolation_wall_east_south': [[23.5,9.5],[24.0,9.5],[24.0,15.5],[23.5,15.5]]
}

var attempted: bool = false
var module_texture: ImageTexture

func build_packets(snapshot: Dictionary, scene_data: Dictionary, current_key: String, projection) -> Array:
	if not _valid_context(snapshot, scene_data, current_key) or not _valid_projection(projection):
		return []
	_load_once()
	if module_texture == null:
		return []
	var packets: Array = []
	for run in RUNS:
		var run_length: float = run.start.distance_to(run.end)
		if not is_finite(run_length) or run_length <= 0.0 or not is_equal_approx(run_length, roundf(run_length)):
			return []
		var direction: Vector3 = (run.end - run.start).normalized()
		var count: int = int(roundf(run_length))
		for index in range(count):
			var world_start: Vector3 = run.start + direction * float(index)
			var world_end: Vector3 = run.start + direction * float(index + 1)
			var screen_start: Vector2 = projection.project(world_start)
			var screen_end: Vector2 = projection.project(world_end)
			var screen_axis: Vector2 = screen_end - screen_start
			var source_axis: Vector2 = FOOT_B - FOOT_A
			var ratio: float = screen_axis.length() / source_axis.length()
			var reflected: bool = run.axis == 'z'
			var target_angle: float = screen_axis.angle()
			var source_angle: float = source_axis.angle()
			var reflected_source_angle: float = PI - source_angle if reflected else source_angle
			var rotation: float = target_angle - reflected_source_angle
			var transform_scale: Vector2 = Vector2(-ratio, ratio) if reflected else Vector2(ratio, ratio)
			var transform_origin: Vector2 = screen_start - _source_to_screen(FOOT_A, rotation, transform_scale)
			var endpoint_check: Vector2 = transform_origin + _source_to_screen(FOOT_B, rotation, transform_scale)
			var painted_rect: Rect2 = _transformed_bounds(transform_origin, rotation, transform_scale, PAINTED_BOUNDS)
			var expected_step: float = projection.ppm * sqrt(1.25)
			if not _finite_vector2(screen_start) or not _finite_vector2(screen_end) or not _finite_vector2(transform_origin) or not is_finite(ratio) or ratio <= 0.0 or absf(endpoint_check.distance_to(screen_end)) > 0.01 or absf(screen_axis.length() - expected_step) > 0.01:
				return []
			if not _valid_rect(painted_rect) or maxf(painted_rect.size.x, painted_rect.size.y) > MAX_EXTENT_PX_AT_48 * (projection.ppm / 48.0) + 0.01:
				return []
			var packet_id: String = 'gh_bio:wall:%s:%02d' % [run.id, index]
			packets.append({
				'id':packet_id,
				'kind':'gh_bio_wall_module',
				'p':(world_start + world_end) * 0.5,
				'run_id':run.id,
				'index':index,
				'fade_group':packet_id,
				'texture':module_texture,
				'source_rect':Rect2(FRAME),
				'transform_origin':transform_origin,
				'transform_rotation':rotation,
				'transform_scale':transform_scale,
				'body_rect':painted_rect
			})
			if packets.size() > MAX_MODULES:
				return []
	return packets if packets.size() == 72 else []

func build_shadow_packets(scene_data: Dictionary, projection) -> Array:
	if not _valid_projection(projection) or not _valid_scene_row_and_collisions(scene_data):
		return []
	var packets: Array = []
	for wall_id in WALL_IDS:
		var points := PackedVector2Array()
		for point in COLLISION_POLYGONS[wall_id]:
			var projected: Vector2 = projection.project(Vector3(float(point[0]), 0.0, float(point[1])))
			if not _finite_vector2(projected):
				return []
			points.append(projected)
		var bounds: Rect2 = _points_bounds(points)
		if not _valid_rect(bounds):
			return []
		packets.append({'id':'gh_bio:wall-shadow:' + wall_id, 'points':points, 'bounds_rect':bounds})
	return packets if packets.size() == 6 else []

func front_fade_alphas(packets: Array, current_by_group: Dictionary, player_rect: Rect2, delta: float, reduced_motion: bool) -> Dictionary:
	var result: Dictionary = {}
	if not _valid_rect(player_rect) or not is_finite(delta):
		return result
	for packet in packets:
		if not packet is Dictionary or typeof(packet.get('fade_group')) != TYPE_STRING or not packet.get('body_rect') is Rect2:
			return {}
		var group: String = packet.fade_group
		var body_rect: Rect2 = packet.body_rect
		var target: float = FADE_TO if _rect_intersects(body_rect, player_rect) else 1.0
		result[group] = step_front_alpha(float(current_by_group.get(group, 1.0)), target, delta, reduced_motion)
	return result

func step_front_alpha(current: float, target: float, delta: float, reduced_motion: bool) -> float:
	if not is_finite(current) or not is_finite(target) or not is_finite(delta):
		return 1.0
	var clamped_target: float = clampf(target, FADE_TO, 1.0)
	if reduced_motion:
		return clamped_target
	return move_toward(clampf(current, FADE_TO, 1.0), clamped_target, clampf(delta, 0.0, 0.1) * 3.75)

func should_suppress_collision_fallback(collision_id, packets: Array) -> bool:
	return _packets_are_complete(packets) and typeof(collision_id) == TYPE_STRING and collision_id in WALL_IDS

func _packets_are_complete(packets: Array) -> bool:
	if packets.size() != 72:
		return false
	var cursor: int = 0
	for run in RUNS:
		var expected_count: int = int(roundf(run.start.distance_to(run.end)))
		for index in range(expected_count):
			var packet = packets[cursor]
			var packet_id: String = 'gh_bio:wall:%s:%02d' % [run.id,index]
			if not packet is Dictionary or packet.get('id') != packet_id or packet.get('run_id') != run.id or packet.get('index') != index or packet.get('kind') != 'gh_bio_wall_module' or not packet.get('body_rect') is Rect2 or not _valid_rect(packet.body_rect) or packet.get('texture') != module_texture or packet.get('source_rect') != Rect2(FRAME):
				return false
			cursor += 1
	return cursor == 72

func draw_contact_shadows(canvas: CanvasItem, packets: Array, viewport: Rect2) -> bool:
	if packets.size() != 6:
		return false
	for packet in packets:
		if not packet is Dictionary or not packet.get('points') is PackedVector2Array or packet.points.size() != 4 or not packet.get('bounds_rect') is Rect2 or not _valid_rect(packet.bounds_rect):
			return false
		for point in packet.points:
			if not _finite_vector2(point):
				return false
		if _rect_intersects(viewport, packet.bounds_rect):
			canvas.draw_colored_polygon(packet.points, Color(0.015, 0.025, 0.03, SHADOW_ALPHA))
	return true

func draw_module(canvas: CanvasItem, packet: Dictionary, alpha: float) -> bool:
	var texture = packet.get('texture')
	var origin = packet.get('transform_origin')
	var rotation = packet.get('transform_rotation')
	var scale = packet.get('transform_scale')
	var source_rect = packet.get('source_rect')
	if not texture is Texture2D or not origin is Vector2 or not scale is Vector2 or not source_rect is Rect2 or not is_finite(rotation) or not _finite_vector2(origin) or not _finite_vector2(scale) or not is_finite(alpha):
		return false
	canvas.draw_set_transform(origin, rotation, scale)
	canvas.draw_texture_rect_region(texture, source_rect, source_rect, Color(1.0, 1.0, 1.0, clampf(alpha, FADE_TO, 1.0)), false, true)
	canvas.draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
	return true

func _valid_context(snapshot: Dictionary, scene_data: Dictionary, current_key: String) -> bool:
	if not P.snapshot(snapshot):
		return false
	if snapshot.get('worldId') != 'grey_hive' or snapshot.get('sceneId') != 'gh_bio_isolation' or scene_data.get('worldId') != 'grey_hive' or scene_data.get('sceneId') != 'gh_bio_isolation':
		return false
	if not P.counter(snapshot.get('worldEpoch'), 1) or current_key != 'grey_hive:gh_bio_isolation:' + str(snapshot.get('worldEpoch')):
		return false
	return _valid_scene_row_and_collisions(scene_data)

func has_valid_layout(scene_data: Dictionary) -> bool:
	if typeof(scene_data.get('worldId')) != TYPE_STRING or scene_data.get('worldId') != 'grey_hive' or typeof(scene_data.get('sceneId')) != TYPE_STRING or scene_data.get('sceneId') != 'gh_bio_isolation':
		return false
	var bounds = scene_data.get('boundsM')
	if not bounds is Dictionary or not _keys_are(bounds, ['x','z','width','depth']):
		return false
	for key in ['x','z','width','depth']:
		if not P.number(bounds.get(key)):
			return false
	if float(bounds.get('x')) != 0.0 or float(bounds.get('z')) != 0.0 or float(bounds.get('width')) != 24.0 or float(bounds.get('depth')) != 16.0:
		return false
	var collisions = scene_data.get('collision')
	if not collisions is Array or collisions.size() != WALL_IDS.size():
		return false
	var found: Dictionary = {}
	for row in collisions:
		if not row is Dictionary:
			return false
		var collision_id = row.get('id')
		if typeof(collision_id) != TYPE_STRING or not (collision_id in WALL_IDS) or found.has(collision_id):
			return false
		if not _polygon_equals(row.get('polygon'), COLLISION_POLYGONS[collision_id]):
			return false
		found[collision_id] = true
	return found.size() == WALL_IDS.size()

func _valid_scene_row_and_collisions(scene_data: Dictionary) -> bool:
	if not has_valid_layout(scene_data):
		return false
	var presentation = scene_data.get('presentation')
	if not presentation is Dictionary or not P.rows(presentation.get('sprites'), 2048) or not P.unique(presentation.sprites, 'id'):
		return false
	var wall_row_count: int = 0
	for row in presentation.sprites:
		if row.get('id') == 'bio_wall_reuse':
			wall_row_count += 1
			if row.get('assetId') != 'runtime2d.grey_hive.wall_tiles.v1' or row.get('layer') != 'visual.props_back' or not _number_array_equals(row.get('position'), [12,0,1]):
				return false
	if wall_row_count != 1:
		return false
	var collisions = scene_data.get('collision')
	if not P.rows(collisions, 512) or not P.unique(collisions, 'id'):
		return false
	var found: Dictionary = {}
	for row in collisions:
		var collision_id = row.get('id')
		if collision_id in WALL_IDS:
			if found.has(collision_id) or not _polygon_equals(row.get('polygon'), COLLISION_POLYGONS[collision_id]):
				return false
			found[collision_id] = true
	return found.size() == WALL_IDS.size()

func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var manifest_file = FileAccess.open(MAPPING_RESOURCE, FileAccess.READ)
	if manifest_file == null or manifest_file.get_length() > 16384:
		return
	var manifest_bytes: PackedByteArray = manifest_file.get_buffer(manifest_file.get_length())
	manifest_file.close()
	var manifest_text: String = manifest_bytes.get_string_from_utf8()
	if manifest_text.to_utf8_buffer() != manifest_bytes or not _valid_manifest(JSON.parse_string(manifest_text)):
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
	if image.load_png_from_buffer(png_bytes) != OK or image.get_width() != 1418 or image.get_height() != 1028 or image.get_format() != Image.FORMAT_RGBA8 or _painted_bounds(image) != PAINTED_BOUNDS:
		return
	if image.get_pixel(int(FOOT_A.x), int(FOOT_A.y)).a <= 0.0 or image.get_pixel(int(FOOT_B.x), int(FOOT_B.y)).a <= 0.0:
		return
	module_texture = ImageTexture.create_from_image(image)

func _valid_manifest(value) -> bool:
	if not value is Dictionary or not _keys_are(value, ['schema','registeredSourceReuse','approval','GodotDisplay','publicRelease','sourceAsset','asset','installation']):
		return false
	if value.get('schema') != 'wuxian.gh-bio-wall-modules.internal-use/v1' or value.get('registeredSourceReuse') != true or value.get('approval') != 'INTERNAL_REGISTERED_SOURCE_REUSE' or value.get('GodotDisplay') != 'NOT_RUN' or value.get('publicRelease') != 'NOT_ESTABLISHED':
		return false
	var source = value.get('sourceAsset')
	var asset = value.get('asset')
	var installation = value.get('installation')
	if not source is Dictionary or not asset is Dictionary or not installation is Dictionary:
		return false
	if not _keys_are(source, ['path','bytes','sha256','canvas','format']) or source.get('path') != 'assets/derived/asset-pipeline-v1/sprites/png/runtime2d.grey_hive.wall_tiles.v1.png' or source.get('bytes') != MODULE_BYTES or source.get('sha256') != MODULE_SHA256 or source.get('format') != 'RGBA8' or not _number_array_equals(source.get('canvas'), [1418,1028]):
		return false
	if not _keys_are(asset, ['path','bytes','sha256','format','canvas','frame','paintedBounds','groundFeetPx']):
		return false
	if asset.get('path') != MODULE_TEXTURE_PATH or asset.get('bytes') != MODULE_BYTES or asset.get('sha256') != MODULE_SHA256 or asset.get('format') != 'RGBA8' or not _number_array_equals(asset.get('canvas'), [1418,1028]) or not _number_array_equals(asset.get('frame'), [0,0,355,480]) or not _number_array_equals(asset.get('paintedBounds'), [41,6,299,457]) or not _number_array_equals(asset.get('groundFeetPx'), [[50,322],[300,460]]):
		return false
	if not _keys_are(installation, ['worldId','sceneId','rowId','assetId','layer','positionM','runIds','runs','cadenceM','maxModules','maxExtentPxAt48','shadowAlpha','fadeTo','fadeDurationMs']):
		return false
	if installation.get('worldId') != 'grey_hive' or installation.get('sceneId') != 'gh_bio_isolation' or installation.get('rowId') != 'bio_wall_reuse' or installation.get('assetId') != 'runtime2d.grey_hive.wall_tiles.v1' or installation.get('layer') != 'visual.props_back' or not _number_array_equals(installation.get('positionM'), [12,0,1]) or installation.get('runIds') != ['north','south','west_north','west_south','east_north','east_south'] or installation.get('cadenceM') != 1 or installation.get('maxModules') != MAX_MODULES or installation.get('maxExtentPxAt48') != MAX_EXTENT_PX_AT_48 or installation.get('shadowAlpha') != SHADOW_ALPHA or installation.get('fadeTo') != FADE_TO or installation.get('fadeDurationMs') != 200:
		return false
	var mapped_runs = installation.get('runs')
	if not mapped_runs is Array or mapped_runs.size() != RUNS.size():
		return false
	for index in range(RUNS.size()):
		var row = mapped_runs[index]
		var expected = RUNS[index]
		if not row is Dictionary or not _keys_are(row, ['id','startM','endM']) or row.get('id') != expected.id or not _number_array_equals(row.get('startM'), [expected.start.x, expected.start.y, expected.start.z]) or not _number_array_equals(row.get('endM'), [expected.end.x, expected.end.y, expected.end.z]):
			return false
	return true

func _valid_projection(projection) -> bool:
	return projection != null and is_finite(projection.ppm) and projection.ppm > 0.0 and projection.ppm <= 256.0

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
		elif not P.number(value[index]) or float(value[index]) != float(expected[index]):
			return false
	return true

func _polygon_equals(value, expected: Array) -> bool:
	return _number_array_equals(value, expected)

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

func _points_bounds(points: PackedVector2Array) -> Rect2:
	if points.is_empty():
		return Rect2()
	var min_point: Vector2 = points[0]
	var max_point: Vector2 = points[0]
	for point in points:
		min_point = Vector2(minf(min_point.x, point.x), minf(min_point.y, point.y))
		max_point = Vector2(maxf(max_point.x, point.x), maxf(max_point.y, point.y))
	return Rect2(min_point, max_point - min_point)

func _rect_intersects(a: Rect2, b: Rect2) -> bool:
	if not _valid_rect(a) or not _valid_rect(b):
		return false
	return a.intersects(b)

func _valid_rect(value: Rect2) -> bool:
	return _finite_vector2(value.position) and _finite_vector2(value.size) and value.size.x > 0.0 and value.size.y > 0.0 and is_finite(value.end.x) and is_finite(value.end.y)

func _finite_vector2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)
