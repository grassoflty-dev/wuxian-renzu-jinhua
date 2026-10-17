extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const RELIEF_PIXELS = 12.0
const MAX_PACKETS = 160
const PIPE_ASSET = 'runtime2d.world.clockworks.pressure_pipes.v1'
const PIPE_ROWS = {
	'cw_pressure_pipe_run_north': {'anchor':Vector3(11,0,4),'footprint':Rect2(9.6,3.45,2.8,1.1)},
	'cw_pressure_pipe_run_south': {'anchor':Vector3(13,0,12),'footprint':Rect2(11.6,11.45,2.8,1.1)}
}
const VALVES = {
	'cw_pressure_valve_visual_01':Vector3(8,0,7),
	'cw_pressure_valve_visual_02':Vector3(12,0,7),
	'cw_pressure_valve_visual_03':Vector3(16,0,7)
}
const WALLS = [
	{'id':'cw_pressure_hall_wall_north','polygon':[[0.0,0.0],[24.0,0.0],[24.0,0.5],[0.0,0.5]],'inner':[[0.0,0.5],[24.0,0.5]],'seams':6,'footprint':Rect2(0,0,24,0.5)},
	{'id':'cw_pressure_hall_wall_south','polygon':[[0.0,15.5],[24.0,15.5],[24.0,16.0],[0.0,16.0]],'inner':[[0.0,15.5],[24.0,15.5]],'seams':6,'footprint':Rect2(0,15.5,24,0.5)},
	{'id':'cw_pressure_hall_wall_west_north','polygon':[[0.0,0.5],[0.5,0.5],[0.5,6.0],[0.0,6.0]],'inner':[[0.5,0.5],[0.5,6.0]],'seams':1,'footprint':Rect2(0,0.5,0.5,5.5)},
	{'id':'cw_pressure_hall_wall_west_south','polygon':[[0.0,9.5],[0.5,9.5],[0.5,15.5],[0.0,15.5]],'inner':[[0.5,9.5],[0.5,15.5]],'seams':1,'footprint':Rect2(0,9.5,0.5,6.0)},
	{'id':'cw_pressure_hall_wall_east_north','polygon':[[23.5,0.5],[24.0,0.5],[24.0,6.0],[23.5,6.0]],'inner':[[23.5,0.5],[23.5,6.0]],'seams':1,'footprint':Rect2(23.5,0.5,0.5,5.5)},
	{'id':'cw_pressure_hall_wall_east_south','polygon':[[23.5,9.5],[24.0,9.5],[24.0,15.5],[23.5,15.5]],'inner':[[23.5,9.5],[23.5,15.5]],'seams':1,'footprint':Rect2(23.5,9.5,0.5,6.0)}
]

func is_wall_id(value) -> bool:
	if typeof(value)!=TYPE_STRING:
		return false
	for wall in WALLS:
		if wall.id==value:
			return true
	return false

func has_pipe_detail(packets: Array, source_id: String) -> bool:
	if not PIPE_ROWS.has(source_id):
		return false
	var roles = ['upper','lower','cross_a','cross_b','center']
	for role in roles:
		var matches: Array = packets.filter(func(packet): return packet is Dictionary and packet.get('id')==source_id+':'+role and packet.get('role')=='pipe_detail' and packet.get('source_id')==source_id and packet.get('points') is PackedVector2Array and packet.points.size()==2)
		if matches.size()!=1:
			return false
	return true

func build_packets(scene_data: Dictionary, snapshot: Dictionary, current_key: String, projection) -> Array:
	if not _ensure_layout(scene_data,snapshot,current_key) or not _projection_valid(projection):
		return []
	var packets: Array = []
	for wall in WALLS:
		if not _add_wall(packets,wall,projection):
			return []
	for source_id in PIPE_ROWS:
		if _pipe_anchor_valid(scene_data,source_id) and not _add_pipe_group(packets,source_id,projection):
			return []
	_add_valve_shadows(packets,scene_data,projection)
	return packets if packets.size()<=MAX_PACKETS else []

