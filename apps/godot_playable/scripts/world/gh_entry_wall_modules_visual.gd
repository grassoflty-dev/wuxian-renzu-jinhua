extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const MAPPING_RESOURCE = 'res://data/gh_entry_wall_modules.internal-use.json'
const MODULE_TEXTURE_PATH = 'res://assets/props/return_station/bulkhead_tiles.png'
const MODULE_BYTES = 1673423
const MODULE_SHA256 = 'c5a18cae9a3ddc95c8515f6b342cb7ca3be12c23ff502ab5b3f16000bb4c9a5b'
const SOURCE_PATH = 'assets/derived/asset-pipeline-v1/sprites/png/runtime2d.grey_hive.wall_tiles.v1.png'
const FRAME = Rect2i(0, 0, 355, 480)
const PAINTED_BOUNDS = Rect2i(41, 6, 299, 457)
const FOOT_A = Vector2(50.0, 322.0)
const FOOT_B = Vector2(300.0, 460.0)
const MAX_MODULES = 96
const MAX_EXTENT_PX_AT_48 = 96.0
const SHADOW_ALPHA = 0.18
const FADE_TO = 0.25
const MAX_PPM = 256.0
const WORLD_ID = 'grey_hive'
const SCENE_ID = 'gh_entry_maintenance'
const ROW_ID = 'entry_wall'
const ASSET_ID = 'runtime2d.grey_hive.wall_tiles.v1'
const WALL_IDS = [
	'gh_entry_wall_north',
	'gh_entry_wall_south',
	'gh_entry_wall_west',
	'gh_entry_wall_east_north',
	'gh_entry_wall_east_south'
]
const RUNS = [
	{'id':'north','collision_id':'gh_entry_wall_north','start':Vector3(0.0,0.0,0.25),'end':Vector3(20.0,0.0,0.25),'axis':'x'},
	{'id':'south','collision_id':'gh_entry_wall_south','start':Vector3(0.0,0.0,13.75),'end':Vector3(20.0,0.0,13.75),'axis':'x'},
	{'id':'west','collision_id':'gh_entry_wall_west','start':Vector3(0.25,0.0,0.5),'end':Vector3(0.25,0.0,13.5),'axis':'z'},
	{'id':'east_north','collision_id':'gh_entry_wall_east_north','start':Vector3(19.75,0.0,0.5),'end':Vector3(19.75,0.0,5.5),'axis':'z'},
	{'id':'east_south','collision_id':'gh_entry_wall_east_south','start':Vector3(19.75,0.0,8.5),'end':Vector3(19.75,0.0,13.5),'axis':'z'}
]
const COLLISION_POLYGONS = {
	'gh_entry_wall_north': [[0.0,0.0],[20.0,0.0],[20.0,0.5],[0.0,0.5]],
	'gh_entry_wall_south': [[0.0,13.5],[20.0,13.5],[20.0,14.0],[0.0,14.0]],
	'gh_entry_wall_west': [[0.0,0.5],[0.5,0.5],[0.5,13.5],[0.0,13.5]],
	'gh_entry_wall_east_north': [[19.5,0.5],[20.0,0.5],[20.0,5.5],[19.5,5.5]],
	'gh_entry_wall_east_south': [[19.5,8.5],[20.0,8.5],[20.0,13.5],[19.5,13.5]]
}
const MODULE_PACKET_KEYS = [
	'id','kind','context_key','run_id','index','p','ground_start','ground_end',
	'screen_start','screen_end','fade_group','texture','source_rect',
	'transform_origin','transform_rotation','transform_scale','body_rect'
]
const SHADOW_PACKET_KEYS = ['id','kind','context_key','collision_id','points','bounds_rect']

var attempted: bool = false
var module_texture: ImageTexture

func has_valid_context(snapshot, scene_data, current_key) -> bool:
	if not snapshot is Dictionary or not scene_data is Dictionary or typeof(current_key) != TYPE_STRING:
		return false
	if not P.snapshot(snapshot):
		return false
	if snapshot.get('worldId') != WORLD_ID or snapshot.get('sceneId') != SCENE_ID or scene_data.get('worldId') != WORLD_ID or scene_data.get('sceneId') != SCENE_ID:
		return false
	if not P.counter(snapshot.get('worldEpoch'), 1) or current_key != WORLD_ID + ':' + SCENE_ID + ':' + str(snapshot.get('worldEpoch')):
		return false
	return _valid_scene_row_and_collisions(scene_data)

