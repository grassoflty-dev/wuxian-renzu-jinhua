extends SceneTree
# Focused real-edge preservation regression; no native or physical movement proof.
var checks=0
var failures=0
class Transport extends RefCounted:
	var authenticated=true
	var reject_inputs=0
	var sequence=0
	var connection_generation=1
	var calls=[]
	func request(method,params,_association):
		if method=='formal_submit_input' and reject_inputs>0:
			reject_inputs-=1;return 0
		sequence+=1
		if method=='formal_submit_input': calls.append(params.duplicate(true))
		return sequence
class Peripheral extends RefCounted:
	var visible=false
	var current={}
	var events=[]
	var sound=[]
	var cursor=0
	var sound_cursor=0
	var failed=false
	var status_text=''
	func clear_interaction_focus(): pass
	func suspend_world(): pass
	func _cancel_guard_capture(): pass
	func update(_a,_b,_c,_d,_e): pass
	func project(_a,_b,_c): return []
	func set_interaction_focus(_a,_b,_c,_d): pass
	func _clear_movement_captures(): pass
	func set_input_as_handled(): pass
	func update_view(_a,_b,_c,_d,_e,_f,_g,_h,_i,_j,_k,_l): pass
func check(ok,label):
	checks+=1
	if not ok: failures+=1;printerr('FAIL '+label)
func extract(text,name):
	var marker='func '+name+'('
	check(text.count(marker)==1,'unique RS001 method '+name)
	var start=text.find(marker)
	var end=text.find('\nfunc ',start+1)
	return text.substr(start) if end<0 else text.substr(start,end-start)
func edge(a,pressed,time,key=KEY_D,echo=false,physical=false):
	a.clock=time
	var event=InputEventKey.new()
	event.keycode=key;event.physical_keycode=key if physical else 0;event.pressed=pressed;event.echo=echo
	a._input(event)
	a.rows.append({'stage':'after_edge','time':time,'pressed':pressed,'logical_down':a.logical_down.keys(),'axes':[a._movement_axes().x,a._movement_axes().y],'eligible':a._eligible()})
func tick(a,time):
	a.clock=time;a._process(0.05)
func subject(script):
	var a=script.new()
	a.rpc=Transport.new()
	for field in ['panels','catalog','renderer','audio','event_view','hud','viewport']: a.set(field,Peripheral.new())
	a.snapshot={'worldId':'return_station','sceneId':'rs_core_room','worldEpoch':1,'entryToken':null,'player':{'currentHp':100}}
	a.status={'worldId':'return_station','sceneId':'rs_core_room','worldEpoch':1,'paused':false,'entryToken':null}
	a.journey_started=true;a.focused=true;a.last_snapshot_at=1000;a.last_status_at=1000;a.last_input_packet_at=1000
	return a
