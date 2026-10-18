extends RefCounted
## Inert presentation candidate. No authority writes, input reads, file I/O or clock.
## A caller must prove the entire consumed interval is supported locomotion.
## Position/velocity alone are insufficient (conveyors, jumps and traversal exist).
const P = preload('res://scripts/bridge/protocol.gd')
const SCHEMA = 'wuxian.cenyao-locomotion-distance-candidate/v1'
const DIRECTIONS = ['east','south_east','south','south_west','west','north_west','north','north_east']
const MAX_FRAMES = 128
const CANVAS = Vector2(336,560)
const ANCHOR = Vector2(168,551)
var _texture: Texture2D
var _atlas_size = Vector2.ZERO
var _clips: Dictionary = {}
var _config: Dictionary = {}
var _context: Array = []
var _tick = -1
var _revision = -1
var _player: Dictionary = {}
var _last_clock_ms = -1
var _has_anchor = false
var _anchor = Vector3.ZERO
var _anchor_at_ms = -1
var _anchor_tick = -1
var _anchor_revision = -1
var _phase = 0.0
var _moving = false
var _movement_at_ms = -1

func suspend() -> void:
	# Keep consumed versions: pause/hide/stale cannot replay the same snapshot.
	_has_anchor = false
	_anchor = Vector3.ZERO
	_anchor_at_ms = -1
	_anchor_tick = -1
	_anchor_revision = -1
	_phase = 0.0
	_moving = false
	_movement_at_ms = -1

func reset() -> void:
	# Caller invokes at an explicit presentation-lifetime boundary only.
	suspend()
	_context.clear()
	_tick = -1
	_revision = -1
	_player.clear()
	_last_clock_ms = -1

func clear() -> void:
	_texture = null
	_atlas_size = Vector2.ZERO
	_clips.clear()
	_config.clear()
	suspend()

func configure(metadata: Dictionary, texture: Texture2D, approved: Dictionary) -> bool:
	# No guessed defaults. A failed binding removes old art and breaks continuity.
	# Preserve version high-water marks even across reconfiguration.
	clear()
	if texture==null or metadata.get('schema')!=SCHEMA or metadata.get('motion')!='forward_ground_locomotion' or not P.ident(metadata.get('clip_id')):
		return false
	if not P.number(approved.get('stride_m'),0.000001,1000000) or not P.number(approved.get('stop_distance_m'),0,1000000) or not P.number(approved.get('teleport_distance_m'),0.000001,1000000) or not P.counter(approved.get('stale_ms'),1,60000) or not P.number(approved.get('min_forward_dot'),0.000001,1):
		return false
	if approved.stop_distance_m>=approved.teleport_distance_m:
		return false
	var size = metadata.get('atlas_size')
	if not size is Array or size.size()!=2 or not P.counter(size[0],1,8192) or not P.counter(size[1],1,8192):
		return false
	var atlas_size = Vector2(float(size[0]),float(size[1]))
	if texture.get_size()!=atlas_size:
		return false
	var clips = metadata.get('clips')
	if not clips is Array or clips.size()!=8:
		return false
	var checked: Dictionary = {}
	var common_ends: Array = []
	for clip in clips:
		if not clip is Dictionary or not clip.get('direction') in DIRECTIONS or checked.has(clip.get('direction')) or typeof(clip.get('loop'))!=TYPE_BOOL or not clip.loop:
			return false
		var frames = clip.get('frames')
		if not frames is Array or frames.size()<2 or frames.size()>MAX_FRAMES:
			return false
		var compiled: Array = []
		var ends: Array = []
		var previous_end = 0.0
		for frame in frames:
			if not frame is Dictionary or not P.number(frame.get('phase_end'),0,1) or float(frame.phase_end)<=previous_end:
				return false
			var rect = frame.get('rect')
			var anchor = frame.get('anchor')
			if not rect is Array or rect.size()!=4 or not P.counter(rect[0],0,8192) or not P.counter(rect[1],0,8192) or not P.counter(rect[2],336,336) or not P.counter(rect[3],560,560) or not anchor is Array or anchor.size()!=2 or not P.counter(anchor[0],168,168) or not P.counter(anchor[1],551,551):
				return false
			var source = Rect2(float(rect[0]),float(rect[1]),336,560)
			if source.end.x>atlas_size.x or source.end.y>atlas_size.y:
				return false
			for prior in compiled:
				if prior.rect==source:
					return false
			previous_end = float(frame.phase_end)
			ends.append(previous_end)
			compiled.append({'rect':source,'phase_end':previous_end})
		if previous_end!=1.0:
			return false
		if common_ends.is_empty():
			common_ends = ends
		elif common_ends!=ends:
			# Facing changes retain the same admitted cycle phase/pose boundaries.
			return false
		checked[clip.direction] = compiled
	_texture = texture
	_atlas_size = atlas_size
	_clips = checked
	_config = {
		'clip_id':metadata.clip_id,
		'stride_m':float(approved.stride_m),
		'stop_distance_m':float(approved.stop_distance_m),
		'teleport_distance_m':float(approved.teleport_distance_m),
		'stale_ms':int(approved.stale_ms),
		'min_forward_dot':float(approved.min_forward_dot)
	}
	return true

