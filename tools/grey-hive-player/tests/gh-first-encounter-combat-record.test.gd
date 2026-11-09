extends SceneTree

# Exercise the production methods with only clock, UI and transport boundaries.
# This is a headless source fixture; it does not start a game or native service.
const P = preload('res://scripts/bridge/protocol.gd')
var checks = 0
var failures = 0

class Transport extends RefCounted:
	var authenticated = true
	var connection_generation = 3
	var pending: Dictionary = {}
	var calls: Array = []
	var sequence = 0
	var reject_inputs = 0
	func request(method, params, association):
		var admitted = true
		if method=='formal_submit_input' and reject_inputs>0:
			reject_inputs -= 1
			admitted = false
		if admitted:
			sequence += 1
		var id = sequence if admitted else 0
		calls.append({'id':id,'method':method,'params':params.duplicate(true),'association':association.duplicate(true)})
		if admitted:
			pending[id] = true
		return id

class CameraBoundary extends RefCounted:
	func inverse(_screen, _height): return Vector3(3,0,4)

class Peripheral extends RefCounted:
	var current: Dictionary = {}
	var manifest: Dictionary = {'gh_entry_maintenance':true,'other_scene':true}
	var source_hash = 'fixture'
	var camera = CameraBoundary.new()
	var cursor = 10
	var sound_cursor = 0
	var sound = []
	var events = []
	var failed = false
	var status_text = ''
	var pad_trace_enabled = false
	var hovered = false
	func gui_get_hovered_control(): return self if hovered else null
	func get_mouse_position(): return Vector2(100,100)
	func set_input_as_handled(): pass
	func contains(_position): return true
	func build(_world, _scene): return true
	func commit(_snapshot, _scene, _hash): pass
	func accept(_rows, _snapshot, _renderer): pass
	func accept_sound(_rows, _snapshot): pass
	func accept_events(_rows, _snapshot, _generation, _eligible): pass
	func reset(_epoch): pass
	func update(_snapshot, _scene, _generation, _eligible, _focused): pass
	func project(_snapshot, _renderer, _eligible): return []
	func clear_interaction_focus(): pass
	func set_interaction_focus(_row, _snapshot, _eligible, _generation): pass
	func suspend_world(): pass
	func ui_cue(_kind, _focused): pass
	func _clear_movement_captures(): pass
	func _cancel_guard_capture(): pass
	func update_view(_snapshot, _connection, _eligible, _hint, _scale, _interaction, _traversal, _actions, _guard, _movements, _pad, _owner): pass

class PanelsBoundary extends RefCounted:
	var visible = false
	var mode = 'pause'
	func set_message(_message, _confirm = false): pass
	func close_page(): visible = false

class TreeBoundary extends RefCounted:
	func quit(): pass

func check(ok: bool, label: String) -> void:
	checks += 1
	if not ok:
		failures += 1
		printerr('FAIL '+label)

func extract(source: String, name: String) -> String:
	var marker = 'func '+name+'('
	check(source.count(marker)==1,'one production method '+name)
	if source.count(marker)!=1:
		return ''
	var begin = source.find(marker)
	var end = source.find('\nfunc ',begin+1)
	return source.substr(begin) if end<0 else source.substr(begin,end-begin)

