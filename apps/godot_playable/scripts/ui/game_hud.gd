extends Control
const GhDenseServiceWingVisual = preload('res://scripts/world/gh_dense_service_wing_visual.gd')
var gh_dense_region_visual = GhDenseServiceWingVisual.new()
const UiArt = preload('res://scripts/ui/ui_art_texture.gd')
const HudTheme = preload('res://scripts/ui/theme_model.gd')
const Content = preload('res://scripts/ui/content_model.gd')
const P = preload('res://scripts/bridge/protocol.gd')
const Icons = preload('res://scripts/ui/skill_icon_row.gd')
const PlayStatus = preload('res://scripts/ui/play_status_model.gd')
const WorldMechanism = preload('res://scripts/ui/world_mechanism_model.gd')
const CheckpointStatus = preload('res://scripts/ui/checkpoint_status_model.gd')
const BOTTOM_HEIGHT_FRACTION = 0.22
const PLAYER_ACTION_LABELS = {'primaryAttack':'攻击','dash':'冲刺','pulse':'脉冲','pierce':'穿刺'}
var player_action_states: Dictionary = {}
var skill_button_backgrounds_attempted = false
var skill_family_textures: Dictionary = {}
var skill_family_original: Dictionary = {}
var skill_family_active = false
signal intent(kind, value)
signal pad_trace(stage, control_id, reason, facts)
var pad_trace_enabled = false
const MOVEMENT_LABELS = {'move_up':'上移','move_down':'下移','move_left':'左移','move_right':'右移','move_up_left':'左上移','move_up_right':'右上移','move_down_left':'左下移','move_down_right':'右下移'}
const MOVEMENT_ARROWS = {'move_up_left':'↖','move_up':'↑','move_up_right':'↗','move_left':'←','move_right':'→','move_down_left':'↙','move_down':'↓','move_down_right':'↘'}
var movement_pad_enabled = false
var movement_pad: GridContainer
var movement_help: Label
var movement_toggle: CheckBox
var movement_toggle_owner: Dictionary = {}
var movement_toggle_press: Dictionary = {}
var movement_states: Dictionary = {}
var guard_button: Button
var guard_association: Dictionary = {}
var guard_capture: Dictionary = {}
var hud_identity_backing: TextureRect
var hud_objective_backing: TextureRect
var backing_layout_queued = false
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
var general_help: Label
var mechanism: Label
var checkpoint: Label
var combat_feedback: Label
var current_action_feedback: Label
var icons
var bottom: PanelContainer
var bottom_scroll: ScrollContainer
var movement_scroll: ScrollContainer
var feedback_strip: VBoxContainer
var bottom_columns: HBoxContainer
var movement_column: VBoxContainer
var bottom_content: VBoxContainer
var bottom_layout_valid = false
var layout_queued = false
var layout_busy = false
var layout_scale = 1.0
var _applied_scale: float = -1.0
var top_status_clip: Control
var top_goal_clip: Control
var details_button: Button
var details_panel: PanelContainer
var details_scroll: ScrollContainer
var details_text: Label
var details_close: Button
var details_up: Button
var details_down: Button
var top_left_rect := Rect2()
var top_right_rect := Rect2()
var top_layout_queued := false
func _install_portrait_image() -> void:
	if not is_instance_valid(portrait): return
	var source = 'res://assets/characters/cenyao/ui/cenyao_portrait_local_r1.png'
	var texture: Texture2D
	if ResourceLoader.exists(source,'Texture2D'):
		var imported = ResourceLoader.load(source,'Texture2D')
		if imported is Texture2D:
			var image: Image = imported.get_image()
			if is_instance_valid(image) and image.get_width()==1254 and image.get_height()==1254 and image.get_format()==Image.FORMAT_RGBA8: texture = imported
	if not is_instance_valid(texture):
		var image = Image.new()
		var bytes = FileAccess.get_file_as_bytes(source)
		if image.load_png_from_buffer(bytes)==OK and image.get_width()==1254 and image.get_height()==1254 and image.get_format()==Image.FORMAT_RGBA8: texture = ImageTexture.create_from_image(image)
	if not is_instance_valid(texture): return
	var image_rect = TextureRect.new()
	image_rect.texture = texture
	image_rect.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	image_rect.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	image_rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
	portrait.add_child(image_rect)
	image_rect.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	portrait.self_modulate = Color(1,1,1,0)

