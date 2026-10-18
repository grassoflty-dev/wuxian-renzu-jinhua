extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const ProjectionScript = preload("res://scripts/world/projection.gd")
const SCENE_PATH: String = "res://data/scenes/mh_breakwater.json"
const SCENE_BYTES: int = 8283
const SCENE_SHA: String = "2de6f7b471b6b873fafad7e943a0f163e057ffca0787823625bf37bb23bd0b14"
const MAP_PATH: String = "res://data/mh_breakwater_environment.internal-use.json"
const MAP_BYTES: int = 13530
const MAP_SHA: String = "9fc84205b01f672aea6b2af19e4667aeb29060b1395e42d54758fd39f52b1ada"
const SPEC: Dictionary = {
	"schema": "wuxian.mh-breakwater-environment.internal-use/v1",
	"registeredSourceReuse": true,
	"approval": "INTERNAL_REGISTERED_SOURCE_REUSE",
	"roomUse": "MAIN_EXPLICIT_BREAKWATER_ONLY",
	"GodotDisplay": "NOT_RUN",
	"publicRelease": "NOT_ESTABLISHED",
	"source": {
		"path": "res://data/scenes/mh_breakwater.json",
		"bytes": 8283,
		"sha256": "2de6f7b471b6b873fafad7e943a0f163e057ffca0787823625bf37bb23bd0b14",
		"worldId": "mist_harbor",
		"sceneId": "mh_breakwater",
		"boundsM": [
			0,
			0,
			24,
			16
		]
	},
	"sourcePins": [
		{
			"path": "apps/godot_playable/scripts/world/renderer.gd",
			"bytes": 124062,
			"sha256": "6ffe76c9f30d016d96e1c56247826ea0cbff8d10632f4fc651542ad8a7edcc5a"
		},
		{
			"path": "apps/godot_playable/data/scenes/mh_breakwater.json",
			"bytes": 8283,
			"sha256": "2de6f7b471b6b873fafad7e943a0f163e057ffca0787823625bf37bb23bd0b14"
		},
		{
			"path": "content/scenes/compiled/mh_breakwater.json",
			"bytes": 8283,
			"sha256": "2de6f7b471b6b873fafad7e943a0f163e057ffca0787823625bf37bb23bd0b14"
		},
		{
			"path": "apps/godot_playable/scripts/bridge/protocol.gd",
			"bytes": 7675,
			"sha256": "9558f4192c4738b475c6c03046a1db1cd4074a8caf689008cccab013404b696b"
		},
		{
			"path": "apps/godot_playable/scripts/world/projection.gd",
			"bytes": 3301,
			"sha256": "45f518f6dddebaebedd6bc1b5eb66476ebd3f039779cb353cc6124fbc5c9c82e"
		},
		{
			"path": "apps/godot_playable/scripts/world/presentation.gd",
			"bytes": 7353,
			"sha256": "659c9ccf61f4be94eaee10394e08b57accf7556f12f568809e8d61ae7a43e087"
		},
		{
			"path": "apps/godot_playable/scripts/world/mh_cw_actor_visual.gd",
			"bytes": 4632,
			"sha256": "3e334dec418f1319881a82bb6024a06fb7b1d426b1000e25ecc61f9f7c7f7568"
		},
		{
			"path": "apps/godot_playable/data/mh_cw_actor.internal-use.json",
			"bytes": 3028,
			"sha256": "7a88eece1864b352e2934163546a2303bd3a46457c167a00a5398f657c49590f"
		},
		{
			"path": "apps/godot_playable/scripts/world/cenyao_idle_visual.gd",
			"bytes": 3875,
			"sha256": "52ead3447f264a4a6328b82142345f3fd3566599b95f4dadc15fa80d8a217705"
		},
		{
			"path": "apps/godot_playable/data/cenyao_idle.internal-use.json",
			"bytes": 3925,
			"sha256": "b063efd36046834c41ebed05c12ffb8729926f7d4edff0e15d36dccdbb1da881"
		},
		{
			"path": "apps/godot_playable/scripts/world/mh_fog_pier_stone_material_visual.gd",
			"bytes": 14903,
			"sha256": "f2639e9a11098bdb729fe1af47a5b2f7006a9ca155e644d9dee73b56794c5eb0"
		},
		{
			"path": "apps/godot_playable/data/mh_fog_pier_stone_material.internal-use.json",
			"bytes": 1517,
			"sha256": "c56ad7549a49809341e2aea6efcbe7fbe0477eb51026e776c4a49fa03077585a"
		},
		{
			"path": "apps/godot_playable/scripts/world/mh_fog_pier_layered_fog_visual.gd",
			"bytes": 21606,
			"sha256": "656d7cfa2eb43f6d2c18924978e6ef70685f39fb51447a42fbbcb9eb81d3bf97"
		},
		{
			"path": "apps/godot_playable/data/mh_fog_pier_layered_fog.internal-use.json",
			"bytes": 2270,
			"sha256": "027980b44e5f92aa5e7759611eeffe69d6ae59be90a4ae4e0858a96123d2bee0"
		},
		{
			"path": "apps/godot_playable/scripts/world/mh_drowned_quay_environment_visual.gd",
			"bytes": 20501,
			"sha256": "86557ddb26f254963ee2fcdb1f26086e08ecbf0fd7090e967574f5815a52a693"
		},
		{
			"path": "apps/godot_playable/data/mh_drowned_quay_environment.internal-use.json",
			"bytes": 8536,
			"sha256": "1470a6290a67a664c054e049613ce17bff71119a572a5940a4e47d5cd39afded"
		},
		{
			"path": "apps/godot_playable/project.godot",
			"bytes": 556,
			"sha256": "9f6c26d26037b0f61bc07f36db40f451f544e7f5542f49ca6d53055f5b5758a3"
		}
	],
	"upstreamMaps": {
		"stone": {
			"path": "res://data/mh_fog_pier_stone_material.internal-use.json",
			"bytes": 1517,
			"sha256": "c56ad7549a49809341e2aea6efcbe7fbe0477eb51026e776c4a49fa03077585a"
		},
		"fog": {
			"path": "res://data/mh_fog_pier_layered_fog.internal-use.json",
			"bytes": 2270,
			"sha256": "027980b44e5f92aa5e7759611eeffe69d6ae59be90a4ae4e0858a96123d2bee0"
		},
		"actor": {
			"path": "res://data/mh_cw_actor.internal-use.json",
			"bytes": 3028,
			"sha256": "7a88eece1864b352e2934163546a2303bd3a46457c167a00a5398f657c49590f"
		},
		"idle": {
			"path": "res://data/cenyao_idle.internal-use.json",
			"bytes": 3925,
			"sha256": "b063efd36046834c41ebed05c12ffb8729926f7d4edff0e15d36dccdbb1da881"
		}
	},
	"resources": {
		"stone": {
			"assetId": "runtime2d.world.mistharbor.water_edge_stone.v1",
			"derivedRelative": "assets/derived/asset-pipeline-v1/sprites/png/runtime2d.world.mistharbor.water_edge_stone.v1.png",
			"sourceRelative": "batch07/02_runtime2d.world.mistharbor.water_edge_stone.v1.png",
			"sourceSha256": "dc09452bf3f55c57cca1a93bb09f169ef441af90321dff47d91c4be7e62d3a83",
			"path": "res://assets/props/mist_harbor/fog_pier_stone_source.png",
			"bytes": 1983603,
			"sha256": "e823434b3fbc420fc14270200f81e0a459e594597304b1323d336e532a924f9d",
			"canvas": [
				1228,
				1254
			],
			"format": "RGBA8"
		},
		"fog": {
			"assetId": "runtime2d.world.mistharbor.fog_bank.v1",
			"path": "res://assets/props/mist_harbor/fog_pier_fog_bank_source.png",
			"bytes": 684960,
			"sha256": "9bbbfd9783f70a8b1f6ffeaabd44e568074f4c20f1f7500d7ef4f0e534edeac2",
			"canvas": [
				1254,
				1236
			],
			"format": "RGBA8"
		},
		"internalReads": [
			{
				"path": "res://assets/props/mist_harbor/fog_pier_stone_source.png",
				"bytes": 1983603,
				"sha256": "e823434b3fbc420fc14270200f81e0a459e594597304b1323d336e532a924f9d"
			},
			{
				"path": "res://assets/props/mist_harbor/fog_pier_fog_bank_source.png",
				"bytes": 684960,
				"sha256": "9bbbfd9783f70a8b1f6ffeaabd44e568074f4c20f1f7500d7ef4f0e534edeac2"
			},
			{
				"path": "res://assets/enemies/mist_harbor/drowned.png",
				"bytes": 1128162,
				"sha256": "752ac830587fc957d5695cd7714c57e584f40ec1a8f511f324ee491adaee7f2d"
			},
			{
				"path": "res://assets/enemies/clockworks/forged_guard.png",
				"bytes": 1300721,
				"sha256": "b41cdc83c587805d0beee9d48668a9328001882595e0ab97fa69347d9bcf8cb4"
			},
			{
				"path": "res://assets/enemies/mist_harbor/signal_wraith.png",
				"bytes": 1440594,
				"sha256": "942a294e86076bfa91762113cd9bb14c81db329baabf5804ba9138942cc7166b"
			},
			{
				"path": "res://assets/characters/cenyao/idle/cenyao_idle_south_west.png",
				"bytes": 158163,
				"sha256": "3d3e34d0a5d388de522fd6a94fd2769efbf8137aea1ae7c0b69881c5416520be"
			},
			{
				"path": "res://assets/characters/cenyao/idle/cenyao_idle_east.png",
				"bytes": 109177,
				"sha256": "fd38db1daec829c0a7c2adffcdfd5c605f54b3611bb515599cd8deeea1f47402"
			},
			{
				"path": "res://assets/characters/cenyao/idle/cenyao_idle_west.png",
				"bytes": 121863,
				"sha256": "fc8e0b132c48f594a49a20bd39955d731f02077502e33131499608cc3bd7a70c"
			},
			{
				"path": "res://assets/characters/cenyao/idle/cenyao_idle_south.png",
				"bytes": 158435,
				"sha256": "b6cff22dabb303f0a9c95ba24a41fb3c2f5159ff191adefacf0c6dad38ffe875"
			},
			{
				"path": "res://assets/characters/cenyao/idle/cenyao_idle_north_east.png",
				"bytes": 144761,
				"sha256": "19b8e318bf5216d6075f64ac986ef7c3c3299241816b472843aac24bbf4a7676"
			},
			{
				"path": "res://assets/characters/cenyao/idle/cenyao_idle_south_east.png",
				"bytes": 158547,
				"sha256": "bfe0107fe1efdcd73b90d57b01283a22658960b5847bb5e88b401dae9857be56"
			},
			{
				"path": "res://assets/characters/cenyao/idle/cenyao_idle_north.png",
				"bytes": 151244,
				"sha256": "6608ed34685f28f3380849d7fe5e5319078e8feb1fbd66cbcd4cb3782a1a8221"
			},
			{
				"path": "res://assets/characters/cenyao/idle/cenyao_idle_north_west.png",
				"bytes": 143105,
				"sha256": "8dc527b975b1da5ed7b36a54cf4ad260cdcd9d7d3ccba24660308cf971c3164e"
			}
		],
		"actorDisplay": "Only two normally gated Breakwater Signal Wraith IDs and Cenyao; Drowned/Guard integrity-only, Tidebound fallback unchanged"
	},
	"collision": [
		{
			"id": "breakwater_wall_north",
			"polygon": [
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
			]
		},
		{
			"id": "breakwater_wall_south",
			"polygon": [
				[
					0,
					15.5
				],
				[
					24,
					15.5
				],
				[
					24,
					16
				],
				[
					0,
					16
				]
			]
		},
		{
			"id": "breakwater_wall_west_north",
			"polygon": [
				[
					0,
					0.5
				],
				[
					0.5,
					0.5
				],
				[
					0.5,
					6
				],
				[
					0,
					6
				]
			]
		},
		{
			"id": "breakwater_wall_west_south",
			"polygon": [
				[
					0,
					9.5
				],
				[
					0.5,
					9.5
				],
				[
					0.5,
					15.5
				],
				[
					0,
					15.5
				]
			]
		},
		{
			"id": "breakwater_wall_east_north",
			"polygon": [
				[
					23.5,
					0.5
				],
				[
					24,
					0.5
				],
				[
					24,
					6
				],
				[
					23.5,
					6
				]
			]
		},
		{
			"id": "breakwater_wall_east_south",
			"polygon": [
				[
					23.5,
					9.5
				],
				[
					24,
					9.5
				],
				[
					24,
					15.5
				],
				[
					23.5,
					15.5
				]
			]
		}
	],
	"walkablePolygons": [
		{
			"id": "mh_breakwater_walkable_area",
			"polygon": [
				[
					0.5,
					0.5
				],
				[
					23.5,
					0.5
				],
				[
					23.5,
					15.5
				],
				[
					0.5,
					15.5
				]
			]
		}
	],
	"sprites": [
		{
			"assetId": "runtime2d.world.mistharbor.water_edge_stone.v1",
			"id": "approved_seawall_stone_water_module",
			"layer": "visual.floor",
			"position": [
				12,
				0,
				8
			]
		},
		{
			"assetId": "runtime2d.world.mistharbor.fog_bank.v1",
			"id": "breakwater_heavy_fog_bank",
			"layer": "visual.props_back",
			"position": [
				8,
				0,
				3
			]
		},
		{
			"assetId": "runtime2d.world.mistharbor.water_edge.v1",
			"id": "breakwater_outer_water_edge",
			"layer": "visual.props_back",
			"position": [
				12,
				0,
				14.5
			]
		},
		{
			"assetId": "runtime2d.world.mistharbor.fog_bank.v1",
			"id": "breakwater_fog_bank_foreground",
			"layer": "visual.foreground",
			"position": [
				20,
				0,
				4
			]
		}
	],
	"floor": {
		"originM": [
			0.5,
			0.5
		],
		"limitM": [
			23.5,
			15.5
		],
		"graphicCellM": [
			4,
			4
		],
		"grid": [
			6,
			4
		],
		"pieces": 24,
		"clippedLastUV": [
			0.75,
			0.75
		],
		"groundY": 0,
		"pixelEdgeCorners": [
			[
				820,
				240
			],
			[
				1032,
				342
			],
			[
				392,
				627
			],
			[
				180,
				525
			]
		],
		"canvas": [
			1228,
			1254
		],
		"tint": [
			0.4,
			0.5,
			0.54,
			1
		],
		"normalAlpha": true,
		"linear": true,
		"repeat": false,
		"metricSourceMeasurement": false
	},
	"walls": {
		"visualHeightM": 0.35,
		"face": "#33494f",
		"cap": "#60767c",
		"contact": [
			0.025,
			0.032,
			0.035,
			0.1
		],
		"contactInsideFootprint": true,
		"newCollision": false
	},
	"fog": {
		"canvas": [
			1254,
			1236
		],
		"cropXYWH": [
			27,
			370,
			1201,
			498
		],
		"centerAnchor": [
			0.5,
			0.5
		],
		"rgb": [
			0.8,
			0.88,
			0.92
		],
		"normalAlpha": true,
		"linear": true,
		"repeat": false,
		"static": true,
		"reducedMotion": "identical geometry/alpha, no clock",
		"densityAlphaCap": 0.3
	},
	"banks": [
		{
			"id": "breakwater_heavy_fog_bank",
			"assetId": "runtime2d.world.mistharbor.fog_bank.v1",
			"layer": "visual.props_back",
			"position": [
				8,
				0,
				3
			],
			"phase": "back",
			"widthPerPpm": 7.5,
			"alpha": 0.3
		},
		{
			"id": "breakwater_fog_bank_foreground",
			"assetId": "runtime2d.world.mistharbor.fog_bank.v1",
			"layer": "visual.foreground",
			"position": [
				20,
				0,
				4
			],
			"phase": "front",
			"widthPerPpm": 5,
			"alpha": 0.1
		}
	],
	"excluded": [
		"shore/water/terrain/support geometry",
		"beacon operations/body",
		"new vfx marker consumption",
		"actor/perception/roster/body/label/culling changes",
		"FogPier calibration/time",
		"Quay drained failure repair",
		"Gear retry",
		"UI/native/game/save/rights/media release"
	]
}
const STONE_COLOR: Color = Color(0.40,0.50,0.54,1.0)
const WALL_FACE: Color = Color("#33494f")
const WALL_CAP: Color = Color("#60767c")
const WALL_CONTACT: Color = Color(0.025,0.032,0.035,0.10)

