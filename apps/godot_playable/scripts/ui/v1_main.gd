extends Node2D
const Content = preload('res://scripts/ui/content_model.gd')
const ThemeModel = preload('res://scripts/ui/theme_model.gd')
const P = preload('res://scripts/bridge/protocol.gd')
const ShaftPresentation = preload('res://scripts/world/presentation.gd')
const Rpc = preload('res://scripts/bridge/rpc_client.gd')
const Catalog = preload('res://scripts/world/scene_catalog.gd')
const Renderer = preload('res://scripts/world/renderer.gd')
const EventView = preload('res://scripts/world/event_view.gd')
const CombatFeedback = preload('res://scripts/ui/combat_feedback_model.gd')
const Hud = preload('res://scripts/ui/game_hud.gd')
const NativeAudio = preload('res://scripts/audio/native_audio.gd')
const Panels = preload('res://scripts/ui/panels.gd')
const WorldLoading = preload('res://scripts/ui/world_loading_view.gd')
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
var combat_feedback = CombatFeedback.new()
var hud
var damage_feedback: ColorRect
var damage_feedback_remaining = 0.0
var panels
var world_loading
var grey_hive_loading_art: Texture2D = null
var loading_art_cache: Dictionary = {}
var loading_request_id = 0
var loading_generation = -1
var gh_arrival_token: Dictionary = {}
var snapshot: Dictionary = {}
var exit_entry_preview: Dictionary = {}
var beacon_exit_ready_pending: Dictionary = {}
var hive_route_ready_pending: Dictionary = {}
var exit_preview_journey := false
var exit_postframe_resume: Dictionary = {}
var status: Dictionary = {}
var generation = 0
var panel_generation = 0
var request_sequence = 0
var input_sequence = 0
var action_sequence = 0
var lifecycle_sequence = 0
var journey_started = false
var focused = true
var gameplay_rearm = false
var dense_visual_hold = false
var pair_visual_hold_pending: Dictionary = {}
var pair_visual_hold_receipt: Dictionary = {}
var seam_rearm_pending = false
var mouse_move_owner: Dictionary = {}
var mouse_move_rearm = false
var movement_pad_enabled = false
var mouse_guard_owner: Dictionary = {}
var mouse_guard_rearm = false
var guard_active = false
var blocked_keys = {}
var logical_down: Dictionary = {}
var movement_down: Dictionary = {}
var pending_movement: Dictionary = {}
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
var save_notice: Dictionary = {}
var preferences = {'scale':1.0,'fullscreen':false,'color':false,'reduced':false,'muted':false,'volume':0.35}
var pointer_seen = false
var aim_pointer_fresh = false
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
var trace_pad_records = 0
var trace_pad_samples = 0
var trace_save_records = 0
var trace_sink = null
var trace_sink_records: int = 0
var trace_sink_bytes: int = 0
var trace_sink_attempted: bool = false
# Separate, bounded observation of the first maintenance encounter. The existing
# input-trace opt-in is inherited by the current player launcher and Godot log.
var combat_record_enabled: bool = false
var combat_record_finished: bool = false
var combat_record_started_ms: int = -1
var combat_record_count: int = 0
var combat_record_bytes: int = 0
var combat_record_context: Dictionary = {}
var combat_record_generation: int = -1
var combat_record_connection: int = -1
var combat_record_cursor: int = 0
var combat_record_last_hp: int = -1
var combat_record_clicks: Array = []
var combat_record_seen_events: Dictionary = {}
var combat_record_dispatch_click: int = 0
# Independent, event-driven pause provenance; global budgets are never reset.
var pause_record_enabled: bool = false
var pause_record_started_ms: int = 0
var pause_record_count: int = 0
var pause_record_bytes: int = 0
var pause_record_exhausted: bool = false
var pause_record_episodes: int = 0
var pause_record_active: int = 0
var pause_record_context: Dictionary = {}
var pause_record_generation: int = -1
var pause_record_connection: int = -1
var pause_record_pending_id: int = 0
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
	damage_feedback = ColorRect.new()
	damage_feedback.name = 'DamageFeedback'
	damage_feedback.mouse_filter = Control.MOUSE_FILTER_IGNORE
	damage_feedback.visible = false
	ui.add_child(damage_feedback)
	damage_feedback.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	hud = Hud.new()
	hud.name = 'GameHUD'
	ui.add_child(hud)
	renderer.safe_rect_provider = Callable(hud,'get_world_safe_rect')
	renderer.rs_camera_fit_provider = Callable(hud,'get_rs_camera_fit_rect')
	hud.intent.connect(_hud_intent)
	hud.pad_trace.connect(_trace_pad)
	world_loading = WorldLoading.new()
	world_loading.name = 'WorldLoading'
	ui.add_child(world_loading)
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
	if not exit_entry_preview.is_empty():
		var t: Dictionary=exit_entry_preview.token
		return {'worldId':t.worldId,'sceneId':t.sceneId,'worldEpoch':t.worldEpoch}
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
	if method=='formal_pause':
		if exit_entry_preview.is_empty(): exit_postframe_resume.clear()
		_pause_record_prepare(extra)
	if method in ['formal_new','formal_continue','formal_continue_slot','formal_scene_transition','formal_world_gate','formal_return','formal_return_slot']:
		gh_arrival_token.clear()
	var id = rpc.request(method,params,_assoc(extra))
	if id>0 and rpc.authenticated and rpc.pending.has(id) and method in ['formal_new','formal_continue','formal_continue_slot','formal_scene_transition','formal_world_gate','formal_return','formal_return_slot']:
		loading_request_id = id
		loading_generation = generation
		var hint_world = ''
		var loading_message = '正在载入，等待请求完成'
		if method in ['formal_new','formal_return','formal_return_slot']:
			hint_world = 'return_station'
			loading_message = '正在建立旅程，等待归航站响应' if method=='formal_new' else '正在返回归航站，等待请求完成'
		elif method in ['formal_continue','formal_continue_slot']:
			loading_message = '正在读取存档，等待旅程恢复'
		elif method=='formal_scene_transition' and P.snapshot(snapshot) and P.context(_context()) and P.same(_context(),snapshot):
			hint_world = str(snapshot.worldId)
			loading_message = '正在切换场景，等待请求完成'
		elif method=='formal_world_gate' and P.snapshot(snapshot) and P.context(_context()) and P.same(_context(),snapshot):
			var origin = snapshot.worldId+':'+snapshot.sceneId+':'+str(params.get('id',''))
			hint_world = {'return_station:rs_core_room:rs_world_gate_to_gh':'grey_hive','return_station:rs_core_room:rs_world_gate_to_mh':'mist_harbor','return_station:rs_core_room:rs_world_gate_to_cw':'clockworks','grey_hive:gh_exit:gh_extraction_return_to_rs':'return_station','mist_harbor:mh_extraction:mh_extraction_return_to_rs':'return_station','clockworks:cw_shutdown_exit:cw_shutdown_return_to_rs':'return_station'}.get(origin,'')
			if hint_world=='return_station': loading_message = '正在返回归航站，等待请求完成'
		world_loading.begin_loading(Panels.WORLDS.get(hint_world,'正在进入世界'))
		world_loading.set_short_lore(loading_message)
		panels.set_world_loading_active(true)
		_set_world_loading_art(hint_world)
	if method=='formal_pause':
		_pause_record_submitted(id,extra)
	if id>0 and method in ['formal_continue','formal_save']:
		_trace_save_flow('request_queued',method,id)
	if id==0 and block:
		critical = ''
		_feedback('链路暂不可用')
	return id
func _id(prefix: String) -> String:
	request_sequence += 1
	return prefix+'-'+str(generation)+'-'+str(request_sequence)
func _connection(connected: bool, code: String) -> void:
	if not connected:
		save_notice.clear()
		_clear_world_loading()
		gh_arrival_token.clear()
	_pause_record_clear()
	if not connected:
		_combat_record_finish('connection_closed')
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
		if close_attempted:
			_trace_frontend('close_disposition',{'reason':'close_disconnected'})
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
	return not gameplay_rearm and _eligible_without_rearm()
func _eligible_without_rearm() -> bool:
	return exit_entry_preview.is_empty() and not close_attempted and journey_started and rpc.authenticated and focused and not uncertain and critical.is_empty() and not panels.visible and not snapshot.is_empty() and not status.is_empty() and P.same(status,snapshot) and not status.paused and status.get('entryToken')==null and snapshot.get('entryToken')==null and ready_pending.is_empty() and snapshot.player.currentHp>0 and is_instance_valid(renderer) and renderer.dense_region_ready() and Time.get_ticks_msec()-last_snapshot_at<1000 and Time.get_ticks_msec()-last_status_at<1000
func _gameplay_inputs_released() -> bool:
	for key in [KEY_W,KEY_A,KEY_S,KEY_D,KEY_UP,KEY_DOWN,KEY_LEFT,KEY_RIGHT,KEY_J,KEY_SHIFT,KEY_SPACE,KEY_Q,KEY_E,KEY_R,KEY_F,KEY_T,KEY_M]:
		if _key_held(key):
			return false
	return not Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
func _process(delta: float) -> void:
	_maybe_beacon_exit_ready()
	_maybe_hive_route_ready()
	_maybe_exit_ready()
	_maybe_exit_resume()
	var pair_view=snapshot.get('beaconExitNeighbor',{})
	var pair_active: bool = (pair_view is Dictionary and pair_view.get('ready')==true) or snapshot.get('beaconExitHold') is Dictionary or snapshot.get('beaconExitHoldMissing')==true
	var route_view=snapshot.get('hiveRouteNeighbor',{})
	var route_boundary=snapshot.get('hiveRouteBoundary',{})
	var route_active: bool = (route_view is Dictionary and route_view.get('ready')==true) or snapshot.get('hiveRouteHold') is Dictionary or snapshot.get('hiveRouteHoldMissing')==true
	var route_scope: bool = route_active or (route_boundary is Dictionary and route_boundary.get('closed')==true)
	var prepared_seam: bool = is_instance_valid(renderer) and seam_rearm_pending and renderer._pair_visibility_scope() and renderer.is_dense_region_available()
	var visual_scope: bool = is_instance_valid(renderer) and (renderer._region_visibility_scope() or renderer.shaft_visual_expected() or renderer.shaft_legacy_deferred() or pair_active or route_scope)
	var visual_ready: bool = renderer.dense_region_ready() if visual_scope else false
	var frame_pending: bool = (renderer._decon_frame_pending() or renderer._pair_frame_pending() or renderer.hive_route_frame_pending() or prepared_seam) if visual_scope and not visual_ready else false
	var dense_unavailable: bool = visual_scope and not visual_ready and not frame_pending
	if dense_unavailable and not dense_visual_hold:
		dense_visual_hold = true
		_release()
		# Once the native pair owns movement, losing its matching visual plan
		# must hold the authoritative owner rather than leave a running dash.
		if (pair_active or route_active) and P.snapshot(snapshot) and not status.is_empty() and P.same(status,snapshot) and status.get('paused')==false and critical.is_empty() and _advance_lifecycle():
			var pause_id = _request('formal_pause',{'context':_context(),'commandSequence':lifecycle_sequence},'pair_visual_hold',{'sequence':lifecycle_sequence},true)
			if pause_id>0 and rpc.pending.has(pause_id):
				pair_visual_hold_receipt.clear()
				pair_visual_hold_pending = {'id':pause_id,'generation':generation,'connection':rpc.connection_generation,'context':_context().duplicate(true),'sequence':lifecycle_sequence}
	elif not dense_unavailable:
		dense_visual_hold = false
	if not pair_visual_hold_pending.is_empty() and (uncertain or close_attempted or not rpc.authenticated or pair_visual_hold_pending.generation!=generation or pair_visual_hold_pending.connection!=rpc.connection_generation or pair_visual_hold_pending.sequence!=lifecycle_sequence or not P.context(pair_visual_hold_pending.context) or not P.snapshot(snapshot) or not P.same(pair_visual_hold_pending.context,snapshot) or not rpc.pending.has(pair_visual_hold_pending.id)):
		pair_visual_hold_pending.clear()
	if not pair_visual_hold_receipt.is_empty() and (uncertain or close_attempted or not rpc.authenticated or pair_visual_hold_receipt.generation!=generation or pair_visual_hold_receipt.connection!=rpc.connection_generation or pair_visual_hold_receipt.sequence!=lifecycle_sequence or not P.snapshot(snapshot) or not P.same(pair_visual_hold_receipt.context,snapshot) or snapshot.get('entryToken')!=null or not exit_entry_preview.is_empty() or not ready_pending.is_empty()):
		pair_visual_hold_receipt.clear()
	var show_damage: bool = damage_feedback_remaining>0.0 and _eligible_without_rearm() and not preferences.get('reduced',false)
	if is_instance_valid(damage_feedback):
		damage_feedback.visible = show_damage
		if show_damage:
			damage_feedback.color = Color(0.82,0.18,0.14,0.10*clampf(damage_feedback_remaining/0.45,0.0,1.0))
	damage_feedback_remaining = maxf(0.0,damage_feedback_remaining-delta) if show_damage else 0.0
	if seam_rearm_pending and _eligible_without_rearm():
		seam_rearm_pending = false
		var move_held = Input.is_physical_key_pressed(KEY_W) or Input.is_physical_key_pressed(KEY_A) or Input.is_physical_key_pressed(KEY_S) or Input.is_physical_key_pressed(KEY_D) or Input.is_physical_key_pressed(KEY_UP) or Input.is_physical_key_pressed(KEY_DOWN) or Input.is_physical_key_pressed(KEY_LEFT) or Input.is_physical_key_pressed(KEY_RIGHT)
		var action_held = _key_held(KEY_F) or _key_held(KEY_T) or _key_held(KEY_M)
		for key in ACTION_KEYS:
			if _key_held(key): action_held = true
		if move_held and not action_held and not Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT): gameplay_rearm = false
	if gameplay_rearm and _eligible_without_rearm() and _gameplay_inputs_released():
		gameplay_rearm = false
	_observe_mouse_movement()
	_observe_mouse_guard()
	_trace_tick()
	if combat_record_started_ms>=0 and not combat_record_finished:
		_combat_record_live()
	if close_confirmed:
		return
	poll_accumulator += delta
	input_accumulator += delta
	if not close_attempted and rpc.authenticated and poll_accumulator>=0.05:
		poll_accumulator = 0
		_request('formal_snapshot',{},'poll')
		_request('bridge_session_status',{},'status')
		if exit_entry_preview.is_empty() and journey_started and not snapshot.is_empty():
			_request('formal_presentation_events',{'worldEpoch':snapshot.worldEpoch,'afterEventId':event_view.cursor},'events')
			_request('formal_sound_cues',{'worldEpoch':snapshot.worldEpoch,'afterEventId':event_view.sound_cursor},'sound')
	var eligible = _eligible()
	if not eligible:
		pending_movement.clear()
	audio.update(snapshot,catalog.current,generation,eligible,focused)
	if input_accumulator>=0.05:
		input_accumulator = 0
		if eligible:
			if _sample(false):
				var combat_dispatch: Dictionary = _combat_record_dispatch_map()
				var queued = pending_actions.duplicate(true)
				pending_actions.clear()
				var combat_dispatch_index = 0
				for edge in queued:
					combat_record_dispatch_click = combat_dispatch.get(combat_dispatch_index,0)
					_send_action(edge.kind,edge.time)
					combat_dispatch_index += 1
				combat_record_dispatch_click = 0
		elif neutral_pending and critical.is_empty():
			_sample(true)
	if guard_active and (not eligible or not (_key_held(KEY_E) or _mouse_guard_valid())):
		_end_guard()
	for key in blocked_keys.keys():
		if not _key_held(key):
			blocked_keys.erase(key)
	var projected_events: Array = []
	if not snapshot.is_empty():
		projected_events = event_view.project(snapshot,renderer,eligible)
		renderer.events = projected_events
		renderer.sound = event_view.sound
	var combat_lines: Array = combat_feedback.update(snapshot,eligible,generation,rpc.connection_generation,event_view.cursor,projected_events)
	if is_instance_valid(panels):
		panels.set_baizhi_region_visibility(snapshot,_baizhi_region_visible())
		panels.update_rear_view(snapshot,_rear_view_owner_valid(panels.page_owner))
	var nearest = _nearest()
	var hint = audio.status_text if audio.failed else last_feedback
	var priority_lines: Array = []
	var visual_notice_context = P.snapshot(snapshot) and snapshot.player.currentHp>0 and snapshot.get('entryToken')==null and exit_entry_preview.is_empty() and ready_pending.is_empty() and not uncertain and not close_attempted and rpc.authenticated and not panels.visible and P.context(_context()) and P.same(_context(),snapshot)
	var confirmed_visual_pause = visual_notice_context and pair_visual_hold_receipt.get('confirmed')==true and P.status(status) and status.get('paused')==true and status.get('lastLifecycleCommandSequence')==pair_visual_hold_receipt.sequence and P.same(status,snapshot) and Time.get_ticks_msec()-last_status_at<1000
	if confirmed_visual_pause:
		priority_lines.append('区域安全暂停已确认')
	elif visual_notice_context and not pair_visual_hold_pending.is_empty():
		priority_lines.append('区域安全暂停请求 · 正在等待确认')
	elif visual_notice_context and frame_pending:
		priority_lines.append('区域画面准备中 · 请等待当前画面准备完成')
	elif visual_notice_context and dense_unavailable:
		priority_lines.append('区域显示暂不可用 · 当前画面尚未就绪')
	if gameplay_rearm and _eligible_without_rearm():
		priority_lines.append('松开按键后可继续操作')
	if renderer._region_visibility_scope() and not renderer.is_dense_region_available() and not (visual_notice_context and (frame_pending or dense_unavailable)):
		priority_lines.append('区域显示暂不可用')
	var current_interaction = _current_hud_interaction(nearest)
	var interaction_presented: bool = not renderer._region_visibility_scope() or renderer.is_dense_visible_interaction(nearest)
	if not interaction_presented: current_interaction = {}
	if eligible and not nearest.is_empty() and interaction_presented:
		var prompt = '[F] '+current_interaction.label if not current_interaction.is_empty() else _interaction_prompt(nearest)
		if current_interaction.is_empty() and prompt.begins_with('[F] '):
			prompt = ''
		if not current_interaction.is_empty():
			priority_lines.append(prompt)
		elif not prompt.is_empty():
			hint = hint+'\n'+prompt if not hint.is_empty() else prompt
	if _eligible_without_rearm() and P.snapshot(snapshot) and snapshot.player.currentHp>0 and float(snapshot.player.currentHp)<=float(snapshot.player.maxHp)*0.25:
		priority_lines.append('生命偏低，谨慎推进')

	var visual_axes: Vector2i = _movement_axes() if eligible else Vector2i.ZERO
	var visual_move := Vector2(visual_axes.y+0.5*visual_axes.x,visual_axes.y-0.5*visual_axes.x)
	renderer.set_visual_movement(visual_move,snapshot,eligible)
	var aim_visible: bool = eligible and aim_pointer_fresh and pointer_seen and get_viewport().gui_get_hovered_control()==null
	renderer.set_aim_reticle(get_viewport().get_mouse_position(),snapshot,aim_visible)
	renderer.set_interaction_focus(current_interaction,snapshot,eligible,generation)
	hud.update_view(snapshot,connection_text,eligible,hint,preferences.scale,current_interaction,_current_hud_traversal(),_current_hud_actions(),_current_hud_guard(),_current_hud_movements(),movement_pad_enabled,_owner(),combat_lines,priority_lines,_shaft_orientation(eligible))

