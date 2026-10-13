extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
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
