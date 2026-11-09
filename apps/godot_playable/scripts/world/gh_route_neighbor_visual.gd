extends RefCounted

# Geometry-only, source-identified strip. Never contains target actors or state.
const P = preload('res://scripts/bridge/protocol.gd')
const Catalog = preload('res://scripts/world/scene_catalog.gd')
const EDGES = {
	'gh_entry_maintenance':['','gh_power_room'], 'gh_power_room':['gh_entry_maintenance',''],
	'gh_lockdown':['','gh_bio_isolation'], 'gh_bio_isolation':['gh_lockdown','gh_gate_b'],
	'gh_gate_b':['gh_bio_isolation',''], 'gh_deep_decon':['','gh_sentinel_arena'],
	'gh_sentinel_arena':['gh_deep_decon','gh_beacon'], 'gh_beacon':['gh_sentinel_arena',''],
}
var catalogs: Dictionary = {}
var attempted: Dictionary = {}

func _catalog(id: String, hash: String):
	var key=id+':'+hash
	if not attempted.has(key):
		attempted[key]=true
		var catalog=Catalog.new()
		if catalog.build('grey_hive',id,hash): catalogs[key]=catalog
	return catalogs.get(key)

func prepare(snapshot: Dictionary, scene: Dictionary, key: String, source_hash: String, camera, art, events: Array, source_regions: Array) -> Dictionary:
	if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or not EDGES.has(snapshot.sceneId) or scene.get('sceneId')!=snapshot.sceneId or key!='grey_hive:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch) or not camera.initialized: return {}
	var v=snapshot.get('hiveRouteNeighbor')
	if not v is Dictionary or v.size()!=9 or v.get('sourceSceneId')!=snapshot.sceneId or v.get('worldEpoch')!=snapshot.worldEpoch or v.get('serverTick')!=snapshot.serverTick or v.get('authorityRevision')!=snapshot.authorityRevision or not P.token(v.get('token')) or typeof(v.get('ready'))!=TYPE_BOOL or typeof(v.get('straddling'))!=TYPE_BOOL: return {}
	if not v.get('targetSceneSourceSha256') is String or v.targetSceneSourceSha256.length()!=64: return {}
	if v.token.get('worldId')!='grey_hive' or v.token.get('sceneId')!=v.get('targetSceneId') or v.token.get('worldEpoch')!=snapshot.worldEpoch+1: return {}
	var source=_catalog(snapshot.sceneId,source_hash)
	if source==null or source.source_hash!=source_hash or source.current!=scene: return {}
	var targets: Array=EDGES[snapshot.sceneId]
	var east: bool = v.targetSceneId==targets[1] and not targets[1].is_empty()
	if not east and (v.targetSceneId!=targets[0] or targets[0].is_empty()): return {}
	var width: float=20.0 if snapshot.sceneId in ['gh_entry_maintenance','gh_power_room'] else 24.0
	var depth: float=14.0 if width==20.0 else 16.0
	var bounds=scene.get('boundsM')
	if not bounds is Dictionary or bounds.get('x')!=0 or bounds.get('z')!=0 or bounds.get('width')!=width or bounds.get('depth')!=depth: return {}
	var target=_catalog(v.targetSceneId,v.targetSceneSourceSha256)
	if target==null or target.source_hash!=v.targetSceneSourceSha256: return {}
	var tb=target.current.get('boundsM')
	if not tb is Dictionary or tb.get('x')!=0 or tb.get('z')!=0 or tb.get('width')!=width or tb.get('depth')!=depth or not art._load_once(): return {}
	var lo: float=5.5 if width==20.0 else 6.5
	var hi: float=8.5 if width==20.0 else 9.5
	var strip=Rect2(0.0 if east else width-2.0,lo,2.0,hi-lo)
	var offset=Vector3(width if east else -width,0,0)
	var project: Callable=func(p: Vector3)->Vector2: return camera.project(p+offset)
	var target_strip=_strip(target.current,strip,project,art,events,offset)
	var source_rect=Rect2(width-2.0 if east else 0.0,lo,2.0,hi-lo)
	var source_strip={'floor':[],'walls':[]}
	for allowed in source_regions:
		if not allowed is Rect2 or allowed.size.x<=0 or allowed.size.y<=0: return {}
		var visible: Rect2=source_rect.intersection(allowed)
		if not visible.has_area(): continue
		var part=_strip(scene,visible,Callable(camera,'project'),art,events,Vector3.ZERO)
		if part.is_empty(): return {}
		source_strip.floor.append_array(part.floor);source_strip.walls.append_array(part.walls)
	if target_strip.is_empty() or source_strip.floor.is_empty(): return {}
	# Optional real wall faces on the two edges of the Entry/Power aperture.
	if snapshot.sceneId in ['gh_entry_maintenance','gh_power_room'] and v.targetSceneId in ['gh_entry_maintenance','gh_power_room']:
		var target_faces=_first_pair_faces(target.current,strip,[strip],project,art,offset)
		var source_faces=_first_pair_faces(scene,source_rect,source_regions,Callable(camera,'project'),art,Vector3.ZERO)
		if not target_faces.is_empty() and not source_faces.is_empty():
			target_strip.walls.append_array(target_faces)
			source_strip.walls.append_array(source_faces)
	return {'floor':target_strip.floor,'walls':target_strip.walls,'reciprocal':source_strip,'token':v.token.duplicate(true),'targetHash':v.targetSceneSourceSha256,'sourceHash':source_hash,'source':snapshot.sceneId,'target':v.targetSceneId,'epoch':snapshot.worldEpoch,'player':snapshot.player.entityId,'offset':offset}

