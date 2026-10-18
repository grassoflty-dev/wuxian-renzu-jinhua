extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const FurnaceScript = preload('res://scripts/world/cw_furnace_visual.gd')
const MAP_PATH: String = 'res://data/cw_forged_guard_arena_environment.internal-use.json'
const MAP_BYTES: int = 8027
const MAP_SHA: String = 'ea25389d03d76821e639a98b3e4731f9a3d812a4d2ffe9b67f74b617ecb5ea38'
const DEFINITION: Dictionary = {
	"schema": "wuxian.cw-forged-guard-arena-environment.internal-use/v1",
	"registeredSourceReuse": true,
	"approval": "INTERNAL_REGISTERED_SOURCE_REUSE",
	"GodotDisplay": "NOT_RUN",
	"publicRelease": "NOT_ESTABLISHED",
	"worldId": "clockworks",
	"sceneId": "cw_forged_guard_arena",
	"roomUse": "MAIN_EXPLICIT_ARENA_ONLY_INTERNAL_USE",
	"scene": {
		"path": "res://data/scenes/cw_forged_guard_arena.json",
		"bytes": 6794,
		"sha256": "38ae047e46e980f953377121c78713a79b69f84b994e2dd1984e1404fe77ccf5"
	},
	"upstream": {
		"steel": {
			"path": "res://data/cw_pressure_floor_material.internal-use.json",
			"bytes": 1391,
			"sha256": "746dbb35f40af89c300d1043322f634663a1f58bb6130a8711a84c972b35015a"
		},
		"furnaces": {
			"path": "res://data/cw_furnaces.internal-use.json",
			"bytes": 1670,
			"sha256": "c797ea5392842aeb0427df01bad7b8562b2e626b05398df6e624c04d88a480d6"
		},
		"idle": {
			"path": "res://data/cenyao_idle.internal-use.json",
			"bytes": 3925,
			"sha256": "b063efd36046834c41ebed05c12ffb8729926f7d4edff0e15d36dccdbb1da881"
		}
	},
	"asset": {
		"assetId": "runtime2d.world.clockworks.floor_steel.v1",
		"path": "res://assets/props/clockworks/pressure_hall_steel_floor_source.png",
		"bytes": 1610704,
		"sha256": "0b4555021799ac849756e0f11c207d4e9309df19f4b8f68d0f4e8177f5204043",
		"canvas": [
			1197,
			1132
		],
		"format": "RGBA8",
		"sourceSha256": "1a01e851654d9f5c98b5486cbe489f357ed99c55ffc601c492af63044d90b190"
	},
	"selection": {
		"kind": "affine_top_plane",
		"sourcePixelEdgeCorners": [
			[
				510,
				200
			],
			[
				970,
				460
			],
			[
				600,
				740
			],
			[
				140,
				480
			]
		],
		"uvConvention": "sourcePixelEdge / standaloneCanvasSize"
	},
	"floor": {
		"row": {
			"assetId": "runtime2d.world.clockworks.floor_steel.v1",
			"id": "cw_forged_guard_arena_steel_floor",
			"layer": "visual.floor",
			"position": [
				12,
				0,
				8
			]
		},
		"bounds": [
			0,
			0,
			24,
			16
		],
		"groundY": 0,
		"patchSize": [
			4,
			4
		],
		"grid": [
			6,
			4
		],
		"patchCount": 24,
		"tint": [
			0.52,
			0.55,
			0.57,
			1
		],
		"sampling": "linear",
		"repeat": false,
		"blend": "normal_alpha",
		"scaleBasis": "ART_DIRECTION_NOT_SOURCE_MEASURED"
	},
	"furnace": {
		"row": {
			"assetId": "runtime2d.world.clockworks.furnace.v2",
			"id": "cw_arena_furnace",
			"layer": "visual.props_back",
			"position": [
				18,
				0,
				5
			]
		},
		"key": "furnace_v2",
		"canvas": [
			1165,
			1240
		],
		"anchor": [
			0.5,
			1
		],
		"heightM": 4,
		"sourceRuntimeStatus": "ADOPT",
		"sourceAdmission": "release_approved",
		"state": "BAKED_FIRE_GAUGES_STATIC_NOT_LIVE_HEAT"
	},
	"walls": {
		"cw_forged_guard_arena_wall_north": {
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
			],
			"height": 0.9
		},
		"cw_forged_guard_arena_wall_south": {
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
			],
			"height": 0.18
		},
		"cw_forged_guard_arena_wall_west_north": {
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
			],
			"height": 0.9
		},
		"cw_forged_guard_arena_wall_west_south": {
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
			],
			"height": 0.9
		},
		"cw_forged_guard_arena_wall_east_north": {
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
			],
			"height": 0.18
		},
		"cw_forged_guard_arena_wall_east_south": {
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
			],
			"height": 0.18
		}
	},
	"excludedBlockers": [
		{
			"id": "cw_arena_shutter_west_blocker",
			"polygon": [
				[
					0.5,
					6
				],
				[
					1,
					6
				],
				[
					1,
					9.5
				],
				[
					0.5,
					9.5
				]
			],
			"requiresActorFirstKill": "cw_forged_guard_elite"
		},
		{
			"id": "cw_arena_shutter_east_blocker",
			"polygon": [
				[
					23,
					6
				],
				[
					23.5,
					6
				],
				[
					23.5,
					9.5
				],
				[
					23,
					9.5
				]
			],
			"requiresActorFirstKill": "cw_forged_guard_elite"
		}
	],
	"allAuthoredSprites": [
		{
			"assetId": "runtime2d.world.clockworks.floor_steel.v1",
			"id": "cw_forged_guard_arena_steel_floor",
			"layer": "visual.floor",
			"position": [
				12,
				0,
				8
			]
		},
		{
			"assetId": "runtime2d.world.clockworks.factory_tiles.v1",
			"id": "cw_forged_guard_arena_factory_tiles",
			"layer": "visual.floor",
			"position": [
				12,
				0,
				8
			]
		},
		{
			"assetId": "runtime2d.world.clockworks.pressure_pipes.v1",
			"id": "cw_arena_forge_pipes",
			"layer": "visual.props_back",
			"position": [
				11,
				0,
				4
			]
		},
		{
			"assetId": "runtime2d.world.clockworks.furnace.v2",
			"id": "cw_arena_furnace",
			"layer": "visual.props_back",
			"position": [
				18,
				0,
				5
			]
		},
		{
			"assetId": "runtime2d.world.clockworks.factory_tiles.v1",
			"id": "cw_arena_factory_module",
			"layer": "visual.props_dynamic",
			"position": [
				8,
				0,
				12
			]
		},
		{
			"assetId": "runtime2d.world.clockworks.factory_tiles.v1",
			"id": "cw_arena_factory_module_side",
			"layer": "visual.props_dynamic",
			"position": [
				20,
				0,
				12
			]
		}
	],
	"wallStyle": {
		"backFace": "#303a3d",
		"frontFace": "#414b4e",
		"cap": "#596467",
		"contact": [
			0.025,
			0.032,
			0.035,
			0.12
		],
		"displayHeightBasis": "ART_DIRECTION_ONLY_NO_COLLISION_CHANGE"
	},
	"cues": {
		"center": [
			18,
			0,
			5
		],
		"vertices": 24,
		"contact": {
			"radii": [
				1.1,
				0.25
			],
			"color": [
				0.025,
				0.032,
				0.035,
				0.12
			]
		},
		"warm": [
			{
				"radii": [
					1.6,
					0.72
				],
				"alpha": 0.012
			},
			{
				"radii": [
					1,
					0.45
				],
				"alpha": 0.018
			},
			{
				"radii": [
					0.55,
					0.25
				],
				"alpha": 0.028
			}
		],
		"warmColor": "#9f784f",
		"condition": "CURRENT_EXACT_FURNACE_V2_SELECTOR_AND_NORMAL_VALID_CACHE",
		"staticReducedMotionIdentical": true,
		"blend": "normal_alpha",
		"physicalFeetMeasured": false
	},
	"protected": [
		"canonical_geometry",
		"both_first_kill_blockers",
		"old_seven_furnace_rows",
		"all_body_calibration_and_sorting",
		"no_new_assets",
		"no_animation_or_state",
		"no_qualified_pipe_factory_elite_art_inferred"
	]
}
var ready: bool = false
var prepared_key: String = ''
var prepared_hash: String = ''
var canonical: Dictionary = {}
var steel_texture: Texture2D = null

