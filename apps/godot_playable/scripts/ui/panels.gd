extends Control
const Content = preload('res://scripts/ui/content_model.gd')
const P = preload('res://scripts/bridge/protocol.gd')
const MapView = preload('res://scripts/ui/map_panel.gd')
const CAPS = {'information.local_map_i':'局部地图','information.enemy_vitals_basic':'敌人生命信息','perception.rear_view_i':'后方视野','perception.acoustic_mapping_i':'声学映射','body.regeneration_i':'再生能力','mobility.air_step_i':'空中踏步'}
const WORLDS = {'return_station':'归航站','grey_hive':'灰巢设施','mist_harbor':'雾港余烬','clockworks':'钟骨工厂'}
signal intent(kind, value, page_owner)
var mode = ''
var page_owner: Dictionary = {}
var enhancement_preview = ''
var enhancement_preview_owner: Dictionary = {}
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
var scroll: ScrollContainer
var layout_busy = false
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
	scroll = ScrollContainer.new()
	# Ignore child horizontal minimums so an earlier wider page cannot block shrink.
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_NEVER
	panel.add_child(scroll)
	box = VBoxContainer.new()
	box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	box.custom_minimum_size.x = 0
	scroll.add_child(box)
	resized.connect(_layout_content)
	scroll.resized.connect(_layout_content)
	visible = false
	call_deferred('_layout_content')
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
	layout_busy = false
func show_page(kind: String, association: Dictionary, s: Dictionary, is_connected: bool, blocked: bool = false) -> void:
	_clear_enhancement_preview()
	mode = kind
	page_owner = association.duplicate(true)
	snapshot = s
	connected = is_connected
	pending = blocked
	visible = true
	if kind=='settings':
		staged = preferences.duplicate(true)
	_rebuild()
	_layout_content()
