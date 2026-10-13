extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const WORLD_NAMES = {'return_station':'归航站','grey_hive':'灰巢设施','mist_harbor':'雾港余烬','clockworks':'钟骨工厂'}
const SCENE_NAMES = {'rs_core_room':'归航站核心室','gh_entry_maintenance':'维护入口','gh_power_room':'主电室','gh_gate_a':'Gate A','gh_central_shaft':'中央竖井','gh_lockdown':'隔离封锁','gh_bio_isolation':'生物隔离室','gh_gate_b':'Gate B','gh_deep_decon':'深层消杀区','gh_sentinel_arena':'哨卫区域','gh_beacon':'信标室','gh_exit':'灰巢撤离区','mh_fog_pier':'雾中码头','mh_tidal_warehouse':'潮汐仓库','mh_signal_yard':'信号场','mh_drowned_quay':'沉没岸台','mh_breakwater':'防波堤','mh_pump_station':'泵站','mh_resonance_tower':'共鸣塔','mh_warden_arena':'守望者区域','mh_extraction':'雾港撤离区','cw_entry_foundry':'铸造入口','cw_pressure_hall':'压力大厅','cw_conveyor_bridge':'输送桥','cw_boiler_chamber':'锅炉室','cw_gear_shaft':'齿轮竖井','cw_furnace_heart':'炉心','cw_forged_guard_arena':'铸造卫士区域','cw_regulator_core':'调节者核心','cw_shutdown_exit':'停机出口'}
const EVENTS = {'grey_hive':['hive_power','hive_lockdown','hive_extraction'],'mist_harbor':['mist_beacon_west','mist_beacon_east','mist_signal'],'clockworks':['clockworks_valves','clockworks_core','clockworks_shutdown']}
const EVENT_LABELS = {'hive_power':'主电恢复','hive_lockdown':'隔离封锁解除','hive_extraction':'灰巢撤离','mist_beacon_west':'西侧航标','mist_beacon_east':'东侧航标','mist_signal':'信号定位','clockworks_valves':'压力阀门','clockworks_core':'调节者核心','clockworks_shutdown':'停机结算'}
static func progress(s: Dictionary, world: String) -> Dictionary:
	var p = s.get('progression')
	if not p is Dictionary or not P.counter(p.get('schemaVersion'),1,1) or not P.counter(p.get('eventSeq')) or not P.rows(p.get('worlds'),16):
		return {}
	var matches = p.worlds.filter(func(row): return row is Dictionary and row.get('worldId')==world)
	if matches.size()!=1:
		return {}
	var row: Dictionary = matches[0]
	if typeof(row.get('completed'))!=TYPE_BOOL or typeof(row.get('firstCompletion'))!=TYPE_BOOL or not P.rows(row.get('completedEvents'),64) or not P.counter(row.get('revisitCount')) or not P.counter(row.get('visitId'),1) or not P.counter(row.get('cycleId'),1):
		return {}
	var seen = {}
	for event in row.completedEvents:
		if typeof(event)!=TYPE_STRING or seen.has(event) or not event in EVENTS.get(world,[]):
			return {}
		seen[event] = true
	return row
static func objective(o: Dictionary) -> String:
	var name = {'gh_restore_main_power':'恢复主电','gh_lockdown_objective':'解除隔离封锁'}.get(o.objectiveId,'当前目标')
	return name+' · '+{'active':'待完成','complete':'已完成','locked':'尚未解锁'}.get(o.state,'状态未确认')