var _attempted: bool = false
var _loaded: bool = false
var _ready: bool = false
var _key: String = ""
var _source: Dictionary = {}
var _textures: Dictionary = {}
var prepare_error: String = ""

func _same(a: Variant, b: Variant) -> bool:
	var at: int = typeof(a)
	var bt: int = typeof(b)
	if at in [TYPE_INT,TYPE_FLOAT] and bt in [TYPE_INT,TYPE_FLOAT]:
		return is_finite(float(a)) and is_finite(float(b)) and float(a) == float(b)
	if at != bt:
		return false
	if a is Dictionary:
		var ad: Dictionary = a
		var bd: Dictionary = b
		if ad.size() != bd.size():
			return false
		for k: Variant in ad:
			if not bd.has(k) or not _same(ad[k],bd[k]):
				return false
		return true
	if a is Array:
		var aa: Array = a
		var ba: Array = b
		if aa.size() != ba.size():
			return false
		for i: int in range(aa.size()):
			if not _same(aa[i],ba[i]):
				return false
		return true
	return a == b

func _bytes(path: String, max_size: int) -> PackedByteArray:
	var file: FileAccess = FileAccess.open(path,FileAccess.READ)
	if file == null:
		return PackedByteArray()
	var size: int = file.get_length()
	if size <= 0 or size > max_size:
		file.close()
		return PackedByteArray()
	var data: PackedByteArray = file.get_buffer(size)
	file.close()
	return data if data.size() == size else PackedByteArray()