func build_packets(snapshot, scene_data, current_key, projection) -> Array:
	if not has_valid_context(snapshot, scene_data, current_key) or not _valid_projection(projection):
		return []
	_load_once()
	if module_texture == null:
		return []
	var packets: Array = []
	for run in RUNS:
		var run_length: float = run.start.distance_to(run.end)
		if not is_finite(run_length) or run_length <= 0.0 or not is_equal_approx(run_length, roundf(run_length)):
			return []
		var count: int = int(roundf(run_length))
		if count > MAX_MODULES or packets.size() + count > MAX_MODULES:
			return []
		var direction: Vector3 = (run.end - run.start).normalized()
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
			var expected_step: float = float(projection.get('ppm')) * sqrt(1.25)
			if not _finite_vector2(screen_start) or not _finite_vector2(screen_end) or not _finite_vector2(transform_origin) or not is_finite(ratio) or ratio <= 0.0 or absf(endpoint_check.distance_to(screen_end)) > 0.01 or absf(screen_axis.length() - expected_step) > 0.01:
				return []
			if not _valid_rect(painted_rect) or maxf(painted_rect.size.x, painted_rect.size.y) > MAX_EXTENT_PX_AT_48 * (float(projection.get('ppm')) / 48.0) + 0.01:
				return []
			var packet_id: String = 'gh_entry:wall:%s:%02d' % [run.id, index]
			packets.append({
				'id':packet_id,
				'kind':'gh_entry_wall_module',
				'context_key':current_key,
				'run_id':run.id,
				'index':index,
				'p':(world_start + world_end) * 0.5,
				'ground_start':world_start,
				'ground_end':world_end,
				'screen_start':screen_start,
				'screen_end':screen_end,
				'fade_group':packet_id,
				'texture':module_texture,
				'source_rect':Rect2(FRAME),
				'transform_origin':transform_origin,
				'transform_rotation':rotation,
				'transform_scale':transform_scale,
				'body_rect':painted_rect
			})
	if packets.size() != 63 or not _packets_are_complete(packets, snapshot, scene_data, current_key, projection):
		return []
	return packets

func build_shadow_packets(snapshot, scene_data, current_key, projection, module_packets: Array) -> Array:
	if not has_valid_context(snapshot, scene_data, current_key) or not _valid_projection(projection) or not _packets_are_complete(module_packets, snapshot, scene_data, current_key, projection):
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
		packets.append({
			'id':'gh_entry:wall-shadow:' + wall_id,
			'kind':'gh_entry_wall_shadow',
			'context_key':current_key,
			'collision_id':wall_id,
			'points':points,
			'bounds_rect':bounds
		})
	return packets if _shadow_packets_are_complete(packets, snapshot, scene_data, current_key, projection, module_packets) else []

func has_complete_sets(snapshot, scene_data, current_key, projection, module_packets: Array, shadow_packets: Array, viewport: Rect2) -> bool:
	return _valid_rect(viewport) and _packets_are_complete(module_packets, snapshot, scene_data, current_key, projection) and _shadow_packets_are_complete(shadow_packets, snapshot, scene_data, current_key, projection, module_packets)

func suppressible_collision_ids(snapshot, scene_data, current_key, projection, module_packets: Array, shadow_packets: Array, viewport: Rect2) -> Array:
	if not has_complete_sets(snapshot, scene_data, current_key, projection, module_packets, shadow_packets, viewport):
		return []
	return WALL_IDS.duplicate()

func should_suppress_collision_fallback(collision_id, snapshot, scene_data, current_key, projection, module_packets: Array, shadow_packets: Array, viewport: Rect2) -> bool:
	return typeof(collision_id) == TYPE_STRING and collision_id in WALL_IDS and has_complete_sets(snapshot, scene_data, current_key, projection, module_packets, shadow_packets, viewport)

func module_intersects_viewport(packet, viewport: Rect2, snapshot, scene_data, current_key, projection, module_packets: Array) -> bool:
	if not _valid_rect(viewport) or not _packets_are_complete(module_packets, snapshot, scene_data, current_key, projection) or not packet is Dictionary or not module_packets.has(packet):
		return false
	var packet_index: int = int(packet.get('index', -1))
	var run = _run_by_id(packet.get('run_id'))
	if run.is_empty() or not _valid_module_packet(packet, run, packet_index, snapshot, scene_data, current_key, projection):
		return false
	return _rect_intersects(packet.body_rect, viewport)