func _shaft_orientation(eligible: bool) -> String:
	if not eligible or not P.snapshot(snapshot) or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_central_shaft' or snapshot.get('entryToken')!=null or snapshot.get('exitEntryPreview')==true or snapshot.get('shaftServiceEntryPreview')==true or snapshot.get('shaftLiftDeferred')!=false or not exit_entry_preview.is_empty() or not ready_pending.is_empty(): return ''
	var view = ShaftPresentation.shaft_lift_view(snapshot.get('shaftLift'),snapshot,catalog.current,catalog.source_hash)
	if view.is_empty() or view.legacyFloorActive: return ''
	if view.rider.mode=='floor':
		var y = float(view.rider.positionM.yM)
		if absf(y)<0.0001: return '所在位置：上层地面'
		if absf(y+4.0)<0.0001: return '所在位置：下层地面'
	elif view.rider.mode=='surface':
		if view.pose.phase=='upper_hold': return '轿厢停靠：上层'
		if view.pose.phase=='lower_hold': return '轿厢停靠：下层'
		if view.pose.phase=='rising' and float(view.pose.velocityMps)>0.0: return '轿厢上行中'
		if view.pose.phase=='falling' and float(view.pose.velocityMps)<0.0: return '轿厢下行中'
		return '所在位置：轿厢内'
	return ''
func _sample(neutral: bool) -> bool:
	if neutral or not _eligible():
		pending_movement.clear()
		if not neutral:
			return false
	if close_attempted:
		_trace_pad_sample('closing',neutral)
		return false
	if snapshot.is_empty() or not rpc.authenticated:
		_trace_pad_sample('unavailable',neutral)
		return false
	if snapshot.get('entryToken')!=null or not ready_pending.is_empty() or not status.is_empty() and P.same(status,snapshot) and status.paused:
		_trace_pad_sample('paused_or_entry',neutral)
		neutral_pending = false
		return false
	var now = Time.get_ticks_msec()
	if now-last_input_packet_at<50:
		_trace_pad_sample('sample_gate',neutral)
		if neutral:
			neutral_pending = true
		return false
	input_sequence += 1
	if input_sequence>=9007199254740991:
		_trace_pad_sample('counter_limit',neutral)
		if not uncertain:
			_hold('E_COUNTER_EXHAUSTED')
		return false
	var axes: Vector2i = _movement_axes() if not neutral else Vector2i.ZERO
	var sx: int = axes.x
	var sy: int = axes.y
	var move = Vector2(sy+0.5*sx,sy-0.5*sx).normalized() if not neutral else Vector2.ZERO
	var aim = Vector2.ZERO
	if pointer_seen and not neutral:
		if get_viewport().gui_get_hovered_control()!=null:
			if P.snapshot(snapshot):
				aim = Vector2(snapshot.player.aimX,snapshot.player.aimZ)
		else:
			var player = P.vector(snapshot.player.transform.positionM)
			var mouse_world = renderer.camera.inverse(get_viewport().get_mouse_position(),player.y)
			aim = Vector2(mouse_world.x-player.x,mouse_world.z-player.z).normalized()
	var sample = {'protocol':'continuous-input','protocolVersion':2,'worldEpoch':snapshot.worldEpoch,'seq':input_sequence,'clientTimeMs':Time.get_ticks_msec(),'moveX':move.x,'moveZ':move.y,'aimX':aim.x,'aimZ':aim.y}
	var submitted = _request('formal_submit_input',{'sample':sample,'combat':[]},'input',{'neutral':neutral})
	if not neutral:
		_combat_record_sample(sample,submitted)
	_trace_pad_sample('sample',neutral,axes,submitted>0)
	if submitted>0:
		pending_movement.clear()
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
	if kind=='guardStart' and not mouse_guard_owner.is_empty() and (guard_active or pending_actions.any(func(edge): return edge.kind=='guardStart')):
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
	var submitted = _request('formal_submit_action',{'action':action},'action',{'actionKind':kind})
	_combat_record_action(kind,action,submitted)
	if submitted>0 and kind=='guardStart':
		guard_active = true
func _end_guard() -> void:
	if _mouse_guard_valid():
		return
	if not mouse_guard_owner.is_empty():
		_clear_mouse_guard()
	pending_actions = pending_actions.filter(func(edge): return edge.kind!='guardStart')
	if guard_active:
		guard_active = false
		_action('guardEnd',true)
func _release() -> void:
	seam_rearm_pending = false
	aim_pointer_fresh = false
	gameplay_rearm = true
	pending_movement.clear()
	movement_down.clear()
	if is_instance_valid(renderer):
		renderer.clear_interaction_focus()
	_clear_mouse_movement()
	_clear_mouse_guard()
	if is_instance_valid(audio):
		audio.suspend_world()
	if guard_active or _key_held(KEY_E) or pending_actions.any(func(edge): return edge.kind=='guardStart'):
		guard_blocked = true
	pending_actions.clear()
	if exit_entry_preview.is_empty():
		_end_guard()
		_sample(true)
	else:
		guard_active=false
		neutral_pending=false
	for key in ACTION_KEYS:
		if _key_held(key):
			blocked_keys[key] = true
func _event_key(event: InputEventKey) -> int:
	if event.physical_keycode in TRACE_KEYS:
		return event.physical_keycode
	return event.keycode if event.keycode in TRACE_KEYS else 0
func _key_held(key: int) -> bool:
	return Input.is_physical_key_pressed(key) or logical_down.has(key)
func _unhandled_input(event: InputEvent) -> void:
	_trace_key(event,'unhandled','received')
	if event is InputEventMouseMotion:
		pointer_seen = true
		if _eligible() and get_viewport().gui_get_hovered_control()==null:
			aim_pointer_fresh = true
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
			_pause_record_event('escape_press','escape')
			_escape()
			get_viewport().set_input_as_handled()
			return
		if key==KEY_M and panels.visible and panels.mode=='map' and not blocked_keys.has(key):
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
	if event is InputEventMouseButton and event.button_index==MOUSE_BUTTON_LEFT and event.pressed:
		var attack_eligible: bool = _eligible()
		var attack_hovered = null
		var before_attack: int = pending_actions.size()
		if attack_eligible:
			attack_hovered = get_viewport().gui_get_hovered_control()!=null
			if not attack_hovered:
				pointer_seen = true
				_action('primaryAttack')
		_combat_record_admission(event,attack_eligible,attack_hovered,before_attack)
func _input(event: InputEvent) -> void:
	_trace_key(event,'input','received')
	if event is InputEventMouseButton and event.button_index==MOUSE_BUTTON_LEFT and event.pressed:
		_combat_record_received(event)
	if event is InputEventMouseButton and event.button_index==MOUSE_BUTTON_LEFT and not event.pressed:
		_end_mouse_guard()
		mouse_guard_rearm = false
		_clear_mouse_movement()
		mouse_move_rearm = false
	if not event is InputEventKey:
		return
	var key = _event_key(event)
	if key==0:
		return
	var movement_keys = {KEY_W:Vector2i(0,-1),KEY_UP:Vector2i(0,-1),KEY_S:Vector2i(0,1),KEY_DOWN:Vector2i(0,1),KEY_A:Vector2i(-1,0),KEY_LEFT:Vector2i(-1,0),KEY_D:Vector2i(1,0),KEY_RIGHT:Vector2i(1,0)}
	var movement_edge = event.pressed and not event.echo and not movement_down.has(key)
	if movement_keys.has(key):
		if not event.pressed:
			movement_down.erase(key)
		elif not event.echo:
			movement_down[key] = true
	if not event.physical_keycode in TRACE_KEYS:
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
			if not _eligible() and (ACTION_KEYS.has(key) or key in [KEY_F,KEY_T,KEY_M]) and not (key==KEY_M and panels.visible and panels.mode=='map'):
				blocked_keys[key] = true
	if event.pressed and not event.echo and key in [KEY_W,KEY_A,KEY_S,KEY_D,KEY_UP,KEY_DOWN,KEY_LEFT,KEY_RIGHT]:
		_clear_mouse_movement()
		if movement_edge and _eligible():
			pending_movement = {'axes':movement_keys[key],'generation':generation,'connection':rpc.connection_generation,'context':_context()}
		elif not _eligible():
			pending_movement.clear()
	if event.pressed and not event.echo and key==KEY_ESCAPE:
		_trace_key(event,'input','escape')
		_pause_record_event('escape_press','escape')
		_escape()
		get_viewport().set_input_as_handled()
	# Release is observed even when a native control consumed the key-up.
	if not event.pressed and key==KEY_E:
		blocked_keys.erase(KEY_E)
		guard_blocked = false
		_end_guard()
func _notification(what: int) -> void:
	if what==NOTIFICATION_WM_CLOSE_REQUEST and (not is_instance_valid(rpc) or not rpc.authenticated):
		_trace_frontend('close_disposition',{'reason':'client_quit_disconnected'})
		_trace_sink_close()
		get_tree().quit()
		return
	if not is_instance_valid(rpc):
		return
	if what==NOTIFICATION_APPLICATION_FOCUS_OUT:
		_pause_record_event('focus_out','focus_out')
		guard_blocked = true
		var was_eligible = _eligible_without_rearm()
		_release()
		if was_eligible:
			_open_paused('pause',{},'focus_out')
		logical_down.clear()
		focused = false
		audio.stop_all()
	elif what==NOTIFICATION_APPLICATION_FOCUS_IN:
		_pause_record_event('focus_in','focus_in')
		focused = true
		_release()
	elif what==NOTIFICATION_WM_CLOSE_REQUEST:
		if not critical.is_empty() or not ready_pending.is_empty() or not ticket.is_empty():
			_trace_frontend('close_disposition',{'reason':'notification_blocked'})
			_feedback('请先等待当前确认或关闭对白')
			return
		if journey_started and rpc.authenticated and not uncertain and critical.is_empty():
			_trace_frontend('close_disposition',{'reason':'confirmation_requested'})
			_open_paused('exit_confirm')
		else:
			_trace_frontend('close_disposition',{'reason':'notification_no_confirmed_save'})
			_show('error' if uncertain else 'start')
			panels.set_message('当前没有已确认保存；退出客户端不证明服务已关闭')
func _escape() -> void:
	if not focused or uncertain or not critical.is_empty() or not ready_pending.is_empty() or snapshot.get('entryToken')!=null:
		return
	if panels.visible:
		if panels.mode=='pause':
			_resume()
		elif panels.mode in ['settings','inventory','character','capability','mission','archive','save','slots','map','new_confirm','exit_confirm','named_save_confirm']:
			_back()
		return
	if _eligible_without_rearm():
		_open_paused('pause',{},'escape')
func _show(page: String, extra: Dictionary = {}, blocked: bool = false, refresh_saves: bool = true, preserve_save_entry: bool = false, pause_observation: Dictionary = {}) -> void:
	var pause_was_displayed: bool = is_instance_valid(panels) and panels.visible and panels.mode=='pause'
	if is_instance_valid(panels) and not panels.visible:
		_release()
	_clear_mouse_movement()
	for key in logical_down:
		if ACTION_KEYS.has(key) or key in [KEY_F,KEY_T,KEY_M]:
			blocked_keys[key] = true
	panel_generation += 1
	var o = _owner()
	o.merge(extra,true)
	if page=='slots' and is_instance_valid(rpc): o['slotConnection'] = rpc.connection_generation
	panels.preferences = preferences.duplicate(true)
	panels.font_scale = preferences.scale
	panels.set_baizhi_region_visibility(snapshot,_baizhi_region_visible())
	_publish_saves(false)
	panels.show_page(page,o,snapshot,rpc.authenticated if is_instance_valid(rpc) else false,blocked,preserve_save_entry)
	if page=='pause' and not save_notice.is_empty():
		if not blocked and is_instance_valid(rpc) and save_notice.get('generation')==generation and save_notice.get('connection')==rpc.connection_generation and P.context(save_notice.get('context')) and P.same(save_notice.context,_context()):
			panels.set_message(save_notice.text)
		save_notice.clear()
	if page=='pause' and not pause_was_displayed and panels.visible and panels.mode=='pause':
		_pause_record_display(pause_observation)
	if refresh_saves and page in ['slots','save'] and not blocked:
		_refresh_saves()
func _open_paused(page: String, page_data: Dictionary = {}, pause_origin: String = 'unknown') -> void:
	if not journey_started or not snapshot.is_empty() and snapshot.player.currentHp<=0 and page in ['slots','new_confirm','settings']:
		_show(page,page_data)
		return
	if snapshot.is_empty() or status.is_empty() or not P.same(status,snapshot) or not focused or uncertain or not critical.is_empty() or snapshot.get('entryToken')!=null or status.get('entryToken')!=null or snapshot.player.currentHp<=0:
		if page=='pause':
			_pause_record_event('page_blocked',pause_origin)
		_feedback('当前不能打开此面板')
		if page=='save' and is_instance_valid(panels) and panels.visible:
			panels.set_message(last_feedback)
		return
	if page=='pause':
		_pause_record_open(page,pause_origin)
	_release()
	if status.paused:
		_show(page,page_data,false,true,false,{'route':'already_paused','origin':pause_origin})
	else:
		if not _advance_lifecycle():
			if page=='pause':
				_pause_record_clear()
			return
		var sequence = lifecycle_sequence
		var pause_extra = {'page':page,'pageData':page_data,'sequence':sequence}
		if pause_record_enabled and pause_record_active>0:
			pause_extra['pause_record_origin'] = pause_origin
		if _request('formal_pause',{'context':_context(),'commandSequence':sequence},'open_page',pause_extra,true)>0:
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
	if panels.mode=='named_save_confirm':
		_show('save',{},false,false)
	elif not ticket.is_empty():
		_close_baizhi()
	elif journey_started:
		if not snapshot.is_empty() and snapshot.player.currentHp<=0:
			_show('death')
			return
		if panels.mode=='enhancement':
			enhancement_pending.clear()
		_show('pause',{},false,true,false,{'route':'back','origin':'unknown'})
	else:
		_show('start')
func _hud_intent(kind: String, value) -> void:
	if kind=='movement_pad_toggle':
		if value is Dictionary and value.size()==2 and value.has_all(['enabled','owner']) and value.owner is Dictionary:
			_ui_intent(kind,value.enabled,value.owner)
		return
	# Background HUD navigation must not borrow an open modal page's owner.
	if panels.visible and kind in ['mission','inventory','character','capability','map','pause']:
		return
	_ui_intent(kind,value,_owner() if kind in ['interact_current','traverse_current','player_action_current','guard_current_begin','guard_current_end','movement_current_begin','movement_current_end'] else panels.page_owner.duplicate(true))