func _ready() -> void:
	HudTheme.install_tooltip_skin(self)
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	portrait = Label.new()
	portrait.text = '岑'
	portrait.position = Vector2(20,20)
	portrait.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(portrait)
	_install_portrait_image()
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
	_install_top_details()
	var backing_material = UiArt.atlas('res://assets/ui/global_ui_panel_b.png',Rect2(400,300,128,128))
	if backing_material != null:
		hud_identity_backing = TextureRect.new()
		hud_objective_backing = TextureRect.new()
		for backing in [hud_identity_backing,hud_objective_backing]:
			backing.texture = backing_material
			backing.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
			backing.stretch_mode = TextureRect.STRETCH_SCALE
			backing.mouse_filter = Control.MOUSE_FILTER_IGNORE
			backing.visible = false
			add_child(backing)
			move_child(backing,0)
		resized.connect(_queue_hud_backings)
		for control in [portrait,status_label,vitals,objectives]:
			control.resized.connect(_queue_hud_backings)
		call_deferred('_queue_hud_backings')
	bottom = PanelContainer.new()
	bottom.name = 'BottomPanel'
	bottom.set_anchors_and_offsets_preset(Control.PRESET_CENTER_BOTTOM)
	bottom.offset_left = -230
	bottom.offset_right = 230
	bottom.offset_top = -72
	bottom.mouse_filter = Control.MOUSE_FILTER_STOP
	add_child(bottom)
	bottom.resized.connect(_queue_top_layout)
	bottom.visibility_changed.connect(_queue_top_layout)
	var style = StyleBoxFlat.new()
	style.bg_color = Color('#16202a')
	style.border_color = Color('#344652')
	style.set_border_width_all(1)
	style.content_margin_left = 12
	style.content_margin_right = 12
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	bottom.add_theme_stylebox_override('panel',style)
	HudTheme.install_hud_information_panel(bottom)
	var bottom_stack = VBoxContainer.new()
	bottom_stack.name = 'BottomStack'
	bottom.add_child(bottom_stack)
	feedback_strip = VBoxContainer.new()
	feedback_strip.name = 'EssentialFeedback'
	feedback_strip.mouse_filter = Control.MOUSE_FILTER_IGNORE
	bottom_stack.add_child(feedback_strip)
	hint = Label.new()
	hint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	hint.mouse_filter = Control.MOUSE_FILTER_IGNORE
	feedback_strip.add_child(hint)
	current_action_feedback = Label.new()
	current_action_feedback.mouse_filter = Control.MOUSE_FILTER_IGNORE
	current_action_feedback.max_lines_visible = 1
	current_action_feedback.clip_text = true
	current_action_feedback.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	current_action_feedback.visible = false
	feedback_strip.add_child(current_action_feedback)
	combat_feedback = Label.new()
	combat_feedback.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	combat_feedback.mouse_filter = Control.MOUSE_FILTER_IGNORE
	combat_feedback.visible = false
	feedback_strip.add_child(combat_feedback)
	bottom_columns = HBoxContainer.new()
	bottom_columns.name = 'BottomColumns'
	bottom_columns.size_flags_vertical = Control.SIZE_EXPAND_FILL
	bottom_columns.add_theme_constant_override('separation',12)
	bottom_stack.add_child(bottom_columns)
	bottom_scroll = ScrollContainer.new()
	bottom_scroll.name = 'HudContentScroll'
	bottom_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bottom_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_NEVER
	bottom_columns.add_child(bottom_scroll)
	movement_scroll = ScrollContainer.new()
	movement_scroll.name = 'MouseHoldPadScroll'
	movement_scroll.custom_minimum_size.x = 260
	movement_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	movement_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_NEVER
	bottom_columns.add_child(movement_scroll)
	movement_column = VBoxContainer.new()
	movement_column.name = 'MouseHoldPadColumn'
	movement_column.custom_minimum_size.x = 260
	movement_column.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
	movement_scroll.add_child(movement_column)
	movement_scroll.get_v_scroll_bar().value_changed.connect(func(_value): _clear_movement_captures())
	bottom_content = VBoxContainer.new()
	bottom_content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bottom_content.mouse_filter = Control.MOUSE_FILTER_IGNORE
	bottom_scroll.add_child(bottom_content)
	icons = Icons.new()
	icons.name = 'SkillIconRow'
	icons.visible = false
	bottom_content.add_child(icons)
	var side = VBoxContainer.new()
	side.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	side.mouse_filter = Control.MOUSE_FILTER_IGNORE
	bottom_content.add_child(side)
	general_help = Label.new()
	general_help.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	general_help.mouse_filter = Control.MOUSE_FILTER_IGNORE
	side.add_child(general_help)
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
	buttons.alignment = FlowContainer.ALIGNMENT_CENTER
	buttons.resized.connect(_queue_bottom_layout)
	side.add_child(buttons)
	for item in [['mission','任务'],['inventory','背包'],['character','角色'],['capability','能力'],['map','地图'],['pause','菜单']]:
		var button = Button.new()
		button.text = item[1]
		button.visible = item[0]=='pause'
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
	guard_button.text = 'E'
	guard_button.expand_icon = true
	guard_button.add_theme_constant_override('icon_max_width',48)
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
	movement_toggle = CheckBox.new()
	movement_toggle.name = 'MouseHoldPadToggle'
	movement_toggle.text = '鼠标移动'
	movement_toggle.focus_mode = Control.FOCUS_NONE
	movement_toggle.mouse_filter = Control.MOUSE_FILTER_STOP
	movement_toggle.button_down.connect(func():
		movement_toggle_press = movement_toggle_owner.duplicate(true) if not movement_toggle.disabled else {})
	movement_toggle.toggled.connect(func(enabled: bool):
		var captured: Dictionary = movement_toggle_press.duplicate(true)
		movement_toggle_press.clear()
		if not captured.is_empty():
			intent.emit('movement_pad_toggle',{'enabled':enabled,'owner':captured}))
	buttons.add_child(movement_toggle)
	movement_toggle.visible = false
	movement_help = Label.new()
	movement_help.name = 'MouseHoldPadHelp'
	movement_help.text = '左键按住移动\n松开或移出停止\nWASD / 方向键优先'
	movement_help.tooltip_text = '按住鼠标左键移动；松开或移出按钮停止。WASD / 方向键优先。'
	movement_help.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	movement_help.mouse_filter = Control.MOUSE_FILTER_STOP
	movement_column.add_child(movement_help)
	movement_pad = GridContainer.new()
	movement_pad.name = 'MouseHoldPad'
	movement_pad.columns = 3
	movement_pad.focus_mode = Control.FOCUS_NONE
	movement_pad.mouse_filter = Control.MOUSE_FILTER_STOP
	movement_column.add_child(movement_pad)
	for direction_id in ['move_up_left','move_up','move_up_right','move_left','','move_right','move_down_left','move_down','move_down_right']:
		if direction_id.is_empty():
			var stop_label = Label.new()
			stop_label.name = 'MouseHoldPadStop'
			stop_label.text = '松开停止'
			stop_label.mouse_filter = Control.MOUSE_FILTER_STOP
			movement_pad.add_child(stop_label)
		else:
			_add_movement_button(movement_pad,direction_id)
	# The hotbar uses the existing owned action buttons in Q/E/R/Shift order.
	buttons.move_child(player_action_states.pulse.button,0)
	buttons.move_child(guard_button,1)
	buttons.move_child(player_action_states.pierce.button,2)
	buttons.move_child(player_action_states.dash.button,3)
	movement_pad.visible = false
	movement_help.visible = false
	movement_scroll.visible = false
	call_deferred('_load_internal_icons')
	call_deferred('_load_skill_family')
	resized.connect(_queue_bottom_layout)
	resized.connect(_layout_vital_meters)
	bottom.resized.connect(_queue_bottom_layout)
	bottom_scroll.resized.connect(_queue_bottom_layout)
	movement_scroll.resized.connect(_queue_bottom_layout)
	feedback_strip.minimum_size_changed.connect(_queue_bottom_layout)
	bottom_content.minimum_size_changed.connect(_queue_bottom_layout)
	movement_column.minimum_size_changed.connect(_queue_bottom_layout)
	_queue_bottom_layout()
	_applied_scale = -1.0

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
	var frame_texture = UiArt.atlas('res://assets/ui/global_ui_frame.png',Rect2(73,430,1390,153))
	var fill_path = 'res://assets/ui/global_ui_hp.png' if meter_name == 'HpRatioMeter' else 'res://assets/ui/global_ui_energy.png'
	var fill_texture = UiArt.atlas(fill_path,Rect2(100,480,1336,60))
	if frame_texture != null and fill_texture != null:
		var frame_style = StyleBoxTexture.new()
		frame_style.texture = frame_texture
		meter.add_theme_stylebox_override('background',frame_style)
		# Native ProgressBar fill spans the full frame, so hide it for skinned meters.
		meter.add_theme_stylebox_override('fill',StyleBoxEmpty.new())
		var clip = Control.new()
		clip.name = 'ArtFillClip'
		clip.mouse_filter = Control.MOUSE_FILTER_IGNORE
		clip.clip_contents = true
		meter.add_child(clip)
		var art_fill = TextureRect.new()
		art_fill.name = 'ArtFill'
		art_fill.mouse_filter = Control.MOUSE_FILTER_IGNORE
		art_fill.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		art_fill.stretch_mode = TextureRect.STRETCH_SCALE
		art_fill.texture = fill_texture
		clip.add_child(art_fill)
		meter.value_changed.connect(func(_value): _sync_vital_fill(meter))
	else:
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
	var right_edge = top_left_rect.end.x-gap if top_left_rect.size.x>0 else size.x-20.0
	for pair in [[hp_label,hp_meter],[energy_label,energy_meter]]:
		var label: Label = pair[0]
		var meter: ProgressBar = pair[1]
		var origin_x = vitals.position.x+label.position.x+label.size.x+gap
		var width = minf(128.0*meter_scale,right_edge-origin_x)
		var frame = meter.get_theme_stylebox('background')
		var skinned = frame is StyleBoxTexture
		if skinned:
			width = minf(width,label.size.y*(1390.0/153.0))
		meter.visible = vitals_snapshot_valid and is_finite(width) and width>=64.0*meter_scale
		if meter.visible:
			var height = width*(153.0/1390.0) if skinned else 8.0*meter_scale
			meter.custom_minimum_size = Vector2(64.0*meter_scale,0)
			meter.position = Vector2(label.size.x+gap,maxf(0.0,(label.size.y-height)*0.5))
			meter.size = Vector2(width,height)
			if skinned:
				_sync_vital_fill(meter)

