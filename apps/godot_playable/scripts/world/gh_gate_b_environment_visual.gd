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

const EXPOSED_FACES: Dictionary = {
	'gh_gate_b_wall_south': [[0.0,16.0],[24.0,16.0]],
	'gh_gate_b_wall_east_north': [[24.0,0.5],[24.0,6.5]],
	'gh_gate_b_wall_east_south': [[24.0,9.5],[24.0,15.5]]
}

# GB_SHORT_DATA_BEGIN
const SHORT_UNION_FACES: Array = [
	{"owner":"gh_gate_b_wall_north","a":[24,0],"b":[24,0.5],"low_owner":""},
	{"owner":"gh_gate_b_wall_south","a":[24,15.5],"b":[24,16],"low_owner":""},
	{"owner":"gh_gate_b_wall_west_north","a":[0,6.5],"b":[0.5,6.5],"low_owner":""},
	{"owner":"gh_gate_b_wall_east_north","a":[23.5,6.5],"b":[24,6.5],"low_owner":""},
	{"owner":"gh_gate_b_wall_west_south","a":[0,15.5],"b":[0.5,15.5],"low_owner":"gh_gate_b_wall_south"}
]
const SHORT_SCENE_BYTES: int = 6855
const SHORT_SCENE_SHA256: String = 'a82c10a6fa50129c48267e255d2ca86f4ebaca43e0e0da81b639ee88228312c1'
var _short_scene_attempted: bool = false
var _short_canonical_scene: Dictionary = {}
# GB_SHORT_DATA_END

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
	if EXPOSED_FACES.has(id_value):
		var bound_snapshot: Variant = canvas.get('snapshot')
		var bound_scene: Variant = canvas.get('scene')
		var bound_key: Variant = canvas.get('current_key')
		var bound_camera: Variant = canvas.get('camera')
		var bound_viewport: Variant = canvas.get('viewport_rect')
		var projection_viewport: Variant = projection.get('viewport')
		var requested_viewport: Rect2 = viewport
		var bindings_match: bool = bound_snapshot is Dictionary and bound_snapshot == snapshot and bound_scene is Dictionary and bound_scene == scene and bound_key == current_key and bound_camera == projection and bound_viewport is Rect2 and bound_viewport == requested_viewport and projection_viewport is Vector2 and projection_viewport == requested_viewport.size
		if bindings_match:
			var legacy_commands: Array[Dictionary] = commands.duplicate()
			var exposed_command: Dictionary = _stage_exposed_face(projection,wall_data,EXPOSED_FACES.get(id_value))
			if exposed_command.is_empty():
				return false
			commands.insert(1,exposed_command)
			if commands.size() != 5 or commands[1] != exposed_command:
				return false
			for index in range(4):
				var placed_index: int = index if index == 0 else index + 1
				if commands[placed_index] != legacy_commands[index]:
					return false
			var expected_kinds: Array[String] = ['solid','solid','solid','solid','polyline']
			for index in range(5):
				if commands[index].get('kind') != expected_kinds[index]:
					return false
	_short_apply(canvas,snapshot,scene,current_key,projection,id_value,viewport,commands,false) # GB_SHORT_NORMAL
	_draw_commands(canvas,commands)
	if id_value == 'gh_gate_b_wall_west_north' and is_instance_valid(cable_visual) and cable_receipt is Dictionary and cable_receipt.size() == 1 and typeof(cable_receipt.get('pipe_submitted')) == TYPE_BOOL and cable_receipt['pipe_submitted'] == false:
		if cable_visual.draw_wall_cable(canvas,snapshot,scene,current_key,projection,wall,viewport):
			cable_receipt['pipe_submitted'] = true
	return true

