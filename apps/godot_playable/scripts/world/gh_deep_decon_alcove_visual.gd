extends RefCounted

const Boundary = preload('res://scripts/world/gh_deep_decon_environment_visual.gd')
const DeconFloor = preload('res://scripts/world/gh_deep_decon_floor_material_visual.gd')
const OWNER_IDS: Array = ["deep_decon_safety_alcove_west","deep_decon_safety_alcove_south"]
const SIDE_ROWS: Array = [{"a":[1.5,10.5],"b":[1.5,13],"clip":true},{"a":[4,13],"b":[4,13.5],"clip":false},{"a":[1,13.5],"b":[4,13.5],"clip":false}]
const CAP_RECTS: Array = [[1,10.5,1.5,13],[1,13,4,13.5]]

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

var _guard = Boundary.new()

func owns_collision(wall_value: Variant) -> bool:
	if not wall_value is Dictionary:
		return false
	for index: int in range(OWNER_IDS.size()):
		if wall_value.get('id') == OWNER_IDS[index] and _guard._same_json(wall_value,Boundary.CANONICAL_SCENE.collision[6+index]):
			return true
	return false

func _binding_ok(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, projection: Variant, viewport: Variant) -> bool:
	return _guard._binding_ok(canvas,snapshot,scene,key,scene_hash,projection,viewport)

func _stage(projection: RefCounted, viewport: Rect2) -> Dictionary:
	var all: Array[Dictionary] = []
	var visible: Array[Dictionary] = []
	for row_value: Variant in SIDE_ROWS:
		if not row_value is Dictionary or not row_value.get('a') is Array or not row_value.get('b') is Array or typeof(row_value.get('clip')) != TYPE_BOOL:
			return {}
		var row: Dictionary = row_value
		var a: Array = row.a
		var b: Array = row.b
		if a.size() != 2 or b.size() != 2:
			return {}
		var ground_a: Vector3 = Vector3(a[0],0.0,a[1])
		var ground_b: Vector3 = Vector3(b[0],0.0,b[1])
		var top_a: Vector3 = Vector3(a[0],Boundary.HEIGHT,a[1])
		var top_b: Vector3 = Vector3(b[0],Boundary.HEIGHT,b[1])
		var axis: Vector3 = (ground_b-ground_a).normalized()
		if not is_finite(axis.x) or not is_finite(axis.y) or not is_finite(axis.z) or axis == Vector3.ZERO:
			return {}
		var lower_b: Vector3 = ground_b-axis*Boundary.HEIGHT if row.clip else ground_b
		var original: PackedVector2Array = _guard._project_world(projection,[ground_a,ground_b,top_b,top_a])
		var points: PackedVector2Array = _guard._project_world(projection,[ground_a,lower_b,top_b,top_a])
		if not _guard._quad_ok(original,Boundary.SIDE_COLOR) or not _guard._quad_ok(points,Boundary.SIDE_COLOR):
			return {}
		var overlap: int = _guard._visible(points,viewport)
		if overlap < 0:
			return {}
		var command: Dictionary = {'original':original,'points':points,'color':Boundary.SIDE_COLOR}
		all.append(command)
		if overlap == 1:
			visible.append(command)
	for cap_index: int in range(CAP_RECTS.size()):
		var rect_value: Variant = CAP_RECTS[cap_index]
		if not rect_value is Array or rect_value.size() != 4:
			return {}
		var rect: Array = rect_value
		var cap_points: PackedVector2Array = _guard._project_world(projection,[Vector3(rect[0],Boundary.HEIGHT,rect[1]),Vector3(rect[2],Boundary.HEIGHT,rect[1]),Vector3(rect[2],Boundary.HEIGHT,rect[3]),Vector3(rect[0],Boundary.HEIGHT,rect[3])])
		if not _guard._quad_ok(cap_points,Boundary.CAP_COLOR):
			return {}
		var cap_overlap: int = _guard._visible(cap_points,viewport)
		if cap_overlap < 0:
			return {}
		var cap_command: Dictionary = {'original':cap_points,'points':cap_points,'color':Boundary.CAP_COLOR,'cap_index':cap_index}
		all.append(cap_command)
		if cap_overlap == 1:
			visible.append(cap_command)
	return {'all':all,'visible':visible} if all.size() == 5 else {}

func _stamp(snapshot: Dictionary, projection: RefCounted, key: String, scene_hash: String) -> Array:
	return [snapshot.duplicate(true),_guard._capture(snapshot,projection),key,scene_hash,OWNER_IDS[0]]

func _receipt_matches(receipt: Dictionary, canvas: Variant, snapshot: Dictionary, projection: RefCounted, key: String, scene_hash: String, owner_index: int) -> bool:
	if owner_index != 1 or receipt.size() != 3 or not receipt.has('stamp') or not receipt.has('camera') or not receipt.has('canvas') or receipt.camera != projection or receipt.canvas != canvas:
		return false
	var stamp: Variant = receipt.stamp
	return stamp is Array and stamp.size() == 5 and stamp[0] is Dictionary and _guard._same_json(stamp[0],snapshot) and stamp[1] is Array and _guard._capture_matches(snapshot,projection,stamp[1]) and stamp[2] == key and stamp[3] == scene_hash and stamp[4] == OWNER_IDS[0]