func _sync_vital_fill(meter: ProgressBar) -> void:
	var clip = meter.get_node_or_null('ArtFillClip') as Control
	if clip == null:
		return
	var art_fill = clip.get_node('ArtFill') as TextureRect
	# Source frame ROI is (73,430)-(1463,583); this window (160,458)-(1376,558)
	# stays inside its beveled ends, central steps and top/bottom frame.
	var full_size = Vector2(meter.size.x*(1216.0/1390.0),meter.size.y*(100.0/153.0))
	clip.position = Vector2(meter.size.x*(87.0/1390.0),meter.size.y*(28.0/153.0))
	art_fill.size = full_size
	var ratio = clampf(meter.value/100.0,0.0,1.0)
	clip.size = Vector2(full_size.x*ratio,full_size.y)
	clip.visible = ratio>0.0

func _install_top_details() -> void:
	top_status_clip = Control.new()
	top_goal_clip = Control.new()
	for clip in [top_status_clip,top_goal_clip]:
		clip.clip_contents = true
		clip.mouse_filter = Control.MOUSE_FILTER_IGNORE
		add_child(clip)
	status_label.reparent(top_status_clip)
	objectives.reparent(top_goal_clip)
	for label in [status_label,objectives]:
		label.set_anchors_and_offsets_preset(Control.PRESET_TOP_LEFT)
		label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		label.max_lines_visible = 2
		label.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	details_button = Button.new()
	details_button.text = '目标与操作'
	details_button.focus_mode = Control.FOCUS_ALL
	add_child(details_button)
	details_panel = PanelContainer.new()
	details_panel.visible = false
	details_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	add_child(details_panel)
	HudTheme.install_hud_information_panel(details_panel)
	var column = VBoxContainer.new()
	details_panel.add_child(column)
	var controls = HBoxContainer.new()
	column.add_child(controls)
	details_close = Button.new()
	details_close.text = '收起'
	details_up = Button.new()
	details_up.text = '上一页'
	details_down = Button.new()
	details_down.text = '下一页'
	for button in [details_close,details_up,details_down]:
		button.focus_mode = Control.FOCUS_ALL
		controls.add_child(button)
	details_scroll = ScrollContainer.new()
	details_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	details_scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	details_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	details_scroll.focus_mode = Control.FOCUS_ALL
	column.add_child(details_scroll)
	details_text = Label.new()
	details_text.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
	details_text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	details_text.mouse_filter = Control.MOUSE_FILTER_IGNORE
	details_scroll.add_child(details_text)
	details_button.pressed.connect(func(): details_panel.visible = not details_panel.visible; _queue_top_layout())
	details_close.pressed.connect(func(): details_panel.hide(); details_button.grab_focus(); _queue_top_layout())
	details_up.pressed.connect(func(): details_scroll.scroll_vertical = maxi(0,details_scroll.scroll_vertical-int(details_scroll.size.y)))
	details_down.pressed.connect(func(): details_scroll.scroll_vertical += int(details_scroll.size.y))
	resized.connect(_queue_top_layout)
	call_deferred('_queue_top_layout')

