extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
# Same shell relief, palette and panel rhythm as gh_power_environment_visual.
const WALLS: Array = [
	{'id':'gh_gate_wall_north','polygon':[[0,0],[20,0],[20,0.5],[0,0.5]],'inner':[[0,0.5],[20,0.5]],'seams':4},
	{'id':'gh_gate_wall_south','polygon':[[0,13.5],[20,13.5],[20,14],[0,14]],'inner':[[0,13.5],[20,13.5]],'seams':4},
	{'id':'gh_gate_wall_west_north','polygon':[[0,0.5],[0.5,0.5],[0.5,5.5],[0,5.5]],'inner':[[0.5,0.5],[0.5,5.5]],'seams':1},
	{'id':'gh_gate_wall_west_south','polygon':[[0,8.5],[0.5,8.5],[0.5,13.5],[0,13.5]],'inner':[[0.5,8.5],[0.5,13.5]],'seams':1},
	{'id':'gh_gate_wall_east_north','polygon':[[19.5,0.5],[20,0.5],[20,5.5],[19.5,5.5]],'inner':[[19.5,0.5],[19.5,5.5]],'seams':1},
	{'id':'gh_gate_wall_east_south','polygon':[[19.5,8.5],[20,8.5],[20,13.5],[19.5,13.5]],'inner':[[19.5,8.5],[19.5,13.5]],'seams':1}
]
const BARRIER_ID: String = 'gh_gate_a_closed_barrier'
const BARRIER_POLYGON: Array = [[9.5,0.5],[10.5,0.5],[10.5,13.5],[9.5,13.5]]

func _definition(id_value: Variant) -> Dictionary:
	if typeof(id_value) != TYPE_STRING:
		return {}
	for definition: Dictionary in WALLS:
		if definition.id == id_value:
			return definition
	return {}

func _polygon_matches(value: Variant, expected: Array) -> bool:
	if not value is Array or value.size() != expected.size():
		return false
	for index: int in range(expected.size()):
		var point: Variant = value[index]
		if not point is Array or point.size() != 2:
			return false
		for axis: int in range(2):
			if not P.number(point[axis],float(expected[index][axis]),float(expected[index][axis])):
				return false
	return true

func _layout(snapshot: Dictionary, scene: Dictionary, key: Variant, wall: Dictionary) -> bool:
	if snapshot.get('worldId') != 'grey_hive' or scene.get('worldId') != 'grey_hive' or snapshot.get('sceneId') != 'gh_gate_a' or scene.get('sceneId') != 'gh_gate_a':
		return false
	var epoch: Variant = snapshot.get('worldEpoch')
	if not P.counter(epoch,1) or typeof(key) != TYPE_STRING or key != 'grey_hive:gh_gate_a:'+str(epoch) or not P.counter(scene.get('schemaVersion'),1,1):
		return false
	var bounds: Variant = scene.get('boundsM')
	if not bounds is Dictionary or bounds.size() != 4 or not P.number(bounds.get('x'),0,0) or not P.number(bounds.get('z'),0,0) or not P.number(bounds.get('width'),20,20) or not P.number(bounds.get('depth'),14,14):
		return false
	var rows: Variant = scene.get('collision')
	if not P.rows(rows,7) or rows.size() != 7:
		return false
	var seen: Dictionary = {}
	var matched: bool = false
	for row: Variant in rows:
		if not row is Dictionary or typeof(row.get('id')) != TYPE_STRING or seen.has(row.id):
			return false
		seen[row.id] = true
		if row.id == BARRIER_ID:
			if row.size() != 3 or row.get('requiresEvent') != 'hive_power' or not _polygon_matches(row.get('polygon'),BARRIER_POLYGON):
				return false
		else:
			var definition: Dictionary = _definition(row.id)
			if definition.is_empty() or row.size() != 2 or not _polygon_matches(row.get('polygon'),definition.polygon):
				return false
			if row == wall:
				matched = true
	return matched and seen.has(BARRIER_ID)

func _v2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _rect(value: Rect2) -> bool:
	return _v2(value.position) and _v2(value.size) and _v2(value.end) and value.size.x > 0 and value.size.y > 0

func _camera(camera: Variant) -> bool:
	if not camera is RefCounted or not is_instance_valid(camera) or camera.get_script() != ProjectionScript or not camera.initialized:
		return false
	return is_finite(camera.ppm) and camera.ppm > 0 and camera.ppm <= 256 and is_finite(camera.origin.x) and is_finite(camera.origin.y) and is_finite(camera.origin.z) and _v2(camera.screen_center) and _v2(camera.viewport) and camera.viewport.x > 0 and camera.viewport.y > 0

