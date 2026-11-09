extends RefCounted
# Source-owned physical floor/wall packets only. This never projects target actors.
const P=preload('res://scripts/bridge/protocol.gd')
const Catalog=preload('res://scripts/world/scene_catalog.gd')
const EDGES={'mh_fog_pier':['','mh_tidal_warehouse'],'mh_tidal_warehouse':['mh_fog_pier','mh_signal_yard'],'mh_signal_yard':['mh_tidal_warehouse','mh_drowned_quay'],'mh_drowned_quay':['mh_signal_yard','mh_breakwater'],'mh_breakwater':['mh_drowned_quay','mh_pump_station'],'mh_pump_station':['mh_breakwater','mh_resonance_tower'],'mh_resonance_tower':['mh_pump_station','mh_warden_arena'],'mh_warden_arena':['mh_resonance_tower','mh_extraction'],'mh_extraction':['mh_warden_arena','']}
const PREFIX={'mh_fog_pier':'pier','mh_tidal_warehouse':'warehouse','mh_signal_yard':'yard','mh_drowned_quay':'quay','mh_breakwater':'breakwater','mh_pump_station':'pump','mh_resonance_tower':'tower','mh_warden_arena':'warden','mh_extraction':'extraction'}
const CORNERS=[Vector2(820,240),Vector2(1032,342),Vector2(392,627),Vector2(180,525)]
var catalogs: Dictionary={}
var attempted: Dictionary={}

func _catalog(id: String, hash: String):
	var key=id+':'+hash
	if not attempted.has(key):
		attempted[key]=true
		var catalog=Catalog.new()
		if catalog.build('mist_harbor',id,hash): catalogs[key]=catalog
	return catalogs.get(key)

func _context(s: Dictionary,scene: Dictionary,key: String,hash: String,camera,art,stone) -> bool:
	if not P.snapshot(s) or s.worldId!='mist_harbor' or not EDGES.has(s.sceneId) or scene.get('sceneId')!=s.sceneId or key!='mist_harbor:'+str(s.sceneId)+':'+str(s.worldEpoch) or not camera.initialized or not art._load_once(): return false
	var source=_catalog(s.sceneId,hash)
	return source!=null and source.source_hash==hash and source.current==scene and stone._load_texture_after_visible_validation()!=null

func _uv(u: float,v: float) -> Vector2:
	return (CORNERS[0].lerp(CORNERS[1],u).lerp(CORNERS[3].lerp(CORNERS[2],u),v))/Vector2(1228,1254)

func _floor(out: Array,rect: Rect2,project: Callable,art,texture: Texture2D,offset: Vector3) -> bool:
	var z=rect.position.y
	while z<rect.end.y-0.00001:
		var z1=minf(rect.end.y,(floor(z/4.5)+1.0)*4.5)
		if z1<=z: return false
		var x=rect.position.x
		while x<rect.end.x-0.00001:
			var x1=minf(rect.end.x,(floor(x/1.5)+1.0)*1.5)
			if x1<=x: return false
			var u=(x-floor(x/1.5)*1.5)/1.5;var u1=u+(x1-x)/1.5
			var v=(z-floor(z/4.5)*4.5)/4.5;var v1=v+(z1-z)/4.5
			if not art._piece(out,project,[Vector3(x,0,z),Vector3(x1,0,z),Vector3(x1,0,z1),Vector3(x,0,z1)],PackedVector2Array([_uv(u,v),_uv(u1,v),_uv(u1,v1),_uv(u,v1)]),texture,{'role':'ground'}): return false
			out[-1].worldAnchor+=offset
			x=x1
		z=z1
	return true

func _height(scene_id: String,id: String) -> float:
	if scene_id in ['mh_fog_pier','mh_drowned_quay']: return 0.25
	if id.ends_with('_north') and not id.contains('_east_') or id.contains('_west_'): return 0.8
	return 0.18

