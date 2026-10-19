extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const GhGateBWestCableVisual = preload('res://scripts/world/gh_gate_b_west_cable_visual.gd')

const FLOOR_BASE: Color = Color(48.0 / 255.0, 59.0 / 255.0, 61.0 / 255.0, 1.0)
const FLOOR_JOINT: Color = Color(0.06, 0.065, 0.065, 0.10)
const FLOOR_CONTACT: Color = Color(0.025, 0.031, 0.034, 0.12)
const WALL_FACE: Color = Color('#252e31')
const WALL_CAP: Color = Color('#4b5658')
const WALL_OUTLINE: Color = Color(0.30, 0.35, 0.36, 0.35)
const GATE_SHADOW: Color = Color(0.012, 0.020, 0.023, 0.15)
const GATE_REFLECTION: Color = Color(0.40, 0.57, 0.60, 0.035)
const CRATE_SHADOW: Color = Color(0.012, 0.019, 0.021, 0.14)
const EPSILON: float = 0.000001

const WALLS: Dictionary = {
	'gh_gate_b_wall_north': {
		'polygon': [[0,0],[24,0],[24,0.5],[0,0.5]], 'inner': [[0,0.5],[24,0.5]], 'out': [0,-1], 'height': 0.7
	},
	'gh_gate_b_wall_south': {
		'polygon': [[0,15.5],[24,15.5],[24,16],[0,16]], 'inner': [[0,15.5],[24,15.5]], 'out': [0,1], 'height': 0.2
	},
	'gh_gate_b_wall_west_north': {
		'polygon': [[0,0.5],[0.5,0.5],[0.5,6.5],[0,6.5]], 'inner': [[0.5,0.5],[0.5,6.5]], 'out': [-1,0], 'height': 0.7
	},
	'gh_gate_b_wall_west_south': {
		'polygon': [[0,9.5],[0.5,9.5],[0.5,15.5],[0,15.5]], 'inner': [[0.5,9.5],[0.5,15.5]], 'out': [-1,0], 'height': 0.7
	},
	'gh_gate_b_wall_east_north': {
		'polygon': [[23.5,0.5],[24,0.5],[24,6.5],[23.5,6.5]], 'inner': [[23.5,0.5],[23.5,6.5]], 'out': [1,0], 'height': 0.2
	},
	'gh_gate_b_wall_east_south': {
		'polygon': [[23.5,9.5],[24,9.5],[24,15.5],[23.5,15.5]], 'inner': [[23.5,9.5],[23.5,15.5]], 'out': [1,0], 'height': 0.2
	}
}

const SPRITES: Dictionary = {
	'approved_floor': ['runtime2d.grey_hive.floor_tiles.v1','visual.floor',[12,0,8]],
	'gate_b_corridor_wall': ['runtime2d.grey_hive.wall_tiles.v1','visual.props_back',[12,0,1]],
	'gate_b_compression_pipes': ['runtime2d.grey_hive.pipes_cables.v1','visual.props_back',[15,0,3]],
	'gate_b_supply_crates': ['runtime2d.prop.supply_crates.v1','visual.props_back',[5,0,11.5]],
	'gate_b_approved_clutter': ['runtime2d.grey_hive.clutter.v1','visual.props_back',[19,0,11.5]],
	'gate_b_visual_approved': ['runtime2d.prop.gate_b.v1','visual.props_dynamic',[12,0,8]],
	'gate_b_security_01_visual': ['runtime2d.enemy.infected_security.v1','visual.props_dynamic',[7,0,4.5]],
	'gate_b_security_02_visual': ['runtime2d.enemy.infected_security.v1','visual.props_dynamic',[17,0,11.5]]
}

func _finite(value: float) -> bool:
	return is_finite(value)

func _finite_vec2(value: Vector2) -> bool:
	return _finite(value.x) and _finite(value.y) and absf(value.x) <= 10000000.0 and absf(value.y) <= 10000000.0

func _finite_color(value: Color) -> bool:
	return _finite(value.r) and _finite(value.g) and _finite(value.b) and _finite(value.a) and value.r >= 0.0 and value.r <= 1.0 and value.g >= 0.0 and value.g <= 1.0 and value.b >= 0.0 and value.b <= 1.0 and value.a >= 0.0 and value.a <= 1.0

func _position_matches(value: Variant, expected: Array) -> bool:
	if not value is Array or value.size() != 3:
		return false
	for index in range(3):
		var component: Variant = value[index]
		var wanted: float = float(expected[index])
		if not P.number(component, wanted - EPSILON, wanted + EPSILON):
			return false
	return true

