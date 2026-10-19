extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const DEFINITION: Dictionary = {
  "schema": "wuxian.gh-gate-b-west-cable.internal-use/v1",
  "approval": "SOURCE_ONLY_NEW_USE_APPROVED",
  "GodotDisplay": "NOT_RUN",
  "publicRelease": "NOT_ESTABLISHED",
  "runtimeValidation": "NOT_RUN",
  "use": "static wall-mounted curved cable only; no pipe function or collision",
  "worldId": "grey_hive",
  "sceneId": "gh_gate_b",
  "source": {
    "path": "res://assets/props/return_station/service_pipes.png",
    "bytes": 1049893,
    "sha256": "7f3c67879398e89adc9026fc2aca5fbe7792b1dc360b73c737c1b2299eac96da",
    "gitBlob": "e760555aae85e9299b3bce92422022362277688e",
    "canvas": [
      1414,
      1047
    ],
    "format": "RGBA8"
  },
  "selection": {
    "seed": [
      1190,
      820
    ],
    "connectivity": 4,
    "alpha": "greater_than_zero",
    "count": 46260,
    "boundsInclusive": [
      1011,
      647,
      1401,
      954
    ],
    "frame": [
      1011,
      647,
      391,
      308
    ],
    "frameNonzeroCount": 47323,
    "channels": "preserve selected original RGBA; all nonselected output RGBA zero"
  },
  "placeholder": {
    "id": "gate_b_compression_pipes",
    "assetId": "runtime2d.grey_hive.pipes_cables.v1",
    "layer": "visual.props_back",
    "position": [
      15,
      0,
      3
    ]
  },
  "wall": {
    "id": "gh_gate_b_wall_west_north",
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
        6.5
      ],
      [
        0,
        6.5
      ]
    ]
  },
  "display": {
    "worldReference": [
      0.5,
      0.35515625,
      3
    ],
    "frameAnchor": [
      195.5,
      154
    ],
    "screenScaleAt48Ppm": 0.15,
    "rotation": 0,
    "mirror": false,
    "modulate": [
      1,
      1,
      1,
      1
    ],
    "offscreen": "submit valid command and allow viewport clipping"
  },
  "excluded": [
    "straight flange pipe",
    "clutter",
    "new PNG",
    "native position change",
    "floor shadow"
  ]
}

var attempted: bool = false
var texture: Texture2D = null

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
	if hash.start(HashingContext.HASH_SHA256)!=OK or hash.update(bytes)!=OK:
		return ''
	return hash.finish().hex_encode()

func _selected_pixels(raw: PackedByteArray) -> PackedByteArray:
	if raw.size()!=1414*1047*4:
		return PackedByteArray()
	var seed: int = 820*1414+1190
	if raw[seed*4+3]==0:
		return PackedByteArray()
	var visited: PackedByteArray = PackedByteArray()
	visited.resize(1414*1047)
	visited.fill(0)
	var queue: PackedInt32Array = PackedInt32Array([seed])
	visited[seed] = 1
	var cursor: int = 0
	var low: Vector2i = Vector2i(1414,1047)
	var high: Vector2i = Vector2i(-1,-1)
	while cursor<queue.size():
		var pixel: int = queue[cursor]
		cursor += 1
		var x: int = pixel%1414
		var y: int = int(pixel/1414)
		# Traversal is in full source coordinates; never crop a connected overflow.
		if x<1011 or x>1401 or y<647 or y>954:
			return PackedByteArray()
		low = Vector2i(mini(low.x,x),mini(low.y,y))
		high = Vector2i(maxi(high.x,x),maxi(high.y,y))
		for delta: Vector2i in [Vector2i(-1,0),Vector2i(1,0),Vector2i(0,-1),Vector2i(0,1)]:
			var nx: int = x+delta.x
			var ny: int = y+delta.y
			if nx<0 or nx>=1414 or ny<0 or ny>=1047:
				continue
			var neighbor: int = ny*1414+nx
			if visited[neighbor]==0 and raw[neighbor*4+3]>0:
				visited[neighbor] = 1
				queue.append(neighbor)
				if queue.size()>46260:
					return PackedByteArray()
	if queue.size()!=46260 or low!=Vector2i(1011,647) or high!=Vector2i(1401,954):
		return PackedByteArray()
	var output: PackedByteArray = PackedByteArray()
	output.resize(391*308*4)
	output.fill(0)
	var frame_nonzero: int = 0
	var copied: int = 0
	for y: int in range(647,955):
		for x: int in range(1011,1402):
			var source_pixel: int = y*1414+x
			if raw[source_pixel*4+3]>0:
				frame_nonzero += 1
			if visited[source_pixel]==1:
				var destination: int = ((y-647)*391+x-1011)*4
				for channel: int in range(4):
					output[destination+channel] = raw[source_pixel*4+channel]
				copied += 1
	return output if copied==46260 and frame_nonzero==47323 else PackedByteArray()