func _sha(data: PackedByteArray) -> String:
	var h: HashingContext = HashingContext.new()
	if h.start(HashingContext.HASH_SHA256) != OK or h.update(data) != OK:
		return ""
	return h.finish().hex_encode()

func _basic(sv: Variant, dv: Variant, kv: Variant, hv: Variant) -> bool:
	if not sv is Dictionary or not dv is Dictionary or not P.snapshot(sv):
		return false
	var s: Dictionary = sv
	var d: Dictionary = dv
	return s.get("entryToken") == null and s.get("worldId") == "mist_harbor" and s.get("sceneId") == "mh_breakwater" and P.counter(s.get("worldEpoch"),1) and typeof(kv) == TYPE_STRING and kv == "mist_harbor:mh_breakwater:"+str(s.get("worldEpoch")) and hv == SCENE_SHA and d.get("worldId") == "mist_harbor" and d.get("sceneId") == "mh_breakwater"

func _source_shape(d: Dictionary) -> bool:
	return P.counter(d.get("schemaVersion"),1,1) and _same(d.get("boundsM"),{"x":0,"z":0,"width":24,"depth":16}) and _same(d.get("collision"),SPEC.collision) and _same(d.get("walkablePolygons"),SPEC.walkablePolygons) and d.get("presentation") is Dictionary and _same(d.presentation.get("sprites"),SPEC.sprites)

