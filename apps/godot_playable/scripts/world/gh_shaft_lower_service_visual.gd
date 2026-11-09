extends RefCounted

# Candidate planner only. A future caller must bind this plan to an exact
# native selected-recipe/epoch proof before installing any frame.
const SHA := "6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832"
const DATA := "res://data/gh_shaft_lower_service_v2.local.json"
const ATLAS := "res://assets/props/grey_hive/lower_service_fixtures_local_r1.png"
const SHAFT_VISUAL := preload("res://scripts/world/gh_shaft_lift_visual.gd")
const RETAINED_PROPS := preload("res://scripts/world/gh_exit_source_owned_visual.gd")
const RETAINED_LEGACY := preload("res://scripts/world/gh_exit_environment_visual.gd")
const EXPECTED_WALLS := [{"id":"connector_north_a","rect_xz":[19.75,5,23.25,5.25],"band":[-4,-1.6]},{"id":"connector_north_turn","rect_xz":[23.25,5.25,23.5,6.5],"band":[-4,-1.6]},{"id":"connector_north_b","rect_xz":[24,6.25,26,6.5],"band":[-4,-1.6]},{"id":"connector_south_a","rect_xz":[19.75,9.25,23,9.5],"band":[-4,-1.6]},{"id":"connector_south_b","rect_xz":[23,9.5,26,9.75],"band":[-4,-1.6]},{"id":"sector_north","rect_xz":[26,1,50,1.25],"band":[-4,-1.6]},{"id":"sector_south","rect_xz":[26,20.75,50,21],"band":[-4,-1.6]},{"id":"sector_east","rect_xz":[49.75,1.25,50,20.75],"band":[-4,-1.6]},{"id":"sector_west_north","rect_xz":[26,1.25,26.25,6.5],"band":[-4,-1.6]},{"id":"sector_west_south","rect_xz":[26,9.5,26.25,20.75],"band":[-4,-1.6]},{"id":"north_bays_front_0","rect_xz":[26.25,6.75,29,7],"band":[-4,-1.6]},{"id":"north_bays_front_1","rect_xz":[31,6.75,37,7],"band":[-4,-1.6]},{"id":"north_bays_front_2","rect_xz":[39,6.75,45,7],"band":[-4,-1.6]},{"id":"north_bays_front_last","rect_xz":[47,6.75,49.75,7],"band":[-4,-1.6]},{"id":"north_divider_34","rect_xz":[33.875,1.25,34.125,6.75],"band":[-4,-1.6]},{"id":"north_divider_42","rect_xz":[41.875,1.25,42.125,6.75],"band":[-4,-1.6]},{"id":"middle_bays_front_0","rect_xz":[29,10,31,10.25],"band":[-4,-1.6]},{"id":"middle_bays_front_1","rect_xz":[33,10,37,10.25],"band":[-4,-1.6]},{"id":"middle_bays_front_2","rect_xz":[39,10,43,10.25],"band":[-4,-1.6]},{"id":"middle_bays_front_last","rect_xz":[45,10,47,10.25],"band":[-4,-1.6]},{"id":"middle_bays_rear_0","rect_xz":[29,13.75,31,14],"band":[-4,-1.6]},{"id":"middle_bays_rear_1","rect_xz":[33,13.75,37,14],"band":[-4,-1.6]},{"id":"middle_bays_rear_2","rect_xz":[39,13.75,43,14],"band":[-4,-1.6]},{"id":"middle_bays_rear_last","rect_xz":[45,13.75,47,14],"band":[-4,-1.6]},{"id":"middle_divider_35","rect_xz":[34.875,10.25,35.125,13.75],"band":[-4,-1.6]},{"id":"middle_divider_41","rect_xz":[40.875,10.25,41.125,13.75],"band":[-4,-1.6]},{"id":"middle_west","rect_xz":[28.75,10,29,14],"band":[-4,-1.6]},{"id":"middle_east","rect_xz":[47,10,47.25,14],"band":[-4,-1.6]},{"id":"south_bays_front_0","rect_xz":[26.25,17,29,17.25],"band":[-4,-1.6]},{"id":"south_bays_front_1","rect_xz":[31,17,37,17.25],"band":[-4,-1.6]},{"id":"south_bays_front_2","rect_xz":[39,17,45,17.25],"band":[-4,-1.6]},{"id":"south_bays_front_last","rect_xz":[47,17,49.75,17.25],"band":[-4,-1.6]},{"id":"south_divider_34","rect_xz":[33.875,17.25,34.125,20.75],"band":[-4,-1.6]},{"id":"south_divider_42","rect_xz":[41.875,17.25,42.125,20.75],"band":[-4,-1.6]}]
const SERVICE_REGIONS := {
 "pipe_fittings":[[26.25,1.25,33.875,6.75],[29,6.75,31,7]],
 "cable_sorting":[[34.125,1.25,41.875,6.75],[37,6.75,39,7]],
 "filter_storage":[[42.125,1.25,49.75,6.75],[45,6.75,47,7]],
 "seal_assembly":[[29,10.25,34.875,13.75],[31,10,33,10.25],[31,13.75,33,14]],
 "tool_check":[[35.125,10.25,40.875,13.75],[37,10,39,10.25],[37,13.75,39,14]],
 "equipment_service":[[41.125,10.25,47,13.75],[43,10,45,10.25],[43,13.75,45,14]],
 "turnover_storage":[[26.25,17.25,33.875,20.75],[29,17,31,17.25]],
 "records_sorting":[[34.125,17.25,41.875,20.75],[37,17,39,17.25]],
 "shift_recess":[[42.125,17.25,49.75,20.75],[45,17,47,17.25]],
 "distribution_spine":[[26.25,7,49.75,10]],"west_return_link":[[26.25,10,28.75,14]],
 "east_return_link":[[47.25,10,49.75,14]],"rear_return_spine":[[26.25,14,49.75,17]],
 "connector":[[19.5,5.25,19.75,7.75],[19.75,5.25,23.25,9.25],[23.25,6.5,23.5,9.25],[23.5,6.5,26.25,9.5],[23,9.25,23.5,9.5]]}