func _clock_ok(now_ms: int) -> bool:
	if now_ms<0 or now_ms<_last_clock_ms:
		suspend()
		return false
	_last_clock_ms = now_ms
	return true

func _gate_ok(snapshot: Dictionary, gate: Dictionary) -> bool:
	# These are candidate caller facts, NOT new native protocol fields.
	# Never set ordinary_locomotion_confirmed from held input or nonzero velocity.
	if not P.context(gate) or not P.same(snapshot,gate) or not P.ident(gate.get('player_id')) or gate.player_id!=snapshot.player.entityId or not P.counter(gate.get('serverTick')) or not P.counter(gate.get('authorityRevision')) or gate.serverTick!=snapshot.serverTick or gate.authorityRevision!=snapshot.authorityRevision:
		return false
	for key in ['presentation_allowed','visible','reduced_motion','ordinary_locomotion_confirmed','higher_priority_body']:
		if typeof(gate.get(key))!=TYPE_BOOL:
			return false
	return gate.presentation_allowed and gate.visible and not gate.reduced_motion and gate.ordinary_locomotion_confirmed and not gate.higher_priority_body

func _player_ok(player: Dictionary, snapshot: Dictionary) -> bool:
	# Native idle means no timed combat action, not proof of walking.
	if snapshot.get('entryToken')!=null or not P.counter(player.get('currentHp'),1,4294967295) or player.get('actionState')!='idle' or player.get('actionPresentation')!=null or not P.transform(player.get('transform')) or not P.point(player.get('velocityMps')) or not P.number(player.get('facingX'),-1,1) or not P.number(player.get('facingZ'),-1,1):
		return false
	# This narrow forward-ground clip does not pretend to cover jumping/lifts.
	return player.velocityMps.yM==0 and (player.facingX!=0 or player.facingZ!=0)

func anchor_version() -> Dictionary:
	# Identifies the interval start for a caller evidence adapter. This identity
	# is NOT evidence that movement between the endpoints was supported.
	if not _has_anchor:
		return {}
	return {'worldId':_context[0],'sceneId':_context[1],'worldEpoch':_context[2],'player_id':_context[3],'serverTick':_anchor_tick,'authorityRevision':_anchor_revision}

