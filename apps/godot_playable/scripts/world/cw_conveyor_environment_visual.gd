extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript: Script = preload('res://scripts/world/projection.gd')
const WORLD_ID: String = 'clockworks'
const SCENE_ID: String = 'cw_conveyor_bridge'
const FLOOR_ID: String = 'cw_conveyor_bridge_steel_floor'
const FLOOR_ASSET: String = 'runtime2d.world.clockworks.floor_steel.v1'
const BELT_ID: String = 'cw_conveyor_belt'
const BELT_POLYGON: Array = [[6.0,6.0],[18.0,6.0],[18.0,10.0],[6.0,10.0]]
const BELT_VELOCITY: Array = [-1.2,0.0]
const WALLS = [
	{'id':'cw_conveyor_bridge_wall_north','polygon':[[0.0,0.0],[24.0,0.0],[24.0,0.5],[0.0,0.5]],'inner':[[0.0,0.5],[24.0,0.5]],'contact':Rect2(0.0,0.46,24.0,0.04)},
	{'id':'cw_conveyor_bridge_wall_south','polygon':[[0.0,15.5],[24.0,15.5],[24.0,16.0],[0.0,16.0]],'inner':[[0.0,15.5],[24.0,15.5]],'contact':Rect2(0.0,15.5,24.0,0.04)},
	{'id':'cw_conveyor_bridge_wall_west_north','polygon':[[0.0,0.5],[0.5,0.5],[0.5,6.0],[0.0,6.0]],'inner':[[0.5,0.5],[0.5,6.0]],'contact':Rect2(0.46,0.5,0.04,5.5)},
	{'id':'cw_conveyor_bridge_wall_west_south','polygon':[[0.0,9.5],[0.5,9.5],[0.5,15.5],[0.0,15.5]],'inner':[[0.5,9.5],[0.5,15.5]],'contact':Rect2(0.46,9.5,0.04,6.0)},
	{'id':'cw_conveyor_bridge_wall_east_north','polygon':[[23.5,0.5],[24.0,0.5],[24.0,6.0],[23.5,6.0]],'inner':[[23.5,0.5],[23.5,6.0]],'contact':Rect2(23.5,0.5,0.04,5.5)},
	{'id':'cw_conveyor_bridge_wall_east_south','polygon':[[23.5,9.5],[24.0,9.5],[24.0,15.5],[23.5,15.5]],'inner':[[23.5,9.5],[23.5,15.5]],'contact':Rect2(23.5,9.5,0.04,6.0)}
]
const EXTERIOR_WALL_EDGES: Dictionary = {
	'cw_conveyor_bridge_wall_south':[3,2],
	'cw_conveyor_bridge_wall_east_north':[1,2],
	'cw_conveyor_bridge_wall_east_south':[1,2]
}

func draw_floor(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, projection_value: Variant, viewport_value: Variant) -> bool:
	if not _material_context_valid(canvas_value,snapshot_value,scene_value,current_key_value,projection_value,viewport_value):
		return false
	var commands: Array[Dictionary] = _build_floor_commands(projection_value)
	if not _commands_valid(commands,32,true):
		return false
	var canvas_item: CanvasItem = canvas_value
	_draw_commands(canvas_item,commands)
	return true

