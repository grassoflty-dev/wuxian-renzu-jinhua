extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')

const WORLD_ID = 'mist_harbor'
const SCENE_ID = 'mh_fog_pier'
const PACKET_KIND = 'mh_fog_pier_layered_fog_v1'
const DEFINITION_RESOURCE = 'res://data/mh_fog_pier_layered_fog.internal-use.json'
const TEXTURE_RESOURCE = 'res://assets/props/mist_harbor/fog_pier_fog_bank_source.png'
const ASSET_ID = 'runtime2d.world.mistharbor.fog_bank.v1'
const OUTPUT_BYTES = 684960
const OUTPUT_SHA256 = '9bbbfd9783f70a8b1f6ffeaabd44e568074f4c20f1f7500d7ef4f0e534edeac2'
const CANVAS_WIDTH = 1254
const CANVAS_HEIGHT = 1236
const FRAME = [27, 370, 1201, 498]
const MAX_DEFINITION_BYTES = 16384
const PERIOD_SECONDS = 120.0
const DELTA_CAP_SECONDS = 0.25
const WORLD_KEY_PREFIX = 'mist_harbor:mh_fog_pier:'
const BANK_IDS = ['approved_fog_bank_north', 'approved_fog_bank_east']
const BAND_ORDER = ['back', 'mid', 'front']
const BAND_SPECS = {
	'back': {'width': 9.0, 'offset': [0.0, -1.2], 'rgb': [0.70, 0.79, 0.83], 'alpha': 0.080, 'amplitude': [0.30, 0.08], 'cycles': 1},
	'mid': {'width': 7.0, 'offset': [-0.6, -0.4], 'rgb': [0.70, 0.79, 0.83], 'alpha': 0.055, 'amplitude': [0.22, 0.06], 'cycles': 2},
	'front': {'width': 5.0, 'offset': [0.5, 0.35], 'rgb': [0.70, 0.79, 0.83], 'alpha': 0.035, 'amplitude': [0.14, 0.04], 'cycles': 3}
}
const TOP_LEVEL_KEYS = ['schema', 'registeredSourceReuse', 'approval', 'GodotDisplay', 'publicRelease', 'asset', 'frame', 'rows', 'bands', 'animation', 'densityAlphaCap']
const ASSET_KEYS = ['assetId', 'path', 'bytes', 'sha256', 'canvas', 'format']
const FRAME_KEYS = ['kind', 'xywh', 'sourcePixelEdgeCorners', 'uvConvention', 'omitsFaintAlphaOutliers', 'atlasOffsetUsed']
const ROW_KEYS = ['id', 'assetId', 'layer', 'position', 'driftSign']
const BAND_KEYS = ['band', 'widthPerPpm', 'centerOffsetPerPpm', 'rgbMultiplier', 'alphaMultiplier', 'driftAmplitudePerPpm', 'cyclesPer120Seconds']
const ANIMATION_KEYS = ['owner', 'periodSeconds', 'deltaCapSeconds', 'reducedMotion', 'contextChange', 'serverAuthority']
const PACKET_KEYS = ['kind', 'worldId', 'sceneId', 'worldEpoch', 'currentKey', 'ppm', 'phaseSeconds', 'reducedMotion', 'billboards']
const BILLBOARD_KEYS = ['id', 'band', 'sourceRowId', 'anchor', 'screenRect', 'screenPoints', 'uvs', 'tint']

var _definition_attempted: bool = false
var _definition: Dictionary = {}
var _texture_attempted: bool = false
var _cached_texture: Texture2D = null

func _finite_number(value) -> bool:
	return (typeof(value) == TYPE_INT or typeof(value) == TYPE_FLOAT) and is_finite(float(value))

func _same_number(value, expected: float) -> bool:
	return _finite_number(value) and float(value) == expected

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

func _exact_keys(value: Dictionary, expected: Array) -> bool:
	if value.size() != expected.size():
		return false
	for key in expected:
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
	for index in range(expected.size()):
		if not _numbers_match(value[index], expected[index]):
			return false
	return true

