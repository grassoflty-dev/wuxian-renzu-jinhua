extends Control
const JourneyPlayStatus = preload('res://scripts/ui/play_status_model.gd')
const StyleBButtonArt = preload('res://scripts/ui/style_b_button_art.gd')
const UiArt = preload('res://scripts/ui/ui_art_texture.gd')
const RsBackdrop = preload('res://scripts/world/rs_exterior_backplate_visual.gd')
const Content = preload('res://scripts/ui/content_model.gd')
const P = preload('res://scripts/bridge/protocol.gd')
const BAIZHI_LINES = {'taken':'我会帮你离开这里。','left':'我不能带你走。'}
const MapView = preload('res://scripts/ui/map_panel.gd')
const ThemeModel = preload('res://scripts/ui/theme_model.gd')
const CAPS = {'information.local_map_i':'局部地图','information.enemy_vitals_basic':'敌人生命信息','perception.rear_view_i':'后方视野','perception.acoustic_mapping_i':'声学映射','body.regeneration_i':'再生能力','mobility.air_step_i':'空中踏步'}
const WORLDS = {'return_station':'归航站','grey_hive':'灰巢设施','mist_harbor':'雾港余烬','clockworks':'钟骨工厂'}
signal intent(kind, value, page_owner)
var mode = ''
var page_owner: Dictionary = {}
var enhancement_preview = ''
var enhancement_preview_owner: Dictionary = {}
var box: VBoxContainer
var save_entry: LineEdit
var retained_save_entry: LineEdit
var title: Label
var status: Label
var snapshot: Dictionary = {}
var baizhi_region_authorized := true
var slots: Array = []
var default_save_state = 'unknown'
var slot_list_state = 'unknown'
var connected = false
var pending = false
var preferences = {'scale':1.0,'fullscreen':false,'color':false,'reduced':false,'muted':false,'volume':0.35}
var staged = {}
var settings_build_id = 0
var settings_apply_control: Button
var slot_continue_buttons: Dictionary = {}
var font_scale = 1.0
var normal_button_texture: AtlasTexture
var panel: PanelContainer
var panel_b_skin: Control
var panel_b_patches: Array = []
var panel_b_style: StyleBoxEmpty
var panel_background: TextureRect
var shade: ColorRect
var start_backdrop: TextureRect
var world_loading_active: bool = false
var scroll: ScrollContainer
var rear_status_label: Label
var rear_activate_button: Button
var rear_grant_label: Label
var rear_selection_label: Label
var rear_select_button: Button
var rear_select_intent: Dictionary = {}
var rear_view_presented: Dictionary = {}
var rear_view_key: Array = []
var layout_busy = false
func _ready() -> void:
	ThemeModel.install_tooltip_skin(self)
	normal_button_texture = UiArt.atlas('res://assets/ui/global_ui_button.png',Rect2(200,445,1100,130))
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_STOP
	focus_mode = Control.FOCUS_ALL
	shade = ColorRect.new()
	shade.color = Color(0.02,0.04,0.06,0.82)
	shade.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(shade)
	start_backdrop = TextureRect.new()
	start_backdrop.texture = RsBackdrop.get_whole_texture()
	start_backdrop.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	start_backdrop.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	start_backdrop.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_COVERED
	start_backdrop.mouse_filter = Control.MOUSE_FILTER_IGNORE
	start_backdrop.visible = false
	add_child(start_backdrop)
	_setup_panel_b_skin()
	panel = PanelContainer.new()
	panel.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	panel.offset_left = 100
	panel.offset_right = -100
	panel.offset_top = 80
	panel.offset_bottom = -80
	add_child(panel)
	var style = StyleBoxFlat.new()
	style.bg_color = Color('#16202a')
	style.border_color = Color('#344652')
	style.set_border_width_all(2)
	style.content_margin_left = 24
	style.content_margin_right = 24
	style.content_margin_top = 18
	style.content_margin_bottom = 18
	panel.add_theme_stylebox_override('panel',style)
	if panel_b_skin != null:
		panel_b_style = StyleBoxEmpty.new()
		panel.add_theme_stylebox_override('panel',panel_b_style)
	panel.resized.connect(_layout_panel_b_skin)
	# Use only the quiet central material; the illustrated frame is not slice-safe.
	var panel_texture = _load_panel_texture()
	panel_background = TextureRect.new()
	if panel_texture != null:
		var panel_material = AtlasTexture.new()
		panel_material.atlas = panel_texture
		panel_material.region = Rect2(160,235,1305,465)
		panel_background.texture = panel_material
	panel_background.visible = panel_b_skin == null
	panel_background.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	panel_background.stretch_mode = TextureRect.STRETCH_SCALE
	panel_background.mouse_filter = Control.MOUSE_FILTER_IGNORE
	panel.add_child(panel_background)
	scroll = ScrollContainer.new()
	# Keep keyboard-focused controls visible when scaled pages need scrolling.
	scroll.follow_focus = true
	# Ignore child horizontal minimums so an earlier wider page cannot block shrink.
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_NEVER
	panel.add_child(scroll)
	ThemeModel.install_range_track(scroll.get_v_scroll_bar(),'scroll',true)
	box = VBoxContainer.new()
	box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	box.custom_minimum_size.x = 0
	scroll.add_child(box)
	resized.connect(_layout_content)
	scroll.resized.connect(_layout_content)
	visible = false
	call_deferred('_layout_content')
func _setup_panel_b_skin() -> void:
	var full = UiArt.atlas('res://assets/ui/global_ui_panel_b.png',Rect2(0,0,1536,1024))
	if full == null:
		return
	panel_b_skin = Control.new()
	panel_b_skin.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(panel_b_skin)
	var xs = [0,96,1440,1536]
	var ys = [0,96,928,1024]
	for row in range(3):
		for col in range(3):
			var region = AtlasTexture.new()
			region.atlas = full.atlas
			region.region = Rect2(xs[col],ys[row],xs[col+1]-xs[col],ys[row+1]-ys[row])
			var patch = TextureRect.new()
			patch.texture = region
			patch.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
			patch.stretch_mode = TextureRect.STRETCH_SCALE
			patch.mouse_filter = Control.MOUSE_FILTER_IGNORE
			panel_b_skin.add_child(patch)
			panel_b_patches.append(patch)

func _layout_panel_b_skin() -> void:
	if not is_instance_valid(panel_b_skin) or not is_instance_valid(panel) or panel_b_style == null:
		return
	var extent = panel.size
	if not is_finite(extent.x) or not is_finite(extent.y) or extent.x<=0 or extent.y<=0:
		return
	panel_b_skin.position = panel.position
	panel_b_skin.size = extent
	var scale = font_scale if P.number(font_scale,1,1.5) else 1.0
	var q = 0.75*scale
	var cap = 96.0*q
	var framed = extent.x>=320.0*scale and extent.y>=180.0*scale and extent.x>2.0*cap and extent.y>2.0*cap
	var inset_x = 64.0*q if framed else 24.0
	var inset_y = 64.0*q if framed else 18.0
	if panel_b_style.content_margin_left!=inset_x or panel_b_style.content_margin_top!=inset_y:
		panel_b_style.content_margin_left = inset_x
		panel_b_style.content_margin_right = inset_x
		panel_b_style.content_margin_top = inset_y
		panel_b_style.content_margin_bottom = inset_y
	if not framed:
		for index in range(9):
			var patch: TextureRect = panel_b_patches[index]
			patch.visible = index==4
			if index==4:
				patch.position = Vector2.ZERO
				patch.size = extent
		return
	var x = [0.0,cap,extent.x-cap,extent.x]
	var y = [0.0,cap,extent.y-cap,extent.y]
	for row in range(3):
		for col in range(3):
			var patch: TextureRect = panel_b_patches[row*3+col]
			patch.visible = true
			patch.position = Vector2(x[col],y[row])
			patch.size = Vector2(x[col+1]-x[col],y[row+1]-y[row])
