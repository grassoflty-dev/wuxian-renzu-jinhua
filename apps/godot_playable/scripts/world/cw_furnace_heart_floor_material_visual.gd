extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const SCENE_SHA256 = 'f16f3941767aed71023cd8b9ce75642345a103eabf5c77682e358c96c2e219d5'
const ROOM_WIDTH: float = 24.0
const ROOM_DEPTH: float = 16.0
const CELL_SIZE: float = 4.0
const CELL_COLUMNS: int = 6
const CELL_ROWS: int = 4
const JOINT_WIDTH: float = 0.02
const VERTICAL_JOINTS: Array[float] = [4.0,8.0,12.0,16.0,20.0]
const HORIZONTAL_JOINTS: Array[float] = [4.0,8.0,12.0]
const BASE_COLOR: Color = Color(52.0 / 255.0,58.0 / 255.0,60.0 / 255.0,1.0)
const JOINT_COLOR: Color = Color(0.04,0.055,0.06,0.075)

func _finite_v2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _finite_v3(value: Vector3) -> bool:
	return is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _finite_color(value: Color) -> bool:
	return is_finite(value.r) and is_finite(value.g) and is_finite(value.b) and is_finite(value.a)

func _valid_rect(value: Rect2) -> bool:
	return _finite_v2(value.position) and _finite_v2(value.size) and _finite_v2(value.end) and value.size.x > 0.0 and value.size.y > 0.0

func _layout_matches(scene: Dictionary) -> bool:
	if scene.get('worldId') != 'clockworks' or scene.get('sceneId') != 'cw_furnace_heart' or not P.counter(scene.get('schemaVersion'),1,1):
		return false
	var bounds_value: Variant = scene.get('boundsM')
	if not bounds_value is Dictionary or bounds_value.size() != 4:
		return false
	for key: String in ['x','z','width','depth']:
		if not P.number(bounds_value.get(key)):
			return false
	if float(bounds_value.x) != 0.0 or float(bounds_value.z) != 0.0 or float(bounds_value.width) != ROOM_WIDTH or float(bounds_value.depth) != ROOM_DEPTH:
		return false
	var presentation_value: Variant = scene.get('presentation')
	if not presentation_value is Dictionary:
		return false
	var layers_value: Variant = presentation_value.get('layers')
	var sprites_value: Variant = presentation_value.get('sprites')
	if not layers_value is Array or not sprites_value is Array:
		return false
	var floor_layer_count: int = 0
	for layer_value: Variant in layers_value:
		if not layer_value is Dictionary:
			return false
		var layer: Dictionary = layer_value
		if layer.get('id') == 'visual.floor':
			floor_layer_count += 1
			if not P.counter(layer.get('zGroup'),0,0):
				return false
	if floor_layer_count != 1:
		return false
	var found_steel: int = 0
	var found_factory: int = 0
	for sprite_value: Variant in sprites_value:
		if not sprite_value is Dictionary:
			return false
		var sprite: Dictionary = sprite_value
		var sprite_id: Variant = sprite.get('id')
		if sprite_id != 'cw_furnace_heart_steel_floor' and sprite_id != 'cw_furnace_heart_factory_tiles':
			continue
		if sprite.get('layer') != 'visual.floor' or not sprite.get('position') is Array or sprite.position.size() != 3:
			return false
		for index: int in range(3):
			if not P.number(sprite.position[index]):
				return false
		if float(sprite.position[0]) != 12.0 or float(sprite.position[1]) != 0.0 or float(sprite.position[2]) != 8.0:
			return false
		if sprite_id == 'cw_furnace_heart_steel_floor':
			if sprite.get('assetId') != 'runtime2d.world.clockworks.floor_steel.v1':
				return false
			found_steel += 1
		else:
			if sprite.get('assetId') != 'runtime2d.world.clockworks.factory_tiles.v1':
				return false
			found_factory += 1
	return found_steel == 1 and found_factory == 1

