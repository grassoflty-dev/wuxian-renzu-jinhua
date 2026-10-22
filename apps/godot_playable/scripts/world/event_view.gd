extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const V = preload('res://scripts/world/presentation.gd')
const COMBAT = {'Hit':'enemy_hit','PulseHit':'enemy_hit','PierceHit':'enemy_hit','Damaged':'player_hurt','GuardImpact':'blocked','DamageAbsorbed':'absorbed','ActionRejected':'rejected'}
const ENEMY = ['EnemyAlert','EnemyAttackTelegraph','EnemyStagger','EnemyDeath','PressureShotMotion','FurnaceHoundLeapMotion','EnemyChargeMotion','EnemyLungeMotion','SignalShotMotion','SignalBlink','EnemyMemberHit','EnemyMemberDisperse','EnemyAttackImpact']
var epoch = -1
var cursor = 0
var sound_cursor = 0
var seeded = false
var sound_seeded = false
var active: Array = []
var deferred: Array = []
var sound: Dictionary = {}
var previous_visible = {}
func reset(new_epoch: int) -> void:
	epoch = new_epoch
	cursor = 0
	sound_cursor = 0
	seeded = false
	sound_seeded = false
	active.clear()
	deferred.clear()
	sound.clear()
	previous_visible.clear()
func envelope(e) -> bool:
	return e is Dictionary and P.counter(e.get('protocolVersion'),1,2) and P.counter(e.get('eventId'),1) and P.counter(e.get('worldEpoch')) and P.counter(e.get('serverTick')) and P.ident(e.get('kind')) and P.point(e.get('positionM')) and P.number(e.get('directionRad'),-10000,10000) and P.number(e.get('radiusM'),0,10000) and P.number(e.get('intensity'),0,2)
func accept(rows, s: Dictionary, renderer) -> void:
	if s.worldEpoch!=epoch:
		reset(int(s.worldEpoch))
	if not P.rows(rows,2048):
		return
	var ordered: Array = deferred.duplicate(true)
	deferred.clear()
	for row in rows:
		if row is Dictionary and P.counter(row.get('eventId'),1) and not ordered.any(func(item): return item.get('eventId')==row.get('eventId')):
			ordered.append(row)
	ordered.sort_custom(func(a,b): return a.get('eventId',0)<b.get('eventId',0) if a is Dictionary and b is Dictionary else false)
	var seed_only = not seeded
	var future_barrier = false
	for e in ordered:
		if not envelope(e) or e.worldEpoch!=epoch or e.eventId<=cursor:
			continue
		if not seed_only and (future_barrier or e.serverTick>s.serverTick):
			future_barrier = true
			if deferred.size()<256:
				deferred.append(e)
			continue
		cursor = int(e.eventId)
		if seed_only or not _authorized(e,s,renderer):
			continue
		var duration = _duration(e)
		if duration<=0 or (s.serverTick-e.serverTick)*1000.0/60>=duration:
			continue
		var item: Dictionary = e.duplicate(true)
		item['_duration'] = duration
		item['_accepted'] = Time.get_ticks_msec()
		item['_initial_age'] = (s.serverTick-e.serverTick)*1000.0/60
		if e.kind in ['EnemyStagger','EnemyDeath','EnemyAttackImpact']:
			active = active.filter(func(prior): return prior.get('actorId')!=e.get('actorId') or prior.get('memberId')!=null and prior.serverTick==e.serverTick)
		elif e.kind=='EnemyAttackTelegraph' or 'Motion' in e.kind:
			var terminal = false
			for prior in active:
				if prior.get('actorId')==e.get('actorId') and prior.kind in ['EnemyStagger','EnemyDeath','EnemyAttackImpact'] and prior.serverTick>=e.serverTick:
					terminal = true
			if terminal:
				continue
		active.append(item)
		if active.size()>64:
			active.pop_front()
	seeded = true