func _pack(selected: Dictionary,clip: Rect2,project: Callable,art,stone,wall_art,offset: Vector3,pump_state: Variant) -> Dictionary:
	var rows=selected.get('collision');var scene_id=str(selected.get('sceneId',''))
	if not PREFIX.has(scene_id) or not P.rows(rows,64) or rows.size()!=(5 if scene_id=='mh_extraction' else 6) or not P.unique(rows,'id'): return {}
	var walkable=selected.get('walkablePolygons')
	if not walkable is Array or walkable.size()!=1 or not walkable[0] is Dictionary or walkable[0].get('polygon')!=[[0.5,0.5],[23.5,0.5],[23.5,15.5],[0.5,15.5]]: return {}
	var floor: Array=[];var walls: Array=[]
	var actual_floor=clip.intersection(Rect2(0.5,0.5,23,15))
	if not actual_floor.has_area(): return {}
	if scene_id=='mh_pump_station':
		var water=Rect2(18,5.5,5.5,5)
		var land=[Rect2(0.5,0.5,17.5,15),Rect2(18,0.5,5.5,5),Rect2(18,10.5,5.5,5)]
		for portion in land:
			var clipped=actual_floor.intersection(portion)
			if clipped.has_area() and not _floor(floor,clipped,project,art,stone,offset): return {}
		var clipped=actual_floor.intersection(water)
		if not pump_state is Dictionary or pump_state.size()!=1 or pump_state.get('state') not in ['ready','draining','drained']: return {}
		if clipped.has_area() and pump_state.state=='drained':
			if not _floor(floor,clipped,project,art,stone,offset): return {}
		elif clipped.has_area():
			var tex=load('res://assets/materials/mh_pump_deep_water_surface_local_r1.png')
			if not tex is Texture2D or tex.get_width()!=1254 or tex.get_height()!=1254: return {}
			var z0=clipped.position.y
			while z0<clipped.end.y-0.00001:
				var z1=minf(clipped.end.y,floorf(z0)+1.0)
				var x0=clipped.position.x
				while x0<clipped.end.x-0.00001:
					var x1=minf(clipped.end.x,floorf(x0)+1.0)
					var u=x0-floorf(x0);var v=z0-floorf(z0)
					if not art._piece(floor,project,[Vector3(x0,0,z0),Vector3(x1,0,z0),Vector3(x1,0,z1),Vector3(x0,0,z1)],PackedVector2Array([Vector2(u,v),Vector2(u+x1-x0,v),Vector2(u+x1-x0,v+z1-z0),Vector2(u,v+z1-z0)]),tex,{'role':'ground'}): return {}
					floor[-1].worldAnchor+=offset
					x0=x1
				z0=z1
	elif not _floor(floor,actual_floor,project,art,stone,offset): return {}
	var suffixes=['north','south','west_north','west_south','east'] if scene_id=='mh_extraction' else ['north','south','west_north','west_south','east_north','east_south']
	for suffix in suffixes:
		var id: String=PREFIX[scene_id]+'_wall_'+suffix
		var found=rows.filter(func(row): return row is Dictionary and row.get('id')==id)
		if found.size()!=1 or found[0].get('requiresEvent')!=null or found[0].get('requiresActorFirstKill')!=null: return {}
		var original: Rect2=art._rect(found[0].get('polygon'))
		if original.size.x<=0 or original.size.y<=0: return {}
		var part: Rect2=original.intersection(clip)
		if not part.has_area(): continue
		var start=walls.size()
		if not wall_art._wall_fragments(walls,project,original,part,clip,_height(scene_id,id),id,art): return {}
		for n in range(start,walls.size()): walls[n].worldAnchor+=offset
	return {'floor':floor,'walls':walls}

func current(s: Dictionary,scene: Dictionary,key: String,hash: String,camera,art,stone,wall_art) -> Dictionary:
	if not _context(s,scene,key,hash,camera,art,stone): return {}
	return _pack(scene,Rect2(0,0,24,16),Callable(camera,'project'),art,stone._load_texture_after_visible_validation(),wall_art,Vector3.ZERO,s.get('mistHarborPump'))

