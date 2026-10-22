extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const ProjectionScript = preload("res://scripts/world/projection.gd")

const WORLD_ID: String = "grey_hive"
const SCENE_ID: String = "gh_entry_maintenance"
const SCENE_SHA256: String = "5deb6c01b2ad81144144ae2526c06f8c337839e8d880aeb1b3af3a225afe787b"
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
var _pending_delta: float = 0.0

func reset() -> void:
	_context_key = ""
	_phase_seconds = 0.0
	_pending_delta = 0.0

func advance_visual(delta: float, reduced_motion: bool) -> void:
	_pending_delta = 0.0
	if not is_finite(delta) or delta < 0.0 or reduced_motion:
		return
	_pending_delta = delta

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
		_pending_delta = 0.0
		return false
	if not snapshot.has("entryToken") or snapshot.get("entryToken") != null or not P.counter(snapshot.get("worldEpoch"), 1):
		_pending_delta = 0.0
		return false
	_bind_context(snapshot)
	var pending_delta: float = _pending_delta
	_pending_delta = 0.0
	var frame: Dictionary = _capture_air_frame(canvas, snapshot, scene_data, projection)
	if frame.is_empty():
		return false
	var candidate_phase: float = _phase_seconds
	if frame.get("eligible") and not frame.get("reduced_motion"):
		var next_phase: float = _phase_seconds + pending_delta
		if not is_finite(next_phase):
			return false
		candidate_phase = fposmod(next_phase, MOTE_PHASE_PERIOD)
		if not is_finite(candidate_phase):
			return false
	var viewport_rect: Rect2 = frame.get("viewport")
	var ppm: float = frame.get("ppm")
	var staged: Array[Dictionary] = []
	if MOTES.size() != 14:
		return false
	for index: int in range(MOTES.size()):
		var raw_mote: Variant = MOTES[index]
		if not raw_mote is Array or raw_mote.size() != 4:
			return false
		var mote: Array = raw_mote
		var raw_base: Variant = mote[0]
		if not raw_base is Vector3 or not P.number(mote[1]) or not P.number(mote[2], 0.000001, 256.0) or not P.number(mote[3], 0.0, 1.0):
			return false
		var base: Vector3 = raw_base
		if not _finite_vector3(base):
			return false
		var offset: float = float(mote[1])
		var angle: float = candidate_phase * MOTE_PHASE_RATE + offset
		if not is_finite(angle):
			return false
		var world_point: Vector3 = Vector3(
			base.x + sin(angle) * MOTE_DRIFT_X,
			base.y,
			base.z + cos(angle) * MOTE_DRIFT_Z
		)
		if not _finite_vector3(world_point):
			return false
		if world_point.x < 0.8 or world_point.x > 19.2 or world_point.z < 0.8 or world_point.z > 13.2 or world_point.y < 0.4 or world_point.y > 1.8:
			continue
		var screen_point: Vector2 = projection.project(world_point)
		if not _finite_vector2(screen_point):
			return false
		var radius: float = float(mote[2]) * ppm / 48.0
		var mote_color: Color = MOTE_COLOR
		mote_color.a = float(mote[3])
		if not is_finite(radius) or radius <= 0.0 or not is_finite(mote_color.r) or not is_finite(mote_color.g) or not is_finite(mote_color.b) or not is_finite(mote_color.a):
			return false
		if not viewport_rect.has_point(screen_point):
			continue
		staged.append({"point": screen_point, "radius": radius, "color": mote_color})
	var final_frame: Dictionary = _capture_air_frame(canvas, snapshot, scene_data, projection)
	if final_frame.is_empty() or not _same_air_frame(frame, final_frame):
		return false
	_phase_seconds = candidate_phase
	for plan: Dictionary in staged:
		canvas.draw_circle(plan.get("point"), plan.get("radius"), plan.get("color"), true)
	return true

