extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const PROJECTION_SCRIPT = preload('res://scripts/world/projection.gd')
const RELIEF_PIXELS = 12.0
const MAX_PACKETS = 160
const PIPE_PROP = 'power_room_pipes'
const WALLS = [
	{'id':'gh_power_wall_north','polygon':[[0.0,0.0],[20.0,0.0],[20.0,0.5],[0.0,0.5]],'inner':[[0.0,0.5],[20.0,0.5]],'seams':4,'footprint':[0.0,0.0,20.0,0.5]},
	{'id':'gh_power_wall_south','polygon':[[0.0,13.5],[20.0,13.5],[20.0,14.0],[0.0,14.0]],'inner':[[0.0,13.5],[20.0,13.5]],'seams':4,'footprint':[0.0,13.5,20.0,0.5]},
	{'id':'gh_power_wall_west_north','polygon':[[0.0,0.5],[0.5,0.5],[0.5,5.5],[0.0,5.5]],'inner':[[0.5,0.5],[0.5,5.5]],'seams':1,'footprint':[0.0,0.5,0.5,5.0]},
	{'id':'gh_power_wall_west_south','polygon':[[0.0,8.5],[0.5,8.5],[0.5,13.5],[0.0,13.5]],'inner':[[0.5,8.5],[0.5,13.5]],'seams':1,'footprint':[0.0,8.5,0.5,5.0]},
	{'id':'gh_power_wall_east_north','polygon':[[19.5,0.5],[20.0,0.5],[20.0,5.5],[19.5,5.5]],'inner':[[19.5,0.5],[19.5,5.5]],'seams':1,'footprint':[19.5,0.5,0.5,5.0]},
	{'id':'gh_power_wall_east_south','polygon':[[19.5,8.5],[20.0,8.5],[20.0,13.5],[19.5,13.5]],'inner':[[19.5,8.5],[19.5,13.5]],'seams':1,'footprint':[19.5,8.5,0.5,5.0]}
]
const OUTER_WALL_EDGES: Dictionary = {
	'gh_power_wall_south': [Vector2(0.0,14.0),Vector2(20.0,14.0)],
	'gh_power_wall_east_north': [Vector2(20.0,0.5),Vector2(20.0,5.5)],
	'gh_power_wall_east_south': [Vector2(20.0,8.5),Vector2(20.0,13.5)]
}

func is_wall_id(value) -> bool:
	if typeof(value)!=TYPE_STRING:
		return false
	for wall in WALLS:
		if wall.id==value:
			return true
	return false

func has_pipe_detail(packets: Array) -> bool:
	for packet in packets:
		if packet is Dictionary and packet.get('role')=='pipe_detail' and packet.get('source_id')==PIPE_PROP:
			return true
	return false

func build_packets(scene_data: Dictionary, snapshot: Dictionary, current_key: String, projection, light_state: String = 'unknown') -> Array:
	if not _ensure_layout(scene_data,snapshot,current_key) or not _projection_valid(projection):
		return []
	var packets: Array = []
	for wall in WALLS:
		var wall_start: int = packets.size()
		if not _add_wall(packets,wall,projection):
			return []
		var exterior_packet: Variant = _build_outer_wall_packet(scene_data,snapshot,current_key,projection,wall)
		if typeof(exterior_packet)==TYPE_BOOL:
			return []
		if exterior_packet is Dictionary:
			packets.insert(wall_start+1,exterior_packet)
	if _pipe_anchor_valid(scene_data) and not _add_pipe_detail(packets,projection):
		return []
	if not _add_contact_shadow(packets,projection):
		return []
	if light_state=='on' and not _add_task_light(packets,projection):
		return []
	if light_state=='off' and not _add_off_emergency_light(packets,projection):
		return []
	return packets if packets.size()<=MAX_PACKETS else []

