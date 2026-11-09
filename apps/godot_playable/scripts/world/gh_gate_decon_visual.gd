extends RefCounted

# Six actual collision owners, not a room image or a new collision plan.
const P = preload('res://scripts/bridge/protocol.gd')
const Catalog = preload('res://scripts/world/scene_catalog.gd')
const LOCKDOWN_DENSE_SHA = '23e5d5d9f37f6f44b1f8b9105aab656424778ef5edf0e775bbc364143fe1b30c'
const LOCKDOWN_PARTITIONS = {
	'north_west_front_segment_0':Rect2(0.5,6.2,2.5,0.3), 'north_west_front_segment_1':Rect2(5,6.2,2.75,0.3), 'north_west_front_segment_2':Rect2(9.75,6.2,0.25,0.3),
	'north_east_front_segment_3':Rect2(11,6.2,2,0.3), 'north_east_front_segment_4':Rect2(15,6.2,5,0.3), 'north_east_front_segment_5':Rect2(22,6.2,1.5,0.3),
	'south_west_front_segment_6':Rect2(0.5,9.5,1.5,0.3), 'south_west_front_segment_7':Rect2(4,9.5,3.5,0.3), 'south_west_front_segment_8':Rect2(9.5,9.5,0.5,0.3),
	'south_east_front_segment_9':Rect2(11,9.5,2,0.3), 'south_east_front_segment_10':Rect2(15,9.5,5,0.3), 'south_east_front_segment_11':Rect2(22,9.5,1.5,0.3),
	'north_west_cross_segment_12':Rect2(7.35,0.5,0.3,1.5), 'north_west_cross_segment_13':Rect2(7.35,4,0.3,2.2),
	'north_east_cross_segment_14':Rect2(17.35,0.5,0.3,1.5), 'north_east_cross_segment_15':Rect2(17.35,4,0.3,2.2),
	'south_west_cross_segment_16':Rect2(5.35,9.8,0.3,2.2), 'south_west_cross_segment_17':Rect2(5.35,14,0.3,1.5),
	'south_east_cross_segment_18':Rect2(18.35,9.8,0.3,2.2), 'south_east_cross_segment_19':Rect2(18.35,14,0.3,1.5),
}
const RECTS = {
	'north':Rect2(0,0,24,0.5), 'south':Rect2(0,15.5,24,0.5),
	'west_north':Rect2(0,0.5,0.5,6), 'west_south':Rect2(0,9.5,0.5,6),
	'east_north':Rect2(23.5,0.5,0.5,6), 'east_south':Rect2(23.5,9.5,0.5,6),
}
var catalogs: Dictionary = {}
var attempted: Dictionary = {}

func _rects(scene: Dictionary) -> Dictionary:
	if scene.get('worldId')!='grey_hive' or not scene.get('sceneId') in ['gh_gate_b','gh_deep_decon','gh_lockdown']: return {}
	var bounds=scene.get('boundsM')
	if not bounds is Dictionary or bounds.get('x')!=0 or bounds.get('z')!=0 or bounds.get('width')!=24 or bounds.get('depth')!=16: return {}
	if not scene.get('collision') is Array: return {}
	var prefix: String = 'gh_gate_b_wall_' if scene.sceneId=='gh_gate_b' else ('lockdown_wall_' if scene.sceneId=='gh_lockdown' else 'deep_decon_wall_')
	var found: Dictionary = {}
	for row in scene.collision:
		if not row is Dictionary or not row.get('id') is String or not row.id.begins_with(prefix): continue
		var suffix: String = row.id.substr(prefix.length())
		if not RECTS.has(suffix) or found.has(row.id) or not row.get('polygon') is Array or row.polygon.size()!=4: return {}
		var r: Rect2 = RECTS[suffix]
		var expected: Array = [r.position,Vector2(r.end.x,r.position.y),r.end,Vector2(r.position.x,r.end.y)]
		for i in range(4):
			var p=row.polygon[i]
			if not p is Array or p.size()!=2 or not P.number(p[0]) or not P.number(p[1]) or Vector2(float(p[0]),float(p[1]))!=expected[i]: return {}
		found[row.id]=r
	return found if found.size()==6 else {}