func _native_gate(snapshot: Dictionary, local_gate: Dictionary) -> Dictionary:
	# This opt-in adapter consumes native evidence, not endpoint inference.
	# The caller still owns local eligibility and MUST suspend on every local
	# interruption, even when no snapshot, draw or sample is delivered.
	if _config.is_empty() or not P.snapshot(snapshot):
		return {}
	if not P.context(local_gate) or not P.same(snapshot,local_gate) or not P.ident(local_gate.get('player_id')) or local_gate.player_id!=snapshot.player.entityId:
		return {}
	for key in ['serverTick','authorityRevision']:
		if not P.counter(local_gate.get(key)) or local_gate[key]!=snapshot[key]:
			return {}
	for key in ['presentation_allowed','visible','reduced_motion','higher_priority_body']:
		if typeof(local_gate.get(key))!=TYPE_BOOL:
			return {}
	if not local_gate.presentation_allowed or not local_gate.visible or local_gate.reduced_motion or local_gate.higher_priority_body:
		return {}
	var certificate = snapshot.player.get('floorLocomotion')
	if not certificate is Dictionary or not P.counter(certificate.get('fromServerTick')) or not P.counter(certificate.get('fromAuthorityRevision')) or not P.number(certificate.get('minForwardDot'),-1,1):
		return {}
	if certificate.minForwardDot<_config.min_forward_dot:
		return {}
	# Validate numeric types and safe integral bounds BEFORE integer conversion.
	# JSON integral floats are valid counters; strings/booleans/fractions are not.
	var start_tick = int(certificate.fromServerTick)
	var start_revision = int(certificate.fromAuthorityRevision)
	var current_tick = int(snapshot.serverTick)
	var current_revision = int(snapshot.authorityRevision)
	if start_tick>=current_tick or start_revision>current_revision:
		return {}
	# Ordered safe counters make both signed subtractions checked and exact.
	if current_tick-start_tick!=current_revision-start_revision:
		return {}
	var gate = {
		'worldId':snapshot.worldId,'sceneId':snapshot.sceneId,
		'worldEpoch':snapshot.worldEpoch,'player_id':snapshot.player.entityId,
		'serverTick':snapshot.serverTick,'authorityRevision':snapshot.authorityRevision,
		'presentation_allowed':local_gate.presentation_allowed,'visible':local_gate.visible,
		'reduced_motion':local_gate.reduced_motion,'higher_priority_body':local_gate.higher_priority_body,
		'ordinary_locomotion_confirmed':true
	}
	# The actual helper anchor, never a caller-selected or fabricated endpoint.
	# Current qualification may seed a baseline when this history cannot cover
	# the anchor. observe() then discards that span without distance credit.
	var anchor = anchor_version()
	if anchor.is_empty() or not P.same(snapshot,anchor) or anchor.player_id!=snapshot.player.entityId:
		return gate
	if not P.counter(anchor.serverTick) or not P.counter(anchor.authorityRevision):
		return gate
	var anchor_tick = int(anchor.serverTick)
	var anchor_revision = int(anchor.authorityRevision)
	if start_tick>anchor_tick or anchor_tick>=current_tick or start_revision>anchor_revision or anchor_revision>=current_revision:
		return gate
	if anchor_tick-start_tick!=anchor_revision-start_revision:
		return gate
	gate['interval'] = {
		'from_serverTick':anchor_tick,'from_authorityRevision':anchor_revision,
		'to_serverTick':current_tick,'to_authorityRevision':current_revision,
		'ordinary_locomotion_continuous':true
	}
	return gate

func observe_native(snapshot: Dictionary, now_ms: int, local_gate: Dictionary) -> void:
	# Missing/malformed evidence still reaches observe(), consuming the version
	# and suspending motion. A duplicate cannot later upgrade discarded history.
	observe(snapshot,now_ms,_native_gate(snapshot,local_gate))

func sample_native(snapshot: Dictionary, now_ms: int, local_gate: Dictionary) -> Dictionary:
	# After observe advances the anchor to current, sample only needs current
	# qualification, not the already-consumed interval a second time.
	return sample(snapshot,now_ms,_native_gate(snapshot,local_gate))

func draw_body_native(canvas: CanvasItem, foot: Vector2, snapshot: Dictionary, ppm: float, now_ms: int, local_gate: Dictionary) -> bool:
	return draw_body(canvas,foot,snapshot,ppm,now_ms,_native_gate(snapshot,local_gate))

func _interval_ok(gate: Dictionary) -> bool:
	# Whole-interval evidence is required even for adjacent observed versions.
	# It must exclude interruptions, stops, side/back motion and passive carry,
	# not merely classify two endpoints. No input/tick-frequency inference.
	var interval = gate.get('interval')
	if not interval is Dictionary or typeof(interval.get('ordinary_locomotion_continuous'))!=TYPE_BOOL or not interval.ordinary_locomotion_continuous:
		return false
	for key in ['from_serverTick','from_authorityRevision','to_serverTick','to_authorityRevision']:
		if not P.counter(interval.get(key)):
			return false
	return interval.from_serverTick==_anchor_tick and interval.from_authorityRevision==_anchor_revision and interval.to_serverTick==_tick and interval.to_authorityRevision==_revision

func _baseline(position: Vector3, now_ms: int) -> void:
	suspend()
	_has_anchor = true
	_anchor = position
	_anchor_at_ms = now_ms
	_anchor_tick = _tick
	_anchor_revision = _revision

