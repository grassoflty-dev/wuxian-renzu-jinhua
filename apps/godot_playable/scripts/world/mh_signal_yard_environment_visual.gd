extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const DEFINITION: Dictionary = {
  "schema": "wuxian.mh-signal-yard-environment.internal-use/v1",
  "registeredSourceReuse": true,
  "approval": "INTERNAL_REGISTERED_SOURCE_REUSE",
  "GodotDisplay": "NOT_RUN",
  "publicRelease": "NOT_ESTABLISHED",
  "worldId": "mist_harbor",
  "sceneId": "mh_signal_yard",
  "references": {
    "stoneDefinition": "res://data/mh_fog_pier_stone_material.internal-use.json",
    "stoneDefinitionSha256": "c56ad7549a49809341e2aea6efcbe7fbe0477eb51026e776c4a49fa03077585a",
    "fogDefinition": "res://data/mh_fog_pier_layered_fog.internal-use.json",
    "fogDefinitionSha256": "027980b44e5f92aa5e7759611eeffe69d6ae59be90a4ae4e0858a96123d2bee0"
  },
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
    "sourceSha256": "dc09452bf3f55c57cca1a93bb09f169ef441af90321dff47d91c4be7e62d3a83",
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
    "uvConvention": "sourcePixelEdge / standaloneCanvasSize",
    "rowId": "approved_stone_water_surface",
    "layer": "visual.floor",
    "position": [
      12,
      0,
      8
    ],
    "walkableId": "mh_signal_yard_walkable_area",
    "walkablePolygon": [
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
    ],
    "patchSizeM": [
      2,
      6
    ],
    "grid": [
      12,
      3
    ],
    "patchCount": 36,
    "groundY": 0,
    "tint": [
      0.46,
      0.55,
      0.6,
      1
    ],
    "seamlessTileability": "NOT_ESTABLISHED"
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
    "cropXYWH": [
      27,
      370,
      1201,
      498
    ],
    "uvConvention": "sourcePixelEdge / standaloneCanvasSize",
    "rowId": "yard_north_fog_bank",
    "layer": "visual.props_back",
    "position": [
      8,
      0,
      3
    ],
    "widthPerPpm": 10,
    "centerOffsetPerPpm": [
      0,
      -1
    ],
    "modulate": [
      0.78,
      0.86,
      0.9,
      0.18
    ],
    "sampling": "linear",
    "repeat": false,
    "blend": "normal-alpha",
    "temporal": "STATIC"
  },
  "air": {
    "kind": "bounded_decorative_interference_air",
    "hazardId": "mh_signal_interference_region",
    "sourceKind": "signal_interference_zone",
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
        10
      ],
      [
        13,
        10
      ]
    ],
    "ribbonZ": [
      5.7,
      7.5,
      9.3
    ],
    "xRange": [
      13.35,
      18.65
    ],
    "heightCapM": 0.4,
    "perRibbonLayers": 2,
    "polygonCap": 6,
    "verticesCap": 10,
    "rgb": [
      0.52,
      0.64,
      0.69
    ],
    "layerAlpha": [
      0.04,
      0.055
    ],
    "temporal": "NATIVE_TICK_WARP_WITH_EXACT_STATIC_FALLBACK",
    "motion": {
      "periodTicks": 360,
      "nominalTickHz": 60,
      "maxOffsetXM": 0.06,
      "maxTopInsetM": 0.02,
      "phasePerVertexRad": 0.7,
      "phasePerRibbonRad": 0.8,
      "phasePerLayerRad": 0.3,
      "xEnvelopeM": [
        13.29,
        18.71
      ],
      "zEnvelopeM": [
        5.7,
        9.38
      ],
      "reducedMotion": "EXACT_STATIC_GEOMETRY",
      "alphaMotion": "NONE",
      "scope": "EXISTING_SIX_AIR_POLYGONS_ONLY"
    }
  }
}
const WALLS: Dictionary = {
 'yard_wall_north':[[[0,0],[24,0],[24,0.5],[0,0.5]],[0,0.5],[24,0.5],[0,-1],0.8],
 'yard_wall_south':[[[0,15.5],[24,15.5],[24,16],[0,16]],[0,15.5],[24,15.5],[0,1],0.18],
 'yard_wall_west_north':[[[0,0.5],[0.5,0.5],[0.5,6],[0,6]],[0.5,0.5],[0.5,6],[-1,0],0.8],
 'yard_wall_west_south':[[[0,9.5],[0.5,9.5],[0.5,15.5],[0,15.5]],[0.5,9.5],[0.5,15.5],[-1,0],0.8],
 'yard_wall_east_north':[[[23.5,0.5],[24,0.5],[24,6],[23.5,6]],[23.5,0.5],[23.5,6],[1,0],0.18],
 'yard_wall_east_south':[[[23.5,9.5],[24,9.5],[24,15.5],[23.5,15.5]],[23.5,9.5],[23.5,15.5],[1,0],0.18]
}
const OUTER_FACES: Array[Dictionary] = [
 {'id':'yard_wall_north','a':[24,0],'b':[24,0.5],'low_from':''},
 {'id':'yard_wall_south','a':[0,16],'b':[24,16],'low_from':''},
 {'id':'yard_wall_south','a':[24,15.5],'b':[24,16],'low_from':''},
 {'id':'yard_wall_west_north','a':[0,6],'b':[0.5,6],'low_from':''},
 {'id':'yard_wall_west_south','a':[0,15.5],'b':[0.5,15.5],'low_from':'yard_wall_south'},
 {'id':'yard_wall_east_north','a':[24,0.5],'b':[24,6],'low_from':''},
 {'id':'yard_wall_east_north','a':[23.5,6],'b':[24,6],'low_from':''},
 {'id':'yard_wall_east_south','a':[24,9.5],'b':[24,15.5],'low_from':''}
]
const OUTER_SCENE_BYTES: int = 8327
const OUTER_SCENE_SHA256: String = '14f708142f6d4c15b8abdb3671e5357f845d96228c5b9aed719e19895e3a66d8'
const SPRITES: Dictionary = {
 'approved_stone_water_surface':['runtime2d.world.mistharbor.water_edge_stone.v1','visual.floor',[12,0,8]],
 'yard_north_fog_bank':['runtime2d.world.mistharbor.fog_bank.v1','visual.props_back',[8,0,3]],
 'yard_south_water_edge':['runtime2d.world.mistharbor.water_edge.v1','visual.props_back',[12,0,14.5]]
}
var attempted: bool = false
var definition_ready: bool = false
var textures: Dictionary = {}
var _outer_scene_attempted: bool = false
var _outer_scene: Dictionary = {}

