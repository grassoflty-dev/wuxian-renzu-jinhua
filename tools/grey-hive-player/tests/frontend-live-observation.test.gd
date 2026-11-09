extends SceneTree
const P=preload('res://scripts/bridge/protocol.gd')
var checks: int=0
var failures: int=0
class Sink extends RefCounted:
	var events: Array=[]
	var binding='attempt-A'
	var mode='ok'
	var pending=''
	var written=0
	var error=OK
	var closed=false
	func get_length(): return 1 if mode=='nonempty' else written
	func get_position(): return written
	func get_error(): return error
	func store_buffer(bytes: PackedByteArray):
		pending=bytes.get_string_from_utf8()
		events.append({'type':'write','line':pending,'binding':binding})
		written+=bytes.size()
		if mode=='write': error=ERR_FILE_CANT_WRITE
	func flush():
		if mode=='flush': error=ERR_FILE_CANT_WRITE; return
		events.append({'type':'flush','line':pending,'binding':binding})
		pending=''
	func close(): closed=true; events.append({'type':'close','binding':binding})
class Storage extends RefCounted:
	var path='C:/fixture/attempt-A/frontend-live-observation.jsonl'
	var events: Array=[]
	var mode='ok'
	var handle
	func open(destination,access):
		events.append({'type':'open','binding':'attempt-A','access':access})
		if mode=='open' or destination!=path: return null
		handle=Sink.new();handle.events=events;handle.mode=mode
		if mode=='initial_error': handle.error=ERR_FILE_CANT_READ
		return handle
class Boundary extends RefCounted:
	var authenticated=true
	var connection_generation=7
	var pending: Dictionary={}
	var calls: Array=[]
	var events: Array=[]
	var allowed=true
	var manifest={'rs_core_room':true}
	var current: Dictionary={}
	var source_hash=''
	var pad_trace_enabled=false
	func request(method,params,association):
		calls.append({'method':method,'params':params.duplicate(true),'association':association.duplicate(true)})
		events.append({'type':'request','method':method})
		if not allowed: return 0
		var id: int=pending.size()+1;pending[id]=association.duplicate(true);return id
	func build(_world,_scene): return allowed
	func contains(_point): return allowed
	func commit(_s,_scene,_hash): events.append({'type':'commit'})
	func suspend_world(): pass
	func stop_all(): pass
	func ui_cue(_kind,_focus): pass
	func reset(_epoch): pass
	func quit(): events.append({'type':'quit'})
	func has_focus(): return true
	func gui_get_focus_owner(): return null
class LabelBoundary extends RefCounted:
	var text=''
class PanelBoundary extends RefCounted:
	var visible=false
	var mode='start'
	var pending=false
	var preferences={}
	var font_scale=1.0
	var status=LabelBoundary.new()
	var effects: Array=[]
	var availability: Array=[]
	var pending_after_availability=false
	func set_save_availability(default_state,slot_state,rows):
		availability=[default_state,slot_state,rows.duplicate(true)]
		if pending_after_availability: pending=true
		effects.append(['availability',availability.duplicate(true)])
	func show_page(page,owner,_snapshot,authenticated,blocked,preserve):
		mode=page;visible=true;pending=blocked
		effects.append(['page',page,owner.duplicate(true),authenticated,blocked,preserve])
	func close_page(): visible=false;effects.append(['close'])
	func set_message(message,blocked=false): status.text=message;pending=blocked;effects.append(['message',message,blocked])
func check(ok: bool,label: String) -> void:
	checks+=1
	if not ok: failures+=1;printerr('FAIL '+label)
func unique_target(text: String,name: String) -> bool:
	return text.count('func '+name+'(')==1
func extract(text: String,name: String) -> String:
	var marker='func '+name+'('
	if not unique_target(text,name):
		check(false,'missing/duplicate extraction '+name)
		return ''
	var start=text.find(marker);var end=text.find('\nfunc ',start+1)
	return text.substr(start) if end<0 else text.substr(start,end-start)
