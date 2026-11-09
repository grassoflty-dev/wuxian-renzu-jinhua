extends RefCounted
# The secondary scene is loaded through the same pinned manifest as the active scene.
const Catalog = preload('res://scripts/world/scene_catalog.gd')
const P = preload('res://scripts/bridge/protocol.gd')
const Content = preload('res://scripts/ui/content_model.gd')
var UV = PackedVector2Array([Vector2(187.5/1434.0,184.5/980.0),Vector2(345.5/1434.0,305.5/980.0),Vector2(185.5/1434.0,426.5/980.0),Vector2(27.5/1434.0,305.5/980.0)])
var catalogs: Dictionary = {}
var attempted: Dictionary = {}
var wall_attempted = false
var wall_texture: Texture2D
func _wall_texture() -> Texture2D:
	if wall_attempted: return wall_texture
	wall_attempted = true
	var path = 'res://assets/materials/cw_pressure_end_steel_local_r1.png'
	if ResourceLoader.exists(path,'Texture2D'):
		var loaded = ResourceLoader.load(path,'Texture2D')
		if loaded is Texture2D:
			var img: Image = loaded.get_image()
			if is_instance_valid(img) and img.get_width()==1254 and img.get_height()==1254 and img.get_format()==Image.FORMAT_RGB8: wall_texture=loaded
	if not is_instance_valid(wall_texture):
		var img = Image.new()
		if img.load_png_from_buffer(FileAccess.get_file_as_bytes(path))==OK and img.get_width()==1254 and img.get_height()==1254 and img.get_format()==Image.FORMAT_RGB8: wall_texture=ImageTexture.create_from_image(img)
	return wall_texture


func _neighbor(scene_id: String):
	var target = 'gh_gate_a' if scene_id=='gh_power_room' else 'gh_power_room' if scene_id=='gh_gate_a' else ''
	if target.is_empty(): return null
	if not attempted.has(target):
		attempted[target] = true
		var c = Catalog.new()
		if c.build('grey_hive',target):
			var b = c.current.get('boundsM')
			if b is Dictionary and b.get('x')==0 and b.get('z')==0 and b.get('width')==20 and b.get('depth')==14:
				catalogs[target] = c
	return catalogs.get(target)