func _catalog(scene_id: String, source_hash: String = ''):
	var cache_key=scene_id+':'+source_hash
	if not attempted.has(cache_key):
		attempted[cache_key]=true
		var catalog=Catalog.new()
		if catalog.build('grey_hive',scene_id,source_hash): catalogs[cache_key]=catalog
	return catalogs.get(cache_key)

func prepare(snapshot: Dictionary,scene: Dictionary,key: String,scene_hash: String,camera,art,barrier_open: Variant,lockdown_visual = null) -> Dictionary:
	if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or key!='grey_hive:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch) or not camera.initialized or not art._load_once(): return {}
	var current=_catalog(snapshot.sceneId,scene_hash)
	if current==null or scene_hash.is_empty() or current.source_hash!=scene_hash: return {}
	var rects: Dictionary = _rects(scene)
	if rects.is_empty(): return {}
	if snapshot.sceneId=='gh_gate_b':
		if typeof(barrier_open)!=TYPE_BOOL: return {}
		var barriers=scene.collision.filter(func(row): return row is Dictionary and row.get('id')=='gh_gate_b_gate_barrier')
		if barriers.size()!=1 or barriers[0].get('requiresEvent')!='hive_lockdown' or barriers[0].get('polygon')!=[[11.5,0.5],[12.5,0.5],[12.5,15.5],[11.5,15.5]]: return {}
		if not barrier_open: rects['gh_gate_b_gate_barrier']=Rect2(11.5,0.5,1,15)
	elif snapshot.sceneId=='gh_lockdown':
		if typeof(barrier_open)!=TYPE_BOOL: return {}
		var seals=scene.collision.filter(func(row): return row is Dictionary and row.get('id')=='gh_lockdown_bio_seal')
		if seals.size()!=1 or seals[0].get('requiresEvent')!='hive_lockdown': return {}
		var poly=seals[0].get('polygon')
		var expected=[[10.0,0.5],[11.0,0.5],[11.0,15.5],[10.0,15.5]]
		if not poly is Array or poly.size()!=4: return {}
		for i in range(4):
			if not poly[i] is Array or poly[i].size()!=2 or not P.number(poly[i][0],expected[i][0],expected[i][0]) or not P.number(poly[i][1],expected[i][1],expected[i][1]): return {}
		if not barrier_open: rects['gh_lockdown_bio_seal']=Rect2(10,0.5,1,15)
		if scene_hash==LOCKDOWN_DENSE_SHA:
			for id in LOCKDOWN_PARTITIONS:
				var matches=scene.collision.filter(func(row): return row is Dictionary and row.get('id')==id)
				if matches.size()!=1 or matches[0].get('requiresEvent')!=null: return {}
				var r: Rect2 = LOCKDOWN_PARTITIONS[id]
				var partition_poly=matches[0].get('polygon')
				var corners=[r.position,Vector2(r.end.x,r.position.y),r.end,Vector2(r.position.x,r.end.y)]
				if not partition_poly is Array or partition_poly.size()!=4: return {}
				for i in range(4):
					if not partition_poly[i] is Array or partition_poly[i].size()!=2 or not P.number(partition_poly[i][0]) or not P.number(partition_poly[i][1]) or not is_equal_approx(float(partition_poly[i][0]),corners[i].x) or not is_equal_approx(float(partition_poly[i][1]),corners[i].y): return {}
				rects[id]=r
	var floor: Array = []
	var phase := Vector2(0,-1) if snapshot.sceneId=='gh_lockdown' else Vector2.ZERO
	if not art._floor_rect(floor,Callable(camera,'project'),Rect2(0,0,24,16),0.0,{'floor':'upper','role':'ground'},phase): return {}
	var owners: Dictionary = {}
	for id in rects:
		var r: Rect2 = rects[id]
		var pieces: Array = []
		var height: float = 0.35
		if snapshot.sceneId=='gh_lockdown':
			if lockdown_visual==null or not lockdown_visual.has_method('_wall_spec'): return {}
			var spec=lockdown_visual._wall_spec('lockdown_wall_north' if id=='gh_lockdown_bio_seal' or LOCKDOWN_PARTITIONS.has(id) else id)
			if not spec is Dictionary or not P.number(spec.get('height'),0.01,1.0): return {}
			height=float(spec.height)
		if not art._wall(pieces,Callable(camera,'project'),r,0.0,height,{'role':'wall','ownerRegions':['upper']}): return {}
		var x: float = r.position.x
		while x<r.end.x-0.00001:
			var x1: float = minf(r.end.x,x+0.5)
			var z: float = r.position.y
			while z<r.end.y-0.00001:
				var z1: float = minf(r.end.y,z+0.5)
				var uv=PackedVector2Array([Vector2(0,0),Vector2((x1-x)/0.5,0),Vector2((x1-x)/0.5,(z1-z)/0.5),Vector2(0,(z1-z)/0.5)])
				if not art._piece(pieces,Callable(camera,'project'),[Vector3(x,height,z),Vector3(x1,height,z),Vector3(x1,height,z1),Vector3(x,height,z1)],uv,art._steel,{'role':'cap'}): return {}
				z=z1
			x=x1
		owners[id]=pieces
	return {'floor':floor,'owners':owners,'key':key,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,'revision':snapshot.authorityRevision,'player':snapshot.player.entityId}

