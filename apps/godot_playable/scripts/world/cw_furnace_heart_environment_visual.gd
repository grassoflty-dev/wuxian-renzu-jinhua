extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const SCENE_PATH = 'res://data/scenes/cw_furnace_heart.json'
const SCENE_BYTES = 8424
const SCENE_SHA256 = 'f16f3941767aed71023cd8b9ce75642345a103eabf5c77682e358c96c2e219d5'
# Proposed Heart cutaway appearance, following the existing CW steel palette.
# These are display heights only, never collision or traversable support heights.
const HIGH_M = 1.0
const LOW_M = 0.18
const SIDE_COLOR = Color(0.18,0.22,0.24,1.0)
const CAP_COLOR = Color(0.25,0.29,0.31,1.0)
const WALLS = [
	{'id':'cw_furnace_heart_wall_north','polygon':[[0,0],[24,0],[24,0.5],[0,0.5]]},
	{'id':'cw_furnace_heart_wall_south','polygon':[[0,15.5],[24,15.5],[24,16],[0,16]]},
	{'id':'cw_furnace_heart_wall_west_north','polygon':[[0,0.5],[0.5,0.5],[0.5,6],[0,6]]},
	{'id':'cw_furnace_heart_wall_west_south','polygon':[[0,9.5],[0.5,9.5],[0.5,15.5],[0,15.5]]},
	{'id':'cw_furnace_heart_wall_east_north','polygon':[[23.5,0.5],[24,0.5],[24,6],[23.5,6]]},
	{'id':'cw_furnace_heart_wall_east_south','polygon':[[23.5,9.5],[24,9.5],[24,15.5],[23.5,15.5]]}
]
const CAP_HEIGHTS = [HIGH_M,LOW_M,HIGH_M,HIGH_M,LOW_M,LOW_M]
# Complete +x/+z exterior union, including both 0.82m high step faces.
# a/b are XZ endpoints; clip_low_end applies only to the two low-cap corners.
const SIDES = [
	{'a':[0.5,0.5],'b':[23.5,0.5],'low':0.0,'high':HIGH_M,'clip_low_end':true},
	{'a':[23.5,0.5],'b':[24,0.5],'low':LOW_M,'high':HIGH_M,'clip_low_end':false},
	{'a':[24,0],'b':[24,0.5],'low':0.0,'high':HIGH_M,'clip_low_end':false},
	{'a':[0,16],'b':[24,16],'low':0.0,'high':LOW_M,'clip_low_end':false},
	{'a':[24,15.5],'b':[24,16],'low':0.0,'high':LOW_M,'clip_low_end':false},
	{'a':[0.5,0.5],'b':[0.5,6],'low':0.0,'high':HIGH_M,'clip_low_end':false},
	{'a':[0,6],'b':[0.5,6],'low':0.0,'high':HIGH_M,'clip_low_end':false},
	{'a':[0.5,9.5],'b':[0.5,15.5],'low':0.0,'high':HIGH_M,'clip_low_end':true},
	{'a':[0,15.5],'b':[0.5,15.5],'low':LOW_M,'high':HIGH_M,'clip_low_end':false},
	{'a':[24,0.5],'b':[24,6],'low':0.0,'high':LOW_M,'clip_low_end':false},
	{'a':[23.5,6],'b':[24,6],'low':0.0,'high':LOW_M,'clip_low_end':false},
	{'a':[24,9.5],'b':[24,15.5],'low':0.0,'high':LOW_M,'clip_low_end':false}
]

const LOW_FACE_PATH = 'res://assets/props/return_station/bulkhead_tiles.png'
const LOW_FACE_SIZE = Vector2(1418,1028)
var _low_face_attempted: bool = false
var _low_face_texture: Texture2D = null

func _low_face_atlas() -> Texture2D:
	if _low_face_attempted:
		return _low_face_texture
	_low_face_attempted = true
	var texture: Texture2D = null
	if ResourceLoader.exists(LOW_FACE_PATH, 'Texture2D'):
		var loaded: Variant = ResourceLoader.load(LOW_FACE_PATH, 'Texture2D')
		if loaded is Texture2D:
			texture = loaded
	if not is_instance_valid(texture):
		var image: Image = Image.new()
		if image.load(LOW_FACE_PATH) == OK and not image.is_empty():
			texture = ImageTexture.create_from_image(image)
	if is_instance_valid(texture) and texture.get_width() == int(LOW_FACE_SIZE.x) and texture.get_height() == int(LOW_FACE_SIZE.y):
		_low_face_texture = texture
	return _low_face_texture

var _scene_attempted: bool = false
var _canonical_scene: Dictionary = {}

