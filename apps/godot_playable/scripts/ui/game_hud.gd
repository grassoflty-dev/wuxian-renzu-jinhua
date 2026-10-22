extends Control
const HudTheme = preload('res://scripts/ui/theme_model.gd')
const Content = preload('res://scripts/ui/content_model.gd')
const P = preload('res://scripts/bridge/protocol.gd')
const Icons = preload('res://scripts/ui/skill_icon_row.gd')
const PlayStatus = preload('res://scripts/ui/play_status_model.gd')
const WorldMechanism = preload('res://scripts/ui/world_mechanism_model.gd')
const CheckpointStatus = preload('res://scripts/ui/checkpoint_status_model.gd')
const PLAYER_ACTION_LABELS = {'primaryAttack':'攻击','dash':'冲刺','pulse':'脉冲','pierce':'穿刺'}
var player_action_states: Dictionary = {}
signal intent(kind, value)
const MOVEMENT_LABELS = {'move_up':'上移','move_down':'下移','move_left':'左移','move_right':'右移'}
const SHOW_MOVEMENT_CONTROLS = false
var movement_states: Dictionary = {}
var guard_button: Button
var guard_association: Dictionary = {}
var guard_capture: Dictionary = {}
var portrait: Label
var button_nodes: Array = []
var traverse_button: Button
var traverse_association: Dictionary = {}
var traverse_press: Dictionary = {}
var interact_button: Button
var interact_association: Dictionary = {}
var interact_press: Dictionary = {}
var status_label: Label
var hp_label: Label
var energy_label: Label
var vitals: VBoxContainer
var hp_meter: ProgressBar
var energy_meter: ProgressBar
var vitals_snapshot_valid = false
var objectives: Label
var hint: Label
var entry_wait_label: Label
var mechanism: Label
var checkpoint: Label
var icons
var bottom: ScrollContainer
var bottom_content: VBoxContainer
var bottom_layout_valid = false
var layout_queued = false
var layout_busy = false
var layout_scale = 1.0
func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	portrait = Label.new()
	portrait.text = '岑'
	portrait.position = Vector2(20,20)
	portrait.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(portrait)
	status_label = Label.new()
	status_label.position = Vector2(60,12)
	status_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(status_label)
	vitals = VBoxContainer.new()
	vitals.position = Vector2(60,44)
	vitals.mouse_filter = Control.MOUSE_FILTER_IGNORE
	vitals.add_theme_constant_override('separation',0)
	add_child(vitals)
	hp_label = Label.new()
	hp_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	hp_label.add_theme_color_override('font_color',Color(HudTheme.BASE.danger))
	vitals.add_child(hp_label)
	energy_label = Label.new()
	energy_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	energy_label.add_theme_color_override('font_color',Color(HudTheme.BASE.system))
	vitals.add_child(energy_label)
	hp_meter = _make_vital_meter(hp_label,'HpRatioMeter',Color(HudTheme.BASE.danger))
	energy_meter = _make_vital_meter(energy_label,'EnergyRatioMeter',Color(HudTheme.BASE.system))
	objectives = Label.new()
	objectives.set_anchors_and_offsets_preset(Control.PRESET_TOP_RIGHT)
	objectives.offset_left = -430
	objectives.offset_right = -20
	objectives.offset_top = 12
	objectives.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	objectives.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(objectives)
	bottom = ScrollContainer.new()
	bottom.name = 'BottomPanel'
	bottom.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_WIDE)
	bottom.offset_top = -155
	bottom.mouse_filter = Control.MOUSE_FILTER_STOP
	bottom.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_NEVER
	add_child(bottom)
	var style = StyleBoxFlat.new()
	style.bg_color = Color('#16202a')
	style.border_color = Color('#344652')
	style.set_border_width_all(1)
	style.content_margin_left = 12
	style.content_margin_right = 12
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	bottom.add_theme_stylebox_override('panel',style)
	bottom_content = VBoxContainer.new()
	bottom_content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bottom_content.mouse_filter = Control.MOUSE_FILTER_IGNORE
	bottom.add_child(bottom_content)
	icons = Icons.new()
	icons.name = 'SkillIconRow'
	bottom_content.add_child(icons)
	var side = VBoxContainer.new()
	side.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	side.mouse_filter = Control.MOUSE_FILTER_IGNORE
	bottom_content.add_child(side)
	entry_wait_label = Label.new()
	entry_wait_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	entry_wait_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	entry_wait_label.visible = false
	side.add_child(entry_wait_label)
	hint = Label.new()
	hint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	hint.mouse_filter = Control.MOUSE_FILTER_IGNORE
	side.add_child(hint)
	mechanism = Label.new()
	mechanism.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	mechanism.mouse_filter = Control.MOUSE_FILTER_IGNORE
	mechanism.visible = false
	side.add_child(mechanism)
	checkpoint = Label.new()
	checkpoint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	checkpoint.mouse_filter = Control.MOUSE_FILTER_IGNORE
	checkpoint.visible = false
	side.add_child(checkpoint)
	var buttons = HFlowContainer.new()
	side.add_child(buttons)
	for item in [['mission','任务'],['inventory','背包'],['character','角色'],['capability','能力'],['map','地图'],['pause','菜单']]:
		var button = Button.new()
		button.text = item[1]
		button.pressed.connect(func(): intent.emit(item[0],null))
		buttons.add_child(button)
		button_nodes.append(button)
	interact_button = Button.new()
	interact_button.visible = false
	interact_button.disabled = true
	interact_button.mouse_filter = Control.MOUSE_FILTER_STOP
	interact_button.button_down.connect(func():
		interact_press.clear()
		if not interact_button.disabled and _interaction_association_valid(interact_association):
			interact_press = interact_association.duplicate(true))
	interact_button.pressed.connect(func():
		if interact_button.disabled or not _interaction_association_valid(interact_press) or not _interaction_association_valid(interact_association) or not _interaction_identity_same(interact_press,interact_association):
			interact_press.clear()
			return
		var captured = interact_press.duplicate(true)
		interact_press.clear()
		interact_association.clear()
		interact_button.disabled = true
		intent.emit('interact_current',captured))
	buttons.add_child(interact_button)
	button_nodes.append(interact_button)
	traverse_button = Button.new()
	traverse_button.visible = false
	traverse_button.disabled = true
	traverse_button.mouse_filter = Control.MOUSE_FILTER_STOP
	traverse_button.button_down.connect(func():
		traverse_press.clear()
		if not traverse_button.disabled and _interaction_association_valid(traverse_association):
			traverse_press = traverse_association.duplicate(true))
	traverse_button.pressed.connect(func():
		if traverse_button.disabled or not _interaction_association_valid(traverse_press) or not _interaction_association_valid(traverse_association) or not _interaction_identity_same(traverse_press,traverse_association):
			traverse_press.clear()
			return
		var captured = traverse_press.duplicate(true)
		traverse_press.clear()
		traverse_association.clear()
		traverse_button.disabled = true
		intent.emit('traverse_current',captured))
	buttons.add_child(traverse_button)
	button_nodes.append(traverse_button)
	for action_id in PLAYER_ACTION_LABELS:
		_add_player_action(buttons,action_id)
	guard_button = Button.new()
	guard_button.text = '防御'
	guard_button.tooltip_text = '按住尝试防御；松开或取消时结束鼠标防御。'+'\n'+Icons.USE_TEXT['guard']
	guard_button.mouse_filter = Control.MOUSE_FILTER_STOP
	guard_button.focus_mode = Control.FOCUS_NONE
	guard_button.visible = false
	guard_button.disabled = true
	guard_button.button_down.connect(func():
		if not guard_capture.is_empty() or guard_button.disabled or not _guard_association_valid(guard_association):
			return
		guard_capture = guard_association.duplicate(true)
		intent.emit('guard_current_begin',guard_capture.duplicate(true)))
	guard_button.button_up.connect(_cancel_guard_capture)
	buttons.add_child(guard_button)
	button_nodes.append(guard_button)
	for direction_id in MOVEMENT_LABELS:
		_add_movement_button(buttons,direction_id)
	call_deferred('_load_internal_icons')
	resized.connect(_queue_bottom_layout)
	resized.connect(_layout_vital_meters)
	bottom.resized.connect(_queue_bottom_layout)
	bottom_content.minimum_size_changed.connect(_queue_bottom_layout)
	_queue_bottom_layout()
