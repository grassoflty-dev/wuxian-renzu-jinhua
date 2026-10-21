extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const MAP_PATH = 'res://data/mh_warehouse_crates.internal-use.json'
const SCENE_PATH = 'res://data/scenes/mh_tidal_warehouse.json'
const SCENE_BYTES = 8250
const SCENE_SHA256 = 'bcf83727cbf17118a534547e8d50abd65f8c606a342290b66e5a778a0c2d61d9'
const EXPECTED_JSON = '{"schema":"wuxian.mh-warehouse-crates.internal-use/v1","registeredSourceReuse":true,"approval":"INTERNAL_REGISTERED_SOURCE_REUSE","GodotDisplay":"NOT_RUN","publicRelease":"NOT_ESTABLISHED","roomUse":"NORTH_WALL_BEHIND_LID_DISPLAY_ONLY","asset":{"format":"RGBA8","canvas":[1238,1233],"sha256":"0dbad1364c14649b021f20510b9faf970ff317e6202131bd4cb2c32648b34b00","bytes":2067549,"derivedRelative":"assets/derived/asset-pipeline-v1/sprites/png/runtime2d.prop.supply_crates.v1.png","sourceRelative":"batch03/02_runtime2d.prop.supply_crates.v1.png","anchor":[270,550],"heightM":1,"sourceSha256":"e8250132f14a9c621fd819c9b188a7a1d100e20b4da010966b45d987d51eac68","sourceRuntimeStatus":"ADOPT","path":"res://assets/props/grey_hive/supply_crates_sheet.png","sourceAdmission":"release_approved","sourceId":"runtime2d.prop.supply_crates.v1","cutout":[[250,0],[539,20],[539,440],[295,549],[15,420],[15,20]],"region":[0,640,540,550],"assetId":"runtime2d.prop.supply_crates.v1"},"installation":{"worldId":"mist_harbor","sceneId":"mh_tidal_warehouse","requiredWallId":"warehouse_wall_north","requiredDisplayHeightM":0.8,"atomicOrder":"validated_north_wall_then_optional_crates_then_original_contact_side_cap","instances":[{"id":"mh_warehouse_back_storage_01","position":[6,0,0]},{"id":"mh_warehouse_back_storage_02","position":[7.2,0,0]}]}}'
const CUTOUT = [[250,0],[539,20],[539,440],[295,549],[15,420],[15,20]]
const ORIGINS = [Vector3(6,0,0),Vector3(7.2,0,0)]
const WALLS: Dictionary = {
	'warehouse_wall_north':[[0,0],[24,0],[24,0.5],[0,0.5]],
	'warehouse_wall_south':[[0,15.5],[24,15.5],[24,16],[0,16]],
	'warehouse_wall_west_north':[[0,0.5],[0.5,0.5],[0.5,6],[0,6]],
	'warehouse_wall_west_south':[[0,9.5],[0.5,9.5],[0.5,15.5],[0,15.5]],
	'warehouse_wall_east_north':[[23.5,0.5],[24,0.5],[24,6],[23.5,6]],
	'warehouse_wall_east_south':[[23.5,9.5],[24,9.5],[24,15.5],[23.5,15.5]]
}
const ROWS: Dictionary = {
	'approved_warehouse_stone_water_module':{'id':'approved_warehouse_stone_water_module','assetId':'runtime2d.world.mistharbor.water_edge_stone.v1','layer':'visual.floor','position':[12,0,8]},
	'warehouse_inner_fog_bank':{'id':'warehouse_inner_fog_bank','assetId':'runtime2d.world.mistharbor.fog_bank.v1','layer':'visual.props_back','position':[18,0,3]},
	'warehouse_outer_water_edge':{'id':'warehouse_outer_water_edge','assetId':'runtime2d.world.mistharbor.water_edge.v1','layer':'visual.props_back','position':[12,0,14.5]},
	'warehouse_fog_bank_foreground':{'id':'warehouse_fog_bank_foreground','assetId':'runtime2d.world.mistharbor.fog_bank.v1','layer':'visual.foreground','position':[6,0,12]}
}

