extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const ProjectionScript = preload("res://scripts/world/projection.gd")

const WORLD_ID: String = "grey_hive"
const SCENE_ID: String = "gh_entry_maintenance"
const FLOOR_WASH: Color = Color("#496d73")
const FLOOR_WASH_ALPHA: float = 0.04
const MOTE_COLOR: Color = Color("#b8ccce")
const MOTE_PHASE_RATE: float = 0.2
const MOTE_PHASE_PERIOD: float = TAU / MOTE_PHASE_RATE
const MOTE_DRIFT_X: float = 0.075
const MOTE_DRIFT_Z: float = 0.04
const MOTES: Array = [
	[Vector3(1.35, 0.52, 1.55), 0.15, 0.72, 0.032],
	[Vector3(3.8, 0.84, 3.1), 0.63, 0.9, 0.038],
	[Vector3(6.15, 1.32, 1.25), 1.07, 0.82, 0.034],
	[Vector3(8.6, 0.58, 4.35), 1.52, 1.05, 0.041],
	[Vector3(11.2, 1.64, 2.2), 1.96, 0.68, 0.03],
	[Vector3(13.4, 0.96, 5.15), 2.41, 0.96, 0.04],
	[Vector3(16.1, 0.46, 3.65), 2.84, 0.78, 0.033],
	[Vector3(18.55, 1.18, 1.85), 3.28, 1.12, 0.044],
	[Vector3(2.4, 1.52, 8.25), 3.72, 0.86, 0.037],
	[Vector3(5.45, 0.72, 11.25), 4.16, 1.0, 0.042],
	[Vector3(9.35, 1.08, 8.95), 4.59, 0.7, 0.031],
	[Vector3(12.1, 0.42, 12.45), 5.03, 0.92, 0.039],
	[Vector3(15.45, 1.78, 11.55), 5.47, 1.08, 0.045],
	[Vector3(18.2, 0.9, 8.1), 5.91, 0.8, 0.035]
]

var _context_key: String = ""
var _phase_seconds: float = 0.0

func reset() -> void:
	_context_key = ""
	_phase_seconds = 0.0

func advance_visual(delta: float, reduced_motion: bool) -> void:
	if not is_finite(delta) or delta < 0.0 or reduced_motion:
		return
	var next_phase: float = _phase_seconds + delta
	if not is_finite(next_phase):
		return
	_phase_seconds = fposmod(next_phase, MOTE_PHASE_PERIOD)

func draw_floor(canvas, snapshot, scene_data, projection) -> bool:
	if not canvas is CanvasItem or not _valid_context(snapshot, scene_data, projection):
		return false
	_bind_context(snapshot)
	var points: PackedVector2Array = PackedVector2Array()
	var corners: Array[Vector3] = [
		Vector3(0.0, 0.0, 0.0),
		Vector3(20.0, 0.0, 0.0),
		Vector3(20.0, 0.0, 14.0),
		Vector3(0.0, 0.0, 14.0)
	]
	for corner: Vector3 in corners:
		var screen_point: Vector2 = projection.project(corner)
		if not _finite_vector2(screen_point):
			return false
		points.append(screen_point)
	var wash: Color = FLOOR_WASH
	wash.a = FLOOR_WASH_ALPHA
	canvas.draw_colored_polygon(points, wash)
	return true

func draw_air(canvas, snapshot, scene_data, projection) -> bool:
	if not canvas is CanvasItem or not _valid_context(snapshot, scene_data, projection):
		return false
	_bind_context(snapshot)
	var viewport_rect: Rect2 = canvas.get_viewport_rect()
	if not _finite_rect(viewport_rect) or viewport_rect.size.x <= 0.0 or viewport_rect.size.y <= 0.0:
		return false
	var ppm: float = float(projection.get("ppm"))
	for index: int in range(MOTES.size()):
		var mote: Array = MOTES[index]
		var base: Vector3 = mote[0]
		var offset: float = float(mote[1])
		var angle: float = _phase_seconds * MOTE_PHASE_RATE + offset
		var world_point: Vector3 = Vector3(
			base.x + sin(angle) * MOTE_DRIFT_X,
			base.y,
			base.z + cos(angle) * MOTE_DRIFT_Z
		)
		if world_point.x < 0.8 or world_point.x > 19.2 or world_point.z < 0.8 or world_point.z > 13.2 or world_point.y < 0.4 or world_point.y > 1.8:
			continue
		var screen_point: Vector2 = projection.project(world_point)
		if not _finite_vector2(screen_point) or not viewport_rect.has_point(screen_point):
			continue
		var radius: float = float(mote[2]) * ppm / 48.0
		var mote_color: Color = MOTE_COLOR
		mote_color.a = float(mote[3])
		canvas.draw_circle(screen_point, radius, mote_color, true)
	return true

func _valid_context(snapshot, scene_data, projection) -> bool:
	if not snapshot is Dictionary or not P.snapshot(snapshot):
		return false
	if snapshot.get("worldId") != WORLD_ID or snapshot.get("sceneId") != SCENE_ID:
		return false
	if not scene_data is Dictionary or scene_data.get("worldId") != WORLD_ID or scene_data.get("sceneId") != SCENE_ID:
		return false
	if not P.counter(snapshot.get("worldEpoch"), 1):
		return false
	if not P.counter(scene_data.get("schemaVersion"), 1, 1):
		return false
	var raw_bounds = scene_data.get("boundsM")
	if not raw_bounds is Dictionary:
		return false
	var bounds: Dictionary = raw_bounds
	if bounds.size() != 4 or not P.number(bounds.get("x"), 0.0, 0.0) or not P.number(bounds.get("z"), 0.0, 0.0):
		return false
	if not P.number(bounds.get("width"), 20.0, 20.0) or not P.number(bounds.get("depth"), 14.0, 14.0):
		return false
	if not projection is RefCounted or projection.get_script() != ProjectionScript or not projection.has_method("project"):
		return false
	var raw_origin = projection.get("origin")
	var raw_center = projection.get("screen_center")
	if not raw_origin is Vector3 or not raw_center is Vector2 or not P.number(projection.get("ppm"), 0.000001, 256.0):
		return false
	var origin: Vector3 = raw_origin
	var center: Vector2 = raw_center
	if not _finite_vector3(origin) or not _finite_vector2(center):
		return false
	return true

func _bind_context(snapshot: Dictionary) -> void:
	var key: String = WORLD_ID + ":" + SCENE_ID + ":" + str(snapshot.get("worldEpoch"))
	if key != _context_key:
		_context_key = key
		_phase_seconds = 0.0

func _finite_vector2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _finite_vector3(value: Vector3) -> bool:
	return is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _finite_rect(value: Rect2) -> bool:
	return _finite_vector2(value.position) and _finite_vector2(value.size) and _finite_vector2(value.end)
