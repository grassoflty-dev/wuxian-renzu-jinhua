extends RefCounted
# Source-owned information geometry; it never changes collision or grants discovery.
const P = preload('res://scripts/bridge/protocol.gd')
const SOURCE = 'res://data/gh_decon_regions.local.json'
const SHA = 'a53fd5c62a4540a375b51f7286798c7bab1a5c73aa37e98d3f4d6bad7b24f1c0'
var data: Dictionary = {}
var attempted := false

func _load() -> bool:
	if not attempted:
		attempted=true
		var decoded=JSON.parse_string(FileAccess.get_file_as_string(SOURCE))
		if decoded is Dictionary: data=decoded
	return data.get('schema')=='gh-decon-info/v1' and data.get('regions') is Array and data.regions.size()==9 and data.get('owners') is Array and data.owners.size()==25 and data.get('fragments') is Dictionary and data.fragments.get('rows') is Array and data.fragments.rows.size()==111

func _rect(poly: Variant) -> Rect2:
	if not poly is Array or poly.size()!=4: return Rect2()
	for p in poly:
		if not p is Array or p.size()!=2 or not P.number(p[0]) or not P.number(p[1]): return Rect2()
	var a=Vector2(float(poly[0][0]),float(poly[0][1]))
	var b=Vector2(float(poly[2][0]),float(poly[2][1]))
	# Corner identity is compared in float64, exactly as the JSON writes it. Reading the
	# corners back through Vector2 first truncates them to float32 (f32(6.2)=6.199999809265137),
	# which rejected whole rects and emptied the region plan for this room.
	var x0=float(poly[0][0]);var z0=float(poly[0][1]);var x1=float(poly[2][0]);var z1=float(poly[2][1])
	if poly[1]!=[x1,z0] or poly[3]!=[x0,z1] or b.x<=a.x or b.y<=a.y: return Rect2()
	return Rect2(a,b-a)