func _equal(actual: Variant, expected: Variant) -> bool:
	if expected is Dictionary:
		if not actual is Dictionary or actual.size()!=expected.size():
			return false
		for key: Variant in expected:
			if not actual.has(key) or not _equal(actual[key],expected[key]):
				return false
		return true
	if expected is Array:
		if not actual is Array or actual.size()!=expected.size():
			return false
		for index: int in range(expected.size()):
			if not _equal(actual[index],expected[index]):
				return false
		return true
	if typeof(expected) in [TYPE_INT,TYPE_FLOAT]:
		return P.number(actual,float(expected),float(expected))
	return typeof(actual)==typeof(expected) and actual==expected

func _layout(scene: Dictionary) -> bool:
	if scene.get('worldId')!='mist_harbor' or scene.get('sceneId')!='mh_signal_yard' or not P.counter(scene.get('schemaVersion'),1,1):
		return false
	if not _equal(scene.get('boundsM'),{'x':0,'z':0,'width':24,'depth':16}):
		return false
	var walk_value: Variant = scene.get('walkablePolygons')
	if not walk_value is Array or walk_value.size()!=1:
		return false
	var walk_row: Variant = walk_value[0]
	if not walk_row is Dictionary or walk_row.get('id')!='mh_signal_yard_walkable_area' or not _equal(walk_row.get('polygon'),DEFINITION.stone.walkablePolygon):
		return false
	var collisions: Variant = scene.get('collision')
	if not collisions is Array or collisions.size()!=6:
		return false
	var seen: Dictionary = {}
	for item: Variant in collisions:
		if not item is Dictionary:
			return false
		var row: Dictionary = item
		var id_value: Variant = row.get('id')
		if typeof(id_value)!=TYPE_STRING or not WALLS.has(id_value) or seen.has(id_value):
			return false
		var wall_definition: Array = WALLS[id_value]
		if not _equal(row.get('polygon'),wall_definition[0]):
			return false
		seen[id_value] = true
	var presentation_value: Variant = scene.get('presentation')
	if not presentation_value is Dictionary:
		return false
	var sprites: Variant = presentation_value.get('sprites')
	if not sprites is Array or sprites.size()!=3:
		return false
	seen.clear()
	for item: Variant in sprites:
		if not item is Dictionary:
			return false
		var row: Dictionary = item
		var id_value: Variant = row.get('id')
		if typeof(id_value)!=TYPE_STRING or not SPRITES.has(id_value) or seen.has(id_value):
			return false
		var wanted: Array = SPRITES[id_value]
		if row.get('assetId')!=wanted[0] or row.get('layer')!=wanted[1] or not _equal(row.get('position'),wanted[2]):
			return false
		seen[id_value] = true
	var layers: Variant = presentation_value.get('layers')
	if not layers is Array or layers.size()!=4:
		return false
	var expected_layers: Dictionary = {'visual.floor':0,'visual.props_back':10,'visual.props_dynamic':20,'visual.vfx_markers':30}
	seen.clear()
	for item: Variant in layers:
		if not item is Dictionary:
			return false
		var id_value: Variant = item.get('id')
		if typeof(id_value)!=TYPE_STRING or not expected_layers.has(id_value) or seen.has(id_value) or not P.counter(item.get('zGroup'),expected_layers[id_value],expected_layers[id_value]):
			return false
		seen[id_value] = true
	var source_hazards: Variant = scene.get('hazards')
	if not source_hazards is Array or source_hazards.size()!=1:
		return false
	var hazard: Variant = source_hazards[0]
	return hazard is Dictionary and hazard.get('id')==DEFINITION.air.hazardId and hazard.get('kind')==DEFINITION.air.sourceKind and _equal(hazard.get('polygon'),DEFINITION.air.polygon)