func draw_wall(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, projection_value: Variant, wall_value: Variant, viewport_value: Variant) -> bool:
	if not _material_context_valid(canvas_value,snapshot_value,scene_value,current_key_value,projection_value,viewport_value) or not wall_value is Dictionary:
		return false
	var wall_data: Dictionary = wall_value
	if wall_data.size()!=2:
		return false
	var wall_id_value: Variant = wall_data.get('id')
	if typeof(wall_id_value)!=TYPE_STRING:
		return false
	var definition: Dictionary = _wall_definition(str(wall_id_value))
	if definition.is_empty() or not _polygon_matches(wall_data.get('polygon'),definition.get('polygon')):
		return false
	var commands: Array[Dictionary] = _build_wall_commands(projection_value,definition)
	if not _commands_valid(commands,4,true):
		return false
	var canvas_item: CanvasItem = canvas_value
	var exterior_indices_value: Variant = EXTERIOR_WALL_EDGES.get(wall_id_value)
	if exterior_indices_value != null:
		var actual_snapshot_value: Variant = canvas_item.get('snapshot')
		var actual_scene_value: Variant = canvas_item.get('scene')
		var actual_key_value: Variant = canvas_item.get('current_key')
		var actual_camera_value: Variant = canvas_item.get('camera')
		var actual_viewport_value: Variant = canvas_item.get('viewport_rect')
		var projection_viewport_value: Variant = projection_value.get('viewport')
		var binding_matches: bool = false
		if actual_snapshot_value is Dictionary and actual_scene_value is Dictionary and typeof(actual_key_value)==TYPE_STRING and actual_camera_value is RefCounted and typeof(actual_viewport_value)==TYPE_RECT2 and typeof(projection_viewport_value)==TYPE_VECTOR2:
			var actual_snapshot: Dictionary = actual_snapshot_value
			var actual_scene: Dictionary = actual_scene_value
			var actual_viewport: Rect2 = actual_viewport_value
			var projection_viewport: Vector2 = projection_viewport_value
			binding_matches = actual_snapshot==snapshot_value and actual_scene==scene_value and actual_key_value==current_key_value and actual_camera_value==projection_value and actual_viewport==viewport_value and projection_viewport==actual_viewport.size
		if binding_matches:
			if not exterior_indices_value is Array or exterior_indices_value.size()!=2:
				return false
			var exterior_indices: Array = exterior_indices_value
			var polygon_value: Variant = definition.get('polygon')
			if not polygon_value is Array or typeof(exterior_indices[0])!=TYPE_INT or typeof(exterior_indices[1])!=TYPE_INT:
				return false
			var polygon: Array = polygon_value
			var first_index: int = exterior_indices[0]
			var second_index: int = exterior_indices[1]
			if first_index<0 or second_index<0 or first_index>=polygon.size() or second_index>=polygon.size():
				return false
			var first_vertex_value: Variant = polygon[first_index]
			var second_vertex_value: Variant = polygon[second_index]
			if not first_vertex_value is Array or not second_vertex_value is Array:
				return false
			var first_vertex: Array = first_vertex_value
			var second_vertex: Array = second_vertex_value
			if first_vertex.size()!=2 or second_vertex.size()!=2 or not P.number(first_vertex[0],-24.0,24.0) or not P.number(first_vertex[1],0.0,16.0) or not P.number(second_vertex[0],-24.0,24.0) or not P.number(second_vertex[1],0.0,16.0):
				return false
			var projection: RefCounted = projection_value
			var first_ground_value: Variant = projection.call('project',Vector3(float(first_vertex[0]),0.0,float(first_vertex[1])))
			var second_ground_value: Variant = projection.call('project',Vector3(float(second_vertex[0]),0.0,float(second_vertex[1])))
			var ppm_value: Variant = projection.get('ppm')
			if not _vector2_finite(first_ground_value) or not _vector2_finite(second_ground_value) or not P.number(ppm_value,0.000001,256.0):
				return false
			var relief: float = 12.0*float(ppm_value)/48.0
			var first_ground: Vector2 = first_ground_value
			var second_ground: Vector2 = second_ground_value
			var exterior_points: PackedVector2Array = PackedVector2Array([first_ground,second_ground,second_ground-Vector2(0.0,relief),first_ground-Vector2(0.0,relief)])
			if not is_finite(relief) or not _points_finite(exterior_points):
				return false
			var twice_area: float = 2.0*(second_ground-first_ground).cross(Vector2(0.0,-relief))
			if not is_finite(twice_area) or twice_area==0.0:
				return false
			var exterior_commands: Array[Dictionary] = [{'kind':'polygon','points':exterior_points,'color':Color('#303633')}]
			if not _commands_valid(exterior_commands,1,false):
				return false
			commands.insert(1,exterior_commands[0])
			if not _commands_valid(commands,5,true):
				return false
	_draw_commands(canvas_item,commands)
	return true

