extends RefCounted
const P=preload('res://scripts/bridge/protocol.gd')
const Catalog=preload('res://scripts/world/scene_catalog.gd')
const EDGES={'cw_entry_foundry':['','cw_pressure_hall'],'cw_pressure_hall':['cw_entry_foundry','cw_conveyor_bridge'],'cw_conveyor_bridge':['cw_pressure_hall','cw_boiler_chamber'],'cw_boiler_chamber':['cw_conveyor_bridge','']}
const BELT='res://assets/materials/cw_conveyor_belt_temporal_local_r1.png'
var catalogs: Dictionary={}
var attempted: Dictionary={}
var belt_attempted:=false
var belt_texture: Texture2D

func _catalog(id: String,hash: String):
	var key=id+':'+hash
	if not attempted.has(key):
		attempted[key]=true
		var catalog=Catalog.new()
		if catalog.build('clockworks',id,hash): catalogs[key]=catalog
	return catalogs.get(key)

func _belt_texture() -> Texture2D:
	if belt_attempted: return belt_texture
	belt_attempted=true
	var image=Image.new()
	if image.load(BELT)==OK and image.get_size()==Vector2i(1254,1254) and image.get_format()==Image.FORMAT_RGB8: belt_texture=ImageTexture.create_from_image(image)
	return belt_texture

func _context(s: Dictionary,scene: Dictionary,key: String,hash: String,camera,art,floor_material) -> bool:
	if not P.snapshot(s) or s.worldId!='clockworks' or not EDGES.has(s.sceneId) or scene.get('sceneId')!=s.sceneId or key!='clockworks:'+str(s.sceneId)+':'+str(s.worldEpoch) or not camera.initialized or not art._load_once() or _belt_texture()==null: return false
	var source=_catalog(s.sceneId,hash)
	return source!=null and source.source_hash==hash and source.current==scene and floor_material._load_texture()!=null

func _uv(u: float,v: float) -> Vector2:
	return (Vector2(510,200)+Vector2(460,260)*u+Vector2(-370,280)*v)/Vector2(1197,1132)

func _floor(out: Array,rect: Rect2,project: Callable,art,texture: Texture2D,offset: Vector3) -> bool:
	var z=rect.position.y
	while z<rect.end.y-0.00001:
		var z1=minf(rect.end.y,(floor(z/2.0)+1.0)*2.0)
		if z1<=z: return false
		var x=rect.position.x
		while x<rect.end.x-0.00001:
			var x1=minf(rect.end.x,(floor(x/2.0)+1.0)*2.0)
			if x1<=x: return false
			var u=(x-floor(x/2.0)*2.0)/2.0;var u1=u+(x1-x)/2.0
			var v=(z-floor(z/2.0)*2.0)/2.0;var v1=v+(z1-z)/2.0
			if not art._piece(out,project,[Vector3(x,0,z),Vector3(x1,0,z),Vector3(x1,0,z1),Vector3(x,0,z1)],PackedVector2Array([_uv(u,v),_uv(u1,v),_uv(u1,v1),_uv(u,v1)]),texture,{'role':'ground'}): return false
			out[-1].worldAnchor+=offset;x=x1
		z=z1
	return true

func _height(scene_id: String,id: String) -> float:
	var high=id.ends_with('_wall_north') or id.contains('_wall_west_')
	if scene_id=='cw_entry_foundry': return 1.0 if high else 0.18
	if scene_id=='cw_boiler_chamber': return 0.75 if high else 0.22
	return 0.25

func _pack(selected: Dictionary,clip: Rect2,project: Callable,art,floor_texture,wall_art,offset: Vector3,pressure_owned: bool) -> Dictionary:
	var rows=selected.get('collision');var id=str(selected.get('sceneId',''))
	if not EDGES.has(id) or not P.rows(rows,64) or rows.size()!=6 or not P.unique(rows,'id') or not selected.get('walkablePolygons',[]) is Array or not selected.get('walkablePolygons',[]).is_empty(): return {}
	var floor: Array=[];var walls: Array=[]
	if not _floor(floor,clip,project,art,floor_texture,offset): return {}
	for suffix in ['north','south','west_north','west_south','east_north','east_south']:
		var wall_id=id+'_wall_'+suffix
		var matches=rows.filter(func(row): return row is Dictionary and row.get('id')==wall_id)
		if matches.size()!=1 or matches[0].get('requiresEvent')!=null or matches[0].get('requiresActorFirstKill')!=null: return {}
		var original: Rect2=art._rect(matches[0].get('polygon'))
		if original.size.x<=0 or original.size.y<=0: return {}
		if pressure_owned:
			if suffix.begins_with('west_') or suffix.begins_with('east_'):
				var x: float=original.end.x if suffix.begins_with('west_') else original.position.x
				var z=original.position.y
				while z<original.end.y:
					var z1=minf(original.end.y,(floor(z/0.5)+1.0)*0.5)
					if z1<=z: return {}
					var uv=PackedVector2Array([Vector2(0,0.5),Vector2((z1-z)/0.5,0.5),Vector2((z1-z)/0.5,0),Vector2(0,0)])
					if not art._piece(walls,project,[Vector3(x,0,z),Vector3(x,0,z1),Vector3(x,0.25,z1),Vector3(x,0.25,z)],uv,art._steel,{'role':'wall','id':wall_id}): return {}
					walls[-1].worldAnchor+=offset;z=z1
			continue
		var part: Rect2=original.intersection(clip)
		if not part.has_area(): continue
		var start=walls.size()
		if not wall_art._wall_fragments(walls,project,original,part,clip,_height(id,wall_id),wall_id,art): return {}
		for n in range(start,walls.size()): walls[n].worldAnchor+=offset
	return {'floor':floor,'walls':walls}

