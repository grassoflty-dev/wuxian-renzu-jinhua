extends Node2D
const Content = preload('res://scripts/ui/content_model.gd')
const ThemeModel = preload('res://scripts/ui/theme_model.gd')
const P = preload('res://scripts/bridge/protocol.gd')
const Rpc = preload('res://scripts/bridge/rpc_client.gd')
const Catalog = preload('res://scripts/world/scene_catalog.gd')
const Renderer = preload('res://scripts/world/renderer.gd')
const EventView = preload('res://scripts/world/event_view.gd')
const Hud = preload('res://scripts/ui/game_hud.gd')
const NativeAudio = preload('res://scripts/audio/native_audio.gd')
const Panels = preload('res://scripts/ui/panels.gd')
const FREE_TRAVERSAL_STARTS = {
	'cw_conveyor_bridge':{'row':{'id':'cw_conveyor_context_vault','from':[8.0,0.0,8.0],'to':[10.0,0.0,8.0],'rangeM':1.2,'cooldownMs':350,'requiredCapabilities':[]},'band':[0.0,0.1],'label':'输送桥短跨'},
	'cw_gear_shaft':{'row':{'id':'cw_gear_upper_gap','from':[17.8,2.0,8.0],'to':[20.2,2.0,8.0],'rangeM':1.2,'cooldownMs':350,'requiredCapabilities':[],'fromHeightRangeM':[1.9,2.1]},'band':[1.9,2.1],'label':'上层间隙'}
}
const EARNED_TRAVERSAL_START = {'row':{'id':'cw_gear_earned_air_step','from':[11.0,0.0,8.0],'to':[16.0,2.0,8.0],'rangeM':1.2,'cooldownMs':350,'fromHeightRangeM':[0.0,0.1],'requiredCapabilities':['mobility.air_step_i']},'band':[0.0,0.1],'label':'空中踏步'}
const ACTION_KEYS = {KEY_J:'primaryAttack',KEY_SHIFT:'dash',KEY_SPACE:'contextTraversal',KEY_Q:'pulse',KEY_E:'guardStart',KEY_R:'pierce'}
const SCENE_RPC = {'scene_transition':'formal_scene_transition','scene_checkpoint':'formal_scene_checkpoint','scene_trigger':'formal_scene_trigger','environment_control':'formal_environment_control','world_gate':'formal_world_gate','mission_terminal':'formal_mission_terminal_status','capability_terminal':'formal_capability_terminal_status','save_rest_terminal':'formal_save_rest_terminal'}
const INTERACT_KINDS = ['power_console','door_panel','terminal','lockdown_terminal','facility_log','bio_log_terminal','gate_b_panel','extraction_console','beacon','pump_control','valve_control','beacon_collect','beacon_mount','coolant_valve','static_dialogue_marker']
var audio
var rpc
var catalog = Catalog.new()
var renderer
var event_view = EventView.new()
var hud
var panels
var snapshot: Dictionary = {}
var status: Dictionary = {}
var generation = 0
var panel_generation = 0
var request_sequence = 0
var input_sequence = 0
var action_sequence = 0
var lifecycle_sequence = 0
var journey_started = false
var focused = true
var guard_active = false
var blocked_keys = {}
var logical_down: Dictionary = {}
var critical = ''
var uncertain = false
var connection_text = '未连接'
var poll_accumulator = 0.0
var input_accumulator = 0.0
var last_snapshot_at = 0
var ready_pending: Dictionary = {}
var ready_sent = {}
var frame_key = ''
var pause_owned: Dictionary = {}
var ticket: Dictionary = {}
var enhancement_pending: Dictionary = {}
var last_feedback = ''
var preferences = {'scale':1.0,'fullscreen':false,'color':false,'reduced':false,'muted':false,'volume':0.35}
var pointer_seen = false
var pending_actions: Array = []
var guard_blocked = false
var last_status_at = 0
var last_input_packet_at = -1000
var neutral_pending = false
var close_attempted = false
var close_pending: Dictionary = {}
var close_confirmed = false
var close_submitting = false
var save_read_sequence = 0
var save_reads: Dictionary = {}
var default_save_state = 'unknown'
var slot_list_state = 'unknown'
var current_slots: Array = []
var save_projection_generation = -1
var save_projection_connection = -1
const TRACE_KEYS = [KEY_W,KEY_A,KEY_S,KEY_D,KEY_UP,KEY_DOWN,KEY_LEFT,KEY_RIGHT,KEY_J,KEY_SHIFT,KEY_SPACE,KEY_Q,KEY_E,KEY_R,KEY_F,KEY_T,KEY_M,KEY_ESCAPE]
var trace_enabled = false
var trace_started_ms = 0
var trace_records = 0
var trace_heartbeats = 0
var trace_observations = 0
var trace_input_count = 0
var trace_unhandled_count = 0
var trace_next_ms = 0
func _ready() -> void:
	name = 'V1Main'
	audio = NativeAudio.new()
	audio.name = 'Audio'
	add_child(audio)
	get_tree().auto_accept_quit = false
	get_window().min_size = Vector2i(720,480)
	renderer = Renderer.new()
	renderer.name = 'World'
	add_child(renderer)
	renderer.frame_committed.connect(_scene_frame_committed)
	var ui = CanvasLayer.new()
	ui.name = 'UI'
	add_child(ui)
	hud = Hud.new()
	hud.name = 'GameHUD'
	ui.add_child(hud)
	hud.intent.connect(func(kind,value): _ui_intent(kind,value,_owner() if kind in ['interact_current','traverse_current'] else panels.page_owner.duplicate(true)))
	panels = Panels.new()
	panels.name = 'Panels'
	ui.add_child(panels)
	panels.intent.connect(_ui_intent)
	rpc = Rpc.new()
	rpc.name = 'Bridge'
	add_child(rpc)
	rpc.response.connect(_response)
	rpc.failed.connect(_failed)
	rpc.connection_changed.connect(_connection)
	_trace_start()
	_show('start')
	rpc.start_from_arguments()
func _context() -> Dictionary:
	return {} if snapshot.is_empty() else {'worldId':snapshot.worldId,'sceneId':snapshot.sceneId,'worldEpoch':snapshot.worldEpoch}
func _owner() -> Dictionary:
	return {'generation':generation,'panelGeneration':panel_generation,'context':_context()}
func _assoc(extra: Dictionary = {}) -> Dictionary:
	var a = {'generation':generation,'context':_context()}
	a.merge(extra,true)
	return a
func _current(a: Dictionary, transition: bool = false) -> bool:
	return a.get('generation')==generation and (transition or a.get('context',{}).is_empty() or P.same(a.context,_context()))
func _request(method: String, params: Dictionary, purpose: String = '', extra: Dictionary = {}, block: bool = false) -> int:
	if close_attempted and not (close_submitting and method=='session_close'):
		return 0
	if block:
		if not critical.is_empty() or uncertain:
			_feedback('当前请求尚未完成')
			return 0
		_release()
		critical = method
	extra['purpose'] = purpose
	var id = rpc.request(method,params,_assoc(extra))
	if id==0 and block:
		critical = ''
		_feedback('链路暂不可用')
	return id
func _id(prefix: String) -> String:
	request_sequence += 1
	return prefix+'-'+str(generation)+'-'+str(request_sequence)
func _connection(connected: bool, code: String) -> void:
	if connected:
		close_confirmed = false
	if close_confirmed:
		return
	if connected:
		close_attempted = false
		close_pending.clear()
		close_submitting = false
	_invalidate_saves()
	connection_text = '已连接' if connected else '未连接 / '+_localized(code)
	if not connected:
		_release()
		logical_down.clear()
		status.clear()
		if journey_started:
			uncertain = true
			_show('error')
			panels.set_message('连接中断；未确认操作保持结果未知',false)
		else:
			_show('start')
	else:
		_show('start')
		_refresh_saves()
func _eligible() -> bool:
	return not close_attempted and journey_started and rpc.authenticated and focused and not uncertain and critical.is_empty() and not panels.visible and not snapshot.is_empty() and not status.is_empty() and P.same(status,snapshot) and not status.paused and status.get('entryToken')==null and snapshot.get('entryToken')==null and ready_pending.is_empty() and snapshot.player.currentHp>0 and Time.get_ticks_msec()-last_snapshot_at<1000 and Time.get_ticks_msec()-last_status_at<1000
