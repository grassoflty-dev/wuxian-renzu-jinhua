extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const Presentation = preload('res://scripts/world/presentation.gd')
const SEGMENTS = 32
const TICKS = 8

# Validated current snapshot only. A false return leaves the old ring as fallback.
func draw_warning(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, key: String, projection, viewport: Rect2) -> bool:
	if not is_instance_valid(canvas) or not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_sentinel_arena':
		return false
	if scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_sentinel_arena' or key!='grey_hive:gh_sentinel_arena:'+str(snapshot.worldEpoch):
		return false
	if projection == null or not projection is Object or not projection.has_method('project') or not P.number(projection.get('ppm'),0.000001,256.0):
		return false
	if not _finite_rect(viewport):
		return false
	var encounter = Presentation.sentinel(snapshot)
	if encounter.is_empty() or not encounter.get('phase') in ['light_windup','heavy_windup'] or not encounter.get('warning') is Dictionary:
		return false
	var warning: Dictionary = encounter.warning
	if warning.get('shape')!='circle' or not P.point(warning.get('originM')) or not P.number(warning.get('radiusM'),1.8,2.2):
		return false
	var radius: float = float(warning.radiusM)
	if not is_equal_approx(radius,1.8 if encounter.phase=='light_windup' else 2.2):
		return false
	var center: Vector3 = P.vector(warning.originM)
	var inner: float = radius*(0.91 if encounter.phase=='light_windup' else 0.85)
	var shapes: Array[PackedVector2Array] = []
	var lo = Vector2(INF,INF)
	var hi = Vector2(-INF,-INF)
	for index in range(SEGMENTS):
		var a = TAU*float(index)/float(SEGMENTS)
		var b = TAU*float(index+1)/float(SEGMENTS)
		var quad = _quad(projection,center,radius,inner,a,b)
		if quad.size()!=4:
			return false
		shapes.append(quad)
		for point in quad:
			lo = lo.min(point)
			hi = hi.max(point)
	for index in range(TICKS):
		var a = TAU*float(index)/float(TICKS)
		var tick = _quad(projection,center,radius*0.83,radius*0.72,a-0.018,a+0.018)
		if tick.size()!=4:
			return false
		shapes.append(tick)
	var bounds = Rect2(lo,hi-lo)
	if not _finite_rect(bounds):
		return false
	if not viewport.intersects(bounds):
		return true
	var color = Color('#f0ad6a')
	color.a = 0.72
	for shape in shapes:
		canvas.draw_colored_polygon(shape,color)
	return true

func _quad(projection, center: Vector3, outer: float, inner: float, a: float, b: float) -> PackedVector2Array:
	var points = PackedVector2Array()
	for item in [[outer,a],[outer,b],[inner,b],[inner,a]]:
		var p = projection.project(center+Vector3(sin(item[1])*item[0],0,cos(item[1])*item[0]))
		if not p is Vector2 or not is_finite(p.x) or not is_finite(p.y):
			return PackedVector2Array()
		points.append(p)
	return points

func _finite_rect(rect: Rect2) -> bool:
	return is_finite(rect.position.x) and is_finite(rect.position.y) and is_finite(rect.size.x) and is_finite(rect.size.y) and is_finite(rect.end.x) and is_finite(rect.end.y) and rect.size.x>0.0 and rect.size.y>0.0