func is_definition_valid(value) -> bool:
	if typeof(value) != TYPE_DICTIONARY or not _exact_keys(value, TOP_LEVEL_KEYS):
		return false
	if value.get('schema') != 'wuxian.mh-fog-pier-layered-fog.internal-use/v1' or typeof(value.get('registeredSourceReuse')) != TYPE_BOOL or value.registeredSourceReuse != true:
		return false
	if value.get('approval') != 'INTERNAL_REGISTERED_SOURCE_REUSE' or value.get('GodotDisplay') != 'NOT_RUN' or value.get('publicRelease') != 'NOT_ESTABLISHED':
		return false
	if not _same_number(value.get('densityAlphaCap'), 0.30):
		return false
	var asset = value.get('asset')
	if typeof(asset) != TYPE_DICTIONARY or not _exact_keys(asset, ASSET_KEYS):
		return false
	if asset.get('assetId') != ASSET_ID or asset.get('path') != TEXTURE_RESOURCE or not _same_number(asset.get('bytes'), float(OUTPUT_BYTES)):
		return false
	if asset.get('sha256') != OUTPUT_SHA256 or not _numbers_match(asset.get('canvas'), [CANVAS_WIDTH, CANVAS_HEIGHT]) or asset.get('format') != 'RGBA8':
		return false
	var frame = value.get('frame')
	if typeof(frame) != TYPE_DICTIONARY or not _exact_keys(frame, FRAME_KEYS):
		return false
	if frame.get('kind') != 'authored_display_crop' or frame.get('uvConvention') != 'sourcePixelEdge / standaloneCanvasSize':
		return false
	if typeof(frame.get('omitsFaintAlphaOutliers')) != TYPE_BOOL or frame.omitsFaintAlphaOutliers != true or typeof(frame.get('atlasOffsetUsed')) != TYPE_BOOL or frame.atlasOffsetUsed != false:
		return false
	if not _numbers_match(frame.get('xywh'), FRAME) or not _number_matrix_match(frame.get('sourcePixelEdgeCorners'), [[27, 370], [1228, 370], [1228, 868], [27, 868]]):
		return false
	var rows = value.get('rows')
	if typeof(rows) != TYPE_ARRAY or rows.size() != 2:
		return false
	var row_expectations = [
		{'id': 'approved_fog_bank_north', 'position': [13, 0, 2.5], 'driftSign': 1},
		{'id': 'approved_fog_bank_east', 'position': [20, 0, 11], 'driftSign': -1}
	]
	for index in range(rows.size()):
		var row = rows[index]
		var expected = row_expectations[index]
		if typeof(row) != TYPE_DICTIONARY or not _exact_keys(row, ROW_KEYS):
			return false
		if row.get('id') != expected.id or row.get('assetId') != ASSET_ID or row.get('layer') != 'visual.props_back':
			return false
		if not _numbers_match(row.get('position'), expected.position) or not _same_number(row.get('driftSign'), float(expected.driftSign)):
			return false
	var bands = value.get('bands')
	if typeof(bands) != TYPE_ARRAY or bands.size() != BAND_ORDER.size():
		return false
	for index in range(BAND_ORDER.size()):
		var row = bands[index]
		var band = BAND_ORDER[index]
		var expected = BAND_SPECS[band]
		if typeof(row) != TYPE_DICTIONARY or not _exact_keys(row, BAND_KEYS) or row.get('band') != band:
			return false
		if not _same_number(row.get('widthPerPpm'), expected.width) or not _numbers_match(row.get('centerOffsetPerPpm'), expected.offset):
			return false
		if not _numbers_match(row.get('rgbMultiplier'), expected.rgb) or not _same_number(row.get('alphaMultiplier'), expected.alpha):
			return false
		if not _numbers_match(row.get('driftAmplitudePerPpm'), expected.amplitude) or not _same_number(row.get('cyclesPer120Seconds'), float(expected.cycles)):
			return false
	var animation = value.get('animation')
	if typeof(animation) != TYPE_DICTIONARY or not _exact_keys(animation, ANIMATION_KEYS):
		return false
	if animation.get('owner') != 'existing Renderer _process(delta)' or not _same_number(animation.get('periodSeconds'), PERIOD_SECONDS):
		return false
	if not _same_number(animation.get('deltaCapSeconds'), DELTA_CAP_SECONDS) or animation.get('reducedMotion') != 'hold accumulator; static phase zero':
		return false
	if animation.get('contextChange') != 'reset0' or typeof(animation.get('serverAuthority')) != TYPE_BOOL or animation.serverAuthority != false:
		return false
	return true

func _valid_projection(projection) -> bool:
	if not projection is RefCounted or projection.get_script() != ProjectionScript:
		return false
	var origin = projection.get('origin')
	var screen_center = projection.get('screen_center')
	var viewport = projection.get('viewport')
	var ppm = projection.get('ppm')
	return _finite_vector3(origin) and _finite_vector2(screen_center) and _finite_vector2(viewport) and viewport.x > 0.0 and viewport.y > 0.0 and _finite_number(ppm) and float(ppm) > 0.0 and float(ppm) <= 256.0

