extends Control
const Content = preload('res://scripts/ui/content_model.gd')
const P = preload('res://scripts/bridge/protocol.gd')
const MapView = preload('res://scripts/ui/map_panel.gd')
const CAPS = {'information.local_map_i':'局部地图','information.enemy_vitals_basic':'敌人生命信息','perception.rear_view_i':'后方视野','perception.acoustic_mapping_i':'声学映射','body.regeneration_i':'再生能力','mobility.air_step_i':'空中踏步'}
const WORLDS = {'return_station':'归航站','grey_hive':'灰巢设施','mist_harbor':'雾港余烬','clockworks':'钟骨工厂'}
signal intent(kind, value, page_owner)
var mode = ''
var page_owner: Dictionary = {}
var box: VBoxContainer
var title: Label
var status: Label
var snapshot: Dictionary = {}
var slots: Array = []
var default_save_state = 'unknown'
var slot_list_state = 'unknown'
var connected = false
var pending = false
var preferences = {'scale':1.0,'fullscreen':false,'color':false,'reduced':false,'muted':false,'volume':0.35}
var staged = {}
var font_scale = 1.0
var panel: PanelContainer
func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_STOP
	var shade = ColorRect.new()
	shade.color = Color(0.02,0.04,0.06,0.82)
	shade.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(shade)
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
	var scroll = ScrollContainer.new()
	panel.add_child(scroll)
	box = VBoxContainer.new()
	box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	box.custom_minimum_size.x = 520
	scroll.add_child(box)
	visible = false
func show_page(kind: String, association: Dictionary, s: Dictionary, is_connected: bool, blocked: bool = false) -> void:
	mode = kind
	page_owner = association.duplicate(true)
	snapshot = s
	connected = is_connected
	pending = blocked
	visible = true
	if kind=='settings':
		staged = preferences.duplicate(true)
	_rebuild()
func close_page() -> void:
	visible = false
	mode = ''
	page_owner.clear()
func set_save_availability(default_state: String, list_state: String, rows: Array) -> void:
	default_save_state = default_state
	slot_list_state = list_state
	slots = rows.duplicate(true) if list_state=='list' else []
func _save_availability_text() -> void:
	_text({'unknown':'默认存档状态未确认','pending':'正在确认默认存档','available':'默认存档可继续','unavailable':'尚无默认存档','invalid':'默认存档响应无效；请刷新','error':'默认存档读取失败；请刷新'}.get(default_save_state,'默认存档状态未确认'))
	_text({'unknown':'存档列表未确认','pending':'正在读取存档列表','empty':'当前没有命名存档','list':'当前存档列表已确认','invalid':'存档列表响应无效；请刷新','error':'存档列表读取失败；请刷新'}.get(slot_list_state,'存档列表未确认'))
func set_message(text: String, blocked: bool = false) -> void:
	pending = blocked
	if is_instance_valid(status):
		status.text = text
func _clear() -> void:
	for child in box.get_children():
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
	button.disabled = not enabled or pending
	button.custom_minimum_size.y = 38*font_scale
	button.add_theme_font_size_override('font_size',int(18*font_scale))
	var captured = page_owner.duplicate(true)
	button.pressed.connect(func():
		if page_owner==captured and visible and not pending:
			intent.emit(action,value,captured))
	box.add_child(button)
	return button