func front_fade_alphas(packets: Array, current_by_group: Dictionary, player_rect: Rect2, delta: float, reduced_motion: bool, snapshot, scene_data, current_key, projection) -> Dictionary:
	var result: Dictionary = {}
	if not _valid_rect(player_rect) or not is_finite(delta) or not _packets_are_complete(packets, snapshot, scene_data, current_key, projection):
		return result
	for packet in packets:
		var group: String = packet.fade_group
		var body_rect: Rect2 = packet.body_rect
		var target: float = FADE_TO if _rect_intersects(body_rect, player_rect) else 1.0
		result[group] = step_front_alpha(float(current_by_group.get(group, 1.0)), target, delta, reduced_motion)
	return result if result.size() == 63 else {}

func step_front_alpha(current: float, target: float, delta: float, reduced_motion: bool) -> float:
	if not is_finite(current) or not is_finite(target) or not is_finite(delta):
		return 1.0
	var clamped_target: float = clampf(target, FADE_TO, 1.0)
	if reduced_motion:
		return clamped_target
	return move_toward(clampf(current, FADE_TO, 1.0), clamped_target, clampf(delta, 0.0, 0.1) * 3.75)

func draw_contact_shadows(canvas, packets: Array, viewport: Rect2, snapshot, scene_data, current_key, projection, module_packets: Array) -> bool:
	if not canvas is CanvasItem or not has_complete_sets(snapshot, scene_data, current_key, projection, module_packets, packets, viewport):
		return false
	for packet in packets:
		canvas.draw_colored_polygon(packet.points, Color(0.015, 0.025, 0.03, SHADOW_ALPHA))
	return true

func draw_module(canvas, packet, alpha: float, snapshot, scene_data, current_key, projection) -> bool:
	if not canvas is CanvasItem or not has_valid_context(snapshot, scene_data, current_key) or not _valid_projection(projection) or not is_finite(alpha) or alpha < FADE_TO or alpha > 1.0:
		return false
	if not packet is Dictionary or typeof(packet.get('run_id')) != TYPE_STRING or not P.counter(packet.get('index'), 0):
		return false
	var run = _run_by_id(packet.run_id)
	if run.is_empty() or not _valid_module_packet(packet, run, int(packet.index), snapshot, scene_data, current_key, projection):
		return false
	canvas.draw_set_transform(packet.transform_origin, packet.transform_rotation, packet.transform_scale)
	canvas.draw_texture_rect_region(module_texture, packet.source_rect, packet.source_rect, Color(1.0, 1.0, 1.0, alpha), false, true)
	canvas.draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
	return true

func _valid_scene_row_and_collisions(scene_data) -> bool:
	if not has_valid_layout(scene_data):
		return false
	var presentation = scene_data.get('presentation')
	if not presentation is Dictionary or not P.rows(presentation.get('sprites'), 2048) or not P.unique(presentation.sprites, 'id'):
		return false
	var row_count: int = 0
	for row in presentation.sprites:
		if row.get('id') == ROW_ID or row.get('assetId') == ASSET_ID:
			row_count += 1
			if not _matches_authored_row(row):
				return false
	if row_count != 1:
		return false
	var collisions = scene_data.get('collision')
	return P.rows(collisions, 512) and P.unique(collisions, 'id') and collisions.size() == WALL_IDS.size()

func has_valid_layout(scene_data) -> bool:
	if not scene_data is Dictionary or not P.counter(scene_data.get('schemaVersion'), 1, 1) or typeof(scene_data.get('worldId')) != TYPE_STRING or scene_data.get('worldId') != WORLD_ID or typeof(scene_data.get('sceneId')) != TYPE_STRING or scene_data.get('sceneId') != SCENE_ID:
		return false
	var bounds = scene_data.get('boundsM')
	if not bounds is Dictionary or not _keys_are(bounds, ['x','z','width','depth']):
		return false
	for key in ['x','z','width','depth']:
		if not P.number(bounds.get(key)):
			return false
	if float(bounds.x) != 0.0 or float(bounds.z) != 0.0 or float(bounds.width) != 20.0 or float(bounds.depth) != 14.0:
		return false
	var collisions = scene_data.get('collision')
	if not P.rows(collisions, 512) or collisions.size() != WALL_IDS.size() or not P.unique(collisions, 'id'):
		return false
	var found: Dictionary = {}
	for row in collisions:
		if not row is Dictionary:
			return false
		var collision_id = row.get('id')
		if typeof(collision_id) != TYPE_STRING or collision_id not in WALL_IDS or found.has(collision_id):
			return false
		if not _polygon_equals(row.get('polygon'), COLLISION_POLYGONS[collision_id]):
			return false
		found[collision_id] = true
	return found.size() == WALL_IDS.size()