func draw_conveyor(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, projection_value: Variant, viewport_value: Variant, reduced_motion_value: Variant) -> bool:
	if typeof(reduced_motion_value)!=TYPE_BOOL or not _material_context_valid(canvas_value,snapshot_value,scene_value,current_key_value,projection_value,viewport_value):
		return false
	if not reduced_motion_value:
		# Bound the safe integer tick before float conversion; no wall-clock timer.
		var phase: float = float(int(snapshot_value.serverTick)%25)*0.02
		var moving: Array[Dictionary] = _build_moving_conveyor_commands(projection_value,phase)
		if _moving_conveyor_commands_valid(moving):
			_draw_commands(canvas_value,moving)
			return true
	# Reduced motion and any failed moving plan retain the original static path.
	var commands: Array[Dictionary] = _build_conveyor_commands(projection_value)
	if not _commands_valid(commands,26,true):
		return false
	var canvas_item: CanvasItem = canvas_value
	_draw_commands(canvas_item,commands)
	return true

func _material_context_valid(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, projection_value: Variant, viewport_value: Variant) -> bool:
	if not canvas_value is CanvasItem or not is_instance_valid(canvas_value):
		return false
	var canvas_item: CanvasItem = canvas_value
	if canvas_item.is_queued_for_deletion() or not canvas_item.is_inside_tree():
		return false
	if not snapshot_value is Dictionary or not P.snapshot(snapshot_value) or not scene_value is Dictionary:
		return false
	var snapshot_data: Dictionary = snapshot_value
	var scene_data: Dictionary = scene_value
	var snapshot_world: Variant = snapshot_data.get('worldId')
	var snapshot_scene: Variant = snapshot_data.get('sceneId')
	var scene_world: Variant = scene_data.get('worldId')
	var scene_id: Variant = scene_data.get('sceneId')
	if typeof(snapshot_world)!=TYPE_STRING or snapshot_world!=WORLD_ID or typeof(snapshot_scene)!=TYPE_STRING or snapshot_scene!=SCENE_ID:
		return false
	if typeof(scene_world)!=TYPE_STRING or scene_world!=WORLD_ID or typeof(scene_id)!=TYPE_STRING or scene_id!=SCENE_ID:
		return false
	if not P.number(scene_data.get('schemaVersion'),1.0,1.0) or typeof(current_key_value)!=TYPE_STRING:
		return false
	var epoch_value: Variant = snapshot_data.get('worldEpoch')
	if not P.counter(epoch_value,1):
		return false
	var current_key: String = current_key_value
	if current_key!=WORLD_ID+':'+SCENE_ID+':'+str(epoch_value):
		return false
	var bounds_value: Variant = scene_data.get('boundsM')
	if not bounds_value is Dictionary:
		return false
	var bounds: Dictionary = bounds_value
	if bounds.size()!=4 or not P.number(bounds.get('x'),0.0,0.0) or not P.number(bounds.get('z'),0.0,0.0) or not P.number(bounds.get('width'),24.0,24.0) or not P.number(bounds.get('depth'),16.0,16.0):
		return false
	if not _collision_layout_valid(scene_data) or not _conveyor_region_valid(scene_data) or not _floor_reference_valid(scene_data):
		return false
	return _projection_valid(projection_value) and _viewport_valid(viewport_value)

func _collision_layout_valid(scene_data: Dictionary) -> bool:
	var collision_value: Variant = scene_data.get('collision')
	if not P.rows(collision_value,6):
		return false
	var collision_rows: Array = collision_value
	if collision_rows.size()!=6:
		return false
	var rows_by_id: Dictionary = {}
	for row_value: Variant in collision_rows:
		if not row_value is Dictionary:
			return false
		var row_data: Dictionary = row_value
		if row_data.size()!=2:
			return false
		var row_id: Variant = row_data.get('id')
		if typeof(row_id)!=TYPE_STRING or rows_by_id.has(row_id):
			return false
		rows_by_id[row_id] = row_data
	for expected_value: Variant in WALLS:
		var expected_wall: Dictionary = expected_value
		var expected_id: String = expected_wall.get('id')
		if not rows_by_id.has(expected_id):
			return false
		var actual_wall: Dictionary = rows_by_id.get(expected_id)
		var expected_polygon_value: Variant = expected_wall.get('polygon')
		if not expected_polygon_value is Array or not _polygon_matches(actual_wall.get('polygon'),expected_polygon_value):
			return false
	return rows_by_id.size()==WALLS.size()