func set_entry_wait_text(value: String) -> void:
	if not is_instance_valid(entry_wait_label):
		return
	if entry_wait_label.text==value and entry_wait_label.visible==(not value.is_empty()):
		return
	entry_wait_label.text = value
	entry_wait_label.visible = not value.is_empty()
	_queue_bottom_layout()

func _make_vital_meter(label: Label, meter_name: String, tint: Color) -> ProgressBar:
	var meter = ProgressBar.new()
	meter.name = meter_name
	meter.mouse_filter = Control.MOUSE_FILTER_IGNORE
	meter.focus_mode = Control.FOCUS_NONE
	meter.show_percentage = false
	meter.min_value = 0.0
	meter.max_value = 100.0
	meter.step = 0.0
	meter.value = 0.0
	meter.visible = false
	meter.clip_contents = true
	var background = StyleBoxFlat.new()
	background.bg_color = Color(HudTheme.BASE.background)
	var fill = StyleBoxFlat.new()
	fill.bg_color = tint
	for style in [background,fill]:
		style.set_border_width_all(0)
		style.content_margin_left = 0.0
		style.content_margin_right = 0.0
		style.content_margin_top = 0.0
		style.content_margin_bottom = 0.0
	meter.add_theme_stylebox_override('background',background)
	meter.add_theme_stylebox_override('fill',fill)
	label.add_child(meter)
	label.resized.connect(_layout_vital_meters)
	return meter

