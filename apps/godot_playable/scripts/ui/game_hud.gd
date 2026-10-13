extends Control
const Content = preload('res://scripts/ui/content_model.gd')
const P = preload('res://scripts/bridge/protocol.gd')
const Icons = preload('res://scripts/ui/skill_icon_row.gd')
const PlayStatus = preload('res://scripts/ui/play_status_model.gd')
const WorldMechanism = preload('res://scripts/ui/world_mechanism_model.gd')
signal intent(kind, value)
var portrait: Label
var button_nodes: Array = []
var status_label: Label
var meters: Label
var objectives: Label
var hint: Label
var mechanism: Label
var icons
var bottom: ScrollContainer
var bottom_content: VBoxContainer
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
	meters = Label.new()
	meters.position = Vector2(60,44)
	meters.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(meters)
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
	hint = Label.new()
	hint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	hint.mouse_filter = Control.MOUSE_FILTER_IGNORE
	side.add_child(hint)
	mechanism = Label.new()
	mechanism.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	mechanism.mouse_filter = Control.MOUSE_FILTER_IGNORE
	mechanism.visible = false
	side.add_child(mechanism)
	var buttons = HFlowContainer.new()
	side.add_child(buttons)
	for item in [['mission','任务'],['inventory','背包'],['character','角色'],['capability','能力'],['map','地图'],['pause','菜单']]:
		var button = Button.new()
		button.text = item[1]
		button.pressed.connect(func(): intent.emit(item[0],null))
		buttons.add_child(button)
		button_nodes.append(button)
	call_deferred('_load_internal_icons')
	resized.connect(_queue_bottom_layout)
	bottom.resized.connect(_queue_bottom_layout)
	bottom_content.minimum_size_changed.connect(_queue_bottom_layout)
	_queue_bottom_layout()
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
	layout_busy = false
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
func update_view(s: Dictionary, connection: String, eligible: bool, interaction: String, scale: float) -> void:
	status_label.text = connection if s.is_empty() else {'return_station':'归航站','grey_hive':'灰巢设施','mist_harbor':'雾港余烬','clockworks':'钟骨工厂'}.get(s.worldId,s.worldId)+' · '+Content.SCENE_NAMES.get(s.sceneId,'当前场景')+' · '+connection
	meters.text = '' if s.is_empty() else '岑遥 · 生命 %d / %d    能量 %d / %d'%[s.player.currentHp,s.player.maxHp,s.player.currentEnergy,s.player.maxEnergy]
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
	for label in [portrait,status_label,meters,objectives,hint,mechanism]:
		label.add_theme_font_size_override('font_size',int(16*scale))
	for button in button_nodes:
		button.add_theme_font_size_override('font_size',int(16*scale))
		button.custom_minimum_size.y = 32*scale
	icons.update_display(scale,'' if s.is_empty() else s.player.actionState,eligible,display.action)
	if layout_scale!=scale:
		layout_scale = scale
		_queue_bottom_layout()
