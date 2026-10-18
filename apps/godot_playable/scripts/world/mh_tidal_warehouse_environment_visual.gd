extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const MAP_PATH: String = 'res://data/mh_tidal_warehouse_environment.internal-use.json'
const EXPECTED_JSON: String = '{"schema":"wuxian.mh-tidal-warehouse-environment.internal-use/v1","registeredSourceReuse":true,"approval":"INTERNAL_REGISTERED_SOURCE_REUSE","roomUse":"ONLY_EXPLICIT_MAIN_ADMITTED_MH_TIDAL_WAREHOUSE_SCOPE","GodotDisplay":"NOT_RUN","publicRelease":"NOT_ESTABLISHED","sourceContracts":[{"path":"res://data/mh_fog_pier_stone_material.internal-use.json","bytes":1517,"sha256":"c56ad7549a49809341e2aea6efcbe7fbe0477eb51026e776c4a49fa03077585a"},{"path":"res://data/mh_fog_pier_layered_fog.internal-use.json","bytes":2270,"sha256":"027980b44e5f92aa5e7759611eeffe69d6ae59be90a4ae4e0858a96123d2bee0"}],"assets":{"stone":{"assetId":"runtime2d.world.mistharbor.water_edge_stone.v1","derivedRelative":"assets/derived/asset-pipeline-v1/sprites/png/runtime2d.world.mistharbor.water_edge_stone.v1.png","sourceRelative":"batch07/02_runtime2d.world.mistharbor.water_edge_stone.v1.png","sourceSha256":"dc09452bf3f55c57cca1a93bb09f169ef441af90321dff47d91c4be7e62d3a83","path":"res://assets/props/mist_harbor/fog_pier_stone_source.png","bytes":1983603,"sha256":"e823434b3fbc420fc14270200f81e0a459e594597304b1323d336e532a924f9d","canvas":[1228,1254],"format":"RGBA8"},"fog":{"assetId":"runtime2d.world.mistharbor.fog_bank.v1","path":"res://assets/props/mist_harbor/fog_pier_fog_bank_source.png","bytes":684960,"sha256":"9bbbfd9783f70a8b1f6ffeaabd44e568074f4c20f1f7500d7ef4f0e534edeac2","canvas":[1254,1236],"format":"RGBA8"}},"installation":{"worldId":"mist_harbor","sceneId":"mh_tidal_warehouse","boundsM":[0,0,24,16],"stoneRowId":"approved_warehouse_stone_water_module","stoneLayer":"visual.floor","stonePosition":[12,0,8],"walkableId":"mh_tidal_warehouse_walkable_area","walkablePolygon":[[0.5,0.5],[23.5,0.5],[23.5,15.5],[0.5,15.5]],"groundY":0},"stone":{"sourcePixelEdgeCorners":[[820,240],[1032,342],[392,627],[180,525]],"uvConvention":"sourcePixelEdge / standaloneCanvasSize","patchSizeM":[1.5,4.5],"grid":[16,4],"patchCount":64,"edgeCells":"CLIP_WORLD_AND_UV_FRACTIONS_NOT_STRETCH","tint":[0.48,0.58,0.63,1]},"fog":{"sourceRect":[27,370,1201,498],"sampling":"linear","repeat":false,"blend":"normal_alpha_preserve_RGBA","static":true,"rows":[{"id":"warehouse_inner_fog_bank","assetId":"runtime2d.world.mistharbor.fog_bank.v1","layer":"visual.props_back","position":[18,0,3],"phase":"back","widthPerPpm":9,"centerOffsetPerPpm":[0,-1.25],"modulate":[0.8,0.88,0.92,0.14]},{"id":"warehouse_fog_bank_foreground","assetId":"runtime2d.world.mistharbor.fog_bank.v1","layer":"visual.foreground","position":[6,0,12],"phase":"front","widthPerPpm":6,"centerOffsetPerPpm":[0,0.05],"modulate":[0.8,0.88,0.92,0.045]}]},"walls":{"highIds":["warehouse_wall_north","warehouse_wall_west_north","warehouse_wall_west_south"],"highDisplayHeight":0.8,"lowDisplayHeight":0.18,"sideColor":"#273b40","capColor":"#506d75","contactAlpha":0.14,"contactWidthM":0.06}}'
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
const BANKS: Dictionary = {
 'warehouse_inner_fog_bank':{'position':Vector3(18,0,3),'width':9.0,'offset':Vector2(0,-1.25),'color':Color(0.8,0.88,0.92,0.14)},
 'warehouse_fog_bank_foreground':{'position':Vector3(6,0,12),'width':6.0,'offset':Vector2(0,0.05),'color':Color(0.8,0.88,0.92,0.045)}
}
var attempted: bool = false
var definition_valid: bool = false
var textures: Dictionary = {}