func _ui_intent(kind: String, value, owner: Dictionary) -> void:
	if kind=='movement_current_end':
		if _movement_identity_same(value,mouse_move_owner):
			_clear_mouse_movement()
		return
	if kind=='guard_current_end':
		if _guard_association_valid(value) and _guard_identity_same(value,mouse_guard_owner):
			_end_mouse_guard()
		return
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
		if is_instance_valid(panels) and panels.visible and panels.mode=='settings' and panels.page_owner==owner and not panels.pending and value is Dictionary and value==panels.staged and value!=preferences and value.get('scale') in [1.0,1.25,1.5] and typeof(value.get('fullscreen'))==TYPE_BOOL and typeof(value.get('color'))==TYPE_BOOL and typeof(value.get('reduced'))==TYPE_BOOL and typeof(value.get('muted'))==TYPE_BOOL and P.number(value.get('volume'),0,1) and value.size()==6:
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
	if kind in ['continue_default','save_exit']:
		_trace_save_flow('ui_intent_admitted','formal_continue' if kind=='continue_default' else 'formal_save',0)
	if kind in ['back','resume','start','dialogue_close','baizhi_page']:
		audio.ui_cue('cancel',focused)
	elif kind in ['settings','inventory','character','capability','mission','archive','map','pause']:
		audio.ui_cue('tab',focused)
	elif kind in ['new_confirm','exit_confirm','save','new','continue_default','continue_slot','slots','slots_refresh','save_default','save_slot','save_exit','build_item','capability_select','capability_activate','enhancement_choose','baizhi_confirm','baizhi_commit','return','close_client']:
		audio.ui_cue('confirm',focused)
	match kind:
		'movement_pad_toggle':
			if typeof(value)!=TYPE_BOOL or owner!=_owner() or panels.visible or not _eligible():
				return
			movement_pad_enabled = value
			_clear_mouse_movement()
		'movement_current_begin':
			var admitted: bool = _movement_capture_admitted(value)
			_trace_pad('main',str(value.get('entityId','')) if value is Dictionary else '','admitted' if admitted else 'rejected',{'admitted':admitted})
			if not admitted:
				return
			for current in _current_hud_movements():
				if _movement_identity_same(value,current):
					pending_movement.clear()
					mouse_move_owner = value.duplicate(true)
					_trace_pad('main',mouse_move_owner.entityId,'captured',{'captured':true})
					return
		'guard_current_begin':
			if not _guard_association_valid(value) or value.generation!=generation or not P.same(value.context,_context()) or not mouse_guard_owner.is_empty() or not mouse_move_owner.is_empty() or mouse_guard_rearm or not Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
				return
			var current = _current_hud_guard()
			if current.is_empty() or not _guard_identity_same(value,current) or guard_blocked and _key_held(KEY_E):
				return
			if not _key_held(KEY_E):
				guard_blocked = false
			mouse_guard_owner = value.duplicate(true)
			if not guard_active and not pending_actions.any(func(edge): return edge.kind=='guardStart'):
				_action('guardStart')
		'player_action_current':
			if not value is Dictionary or value.size()!=5 or not value.has_all(['generation','context','entityId','kind','label']) or not P.counter(value.generation) or value.generation!=generation or not P.context(value.context) or not P.same(value.context,_context()) or typeof(value.entityId)!=TYPE_STRING or typeof(value.kind)!=TYPE_STRING or value.kind!='player_action' or typeof(value.label)!=TYPE_STRING or not PLAYER_ACTION_LABELS.has(value.entityId) or value.label!=PLAYER_ACTION_LABELS[value.entityId]:
				return
			for current in _current_hud_actions():
				if current.generation==value.generation and P.same(current.context,value.context) and current.entityId==value.entityId and current.kind==value.kind and current.label==value.label:
					_action(value.entityId)
					return
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
		'pause':
			_pause_record_event('ui_pause_intent','ui_pause')
			_open_paused(kind,{},'ui_pause')
		'new_confirm','settings','inventory','character','capability','mission','archive','save','map','exit_confirm': _open_paused(kind)
		'back': _back()
		'resume': _resume()
		'start': _show('start')
		'new': _begin_journey('formal_new',{})
		'continue_default': _begin_journey('formal_continue',{})
		'continue_slot': _continue_slot_row(value,owner)
		'slots':
			_open_paused('slots')
		'slots_refresh': _refresh_saves()
		'save_default': _save_request('formal_save',{},false)
		'save_exit': _save_request('formal_save',{},true)
		'named_save_prepare': _named_save_prepare(value,owner)
		'named_save_confirm': _named_save_confirm(value,owner)
		'save_slot_new': _save_request('formal_save_slot',{'slotId':'v1-'+str(Time.get_ticks_msec())+'-'+str(request_sequence+1),'displayName':value,'create':true},false)
		'return':
			if journey_started:
				_request('formal_return',{},'return',{},true)
		'build_item': _build_item(value,owner)
		'capability_select':
			if value is Dictionary:
				var current_rear = Panels.rear_view_readiness(snapshot)
				if value.size()==3 and value.get('id')=='perception.rear_view_i' and value.get('granted')==true and typeof(value.get('selected'))==TYPE_BOOL and _rear_view_owner_valid(owner) and current_rear.get('granted')==true and current_rear.get('selected')==value.selected and panels.rear_view_presented==current_rear:
					_capability_select('perception.rear_view_i')
				elif panels.visible and panels.mode=='capability' and panels.page_owner==owner:
					panels.set_message('能力状态已更新，请重新确认。')
			elif value!='perception.rear_view_i':
				_capability_select(value)
		'capability_activate':
			var readiness = Panels.rear_view_readiness(snapshot)
			if value=='perception.rear_view_i' and _rear_view_owner_valid(owner) and readiness.get('state')=='ready' and panels.rear_view_presented==readiness:
				if action_sequence>=9007199254740990:
					_hold('E_COUNTER_EXHAUSTED')
					return
				action_sequence += 1
				_request('formal_capability_command',{'command':{'kind':'activate','capabilityId':value,'requestId':action_sequence}},'capability',{},true)
			elif panels.visible and panels.mode=='capability' and panels.page_owner==owner:
				panels.set_message('能力状态已更新，请重新确认。')
		'enhancement_choose': _choose_enhancement(value)
		'baizhi_confirm':
			if _baizhi_choice_ready(owner,value,'baizhi'):
				_show('baizhi_confirm',{'choice':value,'dialogueTicket':ticket.duplicate(true)})
		'baizhi_page':
			if _baizhi_region_visible(): _show('baizhi')
		'baizhi_commit': _commit_baizhi(value,owner)
		'baizhi_unresolved':
			if not ticket.is_empty() and _baizhi_region_visible():
				_show('dialogue',{'text':'那就先别打开那扇门。','baizhiOwned':true})
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
	_pause_record_clear()
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
func _exit_save_message(text: String, owner: Dictionary) -> void:
	if is_instance_valid(panels) and panels.visible and panels.mode=='exit_confirm' and owner.get('generation')==generation and owner.get('panelGeneration')==panel_generation and P.context(owner.get('context')) and P.same(owner.context,_context()):
		panels.set_message(text)
func _save_request(method: String, params: Dictionary, exit_after: bool) -> void:
	var save_owner = _owner().duplicate(true)
	if not journey_started or snapshot.is_empty() or not ticket.is_empty() or snapshot.get('entryToken')!=null:
		if exit_after:
			_exit_save_message('当前不能保存并退出，请先完成当前操作',save_owner)
		return
	if method=='formal_save_slot' and params.get('create')!=true and not _writable_slot_params(params):
		return
	if _request(method,params,'save',{'exitAfter':exit_after,'saveOwner':save_owner,'connectionGeneration':rpc.connection_generation},true)>0:
		_set_save_notice('保存中 · 等待实际回执')
		panels.set_message('保存中 · 等待实际回执',true)
	else:
		_set_save_notice('保存请求未发送，请重试')
		if exit_after:
			_exit_save_message('保存请求未发送，请稍后重试',save_owner)
		elif is_instance_valid(panels) and panels.visible:
			panels.set_message('保存请求未发送，请重试')
func _build_item(value, owner: Dictionary) -> void:
	if not is_instance_valid(panels) or not panels.visible or panels.mode!='inventory' or panels.page_owner!=owner: return
	var valid = value is Dictionary and value.size()==6 and value.has_all(['revision','itemId','slotId','equipped','expected','kind'])
	if valid:
		valid = P.counter(value.revision) and P.ident(value.itemId) and P.ident(value.slotId) and typeof(value.equipped)==TYPE_BOOL and (value.expected==null or P.ident(value.expected)) and value.kind==('unequip' if value.equipped else 'equip')
	if valid:
		valid = rpc.authenticated and focused and not panels.pending and not uncertain and critical.is_empty() and P.snapshot(snapshot) and snapshot.player.currentHp>0 and snapshot.get('entryToken')==null and P.context(owner.get('context')) and P.same(owner.context,snapshot) and P.status(status) and status.paused and P.same(status,snapshot) and P.build(snapshot.get('build')) and snapshot.build.revision==value.revision
	if valid:
		var items = snapshot.build.items.filter(func(item): return item.itemId==value.itemId)
		var slots = snapshot.build.equipment.filter(func(e): return e.slotId==value.slotId)
		var current = slots[0].itemId if slots.size()==1 else null
		valid = items.size()==1 and items[0].slotId==value.slotId and items[0].equipped==value.equipped and current==value.expected and (not value.equipped or current==value.itemId)
	if not valid:
		panels.set_message('物品状态已更新，请重新选择。')
		return
	var action = {'kind':'unequip','slotId':value.slotId,'expectedItemId':value.itemId} if value.equipped else {'kind':'equip','itemId':value.itemId,'expectedItemId':value.expected}
	var rid = _id('build')
	_request('formal_build_command',{'command':{'requestId':rid,'worldEpoch':snapshot.worldEpoch,'expectedBuildRevision':value.revision,'action':action}},'build',{'expectedCommand':'build:'+rid,'buildRevision':value.revision,'buildAction':action},true)
func _rear_view_owner_valid(owner: Dictionary) -> bool:
	return is_instance_valid(panels) and panels.visible and panels.mode=='capability' and panels.page_owner==owner and not panels.pending and is_instance_valid(rpc) and rpc.authenticated and focused and journey_started and not uncertain and not close_attempted and critical.is_empty() and ticket.is_empty() and exit_entry_preview.is_empty() and ready_pending.is_empty() and P.snapshot(snapshot) and snapshot.player.currentHp>0 and snapshot.get('entryToken')==null and snapshot.get('exitEntryPreview')!=true and snapshot.get('shaftServiceEntryPreview')!=true and P.context(owner.get('context')) and owner.get('generation')==generation and owner.get('panelGeneration')==panel_generation and P.same(owner.context,snapshot) and P.status(status) and status.paused and status.get('entryToken')==null and P.same(status,snapshot) and Time.get_ticks_msec()-last_snapshot_at<1000 and Time.get_ticks_msec()-last_status_at<1000

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
		if i is Dictionary and (renderer.beacon_exit_ordinary_suppressed(i) or renderer.hive_route_ordinary_suppressed(i)): continue
		if renderer._region_visibility_scope() and not renderer.is_dense_visible_interaction(i): continue
		if player.distance_to(P.vector(i.transform.positionM))<=2.5 and renderer.visible_point(P.vector(i.transform.positionM)):
			rows.append(i)
	rows.sort_custom(func(a,b):
		if a.active!=b.active:
			return a.active
		var ad = player.distance_to(P.vector(a.transform.positionM))
		var bd = player.distance_to(P.vector(b.transform.positionM))
		return a.entityId<b.entityId if is_equal_approx(ad,bd) else ad<bd)
	if rows.is_empty():
		return {}
	var closest: Dictionary = rows[0]
	if _is_rs_gh_signpost(closest):
		var gates: Array = []
		for candidate in rows:
			if typeof(candidate.get('active'))==TYPE_BOOL and candidate.get('active') and candidate.get('entityId')=='rs_world_gate_marker' and candidate.get('kind')=='world_gate' and not _destination_label(candidate).is_empty():
				gates.append(candidate)
		if gates.size()==1:
			return gates[0]
	return closest
func _baizhi_available(i: Dictionary) -> bool:
	var b = snapshot.get('baizhi')
	return i.get('kind')=='npc_dialogue' and i.get('entityId')=='gh_bz_first_contact' and snapshot.get('worldId')=='grey_hive' and snapshot.get('sceneId')=='gh_bio_isolation' and typeof(i.get('active'))==TYPE_BOOL and i.active and b is Dictionary and typeof(b.get('canInteract'))==TYPE_BOOL and b.canInteract

func _baizhi_region_visible() -> bool:
	if not P.snapshot(snapshot) or not is_instance_valid(renderer) or not P.snapshot(renderer.snapshot) or not P.same(snapshot,renderer.snapshot) or snapshot.serverTick!=renderer.snapshot.serverTick or snapshot.authorityRevision!=renderer.snapshot.authorityRevision: return false
	return renderer.is_baizhi_region_authorized()
func _storage_inventory_available(i: Dictionary) -> bool:
	if not P.snapshot(snapshot) or not _eligible():
		return false
	var current_context = _context()
	if not P.context(current_context) or not P.same(current_context,snapshot):
		return false
	if snapshot.get('worldId')!='return_station' or snapshot.get('sceneId')!='rs_core_room' or snapshot.get('entryToken')!=null or snapshot.player.currentHp<=0:
		return false
	if catalog.current.get('worldId')!=snapshot.worldId or catalog.current.get('sceneId')!=snapshot.sceneId:
		return false
	if typeof(i.get('entityId'))!=TYPE_STRING or i.get('entityId')!='rs_storage_inventory_marker' or typeof(i.get('kind'))!=TYPE_STRING or i.get('kind')!='storage_inventory_marker':
		return false
	if typeof(i.get('active'))!=TYPE_BOOL or not i.get('active') or not P.transform(i.get('transform')):
		return false
	var native_rows = snapshot.get('interactables')
	if not P.rows(native_rows,2048) or not P.unique(native_rows,'entityId'):
		return false
	var native_matches: Array = []
	for row in native_rows:
		if row is Dictionary and typeof(row.get('entityId'))==TYPE_STRING and row.entityId=='rs_storage_inventory_marker' and typeof(row.get('kind'))==TYPE_STRING and row.kind=='storage_inventory_marker':
			native_matches.append(row)
	if native_matches.size()!=1:
		return false
	var native_row: Dictionary = native_matches[0]
	if typeof(native_row.get('active'))!=TYPE_BOOL or not native_row.get('active') or not P.transform(native_row.get('transform')) or native_row.get('active')!=i.get('active') or native_row.get('transform')!=i.get('transform'):
		return false
	var authored_rows = catalog.current.get('interactions')
	if not P.rows(authored_rows,2048) or not P.unique(authored_rows,'id'):
		return false
	var authored_matches: Array = []
	for row in authored_rows:
		if row is Dictionary and typeof(row.get('id'))==TYPE_STRING and row.id=='rs_storage_inventory_marker':
			authored_matches.append(row)
	if authored_matches.size()!=1:
		return false
	var authored_row: Dictionary = authored_matches[0]
	if typeof(authored_row.get('kind'))!=TYPE_STRING or authored_row.get('kind')!='storage_inventory_marker':
		return false
	var position = authored_row.get('position')
	if typeof(position)!=TYPE_ARRAY or position.size()!=3:
		return false
	if not P.number(position[0]) or not P.number(position[1]) or not P.number(position[2]):
		return false
	var authored_position = Vector3(float(position[0]),float(position[1]),float(position[2]))
	if authored_position!=Vector3(15.0,0.0,12.0):
		return false
	return P.vector(native_row.transform.positionM)==authored_position
func _is_rs_gh_signpost(i: Dictionary) -> bool:
	return snapshot.get('worldId')=='return_station' and snapshot.get('sceneId')=='rs_core_room' and catalog.current.get('worldId')=='return_station' and catalog.current.get('sceneId')=='rs_core_room' and i.get('entityId')=='rs_gh_entry_marker' and i.get('kind')=='grey_hive_entry_marker'
func _interaction_prompt(i: Dictionary) -> String:
	var label = _interaction_label(i)
	if _is_rs_gh_signpost(i):
		return '灰巢入口标识 · 仅作标识，不可操作'
	if typeof(i.get('active'))!=TYPE_BOOL or not i.active:
		return label+' · 当前不可用'
	if i.get('kind')=='npc_dialogue':
		return '[F] '+label if _baizhi_available(i) else label+' · 当前对白不可用'
	if SCENE_RPC.has(i.get('kind')) or i.get('kind') in INTERACT_KINDS or _storage_inventory_available(i):
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
	elif i.kind=='world_gate' and i.entityId=='gh_extraction_return_to_rs' and snapshot.worldId=='grey_hive' and snapshot.sceneId=='gh_exit':
		if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or not P.counter(snapshot.get('worldEpoch'),1) or not P.context(_context()) or not P.same(_context(),snapshot) or typeof(i.get('active'))!=TYPE_BOOL or not P.transform(i.get('transform')):
			return ''
		var native_rows = snapshot.get('interactables')
		var authored_rows = catalog.current.get('interactions')
		if not P.rows(native_rows,2048) or not P.unique(native_rows,'entityId') or not P.rows(authored_rows,2048) or not P.unique(authored_rows,'id'):
			return ''
		var native_matches = native_rows.filter(func(row): return row.get('entityId')=='gh_extraction_return_to_rs')
		var console_matches = authored_rows.filter(func(row): return row.get('id')=='gh_exit_extraction_console')
		var progress = Content.progress(snapshot,'grey_hive')
		if native_matches.size()!=1 or native_matches[0].get('kind')!='world_gate' or typeof(native_matches[0].get('active'))!=TYPE_BOOL or native_matches[0].active!=i.active or not P.transform(native_matches[0].get('transform')) or native_matches[0].transform!=i.transform or P.vector(i.transform.positionM)!=Vector3(8,0,8):
			return ''
		if console_matches.size()!=1 or console_matches[0].size()!=4 or console_matches[0].get('kind')!='extraction_console' or console_matches[0].get('event')!='hive_extraction' or console_matches[0].get('position')!=[8,0,8]:
			return ''
		if progress.is_empty() or snapshot.progression.get('currentWorldId')!='grey_hive' or not progress.completed or not progress.firstCompletion or not 'hive_extraction' in progress.completedEvents:
			return ''
		return '返回归航站'
	elif i.kind=='world_gate' and ((snapshot.worldId=='mist_harbor' and snapshot.sceneId=='mh_extraction' and i.entityId=='mh_extraction_return_to_rs') or (snapshot.worldId=='clockworks' and snapshot.sceneId=='cw_shutdown_exit' and i.entityId=='cw_shutdown_return_to_rs')):
		if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or not P.counter(snapshot.get('worldEpoch'),1) or not P.context(_context()) or not P.same(_context(),snapshot) or i.get('active')!=true or not P.transform(i.get('transform')):
			return ''
		var native_rows = snapshot.get('interactables')
		if not P.rows(native_rows,2048) or not P.unique(native_rows,'entityId'):
			return ''
		var matches = native_rows.filter(func(row): return row.get('entityId')==i.entityId)
		if matches.size()!=1 or matches[0].get('kind')!='world_gate' or matches[0].get('active')!=true or not P.transform(matches[0].get('transform')) or matches[0].transform!=i.transform:
			return ''
		return '返回归航站'
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
const PLAYER_ACTION_LABELS = {'primaryAttack':'攻击','dash':'冲刺','pulse':'脉冲','pierce':'穿刺'}
const MOVEMENT_LABELS = {'move_up':'上移','move_down':'下移','move_left':'左移','move_right':'右移','move_up_left':'左上移','move_up_right':'右上移','move_down_left':'左下移','move_down_right':'右下移'}
const MOVEMENT_AXES = {'move_up_left':Vector2i(-1,-1),'move_up':Vector2i(0,-1),'move_up_right':Vector2i(1,-1),'move_left':Vector2i(-1,0),'move_right':Vector2i(1,0),'move_down_left':Vector2i(-1,1),'move_down':Vector2i(0,1),'move_down_right':Vector2i(1,1)}
func _keyboard_move_state() -> Dictionary:
	var right: bool = _key_held(KEY_D) or _key_held(KEY_RIGHT)
	var left: bool = _key_held(KEY_A) or _key_held(KEY_LEFT)
	var down: bool = _key_held(KEY_S) or _key_held(KEY_DOWN)
	var up: bool = _key_held(KEY_W) or _key_held(KEY_UP)
	return {'held':right or left or down or up,'axes':Vector2i(int(right)-int(left),int(down)-int(up))}