func _conveyor_region_valid(scene_data: Dictionary) -> bool:
	var regions_value: Variant = scene_data.get('terrainRegions')
	if not P.rows(regions_value,1):
		return false
	var regions: Array = regions_value
	if regions.size()!=1 or not regions[0] is Dictionary:
		return false
	var belt: Dictionary = regions[0]
	if belt.size()!=4:
		return false
	var belt_id: Variant = belt.get('id')
	var belt_tag: Variant = belt.get('tag')
	if typeof(belt_id)!=TYPE_STRING or belt_id!=BELT_ID or typeof(belt_tag)!=TYPE_STRING or belt_tag!='conveyor':
		return false
	var polygon_value: Variant = belt.get('polygon')
	if not polygon_value is Array or not _polygon_matches(polygon_value,BELT_POLYGON):
		return false
	var velocity_value: Variant = belt.get('surfaceVelocityMps')
	if not velocity_value is Array:
		return false
	var velocity: Array = velocity_value
	if velocity.size()!=2 or not _number_matches(velocity[0],-1.2) or not _number_matches(velocity[1],0.0):
		return false
	return true

func _floor_reference_valid(scene_data: Dictionary) -> bool:
	var presentation_value: Variant = scene_data.get('presentation')
	if not presentation_value is Dictionary:
		return false
	var presentation_data: Dictionary = presentation_value
	var sprites_value: Variant = presentation_data.get('sprites')
	if not P.rows(sprites_value,64):
		return false
	var sprites: Array = sprites_value
	var candidates: int = 0
	for sprite_value: Variant in sprites:
		if not sprite_value is Dictionary:
			return false
		var sprite: Dictionary = sprite_value
		var sprite_id: Variant = sprite.get('id')
		var asset_id: Variant = sprite.get('assetId')
		if sprite_id==FLOOR_ID or asset_id==FLOOR_ASSET:
			candidates += 1
			if sprite.size()!=4 or typeof(sprite_id)!=TYPE_STRING or sprite_id!=FLOOR_ID or typeof(asset_id)!=TYPE_STRING or asset_id!=FLOOR_ASSET:
				return false
			var layer: Variant = sprite.get('layer')
			if typeof(layer)!=TYPE_STRING or layer!='visual.floor' or not _position_matches(sprite.get('position'),Vector3(12.0,0.0,8.0)):
				return false
	return candidates==1

func _position_matches(value: Variant, expected: Vector3) -> bool:
	if not value is Array:
		return false
	var position: Array = value
	if position.size()!=3:
		return false
	return _number_matches(position[0],expected.x) and _number_matches(position[1],expected.y) and _number_matches(position[2],expected.z)

func _projection_valid(projection_value: Variant) -> bool:
	if not projection_value is RefCounted or not is_instance_valid(projection_value):
		return false
	var projection: RefCounted = projection_value
	if projection.get_script()!=ProjectionScript:
		return false
	var origin_value: Variant = projection.get('origin')
	var center_value: Variant = projection.get('screen_center')
	var projection_viewport_value: Variant = projection.get('viewport')
	var ppm_value: Variant = projection.get('ppm')
	var initialized_value: Variant = projection.get('initialized')
	if not _vector3_finite(origin_value) or not _vector2_finite(center_value) or not _vector2_finite(projection_viewport_value):
		return false
	var projection_viewport: Vector2 = projection_viewport_value
	if projection_viewport.x<=0.0 or projection_viewport.y<=0.0 or not P.number(ppm_value,0.000001,256.0):
		return false
	return typeof(initialized_value)==TYPE_BOOL and initialized_value

func _viewport_valid(viewport_value: Variant) -> bool:
	if typeof(viewport_value)!=TYPE_RECT2:
		return false
	var viewport: Rect2 = viewport_value
	return _vector2_finite(viewport.position) and _vector2_finite(viewport.size) and _vector2_finite(viewport.end) and viewport.size.x>0.0 and viewport.size.y>0.0

