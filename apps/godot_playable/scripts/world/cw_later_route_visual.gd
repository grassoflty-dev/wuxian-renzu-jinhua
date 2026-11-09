extends "res://scripts/world/cw_early_route_visual.gd"
# Only registered same-floor pairs; Gear east remains its authored Y2 transfer.
const LATER_EDGES={'cw_boiler_chamber':['','cw_gear_shaft'],'cw_gear_shaft':['cw_boiler_chamber',''],'cw_furnace_heart':['','cw_forged_guard_arena'],'cw_forged_guard_arena':['cw_furnace_heart','cw_regulator_core'],'cw_regulator_core':['cw_forged_guard_arena','cw_shutdown_exit'],'cw_shutdown_exit':['cw_regulator_core','']}
const PREFIX={'cw_boiler_chamber':'cw_boiler_chamber','cw_gear_shaft':'cw_gear_shaft','cw_furnace_heart':'cw_furnace_heart','cw_forged_guard_arena':'cw_forged_guard_arena','cw_regulator_core':'cw_regulator_core','cw_shutdown_exit':'cw_shutdown'}

func _later_context(s: Dictionary,scene: Dictionary,key: String,hash: String,camera,art,material) -> bool:
	if not P.snapshot(s) or s.worldId!='clockworks' or not LATER_EDGES.has(s.sceneId) or scene.get('sceneId')!=s.sceneId or key!='clockworks:'+str(s.sceneId)+':'+str(s.worldEpoch) or not camera.initialized or not art._load_once(): return false
	var source=_catalog(s.sceneId,hash)
	return source!=null and source.source_hash==hash and source.current==scene and material._load_texture()!=null

func _later_height(id: String,wall_id: String) -> float:
	var high=wall_id.ends_with('_wall_north') or wall_id.contains('_wall_west_')
	if id=='cw_gear_shaft': return 0.65 if high else 0.18
	if id=='cw_boiler_chamber': return 0.75 if high else 0.22
	return 1.0 if high else 0.18

func _surface(out: Array,rect: Rect2,height: float,project: Callable,art,texture: Texture2D,offset: Vector3) -> bool:
	var first=out.size()
	var elevated: Callable=func(p: Vector3)->Vector2: return project.call(p+Vector3(0,height,0))
	if not _floor(out,rect,elevated,art,texture,offset): return false
	for index in range(first,out.size()): out[index].worldAnchor+=Vector3(0,height,0)
	var bottom=maxf(0.0,height-0.12)
	if height>bottom:
		var x0=rect.position.x;var x1=rect.end.x;var z0=rect.position.y;var z1=rect.end.y
		for edge in [[Vector3(x1,bottom,z0),Vector3(x1,bottom,z1),Vector3(x1,height,z1),Vector3(x1,height,z0)],[Vector3(x0,bottom,z1),Vector3(x1,bottom,z1),Vector3(x1,height,z1),Vector3(x0,height,z1)]]:
			var u=PackedVector2Array([Vector2(0,1),Vector2(1,1),Vector2(1,0),Vector2(0,0)])
			if not art._piece(out,project,edge,u,art._steel,{'role':'support_fascia'}): return false
			out[-1].worldAnchor+=offset
	return true