func _process(delta: float) -> void:
	_trace_tick()
	if close_confirmed:
		return
	poll_accumulator += delta
	input_accumulator += delta
	if not close_attempted and rpc.authenticated and poll_accumulator>=0.05:
		poll_accumulator = 0
		_request('formal_snapshot',{},'poll')
		_request('bridge_session_status',{},'status')
		if journey_started and not snapshot.is_empty():
			_request('formal_presentation_events',{'worldEpoch':snapshot.worldEpoch,'afterEventId':event_view.cursor},'events')
			_request('formal_sound_cues',{'worldEpoch':snapshot.worldEpoch,'afterEventId':event_view.sound_cursor},'sound')
	var eligible = _eligible()
	audio.update(snapshot,catalog.current,generation,eligible,focused)
	if input_accumulator>=0.05:
		input_accumulator = 0
		if eligible:
			if _sample(false):
				var queued = pending_actions.duplicate(true)
				pending_actions.clear()
				for edge in queued:
					_send_action(edge.kind,edge.time)
		elif neutral_pending and critical.is_empty():
			_sample(true)
	if guard_active and (not eligible or not _key_held(KEY_E)):
		_end_guard()
	for key in blocked_keys.keys():
		if not _key_held(key):
			blocked_keys.erase(key)
	if not snapshot.is_empty():
		renderer.events = event_view.project(snapshot,renderer,eligible)
		renderer.sound = event_view.sound
	var nearest = _nearest()
	var hint = audio.status_text if audio.failed else last_feedback
	if eligible and not nearest.is_empty():
		hint = _interaction_prompt(nearest)+' · '+hint
	hud.update_view(snapshot,connection_text,eligible,hint,preferences.scale,_current_hud_interaction(),_current_hud_traversal())
func _sample(neutral: bool) -> bool:
	if close_attempted:
		return false
	if snapshot.is_empty() or not rpc.authenticated:
		return false
	if snapshot.get('entryToken')!=null or not ready_pending.is_empty() or not status.is_empty() and P.same(status,snapshot) and status.paused:
		neutral_pending = false
		return false
	var now = Time.get_ticks_msec()
	if now-last_input_packet_at<50:
		if neutral:
			neutral_pending = true
		return false
	input_sequence += 1
	if input_sequence>=9007199254740991:
		if not uncertain:
			_hold('E_COUNTER_EXHAUSTED')
		return false
	var right = _key_held(KEY_D) or _key_held(KEY_RIGHT)
	var left = _key_held(KEY_A) or _key_held(KEY_LEFT)
	var down = _key_held(KEY_S) or _key_held(KEY_DOWN)
	var up = _key_held(KEY_W) or _key_held(KEY_UP)
	var sx = int(right)-int(left)
	var sy = int(down)-int(up)
	var move = Vector2(sy+0.5*sx,sy-0.5*sx).normalized() if not neutral else Vector2.ZERO
	var aim = Vector2.ZERO
	if pointer_seen and not neutral:
		var player = P.vector(snapshot.player.transform.positionM)
		var mouse_world = renderer.camera.inverse(get_viewport().get_mouse_position(),player.y)
		aim = Vector2(mouse_world.x-player.x,mouse_world.z-player.z).normalized()
	var sample = {'protocol':'continuous-input','protocolVersion':2,'worldEpoch':snapshot.worldEpoch,'seq':input_sequence,'clientTimeMs':Time.get_ticks_msec(),'moveX':move.x,'moveZ':move.y,'aimX':aim.x,'aimZ':aim.y}
	var submitted = _request('formal_submit_input',{'sample':sample,'combat':[]},'input',{'neutral':neutral})
	if submitted>0:
		last_input_packet_at = now
		neutral_pending = false
		return true
	return false
func _action(kind: String, release: bool = false) -> void:
	if not release and not _eligible():
		_feedback('当前状态不能执行动作')
		return
	if snapshot.is_empty() or not rpc.authenticated or not kind in ['primaryAttack','dash','pulse','guardStart','guardEnd','pierce','contextTraversal']:
		return
	if kind=='guardStart' and guard_blocked:
		_feedback('请松开 E 后重新防御')
		return
	if not release:
		if pending_actions.size()>=32:
			_hold('E_ACTION_QUEUE_LIMIT')
			return
		pending_actions.append({'kind':kind,'time':Time.get_ticks_msec()})
		return
	_send_action(kind,Time.get_ticks_msec())
func _send_action(kind: String, _event_time: int) -> void:
	if close_attempted:
		return
	action_sequence += 1
	if action_sequence>=9007199254740991:
		_hold('E_COUNTER_EXHAUSTED')
		return
	var action = {'protocolVersion':2,'worldEpoch':snapshot.worldEpoch,'requestId':action_sequence,'clientTimeMs':Time.get_ticks_msec(),'kind':kind}
	if _request('formal_submit_action',{'action':action},'action',{'actionKind':kind})>0 and kind=='guardStart':
		guard_active = true
func _end_guard() -> void:
	pending_actions = pending_actions.filter(func(edge): return edge.kind!='guardStart')
	if guard_active:
		guard_active = false
		_action('guardEnd',true)
func _release() -> void:
	if is_instance_valid(audio):
		audio.suspend_world()
	if guard_active or _key_held(KEY_E) or pending_actions.any(func(edge): return edge.kind=='guardStart'):
		guard_blocked = true
	pending_actions.clear()
	_end_guard()
	_sample(true)
	for key in ACTION_KEYS:
		if _key_held(key):
			blocked_keys[key] = true
func _event_key(event: InputEventKey) -> int:
	if event.physical_keycode!=0:
		return event.physical_keycode
	return event.keycode if event.keycode in [KEY_W,KEY_A,KEY_S,KEY_D,KEY_UP,KEY_DOWN,KEY_LEFT,KEY_RIGHT,KEY_J,KEY_SHIFT,KEY_SPACE,KEY_Q,KEY_E,KEY_R,KEY_F,KEY_T,KEY_M,KEY_ESCAPE] else 0
func _key_held(key: int) -> bool:
	return Input.is_physical_key_pressed(key) or logical_down.has(key)
func _unhandled_input(event: InputEvent) -> void:
	_trace_key(event,'unhandled','received')
	if event is InputEventMouseMotion:
		pointer_seen = true
	if event is InputEventKey:
		var key = _event_key(event)
		if not event.pressed:
			_trace_key(event,'unhandled','release')
			blocked_keys.erase(key)
			if key==KEY_E:
				_end_guard()
			return
		if event.echo:
			_trace_key(event,'unhandled','echo')
			return
		if key==KEY_ESCAPE:
			_trace_key(event,'unhandled','escape')
			_escape()
			get_viewport().set_input_as_handled()
			return
		if not _eligible() or blocked_keys.has(key):
			_trace_key(event,'unhandled','ineligible_or_blocked')
			return
		if key==KEY_F:
			_trace_key(event,'unhandled','dispatch_f')
			_interact()
		elif key==KEY_T:
			_trace_key(event,'unhandled','dispatch_t')
			_open_paused('mission')
		elif key==KEY_M:
			_trace_key(event,'unhandled','dispatch_m')
			_open_paused('map')
		elif ACTION_KEYS.has(key):
			_trace_key(event,'unhandled','dispatch_action')
			_action(ACTION_KEYS[key])
	if event is InputEventMouseButton and event.button_index==MOUSE_BUTTON_LEFT and event.pressed and _eligible() and get_viewport().gui_get_hovered_control()==null:
		pointer_seen = true
		_action('primaryAttack')
func _input(event: InputEvent) -> void:
	_trace_key(event,'input','received')
	if not event is InputEventKey:
		return
	var key = _event_key(event)
	if key==0:
		return
	if event.physical_keycode==0:
		if not event.pressed:
			logical_down.erase(key)
			blocked_keys.erase(key)
		elif event.echo:
			return
		elif not focused or logical_down.has(key):
			_trace_key(event,'input','logical_duplicate_or_unfocused')
			get_viewport().set_input_as_handled()
			return
		else:
			logical_down[key] = true
			if not _eligible() and (ACTION_KEYS.has(key) or key in [KEY_F,KEY_T,KEY_M]):
				blocked_keys[key] = true
	if event.pressed and not event.echo and key==KEY_ESCAPE:
		_trace_key(event,'input','escape')
		_escape()
		get_viewport().set_input_as_handled()
	# Release is observed even when a native control consumed the key-up.
	if not event.pressed and key==KEY_E:
		blocked_keys.erase(KEY_E)
		guard_blocked = false
		_end_guard()
func _notification(what: int) -> void:
	if what==NOTIFICATION_WM_CLOSE_REQUEST and (not is_instance_valid(rpc) or not rpc.authenticated):
		get_tree().quit()
		return
	if not is_instance_valid(rpc):
		return
	if what==NOTIFICATION_APPLICATION_FOCUS_OUT:
		guard_blocked = true
		_release()
		if _eligible():
			_open_paused('pause')
		logical_down.clear()
		focused = false
		audio.stop_all()
	elif what==NOTIFICATION_APPLICATION_FOCUS_IN:
		focused = true
		_release()
	elif what==NOTIFICATION_WM_CLOSE_REQUEST:
		if not critical.is_empty() or not ready_pending.is_empty() or not ticket.is_empty():
			_feedback('请先等待当前确认或关闭对白')
			return
		if journey_started and rpc.authenticated and not uncertain and critical.is_empty():
			_open_paused('exit_confirm')
		else:
			_show('error' if uncertain else 'start')
			panels.set_message('当前没有已确认保存；退出客户端不证明服务已关闭')
