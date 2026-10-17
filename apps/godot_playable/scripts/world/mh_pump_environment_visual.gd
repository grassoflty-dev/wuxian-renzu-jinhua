extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript: Script = preload('res://scripts/world/projection.gd')
const RELIEF_PIXELS = 12.0
const MAX_PACKETS = 160
const SCENE_ID = 'mh_pump_station'
const CHANNEL_ID = 'mh_pump_east_channel_water'
const CHANNEL_POLYGON = [[18.0,5.5],[23.5,5.5],[23.5,10.5],[18.0,10.5]]
const WALLS = [
	{'id':'pump_wall_north','polygon':[[0.0,0.0],[24.0,0.0],[24.0,0.5],[0.0,0.5]],'inner':[[0.0,0.5],[24.0,0.5]],'seams':6,'footprint':Rect2(0,0,24,0.5)},
	{'id':'pump_wall_south','polygon':[[0.0,15.5],[24.0,15.5],[24.0,16.0],[0.0,16.0]],'inner':[[0.0,15.5],[24.0,15.5]],'seams':6,'footprint':Rect2(0,15.5,24,0.5)},
	{'id':'pump_wall_west_north','polygon':[[0.0,0.5],[0.5,0.5],[0.5,6.0],[0.0,6.0]],'inner':[[0.5,0.5],[0.5,6.0]],'seams':1,'footprint':Rect2(0,0.5,0.5,5.5)},
	{'id':'pump_wall_west_south','polygon':[[0.0,9.5],[0.5,9.5],[0.5,15.5],[0.0,15.5]],'inner':[[0.5,9.5],[0.5,15.5]],'seams':1,'footprint':Rect2(0,9.5,0.5,6.0)},
	{'id':'pump_wall_east_north','polygon':[[23.5,0.5],[24.0,0.5],[24.0,6.0],[23.5,6.0]],'inner':[[23.5,0.5],[23.5,6.0]],'seams':1,'footprint':Rect2(23.5,0.5,0.5,5.5)},
	{'id':'pump_wall_east_south','polygon':[[23.5,9.5],[24.0,9.5],[24.0,15.5],[23.5,15.5]],'inner':[[23.5,9.5],[23.5,15.5]],'seams':1,'footprint':Rect2(23.5,9.5,0.5,6.0)}
]

func is_wall_id(value) -> bool:
	if typeof(value)!=TYPE_STRING:
		return false
	for wall in WALLS:
		if wall.id==value:
			return true
	return false

func build_packets(scene_data: Dictionary, snapshot: Dictionary, current_key: String, projection) -> Array:
	if not _ensure_layout(scene_data,snapshot,current_key) or not _projection_valid(projection):
		return []
	var packets: Array = []
	for wall in WALLS:
		if not _add_wall(packets,wall,projection):
			return []
	if _channel_region_valid(scene_data) and not _add_channel_edge(packets,projection):
		return []
	return packets if packets.size()<MAX_PACKETS else []

func _ensure_layout(scene_data: Dictionary, snapshot: Dictionary, current_key: String) -> bool:
	if snapshot.get('worldId')!='mist_harbor' or scene_data.get('worldId')!='mist_harbor' or snapshot.get('sceneId')!=SCENE_ID or scene_data.get('sceneId')!=SCENE_ID:
		return false
	var epoch = snapshot.get('worldEpoch')
	if not P.counter(epoch,1) or current_key!='mist_harbor:'+SCENE_ID+':'+str(epoch):
		return false
	var bounds = scene_data.get('boundsM')
	if not bounds is Dictionary or bounds.size()!=4 or not P.number(bounds.get('x'),0.0,0.0) or not P.number(bounds.get('z'),0.0,0.0) or not P.number(bounds.get('width'),24.0,24.0) or not P.number(bounds.get('depth'),16.0,16.0):
		return false
	var rows = scene_data.get('collision')
	if not P.rows(rows,16) or rows.size()!=WALLS.size():
		return false
	var by_id: Dictionary = {}
	for row in rows:
		if not row is Dictionary or not is_wall_id(row.get('id')) or by_id.has(row.id):
			return false
		by_id[row.id] = row
	for wall in WALLS:
		if not by_id.has(wall.id) or not _matches_polygon(by_id[wall.id].get('polygon'),wall.polygon):
			return false
	return true