func make_script(source: String) -> GDScript:
	var header = "extends RefCounted\nconst P=preload('res://scripts/bridge/protocol.gd')\nconst Rpc=preload('res://scripts/bridge/rpc_client.gd')\nconst ThemeModel=preload('res://scripts/ui/theme_model.gd')\nvar clock=1000\nvar test_optin=''\nvar viewport\nvar trace_lines: Array=[]\n"
	for line in source.substr(0,source.find('func _ready()')).split('\n'):
		if line.begins_with('const ACTION_KEYS = ') or line.begins_with('const TRACE_KEYS = '):
			header += line+'\n'
		if line.begins_with('var '):
			if line.begins_with('var catalog ') or line.begins_with('var event_view '):
				line = 'var '+line.get_slice(' ',1)
			header += line+'\n'
	for line in source.split('\n'):
		if line.begins_with('const MOVEMENT_AXES = '): header += line+'\n'
	var names = ['_context','_owner','_assoc','_current','_request','_eligible','_process','_sample','_action','_send_action','_end_guard','_release','_event_key','_unhandled_input','_input','_keyboard_move_state','_movement_axes','_clear_mouse_movement','_clear_mouse_guard','_end_mouse_guard','_response','_accept_snapshot','_failed','_hold','_feedback','_localized','_close_current','_submit_close','_trace_start','_combat_record_drop','_combat_record_emit','_combat_record_finish','_combat_record_live','_combat_record_received','_combat_record_admission','_combat_record_sample','_combat_record_dispatch_map','_combat_record_action','_combat_record_response','_combat_record_failure','_combat_record_events','_combat_record_snapshot']
	var body = ''
	for name in names: body += extract(source,name)+'\n'
	body = body.replace('Time.get_ticks_msec()','clock').replace('Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)','false')
	body = body.replace("OS.get_environment('WUXIAN_V1_INPUT_TRACE')",'test_optin').replace('print(line)','trace_lines.append(line)')
	var boundaries = '''
func get_viewport(): return viewport
func get_tree(): return TreeBoundary.new()
func _key_held(_key): return false
func _mouse_guard_valid(): return false
func _mouse_movement_valid(): return false
func _observe_mouse_movement(): pass
func _observe_mouse_guard(): pass
func _trace_tick(): pass
func _trace_key(_event,_stage,_tag): pass
func _trace_pad_sample(_reason,_neutral,_axes=Vector2i.ZERO,_accepted=false): pass
func _trace_pad(_stage,_id,_reason,_facts={}): pass
func _trace_save_flow(_stage,_method,_id,_receipt={},_state={}): pass
func _trace_frontend(_stage,_facts): pass
func _trace_sink_close(): pass
func _trace_sink_start(): pass
func _trace_write(_kind,_stage,_tag,_elapsed): pass
func _current_hud_interaction(): return {}
func _current_hud_traversal(): return {}
func _current_hud_actions(): return []
func _current_hud_guard(): return {}
func _current_hud_movements(): return []
func _nearest(): return {}
func _interaction_prompt(_row): return ''
func _interact(): pass
func _escape(): pass
func _open_paused(_page): pass
func _show(page,_extra={},_blocked=false): panels.mode=page;panels.visible=true
func _prepare_entry(_token): pass
func _save_read_response(_id,_method,_result,_association): pass
func _save_read_current(_id,_method,_association): return false
func _set_save_read_state(_method,_state): pass
func _publish_saves(): pass
func _interaction_result(_association,_receipt): pass
'''
	# The tiny tree boundary is available to the generated script as a local class.
	header += 'class TreeBoundary extends RefCounted:\n\tfunc quit(): pass\n'
	var script = GDScript.new()
	script.source_code = header+boundaries+body
	var err = script.reload()
	check(err==OK and script.can_instantiate(),'instrumented production methods compile with test boundaries')
	return script

func make_snapshot() -> Dictionary:
	var point = {'xM':0.0,'yM':0.0,'zM':0.0}
	return {'kind':'full','protocolVersion':3,'schemaVersion':'freeze-v02-interfaces/1.2','worldId':'grey_hive','sceneId':'gh_entry_maintenance','worldEpoch':2,'checkpointId':null,'serverTick':10,'authorityRevision':2,'ackSeq':0,'entryToken':null,'actors':[],'doors':[],'interactables':[],'hazards':[],'objectives':[],'capabilities':{'schemaVersion':1},'progression':{'schemaVersion':1,'eventSeq':1,'currentWorldId':'grey_hive','worlds':[]},'player':{'entityId':'player','transform':{'positionM':point,'yawRad':0.0},'velocityMps':{'xM':0.0,'yM':0.0,'zM':0.0},'actionState':'idle','currentHp':100,'maxHp':100,'currentEnergy':100,'maxEnergy':100,'facingX':0.0,'facingZ':1.0,'aimX':0.0,'aimZ':1.0}}

