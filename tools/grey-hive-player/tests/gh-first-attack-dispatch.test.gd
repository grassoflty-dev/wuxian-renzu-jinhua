extends SceneTree
# Frontend queue/dispatch ordering only; no native service, hit or damage proof.
const P=preload('res://scripts/bridge/protocol.gd')
var checks=0
var failures=0
class Transport extends RefCounted:
	var authenticated=true
	var connection_generation=1
	var calls: Array=[]
	var reject_inputs=0
	var sequence=0
	func request(method,params,association):
		var accepted=true
		if method=='formal_submit_input' and reject_inputs>0:
			reject_inputs-=1;accepted=false
		if accepted: sequence+=1
		calls.append({'id':sequence if accepted else 0,'method':method,'params':params.duplicate(true),'association':association.duplicate(true),'submitted':accepted})
		return sequence if accepted else 0
class CameraBoundary extends RefCounted:
	var point=Vector3.ZERO
	func inverse(_screen,_height): return point
class Peripheral extends RefCounted:
	var camera=CameraBoundary.new()
	var current: Dictionary={}
	var cursor=0
	var sound_cursor=0
	var sound=[]
	var events=[]
	var failed=false
	var status_text=''
	func update(_snapshot,_scene,_generation,_eligible,_focused): pass
	func project(_snapshot,_renderer,_eligible): return []
	func visible_point(_position): return true
	func clear_interaction_focus(): pass
	func set_interaction_focus(_row,_snapshot,_eligible,_generation): pass
	func suspend_world(): pass
	func ui_cue(_kind,_focused): pass
	func _clear_movement_captures(): pass
	func _cancel_guard_capture(): pass
	func gui_get_hovered_control(): return null
	func get_mouse_position(): return Vector2(100,100)
	func set_input_as_handled(): pass
	func update_view(_snapshot,_connection,_eligible,_hint,_scale,_interaction,_traversal,_actions,_guard,_movements,_pad,_owner): pass
class PanelsBoundary extends RefCounted:
	var visible=false
	var mode='pause'
	func set_message(_message): pass
func check(ok: bool,label: String) -> void:
	checks+=1
	if not ok: failures+=1;printerr('FAIL '+label)
func extract(text: String,name: String) -> String:
	var marker='func '+name+'('
	check(text.count(marker)==1,'unique actual production method '+name)
	if text.count(marker)!=1: return ''
	var start=text.find(marker);var end=text.find('\nfunc ',start+1)
	return text.substr(start) if end<0 else text.substr(start,end-start)
func snapshot_for(scene: Dictionary,position: Array) -> Dictionary:
	var point={'xM':position[0],'yM':position[1],'zM':position[2]}
	return {'kind':'full','protocolVersion':3,'schemaVersion':'freeze-v02-interfaces/1.2','worldId':scene.worldId,'sceneId':scene.sceneId,'worldEpoch':2,'checkpointId':null,'serverTick':10,'authorityRevision':2,'ackSeq':0,'entryToken':null,'actors':[],'doors':[],'interactables':[],'hazards':[],'objectives':[],'capabilities':{'schemaVersion':1},'progression':{'schemaVersion':1,'eventSeq':1,'currentWorldId':scene.worldId,'worlds':[]},'player':{'entityId':'player','transform':{'positionM':point,'yawRad':0.0},'velocityMps':{'xM':0.0,'yM':0.0,'zM':0.0},'actionState':'idle','currentHp':100,'maxHp':100,'currentEnergy':100,'maxEnergy':100,'facingX':0.0,'facingZ':1.0,'aimX':0.0,'aimZ':1.0}}
func subject(script: GDScript,scene: Dictionary,position: Array,target: Array):
	var a=script.new();a.rpc=Transport.new();a.panels=PanelsBoundary.new()
	for field in ['catalog','renderer','audio','event_view','hud','viewport']: a.set(field,Peripheral.new())
	a.catalog.current=scene;a.renderer.camera.point=Vector3(target[0],target[1],target[2])
	a.snapshot=snapshot_for(scene,position);a.status={'worldId':scene.worldId,'sceneId':scene.sceneId,'worldEpoch':2,'paused':false,'lastLifecycleCommandSequence':0,'entryToken':null,'enhancementTerminalContext':null}
	a.journey_started=true;a.generation=4;a.clock=1000;a.last_snapshot_at=1000;a.last_status_at=1000;a.pointer_seen=true
	return a