func _valid_bounds(scene: Dictionary) -> bool:
	var bounds = scene.get('boundsM')
	return typeof(bounds) == TYPE_DICTIONARY and _exact_keys(bounds, ['x', 'z', 'width', 'depth']) and _same_number(bounds.get('x'), 0.0) and _same_number(bounds.get('z'), 0.0) and _same_number(bounds.get('width'), 24.0) and _same_number(bounds.get('depth'), 16.0)

func _same_point2(value, x: float, z: float) -> bool:
	return typeof(value) == TYPE_ARRAY and value.size() == 2 and _same_number(value[0], x) and _same_number(value[1], z)

func _valid_walkable(scene: Dictionary) -> bool:
	var polygons = scene.get('walkablePolygons')
	if typeof(polygons) != TYPE_ARRAY or polygons.size() != 1:
		return false
	var row = polygons[0]
	if typeof(row) != TYPE_DICTIONARY or not _exact_keys(row, ['id', 'polygon']) or row.get('id') != 'mh_fog_pier_walkable_area':
		return false
	var polygon = row.get('polygon')
	return typeof(polygon) == TYPE_ARRAY and polygon.size() == 4 and _same_point2(polygon[0], 0.5, 0.5) and _same_point2(polygon[1], 23.5, 0.5) and _same_point2(polygon[2], 23.5, 15.5) and _same_point2(polygon[3], 0.5, 15.5)

func _valid_presentation_sprites(scene: Dictionary) -> bool:
	var presentation = scene.get('presentation')
	if typeof(presentation) != TYPE_DICTIONARY:
		return false
	var sprites = presentation.get('sprites')
	if typeof(sprites) != TYPE_ARRAY or sprites.size() > 64:
		return false
	var seen_ids: Dictionary = {}
	var bank_count = 0
	for row in sprites:
		if typeof(row) != TYPE_DICTIONARY:
			return false
		var id = row.get('id')
		var asset_id = row.get('assetId')
		var layer = row.get('layer')
		var position = row.get('position')
		if not P.ident(id) or seen_ids.has(id) or not P.ident(asset_id) or not P.ident(layer) or typeof(position) != TYPE_ARRAY or position.size() != 3:
			return false
		for coordinate in position:
			if not _finite_number(coordinate):
				return false
		seen_ids[id] = true
		if id in BANK_IDS or asset_id == ASSET_ID:
			bank_count += 1
			if not _exact_keys(row, ['id', 'assetId', 'layer', 'position']):
				return false
			if id == BANK_IDS[0]:
				if asset_id != ASSET_ID or layer != 'visual.props_back' or not _numbers_match(position, [13, 0, 2.5]):
					return false
			elif id == BANK_IDS[1]:
				if asset_id != ASSET_ID or layer != 'visual.props_back' or not _numbers_match(position, [20, 0, 11]):
					return false
			else:
				return false
	return bank_count == 2 and seen_ids.has(BANK_IDS[0]) and seen_ids.has(BANK_IDS[1])

func _valid_context(snapshot, scene, current_key, projection) -> bool:
	if not _valid_projection(projection):
		return false
	if typeof(snapshot) != TYPE_DICTIONARY or not P.snapshot(snapshot) or snapshot.get('worldId') != WORLD_ID or snapshot.get('sceneId') != SCENE_ID or not P.counter(snapshot.get('worldEpoch'), 1):
		return false
	if typeof(current_key) != TYPE_STRING or current_key != WORLD_KEY_PREFIX + str(snapshot.get('worldEpoch')):
		return false
	if typeof(scene) != TYPE_DICTIONARY or not P.counter(scene.get('schemaVersion'), 1, 1) or scene.get('worldId') != WORLD_ID or scene.get('sceneId') != SCENE_ID:
		return false
	return _valid_bounds(scene) and _valid_walkable(scene) and _valid_presentation_sprites(scene)

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
	if parser.parse(text) != OK or typeof(parser.data) != TYPE_DICTIONARY or not is_definition_valid(parser.data):
		return {}
	_definition = parser.data.duplicate(true)
	return _definition

func advance_visual_time(current, delta, reduced_motion):
	if not _finite_number(current) or float(current) < 0.0 or float(current) >= PERIOD_SECONDS:
		return 0.0
	var valid_current: float = float(current)
	if typeof(delta) != TYPE_INT and typeof(delta) != TYPE_FLOAT or not _finite_number(delta) or float(delta) < 0.0 or typeof(reduced_motion) != TYPE_BOOL:
		return valid_current
	if reduced_motion:
		return valid_current
	return fposmod(valid_current + minf(float(delta), DELTA_CAP_SECONDS), PERIOD_SECONDS)