var _attempted: bool = false
var _texture: Texture2D
var _scene_attempted: bool = false
var _canonical_scene: Dictionary = {}

func _same(actual: Variant, expected: Variant) -> bool:
	var kind: int = typeof(expected)
	if kind == TYPE_INT or kind == TYPE_FLOAT:
		return P.number(actual,float(expected),float(expected))
	if kind == TYPE_DICTIONARY:
		if not actual is Dictionary or actual.size() != expected.size():
			return false
		for key: Variant in expected:
			if not actual.has(key) or not _same(actual[key],expected[key]):
				return false
		return true
	if kind == TYPE_ARRAY:
		if not actual is Array or actual.size() != expected.size():
			return false
		for index: int in range(expected.size()):
			if not _same(actual[index],expected[index]):
				return false
		return true
	return typeof(actual) == kind and actual == expected

func _v2(value: Variant) -> bool:
	return typeof(value) == TYPE_VECTOR2 and is_finite(value.x) and is_finite(value.y)

func _v3(value: Variant) -> bool:
	return typeof(value) == TYPE_VECTOR3 and is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _rect(value: Variant) -> bool:
	if typeof(value) != TYPE_RECT2:
		return false
	var rect: Rect2 = value
	return _v2(rect.position) and _v2(rect.size) and _v2(rect.end) and rect.size.x > 0.0 and rect.size.y > 0.0

func _context(snapshot_value: Variant, scene_value: Variant, key_value: Variant) -> bool:
	if not snapshot_value is Dictionary or not scene_value is Dictionary or typeof(key_value) != TYPE_STRING:
		return false
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	if not P.snapshot(snapshot) or not snapshot.has('entryToken') or snapshot.get('entryToken') != null or snapshot.get('worldId') != 'mist_harbor' or snapshot.get('sceneId') != 'mh_tidal_warehouse' or not P.counter(snapshot.get('worldEpoch'),1):
		return false
	if key_value != 'mist_harbor:mh_tidal_warehouse:'+str(snapshot.worldEpoch) or scene.get('worldId') != 'mist_harbor' or scene.get('sceneId') != 'mh_tidal_warehouse' or not P.counter(scene.get('schemaVersion'),1,1):
		return false
	if not _same(scene.get('boundsM'),{'x':0,'z':0,'width':24,'depth':16}):
		return false
	var collision_value: Variant = scene.get('collision')
	if not collision_value is Array or collision_value.size() != 6:
		return false
	var seen_walls: Dictionary = {}
	for row_value: Variant in collision_value:
		if not row_value is Dictionary:
			return false
		var row: Dictionary = row_value
		var id_value: Variant = row.get('id')
		if typeof(id_value) != TYPE_STRING or not WALLS.has(id_value) or seen_walls.has(id_value) or not _same(row,{'id':id_value,'polygon':WALLS[id_value]}):
			return false
		seen_walls[id_value] = true
	if not _same(scene.get('walkablePolygons'),[{'id':'mh_tidal_warehouse_walkable_area','polygon':[[0.5,0.5],[23.5,0.5],[23.5,15.5],[0.5,15.5]]}]):
		return false
	var presentation_value: Variant = scene.get('presentation')
	if not presentation_value is Dictionary:
		return false
	var presentation: Dictionary = presentation_value
	var sprites_value: Variant = presentation.get('sprites')
	if not sprites_value is Array or sprites_value.size() != 4:
		return false
	var seen_rows: Dictionary = {}
	for row_value: Variant in sprites_value:
		if not row_value is Dictionary:
			return false
		var row: Dictionary = row_value
		var id_value: Variant = row.get('id')
		if typeof(id_value) != TYPE_STRING or not ROWS.has(id_value) or seen_rows.has(id_value) or not _same(row,ROWS[id_value]):
			return false
		seen_rows[id_value] = true
	return seen_walls.size() == 6 and seen_rows.size() == 4