func _load_panel_texture() -> Texture2D:
	var path = 'res://assets/ui/global_panel_skin_original.png'
	var texture: Texture2D = null
	if ResourceLoader.exists(path, 'Texture2D'):
		texture = ResourceLoader.load(path, 'Texture2D') as Texture2D
	if texture == null:
		var image = Image.new()
		if image.load(path) == OK:
			texture = ImageTexture.create_from_image(image)
	if texture == null or texture.get_width() < 1465 or texture.get_height() < 700:
		return null
	return texture
func _layout_content() -> void:
	if layout_busy or not is_instance_valid(scroll) or not is_instance_valid(box) or not is_finite(size.x) or not is_finite(size.y) or size.x<=48 or size.y<=36:
		return
	layout_busy = true
	var scale = font_scale if P.number(font_scale,1,1.5) else 1.0
	var bar_width = scroll.get_v_scroll_bar().get_combined_minimum_size().x
	if not is_finite(bar_width) or bar_width<0:
		layout_busy = false
		return
	var horizontal = clampf((size.x-48-bar_width-360*scale)*0.5,0,100)
	var vertical = clampf((size.y-36-200*scale)*0.5,0,80)
	if panel.offset_left!=horizontal:
		panel.offset_left = horizontal
		panel.offset_right = -horizontal
	if panel.offset_top!=vertical:
		panel.offset_top = vertical
		panel.offset_bottom = -vertical
	# Use the actual scroll size; zero pre-layout dimensions defer to its resize.
	var width = scroll.size.x-bar_width
	if is_finite(width) and width>0 and box.custom_minimum_size.x!=width:
		box.custom_minimum_size.x = width
	_layout_panel_b_skin()
	layout_busy = false
func set_baizhi_region_visibility(s:Dictionary,allowed:bool) -> void:
	var changed=baizhi_region_authorized!=allowed or (not snapshot.is_empty() and not P.same(snapshot,s))
	baizhi_region_authorized=allowed
	if visible and mode in ['baizhi','baizhi_confirm','dialogue']:
		snapshot=s
		if changed and (mode!='dialogue' or page_owner.get('baizhiOwned')==true): _rebuild()

func show_page(kind: String, association: Dictionary, s: Dictionary, is_connected: bool, blocked: bool = false, preserve_save_entry: bool = false) -> void:
	var retain_entry = preserve_save_entry and _can_retain_save_entry(kind,association,s,is_connected,blocked)
	retained_save_entry = save_entry if retain_entry else null
	var opening = not visible
	_clear_enhancement_preview()
	mode = kind
	panel_background.visible = panel_b_skin == null and panel_background.texture != null and kind in ['pause','save','settings']
	page_owner = association.duplicate(true)
	snapshot = s
	connected = is_connected
	pending = blocked
	visible = true
	if kind=='settings':
		staged = preferences.duplicate(true)
	_rebuild()
	_layout_content()
	# Take focus off the background without selecting a confirmation action.
	if opening:
		grab_focus()
func _can_retain_save_entry(kind: String, association: Dictionary, incoming_snapshot: Dictionary, incoming_connected: bool, incoming_blocked: bool) -> bool:
	if not visible or mode!='save' or kind!='save' or not connected or not incoming_connected or pending or incoming_blocked:
		return false
	if not P.snapshot(snapshot) or not P.snapshot(incoming_snapshot):
		return false
	var old_context = page_owner.get('context')
	var new_context = association.get('context')
	if not P.context(old_context) or not P.context(new_context):
		return false
	if not P.same(snapshot,incoming_snapshot) or not P.same(snapshot,old_context) or not P.same(snapshot,new_context):
		return false
	var old_generation = page_owner.get('generation')
	var new_generation = association.get('generation')
	var old_panel_generation = page_owner.get('panelGeneration')
	var new_panel_generation = association.get('panelGeneration')
	if not P.counter(old_generation,1) or not P.counter(new_generation,1) or old_generation!=new_generation:
		return false
	if not P.counter(old_panel_generation,1) or not P.counter(new_panel_generation,1) or new_panel_generation<=old_panel_generation:
		return false
	if retained_save_entry!=null or not is_instance_valid(save_entry) or save_entry.is_queued_for_deletion() or not is_instance_valid(box) or save_entry.get_parent()!=box:
		return false
	return true
func close_page() -> void:
	_clear_enhancement_preview()
	retained_save_entry = null
	visible = false
	mode = ''
	page_owner.clear()
func set_save_availability(default_state: String, list_state: String, rows: Array) -> void:
	default_save_state = default_state
	slot_list_state = list_state
	slots = rows.duplicate(true) if list_state=='list' else []
func _save_availability_text() -> void:
	_text({'unknown':'默认存档状态未确认','pending':'正在确认默认存档','available':'默认存档可继续','unavailable':'尚无默认存档','invalid':'默认存档响应无效；请刷新','error':'默认存档读取失败；请刷新'}.get(default_save_state,'默认存档状态未确认'))
	_text({'unknown':'记录列表尚未确认。可刷新后重试。' if connected else '记录列表尚未确认。','pending':'正在读取记录列表…','empty':'暂无已保存记录。','list':'当前存档列表已确认','invalid':'记录列表响应无效；请刷新','error':'记录列表读取失败；请刷新'}.get(slot_list_state,'记录列表尚未确认。'))
func set_message(text: String, blocked: bool = false) -> void:
	pending = blocked
	if mode=='settings' and is_instance_valid(settings_apply_control): settings_apply_control.disabled = pending or staged==preferences
	if is_instance_valid(status):
		status.text = text
func _clear() -> void:
	slot_continue_buttons.clear()
	settings_apply_control = null
	rear_status_label = null
	rear_activate_button = null
	rear_grant_label = null
	rear_selection_label = null
	rear_select_button = null
	rear_select_intent.clear()
	rear_view_presented.clear()
	rear_view_key.clear()
	var retained = retained_save_entry
	save_entry = null
	for child in box.get_children():
		if is_instance_valid(retained) and child==retained:
			continue
		box.remove_child(child)
		child.queue_free()
func _text(text: String) -> Label:
	var label = Label.new()
	label.text = text
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.add_theme_font_size_override('font_size',int(18*font_scale))
	box.add_child(label)
	return label
func _button(text: String, action: String, value = null, enabled: bool = true) -> Button:
	var button = Button.new()
	button.text = text
	button.tooltip_text = text
	button.clip_text = true
	button.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	button.disabled = not enabled or pending
	button.custom_minimum_size.y = 38*font_scale
	button.add_theme_font_size_override('font_size',int(18*font_scale))
	# The quiet center scales without stretching the unsliced decorated border.
	if normal_button_texture != null:
		var normal_style = StyleBoxTexture.new()
		normal_style.texture = normal_button_texture
		normal_style.content_margin_left = 8
		normal_style.content_margin_right = 8
		normal_style.content_margin_top = 4
		normal_style.content_margin_bottom = 4
		button.add_theme_stylebox_override('normal',normal_style)
	var captured = page_owner.duplicate(true)
	button.pressed.connect(func():
		if action=='enhancement_choose' and (not _enhancement_ready(captured) or enhancement_preview_owner!=captured or value!=enhancement_preview):
			return
		if page_owner==captured and visible and not pending:
			intent.emit(action,value,captured))
	box.add_child(button)
	var art = StyleBButtonArt.new()
	art.panel_owner = self
	art.font_scale = font_scale
	button.add_child(art)
	return button