func _base(snapshot_value: Variant, scene_value: Variant, key: Variant) -> bool:
	if not snapshot_value is Dictionary or not scene_value is Dictionary or not P.snapshot(snapshot_value) or snapshot_value.get('entryToken')!=null:
		return false
	if snapshot_value.get('worldId')!='mist_harbor' or snapshot_value.get('sceneId')!='mh_signal_yard' or not P.counter(snapshot_value.get('worldEpoch'),1):
		return false
	return typeof(key)==TYPE_STRING and key=='mist_harbor:mh_signal_yard:'+str(snapshot_value.worldEpoch) and _layout(scene_value)

func _read(path: String, minimum: int, maximum: int) -> PackedByteArray:
	var file: FileAccess = FileAccess.open(path,FileAccess.READ)
	if file==null:
		return PackedByteArray()
	var length: int = file.get_length()
	if length<minimum or length>maximum:
		file.close()
		return PackedByteArray()
	var bytes: PackedByteArray = file.get_buffer(length)
	file.close()
	return bytes if bytes.size()==length else PackedByteArray()

func _sha(bytes: PackedByteArray) -> String:
	var hash: HashingContext = HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256)!=OK:
		return ''
	hash.update(bytes)
	return hash.finish().hex_encode()

func _texture(definition: Dictionary) -> Texture2D:
	var bytes: PackedByteArray = _read(definition.path,int(definition.bytes),int(definition.bytes))
	if bytes.is_empty() or _sha(bytes)!=definition.sha256:
		return null
	var image: Image = Image.new()
	if image.load_png_from_buffer(bytes)!=OK or image.get_width()!=int(definition.canvas[0]) or image.get_height()!=int(definition.canvas[1]) or image.get_format()!=Image.FORMAT_RGBA8:
		return null
	var texture: ImageTexture = ImageTexture.create_from_image(image)
	image = null
	return texture

func prepare(snapshot: Variant, scene: Variant, key: Variant) -> bool:
	if not _base(snapshot,scene,key):
		return false
	if attempted:
		return definition_ready
	attempted = true
	var bytes: PackedByteArray = _read('res://data/mh_signal_yard_environment.internal-use.json',1,16384)
	if bytes.is_empty() or bytes.get_string_from_utf8().to_utf8_buffer()!=bytes:
		return false
	var parsed: Variant = JSON.parse_string(bytes.get_string_from_utf8())
	if not parsed is Dictionary or not _equal(parsed,DEFINITION):
		return false
	for prefix: String in ['stone','fog']:
		var reference_path: String = str(DEFINITION.references[prefix+'Definition'])
		var reference_hash: String = str(DEFINITION.references[prefix+'DefinitionSha256'])
		var reference_bytes: PackedByteArray = _read(reference_path,1,16384)
		if reference_bytes.is_empty() or _sha(reference_bytes)!=reference_hash:
			return false
	definition_ready = true
	for kind: String in ['stone','fog']:
		var definition: Dictionary = DEFINITION[kind]
		var texture: Texture2D = _texture(definition)
		if texture!=null:
			textures[kind] = texture
	return true

func _finite(point: Vector2) -> bool:
	return is_finite(point.x) and is_finite(point.y)

func _rect(rect: Rect2) -> bool:
	return _finite(rect.position) and _finite(rect.size) and _finite(rect.end) and rect.size.x>0.0 and rect.size.y>0.0