func _projection(camera_value: Variant, viewport_value: Variant) -> bool:
	if not camera_value is RefCounted or not is_instance_valid(camera_value) or camera_value.get_script() != ProjectionScript or not _rect(viewport_value):
		return false
	var camera: RefCounted = camera_value
	var origin_value: Variant = camera.get('origin')
	var center_value: Variant = camera.get('screen_center')
	var size_value: Variant = camera.get('viewport')
	var initialized_value: Variant = camera.get('initialized')
	if not _v3(origin_value) or not _v2(center_value) or not _v2(size_value) or typeof(initialized_value) != TYPE_BOOL or initialized_value != true or not P.number(camera.get('ppm'),0.000001,256.0):
		return false
	var size: Vector2 = size_value
	var viewport: Rect2 = viewport_value
	return size.x > 0.0 and size.y > 0.0 and size == viewport.size

func _canvas(canvas_value: Variant) -> bool:
	return canvas_value is CanvasItem and is_instance_valid(canvas_value) and canvas_value.is_inside_tree() and canvas_value.is_visible_in_tree() and not canvas_value.is_queued_for_deletion()

func _read(path: String, maximum: int, exact_length: int = -1) -> PackedByteArray:
	var file: FileAccess = FileAccess.open(path,FileAccess.READ)
	if file == null:
		return PackedByteArray()
	var length: int = file.get_length()
	if length <= 0 or length > maximum or exact_length >= 0 and length != exact_length:
		file.close()
		return PackedByteArray()
	var bytes: PackedByteArray = file.get_buffer(length)
	file.close()
	return bytes if bytes.size() == length else PackedByteArray()

func _hash(bytes: PackedByteArray) -> String:
	var context: HashingContext = HashingContext.new()
	if context.start(HashingContext.HASH_SHA256) != OK or context.update(bytes) != OK:
		return ''
	return context.finish().hex_encode()

func _load_scene_once() -> void:
	if _scene_attempted:
		return
	_scene_attempted = true
	var bytes: PackedByteArray = _read(SCENE_PATH,SCENE_BYTES,SCENE_BYTES)
	if bytes.size() != SCENE_BYTES or _hash(bytes) != SCENE_SHA256:
		return
	var source_text: String = bytes.get_string_from_utf8()
	if source_text.to_utf8_buffer() != bytes:
		return
	var decoded: Variant = JSON.parse_string(source_text)
	if decoded is Dictionary:
		_canonical_scene = decoded