func _rebuild() -> void:
	_clear()
	title = _text({'start':'无限人族进化 · 归航','pause':'暂停','new_confirm':'开始新的旅程？','exit_confirm':'保存并退出？','settings':'系统设置','inventory':'背包与装备','character':'岑遥','capability':'能力','enhancement':'首次进化','mission':'任务与世界网络','archive':'档案','save':'存档','slots':'继续旅程','map':'地图','baizhi':'白芷','baizhi_confirm':'确认选择','dialogue':'信息','death':'旅程中断','error':'链路状态','waiting':'等待权威确认'}.get(mode,mode))
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
		'settings': _settings()
		'inventory': _inventory()
		'character':
			if not snapshot.is_empty():
				_text('岑遥 · 单一主角\n生命 %d / %d\n能量 %d / %d\n当前动作：%s\n身体状态：权威未提供'%[snapshot.player.currentHp,snapshot.player.maxHp,snapshot.player.currentEnergy,snapshot.player.maxEnergy,snapshot.player.actionState])
				if P.build(snapshot.get('build')):
					for e in snapshot.build.equipment:
						_text(e.slotId+' · '+e.label)
				for c in P.capability_items(snapshot):
					if c.granted and CAPS.has(c.capabilityId):
						_text(CAPS[c.capabilityId]+(' · 已选中' if c.selected else ' · 已获得'))
			_button('返回','back')
		'capability': _capability()
		'enhancement':
			_text('灰巢首次撤离后的真实能力终端选择。一次确认，无法从此界面授予其他能力。')
			for id in ['information.local_map_i','perception.rear_view_i','body.regeneration_i']:
				_button(CAPS[id],'enhancement_choose',id,connected)
			_button('返回','back')
		'mission': _mission()
		'archive': _archive()
		'save': _save()
		'slots': _slots()
		'map':
			if not snapshot.is_empty():
				var map = MapView.new()
				map.custom_minimum_size = Vector2(520,440)*font_scale
				map.text_scale = font_scale
				box.add_child(map)
				map.set_snapshot(snapshot)
			_button('返回','back')
		'baizhi':
			_text('隔离室里的白芷等待你的回应。对白不会授予 Gate B 的钥匙。')
			_button('我会帮你离开这里。','baizhi_confirm','taken')
			_button('我不能带你走。','baizhi_confirm','left')
			_button('我还需要确认一些事情。','baizhi_unresolved')
		'baizhi_confirm':
			_text('确认后由权威保存当前选择；未知响应不会重复提交。')
			_button('确认','baizhi_commit',page_owner.get('choice'))
			_button('返回对白','baizhi_page')
		'dialogue':
			_text(str(page_owner.get('text','')))
			_button('关闭','dialogue_close')
		'death':
			_text('当前角色已死亡；输入已停止。')
			_button('从存档继续','slots',null,connected)
			_button('新的旅程','new_confirm',null,connected)
		'error':
			_text('游戏输入已停止。请保留当前会话结果，勿盲目重复未知操作。')
			_button('返回状态','start')
			_button('退出客户端','close_client')
		'waiting': _text('正在等待当前请求的真实回执。')
func _settings() -> void:
	_text('仅当前会话；不会写入偏好文件。文字大小保持所选比例，小窗口可以滚动。临时原生合成音频；实际输出尚未验证。静音和音量仅当前会话。')
	var size_pick = OptionButton.new()
	for label in ['100%','125%','150%']:
		size_pick.add_item(label)
	size_pick.select([1.0,1.25,1.5].find(staged.scale))
	size_pick.item_selected.connect(func(index): staged.scale=[1.0,1.25,1.5][index])
	size_pick.add_theme_font_size_override('font_size',int(18*font_scale))
	box.add_child(size_pick)
	for pair in [['fullscreen','全屏'],['color','颜色辅助标记'],['reduced','减少动态效果'],['muted','静音']]:
		var check = CheckBox.new()
		check.text = pair[1]
		check.add_theme_font_size_override('font_size',int(18*font_scale))
		check.button_pressed = staged[pair[0]]
		var key: String = pair[0]
		check.toggled.connect(func(value): staged[key]=value)
		box.add_child(check)
	var captured = page_owner.duplicate(true)
	var volume_label = Label.new()
	volume_label.text = '主音量：'+str(int(round(staged.volume*100)))+'%'
	volume_label.add_theme_font_size_override('font_size',int(18*font_scale))
	box.add_child(volume_label)
	var slider = HSlider.new()
	slider.min_value = 0
	slider.max_value = 100
	slider.step = 1
	slider.value = staged.volume*100
	slider.custom_minimum_size.y = 36*font_scale
	box.add_child(slider)
	var settings_owner = page_owner.duplicate(true)
	slider.value_changed.connect(func(value):
		if page_owner==settings_owner:
			staged.volume=value/100.0
			volume_label.text='主音量：'+str(int(value))+'%')
	var defaults = _button('默认值','settings_defaults')
	defaults.pressed.connect(func():
		if page_owner==captured:
			staged={'scale':1.0,'fullscreen':false,'color':false,'reduced':false,'muted':false,'volume':0.35}
			_rebuild())
	var apply = _button('应用','settings_apply')
	apply.pressed.connect(func():
		if page_owner==captured:
			intent.emit('apply_preferences',staged.duplicate(true),captured))
	_button('返回（放弃未应用修改）','back')
func _inventory() -> void:
	if snapshot.is_empty() or not P.build(snapshot.get('build')):
		_text('尚无有效权威背包投影；不创建物品或预设。')
	else:
		_text('物品 / 装备 / 描述 · 修订 '+str(snapshot.build.revision))
		for item in snapshot.build.items:
			_text(item.label+' ×'+str(item.quantity)+'\n'+item.description+'\n槽位：'+item.slotId+(' · 已装备' if item.equipped else ''))
			_button('卸下' if item.equipped else '装备','build_item',item.itemId,connected)
	_button('返回','back')
func _capability() -> void:
	if snapshot.is_empty():
		_text('能力状态尚未同步')
	else:
		_text('从普通人到超越限制 · 只显示实际授予能力；树关系与成本未提供')
		for c in P.capability_items(snapshot):
			if not CAPS.has(c.capabilityId):
				continue
			_text(CAPS[c.capabilityId]+' · '+('已选中' if c.selected else '已获得' if c.granted else '未获得'))
			if c.granted:
				_button('取消选择' if c.selected else '选择','capability_select',c.capabilityId,connected)
				_button('激活','capability_activate',c.capabilityId,connected and c.get('cooldownRemainingMs',0)==0)
	_button('返回','back')