func _channel_region_valid(scene_data: Dictionary) -> bool:
	var regions = scene_data.get('terrainRegions')
	if not P.rows(regions,128):
		return false
	var matches: Array = regions.filter(func(row): return row is Dictionary and row.get('id')==CHANNEL_ID)
	return matches.size()==1 and matches[0].get('tag')=='terrain.water_deep' and _matches_polygon(matches[0].get('polygon'),CHANNEL_POLYGON)

func _matches_polygon(value, expected: Array) -> bool:
	if not value is Array or value.size()!=expected.size():
		return false
	for index in range(expected.size()):
		var actual = value[index]
		var target: Array = expected[index]
		if not actual is Array or actual.size()!=2 or not P.number(actual[0],0.0,24.0) or not P.number(actual[1],0.0,16.0) or absf(float(actual[0])-float(target[0]))>0.000001 or absf(float(actual[1])-float(target[1]))>0.000001:
			return false
	return true

func _projection_valid(projection) -> bool:
	if projection==null or not is_finite(float(projection.ppm)) or projection.ppm<=0.0 or projection.ppm>256.0:
		return false
	var sample: Vector2 = projection.project(Vector3.ZERO)
	return is_finite(sample.x) and is_finite(sample.y)

func _add_wall(packets: Array, wall: Dictionary, projection) -> bool:
	var inner: Array = wall.inner
	var a = Vector2(float(inner[0][0]),float(inner[0][1]))
	var b = Vector2(float(inner[1][0]),float(inner[1][1]))
	var scale = projection.ppm/48.0
	var relief = RELIEF_PIXELS*scale
	var ground_a: Vector2 = projection.project(Vector3(a.x,0.0,a.y))
	var ground_b: Vector2 = projection.project(Vector3(b.x,0.0,b.y))
	var top_a = ground_a-Vector2(0,relief)
	var top_b = ground_b-Vector2(0,relief)
	var face = PackedVector2Array([ground_a,ground_b,top_b,top_a])
	if not _push(packets,wall.id+':face','wall_face',face,Color(0.105,0.17,0.20),true,wall.id,wall.footprint):
		return false
	var cap = PackedVector2Array()
	for point in wall.polygon:
		cap.append(projection.project(Vector3(float(point[0]),0.0,float(point[1])))-Vector2(0,relief))
	if not _push(packets,wall.id+':cap','wall_cap',cap,Color(0.24,0.32,0.34),true,wall.id,wall.footprint):
		return false
	var outline: PackedVector2Array = cap.duplicate()
	outline.append(outline[0])
	if not _push(packets,wall.id+':fascia','wall_fascia',outline,Color(0.36,0.47,0.49,0.72),false,wall.id,wall.footprint,1.0*scale):
		return false
	var weather_a = ground_a.lerp(top_a,0.24)
	var weather_b = ground_b.lerp(top_b,0.24)
	if not _push(packets,wall.id+':weather','weather_strip',PackedVector2Array([weather_a,weather_b]),Color(0.18,0.38,0.39,0.38),false,wall.id,wall.footprint,1.2*scale):
		return false
	for index in range(1,int(wall.seams)+1):
		var t = float(index)/float(int(wall.seams)+1)
		var foot = ground_a.lerp(ground_b,t)
		var raised = top_a.lerp(top_b,t)
		if not _push(packets,wall.id+':seam:'+str(index),'panel_seam',PackedVector2Array([foot,raised]),Color(0.18,0.25,0.27,0.76),false,wall.id,wall.footprint,0.9*scale):
			return false
		if not _add_fastener(packets,wall.id+':fastener:'+str(index),foot.lerp(raised,0.72),1.2*scale,wall.id,wall.footprint):
			return false
	return packets.size()<MAX_PACKETS