func _escape() -> void:
	if not focused or uncertain or not critical.is_empty() or not ready_pending.is_empty() or snapshot.get('entryToken')!=null:
		return
	if panels.visible:
		if panels.mode=='pause':
			_resume()
		elif panels.mode in ['settings','inventory','character','capability','mission','archive','save','slots','map','new_confirm','exit_confirm']:
			_back()
		return
	if _eligible():
		_open_paused('pause')
func _show(page: String, extra: Dictionary = {}, blocked: bool = false, refresh_saves: bool = true) -> void:
	for key in logical_down:
		if ACTION_KEYS.has(key) or key in [KEY_F,KEY_T,KEY_M]:
			blocked_keys[key] = true
	panel_generation += 1
	var o = _owner()
	o.merge(extra,true)
	panels.preferences = preferences.duplicate(true)
	panels.font_scale = preferences.scale
	_publish_saves(false)
	panels.show_page(page,o,snapshot,rpc.authenticated if is_instance_valid(rpc) else false,blocked)
	if refresh_saves and page in ['slots','save'] and not blocked:
		_refresh_saves()
func _open_paused(page: String, page_data: Dictionary = {}) -> void:
	if not journey_started or not snapshot.is_empty() and snapshot.player.currentHp<=0 and page in ['slots','new_confirm','settings']:
		_show(page,page_data)
		return
	if snapshot.is_empty() or status.is_empty() or not P.same(status,snapshot) or not focused or uncertain or not critical.is_empty() or snapshot.get('entryToken')!=null or status.get('entryToken')!=null or snapshot.player.currentHp<=0:
		_feedback('当前不能打开此面板')
		return
	_release()
	if status.paused:
		_show(page,page_data)
	else:
		if not _advance_lifecycle():
			return
		var sequence = lifecycle_sequence
		if _request('formal_pause',{'context':_context(),'commandSequence':sequence},'open_page',{'page':page,'pageData':page_data,'sequence':sequence},true)>0:
			_show('waiting',{},true)
func _resume() -> void:
	if not journey_started or uncertain or not critical.is_empty() or status.is_empty() or not status.paused or not P.same(status,snapshot) or not P.same(pause_owned,_context()) or not ticket.is_empty():
		_feedback('暂停归属未确认；不能恢复')
		return
	if not _advance_lifecycle():
		return
	_request('formal_resume',{'context':_context(),'commandSequence':lifecycle_sequence},'resume',{'sequence':lifecycle_sequence},true)
	panels.set_message('等待恢复确认',true)
func _back() -> void:
	if not ticket.is_empty():
		_close_baizhi()
	elif journey_started:
		if not snapshot.is_empty() and snapshot.player.currentHp<=0:
			_show('death')
			return
		if panels.mode=='enhancement':
			enhancement_pending.clear()
		_show('pause')
	else:
		_show('start')
func _ui_intent(kind: String, value, owner: Dictionary) -> void:
	if panels.visible and owner!=panels.page_owner:
		return
	if not owner.is_empty() and (owner.get('generation')!=generation or not owner.get('context',{}).is_empty() and not P.same(owner.context,_context())):
		return
	if kind=='close_client' and not rpc.authenticated:
		get_tree().quit()
		return
	if close_confirmed or close_attempted and kind!='close_client':
		return
	if kind=='settings_defaults':
		audio.ui_cue('confirm',focused)
	if kind in ['settings_defaults','settings_apply','slot_create_prepare']:
		return
	if kind=='apply_preferences':
		if value is Dictionary and value.get('scale') in [1.0,1.25,1.5] and typeof(value.get('fullscreen'))==TYPE_BOOL and typeof(value.get('color'))==TYPE_BOOL and typeof(value.get('reduced'))==TYPE_BOOL and typeof(value.get('muted'))==TYPE_BOOL and P.number(value.get('volume'),0,1) and value.size()==6:
			preferences = value.duplicate(true)
			audio.configure(preferences.muted,preferences.volume)
			audio.ui_cue('confirm',focused)
			renderer.text_scale = preferences.scale
			renderer.color_assistance = preferences.color
			renderer.reduced_motion = preferences.reduced
			DisplayServer.window_set_mode(DisplayServer.WINDOW_MODE_FULLSCREEN if preferences.fullscreen else DisplayServer.WINDOW_MODE_WINDOWED)
			panels.preferences = preferences.duplicate(true)
			panels.font_scale = preferences.scale
			_show('settings')
			panels.set_message('已应用 · 仅当前会话 · '+audio.status_text)
		return
	if not critical.is_empty() or uncertain:
		if kind=='close_client':
			get_tree().quit()
		return
	if kind in ['back','resume','start','dialogue_close','baizhi_page']:
		audio.ui_cue('cancel',focused)
	elif kind in ['settings','inventory','character','capability','mission','archive','map','pause']:
		audio.ui_cue('tab',focused)
	elif kind in ['new_confirm','exit_confirm','save','new','continue_default','continue_slot','slots','slots_refresh','save_default','save_slot','save_exit','build_item','capability_select','capability_activate','enhancement_choose','baizhi_confirm','baizhi_commit','return','close_client']:
		audio.ui_cue('confirm',focused)
	match kind:
		'traverse_current':
			if not value is Dictionary or value.size()!=5 or not value.has_all(['generation','context','entityId','kind','label']) or not P.counter(value.generation) or value.generation!=generation or not P.context(value.context) or not P.same(value.context,_context()) or not P.ident(value.entityId) or typeof(value.kind)!=TYPE_STRING or value.kind!='contextTraversal' or typeof(value.label)!=TYPE_STRING or not value.entityId in ['cw_conveyor_context_vault','cw_gear_upper_gap','cw_gear_earned_air_step']:
				return
			var current = _current_hud_traversal()
			if current.is_empty() or current.generation!=value.generation or not P.same(current.context,value.context) or current.entityId!=value.entityId or current.kind!=value.kind:
				return
			_action('contextTraversal')
		'interact_current':
			if not value is Dictionary or value.size()!=5 or not value.has_all(['generation','context','entityId','kind','label']) or not P.counter(value.generation) or value.generation!=generation or not P.context(value.context) or not P.same(value.context,_context()) or not P.ident(value.entityId) or typeof(value.kind)!=TYPE_STRING or typeof(value.label)!=TYPE_STRING:
				return
			var current = _current_hud_interaction()
			if current.is_empty() or current.entityId!=value.entityId or current.kind!=value.kind:
				return
			_interact()
		'new_confirm','settings','inventory','character','capability','mission','archive','save','map','pause','exit_confirm': _open_paused(kind)
		'back': _back()
		'resume': _resume()
		'start': _show('start')
		'new': _begin_journey('formal_new',{})
		'continue_default': _begin_journey('formal_continue',{})
		'continue_slot': _begin_journey('formal_continue_slot',{'slotId':value})
		'slots':
			_open_paused('slots')
		'slots_refresh': _refresh_saves()
		'save_default': _save_request('formal_save',{},false)
		'save_exit': _save_request('formal_save',{},true)
		'save_slot': _save_request('formal_save_slot',value,false)
		'return_slot':
			if journey_started and _writable_slot_params(value):
				_request('formal_return_slot',value,'return',{},true)
		'save_slot_new': _save_request('formal_save_slot',{'slotId':'v1-'+str(Time.get_ticks_msec())+'-'+str(request_sequence+1),'displayName':value,'create':true},false)
		'return':
			if journey_started:
				_request('formal_return',{},'return',{},true)
		'build_item': _build_item(value)
		'capability_select': _capability_select(value)
		'capability_activate':
			if P.granted(snapshot,value):
				if action_sequence>=9007199254740990:
					_hold('E_COUNTER_EXHAUSTED')
					return
				action_sequence += 1
				_request('formal_capability_command',{'command':{'kind':'activate','capabilityId':value,'requestId':action_sequence}},'capability',{},true)
		'enhancement_choose': _choose_enhancement(value)
		'baizhi_confirm':
			if not ticket.is_empty() and value in ['taken','left']:
				_show('baizhi_confirm',{'choice':value})
		'baizhi_page': _show('baizhi')
		'baizhi_commit': _commit_baizhi(value)
		'baizhi_unresolved':
			if not ticket.is_empty():
				_show('dialogue',{'text':'那就先别打开那扇门。'})
		'baizhi_close': _close_baizhi()
		'dialogue_close':
			if not ticket.is_empty():
				_close_baizhi()
			else:
				_back()
		'close_client':
			if rpc.authenticated:
				_submit_close()
			else:
				get_tree().quit()
