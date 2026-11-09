extends SceneTree

const P = preload('res://scripts/bridge/protocol.gd')
const RpcClientScript = preload('res://scripts/bridge/rpc_client.gd')
const SceneCatalogScript = preload('res://scripts/world/scene_catalog.gd')
const RendererScript = preload('res://scripts/world/renderer.gd')
const HEART_SCENE_SHA256 = 'f16f3941767aed71023cd8b9ce75642345a103eabf5c77682e358c96c2e219d5'
const MAX_CAPTURES = 2

var _rpc: Node
var _renderer: Node2D
var _catalog: RefCounted
var _connection_state: String = 'pending'
var _responses: Dictionary = {}
var _failures: Dictionary = {}
var _last_frame_key: String = ''
var _frame_count: int = 0
var _capture_records: Array[Dictionary] = []
var _close_attempted: bool = false
var _failure_code: String = ''

func _initialize() -> void:
	call_deferred('_run')

func _run() -> void:
	var succeeded: bool = await _observe()
	var closed: bool = false
	if _rpc != null and _rpc.authenticated and not _close_attempted:
		closed = await _close_once()
	if _rpc != null and is_instance_valid(_rpc):
		_rpc.peer.disconnect_from_host()
	if succeeded and closed:
		print('HEART_FLOOR_OBSERVATION_COMPLETE captures='+str(_capture_records.size()))
		quit(0)
	else:
		print('HEART_FLOOR_OBSERVATION_STOP code='+(_failure_code if not _failure_code.is_empty() else 'CLOSE_NOT_CONFIRMED'))
		quit(1)

func _observe() -> bool:
	if not _verify_profile_binding():
		_failure_code = 'PROFILE_BINDING'
		return false
	var user_args: PackedStringArray = OS.get_cmdline_user_args()
	if user_args.size()!=4 or user_args[0]!='--v1-ready' or user_args[2]!='--v1-token':
		_failure_code = 'RPC_ARGUMENTS'
		return false
	var run_root: String = _normalized(OS.get_environment('HEART_OBS_RUN_ROOT'))
	var outputs: String = _normalized(OS.get_environment('HEART_OBS_OUTPUTS'))
	if not _contained(run_root,outputs) or outputs!=run_root.path_join('outputs'):
		_failure_code = 'OUTPUT_NAMESPACE'
		return false
	if FileAccess.file_exists(outputs.path_join('whole-floor.png')) or FileAccess.file_exists(outputs.path_join('entrance-joint.png')) or FileAccess.file_exists(outputs.path_join('context.json')):
		_failure_code = 'OUTPUT_COLLISION'
		return false
	_rpc = RpcClientScript.new()
	_rpc.connection_changed.connect(_on_connection_changed)
	_rpc.response.connect(_on_rpc_response)
	_rpc.failed.connect(_on_rpc_failed)
	get_root().add_child(_rpc)
	_rpc.start_from_arguments()
	if not await _wait_connected(12000):
		_failure_code = 'RPC_AUTHENTICATION'
		return false
	var continued: Dictionary = await _rpc_call('formal_continue',{},12000)
	if not P.success(continued) or continued.get('commandId')!='continue':
		_failure_code = 'CONTINUE_RECEIPT'
		return false
	var prepared: Variant = continued.get('snapshot')
	if not _valid_heart_snapshot(prepared,true):
		_failure_code = 'PREPARED_SNAPSHOT'
		return false
	_catalog = SceneCatalogScript.new()
	if not _catalog.build('clockworks','cw_furnace_heart') or _catalog.source_hash!=HEART_SCENE_SHA256 or _catalog.current.is_empty():
		_failure_code = 'CANONICAL_SCENE'
		return false
	if not P.snapshot(prepared) or prepared.get('entryToken')==null:
		_failure_code = 'PREPARED_CONTEXT'
		return false
	_renderer = RendererScript.new()
	_renderer.set_process(false)
	_renderer.set_physics_process(false)
	_renderer.frame_committed.connect(_on_frame_committed)
	get_root().add_child(_renderer)
	_renderer.commit(prepared,_catalog.current,_catalog.source_hash)
	if not await _bind_viewport(Vector2i(1920,1080),Vector3(12,0,8),42.0,prepared):
		_failure_code = 'PREPARED_FRAME'
		return false
	var token: Dictionary = prepared.entryToken.duplicate(true)
	if not P.token(token) or not P.same(token,prepared):
		_failure_code = 'ENTRY_TOKEN'
		return false
	var ready_result: Dictionary = await _rpc_call('formal_scene_ready',{'token':token,'remainPaused':true},12000)
	if not P.success(ready_result) or ready_result.get('commandId')!='scene-ready:'+str(token.generation):
		_failure_code = 'SCENE_READY_RECEIPT'
		return false
	var ready_snapshot: Variant = ready_result.get('snapshot')
	if not _valid_heart_snapshot(ready_snapshot,false) or ready_snapshot.worldEpoch!=prepared.worldEpoch:
		_failure_code = 'READY_SNAPSHOT'
		return false
	if not await _commit_and_wait(ready_snapshot):
		_failure_code = 'READY_FRAME'
		return false
	var status: Dictionary = await _rpc_call('bridge_session_status',{},8000)
	if not P.status(status) or not P.same(status,ready_snapshot) or status.get('paused')!=true or status.get('entryToken')!=null:
		_failure_code = 'READY_STATUS'
		return false
	if not await _capture('whole-floor.png',Vector2i(1920,1080),Vector3(12,0,8),42.0,ready_snapshot):
		_failure_code = 'WHOLE_FLOOR_CAPTURE'
		return false
	if not await _capture('entrance-joint.png',Vector2i(1280,800),Vector3(4,0,8),96.0,ready_snapshot):
		_failure_code = 'ENTRANCE_CAPTURE'
		return false
	if _capture_records.size()!=MAX_CAPTURES:
		_failure_code = 'CAPTURE_COUNT'
		return false
	return _write_context(run_root,outputs,ready_snapshot)

