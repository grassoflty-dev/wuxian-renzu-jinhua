extends RefCounted

const V = preload("res://scripts/world/presentation.gd")
const DATA = "res://data/gh_central_shaft_lift.local.json"
const FLOOR = "res://assets/props/grey_hive/floor_tiles.png"
const STEEL = "res://assets/materials/cw_pressure_end_steel_local_r1.png"
const UPPER_WALLS = {
	"shaft_wall_north":Rect2(0,0,24,0.5), "shaft_wall_south":Rect2(0,15.5,24,0.5),
	"shaft_wall_west_north":Rect2(0,0.5,0.5,6), "shaft_wall_west_south":Rect2(0,9.5,0.5,6),
	"shaft_wall_east_north":Rect2(23.5,0.5,0.5,6), "shaft_wall_east_south":Rect2(23.5,9.5,0.5,6),
	"shaft_support_column_a":Rect2(8,5,1,2), "shaft_support_column_b":Rect2(15,9,1,2),
}
var _attempted := false
var _data: Dictionary = {}
var _floor: Texture2D
var _steel: Texture2D

func _texture(path: String) -> Texture2D:
	var t: Texture2D = null
	if ResourceLoader.exists(path):
		t = load(path) as Texture2D
	if t == null:
		var img := Image.new()
		if img.load(path) == OK:
			t = ImageTexture.create_from_image(img)
	return t

func _number(value: Variant) -> bool:
	return (value is int or value is float) and is_finite(float(value))

func _band(value: Variant) -> bool:
	return value is Array and value.size() == 2 and _number(value[0]) and _number(value[1]) and float(value[0]) < float(value[1])

func _load_once() -> bool:
	if not _attempted:
		_attempted = true
		var f := FileAccess.open(DATA, FileAccess.READ)
		if f == null: return false
		var parsed = JSON.parse_string(f.get_as_text())
		if not parsed is Dictionary: return false
		_data = parsed
		_floor = _texture(FLOOR)
		_steel = _texture(STEEL)
	if _floor == null or _steel == null: return false
	if _floor.get_size() != Vector2(1434, 980) or _steel.get_size() != Vector2(1254, 1254): return false
	var mat = _data.get("materials")
	return _data.get("schema") == "gh-shaft-lift/v1" and _data.get("worldId") == "grey_hive" and _data.get("sceneId") == "gh_central_shaft" and mat is Dictionary and mat.get("floor") == FLOOR and mat.get("steel") == STEEL and _number(mat.get("tileM")) and float(mat.tileM) == 0.5

func _rect(poly) -> Rect2:
	if not poly is Array or poly.size() != 4: return Rect2()
	var points := PackedVector2Array()
	for p in poly:
		if not p is Array or p.size() != 2: return Rect2()
		if not (p[0] is int or p[0] is float) or not (p[1] is int or p[1] is float): return Rect2()
		var v := Vector2(float(p[0]), float(p[1]))
		if not v.is_finite(): return Rect2()
		points.append(v)
	var r := Rect2(points[0], Vector2.ZERO)
	for p in points: r = r.expand(p)
	if r.size.x <= 0 or r.size.y <= 0 or r.position.x < 0 or r.position.y < 0 or r.end.x > 24 or r.end.y > 16: return Rect2()
	var expected := [r.position, Vector2(r.end.x, r.position.y), r.end, Vector2(r.position.x, r.end.y)]
	for i in range(4):
		if not points[i].is_equal_approx(expected[i]): return Rect2()
	return r

func _piece(plan: Array, project: Callable, world: Array, uv: PackedVector2Array, texture: Texture2D, meta: Dictionary) -> bool:
	var points := PackedVector2Array()
	for w in world:
		var p = project.call(w)
		if not p is Vector2 or not p.is_finite(): return false
		points.append(p)
	var piece := meta.duplicate(true)
	piece.merge({"points": points, "uv": uv, "texture": texture, "worldAnchor": world[0]})
	plan.append(piece)
	return true

func _floor_uv(x: float, z: float) -> Vector2:
	var a := Vector2(187.5, 184.5)
	var b := Vector2(345.5, 305.5)
	var d := Vector2(27.5, 305.5)
	return (a + (b-a)*x + (d-a)*z) / Vector2(1434, 980)

func _floor_rect(plan: Array, project: Callable, r: Rect2, y: float, meta: Dictionary, phase: Vector2 = Vector2.ZERO) -> bool:
	var z := r.position.y
	while z < r.end.y - 0.00001:
		var z1 := minf(r.end.y, (floor((z+phase.y) / 2.0) + 1.0) * 2.0-phase.y)
		if z1 <= z: return false
		var x := r.position.x
		while x < r.end.x - 0.00001:
			var x1 := minf(r.end.x, (floor((x+phase.x) / 2.0) + 1.0) * 2.0-phase.x)
			if x1 <= x: return false
			var u: float = (x+phase.x - floor((x+phase.x) / 2.0)*2.0)/2.0
			var v: float = (z+phase.y - floor((z+phase.y) / 2.0)*2.0)/2.0
			var u1: float = u + (x1-x)/2.0
			var v1: float = v + (z1-z)/2.0
			if not _piece(plan, project, [Vector3(x,y,z), Vector3(x1,y,z), Vector3(x1,y,z1), Vector3(x,y,z1)], PackedVector2Array([_floor_uv(u,v),_floor_uv(u1,v),_floor_uv(u1,v1),_floor_uv(u,v1)]), _floor, meta): return false
			x = x1
		z = z1
	return true