func subject(script: GDScript, enabled: bool):
	var a = script.new()
	a.rpc = Transport.new()
	a.panels = PanelsBoundary.new()
	for field in ['catalog','renderer','audio','event_view','hud','viewport']: a.set(field,Peripheral.new())
	a.catalog.current = {'worldId':'grey_hive','sceneId':'gh_entry_maintenance'}
	a.snapshot = make_snapshot()
	a.status = {'worldId':'grey_hive','sceneId':'gh_entry_maintenance','worldEpoch':2,'paused':false,'lastLifecycleCommandSequence':0,'entryToken':null,'enhancementTerminalContext':null}
	a.journey_started = true
	a.generation = 4
	a.clock = 1000
	a.last_snapshot_at = 1000
	a.last_status_at = 1000
	a.test_optin = '1' if enabled else ''
	a._trace_start()
	return a

func click(a, unhandled: bool = true) -> void:
	var e = InputEventMouseButton.new()
	e.button_index = MOUSE_BUTTON_LEFT
	e.pressed = true
	a._input(e)
	if unhandled: a._unhandled_input(e)

func tick(a, at: int) -> void:
	a.clock = at
	a._process(0.05)

func calls(a, method: String) -> Array:
	return a.rpc.calls.filter(func(row): return row.method==method)

func records(a, stage: String) -> Array:
	var result: Array = []
	for line in a.trace_lines:
		var row = JSON.parse_string(line.trim_prefix('WUXIAN_GH_COMBAT_RECORD:'))
		if row is Dictionary and row.get('stage')==stage: result.append(row)
	return result

func receipt(s: Dictionary, accepted: bool) -> Dictionary:
	return {'commandId':'fixture-action','applied':accepted,'alreadyApplied':false,'errorCode':null if accepted else 'E_COOLDOWN','snapshot':s.duplicate(true),'worldEpoch':s.worldEpoch,'serverTick':s.serverTick,'authorityRevision':s.authorityRevision}

func event_row(id: int, kind: String, request_id, target: String, outcome: String, source: String = 'player') -> Dictionary:
	return {'eventId':id,'worldEpoch':2,'kind':kind,'combatFeedback':{'worldId':'grey_hive','sceneId':'gh_entry_maintenance','outcome':outcome,'sourceId':source,'targetId':target,'requestId':request_id,'reason':'no_target' if kind=='ActionRejected' else null}}