func prepare(sv: Variant, dv: Variant, kv: Variant, hv: Variant) -> bool:
	_ready = false
	_key = ""
	prepare_error = ""
	if not _basic(sv,dv,kv,hv):
		return false
	if _attempted:
		if not _loaded or not _same(dv,_source):
			prepare_error = "cached resource failure or current scene mismatch"
			return false
		_key = str(kv)
		_ready = true
		return true
	_attempted = true
	var mb: PackedByteArray = _bytes(MAP_PATH,32768)
	if mb.size() != MAP_BYTES or _sha(mb) != MAP_SHA or mb.get_string_from_utf8().to_utf8_buffer() != mb:
		prepare_error = "map bytes/hash/UTF8"
		return false
	var map_value: Variant = JSON.parse_string(mb.get_string_from_utf8())
	if not _same(map_value,SPEC):
		prepare_error = "full map literal/keysets"
		return false
	var sb: PackedByteArray = _bytes(SCENE_PATH,16384)
	if sb.size() != SCENE_BYTES or _sha(sb) != SCENE_SHA or sb.get_string_from_utf8().to_utf8_buffer() != sb:
		prepare_error = "canonical scene bytes/hash/UTF8"
		return false
	var scene_value: Variant = JSON.parse_string(sb.get_string_from_utf8())
	if not scene_value is Dictionary or not _source_shape(scene_value) or not _same(dv,scene_value):
		prepare_error = "current canonical scene binding"
		return false
	for name: String in ["stone","fog"]:
		var cfg_value: Variant = SPEC.resources.get(name)
		if not cfg_value is Dictionary:
			prepare_error = "resource definition"
			return false
		var cfg: Dictionary = cfg_value
		var png: PackedByteArray = _bytes(str(cfg.path),4*1024*1024)
		if png.size() != int(cfg.bytes) or _sha(png) != cfg.sha256:
			prepare_error = name+" bytes/hash"
			return false
		var image: Image = Image.new()
		if image.load_png_from_buffer(png) != OK or image.get_format() != Image.FORMAT_RGBA8 or image.get_width() != int(cfg.canvas[0]) or image.get_height() != int(cfg.canvas[1]):
			prepare_error = name+" PNG decode/canvas/RGBA8"
			return false
		var texture: ImageTexture = ImageTexture.create_from_image(image)
		if texture == null:
			prepare_error = name+" texture creation"
			return false
		_textures[name] = texture
	_source = scene_value.duplicate(true)
	_loaded = true
	_key = str(kv)
	_ready = true
	return true