# Numeric contracts compare values, not JSON serialization or Array hash types.
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

func _read_bytes(path: String, length: int) -> PackedByteArray:
	if not FileAccess.file_exists(path):
		return PackedByteArray()
	var file: FileAccess = FileAccess.open(path,FileAccess.READ)
	if file == null:
		return PackedByteArray()
	if file.get_length() != length:
		file.close()
		return PackedByteArray()
	var bytes: PackedByteArray = file.get_buffer(length)
	file.close()
	return bytes if bytes.size() == length else PackedByteArray()

func _hash(bytes: PackedByteArray) -> String:
	var hashing: HashingContext = HashingContext.new()
	if hashing.start(HashingContext.HASH_SHA256) != OK or hashing.update(bytes) != OK:
		return ''
	return hashing.finish().hex_encode()

func _basic(snapshot_value: Variant, scene_value: Variant, key: Variant, hash_value: Variant) -> bool:
	if not snapshot_value is Dictionary or not scene_value is Dictionary or not P.snapshot(snapshot_value):
		return false
	if snapshot_value.get('entryToken') != null or snapshot_value.get('worldId') != 'clockworks' or snapshot_value.get('sceneId') != 'cw_forged_guard_arena' or not P.counter(snapshot_value.get('worldEpoch'),1):
		return false
	return typeof(key) == TYPE_STRING and key == 'clockworks:cw_forged_guard_arena:'+str(snapshot_value.worldEpoch) and typeof(hash_value) == TYPE_STRING and hash_value == DEFINITION.scene.sha256 and scene_value.get('worldId') == 'clockworks' and scene_value.get('sceneId') == 'cw_forged_guard_arena'

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
	if not source_scene is Dictionary or not _same(scene_value,source_scene):
		return false
	# The SHA binds the complete canonical definition; selected rows remain exact.
	var source_collision: Variant = source_scene.get('collision')
	var presentation: Variant = source_scene.get('presentation')
	if not source_collision is Array or source_collision.size() != 8 or not presentation is Dictionary or not _same(presentation.get('sprites'),DEFINITION.allAuthoredSprites):
		return false
	var seen: Dictionary = {}
	for row_value: Variant in source_collision:
		if not row_value is Dictionary:
			return false
		var id_value: Variant = row_value.get('id')
		if typeof(id_value) != TYPE_STRING or seen.has(id_value):
			return false
		seen[id_value] = true
		if DEFINITION.walls.has(id_value):
			if not _same(row_value.get('polygon'),DEFINITION.walls[id_value].polygon):
				return false
		elif not DEFINITION.excludedBlockers.any(func(v: Variant) -> bool: return _same(row_value,v)):
			return false
	if steel_texture == null:
		var png: PackedByteArray = _read_bytes(DEFINITION.asset.path,int(DEFINITION.asset.bytes))
		if _hash(png) != DEFINITION.asset.sha256:
			return false
		var image: Image = Image.new()
		if image.load_png_from_buffer(png) != OK or image.get_width() != 1197 or image.get_height() != 1132 or image.get_format() != Image.FORMAT_RGBA8:
			return false
		steel_texture = ImageTexture.create_from_image(image)
	if steel_texture == null or steel_texture.get_width() != 1197 or steel_texture.get_height() != 1132:
		return false
	canonical = source_scene.duplicate(true)
	prepared_key = key
	prepared_hash = hash_value
	ready = true
	return true

