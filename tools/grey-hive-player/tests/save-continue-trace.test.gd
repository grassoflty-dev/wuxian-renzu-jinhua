extends SceneTree

const P = preload('res://scripts/bridge/protocol.gd')
var checks: int = 0
var failures: int = 0
class Boundary extends RefCounted:
	var authenticated = true
	var connection_generation = 7
	var pending: Dictionary = {}
	var events: Array = []
	var visible = false
	var mode = 'start'
	var page_owner: Dictionary = {}
	var manifest: Dictionary = {'rs_core_room':true}
	var current: Dictionary = {}
	var source_hash = ''
	var allowed = true
	func request(method, _params, association):
		events.append('request:'+method)
		if not allowed: return 0
		var id: int = pending.size()+1
		pending[id]=association.duplicate(true)
		return id
	func build(_world,_scene): return allowed
	func contains(_point): return allowed
	func commit(_s,_scene,_hash): events.append('commit')
	func close_page(): visible=false
	func set_message(_message,_blocked=false): pass
	func suspend_world(): pass
	func ui_cue(_kind,_focus): pass
	func reset(_epoch): pass
	func quit(): events.append('quit')
func save_precedes_close(timeline: Array) -> bool:
	var event_index: int = timeline.find('trace:snapshot_applied:formal_save')
	var close_index: int = timeline.find('request:session_close')
	return event_index>=0 and close_index>=0 and event_index<close_index
func check(ok: bool, label: String) -> void:
	checks+=1
	if not ok:
		failures+=1
		printerr('FAIL '+label)
func function_text(text: String, name: String) -> String:
	var start: int = text.find('func '+name+'(')
	if start<0: return ''
	var end: int = text.find('\nfunc ',start+1)
	return text.substr(start) if end<0 else text.substr(start,end-start)
func fixture_snapshot() -> Dictionary:
	var point: Dictionary = {'xM':2.0,'yM':0.0,'zM':3.0}
	return {'kind':'full','protocolVersion':3,'schemaVersion':'freeze-v02-interfaces/1.2','worldId':'return_station','sceneId':'rs_core_room','worldEpoch':1,'checkpointId':null,'serverTick':10,'authorityRevision':2,'ackSeq':0,'entryToken':null,'actors':[],'doors':[],'interactables':[],'hazards':[],'objectives':[],'capabilities':{'schemaVersion':1},'progression':{'schemaVersion':1,'eventSeq':5,'currentWorldId':'return_station','worlds':[]},'player':{'entityId':'player','transform':{'positionM':point,'yawRad':0.0},'velocityMps':point,'actionState':'idle','currentHp':80,'maxHp':100,'currentEnergy':100,'maxEnergy':100,'facingX':0.0,'facingZ':1.0,'aimX':0.0,'aimZ':1.0}}
func receipt(command: String, s: Dictionary) -> Dictionary:
	return {'commandId':command,'applied':true,'alreadyApplied':false,'errorCode':null,'snapshot':s,'worldEpoch':s.worldEpoch,'serverTick':s.serverTick,'authorityRevision':s.authorityRevision}
func subject(script: GDScript, enabled: bool):
	var actual = script.new()
	for field in ['rpc','catalog','renderer','audio','panels','event_view','hud','tree']:
		var boundary := Boundary.new()
		boundary.events=actual.timeline
		actual.set(field,boundary)
	actual.generation=4
	actual.save_projection_generation=4
	actual.save_projection_connection=7
	actual.default_save_state='available'
	actual.snapshot=fixture_snapshot()
	actual.trace_enabled=enabled
	return actual