func _ensure_layout(scene_data: Dictionary, snapshot: Dictionary, current_key: String) -> bool:
	if snapshot.get('worldId')!='grey_hive' or scene_data.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_power_room' or scene_data.get('sceneId')!='gh_power_room':
		return false
	var epoch = snapshot.get('worldEpoch')
	if not P.counter(epoch,1) or current_key!='grey_hive:gh_power_room:'+str(epoch):
		return false
	var bounds = scene_data.get('boundsM')
	if not bounds is Dictionary or bounds.size()!=4 or not P.number(bounds.get('x'),0.0,0.0) or not P.number(bounds.get('z'),0.0,0.0) or not P.number(bounds.get('width'),20.0,20.0) or not P.number(bounds.get('depth'),14.0,14.0):
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
		var actual = _read_point(value[index])
		var target: Array = expected[index]
		if not is_finite(actual.x) or not is_finite(actual.y) or absf(actual.x-float(target[0]))>0.000001 or absf(actual.y-float(target[1]))>0.000001:
			return false
	return true

func _read_point(value) -> Vector2:
	if value is String:
		var parts = value.strip_edges().split(' ',false)
		if parts.size()!=2 or not parts[0].is_valid_float() or not parts[1].is_valid_float():
			return Vector2(INF,INF)
		var x = parts[0].to_float()
		var z = parts[1].to_float()
		if not P.number(x,0.0,20.0) or not P.number(z,0.0,14.0):
			return Vector2(INF,INF)
		return Vector2(x,z)
	if value is Array and value.size()==2 and P.number(value[0],0.0,20.0) and P.number(value[1],0.0,14.0):
		return Vector2(float(value[0]),float(value[1]))
	return Vector2(INF,INF)

func _pipe_anchor_valid(scene_data: Dictionary) -> bool:
	var presentation = scene_data.get('presentation')
	if not presentation is Dictionary:
		return false
	var sprites = presentation.get('sprites')
	if not P.rows(sprites,2048):
		return false
	var matches: Array = sprites.filter(func(row): return row is Dictionary and row.get('id')==PIPE_PROP)
	if matches.size()!=1:
		return false
	var row: Dictionary = matches[0]
	var position = row.get('position')
	if row.get('assetId')!='runtime2d.grey_hive.pipes_cables.v1' or row.get('layer')!='visual.props_back' or not position is Array or position.size()!=3:
		return false
	return P.number(position[0],6.0,6.0) and P.number(position[1],0.0,0.0) and P.number(position[2],2.0,2.0)

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
	var top_a: Vector2 = ground_a-Vector2(0.0,relief)
	var top_b: Vector2 = ground_b-Vector2(0.0,relief)
	var footprint = _footprint(wall.footprint)
	var face = PackedVector2Array([ground_a,ground_b,top_b,top_a])
	if not _push_polygon(packets,wall.id+':face','wall_face',face,Color(0.18,0.25,0.29,1.0),true,wall.id,footprint):
		return false
	var cap_points = _project_polygon(wall.polygon,projection,RELIEF_PIXELS)
	if cap_points.is_empty() or not _push_polygon(packets,wall.id+':cap','wall_cap',cap_points,Color(0.29,0.39,0.43,1.0),true,wall.id,footprint):
		return false
	if not _push_polygon(packets,wall.id+':cap-edge','cap_edge',cap_points,Color(0.53,0.66,0.68,0.8),false,wall.id,footprint,1.2*scale):
		return false
	var seam_count: int = int(wall.seams)
	for index in range(1,seam_count+1):
		var t = float(index)/float(seam_count+1)
		var room_point = a.lerp(b,t)
		var low: Vector2 = projection.project(Vector3(room_point.x,0.0,room_point.y))
		var high: Vector2 = low-Vector2(0.0,relief)
		var seam_points = PackedVector2Array([low,high])
		if not _push_polygon(packets,wall.id+':seam:'+str(index),'panel_seam',seam_points,Color(0.34,0.46,0.5,0.76),false,wall.id,footprint,1.0*scale):
			return false
		var fastener_center: Vector2 = low.lerp(high,0.73)
		if not _add_fastener(packets,wall.id+':fastener:'+str(index),fastener_center,1.25*scale,wall.id,footprint):
			return false
	return true

