extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const Presentation = preload('res://scripts/world/presentation.gd')
const Content = preload('res://scripts/ui/content_model.gd')
const VALVES = ['cw_pressure_valve_01_staged','cw_pressure_valve_02_staged','cw_pressure_valve_03_staged']
static func project(s: Dictionary, eligible) -> Array:
	if typeof(eligible)!=TYPE_BOOL or not eligible or s.get('kind')!='full' or s.get('schemaVersion')!='freeze-v02-interfaces/1.2' or not P.counter(s.get('protocolVersion'),3,3) or not P.context(s) or s.get('entryToken')!=null or not Content.WORLD_NAMES.has(s.worldId) or not Content.SCENE_NAMES.has(s.sceneId):
		return []
	for field in ['worldEpoch','serverTick','authorityRevision','ackSeq']:
		if not P.counter(s.get(field)):
			return []
	var prefix: String = {'grey_hive':'gh_','mist_harbor':'mh_','clockworks':'cw_','return_station':'rs_'}[s.worldId]
	if not s.sceneId.begins_with(prefix):
		return []
	if s.worldId=='grey_hive':
		var decon_lines: Array = local_decon_environment(s)
		var beacon = s.get('greyHiveBeacon')
		if not beacon is Dictionary or beacon.size()!=2 or typeof(beacon.get('state'))!=TYPE_STRING or not beacon.state in ['uncollected','carried','mounted'] or typeof(beacon.get('legacyCompletedWithoutReceipt'))!=TYPE_BOOL or beacon.legacyCompletedWithoutReceipt and beacon.state!='uncollected':
			return ['信标：状态未确认']+extraction_controls(s)+decon_lines
		if beacon.legacyCompletedWithoutReceipt:
			return ['信标：收取回执未确认']+extraction_controls(s)+decon_lines
		return ['信标：'+{'uncollected':'尚未收取','carried':'已携带','mounted':'已部署'}[beacon.state]]+extraction_controls(s)+decon_lines
	if s.worldId=='mist_harbor':
		var pump = s.get('mistHarborPump')
		if not pump is Dictionary or pump.size()!=1 or typeof(pump.get('state'))!=TYPE_STRING or not pump.state in ['ready','draining','drained']:
			return ['泵站：状态未确认']+extraction_controls(s)
		return ['泵站：'+{'ready':'就绪','draining':'排水中','drained':'排水完成'}[pump.state]]+extraction_controls(s)
	if s.worldId=='clockworks' and s.sceneId in ['cw_conveyor_bridge','cw_gear_shaft']:
		return local_traversal(s)
	if s.worldId=='clockworks' and s.sceneId in ['cw_boiler_chamber','cw_furnace_heart']:
		return local_environment(s)
	if s.worldId=='clockworks' and s.sceneId=='cw_pressure_hall':
		return valves(s.get('interactables')) + pressure_valve_receipts(s)
	if s.worldId=='clockworks' and s.sceneId=='cw_shutdown_exit':
		return extraction_controls(s)
	return []
static func extraction_controls(s: Dictionary) -> Array:
	if not P.context(s) or s.get('entryToken')!=null:
		return []
	var specs: Array = []
	if s.worldId=='grey_hive' and s.sceneId in ['gh_power_room','gh_lockdown']:
		if not P.snapshot(s):
			return []
		var current_specs: Dictionary = {
			'gh_power_room':['gh_power_console','power_console','主电控制'],
			'gh_lockdown':['gh_lockdown_terminal','lockdown_terminal','隔离控制']
		}
		var current_spec: Array = current_specs[s.sceneId]
		var known_rows: Array = s.interactables.filter(func(row): return row.entityId==current_spec[0])
		if known_rows.size()!=1:
			return []
		specs = [current_spec]
	elif s.worldId=='grey_hive' and s.sceneId=='gh_exit':
		specs = [['gh_exit_extraction_console','extraction_console','撤离控制'],['gh_extraction_return_to_rs','world_gate','归航门']]
	elif s.worldId=='mist_harbor' and s.sceneId=='mh_extraction':
		specs = [['mh_extraction_return_to_rs','world_gate','归航门']]
	elif s.worldId=='clockworks' and s.sceneId=='cw_shutdown_exit':
		specs = [['cw_master_shutdown_staged','terminal','停机控制'],['cw_shutdown_return_to_rs','world_gate','归航门']]
	else:
		return []
	var lines: Array = []
	var rows = s.get('interactables')
	var valid_rows = P.snapshot(s) and P.rows(rows,2048) and P.unique(rows,'entityId')
	for spec in specs:
		var state = '状态未确认'
		if valid_rows:
			var matches = rows.filter(func(row): return row.entityId==spec[0])
			if matches.size()==1 and typeof(matches[0].get('kind'))==TYPE_STRING and matches[0].kind==spec[1] and typeof(matches[0].get('active'))==TYPE_BOOL and P.transform(matches[0].get('transform')):
				state = '当前启用' if matches[0].active else '当前未启用'
		lines.append(spec[2]+'：'+state)
	return lines