func _duration(e: Dictionary) -> float:
	if e.kind=='ResonanceWardenPulseImpact':
		return 600.0 # Candidate fog response, not native attack duration.
	if e.kind in COMBAT:
		return e.durationMs if P.counter(e.get('durationMs'),1,2000) else 0
	if e.kind in ENEMY:
		return e.durationMs if P.number(e.get('durationMs'),0.001,60000) else 0
	return {'SentinelAttackWindup':250,'SentinelHeavyWindup':400,'SentinelChargeWindup':884,'SentinelDeath':1000,'ForgedGuardEliteDeath':1000,'ForgedGuardPressureWindup':900,'PrimeRegulatorPressureWindup':1400,'ForgedGuardPressureImpact':200,'PrimeRegulatorPressureImpact':200}.get(e.kind,0)
func _authorized(e: Dictionary, s: Dictionary, renderer) -> bool:
	if e.get('protocolVersion')!=2 or not renderer.visible_point(P.vector(e.positionM)):
		return false
	if e.kind=='ResonanceWardenPulseImpact':
		return warden_pulse_authorized(e,s,renderer)
	if e.kind in COMBAT:
		var f = e.get('combatFeedback')
		if not f is Dictionary or f.get('worldId')!=s.worldId or f.get('sceneId')!=s.sceneId or f.get('outcome')!=COMBAT[e.kind] or not P.counter(e.get('durationMs'),1,2000):
			return false
		if f.get('requestId')!=null and not P.counter(f.requestId,1):
			return false
		if f.outcome=='rejected':
			return f.get('sourceId')=='player' and P.counter(f.get('requestId'),1) and typeof(f.get('reason'))==TYPE_STRING
		if f.outcome=='enemy_hit':
			return f.get('sourceId')=='player' and typeof(f.get('targetId'))==TYPE_STRING and (renderer.actor_visible(f.targetId,true) or previous_visible.has(f.targetId))
		if f.get('targetId')!='player' or typeof(f.get('sourceId'))!=TYPE_STRING:
			return false
		return f.outcome!='absorbed' or f.get('reason') in ['invulnerable','damage_reduced_to_zero']
	if e.kind in ENEMY:
		if not P.ident(e.get('actorId')) or not P.number(e.get('radiusM'),0.001,64) or not P.number(e.get('durationMs'),0.001,60000):
			return false
		var actor: Dictionary = {}
		for a in s.actors:
			if a.entityId==e.actorId:
				actor = a
		if actor.is_empty() or actor.actorKind!='enemy' or not renderer.actor_visible(actor.entityId,e.kind in ['EnemyDeath','EnemyMemberDisperse']):
			return false
		if e.kind=='EnemyDeath' and actor.active or e.kind not in ['EnemyDeath','EnemyMemberDisperse'] and not actor.active:
			return false
		var allowed = {'enemy.clockworks.pressure_drone':['pressure_shot'],'enemy.clockworks.furnace_hound':['bite','leap'],'enemy.grey_hive.brute':['charge','slam'],'enemy.grey_hive.swarm':['lunge'],'enemy.mist_harbor.tidebound':['tide_swing','tide_charge'],'enemy.mist_harbor.signal_wraith':['signal_shot']}
		if not _enemy_source(actor,s):
			return false
		if not allowed.has(actor.entityType):
			return false
		if e.get('attackKind')!=null and not e.attackKind in allowed[actor.entityType]:
			return false
		if e.kind=='EnemyAttackTelegraph' or 'Motion' in e.kind or e.kind=='EnemyAttackImpact':
			if not e.has('attackKind') or e.has('rangeM') and not P.number(e.rangeM,0,128) or e.kind=='EnemyAttackTelegraph' and not P.number(e.get('rangeM'),0.001,128):
				return false
		if e.get('attackKind')=='tide_swing' and not P.number(e.get('halfAngleRad'),0.001,PI):
			return false
		if actor.entityType=='enemy.mist_harbor.signal_wraith':
			if V.signal_positions(actor,s).is_empty() or not actor.signalPerception.precise:
				return false
		if e.kind in ['EnemyMemberHit','EnemyMemberDisperse']:
			var members = V.swarm_members(actor)
			var found = false
			for m in members:
				if m.memberId==e.get('memberId') and m.active==(e.kind=='EnemyMemberHit'):
					found = true
			return found
		if e.has('memberId'):
			return false
		var motion_type = {'PressureShotMotion':'enemy.clockworks.pressure_drone','FurnaceHoundLeapMotion':'enemy.clockworks.furnace_hound','SignalShotMotion':'enemy.mist_harbor.signal_wraith','SignalBlink':'enemy.mist_harbor.signal_wraith','EnemyLungeMotion':'enemy.grey_hive.swarm'}
		if motion_type.has(e.kind) and actor.entityType!=motion_type[e.kind]:
			return false
		if e.kind=='EnemyChargeMotion' and not actor.entityType in ['enemy.grey_hive.brute','enemy.mist_harbor.tidebound']:
			return false
		return true
	if e.kind.begins_with('Sentinel'):
		var v = V.sentinel(s)
		if v.is_empty():
			return false
		if e.kind=='SentinelDeath':
			return v.phase=='dead'
		var expected = {'SentinelAttackWindup':'light_windup','SentinelHeavyWindup':'heavy_windup','SentinelChargeWindup':'charge_windup'}.get(e.kind,'')
		if expected.is_empty() or v.phase!=expected or not v.warning is Dictionary:
			return false
		return e.get('actorId')==v.actorId and e.get('attackId')==v.attackSerial and P.vector(e.positionM).distance_to(P.vector(v.warning.originM))<=0.001 and abs(e.directionRad-v.warning.directionRad)<=0.001 and abs(e.radiusM-v.warning.radiusM)<=0.001 and P.number(e.get('rangeM')) and abs(e.rangeM-v.warning.rangeM)<=0.001
	if e.kind=='ForgedGuardEliteDeath':
		if s.worldId!='clockworks' or s.sceneId!='cw_forged_guard_arena':
			return false
		var elite = s.actors.filter(func(a): return a.entityId=='cw_forged_guard_elite')
		return elite.size()==1 and elite[0].entityType=='runtime2d.enemy.clockworks.forged_guard.v2' and not elite[0].active and renderer.actor_visible(elite[0].entityId,true) and P.vector(e.positionM).distance_to(P.vector(elite[0].transform.positionM))<=0.001
	if e.kind in ['ForgedGuardPressureWindup','ForgedGuardPressureImpact','PrimeRegulatorPressureWindup','PrimeRegulatorPressureImpact']:
		var id = 'cw_forged_guard_elite' if e.kind.begins_with('ForgedGuard') else 'cw_prime_regulator'
		var scene_id = 'cw_forged_guard_arena' if id=='cw_forged_guard_elite' else 'cw_regulator_core'
		return s.worldId=='clockworks' and s.sceneId==scene_id and e.radiusM>0 and e.radiusM<=8 and renderer.actor_visible(id)
	return false
