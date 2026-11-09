extends SceneTree

const P = preload('res://scripts/bridge/protocol.gd')
const OBS_FLOW: Array[Dictionary] = [
	{'state':'AUTH','event':'connected','next':'WAIT_CONTINUE','action':'continue'},
	{'state':'WAIT_CONTINUE','event':'continue_ok','next':'WAIT_FRAME','action':'commit'},
	{'state':'WAIT_FRAME','event':'prepared_frame','next':'WAIT_READY','action':'ready'},
	{'state':'WAIT_READY','event':'ready_ok','next':'WAIT_STATUS','action':'status'},
	{'state':'WAIT_STATUS','event':'status_ok','next':'CAPTURING','action':'capture'},
	{'state':'CAPTURING','event':'captures_done','next':'WAIT_CLOSE','action':'close'},
	{'state':'WAIT_CLOSE','event':'close_ok','next':'DONE','action':'quit'}
]
var state: String = 'AUTH'
var outcome: String = 'RUNNING'
var close_started: bool = false
var close_confirmed: bool = false
var sent: Dictionary = {}
var pending: Dictionary = {}
var association_sequence: int = 0
var rpc: Variant
var catalog: Variant
var renderer: Variant
var projection_script: Script
var prepared_snapshot: Dictionary = {}
var ready_snapshot: Dictionary = {}
var ready_status: Dictionary = {}
var prepared_token: Dictionary = {}
var prepared_key: String = ''
var manifest: Dictionary = {}
var manifest_path: String = ''
var outputs_saved: Dictionary = {}
func _initialize() -> void:
	if not _verify_profile():
		_fail('FAILED')
		return
	_load_production()
func _verify_profile() -> bool:
	manifest_path = OS.get_environment('WX_FOUNDRY_OBS_MANIFEST')
	var expected_hash: String = OS.get_environment('WX_FOUNDRY_OBS_MANIFEST_SHA256')
	if manifest_path.is_empty() or not manifest_path.is_absolute_path() or manifest_path.simplify_path() != manifest_path or expected_hash.length() != 64:
		return false
	for character: String in expected_hash:
		if not character in '0123456789abcdef':
			return false
	var file: FileAccess = FileAccess.open(manifest_path,FileAccess.READ)
	if file == null or file.get_length() <= 0 or file.get_length() > 262144:
		return false
	var bytes: PackedByteArray = file.get_buffer(file.get_length())
	file.close()
	if bytes.is_empty() or bytes.get_string_from_utf8().to_utf8_buffer() != bytes:
		return false
	var hash: HashingContext = HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256) != OK or hash.update(bytes) != OK or hash.finish().hex_encode() != expected_hash:
		return false
	var decoded: Variant = JSON.parse_string(bytes.get_string_from_utf8())
	if not decoded is Dictionary or decoded.get('schema') != 'wuxian.foundry-owned-observation/v1' or not decoded.get('paths') is Dictionary:
		return false
	manifest = decoded
	var roaming: String = str(manifest.paths.get('roaming','')).replace('\\','/')
	var local: String = str(manifest.paths.get('local','')).replace('\\','/')
	var temporary: String = str(manifest.paths.get('temp','')).replace('\\','/')
	var expected_user: String = roaming.path_join('WuxianV1Campaign141')
	var actual_user: String = OS.get_user_data_dir().replace('\\','/')
	var global_user: String = ProjectSettings.globalize_path('user://').replace('\\','/')
	return not roaming.is_empty() and not local.is_empty() and not temporary.is_empty() and OS.get_environment('APPDATA').replace('\\','/') == roaming and OS.get_environment('LOCALAPPDATA').replace('\\','/') == local and OS.get_environment('TEMP').replace('\\','/') == temporary and OS.get_environment('TMP').replace('\\','/') == temporary and actual_user == expected_user and global_user == expected_user
