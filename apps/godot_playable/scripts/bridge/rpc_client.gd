extends Node
const P = preload('res://scripts/bridge/protocol.gd')
signal response(id, method, result, association)
signal failed(id, method, code, association)
signal connection_changed(connected, code)
const OUTER = 'wuxian.godot-v1.rpc'
const READS = ['hello','formal_snapshot','bridge_session_status','formal_presentation_events','formal_sound_cues','formal_has_save','formal_list_save_slots']
const METHODS = ['hello','formal_snapshot','formal_new','formal_submit_input','formal_submit_action','formal_interact','formal_scene_transition','formal_scene_checkpoint','formal_scene_trigger','formal_environment_control','formal_world_gate','formal_mission_terminal_status','formal_capability_terminal_status','formal_save_rest_terminal','formal_build_command','formal_capability_command','formal_choose_first_enhancement','formal_baizhi_begin','formal_baizhi_commit','formal_baizhi_close','formal_has_save','formal_continue','formal_save','formal_return','formal_list_save_slots','formal_save_slot','formal_return_slot','formal_continue_slot','formal_pause','formal_resume','formal_scene_ready','formal_presentation_events','formal_sound_cues','bridge_session_status','session_close']
var peer = StreamPeerTCP.new()
var authenticated = false
var connection_generation = 0
var next_id = 0
var pending = {}
var queue: Array = []
var queued_bytes = 0
var incoming = PackedByteArray()
var token_secret = ''
var connect_started = 0
var hello_sent = false
var closing_expected = false
var terminal_code = ''
var terminal_started = 0
var transport_stopped = false
func start_from_arguments() -> void:
	var args = OS.get_cmdline_user_args()
	var values = {}
	var n = 0
	while n < args.size():
		var key = args[n]
		if not key in ['--v1-ready','--v1-token'] or values.has(key) or n+1 >= args.size():
			connection_changed.emit(false,'E_CONNECTION_ARGUMENTS')
			return
		values[key] = args[n+1]
		n += 2
	if values.size()!=2 or not _valid_token(values['--v1-token']):
		connection_changed.emit(false,'E_CONNECTION_ARGUMENTS')
		return
	var locator: String = values['--v1-ready'].replace('\\','/')
	if not locator.is_absolute_path() or locator.simplify_path()!=locator or not locator.ends_with('/run/bridge-ready.json'):
		connection_changed.emit(false,'E_READY_LOCATOR')
		return
	var f = FileAccess.open(locator,FileAccess.READ)
	if f == null or f.get_length() > 16384 or f.get_length() == 0:
		connection_changed.emit(false,'E_READY_UNAVAILABLE')
		return
	var raw = f.get_buffer(f.get_length())
	f.close()
	if raw.get_string_from_utf8().to_utf8_buffer()!=raw:
		connection_changed.emit(false,'E_READY_ENCODING')
		return
	var ready = JSON.parse_string(raw.get_string_from_utf8())
	if not ready is Dictionary or ready.get('schema')!='wuxian.godot-v1.ready/v1' or not P.counter(ready.get('protocolVersion'),1,1) or ready.get('host')!='127.0.0.1' or not P.counter(ready.get('port'),1,65535) or not P.counter(ready.get('processId'),1,4294967295) or ready.get('sessionToken')!=values['--v1-token']:
		connection_changed.emit(false,'E_READY_IDENTITY')
		return
	var session = locator.get_base_dir().get_base_dir()
	if typeof(ready.get('sessionDir'))!=TYPE_STRING or typeof(ready.get('saveDir'))!=TYPE_STRING or ready.sessionDir.replace('\\','/') != session or ready.saveDir.replace('\\','/') != session+'/saves':
		connection_changed.emit(false,'E_READY_NAMESPACE')
		return
	# Live process/artifact/ordinary namespace provenance belongs to the future owned caller.
	token_secret = values['--v1-token']
	closing_expected = false
	terminal_code = ''
	terminal_started = 0
	transport_stopped = false
	connection_generation += 1
	connect_started = Time.get_ticks_msec()
	var err = peer.connect_to_host('127.0.0.1',int(ready.port))
	if err != OK:
		_drop('E_CONNECT_FAILED')
func _valid_token(v: String) -> bool:
	if v.length()<32 or v.length()>128:
		return false
	for c in v:
		if not c in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-':
			return false
	return true
func request(method: String, params: Dictionary, association: Dictionary = {}) -> int:
	if transport_stopped or not terminal_code.is_empty() or closing_expected or not method in METHODS or (not authenticated and method!='hello') or peer.get_status()!=StreamPeerTCP.STATUS_CONNECTED or next_id>=9007199254740991:
		return 0
	# Read work is coalesced while pending; mutations always preserve their original id/order.
	if method in READS:
		for item in pending.values():
			if item.method==method:
				return 0
	next_id += 1
	var bytes = (JSON.stringify({'protocol':OUTER,'protocolVersion':1,'id':next_id,'method':method,'params':params})+'\n').to_utf8_buffer()
	if bytes.size()>65536 or queued_bytes+bytes.size()>4194304 or pending.size()>=128:
		_drop('E_TRANSPORT_LIMIT')
		return 0
	pending[next_id] = {'method':method,'association':association.duplicate(true),'started':Time.get_ticks_msec(),'generation':connection_generation}
	queue.append({'id':next_id,'bytes':bytes,'offset':0})
	queued_bytes += bytes.size()
	return next_id