# Draw only independently validated, static neighboring floor. Actors and gate state remain authoritative.
func draw_floor(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, key: String, camera, floor_material, draw_enabled: bool = true) -> bool:
	if not is_instance_valid(canvas) or not P.snapshot(snapshot) or snapshot.get('worldId')!='grey_hive' or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!=snapshot.get('sceneId') or key!='grey_hive:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch) or not camera.initialized: return false
	var secondary = _neighbor(snapshot.sceneId)
	if secondary==null: return false
	floor_material._load_definition_once(secondary.current.sceneId)
	if not floor_material.definition_valid_by_scene.get(secondary.current.sceneId,false) or not floor_material._valid_scene_layout(secondary.current,floor_material._profile_for_scene(secondary.current.sceneId)) or not floor_material._load_texture_once() or not is_instance_valid(floor_material.floor_texture): return false
	var metal = _wall_texture()
	if not is_instance_valid(metal): return false
	var offset = 20.0 if snapshot.sceneId=='gh_power_room' else -20.0
	var barrier_plan: Dictionary = {}
	var show_barrier = false
	if secondary.current.sceneId=='gh_gate_a':
		if snapshot.progression.get('currentWorldId')!='grey_hive' or not P.unique(snapshot.progression.get('worlds'),'worldId'): return false
		var progress: Dictionary = Content.progress(snapshot,'grey_hive')
		if progress.is_empty() or progress.visitId<=0: return false
		show_barrier = not 'hive_power' in progress.completedEvents
		if show_barrier:
			barrier_plan = prepare_gate_walls(secondary.current,camera)
			if barrier_plan.is_empty(): return false
	var quads: Array = []
	for z in range(7):
		for x in range(10):
			var x0 = float(x*2)+offset
			var z0 = float(z*2)
			var pts = PackedVector2Array()
			for point in [Vector3(x0,0,z0),Vector3(x0+2,0,z0),Vector3(x0+2,0,z0+2),Vector3(x0,0,z0+2)]:
				var screen: Vector2 = camera.project(point)
				if not is_finite(screen.x) or not is_finite(screen.y): return false
				pts.append(screen)
			quads.append(pts)
	if quads.size()!=70: return false
	var wall_quads: Array = []
	var prefix = 'gh_gate_wall_' if secondary.current.sceneId=='gh_gate_a' else 'gh_power_wall_'
	var suffixes = ['north','south','west_north','west_south','east_north','east_south']
	for suffix in suffixes:
		var matches = secondary.current.collision.filter(func(row): return row is Dictionary and row.get('id')==prefix+suffix)
		if matches.size()!=1: return false
		var poly: Array = matches[0].polygon
		if poly.size()!=4: return false
		var x0 = float(poly[0][0])+offset
		var z0 = float(poly[0][1])
		var x1 = float(poly[2][0])+offset
		var z1 = float(poly[2][1])
		var nx = int(round((x1-x0)*2.0))
		var nz = int(round((z1-z0)*2.0))
		if nx<1 or nz<1 or nx>40 or nz>40: return false
		for ix in range(nx):
			for iz in range(nz):
				var ax=x0+ix*0.5
				var az=z0+iz*0.5
				var cap = PackedVector2Array([camera.project(Vector3(ax,0.25,az)),camera.project(Vector3(ax+0.5,0.25,az)),camera.project(Vector3(ax+0.5,0.25,az+0.5)),camera.project(Vector3(ax,0.25,az+0.5))])
				wall_quads.append({'points':cap,'uv':PackedVector2Array([Vector2(0,0),Vector2(1,0),Vector2(1,1),Vector2(0,1)])})
		for edge in range(4):
			var a = Vector2(float(poly[edge][0])+offset,float(poly[edge][1]))
			var b = Vector2(float(poly[(edge+1)%4][0])+offset,float(poly[(edge+1)%4][1]))
			var count = int(round(a.distance_to(b)*2.0))
			if count<1 or count>40: return false
			for i in range(count):
				var v0 = a.lerp(b,float(i)/count)
				var v1 = a.lerp(b,float(i+1)/count)
				var face = PackedVector2Array([camera.project(Vector3(v0.x,0,v0.y)),camera.project(Vector3(v1.x,0,v1.y)),camera.project(Vector3(v1.x,0.25,v1.y)),camera.project(Vector3(v0.x,0.25,v0.y))])
				wall_quads.append({'points':face,'uv':PackedVector2Array([Vector2(0,1),Vector2(1,1),Vector2(1,0.5),Vector2(0,0.5)])})
	for record in wall_quads:
		for pt in record.points:
			if not is_finite(pt.x) or not is_finite(pt.y): return false
	var barrier_pieces: Array = []
	if show_barrier:
		var translation: Vector2 = camera.project(Vector3(offset,0,0))-camera.project(Vector3.ZERO)
		if not is_finite(translation.x) or not is_finite(translation.y): return false
		for piece in barrier_plan.owners['gh_gate_a_closed_barrier']:
			var shifted: PackedVector2Array = PackedVector2Array()
			for point in piece.points:
				var mapped: Vector2 = point+translation
				if not is_finite(mapped.x) or not is_finite(mapped.y): return false
				shifted.append(mapped)
			barrier_pieces.append({'points':shifted,'uv':piece.uv})
	if not draw_enabled: return true
	for pts in quads: canvas.draw_polygon(pts,PackedColorArray([Color.WHITE]),UV,floor_material.floor_texture)
	for record in wall_quads: canvas.draw_polygon(record.points,PackedColorArray([Color.WHITE]),record.uv,metal)
	for piece in barrier_pieces: canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,barrier_plan.texture)
	return true

# Stage the four Power side-wall owners as one image group. North and south retain their existing image paths.
func prepare_power_sides(scene: Dictionary, camera) -> Dictionary:
	if scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_power_room': return {}
	return _prepare_owners(scene,camera,['gh_power_wall_south','gh_power_wall_west_north','gh_power_wall_west_south','gh_power_wall_east_north','gh_power_wall_east_south'])