func _polygon_matches(value: Variant, expected: Array) -> bool:
	if not value is Array or value.size() != expected.size():
		return false
	for index in range(expected.size()):
		var actual_point: Variant = value[index]
		var expected_point: Array = expected[index]
		if not actual_point is Array or actual_point.size() != 2:
			return false
		for axis in range(2):
			var component: Variant = actual_point[axis]
			var wanted: float = float(expected_point[axis])
			if not P.number(component, wanted - EPSILON, wanted + EPSILON):
				return false
	return true

func _valid_projection(projection: Variant, viewport: Variant) -> bool:
	if projection == null or not projection is RefCounted or not is_instance_valid(projection) or projection.get_script() != ProjectionScript:
		return false
	if not viewport is Rect2:
		return false
	var initialized_value: Variant = projection.get('initialized')
	var origin_value: Variant = projection.get('origin')
	var center_value: Variant = projection.get('screen_center')
	var camera_viewport_value: Variant = projection.get('viewport')
	var ppm_value: Variant = projection.get('ppm')
	if typeof(initialized_value) != TYPE_BOOL or not initialized_value:
		return false
	if typeof(origin_value) != TYPE_VECTOR3 or typeof(center_value) != TYPE_VECTOR2 or typeof(camera_viewport_value) != TYPE_VECTOR2:
		return false
	var origin: Vector3 = origin_value
	var center: Vector2 = center_value
	var camera_viewport: Vector2 = camera_viewport_value
	if not _finite(origin.x) or not _finite(origin.y) or not _finite(origin.z) or not _finite_vec2(center) or not _finite_vec2(camera_viewport):
		return false
	if not P.number(ppm_value, 0.000001, 256.0):
		return false
	var view: Rect2 = viewport
	if not _finite_vec2(view.position) or not _finite_vec2(view.size) or view.size.x <= 0.0 or view.size.y <= 0.0 or not _finite_vec2(view.end):
		return false
	if absf(camera_viewport.x - view.size.x) > EPSILON or absf(camera_viewport.y - view.size.y) > EPSILON:
		return false
	return true

func _valid_scene_rows(scene: Dictionary) -> bool:
	if scene.get('worldId') != 'grey_hive' or scene.get('sceneId') != 'gh_gate_b' or not P.counter(scene.get('schemaVersion'), 1, 1):
		return false
	var bounds_value: Variant = scene.get('boundsM')
	if not bounds_value is Dictionary:
		return false
	var bounds: Dictionary = bounds_value
	if not P.number(bounds.get('x'), 0.0, 0.0) or not P.number(bounds.get('z'), 0.0, 0.0) or not P.number(bounds.get('width'), 24.0, 24.0) or not P.number(bounds.get('depth'), 16.0, 16.0):
		return false
	var collision_value: Variant = scene.get('collision')
	if not collision_value is Array or collision_value.size() != 7:
		return false
	var seen_collision: Dictionary = {}
	for item in collision_value:
		if not item is Dictionary:
			return false
		var row: Dictionary = item
		var id_value: Variant = row.get('id')
		if typeof(id_value) != TYPE_STRING or seen_collision.has(id_value):
			return false
		seen_collision[id_value] = true
		if id_value == 'gh_gate_b_gate_barrier':
			if not _polygon_matches(row.get('polygon'), [[11.5,0.5],[12.5,0.5],[12.5,15.5],[11.5,15.5]]) or row.get('requiresEvent') != 'hive_lockdown':
				return false
		elif WALLS.has(id_value):
			var wall_data: Dictionary = WALLS[id_value]
			if not _polygon_matches(row.get('polygon'), wall_data.polygon):
				return false
		else:
			return false
	if seen_collision.size() != 7:
		return false
	var door_rows_value: Variant = scene.get('doors')
	if not door_rows_value is Array or door_rows_value.size() != 1 or not door_rows_value[0] is Dictionary:
		return false
	var scene_door: Dictionary = door_rows_value[0]
	if scene_door.get('id') != 'gh_gate_b' or scene_door.get('assetId') != 'runtime2d.prop.gate_b.v1' or scene_door.get('requiresEvent') != 'hive_lockdown' or not _position_matches(scene_door.get('position'), [12,0,8]):
		return false
	var interaction_value: Variant = scene.get('interactions')
	if not interaction_value is Array or interaction_value.size() != 1 or not interaction_value[0] is Dictionary:
		return false
	var interaction: Dictionary = interaction_value[0]
	if interaction.get('id') != 'gh_gate_b_panel' or interaction.get('kind') != 'gate_b_panel' or not interaction.has('event') or interaction.get('event') != null or not _position_matches(interaction.get('position'), [12,0,8]):
		return false
	var presentation_value: Variant = scene.get('presentation')
	if not presentation_value is Dictionary:
		return false
	var sprites_value: Variant = presentation_value.get('sprites')
	if not sprites_value is Array or sprites_value.size() != SPRITES.size():
		return false
	var seen_sprites: Dictionary = {}
	for sprite_item in sprites_value:
		if not sprite_item is Dictionary:
			return false
		var sprite: Dictionary = sprite_item
		var sprite_id_value: Variant = sprite.get('id')
		if typeof(sprite_id_value) != TYPE_STRING or seen_sprites.has(sprite_id_value) or not SPRITES.has(sprite_id_value):
			return false
		seen_sprites[sprite_id_value] = true
		var expected: Array = SPRITES[sprite_id_value]
		if sprite.get('assetId') != expected[0] or sprite.get('layer') != expected[1] or not _position_matches(sprite.get('position'), expected[2]):
			return false
	return seen_sprites.size() == SPRITES.size()

