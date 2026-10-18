extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const V = preload('res://scripts/world/presentation.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')

const USE_MAP_PATH: String = 'res://data/cw_gear_shaft_environment.internal-use.json'
const USE_MAP_BYTES: int = 12054
const USE_MAP_SHA256: String = '940ea260d2210d8181a4de299db5c2f8774b5ab56c65697fa153117ff03fa916'
const SCENE_PATH: String = 'res://data/scenes/cw_gear_shaft.json'
const SCENE_BYTES: int = 8965
const SCENE_SHA256: String = '91afdb035c3df824806884c49d1e8f71034ff8f51b7bf90280bf71594896fea9'
const STEEL_PATH: String = 'res://assets/props/clockworks/pressure_hall_steel_floor_source.png'
const STEEL_BYTES: int = 1610704
const STEEL_SHA256: String = '0b4555021799ac849756e0f11c207d4e9309df19f4b8f68d0f4e8177f5204043'

const SOURCE_A: Vector2 = Vector2(510.0,200.0)
const SOURCE_B: Vector2 = Vector2(970.0,460.0)
const SOURCE_D: Vector2 = Vector2(140.0,480.0)
const STEEL_CANVAS: Vector2 = Vector2(1197.0,1132.0)
const FLOOR_TINT: Color = Color(0.50,0.53,0.55,1.0)
const SUPPORT_TINT: Color = Color(0.59,0.62,0.64,1.0)
const WALL_CONTACT: Color = Color(0.025,0.032,0.035,0.10)
const WALL_SIDE_DARK: Color = Color('#303a40')
const WALL_SIDE: Color = Color('#414d52')
const WALL_CAP: Color = Color('#5b686e')
const FASCIA_COLOR: Color = Color('#48565d')
const FASCIA_HIGHLIGHT: Color = Color(138.0/255.0,149.0/255.0,152.0/255.0,0.08)
const POSE_EPSILON: float = 0.0001
const FASCIA_HEIGHT: float = 0.12

const WALL_ROWS: Array = [
	{'id':'cw_gear_shaft_wall_north','polygon':[[0,0],[24,0],[24,0.5],[0,0.5]],'displayHeightM':0.65},
	{'id':'cw_gear_shaft_wall_south','polygon':[[0,15.5],[24,15.5],[24,16],[0,16]],'displayHeightM':0.18},
	{'id':'cw_gear_shaft_wall_west_north','polygon':[[0,0.5],[0.5,0.5],[0.5,6],[0,6]],'displayHeightM':0.65},
	{'id':'cw_gear_shaft_wall_west_south','polygon':[[0,9.5],[0.5,9.5],[0.5,15.5],[0,15.5]],'displayHeightM':0.65},
	{'id':'cw_gear_shaft_wall_east_north','polygon':[[23.5,0.5],[24,0.5],[24,6],[23.5,6]],'displayHeightM':0.18},
	{'id':'cw_gear_shaft_wall_east_south','polygon':[[23.5,9.5],[24,9.5],[24,15.5],[23.5,15.5]],'displayHeightM':0.18}
]
const SPRITE_ROWS: Array = [
	{'id':'cw_gear_shaft_steel_floor','assetId':'runtime2d.world.clockworks.floor_steel.v1','layer':'visual.floor','position':[12,0,8]},
	{'id':'cw_gear_shaft_factory_tiles','assetId':'runtime2d.world.clockworks.factory_tiles.v1','layer':'visual.floor','position':[12,0,8]},
	{'id':'cw_gear_pressure_pipe_run','assetId':'runtime2d.world.clockworks.pressure_pipes.v1','layer':'visual.props_back','position':[11,0,4]},
	{'id':'cw_gear_factory_module','assetId':'runtime2d.world.clockworks.factory_tiles.v1','layer':'visual.props_back','position':[16,0,12]},
	{'id':'cw_gear_factory_module_side','assetId':'runtime2d.world.clockworks.factory_tiles.v1','layer':'visual.props_dynamic','position':[20,0,7]}
]
const STANDING_DECKS: Array = [
	{'id':'cw_gear_upper_dock','polygon':[[14,6],[18.5,6],[18.5,10],[14,10]],'heightM':2},
	{'id':'cw_gear_furnace_landing','polygon':[[19.5,6],[24,6],[24,10],[19.5,10]],'heightM':2}
]
const MOVING_SUPPORTS: Array = [
	{'id':'cw_gear_main_lift','polygon':[[10,6],[15,6],[15,10],[10,10]],'lowerM':0,'upperM':2,'travelMs':4000,'endpointHoldMs':2000}
]
const SUPPORT_RECTANGLES: Dictionary = {
	'cw_gear_upper_dock': {'x0':14.0,'z0':6.0,'x1':18.5,'z1':10.0},
	'cw_gear_furnace_landing': {'x0':19.5,'z0':6.0,'x1':24.0,'z1':10.0},
	'cw_gear_main_lift': {'x0':10.0,'z0':6.0,'x1':15.0,'z1':10.0}
}
const SUPPORT_ID_ORDER: Array[String] = ['cw_gear_furnace_landing','cw_gear_main_lift','cw_gear_upper_dock']
const GUARD_ROWS: Array = [
	{'id':'cw_gear_shaft_forged_guard_01','kind':'enemy','entityType':'enemy.clockworks.forged_guard','navNode':null,'position':[6.5,0,8]},
	{'id':'cw_gear_shaft_forged_guard_02','kind':'enemy','entityType':'enemy.clockworks.forged_guard','navNode':null,'position':[9.5,0,8]}
]