func _mission() -> void:
	if snapshot.is_empty():
		_text('尚未收到当前旅程')
	else:
		_text('当前世界：'+WORLDS.get(snapshot.worldId,snapshot.worldId))
		for o in snapshot.objectives:
			_text(Content.objective(o))
		for world in ['grey_hive','mist_harbor','clockworks']:
			var row = Content.progress(snapshot,world)
			if not row.is_empty():
				_text(WORLDS[world]+' · '+('已完成' if row.completed else '尚未完成')+' · 复访 '+str(int(row.revisitCount)))
				for event in row.completedEvents:
					_text('✓ '+Content.EVENT_LABELS[event])
		for pair in [['rs_world_gate_marker','灰巢入口'],['rs_mh_world_gate_marker','雾港入口'],['rs_cw_world_gate_marker','钟骨入口']]:
			for i in snapshot.interactables:
				if i.entityId==pair[0] and i.kind=='world_gate':
					_text(pair[1]+' · '+('可用' if i.active else '锁定 / 当前不可用'))
		var beacon = snapshot.get('greyHiveBeacon')
		if snapshot.worldId=='grey_hive' and beacon is Dictionary and beacon.get('state') in ['uncollected','carried','mounted'] and typeof(beacon.get('legacyCompletedWithoutReceipt'))==TYPE_BOOL and (not beacon.legacyCompletedWithoutReceipt or beacon.state=='uncollected'):
			_text('信标：'+{'uncollected':'尚未收取','carried':'已携带','mounted':'已部署'}[beacon.state])
		var pump = snapshot.get('mistHarborPump')
		if snapshot.worldId=='mist_harbor' and pump is Dictionary and pump.get('state') in ['ready','draining','drained']:
			_text('泵站：'+{'ready':'就绪','draining':'排水中','drained':'排水完成'}[pump.state])
	_button('档案','archive')
	_button('能力','capability')
	_button('系统','settings')
	_button('返回','back')
func _archive() -> void:
	if not snapshot.is_empty():
		for world in ['grey_hive','mist_harbor','clockworks']:
			var row = Content.progress(snapshot,world)
			if row.is_empty():
				continue
			if row.get('worldId')=='grey_hive':
				if 'hive_power' in row.completedEvents:
					_text('灰巢：主电恢复记录\n主电还活着，只是被人为切断。')
				if 'hive_lockdown' in row.completedEvents:
					_text('灰巢：隔离协议记录\n隔离协议已被局部覆盖。')
			if row.get('worldId')=='mist_harbor' and 'mist_beacon_west' in row.completedEvents and 'mist_beacon_east' in row.completedEvents:
				_text('雾港：双基准同步记录\n双基准建立，中心干扰源可定位。')
	_button('返回','back')
func _save() -> void:
	_text('保存由权威 SaveV6 / 槽位事务完成；客户端不读取或写入存档。')
	_save_availability_text()
	_button('保存当前默认存档','save_default',null,connected)
	_button('保存并退出','exit_confirm',null,connected)
	_button('返回归航站并保存','return',null,connected)
	for slot in slots:
		if slot.valid and not slot.readOnly:
			_button('覆盖：'+slot.displayName,'save_slot',{'slotId':slot.slotId,'displayName':slot.displayName,'create':false},connected)
			_button('返回并保存：'+slot.displayName,'return_slot',{'slotId':slot.slotId,'displayName':slot.displayName,'create':false},connected)
	var entry = LineEdit.new()
	entry.placeholder_text = '新存档名称'
	entry.max_length = 80
	entry.add_theme_font_size_override('font_size',int(18*font_scale))
	box.add_child(entry)
	var captured = page_owner.duplicate(true)
	var create = _button('创建新槽位','slot_create_prepare')
	create.pressed.connect(func():
		if page_owner==captured and not entry.text.strip_edges().is_empty():
			intent.emit('save_slot_new',entry.text.strip_edges(),captured))
	_button('刷新列表','slots_refresh',null,connected)
	_button('返回','back')
func _slots() -> void:
	_text('只继续服务报告的有效存档；缺失、只读和错误状态保留。')
	_save_availability_text()
	_button('继续默认存档','continue_default',null,connected and default_save_state=='available')
	for slot in slots:
		_text(slot.displayName+' · '+('有效' if slot.valid else '不可用')+(' · 只读' if slot.readOnly else '')+(' · 可恢复' if slot.get('recoverable',false) else ''))
		_button('继续：'+slot.displayName,'continue_slot',slot.slotId,connected and slot.valid)
	_button('刷新列表','slots_refresh',null,connected)
	_button('返回','back')
