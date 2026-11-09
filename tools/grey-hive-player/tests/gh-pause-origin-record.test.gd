extends SceneTree

# Source-method fixture only: no native service, player session or real profile.
const P = preload('res://scripts/bridge/protocol.gd')
var checks = 0
var failures = 0

class Transport extends RefCounted:
	var authenticated = true
	var connection_generation = 7
	var pending: Dictionary = {}
	var calls: Array = []
	var sequence = 0
	var reject_pause = false
	func request(method, params, association):
		if method=='formal_pause' and reject_pause:
			calls.append({'id':0,'method':method,'params':params.duplicate(true),'association':association.duplicate(true)})
			return 0
		sequence += 1
		pending[sequence] = true
		calls.append({'id':sequence,'method':method,'params':params.duplicate(true),'association':association.duplicate(true)})
		return sequence

class CameraBoundary extends RefCounted:
	func inverse(_screen, _height): return Vector3.ZERO

class Peripheral extends RefCounted:
	var current: Dictionary = {}
	var manifest: Dictionary = {'rs_core_room':true,'gh_entry_maintenance':true}
	var source_hash = 'fixture'
	var camera = CameraBoundary.new()
	var cursor = 0
	var sound_cursor = 0
	var sound = []
	var events = []
	var failed = false
	var status_text = ''
	var pad_trace_enabled = false
	var window_focused = true
	func has_focus(): return window_focused
	func gui_get_hovered_control(): return null
	func get_mouse_position(): return Vector2.ZERO
	func set_input_as_handled(): pass
	func contains(_position): return true
	func build(_world, _scene): return true
	func commit(_snapshot, _scene, _hash): pass
	func accept(_rows, _snapshot, _renderer): pass
	func accept_sound(_rows, _snapshot): pass
	func accept_events(_rows, _snapshot, _generation, _eligible): pass
	func reset(_epoch): pass
	func clear_interaction_focus(): pass
	func suspend_world(): pass
	func stop_all(): pass
	func ui_cue(_kind, _focused): pass
	func configure(_muted, _volume): pass
	func _clear_movement_captures(): pass
	func _cancel_guard_capture(): pass

class PanelsBoundary extends RefCounted:
	var visible = false
	var mode = 'start'
	var page_owner: Dictionary = {}
	var preferences: Dictionary = {}
	var font_scale = 1.0
	var pending = false
	func show_page(page, owner, _snapshot, _connected, blocked = false, _preserve = false):
		mode = page
		page_owner = owner.duplicate(true)
		pending = blocked
		visible = true
	func close_page(): visible = false
	func set_message(_message, _confirm = false): pass

func check(ok: bool, label: String) -> void:
	checks += 1
	if not ok:
		failures += 1
		printerr('FAIL '+label)

func extract(source: String, name: String) -> String:
	var marker = 'func '+name+'('
	check(source.count(marker)==1,'one actual production method '+name)
	if source.count(marker)!=1: return ''
	var begin = source.find(marker)
	var end = source.find('\nfunc ',begin+1)
	return source.substr(begin) if end<0 else source.substr(begin,end-begin)

