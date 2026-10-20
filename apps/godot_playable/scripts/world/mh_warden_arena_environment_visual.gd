extends RefCounted

const EventViewScript = preload('res://scripts/world/event_view.gd')
const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const MAP_PATH: String = 'res://data/mh_warden_arena_environment.internal-use.json'
const SCENE_SHA: String = 'bb0ddca043c57ef4d0d1b327ace455e7f5f8c870845c113064144f64006df86b'

const WALLS: Dictionary = {
	'warden_wall_north': [[0,0],[24,0],[24,0.5],[0,0.5]],
	'warden_wall_south': [[0,15.5],[24,15.5],[24,16],[0,16]],
	'warden_wall_west_north': [[0,0.5],[0.5,0.5],[0.5,6],[0,6]],
	'warden_wall_west_south': [[0,9.5],[0.5,9.5],[0.5,15.5],[0,15.5]],
	'warden_wall_east_north': [[23.5,0.5],[24,0.5],[24,6],[23.5,6]],
	'warden_wall_east_south': [[23.5,9.5],[24,9.5],[24,15.5],[23.5,15.5]]
}
const WALL_ROWS: Array = [
	{'id':'warden_wall_north','polygon':[[0,0],[24,0],[24,0.5],[0,0.5]]},
	{'id':'warden_wall_south','polygon':[[0,15.5],[24,15.5],[24,16],[0,16]]},
	{'id':'warden_wall_west_north','polygon':[[0,0.5],[0.5,0.5],[0.5,6],[0,6]]},
	{'id':'warden_wall_west_south','polygon':[[0,9.5],[0.5,9.5],[0.5,15.5],[0,15.5]]},
	{'id':'warden_wall_east_north','polygon':[[23.5,0.5],[24,0.5],[24,6],[23.5,6]]},
	{'id':'warden_wall_east_south','polygon':[[23.5,9.5],[24,9.5],[24,15.5],[23.5,15.5]]}
]
const SPRITE_ROWS: Array = [
	{'id':'approved_warden_arena_stone_water_module','assetId':'runtime2d.world.mistharbor.water_edge_stone.v1','layer':'visual.floor','position':[12,0,8]},
	{'id':'warden_arena_fog_bank','assetId':'runtime2d.world.mistharbor.fog_bank.v1','layer':'visual.props_back','position':[18,0,3]},
	{'id':'warden_arena_water_edge','assetId':'runtime2d.world.mistharbor.water_edge.v1','layer':'visual.props_back','position':[12,0,14.5]}
]
const MARKER_ROWS: Array = [
	{'id':'warden_arena_ambient_fog_01','assetId':'runtime2d.world.mistharbor.fog_vfx.v1','position':[8,0,4]},
	{'id':'warden_arena_ambient_fog_02','assetId':'runtime2d.world.mistharbor.fog_vfx.v1','position':[18,0,11]}
]
const EXPECTED_MAP: Dictionary = {
	'schema':'wuxian.mh-warden-arena-environment.internal-use/v1',
	'registeredSourceReuse':true,
	'approval':'INTERNAL_REGISTERED_SOURCE_REUSE',
	'runtimeAdmission':'MAIN_ADMITTED_INTERNAL_ROOM_USE',
	'GodotDisplay':'NOT_RUN',
	'publicRelease':'NOT_ESTABLISHED',
	'roomUse':'ONLY_MAIN_ADMITTED_MH_WARDEN_ARENA_ENVIRONMENT_SCOPE',
	'sourceContracts':[
		{'path':'res://data/mh_fog_pier_stone_material.internal-use.json','bytes':1517,'sha256':'c56ad7549a49809341e2aea6efcbe7fbe0477eb51026e776c4a49fa03077585a'},
		{'path':'res://data/mh_fog_pier_layered_fog.internal-use.json','bytes':2270,'sha256':'027980b44e5f92aa5e7759611eeffe69d6ae59be90a4ae4e0858a96123d2bee0'}
	],
	'scene':{
		'path':'res://data/scenes/mh_warden_arena.json','bytes':6991,'sha256':SCENE_SHA,
		'worldId':'mist_harbor','sceneId':'mh_warden_arena','schemaVersion':1,
		'boundsM':{'x':0,'z':0,'width':24,'depth':16},
		'walkablePolygons':[{'id':'mh_warden_arena_walkable_area','polygon':[[0.5,0.5],[23.5,0.5],[23.5,15.5],[0.5,15.5]]}],
		'collision':WALL_ROWS,'sprites':SPRITE_ROWS,'vfxMarkers':MARKER_ROWS
	},
	'assets':{
		'stone':{'assetId':'runtime2d.world.mistharbor.water_edge_stone.v1','path':'res://assets/props/mist_harbor/fog_pier_stone_source.png','bytes':1983603,'sha256':'e823434b3fbc420fc14270200f81e0a459e594597304b1323d336e532a924f9d','canvas':[1228,1254],'format':'RGBA8','sourcePixelEdgeCorners':[[820,240],[1032,342],[392,627],[180,525]],'uvConvention':'sourcePixelEdge / standaloneCanvasSize'},
		'fog':{'assetId':'runtime2d.world.mistharbor.fog_bank.v1','path':'res://assets/props/mist_harbor/fog_pier_fog_bank_source.png','bytes':684960,'sha256':'9bbbfd9783f70a8b1f6ffeaabd44e568074f4c20f1f7500d7ef4f0e534edeac2','canvas':[1254,1236],'format':'RGBA8','sourceRect':[27,370,1201,498],'sampling':'linear','repeat':false,'blend':'normal_alpha_preserve_RGBA'}
	},
	'floor':{'walkableId':'mh_warden_arena_walkable_area','groundY':0,'patchSizeM':[3,3],'columnWidthsM':[3,3,3,3,3,3,3,2],'rowDepthsM':[3,3,3,3,3],'grid':[8,5],'patchCount':40,'edgeCells':'CLIP_WORLD_AND_UV_FRACTIONS_NOT_STRETCH','tint':[0.34,0.41,0.44,1],'sampling':'linear','repeat':false},
	'walls':{'highIds':['warden_wall_north','warden_wall_west_north','warden_wall_west_south'],'highDisplayHeight':0.5,'lowDisplayHeight':0.18,'sideColor':'#293f47','capColor':'#485f67','contactColor':[0.025,0.032,0.035,0.1],'contactWidthM':0.06},
	'bank':{'id':'warden_arena_fog_bank','assetId':'runtime2d.world.mistharbor.fog_bank.v1','layer':'visual.props_back','position':[18,0,3],'sourceRect':[27,370,1201,498],'widthPerPpm':9,'centerOffsetPerPpm':[0,0],'modulate':[0.8,0.88,0.92,0.38],'static':true,'sampling':'linear','repeat':false,'blend':'normal_alpha_preserve_RGBA'},
	'ambient':{'markerAssetId':'runtime2d.world.mistharbor.fog_vfx.v1','centers':[[8,0,4],[18,0,11]],'xOffsets':[-0.2,0,0.2],'heights':[0.16,0.28,0.2],'outerProfile':[[-0.12,0,0],[-0.1,0.05,-0.03],[-0.06,0.7,-0.06],[0,1,-0.08],[0.05,0.78,-0.06],[0.1,0.08,-0.02],[0.12,0,0],[0,0.025,0.1]],'innerScaleXZ':0.65,'innerScaleY':0.75,'color':'#90a9b2','outerAlpha':0.055,'innerAlpha':0.025,'polygonCount':12,'maxWorldY':0.3,'static':false,'staticWithoutImpact':true,'pulseResponse':{'kind':'ResonanceWardenPulseImpact','radiusM':3.5,'durationMs':600,'maxOffsetM':0.16,'maxFadeFraction':0.25,'curve':'sin_squared_pi_age_fraction','selection':'FULL_WISP_ENVELOPE_WITH_OFFSET_MARGIN','reducedMotion':'STATIC_BASELINE'}}
}