func _verify_profile_binding() -> bool:
	var raw_root: String = OS.get_environment('HEART_OBS_RUN_ROOT')
	var raw_expected: String = OS.get_environment('HEART_OBS_EXPECTED_USER_DIR')
	var manifest_digest: String = OS.get_environment('HEART_OBS_MANIFEST_SHA256')
	var run_id: String = OS.get_environment('HEART_OBS_RUN_ID')
	if raw_root.is_empty() or raw_expected.is_empty() or manifest_digest.length()!=64 or run_id.length()!=36:
		return false
	for character: String in manifest_digest:
		if not character in '0123456789abcdef':
			return false
	var root_path: String = _normalized(raw_root)
	var expected: String = _normalized(raw_expected)
	if root_path.is_empty() or expected!=root_path.path_join('profile/roaming/WuxianV1Campaign141'):
		return false
	if not _contained(root_path,expected):
		return false
	var actual_os: String = _normalized(OS.get_user_data_dir())
	var actual_user: String = _normalized(ProjectSettings.globalize_path('user://'))
	return actual_os==expected and actual_user==expected

func _valid_heart_snapshot(value: Variant, pending_entry: bool) -> bool:
	if not P.snapshot(value) or value.get('worldId')!='clockworks' or value.get('sceneId')!='cw_furnace_heart':
		return false
	if pending_entry:
		return P.token(value.get('entryToken')) and P.same(value.entryToken,value)
	return value.get('entryToken')==null

func _bind_viewport(requested: Vector2i, origin: Vector3, ppm: float, snapshot_value: Dictionary) -> bool:
	var root_window: Window = get_root()
	root_window.size = requested
	root_window.content_scale_size = requested
	var start: int = Time.get_ticks_msec()
	while Time.get_ticks_msec()-start<8000:
		await process_frame
		var actual: Rect2 = root_window.get_visible_rect()
		if actual.position==Vector2.ZERO and actual.size==Vector2(requested):
			var viewport: Rect2 = _renderer.get_viewport_rect()
			if not _renderer.is_visible_in_tree() or viewport!=actual:
				return false
			_renderer.viewport_rect = viewport
			_renderer.camera.viewport = viewport.size
			_renderer.camera.origin = origin
			_renderer.camera.screen_center = viewport.size*0.5
			_renderer.camera.ppm = ppm
			_renderer.camera.initialized = true
			var key: String = snapshot_value.worldId+':'+snapshot_value.sceneId+':'+str(snapshot_value.worldEpoch)
			var previous_count: int = _frame_count
			_renderer.queue_redraw()
			var draw_complete: bool = await _wait_for_frame(key,previous_count,10000)
			return draw_complete and P.same(_renderer.snapshot,snapshot_value)
	return false

func _commit_and_wait(snapshot_value: Dictionary) -> bool:
	var key: String = snapshot_value.worldId+':'+snapshot_value.sceneId+':'+str(snapshot_value.worldEpoch)
	_renderer.commit(snapshot_value,_catalog.current,_catalog.source_hash)
	var committed_frame: bool = await _bind_viewport(Vector2i(get_root().size),Vector3(12,0,8),42.0,snapshot_value)
	return committed_frame and _renderer.current_key==key and P.same(_renderer.snapshot,snapshot_value)

func _wait_for_frame(key: String, previous_count: int, timeout_ms: int) -> bool:
	var start: int = Time.get_ticks_msec()
	while _frame_count<=previous_count and Time.get_ticks_msec()-start<timeout_ms:
		await process_frame
	if _frame_count<=previous_count or _last_frame_key!=key:
		return false
	await RenderingServer.frame_post_draw
	return _last_frame_key==key and _renderer.current_key==key and P.snapshot(_renderer.snapshot)