func _load_production() -> void:
	var rpc_source: Variant = load('res://scripts/bridge/rpc_client.gd')
	var catalog_source: Variant = load('res://scripts/world/scene_catalog.gd')
	var renderer_source: Variant = load('res://scripts/world/renderer.gd')
	projection_script = load('res://scripts/world/projection.gd')
	if rpc_source == null or catalog_source == null or renderer_source == null or projection_script == null:
		_fail('FAILED')
		return
	catalog = catalog_source.new()
	rpc = rpc_source.new()
	rpc.name = 'OwnedObservationRpc'
	root.add_child(rpc)
	rpc.response.connect(_on_response)
	rpc.failed.connect(_on_failed)
	rpc.connection_changed.connect(_on_connection_changed)
	renderer = renderer_source.new()
	renderer.name = 'World'
	root.add_child(renderer)
	renderer.frame_committed.connect(_on_frame)
	rpc.start_from_arguments()
func _on_connection_changed(connected: bool, _code: String) -> void:
	if connected:
		await _advance('connected')
	elif outcome == 'RUNNING' and not close_confirmed and not close_started:
		_fail('UNKNOWN')
	elif close_started and not close_confirmed:
		outcome = 'UNKNOWN'
		state = 'UNKNOWN'
		_quit_clean()
func _on_failed(_id: int, _method: String, _code: String, _association: Dictionary) -> void:
	_fail('UNKNOWN')
func _send_once(method: String, params: Dictionary, purpose: String) -> bool:
	if rpc == null or not is_instance_valid(rpc) or sent.has(method) or not rpc.authenticated or close_started and method != 'session_close':
		_fail('UNKNOWN')
		return false
	sent[method] = true
	association_sequence += 1
	var generation: int = rpc.connection_generation
	var association: Dictionary = {'purpose':'foundry_observation','step':purpose,'method':method,'sequence':association_sequence,'connectionGeneration':generation}
	var id: int = int(rpc.request(method,params,association))
	if id <= 0:
		_fail('UNKNOWN')
		return false
	pending[id] = {'id':id,'method':method,'association':association.duplicate(true),'generation':generation}
	return true
func _advance(event: String) -> void:
	var matches: Array[Dictionary] = []
	for row: Dictionary in OBS_FLOW:
		if row.state == state and row.event == event:
			matches.append(row)
	if matches.size() != 1:
		_fail('UNKNOWN')
		return
	var row: Dictionary = matches[0]
	state = row.next
	await _effect(row.action)
func _effect(action: String) -> void:
	match action:
		'continue':
			_continue_once()
		'commit':
			_commit_prepared()
		'ready':
			_ready_once()
		'status':
			_status_once()
		'capture':
			if await _capture_final():
				await _advance('captures_done')
			else:
				_fail('UNKNOWN')
		'close':
			_begin_close()
		'quit':
			_quit_clean()
		_:
			_fail('UNKNOWN')
func _continue_once() -> void:
	_send_once('formal_continue',{},'continue')
func _validate_receipt(method: String, result: Variant) -> bool:
	if not P.receipt(result) or not P.success(result) or not P.snapshot(result.snapshot):
		return false
	var snapshot: Dictionary = result.snapshot
	if snapshot.worldId != 'clockworks' or snapshot.sceneId != 'cw_entry_foundry':
		return false
	if method == 'formal_continue':
		return result.commandId == 'continue' and snapshot.get('entryToken') != null and P.token(snapshot.entryToken) and P.same(snapshot.entryToken,snapshot)
	return method == 'formal_scene_ready' and result.commandId is String and not result.commandId.is_empty() and snapshot.get('entryToken') == null and P.same(prepared_snapshot,snapshot) and snapshot.serverTick >= prepared_snapshot.serverTick and snapshot.authorityRevision >= prepared_snapshot.authorityRevision and snapshot.ackSeq >= prepared_snapshot.ackSeq and snapshot.player.currentHp > 0
func _validate_status(result: Variant) -> bool:
	if not P.status(result) or result.worldId != 'clockworks' or result.sceneId != 'cw_entry_foundry' or result.get('entryToken') != null or result.get('paused') != true:
		return false
	return P.counter(result.get('worldEpoch'),1) and result.worldEpoch == ready_snapshot.get('worldEpoch')
