extends RefCounted

const STRUCTURES = [
	{"id":"north_bulkhead","x0":0.0,"x1":24.0,"z0":0.0,"z1":0.5,"kind":"bulkhead","relief":18.0},
	{"id":"south_bulkhead","x0":0.0,"x1":24.0,"z0":15.5,"z1":16.0,"kind":"bulkhead","relief":18.0},
	{"id":"west_bulkhead_north","x0":0.0,"x1":0.5,"z0":0.5,"z1":5.0,"kind":"bulkhead","relief":18.0},
	{"id":"west_bulkhead_south","x0":0.0,"x1":0.5,"z0":10.0,"z1":15.5,"kind":"bulkhead","relief":18.0},
	{"id":"east_bulkhead_north","x0":23.5,"x1":24.0,"z0":0.5,"z1":5.0,"kind":"bulkhead","relief":18.0},
	{"id":"east_bulkhead_south","x0":23.5,"x1":24.0,"z0":10.0,"z1":15.5,"kind":"bulkhead","relief":18.0},
	{"id":"service_island_north","x0":9.5,"x1":14.5,"z0":6.5,"z1":7.5,"kind":"island_north","relief":8.0},
	{"id":"service_island_south","x0":9.5,"x1":14.5,"z0":8.5,"z1":9.5,"kind":"island_south","relief":8.0}
]
const TERMINALS = [
	{"id":"mission_terminal_art","position":[8.0,0.0,4.0],"width":1.25},
	{"id":"capability_terminal_art","position":[15.0,0.0,4.0],"width":1.15},
	{"id":"storage_terminal_art","position":[15.0,0.0,11.0],"width":1.10},
	{"id":"world_gate_terminal_art","position":[20.0,0.0,6.0],"width":1.35},
	{"id":"save_rest_terminal_art","position":[8.0,0.0,11.0],"width":1.20}
]
const STRUCTURE_IDS = ["north_bulkhead","south_bulkhead","west_bulkhead_north","west_bulkhead_south","east_bulkhead_north","east_bulkhead_south","service_island_north","service_island_south"]
const TERMINAL_IDS = ["mission_terminal_art","capability_terminal_art","storage_terminal_art","world_gate_terminal_art","save_rest_terminal_art"]

var _cache_key = ""
var _cached_shapes: Array = []

func has_valid_layout(world_id: String, scene_id: String, scene_data: Dictionary) -> bool:
	return _ensure_layout(world_id,scene_id,scene_data)

func selected_structure_ids(world_id: String, scene_id: String, scene_data: Dictionary) -> Array:
	if not _ensure_layout(world_id,scene_id,scene_data):
		return []
	return STRUCTURE_IDS.duplicate()

func is_structure_id(value) -> bool:
	return typeof(value)==TYPE_STRING and value in STRUCTURE_IDS

func build_packets(scene_data: Dictionary, world_id: String, scene_id: String, projection, terminal_feet: Array, phase: String) -> Array:
	if phase not in ["back","front"] or not _ensure_layout(world_id,scene_id,scene_data):
		return []
	if not is_finite(projection.ppm) or projection.ppm<=0 or projection.ppm>256:
		return []
	var scale: float = projection.ppm/48.0
	var packets: Array = []
	if phase=="back":
		_add_ground_lighting(packets,projection,scale)
		for shape in _cached_shapes:
			_add_structure_back(packets,shape,projection,scale)
		_add_terminal_bases_and_pools(packets,projection,scale,terminal_feet)
	else:
		for shape in _cached_shapes:
			_add_structure_front(packets,shape,projection,scale)
	return packets if packets.size()<=256 else []

func front_overlap_target(packets: Array, player_rect: Rect2) -> float:
	if player_rect.size.x<=0 or player_rect.size.y<=0 or not _finite_rect(player_rect):
		return 1.0
	for packet in packets:
		if not packet is Dictionary or packet.get("role")!="front_face":
			continue
		var points = packet.get("points")
		if points is PackedVector2Array and _polygon_intersects_rect(points,player_rect):
			return 0.25
	return 1.0