func set_world_loading_active(value: bool) -> void:
	world_loading_active = value
	_apply_world_loading_presentation()
func _apply_world_loading_presentation() -> void:
	var show_body = not (mode=='waiting' and world_loading_active)
	var show_start_art = show_body and mode=='start' and is_instance_valid(start_backdrop) and start_backdrop.texture != null
	if is_instance_valid(start_backdrop):
		start_backdrop.visible = show_start_art
	if is_instance_valid(shade):
		shade.visible = show_body and not show_start_art
	if is_instance_valid(panel):
		panel.visible = show_body
	if is_instance_valid(panel_b_skin):
		panel_b_skin.visible = show_body
func _rebuild() -> void:
	_apply_world_loading_presentation()
	_clear()
	title = _text({'start':'无限人族进化 · 归航','pause':'暂停','new_confirm':'开始新的旅程？','exit_confirm':'保存并退出？','named_save_confirm':'确认记录操作','settings':'系统设置','inventory':'背包与装备','character':'岑遥','capability':'能力','enhancement':'首次进化','mission':'任务与世界网络','archive':'档案','save':'存档','slots':'继续旅程','map':'地图','baizhi':'白芷','baizhi_confirm':'确认选择','dialogue':'信息','death':'旅程中断','error':'链路状态','waiting':'等待权威确认'}.get(mode,mode))
	if mode in ['baizhi','baizhi_confirm'] and not baizhi_region_authorized: title.text='信息暂不可用'
	status = _text('等待确认' if pending else '')
	match mode:
		'start':
			_text('已连接权威服务' if connected else '尚未连接；需要专属会话启动参数')
			_button('新的旅程','new_confirm',null,connected)
			_button('继续','slots',null,connected)
			_button('设置','settings')
			_button('退出','close_client')
		'pause':
			_button('继续当前旅程','resume',null,connected)
			_text('操作参考\nWASD / 方向键移动，鼠标指向瞄准；J / 左键攻击，Shift 冲刺。Q 脉冲，按住 E 防御，R 穿刺；F 交互，Space 情境穿越。T 任务，M 地图，Esc 暂停；动作与地图信息以当前状态和权限为准。')
			for item in [['mission','任务 / 世界网络'],['inventory','背包'],['character','角色'],['capability','能力'],['archive','档案'],['map','地图'],['save','保存'],['settings','设置']]:
				_button(item[1],item[0])
			_button('返回归航站','return',null,connected)
			_button('新的旅程','new_confirm',null,connected)
			_button('保存并退出','exit_confirm',null,connected)
		'new_confirm':
			_text('未保存进度将丢失。新旅程由权威服务重置。')
			_button('确认新旅程','new',null,connected)
			_button('返回','back')
		'exit_confirm':
			_text('成功保存回执到达后才请求关闭。失败或结果未知会留在此界面。')
			_button('保存并退出','save_exit',null,connected)
			_button('返回','back')
		'named_save_confirm':
			_named_save_confirmation()
		'settings': _settings()
		'inventory': _inventory()
		'character':
			_character_sheet()
			_exploration_resume()
			_button('返回','back')
		'capability': _capability()
		'enhancement':
			_enhancement()
		'mission': _mission()
		'archive': _archive()
		'save': _save()
		'slots': _slots()
		'map':
			if not snapshot.is_empty():
				var map = MapView.new()
				map.custom_minimum_size = Vector2(0,440*font_scale)
				map.size_flags_horizontal = Control.SIZE_EXPAND_FILL
				map.text_scale = font_scale
				box.add_child(map)
				map.set_snapshot(snapshot)
			_exploration_resume()
			_button('返回','back')
		'baizhi':
			_baizhi_page()
		'baizhi_confirm':
			_baizhi_confirmation()
		'dialogue':
			_text('当前区域信息不可用。' if page_owner.get('baizhiOwned')==true and not baizhi_region_authorized else str(page_owner.get('text','')))
			_button('关闭','dialogue_close')
		'death':
			_text('当前角色已死亡；输入已停止。')
			_text('仅能继续此前已成功保存的进度；退出不会保存本次未保存进度。')
			_button('继续已有存档','slots',null,connected)
			_button('新的旅程','new_confirm',null,connected)
			_button('退出游戏（不保存）','close_client')
		'error':
			_text('游戏输入已停止。结果未知的操作不会自动重发。')
			_text('若权威状态已重新确认，可尝试「重新同步」并松开按键继续；否则请保留当前会话并退出。')
			_button('重新同步','resync',null,connected)
			_button('返回状态','start')
			_button('退出客户端','close_client')
		'waiting': _text('正在等待当前请求的真实回执。')
	retained_save_entry = null
func _baizhi_ticket_context_ready() -> bool:
	return connected and not pending and P.snapshot(snapshot) and snapshot.get('entryToken')==null and snapshot.worldId=='grey_hive' and snapshot.sceneId=='gh_bio_isolation' and P.context(page_owner.get('context')) and P.same(page_owner.context,snapshot)
func _baizhi_page_context_ready() -> bool:
	return baizhi_region_authorized and _baizhi_ticket_context_ready()
func _baizhi_choice_state() -> String:
	if not _baizhi_page_context_ready() or snapshot.player.currentHp<=0:
		return 'unknown'
	var b = snapshot.get('baizhi')
	if not b is Dictionary or b.size()!=4 or not b.has_all(['schemaVersion','choice','available','canInteract']) or not P.counter(b.schemaVersion,1,1) or typeof(b.choice)!=TYPE_STRING or not b.choice in ['unresolved','taken','left'] or typeof(b.available)!=TYPE_BOOL or not b.available or typeof(b.canInteract)!=TYPE_BOOL:
		return 'unknown'
	return b.choice
func _baizhi_page() -> void:
	if not baizhi_region_authorized:
		_text('当前区域信息不可用。')
		_button('结束对白','baizhi_close',null,_baizhi_ticket_context_ready())
		return
	match _baizhi_choice_state():
		'unresolved':
			var card = _mission_card(box,'baizhi_dialogue',ThemeModel.palette(snapshot.worldId))
			for line in ['别开那扇门。先听我说。','隔离室里的白芷等待你的回应。对白不会授予 Gate B 的钥匙。']:
				var label = _text(line)
				box.remove_child(label)
				card.add_child(label)
			_button(BAIZHI_LINES.taken,'baizhi_confirm','taken')
			_button(BAIZHI_LINES.left,'baizhi_confirm','left')
			_button('我还需要确认一些事情。','baizhi_unresolved')
		'taken':
			_text('已记录的白芷选择：我会帮你离开这里。')
		'left':
			_text('已记录的白芷选择：我不能带你走。')
		_:
			_text('白芷选择状态未确认；不能提交新选择。')
	_button('结束对白','baizhi_close',null,_baizhi_ticket_context_ready())

func _baizhi_confirmation() -> void:
	if not baizhi_region_authorized:
		_text('当前区域信息不可用。')
		_button('结束对白','baizhi_close',null,_baizhi_ticket_context_ready())
		return
	var choice = page_owner.get('choice')
	if _baizhi_choice_state()!='unresolved' or typeof(choice)!=TYPE_STRING or not BAIZHI_LINES.has(choice) or not P.ticket(page_owner.get('dialogueTicket'),page_owner.get('context',{})):
		_text('白芷选择状态未确认；不能提交新选择。')
		_button('结束对白','baizhi_close',null,_baizhi_ticket_context_ready())
		return
	title.text = '确认这项回应？'
	var card = _mission_card(box,'baizhi_choice',ThemeModel.palette(snapshot.worldId))
	for line in ['所选回应',BAIZHI_LINES[choice]]:
		var label = _text(line)
		box.remove_child(label)
		card.add_child(label)
	var back = _button('返回对话','baizhi_page',null,true)
	_button('确认回应','baizhi_commit',choice,true)
	_button('结束对白','baizhi_close',null,_baizhi_ticket_context_ready())
	back.grab_focus()