const EXPECTED_PIN_ROWS: Array = [
	['apps/godot_playable/scripts/world/renderer.gd',123213,'a9e3e8e0ab4e83a006640b568ca03940168d337d83bc574fefe761a3554c90fa'],
	['apps/godot_playable/data/scenes/cw_gear_shaft.json',8965,'91afdb035c3df824806884c49d1e8f71034ff8f51b7bf90280bf71594896fea9'],
	['content/scenes/compiled/cw_gear_shaft.json',8965,'91afdb035c3df824806884c49d1e8f71034ff8f51b7bf90280bf71594896fea9'],
	['apps/godot_playable/scripts/world/projection.gd',3301,'45f518f6dddebaebedd6bc1b5eb66476ebd3f039779cb353cc6124fbc5c9c82e'],
	['apps/godot_playable/scripts/bridge/protocol.gd',7675,'9558f4192c4738b475c6c03046a1db1cd4074a8caf689008cccab013404b696b'],
	['apps/godot_playable/scripts/world/presentation.gd',7353,'659c9ccf61f4be94eaee10394e08b57accf7556f12f568809e8d61ae7a43e087'],
	['apps/godot_playable/scripts/world/mh_cw_actor_visual.gd',4632,'3e334dec418f1319881a82bb6024a06fb7b1d426b1000e25ecc61f9f7c7f7568'],
	['apps/godot_playable/data/mh_cw_actor.internal-use.json',3028,'7a88eece1864b352e2934163546a2303bd3a46457c167a00a5398f657c49590f'],
	['apps/godot_playable/scripts/world/cenyao_idle_visual.gd',3875,'52ead3447f264a4a6328b82142345f3fd3566599b95f4dadc15fa80d8a217705'],
	['apps/godot_playable/data/cenyao_idle.internal-use.json',3925,'b063efd36046834c41ebed05c12ffb8729926f7d4edff0e15d36dccdbb1da881'],
	['apps/godot_playable/data/cw_pressure_floor_material.internal-use.json',1391,'746dbb35f40af89c300d1043322f634663a1f58bb6130a8711a84c972b35015a'],
	['apps/godot_playable/scripts/world/cw_pressure_floor_material_visual.gd',18553,'76a0b7946d8b0cb68eb06554db330a3227a8c42eeec4525f2a8194f539e98e4d'],
	['apps/godot_playable/scripts/world/cw_forged_guard_arena_environment_visual.gd',18813,'3e6d99c67b4aec9a01ebc69046aad37ad35fec966c2b0133bc94287802687e30'],
	['apps/godot_playable/data/cw_forged_guard_arena_environment.internal-use.json',8027,'ea25389d03d76821e639a98b3e4731f9a3d812a4d2ffe9b67f74b617ecb5ea38'],
	['apps/godot_playable/project.godot',556,'9f6c26d26037b0f61bc07f36db40f451f544e7f5542f49ca6d53055f5b5758a3'],
	['server-rs/src/moving_support.rs',13117,'4471fade4c5eded7e3ccf2b06e97a02a950d880df2f91858d9807c7622f24b7b'],
	['server-rs/src/formal_runtime.rs',472375,'ed60b246d8a338d60a4603fe621e3e73bf813869600b7dc4fe8454dd6b3080d4'],
	['server-rs/tests/support_projection.rs',10067,'b9b6cfb302e45a9b28f7b977aaf1959751e55b362138b5aad8c48c1c87bf03fc']
]

const EXPECTED_USE_FIELDS: Dictionary = {
	'schema':'wuxian.cw-gear-shaft-environment.internal-use/v1',
	'registeredSourceReuse':true,
	'approval':'INTERNAL_REGISTERED_SOURCE_REUSE',
	'roomUse':'ONLY_EXPLICIT_MAIN_ADMITTED_CW_GEAR_SHAFT_SCOPE',
	'GodotDisplay':'NOT_RUN',
	'publicRelease':'NOT_ESTABLISHED',
	'worldId':'clockworks',
	'sceneId':'cw_gear_shaft',
	'upstreamUseMaps':{
		'steel':{'path':'res://data/cw_pressure_floor_material.internal-use.json','bytes':1391,'sha256':'746dbb35f40af89c300d1043322f634663a1f58bb6130a8711a84c972b35015a','approval':'INTERNAL_REGISTERED_SOURCE_REUSE'},
		'actors':{'path':'res://data/mh_cw_actor.internal-use.json','bytes':3028,'sha256':'7a88eece1864b352e2934163546a2303bd3a46457c167a00a5398f657c49590f','approval':'INTERNAL_REGISTERED_SOURCE_REUSE'},
		'cenyao':{'path':'res://data/cenyao_idle.internal-use.json','bytes':3925,'sha256':'b063efd36046834c41ebed05c12ffb8729926f7d4edff0e15d36dccdbb1da881','approval':'INTERNAL_REGISTERED_SOURCE_REUSE'}
	},
	'scene':{'path':'res://data/scenes/cw_gear_shaft.json','bytes':8965,'sha256':'91afdb035c3df824806884c49d1e8f71034ff8f51b7bf90280bf71594896fea9','worldId':'clockworks','sceneId':'cw_gear_shaft','schemaVersion':1,'boundsM':[0,0,24,16]},
	'resources':{
		'steel':{'assetId':'runtime2d.world.clockworks.floor_steel.v1','path':STEEL_PATH,'bytes':STEEL_BYTES,'sha256':STEEL_SHA256,'sourceSha256':'1a01e851654d9f5c98b5486cbe489f357ed99c55ffc601c492af63044d90b190','canvas':[1197,1132],'format':'RGBA8'},
		'forgedGuard':{'assetId':'runtime2d.enemy.clockworks.forged_guard.v2','path':'res://assets/enemies/clockworks/forged_guard.png','bytes':1300721,'sha256':'b41cdc83c587805d0beee9d48668a9328001882595e0ab97fa69347d9bcf8cb4','canvas':[1226,1254],'format':'RGBA8','anchor':[0.5,1],'visualHeightM':1.72,'sourceSha256':'5c542927516e2e393eccada493e5ca4e20d4bc5d75db8431d57f306910c5d0c0','sourceRuntimeStatus':'ADOPT','sourceAdmission':'release_approved'},
		'drownedIntegrityOnly':{'path':'res://assets/enemies/mist_harbor/drowned.png','bytes':1128162,'sha256':'752ac830587fc957d5695cd7714c57e584f40ec1a8f511f324ee491adaee7f2d','readPurpose':'existing MH_CW loader integrity; NOT displayed'},
		'signalWraithIntegrityOnly':{'path':'res://assets/enemies/mist_harbor/signal_wraith.png','bytes':1440594,'sha256':'942a294e86076bfa91762113cd9bb14c81db329baabf5804ba9138942cc7166b','readPurpose':'existing MH_CW loader integrity; NOT displayed'},
		'cenyaoIdle':{'canvas':[336,560],'footAnchor':[168,551],'visualHeightPxAt48ppm':104,'frames':[
			{'file':'cenyao_idle_south_west.png','bytes':158163,'sha256':'3d3e34d0a5d388de522fd6a94fd2769efbf8137aea1ae7c0b69881c5416520be'},
			{'file':'cenyao_idle_east.png','bytes':109177,'sha256':'fd38db1daec829c0a7c2adffcdfd5c605f54b3611bb515599cd8deeea1f47402'},
			{'file':'cenyao_idle_west.png','bytes':121863,'sha256':'fc8e0b132c48f594a49a20bd39955d731f02077502e33131499608cc3bd7a70c'},
			{'file':'cenyao_idle_south.png','bytes':158435,'sha256':'b6cff22dabb303f0a9c95ba24a41fb3c2f5159ff191adefacf0c6dad38ffe875'},
			{'file':'cenyao_idle_north_east.png','bytes':144761,'sha256':'19b8e318bf5216d6075f64ac986ef7c3c3299241816b472843aac24bbf4a7676'},
			{'file':'cenyao_idle_south_east.png','bytes':158547,'sha256':'bfe0107fe1efdcd73b90d57b01283a22658960b5847bb5e88b401dae9857be56'},
			{'file':'cenyao_idle_north.png','bytes':151244,'sha256':'6608ed34685f28f3380849d7fe5e5319078e8feb1fbd66cbcd4cb3782a1a8221'},
			{'file':'cenyao_idle_north_west.png','bytes':143105,'sha256':'8dc527b975b1da5ed7b36a54cf4ad260cdcd9d7d3ccba24660308cf971c3164e'}
		]}
	},
	'presentationRows':SPRITE_ROWS,
	'floor':{'rowId':'cw_gear_shaft_steel_floor','groundY':0,'sourcePixelEdgeCorners':[[510,200],[970,460],[600,740],[140,480]],'uvConvention':'affine source pixel edge / standalone canvas; no half-pixel offset','groundPatchM':[4,4],'groundGrid':[6,4],'groundPatchCount':24,'tint':[0.50,0.53,0.55,1],'normalAlpha':true,'linearFilter':true,'repeat':false,'metricSourceMeasurement':false},
	'walls':WALL_ROWS,
	'wallsStyle':{'backface':'#303a40','side':'#414d52','cap':'#5b686e','contactRGBA':[0.025,0.032,0.035,0.10],'contactInsideCanonicalFootprint':true,'newCollisionOrOccluder':false},
	'supports':{
		'standingDecks':STANDING_DECKS,
		'movingSupports':MOVING_SUPPORTS,
		'authoritativeInput':'snapshot.supportScene validated by existing Presentation.support',
		'poseSource':'existing native cw_gear_main_lift pose_at; no new time or interpolation',
		'poseCycleMs':12000,
		'standingPose':{'phase':'stationary','heightM':2,'velocityMps':0,'phaseElapsedMs':0},
		'globalTopGridM':[2,2],
		'globalGridAnchorM':[0,0],
		'maxTexturedPieces':18,
		'topTint':[0.59,0.62,0.64,1],
		'normalAlpha':true,
		'drawOrder':['cw_gear_main_lift','cw_gear_furnace_landing','cw_gear_upper_dock'],
		'fascia':{'edges':['east','south'],'heightM':0.12,'bottomY':'max(0,currentTopY-0.12)','color':'#48565d','highlight':'#8a9598','highlightMaxAlpha':0.12,'highlightMaxWidthScale':0.65,'insideFootprint':true,'collisionOrOccluder':false,'upperGapWidthM':1}
	},
	'photoRows':[
		{'id':'cw_gear_shaft_forged_guard_01','entityType':'runtime2d.enemy.clockworks.forged_guard.v2','authoringType':'enemy.clockworks.forged_guard','position':[6.5,0,8]},
		{'id':'cw_gear_shaft_forged_guard_02','entityType':'runtime2d.enemy.clockworks.forged_guard.v2','authoringType':'enemy.clockworks.forged_guard','position':[9.5,0,8]}
	],
	'excludedBehavior':['lift operation','native ride or traversal proof','mobility.air_step_i','first-kill or progression','save/Continue','pressure drones not displayed','factory compound as floor/support/wall','pressure pipe compound as registered support','any furnace/fire/gear-machine source'],
	'protected':['all scene copies','support/collision/navigation/checkpoint/traversal geometry','actor roster/body selectors/culling/sorting/labels','existing fallback rendering','other room helpers and mappings','native code','Main/Panels/UI','project/import/cache/settings','all previous source and public/private lineage differences']
}

