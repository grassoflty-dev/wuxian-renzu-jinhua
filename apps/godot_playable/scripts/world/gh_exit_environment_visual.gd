extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const MAP_PATH = 'res://data/gh_exit_environment.internal-use.json'
const SCENE_PATH = 'res://data/scenes/gh_exit.json'
const SCENE_BYTES = 5617
const SCENE_SHA256 = '6ab262467e1992feac94f27d8bf571dcf77ddbc8c1693a79c57b3aeebff0c54c'
const MAX_PPM = 256.0
const MAX_MAP_BYTES = 16384
const SHADOW_STEPS = 24
const EXTERIOR_IDS = [
	'exit_wall_north', 'exit_wall_south', 'exit_wall_west_north',
	'exit_wall_west_south', 'exit_wall_east_north', 'exit_wall_east_south'
]
const LOW_WALLS = {
	'exit_wall_south': {
		'inner_a': [0, 15.5], 'inner_b': [24, 15.5],
		'outer_a': [0, 16], 'outer_b': [24, 16],
		'contact': [[0,15.5],[24,15.5],[24,15.55],[0,15.55]], 'axis': 'x'
	},
	'exit_wall_west_north': {
		'inner_a': [0.5,0.5], 'inner_b': [0.5,6.5],
		'outer_a': [0,0.5], 'outer_b': [0,6.5],
		'contact': [[0.5,0.5],[0.45,0.5],[0.45,6.5],[0.5,6.5]], 'axis': 'z'
	},
	'exit_wall_west_south': {
		'inner_a': [0.5,9.5], 'inner_b': [0.5,15.5],
		'outer_a': [0,9.5], 'outer_b': [0,15.5],
		'contact': [[0.5,9.5],[0.45,9.5],[0.45,15.5],[0.5,15.5]], 'axis': 'z'
	},
	'exit_wall_east_north': {
		'inner_a': [23.5,0.5], 'inner_b': [23.5,6.5],
		'outer_a': [24,0.5], 'outer_b': [24,6.5],
		'contact': [[23.5,0.5],[23.55,0.5],[23.55,6.5],[23.5,6.5]], 'axis': 'z'
	},
	'exit_wall_east_south': {
		'inner_a': [23.5,9.5], 'inner_b': [23.5,15.5],
		'outer_a': [24,9.5], 'outer_b': [24,15.5],
		'contact': [[23.5,9.5],[23.55,9.5],[23.55,15.5],[23.5,15.5]], 'axis': 'z'
	}
}
const SEALED_MAP = {
	'schema': 'wuxian.gh-exit-environment.internal-use/v1',
	'registeredSourceReuse': true,
	'approval': 'INTERNAL_REGISTERED_SOURCE_REUSE',
	'GodotDisplay': 'NOT_RUN',
	'publicRelease': 'NOT_ESTABLISHED',
	'worldId': 'grey_hive',
	'sceneId': 'gh_exit',
	'canonicalScene': {'path':'res://data/scenes/gh_exit.json','bytes':5617,'sha256':SCENE_SHA256},
	'floor': {
		'path':'res://assets/props/grey_hive/floor_tiles.png','bytes':1315059,
		'sha256':'4c93e192505e71d6ec4e414aeb5c9b76376b7407d677017b31b8eba5e24e744f',
		'canvas':[1434,980],'format':'RGBA8','assetId':'runtime2d.grey_hive.floor_tiles.v1',
		'rowId':'exit_approved_floor','layer':'visual.floor','positionM':[12,0,8],
		'selection':{'kind':'affine_top_face','pixelCenterCorners':[[187,184],[345,305],[185,426],[27,305]],'uvConvention':'(sourcePixelCenter + 0.5) / canvasSize','interiorAlphaRange':[251,254]},
		'boundsXZWH':[0,0,24,16],'cellSizeM':[2,2],'grid':[12,8],'patchCount':96,
		'tint':[0.56,0.60,0.58,1],'tileability':'NOT_ESTABLISHED; inspect actual seams'
	},
	'wall': {
		'path':'res://assets/props/return_station/bulkhead_tiles.png','bytes':1673423,
		'sha256':'c5a18cae9a3ddc95c8515f6b342cb7ca3be12c23ff502ab5b3f16000bb4c9a5b',
		'format':'RGBA8','canvas':[1418,1028],'frame':[0,0,355,480],
		'paintedBounds':[41,6,299,457],'groundFeetPx':[[50,322],[300,460]],
		'rowId':'exit_wall_tiles','assetId':'runtime2d.grey_hive.wall_tiles.v1',
		'layer':'visual.props_back','positionM':[12,0,1],
		'texturedRuns':[{'id':'north','collisionId':'exit_wall_north','startM':[0,0,0.25],'endM':[24,0,0.25],'count':24,'cadenceM':1}],
		'maxExtentPxAt48':96,'tint':[0.62,0.68,0.66,1],'rearOnly':true
	},
	'pipe': {
		'path':'res://assets/props/return_station/service_pipes.png','bytes':1049893,
		'sha256':'7f3c67879398e89adc9026fc2aca5fbe7792b1dc360b73c737c1b2299eac96da',
		'canvas':[1414,1047],'format':'RGBA8','assetId':'runtime2d.grey_hive.pipes_cables.v1',
		'rowId':'exit_service_pipes','layer':'visual.props_back','positionM':[9,0,3],
		'selection':{'framePx':[0,219,388,270],'seedPx':[100,320],'componentBoundsPx':[0,219,388,270],'connectedAlphaPixels':46171,'contactsPx':[[50,345],[325,482]]},
		'contactsM':[[8.5,0,3],[9.5,0,3]],'maximumRotationRadians':0.025,'maxExtentPxAt48':80,
		'shadow':{'centerM':[9,0,3],'radiiM':[0.6,0.16],'alpha':0.18}
	},
	'clutter': {
		'path':'res://assets/props/grey_hive/clutter.png','bytes':1857722,
		'sha256':'4743030e58286fdac5c4e119941608763d1ceb1d3bbed79ca1d7361ceab97ebe',
		'canvas':[1428,1061],'format':'RGBA8','assetId':'runtime2d.grey_hive.clutter.v1',
		'rowId':'exit_approved_clutter','layer':'visual.props_back','positionM':[5,0,11.5],
		'selection':{'framePx':[40,716,347,287],'seedPx':[150,800],'seedLocalPx':[110,84],'componentBoundsLocalPx':[0,0,347,287],'connectedAlphaPixels':73359,'disconnectedFrameAlphaPixels':168,'contactsSourcePx':[[50,929],[369,898]],'contactsLocalPx':[[10,213],[329,182]],'localOriginConvention':'frameTopLeft'},
		'contactsM':[[4.625,0,12],[5.375,0,11]],'maximumRotationRadians':0.06,'maxExtentPxAt48':96,
		'shadow':{'centerM':[5,0,11.5],'radiiM':[0.5,0.55],'alpha':0.16}
	},
	'terminal': {
		'rowId':'exit_extraction_console_approved','assetId':'runtime2d.prop.lockdown_terminal.v1',
		'layer':'visual.props_dynamic','positionM':[8,0,8],
		'consumer':'gh_bio_lockdown_visual.draw_body key lockdown_terminal',
		'canvas':[1270,1086],'anchor':[0.5,1],'heightM':1.4,
		'staticDecorativeOnly':true,'interactionOrPowerClaim':false
	},
	'crate': {
		'rowId':'exit_console_crates','assetId':'runtime2d.prop.supply_crates.v1',
		'layer':'visual.props_back','positionM':[18,0,11.5],
		'consumer':'unchanged existing _gh_crate_selected / gh_supply_crate_visual','heightM':1
	},
	'cutaway': {
		'exteriorHeightM':0.35,
		'onlyExteriorIds':['exit_wall_north','exit_wall_south','exit_wall_west_north','exit_wall_west_south','exit_wall_east_north','exit_wall_east_south'],
		'cap':'#64716e','faceX':'#34433d','faceZ':'#3d4d46','contactAlpha':0.1
	},
	'ambient': {'kind':'static floor-bound cool tonal wash','maxPolygons':2,'maxAlpha':0.025,'color':'#77918b','noEmitterOrGameplayState':true},
	'sealedBarrier': {'id':'exit_sealed_extraction_barrier','requiresEvent':'hive_extraction','policy':'EXCLUDED from new wall consumer; unchanged original fallback/geometry/event'},
	'temporal':'NONE; same new geometry/tint for reduced_motion true/false; no clock/fog/particles',
	'registrationMeaning':'New room-specific visual contacts/cutaway/art direction, not source-measured physical dimensions; MAIN future explicit admission required'
}

