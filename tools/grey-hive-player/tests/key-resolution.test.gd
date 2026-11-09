extends SceneTree

# Compile only the actual key functions, with inert action/bridge substitutes.
func extract_function(source: String, name: String) -> String:
	var start = source.find('func '+name+'(')
	assert(start >= 0, 'Missing function '+name)
	var end = source.find('\nfunc ', start+1)
	return source.substr(start) if end < 0 else source.substr(start, end-start)

func _initialize() -> void:
	var args = OS.get_cmdline_user_args()
	if args.size()!=1:
		printerr('Expected absolute v1_main.gd source argument')
		quit(1)
		return
	var source = FileAccess.get_file_as_string(args[0]).replace('\r\n','\n')
	var constants = ''
	for line in source.split('\n'):
		if line.begins_with('const TRACE_KEYS = '):
			constants = line+'\n'
	var input = extract_function(source,'_input')
	if constants.is_empty() or not input.contains('if not event.physical_keycode in TRACE_KEYS:'):
		printerr('Missing shared fallback classification')
		quit(1)
		return
	var script = GDScript.new()
	script.source_code = 'extends Node\n'+constants+'''
var logical_down = {}
var blocked_keys = {}
var focused = true
var mouse_guard_rearm = false
var mouse_move_rearm = false
var guard_blocked = false
const ACTION_KEYS = {}
func _trace_key(_event, _stage, _tag): pass
func _end_mouse_guard(): pass
func _clear_mouse_movement(): pass
func _end_guard(): pass
func _eligible(): return true
func _escape(): pass
'''+extract_function(source,'_event_key')+'\n'+extract_function(source,'_key_held')+'\n'+input
	if script.reload()!=OK:
		printerr('Isolated actual key functions did not compile')
		quit(1)
		return
	var subject = script.new()
	var cases = [
		['missing physical F',0,KEY_F,KEY_F],
		['unknown physical F',KEY_F1,KEY_F,KEY_F],
		['recognized physical priority',KEY_D,KEY_F,KEY_D],
		['unknown both',KEY_F1,KEY_F2,0],
		['Escape fallback',KEY_F1,KEY_ESCAPE,KEY_ESCAPE],
		['physical Escape priority',KEY_ESCAPE,KEY_F,KEY_ESCAPE]
	]
	var failures = 0
	for fixture in cases:
		var event = InputEventKey.new()
		event.physical_keycode = fixture[1]
		event.keycode = fixture[2]
		if subject._event_key(event)!=fixture[3]:
			printerr('FAIL '+fixture[0])
			failures += 1
		else:
			print('PASS '+fixture[0])
	# Exercise the actual fallback down/up path for movement and interaction.
	for key in [KEY_W,KEY_A,KEY_S,KEY_D,KEY_F]:
		var event = InputEventKey.new()
		event.physical_keycode = KEY_F1
		event.keycode = key
		event.pressed = true
		subject._input(event)
		if not subject._key_held(key): failures += 1
		event.pressed = false
		subject._input(event)
		if subject.logical_down.has(key): failures += 1
	print('Actual fallback WASD/F press-release exercised; failures='+str(failures))
	# Existing focus-loss clearing stays in the production notification handler.
	var notification = extract_function(source,'_notification')
	if not notification.contains('NOTIFICATION_APPLICATION_FOCUS_OUT') or not notification.contains('logical_down.clear()'):
		printerr('FAIL existing focus-loss clearing contract')
		failures += 1
	subject.free()
	quit(0 if failures==0 else 1)