func _v2(p: Vector2) -> bool:
	return is_finite(p.x) and is_finite(p.y)

func _v3(p: Vector3) -> bool:
	return is_finite(p.x) and is_finite(p.y) and is_finite(p.z)

func matches_context(cv: Variant, sv: Variant, dv: Variant, kv: Variant, camera: Variant, viewport: Variant) -> bool:
	if not _ready or not _loaded or not _basic(sv,dv,kv,SCENE_SHA) or kv != _key or not _same(dv,_source):
		return false
	if not cv is CanvasItem or not is_instance_valid(cv) or not cv.is_inside_tree() or cv.is_queued_for_deletion():
		return false
	if cv.get("current_key") != kv or cv.get("scene_hash") != SCENE_SHA or not _same(cv.get("snapshot"),sv) or not _same(cv.get("scene"),dv):
		return false
	if cv.get("camera") != camera or cv.get("viewport_rect") != viewport:
		return false
	if not camera is RefCounted or not is_instance_valid(camera) or camera.get_script() != ProjectionScript or not viewport is Rect2:
		return false
	var origin: Variant = camera.get("origin")
	var center: Variant = camera.get("screen_center")
	var size: Variant = camera.get("viewport")
	var initialized: Variant = camera.get("initialized")
	if not origin is Vector3 or not center is Vector2 or not size is Vector2 or typeof(initialized) != TYPE_BOOL or not initialized or not P.number(camera.get("ppm"),0.000001,256):
		return false
	var rect: Rect2 = viewport
	return _v3(origin) and _v2(center) and _v2(size) and _v2(rect.position) and _v2(rect.size) and _v2(rect.end) and rect.size.x > 0 and rect.size.y > 0 and size == rect.size

