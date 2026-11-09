extends RefCounted

const PATH := "res://assets/enemies/grey_hive/gate_b_brute_temporal_local_r1.png"
const ENTITY := "gh_gate_b_brute_01"
const TYPE := "enemy.grey_hive.brute"
var _attempted := false
var _texture: Texture2D
var _scope := ""
var _previous: Dictionary = {}

func _counter(v: Variant) -> bool:
	if not (v is int or v is float): return false
	var n := float(v)
	return is_finite(n) and n >= 0.0 and n <= 9007199254740991.0 and floor(n) == n

func _load_once() -> Texture2D:
	if _attempted: return _texture
	_attempted = true
	var t: Texture2D = null
	if ResourceLoader.exists(PATH): t = load(PATH) as Texture2D
	if t == null:
		var raw := Image.new()
		if raw.load(PATH) == OK and raw.get_format() == Image.FORMAT_RGBA8:
			t = ImageTexture.create_from_image(raw)
	if t != null and t.get_size() == Vector2(1254,1254):
		var im := t.get_image()
		if im != null and im.get_format() == Image.FORMAT_RGBA8: _texture = t
	return _texture

func reset() -> void:
	_scope = ""
	_previous.clear()

# Caller validates unique scene/spawn/visibility and provides ONLY currently
# active, unexpired authoritative presentation events from the same snapshot
# timeline; it owns tick-to-duration conversion, event filtering and expiry.
# No event survives here between calls. False means use the existing fallback.
func draw_body(canvas: CanvasItem, scene_key: String, epoch: int, tick: int, server_ms: int, actor: Dictionary, active_events: Array, foot: Vector2, ppm: float, animate_gait: bool = true) -> bool:
	if not is_instance_valid(canvas) or scene_key != "grey_hive:gh_gate_b:" + str(epoch) or epoch <= 0 or tick < 0 or server_ms < 0 or not foot.is_finite() or not is_finite(ppm) or ppm <= 0: return false
	if _scope != scene_key:
		_scope = scene_key
		_previous.clear()
	if actor.get("entityId") != ENTITY or actor.get("entityType") != TYPE or actor.get("active") != true:
		_previous.clear()
		return false
	var transform = actor.get("transform")
	if not transform is Dictionary or not transform.get("positionM") is Dictionary:
		_previous.clear()
		return false
	var pos: Dictionary = transform.positionM
	for key in ["xM","yM","zM"]:
		var v = pos.get(key)
		if not (v is int or v is float) or not is_finite(float(v)):
			_previous.clear()
			return false
	var position := Vector3(float(pos.xM),float(pos.yM),float(pos.zM))
	if not position.is_finite():
		_previous.clear()
		return false
	var moving := false
	if not _previous.is_empty():
		if tick < _previous.tick or server_ms < _previous.time:
			_previous.clear()
			return false
		if tick == _previous.tick and server_ms == _previous.time:
			if position != _previous.position:
				_previous.clear()
				return false
			moving = _previous.moving
		elif tick > _previous.tick and server_ms > _previous.time:
			moving = position.distance_squared_to(_previous.position) > 0.00000001
		else:
			_previous.clear()
			return false
	_previous = {"tick":tick,"time":server_ms,"position":position,"moving":moving}
	var chosen: Dictionary = {}
	var ids := {}
	for e in active_events:
		if not e is Dictionary or e.get("actorId") != ENTITY or not _counter(e.get("worldEpoch")) or int(e.worldEpoch) != epoch or not _counter(e.get("eventId")) or not _counter(e.get("serverTick")) or int(e.serverTick) > tick or not _counter(e.get("durationMs")) or int(e.durationMs) <= 0:
			return false
		var event_id := int(e.eventId)
		if ids.has(event_id): return false
		ids[event_id] = true
		var kind = e.get("kind")
		var attack = e.get("attackKind")
		var supported: bool = kind == "EnemyAttackTelegraph" and attack in ["slam","charge"] or kind == "EnemyChargeMotion" and attack == "charge" or kind == "EnemyAttackImpact" and attack in ["slam","charge"]
		if not supported: return false
		if chosen.is_empty() or int(e.serverTick) > int(chosen.serverTick) or (int(e.serverTick) == int(chosen.serverTick) and event_id > int(chosen.eventId)):
			chosen = e
	var cell := 0
	if not chosen.is_empty() and chosen.kind == "EnemyAttackTelegraph":
		cell = 2
	elif not chosen.is_empty() and chosen.kind == "EnemyAttackImpact" and chosen.attackKind == "slam":
		cell = 3
	elif moving and animate_gait:
		cell = int(server_ms / 166) % 2
	var texture := _load_once()
	if texture == null: return false
	var scale := 2.4 * ppm / 600.0
	if not is_finite(scale) or scale <= 0: return false
	var target := Rect2(foot - Vector2(314,610)*scale,Vector2(627,627)*scale)
	var source := Rect2(Vector2((cell % 2)*627,int(cell / 2)*627),Vector2(627,627))
	canvas.draw_texture_rect_region(texture,target,source,Color.WHITE,false,true)
	return true