func _matches_authored_row(row) -> bool:
	return row is Dictionary and _keys_are(row, ['id','assetId','layer','position']) and typeof(row.get('id')) == TYPE_STRING and row.get('id') == ROW_ID and typeof(row.get('assetId')) == TYPE_STRING and row.get('assetId') == ASSET_ID and typeof(row.get('layer')) == TYPE_STRING and row.get('layer') == 'visual.props_back' and _numbers_equal(row.get('position'), [1,0,2])

func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var manifest_file = FileAccess.open(MAPPING_RESOURCE, FileAccess.READ)
	if manifest_file == null:
		return
	var manifest_length: int = manifest_file.get_length()
	if manifest_length < 1 or manifest_length > 16384:
		manifest_file.close()
		return
	var manifest_bytes: PackedByteArray = manifest_file.get_buffer(manifest_length)
	manifest_file.close()
	if manifest_bytes.size() != manifest_length:
		return
	var manifest_text: String = manifest_bytes.get_string_from_utf8()
	if manifest_text.to_utf8_buffer() != manifest_bytes or not _valid_manifest(JSON.parse_string(manifest_text)):
		return
	var image_file = FileAccess.open(MODULE_TEXTURE_PATH, FileAccess.READ)
	if image_file == null:
		return
	if image_file.get_length() != MODULE_BYTES:
		image_file.close()
		return
	var png_bytes: PackedByteArray = image_file.get_buffer(MODULE_BYTES)
	image_file.close()
	if png_bytes.size() != MODULE_BYTES:
		return
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
	if value.get('schema') != 'wuxian.gh-entry-wall-modules.internal-use/v1' or value.get('registeredSourceReuse') != true or value.get('approval') != 'INTERNAL_REGISTERED_SOURCE_REUSE' or value.get('GodotDisplay') != 'NOT_RUN' or value.get('publicRelease') != 'NOT_ESTABLISHED':
		return false
	var source = value.get('sourceAsset')
	var asset = value.get('asset')
	var installation = value.get('installation')
	if not source is Dictionary or not asset is Dictionary or not installation is Dictionary:
		return false
	if not _keys_are(source, ['path','bytes','sha256','canvas','format']) or source.get('path') != SOURCE_PATH or not P.counter(source.get('bytes'), MODULE_BYTES, MODULE_BYTES) or source.get('sha256') != MODULE_SHA256 or source.get('format') != 'RGBA8' or not _integer_array_equals(source.get('canvas'), [1418,1028]):
		return false
	if not _keys_are(asset, ['path','bytes','sha256','format','canvas','frame','paintedBounds','groundFeetPx']):
		return false
	if asset.get('path') != MODULE_TEXTURE_PATH or not P.counter(asset.get('bytes'), MODULE_BYTES, MODULE_BYTES) or asset.get('sha256') != MODULE_SHA256 or asset.get('format') != 'RGBA8' or not _integer_array_equals(asset.get('canvas'), [1418,1028]) or not _integer_array_equals(asset.get('frame'), [0,0,355,480]) or not _integer_array_equals(asset.get('paintedBounds'), [41,6,299,457]) or not _integer_array_equals(asset.get('groundFeetPx'), [[50,322],[300,460]]):
		return false
	if not _keys_are(installation, ['worldId','sceneId','rowId','assetId','layer','positionM','runIds','runs','cadenceM','maxModules','maxExtentPxAt48','shadowAlpha','fadeTo','fadeDurationMs']):
		return false
	if installation.get('worldId') != WORLD_ID or installation.get('sceneId') != SCENE_ID or installation.get('rowId') != ROW_ID or installation.get('assetId') != ASSET_ID or installation.get('layer') != 'visual.props_back' or not _numbers_equal(installation.get('positionM'), [1,0,2]) or installation.get('runIds') != ['north','south','west','east_north','east_south']:
		return false
	if not P.counter(installation.get('cadenceM'), 1, 1) or not P.counter(installation.get('maxModules'), MAX_MODULES, MAX_MODULES) or not P.counter(installation.get('maxExtentPxAt48'), int(MAX_EXTENT_PX_AT_48), int(MAX_EXTENT_PX_AT_48)) or installation.get('shadowAlpha') != SHADOW_ALPHA or installation.get('fadeTo') != FADE_TO or not P.counter(installation.get('fadeDurationMs'), 200, 200):
		return false
	var mapped_runs = installation.get('runs')
	if not mapped_runs is Array or mapped_runs.size() != RUNS.size():
		return false
	for index in range(RUNS.size()):
		var row = mapped_runs[index]
		var expected = RUNS[index]
		if not row is Dictionary or not _keys_are(row, ['id','startM','endM']) or row.get('id') != expected.id or not _numbers_equal(row.get('startM'), [expected.start.x,expected.start.y,expected.start.z]) or not _numbers_equal(row.get('endM'), [expected.end.x,expected.end.y,expected.end.z]):
			return false
	return true

