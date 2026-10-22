extends SceneTree
# SOURCE ONLY: not executed. Off-tree controls; never call HUD _ready/update_view.
const Hud = preload('res://scripts/ui/game_hud.gd')
const P = preload('res://scripts/bridge/protocol.gd')
const Fixture = preload('res://tests/ui/decon_hazard_text_contract.gd')
const BUSY = ['primaryAttack','dash','pulse','pierce','guard']
var failures: int = 0
var events: Array = []
var tips: Dictionary = {}
func _initialize() -> void:
	var s: Dictionary = Fixture._fixture()
	_check(P.snapshot(s),'existing full fixture validates')
	var before: Dictionary = s.duplicate(true)
	var h = Hud.new()
	var buttons = HFlowContainer.new()
	h.add_child(buttons)
	for id in Hud.PLAYER_ACTION_LABELS:
		h._add_player_action(buttons,id)
		tips[id] = h.player_action_states[id].button.tooltip_text
	h.guard_button = Button.new()
	h.guard_button.disabled = true
	h.guard_button.visible = false
	h.add_child(h.guard_button)
	h.intent.connect(func(kind, value): events.append([kind,value.duplicate(true)]))
	var actions: Array = []
	for id in Hud.PLAYER_ACTION_LABELS:
		actions.append(_association(s,id,'player_action',Hud.PLAYER_ACTION_LABELS[id]))
	var original: Array = actions.duplicate(true)
	_transitions(h,s,actions)
	_ineligible(h,s,actions)
	_invalid(h,s,actions)
	_guard(h,s,actions)
	_check(s==before and actions==original,'fixture and action inputs preserved')
	h.free()
	print('COMBAT_ACTION_BUSY_CONTRACT failures='+str(failures))
	quit(0 if failures==0 else 1)

func _check(ok: bool, label: String) -> void:
	if not ok:
		failures += 1
		push_error(label)

func _association(s: Dictionary, id: String, kind: String, label: String) -> Dictionary:
	return {'generation':7,'context':{'worldId':s.worldId,'sceneId':s.sceneId,
		'worldEpoch':s.worldEpoch},'entityId':id,'kind':kind,'label':label}

func _update(h, s: Dictionary, eligible: bool, actions: Array) -> void:
	var before: Dictionary = s.duplicate(true)
	var rows: Array = actions.duplicate(true)
	_check(P.snapshot(s),'transition snapshot validates')
	h._update_player_actions(s,eligible,actions)
	_check(s==before and actions==rows,'update does not mutate inputs')

func _idle(h) -> void:
	for id in Hud.PLAYER_ACTION_LABELS:
		var st: Dictionary = h.player_action_states[id]
		_check(st.button.visible and not st.button.disabled and st.button.tooltip_text==tips[id],id+'/idle tooltip restored')

func _hidden(h) -> void:
	for st in h.player_action_states.values():
		_check(not st.button.visible and st.button.disabled and st.press.is_empty() and st.display.is_empty(),'unavailable clears capture')
		st.button.pressed.emit()

func _transitions(h, s: Dictionary, actions: Array) -> void:
	for busy in BUSY:
		for id in Hud.PLAYER_ACTION_LABELS:
			events.clear()
			s.player.actionState = 'idle'
			_update(h,s,true,actions)
			_idle(h)
			var st: Dictionary = h.player_action_states[id]
			var expected: Dictionary = st.display.duplicate(true)
			st.button.button_down.emit()
			_check(not expected.is_empty() and st.press==expected,busy+'/'+id+'/capture')
			s.player.actionState = busy
			_update(h,s,true,actions)
			for key in Hud.PLAYER_ACTION_LABELS:
				var row: Dictionary = h.player_action_states[key]
				_check(row.button.visible and row.button.disabled and row.press.is_empty(),busy+'/'+key+'/busy disabled and cleared')
				_check(row.button.tooltip_text==tips[key]+'\n当前动作尚未结束',busy+'/'+key+'/busy tooltip')
			st.button.button_down.emit()
			st.button.pressed.emit()
			_check(events.is_empty() and st.press.is_empty(),busy+'/'+id+'/busy cannot capture or emit')
			s.player.actionState = 'idle'
			_update(h,s,true,actions)
			_idle(h)
			st.button.pressed.emit()
			_check(events.is_empty(),busy+'/'+id+'/old release cannot emit after idle')
			st.button.button_down.emit()
			st.button.pressed.emit()
			_check(events==[['player_action_current',expected]],busy+'/'+id+'/fresh click emits original association once')
			st.button.pressed.emit()
			_check(events.size()==1 and st.press.is_empty() and st.display.is_empty() and st.button.disabled,id+'/consumed once')

func _ineligible(h, s: Dictionary, actions: Array) -> void:
	for state in ['idle']+BUSY:
		events.clear()
		s.player.actionState = 'idle'
		_update(h,s,true,actions)
		for st in h.player_action_states.values():
			st.button.button_down.emit()
		s.player.actionState = state
		_update(h,s,false,actions)
		_hidden(h)
		s.player.actionState = 'idle'
		_update(h,s,true,actions)
		_idle(h)
		for st in h.player_action_states.values():
			st.button.pressed.emit()
		_check(events.is_empty(),state+'/ineligible capture never revives')

func _invalid(h, s: Dictionary, actions: Array) -> void:
	for problem in ['short','duplicate','generation','context','label','snapshot']:
		events.clear()
		_update(h,s,true,actions)
		h.player_action_states.primaryAttack.button.button_down.emit()
		var rows: Array = actions.duplicate(true)
		var sample: Dictionary = s.duplicate(true)
		match problem:
			'short': rows.pop_back()
			'duplicate': rows[1] = rows[0].duplicate(true)
			'generation': rows[1].generation += 1
			'context': rows[1].context.worldEpoch += 1
			'label': rows[1].label = 'wrong'
			'snapshot': sample.player.erase('currentHp')
		_check(P.snapshot(sample)==(problem!='snapshot'),problem+'/fixture gate')
		h._update_player_actions(sample,true,rows)
		_hidden(h)
		_update(h,s,true,actions)
		_idle(h)
		h.player_action_states.primaryAttack.button.pressed.emit()
		_check(events.is_empty(),problem+'/invalid capture cannot revive')

func _guard(h, s: Dictionary, actions: Array) -> void:
	events.clear()
	var association: Dictionary = _association(s,'guard','player_guard','防御')
	h._update_guard(s,true,association)
	h.guard_capture = association.duplicate(true)
	s.player.actionState = 'guard'
	_update(h,s,true,actions)
	h._update_guard(s,true,association)
	_check(h.guard_capture==association and h.guard_button.visible and not h.guard_button.disabled and events.is_empty(),'held guard survives busy refresh')
	h._cancel_guard_capture()
	h._cancel_guard_capture()
	_check(events==[['guard_current_end',association]] and h.guard_capture.is_empty(),'guard release ends exactly once')
	h.guard_capture = association.duplicate(true)
	h._update_guard(s,false,association)
	_check(events==[['guard_current_end',association],['guard_current_end',association]],'ineligible guard cancels once')
	_check(h.guard_capture.is_empty() and not h.guard_button.visible and h.guard_button.disabled,'guard unavailable clears held state')
	s.player.actionState = 'idle'