func _later_pack(selected: Dictionary,clip: Rect2,project: Callable,art,texture: Texture2D,wall_art,offset: Vector3,shutters_open: bool,excluded: String,gear_frame: Variant,gear) -> Dictionary:
	var rows=selected.get('collision');var id=str(selected.get('sceneId',''))
	var count=5 if id=='cw_shutdown_exit' else (8 if id=='cw_forged_guard_arena' else 6)
	if not PREFIX.has(id) or not P.rows(rows,64) or rows.size()!=count or not P.unique(rows,'id') or not selected.get('walkablePolygons',[]) is Array or not selected.get('walkablePolygons',[]).is_empty(): return {}
	var ground: Array=[];var walls: Array=[]
	if not _floor(ground,clip,project,art,texture,offset): return {}
	var suffixes=['north','south','west_north','west_south','east'] if id=='cw_shutdown_exit' else ['north','south','west_north','west_south','east_north','east_south']
	for suffix in suffixes:
		var wall_id=PREFIX[id]+'_wall_'+suffix
		var found=rows.filter(func(row): return row is Dictionary and row.get('id')==wall_id)
		if found.size()!=1 or found[0].get('requiresEvent')!=null or found[0].get('requiresActorFirstKill')!=null: return {}
		var original: Rect2=art._rect(found[0].get('polygon'));var part: Rect2=original.intersection(clip)
		if original.size.x<=0 or original.size.y<=0: return {}
		if not part.has_area(): continue
		var start=walls.size()
		if not wall_art._wall_fragments(walls,project,original,part,clip,_later_height(id,wall_id),wall_id,art): return {}
		for n in range(start,walls.size()): walls[n].worldAnchor+=offset
	if id=='cw_forged_guard_arena':
		for pair in [['cw_arena_shutter_west_blocker',Rect2(0.5,6,0.5,3.5)],['cw_arena_shutter_east_blocker',Rect2(23,6,0.5,3.5)]]:
			var found=rows.filter(func(row): return row is Dictionary and row.get('id')==pair[0])
			if found.size()!=1 or found[0].get('requiresActorFirstKill')!='cw_forged_guard_elite' or art._rect(found[0].get('polygon'))!=pair[1]: return {}
			if shutters_open or excluded==pair[0]: continue
			var part: Rect2=pair[1].intersection(clip)
			if not part.has_area(): continue
			var start=walls.size()
			if not wall_art._wall_fragments(walls,project,pair[1],part,clip,0.25,pair[0],art): return {}
			for n in range(start,walls.size()): walls[n].worldAnchor+=offset
	if id=='cw_gear_shaft' and clip.position.x<=10 and clip.end.x>=15:
		if not gear_frame is Dictionary or not gear_frame.get('poses') is Array or not P.counter(gear_frame.get('serverTimeMs')): return {}
		var poses: Dictionary={}
		for pose in gear_frame.poses:
			if not pose is Dictionary or typeof(pose.get('supportId'))!=TYPE_STRING or poses.has(pose.supportId) or not gear._pose_values_match(pose,pose.supportId,gear_frame): return {}
			poses[pose.supportId]=pose
		if poses.size()!=3 or not poses.has('cw_gear_main_lift') or not poses.has('cw_gear_upper_dock') or not poses.has('cw_gear_furnace_landing'): return {}
		if not gear._same(selected.get('movingSupports'),gear.MOVING_SUPPORTS) or not gear._same(selected.get('standingDecks'),gear.STANDING_DECKS): return {}
		for sid in ['cw_gear_main_lift','cw_gear_upper_dock','cw_gear_furnace_landing']:
			var r=gear.SUPPORT_RECTANGLES[sid];var part=Rect2(r.x0,r.z0,r.x1-r.x0,r.z1-r.z0).intersection(clip)
			if part.has_area() and not _surface(walls,part,float(poses[sid].heightM),project,art,texture,offset): return {}
	return {'floor':ground,'walls':walls}

func later_current(s: Dictionary,scene: Dictionary,key: String,hash: String,camera,art,material,wall_art,gear) -> Dictionary:
	if not _later_context(s,scene,key,hash,camera,art,material): return {}
	if s.sceneId=='cw_gear_shaft':
		var f=s.get('supportScene')
		if not f is Dictionary or not f.get('poses') is Array: return {}
		var poses={}
		for pose in f.poses:
			if not pose is Dictionary or typeof(pose.get('supportId'))!=TYPE_STRING or poses.has(pose.supportId): return {}
			poses[pose.supportId]=pose
		if not gear._support_scene_valid(s,poses): return {}
	var open=false
	if s.sceneId=='cw_forged_guard_arena':
		var value=P.forged_guard_shutters_open(s)
		if typeof(value)!=TYPE_BOOL: return {}
		open=value
	return _later_pack(scene,Rect2(0,0,24,16),Callable(camera,'project'),art,material._load_texture(),wall_art,Vector3.ZERO,open,'',s.get('supportScene'),gear)

