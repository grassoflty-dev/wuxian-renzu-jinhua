extends RefCounted
const P = preload("res://scripts/bridge/protocol.gd")
const V = preload("res://scripts/world/presentation.gd")
const PATH := "res://assets/enemies/mist_harbor/signal_wraith_temporal_local_r1.png"
const TYPE := "enemy.mist_harbor.signal_wraith"
const SCENES := ["mh_tidal_warehouse", "mh_signal_yard", "mh_breakwater", "mh_resonance_tower"]
var _attempted := false
var _texture: Texture2D
var _scope := ""
var _actors := {}

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
	_actors.clear()

# Caller MUST enforce actor_visible, _mh_cw_body_key and no body/HP fallback on
# uncertainty; reset on invisibility, uncertainty, removal or scene changes.
# active_events contain ONLY matching currently active/unexpired authoritative
# events; caller owns timeline conversion/expiry, no event is cached here.
# project maps the sole authorized V.signal_positions point to canvas space.
func draw_body(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, actor: Dictionary, authorized_visible: bool, active_events: Array, project: Callable, ppm: float) -> bool:
	if not authorized_visible or not is_instance_valid(canvas) or not project.is_valid() or not is_finite(ppm) or ppm <= 0:
		reset()
		return false
	var perception = actor.get("signalPerception")
	if not perception is Dictionary or perception.get("precise") != true:
		reset()
		return false
	var sid = snapshot.get("sceneId")
	var epoch = snapshot.get("worldEpoch")
	var tick = snapshot.get("serverTick")
	var tm = snapshot.get("serverTimeMs")
	if snapshot.get("worldId") != "mist_harbor" or sid not in SCENES or scene.get("worldId") != "mist_harbor" or scene.get("sceneId") != sid or not _counter(epoch) or int(epoch) <= 0 or not _counter(tick) or not _counter(tm):
		reset()
		return false
	var scope := str(sid) + ":" + str(int(epoch))
	if _scope != scope:
		_scope = scope
		_actors.clear()
	var id = actor.get("entityId")
	if not id is String or id.is_empty() or actor.get("entityType") != TYPE or actor.get("actorKind") != "enemy" or actor.get("active") != true or not P.transform(actor.get("transform")):
		reset()
		return false
	var actors = snapshot.get("actors")
	var spawns = scene.get("spawns")
	if not actors is Array or not spawns is Array:
		reset()
		return false
	var matches: Array = actors.filter(func(row): return row is Dictionary and row.get("entityId") == id)
	var authored: Array = spawns.filter(func(row): return row is Dictionary and row.get("id") == id and row.get("kind") == "enemy" and row.get("entityType") == TYPE)
	if matches.size()!=1 or matches[0]!=actor or authored.size()!=1:
		reset()
		return false
	var positions: Array = V.signal_positions(actor,snapshot)
	if positions.size()!=1 or not P.point(positions[0]):
		reset()
		return false
	var position: Vector3 = P.vector(positions[0])
	var projected = project.call(position)
	if not projected is Vector2 or not projected.is_finite():
		reset()
		return false
	var chosen: Dictionary = {}
	var blink := false
	var ids := {}
	for e in active_events:
		if not e is Dictionary or e.get("actorId") != id or not _counter(e.get("worldEpoch")) or int(e.worldEpoch)!=int(epoch) or not _counter(e.get("eventId")) or not _counter(e.get("serverTick")) or int(e.serverTick)>int(tick) or not _counter(e.get("durationMs")) or int(e.durationMs)<=0:
			reset()
			return false
		var event_id := int(e.eventId)
		if ids.has(event_id):
			reset()
			return false
		ids[event_id] = true
		var kind = e.get("kind")
		if kind == "SignalBlink":
			blink = true
			continue
		if kind not in ["EnemyAttackTelegraph", "SignalShotMotion", "EnemyAttackImpact"] or e.get("attackKind") != "signal_shot":
			reset()
			return false
		if chosen.is_empty() or int(e.serverTick)>int(chosen.serverTick) or (int(e.serverTick)==int(chosen.serverTick) and event_id>int(chosen.eventId)): chosen = e
	var previous: Dictionary = _actors.get(id,{})
	var moving := false
	if blink:
		_actors.erase(id)
	elif not previous.is_empty():
		if int(tick)<previous.tick or int(tm)<previous.time:
			reset()
			return false
		if int(tick)==previous.tick and int(tm)==previous.time:
			if position!=previous.position:
				reset()
				return false
			moving = previous.moving
		elif int(tick)>previous.tick and int(tm)>previous.time:
			moving = position.distance_squared_to(previous.position)>0.00000001
		else:
			reset()
			return false
	_actors[id] = {"tick":int(tick),"time":int(tm),"position":position,"moving":moving}
	var cell := -1
	if not chosen.is_empty() and chosen.kind == "EnemyAttackTelegraph": cell = 2
	elif not chosen.is_empty() and chosen.kind in ["SignalShotMotion", "EnemyAttackImpact"]: cell = 3
	elif moving: cell = int(int(tm)/166)%2
	if cell < 0: return false
	var tex := _load_once()
	if tex == null: return false
	var scale := 1.72*ppm/600.0
	if not is_finite(scale) or scale<=0: return false
	var target := Rect2(projected-Vector2(384,610)*scale,Vector2(627,627)*scale)
	var source := Rect2(Vector2((cell%2)*627,int(cell/2)*627),Vector2(627,627))
	canvas.draw_texture_rect_region(tex,target,source,Color.WHITE,false,true)
	return true