func _valid_snapshot_door(snapshot: Dictionary) -> Dictionary:
	var rows_value: Variant = snapshot.get('doors')
	if not rows_value is Array or rows_value.size() != 1 or not rows_value[0] is Dictionary:
		return {}
	var row: Dictionary = rows_value[0]
	var door_id: Variant = row.get('doorId')
	var open_value: Variant = row.get('open')
	var locked_value: Variant = row.get('locked')
	var transform_value: Variant = row.get('transform')
	if door_id != 'gh_gate_b' or typeof(open_value) != TYPE_BOOL or typeof(locked_value) != TYPE_BOOL or not P.transform(transform_value):
		return {}
	var transform: Dictionary = transform_value
	var position_value: Variant = transform.get('positionM')
	if not position_value is Dictionary:
		return {}
	var position: Dictionary = position_value
	if not P.number(position.get('xM'), 12.0, 12.0) or not P.number(position.get('yM'), 0.0, 0.0) or not P.number(position.get('zM'), 8.0, 8.0) or not P.number(transform.get('yawRad'), 0.0, 0.0):
		return {}
	return row

func _valid_context(canvas: Variant, snapshot_value: Variant, scene_value: Variant, current_key: Variant, projection: Variant, viewport: Variant) -> bool:
	if canvas == null or not canvas is CanvasItem or not is_instance_valid(canvas) or not canvas.is_inside_tree() or canvas.is_queued_for_deletion():
		return false
	if not snapshot_value is Dictionary or not P.snapshot(snapshot_value):
		return false
	if not scene_value is Dictionary or typeof(current_key) != TYPE_STRING:
		return false
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	if snapshot.get('worldId') != 'grey_hive' or snapshot.get('sceneId') != 'gh_gate_b' or scene.get('worldId') != 'grey_hive' or scene.get('sceneId') != 'gh_gate_b' or not P.counter(snapshot.get('worldEpoch'), 1):
		return false
	var expected_key: String = 'grey_hive:gh_gate_b:' + str(snapshot.get('worldEpoch'))
	if current_key != expected_key or not _valid_scene_rows(scene) or not _valid_projection(projection, viewport):
		return false
	return not _valid_snapshot_door(snapshot).is_empty()

func _project(projection: RefCounted, x: float, y: float, z: float) -> Vector2:
	var point: Vector2 = projection.project(Vector3(x, y, z))
	return point

func _smoothstep01(value: float) -> float:
	var t: float = clampf(value, 0.0, 1.0)
	return t * t * (3.0 - 2.0 * t)

func _floor_color(x: float, z: float, delta: float) -> Color:
	var edge_distance: float = minf(minf(x, 24.0 - x), minf(z, 16.0 - z))
	var edge: float = 1.0 - _smoothstep01(edge_distance / 2.0)
	var bay: float = 1.0 - _smoothstep01(absf(z - 8.0) / 6.0)
	var lift: float = delta - 0.016 * edge + 0.012 * bay
	return Color(FLOOR_BASE.r + lift, FLOOR_BASE.g + lift, FLOOR_BASE.b + lift, 1.0)

func _append_gradient(commands: Array[Dictionary], projection: RefCounted, x0: float, z0: float, x1: float, z1: float, delta: float) -> bool:
	var world_points: Array[Vector2] = [Vector2(x0,z0), Vector2(x1,z0), Vector2(x1,z1), Vector2(x0,z1)]
	var points: PackedVector2Array = PackedVector2Array()
	var colors: PackedColorArray = PackedColorArray()
	for world_point: Vector2 in world_points:
		var screen_point: Vector2 = _project(projection, world_point.x, 0.0, world_point.y)
		var color: Color = _floor_color(world_point.x, world_point.y, delta)
		if not _finite_vec2(screen_point) or not _finite_color(color):
			return false
		points.append(screen_point)
		colors.append(color)
	commands.append({'kind':'gradient','points':points,'colors':colors})
	return true

