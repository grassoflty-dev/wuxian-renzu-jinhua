extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const FurnaceScript = preload('res://scripts/world/cw_furnace_visual.gd')
const SCENE_PATH = 'res://data/scenes/cw_entry_foundry.json'
const SCENE_BYTES = 5551
const SCENE_SHA256 = 'd90f852367b998cf5874b82fed583f1254664d4a88a8175bb3c94524b8f496b1'
const USE_PATH = 'res://data/cw_entry_foundry_environment.internal-use.json'
const STONE_PATH = 'res://assets/props/clockworks/pressure_hall_steel_floor_source.png'
const STONE_BYTES = 1610704
const STONE_SHA256 = '0b4555021799ac849756e0f11c207d4e9309df19f4b8f68d0f4e8177f5204043'
const EXPECTED_USE: Dictionary = {
	'schema':'wuxian.cw-entry-foundry-environment.internal-use/v1',
	'registeredSourceReuse':true,
	'approval':'INTERNAL_REGISTERED_SOURCE_REUSE',
	'GodotDisplay':'NOT_RUN',
	'publicRelease':'NOT_ESTABLISHED',
	'roomUse':'MAIN_ADMITTED_CW_ENTRY_FOUNDRY_ONLY',
	'sourceContracts':[
		{'path':'res://data/cw_furnaces.internal-use.json','bytes':1670,'sha256':'c797ea5392842aeb0427df01bad7b8562b2e626b05398df6e624c04d88a480d6'},
		{'path':'res://data/cw_pressure_floor_material.internal-use.json','bytes':1391,'sha256':'746dbb35f40af89c300d1043322f634663a1f58bb6130a8711a84c972b35015a'}
	],
	'asset':{
		'assetId':'runtime2d.world.clockworks.floor_steel.v1',
		'derivedRelative':'assets/derived/asset-pipeline-v1/sprites/png/runtime2d.world.clockworks.floor_steel.v1.png',
		'sourceRelative':'batch07/05_runtime2d.world.clockworks.floor_steel.v1.png',
		'sourceSha256':'1a01e851654d9f5c98b5486cbe489f357ed99c55ffc601c492af63044d90b190',
		'path':'res://assets/props/clockworks/pressure_hall_steel_floor_source.png',
		'bytes':1610704,
		'sha256':'0b4555021799ac849756e0f11c207d4e9309df19f4b8f68d0f4e8177f5204043',
		'canvas':[1197,1132],
		'format':'RGBA8'
	},
	'installation':{
		'worldId':'clockworks','sceneId':'cw_entry_foundry','boundsM':[0,0,24,16],
		'rowId':'cw_entry_foundry_steel_floor','layer':'visual.floor','position':[12,0,8],'groundY':0
	},
	'steel':{
		'sourcePixelEdgeCorners':[[510,200],[970,460],[600,740],[140,480]],
		'uvConvention':'sourcePixelEdge / standaloneCanvasSize','patchSizeM':[3,3],
		'grid':[8,6],'patchCount':48,'lastRowUvDepthFraction':0.3333333333333333,
		'edgeCells':'CLIP_WORLD_AND_UV_FRACTIONS_NOT_STRETCH','tint':[0.35,0.4,0.43,1]
	},
	'walls':{
		'segments':[
			{'id':'cw_entry_foundry_wall_north','polygon':[[0,0],[24,0],[24,0.5],[0,0.5]],'displayHeightM':1},
			{'id':'cw_entry_foundry_wall_south','polygon':[[0,15.5],[24,15.5],[24,16],[0,16]],'displayHeightM':0.18},
			{'id':'cw_entry_foundry_wall_west_north','polygon':[[0,0.5],[0.5,0.5],[0.5,6],[0,6]],'displayHeightM':1},
			{'id':'cw_entry_foundry_wall_west_south','polygon':[[0,9.5],[0.5,9.5],[0.5,15.5],[0,15.5]],'displayHeightM':1},
			{'id':'cw_entry_foundry_wall_east_north','polygon':[[23.5,0.5],[24,0.5],[24,6],[23.5,6]],'displayHeightM':0.18},
			{'id':'cw_entry_foundry_wall_east_south','polygon':[[23.5,9.5],[24,9.5],[24,15.5],[23.5,15.5]],'displayHeightM':0.18}
		],
		'sideColor':[0.18,0.22,0.24,1],
		'capColor':[0.25,0.29,0.31,1],
		'contactColor':[0.04,0.06,0.065,0.12],
		'contactWidthM':0.05,'buttJoints':true,'openingsRemainUnfilled':true
	},
	'furnaceGroundCue':{
		'rowId':'cw_entry_furnace','key':'furnace_v2','position':[19,0,6],
		'radiiM':[[1.25,0.55],[0.9,0.38],[0.45,0.2]],
		'color':[0.62,0.42,0.25],'alphas':[0.02,0.035,0.045],
		'verticesPerRing':24,'static':true,'requiresExistingCacheAndCanvasRect':true
	}
}
const WALL_CHAIN_USE_PATH = 'res://data/cw_entry_foundry_wall_chain.internal-use.json'
const WALL_CHAIN_SOURCE_PATH = 'res://assets/props/clockworks/foundry_wall_chain_source.png'
const WALL_CHAIN_EXPECTED_USE: Dictionary = {
	"schema": "wuxian.cw-entry-foundry-wall-chain.internal-use/v1",
	"approval": "SOURCE_ONLY_NEW_USE_APPROVED",
	"publicRelease": "NOT_ESTABLISHED",
	"GodotDisplay": "NOT_RUN",
	"runtimeValidation": "NOT_RUN",
	"sourceHistoricalStatus": "PARTIAL_SOURCE_CANDIDATE_NOT_RUNTIME_ADMITTED",
	"source": {
		"path": "res://assets/props/clockworks/foundry_wall_chain_source.png",
		"bytes": 819043,
		"sha256": "001a331d8f9cd7caeba3432fc7325782f1b5a0230ff514b0f2b45aa72f6cfd7f",
		"gitBlob": "075ea0cb49a2a0690346a589190aa3ef0bac5714",
		"canvas": [
			1983,
			793
		],
		"format": "RGBA8"
	},
	"installation": {
		"worldId": "clockworks",
		"sceneId": "cw_entry_foundry",
		"wallId": "cw_entry_foundry_wall_north",
		"wallPolygon": [
			[
				0,
				0
			],
			[
				24,
				0
			],
			[
				24,
				0.5
			],
			[
				0,
				0.5
			]
		],
		"visualHeightM": 1,
		"scene": {
			"path": "res://data/scenes/cw_entry_foundry.json",
			"bytes": 5551,
			"sha256": "d90f852367b998cf5874b82fed583f1254664d4a88a8175bb3c94524b8f496b1",
			"gitBlob": "3ad44fa1c5aab21738ffcbf744eb1059aadf303a"
		}
	},
	"sourcePixelEdgeCorners": [
		[
			0,
			0
		],
		[
			1983,
			0
		],
		[
			1983,
			793
		],
		[
			0,
			793
		]
	],
	"uvConvention": "completeStandaloneCanvasEdges",
	"UV": [
		[
			0,
			0
		],
		[
			1,
			0
		],
		[
			1,
			1
		],
		[
			0,
			1
		]
	],
	"worldCorners": [
		[
			4,
			0.8,
			0.5
		],
		[
			5.5,
			0.8,
			0.5
		],
		[
			5.5,
			0.2,
			0.5
		],
		[
			4,
			0.2,
			0.5
		]
	],
	"modulate": [
		1,
		1,
		1,
		1
	],
	"static": true,
	"marginException": {
		"requested12PercentTransparentMarginNotMet": true,
		"acceptedForThisFullCanvasWallUseOnly": true,
		"atlasOrCropClaim": false,
		"originalDimensionsPreserved": true
	},
	"sameFrameRequiresOriginalNorthWallDraw": true,
	"resourceFailure": "skipsOptionalChainOnly",
	"noFloorShadow": true,
	"noCollision": true
}
const WALLS: Dictionary = {
	'cw_entry_foundry_wall_north':{'polygon':[[0,0],[24,0],[24,0.5],[0,0.5]],'height':1.0,'inner':[0.0,0.5,24.0,0.5],'contact': [0.0,-1.0]},
	'cw_entry_foundry_wall_south':{'polygon':[[0,15.5],[24,15.5],[24,16],[0,16]],'height':0.18,'inner':[0.0,15.5,24.0,15.5],'contact':[0.0,1.0]},
	'cw_entry_foundry_wall_west_north':{'polygon':[[0,0.5],[0.5,0.5],[0.5,6],[0,6]],'height':1.0,'inner':[0.5,0.5,0.5,6.0],'contact':[-1.0,0.0]},
	'cw_entry_foundry_wall_west_south':{'polygon':[[0,9.5],[0.5,9.5],[0.5,15.5],[0,15.5]],'height':1.0,'inner':[0.5,9.5,0.5,15.5],'contact':[-1.0,0.0]},
	'cw_entry_foundry_wall_east_north':{'polygon':[[23.5,0.5],[24,0.5],[24,6],[23.5,6]],'height':0.18,'inner':[23.5,0.5,23.5,6.0],'contact':[1.0,0.0]},
	'cw_entry_foundry_wall_east_south':{'polygon':[[23.5,9.5],[24,9.5],[24,15.5],[23.5,15.5]],'height':0.18,'inner':[23.5,9.5,23.5,15.5],'contact':[1.0,0.0]}
}
# BEGIN short union face data
const SHORT_UNION_FACES: Array = [
	{"id":"north_east_end","owner":"cw_entry_foundry_wall_north","edge":[1,2],"low_owner":""},
	{"id":"south_east_end","owner":"cw_entry_foundry_wall_south","edge":[1,2],"low_owner":""},
	{"id":"east_north_jamb","owner":"cw_entry_foundry_wall_east_north","edge":[3,2],"low_owner":""},
	{"id":"west_south_step","owner":"cw_entry_foundry_wall_west_south","edge":[3,2],"low_owner":"cw_entry_foundry_wall_south"}
]
# END short union face data
const WEST_NORTH_REVEAL_WORLD = [Vector3(0.0,0.0,6.0),Vector3(0.5,0.0,6.0),Vector3(0.5,1.0,6.0),Vector3(0.0,1.0,6.0)]
const SPRITES: Dictionary = {
	'cw_entry_foundry_steel_floor':['runtime2d.world.clockworks.floor_steel.v1','visual.floor',[12,0,8]],
	'cw_entry_foundry_factory_tiles':['runtime2d.world.clockworks.factory_tiles.v1','visual.floor',[12,0,8]],
	'cw_entry_furnace':['runtime2d.world.clockworks.furnace.v2','visual.props_back',[19,0,6]],
	'cw_entry_factory_module':['runtime2d.world.clockworks.factory_tiles.v1','visual.props_back',[11,0,4]]
}
var preparation_attempted: bool = false
var use_map_valid: bool = false
var stone_texture: Texture2D
var canonical_scene: Dictionary = {}
var current_scene_ready: bool = false
var wall_chain_preparation_attempted: bool = false
var wall_chain_texture: Texture2D = null

