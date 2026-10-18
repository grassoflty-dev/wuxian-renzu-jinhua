extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const FurnaceScript = preload('res://scripts/world/cw_furnace_visual.gd')
const WALLS: Dictionary = {
	'cw_regulator_core_wall_north': [[[0,0],[24,0],[24,0.5],[0,0.5]], [0,0.5], [24,0.5], [0,-1], 0.85],
	'cw_regulator_core_wall_south': [[[0,15.5],[24,15.5],[24,16],[0,16]], [0,15.5], [24,15.5], [0,1], 0.18],
	'cw_regulator_core_wall_west_north': [[[0,0.5],[0.5,0.5],[0.5,6],[0,6]], [0.5,0.5], [0.5,6], [-1,0], 0.85],
	'cw_regulator_core_wall_west_south': [[[0,9.5],[0.5,9.5],[0.5,15.5],[0,15.5]], [0.5,9.5], [0.5,15.5], [-1,0], 0.85],
	'cw_regulator_core_wall_east_north': [[[23.5,0.5],[24,0.5],[24,6],[23.5,6]], [23.5,0.5], [23.5,6], [1,0], 0.18],
	'cw_regulator_core_wall_east_south': [[[23.5,9.5],[24,9.5],[24,15.5],[23.5,15.5]], [23.5,9.5], [23.5,15.5], [1,0], 0.18]
}
const SPRITES: Dictionary = {
	'cw_regulator_core_steel_floor': ['runtime2d.world.clockworks.floor_steel.v1','visual.floor',[12,0,8]],
	'cw_regulator_core_factory_tiles': ['runtime2d.world.clockworks.factory_tiles.v1','visual.floor',[12,0,8]],
	'cw_regulator_pressure_pipes': ['runtime2d.world.clockworks.pressure_pipes.v1','visual.props_back',[10,0,4]],
	'cw_regulator_furnace_v2': ['runtime2d.world.clockworks.furnace.v2','visual.props_back',[16,0,6]],
	'cw_regulator_factory_module': ['runtime2d.world.clockworks.factory_tiles.v1','visual.props_dynamic',[20,0,11]],
	'cw_regulator_furnace_v2_side': ['runtime2d.world.clockworks.furnace.v2','visual.props_dynamic',[12,0,12]]
}