func _queue_top_layout() -> void:
	if top_layout_queued: return
	top_layout_queued = true
	call_deferred('_layout_top')

func _layout_top() -> void:
	top_layout_queued = false
	if not is_instance_valid(details_button) or not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0: return
	var u = layout_scale if is_finite(layout_scale) and layout_scale>0 else 1.0
	var safe = get_world_safe_rect()
	var frame = get_global_transform_with_canvas()
	if not is_finite(frame.determinant()) or absf(frame.determinant())<0.000001 or not is_zero_approx(frame.x.y) or not is_zero_approx(frame.y.x) or frame.y.y<=0.0: return
	var safe_bottom = clampf((frame.affine_inverse()*Vector2(safe.position.x,safe.end.y)).y,0.0,size.y)
	var margin = 12.0*u
	var gap = 12.0*u
	var width = maxf(0.0,size.x-2.0*margin)
	var available_height = maxf(0.0,safe_bottom-margin)
	if width<=0.0 or available_height<=0.0: return
	var font = hp_label.get_theme_font('font')
	if font==null: return
	var fs = hp_label.get_theme_font_size('font_size')
	var vital_width = maxf(font.get_string_size(hp_label.text,HORIZONTAL_ALIGNMENT_LEFT,-1,fs).x,font.get_string_size(energy_label.text,HORIZONTAL_ALIGNMENT_LEFT,-1,fs).x)
	var left_min = 48.0*u+vital_width+80.0*u
	var right_min = details_button.get_combined_minimum_size().x+16.0*u
	var side = width>=left_min+right_min+gap
	var left_width = minf(width,maxf(left_min,width-minf(410.0*u,width*0.46)-gap)) if side else width
	var right_width = width-left_width-gap if side else width
	if side and not details_panel.visible:
		var goal_font=objectives.get_theme_font('font')
		var goal_size=objectives.get_theme_font_size('font_size')
		var lines=objectives.text.split('\n',true)
		if goal_font!=null and goal_size>0 and lines.size()<=2:
			var measured_goal=0.0
			for text_line in lines:
				measured_goal=maxf(measured_goal,goal_font.get_string_size(text_line,HORIZONTAL_ALIGNMENT_LEFT,-1,goal_size).x)
			var needed=maxf(right_min,measured_goal+16.0*u)
			if is_finite(measured_goal) and is_finite(needed) and measured_goal<=right_width-8.0*u and needed<=right_width:
				right_width=needed
	var line = maxf(1.0,font.get_height(fs))
	var left_height = minf(available_height,maxf(64.0*u,4.0*line+16.0*u))
	var right_height = minf(available_height,maxf(4.0*line+details_button.get_combined_minimum_size().y+8.0*u,64.0*u))
	if not side:
		left_height = minf(left_height,maxf(0.0,available_height-gap-details_button.get_combined_minimum_size().y))
	top_left_rect = Rect2(margin,margin,left_width,left_height)
	var right_y = margin if side else margin+left_height+gap
	top_right_rect = Rect2(margin+width-right_width if side else margin,right_y,right_width,minf(right_height,maxf(0.0,safe_bottom-right_y)))
	portrait.position = top_left_rect.position+Vector2(8.0*u,8.0*u)
	top_status_clip.position = top_left_rect.position+Vector2(40.0*u,4.0*u)
	top_status_clip.size = Vector2(maxf(0.0,left_width-48.0*u),2.0*line)
	status_label.position = Vector2.ZERO
	status_label.size = top_status_clip.size
	vitals.position = top_left_rect.position+Vector2(40.0*u,4.0*u+2.0*line)
	var button_h = minf(top_right_rect.size.y,details_button.get_combined_minimum_size().y)
	details_button.position = top_right_rect.position+Vector2(4.0*u,maxf(0.0,top_right_rect.size.y-button_h))
	details_button.size = Vector2(maxf(0.0,minf(right_width-8.0*u,right_min)),button_h)
	top_goal_clip.position = top_right_rect.position+Vector2(4.0*u,4.0*u)
	top_goal_clip.size = Vector2(maxf(0.0,right_width-8.0*u),maxf(0.0,top_right_rect.size.y-button_h-8.0*u))
	objectives.position = Vector2.ZERO
	objectives.size = top_goal_clip.size
	if details_panel.visible:
		var y = maxf(top_left_rect.end.y,top_right_rect.end.y)+gap
		var minimum = details_close.get_combined_minimum_size().y+2.0*line+16.0*u
		if safe_bottom-y<minimum: y = margin
		var h = maxf(0.0,minf(maxf(size.y*0.4,minimum),safe_bottom-y))
		details_panel.position = Vector2(margin,y)
		details_panel.size = Vector2(minf(width,maxf(right_width,410.0*u)),h)
		details_text.custom_minimum_size.x = maxf(0.0,details_panel.size.x-48.0*u)
	_layout_vital_meters()
	_queue_hud_backings()