func _same_json(value: Variant, expected: Variant, depth: int = 0) -> bool:
	if depth > 64:
		return false
	if typeof(expected) == TYPE_INT or typeof(expected) == TYPE_FLOAT:
		return P.number(value) and float(value) == float(expected)
	if typeof(value) != typeof(expected):
		return false
	if expected is Dictionary:
		if value.size() != expected.size():
			return false
		for key: Variant in expected:
			if not value.has(key) or not _same_json(value[key],expected[key],depth+1):
				return false
		return true
	if expected is Array:
		if value.size() != expected.size():
			return false
		for index: int in range(expected.size()):
			if not _same_json(value[index],expected[index],depth+1):
				return false
		return true
	return value == expected

# Renderer must require draw_shell success in the same frame before using this.
func owns_collision(wall_value: Variant) -> bool:
	if not wall_value is Dictionary or wall_value.size() != 2:
		return false
	for expected: Dictionary in WALLS:
		if _same_json(wall_value,expected):
			return true
	return false

func _load_scene_once() -> void:
	if _scene_attempted:
		return
	_scene_attempted = true
	var file: FileAccess = FileAccess.open(SCENE_PATH,FileAccess.READ)
	if file == null:
		return
	if file.get_length() != SCENE_BYTES:
		file.close()
		return
	var bytes: PackedByteArray = file.get_buffer(SCENE_BYTES)
	file.close()
	if bytes.size() != SCENE_BYTES:
		return
	var hash: HashingContext = HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256) != OK or hash.update(bytes) != OK or hash.finish().hex_encode() != SCENE_SHA256:
		return
	var source_text: String = bytes.get_string_from_utf8()
	if source_text.to_utf8_buffer() != bytes:
		return
	var decoded: Variant = JSON.parse_string(source_text)
	if decoded is Dictionary:
		_canonical_scene = decoded

func _context(snapshot: Dictionary, scene: Dictionary, key_value: Variant, hash_value: Variant) -> bool:
	if snapshot.get('worldId') != 'clockworks' or snapshot.get('sceneId') != 'cw_furnace_heart' or snapshot.get('entryToken') != null:
		return false
	if not P.snapshot(snapshot) or hash_value != SCENE_SHA256:
		return false
	var epoch: Variant = snapshot.get('worldEpoch')
	if not P.counter(epoch,1) or typeof(key_value) != TYPE_STRING or key_value != 'clockworks:cw_furnace_heart:'+str(epoch):
		return false
	_load_scene_once()
	if _canonical_scene.is_empty() or not _same_json(scene,_canonical_scene):
		return false
	var rows: Variant = scene.get('collision')
	if not rows is Array or rows.size() != WALLS.size():
		return false
	var seen: Dictionary = {}
	for wall: Variant in rows:
		if not owns_collision(wall) or seen.has(wall.id):
			return false
		seen[wall.id] = true
	return true

func _finite_v2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _finite_v3(value: Vector3) -> bool:
	return is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _valid_rect(value: Rect2) -> bool:
	return _finite_v2(value.position) and _finite_v2(value.size) and _finite_v2(value.end) and value.size.x > 0.0 and value.size.y > 0.0

func _valid_camera(camera: Variant, viewport: Rect2) -> bool:
	if not camera is RefCounted or not is_instance_valid(camera) or camera.get_script() != ProjectionScript:
		return false
	if typeof(camera.initialized) != TYPE_BOOL or not camera.initialized or not camera.origin is Vector3 or not camera.screen_center is Vector2 or not camera.viewport is Vector2:
		return false
	return P.number(camera.ppm,0.0,256.0) and camera.ppm > 0.0 and _finite_v3(camera.origin) and _finite_v2(camera.screen_center) and _finite_v2(camera.viewport) and camera.viewport.x > 0.0 and camera.viewport.y > 0.0 and camera.viewport.is_equal_approx(viewport.size)