func _ensure_layout(scene_data: Dictionary, snapshot: Dictionary, current_key: String) -> bool:
	if snapshot.get('worldId')!='clockworks' or scene_data.get('worldId')!='clockworks' or snapshot.get('sceneId')!='cw_pressure_hall' or scene_data.get('sceneId')!='cw_pressure_hall':
		return false
	var epoch = snapshot.get('worldEpoch')
	if not P.counter(epoch,1) or current_key!='clockworks:cw_pressure_hall:'+str(epoch):
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
	var edge: Array = wall.inner
	var a = Vector2(float(edge[0][0]),float(edge[0][1]))
	var b = Vector2(float(edge[1][0]),float(edge[1][1]))
	var scale = projection.ppm/48.0
	var relief = RELIEF_PIXELS*scale
	var ground_a: Vector2 = projection.project(Vector3(a.x,0.0,a.y))
	var ground_b: Vector2 = projection.project(Vector3(b.x,0.0,b.y))
	var top_a = ground_a-Vector2(0,relief)
	var top_b = ground_b-Vector2(0,relief)
	var face = PackedVector2Array([ground_a,ground_b,top_b,top_a])
	if not _push(packets,wall.id+':face','wall_face',face,Color(0.19,0.22,0.23),true,wall.id,wall.footprint):
		return false
	var cap = PackedVector2Array()
	for point in wall.polygon:
		cap.append(projection.project(Vector3(float(point[0]),0.0,float(point[1])))-Vector2(0,relief))
	if not _push(packets,wall.id+':cap','wall_cap',cap,Color(0.31,0.29,0.25),true,wall.id,wall.footprint):
		return false
	if not _push(packets,wall.id+':edge','wall_edge',cap,Color(0.48,0.45,0.39,0.50),false,wall.id,wall.footprint,1.1*scale):
		return false
	for index in range(1,int(wall.seams)+1):
		var t = float(index)/float(int(wall.seams)+1)
		var room_point = a.lerp(b,t)
		var foot: Vector2 = projection.project(Vector3(room_point.x,0.0,room_point.y))
		var raised = foot-Vector2(0,relief)
		if not _push(packets,wall.id+':seam:'+str(index),'panel_seam',PackedVector2Array([foot,raised]),Color(0.39,0.39,0.36,0.55),false,wall.id,wall.footprint,0.9*scale):
			return false
		if not _add_fastener(packets,wall.id+':fastener:'+str(index),foot.lerp(raised,0.72),1.25*scale,wall.id,wall.footprint):
			return false
	return true

func _add_fastener(packets: Array, packet_id: String, center: Vector2, radius: float, wall_id: String, footprint: Rect2) -> bool:
	var points = PackedVector2Array()
	for index in range(6):
		var angle = TAU*float(index)/6.0
		points.append(center+Vector2(cos(angle),sin(angle))*radius)
	return _push(packets,packet_id,'fastener',points,Color(0.52,0.48,0.40,0.60),true,wall_id,footprint)

func _pipe_anchor_valid(scene_data: Dictionary, source_id: String) -> bool:
	var presentation = scene_data.get('presentation')
	if not presentation is Dictionary:
		return false
	var sprites = presentation.get('sprites')
	if not P.rows(sprites,2048):
		return false
	var matches: Array = sprites.filter(func(row): return row is Dictionary and row.get('id')==source_id)
	if matches.size()!=1:
		return false
	var row: Dictionary = matches[0]
	var anchor: Vector3 = PIPE_ROWS[source_id].anchor
	return row.get('assetId')==PIPE_ASSET and row.get('layer')=='visual.props_back' and _authored_position(row.get('position'),anchor)

func _authored_position(value, expected: Vector3) -> bool:
	return value is Array and value.size()==3 and P.number(value[0],expected.x,expected.x) and P.number(value[1],expected.y,expected.y) and P.number(value[2],expected.z,expected.z)

func _add_pipe_group(packets: Array, source_id: String, projection) -> bool:
	var anchor: Vector3 = PIPE_ROWS[source_id].anchor
	var scale = projection.ppm/48.0
	if not _add_pipe_shadow(packets,source_id,anchor,projection):
		return false
	var x0 = anchor.x-1.2
	var x1 = anchor.x+1.2
	var z0 = anchor.z-0.19
	var z1 = anchor.z+0.19
	var offset = Vector2(0,1.2*scale)
	var a = projection.project(Vector3(x0,0,z0))-offset
	var b = projection.project(Vector3(x1,0,z0))-offset
	var c = projection.project(Vector3(x0,0,z1))-offset
	var d = projection.project(Vector3(x1,0,z1))-offset
	var center_a = projection.project(Vector3(x0+0.3,0,anchor.z))-offset
	var center_b = projection.project(Vector3(x1-0.3,0,anchor.z))-offset
	var rows = [
		['upper',PackedVector2Array([a,b]),Color(0.16,0.19,0.20,0.96),2.6],
		['lower',PackedVector2Array([c,d]),Color(0.16,0.19,0.20,0.96),2.6],
		['cross_a',PackedVector2Array([a,c]),Color(0.39,0.41,0.40,0.92),1.5],
		['cross_b',PackedVector2Array([b,d]),Color(0.39,0.41,0.40,0.92),1.5],
		['center',PackedVector2Array([center_a,center_b]),Color(0.28,0.31,0.30,0.8),1.0]
	]
	for row in rows:
		if not _push(packets,source_id+':'+row[0],'pipe_detail',row[1],row[2],false,'',PIPE_ROWS[source_id].footprint,float(row[3])*scale,source_id):
			return false
	return true