func _capture(name: String, size: Vector2i, origin: Vector3, ppm: float, snapshot_value: Dictionary) -> bool:
	if _capture_records.size()>=MAX_CAPTURES:
		return false
	if not await _bind_viewport(size,origin,ppm,snapshot_value):
		return false
	var actual: Rect2 = get_root().get_visible_rect()
	if actual.position!=Vector2.ZERO or actual.size!=Vector2(size) or _renderer.get_viewport_rect()!=actual or _renderer.viewport_rect!=actual:
		return false
	await RenderingServer.frame_post_draw
	var image: Image = get_root().get_texture().get_image()
	if image.is_empty() or image.get_size()!=size:
		return false
	var outputs: String = _normalized(OS.get_environment('HEART_OBS_OUTPUTS'))
	var destination: String = outputs.path_join(name)
	if not _contained(outputs,destination) or FileAccess.file_exists(destination) or image.save_png(destination)!=OK:
		return false
	var bytes: PackedByteArray = FileAccess.get_file_as_bytes(destination)
	var digest: HashingContext = HashingContext.new()
	if bytes.is_empty() or digest.start(HashingContext.HASH_SHA256)!=OK or digest.update(bytes)!=OK:
		return false
	_capture_records.append({'name':name,'width':image.get_width(),'height':image.get_height(),'sha256':digest.finish().hex_encode(),'worldEpoch':snapshot_value.worldEpoch,'cameraOrigin':[origin.x,origin.y,origin.z],'ppm':ppm,'viewport':[actual.size.x,actual.size.y]})
	return true

func _write_context(run_root: String, outputs: String, snapshot_value: Dictionary) -> bool:
	var destination: String = outputs.path_join('context.json')
	if not _contained(run_root,destination) or FileAccess.file_exists(destination):
		return false
	var record: Dictionary = {'task':'EXEC-HEART-OWNED-OBSERVATION-FIXTURE-20261021-001/r1','worldId':'clockworks','sceneId':'cw_furnace_heart','worldEpoch':snapshot_value.worldEpoch,'sceneHash':_catalog.source_hash,'captureCount':_capture_records.size(),'captures':_capture_records.duplicate(true),'runtimeProfileVerified':true,'sourceStatus':'OBSERVED_ONLY'}
	var file: FileAccess = FileAccess.open(destination,FileAccess.WRITE)
	if file==null:
		return false
	file.store_string(JSON.stringify(record,'\t')+'\n')
	file.flush()
	file.close()
	return FileAccess.file_exists(destination)

func _close_once() -> bool:
	if _close_attempted or _rpc==null or not _rpc.authenticated:
		return false
	_close_attempted = true
	var result: Dictionary = await _rpc_call('session_close',{},8000)
	return typeof(result.get('closing'))==TYPE_BOOL and result.get('closing')==true and _rpc.closing_expected

func _rpc_call(method: String, params: Dictionary, timeout_ms: int) -> Dictionary:
	var request_id: int = _rpc.request(method,params,{})
	if request_id<=0:
		return {}
	var start: int = Time.get_ticks_msec()
	while not _responses.has(request_id) and not _failures.has(request_id) and Time.get_ticks_msec()-start<timeout_ms:
		await process_frame
	if _failures.has(request_id):
		_failures.erase(request_id)
		return {}
	if not _responses.has(request_id):
		return {}
	var response: Dictionary = _responses[request_id]
	_responses.erase(request_id)
	return response.get('method')==method and response.get('result') is Dictionary ? response.result : {}

func _wait_connected(timeout_ms: int) -> bool:
	var start: int = Time.get_ticks_msec()
	while _connection_state=='pending' and Time.get_ticks_msec()-start<timeout_ms:
		await process_frame
	return _connection_state=='connected' and _rpc.authenticated

func _on_connection_changed(connected: bool, code: String) -> void:
	_connection_state = 'connected' if connected else 'failed'
	if not connected and not code.is_empty():
		_failure_code = 'RPC_CONNECTION'

func _on_rpc_response(request_id: int, method: String, result: Variant, _association: Dictionary) -> void:
	_responses[request_id] = {'method':method,'result':result}

func _on_rpc_failed(request_id: int, _method: String, _code: String, _association: Dictionary) -> void:
	_failures[request_id] = true

func _on_frame_committed(key: String) -> void:
	_last_frame_key = key
	_frame_count += 1

func _normalized(path: String) -> String:
	return path.replace('\\','/').simplify_path().trim_suffix('/')

func _contained(root_path: String, candidate: String) -> bool:
	var root_value: String = _normalized(root_path).to_lower()
	var candidate_value: String = _normalized(candidate).to_lower()
	return candidate_value.begins_with(root_value+'/')