func press_j(a,echo=false) -> void:
	var event=InputEventKey.new();event.physical_keycode=KEY_J;event.keycode=KEY_J;event.pressed=true;event.echo=echo
	a._unhandled_input(event)
func calls(a,method: String) -> Array:
	return a.rpc.calls.filter(func(row):return row.method==method)
func tick(a,time: int) -> void:
	a.clock=time;a._process(0.05)
func _initialize() -> void:
	var text=FileAccess.get_file_as_string('res://scripts/ui/v1_main.gd').replace('\r\n','\n')
	var selected=['_context','_owner','_assoc','_current','_request','_eligible','_unhandled_input','_event_key','_action','_process','_sample','_send_action','_release','_end_guard','_clear_mouse_movement','_clear_mouse_guard','_keyboard_move_state','_movement_axes','_nearest','_feedback','_hold','_localized']
	var header="extends RefCounted\nconst P=preload('res://scripts/bridge/protocol.gd')\nvar clock=1000\nvar viewport\n"
	for line in text.substr(0,text.find('func _ready()')).split('\n'):
		if line.begins_with('const ACTION_KEYS = ') or line.begins_with('const TRACE_KEYS = ') or line.begins_with('const MOVEMENT_AXES = '): header+=line+'\n'
		if line.begins_with('var '):
			if line.begins_with('var catalog ') or line.begins_with('var event_view '): line='var '+line.get_slice(' ',1)
			header+=line+'\n'
	# MOVEMENT_AXES is declared after _ready, so retain that literal definition too.
	for line in text.split('\n'):
		if line.begins_with('const MOVEMENT_AXES = '): header+=line+'\n'
	var body=''
	for name in selected: body+=extract(text,name)+'\n'
	body=body.replace('Time.get_ticks_msec()','clock').replace('Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)','false')
	var boundaries='''
func _key_held(_key): return false
func _mouse_guard_valid(): return false
func _mouse_movement_valid(): return false
func _observe_mouse_movement(): pass
func _observe_mouse_guard(): pass
func _trace_tick(): pass
func _trace_key(_event,_stage,_reason): pass
func _trace_pad_sample(_reason,_neutral,_axes=Vector2i.ZERO,_accepted=false): pass
func _trace_pad(_stage,_id,_reason,_facts={}): pass
func _trace_save_flow(_stage,_method,_id,_receipt={},_state={}): pass
func _trace_frontend(_stage,_facts={}): pass
func _current_hud_interaction(): return {}
func _current_hud_traversal(): return {}
func _current_hud_actions(): return []
func _current_hud_guard(): return {}
func _current_hud_movements(): return []
func _interaction_prompt(_nearest): assert(false,'unexpected nonattack UI branch');return ''
func _interact(): assert(false,'unexpected F branch')
func _escape(): assert(false,'unexpected Escape branch')
func _open_paused(_page): assert(false,'unexpected peripheral page branch')
func _show(page): panels.mode=page;panels.visible=true
func get_viewport(): return viewport
'''
	var script=GDScript.new();script.source_code=header+boundaries+body
	var error=script.reload();check(error==OK and script.can_instantiate(),'actual action/queue/process/sample/send/release methods compile')
	if failures or error!=OK: finish();return
	var scene=JSON.parse_string(FileAccess.get_file_as_string('res://../../content/scenes/compiled/gh_entry_maintenance.json'))
	check(scene is Dictionary and scene.worldId=='grey_hive' and scene.sceneId=='gh_entry_maintenance','authored first GH scene context')
	if not scene is Dictionary: finish();return
	var players=scene.spawns.filter(func(row):return row.kind=='player')
	var enemies=scene.spawns.filter(func(row):return row.kind=='enemy')
	check(players.size()==1 and not enemies.is_empty(),'authored player and aim-target positions available')
	if players.size()!=1 or enemies.is_empty(): finish();return
	var position: Array=players[0].position;var target: Array=enemies[0].position
	var a=subject(script,scene,position,target)
	check(P.snapshot(a.snapshot) and P.status(a.status) and a._eligible(),'protocol-valid ready current GH context is eligible')
	press_j(a);press_j(a,true)
	check(a.pending_actions.size()==1 and a.pending_actions[0].kind=='primaryAttack' and a.pending_actions[0].time==1000,'actual J queues one attack; echoed J ignored')
	check(calls(a,'formal_submit_action').is_empty(),'J queue admission does not dispatch attack early')
	tick(a,1000)
	var input=calls(a,'formal_submit_input');var attack=calls(a,'formal_submit_action')
	check(input.size()==1 and input[0].submitted and attack.size()==1 and attack[0].submitted,'actual sampling tick submits input and one attack')
	if input.size()==1 and attack.size()==1:
		var expected_aim=Vector2(target[0]-position[0],target[2]-position[2]).normalized()
		check(input[0].params.sample.aimX==expected_aim.x and input[0].params.sample.aimZ==expected_aim.y,'actual sample carries current camera-projected aim')
		check(input[0].params.sample.worldEpoch==a.snapshot.worldEpoch and attack[0].params.action.worldEpoch==a.snapshot.worldEpoch and attack[0].params.action.kind=='primaryAttack','input and attack carry current authored GH epoch')
		check(a.rpc.calls.find(input[0])<a.rpc.calls.find(attack[0]) and input[0].id<attack[0].id,'current aim input submission precedes attack dispatch')
		check(attack[0].association.context==a._context() and attack[0].association.generation==4,'attack association belongs to current world/generation')
	check(a.pending_actions.is_empty(),'actual process consumes dispatched queue')
	tick(a,1050)
	check(calls(a,'formal_submit_input').size()==2 and calls(a,'formal_submit_action').size()==1,'next sampling tick does not resend consumed attack')
	var rejected=subject(script,scene,position,target);rejected.rpc.reject_inputs=1;press_j(rejected);tick(rejected,1000)
	check(calls(rejected,'formal_submit_input').size()==1 and not calls(rejected,'formal_submit_input')[0].submitted,'transport boundary rejects actual input submission')
	check(rejected.pending_actions.size()==1 and calls(rejected,'formal_submit_action').is_empty(),'rejected input retains queued attack and sends no action')
	tick(rejected,1050)
	check(rejected.pending_actions.is_empty() and calls(rejected,'formal_submit_action').size()==1,'next successful input dispatches retained attack once')
	var recovered_input=calls(rejected,'formal_submit_input')[1];var recovered_attack=calls(rejected,'formal_submit_action')[0]
	check(recovered_input.submitted and rejected.rpc.calls.find(recovered_input)<rejected.rpc.calls.find(recovered_attack),'recovered input still precedes attack')
	tick(rejected,1100);check(calls(rejected,'formal_submit_action').size()==1,'recovery does not resend attack next tick')
	var gated=subject(script,scene,position,target);gated.last_input_packet_at=1000;press_j(gated);tick(gated,1020)
	check(calls(gated,'formal_submit_input').is_empty() and calls(gated,'formal_submit_action').is_empty() and gated.pending_actions.size()==1,'actual sample timing gate preserves attack without transport submission')
	tick(gated,1050)
	check(calls(gated,'formal_submit_input').size()==1 and calls(gated,'formal_submit_action').size()==1 and gated.pending_actions.is_empty(),'first allowed sample dispatches timing-gated attack once')
	var paused=subject(script,scene,position,target);press_j(paused);paused.status.paused=true;paused.panels.visible=true
	check(not paused._eligible() and paused.pending_actions.size()==1,'pause before sampling blocks gameplay with queued attack')
	paused._release()
	check(paused.pending_actions.is_empty() and calls(paused,'formal_submit_action').is_empty(),'actual release clears queued attack without firing')
	press_j(paused)
	check(paused.pending_actions.is_empty(),'paused J cannot requeue attack')
	paused.status.paused=false;paused.panels.visible=false;tick(paused,1050)
	check(paused._eligible() and calls(paused,'formal_submit_input').size()==1 and calls(paused,'formal_submit_action').is_empty(),'resume samples input but never fires released attack')
	finish()
func finish() -> void:
	print('GH_FIRST_ATTACK_DISPATCH checks='+str(checks)+' failures='+str(failures))
	quit(1 if failures else 0)