var _definitions_loaded: bool = false
var _definition: Dictionary = {}
var _canonical_scene: Dictionary = {}
var _assets_attempted: bool = false
var _assets_ready: bool = false
var _floor_texture: ImageTexture
var _wall_texture: ImageTexture
var _pipe_texture: ImageTexture
var _clutter_texture: ImageTexture
var _current_ready: bool = false
var _prepared_key: String = ''

func prepare(snapshot: Dictionary, scene: Dictionary, current_key: String, scene_hash: String) -> bool:
	_current_ready = false
	_prepared_key = ''
	if not _basic_context(snapshot, scene, current_key):
		return false
	if not _definitions_loaded and not _load_definitions():
		return false
	if scene_hash != SCENE_SHA256 or not _deep_equal(scene, _canonical_scene):
		return false
	if not _assets_attempted:
		_assets_attempted = true
		_assets_ready = _load_assets()
	_current_ready = _assets_ready
	_prepared_key = current_key if _current_ready else ''
	return _current_ready

func _basic_context(snapshot: Dictionary, scene: Dictionary, current_key: String) -> bool:
	if not P.snapshot(snapshot) or snapshot.get('entryToken') != null or not P.counter(snapshot.get('worldEpoch'), 1):
		return false
	if snapshot.get('worldId') != 'grey_hive' or snapshot.get('sceneId') != 'gh_exit' or current_key != 'grey_hive:gh_exit:' + str(snapshot.get('worldEpoch')):
		return false
	if scene.get('worldId') != 'grey_hive' or scene.get('sceneId') != 'gh_exit' or not P.counter(scene.get('schemaVersion'), 1, 1):
		return false
	var bounds_variant: Variant = scene.get('boundsM')
	if not bounds_variant is Dictionary:
		return false
	var bounds: Dictionary = bounds_variant
	return _keys_are(bounds, ['x','z','width','depth']) and _numbers_equal([bounds.get('x'),bounds.get('z'),bounds.get('width'),bounds.get('depth')], [0,0,24,16]) and _scene_row_types(scene)

func _scene_row_types(scene: Dictionary) -> bool:
	var collision_variant: Variant = scene.get('collision')
	var presentation_variant: Variant = scene.get('presentation')
	if not collision_variant is Array or not presentation_variant is Dictionary:
		return false
	var collision: Array = collision_variant
	if collision.size() != 7:
		return false
	var collision_ids: Dictionary = {}
	for row_variant in collision:
		if not row_variant is Dictionary:
			return false
		var row: Dictionary = row_variant
		var id_variant: Variant = row.get('id')
		if typeof(id_variant) != TYPE_STRING or collision_ids.has(id_variant):
			return false
		collision_ids[id_variant] = true
		var polygon_variant: Variant = row.get('polygon')
		if not polygon_variant is Array or not P.polygon(polygon_variant):
			return false
	var sprites_variant: Variant = presentation_variant.get('sprites')
	if not sprites_variant is Array:
		return false
	var sprites: Array = sprites_variant
	if sprites.size() != 6:
		return false
	var sprite_ids: Dictionary = {}
	for sprite_variant in sprites:
		if not sprite_variant is Dictionary:
			return false
		var sprite: Dictionary = sprite_variant
		var sprite_id_variant: Variant = sprite.get('id')
		if typeof(sprite_id_variant) != TYPE_STRING or sprite_ids.has(sprite_id_variant):
			return false
		sprite_ids[sprite_id_variant] = true
		var asset_variant: Variant = sprite.get('assetId')
		var layer_variant: Variant = sprite.get('layer')
		var position_variant: Variant = sprite.get('position')
		if typeof(asset_variant) != TYPE_STRING or typeof(layer_variant) != TYPE_STRING:
			return false
		if not _position_is_numeric(position_variant):
			return false
	return true

