extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const Catalog = preload('res://scripts/world/scene_catalog.gd')
const WorldProjection = preload('res://scripts/world/projection.gd')
var catalogs: Dictionary = {}

func _wall_faces(out: Array, project: Callable, art, original: Rect2, visible: Rect2, height: float) -> bool:
	var a=visible.position;var b=visible.end
	var sides=[ [Vector2(a.x,a.y),Vector2(a.x,b.y),is_equal_approx(a.x,original.position.x)], [Vector2(b.x,a.y),Vector2(b.x,b.y),is_equal_approx(b.x,original.end.x)], [Vector2(a.x,a.y),Vector2(b.x,a.y),is_equal_approx(a.y,original.position.y)], [Vector2(a.x,b.y),Vector2(b.x,b.y),is_equal_approx(b.y,original.end.y)] ]
	for side in sides:
		if not side[2]: continue
		var p0: Vector2=side[0];var p1: Vector2=side[1];var length: float=p0.distance_to(p1);var n:=0.0
		while n<length-0.00001:
			var n1: float=minf(length,n+0.5);var q0: Vector2=p0.lerp(p1,n/length);var q1: Vector2=p0.lerp(p1,n1/length)
			var start: float=q0.y if is_equal_approx(q0.x,q1.x) else q0.x
			var u0: float=fposmod(start,0.5)/0.5;var u1: float=u0+(n1-n)/0.5;var y:=0.0
			while y<height-0.00001:
				var y1: float=minf(height,y+0.5);var v0: float=1.0-fposmod(y,0.5)/0.5;var v1: float=v0-(y1-y)/0.5
				if not art._piece(out,project,[Vector3(q0.x,y,q0.y),Vector3(q1.x,y,q1.y),Vector3(q1.x,y1,q1.y),Vector3(q0.x,y1,q0.y)],PackedVector2Array([Vector2(u0,v0),Vector2(u1,v0),Vector2(u1,v1),Vector2(u0,v1)]),art._steel,{'role':'neighbor_wall'}): return false
				y=y1
			n=n1
	return true

func _catalog(id: String, hash: String):
	var key=id+':'+hash
	if not catalogs.has(key):
		var c=Catalog.new()
		if not c.build('grey_hive',id,hash): return null
		catalogs[key]=c
	return catalogs[key]

# The native view authorizes static geometry only. The source snapshot and
# canvas remain untouched; no target actors or interactions are projected.
func prepare(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, source_hash: String, key: String, camera, viewport: Rect2, art, exit_visual, exit_legacy, terminal, crate, beacon_visual, utility, props) -> Dictionary:
	if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or not snapshot.sceneId in ['gh_beacon','gh_exit'] or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!=snapshot.sceneId or key!='grey_hive:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch) or not camera.initialized: return {}
	var v=snapshot.get('beaconExitNeighbor')
	var target: String='gh_exit' if snapshot.sceneId=='gh_beacon' else 'gh_beacon'
	if not v is Dictionary or v.size()!=9 or v.get('sourceSceneId')!=snapshot.sceneId or v.get('targetSceneId')!=target or v.get('worldEpoch')!=snapshot.worldEpoch or v.get('serverTick')!=snapshot.serverTick or v.get('authorityRevision')!=snapshot.authorityRevision or typeof(v.get('ready'))!=TYPE_BOOL or typeof(v.get('straddling'))!=TYPE_BOOL or not P.token(v.get('token')): return {}
	if v.token.worldId!='grey_hive' or v.token.sceneId!=target or v.token.worldEpoch!=snapshot.worldEpoch+1 or not v.get('targetSceneSourceSha256') is String: return {}
	var catalog=_catalog(target,v.targetSceneSourceSha256)
	if catalog==null or catalog.source_hash!=v.targetSceneSourceSha256 or not art._load_once(): return {}
	var source_catalog=_catalog(snapshot.sceneId,source_hash)
	if source_catalog==null or source_catalog.source_hash!=source_hash or source_catalog.current!=scene: return {}
	var offset=Vector3(24,0,0) if target=='gh_exit' else Vector3(-24,0,0)
	var projection=WorldProjection.new()
	projection.origin=camera.origin-offset;projection.viewport=camera.viewport;projection.screen_center=camera.screen_center;projection.ppm=camera.ppm;projection.initialized=camera.initialized;projection.boss_scale=camera.boss_scale
	if target=='gh_exit':
		if exit_visual.prepare(snapshot,catalog.current,key,v.targetSceneSourceSha256,projection,art,exit_legacy,terminal,crate,v).is_empty(): return {}
	else:
		if not beacon_visual.neighbor_static_ready(canvas,snapshot,catalog.current,v.targetSceneSourceSha256,key,projection,viewport,v,utility,art,props): return {}
	var target_strip=_strip(catalog.current,target,projection,art)
	var source_strip=_strip(scene,snapshot.sceneId,camera,art)
	if target_strip.is_empty() or source_strip.is_empty(): return {}
	return {'floor':target_strip.floor,'walls':target_strip.walls,'reciprocal':source_strip,'token':v.token,'hash':v.targetSceneSourceSha256,'ready':v.ready}