func draw_floor(canvas: CanvasItem,plan: Dictionary) -> bool:
	if not is_instance_valid(canvas) or not plan.get('floor') is Array or plan.floor.is_empty(): return false
	for piece in plan.floor: canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,piece.texture)
	return true

func draw_owner(canvas: CanvasItem,plan: Dictionary,id: String) -> bool:
	if not is_instance_valid(canvas) or not plan.get('owners') is Dictionary or not plan.owners.has(id): return false
	for piece in plan.owners[id]: canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,piece.texture)
	return true

# When power is absent, the real scene boundary holds this aperture closed.
# Draw the boundary plane only; no geometry or transition is changed here.
func shaft_lockdown_seal(snapshot: Dictionary,scene: Dictionary,key: String,scene_hash: String,camera,art,lockdown_visual) -> Dictionary:
	if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or not snapshot.sceneId in ['gh_central_shaft','gh_lockdown'] or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or key!='grey_hive:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch) or not camera.initialized or not art._load_once(): return {}
	var source=_catalog(snapshot.sceneId,scene_hash)
	if source==null or source.source_hash!=scene_hash or not scene.get('transitions') is Array: return {}
	var from_shaft: bool = snapshot.sceneId=='gh_central_shaft'
	var id: String = 'gh_shaft_to_lockdown' if from_shaft else 'gh_lockdown_return_to_shaft'
	var target: String = 'gh_lockdown' if from_shaft else 'gh_central_shaft'
	var rows=scene.transitions.filter(func(row): return row is Dictionary and row.get('id')==id and row.get('toSceneId')==target and row.get('requiresEvent')=='hive_power')
	if rows.size()!=1: return {}
	var expected: Array = [[22.5,7.0],[24.0,7.0],[24.0,9.0],[22.5,9.0]] if from_shaft else [[0.0,7.0],[1.5,7.0],[1.5,9.0],[0.0,9.0]]
	var polygon=rows[0].get('polygon')
	if not polygon is Array or polygon.size()!=4: return {}
	for i in range(4):
		if not polygon[i] is Array or polygon[i].size()!=2 or not P.number(polygon[i][0],expected[i][0],expected[i][0]) or not P.number(polygon[i][1],expected[i][1],expected[i][1]): return {}
	if lockdown_visual==null or not lockdown_visual.has_method('_wall_spec'): return {}
	var spec=lockdown_visual._wall_spec('lockdown_wall_north')
	if not spec is Dictionary or not P.number(spec.get('height'),0.01,1.0): return {}
	var height: float = float(spec.height)
	var x: float = 24.0 if from_shaft else 0.0
	var pieces: Array = []
	var z: float = 6.5
	while z<9.5-0.00001:
		var z1: float = minf(9.5,z+0.5)
		var y: float = 0.0
		while y<height-0.00001:
			var y1: float = minf(height,y+0.5)
			var uv=PackedVector2Array([Vector2(0,(y1-y)/0.5),Vector2((z1-z)/0.5,(y1-y)/0.5),Vector2((z1-z)/0.5,0),Vector2(0,0)])
			if not art._piece(pieces,Callable(camera,'project'),[Vector3(x,y,z),Vector3(x,y,z1),Vector3(x,y1,z1),Vector3(x,y1,z)],uv,art._steel,{'role':'seal'}): return {}
			y=y1
		z=z1
	return {'floor':pieces} if not pieces.is_empty() else {}