func _layout_vital_meters() -> void:
	if not is_instance_valid(hp_meter) or not is_instance_valid(energy_meter) or not is_instance_valid(objectives):
		return
	var meter_scale = layout_scale if is_finite(layout_scale) and layout_scale>0.0 else 1.0
	var gap = 8.0*meter_scale
	var right_edge = minf(size.x-20.0,objectives.position.x-gap)
	for pair in [[hp_label,hp_meter],[energy_label,energy_meter]]:
		var label: Label = pair[0]
		var meter: ProgressBar = pair[1]
		var origin_x = vitals.position.x+label.position.x+label.size.x+gap
		var width = minf(128.0*meter_scale,right_edge-origin_x)
		meter.visible = vitals_snapshot_valid and is_finite(width) and width>=64.0*meter_scale
		if meter.visible:
			meter.custom_minimum_size = Vector2(64.0*meter_scale,8.0*meter_scale)
			meter.position = Vector2(label.size.x+gap,maxf(0.0,(label.size.y-8.0*meter_scale)*0.5))
			meter.size = Vector2(width,8.0*meter_scale)

func _queue_bottom_layout() -> void:
	if layout_queued or layout_busy:
		return
	layout_queued = true
	call_deferred('_layout_bottom')
func _layout_bottom() -> void:
	layout_queued = false
	if layout_busy or not is_instance_valid(bottom_content) or not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0 or bottom.size.x<=0:
		return
	layout_busy = true
	var style = bottom.get_theme_stylebox('panel')
	var padding = style.get_minimum_size()
	var reserve = bottom.get_v_scroll_bar().get_combined_minimum_size().x
	var width = bottom.size.x-padding.x-reserve
	if is_finite(width) and width>0 and bottom_content.custom_minimum_size.x!=width:
		bottom_content.custom_minimum_size.x = width
	var measured = bottom_content.get_combined_minimum_size().y+padding.y
	var height = minf(maxf(155*layout_scale,measured),size.y*0.45)
	if is_finite(height) and height>0 and bottom.offset_top!=-height:
		bottom.offset_top = -height
	bottom_layout_valid = is_finite(height) and height>0 and is_finite(bottom.offset_top) and bottom.offset_top<0
	layout_busy = false