func _settings_control_current(owner: Dictionary, build_id: int, control: Control) -> bool:
	return visible and mode=='settings' and not pending and page_owner==owner and settings_build_id==build_id and is_instance_valid(control) and control.is_inside_tree() and not control.is_queued_for_deletion()

func _settings() -> void:
	settings_build_id += 1
	var build_id = settings_build_id
	var captured = page_owner.duplicate(true)
	_text('仅当前会话生效，不会写入偏好文件。临时合成音频的实际输出尚未验证。')
	var colors = ThemeModel.palette(snapshot.get('worldId','return_station'))
	var display_card = _mission_card(box,'settings_display',colors)
	var assist_card = _mission_card(box,'settings_assist',colors)
	var audio_card = _mission_card(box,'settings_audio',colors)
	for pair in [[display_card,'显示'],[assist_card,'辅助'],[audio_card,'声音']]:
		var heading = _text(pair[1])
		box.remove_child(heading)
		pair[0].add_child(heading)
	var size_label = _text('文字大小')
	box.remove_child(size_label)
	display_card.add_child(size_label)
	var size_pick = OptionButton.new()
	for label in ['100%','125%','150%']: size_pick.add_item(label)
	size_pick.select([1.0,1.25,1.5].find(staged.scale))
	size_pick.add_theme_font_size_override('font_size',int(18*font_scale))
	display_card.add_child(size_pick)
	ThemeModel.install_choice_skin(size_pick)
	var checks = {}
	for pair in [['fullscreen','全屏'],['color','颜色辅助标记'],['reduced','减少动态效果'],['muted','静音']]:
		var check = CheckBox.new()
		check.text = pair[1]
		check.add_theme_font_size_override('font_size',int(18*font_scale))
		check.button_pressed = staged[pair[0]]
		checks[pair[0]] = check
		(display_card if pair[0]=='fullscreen' else audio_card if pair[0]=='muted' else assist_card).add_child(check)
	var volume_label = _text('主音量：'+str(int(round(staged.volume*100)))+'%')
	box.remove_child(volume_label)
	audio_card.add_child(volume_label)
	var slider = HSlider.new()
	slider.min_value = 0
	slider.max_value = 100
	slider.step = 1
	slider.value = staged.volume*100
	slider.custom_minimum_size.y = 36*font_scale
	audio_card.add_child(slider)
	ThemeModel.install_range_track(slider,'slider',false)
	var state_label = _text('当前设置已同步')
	var defaults = _button('恢复默认（待应用）','settings_defaults')
	var apply = _button('应用到本次会话','settings_apply')
	settings_apply_control = apply
	var refresh = func() -> void:
		if not _settings_control_current(captured,build_id,apply): return
		var dirty = staged!=preferences
		state_label.text = '有未应用的修改' if dirty else '当前设置已同步'
		apply.disabled = not dirty or pending
	refresh.call()
	size_pick.item_selected.connect(func(index):
		if _settings_control_current(captured,build_id,size_pick) and index>=0 and index<3:
			staged.scale=[1.0,1.25,1.5][index]
			refresh.call())
	for key in checks:
		var check: CheckBox = checks[key]
		var field: String = key
		check.toggled.connect(func(value):
			if _settings_control_current(captured,build_id,check):
				staged[field]=value
				refresh.call())
	slider.value_changed.connect(func(value):
		if _settings_control_current(captured,build_id,slider):
			staged.volume=value/100.0
			volume_label.text='主音量：'+str(int(value))+'%'
			refresh.call())
	defaults.pressed.connect(func():
		if _settings_control_current(captured,build_id,defaults):
			staged={'scale':1.0,'fullscreen':false,'color':false,'reduced':false,'muted':false,'volume':0.35}
			size_pick.select(0)
			for key in checks: checks[key].set_pressed_no_signal(false)
			slider.set_value_no_signal(35)
			volume_label.text='主音量：35%'
			refresh.call())
	apply.pressed.connect(func():
		if _settings_control_current(captured,build_id,apply) and not apply.disabled and staged!=preferences:
			intent.emit('apply_preferences',staged.duplicate(true),captured))
	_button('返回（放弃未应用修改）','back')
func _character_sheet() -> void:
	var color_world: String = snapshot.worldId if typeof(snapshot.get('worldId'))==TYPE_STRING else 'return_station'
	var colors = ThemeModel.palette(color_world)
	var state_card = _mission_card(box,'character_state',colors)
	var equipment_card = _mission_card(box,'character_equipment',colors)
	var capability_card = _mission_card(box,'character_capabilities',colors)
	_mission_label(state_card,'当前状态',colors,'system')
	_mission_label(equipment_card,'已装备',colors,'system')
	_mission_label(capability_card,'已获得能力',colors,'system')
	var valid = connected and not pending and P.snapshot(snapshot) and snapshot.get('entryToken')==null and snapshot.get('exitEntryPreview')!=true and snapshot.get('shaftServiceEntryPreview')!=true and P.context(page_owner.get('context')) and P.same(page_owner.context,snapshot)
	if not valid:
		_mission_label(state_card,'当前状态尚未确认',colors)
		_mission_label(equipment_card,'装备状态尚未确认',colors)
		_mission_label(capability_card,'能力状态尚未确认',colors)
		return
	_mission_label(state_card,'当前快照 · 生命 %d / %d\n能量 %d / %d'%[snapshot.player.currentHp,snapshot.player.maxHp,snapshot.player.currentEnergy,snapshot.player.maxEnergy],colors)
	var action_names = {'idle':'无当前动作','primaryAttack':'攻击','pulse':'脉冲','guard':'防御','pierce':'穿刺','dash':'冲刺'}
	if snapshot.player.currentHp==0:
		_mission_label(state_card,'快照状态：已死亡',colors)
	elif typeof(snapshot.player.get('actionState'))==TYPE_STRING and action_names.has(snapshot.player.actionState):
		_mission_label(state_card,'快照动作：'+action_names[snapshot.player.actionState],colors)
	if not P.build(snapshot.get('build')):
		_mission_label(equipment_card,'装备状态尚未确认',colors)
	elif snapshot.build.equipment.is_empty():
		_mission_label(equipment_card,'当前没有已装备物品',colors)
	else:
		var slot_names = {'charm':'饰物','lens':'镜片','map_array':'地图阵列','map_chip':'地图芯片','boots':'靴具','water_module':'浅水模块','lining':'内衬'}
		for item in snapshot.build.equipment:
			_mission_label(equipment_card,(slot_names[item.slotId]+' · ' if slot_names.has(item.slotId) else '已装备 · ')+item.label,colors)
	var capabilities = snapshot.get('capabilities')
	var raw_items = capabilities.get('items') if capabilities is Dictionary else null
	var capability_rows = P.capability_items(snapshot)
	if not raw_items is Array or not P.rows(raw_items,128) or not P.unique(raw_items,'capabilityId') or (not raw_items.is_empty() and capability_rows.is_empty()):
		_mission_label(capability_card,'能力状态尚未确认',colors)
	else:
		var granted = false
		for item in capability_rows:
			if not item.granted: continue
			granted = true
			_mission_label(capability_card,(CAPS[item.capabilityId] if CAPS.has(item.capabilityId) else '已获得能力（名称未确认）')+(' · 已选用' if item.selected else ' · 未选用'),colors)
		if not granted: _mission_label(capability_card,'尚未获得能力',colors)