func _add_pipe_shadow(packets: Array, source_id: String, center: Vector3, projection) -> bool:
	var points = PackedVector2Array()
	for index in range(16):
		var angle = TAU*float(index)/16.0
		points.append(projection.project(Vector3(center.x+cos(angle)*1.12,0.0,center.z+sin(angle)*0.26)))
	var footprint: Rect2 = PIPE_ROWS[source_id].footprint
	return _push(packets,source_id+':contact','contact_shadow',points,Color(0.04,0.045,0.045,0.16),true,'',footprint,0.0,source_id)

func _add_valve_shadows(packets: Array, scene_data: Dictionary, projection) -> void:
	var sprites = scene_data.get('presentation',{}).get('sprites')
	if not P.rows(sprites,2048):
		return
	for source_id in VALVES:
		var matches: Array = sprites.filter(func(row): return row is Dictionary and row.get('id')==source_id)
		if matches.size()!=1:
			continue
		var row: Dictionary = matches[0]
		if row.get('assetId')!='runtime2d.world.clockworks.pressure_pipe_valve.v1' or row.get('layer')!='visual.props_dynamic' or not _authored_position(row.get('position'),VALVES[source_id]):
			continue
		var center: Vector3 = VALVES[source_id]
		var points = PackedVector2Array()
		for index in range(16):
			var angle = TAU*float(index)/16.0
			points.append(projection.project(Vector3(center.x+cos(angle)*0.42,0.0,center.z+sin(angle)*0.24)))
		_push(packets,source_id+':contact','contact_shadow',points,Color(0.04,0.05,0.05,0.18),true,'',Rect2(center.x-0.42,center.z-0.24,0.84,0.48),0.0,source_id)

func _push(packets: Array, packet_id: String, role: String, points: PackedVector2Array, color: Color, filled: bool, wall_id: String, footprint: Rect2, width: float = 0.0, source_id: String = '') -> bool:
	if points.size()<(3 if filled else 2):
		return false
	for point in points:
		if not is_finite(point.x) or not is_finite(point.y):
			return false
	var packet: Dictionary = {'id':packet_id,'role':role,'points':points,'color':color,'filled':filled,'wall_id':wall_id,'footprint':footprint}
	if width>0.0:
		packet['width'] = width
	if not source_id.is_empty():
		packet['source_id'] = source_id
	packets.append(packet)
	return packets.size()<=MAX_PACKETS