func _begin_journey(method: String, params: Dictionary) -> void:
	if not rpc.authenticated or uncertain or not critical.is_empty() or not ticket.is_empty():
		return
	if method=='formal_continue' and not _default_continue_available() or method=='formal_continue_slot' and not _slot_available(params.get('slotId')):
		_feedback('存档可用性尚未确认；未发送继续请求')
		return
	_release()
	logical_down.clear()
	generation += 1
	_invalidate_saves()
	journey_started = false
	pause_owned.clear()
	enhancement_pending.clear()
	ready_pending.clear()
	ready_sent.clear()
	status.clear()
	lifecycle_sequence = 0
	if _request(method,params,'journey',{},true)>0:
		_show('waiting',{},true)
func _save_request(method: String, params: Dictionary, exit_after: bool) -> void:
	if not journey_started or snapshot.is_empty() or not ticket.is_empty() or snapshot.get('entryToken')!=null:
		return
	if method=='formal_save_slot' and params.get('create')!=true and not _writable_slot_params(params):
		return
	if _request(method,params,'save',{'exitAfter':exit_after},true)>0:
		panels.set_message('保存中 · 等待实际回执',true)
func _build_item(id: String) -> void:
	if not P.build(snapshot.get('build')):
		return
	for item in snapshot.build.items:
		if item.itemId!=id:
			continue
		var expected = null
		for e in snapshot.build.equipment:
			if e.slotId==item.slotId:
				expected = e.itemId
		var action = {'kind':'unequip','slotId':item.slotId,'expectedItemId':id} if item.equipped else {'kind':'equip','itemId':id,'expectedItemId':expected}
		var rid = _id('build')
		_request('formal_build_command',{'command':{'requestId':rid,'worldEpoch':snapshot.worldEpoch,'expectedBuildRevision':snapshot.build.revision,'action':action}},'build',{'expectedCommand':'build:'+rid,'buildRevision':snapshot.build.revision,'buildAction':action},true)
func _capability_select(id: String) -> void:
	var selected: Array = []
	var found = false
	for c in P.capability_items(snapshot):
		if not c.granted:
			continue
		if c.capabilityId==id:
			found = true
			if not c.selected:
				selected.append(id)
		elif c.selected:
			selected.append(c.capabilityId)
	if found:
		_request('formal_capability_command',{'command':{'kind':'select','capabilityIds':selected}},'capability',{},true)
func _choose_enhancement(id: String) -> void:
	if not id in ['information.local_map_i','perception.rear_view_i','body.regeneration_i'] or enhancement_pending.is_empty() or status.is_empty() or not status.paused or not P.same(status,snapshot):
		return
	var actual = status.get('enhancementTerminalContext')
	if not actual is Dictionary or actual.size()!=4 or not P.counter(actual.get('worldEpoch')) or not P.counter(actual.get('pauseCommandSequence'),1) or actual.get('id')!='rs_capability_terminal_marker' or actual.get('requestId')!=enhancement_pending.requestId or actual.get('worldEpoch')!=snapshot.worldEpoch or actual.get('pauseCommandSequence')!=enhancement_pending.sequence:
		_hold('E_ENHANCEMENT_CONTEXT')
		return
	_request('formal_choose_first_enhancement',{'capabilityId':id,'context':actual.duplicate(true)},'enhancement',{'capabilityId':id},true)
func _nearest() -> Dictionary:
	if snapshot.is_empty():
		return {}
	var player = P.vector(snapshot.player.transform.positionM)
	var rows: Array = []
	for i in snapshot.interactables:
		if player.distance_to(P.vector(i.transform.positionM))<=2.5 and renderer.visible_point(P.vector(i.transform.positionM)):
			rows.append(i)
	rows.sort_custom(func(a,b):
		if a.active!=b.active:
			return a.active
		var ad = player.distance_to(P.vector(a.transform.positionM))
		var bd = player.distance_to(P.vector(b.transform.positionM))
		return a.entityId<b.entityId if is_equal_approx(ad,bd) else ad<bd)
	return {} if rows.is_empty() else rows[0]
func _baizhi_available(i: Dictionary) -> bool:
	var b = snapshot.get('baizhi')
	return i.get('kind')=='npc_dialogue' and i.get('entityId')=='gh_bz_first_contact' and snapshot.get('worldId')=='grey_hive' and snapshot.get('sceneId')=='gh_bio_isolation' and typeof(i.get('active'))==TYPE_BOOL and i.active and b is Dictionary and typeof(b.get('canInteract'))==TYPE_BOOL and b.canInteract
func _interaction_prompt(i: Dictionary) -> String:
	var label = _interaction_label(i)
	if typeof(i.get('active'))!=TYPE_BOOL or not i.active:
		return label+' · 当前不可用'
	if i.get('kind')=='npc_dialogue':
		return '[F] '+label if _baizhi_available(i) else label+' · 当前对白不可用'
	if SCENE_RPC.has(i.get('kind')) or i.get('kind') in INTERACT_KINDS:
		return '[F] '+label
	return label if label=='未知交互（不可执行）' else label+' · 未知交互（不可执行）'
func _scene_in_world(scene: String, world: String) -> bool:
	var scenes = {
		'return_station':['rs_core_room'],
		'grey_hive':['gh_entry_maintenance','gh_power_room','gh_gate_a','gh_central_shaft','gh_lockdown','gh_bio_isolation','gh_gate_b','gh_deep_decon','gh_sentinel_arena','gh_beacon','gh_exit'],
		'mist_harbor':['mh_fog_pier','mh_tidal_warehouse','mh_signal_yard','mh_drowned_quay','mh_breakwater','mh_pump_station','mh_resonance_tower','mh_warden_arena','mh_extraction'],
		'clockworks':['cw_entry_foundry','cw_pressure_hall','cw_conveyor_bridge','cw_boiler_chamber','cw_gear_shaft','cw_furnace_heart','cw_forged_guard_arena','cw_regulator_core','cw_shutdown_exit']}
	return scene in scenes.get(world,[])
func _destination_label(i: Dictionary) -> String:
	if snapshot.is_empty() or catalog.current.get('worldId')!=snapshot.worldId or catalog.current.get('sceneId')!=snapshot.sceneId or typeof(i.get('kind'))!=TYPE_STRING or typeof(i.get('entityId'))!=TYPE_STRING:
		return ''
	if i.kind=='world_gate' and snapshot.worldId=='return_station' and snapshot.sceneId=='rs_core_room':
		var worlds = {'rs_world_gate_marker':'grey_hive','rs_mh_world_gate_marker':'mist_harbor','rs_cw_world_gate_marker':'clockworks'}
		if not worlds.has(i.entityId):
			return ''
		var interactions = catalog.current.get('interactions')
		if not P.rows(interactions,2048):
			return ''
		var matches: Array = []
		for row in interactions:
			if row is Dictionary and typeof(row.get('id'))==TYPE_STRING and row.id==i.entityId:
				matches.append(row)
		if matches.size()==1 and typeof(matches[0].get('kind'))==TYPE_STRING and matches[0].kind=='world_gate_marker':
			return '前往'+Content.WORLD_NAMES[worlds[i.entityId]]
	elif i.kind=='scene_transition':
		var transitions = catalog.current.get('transitions')
		if not P.rows(transitions,2048):
			return ''
		var matches: Array = []
		for row in transitions:
			if row is Dictionary and typeof(row.get('id'))==TYPE_STRING and row.id==i.entityId:
				matches.append(row)
		if matches.size()!=1:
			return ''
		var destination = matches[0].get('toSceneId')
		if typeof(destination)!=TYPE_STRING or not Content.SCENE_NAMES.has(destination) or not _scene_in_world(destination,snapshot.worldId):
			return ''
		var pin = catalog.manifest.get(destination)
		if pin is Dictionary and typeof(pin.get('path'))==TYPE_STRING and pin.path=='res://data/scenes/'+destination+'.json':
			return '前往'+Content.SCENE_NAMES[destination]
	return ''
func _current_hud_interaction() -> Dictionary:
	if not P.snapshot(snapshot) or not P.context(_context()) or not _eligible():
		return {}
	var i = _nearest()
	if i.is_empty() or typeof(i.get('active'))!=TYPE_BOOL or not i.active:
		return {}
	if i.kind=='npc_dialogue':
		if not _baizhi_available(i):
			return {}
	elif not SCENE_RPC.has(i.kind) and not i.kind in INTERACT_KINDS:
		return {}
	return {'generation':generation,'context':_context().duplicate(true),'entityId':i.entityId,'kind':i.kind,'label':_interaction_label(i)}
func _interaction_label(i: Dictionary) -> String:
	var destination = _destination_label(i)
	if not destination.is_empty():
		return destination
	return {'world_gate':'世界入口','mission_terminal':'任务终端','capability_terminal':'能力终端','save_rest_terminal':'休息 / 保存','npc_dialogue':'与白芷交谈','scene_transition':'前往下一场景','scene_checkpoint':'检查点','scene_trigger':'场景装置','environment_control':'环境控制','power_console':'恢复电力','door_panel':'门控制','lockdown_terminal':'隔离协议','facility_log':'设施记录','bio_log_terminal':'生物记录','gate_b_panel':'Gate B 控制','extraction_console':'撤离控制','beacon_collect':'收取信标','beacon_mount':'部署信标','beacon':'信标','pump_control':'泵站控制','valve_control':'阀门','coolant_valve':'冷却阀','static_dialogue_marker':'查看炉心状态','terminal':'终端'}.get(i.kind,'未知交互（不可执行）')
