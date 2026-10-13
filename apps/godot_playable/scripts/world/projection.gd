extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
var origin = Vector3.ZERO
var viewport = Vector2(1280,800)
var ppm = 48.0
var initialized = false
var boss_scale = 1.0
func reset() -> void:
	initialized = false
	boss_scale = 1.0
func project(p: Vector3) -> Vector2:
	var d = p-origin
	return viewport*0.5+Vector2((d.x-d.z)*ppm,(d.x+d.z)*ppm*0.5-d.y*ppm)
func inverse(screen: Vector2, height: float) -> Vector3:
	var d = (screen-viewport*0.5)/ppm
	var sum = 2.0*(d.y+height-origin.y)
	return Vector3(origin.x+(sum+d.x)*0.5,height,origin.z+(sum-d.x)*0.5)
func update(s: Dictionary, scene: Dictionary, size: Vector2, delta: float) -> void:
	viewport = size
	if size.x<=0 or size.y<=0 or not is_finite(delta):
		return
	var player = P.vector(s.player.transform.positionM)
	var base = 48.0*maxf(0.5,minf(size.x/1280.0,size.y/720.0))
	var b: Dictionary = scene.boundsM
	if s.worldId=='return_station' and s.sceneId=='rs_core_room':
		# Native placeholder room envelope includes raised walls and player/terminal height.
		var corners = [Vector3(b.x,0,b.z),Vector3(b.x+b.width,0,b.z),Vector3(b.x+b.width,0,b.z+b.depth),Vector3(b.x,0,b.z+b.depth)]
		var lo = Vector2(INF,INF)
		var hi = Vector2(-INF,-INF)
		for p in corners:
			var u = Vector2(p.x-p.z,(p.x+p.z)*0.5)
			lo = lo.min(u+Vector2(-0.8,-3))
			hi = hi.max(u+Vector2(0.8,0.6))
		ppm = minf(size.x*0.9/(hi.x-lo.x),size.y*0.9/(hi.y-lo.y))
		var middle = (hi+lo)*0.5
		origin = Vector3(middle.y+middle.x*0.5,0,middle.y-middle.x*0.5)
		initialized = true
		return
	var sentinel = 0
	for a in s.actors:
		if a.entityId=='gh_sentinel_arena_sentinel_01' or a.entityType=='enemy.grey_hive.sentinel':
			if a.entityId=='gh_sentinel_arena_sentinel_01' and a.entityType=='enemy.grey_hive.sentinel' and a.active:
				sentinel += 1
			else:
				sentinel += 100
	var target_scale = 0.86 if s.sceneId=='gh_sentinel_arena' and sentinel==1 else 1.0
	boss_scale = lerpf(boss_scale,target_scale,1-exp(-delta/0.24))
	ppm = base*boss_scale
	var aim = Vector2(float(s.player.aimX),float(s.player.aimZ)).normalized()
	var target = player+Vector3(aim.x,0,aim.y)*1.25
	if not initialized:
		origin = target
		initialized = true
	else:
		var relative = project(player)-viewport*0.5
		var lead = Vector2(-(target.x-player.x-(target.z-player.z))*ppm,-(target.x-player.x+target.z-player.z)*ppm*0.5)
		var dead = Vector2(72,42)*ppm/48
		var correction = relative-relative.clamp(lead-dead,lead+dead)
		var desired = origin+Vector3((correction.x+2*correction.y)/(2*ppm),0,(2*correction.y-correction.x)/(2*ppm))
		desired.y = target.y
		origin = origin.lerp(desired,1-exp(-delta/0.18))
	# Keep the follow origin within the authored room; no position or collision mutation.
	origin.x = clampf(origin.x,b.x,b.x+b.width)
	origin.z = clampf(origin.z,b.z,b.z+b.depth)
