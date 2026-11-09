extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const GHPalette = preload('res://scripts/world/gh_palette.gd')
# Same shell relief, palette and panel rhythm as gh_power_environment_visual.
const WALLS: Array = [
	{'id':'gh_gate_wall_north','polygon':[[0,0],[20,0],[20,0.5],[0,0.5]],'inner':[[0,0.5],[20,0.5]],'seams':4},
	{'id':'gh_gate_wall_south','polygon':[[0,13.5],[20,13.5],[20,14],[0,14]],'inner':[[0,13.5],[20,13.5]],'seams':4},
	{'id':'gh_gate_wall_west_north','polygon':[[0,0.5],[0.5,0.5],[0.5,5.5],[0,5.5]],'inner':[[0.5,0.5],[0.5,5.5]],'seams':1},
	{'id':'gh_gate_wall_west_south','polygon':[[0,8.5],[0.5,8.5],[0.5,13.5],[0,13.5]],'inner':[[0.5,8.5],[0.5,13.5]],'seams':1},
	{'id':'gh_gate_wall_east_north','polygon':[[19.5,0.5],[20,0.5],[20,5.5],[19.5,5.5]],'inner':[[19.5,0.5],[19.5,5.5]],'seams':1},
	{'id':'gh_gate_wall_east_south','polygon':[[19.5,8.5],[20,8.5],[20,13.5],[19.5,13.5]],'inner':[[19.5,8.5],[19.5,13.5]],'seams':1}
]
const EXTERIOR_WALL_EDGES: Dictionary = {
	'gh_gate_wall_south': [[0.0,14.0],[20.0,14.0]],
	'gh_gate_wall_east_north': [[20.0,0.5],[20.0,5.5]],
	'gh_gate_wall_east_south': [[20.0,8.5],[20.0,13.5]]
}
const BARRIER_ID: String = 'gh_gate_a_closed_barrier'
const BARRIER_POLYGON: Array = [[9.5,0.5],[10.5,0.5],[10.5,13.5],[9.5,13.5]]