func _interact() -> void:
	var i = _nearest()
	if not _eligible() or i.is_empty():
		_feedback('没有范围内可交互对象')
		return
	if not i.active:
		_feedback('目标已锁定或当前不可用；未发送命令')
		return
	var kind: String = i.kind
	var id: String = i.entityId
	if kind=='npc_dialogue':
		if id!='gh_bz_first_contact' or snapshot.worldId!='grey_hive' or snapshot.sceneId!='gh_bio_isolation' or not _baizhi_available(i):
			_feedback('当前对白不可用')
			return
		_begin_baizhi()
		return
	if kind=='world_gate' and id in ['rs_world_gate_marker','rs_mh_world_gate_marker','rs_cw_world_gate_marker']:
		if snapshot.worldId!='return_station' or snapshot.sceneId!='rs_core_room':
			_feedback('入口来源不匹配')
			return
		id = {'rs_world_gate_marker':'rs_world_gate_to_gh','rs_mh_world_gate_marker':'rs_world_gate_to_mh','rs_cw_world_gate_marker':'rs_world_gate_to_cw'}[id]
	if id=='cw_shutdown_return_to_rs' and (snapshot.worldId!='clockworks' or snapshot.sceneId!='cw_shutdown_exit') or id=='mh_extraction_return_to_rs' and (snapshot.worldId!='mist_harbor' or snapshot.sceneId!='mh_extraction'):
		_feedback('返回入口来源不匹配')
		return
	if kind in ['beacon_collect','beacon_mount'] and (snapshot.worldId!='grey_hive' or snapshot.sceneId!='gh_beacon' or i.entityId!=('gh_beacon_deploy_marker' if kind=='beacon_collect' else 'gh_beacon_storage_mount_marker')):
		_feedback('信标来源不匹配')
		return
	if kind=='coolant_valve' and (id!='cw_regulator_valve_furnace_link_staged' or snapshot.worldId!='clockworks' or snapshot.sceneId!='cw_regulator_core'):
		_feedback('冷却阀来源不匹配')
		return
	if kind=='static_dialogue_marker' and (id!='cw_cy_heat_01_static_dialogue_marker' or snapshot.worldId!='clockworks' or snapshot.sceneId!='cw_furnace_heart'):
		_feedback('未知对白来源')
		return
	var rid = _id('scene')
	var method = SCENE_RPC.get(kind,'formal_interact' if kind in INTERACT_KINDS else '')
	if method.is_empty():
		_feedback('未知交互类型；未发送命令')
		return
	var params = {'id':id,'requestId':rid,'worldEpoch':snapshot.worldEpoch}
	if method=='formal_interact':
		params.erase('id')
		params['actorId'] = id
	_request(method,params,'interaction',{'expectedCommand':rid,'target':i.duplicate(true),'before':snapshot.duplicate(true)},true)
func _begin_baizhi() -> void:
	if status.is_empty() or not P.same(status,snapshot):
		return
	if not _advance_lifecycle():
		return
	var owner_id = 'v1-godot-'+str(generation)+'-'+str(panel_generation+1)
	var request = {'context':_context(),'entityId':'gh_bz_whitezhi_v1','interactionId':'gh_bz_first_contact','ownerId':owner_id,'generation':panel_generation+1,'commandSequence':lifecycle_sequence}
	_request('formal_baizhi_begin',{'request':request},'baizhi_begin',{'ticketOwner':owner_id,'ticketGeneration':panel_generation+1,'sequence':lifecycle_sequence},true)
	_show('waiting',{},true)
func _commit_baizhi(choice: String) -> void:
	if ticket.is_empty() or not P.ticket(ticket,_context()) or not choice in ['taken','left']:
		return
	var rid = _id('baizhi')
	_request('formal_baizhi_commit',{'ticket':ticket.duplicate(true),'requestId':rid,'choice':choice},'baizhi_commit',{'expectedCommand':rid,'choice':choice},true)
	panels.set_message('等待白芷选择确认',true)
func _close_baizhi() -> void:
	if ticket.is_empty() or not P.ticket(ticket,_context()):
		return
	_request('formal_baizhi_close',{'ticket':ticket.duplicate(true)},'baizhi_close',{'expectedCommand':'baizhi-close:'+ticket.ownerId+':'+str(int(ticket.generation))},true)
	panels.set_message('等待对白关闭确认',true)
func _response(_rpc_id: int, method: String, result, association: Dictionary) -> void:
	if close_attempted and method in Rpc.READS:
		return
	if close_confirmed:
		return
	if method in ['formal_has_save','formal_list_save_slots']:
		_save_read_response(_rpc_id,method,result,association)
		return
	var purpose: String = association.get('purpose','')
	var transition = purpose in ['journey','return','interaction']
	if not _current(association,transition):
		return
	if purpose=='poll':
		if not P.snapshot(result):
			_hold('E_REQUIRED_SNAPSHOT')
			return
		_accept_snapshot(result)
		return
	if purpose=='status':
		if not P.status(result):
			status.clear()
			if journey_started:
				_hold('E_SESSION_STATUS')
			return
		if snapshot.is_empty() or not P.same(result,snapshot):
			return
		if result.lastLifecycleCommandSequence<lifecycle_sequence:
			return
		status = result
		if not _eligible():
			audio.suspend_world()
		last_status_at = Time.get_ticks_msec()
		lifecycle_sequence = maxi(lifecycle_sequence,int(status.lastLifecycleCommandSequence))
		var t = result.get('entryToken')
		if t!=null:
			if not P.same(t,snapshot):
				_hold('E_ENTRY_STATUS_IDENTITY')
				return
			_prepare_entry(t)
		if not enhancement_pending.is_empty() and status.paused and not enhancement_pending.get('shown',false):
			var e = status.get('enhancementTerminalContext')
			if e is Dictionary and e.get('id')=='rs_capability_terminal_marker' and e.get('requestId')==enhancement_pending.requestId and e.get('worldEpoch')==snapshot.worldEpoch and e.get('pauseCommandSequence')==enhancement_pending.sequence:
				enhancement_pending['shown'] = true
				_show('enhancement')
		return
	if purpose=='events':
		if not snapshot.is_empty():
			event_view.accept(result,snapshot,renderer)
			audio.accept_events(result,snapshot,generation,_eligible())
		return
	if purpose=='sound':
		if not snapshot.is_empty():
			event_view.accept_sound(result,snapshot)
		return
	if purpose=='close':
		if not _close_current(_rpc_id,method,association):
			return
		close_pending.clear()
		if result is Dictionary and typeof(result.get('closing'))==TYPE_BOOL and result.closing:
			close_confirmed = true
			get_tree().quit()
		else:
			_hold('E_CLOSE_RESPONSE')
		return
	if close_attempted and purpose in ['poll','status','events','sound']:
		return
	var r = result.get('receipt') if result is Dictionary and result.has('receipt') else result
	if not P.receipt(r):
		_hold('E_RECEIPT_INVALID')
		return
	if association.has('expectedCommand') and r.commandId!=association.expectedCommand:
		_hold('E_RECEIPT_REQUEST_MISMATCH')
		return
	if method=='formal_interact' and (not result is Dictionary or result.get('applied')!=r.applied or result.get('alreadyApplied')!=r.alreadyApplied or result.get('errorCode')!=r.get('errorCode')):
		_hold('E_INTERACTION_RESPONSE')
		return
	if purpose in ['open_page','resume','enhancement_pause']:
		if r.commandId!=('resume' if purpose=='resume' else 'pause') or not P.same(r.snapshot,association.context):
			_hold('E_LIFECYCLE_CONTEXT')
			return
	if purpose=='ready':
		var t: Dictionary = association.token
		if r.commandId!='scene-ready:'+str(int(t.generation)) or not P.same(r.snapshot,t) or r.snapshot.get('entryToken')!=null:
			_hold('E_SCENE_READY_RECEIPT')
			return
	if purpose in ['input','action','build','capability','enhancement','save','baizhi_begin','baizhi_commit','baizhi_close'] and not association.context.is_empty() and not P.same(r.snapshot,association.context):
		_hold('E_RECEIPT_STALE_CONTEXT')
		return
	if purpose=='build' and P.success(r):
		if not P.build(r.snapshot.get('build')) or r.snapshot.build.revision<=association.buildRevision:
			_hold('E_BUILD_POSTCONDITION')
			return
		var action: Dictionary = association.buildAction
		var slot = action.get('slotId','')
		if action.kind=='equip':
			for i in r.snapshot.build.items:
				if i.itemId==action.itemId:
					slot = i.slotId
		var equipped = ''
		for e in r.snapshot.build.equipment:
			if e.slotId==slot:
				equipped = e.itemId
		if slot.is_empty() or action.kind=='equip' and equipped!=action.itemId or action.kind=='unequip' and not equipped.is_empty():
			_hold('E_BUILD_POSTCONDITION')
			return
	if critical==method:
		critical = ''
	if not P.success(r):
		if purpose=='ready':
			_hold('E_SCENE_READY_REJECTED')
			return
		_accept_snapshot(r.snapshot)
		audio.ui_cue('error',focused)
		_feedback(_localized(str(r.get('errorCode','E_COMMAND_REJECTED'))))
		if purpose=='open_page':
			panels.close_page()
			return
		if panels.visible:
			var page = 'exit_confirm' if association.get('exitAfter',false) else 'start' if purpose=='journey' else 'pause' if purpose=='resume' else panels.mode
			_show(page)
			panels.set_message(last_feedback)
		return
	if purpose=='journey':
		if r.commandId!=('new' if method=='formal_new' else 'continue') and method!='formal_continue_slot':
			_hold('E_JOURNEY_RECEIPT')
			return
		journey_started = true
		panels.close_page()
	if not _accept_snapshot(r.snapshot,transition):
		return
	match purpose:
		'open_page','enhancement_pause':
			pause_owned = _context()
			status = _context()
			status.merge({'paused':true,'lastLifecycleCommandSequence':association.sequence,'entryToken':null,'enhancementTerminalContext':null})
			if purpose=='enhancement_pause':
				enhancement_pending.sequence = association.sequence
				_show('waiting',{},true)
				_request('bridge_session_status',{},'status')
			else:
				_show(association.page,association.get('pageData',{}))
		'resume':
			pause_owned.clear()
			status = _context()
			status.merge({'paused':false,'lastLifecycleCommandSequence':association.sequence,'entryToken':null,'enhancementTerminalContext':null})
			panels.close_page()
			_release()
		'ready':
			ready_pending.clear()
			status.clear()
			_request('bridge_session_status',{},'status')
			if association.get('remainPaused',false):
				pause_owned = _context()
				_show('pause')
		'save':
			_feedback('已收到保存成功回执；不代表断电耐久性验收')
			if association.get('exitAfter',false):
				_submit_close()
			else:
				_show('save')
				panels.set_message('保存成功')
		'build': _show('inventory')
		'capability': _show('capability')
		'enhancement':
			if snapshot.capabilities.get('firstEnhancementChoice')!=association.capabilityId:
				_hold('E_ENHANCEMENT_POSTCONDITION')
				return
			enhancement_pending.clear()
			_show('pause')
			_feedback('首通选择已确认')
		'baizhi_begin':
			var t = result.get('ticket')
			if not P.ticket(t,_context()) or t.ownerId!=association.ticketOwner or t.generation!=association.ticketGeneration or t.pauseCommandSequence!=association.sequence or r.commandId!='baizhi-begin:'+t.ownerId+':'+str(int(t.generation)) or typeof(result.get('resumed'))!=TYPE_BOOL or result.resumed:
				_hold('E_BAIZHI_TICKET')
				return
			ticket = t
			pause_owned = _context()
			status = _context()
			status.merge({'paused':true,'lastLifecycleCommandSequence':association.sequence,'entryToken':null,'enhancementTerminalContext':null})
			_show('baizhi')
		'baizhi_commit':
			var baizhi = snapshot.get('baizhi')
			if not baizhi is Dictionary or typeof(baizhi.get('choice'))!=TYPE_STRING or typeof(result.get('resumed'))!=TYPE_BOOL or result.resumed or baizhi.choice!=association.choice or not P.ticket(result.get('ticket'),_context()) or result.ticket!=ticket:
				_hold('E_BAIZHI_POSTCONDITION')
				return
			_show('dialogue',{'text':'好，我会等你找到安全的出口。' if association.choice=='taken' else '我明白。'})
		'baizhi_close':
			if typeof(result.get('resumed'))!=TYPE_BOOL or not result.resumed:
				_hold('E_BAIZHI_CLOSE_UNCONFIRMED')
				return
			ticket.clear()
			pause_owned.clear()
			status.clear()
			panels.close_page()
			_request('bridge_session_status',{},'status')
		'interaction': _interaction_result(association,r)