static func valves(rows) -> Array:
	var neutral: Array = ['压力阀门：状态未确认']
	if not P.rows(rows,2048) or not P.unique(rows,'entityId'):
		return neutral
	var selected = {}
	for row in rows:
		if row.entityId in VALVES:
			if row.get('kind')!='valve_control' or typeof(row.get('active'))!=TYPE_BOOL or not P.transform(row.get('transform')):
				return neutral
			selected[row.entityId] = row.active
	if selected.size()!=3:
		return neutral
	var lines: Array = []
	for n in range(3):
		lines.append('阀门 '+str(n+1)+'：'+('当前启用' if selected[VALVES[n]] else '当前未启用'))
	return lines
static func pressure_valve_receipts(s: Dictionary) -> Array:
	if not P.snapshot(s) or s.get('worldId') != 'clockworks' or s.get('sceneId') != 'cw_pressure_hall' or s.get('entryToken') != null:
		return []
	var progression = Content.progress(s, 'clockworks')
	if progression.is_empty():
		return []
	var raw_receipts = progression.get('pressureValveReceipts')
	if raw_receipts == null:
		return []
	if typeof(raw_receipts) != TYPE_ARRAY or raw_receipts.size() > VALVES.size():
		return ['阀门回执：状态未确认']
	var previous_index = -1
	for receipt in raw_receipts:
		if typeof(receipt) != TYPE_STRING:
			return ['阀门回执：状态未确认']
		var valve_index = VALVES.find(receipt)
		if valve_index < 0 or valve_index <= previous_index:
			return ['阀门回执：状态未确认']
		previous_index = valve_index
	var lines: Array = ['阀门回执：'+str(raw_receipts.size())+' / 3']
	for n in range(VALVES.size()):
		var state = '已记录' if raw_receipts.has(VALVES[n]) else '未记录'
		lines.append('阀门 '+str(n+1)+' 回执：'+state)
	return lines
static func local_decon_environment(s: Dictionary) -> Array:
	if not P.snapshot(s) or s.get('worldId')!='grey_hive' or s.get('sceneId')!='gh_deep_decon' or s.get('entryToken')!=null:
		return []
	var point: Dictionary = s.player.transform.positionM
	if not P.number(point.yM,0,1):
		return []
	var specs: Array = [
		['gh_decon_steam_jet_zone','steam_jet','pressure','蒸汽',[[7,4],[10,4],[10,6.5],[7,6.5]]],
		['gh_decon_mist_zone','decon_mist','toxin','消杀雾',[[14,9],[18,9],[18,14],[14,14]]]
	]
	var lines: Array = []
	for spec in specs:
		for h in s.hazards:
			if h.entityId!=spec[0] or h.kind!=spec[1] or typeof(h.get('phaseActive'))!=TYPE_BOOL or h.has('heightRangeM'):
				continue
			var e = Presentation.environment(h)
			if e.is_empty() or e.tag!=spec[2] or h.polygonM!=spec[4] or not decon_contains(point,h.polygonM):
				continue
			lines.append(spec[3]+'：'+{'warning':'预警','active':'危险','recovery':'间歇','suppressed':'已抑制'}[e.phase])
	return lines
static func decon_contains(point: Dictionary, polygon: Array) -> bool:
	if in_polygon(point,polygon):
		return true
	for n in range(polygon.size()):
		var a: Array = polygon[n]
		var b: Array = polygon[(n+1)%polygon.size()]
		var cross: float = (point.xM-a[0])*(b[1]-a[1])-(point.zM-a[1])*(b[0]-a[0])
		if abs(cross)<=0.00001 and point.xM>=minf(a[0],b[0])-0.00001 and point.xM<=maxf(a[0],b[0])+0.00001 and point.zM>=minf(a[1],b[1])-0.00001 and point.zM<=maxf(a[1],b[1])+0.00001:
			return true
	return false
