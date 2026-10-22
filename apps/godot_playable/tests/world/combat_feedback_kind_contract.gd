extends SceneTree
# SOURCE ONLY; future execution NOT_RUN and requires separate authority.
const EventView = preload('res://scripts/world/event_view.gd')
const BRUTE = 'gh_gate_b_brute_01'
class FakeRenderer:
	extends RefCounted
	func visible_point(_point: Vector3) -> bool:
		return true
	func actor_visible(id: String, _include_inactive: bool=false) -> bool:
		return id=='gh_gate_b_brute_01'
var failures: int = 0
var next_id: int = 1
var expected: Array = []
var renderer = FakeRenderer.new()
var snapshot: Dictionary = {'worldId':'grey_hive','sceneId':'gh_gate_b','worldEpoch':7,'serverTick':100,
	'actors':[{'entityId':BRUTE,'entityType':'enemy.grey_hive.brute','actorKind':'enemy','active':true,
	'transform':{'positionM':{'xM':0.0,'yM':0.0,'zM':0.0},'yawRad':0.0}}]}
func _initialize() -> void:
	var v = EventView.new()
	_accept(v,[_event('EnemyAlert')])
	_check(v.seeded and v.cursor==1 and v.active.is_empty(),'first API seed suppresses display')
	_case(v,_event('EnemyAlert'),true,'ordinary omitted feedback')
	for feedback in [{},{'outcome':7,'worldId':false}]:
		var e = _event('EnemyAlert')
		e.combatFeedback = feedback
		_case(v,e,false,'dictionary extra rejected: '+str(feedback))
	for feedback in [null,false,0,1.5,'ignored',[],['ignored']]:
		var e = _event('EnemyAlert')
		e.combatFeedback = feedback
		_case(v,e,true,'non-dictionary extra retains behavior: '+str(feedback))
	for kind in ['ActionRejected','Hit','PulseHit','PierceHit','Damaged','GuardImpact','DamageAbsorbed']:
		var combat = _combat(kind)
		_case(v,combat,true,'legitimate '+kind)
		var spoof = _event('EnemyAlert')
		spoof.combatFeedback = combat.combatFeedback.duplicate(true)
		_case(v,spoof,false,'valid '+kind+' feedback on EnemyAlert')
	_check(next_id==26 and v.cursor==25,'all 25 input IDs unique and consumed through API')
	print('COMBAT_FEEDBACK_KIND_CONTRACT failures='+str(failures))
	quit(0 if failures==0 else 1)
func _check(ok: bool, label: String) -> void:
	if not ok:
		failures += 1
		push_error(label)
func _event(kind: String) -> Dictionary:
	var e: Dictionary = {'protocolVersion':2,'eventId':next_id,'worldEpoch':7,'serverTick':100,
		'kind':kind,'positionM':{'xM':0.0,'yM':0.0,'zM':0.0},'directionRad':0.0,
		'radiusM':0.5,'intensity':1.0,'durationMs':250}
	if kind=='EnemyAlert':
		e.actorId = BRUTE
	next_id += 1
	return e
func _combat(kind: String) -> Dictionary:
	var e = _event(kind)
	var outcome: String = {'ActionRejected':'rejected','Hit':'enemy_hit','PulseHit':'enemy_hit',
		'PierceHit':'enemy_hit','Damaged':'player_hurt','GuardImpact':'blocked','DamageAbsorbed':'absorbed'}[kind]
	var f: Dictionary = {'worldId':'grey_hive','sceneId':'gh_gate_b','outcome':outcome}
	e.radiusM = 1.0
	if kind=='ActionRejected':
		f.merge({'sourceId':'player','requestId':7,'reason':'cooldown'})
		e.durationMs = 1000
	elif kind in ['Hit','PulseHit','PierceHit']:
		f.merge({'sourceId':'player','targetId':BRUTE,'requestId':7})
		e.durationMs = 220
	else:
		f.merge({'sourceId':BRUTE,'targetId':'player'})
		e.durationMs = 160 if kind=='DamageAbsorbed' else 260
		if kind=='DamageAbsorbed':
			f.reason = 'invulnerable'
	e.combatFeedback = f
	return e
func _accept(v, rows: Array) -> void:
	var before = var_to_bytes([rows,snapshot])
	v.accept(rows,snapshot,renderer)
	_check(var_to_bytes([rows,snapshot])==before,'event and snapshot inputs unchanged')
func _case(v, e: Dictionary, allowed: bool, label: String) -> void:
	if allowed:
		expected.append(e.eventId)
	_accept(v,[e])
	_check(v.seeded and v.cursor==e.eventId,'cursor advances even on rejection: '+label)
	_check(v.deferred.is_empty(),'current-tick event is not deferred: '+label)
	_check(v.active.map(func(item): return item.eventId)==expected,'consumer-bound rows: '+label)
	if allowed and not v.active.is_empty() and v.active.back().eventId==e.eventId:
		var copy: Dictionary = v.active.back().duplicate(true)
		for key in ['_duration','_accepted','_initial_age']:
			copy.erase(key)
		_check(var_to_bytes(copy)==var_to_bytes(e),'accepted payload preserved: '+label)
