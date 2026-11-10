extends RefCounted
# Source-owned information geometry; it never changes collision or grants discovery.
const P = preload('res://scripts/bridge/protocol.gd')
const Content = preload('res://scripts/ui/content_model.gd')
const SOURCE = 'res://data/gh_beacon_exit_regions.local.json'
const BEACON = 'e6434c598904d7fe36f4f48ebc67559e6d484eb535393a7b447bc99cbf86c766'
const EXIT = 'b32ad8d1204e9eaf45512b0e2d5ea22e08f5bd09ec0b5e31980e41336c45ee09'
var data: Dictionary = {}
var attempted := false

func _load() -> bool:
	if not attempted:
		attempted=true
		var decoded=JSON.parse_string(FileAccess.get_file_as_string(SOURCE))
		if decoded is Dictionary: data=decoded
	return data.get('schema')=='gh-beacon-exit-info/v1' and data.get('variants') is Dictionary and data.get('owners') is Dictionary

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

func prepare(snapshot: Dictionary,scene: Dictionary,hash: String,camera,art,candidate: bool = false) -> Dictionary:
	if not _load() or not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or not snapshot.sceneId in ['gh_beacon','gh_exit'] or hash!=(BEACON if snapshot.sceneId=='gh_beacon' else EXIT) or snapshot.get('utilitySceneSourceSha256')!=hash or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or not camera.initialized or not art._load_once(): return {}
	if candidate:
		if snapshot.sceneId!='gh_exit' or snapshot.get('exitEntryPreview')!=true or not P.token(snapshot.get('entryToken')) or not P.same(snapshot.entryToken,snapshot): return {}
	elif snapshot.get('exitEntryPreview')==true or not snapshot.get('denseEntityRegions') is Dictionary: return {}
	var progress=Content.progress(snapshot,'grey_hive')
	if progress.is_empty() or not P.counter(progress.get('visitId'),1) or not progress.get('completedEvents') is Array: return {}
	var open: bool = 'hive_extraction' in progress.completedEvents
	var variant=data.variants.get('beacon' if snapshot.sceneId=='gh_beacon' else ('exit_open' if open else 'exit_closed'))
	var owner_data=data.owners.get(snapshot.sceneId)
	if not variant is Dictionary or not variant.get('regions') is Array or not variant.get('wall_boundary_fragments') is Dictionary or not variant.wall_boundary_fragments.get('rows') is Array or not owner_data is Array: return {}
	var visible=snapshot.get('currentVisibleRegions')
	if not candidate and (not visible is Array or visible.size()!=1 or typeof(visible[0])!=TYPE_STRING or snapshot.get('denseOccupiedRegion')!=visible[0]): return {}
	if not scene.get('collision') is Array or scene.collision.size()!=(13 if snapshot.sceneId=='gh_beacon' else 17) or scene.collision.size()!=owner_data.size() or not P.unique(scene.collision,'id'): return {}
	var owners: Array = []
	for item in owner_data:
		if not item is Dictionary or not item.get('id') is String or not item.get('rect_x0_z0_x1_z1') is Array or item.rect_x0_z0_x1_z1.size()!=4: return {}
		var v=item.rect_x0_z0_x1_z1
		var poly=[[v[0],v[1]],[v[2],v[1]],[v[2],v[3]],[v[0],v[3]]]
		var rows=scene.collision.filter(func(row): return row is Dictionary and row.get('id')==item.id)
		if rows.size()!=1 or rows[0].get('polygon')!=poly or rows[0].get('requiresEvent')!=item.get('requiresEvent') or rows[0].get('requiresActorFirstKill')!=null: return {}
		var rect=_rect(poly)
		if rect.size.x<=0 or rect.size.y<=0: return {}
		owners.append(rect)
	if candidate:
		var player=P.vector(snapshot.player.transform.positionM)
		if not player.is_finite() or player.y!=0.0: return {}
		for oi in range(owners.size()):
			if snapshot.sceneId=='gh_exit' and oi==6 and open: continue
			var r: Rect2=owners[oi]
			if player.x>r.position.x and player.x<r.end.x and player.z>r.position.y and player.z<r.end.y: return {}
		visible=[]
		for row in variant.regions:
			if row.get('available')!=true: continue
			for poly in row.get('polygon_parts_xz',[]):
				var r=_rect(poly)
				if r.size.x<=0 or r.size.y<=0: return {}
				if player.x>=r.position.x and player.x<=r.end.x and player.z>=r.position.y and player.z<=r.end.y:
					visible=[row.id]
					break
			if not visible.is_empty(): break
		if visible.is_empty(): return {}
	var region_index := -1
	for i in range(variant.regions.size()):
		if variant.regions[i].get('id')==visible[0] and variant.regions[i].get('available')==true:
			if region_index!=-1: return {}
			region_index=i
	if region_index<0: return {}
	var parts: Array = []
	var floor_pieces: Array = []
	for poly in variant.regions[region_index].get('polygon_parts_xz',[]):
		var rect=_rect(poly)
		if rect.size.x<=0 or rect.size.y<=0 or not art._floor_rect(floor_pieces,Callable(camera,'project'),rect,0.0,{'role':'ground'}): return {}
		parts.append(rect)
	if floor_pieces.is_empty(): return {}
	var pieces: Array = []
	var cap_seen: Dictionary = {}
	for row in variant.wall_boundary_fragments.rows:
		if not row is Array or row.size()!=4 or not P.counter(row[0],0,6) or not row[1] is Array or row[1].is_empty() or not row[2] in ['north','south','east','west'] or not row[3] is Array or row[3].size()!=2: return {}
		if row[0]!=region_index: continue
		if not P.counter(row[1][0],0,owner_data.size()-1) or not row[3][0] is Array or row[3][0].size()!=2 or not row[3][1] is Array or row[3][1].size()!=2: return {}
		for point in row[3]:
			if not P.number(point[0],0,24) or not P.number(point[1],0,16): return {}
		var oi=int(row[1][0])
		if oi<0 or oi>=owners.size(): return {}
		for value in row[1]:
			if not P.counter(value,0,owner_data.size()-1): return {}
			if snapshot.sceneId=='gh_exit' and int(value)==6:
				if open: return {}
				oi=6
			elif snapshot.sceneId=='gh_exit' and oi!=6 and int(value)<6:
				oi=int(value)
		var height: float = 0.75 if snapshot.sceneId=='gh_exit' and oi==6 else (0.5 if snapshot.sceneId=='gh_exit' and oi<6 else 0.25)
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
					if not art._piece(pieces,Callable(camera,'project'),[Vector3(x0,height,z0),Vector3(x1,height,z0),Vector3(x1,height,z1),Vector3(x0,height,z1)],uv,art._steel,{'anchor':Vector3((x0+x1)*0.5,0,(z0+z1)*0.5)}): return {}
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
			var y:=0.0
			while y<height-0.000001:
				var y1=minf(height,(floor(y/0.5)+1.0)*0.5)
				if y1<=y: return {}
				var v0=fposmod(y,0.5)/0.5; var v1=v0+(y1-y)/0.5
				var face_uv=PackedVector2Array([Vector2(u0,v1),Vector2(u1,v1),Vector2(u1,v0),Vector2(u0,v0)])
				if not art._piece(pieces,Callable(camera,'project'),[Vector3(first.x,y,first.y),Vector3(last.x,y,last.y),Vector3(last.x,y1,last.y),Vector3(first.x,y1,first.y)],face_uv,art._steel,{'anchor':Vector3((first.x+last.x)*0.5,0,(first.y+last.y)*0.5)}): return {}
				y=y1
			x=x1
	if pieces.is_empty(): return {}
	return {'floor':floor_pieces,'pieces':pieces,'region':visible[0],'parts':parts,'hash':hash,'scene_id':snapshot.sceneId,'region_open':open}

func contains(plan:Dictionary,p:Vector3) -> bool:
	if not p.is_finite() or p.y!=0.0: return false
	for rect in plan.get('parts',[]):
		if rect is Rect2 and p.x>=rect.position.x and p.x<=rect.end.x and p.z>=rect.position.y and p.z<=rect.end.y: return true
	return false
