extends SceneTree
# Pure frontend seam test: no native service, physical input, Game scene or save.
const P=preload('res://scripts/bridge/protocol.gd')
var checks=0
var failures=0
class Transport extends RefCounted:
	var authenticated=true
	var connection_generation=1
	var calls: Array=[]
	var pending: Dictionary={}
	func request(method,params,association):
		var id=calls.size()+1
		calls.append({'id':id,'method':method,'params':params.duplicate(true),'association':association.duplicate(true)})
		pending[id]=association.duplicate(true)
		return id
class CatalogBoundary extends RefCounted:
	var scenes: Dictionary={}
	var manifest: Dictionary={}
	var current: Dictionary={}
	var source_hash='fixture-authored-scenes'
	func build(world,scene):
		if not scenes.has(scene) or scenes[scene].worldId!=world: return false
		current=scenes[scene]
		return true
	func contains(point):
		var b=current.boundsM
		return point.x>=b.x and point.x<=b.x+b.width and point.z>=b.z and point.z<=b.z+b.depth
class Peripheral extends RefCounted:
	var commits: Array=[]
	var resets: Array=[]
	func visible_point(_point): return true # Renderer visibility boundary, not collision proof.
	func commit(state,scene,_hash): commits.append({'state':state.duplicate(true),'scene':scene.sceneId})
	func clear_interaction_focus(): pass
	func suspend_world(): pass
	func stop_all(): pass
	func reset(epoch): resets.append(epoch)
	func ui_cue(_kind,_focused): pass
	func _clear_movement_captures(): pass
	func _cancel_guard_capture(): pass
	func gui_get_hovered_control(): return null
	func set_input_as_handled(): pass
	func quit(): pass
class PanelsBoundary extends RefCounted:
	var visible=false
	var mode='pause'
	var pending=false
	var messages: Array=[]
	func close_page(): visible=false
	func set_message(message,_blocked=false): messages.append(message)
func check(ok: bool,label: String) -> void:
	checks+=1
	if not ok: failures+=1;printerr('FAIL '+label)
func extract(text: String,name: String) -> String:
	var marker='func '+name+'('
	check(text.count(marker)==1,'unique actual production method '+name)
	if text.count(marker)!=1: return ''
	var start=text.find(marker);var end=text.find('\nfunc ',start+1)
	return text.substr(start) if end<0 else text.substr(start,end-start)
func point(position: Array) -> Dictionary:
	return {'xM':position[0],'yM':position[1],'zM':position[2]}
func player_spawn(scene: Dictionary) -> Array:
	var found=scene.spawns.filter(func(row):return row.kind=='player')
	check(found.size()==1,'unique authored player spawn '+scene.sceneId)
	return found[0].position
func snapshot_for(scene: Dictionary,position: Array,epoch: int,token=null) -> Dictionary:
	var p=point(position)
	var interactables: Array=[]
	for row in scene.interactions:
		if row.kind=='world_gate_marker':
			interactables.append({'entityId':row.id,'kind':'world_gate','active':true,'transform':{'positionM':point(row.position),'yawRad':0.0}})
	return {'kind':'full','protocolVersion':3,'schemaVersion':'freeze-v02-interfaces/1.2','worldId':scene.worldId,'sceneId':scene.sceneId,'worldEpoch':epoch,'checkpointId':null,'serverTick':10,'authorityRevision':2,'ackSeq':0,'entryToken':token,'actors':[],'doors':[],'interactables':interactables,'hazards':[],'objectives':[],'capabilities':{'schemaVersion':1},'progression':{'schemaVersion':1,'eventSeq':1,'currentWorldId':scene.worldId,'worlds':[]},'player':{'entityId':'player','transform':{'positionM':p,'yawRad':0.0},'velocityMps':{'xM':0.0,'yM':0.0,'zM':0.0},'actionState':'idle','currentHp':100,'maxHp':100,'currentEnergy':100,'maxEnergy':100,'facingX':0.0,'facingZ':1.0,'aimX':0.0,'aimZ':1.0}}
func receipt(command: String,state: Dictionary) -> Dictionary:
	return {'commandId':command,'applied':true,'alreadyApplied':false,'errorCode':null,'snapshot':state,'worldEpoch':state.worldEpoch,'serverTick':state.serverTick,'authorityRevision':state.authorityRevision}
func wire(value):
	return JSON.parse_string(JSON.stringify(value,'',true,true))
func lifecycle(state: Dictionary,paused: bool,token=null) -> Dictionary:
	return {'worldId':state.worldId,'sceneId':state.sceneId,'worldEpoch':state.worldEpoch,'paused':paused,'lastLifecycleCommandSequence':0,'entryToken':token,'enhancementTerminalContext':null}
func press_f(a) -> void:
	var event=InputEventKey.new();event.physical_keycode=KEY_F;event.keycode=KEY_F;event.pressed=true
	a._unhandled_input(event)
