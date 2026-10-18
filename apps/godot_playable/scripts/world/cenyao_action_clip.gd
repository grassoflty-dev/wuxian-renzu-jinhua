extends RefCounted
# Candidate only. Caller admits source art; this does not load or approve assets.
const P = preload('res://scripts/bridge/protocol.gd')
const Sampler = preload('res://scripts/world/frame_atlas_sampler.gd')
const DIRECTIONS = ['east','south_east','south','south_west','west','north_west','north','north_east']
const ACTIONS = ['primaryAttack','pulse','pierce','dash']
const FRESH_MS = 250 # Existing renderer action-presentation freshness ceiling.
var bindings: Dictionary = {}
var context: Array = []
var tick = -1
var revision = -1
var active: Dictionary = {}
var blocked_request = -1
var last_request = -1

func reset() -> void:
	context.clear()
	tick = -1
	revision = -1
	active.clear()
	blocked_request = -1
	last_request = -1

func clear() -> void:
	bindings.clear()
	reset()

func configure(action: String, metadata: Dictionary, texture: Texture2D, clip_id: String) -> bool:
	# One explicit full-action clip per exact native action, all eight directions.
	# Failed reconfiguration removes old art. No idle, inferred walk or guard clip.
	bindings.erase(action)
	if not action in ACTIONS:
		return false
	var clips = metadata.get('clips')
	if not clips is Array or clips.size()!=8:
		return false
	var seen: Dictionary = {}
	var full_duration = -1.0
	for clip in clips:
		if not clip is Dictionary or clip.get('id')!=clip_id or not clip.get('direction') in DIRECTIONS or seen.has(clip.get('direction')) or typeof(clip.get('loop'))!=TYPE_BOOL or clip.loop:
			return false
		seen[clip.direction] = true
		var frames = clip.get('frames')
		if not frames is Array or frames.size()<2 or frames.size()>Sampler.MAX_FRAMES:
			return false
		var duration = 0.0
		for frame in frames:
			if not frame is Dictionary or not P.number(frame.get('duration_ms'),1,60000):
				return false
			var rect = frame.get('rect')
			var anchor = frame.get('anchor')
			if not rect is Array or rect.size()!=4 or rect[2]!=336 or rect[3]!=560 or not anchor is Array or anchor.size()!=2 or anchor[0]!=168 or anchor[1]!=551:
				return false
			duration += float(frame.duration_ms)
		if full_duration<0:
			full_duration = duration
		elif duration!=full_duration:
			return false
	var sampler = Sampler.new()
	if not sampler.configure(metadata,texture):
		return false
	bindings[action] = {'sampler':sampler,'clip_id':clip_id,'duration':full_duration}
	return true

func observe(snapshot: Dictionary, now_ms: int) -> void:
	# Called once per accepted commit, never once per draw. No start inferred from
	# phase, facing, velocity, cooldown or the arrival of an atlas configuration.
	var player = snapshot.get('player')
	if now_ms<0 or not P.context(snapshot) or not P.counter(snapshot.get('serverTick')) or not P.counter(snapshot.get('authorityRevision')) or not player is Dictionary or not P.ident(player.get('entityId')):
		reset()
		return
	var next_context = [snapshot.worldId,snapshot.sceneId,snapshot.worldEpoch,player.entityId]
	if next_context!=context:
		reset()
		context = next_context
	if snapshot.serverTick<tick or snapshot.authorityRevision<revision:
		if not active.is_empty():
			blocked_request = active.request
		active.clear()
		return
	# Duplicate snapshots cannot keep a stale animation alive.
	if snapshot.serverTick==tick and snapshot.authorityRevision==revision:
		return
	tick = snapshot.serverTick
	revision = snapshot.authorityRevision
	var ap = player.get('actionPresentation')
	if snapshot.get('entryToken')!=null or not P.counter(player.get('currentHp'),1,4294967295) or typeof(player.get('actionState'))!=TYPE_STRING or not player.actionState in ACTIONS or not ap is Dictionary or not P.counter(ap.get('requestId'),1) or not ap.get('phase') in ['windup','active','recovery'] or not P.counter(ap.get('elapsedMs'),0,60000) or not P.counter(ap.get('durationMs'),1,60000) or ap.elapsedMs>=ap.durationMs:
		active.clear()
		return
	if ap.requestId==blocked_request:
		return
	var same_action = not active.is_empty() and active.request==ap.requestId
	# Native request IDs are monotonically claimed within a player context.
	# A canceled/completed request cannot be restarted by a later repeated packet.
	if not same_action and ap.requestId<=last_request:
		active.clear()
		return
	if same_action and (active.action!=player.actionState or active.duration!=ap.durationMs or ap.elapsedMs<active.elapsed or now_ms<active.accepted):
		blocked_request = ap.requestId
		active.clear()
		return
	var shown = active.shown if same_action else float(ap.elapsedMs)
	var accepted = active.accepted if same_action and ap.elapsedMs==active.elapsed else now_ms
	last_request = ap.requestId
	# Whole-action time only. Phase changes and direction changes do not restart.
	active = {'request':ap.requestId,'action':player.actionState,'elapsed':float(ap.elapsedMs),'duration':float(ap.durationMs),'accepted':accepted,'shown':shown}

func draw_body(canvas: CanvasItem, foot: Vector2, player: Dictionary, snapshot: Dictionary, ppm: float, reduced_motion: bool, now_ms: int) -> bool:
	if reduced_motion or active.is_empty() or not bindings.has(active.action) or not P.number(ppm,0.001,3072) or now_ms<active.accepted or now_ms-active.accepted>=FRESH_MS:
		return false
	if [snapshot.get('worldId'),snapshot.get('sceneId'),snapshot.get('worldEpoch'),player.get('entityId')]!=context or snapshot.get('serverTick')!=tick or snapshot.get('authorityRevision')!=revision or snapshot.get('player')!=player or snapshot.get('entryToken')!=null:
		return false
	var ap = player.get('actionPresentation')
	if not P.counter(player.get('currentHp'),1,4294967295) or player.get('actionState')!=active.action or not ap is Dictionary or not ap.get('phase') in ['windup','active','recovery'] or ap.get('requestId')!=active.request or ap.get('elapsedMs')!=active.elapsed or ap.get('durationMs')!=active.duration or not P.transform(player.get('transform')) or not P.number(player.get('facingX'),-1,1) or not P.number(player.get('facingZ'),-1,1):
		return false
	var binding: Dictionary = bindings[active.action]
	# Never stretch the art to an invented duration or invent phase boundaries.
	if binding.duration!=active.duration:
		return false
	var facing = Vector2(player.facingX-player.facingZ,(player.facingX+player.facingZ)*0.5)
	if facing.length_squared()==0:
		return false
	var sector = int(floor(atan2(facing.y,facing.x)/(PI/4.0)+0.5))
	var direction: String = DIRECTIONS[posmod(sector,8)]
	var elapsed = maxf(active.shown,active.elapsed+float(now_ms-active.accepted))
	# Latch visual completion for this request; a delayed owner update must not
	# resurrect a clip that already reached its end during bounded interpolation.
	active.shown = elapsed
	if elapsed>=active.duration:
		return false
	return binding.sampler.draw_clip(canvas,binding.clip_id,direction,elapsed,foot,(104.0/560.0)*(ppm/48.0),Color.WHITE,true)