func step_front_alpha(current: float, target: float, delta: float, reduced_motion: bool) -> float:
	var bounded_target: float = 0.25 if target<0.625 else 1.0
	if reduced_motion:
		return bounded_target
	var bounded_current: float = clampf(current,0.25,1.0) if is_finite(current) else 1.0
	var bounded_delta: float = clampf(delta,0.0,0.1) if is_finite(delta) else 0.0
	return clampf(move_toward(bounded_current,bounded_target,bounded_delta*3.75),0.25,1.0)

func front_fade_alphas(packets: Array, current_by_group: Dictionary, player_rect: Rect2, delta: float, reduced_motion: bool) -> Dictionary:
	var grouped: Dictionary = {}
	for packet in packets:
		if not packet is Dictionary or packet.get("role")!="front_face":
			continue
		var fade_group = packet.get("fade_group",packet.get("id"))
		if typeof(fade_group)!=TYPE_STRING or fade_group.is_empty():
			continue
		if not grouped.has(fade_group):
			grouped[fade_group] = []
		grouped[fade_group].append(packet)
	var result: Dictionary = {}
	for fade_group in grouped:
		var related_packets: Array = grouped[fade_group]
		var target: float = front_overlap_target(related_packets,player_rect)
		var current: float = float(current_by_group.get(fade_group,1.0))
		result[fade_group] = step_front_alpha(current,target,delta,reduced_motion)
	return result

func _ensure_layout(world_id: String, scene_id: String, scene_data: Dictionary) -> bool:
	if world_id!="return_station" or scene_id!="rs_core_room" or scene_data.get("worldId")!="return_station" or scene_data.get("sceneId")!="rs_core_room":
		return false
	var rows = scene_data.get("collision")
	if not rows is Array or rows.size()!=STRUCTURES.size():
		return false
	var by_id: Dictionary = {}
	for row in rows:
		if not row is Dictionary or typeof(row.get("id"))!=TYPE_STRING or not STRUCTURE_IDS.has(row.id) or by_id.has(row.id):
			return false
		by_id[row.id] = row
	var signature = "return_station:rs_core_room:layout-v1"
	for expected in STRUCTURES:
		if not by_id.has(expected.id) or not _matches_rectangle(by_id[expected.id].get("polygon"),expected):
			return false
	if _cache_key!=signature:
		_cache_key = signature
		_cached_shapes.clear()
		for expected in STRUCTURES:
			_cached_shapes.append(expected.duplicate(true))
	return _cached_shapes.size()==STRUCTURES.size()

func _matches_rectangle(value, shape: Dictionary) -> bool:
	if not value is Array or value.size()!=4:
		return false
	var expected = [Vector2(shape.x0,shape.z0),Vector2(shape.x1,shape.z0),Vector2(shape.x1,shape.z1),Vector2(shape.x0,shape.z1)]
	var seen: Array = []
	for point in value:
		if not point is Array or point.size()!=2 or not _is_number(point[0]) or not _is_number(point[1]):
			return false
		var actual = Vector2(float(point[0]),float(point[1]))
		if not is_finite(actual.x) or not is_finite(actual.y):
			return false
		var match_index = -1
		for index in range(expected.size()):
			if actual==expected[index]:
				match_index = index
				break
		if match_index<0 or match_index in seen:
			return false
		seen.append(match_index)
	return seen.size()==4

func _is_number(value) -> bool:
	return typeof(value)==TYPE_INT or typeof(value)==TYPE_FLOAT

func _add_ground_lighting(packets: Array, projection, scale: float) -> void:
	var edge_points = [Vector2(1.0,0.9),Vector2(23.0,0.9),Vector2(1.0,15.1),Vector2(23.0,15.1)]
	for index in range(edge_points.size()):
		_add_pool(packets,"rs:edge-light:"+str(index),edge_points[index],0.72,0.24,projection,scale)