# GA_SHORT_DATA_BEGIN
const SHORT_SCENE_SHA: String = "3c62f3c35a2c9f2e935707e7a93c34a8d6f5696ed730b8487c4efe0b6d1ee484"
const SHORT_HEIGHT: float = 12.0/48.0
const SHORT_SIDE_COLOR: Color = Color(0.18,0.25,0.29,1.0)
const SHORT_CANONICAL_SCENE: Dictionary = {"boundsM":{"depth":14.0,"width":20.0,"x":0,"z":0},"cameraZones":[],"checkpoints":[{"id":"gh_gate_a_checkpoint","navNode":"gh_gate_a_checkpoint_nav","position":[3.0,0.0,7.0]}],"collision":[{"id":"gh_gate_wall_north","polygon":[[0.0,0.0],[20.0,0.0],[20.0,0.5],[0.0,0.5]]},{"id":"gh_gate_wall_south","polygon":[[0.0,13.5],[20.0,13.5],[20.0,14.0],[0.0,14.0]]},{"id":"gh_gate_wall_west_north","polygon":[[0.0,0.5],[0.5,0.5],[0.5,5.5],[0.0,5.5]]},{"id":"gh_gate_wall_west_south","polygon":[[0.0,8.5],[0.5,8.5],[0.5,13.5],[0.0,13.5]]},{"id":"gh_gate_wall_east_north","polygon":[[19.5,0.5],[20.0,0.5],[20.0,5.5],[19.5,5.5]]},{"id":"gh_gate_wall_east_south","polygon":[[19.5,8.5],[20.0,8.5],[20.0,13.5],[19.5,13.5]]},{"id":"gh_gate_a_closed_barrier","polygon":[[9.5,0.5],[10.5,0.5],[10.5,13.5],[9.5,13.5]],"requiresEvent":"hive_power"}],"doors":[{"assetId":"runtime2d.prop.gate_a.closed_open.v1","id":"gh_gate_a","position":[10.0,0.0,7.0],"requiresEvent":"hive_power"}],"hazards":[],"interactions":[{"event":null,"id":"gh_gate_a_panel","kind":"door_panel","position":[8.0,0.0,7.0]}],"logic":{"traversal":[]},"navigation":{"links":[{"from":"gh_gate_a_spawn_nav","to":"gh_gate_a_checkpoint_nav"},{"from":"gh_gate_a_checkpoint_nav","to":"gh_gate_a_panel_nav"}],"nodes":[{"id":"gh_gate_a_spawn_nav","position":[2.0,0.0,7.0]},{"id":"gh_gate_a_checkpoint_nav","position":[3.0,0.0,7.0]},{"id":"gh_gate_a_panel_nav","position":[8.0,0.0,7.0]}]},"objectives":[],"occluders":[],"presentation":{"cameraProfile":"facility_gate_corridor","layers":[{"id":"visual.floor","zGroup":0},{"id":"visual.props_back","zGroup":10},{"id":"visual.props_dynamic","zGroup":20}],"sprites":[{"assetId":"runtime2d.grey_hive.floor_tiles.v1","id":"approved_floor_atlas","layer":"visual.floor","position":[10.0,0.0,7.0]},{"assetId":"runtime2d.grey_hive.wall_tiles.v1","id":"gate_a_corridor_wall","layer":"visual.props_back","position":[1.0,0.0,2.0]},{"assetId":"runtime2d.grey_hive.pipes_cables.v1","id":"gate_a_cabling","layer":"visual.props_back","position":[6.0,0.0,2.0]},{"assetId":"runtime2d.prop.supply_crates.v1","id":"gate_a_service_crates","layer":"visual.props_back","position":[14.0,0.0,10.0]},{"assetId":"runtime2d.prop.gate_a.closed_open.v1","id":"gate_a_closed_open_visual","layer":"visual.props_dynamic","position":[10.0,0.0,7.0]},{"assetId":"runtime2d.enemy.infected_security.v1","id":"gate_security_01_visual","layer":"visual.props_dynamic","position":[6.0,0.0,4.0]},{"assetId":"runtime2d.enemy.infected_security.v1","id":"gate_security_02_visual","layer":"visual.props_dynamic","position":[8.0,0.0,10.0]}]},"sceneId":"gh_gate_a","schemaVersion":1,"spawns":[{"entityType":null,"id":"gh_gate_a_spawn","kind":"player","navNode":"gh_gate_a_spawn_nav","position":[2.0,0.0,7.0]},{"entityType":"grey_hive.infected_security","id":"gh_gate_a_security_01","kind":"enemy","navNode":"gh_gate_a_checkpoint_nav","position":[6.0,0.0,4.0]},{"entityType":"grey_hive.infected_security","id":"gh_gate_a_security_02","kind":"enemy","navNode":"gh_gate_a_panel_nav","position":[8.0,0.0,10.0]}],"transitions":[{"id":"gh_gate_a_return_to_power_room","polygon":[[0.0,6.0],[3.0,6.0],[3.0,8.0],[0.0,8.0]],"spawnId":"gh_power_spawn","toSceneId":"gh_power_room"},{"id":"gh_gate_a_to_shaft","polygon":[[17.5,6.0],[20.0,6.0],[20.0,8.0],[17.5,8.0]],"requiresEvent":"hive_power","spawnId":"gh_shaft_spawn","toSceneId":"gh_central_shaft"}],"triggers":[],"vfxMarkers":[],"worldId":"grey_hive"}
const SHORT_FACES: Array = [
	{"id":"north_east_end","owner":"gh_gate_wall_north","a":[20,0],"b":[20,0.5],"low":0,"high":0.25},
	{"id":"south_east_end","owner":"gh_gate_wall_south","a":[20,13.5],"b":[20,14],"low":0,"high":0.25},
	{"id":"west_north_jamb","owner":"gh_gate_wall_west_north","a":[0,5.5],"b":[0.5,5.5],"low":0,"high":0.25},
	{"id":"east_north_jamb","owner":"gh_gate_wall_east_north","a":[19.5,5.5],"b":[20,5.5],"low":0,"high":0.25}
]
# GA_SHORT_DATA_END

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
	_append(plan,PackedVector2Array([ground_a,visible_ground_b,ground_b-offset,ground_a-offset]),GHPalette.FACE,true)
	if definition.id in ['gh_gate_wall_west_south','gh_gate_wall_east_south']:
		# Keep pre-clip consumption bounds so culling cannot expose legacy fallback.
		plan[0]['visibility_points'] = PackedVector2Array([ground_a,ground_b,ground_b-offset,ground_a-offset])
	var cap: PackedVector2Array = PackedVector2Array()
	for point: Array in definition.polygon:
		cap.append(camera.project(Vector3(point[0],0,point[1]))-offset)
	_append(plan,cap,GHPalette.CAP,true)
	var outline: PackedVector2Array = cap.duplicate()
	outline.append(cap[0])
	_append(plan,outline,GHPalette.EDGE,false,1.2*scale)
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