func _vec_finite(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _rect_valid(rect: Rect2) -> bool:
	return _vec_finite(rect.position) and _vec_finite(rect.size) and _vec_finite(rect.end) and rect.size.x > 0.0 and rect.size.y > 0.0

func _numbers(value: Variant, expected: Array) -> bool:
	if not value is Array or value.size() != expected.size():
		return false
	for index: int in range(expected.size()):
		var actual: Variant = value[index]
		var target: Variant = expected[index]
		if target is Array:
			if not _numbers(actual,target):
				return false
		elif not P.number(actual,float(target)-0.000001,float(target)+0.000001):
			return false
	return true

func _layout(scene: Dictionary) -> bool:
	if scene.get('worldId') != 'clockworks' or scene.get('sceneId') != 'cw_regulator_core' or not P.counter(scene.get('schemaVersion'),1,1):
		return false
	var bounds_value: Variant = scene.get('boundsM')
	if not bounds_value is Dictionary:
		return false
	var bounds: Dictionary = bounds_value
	if not P.number(bounds.get('x'),0,0) or not P.number(bounds.get('z'),0,0) or not P.number(bounds.get('width'),24,24) or not P.number(bounds.get('depth'),16,16):
		return false
	var walls_value: Variant = scene.get('collision')
	if not walls_value is Array or walls_value.size() != 6:
		return false
	var seen: Dictionary = {}
	for row_value: Variant in walls_value:
		if not row_value is Dictionary:
			return false
		var row: Dictionary = row_value
		var id_value: Variant = row.get('id')
		if typeof(id_value) != TYPE_STRING or not WALLS.has(id_value) or seen.has(id_value):
			return false
		var definition: Array = WALLS[id_value]
		if not _numbers(row.get('polygon'),definition[0]):
			return false
		seen[id_value] = true
	var presentation_value: Variant = scene.get('presentation')
	if not presentation_value is Dictionary:
		return false
	var sprites_value: Variant = presentation_value.get('sprites')
	if not sprites_value is Array or sprites_value.size() != 6:
		return false
	seen.clear()
	for sprite_value: Variant in sprites_value:
		if not sprite_value is Dictionary:
			return false
		var sprite: Dictionary = sprite_value
		var sprite_id: Variant = sprite.get('id')
		if typeof(sprite_id) != TYPE_STRING or not SPRITES.has(sprite_id) or seen.has(sprite_id):
			return false
		var expected: Array = SPRITES[sprite_id]
		if sprite.get('assetId') != expected[0] or sprite.get('layer') != expected[1] or not _numbers(sprite.get('position'),expected[2]):
			return false
		seen[sprite_id] = true
	return true

func _context(canvas: Variant, snapshot_value: Variant, scene_value: Variant, current_key: Variant, projection: Variant, viewport: Variant) -> bool:
	if not canvas is CanvasItem or not is_instance_valid(canvas) or not canvas.is_inside_tree() or canvas.is_queued_for_deletion():
		return false
	if not snapshot_value is Dictionary or not P.snapshot(snapshot_value) or snapshot_value.get('entryToken') != null or not scene_value is Dictionary:
		return false
	var snapshot: Dictionary = snapshot_value
	if snapshot.get('worldId') != 'clockworks' or snapshot.get('sceneId') != 'cw_regulator_core' or not P.counter(snapshot.get('worldEpoch'),1):
		return false
	if typeof(current_key) != TYPE_STRING or current_key != 'clockworks:cw_regulator_core:'+str(snapshot.worldEpoch) or not _layout(scene_value):
		return false
	if not projection is RefCounted or not is_instance_valid(projection) or projection.get_script() != ProjectionScript or not viewport is Rect2:
		return false
	var initialized: Variant = projection.get('initialized')
	var origin_value: Variant = projection.get('origin')
	var center_value: Variant = projection.get('screen_center')
	var viewport_value: Variant = projection.get('viewport')
	if typeof(initialized) != TYPE_BOOL or not initialized or not origin_value is Vector3 or not center_value is Vector2 or not viewport_value is Vector2 or not P.number(projection.get('ppm'),0.000001,256):
		return false
	var origin: Vector3 = origin_value
	var center: Vector2 = center_value
	var camera_viewport: Vector2 = viewport_value
	var view: Rect2 = viewport
	return is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and _vec_finite(center) and _vec_finite(camera_viewport) and _rect_valid(view) and camera_viewport == view.size

func _polygon(plan: Array[Dictionary], projection: Variant, world: Array[Vector3], color: Color) -> bool:
	var points: PackedVector2Array = PackedVector2Array()
	for value: Vector3 in world:
		var point: Vector2 = projection.project(value)
		if not _vec_finite(point):
			return false
		points.append(point)
	plan.append({'points':points,'color':color,'width':0.0})
	return true

func _quad(plan: Array[Dictionary], projection: Variant, x0: float, z0: float, x1: float, z1: float, color: Color) -> bool:
	var world: Array[Vector3] = [Vector3(x0,0,z0),Vector3(x1,0,z0),Vector3(x1,0,z1),Vector3(x0,0,z1)]
	return _polygon(plan,projection,world,color)

func _line(plan: Array[Dictionary], projection: Variant, a: Vector3, b: Vector3, color: Color, width: float) -> bool:
	var start: Vector2 = projection.project(a)
	var finish: Vector2 = projection.project(b)
	if not _vec_finite(start) or not _vec_finite(finish) or not is_finite(width) or width <= 0.0:
		return false
	plan.append({'points':PackedVector2Array([start,finish]),'color':color,'width':width})
	return true

func _plan_valid(plan: Array[Dictionary], limit: int) -> bool:
	if plan.is_empty() or plan.size() > limit:
		return false
	var low: Vector2 = Vector2(INF,INF)
	var high: Vector2 = Vector2(-INF,-INF)
	for command: Dictionary in plan:
		var points: PackedVector2Array = command.points
		var color: Color = command.color
		var width: float = command.width
		if not is_finite(color.r) or not is_finite(color.g) or not is_finite(color.b) or not is_finite(color.a) or color.r < 0.0 or color.g < 0.0 or color.b < 0.0 or color.a < 0.0 or color.r > 1.0 or color.g > 1.0 or color.b > 1.0 or color.a > 1.0 or not is_finite(width) or width < 0.0:
			return false
		if points.size() < 2 or (width == 0.0 and points.size() < 3):
			return false
		for point: Vector2 in points:
			if not _vec_finite(point):
				return false
			low = low.min(point)
			high = high.max(point)
	return _rect_valid(Rect2(low,high-low))

func _emit(canvas: CanvasItem, plan: Array[Dictionary]) -> void:
	for command: Dictionary in plan:
		var points: PackedVector2Array = command.points
		var color: Color = command.color
		var width: float = command.width
		if width > 0.0:
			canvas.draw_line(points[0],points[1],color,width,true)
		else:
			canvas.draw_colored_polygon(points,color)

func _bounce(plan: Array[Dictionary], projection: Variant, viewport: Rect2, furnace_visual: Variant) -> bool:
	if not furnace_visual is RefCounted or not is_instance_valid(furnace_visual) or furnace_visual.get_script() != FurnaceScript:
		return true
	var textures_value: Variant = furnace_visual.get('textures')
	if not textures_value is Dictionary:
		return true
	var texture_value: Variant = textures_value.get('furnace_v2')
	if not texture_value is Texture2D or not is_instance_valid(texture_value) or texture_value.get_width() != 1165 or texture_value.get_height() != 1240:
		return true
	var centers: Array[Vector2] = [Vector2(16,6),Vector2(12,12)]
	var radii: Array[Vector3] = [Vector3(1.35,0.55,0.018),Vector3(0.95,0.38,0.026),Vector3(0.5,0.2,0.035)]
	for center: Vector2 in centers:
		var foot: Vector2 = projection.project(Vector3(center.x,0,center.y))
		var rect_value: Variant = furnace_visual.canvas_rect('furnace_v2',foot,float(projection.get('ppm')))
		if not rect_value is Rect2:
			return false
		var rect: Rect2 = rect_value
		if not _rect_valid(rect):
			return false
		if not viewport.intersects(rect):
			continue
		for radius: Vector3 in radii:
			var world: Array[Vector3] = []
			for index: int in range(24):
				var angle: float = TAU*float(index)/24.0
				world.append(Vector3(center.x+cos(angle)*radius.x,0,center.y+sin(angle)*radius.y))
			if not _polygon(plan,projection,world,Color(162.0/255.0,110.0/255.0,64.0/255.0,radius.z)):
				return false
	return true

func draw_floor(canvas: Variant, snapshot: Variant, scene: Variant, current_key: Variant, projection: Variant, viewport: Variant, furnace_visual: Variant) -> bool:
	if not _context(canvas,snapshot,scene,current_key,projection,viewport):
		return false
	var plan: Array[Dictionary] = []
	var camera: Variant = projection
	for row: int in range(4):
		for column: int in range(6):
			var delta: float = float((column+2*row)%3-1)*0.006
			var color: Color = Color(52.0/255.0+delta,58.0/255.0+delta,60.0/255.0+delta,1.0)
			if not _quad(plan,camera,column*4,row*4,(column+1)*4,(row+1)*4,color):
				return false
	var seam: Color = Color(0.04,0.055,0.06,0.075)
	var width: float = 0.65*float(camera.get('ppm'))/48.0
	for x: int in [4,8,12,16,20]:
		if not _line(plan,camera,Vector3(x,0,0),Vector3(x,0,16),seam,width):
			return false
	for z: int in [4,8,12]:
		if not _line(plan,camera,Vector3(0,0,z),Vector3(24,0,z),seam,width):
			return false
	if not _bounce(plan,camera,viewport,furnace_visual) or not _plan_valid(plan,64):
		return false
	_emit(canvas,plan)
	return true

func draw_wall(canvas: Variant, snapshot: Variant, scene: Variant, current_key: Variant, projection: Variant, wall: Variant, viewport: Variant) -> bool:
	if not _context(canvas,snapshot,scene,current_key,projection,viewport) or not wall is Dictionary:
		return false
	var id_value: Variant = wall.get('id')
	if typeof(id_value) != TYPE_STRING or not WALLS.has(id_value):
		return false
	var definition: Array = WALLS[id_value]
	if not _numbers(wall.get('polygon'),definition[0]):
		return false
	var canonical_member: bool = false
	var collision_rows: Array = scene.get('collision')
	for row_value: Variant in collision_rows:
		if not row_value is Dictionary:
			return false
		var row: Dictionary = row_value
		if row.get('id') == id_value:
			canonical_member = row == wall
	if not canonical_member:
		return false
	var plan: Array[Dictionary] = []
	var camera: Variant = projection
	var a: Vector2 = Vector2(definition[1][0],definition[1][1])
	var b: Vector2 = Vector2(definition[2][0],definition[2][1])
	var outward: Vector2 = Vector2(definition[3][0],definition[3][1])*0.05
	var height: float = float(definition[4])
	var ground_end: Vector2 = b
	if id_value == 'cw_regulator_core_wall_east_south':
		# The adjacent south cap hides these lower fragments under this projection.
		# Keep the top endpoint/cap; clip only the contact and side's ground endpoint.
		var south: Array = WALLS['cw_regulator_core_wall_south']
		ground_end.y = float(south[1][1])-float(south[4])
	var contact: Array[Vector3] = [Vector3(a.x,0,a.y),Vector3(ground_end.x,0,ground_end.y),Vector3(ground_end.x+outward.x,0,ground_end.y+outward.y),Vector3(a.x+outward.x,0,a.y+outward.y)]
	var side: Array[Vector3] = [Vector3(a.x,0,a.y),Vector3(ground_end.x,0,ground_end.y),Vector3(b.x,height,b.y),Vector3(a.x,height,a.y)]
	var cap: Array[Vector3] = []
	var polygon: Array = definition[0]
	for point: Array in polygon:
		cap.append(Vector3(float(point[0]),height,float(point[1])))
	if not _polygon(plan,camera,contact,Color(0.02,0.03,0.035,0.13)) or not _polygon(plan,camera,side,Color('#252c2f')) or not _polygon(plan,camera,cap,Color('#4c5658')) or not _plan_valid(plan,3):
		return false
	_emit(canvas,plan)
	return true