func _same_contract(actual: Variant, expected: Variant) -> bool:
	var expected_type: int = typeof(expected)
	if expected_type == TYPE_INT or expected_type == TYPE_FLOAT:
		return P.number(actual,float(expected),float(expected))
	if expected_type == TYPE_DICTIONARY:
		if not actual is Dictionary or actual.size()!=expected.size():
			return false
		for key in expected.keys():
			if not actual.has(key):
				return false
			var actual_value: Variant = actual[key]
			var expected_value: Variant = expected[key]
			if not _same_contract(actual_value,expected_value):
				return false
		return true
	if expected_type == TYPE_ARRAY:
		if not actual is Array or actual.size()!=expected.size():
			return false
		for index in range(expected.size()):
			var actual_item: Variant = actual[index]
			var expected_item: Variant = expected[index]
			if not _same_contract(actual_item,expected_item):
				return false
		return true
	return typeof(actual)==expected_type and actual==expected

func _context_valid(snapshot: Dictionary, scene: Dictionary, current_key: String) -> bool:
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null:
		return false
	if snapshot.get('worldId')!='clockworks' or snapshot.get('sceneId')!='cw_entry_foundry' or scene.get('worldId')!='clockworks' or scene.get('sceneId')!='cw_entry_foundry':
		return false
	if not P.counter(snapshot.get('worldEpoch'),1) or current_key!='clockworks:cw_entry_foundry:'+str(snapshot.get('worldEpoch')):
		return false
	if not P.counter(scene.get('schemaVersion'),1,1):
		return false
	var bounds_value: Variant = scene.get('boundsM')
	if not bounds_value is Dictionary:
		return false
	var bounds: Dictionary = bounds_value
	if not P.number(bounds.get('x'),0,0) or not P.number(bounds.get('z'),0,0) or not P.number(bounds.get('width'),24,24) or not P.number(bounds.get('depth'),16,16):
		return false
	var collision_value: Variant = scene.get('collision')
	if not collision_value is Array or collision_value.size()!=6:
		return false
	var collision_ids: Dictionary = {}
	for row_value in collision_value:
		if not row_value is Dictionary:
			return false
		var wall_id_value: Variant = row_value.get('id')
		if typeof(wall_id_value)!=TYPE_STRING or not WALLS.has(wall_id_value) or collision_ids.has(wall_id_value):
			return false
		var wall_polygon_value: Variant = row_value.get('polygon')
		if not _same_contract(wall_polygon_value,WALLS[wall_id_value].polygon):
			return false
		collision_ids[wall_id_value] = true
	if collision_ids.size()!=WALLS.size():
		return false
	var presentation_value: Variant = scene.get('presentation')
	if not presentation_value is Dictionary:
		return false
	var sprites_value: Variant = presentation_value.get('sprites')
	if not sprites_value is Array or sprites_value.size()!=SPRITES.size():
		return false
	var sprite_ids: Dictionary = {}
	for sprite_value in sprites_value:
		if not sprite_value is Dictionary:
			return false
		var sprite_id_value: Variant = sprite_value.get('id')
		if typeof(sprite_id_value)!=TYPE_STRING or not SPRITES.has(sprite_id_value) or sprite_ids.has(sprite_id_value):
			return false
		var expected_sprite: Array = SPRITES[sprite_id_value]
		if not _same_contract(sprite_value.get('assetId'),expected_sprite[0]) or not _same_contract(sprite_value.get('layer'),expected_sprite[1]) or not _same_contract(sprite_value.get('position'),expected_sprite[2]):
			return false
		sprite_ids[sprite_id_value] = true
	return sprite_ids.size()==SPRITES.size()