func _stage_exposed_face(projection: RefCounted, wall_data: Dictionary, segment_value: Variant) -> Dictionary:
	if not segment_value is Array or segment_value.size() != 2:
		return {}
	var height_value: Variant = wall_data.get('height')
	if not P.number(height_value,0.2,0.7) or not _finite_color(WALL_FACE):
		return {}
	var endpoints: Array[Vector2] = []
	for point_value: Variant in segment_value:
		if not point_value is Array or point_value.size() != 2 or not P.number(point_value[0],-1000000000.0,1000000000.0) or not P.number(point_value[1],-1000000000.0,1000000000.0):
			return {}
		endpoints.append(Vector2(float(point_value[0]),float(point_value[1])))
	var height: float = float(height_value)
	var world_points: Array[Vector3] = [Vector3(endpoints[0].x,0.0,endpoints[0].y),Vector3(endpoints[1].x,0.0,endpoints[1].y),Vector3(endpoints[1].x,height,endpoints[1].y),Vector3(endpoints[0].x,height,endpoints[0].y)]
	var points: PackedVector2Array = PackedVector2Array()
	for point: Vector3 in world_points:
		var screen: Vector2 = projection.project(point)
		if not _finite_vec2(screen):
			return {}
		points.append(screen)
	var twice_area: float = 0.0
	for index in range(4):
		var next_index: int = (index + 1) % 4
		twice_area += points[index].x * points[next_index].y - points[next_index].x * points[index].y
	if not _finite(twice_area) or twice_area == 0.0:
		return {}
	return {'kind':'solid','points':points,'color':WALL_FACE}

# GB_SHORT_FUNCTION_BEGIN
func _short_json_equal(value: Variant, expected: Variant, depth: int = 0) -> bool:
	if depth > 64: return false
	var expected_type: int = typeof(expected)
	if expected_type == TYPE_INT or expected_type == TYPE_FLOAT:
		if typeof(value) != TYPE_INT and typeof(value) != TYPE_FLOAT: return false
		return is_finite(float(value)) and float(value) == float(expected)
	if typeof(value) != expected_type: return false
	if expected is Dictionary:
		if value.size() != expected.size(): return false
		for key: Variant in expected:
			if not value.has(key) or not _short_json_equal(value[key],expected[key],depth+1): return false
		return true
	if expected is Array:
		if value.size() != expected.size(): return false
		for index: int in range(expected.size()):
			if not _short_json_equal(value[index],expected[index],depth+1): return false
		return true
	return value == expected
func _short_canonical_matches(scene_value: Variant) -> bool:
	if not scene_value is Dictionary: return false
	if not _short_scene_attempted:
		_short_scene_attempted = true
		var file: FileAccess = FileAccess.open('res://data/scenes/gh_gate_b.json',FileAccess.READ)
		if file == null or file.get_length() != SHORT_SCENE_BYTES:
			if file != null: file.close()
			return false
		var bytes: PackedByteArray = file.get_buffer(SHORT_SCENE_BYTES)
		file.close()
		if bytes.size() != SHORT_SCENE_BYTES: return false
		var hash: HashingContext = HashingContext.new()
		if hash.start(HashingContext.HASH_SHA256) != OK or hash.update(bytes) != OK or hash.finish().hex_encode() != SHORT_SCENE_SHA256: return false
		var source_text: String = bytes.get_string_from_utf8()
		if source_text.to_utf8_buffer() != bytes: return false
		var decoded: Variant = JSON.parse_string(source_text)
		if not decoded is Dictionary: return false
		_short_canonical_scene = decoded
	return not _short_canonical_scene.is_empty() and _short_json_equal(scene_value,_short_canonical_scene)
func _short_bindings(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, projection_value: Variant, viewport_value: Variant) -> bool:
	if not canvas_value is CanvasItem or not is_instance_valid(canvas_value) or not canvas_value.is_inside_tree() or not canvas_value.is_visible_in_tree() or canvas_value.is_queued_for_deletion():
		return false
	if not snapshot_value is Dictionary or snapshot_value.get('entryToken') != null or not viewport_value is Rect2 or not _valid_context(canvas_value,snapshot_value,scene_value,current_key_value,projection_value,viewport_value): return false
	var viewport: Rect2 = viewport_value
	if not _finite_vec2(viewport.position) or not _finite_vec2(viewport.size) or not _finite_vec2(viewport.end) or viewport.position != Vector2.ZERO or viewport.size.x <= 0.0 or viewport.size.y <= 0.0: return false
	var bound_snapshot: Variant = canvas_value.get('snapshot')
	var bound_scene: Variant = canvas_value.get('scene')
	var bound_key: Variant = canvas_value.get('current_key')
	var bound_hash: Variant = canvas_value.get('scene_hash')
	var bound_camera: Variant = canvas_value.get('camera')
	var bound_viewport: Variant = canvas_value.get('viewport_rect')
	var actual_viewport: Rect2 = canvas_value.get_viewport_rect()
	if not bound_snapshot is Dictionary or bound_snapshot != snapshot_value or not bound_scene is Dictionary or bound_scene != scene_value or bound_key != current_key_value or bound_hash != SHORT_SCENE_SHA256 or bound_camera != projection_value: return false
	if not bound_viewport is Rect2 or bound_viewport != viewport or actual_viewport != viewport or not projection_value is RefCounted or projection_value.get('viewport') != viewport.size: return false
	return _short_canonical_matches(scene_value)
