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
	if typeof(row.get('completed'))!=TYPE_BOOL or typeof(row.get('firstCompletion'))!=TYPE_BOOL or not P.rows(row.get('completedEvents'),64) or not P.counter(row.get('revisitCount')) or not P.counter(row.get('visitId'),0) or not P.counter(row.get('cycleId'),1):
		return {}
	if row.visitId==0:
		var hive_choice = row.get('hiveChoice')
		var valid_unvisited = (
			P.snapshot(s)
			and s.worldId=='return_station'
			and s.sceneId=='rs_core_room'
			and s.get('entryToken')==null
			and EVENTS.has(world)
			and row.completed==false
			and row.firstCompletion==false
			and row.revisitCount==0
			and row.cycleId==1
			and row.completedEvents.is_empty()
		)
		if not valid_unvisited or (
			hive_choice!=null
			and (world!='grey_hive' or typeof(hive_choice)!=TYPE_STRING or hive_choice!='unresolved')
		):
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
		if P.snapshot(before) and P.snapshot(after) and before.get('entryToken')==null and after.get('entryToken')==null and before.worldId=='grey_hive' and before.sceneId=='gh_beacon' and before.progression.currentWorldId=='grey_hive' and after.progression.currentWorldId=='grey_hive' and typeof(target.get('entityId'))==TYPE_STRING and typeof(target.get('kind'))==TYPE_STRING and typeof(target.get('active'))==TYPE_BOOL and target.active and P.transform(target.get('transform')):
			var physical_beacon_before = before.get('greyHiveBeacon')
			var physical_beacon_after = after.get('greyHiveBeacon')
			if physical_beacon_before is Dictionary and physical_beacon_before.size()==2 and physical_beacon_before.has_all(['state','legacyCompletedWithoutReceipt']) and typeof(physical_beacon_before.state)==TYPE_STRING and physical_beacon_before.state in ['uncollected','carried','mounted'] and typeof(physical_beacon_before.legacyCompletedWithoutReceipt)==TYPE_BOOL and (not physical_beacon_before.legacyCompletedWithoutReceipt or physical_beacon_before.state=='uncollected') and physical_beacon_after is Dictionary and physical_beacon_after.size()==2 and physical_beacon_after.has_all(['state','legacyCompletedWithoutReceipt']) and typeof(physical_beacon_after.state)==TYPE_STRING and physical_beacon_after.state in ['uncollected','carried','mounted'] and typeof(physical_beacon_after.legacyCompletedWithoutReceipt)==TYPE_BOOL and not physical_beacon_after.legacyCompletedWithoutReceipt:
				var physical_beacon_text = ''
				if id=='gh_beacon_deploy_marker' and kind=='beacon_collect' and physical_beacon_before.state=='uncollected' and physical_beacon_after.state=='carried':
					physical_beacon_text = '这就是信号源。带回去，也许能解释这里为什么还在呼叫。 信标：已携带。'
				elif id=='gh_beacon_storage_mount_marker' and kind=='beacon_mount' and physical_beacon_before.state=='carried' and physical_beacon_after.state=='mounted':
					physical_beacon_text = '信标：已部署。'
				if not physical_beacon_text.is_empty():
					var physical_beacon_rows = before.interactables.filter(func(row): return row.entityId==id and row.kind==kind)
					var physical_beacon_prior = progress(before,'grey_hive')
					var physical_beacon_current = progress(after,'grey_hive')
					if physical_beacon_rows.size()==1 and typeof(physical_beacon_rows[0].get('active'))==TYPE_BOOL and physical_beacon_rows[0].active and P.transform(physical_beacon_rows[0].get('transform')) and physical_beacon_rows[0].transform==target.transform and not physical_beacon_prior.is_empty() and not physical_beacon_current.is_empty() and physical_beacon_prior.worldId=='grey_hive' and physical_beacon_current.worldId=='grey_hive' and physical_beacon_prior.visitId==physical_beacon_current.visitId and physical_beacon_prior.cycleId==physical_beacon_current.cycleId and physical_beacon_prior.revisitCount==physical_beacon_current.revisitCount and physical_beacon_prior.completed==physical_beacon_current.completed and physical_beacon_prior.firstCompletion==physical_beacon_current.firstCompletion and physical_beacon_prior.completedEvents==physical_beacon_current.completedEvents and before.progression.eventSeq==after.progression.eventSeq:
						return physical_beacon_text
		if before.worldId=='grey_hive':
			var lines = {'gh_beacon:gh_beacon_deploy_marker:beacon_collect':'这就是信号源。带回去，也许能解释这里为什么还在呼叫。','gh_entry_maintenance:gh_entry_checkpoint:scene_checkpoint':'B-17 链路仅保持最低功率。','gh_power_room:gh_power_console:power_console':'主电还活着，只是被人为切断。','gh_lockdown:gh_lockdown_terminal:lockdown_terminal':'隔离协议已被局部覆盖。','gh_sentinel_arena:gh_sys_sentinel_01:facility_log':'自动防卫单元已接管本区。','gh_entry_maintenance:gh_entry_tutorial_terminal:terminal':'维护终端：WASD / 方向键移动，鼠标指向瞄准；J / 左键攻击，Shift 冲刺。Q 脉冲，按住 E 防御，R 穿刺；F 交互，Space 情境穿越。T 任务，M 地图，Esc 暂停；动作与地图信息以当前状态和权限为准。'}
			return lines.get(before.sceneId+':'+id+':'+kind,'')
		if P.snapshot(before) and P.snapshot(after) and before.worldId=='mist_harbor' and before.sceneId=='mh_pump_station' and typeof(target.get('entityId'))==TYPE_STRING and typeof(target.get('kind'))==TYPE_STRING and id=='mh_pump_control_primary' and kind=='pump_control':
			var pump_before = before.get('mistHarborPump')
			var pump_after = after.get('mistHarborPump')
			if pump_before is Dictionary and pump_before.size()==1 and typeof(pump_before.get('state'))==TYPE_STRING and pump_before.state=='ready' and pump_after is Dictionary and pump_after.size()==1 and typeof(pump_after.get('state'))==TYPE_STRING and pump_after.state=='draining':
				var pump_rows = before.interactables.filter(func(row): return row.entityId==id and row.kind==kind)
				if pump_rows.size()==1 and typeof(pump_rows[0].get('active'))==TYPE_BOOL and pump_rows[0].active and P.transform(pump_rows[0].get('transform')):
					return '泵站：排水已开始。'
		if P.snapshot(before) and P.snapshot(after) and before.worldId=='mist_harbor' and typeof(target.get('entityId'))==TYPE_STRING and typeof(target.get('kind'))==TYPE_STRING and kind=='beacon':
			var beacon_event = 'mist_beacon_west' if before.sceneId=='mh_tidal_warehouse' and id=='mh_west_beacon' else 'mist_beacon_east' if before.sceneId=='mh_breakwater' and id=='mh_east_beacon' else ''
			if not beacon_event.is_empty() and before.progression.currentWorldId=='mist_harbor' and after.progression.currentWorldId=='mist_harbor':
				var beacon_rows = before.interactables.filter(func(row): return row.entityId==id and row.kind==kind)
				var beacon_prior = progress(before,'mist_harbor')
				var beacon_current = progress(after,'mist_harbor')
				if beacon_rows.size()==1 and typeof(beacon_rows[0].get('active'))==TYPE_BOOL and beacon_rows[0].active and P.transform(beacon_rows[0].get('transform')) and not beacon_prior.is_empty() and not beacon_current.is_empty() and beacon_prior.worldId=='mist_harbor' and beacon_current.worldId=='mist_harbor' and beacon_prior.visitId==beacon_current.visitId and beacon_prior.cycleId==beacon_current.cycleId and beacon_prior.revisitCount==beacon_current.revisitCount and after.progression.eventSeq==before.progression.eventSeq+1 and not beacon_event in beacon_prior.completedEvents and beacon_event in beacon_current.completedEvents and beacon_current.completedEvents.size()==beacon_prior.completedEvents.size()+1:
					var beacon_retained = true
					for event in beacon_prior.completedEvents:
						if not event in beacon_current.completedEvents:
							beacon_retained = false
					if beacon_retained:
						var beacon_text = EVENT_LABELS[beacon_event]+'事件已记录。'
						if 'mist_beacon_west' in beacon_current.completedEvents and 'mist_beacon_east' in beacon_current.completedEvents:
							beacon_text += ' 双基准建立，中心干扰源可定位。'
						return beacon_text
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
			var receipt_suffix = ''
			var receipt_context_valid = (
				selected == 'clockworks_valves'
				and P.snapshot(before)
				and P.snapshot(after)
				and before.has('entryToken')
				and before.entryToken == null
				and after.has('entryToken')
				and after.entryToken == null
				and before.get('worldId') == 'clockworks'
				and before.get('sceneId') == 'cw_pressure_hall'
				and after.get('worldId') == 'clockworks'
				and after.get('sceneId') == 'cw_pressure_hall'
				and before.progression.get('currentWorldId') == 'clockworks'
				and after.progression.get('currentWorldId') == 'clockworks'
				and typeof(target.get('entityId')) == TYPE_STRING
				and typeof(target.get('kind')) == TYPE_STRING
				and typeof(target.get('active')) == TYPE_BOOL
				and target.active
				and P.transform(target.get('transform'))
				and P.rows(before.get('interactables'), 2048)
				and P.unique(before.interactables, 'entityId')
			)
			var receipt_progress_valid = (
				not prior.is_empty()
				and not current.is_empty()
				and prior.get('worldId') == 'clockworks'
				and current.get('worldId') == 'clockworks'
				and prior.get('visitId') == current.get('visitId')
				and prior.get('cycleId') == current.get('cycleId')
				and prior.get('revisitCount') == current.get('revisitCount')
				and prior.get('completed') == current.get('completed')
				and prior.get('firstCompletion') == current.get('firstCompletion')
			)
			if receipt_context_valid and receipt_progress_valid:
				var receipt_capture_valid = (
					matches.size() == 1
					and typeof(matches[0].get('active')) == TYPE_BOOL
					and matches[0].active
					and P.transform(matches[0].get('transform'))
					and matches[0].transform == target.transform
				)
				if receipt_capture_valid:
					var receipt_event_path_valid = (
						prior.get('completedEvents') == current.get('completedEvents')
						and before.progression.get('eventSeq') == after.progression.get('eventSeq')
					)
					if selected == 'clockworks_valves' and new_event:
						receipt_event_path_valid = true
					var receipt_prior = prior.get('pressureValveReceipts')
					var receipt_current = current.get('pressureValveReceipts')
					var receipt_order = ['cw_pressure_valve_01_staged', 'cw_pressure_valve_02_staged', 'cw_pressure_valve_03_staged']
					var receipt_arrays_valid = receipt_prior is Array and receipt_current is Array
					if receipt_arrays_valid:
						receipt_arrays_valid = receipt_prior.size() <= 3 and receipt_current.size() <= 3
					if receipt_arrays_valid:
						var prior_receipt_order = -1
						for prior_receipt_id in receipt_prior:
							if typeof(prior_receipt_id) != TYPE_STRING:
								receipt_arrays_valid = false
								break
							var prior_receipt_index = receipt_order.find(prior_receipt_id)
							if prior_receipt_index < 0 or prior_receipt_index <= prior_receipt_order:
								receipt_arrays_valid = false
								break
							prior_receipt_order = prior_receipt_index
						if receipt_arrays_valid:
							var current_receipt_order = -1
							for current_receipt_id in receipt_current:
								if typeof(current_receipt_id) != TYPE_STRING:
									receipt_arrays_valid = false
									break
								var current_receipt_index = receipt_order.find(current_receipt_id)
								if current_receipt_index < 0 or current_receipt_index <= current_receipt_order:
									receipt_arrays_valid = false
									break
								current_receipt_order = current_receipt_index
					if receipt_arrays_valid and receipt_event_path_valid:
						var prior_receipts_retained = true
						for retained_receipt_id in receipt_prior:
							if not receipt_current.has(retained_receipt_id):
								prior_receipts_retained = false
						if prior_receipts_retained and not receipt_prior.has(id) and receipt_current.has(id) and receipt_current.size() == receipt_prior.size() + 1:
							receipt_suffix = ' 阀门 ' + valves[id] + ' 回执：已记录'
			if new_event:
				var recorded = '压力阀门事件已记录。' if selected=='clockworks_valves' else '调节者核心事件已记录。'
				return acknowledgement+(' ' if not acknowledgement.is_empty() else '')+recorded+receipt_suffix
			return acknowledgement+receipt_suffix
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