func _exploration_resume() -> void:
	if P.snapshot(snapshot):
		_button('继续探索','resume',null,connected and snapshot.player.currentHp>0)
func _inventory() -> void:
	if snapshot.is_empty() or not P.build(snapshot.get('build')):
		_text('尚无有效权威背包投影；不创建物品或预设。')
	else:
		_text('物品 / 装备 / 描述 · 修订 '+str(snapshot.build.revision))
		var pane_flow = HFlowContainer.new()
		pane_flow.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		pane_flow.custom_minimum_size.x = 0.0
		box.add_child(pane_flow)
		var item_column = VBoxContainer.new()
		var equipment_column = VBoxContainer.new()
		var description_column = VBoxContainer.new()
		var pane_columns: Array = [item_column,equipment_column,description_column]
		for pane in pane_columns:
			pane.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			pane.custom_minimum_size.x = 0.0
			pane_flow.add_child(pane)
		var item_heading = _text('物品')
		box.remove_child(item_heading)
		item_heading.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		item_column.add_child(item_heading)
		if snapshot.build.items.is_empty():
			var empty_item_label = _text('背包投影中暂无物品。')
			box.remove_child(empty_item_label)
			empty_item_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			item_column.add_child(empty_item_label)
		var equipment_heading = _text('装备')
		box.remove_child(equipment_heading)
		equipment_heading.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		equipment_column.add_child(equipment_heading)
		if snapshot.build.equipment.is_empty():
			var empty_equipment_label = _text('装备投影中暂无已装备物品。')
			box.remove_child(empty_equipment_label)
			empty_equipment_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			equipment_column.add_child(empty_equipment_label)
		var description_heading = _text('描述')
		box.remove_child(description_heading)
		description_heading.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		description_column.add_child(description_heading)
		var detail_label = _text('暂无物品描述。' if snapshot.build.items.is_empty() else '物品描述将在选择后显示。')
		box.remove_child(detail_label)
		detail_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		description_column.add_child(detail_label)
		var detail_state = {'selected_item_id':''}
		var current_item_for_selection = func(captured_owner: Dictionary, captured_revision, requested_item_id: String) -> Dictionary:
			if not visible or mode != 'inventory' or not connected or pending:
				return {}
			if not P.snapshot(snapshot) or snapshot.get('entryToken') != null:
				return {}
			if snapshot.player.currentHp <= 0:
				return {}
			if not P.context(page_owner.get('context')) or not P.same(page_owner.context,snapshot):
				return {}
			if page_owner != captured_owner:
				return {}
			if not P.build(snapshot.get('build')) or snapshot.build.revision != captured_revision:
				return {}
			var matching_item: Dictionary = {}
			var match_count = 0
			for current_item in snapshot.build.items:
				if current_item.get('itemId') == requested_item_id:
					matching_item = current_item
					match_count += 1
			if match_count != 1:
				return {}
			return matching_item
		var select_inventory_item = func(captured_owner: Dictionary, captured_revision, requested_item_id: String) -> void:
			var actual_item: Dictionary = current_item_for_selection.call(captured_owner,captured_revision,requested_item_id)
			if actual_item.is_empty():
				return
			detail_state['selected_item_id'] = requested_item_id
			detail_label.text = actual_item.label+' ×'+str(actual_item.quantity)+'\n'+actual_item.description+'\n槽位：'+actual_item.slotId+(' · 已装备' if actual_item.equipped else '')
			if not actual_item.equipped:
				for equipped_item in snapshot.build.equipment:
					if equipped_item.slotId==actual_item.slotId:
						detail_label.text += '\n将替换：'+equipped_item.label
		for item in snapshot.build.items:
			var item_row = HBoxContainer.new()
			item_row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			item_row.custom_minimum_size.x = 0.0
			item_column.add_child(item_row)
			var selector = Button.new()
			var selector_text = item.label+' ×'+str(item.quantity)+(' · 已装备' if item.equipped else '')
			selector.text = selector_text
			selector.tooltip_text = selector_text
			selector.clip_text = true
			selector.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
			selector.custom_minimum_size.y = 38*font_scale
			selector.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			selector.add_theme_font_size_override('font_size',int(18*font_scale))
			var selector_owner: Dictionary = page_owner.duplicate(true)
			var selector_revision = snapshot.build.revision
			var selector_item_id: String = item.itemId
			var current_selector_item: Dictionary = current_item_for_selection.call(selector_owner,selector_revision,selector_item_id)
			selector.disabled = current_selector_item.is_empty()
			selector.pressed.connect(select_inventory_item.bind(selector_owner,selector_revision,selector_item_id))
			item_row.add_child(selector)
			var occupant = null
			var occupant_label = ''
			for equipment in snapshot.build.equipment:
				if equipment.slotId==item.slotId:
					occupant = equipment.itemId
					occupant_label = equipment.label
			var kind = 'unequip' if item.equipped else 'equip'
			var payload = {'revision':snapshot.build.revision,'itemId':item.itemId,'slotId':item.slotId,'equipped':item.equipped,'expected':occupant,'kind':kind}
			var action_button = _button('卸下' if item.equipped else ('替换' if occupant!=null else '装备'),'build_item',payload,not current_selector_item.is_empty())
			if not item.equipped and occupant!=null:
				action_button.tooltip_text = '将替换：'+occupant_label
			box.remove_child(action_button)
			item_row.add_child(action_button)
		for equipment in snapshot.build.equipment:
			var equipment_label = _text(equipment.label+'\n槽位：'+equipment.slotId)
			box.remove_child(equipment_label)
			equipment_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			equipment_column.add_child(equipment_label)
		var update_pane_minimums = func() -> void:
			var usable_width = pane_flow.size.x
			if not is_finite(usable_width) or usable_width <= 0.0:
				return
			var minimum_width = minf(usable_width,18.0*font_scale*12.0)
			for pane in pane_columns:
				if pane.custom_minimum_size.x != minimum_width:
					pane.custom_minimum_size.x = minimum_width
		pane_flow.resized.connect(update_pane_minimums)
		if pane_flow.size.x > 0.0:
			update_pane_minimums.call()
	_exploration_resume()
	_button('返回','back')
static func rear_view_readiness(s: Dictionary) -> Dictionary:
	if not P.snapshot(s): return {'state':'unknown'}
	for item in P.capability_items(s):
		if item.capabilityId!='perception.rear_view_i': continue
		if not item.granted: return {'state':'ungranted','granted':false,'selected':item.selected}
		if not item.has('cooldownRemainingMs'): return {'state':'unknown','granted':true,'selected':item.selected}
		var remaining = int(item.cooldownRemainingMs)
		return {'state':'ready' if remaining==0 else 'cooldown','remaining':remaining,'granted':true,'selected':item.selected}
	return {'state':'unknown'}