func _first_pair_faces(selected: Dictionary, strip: Rect2, allowed_regions: Array, project: Callable, art, offset: Vector3) -> Array:
	var entry=selected.get('sceneId')=='gh_entry_maintenance'
	if not entry and selected.get('sceneId')!='gh_power_room': return []
	var expected_strip=Rect2(18,5.5,2,3) if entry else Rect2(0,5.5,2,3)
	if strip!=expected_strip: return []
	var rows=selected.get('collision')
	if not P.rows(rows,2048) or not P.unique(rows,'id'): return []
	var prefix='gh_entry_wall_east_' if entry else 'gh_power_wall_west_'
	var x0=19.5 if entry else 0.0
	var result: Array=[]
	for side in ['north','south']:
		var found=rows.filter(func(row): return row is Dictionary and row.get('id')==prefix+side)
		var expected=Rect2(x0,0.5,0.5,5.0) if side=='north' else Rect2(x0,8.5,0.5,5.0)
		if found.size()!=1 or art._rect(found[0].get('polygon'))!=expected or found[0].get('requiresEvent')!=null or found[0].get('requiresActorFirstKill')!=null: return []
		var z=5.5 if side=='north' else 8.5
		var intervals: Array=[]
		for region in allowed_regions:
			if not region is Rect2 or region.position.y>z or region.end.y<z: continue
			var start=maxf(x0,region.position.x)
			var end=minf(x0+0.5,region.end.x)
			if end>start: intervals.append(Vector2(start,end))
		intervals.sort_custom(func(a,b): return a.x<b.x)
		if intervals.is_empty(): return []
		var merged: Array=[]
		for interval in intervals:
			if merged.is_empty() or interval.x>merged.back().y: merged.append(interval)
			else: merged[merged.size()-1]=Vector2(merged.back().x,maxf(merged.back().y,interval.y))
		for interval in merged:
			var uv=PackedVector2Array([Vector2(interval.x/0.5,1),Vector2(interval.y/0.5,1),Vector2(interval.y/0.5,0),Vector2(interval.x/0.5,0)])
			var pieces: Array=[]
			if not art._piece(pieces,project,[Vector3(interval.x,0,z),Vector3(interval.y,0,z),Vector3(interval.y,0.5,z),Vector3(interval.x,0.5,z)],uv,art._steel,{'role':'wall','id':prefix+side}): return []
			for piece in pieces: piece.worldAnchor+=offset
			result.append_array(pieces)
	return result

func _strip(selected: Dictionary,strip: Rect2,project: Callable,art,events: Array,offset: Vector3) -> Dictionary:
	var floor: Array=[]
	if not art._floor_rect(floor,project,strip,0.0,{'role':'ground'}) or floor.is_empty(): return {}
	var walls: Array=[]
	var rows=selected.get('collision')
	if not P.rows(rows,2048) or not P.unique(rows,'id'): return {}
	for row in rows:
		if not row is Dictionary or not row.get('id') is String: return {}
		var event=row.get('requiresEvent')
		if event!=null and (typeof(event)!=TYPE_STRING or event.is_empty()): return {}
		if event!=null and events.has(event): continue
		if row.get('requiresActorFirstKill')!=null: return {}
		var r: Rect2=art._rect(row.get('polygon'))
		if r.size.x<=0 or r.size.y<=0: return {}
		var clipped=r.intersection(strip)
		if clipped.size.x<=0 or clipped.size.y<=0: continue
		var height: float=0.5 if 'wall' in row.id else 0.25
		if not _wall_fragments(walls,project,r,clipped,strip,height,row.id,art): return {}
	for piece in floor+walls: piece.worldAnchor+=offset
	return {'floor':floor,'walls':walls}

func current(snapshot: Dictionary,scene: Dictionary,key: String,source_hash: String,camera,art,events: Array) -> Dictionary:
	if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or not snapshot.sceneId in ['gh_sentinel_arena','gh_bio_isolation','gh_beacon'] or scene.get('sceneId')!=snapshot.sceneId or key!='grey_hive:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch) or not camera.initialized: return {}
	var source=_catalog(snapshot.sceneId,source_hash)
	if source==null or source.source_hash!=source_hash or source.current!=scene or not art._load_once(): return {}
	var bounds=scene.get('boundsM')
	if not bounds is Dictionary or bounds.get('x')!=0 or bounds.get('z')!=0 or bounds.get('width')!=24 or bounds.get('depth')!=16: return {}
	return _strip(scene,Rect2(0,0,24,16),Callable(camera,'project'),art,events,Vector3.ZERO)