func current(s: Dictionary,scene: Dictionary,key: String,hash: String,camera,art,floor_material,wall_art) -> Dictionary:
	if not _context(s,scene,key,hash,camera,art,floor_material): return {}
	var plan=_pack(scene,Rect2(0,0,24,16),Callable(camera,'project'),art,floor_material._load_texture(),wall_art,Vector3.ZERO,s.sceneId=='cw_pressure_hall' and s.get('entryToken')==null)
	if plan.is_empty(): return {}
	if s.sceneId=='cw_conveyor_bridge':
		var rows=scene.get('terrainRegions')
		if not rows is Array or rows.size()!=1 or not rows[0] is Dictionary or rows[0].get('id')!='cw_conveyor_belt' or rows[0].get('tag')!='conveyor' or rows[0].get('surfaceVelocityMps')!=[-1.2,0.0] or rows[0].get('polygon')!=[[6.0,6.0],[18.0,6.0],[18.0,10.0],[6.0,10.0]] or not P.counter(s.get('serverTimeMs')): return {}
		var phase=fposmod(-1.2*float(s.serverTimeMs)/1000.0,0.5)/0.5
		var uv=PackedVector2Array([Vector2(12-phase,12),Vector2(36-phase,12),Vector2(36-phase,20),Vector2(12-phase,20)])
		if not art._piece(plan.floor,Callable(camera,'project'),[Vector3(6,0,6),Vector3(18,0,6),Vector3(18,0,10),Vector3(6,0,10)],uv,_belt_texture(),{'role':'conveyor'}): return {}
	return plan

func prepare(s: Dictionary,scene: Dictionary,key: String,hash: String,camera,art,floor_material,wall_art) -> Dictionary:
	if not _context(s,scene,key,hash,camera,art,floor_material): return {}
	var v=s.get('hiveRouteNeighbor')
	if not v is Dictionary or v.size()!=9 or v.get('sourceSceneId')!=s.sceneId or v.get('worldEpoch')!=s.worldEpoch or v.get('serverTick')!=s.serverTick or v.get('authorityRevision')!=s.authorityRevision or not P.token(v.get('token')) or typeof(v.get('ready'))!=TYPE_BOOL or typeof(v.get('straddling'))!=TYPE_BOOL: return {}
	if not v.get('targetSceneSourceSha256') is String or v.targetSceneSourceSha256.length()!=64 or v.token.worldId!='clockworks' or v.token.sceneId!=v.get('targetSceneId') or v.token.worldEpoch!=s.worldEpoch+1: return {}
	var edge: Array=EDGES[s.sceneId];var east=v.targetSceneId==edge[1] and not edge[1].is_empty()
	if not east and (v.targetSceneId!=edge[0] or edge[0].is_empty()): return {}
	var target=_catalog(v.targetSceneId,v.targetSceneSourceSha256)
	if target==null or target.source_hash!=v.targetSceneSourceSha256: return {}
	var offset=Vector3(24 if east else -24,0,0)
	var project: Callable=func(p: Vector3)->Vector2: return camera.project(p+offset)
	var target_rect=Rect2(0.0 if east else 22.0,5.5,2.0,4.5)
	var source_rect=Rect2(22.0 if east else 0.0,5.5,2.0,4.5)
	var target_plan=_pack(target.current,target_rect,project,art,floor_material._load_texture(),wall_art,offset,false)
	var source_plan=_pack(scene,source_rect,Callable(camera,'project'),art,floor_material._load_texture(),wall_art,Vector3.ZERO,false)
	if target_plan.is_empty() or source_plan.is_empty(): return {}
	return {'floor':target_plan.floor,'walls':target_plan.walls,'reciprocal':source_plan,'token':v.token.duplicate(true),'targetHash':v.targetSceneSourceSha256,'sourceHash':hash,'source':s.sceneId,'target':v.targetSceneId,'epoch':s.worldEpoch,'player':s.player.entityId,'offset':offset}

func boundary(s: Dictionary,scene: Dictionary,key: String,hash: String,camera,art,floor_material) -> Dictionary:
	if not _context(s,scene,key,hash,camera,art,floor_material): return {}
	var v=s.get('hiveRouteBoundary')
	if not v is Dictionary or v.size()!=6 or v.get('closed')!=true or typeof(v.get('east'))!=TYPE_BOOL or v.get('sourceSceneId')!=s.sceneId or v.get('worldEpoch')!=s.worldEpoch or v.get('sourceSceneSourceSha256')!=hash: return {}
	var target: String=EDGES[s.sceneId][1 if v.east else 0]
	if target.is_empty() or v.get('targetSceneId')!=target: return {}
	# The source boundary remains the native bounds; no authored shutter exists.
	return {'walls':[],'boundary':v.duplicate(true)}