func _accept_snapshot(s: Dictionary, permit_transition: bool = false) -> bool:
	if not P.snapshot(s) or not catalog.manifest.has(s.sceneId):
		_hold('E_REQUIRED_SNAPSHOT')
		return false
	var changed = snapshot.is_empty() or not P.same(snapshot,s)
	if not snapshot.is_empty():
		if snapshot.player.currentHp<=0 and not permit_transition:
			if P.same(snapshot,s) and s.player.currentHp>0:
				_hold('E_DEAD_REVIVAL')
			return false
		if s.worldEpoch<snapshot.worldEpoch:
			return false
		if not changed and (s.serverTick<snapshot.serverTick or s.authorityRevision<snapshot.authorityRevision or s.ackSeq<snapshot.ackSeq):
			return false
		if changed and not permit_transition and s.get('entryToken')==null:
			_hold('E_UNASSOCIATED_EPOCH')
			return false
	if changed:
		audio.suspend_world()
		# The core transition already neutralized the old epoch. Do not send stale input.
		guard_active = false
		guard_blocked = _key_held(KEY_E)
		pending_actions.clear()
		neutral_pending = false
		for key in ACTION_KEYS:
			if _key_held(key):
				blocked_keys[key] = true
		logical_down.clear()
		if not catalog.build(s.worldId,s.sceneId):
			_hold('E_SCENE_RESOURCE')
			return false
		ThemeModel.apply(hud,s.worldId)
		ThemeModel.apply(panels,s.worldId)
		status.clear()
		pause_owned.clear()
		ticket.clear()
		enhancement_pending.clear()
		ready_pending.clear()
		frame_key = ''
		event_view.reset(int(s.worldEpoch))
		if journey_started:
			panels.close_page()
	if not catalog.contains(P.vector(s.player.transform.positionM)):
		_hold('E_PLAYER_PROJECTION_BOUNDS')
		return false
	snapshot = s
	last_snapshot_at = Time.get_ticks_msec()
	renderer.commit(s,catalog.current,catalog.source_hash)
	if journey_started and s.get('entryToken')!=null:
		_prepare_entry(s.entryToken)
	if journey_started and s.player.currentHp<=0 and critical.is_empty():
		_release()
		if panels.mode!='death':
			_show('death')
	return true
func _prepare_entry(t: Dictionary) -> void:
	if not journey_started or not P.token(t) or not P.same(t,snapshot) or not critical.is_empty() or uncertain:
		return
	var key = snapshot.worldId+':'+snapshot.sceneId+':'+str(snapshot.worldEpoch)
	var token_key = str([t.generation,t.worldId,t.sceneId,t.worldEpoch])
	if ready_sent.has(token_key):
		return
	ready_pending = t.duplicate(true)
	_release()
	if frame_key==key:
		_scene_frame_committed(key)
func _scene_frame_committed(key: String) -> void:
	frame_key = key
	if ready_pending.is_empty() or not P.same(ready_pending,snapshot) or uncertain or not critical.is_empty():
		return
	var expected = snapshot.worldId+':'+snapshot.sceneId+':'+str(snapshot.worldEpoch)
	if key!=expected or catalog.current.get('sceneId')!=snapshot.sceneId or catalog.current.get('worldId')!=snapshot.worldId:
		return
	var t = ready_pending.duplicate(true)
	var token_key = str([t.generation,t.worldId,t.sceneId,t.worldEpoch])
	if ready_sent.has(token_key):
		return
	var remain_paused = not focused
	if _request('formal_scene_ready',{'token':t,'remainPaused':remain_paused},'ready',{'token':t,'remainPaused':remain_paused},true)>0:
		ready_sent[token_key] = true
func _interaction_result(a: Dictionary, r: Dictionary) -> void:
	var target: Dictionary = a.target
	var before: Dictionary = a.before
	var kind: String = target.kind
	var narrative = Content.narrative(before,target,r)
	_feedback(narrative if not narrative.is_empty() else '交互已确认')
	if kind=='mission_terminal':
		_open_paused('mission')
	elif kind=='save_rest_terminal':
		_open_paused('save')
	elif kind=='capability_terminal':
		var clear = false
		for row in snapshot.progression.get('worlds',[]):
			if row is Dictionary and row.get('worldId')=='grey_hive' and row.get('completed')==true:
				clear = true
		if clear and snapshot.capabilities.get('firstEnhancementChoice')==null and snapshot.worldId=='return_station' and snapshot.sceneId=='rs_core_room':
			if status.is_empty() or not P.same(status,snapshot):
				_feedback('终端已确认；等待当前生命周期状态后再交互')
				return
			enhancement_pending = {'requestId':a.expectedCommand,'sequence':0}
			if not _advance_lifecycle():
				return
			_request('formal_pause',{'context':_context(),'commandSequence':lifecycle_sequence},'enhancement_pause',{'sequence':lifecycle_sequence},true)
		else:
			_open_paused('capability')
	elif kind=='static_dialogue_marker' and target.entityId=='cw_cy_heat_01_static_dialogue_marker' and r.applied and not r.alreadyApplied and P.same(before,snapshot) and snapshot.authorityRevision>before.authorityRevision:
		_open_paused('dialogue',{'text':'炉心不是失控——它被维持在过载边缘。'})
	elif target.entityId=='cw_master_shutdown_staged' and r.applied and not r.alreadyApplied and _clockworks_completed(snapshot) and not _clockworks_completed(before):
		_feedback('原来门一直不止三扇。')