func _movement_axes() -> Vector2i:
	var keyboard: Dictionary = _keyboard_move_state()
	if keyboard.held:
		return keyboard.axes
	if not pending_movement.is_empty():
		if _eligible() and pending_movement.generation==generation and pending_movement.connection==rpc.connection_generation and P.same(pending_movement.context,_context()) and mouse_move_owner.is_empty():
			return pending_movement.axes
		pending_movement.clear()
	return MOVEMENT_AXES[mouse_move_owner.entityId] if _mouse_movement_valid() else Vector2i.ZERO
func _movement_capture_admitted(value) -> bool:
	return movement_pad_enabled and _eligible() and not _keyboard_move_state().held and _movement_association_valid(value) and value.generation==generation and P.same(value.context,_context()) and mouse_move_owner.is_empty() and not mouse_move_rearm and mouse_guard_owner.is_empty() and Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) and is_instance_valid(hud) and hud._movement_capture_valid(value)
func _movement_association_valid(value) -> bool:
	return value is Dictionary and value.size()==5 and value.has_all(['generation','context','entityId','kind','label']) and P.counter(value.generation) and P.context(value.context) and typeof(value.entityId)==TYPE_STRING and MOVEMENT_LABELS.has(value.entityId) and typeof(value.kind)==TYPE_STRING and value.kind=='player_move' and typeof(value.label)==TYPE_STRING and value.label==MOVEMENT_LABELS[value.entityId]
func _movement_identity_same(a, b) -> bool:
	return _movement_association_valid(a) and _movement_association_valid(b) and a.generation==b.generation and P.same(a.context,b.context) and a.entityId==b.entityId and a.kind==b.kind and a.label==b.label
func _current_hud_movements() -> Array:
	if not movement_pad_enabled or not P.snapshot(snapshot) or not P.context(_context()) or not _eligible() or mouse_move_rearm:
		return []
	var result: Array = []
	for direction_id in MOVEMENT_LABELS:
		result.append({'generation':generation,'context':_context().duplicate(true),'entityId':direction_id,'kind':'player_move','label':MOVEMENT_LABELS[direction_id]})
	return result
func _mouse_movement_valid() -> bool:
	if not movement_pad_enabled or _keyboard_move_state().held or mouse_move_owner.is_empty() or mouse_move_rearm or not mouse_guard_owner.is_empty() or not Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) or not is_instance_valid(hud) or not hud._movement_capture_valid(mouse_move_owner):
		return false
	for current in _current_hud_movements():
		if _movement_identity_same(mouse_move_owner,current):
			return true
	return false
func _clear_mouse_movement() -> void:
	var had_capture: bool = not mouse_move_owner.is_empty()
	var control_id: String = mouse_move_owner.get('entityId','')
	mouse_move_owner.clear()
	mouse_move_rearm = Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
	if is_instance_valid(hud):
		hud._clear_movement_captures()
	if had_capture and not _keyboard_move_state().held:
		neutral_pending = true
	if had_capture:
		_trace_pad('main',control_id,'clear',{'has_owner':false,'rearm':mouse_move_rearm,'neutral':neutral_pending})
func _observe_mouse_movement() -> void:
	if not Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
		_clear_mouse_movement()
		mouse_move_rearm = false
	elif not mouse_move_owner.is_empty() and not _mouse_movement_valid():
		_clear_mouse_movement()
func _guard_association_valid(value) -> bool:
	return value is Dictionary and value.size()==5 and value.has_all(['generation','context','entityId','kind','label']) and P.counter(value.generation) and P.context(value.context) and typeof(value.entityId)==TYPE_STRING and value.entityId=='guard' and typeof(value.kind)==TYPE_STRING and value.kind=='player_guard' and typeof(value.label)==TYPE_STRING and value.label=='防御'
func _guard_identity_same(a: Dictionary, b: Dictionary) -> bool:
	return _guard_association_valid(a) and _guard_association_valid(b) and a.generation==b.generation and P.same(a.context,b.context) and a.entityId==b.entityId and a.kind==b.kind and a.label==b.label
func _current_hud_guard() -> Dictionary:
	if not P.snapshot(snapshot) or not P.context(_context()) or not _eligible() or mouse_guard_rearm:
		return {}
	return {'generation':generation,'context':_context().duplicate(true),'entityId':'guard','kind':'player_guard','label':'防御'}
func _mouse_guard_valid() -> bool:
	return not mouse_guard_owner.is_empty() and not mouse_guard_rearm and _guard_identity_same(mouse_guard_owner,_current_hud_guard()) and Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
func _clear_mouse_guard() -> void:
	mouse_guard_owner.clear()
	mouse_guard_rearm = Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
	if is_instance_valid(hud):
		hud._cancel_guard_capture()
func _end_mouse_guard() -> void:
	if mouse_guard_owner.is_empty():
		return
	_clear_mouse_guard()
	if not (_eligible() and not guard_blocked and _key_held(KEY_E)):
		_end_guard()
func _observe_mouse_guard() -> void:
	if not Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
		mouse_guard_rearm = false
	if not mouse_guard_owner.is_empty() and not _mouse_guard_valid():
		_end_mouse_guard()
func _current_hud_actions() -> Array:
	if not P.snapshot(snapshot) or not P.context(_context()) or not _eligible():
		return []
	var result: Array = []
	for action_id in PLAYER_ACTION_LABELS:
		result.append({'generation':generation,'context':_context().duplicate(true),'entityId':action_id,'kind':'player_action','label':PLAYER_ACTION_LABELS[action_id]})
	return result
func _current_hud_interaction(supplied_nearest = null) -> Dictionary:
	if not P.snapshot(snapshot) or not P.context(_context()) or not _eligible():
		return {}
	var i = supplied_nearest if supplied_nearest!=null else _nearest()
	if i.is_empty() or typeof(i.get('active'))!=TYPE_BOOL or not i.active:
		return {}
	if i.kind=='npc_dialogue':
		if not _baizhi_available(i):
			return {}
	elif i.kind=='storage_inventory_marker':
		if not _storage_inventory_available(i):
			return {}
	elif not SCENE_RPC.has(i.kind) and not i.kind in INTERACT_KINDS:
		return {}
	return {'generation':generation,'context':_context().duplicate(true),'entityId':i.entityId,'kind':i.kind,'label':_interaction_label(i)}
func _interaction_label(i: Dictionary) -> String:
	if _is_rs_gh_signpost(i):
		return '灰巢入口标识'
	if _storage_inventory_available(i):
		return '背包'
	var destination = _destination_label(i)
	if not destination.is_empty():
		return destination
	if P.snapshot(snapshot) and snapshot.worldId=='clockworks' and snapshot.sceneId=='cw_pressure_hall' and catalog.current.get('worldId')==snapshot.worldId and catalog.current.get('sceneId')==snapshot.sceneId and typeof(i.get('entityId'))==TYPE_STRING and typeof(i.get('kind'))==TYPE_STRING and i.kind=='valve_control' and typeof(i.get('active'))==TYPE_BOOL and P.transform(i.get('transform')):
		var valve_labels = {'cw_pressure_valve_01_staged':'阀门 1','cw_pressure_valve_02_staged':'阀门 2','cw_pressure_valve_03_staged':'阀门 3'}
		var authored = catalog.current.get('interactions')
		if valve_labels.has(i.entityId) and P.rows(authored,2048) and P.unique(authored,'id'):
			var native_matches = 0
			for row in snapshot.interactables:
				if row.entityId==i.entityId and row.kind==i.kind and typeof(row.get('active'))==TYPE_BOOL and P.transform(row.get('transform')) and row.active==i.active and row.transform==i.transform:
					native_matches += 1
			var authored_matches = 0
			for row in authored:
				if row.id==i.entityId and typeof(row.get('kind'))==TYPE_STRING and row.kind=='valve_control':
					authored_matches += 1
			if native_matches==1 and authored_matches==1:
				return valve_labels[i.entityId]
	return {'world_gate':'世界入口','mission_terminal':'任务终端','capability_terminal':'能力终端','save_rest_terminal':'休息 / 保存','npc_dialogue':'与白芷交谈','scene_transition':'前往下一场景','scene_checkpoint':'检查点','scene_trigger':'场景装置','environment_control':'环境控制','power_console':'恢复电力','door_panel':'门控制','lockdown_terminal':'隔离协议','facility_log':'设施记录','bio_log_terminal':'生物记录','gate_b_panel':'Gate B 控制','extraction_console':'撤离控制','beacon_collect':'收取信标','beacon_mount':'部署信标','beacon':'信标','pump_control':'泵站控制','valve_control':'阀门','coolant_valve':'冷却阀','static_dialogue_marker':'查看炉心状态','terminal':'终端'}.get(i.kind,'未知交互（不可执行）')
func _interact() -> void:
	var i = _nearest()
	if i is Dictionary and renderer.gh_first_pair_portal(i):
		_feedback('请步行通过通道')
		return
	if not _eligible() or i.is_empty():
		_feedback('没有范围内可交互对象')
		return
	if renderer._region_visibility_scope() and not renderer.is_dense_visible_interaction(i):
		_feedback('当前区域交互不可用')
		return
	if not i.active:
		_feedback('目标已锁定或当前不可用；未发送命令')
		return
	var kind: String = i.kind
	var id: String = i.entityId
	if kind=='storage_inventory_marker' and _storage_inventory_available(i):
		_open_paused('inventory')
		return
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
	if renderer._region_visibility_scope() and not renderer.is_dense_visible_interaction(i):
		_feedback('当前区域交互不可用')
		return
	_request(method,params,'interaction',{'expectedCommand':rid,'target':i.duplicate(true),'before':snapshot.duplicate(true)},true)
func _begin_baizhi() -> void:
	if status.is_empty() or not P.same(status,snapshot) or not _baizhi_region_visible():
		return
	if not _advance_lifecycle():
		return
	var owner_id = 'v1-godot-'+str(generation)+'-'+str(panel_generation+1)
	var request = {'context':_context(),'entityId':'gh_bz_whitezhi_v1','interactionId':'gh_bz_first_contact','ownerId':owner_id,'generation':panel_generation+1,'commandSequence':lifecycle_sequence}
	_request('formal_baizhi_begin',{'request':request},'baizhi_begin',{'ticketOwner':owner_id,'ticketGeneration':panel_generation+1,'sequence':lifecycle_sequence},true)
	_show('waiting',{},true)
func _baizhi_choice_ready(owner: Dictionary, choice, page: String) -> bool:
	if not is_instance_valid(panels) or not panels.visible or panels.mode!=page or panels.page_owner!=owner or panels.pending or not is_instance_valid(rpc) or not rpc.authenticated or not focused or uncertain or close_attempted or not critical.is_empty() or not journey_started: return false
	if typeof(choice)!=TYPE_STRING or not choice in ['taken','left'] or not P.snapshot(snapshot) or snapshot.player.currentHp<=0 or snapshot.get('entryToken')!=null or not exit_entry_preview.is_empty() or not ready_pending.is_empty() or not P.context(owner.get('context')) or not P.same(owner.context,snapshot) or not P.ticket(ticket,_context()) or not _baizhi_region_visible(): return false
	if page=='baizhi_confirm' and (owner.get('choice')!=choice or owner.get('dialogueTicket')!=ticket): return false
	return P.status(status) and status.paused and P.same(status,snapshot) and status.lastLifecycleCommandSequence==ticket.pauseCommandSequence and P.snapshot(panels.snapshot) and P.same(panels.snapshot,snapshot) and panels.snapshot.serverTick==snapshot.serverTick and panels.snapshot.authorityRevision==snapshot.authorityRevision and panels._baizhi_choice_state()=='unresolved'

func _commit_baizhi(choice, owner: Dictionary) -> void:
	if not _baizhi_choice_ready(owner,choice,'baizhi_confirm'):
		if panels.visible and panels.mode=='baizhi_confirm' and panels.page_owner==owner and not uncertain and not close_attempted:
			panels.set_message('当前信息不可用，请重新确认。')
		return
	var rid = _id('baizhi')
	var request_id = _request('formal_baizhi_commit',{'ticket':ticket.duplicate(true),'requestId':rid,'choice':choice},'baizhi_commit',{'expectedCommand':rid,'choice':choice},true)
	if panels.visible and panels.mode=='baizhi_confirm' and panels.page_owner==owner and not uncertain and not close_attempted and rpc.authenticated:
		if request_id>0 and rpc.pending.has(request_id): panels.set_message('正在提交回应，请等待确认。',true)
		else: panels.set_message('请求未发送，请稍后重试。')
func _close_baizhi() -> void:
	if ticket.is_empty() or not P.ticket(ticket,_context()):
		return
	var owner = panels.page_owner.duplicate(true)
	var page: String = panels.mode
	var request_id = _request('formal_baizhi_close',{'ticket':ticket.duplicate(true)},'baizhi_close',{'expectedCommand':'baizhi-close:'+ticket.ownerId+':'+str(int(ticket.generation))},true)
	if panels.visible and panels.mode==page and panels.page_owner==owner and not uncertain and not close_attempted and rpc.authenticated:
		if request_id>0 and rpc.pending.has(request_id): panels.set_message('正在等待对白关闭确认。',true)
		else: panels.set_message('请求未发送，请稍后重试。')