const PRESENTATION := preload("res://scripts/world/presentation.gd")
const PROTOCOL := preload("res://scripts/bridge/protocol.gd")
var _attempted := false
var _data: Dictionary = {}
var _atlas: Texture2D
var _base = SHAFT_VISUAL.new()
var _retained = RETAINED_PROPS.new()
var _retained_legacy = RETAINED_LEGACY.new()
var _unit_ground: Array = []
var _unit_depth: Array = []
var _projection_valid := false
var _projection_zero := Vector2.ZERO
var _projection_ppm := 0.0
var _projection_ground: Array = []
var _projection_depth: Array = []

func _load_candidate() -> bool:
	if not _attempted:
		_attempted = true
		var file := FileAccess.open(DATA, FileAccess.READ)
		if file == null: return false
		var parsed = JSON.parse_string(file.get_as_text())
		if not parsed is Dictionary: return false
		_data = parsed
		var image := Image.new()
		if image.load(ATLAS) != OK or image.get_size() != Vector2i(1536,1024) or image.get_format() != Image.FORMAT_RGBA8: return false
		_atlas = ImageTexture.create_from_image(image)
	if _atlas == null or not _base._load_once(): return false
	if _data.get("schema") != "gh-lower-service/v2" or _data.get("status") != "candidate-not-admitted" or _data.get("scene") != "gh_central_shaft" or _data.get("source_sha") != SHA: return false
	if _data.get("floor_y") != -4 or not _same_definition(_data.get("floor_union_rects"),[[1.5,5,19.5,14],[19.5,5.25,23.5,9.25],[23,6.5,26.25,9.5],[26,1,50,21]]): return false
	if not _same_definition(_data.get("old_owner_cut"),[19.5,5.25,19.75,7.75]) or not _same_definition(_data.get("retained_spans"),[[19.5,4.75,19.75,5.25],[19.5,7.75,19.75,14.25]]): return false
	if not _same_definition(_data.get("new_walls"),EXPECTED_WALLS): return false
	var old_walls: Array = _base._data.get("walls", [])
	var matching: Array = old_walls.filter(func(w): return w is Dictionary and w.get("id") == "lower_east")
	return matching.size() == 1 and _same_definition(matching[0].get("polygon"),[[19.5,4.75],[19.75,4.75],[19.75,14.25],[19.5,14.25]]) and _same_definition(matching[0].get("heightBandM"),[-4,-1.6])