var _load_attempted: bool = false
var _load_ok: bool = false
var _ready_for_context: bool = false
var _bound_key: String = ''
var _bound_hash: String = ''
var _source_scene: Dictionary = {}
var _canonical_scene: Dictionary = {}
var _steel_texture: Texture2D
var last_prepare_error: String = ''
var last_floor_emitted: bool = false
var last_emitted_wall_ids: Array[String] = []
var last_emitted_support_ids: Array[String] = []

func _clear_binding() -> void:
	_ready_for_context = false
	_bound_key = ''
	_bound_hash = ''
	_canonical_scene.clear()
	last_floor_emitted = false
	last_emitted_wall_ids.clear()
	last_emitted_support_ids.clear()

func _read_bytes(path: String, maximum: int) -> PackedByteArray:
	var file: FileAccess = FileAccess.open(path,FileAccess.READ)
	if file == null:
		return PackedByteArray()
	var length: int = file.get_length()
	if length <= 0 or length > maximum:
		file.close()
		return PackedByteArray()
	var bytes: PackedByteArray = file.get_buffer(length)
	file.close()
	if bytes.size() != length:
		return PackedByteArray()
	return bytes

func _sha256(bytes: PackedByteArray) -> String:
	var hashing: HashingContext = HashingContext.new()
	if hashing.start(HashingContext.HASH_SHA256) != OK or hashing.update(bytes) != OK:
		return ''
	return hashing.finish().hex_encode()

func _same(left: Variant, right: Variant) -> bool:
	var left_type: int = typeof(left)
	var right_type: int = typeof(right)
	var left_numeric: bool = left_type == TYPE_INT or left_type == TYPE_FLOAT
	var right_numeric: bool = right_type == TYPE_INT or right_type == TYPE_FLOAT
	if left_numeric and right_numeric:
		return is_finite(float(left)) and is_finite(float(right)) and float(left) == float(right)
	if left_type != right_type:
		return false
	if left is Dictionary:
		var left_dict: Dictionary = left
		var right_dict: Dictionary = right
		if left_dict.size() != right_dict.size():
			return false
		for key_variant: Variant in left_dict.keys():
			if not right_dict.has(key_variant) or not _same(left_dict.get(key_variant),right_dict.get(key_variant)):
				return false
		return true
	if left is Array:
		var left_array: Array = left
		var right_array: Array = right
		if left_array.size() != right_array.size():
			return false
		for index: int in range(left_array.size()):
			if not _same(left_array[index],right_array[index]):
				return false
		return true
	return left == right

func _use_map_matches(parsed_value: Variant) -> bool:
	if not parsed_value is Dictionary:
		return false
	var parsed_map: Dictionary = parsed_value
	if parsed_map.size() != EXPECTED_USE_FIELDS.size()+1 or not parsed_map.has('sourcePins'):
		return false
	for expected_key_variant: Variant in EXPECTED_USE_FIELDS.keys():
		if not parsed_map.has(expected_key_variant):
			return false
		if not _same(parsed_map.get(expected_key_variant),EXPECTED_USE_FIELDS.get(expected_key_variant)):
			return false
	var pins_value: Variant = parsed_map.get('sourcePins')
	if not pins_value is Array:
		return false
	var pins: Array = pins_value
	if pins.size() != EXPECTED_PIN_ROWS.size():
		return false
	for index: int in range(EXPECTED_PIN_ROWS.size()):
		var row_value: Variant = pins[index]
		var pin_expected: Array = EXPECTED_PIN_ROWS[index]
		if not row_value is Dictionary or not pin_expected is Array:
			return false
		var row: Dictionary = row_value
		if row.size() != 4 or typeof(row.get('path')) != TYPE_STRING or row.get('path') != pin_expected[0]:
			return false
		if not P.number(row.get('bytes'),float(pin_expected[1]),float(pin_expected[1])) or row.get('sha256') != pin_expected[2] or typeof(row.get('role')) != TYPE_STRING or row.get('role').is_empty():
			return false
	return true