func snapshot_fixture() -> Dictionary:
	var point={'xM':2.0,'yM':0.0,'zM':3.0}
	return {'kind':'full','protocolVersion':3,'schemaVersion':'freeze-v02-interfaces/1.2','worldId':'return_station','sceneId':'rs_core_room','worldEpoch':1,'checkpointId':null,'serverTick':10,'authorityRevision':2,'ackSeq':0,'entryToken':null,'actors':[],'doors':[],'interactables':[],'hazards':[],'objectives':[],'capabilities':{'schemaVersion':1},'progression':{'schemaVersion':1,'eventSeq':5,'currentWorldId':'return_station','worlds':[]},'player':{'entityId':'player','transform':{'positionM':point,'yawRad':0.0},'velocityMps':point,'actionState':'idle','currentHp':80,'maxHp':100,'currentEnergy':100,'maxEnergy':100,'facingX':0.0,'facingZ':1.0,'aimX':0.0,'aimZ':1.0}}
func receipt(command: String,s: Dictionary) -> Dictionary:
	return {'commandId':command,'applied':true,'alreadyApplied':false,'errorCode':null,'snapshot':s,'worldEpoch':s.worldEpoch,'serverTick':s.serverTick,'authorityRevision':s.authorityRevision}
func subject(script: GDScript,mode='ok'):
	var a=script.new()
	for field in ['rpc','catalog','renderer','audio','event_view','hud','tree','window','viewport']:
		var boundary=Boundary.new();boundary.events=a.timeline;a.set(field,boundary)
	a.panels=PanelBoundary.new();a.storage=Storage.new();a.storage.events=a.timeline;a.storage.mode=mode
	a.destination=a.storage.path;a.clock=100;a.generation=4;a.save_projection_generation=4;a.save_projection_connection=7
	a.default_save_state='available';a.slot_list_state='empty';a.snapshot=snapshot_fixture();a.trace_enabled=mode!='off'
	a._trace_sink_start()
	return a
func rows(a) -> Array:
	var result: Array=[]
	for event in a.timeline:
		if event.type=='flush':
			var line: String=event.line
			var prefix='WUXIAN_SAVE_FLOW_TRACE:' if line.begins_with('WUXIAN_SAVE_FLOW_TRACE:') else 'WUXIAN_FRONTEND_TRACE:'
			var row=JSON.parse_string(line.substr(prefix.length()))
			if row is Dictionary: result.append(row)
	return result
func is_save(event: Dictionary) -> bool:
	return event.get('line','').contains('"stage":"snapshot_applied"') and event.get('line','').contains('"method":"formal_save"')
func save_order(events: Array) -> bool:
	var writes: Array=[];var flushes: Array=[];var closes: Array=[]
	for i in range(events.size()):
		var e: Dictionary=events[i]
		if e.type=='write' and is_save(e): writes.append(i)
		if e.type=='flush' and is_save(e): flushes.append(i)
		if e.type=='request' and e.method=='session_close': closes.append(i)
	return writes.size()==1 and flushes.size()==1 and closes.size()==1 and writes[0]>=0 and flushes[0]>=0 and closes[0]>=0 and writes[0]<flushes[0] and flushes[0]<closes[0]
func save_and_close(a) -> void:
	a.journey_started=true;a._save_request('formal_save',{},true)
	var association: Dictionary=a.rpc.pending[1]
	a._response(1,'formal_save',receipt('save',snapshot_fixture()),association)
func business(a) -> Dictionary:
	return {'requests':a.rpc.calls,'pending':a.rpc.pending,'snapshot':a.snapshot,'critical':a.critical,'uncertain':a.uncertain,'close':a.close_pending,'attempted':a.close_attempted,'confirmed':a.close_confirmed,'panels':a.panels.effects}