func _strip(selected: Dictionary, target: String, projection, art) -> Dictionary:
	var floor_rect=Rect2(0,6.5,2,3) if target=='gh_exit' else Rect2(22,6.5,2,3)
	var wall_rect=Rect2(0,6,2,4) if target=='gh_exit' else Rect2(22,6,2,4)
	var floor_pieces: Array=[]
	if not art._floor_rect(floor_pieces,Callable(projection,'project'),floor_rect,0.0,{'role':'ground'}) or floor_pieces.is_empty(): return {}
	var walls: Array=[]
	for row in selected.get('collision',[]):
		if not row is Dictionary or not row.get('id') is String: return {}
		var r: Rect2=art._rect(row.get('polygon'))
		if not r.has_area(): return {}
		var visible=r.intersection(wall_rect)
		if not visible.has_area(): continue
		if row.get('requiresEvent')!=null or row.get('requiresActorFirstKill')!=null: return {}
		var height: float = (0.5 if row.id.begins_with('exit_wall_') else 0.25) if target=='gh_exit' else 0.25
		if not _wall_faces(walls,Callable(projection,'project'),art,r,visible,height): return {}
		var x: float=visible.position.x
		while x<visible.end.x-0.00001:
			var x1: float=minf(visible.end.x,(floor(x/0.5)+1.0)*0.5)
			if x1<=x: return {}
			var z: float=visible.position.y
			while z<visible.end.y-0.00001:
				var z1: float=minf(visible.end.y,(floor(z/0.5)+1.0)*0.5)
				if z1<=z: return {}
				var u0: float=fposmod(x,0.5)/0.5;var u1: float=u0+(x1-x)/0.5
				var v0: float=fposmod(z,0.5)/0.5;var v1: float=v0+(z1-z)/0.5
				if not art._piece(walls,Callable(projection,'project'),[Vector3(x,height,z),Vector3(x1,height,z),Vector3(x1,height,z1),Vector3(x,height,z1)],PackedVector2Array([Vector2(u0,v0),Vector2(u1,v0),Vector2(u1,v1),Vector2(u0,v1)]),art._steel,{'role':'neighbor_cap'}): return {}
				z=z1
			x=x1
	return {'floor':floor_pieces,'walls':walls}

func draw(canvas: CanvasItem, plan: Dictionary) -> bool:
	if not is_instance_valid(canvas) or not plan.get('floor') is Array or plan.floor.is_empty() or not plan.get('walls') is Array: return false
	for piece in plan.floor+plan.walls: canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,piece.texture)
	return true

# Reproject only the already-authorized static strip. Translation and uniform
# zoom are exact for the shared parallel projection; no target state is added.
func reproject(plan: Dictionary, old_origin: Vector3, old_center: Vector2, old_ppm: float, camera, origin_shift: Vector3) -> Dictionary:
	if not plan.get('floor') is Array or plan.floor.is_empty() or not plan.get('walls') is Array or not old_origin.is_finite() or not old_center.is_finite() or not is_finite(old_ppm) or old_ppm<=0.0 or not camera.initialized or not is_finite(camera.ppm) or camera.ppm<=0.0: return {}
	var anchor: Vector2=camera.project(old_origin+origin_shift)
	if not anchor.is_finite(): return {}
	var scale: float=camera.ppm/old_ppm
	var result={'floor':[],'walls':[]}
	for group in ['floor','walls']:
		for piece in plan[group]:
			if not piece is Dictionary or not piece.get('points') is PackedVector2Array or not piece.get('uv') is PackedVector2Array or not piece.get('texture') is Texture2D or not is_instance_valid(piece.texture): return {}
			var copy: Dictionary=piece.duplicate()
			var points: PackedVector2Array=[]
			for p in piece.points:
				var q=anchor+(p-old_center)*scale
				if not q.is_finite(): return {}
				points.append(q)
			copy.points=points;result[group].append(copy)
	return result