func update_rear_view(s: Dictionary, owner_ready: bool) -> void:
	if not visible or mode!='capability' or not is_instance_valid(rear_status_label) or not is_instance_valid(rear_activate_button): return
	var current = rear_view_readiness(s) if P.context(page_owner.get('context')) and P.same(page_owner.context,s) else {'state':'unknown'}
	var key = [current,owner_ready,connected,pending]
	if key==rear_view_key: return
	rear_view_key = key.duplicate(true)
	rear_view_presented = current.duplicate(true)
	if is_instance_valid(rear_grant_label): rear_grant_label.text = '获得状态：'+('已获得' if current.get('granted')==true else '未获得' if current.get('granted')==false else '尚未确认')
	if is_instance_valid(rear_selection_label): rear_selection_label.text = '选用状态：'+('已选用' if current.get('selected')==true else '未选用' if current.get('selected')==false else '尚未确认')
	if is_instance_valid(rear_select_button):
		rear_select_intent.clear()
		rear_select_button.visible = current.get('granted')==true
		if current.get('granted')==true and typeof(current.get('selected'))==TYPE_BOOL:
			rear_select_intent.merge({'id':'perception.rear_view_i','granted':true,'selected':current.selected})
			rear_select_button.text = '取消选择' if current.selected else '选择'
			rear_select_button.tooltip_text = rear_select_button.text
		rear_select_button.disabled = not owner_ready or not connected or pending or rear_select_intent.is_empty()
	rear_activate_button.visible = current.get('granted')==true
	match current.state:
		'ready': rear_status_label.text = '冷却：已结束（当前快照）'
		'cooldown': rear_status_label.text = '冷却：剩余约 %.1f 秒（当前快照）' % (ceil(float(current.remaining)/100.0)/10.0)
		_: rear_status_label.text = '冷却：尚未确认'
	rear_activate_button.disabled = not owner_ready or not connected or pending or current.state!='ready'

func _capability() -> void:
	if not P.snapshot(snapshot):
		_text('能力状态尚未同步')
	else:
		var use_text = {
			'information.local_map_i':'M / 地图：需先获得并选用本能力；只显示权威允许的已探索信息。',
			'information.enemy_vitals_basic':'生命信息：只显示健康、受伤、重伤、濒危四档，不显示精确生命数值。',
			'perception.rear_view_i':'后方视野：显示来自当前权威授权的信息。',
			'perception.acoustic_mapping_i':'声学映射：雾港声源方向与距离来自权威投影。',
			'body.regeneration_i':'再生：由权威自动处理，受伤与战斗后的恢复不由本页触发。',
			'mobility.air_step_i':'空中踏步：靠近场景穿越起点后按空格；权威验证位置、能力与冷却。'}
		var known = false
		for c in P.capability_items(snapshot):
			if not CAPS.has(c.capabilityId):
				continue
			if not known:
				_text('只显示永久能力状态；树关系与成本未提供')
				known = true
			var card = _mission_card(box,'cap_'+c.capabilityId,ThemeModel.palette(snapshot.worldId))
			var card_lines = [CAPS[c.capabilityId], '获得状态：'+('已获得' if c.granted else '未获得'), '选用状态：'+('已选用' if c.selected else '未选用'), use_text[c.capabilityId]]
			for index in range(card_lines.size()):
				var label = _text(card_lines[index])
				box.remove_child(label)
				card.add_child(label)
				if c.capabilityId=='perception.rear_view_i' and index==1: rear_grant_label = label
				if c.capabilityId=='perception.rear_view_i' and index==2: rear_selection_label = label
			if c.granted and c.capabilityId!='perception.rear_view_i':
				var select_button = _button('取消选择' if c.selected else '选择','capability_select',c.capabilityId,connected)
				box.remove_child(select_button)
				card.add_child(select_button)
			if c.capabilityId=='perception.rear_view_i':
				rear_select_button = _button('取消选择' if c.selected else '选择','capability_select',rear_select_intent,false)
				rear_select_button.visible = c.granted
				box.remove_child(rear_select_button)
				card.add_child(rear_select_button)
				rear_status_label = _text('冷却：尚未确认')
				box.remove_child(rear_status_label)
				card.add_child(rear_status_label)
				rear_activate_button = _button('确认后方视野','capability_activate',c.capabilityId,false)
				rear_activate_button.visible = c.granted
				box.remove_child(rear_activate_button)
				card.add_child(rear_activate_button)
		if not known:
			_text('当前能力状态未确认')
		_button('继续探索','resume',null,connected and P.snapshot(snapshot) and snapshot.player.currentHp>0)
	_button('返回','back')
func _mission_route_guidance() -> Array:
	var neutral = ['任务参考：当前路线状态未确认。']
	if not connected or pending or not P.snapshot(snapshot) or snapshot.player.currentHp<=0 or snapshot.get('entryToken')!=null:
		return []
	if not P.context(page_owner.get('context')) or not P.same(page_owner.context,snapshot):
		return neutral
	return JourneyPlayStatus.route_guidance(snapshot)
func _mission_revisit_limit_suffix(world: String, row: Dictionary) -> String:
	# Fixed caps from the frozen progression catalog and gameplay §33 policy.
	var limits = {'grey_hive':2,'mist_harbor':2,'clockworks':4}
	if mode != 'mission' or not limits.has(world) or not Content.EVENTS.has(world):
		return ''
	if not connected or pending or not P.snapshot(snapshot):
		return ''
	if snapshot.player.currentHp <= 0 or snapshot.get('entryToken') != null:
		return ''
	if not P.context(page_owner.get('context')) or not P.same(page_owner.context,snapshot):
		return ''
	var current: Dictionary = Content.progress(snapshot,world)
	if current.is_empty() or row != current:
		return ''
	if typeof(row.get('worldId')) != TYPE_STRING or row.worldId != world:
		return ''
	if typeof(row.get('completed')) != TYPE_BOOL or not row.completed:
		return ''
	if typeof(row.get('firstCompletion')) != TYPE_BOOL or not row.firstCompletion:
		return ''
	var limit: int = limits[world]
	if not P.counter(row.get('visitId'),1) or not P.counter(row.get('cycleId'),1,1) or not P.counter(row.get('revisitCount'),0,limit):
		return ''
	if row.visitId != row.revisitCount + 1 or row.cycleId != 1:
		return ''
	var completed_events = row.get('completedEvents')
	var canonical_events: Array = Content.EVENTS[world]
	if not completed_events is Array or completed_events.size() != 3 or canonical_events.size() != 3:
		return ''
	for event in canonical_events:
		if not event in completed_events:
			return ''
	return ' / ' + str(limit)
func _mission_card(parent: Control, key: String, colors: Dictionary) -> VBoxContainer:
	var card = PanelContainer.new()
	card.name = 'MissionCard_'+key
	card.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	card.custom_minimum_size.x = 0.0
	var style = StyleBoxFlat.new()
	style.bg_color = Color(colors.get('panelRaised','#1B2934'))
	style.border_color = Color(colors.get('border','#344652'))
	style.set_border_width_all(1)
	style.set_corner_radius_all(2)
	var padding = 12.0*font_scale
	style.content_margin_left = padding
	style.content_margin_right = padding
	style.content_margin_top = padding
	style.content_margin_bottom = padding
	card.add_theme_stylebox_override('panel',style)
	ThemeModel.install_hud_information_panel(card)
	var content = VBoxContainer.new()
	content.name = 'MissionCardContent'
	content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	content.custom_minimum_size.x = 0.0
	content.add_theme_constant_override('separation',int(6*font_scale))
	card.add_child(content)
	parent.add_child(card)
	return content
func _mission_label(parent: Control, text_value: String, colors: Dictionary, role: String = 'text', size_scale: float = 1.0) -> Label:
	var label = Label.new()
	label.text = text_value
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	label.custom_minimum_size.x = 0.0
	label.add_theme_font_size_override('font_size',int(18*font_scale*size_scale))
	label.add_theme_color_override('font_color',Color(colors.get(role,colors.get('text','#E7EDF2'))))
	parent.add_child(label)
	return label