func _add_fastener(packets: Array, packet_id: String, center: Vector2, radius: float, wall_id: String, footprint: Rect2) -> bool:
	var points = PackedVector2Array()
	for index in range(6):
		var angle = TAU*float(index)/6.0
		points.append(center+Vector2(cos(angle),sin(angle))*radius)
	return _push_polygon(packets,packet_id,'fastener',points,Color(0.57,0.68,0.69,0.9),true,wall_id,footprint)

func _add_pipe_detail(packets: Array, projection) -> bool:
	var scale = projection.ppm/48.0
	var footprint = Rect2(4.8,1.35,2.4,1.1)
	var runs = [
		{'id':'pipe-upper','a':Vector2(5.0,1.52),'b':Vector2(7.0,1.52),'screen_raise':1.0},
		{'id':'pipe-lower','a':Vector2(5.0,1.9),'b':Vector2(7.0,1.9),'screen_raise':1.0},
		{'id':'cable-center','a':Vector2(5.25,2.22),'b':Vector2(6.75,2.22),'screen_raise':1.5},
		{'id':'pipe-drop-left','a':Vector2(5.25,1.52),'b':Vector2(5.25,1.9),'screen_raise':1.0},
		{'id':'pipe-drop-right','a':Vector2(6.75,1.52),'b':Vector2(6.75,1.9),'screen_raise':1.0}
	]
	for run in runs:
		var offset = Vector2(0.0,float(run.screen_raise)*scale)
		var first: Vector2 = projection.project(Vector3(run.a.x,0.0,run.a.y))-offset
		var second: Vector2 = projection.project(Vector3(run.b.x,0.0,run.b.y))-offset
		var points = PackedVector2Array([first,second])
		var color = Color(0.13,0.2,0.23,0.94) if 'pipe' in run.id else Color(0.31,0.47,0.51,0.82)
		var width = (2.3 if 'pipe' in run.id else 1.1)*scale
		if not _push_polygon(packets,'gh_power_pipe:'+run.id,'pipe_detail',points,color,false,'',footprint,width,PIPE_PROP):
			return false
	return true

func _add_contact_shadow(packets: Array, projection) -> bool:
	return _add_ground_ellipse(packets,'gh_power_console:contact','contact_shadow',Vector3(4.0,0.0,7.0),0.56,0.24,Color(0.07,0.11,0.13,0.18),projection,'power_console_off_on')

func _add_task_light(packets: Array, projection) -> bool:
	return _add_ground_ellipse(packets,'gh_power_console:task-light','powered_task_light',Vector3(4.75,0.0,6.45),0.72,0.48,Color(0.47,0.78,0.82,0.12),projection,'power_console_off_on')

func _add_off_emergency_light(packets: Array, projection) -> bool:
	return _add_ground_ellipse(packets,'gh_power_console:off-emergency-light','off_emergency_light',Vector3(4.75,0.0,6.45),0.72,0.48,Color(0.55,0.18,0.16,0.06),projection,'power_console_off_on')

func _add_ground_ellipse(packets: Array, packet_id: String, role: String, center: Vector3, radius_x: float, radius_z: float, color: Color, projection, source_id: String) -> bool:
	var points = PackedVector2Array()
	for index in range(20):
		var angle = TAU*float(index)/20.0
		var world = Vector3(center.x+cos(angle)*radius_x,0.0,center.z+sin(angle)*radius_z)
		points.append(projection.project(world))
	return _push_polygon(packets,packet_id,role,points,color,true,'',Rect2(center.x-radius_x,center.z-radius_z,radius_x*2.0,radius_z*2.0),0.0,source_id)

func _project_polygon(points: Array, projection, relief_pixels: float = 0.0) -> PackedVector2Array:
	var projected = PackedVector2Array()
	var offset = Vector2(0.0,relief_pixels*(projection.ppm/48.0))
	for point in points:
		projected.append(projection.project(Vector3(float(point[0]),0.0,float(point[1])))-offset)
	return projected