func _initialize() -> void:
	var text: String = FileAccess.get_file_as_string('res://scripts/ui/v1_main.gd').replace('\r\n','\n')
	if not text.contains('func _trace_save_flow('):
		check(false,'passive Save/Continue trace channel absent before implementation')
		print('SAVE_CONTINUE_TRACE checks='+str(checks)+' failures='+str(failures))
		quit(1)
		return
	var full: Script = load('res://scripts/ui/v1_main.gd')
	check(full is GDScript and full.can_instantiate(),'FULL production ScriptResource compile without main instantiation')
	var header: String = "extends RefCounted\nconst P=preload('res://scripts/bridge/protocol.gd')\nconst Rpc=preload('res://scripts/bridge/rpc_client.gd')\nconst Content=preload('res://scripts/ui/content_model.gd')\nconst ACTION_KEYS={}\nvar clock=100\nvar trace_rows: Array=[]\nvar timeline: Array=[]\nvar tree\n"
	for line in text.substr(0,text.find('func _ready()')).split('\n'):
		if line.begins_with('const ACTION_KEYS = '): header=header.replace('const ACTION_KEYS={}',line)
		if line.begins_with('var '):
			if line.begins_with('var catalog ') or line.begins_with('var event_view '): line='var '+line.get_slice(' ',1)
			header+=line+'\n'
	var selected: Array[String] = ['_context','_owner','_assoc','_current','_request','_begin_journey','_save_request','_response','_accept_snapshot','_submit_close','_close_current','_invalidate_saves','_save_scope_current','_default_continue_available','_slot_available','_writable_slot_params','_trace_save_flow']
	var body: String = ''
	for name in selected: body+=function_text(text,name)+'\n'
	# Exercise the exact UI admission prefix and the two named routes, not unrelated UI actions.
	var ui: String = function_text(text,'_ui_intent')
	body+=ui.substr(0,ui.find('\n\tmatch kind:'))+'\n\tmatch kind:\n'
	for line in ui.split('\n'):
		if line.begins_with("\t\t'continue_default':") or line.begins_with("\t\t'save_exit':"): body+=line+'\n'
	var boundaries: String = '''
func _release(): timeline.append('release')
func _clear_mouse_movement(): pass
func _key_held(_key): return false
func _theme_apply(_control,_world): pass
func _show(page,_extra={},_blocked=false):
	panels.mode=page
	panels.visible=true
func _feedback(message): last_feedback=message
func _hold(code):
	uncertain=true
	timeline.append('hold:'+code)
func _localized(code): return code
func _eligible(): return true
func _prepare_entry(_token): pass
func _publish_saves(_rebuild=true): pass
func _save_read_response(_id,_method,_result,_association): pass
func _interaction_result(_association,_receipt): pass
func _movement_identity_same(_a,_b): return false
func _guard_association_valid(_a): return false
func _guard_identity_same(_a,_b): return false
func _end_mouse_guard(): pass
func _open_paused(_page): pass
func get_tree(): return tree
'''
	# Only clock, UI application and output boundaries are substituted; P validators stay real.
	body=body.replace('Time.get_ticks_msec()','clock').replace('ThemeModel.apply(','_theme_apply(')
	body=body.replace("print('WUXIAN_SAVE_FLOW_TRACE:'+JSON.stringify(row))","trace_rows.append(row.duplicate(true)); timeline.append('trace:'+row.stage+':'+row.method)")
	var script := GDScript.new()
	script.source_code=header+boundaries+body
	check(script.reload()==OK,'actual request/response/snapshot application methods compile')
	if not script.can_instantiate():
		quit(1)
		return
	check(P.snapshot(fixture_snapshot()),'real protocol snapshot fixture valid')
	for mode in ['modal_owner','critical','uncertain','unavailable']:
		var guarded=subject(script,true)
		if mode=='modal_owner':
			guarded.panels.visible=true
			guarded.panels.page_owner={'generation':4}
		if mode=='critical': guarded.critical='formal_save'
		if mode=='uncertain': guarded.uncertain=true
		if mode=='unavailable': guarded.default_save_state='unknown'
		guarded._ui_intent('continue_default',null,{})
		check(guarded.rpc.pending.is_empty(),mode+' never queues Continue')
		check(guarded.trace_rows.size()==(1 if mode=='unavailable' else 0),mode+' distinguishes UI admission from availability/queue admission')
	var a=subject(script,true)
	a._ui_intent('continue_default',null,{'generation':99})
	check(a.trace_rows.is_empty() and a.rpc.pending.is_empty(),'stale UI owner produces no event or request')
	a._ui_intent('continue_default',null,{})
	check(a.trace_rows.size()==2 and a.trace_rows[0].stage=='ui_intent_admitted' and a.trace_rows[1].stage=='request_queued','admitted Continue then positive LOCAL queue event')
	var association: Dictionary = a.rpc.pending[1]
	var r: Dictionary = receipt('continue',fixture_snapshot())
	check(P.receipt(r) and P.success(r),'real successful receipt fixture valid')
	a._response(1,'formal_continue',r,association)
	check(a.trace_rows.size()==3 and a.trace_rows[2].stage=='snapshot_applied','matched Continue traces actual successful snapshot application')
	check(a.snapshot==r.snapshot and a.timeline.count('commit')==1,'snapshot assigned and renderer committed exactly once')
	check(a.timeline.find('commit')<a.timeline.find('trace:snapshot_applied:formal_continue'),'trace follows actual renderer commit')
	check(a.trace_rows[2].event_seq==5 and a.trace_rows[2].hp==80 and a.trace_rows[2].x==2.0,'bounded restored-state projection')
	for mode in ['stale','malformed','rejected','apply_failed','wrong_command']:
		var b=subject(script,true)
		var ar: Dictionary = {'generation':4,'context':b._context(),'purpose':'journey'}
		var response: Dictionary = receipt('continue',fixture_snapshot())
		if mode=='stale': ar.generation=3
		if mode=='malformed': response.applied='bad'
		if mode=='rejected':
			response.applied=false
			response.errorCode='E_COMMAND_REJECTED'
		if mode=='apply_failed': b.catalog.allowed=false
		if mode=='wrong_command': response.commandId='new'
		b._response(9,'formal_continue',response,ar)
		check(b.trace_rows.is_empty(),mode+' has no snapshot_applied event')
	var sequence_results: Array = []
	for enabled in [false,true]:
		var b=subject(script,enabled)
		b.journey_started=true
		b._ui_intent('save_exit',null,{})
		var ar: Dictionary = b.rpc.pending[1]
		b._response(1,'formal_save',receipt('save',fixture_snapshot()),ar)
		check(b.close_attempted and b.rpc.pending.size()==2,'existing save then close submission preserved '+str(enabled))
		if enabled:
			check(save_precedes_close(b.timeline),'save applied trace precedes actual close submission')
			check(not save_precedes_close(b.timeline.filter(func(v): return v!='trace:snapshot_applied:formal_save')),'missing save trace cannot pass ordering')
			check(not save_precedes_close(b.timeline.filter(func(v): return v!='request:session_close')),'missing close request cannot pass ordering')
		else: check(b.trace_rows.is_empty(),'trace OFF emits zero events')
		sequence_results.append({'timeline':b.timeline.filter(func(v): return not v.begins_with('trace:')),'snapshot':b.snapshot,'pending':b.rpc.pending,'close':b.close_pending})
	check(sequence_results[0]==sequence_results[1],'trace OFF/ON identical business state and request sequence')
	var refused=subject(script,true)
	refused.rpc.allowed=false
	refused._request('formal_save',{},'save')
	check(refused.trace_rows.is_empty(),'zero request ID is not queue admission')
	var cap=subject(script,true)
	for n in range(40): cap._trace_save_flow('request_queued','formal_continue',n+1)
	check(cap.trace_rows.size()==32 and cap.trace_save_records==32,'event subcap 32')
	cap._begin_journey('formal_continue',{})
	check(cap.trace_save_records==32 and cap.trace_started_ms==0 and cap.trace_rows.size()==32,'new journey does not reset trace window or event budget')
	var total=subject(script,true)
	total.trace_records=222
	total._trace_save_flow('request_queued','formal_continue',1)
	total._trace_save_flow('request_queued','formal_continue',2)
	check(total.trace_rows.size()==1 and total.trace_records==223,'shared original record cap retained')
	for time in [-1,120000,120001]:
		var clocked=subject(script,true)
		clocked.clock=time
		clocked._trace_save_flow('request_queued','formal_continue',1)
		check(clocked.trace_rows.is_empty(),'original window bound '+str(time))
	var privacy=subject(script,true)
	privacy._trace_save_flow('PRIVATE-PATH-TOKEN','formal_continue',1)
	privacy._trace_save_flow('request_queued','PRIVATE-PATH-TOKEN',1)
	check(privacy.trace_rows.is_empty(),'unknown stage/method cannot enter trace')
	var bad: Dictionary = fixture_snapshot()
	bad.worldId='PRIVATE-PATH-TOKEN'
	bad.sceneId='PRIVATE-PATH-TOKEN'
	bad.checkpointId={'secret':'PRIVATE-PATH-TOKEN'}
	bad.player.currentHp=NAN
	bad.player.transform.positionM={'xM':NAN,'yM':{},'zM':'PRIVATE-PATH-TOKEN'}
	bad.progression.eventSeq=INF
	privacy._trace_save_flow('snapshot_applied','formal_continue',1,{'applied':true,'alreadyApplied':false,'secret':'PRIVATE-PATH-TOKEN'},bad)
	check(privacy.trace_rows.size()==1 and not JSON.stringify(privacy.trace_rows).contains('PRIVATE-PATH-TOKEN'),'malicious projection fields omitted without coercion')
	check(not privacy.trace_rows[0].has('hp') and not privacy.trace_rows[0].has('x') and not privacy.trace_rows[0].has('event_seq'),'nonfinite/bad values omitted')
	for key in privacy.trace_rows[0]:
		check(key in ['stage','method','request_id','elapsed_ms','generation','connection_generation','applied','already_applied','world','scene','checkpoint_present','x','y','z','hp','event_seq'],'fixed payload key '+key)
	var logger: String = function_text(text,'_trace_save_flow')
	for forbidden in ['P.','_request(','_sample(','_accept_snapshot(','renderer.','_current(','str(']:
		check(not logger.contains(forbidden),'logger has no business call/coercion '+forbidden)
	print('SAVE_CONTINUE_TRACE checks='+str(checks)+' failures='+str(failures))
	quit(1 if failures else 0)