func _view(snapshot: Variant, scene: Variant, key: Variant, projection: Variant, viewport: Variant) -> bool:
	if not definition_ready or not _base(snapshot,scene,key) or not projection is RefCounted or not is_instance_valid(projection) or projection.get_script()!=ProjectionScript or not viewport is Rect2:
		return false
	var origin: Variant = projection.get('origin')
	var center: Variant = projection.get('screen_center')
	var size: Variant = projection.get('viewport')
	var initialized: Variant = projection.get('initialized')
	if not origin is Vector3 or not center is Vector2 or not size is Vector2 or typeof(initialized)!=TYPE_BOOL or not initialized or not P.number(projection.get('ppm'),0,256) or float(projection.get('ppm'))<=0.0:
		return false
	return is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and _finite(center) and _finite(size) and _rect(viewport) and size==viewport.size

func _canvas(canvas: Variant) -> bool:
	return canvas is CanvasItem and is_instance_valid(canvas) and canvas.is_inside_tree() and not canvas.is_queued_for_deletion()

func _bounds(points: PackedVector2Array) -> Rect2:
	var low: Vector2 = Vector2(INF,INF)
	var high: Vector2 = Vector2(-INF,-INF)
	for point: Vector2 in points:
		low = low.min(point)
		high = high.max(point)
	return Rect2(low,high-low)

func _project(projection: Variant, world: Array[Vector3]) -> PackedVector2Array:
	var points: PackedVector2Array = PackedVector2Array()
	for vertex: Vector3 in world:
		var point: Vector2 = projection.project(vertex)
		if not _finite(point):
			return PackedVector2Array()
		points.append(point)
	return points

func _color_valid(color: Color) -> bool:
	return is_finite(color.r) and is_finite(color.g) and is_finite(color.b) and is_finite(color.a) and color.r>=0 and color.r<=1 and color.g>=0 and color.g<=1 and color.b>=0 and color.b<=1 and color.a>=0 and color.a<=1

func draw_floor(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, projection: Variant, viewport: Rect2) -> bool:
	if not _canvas(canvas) or not _view(snapshot,scene,key,projection,viewport):
		return false
	var texture_value: Variant = textures.get('stone')
	if not texture_value is Texture2D or not is_instance_valid(texture_value):
		return false
	var plan: Array[Dictionary] = []
	var tint: Color = Color(0.46,0.55,0.60,1)
	if not _color_valid(tint):
		return false
	for row: int in range(3):
		for column: int in range(12):
			var x0: float = 0.5+2.0*column
			var z0: float = 0.5+6.0*row
			var x1: float = minf(x0+2.0,23.5)
			var z1: float = minf(z0+6.0,15.5)
			var world: Array[Vector3] = [Vector3(x0,0,z0),Vector3(x1,0,z0),Vector3(x1,0,z1),Vector3(x0,0,z1)]
			var points: PackedVector2Array = _project(projection,world)
			var uvs: PackedVector2Array = PackedVector2Array()
			for vertex: Vector3 in world:
				var u: float = (vertex.x-x0)/2.0
				var v: float = (vertex.z-z0)/6.0
				var pixel: Vector2 = Vector2(820,240)+u*Vector2(212,102)+v*Vector2(-640,285)
				var uv: Vector2 = pixel/Vector2(1228,1254)
				if not _finite(uv) or uv.x<0 or uv.y<0 or uv.x>1 or uv.y>1:
					return false
				uvs.append(uv)
			if points.size()!=4 or not _rect(_bounds(points)):
				return false
			plan.append({'points':points,'uvs':uvs,'bounds':_bounds(points)})
	if plan.size()!=36:
		return false
	var drawn: bool = false
	for command: Dictionary in plan:
		var bounds: Rect2 = command.bounds
		if bounds.intersects(viewport):
			canvas.draw_polygon(command.points,PackedColorArray([tint,tint,tint,tint]),command.uvs,texture_value)
			drawn = true
	return drawn