func _addition_binding_matches(canvas: Variant, snapshot: Dictionary, scene: Dictionary, current_key: Variant, camera: Variant, viewport: Rect2) -> bool:
	if not P.snapshot(snapshot) or snapshot.get('entryToken') != null:
		return false
	var canvas_snapshot: Variant = canvas.get('snapshot')
	var canvas_scene: Variant = canvas.get('scene')
	var canvas_key: Variant = canvas.get('current_key')
	var canvas_camera: Variant = canvas.get('camera')
	var canvas_viewport: Variant = canvas.get('viewport_rect')
	var initialized: Variant = camera.get('initialized')
	return canvas_snapshot == snapshot and canvas_scene == scene and canvas_key == current_key and canvas_camera == camera and canvas_viewport == viewport and canvas.get_viewport_rect() == viewport and camera.viewport == viewport.size and typeof(initialized) == TYPE_BOOL and initialized == true

func _try_add_exterior_face(plan: Array[Dictionary], canvas: Variant, snapshot: Dictionary, scene: Dictionary, current_key: Variant, camera: Variant, wall: Dictionary, viewport: Rect2) -> int:
	var wall_id: Variant = wall.get('id')
	if typeof(wall_id) != TYPE_STRING or not EXTERIOR_WALL_EDGES.has(wall_id):
		return 0
	if not _addition_binding_matches(canvas,snapshot,scene,current_key,camera,viewport):
		return 0
	var edge: Variant = EXTERIOR_WALL_EDGES.get(wall_id)
	if not edge is Array or edge.size() != 2 or not edge[0] is Array or not edge[1] is Array or edge[0].size() != 2 or edge[1].size() != 2:
		return -1
	for point: Variant in edge:
		for coordinate: Variant in point:
			if not P.number(coordinate):
				return -1
	var ground_a: Vector2 = camera.project(Vector3(float(edge[0][0]),0.0,float(edge[0][1])))
	var ground_b: Vector2 = camera.project(Vector3(float(edge[1][0]),0.0,float(edge[1][1])))
	var relief: Vector2 = Vector2(0.0,12.0*camera.ppm/48.0)
	var quad: PackedVector2Array = PackedVector2Array([ground_a,ground_b,ground_b-relief,ground_a-relief])
	for point: Vector2 in quad:
		if not _v2(point):
			return -1
	var area: float = (quad[1]-quad[0]).cross(quad[3]-quad[0])
	if not is_finite(area):
		return -1
	if area == 0.0:
		return 0
	var color: Color = GHPalette.FACE # Color(0.18,0.25,0.29,1)
	if not is_finite(color.r) or not is_finite(color.g) or not is_finite(color.b) or not is_finite(color.a):
		return -1
	var staged: Array[Dictionary] = []
	_append(staged,quad,color,true)
	if staged.size() != 1:
		return -1
	plan.insert(1,staged[0])
	return 1