func _capture_air_frame(canvas, snapshot_value, scene_value, projection_value) -> Dictionary:
	if not canvas is CanvasItem or not is_instance_valid(canvas):
		return {}
	var canvas_item: CanvasItem = canvas
	if not canvas_item.is_inside_tree() or canvas_item.is_queued_for_deletion() or not canvas_item.visible or not canvas_item.is_visible_in_tree():
		return {}
	if not snapshot_value is Dictionary or not snapshot_value.has("entryToken") or snapshot_value.get("entryToken") != null or not P.counter(snapshot_value.get("worldEpoch"), 1) or not _valid_context(snapshot_value, scene_value, projection_value):
		return {}
	var host_value: Variant = canvas_item.get_parent()
	if not host_value is Node or not is_instance_valid(host_value):
		return {}
	var host: Node = host_value
	if not host.is_inside_tree() or host.is_queued_for_deletion() or not host.has_method("_eligible"):
		return {}
	var host_script: Variant = host.get_script()
	if not host_script is Script or host_script.resource_path != "res://scripts/ui/v1_main.gd" or host.get("renderer") != canvas_item:
		return {}
	var generation: Variant = host.get("generation")
	var host_snapshot: Variant = host.get("snapshot")
	var host_status: Variant = host.get("status")
	if not P.counter(generation, 1) or not host_snapshot is Dictionary or not P.snapshot(host_snapshot) or host_snapshot != snapshot_value:
		return {}
	if not host_status is Dictionary or not P.status(host_status) or not P.same(host_status, snapshot_value) or host_status.get("entryToken") != null:
		return {}
	var eligible: Variant = host.call("_eligible")
	if typeof(eligible) != TYPE_BOOL:
		return {}
	var canvas_snapshot: Variant = canvas_item.get("snapshot")
	var canvas_scene: Variant = canvas_item.get("scene")
	var current_key: Variant = canvas_item.get("current_key")
	var scene_hash: Variant = canvas_item.get("scene_hash")
	var live_projection: Variant = canvas_item.get("camera")
	var stored_viewport: Variant = canvas_item.get("viewport_rect")
	var reduced_motion: Variant = canvas_item.get("reduced_motion")
	if not canvas_snapshot is Dictionary or not P.snapshot(canvas_snapshot) or canvas_snapshot != snapshot_value:
		return {}
	if not canvas_scene is Dictionary or canvas_scene != scene_value or typeof(current_key) != TYPE_STRING or current_key != WORLD_ID + ":" + SCENE_ID + ":" + str(snapshot_value.get("worldEpoch")):
		return {}
	if typeof(scene_hash) != TYPE_STRING or scene_hash != SCENE_SHA256 or live_projection != projection_value or typeof(reduced_motion) != TYPE_BOOL:
		return {}
	var initialized: Variant = projection_value.get("initialized")
	if typeof(initialized) != TYPE_BOOL or not initialized:
		return {}
	var origin: Variant = projection_value.get("origin")
	var center: Variant = projection_value.get("screen_center")
	var camera_viewport: Variant = projection_value.get("viewport")
	var ppm: Variant = projection_value.get("ppm")
	if not origin is Vector3 or not center is Vector2 or not camera_viewport is Vector2 or not P.number(ppm, 0.000001, 256.0):
		return {}
	if not _finite_vector3(origin) or not _finite_vector2(center) or not _finite_vector2(camera_viewport) or camera_viewport.x <= 0.0 or camera_viewport.y <= 0.0:
		return {}
	var actual_viewport: Variant = canvas_item.get_viewport_rect()
	if not actual_viewport is Rect2 or not stored_viewport is Rect2 or actual_viewport != stored_viewport or not _finite_rect(actual_viewport) or actual_viewport.size.x <= 0.0 or actual_viewport.size.y <= 0.0 or camera_viewport != actual_viewport.size:
		return {}
	if not is_finite(_phase_seconds):
		return {}
	return {"canvas": canvas_item, "snapshot": snapshot_value.duplicate(true), "scene": scene_value.duplicate(true), "host": host, "status": host_status.duplicate(true), "generation": generation, "key": current_key, "hash": scene_hash, "projection": projection_value, "initialized": initialized, "origin": origin, "center": center, "camera_viewport": camera_viewport, "ppm": ppm, "viewport": actual_viewport, "eligible": eligible, "reduced_motion": reduced_motion, "phase": _phase_seconds}

func _same_air_frame(first: Dictionary, final: Dictionary) -> bool:
	return not first.is_empty() and not final.is_empty() and first == final

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
		_pending_delta = 0.0

func _finite_vector2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _finite_vector3(value: Vector3) -> bool:
	return is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _finite_rect(value: Rect2) -> bool:
	return _finite_vector2(value.position) and _finite_vector2(value.size) and _finite_vector2(value.end)
