extends RefCounted

# Candidate surface-bound dust flecks: no roller, outlet, contact event or emission.
# Native surface transport only; no local time, cache, interaction or raised cloud.
const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const PERIOD_TICKS: int = 550
const LANES: Array[float] = [6.7,7.4,8.6,9.3]
const INK: Color = Color(0.55,0.51,0.43,0.12)
const SCENE_SHA: String = '845b664b2212c5d9bf55cf72d34c9c69bf0caf345fa90ed9a7173a1df276e0ae'
const COLLISION: Array = [{"id":"cw_conveyor_bridge_wall_north","polygon":[[0.0,0.0],[24.0,0.0],[24.0,0.5],[0.0,0.5]]},{"id":"cw_conveyor_bridge_wall_south","polygon":[[0.0,15.5],[24.0,15.5],[24.0,16.0],[0.0,16.0]]},{"id":"cw_conveyor_bridge_wall_west_north","polygon":[[0.0,0.5],[0.5,0.5],[0.5,6.0],[0.0,6.0]]},{"id":"cw_conveyor_bridge_wall_west_south","polygon":[[0.0,9.5],[0.5,9.5],[0.5,15.5],[0.0,15.5]]},{"id":"cw_conveyor_bridge_wall_east_north","polygon":[[23.5,0.5],[24.0,0.5],[24.0,6.0],[23.5,6.0]]},{"id":"cw_conveyor_bridge_wall_east_south","polygon":[[23.5,9.5],[24.0,9.5],[24.0,15.5],[23.5,15.5]]}]
const TERRAIN: Array = [{"id":"cw_conveyor_belt","polygon":[[6.0,6.0],[18.0,6.0],[18.0,10.0],[6.0,10.0]],"surfaceVelocityMps":[-1.2,0.0],"tag":"conveyor"}]
const TRANSITIONS: Array = [{"id":"cw_bridge_to_pressure_hall","polygon":[[0.5,6.5],[2.5,6.5],[2.5,9.5],[0.5,9.5]],"spawnId":"cw_pressure_hall_spawn","toSceneId":"cw_pressure_hall"},{"id":"cw_bridge_to_boiler_chamber","polygon":[[22.0,6.5],[24.0,6.5],[24.0,9.5],[22.0,9.5]],"spawnId":"cw_boiler_chamber_spawn","toSceneId":"cw_boiler_chamber"}]
const PRESENTATION: Dictionary = {"cameraProfile":"oblique_default","layers":[{"id":"visual.floor","zGroup":0},{"id":"visual.props_back","zGroup":10},{"id":"visual.props_dynamic","zGroup":20}],"sprites":[{"assetId":"runtime2d.world.clockworks.floor_steel.v1","id":"cw_conveyor_bridge_steel_floor","layer":"visual.floor","position":[12.0,0.0,8.0]},{"assetId":"runtime2d.world.clockworks.factory_tiles.v1","id":"cw_conveyor_bridge_factory_tiles","layer":"visual.floor","position":[12.0,0.0,8.0]},{"assetId":"runtime2d.world.clockworks.pressure_pipes.v1","id":"cw_bridge_pressure_pipe_run","layer":"visual.props_back","position":[11.0,0.0,3.5]},{"assetId":"runtime2d.world.clockworks.factory_tiles.v1","id":"cw_bridge_factory_module","layer":"visual.props_back","position":[13.0,0.0,12.0]},{"assetId":"runtime2d.world.clockworks.factory_tiles.v1","id":"cw_bridge_factory_module_side","layer":"visual.props_dynamic","position":[18.0,0.0,7.0]}]}

func _same(actual: Variant, expected: Variant) -> bool:
	if expected is Dictionary:
		if not actual is Dictionary or actual.size()!=expected.size():
			return false
		for key: Variant in expected:
			if not actual.has(key) or not _same(actual[key],expected[key]):
				return false
		return true
	if expected is Array:
		if not actual is Array or actual.size()!=expected.size():
			return false
		for index: int in range(expected.size()):
			if not _same(actual[index],expected[index]):
				return false
		return true
	if typeof(expected) in [TYPE_INT,TYPE_FLOAT]:
		return P.number(actual,float(expected),float(expected))
	return typeof(actual)==typeof(expected) and actual==expected

func _finite2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y) and absf(value.x)<=10000000.0 and absf(value.y)<=10000000.0