func matches_context(snapshot_value: Variant, scene_value: Variant, key: Variant, hash_value: Variant) -> bool:
	return ready and steel_texture != null and _basic(snapshot_value,scene_value,key,hash_value) and key == prepared_key and hash_value == prepared_hash and _same(scene_value,canonical)

func furnace_row(scene_value: Variant) -> Dictionary:
	if not scene_value is Dictionary or not _same(scene_value,canonical):
		return {}
	var presentation: Variant = scene_value.get('presentation')
	if not presentation is Dictionary:
		return {}
	var rows: Variant = presentation.get('sprites')
	if not rows is Array:
		return {}
	for row: Variant in rows:
		if row is Dictionary and row.get('id') == 'cw_arena_furnace' and _same(row,DEFINITION.furnace.row):
			return row
	return {}

func _v2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _context(canvas: Variant, snapshot_value: Variant, scene_value: Variant, key: Variant, camera: Variant, viewport: Variant) -> bool:
	if not canvas is CanvasItem or not is_instance_valid(canvas) or not canvas.is_inside_tree() or canvas.is_queued_for_deletion() or not matches_context(snapshot_value,scene_value,key,prepared_hash):
		return false
	if canvas.get('scene_hash') != prepared_hash or canvas.get('current_key') != key:
		return false
	if not camera is RefCounted or not is_instance_valid(camera) or camera.get_script() != ProjectionScript or not viewport is Rect2:
		return false
	var origin: Variant = camera.get('origin')
	var center: Variant = camera.get('screen_center')
	var size: Variant = camera.get('viewport')
	var initialized: Variant = camera.get('initialized')
	if not origin is Vector3 or not center is Vector2 or not size is Vector2 or typeof(initialized) != TYPE_BOOL or not initialized or not P.number(camera.get('ppm'),0.000001,256):
		return false
	var view: Rect2 = viewport
	return is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and _v2(center) and _v2(size) and _v2(view.position) and _v2(view.size) and _v2(view.end) and view.size.x > 0 and view.size.y > 0 and size == view.size

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