# JSON represents authored numbers as floats; compare numeric values without
# weakening exact array structure, keys or geometry.
func _same_definition(a, b) -> bool:
	if (typeof(a)==TYPE_INT or typeof(a)==TYPE_FLOAT) and (typeof(b)==TYPE_INT or typeof(b)==TYPE_FLOAT):
		return is_finite(float(a)) and is_finite(float(b)) and float(a)==float(b)
	if a is Array and b is Array:
		if a.size()!=b.size(): return false
		for index in range(a.size()):
			if not _same_definition(a[index],b[index]): return false
		return true
	if a is Dictionary and b is Dictionary:
		if a.size()!=b.size(): return false
		for key in a:
			if not b.has(key) or not _same_definition(a[key],b[key]): return false
		return true
	return typeof(a)==typeof(b) and a==b
func _unit_project(v: Vector3) -> Vector2:
	return Vector2(v.x-v.z,(v.x+v.z)*0.5-v.y)
func _reproject(source: Array, offset: Vector2, ppm: float) -> Array:
	var result: Array=[]
	for original in source:
		var piece: Dictionary=original.duplicate()
		var points:=PackedVector2Array()
		for point in original.points: points.append(offset+point*ppm)
		piece.points=points
		result.append(piece)
	return result
func _rect4(a: Array) -> Rect2:
	return Rect2(Vector2(float(a[0]),float(a[1])),Vector2(float(a[2])-float(a[0]),float(a[3])-float(a[1])))

func region_plan(snapshot: Dictionary, scene: Dictionary, scene_hash: String) -> Dictionary:
	if not _load_candidate() or not PROTOCOL.snapshot(snapshot) or snapshot.get("shaftLowerService") is not Dictionary: return {}
	var identity: Dictionary = snapshot.shaftLowerService
	if identity.size()!=2 or identity.get("recipeVersion")!=2 or identity.get("sceneSourceSha256")!=SHA or scene_hash!=SHA: return {}
	var view: Dictionary=PRESENTATION.shaft_lift_view(snapshot.get("shaftLift"),snapshot,scene,scene_hash)
	if view.is_empty() or view.rider.mode!="floor" or float(view.rider.positionM.yM)!=-4.0: return {}
	var visible=snapshot.get("currentVisibleRegions")
	if not visible is Array or visible.size()!=1 or typeof(visible[0])!=TYPE_STRING or snapshot.get("denseOccupiedRegion")!=visible[0]: return {}
	var prefix := "grey_hive.gh_central_shaft.lower_service_v2."
	if not visible[0].begins_with(prefix): return {}
	var key: String=visible[0].trim_prefix(prefix)
	if not SERVICE_REGIONS.has(key): return {}
	var parts: Array=[]
	for values in SERVICE_REGIONS[key]: parts.append(_rect4(values))
	var p: Vector3=PROTOCOL.vector(snapshot.player.transform.positionM)
	var contains := false
	for r: Rect2 in parts:
		if p.x>=r.position.x and p.x<=r.end.x and p.z>=r.position.y and p.z<=r.end.y: contains=true
	return {"region":visible[0],"parts":parts,"floor_y":-4.0} if contains else {}
# This is an image-composition check, never a physical footprint. Clip each
# projected lane quadrilateral against the full upright cell rectangle.
func _clip_edge(poly: Array, axis: int, bound: float, greater: bool) -> Array:
	if poly.is_empty(): return []
	var result: Array=[]
	var previous: Vector2=poly.back()
	var prev_value: float=previous[axis]
	var prev_inside: bool=prev_value>=bound if greater else prev_value<=bound
	for point: Vector2 in poly:
		var value: float=point[axis]
		var inside: bool=value>=bound if greater else value<=bound
		if inside!=prev_inside:
			var denominator: float=value-prev_value
			if denominator==0.0: return []
			result.append(previous.lerp(point,(bound-prev_value)/denominator))
		if inside: result.append(point)
		previous=point;prev_value=value;prev_inside=inside
	return result
func _positive_overlap(poly: Array, dest: Rect2) -> bool:
	var clipped: Array=_clip_edge(poly,0,dest.position.x,true)
	clipped=_clip_edge(clipped,0,dest.end.x,false)
	clipped=_clip_edge(clipped,1,dest.position.y,true)
	clipped=_clip_edge(clipped,1,dest.end.y,false)
	if clipped.size()<3: return false
	var twice_area:=0.0
	for index in range(clipped.size()):
		var a: Vector2=clipped[index]
		var b: Vector2=clipped[(index+1)%clipped.size()]
		twice_area+=a.x*b.y-b.x*a.y
	return absf(twice_area)>0.000001
