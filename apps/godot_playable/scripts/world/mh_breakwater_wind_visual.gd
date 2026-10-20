extends RefCounted

# Original S1 mh_breakwater wind/fog presentation at the two authored VFX markers.
# Procedural translucent polygons, not bitmap animation or a gameplay wind model.
const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const SCENE_SHA: String = '2de6f7b471b6b873fafad7e943a0f163e057ffca0787823625bf37bb23bd0b14'
const MARKERS: Array = [
	{'assetId':'runtime2d.world.mistharbor.fog_vfx.v1','id':'breakwater_fog_marker_01','position':[6,0,5]},
	{'assetId':'runtime2d.world.mistharbor.fog_vfx.v1','id':'breakwater_fog_marker_02','position':[17.5,0,10]}
]
const WALKABLE: Array = [
	{'id':'mh_breakwater_walkable_area','polygon':[[0.5,0.5],[23.5,0.5],[23.5,15.5],[0.5,15.5]]}
]
const PERIOD_TICKS: int = 360
const RGB: Color = Color(138.0/255.0,166.0/255.0,172.0/255.0,1.0)

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
	if snapshot.worldId!='mist_harbor' or snapshot.sceneId!='mh_breakwater' or not P.counter(snapshot.worldEpoch,1):
		return false
	if typeof(key)!=TYPE_STRING or key!='mist_harbor:mh_breakwater:'+str(snapshot.worldEpoch) or scene_hash!=SCENE_SHA:
		return false
	if not scene is Dictionary or scene.get('worldId')!='mist_harbor' or scene.get('sceneId')!='mh_breakwater':
		return false
	if not P.counter(scene.get('schemaVersion'),1,1) or not _same(scene.get('boundsM'),{'x':0,'z':0,'width':24,'depth':16}):
		return false
	if not _same(scene.get('vfxMarkers'),MARKERS) or not _same(scene.get('walkablePolygons'),WALKABLE) or not _same(scene.get('occluders'),[]) or not _same(scene.get('hazards'),[]):
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
	if typeof(projection.get('initialized'))!=TYPE_BOOL or not projection.get('initialized') or not P.number(projection.get('ppm'),0.001,256):
		return false
	return is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and _finite2(center) and _finite2(size) and _finite2(viewport.position) and _finite2(viewport.size) and _finite2(viewport.end) and viewport.position==Vector2.ZERO and viewport.size==size and size.x>0 and size.y>0

func _points(projection: Variant, center: Vector3, scale_value: float) -> PackedVector2Array:
	# A low, tapered horizontal wisp. Wind travel stays inside a 3.2 x 0.6 m
	# marker envelope; no footprint, support, visibility or collision changes.
	var offsets: Array[Vector2] = [
		Vector2(-0.85,0),Vector2(-0.50,-0.16),Vector2(0.10,-0.20),
		Vector2(0.65,-0.09),Vector2(0.85,0),Vector2(0.50,0.16),
		Vector2(-0.10,0.20),Vector2(-0.65,0.09)
	]
	var result: PackedVector2Array = PackedVector2Array()
	for offset: Vector2 in offsets:
		var point: Vector3 = center+Vector3(offset.x*scale_value,0,offset.y*scale_value)
		if point.x<0.5 or point.x>23.5 or point.z<0.5 or point.z>15.5 or point.y<0 or point.y>0.20:
			return PackedVector2Array()
		var projected: Variant = projection.call('project',point)
		if not projected is Vector2 or not _finite2(projected):
			return PackedVector2Array()
		result.append(projected)
	var twice_area: float = 0.0
	for index: int in range(result.size()):
		twice_area += result[index].cross(result[(index+1)%result.size()])
	return result if result.size()==8 and is_finite(twice_area) and absf(twice_area)>0.000001 else PackedVector2Array()

func _bounds(points: PackedVector2Array) -> Rect2:
	var low: Vector2 = points[0]
	var high: Vector2 = points[0]
	for point: Vector2 in points:
		low = low.min(point)
		high = high.max(point)
	return Rect2(low,high-low)

func draw_wind(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, projection: Variant, viewport: Variant, gate: Variant) -> int:
	if not _context(canvas,snapshot,scene,key,scene_hash,projection,viewport,gate):
		return 0
	# Native tick only. Same tick has identical geometry; there is no local
	# wall-clock interpolation, accumulated delta, replay queue or save state.
	var native_phase: float = float(int(snapshot.serverTick)%PERIOD_TICKS)/float(PERIOD_TICKS)
	var plans: Array[Dictionary] = []
	for marker_index: int in range(MARKERS.size()):
		var row: Dictionary = MARKERS[marker_index]
		var anchor: Vector3 = Vector3(float(row.position[0]),0,float(row.position[2]))
		for ribbon_index: int in range(2):
			var phase: float = 0.5 if gate.reduced_motion else fposmod(native_phase+0.5*float(ribbon_index)+0.25*float(marker_index),1.0)
			var envelope: float = (1.0 if ribbon_index==0 else 0.0) if gate.reduced_motion else pow(sin(PI*phase),2.0)
			# Both markers share +X decorative travel. No native wind vector is claimed.
			var drift: float = 0.0 if gate.reduced_motion else -0.70+1.40*phase
			var center: Vector3 = anchor+Vector3(drift,0.08+0.04*float(ribbon_index),-0.08+0.16*float(ribbon_index))
			for inner: bool in [false,true]:
				var points: PackedVector2Array = _points(projection,center,0.60 if inner else 1.0)
				if points.size()!=8:
					return 0
				var color: Color = RGB
				color.a = (0.045 if inner else 0.032)*envelope
				plans.append({'points':points,'color':color,'bounds':_bounds(points)})
	# Plan all eight polygons before the first submission; fail closed as a unit.
	if plans.size()!=8:
		return 0
	var submitted: int = 0
	for plan: Dictionary in plans:
		if plan.color.a>0.00001 and plan.bounds.intersects(viewport):
			canvas.draw_colored_polygon(plan.points,plan.color)
			submitted += 1
	return submitted