func preview(snapshot: Dictionary,scene: Dictionary,key: String,camera,art,utility = null) -> Dictionary:
	if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or not snapshot.sceneId in ['gh_gate_b','gh_deep_decon'] or scene.get('sceneId')!=snapshot.sceneId or key!='grey_hive:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch) or not camera.initialized: return {}
	var v=snapshot.get('gateDeconNeighbor')
	var target: String = 'gh_deep_decon' if snapshot.sceneId=='gh_gate_b' else 'gh_gate_b'
	if not v is Dictionary or v.size()!=6 or v.get('sourceSceneId')!=snapshot.sceneId or v.get('targetSceneId')!=target or not P.counter(v.get('worldEpoch')) or not P.counter(v.get('serverTick')) or not P.counter(v.get('authorityRevision')) or v.worldEpoch!=snapshot.worldEpoch or v.serverTick!=snapshot.serverTick or v.authorityRevision!=snapshot.authorityRevision: return {}
	if not v.get('targetSceneSourceSha256') is String: return {}
	var catalog=_catalog(target,v.targetSceneSourceSha256)
	if catalog==null or catalog.source_hash!=v.get('targetSceneSourceSha256') or _rects(catalog.current).is_empty() or not art._load_once(): return {}
	var strip := Rect2(0,7,2,2) if target=='gh_deep_decon' else Rect2(22,7,2,2)
	for row in catalog.current.collision:
		if not row is Dictionary or not row.get('polygon') is Array or row.polygon.size()<3: return {}
		var min_x: float = INF
		var min_z: float = INF
		var max_x: float = -INF
		var max_z: float = -INF
		for point in row.polygon:
			if not point is Array or point.size()!=2 or not P.number(point[0]) or not P.number(point[1]): return {}
			min_x=minf(min_x,float(point[0])); max_x=maxf(max_x,float(point[0]))
			min_z=minf(min_z,float(point[1])); max_z=maxf(max_z,float(point[1]))
		if Rect2(min_x,min_z,max_x-min_x,max_z-min_z).intersects(strip): return {}
	var offset := Vector3(24,0,0) if target=='gh_deep_decon' else Vector3(-24,0,0)
	var project: Callable = func(p: Vector3) -> Vector2: return camera.project(p+offset)
	if utility!=null and utility.is_new(target,catalog.source_hash) and utility.prepare(catalog.current,catalog.source_hash,project,art).is_empty(): return {}
	var pieces: Array = []
	if not art._floor_rect(pieces,project,strip,0.0,{'floor':'upper','role':'ground'}) or pieces.is_empty(): return {}
	return {'floor':pieces}