func _plan_visible(plan: Array[Dictionary], viewport: Rect2, extra_count: int = 0) -> bool:
	if plan.is_empty() or extra_count < 0 or extra_count > 1 or plan.size() > 11+extra_count:
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
func whole_static_ready(canvas: Variant, snapshot: Variant, scene: Variant, current_key: Variant, camera: Variant, viewport: Variant) -> bool:
	if not canvas is CanvasItem or not is_instance_valid(canvas) or not canvas.is_inside_tree() or not canvas.is_visible_in_tree(): return false
	if not snapshot is Dictionary or not scene is Dictionary or not viewport is Rect2 or not _rect(viewport) or not _camera(camera): return false
	var collision = scene.get('collision')
	if not collision is Array: return false
	var seen: Dictionary = {}
	for wall in collision:
		if not wall is Dictionary: return false
		var definition = _definition(wall.get('id'))
		if definition.is_empty(): continue
		if seen.has(wall.id) or not _layout(snapshot,scene,current_key,wall): return false
		var plan: Array[Dictionary] = _build(definition,camera)
		var extra = _try_add_exterior_face(plan,canvas,snapshot,scene,current_key,camera,wall,viewport)
		if extra<0 or not _plan_visible(plan,viewport,extra): return false
		seen[wall.id] = true
	return seen.size()==6

func draw_wall(canvas: Variant, snapshot: Variant, scene: Variant, current_key: Variant, camera: Variant, wall: Variant, viewport: Variant) -> bool:
	if not wall is Dictionary or _definition(wall.get('id')).is_empty():
		return false
	if not canvas is CanvasItem or not is_instance_valid(canvas) or not canvas.is_inside_tree() or not canvas.is_visible_in_tree():
		return false
	if not snapshot is Dictionary or not scene is Dictionary or not viewport is Rect2 or not _rect(viewport) or not _camera(camera) or not _layout(snapshot,scene,current_key,wall):
		return false
	var plan: Array[Dictionary] = _build(_definition(wall.id),camera)
	var extra_count: int = _try_add_exterior_face(plan,canvas,snapshot,scene,current_key,camera,wall,viewport)
	if extra_count < 0:
		return false
	# GA_SHORT_HOOK_BEGIN
	if _short_draw(canvas,snapshot,scene,current_key,camera,wall,viewport,plan,extra_count):
		return true
	# GA_SHORT_HOOK_END
	if not _plan_visible(plan,viewport,extra_count):
		return false
	# All commands are finite and staged before the first draw; no later false exit.
	for command: Dictionary in plan:
		if command.filled:
			canvas.draw_colored_polygon(command.points,command.color)
		else:
			canvas.draw_polyline(command.points,command.color,command.width,true)
	return true

# GA_SHORT_FUNCTION_BEGIN
func _short_equal(actual: Variant, expected: Variant, depth: int = 0) -> bool:
	if depth > 64: return false
	if typeof(expected) == TYPE_INT or typeof(expected) == TYPE_FLOAT: return P.number(actual,float(expected),float(expected))
	if typeof(actual) != typeof(expected): return false
	if expected is Dictionary:
		if actual.size() != expected.size(): return false
		for key: Variant in expected:
			if not actual.has(key) or not _short_equal(actual[key],expected[key],depth+1): return false
		return true
	if expected is Array:
		if actual.size() != expected.size(): return false
		for index: int in range(expected.size()):
			if not _short_equal(actual[index],expected[index],depth+1): return false
		return true
	return actual == expected