func draw_wall(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, projection: Variant, wall: Variant, viewport: Rect2) -> bool:
	if not _canvas(canvas) or not _view(snapshot,scene,key,projection,viewport) or not wall is Dictionary:
		return false
	var id_value: Variant = wall.get('id')
	if typeof(id_value)!=TYPE_STRING or not WALLS.has(id_value):
		return false
	var member: bool = false
	var rows: Array = scene.collision
	for row_value: Variant in rows:
		if not row_value is Dictionary:
			return false
		if row_value.get('id')==id_value:
			member = row_value==wall
	if not member:
		return false
	var definition: Array = WALLS[id_value]
	var a: Vector2 = Vector2(definition[1][0],definition[1][1])
	var b: Vector2 = Vector2(definition[2][0],definition[2][1])
	var outward: Vector2 = Vector2(definition[3][0],definition[3][1])*0.05
	var height: float = float(definition[4])
	var contact_world: Array[Vector3] = [Vector3(a.x,0,a.y),Vector3(b.x,0,b.y),Vector3(b.x+outward.x,0,b.y+outward.y),Vector3(a.x+outward.x,0,a.y+outward.y)]
	var face_world: Array[Vector3] = [Vector3(a.x,0,a.y),Vector3(b.x,0,b.y),Vector3(b.x,height,b.y),Vector3(a.x,height,a.y)]
	var cap_world: Array[Vector3] = []
	var polygon: Array = definition[0]
	for pair: Array in polygon:
		cap_world.append(Vector3(pair[0],height,pair[1]))
	var colors: Array[Color] = [Color(0.015,0.025,0.03,0.12),Color('#21333b'),Color('#39505a')]
	var plan: Array[PackedVector2Array] = []
	for world: Array[Vector3] in [contact_world,face_world,cap_world]:
		var points: PackedVector2Array = _project(projection,world)
		if points.size()!=4 or not _rect(_bounds(points)):
			return false
		plan.append(points)
	var outer_for_wall: Array[PackedVector2Array] = []
	if _outer_context_matches(canvas,snapshot,scene,key,projection,viewport):
		var outer_plan: Dictionary = _outer_plan(projection,viewport)
		var staged: Variant = outer_plan.get('faces')
		if staged is Array and _outer_context_matches(canvas,snapshot,scene,key,projection,viewport):
			for face_value: Variant in staged:
				if not face_value is Dictionary:
					outer_for_wall.clear()
					break
				if face_value.get('id')!=id_value:
					continue
				var points_value: Variant = face_value.get('points')
				if not points_value is PackedVector2Array:
					outer_for_wall.clear()
					break
				var extra_points: PackedVector2Array = points_value
				outer_for_wall.append(extra_points)
			plan.append_array(outer_for_wall)
	var combined: PackedVector2Array = PackedVector2Array()
	for points: PackedVector2Array in plan:
		combined.append_array(points)
	if not _bounds(combined).intersects(viewport):
		return false
	for color: Color in colors:
		if not _color_valid(color):
			return false
	for index: int in range(3):
		if index==2:
			for points: PackedVector2Array in outer_for_wall:
				canvas.draw_colored_polygon(points,colors[1])
		canvas.draw_colored_polygon(plan[index],colors[index])
	return true

func _load_outer_scene_once() -> void:
	if _outer_scene_attempted:
		return
	_outer_scene_attempted = true
	var bytes: PackedByteArray = _read('res://data/scenes/mh_signal_yard.json',OUTER_SCENE_BYTES,OUTER_SCENE_BYTES)
	if bytes.size()!=OUTER_SCENE_BYTES or _sha(bytes)!=OUTER_SCENE_SHA256:
		return
	var text: String = bytes.get_string_from_utf8()
	if text.to_utf8_buffer()!=bytes:
		return
	var decoded: Variant = JSON.parse_string(text)
	if decoded is Dictionary:
		_outer_scene = decoded

func _outer_context_matches(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, projection_value: Variant, viewport_value: Variant) -> bool:
	if not _canvas(canvas_value) or not canvas_value.is_visible_in_tree() or not viewport_value is Rect2 or not _view(snapshot_value,scene_value,key_value,projection_value,viewport_value):
		return false
	var viewport: Rect2 = viewport_value
	if viewport.position!=Vector2.ZERO or canvas_value.get_viewport_rect()!=viewport:
		return false
	var current_snapshot: Variant = canvas_value.get('snapshot')
	var current_scene: Variant = canvas_value.get('scene')
	var current_key: Variant = canvas_value.get('current_key')
	var current_hash: Variant = canvas_value.get('scene_hash')
	var current_camera: Variant = canvas_value.get('camera')
	var current_viewport: Variant = canvas_value.get('viewport_rect')
	if not current_snapshot is Dictionary or not current_scene is Dictionary or not current_viewport is Rect2 or not _equal(current_snapshot,snapshot_value) or not _equal(current_scene,scene_value) or current_key!=key_value or current_hash!=OUTER_SCENE_SHA256 or current_camera!=projection_value or current_viewport!=viewport:
		return false
	_load_outer_scene_once()
	return not _outer_scene.is_empty() and _equal(scene_value,_outer_scene)