func _polygon_matches(value: Variant, expected: Array) -> bool:
	if not value is Array or value.size()!=expected.size():
		return false
	var actual_polygon: Array = value
	for vertex_index: int in range(expected.size()):
		var actual_vertex_value: Variant = actual_polygon[vertex_index]
		var expected_vertex_value: Variant = expected[vertex_index]
		if not actual_vertex_value is Array or not expected_vertex_value is Array:
			return false
		var actual_vertex: Array = actual_vertex_value
		var expected_vertex: Array = expected_vertex_value
		if actual_vertex.size()!=2 or expected_vertex.size()!=2 or not P.number(actual_vertex[0],-1000000.0,1000000.0) or not P.number(actual_vertex[1],-1000000.0,1000000.0):
			return false
		if absf(float(actual_vertex[0])-float(expected_vertex[0]))>0.000001 or absf(float(actual_vertex[1])-float(expected_vertex[1]))>0.000001:
			return false
	return true

func _number_matches(value: Variant, expected: float) -> bool:
	return P.number(value,expected-0.000001,expected+0.000001)

func _wall_definition(wall_id: String) -> Dictionary:
	for wall_value: Variant in WALLS:
		var wall_definition: Dictionary = wall_value
		if wall_definition.get('id')==wall_id:
			return wall_definition
	return {}

func _build_floor_commands(projection_value: Variant) -> Array[Dictionary]:
	var commands: Array[Dictionary] = []
	for row: int in range(4):
		for column: int in range(6):
			var cell: Rect2 = Rect2(float(column*4),float(row*4),4.0,4.0)
			var points: PackedVector2Array = _project_rectangle(projection_value,cell,0.0)
			if points.size()!=4 or not _points_finite(points):
				return []
			var variation_step: int = ((column+2*row)%3)-1
			var delta: float = float(variation_step)*0.004
			var colors: PackedColorArray = PackedColorArray()
			colors.append(_floor_vertex_color(cell.position.x,cell.position.y,delta))
			colors.append(_floor_vertex_color(cell.end.x,cell.position.y,delta))
			colors.append(_floor_vertex_color(cell.end.x,cell.end.y,delta))
			colors.append(_floor_vertex_color(cell.position.x,cell.end.y,delta))
			commands.append({'kind':'gradient_polygon','points':points,'colors':colors})
	var ppm_value: Variant = projection_value.get('ppm')
	var ppm: float = float(ppm_value)
	var joint_color: Color = Color(0.06,0.065,0.065,0.10)
	var joint_width: float = 0.6*ppm/48.0
	var x_joints: Array[float] = [4.0,8.0,12.0,16.0,20.0]
	for joint_x: float in x_joints:
		if not _append_line(commands,projection_value,Vector3(joint_x,0.0,0.0),Vector3(joint_x,0.0,16.0),joint_color,joint_width):
			return []
	var z_joints: Array[float] = [4.0,8.0,12.0]
	for joint_z: float in z_joints:
		if not _append_line(commands,projection_value,Vector3(0.0,0.0,joint_z),Vector3(24.0,0.0,joint_z),joint_color,joint_width):
			return []
	return commands

func _floor_vertex_color(x: float, z: float, delta: float) -> Color:
	var edge_distance: float = minf(minf(x,24.0-x),minf(z,16.0-z))
	var edge_value: float = 1.0-_smoothstep01(edge_distance/2.0)
	var bay_value: float = 1.0-_smoothstep01(absf(z-8.0)/6.0)
	var value_delta: float = delta-0.016*edge_value+0.012*bay_value
	var base: Color = Color('#3b3833')
	return Color(base.r+value_delta,base.g+value_delta,base.b+value_delta,1.0)

func _smoothstep01(value: float) -> float:
	var t: float = clampf(value,0.0,1.0)
	return t*t*(3.0-2.0*t)