func _color_plan(plan: Array[Dictionary], camera: Variant, world: Array[Vector3], color: Color) -> bool:
	var points: PackedVector2Array = _project(camera,world)
	if points.is_empty():
		return false
	plan.append({'points':points,'color':color})
	return true

func _emit(canvas: CanvasItem, plan: Array[Dictionary]) -> void:
	for command: Dictionary in plan:
		canvas.draw_colored_polygon(command.points,command.color)

func draw_floor(canvas: Variant, snapshot_value: Variant, scene_value: Variant, key: Variant, camera: Variant, viewport: Variant, furnace: Variant, furnace_key: Variant) -> bool:
	if not _context(canvas,snapshot_value,scene_value,key,camera,viewport):
		return false
	var tiles: Array[Dictionary] = []
	var corners: Array[Vector2] = [Vector2(510,200),Vector2(970,460),Vector2(600,740),Vector2(140,480)]
	var uv: PackedVector2Array = PackedVector2Array()
	for edge: Vector2 in corners:
		uv.append(edge/Vector2(1197,1132))
	for row: int in range(4):
		for column: int in range(6):
			var x: float = column*4.0
			var z: float = row*4.0
			var world: Array[Vector3] = [Vector3(x,0,z),Vector3(x+4,0,z),Vector3(x+4,0,z+4),Vector3(x,0,z+4)]
			var points: PackedVector2Array = _project(camera,world)
			if points.is_empty():
				return false
			tiles.append({'points':points,'uv':uv})
	var cues: Array[Dictionary] = []
	if furnace_key == 'furnace_v2' and furnace is RefCounted and is_instance_valid(furnace) and furnace.get_script() == FurnaceScript:
		var textures: Variant = furnace.get('textures')
		if textures is Dictionary:
			var texture: Variant = textures.get('furnace_v2')
			if texture is Texture2D and is_instance_valid(texture) and texture.get_width() == 1165 and texture.get_height() == 1240:
				var profiles: Array[Vector3] = [Vector3(1.1,0.25,0.12),Vector3(1.6,0.72,0.012),Vector3(1.0,0.45,0.018),Vector3(0.55,0.25,0.028)]
				for i: int in range(profiles.size()):
					var radius: Vector3 = profiles[i]
					var world: Array[Vector3] = []
					for vertex: int in range(24):
						var angle: float = TAU*float(vertex)/24.0
						world.append(Vector3(18+cos(angle)*radius.x,0,5+sin(angle)*radius.y))
					var color: Color = Color(0.025,0.032,0.035,radius.z) if i == 0 else Color(159.0/255.0,120.0/255.0,79.0/255.0,radius.z)
					if not _color_plan(cues,camera,world,color):
						return false
	var tint: Color = Color(0.52,0.55,0.57,1)
	for tile: Dictionary in tiles:
		canvas.draw_polygon(tile.points,PackedColorArray([tint,tint,tint,tint]),tile.uv,steel_texture)
	_emit(canvas,cues)
	return true