func _outer_quad_valid(points: PackedVector2Array) -> bool:
	if points.size()!=4:
		return false
	for point: Vector2 in points:
		if not _finite(point):
			return false
	if not _rect(_bounds(points)):
		return false
	var area2: float = 0.0
	for index: int in range(1,3):
		area2 += (points[index]-points[0]).cross(points[index+1]-points[0])
	if not is_finite(area2) or area2==0.0:
		return false
	for index: int in range(4):
		var edge: Vector2 = points[(index+1)%4]-points[index]
		var next_edge: Vector2 = points[(index+2)%4]-points[(index+1)%4]
		var turn: float = edge.cross(next_edge)
		if edge==Vector2.ZERO or not _finite(edge) or not is_finite(turn) or turn==0.0 or (turn>0.0)!=(area2>0.0):
			return false
	return true

func _outer_visibility(points: PackedVector2Array, viewport: Rect2) -> int:
	var corners: PackedVector2Array = PackedVector2Array([viewport.position,Vector2(viewport.end.x,viewport.position.y),viewport.end,Vector2(viewport.position.x,viewport.end.y)])
	var axes: Array[Vector2] = [Vector2.RIGHT,Vector2.DOWN]
	for index: int in range(4):
		var edge: Vector2 = points[(index+1)%4]-points[index]
		axes.append(Vector2(-edge.y,edge.x))
	var separated: bool = false
	for axis: Vector2 in axes:
		if not _finite(axis) or axis==Vector2.ZERO:
			return -1
		var p_min: float = INF
		var p_max: float = -INF
		var v_min: float = INF
		var v_max: float = -INF
		for point: Vector2 in points:
			var projection: float = (point-points[0]).dot(axis)
			if not is_finite(projection):
				return -1
			p_min = minf(p_min,projection)
			p_max = maxf(p_max,projection)
		for point: Vector2 in corners:
			var projection: float = (point-points[0]).dot(axis)
			if not is_finite(projection):
				return -1
			v_min = minf(v_min,projection)
			v_max = maxf(v_max,projection)
		if p_min>=p_max or v_min>=v_max:
			return -1
		if p_max<=v_min or v_max<=p_min:
			separated = true
	return 0 if separated else 1

func _outer_plan(projection: Variant, viewport: Rect2) -> Dictionary:
	var candidates: Array[Dictionary] = []
	for face: Dictionary in OUTER_FACES:
		var wall: Array = WALLS[face.id]
		var low: float = float(WALLS[face.low_from][4]) if face.low_from!='' else 0.0
		var high: float = float(wall[4])
		if low<0.0 or high<=low:
			return {}
		var a: Vector2 = Vector2(face.a[0],face.a[1])
		var b: Vector2 = Vector2(face.b[0],face.b[1])
		var world: Array[Vector3] = [Vector3(a.x,low,a.y),Vector3(b.x,low,b.y),Vector3(b.x,high,b.y),Vector3(a.x,high,a.y)]
		var points: PackedVector2Array = _project(projection,world)
		if not _outer_quad_valid(points):
			return {}
		candidates.append({'id':face.id,'points':points})
	var faces: Array[Dictionary] = []
	for candidate: Dictionary in candidates:
		var points: PackedVector2Array = candidate.points
		var visibility: int = _outer_visibility(points,viewport)
		if visibility<0:
			return {}
		if visibility==1:
			faces.append(candidate)
	return {'faces':faces}

func bank_rect(projection: Variant) -> Rect2:
	var width: float = 10.0*float(projection.get('ppm'))
	var center: Vector2 = projection.project(Vector3(8,0,3))+Vector2(0,-float(projection.get('ppm')))
	var size: Vector2 = Vector2(width,width*498.0/1201.0)
	return Rect2(center-size*0.5,size)

func draw_back_bank(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, projection: Variant, viewport: Rect2) -> bool:
	if not _canvas(canvas) or not _view(snapshot,scene,key,projection,viewport):
		return false
	var texture_value: Variant = textures.get('fog')
	if not texture_value is Texture2D or not is_instance_valid(texture_value):
		return false
	var rect: Rect2 = bank_rect(projection)
	var modulate: Color = Color(0.78,0.86,0.90,0.18)
	if not _rect(rect) or not rect.intersects(viewport) or not _color_valid(modulate):
		return false
	canvas.draw_texture_rect_region(texture_value,rect,Rect2(27,370,1201,498),modulate,false,true)
	return true