func _context(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, projection: Variant, viewport: Variant, gate: Variant) -> bool:
	if not canvas is CanvasItem or not is_instance_valid(canvas) or not canvas.is_inside_tree() or canvas.is_queued_for_deletion():
		return false
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.player.currentHp<=0:
		return false
	if snapshot.worldId!='clockworks' or snapshot.sceneId!='cw_conveyor_bridge' or not P.counter(snapshot.worldEpoch,1):
		return false
	if typeof(key)!=TYPE_STRING or key!='clockworks:cw_conveyor_bridge:'+str(snapshot.worldEpoch) or scene_hash!=SCENE_SHA:
		return false
	if not scene is Dictionary or scene.get('worldId')!='clockworks' or scene.get('sceneId')!='cw_conveyor_bridge':
		return false
	if not P.counter(scene.get('schemaVersion'),1,1) or not _same(scene.get('boundsM'),{'x':0,'z':0,'width':24,'depth':16}):
		return false
	if not _same(scene.get('collision'),COLLISION) or not _same(scene.get('terrainRegions'),TERRAIN) or not _same(scene.get('transitions'),TRANSITIONS) or not _same(scene.get('hazards'),[]) or not _same(scene.get('occluders'),[]) or not _same(scene.get('presentation'),PRESENTATION):
		return false
	if canvas.get('snapshot')!=snapshot or canvas.get('scene')!=scene or canvas.get('current_key')!=key or canvas.get('scene_hash')!=scene_hash:
		return false
	if not gate is Dictionary or not P.same(gate,snapshot) or gate.get('serverTick')!=snapshot.serverTick or gate.get('authorityRevision')!=snapshot.authorityRevision or gate.get('player_id')!=snapshot.player.entityId:
		return false
	for field: String in ['presentation_allowed','visible','reduced_motion']:
		if typeof(gate.get(field))!=TYPE_BOOL:
			return false
	if not gate.presentation_allowed or not gate.visible or gate.reduced_motion:
		return false
	if not projection is RefCounted or not is_instance_valid(projection) or projection.get_script()!=ProjectionScript or canvas.get('camera')!=projection:
		return false
	if not viewport is Rect2 or canvas.get('viewport_rect')!=viewport or viewport!=canvas.get_viewport_rect():
		return false
	var origin: Variant = projection.get('origin')
	var center: Variant = projection.get('screen_center')
	var size: Variant = projection.get('viewport')
	if not origin is Vector3 or not center is Vector2 or not size is Vector2:
		return false
	if typeof(projection.get('initialized'))!=TYPE_BOOL or not projection.get('initialized') or not P.number(projection.get('ppm'),0.001,256):
		return false
	return is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and _finite2(center) and _finite2(size) and _finite2(viewport.position) and _finite2(viewport.size) and _finite2(viewport.end) and viewport.position==Vector2.ZERO and viewport.size==size and size.x>0 and size.y>0

func _smooth(value: float) -> float:
	var t: float = clampf(value,0.0,1.0)
	return t*t*(3.0-2.0*t)

func draw_dust(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, projection: Variant, viewport: Variant, gate: Variant) -> int:
	if not _context(canvas,snapshot,scene,key,scene_hash,projection,viewport,gate):
		return 0
	# Reduce validated tick BEFORE adding offsets to avoid safe-counter overflow.
	var tick: int = int(snapshot.serverTick)%PERIOD_TICKS
	var plans: Array[Dictionary] = []
	for lane: int in range(LANES.size()):
		for member: int in range(3):
			var offset: int = member*183+lane*17
			var q: float = float((tick+offset)%PERIOD_TICKS)/float(PERIOD_TICKS)
			# 11m / 550 ticks = .02m per tick = authored 1.2m/s at native60Hz.
			var center: Vector2 = Vector2(17.5-11.0*q,LANES[lane])
			var alpha: float = INK.a*_smooth(q/0.1)*_smooth((1.0-q)/0.1)
			if not is_finite(alpha) or alpha<0.0 or alpha>INK.a:
				return 0
			var points: PackedVector2Array = PackedVector2Array()
			for delta: Vector2 in [Vector2(-0.035,0),Vector2(0,-0.02),Vector2(0.035,0),Vector2(0,0.02)]:
				var ground: Vector2 = center+delta
				if not _finite2(ground) or ground.x<6.46 or ground.x>17.54 or ground.y<6.67 or ground.y>9.33:
					return 0
				var point: Variant = projection.call('project',Vector3(ground.x,0.0,ground.y))
				if not point is Vector2 or not _finite2(point):
					return 0
				points.append(point)
			if points.size()!=4:
				return 0
			var twice_area: float = (points[1]-points[0]).cross(points[2]-points[0])+(points[2]-points[0]).cross(points[3]-points[0])
			if not is_finite(twice_area) or absf(twice_area)<=0.000000001:
				return 0
			var bounds: Rect2 = Rect2(points[0],Vector2.ZERO)
			for point: Vector2 in points:
				bounds = bounds.expand(point)
			if not _finite2(bounds.position) or not _finite2(bounds.end) or bounds.size.x<=0.0 or bounds.size.y<=0.0:
				return 0
			plans.append({'points':points,'bounds':bounds,'color':Color(INK.r,INK.g,INK.b,alpha)})
	# Complete12-fleck plan before any new-layer submission; preserve base on failure.
	if plans.size()!=12 or not _context(canvas,snapshot,scene,key,scene_hash,projection,viewport,gate):
		return 0
	var submitted: int = 0
	for plan: Dictionary in plans:
		if plan.color.a>0.0 and plan.bounds.intersects(viewport):
			canvas.draw_colored_polygon(plan.points,plan.color)
			submitted += 1
	return submitted