func _position_is_numeric(value: Variant) -> bool:
	if not value is Array or value.size() != 3:
		return false
	for coordinate in value:
		if not P.number(coordinate, -64.0, 64.0):
			return false
	return true

func _load_definitions() -> bool:
	var scene_bytes: PackedByteArray = _read_file(SCENE_PATH, SCENE_BYTES, SCENE_BYTES)
	if scene_bytes.size() != SCENE_BYTES or _sha256(scene_bytes) != SCENE_SHA256 or not _valid_utf8(scene_bytes):
		return false
	var scene_variant: Variant = JSON.parse_string(scene_bytes.get_string_from_utf8())
	if not scene_variant is Dictionary:
		return false
	var map_bytes: PackedByteArray = _read_file(MAP_PATH, 1, MAX_MAP_BYTES)
	if map_bytes.is_empty() or not _valid_utf8(map_bytes):
		return false
	var map_variant: Variant = JSON.parse_string(map_bytes.get_string_from_utf8())
	if not map_variant is Dictionary or not _deep_equal(map_variant, SEALED_MAP):
		return false
	var scene: Dictionary = scene_variant
	if not _scene_row_types(scene) or scene.get('worldId') != 'grey_hive' or scene.get('sceneId') != 'gh_exit':
		return false
	_canonical_scene = scene
	_definition = map_variant
	_definitions_loaded = true
	return true

func _load_assets() -> bool:
	if not _definitions_loaded:
		return false
	var floor_entry: Dictionary = _definition.get('floor', {})
	var wall_entry: Dictionary = _definition.get('wall', {})
	var pipe_entry: Dictionary = _definition.get('pipe', {})
	var clutter_entry: Dictionary = _definition.get('clutter', {})
	var floor_image: Image = _read_png_image(floor_entry)
	var wall_image: Image = _read_png_image(wall_entry)
	var pipe_image: Image = _read_png_image(pipe_entry)
	var clutter_image: Image = _read_png_image(clutter_entry)
	if floor_image == null or wall_image == null or pipe_image == null or clutter_image == null:
		return false
	var pipe_selection: Dictionary = pipe_entry.get('selection', {})
	var clutter_selection: Dictionary = clutter_entry.get('selection', {})
	var pipe_frame: Rect2i = _rect2i_from_array(pipe_selection.get('framePx'))
	var clutter_frame: Rect2i = _rect2i_from_array(clutter_selection.get('framePx'))
	var pipe_seed: Vector2i = _vector2i_from_array(pipe_selection.get('seedPx'))
	var clutter_seed: Vector2i = _vector2i_from_array(clutter_selection.get('seedPx'))
	if pipe_frame.size != Vector2i(388,270) or clutter_frame.size != Vector2i(347,287) or pipe_seed != Vector2i(100,320) or clutter_seed != Vector2i(150,800):
		return false
	var pipe_cutout: Image = _isolate_component(pipe_image, pipe_frame, pipe_seed, 46171, -1)
	var clutter_cutout: Image = _isolate_component(clutter_image, clutter_frame, clutter_seed, 73359, 168)
	if pipe_cutout == null or clutter_cutout == null:
		return false
	_floor_texture = ImageTexture.create_from_image(floor_image)
	_wall_texture = ImageTexture.create_from_image(wall_image)
	_pipe_texture = ImageTexture.create_from_image(pipe_cutout)
	_clutter_texture = ImageTexture.create_from_image(clutter_cutout)
	return _texture_size(_floor_texture, Vector2i(1434,980)) and _texture_size(_wall_texture, Vector2i(1418,1028)) and _texture_size(_pipe_texture, Vector2i(388,270)) and _texture_size(_clutter_texture, Vector2i(347,287))

func _read_png_image(entry: Dictionary) -> Image:
	var path_variant: Variant = entry.get('path')
	var bytes_variant: Variant = entry.get('bytes')
	var hash_variant: Variant = entry.get('sha256')
	var canvas_variant: Variant = entry.get('canvas')
	if typeof(path_variant) != TYPE_STRING or typeof(hash_variant) != TYPE_STRING or not P.counter(bytes_variant, 1, 10000000) or not _integer_pair(canvas_variant):
		return null
	var canvas_size: Vector2i = _vector2i_from_array(canvas_variant)
	var file_bytes: PackedByteArray = _read_file(path_variant, int(bytes_variant), int(bytes_variant))
	if file_bytes.size() != int(bytes_variant) or _sha256(file_bytes) != hash_variant:
		return null
	var image: Image = Image.new()
	if image.load_png_from_buffer(file_bytes) != OK or image.is_empty() or image.get_format() != Image.FORMAT_RGBA8 or image.get_size() != canvas_size:
		return null
	return image