func close_page() -> void:
	_clear_enhancement_preview()
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
	button.tooltip_text = text
	button.clip_text = true
	button.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	button.disabled = not enabled or pending
	button.custom_minimum_size.y = 38*font_scale
	button.add_theme_font_size_override('font_size',int(18*font_scale))
	var captured = page_owner.duplicate(true)
	button.pressed.connect(func():
		if action=='enhancement_choose' and (not _enhancement_ready(captured) or enhancement_preview_owner!=captured or value!=enhancement_preview):
			return
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
func _exploration_resume() -> void:
	if P.snapshot(snapshot):
		_button('继续探索','resume',null,connected and snapshot.player.currentHp>0)
func _inventory() -> void:
	if snapshot.is_empty() or not P.build(snapshot.get('build')):
		_text('尚无有效权威背包投影；不创建物品或预设。')
	else:
		_text('物品 / 装备 / 描述 · 修订 '+str(snapshot.build.revision))
		for item in snapshot.build.items:
			_text(item.label+' ×'+str(item.quantity)+'\n'+item.description+'\n槽位：'+item.slotId+(' · 已装备' if item.equipped else ''))
			_button('卸下' if item.equipped else '装备','build_item',item.itemId,connected)
	_exploration_resume()
	_button('返回','back')
func _capability() -> void:
	if not P.snapshot(snapshot):
		_text('能力状态尚未同步')
	else:
		var use_text = {
			'information.local_map_i':'M / 地图：只显示权威允许的已探索信息。',
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
			_text(CAPS[c.capabilityId]+' · '+('已选中' if c.selected else '已获得' if c.granted else '未获得'))
			_text(use_text[c.capabilityId])
			if c.granted:
				_button('取消选择' if c.selected else '选择','capability_select',c.capabilityId,connected)
				if c.capabilityId=='perception.rear_view_i':
					_button('确认后方视野','capability_activate',c.capabilityId,connected and c.get('cooldownRemainingMs',0)==0)
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
	var world: String = snapshot.worldId
	var scene: String = snapshot.sceneId
	if not Content.WORLD_NAMES.has(world) or not Content.SCENE_NAMES.has(scene):
		return neutral
	if world=='return_station':
		if scene!='rs_core_room':
			return neutral
	elif not scene.begins_with({'grey_hive':'gh_','mist_harbor':'mh_','clockworks':'cw_'}[world]):
		return neutral
	var rows = {}
	for id in ['grey_hive','mist_harbor','clockworks'] if world=='return_station' else [world]:
		var row: Dictionary = Content.progress(snapshot,id)
		if row.is_empty():
			return neutral
		var events: Array = row.completedEvents
		var canonical: Array = Content.EVENTS[id]
		if id=='mist_harbor':
			if canonical[2] in events and (not canonical[0] in events or not canonical[1] in events):
				return neutral
		elif (canonical[1] in events and not canonical[0] in events) or (canonical[2] in events and (not canonical[0] in events or not canonical[1] in events)):
			return neutral
		rows[id] = row
	if world=='return_station':
		if (rows.mist_harbor.completed and not rows.grey_hive.completed) or (rows.clockworks.completed and (not rows.grey_hive.completed or not rows.mist_harbor.completed)):
			return neutral
		if not rows.grey_hive.completed:
			return ['任务参考：恢复场景后，靠近灰巢入口查看当前交互。']
		if not rows.mist_harbor.completed:
			return ['任务参考：恢复场景后，靠近雾港入口查看当前交互。']
		if not rows.clockworks.completed:
			return ['任务参考：恢复场景后，靠近钟骨入口查看当前交互。']
		return ['三处世界进度均已记录为完成。可查看当前保存与复访选项。']
	if snapshot.progression.get('currentWorldId')!=world:
		return neutral
	var current: Dictionary = rows[world]
	if current.revisitCount>0 or current.visitId>1:
		return ['复访任务参考：以当前事件、门和交互提示为准；首通奖励不重复。']
	if current.visitId!=1 or current.revisitCount!=0:
		return neutral
	var events: Array = current.completedEvents
	var canonical: Array = Content.EVENTS[world]
	if world=='grey_hive':
		if not canonical[0] in events:
			return ['任务参考：沿当前合法通路前往主电室，查看恢复主电的交互。']
		if not canonical[1] in events:
			return ['任务参考：沿当前合法通路前往隔离封锁，查看隔离协议交互。']
		if canonical[2] in events:
			return ['任务参考：灰巢撤离事件已记录；查看灰巢撤离区的当前归航交互。']
		var beacon = snapshot.get('greyHiveBeacon')
		if not beacon is Dictionary or beacon.size()!=2 or not beacon.has_all(['state','legacyCompletedWithoutReceipt']) or typeof(beacon.state)!=TYPE_STRING or not beacon.state in ['uncollected','carried','mounted'] or typeof(beacon.legacyCompletedWithoutReceipt)!=TYPE_BOOL or beacon.legacyCompletedWithoutReceipt:
			return ['任务参考：信标收取状态未确认；查看当前信标交互提示。']
		if beacon.state=='uncollected':
			return ['任务参考：前往信标室查看收取信标的交互；部署为可选。']
		return ['任务参考：前往灰巢撤离区查看撤离控制；具体条件以当前交互为准。']
	if world=='mist_harbor':
		if not canonical[0] in events or not canonical[1] in events:
			var lines = ['任务参考：两处航标可以按任意顺序确认。']
			if not canonical[0] in events:
				lines.append('潮汐仓库：查看西侧航标交互。')
			if not canonical[1] in events:
				lines.append('防波堤：查看东侧航标交互。')
			return lines
		if not canonical[2] in events:
			return ['任务参考：沿当前合法通路前往共鸣塔，查看信号终端。']
		return ['任务参考：处理守望者区域的战斗后，前往雾港撤离区查看当前归航交互。']
	if not canonical[0] in events:
		return ['任务参考：前往压力大厅查看三处压力阀门；具体条件以当前交互为准。']
	if not canonical[1] in events:
		return ['任务参考：沿当前合法通路前往调节者核心，处理当前战斗与终端。']
	if not canonical[2] in events:
		return ['任务参考：前往停机出口查看停机终端。']
	return ['任务参考：停机事件已记录；查看停机出口的当前归航交互。']
func _mission() -> void:
	if snapshot.is_empty():
		_text('尚未收到当前旅程')
	elif not P.snapshot(snapshot):
		_text('当前旅程状态未确认')
	else:
		_text('当前世界：'+WORLDS.get(snapshot.worldId,'世界名称未确认'))
		_text('当前场景：'+Content.SCENE_NAMES.get(snapshot.sceneId,'场景名称未确认'))
		for line in _mission_route_guidance():
			_text(line)
		for o in snapshot.objectives:
			_text(Content.objective(o))
		for world in ['grey_hive','mist_harbor','clockworks']:
			var row = Content.progress(snapshot,world)
			if row.is_empty():
				_text(WORLDS[world]+' · 进度未确认')
			else:
				_text(WORLDS[world]+' · '+('已完成' if row.completed else '尚未完成')+' · 复访 '+str(int(row.revisitCount)))
				_text('事件记录 %d / 3'%row.completedEvents.size())
				for event in Content.EVENTS[world]:
					var recorded = event in row.completedEvents
					_text(('✓ ' if recorded else '○ ')+Content.EVENT_LABELS[event]+(' · 已记录' if recorded else ' · 尚未记录'))
		if snapshot.worldId=='return_station' and snapshot.sceneId=='rs_core_room':
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
		_text('目标交互需在场景恢复后进行。')
		if snapshot.worldId=='return_station' and snapshot.sceneId=='rs_core_room':
			_text('返回场景、靠近世界门，再查看当前交互提示。')
		_button('继续探索','resume',null,connected and P.snapshot(snapshot) and snapshot.player.currentHp>0)
	_button('档案','archive')
	_button('能力','capability')
	_button('系统','settings')
	_button('返回','back')
func _archive() -> void:
	if snapshot.is_empty() or not P.snapshot(snapshot):
		_text('档案状态未确认')
	else:
		var shown = false
		for world in ['grey_hive','mist_harbor','clockworks']:
			var row = Content.progress(snapshot,world)
			if row.is_empty():
				continue
			if row.get('worldId')=='grey_hive':
				if 'hive_power' in row.completedEvents:
					_text('灰巢：主电恢复记录\n主电还活着，只是被人为切断。')
					shown = true
				if 'hive_lockdown' in row.completedEvents:
					_text('灰巢：隔离协议记录\n隔离协议已被局部覆盖。')
					shown = true
			if row.get('worldId')=='mist_harbor' and 'mist_beacon_west' in row.completedEvents and 'mist_beacon_east' in row.completedEvents:
				_text('雾港：双基准同步记录\n双基准建立，中心干扰源可定位。')
				shown = true
			if world=='mist_harbor' and 'mist_signal' in row.completedEvents and P.granted(snapshot,'perception.acoustic_mapping_i'):
				_text('雾港：声学映射记录\n这次我不是在看路——是在听。')
				shown = true
			var complete = row.completed==true and row.firstCompletion==true and row.completedEvents.size()==3
			for event in Content.EVENTS[world]:
				complete = complete and event in row.completedEvents
			if complete:
				if world=='grey_hive':
					_text('灰巢：归航记录\n灰巢信标已解析。检测到第二组低可信坐标：雾港余烬。')
					shown = true
				elif world=='mist_harbor' and P.granted(snapshot,'perception.acoustic_mapping_i'):
					_text('雾港：归航记录\n声学映射已固化。第三坐标的机械周期与信号高度同步。')
					shown = true
				elif world=='clockworks' and P.granted(snapshot,'mobility.air_step_i'):
					_text('钟骨：归航记录\n三处坐标不是孤立事件。网络中仍存在未识别节点。')
					shown = true
		if not shown:
			_text('尚无已确认档案')
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
		'information.local_map_i':'M / 地图：只显示权威允许的已探索信息。',
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