func _add_terminal_bases_and_pools(packets: Array, projection, scale: float, terminal_feet: Array) -> void:
	if terminal_feet.size()!=TERMINALS.size():
		return
	var by_id: Dictionary = {}
	for foot in terminal_feet:
		if not foot is Dictionary or typeof(foot.get("id"))!=TYPE_STRING or not TERMINAL_IDS.has(foot.id) or by_id.has(foot.id) or not foot.get("position") is Vector3:
			return
		by_id[foot.id] = foot.position
	for terminal in TERMINALS:
		if not by_id.has(terminal.id):
			return
		var position: Vector3 = by_id[terminal.id]
		var authored: Array = terminal.position
		if position!=Vector3(authored[0],authored[1],authored[2]):
			return
		var half_width: float = float(terminal.width)*0.5
		var pad = _rect_world(position.x-half_width,position.x+half_width,position.z-0.22,position.z+0.22)
		var pad_points = _project_world(pad,projection)
		_append_polygon(packets,"rs:terminal-base:"+terminal.id,"terminal_base",pad_points,Color("#26373d"),true)
		_append_polygon(packets,"rs:terminal-base-rim:"+terminal.id,"terminal_base",pad_points,Color(0.73,0.76,0.73,0.34),false,0.8*scale)
		_add_pool(packets,"rs:terminal-light:"+terminal.id,Vector2(position.x,position.z),0.68,0.28,projection,scale)

func _add_pool(packets: Array, packet_id: String, center: Vector2, radius_x: float, radius_z: float, projection, scale: float) -> void:
	var alphas = [0.018,0.028,0.040]
	for index in range(alphas.size()):
		var factor: float = 1.0-float(index)*0.22
		var points: Array = []
		for segment in range(16):
			var angle: float = TAU*float(segment)/16.0
			points.append(Vector2(center.x+cos(angle)*radius_x*factor,center.y+sin(angle)*radius_z*factor))
		var screen = _project_world(points,projection)
		_append_polygon(packets,packet_id+":"+str(index),"ground_light",screen,Color(0.96,0.90,0.78,alphas[index]),true)

func _add_structure_back(packets: Array, shape: Dictionary, projection, scale: float) -> void:
	var full = _rect_world(shape.x0,shape.x1,shape.z0,shape.z1)
	var base = _project_world(full,projection)
	if base.size()!=4:
		return
	var relief: float = minf(float(shape.relief),24.0 if shape.kind=="bulkhead" else 12.0)*scale
	var relief_offset = Vector2(0,-relief)
	var raised_base = _offset_points(base,relief_offset)
	var shadow = _offset_points(base,Vector2(0,2.5*scale))
	_append_polygon(packets,"rs:shadow:"+shape.id,"ground_shadow",shadow,Color(0.025,0.035,0.040,0.30),true)
	var surface_color: Color = Color("#35474d") if shape.kind=="bulkhead" else Color("#46565a")
	_append_polygon(packets,"rs:top:"+shape.id,"structure_top",raised_base,surface_color,true)
	_append_polygon(packets,"rs:top-edge:"+shape.id,"structure_cap",raised_base,Color(0.88,0.86,0.78,0.38),false,maxf(0.8,1.0*scale))
	var inset_x: float = 0.035 if shape.kind=="bulkhead" else 0.14
	var inset_z: float = 0.035 if shape.kind=="bulkhead" else 0.10
	var inset = _offset_points(_project_world(_rect_world(shape.x0+inset_x,shape.x1-inset_x,shape.z0+inset_z,shape.z1-inset_z),projection),relief_offset)
	var inset_color: Color = Color("#3d5055") if shape.kind=="bulkhead" else Color("#4c5c5d")
	_append_polygon(packets,"rs:inset:"+shape.id,"structure_recess",inset,inset_color,true)
	var top_edge = _offset_points(_project_world([Vector2(shape.x0+inset_x,shape.z0+inset_z),Vector2(shape.x1-inset_x,shape.z0+inset_z)],projection),relief_offset)
	_append_polygon(packets,"rs:key-edge:"+shape.id,"structure_cap",top_edge,Color(0.95,0.88,0.73,0.42),false,maxf(0.75,0.9*scale))
	if shape.kind=="bulkhead":
		_add_bulkhead_seams(packets,shape,projection,scale,relief_offset)
	else:
		_add_island_recess(packets,shape,projection,scale,relief_offset)