func _canvas_matches(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, scene_hash_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not canvas_value is CanvasItem or not is_instance_valid(canvas_value) or not canvas_value.is_inside_tree() or not canvas_value.is_visible_in_tree() or canvas_value.is_queued_for_deletion():
		return false
	if not snapshot_value is Dictionary or not scene_value is Dictionary or typeof(key_value) != TYPE_STRING or typeof(scene_hash_value) != TYPE_STRING or not viewport_value is Rect2:
		return false
	var viewport: Rect2 = viewport_value
	if viewport.position != Vector2.ZERO or not _valid_rect(viewport) or not _valid_camera(camera_value,viewport):
		return false
	if canvas_value.get('snapshot') != snapshot_value or canvas_value.get('scene') != scene_value or canvas_value.get('current_key') != key_value or canvas_value.get('scene_hash') != scene_hash_value or canvas_value.get('camera') != camera_value or canvas_value.get('viewport_rect') != viewport:
		return false
	if canvas_value.get_viewport_rect() != viewport or camera_value.viewport != viewport.size:
		return false
	return true

func _project_world(points: Array[Vector3], camera: RefCounted) -> PackedVector2Array:
	var result: PackedVector2Array = PackedVector2Array()
	for point: Vector3 in points:
		if not _finite_v3(point):
			return PackedVector2Array()
		var projected: Vector2 = camera.project(point)
		if not _finite_v2(projected):
			return PackedVector2Array()
		result.append(projected)
	return result

func _plan(camera: RefCounted) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for face: Dictionary in SIDES:
		var a: Vector3 = Vector3(face.a[0],face.low,face.a[1])
		var b: Vector3 = Vector3(face.b[0],face.low,face.b[1])
		var top_a: Vector3 = Vector3(a.x,face.high,a.z)
		var top_b: Vector3 = Vector3(b.x,face.high,b.z)
		var original_world: Array[Vector3] = [a,b,top_b,top_a]
		var final_world: Array[Vector3] = original_world.duplicate()
		if face.clip_low_end:
			# The .18m adjacent cap hides only the lower terminal triangle.
			# Keep the endpoint at cap height and the entire upper face.
			var axis: Vector3 = Vector3.RIGHT if b.x > a.x else Vector3(0,0,1)
			final_world = [a,b-axis*LOW_M,Vector3(b.x,LOW_M,b.z),top_b,top_a]
		result.append({'original':_project_world(original_world,camera),'points':_project_world(final_world,camera),'color':SIDE_COLOR})
	for index: int in range(WALLS.size()):
		var cap_world: Array[Vector3] = []
		for point: Array in WALLS[index].polygon:
			cap_world.append(Vector3(point[0],CAP_HEIGHTS[index],point[1]))
		var cap_points: PackedVector2Array = _project_world(cap_world,camera)
		result.append({'original':cap_points,'points':cap_points,'color':CAP_COLOR})
	return result

# All eighteen original and final convex faces must remain fully renderable.
func _valid_polygon(points: PackedVector2Array) -> bool:
	if points.size() < 4 or points.size() > 5:
		return false
	for point: Vector2 in points:
		if not _finite_v2(point):
			return false
	var area2: float = 0.0
	for index: int in range(1,points.size()-1):
		var triangle: float = (points[index]-points[0]).cross(points[index+1]-points[0])
		if not is_finite(triangle):
			return false
		area2 += triangle
	if not is_finite(area2) or area2 == 0.0:
		return false
	for index: int in range(points.size()):
		var edge: Vector2 = points[(index+1)%points.size()]-points[index]
		var next_edge: Vector2 = points[(index+2)%points.size()]-points[(index+1)%points.size()]
		var turn: float = edge.cross(next_edge)
		if not _finite_v2(edge) or edge == Vector2.ZERO or not is_finite(turn) or turn == 0.0 or (turn > 0.0) != (area2 > 0.0):
			return false
	return true

# Strict convex polygon/rectangle SAT: -1 invalid, 0 outside, 1 positive overlap.
# No clipping or generated fragments; at most seven axes per fixed face.
func _visible(points: PackedVector2Array, viewport: Rect2) -> int:
	var axes: Array[Vector2] = [Vector2.RIGHT,Vector2.DOWN]
	for index: int in range(points.size()):
		var edge: Vector2 = points[(index+1)%points.size()]-points[index]
		axes.append(Vector2(-edge.y,edge.x))
	var corners: PackedVector2Array = PackedVector2Array([viewport.position,Vector2(viewport.end.x,viewport.position.y),viewport.end,Vector2(viewport.position.x,viewport.end.y)])
	var origin: Vector2 = points[0]
	var separated: bool = false
	for axis: Vector2 in axes:
		if not _finite_v2(axis) or axis == Vector2.ZERO:
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
		if not is_finite(polygon_min) or not is_finite(polygon_max) or not is_finite(rect_min) or not is_finite(rect_max) or polygon_min >= polygon_max or rect_min >= rect_max:
			return -1
		if polygon_max <= rect_min or rect_max <= polygon_min:
			separated = true
	return 0 if separated else 1

func _steel_cap_pieces(camera: RefCounted, polygon: Array, height: float) -> Array:
	var x0=float(polygon[0][0]); var z0=float(polygon[0][1])
	var x1=float(polygon[2][0]); var z1=float(polygon[2][1])
	if not is_finite(x0) or not is_finite(z0) or not is_finite(x1) or not is_finite(z1) or x0>=x1 or z0>=z1:
		return []
	var pieces: Array = []
	var x=x0
	while x<x1:
		var cx=floorf(x/2.0)*2.0
		var xe=minf(x1,cx+2.0)
		if xe<=x: return []
		var z=z0
		while z<z1:
			var cz=floorf(z/2.0)*2.0
			var ze=minf(z1,cz+2.0)
			if ze<=z: return []
			var points=_project_world([Vector3(x,height,z),Vector3(xe,height,z),Vector3(xe,height,ze),Vector3(x,height,ze)],camera)
			if not _valid_polygon(points): return []
			var u0=(x-cx)/2.0; var u1=(xe-cx)/2.0; var v0=(z-cz)/2.0; var v1=(ze-cz)/2.0
			var uvs:=PackedVector2Array()
			for uv in [Vector2(u0,v0),Vector2(u1,v0),Vector2(u1,v1),Vector2(u0,v1)]:
				uvs.append((Vector2(510,200)+Vector2(460,260)*uv.x+Vector2(-370,280)*uv.y)/Vector2(1197,1132))
			pieces.append({'points':points,'uv':uvs})
			z=ze
		x=xe
	return pieces

# Integer world-X phase, including the half-cell at X 23.5.
func _low_face_pieces(camera: RefCounted, x0: float, x1: float, z: float) -> Array:
	var pieces: Array = []
	var x: float = x0
	while x < x1:
		var base: float = floorf(x)
		var end: float = minf(x1,base+1.0)
		if end <= x:
			return []
		var points: PackedVector2Array = _project_world([Vector3(x,0,z),Vector3(end,0,z),Vector3(end,LOW_M,z),Vector3(x,LOW_M,z)],camera)
		if not _valid_polygon(points):
			return []
		var uv: PackedVector2Array = PackedVector2Array()
		for pixel: Vector2 in [Vector2(50,322)+Vector2(250,138)*(x-base),Vector2(50,322)+Vector2(250,138)*(end-base),Vector2(51.872,276.064)+Vector2(250,138)*(end-base),Vector2(51.872,276.064)+Vector2(250,138)*(x-base)]:
			if not _finite_v2(pixel) or pixel.x < 0.0 or pixel.y < 0.0 or pixel.x > LOW_FACE_SIZE.x or pixel.y > LOW_FACE_SIZE.y:
				return []
			uv.append(pixel/LOW_FACE_SIZE)
		pieces.append({'points':points,'uv':uv})
		x = end
	return pieces

# Immutable world-space cells and their projected cache; original shell guards
# still run on every draw before this cache is consumed.
var _high_layout_attempted: bool = false
var _high_layout: Dictionary = {}
var _high_camera: Variant = null
var _high_camera_frame: Array = []
var _high_projected: Dictionary = {}

func _high_layout_once() -> Dictionary:
	if _high_layout_attempted:
		return _high_layout
	_high_layout_attempted = true
	var groups: Dictionary = {}
	for spec: Array in [[0,2,94,0.5,0.0,true],[1,94,96,0.5,LOW_M,false],[6,0,2,6.0,0.0,false],[8,0,2,15.5,LOW_M,false]]:
		var index: int = spec[0]
		var cells: Array = []
		for xi: int in range(spec[1],spec[2]):
			var x0: float = float(xi)*0.25
			var x1: float = x0+0.25
			for yi: int in range(4):
				var cell_y: float = float(yi)*0.25
				var y0: float = maxf(cell_y,float(spec[4]))
				var y1: float = cell_y+0.25
				if y0 >= y1:
					continue
				var xy: Array[Vector2] = [Vector2(x0,y0),Vector2(x1,y0),Vector2(x1,y1),Vector2(x0,y1)]
				if bool(spec[5]) and xi==93 and yi==0:
					# Exact original clipped boundary from (b.x-.18,0) to (b.x,.18).
					xy = [Vector2(x0,0),Vector2(23.5-LOW_M,0),Vector2(23.5,LOW_M),Vector2(x1,y1),Vector2(x0,y1)]
				var world: Array[Vector3] = []
				var uvs: PackedVector2Array = PackedVector2Array()
				for point: Vector2 in xy:
					var u: float = (point.x-x0)/0.25
					var v: float = (point.y-cell_y)/0.25
					if not _finite_v2(point) or u<0.0 or u>1.0 or v<0.0 or v>1.0:
						return {}
					var pixel: Vector2 = Vector2(150,270)+Vector2(62.5,34.5)*u+Vector2(2.6,-63.8)*v
					if not _finite_v2(pixel):
						return {}
					world.append(Vector3(point.x,point.y,float(spec[3])))
					uvs.append(pixel/LOW_FACE_SIZE)
				cells.append({'world':world,'uv':uvs})
		groups[index] = cells
	if groups.size()!=4 or groups[0].size()!=368 or groups[1].size()!=8 or groups[6].size()!=8 or groups[8].size()!=8:
		return {}
	_high_layout = groups
	return _high_layout

func _high_face_groups(camera: RefCounted) -> Dictionary:
	var frame: Array = [camera.origin,camera.screen_center,camera.viewport,camera.ppm,camera.initialized]
	if _high_camera == camera and _high_camera_frame == frame:
		return _high_projected
	_high_camera = camera
	_high_camera_frame = frame
	_high_projected = {}
	var layout: Dictionary = _high_layout_once()
	if layout.size()!=4:
		return {}
	var groups: Dictionary = {}
	for index: int in [0,1,6,8]:
		var projected_cells: Array = []
		for cell: Dictionary in layout[index]:
			var points: PackedVector2Array = _project_world(cell.world,camera)
			if not _valid_polygon(points) or points.size()!=cell.uv.size():
				return {}
			projected_cells.append({'points':points,'uv':cell.uv})
		groups[index] = projected_cells
	_high_projected = groups
	return _high_projected

func draw_shell(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, scene_hash_value: Variant, camera_value: Variant, viewport_value: Variant) -> bool:
	if not _canvas_matches(canvas_value,snapshot_value,scene_value,key_value,scene_hash_value,camera_value,viewport_value):
		return false
	if not _context(snapshot_value,scene_value,key_value,scene_hash_value):
		return false
	var viewport: Rect2 = viewport_value
	var plan: Array[Dictionary] = _plan(camera_value)
	if plan.size() != 18:
		return false
	var drawable: Array[Dictionary] = []
	var final_vertex_count: int = 0
	for index in range(plan.size()):
		var command: Dictionary = plan[index]
		var original: PackedVector2Array = command.original
		var points: PackedVector2Array = command.points
		if not _valid_polygon(original) or not _valid_polygon(points):
			return false
		final_vertex_count += points.size()
		var visible: int = _visible(points,viewport)
		if visible < 0:
			return false
		if visible > 0:
			drawable.append({'index':index,'command':command})
	if final_vertex_count != 74 or drawable.is_empty():
		return false
	if not _canvas_matches(canvas_value,snapshot_value,scene_value,key_value,scene_hash_value,camera_value,viewport_value):
		return false
	var floor_helper = canvas_value.get('cw_pressure_floor_material_visual')
	var texture: Texture2D = null
	if floor_helper is Object and floor_helper.has_method('cached_furnace_floor_texture'):
		texture = floor_helper.cached_furnace_floor_texture()
	var cap_groups: Array = []
	if is_instance_valid(texture):
		for i in range(6):
			var pieces = _steel_cap_pieces(camera_value,WALLS[i].polygon,CAP_HEIGHTS[i])
			if pieces.is_empty():
				cap_groups.clear()
				break
			cap_groups.append(pieces)
	# Preflight both selected sides before either replaces its legacy draw.
	var face_texture: Texture2D = null
	var face_groups: Array = []
	if canvas_value.material == null and not canvas_value.use_parent_material:
		face_texture = _low_face_atlas()
		if is_instance_valid(face_texture):
			var long_face: Array = _low_face_pieces(camera_value,0.0,24.0,16.0)
			var short_face: Array = _low_face_pieces(camera_value,23.5,24.0,6.0)
			if long_face.size() == 24 and short_face.size() == 1:
				face_groups = [long_face,short_face]
	# This is independent of the existing low-face group's preflight and fallback.
	var high_groups: Dictionary = {}
	if is_instance_valid(face_texture):
		high_groups = _high_face_groups(camera_value)
	# No command has been emitted until the complete geometry, SAT and binding pass.
	for item: Dictionary in drawable:
		var command: Dictionary = item.command
		if item.index in [0,1,6,8] and high_groups.size()==4:
			for piece: Dictionary in high_groups[item.index]:
				canvas_value.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,face_texture)
		elif (item.index == 3 or item.index == 10) and face_groups.size() == 2:
			var group: Array = face_groups[0] if item.index == 3 else face_groups[1]
			for piece: Dictionary in group:
				canvas_value.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,face_texture)
		elif item.index>=12 and cap_groups.size()==6:
			for piece in cap_groups[item.index-12]:
				canvas_value.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,texture)
		else:
			canvas_value.draw_colored_polygon(command.points,command.color)
	return true