func _add_fastener(packets: Array, packet_id: String, center: Vector2, radius: float, wall_id: String, footprint: Rect2) -> bool:
	var points = PackedVector2Array()
	for index in range(6):
		var angle = TAU*float(index)/6.0
		points.append(center+Vector2(cos(angle),sin(angle))*radius)
	return _push(packets,packet_id,'fastener',points,Color(0.38,0.45,0.44,0.78),true,wall_id,footprint)

func _add_channel_edge(packets: Array, projection) -> bool:
	var scale = projection.ppm/48.0
	var edge_color = Color(0.29,0.64,0.65,0.14)
	var edges = [
		['north',Vector2(18.0,5.5),Vector2(23.5,5.5),Rect2(18.0,5.5,5.5,0.0)],
		['east_north',Vector2(23.5,5.5),Vector2(23.5,6.0),Rect2(23.5,5.5,0.0,0.5)],
		['east_south',Vector2(23.5,9.5),Vector2(23.5,10.5),Rect2(23.5,9.5,0.0,1.0)],
		['south',Vector2(23.5,10.5),Vector2(18.0,10.5),Rect2(18.0,10.5,5.5,0.0)],
		['west',Vector2(18.0,10.5),Vector2(18.0,5.5),Rect2(18.0,5.5,0.0,5.0)]
	]
	for edge in edges:
		var a: Vector2 = edge[1]
		var b: Vector2 = edge[2]
		var points = PackedVector2Array([projection.project(Vector3(a.x,0.0,a.y)),projection.project(Vector3(b.x,0.0,b.y))])
		if not _push(packets,CHANNEL_ID+':water_edge:'+edge[0],'channel_water_edge',points,edge_color,false,'',edge[3],maxf(0.5,0.9*scale),CHANNEL_ID):
			return false
	return true

func _push(packets: Array, packet_id: String, role: String, points: PackedVector2Array, color: Color, filled: bool, wall_id: String, footprint: Rect2, width: float = 0.0, source_id: String = '') -> bool:
	if points.size()<(3 if filled else 2) or not is_finite(footprint.position.x) or not is_finite(footprint.position.y) or not is_finite(footprint.size.x) or not is_finite(footprint.size.y):
		return false
	for point in points:
		if not is_finite(point.x) or not is_finite(point.y):
			return false
	if width>0.0 and (not is_finite(width) or width>8.0):
		return false
	var packet: Dictionary = {'id':packet_id,'role':role,'points':points,'color':color,'filled':filled,'wall_id':wall_id,'footprint':footprint}
	if width>0.0:
		packet['width'] = width
	if not source_id.is_empty():
		packet['source_id'] = source_id
	packets.append(packet)
	return packets.size()<MAX_PACKETS

func draw_land_material(canvas: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, projection_value: Variant, viewport_value: Variant) -> bool:
	if not _material_context_valid(canvas,snapshot_value,scene_value,current_key_value,projection_value,viewport_value):
		return false
	var commands: Array[Dictionary] = _build_land_material_commands(projection_value)
	if not _material_commands_valid(commands,40,true):
		return false
	var canvas_item: CanvasItem = canvas
	_draw_material_commands(canvas_item,commands)
	return true

func draw_channel_material(canvas: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, projection_value: Variant, viewport_value: Variant) -> bool:
	if not _material_context_valid(canvas,snapshot_value,scene_value,current_key_value,projection_value,viewport_value):
		return false
	var reflection_rects: Array[Rect2] = [
		Rect2(18.25,6.1,1.95,0.25),
		Rect2(20.65,8.0,2.2,0.15),
		Rect2(18.6,9.6,1.4,0.2)
	]
	var reflection_color: Color = Color(0.28,0.56,0.62,0.07)
	var commands: Array[Dictionary] = []
	for reflection_rect: Rect2 in reflection_rects:
		var points: PackedVector2Array = _project_land_rectangle(projection_value,reflection_rect)
		if points.size()!=4 or not _material_points_finite(points) or not _material_color_finite(reflection_color):
			return false
		commands.append({'kind':'polygon','points':points,'color':reflection_color})
	if not _material_commands_valid(commands,3,false):
		return false
	var canvas_item: CanvasItem = canvas
	_draw_material_commands(canvas_item,commands)
	return true