func _on_response(id: int, method: String, result: Variant, association: Dictionary) -> void:
	if not pending.has(id):
		_fail('UNKNOWN')
		return
	var expected: Dictionary = pending[id]
	pending.erase(id)
	if expected.id != id or method != expected.method or association != expected.association or rpc.connection_generation != expected.generation:
		_fail('UNKNOWN')
		return
	match method:
		'formal_continue':
			if not _validate_receipt(method,result):
				_fail('UNKNOWN')
				return
		prepared_snapshot = result.snapshot.duplicate(true)
		prepared_token = prepared_snapshot.entryToken.duplicate(true)
		prepared_key = _snapshot_key(prepared_snapshot)
		await _advance('continue_ok')
		'formal_scene_ready':
			if not _validate_receipt(method,result):
				_fail('UNKNOWN')
				return
		ready_snapshot = result.snapshot.duplicate(true)
		if ready_snapshot.get('entryToken') != null:
			_fail('UNKNOWN')
			return
		await _advance('ready_ok')
		'bridge_session_status':
			if not _validate_status(result):
				_fail('UNKNOWN')
				return
		ready_status = result.duplicate(true)
		await _advance('status_ok')
		'session_close':
			if not result is Dictionary or result.get('closing') != true:
				_fail('UNKNOWN')
				return
		close_confirmed = true
		if state == 'WAIT_CLOSE':
			await _advance('close_ok')
		else:
			_quit_clean()
		_:
			_fail('UNKNOWN')
func _snapshot_key(snapshot: Dictionary) -> String:
	return str(snapshot.worldId)+':'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch)
func _commit_snapshot(snapshot: Dictionary) -> bool:
	if not P.snapshot(snapshot) or snapshot.worldId != 'clockworks' or snapshot.sceneId != 'cw_entry_foundry' or not catalog.build(snapshot.worldId,snapshot.sceneId):
		return false
	var scene: Dictionary = catalog.current
	if scene.is_empty() or catalog.source_hash.length() != 64:
		return false
	renderer.commit(snapshot,scene,catalog.source_hash)
	return renderer.snapshot == snapshot and renderer.scene == scene and renderer.scene_hash == catalog.source_hash and renderer.current_key == _snapshot_key(snapshot)

func _commit_prepared() -> void:
	if prepared_snapshot.is_empty() or not P.token(prepared_token) or not P.same(prepared_token,prepared_snapshot) or prepared_snapshot.get('entryToken') != prepared_token or not _commit_snapshot(prepared_snapshot):
		_fail('UNKNOWN')

func _assert_prepared_binding() -> bool:
	if renderer == null or not is_instance_valid(renderer) or not P.snapshot(prepared_snapshot) or prepared_snapshot.get('entryToken') != prepared_token or not P.same(prepared_token,prepared_snapshot):
		return false
	if renderer.snapshot != prepared_snapshot or renderer.current_key != prepared_key or renderer.scene != catalog.current or renderer.scene_hash != catalog.source_hash:
		return false
	var camera: Variant = renderer.camera
	var actual: Rect2 = renderer.get_viewport_rect()
	return camera != null and is_instance_valid(camera) and camera.get_script() == projection_script and camera.initialized == true and camera.viewport == actual.size and renderer.get_viewport() == root and actual == renderer.viewport_rect

func _on_frame(key: String) -> void:
	if state != 'WAIT_FRAME':
		return
	if key != prepared_key or not _assert_prepared_binding():
		_fail('UNKNOWN')
		return
	await _advance('prepared_frame')

func _ready_once() -> void:
	if not P.token(prepared_token) or not P.same(prepared_token,prepared_snapshot):
		_fail('UNKNOWN')
		return
	_send_once('formal_scene_ready',{'token':prepared_token.duplicate(true),'remainPaused':true},'ready')

func _commit_final() -> bool:
	if ready_snapshot.is_empty() or ready_snapshot.get('entryToken') != null or not P.snapshot(ready_snapshot) or not P.same(prepared_snapshot,ready_snapshot):
		return false
	return _commit_snapshot(ready_snapshot)

func _status_once() -> void:
	if not _commit_final():
		_fail('UNKNOWN')
		return
	_send_once('bridge_session_status',{},'status')