func _finite_vector2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _finite_vector3(value: Vector3) -> bool:
	return is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _canonical_source_scene(source_value: Variant) -> bool:
	if not source_value is Dictionary:
		return false
	var source_scene: Dictionary = source_value
	if source_scene.get('worldId') != 'clockworks' or source_scene.get('sceneId') != 'cw_gear_shaft' or not P.counter(source_scene.get('schemaVersion'),1,1):
		return false
	var bounds_value: Variant = source_scene.get('boundsM')
	if not bounds_value is Dictionary or not _same(bounds_value,{'x':0,'z':0,'width':24,'depth':16}):
		return false
	var collision_value: Variant = source_scene.get('collision')
	if not collision_value is Array:
		return false
	var collision: Array = collision_value
	if collision.size() != WALL_ROWS.size() or not P.unique(collision,'id'):
		return false
	var seen_walls: Dictionary = {}
	for wall_value: Variant in collision:
		if not wall_value is Dictionary:
			return false
		var wall: Dictionary = wall_value
		var wall_id_value: Variant = wall.get('id')
		if typeof(wall_id_value) != TYPE_STRING or seen_walls.has(wall_id_value):
			return false
		seen_walls[wall_id_value] = true
		var matched_wall: bool = false
		for expected_wall_value: Variant in WALL_ROWS:
			if expected_wall_value is Dictionary and wall.get('id') == expected_wall_value.get('id'):
				matched_wall = _wall_geometry_matches(wall,expected_wall_value)
				break
		if not matched_wall:
			return false
	var standing_value: Variant = source_scene.get('standingDecks')
	var moving_value: Variant = source_scene.get('movingSupports')
	if not standing_value is Array or not moving_value is Array:
		return false
	if MOVING_SUPPORTS.size() != 1 or not _same(standing_value,STANDING_DECKS) or not _same(moving_value,MOVING_SUPPORTS):
		return false
	var presentation_value: Variant = source_scene.get('presentation')
	if not presentation_value is Dictionary:
		return false
	var presentation: Dictionary = presentation_value
	var sprites_value: Variant = presentation.get('sprites')
	if not sprites_value is Array or not _same(sprites_value,SPRITE_ROWS):
		return false
	var spawns_value: Variant = source_scene.get('spawns')
	if not spawns_value is Array:
		return false
	var spawns: Array = spawns_value
	if spawns.size() != 6 or not P.unique(spawns,'id'):
		return false
	var guard_count: int = 0
	for spawn_value: Variant in spawns:
		if not spawn_value is Dictionary:
			return false
		var spawn: Dictionary = spawn_value
		if spawn.get('id') == GUARD_ROWS[0].id or spawn.get('id') == GUARD_ROWS[1].id:
			var matched_guard: bool = false
			for expected_guard_value: Variant in GUARD_ROWS:
				if expected_guard_value is Dictionary and spawn.get('id') == expected_guard_value.get('id'):
					matched_guard = _same(spawn,expected_guard_value)
					break
			if not matched_guard:
				return false
			guard_count += 1
	return guard_count == 2

func _basic(snapshot_value: Variant, scene_value: Variant, key_value: Variant, hash_value: Variant) -> bool:
	if not snapshot_value is Dictionary or not scene_value is Dictionary or not P.snapshot(snapshot_value):
		return false
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	if snapshot.get('entryToken') != null or snapshot.get('worldId') != 'clockworks' or snapshot.get('sceneId') != 'cw_gear_shaft' or not P.counter(snapshot.get('worldEpoch'),1):
		return false
	if typeof(key_value) != TYPE_STRING or key_value != 'clockworks:cw_gear_shaft:'+str(snapshot.get('worldEpoch')):
		return false
	if typeof(hash_value) != TYPE_STRING or hash_value != SCENE_SHA256:
		return false
	return scene.get('worldId') == 'clockworks' and scene.get('sceneId') == 'cw_gear_shaft'

func prepare(snapshot_value: Variant, scene_value: Variant, key_value: Variant, hash_value: Variant) -> bool:
	_clear_binding()
	last_prepare_error = ''
	if not _basic(snapshot_value,scene_value,key_value,hash_value):
		last_prepare_error = 'context rejected before resource access'
		return false
	if _load_attempted:
		if not _load_ok:
			last_prepare_error = 'first admitted resource preparation failed; no retry'
			return false
		if not _same(scene_value,_source_scene):
			last_prepare_error = 'current scene differs from pinned canonical source'
			return false
		_canonical_scene = _source_scene.duplicate(true)
		_ready_for_context = true
		_bound_key = str(key_value)
		_bound_hash = str(hash_value)
		return true
	_load_attempted = true
	var map_bytes: PackedByteArray = _read_bytes(USE_MAP_PATH,32768)
	if map_bytes.size() != USE_MAP_BYTES or _sha256(map_bytes) != USE_MAP_SHA256 or map_bytes.get_string_from_utf8().to_utf8_buffer() != map_bytes:
		last_prepare_error = 'new room-use map byte/hash/UTF-8 pin failed'
		return false
	var parsed_map_value: Variant = JSON.parse_string(map_bytes.get_string_from_utf8())
	if not _use_map_matches(parsed_map_value):
		last_prepare_error = 'new room-use map literal/keyset mismatch'
		return false
	var scene_bytes: PackedByteArray = _read_bytes(SCENE_PATH,16384)
	if scene_bytes.size() != SCENE_BYTES or _sha256(scene_bytes) != SCENE_SHA256 or scene_bytes.get_string_from_utf8().to_utf8_buffer() != scene_bytes:
		last_prepare_error = 'canonical scene byte/hash/UTF-8 pin failed'
		return false
	var decoded_scene_value: Variant = JSON.parse_string(scene_bytes.get_string_from_utf8())
	if not _canonical_source_scene(decoded_scene_value) or not _same(scene_value,decoded_scene_value):
		last_prepare_error = 'current scene is not the exact parsed canonical source'
		return false
	var steel_bytes: PackedByteArray = _read_bytes(STEEL_PATH,4*1024*1024)
	if steel_bytes.size() != STEEL_BYTES or _sha256(steel_bytes) != STEEL_SHA256:
		last_prepare_error = 'installed steel PNG byte/hash pin failed'
		return false
	var image: Image = Image.new()
	if image.load_png_from_buffer(steel_bytes) != OK or image.get_width() != 1197 or image.get_height() != 1132 or image.get_format() != Image.FORMAT_RGBA8:
		last_prepare_error = 'installed steel PNG decode/canvas/format pin failed'
		return false
	var created_texture: ImageTexture = ImageTexture.create_from_image(image)
	if created_texture == null or created_texture.get_width() != 1197 or created_texture.get_height() != 1132:
		last_prepare_error = 'steel texture creation/canvas failed'
		return false
	_steel_texture = created_texture
	_source_scene = decoded_scene_value.duplicate(true)
	_canonical_scene = _source_scene.duplicate(true)
	_load_ok = true
	_ready_for_context = true
	_bound_key = str(key_value)
	_bound_hash = str(hash_value)
	return true