func prepare_resources() -> void:
	if attempted:
		return
	attempted = true
	var map_bytes: PackedByteArray = _read('res://data/gh_gate_b_west_cable.internal-use.json',1,16384)
	if map_bytes.is_empty() or map_bytes.get_string_from_utf8().to_utf8_buffer()!=map_bytes:
		return
	var definition: Variant = JSON.parse_string(map_bytes.get_string_from_utf8())
	if not _equal(definition,DEFINITION):
		return
	var bytes: PackedByteArray = _read(DEFINITION.source.path,1049893,1049893)
	if bytes.is_empty() or _sha(bytes)!=DEFINITION.source.sha256:
		return
	var original: Image = Image.new()
	if original.load_png_from_buffer(bytes)!=OK or original.get_width()!=1414 or original.get_height()!=1047 or original.get_format()!=Image.FORMAT_RGBA8 or original.has_mipmaps():
		return
	var selected: PackedByteArray = _selected_pixels(original.get_data())
	if selected.size()!=391*308*4:
		return
	var image: Image = Image.create_from_data(391,308,false,Image.FORMAT_RGBA8,selected)
	if image==null or image.is_empty() or image.get_data()!=selected:
		return
	var candidate: ImageTexture = ImageTexture.create_from_image(image)
	if candidate==null or not is_instance_valid(candidate) or candidate.get_width()!=391 or candidate.get_height()!=308:
		return
	texture = candidate

func matches_authored_row(prop: Variant) -> bool:
	return _equal(prop,DEFINITION.placeholder)

func _finite(point: Vector2) -> bool:
	return is_finite(point.x) and is_finite(point.y) and absf(point.x)<=10000000.0 and absf(point.y)<=10000000.0

func _context(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, projection: Variant, wall: Variant, viewport: Variant) -> bool:
	if not canvas is CanvasItem or not is_instance_valid(canvas) or not canvas.is_inside_tree() or canvas.is_queued_for_deletion():
		return false
	if not snapshot is Dictionary or not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or not scene is Dictionary:
		return false
	if snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_gate_b' or not P.counter(snapshot.get('worldEpoch'),1) or typeof(key)!=TYPE_STRING or key!='grey_hive:gh_gate_b:'+str(snapshot.worldEpoch):
		return false
	if scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_gate_b' or not P.counter(scene.get('schemaVersion'),1,1) or not _equal(scene.get('boundsM'),{'x':0,'z':0,'width':24,'depth':16}) or not _equal(scene.get('occluders'),[]):
		return false
	if not _equal(wall,DEFINITION.wall):
		return false
	var collisions: Variant = scene.get('collision')
	if not collisions is Array or collisions.size()!=7:
		return false
	var walls_found: int = 0
	for item: Variant in collisions:
		if not item is Dictionary:
			return false
		if item.get('id')==DEFINITION.wall.id:
			if not _equal(item,wall):
				return false
			walls_found += 1
	if walls_found!=1:
		return false
	var presentation: Variant = scene.get('presentation')
	if not presentation is Dictionary:
		return false
	var sprites: Variant = presentation.get('sprites')
	if not sprites is Array or sprites.size()!=8:
		return false
	var props_found: int = 0
	for item: Variant in sprites:
		if not item is Dictionary:
			return false
		if item.get('id')==DEFINITION.placeholder.id:
			if not matches_authored_row(item):
				return false
			props_found += 1
	if props_found!=1 or not projection is RefCounted or not is_instance_valid(projection) or projection.get_script()!=ProjectionScript or not viewport is Rect2:
		return false
	var initialized: Variant = projection.get('initialized')
	var origin: Variant = projection.get('origin')
	var center: Variant = projection.get('screen_center')
	var size: Variant = projection.get('viewport')
	if typeof(initialized)!=TYPE_BOOL or not initialized or not origin is Vector3 or not center is Vector2 or not size is Vector2 or not P.number(projection.get('ppm'),0.000001,256.0):
		return false
	return is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and _finite(center) and _finite(size) and _finite(viewport.position) and _finite(viewport.size) and _finite(viewport.end) and viewport.size.x>0 and viewport.size.y>0 and size==viewport.size

func draw_wall_cable(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, projection: Variant, wall: Variant, viewport: Variant) -> bool:
	if texture==null or not is_instance_valid(texture) or texture.get_width()!=391 or texture.get_height()!=308 or not _context(canvas,snapshot,scene,key,projection,wall,viewport):
		return false
	var scale_value: float = 0.15*float(projection.get('ppm'))/48.0
	var reference: Vector2 = projection.project(Vector3(0.5,0.35515625,3.0))
	var size: Vector2 = Vector2(391,308)*scale_value
	var rect: Rect2 = Rect2(reference-Vector2(195.5,154)*scale_value,size)
	if not is_finite(scale_value) or scale_value<=0 or not _finite(reference) or not _finite(rect.position) or not _finite(rect.size) or not _finite(rect.end) or rect.size.x<=0 or rect.size.y<=0 or rect.end.x<=rect.position.x or rect.end.y<=rect.position.y:
		return false
	# Deliberately submit valid offscreen commands: the old prop anchor is elsewhere.
	canvas.draw_texture_rect(texture,rect,false,Color.WHITE)
	return true
