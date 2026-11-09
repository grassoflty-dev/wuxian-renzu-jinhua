extends SceneTree

var failures: int = 0
var checks: int = 0
func check(ok: bool, label: String) -> void:
	checks += 1
	if not ok:
		failures += 1
		printerr('FAIL '+label)
func function_text(source: String, name: String) -> String:
	var start: int = source.find('func '+name+'(')
	assert(start>=0, name)
	var end: int = source.find('\nfunc ',start+1)
	return source.substr(start) if end<0 else source.substr(start,end-start)
func constants(source: String, names: Array) -> String:
	var result: String = ''
	for line in source.split('\n'):
		for name in names:
			if line.begins_with('const '+name+' = '): result += line+'\n'
	return result
func compile_functions(header: String, source: String, names: Array) -> GDScript:
	var script: GDScript = GDScript.new()
	script.source_code = header
	for name in names: script.source_code += function_text(source,name)+'\n'
	# Substitute only native input, clock and pointer/visibility observation boundaries.
	script.source_code = script.source_code.replace('Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)','_left_held()').replace('Time.get_ticks_msec()','clock')
	script.source_code = script.source_code.replace('button.is_visible_in_tree()',"(button.visible and button.get_meta('fixture_visible'))").replace('button.get_local_mouse_position()',"button.get_meta('fixture_pointer')")
	script.source_code = script.source_code.replace("print('WUXIAN_PAD_TRACE:'+JSON.stringify(row))",'trace_rows.append(row.duplicate(true))')
	if script.reload()!=OK: return null
	return script
func branch(source: String, start: String, end: String) -> String:
	var from: int = source.find(start)+start.length()
	var until: int = source.find(end,from)
	assert(from>=start.length() and until>from)
	return ('\n'+source.substr(from,until-from)).replace('\n\t\t','\n').trim_prefix('\n').strip_edges(false,true)+'\n'