static func local_environment(s: Dictionary) -> Array:
	if not P.snapshot(s) or not P.point(s.player.transform.positionM) or not P.number(s.player.transform.positionM.yM,0,0.1):
		return []
	var specs: Array = []
	if s.sceneId=='cw_boiler_chamber':
		specs = [['cw_heat_accumulation_staged','heat_zone','heat','热区',[[11,2],[17,2],[17,6],[11,6]]],['cw_boiler_vent_staged','steam_jet','pressure','蒸汽口',[[11,10],[13,10],[13,13],[11,13]]]]
	elif s.sceneId=='cw_furnace_heart':
		specs = [['cw_heat_accumulation_staged','heat_zone','heat','热区',[[8,2],[14,2],[14,6],[8,6]]]]
	else:
		return []
	var hazards = s.get('hazards')
	if not P.rows(hazards,2048) or not P.unique(hazards,'entityId'):
		return ['环境状态未确认']
	var selected = {}
	for h in hazards:
		selected[h.entityId] = h
	var lines: Array = []
	for spec in specs:
		var h = selected.get(spec[0])
		if not h is Dictionary or h.get('kind')!=spec[1] or typeof(h.get('active'))!=TYPE_BOOL or typeof(h.get('phaseActive'))!=TYPE_BOOL or not P.transform(h.get('transform')):
			lines.append(spec[3]+'：状态未确认')
			continue
		var e = Presentation.environment(h)
		if e.is_empty() or e.tag!=spec[2] or h.polygonM!=spec[4]:
			lines.append(spec[3]+'：状态未确认')
			continue
		if not in_polygon(s.player.transform.positionM,h.polygonM) and not (e.tag=='heat' and e.exposureBps>0):
			continue
		var line: String = spec[3]+'：'+{'warning':'预警','active':'生效','recovery':'恢复','suppressed':'抑制'}[e.phase]+' · 当前阶段余时 '+str(int(e.remainingMs))+'ms'
		if e.tag=='heat':
			line += ' · 热暴露 '+str(float(e.exposureBps)/100.0)+'%'
		lines.append(line)
	return lines
static func in_polygon(point: Dictionary, polygon: Array) -> bool:
	var inside = false
	var x: float = point.xM
	var z: float = point.zM
	for n in range(polygon.size()):
		var a: Array = polygon[n]
		var b: Array = polygon[(n+1)%polygon.size()]
		var cross = (x-a[0])*(b[1]-a[1])-(z-a[1])*(b[0]-a[0])
		if cross==0 and x>=minf(a[0],b[0]) and x<=maxf(a[0],b[0]) and z>=minf(a[1],b[1]) and z<=maxf(a[1],b[1]):
			return true
		if (a[1]>z)!=(b[1]>z) and x<(b[0]-a[0])*(z-a[1])/(b[1]-a[1])+a[0]:
			inside = not inside
	return inside
static func local_traversal(s: Dictionary) -> Array:
	if not P.snapshot(s):
		return []
	var point: Dictionary = s.player.transform.positionM
	if not P.point(point):
		return []
	if s.worldId=='clockworks' and s.sceneId=='cw_gear_shaft' and P.granted(s,'mobility.air_step_i') and P.number(point.yM,0,0.1):
		var earned_dx: float = float(point.xM)-11.0
		var earned_dz: float = float(point.zM)-8.0
		if earned_dx*earned_dx+earned_dz*earned_dz<=1.2*1.2:
			return ['Space：空中踏步起点']
	var start_x = 8.0
	var minimum_y = 0.0
	var maximum_y = 0.1
	var label = 'Space：输送桥短跨起点'
	if s.sceneId=='cw_gear_shaft':
		start_x = 17.8
		minimum_y = 1.9
		maximum_y = 2.1
		label = 'Space：上层间隙起点'
	if not P.point(point) or not P.number(point.yM,minimum_y,maximum_y):
		return []
	var dx: float = float(point.xM)-start_x
	var dz: float = float(point.zM)-8.0
	if dx*dx+dz*dz>1.2*1.2:
		return []
	return [label]