func _response(_rpc_id: int, method: String, result, association: Dictionary) -> void:
	var pause_observation: Dictionary = {}
	if close_attempted and method in Rpc.READS:
		return
	if close_confirmed:
		return
	if method in ['formal_has_save','formal_list_save_slots']:
		_save_read_response(_rpc_id,method,result,association)
		return
	var purpose: String = association.get('purpose','')
	if purpose=='beacon_exit_ready' and beacon_exit_ready_pending.get('id')==_rpc_id: beacon_exit_ready_pending.clear()
	if purpose=='hive_route_ready' and hive_route_ready_pending.get('id')==_rpc_id: hive_route_ready_pending.clear()
	var transition = purpose in ['journey','return','interaction'] or (purpose in ['ready','exit_cancel'] and not exit_entry_preview.is_empty())
	if not _current(association,transition):
		if purpose=='close':
			_trace_frontend('close_disposition',{'reason':'close_stale_association'})
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
		if result.get('exitEntryPreview')==true and not exit_entry_preview.is_empty():
			if not P.same(result,exit_entry_preview.token): _hold('E_EXIT_PREVIEW_STATUS')
			else: lifecycle_sequence=maxi(lifecycle_sequence,int(result.lastLifecycleCommandSequence))
			return
		if snapshot.is_empty() or not P.same(result,snapshot):
			return
		if result.lastLifecycleCommandSequence<lifecycle_sequence:
			return
		status = result
		if not pair_visual_hold_receipt.is_empty():
			if pair_visual_hold_receipt.generation==generation and pair_visual_hold_receipt.connection==rpc.connection_generation and pair_visual_hold_receipt.sequence==result.lastLifecycleCommandSequence and result.paused==true and result.get('entryToken')==null and result.get('exitEntryPreview')!=true and result.get('shaftServiceEntryPreview')!=true and snapshot.get('entryToken')==null and snapshot.get('exitEntryPreview')!=true and snapshot.get('shaftServiceEntryPreview')!=true and P.same(pair_visual_hold_receipt.context,result) and P.same(result,snapshot) and exit_entry_preview.is_empty() and ready_pending.is_empty():
				pair_visual_hold_receipt['confirmed'] = true
			else:
				pair_visual_hold_receipt.clear()
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
			_combat_record_events(result)
			event_view.accept(result,snapshot,renderer)
			audio.accept_events(result,snapshot,generation,_eligible())
		return
	if purpose=='sound':
		if not snapshot.is_empty():
			event_view.accept_sound(result,snapshot)
		return
	if purpose=='close':
		if not _close_current(_rpc_id,method,association):
			_trace_frontend('close_disposition',{'reason':'close_stale_association'})
			return
		close_pending.clear()
		if result is Dictionary and typeof(result.get('closing'))==TYPE_BOOL and result.closing:
			close_confirmed = true
			_trace_frontend('close_disposition',{'reason':'close_confirmed_quit'})
			_trace_sink_close()
			get_tree().quit()
		else:
			_trace_frontend('close_disposition',{'reason':'close_invalid_reply'})
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
	if method=='formal_pause':
		pause_observation = _pause_record_response(_rpc_id,r,association)
	if purpose=='beacon_exit_ready':
		if not P.success(r) or not P.same(r.snapshot,association.context): return
		var confirmed=r.snapshot.get('beaconExitNeighbor')
		if not confirmed is Dictionary or confirmed.get('ready')!=true or confirmed.get('token')!=association.get('token') or confirmed.get('targetSceneSourceSha256')!=association.get('hash'): return
		_accept_snapshot(r.snapshot)
		return
	if purpose=='hive_route_ready':
		if not P.success(r) or not P.same(r.snapshot,association.context): return
		var confirmed=r.snapshot.get('hiveRouteNeighbor')
		if not confirmed is Dictionary or confirmed.get('ready')!=true or confirmed.get('token')!=association.get('token') or confirmed.get('targetSceneSourceSha256')!=association.get('hash'): return
		_accept_snapshot(r.snapshot)
		return
	if purpose=='ready':
		var t: Dictionary = association.token
		if r.commandId!='scene-ready:'+str(int(t.generation)) or not P.same(r.snapshot,t) or r.snapshot.get('entryToken')!=null:
			_hold('E_SCENE_READY_RECEIPT')
			return
	if purpose in ['input','action','build','capability','enhancement','save','baizhi_begin','baizhi_commit','baizhi_close'] and not association.context.is_empty() and not P.same(r.snapshot,association.context):
		_hold('E_RECEIPT_STALE_CONTEXT')
		return
	if purpose=='action' and method=='formal_submit_action':
		_combat_record_response(_rpc_id,r,association)
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
	if purpose=='exit_cancel' and not exit_entry_preview.is_empty():
		if not P.success(r) or not P.same(r.snapshot,exit_entry_preview.base) or r.snapshot.get('exitEntryPreview')!=null:
			_hold('E_EXIT_PREVIEW_CANCEL')
			return
		exit_entry_preview.clear()
		exit_preview_journey=false
		_clear_world_loading()
		if not snapshot.is_empty() and not _accept_snapshot(r.snapshot,false,_rpc_id): return
		status.clear()
		_feedback('出口资源暂不可用，已保留原场景。')
		return
	if not P.success(r):
		_finish_world_loading(_rpc_id)
		if purpose=='ready':
			gh_arrival_token.clear()
		if purpose=='ready':
			_hold('E_SCENE_READY_REJECTED')
			return
		_accept_snapshot(r.snapshot)
		audio.ui_cue('error',focused)
		_feedback(_localized(str(r.get('errorCode','E_COMMAND_REJECTED'))))
		if purpose=='journey' and (uncertain or close_attempted): return
		if purpose=='journey' and r.get('errorCode')=='UNKNOWN_MUTATION':
			_hold('UNKNOWN_MUTATION')
			return
		if purpose=='save':
			_set_save_notice('保存未成功：'+last_feedback)
		if purpose=='open_page':
			panels.close_page()
			return
		if panels.visible:
			var recovery_page = _journey_failure_page(purpose,method)
			if not recovery_page.is_empty(): _feedback(_journey_failure_notice(recovery_page,last_feedback))
			var page = 'exit_confirm' if association.get('exitAfter',false) else recovery_page if not recovery_page.is_empty() else 'pause' if purpose=='resume' else panels.mode
			_show(page,{},false,recovery_page.is_empty())
			panels.set_message(last_feedback)
		return
	if r.snapshot.get('exitEntryPreview')==true or r.snapshot.get('shaftServiceEntryPreview')==true:
		if purpose=='journey' and r.commandId!=('new' if method=='formal_new' else 'continue') and method!='formal_continue_slot':
			_hold('E_JOURNEY_RECEIPT')
			return
		if purpose=='journey': exit_preview_journey=true
		_accept_snapshot(r.snapshot,true,_rpc_id)
		return
	if purpose=='ready' and association.get('shaftServiceExpected') is Dictionary:
		if r.snapshot.get('shaftLowerService')!=association.shaftServiceExpected or not P.same(r.snapshot,association.token):
			_hold('E_SHAFT_SERVICE_COMMIT')
			return
	if purpose=='ready' and not exit_entry_preview.is_empty():
		var shaft_preview: bool=exit_entry_preview.get('kind')=='shaft'
		var resume_key: String='shaftServiceEntryResumeAfterFrame' if shaft_preview else 'exitEntryResumeAfterFrame'
		var valid_source: bool = r.snapshot.get('shaftLowerService')==exit_entry_preview.get('identity') if shaft_preview else r.snapshot.get('utilitySceneSourceSha256')==exit_entry_preview.hash
		if not P.same(r.snapshot,exit_entry_preview.token) or not valid_source or r.snapshot.get('exitEntryPreview')!=null or r.snapshot.get('shaftServiceEntryPreview')!=null or typeof(r.snapshot.get(resume_key))!=TYPE_BOOL:
			_hold('E_EXIT_PREVIEW_COMMIT')
			return
		if r.snapshot.get(resume_key)==true:
			exit_postframe_resume={'context':{'worldId':r.snapshot.worldId,'sceneId':r.snapshot.sceneId,'worldEpoch':r.snapshot.worldEpoch},'hash':exit_entry_preview.hash,'kind':exit_entry_preview.get('kind','exit')}
		association.remainPaused=true
		if exit_preview_journey:
			journey_started=true
			panels.close_page()
	if purpose=='journey':
		if r.commandId!=('new' if method=='formal_new' else 'continue') and method!='formal_continue_slot':
			_hold('E_JOURNEY_RECEIPT')
			return
		journey_started = true
		panels.close_page()
	var retained_save: bool = purpose=='save' and _older_save_snapshot(r.snapshot,association)
	var snapshot_applied: bool = false
	if not retained_save:
		if not _accept_snapshot(r.snapshot,transition,_rpc_id):
			return
		snapshot_applied = true
	if purpose=='ready' and association.get('shaftServiceExpected') is Dictionary:
		if r.snapshot.get('shaftLowerService')!=association.shaftServiceExpected or not P.same(r.snapshot,association.token):
			_hold('E_SHAFT_SERVICE_COMMIT')
			return
	if purpose=='ready' and not exit_entry_preview.is_empty():
		exit_entry_preview.clear()
		exit_preview_journey=false
		_clear_world_loading()
	if purpose=='interaction' and method=='formal_world_gate' and r.applied and not r.alreadyApplied:
		var origin = association.get('before')
		var gate_target = association.get('target')
		if origin is Dictionary and gate_target is Dictionary and P.snapshot(origin) and origin.worldId=='return_station' and origin.sceneId=='rs_core_room' and gate_target.get('entityId')=='rs_world_gate_marker' and gate_target.get('kind')=='world_gate' and snapshot.worldId=='grey_hive' and snapshot.sceneId=='gh_entry_maintenance' and snapshot.get('checkpointId')=='gh_entry_checkpoint' and P.token(snapshot.get('entryToken')) and P.same(snapshot.entryToken,snapshot):
			var prior_gh = Content.progress(origin,'grey_hive')
			var current_gh = Content.progress(snapshot,'grey_hive')
			if not prior_gh.is_empty() and prior_gh.visitId==0 and not current_gh.is_empty() and current_gh.visitId==1:
				gh_arrival_token = snapshot.entryToken.duplicate(true)
	if snapshot_applied and (method=='formal_continue' and purpose=='journey' or method=='formal_save' and purpose=='save'):
		_trace_save_flow('snapshot_applied',method,_rpc_id,r,snapshot)
	match purpose:
		'pair_visual_hold':
			if method=='formal_pause' and not pair_visual_hold_pending.is_empty() and pair_visual_hold_pending.id==_rpc_id and pair_visual_hold_pending.generation==generation and pair_visual_hold_pending.connection==rpc.connection_generation and association.get('sequence')==pair_visual_hold_pending.sequence and lifecycle_sequence==pair_visual_hold_pending.sequence and P.context(association.get('context')) and P.same(association.context,pair_visual_hold_pending.context) and r.commandId=='pause' and P.same(r.snapshot,pair_visual_hold_pending.context) and r.snapshot.get('entryToken')==null:
				pair_visual_hold_receipt = pair_visual_hold_pending.duplicate(true)
				pair_visual_hold_receipt['confirmed'] = false
			pair_visual_hold_pending.clear()
		'journey':
			if method=='formal_new' and r.applied==true and r.alreadyApplied==false and P.snapshot(snapshot) and snapshot.worldId=='return_station' and snapshot.sceneId=='rs_core_room' and P.same(r.snapshot,snapshot):
				_feedback('归航链路稳定。请选择首次回收坐标：灰巢设施。')
		'open_page','enhancement_pause':
			pause_owned = _context()
			status = _context()
			status.merge({'paused':true,'lastLifecycleCommandSequence':association.sequence,'entryToken':null,'enhancementTerminalContext':null})
			if purpose=='enhancement_pause':
				enhancement_pending.sequence = association.sequence
				_show('waiting',{},true)
				_request('bridge_session_status',{},'status')
			else:
				_show(association.page,association.get('pageData',{}),false,true,false,pause_observation)
		'resume':
			pause_owned.clear()
			status = _context()
			status.merge({'paused':false,'lastLifecycleCommandSequence':association.sequence,'entryToken':null,'enhancementTerminalContext':null})
			panels.close_page()
			_release()
			_pause_record_clear()
		'ready':
			var announce_gh = r.applied and not r.alreadyApplied and not gh_arrival_token.is_empty() and association.get('token')==gh_arrival_token and P.same(gh_arrival_token,snapshot) and snapshot.worldId=='grey_hive' and snapshot.sceneId=='gh_entry_maintenance' and snapshot.get('checkpointId')=='gh_entry_checkpoint'
			gh_arrival_token.clear()
			ready_pending.clear()
			if announce_gh:
				_feedback('B-17 链路仅保持最低功率。')
			status.clear()
			_request('bridge_session_status',{},'status')
			if association.get('remainPaused',false) and exit_postframe_resume.is_empty():
				pause_owned = _context()
				_show('pause')
		'save':
			_set_save_notice('已收到保存成功回执')
			_feedback('已收到保存成功回执；不代表断电耐久性验收')
			if association.get('exitAfter',false):
				var save_owner = association.get('saveOwner',{})
				if is_instance_valid(panels) and panels.visible and panels.mode=='exit_confirm' and save_owner.get('generation')==generation and save_owner.get('panelGeneration')==panel_generation and P.context(save_owner.get('context')) and P.same(save_owner.context,_context()) and association.get('connectionGeneration')==rpc.connection_generation:
					if close_attempted or not rpc.authenticated or uncertain or not critical.is_empty():
						_exit_save_message('保存已确认，但关闭尚未提交',save_owner)
					else:
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
			_show('dialogue',{'text':'好，我会等你找到安全的出口。' if association.choice=='taken' else '我明白。','baizhiOwned':true})
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
# Preserve a newer display snapshot when a matching save receipt arrives late.
func _older_save_snapshot(s: Dictionary, association: Dictionary) -> bool:
	if not is_instance_valid(rpc) or not rpc.authenticated or uncertain or close_attempted or not journey_started or association.get('connectionGeneration')!=rpc.connection_generation:
		return false
	if not P.snapshot(snapshot) or not P.snapshot(s) or not catalog.manifest.has(s.sceneId) or not P.same(snapshot,s):
		return false
	if snapshot.get('entryToken')!=null or s.get('entryToken')!=null or snapshot.player.currentHp<=0 or s.player.currentHp<=0 or snapshot.player.entityId!=s.player.entityId:
		return false
	var older: bool = false
	for key in ['serverTick','authorityRevision','ackSeq']:
		if s[key]>snapshot[key]:
			return false
		if s[key]<snapshot[key]:
			older = true
	return older
func _mh_shore_position_in_admitted_pair(s: Dictionary, position: Vector3) -> bool:
	if not P.snapshot(s) or s.worldId!='mist_harbor' or position.y!=0.0 or position.z<6.0 or position.z>9.5: return false
	var edges={'mh_fog_pier':['','mh_tidal_warehouse'],'mh_tidal_warehouse':['mh_fog_pier','mh_signal_yard'],'mh_signal_yard':['mh_tidal_warehouse','mh_drowned_quay'],'mh_drowned_quay':['mh_signal_yard','mh_breakwater'],'mh_breakwater':['mh_drowned_quay','mh_pump_station'],'mh_pump_station':['mh_breakwater','mh_resonance_tower'],'mh_resonance_tower':['mh_pump_station','mh_warden_arena'],'mh_warden_arena':['mh_resonance_tower','mh_extraction'],'mh_extraction':['mh_warden_arena','']}
	var boundary=s.get('hiveRouteBoundary')
	if not edges.has(s.sceneId) or not boundary is Dictionary or boundary.get('sourceSceneId')!=s.sceneId or boundary.get('worldEpoch')!=s.worldEpoch or boundary.get('sourceSceneSourceSha256')!=catalog.source_hash or boundary.get('closed')!=false or typeof(boundary.get('east'))!=TYPE_BOOL: return false
	var target: String=edges[s.sceneId][1 if boundary.east else 0]
	if target.is_empty() or boundary.get('targetSceneId')!=target: return false
	if boundary.east and (position.x<24.0 or position.x>48.0) or not boundary.east and (position.x>0.0 or position.x< -24.0): return false
	var route=s.get('hiveRouteNeighbor')
	if route is Dictionary and route.get('ready')==true and route.get('sourceSceneId')==s.sceneId and route.get('targetSceneId')==target and route.get('worldEpoch')==s.worldEpoch and P.token(route.get('token')) and route.token.worldId=='mist_harbor' and route.token.sceneId==target and route.token.worldEpoch==s.worldEpoch+1 and route.get('targetSceneSourceSha256') is String and route.targetSceneSourceSha256.length()==64: return true
	var hold=s.get('hiveRouteHold')
	return hold is Dictionary and hold.get('sourceSceneId')==s.sceneId and hold.get('targetSceneId')==target and hold.get('worldEpoch')==s.worldEpoch and hold.get('sourceSceneSourceSha256')==catalog.source_hash and hold.get('targetSceneSourceSha256') is String and hold.targetSceneSourceSha256.length()==64

func _cw_early_position_in_admitted_pair(s: Dictionary, position: Vector3) -> bool:
	if not P.snapshot(s) or s.worldId!='clockworks' or position.y!=0.0 or position.z<6.0 or position.z>9.5: return false
	var edges={'cw_entry_foundry':['','cw_pressure_hall'],'cw_pressure_hall':['cw_entry_foundry','cw_conveyor_bridge'],'cw_conveyor_bridge':['cw_pressure_hall','cw_boiler_chamber'],'cw_boiler_chamber':['cw_conveyor_bridge','cw_gear_shaft'],'cw_gear_shaft':['cw_boiler_chamber',''],'cw_furnace_heart':['','cw_forged_guard_arena'],'cw_forged_guard_arena':['cw_furnace_heart','cw_regulator_core'],'cw_regulator_core':['cw_forged_guard_arena','cw_shutdown_exit'],'cw_shutdown_exit':['cw_regulator_core','']}
	var b=s.get('hiveRouteBoundary')
	if not edges.has(s.sceneId) or not b is Dictionary or b.get('sourceSceneId')!=s.sceneId or b.get('worldEpoch')!=s.worldEpoch or b.get('sourceSceneSourceSha256')!=catalog.source_hash or b.get('closed')!=false or typeof(b.get('east'))!=TYPE_BOOL: return false
	var target: String=edges[s.sceneId][1 if b.east else 0]
	if target.is_empty() or b.get('targetSceneId')!=target: return false
	if b.east and (position.x<24.0 or position.x>48.0) or not b.east and (position.x>0.0 or position.x< -24.0): return false
	var route=s.get('hiveRouteNeighbor')
	if route is Dictionary and route.get('ready')==true and route.get('sourceSceneId')==s.sceneId and route.get('targetSceneId')==target and route.get('worldEpoch')==s.worldEpoch and P.token(route.get('token')) and route.token.worldId=='clockworks' and route.token.sceneId==target and route.token.worldEpoch==s.worldEpoch+1 and route.get('targetSceneSourceSha256') is String and route.targetSceneSourceSha256.length()==64: return true
	var hold=s.get('hiveRouteHold')
	return hold is Dictionary and hold.get('sourceSceneId')==s.sceneId and hold.get('targetSceneId')==target and hold.get('worldEpoch')==s.worldEpoch and hold.get('sourceSceneSourceSha256')==catalog.source_hash and hold.get('targetSceneSourceSha256') is String and hold.targetSceneSourceSha256.length()==64

func _verified_pair_seam(old: Dictionary, next: Dictionary) -> bool:
	if not P.snapshot(old) or not P.snapshot(next) or old.worldId!=next.worldId or old.worldId not in ['grey_hive','mist_harbor','clockworks'] or int(next.worldEpoch)!=int(old.worldEpoch)+1 or old.player.entityId!=next.player.entityId: return false
	var route=old.get('hiveRouteNeighbor')
	if route is Dictionary and route.get('ready')==true and route.get('sourceSceneId')==old.sceneId and route.get('targetSceneId')==next.sceneId and route.get('worldEpoch')==old.worldEpoch and P.token(route.get('token')) and P.token(next.get('entryToken')) and next.entryToken.generation==route.token.generation+1 and next.entryToken.worldEpoch==next.worldEpoch:
		var handoff=next.get('hiveRouteHandoff')
		if not handoff is Dictionary or handoff.get('sourceSceneId')!=next.sceneId or handoff.get('targetSceneId')!=old.sceneId or handoff.get('worldEpoch')!=next.worldEpoch or handoff.get('sourceSceneSourceSha256')!=route.get('targetSceneSourceSha256') or handoff.get('targetSceneSourceSha256')!=catalog.source_hash: return false
		var edges={'gh_entry_maintenance':['','gh_power_room'],'gh_power_room':['gh_entry_maintenance',''],'gh_lockdown':['','gh_bio_isolation'],'gh_bio_isolation':['gh_lockdown','gh_gate_b'],'gh_gate_b':['gh_bio_isolation',''],'gh_deep_decon':['','gh_sentinel_arena'],'gh_sentinel_arena':['gh_deep_decon','gh_beacon'],'gh_beacon':['gh_sentinel_arena','']}
		if old.worldId=='mist_harbor': edges={'mh_fog_pier':['','mh_tidal_warehouse'],'mh_tidal_warehouse':['mh_fog_pier','mh_signal_yard'],'mh_signal_yard':['mh_tidal_warehouse','mh_drowned_quay'],'mh_drowned_quay':['mh_signal_yard','mh_breakwater'],'mh_breakwater':['mh_drowned_quay','mh_pump_station'],'mh_pump_station':['mh_breakwater','mh_resonance_tower'],'mh_resonance_tower':['mh_pump_station','mh_warden_arena'],'mh_warden_arena':['mh_resonance_tower','mh_extraction'],'mh_extraction':['mh_warden_arena','']}
		elif old.worldId=='clockworks': edges={'cw_entry_foundry':['','cw_pressure_hall'],'cw_pressure_hall':['cw_entry_foundry','cw_conveyor_bridge'],'cw_conveyor_bridge':['cw_pressure_hall','cw_boiler_chamber'],'cw_boiler_chamber':['cw_conveyor_bridge','cw_gear_shaft'],'cw_gear_shaft':['cw_boiler_chamber',''],'cw_furnace_heart':['','cw_forged_guard_arena'],'cw_forged_guard_arena':['cw_furnace_heart','cw_regulator_core'],'cw_regulator_core':['cw_forged_guard_arena','cw_shutdown_exit'],'cw_shutdown_exit':['cw_regulator_core','']}
		var targets: Array=edges.get(old.sceneId,[])
		if targets.size()!=2: return false
		var east: bool=next.sceneId==targets[1] and not targets[1].is_empty()
		if not east and (next.sceneId!=targets[0] or targets[0].is_empty()): return false
		return true
	if old.worldId!='grey_hive': return false
	if (old.sceneId=='gh_beacon' and next.sceneId=='gh_exit') or (old.sceneId=='gh_exit' and next.sceneId=='gh_beacon'):
		var v=old.get('beaconExitNeighbor')
		if not v is Dictionary or v.get('ready')!=true or v.get('sourceSceneId')!=old.sceneId or v.get('targetSceneId')!=next.sceneId or v.get('targetSceneSourceSha256')!=next.get('utilitySceneSourceSha256') or not P.token(v.get('token')) or not P.token(next.get('entryToken')): return false
		if next.entryToken.generation!=v.token.generation+1: return false
		var previous=P.vector(old.player.transform.positionM)+(Vector3.ZERO if old.sceneId=='gh_beacon' else Vector3(24,0,0))
		var following=P.vector(next.player.transform.positionM)+(Vector3.ZERO if next.sceneId=='gh_beacon' else Vector3(24,0,0))
		if previous.y!=0.0 or following.y!=0.0 or previous.z<6.5 or previous.z>9.5 or following.z<6.5 or following.z>9.5: return false
		return (previous.x<=24.0 and following.x>24.0) if old.sceneId=='gh_beacon' else (previous.x>=24.0 and following.x<24.0)
	var a: float
	var b: float
	var z_a: float = 0.0
	var z_b: float = 0.0
	var seam_x: float = 20.0
	var seam_low: float = 6.0
	var seam_high: float = 8.0
	if old.sceneId=='gh_power_room' and next.sceneId=='gh_gate_a':
		a=0.0; b=20.0
	elif old.sceneId=='gh_gate_a' and next.sceneId=='gh_power_room':
		a=20.0; b=0.0
	elif old.sceneId=='gh_gate_a' and next.sceneId=='gh_central_shaft':
		a=20.0; b=40.0; z_b=-1.0; seam_x=40.0
	elif old.sceneId=='gh_central_shaft' and next.sceneId=='gh_gate_a':
		a=40.0; b=20.0; z_a=-1.0; seam_x=40.0
	elif old.sceneId=='gh_central_shaft' and next.sceneId=='gh_lockdown':
		a=40.0; b=64.0; z_a=-1.0; z_b=-1.0; seam_x=64.0
	elif old.sceneId=='gh_lockdown' and next.sceneId=='gh_central_shaft':
		a=64.0; b=40.0; z_a=-1.0; z_b=-1.0; seam_x=64.0
	elif old.sceneId=='gh_gate_b' and next.sceneId=='gh_deep_decon':
		a=0.0; b=24.0; seam_x=24.0; seam_low=7.0; seam_high=9.0
	elif old.sceneId=='gh_deep_decon' and next.sceneId=='gh_gate_b':
		a=24.0; b=0.0; seam_x=24.0; seam_low=7.0; seam_high=9.0
	else: return false
	var previous: Vector3 = P.vector(old.player.transform.positionM)+Vector3(a,0,z_a)
	var following: Vector3 = P.vector(next.player.transform.positionM)+Vector3(b,0,z_b)
	return previous.y==0.0 and following.y==0.0 and absf(previous.x-seam_x)<=0.7 and absf(following.x-seam_x)<=0.7 and previous.distance_to(following)<=0.7 and previous.z>=seam_low and previous.z<=seam_high and following.z>=seam_low and following.z<=seam_high