func _material_context_valid(canvas: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, projection_value: Variant, viewport_value: Variant) -> bool:
	if not canvas is CanvasItem or not is_instance_valid(canvas) or canvas.is_queued_for_deletion() or not canvas.is_inside_tree():
		return false
	if not snapshot_value is Dictionary or not P.snapshot(snapshot_value) or not scene_value is Dictionary:
		return false
	var snapshot_data: Dictionary = snapshot_value
	var scene_data: Dictionary = scene_value
	if typeof(snapshot_data.get('worldId'))!=TYPE_STRING or snapshot_data.get('worldId')!='mist_harbor' or typeof(snapshot_data.get('sceneId'))!=TYPE_STRING or snapshot_data.get('sceneId')!=SCENE_ID:
		return false
	if typeof(scene_data.get('worldId'))!=TYPE_STRING or scene_data.get('worldId')!='mist_harbor' or typeof(scene_data.get('sceneId'))!=TYPE_STRING or scene_data.get('sceneId')!=SCENE_ID:
		return false
	if not P.number(scene_data.get('schemaVersion'),1.0,1.0) or typeof(current_key_value)!=TYPE_STRING:
		return false
	var current_key: String = str(current_key_value)
	if not _ensure_layout(scene_data,snapshot_data,current_key) or not _material_references_valid(scene_data):
		return false
	if not _material_projection_valid(projection_value) or not _material_viewport_valid(viewport_value):
		return false
	return true

func _material_references_valid(scene_data: Dictionary) -> bool:
	var walkable_value: Variant = scene_data.get('walkablePolygons')
	if not P.rows(walkable_value,16) or walkable_value.size()!=1:
		return false
	var walkable_row: Dictionary = walkable_value[0]
	if typeof(walkable_row.get('id'))!=TYPE_STRING or walkable_row.get('id')!='mh_pump_station_walkable_area' or not _matches_polygon(walkable_row.get('polygon'),[[0.5,0.5],[23.5,0.5],[23.5,15.5],[0.5,15.5]]):
		return false
	var terrain_value: Variant = scene_data.get('terrainRegions')
	if not P.rows(terrain_value,128):
		return false
	var deep_water_count: int = 0
	for terrain_item: Variant in terrain_value:
		var terrain_row: Dictionary = terrain_item
		if terrain_row.get('tag')=='terrain.water_deep':
			deep_water_count += 1
			if typeof(terrain_row.get('id'))!=TYPE_STRING or terrain_row.get('id')!=CHANNEL_ID or not _matches_polygon(terrain_row.get('polygon'),CHANNEL_POLYGON):
				return false
	if deep_water_count!=1:
		return false
	var presentation_value: Variant = scene_data.get('presentation')
	if not presentation_value is Dictionary:
		return false
	var presentation_data: Dictionary = presentation_value
	var sprites_value: Variant = presentation_data.get('sprites')
	if not P.rows(sprites_value,64):
		return false
	var floor_reference_count: int = 0
	for sprite_item: Variant in sprites_value:
		var sprite_row: Dictionary = sprite_item
		var sprite_id: Variant = sprite_row.get('id')
		var asset_id: Variant = sprite_row.get('assetId')
		if sprite_id=='approved_pump_station_stone_water_module' or asset_id=='runtime2d.world.mistharbor.water_edge_stone.v1':
			floor_reference_count += 1
			if typeof(sprite_id)!=TYPE_STRING or sprite_id!='approved_pump_station_stone_water_module' or typeof(asset_id)!=TYPE_STRING or asset_id!='runtime2d.world.mistharbor.water_edge_stone.v1' or sprite_row.get('layer')!='visual.floor' or not _material_position_matches(sprite_row.get('position'),Vector3(12.0,0.0,8.0)):
				return false
	return floor_reference_count==1