func _build_wall_commands(projection_value: Variant, wall: Dictionary) -> Array[Dictionary]:
	var commands: Array[Dictionary] = []
	var contact_value: Variant = wall.get('contact')
	if typeof(contact_value)!=TYPE_RECT2:
		return []
	var contact_rect: Rect2 = contact_value
	var south_join: bool = wall.get('id') in ['cw_conveyor_bridge_wall_west_south','cw_conveyor_bridge_wall_east_south']
	if south_join:
		# Rect2 is a value copy; canonical contact/layout constants stay unchanged.
		contact_rect.size.y -= 12.0/48.0
	var contact_points: PackedVector2Array = _project_rectangle(projection_value,contact_rect,0.0)
	if contact_points.size()!=4 or not _points_finite(contact_points):
		return []
	commands.append({'kind':'polygon','points':contact_points,'color':Color(0.025,0.03,0.03,0.12)})
	var inner_value: Variant = wall.get('inner')
	var polygon_value: Variant = wall.get('polygon')
	if not inner_value is Array or not polygon_value is Array:
		return []
	var inner: Array = inner_value
	if inner.size()!=2 or not inner[0] is Array or not inner[1] is Array:
		return []
	var first: Array = inner[0]
	var second: Array = inner[1]
	if first.size()!=2 or second.size()!=2:
		return []
	var projection: RefCounted = projection_value
	var first_ground_value: Variant = projection.call('project',Vector3(float(first[0]),0.0,float(first[1])))
	var second_ground_value: Variant = projection.call('project',Vector3(float(second[0]),0.0,float(second[1])))
	if not _vector2_finite(first_ground_value) or not _vector2_finite(second_ground_value):
		return []
	var first_ground: Vector2 = first_ground_value
	var second_ground: Vector2 = second_ground_value
	var ppm_value: Variant = projection.get('ppm')
	var relief: float = 12.0*float(ppm_value)/48.0
	if not is_finite(relief) or relief<=0.0 or relief>64.0:
		return []
	var first_raised: Vector2 = first_ground-Vector2(0.0,relief)
	var second_raised: Vector2 = second_ground-Vector2(0.0,relief)
	var visible_second_ground: Vector2 = second_ground
	if south_join:
		# Keep the original raised end/cap; remove only the south-cap-hidden corner.
		visible_second_ground = projection.project(Vector3(float(second[0]),0.0,float(second[1])-12.0/48.0))
	var side_points: PackedVector2Array = PackedVector2Array([first_ground,visible_second_ground,second_raised,first_raised])
	commands.append({'kind':'polygon','points':side_points,'color':Color('#303633')})
	var cap_points: PackedVector2Array = _project_polygon(projection_value,polygon_value,relief)
	if cap_points.size()!=4 or not _points_finite(cap_points):
		return []
	commands.append({'kind':'polygon','points':cap_points,'color':Color('#454c46')})
	var outline_points: PackedVector2Array = cap_points.duplicate()
	outline_points.append(outline_points[0])
	var outline_width: float = 0.8*float(ppm_value)/48.0
	commands.append({'kind':'polyline','points':outline_points,'color':Color(0.44,0.49,0.45,0.35),'width':outline_width})
	return commands

func _build_conveyor_commands(projection_value: Variant) -> Array[Dictionary]:
	var commands: Array[Dictionary] = []
	if not _append_rectangle(commands,projection_value,Rect2(6.0,6.0,12.0,4.0),Color('#20292a')):
		return []
	if not _append_rectangle(commands,projection_value,Rect2(6.0,6.0,12.0,0.08),Color(0.04,0.06,0.065,0.20)):
		return []
	if not _append_rectangle(commands,projection_value,Rect2(6.0,9.92,12.0,0.08),Color(0.04,0.06,0.065,0.20)):
		return []
	var projection: RefCounted = projection_value
	var ppm_value: Variant = projection.get('ppm')
	var rib_color: Color = Color(0.24,0.28,0.27,0.45)
	var rib_width: float = 0.7*float(ppm_value)/48.0
	for rib_index: int in range(23):
		var rib_x: float = 6.5+0.5*float(rib_index)
		if not _append_line(commands,projection_value,Vector3(rib_x,0.0,6.08),Vector3(rib_x,0.0,9.92),rib_color,rib_width):
			return []
	return commands

func _append_rectangle(commands: Array[Dictionary], projection_value: Variant, rectangle: Rect2, color: Color) -> bool:
	var points: PackedVector2Array = _project_rectangle(projection_value,rectangle,0.0)
	if points.size()!=4 or not _points_finite(points) or not _color_finite(color):
		return false
	commands.append({'kind':'polygon','points':points,'color':color})
	return commands.size()<=64

