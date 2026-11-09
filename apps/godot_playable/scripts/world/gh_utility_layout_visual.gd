extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const Catalog = preload('res://scripts/world/scene_catalog.gd')
const BIO = 'bf531fd9d49ee875653a8048130dd0c6e1d9b2170c1848ad48c49f2cb35ce0db'
const DECON = 'a53fd5c62a4540a375b51f7286798c7bab1a5c73aa37e98d3f4d6bad7b24f1c0'
const BIO_OLD = 'a3d629f04dc955cf0f8e644b54a4a64d054411a7d59a9fb15da5c9d9ea953aff'
const BEACON = 'e6434c598904d7fe36f4f48ebc67559e6d484eb535393a7b447bc99cbf86c766'
const BEACON_OLD = '3cab424771d10799ee82a453a72117affb0ebca09dd814f25308be70310da9ee'
var catalogs: Dictionary = {}

func _catalog(id: String, hash: String):
	var key=id+':'+hash
	if not catalogs.has(key):
		var item=Catalog.new()
		if not item.build('grey_hive',id,hash): return null
		catalogs[key]=item
	return catalogs[key]

func is_new(scene_id: String, hash: String) -> bool:
	return (scene_id=='gh_bio_isolation' and hash==BIO) or (scene_id=='gh_deep_decon' and hash==DECON) or (scene_id=='gh_beacon' and hash==BEACON)

func _original_scene(scene: Dictionary, id: String, old_hash: String) -> Dictionary:
	var old=_catalog(id,old_hash)
	if old==null: return {}
	var a: Dictionary = scene.duplicate(true)
	var b: Dictionary = old.current.duplicate(true)
	a.erase('collision');a.erase('navigation');b.erase('collision');b.erase('navigation')
	if a!=b: return {}
	for prior in old.current.collision:
		var found=scene.collision.filter(func(row): return row is Dictionary and row.get('id')==prior.id)
		if found.size()!=1 or found[0]!=prior: return {}
	return old.current

func _edge_values(rects: Array, axis: String, limit: float) -> Array:
	var values: Array = []
	var v:=0.0
	while v<=limit+0.00001:
		values.append(v);v+=0.5
	for row in rects:
		var r: Rect2 = row.rect
		values.append(r.position.x if axis=='x' else r.position.y)
		values.append(r.end.x if axis=='x' else r.end.y)
	values.sort()
	var unique: Array = []
	for value in values:
		if unique.is_empty() or absf(float(value)-float(unique.back()))>0.00001: unique.append(value)
	return unique

func _cell_owner(rects: Array, x: float, z: float) -> String:
	for row in rects:
		var r: Rect2 = row.rect
		if x>r.position.x and x<r.end.x and z>r.position.y and z<r.end.y: return row.id
	return ''

func _piece(art, project: Callable, world: Array, uv: PackedVector2Array, owner: String, anchor: Vector3) -> Dictionary:
	var result: Array = []
	if not art._piece(result,project,world,uv,art._steel,{'owner':owner,'anchor':anchor}) or result.size()!=1: return {}
	return result[0]