func _read_bytes(path: String, maximum: int) -> PackedByteArray:
	var file: FileAccess = FileAccess.open(path,FileAccess.READ)
	if file==null:
		return PackedByteArray()
	var length: int = file.get_length()
	if length<=0 or length>maximum:
		file.close()
		return PackedByteArray()
	var bytes: PackedByteArray = file.get_buffer(length)
	file.close()
	if bytes.size()!=length:
		return PackedByteArray()
	return bytes

func _read_text_bytes(path: String, maximum: int) -> PackedByteArray:
	var bytes: PackedByteArray = _read_bytes(path,maximum)
	if bytes.is_empty() or bytes.get_string_from_utf8().to_utf8_buffer()!=bytes:
		return PackedByteArray()
	return bytes

func _sha256(bytes: PackedByteArray) -> String:
	var hash: HashingContext = HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256)!=OK:
		return ''
	if hash.update(bytes)!=OK:
		return ''
	return hash.finish().hex_encode()

func _verify_source_contract(path: String, expected_bytes: int, expected_sha: String) -> bool:
	var bytes: PackedByteArray = _read_text_bytes(path,8192)
	return bytes.size()==expected_bytes and _sha256(bytes)==expected_sha

func prepare(snapshot: Dictionary, scene: Dictionary, current_key: String, scene_hash: String) -> bool:
	current_scene_ready = false
	if scene_hash!=SCENE_SHA256 or not _context_valid(snapshot,scene,current_key):
		return false
	_prepare_wall_chain_once()
	if preparation_attempted:
		current_scene_ready = use_map_valid and not canonical_scene.is_empty() and _same_contract(scene,canonical_scene)
		return current_scene_ready and stone_texture!=null
	preparation_attempted = true
	var scene_bytes: PackedByteArray = _read_text_bytes(SCENE_PATH,16384)
	if scene_bytes.size()!=SCENE_BYTES or _sha256(scene_bytes)!=SCENE_SHA256:
		return false
	var decoded_scene: Variant = JSON.parse_string(scene_bytes.get_string_from_utf8())
	if not decoded_scene is Dictionary or not _same_contract(scene,decoded_scene):
		return false
	canonical_scene = decoded_scene
	var use_bytes: PackedByteArray = _read_text_bytes(USE_PATH,16384)
	if use_bytes.is_empty() or use_bytes.size()>16384:
		return false
	var parsed_use: Variant = JSON.parse_string(use_bytes.get_string_from_utf8())
	if not parsed_use is Dictionary or not _same_contract(parsed_use,EXPECTED_USE):
		return false
	if not _verify_source_contract('res://data/cw_furnaces.internal-use.json',1670,'c797ea5392842aeb0427df01bad7b8562b2e626b05398df6e624c04d88a480d6'):
		return false
	if not _verify_source_contract('res://data/cw_pressure_floor_material.internal-use.json',1391,'746dbb35f40af89c300d1043322f634663a1f58bb6130a8711a84c972b35015a'):
		return false
	use_map_valid = true
	current_scene_ready = true
	var png_bytes: PackedByteArray = _read_bytes(STONE_PATH,4*1024*1024)
	if png_bytes.size()!=STONE_BYTES or _sha256(png_bytes)!=STONE_SHA256:
		return false
	var image: Image = Image.new()
	if image.load_png_from_buffer(png_bytes)!=OK or image.get_width()!=1197 or image.get_height()!=1132 or image.get_format()!=Image.FORMAT_RGBA8:
		return false
	stone_texture = ImageTexture.create_from_image(image)
	if stone_texture==null:
		return false
	return true