func check_wire_projection(script: GDScript,method: String,hp: int,sequence: int) -> void:
	var a=subject(script)
	if method=='formal_continue': a._begin_journey(method,{})
	else: a.journey_started=true;a._save_request(method,{},false)
	var association: Dictionary=a.rpc.pending[1]
	var state=snapshot_fixture();state.player.currentHp=hp;state.player.maxHp=maxi(100,hp);state.progression.eventSeq=sequence
	var parsed=JSON.parse_string(JSON.stringify(receipt('continue' if method=='formal_continue' else 'save',state),'',true,true))
	var label=method+' wire hp='+str(hp)+' seq='+str(sequence)
	check(typeof(parsed.snapshot.player.currentHp)==TYPE_FLOAT and typeof(parsed.snapshot.progression.eventSeq)==TYPE_FLOAT,label+' actual JSON wire numbers are floats')
	check(P.receipt(parsed),label+' positive wire receipt prerequisite accepted')
	a._response(1,method,parsed,association)
	var applied=rows(a).filter(func(row):return row.get('stage')=='snapshot_applied' and row.get('method')==method)
	check(applied.size()==1,label+' exactly one applied record reaches flushed sink')
	if applied.size()==1:
		var row: Dictionary=applied[0]
		check(row.request_id==1 and row.generation==association.generation and row.connection_generation==a.rpc.connection_generation,label+' request/generation/connection association')
		check(row.get('hp')==hp,label+' hp retained')
		check(row.get('event_seq')==sequence,label+' event sequence retained')
		check(row.world=='return_station' and row.scene=='rs_core_room' and row.checkpoint_present==false and row.x==2 and row.y==0 and row.z==3 and row.applied==true and row.already_applied==false,label+' bounded projection retained')
		check(a.timeline.filter(func(e):return e.type=='flush' and e.binding=='attempt-A' and e.line.contains('"stage":"snapshot_applied"')).size()==1,label+' applied fact flushed in attempt sink')
func check_wire_upstream_max_rejection(script: GDScript,method: String) -> void:
	var a=subject(script)
	if method=='formal_continue': a._begin_journey(method,{})
	else: a.journey_started=true;a._save_request(method,{},false)
	var association: Dictionary=a.rpc.pending[1]
	var state=snapshot_fixture();state.player.currentHp=4294967295;state.player.maxHp=4294967295;state.progression.eventSeq=9007199254740991
	var parsed=JSON.parse_string(JSON.stringify(receipt('continue' if method=='formal_continue' else 'save',state),'',true,true))
	var label=method+' current upstream rejection at emitter maximum'
	check(typeof(parsed.snapshot.player.currentHp)==TYPE_FLOAT and typeof(parsed.snapshot.progression.eventSeq)==TYPE_FLOAT,label+' parsed float types')
	check(parsed.snapshot.player.currentHp==4294967295 and int(parsed.snapshot.player.currentHp)==4294967295 and parsed.snapshot.progression.eventSeq==9007199254740991 and int(parsed.snapshot.progression.eventSeq)==9007199254740991,label+' parsed numeric values exact')
	check(P.counter(parsed.snapshot.player.currentHp,0,4294967295),label+' explicit HP bound accepted')
	check(not P.counter(parsed.snapshot.progression.eventSeq),label+' observed default counter rejects; cause UNKNOWN')
	check(not P.receipt(parsed),label+' current receipt prerequisite rejects')
	a._response(1,method,parsed,association)
	check(not rows(a).any(func(row):return row.get('stage')=='snapshot_applied'),label+' actual response emits no applied proof')