func _project(camera: Variant, p: Vector3) -> Vector2:
	if not _v3(p):
		return Vector2(INF,INF)
	var value: Variant = camera.call("project",p)
	return value if value is Vector2 and _v2(value) else Vector2(INF,INF)

func _poly(camera: Variant, world: Array[Vector3]) -> PackedVector2Array:
	var points: PackedVector2Array = PackedVector2Array()
	for p: Vector3 in world:
		var q: Vector2 = _project(camera,p)
		if not _v2(q):
			return PackedVector2Array()
		points.append(q)
	if points.size() < 3:
		return PackedVector2Array()
	var area: float = 0
	for i: int in range(points.size()):
		area += points[i].cross(points[(i+1)%points.size()])
	return points if is_finite(area) and absf(area) > 0.000001 else PackedVector2Array()

func _uv(u: float, v: float) -> Vector2:
	return (Vector2(820,240)+Vector2(212,102)*u+Vector2(-640,285)*v)/Vector2(1228,1254)

func draw_ground(cv: Variant, sv: Variant, dv: Variant, kv: Variant, camera: Variant, viewport: Variant) -> bool:
	if not matches_context(cv,sv,dv,kv,camera,viewport) or not _textures.get("stone") is Texture2D:
		return false
	var polygons: Array[PackedVector2Array] = []
	var uvs: Array[PackedVector2Array] = []
	for row: int in range(4):
		for column: int in range(6):
			var x0: float = 0.5+column*4.0
			var z0: float = 0.5+row*4.0
			var x1: float = minf(x0+4.0,23.5)
			var z1: float = minf(z0+4.0,15.5)
			var points: PackedVector2Array = _poly(camera,[Vector3(x0,0,z0),Vector3(x1,0,z0),Vector3(x1,0,z1),Vector3(x0,0,z1)])
			if points.is_empty():
				return false
			var u1: float = (x1-x0)/4.0
			var v1: float = (z1-z0)/4.0
			var uv: PackedVector2Array = PackedVector2Array([_uv(0,0),_uv(u1,0),_uv(u1,v1),_uv(0,v1)])
			for p: Vector2 in uv:
				if not _v2(p):
					return false
			polygons.append(points)
			uvs.append(uv)
	var canvas: CanvasItem = cv
	var texture: Texture2D = _textures["stone"]
	for i: int in range(polygons.size()):
		canvas.draw_polygon(polygons[i],PackedColorArray([STONE_COLOR,STONE_COLOR,STONE_COLOR,STONE_COLOR]),uvs[i],texture)
	return polygons.size() == 24