func draw_wall(canvas: Variant, snapshot_value: Variant, scene_value: Variant, key: Variant, camera: Variant, wall_value: Variant, viewport: Variant) -> bool:
	if not _context(canvas,snapshot_value,scene_value,key,camera,viewport) or not wall_value is Dictionary:
		return false
	var id_value: Variant = wall_value.get('id')
	if typeof(id_value) != TYPE_STRING or not DEFINITION.walls.has(id_value):
		return false
	var selected: Dictionary = DEFINITION.walls[id_value]
	if wall_value.size() != 2 or not _same(wall_value.get('polygon'),selected.polygon):
		return false
	var member: bool = false
	for row: Variant in canonical.collision:
		if row is Dictionary and row.get('id') == id_value and _same(row,wall_value):
			member = true
	if not member:
		return false
	var footprint: Array = selected.polygon
	var height: float = float(selected.height)
	var face: Color = Color('#303a3d') if height > 0.5 else Color('#414b4e')
	var contact: Array[Vector3] = []
	var cap: Array[Vector3] = []
	for pair: Array in footprint:
		contact.append(Vector3(pair[0],0,pair[1]))
		cap.append(Vector3(pair[0],height,pair[1]))
	var west_join: bool = id_value == 'cw_forged_guard_arena_wall_west_south'
	var east_join: bool = id_value == 'cw_forged_guard_arena_wall_east_south'
	var south: Dictionary = DEFINITION.walls['cw_forged_guard_arena_wall_south']
	var south_height: float = float(south.height)
	var join_z: float = float(south.polygon[0][1])
	var clipped_z: float = join_z-south_height
	var outer_x: float = float(south.polygon[1][0])
	var clipped_x: float = outer_x-south_height
	var extra_contact: Array[Vector3] = []
	if west_join or east_join:
		# Clip only footprint fragments whose view ray reaches the south cap.
		contact[2] = Vector3(contact[2].x,0,clipped_z)
		contact[3] = Vector3(contact[3].x,0,clipped_z)
		if east_join:
			# The outboard end strip is outside that cap and must remain visible.
			extra_contact = [Vector3(clipped_x,0,clipped_z),Vector3(outer_x,0,clipped_z),Vector3(outer_x,0,join_z),Vector3(clipped_x,0,join_z)]
	var plan: Array[Dictionary] = []
	if not _color_plan(plan,camera,contact,Color(0.025,0.032,0.035,0.12)):
		return false
	if east_join and not _color_plan(plan,camera,extra_contact,Color(0.025,0.032,0.035,0.12)):
		return false
	for index: int in range(4):
		# Side/cap endpoints come from the canonical footprint, not clipped contact.
		var a: Vector3 = Vector3(footprint[index][0],0,footprint[index][1])
		var b: Vector3 = Vector3(footprint[(index+1)%4][0],0,footprint[(index+1)%4][1])
		var side: Array[Vector3] = [a,b,Vector3(b.x,height,b.z),Vector3(a.x,height,a.z)]
		if west_join:
			if index == 1:
				side = [a,Vector3(b.x,0,clipped_z),Vector3(b.x,south_height,join_z),Vector3(b.x,height,b.z),Vector3(a.x,height,a.z)]
			elif index == 2:
				side = [Vector3(a.x,south_height,join_z),Vector3(b.x,south_height,join_z),Vector3(b.x,height,b.z),Vector3(a.x,height,a.z)]
			elif index == 3:
				side = [Vector3(a.x,0,clipped_z),b,Vector3(b.x,height,b.z),Vector3(a.x,height,a.z),Vector3(a.x,south_height,join_z)]
		elif east_join:
			if index == 2:
				# Only the outward triangle of the end face is beyond the south cap.
				side = [a,Vector3(clipped_x,0,join_z),Vector3(outer_x,height,join_z)]
			elif index == 3:
				side = [Vector3(a.x,0,clipped_z),b,Vector3(b.x,height,b.z),Vector3(a.x,height,a.z)]
		if not _color_plan(plan,camera,side,face):
			return false
	if not _color_plan(plan,camera,cap,Color('#596467')):
		return false
	_emit(canvas,plan)
	return true