func _active_hazard(snapshot: Dictionary) -> bool:
	var hazards_value: Variant = snapshot.get('hazards')
	if not hazards_value is Array or hazards_value.size()!=1:
		return false
	var hazard_value: Variant = hazards_value[0]
	if not hazard_value is Dictionary:
		return false
	var hazard: Dictionary = hazard_value
	return hazard.get('entityId')=='mh_signal_interference_region' and hazard.get('kind')=='signal_interference_zone' and typeof(hazard.get('active'))==TYPE_BOOL and hazard.active and P.transform(hazard.get('transform')) and (not hazard.has('polygonM') or (P.polygon(hazard.get('polygonM')) and _equal(hazard.get('polygonM'),DEFINITION.air.polygon)))

# Phase is presentation-only. Missing/mismatched gates keep exact original static geometry.
func _air_phase(snapshot: Dictionary, gate: Dictionary) -> float:
	if typeof(gate.get('presentation_allowed'))!=TYPE_BOOL or not gate.presentation_allowed or typeof(gate.get('visible'))!=TYPE_BOOL or not gate.visible or typeof(gate.get('reduced_motion'))!=TYPE_BOOL or gate.reduced_motion:
		return -1.0
	if gate.get('worldId')!=snapshot.worldId or gate.get('sceneId')!=snapshot.sceneId or not P.counter(gate.get('worldEpoch'),1) or gate.worldEpoch!=snapshot.worldEpoch or gate.get('player_id')!=snapshot.player.entityId:
		return -1.0
	if not P.counter(gate.get('serverTick')) or gate.serverTick!=snapshot.serverTick or not P.counter(gate.get('authorityRevision')) or gate.authorityRevision!=snapshot.authorityRevision:
		return -1.0
	# P.snapshot/_view already validate the authoritative safe-integer tick.
	# Integer modulo precedes conversion, avoiding unbounded float phase precision.
	return TAU*float(int(snapshot.serverTick)%int(DEFINITION.air.motion.periodTicks))/float(DEFINITION.air.motion.periodTicks)

func draw_air(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, projection: Variant, viewport: Rect2, gate: Dictionary = {}) -> int:
	if not _canvas(canvas) or not _view(snapshot,scene,key,projection,viewport) or not _active_hazard(snapshot):
		return 0
	var phase: float = _air_phase(snapshot,gate)
	var xs: Array[float] = [13.35,14.6,15.8,17.1,18.65]
	var tops: Array[float] = [0.20,0.30,0.25,0.40,0.15]
	var plan: Array[Dictionary] = []
	for ribbon_index: int in range(3):
		var z: float = DEFINITION.air.ribbonZ[ribbon_index]
		for layer: int in range(2):
			var world: Array[Vector3] = []
			var dz: float = 0.08*layer
			var offsets: Array[float] = []
			for index: int in range(5):
				var wave: float = 0.0
				var dx: float = 0.0
				var inset: float = 0.0
				if phase>=0.0:
					wave = sin(phase+0.7*index+0.8*ribbon_index+0.3*layer)
					dx = 0.06*wave
					inset = 0.02*(0.5+0.5*wave)
				offsets.append(dx)
				world.append(Vector3(xs[index]+dx,tops[index]-0.02*layer-inset,z+dz))
			for index: int in range(4,-1,-1):
				world.append(Vector3(xs[index]+offsets[index],0.04+0.02*layer,z+dz))
			for vertex: Vector3 in world:
				if vertex.x<13 or vertex.x>19 or vertex.z<5 or vertex.z>10 or vertex.y<0 or vertex.y>0.400001:
					return 0
			var points: PackedVector2Array = _project(projection,world)
			var color: Color = Color(0.52,0.64,0.69,0.04 if layer==0 else 0.055)
			if points.size()!=10 or not _rect(_bounds(points)) or not _color_valid(color):
				return 0
			plan.append({'points':points,'color':color,'bounds':_bounds(points)})
	if plan.size()!=6:
		return 0
	var count: int = 0
	for command: Dictionary in plan:
		var bounds: Rect2 = command.bounds
		if bounds.intersects(viewport):
			canvas.draw_colored_polygon(command.points,command.color)
			count += 1
	return count

func should_suppress_bank(prop: Variant, snapshot: Variant, scene: Variant, key: Variant, projection: Variant, viewport: Rect2, drawn_ids: Array) -> bool:
	if not _view(snapshot,scene,key,projection,viewport) or not prop is Dictionary or not 'yard_north_fog_bank' in drawn_ids:
		return false
	return prop.get('id')=='yard_north_fog_bank' and prop.get('assetId')==DEFINITION.fog.assetId and prop.get('layer')=='visual.props_back' and _equal(prop.get('position'),[8,0,3]) and textures.get('fog') is Texture2D and _rect(bank_rect(projection)) and bank_rect(projection).intersects(viewport)