func _add_bulkhead_seams(packets: Array, shape: Dictionary, projection, scale: float, relief_offset: Vector2) -> void:
	var width: float = shape.x1-shape.x0
	var depth: float = shape.z1-shape.z0
	if width>depth:
		for fraction in [0.25,0.5,0.75]:
			var x: float = lerpf(shape.x0,shape.x1,fraction)
			var points = _offset_points(_project_world([Vector2(x,shape.z0+0.06),Vector2(x,shape.z1-0.06)],projection),relief_offset)
			_append_polygon(packets,"rs:seam:"+shape.id+":"+str(fraction),"structure_seam",points,Color(0.09,0.13,0.15,0.48),false,maxf(0.8,1.0*scale))
	else:
		var z: float = lerpf(shape.z0,shape.z1,0.5)
		var points = _offset_points(_project_world([Vector2(shape.x0+0.06,z),Vector2(shape.x1-0.06,z)],projection),relief_offset)
		_append_polygon(packets,"rs:seam:"+shape.id,"structure_seam",points,Color(0.09,0.13,0.15,0.48),false,maxf(0.8,1.0*scale))

func _add_island_recess(packets: Array, shape: Dictionary, projection, scale: float, relief_offset: Vector2) -> void:
	var x0: float = shape.x0+0.42
	var x1: float = shape.x1-0.42
	var z0: float = shape.z0+0.18
	var z1: float = shape.z1-0.18
	var panel = _offset_points(_project_world(_rect_world(x0,x1,z0,z1),projection),relief_offset)
	var panel_color: Color = Color("#35494d") if shape.kind=="island_north" else Color("#3a4849")
	_append_polygon(packets,"rs:panel:"+shape.id,"structure_recess",panel,panel_color,true)
	_append_polygon(packets,"rs:panel-rim:"+shape.id,"structure_cap",panel,Color(0.79,0.82,0.77,0.30),false,maxf(0.75,0.9*scale))
	if shape.kind=="island_north":
		var trim = _offset_points(_project_world([Vector2(x0,z0+0.035),Vector2(x1,z0+0.035)],projection),relief_offset)
		_append_polygon(packets,"rs:static-info-trim:"+shape.id,"static_trim",trim,Color(0.35,0.84,0.91,0.34),false,maxf(0.8,1.0*scale))
		var channel = _offset_points(_project_world([Vector2(x0+0.28,(z0+z1)*0.5),Vector2(x1-0.28,(z0+z1)*0.5)],projection),relief_offset)
		_append_polygon(packets,"rs:service-channel:"+shape.id,"structure_seam",channel,Color(0.08,0.13,0.15,0.56),false,maxf(1.0,1.2*scale))
	else:
		var channel_x: float = (x0+x1)*0.5
		for offset in [-0.12,0.12]:
			var channel = _offset_points(_project_world([Vector2(channel_x+offset,z0+0.03),Vector2(channel_x+offset,z1-0.03)],projection),relief_offset)
			_append_polygon(packets,"rs:service-channel:"+shape.id+":"+str(offset),"structure_seam",channel,Color(0.08,0.13,0.15,0.52),false,maxf(0.8,1.0*scale))

