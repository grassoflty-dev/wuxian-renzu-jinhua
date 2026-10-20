extends RefCounted

# S1 shaft dust only. All centers, envelope, color and timing below are candidate
# display parameters, not authored emitter anchors, lamps, routes or mechanics.
# Stateless accepted native tick; reduced motion omits the entire dust layer.
const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const SCENE_SHA: String = '806ef2be417d59ab7041002cecfb26c1fc383db920157feb29c2669c343df5da'
const PERIOD_TICKS: int = 720
const RGB: Color = Color(183.0/255.0,197.0/255.0,188.0/255.0,1.0)
const CENTERS: Array[Vector2] = [Vector2(3,2),Vector2(7,2),Vector2(12,2),Vector2(21,2),Vector2(3,8.25),Vector2(7,8.25),Vector2(12,8.25),Vector2(21,8.25),Vector2(3,13),Vector2(7,13),Vector2(12,13),Vector2(21,13)]
const COLLISION: Array = [{"id":"shaft_wall_north","polygon":[[0.0,0.0],[24.0,0.0],[24.0,0.5],[0.0,0.5]]},{"id":"shaft_wall_south","polygon":[[0.0,15.5],[24.0,15.5],[24.0,16.0],[0.0,16.0]]},{"id":"shaft_wall_west_north","polygon":[[0.0,0.5],[0.5,0.5],[0.5,6.5],[0.0,6.5]]},{"id":"shaft_wall_west_south","polygon":[[0.0,9.5],[0.5,9.5],[0.5,15.5],[0.0,15.5]]},{"id":"shaft_wall_east_north","polygon":[[23.5,0.5],[24.0,0.5],[24.0,6.5],[23.5,6.5]]},{"id":"shaft_wall_east_south","polygon":[[23.5,9.5],[24.0,9.5],[24.0,15.5],[23.5,15.5]]},{"id":"shaft_support_column_a","polygon":[[8.0,5.0],[9.0,5.0],[9.0,7.0],[8.0,7.0]]},{"id":"shaft_support_column_b","polygon":[[15.0,9.0],[16.0,9.0],[16.0,11.0],[15.0,11.0]]}]
const OCCLUDERS: Array = [{"fadeTo":0.2,"id":"gh_shaft_occlusion_bridge_beam","polygon":[[11.0,6.5],[18.0,6.5],[18.0,7.5],[11.0,7.5]]},{"fadeTo":0.3,"id":"gh_shaft_occlusion_lift_beam","polygon":[[6.0,4.0],[9.0,4.0],[9.0,4.75],[6.0,4.75]]}]

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
	if snapshot.worldId!='grey_hive' or snapshot.sceneId!='gh_central_shaft' or not P.counter(snapshot.worldEpoch,1):
		return false
	if typeof(key)!=TYPE_STRING or key!='grey_hive:gh_central_shaft:'+str(snapshot.worldEpoch) or scene_hash!=SCENE_SHA:
		return false
	if not scene is Dictionary or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_central_shaft':
		return false
	if not P.counter(scene.get('schemaVersion'),1,1) or not _same(scene.get('boundsM'),{'x':0,'z':0,'width':24,'depth':16}):
		return false
	if not _same(scene.get('vfxMarkers'),[]) or not _same(scene.get('collision'),COLLISION) or not _same(scene.get('occluders'),OCCLUDERS) or not _same(scene.get('hazards'),[]):
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
	if typeof(projection.get('initialized'))!=TYPE_BOOL or not projection.get('initialized') or not P.number(projection.get('ppm'),0.001,3072):
		return false
	return is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and _finite2(center) and _finite2(size) and _finite2(viewport.position) and _finite2(viewport.size) and _finite2(viewport.end) and viewport.position==Vector2.ZERO and viewport.size==size and size.x>0 and size.y>0

func draw_dust(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, projection: Variant, viewport: Variant, gate: Variant) -> int:
	if not _context(canvas,snapshot,scene,key,scene_hash,projection,viewport,gate):
		return 0
	# Modulo before offset keeps large legal native tick values bounded.
	var tick: int = int(snapshot.serverTick)%PERIOD_TICKS
	var radius: float = minf(0.025*float(projection.ppm),2.0)
	if not is_finite(radius) or radius<=0.0 or radius>2.0:
		return 0
	var plans: Array[Dictionary] = []
	for index: int in range(CENTERS.size()):
		var q: float = float((tick+60*index)%PERIOD_TICKS)/float(PERIOD_TICKS)
		var phase: float = TAU*q
		var center: Vector3 = Vector3(CENTERS[index].x+0.28*sin(phase),0.10+0.16*q,CENTERS[index].y+0.18*sin(phase+PI/3.0))
		# Conservative XZ envelope includes physical drift and the entire screen
		# circle's inverse ground projection: +/-0.60 X and +/-0.50 Z.
		# Exact canonical collision/occluder separation is documented in evidence.
		if not is_finite(center.x) or not is_finite(center.y) or not is_finite(center.z) or absf(center.x-CENTERS[index].x)>0.280001 or absf(center.z-CENTERS[index].y)>0.180001 or center.y<0.10 or center.y>0.26:
			return 0
		var projected: Variant = projection.call('project',center)
		if not projected is Vector2 or not _finite2(projected):
			return 0
		var fade: float = sin(PI*q)
		var color: Color = RGB
		color.a = 0.12*fade*fade
		var bounds: Rect2 = Rect2(projected-Vector2(radius,radius),Vector2(radius*2.0,radius*2.0))
		if not _finite2(bounds.position) or not _finite2(bounds.size) or not _finite2(bounds.end) or bounds.size.x<=0.0 or bounds.size.y<=0.0 or not is_finite(color.a) or color.a<0.0 or color.a>0.12:
			return 0
		plans.append({'point':projected,'radius':radius,'color':color,'bounds':bounds})
	# No submission until all twelve particles and the current context validate.
	if plans.size()!=12 or not _context(canvas,snapshot,scene,key,scene_hash,projection,viewport,gate):
		return 0
	var submitted: int = 0
	for plan: Dictionary in plans:
		if plan.color.a>0.0 and plan.bounds.intersects(viewport):
			canvas.draw_circle(plan.point,plan.radius,plan.color)
			submitted += 1
	return submitted