func _append_line(commands: Array[Dictionary], projection: RefCounted, a: Vector2, b: Vector2, color: Color, width: float) -> bool:
	var start: Vector2 = _project(projection, a.x, 0.0, a.y)
	var finish: Vector2 = _project(projection, b.x, 0.0, b.y)
	if not _finite_vec2(start) or not _finite_vec2(finish) or not _finite_color(color) or not _finite(width) or width <= 0.0 or width > 8.0:
		return false
	commands.append({'kind':'line','a':start,'b':finish,'color':color,'width':width})
	return true

func _append_ellipse(commands: Array[Dictionary], projection: RefCounted, center: Vector2, radius_x: float, radius_z: float, color: Color) -> bool:
	if not _finite_vec2(center) or not _finite(radius_x) or not _finite(radius_z) or radius_x <= 0.0 or radius_z <= 0.0 or radius_x > 2.0 or radius_z > 1.0 or not _finite_color(color):
		return false
	var points: PackedVector2Array = PackedVector2Array()
	for index in range(32):
		var angle: float = float(index) * (PI * 2.0 / 32.0)
		var x: float = center.x + cos(angle) * radius_x
		var z: float = center.y + sin(angle) * radius_z
		var screen: Vector2 = _project(projection, x, 0.0, z)
		if not _finite_vec2(screen):
			return false
		points.append(screen)
	commands.append({'kind':'solid','points':points,'color':color})
	return true

func _append_wall_commands(commands: Array[Dictionary], projection: RefCounted, wall_data: Dictionary) -> bool:
	var polygon: Array = wall_data.polygon
	var inner: Array = wall_data.inner
	var outward: Array = wall_data.out
	var height_value: Variant = wall_data.get('height')
	if not P.number(height_value, 0.2, 0.7):
		return false
	var height: float = float(height_value)
	var inner_a: Array = inner[0]
	var inner_b: Array = inner[1]
	var ax: float = float(inner_a[0])
	var az: float = float(inner_a[1])
	var bx: float = float(inner_b[0])
	var bz: float = float(inner_b[1])
	var ox: float = float(outward[0])
	var oz: float = float(outward[1])
	var contact_world: Array[Vector2] = [Vector2(ax,az),Vector2(bx,bz),Vector2(bx+ox*0.04,bz+oz*0.04),Vector2(ax+ox*0.04,az+oz*0.04)]
	var contact_points: PackedVector2Array = PackedVector2Array()
	for point in contact_world:
		var screen: Vector2 = _project(projection,point.x,0.0,point.y)
		if not _finite_vec2(screen):
			return false
		contact_points.append(screen)
	commands.append({'kind':'solid','points':contact_points,'color':FLOOR_CONTACT})
	var side_points: PackedVector2Array = PackedVector2Array()
	var side_world: Array[Vector3] = [Vector3(ax,0.0,az),Vector3(bx,0.0,bz),Vector3(bx,height,bz),Vector3(ax,height,az)]
	for point in side_world:
		var screen: Vector2 = projection.project(point)
		if not _finite_vec2(screen):
			return false
		side_points.append(screen)
	commands.append({'kind':'solid','points':side_points,'color':WALL_FACE})
	var cap_points: PackedVector2Array = PackedVector2Array()
	for point_value in polygon:
		var point: Array = point_value
		var screen: Vector2 = _project(projection,float(point[0]),height,float(point[1]))
		if not _finite_vec2(screen):
			return false
		cap_points.append(screen)
	commands.append({'kind':'solid','points':cap_points,'color':WALL_CAP})
	var outline_points: PackedVector2Array = cap_points.duplicate()
	outline_points.append(cap_points[0])
	var outline_width: float = float(projection.get('ppm')) * 0.8 / 48.0
	if not _finite_color(WALL_OUTLINE) or not _finite(outline_width) or outline_width <= 0.0 or outline_width > 8.0:
		return false
	commands.append({'kind':'polyline','points':outline_points,'color':WALL_OUTLINE,'width':outline_width})
	return true