func project(s: Dictionary, renderer, eligible: bool) -> Array:
	if not eligible:
		active.clear()
		deferred.clear()
		sound.clear()
		return []
	if not deferred.is_empty():
		accept([],s,renderer)
	var now = Time.get_ticks_msec()
	active = active.filter(func(e): return e.worldEpoch==s.worldEpoch and maxf((s.serverTick-e.serverTick)*1000.0/60,e._initial_age+now-e._accepted)<e._duration and _authorized(e,s,renderer))
	if not sound.is_empty() and (now>=sound.get('_until',0) or not V.mapping(s)):
		sound.clear()
	previous_visible.clear()
	for a in s.actors:
		if a.active and renderer.actor_visible(a.entityId):
			previous_visible[a.entityId] = true
	return active
func accept_sound(rows, s: Dictionary) -> void:
	if not P.rows(rows,2048):
		return
	var seed_only = not sound_seeded
	for e in rows:
		if not e is Dictionary or not P.counter(e.get('protocolVersion'),1,1) or not P.counter(e.get('eventId'),1) or e.get('worldEpoch')!=s.worldEpoch or not P.counter(e.get('serverTick')) or e.eventId<=sound_cursor:
			continue
		# Leave the future cue and later IDs available for the next snapshot poll.
		if e.serverTick>s.serverTick and not seed_only:
			break
		sound_cursor = int(e.eventId)
		if seed_only or not V.mapping(s) or e.get('worldId')!=s.worldId or e.get('sceneId')!=s.sceneId or not e.get('kind') in ['signal_ping','warden_true_call'] or not P.number(e.get('directionRad'),-PI,PI) or not P.number(e.get('distanceM'),0,10000) or e.kind=='warden_true_call' and (s.worldId!='mist_harbor' or s.sceneId!='mh_warden_arena'):
			continue
		sound = e.duplicate(true)
		sound['_until'] = Time.get_ticks_msec()+2400
	sound_seeded = true