func _live_context(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not _canvas(canvas_value) or not _context(snapshot_value,scene_value,key_value) or not _projection(camera_value,viewport_value):
		return false
	var canvas: CanvasItem = canvas_value
	var viewport: Rect2 = viewport_value
	if viewport.position != Vector2.ZERO or canvas.get('snapshot') != snapshot_value or canvas.get('scene') != scene_value or canvas.get('current_key') != key_value or canvas.get('scene_hash') != SCENE_SHA256 or canvas.get('camera') != camera_value:
		return false
	var stored_viewport_value: Variant = canvas.get('viewport_rect')
	if not _rect(stored_viewport_value) or stored_viewport_value != viewport:
		return false
	var actual_viewport_value: Variant = canvas.get_viewport_rect()
	if not _rect(actual_viewport_value) or actual_viewport_value != viewport:
		return false
	_load_scene_once()
	return not _canonical_scene.is_empty() and _same(scene_value,_canonical_scene)

func prepare_resources(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, camera: Variant, viewport: Variant) -> void:
	if not _live_context(canvas,snapshot,scene,key,camera,viewport) or _attempted:
		return
	_attempted = true
	var bytes: PackedByteArray = _read(MAP_PATH,8192)
	if bytes.is_empty() or bytes.get_string_from_utf8().to_utf8_buffer() != bytes:
		return
	var expected: Variant = JSON.parse_string(EXPECTED_JSON)
	var actual: Variant = JSON.parse_string(bytes.get_string_from_utf8())
	if not expected is Dictionary or not actual is Dictionary or not _same(actual,expected):
		return
	var png: PackedByteArray = _read('res://assets/props/grey_hive/supply_crates_sheet.png',4194304,2067549)
	if png.size() != 2067549 or _hash(png) != '0dbad1364c14649b021f20510b9faf970ff317e6202131bd4cb2c32648b34b00':
		return
	var image: Image = Image.new()
	if image.load_png_from_buffer(png) != OK or image.get_width() != 1238 or image.get_height() != 1233 or image.get_format() != Image.FORMAT_RGBA8:
		return
	var texture: ImageTexture = ImageTexture.create_from_image(image)
	if texture == null or not is_instance_valid(texture) or texture.get_width() != 1238 or texture.get_height() != 1233:
		return
	_texture = texture

func _valid_body(points: PackedVector2Array, uvs: PackedVector2Array) -> bool:
	if points.size() != 6 or uvs.size() != 6:
		return false
	for index: int in range(6):
		if not _v2(points[index]) or not _v2(uvs[index]) or uvs[index].x < 0.0 or uvs[index].x > 1.0 or uvs[index].y < 0.0 or uvs[index].y > 1.0:
			return false
	var area: float = 0.0
	for index: int in range(1,5):
		area += (points[index]-points[0]).cross(points[index+1]-points[0])
	if not is_finite(area) or area == 0.0:
		return false
	for index: int in range(6):
		var edge: Vector2 = points[(index+1)%6]-points[index]
		var next_edge: Vector2 = points[(index+2)%6]-points[(index+1)%6]
		var turn: float = edge.cross(next_edge)
		if not _v2(edge) or not _v2(next_edge) or edge == Vector2.ZERO or next_edge == Vector2.ZERO or not is_finite(turn) or turn == 0.0 or (turn > 0.0) != (area > 0.0):
			return false
	var bounds: Rect2 = Rect2(points[0],Vector2.ZERO)
	for point: Vector2 in points:
		bounds = bounds.expand(point)
	return _rect(bounds)

func draw_north_backdrop(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, key: Variant, camera: Variant, viewport: Rect2) -> bool:
	if not _live_context(canvas,snapshot,scene,key,camera,viewport):
		return false
	prepare_resources(canvas,snapshot,scene,key,camera,viewport)
	if _texture == null or not is_instance_valid(_texture) or _texture.get_width() != 1238 or _texture.get_height() != 1233:
		return false
	var factor: float = float(camera.get('ppm'))/550.0
	if not is_finite(factor) or factor <= 0.0:
		return false
	var commands: Array[Dictionary] = []
	var visible: bool = false
	for origin: Vector3 in ORIGINS:
		var foot_value: Variant = camera.call('project',origin)
		if not _v2(foot_value):
			return false
		var foot: Vector2 = foot_value
		var points: PackedVector2Array = PackedVector2Array()
		var uvs: PackedVector2Array = PackedVector2Array()
		for pair: Array in CUTOUT:
			var q: Vector2 = Vector2(float(pair[0]),float(pair[1]))
			var point: Vector2 = foot+(q-Vector2(270,550))*factor
			var uv: Vector2 = (q+Vector2(0,640))/Vector2(1238,1233)
			points.append(point)
			uvs.append(uv)
		if not _valid_body(points,uvs):
			return false
		var bounds: Rect2 = Rect2(points[0],Vector2.ZERO)
		for point: Vector2 in points:
			bounds = bounds.expand(point)
		if not _rect(bounds):
			return false
		visible = visible or (bounds.position.x < viewport.end.x and viewport.position.x < bounds.end.x and bounds.position.y < viewport.end.y and viewport.position.y < bounds.end.y)
		commands.append({'points':points,'uvs':uvs})
	if commands.size() != 2 or not visible or not _live_context(canvas,snapshot,scene,key,camera,viewport) or _texture == null or not is_instance_valid(_texture):
		return false
	for command: Dictionary in commands:
		canvas.draw_colored_polygon(command.points,Color.WHITE,command.uvs,_texture)
	return true
