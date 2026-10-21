extends RefCounted

const Boundary = preload('res://scripts/world/gh_deep_decon_environment_visual.gd')
const OWNER_IDS: Array = ["deep_decon_safety_alcove_west","deep_decon_safety_alcove_south"]
const SIDE_ROWS: Array = [{"a":[1.5,10.5],"b":[1.5,13],"clip":true},{"a":[4,13],"b":[4,13.5],"clip":false},{"a":[1,13.5],"b":[4,13.5],"clip":false}]
const CAP_RECTS: Array = [[1,10.5,1.5,13],[1,13,4,13.5]]

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
	for rect_value: Variant in CAP_RECTS:
		if not rect_value is Array or rect_value.size() != 4:
			return {}
		var rect: Array = rect_value
		var cap_points: PackedVector2Array = _guard._project_world(projection,[Vector3(rect[0],Boundary.HEIGHT,rect[1]),Vector3(rect[2],Boundary.HEIGHT,rect[1]),Vector3(rect[2],Boundary.HEIGHT,rect[3]),Vector3(rect[0],Boundary.HEIGHT,rect[3])])
		if not _guard._quad_ok(cap_points,Boundary.CAP_COLOR):
			return {}
		var cap_overlap: int = _guard._visible(cap_points,viewport)
		if cap_overlap < 0:
			return {}
		var cap_command: Dictionary = {'original':cap_points,'points':cap_points,'color':Boundary.CAP_COLOR}
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

func draw_or_consume(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, key_value: Variant, scene_hash_value: Variant, projection_value: Variant, wall_value: Variant, viewport_value: Variant, receipt: Dictionary) -> bool:
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
	for command: Dictionary in plan.visible:
		canvas.draw_colored_polygon(command.points,command.color)
	receipt['stamp'] = [snapshot.duplicate(true),captured.duplicate(true),key,scene_hash,OWNER_IDS[0]]
	receipt['camera'] = projection
	receipt['canvas'] = canvas
	return true
