extends RefCounted

# S1 sterile fog only. Layout and color below are candidate visual parameters,
# not authored scene anchors, scanning devices, hazards or native mechanisms.
const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const SCENE_SHA: String = 'a3d629f04dc955cf0f8e644b54a4a64d054411a7d59a9fb15da5c9d9ea953aff'
const PERIOD_TICKS: int = 480
const RGB: Color = Color(186.0/255.0,211.0/255.0,205.0/255.0,1.0)
const CENTERS: Array[Vector2] = [Vector2(4,4),Vector2(12,4),Vector2(20,4),Vector2(4,12),Vector2(12,12),Vector2(20,12)]
const COLLISION: Array = [{"id":"gh_bio_isolation_wall_north","polygon":[[0.0,0.0],[24.0,0.0],[24.0,0.5],[0.0,0.5]]},{"id":"gh_bio_isolation_wall_south","polygon":[[0.0,15.5],[24.0,15.5],[24.0,16.0],[0.0,16.0]]},{"id":"gh_bio_isolation_wall_west_north","polygon":[[0.0,0.5],[0.5,0.5],[0.5,6.5],[0.0,6.5]]},{"id":"gh_bio_isolation_wall_west_south","polygon":[[0.0,9.5],[0.5,9.5],[0.5,15.5],[0.0,15.5]]},{"id":"gh_bio_isolation_wall_east_north","polygon":[[23.5,0.5],[24.0,0.5],[24.0,6.5],[23.5,6.5]]},{"id":"gh_bio_isolation_wall_east_south","polygon":[[23.5,9.5],[24.0,9.5],[24.0,15.5],[23.5,15.5]]}]

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
	if snapshot.worldId!='grey_hive' or snapshot.sceneId!='gh_bio_isolation' or not P.counter(snapshot.worldEpoch,1):
		return false
	if typeof(key)!=TYPE_STRING or key!='grey_hive:gh_bio_isolation:'+str(snapshot.worldEpoch) or scene_hash!=SCENE_SHA:
		return false
	if not scene is Dictionary or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_bio_isolation':
		return false
	if not P.counter(scene.get('schemaVersion'),1,1) or not _same(scene.get('boundsM'),{'x':0,'z':0,'width':24,'depth':16}):
		return false
	if not _same(scene.get('vfxMarkers'),[]) or not _same(scene.get('collision'),COLLISION) or not _same(scene.get('occluders'),[]) or not _same(scene.get('hazards'),[]):
		return false
	if canvas.get('snapshot')!=snapshot or canvas.get('scene')!=scene or canvas.get('current_key')!=key or canvas.get('scene_hash')!=scene_hash:
		return false
	if not gate is Dictionary or not P.same(gate,snapshot) or gate.get('serverTick')!=snapshot.serverTick or gate.get('authorityRevision')!=snapshot.authorityRevision or gate.get('player_id')!=snapshot.player.entityId:
		return false
	for field: String in ['presentation_allowed','visible','reduced_motion']:
		if typeof(gate.get(field))!=TYPE_BOOL:
			return false
	if not gate.presentation_allowed or not gate.visible:
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
	if typeof(projection.get('initialized'))!=TYPE_BOOL or not projection.get('initialized') or not P.number(projection.get('ppm'),0.001,3072):
		return false
	return is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and _finite2(center) and _finite2(size) and _finite2(viewport.position) and _finite2(viewport.size) and _finite2(viewport.end) and viewport.position==Vector2.ZERO and viewport.size==size and size.x>0 and size.y>0

func _points(projection: Variant, center: Vector3, inner: bool) -> PackedVector2Array:
	var result: PackedVector2Array = PackedVector2Array()
	var scale_value: float = 0.62 if inner else 1.0
	for index: int in range(8):
		var angle: float = TAU*float(index)/8.0
		var point: Vector3 = center+Vector3(cos(angle)*1.55*scale_value,0,sin(angle)*0.55*scale_value)
		if point.x<0.5 or point.x>23.5 or point.z<0.5 or point.z>15.5 or not is_equal_approx(point.y,0.06):
			return PackedVector2Array()
		var projected: Variant = projection.call('project',point)
		if not projected is Vector2 or not _finite2(projected):
			return PackedVector2Array()
		result.append(projected)
	var twice_area: float = 0.0
	# Translation-relative area avoids cancellation far from screen origin.
	for index: int in range(1,7):
		twice_area += (result[index]-result[0]).cross(result[index+1]-result[0])
	return result if is_finite(twice_area) and absf(twice_area)>0.0 else PackedVector2Array()

func _bounds(points: PackedVector2Array) -> Rect2:
	var low: Vector2 = points[0]
	var high: Vector2 = points[0]
	for point: Vector2 in points:
		low = low.min(point)
		high = high.max(point)
	return Rect2(low,high-low)

func draw_fog(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, projection: Variant, viewport: Variant, gate: Variant) -> int:
	if not _context(canvas,snapshot,scene,key,scene_hash,projection,viewport,gate):
		return 0
	# Current native tick only: no state, delta, wall clock, queue or phase cache.
	var native_phase: float = float(int(snapshot.serverTick)%PERIOD_TICKS)/float(PERIOD_TICKS)
	var plans: Array[Dictionary] = []
	for index: int in range(CENTERS.size()):
		var phase: float = TAU*(native_phase+float(index)/6.0)
		var drift: Vector2 = Vector2.ZERO if gate.reduced_motion else Vector2(sin(phase)*0.18,cos(phase)*0.10)
		var envelope: float = 0.85 if gate.reduced_motion else 0.85+0.15*sin(phase)
		var center: Vector3 = Vector3(CENTERS[index].x+drift.x,0.06,CENTERS[index].y+drift.y)
		for inner: bool in [false,true]:
			var points: PackedVector2Array = _points(projection,center,inner)
			if points.size()!=8:
				return 0
			var color: Color = RGB
			color.a = (0.028 if inner else 0.022)*envelope
			var bounds: Rect2 = _bounds(points)
			if not _finite2(bounds.position) or not _finite2(bounds.size) or not _finite2(bounds.end) or bounds.size.x<=0 or bounds.size.y<=0 or not is_finite(color.a) or color.a<=0 or color.a>0.028:
				return 0
			plans.append({'points':points,'color':color,'bounds':bounds})
	# Validate the entire twelve-polygon unit before any draw submission.
	if plans.size()!=12 or not _context(canvas,snapshot,scene,key,scene_hash,projection,viewport,gate):
		return 0
	var submitted: int = 0
	for plan: Dictionary in plans:
		if plan.bounds.intersects(viewport):
			canvas.draw_colored_polygon(plan.points,plan.color)
			submitted += 1
	return submitted