func _clockworks_completed(s: Dictionary) -> bool:
	if not P.granted(s,'mobility.air_step_i'):
		return false
	var matches: Array = []
	for row in s.progression.get('worlds',[]):
		if row is Dictionary and row.get('worldId')=='clockworks':
			matches.append(row)
	if matches.size()!=1:
		return false
	var row: Dictionary = matches[0]
	return row.get('completed')==true and row.get('firstCompletion')==true and P.counter(row.get('revisitCount'),0,4) and row.get('visitId')==row.revisitCount+1 and row.get('completedEvents') is Array and row.completedEvents.size()==3 and 'clockworks_valves' in row.completedEvents and 'clockworks_core' in row.completedEvents and 'clockworks_shutdown' in row.completedEvents
func _invalidate_saves() -> void:
	save_reads.clear()
	default_save_state = 'unknown'
	slot_list_state = 'unknown'
	current_slots.clear()
	save_projection_generation = generation
	save_projection_connection = rpc.connection_generation if is_instance_valid(rpc) else -1
	_publish_saves(false)
func _save_scope_current() -> bool:
	return is_instance_valid(rpc) and rpc.authenticated and save_projection_generation==generation and save_projection_connection==rpc.connection_generation
func _default_continue_available() -> bool:
	return _save_scope_current() and default_save_state=='available'
func _slot_available(id, writable: bool = false) -> bool:
	if not _save_scope_current() or slot_list_state!='list' or not P.ident(id):
		return false
	for row in current_slots:
		if row.slotId==id:
			return row.valid and (not writable or not row.readOnly)
	return false
func _writable_slot_params(value) -> bool:
	if not value is Dictionary or value.get('create')!=false or not _slot_available(value.get('slotId'),true):
		return false
	for row in current_slots:
		if row.slotId==value.slotId:
			return value.get('displayName')==row.displayName
	return false
func _publish_saves(rebuild: bool = true) -> void:
	if not is_instance_valid(panels):
		return
	var current = _save_scope_current()
	panels.set_save_availability(default_save_state if current else 'unknown',slot_list_state if current else 'unknown',current_slots if current else [])
	if rebuild and panels.visible and panels.mode in ['slots','save'] and critical.is_empty() and not uncertain and not panels.pending:
		var page: String = panels.mode
		var text = panels.status.text if is_instance_valid(panels.status) else ''
		_show(page,{},false,false)
		panels.set_message(text)
func _set_save_read_state(method: String, state: String) -> void:
	if method=='formal_has_save':
		default_save_state = state
	else:
		slot_list_state = state
		current_slots.clear()
func _refresh_saves() -> void:
	_save_read('formal_has_save',{'defaultOnly':true})
	_save_read('formal_list_save_slots',{})
func _save_read(method: String, params: Dictionary) -> void:
	if close_attempted:
		return
	if not rpc.authenticated:
		return
	var owned = save_reads.get(method,{})
	if not owned.is_empty() and _save_read_current(owned.id,method,owned) and rpc.pending.has(owned.id):
		# The transport will coalesce this read; retain its sole response token.
		return
	save_reads.erase(method)
	if save_read_sequence>=9007199254740991:
		_set_save_read_state(method,'error')
		_publish_saves()
		return
	save_read_sequence += 1
	var a = _assoc({'purpose':'save_availability','readSequence':save_read_sequence,'connectionGeneration':rpc.connection_generation,'method':method})
	var id = rpc.request(method,params,a)
	# request may synchronously drop the connection and invalidate all projections.
	if not rpc.authenticated or a.connectionGeneration!=rpc.connection_generation or a.generation!=generation:
		return
	save_projection_generation = generation
	save_projection_connection = rpc.connection_generation
	if id<=0:
		_set_save_read_state(method,'unknown')
	else:
		a['id'] = id
		save_reads[method] = a.duplicate(true)
		_set_save_read_state(method,'pending')
	_publish_saves()
func _save_read_current(id: int, method: String, a: Dictionary) -> bool:
	var owned = save_reads.get(method,{})
	return rpc.authenticated and not owned.is_empty() and owned.id==id and owned.method==method and a.get('method')==method and a.get('readSequence')==owned.readSequence and a.get('connectionGeneration')==rpc.connection_generation and owned.connectionGeneration==rpc.connection_generation and _current(a) and _current(owned)
func _save_read_response(id: int, method: String, result, a: Dictionary) -> void:
	if not _save_read_current(id,method,a):
		return
	save_reads.erase(method)
	if method=='formal_has_save':
		default_save_state = ('available' if result else 'unavailable') if typeof(result)==TYPE_BOOL else 'invalid'
	else:
		current_slots.clear()
		if _valid_slots(result):
			current_slots = result.duplicate(true)
			slot_list_state = 'empty' if current_slots.is_empty() else 'list'
		else:
			slot_list_state = 'invalid'
	_publish_saves()
func _valid_slots(value) -> bool:
	if not P.rows(value,128) or not P.unique(value,'slotId'):
		return false
	for slot in value:
		if typeof(slot.get('displayName'))!=TYPE_STRING or slot.displayName.length()>256 or not P.counter(slot.get('updatedAtMs')) or typeof(slot.get('readOnly'))!=TYPE_BOOL or typeof(slot.get('valid'))!=TYPE_BOOL or typeof(slot.get('recoverable',false))!=TYPE_BOOL or not P.rows(slot.get('completedEvents'),64) or slot.get('errorCode')!=null and typeof(slot.errorCode)!=TYPE_STRING:
			return false
	return true
func _failed(_id_value: int, method: String, code: String, association: Dictionary) -> void:
	if close_confirmed:
		return
	if method=='session_close':
		if not _close_current(_id_value,method,association):
			return
		close_pending.clear()
	elif close_attempted and method in Rpc.READS:
		return
	if method in ['formal_has_save','formal_list_save_slots']:
		if not _save_read_current(_id_value,method,association):
			return
		save_reads.erase(method)
		_set_save_read_state(method,'error')
		_publish_saves()
		return
	if not _current(association,association.get('purpose') in ['journey','return','interaction']):
		return
	audio.ui_cue('error',focused)
	if method=='formal_presentation_events':
		audio.suspend_world()
	if method=='bridge_session_status':
		status.clear()
		_release()
	if method in ['formal_snapshot','bridge_session_status','formal_presentation_events','formal_sound_cues','formal_has_save','formal_list_save_slots']:
		_feedback(_localized(code))
		return
	if code=='UNKNOWN_MUTATION':
		uncertain = true
		critical = ''
		_release()
		_show('error')
		panels.set_message('响应丢失：操作结果未知。未重复提交。')
		return
	if critical==method:
		critical = ''
	_feedback(_localized(code))
	if panels.visible:
		_show(panels.mode)
		panels.set_message(last_feedback)
func _hold(code: String) -> void:
	audio.ui_cue('error',focused)
	uncertain = true
	critical = ''
	_release()
	_show('error')
	panels.set_message(_localized(code))
func _feedback(text: String) -> void:
	last_feedback = text
func _localized(code: String) -> String:
	if code=='UNKNOWN_MUTATION':
		return '操作结果未知'
	if 'OUT_OF_RANGE' in code or 'RANGE' in code:
		return '距离不足；请靠近当前目标'
	if 'LOCKED' in code or 'PROGRESSION' in code or 'PREREQUISITE' in code:
		return '条件尚未满足 / 入口锁定'
	if 'STALE' in code or 'EPOCH' in code or 'CONTEXT' in code:
		return '状态已变化；旧请求未应用'
	if 'COOLDOWN' in code:
		return '动作仍在冷却'
	if 'ENERGY' in code:
		return '能量不足'
	if 'DEAD' in code:
		return '角色已死亡'
	if 'SAVE' in code:
		return '存档操作未确认；请保留当前会话'
	if code.is_empty():
		return ''
	return '当前操作不可用 / 状态未确认'

func _advance_lifecycle() -> bool:
	if status.is_empty() or not P.counter(status.get('lastLifecycleCommandSequence')):
		_feedback('生命周期序号尚未确认')
		return false
	var previous = maxi(lifecycle_sequence,int(status.lastLifecycleCommandSequence))
	if previous>=9007199254740991:
		_hold('E_COUNTER_EXHAUSTED')
		return false
	lifecycle_sequence = previous+1
	return true