func _same(actual, expected) -> bool:
	var kind: int = typeof(expected)
	if kind == TYPE_INT or kind == TYPE_FLOAT:
		return P.number(actual,float(expected),float(expected))
	if kind == TYPE_DICTIONARY:
		if not actual is Dictionary or actual.size() != expected.size():
			return false
		for key in expected:
			if not actual.has(key) or not _same(actual[key],expected[key]):
				return false
		return true
	if kind == TYPE_ARRAY:
		if not actual is Array or actual.size() != expected.size():
			return false
		for index in range(expected.size()):
			if not _same(actual[index],expected[index]):
				return false
		return true
	return typeof(actual) == kind and actual == expected

func _v2(value) -> bool:
	return typeof(value) == TYPE_VECTOR2 and is_finite(value.x) and is_finite(value.y)

func _v3(value) -> bool:
	return typeof(value) == TYPE_VECTOR3 and is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _rect(value) -> bool:
	if typeof(value) != TYPE_RECT2:
		return false
	var rect: Rect2 = value
	return _v2(rect.position) and _v2(rect.size) and _v2(rect.end) and rect.size.x > 0.0 and rect.size.y > 0.0

func _context(snapshot_value, scene_value, key_value) -> bool:
	if not snapshot_value is Dictionary or not scene_value is Dictionary or typeof(key_value) != TYPE_STRING:
		return false
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	if not P.snapshot(snapshot) or snapshot.get('entryToken') != null or snapshot.get('worldId') != 'mist_harbor' or snapshot.get('sceneId') != 'mh_tidal_warehouse' or not P.counter(snapshot.get('worldEpoch'),1):
		return false
	if key_value != 'mist_harbor:mh_tidal_warehouse:'+str(snapshot.worldEpoch) or scene.get('worldId') != 'mist_harbor' or scene.get('sceneId') != 'mh_tidal_warehouse' or not P.counter(scene.get('schemaVersion'),1,1):
		return false
	if not _same(scene.get('boundsM'),{'x':0,'z':0,'width':24,'depth':16}):
		return false
	var collision_value: Variant = scene.get('collision')
	if not collision_value is Array or collision_value.size() != 6:
		return false
	var seen_walls: Dictionary = {}
	for row_value in collision_value:
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
	for row_value in sprites_value:
		if not row_value is Dictionary:
			return false
		var row: Dictionary = row_value
		var id_value: Variant = row.get('id')
		if typeof(id_value) != TYPE_STRING or not ROWS.has(id_value) or seen_rows.has(id_value) or not _same(row,ROWS[id_value]):
			return false
		seen_rows[id_value] = true
	return seen_walls.size() == 6 and seen_rows.size() == 4

func _projection(camera_value, viewport_value) -> bool:
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

func _canvas(canvas_value) -> bool:
	return canvas_value is CanvasItem and is_instance_valid(canvas_value) and canvas_value.is_inside_tree() and not canvas_value.is_queued_for_deletion()

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