func _draw_commands(canvas: CanvasItem, commands: Array[Dictionary]) -> void:
	for command in commands:
		var kind: String = str(command.get('kind'))
		if kind == 'gradient':
			var gradient_points: PackedVector2Array = command.get('points')
			var gradient_colors: PackedColorArray = command.get('colors')
			canvas.draw_polygon(gradient_points, gradient_colors)
		elif kind == 'solid':
			var solid_points: PackedVector2Array = command.get('points')
			var solid_color: Color = command.get('color')
			canvas.draw_colored_polygon(solid_points, solid_color)
		elif kind == 'line':
			var line_start: Vector2 = command.get('a')
			var line_end: Vector2 = command.get('b')
			var line_color: Color = command.get('color')
			var line_width: float = float(command.get('width'))
			canvas.draw_line(line_start, line_end, line_color, line_width, true)
		elif kind == 'polyline':
			var outline_points: PackedVector2Array = command.get('points')
			var outline_color: Color = command.get('color')
			var outline_width: float = float(command.get('width'))
			canvas.draw_polyline(outline_points, outline_color, outline_width, true)

func draw_floor(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, current_key: String, projection: Variant, viewport: Variant, state_props: Variant, crate_visual: Variant) -> bool:
	if not _valid_context(canvas,snapshot,scene,current_key,projection,viewport):
		return false
	var commands: Array[Dictionary] = []
	var camera: RefCounted = projection
	for row in range(4):
		for column in range(6):
			var delta: float = float((column + 2 * row) % 3 - 1) * 0.004
			if not _append_gradient(commands,camera,float(column*4),float(row*4),float((column+1)*4),float((row+1)*4),delta):
				return false
	var ppm: float = float(camera.get('ppm'))
	var joint_width: float = 0.6 * ppm / 48.0
	for x_index in range(1,6):
		var x: float = float(x_index * 4)
		if not _append_line(commands,camera,Vector2(x,0.0),Vector2(x,16.0),FLOOR_JOINT,joint_width):
			return false
	for z_index in range(1,4):
		var z: float = float(z_index * 4)
		if not _append_line(commands,camera,Vector2(0.0,z),Vector2(24.0,z),FLOOR_JOINT,joint_width):
			return false
	var door: Dictionary = _valid_snapshot_door(snapshot)
	if not door.is_empty() and door.get('open') == false and state_props is Object and is_instance_valid(state_props):
		var textures_value: Variant = state_props.get('textures')
		if textures_value is Dictionary:
			var textures: Dictionary = textures_value
			var gate_texture: Variant = textures.get('gate_b')
			if gate_texture is Texture2D:
				if not _append_ellipse(commands,camera,Vector2(12.0,8.0),1.4,0.6,GATE_REFLECTION):
					return false
				if not _append_ellipse(commands,camera,Vector2(12.0,8.0),1.0,0.42,Color(0.40,0.57,0.60,0.045)):
					return false
				if not _append_ellipse(commands,camera,Vector2(12.0,8.0),0.65,0.25,Color(0.40,0.57,0.60,0.055)):
					return false
				if not _append_ellipse(commands,camera,Vector2(12.0,8.0),0.9,0.28,GATE_SHADOW):
					return false
	if crate_visual is Object and is_instance_valid(crate_visual):
		var crate_texture: Variant = crate_visual.get('texture')
		if crate_texture is Texture2D:
			if not _append_ellipse(commands,camera,Vector2(5.0,11.5),0.45,0.18,CRATE_SHADOW):
				return false
	if commands.size() > 40:
		return false
	_draw_commands(canvas,commands)
	return true

func draw_wall(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, current_key: String, projection: Variant, wall: Variant, viewport: Variant, cable_visual: GhGateBWestCableVisual = null, cable_receipt: Variant = null) -> bool:
	if not _valid_context(canvas,snapshot,scene,current_key,projection,viewport) or not wall is Dictionary:
		return false
	var id_value: Variant = wall.get('id')
	if typeof(id_value) != TYPE_STRING or not WALLS.has(id_value):
		return false
	var wall_data: Dictionary = WALLS[id_value]
	if not _polygon_matches(wall.get('polygon'),wall_data.polygon):
		return false
	var commands: Array[Dictionary] = []
	var camera: RefCounted = projection
	if not _append_wall_commands(commands,camera,wall_data) or commands.size() != 4:
		return false
	_draw_commands(canvas,commands)
	if id_value == 'gh_gate_b_wall_west_north' and is_instance_valid(cable_visual) and cable_receipt is Dictionary and cable_receipt.size() == 1 and typeof(cable_receipt.get('pipe_submitted')) == TYPE_BOOL and cable_receipt['pipe_submitted'] == false:
		if cable_visual.draw_wall_cable(canvas,snapshot,scene,current_key,projection,wall,viewport):
			cable_receipt['pipe_submitted'] = true
	return true