func _short_bindings(canvas: Variant, snapshot: Variant, scene: Variant, current_key: Variant, camera: Variant, wall: Variant, viewport: Variant) -> bool:
	if not canvas is CanvasItem or not is_instance_valid(canvas) or not canvas.is_inside_tree() or not canvas.is_visible_in_tree() or canvas.is_queued_for_deletion(): return false
	if not snapshot is Dictionary or not scene is Dictionary or not wall is Dictionary or not viewport is Rect2: return false
	var valid_snapshot: Dictionary = snapshot
	var valid_scene: Dictionary = scene
	var valid_wall: Dictionary = wall
	var view: Rect2 = viewport
	if not P.snapshot(valid_snapshot) or valid_snapshot.get('entryToken') != null or not P.counter(valid_snapshot.get('worldEpoch'),1) or typeof(current_key) != TYPE_STRING or current_key != 'grey_hive:gh_gate_a:'+str(valid_snapshot.get('worldEpoch')): return false
	if not _layout(valid_snapshot,valid_scene,current_key,valid_wall) or not _short_equal(valid_scene,SHORT_CANONICAL_SCENE) or not _camera(camera) or camera.get_script() != ProjectionScript: return false
	var initialized: Variant = camera.get('initialized')
	if typeof(initialized) != TYPE_BOOL or initialized != true or not _rect(view) or view.position != Vector2.ZERO: return false
	var camera_viewport: Variant = camera.get('viewport')
	var stored_viewport: Variant = canvas.get('viewport_rect')
	var actual_viewport: Rect2 = canvas.get_viewport_rect()
	return camera_viewport is Vector2 and camera_viewport == view.size and stored_viewport is Rect2 and stored_viewport == view and actual_viewport == view and canvas.get('snapshot') is Dictionary and _short_equal(canvas.get('snapshot'),snapshot) and canvas.get('scene') is Dictionary and _short_equal(canvas.get('scene'),scene) and canvas.get('current_key') == current_key and canvas.get('scene_hash') == SHORT_SCENE_SHA and canvas.get('camera') == camera

func _short_capture(canvas, snapshot, scene, current_key, camera, viewport) -> Dictionary:
	return {'snapshot':snapshot.duplicate(true),'scene':scene.duplicate(true),'key':current_key,'hash':SHORT_SCENE_SHA,'canvas':canvas,'camera':camera,'origin':camera.get('origin'),'center':camera.get('screen_center'),'viewport':camera.get('viewport'),'ppm':camera.get('ppm'),'viewport_rect':viewport}
func _short_unchanged(canvas: Variant, snapshot: Variant, scene: Variant, current_key: Variant, camera: Variant, wall: Variant, viewport: Variant, stamp: Variant) -> bool:
	if not stamp is Dictionary or not _short_bindings(canvas,snapshot,scene,current_key,camera,wall,viewport): return false
	return _short_equal(stamp.get('snapshot'),snapshot) and _short_equal(stamp.get('scene'),scene) and stamp.get('key') == current_key and stamp.get('hash') == SHORT_SCENE_SHA and stamp.get('canvas') == canvas and stamp.get('camera') == camera and stamp.get('origin') == camera.get('origin') and stamp.get('center') == camera.get('screen_center') and stamp.get('viewport') == camera.get('viewport') and stamp.get('ppm') == camera.get('ppm') and stamp.get('viewport_rect') == viewport

func _short_edge_matches(face: Dictionary, wall_record: Dictionary) -> bool:
	var polygon: Variant = wall_record.get('polygon')
	if not polygon is Array or polygon.size() != 4: return false
	for index: int in range(polygon.size()):
		var a: Variant = polygon[index]
		var b: Variant = polygon[(index+1)%polygon.size()]
		if _short_equal(face.get('a'),a) and _short_equal(face.get('b'),b): return true
		if _short_equal(face.get('a'),b) and _short_equal(face.get('b'),a): return true
	return false