func _short_capture(snapshot_value: Dictionary, scene_value: Dictionary, current_key_value: String, projection_value: RefCounted) -> Dictionary:
	return {'snapshot':snapshot_value.duplicate(true),'scene':scene_value.duplicate(true),'key':current_key_value,'hash':SHORT_SCENE_SHA256,'camera':projection_value,'origin':projection_value.get('origin'),'screen_center':projection_value.get('screen_center'),'viewport':projection_value.get('viewport'),'ppm':projection_value.get('ppm'),'initialized':projection_value.get('initialized')}
func _short_recheck(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, projection_value: Variant, viewport_value: Variant, before: Dictionary) -> bool:
	if not _short_bindings(canvas_value,snapshot_value,scene_value,current_key_value,projection_value,viewport_value): return false
	if not snapshot_value is Dictionary or not scene_value is Dictionary or typeof(current_key_value) != TYPE_STRING or not projection_value is RefCounted: return false
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	var current_key: String = current_key_value
	var camera: RefCounted = projection_value
	return _short_json_equal(before,_short_capture(snapshot,scene,current_key,camera))
func _short_quad_valid(points_value: Variant) -> bool:
	if not points_value is PackedVector2Array or points_value.size() != 4: return false
	var points: PackedVector2Array = points_value
	var minimum: Vector2 = Vector2(INF,INF)
	var maximum: Vector2 = Vector2(-INF,-INF)
	for point: Vector2 in points:
		if not _finite_vec2(point): return false
		minimum = minimum.min(point)
		maximum = maximum.max(point)
	if not _finite_vec2(minimum) or not _finite_vec2(maximum) or minimum.x >= maximum.x or minimum.y >= maximum.y: return false
	var origin: Vector2 = points[0]
	var area: float = (points[1]-origin).cross(points[2]-origin) + (points[2]-origin).cross(points[3]-origin)
	if not _finite(area) or area == 0.0: return false
	for index: int in range(4):
		var edge: Vector2 = points[(index+1)%4]-points[index]
		var next_edge: Vector2 = points[(index+2)%4]-points[(index+1)%4]
		var turn: float = edge.cross(next_edge)
		if not _finite_vec2(edge) or edge == Vector2.ZERO or not _finite(turn) or turn == 0.0 or (turn > 0.0) != (area > 0.0): return false
	return true
# SAT returns -1 for invalid arithmetic, 0 for outside/touching and 1 for overlap.
func _short_visible(points: PackedVector2Array, viewport: Rect2) -> int:
	var corners: PackedVector2Array = PackedVector2Array([viewport.position,Vector2(viewport.end.x,viewport.position.y),viewport.end,Vector2(viewport.position.x,viewport.end.y)])
	var axes: Array[Vector2] = [Vector2.RIGHT,Vector2.DOWN]
	for index: int in range(4):
		var edge: Vector2 = points[(index+1)%4]-points[index]
		axes.append(Vector2(-edge.y,edge.x))
	var separated: bool = false
	for axis: Vector2 in axes:
		if not _finite_vec2(axis) or axis == Vector2.ZERO: return -1
		var polygon_min: float = INF
		var polygon_max: float = -INF
		var view_min: float = INF
		var view_max: float = -INF
		for point: Vector2 in points:
			var projected: float = (point-points[0]).dot(axis)
			if not _finite(projected): return -1
			polygon_min = minf(polygon_min,projected)
			polygon_max = maxf(polygon_max,projected)
		for point: Vector2 in corners:
			var projected: float = (point-points[0]).dot(axis)
			if not _finite(projected): return -1
			view_min = minf(view_min,projected)
			view_max = maxf(view_max,projected)
		if not _finite(polygon_min) or not _finite(polygon_max) or not _finite(view_min) or not _finite(view_max) or polygon_min >= polygon_max or view_min >= view_max: return -1
		if polygon_max <= view_min or view_max <= polygon_min:
			separated = true
	return 0 if separated else 1