func make_script(source: String) -> GDScript:
	var header = "extends RefCounted\nconst P=preload('res://scripts/bridge/protocol.gd')\nconst Rpc=preload('res://scripts/bridge/rpc_client.gd')\nconst Content=preload('res://scripts/ui/content_model.gd')\nconst NOTIFICATION_WM_CLOSE_REQUEST=Node.NOTIFICATION_WM_CLOSE_REQUEST\nconst NOTIFICATION_APPLICATION_FOCUS_OUT=Node.NOTIFICATION_APPLICATION_FOCUS_OUT\nconst NOTIFICATION_APPLICATION_FOCUS_IN=Node.NOTIFICATION_APPLICATION_FOCUS_IN\nvar clock=1000\nvar test_optin=''\nvar viewport\nvar trace_lines: Array=[]\n"
	for line in source.substr(0,source.find('func _ready()')).split('\n'):
		if line.begins_with('const ACTION_KEYS = ') or line.begins_with('const TRACE_KEYS = '): header += line+'\n'
		if line.begins_with('var '):
			if line.begins_with('var catalog ') or line.begins_with('var event_view '): line = 'var '+line.get_slice(' ',1)
			header += line+'\n'
	for line in source.split('\n'):
		if line.begins_with('const MOVEMENT_AXES = ') or line.begins_with('const PLAYER_ACTION_LABELS = '): header += line+'\n'
	header += 'class TreeBoundary extends RefCounted:\n\tfunc quit(): pass\nclass ThemeModel:\n\tstatic func apply(_control,_world): pass\n'
	var names = ['_context','_owner','_assoc','_current','_request','_connection','_eligible','_sample','_action','_send_action','_end_guard','_release','_event_key','_unhandled_input','_input','_notification','_escape','_show','_open_paused','_resume','_back','_ui_intent','_begin_journey','_keyboard_move_state','_movement_axes','_clear_mouse_movement','_clear_mouse_guard','_end_mouse_guard','_response','_accept_snapshot','_failed','_hold','_feedback','_localized','_advance_lifecycle','_close_current','_submit_close','_trace_start','_trace_live','_trace_stop','_pause_record_forget','_pause_record_current','_pause_record_emit','_pause_record_event','_pause_record_begin','_pause_record_open','_pause_record_prepare','_pause_record_submitted','_pause_record_matches','_pause_record_response','_pause_record_failure','_pause_record_clear','_pause_record_display']
	var body = ''
	for name in names: body += extract(source,name)+'\n'
	body = body.replace('Time.get_ticks_msec()','clock').replace('Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)','false')
	body = body.replace("OS.get_environment('WUXIAN_V1_INPUT_TRACE')",'test_optin').replace('print(line)','trace_lines.append(line)')
	var boundaries = '''
func get_viewport(): return viewport
func get_window(): return viewport
func get_tree(): return TreeBoundary.new()
func _key_held(_key): return false
func _mouse_guard_valid(): return false
func _mouse_movement_valid(): return false
func _trace_key(_event,_stage,_tag): pass
func _trace_pad_sample(_reason,_neutral,_axes=Vector2i.ZERO,_accepted=false): pass
func _trace_pad(_stage,_id,_reason,_facts={}): pass
func _trace_save_flow(_stage,_method,_id,_receipt={},_state={}): pass
func _trace_frontend(_stage,_facts): pass
func _trace_sink_close(): pass
func _trace_sink_start(): pass
func _trace_write(_kind,_stage,_tag,_elapsed): pass
func _combat_record_finish(_reason): pass
func _combat_record_received(_event): pass
func _combat_record_admission(_event,_eligible,_hovered,_before): pass
func _combat_record_sample(_sample,_id): pass
func _combat_record_action(_kind,_action,_id): pass
func _combat_record_response(_id,_receipt,_association): pass
func _combat_record_failure(_id,_code,_association): pass
func _combat_record_events(_rows): pass
func _combat_record_snapshot(_snapshot): pass
func _publish_saves(_rebuild=true): pass
func _refresh_saves(): pass
func _invalidate_saves(): pass
func _prepare_entry(_token): pass
func _save_read_response(_id,_method,_result,_association): pass
func _save_read_current(_id,_method,_association): return false
func _set_save_read_state(_method,_state): pass
func _interaction_result(_association,_receipt): pass
func _movement_identity_same(_a,_b): return false
func _movement_capture_admitted(_value): return false
func _guard_association_valid(_value): return false
func _guard_identity_same(_a,_b): return false
func _current_hud_movements(): return []
func _current_hud_guard(): return {}
func _current_hud_actions(): return []
func _current_hud_traversal(): return {}
func _current_hud_interaction(): return {}
func _interact(): pass
func _default_continue_available(): return false
func _slot_available(_id): return false
func _writable_slot_params(_value): return false
func _save_request(_method,_params,_exit): pass
func _build_item(_value): pass
func _capability_select(_value): pass
func _choose_enhancement(_value): pass
func _commit_baizhi(_value): pass
func _close_baizhi(): pass
'''
	var script = GDScript.new()
	script.source_code = header+boundaries+body
	var err = script.reload()
	check(err==OK and script.can_instantiate(),'actual instrumented methods compile with test boundaries')
	return script