func _initialize():
	var production=load('res://scripts/ui/v1_main.gd')
	check(production is GDScript and production.can_instantiate(),'full production ScriptResource compiles without startup')
	var text=FileAccess.get_file_as_string('res://scripts/ui/v1_main.gd').replace('\r\n','\n')
	var header="extends RefCounted\nconst P=preload('res://scripts/bridge/protocol.gd')\nvar clock=1000\nvar viewport\nvar rows=[]\n"
	for line in text.substr(0,text.find('func _ready()')).split('\n'):
		if line.begins_with('const ACTION_KEYS = ') or line.begins_with('const TRACE_KEYS = '): header+=line+'\n'
		if line.begins_with('var '):
			if line.begins_with('var catalog ') or line.begins_with('var event_view '): line='var '+line.get_slice(' ',1)
			header+=line+'\n'
	for line in text.split('\n'):
		if line.begins_with('const MOVEMENT_AXES = '): header+=line+'\n'
	var body=''
	for name in ['_input','_event_key','_key_held','_keyboard_move_state','_movement_axes','_process','_sample','_eligible','_request','_assoc','_context','_owner','_clear_mouse_movement','_release','_end_guard','_clear_mouse_guard']:
		body+=extract(text,name)+'\n'
	body=body.replace('Time.get_ticks_msec()','clock').replace('Input.is_physical_key_pressed(key)','false').replace('Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)','false')
	var boundaries="""
func _trace_key(event,stage,reason):
	if event is InputEventKey: rows.append({'stage':stage,'reason':reason,'time':clock,'key':event.keycode,'physical':event.physical_keycode,'pressed':event.pressed,'logical_down':logical_down.keys()})
func _trace_pad_sample(reason,neutral,axes=Vector2i.ZERO,accepted=false):
	rows.append({'stage':'sampler','time':clock,'reason':reason,'neutral':neutral,'logical_down':logical_down.keys(),'axes':[axes.x,axes.y],'accepted':accepted})
func _trace_pad(_a,_b,_c,_d={}): pass
func _trace_tick(): pass
func _trace_save_flow(_a,_b,_c): pass
func _observe_mouse_movement(): pass
func _observe_mouse_guard(): pass
func _mouse_movement_valid(): return false
func _mouse_guard_valid(): return false
func _current_hud_interaction(): return {}
func _current_hud_traversal(): return {}
func _current_hud_actions(): return []
func _current_hud_guard(): return {}
func _current_hud_movements(): return []
func _nearest(): return {}
func _interaction_prompt(_a): assert(false);return ''
func _end_mouse_guard(): assert(false)
func _escape(): assert(false)
func _send_action(_a,_b): assert(false)
func _action(_a,_b=false): assert(false)
func _feedback(_a): assert(false)
func _hold(_a): assert(false)
func get_viewport(): return viewport
"""
	var script=GDScript.new();script.source_code=header+boundaries+body
	var error=script.reload()
	check(error==OK and script.can_instantiate(),'actual methods compile')
	if error!=OK or failures: finish();return
	for between in [true,false]:
		var a=subject(script)
		check(a._eligible(),'eligible station context')
		edge(a,true,1010)
		check(a.logical_down.has(KEY_D) and a._movement_axes()==Vector2i(1,0),'logical D selects movement')
		if between: edge(a,false,1011)
		# Same explicit real sampler gate probe at 49 ms; no packet may escape.
		a.clock=1049
		check(not a._sample(false) and a.rpc.calls.is_empty(),'actual 50ms gate rejects 49ms')
		tick(a,1050)
		check(a.rpc.calls.size()==1,'process sampling cadence submits first packet at 50ms')
		if not between: edge(a,false,1051)
		tick(a,1100)
		check(a.rpc.calls.size()==2,'next process tick submits neutral packet')
		var first=a.rpc.calls[0].sample;var second=a.rpc.calls[1].sample
		check(first.clientTimeMs==1050 and second.clientTimeMs==1100,'actual sampler clock and 50ms spacing')
		check(first.worldEpoch==1 and second.worldEpoch==1 and first.seq==1 and second.seq==2,'current epoch and consecutive sequences')
		check(second.moveX==0.0 and second.moveZ==0.0,'released D yields neutral second sample')
		if between:
			check(first.moveX>0.0 and first.moveZ<0.0,'completed real edge survives until one submitted sample')
		else:
			check(first.moveX>0.0 and first.moveZ<0.0,'D spanning tick produces nonzero isometric movement')
		check(a.logical_down.is_empty(),'release removes logical D')
		print(JSON.stringify({'case':'down_up_before_tick' if between else 'down_tick_up_tick','ordered':a.rows,'submitted_formal_submit_input':a.rpc.calls}))

	# Duplicate logical and physical deliveries must not become another edge.
	for physical in [false,true]:
		var a=subject(script)
		edge(a,true,1010,KEY_D,false,physical)
		edge(a,true,1011,KEY_D,true,physical)
		edge(a,true,1012,KEY_D,false,physical)
		check(a.pending_movement.size()==4,'echo and duplicate retain only one bounded edge')
		edge(a,false,1013,KEY_D,false,physical)
		tick(a,1050);tick(a,1100)
		check(a.rpc.calls.size()==2 and a.rpc.calls[0].sample.moveX>0 and a.rpc.calls[1].sample.moveX==0,'duplicate deliveries produce one sample only')
	var newest=subject(script)
	edge(newest,true,1010);edge(newest,false,1011)
	edge(newest,true,1020,KEY_A);edge(newest,false,1021,KEY_A)
	tick(newest,1050);tick(newest,1100)
	check(newest.rpc.calls[0].sample.moveX<0 and newest.rpc.calls[0].sample.moveZ>0 and newest.rpc.calls[1].sample.moveX==0,'newest direction replaces older direction without backlog')
	var failed=subject(script)
	var held=subject(script)
	edge(held,true,1010);edge(held,false,1011)
	edge(held,true,1020,KEY_A)
	tick(held,1050)
	check(held.rpc.calls[0].sample.moveX<0 and held.pending_movement.is_empty(),'current held keyboard wins and consumes pending direction')
	edge(held,false,1051,KEY_A);tick(held,1100)
	check(held.rpc.calls[1].sample.moveX==0 and held.rpc.calls[1].sample.moveZ==0,'held sample leaves no extra post-release movement')
	edge(failed,true,1010);edge(failed,false,1011);failed.rpc.reject_inputs=1
	tick(failed,1050)
	check(not failed.pending_movement.is_empty() and failed.rpc.calls.is_empty(),'failed submission retains real edge')
	failed.clock=1051;failed._process(0.001)
	check(failed.rpc.calls.is_empty(),'no immediate retry or cadence bypass after failure')
	failed.clock=1100;failed._process(0.049)
	check(failed.rpc.calls.size()==1 and failed.pending_movement.is_empty() and failed.rpc.calls[0].sample.moveX>0,'next legitimate tick consumes edge once')
	tick(failed,1150)
	check(failed.rpc.calls.size()==2 and failed.rpc.calls[1].sample.moveX==0,'successful retry followed by neutral')
	for invalidation in ['pause','focus','modal','death','close','disconnect','generation','connection','scene','release','mouse']:
		var a=subject(script)
		edge(a,true,1010);edge(a,false,1011)
		match invalidation:
			'pause': a.status.paused=true
			'focus': a.focused=false
			'modal': a.panels.visible=true
			'death': a.snapshot.player.currentHp=0
			'close': a.close_attempted=true
			'disconnect': a.rpc.authenticated=false
			'generation': a.generation+=1
			'connection': a.rpc.connection_generation+=1
			'scene': a.snapshot.sceneId='gh_entry_maintenance';a.status.sceneId='gh_entry_maintenance'
			'release': a._release()
			'mouse': a.mouse_move_owner={'entityId':'move_right'}
		tick(a,1050)
		check(a.pending_movement.is_empty(),'pending edge invalidated: '+invalidation)
		a.status.paused=false;a.focused=true;a.panels.visible=false;a.snapshot.player.currentHp=100;a.close_attempted=false;a.rpc.authenticated=true;a.mouse_move_owner.clear()
		tick(a,1100)
		check(not a.rpc.calls.is_empty() and a.rpc.calls[-1].sample.moveX==0 and a.rpc.calls[-1].sample.moveZ==0,'no deferred movement after resumption: '+invalidation)
	for invalidation in ['pause','focus','modal','death','disconnect']:
		var a=subject(script)
		match invalidation:
			'pause': a.status.paused=true
			'focus': a.focused=false
			'modal': a.panels.visible=true
			'death': a.snapshot.player.currentHp=0
			'disconnect': a.rpc.authenticated=false
		edge(a,true,1010);edge(a,false,1011)
		check(a.pending_movement.is_empty(),'ineligible press never deferred: '+invalidation)
	finish()
func finish():
	print('SHORT_KEY_SAMPLING checks='+str(checks)+' failures='+str(failures))
	quit(1 if failures else 0)