func _valid_projection(projection) -> bool:
	if not projection is RefCounted or not projection.has_method('project'):
		return false
	return P.number(projection.get('ppm'), 0.000001, MAX_PPM)

func _packets_are_complete(packets: Array, snapshot, scene_data, current_key, projection) -> bool:
	if packets.size() != 63 or module_texture == null or not has_valid_context(snapshot, scene_data, current_key) or not _valid_projection(projection):
		return false
	var cursor: int = 0
	for run in RUNS:
		var count: int = int(roundf(run.start.distance_to(run.end)))
		for index in range(count):
			if cursor >= packets.size() or not _valid_module_packet_content(packets[cursor], run, index, current_key, projection):
				return false
			cursor += 1
	return cursor == 63

func _valid_module_packet(packet, run: Dictionary, index: int, snapshot, scene_data, current_key, projection) -> bool:
	if not has_valid_context(snapshot, scene_data, current_key) or not _valid_projection(projection):
		return false
	return _valid_module_packet_content(packet, run, index, current_key, projection)

func _valid_module_packet_content(packet, run: Dictionary, index: int, current_key, projection) -> bool:
	if not packet is Dictionary or not _keys_are(packet, MODULE_PACKET_KEYS):
		return false
	if index < 0 or index >= int(roundf(run.start.distance_to(run.end))):
		return false
	var packet_id: String = 'gh_entry:wall:%s:%02d' % [run.id,index]
	if packet.get('id') != packet_id or packet.get('kind') != 'gh_entry_wall_module' or packet.get('context_key') != current_key or packet.get('run_id') != run.id or not P.counter(packet.get('index'), index, index) or packet.get('fade_group') != packet_id or packet.get('texture') != module_texture or packet.get('source_rect') != Rect2(FRAME):
		return false
	var direction: Vector3 = (run.end - run.start).normalized()
	var world_start: Vector3 = run.start + direction * float(index)
	var world_end: Vector3 = run.start + direction * float(index + 1)
	var midpoint: Vector3 = (world_start + world_end) * 0.5
	if not packet.get('p') is Vector3 or not _finite_vector3(packet.get('p')) or packet.p.distance_to(midpoint) > 0.000001 or not packet.get('ground_start') is Vector3 or not _finite_vector3(packet.get('ground_start')) or packet.ground_start.distance_to(world_start) > 0.000001 or not packet.get('ground_end') is Vector3 or not _finite_vector3(packet.get('ground_end')) or packet.ground_end.distance_to(world_end) > 0.000001:
		return false
	var screen_start: Vector2 = projection.project(world_start)
	var screen_end: Vector2 = projection.project(world_end)
	if not packet.get('screen_start') is Vector2 or not packet.get('screen_end') is Vector2:
		return false
	if not _finite_vector2(packet.screen_start) or not _finite_vector2(packet.screen_end) or packet.screen_start.distance_to(screen_start) > 0.001 or packet.screen_end.distance_to(screen_end) > 0.001:
		return false
	var screen_axis: Vector2 = screen_end - screen_start
	var ratio: float = screen_axis.length() / (FOOT_B - FOOT_A).length()
	var reflected: bool = run.axis == 'z'
	var source_angle: float = (FOOT_B - FOOT_A).angle()
	var reflected_source_angle: float = PI - source_angle if reflected else source_angle
	var rotation: float = screen_axis.angle() - reflected_source_angle
	var expected_scale: Vector2 = Vector2(-ratio, ratio) if reflected else Vector2(ratio, ratio)
	var expected_origin: Vector2 = screen_start - _source_to_screen(FOOT_A, rotation, expected_scale)
	if not packet.get('transform_origin') is Vector2 or not _finite_vector2(packet.get('transform_origin')) or packet.transform_origin.distance_to(expected_origin) > 0.001 or not P.number(packet.get('transform_rotation'), -1000.0, 1000.0) or absf(float(packet.transform_rotation) - rotation) > 0.000001 or not packet.get('transform_scale') is Vector2 or not _finite_vector2(packet.get('transform_scale')) or packet.transform_scale.distance_to(expected_scale) > 0.000001:
		return false
	if absf(screen_axis.length() - float(projection.get('ppm')) * sqrt(1.25)) > 0.01:
		return false
	var expected_rect: Rect2 = _transformed_bounds(expected_origin, rotation, expected_scale, PAINTED_BOUNDS)
	var actual_rect = packet.get('body_rect')
	if not actual_rect is Rect2 or not _rect_close(actual_rect, expected_rect) or maxf(actual_rect.size.x, actual_rect.size.y) > MAX_EXTENT_PX_AT_48 * (float(projection.get('ppm')) / 48.0) + 0.01:
		return false
	return true

