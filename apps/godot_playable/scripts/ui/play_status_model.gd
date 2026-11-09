extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const Content = preload('res://scripts/ui/content_model.gd')
const PHASES = {'windup':'准备','active':'执行','recovery':'恢复'}
static func project(s: Dictionary, eligible: bool) -> Dictionary:
	var result = {'objectives':[],'milestones':[],'action':{},'current_action':'','guidance':[],'guidance_compact':'','hasActiveObjective':false}
	if not eligible or s.is_empty() or not P.context(s) or not P.counter(s.get('protocolVersion'),3,3) or not Content.WORLD_NAMES.has(s.get('worldId')) or not Content.SCENE_NAMES.has(s.get('sceneId')):
		return result
	result.objectives = objectives(s.get('objectives'))
	if _valid_objective_rows(s.get('objectives')):
		for row in s.objectives:
			if row.state=='active': result.hasActiveObjective = true; break
	result.milestones = milestones(s)
	result.action = action(s.get('player'))
	if P.snapshot(s) and s.player.currentHp>0 and s.get('entryToken')==null and s.get('exitEntryPreview')!=true and s.get('shaftServiceEntryPreview')!=true:
		var names = {'primaryAttack':'攻击','pulse':'脉冲','guard':'防御','pierce':'穿刺','dash':'冲刺'}
		if s.player.actionState=='dash':
			result.current_action = '当前动作：冲刺'
		elif not result.action.is_empty() and names.has(result.action.kind):
			var phases = {'windup':'准备中','active':'执行中','recovery':'收势中'}
			result.current_action = '当前动作：'+names[result.action.kind]+' · '+phases[result.action.phase]
		var route = route_guidance(s)
		if not route.is_empty() and route[0]!='任务参考：当前路线状态未确认。' and not route[0].begins_with('三处世界进度均已记录'):
			for line in route:
				if typeof(line)==TYPE_STRING:
					result.guidance.append(line.replace('恢复场景后，',''))
			result.guidance_compact = _compact_guidance(result.guidance)
	return result
static func _valid_objective_rows(rows) -> bool:
	if not P.rows(rows,2048) or not P.unique(rows,'objectiveId'): return false
	for row in rows:
		if not P.point(row.get('positionM')) or typeof(row.get('state'))!=TYPE_STRING: return false
	return true

static func _compact_guidance(lines: Array) -> String:
	if lines.is_empty(): return ''
	var first: String = lines[0]
	var copies = {
		'任务参考：靠近灰巢入口查看当前交互。':'下一步：灰巢入口',
		'任务参考：靠近雾港入口查看当前交互。':'下一步：雾港入口',
		'任务参考：靠近钟骨入口查看当前交互。':'下一步：钟骨入口',
		'复访任务参考：以当前事件、门和交互提示为准；首通奖励不重复。':'复访：以当前交互为准',
		'任务参考：沿当前合法通路前往主电室，查看恢复主电的交互。':'下一步：查看主电交互',
		'任务参考：沿当前合法通路前往隔离封锁，查看隔离协议交互。':'下一步：查看隔离协议',
		'任务参考：灰巢撤离事件已记录；查看灰巢撤离区的当前归航交互。':'下一步：查看灰巢归航交互',
		'任务参考：信标收取状态未确认；查看当前信标交互提示。':'信标状态未确认：查看交互',
		'任务参考：前往信标室查看收取信标的交互；部署为可选。':'下一步：查看信标收取交互',
		'任务参考：前往灰巢撤离区查看撤离控制；具体条件以当前交互为准。':'下一步：查看撤离控制',
		'任务参考：沿当前合法通路前往共鸣塔，查看信号终端。':'下一步：查看共鸣塔终端',
		'任务参考：处理守望者区域的战斗后，前往雾港撤离区查看当前归航交互。':'下一步：守望者区域与撤离',
		'任务参考：前往压力大厅查看三处压力阀门；具体条件以当前交互为准。':'下一步：查看压力阀门',
		'任务参考：沿当前合法通路前往调节者核心，处理当前战斗与终端。':'下一步：调节者核心',
		'任务参考：前往停机出口查看停机终端。':'下一步：查看停机终端',
		'任务参考：停机事件已记录；查看停机出口的当前归航交互。':'下一步：查看停机出口归航交互'}
	if first=='任务参考：两处航标可以按任意顺序确认。':
		var west = '潮汐仓库：查看西侧航标交互。' in lines
		var east = '防波堤：查看东侧航标交互。' in lines
		if west and east: return '航标任选：潮汐仓库西 / 防波堤东'
		if west: return '下一步：潮汐仓库西侧航标'
		if east: return '下一步：防波堤东侧航标'
		return ''
	return copies.get(first,'')