func _footprint(values: Array) -> Rect2:
	return Rect2(float(values[0]),float(values[1]),float(values[2]),float(values[3]))

func _push_polygon(packets: Array, packet_id: String, role: String, points: PackedVector2Array, color: Color, filled: bool, wall_id: String, footprint: Rect2, width: float = 0.0, source_id: String = '') -> bool:
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

func _outer_context_valid(scene_data: Dictionary, snapshot: Dictionary, current_key: String, projection) -> bool:
	if not P.snapshot(snapshot) or not snapshot.has('entryToken') or snapshot.get('entryToken')!=null or not _ensure_layout(scene_data,snapshot,current_key):
		return false
	if not projection is Object or projection.get_script()!=PROJECTION_SCRIPT:
		return false
	var initialized: Variant = projection.get('initialized')
	var origin_variant: Variant = projection.get('origin')
	var center_variant: Variant = projection.get('screen_center')
	var viewport_variant: Variant = projection.get('viewport')
	var ppm_variant: Variant = projection.get('ppm')
	if typeof(initialized)!=TYPE_BOOL or not initialized or not origin_variant is Vector3 or not center_variant is Vector2 or not viewport_variant is Vector2 or not P.number(ppm_variant) or float(ppm_variant)<=0.0 or float(ppm_variant)>256.0:
		return false
	var origin: Vector3 = origin_variant
	var center: Vector2 = center_variant
	var viewport: Vector2 = viewport_variant
	return is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and is_finite(center.x) and is_finite(center.y) and is_finite(viewport.x) and is_finite(viewport.y) and viewport.x>0.0 and viewport.y>0.0

func _build_outer_wall_packet(scene_data: Dictionary, snapshot: Dictionary, current_key: String, projection, wall: Dictionary) -> Variant:
	var wall_id: Variant = wall.get('id')
	var edge_variant: Variant = OUTER_WALL_EDGES.get(wall_id)
	if edge_variant==null:
		return null
	if not _outer_context_valid(scene_data,snapshot,current_key,projection):
		return null
	if not edge_variant is Array or edge_variant.size()!=2 or not edge_variant[0] is Vector2 or not edge_variant[1] is Vector2:
		return false
	var edge_a: Vector2 = edge_variant[0]
	var edge_b: Vector2 = edge_variant[1]
	if not is_finite(edge_a.x) or not is_finite(edge_a.y) or not is_finite(edge_b.x) or not is_finite(edge_b.y):
		return false
	var relief: float = RELIEF_PIXELS*float(projection.get('ppm'))/48.0
	var ground_a_variant: Variant = projection.call('project',Vector3(edge_a.x,0.0,edge_a.y))
	var ground_b_variant: Variant = projection.call('project',Vector3(edge_b.x,0.0,edge_b.y))
	if not ground_a_variant is Vector2 or not ground_b_variant is Vector2:
		return false
	var ground_a: Vector2 = ground_a_variant
	var ground_b: Vector2 = ground_b_variant
	var raised_a: Vector2 = ground_a-Vector2(0.0,relief)
	var raised_b: Vector2 = ground_b-Vector2(0.0,relief)
	var points: PackedVector2Array = PackedVector2Array([ground_a,ground_b,raised_b,raised_a])
	var twice_area: float = 0.0
	for index: int in range(points.size()):
		var point: Vector2 = points[index]
		if not is_finite(point.x) or not is_finite(point.y):
			return false
		var next_point: Vector2 = points[(index+1)%points.size()]
		twice_area += point.x*next_point.y-point.y*next_point.x
	if not is_finite(twice_area):
		return false
	if twice_area==0.0:
		return null
	var staged_packets: Array = []
	if not _push_polygon(staged_packets,str(wall_id)+':exterior_face','wall_face',points,Color(0.18,0.25,0.29,1.0),true,str(wall_id),_footprint(wall.footprint)) or staged_packets.size()!=1:
		return false
	return staged_packets[0]