func _append(plan: Array[Dictionary], points: PackedVector2Array, color: Color, filled: bool, width: float = 0.0) -> void:
	plan.append({'points':points,'color':color,'filled':filled,'width':width})

func _build(definition: Dictionary, camera: Variant) -> Array[Dictionary]:
	var plan: Array[Dictionary] = []
	var scale: float = camera.ppm/48.0
	var offset: Vector2 = Vector2(0,12.0*scale)
	var edge: Array = definition.inner
	var a: Vector3 = Vector3(edge[0][0],0,edge[0][1])
	var b: Vector3 = Vector3(edge[1][0],0,edge[1][1])
	var ground_a: Vector2 = camera.project(a)
	var ground_b: Vector2 = camera.project(b)
	var visible_ground_b: Vector2 = ground_b
	if definition.id in ['gh_gate_wall_west_south','gh_gate_wall_east_south']:
		# Preserve the top endpoint; only the lower corner lies behind the south cap.
		visible_ground_b = camera.project(Vector3(b.x,0,b.z-12.0/48.0))
	_append(plan,PackedVector2Array([ground_a,visible_ground_b,ground_b-offset,ground_a-offset]),Color(0.18,0.25,0.29,1),true)
	if definition.id in ['gh_gate_wall_west_south','gh_gate_wall_east_south']:
		# Keep pre-clip consumption bounds so culling cannot expose legacy fallback.
		plan[0]['visibility_points'] = PackedVector2Array([ground_a,ground_b,ground_b-offset,ground_a-offset])
	var cap: PackedVector2Array = PackedVector2Array()
	for point: Array in definition.polygon:
		cap.append(camera.project(Vector3(point[0],0,point[1]))-offset)
	_append(plan,cap,Color(0.29,0.39,0.43,1),true)
	var outline: PackedVector2Array = cap.duplicate()
	outline.append(cap[0])
	_append(plan,outline,Color(0.53,0.66,0.68,0.8),false,1.2*scale)
	for index: int in range(1,int(definition.seams)+1):
		var low: Vector2 = camera.project(a.lerp(b,float(index)/float(definition.seams+1)))
		var high: Vector2 = low-offset
		_append(plan,PackedVector2Array([low,high]),Color(0.34,0.46,0.5,0.76),false,scale)
		var center: Vector2 = low.lerp(high,0.73)
		var fastener: PackedVector2Array = PackedVector2Array()
		for corner: int in range(6):
			var angle: float = TAU*float(corner)/6.0
			fastener.append(center+Vector2(cos(angle),sin(angle))*1.25*scale)
		_append(plan,fastener,Color(0.57,0.68,0.69,0.9),true)
	return plan

func _plan_visible(plan: Array[Dictionary], viewport: Rect2) -> bool:
	if plan.is_empty() or plan.size() > 11:
		return false
	var visible: bool = false
	for command: Dictionary in plan:
		var points: PackedVector2Array = command.points
		var color: Color = command.color
		if points.size() < (3 if command.filled else 2) or not is_finite(color.r) or not is_finite(color.g) or not is_finite(color.b) or not is_finite(color.a) or color.a <= 0 or not is_finite(command.width) or (not command.filled and command.width <= 0):
			return false
		var box: Rect2 = Rect2(points[0],Vector2.ZERO)
		for point: Vector2 in points:
			if not _v2(point):
				return false
			box = box.expand(point)
		if command.has('visibility_points'):
			for point: Vector2 in command.visibility_points:
				if not _v2(point):
					return false
				box = box.expand(point)
		if not _v2(box.end) or not _v2(box.size):
			return false
		if box.grow(command.width*0.5).intersects(viewport):
			visible = true
	return visible

# False leaves the renderer's legacy wall fallback intact, including the barrier.
func draw_wall(canvas: Variant, snapshot: Variant, scene: Variant, current_key: Variant, camera: Variant, wall: Variant, viewport: Variant) -> bool:
	if not wall is Dictionary or _definition(wall.get('id')).is_empty():
		return false
	if not canvas is CanvasItem or not is_instance_valid(canvas) or not canvas.is_inside_tree() or not canvas.is_visible_in_tree():
		return false
	if not snapshot is Dictionary or not scene is Dictionary or not viewport is Rect2 or not _rect(viewport) or not _camera(camera) or not _layout(snapshot,scene,current_key,wall):
		return false
	var plan: Array[Dictionary] = _build(_definition(wall.id),camera)
	if not _plan_visible(plan,viewport):
		return false
	# All commands are finite and staged before the first draw; no later false exit.
	for command: Dictionary in plan:
		if command.filled:
			canvas.draw_colored_polygon(command.points,command.color)
		else:
			canvas.draw_polyline(command.points,command.color,command.width,true)
	return true