var attempted: bool = false
var definition_valid: bool = false
var textures: Dictionary = {}

func _same(actual: Variant, expected: Variant) -> bool:
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

func _finite_number(value: Variant) -> bool:
	return (typeof(value) == TYPE_INT or typeof(value) == TYPE_FLOAT) and is_finite(float(value))

func _finite_v2(value: Variant) -> bool:
	return typeof(value) == TYPE_VECTOR2 and is_finite(value.x) and is_finite(value.y)

func _finite_v3(value: Variant) -> bool:
	return typeof(value) == TYPE_VECTOR3 and is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _valid_rect(value: Variant) -> bool:
	if typeof(value) != TYPE_RECT2:
		return false
	var rect: Rect2 = value
	return _finite_v2(rect.position) and _finite_v2(rect.size) and _finite_v2(rect.end) and rect.size.x > 0.0 and rect.size.y > 0.0

func _sha256(bytes: PackedByteArray) -> String:
	var context: HashingContext = HashingContext.new()
	if context.start(HashingContext.HASH_SHA256) != OK or context.update(bytes) != OK:
		return ''
	var digest: PackedByteArray = context.finish()
	return digest.hex_encode()

func _read_exact(path: String, byte_count: int, digest: String) -> PackedByteArray:
	var bytes: PackedByteArray = FileAccess.get_file_as_bytes(path)
	if bytes.size() != byte_count or _sha256(bytes) != digest:
		return PackedByteArray()
	return bytes

