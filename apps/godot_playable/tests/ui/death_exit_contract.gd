extends SceneTree

# Off-tree controls only: exercise the real Panels page and signal callbacks.
const Panels = preload('res://scripts/ui/panels.gd')

var checks: int = 0
var failures: int = 0
var events: Array = []

func _initialize() -> void:
	var ui = Panels.new()
	ui.box = VBoxContainer.new()
	ui.add_child(ui.box)
	# show_page's focus request is unnecessary for this off-tree fixture.
	ui.visible = true
	ui.intent.connect(func(kind, value, owner): events.append([kind,value,owner.duplicate(true)]))

	var snapshot: Dictionary = {'worldId':'grey_hive','sceneId':'gh_entry_maintenance',
		'worldEpoch':7,'player':{'currentHp':0},'save':{'slot':'fixture','unsaved':true}}
	var snapshot_before: Dictionary = snapshot.duplicate(true)
	ui.set_save_availability('available','list',[{'slotId':'fixture_slot','valid':true}])
	var slots_before: Array = ui.slots.duplicate(true)
	var owners: Array = [_owner(11),_owner(12),_owner(13),_owner(14)]
	var owners_before: Array = owners.duplicate(true)

	ui.show_page('death',owners[0],snapshot,true)
	_check(_labels_named(ui,'当前角色已死亡；输入已停止。')==1,'original death message retained')
	_check(_labels_named(ui,'仅能继续此前已成功保存的进度；退出不会保存本次未保存进度。')==1,'successful-save and no-save explanation')
	var exit_buttons: Array = _buttons_named(ui,'退出游戏（不保存）')
	var save_buttons: Array = _buttons_named(ui,'从存档继续')
	var new_buttons: Array = _buttons_named(ui,'新的旅程')
	_check(exit_buttons.size()==1 and save_buttons.size()==1 and new_buttons.size()==1,'exactly one of each death action')
	_check(_button_count(ui)==3,'no additional death action controls')
	if exit_buttons.size()==1 and save_buttons.size()==1 and new_buttons.size()==1:
		_check(not exit_buttons[0].disabled and not save_buttons[0].disabled and not new_buttons[0].disabled,'connected actions enabled')
		exit_buttons[0].pressed.emit()
		_check(events==[['close_client',null,owners[0]]],'exit emits only close_client with owner and null payload')
		exit_buttons[0].pressed.emit()
		_check(events==[['close_client',null,owners[0]],['close_client',null,owners[0]]],'one close intent per deliberate press')
		events.clear()
		save_buttons[0].pressed.emit()
		new_buttons[0].pressed.emit()
		_check(events==[['slots',null,owners[0]],['new_confirm',null,owners[0]]],'connected recovery intents unchanged')

	var old_children: Array = ui.box.get_children()
	var stale_exit = exit_buttons[0] if exit_buttons.size()==1 else null
	ui.show_page('death',owners[1],snapshot,false)
	events.clear()
	_check(is_instance_valid(stale_exit),'old exit callback retained for owner-replacement check')
	if is_instance_valid(stale_exit):
		stale_exit.pressed.emit()
	_check(events.is_empty(),'stale owner callback cannot emit after replacement')
	_free_controls(old_children)
	exit_buttons = _buttons_named(ui,'退出游戏（不保存）')
	save_buttons = _buttons_named(ui,'从存档继续')
	new_buttons = _buttons_named(ui,'新的旅程')
	_check(exit_buttons.size()==1 and save_buttons.size()==1 and new_buttons.size()==1,'disconnected actions retained')
	if exit_buttons.size()==1 and save_buttons.size()==1 and new_buttons.size()==1:
		# Disabled buttons are not pressed by the UI; emitting their signal directly
		# would bypass Button's own connected gating rather than test it.
		_check(not exit_buttons[0].disabled and save_buttons[0].disabled and new_buttons[0].disabled,'only exit enabled while disconnected')
		exit_buttons[0].pressed.emit()
		_check(events==[['close_client',null,owners[1]]],'disconnected exit emits original close intent')

	old_children = ui.box.get_children()
	ui.show_page('death',owners[2],snapshot,true,true)
	_free_controls(old_children)
	events.clear()
	exit_buttons = _buttons_named(ui,'退出游戏（不保存）')
	save_buttons = _buttons_named(ui,'从存档继续')
	new_buttons = _buttons_named(ui,'新的旅程')
	_check(exit_buttons.size()==1 and save_buttons.size()==1 and new_buttons.size()==1,'pending actions retained')
	if exit_buttons.size()==1 and save_buttons.size()==1 and new_buttons.size()==1:
		_check(exit_buttons[0].disabled and save_buttons[0].disabled and new_buttons[0].disabled,'pending disables all death buttons')
		exit_buttons[0].pressed.emit()
		_check(events.is_empty(),'pending callback emits nothing even if signalled directly')

	old_children = ui.box.get_children()
	ui.show_page('death',owners[3],snapshot,true)
	_free_controls(old_children)
	exit_buttons = _buttons_named(ui,'退出游戏（不保存）')
	_check(exit_buttons.size()==1,'current exit retained before closure')
	if exit_buttons.size()==1:
		ui.set_message('fixture pending',true)
		exit_buttons[0].pressed.emit()
		_check(events.is_empty(),'existing callback rejects pending state after creation')
		ui.set_message('',false)
		ui.close_page()
		exit_buttons[0].pressed.emit()
		_check(events.is_empty(),'closed page callback emits nothing')
	_check(snapshot==snapshot_before and owners==owners_before,'input snapshot and owner associations unchanged')
	_check(ui.slots==slots_before and ui.default_save_state=='available' and ui.slot_list_state=='list','save metadata unchanged')
	ui.free()
	print('DEATH_EXIT_CONTRACT checks='+str(checks)+' failures='+str(failures))
	quit(0 if failures==0 else 1)

func _owner(generation: int) -> Dictionary:
	return {'generation':8,'panelGeneration':generation,
		'context':{'worldId':'grey_hive','sceneId':'gh_entry_maintenance','worldEpoch':7}}

func _buttons_named(ui, text: String) -> Array:
	var result: Array = []
	for child in ui.box.get_children():
		if child is Button and child.text==text:
			result.append(child)
	return result

func _labels_named(ui, text: String) -> int:
	var count: int = 0
	for child in ui.box.get_children():
		if child is Label and child.text==text:
			count += 1
	return count

func _button_count(ui) -> int:
	var count: int = 0
	for child in ui.box.get_children():
		if child is Button:
			count += 1
	return count

func _free_controls(controls: Array) -> void:
	for control in controls:
		if is_instance_valid(control):
			control.free()

func _check(ok: bool, label: String) -> void:
	checks += 1
	if not ok:
		failures += 1
		push_error(label)