func prepare(s: Dictionary,scene: Dictionary,key: String,hash: String,camera,art,stone,wall_art) -> Dictionary:
	if not _context(s,scene,key,hash,camera,art,stone): return {}
	var v=s.get('hiveRouteNeighbor')
	if not v is Dictionary or v.size()!=9 or v.get('sourceSceneId')!=s.sceneId or v.get('worldEpoch')!=s.worldEpoch or v.get('serverTick')!=s.serverTick or v.get('authorityRevision')!=s.authorityRevision or not P.token(v.get('token')) or typeof(v.get('ready'))!=TYPE_BOOL or typeof(v.get('straddling'))!=TYPE_BOOL: return {}
	if not v.get('targetSceneSourceSha256') is String or v.targetSceneSourceSha256.length()!=64 or v.token.worldId!='mist_harbor' or v.token.sceneId!=v.get('targetSceneId') or v.token.worldEpoch!=s.worldEpoch+1: return {}
	var edge: Array=EDGES[s.sceneId];var east=v.targetSceneId==edge[1] and not edge[1].is_empty()
	if not east and (v.targetSceneId!=edge[0] or edge[0].is_empty()): return {}
	var target=_catalog(v.targetSceneId,v.targetSceneSourceSha256)
	if target==null or target.source_hash!=v.targetSceneSourceSha256: return {}
	var offset=Vector3(24 if east else -24,0,0)
	var project: Callable=func(p: Vector3)->Vector2: return camera.project(p+offset)
	var target_rect=Rect2(0.0 if east else 22.0,5.5,2.0,4.5)
	var source_rect=Rect2(22.0 if east else 0.0,5.5,2.0,4.5)
	var target_plan=_pack(target.current,target_rect,project,art,stone._load_texture_after_visible_validation(),wall_art,offset,s.get('mistHarborPump'))
	var source_plan=_pack(scene,source_rect,Callable(camera,'project'),art,stone._load_texture_after_visible_validation(),wall_art,Vector3.ZERO,s.get('mistHarborPump'))
	if target_plan.is_empty() or source_plan.is_empty(): return {}
	var bridge=Rect2(23.5 if east else -0.5,6,1,3.5)
	if not _floor(target_plan.floor,bridge,Callable(camera,'project'),art,stone._load_texture_after_visible_validation(),Vector3.ZERO): return {}
	var reciprocal_bridge=Rect2(bridge.position.x-offset.x,6,1,3.5)
	if not _floor(source_plan.floor,reciprocal_bridge,project,art,stone._load_texture_after_visible_validation(),offset): return {}
	return {'floor':target_plan.floor,'walls':target_plan.walls,'reciprocal':source_plan,'token':v.token.duplicate(true),'targetHash':v.targetSceneSourceSha256,'sourceHash':hash,'source':s.sceneId,'target':v.targetSceneId,'epoch':s.worldEpoch,'player':s.player.entityId,'offset':offset}

func boundary(s: Dictionary,scene: Dictionary,key: String,hash: String,camera,art,stone) -> Dictionary:
	if not _context(s,scene,key,hash,camera,art,stone): return {}
	var v=s.get('hiveRouteBoundary')
	if not v is Dictionary or v.size() not in [6,7] or v.get('closed')!=true or typeof(v.get('east'))!=TYPE_BOOL or v.get('sourceSceneId')!=s.sceneId or v.get('worldEpoch')!=s.worldEpoch or v.get('sourceSceneSourceSha256')!=hash: return {}
	var target: String=EDGES[s.sceneId][1 if v.east else 0]
	if target.is_empty() or v.get('targetSceneId')!=target: return {}
	if v.has('prerequisiteBlocked'):
		if typeof(v.prerequisiteBlocked)!=TYPE_BOOL: return {}
		if not v.prerequisiteBlocked: return {'walls':[],'boundary':v.duplicate(true)}
	var x: float=23.5 if v.east else 0.5
	var high: float=0.25 if v.has('prerequisiteBlocked') or s.sceneId in ['mh_fog_pier','mh_drowned_quay'] else (0.18 if v.east else 0.8)
	var uv=PackedVector2Array([Vector2(12,high/0.5),Vector2(19,high/0.5),Vector2(19,0),Vector2(12,0)])
	var walls: Array=[]
	if not art._piece(walls,Callable(camera,'project'),[Vector3(x,0,6),Vector3(x,0,9.5),Vector3(x,high,9.5),Vector3(x,high,6)],uv,art._steel,{'role':'boundary'}): return {}
	return {'walls':walls,'boundary':v.duplicate(true)}