func _accept_snapshot(s: Dictionary, permit_transition: bool = false, load_request_id: int = 0) -> bool:
	if not P.snapshot(s) or not catalog.manifest.has(s.sceneId):
		_hold('E_REQUIRED_SNAPSHOT')
		return false
	if s.has('shaftServiceEntryPreview') and s.get('shaftServiceEntryPreview')!=true:
		_hold('E_SHAFT_SERVICE_PREVIEW_IDENTITY')
		return false
	if s.get('shaftServiceEntryPreview')==true:
		var t=s.get('entryToken')
		var identity=s.get('shaftLowerService')
		var base=s.get('shaftServiceEntryBase')
		var seq=s.get('shaftServiceEntryLifecycleSequence')
		if s.has('exitEntryPreview') or s.worldId!='grey_hive' or s.sceneId!='gh_central_shaft' or not P.token(t) or not P.same(t,s) or not identity is Dictionary or identity.size()!=2 or identity.get('recipeVersion')!=2 or not identity.get('sceneSourceSha256') is String or not P.context(base) or not P.counter(seq) or (not snapshot.is_empty() and (s.worldEpoch<=snapshot.worldEpoch or not P.same(base,snapshot))):
			_hold('E_SHAFT_SERVICE_PREVIEW_IDENTITY')
			return false
		var hash: String=identity.sceneSourceSha256
		if not exit_entry_preview.is_empty():
			if exit_entry_preview.get('kind')!='shaft' or exit_entry_preview.token!=t or exit_entry_preview.hash!=hash or exit_entry_preview.identity!=identity or not P.same(exit_entry_preview.base,base) or int(seq)<int(exit_entry_preview.sequence):
				_hold('E_SHAFT_SERVICE_PREVIEW_STALE')
				return false
			exit_entry_preview.sequence=int(seq)
			return exit_entry_preview.get('prepared')==true
		exit_entry_preview={'kind':'shaft','identity':identity.duplicate(true),'token':t.duplicate(true),'hash':hash,'base':base.duplicate(true),'sequence':int(seq),'prepared':false,'sent':false,'cancel':false}
		var preview_catalog=Catalog.new()
		if not preview_catalog.build(s.worldId,s.sceneId,hash) or not renderer.prepare_shaft_service_preview(s,preview_catalog.current,hash):
			exit_entry_preview.cancel=true
			_maybe_exit_ready()
			return false
		exit_entry_preview.prepared=true
		status.clear()
		_maybe_exit_ready()
		return true
	if s.has('exitEntryPreview') and s.get('exitEntryPreview')!=true:
		_hold('E_EXIT_PREVIEW_IDENTITY')
		return false
	if s.get('exitEntryPreview')==true:
		var t=s.get('entryToken')
		var hash=s.get('utilitySceneSourceSha256')
		var base=s.get('exitEntryBase')
		var seq=s.get('exitEntryLifecycleSequence')
		if s.worldId!='grey_hive' or s.sceneId!='gh_exit' or not P.token(t) or not P.same(t,s) or not hash is String or hash.is_empty() or not P.context(base) or not P.counter(seq) or (not snapshot.is_empty() and (s.worldEpoch<=snapshot.worldEpoch or not P.same(base,snapshot))):
			_hold('E_EXIT_PREVIEW_IDENTITY')
			return false
		if not exit_entry_preview.is_empty():
			if exit_entry_preview.token!=t or exit_entry_preview.hash!=hash or not P.same(exit_entry_preview.base,base) or int(seq)<int(exit_entry_preview.sequence):
				_hold('E_EXIT_PREVIEW_STALE')
				return false
			exit_entry_preview.sequence=int(seq)
			return exit_entry_preview.get('prepared')==true
		exit_entry_preview={'token':t.duplicate(true),'hash':hash,'base':base.duplicate(true),'sequence':int(seq),'prepared':false,'sent':false,'cancel':false}
		var preview_catalog=Catalog.new()
		if not preview_catalog.build(s.worldId,s.sceneId,hash) or not preview_catalog.contains(P.vector(s.player.transform.positionM)) or not renderer.prepare_exit_preview(s,preview_catalog.current,hash):
			exit_entry_preview.cancel=true
			_maybe_exit_ready()
			return false
		exit_entry_preview.prepared=true
		status.clear()
		_maybe_exit_ready()
		return true
	if not exit_entry_preview.is_empty():
		if P.same(s,exit_entry_preview.base):
			exit_entry_preview.clear()
			exit_preview_journey=false
			exit_postframe_resume.clear()
			_clear_world_loading()
		elif not (permit_transition and P.same(s,exit_entry_preview.token) and typeof(s.get('shaftServiceEntryResumeAfterFrame' if exit_entry_preview.get('kind')=='shaft' else 'exitEntryResumeAfterFrame'))==TYPE_BOOL):
			return false
	var seam_resume = _verified_pair_seam(snapshot,s) and focused and not uncertain and critical.is_empty()
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
		beacon_exit_ready_pending.clear()
		hive_route_ready_pending.clear()
		save_notice.clear()
		aim_pointer_fresh = false
		gameplay_rearm = true
		pending_movement.clear()
		movement_down.clear()
		audio.suspend_world()
		_clear_mouse_movement()
		_clear_mouse_guard()
		# The core transition already neutralized the old epoch. Do not send stale input.
		guard_active = false
		guard_blocked = _key_held(KEY_E)
		pending_actions.clear()
		neutral_pending = false
		for key in ACTION_KEYS:
			if _key_held(key):
				blocked_keys[key] = true
		logical_down.clear()
		var variant_scene: bool = s.worldId=='grey_hive' and s.sceneId in ['gh_lockdown','gh_bio_isolation','gh_deep_decon','gh_beacon','gh_exit']
		var variant_hash = s.get('lockdownSceneSourceSha256') if variant_scene and s.sceneId=='gh_lockdown' else (s.get('utilitySceneSourceSha256') if variant_scene else '')
		if not variant_hash is String or (variant_scene and variant_hash.is_empty()) or not catalog.build(s.worldId,s.sceneId,variant_hash):
			_hold('E_SCENE_RESOURCE')
			return false
		if seam_resume and snapshot.get('hiveRouteNeighbor') is Dictionary and snapshot.hiveRouteNeighbor.get('targetSceneSourceSha256')!=catalog.source_hash: seam_resume=false
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
	if s.worldId=='grey_hive' and s.sceneId=='gh_lockdown' and s.get('lockdownSceneSourceSha256')!=catalog.source_hash:
		_hold('E_SCENE_RESOURCE')
		return false
	if s.worldId=='grey_hive' and s.sceneId in ['gh_bio_isolation','gh_deep_decon','gh_beacon','gh_exit'] and s.get('utilitySceneSourceSha256')!=catalog.source_hash:
		_hold('E_SCENE_RESOURCE')
		return false
	var shaft_service_fit: bool = s.get('shaftLowerService') is Dictionary and not ShaftPresentation.shaft_lift_view(s.get('shaftLift'),s,catalog.current,catalog.source_hash).is_empty()
	if not catalog.contains(P.vector(s.player.transform.positionM)) and not shaft_service_fit and not _mh_shore_position_in_admitted_pair(s,P.vector(s.player.transform.positionM)) and not _cw_early_position_in_admitted_pair(s,P.vector(s.player.transform.positionM)):
		_hold('E_PLAYER_PROJECTION_BOUNDS')
		return false
	if changed:
		_pause_record_clear()
	# Only a validated decrease for the same player and context is shown locally.
	if changed or snapshot.is_empty() or snapshot.player.entityId!=s.player.entityId or snapshot.player.maxHp!=s.player.maxHp:
		damage_feedback_remaining = 0.0
	elif s.player.currentHp>snapshot.player.currentHp or s.player.currentHp<=0:
		damage_feedback_remaining = 0.0
	elif s.player.currentHp<snapshot.player.currentHp and snapshot.player.currentHp>0 and s.get('entryToken')==null and _eligible_without_rearm():
		damage_feedback_remaining = 0.45
	_combat_record_snapshot(s)
	snapshot = s
	if changed:
		last_feedback = ''
	last_snapshot_at = Time.get_ticks_msec()
	if load_request_id>0 and load_request_id==loading_request_id and loading_generation==generation:
		world_loading.set_world_title(Panels.WORLDS.get(s.worldId,'正在进入世界'))
	renderer.commit(s,catalog.current,catalog.source_hash)
	_finish_world_loading(load_request_id)
	if journey_started and s.get('entryToken')!=null:
		_prepare_entry(s.entryToken)
		if seam_resume: seam_rearm_pending = true
	if journey_started and s.player.currentHp<=0 and critical.is_empty():
		_release()
		if panels.mode!='death':
			_show('death')
	return true

func _shaft_service_ready_proof(identity: Dictionary) -> Dictionary:
	if identity.size()!=2 or not P.counter(identity.get('recipeVersion'),2,2) or not identity.get('sceneSourceSha256') is String or identity.sceneSourceSha256.is_empty(): return {}
	return {'shaftLowerService':{'recipeVersion':int(identity.recipeVersion),'sceneSourceSha256':identity.sceneSourceSha256}}
func _maybe_exit_ready() -> void:
	if exit_entry_preview.is_empty() or exit_entry_preview.get('sent')==true or not critical.is_empty() or uncertain or close_attempted: return
	if exit_entry_preview.get('cancel')==true:
		var n: int=maxi(lifecycle_sequence,int(exit_entry_preview.sequence))
		if n>=9007199254740991: _hold('E_COUNTER_EXHAUSTED'); return
		lifecycle_sequence=n+1
		if _request('formal_pause',{'context':exit_entry_preview.base,'commandSequence':lifecycle_sequence},'exit_cancel',{'sequence':lifecycle_sequence},true)>0:
			exit_entry_preview.sent=true
		else: _hold('E_EXIT_PREVIEW_CANCEL')
		return
	if exit_entry_preview.get('prepared')!=true: return
	var t: Dictionary=exit_entry_preview.token
	var remain_paused=not focused
	var args={'token':t,'remainPaused':remain_paused,'exitPreparedSourceSha256':exit_entry_preview.hash}
	if exit_entry_preview.get('kind')=='shaft': args['pairProof']=_shaft_service_ready_proof(exit_entry_preview.identity)
	if _request('formal_scene_ready',args,'ready',{'token':t,'remainPaused':remain_paused},true)>0:
		exit_entry_preview.sent=true
	else:
		_hold('E_EXIT_PREVIEW_READY')

func _maybe_beacon_exit_ready() -> void:
	if not beacon_exit_ready_pending.is_empty() or not exit_entry_preview.is_empty() or not journey_started or not focused or uncertain or close_attempted or not critical.is_empty() or not is_instance_valid(renderer) or not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or status.is_empty() or not P.same(status,snapshot) or status.get('paused')!=false: return
	var v=snapshot.get('beaconExitNeighbor')
	if not v is Dictionary or v.get('ready')!=false or not P.token(v.get('token')) or not v.get('targetSceneSourceSha256') is String or not renderer.beacon_exit_preview_ready(): return
	var token: Dictionary=v.token.duplicate(true)
	var hash: String=v.targetSceneSourceSha256
	var id=_request('formal_scene_ready',{'token':token,'remainPaused':false,'exitPreparedSourceSha256':hash,'pairProof':{'progression':snapshot.progression.duplicate(true),'beacon':snapshot.get('greyHiveBeacon'),'sourceSha':catalog.source_hash}},'beacon_exit_ready',{'token':token,'hash':hash,'expectedCommand':'scene-ready:'+str(int(token.generation))},false)
	if id>0: beacon_exit_ready_pending={'id':id,'token':token}

func _maybe_hive_route_ready() -> void:
	if not hive_route_ready_pending.is_empty() or not exit_entry_preview.is_empty() or not journey_started or not focused or uncertain or close_attempted or not critical.is_empty() or not is_instance_valid(renderer) or not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or status.is_empty() or not P.same(status,snapshot) or status.get('paused')!=false: return
	var v=snapshot.get('hiveRouteNeighbor')
	if not v is Dictionary or v.get('ready')!=false or not P.token(v.get('token')) or not v.get('targetSceneSourceSha256') is String or not renderer.hive_route_preview_ready(): return
	var token: Dictionary=v.token.duplicate(true)
	var hash: String=v.targetSceneSourceSha256
	var proof={'hiveRoute':true,'progression':snapshot.progression.duplicate(true),'sourceSha':catalog.source_hash}
	if v.has('arenaShuttersOpen'):
		if typeof(v.arenaShuttersOpen)!=TYPE_BOOL: return
		proof['arenaShuttersOpen']=v.arenaShuttersOpen
	var id=_request('formal_scene_ready',{'token':token,'remainPaused':false,'exitPreparedSourceSha256':hash,'pairProof':proof},'hive_route_ready',{'token':token,'hash':hash,'expectedCommand':'scene-ready:'+str(int(token.generation))},false)
	if id>0: hive_route_ready_pending={'id':id,'token':token}

func _maybe_exit_resume() -> void:
	if exit_postframe_resume.is_empty() or not exit_entry_preview.is_empty() or not focused or uncertain or close_attempted or not critical.is_empty() or snapshot.is_empty() or status.is_empty(): return
	if panels.visible:
		exit_postframe_resume.clear()
		return
	var source_matches: bool = catalog.source_hash==exit_postframe_resume.hash if exit_postframe_resume.get('kind')=='shaft' else snapshot.get('utilitySceneSourceSha256')==exit_postframe_resume.hash
	if not P.same(snapshot,exit_postframe_resume.context) or not source_matches or not P.same(status,snapshot) or not status.paused or not renderer.dense_region_ready(): return
	if not _advance_lifecycle(): return
	exit_postframe_resume.clear()
	if _request('formal_resume',{'context':_context(),'commandSequence':lifecycle_sequence},'resume',{'sequence':lifecycle_sequence},true)<=0:
		_hold('E_EXIT_PREVIEW_RESUME')
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
	if not renderer.dense_region_ready():
		_hold('E_DENSE_RESOURCE_NOT_READY')
		return
	var t = ready_pending.duplicate(true)
	var token_key = str([t.generation,t.worldId,t.sceneId,t.worldEpoch])
	if ready_sent.has(token_key):
		return
	var remain_paused = not focused
	var ready_args: Dictionary={'token':t,'remainPaused':remain_paused}
	var ready_context: Dictionary={'token':t,'remainPaused':remain_paused}
	var proposal=snapshot.get('shaftLowerServiceProposal')
	if proposal is Dictionary and renderer.prepare_shaft_service_proposal():
		ready_args['pairProof']=_shaft_service_ready_proof(proposal)
		ready_context['shaftServiceExpected']=proposal.duplicate(true)
	if _request('formal_scene_ready',ready_args,'ready',ready_context,true)>0:
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
		_open_paused('dialogue',{'text':'原来门一直不止三扇。'})

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

func _continue_slot_row(row, owner: Dictionary) -> void:
	if not is_instance_valid(panels) or not panels.visible or panels.mode!='slots' or panels.page_owner!=owner: return
	var valid = not panels.pending and not uncertain and not close_attempted and critical.is_empty() and is_instance_valid(rpc) and rpc.authenticated and owner.get('generation')==generation and owner.get('panelGeneration')==panel_generation and owner.get('context',{})==_context() and owner.get('slotConnection')==rpc.connection_generation and _save_scope_current() and slot_list_state=='list' and not save_reads.has('formal_list_save_slots') and row is Dictionary and P.ident(row.get('slotId')) and P.counter(row.get('updatedAtMs')) and row.get('valid')==true and P.rows(current_slots,128) and P.unique(current_slots,'slotId')
	if valid:
		var matches = current_slots.filter(func(item): return item.get('slotId')==row.slotId)
		valid = matches.size()==1 and matches[0]==row and _slot_available(row.slotId)
	if not valid:
		panels.set_message('记录状态已变化，请重新选择或刷新列表。')
		return
	_begin_journey('formal_continue_slot',{'slotId':row.slotId})