func prepare(snapshot: Dictionary,scene: Dictionary,hash: String,camera,art) -> Dictionary:
	if not _load() or not P.snapshot(snapshot) or hash!=SHA or snapshot.get('utilitySceneSourceSha256')!=SHA or snapshot.worldId!='grey_hive' or snapshot.sceneId!='gh_deep_decon' or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or not snapshot.get('denseEntityRegions') is Dictionary or not camera.initialized or not art._load_once(): return {}
	var visible=snapshot.get('currentVisibleRegions')
	if not visible is Array or visible.size()!=1 or typeof(visible[0])!=TYPE_STRING or snapshot.get('denseOccupiedRegion')!=visible[0]: return {}
	if not scene.get('collision') is Array or scene.collision.size()!=25 or not P.unique(scene.collision,'id'): return {}
	var owners: Array = []
	for item in data.owners:
		if not item is Dictionary or not item.get('id') is String or not item.get('rect_x0_z0_x1_z1') is Array or item.rect_x0_z0_x1_z1.size()!=4: return {}
		var v=item.rect_x0_z0_x1_z1
		var poly=[[v[0],v[1]],[v[2],v[1]],[v[2],v[3]],[v[0],v[3]]]
		var rows=scene.collision.filter(func(row): return row is Dictionary and row.get('id')==item.id)
		if rows.size()!=1 or rows[0].get('polygon')!=poly or rows[0].get('requiresEvent')!=null or rows[0].get('requiresActorFirstKill')!=null: return {}
		var rect=_rect(poly)
		if rect.size.x<=0 or rect.size.y<=0: return {}
		owners.append(rect)
	var region_index := -1
	for i in range(data.regions.size()):
		if data.regions[i].get('id')==visible[0]:
			if region_index!=-1: return {}
			region_index=i
	if region_index<0: return {}
	var parts: Array = []
	var floor_pieces: Array = []
	for poly in data.regions[region_index].get('polygon_parts_xz',[]):
		var rect=_rect(poly)
		if rect.size.x<=0 or rect.size.y<=0 or not art._floor_rect(floor_pieces,Callable(camera,'project'),rect,0.0,{'role':'ground'}): return {}
		parts.append(rect)
	if floor_pieces.is_empty(): return {}
	var pieces: Array = []
	var cap_seen: Dictionary = {}
	for row in data.fragments.rows:
		if not row is Array or row.size()!=4 or not P.counter(row[0],0,8) or not row[1] is Array or row[1].is_empty() or not row[2] in ['north','south','east','west'] or not row[3] is Array or row[3].size()!=2: return {}
		if row[0]!=region_index: continue
		if not P.counter(row[1][0],0,24) or not row[3][0] is Array or row[3][0].size()!=2 or not row[3][1] is Array or row[3][1].size()!=2: return {}
		for point in row[3]:
			if not P.number(point[0],0,24) or not P.number(point[1],0,16): return {}
		var oi=int(row[1][0])
		if oi<0 or oi>=owners.size(): return {}
		var a=Vector2(float(row[3][0][0]),float(row[3][0][1]))
		var b=Vector2(float(row[3][1][0]),float(row[3][1][1]))
		var vertical=absf(a.x-b.x)<0.000001
		if not vertical and absf(a.y-b.y)>0.000001: return {}
		var owner: Rect2=owners[oi]
		if vertical:
			if not (is_equal_approx(a.x,owner.position.x) or is_equal_approx(a.x,owner.end.x)) or minf(a.y,b.y)<owner.position.y or maxf(a.y,b.y)>owner.end.y: return {}
		else:
			if not (is_equal_approx(a.y,owner.position.y) or is_equal_approx(a.y,owner.end.y)) or minf(a.x,b.x)<owner.position.x or maxf(a.x,b.x)>owner.end.x: return {}
		var cap=Rect2(Vector2(owner.position.x,minf(a.y,b.y)),Vector2(owner.size.x,absf(a.y-b.y))) if vertical else Rect2(Vector2(minf(a.x,b.x),owner.position.y),Vector2(absf(a.x-b.x),owner.size.y))
		if cap.size.x<=0 or cap.size.y<=0: return {}
		var tag=str(cap)
		if not cap_seen.has(tag):
			cap_seen[tag]=true
			var z0=cap.position.y
			while z0<cap.end.y-0.000001:
				var z1=minf(cap.end.y,(floor(z0/0.5)+1.0)*0.5)
				if z1<=z0: return {}
				var x0=cap.position.x
				while x0<cap.end.x-0.000001:
					var x1=minf(cap.end.x,(floor(x0/0.5)+1.0)*0.5)
					if x1<=x0: return {}
					var u0=fposmod(x0,0.5)/0.5; var v0=fposmod(z0,0.5)/0.5
					var u1=u0+(x1-x0)/0.5; var v1=v0+(z1-z0)/0.5
					var uv=PackedVector2Array([Vector2(u0,v0),Vector2(u1,v0),Vector2(u1,v1),Vector2(u0,v1)])
					if not art._piece(pieces,Callable(camera,'project'),[Vector3(x0,0.25,z0),Vector3(x1,0.25,z0),Vector3(x1,0.25,z1),Vector3(x0,0.25,z1)],uv,art._steel,{'anchor':Vector3((x0+x1)*0.5,0,(z0+z1)*0.5)}): return {}
					x0=x1
				z0=z1
		var start=a.y if vertical else a.x
		var finish=b.y if vertical else b.x
		if finish<=start: return {}
		var x=start
		while x<finish-0.000001:
			var x1=minf(finish,(floor(x/0.5)+1.0)*0.5)
			if x1<=x: return {}
			var first=Vector2(a.x,x) if vertical else Vector2(x,a.y)
			var last=Vector2(a.x,x1) if vertical else Vector2(x1,a.y)
			var u0=fposmod(x,0.5)/0.5; var u1=u0+(x1-x)/0.5
			var face_uv=PackedVector2Array([Vector2(u0,0.5),Vector2(u1,0.5),Vector2(u1,0),Vector2(u0,0)])
			if not art._piece(pieces,Callable(camera,'project'),[Vector3(first.x,0,first.y),Vector3(last.x,0,last.y),Vector3(last.x,0.25,last.y),Vector3(first.x,0.25,first.y)],face_uv,art._steel,{'anchor':Vector3((first.x+last.x)*0.5,0,(first.y+last.y)*0.5)}): return {}
			x=x1
	if pieces.is_empty(): return {}
	return {'floor':floor_pieces,'pieces':pieces,'region':visible[0],'parts':parts,'hash':hash,'scene_id':snapshot.sceneId}

func contains(plan:Dictionary,p:Vector3) -> bool:
	if not p.is_finite() or p.y!=0.0: return false
	for rect in plan.get('parts',[]):
		if rect is Rect2 and p.x>=rect.position.x and p.x<=rect.end.x and p.z>=rect.position.y and p.z<=rect.end.y: return true
	return false