func _shadow_packets_are_complete(packets: Array, snapshot, scene_data, current_key, projection, module_packets: Array) -> bool:
	if packets.size() != WALL_IDS.size() or not _packets_are_complete(module_packets, snapshot, scene_data, current_key, projection):
		return false
	for index in range(WALL_IDS.size()):
		var wall_id: String = WALL_IDS[index]
		var packet = packets[index]
		if not packet is Dictionary or not _keys_are(packet, SHADOW_PACKET_KEYS) or packet.get('id') != 'gh_entry:wall-shadow:' + wall_id or packet.get('kind') != 'gh_entry_wall_shadow' or packet.get('context_key') != current_key or packet.get('collision_id') != wall_id:
			return false
		var expected_points := PackedVector2Array()
		for point in COLLISION_POLYGONS[wall_id]:
			var projected: Vector2 = projection.project(Vector3(float(point[0]), 0.0, float(point[1])))
			if not _finite_vector2(projected):
				return false
			expected_points.append(projected)
		var points = packet.get('points')
		if not points is PackedVector2Array or points.size() != 4:
			return false
		for point_index in range(4):
			if not _finite_vector2(points[point_index]) or points[point_index].distance_to(expected_points[point_index]) > 0.001:
				return false
		var bounds = packet.get('bounds_rect')
		if not bounds is Rect2 or not _rect_close(bounds, _points_bounds(expected_points)):
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
	return _points_bounds(points)

func _points_bounds(points) -> Rect2:
	if points.is_empty():
		return Rect2()
	var min_point: Vector2 = points[0]
	var max_point: Vector2 = points[0]
	for point in points:
		min_point = Vector2(minf(min_point.x, point.x), minf(min_point.y, point.y))
		max_point = Vector2(maxf(max_point.x, point.x), maxf(max_point.y, point.y))
	return Rect2(min_point, max_point - min_point)

func _run_by_id(run_id) -> Dictionary:
	if typeof(run_id) != TYPE_STRING:
		return {}
	for run in RUNS:
		if run.id == run_id:
			return run
	return {}

func _polygon_equals(value, expected: Array) -> bool:
	return _numbers_equal(value, expected)

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

func _integer_array_equals(value, expected: Array) -> bool:
	if not value is Array or value.size() != expected.size():
		return false
	for index in range(expected.size()):
		if expected[index] is Array:
			if not _integer_array_equals(value[index], expected[index]):
				return false
		elif not P.counter(value[index], int(expected[index]), int(expected[index])):
			return false
	return true

func _keys_are(value: Dictionary, expected: Array) -> bool:
	var actual: Array = value.keys()
	var wanted: Array = expected.duplicate()
	actual.sort()
	wanted.sort()
	return actual == wanted

func _rect_intersects(a: Rect2, b: Rect2) -> bool:
	return _valid_rect(a) and _valid_rect(b) and a.intersects(b)

func _rect_close(actual: Rect2, expected: Rect2) -> bool:
	return _valid_rect(actual) and _valid_rect(expected) and actual.position.distance_to(expected.position) <= 0.001 and actual.size.distance_to(expected.size) <= 0.001

func _valid_rect(value: Rect2) -> bool:
	return _finite_vector2(value.position) and _finite_vector2(value.size) and value.size.x > 0.0 and value.size.y > 0.0 and is_finite(value.end.x) and is_finite(value.end.y)

func _finite_vector2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _finite_vector3(value: Vector3) -> bool:
	return is_finite(value.x) and is_finite(value.y) and is_finite(value.z)