func _wall_fragments(out: Array,project: Callable,original: Rect2,clip: Rect2,strip: Rect2,height: float,id: String,art) -> bool:
	var uv=PackedVector2Array([clip.position/0.5,Vector2(clip.end.x,clip.position.y)/0.5,clip.end/0.5,Vector2(clip.position.x,clip.end.y)/0.5])
	if not art._piece(out,project,[Vector3(clip.position.x,height,clip.position.y),Vector3(clip.end.x,height,clip.position.y),Vector3(clip.end.x,height,clip.end.y),Vector3(clip.position.x,height,clip.end.y)],uv,art._steel,{'role':'wall','id':id}): return false
	for side in range(4):
		var along_x: bool=side==0 or side==2
		var fixed: float=original.position.y if side==0 else original.end.x if side==1 else original.end.y if side==2 else original.position.x
		if along_x and (fixed<strip.position.y or fixed>strip.end.y): continue
		if not along_x and (fixed<strip.position.x or fixed>strip.end.x): continue
		var start: float=clip.position.x if along_x else clip.position.y
		var end: float=clip.end.x if along_x else clip.end.y
		if end<=start: continue
		var a=Vector3(start,0,fixed) if along_x else Vector3(fixed,0,start)
		var b=Vector3(end,0,fixed) if along_x else Vector3(fixed,0,end)
		var face_uv=PackedVector2Array([Vector2(start/0.5,height/0.5),Vector2(end/0.5,height/0.5),Vector2(end/0.5,0),Vector2(start/0.5,0)])
		if not art._piece(out,project,[a,b,b+Vector3(0,height,0),a+Vector3(0,height,0)],face_uv,art._steel,{'role':'wall','id':id}): return false
	return true

func boundary(snapshot: Dictionary,scene: Dictionary,key: String,source_hash: String,camera,art) -> Dictionary:
	var v=snapshot.get('hiveRouteBoundary')
	if not P.snapshot(snapshot) or not v is Dictionary or v.size()!=6 or v.get('closed')!=true or typeof(v.get('east'))!=TYPE_BOOL or not EDGES.has(snapshot.sceneId): return {}
	if v.get('sourceSceneId')!=snapshot.sceneId or v.get('worldEpoch')!=snapshot.worldEpoch or v.get('sourceSceneSourceSha256')!=source_hash or key!='grey_hive:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch): return {}
	var target: String=EDGES[snapshot.sceneId][1 if v.east else 0]
	if target.is_empty() or v.get('targetSceneId')!=target: return {}
	var source=_catalog(snapshot.sceneId,source_hash)
	if source==null or source.current!=scene or not art._load_once(): return {}
	var width: float=20.0 if snapshot.sceneId in ['gh_entry_maintenance','gh_power_room'] else 24.0
	var x: float=width if v.east else 0.0
	var lo: float=5.5 if width==20.0 else 6.5
	var hi: float=8.5 if width==20.0 else 9.5
	var pieces: Array=[]
	var uv=PackedVector2Array([Vector2(lo/0.5,1),Vector2(hi/0.5,1),Vector2(hi/0.5,0),Vector2(lo/0.5,0)])
	if not art._piece(pieces,Callable(camera,'project'),[Vector3(x,0,lo),Vector3(x,0,hi),Vector3(x,0.5,hi),Vector3(x,0.5,lo)],uv,art._steel,{'role':'boundary'}): return {}
	return {'walls':pieces,'boundary':v.duplicate(true)}

func draw_piece(canvas: CanvasItem,piece: Dictionary) -> bool:
	if not piece.get('points') is PackedVector2Array or not piece.get('uv') is PackedVector2Array or not piece.get('texture') is Texture2D: return false
	var old=canvas.texture_repeat
	canvas.texture_repeat=CanvasItem.TEXTURE_REPEAT_ENABLED
	canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,piece.texture)
	canvas.texture_repeat=old
	return true

func reproject(plan: Dictionary,old_origin: Vector3,old_center: Vector2,old_ppm: float,camera,shift: Vector3) -> Dictionary:
	if not plan.get('floor') is Array or plan.floor.is_empty() or not plan.get('walls') is Array or not old_origin.is_finite() or not old_center.is_finite() or not is_finite(old_ppm) or old_ppm<=0.0 or not camera.initialized: return {}
	var anchor: Vector2=camera.project(old_origin+shift)
	var scale: float=camera.ppm/old_ppm
	if not anchor.is_finite() or not is_finite(scale) or scale<=0.0: return {}
	var result={'floor':[],'walls':[]}
	for group in ['floor','walls']:
		for piece in plan[group]:
			if not piece is Dictionary or not piece.get('points') is PackedVector2Array or not piece.get('worldAnchor') is Vector3 or not piece.get('texture') is Texture2D or not is_instance_valid(piece.texture): return {}
			var copy: Dictionary=piece.duplicate()
			var points:=PackedVector2Array()
			for p in piece.points:
				var q: Vector2=anchor+(p-old_center)*scale
				if not q.is_finite(): return {}
				points.append(q)
			copy.points=points;copy.worldAnchor=piece.worldAnchor+shift;result[group].append(copy)
	return result