func _projection_valid(projection_value: Variant, viewport: Rect2) -> bool:
	if not projection_value is RefCounted or not is_instance_valid(projection_value) or projection_value.get_script() != ProjectionScript:
		return false
	var initialized_value: Variant = projection_value.get('initialized')
	var origin_value: Variant = projection_value.get('origin')
	var center_value: Variant = projection_value.get('screen_center')
	var camera_viewport_value: Variant = projection_value.get('viewport')
	var ppm_value: Variant = projection_value.get('ppm')
	if typeof(initialized_value) != TYPE_BOOL or not initialized_value or not origin_value is Vector3 or not center_value is Vector2 or not camera_viewport_value is Vector2:
		return false
	if not P.number(ppm_value,0.0,256.0) or float(ppm_value) <= 0.0:
		return false
	return _finite_v3(origin_value) and _finite_v2(center_value) and _finite_v2(camera_viewport_value) and camera_viewport_value == viewport.size

func _canvas_matches(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, current_key: String, scene_hash: String, projection: RefCounted, viewport: Rect2) -> bool:
	if not is_instance_valid(canvas) or not canvas.is_inside_tree() or not canvas.is_visible_in_tree() or canvas.is_queued_for_deletion():
		return false
	if canvas.get('snapshot') != snapshot or canvas.get('scene') != scene or canvas.get('current_key') != current_key or canvas.get('scene_hash') != scene_hash or canvas.get('camera') != projection:
		return false
	var stored_viewport: Variant = canvas.get('viewport_rect')
	if not stored_viewport is Rect2 or stored_viewport != viewport or canvas.get_viewport_rect() != viewport:
		return false
	return _valid_rect(viewport) and _projection_valid(projection,viewport)

func _context_matches(snapshot: Dictionary, scene: Dictionary, current_key: Variant, scene_hash: Variant) -> bool:
	if not P.snapshot(snapshot) or snapshot.get('entryToken') != null or snapshot.get('worldId') != 'clockworks' or snapshot.get('sceneId') != 'cw_furnace_heart':
		return false
	var epoch: Variant = snapshot.get('worldEpoch')
	if not P.counter(epoch,1) or typeof(current_key) != TYPE_STRING or current_key != 'clockworks:cw_furnace_heart:'+str(epoch) or scene_hash != SCENE_SHA256:
		return false
	return _layout_matches(scene)

func _color_valid(color: Variant) -> bool:
	return color is Color and _finite_color(color) and color.a >= 0.0 and color.a <= 1.0

func _quad_valid(points: PackedVector2Array) -> bool:
	if points.size() != 4:
		return false
	var minimum: Vector2 = Vector2(INF,INF)
	var maximum: Vector2 = Vector2(-INF,-INF)
	for point: Vector2 in points:
		if not _finite_v2(point):
			return false
		minimum = minimum.min(point)
		maximum = maximum.max(point)
	if not _finite_v2(minimum) or not _finite_v2(maximum) or maximum.x <= minimum.x or maximum.y <= minimum.y:
		return false
	var area2: float = 0.0
	for index: int in range(1,3):
		var term: float = (points[index]-points[0]).cross(points[index+1]-points[0])
		if not is_finite(term):
			return false
		area2 += term
	if not is_finite(area2) or area2 == 0.0:
		return false
	for index: int in range(4):
		var edge: Vector2 = points[(index+1)%4]-points[index]
		var next_edge: Vector2 = points[(index+2)%4]-points[(index+1)%4]
		var turn: float = edge.cross(next_edge)
		if not _finite_v2(edge) or edge == Vector2.ZERO or not is_finite(turn) or turn == 0.0 or (turn > 0.0) != (area2 > 0.0):
			return false
	return true

func _world_rect_valid(rect: Rect2) -> bool:
	return _valid_rect(rect) and rect.position.x >= 0.0 and rect.position.y >= 0.0 and rect.end.x <= ROOM_WIDTH and rect.end.y <= ROOM_DEPTH

func _append_rect(plan: Array[Dictionary], x0: float, z0: float, x1: float, z1: float, color: Color, kind: String, projection: RefCounted) -> bool:
	var world_rect: Rect2 = Rect2(Vector2(x0,z0),Vector2(x1-x0,z1-z0))
	if not _world_rect_valid(world_rect) or not _color_valid(color):
		return false
	var points: PackedVector2Array = PackedVector2Array()
	for world_point: Vector3 in [Vector3(x0,0.0,z0),Vector3(x1,0.0,z0),Vector3(x1,0.0,z1),Vector3(x0,0.0,z1)]:
		var projected: Vector2 = projection.project(world_point)
		if not _finite_v2(projected):
			return false
		points.append(projected)
	if not _quad_valid(points):
		return false
	plan.append({'world_rect':world_rect,'points':points,'color':color,'kind':kind})
	return true