func _wall(plan: Array, project: Callable, r: Rect2, low: float, high: float, meta: Dictionary) -> bool:
	if not is_finite(low) or not is_finite(high) or low < -4.01 or high > 3.51 or high <= low: return false
	var c := [r.position, Vector2(r.end.x,r.position.y),r.end,Vector2(r.position.x,r.end.y)]
	for side in range(4):
		var a: Vector2 = c[side]
		var b: Vector2 = c[(side+1)%4]
		var length := a.distance_to(b)
		var offset := 0.0
		while offset < length - 0.00001:
			var end := minf(length, offset + 0.5)
			if end <= offset: return false
			var y := low
			while y < high - 0.00001:
				var y1 := minf(high, y + 0.5)
				if y < 0.0 and y1 > 0.0: y1 = 0.0
				if y1 <= y: return false
				var segment := meta.duplicate(true)
				segment["floor"] = "cabin" if meta.get("role") in ["cabin_rail", "cabin_gate"] else ("lower" if y < 0.0 else "upper")
				segment["heightBand"] = Vector2(y,y1)
				var p0 := a.lerp(b,offset/length)
				var p1 := a.lerp(b,end/length)
				# Region ownership follows this actual half-metre surface segment,
				# not the entire long wall that may span several hidden rooms.
				if meta.get('role') in ['wall','landing_gate'] and 'upper' not in meta.get('ownerRegions',[]):
					var segment_rect := Rect2(Vector2(minf(p0.x,p1.x),minf(p0.y,p1.y)),Vector2(absf(p1.x-p0.x),absf(p1.y-p0.y))).grow(0.001)
					segment['ownerRegions'] = _owners(segment_rect,_data.get('regions',[]),0.001)
				var umax := (end-offset)/0.5
				var vmax := (y1-y)/0.5
				if not _piece(plan,project,[Vector3(p0.x,y,p0.y),Vector3(p1.x,y,p1.y),Vector3(p1.x,y1,p1.y),Vector3(p0.x,y1,p0.y)],PackedVector2Array([Vector2(0,vmax),Vector2(umax,vmax),Vector2(umax,0),Vector2(0,0)]),_steel,segment): return false
				y = y1
			offset = end
	return true

# Caller chooses a floor and nonempty visible regions from validated player/support
# context; this helper does not decide visibility or infer a region graph.
func _owners(r: Rect2, regions: Array, padding: float = 0.26) -> Array:
	var owners: Array = []
	if r.grow(padding).intersects(Rect2(1.5,1,4,4)): owners.append("well")
	for item in regions:
		if r.grow(padding).intersects(_rect(item.polygon)): owners.append(item.id)
	return owners