# Covered, flush cable-trough display; proposed calibration, not native geometry.
# Independent of image preparation and internal-use resource admission.
func _trough_view(snapshot: Variant, scene: Variant, key: Variant, projection: Variant, viewport: Rect2) -> bool:
	if not _base(snapshot,scene,key) or not projection is RefCounted or not is_instance_valid(projection) or projection.get_script()!=ProjectionScript:
		return false
	if not _equal(scene.get('occluders'),[]) or not _equal(scene.get('doors'),[]):
		return false
	var origin: Variant = projection.get('origin')
	var center: Variant = projection.get('screen_center')
	var size: Variant = projection.get('viewport')
	var initialized: Variant = projection.get('initialized')
	if not origin is Vector3 or not center is Vector2 or not size is Vector2 or typeof(initialized)!=TYPE_BOOL or not initialized or not P.number(projection.get('ppm'),0,256) or float(projection.get('ppm'))<=0.0:
		return false
	return is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and _finite(center) and _finite(size) and _rect(viewport) and size==viewport.size

func _trough_quad_valid(points: PackedVector2Array) -> bool:
	if points.size()!=4:
		return false
	var sign_value: float = 0.0
	for index: int in range(4):
		if not _finite(points[index]):
			return false
		var a: Vector2 = points[(index+1)%4]-points[index]
		var b: Vector2 = points[(index+2)%4]-points[(index+1)%4]
		var cross_value: float = a.cross(b)
		if not _finite(a) or not _finite(b) or not is_finite(cross_value) or absf(cross_value)<=0.000001:
			return false
		if index==0:
			sign_value = cross_value
		elif (cross_value>0.0)!=(sign_value>0.0):
			return false
	return true

# -1 invalid arithmetic, 0 disjoint/tangent, 1 positive-area intersection.
func _trough_intersection(points: PackedVector2Array, viewport: Rect2) -> int:
	var corners: PackedVector2Array = PackedVector2Array([viewport.position,Vector2(viewport.end.x,viewport.position.y),viewport.end,Vector2(viewport.position.x,viewport.end.y)])
	var axes: Array[Vector2] = [Vector2.RIGHT,Vector2.DOWN]
	for index: int in range(4):
		var edge: Vector2 = points[(index+1)%4]-points[index]
		axes.append(Vector2(-edge.y,edge.x))
	var separated: bool = false
	for axis: Vector2 in axes:
		if not _finite(axis) or axis==Vector2.ZERO:
			return -1
		var low: float = INF
		var high: float = -INF
		var rect_low: float = INF
		var rect_high: float = -INF
		for point: Vector2 in points:
			var value: float = (point-points[0]).dot(axis)
			if not is_finite(value):
				return -1
			low = minf(low,value)
			high = maxf(high,value)
		for point: Vector2 in corners:
			var value: float = (point-points[0]).dot(axis)
			if not is_finite(value):
				return -1
			rect_low = minf(rect_low,value)
			rect_high = maxf(rect_high,value)
		if high<=rect_low or rect_high<=low:
			separated = true
	return 0 if separated else 1

func draw_cable_trough(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, projection: Variant, viewport: Rect2) -> bool:
	if not _canvas(canvas) or not _trough_view(snapshot,scene,key,projection,viewport):
		return false
	var colors: Array[Color] = [Color('#30373b'),Color('#576267')]
	for color: Color in colors:
		if not _color_valid(color):
			return false
	var plan: Array[PackedVector2Array] = []
	var visible: bool = false
	for index: int in range(9):
		var x0: float = 4.0 if index==0 else 4.0+2.0*(index-1)+0.02
		var x1: float = 20.0 if index==0 else 4.0+2.0*(index-1)+1.98
		var z0: float = 1.0 if index==0 else 1.02
		var z1: float = 1.3 if index==0 else 1.28
		var world: Array[Vector3] = [Vector3(x0,0,z0),Vector3(x1,0,z0),Vector3(x1,0,z1),Vector3(x0,0,z1)]
		var points: PackedVector2Array = _project(projection,world)
		if not _trough_quad_valid(points):
			return false
		var intersection: int = _trough_intersection(points,viewport)
		if intersection<0:
			return false
		visible = visible or intersection==1
		plan.append(points)
	if plan.size()!=9 or not visible:
		return false
	for index: int in range(9):
		canvas.draw_colored_polygon(plan[index],colors[0 if index==0 else 1])
	return true