func _mission() -> void:
	title.add_theme_font_size_override('font_size',int(24*font_scale))
	if snapshot.is_empty():
		_text('尚未收到当前旅程')
	elif not P.snapshot(snapshot):
		_text('当前旅程状态未确认')
	else:
		var colors = ThemeModel.palette(snapshot.worldId)
		var stack = VBoxContainer.new()
		stack.name = 'MissionContent'
		stack.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		stack.custom_minimum_size.x = 0.0
		stack.add_theme_constant_override('separation',int(8*font_scale))
		box.add_child(stack)
		var overview = _mission_card(stack,'overview',colors)
		_mission_label(overview,'当前世界：'+WORLDS.get(snapshot.worldId,'世界名称未确认'),colors,'system',1.12)
		_mission_label(overview,'当前场景：'+Content.SCENE_NAMES.get(snapshot.sceneId,'场景名称未确认'),colors,'textMuted')
		for line in _mission_route_guidance():
			_mission_label(overview,line,colors,'accent')
		for o in snapshot.objectives:
			_mission_label(overview,Content.objective(o),colors,'accent')
		for world in ['grey_hive','mist_harbor','clockworks']:
			var world_card = _mission_card(stack,world,colors)
			var row = Content.progress(snapshot,world)
			if row.is_empty():
				_mission_label(world_card,WORLDS[world]+' · 进度未确认',colors,'text',1.12)
			else:
				_mission_label(world_card,WORLDS[world]+' · '+('已完成' if row.completed else '尚未完成')+' · 复访 '+str(int(row.revisitCount))+_mission_revisit_limit_suffix(world,row),colors,'text',1.12)
				_mission_label(world_card,'事件记录 %d / 3'%row.completedEvents.size(),colors,'textMuted')
				for event in Content.EVENTS[world]:
					var recorded = event in row.completedEvents
					_mission_label(world_card,('✓ ' if recorded else '○ ')+Content.EVENT_LABELS[event]+(' · 已记录' if recorded else ' · 尚未记录'),colors)
		var context_lines: Array[String] = []
		if snapshot.worldId=='return_station' and snapshot.sceneId=='rs_core_room':
			for pair in [['rs_world_gate_marker','灰巢入口'],['rs_mh_world_gate_marker','雾港入口'],['rs_cw_world_gate_marker','钟骨入口']]:
				for i in snapshot.interactables:
					if i.entityId==pair[0] and i.kind=='world_gate':
						context_lines.append(pair[1]+' · '+('当前启用' if i.active else '当前未启用'))
		var beacon = snapshot.get('greyHiveBeacon')
		if snapshot.worldId=='grey_hive' and beacon is Dictionary and beacon.get('state') in ['uncollected','carried','mounted'] and typeof(beacon.get('legacyCompletedWithoutReceipt'))==TYPE_BOOL and (not beacon.legacyCompletedWithoutReceipt or beacon.state=='uncollected'):
			context_lines.append('信标：'+{'uncollected':'尚未收取','carried':'已携带','mounted':'已部署'}[beacon.state])
		var pump = snapshot.get('mistHarborPump')
		if snapshot.worldId=='mist_harbor' and pump is Dictionary and pump.get('state') in ['ready','draining','drained']:
			context_lines.append('泵站：'+{'ready':'就绪','draining':'排水中','drained':'排水完成'}[pump.state])
		if not context_lines.is_empty():
			var context_card = _mission_card(stack,'context',colors)
			for line in context_lines:
				_mission_label(context_card,line,colors)
		_mission_label(stack,'目标交互需在场景恢复后进行。',colors,'textMuted')
		if snapshot.worldId=='return_station' and snapshot.sceneId=='rs_core_room':
			_mission_label(stack,'返回场景、靠近世界门，再查看当前交互提示。',colors,'textMuted')
		_button('继续探索','resume',null,connected and P.snapshot(snapshot) and snapshot.player.currentHp>0)
	_button('档案','archive')
	_button('能力','capability')
	_button('系统','settings')
	_button('返回','back')
func _archive_saved_baizhi_choice() -> String:
	if not connected or pending or not P.snapshot(snapshot):
		return ''
	if snapshot.player.currentHp <= 0 or snapshot.get('entryToken') != null:
		return ''
	if not P.context(page_owner.get('context')) or not P.same(page_owner.context,snapshot):
		return ''
	var row = Content.progress(snapshot,'grey_hive')
	if row.is_empty() or row.get('worldId') != 'grey_hive':
		return ''
	var choice = row.get('hiveChoice')
	if typeof(choice) != TYPE_STRING:
		return ''
	if choice == 'taken':
		return '已记录的白芷选择：我会帮你离开这里。'
	if choice == 'left':
		return '已记录的白芷选择：我不能带你走。'
	return ''
func _archive() -> void:
	if snapshot.is_empty() or not P.snapshot(snapshot):
		_text('档案状态未确认')
	else:
		title.add_theme_font_size_override('font_size',int(24*font_scale))
		var colors = ThemeModel.palette(snapshot.worldId)
		var stack = VBoxContainer.new()
		stack.name = 'ArchiveContent'
		stack.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		stack.custom_minimum_size.x = 0.0
		stack.add_theme_constant_override('separation',int(8*font_scale))
		box.add_child(stack)
		var shown = false
		var saved_choice = _archive_saved_baizhi_choice()
		if not saved_choice.is_empty():
			var choice_card = _mission_card(stack,'archive_saved_baizhi_choice',colors)
			_mission_label(choice_card,saved_choice,colors)
			shown = true
		for world in ['grey_hive','mist_harbor','clockworks']:
			var row = Content.progress(snapshot,world)
			if row.is_empty():
				continue
			var records: Array[String] = []
			if row.get('worldId')=='grey_hive':
				if 'hive_power' in row.completedEvents:
					records.append('灰巢：主电恢复记录\n主电还活着，只是被人为切断。')
				if 'hive_lockdown' in row.completedEvents:
					records.append('灰巢：隔离协议记录\n隔离协议已被局部覆盖。')
			if row.get('worldId')=='mist_harbor' and 'mist_beacon_west' in row.completedEvents and 'mist_beacon_east' in row.completedEvents:
				records.append('雾港：双基准同步记录\n双基准建立，中心干扰源可定位。')
			if world=='mist_harbor' and 'mist_signal' in row.completedEvents and P.granted(snapshot,'perception.acoustic_mapping_i'):
				records.append('雾港：声学映射记录\n这次我不是在看路——是在听。')
			var complete = row.completed==true and row.firstCompletion==true and row.completedEvents.size()==3
			for event in Content.EVENTS[world]:
				complete = complete and event in row.completedEvents
			if complete:
				if world=='grey_hive':
					records.append('灰巢：归航记录\n灰巢信标已解析。检测到第二组低可信坐标：雾港余烬。')
				elif world=='mist_harbor' and P.granted(snapshot,'perception.acoustic_mapping_i'):
					records.append('雾港：归航记录\n声学映射已固化。第三坐标的机械周期与信号高度同步。')
				elif world=='clockworks' and P.granted(snapshot,'mobility.air_step_i'):
					records.append('钟骨：归航记录\n三处坐标不是孤立事件。网络中仍存在未识别节点。')
					records.append('钟骨：尾声记录\n原来门一直不止三扇。')
			if records.is_empty():
				continue
			var world_card = _mission_card(stack,'archive_records_'+world,colors)
			_mission_label(world_card,WORLDS[world],colors,'system',1.12)
			for record in records:
				_mission_label(world_card,record,colors)
			shown = true
		if not shown:
			_mission_label(stack,'尚无已确认档案',colors)
	_exploration_resume()
	_button('返回','back')