func prepare_resources(snapshot, scene, current_key) -> void:
	if not _context(snapshot,scene,current_key) or attempted:
		return
	attempted = true
	var map_bytes: PackedByteArray = _read(MAP_PATH,16384)
	if map_bytes.is_empty() or map_bytes.get_string_from_utf8().to_utf8_buffer() != map_bytes:
		return
	var expected_value: Variant = JSON.parse_string(EXPECTED_JSON)
	var mapping_value: Variant = JSON.parse_string(map_bytes.get_string_from_utf8())
	if not expected_value is Dictionary or not mapping_value is Dictionary or not _same(mapping_value,expected_value):
		return
	var expected: Dictionary = expected_value
	var contracts: Array = expected.sourceContracts
	for contract_value in contracts:
		if not contract_value is Dictionary:
			return
		var contract: Dictionary = contract_value
		var bytes: PackedByteArray = _read(contract.path,16384,int(contract.bytes))
		if bytes.is_empty() or _hash(bytes) != contract.sha256:
			return
	definition_valid = true
	var assets: Dictionary = expected.assets
	for key_value in ['stone','fog']:
		var key: String = str(key_value)
		var asset: Dictionary = assets[key]
		var bytes: PackedByteArray = _read(asset.path,4194304,int(asset.bytes))
		if bytes.is_empty() or _hash(bytes) != asset.sha256:
			continue
		var image: Image = Image.new()
		if image.load_png_from_buffer(bytes) != OK or image.get_width() != int(asset.canvas[0]) or image.get_height() != int(asset.canvas[1]) or image.get_format() != Image.FORMAT_RGBA8:
			continue
		var texture: ImageTexture = ImageTexture.create_from_image(image)
		if texture != null:
			textures[key] = texture

func _texture_ready(key: String) -> bool:
	if not textures.has(key):
		return false
	var value: Variant = textures[key]
	if not value is Texture2D or not is_instance_valid(value):
		return false
	var texture: Texture2D = value
	return texture.get_width() == (1228 if key == 'stone' else 1254) and texture.get_height() == (1254 if key == 'stone' else 1236)

func _screen_polygon(world_points: Array, camera: RefCounted) -> PackedVector2Array:
	if world_points.size() < 3 or world_points.size() > 4:
		return PackedVector2Array()
	var points: PackedVector2Array = PackedVector2Array()
	for world_value in world_points:
		if not _v3(world_value):
			return PackedVector2Array()
		var world: Vector3 = world_value
		var screen_value: Variant = camera.call('project',world)
		if not _v2(screen_value):
			return PackedVector2Array()
		var screen: Vector2 = screen_value
		points.append(screen)
	var twice_area: float = 0.0
	for index in range(points.size()):
		var next: int = (index+1)%points.size()
		twice_area += points[index].x*points[next].y-points[next].x*points[index].y
	return points if is_finite(twice_area) and absf(twice_area) > 0.000001 else PackedVector2Array()

func _uv(u: float, v: float) -> Vector2:
	return Vector2(820.0+u*212.0-v*640.0,240.0+u*102.0+v*285.0)/Vector2(1228,1254)