func _camera_valid(camera: Object, viewport: Rect2) -> bool:
	if camera==null or camera.get_script()!=ProjectionScript:
		return false
	var initialized_value: Variant = camera.get('initialized')
	var origin_value: Variant = camera.get('origin')
	var center_value: Variant = camera.get('screen_center')
	var ppm_value: Variant = camera.get('ppm')
	var camera_view_value: Variant = camera.get('viewport')
	if typeof(initialized_value)!=TYPE_BOOL or not initialized_value or not origin_value is Vector3 or not center_value is Vector2 or not camera_view_value is Vector2:
		return false
	if not P.number(ppm_value,0.000001,256) or not is_finite(origin_value.x) or not is_finite(origin_value.y) or not is_finite(origin_value.z) or not is_finite(center_value.x) or not is_finite(center_value.y):
		return false
	if not is_finite(viewport.position.x) or not is_finite(viewport.position.y) or not is_finite(viewport.size.x) or not is_finite(viewport.size.y) or not is_finite(viewport.end.x) or not is_finite(viewport.end.y) or viewport.size.x<=0 or viewport.size.y<=0:
		return false
	return camera_view_value.is_equal_approx(viewport.size)

func _live_canvas(canvas: CanvasItem) -> bool:
	return canvas!=null and canvas.is_inside_tree() and not canvas.is_queued_for_deletion()

func _draw_ready(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, current_key: String, camera: Object, viewport: Rect2) -> bool:
	return current_scene_ready and use_map_valid and not canonical_scene.is_empty() and _same_contract(scene,canonical_scene) and _context_valid(snapshot,scene,current_key) and _live_canvas(canvas) and _camera_valid(camera,viewport)