func _bind_camera(recipe: Dictionary) -> bool:
	if renderer == null or not is_instance_valid(renderer) or root.size != recipe.size or renderer.get_viewport() != root or renderer.snapshot != ready_snapshot or renderer.current_key != _snapshot_key(ready_snapshot) or renderer.scene != catalog.current or renderer.scene_hash != catalog.source_hash:
		return false
	var camera: Variant = renderer.camera
	if camera == null or not is_instance_valid(camera) or camera.get_script() != projection_script:
		return false
	camera.origin = recipe.origin
	camera.viewport = Vector2(recipe.size)
	camera.screen_center = Vector2(recipe.size)*0.5
	camera.ppm = recipe.ppm
	camera.initialized = true
	renderer.viewport_rect = renderer.get_viewport_rect()
	return _assert_live_bindings(recipe)

func _assert_live_bindings(recipe: Dictionary) -> bool:
	if renderer == null or not is_instance_valid(renderer) or root.size != recipe.size or renderer.get_viewport() != root:
		return false
	var actual: Rect2 = renderer.get_viewport_rect()
	var camera: Variant = renderer.camera
	return actual == renderer.viewport_rect and actual.size == Vector2(recipe.size) and camera != null and is_instance_valid(camera) and camera.get_script() == projection_script and camera.initialized == true and camera.origin == recipe.origin and camera.viewport == actual.size and camera.screen_center == actual.size*0.5 and camera.ppm == recipe.ppm and renderer.snapshot == ready_snapshot and renderer.scene == catalog.current and renderer.current_key == _snapshot_key(ready_snapshot)

func _wait_final_draw() -> bool:
	if renderer == null or not is_instance_valid(renderer):
		return false
	renderer.queue_redraw()
	await RenderingServer.frame_post_draw
	return is_instance_valid(renderer)

func _save_frame_once(name: String, recipe: Dictionary) -> bool:
	if not _assert_live_bindings(recipe):
		return false
	var path: String = str(manifest.paths.outputs).path_join(name+'.png')
	if outputs_saved.has(path) or FileAccess.file_exists(path):
		return false
	outputs_saved[path] = true
	var texture: Texture2D = root.get_texture()
	if texture == null or not is_instance_valid(texture):
		return false
	var image: Image = texture.get_image()
	if image == null or image.is_empty() or image.get_width() != recipe.size.x or image.get_height() != recipe.size.y:
		return false
	return image.save_png(path) == OK

func _capture_final() -> bool:
	if ready_status.is_empty() or not _validate_status(ready_status) or not _commit_final():
		return false
	renderer.set_process(false)
	var recipes: Array[Dictionary] = [
		{'name':'whole-room','size':Vector2i(1920,1080),'origin':Vector3(12,0,8),'ppm':42.0},
		{'name':'north-east','size':Vector2i(1280,800),'origin':Vector3(24,0,3),'ppm':128.0},
		{'name':'south-east','size':Vector2i(1280,800),'origin':Vector3(24,0,15.75),'ppm':256.0},
		{'name':'south-west','size':Vector2i(1280,800),'origin':Vector3(0.25,0,15.5),'ppm':144.0}
	]
	for recipe: Dictionary in recipes:
		root.size = recipe.size
		await process_frame
		await process_frame
		if not _bind_camera(recipe):
			return false
		if not await _wait_final_draw():
			return false
		if not _assert_live_bindings(recipe) or not _save_frame_once(recipe.name,recipe):
			return false
	return outputs_saved.size() == 4

func _begin_close() -> void:
	if close_started:
		return
	close_started = true
	_close_once()

func _close_once() -> void:
	if rpc == null or not is_instance_valid(rpc) or not rpc.authenticated:
		outcome = 'UNKNOWN'
		_quit_clean()
		return
	_send_once('session_close',{},'close')

func _fail(result: String) -> void:
	if outcome == 'DONE' or outcome == 'FAILED' or outcome == 'UNKNOWN':
		return
	var was_closing: bool = close_started
	outcome = 'UNKNOWN' if result == 'UNKNOWN' else 'FAILED'
	state = outcome
	_begin_close()
	if was_closing and not close_confirmed:
		_quit_clean()

func _quit_clean() -> void:
	if renderer != null and is_instance_valid(renderer):
		renderer.queue_free()
	if rpc != null and is_instance_valid(rpc):
		rpc.queue_free()
	if outcome == 'RUNNING':
		outcome = 'DONE' if close_confirmed else 'UNKNOWN'
	quit(0 if outcome == 'DONE' and close_confirmed else 2)