func get_world_safe_rect() -> Rect2:
	var full = get_viewport_rect()
	if not is_visible_in_tree() or not bottom_layout_valid or not is_instance_valid(bottom):
		return full
	if not bottom.is_visible_in_tree():
		return full
	var occupied = bottom.get_global_rect()
	if not is_finite(occupied.position.x) or not is_finite(occupied.position.y) or not is_finite(occupied.size.x) or not is_finite(occupied.size.y):
		return full
	if occupied.size.x<=0 or occupied.size.y<=0 or not occupied.intersects(full):
		return full
	var safe_bottom = clampf(occupied.position.y,full.position.y,full.end.y)
	var safe = Rect2(full.position,Vector2(full.size.x,safe_bottom-full.position.y)).intersection(full)
	return safe if safe.size.x>0 and safe.size.y>0 else full
func _load_internal_icons() -> void:
	var file = FileAccess.open('res://data/skill_icons.internal-use.json',FileAccess.READ)
	if file == null:
		return
	if file.get_length()>16384:
		file.close()
		return
	var mapping = JSON.parse_string(file.get_as_text())
	file.close()
	if mapping is Dictionary:
		icons.apply_admitted_mapping(mapping)
func _interaction_association_valid(value) -> bool:
	return value is Dictionary and value.size()==5 and value.has_all(['generation','context','entityId','kind','label']) and P.counter(value.generation) and P.context(value.context) and P.ident(value.entityId) and typeof(value.kind)==TYPE_STRING and typeof(value.label)==TYPE_STRING
func _interaction_identity_same(a: Dictionary, b: Dictionary) -> bool:
	return not a.is_empty() and not b.is_empty() and a.generation==b.generation and P.same(a.context,b.context) and a.entityId==b.entityId and a.kind==b.kind
func _player_action_valid(value) -> bool:
	return _interaction_association_valid(value) and value.kind=='player_action' and PLAYER_ACTION_LABELS.has(value.entityId) and value.label==PLAYER_ACTION_LABELS[value.entityId]
func _add_player_action(buttons: HFlowContainer, action_id: String) -> void:
	var button = Button.new()
	var state = {'button':button,'display':{},'press':{}}
	player_action_states[action_id] = state
	button.text = PLAYER_ACTION_LABELS[action_id]
	button.tooltip_text = '尝试'+button.text+('；使用当前瞄准方向。' if action_id in ['primaryAttack','pierce'] else '。')+('\n'+Icons.USE_TEXT[action_id] if Icons.USE_TEXT.has(action_id) else '')
	state['tooltip'] = button.tooltip_text
	button.mouse_filter = Control.MOUSE_FILTER_STOP
	button.focus_mode = Control.FOCUS_NONE
	button.visible = false
	button.disabled = true
	button.button_down.connect(func():
		state.press.clear()
		if not button.disabled and _player_action_valid(state.display):
			state.press = state.display.duplicate(true))
	button.pressed.connect(func():
		if button.disabled or not _player_action_valid(state.press) or not _player_action_valid(state.display) or not _interaction_identity_same(state.press,state.display):
			state.press.clear()
			return
		var captured = state.press.duplicate(true)
		state.press.clear()
		state.display.clear()
		button.disabled = true
		intent.emit('player_action_current',captured))
	buttons.add_child(button)
	button_nodes.append(button)
func _update_player_actions(s: Dictionary, eligible: bool, actions) -> void:
	var selected: Dictionary = {}
	var common_generation = -1
	if eligible and P.snapshot(s) and actions is Array and actions.size()==4:
		for row in actions:
			if not _player_action_valid(row) or not P.same(row.context,s) or selected.has(row.entityId) or common_generation!=-1 and common_generation!=row.generation:
				selected.clear()
				break
			common_generation = row.generation
			selected[row.entityId] = row.duplicate(true)
	var busy = not selected.is_empty() and (PLAYER_ACTION_LABELS.has(s.player.actionState) or s.player.actionState=='guard')
	for action_id in PLAYER_ACTION_LABELS:
		var state = player_action_states[action_id]
		var desired: Dictionary = selected.get(action_id,{})
		if busy or desired.is_empty() or not _interaction_identity_same(desired,state.display):
			state.press.clear()
			state.display.clear()
		if desired.is_empty():
			state.button.disabled = true
			state.button.visible = false
		else:
			state.display = desired
			state.button.tooltip_text = state.tooltip+('\n当前动作尚未结束' if busy else '')
			if not state.button.visible:
				state.button.visible = true
			state.button.disabled = busy