func _queue_hud_backings() -> void:
	if backing_layout_queued: return
	backing_layout_queued = true
	call_deferred('_layout_hud_backings')

func _layout_hud_backings() -> void:
	backing_layout_queued = false
	if not is_instance_valid(hud_identity_backing) or not is_instance_valid(hud_objective_backing) or not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0: return
	var pad = 8.0*(layout_scale if is_finite(layout_scale) and layout_scale>0 else 1.0)
	var screen = Rect2(Vector2.ZERO,size)
	var bounds = top_left_rect.intersection(screen)
	hud_identity_backing.visible = bounds.size.x>0 and bounds.size.y>0
	if hud_identity_backing.visible:
		hud_identity_backing.position = bounds.position
		hud_identity_backing.size = bounds.size
	var goal_bounds = top_right_rect.intersection(screen)
	hud_objective_backing.visible = goal_bounds.size.x>0 and goal_bounds.size.y>0
	if hud_objective_backing.visible:
		hud_objective_backing.position = goal_bounds.position
		hud_objective_backing.size = goal_bounds.size
func _queue_bottom_layout() -> void:
	if layout_queued or layout_busy:
		return
	layout_queued = true
	call_deferred('_layout_bottom')
func _layout_bottom() -> void:
	layout_queued = false
	if layout_busy or not is_instance_valid(bottom_content) or not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0 or bottom.size.x<=0:
		return
	var previous_valid = bottom_layout_valid
	var previous_top = bottom.offset_top
	layout_busy = true
	var target_width: float = minf(460.0*layout_scale,maxf(1.0,size.x-24.0*layout_scale))
	bottom.offset_left = -target_width*0.5
	bottom.offset_right = target_width*0.5
	var style = bottom.get_theme_stylebox('panel')
	var padding = style.get_minimum_size()
	var reserve: float = bottom_scroll.get_v_scroll_bar().size.x if bottom_scroll.get_v_scroll_bar().visible else 0.0
	var width: float = bottom_scroll.size.x-reserve
	_refresh_skill_family(width)
	if is_finite(width) and width>0 and bottom_content.custom_minimum_size.x!=width:
		bottom_content.custom_minimum_size.x = width
	var measured = feedback_strip.get_combined_minimum_size().y+maxf(bottom_content.get_combined_minimum_size().y,movement_column.get_combined_minimum_size().y)+padding.y+4.0*layout_scale
	var height = minf(maxf(72*layout_scale,measured),size.y*BOTTOM_HEIGHT_FRACTION)
	if is_finite(height) and height>0 and bottom.offset_top!=-height:
		bottom.offset_top = -height
	bottom_layout_valid = is_finite(height) and height>0 and is_finite(bottom.offset_top) and bottom.offset_top<0
	layout_busy = false
	if previous_valid!=bottom_layout_valid or previous_top!=bottom.offset_top:
		_queue_top_layout()
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