func _wall_plan(id: String, camera: RefCounted) -> Dictionary:
	var polygon: Array = WALLS[id]
	var height: float = 0.8 if id in ['warehouse_wall_north','warehouse_wall_west_north','warehouse_wall_west_south'] else 0.18
	var cap_world: Array = []
	for point_value in polygon:
		var point: Array = point_value
		cap_world.append(Vector3(float(point[0]),height,float(point[1])))
	var a: Vector3
	var b: Vector3
	var contact_world: Array = []
	if id == 'warehouse_wall_north':
		a = Vector3(0,0,0.5); b = Vector3(24,0,0.5)
		contact_world = [Vector3(0,0,0.44),Vector3(24,0,0.44),Vector3(24,0,0.5),Vector3(0,0,0.5)]
	elif id == 'warehouse_wall_south':
		a = Vector3(0,0,15.5); b = Vector3(24,0,15.5)
		contact_world = [Vector3(0,0,15.5),Vector3(24,0,15.5),Vector3(24,0,15.56),Vector3(0,0,15.56)]
	else:
		var west: bool = id.begins_with('warehouse_wall_west')
		var x: float = 0.5 if west else 23.5
		var z0: float = float(polygon[0][1])
		var z1: float = float(polygon[2][1])
		a = Vector3(x,0,z0); b = Vector3(x,0,z1)
		var c0: float = x-0.06 if west else x
		var c1: float = x if west else x+0.06
		contact_world = [Vector3(c0,0,z0),Vector3(c1,0,z0),Vector3(c1,0,z1),Vector3(c0,0,z1)]
	var side: PackedVector2Array = _screen_polygon([a,b,Vector3(b.x,height,b.z),Vector3(a.x,height,a.z)],camera)
	var cap: PackedVector2Array = _screen_polygon(cap_world,camera)
	var contact: PackedVector2Array = _screen_polygon(contact_world,camera)
	if side.size() != 4 or cap.size() != 4 or contact.size() != 4:
		return {}
	return {'side':side,'cap':cap,'contact':contact}

func build_frame(snapshot_value, scene_value, key_value, camera_value, viewport_value) -> Dictionary:
	if not definition_valid or not _context(snapshot_value,scene_value,key_value) or not _projection(camera_value,viewport_value):
		return {}
	var snapshot: Dictionary = snapshot_value
	var camera: RefCounted = camera_value
	var viewport: Rect2 = viewport_value
	var ppm: float = float(camera.get('ppm'))
	var floor_patches: Array[Dictionary] = []
	for row in range(4):
		var z0: float = 0.5+float(row)*4.5
		var z1: float = minf(z0+4.5,15.5)
		for column in range(16):
			var x0: float = 0.5+float(column)*1.5
			var x1: float = minf(x0+1.5,23.5)
			var u: float = (x1-x0)/1.5
			var v: float = (z1-z0)/4.5
			var points: PackedVector2Array = _screen_polygon([Vector3(x0,0,z0),Vector3(x1,0,z0),Vector3(x1,0,z1),Vector3(x0,0,z1)],camera)
			var uvs: PackedVector2Array = PackedVector2Array([_uv(0,0),_uv(u,0),_uv(u,v),_uv(0,v)])
			if points.size() != 4:
				return {}
			for uv: Vector2 in uvs:
				if not _v2(uv) or uv.x < 0.0 or uv.x > 1.0 or uv.y < 0.0 or uv.y > 1.0:
					return {}
			floor_patches.append({'points':points,'uvs':uvs})
	var walls: Dictionary = {}
	for id_value in WALLS:
		var id: String = str(id_value)
		var wall: Dictionary = _wall_plan(id,camera)
		if wall.is_empty():
			return {}
		walls[id] = wall
	var banks: Dictionary = {}
	for id_value in BANKS:
		var id: String = str(id_value)
		var spec: Dictionary = BANKS[id]
		var position: Vector3 = spec.position
		var offset: Vector2 = spec.offset
		var foot_value: Variant = camera.call('project',position)
		if not _v2(foot_value):
			return {}
		var foot: Vector2 = foot_value
		var width: float = float(spec.width)*ppm
		var size: Vector2 = Vector2(width,width*498.0/1201.0)
		var rect: Rect2 = Rect2(foot+offset*ppm-size*0.5,size)
		if not _rect(rect):
			return {}
		banks[id] = {'rect':rect,'color':spec.color}
	return {'kind':'mh_tidal_warehouse_frame_v1','worldId':snapshot.worldId,'sceneId':snapshot.sceneId,'worldEpoch':snapshot.worldEpoch,'currentKey':key_value,'origin':camera.get('origin'),'ppm':ppm,'center':camera.get('screen_center'),'cameraViewport':camera.get('viewport'),'viewport':viewport,'stoneReady':_texture_ready('stone'),'fogReady':_texture_ready('fog'),'floor':floor_patches,'walls':walls,'banks':banks}