func _writable_slot_params(value) -> bool:
	if not value is Dictionary or value.get('create')!=false or not _slot_available(value.get('slotId'),true):
		return false
	for row in current_slots:
		if row.slotId==value.slotId:
			return value.get('displayName')==row.displayName
	return false

func _named_save_current(choice, owner: Dictionary, expected_page: String) -> bool:
	if not is_instance_valid(panels) or not panels.visible or panels.mode!=expected_page or panels.page_owner!=owner or panels.pending or not journey_started or not focused or uncertain or close_attempted or not critical.is_empty() or not ticket.is_empty() or not is_instance_valid(rpc) or not rpc.authenticated: return false
	if not P.snapshot(snapshot) or snapshot.player.currentHp<=0 or snapshot.get('entryToken')!=null or snapshot.get('exitEntryPreview')==true or snapshot.get('shaftServiceEntryPreview')==true or not exit_entry_preview.is_empty() or not ready_pending.is_empty() or not P.status(status) or not status.paused or not P.same(status,snapshot) or not P.context(owner.get('context')) or not P.same(owner.context,snapshot): return false
	if not choice is Dictionary or not choice.get('operation') in ['save_slot','return_slot'] or not choice.get('row') is Dictionary: return false
	if expected_page=='named_save_confirm' and (choice.get('connection')!=rpc.connection_generation or owner.get('namedSave')!=choice): return false
	var row: Dictionary = choice.row
	if not P.ident(row.get('slotId')) or typeof(row.get('displayName'))!=TYPE_STRING or not P.counter(row.get('updatedAtMs')) or row.get('valid')!=true or row.get('readOnly')!=false: return false
	if not _save_scope_current() or slot_list_state!='list' or save_reads.has('formal_list_save_slots') or not P.rows(current_slots,128) or not P.unique(current_slots,'slotId'): return false
	var matches = current_slots.filter(func(item): return item.get('slotId')==row.slotId)
	return matches.size()==1 and matches[0]==row and _writable_slot_params({'slotId':row.slotId,'displayName':row.displayName,'create':false})

func _named_save_prepare(value, owner: Dictionary) -> void:
	if not _named_save_current(value,owner,'save'):
		if panels.visible and panels.mode=='save' and panels.page_owner==owner: panels.set_message('记录状态已变化，请返回重新选择。')
		return
	var choice: Dictionary = value.duplicate(true)
	choice['connection'] = rpc.connection_generation
	_show('named_save_confirm',{'namedSave':choice},false,false)

func _named_save_confirm(value, owner: Dictionary) -> void:
	if not panels.visible or panels.mode!='named_save_confirm' or panels.page_owner!=owner: return
	if not _named_save_current(value,owner,'named_save_confirm'):
		if not uncertain and not close_attempted and rpc.authenticated:
			_show('save',{},false,false)
			panels.set_message('记录状态已变化，请返回重新选择。')
		return
	var params = {'slotId':value.row.slotId,'displayName':value.row.displayName,'create':false}
	var operation: String = value.operation
	_show('save',{},false,false)
	if operation=='save_slot':
		_save_request('formal_save_slot',params,false)
	else:
		_request('formal_return_slot',params,'return',{},true)
func _publish_saves(rebuild: bool = true) -> void:
	if not is_instance_valid(panels):
		return
	var current = _save_scope_current()
	var effective_default = default_save_state if current else 'unknown'
	var effective_slots = slot_list_state if current else 'unknown'
	var effective_rows: Array = current_slots if current else []
	panels.set_save_availability(effective_default,effective_slots,effective_rows)
	var rebuild_allowed = rebuild and panels.visible and panels.mode in ['slots','save'] and critical.is_empty() and not uncertain and not panels.pending
	_trace_frontend('computed_publish_saves',{'current_scope':current,'default_state':effective_default,'slot_state':effective_slots,'slot_count':effective_rows.size(),'rebuild_allowed':rebuild_allowed})
	if rebuild_allowed:
		var page: String = panels.mode
		var focused_slot = panels.focused_continue_slot() if page=='slots' else ''
		var text = panels.status.text if is_instance_valid(panels.status) else ''
		_show(page,{},false,false,page=='save')
		panels.set_message(text)
		if not focused_slot.is_empty(): panels.focus_continue_slot(focused_slot)
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
func _journey_failure_page(purpose: String, method: String) -> String:
	if purpose!='journey': return ''
	if method=='formal_new': return 'start'
	if method in ['formal_continue','formal_continue_slot']: return 'slots'
	return ''

func _journey_failure_notice(page: String, reason: String) -> String:
	var message = '未能开始新旅程。请查看提示后重新选择。' if page=='start' else '未能继续此记录。请选择其他记录，或刷新列表。'
	return message+(' '+reason if not reason.is_empty() else '')

func _failed(_id_value: int, method: String, code: String, association: Dictionary) -> void:
	if association.get('purpose')=='pair_visual_hold' and pair_visual_hold_pending.get('id')==_id_value:
		pair_visual_hold_pending.clear()
		pair_visual_hold_receipt.clear()
	if association.get('purpose')=='beacon_exit_ready' and beacon_exit_ready_pending.get('id')==_id_value: beacon_exit_ready_pending.clear()
	if association.get('purpose')=='hive_route_ready' and hive_route_ready_pending.get('id')==_id_value: hive_route_ready_pending.clear()
	_finish_world_loading(_id_value)
	if close_confirmed:
		return
	if method=='session_close':
		if not _close_current(_id_value,method,association):
			_trace_frontend('close_disposition',{'reason':'close_stale_association'})
			return
		close_pending.clear()
		_trace_frontend('close_disposition',{'reason':'close_unknown' if code=='UNKNOWN_MUTATION' else 'close_failed'})
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
	if association.get('purpose')=='save':
		_set_save_notice('保存结果未知，未重复提交' if code=='UNKNOWN_MUTATION' else '保存未确认：'+_localized(code))
	if method=='formal_submit_action':
		_combat_record_failure(_id_value,code,association)
	if method=='formal_pause':
		_pause_record_failure(_id_value,association)
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
		var recovery_page = _journey_failure_page(str(association.get('purpose','')),method)
		if not recovery_page.is_empty() and (uncertain or close_attempted): return
		if not recovery_page.is_empty(): _feedback(_journey_failure_notice(recovery_page,last_feedback))
		_show(recovery_page if not recovery_page.is_empty() else panels.mode,{},false,recovery_page.is_empty())
		panels.set_message(last_feedback)
func _hold(code: String) -> void:
	_clear_world_loading()
	if close_attempted:
		_trace_frontend('close_disposition',{'reason':'close_held'})
	audio.ui_cue('error',focused)
	uncertain = true
	critical = ''
	_release()
	_show('error')
	panels.set_message(_localized(code))
func _set_world_loading_art(world: String) -> void:
	if not world in ['return_station','grey_hive','mist_harbor','clockworks']:
		world_loading.set_loading_art(null)
		return
	if not loading_art_cache.has(world):
		var texture: Texture2D = null
		if world=='grey_hive':
			var loaded = load('res://assets/world_loading/grey_hive_keyart_20261030_001.png')
			if loaded is Texture2D and loaded.get_width()>0 and loaded.get_height()>0: texture = loaded
		else:
			var paths = {'return_station':'res://scripts/world/rs_exterior_backplate_visual.gd','mist_harbor':'res://scripts/world/mh_fog_pier_exterior_backplate_visual.gd','clockworks':'res://scripts/world/cw_entry_foundry_exterior_backplate_visual.gd'}
			var provider_script = load(paths[world])
			if provider_script is Script:
				var provider = provider_script.new()
				provider._load_once()
				if provider._texture is Texture2D: texture = provider._texture
		loading_art_cache[world] = texture
	world_loading.set_loading_art(loading_art_cache[world])
func _finish_world_loading(id: int) -> void:
	if id>0 and id==loading_request_id and loading_generation==generation:
		_clear_world_loading()
func _clear_world_loading() -> void:
	if is_instance_valid(panels):
		panels.set_world_loading_active(false)
	loading_request_id = 0
	loading_generation = -1
	if is_instance_valid(world_loading):
		world_loading.set_loading_art(null)
		world_loading.end_loading()
func _set_save_notice(text: String) -> void:
	save_notice.clear()
	if is_instance_valid(rpc) and P.snapshot(snapshot) and P.context(_context()):
		save_notice = {'text':text,'generation':generation,'connection':rpc.connection_generation,'context':_context().duplicate(true)}
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
	_pause_record_clear()
	_combat_record_finish('closure')
	_trace_sink_close()
	if is_instance_valid(audio):
		audio.stop_all()

func _close_current(id: int, method: String, association: Dictionary) -> bool:
	return method=='session_close' and not close_pending.is_empty() and id==close_pending.id and close_pending.generation==generation and close_pending.connectionGeneration==rpc.connection_generation and association.get('connectionGeneration')==close_pending.connectionGeneration and association.get('purpose')=='close' and _current(association) and _current(close_pending)
func _submit_close() -> void:
	if close_attempted or not rpc.authenticated or uncertain or not critical.is_empty():
		_trace_frontend('close_disposition',{'reason':'submit_blocked'})
		return
	_pause_record_clear()
	_combat_record_finish('closure')
	# Release existing Guard/input/audio work before the once-only close barrier.
	_release()
	close_attempted = true
	close_submitting = true
	var connection = rpc.connection_generation
	var journey = generation
	var context = _context().duplicate(true)
	_trace_frontend('close_disposition',{'reason':'submit_admitted'})
	var id = _request('session_close',{},'close',{'connectionGeneration':connection},true)
	close_submitting = false
	if id>0 and rpc.authenticated and rpc.connection_generation==connection and generation==journey and rpc.pending.has(id):
		close_pending = {'id':id,'generation':journey,'connectionGeneration':connection,'context':context}
		_trace_frontend('close_disposition',{'reason':'submit_queued'})
		_show('waiting',{},true)
	elif rpc.authenticated:
		_trace_frontend('close_disposition',{'reason':'submit_rejected'})
		_hold('E_CLOSE_SUBMISSION')
	# A synchronous drop already published its usable disconnected error page.

func _trace_start() -> void:
	trace_enabled = OS.get_environment('WUXIAN_V1_INPUT_TRACE')=='1'
	combat_record_enabled = trace_enabled
	pause_record_enabled = trace_enabled
	pause_record_started_ms = Time.get_ticks_msec()
	hud.pad_trace_enabled = trace_enabled
	if not trace_enabled:
		return
	trace_started_ms = Time.get_ticks_msec()
	trace_next_ms = trace_started_ms
	_trace_sink_start()
	_trace_write('start','','',0)

# Pause observations are independent of the startup input trace and its sink.
# Association metadata remains local to the existing RPC pending entry.
func _pause_record_forget() -> void:
	pause_record_active = 0
	pause_record_context.clear()
	pause_record_generation = -1
	pause_record_connection = -1
	pause_record_pending_id = 0
func _pause_record_current() -> bool:
	if not pause_record_enabled or pause_record_exhausted or not journey_started or not is_instance_valid(rpc) or not is_instance_valid(panels) or not P.context(snapshot) or not P.counter(generation) or not P.counter(rpc.connection_generation):
		return false
	if pause_record_active>0:
		return generation==pause_record_generation and rpc.connection_generation==pause_record_connection and P.same(snapshot,pause_record_context)
	return pause_record_episodes<2
func _pause_record_emit(stage: String, origin: String, fields: Dictionary = {}) -> bool:
	if not _pause_record_current() or stage not in ['focus_out','focus_in','escape_press','ui_pause_intent','page_blocked','page_request','pause_request','pause_receipt','page_display']:
		return false
	if origin not in ['focus_out','focus_in','escape','ui_pause','unknown']:
		origin = 'unknown'
	var elapsed: int = Time.get_ticks_msec()-pause_record_started_ms
	if elapsed<0:
		return false
	var window = get_window()
	var panel = panels.mode if panels.mode in ['start','waiting','pause','settings','inventory','character','capability','mission','archive','save','slots','map','new_confirm','exit_confirm','enhancement','death','error','dialogue','baizhi'] else 'unknown'
	var valid_snapshot: bool = P.snapshot(snapshot)
	var valid_status: bool = P.status(status) and P.same(status,snapshot)
	var row: Dictionary = {'stage':stage,'origin':origin,'episode':pause_record_active if pause_record_active>0 else null,'elapsed_ms':elapsed,'focused':focused,'window_focus':window.has_focus() if is_instance_valid(window) else null,'panel':panel,'eligible':_eligible() if valid_snapshot and valid_status else null,'paused':status.paused if valid_status else null,'world_id':snapshot.worldId if snapshot.worldId in ['return_station','grey_hive','mist_harbor','clockworks'] else null,'scene_id':snapshot.sceneId if Content.SCENE_NAMES.has(snapshot.sceneId) else null,'world_epoch':snapshot.worldEpoch,'generation':generation,'connection_generation':rpc.connection_generation}
	if P.counter(fields.get('request_id')):
		row.request_id = int(fields.request_id)
	if fields.get('result') in ['queued','not_queued','accepted','rejected','unknown']:
		row.result = fields.result
	if fields.get('route') in ['receipt','already_paused','back','unknown']:
		row.route = fields.route
	if fields.get('page') in ['pause','settings','inventory','character','capability','mission','archive','save','map','new_confirm','exit_confirm','enhancement','unknown']:
		row.page = fields.page
	var line: String = 'WUXIAN_GH_PAUSE_RECORD:'+JSON.stringify(row)
	var bytes: int = line.to_utf8_buffer().size()+2
	if pause_record_count>=16 or pause_record_bytes+bytes>8192:
		pause_record_exhausted = true
		_pause_record_forget()
		return false
	pause_record_count += 1
	pause_record_bytes += bytes
	print(line)
	if pause_record_count>=16:
		pause_record_exhausted = true
		_pause_record_forget()
	return true
func _pause_record_event(stage: String, origin: String) -> void:
	_pause_record_emit(stage,origin)
func _pause_record_begin() -> bool:
	if not _pause_record_current():
		return false
	if pause_record_active==0:
		pause_record_episodes += 1
		pause_record_active = pause_record_episodes
		pause_record_context = _context()
		pause_record_generation = generation
		pause_record_connection = rpc.connection_generation
	return true
func _pause_record_open(page: String, origin: String) -> void:
	if _pause_record_begin():
		_pause_record_emit('page_request',origin,{'page':page})
func _pause_record_prepare(extra: Dictionary) -> void:
	if not _pause_record_current():
		return
	if pause_record_active==0:
		_pause_record_open(extra.get('page','enhancement'),'unknown')
	if pause_record_active==0:
		return
	var origin = extra.get('pause_record_origin','unknown')
	extra['pause_record_origin'] = origin if origin in ['focus_out','escape','ui_pause','unknown'] else 'unknown'
	extra['pause_record_episode'] = pause_record_active
	extra['pause_record_connection'] = pause_record_connection
func _pause_record_submitted(id: int, extra: Dictionary) -> void:
	if not _pause_record_current() or pause_record_active==0 or extra.get('pause_record_episode')!=pause_record_active or extra.get('pause_record_connection')!=pause_record_connection:
		return
	var origin = extra.get('pause_record_origin','unknown')
	if id>0:
		pause_record_pending_id = id
	_pause_record_emit('pause_request',origin,{'request_id':id,'result':'queued' if id>0 else 'not_queued'})
	if id<=0:
		_pause_record_forget()
func _pause_record_matches(id: int, association: Dictionary) -> bool:
	return _pause_record_current() and pause_record_active>0 and id>0 and id==pause_record_pending_id and association.get('generation')==pause_record_generation and association.get('pause_record_episode')==pause_record_active and association.get('pause_record_connection')==pause_record_connection and association.get('pause_record_origin') in ['focus_out','escape','ui_pause','unknown'] and association.get('context') is Dictionary and P.same(association.context,pause_record_context)
func _pause_record_response(id: int, receipt: Dictionary, association: Dictionary) -> Dictionary:
	if not _pause_record_matches(id,association):
		return {}
	var accepted: bool = P.success(receipt)
	var result = {'route':'receipt','origin':association.pause_record_origin,'request_id':id,'episode':pause_record_active}
	_pause_record_emit('pause_receipt',association.pause_record_origin,{'request_id':id,'result':'accepted' if accepted else 'rejected'})
	pause_record_pending_id = 0
	if not accepted:
		_pause_record_forget()
		return {}
	return result
func _pause_record_failure(id: int, association: Dictionary) -> void:
	if _pause_record_matches(id,association):
		_pause_record_emit('pause_receipt',association.pause_record_origin,{'request_id':id,'result':'unknown'})
		_pause_record_forget()
func _pause_record_clear() -> void:
	if pause_record_pending_id>0 and _pause_record_current():
		_pause_record_emit('pause_receipt','unknown',{'request_id':pause_record_pending_id,'result':'unknown'})
	_pause_record_forget()
func _pause_record_display(observation: Dictionary) -> void:
	if not _pause_record_begin():
		return
	var route = observation.get('route','unknown')
	var origin = observation.get('origin','unknown')
	if route not in ['receipt','already_paused','back']:
		route = 'unknown'
		origin = 'unknown'
	var fields: Dictionary = {'route':route,'page':'pause'}
	if route=='receipt':
		if observation.get('episode')!=pause_record_active or not P.counter(observation.get('request_id'),1):
			fields.route = 'unknown'
			origin = 'unknown'
		else:
			fields.request_id = observation.request_id
	_pause_record_emit('page_display',origin,fields)

# These records go to the existing owner-selected Godot log, not the separately
# capped frontend sidecar. They do not extend or consume the older trace limits.
func _combat_record_drop() -> void:
	combat_record_finished = true
	combat_record_clicks.clear()
	combat_record_seen_events.clear()
	combat_record_dispatch_click = 0
	combat_record_context.clear()