func _short_quad(face: Dictionary, camera: Variant) -> Dictionary:
	var a: Variant = face.get('a')
	var b: Variant = face.get('b')
	var low: Variant = face.get('low')
	var high: Variant = face.get('high')
	if not a is Array or not b is Array or a.size() != 2 or b.size() != 2 or not P.number(low,0.0,0.0) or not P.number(high,SHORT_HEIGHT,SHORT_HEIGHT): return {}
	var a_points: Array = a
	var b_points: Array = b
	for coordinate: Variant in a_points:
		if not P.number(coordinate): return {}
	for coordinate: Variant in b_points:
		if not P.number(coordinate): return {}
	var low_value: float = float(low)
	var high_value: float = float(high)
	var world: Array[Vector3] = [Vector3(float(a_points[0]),low_value,float(a_points[1])),Vector3(float(b_points[0]),low_value,float(b_points[1])),Vector3(float(b_points[0]),high_value,float(b_points[1])),Vector3(float(a_points[0]),high_value,float(a_points[1]))]
	var points: PackedVector2Array = PackedVector2Array()
	for point: Vector3 in world:
		if not is_finite(point.x) or not is_finite(point.y) or not is_finite(point.z): return {}
		var projected: Variant = camera.call('project',point)
		if not projected is Vector2: return {}
		var projected_point: Vector2 = projected
		if not _v2(projected_point): return {}
		points.append(projected_point)
	if points.size() != 4: return {}
	var origin: Vector2 = points[0]
	var area2: float = 0.0
	for index: int in range(1,3):
		var first: Vector2 = points[index] - origin
		var second: Vector2 = points[index+1] - origin
		if not _v2(first) or not _v2(second): return {}
		area2 += first.cross(second)
	if not is_finite(area2) or area2 == 0.0: return {}
	var winding: float = 0.0
	for index: int in range(4):
		var edge: Vector2 = points[(index+1)%4] - points[index]
		var next_edge: Vector2 = points[(index+2)%4] - points[(index+1)%4]
		var turn: float = edge.cross(next_edge)
		if not _v2(edge) or edge == Vector2.ZERO or not _v2(next_edge) or not is_finite(turn) or turn == 0.0: return {}
		if index == 0: winding = turn
		elif (turn > 0.0) != (winding > 0.0): return {}
	var color: Color = SHORT_SIDE_COLOR
	if not is_finite(color.r) or not is_finite(color.g) or not is_finite(color.b) or not is_finite(color.a): return {}
	return {'points':points,'color':color,'filled':true,'width':0.0}

func _short_visible(points: PackedVector2Array, viewport: Rect2) -> int:
	if points.size() != 4 or not _rect(viewport) or not _v2(viewport.position) or not _v2(viewport.end): return -1
	for point: Vector2 in points:
		if not _v2(point): return -1
	var corners: PackedVector2Array = PackedVector2Array([viewport.position,Vector2(viewport.end.x,viewport.position.y),viewport.end,Vector2(viewport.position.x,viewport.end.y)])
	var axes: Array[Vector2] = [Vector2.RIGHT,Vector2.DOWN]
	for index: int in range(4):
		var edge: Vector2 = points[(index+1)%4] - points[index]
		if not _v2(edge) or edge == Vector2.ZERO: return -1
		axes.append(Vector2(-edge.y,edge.x))
	var separated: bool = false
	for axis: Vector2 in axes:
		if not _v2(axis) or axis == Vector2.ZERO: return -1
		var polygon_min: float = INF
		var polygon_max: float = -INF
		var view_min: float = INF
		var view_max: float = -INF
		for point: Vector2 in points:
			var value: float = (point-points[0]).dot(axis)
			if not is_finite(value): return -1
			polygon_min = minf(polygon_min,value)
			polygon_max = maxf(polygon_max,value)
		for point: Vector2 in corners:
			var value: float = (point-points[0]).dot(axis)
			if not is_finite(value): return -1
			view_min = minf(view_min,value)
			view_max = maxf(view_max,value)
		if not is_finite(polygon_min) or not is_finite(polygon_max) or not is_finite(view_min) or not is_finite(view_max) or polygon_min >= polygon_max or view_min >= view_max: return -1
		if polygon_max <= view_min or view_max <= polygon_min: separated = true
	return 0 if separated else 1

