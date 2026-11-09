extends RefCounted
# Source-owned information geometry; it never changes collision or grants discovery.
const P = preload('res://scripts/bridge/protocol.gd')
const SOURCE = 'res://data/gh_bio_regions.local.json'
const SHA = 'bf531fd9d49ee875653a8048130dd0c6e1d9b2170c1848ad48c49f2cb35ce0db'
var data: Dictionary = {}
var attempted := false

func _load() -> bool:
	if not attempted:
		attempted=true
		var decoded=JSON.parse_string(FileAccess.get_file_as_string(SOURCE))
		if decoded is Dictionary: data=decoded
	return data.get('schema')=='gh-bio-info/v1' and data.get('regions') is Array and data.regions.size()==7 and data.get('owners') is Array and data.owners.size()==24 and data.get('fragments') is Dictionary and data.fragments.get('rows') is Array and data.fragments.rows.size()==102

func _rect(poly: Variant) -> Rect2:
	if not poly is Array or poly.size()!=4: return Rect2()
	for p in poly:
		if not p is Array or p.size()!=2 or not P.number(p[0]) or not P.number(p[1]): return Rect2()
	var a=Vector2(float(poly[0][0]),float(poly[0][1]))
	var b=Vector2(float(poly[2][0]),float(poly[2][1]))
	if poly[1]!=[b.x,a.y] or poly[3]!=[a.x,b.y] or b.x<=a.x or b.y<=a.y: return Rect2()
	return Rect2(a,b-a)

func prepare(snapshot: Dictionary,scene: Dictionary,hash: String,camera,art) -> Dictionary:
	if not _load() or not P.snapshot(snapshot) or hash!=SHA or snapshot.get('utilitySceneSourceSha256')!=SHA or snapshot.worldId!='grey_hive' or snapshot.sceneId!='gh_bio_isolation' or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or not snapshot.get('denseEntityRegions') is Dictionary or not camera.initialized or not art._load_once(): return {}
	var visible=snapshot.get('currentVisibleRegions')
	if not visible is Array or visible.size()!=1 or typeof(visible[0])!=TYPE_STRING or snapshot.get('denseOccupiedRegion')!=visible[0]: return {}
	if not scene.get('collision') is Array or scene.collision.size()!=24 or not P.unique(scene.collision,'id'): return {}
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
	var outer: Array = []
	var cap_seen: Dictionary = {}
	for row in data.fragments.rows:
		if not row is Array or row.size()!=4 or not P.counter(row[0],0,6) or not row[1] is Array or row[1].is_empty() or not row[2] in ['north','south','east','west'] or not row[3] is Array or row[3].size()!=2: return {}
		if row[0]!=region_index: continue
		if not P.counter(row[1][0],0,23) or not row[3][0] is Array or row[3][0].size()!=2 or not row[3][1] is Array or row[3][1].size()!=2: return {}
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
		if oi<6:
			outer.append({'owner':data.owners[oi].id,'a':a,'b':b,'rect':owner})
			continue
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
	if pieces.is_empty() and outer.is_empty(): return {}
	return {'floor':floor_pieces,'pieces':pieces,'outer':outer,'region':visible[0],'parts':parts,'hash':hash,'scene_id':snapshot.sceneId}

func contains(plan:Dictionary,p:Vector3) -> bool:
	if not p.is_finite() or p.y!=0.0: return false
	for rect in plan.get('parts',[]):
		if rect is Rect2 and p.x>=rect.position.x and p.x<=rect.end.x and p.z>=rect.position.y and p.z<=rect.end.y: return true
	return false