func _clear_lanes(dest: Rect2, project: Callable) -> bool:
	for r in [[1.5,1,5.5,5],[1.5,5,5.5,5.15],[19.5,5.25,23.5,9.25],[23,6.5,26.25,9.5],
		[26.25,7,49.75,10],[26.25,10,28.75,17],[47.25,10,49.75,17],[26.25,14,49.75,17],
		[29,6.75,31,7],[37,6.75,39,7],[45,6.75,47,7],
		[31,10,33,10.25],[37,10,39,10.25],[43,10,45,10.25],
		[31,13.75,33,14],[37,13.75,39,14],[43,13.75,45,14],
		[29,17,31,17.25],[37,17,39,17.25],[45,17,47,17.25]]:
		var quad: Array=[]
		var bounds := Rect2()
		for p in [Vector3(r[0],-4,r[1]),Vector3(r[2],-4,r[1]),Vector3(r[2],-4,r[3]),Vector3(r[0],-4,r[3])]:
			var point=project.call(p)
			if not point is Vector2 or not point.is_finite(): return false
			if quad.is_empty(): bounds=Rect2(point,Vector2.ZERO)
			else: bounds=bounds.expand(point)
			quad.append(point)
		if dest.intersects(bounds,true) and _positive_overlap(quad,dest): return false
	return true