func prepare_gate_walls(scene: Dictionary, camera) -> Dictionary:
	if scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_gate_a': return {}
	var barrier = scene.get('collision',[]).filter(func(row): return row is Dictionary and row.get('id')=='gh_gate_a_closed_barrier')
	if barrier.size()!=1 or barrier[0].get('requiresEvent')!='hive_power': return {}
	var poly = barrier[0].get('polygon')
	var expected = [[9.5,0.5],[10.5,0.5],[10.5,13.5],[9.5,13.5]]
	if not poly is Array or poly.size()!=4: return {}
	for i in range(4):
		if not poly[i] is Array or poly[i].size()!=2 or not P.number(poly[i][0],expected[i][0],expected[i][0]) or not P.number(poly[i][1],expected[i][1],expected[i][1]): return {}
	return _prepare_owners(scene,camera,['gh_gate_wall_north','gh_gate_wall_south','gh_gate_wall_west_north','gh_gate_wall_west_south','gh_gate_wall_east_north','gh_gate_wall_east_south','gh_gate_a_closed_barrier'])

func _prepare_owners(scene: Dictionary, camera, ids: Array) -> Dictionary:
	if not camera.initialized: return {}
	var texture = _wall_texture()
	if not is_instance_valid(texture): return {}
	var owners = {}
	for id in ids:
		var matches = scene.get('collision',[]).filter(func(row): return row is Dictionary and row.get('id')==id)
		if matches.size()!=1 or not matches[0].get('polygon') is Array or matches[0].polygon.size()!=4: return {}
		var poly: Array = matches[0].polygon
		for point in poly:
			if not point is Array or point.size()!=2 or not P.number(point[0]) or not P.number(point[1]): return {}
		var x0=float(poly[0][0]); var z0=float(poly[0][1]); var x1=float(poly[2][0]); var z1=float(poly[2][1])
		var nx=int(round((x1-x0)*2)); var nz=int(round((z1-z0)*2))
		if nx<1 or nz<1 or nx>40 or nz>40 or not is_equal_approx(x0+nx*0.5,x1) or not is_equal_approx(z0+nz*0.5,z1): return {}
		var pieces: Array = []
		for ix in range(nx):
			for iz in range(nz):
				var x=x0+ix*0.5; var z=z0+iz*0.5
				pieces.append({'part':'cap','world':Rect2(x,z,0.5,0.5),'points':PackedVector2Array([camera.project(Vector3(x,0.25,z)),camera.project(Vector3(x+0.5,0.25,z)),camera.project(Vector3(x+0.5,0.25,z+0.5)),camera.project(Vector3(x,0.25,z+0.5))]),'uv':PackedVector2Array([Vector2(0,0),Vector2(1,0),Vector2(1,1),Vector2(0,1)])})
		for edge in range(4):
			var a=Vector2(float(poly[edge][0]),float(poly[edge][1])); var b=Vector2(float(poly[(edge+1)%4][0]),float(poly[(edge+1)%4][1]))
			var count=int(round(a.distance_to(b)*2))
			if count<1 or count>40 or not is_equal_approx(a.distance_to(b),count*0.5): return {}
			for i in range(count):
				var v0=a.lerp(b,float(i)/count); var v1=a.lerp(b,float(i+1)/count)
				pieces.append({'part':'face','world':Rect2(minf(v0.x,v1.x),minf(v0.y,v1.y),absf(v1.x-v0.x),absf(v1.y-v0.y)),'points':PackedVector2Array([camera.project(Vector3(v0.x,0,v0.y)),camera.project(Vector3(v1.x,0,v1.y)),camera.project(Vector3(v1.x,0.25,v1.y)),camera.project(Vector3(v0.x,0.25,v0.y))]),'uv':PackedVector2Array([Vector2(0,1),Vector2(1,1),Vector2(1,0.5),Vector2(0,0.5)])})
		for piece in pieces:
			for point in piece.points:
				if not is_finite(point.x) or not is_finite(point.y): return {}
		owners[id]=pieces
	if owners.size()!=ids.size(): return {}
	return {'texture':texture,'owners':owners}

func draw_owner(canvas: CanvasItem, plan: Dictionary, id: String) -> void:
	for piece in plan.owners[id]: canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,plan.texture)

func draw_caps(canvas: CanvasItem, plan: Dictionary, id: String) -> void:
	for piece in plan.owners[id]:
		if piece.part=='cap': canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,plan.texture)