func get_rs_camera_fit_rect() -> Rect2:
	var full=get_viewport_rect()
	if not is_finite(full.position.x) or not is_finite(full.position.y) or not is_finite(full.size.x) or not is_finite(full.size.y) or full.size.x<=0.0 or full.size.y<=0.0: return full
	return Rect2(full.position,Vector2(full.size.x,full.size.y*(1.0-BOTTOM_HEIGHT_FRACTION)))
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
		for id in ['pulse','pierce','dash']:
			player_action_states[id].button.icon = icons.slots[id].icon.texture
		guard_button.icon = icons.slots.guard.icon.texture
func _skill_family_buttons() -> Dictionary:
	return {'pulse':player_action_states.pulse.button,'guard':guard_button,'pierce':player_action_states.pierce.button,'dash':player_action_states.dash.button}

func _load_skill_family() -> void:
	var loaded = {}
	for id in ['pulse','guard','pierce','dash']:
		var atlas = UiArt.atlas('res://assets/ui/skill_family/'+id+'.png',Rect2(0,0,1254,1254))
		if atlas == null or atlas.atlas.get_size()!=Vector2(1254,1254):
			return
		loaded[id] = atlas
	var buttons = _skill_family_buttons()
	for id in buttons:
		skill_family_original[id] = buttons[id].icon
	skill_family_textures = loaded
	_queue_bottom_layout()

func _refresh_skill_family(available: float) -> void:
	if skill_family_textures.size()!=4 or skill_family_original.size()!=4:
		return
	var buttons = _skill_family_buttons()
	var flow = guard_button.get_parent() as HFlowContainer
	var fits = is_instance_valid(flow) and is_finite(available) and available>0 and is_finite(layout_scale) and layout_scale>0
	if fits:
		var width = minf(available,flow.size.x)
		var needed = 0.0
		var count = 0
		for id in ['pulse','guard','pierce','dash']:
			var button: Button = buttons[id]
			var font = button.get_theme_font('font')
			var box = button.get_theme_stylebox('normal')
			if font == null or box == null:
				fits = false
				break
			var text_width = font.get_string_size(button.text,HORIZONTAL_ALIGNMENT_LEFT,-1,button.get_theme_font_size('font_size')).x
			var desired = 48.0*layout_scale + text_width + maxf(0.0,button.get_theme_constant('h_separation')) + box.get_minimum_size().x
			needed += maxf(desired,button.get_combined_minimum_size().x)
			count += 1
		if fits:
			for sibling in flow.get_children():
				if sibling is Control and sibling.visible and sibling not in buttons.values():
					needed += sibling.get_combined_minimum_size().x
					count += 1
			needed += maxf(0.0,flow.get_theme_constant('h_separation'))*maxi(0,count-1)
			fits = is_finite(width) and width>=needed
	if fits==skill_family_active:
		return
	skill_family_active = fits
	for id in ['pulse','guard','pierce','dash']:
		buttons[id].icon = skill_family_textures[id] if fits else skill_family_original[id]
func _install_skill_button_backgrounds() -> void:
	if skill_button_backgrounds_attempted or not player_action_states.has_all(['pulse','pierce','dash']) or not is_instance_valid(guard_button):
		return
	var targets: Array[Button] = []
	for id in ['pulse','pierce','dash']:
		var state = player_action_states[id]
		if not state is Dictionary or not state.has('button') or not is_instance_valid(state.button) or not state.button is Button:
			return
		targets.append(state.button)
	targets.append(guard_button)
	for button in targets:
		if not is_instance_valid(button) or button.get_parent()==null or not button.is_inside_tree():
			return
	skill_button_backgrounds_attempted = true
	for button in targets:
		HudTheme.install_native_button_backgrounds(button)
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
	button.text = {'primaryAttack':'攻击','dash':'Shift','pulse':'Q','pierce':'R'}[action_id]
	button.expand_icon = true
	button.add_theme_constant_override('icon_max_width',48)
	button.tooltip_text = '尝试'+PLAYER_ACTION_LABELS[action_id]+('；使用当前瞄准方向。' if action_id in ['primaryAttack','pierce'] else '。')+('\n'+Icons.USE_TEXT[action_id] if Icons.USE_TEXT.has(action_id) else '')
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
func _pad_trace(stage: String, control_id: String, reason: String, facts: Dictionary = {}) -> void:
	if pad_trace_enabled:
		pad_trace.emit(stage,control_id,reason,facts)