func snapshot(world: String = 'return_station', scene: String = 'rs_core_room', epoch: int = 2) -> Dictionary:
	var point = {'xM':0.0,'yM':0.0,'zM':0.0}
	return {'kind':'full','protocolVersion':3,'schemaVersion':'freeze-v02-interfaces/1.2','worldId':world,'sceneId':scene,'worldEpoch':epoch,'checkpointId':null,'serverTick':10,'authorityRevision':2,'ackSeq':0,'entryToken':null,'actors':[],'doors':[],'interactables':[],'hazards':[],'objectives':[],'capabilities':{'schemaVersion':1},'progression':{'schemaVersion':1,'eventSeq':1,'currentWorldId':world,'worlds':[]},'player':{'entityId':'player','transform':{'positionM':point,'yawRad':0.0},'velocityMps':{'xM':0.0,'yM':0.0,'zM':0.0},'actionState':'idle','currentHp':100,'maxHp':100,'currentEnergy':100,'maxEnergy':100,'facingX':0.0,'facingZ':1.0,'aimX':0.0,'aimZ':1.0}}

func subject(script: GDScript, enabled: bool):
	var a = script.new()
	a.rpc = Transport.new()
	a.panels = PanelsBoundary.new()
	for field in ['catalog','renderer','audio','event_view','hud','viewport']: a.set(field,Peripheral.new())
	a.catalog.current = {'worldId':'return_station','sceneId':'rs_core_room'}
	a.snapshot = snapshot()
	a.status = {'worldId':'return_station','sceneId':'rs_core_room','worldEpoch':2,'paused':false,'lastLifecycleCommandSequence':0,'entryToken':null,'enhancementTerminalContext':null}
	a.journey_started = true
	a.generation = 5
	a.clock = 1000
	a.last_snapshot_at = 1000
	a.last_status_at = 1000
	a.test_optin = '1' if enabled else ''
	a._trace_start()
	return a

func calls(a, method: String) -> Array:
	return a.rpc.calls.filter(func(row): return row.method==method)

func business_calls(a) -> Array:
	var result = a.rpc.calls.duplicate(true)
	for row in result:
		for key in ['pause_record_origin','pause_record_episode','pause_record_connection']: row.association.erase(key)
	return result

func rows(a, stage: String) -> Array:
	var result: Array = []
	for line in a.trace_lines:
		var row = JSON.parse_string(line.trim_prefix('WUXIAN_GH_PAUSE_RECORD:'))
		if row is Dictionary and row.get('stage')==stage: result.append(row)
	return result

func receipt(s: Dictionary, accepted: bool) -> Dictionary:
	return {'commandId':'pause','applied':accepted,'alreadyApplied':false,'errorCode':null if accepted else 'E_PAUSE_REJECTED','snapshot':s.duplicate(true),'worldEpoch':s.worldEpoch,'serverTick':s.serverTick,'authorityRevision':s.authorityRevision}

func press(a, key: int) -> void:
	var e = InputEventKey.new()
	e.physical_keycode = key
	e.keycode = key
	e.pressed = true
	a._input(e)