func _plan_valid(plan: Array[Dictionary]) -> bool:
	if plan.size() != 47:
		return false
	var cell_count: int = 0
	var joint_count: int = 0
	var joint_seen: bool = false
	var vertex_count: int = 0
	for command: Dictionary in plan:
		var world_rect: Variant = command.get('world_rect')
		var points: Variant = command.get('points')
		var color: Variant = command.get('color')
		var kind: Variant = command.get('kind')
		if not world_rect is Rect2 or not points is PackedVector2Array or not kind is String or not _world_rect_valid(world_rect) or not _quad_valid(points) or not _color_valid(color):
			return false
		vertex_count += points.size()
		if kind == 'cell':
			if joint_seen:
				return false
			cell_count += 1
		elif kind == 'joint':
			joint_seen = true
			joint_count += 1
		else:
			return false
	return cell_count == 24 and joint_count == 23 and vertex_count == 188

func _build_floor_plan(projection: RefCounted) -> Array[Dictionary]:
	var plan: Array[Dictionary] = []
	for row: int in range(CELL_ROWS):
		for column: int in range(CELL_COLUMNS):
			var variation_index: int = posmod(column+2*row,3)-1
			var variation: float = float(variation_index)*0.006
			var cell_color: Color = Color(BASE_COLOR.r+variation,BASE_COLOR.g+variation,BASE_COLOR.b+variation,1.0)
			var x0: float = float(column)*CELL_SIZE
			var z0: float = float(row)*CELL_SIZE
			if not _append_rect(plan,x0,z0,x0+CELL_SIZE,z0+CELL_SIZE,cell_color,'cell',projection):
				return []
	for joint_x: float in VERTICAL_JOINTS:
		if not _append_rect(plan,joint_x-JOINT_WIDTH*0.5,0.0,joint_x+JOINT_WIDTH*0.5,ROOM_DEPTH,JOINT_COLOR,'joint',projection):
			return []
	for joint_z: float in HORIZONTAL_JOINTS:
		for column: int in range(CELL_COLUMNS):
			var x0: float = float(column)*CELL_SIZE
			var x1: float = x0+CELL_SIZE
			if x0 > 0.0:
				x0 += JOINT_WIDTH*0.5
			if x1 < ROOM_WIDTH:
				x1 -= JOINT_WIDTH*0.5
			if not _append_rect(plan,x0,joint_z-JOINT_WIDTH*0.5,x1,joint_z+JOINT_WIDTH*0.5,JOINT_COLOR,'joint',projection):
				return []
	return plan if _plan_valid(plan) else []

func _floor_bounds(plan: Array[Dictionary]) -> Rect2:
	var minimum: Vector2 = Vector2(INF,INF)
	var maximum: Vector2 = Vector2(-INF,-INF)
	for command: Dictionary in plan:
		var points: PackedVector2Array = command.points
		for point: Vector2 in points:
			minimum = minimum.min(point)
			maximum = maximum.max(point)
	return Rect2(minimum,maximum-minimum)

func draw_floor(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, scene_hash_value: Variant, projection_value: Variant, viewport_value: Variant) -> bool:
	if not canvas_value is CanvasItem or not is_instance_valid(canvas_value) or not snapshot_value is Dictionary or not scene_value is Dictionary or typeof(current_key_value) != TYPE_STRING or typeof(scene_hash_value) != TYPE_STRING or not viewport_value is Rect2 or not projection_value is RefCounted:
		return false
	var canvas: CanvasItem = canvas_value
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	var current_key: String = current_key_value
	var scene_hash: String = scene_hash_value
	var projection: RefCounted = projection_value
	var viewport: Rect2 = viewport_value
	if not _context_matches(snapshot,scene,current_key,scene_hash) or not _canvas_matches(canvas,snapshot,scene,current_key,scene_hash,projection,viewport):
		return false
	var plan: Array[Dictionary] = _build_floor_plan(projection)
	if not _plan_valid(plan):
		return false
	var floor_bounds: Rect2 = _floor_bounds(plan)
	if not _valid_rect(floor_bounds) or not floor_bounds.intersects(viewport):
		return false
	if not _context_matches(snapshot,scene,current_key,scene_hash) or not _canvas_matches(canvas,snapshot,scene,current_key,scene_hash,projection,viewport):
		return false
	for command: Dictionary in plan:
		canvas.draw_colored_polygon(command.points,command.color)
	return true
