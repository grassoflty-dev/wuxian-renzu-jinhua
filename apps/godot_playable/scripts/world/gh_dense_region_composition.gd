extends RefCounted
# World-space region composition; no persistent state or movement authority.
const P = preload('res://scripts/bridge/protocol.gd')
const Content = preload('res://scripts/ui/content_model.gd')

func _inside(rect: Rect2, piece: Rect2) -> bool:
	return piece.position.x >= rect.position.x and piece.position.y >= rect.position.y and piece.end.x <= rect.end.x and piece.end.y <= rect.end.y

func _uv(uv: PackedVector2Array, u: float, v: float) -> Vector2:
	return uv[0].lerp(uv[1],u).lerp(uv[3].lerp(uv[2],u),v)

# All geometry and textures are prepared before drawing any region element.
func prepare(snapshot: Dictionary, scene: Dictionary, key: String, source_hash: String, camera, floor_material, neighbor, dense, neighbor_ready: bool) -> Dictionary:
	if not P.snapshot(snapshot) or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!=snapshot.get('sceneId') or key!='grey_hive:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch): return {}
	if not dense.HASHES.has(snapshot.sceneId) or dense.HASHES[snapshot.sceneId]!=source_hash or not camera.initialized: return {}
	var region: Dictionary = dense.current_region(snapshot)
	if region.is_empty() or not region.get('rect') is Rect2 or not neighbor_ready or not snapshot.get('denseEntityRegions') is Dictionary or not snapshot.get('densePartitionEscapeIds') is Array: return {}
	var adjacent = neighbor._neighbor(snapshot.sceneId)
	var metal = neighbor._wall_texture()
	if adjacent==null or not is_instance_valid(metal): return {}
	var shift = 20.0 if snapshot.sceneId=='gh_power_room' else -20.0
	if dense.prepare(snapshot,scene,key,scene,source_hash,0.0,camera,metal).is_empty() or dense.prepare(snapshot,scene,key,adjacent.current,adjacent.source_hash,shift,camera,metal).is_empty(): return {}
	var offset = 20.0 if snapshot.sceneId=='gh_gate_a' else 0.0
	var world: Rect2 = region.rect
	var local = Rect2(world.position-Vector2(offset,0),world.size)
	if local.size.x<=0 or local.size.y<=0: return {}
	var floor_packet = floor_material.build_packet(snapshot,scene,key,camera)
	if floor_packet.is_empty() or not floor_material.is_packet_valid(floor_packet,snapshot,scene,key,camera) or not floor_material._load_texture_once() or not is_instance_valid(floor_material.floor_texture): return {}
	var ids = ['gh_power_wall_north','gh_power_wall_south','gh_power_wall_west_north','gh_power_wall_west_south','gh_power_wall_east_north','gh_power_wall_east_south'] if snapshot.sceneId=='gh_power_room' else ['gh_gate_wall_north','gh_gate_wall_south','gh_gate_wall_west_north','gh_gate_wall_west_south','gh_gate_wall_east_north','gh_gate_wall_east_south','gh_gate_a_closed_barrier']
	var walls: Dictionary = neighbor.prepare_gate_walls(scene,camera) if snapshot.sceneId=='gh_gate_a' else neighbor._prepare_owners(scene,camera,ids)
	if walls.is_empty(): return {}
	var show_barrier = false
	if snapshot.sceneId=='gh_gate_a':
		if snapshot.progression.get('currentWorldId')!='grey_hive' or not P.unique(snapshot.progression.get('worlds'),'worldId'): return {}
		var progress: Dictionary = Content.progress(snapshot,'grey_hive')
		if progress.is_empty() or progress.visitId<=0: return {}
		show_barrier = not 'hive_power' in progress.completedEvents
	var floors: Array = []
	for iz in range(7):
		for ix in range(10):
			var tile=Rect2(ix*2.0,iz*2.0,2.0,2.0)
			var clipped: Rect2 = tile.intersection(local)
			if clipped.size.x<=0 or clipped.size.y<=0: continue
			var u0=(clipped.position.x-tile.position.x)/2.0; var u1=(clipped.end.x-tile.position.x)/2.0
			var v0=(clipped.position.y-tile.position.y)/2.0; var v1=(clipped.end.y-tile.position.y)/2.0
			var points=PackedVector2Array([camera.project(Vector3(clipped.position.x,0,clipped.position.y)),camera.project(Vector3(clipped.end.x,0,clipped.position.y)),camera.project(Vector3(clipped.end.x,0,clipped.end.y)),camera.project(Vector3(clipped.position.x,0,clipped.end.y))])
			for point in points:
				if not point.is_finite(): return {}
			floors.append({'points':points,'uv':PackedVector2Array([_uv(neighbor.UV,u0,v0),_uv(neighbor.UV,u1,v0),_uv(neighbor.UV,u1,v1),_uv(neighbor.UV,u0,v1)])})
	if floors.is_empty(): return {}
	var visible_walls: Array = []
	for id in ids:
		if id=='gh_gate_a_closed_barrier' and not show_barrier: continue
		for piece in walls.owners[id]:
			if not piece.get('world') is Rect2: return {}
			if _inside(local,piece.world): visible_walls.append(piece)
	return {'region':region,'local':local,'floors':floors,'floor_texture':floor_material.floor_texture,'walls':visible_walls,'wall_texture':walls.texture}

func draw(canvas: CanvasItem, plan: Dictionary) -> void:
	for item in plan.floors: canvas.draw_polygon(item.points,PackedColorArray([Color.WHITE]),item.uv,plan.floor_texture)