func _cancel_movement_capture(state: Dictionary, reason: String = 'clear') -> void:
	if state.capture.is_empty():
		return
	var captured = state.capture.duplicate(true)
	state.capture.clear()
	_pad_trace('hud',captured.entityId,reason,{'captured':true})
	intent.emit('movement_current_end',captured)
func _clear_movement_captures() -> void:
	for state in movement_states.values():
		_cancel_movement_capture(state)
func _movement_capture_valid(value) -> bool:
	if not movement_pad_enabled or not _movement_association_valid(value) or not movement_states.has(value.entityId) or not Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
		return false
	var state: Dictionary = movement_states[value.entityId]
	var button: Button = state.button
	return not button.disabled and button.is_visible_in_tree() and Rect2(Vector2.ZERO,button.size).has_point(button.get_local_mouse_position()) and _interaction_identity_same(value,state.display) and _interaction_identity_same(value,state.capture)
func _add_movement_button(buttons: GridContainer, direction_id: String) -> void:
	var button = Button.new()
	var state = {'button':button,'display':{},'capture':{}}
	movement_states[direction_id] = state
	button.name = direction_id
	button.text = MOVEMENT_ARROWS[direction_id]
	button.tooltip_text = MOVEMENT_LABELS[direction_id]+' · 按住移动；松开或移出停止。'
	button.mouse_filter = Control.MOUSE_FILTER_STOP
	button.focus_mode = Control.FOCUS_NONE
	button.disabled = true
	button.visible = false
	button.button_down.connect(func():
		var rejected: bool = not movement_pad_enabled or button.disabled or not button.is_visible_in_tree() or not Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) or not Rect2(Vector2.ZERO,button.size).has_point(button.get_local_mouse_position()) or not state.capture.is_empty() or not _movement_association_valid(state.display)
		_pad_trace('hud',direction_id,'down',{'pressed':true,'admitted':not rejected})
		if rejected:
			return
		state.capture = state.display.duplicate(true)
		intent.emit('movement_current_begin',state.capture.duplicate(true)))
	button.button_up.connect(func():
		_pad_trace('hud',direction_id,'up',{'pressed':false})
		_cancel_movement_capture(state,'release'))
	button.mouse_exited.connect(func(): _cancel_movement_capture(state,'leave'))
	button.visibility_changed.connect(func():
		if not button.is_visible_in_tree(): _cancel_movement_capture(state,'hide'))
	button.tree_exiting.connect(func(): _cancel_movement_capture(state,'remove'))
	buttons.add_child(button)
	button_nodes.append(button)
func _update_movements(s: Dictionary, eligible: bool, movements, enabled: bool = false, owner: Dictionary = {}) -> void:
	movement_pad_enabled = enabled
	movement_toggle_owner = owner.duplicate(true) if eligible else {}
	movement_toggle.disabled = not eligible
	movement_toggle.set_pressed_no_signal(enabled)
	movement_pad.visible = enabled and eligible
	movement_help.visible = enabled and eligible
	var selected: Dictionary = {}
	var common_generation = -1
	if enabled and eligible and P.snapshot(s) and movements is Array and movements.size()==8:
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
			_cancel_movement_capture(state,'projection_change')
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
		if not state.capture.is_empty() and not _movement_capture_valid(state.capture):
			_cancel_movement_capture(state,'capture_invalid')