func check_numeric_emitter(script: GDScript,method: String,hp,sequence,hp_valid: bool,sequence_valid: bool,label: String) -> void:
	var a=subject(script);var before=business(a).duplicate(true)
	var state=snapshot_fixture();state.player.currentHp=hp;state.progression.eventSeq=sequence
	a._trace_save_flow('snapshot_applied',method,17,{'applied':true,'alreadyApplied':false},state)
	var emitted=rows(a)
	check(emitted.size()==1,method+' '+label+' flushed exactly once')
	if emitted.size()==1:
		var row: Dictionary=emitted[0]
		check(row.has('hp')==hp_valid and (not hp_valid or row.hp==int(hp)),method+' '+label+' independent HP validation')
		check(row.has('event_seq')==sequence_valid and (not sequence_valid or row.event_seq==int(sequence)),method+' '+label+' independent event validation')
		check(row.request_id==17 and row.generation==4 and row.connection_generation==7 and row.world=='return_station' and row.scene=='rs_core_room' and row.checkpoint_present==false and row.x==2 and row.y==0 and row.z==3,method+' '+label+' association and other bounded fields unchanged')
	check(business(a)==before,method+' '+label+' passive emitter preserves business state')
func check_wire_equivalence(script: GDScript,method: String) -> void:
	var baseline: Dictionary={}
	for mode in ['off','ok','write','flush']:
		var a=subject(script,mode)
		if method=='formal_continue': a._begin_journey(method,{})
		else: a.journey_started=true;a._save_request(method,{},false)
		var association: Dictionary=a.rpc.pending[1]
		var parsed=JSON.parse_string(JSON.stringify(receipt('continue' if method=='formal_continue' else 'save',snapshot_fixture()),'',true,true))
		a._response(1,method,parsed,association)
		if mode=='off': baseline=business(a).duplicate(true)
		check(business(a)==baseline,method+' JSON wire '+mode+' business equivalence')
		if mode!='ok': check(not rows(a).any(func(row):return row.get('stage')=='snapshot_applied'),method+' JSON wire '+mode+' no fabricated flushed proof')