func _enemy_source(a: Dictionary, s: Dictionary) -> bool:
	match a.entityType:
		'enemy.clockworks.pressure_drone','enemy.clockworks.furnace_hound':
			return s.worldId=='clockworks' and s.sceneId.begins_with('cw_')
		'enemy.grey_hive.brute':
			return s.worldId=='grey_hive' and s.sceneId=='gh_gate_b'
		'enemy.grey_hive.swarm':
			return s.worldId=='grey_hive' and s.sceneId in ['gh_lockdown','gh_deep_decon'] and a.entityId in [s.sceneId+'_swarm_01',s.sceneId+'_swarm_02'] and not V.swarm_members(a).is_empty()
		'enemy.mist_harbor.tidebound':
			var count = {'mh_drowned_quay':1,'mh_breakwater':2,'mh_resonance_tower':1}.get(s.sceneId,0)
			for n in range(count):
				if s.worldId=='mist_harbor' and a.entityId==s.sceneId+'_tidebound_0'+str(n+1):
					return true
		'enemy.mist_harbor.signal_wraith':
			return not V.signal_positions(a,s).is_empty()
	return false

# Actual impact only. Native wire deliberately has no actor/attack identity fields.
static func warden_pulse_authorized(e: Dictionary, s: Dictionary, renderer) -> bool:
	if not P.snapshot(s) or s.get('entryToken')!=null or s.worldId!='mist_harbor' or s.sceneId!='mh_warden_arena':
		return false
	if e.get('kind')!='ResonanceWardenPulseImpact' or e.get('protocolVersion')!=2 or e.get('worldEpoch')!=s.worldEpoch or not P.counter(e.get('eventId'),1) or not P.counter(e.get('serverTick')) or e.serverTick>s.serverTick:
		return false
	if not P.point(e.get('positionM')) or not P.number(e.get('radiusM'),3.5,3.5) or not P.number(e.get('intensity'),1,1) or not P.number(e.get('directionRad'),-PI,PI):
		return false
	for absent in ['actorId','memberId','attackId','attackKind','durationMs','combatFeedback','rangeM','halfAngleRad']:
		if e.get(absent)!=null:
			return false
	var origin: Vector3 = P.vector(e.positionM)
	if origin.x<0.5 or origin.x>23.5 or origin.z<0.5 or origin.z>15.5 or absf(origin.y)>0.001 or not renderer.visible_point(origin):
		return false
	var wardens: Array = s.actors.filter(func(a): return a.entityId=='mh_resonance_warden_staged_marker')
	if wardens.size()!=1:
		return false
	var actor: Dictionary = wardens[0]
	return actor.actorKind=='enemy' and actor.entityType=='runtime2d.enemy.mistharbor.signal_wraith.v2' and actor.active and renderer.actor_visible(actor.entityId)