func _isolate_component(source: Image, frame: Rect2i, seed: Vector2i, expected_pixels: int, expected_disconnected: int) -> Image:
	if source == null or source.is_empty() or frame.size.x <= 0 or frame.size.y <= 0 or frame.position.x < 0 or frame.position.y < 0 or frame.end.x > source.get_width() or frame.end.y > source.get_height():
		return null
	var seed_local: Vector2i = seed - frame.position
	if seed_local.x < 0 or seed_local.y < 0 or seed_local.x >= frame.size.x or seed_local.y >= frame.size.y or source.get_pixel(seed.x,seed.y).a <= 0.0:
		return null
	var total: int = frame.size.x * frame.size.y
	var visited: PackedByteArray = PackedByteArray()
	visited.resize(total)
	var queue: PackedInt32Array = PackedInt32Array()
	queue.resize(total)
	var head: int = 0
	var tail: int = 1
	var seed_index: int = seed_local.y * frame.size.x + seed_local.x
	queue[0] = seed_index
	visited[seed_index] = 1
	var total_alpha: int = 0
	for y in range(frame.size.y):
		for x in range(frame.size.x):
			if source.get_pixel(frame.position.x + x, frame.position.y + y).a > 0.0:
				total_alpha += 1
	while head < tail:
		var current_index: int = queue[head]
		head += 1
		var x: int = current_index % frame.size.x
		var y: int = floori(float(current_index) / float(frame.size.x))
		var neighbors: Array[Vector2i] = [Vector2i(x-1,y),Vector2i(x+1,y),Vector2i(x,y-1),Vector2i(x,y+1)]
		for neighbor in neighbors:
			if neighbor.x < 0 or neighbor.y < 0 or neighbor.x >= frame.size.x or neighbor.y >= frame.size.y:
				continue
			var neighbor_index: int = neighbor.y * frame.size.x + neighbor.x
			if visited[neighbor_index] == 0 and source.get_pixel(frame.position.x + neighbor.x, frame.position.y + neighbor.y).a > 0.0:
				visited[neighbor_index] = 1
				queue[tail] = neighbor_index
				tail += 1
	if tail != expected_pixels or expected_disconnected >= 0 and total_alpha - tail != expected_disconnected:
		return null
	var output: Image = Image.create_empty(frame.size.x,frame.size.y,false,Image.FORMAT_RGBA8)
	if output == null:
		return null
	output.fill(Color(0,0,0,0))
	var minimum: Vector2i = frame.size
	var maximum: Vector2i = Vector2i(-1,-1)
	for i in range(tail):
		var index: int = queue[i]
		var pixel_x: int = index % frame.size.x
		var pixel_y: int = floori(float(index) / float(frame.size.x))
		output.set_pixel(pixel_x,pixel_y,source.get_pixel(frame.position.x + pixel_x,frame.position.y + pixel_y))
		minimum.x = mini(minimum.x,pixel_x)
		minimum.y = mini(minimum.y,pixel_y)
		maximum.x = maxi(maximum.x,pixel_x)
		maximum.y = maxi(maximum.y,pixel_y)
	if minimum != Vector2i.ZERO or maximum != frame.size-Vector2i.ONE:
		return null
	return output

func _texture_size(texture: Texture2D, expected: Vector2i) -> bool:
	return texture != null and texture.get_width() == expected.x and texture.get_height() == expected.y

func _read_file(path: String, minimum: int, maximum: int) -> PackedByteArray:
	var file: FileAccess = FileAccess.open(path, FileAccess.READ)
	if file == null:
		return PackedByteArray()
	var length: int = file.get_length()
	if length < minimum or length > maximum:
		file.close()
		return PackedByteArray()
	var bytes: PackedByteArray = file.get_buffer(length)
	file.close()
	if bytes.size() != length:
		return PackedByteArray()
	return bytes

func _sha256(bytes: PackedByteArray) -> String:
	var context: HashingContext = HashingContext.new()
	if context.start(HashingContext.HASH_SHA256) != OK or context.update(bytes) != OK:
		return ''
	var digest: PackedByteArray = context.finish()
	return digest.hex_encode()

func _valid_utf8(bytes: PackedByteArray) -> bool:
	var text: String = bytes.get_string_from_utf8()
	var encoded: PackedByteArray = text.to_utf8_buffer()
	return _byte_arrays_equal(bytes, encoded)

func _byte_arrays_equal(a: PackedByteArray, b: PackedByteArray) -> bool:
	if a.size() != b.size():
		return false
	for i in range(a.size()):
		if a[i] != b[i]:
			return false
	return true

func _deep_equal(actual: Variant, expected: Variant) -> bool:
	if P.number(actual) and P.number(expected):
		return float(actual) == float(expected)
	if typeof(actual) != typeof(expected):
		return false
	if actual is Dictionary:
		var actual_dictionary: Dictionary = actual
		var expected_dictionary: Dictionary = expected
		if actual_dictionary.size() != expected_dictionary.size():
			return false
		for key in expected_dictionary.keys():
			if not actual_dictionary.has(key) or not _deep_equal(actual_dictionary.get(key), expected_dictionary.get(key)):
				return false
		return true
	if actual is Array:
		var actual_array: Array = actual
		var expected_array: Array = expected
		if actual_array.size() != expected_array.size():
			return false
		for i in range(actual_array.size()):
			if not _deep_equal(actual_array[i], expected_array[i]):
				return false
		return true
	return actual == expected

func _keys_are(value: Dictionary, expected: Array) -> bool:
	if value.size() != expected.size():
		return false
	for key_variant in expected:
		if typeof(key_variant) != TYPE_STRING or not value.has(key_variant):
			return false
	return true

func _numbers_equal(value: Variant, expected: Array) -> bool:
	if not value is Array or value.size() != expected.size():
		return false
	for i in range(expected.size()):
		if not P.number(value[i]) or float(value[i]) != float(expected[i]):
			return false
	return true

func _integer_pair(value: Variant) -> bool:
	return _numbers_equal(value,[1434,980]) or _numbers_equal(value,[1418,1028]) or _numbers_equal(value,[1414,1047]) or _numbers_equal(value,[1428,1061])

func _vector2i_from_array(value: Array) -> Vector2i:
	return Vector2i(int(value[0]),int(value[1]))

func _rect2i_from_array(value: Variant) -> Rect2i:
	if not value is Array or value.size() != 4:
		return Rect2i()
	for part in value:
		if not P.counter(part,0,10000):
			return Rect2i()
	return Rect2i(int(value[0]),int(value[1]),int(value[2]),int(value[3]))

func _numbers_to_vector2(value: Variant) -> Vector2:
	if not value is Array or value.size() != 2 or not P.number(value[0]) or not P.number(value[1]):
		return Vector2(INF,INF)
	return Vector2(float(value[0]),float(value[1]))