func _terminal(code: String) -> void:
	if terminal_code.is_empty():
		terminal_code = code
		terminal_started = Time.get_ticks_msec()
func _process(_delta: float) -> void:
	if transport_stopped:
		return
	peer.poll()
	var state = peer.get_status()
	if state==StreamPeerTCP.STATUS_CONNECTING:
		if Time.get_ticks_msec()-connect_started>6000:
			_drop('E_CONNECT_TIMEOUT')
		return
	if state!=StreamPeerTCP.STATUS_CONNECTED:
		if not hello_sent and not authenticated and connect_started==0:
			return
		_terminal('E_CONNECTION_LOST')
	if terminal_code.is_empty():
		if not hello_sent:
			hello_sent = true
			request('hello',{'sessionToken':token_secret,'clientProtocolVersion':1})
			token_secret = ''
		var work = 0
		while not queue.is_empty() and work<65536:
			var item: Dictionary = queue[0]
			var data: PackedByteArray = item.bytes.slice(item.offset,mini(item.bytes.size(),item.offset+16384))
			var sent = peer.put_partial_data(data)
			if sent[0]!=OK:
				_terminal('E_SEND_FAILED')
				break
			if sent[1]==0:
				break
			item.offset += sent[1]
			queued_bytes -= sent[1]
			work += sent[1]
			if item.offset==item.bytes.size():
				queue.pop_front()
	# The API may still expose bytes at EOF. Never infer bytes it does not expose.
	var available = peer.get_available_bytes()
	if available>0:
		var got = peer.get_partial_data(mini(available,65536))
		incoming.append_array(got[1])
		if got[0]!=OK:
			_terminal('E_READ_FAILED')
			available = 0
	if incoming.size()>2097152:
		_drop('E_RESPONSE_LIMIT')
		return
	var lines = 0
	while lines<16:
		var at = incoming.find(10)
		if at<0:
			break
		if at>2097152:
			_drop('E_RESPONSE_LIMIT')
			return
		var raw = incoming.slice(0,at)
		incoming = incoming.slice(at+1)
		if raw.get_string_from_utf8().to_utf8_buffer()!=raw:
			_drop('E_RESPONSE_ENCODING')
			return
		_handle(JSON.parse_string(raw.get_string_from_utf8()))
		lines += 1
		if transport_stopped:
			return
	# Terminal work is spread across bounded frames; pending ownership survives the drain.
	if peer.get_status()!=StreamPeerTCP.STATUS_CONNECTED:
		_terminal('E_CONNECTION_LOST')
	if not terminal_code.is_empty():
		if Time.get_ticks_msec()-terminal_started>15000:
			_drop('E_RESPONSE_TIMEOUT')
		elif incoming.find(10)>=0 or peer.get_available_bytes()>0:
			return
		else:
			_drop(terminal_code if incoming.is_empty() else 'E_RESPONSE_TRUNCATED')
		return
	for id in pending.keys():
		if Time.get_ticks_msec()-pending[id].started>15000:
			_drop('E_RESPONSE_TIMEOUT')
			return
func _handle(v) -> void:
	if not v is Dictionary or v.get('protocol')!=OUTER or not P.counter(v.get('protocolVersion'),1,1) or not P.counter(v.get('id'),1) or typeof(v.get('ok'))!=TYPE_BOOL or not pending.has(int(v.id)) or not v.has('result') or not v.has('error'):
		_drop('E_RESPONSE_PROTOCOL')
		return
	var id = int(v.id)
	var item: Dictionary = pending[id]
	if Time.get_ticks_msec()-item.started>15000:
		_drop('E_RESPONSE_TIMEOUT')
		return
	if item.generation!=connection_generation or item.method=='session_close' and queue.any(func(row): return row.id==id):
		_drop('E_RESPONSE_PROTOCOL')
		return
	pending.erase(id)
	if not v.ok:
		if not v.error is Dictionary or typeof(v.error.get('code'))!=TYPE_STRING or v.error.code.length()>128:
			_drop('E_RESPONSE_PROTOCOL')
			return
		if item.method=='hello':
			_drop('E_HANDSHAKE_REJECTED')
			return
		failed.emit(id,item.method,v.error.code,item.association)
		return
	if v.error!=null:
		_drop('E_RESPONSE_PROTOCOL')
		return
	if item.method=='hello':
		if not v.result is Dictionary or not P.counter(v.result.get('protocolVersion'),1,1) or v.result.get('service')!='wuxian-v1-formal' or not P.counter(v.result.get('snapshotProtocolVersion'),3,3):
			_drop('E_HANDSHAKE_VERSION')
			return
		authenticated = true
		connection_changed.emit(true,'')
	else:
		if item.method=='session_close' and v.result is Dictionary and typeof(v.result.get('closing'))==TYPE_BOOL and v.result.closing:
			closing_expected = true
		response.emit(id,item.method,v.result,item.association)
func _drop(code: String) -> void:
	if transport_stopped:
		return
	transport_stopped = true
	peer.disconnect_from_host()
	authenticated = false
	connect_started = 0
	token_secret = ''
	for id in pending.keys():
		var item: Dictionary = pending[id]
		failed.emit(id,item.method,code if item.method in READS else 'UNKNOWN_MUTATION',item.association)
	pending.clear()
	queue.clear()
	queued_bytes = 0
	incoming.clear()
	if hello_sent:
		hello_sent = false
	connection_changed.emit(false,'NORMAL_CLOSE_REQUESTED' if closing_expected else code)