func _draw_floor_cap(canvas: CanvasItem, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, projection: RefCounted, viewport: Rect2, cap_index: int, floor_helper: Variant, floor_packet: Variant) -> void:
	if cap_index < 0 or cap_index >= CAP_RECTS.size() or not floor_helper is RefCounted or floor_helper.get_script() != DeconFloor or not floor_helper.is_packet_valid(floor_packet,snapshot,scene,key,scene_hash,projection):
		return
	var texture = floor_helper.get('floor_texture')
	if not texture is ImageTexture or not is_instance_valid(texture) or not floor_helper.get('definition_valid') or not floor_helper.get('texture_attempted'):
		return
	var rect: Array = CAP_RECTS[cap_index]
	var nx: int = int(round((rect[2]-rect[0])*2.0))
	var nz: int = int(round((rect[3]-rect[1])*2.0))
	if nx < 1 or nz < 1 or nx*nz > 6 or (nx != 1 and nz != 1) or rect[0]+float(nx)*0.5 != rect[2] or rect[1]+float(nz)*0.5 != rect[3]:
		return
	var uv: PackedVector2Array = PackedVector2Array()
	for pixel in DeconFloor.PIXEL_CENTERS:
		var point: Vector2 = Vector2((pixel[0]+0.5)/float(DeconFloor.CANVAS_SIZE.x),(pixel[1]+0.5)/float(DeconFloor.CANVAS_SIZE.y))
		if not _guard._v2(point):
			return
		uv.append(point)
	if uv.size() != 4:
		return
	var patches: Array[PackedVector2Array] = []
	for z in range(nz):
		for x in range(nx):
			var px: float = float(rect[0])+float(x)*0.5
			var pz: float = float(rect[1])+float(z)*0.5
			var points: PackedVector2Array = _guard._project_world(projection,[Vector3(px,Boundary.HEIGHT,pz),Vector3(px+0.5,Boundary.HEIGHT,pz),Vector3(px+0.5,Boundary.HEIGHT,pz+0.5),Vector3(px,Boundary.HEIGHT,pz+0.5)])
			if not _guard._quad_ok(points,Color.WHITE):
				return
			var visible: int = _guard._visible(points,viewport)
			if visible < 0:
				return
			if visible == 1:
				patches.append(points)
	var tint: PackedColorArray = PackedColorArray([Color.WHITE,Color.WHITE,Color.WHITE,Color.WHITE])
	for points in patches:
		canvas.draw_polygon(points,tint,uv,texture)

# The third side spans three integer-X cells at the original face height.
func _low_face_pieces(projection: RefCounted) -> Array:
	var pieces: Array = []
	var uv: PackedVector2Array = PackedVector2Array()
	for pixel: Vector2 in [Vector2(50,322),Vector2(300,460),Vector2(302.6,396.2),Vector2(52.6,258.2)]:
		if not _guard._v2(pixel) or pixel.x < 0.0 or pixel.y < 0.0 or pixel.x > LOW_FACE_SIZE.x or pixel.y > LOW_FACE_SIZE.y:
			return []
		uv.append(pixel/LOW_FACE_SIZE)
	for x: int in range(1,4):
		var points: PackedVector2Array = _guard._project_world(projection,[Vector3(x,0,13.5),Vector3(x+1,0,13.5),Vector3(x+1,Boundary.HEIGHT,13.5),Vector3(x,Boundary.HEIGHT,13.5)])
		if not _guard._quad_ok(points,Color.WHITE):
			return []
		pieces.append({'points':points,'uv':uv})
	return pieces

func draw_or_consume(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, scene_hash_value: Variant, projection_value: Variant, wall_value: Variant, viewport_value: Variant, receipt: Dictionary, floor_helper: Variant, floor_packet: Variant, floor_drawn: bool) -> bool:
	if not _binding_ok(canvas_value,snapshot_value,scene_value,key_value,scene_hash_value,projection_value,viewport_value):
		return false
	var owner_index: int = -1
	for index: int in range(OWNER_IDS.size()):
		if wall_value is Dictionary and wall_value.get('id') == OWNER_IDS[index] and _guard._same_json(wall_value,Boundary.CANONICAL_SCENE.collision[6+index]):
			owner_index = index
			break
	if owner_index < 0:
		return false
	var snapshot: Dictionary = snapshot_value
	var projection: RefCounted = projection_value
	var key: String = key_value
	var scene_hash: String = scene_hash_value
	if owner_index == 1:
		return _receipt_matches(receipt,canvas_value,snapshot,projection,key,scene_hash,owner_index)
	if not receipt.is_empty():
		return false
	var captured: Array = _guard._capture(snapshot,projection)
	var plan: Dictionary = _stage(projection,viewport_value)
	if plan.is_empty() or not plan.has('all') or plan.all.size() != 5 or not plan.has('visible') or plan.visible.is_empty():
		return false
	if not _binding_ok(canvas_value,snapshot_value,scene_value,key_value,scene_hash_value,projection_value,viewport_value) or not _guard._capture_matches(snapshot,projection,captured):
		return false
	var canvas: CanvasItem = canvas_value
	# Preflight all three cells before any command in the five-item union.
	var face_texture: Texture2D = null
	var face_pieces: Array = []
	if Boundary.HEIGHT == 0.25 and canvas.material == null and not canvas.use_parent_material:
		face_texture = _low_face_atlas()
		if is_instance_valid(face_texture):
			face_pieces = _low_face_pieces(projection)
	if face_pieces.size() != 3:
		face_pieces.clear()
	for command: Dictionary in plan.visible:
		if command == plan.all[2] and face_pieces.size() == 3:
			for piece: Dictionary in face_pieces:
				canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,face_texture)
		else:
			canvas.draw_colored_polygon(command.points,command.color)
		if command.has('cap_index') and floor_drawn:
			_draw_floor_cap(canvas,snapshot_value,scene_value,key_value,scene_hash_value,projection,viewport_value,command.cap_index,floor_helper,floor_packet)
	receipt['stamp'] = [snapshot.duplicate(true),captured.duplicate(true),key,scene_hash,OWNER_IDS[0]]
	receipt['camera'] = projection
	receipt['canvas'] = canvas
	return true