func _append_line(commands: Array[Dictionary], projection_value: Variant, start_world: Vector3, end_world: Vector3, color: Color, width: float) -> bool:
	if not _color_finite(color) or not is_finite(width) or width<=0.0 or width>8.0:
		return false
	var projection: RefCounted = projection_value
	var start_value: Variant = projection.call('project',start_world)
	var end_value: Variant = projection.call('project',end_world)
	if not _vector2_finite(start_value) or not _vector2_finite(end_value):
		return false
	var points: PackedVector2Array = PackedVector2Array([start_value,end_value])
	commands.append({'kind':'line','points':points,'color':color,'width':width})
	return commands.size()<=64

func _project_rectangle(projection_value: Variant, rectangle: Rect2, screen_raise: float) -> PackedVector2Array:
	var world_points: Array[Vector3] = [
		Vector3(rectangle.position.x,0.0,rectangle.position.y),
		Vector3(rectangle.end.x,0.0,rectangle.position.y),
		Vector3(rectangle.end.x,0.0,rectangle.end.y),
		Vector3(rectangle.position.x,0.0,rectangle.end.y)
	]
	var points: PackedVector2Array = PackedVector2Array()
	var projection: RefCounted = projection_value
	for world_point: Vector3 in world_points:
		var point_value: Variant = projection.call('project',world_point)
		if not _vector2_finite(point_value):
			return PackedVector2Array()
		var point: Vector2 = point_value
		points.append(point-Vector2(0.0,screen_raise))
	return points

func _project_polygon(projection_value: Variant, polygon_value: Variant, screen_raise: float) -> PackedVector2Array:
	if not polygon_value is Array:
		return PackedVector2Array()
	var polygon: Array = polygon_value
	var points: PackedVector2Array = PackedVector2Array()
	var projection: RefCounted = projection_value
	for vertex_value: Variant in polygon:
		if not vertex_value is Array:
			return PackedVector2Array()
		var vertex: Array = vertex_value
		if vertex.size()!=2 or not P.number(vertex[0],0.0,24.0) or not P.number(vertex[1],0.0,16.0):
			return PackedVector2Array()
		var world_point: Vector3 = Vector3(float(vertex[0]),0.0,float(vertex[1]))
		var point_value: Variant = projection.call('project',world_point)
		if not _vector2_finite(point_value):
			return PackedVector2Array()
		var point: Vector2 = point_value
		points.append(point-Vector2(0.0,screen_raise))
	return points

func _commands_valid(commands: Array[Dictionary], expected_count: int, allow_lines: bool) -> bool:
	if commands.size()!=expected_count or commands.is_empty() or commands.size()>64:
		return false
	for command: Dictionary in commands:
		var kind_value: Variant = command.get('kind')
		var points_value: Variant = command.get('points')
		if typeof(kind_value)!=TYPE_STRING or typeof(points_value)!=TYPE_PACKED_VECTOR2_ARRAY:
			return false
		var points: PackedVector2Array = points_value
		if not _points_finite(points):
			return false
		if kind_value=='gradient_polygon':
			var colors_value: Variant = command.get('colors')
			if typeof(colors_value)!=TYPE_PACKED_COLOR_ARRAY or points.size()!=4:
				return false
			var vertex_colors: PackedColorArray = colors_value
			if vertex_colors.size()!=points.size():
				return false
			for vertex_color: Color in vertex_colors:
				if not _color_finite(vertex_color):
					return false
		elif kind_value=='polygon':
			var polygon_color_value: Variant = command.get('color')
			if points.size()!=4 or typeof(polygon_color_value)!=TYPE_COLOR:
				return false
			var polygon_color: Color = polygon_color_value
			if not _color_finite(polygon_color):
				return false
		elif kind_value=='line' or kind_value=='polyline':
			if not allow_lines or typeof(command.get('color'))!=TYPE_COLOR:
				return false
			var line_color: Color = command.get('color')
			var width_value: Variant = command.get('width')
			if not _color_finite(line_color) or not P.number(width_value,0.000001,8.0):
				return false
			if kind_value=='line' and points.size()!=2 or kind_value=='polyline' and points.size()<3:
				return false
		else:
			return false
	return true