func _numbers_to_vector3(value: Variant) -> Vector3:
	if not value is Array or value.size() != 3 or not P.number(value[0]) or not P.number(value[1]) or not P.number(value[2]):
		return Vector3(INF,INF,INF)
	return Vector3(float(value[0]),float(value[1]),float(value[2]))

func draw_floor(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, current_key: String, projection: Variant, viewport: Rect2) -> bool:
	if not _valid_context(snapshot,scene,current_key) or not _valid_canvas(canvas) or not _valid_projection(projection,viewport) or not _texture_size(_floor_texture,Vector2i(1434,980)):
		return false
	var floor_entry: Dictionary = _definition.get('floor',{})
	var selection: Dictionary = floor_entry.get('selection',{})
	var pixel_corners_variant: Variant = selection.get('pixelCenterCorners')
	if not pixel_corners_variant is Array or pixel_corners_variant.size() != 4:
		return false
	var pixel_corners: Array[Vector2] = []
	for corner_variant in pixel_corners_variant:
		var corner: Vector2 = _numbers_to_vector2(corner_variant)
		if not _finite_point(corner):
			return false
		pixel_corners.append(corner)
	var canvas_size: Vector2i = _vector2i_from_array(floor_entry.get('canvas'))
	var uvs: PackedVector2Array = PackedVector2Array()
	for pixel_corner in pixel_corners:
		uvs.append((pixel_corner + Vector2(0.5,0.5)) / Vector2(float(canvas_size.x),float(canvas_size.y)))
	if uvs.size() != 4 or not _finite_points(uvs):
		return false
	var patches: Array[PackedVector2Array] = []
	var patch_bounds: Array[Rect2] = []
	for row in range(8):
		for column in range(12):
			var x0: float = float(column) * 2.0
			var z0: float = float(row) * 2.0
			var world_points: Array[Vector3] = [Vector3(x0,0,z0),Vector3(x0+2.0,0,z0),Vector3(x0+2.0,0,z0+2.0),Vector3(x0,0,z0+2.0)]
			var points: PackedVector2Array = PackedVector2Array()
			for world_point in world_points:
				var screen_point: Vector2 = _project(projection,world_point)
				if not _finite_point(screen_point):
					return false
				points.append(screen_point)
			var bounds: Rect2 = _points_bounds(points)
			if points.size() != 4 or not _valid_rect(bounds):
				return false
			patches.append(points)
			patch_bounds.append(bounds)
	if patches.size() != 96:
		return false
	var wash_points: PackedVector2Array = _ellipse_points(projection,Vector3(12,0,8),Vector2(10,7),32)
	if wash_points.size() != 32 or not _finite_points(wash_points) or not _valid_rect(_points_bounds(wash_points)):
		return false
	var tint: Color = Color(0.56,0.60,0.58,1.0)
	var colors: PackedColorArray = PackedColorArray([tint,tint,tint,tint])
	for i in range(patches.size()):
		if patch_bounds[i].intersects(viewport):
			canvas.draw_polygon(patches[i],colors,uvs,_floor_texture)
	var ambient_color: Color = Color.from_string('#77918b',Color.WHITE)
	ambient_color.a = 0.018
	if _rect_intersects(viewport,_points_bounds(wash_points)):
		canvas.draw_colored_polygon(wash_points,ambient_color)
	return true

func draw_wall(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, current_key: String, wall: Dictionary, projection: Variant, viewport: Rect2) -> bool:
	if not _valid_context(snapshot,scene,current_key) or not _valid_canvas(canvas) or not _valid_projection(projection,viewport):
		return false
	var id_variant: Variant = wall.get('id')
	if typeof(id_variant) != TYPE_STRING or not EXTERIOR_IDS.has(id_variant):
		return false
	var matched_count: int = 0
	var canonical_wall: Dictionary = {}
	for wall_variant in _canonical_scene.get('collision',[]):
		if not wall_variant is Dictionary:
			return false
		var candidate: Dictionary = wall_variant
		if candidate.get('id') == id_variant:
			matched_count += 1
			canonical_wall = candidate
	if matched_count != 1 or not _deep_equal(wall,canonical_wall):
		return false
	var cutaway: Dictionary = _definition.get('cutaway',{})
	var height_variant: Variant = cutaway.get('exteriorHeightM')
	if not P.number(height_variant,0.0,1.0):
		return false
	var height: float = float(height_variant)
	if id_variant == 'exit_wall_north':
		return _draw_north_wall(canvas,projection,viewport,canonical_wall,height)
	if not LOW_WALLS.has(id_variant):
		return false
	var descriptor: Dictionary = LOW_WALLS[id_variant]
	var inner_a: Vector2 = _numbers_to_vector2(descriptor.get('inner_a'))
	var inner_b: Vector2 = _numbers_to_vector2(descriptor.get('inner_b'))
	var outer_a: Vector2 = _numbers_to_vector2(descriptor.get('outer_a'))
	var outer_b: Vector2 = _numbers_to_vector2(descriptor.get('outer_b'))
	if not _finite_point(inner_a) or not _finite_point(inner_b) or not _finite_point(outer_a) or not _finite_point(outer_b):
		return false
	var contact: PackedVector2Array = _project_xz_polygon(projection,descriptor.get('contact'),0.0)
	var cap: PackedVector2Array = _project_xz_polygon(projection,canonical_wall.get('polygon'),height)
	var inside_face: PackedVector2Array = _project_vertical_face(projection,inner_a,inner_b,height)
	var outside_face: PackedVector2Array = _project_vertical_face(projection,outer_a,outer_b,height)
	if contact.size() != 4 or cap.size() != 4 or inside_face.size() != 4 or outside_face.size() != 4 or not _finite_points(contact) or not _finite_points(cap) or not _finite_points(inside_face) or not _finite_points(outside_face):
		return false
	var contact_color: Color = Color(0.025,0.035,0.038,float(cutaway.get('contactAlpha',0.1)))
	var face_key: String = 'faceX' if descriptor.get('axis') == 'x' else 'faceZ'
	var face_color: Color = Color.from_string(str(cutaway.get(face_key,'')),Color('#34433d'))
	var cap_color: Color = Color.from_string(str(cutaway.get('cap','')),Color('#64716e'))
	canvas.draw_colored_polygon(contact,contact_color)
	canvas.draw_colored_polygon(outside_face,face_color)
	canvas.draw_colored_polygon(inside_face,face_color)
	canvas.draw_colored_polygon(cap,cap_color)
	return true