# The full original atlas and six shadow plans must qualify first. Only the
# selected physical owner spans are submitted; UVs stay in the original atlas.
func prepare_atlas(plan:Dictionary,packets:Array,shadows:Array,bio,camera) -> Dictionary:
	if not bio._packets_are_complete(packets) or shadows.size()!=6 or not camera.initialized: return {}
	for shadow in shadows:
		if not shadow is Dictionary or not shadow.get('points') is PackedVector2Array or shadow.points.size()!=4 or not shadow.get('bounds_rect') is Rect2 or not shadow.bounds_rect.has_area(): return {}
	var result: Array=[]
	var shadow_pieces: Array=[]
	var seen: Dictionary={}
	for fragment in plan.get('outer',[]):
		if not fragment is Dictionary or not fragment.get('a') is Vector2 or not fragment.get('b') is Vector2 or not fragment.get('rect') is Rect2: return {}
		var runs=bio.RUNS.filter(func(run): return run.collision_id==fragment.owner)
		if runs.size()!=1: return {}
		var run=runs[0]
		var vertical: bool=run.axis=='z'
		var start: float=run.start.z if vertical else run.start.x
		var finish: float=run.end.z if vertical else run.end.x
		var a: Vector2=fragment.a
		var b: Vector2=fragment.b
		var lo: float=minf(a.y,b.y) if vertical else minf(a.x,b.x)
		var hi: float=maxf(a.y,b.y) if vertical else maxf(a.x,b.x)
		# A wall-mouth end is orthogonal to the run. Its visible section is
		# bounded by the actual owner thickness at that end.
		if is_equal_approx(lo,hi):
			var thickness: float=fragment.rect.size.x if vertical else fragment.rect.size.y
			if is_equal_approx(lo,start): hi=minf(finish,start+thickness)
			elif is_equal_approx(lo,finish): lo=maxf(start,finish-thickness)
			else: return {}
		if lo<start or hi>finish or hi<=lo: return {}
		var world_rect: Rect2=fragment.rect
		var clipped=Rect2(Vector2(world_rect.position.x,lo),Vector2(world_rect.size.x,hi-lo)) if vertical else Rect2(Vector2(lo,world_rect.position.y),Vector2(hi-lo,world_rect.size.y))
		var shade=PackedVector2Array()
		for p in [clipped.position,Vector2(clipped.end.x,clipped.position.y),clipped.end,Vector2(clipped.position.x,clipped.end.y)]:
			var screen=camera.project(Vector3(p.x,0,p.y))
			if not screen is Vector2 or not screen.is_finite(): return {}
			shade.append(screen)
		shadow_pieces.append(shade)
		for packet in packets:
			if packet.get('run_id')!=run.id: continue
			var module_start=start+float(packet.index)
			var left=maxf(lo,module_start);var right=minf(hi,module_start+1.0)
			if right<=left: continue
			var id=str(packet.id)+':'+str(left)+':'+str(right)
			if seen.has(id): continue
			seen[id]=true
			var x0=bio.FOOT_A.x+(left-module_start)*(bio.FOOT_B.x-bio.FOOT_A.x)
			var x1=bio.FOOT_A.x+(right-module_start)*(bio.FOOT_B.x-bio.FOOT_A.x)
			var raw=[Vector2(x0,0),Vector2(x1,0),Vector2(x1,float(bio.FRAME.size.y)),Vector2(x0,float(bio.FRAME.size.y))]
			var points=PackedVector2Array();var uv=PackedVector2Array()
			var tex: Texture2D=packet.texture
			if tex==null or tex.get_size().x<=0 or tex.get_size().y<=0: return {}
			for point in raw:
				var transformed=packet.transform_origin+Vector2(point.x*packet.transform_scale.x,point.y*packet.transform_scale.y).rotated(packet.transform_rotation)
				if not transformed.is_finite(): return {}
				points.append(transformed);uv.append(point/tex.get_size())
			var anchor: Vector3=run.start+(run.end-run.start)*((left+right)*0.5-start)/(finish-start)
			result.append({'points':points,'uv':uv,'texture':tex,'anchor':anchor,'fade_group':packet.fade_group})
	return {'pieces':result,'shadows':shadow_pieces}