func _load_texture(asset: Dictionary) -> Texture2D:
	var path_value: Variant = asset.get('path')
	var size_value: Variant = asset.get('bytes')
	var hash_value: Variant = asset.get('sha256')
	if typeof(path_value) != TYPE_STRING or not P.counter(size_value,1,4194304) or typeof(hash_value) != TYPE_STRING:
		return null
	var asset_path: String = path_value
	var asset_hash: String = hash_value
	var bytes: PackedByteArray = _read_exact(asset_path,int(size_value),asset_hash)
	if bytes.is_empty():
		return null
	var image: Image = Image.new()
	var image_error: int = image.load_png_from_buffer(bytes)
	if image_error != OK or image.get_format() != Image.FORMAT_RGBA8:
		return null
	var canvas_value: Variant = asset.get('canvas')
	if not canvas_value is Array or canvas_value.size() != 2 or not P.counter(canvas_value[0],1,4096) or not P.counter(canvas_value[1],1,4096):
		return null
	if image.get_width() != int(canvas_value[0]) or image.get_height() != int(canvas_value[1]):
		return null
	return ImageTexture.create_from_image(image)

func prepare() -> bool:
	if attempted:
		return definition_valid
	attempted = true
	var map_bytes: PackedByteArray = FileAccess.get_file_as_bytes(MAP_PATH)
	if map_bytes.is_empty() or map_bytes.size() > 16384:
		return false
	var map_text: String = map_bytes.get_string_from_utf8()
	var parsed_value: Variant = JSON.parse_string(map_text)
	if not parsed_value is Dictionary or not _same(parsed_value,EXPECTED_MAP):
		return false
	var parsed: Dictionary = parsed_value
	var contracts_value: Variant = parsed.get('sourceContracts')
	if not contracts_value is Array or contracts_value.size() != 2:
		return false
	for contract_value in contracts_value:
		if not contract_value is Dictionary:
			return false
		var contract: Dictionary = contract_value
		var contract_path: Variant = contract.get('path')
		var contract_bytes: Variant = contract.get('bytes')
		var contract_hash: Variant = contract.get('sha256')
		if typeof(contract_path) != TYPE_STRING or not P.counter(contract_bytes,1,16384) or typeof(contract_hash) != TYPE_STRING:
			return false
		var checked_contract_path: String = contract_path
		var checked_contract_hash: String = contract_hash
		var source_bytes: PackedByteArray = _read_exact(checked_contract_path,int(contract_bytes),checked_contract_hash)
		if source_bytes.is_empty():
			return false
	var assets_value: Variant = parsed.get('assets')
	if not assets_value is Dictionary:
		return false
	var assets: Dictionary = assets_value
	var stone_value: Variant = assets.get('stone')
	var fog_value: Variant = assets.get('fog')
	if not stone_value is Dictionary or not fog_value is Dictionary:
		return false
	var stone_asset: Dictionary = stone_value
	var fog_asset: Dictionary = fog_value
	var stone_texture: Texture2D = _load_texture(stone_asset)
	var fog_texture: Texture2D = _load_texture(fog_asset)
	if stone_texture == null or fog_texture == null:
		return false
	textures = {'stone':stone_texture,'fog':fog_texture}
	definition_valid = true
	return true