static func narrative(before: Dictionary, target: Dictionary, receipt: Dictionary) -> String:
	if not P.success(receipt) or not receipt.applied or receipt.alreadyApplied or before.is_empty():
		return ''
	var after: Dictionary = receipt.snapshot
	var id: String = target.entityId
	var kind: String = target.kind
	if P.same(before,after) and after.serverTick>=before.serverTick and after.authorityRevision>before.authorityRevision:
		if kind=='scene_checkpoint' and typeof(after.get('checkpointId'))==TYPE_STRING and after.checkpointId==id:
			var checkpoints = {
				'mist_harbor:mh_fog_pier:mh_fog_pier_checkpoint':'',
				'mist_harbor:mh_tidal_warehouse:mh_warehouse_checkpoint':'',
				'mist_harbor:mh_signal_yard:mh_signal_yard_checkpoint':'',
				'mist_harbor:mh_drowned_quay:mh_drowned_quay_checkpoint':'',
				'mist_harbor:mh_breakwater:mh_breakwater_checkpoint':'',
				'mist_harbor:mh_pump_station:mh_pump_station_checkpoint':'',
				'mist_harbor:mh_resonance_tower:mh_resonance_tower_checkpoint':'',
				'mist_harbor:mh_warden_arena:mh_warden_arena_checkpoint':'',
				'mist_harbor:mh_extraction:mh_extraction_checkpoint':'',
				'clockworks:cw_entry_foundry:cw_entry_foundry_checkpoint':'',
				'clockworks:cw_pressure_hall:cw_pressure_hall_checkpoint':'',
				'clockworks:cw_conveyor_bridge:cw_conveyor_bridge_checkpoint':'',
				'clockworks:cw_boiler_chamber:cw_boiler_chamber_checkpoint':'',
				'clockworks:cw_gear_shaft:cw_gear_shaft_checkpoint':'',
				'clockworks:cw_gear_shaft:cw_gear_upper_dock_checkpoint':' · 上层停靠点',
				'clockworks:cw_furnace_heart:cw_furnace_heart_checkpoint':'',
				'clockworks:cw_forged_guard_arena:cw_forged_guard_arena_checkpoint':'',
				'clockworks:cw_regulator_core:cw_regulator_core_checkpoint':'',
				'clockworks:cw_shutdown_exit:cw_shutdown_exit_checkpoint':''
			}
			var key = before.worldId+':'+before.sceneId+':'+id
			if checkpoints.has(key) and SCENE_NAMES.has(before.sceneId):
				return '检查点已确认：'+SCENE_NAMES[before.sceneId]+checkpoints[key]+'。需要保留旅程时，请使用保存。'
		if P.snapshot(before) and P.snapshot(after) and typeof(target.get('entityId'))==TYPE_STRING and typeof(target.get('kind'))==TYPE_STRING and kind=='scene_checkpoint' and typeof(after.get('checkpointId'))==TYPE_STRING and after.checkpointId==id and SCENE_NAMES.has(before.sceneId):
			var station_hive_checkpoints = {
				'return_station:rs_core_room:rs_core_checkpoint':true,
				'grey_hive:gh_entry_maintenance:gh_entry_checkpoint':true,
				'grey_hive:gh_power_room:gh_power_checkpoint':true,
				'grey_hive:gh_gate_a:gh_gate_a_checkpoint':true,
				'grey_hive:gh_central_shaft:gh_shaft_checkpoint':true,
				'grey_hive:gh_lockdown:gh_lockdown_checkpoint':true,
				'grey_hive:gh_deep_decon:gh_decon_safety_alcove_checkpoint':true,
				'grey_hive:gh_beacon:gh_beacon_checkpoint':true,
				'grey_hive:gh_exit:gh_exit_checkpoint':true}
			var checkpoint_key = before.worldId+':'+before.sceneId+':'+id
			if station_hive_checkpoints.has(checkpoint_key):
				var checkpoint_rows = before.interactables.filter(func(row): return row.entityId==id and row.kind==kind)
				if checkpoint_rows.size()==1 and typeof(checkpoint_rows[0].get('active'))==TYPE_BOOL and checkpoint_rows[0].active and P.transform(checkpoint_rows[0].get('transform')):
					var checkpoint_text = '检查点已确认：'+SCENE_NAMES[before.sceneId]+'。需要保留旅程时，请使用保存。'
					return ('B-17 链路仅保持最低功率。 ' if checkpoint_key=='grey_hive:gh_entry_maintenance:gh_entry_checkpoint' else '')+checkpoint_text
		if before.worldId=='grey_hive':
			var lines = {'gh_beacon:gh_beacon_deploy_marker:beacon_collect':'这就是信号源。带回去，也许能解释这里为什么还在呼叫。','gh_entry_maintenance:gh_entry_checkpoint:scene_checkpoint':'B-17 链路仅保持最低功率。','gh_power_room:gh_power_console:power_console':'主电还活着，只是被人为切断。','gh_lockdown:gh_lockdown_terminal:lockdown_terminal':'隔离协议已被局部覆盖。','gh_sentinel_arena:gh_sys_sentinel_01:facility_log':'自动防卫单元已接管本区。','gh_entry_maintenance:gh_entry_tutorial_terminal:terminal':'维护终端：WASD / 方向键移动，鼠标指向瞄准；J / 左键攻击，Shift 冲刺。Q 脉冲，按住 E 防御，R 穿刺；F 交互，Space 情境穿越。T 任务，M 地图，Esc 暂停；动作与地图信息以当前状态和权限为准。'}
			return lines.get(before.sceneId+':'+id+':'+kind,'')
		if before.worldId=='mist_harbor':
			var prior = progress(before,'mist_harbor')
			var current = progress(after,'mist_harbor')
			if prior.is_empty() or current.is_empty() or after.progression.eventSeq<=before.progression.eventSeq:
				return ''
			var selected = 'mist_beacon_west' if id=='mh_west_beacon' and before.sceneId=='mh_tidal_warehouse' else 'mist_beacon_east' if id=='mh_east_beacon' and before.sceneId=='mh_breakwater' else ''
			if kind=='beacon' and not selected.is_empty() and not selected in prior.completedEvents and 'mist_beacon_west' in current.completedEvents and 'mist_beacon_east' in current.completedEvents and prior.completedEvents.size()+1==current.completedEvents.size():
				return '双基准建立，中心干扰源可定位。'
			if kind=='terminal' and id=='mh_signal_console_staged' and before.sceneId=='mh_resonance_tower' and not 'mist_signal' in prior.completedEvents and 'mist_signal' in current.completedEvents and not P.granted(before,'perception.acoustic_mapping_i') and P.granted(after,'perception.acoustic_mapping_i',true):
				return '这次我不是在看路——是在听。'
		if before.get('worldId')=='clockworks' and P.snapshot(before) and P.snapshot(after):
			if typeof(target.get('entityId'))!=TYPE_STRING or typeof(target.get('kind'))!=TYPE_STRING:
				return ''
			var valves = {'cw_pressure_valve_01_staged':'1','cw_pressure_valve_02_staged':'2','cw_pressure_valve_03_staged':'3'}
			var selected = ''
			var acknowledgement = ''
			if before.sceneId=='cw_pressure_hall' and kind=='valve_control' and valves.has(id):
				selected = 'clockworks_valves'
				acknowledgement = '阀门 '+valves[id]+'：本次操作已确认。'
			elif before.sceneId=='cw_regulator_core' and kind=='terminal' and id=='cw_regulator_core_console_staged':
				selected = 'clockworks_core'
			else:
				return ''
			var matches = before.interactables.filter(func(row): return row.entityId==id and row.kind==kind)
			if matches.size()!=1 or typeof(matches[0].get('active'))!=TYPE_BOOL or not matches[0].active or not P.transform(matches[0].get('transform')):
				return ''
			var prior = progress(before,'clockworks')
			var current = progress(after,'clockworks')
			var new_event = false
			if not prior.is_empty() and not current.is_empty() and prior.worldId==current.worldId and prior.visitId==current.visitId and prior.cycleId==current.cycleId and prior.revisitCount==current.revisitCount and not prior.completed and not current.completed and not prior.firstCompletion and not current.firstCompletion and before.progression.currentWorldId=='clockworks' and after.progression.currentWorldId=='clockworks' and after.progression.eventSeq==before.progression.eventSeq+1 and current.completedEvents.size()==prior.completedEvents.size()+1 and not selected in prior.completedEvents and selected in current.completedEvents:
				var retained = true
				for event in prior.completedEvents:
					if not event in current.completedEvents:
						retained = false
				new_event = retained
			if new_event:
				var recorded = '压力阀门事件已记录。' if selected=='clockworks_valves' else '调节者核心事件已记录。'
				return acknowledgement+(' ' if not acknowledgement.is_empty() else '')+recorded
			return acknowledgement
	if kind=='world_gate' and after.worldId=='return_station' and after.sceneId=='rs_core_room' and after.worldEpoch==before.worldEpoch+1 and after.authorityRevision>before.authorityRevision:
		var row = progress(before,before.worldId)
		var next = progress(after,before.worldId)
		if row.is_empty() or next.is_empty() or row.visitId!=1 or row.revisitCount!=0 or next.visitId!=1 or next.revisitCount!=0:
			return ''
		if before.worldId=='grey_hive' and before.sceneId=='gh_exit' and id=='gh_extraction_return_to_rs' and row.completed and row.firstCompletion and next.completed and next.firstCompletion and row.completedEvents.size()==3 and next.completedEvents==row.completedEvents and after.progression.eventSeq==before.progression.eventSeq:
			return '灰巢信标已解析。检测到第二组低可信坐标：雾港余烬。'
		if before.worldId=='mist_harbor' and before.sceneId=='mh_extraction' and id=='mh_extraction_return_to_rs' and not row.completed and next.completed and next.firstCompletion and row.completedEvents.size()==3 and next.completedEvents==row.completedEvents and after.progression.eventSeq==before.progression.eventSeq+1:
			return '声学映射已固化。第三坐标的机械周期与信号高度同步。'
		if before.worldId=='clockworks' and before.sceneId=='cw_shutdown_exit' and id=='cw_shutdown_return_to_rs' and row.completed and row.firstCompletion and next.completed and next.firstCompletion and row.completedEvents.size()==3 and next.completedEvents==row.completedEvents and after.progression.eventSeq==before.progression.eventSeq:
			return '三处坐标不是孤立事件。网络中仍存在未识别节点。'
	return ''