func _initialize() -> void:
	var production = load('res://scripts/ui/v1_main.gd')
	check(production is GDScript and production.can_instantiate(),'actual production script compiles')
	var source = FileAccess.get_file_as_string('res://scripts/ui/v1_main.gd').replace('\r\n','\n')
	var script = make_script(source)
	if failures>0 or not script.can_instantiate(): finish(); return
	check(P.snapshot(snapshot()),'fixture starts with a valid current snapshot')

	var off = subject(script,false)
	var on = subject(script,true)
	for a in [off,on]:
		a.clock = 121001
		a.last_snapshot_at = a.clock
		a.last_status_at = a.clock
		a.pending_actions = [{'kind':'primaryAttack','time':a.clock}]
		if a.trace_enabled: a._trace_live(a.clock)
		a._notification(Node.NOTIFICATION_APPLICATION_FOCUS_OUT)
	check(off.trace_lines.is_empty() and not off.pause_record_enabled,'default off emits no pause records')
	check(not on.trace_enabled and on.pause_record_enabled,'pause observation survives expiry of the old startup trace')
	check(calls(on,'formal_pause').size()==1 and calls(on,'formal_submit_input').size()==1 and on.pending_actions.is_empty() and not on.focused,'late focus-out keeps the actual release and pause path')
	check(business_calls(off)==business_calls(on) and off.focused==on.focused and off.critical==on.critical and off.panels.mode==on.panels.mode and off.pending_actions==on.pending_actions and off.status==on.status,'on/off meaningful RPC and gameplay state are equivalent')
	check(rows(on,'focus_out').size()==1 and rows(on,'focus_out')[0].origin=='focus_out','focus-out branch records its own origin')
	check(rows(on,'page_request').size()==1 and rows(on,'page_request')[0].origin=='focus_out','actual pause-page request keeps focus-out origin')
	var focus_call = calls(on,'formal_pause')[0]
	check(rows(on,'pause_request').size()==1 and rows(on,'pause_request')[0].request_id==focus_call.id and rows(on,'pause_request')[0].result=='queued','positive actual local ID is recorded only as queued')
	on._notification(Node.NOTIFICATION_APPLICATION_FOCUS_IN)
	check(rows(on,'focus_in').size()==1 and rows(on,'focus_in')[0].origin=='focus_in','focus-in branch is distinct')
	on._response(focus_call.id,'formal_pause',receipt(on.snapshot,true),focus_call.association)
	check(rows(on,'pause_receipt').size()==1 and rows(on,'pause_receipt')[0].result=='accepted' and rows(on,'pause_receipt')[0].request_id==focus_call.id,'matching actual receipt records accepted')
	check(rows(on,'page_display').size()==1 and rows(on,'page_display')[0].route=='receipt' and rows(on,'page_display')[0].origin=='focus_out' and on.panels.mode=='pause','actual displayed pause page retains only matched receipt origin')
	check(on.trace_records==0 and on.trace_sink_records==0 and on.combat_record_count==0,'old trace and combat budgets are untouched')
	on._resume()
	var resume_call = calls(on,'formal_resume')[0]
	var resumed_receipt = receipt(on.snapshot,true); resumed_receipt.commandId = 'resume'
	on._response(resume_call.id,'formal_resume',resumed_receipt,resume_call.association)
	check(on.pause_record_active==0 and on.pause_record_episodes==1 and not on.status.paused and not on.panels.visible,'actual accepted resume clears the first episode')
	press(on,KEY_ESCAPE)
	check(on.pause_record_active==2 and on.pause_record_episodes==2 and rows(on,'page_request')[-1].origin=='escape','next actual pause request begins the second episode')

	var escape = subject(script,true); press(escape,KEY_ESCAPE)
	check(rows(escape,'escape_press').size()==1 and rows(escape,'page_request')[0].origin=='escape' and calls(escape,'formal_pause').size()==1,'Escape press and its pause request are identified')
	var ui = subject(script,true); ui._ui_intent('pause',null,ui._owner())
	check(rows(ui,'ui_pause_intent').size()==1 and rows(ui,'page_request')[0].origin=='ui_pause' and calls(ui,'formal_pause').size()==1,'admitted UI pause is distinct from Escape and focus')
	var movement = subject(script,true); press(movement,KEY_S)
	check(movement.trace_lines.is_empty() and calls(movement,'formal_pause').is_empty() and not movement.pending_movement.is_empty(),'S movement alone produces no pause origin')
	var blocked_ui = subject(script,true); blocked_ui.critical = 'other'; blocked_ui._ui_intent('pause',null,blocked_ui._owner())
	check(rows(blocked_ui,'ui_pause_intent').is_empty() and calls(blocked_ui,'formal_pause').is_empty(),'blocked UI intent is not reported as admitted')

	var rejected = subject(script,true); press(rejected,KEY_ESCAPE)
	var rejected_call = calls(rejected,'formal_pause')[0]
	rejected._response(rejected_call.id,'formal_pause',receipt(rejected.snapshot,false),rejected_call.association)
	check(rows(rejected,'pause_receipt').size()==1 and rows(rejected,'pause_receipt')[0].result=='rejected' and rows(rejected,'page_display').is_empty(),'valid negative receipt records rejection without claiming display')
	var not_queued = subject(script,true); not_queued.rpc.reject_pause = true; press(not_queued,KEY_ESCAPE)
	check(rows(not_queued,'pause_request').size()==1 and rows(not_queued,'pause_request')[0].request_id==0 and rows(not_queued,'pause_request')[0].result=='not_queued' and rows(not_queued,'pause_receipt').is_empty(),'zero local request ID records not queued without a receipt')
	var missing = subject(script,true); press(missing,KEY_ESCAPE)
	var missing_call = calls(missing,'formal_pause')[0]
	missing._connection(false,'E_TEST_DISCONNECTED')
	check(rows(missing,'pause_receipt').size()==1 and rows(missing,'pause_receipt')[0].request_id==missing_call.id and rows(missing,'pause_receipt')[0].result=='unknown' and missing.pause_record_pending_id==0,'connection loss closes a missing receipt as unknown')
	var failure = subject(script,true); press(failure,KEY_ESCAPE)
	var failed_call = calls(failure,'formal_pause')[0]
	failure._failed(failed_call.id,'formal_pause','UNKNOWN_MUTATION',failed_call.association)
	check(rows(failure,'pause_receipt').size()==1 and rows(failure,'pause_receipt')[0].result=='unknown','transport failure remains unknown')

	var stale = subject(script,true); press(stale,KEY_ESCAPE)
	var old_call = calls(stale,'formal_pause')[0]
	var new_snapshot = snapshot('grey_hive','gh_entry_maintenance',3)
	check(stale._accept_snapshot(new_snapshot,true) and stale.pause_record_pending_id==0,'actual context transition clears the old association')
	var old_count = rows(stale,'pause_receipt').size()
	stale._response(old_call.id,'formal_pause',receipt(snapshot(),true),old_call.association)
	check(rows(stale,'pause_receipt').size()==old_count,'late old-context receipt cannot attach to the new context')
	var session = subject(script,true); press(session,KEY_ESCAPE)
	session.critical = ''; session.panels.visible = false
	session._begin_journey('formal_new',{})
	check(session.pause_record_pending_id==0 and session.pause_record_active==0,'actual new-journey boundary clears pause association')
	var closing = subject(script,true); press(closing,KEY_ESCAPE)
	closing.critical = ''; closing._submit_close()
	check(closing.pause_record_pending_id==0 and closing.pause_record_active==0 and rows(closing,'pause_receipt')[0].result=='unknown','actual closure clears pending association without inventing acceptance')

	var direct = subject(script,true); direct.status.paused = true; direct.panels.visible = true; direct.panels.mode = 'settings'
	direct._back()
	check(rows(direct,'page_display').size()==1 and rows(direct,'page_display')[0].route=='back' and rows(direct,'page_display')[0].origin=='unknown','direct back display is not misattributed to focus')
	var already = subject(script,true); already.status.paused = true; already._open_paused('pause',{},'ui_pause')
	check(rows(already,'page_display').size()==1 and rows(already,'page_display')[0].route=='already_paused' and rows(already,'page_display')[0].origin=='ui_pause' and calls(already,'formal_pause').is_empty(),'already-paused display records its explicit direct route')

	var episode = subject(script,true)
	check(episode._pause_record_begin(),'first pause episode begins')
	episode._pause_record_event('focus_out','focus_out')
	var prior_count = episode.pause_record_count; var prior_bytes = episode.pause_record_bytes
	episode._pause_record_clear()
	check(episode.pause_record_count==prior_count and episode.pause_record_bytes==prior_bytes,'lifecycle clear never resets global budgets')
	check(episode._pause_record_begin(),'second pause episode begins')
	episode._pause_record_clear()
	check(not episode._pause_record_begin() and episode.pause_record_episodes==2,'third pause episode is not recorded')
	var bounded = subject(script,true)
	for n in range(30): bounded._pause_record_event('focus_out','focus_out')
	check(bounded.pause_record_count==16 and bounded.trace_lines.size()==16 and bounded.pause_record_bytes<=8192 and bounded.pause_record_exhausted,'independent 16-record and byte limits')
	var bytes = subject(script,true); bytes.pause_record_bytes = 8190
	bytes._pause_record_event('focus_out','focus_out')
	check(bytes.trace_lines.is_empty() and bytes.pause_record_exhausted,'8192-byte limit rejects an oversized next record')
	finish()

func finish() -> void:
	print('GH_PAUSE_ORIGIN_RECORD checks='+str(checks)+' failures='+str(failures))
	quit(1 if failures else 0)