func _draw_commands(canvas: CanvasItem, commands: Array[Dictionary]) -> void:
	for command: Dictionary in commands:
		var kind_value: Variant = command.get('kind')
		var points: PackedVector2Array = command.get('points')
		if kind_value=='gradient_polygon':
			var gradient_colors: PackedColorArray = command.get('colors')
			canvas.draw_polygon(points,gradient_colors)
		elif kind_value=='polygon':
			var polygon_draw_color: Color = command.get('color')
			canvas.draw_colored_polygon(points,polygon_draw_color)
		elif kind_value=='line':
			var line_draw_color: Color = command.get('color')
			var line_width: float = float(command.get('width'))
			canvas.draw_line(points[0],points[1],line_draw_color,line_width)
		else:
			var polyline_draw_color: Color = command.get('color')
			var polyline_width: float = float(command.get('width'))
			canvas.draw_polyline(points,polyline_draw_color,polyline_width)

func _points_finite(points: PackedVector2Array) -> bool:
	if points.is_empty():
		return false
	for point: Vector2 in points:
		if not is_finite(point.x) or not is_finite(point.y):
			return false
	return true

func _vector2_finite(value: Variant) -> bool:
	if typeof(value)!=TYPE_VECTOR2:
		return false
	var point: Vector2 = value
	return is_finite(point.x) and is_finite(point.y)

func _vector3_finite(value: Variant) -> bool:
	if typeof(value)!=TYPE_VECTOR3:
		return false
	var point: Vector3 = value
	return is_finite(point.x) and is_finite(point.y) and is_finite(point.z)

func _color_finite(color: Color) -> bool:
	return is_finite(color.r) and is_finite(color.g) and is_finite(color.b) and is_finite(color.a)

# Candidate material motion only. These x-clipped fills have no exterior stroke;
# their perpendicular projected width matches the original 0.7*ppm/48 ribs.
func _build_moving_conveyor_commands(projection_value: Variant, phase: float) -> Array[Dictionary]:
	var commands: Array[Dictionary] = []
	var ppm: float = float(projection_value.get('ppm'))
	var screen_width: float = 0.7*ppm/48.0
	if not is_finite(phase) or phase<0.0 or phase>=0.5 or not P.number(screen_width,0.000001,8.0):
		return []
	if not _append_rectangle(commands,projection_value,Rect2(6.0,6.0,12.0,4.0),Color('#20292a')):
		return []
	if not _append_rectangle(commands,projection_value,Rect2(6.0,6.0,12.0,0.08),Color(0.04,0.06,0.065,0.20)):
		return []
	if not _append_rectangle(commands,projection_value,Rect2(6.0,9.92,12.0,0.08),Color(0.04,0.06,0.065,0.20)):
		return []
	var half_width: float = 0.5*0.7*sqrt(1.25)/48.0
	for rib_index: int in range(25):
		var center: float = 6.0+0.5*float(rib_index)-phase
		var left: float = maxf(6.0,center-half_width)
		var right: float = minf(18.0,center+half_width)
		if right<=left:
			continue
		# Every retained positive-world-area fragment is mandatory in preflight.
		if not _append_rectangle(commands,projection_value,Rect2(left,6.08,right-left,9.92-6.08),Color(0.24,0.28,0.27,0.45)):
			return []
	return commands

func _moving_conveyor_commands_valid(commands: Array[Dictionary]) -> bool:
	if commands.size()!=27 and commands.size()!=28:
		return false
	if not _commands_valid(commands,27,false) and not _commands_valid(commands,28,false):
		return false
	for index: int in range(commands.size()):
		var command: Dictionary = commands[index]
		if command.get('kind')!='polygon':
			return false
		var expected_color: Color = Color(0.24,0.28,0.27,0.45)
		if index==0:
			expected_color = Color('#20292a')
		elif index<3:
			expected_color = Color(0.04,0.06,0.065,0.20)
		if command.get('color')!=expected_color:
			return false
		var points: PackedVector2Array = command.points
		var twice_area: float = 0.0
		for vertex: int in range(1,3):
			var triangle: float = (points[vertex]-points[0]).cross(points[vertex+1]-points[0])
			if not is_finite(triangle):
				return false
			twice_area += triangle
		if not is_finite(twice_area) or absf(twice_area)*0.5<=0.000001:
			return false
	return true