func _exit_tree() -> void:
	if is_instance_valid(audio):
		audio.stop_all()

func _close_current(id: int, method: String, association: Dictionary) -> bool:
	return method=='session_close' and not close_pending.is_empty() and id==close_pending.id and close_pending.generation==generation and close_pending.connectionGeneration==rpc.connection_generation and association.get('connectionGeneration')==close_pending.connectionGeneration and association.get('purpose')=='close' and _current(association) and _current(close_pending)
func _submit_close() -> void:
	if close_attempted or not rpc.authenticated or uncertain or not critical.is_empty():
		return
	# Release existing Guard/input/audio work before the once-only close barrier.
	_release()
	close_attempted = true
	close_submitting = true
	var connection = rpc.connection_generation
	var journey = generation
	var context = _context().duplicate(true)
	var id = _request('session_close',{},'close',{'connectionGeneration':connection},true)
	close_submitting = false
	if id>0 and rpc.authenticated and rpc.connection_generation==connection and generation==journey and rpc.pending.has(id):
		close_pending = {'id':id,'generation':journey,'connectionGeneration':connection,'context':context}
		_show('waiting',{},true)
	elif rpc.authenticated:
		_hold('E_CLOSE_SUBMISSION')
	# A synchronous drop already published its usable disconnected error page.

func _trace_start() -> void:
	trace_enabled = OS.get_environment('WUXIAN_V1_INPUT_TRACE')=='1'
	if not trace_enabled:
		return
	trace_started_ms = Time.get_ticks_msec()
	trace_next_ms = trace_started_ms
	_trace_write('start','','',0)
func _trace_live(now: int) -> bool:
	if now-trace_started_ms>=120000 or trace_records>=223:
		_trace_stop('duration' if now-trace_started_ms>=120000 else 'records',now)
		return false
	return true
func _trace_stop(reason: String, now: int) -> void:
	trace_enabled = false
	_trace_write('stopped','',reason,mini(120000,maxi(0,now-trace_started_ms)))
func _trace_tick() -> void:
	if not trace_enabled:
		return
	var now = Time.get_ticks_msec()
	if not _trace_live(now) or now<trace_next_ms or trace_heartbeats>=121:
		return
	trace_next_ms = now+1000
	trace_heartbeats += 1
	_trace_write('state','','',maxi(0,now-trace_started_ms))
func _trace_key(event: InputEvent, stage: String, tag: String) -> void:
	if not trace_enabled:
		return
	var now = Time.get_ticks_msec()
	if not _trace_live(now) or trace_observations>=96 or not event is InputEventKey:
		return
	var resolved = _event_key(event)
	if not event.physical_keycode in TRACE_KEYS and not event.keycode in TRACE_KEYS:
		return
	trace_observations += 1
	if tag=='received':
		if stage=='input':
			trace_input_count += 1
		else:
			trace_unhandled_count += 1
	_trace_write('key',stage,tag,maxi(0,now-trace_started_ms),event)
func _trace_write(kind: String, stage: String, tag: String, elapsed: int, event: InputEventKey = null) -> void:
	var window = get_window()
	var viewport = get_viewport()
	var nodes = is_instance_valid(rpc) and is_instance_valid(panels)
	var valid_snapshot = P.snapshot(snapshot)
	var valid_status = P.status(status)
	var observed_eligible = null
	if nodes and (snapshot.is_empty() or valid_snapshot) and (status.is_empty() or valid_status):
		observed_eligible = _eligible()
	var row = {
		'kind':kind,'stage':stage,'tag':tag,'seq':trace_records+1,'elapsed_ms':elapsed,
		'input_count':trace_input_count,'unhandled_count':trace_unhandled_count,
		'physical':0,'logical':0,'resolved':0,'pressed':null,'echo':null,
		'window_focus':window.has_focus() if is_instance_valid(window) else null,
		'gui_focus':is_instance_valid(viewport) and viewport.gui_get_focus_owner()!=null,
		'eligible':observed_eligible,'close_attempted':close_attempted,'close_confirmed':close_confirmed,
		'journey_started':journey_started,'authenticated':rpc.authenticated if is_instance_valid(rpc) else false,
		'focused':focused,'uncertain':uncertain,'critical':not critical.is_empty(),
		'panels_visible':panels.visible if is_instance_valid(panels) else false,
		'snapshot_present':not snapshot.is_empty(),'status_present':not status.is_empty(),
		'same_context':valid_snapshot and valid_status and P.same(snapshot,status),
		'paused':status.paused if valid_status and typeof(status.get('paused'))==TYPE_BOOL else null,
		'snapshot_entry':snapshot.get('entryToken')!=null,'status_entry':status.get('entryToken')!=null,
		'ready_pending':not ready_pending.is_empty(),'player_alive':valid_snapshot and snapshot.player.currentHp>0,
		'snapshot_fresh':valid_snapshot and Time.get_ticks_msec()-last_snapshot_at<1000,
		'status_fresh':valid_status and Time.get_ticks_msec()-last_status_at<1000,
		'blocked_key':false}
	if event!=null:
		row.physical = event.physical_keycode if event.physical_keycode in TRACE_KEYS else 0
		row.logical = event.keycode if event.keycode in TRACE_KEYS else 0
		var resolved = _event_key(event)
		row.resolved = resolved if resolved in TRACE_KEYS else 0
		row.pressed = event.pressed
		row.echo = event.echo
		row.blocked_key = blocked_keys.has(resolved)
		if tag=='ineligible_or_blocked':
			row.tag = 'ineligible' if observed_eligible==false else 'blocked'
	var encoded = JSON.stringify(row)
	if encoded.to_utf8_buffer().size()>2048 or trace_records>=224:
		trace_enabled = false
		return
	trace_records += 1
	print('WUXIAN_INPUT_TRACE:'+encoded)

func _free_traversal_row_valid(row, expected: Dictionary) -> bool:
	if not row is Dictionary or row.size()!=expected.size() or not row.has_all(expected.keys()) or typeof(row.get('id'))!=TYPE_STRING or row.id!=expected.id or not P.number(row.get('rangeM'),1.2,1.2) or not P.counter(row.get('cooldownMs'),350,350) or not row.get('requiredCapabilities') is Array or row.requiredCapabilities.size()!=expected.requiredCapabilities.size():
		return false
	for index in range(expected.requiredCapabilities.size()):
		if typeof(row.requiredCapabilities[index])!=TYPE_STRING or row.requiredCapabilities[index]!=expected.requiredCapabilities[index]:
			return false
	for field in ['from','to','fromHeightRangeM']:
		if not expected.has(field):
			continue
		var values = row.get(field)
		if not values is Array or values.size()!=expected[field].size():
			return false
		for index in range(values.size()):
			if not P.number(values[index],expected[field][index],expected[field][index]):
				return false
	return true
func _current_hud_traversal() -> Dictionary:
	if not P.snapshot(snapshot) or not P.context(_context()) or not _eligible() or snapshot.worldId!='clockworks' or not FREE_TRAVERSAL_STARTS.has(snapshot.sceneId):
		return {}
	if catalog.current.get('worldId')!=snapshot.worldId or catalog.current.get('sceneId')!=snapshot.sceneId:
		return {}
	var logic = catalog.current.get('logic')
	if not logic is Dictionary:
		return {}
	var rows = logic.get('traversal')
	if not P.rows(rows,128) or not P.unique(rows,'id'):
		return {}
	if snapshot.sceneId=='cw_gear_shaft' and P.granted(snapshot,'mobility.air_step_i'):
		var earned_position = snapshot.player.transform.positionM
		if P.point(earned_position) and earned_position.yM>=0.0 and earned_position.yM<=0.1:
			var earned_dx = earned_position.xM-11.0
			var earned_dz = earned_position.zM-8.0
			if earned_dx*earned_dx+earned_dz*earned_dz<=1.2*1.2:
				var earned_matches = rows.filter(func(row): return row.id==EARNED_TRAVERSAL_START.row.id)
				if earned_matches.size()==1 and _free_traversal_row_valid(earned_matches[0],EARNED_TRAVERSAL_START.row):
					return {'generation':generation,'context':_context().duplicate(true),'entityId':EARNED_TRAVERSAL_START.row.id,'kind':'contextTraversal','label':EARNED_TRAVERSAL_START.label}
	var start: Dictionary = FREE_TRAVERSAL_STARTS[snapshot.sceneId]
	var matches = rows.filter(func(row): return row.id==start.row.id)
	if matches.size()!=1 or not _free_traversal_row_valid(matches[0],start.row):
		return {}
	var position = snapshot.player.transform.positionM
	if not P.point(position) or position.yM<start.band[0] or position.yM>start.band[1]:
		return {}
	var dx = position.xM-start.row['from'][0]
	var dz = position.zM-start.row['from'][2]
	if dx*dx+dz*dz>1.2*1.2:
		return {}
	return {'generation':generation,'context':_context().duplicate(true),'entityId':start.row.id,'kind':'contextTraversal','label':start.label}