func _material_projection_valid(projection_value: Variant) -> bool:
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
	if not _material_vector3_finite(origin_value) or not _material_vector2_finite(center_value) or not _material_vector2_finite(projection_viewport_value):
		return false
	var projection_viewport: Vector2 = projection_viewport_value
	if projection_viewport.x<=0.0 or projection_viewport.y<=0.0 or not P.number(ppm_value,0.000001,256.0):
		return false
	return typeof(initialized_value)==TYPE_BOOL and initialized_value

func _material_viewport_valid(viewport_value: Variant) -> bool:
	if typeof(viewport_value)!=TYPE_RECT2:
		return false
	var viewport: Rect2 = viewport_value
	return _material_vector2_finite(viewport.position) and _material_vector2_finite(viewport.size) and _material_vector2_finite(viewport.end) and viewport.size.x>0.0 and viewport.size.y>0.0

func _material_position_matches(value: Variant, expected: Vector3) -> bool:
	if not value is Array or value.size()!=3:
		return false
	return P.number(value[0],expected.x,expected.x) and P.number(value[1],expected.y,expected.y) and P.number(value[2],expected.z,expected.z)

func _build_land_material_commands(projection_value: Variant) -> Array[Dictionary]:
	var commands: Array[Dictionary] = []
	var regions: Array[Rect2] = [Rect2(0.0,0.0,18.0,16.0),Rect2(18.0,0.0,6.0,5.5),Rect2(18.0,10.5,6.0,5.5),Rect2(23.5,5.5,0.5,5.0)]
	var base_color: Color = Color('#243943')
	var face_count: int = 0
	for region: Rect2 in regions:
		var first_x: int = floori(region.position.x/4.0)
		var last_x: int = ceili(region.end.x/4.0)
		var first_z: int = floori(region.position.y/4.0)
		var last_z: int = ceili(region.end.y/4.0)
		for gx: int in range(first_x,last_x):
			var x0: float = maxf(region.position.x,float(gx)*4.0)
			var x1: float = minf(region.end.x,float(gx+1)*4.0)
			for gz: int in range(first_z,last_z):
				var z0: float = maxf(region.position.y,float(gz)*4.0)
				var z1: float = minf(region.end.y,float(gz+1)*4.0)
				if x1<=x0 or z1<=z0:
					continue
				var face_rect: Rect2 = Rect2(x0,z0,x1-x0,z1-z0)
				var face_points: PackedVector2Array = _project_land_rectangle(projection_value,face_rect)
				if face_points.size()!=4 or not _material_points_finite(face_points):
					return []
				var variation_step: int = ((gx+2*gz)%3)-1
				var delta: float = float(variation_step)*0.006
				var face_color: Color = Color(base_color.r+delta,base_color.g+delta,base_color.b+delta,1.0)
				if not _material_color_finite(face_color):
					return []
				commands.append({'kind':'polygon','points':face_points,'color':face_color})
				face_count += 1
				if commands.size()>64:
					return []
	if face_count!=30:
		return []
	var ppm: float = float(projection_value.get('ppm'))
	var joint_width: float = 0.6*ppm/48.0
	var joint_color: Color = Color(0.04,0.08,0.10,0.14)
	var joint_count: int = 0
	var full_x_joints: Array[float] = [4.0,8.0,12.0,16.0]
	for x: float in full_x_joints:
		if not _append_material_line(commands,projection_value,Vector3(x,0.0,0.0),Vector3(x,0.0,16.0),joint_color,joint_width):
			return []
		joint_count += 1
	if not _append_material_line(commands,projection_value,Vector3(20.0,0.0,0.0),Vector3(20.0,0.0,5.5),joint_color,joint_width):
		return []
	joint_count += 1
	if not _append_material_line(commands,projection_value,Vector3(20.0,0.0,10.5),Vector3(20.0,0.0,16.0),joint_color,joint_width):
		return []
	joint_count += 1
	if not _append_material_line(commands,projection_value,Vector3(0.0,0.0,4.0),Vector3(24.0,0.0,4.0),joint_color,joint_width):
		return []
	joint_count += 1
	if not _append_material_line(commands,projection_value,Vector3(0.0,0.0,8.0),Vector3(18.0,0.0,8.0),joint_color,joint_width):
		return []
	joint_count += 1
	if not _append_material_line(commands,projection_value,Vector3(23.5,0.0,8.0),Vector3(24.0,0.0,8.0),joint_color,joint_width):
		return []
	joint_count += 1
	if not _append_material_line(commands,projection_value,Vector3(0.0,0.0,12.0),Vector3(24.0,0.0,12.0),joint_color,joint_width):
		return []
	joint_count += 1
	return commands if joint_count==10 and _material_commands_valid(commands,40,true) else []