func prepare(scene: Dictionary, hash: String, project: Callable, art) -> Dictionary:
	var id: String = str(scene.get('sceneId',''))
	if scene.get('worldId')!='grey_hive' or not is_new(id,hash) or not project.is_valid() or not art._load_once(): return {}
	var catalog=_catalog(id,hash)
	if catalog==null or catalog.source_hash!=hash or catalog.current!=scene: return {}
	var count: int = 24 if id=='gh_bio_isolation' else (13 if id=='gh_beacon' else 25)
	if not scene.get('collision') is Array or scene.collision.size()!=count or not P.unique(scene.collision,'id'): return {}
	var original: Dictionary = _original_scene(scene,id,BIO_OLD if id=='gh_bio_isolation' else BEACON_OLD) if id in ['gh_bio_isolation','gh_beacon'] else {}
	if id in ['gh_bio_isolation','gh_beacon'] and original.is_empty(): return {}
	var rects: Array = []
	for row in scene.collision:
		if not row is Dictionary or not row.get('id') is String or row.get('requiresEvent')!=null or row.get('requiresActorFirstKill')!=null: return {}
		var r: Rect2 = art._rect(row.get('polygon'))
		if r.size.x<=0 or r.size.y<=0: return {}
		rects.append({'id':row.id,'rect':r})
	var floor: Array = []
	if not art._floor_rect(floor,project,Rect2(0,0,24,16),0.0,{'role':'ground'}) or floor.is_empty(): return {}
	var xs=_edge_values(rects,'x',24.0)
	var zs=_edge_values(rects,'z',16.0)
	var pieces: Array = []
	var seen: Dictionary = {}
	for xi in range(xs.size()-1):
		for zi in range(zs.size()-1):
			var x0: float=xs[xi];var x1: float=xs[xi+1];var z0: float=zs[zi];var z1: float=zs[zi+1]
			var owner=_cell_owner(rects,(x0+x1)*0.5,(z0+z1)*0.5)
			if owner.is_empty(): continue
			seen[owner]=true
			# Bio's six outer owners retain the original validated atlas.
			if id=='gh_bio_isolation' and owner.begins_with('gh_bio_isolation_wall_'): continue
			if id=='gh_beacon' and owner.begins_with('beacon_wall_'): continue
			var u0: float=fposmod(x0,0.5)/0.5;var u1: float=u0+(x1-x0)/0.5
			var v0: float=fposmod(z0,0.5)/0.5;var v1: float=v0+(z1-z0)/0.5
			var anchor=Vector3((x0+x1)*0.5,0,(z0+z1)*0.5)
			var cap=_piece(art,project,[Vector3(x0,0.25,z0),Vector3(x1,0.25,z0),Vector3(x1,0.25,z1),Vector3(x0,0.25,z1)],PackedVector2Array([Vector2(u0,v0),Vector2(u1,v0),Vector2(u1,v1),Vector2(u0,v1)]),owner,anchor)
			if cap.is_empty(): return {}
			pieces.append(cap)
			var checks=[Vector2(x0-0.0001,(z0+z1)*0.5),Vector2(x1+0.0001,(z0+z1)*0.5),Vector2((x0+x1)*0.5,z0-0.0001),Vector2((x0+x1)*0.5,z1+0.0001)]
			var edges=[[Vector2(x0,z0),Vector2(x0,z1)],[Vector2(x1,z0),Vector2(x1,z1)],[Vector2(x0,z0),Vector2(x1,z0)],[Vector2(x0,z1),Vector2(x1,z1)]]
			for side in range(4):
				if not _cell_owner(rects,checks[side].x,checks[side].y).is_empty(): continue
				var a: Vector2=edges[side][0];var b: Vector2=edges[side][1]
				var start: float=a.y if side<2 else a.x
				var finish: float=b.y if side<2 else b.x
				var f0: float=fposmod(start,0.5)/0.5;var f1: float=f0+(finish-start)/0.5
				var face=_piece(art,project,[Vector3(a.x,0,a.y),Vector3(b.x,0,b.y),Vector3(b.x,0.25,b.y),Vector3(a.x,0.25,a.y)],PackedVector2Array([Vector2(f0,0.5),Vector2(f1,0.5),Vector2(f1,0),Vector2(f0,0)]),owner,anchor)
				if face.is_empty(): return {}
				pieces.append(face)
	if seen.size()!=rects.size() or pieces.is_empty(): return {}
	return {'floor':floor,'pieces':pieces,'original_bio':original,'original_beacon':original,'hash':hash,'scene_id':id,'owner_count':seen.size()}

func draw_piece(canvas: CanvasItem, piece: Dictionary) -> bool:
	if not is_instance_valid(canvas) or piece.is_empty(): return false
	canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,piece.texture)
	return true