func _frame_valid(frame_value, snapshot, scene, key, camera, viewport) -> bool:
	if not frame_value is Dictionary:
		return false
	var canonical: Dictionary = build_frame(snapshot,scene,key,camera,viewport)
	return not canonical.is_empty() and _same(frame_value,canonical)

func draw_floor(canvas_value, frame_value, snapshot, scene, key, camera, viewport) -> bool:
	if not _canvas(canvas_value) or not _frame_valid(frame_value,snapshot,scene,key,camera,viewport):
		return false
	var frame: Dictionary = frame_value
	if frame.stoneReady != true:
		return false
	var canvas: CanvasItem = canvas_value
	var texture: Texture2D = textures['stone']
	var tint: Color = Color(0.48,0.58,0.63,1)
	var colors: PackedColorArray = PackedColorArray([tint,tint,tint,tint])
	for patch_value in frame.floor:
		var patch: Dictionary = patch_value
		var points: PackedVector2Array = patch.points
		var uvs: PackedVector2Array = patch.uvs
		canvas.draw_polygon(points,colors,uvs,texture)
	return true

func draw_wall(canvas_value, frame_value, wall_value, snapshot, scene, key, camera, viewport) -> bool:
	if not _canvas(canvas_value) or not wall_value is Dictionary or not _frame_valid(frame_value,snapshot,scene,key,camera,viewport):
		return false
	var wall: Dictionary = wall_value
	var id_value: Variant = wall.get('id')
	if typeof(id_value) != TYPE_STRING or not WALLS.has(id_value) or not _same(wall,{'id':id_value,'polygon':WALLS[id_value]}):
		return false
	var frame: Dictionary = frame_value
	var plan: Dictionary = frame.walls[id_value]
	var canvas: CanvasItem = canvas_value
	var contact: PackedVector2Array = plan.contact
	var side: PackedVector2Array = plan.side
	var cap: PackedVector2Array = plan.cap
	canvas.draw_colored_polygon(contact,Color(0.01,0.025,0.03,0.14))
	canvas.draw_colored_polygon(side,Color('#273b40'))
	canvas.draw_colored_polygon(cap,Color('#506d75'))
	return true

func draw_bank(canvas_value, frame_value, bank_id_value, snapshot, scene, key, camera, viewport) -> bool:
	if not _canvas(canvas_value) or typeof(bank_id_value) != TYPE_STRING or not BANKS.has(bank_id_value) or not _frame_valid(frame_value,snapshot,scene,key,camera,viewport):
		return false
	var frame: Dictionary = frame_value
	if frame.fogReady != true:
		return false
	var bank: Dictionary = frame.banks[bank_id_value]
	var rect: Rect2 = bank.rect
	var color: Color = bank.color
	var texture: Texture2D = textures['fog']
	var canvas: CanvasItem = canvas_value
	canvas.draw_texture_rect_region(texture,rect,Rect2(27,370,1201,498),color,false,true)
	return true

func legacy_bank_consumed(prop_value, snapshot, scene, key, camera, viewport, consumed_value) -> bool:
	if not definition_valid or not _context(snapshot,scene,key) or not _projection(camera,viewport) or not _texture_ready('fog') or not prop_value is Dictionary or not consumed_value is Array or consumed_value.size() > 2:
		return false
	var prop: Dictionary = prop_value
	var id_value: Variant = prop.get('id')
	if typeof(id_value) != TYPE_STRING or not BANKS.has(id_value) or not _same(prop,ROWS[id_value]):
		return false
	var seen: Dictionary = {}
	for value in consumed_value:
		if typeof(value) != TYPE_STRING or not BANKS.has(value) or seen.has(value):
			return false
		seen[value] = true
	var frame: Dictionary = build_frame(snapshot,scene,key,camera,viewport)
	return not frame.is_empty() and frame.fogReady == true and seen.has(id_value)