func _append_material_line(commands: Array[Dictionary], projection_value: Variant, start_point: Vector3, end_point: Vector3, color: Color, width: float) -> bool:
	if not _material_color_finite(color) or not is_finite(width) or width<=0.0 or width>8.0:
		return false
	var start_value: Variant = projection_value.call('project',start_point)
	var end_value: Variant = projection_value.call('project',end_point)
	if not _material_vector2_finite(start_value) or not _material_vector2_finite(end_value):
		return false
	var line_points: PackedVector2Array = PackedVector2Array([start_value,end_value])
	commands.append({'kind':'line','points':line_points,'color':color,'width':width})
	return commands.size()<=64

func _project_land_rectangle(projection_value: Variant, rectangle: Rect2) -> PackedVector2Array:
	var world_corners: Array[Vector2] = [rectangle.position,Vector2(rectangle.end.x,rectangle.position.y),rectangle.end,Vector2(rectangle.position.x,rectangle.end.y)]
	var points: PackedVector2Array = PackedVector2Array()
	for world_corner: Vector2 in world_corners:
		var projected_value: Variant = projection_value.call('project',Vector3(world_corner.x,0.0,world_corner.y))
		if not _material_vector2_finite(projected_value):
			return PackedVector2Array()
		points.append(projected_value)
	return points

func _material_commands_valid(commands: Array[Dictionary], expected_count: int, allow_lines: bool) -> bool:
	if commands.size()!=expected_count or commands.is_empty() or commands.size()>64:
		return false
	for command: Dictionary in commands:
		var kind_value: Variant = command.get('kind')
		var points_value: Variant = command.get('points')
		var color_value: Variant = command.get('color')
		if typeof(kind_value)!=TYPE_STRING or typeof(points_value)!=TYPE_PACKED_VECTOR2_ARRAY or typeof(color_value)!=TYPE_COLOR:
			return false
		var points: PackedVector2Array = points_value
		var color: Color = color_value
		if not _material_points_finite(points) or not _material_color_finite(color):
			return false
		if kind_value=='polygon':
			if points.size()!=4:
				return false
		elif kind_value=='line' and allow_lines:
			var width_value: Variant = command.get('width')
			if points.size()!=2 or not P.number(width_value,0.000001,8.0):
				return false
		else:
			return false
	return true

func _draw_material_commands(canvas: CanvasItem, commands: Array[Dictionary]) -> void:
	for command: Dictionary in commands:
		var points: PackedVector2Array = command.get('points')
		var color: Color = command.get('color')
		if command.get('kind')=='polygon':
			canvas.draw_colored_polygon(points,color)
		else:
			var width: float = float(command.get('width'))
			canvas.draw_line(points[0],points[1],color,width)

func _material_points_finite(points: PackedVector2Array) -> bool:
	if points.is_empty():
		return false
	for point: Vector2 in points:
		if not is_finite(point.x) or not is_finite(point.y):
			return false
	return true

func _material_vector2_finite(value: Variant) -> bool:
	if typeof(value)!=TYPE_VECTOR2:
		return false
	var point: Vector2 = value
	return is_finite(point.x) and is_finite(point.y)

func _material_vector3_finite(value: Variant) -> bool:
	if typeof(value)!=TYPE_VECTOR3:
		return false
	var point: Vector3 = value
	return is_finite(point.x) and is_finite(point.y) and is_finite(point.z)

func _material_color_finite(color: Color) -> bool:
	return is_finite(color.r) and is_finite(color.g) and is_finite(color.b) and is_finite(color.a)