func observe(snapshot: Dictionary, now_ms: int, gate: Dictionary) -> void:
	# Accepted full snapshots only. Draw frequency never adds distance or phase.
	if not _clock_ok(now_ms):
		return
	if not P.snapshot(snapshot):
		suspend()
		return
	var player: Dictionary = snapshot.player
	var next_context = [snapshot.worldId,snapshot.sceneId,snapshot.worldEpoch,player.entityId]
	if next_context!=_context:
		reset()
		_context = next_context
		_last_clock_ms = now_ms
	if snapshot.serverTick<_tick or snapshot.authorityRevision<_revision:
		suspend()
		return
	var duplicate = snapshot.serverTick==_tick and snapshot.authorityRevision==_revision
	var tick_advanced = snapshot.serverTick>_tick
	var revision_advanced = snapshot.authorityRevision>_revision
	if duplicate:
		if player!=_player or not _gate_ok(snapshot,gate) or not _player_ok(player,snapshot) or not _config.is_empty() and _has_anchor and now_ms-_anchor_at_ms>=_config.stale_ms:
			suspend()
		return
	var previous_player: Dictionary = _player
	_tick = snapshot.serverTick
	_revision = snapshot.authorityRevision
	_player = player.duplicate(true)
	if _texture==null or _config.is_empty() or not _gate_ok(snapshot,gate) or not _player_ok(player,snapshot):
		suspend()
		return
	var position = P.vector(player.transform.positionM)
	if not tick_advanced or not revision_advanced:
		# Revision-only updates cannot accrue distance or refresh freshness. A change
		# to the player at that version breaks continuity rather than inventing time.
		if player!=previous_player:
			suspend()
		return
	if not _has_anchor or now_ms-_anchor_at_ms>=_config.stale_ms:
		_baseline(position,now_ms)
		return
	var delta = position-_anchor
	var horizontal = Vector2(delta.x,delta.z)
	var distance = horizontal.length()
	var velocity = Vector2(float(player.velocityMps.xM),float(player.velocityMps.zM))
	var facing = Vector2(float(player.facingX),float(player.facingZ))
	if not is_finite(distance) or delta.y!=0 or distance>=_config.teleport_distance_m:
		# Discontinuity consumes this version, then requires a new anchor snapshot.
		suspend()
		return
	if distance<=_config.stop_distance_m or velocity.length_squared()==0:
		_baseline(position,now_ms)
		return
	if not _interval_ok(gate):
		# Unknown/unproved span contributes no distance. Current qualification can
		# seed a new baseline, but duplicates cannot upgrade/replay this span.
		_baseline(position,now_ms)
		return
	if horizontal.normalized().dot(facing.normalized())<_config.min_forward_dot or velocity.normalized().dot(facing.normalized())<_config.min_forward_dot:
		# No invented strafe/backpedal clips and no direction inferred from motion.
		suspend()
		return
	_phase = fposmod(_phase+distance/_config.stride_m,1.0)
	_anchor = position
	_anchor_at_ms = now_ms
	_anchor_tick = _tick
	_anchor_revision = _revision
	_movement_at_ms = now_ms
	_moving = true

func sample(snapshot: Dictionary, now_ms: int, gate: Dictionary) -> Dictionary:
	if not _clock_ok(now_ms):
		return {}
	if _texture==null or _config.is_empty():
		return {}
	var player = snapshot.get('player')
	if not P.context(snapshot) or not P.counter(snapshot.get('serverTick')) or not P.counter(snapshot.get('authorityRevision')) or not player is Dictionary or [snapshot.get('worldId'),snapshot.get('sceneId'),snapshot.get('worldEpoch'),player.get('entityId')]!=_context or snapshot.get('serverTick')!=_tick or snapshot.get('authorityRevision')!=_revision or player!=_player:
		suspend()
		return {}
	if not _gate_ok(snapshot,gate) or not _player_ok(player,snapshot) or _has_anchor and now_ms-_anchor_at_ms>=_config.stale_ms or _texture.get_size()!=_atlas_size:
		suspend()
		return {}
	if not _moving:
		return {}
	if now_ms-_movement_at_ms>=_config.stale_ms:
		suspend()
		return {}
	var facing = Vector2(player.facingX-player.facingZ,(player.facingX+player.facingZ)*0.5)
	var sector = int(floor(atan2(facing.y,facing.x)/(PI/4.0)+0.5))
	var direction: String = DIRECTIONS[posmod(sector,8)]
	var frames: Array = _clips[direction]
	for index in range(frames.size()):
		if _phase<float(frames[index].phase_end):
			return {'clip_id':_config.clip_id,'direction':direction,'frame_index':index,'phase':_phase,'rect':frames[index].rect,'anchor':ANCHOR}
	return {}

func draw_body(canvas: CanvasItem, foot: Vector2, snapshot: Dictionary, ppm: float, now_ms: int, gate: Dictionary) -> bool:
	var packet = sample(snapshot,now_ms,gate)
	if packet.is_empty() or not is_instance_valid(canvas) or not is_finite(foot.x) or not is_finite(foot.y) or not P.number(ppm,0.001,3072):
		return false
	var factor = (104.0/560.0)*(ppm/48.0)
	canvas.draw_texture_rect_region(_texture,Rect2(foot-ANCHOR*factor,CANVAS*factor),packet.rect,Color.WHITE,false,true)
	return true
