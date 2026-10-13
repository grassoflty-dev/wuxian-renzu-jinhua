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
		var beacon = s.get('greyHiveBeacon')
		if not beacon is Dictionary or beacon.size()!=2 or typeof(beacon.get('state'))!=TYPE_STRING or not beacon.state in ['uncollected','carried','mounted'] or typeof(beacon.get('legacyCompletedWithoutReceipt'))!=TYPE_BOOL or beacon.legacyCompletedWithoutReceipt and beacon.state!='uncollected':
			return ['信标：状态未确认']
		if beacon.legacyCompletedWithoutReceipt:
			return ['信标：收取回执未确认']
		return ['信标：'+{'uncollected':'尚未收取','carried':'已携带','mounted':'已部署'}[beacon.state]]
	if s.worldId=='mist_harbor':
		var pump = s.get('mistHarborPump')
		if not pump is Dictionary or pump.size()!=1 or typeof(pump.get('state'))!=TYPE_STRING or not pump.state in ['ready','draining','drained']:
			return ['泵站：状态未确认']
		return ['泵站：'+{'ready':'就绪','draining':'排水中','drained':'排水完成'}[pump.state]]
	if s.worldId=='clockworks' and s.sceneId in ['cw_conveyor_bridge','cw_gear_shaft']:
		return local_traversal(s)
	if s.worldId=='clockworks' and s.sceneId in ['cw_boiler_chamber','cw_furnace_heart']:
		return local_environment(s)
	if s.worldId=='clockworks' and s.sceneId=='cw_pressure_hall':
		return valves(s.get('interactables'))
	return []
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
		lines.append('阀门 '+str(n+1)+'：'+('可交互' if selected[VALVES[n]] else '当前不可交互'))
	return lines
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