func _finite_phase(phase_seconds) -> bool:
	return _finite_number(phase_seconds) and float(phase_seconds) >= 0.0 and float(phase_seconds) < PERIOD_SECONDS

func _uvs() -> Array:
	return [
		Vector2(27.0 / CANVAS_WIDTH, 370.0 / CANVAS_HEIGHT),
		Vector2(1228.0 / CANVAS_WIDTH, 370.0 / CANVAS_HEIGHT),
		Vector2(1228.0 / CANVAS_WIDTH, 868.0 / CANVAS_HEIGHT),
		Vector2(27.0 / CANVAS_WIDTH, 868.0 / CANVAS_HEIGHT)
	]

func _candidate_rows(scene: Dictionary) -> Dictionary:
	var result: Dictionary = {}
	for row in scene.presentation.sprites:
		if row.id in BANK_IDS:
			result[row.id] = row
	return result

func _canonical_packet(snapshot: Dictionary, scene: Dictionary, current_key: String, projection, phase_seconds: float, reduced_motion: bool) -> Dictionary:
	var effective_phase: float = 0.0 if reduced_motion else phase_seconds
	var anchors = _candidate_rows(scene)
	if anchors.size() != 2:
		return {}
	var billboards: Array = []
	for band in BAND_ORDER:
		var spec: Dictionary = BAND_SPECS[band]
		var width: float = float(spec.width) * float(projection.get('ppm'))
		var height: float = width * 498.0 / 1201.0
		var cycles: float = float(spec.cycles)
		var angle: float = TAU * cycles * effective_phase / PERIOD_SECONDS
		for source_row_id in BANK_IDS:
			var source_row: Dictionary = anchors[source_row_id]
			var anchor_values: Array = source_row.position
			var anchor := Vector3(float(anchor_values[0]), float(anchor_values[1]), float(anchor_values[2]))
			var projected = projection.call('project', anchor)
			if not _finite_vector2(projected):
				return {}
			var drift_sign: float = 1.0 if source_row_id == BANK_IDS[0] else -1.0
			var amplitude: Array = spec.amplitude
			var drift := Vector2(drift_sign * float(amplitude[0]) * sin(angle), drift_sign * float(amplitude[1]) * sin(2.0 * angle))
			var offset: Array = spec.offset
			var center: Vector2 = projected + (Vector2(float(offset[0]), float(offset[1])) + drift) * float(projection.get('ppm'))
			var rect := Rect2(center - Vector2(width, height) * 0.5, Vector2(width, height))
			if not _finite_rect(rect, true):
				return {}
			var points: Array = [rect.position, Vector2(rect.end.x, rect.position.y), rect.end, Vector2(rect.position.x, rect.end.y)]
			for point in points:
				if not _finite_vector2(point):
					return {}
			var rgb: Array = spec.rgb
			var tint := Color(float(rgb[0]), float(rgb[1]), float(rgb[2]), float(spec.alpha))
			var tints: Array = [tint, tint, tint, tint]
			billboards.append({
				'id': source_row_id + ':' + band,
				'band': band,
				'sourceRowId': source_row_id,
				'anchor': anchor,
				'screenRect': rect,
				'screenPoints': points,
				'uvs': _uvs(),
				'tint': tints
			})
	if billboards.size() != 6:
		return {}
	return {
		'kind': PACKET_KIND,
		'worldId': WORLD_ID,
		'sceneId': SCENE_ID,
		'worldEpoch': snapshot.get('worldEpoch'),
		'currentKey': current_key,
		'ppm': float(projection.get('ppm')),
		'phaseSeconds': effective_phase,
		'reducedMotion': reduced_motion,
		'billboards': billboards
	}

func build_packet(snapshot, scene, current_key, projection, phase_seconds, reduced_motion) -> Dictionary:
	if not _valid_context(snapshot, scene, current_key, projection) or not _finite_phase(phase_seconds) or typeof(reduced_motion) != TYPE_BOOL:
		return {}
	if _load_definition_after_context().is_empty():
		return {}
	return _canonical_packet(snapshot, scene, current_key, projection, float(phase_seconds), reduced_motion)