# BEGIN short union helper functions
func _short_bindings(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not canvas_value is CanvasItem or not is_instance_valid(canvas_value) or not canvas_value.is_inside_tree() or not canvas_value.is_visible_in_tree() or canvas_value.is_queued_for_deletion():
		return false
	if not snapshot_value is Dictionary or not snapshot_value.has('entryToken') or snapshot_value.get('entryToken')!=null or not scene_value is Dictionary or typeof(current_key_value)!=TYPE_STRING or not camera_value is Object or not is_instance_valid(camera_value) or not viewport_value is Rect2:
		return false
	var canvas: CanvasItem = canvas_value
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	var current_key: String = current_key_value
	var camera: Object = camera_value
	var viewport: Rect2 = viewport_value
	if not _draw_ready(canvas,snapshot,scene,current_key,camera,viewport):
		return false
	var canvas_snapshot_value: Variant = canvas.get('snapshot')
	var canvas_scene_value: Variant = canvas.get('scene')
	var canvas_key_value: Variant = canvas.get('current_key')
	var canvas_hash_value: Variant = canvas.get('scene_hash')
	var canvas_camera_value: Variant = canvas.get('camera')
	var stored_viewport_value: Variant = canvas.get('viewport_rect')
	if not canvas_snapshot_value is Dictionary or not canvas_scene_value is Dictionary or typeof(canvas_key_value)!=TYPE_STRING or typeof(canvas_hash_value)!=TYPE_STRING or not stored_viewport_value is Rect2:
		return false
	if not _same_contract(canvas_snapshot_value,snapshot_value) or not _same_contract(canvas_scene_value,scene_value) or canvas_key_value!=current_key_value or canvas_hash_value!=SCENE_SHA256 or canvas_camera_value!=camera_value or stored_viewport_value!=viewport_value:
		return false
	var actual_viewport: Rect2 = canvas_value.get_viewport_rect()
	if actual_viewport!=viewport_value or viewport.position!=Vector2.ZERO or not is_finite(viewport.position.x) or not is_finite(viewport.position.y) or not is_finite(viewport.size.x) or not is_finite(viewport.size.y) or not is_finite(viewport.end.x) or not is_finite(viewport.end.y) or viewport.size.x<=0.0 or viewport.size.y<=0.0:
		return false
	var camera_viewport_value: Variant = camera_value.get('viewport')
	if not camera_viewport_value is Vector2 or not is_finite(camera_viewport_value.x) or not is_finite(camera_viewport_value.y) or camera_viewport_value!=viewport.size:
		return false
	return true

func _short_capture(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, camera_value: Variant, viewport_value: Variant) -> Dictionary:
	if not _short_bindings(canvas_value,snapshot_value,scene_value,current_key_value,camera_value,viewport_value):
		return {}
	return {'canvas':canvas_value,'snapshot':snapshot_value.duplicate(true),'scene':scene_value.duplicate(true),'current_key':current_key_value,'scene_hash':SCENE_SHA256,'camera':camera_value,'origin':camera_value.get('origin'),'center':camera_value.get('screen_center'),'ppm':camera_value.get('ppm'),'camera_viewport':camera_value.get('viewport'),'viewport':viewport_value}

func _short_unchanged(frame: Dictionary, canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if frame.is_empty() or not _short_bindings(canvas_value,snapshot_value,scene_value,current_key_value,camera_value,viewport_value):
		return false
	return frame.get('canvas')==canvas_value and _same_contract(frame.get('snapshot'),snapshot_value) and _same_contract(frame.get('scene'),scene_value) and frame.get('current_key')==current_key_value and frame.get('scene_hash')==SCENE_SHA256 and frame.get('camera')==camera_value and frame.get('origin')==camera_value.get('origin') and frame.get('center')==camera_value.get('screen_center') and frame.get('ppm')==camera_value.get('ppm') and frame.get('camera_viewport')==camera_value.get('viewport') and frame.get('viewport')==viewport_value

func _short_quad_valid(points: PackedVector2Array) -> bool:
	if points.size()!=4:
		return false
	for point: Vector2 in points:
		if not is_finite(point.x) or not is_finite(point.y):
			return false
	var origin: Vector2 = points[0]
	var area2: float = (points[1]-origin).cross(points[2]-origin)+(points[2]-origin).cross(points[3]-origin)
	if not is_finite(area2) or area2==0.0:
		return false
	for index: int in range(points.size()):
		var edge: Vector2 = points[(index+1)%4]-points[index]
		var next_edge: Vector2 = points[(index+2)%4]-points[(index+1)%4]
		var turn: float = edge.cross(next_edge)
		if not is_finite(edge.x) or not is_finite(edge.y) or edge==Vector2.ZERO or not is_finite(turn) or turn==0.0 or (turn>0.0)!=(area2>0.0):
			return false
	var bounds: Rect2 = Rect2(points[0],Vector2.ZERO)
	for point: Vector2 in points:
		bounds = bounds.expand(point)
	return is_finite(bounds.position.x) and is_finite(bounds.position.y) and is_finite(bounds.size.x) and is_finite(bounds.size.y) and is_finite(bounds.end.x) and is_finite(bounds.end.y) and bounds.size.x>0.0 and bounds.size.y>0.0

func _short_sat(points: PackedVector2Array, viewport: Rect2) -> int:
	if points.size()!=4 or viewport.size.x<=0.0 or viewport.size.y<=0.0:
		return -1
	var axes: Array[Vector2] = [Vector2.RIGHT,Vector2.DOWN]
	for index: int in range(4):
		var edge: Vector2 = points[(index+1)%4]-points[index]
		axes.append(Vector2(-edge.y,edge.x))
	var corners: PackedVector2Array = PackedVector2Array([viewport.position,Vector2(viewport.end.x,viewport.position.y),viewport.end,Vector2(viewport.position.x,viewport.end.y)])
	var origin: Vector2 = points[0]
	var separated: bool = false
	for axis: Vector2 in axes:
		if not is_finite(axis.x) or not is_finite(axis.y) or axis==Vector2.ZERO:
			return -1
		var polygon_min: float = INF
		var polygon_max: float = -INF
		var rect_min: float = INF
		var rect_max: float = -INF
		for point: Vector2 in points:
			var projected: float = (point-origin).dot(axis)
			if not is_finite(projected):
				return -1
			polygon_min = minf(polygon_min,projected)
			polygon_max = maxf(polygon_max,projected)
		for corner: Vector2 in corners:
			var projected: float = (corner-origin).dot(axis)
			if not is_finite(projected):
				return -1
			rect_min = minf(rect_min,projected)
			rect_max = maxf(rect_max,projected)
		if not is_finite(polygon_min) or not is_finite(polygon_max) or not is_finite(rect_min) or not is_finite(rect_max) or polygon_min>=polygon_max or rect_min>=rect_max:
			return -1
		if polygon_max<=rect_min or rect_max<=polygon_min:
			separated = true
	return 0 if separated else 1

func _short_plan(canvas_value: Variant, wall_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, camera_value: Variant, viewport_value: Variant) -> Dictionary:
	var capture: Dictionary = _short_capture(canvas_value,snapshot_value,scene_value,current_key_value,camera_value,viewport_value)
	if capture.is_empty() or not wall_value is Dictionary:
		return {}
	var wall_id_value: Variant = wall_value.get('id')
	var collision: Variant = scene_value.get('collision')
	if typeof(wall_id_value)!=TYPE_STRING or not collision is Array:
		return {}
	var wall_matches: int = 0
	for row: Variant in collision:
		if row is Dictionary and row.get('id')==wall_id_value:
			wall_matches += 1
			if not _same_contract(row,wall_value):
				return {}
	if wall_matches!=1:
		return {}
	var plans: Array[Dictionary] = []
	for spec: Variant in SHORT_UNION_FACES:
		if not spec is Dictionary or typeof(spec.get('owner'))!=TYPE_STRING or typeof(spec.get('low_owner'))!=TYPE_STRING:
			return {}
		var owner: String = spec.owner
		if not WALLS.has(owner):
			return {}
		var info: Dictionary = WALLS[owner]
		var polygon: Variant = info.get('polygon')
		var edge_value: Variant = spec.edge
		if not polygon is Array or polygon.size()!=4 or not edge_value is Array or edge_value.size()!=2 or typeof(edge_value[0])!=TYPE_INT or typeof(edge_value[1])!=TYPE_INT:
			return {}
		var owner_rows: int = 0
		for row: Variant in collision:
			if row is Dictionary and row.get('id')==owner:
				owner_rows += 1
				if not _same_contract(row.get('polygon'),info.polygon):
					return {}
		if owner_rows!=1:
			return {}
		var ia: int = edge_value[0]
		var ib: int = edge_value[1]
		if ia<0 or ia>=4 or ib<0 or ib>=4 or ia==ib:
			return {}
		var a_value: Variant = polygon[ia]
		var b_value: Variant = polygon[ib]
		if not a_value is Array or a_value.size()!=2 or not b_value is Array or b_value.size()!=2 or not P.number(a_value[0],0.0,24.0) or not P.number(a_value[1],0.0,16.0) or not P.number(b_value[0],0.0,24.0) or not P.number(b_value[1],0.0,16.0):
			return {}
		var low: float = 0.0
		var low_owner: String = spec.low_owner
		if not low_owner.is_empty():
			if not WALLS.has(low_owner) or not P.number(WALLS[low_owner].get('height')):
				return {}
			low = float(WALLS[low_owner].height)
		var high_value: Variant = info.get('height')
		if not P.number(high_value) or float(high_value)<=low:
			return {}
		var high: float = float(high_value)
		var world: Array[Vector3] = [Vector3(float(a_value[0]),low,float(a_value[1])),Vector3(float(b_value[0]),low,float(b_value[1])),Vector3(float(b_value[0]),high,float(b_value[1])),Vector3(float(a_value[0]),high,float(a_value[1]))]
		var points: PackedVector2Array = PackedVector2Array()
		for point: Vector3 in world:
			var projected: Vector2 = _project(camera_value,point)
			if not is_finite(projected.x) or not is_finite(projected.y):
				return {}
			points.append(projected)
		if not _short_quad_valid(points):
			return {}
		plans.append({'id':spec.get('id'),'owner':owner,'points':points})
	if plans.size()!=4:
		return {}
	for index: int in range(plans.size()):
		var visibility: int = _short_sat(plans[index].points,viewport_value)
		if visibility<0:
			return {}
		plans[index]['visible'] = visibility>0
	if not _short_unchanged(capture,canvas_value,snapshot_value,scene_value,current_key_value,camera_value,viewport_value):
		return {}
	var faces: Array[PackedVector2Array] = []
	for plan: Dictionary in plans:
		if plan.owner==wall_id_value and plan.visible:
			faces.append(plan.points)
	if faces.is_empty():
		return {}
	return {'capture':capture,'owner':wall_id_value,'faces':faces,'faces_capture':faces.duplicate(true)}

func _short_emit(canvas_value: Variant, stage: Dictionary, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, camera_value: Variant, viewport_value: Variant) -> void:
	var capture: Variant = stage.get('capture')
	var faces: Variant = stage.get('faces')
	if stage.is_empty() or not capture is Dictionary or not faces is Array or faces.is_empty() or faces.size()>4:
		return
	for face: Variant in faces:
		if not face is PackedVector2Array or not _short_quad_valid(face):
			return
	if not _same_contract(faces,stage.get('faces_capture')) or not _short_unchanged(capture,canvas_value,snapshot_value,scene_value,current_key_value,camera_value,viewport_value):
		return
	var canvas: CanvasItem = canvas_value
	for face: PackedVector2Array in faces:
		canvas.draw_colored_polygon(face,Color(0.18,0.22,0.24,1.0))
# END short union helper functions

func _project(camera: Object, point: Vector3) -> Vector2:
	var projected_value: Variant = camera.call('project',point)
	if not projected_value is Vector2 or not is_finite(projected_value.x) or not is_finite(projected_value.y):
		return Vector2(INF,INF)
	return projected_value

func _project_xz(camera: Object, point: Vector2, height: float) -> Vector2:
	return _project(camera,Vector3(point.x,height,point.y))

func draw_floor(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, current_key: String, camera: Object, viewport: Rect2, furnace_consumer: Object) -> bool:
	if not _draw_ready(canvas,snapshot,scene,current_key,camera,viewport) or stone_texture==null:
		return false
	var screen_quads: Array[PackedVector2Array] = []
	var uv_quads: Array[PackedVector2Array] = []
	for row in range(6):
		for column in range(8):
			var x0: float = float(column)*3.0
			var x1: float = x0+3.0
			var z0: float = float(row)*3.0
			var z1: float = minf(z0+3.0,16.0)
			var v1: float = (z1-z0)/3.0
			var world: Array[Vector3] = [Vector3(x0,0,z0),Vector3(x1,0,z0),Vector3(x1,0,z1),Vector3(x0,0,z1)]
			var projected: PackedVector2Array = PackedVector2Array()
			var uvs: PackedVector2Array = PackedVector2Array()
			for corner in range(4):
				var u: float = 1.0 if corner==1 or corner==2 else 0.0
				var v: float = v1 if corner==2 or corner==3 else 0.0
				var source_pixel: Vector2 = Vector2(510,200)+Vector2(460,260)*u+Vector2(-370,280)*v
				var screen_point: Vector2 = _project(camera,world[corner])
				if not is_finite(screen_point.x) or not is_finite(screen_point.y):
					return false
				projected.append(screen_point)
				uvs.append(Vector2(source_pixel.x/1197.0,source_pixel.y/1132.0))
			screen_quads.append(projected)
			uv_quads.append(uvs)
	for index in range(screen_quads.size()):
		canvas.draw_colored_polygon(screen_quads[index],Color(0.35,0.40,0.43,1.0),uv_quads[index],stone_texture)
	_draw_furnace_cue(canvas,snapshot,scene,current_key,camera,viewport,furnace_consumer)
	return true

func _draw_furnace_cue(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, current_key: String, camera: Object, viewport: Rect2, furnace_consumer: Object) -> void:
	if not _draw_ready(canvas,snapshot,scene,current_key,camera,viewport) or furnace_consumer==null or furnace_consumer.get_script()!=FurnaceScript or not furnace_consumer.has_method('canvas_rect'):
		return
	var texture_cache_value: Variant = furnace_consumer.get('textures')
	if not texture_cache_value is Dictionary or not texture_cache_value.has('furnace_v2'):
		return
	var cached_texture_value: Variant = texture_cache_value.get('furnace_v2')
	if not cached_texture_value is Texture2D or not is_instance_valid(cached_texture_value) or cached_texture_value.get_width()!=1165 or cached_texture_value.get_height()!=1240:
		return
	var ppm_value: Variant = camera.get('ppm')
	var foot: Vector2 = _project(camera,Vector3(19,0,6))
	if not is_finite(foot.x) or not is_finite(foot.y):
		return
	var rect_value: Variant = furnace_consumer.call('canvas_rect','furnace_v2',foot,float(ppm_value))
	if not rect_value is Rect2 or rect_value.size.x<=0 or rect_value.size.y<=0 or not is_finite(rect_value.position.x) or not is_finite(rect_value.position.y) or not is_finite(rect_value.end.x) or not is_finite(rect_value.end.y):
		return
	var radii: Array[Vector2] = [Vector2(1.25,0.55),Vector2(0.9,0.38),Vector2(0.45,0.2)]
	var alphas: Array[float] = [0.02,0.035,0.045]
	var rings: Array[PackedVector2Array] = []
	for ring_index in range(3):
		var ring: PackedVector2Array = PackedVector2Array()
		for vertex in range(24):
			var angle: float = TAU*float(vertex)/24.0
			var world_point: Vector3 = Vector3(19.0+cos(angle)*radii[ring_index].x,0,6.0+sin(angle)*radii[ring_index].y)
			var screen_point: Vector2 = _project(camera,world_point)
			if not is_finite(screen_point.x) or not is_finite(screen_point.y):
				return
			ring.append(screen_point)
		rings.append(ring)
	for ring_index in range(3):
		canvas.draw_colored_polygon(rings[ring_index],Color(0.62,0.42,0.25,alphas[ring_index]))

func draw_wall(canvas: CanvasItem, wall: Dictionary, snapshot: Dictionary, scene: Dictionary, current_key: String, camera: Object, viewport: Rect2) -> bool:
	if not _draw_ready(canvas,snapshot,scene,current_key,camera,viewport):
		return false
	var wall_id_value: Variant = wall.get('id')
	if typeof(wall_id_value)!=TYPE_STRING or not WALLS.has(wall_id_value):
		return false
	var info: Dictionary = WALLS[wall_id_value]
	if not _same_contract(wall.get('polygon'),info.polygon):
		return false
	var west_north_reveal_screen: PackedVector2Array = PackedVector2Array()
	if wall_id_value == 'cw_entry_foundry_wall_west_north':
		var canvas_snapshot_value: Variant = canvas.get('snapshot')
		var canvas_scene_value: Variant = canvas.get('scene')
		var canvas_key_value: Variant = canvas.get('current_key')
		if not canvas_snapshot_value is Dictionary or not canvas_scene_value is Dictionary or typeof(canvas_key_value)!=TYPE_STRING:
			return false
		var canvas_snapshot: Dictionary = canvas_snapshot_value
		var canvas_scene: Dictionary = canvas_scene_value
		var canvas_key: String = canvas_key_value
		if not _same_contract(canvas_snapshot,snapshot) or not _same_contract(canvas_scene,scene) or canvas_key!=current_key:
			return false
	var inner: Array = info.inner
	var contact_direction: Array = info.contact
	var a: Vector2 = Vector2(inner[0],inner[1])
	var b: Vector2 = Vector2(inner[2],inner[3])
	var offset: Vector2 = Vector2(contact_direction[0],contact_direction[1])*0.05
	var footprint_value: Array = info.polygon
	var ground_end: Vector2 = b
	var south_join: bool = wall_id_value in ['cw_entry_foundry_wall_west_south','cw_entry_foundry_wall_east_south']
	var south_height: float = float(WALLS['cw_entry_foundry_wall_south'].height)
	if south_join:
		ground_end.y -= south_height
	var contact_world: Array[Vector3] = [Vector3(a.x,0,a.y),Vector3(ground_end.x,0,ground_end.y),Vector3(ground_end.x+offset.x,0,ground_end.y+offset.y),Vector3(a.x+offset.x,0,a.y+offset.y)]
	var side_world: Array[Vector3] = [Vector3(a.x,0,a.y),Vector3(ground_end.x,0,ground_end.y)]
	if south_join and float(info.height) > south_height:
		# Retain the tall west face above the adjacent south cap.
		side_world.append(Vector3(b.x,south_height,b.y))
	side_world.append(Vector3(b.x,info.height,b.y))
	side_world.append(Vector3(a.x,info.height,a.y))
	var cap_world: Array[Vector3] = []
	for point_value in footprint_value:
		if not point_value is Array or point_value.size()!=2 or not P.number(point_value[0]) or not P.number(point_value[1]):
			return false
		cap_world.append(Vector3(float(point_value[0]),info.height,float(point_value[1])))
	var contact_screen: PackedVector2Array = PackedVector2Array()
	var side_screen: PackedVector2Array = PackedVector2Array()
	var cap_screen: PackedVector2Array = PackedVector2Array()
	for point_value in contact_world:
		var screen_point: Vector2 = _project(camera,point_value)
		if not is_finite(screen_point.x) or not is_finite(screen_point.y):
			return false
		contact_screen.append(screen_point)
	for point_value in side_world:
		var screen_point: Vector2 = _project(camera,point_value)
		if not is_finite(screen_point.x) or not is_finite(screen_point.y):
			return false
		side_screen.append(screen_point)
	for point_value in cap_world:
		var screen_point: Vector2 = _project(camera,point_value)
		if not is_finite(screen_point.x) or not is_finite(screen_point.y):
			return false
		cap_screen.append(screen_point)
	if wall_id_value == 'cw_entry_foundry_wall_west_north':
		for point_value in WEST_NORTH_REVEAL_WORLD:
			if not point_value is Vector3:
				return false
			var reveal_world_point: Vector3 = point_value
			var reveal_screen_point: Vector2 = _project(camera,reveal_world_point)
			if not is_finite(reveal_screen_point.x) or not is_finite(reveal_screen_point.y):
				return false
			west_north_reveal_screen.append(reveal_screen_point)
	# BEGIN short union plan injection
	var short_stage: Dictionary = _short_plan(canvas,wall,snapshot,scene,current_key,camera,viewport)
	# END short union plan injection
	canvas.draw_colored_polygon(contact_screen,Color(0.04,0.06,0.065,0.12))
	canvas.draw_colored_polygon(side_screen,Color(0.18,0.22,0.24,1.0))
	if wall_id_value == 'cw_entry_foundry_wall_west_north':
		canvas.draw_colored_polygon(west_north_reveal_screen,Color(0.18,0.22,0.24,1.0))
	# BEGIN short union emit injection
	_short_emit(canvas,short_stage,snapshot,scene,current_key,camera,viewport)
	# END short union emit injection
	canvas.draw_colored_polygon(cap_screen,Color(0.25,0.29,0.31,1.0))
	if wall_id_value=='cw_entry_foundry_wall_north':
		_draw_wall_chain(canvas,snapshot,scene,current_key,camera,viewport)
	return true

func _prepare_wall_chain_once() -> void:
	if wall_chain_preparation_attempted:
		return
	wall_chain_preparation_attempted = true
	var use_bytes: PackedByteArray = _read_text_bytes(WALL_CHAIN_USE_PATH,16384)
	if use_bytes.is_empty():
		return
	var parsed_use: Variant = JSON.parse_string(use_bytes.get_string_from_utf8())
	if not parsed_use is Dictionary or not _same_contract(parsed_use,WALL_CHAIN_EXPECTED_USE):
		return
	var png_bytes: PackedByteArray = _read_bytes(WALL_CHAIN_SOURCE_PATH,819043)
	if png_bytes.size()!=819043 or _sha256(png_bytes)!='001a331d8f9cd7caeba3432fc7325782f1b5a0230ff514b0f2b45aa72f6cfd7f':
		return
	var image: Image = Image.new()
	if image.load_png_from_buffer(png_bytes)!=OK or image.get_width()!=1983 or image.get_height()!=793 or image.get_format()!=Image.FORMAT_RGBA8 or image.has_mipmaps():
		return
	var texture: ImageTexture = ImageTexture.create_from_image(image)
	if texture==null or not is_instance_valid(texture) or texture.get_width()!=1983 or texture.get_height()!=793:
		return
	wall_chain_texture = texture

func _draw_wall_chain(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, current_key: String, camera: Object, viewport: Rect2) -> void:
	if not _draw_ready(canvas,snapshot,scene,current_key,camera,viewport) or wall_chain_texture==null or not is_instance_valid(wall_chain_texture) or wall_chain_texture.get_width()!=1983 or wall_chain_texture.get_height()!=793:
		return
	var north_count: int = 0
	for row in scene.get('collision',[]):
		if row is Dictionary and row.get('id')=='cw_entry_foundry_wall_north':
			if not _same_contract(row.get('polygon'),WALL_CHAIN_EXPECTED_USE.installation.wallPolygon):
				return
			north_count += 1
	if north_count!=1:
		return
	var world: Array[Vector3] = [Vector3(4,0.8,0.5),Vector3(5.5,0.8,0.5),Vector3(5.5,0.2,0.5),Vector3(4,0.2,0.5)]
	var uvs: PackedVector2Array = PackedVector2Array([Vector2(0,0),Vector2(1,0),Vector2(1,1),Vector2(0,1)])
	var points: PackedVector2Array = PackedVector2Array()
	for index in range(4):
		var point: Vector3 = world[index]
		var uv: Vector2 = uvs[index]
		if not is_finite(point.x) or not is_finite(point.y) or not is_finite(point.z) or not is_finite(uv.x) or not is_finite(uv.y) or uv.x<0.0 or uv.x>1.0 or uv.y<0.0 or uv.y>1.0:
			return
		var projected: Vector2 = _project(camera,point)
		if not is_finite(projected.x) or not is_finite(projected.y):
			return
		points.append(projected)
	var winding: float = 0.0
	var area: float = 0.0
	for index in range(4):
		var edge: Vector2 = points[(index+1)%4]-points[index]
		var next_edge: Vector2 = points[(index+2)%4]-points[(index+1)%4]
		var relative: Vector2 = points[index]-points[0]
		var next_relative: Vector2 = points[(index+1)%4]-points[0]
		if not is_finite(edge.x) or not is_finite(edge.y) or not is_finite(next_edge.x) or not is_finite(next_edge.y) or not is_finite(relative.x) or not is_finite(relative.y) or not is_finite(next_relative.x) or not is_finite(next_relative.y) or edge==Vector2.ZERO:
			return
		var turn: float = edge.cross(next_edge)
		var term: float = relative.cross(next_relative)
		if not is_finite(turn) or turn==0.0 or not is_finite(term):
			return
		if index==0:
			winding = turn
		elif (turn>0.0)!=(winding>0.0):
			return
		area += term
		if not is_finite(area):
			return
	if absf(area)<=0.000001:
		return
	canvas.draw_colored_polygon(points,Color.WHITE,uvs,wall_chain_texture)