func _combat_record_emit(stage: String, click: int, fields: Dictionary = {}) -> bool:
	if not combat_record_enabled or combat_record_finished or combat_record_started_ms<0:
		return false
	var elapsed: int = Time.get_ticks_msec()-combat_record_started_ms
	if elapsed<0 or elapsed>=120000 or combat_record_count>=32:
		_combat_record_drop()
		return false
	if stage not in ['click_received','click_admission','input_sample','action_submitted','action_response','action_error','event','player_hp','end']:
		return false
	var row: Dictionary = {'stage':stage,'click':click if click>0 else null,'elapsed_ms':elapsed,'world_id':'grey_hive','scene_id':'gh_entry_maintenance','world_epoch':combat_record_context.get('worldEpoch'),'generation':combat_record_generation,'connection_generation':combat_record_connection}
	var allowed = ['eligible','gui_hover','queued','input_sequence','aim_x','aim_z','input_rpc_id','action_request_id','action_rpc_id','response','error_code','event_id','event_kind','target_id','source_id','reason','correlation','hp_before','hp_after','stop_reason']
	for key in allowed:
		if not fields.has(key):
			continue
		var value = fields[key]
		if value==null or typeof(value)==TYPE_BOOL or typeof(value)==TYPE_INT or typeof(value)==TYPE_FLOAT and is_finite(value) or typeof(value)==TYPE_STRING and P.ident(value):
			row[key] = value
	var line: String = 'WUXIAN_GH_COMBAT_RECORD:'+JSON.stringify(row)
	# Count a two-byte line ending even where Godot writes only LF.
	var bytes: int = line.to_utf8_buffer().size()+2
	if combat_record_bytes+bytes>16384:
		_combat_record_drop()
		return false
	combat_record_count += 1
	combat_record_bytes += bytes
	print(line)
	if combat_record_count>=32:
		_combat_record_drop()
		return false
	return true
func _combat_record_finish(reason: String) -> void:
	if not combat_record_enabled or combat_record_finished or combat_record_started_ms<0:
		return
	_combat_record_emit('end',0,{'stop_reason':reason})
	_combat_record_drop()
func _combat_record_live() -> bool:
	if not combat_record_enabled or combat_record_finished or combat_record_started_ms<0:
		return false
	var elapsed: int = Time.get_ticks_msec()-combat_record_started_ms
	if elapsed<0 or elapsed>=120000:
		_combat_record_drop()
		return false
	if close_attempted or close_confirmed:
		_combat_record_finish('closure')
		return false
	if not is_instance_valid(rpc) or rpc.connection_generation!=combat_record_connection or generation!=combat_record_generation or not P.same(snapshot,combat_record_context):
		_combat_record_finish('context_changed')
		return false
	if not snapshot.get('player') is Dictionary or not P.counter(snapshot.player.get('currentHp'),0,4294967295) or snapshot.player.currentHp<=0:
		_combat_record_finish('death_or_invalid')
		return false
	return true
func _combat_record_received(event: InputEventMouseButton) -> void:
	if not combat_record_enabled or combat_record_finished:
		return
	if combat_record_started_ms<0:
		if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or snapshot.sceneId!='gh_entry_maintenance' or snapshot.player.currentHp<=0 or close_attempted or close_confirmed or not is_instance_valid(rpc) or not P.counter(rpc.connection_generation) or not P.counter(generation):
			return
		combat_record_context = _context()
		combat_record_generation = generation
		combat_record_connection = rpc.connection_generation
		combat_record_last_hp = int(snapshot.player.currentHp)
		if is_instance_valid(event_view) and P.counter(event_view.cursor):
			combat_record_cursor = int(event_view.cursor)
		combat_record_started_ms = Time.get_ticks_msec()
	if not _combat_record_live() or combat_record_clicks.size()>=3:
		return
	var click: int = combat_record_clicks.size()+1
	combat_record_clicks.append({'click':click,'event':event,'edge':null,'action_request_id':null,'action_rpc_id':null})
	var hovered = get_viewport().gui_get_hovered_control()!=null
	_combat_record_emit('click_received',click,{'eligible':_eligible(),'gui_hover':hovered,'queued':null})
func _combat_record_admission(event: InputEventMouseButton, eligible: bool, hovered, before: int) -> void:
	if not _combat_record_live():
		return
	for entry in combat_record_clicks:
		if entry.event!=null and is_same(entry.event,event):
			entry.event = null
			var queued: bool = eligible and hovered==false and pending_actions.size()==before+1 and pending_actions.back().get('kind')=='primaryAttack'
			if queued:
				entry.edge = pending_actions.back()
			_combat_record_emit('click_admission',entry.click,{'eligible':eligible,'gui_hover':hovered,'queued':queued})
			return
func _combat_record_sample(sample: Dictionary, submitted: int) -> void:
	if not _combat_record_live():
		return
	for entry in combat_record_clicks:
		if entry.edge==null:
			continue
		var found = false
		for edge in pending_actions:
			if is_same(edge,entry.edge):
				found = true
				break
		if found:
			if not _combat_record_emit('input_sample',entry.click,{'input_sequence':sample.seq,'aim_x':sample.aimX,'aim_z':sample.aimZ,'input_rpc_id':submitted,'queued':submitted>0}):
				return
func _combat_record_dispatch_map() -> Dictionary:
	var dispatch: Dictionary = {}
	if not _combat_record_live():
		return dispatch
	for index in range(pending_actions.size()):
		for entry in combat_record_clicks:
			if entry.edge!=null and is_same(entry.edge,pending_actions[index]):
				dispatch[index] = entry.click
				break
	return dispatch
func _combat_record_action(kind: String, action: Dictionary, submitted: int) -> void:
	if kind!='primaryAttack' or combat_record_dispatch_click<=0 or not _combat_record_live():
		return
	for entry in combat_record_clicks:
		if entry.click==combat_record_dispatch_click:
			entry.edge = null
			entry.action_request_id = action.requestId
			entry.action_rpc_id = submitted
			_combat_record_emit('action_submitted',entry.click,{'action_request_id':action.requestId,'action_rpc_id':submitted,'queued':submitted>0})
			return
func _combat_record_response(id: int, receipt: Dictionary, association: Dictionary) -> void:
	if not _combat_record_live() or not association.get('context') is Dictionary or not P.same(association.context,combat_record_context):
		return
	for entry in combat_record_clicks:
		if id>0 and entry.action_rpc_id==id:
			var error = receipt.get('errorCode')
			_combat_record_emit('action_response',entry.click,{'action_request_id':entry.action_request_id,'action_rpc_id':id,'response':'accepted' if P.success(receipt) else 'rejected','error_code':error if P.ident(error) else null})
			return
func _combat_record_failure(id: int, code: String, association: Dictionary) -> void:
	if not _combat_record_live() or not association.get('context') is Dictionary or not P.same(association.context,combat_record_context):
		return
	for entry in combat_record_clicks:
		if id>0 and entry.action_rpc_id==id:
			_combat_record_emit('action_error',entry.click,{'action_request_id':entry.action_request_id,'action_rpc_id':id,'response':'unknown','error_code':code if P.ident(code) else null})
			return
func _combat_record_events(rows) -> void:
	if not _combat_record_live() or not P.rows(rows,2048):
		return
	for event in rows:
		if not event is Dictionary or not P.counter(event.get('eventId'),1) or event.eventId<=combat_record_cursor or event.get('worldEpoch')!=combat_record_context.worldEpoch or combat_record_seen_events.has(event.eventId):
			continue
		var feedback = event.get('combatFeedback')
		if not feedback is Dictionary or feedback.get('worldId')!='grey_hive' or feedback.get('sceneId')!='gh_entry_maintenance':
			continue
		var kind = event.get('kind')
		if not (kind=='Hit' and feedback.get('outcome')=='enemy_hit' and feedback.get('sourceId')=='player' or kind=='ActionRejected' and feedback.get('outcome')=='rejected' and feedback.get('sourceId')=='player' or kind=='Damaged' and feedback.get('outcome')=='player_hurt' and feedback.get('targetId')=='player'):
			continue
		var action_id = feedback.get('requestId') if P.counter(feedback.get('requestId'),1) else null
		var click = 0
		if kind!='Damaged' and action_id!=null:
			for entry in combat_record_clicks:
				if entry.action_rpc_id!=null and entry.action_rpc_id>0 and entry.action_request_id==action_id:
					click = entry.click
					break
			if click==0:
				continue
		var target = feedback.get('targetId')
		var source = feedback.get('sourceId')
		var reason = feedback.get('reason')
		combat_record_seen_events[event.eventId] = true
		if not _combat_record_emit('event',click,{'event_id':int(event.eventId),'event_kind':kind,'action_request_id':action_id,'target_id':target if P.ident(target) else null,'source_id':source if P.ident(source) else null,'reason':reason if P.ident(reason) else null,'correlation':'matched' if click>0 else 'unknown'}):
			return
func _combat_record_snapshot(next_snapshot: Dictionary) -> void:
	if not _combat_record_live():
		return
	if not P.same(next_snapshot,combat_record_context):
		_combat_record_finish('scene_changed')
		return
	var hp = next_snapshot.player.currentHp
	if P.counter(hp,0,4294967295):
		if hp<combat_record_last_hp:
			_combat_record_emit('player_hp',0,{'hp_before':combat_record_last_hp,'hp_after':int(hp),'correlation':'unknown'})
		combat_record_last_hp = int(hp)
		if hp<=0:
			_combat_record_finish('death')

func _trace_live(now: int) -> bool:
	if now-trace_started_ms>=120000 or trace_records>=223:
		_trace_stop('duration' if now-trace_started_ms>=120000 else 'records',now)
		return false
	return true
func _trace_stop(reason: String, now: int) -> void:
	trace_enabled = false
	_trace_sink_close()
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
	if event is InputEventMouseButton and event.button_index==MOUSE_BUTTON_LEFT and tag=='received':
		_trace_pad('raw','',('down' if event.pressed else 'up'),{'pressed':event.pressed})
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
		'physical_zero':null,'physical_recognized':null,'resolved_zero':null,'resolved_recognized':null,
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
		row.physical_zero = event.physical_keycode==0
		row.physical_recognized = event.physical_keycode in TRACE_KEYS
		row.resolved_zero = resolved==0
		row.resolved_recognized = resolved in TRACE_KEYS
		row.resolved = resolved if resolved in TRACE_KEYS else 0
		row.pressed = event.pressed
		row.echo = event.echo
		row.blocked_key = blocked_keys.has(resolved)
		if tag=='ineligible_or_blocked':
			row.tag = 'ineligible' if observed_eligible==false else 'blocked'
	var encoded = JSON.stringify(row)
	if encoded.to_utf8_buffer().size()>2048 or trace_records>=224:
		trace_enabled = false
		_trace_sink_close()
		return
	trace_records += 1
	print('WUXIAN_INPUT_TRACE:'+encoded)

func _trace_save_flow(stage: String, method: String, request_id: int, receipt: Dictionary = {}, state: Dictionary = {}) -> void:
	# Passive facts only; queue admission is not server success or save durability.
	if not trace_enabled or trace_save_records>=32 or trace_records>=223:
		return
	var elapsed: int = Time.get_ticks_msec()-trace_started_ms
	if elapsed<0 or elapsed>=120000 or stage not in ['ui_intent_admitted','request_queued','snapshot_applied'] or method not in ['formal_continue','formal_save']:
		return
	if request_id<0 or request_id>9007199254740991 or stage!='ui_intent_admitted' and request_id==0:
		return
	var row: Dictionary = {'stage':stage,'method':method,'request_id':request_id,'elapsed_ms':elapsed}
	var connection = rpc.get('connection_generation') if is_instance_valid(rpc) else null
	for key in ['generation','connection_generation']:
		var value = generation if key=='generation' else connection
		if typeof(value)==TYPE_INT and value>=0 and value<=9007199254740991:
			row[key] = value
	if stage=='snapshot_applied':
		for key in ['applied','alreadyApplied']:
			if typeof(receipt.get(key))==TYPE_BOOL:
				row['already_applied' if key=='alreadyApplied' else 'applied'] = receipt[key]
		if typeof(state.get('worldId'))==TYPE_STRING and state.worldId in ['return_station','grey_hive','mist_harbor','clockworks']:
			row['world'] = state.worldId
		if typeof(state.get('sceneId'))==TYPE_STRING and Content.SCENE_NAMES.has(state.sceneId):
			row['scene'] = state.sceneId
		if state.has('checkpointId') and (state.checkpointId==null or typeof(state.checkpointId)==TYPE_STRING):
			row['checkpoint_present'] = state.checkpointId!=null
		var player = state.get('player')
		if player is Dictionary:
			var hp = player.get('currentHp')
			if P.counter(hp,0,4294967295):
				row['hp'] = int(hp)
			var transform = player.get('transform')
			var position = transform.get('positionM') if transform is Dictionary else null
			if position is Dictionary:
				for axis in ['x','y','z']:
					var coordinate = position.get(axis+'M')
					if (typeof(coordinate)==TYPE_INT or typeof(coordinate)==TYPE_FLOAT) and is_finite(float(coordinate)) and absf(float(coordinate))<=1000000.0:
						row[axis] = coordinate
		var progression = state.get('progression')
		var sequence = progression.get('eventSeq') if progression is Dictionary else null
		if P.counter(sequence,0,9007199254740991):
			row['event_seq'] = int(sequence)
	trace_records += 1
	trace_save_records += 1
	var line: String = 'WUXIAN_SAVE_FLOW_TRACE:'+JSON.stringify(row)
	print(line)
	_trace_sink_write(line)

func _trace_sink_destination() -> String:
	return OS.get_environment('WUXIAN_GH_FRONTEND_OBSERVATION_PATH')
func _trace_storage_open(path: String):
	return FileAccess.open(path,FileAccess.READ_WRITE)
func _trace_sink_start() -> void:
	if not trace_enabled or trace_sink_attempted:
		return
	trace_sink_attempted = true
	var path: String = _trace_sink_destination()
	if not path.is_absolute_path() or path.contains('://') or path.get_file()!='frontend-live-observation.jsonl' or path.replace('\\','/')!=path.simplify_path():
		return
	var handle = _trace_storage_open(path)
	if handle==null:
		return
	if handle.get_length()!=0 or handle.get_position()!=0 or handle.get_error()!=OK:
		handle.close()
		return
	trace_sink = handle
func _trace_sink_close() -> void:
	if trace_sink!=null:
		trace_sink.close()
		trace_sink = null
func _trace_sink_write(line: String) -> void:
	if trace_sink==null:
		return
	var elapsed: int = Time.get_ticks_msec()-trace_started_ms
	var bytes: PackedByteArray = (line+'\n').to_utf8_buffer()
	if not trace_enabled or elapsed<0 or elapsed>=120000 or trace_sink_records>=32 or trace_sink_bytes+bytes.size()>65536:
		_trace_sink_close()
		return
	trace_sink.store_buffer(bytes)
	if trace_sink.get_error()!=OK:
		_trace_sink_close()
		return
	trace_sink.flush() # Requested flush is not OS durability or exclusive ownership proof.
	if trace_sink.get_error()!=OK:
		_trace_sink_close()
		return
	trace_sink_records += 1
	trace_sink_bytes += bytes.size()
func _trace_frontend(stage: String, facts: Dictionary) -> void:
	if not trace_enabled or trace_sink==null or trace_records>=223 or trace_sink_records>=32:
		return
	var elapsed: int = Time.get_ticks_msec()-trace_started_ms
	if elapsed<0 or elapsed>=120000 or stage not in ['computed_publish_saves','close_disposition']:
		return
	var row: Dictionary = {'stage':stage,'elapsed_ms':elapsed}
	if stage=='computed_publish_saves':
		for key in ['current_scope','rebuild_allowed']:
			if typeof(facts.get(key))!=TYPE_BOOL:
				return
			row[key] = facts[key]
		if facts.get('default_state') not in ['unknown','available','unavailable','pending','invalid','error'] or facts.get('slot_state') not in ['unknown','list','empty','pending','invalid','error'] or typeof(facts.get('slot_count'))!=TYPE_INT or facts.slot_count<0 or facts.slot_count>128:
			return
		for key in ['default_state','slot_state','slot_count']:
			row[key] = facts[key]
	else:
		if facts.get('reason') not in ['client_quit_disconnected','notification_blocked','confirmation_requested','notification_no_confirmed_save','close_stale_association','close_confirmed_quit','close_invalid_reply','close_disconnected','close_failed','close_unknown','close_held','submit_blocked','submit_admitted','submit_queued','submit_rejected']:
			return
		row['reason'] = facts.reason
	var line: String = 'WUXIAN_FRONTEND_TRACE:'+JSON.stringify(row)
	if (line+'\n').to_utf8_buffer().size()+trace_sink_bytes>65536:
		return
	trace_records += 1
	_trace_sink_write(line)

func _trace_pad_sample(reason: String, neutral: bool, axes: Vector2i = Vector2i.ZERO, accepted: bool = false) -> void:
	if not trace_enabled or trace_pad_samples<=0:
		return
	trace_pad_samples -= 1
	_trace_pad('sample',mouse_move_owner.get('entityId',''),reason,{'neutral':neutral,'sx':axes.x,'sy':axes.y,'request_accepted':accepted})
func _trace_pad(stage: String, control_id: String, reason: String, facts: Dictionary = {}) -> void:
	# Passive fixed-schema formatting only: no business validation, sampling or mutation.
	if not trace_enabled or trace_pad_records>=80 or trace_records>=223:
		return
	var elapsed: int = Time.get_ticks_msec()-trace_started_ms
	if elapsed<0 or elapsed>=120000:
		return
	if stage not in ['raw','hud','main','sample']:
		stage = ''
	if reason not in ['down','up','clear','release','leave','hide','remove','projection_change','capture_invalid','admitted','rejected','captured','closing','unavailable','paused_or_entry','sample_gate','counter_limit','sample']:
		reason = ''
	var row: Dictionary = {'stage':stage,'control_id':control_id if control_id in MOVEMENT_LABELS else '','reason':reason,'elapsed_ms':elapsed,'sx':0,'sy':0,'request_accepted':false}
	for key in ['pressed','admitted','captured','has_owner','rearm','neutral','request_accepted']:
		if typeof(facts.get(key))==TYPE_BOOL:
			row[key] = facts[key]
	for key in ['sx','sy']:
		if typeof(facts.get(key))==TYPE_INT and facts[key] in [-1,0,1]:
			row[key] = facts[key]
	if reason in ['captured','clear']:
		trace_pad_samples = 4
	trace_records += 1
	trace_pad_records += 1
	print('WUXIAN_PAD_TRACE:'+JSON.stringify(row))

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
