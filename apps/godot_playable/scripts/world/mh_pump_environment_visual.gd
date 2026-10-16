extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
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