func _short_stage(canvas: Variant, snapshot: Variant, scene: Variant, current_key: Variant, camera: Variant, wall: Variant, viewport: Variant) -> Dictionary:
	if not _short_bindings(canvas,snapshot,scene,current_key,camera,wall,viewport): return {}
	var stamp: Dictionary = _short_capture(canvas,snapshot,scene,current_key,camera,viewport)
	var collision: Variant = scene.get('collision')
	if not collision is Array or collision.size() != 7 or not wall is Dictionary: return {}
	var collision_rows: Array = collision
	var owner_id: Variant = wall.get('id')
	var all: Array[Dictionary] = []
	var selected: Dictionary = {}
	var seen: Dictionary = {}
	for face_value: Variant in SHORT_FACES:
		if not face_value is Dictionary or face_value.size() != 6: return {}
		var face: Dictionary = face_value
		var id_value: Variant = face.get('id')
		var owner: Variant = face.get('owner')
		if typeof(id_value) != TYPE_STRING or typeof(owner) != TYPE_STRING or seen.has(id_value): return {}
		seen[id_value] = true
		var definition: Dictionary = _definition(owner)
		var wall_record: Dictionary = {}
		for row: Variant in collision_rows:
			if row is Dictionary and row.get('id') == owner: wall_record = row
		if definition.is_empty() or wall_record.is_empty() or not _short_edge_matches(face,wall_record): return {}
		var quad: Dictionary = _short_quad(face,camera)
		var points: Variant = quad.get('points')
		if not points is PackedVector2Array or points.size() != 4: return {}
		var projected_points: PackedVector2Array = points
		var view_rect: Rect2 = viewport
		var visibility: int = _short_visible(projected_points,view_rect)
		if visibility < 0: return {}
		quad['owner'] = owner
		quad['visibility'] = visibility
		all.append(quad)
		if owner == owner_id and visibility == 1:
			if not selected.is_empty(): return {}
			selected = quad
	if all.size() != 4 or selected.is_empty() or not _short_unchanged(canvas,snapshot,scene,current_key,camera,wall,viewport,stamp): return {}
	return {'face':selected,'stamp':stamp}

func _short_composite(plan: Array[Dictionary], face: Dictionary, viewport: Rect2, extra_count: int) -> Array[Dictionary]:
	if plan.is_empty() or extra_count < 0 or extra_count > 1: return []
	var copy: Array[Dictionary] = []
	for command: Variant in plan.duplicate(true):
		if not command is Dictionary: return []
		copy.append(command)
	var selected_points: Variant = face.get('points')
	if not selected_points is PackedVector2Array or selected_points.size() != 4: return []
	var face_points: PackedVector2Array = selected_points
	var first: Dictionary = copy[0]
	var witness: PackedVector2Array = PackedVector2Array()
	if first.has('visibility_points'):
		var previous: Variant = first.get('visibility_points')
		if not previous is PackedVector2Array: return []
		witness = previous.duplicate()
	for point: Vector2 in face_points: witness.append(point)
	first['visibility_points'] = witness
	copy[0] = first
	if not _plan_visible(copy,viewport,extra_count): return []
	copy.insert(1+extra_count,face.duplicate(true))
	return copy

func _short_draw(canvas: Variant, snapshot: Variant, scene: Variant, current_key: Variant, camera: Variant, wall: Variant, viewport: Variant, plan: Array[Dictionary], extra_count: int) -> bool:
	var staged: Dictionary = _short_stage(canvas,snapshot,scene,current_key,camera,wall,viewport)
	if staged.is_empty(): return false
	var face_value: Variant = staged.get('face')
	var stamp_value: Variant = staged.get('stamp')
	if not face_value is Dictionary or not stamp_value is Dictionary: return false
	var face: Dictionary = face_value
	var stamp: Dictionary = stamp_value
	var rect: Rect2 = viewport
	var merged: Array[Dictionary] = _short_composite(plan,face,rect,extra_count)
	if merged.size() != plan.size()+1 or not _short_unchanged(canvas,snapshot,scene,current_key,camera,wall,viewport,stamp): return false
	for command: Dictionary in merged:
		if command.get('filled'):
			canvas.draw_colored_polygon(command.get('points'),command.get('color'))
		else:
			canvas.draw_polyline(command.get('points'),command.get('color'),command.get('width'),true)
	return true
# GA_SHORT_FUNCTION_END