func _guard_association_valid(value) -> bool:
	return _interaction_association_valid(value) and typeof(value.entityId)==TYPE_STRING and value.entityId=='guard' and value.kind=='player_guard' and value.label=='防御'
func _cancel_guard_capture() -> void:
	if guard_capture.is_empty():
		return
	var captured = guard_capture.duplicate(true)
	guard_capture.clear()
	intent.emit('guard_current_end',captured)
func _update_guard(s: Dictionary, eligible: bool, value) -> void:
	var desired: Dictionary = {}
	if eligible and P.snapshot(s) and _guard_association_valid(value) and P.same(value.context,s):
		desired = value.duplicate(true)
	if desired.is_empty() or not _interaction_identity_same(desired,guard_association):
		_cancel_guard_capture()
		guard_association.clear()
	if desired.is_empty():
		guard_button.disabled = true
		guard_button.visible = false
	else:
		guard_association = desired
		if not guard_button.visible:
			guard_button.visible = true
		if guard_button.disabled:
			guard_button.disabled = false
func _movement_association_valid(value) -> bool:
	return _interaction_association_valid(value) and typeof(value.entityId)==TYPE_STRING and MOVEMENT_LABELS.has(value.entityId) and value.kind=='player_move' and value.label==MOVEMENT_LABELS[value.entityId]
func _cancel_movement_capture(state: Dictionary) -> void:
	if state.capture.is_empty():
		return
	var captured = state.capture.duplicate(true)
	state.capture.clear()
	intent.emit('movement_current_end',captured)
func _clear_movement_captures() -> void:
	for state in movement_states.values():
		_cancel_movement_capture(state)
func _add_movement_button(buttons: HFlowContainer, direction_id: String) -> void:
	var button = Button.new()
	var state = {'button':button,'display':{},'capture':{}}
	movement_states[direction_id] = state
	button.text = MOVEMENT_LABELS[direction_id]
	button.tooltip_text = '按住沿屏幕方向移动；松开或取消时停止鼠标移动。'
	button.mouse_filter = Control.MOUSE_FILTER_STOP if SHOW_MOVEMENT_CONTROLS else Control.MOUSE_FILTER_IGNORE
	button.focus_mode = Control.FOCUS_NONE
	button.disabled = true
	button.visible = false
	button.button_down.connect(func():
		if button.disabled or not state.capture.is_empty() or not _movement_association_valid(state.display):
			return
		state.capture = state.display.duplicate(true)
		intent.emit('movement_current_begin',state.capture.duplicate(true)))
	button.button_up.connect(func(): _cancel_movement_capture(state))
	buttons.add_child(button)
	button_nodes.append(button)
func _update_movements(s: Dictionary, eligible: bool, movements) -> void:
	var selected: Dictionary = {}
	var common_generation = -1
	if SHOW_MOVEMENT_CONTROLS and eligible and P.snapshot(s) and movements is Array and movements.size()==4:
		for row in movements:
			if not _movement_association_valid(row) or not P.same(row.context,s) or selected.has(row.entityId) or common_generation!=-1 and common_generation!=row.generation:
				selected.clear()
				break
			common_generation = row.generation
			selected[row.entityId] = row.duplicate(true)
	for direction_id in MOVEMENT_LABELS:
		var state = movement_states[direction_id]
		var desired: Dictionary = selected.get(direction_id,{})
		if desired.is_empty() or not _interaction_identity_same(desired,state.display):
			_cancel_movement_capture(state)
			state.display.clear()
		if desired.is_empty():
			state.button.disabled = true
			state.button.visible = false
		else:
			state.display = desired
			if not state.button.visible:
				state.button.visible = true
			if state.button.disabled:
				state.button.disabled = false
