extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const LABELS = {'enemy_hit':'命中敌人','player_hurt':'受到伤害','blocked':'攻击已格挡','absorbed':'伤害被抵消','rejected':'动作未执行'}
const ABSORB_REASONS = {'invulnerable':'无敌期间','damage_reduced_to_zero':'伤害降至零'}
const REJECTION_REASONS = {'cooldown':'技能仍在冷却','insufficient_energy':'能量不足','action_busy':'当前动作尚未结束','guard_not_active':'防御已结束','dash_rejected':'当前无法冲刺','no_target':'未命中目标'}
static func rejection_text(reason) -> String:
	if typeof(reason)!=TYPE_STRING: return '动作结果未确认'
	return REJECTION_REASONS.get(reason,'动作结果未确认')
var context: Dictionary = {}
var owner_generation = -1
var owner_connection = -1
var baseline_tick = -1
var last_event_id = 0
var history: Array = []
func clear() -> void:
	context.clear()
	owner_generation = -1
	owner_connection = -1
	baseline_tick = -1
	last_event_id = 0
	history.clear()
func update(s: Dictionary, eligible: bool, generation: int, connection: int, cursor: int, projected: Array) -> Array:
	if not eligible or not P.snapshot(s) or s.get('entryToken')!=null or s.player.currentHp<=0 or not P.counter(generation,1) or not P.counter(connection) or not P.counter(cursor):
		clear()
		return []
	var current = {'worldId':s.worldId,'sceneId':s.sceneId,'worldEpoch':s.worldEpoch}
	if context.is_empty() or not P.same(context,current) or owner_generation!=generation or owner_connection!=connection:
		clear()
		context = current
		owner_generation = generation
		owner_connection = connection
		baseline_tick = int(s.serverTick)
		last_event_id = cursor
		return []
	var now = Time.get_ticks_msec()
	history = history.filter(func(row): return row.expires>now)
	for event in projected:
		if not event is Dictionary or not P.counter(event.get('eventId'),1) or not P.counter(event.get('serverTick')) or event.eventId<=last_event_id or event.serverTick<=baseline_tick:
			continue
		var feedback = event.get('combatFeedback')
		if not feedback is Dictionary or feedback.get('worldId')!=s.worldId or feedback.get('sceneId')!=s.sceneId or not LABELS.has(feedback.get('outcome')):
			continue
		last_event_id = int(event.eventId)
		var text: String = LABELS[feedback.outcome]
		if feedback.outcome=='rejected':
			text = rejection_text(feedback.get('reason'))
		if feedback.outcome=='absorbed' and ABSORB_REASONS.has(feedback.get('reason')):
			text += ' · '+ABSORB_REASONS[feedback.reason]
		history.append({'text':text,'expires':now+2000})
		if history.size()>3:
			history.pop_front()
	var lines: Array = []
	for row in history:
		lines.append(row.text)
	return lines