func _initialize() -> void:
	# Full production scripts compile with their real res:// dependencies. No new()/main/_ready.
	var main_script: Script = load('res://scripts/ui/v1_main.gd')
	var hud_script: Script = load('res://scripts/ui/game_hud.gd')
	check(main_script is GDScript and main_script.can_instantiate(),'FULL v1_main compile')
	check(hud_script is GDScript and hud_script.can_instantiate(),'FULL game_hud compile')
	if failures:
		quit(1)
		return
	var main: String = FileAccess.get_file_as_string('res://scripts/ui/v1_main.gd').replace('\r\n','\n')
	var hud_source: String = FileAccess.get_file_as_string('res://scripts/ui/game_hud.gd').replace('\r\n','\n')
	var protocol = load('res://scripts/bridge/protocol.gd')
	var point: Dictionary = {'xM':0.0,'yM':0.0,'zM':0.0}
	var snapshot: Dictionary = {'kind':'full','protocolVersion':3,'schemaVersion':'freeze-v02-interfaces/1.2','worldId':'return_station','sceneId':'rs_core_room','worldEpoch':1,'checkpointId':null,'serverTick':1,'authorityRevision':1,'ackSeq':0,'actors':[],'doors':[],'interactables':[],'hazards':[],'objectives':[],'capabilities':{'schemaVersion':1},'progression':{'schemaVersion':1,'eventSeq':0,'currentWorldId':'return_station','worlds':[]},'player':{'entityId':'player','transform':{'positionM':point,'yawRad':0.0},'velocityMps':point,'actionState':'idle','currentHp':100,'maxHp':100,'currentEnergy':100,'maxEnergy':100,'facingX':0.0,'facingZ':1.0,'aimX':0.0,'aimZ':1.0}}
	check(protocol.snapshot(snapshot),'real protocol snapshot fixture')
	var hud_header: String = '''extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
signal intent(kind, value)
var movement_pad_enabled = false
var movement_states = {}
var button_nodes = []
var movement_toggle_owner = {}
var movement_toggle = CheckBox.new()
var movement_pad = GridContainer.new()
var movement_help = Label.new()
var left = true
func _left_held(): return left
func _pad_trace(_stage, _id, _reason, _facts = {}): pass
'''+constants(hud_source,['MOVEMENT_LABELS','MOVEMENT_ARROWS'])
	var hs: GDScript = compile_functions(hud_header,hud_source,['_interaction_association_valid','_interaction_identity_same','_movement_association_valid','_movement_capture_valid','_cancel_movement_capture','_clear_movement_captures','_add_movement_button','_update_movements'])
	check(hs!=null,'actual HUD functions compile')
	if hs==null: quit(1); return
	var hud = hs.new()
	for id in hud.MOVEMENT_LABELS:
		hud._add_movement_button(hud.movement_pad,id)
		var button: Button = hud.movement_states[id].button
		button.size = Vector2(80,40)
		button.set_meta('fixture_visible',true)
		button.set_meta('fixture_pointer',Vector2(10,10))
	var header: String = '''extends Node
const P = preload('res://scripts/bridge/protocol.gd')
var movement_pad_enabled = false
var mouse_move_owner = {}
var mouse_move_rearm = false
var mouse_guard_owner = {}
var generation = 7
var panel_generation = 0
var neutral_pending = false
var hud
var snapshot = {}
var keys = {}
var eligible = true
var left = true
var clock = 1000
var close_attempted = false
var ready_pending = {}
var status = {}
var last_input_packet_at = -1000
var input_sequence = 0
var pointer_seen = false
var renderer
var requests = []
var rpc = {'authenticated':true}
var logical_down = {}
var blocked_keys = {}
var uncertain = false
var focused = true
var mouse_guard_rearm = false
var trace_enabled = false
var trace_started_ms = 0
var trace_records = 0
var trace_pad_records = 0
var trace_pad_samples = 0
var trace_rows = []
const ACTION_KEYS = {}
var audio
var guard_active = false
var guard_blocked = false
var pending_actions = []
var panels = {'visible':false}
func _context(): return {'worldId':snapshot.worldId,'sceneId':snapshot.sceneId,'worldEpoch':snapshot.worldEpoch}
func _owner(): return {'generation':generation,'panelGeneration':panel_generation,'context':_context()}
func _left_held(): return left
func _key_held(key): return keys.has(key) or logical_down.has(key)
func _eligible(): return eligible
func _hold(_reason): pass
func _trace_key(_event, _stage, _tag): pass
func _end_mouse_guard(): pass
func _end_guard(): pass
func _escape(): pass
func _request(_method, value, _kind, _association): requests.append(value); return 1
'''+constants(main,['MOVEMENT_LABELS','MOVEMENT_AXES','TRACE_KEYS'])
	var methods: Array = ['_keyboard_move_state','_movement_axes','_movement_capture_admitted','_movement_association_valid','_movement_identity_same','_current_hud_movements','_mouse_movement_valid','_clear_mouse_movement','_observe_mouse_movement','_sample','_event_key','_input','_trace_pad','_trace_pad_sample']
	header += 'func admit(value):\n'+branch(main,"\t\t'movement_current_begin':\n","\t\t'guard_current_begin':")
	header += 'func toggle(value, owner):\n'+branch(main,"\t\t'movement_pad_toggle':\n","\t\t'movement_current_begin':")
	# Execute the exact transition neutralization block before any catalog/UI work.
	var accept: String = function_text(main,'_accept_snapshot')
	var transition_start: int = accept.find('\t\t_clear_mouse_movement()')
	var transition_end: int = accept.find('\t\tif not catalog.build',transition_start)
	check(transition_start>=0 and transition_end>transition_start,'scene cancel before neutral reset')
	header += 'func scene_transition():\n'+('\n'+accept.substr(transition_start,transition_end-transition_start)).replace('\n\t\t','\n\t').trim_prefix('\n')
	var ms: GDScript = compile_functions(header,main,methods)
	check(ms!=null,'actual movement selection/cancellation/sample compile')
	if ms==null:
		hud.movement_pad.free(); hud.movement_toggle.free(); hud.movement_help.free()
		quit(1)
		return
	var subject = ms.new()
	subject.hud = hud
	subject.snapshot = snapshot
	# HUD signal routing is a UI boundary; only exact owner cancellation is forwarded.
	hud.intent.connect(func(kind, value):
		if kind=='movement_current_end' and subject._movement_identity_same(value,subject.mouse_move_owner): subject._clear_mouse_movement())
	check(subject._current_hud_movements().is_empty(),'default disabled projection')
	subject.movement_pad_enabled = true
	var rows: Array = subject._current_hud_movements()
	check(rows.size()==8,'complete eight directions')
	var expected_axes: Dictionary = {'move_up_left':Vector2i(-1,-1),'move_up':Vector2i(0,-1),'move_up_right':Vector2i(1,-1),'move_left':Vector2i(-1,0),'move_right':Vector2i(1,0),'move_down_left':Vector2i(-1,1),'move_down':Vector2i(0,1),'move_down_right':Vector2i(1,1)}
	var expected_world: Dictionary = {'move_up_left':Vector2(-1.5,-0.5),'move_up':Vector2(-1,-1),'move_up_right':Vector2(-0.5,-1.5),'move_left':Vector2(-0.5,0.5),'move_right':Vector2(0.5,-0.5),'move_down_left':Vector2(0.5,1.5),'move_down':Vector2(1,1),'move_down_right':Vector2(1.5,0.5)}
	var invalid_rows: Array = rows.duplicate(true)
	invalid_rows[1] = invalid_rows[0].duplicate(true)
	hud._update_movements(snapshot,true,invalid_rows,true,subject._owner())
	check(hud.movement_states.move_up.display.is_empty(),'duplicate complete-set projection rejected')
	hud._update_movements(snapshot,true,rows,true,subject._owner())
	for row in rows:
		var state: Dictionary = hud.movement_states[row.entityId]
		state.capture = row.duplicate(true)
		check(subject._movement_capture_admitted(row),'admission '+row.entityId)
		subject.admit(row)
		check(not subject.mouse_move_owner.is_empty(),'single owner '+row.entityId)
		check(not subject._movement_capture_admitted(row),'duplicate rejected '+row.entityId)
		var axes: Vector2i = subject._movement_axes()
		check(axes==expected_axes[row.entityId],'axes '+row.entityId)
		subject.clock += 51
		subject._sample(false)
		var sample: Dictionary = subject.requests.back().sample
		var expected: Vector2 = expected_world[row.entityId].normalized()
		check(Vector2(sample.moveX,sample.moveZ).is_equal_approx(expected) and is_equal_approx(expected.length(),1.0),'normalized sample '+row.entityId)
		subject.left = false; hud.left = false
		subject._observe_mouse_movement()
		check(subject.mouse_move_owner.is_empty() and subject.neutral_pending,'release cancellation '+row.entityId)
		subject.left = true; hud.left = true
	var row: Dictionary = rows[0]
	var state: Dictionary = hud.movement_states[row.entityId]
	state.capture = row.duplicate(true)
	subject.admit(row)
	subject.neutral_pending = false
	var key_event: InputEventKey = InputEventKey.new()
	key_event.physical_keycode = KEY_F1; key_event.keycode = KEY_D; key_event.pressed = true
	subject._input(key_event)
	check(subject.mouse_move_owner.is_empty() and subject.mouse_move_rearm and not subject.neutral_pending and subject._movement_axes()==Vector2i(1,0),'actual keydown immediate takeover preserves keyboard')
	key_event.pressed = false; subject._input(key_event)
	check(not subject._keyboard_move_state().held,'actual takeover key release clears logical state')
	subject.left = false; hud.left = false; subject._observe_mouse_movement()
	subject.left = true; hud.left = true
	state.capture = row.duplicate(true); subject.admit(row)
	subject.keys = {KEY_A:true,KEY_D:true}
	check(subject._keyboard_move_state().held and subject._movement_axes()==Vector2i.ZERO,'opposite keyboard keys win at zero vector')
	subject._observe_mouse_movement()
	check(subject.mouse_move_owner.is_empty() and not subject.neutral_pending and subject.mouse_move_rearm,'keyboard takeover without keyboard neutralization')
	subject.keys.clear()
	state.capture = row.duplicate(true)
	check(not subject._movement_capture_admitted(row),'no automatic resume while held')
	subject.left = false; hud.left = false; subject._observe_mouse_movement()
	subject.left = true; hud.left = true
	state.capture = row.duplicate(true); subject.admit(row)
	state.button.set_meta('fixture_pointer',Vector2(90,10))
	subject._observe_mouse_movement()
	check(subject.mouse_move_owner.is_empty() and subject.mouse_move_rearm,'pointer leave cancels')
	state.button.set_meta('fixture_pointer',Vector2(10,10)); state.capture = row.duplicate(true)
	check(not subject._movement_capture_admitted(row),'pointer reentry cannot restart held capture')
	subject.left = false; hud.left = false; subject._observe_mouse_movement()
	state.capture = row.duplicate(true)
	check(not subject._movement_capture_admitted(row),'native held button required')
	subject.left = true; hud.left = true; state.capture = row.duplicate(true)
	state.button.disabled = true
	check(not subject._movement_capture_admitted(row),'disabled arrow rejected')
	state.button.disabled = false; state.button.set_meta('fixture_visible',false)
	check(not subject._movement_capture_admitted(row),'hidden arrow rejected')
	state.button.set_meta('fixture_visible',true)
	subject.mouse_guard_owner = {'held':true}
	check(not subject._movement_capture_admitted(row),'mouse guard excludes movement')
	subject.mouse_guard_owner.clear()
	var stale: Dictionary = row.duplicate(true); stale.generation -= 1
	check(not subject._movement_capture_admitted(stale),'stale generation rejected')
	stale = row.duplicate(true); stale.context.worldEpoch += 1
	check(not subject._movement_capture_admitted(stale),'stale epoch rejected')
	subject.admit(row)
	var old_owner: Dictionary = subject._owner(); old_owner.panelGeneration -= 1
	subject.toggle(false,old_owner)
	check(subject.movement_pad_enabled,'stale toggle owner rejected')
	subject.toggle(false,subject._owner())
	check(not subject.movement_pad_enabled and subject.mouse_move_owner.is_empty(),'toggle off cancels')
	state.capture = row.duplicate(true)
	check(not subject._movement_capture_admitted(row) and not subject._mouse_movement_valid(),'disabled admission and validity')
	hud._update_movements(snapshot,true,rows,false,subject._owner())
	check(not hud.movement_pad.visible and not hud._movement_capture_valid(row),'disabled HUD projection/capture')
	subject.movement_pad_enabled = true
	for reason in ['pause','focus','death','close']:
		subject.left = false; hud.left = false; subject._observe_mouse_movement()
		subject.left = true; hud.left = true; subject.eligible = true
		hud._update_movements(snapshot,true,subject._current_hud_movements(),true,subject._owner())
		state.capture = row.duplicate(true); subject.admit(row)
		subject.eligible = false; subject._observe_mouse_movement()
		check(subject.mouse_move_owner.is_empty() and subject.mouse_move_rearm,'eligibility loss cancels '+reason)
		subject.eligible = true
	subject.left = false; hud.left = false; subject._observe_mouse_movement()
	subject.left = true; hud.left = true
	hud._update_movements(snapshot,true,subject._current_hud_movements(),true,subject._owner())
	state.capture = row.duplicate(true); subject.admit(row)
	var before: int = subject.requests.size()
	subject.scene_transition()
	check(subject.mouse_move_owner.is_empty() and not subject.neutral_pending and subject.requests.size()==before,'scene cancel sends zero stale-epoch requests')
	subject.left = false; hud.left = false; subject._observe_mouse_movement()
	subject.neutral_pending = false
	subject._observe_mouse_movement()
	check(not subject.neutral_pending,'empty observer never queues neutral')
	subject.clock = subject.last_input_packet_at+1
	before = subject.requests.size(); subject._sample(true)
	check(subject.requests.size()==before,'existing 50ms sample gate retained')
	check(function_text(main,'_show').contains('_clear_mouse_movement()') and function_text(main,'_release').contains('_clear_mouse_movement()'),'actual panel/lifecycle cancellation hooks')
	for button in hud.button_nodes: check(button.focus_mode==Control.FOCUS_NONE and button.mouse_filter==Control.MOUSE_FILTER_STOP,'arrow focus/GUI consumption')
	var business_before: Dictionary = {'owner':subject.mouse_move_owner.duplicate(true),'rearm':subject.mouse_move_rearm,'neutral':subject.neutral_pending,'requests':subject.requests.duplicate(true)}
	subject._trace_pad('main','move_up','captured',{'owner':'SECRET','session':'SECRET','sx':1,'sy':0})
	check(subject.trace_rows.is_empty(),'trace OFF zero output')
	subject.trace_enabled = true; subject.clock = 1000
	subject._trace_pad('EVIL_SECRET','SECRET','SECRET',{'owner':'SECRET','context':'SECRET','event':'SECRET','token':'SECRET','path':'SECRET','argv':'SECRET','pressed':true,'sx':99,'sy':-1})
	var encoded: String = JSON.stringify(subject.trace_rows.back())
	check(not encoded.contains('SECRET') and not encoded.contains('owner') and not encoded.contains('path'),'trace fixed-schema malicious fields discarded')
	check(subject.trace_rows.back().sx==0 and subject.trace_rows.back().sy==-1,'trace axes bounded')
	for index in range(100): subject._trace_pad('raw','','down',{'pressed':true})
	check(subject.trace_rows.size()==80 and subject.trace_pad_records==80,'pad trace capped at 80')
	subject.trace_pad_records = 0; subject.trace_records = 223
	subject._trace_pad('raw','','up',{})
	check(subject.trace_rows.size()==80,'existing total trace cap retained')
	subject.trace_records = 0; subject.clock = 120000
	subject._trace_pad('raw','','up',{})
	check(subject.trace_rows.size()==80,'120 second trace bound')
	check(business_before=={'owner':subject.mouse_move_owner.duplicate(true),'rearm':subject.mouse_move_rearm,'neutral':subject.neutral_pending,'requests':subject.requests.duplicate(true)},'logger never changes owner/cancel/sample/request state')
	var sequences: Array = []
	for enabled in [false,true]:
		subject.trace_enabled = enabled; subject.trace_rows.clear(); subject.trace_records=0; subject.trace_pad_records=0; subject.clock+=1000; subject.trace_started_ms=subject.clock-100
		var count_before: int = subject.requests.size()
		subject.left=false; hud.left=false; subject._observe_mouse_movement(); subject.left=true; hud.left=true
		hud._update_movements(snapshot,true,subject._current_hud_movements(),true,subject._owner())
		state.capture=row.duplicate(true); subject.admit(row)
		subject._sample(false); subject.left=false; hud.left=false; subject._observe_mouse_movement()
		subject.clock+=51; subject._sample(false)
		var tail: Array = subject.requests.slice(subject.requests.size()-2)
		check(subject.requests.size()==count_before+2 and tail[0].sample.moveX!=0 and tail[1].sample.moveX==0 and subject.mouse_move_owner.is_empty(),'same move/cancel/request sequence trace='+str(enabled))
		sequences.append({'moves':[[tail[0].sample.moveX,tail[0].sample.moveZ],[tail[1].sample.moveX,tail[1].sample.moveZ]],'owner':subject.mouse_move_owner.duplicate(true),'rearm':subject.mouse_move_rearm,'neutral':subject.neutral_pending})
	check(sequences[0]==sequences[1],'identical business sequence trace OFF versus ON')
	var logger: String = function_text(main,'_trace_pad')
	check(not logger.contains('_request(') and not logger.contains('_sample(') and not logger.contains('_eligible(') and not logger.contains('_clear_mouse_movement('),'logger contains no business calls')
	var trace_header: String = '''extends RefCounted
var trace_enabled = true
var trace_started_ms = 0
var trace_records = 0
var trace_pad_records = 0
var trace_pad_samples = 0
var trace_observations = 0
var trace_input_count = 0
var trace_unhandled_count = 0
var keyboard_writes = 0
var mouse_move_owner = {}
var trace_rows = []
var clock = 10
func _trace_live(_now): return true
func _trace_write(_kind, _stage, _tag, _elapsed, _event): keyboard_writes += 1
'''+constants(main,['MOVEMENT_LABELS','TRACE_KEYS'])
	var raw_script: GDScript = compile_functions(trace_header,main,['_trace_pad','_trace_pad_sample','_trace_key','_event_key'])
	check(raw_script!=null,'actual three-argument trace interface compiles')
	if raw_script!=null:
		var raw = raw_script.new()
		var click: InputEventMouseButton = InputEventMouseButton.new()
		click.button_index=MOUSE_BUTTON_LEFT; click.pressed=true
		raw._trace_key(click,'input','received')
		click.pressed=false; raw._trace_key(click,'input','received')
		check(raw.trace_rows.size()==2 and raw.trace_rows[0].reason=='down' and raw.trace_rows[1].reason=='up','actual raw left down/up facts')
		var key: InputEventKey = InputEventKey.new(); key.keycode=KEY_F; key.pressed=true
		raw._trace_key(key,'input','received')
		check(raw.trace_input_count==1 and raw.keyboard_writes==1,'keyboard trace interface retained')
		raw._trace_pad_sample('sample',false,Vector2i.ZERO,true)
		check(raw.trace_rows.size()==2,'idle samples consume zero pad budget')
	await geometry_checks(snapshot,hud_script,rows)
	hud.movement_pad.free(); hud.movement_toggle.free(); hud.movement_help.free()
	subject.free()
	print('MOUSE_HOLD_PAD checks='+str(checks)+' failures='+str(failures))
	quit(0 if failures==0 else 1)