func _initialize() -> void:
	var production = load('res://scripts/ui/v1_main.gd')
	check(production is GDScript and production.can_instantiate(),'actual production script loads and compiles')
	var source = FileAccess.get_file_as_string('res://scripts/ui/v1_main.gd').replace('\r\n','\n')
	var script = make_script(source)
	if failures>0 or not script.can_instantiate(): finish(); return
	check(P.snapshot(make_snapshot()),'fixture uses a valid current snapshot')

	var off = subject(script,false)
	var on = subject(script,true)
	click(off); click(on)
	tick(off,1000); tick(on,1000)
	check(off.trace_lines.is_empty() and not off.combat_record_enabled,'default off produces no combat records')
	check(off.rpc.calls==on.rpc.calls,'opt-in leaves actual input and action calls unchanged')
	check(on.trace_records==0 and on.trace_sink_records==0,'combat records do not consume older trace counters')
	var inputs = calls(on,'formal_submit_input')
	var attacks = calls(on,'formal_submit_action')
	check(inputs.size()==1 and attacks.size()==1,'ordinary click samples then submits one action')
	if inputs.size()!=1 or attacks.size()!=1: finish(); return
	var received = records(on,'click_received')
	var admission = records(on,'click_admission')
	var samples = records(on,'input_sample')
	var submitted = records(on,'action_submitted')
	check(received.size()==1 and received[0].queued==null and received[0].eligible and not received[0].gui_hover,'received click records eligibility and initially unknown queue')
	check(admission.size()==1 and admission[0].queued and admission[0].click==1,'actual queue admission is observed')
	check(samples.size()==1,'one input sample is associated with the queued click')
	if samples.size()==1:
		check(samples[0].input_sequence==inputs[0].params.sample.seq and samples[0].input_rpc_id==inputs[0].id,'sample uses actual input sequence and transport ID')
		check(is_equal_approx(float(samples[0].aim_x),float(inputs[0].params.sample.aimX)) and is_equal_approx(float(samples[0].aim_z),float(inputs[0].params.sample.aimZ)),'serialized aim matches the submitted vector within float precision')
	check(submitted.size()==1 and submitted[0].action_request_id==attacks[0].params.action.requestId and submitted[0].action_rpc_id==attacks[0].id,'actual combat and transport request IDs are distinct recorded facts')
	check(received[0].world_epoch==2 and received[0].generation==4 and received[0].connection_generation==3,'record bound to actual scene epoch and session generations')

	on._response(attacks[0].id,'formal_submit_action',receipt(on.snapshot,true),attacks[0].association)
	var replies = records(on,'action_response')
	check(replies.size()==1 and replies[0].response=='accepted' and replies[0].click==1,'actual matched receipt records accepted without claiming a hit')
	var action_id = attacks[0].params.action.requestId
	var event_assoc = on._assoc({'purpose':'events'})
	on._response(200,'formal_presentation_events',[event_row(11,'Hit',action_id,'gh_worker_01','enemy_hit'),event_row(12,'Hit',null,'gh_worker_01','enemy_hit'),event_row(13,'Damaged',null,'player','player_hurt','gh_worker_01'),event_row(14,'Hit',999,'gh_worker_01','enemy_hit')],event_assoc)
	var events = records(on,'event')
	check(events.size()==3,'matching, unlinked and player-damage events recorded; foreign request ignored')
	if events.size()==3:
		check(events[0].click==1 and events[0].correlation=='matched' and events[0].event_id==11 and events[0].target_id=='gh_worker_01','matching Hit uses the real action and event IDs')
		check(events[1].click==null and events[1].correlation=='unknown' and events[1].action_request_id==null,'missing hit request remains unknown')
		check(events[2].event_kind=='Damaged' and events[2].click==null and events[2].correlation=='unknown','player damage is not attributed to an attack')
	on._response(201,'formal_presentation_events',[event_row(11,'Hit',action_id,'gh_worker_01','enemy_hit')],event_assoc)
	check(records(on,'event').size()==3,'repeated presentation event is not logged twice')
	var miss = subject(script,true)
	click(miss); tick(miss,1000)
	var miss_call = calls(miss,'formal_submit_action')[0]
	miss._response(202,'formal_presentation_events',[event_row(11,'ActionRejected',miss_call.params.action.requestId,'','rejected')],miss._assoc({'purpose':'events'}))
	var misses = records(miss,'event')
	check(misses.size()==1 and misses[0].click==1 and misses[0].correlation=='matched' and misses[0].event_kind=='ActionRejected' and misses[0].reason=='no_target','rejection event retains actual request event ID and reason')

	var hover = subject(script,true)
	hover.viewport.hovered = true
	click(hover)
	check(records(hover,'click_admission').size()==1 and not records(hover,'click_admission')[0].queued and records(hover,'click_admission')[0].gui_hover,'GUI-hover admission records not queued')
	var consumed = subject(script,true)
	click(consumed,false)
	check(records(consumed,'click_received')[0].queued==null and records(consumed,'click_admission').is_empty(),'GUI-consumed click remains queue unknown')
	var capped = subject(script,true)
	for n in range(4): click(capped,false)
	check(records(capped,'click_received').size()==3,'only first three encounter clicks are recorded')

	var rejected_input = subject(script,true)
	rejected_input.rpc.reject_inputs = 1
	click(rejected_input); tick(rejected_input,1000)
	check(calls(rejected_input,'formal_submit_action').is_empty() and records(rejected_input,'input_sample')[0].queued==false,'rejected input retains the queued attack')
	tick(rejected_input,1050)
	check(calls(rejected_input,'formal_submit_action').size()==1 and records(rejected_input,'action_submitted')[0].click==1,'later accepted sample binds the same queued edge')

	var error = subject(script,true)
	click(error); tick(error,1000)
	var error_call = calls(error,'formal_submit_action')[0]
	error._failed(error_call.id,'formal_submit_action','UNKNOWN_MUTATION',error_call.association)
	check(records(error,'action_error').size()==1 and records(error,'action_error')[0].response=='unknown','transport failure remains unknown')
	var refused = subject(script,true)
	click(refused); tick(refused,1000)
	var refused_call = calls(refused,'formal_submit_action')[0]
	refused._response(refused_call.id,'formal_submit_action',receipt(refused.snapshot,false),refused_call.association)
	check(records(refused,'action_response').size()==1 and records(refused,'action_response')[0].response=='rejected','valid negative action receipt records rejection')

	var hp = subject(script,true)
	click(hp,false)
	var hurt = hp.snapshot.duplicate(true); hurt.player.currentHp = 68; hurt.serverTick += 1
	check(hp._accept_snapshot(hurt),'actual snapshot path accepts observed health')
	check(records(hp,'player_hp').size()==1 and records(hp,'player_hp')[0].hp_before==100 and records(hp,'player_hp')[0].hp_after==68,'actual snapshot records observed HP decrease')
	var dead = hp.snapshot.duplicate(true); dead.player.currentHp = 0; dead.serverTick += 1
	check(hp._accept_snapshot(dead) and hp.combat_record_finished,'death ends recorder through the actual snapshot path')
	check(records(hp,'end').size()==1 and records(hp,'end')[0].stop_reason=='death','death termination is recorded')
	var after_death = hp.trace_lines.size(); click(hp,false)
	check(hp.trace_lines.size()==after_death and hp.combat_record_clicks.is_empty(),'no cross-death or retained click backlog')

	var scene = subject(script,true); click(scene,false)
	var next = scene.snapshot.duplicate(true); next.sceneId = 'other_scene'; next.worldEpoch = 3
	scene._combat_record_snapshot(next)
	check(scene.combat_record_finished and records(scene,'end')[0].stop_reason=='scene_changed','scene change ends recorder')
	var closed = subject(script,true); click(closed,false); closed._submit_close()
	check(closed.combat_record_finished and records(closed,'end')[0].stop_reason=='closure','actual close submission ends recorder')

	var timed = subject(script,true)
	timed.clock = 131000; timed.last_snapshot_at = 131000; timed.last_status_at = 131000
	click(timed,false)
	check(timed.combat_record_started_ms==131000 and timed.trace_lines.size()==1,'encounter window starts at first click, not trace startup')
	timed.clock = 250999
	check(timed._combat_record_live(),'recorder remains live before 120 seconds')
	timed.clock = 251000
	check(not timed._combat_record_live() and timed.combat_record_finished,'recorder stops at the 120 second limit')
	var bounded = subject(script,true); click(bounded,false)
	for n in range(40): bounded._combat_record_emit('event',0,{'event_id':n+1,'event_kind':'Hit','correlation':'unknown'})
	check(bounded.combat_record_count==32 and bounded.trace_lines.size()==32 and bounded.combat_record_bytes<=16384 and bounded.combat_record_finished,'32-record independent cap and byte ceiling')
	var byte_limited = subject(script,true); click(byte_limited,false)
	byte_limited.combat_record_bytes = 16380
	var before = byte_limited.trace_lines.size()
	byte_limited._combat_record_emit('event',0,{'event_id':1,'event_kind':'Hit','correlation':'unknown'})
	check(byte_limited.trace_lines.size()==before and byte_limited.combat_record_finished,'16 KiB cap rejects the next oversized record')
	finish()

func finish() -> void:
	print('GH_FIRST_ENCOUNTER_COMBAT_RECORD checks='+str(checks)+' failures='+str(failures))
	quit(1 if failures else 0)
