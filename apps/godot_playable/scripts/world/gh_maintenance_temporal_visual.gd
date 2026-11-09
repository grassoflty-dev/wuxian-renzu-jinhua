extends RefCounted

# Caller validates scene/spawn/visibility and supplies authoritative epoch, tick,
# server milliseconds and projected foot. False means draw the existing still.
const PATH := "res://assets/enemies/grey_hive/maintenance_temporal_local_r1.png"
const TYPE := "runtime2d.enemy.infected_maintenance_worker.v1"
var _attempted := false
var _texture: Texture2D
var _scope := ""
var _actors := {}

func _load_once() -> Texture2D:
	if _attempted:
		return _texture
	_attempted = true
	var t: Texture2D = null
	if ResourceLoader.exists(PATH):
		t = load(PATH) as Texture2D
	if t == null:
		var raw := Image.new()
		if raw.load(PATH) == OK and raw.get_format() == Image.FORMAT_RGBA8:
			t = ImageTexture.create_from_image(raw)
	if t != null and t.get_size() == Vector2(1254, 1254):
		var im := t.get_image()
		if im != null and im.get_format() == Image.FORMAT_RGBA8:
			_texture = t
	return _texture

func _safe_counter(value: Variant) -> bool:
	if not (value is int or value is float):
		return false
	var n := float(value)
	return is_finite(n) and n >= 0.0 and n <= 9007199254740991.0 and floor(n) == n
func reset() -> void:
	_scope = ""
	_actors.clear()

# actor supplies entityId, entityType, active, transform.positionM and the
# nonpersistent animationPresentation. tick/time must be authoritative, not wall-clock.
func draw_body(canvas: CanvasItem, scene_key: String, epoch: int, tick: int, server_ms: int, actor: Dictionary, foot: Vector2, ppm: float, animate_gait: bool = true) -> bool:
	if not is_instance_valid(canvas) or scene_key.is_empty() or epoch <= 0 or tick < 0 or server_ms < 0 or not foot.is_finite() or not is_finite(ppm) or ppm <= 0:
		return false
	var scope := scene_key + ":" + str(epoch)
	if scope != _scope:
		_scope = scope
		_actors.clear()
	var id = actor.get("entityId")
	if not id is String or id.is_empty() or actor.get("entityType") != TYPE or actor.get("active") != true:
		return false
	var trans = actor.get("transform")
	var a = actor.get("animationPresentation")
	if not trans is Dictionary or not a is Dictionary:
		_actors.erase(id)
		return false
	var pos = trans.get("positionM")
	if not pos is Dictionary:
		_actors.erase(id)
		return false
	for key in ["xM", "yM", "zM"]:
		var v = pos.get(key)
		if not (v is int or v is float) or not is_finite(float(v)):
			_actors.erase(id)
			return false
	var position := Vector3(float(pos.xM), float(pos.yM), float(pos.zM))
	if not position.is_finite():
		_actors.erase(id)
		return false
	var state = a.get("state")
	var rem = a.get("remainingMs")
	var serial = a.get("attackSerial")
	var attack = a.get("currentAttack")
	if state not in ["idle", "chasing", "windup", "cooldown"] or not _safe_counter(rem) or not _safe_counter(serial) or (attack != null and attack != "heavy_melee"):
		_actors.erase(id)
		return false
	rem = int(rem)
	serial = int(serial)
	if (state == "windup" and (rem == 0 or attack != "heavy_melee")) or (state == "cooldown" and (rem == 0 or attack != null)) or (state in ["idle", "chasing"] and (rem != 0 or attack != null)):
		_actors.erase(id)
		return false
	var previous: Dictionary = _actors.get(id, {})
	if not previous.is_empty() and (tick < previous.tick or server_ms < previous.time):
		_actors.erase(id)
		return false
	var moved := false
	var strike := false
	if not previous.is_empty() and tick > previous.tick and server_ms > previous.time:
		moved = state == "chasing" and previous.state == "chasing" and position.distance_squared_to(previous.pos) > 0.00000001
		strike = state == "cooldown" and previous.state == "windup" and previous.attack == "heavy_melee" and serial == previous.serial + 1
	elif not previous.is_empty() and tick == previous.tick and server_ms == previous.time:
		if previous.pos != position or previous.state != state or previous.serial != serial or previous.rem != rem:
			_actors.erase(id)
			return false
		moved = previous.moved
		strike = previous.strike
	else:
		if not previous.is_empty():
			_actors.erase(id)
			return false
	_actors[id] = {"tick": tick, "time": server_ms, "pos": position, "state": state, "serial": serial, "attack": attack, "rem": rem, "moved": moved, "strike": strike}
	var cell := -1
	if moved and animate_gait:
		cell = int(server_ms / 166) % 2
	elif state == "windup":
		cell = 2
	elif strike:
		cell = 3
	if cell < 0:
		return false
	var texture := _load_once()
	if texture == null:
		return false
	var scale := 1.72 * ppm / 600.0
	if not is_finite(scale) or scale <= 0:
		return false
	var target := Rect2(foot - Vector2(314, 610) * scale, Vector2(627, 627) * scale)
	var source := Rect2(Vector2((cell % 2) * 627, int(cell / 2) * 627), Vector2(627, 627))
	canvas.draw_texture_rect_region(texture, target, source, Color.WHITE, false, true)
	return true