func update_view(s: Dictionary, connection: String, eligible: bool, interaction: String, scale: float, target = {}, traversal = {}, actions = [], guard = {}, movements = [], pulse_key: String = 'Q') -> void:
	var valid_snapshot = P.snapshot(s)
	if not valid_snapshot:
		s = {}
	var desired: Dictionary = {}
	if eligible and P.snapshot(s) and _interaction_association_valid(target) and P.same(target.context,s):
		desired = target.duplicate(true)
	if desired.is_empty() or not _interaction_identity_same(desired,interact_association):
		interact_press.clear()
		interact_association.clear()
	if desired.is_empty():
		interact_button.disabled = true
		interact_button.visible = false
	else:
		interact_association = desired
		interact_button.text = '交互 · '+desired.label
		if not interact_button.visible:
			interact_button.visible = true
		if interact_button.disabled:
			interact_button.disabled = false
	var traverse_desired: Dictionary = {}
	if eligible and P.snapshot(s) and _interaction_association_valid(traversal) and traversal.kind=='contextTraversal' and P.same(traversal.context,s):
		traverse_desired = traversal.duplicate(true)
	if traverse_desired.is_empty() or not _interaction_identity_same(traverse_desired,traverse_association):
		traverse_press.clear()
		traverse_association.clear()
	if traverse_desired.is_empty():
		traverse_button.disabled = true
		traverse_button.visible = false
	else:
		traverse_association = traverse_desired
		traverse_button.text = '穿越 · '+traverse_desired.label
		if not traverse_button.visible:
			traverse_button.visible = true
		if traverse_button.disabled:
			traverse_button.disabled = false
	_update_player_actions(s,eligible,actions)
	_update_guard(s,eligible,guard)
	_update_movements(s,eligible,movements)
	status_label.text = connection if s.is_empty() else {'return_station':'归航站','grey_hive':'灰巢设施','mist_harbor':'雾港余烬','clockworks':'钟骨工厂'}.get(s.worldId,s.worldId)+' · '+Content.SCENE_NAMES.get(s.sceneId,'当前场景')+' · '+connection
	if s.is_empty():
		hp_label.text = ''
		energy_label.text = ''
	else:
		hp_label.text = '岑遥 · 生命 %d / %d'%[s.player.currentHp,s.player.maxHp]
		energy_label.text = '能量 %d / %d'%[s.player.currentEnergy,s.player.maxEnergy]
	vitals_snapshot_valid = valid_snapshot
	hp_meter.value = 100.0*(float(s.player.currentHp)/float(s.player.maxHp)) if valid_snapshot else 0.0
	energy_meter.value = 100.0*(float(s.player.currentEnergy)/float(s.player.maxEnergy)) if valid_snapshot else 0.0
	if not valid_snapshot:
		hp_meter.hide()
		energy_meter.hide()
	var display = PlayStatus.project(s,eligible)
	var lines: Array = display.objectives.duplicate()
	if not display.milestones.is_empty():
		if not lines.is_empty():
			lines.append('')
		lines.append(' · '.join(PackedStringArray(display.milestones)))
	objectives.text = '\n'.join(PackedStringArray(lines))
	hint.text = interaction+'\nWASD / 方向键移动 · J / 鼠标左键攻击 · 空格：场景穿越 · F：交互'
	if not display.action.is_empty() and display.action.kind=='primaryAttack':
		hint.text += '\n'+display.action.text
	mechanism.text = '\n'.join(PackedStringArray(WorldMechanism.project(s,eligible)))
	mechanism.visible = not mechanism.text.is_empty()
	checkpoint.text = CheckpointStatus.project(s,eligible)
	checkpoint.visible = not checkpoint.text.is_empty()
	for label in [portrait,status_label,hp_label,energy_label,objectives,hint,entry_wait_label,mechanism,checkpoint]:
		label.add_theme_font_size_override('font_size',int(16*scale))
	for button in button_nodes:
		button.add_theme_font_size_override('font_size',int(16*scale))
		button.custom_minimum_size.y = 32*scale
	icons.update_display(scale,'' if s.is_empty() else s.player.actionState,eligible,display.action,pulse_key)
	if layout_scale!=scale:
		layout_scale = scale
		_queue_bottom_layout()
	call_deferred('_layout_vital_meters')