func _draw_north_wall(canvas: CanvasItem, projection: Variant, viewport: Rect2, wall: Dictionary, height: float) -> bool:
	if not _texture_size(_wall_texture,Vector2i(1418,1028)) or wall.get('id') != 'exit_wall_north':
		return false
	var wall_entry: Dictionary = _definition.get('wall',{})
	var run_list_variant: Variant = wall_entry.get('texturedRuns')
	var frame_variant: Variant = wall_entry.get('frame')
	var feet_variant: Variant = wall_entry.get('groundFeetPx')
	var paint_variant: Variant = wall_entry.get('paintedBounds')
	if not run_list_variant is Array or run_list_variant.size() != 1 or not frame_variant is Array or not feet_variant is Array or feet_variant.size() != 2 or not paint_variant is Array:
		return false
	var run_variant: Variant = run_list_variant[0]
	if not run_variant is Dictionary:
		return false
	var run: Dictionary = run_variant
	if run.get('id') != 'north' or run.get('collisionId') != 'exit_wall_north' or not P.counter(run.get('count'),24,24) or not P.number(run.get('cadenceM'),1.0,1.0):
		return false
	var start_variant: Variant = run.get('startM')
	var end_variant: Variant = run.get('endM')
	var start: Vector3 = _numbers_to_vector3(start_variant)
	var end: Vector3 = _numbers_to_vector3(end_variant)
	if not _finite_point3(start) or not _finite_point3(end) or start != Vector3(0,0,0.25) or end != Vector3(24,0,0.25):
		return false
	var source_rect_i: Rect2i = _rect2i_from_array(frame_variant)
	var painted_rect_i: Rect2i = _rect2i_from_array(paint_variant)
	var foot_a: Vector2 = _numbers_to_vector2(feet_variant[0])
	var foot_b: Vector2 = _numbers_to_vector2(feet_variant[1])
	if source_rect_i != Rect2i(0,0,355,480) or painted_rect_i != Rect2i(41,6,299,457) or not _finite_point(foot_a) or not _finite_point(foot_b):
		return false
	var source_axis: Vector2 = foot_b-foot_a
	if source_axis.length() <= 0.0 or not is_finite(source_axis.length()):
		return false
	var modules: Array[Dictionary] = []
	for i in range(24):
		var world_a: Vector3 = start + Vector3(float(i),0,0)
		var world_b: Vector3 = world_a + Vector3(1,0,0)
		var screen_a: Vector2 = _project(projection,world_a)
		var screen_b: Vector2 = _project(projection,world_b)
		var screen_axis: Vector2 = screen_b-screen_a
		if not _finite_point(screen_a) or not _finite_point(screen_b) or not is_finite(screen_axis.length()) or screen_axis.length() <= 0.0:
			return false
		var scale: float = screen_axis.length()/source_axis.length()
		var rotation: float = screen_axis.angle()-source_axis.angle()
		var origin: Vector2 = screen_a-foot_a.rotated(rotation)*scale
		var endpoint: Vector2 = origin+foot_b.rotated(rotation)*scale
		var bounds: Rect2 = _transformed_bounds(origin,rotation,Vector2(scale,scale),Rect2(Vector2(painted_rect_i.position),Vector2(painted_rect_i.size)))
		var max_extent: float = float(wall_entry.get('maxExtentPxAt48',96))*float(projection.get('ppm'))/48.0
		if not _finite_point(origin) or not _finite_point(endpoint) or endpoint.distance_to(screen_b) > 0.01 or not _valid_rect(bounds) or maxf(bounds.size.x,bounds.size.y) > max_extent+0.01:
			return false
		modules.append({'origin':origin,'rotation':rotation,'scale':scale,'bounds':bounds})
	if modules.size() != 24:
		return false
	var base: PackedVector2Array = _project_xz_polygon(projection,wall.get('polygon'),0.0)
	var contact: PackedVector2Array = _project_xz_polygon(projection,[[0,0.5],[24,0.5],[24,0.45],[0,0.45]],0.0)
	if base.size() != 4 or contact.size() != 4 or not _finite_points(base) or not _finite_points(contact):
		return false
	var tint_array: Variant = wall_entry.get('tint')
	if not _numbers_equal(tint_array,[0.62,0.68,0.66,1]):
		return false
	var tint: Color = Color(0.62,0.68,0.66,1.0)
	var frame: Rect2 = Rect2(Vector2.ZERO,Vector2(float(source_rect_i.size.x),float(source_rect_i.size.y)))
	canvas.draw_colored_polygon(base,Color('#34433d'))
	canvas.draw_colored_polygon(contact,Color(0.025,0.035,0.038,0.1))
	for module in modules:
		canvas.draw_set_transform(module.get('origin'),float(module.get('rotation')),Vector2(float(module.get('scale')),float(module.get('scale'))))
		canvas.draw_texture_rect_region(_wall_texture,frame,frame,tint,false,true)
		canvas.draw_set_transform(Vector2.ZERO,0.0,Vector2.ONE)
	return true

func _valid_context(snapshot: Dictionary, scene: Dictionary, current_key: String) -> bool:
	return _current_ready and _prepared_key == current_key and _definitions_loaded and _assets_ready and _basic_context(snapshot,scene,current_key) and _deep_equal(scene,_canonical_scene)

func _valid_canvas(canvas: CanvasItem) -> bool:
	return canvas != null and is_instance_valid(canvas) and canvas.is_inside_tree() and not canvas.is_queued_for_deletion()