func _context(snapshot_value: Variant, scene_value: Variant, key_value: Variant, scene_hash_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not definition_valid or not snapshot_value is Dictionary or not scene_value is Dictionary or typeof(key_value) != TYPE_STRING or typeof(scene_hash_value) != TYPE_STRING:
		return false
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	if not P.snapshot(snapshot) or snapshot.get('entryToken') != null or snapshot.get('worldId') != 'mist_harbor' or snapshot.get('sceneId') != 'mh_warden_arena' or not P.counter(snapshot.get('worldEpoch'),1):
		return false
	if key_value != 'mist_harbor:mh_warden_arena:'+str(snapshot.get('worldEpoch')) or scene_hash_value != SCENE_SHA:
		return false
	if scene.get('worldId') != 'mist_harbor' or scene.get('sceneId') != 'mh_warden_arena' or not P.counter(scene.get('schemaVersion'),1,1):
		return false
	if not _same(scene.get('boundsM'),EXPECTED_MAP.scene.boundsM) or not _same(scene.get('walkablePolygons'),EXPECTED_MAP.scene.walkablePolygons):
		return false
	var collision_value: Variant = scene.get('collision')
	if not collision_value is Array or collision_value.size() != WALL_ROWS.size():
		return false
	var seen_walls: Dictionary = {}
	for wall_value in collision_value:
		if not wall_value is Dictionary:
			return false
		var wall: Dictionary = wall_value
		var wall_id: Variant = wall.get('id')
		if typeof(wall_id) != TYPE_STRING or not WALLS.has(wall_id) or seen_walls.has(wall_id):
			return false
		var expected_wall: Dictionary = {'id':wall_id,'polygon':WALLS[wall_id]}
		if not _same(wall,expected_wall):
			return false
		seen_walls[wall_id] = true
	if seen_walls.size() != WALLS.size():
		return false
	var presentation_value: Variant = scene.get('presentation')
	if not presentation_value is Dictionary:
		return false
	var presentation: Dictionary = presentation_value
	if not _same(presentation.get('sprites'),SPRITE_ROWS) or not _same(scene.get('vfxMarkers'),MARKER_ROWS):
		return false
	return _projection(camera_value,viewport_value)

func _projection(camera_value: Variant, viewport_value: Variant) -> bool:
	if not camera_value is ProjectionScript or not _valid_rect(viewport_value):
		return false
	var camera: ProjectionScript = camera_value
	var viewport: Rect2 = viewport_value
	if typeof(camera.initialized) != TYPE_BOOL or not camera.initialized or not _finite_v3(camera.origin) or not _finite_v2(camera.screen_center) or not _finite_number(camera.ppm) or camera.ppm <= 0.0 or camera.ppm > 256.0:
		return false
	if not _finite_v2(camera.viewport) or camera.viewport != viewport.size:
		return false
	return true

func _canvas_valid(canvas_value: Variant) -> bool:
	if not canvas_value is CanvasItem:
		return false
	var canvas: CanvasItem = canvas_value
	return canvas.is_inside_tree() and not canvas.is_queued_for_deletion()

func _project(camera: ProjectionScript, point: Vector3) -> Vector2:
	return camera.project(point)

func _points_finite(points: PackedVector2Array) -> bool:
	if points.size() < 3:
		return false
	for point in points:
		if not is_finite(point.x) or not is_finite(point.y):
			return false
	return true

func draw_ground(canvas_value: Variant, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not _canvas_valid(canvas_value) or not _context(snapshot,scene,key,scene_hash,camera_value,viewport_value):
		return false
	var camera: ProjectionScript = camera_value
	var canvas: CanvasItem = canvas_value
	var texture_value: Variant = textures.get('stone')
	if not texture_value is Texture2D:
		return false
	var texture: Texture2D = texture_value
	var patches: Array = []
	var widths: Array[float] = [3.0,3.0,3.0,3.0,3.0,3.0,3.0,2.0]
	var depths: Array[float] = [3.0,3.0,3.0,3.0,3.0]
	var x0: float = 0.5
	for column_index in range(widths.size()):
		var width: float = widths[column_index]
		var z0: float = 0.5
		for row_index in range(depths.size()):
			var depth: float = depths[row_index]
			var u0: float = 0.0
			var u1: float = width/3.0
			if column_index % 2 == 1:
				u0 = 1.0
				u1 = 1.0-width/3.0
			var v0: float = 0.0
			var v1: float = depth/3.0
			if row_index % 2 == 1:
				v0 = 1.0
				v1 = 1.0-depth/3.0
			var source_a: Vector2 = Vector2(820,240)
			var source_b: Vector2 = Vector2(1032,342)
			var source_d: Vector2 = Vector2(180,525)
			var source_size: Vector2 = Vector2(1228,1254)
			var source_uv: Array[Vector2] = [source_a+(source_b-source_a)*u0+(source_d-source_a)*v0,source_a+(source_b-source_a)*u1+(source_d-source_a)*v0,source_a+(source_b-source_a)*u1+(source_d-source_a)*v1,source_a+(source_b-source_a)*u0+(source_d-source_a)*v1]
			var uv_points: PackedVector2Array = PackedVector2Array()
			var projected_points: PackedVector2Array = PackedVector2Array()
			for index in range(4):
				uv_points.append(source_uv[index]/source_size)
			var world_points: Array[Vector3] = [Vector3(x0,0,z0),Vector3(x0+width,0,z0),Vector3(x0+width,0,z0+depth),Vector3(x0,0,z0+depth)]
			for world_point in world_points:
				projected_points.append(_project(camera,world_point))
			if not _points_finite(projected_points) or uv_points.size() != 4:
				return false
			patches.append({'points':projected_points,'uvs':uv_points})
			z0 += depth
		x0 += width
	if patches.size() != 40:
		return false
	var tint: Color = Color(0.34,0.41,0.44,1.0)
	for patch_value in patches:
		if not patch_value is Dictionary:
			return false
		var patch: Dictionary = patch_value
		var points_value: Variant = patch.get('points')
		var uvs_value: Variant = patch.get('uvs')
		if typeof(points_value) != TYPE_PACKED_VECTOR2_ARRAY or typeof(uvs_value) != TYPE_PACKED_VECTOR2_ARRAY:
			return false
		var points: PackedVector2Array = points_value
		var uvs: PackedVector2Array = uvs_value
		var colors: PackedColorArray = PackedColorArray([tint,tint,tint,tint])
		canvas.draw_polygon(points,colors,uvs,texture)
	return true

func _contact_polygon(wall_id: String) -> Array[Vector2]:
	if wall_id == 'warden_wall_north':
		return [Vector2(0,0.44),Vector2(24,0.44),Vector2(24,0.5),Vector2(0,0.5)]
	if wall_id == 'warden_wall_south':
		return [Vector2(0,15.5),Vector2(24,15.5),Vector2(24,15.56),Vector2(0,15.56)]
	if wall_id == 'warden_wall_west_north':
		return [Vector2(0.44,0.5),Vector2(0.5,0.5),Vector2(0.5,6),Vector2(0.44,6)]
	if wall_id == 'warden_wall_west_south':
		return [Vector2(0.44,9.5),Vector2(0.5,9.5),Vector2(0.5,15.5),Vector2(0.44,15.5)]
	if wall_id == 'warden_wall_east_north':
		return [Vector2(23.5,0.5),Vector2(23.56,0.5),Vector2(23.56,6),Vector2(23.5,6)]
	if wall_id == 'warden_wall_east_south':
		return [Vector2(23.5,9.5),Vector2(23.56,9.5),Vector2(23.56,15.5),Vector2(23.5,15.5)]
	return []

func draw_wall(canvas_value: Variant, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, wall_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not _canvas_valid(canvas_value) or not _context(snapshot,scene,key,scene_hash,camera_value,viewport_value) or not wall_value is Dictionary:
		return false
	var wall: Dictionary = wall_value
	var wall_id: Variant = wall.get('id')
	if typeof(wall_id) != TYPE_STRING or not WALLS.has(wall_id):
		return false
	var expected_wall: Dictionary = {'id':wall_id,'polygon':WALLS[wall_id]}
	if not _same(wall,expected_wall):
		return false
	var camera: ProjectionScript = camera_value
	var canvas: CanvasItem = canvas_value
	var polygon_value: Variant = wall.get('polygon')
	if not polygon_value is Array or polygon_value.size() != 4:
		return false
	var world_points: Array[Vector2] = []
	for point_value in polygon_value:
		if not point_value is Array or point_value.size() != 2 or not P.number(point_value[0],0.0,24.0) or not P.number(point_value[1],0.0,16.0):
			return false
		world_points.append(Vector2(float(point_value[0]),float(point_value[1])))
	var high_ids: Array = ['warden_wall_north','warden_wall_west_north','warden_wall_west_south']
	var height: float = 0.5 if wall_id in high_ids else 0.18
	var side_faces: Array[PackedVector2Array] = []
	for index in range(world_points.size()):
		var first: Vector2 = world_points[index]
		var second: Vector2 = world_points[(index+1)%world_points.size()]
		var face: PackedVector2Array = PackedVector2Array([
			_project(camera,Vector3(first.x,0,first.y)),
			_project(camera,Vector3(second.x,0,second.y)),
			_project(camera,Vector3(second.x,height,second.y)),
			_project(camera,Vector3(first.x,height,first.y))
		])
		if not _points_finite(face):
			return false
		side_faces.append(face)
	var cap_points: PackedVector2Array = PackedVector2Array()
	for point in world_points:
		cap_points.append(_project(camera,Vector3(point.x,height,point.y)))
	if not _points_finite(cap_points):
		return false
	var contact_points: PackedVector2Array = PackedVector2Array()
	for point in _contact_polygon(wall_id):
		contact_points.append(_project(camera,Vector3(point.x,0,point.y)))
	if not _points_finite(contact_points):
		return false
	var side_color: Color = Color('#293f47')
	for face_points in side_faces:
		canvas.draw_colored_polygon(face_points,side_color)
	canvas.draw_colored_polygon(cap_points,Color('#485f67'))
	canvas.draw_colored_polygon(contact_points,Color(0.025,0.032,0.035,0.1))
	return true

# Additional binding is dynamic-only; the original static fallback is unchanged.
func _pulse_context(canvas, snapshot: Dictionary, scene: Dictionary, key: String, scene_hash: String, camera: ProjectionScript, viewport: Rect2, rows: Array, gate: Dictionary) -> bool:
	if not textures.get('stone') is Texture2D or not textures.get('fog') is Texture2D:
		return false
	if canvas.snapshot!=snapshot or canvas.scene!=scene or canvas.current_key!=key or canvas.scene_hash!=scene_hash or canvas.camera!=camera or canvas.viewport_rect!=viewport or canvas.get_viewport_rect()!=viewport or canvas.events!=rows:
		return false
	if not _context(snapshot,scene,key,scene_hash,camera,viewport):
		return false
	for flag in ['presentation_allowed','visible','reduced_motion']:
		if typeof(gate.get(flag))!=TYPE_BOOL:
			return false
	if gate.get('authorityRevision')!=snapshot.authorityRevision or gate.get('player_id')!=snapshot.player.entityId:
		return false
	return gate==canvas._locomotion_gate()

# No state/timers retained by this helper. Existing accepted events own lifecycle.
func _pulse_packet(snapshot: Dictionary, rows: Array, gate: Dictionary, canvas) -> Dictionary:
	if gate.get('worldId')!=snapshot.worldId or gate.get('sceneId')!=snapshot.sceneId or gate.get('worldEpoch')!=snapshot.worldEpoch or gate.get('serverTick')!=snapshot.serverTick or gate.get('presentation_allowed')!=true or gate.get('visible')!=true or gate.get('reduced_motion')!=false:
		return {}
	var chosen: Dictionary = {}
	var now: int = Time.get_ticks_msec()
	for value in rows:
		if not value is Dictionary:
			continue
		var e: Dictionary = value
		if not EventViewScript.warden_pulse_authorized(e,snapshot,canvas) or not P.number(e.get('_duration'),600,600) or not P.counter(e.get('_accepted')) or not P.number(e.get('_initial_age'),0,600) or now<int(e._accepted):
			continue
		var age: float = maxf((snapshot.serverTick-e.serverTick)*1000.0/60.0,float(e._initial_age)+now-int(e._accepted))
		if age<0.0 or age>=600.0:
			continue
		if chosen.is_empty() or e.eventId>chosen.event_id:
			var phase: float = age/600.0
			var wave: float = sin(PI*phase)
			chosen = {'event_id':e.eventId,'origin':P.vector(e.positionM),'strength':wave*wave}
	return chosen

func _wisp_response(anchor: Vector3, pulse: Dictionary, canvas) -> Dictionary:
	var unchanged: Dictionary = {'offset':Vector3.ZERO,'alpha_scale':1.0}
	if pulse.is_empty() or not canvas.visible_point(anchor):
		return unchanged
	var origin: Vector3 = pulse.origin
	var delta: Vector3 = Vector3(anchor.x-origin.x,0,anchor.z-origin.z)
	var distance: float = delta.length()
	# Every outer-profile vertex is within 0.16m in XZ; inner is smaller.
	# Full envelope plus maximum translation remains inside actual 3.5m circle.
	if distance+0.16+0.16>3.5:
		return unchanged
	var offset: Vector3 = Vector3.ZERO
	if distance>0.0001:
		offset = delta/distance*(0.16*float(pulse.strength))
	return {'offset':offset,'alpha_scale':1.0-0.25*float(pulse.strength)}

func _air_plan(camera: ProjectionScript, pulse: Dictionary, canvas) -> Array[Dictionary]:
	var centers: Array[Array] = [[8,0,4],[18,0,11]]
	var x_offsets: Array[float] = [-0.2,0.0,0.2]
	var heights: Array[float] = [0.16,0.28,0.20]
	var profile: Array[Vector3] = [Vector3(-0.12,0,0),Vector3(-0.10,0.05,-0.03),Vector3(-0.06,0.70,-0.06),Vector3(0,1,-0.08),Vector3(0.05,0.78,-0.06),Vector3(0.10,0.08,-0.02),Vector3(0.12,0,0),Vector3(0,0.025,0.10)]
	var plans: Array[Dictionary] = []
	for center_value in centers:
		for wisp_index in range(3):
			var center: Array = center_value
			var height: float = heights[wisp_index]
			var offset_x: float = x_offsets[wisp_index]
			var response: Dictionary = _wisp_response(Vector3(float(center[0])+offset_x,0,float(center[2])),pulse,canvas)
			var displacement: Vector3 = response.offset
			for layer_index in range(2):
				var scale_xz: float = 1.0 if layer_index == 0 else 0.65
				var scale_y: float = 1.0 if layer_index == 0 else 0.75
				var alpha: float = (0.055 if layer_index == 0 else 0.025)*float(response.alpha_scale)
				var projected: PackedVector2Array = PackedVector2Array()
				for local_point in profile:
					var x: float = float(center[0])+offset_x+local_point.x*scale_xz+displacement.x
					var z: float = float(center[2])+local_point.z*scale_xz+displacement.z
					var y: float = local_point.y*height*scale_y
					if x<0.5 or x>23.5 or z<0.5 or z>15.5 or y<0.0 or y>0.3:
						return []
					projected.append(_project(camera,Vector3(x,y,z)))
				if projected.size() != 8 or not _points_finite(projected):
					return []
				var bounds: Rect2 = Rect2(projected[0],Vector2.ZERO)
				for point_index in range(1,projected.size()):
					bounds = bounds.expand(projected[point_index])
				if not _valid_rect(bounds) or not is_finite(alpha) or alpha < 0.0 or alpha > 0.055:
					return []
				plans.append({'points':projected,'bounds':bounds,'alpha':alpha})
	return plans

func draw_back_air(canvas_value: Variant, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, camera_value: Variant, viewport_value: Variant, accepted_events: Array, gate: Dictionary) -> Dictionary:
	var report: Dictionary = {'bank_ids':[],'ambient_polygons':0}
	if not _canvas_valid(canvas_value) or not _context(snapshot,scene,key,scene_hash,camera_value,viewport_value):
		return report
	var canvas: CanvasItem = canvas_value
	var camera: ProjectionScript = camera_value
	var viewport: Rect2 = viewport_value
	var pulse: Dictionary = {}
	if _pulse_context(canvas,snapshot,scene,key,scene_hash,camera,viewport,accepted_events,gate):
		pulse = _pulse_packet(snapshot,accepted_events,gate,canvas)
	var plans: Array[Dictionary] = _air_plan(camera,pulse,canvas)
	# Recheck after planning; lost dynamic authority rebuilds the original static plan.
	if not pulse.is_empty():
		var current_pulse: Dictionary = {}
		if _pulse_context(canvas,snapshot,scene,key,scene_hash,camera,viewport,accepted_events,gate):
			current_pulse = _pulse_packet(snapshot,accepted_events,gate,canvas)
		if current_pulse.is_empty() or current_pulse.get('event_id')!=pulse.event_id or current_pulse.get('origin')!=pulse.origin:
			plans = _air_plan(camera,{},canvas)
	if plans.size() != 12:
		return report
	var texture_value: Variant = textures.get('fog')
	var fog_rect: Rect2 = Rect2()
	var fog_drawable: bool = false
	if texture_value is Texture2D:
		var center: Vector2 = _project(camera,Vector3(18,0,3))
		var width: float = 9.0*camera.ppm
		var height: float = width*498.0/1201.0
		fog_rect = Rect2(center-Vector2(width,height)*0.5,Vector2(width,height))
		fog_drawable = _valid_rect(fog_rect)
		if not fog_drawable:
			return report
	if texture_value is Texture2D and fog_drawable and viewport.intersects(fog_rect):
		var texture: Texture2D = texture_value
		canvas.draw_texture_rect_region(texture,fog_rect,Rect2(Vector2(27,370),Vector2(1201,498)),Color(0.8,0.88,0.92,0.38),false)
		var bank_ids: Array[String] = ['warden_arena_fog_bank']
		report['bank_ids'] = bank_ids
	var ambient_count: int = 0
	for plan_value in plans:
		if not plan_value is Dictionary:
			return {'bank_ids':report.bank_ids,'ambient_polygons':ambient_count}
		var plan: Dictionary = plan_value
		var points_value: Variant = plan.get('points')
		var bounds_value: Variant = plan.get('bounds')
		if typeof(points_value) != TYPE_PACKED_VECTOR2_ARRAY or typeof(bounds_value) != TYPE_RECT2:
			return {'bank_ids':report.bank_ids,'ambient_polygons':ambient_count}
		var points: PackedVector2Array = points_value
		var polygon_bounds: Rect2 = bounds_value
		if not _valid_rect(polygon_bounds) or not viewport.intersects(polygon_bounds):
			continue
		var alpha_value: Variant = plan.get('alpha')
		if not P.number(alpha_value,0.0,0.055):
			return {'bank_ids':report.bank_ids,'ambient_polygons':ambient_count}
		canvas.draw_colored_polygon(points,Color('#90a9b2',float(alpha_value)))
		ambient_count += 1
	report['ambient_polygons'] = ambient_count
	return report

func legacy_bank_consumed(prop_value: Variant, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, camera: Variant, viewport: Variant, consumed_value: Variant) -> bool:
	if not _context(snapshot,scene,key,scene_hash,camera,viewport) or not consumed_value is Array or consumed_value.size() > 1 or not prop_value is Dictionary:
		return false
	var seen: bool = false
	for value in consumed_value:
		if typeof(value) != TYPE_STRING or value != 'warden_arena_fog_bank' or seen:
			return false
		seen = true
	var prop: Dictionary = prop_value
	var expected_bank: Dictionary = {'id':'warden_arena_fog_bank','assetId':'runtime2d.world.mistharbor.fog_bank.v1','layer':'visual.props_back','position':[18,0,3]}
	return seen and textures.get('fog') is Texture2D and _same(prop,expected_bank)