func _initialize() -> void:
	var text=FileAccess.get_file_as_string('res://scripts/ui/v1_main.gd').replace('\r\n','\n')
	var full: Script=load('res://scripts/ui/v1_main.gd')
	check(full is GDScript and full.can_instantiate(),'FULL production ScriptResource compile without main instantiation')
	var header="extends RefCounted\nconst P=preload('res://scripts/bridge/protocol.gd')\nconst Rpc=preload('res://scripts/bridge/rpc_client.gd')\nconst Content=preload('res://scripts/ui/content_model.gd')\nconst NOTIFICATION_WM_CLOSE_REQUEST=Node.NOTIFICATION_WM_CLOSE_REQUEST\nconst NOTIFICATION_APPLICATION_FOCUS_OUT=Node.NOTIFICATION_APPLICATION_FOCUS_OUT\nconst NOTIFICATION_APPLICATION_FOCUS_IN=Node.NOTIFICATION_APPLICATION_FOCUS_IN\nvar clock=100\nvar timeline: Array=[]\nvar console_lines: Array=[]\nvar tree\nvar window\nvar viewport\nvar storage\nvar destination=''\nvar optin='1'\n"
	for line in text.substr(0,text.find('func _ready()')).split('\n'):
		if line.begins_with('const ACTION_KEYS = ') or line.begins_with('const TRACE_KEYS = '): header+=line+'\n'
		if line.begins_with('var '):
			if line.begins_with('var catalog ') or line.begins_with('var event_view '): line='var '+line.get_slice(' ',1)
			header+=line+'\n'
	var selected=['_context','_owner','_assoc','_current','_request','_begin_journey','_save_request','_response','_accept_snapshot','_submit_close','_close_current','_invalidate_saves','_save_scope_current','_default_continue_available','_slot_available','_writable_slot_params','_publish_saves','_show','_back','_failed','_connection','_hold','_localized','_notification','_exit_tree','_open_paused','_advance_lifecycle','_refresh_saves','_save_read','_set_save_read_state','_save_read_current','_save_read_response','_valid_slots','_event_key','_trace_save_flow','_trace_frontend','_trace_sink_destination','_trace_storage_open','_trace_sink_start','_trace_sink_close','_trace_sink_write','_trace_start','_trace_live','_trace_stop','_trace_write']
	var body=''
	for name in selected: body+=extract(text,name)+'\n'
	if failures>0: print('FRONTEND_LIVE checks='+str(checks)+' failures='+str(failures));quit(1);return
	# Substitute only clock/input/UI/theme/tree and low-level storage/output boundaries.
	body=body.replace('Time.get_ticks_msec()','clock').replace('ThemeModel.apply(','_theme_apply(')
	body=body.replace("OS.get_environment('WUXIAN_GH_FRONTEND_OBSERVATION_PATH')","destination").replace("OS.get_environment('WUXIAN_V1_INPUT_TRACE')","optin")
	body=body.replace('FileAccess.open(path,FileAccess.READ_WRITE)','storage.open(path,FileAccess.READ_WRITE)').replace('print(line)','console_lines.append(line)').replace("print('WUXIAN_INPUT_TRACE:'+encoded)","console_lines.append('WUXIAN_INPUT_TRACE:'+encoded)")
	var boundaries='''
func _release(): timeline.append({'type':'release'})
func _clear_mouse_movement(): pass
func _key_held(_key): return false
func _theme_apply(_control,_world): pass
func _feedback(message): last_feedback=message
func _eligible(): return false
func _prepare_entry(_token): pass
func _interaction_result(_association,_receipt): pass
func _guard_association_valid(_a): return false
func _guard_identity_same(_a,_b): return false
func _end_mouse_guard(): pass
func _close_baizhi(): pass
func get_tree(): return tree
func get_window(): return window
func get_viewport(): return viewport
'''
	var script=GDScript.new();script.source_code=header+boundaries+body
	var error=script.reload();check(error==OK and script.can_instantiate(),'strict actual-method extraction compiles')
	if error!=OK or not script.can_instantiate(): print('FRONTEND_LIVE checks='+str(checks)+' failures='+str(failures));quit(1);return
	check(P.snapshot(snapshot_fixture()),'real protocol snapshot accepted')
	var a=subject(script);save_and_close(a)
	check(save_order(a.timeline),'actual saved fact and its flush precede session_close')
	var saved=rows(a).filter(func(row):return row.get('stage')=='snapshot_applied' and row.get('method')=='formal_save')
	check(saved.size()==1 and saved[0].request_id==1 and saved[0].generation==4 and saved[0].connection_generation==7,'save association projection matches actual request')
	check(a.timeline.filter(func(e):return e.type=='flush' and is_save(e) and e.binding=='attempt-A').size()==1,'flush tied to opaque attempt binding')
	check(not save_order(a.timeline.filter(func(e):return not(e.type=='write' and is_save(e)))),'missing saved fact negative control')
	check(not save_order(a.timeline.filter(func(e):return not(e.type=='flush' and is_save(e)))),'missing save flush negative control')
	var moved=a.timeline.filter(func(e):return not(e.type=='flush' and is_save(e)));moved.append(a.timeline.filter(func(e):return e.type=='flush' and is_save(e))[0])
	check(not save_order(moved),'late save flush negative control')
	var close_assoc: Dictionary=a.rpc.pending[2]
	a._response(2,'session_close',{'closing':true},close_assoc)
	var confirmed=-1;var quit_index=-1
	for i in range(a.timeline.size()):
		if a.timeline[i].type=='flush' and a.timeline[i].line.contains('close_confirmed_quit'): confirmed=i
		if a.timeline[i].type=='quit': quit_index=i
	check(confirmed>=0 and quit_index>=0 and confirmed<quit_index and a.trace_sink==null,'confirmed disposition flushed/closed before quit')
	var baseline: Dictionary={}
	for mode in ['off','ok','open','write','flush','nonempty','initial_error']:
		var b=subject(script,mode);save_and_close(b)
		if mode=='off': baseline=business(b);check(b.timeline.filter(func(e):return e.type=='open').is_empty(),'OFF performs no diagnostic I/O')
		else: check(business(b)==baseline,mode+' preserves requests associations snapshot flags and panel effects')
		if mode in ['open','write','flush','nonempty','initial_error']: check(b.trace_sink==null,mode+' disables only sink')
	for stale in [false,true]:
		var b=subject(script);b.current_slots=[{'slotId':'PRIVATE','displayName':'SECRET'}];b.slot_list_state='list';b.save_projection_generation=3 if stale else 4
		b._publish_saves(false)
		var row=rows(b).back()
		check(row.stage=='computed_publish_saves' and row.default_state==b.panels.availability[0] and row.slot_state==b.panels.availability[1] and row.slot_count==b.panels.availability[2].size() and row.current_scope==not stale,'computed projection equals panel arguments without pixel claim')
		check(not JSON.stringify(row).contains('PRIVATE') and not JSON.stringify(row).contains('SECRET'),'projection excludes slot names/IDs')
	for blocker in ['pending','critical','uncertain','setter_pending','none']:
		var b=subject(script);b.panels.visible=true;b.panels.mode='save';b.panels.pending=blocker=='pending';b.critical='formal_save' if blocker=='critical' else '';b.uncertain=blocker=='uncertain'
		b.panels.pending_after_availability=blocker=='setter_pending'
		b._publish_saves()
		check(rows(b)[0].rebuild_allowed==(blocker=='none'),'actual rebuild_allowed '+blocker)
	for branch in ['blocked','requested','no_save','disconnected']:
		var b=subject(script);b.journey_started=branch=='requested';b.critical='formal_save' if branch=='blocked' else '';b.rpc.authenticated=branch!='disconnected'
		b._notification(Node.NOTIFICATION_WM_CLOSE_REQUEST)
		var expected={'blocked':'notification_blocked','requested':'confirmation_requested','no_save':'notification_no_confirmed_save','disconnected':'client_quit_disconnected'}[branch]
		check(rows(b).any(func(row):return row.get('reason')==expected),'actual notification disposition '+branch)
	for branch in ['stale','invalid','failed','unknown','disconnected']:
		var b=subject(script);b.journey_started=true;b._submit_close();var assoc: Dictionary=b.rpc.pending[1].duplicate(true)
		if branch=='stale': assoc.generation=99;b._response(1,'session_close',{'closing':true},assoc)
		elif branch=='invalid': b._response(1,'session_close',{'closing':'bad'},assoc)
		elif branch in ['failed','unknown']: b._failed(1,'session_close','UNKNOWN_MUTATION' if branch=='unknown' else 'PRIVATE_ERROR',assoc)
		else: b.rpc.authenticated=false;b._connection(false,'PRIVATE_ERROR')
		check(not b.close_confirmed and not rows(b).any(func(row):return row.get('reason')=='close_confirmed_quit'),branch+' never claims close confirmation')
		check(not JSON.stringify(rows(b)).contains('PRIVATE_ERROR'),branch+' omits raw errors')
	var cap=subject(script)
	for n in range(33): cap._trace_frontend('close_disposition',{'reason':'submit_blocked'})
	check(cap.trace_sink_records==32 and cap.trace_records==32 and rows(cap).size()==32,'sink32/shared counter each charged once')
	var mirror=subject(script);mirror._trace_save_flow('request_queued','formal_continue',1)
	check(mirror.trace_records==1 and mirror.trace_save_records==1 and mirror.trace_sink_records==1 and mirror.console_lines.size()==1,'SaveFlow mirror no double debit/format')
	check(mirror.timeline.filter(func(e):return e.type=='flush')[0].line==mirror.console_lines[0]+'\n','console and mirrored accepted line identical')
	for instant in [119999,120000]:
		var b=subject(script);b.clock=instant;b._trace_frontend('close_disposition',{'reason':'submit_blocked'})
		check(rows(b).size()==(1 if instant==119999 else 0),'original elapsed boundary '+str(instant))
	var shared=subject(script);shared.trace_records=222;shared._trace_frontend('close_disposition',{'reason':'submit_blocked'});shared._trace_save_flow('request_queued','formal_continue',1)
	check(shared.trace_records==223 and shared.trace_sink_records==1,'remaining original shared capacity not refreshed')
	var bytecap=subject(script);bytecap._trace_sink_write('é'.repeat(32767)+'a')
	check(bytecap.trace_sink_bytes==65536 and bytecap.trace_sink_records==1,'UTF8 byte boundary includes newline')
	bytecap._trace_sink_write('x');check(bytecap.trace_sink==null and bytecap.trace_sink_records==1,'over byte limit does not flush')
	for path in ['', 'relative/frontend-live-observation.jsonl','C:/fixture/attempt-A/other.jsonl','C:/fixture/../frontend-live-observation.jsonl','C:/fixture/wrong/frontend-live-observation.jsonl','user://frontend-live-observation.jsonl','res://frontend-live-observation.jsonl']:
		var b=subject(script,'off');b.trace_enabled=true;b.trace_sink_attempted=false;b.destination=path;b._trace_sink_start()
		check(b.trace_sink==null,'wrong/missing binding has no fallback '+path)
	var private_row=subject(script);private_row._trace_frontend('close_disposition',{'reason':'PRIVATE','path':'PRIVATE'});private_row._trace_frontend('computed_publish_saves',{'current_scope':'bad'})
	check(rows(private_row).is_empty(),'privacy/type whitelist rejects arbitrary facts')
	var close_exit=subject(script);close_exit._exit_tree();check(close_exit.trace_sink==null,'exit closes retained sink')
	var stopped=subject(script);stopped._trace_stop('duration',120000);check(stopped.trace_sink==null and not stopped.trace_enabled,'trace stop closes sink')
	var continued_states: Array=[]
	for mode in ['off','ok','write','flush']:
		var b=subject(script,mode);b._begin_journey('formal_continue',{})
		var assoc: Dictionary=b.rpc.pending[1]
		b._response(1,'formal_continue',receipt('continue',snapshot_fixture()),assoc)
		continued_states.append(business(b))
		if mode=='ok':
			var applied=rows(b).filter(func(row):return row.get('method')=='formal_continue' and row.get('stage')=='snapshot_applied')
			check(applied.size()==1 and applied[0].request_id==1 and applied[0].generation==assoc.generation and applied[0].connection_generation==7 and applied[0].scene=='rs_core_room','actual fresh Continue response reaches bound live sink after application')
		else: check(not rows(b).any(func(row):return row.get('stage')=='snapshot_applied'),mode+' does not fabricate live Continue evidence')
	check(continued_states.all(func(state):return state==continued_states[0]),'Continue OFF/ON/failed-I/O business equivalence')
	for bad in ['stale','malformed','rejected','apply_failed']:
		var b=subject(script);b._begin_journey('formal_continue',{});var assoc: Dictionary=b.rpc.pending[1].duplicate(true);var reply=receipt('continue',snapshot_fixture())
		if bad=='stale': assoc.generation-=1
		if bad=='malformed': reply.applied='bad'
		if bad=='rejected': reply.applied=false;reply.errorCode='E_REJECTED'
		if bad=='apply_failed': b.catalog.allowed=false
		b._response(1,'formal_continue',reply,assoc)
		check(not rows(b).any(func(row):return row.get('stage')=='snapshot_applied'),bad+' Continue does not emit applied proof')
	var back=subject(script);back.panels.mode='exit_confirm';back.panels.visible=true;back._back()
	check(not back.close_attempted and not rows(back).any(func(row):return row.get('reason')=='close_confirmed_quit' or row.get('stage')=='snapshot_applied'),'back/cancel is no close/resume/save success proof')
	var near=subject(script);near.trace_sink_records=31;near._trace_frontend('close_disposition',{'reason':'submit_blocked'});near._trace_frontend('close_disposition',{'reason':'submit_blocked'})
	check(near.trace_sink_records==32 and rows(near).size()==1,'adjacent-to32 record admission rejects extra flush')
	var bytes=subject(script);bytes.trace_sink_bytes=65535;bytes._trace_frontend('close_disposition',{'reason':'submit_blocked'})
	check(bytes.trace_records==0 and rows(bytes).is_empty(),'oversized serialized projection neither charges nor flushes')
	var epoch=subject(script);epoch.trace_sink_records=31;epoch.trace_started_ms=10;epoch.clock=100;epoch._begin_journey('formal_continue',{});var old_count=epoch.trace_sink_records;epoch.clock=120010;epoch._publish_saves(false)
	check(epoch.trace_started_ms==10 and epoch.trace_sink_records==old_count,'journey/publication/clock do not renew sink budget or original window')
	var privacy=subject(script);var state=snapshot_fixture();state.worldId='PRIVATE';state.sceneId='PRIVATE';state.checkpointId='PRIVATE';state.player.currentHp=NAN;state.player.transform.positionM={'xM':NAN,'yM':'PRIVATE','zM':2};state.progression.eventSeq=INF
	privacy._trace_save_flow('snapshot_applied','formal_continue',1,{'applied':true,'alreadyApplied':false,'secret':'PRIVATE'},state)
	var allowed=['stage','method','request_id','elapsed_ms','generation','connection_generation','applied','already_applied','world','scene','checkpoint_present','hp','x','y','z','event_seq']
	check(rows(privacy).size()==1 and rows(privacy)[0].keys().all(func(key):return key in allowed) and not JSON.stringify(rows(privacy)).contains('PRIVATE'),'mirrored SaveFlow exact key filter, no raw values or nonfinite projection')
	for mode in ['write','flush','initial_error','nonempty']:
		var b=subject(script,mode);b._trace_frontend('close_disposition',{'reason':'submit_blocked'});var opens=b.timeline.filter(func(e):return e.type=='open').size();b._trace_sink_start()
		check(rows(b).is_empty() and b.timeline.filter(func(e):return e.type=='open').size()==opens,mode+' cannot flush rejected data or reopen/adopt/fallback')
	var offstart=subject(script,'off');offstart.optin='0';offstart._trace_start();check(offstart.timeline.is_empty(),'actual trace-start OFF has no diagnostic I/O')
	var start=subject(script,'off');start.optin='1';start._trace_start();check(start.trace_started_ms==100 and start.trace_sink!=null and start.timeline.filter(func(e):return e.type=='open').size()==1,'actual trace-start initializes sink once after opt-in/time')
	for method in ['formal_save','formal_continue']:
		for pair in [[0,0],[80,5],[4294967295,5]]: check_wire_projection(script,method,pair[0],pair[1])
		check_wire_upstream_max_rejection(script,method)
		for pair in [[0,0],[80,5],[4294967295,9007199254740991],[0.0,0.0],[80.0,5.0],[4294967295.0,9007199254740991.0]]:
			check_numeric_emitter(script,method,pair[0],pair[1],true,true,'valid '+str(pair))
		for invalid in [-1,0.5,4294967296,null,true,'80',NAN,INF,-INF]:
			check_numeric_emitter(script,method,invalid,5,false,true,'invalid HP '+str(invalid))
		for invalid in [-1,0.5,9007199254740992,null,true,'5',NAN,INF,-INF]:
			check_numeric_emitter(script,method,80,invalid,true,false,'invalid event '+str(invalid))
		check_wire_equivalence(script,method)
	check(not unique_target('func missing(','target') and not unique_target('func target(\nfunc target(','target') and unique_target('func target(','target'),'actual extraction admission rejects missing/duplicate, not expected I/O failure')
	print('FRONTEND_LIVE checks='+str(checks)+' failures='+str(failures))
	quit(1 if failures else 0)