func _valid_projection(projection: Variant, viewport: Rect2) -> bool:
	if not projection is RefCounted or not is_instance_valid(projection) or projection.get_script() != ProjectionScript or not _valid_rect(viewport):
		return false
	var initialized_variant: Variant = projection.get('initialized')
	var origin_variant: Variant = projection.get('origin')
	var center_variant: Variant = projection.get('screen_center')
	var size_variant: Variant = projection.get('viewport')
	var ppm_variant: Variant = projection.get('ppm')
	if typeof(initialized_variant) != TYPE_BOOL or not initialized_variant or not origin_variant is Vector3 or not center_variant is Vector2 or not size_variant is Vector2 or not P.number(ppm_variant,0.000001,MAX_PPM):
		return false
	return _finite_point3(origin_variant) and _finite_point(center_variant) and _finite_point(size_variant) and absf(size_variant.x-viewport.size.x) <= 0.01 and absf(size_variant.y-viewport.size.y) <= 0.01

func _project(projection: Variant, world_point: Vector3) -> Vector2:
	var result: Variant = projection.call('project',world_point)
	if not result is Vector2:
		return Vector2(INF,INF)
	return result

func _project_xz_polygon(projection: Variant, polygon: Variant, y: float) -> PackedVector2Array:
	var result: PackedVector2Array = PackedVector2Array()
	if not polygon is Array or polygon.size() < 3:
		return result
	for point_variant in polygon:
		if not point_variant is Array or point_variant.size() != 2 or not P.number(point_variant[0]) or not P.number(point_variant[1]):
			return PackedVector2Array()
		var screen: Vector2 = _project(projection,Vector3(float(point_variant[0]),y,float(point_variant[1])))
		if not _finite_point(screen):
			return PackedVector2Array()
		result.append(screen)
	return result

func _project_vertical_face(projection: Variant, a: Vector2, b: Vector2, height: float) -> PackedVector2Array:
	var result: PackedVector2Array = PackedVector2Array()
	result.append(_project(projection,Vector3(a.x,0,a.y)))
	result.append(_project(projection,Vector3(b.x,0,b.y)))
	result.append(_project(projection,Vector3(b.x,height,b.y)))
	result.append(_project(projection,Vector3(a.x,height,a.y)))
	return result if _finite_points(result) else PackedVector2Array()

func _ellipse_points(projection: Variant, center: Vector3, radii: Vector2, count: int) -> PackedVector2Array:
	var result: PackedVector2Array = PackedVector2Array()
	if count < 3 or not _finite_point3(center) or not _finite_point(radii) or radii.x <= 0.0 or radii.y <= 0.0:
		return result
	for i in range(count):
		var angle: float = TAU*float(i)/float(count)
		var world: Vector3 = Vector3(center.x+cos(angle)*radii.x,center.y,center.z+sin(angle)*radii.y)
		var screen: Vector2 = _project(projection,world)
		if not _finite_point(screen):
			return PackedVector2Array()
		result.append(screen)
	return result

func _points_bounds(points: PackedVector2Array) -> Rect2:
	if points.is_empty() or not _finite_points(points):
		return Rect2()
	var minimum: Vector2 = points[0]
	var maximum: Vector2 = points[0]
	for point in points:
		minimum.x = minf(minimum.x,point.x)
		minimum.y = minf(minimum.y,point.y)
		maximum.x = maxf(maximum.x,point.x)
		maximum.y = maxf(maximum.y,point.y)
	return Rect2(minimum,maximum-minimum)

func _transformed_bounds(origin: Vector2, rotation: float, scale: Vector2, source: Rect2) -> Rect2:
	if not _finite_point(origin) or not _finite_point(scale) or not is_finite(rotation) or not _valid_rect(source):
		return Rect2()
	var corners: Array[Vector2] = [source.position,Vector2(source.end.x,source.position.y),source.end,Vector2(source.position.x,source.end.y)]
	var points: PackedVector2Array = PackedVector2Array()
	for corner in corners:
		points.append(origin+corner.rotated(rotation)*scale)
	return _points_bounds(points)