func ready_calls(a) -> Array:
	return a.rpc.calls.filter(func(call):return call.method=='formal_scene_ready')
func _initialize() -> void:
	var text=FileAccess.get_file_as_string('res://scripts/ui/v1_main.gd').replace('\r\n','\n')
	var selected=['_context','_assoc','_current','_request','_id','_eligible','_nearest','_interact','_response','_accept_snapshot','_prepare_entry','_scene_frame_committed','_interaction_result','_unhandled_input','_event_key','_release','_clear_mouse_movement','_keyboard_move_state','_clear_mouse_guard','_end_guard','_feedback','_localized','_hold','_clockworks_completed']
	var header="extends RefCounted\nconst P=preload('res://scripts/bridge/protocol.gd')\nconst Rpc=preload('res://scripts/bridge/rpc_client.gd')\nconst Content=preload('res://scripts/ui/content_model.gd')\nvar clock=1000\nvar boundary_events: Array=[]\nvar viewport\nvar tree\n"
	for line in text.substr(0,text.find('func _ready()')).split('\n'):
		if line.begins_with('const ACTION_KEYS = ') or line.begins_with('const TRACE_KEYS = ') or line.begins_with('const SCENE_RPC = ') or line.begins_with('const INTERACT_KINDS = '): header+=line+'\n'
		if line.begins_with('var '):
			if line.begins_with('var catalog ') or line.begins_with('var event_view '): line='var '+line.get_slice(' ',1)
			header+=line+'\n'
	var body=''
	for name in selected: body+=extract(text,name)+'\n'
	# Only deterministic clock, theme/UI and physical peripheral boundaries change.
	body=body.replace('Time.get_ticks_msec()','clock').replace('ThemeModel.apply(','_theme_apply(').replace('Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)','false')
	var boundaries='''
func _key_held(_key): return false
func _mouse_guard_valid(): return false
func _sample(neutral): boundary_events.append(['physical_sample',neutral]);return true
func _action(kind,_force=false): boundary_events.append(['physical_action',kind])
func _theme_apply(_control,_world): pass
func _show(page,_data={},_blocked=false,_extra=false,_preserve=false): panels.mode=page;panels.visible=true
func _open_paused(_page,_data={}): assert(false,'unexpected peripheral page branch')
func _escape(): assert(false,'unexpected Escape branch')
func _storage_inventory_available(_i): assert(false,'unexpected storage branch');return false
func _baizhi_available(_i): assert(false,'unexpected dialogue branch');return false
func _begin_baizhi(): assert(false,'unexpected dialogue branch')
func _advance_lifecycle(): assert(false,'unexpected lifecycle command branch');return false
func _save_read_response(_id,_method,_result,_association): assert(false,'unexpected save read')
func _close_current(_id,_method,_association): assert(false,'unexpected close');return false
func _submit_close(): assert(false,'unexpected save-close branch')
func _trace_frontend(_stage,_facts={}): pass
func _trace_key(_event,_stage,_reason): pass
func _trace_pad(_source,_id,_reason,_facts={}): pass
func _trace_save_flow(_stage,_method,_id,_receipt={},_state={}): pass
func _trace_sink_close(): pass
func get_viewport(): return viewport
func get_tree(): return tree
'''
	var script=GDScript.new();script.source_code=header+boundaries+body
	var error=script.reload();check(error==OK and script.can_instantiate(),'actual handoff methods compile; no substituted entry handlers')
	if failures or error!=OK: finish();return
	var rs=JSON.parse_string(FileAccess.get_file_as_string('res://../../content/scenes/compiled/rs_core_room.json'))
	var gh=JSON.parse_string(FileAccess.get_file_as_string('res://../../content/scenes/compiled/gh_entry_maintenance.json'))
	check(rs is Dictionary and gh is Dictionary,'actual authored scenes loaded')
	if not rs is Dictionary or not gh is Dictionary: finish();return
	var a=script.new();a.rpc=Transport.new();a.catalog=CatalogBoundary.new()
	a.catalog.scenes={rs.sceneId:rs,gh.sceneId:gh};a.catalog.manifest={rs.sceneId:true,gh.sceneId:true};a.catalog.current=rs
	for field in ['renderer','audio','event_view','hud','viewport','tree']: a.set(field,Peripheral.new())
	a.panels=PanelsBoundary.new();a.generation=4;a.journey_started=true
	var spawn=player_spawn(rs)
	check(spawn==[4.0,0.0,8.0],'authored station spawn4,0,8')
	a.snapshot=snapshot_for(rs,spawn,1);a.status=lifecycle(a.snapshot,false);a.last_snapshot_at=1000;a.last_status_at=1000
	check(P.snapshot(a.snapshot) and P.status(a.status) and a._eligible(),'protocol-valid current station is eligible')
	check(a._nearest().is_empty(),'station spawn cannot select distant gate')
	press_f(a)
	check(a.rpc.calls.is_empty(),'actual F dispatch at station spawn submits no distant gate')
	var gates=rs.interactions.filter(func(row):return row.id=='rs_world_gate_marker')
	check(gates.size()==1 and gates[0].kind=='world_gate_marker','unique authored GH gate marker')
	var gate_position: Array=gates[0].position
	# Authority-position boundary: authored approach, not simulated walking proof.
	a.snapshot.player.transform.positionM=point([gate_position[0]-1.0,gate_position[1],gate_position[2]])
	check(a._nearest().get('entityId')=='rs_world_gate_marker','authored gate approach selects GH marker')
	press_f(a)
	check(a.rpc.calls.size()==1 and a.rpc.calls[0].method=='formal_world_gate','actual F submits exactly one world-gate request')
	var gate_call: Dictionary=a.rpc.calls[0]
	check(gate_call.params.id=='rs_world_gate_to_gh' and gate_call.params.worldEpoch==1 and gate_call.association.target.entityId=='rs_world_gate_marker','actual production request routes GH gate with current identity')
	check(not a._eligible(),'pending gate request blocks gameplay eligibility')
	var token={'worldId':gh.worldId,'sceneId':gh.sceneId,'worldEpoch':2,'generation':5}
	var entered=snapshot_for(gh,player_spawn(gh),2,token)
	var parsed=wire(receipt(gate_call.params.requestId,entered))
	check(P.receipt(parsed) and typeof(parsed.snapshot.worldEpoch)==TYPE_FLOAT,'actual JSON-parsed successful gate receipt is protocol-valid')
	a.logical_down={KEY_D:true};a.mouse_move_owner={'entityId':'move_right'};a.pending_actions=[{'kind':'primaryAttack'}];a.guard_active=true;a.frame_key='return_station:rs_core_room:1'
	a._response(gate_call.id,gate_call.method,parsed,gate_call.association)
	check(a.snapshot.worldId=='grey_hive' and a.snapshot.sceneId=='gh_entry_maintenance' and a.catalog.current.sceneId==gh.sceneId,'actual response installs GH authored scene')
	check(a.logical_down.is_empty() and a.mouse_move_owner.is_empty() and a.pending_actions.is_empty() and not a.guard_active and not a.neutral_pending,'scene installation clears old movement/actions without stale gameplay')
	check(a.snapshot.entryToken==parsed.snapshot.entryToken and a.ready_pending==parsed.snapshot.entryToken,'actual prepare-entry retains pending token')
	check(a.renderer.commits.size()==1 and ready_calls(a).is_empty() and not a._eligible(),'render submission is not frame commit or readiness')
	a._scene_frame_committed('return_station:rs_core_room:1')
	check(ready_calls(a).is_empty() and not a._eligible(),'prior-scene frame cannot unlock GH entry')
	var frame=gh.worldId+':'+gh.sceneId+':'+str(a.snapshot.worldEpoch)
	a._scene_frame_committed(frame)
	check(ready_calls(a).size()==1,'matching scene-frame commit emits exactly one readiness request')
	var ready: Dictionary=ready_calls(a)[0]
	check(ready.params.token==a.ready_pending and ready.params.remainPaused==false and ready.association.generation==a.generation,'ready request carries actual token/current association/unpaused intent')
	a._scene_frame_committed(frame)
	check(ready_calls(a).size()==1 and not a._eligible(),'repeated frame cannot duplicate readiness or unlock gameplay')
	var ready_state=entered.duplicate(true);ready_state.entryToken=null;ready_state.serverTick=11;ready_state.authorityRevision=3
	var ready_reply=wire(receipt('scene-ready:'+str(int(token.generation)),ready_state))
	check(P.receipt(ready_reply),'protocol-valid accepted readiness response')
	a._response(ready.id,ready.method,ready_reply,ready.association)
	check(a.ready_pending.is_empty() and a.snapshot.entryToken==null and a.status.is_empty() and not a._eligible(),'ready receipt clears token but waits for current status')
	var statuses=a.rpc.calls.filter(func(call):return call.method=='bridge_session_status')
	check(statuses.size()==1,'actual ready handler requests lifecycle status once')
	var status_call: Dictionary=statuses[0]
	a._response(status_call.id,status_call.method,wire(lifecycle(parsed.snapshot,true,parsed.snapshot.entryToken)),status_call.association)
	check(not a._eligible(),'pending-entry paused status cannot restore gameplay')
	a._response(status_call.id,status_call.method,wire(lifecycle(ready_state,false)),status_call.association)
	check(a._eligible() and P.same(a.status,a.snapshot),'accepted ready plus current unpaused status restores actual eligibility')
	a._scene_frame_committed('return_station:rs_core_room:1');a._scene_frame_committed(frame)
	check(ready_calls(a).size()==1 and a._eligible(),'post-ready callbacks do not duplicate or manufacture readiness')
	finish()
func finish() -> void:
	print('GH_ENTRY_HANDOFF checks='+str(checks)+' failures='+str(failures))
	quit(1 if failures else 0)