func update_view(s: Dictionary, connection: String, eligible: bool, interaction: String, scale: float, target = {}, traversal = {}, actions = [], guard = {}, movements = [], movement_enabled: bool = false, movement_owner: Dictionary = {}, combat_lines: Array = [], priority_lines: Array = [], orientation: String = '') -> void:
	var valid_snapshot = P.snapshot(s)
	var old_hp = hp_label.text
	var old_energy = energy_label.text
	var vital_layout_needed: bool = vitals_snapshot_valid != valid_snapshot or layout_scale != scale
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
	_update_movements(s,eligible,movements,movement_enabled,movement_owner)
	movement_scroll.visible = false
	player_action_states.primaryAttack.button.visible = false
	status_label.text = connection if s.is_empty() else {'return_station':'归航站','grey_hive':'灰巢设施','mist_harbor':'雾港余烬','clockworks':'钟骨工厂'}.get(s.worldId,s.worldId)+' · '+Content.SCENE_NAMES.get(s.sceneId,'当前场景')+' · '+connection
	if not s.is_empty():
		var region: Dictionary = gh_dense_region_visual.current_region(s)
		if not region.is_empty(): status_label.text += ' · '+str(region.name)
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
	# Keep current authored mechanism/checkpoint status and contextual feedback in the upper rail.
	var mechanism_lines: Array = WorldMechanism.project(s,eligible)
	for line in mechanism_lines:
		if typeof(line)==TYPE_STRING and not line.is_empty():
			lines.append(line)
	var checkpoint_text: String = CheckpointStatus.project(s,eligible)
	if not checkpoint_text.is_empty():
		lines.append(checkpoint_text)
	if not interaction.is_empty():
		lines.append(interaction)
	var traversal_cue = '[Space] '+traverse_desired.label if not traverse_desired.is_empty() and not traverse_desired.label.is_empty() else ''
	if not traversal_cue.is_empty() and not lines.has(traversal_cue):
		lines.append(traversal_cue)
	var detail_lines = lines.duplicate()
	if not display.current_action.is_empty(): detail_lines.append(display.current_action)
	if not combat_lines.is_empty(): detail_lines.append('最近战斗 · '+str(combat_lines.back()))
	if not orientation.is_empty(): detail_lines.append(orientation)
	for guidance_line in display.guidance:
		if not detail_lines.has(guidance_line): detail_lines.append(guidance_line)
	var full_details = status_label.text+'\n\n'+'\n'.join(PackedStringArray(detail_lines))
	if details_text.text!=full_details:
		details_text.text = full_details
	var summary: Array[String] = []
	if not display.hasActiveObjective and not display.guidance_compact.is_empty():
		summary.append(display.guidance_compact)
	else:
		for line in lines:
			if typeof(line)==TYPE_STRING and not line.is_empty():
				summary.append(line)
				break
	if not interaction.is_empty() and not summary.has(interaction): summary.append(interaction)
	var summary_text='\n'.join(PackedStringArray(summary))
	if objectives.text!=summary_text:
		objectives.text=summary_text
		_queue_top_layout()
	var visible_priority = priority_lines.duplicate()
	if not traversal_cue.is_empty() and not visible_priority.has(traversal_cue):
		visible_priority.append(traversal_cue)
	if not orientation.is_empty() and visible_priority.is_empty() and interaction.is_empty() and display.objectives.is_empty() and display.action.is_empty():
		visible_priority.append(orientation)
	hint.text = '\n'.join(PackedStringArray(visible_priority))
	hint.visible = not visible_priority.is_empty()
	current_action_feedback.text = display.current_action
	current_action_feedback.visible = not display.current_action.is_empty()
	general_help.visible = false
	general_help.text = (interaction+'\n' if not interaction.is_empty() else '')+'WASD / 方向键移动 · J / 鼠标左键攻击 · 空格：场景穿越'
	if not display.action.is_empty() and display.action.kind=='primaryAttack':
		general_help.text += '\n'+display.action.text
	mechanism.text = '\n'.join(PackedStringArray(mechanism_lines))
	mechanism.visible = false
	checkpoint.text = checkpoint_text
	checkpoint.visible = false
	var compact_combat: bool = true
	combat_feedback.max_lines_visible = 1 if compact_combat else 4
	combat_feedback.text = ('最近战斗 · '+str(combat_lines.back()) if compact_combat else '最近战斗\n'+'\n'.join(PackedStringArray(combat_lines))) if not combat_lines.is_empty() else ''
	combat_feedback.visible = not combat_feedback.text.is_empty()
	if _applied_scale!=scale:
		for label in [portrait,status_label,hp_label,energy_label,objectives,details_text,hint,current_action_feedback,general_help,mechanism,checkpoint,combat_feedback]:
			label.add_theme_font_size_override('font_size',int(16*scale))
		for button in [details_button,details_close,details_up,details_down]:
			button.add_theme_font_size_override('font_size',int(16*scale))
		for button in button_nodes:
			button.add_theme_font_size_override('font_size',int(16*scale))
			button.custom_minimum_size.y = 48*scale
			button.add_theme_constant_override('icon_max_width',int(48*scale))
		for label in [movement_toggle,movement_help,movement_pad.get_node('MouseHoldPadStop')]:
			label.add_theme_font_size_override('font_size',int(16*scale))
		movement_toggle.custom_minimum_size.y = 32*scale
		movement_column.custom_minimum_size.x = 260*scale
		movement_scroll.custom_minimum_size.x = 260*scale
		bottom_columns.add_theme_constant_override('separation',int(12*scale))
		_applied_scale = scale
	icons.update_display(scale,'' if s.is_empty() else s.player.actionState,eligible,display.action)
	if layout_scale!=scale:
		layout_scale = scale
		_queue_bottom_layout()
		_queue_top_layout()
	if old_hp!=hp_label.text or old_energy!=energy_label.text:
		_queue_top_layout()
	if vital_layout_needed:
		call_deferred('_layout_vital_meters')
	_queue_hud_backings()
