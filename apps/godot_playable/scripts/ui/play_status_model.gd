extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const Content = preload('res://scripts/ui/content_model.gd')
const PHASES = {'windup':'准备','active':'执行','recovery':'恢复'}
static func project(s: Dictionary, eligible: bool) -> Dictionary:
	var result = {'objectives':[],'milestones':[],'action':{}}
	if not eligible or s.is_empty() or not P.context(s) or not P.counter(s.get('protocolVersion'),3,3) or not Content.WORLD_NAMES.has(s.get('worldId')) or not Content.SCENE_NAMES.has(s.get('sceneId')):
		return result
	result.objectives = objectives(s.get('objectives'))
	result.milestones = milestones(s)
	result.action = action(s.get('player'))
	return result
static func objectives(rows) -> Array:
	var lines: Array = []
	if not P.rows(rows,2048) or not P.unique(rows,'objectiveId'):
		return lines
	for row in rows:
		if not P.point(row.get('positionM')) or typeof(row.get('state'))!=TYPE_STRING:
			return []
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