func later_prepare(s: Dictionary,scene: Dictionary,key: String,hash: String,camera,art,material,wall_art,gear) -> Dictionary:
	if not _later_context(s,scene,key,hash,camera,art,material): return {}
	var v=s.get('hiveRouteNeighbor')
	if not v is Dictionary or v.size() not in [9,10] or v.get('sourceSceneId')!=s.sceneId or v.get('worldEpoch')!=s.worldEpoch or v.get('serverTick')!=s.serverTick or v.get('authorityRevision')!=s.authorityRevision or not P.token(v.get('token')) or typeof(v.get('ready'))!=TYPE_BOOL or typeof(v.get('straddling'))!=TYPE_BOOL: return {}
	if not v.get('targetSceneSourceSha256') is String or v.targetSceneSourceSha256.length()!=64 or v.token.worldId!='clockworks' or v.token.sceneId!=v.get('targetSceneId') or v.token.worldEpoch!=s.worldEpoch+1: return {}
	var edge: Array=LATER_EDGES[s.sceneId];var east=v.targetSceneId==edge[1] and not edge[1].is_empty()
	if not east and (v.targetSceneId!=edge[0] or edge[0].is_empty()): return {}
	var target=_catalog(v.targetSceneId,v.targetSceneSourceSha256)
	if target==null or target.source_hash!=v.targetSceneSourceSha256: return {}
	var open=false;var excluded=''
	if v.targetSceneId=='cw_forged_guard_arena':
		if typeof(v.get('arenaShuttersOpen'))!=TYPE_BOOL: return {}
		open=v.arenaShuttersOpen
		if not open: excluded='cw_arena_shutter_west_blocker' if east else 'cw_arena_shutter_east_blocker'
	if v.targetSceneId=='cw_gear_shaft':
		var f=v.get('gearSupportFrame')
		if not f is Dictionary or f.size()!=4 or f.get('worldEpoch')!=v.token.worldEpoch or f.get('serverTimeMs')!=s.serverTimeMs or not f.get('catalogSha256') is String or f.catalogSha256.length()!=64 or not f.get('poses') is Array or f.poses.size()!=3: return {}
		if not gear._same(target.current.get('movingSupports'),gear.MOVING_SUPPORTS) or not gear._same(target.current.get('standingDecks'),gear.STANDING_DECKS): return {}
		var ids={}
		for pose in f.poses:
			if not pose is Dictionary or typeof(pose.get('supportId'))!=TYPE_STRING or ids.has(pose.supportId) or not gear._pose_values_match(pose,pose.supportId,f): return {}
			ids[pose.supportId]=true
		if not ids.has('cw_gear_main_lift') or not ids.has('cw_gear_upper_dock') or not ids.has('cw_gear_furnace_landing'): return {}
	var offset=Vector3(24 if east else -24,0,0)
	var project: Callable=func(p: Vector3)->Vector2: return camera.project(p+offset)
	var target_rect=Rect2(0 if east else 22,5.5,2,4.5);var source_rect=Rect2(22 if east else 0,5.5,2,4.5)
	var target_plan=_later_pack(target.current,target_rect,project,art,material._load_texture(),wall_art,offset,open,excluded,v.get('gearSupportFrame'),gear)
	var source_open=false
	if s.sceneId=='cw_forged_guard_arena':
		var value=P.forged_guard_shutters_open(s)
		if typeof(value)!=TYPE_BOOL: return {}
		source_open=value
	var source_plan=_later_pack(scene,source_rect,Callable(camera,'project'),art,material._load_texture(),wall_art,Vector3.ZERO,source_open,'',s.get('supportScene'),gear)
	if target_plan.is_empty() or source_plan.is_empty(): return {}
	return {'floor':target_plan.floor,'walls':target_plan.walls,'reciprocal':source_plan,'token':v.token.duplicate(true),'targetHash':v.targetSceneSourceSha256,'sourceHash':hash,'source':s.sceneId,'target':v.targetSceneId,'epoch':s.worldEpoch,'player':s.player.entityId,'offset':offset,'arenaState':v.get('arenaShuttersOpen')}

func later_boundary(s: Dictionary,scene: Dictionary,key: String,hash: String,camera,art,material) -> Dictionary:
	if not _later_context(s,scene,key,hash,camera,art,material): return {}
	var v=s.get('hiveRouteBoundary')
	if not v is Dictionary or v.size()!=6 or v.get('closed')!=true or typeof(v.get('east'))!=TYPE_BOOL or v.get('sourceSceneId')!=s.sceneId or v.get('worldEpoch')!=s.worldEpoch or v.get('sourceSceneSourceSha256')!=hash: return {}
	var target: String=LATER_EDGES[s.sceneId][1 if v.east else 0]
	if target.is_empty() or v.get('targetSceneId')!=target: return {}
	return {'walls':[],'boundary':v.duplicate(true)}