extends SceneTree
# SOURCE ONLY; future execution NOT_RUN and requires separate authority.
const EventView = preload('res://scripts/world/event_view.gd')
const P = preload('res://scripts/bridge/protocol.gd')
class FakeRenderer:
	extends RefCounted
	func visible_point(_point: Vector3) -> bool:
		return true
var failures: int = 0
var snapshot: Dictionary = {'worldId':'grey_hive','sceneId':'gh_gate_b','worldEpoch':7,'serverTick':100}
var renderer = FakeRenderer.new()
func _initialize() -> void:
	_mixed()
	_first_seed()
	_envelope_gates()
	_future()
	print('EVENT_VIEW_ID_CONTRACT failures='+str(failures))
	quit(0 if failures==0 else 1)
func _check(ok: bool, label: String) -> void:
	if not ok:
		failures += 1
		push_error(label)
func _event(id: Variant, reason: String='ok', tick: int=100) -> Dictionary:
	return {'protocolVersion':2,'eventId':id,'worldEpoch':7,'serverTick':tick,'kind':'ActionRejected',
		'positionM':{'xM':0.0,'yM':0.0,'zM':0.0},'directionRad':0.0,'radiusM':0.0,
		'intensity':1.0,'durationMs':2000,'combatFeedback':{'worldId':'grey_hive','sceneId':'gh_gate_b',
		'outcome':'rejected','sourceId':'player','requestId':1,'reason':reason}}
func _ready_view():
	var v = EventView.new()
	_accept(v,[],snapshot)
	v.accept_sound([{'protocolVersion':1,'eventId':17,'worldEpoch':7,'serverTick':100,
		'kind':'signal_ping','worldId':'grey_hive','sceneId':'gh_gate_b',
		'directionRad':0.0,'distanceM':1.0}],snapshot)
	_check(v.sound_cursor==17 and v.sound_seeded,'sound seed established through API')
	return v
func _accept(v, rows: Array, s: Dictionary) -> void:
	var before = var_to_bytes([rows,s])
	var sound_before = var_to_bytes([v.sound_cursor,v.sound_seeded,v.sound])
	v.accept(rows,s,renderer)
	_check(var_to_bytes([rows,s])==before,'input nonmutation including NaN')
	_check(var_to_bytes([v.sound_cursor,v.sound_seeded,v.sound])==sound_before,'sound state unchanged')
func _ids(rows: Array) -> Array:
	return rows.map(func(e): return e.eventId)
func _reasons(rows: Array) -> Array:
	return rows.map(func(e): return e.combatFeedback.reason)
func _mixed() -> void:
	var v = _ready_view()
	var rows: Array = [_event(3,'third'),_event(1,'first'),_event(2.0,'float-first'),_event(2,'int-later')]
	for bad in ['not-numeric','2',[],{},null,true,false,0,-1,1.5,INF,NAN,P.SAFE+1.0]:
		rows.insert(1,_event(bad,'invalid'))
	var missing = _event(9)
	missing.erase('eventId')
	rows.append(missing)
	_accept(v,rows,snapshot)
	_check(v.seeded and v.cursor==3 and _ids(v.active)==[1,2,3],'mixed valid IDs sorted and invalid IDs ignored')
	_check(_reasons(v.active)==['first','float-first','third'],'first duplicate row retained')
	_check(v.active.size()==3 and typeof(v.active[1].eventId)==TYPE_FLOAT,'integral float ID retained')
	_check(v.deferred.is_empty(),'mixed IDs do not create deferred rows')
	_accept(v,rows,snapshot)
	_check(v.cursor==3 and _ids(v.active)==[1,2,3],'seeded cursor suppresses replay')
func _first_seed() -> void:
	var v = EventView.new()
	_accept(v,[_event(3),_event('bad'),_event(1),_event(NAN),_event(2.0),_event(2)],snapshot)
	_check(v.seeded and v.cursor==3 and v.active.is_empty() and v.deferred.is_empty(),'first seed advances without display')
	_accept(v,[_event(3,'replay'),_event(4,'live')],snapshot)
	_check(v.cursor==4 and _ids(v.active)==[4] and _reasons(v.active)==['live'],'post-seed event accepted')
func _envelope_gates() -> void:
	for patch in [{'protocolVersion':3},{'worldEpoch':8},{'worldEpoch':true},{'serverTick':-1},
		{'kind':''},{'positionM':null},{'directionRad':NAN},{'radiusM':-1},{'intensity':3}]:
		var v = _ready_view()
		var bad = _event(4,'invalid',101)
		bad.merge(patch,true)
		_accept(v,[bad],snapshot)
		_check(v.cursor==0 and v.active.is_empty() and v.deferred.is_empty(),'invalid envelope or foreign epoch ignored: '+str(patch.keys()))
		_accept(v,[_event(4,'valid')],snapshot)
		_check(v.cursor==4 and _ids(v.active)==[4] and _reasons(v.active)==['valid'],'ignored row reserves no cursor')
	var v = _ready_view()
	var bad = _event(4,'invalid-first')
	bad.directionRad = NAN
	_accept(v,[bad,_event(4,'blocked-duplicate'),_event(5,'survivor')],snapshot)
	_check(v.cursor==5 and _ids(v.active)==[5] and _reasons(v.active)==['survivor'],'invalid first envelope still shadows same-batch duplicate')
func _future() -> void:
	var v = _ready_view()
	var bad = _event(3,'invalid-future',101)
	bad.intensity = 3
	_accept(v,[_event(5,'later'),bad,_event(4,'barrier',101)],snapshot)
	_check(v.cursor==0 and v.active.is_empty() and _ids(v.deferred)==[4,5],'only validated rows deferred in ID order')
	_accept(v,[_event(4,'replacement')],snapshot)
	_check(v.cursor==0 and v.active.is_empty() and _ids(v.deferred)==[4,5],'deferred first row survives duplicate and current tick')
	var next = snapshot.duplicate(true)
	next.serverTick = 101
	_accept(v,[],next)
	_check(v.cursor==5 and v.deferred.is_empty() and _ids(v.active)==[4,5],'future tick releases barrier and later ID')
	_check(_reasons(v.active)==['barrier','later'],'deferred row identity and reason preserved')