func _billboard_shape_valid(row) -> bool:
	if typeof(row) != TYPE_DICTIONARY or not _exact_keys(row, BILLBOARD_KEYS):
		return false
	if typeof(row.get('id')) != TYPE_STRING or typeof(row.get('band')) != TYPE_STRING or typeof(row.get('sourceRowId')) != TYPE_STRING:
		return false
	if not _finite_vector3(row.get('anchor')) or not _finite_rect(row.get('screenRect'), true):
		return false
	for field in ['screenPoints', 'uvs', 'tint']:
		if typeof(row.get(field)) != TYPE_ARRAY or row[field].size() != 4:
			return false
	for point in row.screenPoints:
		if not _finite_vector2(point):
			return false
	for uv in row.uvs:
		if not _finite_vector2(uv):
			return false
	for tint in row.tint:
		if not _finite_color(tint):
			return false
	return row.tint[0] == row.tint[1] and row.tint[1] == row.tint[2] and row.tint[2] == row.tint[3]

func is_packet_valid(packet, snapshot, scene, current_key, projection, phase_seconds, reduced_motion) -> bool:
	if not _valid_context(snapshot, scene, current_key, projection) or not _finite_phase(phase_seconds) or typeof(reduced_motion) != TYPE_BOOL:
		return false
	if _load_definition_after_context().is_empty():
		return false
	if typeof(packet) != TYPE_DICTIONARY or not _exact_keys(packet, PACKET_KEYS):
		return false
	if packet.get('kind') != PACKET_KIND or packet.get('worldId') != WORLD_ID or packet.get('sceneId') != SCENE_ID:
		return false
	if not _same_number(packet.get('worldEpoch'), float(snapshot.get('worldEpoch'))) or typeof(packet.get('currentKey')) != TYPE_STRING or packet.get('currentKey') != current_key:
		return false
	if not _same_number(packet.get('ppm'), float(projection.get('ppm'))) or not _finite_phase(packet.get('phaseSeconds')) or typeof(packet.get('reducedMotion')) != TYPE_BOOL:
		return false
	var billboards = packet.get('billboards')
	if typeof(billboards) != TYPE_ARRAY or billboards.size() != 6:
		return false
	for row in billboards:
		if not _billboard_shape_valid(row):
			return false
	var expected: Dictionary = _canonical_packet(snapshot, scene, current_key, projection, float(phase_seconds), reduced_motion)
	return not expected.is_empty() and packet == expected

func rect_intersects_viewport(rect, viewport) -> bool:
	if not _finite_rect(rect, true) or not _finite_rect(viewport, true):
		return false
	var bounds: Rect2 = rect
	var clip: Rect2 = viewport
	return bounds.position.x < clip.end.x and bounds.end.x > clip.position.x and bounds.position.y < clip.end.y and bounds.end.y > clip.position.y

func _sha256(bytes: PackedByteArray) -> String:
	var hashing := HashingContext.new()
	if hashing.start(HashingContext.HASH_SHA256) != OK or hashing.update(bytes) != OK:
		return ''
	return hashing.finish().hex_encode().to_lower()

func decode_source(bytes) -> Image:
	if typeof(bytes) != TYPE_PACKED_BYTE_ARRAY:
		return null
	if bytes.size() != OUTPUT_BYTES or _sha256(bytes) != OUTPUT_SHA256:
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
	if file == null or file.get_length() != OUTPUT_BYTES:
		if file != null:
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

func draw_phase(canvas, packet, snapshot, scene, current_key, projection, phase_seconds, reduced_motion, viewport, band) -> bool:
	if not canvas is CanvasItem:
		return false
	if not _finite_rect(viewport, true) or typeof(band) != TYPE_STRING or not band in BAND_ORDER:
		return false
	if not is_packet_valid(packet, snapshot, scene, current_key, projection, phase_seconds, reduced_motion):
		return false
	var visible: Array = []
	for row in packet.billboards:
		if row.band == band and rect_intersects_viewport(row.screenRect, viewport):
			visible.append(row)
	if visible.is_empty():
		return true
	var texture: Texture2D = _load_texture_after_visible_validation()
	if texture == null:
		return false
	var drawing_canvas: CanvasItem = canvas
	for row in visible:
		drawing_canvas.draw_polygon(PackedVector2Array(row.screenPoints), PackedColorArray(row.tint), PackedVector2Array(row.uvs), texture)
	return true

func suppressed_bank_ids(packet, snapshot, scene, current_key, projection, phase_seconds, reduced_motion, phase_results) -> Array:
	if typeof(phase_results) != TYPE_ARRAY or phase_results.size() != 3:
		return []
	for result in phase_results:
		if typeof(result) != TYPE_BOOL or result != true:
			return []
	if not is_packet_valid(packet, snapshot, scene, current_key, projection, phase_seconds, reduced_motion):
		return []
	return BANK_IDS.duplicate()