func _save() -> void:
	_text('保存由权威 SaveV6 / 槽位事务完成；客户端不读取或写入存档。')
	_save_availability_text()
	_button('保存当前默认存档','save_default',null,connected)
	_button('保存并退出','exit_confirm',null,connected)
	_button('返回归航站并保存','return',null,connected)
	for slot in slots:
		if slot.valid and not slot.readOnly:
			var target_name = _text('目标记录：'+slot.displayName)
			target_name.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
			var reference = _text('记录标识：'+slot.slotId)
			reference.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
			_button('覆盖：'+slot.displayName,'named_save_prepare',{'operation':'save_slot','row':slot.duplicate(true)},connected)
			_button('返回并保存：'+slot.displayName,'named_save_prepare',{'operation':'return_slot','row':slot.duplicate(true)},connected)
	var entry: LineEdit
	if is_instance_valid(retained_save_entry) and not retained_save_entry.is_queued_for_deletion() and is_instance_valid(box) and retained_save_entry.get_parent()==box:
		entry = retained_save_entry
		box.move_child(entry,box.get_child_count()-1)
	else:
		entry = LineEdit.new()
		entry.placeholder_text = '新存档名称'
		entry.tooltip_text = '存档名称最多 48 个字符；不能全部为空白。'
		entry.max_length = 48
		entry.add_theme_font_size_override('font_size',int(18*font_scale))
		box.add_child(entry)
		ThemeModel.install_save_name_skin(entry)
	save_entry = entry
	var captured = page_owner.duplicate(true)
	var create = _button('创建新槽位','slot_create_prepare')
	create.pressed.connect(func():
		if page_owner==captured and not entry.text.strip_edges().is_empty():
			intent.emit('save_slot_new',entry.text.strip_edges(),captured))
	_button('刷新列表','slots_refresh',null,connected)
	_button('返回','back')
func _named_save_confirmation() -> void:
	var choice = page_owner.get('namedSave')
	if not choice is Dictionary or not choice.get('row') is Dictionary or not choice.get('operation') in ['save_slot','return_slot'] or typeof(choice.row.get('displayName'))!=TYPE_STRING:
		_text('记录状态已变化，请返回重新选择。')
		_button('取消','back')
		return
	var returning = choice.operation=='return_slot'
	_text('返回归航站并覆盖记录？' if returning else '覆盖这条记录？')
	var target_name = _text('目标记录：'+choice.row.displayName)
	target_name.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
	var reference = _text('记录标识：'+choice.row.slotId)
	reference.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
	_text('将按当前返回并保存操作返回归航站，并覆盖指定记录。原记录内容将被本次保存替换。' if returning else '原记录内容将被本次保存替换。')
	var cancel = _button('取消','back')
	_button('确认返回并覆盖' if returning else '确认覆盖','named_save_confirm',choice.duplicate(true),connected)
	cancel.grab_focus()
func _slots() -> void:
	_text('只继续服务报告的有效记录；只读记录也可能可以继续。')
	_save_availability_text()
	_button('继续默认存档','continue_default',null,connected and default_save_state=='available')
	for slot in slots:
		var world_for_color: String = snapshot.worldId if typeof(snapshot.get('worldId'))==TYPE_STRING else 'return_station'
		var colors = ThemeModel.palette(world_for_color)
		var card = _mission_card(box,'continue_slot',colors)
		var name_label = _mission_label(card,slot.displayName,colors,'system')
		name_label.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
		var id_label = _mission_label(card,'记录编号：'+slot.slotId,colors)
		id_label.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
		var state = '此记录暂不可继续。'
		if slot.valid:
			state = ('可尝试备用记录' if slot.get('recoverable',false) else '可继续') if connected and not pending else '有效记录 · 等待连接或请求完成'
			if slot.readOnly: state += ' · 只读记录'
		elif slot.get('recoverable',false):
			state = '记录状态待确认'
		_mission_label(card,state,colors)
		if slot.valid and typeof(slot.get('worldId'))==TYPE_STRING and Content.WORLD_NAMES.has(slot.worldId):
			_mission_label(card,'记录世界：'+Content.WORLD_NAMES[slot.worldId],colors)
		var continue_button = _button('继续此记录','continue_slot',slot.duplicate(true),connected and slot.valid)
		slot_continue_buttons[slot.slotId] = continue_button
		box.remove_child(continue_button)
		card.add_child(continue_button)
	_button('刷新列表','slots_refresh',null,connected)
	_button('返回','back')

func focused_continue_slot() -> String:
	if not visible or mode!='slots': return ''
	for id in slot_continue_buttons:
		var button = slot_continue_buttons[id]
		if is_instance_valid(button) and button.has_focus(): return str(id)
	return ''

func focus_continue_slot(id: String) -> void:
	var button = slot_continue_buttons.get(id)
	if not visible or mode!='slots': return
	if is_instance_valid(button) and not button.disabled: button.grab_focus()
	else: grab_focus()

func _clear_enhancement_preview() -> void:
	enhancement_preview = ''
	enhancement_preview_owner.clear()
func _enhancement_state() -> String:
	if mode!='enhancement' or not connected or pending or not P.snapshot(snapshot) or snapshot.worldId!='return_station' or snapshot.sceneId!='rs_core_room' or snapshot.get('entryToken')!=null:
		return 'unknown'
	var context = page_owner.get('context')
	if not P.context(context) or not P.same(context,snapshot):
		return 'unknown'
	var progress = Content.progress(snapshot,'grey_hive')
	if progress.is_empty() or not progress.completed or not snapshot.capabilities.has('firstEnhancementChoice'):
		return 'unknown'
	var choice = snapshot.capabilities.firstEnhancementChoice
	if choice==null:
		return 'available'
	if typeof(choice)==TYPE_STRING and choice in ['information.local_map_i','perception.rear_view_i','body.regeneration_i'] and P.granted(snapshot,choice):
		return choice
	return 'unknown'
func _enhancement_ready(owner: Dictionary) -> bool:
	return visible and page_owner==owner and _enhancement_state()=='available'
func _enhancement_local_button(text: String, choice: String) -> void:
	var button = Button.new()
	button.text = text
	button.tooltip_text = text
	button.clip_text = true
	button.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	button.disabled = not _enhancement_ready(page_owner)
	button.custom_minimum_size.y = 38*font_scale
	button.add_theme_font_size_override('font_size',int(18*font_scale))
	var captured = page_owner.duplicate(true)
	button.pressed.connect(func():
		if not _enhancement_ready(captured):
			return
		if choice.is_empty():
			_clear_enhancement_preview()
		elif choice in ['information.local_map_i','perception.rear_view_i','body.regeneration_i']:
			enhancement_preview = choice
			enhancement_preview_owner = captured.duplicate(true)
		else:
			return
		_rebuild()
		_layout_content())
	box.add_child(button)
func _enhancement() -> void:
	var use_text = {
		'information.local_map_i':'M / 地图：需先获得并选用本能力；只显示权威允许的已探索信息。',
		'perception.rear_view_i':'后方视野：显示来自当前权威授权的信息。',
		'body.regeneration_i':'再生：由权威自动处理，受伤与战斗后的恢复不由本页触发。'}
	_text('灰巢首次撤离后的真实能力终端选择。一次确认，无法从此界面授予其他能力。')
	var state = _enhancement_state()
	if state=='unknown':
		_text('首次进化状态未确认')
		_button('确认选择','enhancement_choose',null,false)
	elif state!='available':
		_text('首次进化已记录：'+CAPS[state])
		if use_text.has(state):
			_text(use_text[state])
	elif enhancement_preview_owner==page_owner and enhancement_preview in ['information.local_map_i','perception.rear_view_i','body.regeneration_i']:
		_text('选择：'+CAPS[enhancement_preview])
		_text(use_text[enhancement_preview])
		_text('首次进化只能确认一次。确认后将记录在当前旅程。')
		_button('确认选择','enhancement_choose',enhancement_preview,true)
		_enhancement_local_button('返回选择','')
	else:
		for id in ['information.local_map_i','perception.rear_view_i','body.regeneration_i']:
			_enhancement_local_button(CAPS[id],id)
			_text(use_text[id])
	_button('返回','back')