static func objectives(rows) -> Array:
	var lines: Array = []
	if not _valid_objective_rows(rows): return lines
	# Separate local buckets preserve source order without mutating snapshot rows.
	for rank in ['active','locked','complete','unknown']:
		for row in rows:
			var state: String = row.state
			var group = state if state in ['active','locked','complete'] else 'unknown'
			if group==rank:
				lines.append(Content.objective({'objectiveId':row.objectiveId,'state':state}))
				if lines.size()==3:
					return lines
	return lines
static func milestones(s: Dictionary) -> Array:
	var lines: Array = []
	var world: String = s.worldId
	if Content.EVENTS.has(world):
		var progress = Content.progress(s,world)
		if progress.is_empty():
			return lines
		for event in Content.EVENTS[world]:
			lines.append(('✓ ' if event in progress.completedEvents else '○ ')+Content.EVENT_LABELS[event])
			if lines.size()==3:
				break
		if world=='grey_hive' and P.snapshot(s) and s.progression.currentWorldId=='grey_hive' and s.get('entryToken')==null and progress.visitId>0:
			var choice = progress.get('hiveChoice')
			if typeof(choice)==TYPE_STRING:
				if choice=='taken':
					lines.append('已记录的白芷选择：我会帮你离开这里。')
				elif choice=='left':
					lines.append('已记录的白芷选择：我不能带你走。')
	elif world=='return_station':
		for known in ['grey_hive','mist_harbor','clockworks']:
			var progress = Content.progress(s,known)
			if not progress.is_empty():
				lines.append(Content.WORLD_NAMES[known]+' · '+('已完成' if progress.completed else '未完成'))
	return lines
static func action(player) -> Dictionary:
	if not player is Dictionary or typeof(player.get('actionState'))!=TYPE_STRING or not player.actionState in ['primaryAttack','pulse','guard','pierce','dash']:
		return {}
	var a = player.get('actionPresentation')
	if not a is Dictionary or not P.counter(a.get('requestId'),1) or typeof(a.get('phase'))!=TYPE_STRING or not PHASES.has(a.phase) or not P.counter(a.get('durationMs'),1,60000) or not P.counter(a.get('elapsedMs'),0,60000) or a.elapsedMs>=a.durationMs or not P.number(a.get('rangeM'),0,64) or not P.number(a.get('lineHalfWidthM'),0,8):
		return {}
	var remaining = int(a.durationMs-a.elapsedMs)
	return {'kind':player.actionState,'phase':a.phase,'remainingMs':remaining,'text':PHASES[a.phase]+' · '+str(remaining)+'ms'}

static func route_guidance(s: Dictionary) -> Array:
	var neutral = ['任务参考：当前路线状态未确认。']
	if not P.snapshot(s) or s.player.currentHp<=0 or s.get('entryToken')!=null: return []
	var world: String = s.worldId
	var scene: String = s.sceneId
	if not Content.WORLD_NAMES.has(world) or not Content.SCENE_NAMES.has(scene):
		return neutral
	if world=='return_station':
		if scene!='rs_core_room':
			return neutral
	elif not scene.begins_with({'grey_hive':'gh_','mist_harbor':'mh_','clockworks':'cw_'}[world]):
		return neutral
	var rows = {}
	for id in ['grey_hive','mist_harbor','clockworks'] if world=='return_station' else [world]:
		var row: Dictionary = Content.progress(s,id)
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
		var completed_summary: Array = ['三处世界进度均已记录为完成。可查看当前保存与复访选项。']
		if rows.clockworks.firstCompletion==true and rows.clockworks.completedEvents.size()==3 and P.granted(s,'mobility.air_step_i'):
			completed_summary.append('三处坐标不是孤立事件。网络中仍存在未识别节点。')
		return completed_summary
	if s.progression.get('currentWorldId')!=world:
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
		var beacon = s.get('greyHiveBeacon')
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