func _finite_point(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _finite_point3(value: Vector3) -> bool:
	return is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _finite_points(points: PackedVector2Array) -> bool:
	if points.is_empty():
		return false
	for point in points:
		if not _finite_point(point):
			return false
	return true

func _valid_rect(rect: Rect2) -> bool:
	return _finite_point(rect.position) and _finite_point(rect.size) and _finite_point(rect.end) and rect.size.x > 0.0 and rect.size.y > 0.0

func _rect_intersects(a: Rect2, b: Rect2) -> bool:
	return _valid_rect(a) and _valid_rect(b) and a.intersects(b)

func matches_authored_row(row_value: Variant, selection_key: String) -> bool:
	if not row_value is Dictionary or not selection_key in ['pipe','clutter','terminal']:
		return false
	var row: Dictionary = row_value
	var definition_value: Variant = SEALED_MAP.get(selection_key)
	if not definition_value is Dictionary:
		return false
	var definition: Dictionary = definition_value
	return row.get('id') == definition.rowId and row.get('assetId') == definition.assetId and row.get('layer') == definition.layer and _numbers_equal(row.get('position'),definition.positionM)

func matches_terminal_row(row_value: Variant, snapshot: Dictionary, scene: Dictionary, current_key: String, scene_hash: String) -> bool:
	if scene_hash != SCENE_SHA256 or not _valid_context(snapshot,scene,current_key) or not matches_authored_row(row_value,'terminal'):
		return false
	var same_id: int = 0
	for source_value: Variant in _canonical_scene.presentation.sprites:
		if not source_value is Dictionary:
			return false
		if source_value.get('id') == 'exit_extraction_console_approved':
			same_id += 1
			if not _deep_equal(source_value,row_value):
				return false
	return same_id == 1

func build_dressing(snapshot: Dictionary, scene: Dictionary, current_key: String, projection: Variant, viewport: Rect2) -> Array[Dictionary]:
	var packets: Array[Dictionary] = []
	if not _valid_context(snapshot,scene,current_key) or not _valid_projection(projection,viewport):
		return packets
	for selection_key: String in ['pipe','clutter']:
		var matched: int = 0
		for row_value: Variant in _canonical_scene.presentation.sprites:
			if not row_value is Dictionary:
				return []
			if row_value.get('id') == SEALED_MAP[selection_key].rowId:
				matched += 1
				if not matches_authored_row(row_value,selection_key):
					return []
		if matched != 1:
			return []
		var packet: Dictionary = _dressing_packet(selection_key,current_key,projection)
		if packet.is_empty():
			return []
		packets.append(packet)
	return packets

func _dressing_packet(selection_key: String, current_key: String, projection: Variant) -> Dictionary:
	if not selection_key in ['pipe','clutter']:
		return {}
	var definition: Dictionary = _definition[selection_key]
	var texture: Texture2D = _pipe_texture if selection_key == 'pipe' else _clutter_texture
	var size: Vector2i = Vector2i(388,270) if selection_key == 'pipe' else Vector2i(347,287)
	if not _texture_size(texture,size):
		return {}
	var contact_pixels: Array = [[50,126],[325,263]] if selection_key == 'pipe' else [[10,213],[329,182]]
	var contact_a: Vector2 = _numbers_to_vector2(contact_pixels[0])
	var contact_b: Vector2 = _numbers_to_vector2(contact_pixels[1])
	var source_axis: Vector2 = contact_b-contact_a
	var world_a: Vector3 = _numbers_to_vector3(definition.contactsM[0])
	var world_b: Vector3 = _numbers_to_vector3(definition.contactsM[1])
	var screen_a: Vector2 = _project(projection,world_a)
	var screen_b: Vector2 = _project(projection,world_b)
	if not _finite_point(screen_a) or not _finite_point(screen_b) or source_axis.length() <= 0.0:
		return {}
	var screen_axis: Vector2 = screen_b-screen_a
	var factor: float = screen_axis.length()/source_axis.length()
	var rotation: float = screen_axis.angle()-source_axis.angle()
	var origin: Vector2 = screen_a-contact_a.rotated(rotation)*factor
	var endpoint: Vector2 = origin+contact_b.rotated(rotation)*factor
	var scale: Vector2 = Vector2(factor,factor)
	var body_rect: Rect2 = _transformed_bounds(origin,rotation,scale,Rect2(Vector2.ZERO,Vector2(size)))
	var ppm: float = float(projection.get('ppm'))
	if not is_finite(factor) or factor <= 0.0 or not is_finite(rotation) or absf(rotation) > float(definition.maximumRotationRadians) or not _finite_point(origin) or not _finite_point(endpoint) or endpoint.distance_to(screen_b) > 0.01 or not _valid_rect(body_rect) or maxf(body_rect.size.x,body_rect.size.y) > float(definition.maxExtentPxAt48)*ppm/48.0+0.01:
		return {}
	var shadow: Dictionary = definition.shadow
	var shadow_points: PackedVector2Array = _ellipse_points(projection,_numbers_to_vector3(shadow.centerM),_numbers_to_vector2(shadow.radiiM),SHADOW_STEPS)
	var shadow_rect: Rect2 = _points_bounds(shadow_points)
	if shadow_points.size() != SHADOW_STEPS or not _finite_points(shadow_points) or not _valid_rect(shadow_rect):
		return {}
	return {'id':definition.rowId,'kind':'gh_exit_dressing','selection_key':selection_key,'position':_numbers_to_vector3(definition.positionM),'context_key':current_key,'scene_hash':SCENE_SHA256,'camera_origin':projection.get('origin'),'camera_center':projection.get('screen_center'),'camera_ppm':ppm,'texture':texture,'texture_size':size,'origin':origin,'rotation':rotation,'scale':scale,'body_rect':body_rect,'shadow_points':shadow_points,'shadow_rect':shadow_rect,'shadow_alpha':float(shadow.alpha)}

func _current_packet(packet_value: Variant, snapshot: Dictionary, scene: Dictionary, current_key: String, projection: Variant, viewport: Rect2) -> bool:
	if not packet_value is Dictionary or not _valid_context(snapshot,scene,current_key) or not _valid_projection(projection,viewport):
		return false
	var packet: Dictionary = packet_value
	var selection_value: Variant = packet.get('selection_key')
	if typeof(selection_value) != TYPE_STRING or not selection_value in ['pipe','clutter']:
		return false
	var expected: Dictionary = _dressing_packet(selection_value,current_key,projection)
	return not expected.is_empty() and _deep_equal(packet,expected)

func draw_contact_shadow(canvas: CanvasItem, packet_value: Variant, snapshot: Dictionary, scene: Dictionary, current_key: String, projection: Variant, viewport: Rect2) -> bool:
	if not _valid_canvas(canvas) or not _current_packet(packet_value,snapshot,scene,current_key,projection,viewport):
		return false
	var packet: Dictionary = packet_value
	if not _rect_intersects(viewport,packet.shadow_rect):
		return false
	canvas.draw_colored_polygon(packet.shadow_points,Color(0.025,0.035,0.038,packet.shadow_alpha))
	return true

func draw_dressing(canvas: CanvasItem, packet_value: Variant, snapshot: Dictionary, scene: Dictionary, current_key: String, projection: Variant, viewport: Rect2) -> bool:
	if not _valid_canvas(canvas) or not _current_packet(packet_value,snapshot,scene,current_key,projection,viewport):
		return false
	var packet: Dictionary = packet_value
	if not _rect_intersects(viewport,packet.body_rect):
		return false
	canvas.draw_set_transform(packet.origin,packet.rotation,packet.scale)
	var frame: Rect2 = Rect2(Vector2.ZERO,Vector2(packet.texture_size))
	canvas.draw_texture_rect_region(packet.texture,frame,frame,Color.WHITE,false,true)
	canvas.draw_set_transform(Vector2.ZERO,0.0,Vector2.ONE)
	return true