func prepare(snapshot: Dictionary, scene: Dictionary, scene_hash: String, project: Callable, selected_floor: String, visible_regions: Array) -> Dictionary:
	if selected_floor not in ["upper", "lower"] or visible_regions.is_empty() or not project.is_valid() or not _load_once(): return {}
	var view: Dictionary = V.shaft_lift_view(snapshot.get("shaftLift"), snapshot, scene, scene_hash)
	if view.is_empty(): return {}
	var support = _data.get("support")
	var regions = _data.get("regions")
	var walls = _data.get("walls")
	var gates = _data.get("landingGates")
	var rails = _data.get("cabinRails")
	var cabin_gate = _data.get("cabinGate")
	if not support is Dictionary or not regions is Array or regions.size()!=5 or not walls is Array or walls.size()!=18 or not gates is Array or gates.size()!=2 or not rails is Array or rails.size()!=3 or not cabin_gate is Dictionary: return {}
	var cabin := _rect(support.get("polygon"))
	if cabin != Rect2(1.5,1,4,4): return {}
	var allowed: Array = ["well"]
	if selected_floor == "upper": allowed.append("upper")
	var seen := {}
	for region in regions:
		if not region is Dictionary or not region.get("id") is String or region.id.is_empty() or seen.has(region.id) or not _number(region.get("floorY")) or float(region.floorY) != -4.0 or _rect(region.get("polygon")).size == Vector2.ZERO: return {}
		seen[region.id] = true
		if selected_floor == "lower": allowed.append(region.id)
	var selected := {}
	for id in visible_regions:
		if not id is String or id not in allowed or selected.has(id): return {}
		selected[id] = true
	var plan: Array = []
	# Upper floor is a real projected plane with the authored well removed.
	# Cabin ground below supplies the opening at its actual support height.
	var upper_meta := {"sourceId": "shaft_upper_floor", "ownerRegions": ["upper"], "floor": "upper", "heightBand": Vector2(0,0), "role": "ground"}
	for upper_rect in [Rect2(0,0,24,1),Rect2(0,5,24,11),Rect2(0,1,1.5,4),Rect2(5.5,1,18.5,4)]:
		if not _floor_rect(plan,project,upper_rect,0.0,upper_meta,Vector2(0,-1)): return {}
	var collision=scene.get('collision')
	if not collision is Array or collision.size()!=UPPER_WALLS.size(): return {}
	var upper_seen := {}
	for wall in collision:
		if not wall is Dictionary or not wall.get('id') is String or not UPPER_WALLS.has(wall.id) or upper_seen.has(wall.id): return {}
		var wall_rect: Rect2 = _rect(wall.get('polygon'))
		if wall_rect!=UPPER_WALLS[wall.id]: return {}
		upper_seen[wall.id]=true
		var height: float = 0.55 if wall.id in ['shaft_support_column_a','shaft_support_column_b'] else 0.35
		var meta := {"sourceId":wall.id,"ownerRegions":["upper"],"role":"wall"}
		if not _wall(plan,project,wall_rect,0.0,height,meta): return {}
	if upper_seen.size()!=UPPER_WALLS.size(): return {}
	for region in regions:
		var meta := {"sourceId": region.id, "ownerRegions": [region.id], "floor": "lower", "heightBand": Vector2(-4,-4), "role": "ground"}
		if not _floor_rect(plan,project,_rect(region.polygon),-4.0,meta,Vector2(0,-1)): return {}
	var cabin_meta := {"sourceId": support.get("id"), "ownerRegions": ["well"], "floor": "cabin", "heightBand": Vector2(float(view.pose.heightM),float(view.pose.heightM)), "role": "cabin_ground"}
	if support.get("id") != "gh_shaft_two_stop_lift" or not _floor_rect(plan,project,cabin,float(view.pose.heightM),cabin_meta,Vector2(0,-1)): return {}
	var ids := {}
	for wall in walls:
		if not wall is Dictionary or not wall.get("id") is String or wall.id.is_empty() or ids.has(wall.id): return {}
		ids[wall.id] = true
		var band = wall.get("heightBandM")
		var r := _rect(wall.get("polygon"))
		if not _band(band) or r.size == Vector2.ZERO: return {}
		var meta := {"sourceId": wall.id, "ownerRegions": _owners(r,regions), "role": "wall"}
		if not _wall(plan,project,r,float(band[0]),float(band[1]),meta): return {}
	for i in range(2):
		var gate = gates[i]
		if not gate is Dictionary or gate.get("id") != (["upper_landing_gate","lower_landing_gate"][i]): return {}
		var band = gate.get("heightBandM")
		var r := _rect(gate.get("polygon"))
		if not _band(band) or r.size == Vector2.ZERO: return {}
		if not (view.upperGateOpen if i==0 else view.lowerGateOpen):
			var meta := {"sourceId": gate.id, "ownerRegions": _owners(r,regions), "role": "landing_gate"}
			if not _wall(plan,project,r,float(band[0]),float(band[1]),meta): return {}
	var cabin_items: Array = rails.duplicate()
	cabin_items.append(cabin_gate)
	for item in cabin_items:
		if not item is Dictionary or not item.get("id") is String or item.id.is_empty(): return {}
		var band = item.get("relativeHeightBandM")
		var r := _rect(item.get("polygon"))
		if not _band(band) or r.size == Vector2.ZERO: return {}
		if item == cabin_gate and view.cabinGateOpen: continue
		var role := "cabin_gate" if item == cabin_gate else "cabin_rail"
		var meta := {"sourceId": item.id, "ownerRegions": ["well"], "role": role}
		if not _wall(plan,project,r,float(view.pose.heightM)+float(band[0]),float(view.pose.heightM)+float(band[1]),meta): return {}
	var ground: Array = []
	var depth: Array = []
	for piece in plan:
		var visible := false
		for id in piece.ownerRegions:
			if selected.has(id): visible = true
		if not visible: continue
		if piece.floor != "cabin" and piece.floor != selected_floor: continue
		if piece.role in ["ground", "cabin_ground"]: ground.append(piece)
		else: depth.append(piece)
	return {"ground":ground, "depth":depth, "view":view, "selection":{"floor":selected_floor,"regions":visible_regions.duplicate()}}

# Draw ground first; caller places depth records into its existing actor sorter.
func draw_ground(canvas: CanvasItem, plan: Dictionary) -> bool:
	if not is_instance_valid(canvas) or not plan.get("selection") is Dictionary or not plan.get("ground") is Array: return false
	for piece in plan.ground:
		canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,piece.texture)
	return true

func draw_piece(canvas: CanvasItem, plan: Dictionary, index: int) -> bool:
	if not is_instance_valid(canvas) or not plan.get("selection") is Dictionary or not plan.get("depth") is Array or index < 0 or index >= plan.depth.size(): return false
	var piece: Dictionary = plan.depth[index]
	canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,piece.texture)
	return true