func draw_floor_treatment(canvas, scene_data: Dictionary, snapshot: Dictionary, current_key: String, projection, viewport: Rect2) -> bool:
	if not canvas is Node2D or not is_instance_valid(canvas) or canvas.is_queued_for_deletion() or not canvas.is_inside_tree():
		return false
	if not P.snapshot(snapshot) or not _ensure_layout(scene_data,snapshot,current_key) or not _registered_art_rows_valid(scene_data):
		return false
	if not _floor_projection_valid(projection) or not _floor_viewport_valid(viewport):
		return false
	var draw_quads: Array = []
	for row in range(8):
		for column in range(12):
			var x0: float = float(column*2)
			var x1: float = float(column*2+2)
			var z0: float = float(row*2)
			var z1: float = float(row*2+2)
			var x_values: Array[float] = [x0,x1,x1,x0]
			var z_values: Array[float] = [z0,z0,z1,z1]
			var points := PackedVector2Array()
			var edge_colors := PackedColorArray()
			var focus_colors := PackedColorArray()
			for vertex_index in range(4):
				var world_x: float = x_values[vertex_index]
				var world_z: float = z_values[vertex_index]
				var screen_point: Vector2 = projection.project(Vector3(world_x,0.0,world_z))
				if not is_finite(screen_point.x) or not is_finite(screen_point.y):
					return false
				points.append(screen_point)
				var edge_distance: float = minf(minf(world_x,24.0-world_x),minf(world_z,16.0-world_z))
				var edge_t: float = clampf(edge_distance/2.0,0.0,1.0)
				var edge_smooth: float = edge_t*edge_t*(3.0-2.0*edge_t)
				var edge_weight: float = 1.0-edge_smooth
				var focus_x: float = (world_x-12.0)/6.0
				var focus_z: float = (world_z-7.0)/2.5
				var focus_radius: float = sqrt(focus_x*focus_x+focus_z*focus_z)
				var focus_t: float = clampf(focus_radius,0.0,1.0)
				var focus_smooth: float = focus_t*focus_t*(3.0-2.0*focus_t)
				var focus_weight: float = 1.0-focus_smooth
				var edge_color: Color = Color(16.0/255.0,25.0/255.0,35.0/255.0,0.12*edge_weight)
				var focus_color: Color = Color(185.0/255.0,180.0/255.0,166.0/255.0,0.05*focus_weight)
				if not _floor_color_finite(edge_color) or not _floor_color_finite(focus_color):
					return false
				edge_colors.append(edge_color)
				focus_colors.append(focus_color)
			if not _floor_quad_valid(points,edge_colors) or not _floor_quad_valid(points,focus_colors):
				return false
			draw_quads.append({'points':points,'edge':edge_colors,'focus':focus_colors})
	if draw_quads.size()!=96:
		return false
	for quad in draw_quads:
		canvas.draw_polygon(quad.points,quad.edge)
	for quad in draw_quads:
		canvas.draw_polygon(quad.points,quad.focus)
	return true

func _registered_art_rows_valid(scene_data: Dictionary) -> bool:
	var presentation: Variant = scene_data.get('presentation')
	if not presentation is Dictionary:
		return false
	var sprites: Variant = presentation.get('sprites')
	if not P.rows(sprites,2048):
		return false
	for source_id in PIPE_ROWS:
		var matches: Array = []
		for candidate in sprites:
			if candidate is Dictionary and candidate.get('id')==source_id:
				matches.append(candidate)
		if matches.size()!=1:
			return false
		var row: Dictionary = matches[0]
		var expected: Dictionary = PIPE_ROWS[source_id]
		if row.get('assetId')!=PIPE_ASSET or row.get('layer')!='visual.props_back' or not _authored_position(row.get('position'),expected.anchor):
			return false
	for source_id in VALVES:
		var valve_matches: Array = []
		for valve_candidate in sprites:
			if valve_candidate is Dictionary and valve_candidate.get('id')==source_id:
				valve_matches.append(valve_candidate)
		if valve_matches.size()!=1:
			return false
		var valve_row: Dictionary = valve_matches[0]
		if valve_row.get('assetId')!='runtime2d.world.clockworks.pressure_pipe_valve.v1' or valve_row.get('layer')!='visual.props_dynamic' or not _authored_position(valve_row.get('position'),VALVES[source_id]):
			return false
	return true

func _floor_projection_valid(projection) -> bool:
	if projection==null or not projection is RefCounted or not is_instance_valid(projection) or projection.get_script()!=ProjectionScript:
		return false
	var ppm_value: Variant = projection.get('ppm')
	var origin_value: Variant = projection.get('origin')
	var center_value: Variant = projection.get('screen_center')
	if not P.number(ppm_value,0.0,256.0) or float(ppm_value)<=0.0 or not origin_value is Vector3 or not center_value is Vector2:
		return false
	var ppm: float = float(ppm_value)
	var origin: Vector3 = origin_value
	var center: Vector2 = center_value
	return is_finite(ppm) and is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and is_finite(center.x) and is_finite(center.y)

func _floor_viewport_valid(viewport: Rect2) -> bool:
	var end: Vector2 = viewport.end
	return is_finite(viewport.position.x) and is_finite(viewport.position.y) and is_finite(viewport.size.x) and is_finite(viewport.size.y) and viewport.size.x>0.0 and viewport.size.y>0.0 and is_finite(end.x) and is_finite(end.y)

func _floor_color_finite(color: Color) -> bool:
	return is_finite(color.r) and is_finite(color.g) and is_finite(color.b) and is_finite(color.a)

func _floor_quad_valid(points: PackedVector2Array, colors: PackedColorArray) -> bool:
	if points.size()!=4 or colors.size()!=4:
		return false
	for index in range(4):
		var point: Vector2 = points[index]
		if not is_finite(point.x) or not is_finite(point.y) or not _floor_color_finite(colors[index]):
			return false
	return true