func _short_prepare(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, projection_value: Variant, viewport_value: Variant) -> Array[Dictionary]:
	var empty: Array[Dictionary] = []
	if not _short_bindings(canvas_value,snapshot_value,scene_value,current_key_value,projection_value,viewport_value) or not projection_value is RefCounted or not _finite_color(WALL_FACE): return empty
	var camera: RefCounted = projection_value
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	var current_key: String = current_key_value
	var viewport: Rect2 = viewport_value
	var before: Dictionary = _short_capture(snapshot,scene,current_key,camera)
	var staged: Array[Dictionary] = []
	var planned: Array[Dictionary] = []
	for face_value: Variant in SHORT_UNION_FACES:
		if not face_value is Dictionary: return empty
		var face: Dictionary = face_value
		var owner_value: Variant = face.get('owner')
		var a_value: Variant = face.get('a')
		var b_value: Variant = face.get('b')
		var low_owner_value: Variant = face.get('low_owner')
		if typeof(owner_value) != TYPE_STRING or not WALLS.has(owner_value) or not a_value is Array or a_value.size() != 2 or not b_value is Array or b_value.size() != 2 or typeof(low_owner_value) != TYPE_STRING: return empty
		if not P.number(a_value[0],-1000000.0,1000000.0) or not P.number(a_value[1],-1000000.0,1000000.0) or not P.number(b_value[0],-1000000.0,1000000.0) or not P.number(b_value[1],-1000000.0,1000000.0): return empty
		var wall_data: Dictionary = WALLS[owner_value]
		var low: float = 0.0
		if not low_owner_value.is_empty():
			if not WALLS.has(low_owner_value): return empty
			var low_value: Variant = WALLS[low_owner_value].get('height')
			if not P.number(low_value,0.0,0.7): return empty
			low = float(low_value)
		var high_value: Variant = wall_data.get('height')
		if not P.number(high_value,0.2,0.7) or low >= float(high_value): return empty
		var ax: float = float(a_value[0])
		var az: float = float(a_value[1])
		var bx: float = float(b_value[0])
		var bz: float = float(b_value[1])
		var world_points: Array[Vector3] = [Vector3(ax,low,az),Vector3(bx,low,bz),Vector3(bx,float(high_value),bz),Vector3(ax,float(high_value),az)]
		var points: PackedVector2Array = PackedVector2Array()
		for point: Vector3 in world_points:
			var projected: Vector2 = camera.project(point)
			if not _finite_vec2(projected): return empty
			points.append(projected)
		if not _short_quad_valid(points): return empty
		var visibility: int = _short_visible(points,viewport)
		if visibility < 0: return empty
		staged.append({'owner':owner_value,'points':points,'visibility':visibility})
	for command: Dictionary in staged:
		if command.get('visibility') == 1:
			planned.append({'owner':command.owner,'kind':'solid','points':command.points,'color':WALL_FACE})
	if not _short_recheck(canvas_value,snapshot,scene,current_key,camera,viewport,before): return empty
	return planned
func _short_apply(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, projection_value: Variant, wall_id_value: Variant, viewport_value: Variant, commands: Array[Dictionary], cable_branch: bool) -> void:
	if typeof(wall_id_value) != TYPE_STRING or not WALLS.has(wall_id_value) or not _short_bindings(canvas_value,snapshot_value,scene_value,current_key_value,projection_value,viewport_value): return
	if not snapshot_value is Dictionary or not scene_value is Dictionary or typeof(current_key_value) != TYPE_STRING or not projection_value is RefCounted: return
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	var current_key: String = current_key_value
	var camera: RefCounted = projection_value
	var before: Dictionary = _short_capture(snapshot,scene,current_key,camera)
	var planned: Array[Dictionary] = _short_prepare(canvas_value,snapshot_value,scene_value,current_key_value,projection_value,viewport_value)
	if planned.is_empty(): return
	var selected: Dictionary = {}
	for command: Dictionary in planned:
		if command.get('owner') == wall_id_value:
			selected = command
			break
	if selected.is_empty(): return
	if cable_branch:
		var one_command: Array[Dictionary] = [selected]
		if not _short_recheck(canvas_value,snapshot_value,scene_value,current_key_value,projection_value,viewport_value,before): return
		_draw_commands(canvas_value,one_command)
		return
	if commands.size() < 4 or commands[commands.size()-2].get('kind') != 'solid' or commands[commands.size()-1].get('kind') != 'polyline': return
	var insertion_index: int = commands.size()-2
	if not _short_recheck(canvas_value,snapshot_value,scene_value,current_key_value,projection_value,viewport_value,before): return
	commands.insert(insertion_index,selected)
# GB_SHORT_FUNCTION_END