func _camera_valid(camera_value: Variant, viewport_value: Variant) -> bool:
	if not camera_value is RefCounted or not is_instance_valid(camera_value) or camera_value.get_script() != ProjectionScript or not viewport_value is Rect2:
		return false
	var origin_value: Variant = camera_value.get('origin')
	var center_value: Variant = camera_value.get('screen_center')
	var size_value: Variant = camera_value.get('viewport')
	var initialized_value: Variant = camera_value.get('initialized')
	var ppm_value: Variant = camera_value.get('ppm')
	if not origin_value is Vector3 or not center_value is Vector2 or not size_value is Vector2 or typeof(initialized_value) != TYPE_BOOL or not initialized_value:
		return false
	if not P.number(ppm_value,0.000001,256.0) or not _finite_vector3(origin_value) or not _finite_vector2(center_value) or not _finite_vector2(size_value):
		return false
	var viewport: Rect2 = viewport_value
	return _finite_vector2(viewport.position) and _finite_vector2(viewport.size) and _finite_vector2(viewport.end) and viewport.size.x > 0.0 and viewport.size.y > 0.0 and size_value == viewport.size

func _draw_context(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, hash_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not _ready_for_context or not _load_ok or _steel_texture == null or not _basic(snapshot_value,scene_value,key_value,hash_value):
		return false
	if str(key_value) != _bound_key or str(hash_value) != _bound_hash or not _same(scene_value,_canonical_scene):
		return false
	if not canvas_value is CanvasItem or not is_instance_valid(canvas_value):
		return false
	var canvas: CanvasItem = canvas_value
	if not canvas.is_inside_tree() or canvas.is_queued_for_deletion() or canvas.get('scene_hash') != SCENE_SHA256 or canvas.get('current_key') != key_value:
		return false
	return _camera_valid(camera_value,viewport_value)

func _project(camera_value: Variant, world: Vector3) -> Vector2:
	var projected_value: Variant = camera_value.call('project',world)
	if not projected_value is Vector2 or not _finite_vector2(projected_value):
		return Vector2(INF,INF)
	return projected_value

func _project_polygon(camera_value: Variant, world: Array[Vector3]) -> PackedVector2Array:
	if world.size() < 3:
		return PackedVector2Array()
	var result: PackedVector2Array = PackedVector2Array()
	for point: Vector3 in world:
		if not _finite_vector3(point):
			return PackedVector2Array()
		var projected: Vector2 = _project(camera_value,point)
		if not _finite_vector2(projected):
			return PackedVector2Array()
		result.append(projected)
	var twice_area: float = 0.0
	for index: int in range(result.size()):
		twice_area += result[index].cross(result[(index+1)%result.size()])
	if not is_finite(twice_area) or absf(twice_area) <= 0.000001:
		return PackedVector2Array()
	return result

func _project_line(camera_value: Variant, world: Array[Vector3]) -> PackedVector2Array:
	if world.size() != 2:
		return PackedVector2Array()
	var result: PackedVector2Array = PackedVector2Array()
	for point: Vector3 in world:
		if not _finite_vector3(point):
			return PackedVector2Array()
		var projected: Vector2 = _project(camera_value,point)
		if not _finite_vector2(projected):
			return PackedVector2Array()
		result.append(projected)
	if result[0].distance_squared_to(result[1]) <= 0.000001:
		return PackedVector2Array()
	return result

func _uv(u: float, v: float) -> Vector2:
	var source_pixel: Vector2 = SOURCE_A+(SOURCE_B-SOURCE_A)*u+(SOURCE_D-SOURCE_A)*v
	return source_pixel/STEEL_CANVAS

func draw_floor(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, hash_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	last_floor_emitted = false
	if not _draw_context(canvas_value,snapshot_value,scene_value,key_value,hash_value,camera_value,viewport_value):
		return false
	var projected_tiles: Array[PackedVector2Array] = []
	var uv_tiles: Array[PackedVector2Array] = []
	for row: int in range(4):
		for column: int in range(6):
			var x0: float = float(column)*4.0
			var x1: float = x0+4.0
			var z0: float = float(row)*4.0
			var z1: float = z0+4.0
			var world: Array[Vector3] = [Vector3(x0,0.0,z0),Vector3(x1,0.0,z0),Vector3(x1,0.0,z1),Vector3(x0,0.0,z1)]
			var points: PackedVector2Array = _project_polygon(camera_value,world)
			if points.is_empty():
				return false
			var uvs: PackedVector2Array = PackedVector2Array([_uv(0.0,0.0),_uv(1.0,0.0),_uv(1.0,1.0),_uv(0.0,1.0)])
			for uv_point: Vector2 in uvs:
				if not _finite_vector2(uv_point):
					return false
			projected_tiles.append(points)
			uv_tiles.append(uvs)
	if projected_tiles.size() != 24 or uv_tiles.size() != 24:
		return false
	var canvas: CanvasItem = canvas_value
	for index: int in range(projected_tiles.size()):
		canvas.draw_polygon(projected_tiles[index],PackedColorArray([FLOOR_TINT,FLOOR_TINT,FLOOR_TINT,FLOOR_TINT]),uv_tiles[index],_steel_texture)
	last_floor_emitted = true
	return true

func _wall_row(id_value: Variant) -> Dictionary:
	if typeof(id_value) != TYPE_STRING:
		return {}
	for expected_value: Variant in WALL_ROWS:
		if expected_value is Dictionary and expected_value.get('id') == id_value:
			return expected_value
	return {}

func _wall_geometry_matches(source_wall: Dictionary, visual_wall: Dictionary) -> bool:
	return source_wall.size() == 2 and typeof(source_wall.get('id')) == TYPE_STRING and source_wall.get('id') == visual_wall.get('id') and _same(source_wall.get('polygon'),visual_wall.get('polygon'))

func draw_wall(canvas_value: Variant, wall_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, hash_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not _draw_context(canvas_value,snapshot_value,scene_value,key_value,hash_value,camera_value,viewport_value) or not wall_value is Dictionary:
		return false
	var wall: Dictionary = wall_value
	var expected_wall: Dictionary = _wall_row(wall.get('id'))
	if expected_wall.is_empty() or not _wall_geometry_matches(wall,expected_wall):
		return false
	var canonical_member: bool = false
	for row_value: Variant in _canonical_scene.get('collision',[]):
		if row_value is Dictionary and row_value.get('id') == wall.get('id') and _same(row_value,wall):
			canonical_member = true
			break
	if not canonical_member:
		return false
	var polygon_value: Variant = expected_wall.get('polygon')
	var height_value: Variant = expected_wall.get('displayHeightM')
	if not polygon_value is Array or not P.number(height_value,0.001,1.0):
		return false
	var polygon: Array = polygon_value
	if polygon.size() != 4:
		return false
	var base_world: Array[Vector3] = []
	var cap_world: Array[Vector3] = []
	for pair_value: Variant in polygon:
		if not pair_value is Array:
			return false
		var pair: Array = pair_value
		if pair.size() != 2 or not P.number(pair[0],0.0,24.0) or not P.number(pair[1],0.0,16.0):
			return false
		var x_value: float = float(pair[0])
		var z_value: float = float(pair[1])
		base_world.append(Vector3(x_value,0.0,z_value))
		cap_world.append(Vector3(x_value,float(height_value),z_value))
	var projected_plan: Array[PackedVector2Array] = []
	var color_plan: Array[Color] = []
	var contact_points: PackedVector2Array = _project_polygon(camera_value,base_world)
	var cap_points: PackedVector2Array = _project_polygon(camera_value,cap_world)
	if contact_points.is_empty() or cap_points.is_empty():
		return false
	projected_plan.append(contact_points)
	color_plan.append(WALL_CONTACT)
	var side_color: Color = WALL_SIDE_DARK if float(height_value) > 0.5 else WALL_SIDE
	for edge_index: int in range(4):
		var next_index: int = (edge_index+1)%4
		var start: Vector3 = base_world[edge_index]
		var finish: Vector3 = base_world[next_index]
		var side_world: Array[Vector3] = [start,finish,cap_world[next_index],cap_world[edge_index]]
		var side_points: PackedVector2Array = _project_polygon(camera_value,side_world)
		if side_points.is_empty():
			return false
		projected_plan.append(side_points)
		color_plan.append(side_color)
	projected_plan.append(cap_points)
	color_plan.append(WALL_CAP)
	if projected_plan.size() != 6 or color_plan.size() != 6:
		return false
	var canvas: CanvasItem = canvas_value
	for plan_index: int in range(projected_plan.size()):
		canvas.draw_colored_polygon(projected_plan[plan_index],color_plan[plan_index])
	var wall_id: String = str(wall.get('id'))
	if not last_emitted_wall_ids.has(wall_id):
		last_emitted_wall_ids.append(wall_id)
	return true

func _support_scene_valid(snapshot: Dictionary, poses_value: Variant) -> bool:
	var support_scene_value: Variant = snapshot.get('supportScene')
	if not support_scene_value is Dictionary or not poses_value is Dictionary:
		return false
	var support_scene: Dictionary = support_scene_value
	if support_scene.size() != 10 or support_scene.get('schemaVersion') != 1 or support_scene.get('worldId') != 'clockworks' or support_scene.get('sceneId') != 'cw_gear_shaft':
		return false
	if not P.counter(support_scene.get('worldEpoch'),1) or support_scene.get('worldEpoch') != snapshot.get('worldEpoch') or not P.counter(support_scene.get('serverTick')) or support_scene.get('serverTick') != snapshot.get('serverTick') or not P.counter(support_scene.get('authorityRevision')) or support_scene.get('authorityRevision') != snapshot.get('authorityRevision'):
		return false
	if support_scene.get('sceneSourceSha256') != SCENE_SHA256 or not P.counter(support_scene.get('serverTimeMs')):
		return false
	var source_poses_value: Variant = support_scene.get('poses')
	if not source_poses_value is Array:
		return false
	var source_poses: Array = source_poses_value
	if source_poses.size() != 3 or poses_value.size() != 3:
		return false
	var previous_id: String = ''
	var seen: Dictionary = {}
	for index: int in range(source_poses.size()):
		var pose_value: Variant = source_poses[index]
		if not pose_value is Dictionary:
			return false
		var pose: Dictionary = pose_value
		if pose.size() != 5 or typeof(pose.get('supportId')) != TYPE_STRING or pose.get('supportId') <= previous_id or not SUPPORT_RECTANGLES.has(pose.get('supportId')):
			return false
		var support_id: String = str(pose.get('supportId'))
		if seen.has(support_id) or not poses_value.has(support_id) or not P.number(pose.get('heightM'),0.0,2.0) or not P.number(pose.get('velocityMps'),-0.5,0.5) or not P.counter(pose.get('phaseElapsedMs'),0,12000):
			return false
		var validator_pose_value: Variant = poses_value.get(support_id)
		if not validator_pose_value is Dictionary or not _same(pose,validator_pose_value):
			return false
		seen[support_id] = true
		previous_id = support_id
	var rider_value: Variant = support_scene.get('rider')
	if not rider_value is Dictionary:
		return false
	var rider: Dictionary = rider_value
	if rider.size() != 4 or not rider.has('supportId') or not P.point(rider.get('positionM')) or not P.number(rider.get('radiusM'),0.001,4.0) or not rider.get('mode') in ['surface','floor','airborne','traversal']:
		return false
	if P.vector(rider.get('positionM')).distance_to(P.vector(snapshot.player.transform.positionM)) > POSE_EPSILON:
		return false
	if rider.get('mode') == 'surface':
		var rider_support: Variant = rider.get('supportId')
		if typeof(rider_support) != TYPE_STRING or not poses_value.has(rider_support):
			return false
		var rider_pose_value: Variant = poses_value.get(rider_support)
		if not rider_pose_value is Dictionary or absf(float(rider.positionM.yM)-float(rider_pose_value.heightM)) > POSE_EPSILON:
			return false
	else:
		if rider.get('supportId') != null or rider.get('mode') == 'floor' and absf(float(rider.positionM.yM)) > POSE_EPSILON:
			return false
	return true

func _pose_values_match(pose_value: Variant, support_id: String, support_scene: Dictionary) -> bool:
	if not pose_value is Dictionary:
		return false
	var pose: Dictionary = pose_value
	if pose.size() != 5 or pose.get('supportId') != support_id:
		return false
	var phase_value: Variant = pose.get('phase')
	var height_value: Variant = pose.get('heightM')
	var velocity_value: Variant = pose.get('velocityMps')
	var elapsed_value: Variant = pose.get('phaseElapsedMs')
	if typeof(phase_value) != TYPE_STRING or not P.number(height_value,0.0,2.0) or not P.number(velocity_value,-0.5,0.5) or not P.counter(elapsed_value,0,6000):
		return false
	if support_id != 'cw_gear_main_lift':
		return phase_value == 'stationary' and absf(float(height_value)-2.0) <= POSE_EPSILON and float(velocity_value) == 0.0 and int(elapsed_value) == 0
	var server_time_value: Variant = support_scene.get('serverTimeMs')
	if not P.counter(server_time_value):
		return false
	var phase_time: int = int(server_time_value)%12000
	var expected_phase: String = ''
	var expected_height: float = 0.0
	var expected_velocity: float = 0.0
	var expected_elapsed: int = 0
	if phase_time < 2000:
		expected_phase = 'lower_hold'
		expected_height = 0.0
		expected_velocity = 0.0
		expected_elapsed = phase_time
	elif phase_time < 6000:
		expected_phase = 'rising'
		expected_height = 2.0*float(phase_time-2000)/4000.0
		expected_velocity = 0.5
		expected_elapsed = phase_time-2000
	elif phase_time < 8000:
		expected_phase = 'upper_hold'
		expected_height = 2.0
		expected_velocity = 0.0
		expected_elapsed = phase_time-6000
	else:
		expected_phase = 'falling'
		expected_height = 2.0-2.0*float(phase_time-8000)/4000.0
		expected_velocity = -0.5
		expected_elapsed = phase_time-8000
	return phase_value == expected_phase and absf(float(height_value)-expected_height) <= POSE_EPSILON and absf(float(velocity_value)-expected_velocity) <= POSE_EPSILON and int(elapsed_value) == expected_elapsed

func _validated_poses(snapshot_value: Variant, scene_value: Variant, key_value: Variant, hash_value: Variant) -> Dictionary:
	if not _basic(snapshot_value,scene_value,key_value,hash_value) or not _ready_for_context or not _same(scene_value,_canonical_scene):
		return {}
	var snapshot: Dictionary = snapshot_value
	var support_scene_value: Variant = snapshot.get('supportScene')
	if not support_scene_value is Dictionary:
		return {}
	var support_scene: Dictionary = support_scene_value
	var poses_value: Variant = V.support(support_scene,snapshot,SCENE_SHA256)
	if not _support_scene_valid(snapshot,poses_value):
		return {}
	for support_id: String in SUPPORT_ID_ORDER:
		var pose_value: Variant = poses_value.get(support_id)
		if not _pose_values_match(pose_value,support_id,support_scene):
			return {}
	return poses_value

func _support_polygon(support_id: String) -> Array:
	var rectangle_value: Variant = SUPPORT_RECTANGLES.get(support_id)
	if not rectangle_value is Dictionary:
		return []
	var rectangle: Dictionary = rectangle_value
	return [[rectangle.x0,rectangle.z0],[rectangle.x1,rectangle.z0],[rectangle.x1,rectangle.z1],[rectangle.x0,rectangle.z1]]

func _append_support_tiles(camera_value: Variant, support_id: String, height: float, tile_points: Array[PackedVector2Array], tile_uvs: Array[PackedVector2Array]) -> bool:
	var rectangle_value: Variant = SUPPORT_RECTANGLES.get(support_id)
	if not rectangle_value is Dictionary:
		return false
	var rectangle: Dictionary = rectangle_value
	var x0: float = float(rectangle.x0)
	var x1: float = float(rectangle.x1)
	var z0: float = float(rectangle.z0)
	var z1: float = float(rectangle.z1)
	var first_column: int = int(floor(x0/2.0))
	var last_column: int = int(ceil(x1/2.0))-1
	var first_row: int = int(floor(z0/2.0))
	var last_row: int = int(ceil(z1/2.0))-1
	for row_index: int in range(first_row,last_row+1):
		for column_index: int in range(first_column,last_column+1):
			var cell_x0: float = maxf(x0,float(column_index)*2.0)
			var cell_x1: float = minf(x1,float(column_index+1)*2.0)
			var cell_z0: float = maxf(z0,float(row_index)*2.0)
			var cell_z1: float = minf(z1,float(row_index+1)*2.0)
			if cell_x1 <= cell_x0 or cell_z1 <= cell_z0:
				return false
			var u0: float = (cell_x0-float(column_index)*2.0)/2.0
			var u1: float = (cell_x1-float(column_index)*2.0)/2.0
			var v0: float = (cell_z0-float(row_index)*2.0)/2.0
			var v1: float = (cell_z1-float(row_index)*2.0)/2.0
			var world: Array[Vector3] = [Vector3(cell_x0,height,cell_z0),Vector3(cell_x1,height,cell_z0),Vector3(cell_x1,height,cell_z1),Vector3(cell_x0,height,cell_z1)]
			var points: PackedVector2Array = _project_polygon(camera_value,world)
			if points.is_empty():
				return false
			var uvs: PackedVector2Array = PackedVector2Array([_uv(u0,v0),_uv(u1,v0),_uv(u1,v1),_uv(u0,v1)])
			for uv_point: Vector2 in uvs:
				if not _finite_vector2(uv_point):
					return false
			tile_points.append(points)
			tile_uvs.append(uvs)
	return true

func _inside_rectangle(x: float, z: float, rectangle: Dictionary, epsilon: float = 0.0) -> bool:
	return x > float(rectangle.x0)+epsilon and x < float(rectangle.x1)-epsilon and z > float(rectangle.z0)+epsilon and z < float(rectangle.z1)-epsilon

func _edge_already_owned(support_id: String, edge: String, segment0: float, segment1: float, height: float, support_infos: Array[Dictionary]) -> bool:
	var rectangle: Dictionary = SUPPORT_RECTANGLES[support_id]
	var boundary: float = float(rectangle.x1) if edge == 'east' else float(rectangle.z1)
	for other_info: Dictionary in support_infos:
		var other_id: String = str(other_info.id)
		if other_id == support_id or absf(float(other_info.height)-height) > POSE_EPSILON:
			continue
		var other_rectangle: Dictionary = SUPPORT_RECTANGLES[other_id]
		var other_boundary: float = float(other_rectangle.x1) if edge == 'east' else float(other_rectangle.z1)
		var other0: float = float(other_rectangle.z0) if edge == 'east' else float(other_rectangle.x0)
		var other1: float = float(other_rectangle.z1) if edge == 'east' else float(other_rectangle.x1)
		if absf(boundary-other_boundary) <= POSE_EPSILON and maxf(segment0,other0) < minf(segment1,other1)-POSE_EPSILON and other_id < support_id:
			return true
	return false

func _append_fascia_edge(camera_value: Variant, support_id: String, edge: String, height: float, support_infos: Array[Dictionary], fascia_plan: Array[Dictionary]) -> bool:
	if height <= 0.0:
		return true
	var rectangle: Dictionary = SUPPORT_RECTANGLES[support_id]
	var first: float = float(rectangle.z0) if edge == 'east' else float(rectangle.x0)
	var last: float = float(rectangle.z1) if edge == 'east' else float(rectangle.x1)
	var cuts: Array[float] = [first,last]
	for other_id: String in SUPPORT_ID_ORDER:
		if other_id == support_id:
			continue
		var other_rectangle: Dictionary = SUPPORT_RECTANGLES[other_id]
		var low: float = float(other_rectangle.z0) if edge == 'east' else float(other_rectangle.x0)
		var high: float = float(other_rectangle.z1) if edge == 'east' else float(other_rectangle.x1)
		if low > first and low < last:
			cuts.append(low)
		if high > first and high < last:
			cuts.append(high)
	cuts.sort()
	var unique_cuts: Array[float] = []
	for cut: float in cuts:
		if unique_cuts.is_empty() or absf(cut-unique_cuts[unique_cuts.size()-1]) > POSE_EPSILON:
			unique_cuts.append(cut)
	var own_boundary: float = float(rectangle.x1) if edge == 'east' else float(rectangle.z1)
	var bottom: float = maxf(0.0,height-FASCIA_HEIGHT)
	for cut_index: int in range(unique_cuts.size()-1):
		var segment0: float = unique_cuts[cut_index]
		var segment1: float = unique_cuts[cut_index+1]
		if segment1 <= segment0:
			continue
		var midpoint: float = (segment0+segment1)*0.5
		if _edge_already_owned(support_id,edge,segment0,segment1,height,support_infos):
			continue
		var outward_x: float = own_boundary+0.0001 if edge == 'east' else midpoint
		var outward_z: float = midpoint if edge == 'east' else own_boundary+0.0001
		var covered: bool = false
		for other_info: Dictionary in support_infos:
			var other_id: String = str(other_info.id)
			if other_id == support_id or float(other_info.height)+POSE_EPSILON < height:
				continue
			var other_rectangle: Dictionary = SUPPORT_RECTANGLES[other_id]
			if _inside_rectangle(outward_x,outward_z,other_rectangle,0.00005):
				covered = true
				break
		if covered:
			continue
		var world: Array[Vector3]
		if edge == 'east':
			world = [Vector3(own_boundary,bottom,segment0),Vector3(own_boundary,bottom,segment1),Vector3(own_boundary,height,segment1),Vector3(own_boundary,height,segment0)]
		else:
			world = [Vector3(segment0,bottom,own_boundary),Vector3(segment1,bottom,own_boundary),Vector3(segment1,height,own_boundary),Vector3(segment0,height,own_boundary)]
		var points: PackedVector2Array = _project_polygon(camera_value,world)
		if points.is_empty():
			return false
		fascia_plan.append({'kind':'fill','points':points,'color':FASCIA_COLOR,'width':0.0})
		if is_finite(float(camera_value.get('ppm'))):
			var highlight_width: float = 0.65*float(camera_value.get('ppm'))/48.0
			if not is_finite(highlight_width) or highlight_width <= 0.0:
				return false
			var line_world: Array[Vector3]
			if edge == 'east':
				line_world = [Vector3(own_boundary,height,segment0),Vector3(own_boundary,height,segment1)]
			else:
				line_world = [Vector3(segment0,height,own_boundary),Vector3(segment1,height,own_boundary)]
			var line_points: PackedVector2Array = _project_line(camera_value,line_world)
			if line_points.is_empty():
				return false
			fascia_plan.append({'kind':'line','points':line_points,'color':FASCIA_HIGHLIGHT,'width':highlight_width})
	return true

func _support_info(poses: Dictionary) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for support_id: String in SUPPORT_ID_ORDER:
		var pose_value: Variant = poses.get(support_id)
		if not pose_value is Dictionary:
			return []
		var pose: Dictionary = pose_value
		if not P.number(pose.get('heightM'),0.0,2.0):
			return []
		result.append({'id':support_id,'height':float(pose.get('heightM')),'polygon':_support_polygon(support_id)})
	for first_index: int in range(result.size()):
		for second_index: int in range(first_index+1,result.size()):
			var first_info: Dictionary = result[first_index]
			var second_info: Dictionary = result[second_index]
			var swap: bool = float(second_info.height) < float(first_info.height)-POSE_EPSILON
			if absf(float(second_info.height)-float(first_info.height)) <= POSE_EPSILON and str(second_info.id) < str(first_info.id):
				swap = true
			if swap:
				result[first_index] = second_info
				result[second_index] = first_info
	return result

func draw_supports(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, hash_value: Variant, camera_value: Variant, viewport_value: Variant) -> Array[String]:
	last_emitted_support_ids.clear()
	if not _draw_context(canvas_value,snapshot_value,scene_value,key_value,hash_value,camera_value,viewport_value):
		return []
	var poses: Dictionary = _validated_poses(snapshot_value,scene_value,key_value,hash_value)
	if poses.size() != 3:
		return []
	var support_infos: Array[Dictionary] = _support_info(poses)
	if support_infos.size() != 3:
		return []
	var top_points: Array[PackedVector2Array] = []
	var top_uvs: Array[PackedVector2Array] = []
	for info: Dictionary in support_infos:
		if not _append_support_tiles(camera_value,str(info.id),float(info.height),top_points,top_uvs):
			return []
	if top_points.size() != 18 or top_uvs.size() != 18:
		return []
	var fascia_plan: Array[Dictionary] = []
	for info: Dictionary in support_infos:
		var support_id: String = str(info.id)
		var height: float = float(info.height)
		if not _append_fascia_edge(camera_value,support_id,'east',height,support_infos,fascia_plan) or not _append_fascia_edge(camera_value,support_id,'south',height,support_infos,fascia_plan):
			return []
	for fascia_command_value: Variant in fascia_plan:
		if not fascia_command_value is Dictionary:
			return []
		var fascia_command: Dictionary = fascia_command_value
		var command_points_value: Variant = fascia_command.get('points')
		var command_color_value: Variant = fascia_command.get('color')
		var command_kind: Variant = fascia_command.get('kind')
		if not command_points_value is PackedVector2Array or not command_color_value is Color or typeof(command_kind) != TYPE_STRING:
			return []
		var command_points: PackedVector2Array = command_points_value
		if command_kind == 'fill':
			if command_points.size() < 3 or fascia_command.size() != 4 or not P.number(fascia_command.get('width'),0.0,0.0):
				return []
		elif command_kind == 'line':
			var width_value: Variant = fascia_command.get('width')
			if command_points.size() != 2 or fascia_command.size() != 4 or not P.number(width_value,0.000001,0.65*float(camera_value.get('ppm'))/48.0):
				return []
		else:
			return []
	var canvas: CanvasItem = canvas_value
	for tile_index: int in range(top_points.size()):
		canvas.draw_polygon(top_points[tile_index],PackedColorArray([SUPPORT_TINT,SUPPORT_TINT,SUPPORT_TINT,SUPPORT_TINT]),top_uvs[tile_index],_steel_texture)
	for fascia_command: Dictionary in fascia_plan:
		var command_points: PackedVector2Array = fascia_command.get('points')
		var command_color: Color = fascia_command.get('color')
		if fascia_command.get('kind') == 'fill':
			canvas.draw_colored_polygon(command_points,command_color)
		elif fascia_command.get('kind') == 'line':
			var width_value: Variant = fascia_command.get('width')
			canvas.draw_polyline(command_points,command_color,float(width_value),true)
	for info: Dictionary in support_infos:
		last_emitted_support_ids.append(str(info.id))
	return last_emitted_support_ids.duplicate()

func should_skip_legacy_support(canvas_value: Variant, row_value: Variant, field_value: Variant, poses_value: Variant, drawn_ids_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, hash_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not _draw_context(canvas_value,snapshot_value,scene_value,key_value,hash_value,camera_value,viewport_value) or not row_value is Dictionary or typeof(field_value) != TYPE_STRING or not poses_value is Dictionary or not drawn_ids_value is Array:
		return false
	var row: Dictionary = row_value
	var field: String = str(field_value)
	var expected_rows: Array = STANDING_DECKS if field == 'standingDecks' else MOVING_SUPPORTS if field == 'movingSupports' else []
	if expected_rows.is_empty():
		return false
	var expected_row: Dictionary = {}
	for candidate_value: Variant in expected_rows:
		if candidate_value is Dictionary and candidate_value.get('id') == row.get('id'):
			expected_row = candidate_value
			break
	if expected_row.is_empty() or not _same(row,expected_row):
		return false
	var support_id: Variant = row.get('id')
	if typeof(support_id) != TYPE_STRING or not drawn_ids_value.has(support_id) or not poses_value.has(support_id):
		return false
	var snapshot: Dictionary = snapshot_value
	var poses: Dictionary = poses_value
	if not _support_scene_valid(snapshot,poses) or not _pose_values_match(poses.get(support_id),str(support_id),snapshot.get('supportScene')):
		return false
	return true