# Returns an all-or-nothing, undrawn packet. identity is candidate metadata,
# not evidence that a client is allowed to install this plan.
func prepare_candidate(project: Callable, identity: Dictionary, scene: Dictionary, visible_regions: Array, ppm: float) -> Dictionary:
	if not project.is_valid() or identity.get("recipeVersion") != 2 or identity.get("sceneSourceSha256") != SHA or not _load_candidate(): return {}
	if not is_finite(ppm) or ppm <= 0.0 or scene.get("worldId") != "grey_hive" or scene.get("sceneId") != "gh_central_shaft": return {}
	var collision = scene.get("collision")
	if not collision is Array or collision.size() != 8: return {}
	for item in [
		["shaft_wall_east_north", [[23.5,0.5],[24.0,0.5],[24.0,6.5],[23.5,6.5]]],
		["shaft_wall_east_south", [[23.5,9.5],[24.0,9.5],[24.0,15.5],[23.5,15.5]]]]:
		var matches: Array = collision.filter(func(w): return w is Dictionary and w.get("id") == item[0] and _same_definition(w.get("polygon"),item[1]))
		if matches.size() != 1: return {}
	# Build immutable unit-space geometry once. Only the affine camera projection
	# is reapplied to its bounded packet on a later frame.
	if _unit_ground.is_empty() or _unit_depth.is_empty():
		var pieces: Array = []
		for r in [[19.5,5.25,23,9.25],[23,5.25,23.5,9.5],[23.5,6.5,26,9.5],[26,1,50,21]]:
			if not _base._floor_rect(pieces,Callable(self,'_unit_project'),_rect4(r),-4.0,{"sourceId":"lower_service_floor","ownerRegions":["lower_service"],"floor":"lower","heightBand":Vector2(-4,-4),"role":"ground"},Vector2(0,-1)): return {}
		var wall_rows: Array = EXPECTED_WALLS.duplicate(true)
		wall_rows.append_array([
			{"id":"lower_east_retained_north","rect_xz":[19.5,4.75,19.75,5.25]},
			{"id":"lower_east_retained_south","rect_xz":[19.5,7.75,19.75,14.25]},
			{"id":"sector_aperture_lower_north","rect_xz":[23.5,0.5,24,6.5]},
			{"id":"sector_aperture_lower_south","rect_xz":[23.5,9.5,24,15.5]}])
		for wall in wall_rows:
			if not _base._wall(pieces,Callable(self,'_unit_project'),_rect4(wall.rect_xz),-4.0,-1.6,{"sourceId":wall.id,"ownerRegions":["lower_service"],"role":"service_wall"}): return {}
		var next_ground: Array=[]
		var next_depth: Array=[]
		for piece in pieces:
			if piece.role=="ground": next_ground.append(piece)
			else: next_depth.append(piece)
		if next_ground.is_empty() or next_depth.is_empty(): return {}
		_unit_ground=next_ground
		_unit_depth=next_depth
		# Assign fresh arrays: previously returned plans remain immutable.
		_projection_valid=false
		_projection_ground=[]
		_projection_depth=[]
	var zero=project.call(Vector3.ZERO)
	if not zero is Vector2 or not zero.is_finite(): return {}
	# Reject a nonmatching projection rather than silently moving geometry.
	var x_axis=project.call(Vector3.RIGHT)
	var z_axis=project.call(Vector3.BACK)
	var y_axis=project.call(Vector3.UP)
	if not x_axis is Vector2 or not z_axis is Vector2 or not y_axis is Vector2: return {}
	if not x_axis.is_equal_approx(zero+Vector2(ppm,ppm*0.5)) or not z_axis.is_equal_approx(zero+Vector2(-ppm,ppm*0.5)) or not y_axis.is_equal_approx(zero+Vector2(0,-ppm)): return {}
	if not _projection_valid or _projection_zero!=zero or _projection_ppm!=ppm:
		var next_ground: Array=_reproject(_unit_ground,zero,ppm)
		var next_depth: Array=_reproject(_unit_depth,zero,ppm)
		_projection_ground=next_ground
		_projection_depth=next_depth
		_projection_zero=zero
		_projection_ppm=ppm
		_projection_valid=true
	var ground: Array=_projection_ground
	var depth: Array=_projection_depth
	# Descriptors remain undrawn: the eventual consumer must establish native
	# region permission before submitting them.
	var fixtures: Array = []
	for item in [
		["cable_sorting",0,Vector3(38,-4,2.5),1.4], ["filter_storage",1,Vector3(46,-4,2.5),1.4],
		["seal_assembly",2,Vector3(30.3,-4,12),0.85], ["tool_check",3,Vector3(36.2,-4,12.5),1.35],
		["tool_check",2,Vector3(39.8,-4,12),0.85], ["equipment_service",2,Vector3(42.3,-4,12),0.85],
		["records_sorting",4,Vector3(38,-4,19.5),1.25], ["shift_recess",5,Vector3(46,-4,19.5),0.85]]:
		var region: String = "grey_hive.gh_central_shaft.lower_service_v2." + item[0]
		if region not in visible_regions: continue
		var anchor = project.call(item[2])
		if not anchor is Vector2 or not anchor.is_finite(): return {}
		var scale: float = float(item[3]) * ppm / (0.8 * 512.0)
		var source := Rect2((int(item[1]) % 3) * 512, int(item[1] / 3) * 512,512,512)
		var dest := Rect2(anchor - Vector2(256.0,499.2) * scale,Vector2(512,512) * scale)
		if not _clear_lanes(dest,project): continue
		fixtures.append({"source":source,"destination":dest,"worldAnchor":item[2],"region":region})
	# Existing qualified isolated pipe and crate sources remain optional: if
	# their exact component binding is unavailable, omit without substitution.
	if _retained._load_props(_retained_legacy):
		for item in [
			['pipe_fittings','pipe',Vector3(30,-4,2.5)], ['equipment_service','pipe',Vector3(45.8,-4,11.2)],
			['pipe_fittings','crate',Vector3(32,-4,3.5)], ['turnover_storage','crate',Vector3(28,-4,19)], ['turnover_storage','crate',Vector3(32,-4,19.5)]]:
			var region: String='grey_hive.gh_central_shaft.lower_service_v2.'+item[0]
			if region not in visible_regions: continue
			var position: Vector3=item[2]
			var pipe: bool=item[1]=='pipe'
			var texture: Texture2D=_retained.pipe if pipe else _retained.clutter
			var a:=Vector2(50,126) if pipe else Vector2(10,213)
			var b:=Vector2(325,263) if pipe else Vector2(329,182)
			var wa: Vector3=position+Vector3(-0.5,0,0) if pipe else position+Vector3(-0.375,0,0.5)
			var wb: Vector3=position+Vector3(0.5,0,0) if pipe else position+Vector3(0.375,0,-0.5)
			var packet: Dictionary=_retained._prop(project,_retained_legacy,texture,'lower_service_'+str(fixtures.size()),a,b,wa,wb,position,Vector2(0.6,0.16) if pipe else Vector2(0.5,0.55),0.18 if pipe else 0.16,0.025 if pipe else 0.06,80.0 if pipe else 96.0,ppm)
			if packet.is_empty(): continue
			var bounds: Rect2=_retained_legacy._transformed_bounds(packet.origin,packet.rotation,packet.scale,Rect2(Vector2.ZERO,texture.get_size()))
			if not _clear_lanes(bounds,project): continue
			fixtures.append({'kind':'retained','packet':packet,'worldAnchor':position,'region':region})
	return {"ground":ground,"depth":depth,"fixtures":fixtures,"atlas":_atlas,"selection":{"floor":"lower"},"suppressOldOwner":"lower_east","identity":identity.duplicate(true)}