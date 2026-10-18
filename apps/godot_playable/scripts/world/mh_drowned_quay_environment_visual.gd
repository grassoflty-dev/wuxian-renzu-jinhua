extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const MAP_PATH: String = 'res://data/mh_drowned_quay_environment.internal-use.json'
const MAP_BYTES: int = 8536
const MAP_SHA: String = '1470a6290a67a664c054e049613ce17bff71119a572a5940a4e47d5cd39afded'
const DEFINITION: Dictionary = {
	"schema": "wuxian.mh-drowned-quay-environment.internal-use/v1",
	"registeredSourceReuse": true,
	"approval": "INTERNAL_REGISTERED_SOURCE_REUSE",
	"roomUse": "ONLY_EXPLICIT_MAIN_ADMITTED_MH_DROWNED_QUAY_SCOPE",
	"GodotDisplay": "NOT_RUN",
	"publicRelease": "NOT_ESTABLISHED",
	"worldId": "mist_harbor",
	"sceneId": "mh_drowned_quay",
	"scene": {
		"path": "res://data/scenes/mh_drowned_quay.json",
		"bytes": 8840,
		"sha256": "e315ef1be88ab2f1ff8defcf7050beef439b7793cda3d121e4e97b4bdf9b41a9"
	},
	"upstream": {
		"stone": {
			"path": "res://data/mh_fog_pier_stone_material.internal-use.json",
			"bytes": 1517,
			"sha256": "c56ad7549a49809341e2aea6efcbe7fbe0477eb51026e776c4a49fa03077585a"
		},
		"fog": {
			"path": "res://data/mh_fog_pier_layered_fog.internal-use.json",
			"bytes": 2270,
			"sha256": "027980b44e5f92aa5e7759611eeffe69d6ae59be90a4ae4e0858a96123d2bee0"
		}
	},
	"assets": {
		"stone": {
			"assetId": "runtime2d.world.mistharbor.water_edge_stone.v1",
			"path": "res://assets/props/mist_harbor/fog_pier_stone_source.png",
			"bytes": 1983603,
			"sha256": "e823434b3fbc420fc14270200f81e0a459e594597304b1323d336e532a924f9d",
			"canvas": [
				1228,
				1254
			],
			"format": "RGBA8",
			"sourceRelative": "batch07/02_runtime2d.world.mistharbor.water_edge_stone.v1.png",
			"sourceSha256": "dc09452bf3f55c57cca1a93bb09f169ef441af90321dff47d91c4be7e62d3a83"
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
			"format": "RGBA8",
			"crop": [
				27,
				370,
				1201,
				498
			]
		}
	},
	"source": {
		"bounds": {
			"depth": 16,
			"width": 24,
			"x": 0,
			"z": 0
		},
		"walkable": [
			{
				"id": "mh_drowned_quay_walkable_area",
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
		"walls": [
			{
				"id": "quay_wall_north",
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
				"id": "quay_wall_south",
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
				"id": "quay_wall_west_north",
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
				"id": "quay_wall_west_south",
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
				"id": "quay_wall_east_north",
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
				"id": "quay_wall_east_south",
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
		"terrain": [
			{
				"id": "mh_water_depth_region",
				"polygon": [
					[
						13,
						5
					],
					[
						19,
						5
					],
					[
						19,
						11
					],
					[
						13,
						11
					]
				],
				"tag": "terrain.water_shallow"
			}
		],
		"hazards": [
			{
				"id": "mh_water_depth_region",
				"kind": "water_depth_slowdown",
				"polygon": [
					[
						13,
						5
					],
					[
						19,
						5
					],
					[
						19,
						11
					],
					[
						13,
						11
					]
				],
				"terrainTag": "terrain.water_shallow"
			}
		],
		"sprites": [
			{
				"assetId": "runtime2d.world.mistharbor.water_edge_stone.v1",
				"id": "approved_submerged_stone_path",
				"layer": "visual.floor",
				"position": [
					12,
					0,
					8
				]
			},
			{
				"assetId": "runtime2d.world.mistharbor.water_edge.v1",
				"id": "quay_open_waterline",
				"layer": "visual.props_back",
				"position": [
					12,
					0,
					14.5
				]
			},
			{
				"assetId": "runtime2d.world.mistharbor.fog_bank.v1",
				"id": "quay_north_fog_bank",
				"layer": "visual.props_back",
				"position": [
					19,
					0,
					3
				]
			}
		]
	},
	"floor": {
		"rowId": "approved_submerged_stone_path",
		"selection": "affine_top_plane",
		"sourcePixelEdgeCorners": [
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
		"uvConvention": "sourcePixelEdge / standaloneCanvasSize",
		"bounds": [
			0.5,
			0.5,
			23.5,
			15.5
		],
		"patchSize": [
			4,
			4
		],
		"grid": [
			6,
			4
		],
		"patchCount": 24,
		"clipWorldAndUV": true,
		"tint": [
			0.42,
			0.54,
			0.58,
			1
		],
		"scaleBasis": "ART_DIRECTION_NOT_SOURCE_MEASURED"
	},
	"walls": {
		"displayHeight": 0.25,
		"face": "#33494f",
		"cap": "#58717a",
		"contact": [
			0.025,
			0.032,
			0.035,
			0.1
		],
		"geometry": "EXACT_SOURCE_FOOTPRINT_ONLY"
	},
	"water": {
		"id": "mh_water_depth_region",
		"style": [
			{
				"rect": [
					13,
					5,
					19,
					11
				],
				"color": "#2c5865",
				"alpha": 0.32
			},
			{
				"rect": [
					13.12,
					5.12,
					18.88,
					10.88
				],
				"color": "#315e6a",
				"alpha": 0.1
			},
			{
				"rect": [
					13.35,
					5.35,
					18.65,
					10.65
				],
				"color": "#426e78",
				"alpha": 0.05
			},
			{
				"rect": [
					14,
					6.25,
					17,
					6.34
				],
				"color": "#73929a",
				"alpha": 0.045
			},
			{
				"rect": [
					15,
					8.05,
					18,
					8.14
				],
				"color": "#73929a",
				"alpha": 0.045
			},
			{
				"rect": [
					13.8,
					9.55,
					16.8,
					9.64
				],
				"color": "#73929a",
				"alpha": 0.045
			}
		],
		"polygons": 6,
		"groundY": 0,
		"noAnimation": true,
		"noShoreline": true
	},
	"bank": {
		"rowId": "quay_north_fog_bank",
		"center": [
			19,
			0,
			3
		],
		"anchor": [
			0.5,
			0.5
		],
		"screenWidthPpmFactor": 6,
		"cropAspect": [
			1201,
			498
		],
		"tint": [
			0.8,
			0.88,
			0.92,
			0.22
		],
		"order": "BEFORE_ALL_SORTED_BODIES",
		"normalAlpha": true,
		"staticReducedMotionIdentical": true,
		"receipt": "FRESH_SAME_DRAW_SUCCESS_ONLY"
	},
	"sampling": "linear",
	"repeat": false,
	"blend": "normal_alpha",
	"excluded": [
		"quay_open_waterline_HOLD_NOT_REGISTERED",
		"all_fog_vfx_rows",
		"Tidebound_alias",
		"old_FogPier_UV_layout_timing",
		"actor_body_calibration_culling_and_labels",
		"shoreline_fender_new_geometry",
		"native_gameplay_animation_ripple"
	]
}
var ready: bool = false
var prepared_key: String = ''
var prepared_hash: String = ''
var canonical: Dictionary = {}
var textures: Dictionary = {}

func _same(actual: Variant, expected: Variant) -> bool:
	if typeof(expected) == TYPE_INT or typeof(expected) == TYPE_FLOAT:
		return P.number(actual,float(expected),float(expected))
	if expected is Dictionary:
		if not actual is Dictionary or actual.size() != expected.size():
			return false
		for key: Variant in expected:
			if not actual.has(key) or not _same(actual[key],expected[key]):
				return false
		return true
	if expected is Array:
		if not actual is Array or actual.size() != expected.size():
			return false
		for index: int in range(expected.size()):
			if not _same(actual[index],expected[index]):
				return false
		return true
	return typeof(actual) == typeof(expected) and actual == expected

func _read_bytes(path: String, size: int) -> PackedByteArray:
	if not FileAccess.file_exists(path):
		return PackedByteArray()
	var file: FileAccess = FileAccess.open(path,FileAccess.READ)
	if file == null:
		return PackedByteArray()
	if file.get_length() != size:
		file.close()
		return PackedByteArray()
	var bytes: PackedByteArray = file.get_buffer(size)
	file.close()
	return bytes if bytes.size() == size else PackedByteArray()

func _hash(bytes: PackedByteArray) -> String:
	var hash_value: HashingContext = HashingContext.new()
	if hash_value.start(HashingContext.HASH_SHA256) != OK or hash_value.update(bytes) != OK:
		return ''
	return hash_value.finish().hex_encode()

func _basic(snapshot_value: Variant, scene_value: Variant, key: Variant, hash_value: Variant) -> bool:
	if not snapshot_value is Dictionary or not scene_value is Dictionary or not P.snapshot(snapshot_value):
		return false
	if snapshot_value.get('entryToken') != null or snapshot_value.get('worldId') != 'mist_harbor' or snapshot_value.get('sceneId') != 'mh_drowned_quay' or not P.counter(snapshot_value.get('worldEpoch'),1):
		return false
	return typeof(key) == TYPE_STRING and key == 'mist_harbor:mh_drowned_quay:'+str(snapshot_value.worldEpoch) and typeof(hash_value) == TYPE_STRING and hash_value == DEFINITION.scene.sha256 and scene_value.get('worldId') == 'mist_harbor' and scene_value.get('sceneId') == 'mh_drowned_quay'

func prepare(snapshot_value: Variant, scene_value: Variant, key: Variant, hash_value: Variant) -> bool:
	ready = false
	prepared_key = ''
	prepared_hash = ''
	canonical.clear()
	if not _basic(snapshot_value,scene_value,key,hash_value):
		return false
	var map_bytes: PackedByteArray = _read_bytes(MAP_PATH,MAP_BYTES)
	if _hash(map_bytes) != MAP_SHA or map_bytes.get_string_from_utf8().to_utf8_buffer() != map_bytes:
		return false
	var parsed_map: Variant = JSON.parse_string(map_bytes.get_string_from_utf8())
	if not parsed_map is Dictionary or not _same(parsed_map,DEFINITION):
		return false
	var scene_bytes: PackedByteArray = _read_bytes(DEFINITION.scene.path,int(DEFINITION.scene.bytes))
	if _hash(scene_bytes) != DEFINITION.scene.sha256 or scene_bytes.get_string_from_utf8().to_utf8_buffer() != scene_bytes:
		return false
	var source_scene: Variant = JSON.parse_string(scene_bytes.get_string_from_utf8())
	if not source_scene is Dictionary or not _same(source_scene,scene_value):
		return false
	for field: String in ['walkablePolygons','collision','terrainRegions','hazards']:
		var expected_key: String = {'walkablePolygons':'walkable','collision':'walls','terrainRegions':'terrain','hazards':'hazards'}[field]
		var rows: Variant = source_scene.get(field)
		if not rows is Array or not _same(rows,DEFINITION.source[expected_key]):
			return false
		var seen: Dictionary = {}
		for row_value: Variant in rows:
			if not row_value is Dictionary:
				return false
			var id_value: Variant = row_value.get('id')
			if typeof(id_value) != TYPE_STRING or seen.has(id_value):
				return false
			seen[id_value] = true
	var presentation: Variant = source_scene.get('presentation')
	if not presentation is Dictionary or not _same(presentation.get('sprites'),DEFINITION.source.sprites):
		return false
	for texture_key: String in DEFINITION.assets:
		var asset: Dictionary = DEFINITION.assets[texture_key]
		if not textures.has(texture_key):
			var png: PackedByteArray = _read_bytes(asset.path,int(asset.bytes))
			if _hash(png) != asset.sha256:
				return false
			var image: Image = Image.new()
			if image.load_png_from_buffer(png) != OK or image.get_width() != asset.canvas[0] or image.get_height() != asset.canvas[1] or image.get_format() != Image.FORMAT_RGBA8:
				return false
			var texture: Texture2D = ImageTexture.create_from_image(image)
			if texture == null:
				return false
			textures[texture_key] = texture
		if not _texture_valid(texture_key):
			return false
	canonical = source_scene.duplicate(true)
	prepared_key = key
	prepared_hash = hash_value
	ready = true
	return true

func _texture_valid(key: String) -> bool:
	if not DEFINITION.assets.has(key):
		return false
	var value: Variant = textures.get(key)
	var asset: Dictionary = DEFINITION.assets[key]
	return value is Texture2D and is_instance_valid(value) and value.get_width() == asset.canvas[0] and value.get_height() == asset.canvas[1]

func matches_context(snapshot_value: Variant, scene_value: Variant, key: Variant, hash_value: Variant) -> bool:
	return ready and _basic(snapshot_value,scene_value,key,hash_value) and key == prepared_key and hash_value == prepared_hash and _same(scene_value,canonical) and _texture_valid('stone') and _texture_valid('fog')

func _v2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _rect_valid(rect: Rect2) -> bool:
	return _v2(rect.position) and _v2(rect.size) and _v2(rect.end) and rect.size.x > 0 and rect.size.y > 0

func _context(canvas: Variant, snapshot_value: Variant, scene_value: Variant, key: Variant, camera: Variant, viewport: Variant) -> bool:
	if not canvas is CanvasItem or not is_instance_valid(canvas) or not canvas.is_inside_tree() or canvas.is_queued_for_deletion() or not matches_context(snapshot_value,scene_value,key,prepared_hash):
		return false
	if canvas.get('scene_hash') != prepared_hash or canvas.get('current_key') != key:
		return false
	if not camera is RefCounted or not is_instance_valid(camera) or camera.get_script() != ProjectionScript or not viewport is Rect2:
		return false
	var initialized: Variant = camera.get('initialized')
	var origin: Variant = camera.get('origin')
	var center: Variant = camera.get('screen_center')
	var size: Variant = camera.get('viewport')
	if typeof(initialized) != TYPE_BOOL or not initialized or not origin is Vector3 or not center is Vector2 or not size is Vector2 or not P.number(camera.get('ppm'),0.000001,256):
		return false
	var view: Rect2 = viewport
	return is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and _v2(center) and _v2(size) and _rect_valid(view) and size == view.size

func _project(camera: Variant, world: Array[Vector3]) -> PackedVector2Array:
	var points: PackedVector2Array = PackedVector2Array()
	for vertex: Vector3 in world:
		var point: Vector2 = camera.project(vertex)
		if not _v2(point):
			return PackedVector2Array()
		points.append(point)
	if points.size() < 3:
		return PackedVector2Array()
	var area: float = 0.0
	for index: int in range(points.size()):
		area += points[index].cross(points[(index+1)%points.size()])
	return points if is_finite(area) and absf(area) > 0.000001 else PackedVector2Array()

func _colored(plan: Array[Dictionary], camera: Variant, world: Array[Vector3], color: Color) -> bool:
	var points: PackedVector2Array = _project(camera,world)
	if points.is_empty():
		return false
	plan.append({'points':points,'color':color})
	return true

func _emit(canvas: CanvasItem, plan: Array[Dictionary]) -> void:
	for command: Dictionary in plan:
		canvas.draw_colored_polygon(command.points,command.color)

func _quad(camera: Variant, bounds: Array, height: float) -> PackedVector2Array:
	var world: Array[Vector3] = [Vector3(bounds[0],height,bounds[1]),Vector3(bounds[2],height,bounds[1]),Vector3(bounds[2],height,bounds[3]),Vector3(bounds[0],height,bounds[3])]
	return _project(camera,world)

func draw_ground(canvas: Variant, snapshot_value: Variant, scene_value: Variant, key: Variant, camera: Variant, viewport: Variant) -> bool:
	if not _context(canvas,snapshot_value,scene_value,key,camera,viewport):
		return false
	var plan: Array[Dictionary] = []
	var a: Vector2 = Vector2(820,240)
	var ab: Vector2 = Vector2(212,102)
	var ad: Vector2 = Vector2(-640,285)
	for row: int in range(4):
		for column: int in range(6):
			var x0: float = 0.5+column*4.0
			var z0: float = 0.5+row*4.0
			var x1: float = minf(23.5,x0+4.0)
			var z1: float = minf(15.5,z0+4.0)
			var points: PackedVector2Array = _quad(camera,[x0,z0,x1,z1],0)
			if points.is_empty():
				return false
			var u: float = (x1-x0)/4.0
			var v: float = (z1-z0)/4.0
			var uv: PackedVector2Array = PackedVector2Array()
			for local: Vector2 in [Vector2(0,0),Vector2(u,0),Vector2(u,v),Vector2(0,v)]:
				var sample: Vector2 = (a+ab*local.x+ad*local.y)/Vector2(1228,1254)
				if not _v2(sample) or sample.x < 0 or sample.y < 0 or sample.x > 1 or sample.y > 1:
					return false
				uv.append(sample)
			plan.append({'points':points,'uv':uv})
	var tint: Color = Color(0.42,0.54,0.58,1)
	var texture: Texture2D = textures.stone
	for tile: Dictionary in plan:
		canvas.draw_polygon(tile.points,PackedColorArray([tint,tint,tint,tint]),tile.uv,texture)
	return true

func draw_wall(canvas: Variant, snapshot_value: Variant, scene_value: Variant, key: Variant, camera: Variant, wall_value: Variant, viewport: Variant) -> bool:
	if not _context(canvas,snapshot_value,scene_value,key,camera,viewport) or not wall_value is Dictionary:
		return false
	var id_value: Variant = wall_value.get('id')
	if typeof(id_value) != TYPE_STRING:
		return false
	var found: bool = false
	for row: Variant in DEFINITION.source.walls:
		if row is Dictionary and row.get('id') == id_value and _same(row,wall_value):
			found = true
	if not found:
		return false
	var contact: Array[Vector3] = []
	var cap: Array[Vector3] = []
	for pair: Array in wall_value.polygon:
		contact.append(Vector3(pair[0],0,pair[1]))
		cap.append(Vector3(pair[0],0.25,pair[1]))
	var plan: Array[Dictionary] = []
	if not _colored(plan,camera,contact,Color(0.025,0.032,0.035,0.10)):
		return false
	for index: int in range(4):
		var x: Vector3 = contact[index]
		var y: Vector3 = contact[(index+1)%4]
		var world: Array[Vector3] = [x,y,Vector3(y.x,0.25,y.z),Vector3(x.x,0.25,x.z)]
		if not _colored(plan,camera,world,Color('#33494f')):
			return false
	if not _colored(plan,camera,cap,Color('#58717a')):
		return false
	_emit(canvas,plan)
	return true

func draw_water(canvas: Variant, snapshot_value: Variant, scene_value: Variant, key: Variant, camera: Variant, region: Variant, viewport: Variant) -> bool:
	if not _context(canvas,snapshot_value,scene_value,key,camera,viewport) or not region is Dictionary or not _same(region,DEFINITION.source.terrain[0]):
		return false
	var plan: Array[Dictionary] = []
	for style: Dictionary in DEFINITION.water.style:
		var points: PackedVector2Array = _quad(camera,style.rect,0)
		if points.is_empty():
			return false
		var color: Color = Color(style.color)
		color.a = float(style.alpha)
		plan.append({'points':points,'color':color})
	_emit(canvas,plan)
	return true

func _bank_row(row: Variant) -> bool:
	if not row is Dictionary or row.get('id') != 'quay_north_fog_bank':
		return false
	for source_row: Variant in DEFINITION.source.sprites:
		if source_row is Dictionary and source_row.get('id') == 'quay_north_fog_bank':
			return _same(row,source_row)
	return false

func bank_rect(camera: Variant) -> Rect2:
	var center: Vector2 = camera.project(Vector3(19,0,3))
	var size: Vector2 = Vector2(6.0*float(camera.get('ppm')),6.0*float(camera.get('ppm'))*498.0/1201.0)
	return Rect2(center-size*0.5,size)

func draw_back_bank(canvas: Variant, snapshot_value: Variant, scene_value: Variant, key: Variant, camera: Variant, viewport: Variant) -> bool:
	if not _context(canvas,snapshot_value,scene_value,key,camera,viewport):
		return false
	var row_found: bool = false
	for row: Variant in canonical.presentation.sprites:
		if _bank_row(row):
			row_found = true
	if not row_found:
		return false
	var rect: Rect2 = bank_rect(camera)
	var view: Rect2 = viewport
	if not _rect_valid(rect) or not view.intersects(rect):
		return false
	var texture: Texture2D = textures.fog
	canvas.draw_texture_rect_region(texture,rect,Rect2(27,370,1201,498),Color(0.8,0.88,0.92,0.22),false,true)
	return true

# The caller supplies only its fresh local successful draw IDs from this _draw.
# No texture cache or previous-frame receipt grants suppression.
func legacy_bank_consumed(canvas: Variant, prop: Variant, snapshot_value: Variant, scene_value: Variant, key: Variant, camera: Variant, viewport: Variant, drawn_ids: Variant) -> bool:
	if not _context(canvas,snapshot_value,scene_value,key,camera,viewport) or not _bank_row(prop) or not drawn_ids is Array:
		return false
	var count: int = 0
	for id_value: Variant in drawn_ids:
		if typeof(id_value) == TYPE_STRING and id_value == 'quay_north_fog_bank':
			count += 1
	var rect: Rect2 = bank_rect(camera)
	var view: Rect2 = viewport
	return count == 1 and _rect_valid(rect) and view.intersects(rect)