func geometry_checks(snapshot: Dictionary, script: Script, rows: Array) -> void:
	var viewport: SubViewport = SubViewport.new()
	viewport.size = Vector2i(1280,800)
	root.add_child(viewport)
	var view = script.new()
	viewport.add_child(view)
	await process_frame
	await process_frame
	var owner: Dictionary = {'generation':7,'panelGeneration':0,'context':{'worldId':snapshot.worldId,'sceneId':snapshot.sceneId,'worldEpoch':snapshot.worldEpoch}}
	var target: Dictionary = {'generation':7,'context':owner.context,'entityId':'checkpoint','kind':'checkpoint','label':'检查点'}
	var actions: Array = []
	for id in view.PLAYER_ACTION_LABELS: actions.append({'generation':7,'context':owner.context,'entityId':id,'kind':'player_action','label':view.PLAYER_ACTION_LABELS[id]})
	var guard: Dictionary = {'generation':7,'context':owner.context,'entityId':'guard','kind':'player_guard','label':'防御'}
	for scale in [1.0,1.25,1.5]:
		view.update_view(snapshot,'已连接',true,'[F] 检查点 · 当前可用',scale,target,{},actions,guard,rows,true,owner)
		for frame in range(12): await process_frame
		var panel: Rect2 = view.bottom.get_global_rect()
		check(panel.size.y<=360.01,'45 percent HUD cap scale='+str(scale))
		check(is_equal_approx(view.get_world_safe_rect().end.y,panel.position.y),'world-safe rect uses actual top scale='+str(scale))
		var controls: Array = [view.movement_toggle,view.movement_help,view.movement_pad.get_node('MouseHoldPadStop')]
		for state in view.movement_states.values(): controls.append(state.button)
		for control in controls:
			check(control.is_visible_in_tree() and panel.encloses(control.get_global_rect()),'pad control inside panel '+str(control.name)+' scale='+str(scale))
			check(not view.bottom_scroll.is_ancestor_of(control),'pad outside scrolling subtree '+str(control.name))
		check(not view.movement_toggle.get_global_rect().intersects(view.movement_help.get_global_rect()) and not view.movement_help.get_global_rect().intersects(view.movement_pad.get_global_rect()),'pad controls nonoverlap scale='+str(scale))
		var visible_left: Rect2 = view.bottom_scroll.get_global_rect()
		check(visible_left.encloses(view.hint.get_global_rect()) and visible_left.encloses(view.interact_button.get_global_rect()),'instructions/current interaction accessible without scroll scale='+str(scale))
		for state in view.player_action_states.values(): check(visible_left.encloses(state.button.get_global_rect()),'action accessible immediately '+str(state.button.text)+' scale='+str(scale))
		print('GEOMETRY scale='+str(scale)+' panel='+str(panel)+' left='+str(visible_left))
	view.update_view(snapshot,'已连接',true,'[F] 检查点',1.0,target,{},actions,guard,[],false,owner)
	for frame in range(8): await process_frame
	check(not view.movement_pad.visible and not view.movement_toggle.button_pressed,'default OFF compact projection')
	viewport.queue_free()
	await process_frame