func draw_wall(cv: Variant, wallv: Variant, sv: Variant, dv: Variant, kv: Variant, camera: Variant, viewport: Variant) -> bool:
	if not matches_context(cv,sv,dv,kv,camera,viewport) or not wallv is Dictionary:
		return false
	var wall: Dictionary = wallv
	var matched: bool = false
	for expected: Variant in SPEC.collision:
		if _same(wall,expected):
			matched = true
			break
	if not matched:
		return false
	var polygon: Array = wall.polygon
	var base: Array[Vector3] = []
	var top: Array[Vector3] = []
	for pair_value: Variant in polygon:
		if not pair_value is Array or pair_value.size() != 2 or not P.number(pair_value[0]) or not P.number(pair_value[1]):
			return false
		var pair: Array = pair_value
		base.append(Vector3(float(pair[0]),0,float(pair[1])))
		top.append(Vector3(float(pair[0]),0.35,float(pair[1])))
	var plan: Array[PackedVector2Array] = [_poly(camera,base)]
	var colors: Array[Color] = [WALL_CONTACT]
	for i: int in range(4):
		var j: int = (i+1)%4
		plan.append(_poly(camera,[base[i],base[j],top[j],top[i]]))
		colors.append(WALL_FACE)
	plan.append(_poly(camera,top))
	colors.append(WALL_CAP)
	for p: PackedVector2Array in plan:
		if p.is_empty():
			return false
	var canvas: CanvasItem = cv
	for i: int in range(plan.size()):
		canvas.draw_colored_polygon(plan[i],colors[i])
	return true

func _bank_rect(row: Dictionary, camera: Variant) -> Rect2:
	var xyz: Array = row.position
	var center: Vector2 = _project(camera,Vector3(float(xyz[0]),float(xyz[1]),float(xyz[2])))
	var width: float = float(row.widthPerPpm)*float(camera.get("ppm"))
	var size: Vector2 = Vector2(width,width*498.0/1201.0)
	return Rect2(center-size*0.5,size)

func _bank_row(phase: String) -> Dictionary:
	for value: Variant in SPEC.banks:
		if value is Dictionary and value.phase == phase:
			return value
	return {}

func _valid_bank_rect(rect: Rect2, viewport: Rect2) -> bool:
	return _v2(rect.position) and _v2(rect.size) and _v2(rect.end) and rect.size.x > 0 and rect.size.y > 0 and rect.intersects(viewport)

func draw_bank(cv: Variant, phase: String, sv: Variant, dv: Variant, kv: Variant, camera: Variant, viewport: Variant) -> Array[String]:
	if not matches_context(cv,sv,dv,kv,camera,viewport) or not _textures.get("fog") is Texture2D:
		return []
	var row: Dictionary = _bank_row(phase)
	if row.is_empty():
		return []
	var rect: Rect2 = _bank_rect(row,camera)
	if not _valid_bank_rect(rect,viewport):
		return []
	var canvas: CanvasItem = cv
	var texture: Texture2D = _textures["fog"]
	canvas.draw_texture_rect_region(texture,rect,Rect2(27,370,1201,498),Color(0.8,0.88,0.92,float(row.alpha)),false,true)
	var ids: Array[String] = [str(row.id)]
	return ids

func legacy_bank_consumed(cv: Variant, rowv: Variant, sv: Variant, dv: Variant, kv: Variant, camera: Variant, viewport: Variant, drawn_ids: Variant) -> bool:
	if not matches_context(cv,sv,dv,kv,camera,viewport) or not rowv is Dictionary or not drawn_ids is Array or not _textures.get("fog") is Texture2D:
		return false
	var row: Dictionary = rowv
	var canonical: bool = false
	for expected: Variant in SPEC.sprites:
		if expected is Dictionary and expected.get("id") == row.get("id") and expected.get("assetId") == "runtime2d.world.mistharbor.fog_bank.v1" and _same(expected,row):
			canonical = true
			break
	if not canonical or drawn_ids.count(row.get("id")) != 1:
		return false
	for bankv: Variant in SPEC.banks:
		if bankv is Dictionary and bankv.id == row.get("id"):
			var bank: Dictionary = bankv
			return _valid_bank_rect(_bank_rect(bank,camera),viewport)
	return false