func _add_structure_front(packets: Array, shape: Dictionary, projection, scale: float) -> void:
	var world = _rect_world(shape.x0,shape.x1,shape.z0,shape.z1)
	var base = _project_world(world,projection)
	if base.size()!=4:
		return
	var front_index: int = 0
	for index in range(4):
		if world[index].x+world[index].y>world[front_index].x+world[front_index].y:
			front_index = index
	var neighbors = [posmod(front_index-1,4),posmod(front_index+1,4)]
	var relief: float = minf(float(shape.relief),24.0 if shape.kind=="bulkhead" else 12.0)*scale
	var relief_offset = Vector2(0,-relief)
	var raised_base = _offset_points(base,relief_offset)
	for edge_index in range(neighbors.size()):
		var neighbor: int = neighbors[edge_index]
		var face = PackedVector2Array([raised_base[front_index],raised_base[neighbor],base[neighbor],base[front_index]])
		var suffix: String = str(edge_index)
		var fade_group: String = "rs:front-fade:"+shape.id+":"+suffix
		var face_color: Color = Color("#26343a") if shape.kind=="bulkhead" else Color("#303d40")
		_append_polygon(packets,"rs:front:"+shape.id+":"+suffix,"front_face",face,face_color,true,1.0,fade_group)
		var lower = PackedVector2Array([base[neighbor],base[front_index]])
		_append_polygon(packets,"rs:fascia:"+shape.id+":"+suffix,"front_face",lower,Color(0.87,0.84,0.75,0.30),false,maxf(0.75,0.9*scale),fade_group)

func _rect_world(x0: float, x1: float, z0: float, z1: float) -> Array:
	return [Vector2(x0,z0),Vector2(x1,z0),Vector2(x1,z1),Vector2(x0,z1)]

func _project_world(points: Array, projection) -> PackedVector2Array:
	var result = PackedVector2Array()
	for point in points:
		if not point is Vector2 or not is_finite(point.x) or not is_finite(point.y):
			return PackedVector2Array()
		var projected: Vector2 = projection.project(Vector3(point.x,0,point.y))
		if not is_finite(projected.x) or not is_finite(projected.y):
			return PackedVector2Array()
		result.append(projected)
	return result

func _offset_points(points: PackedVector2Array, offset: Vector2) -> PackedVector2Array:
	var result = PackedVector2Array()
	for point in points:
		result.append(point+offset)
	return result

func _append_polygon(packets: Array, packet_id: String, role: String, points: PackedVector2Array, color: Color, filled: bool, width: float = 1.0, fade_group: String = "") -> void:
	if points.size()<(3 if filled else 2) or points.size()>32 or not _finite_points(points) or not is_finite(color.r) or not is_finite(color.g) or not is_finite(color.b) or not is_finite(color.a) or not is_finite(width):
		return
	var packet: Dictionary = {"id":packet_id,"role":role,"kind":"polygon","points":points,"color":color,"filled":filled,"width":width}
	if not fade_group.is_empty():
		packet["fade_group"] = fade_group
	packets.append(packet)

func _finite_points(points: PackedVector2Array) -> bool:
	for point in points:
		if not is_finite(point.x) or not is_finite(point.y):
			return false
	return true

func _finite_rect(rect: Rect2) -> bool:
	return is_finite(rect.position.x) and is_finite(rect.position.y) and is_finite(rect.size.x) and is_finite(rect.size.y)

func _polygon_intersects_rect(points: PackedVector2Array, rect: Rect2) -> bool:
	if points.size()<3 or not _finite_points(points):
		return false
	var bounds = Rect2(points[0],Vector2.ZERO)
	for point in points:
		bounds = bounds.expand(point)
	if not bounds.intersects(rect):
		return false
	for point in points:
		if rect.has_point(point):
			return true
	var corners = [rect.position,Vector2(rect.end.x,rect.position.y),rect.end,Vector2(rect.position.x,rect.end.y)]
	for corner in corners:
		if Geometry2D.is_point_in_polygon(corner,points):
			return true
	for index in range(points.size()):
		var a: Vector2 = points[index]
		var b: Vector2 = points[(index+1)%points.size()]
		for edge in range(4):
			var c: Vector2 = corners[edge]
			var d: Vector2 = corners[(edge+1)%4]
			if _segments_intersect(a,b,c,d):
				return true
	return false

func _segments_intersect(a: Vector2, b: Vector2, c: Vector2, d: Vector2) -> bool:
	var first: Vector2 = b-a
	var second: Vector2 = d-c
	var denominator: float = first.cross(second)
	if absf(denominator)<0.000001:
		return false
	var delta: Vector2 = c-a
	var t: float = delta.cross(second)/denominator
	var u: float = delta.cross(first)/denominator
	return t>=0.0 and t<=1.0 and u>=0.0 and u<=1.0
