extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const KIND = 'DEV_WHITEZHI_PLACEHOLDER'
func resolve(snapshot: Dictionary, scene: Dictionary) -> Dictionary:
	if not P.counter(snapshot.get('protocolVersion'),3,3) or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_bio_isolation' or scene.get('worldId')!=snapshot.get('worldId') or scene.get('sceneId')!=snapshot.get('sceneId') or not P.counter(snapshot.get('worldEpoch'),1):
		return {}
	var b = snapshot.get('baizhi')
	var npcs = snapshot.get('npcs')
	if not b is Dictionary or not P.counter(b.get('schemaVersion'),1,1) or not b.get('choice') is String or not b.choice in ['unresolved','taken','left'] or typeof(b.get('available'))!=TYPE_BOOL or not b.available or typeof(b.get('canInteract'))!=TYPE_BOOL or not npcs is Array or npcs.size()!=1:
		return {}
	var npc = npcs[0]
	if not npc is Dictionary or npc.get('entityId')!='gh_bz_whitezhi_v1' or npc.get('entityType')!='npc.baizhi' or not npc.get('position') is Array or npc.position.size()!=3 or not P.number(npc.position[0],8,8) or not P.number(npc.position[1],0,0) or not P.number(npc.position[2],10.5,10.5) or not P.number(npc.get('yawRad')) or absf(sin(npc.yawRad))>1.0e-6 or absf(cos(npc.yawRad)+1)>1.0e-6 or typeof(npc.get('interactable'))!=TYPE_BOOL or npc.interactable!=b.canInteract:
		return {}
	return {'entityId':'gh_bz_whitezhi_v1','entityType':'npc.baizhi','position':[npc.position[0],npc.position[1],npc.position[2]],'yawRad':npc.yawRad,'interactable':npc.interactable}
func draw_body(canvas: CanvasItem, npc: Dictionary, camera) -> bool:
	var ppm: float = camera.ppm
	if not is_finite(ppm) or ppm<=0:
		return false
	var p = Vector3(npc.position[0],npc.position[1],npc.position[2])
	var foot: Vector2 = camera.project(p)
	var forward: Vector2 = camera.project(p+Vector3(sin(npc.yawRad),0,cos(npc.yawRad)))
	var delta = forward-foot
	var length = delta.length()
	if not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(forward.x) or not is_finite(forward.y) or not is_finite(length) or length<=0:
		return false
	var facing = delta/length
	var width = 0.45*ppm
	var height = 1.7*ppm
	var radius = width*0.4
	var head_y = -height+radius
	var neck_y = head_y+radius*0.8
	var scale = ppm/48.0
	var cap = width*0.5
	var marker = Vector2(facing.x*radius,head_y+facing.y*radius)
	var triangle = PackedVector2Array([marker+Vector2(-2,2)*scale,marker+Vector2(2,2)*scale,Vector2(facing.x*width*0.5,head_y+facing.y*width*0.5)])
	if not is_finite(width) or not is_finite(height) or not is_finite(neck_y) or width<=0 or height<=0 or -neck_y<width:
		return false
	for q in triangle:
		if not is_finite(q.x) or not is_finite(q.y) or q.x < -width*0.5 or q.x > width*0.5 or q.y < -height or q.y > 0:
			return false
	var color = Color('#d5dedf')
	canvas.draw_rect(Rect2(foot+Vector2(-cap,neck_y+cap),Vector2(width,-neck_y-width)),color)
	canvas.draw_circle(foot+Vector2(0,neck_y+cap),cap,color)
	canvas.draw_circle(foot+Vector2(0,-cap),cap,color)
	canvas.draw_circle(foot+Vector2(0,head_y),radius,color)
	for n in triangle.size():
		triangle[n] += foot
	canvas.draw_colored_polygon(triangle,color)
	return true
