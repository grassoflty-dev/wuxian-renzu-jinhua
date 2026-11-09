extends SceneTree
const P = preload("res://scripts/bridge/protocol.gd")
const GH_INTERFACE_JSON = """
{"schema":"wuxian.gh-first-playable.interface/v2","manifestSchema":"wuxian.gh-first-playable.manifest/v2","phaseConfigSchema":"wuxian.gh-first-playable.phase-config/v1","evidenceSchema":"wuxian.gh-first-playable.evidence/v1","driver":"operator","ordinaryCheckout":"C:/Users/GWL/Desktop/wuxian-renzu-jinhua","phaseOrder":["new","continue"],"mainScene":"res://scenes/main.tscn","adapter":"res://tests/campaign/grey_hive_first_playable.gd","sessionRelative":"session","readyRelative":"session/run/bridge-ready.json","saveRelative":"session/saves/formal-save-v6.json","rpcArguments":["--v1-ready","--v1-token"],"environment":["WUXIAN_GH_CONFIG","WUXIAN_GH_CONFIG_SHA256","WUXIAN_GH_PHASE","WUXIAN_GH_EXPECTED_USER_DIR"],"runRequires":["runtime","sourceBaseline","resourcePurposes","profileBinding","operator","ownedFallback"],"requiredEvents":["hive_power","hive_lockdown","hive_extraction"],"baizhiChoices":["taken","left","unresolved"],"route":["rs_core_room","gh_entry_maintenance","gh_power_room","gh_gate_a","gh_central_shaft","gh_lockdown","gh_bio_isolation","gh_gate_b","gh_deep_decon","gh_sentinel_arena","gh_beacon","gh_exit"],"returnScene":"rs_core_room","returnGate":"gh_extraction_return_to_rs","sentinelExit":"gh_sentinel_arena_to_beacon","captureBinding":["snapshot","scene","current_key","scene_hash","camera","viewport_rect"],"manifestKeys":["schema","runId","driver","authority","sourceHead","source","artifacts","resources","optionalAssets","namespace","lease","operator","limits","acceptance","unresolved","deadlineUtc"],"requiredSourcePins":["apps/godot_playable/scripts/ui/v1_main.gd","apps/godot_playable/scripts/ui/panels.gd","apps/godot_playable/scripts/world/renderer.gd","apps/godot_playable/tests/campaign/grey_hive_first_playable.gd","tools/grey-hive-first-playable/run-owned.ps1","tools/grey-hive-first-playable/contract.test.mjs"],"sourceKeys":["checkoutRoot","head","indexEmpty","indexSha256","indexReference","protectedStateReference","pins"],"artifactKeys":["path","bytes","sha256","provenance","version"],"authorityRowKeys":["state","reference"],"resourceKeys":["path","bytes","sha256","presence","required","consumer","readPhase","purposeReference","qualificationState","distinction"],"optionalAssetKeys":["path","presence","bytes","sha256","purposeReference","sourceMapSha256"],"namespaceKeys":["base","runRoot"],"leaseKeys":["id","owner","firstUtc","reserveUtc","hardUtc","maxPairs","fallbackScope"],"operatorKeys":["reference","available","baizhiChoice"],"limitsKeys":["startupSeconds","phaseSeconds","closeSeconds","drainSeconds","totalSeconds","stdoutBytes","stderrBytes","captureCap","captureList"],"captureKeys":["id","phase","sceneId"],"acceptanceKeys":["route","requiredEvents","baizhiChoice","saveFields","comparisonFields","evidence"],"evidenceRequirementKeys":["phase","kind","outcome"],"phaseConfigKeys":["schema","phase","manifestSha256","manifest","sessionDir","saveDir","expectedUserDir","sessionTokenSha256"],"eventKeys":["schema","runId","phase","sequence","utc","monotonicMs","kind","outcome","manifestSha256","sourceBaselineReference","payload"],"eventTransport":"phase-output-file","eventFileName":"events.ndjson","maxConfigBytes":262144,"maxEventLineBytes":1048576,"maxEventFileBytes":134217728,"maxEventCount":50000,"maxCaptureCount":64,"terminalKind":"session_close","terminalOutcome":"CONFIRMED","terminalPayloadKeys":["authenticated","association","closing","connectionGeneration"],"acceptedKind":"run_complete","acceptedOutcome":"ACCEPTED"}
"""
const GH_ROUTE_JSON = """
{"visits":["rs_core_room","gh_entry_maintenance","gh_power_room","gh_gate_a","gh_central_shaft","gh_lockdown","gh_bio_isolation","gh_gate_b","gh_deep_decon","gh_sentinel_arena","gh_beacon","gh_exit"],"requiredEvents":["hive_power","hive_lockdown","hive_extraction"],"baizhiChoices":["taken","left","unresolved"],"returnScene":"rs_core_room","returnGate":"gh_extraction_return_to_rs","sentinelExit":"gh_sentinel_arena_to_beacon","captureBinding":["snapshot","scene","current_key","scene_hash","camera","viewport_rect"]}
"""
const EVENT_SCHEMA = "wuxian.gh-first-playable.evidence/v1"
const ROOT_BASE = "C:/Users/GWL/.codex/visualizations/2026/10/21/01a147f2-ef43-7cd3-9785-e0263fd4e60e/GH-FIRST-PLAYABLE-20261021-001/r1"
const MAX_CONFIG_BYTES = 262144
const MAX_PHASE_A_BYTES = 67108864
const MAX_EVENT_LINE_BYTES = 1048576
const MAX_EVENT_FILE_BYTES = 134217728
const MAX_EVENT_COUNT = 50000
const READ_METHODS = [ 	"hello", "formal_snapshot", "bridge_session_status", 	"formal_presentation_events", "formal_sound_cues", 	"formal_has_save", "formal_list_save_slots" ]
var _iface: Dictionary = {}
var _route: Dictionary = {}
var _config: Dictionary = {}
var _main: Node
var _rpc: Node
var _panels: Node
var _renderer: Node
var _phase = ""
var _run_id = ""
var _manifest_sha256 = ""
var _source_reference = ""
var _run_root = ""
var _ready_path = ""
var _save_path = ""
var _outputs_dir = ""
var _events_path = ""
var _expected_user_dir = ""
var _attached = false
var _stopped = false
var _live_diagnostic_logged = false
var _pending_live_diagnostic: Dictionary = {}
var _initial_viewport_validated = false
var _stop_outcome = ""
var _close_requested = false
var _phase_started_ms = 0
var _event_sequence = 0
var _event_count = 0
var _event_bytes = 0
var _journey_request_count = 0
var _journey_method = ""
var _journey_confirmed = false
var _entry_token_seen: Dictionary = {}
var _entry_ready: Dictionary = {}
var _ready_contexts: Dictionary = {}
var _mutation_ledger: Dictionary = {}
var _route_index = 0
var _route_seen: Dictionary = {}
var _last_ready_scene = ""
var _sentinel_defeat_confirmed = false
var _baizhi_intent = ""
var _baizhi_choice_confirmed = false
var _extraction_confirmed = false
var _return_gate_confirmed = false
var _returned_to_station = false
var _save_confirmed = false
var _save_request_count = 0
var _phase_a_summary: Dictionary = {}
var _comparison_fields: Array = []
var _capture_plan: Array = []
var _capture_queue: Array = []
var _capture_pending: Dictionary = {}
var _capture_done: Dictionary = {}
var _capture_running = false
var _input_receipt_count = 0
var _action_receipt_count = 0
var _combat_receipt_count = 0
var _last_snapshot_signature = ""
var _last_status_signature = ""
var _last_frame_key = ""
func _initialize() -> void:
	call_deferred("_boot_main")
func _boot_main() -> void:
	if not _read_config():
		push_error("Grey Hive adapter refused its reviewed phase configuration.")
		quit(64)
		return
	if not _verify_profile():
		push_error("Grey Hive adapter refused an unbound child profile.")
		quit(64)
		return
	var packed: Variant = load(_iface.mainScene)
	if not packed is PackedScene:
		push_error("Grey Hive production scene is unavailable.")
		quit(64)
		return
	var instance: Variant = packed.instantiate()
	if not instance is Node:
		push_error("Grey Hive production scene could not be instantiated.")
		quit(64)
		return
	_main = instance
	root.add_child(instance)
	await process_frame
	if not _attach_main():
		push_error("Grey Hive production observers could not be attached.")
		quit(64)
		return
	_phase_started_ms = Time.get_ticks_msec()
	if not _write_event("adapter_started", "OBSERVED", { 		"driver": _iface.driver, 		"mainScene": _iface.mainScene, 		"phase": _phase, 		"runId": _run_id 	}): _stop_unknown("evidence_sink_unavailable")
func _read_config() -> bool:
	var parsed_interface: Variant = JSON.parse_string(GH_INTERFACE_JSON)
	var parsed_route: Variant = JSON.parse_string(GH_ROUTE_JSON)
	if not parsed_interface is Dictionary or not parsed_route is Dictionary: return false
	_iface = parsed_interface
	_route = parsed_route
	if _iface.get("schema") != "wuxian.gh-first-playable.interface/v2": return false
	if _iface.get("manifestSchema") != "wuxian.gh-first-playable.manifest/v2": return false
	if _iface.get("phaseConfigSchema") != "wuxian.gh-first-playable.phase-config/v1": return false
	if _iface.get("evidenceSchema") != EVENT_SCHEMA or _iface.get("eventTransport") != "phase-output-file": return false
	if int(_iface.get("maxConfigBytes", 0)) != MAX_CONFIG_BYTES: return false
	if int(_iface.get("maxEventLineBytes", 0)) != MAX_EVENT_LINE_BYTES: return false
	if int(_iface.get("maxEventFileBytes", 0)) != MAX_EVENT_FILE_BYTES or int(_iface.get("maxEventCount", 0)) != MAX_EVENT_COUNT: return false
	if _iface.get("terminalKind") != "session_close" or _iface.get("terminalOutcome") != "CONFIRMED": return false
	if _iface.get("acceptedKind") != "run_complete" or _iface.get("acceptedOutcome") != "ACCEPTED": return false
	if _iface.get("terminalPayloadKeys") != ["authenticated", "association", "closing", "connectionGeneration"]: return false
	if _iface.get("driver") != "operator": return false
	if _iface.get("phaseOrder") != ["new", "continue"]: return false
	if _iface.get("mainScene") != "res://scenes/main.tscn": return false
	if _iface.get("adapter") != "res://tests/campaign/grey_hive_first_playable.gd": return false
	if _iface.get("rpcArguments") != ["--v1-ready", "--v1-token"]: return false
	if _iface.get("runRequires") is not Array or _iface.runRequires.size() != 6: return false
	if _route.get("visits") != _iface.get("route") or _route.get("requiredEvents") != _iface.get("requiredEvents"): return false
	if _route.get("baizhiChoices") != _iface.get("baizhiChoices") or _route.get("returnScene") != _iface.get("returnScene"): return false
	if _route.get("returnGate") != _iface.get("returnGate") or _route.get("sentinelExit") != _iface.get("sentinelExit"): return false
	if _route.get("captureBinding") != _iface.get("captureBinding") or _route.visits.size() != 12: return false
	var env_config = OS.get_environment("WUXIAN_GH_CONFIG").replace("\\", "/")
	var env_hash = OS.get_environment("WUXIAN_GH_CONFIG_SHA256")
	_phase = OS.get_environment("WUXIAN_GH_PHASE")
	_expected_user_dir = OS.get_environment("WUXIAN_GH_EXPECTED_USER_DIR").replace("\\", "/")
	if not _absolute_path(env_config) or not _hex_digest(env_hash): return false
	if not _absolute_path(_expected_user_dir) or not _phase in _iface.phaseOrder: return false
	var args = OS.get_cmdline_user_args()
	if args.size() != 4: return false
	var launch: Dictionary = {}
	for n in range(0, args.size(), 2):
		var key: String = args[n]
		if not key in _iface.rpcArguments or launch.has(key): return false
		launch[key] = args[n + 1]
	if launch.size() != 2 or not launch.has("--v1-ready") or not launch.has("--v1-token"): return false
	if not _valid_session_token(str(launch["--v1-token"])): return false
	_ready_path = str(launch["--v1-ready"]).replace("\\", "/")
	if not _absolute_path(_ready_path): return false
	var file := FileAccess.open(env_config, FileAccess.READ)
	if file == null: return false
	var length := file.get_length()
	if length <= 0 or length > int(_iface.maxConfigBytes):
		file.close()
		return false
	var raw := file.get_buffer(length)
	file.close()
	var text := raw.get_string_from_utf8()
	if text.to_utf8_buffer() != raw: return false
	var digest := HashingContext.new()
	if digest.start(HashingContext.HASH_SHA256) != OK: return false
	digest.update(raw)
	if digest.finish().hex_encode() != env_hash: return false
	var parser := JSON.new()
	if parser.parse(text) != OK or not parser.data is Dictionary: return false
	var phase_config: Dictionary = parser.data
	if not _exact_keys(phase_config, _iface.phaseConfigKeys): return false
	if phase_config.get("schema") != _iface.phaseConfigSchema or phase_config.get("phase") != _phase: return false
	if not _hex_digest(phase_config.get("manifestSha256")) or not phase_config.get("manifest") is Dictionary: return false
	_config = phase_config.manifest
	if not _manifest_gate(): return false
	_run_id = str(_config.runId)
	_manifest_sha256 = str(phase_config.manifestSha256)
	_source_reference = str(_config.authority.sourceBaseline.reference)
	var namespace_value: Dictionary = _config.get("namespace", {})
	_run_root = str(namespace_value.get("runRoot", "")).replace("\\", "/")
	if not _absolute_path(_run_root) or _run_root != ROOT_BASE + "/" + _run_id: return false
	if namespace_value.get("base", "") != ROOT_BASE: return false
	var expected_session := _run_root.path_join(_iface.sessionRelative)
	var expected_save_dir := _run_root.path_join(_iface.saveRelative.get_base_dir())
	if not _same_path(str(phase_config.get("sessionDir", "")).replace("\\", "/"), expected_session): return false
	if not _same_path(str(phase_config.get("saveDir", "")).replace("\\", "/"), expected_save_dir): return false
	if not _same_path(str(phase_config.get("expectedUserDir", "")).replace("\\", "/"), _expected_user_dir): return false
	var token_digest := HashingContext.new()
	if token_digest.start(HashingContext.HASH_SHA256) != OK: return false
	token_digest.update(str(launch["--v1-token"]).to_utf8_buffer())
	if token_digest.finish().hex_encode() != str(phase_config.get("sessionTokenSha256", "")): return false
	var expected_ready := _run_root.path_join(_iface.readyRelative)
	var expected_save := _run_root.path_join(_iface.saveRelative)
	if _ready_path != expected_ready: return false
	_save_path = expected_save
	_outputs_dir = _run_root.path_join("phases").path_join(_phase).path_join("outputs")
	_events_path = _outputs_dir.path_join(_iface.eventFileName)
	if _expected_user_dir != str(_config.get("expectedUserDir", _expected_user_dir)).replace("\\", "/"): return false
	var profile_root := _run_root.path_join("phases").path_join(_phase).path_join("profile").path_join("roaming")
	if not _expected_user_dir.begins_with(profile_root + "/") and _expected_user_dir != profile_root: return false
	if not _absolute_path(_outputs_dir) or not DirAccess.dir_exists_absolute(_outputs_dir): return false
	if FileAccess.file_exists(_events_path): return false
	if not _prepare_acceptance(): return false
	if _phase == "continue":
		_phase_a_summary = _load_phase_a_summary()
		if _phase_a_summary.is_empty(): return false
	return true
func _exact_keys(value: Variant, expected: Array) -> bool:
	if not value is Dictionary or expected.is_empty() or value.size() != expected.size(): return false
	for key in expected:
		if typeof(key) != TYPE_STRING or not value.has(key): return false
	return true
func _manifest_gate() -> bool:
	if not _exact_keys(_config, _iface.manifestKeys): return false
	if _config.get("schema") != _iface.manifestSchema or _config.get("driver") != _iface.driver: return false
	if not _canonical_uuid(_config.get("runId")) or not _hex_oid(_config.get("sourceHead")): return false
	var authority: Variant = _config.get("authority")
	if not authority is Dictionary or authority.size() != _iface.runRequires.size(): return false
	for requirement in _iface.runRequires:
		var item: Variant = authority.get(requirement)
		if not _exact_keys(item, _iface.authorityRowKeys) or item.get("state") != "QUALIFIED": return false
		if typeof(item.get("reference")) != TYPE_STRING or item.reference.is_empty(): return false
	var unresolved: Variant = _config.get("unresolved")
	if not unresolved is Array or not unresolved.is_empty(): return false
	var source: Variant = _config.get("source")
	if not _exact_keys(source, _iface.sourceKeys) or source.get("checkoutRoot") != _iface.ordinaryCheckout: return false
	if source.get("head") != _config.sourceHead or source.get("indexEmpty") != true or not _hex_digest(source.get("indexSha256")): return false
	if typeof(source.get("indexReference")) != TYPE_STRING or source.indexReference.is_empty(): return false
	if typeof(source.get("protectedStateReference")) != TYPE_STRING or source.protectedStateReference.is_empty(): return false
	if not source.get("pins") is Array or source.pins.size() < _iface.requiredSourcePins.size(): return false
	var pin_paths: Dictionary = {}
	for pin in source.pins:
		if not _exact_keys(pin, ["path", "bytes", "sha256"]): return false
		if typeof(pin.path) != TYPE_STRING or pin.path.is_empty() or pin.path.begins_with("/") or ".." in pin.path.split("/"): return false
		if not _positive_integer(pin.bytes) or not _hex_digest(pin.sha256) or pin_paths.has(pin.path): return false
		pin_paths[pin.path] = true
	for required_path in _iface.requiredSourcePins:
		if not pin_paths.has(required_path): return false
	var artifacts: Variant = _config.get("artifacts")
	if not artifacts is Dictionary or artifacts.size() != 2: return false
	for artifact_name in ["native", "godot"]:
		var artifact: Variant = artifacts.get(artifact_name)
		if not _exact_keys(artifact, _iface.artifactKeys) or not _absolute_path(str(artifact.get("path", ""))): return false
		if not _positive_integer(artifact.get("bytes")) or not _hex_digest(artifact.get("sha256")): return false
		if typeof(artifact.get("provenance")) != TYPE_STRING or artifact.provenance.is_empty(): return false
		if artifact.version != null and typeof(artifact.version) != TYPE_STRING: return false
	var resources: Variant = _config.get("resources")
	if not resources is Array or resources.is_empty(): return false
	var resource_paths: Dictionary = {}
	for resource in resources:
		if not _exact_keys(resource, _iface.resourceKeys) or typeof(resource.get("path")) != TYPE_STRING: return false
		if resource.path.is_empty() or not resource.path.begins_with("res://") or ".." in resource.path or "\\" in resource.path: return false
		if resource_paths.has(resource.path) or typeof(resource.required) != TYPE_BOOL: return false
		if typeof(resource.consumer) != TYPE_STRING or resource.consumer.is_empty() or typeof(resource.readPhase) != TYPE_STRING or resource.readPhase.is_empty(): return false
		if typeof(resource.qualificationState) != TYPE_STRING or not resource.qualificationState in ["QUALIFIED", "UNRESOLVED"]: return false
		if typeof(resource.distinction) != TYPE_STRING or not resource.distinction in ["display", "integrity", "incidental"]: return false
		if typeof(resource.presence) != TYPE_STRING: return false
		if resource.presence == "present":
			if not _positive_integer(resource.bytes) or not _hex_digest(resource.sha256) or typeof(resource.purposeReference) != TYPE_STRING or resource.purposeReference.is_empty(): return false
			if resource.qualificationState != "QUALIFIED": return false
		elif resource.presence == "absent":
			if resource.required or resource.bytes != 0 or resource.sha256 != null or resource.purposeReference != null: return false
		else: return false
		resource_paths[resource.path] = resource
	var optional_assets: Variant = _config.get("optionalAssets")
	if not optional_assets is Array: return false
	var optional_paths: Dictionary = {}
	for asset in optional_assets:
		if not _exact_keys(asset, _iface.optionalAssetKeys) or typeof(asset.path) != TYPE_STRING or optional_paths.has(asset.path): return false
		if not asset.path.begins_with("res://") or ".." in asset.path or "\\" in asset.path: return false
		if not _hex_digest(asset.sourceMapSha256) or not asset.presence in ["present", "absent"]: return false
		if asset.presence == "present":
			if not _positive_integer(asset.bytes) or not _hex_digest(asset.sha256) or typeof(asset.purposeReference) != TYPE_STRING or asset.purposeReference.is_empty(): return false
			var bound: Variant = resource_paths.get(asset.path)
			if not bound is Dictionary or bound.presence != "present" or bound.bytes != asset.bytes or bound.sha256 != asset.sha256 or bound.purposeReference != asset.purposeReference: return false
		elif asset.bytes != 0 or asset.sha256 != null or asset.purposeReference != null or resource_paths.has(asset.path): return false
		optional_paths[asset.path] = true
	var namespace_value: Variant = _config.get("namespace")
	if not _exact_keys(namespace_value, _iface.namespaceKeys) or namespace_value.base != ROOT_BASE or namespace_value.runRoot != ROOT_BASE.path_join(_config.runId): return false
	var lease: Variant = _config.get("lease")
	if not _exact_keys(lease, _iface.leaseKeys) or typeof(lease.get("id")) != TYPE_STRING or lease.id.is_empty(): return false
	if lease.get("maxPairs") != 2 or typeof(lease.get("owner")) != TYPE_STRING or lease.owner.is_empty(): return false
	for name in ["firstUtc", "reserveUtc", "hardUtc"]:
		if typeof(lease.get(name)) != TYPE_STRING or lease[name].is_empty(): return false
	if typeof(lease.get("fallbackScope")) != TYPE_STRING or lease.fallbackScope.is_empty(): return false
	if _config.deadlineUtc != lease.hardUtc: return false
	var operator: Variant = _config.get("operator")
	if not _exact_keys(operator, _iface.operatorKeys) or typeof(operator.get("reference")) != TYPE_STRING or operator.reference.is_empty(): return false
	if operator.available != true:
		return false
	if not operator.get("baizhiChoice") in _iface.baizhiChoices: return false
	var limits: Variant = _config.get("limits")
	if not _exact_keys(limits, _iface.limitsKeys): return false
	for field in ["startupSeconds", "phaseSeconds", "closeSeconds", "drainSeconds", "totalSeconds"]:
		if not _positive_limit(limits, [field]): return false
	if not _positive_integer(limits.get("stdoutBytes")) or limits.stdoutBytes > 1048576: return false
	if not _positive_integer(limits.get("stderrBytes")) or limits.stderrBytes > 1048576: return false
	if not _positive_integer(limits.get("captureCap")) or limits.captureCap > int(_iface.maxCaptureCount): return false
	if not limits.get("captureList") is Array or limits.captureList.is_empty() or limits.captureList.size() > limits.captureCap: return false
	var capture_ids: Dictionary = {}
	for capture in limits.captureList:
		if not _exact_keys(capture, _iface.captureKeys) or not _safe_label(str(capture.get("id", ""))): return false
		if not capture.phase in _iface.phaseOrder or not capture.sceneId in _route.visits: return false
		var capture_key := str(capture.phase) + ":" + str(capture.id)
		if capture_ids.has(capture_key): return false
		capture_ids[capture_key] = true
	var acceptance: Variant = _config.get("acceptance")
	if not _exact_keys(acceptance, _iface.acceptanceKeys) or acceptance.route != _route.visits: return false
	if acceptance.requiredEvents != _iface.requiredEvents or acceptance.baizhiChoice != operator.baizhiChoice: return false
	if not acceptance.baizhiChoice in _iface.baizhiChoices: return false
	for field in ["saveFields", "comparisonFields"]:
		var values: Variant = acceptance.get(field)
		if not values is Array or values.is_empty(): return false
		var unique_values: Dictionary = {}
		for value in values:
			if typeof(value) != TYPE_STRING or not _valid_field_path(value) or unique_values.has(value): return false
			unique_values[value] = true
	if not acceptance.evidence is Array or acceptance.evidence.is_empty(): return false
	for requirement in acceptance.evidence:
		if not _exact_keys(requirement, _iface.evidenceRequirementKeys): return false
		if not requirement.phase in _iface.phaseOrder or typeof(requirement.kind) != TYPE_STRING or requirement.kind.is_empty() or typeof(requirement.outcome) != TYPE_STRING or requirement.outcome.is_empty(): return false
	return true
func _prepare_acceptance() -> bool:
	var acceptance: Variant = _config.get("acceptance")
	var operator: Dictionary = _config.get("operator", {})
	var limits: Dictionary = _config.get("limits", {})
	if not acceptance is Dictionary: return false
	if acceptance.get("requiredEvents") != _iface.requiredEvents: return false
	if acceptance.get("baizhiChoice") != operator.get("baizhiChoice"): return false
	_comparison_fields = acceptance.get("comparisonFields", [])
	if not _comparison_fields is Array or _comparison_fields.is_empty() or _comparison_fields.size() > 64: return false
	for path in _comparison_fields:
		if typeof(path) != TYPE_STRING or not _valid_field_path(path): return false
	var captures: Variant = limits.get("captureList", [])
	if not captures is Array or captures.size() > int(limits.get("captureCap", 0)): return false
	var expected = [
		"new:new-rs:rs_core_room", "new:gh-entry:gh_entry_maintenance",
		"new:sentinel-encounter:gh_sentinel_arena", "new:gh-exit-extraction:gh_exit",
		"new:returned-rs-saved:rs_core_room", "continue:continue-rs:rs_core_room"
	]
	var actual: Array = []
	var ids: Dictionary = {}
	for row in captures:
		if not _exact_keys(row, _iface.captureKeys): return false
		var phase_value: Variant = row.get("phase")
		var scene_value: Variant = row.get("sceneId")
		var label: Variant = row.get("id")
		if not phase_value in _iface.phaseOrder or not scene_value in _route.visits: return false
		if typeof(label) != TYPE_STRING or not _safe_label(label): return false
		var capture_key: String = str(phase_value) + ":" + label
		if ids.has(capture_key): return false
		ids[capture_key] = true
		actual.append(str(phase_value) + ":" + str(label) + ":" + str(scene_value))
		if phase_value == _phase: _capture_plan.append({"id": label, "sceneId": scene_value})
	if limits.captureCap != 6 or actual != expected: return false
	var evidence: Variant = acceptance.get("evidence")
	if not evidence is Array or evidence.is_empty(): return false
	for requirement in evidence:
		if not _exact_keys(requirement, _iface.evidenceRequirementKeys): return false
		if not requirement.phase in _iface.phaseOrder or typeof(requirement.kind) != TYPE_STRING or requirement.kind.is_empty(): return false
		if typeof(requirement.outcome) != TYPE_STRING or requirement.outcome.is_empty(): return false
	return true
func _verify_profile() -> bool:
	var user_dir := OS.get_user_data_dir().replace("\\", "/")
	var global_user_dir := ProjectSettings.globalize_path("user://").replace("\\", "/").trim_suffix("/")
	if not _absolute_path(user_dir) or not _absolute_path(global_user_dir): return false
	return _same_path(user_dir, _expected_user_dir) and _same_path(global_user_dir, _expected_user_dir)
func _attach_main() -> bool:
	if not is_instance_valid(_main) or not _main.has_method("_submit_close"): return false
	_rpc = _main.get("rpc")
	_panels = _main.get("panels")
	_renderer = _main.get("renderer")
	if not is_instance_valid(_rpc) or not is_instance_valid(_panels) or not is_instance_valid(_renderer): return false
	if not _rpc.has_signal("response") or not _rpc.has_signal("failed") or not _rpc.has_signal("connection_changed"): return false
	if not _panels.has_signal("intent") or not _renderer.has_signal("frame_committed"): return false
	_rpc.connect("response", Callable(self, "_on_rpc_response"))
	_rpc.connect("failed", Callable(self, "_on_rpc_failed"))
	_rpc.connect("connection_changed", Callable(self, "_on_connection_changed"))
	_panels.connect("intent", Callable(self, "_on_intent"))
	_renderer.connect("frame_committed", Callable(self, "_on_frame_committed"))
	_attached = true
	return true
func _process(_delta: float) -> bool:
	if not _attached or _stopped: return false
	if Time.get_ticks_msec() - _phase_started_ms > _phase_limit_ms():
		_fail_known("phase_deadline")
		return false
	_observe_pending()
	if _stopped: return false
	_observe_snapshot()
	if _stopped: return false
	_observe_status()
	if _stopped: return false
	if bool(_main.get("uncertain")): _stop_unknown("main_uncertain")
	return false
func _observe_pending() -> void:
	var pending: Variant = _rpc.get("pending")
	if not pending is Dictionary:
		_stop_unknown("pending_table_invalid")
		return
	for id_value in pending.keys():
		var item: Variant = pending[id_value]
		if not item is Dictionary or typeof(item.get("method")) != TYPE_STRING:
			_stop_unknown("pending_record_invalid")
			return
		var method: String = item.method
		if method in READ_METHODS: continue
		var id := int(id_value)
		if not P.counter(id, 1):
			_stop_unknown("pending_id_invalid")
			return
		var association: Variant = item.get("association", {})
		if not association is Dictionary:
			_stop_unknown("pending_association_invalid")
			return
		_register_mutation(id, method, association)
		if _stopped: return
	for key in _mutation_ledger.keys():
		var id := int(key)
		if not pending.has(id):
			_stop_unknown("mutation_terminal_missing")
			return
		if _mutation_ledger[key].get("connectionGeneration") != int(_rpc.get("connection_generation")):
			_stop_unknown("mutation_connection_changed")
			return
func _register_mutation(id: int, method: String, association: Dictionary) -> void:
	var key := str(id)
	if _mutation_ledger.has(key): return
	var connection := int(_rpc.get("connection_generation"))
	_mutation_ledger[key] = { 		"method": method, 		"association": association.duplicate(true), 		"connectionGeneration": connection, 		"state": "PENDING" 	}
	if method in ["formal_new", "formal_continue", "formal_continue_slot"]:
		_journey_request_count += 1
		if _journey_request_count != 1:
			_fail_known("duplicate_entry_request")
			return
		if method == "formal_continue" and _journey_method != "continue_default_intent":
			_fail_known("continue_without_default_intent")
			return
		_journey_method = method
		if method == "formal_new" and _phase != "new": _fail_known("wrong_phase_new")
		elif method == "formal_continue" and _phase != "continue": _fail_known("wrong_phase_continue")
		elif method == "formal_continue_slot": _fail_known("non_default_continue")
	elif method == "formal_save":
		_save_request_count += 1
		if _save_request_count != 1 or _phase != "new" or association.get("purpose") != "save" or association.get("exitAfter") != false: _fail_known("out_of_scope_save")
	elif method in ["formal_save_slot", "formal_continue_slot"]: _fail_known("slot_operation_out_of_scope")
	elif method == "session_close":
		if _phase == "new" and (not _save_confirmed or not _close_requested or not _captures_complete()): _fail_known("close_before_saved_frame_acceptance")
		if _phase == "continue" and not _close_requested: _fail_known("operator_closed_continue_early")
func _on_rpc_response(id: int, method: String, result: Variant, association: Dictionary) -> void:
	_record_receipt(id, method, result, association, false, "")
func _on_rpc_failed(id: int, method: String, code: String, association: Dictionary) -> void:
	_record_receipt(id, method, null, association, true, code)
func _on_connection_changed(connected: bool, code: String) -> void:
	_write_event("connection", "CONNECTED" if connected else "DISCONNECTED", { 		"connected": connected, 		"code": code, 		"connectionGeneration": int(_rpc.get("connection_generation")) 	})
	if not connected and _journey_confirmed: _stop_unknown("native_connection_lost")
func _on_intent(kind: String, value: Variant, page_owner: Dictionary) -> void:
	var safe_value: Variant = null
	if typeof(value) == TYPE_BOOL or typeof(value) == TYPE_INT or typeof(value) == TYPE_FLOAT: safe_value = value
	elif typeof(value) == TYPE_STRING and (value in _iface.baizhiChoices or value in ["new", "continue_default", "save_exit", "close_client"]): safe_value = value
	var context_value: Variant = page_owner.get("context", {})
	var safe_owner := { 		"generation": page_owner.get("generation", -1), 		"panelGeneration": page_owner.get("panelGeneration", -1), 		"context": context_value.duplicate(true) if context_value is Dictionary else {} 	}
	_write_event("ui_intent", "OBSERVED", { 		"kind": kind, 		"value": safe_value, 		"owner": safe_owner 	})
	if kind == "continue_default":
		if _phase != "continue" or not _main.has_method("_default_continue_available") or not _main.call("_default_continue_available"): _fail_known("default_continue_unavailable_or_wrong_phase")
		elif _journey_method == "continue_default_intent": _fail_known("duplicate_default_continue_intent")
		else: _journey_method = "continue_default_intent"
	elif kind in ["continue_slot", "save_slot", "save_slot_new", "return_slot"]: _fail_known("slot_operation_out_of_scope")
	elif kind in ["save", "save_default"]:
		if _phase != "new" or _save_confirmed: _fail_known("save_out_of_scope")
	elif kind == "save_exit": _fail_known("save_exit_precedes_post_save_frame")
	elif kind == "new" and _phase != "new": _fail_known("new_journey_in_continue_phase")
	elif kind == "baizhi_confirm":
		if value in ["taken", "left"]:
			_baizhi_intent = str(value)
			if _baizhi_intent != str(_config.operator.baizhiChoice): _fail_known("baizhi_choice_differs_from_manifest")
	elif kind == "baizhi_unresolved":
		_baizhi_intent = "unresolved"
		if _baizhi_intent != str(_config.operator.baizhiChoice): _fail_known("baizhi_choice_differs_from_manifest")
	elif kind == "baizhi_commit":
		if typeof(value) != TYPE_STRING or value != _config.operator.baizhiChoice: _fail_known("baizhi_commit_differs_from_manifest")
	elif kind == "close_client" and not _close_requested: _fail_known("operator_closed_before_acceptance")
func _record_receipt(id: int, method: String, result: Variant, association: Dictionary, failed: bool, failure_code: String) -> void:
	if method in READ_METHODS:
		if method == "formal_presentation_events" and _has_native_events(result):
			_write_event("native_presentation_events", "OBSERVED", { 				"requestId": id, 				"events": result 			})
		elif method == "formal_sound_cues" and _has_native_events(result): _write_event("native_sound_events", "OBSERVED", { 				"requestId": id, 				"events": result 			})
		return
	if not _mutation_ledger.has(str(id)): _register_mutation(id, method, association)
	if method == "session_close":
		var close_record: Dictionary = _mutation_ledger.get(str(id), {}).duplicate(true)
		_mutation_ledger.erase(str(id))
		if failed:
			_write_event("session_close_receipt", "UNKNOWN" if failure_code == "UNKNOWN_MUTATION" else "FAILED", { 				"requestId": id, 				"failureCode": failure_code, 				"association": _safe_association(association) 			})
			if failure_code == "UNKNOWN_MUTATION": _stop_unknown("session_close_unknown")
			else: _fail_known("session_close_failed")
			return
		_record_close_if_valid(result, association, int(close_record.get("connectionGeneration", -1)))
		return
	if _stopped:
		_mutation_ledger.erase(str(id))
		return
	var record: Dictionary = _mutation_ledger.get(str(id), {})
	record["state"] = "TERMINAL"
	_mutation_ledger.erase(str(id))
	var native_receipt: Variant = result
	if result is Dictionary and result.has("receipt"): native_receipt = result.get("receipt")
	var receipt_valid := not failed and P.receipt(native_receipt)
	var accepted := receipt_valid and P.success(native_receipt)
	var outcome := "UNKNOWN" if failed and failure_code == "UNKNOWN_MUTATION" else "FAILED" if failed else "CONFIRMED" if accepted else "REJECTED" if receipt_valid else "UNKNOWN"
	var payload := { 		"requestId": id, 		"method": method, 		"outcome": outcome, 		"association": _safe_association(association), 		"result": result if not failed else null, 		"failureCode": failure_code if failed else "" 	}
	if method in ["formal_new", "formal_continue", "formal_scene_ready", "formal_save", "session_close"]: _write_event("native_receipt", outcome, payload)
	elif method == "formal_submit_input":
		_input_receipt_count += 1
		if accepted and (_input_receipt_count == 1 or _input_receipt_count % 50 == 0):
			payload["receiptOrdinal"] = _input_receipt_count
			_write_event("native_control_receipt", "CONFIRMED", payload)
	elif method == "formal_submit_action":
		_action_receipt_count += 1
		if accepted: _combat_receipt_count += 1
		_write_event("native_action_receipt", outcome, payload)
	else: _write_event("native_receipt", outcome, payload)
	if failed:
		if failure_code == "UNKNOWN_MUTATION": _stop_unknown("native_unknown_mutation")
		elif method in ["formal_new", "formal_continue", "formal_scene_ready", "formal_save", "session_close"]: _fail_known("required_mutation_rejected")
		return
	if not receipt_valid:
		_stop_unknown("native_receipt_invalid")
		return
	if not accepted:
		if method in ["formal_new", "formal_continue", "formal_scene_ready", "formal_save", "session_close"]: _fail_known("required_mutation_rejected")
		return
	match method:
		"formal_new":
			if _phase != "new" or association.get("purpose") != "journey" or native_receipt.get("commandId") != "new":
				_fail_known("new_receipt_identity")
				return
			_journey_confirmed = true
			if native_receipt.get("applied") != true or native_receipt.get("alreadyApplied") != false:
				_fail_known("new_not_fresh")
				return
			_observe_snapshot()
		"formal_continue":
			if _phase != "continue" or association.get("purpose") != "journey" or native_receipt.get("commandId") != "continue":
				_fail_known("continue_receipt_identity")
				return
			_journey_confirmed = true
			_journey_method = method
			if not _compare_with_phase_a(native_receipt.snapshot):
				_fail_known("continue_comparison_mismatch")
				return
			_write_event("continue_comparison", "MATCHED", { 				"comparison": _comparison_values(native_receipt.snapshot) 			})
			_observe_snapshot()
		"formal_scene_ready":
			_record_scene_ready(native_receipt, association)
		"formal_baizhi_commit":
			if _baizhi_intent in ["taken", "left"] and _choice_matches(native_receipt.snapshot, _baizhi_intent):
				_baizhi_choice_confirmed = true
				_write_event("baizhi_choice", "CONFIRMED", {"choice": _baizhi_intent})
			else: _fail_known("baizhi_choice_not_authoritatively_confirmed")
		"formal_baizhi_close":
			if _baizhi_intent == "unresolved" and _choice_matches(native_receipt.snapshot, "unresolved"):
				_baizhi_choice_confirmed = true
				_write_event("baizhi_choice", "CONFIRMED", {"choice": "unresolved"})
			elif _baizhi_intent == "unresolved": _fail_known("unresolved_choice_not_authoritatively_confirmed")
		"formal_interact":
			_record_extraction_if_valid(native_receipt, association)
		"formal_world_gate":
			_record_return_gate_if_valid(native_receipt, association)
		"formal_save":
			_record_save_if_valid(native_receipt, association)
		"session_close":
			_record_close_if_valid(result, association, int(record.get("connectionGeneration", -1)))
		_:
			pass
func _record_scene_ready(receipt: Dictionary, association: Dictionary) -> void:
	var token_value: Variant = association.get("token")
	if not token_value is Dictionary or not P.token(token_value):
		_fail_known("scene_ready_token_missing")
		return
	var key := _token_key(token_value)
	if not _entry_token_seen.has(key) or _entry_ready.has(key):
		_fail_known("scene_ready_token_replayed_or_unobserved")
		return
	if receipt.get("commandId") != "scene-ready:" + str(int(token_value.generation)):
		_fail_known("scene_ready_command_mismatch")
		return
	var ready_snapshot: Variant = receipt.get("snapshot", {})
	if not ready_snapshot is Dictionary or ready_snapshot.get("entryToken") != null or not P.same(ready_snapshot, token_value):
		_fail_known("scene_ready_receipt_context")
		return
	_entry_ready[key] = true
	var context_key := _context_key(ready_snapshot)
	if context_key.is_empty():
		_fail_known("scene_ready_context_invalid")
		return
	_ready_contexts[context_key] = true
	if _phase == "continue" and not _compare_with_phase_a(ready_snapshot):
		_fail_known("continue_ready_comparison_mismatch")
		return
	_advance_route(ready_snapshot)
	if _stopped: return
	_write_event("scene_ready", "CONFIRMED", { 		"token": _token_identity(token_value), 		"snapshot": ready_snapshot.duplicate(true) 	})
	_queue_captures_for_scene(str(ready_snapshot.get("sceneId", "")))
	_maybe_finish_phase()
func _observe_snapshot() -> void:
	if not is_instance_valid(_main): return
	var snapshot: Variant = _main.get("snapshot")
	if not snapshot is Dictionary or snapshot.is_empty(): return
	if not P.snapshot(snapshot):
		_stop_unknown("live_snapshot_invalid")
		return
	if not _validate_live(true):
		var status_value: Variant = _main.get("status")
		if status_value is Dictionary and not status_value.is_empty() and P.status(status_value) and not P.same(status_value, snapshot): return
		if _wait_initial_viewport(): return
		_emit_live_diagnostic("observe_snapshot")
		_stop_unknown("live_context_mismatch")
		return
	_initial_viewport_validated = true
	if snapshot.player.get("currentHp", 0) <= 0:
		_fail_known("player_death")
		return
	_observe_entry_token(snapshot.get("entryToken"), snapshot)
	if _stopped: return
	_advance_route(snapshot)
	if _stopped: return
	if snapshot.get("sceneId") == "gh_sentinel_arena" and _sentinel_defeated(snapshot) and not _sentinel_defeat_confirmed:
		_sentinel_defeat_confirmed = true
		_write_event("sentinel_defeat", "AUTHORITATIVE_STATE_CONFIRMED", { 			"snapshot": snapshot.duplicate(true), 			"actorId": "gh_sentinel_arena_sentinel_01" 		})
	var signature := _snapshot_signature(snapshot)
	if signature != _last_snapshot_signature:
		_last_snapshot_signature = signature
		_write_event("native_snapshot", "VALIDATED", { 			"snapshot": snapshot.duplicate(true), 			"currentKey": _renderer.get("current_key"), 			"sceneHash": _renderer.get("scene_hash") 		})
func _observe_status() -> void:
	if not is_instance_valid(_main): return
	var status_value: Variant = _main.get("status")
	if not status_value is Dictionary or status_value.is_empty(): return
	if not P.status(status_value):
		_stop_unknown("live_status_invalid")
		return
	var snapshot: Variant = _main.get("snapshot")
	if not snapshot is Dictionary or snapshot.is_empty() or not P.snapshot(snapshot): return
	if not P.same(status_value, snapshot): return
	if not _validate_live(true):
		if _wait_initial_viewport(): return
		_stop_unknown("status_context_mismatch")
		return
	_initial_viewport_validated = true
	var token_value: Variant = status_value.get("entryToken")
	if token_value != null:
		_observe_entry_token(token_value, snapshot)
		if _stopped: return
	var signature := str([ 		status_value.get("worldId"), 		status_value.get("sceneId"), 		status_value.get("worldEpoch"), 		status_value.get("paused"), 		status_value.get("lastLifecycleCommandSequence"), 		_token_key(token_value) if token_value is Dictionary else "" 	])
	if signature != _last_status_signature:
		_last_status_signature = signature
		_write_event("native_status", "VALIDATED", {"status": status_value.duplicate(true)})
func _observe_entry_token(token_value: Variant, snapshot: Dictionary) -> void:
	if token_value == null: return
	if not token_value is Dictionary or not P.token(token_value) or not P.same(token_value, snapshot):
		_stop_unknown("entry_token_identity")
		return
	var key := _token_key(token_value)
	if key.is_empty():
		_stop_unknown("entry_token_key")
		return
	if not _entry_token_seen.has(key):
		_entry_token_seen[key] = _token_identity(token_value)
		_write_event("entry_token", "OBSERVED", { 			"token": _token_identity(token_value), 			"snapshot": snapshot.duplicate(true) 		})
func _advance_route(snapshot: Dictionary) -> void:
	if _phase != "new" or not _journey_confirmed: return
	var context_key := _context_key(snapshot)
	if context_key.is_empty() or not _ready_contexts.has(context_key): return
	if _route_seen.has(context_key): return
	var world_id := str(snapshot.get("worldId", ""))
	var scene_id := str(snapshot.get("sceneId", ""))
	var is_route_scene: bool = scene_id in _route.visits
	if world_id == "mist_harbor" or world_id == "clockworks":
		_fail_known("out_of_scope_world")
		return
	if world_id == "return_station" and scene_id == _route.returnScene:
		if _route_index == 0 and not _return_gate_confirmed:
			_route_seen[context_key] = true
			_route_index = 1
			_last_ready_scene = scene_id
			_write_event("route_visit", "FIRST_SCENE_READY", { 				"sceneId": scene_id, 				"worldId": world_id, 				"routeIndex": 0, 				"snapshot": snapshot.duplicate(true) 			})
			return
		if _route_index == _route.visits.size() and _return_gate_confirmed and _extraction_confirmed and _hive_settled(snapshot):
			_route_seen[context_key] = true
			_returned_to_station = true
			_last_ready_scene = scene_id
			_write_event("route_complete", "CONFIRMED", { 				"snapshot": snapshot.duplicate(true), 				"routeIndex": _route_index, 				"visits": _route.visits.duplicate() 			})
			return
		_fail_known("unexpected_return_station_visit")
		return
	if world_id != "grey_hive" or not is_route_scene:
		_fail_known("out_of_scope_scene")
		return
	if scene_id == "gh_beacon" and _last_ready_scene == "gh_sentinel_arena" and not _sentinel_defeat_confirmed:
		_fail_known("sentinel_exit_without_authoritative_defeat")
		return
	var expected_scene := str(_route.visits[_route_index]) if _route_index < _route.visits.size() else ""
	if scene_id == expected_scene:
		if scene_id == "gh_beacon" and not _sentinel_defeat_confirmed:
			_fail_known("sentinel_not_defeated_before_beacon")
			return
		_route_seen[context_key] = true
		var accepted_index: int = _route_index
		_route_index += 1
		_last_ready_scene = scene_id
		_write_event("route_visit", "FIRST_SCENE_READY", { 			"sceneId": scene_id, 			"worldId": world_id, 			"routeIndex": accepted_index, 			"snapshot": snapshot.duplicate(true) 		})
	elif _route_index >= _route.visits.size():
		_route_seen[context_key] = true
		_last_ready_scene = scene_id
		_write_event("route_backtrack", "OBSERVED", { 			"sceneId": scene_id, 			"routeIndex": _route_index 		})
	else:
		_route_seen[context_key] = true
		_last_ready_scene = scene_id
		_write_event("route_out_of_order", "NOT_COUNTED", { 			"sceneId": scene_id, 			"expected": expected_scene, 			"routeIndex": _route_index 		})
func _sentinel_defeated(snapshot: Dictionary) -> bool:
	if snapshot.get("worldId") != "grey_hive" or snapshot.get("sceneId") != "gh_sentinel_arena": return false
	var encounter: Variant = snapshot.get("sentinelEncounter")
	if not encounter is Dictionary or encounter.size() != 6: return false
	if encounter.get("actorId") != "gh_sentinel_arena_sentinel_01": return false
	if not encounter.get("phase") in ["dead", "inactive"] or encounter.get("remainingMs") != 0: return false
	if not P.counter(encounter.get("attackSerial")) or not P.point(encounter.get("positionM")): return false
	var matches: Array = []
	for actor in snapshot.get("actors", []):
		if actor is Dictionary and (actor.get("entityId") == "gh_sentinel_arena_sentinel_01" or actor.get("entityType") == "enemy.grey_hive.sentinel"): matches.append(actor)
	if matches.size() != 1: return false
	var actor: Dictionary = matches[0]
	if actor.get("entityId") != encounter.actorId or actor.get("entityType") != "enemy.grey_hive.sentinel": return false
	if actor.get("actorKind") != "sentinel" or actor.get("active") != false: return false
	if not P.transform(actor.get("transform")): return false
	return P.vector(encounter.positionM).distance_to(P.vector(actor.transform.positionM)) <= 0.001
func _hive_progress(snapshot: Dictionary) -> Dictionary:
	var progression: Variant = snapshot.get("progression")
	if not progression is Dictionary or not progression.get("worlds") is Array: return {}
	var rows: Array = []
	for row in progression.worlds:
		if row is Dictionary and row.get("worldId") == "grey_hive": rows.append(row)
	if rows.size() != 1: return {}
	var row: Dictionary = rows[0]
	if typeof(row.get("completed")) != TYPE_BOOL or typeof(row.get("firstCompletion")) != TYPE_BOOL: return {}
	if not row.get("completedEvents") is Array: return {}
	var seen: Dictionary = {}
	for event in row.completedEvents:
		if typeof(event) != TYPE_STRING or not event in _iface.requiredEvents or seen.has(event): return {}
		seen[event] = true
	return row
func _hive_settled(snapshot: Dictionary) -> bool:
	var progress := _hive_progress(snapshot)
	if progress.is_empty() or progress.get("completed") != true or progress.get("firstCompletion") != true: return false
	if progress.get("revisitCount") != 0 or progress.get("visitId") != 1: return false
	if progress.completedEvents.size() != _iface.requiredEvents.size(): return false
	for event in _iface.requiredEvents:
		if not event in progress.completedEvents: return false
	return true
func _choice_matches(snapshot: Dictionary, choice: String) -> bool:
	var progress := _hive_progress(snapshot)
	var progress_choice = progress.get("hiveChoice") if not progress.is_empty() else null
	var baizhi: Variant = snapshot.get("baizhi")
	var snapshot_choice = baizhi.get("choice") if baizhi is Dictionary else null
	if choice == "unresolved": return (progress_choice == "unresolved" or snapshot_choice == "unresolved") and (progress_choice == null or snapshot_choice == null or progress_choice == snapshot_choice)
	return (progress_choice == choice or snapshot_choice == choice) and (progress_choice == null or snapshot_choice == null or progress_choice == snapshot_choice)
func _record_extraction_if_valid(receipt: Dictionary, association: Dictionary) -> void:
	var target: Variant = association.get("target")
	var before: Variant = association.get("before")
	if not target is Dictionary or not before is Dictionary: return
	if target.get("kind") != "extraction_console" or before.get("worldId") != "grey_hive" or before.get("sceneId") != "gh_exit": return
	var old_beacon: Variant = before.get("greyHiveBeacon")
	var new_beacon: Variant = receipt.snapshot.get("greyHiveBeacon")
	if not old_beacon is Dictionary or not new_beacon is Dictionary:
		_fail_known("extraction_beacon_state_missing")
		return
	if old_beacon.get("state") != "mounted" or old_beacon.get("legacyCompletedWithoutReceipt") != false:
		_fail_known("extraction_without_mounted_beacon")
		return
	if new_beacon.get("state") != "mounted" or new_beacon.get("legacyCompletedWithoutReceipt") != false:
		_fail_known("extraction_beacon_postcondition")
		return
	var old_progress := _hive_progress(before)
	var new_progress := _hive_progress(receipt.snapshot)
	if old_progress.is_empty() or new_progress.is_empty():
		_fail_known("extraction_progress_invalid")
		return
	if old_progress.completedEvents.has("hive_extraction") or not new_progress.completedEvents.has("hive_extraction"):
		_fail_known("extraction_event_not_new")
		return
	if int(receipt.snapshot.progression.eventSeq) != int(before.progression.eventSeq) + 1:
		_fail_known("extraction_event_sequence")
		return
	if not _hive_settled(receipt.snapshot):
		_fail_known("extraction_settlement_missing")
		return
	_extraction_confirmed = true
	_write_event("beacon_extraction", "AUTHORITATIVE_RECEIPT_AND_STATE_CONFIRMED", { 		"targetId": target.get("entityId", ""), 		"beforeBeacon": old_beacon.duplicate(true), 		"afterBeacon": new_beacon.duplicate(true), 		"beforeProgression": old_progress.duplicate(true), 		"afterProgression": new_progress.duplicate(true), 		"receipt": receipt.duplicate(true) 	})
	_queue_captures_for_scene("gh_exit")
func _record_return_gate_if_valid(receipt: Dictionary, association: Dictionary) -> void:
	var target: Variant = association.get("target")
	var before: Variant = association.get("before")
	if not target is Dictionary or not before is Dictionary: return
	if target.get("entityId") == _route.get("sentinelExit"):
		if target.get("kind") != "world_gate" or before.get("worldId") != "grey_hive" or before.get("sceneId") != "gh_sentinel_arena" or receipt.snapshot.get("worldId") != "grey_hive" or receipt.snapshot.get("sceneId") != "gh_beacon" or not _sentinel_defeat_confirmed or not _sentinel_defeated(before): _fail_known("sentinel_exit_without_authoritative_defeat")
		else: _write_event("sentinel_exit", "AUTHORITATIVE_RECEIPT_CONFIRMED", {"gateId": target.entityId, "before": before.duplicate(true), "after": receipt.snapshot.duplicate(true)})
		return
	if target.get("kind") != "world_gate" or target.get("entityId") != _route.get("returnGate"): return
	if before.get("worldId") != "grey_hive" or before.get("sceneId") != "gh_exit":
		_fail_known("return_gate_source")
		return
	var after: Dictionary = receipt.snapshot
	if after.get("worldId") != "return_station" or after.get("sceneId") != _route.returnScene:
		_fail_known("return_gate_destination")
		return
	if not _extraction_confirmed or not _hive_settled(after):
		_fail_known("return_gate_without_settlement")
		return
	_return_gate_confirmed = true
	_write_event("return_gate", "AUTHORITATIVE_RECEIPT_CONFIRMED", { 		"gateId": target.entityId, 		"before": before.duplicate(true), 		"after": after.duplicate(true), 		"receipt": receipt.duplicate(true) 	})
func _record_save_if_valid(receipt: Dictionary, association: Dictionary) -> void:
	if _phase != "new" or association.get("purpose") != "save" or association.get("exitAfter") != false:
		_fail_known("save_scope_or_exit_binding")
		return
	if not _phase_a_ready_for_save(receipt.snapshot):
		_fail_known("save_before_full_route_acceptance")
		return
	var comparison := _comparison_values(receipt.snapshot)
	if comparison.is_empty():
		_fail_known("save_comparison_fields_missing")
		return
	if not _write_event("phase_summary", "SAVED", { 		"routeComplete": true, 		"events": _iface.requiredEvents.duplicate(), 		"baizhiChoice": _config.operator.baizhiChoice, 		"extractionConfirmed": _extraction_confirmed, 		"settlement": _hive_progress(receipt.snapshot).duplicate(true), 		"comparison": comparison, 		"snapshot": receipt.snapshot.duplicate(true) 	}):
		_stop_unknown("phase_summary_write_failed")
		return
	_save_confirmed = true
	_queue_captures_for_scene(_route.returnScene)
func _phase_a_ready_for_save(snapshot: Dictionary) -> bool:
	if _phase != "new" or not _journey_confirmed: return false
	if _route_index != _route.visits.size() or not _returned_to_station or not _return_gate_confirmed: return false
	if not _extraction_confirmed or not _hive_settled(snapshot) or not _baizhi_choice_confirmed: return false
	if snapshot.get("worldId") != "return_station" or snapshot.get("sceneId") != _route.returnScene: return false
	if not _captures_complete(true): return false
	return true
func _record_close_if_valid(result: Variant, association: Dictionary, connection_generation: int) -> void:
	if association.get("purpose") != "close" or not result is Dictionary or result.get("closing") != true:
		_fail_known("session_close_not_confirmed")
		return
	if not is_instance_valid(_rpc) or _rpc.get("authenticated") != true or connection_generation < 0 or int(_rpc.get("connection_generation")) != connection_generation:
		_stop_unknown("session_close_binding_changed")
		return
	if not _write_event("session_close", "CONFIRMED", { 		"authenticated": true, 		"association": _safe_association(association), 		"closing": true, 		"connectionGeneration": connection_generation 	}):
		_stop_unknown("session_close_evidence_write_failed")
		return
	if not _stop_outcome.is_empty(): return
	if _phase == "new" and (not _save_confirmed or not _close_requested or not _captures_complete()):
		_fail_known("close_without_save_receipt")
		return
	if _phase == "continue" and (not _journey_confirmed or _entry_ready.is_empty() or not _captures_complete() or not _phase_b_comparison_matches_current()):
		_fail_known("continue_closed_before_acceptance")
		return
	_write_event("run_complete", "ACCEPTED", { 		"phase": _phase, 		"journeyMethod": _journey_method, 		"routeIndex": _route_index, 		"returnedToStation": _returned_to_station, 		"saveReceiptConfirmed": _save_confirmed, 		"captures": _capture_done.keys(), 		"inputReceipts": _input_receipt_count, 		"actionReceipts": _action_receipt_count, 		"combatReceipts": _combat_receipt_count 	})
func _maybe_finish_phase() -> void:
	if _stopped or not _journey_confirmed or _entry_ready.is_empty(): return
	if not _captures_complete(): return
	if _capture_running or not _capture_queue.is_empty(): return
	if _phase == "new":
		if _save_confirmed: _request_close_once("saved_return_station_frame_confirmed")
	elif _phase == "continue" and _phase_b_comparison_matches_current():
		_request_close_once("continued_state_and_evidence_confirmed")
func _phase_b_comparison_matches_current() -> bool:
	if _phase != "continue" or not _journey_confirmed or not is_instance_valid(_main): return false
	var snapshot: Variant = _main.get("snapshot")
	return snapshot is Dictionary and P.snapshot(snapshot) and _compare_with_phase_a(snapshot)
func _compare_with_phase_a(snapshot: Dictionary) -> bool:
	if _phase != "continue" or _phase_a_summary.is_empty(): return false
	var actual := _comparison_values(snapshot)
	if actual.is_empty() or actual.size() != _comparison_fields.size(): return false
	for path in _comparison_fields:
		if not _phase_a_summary.has(path) or not actual.has(path): return false
		if actual[path] != _phase_a_summary[path]: return false
	return true
func _comparison_values(snapshot: Dictionary) -> Dictionary:
	var out: Dictionary = {}
	for path in _comparison_fields:
		var found := _lookup_path(snapshot, path)
		if not found.get("found", false): return {}
		out[path] = found.value
	return out
func _lookup_path(root_value: Variant, path: String) -> Dictionary:
	var current: Variant = root_value
	for segment in path.split("."):
		if current is Dictionary and current.has(segment): current = current[segment]
		elif current is Array and segment.is_valid_int():
			var index := int(segment)
			if index < 0 or index >= current.size(): return {"found": false}
			current = current[index]
		else: return {"found": false}
	return {"found": true, "value": current}
func _load_phase_a_summary() -> Dictionary:
	var prior_path := _run_root.path_join("phases").path_join("new").path_join("outputs").path_join(_iface.eventFileName)
	if not FileAccess.file_exists(prior_path): return {}
	var file := FileAccess.open(prior_path, FileAccess.READ)
	if file == null: return {}
	var length := file.get_length()
	if length <= 0 or length > MAX_PHASE_A_BYTES:
		file.close()
		return {}
	var raw := file.get_buffer(length)
	file.close()
	var text := raw.get_string_from_utf8()
	if text.to_utf8_buffer() != raw: return {}
	var summary: Dictionary = {}
	var completed := false
	var last_sequence := 0
	for line in text.split("\n"):
		if line.is_empty(): continue
		var value: Variant = JSON.parse_string(line)
		if not value is Dictionary: return {}
		if value.get("schema") != EVENT_SCHEMA or value.get("runId") != _run_id or value.get("phase") != "new": return {}
		if value.get("manifestSha256") != _manifest_sha256 or value.get("sourceBaselineReference") != _source_reference: return {}
		var sequence_value: Variant = value.get("sequence")
		if not P.counter(sequence_value, 1) or int(sequence_value) <= last_sequence: return {}
		last_sequence = int(sequence_value)
		if value.get("kind") == "phase_summary" and value.get("outcome") == "SAVED":
			if not value.get("payload") is Dictionary or not value.payload.get("routeComplete", false): return {}
			var comparison: Variant = value.payload.get("comparison")
			if not comparison is Dictionary or comparison.size() != _comparison_fields.size(): return {}
			for field in _comparison_fields:
				if not comparison.has(field): return {}
			summary = comparison.duplicate(true)
		if value.get("kind") == "run_complete" and value.get("outcome") == "ACCEPTED": completed = true
	if summary.is_empty() or not completed: return {}
	return summary
func _on_frame_committed(key: String) -> void:
	if not _validate_live(true):
		_emit_live_diagnostic("frame_committed")
		_stop_unknown("frame_context_mismatch")
		return
	var current_key := str(_renderer.get("current_key"))
	if key != current_key:
		_stop_unknown("frame_key_mismatch")
		return
	if key != _last_frame_key:
		_last_frame_key = key
		_write_event("frame_committed", "OBSERVED", { 			"key": key, 			"sceneHash": _renderer.get("scene_hash"), 			"viewport": _rect_array(_renderer.get("viewport_rect")) 		})
func _bind_capture() -> Dictionary:
	if not _validate_live() or not is_instance_valid(_main) or not is_instance_valid(_renderer): return {}
	var snapshot: Variant = _main.get("snapshot")
	var scene: Variant = _main.get("catalog").get("current")
	var camera: Variant = _renderer.get("camera")
	var viewport: Variant = _renderer.get("viewport_rect")
	if not snapshot is Dictionary or not P.snapshot(snapshot) or not scene is Dictionary or scene.is_empty(): return {}
	if not viewport is Rect2 or viewport.size.x <= 0.0 or viewport.size.y <= 0.0: return {}
	if not is_instance_valid(camera): return {}
	return { 		"snapshot": snapshot.duplicate(true), 		"scene": scene.duplicate(true), 		"current_key": str(_renderer.get("current_key")), 		"scene_hash": str(_renderer.get("scene_hash")), 		"camera": camera, 		"viewport_rect": viewport 	}
func _capture(label: String) -> void:
	var before := _bind_capture()
	if before.is_empty() or not _capture_scene_matches(label, before):
		_finish_capture_queue()
		_stop_unknown("capture_initial_binding_missing")
		return
	_renderer.call("queue_redraw")
	await RenderingServer.frame_post_draw
	if not _validate_live():
		_finish_capture_queue()
		_stop_unknown("capture_live_context_changed")
		return
	var after := _bind_capture()
	if after.is_empty() or not _capture_scene_matches(label, after) or not _capture_bindings_match(before, after):
		_finish_capture_queue()
		_stop_unknown("capture_binding_changed")
		return
	var viewport := _main.get_viewport()
	if viewport == null or not is_instance_valid(viewport):
		_finish_capture_queue()
		_stop_unknown("capture_viewport_missing")
		return
	var texture := viewport.get_texture()
	if texture == null:
		_finish_capture_queue()
		_stop_unknown("capture_texture_missing")
		return
	var image: Image = texture.get_image()
	if image == null or image.is_empty():
		_finish_capture_queue()
		_stop_unknown("capture_image_missing")
		return
	if not _write_capture(label, image, after):
		_finish_capture_queue()
		_stop_unknown("capture_write_or_binding_failed")
		return
	_finish_capture_queue()
func _write_capture(label: String, image: Image, binding: Dictionary) -> bool:
	if not _safe_label(label) or not image is Image or image.is_empty(): return false
	var path := _outputs_dir.path_join("capture-" + label + ".png")
	if not path.begins_with(_outputs_dir + "/") or FileAccess.file_exists(path): return false
	if not _validate_live(): return false
	var final_binding := _bind_capture()
	if final_binding.is_empty() or not _capture_scene_matches(label, final_binding) or not _capture_bindings_match(binding, final_binding): return false
	if image.get_width() <= 0 or image.get_height() <= 0: return false
	var error := image.save_png(path)
	if error != OK: return false
	_capture_done[label] = true
	var payload := { 		"label": label, 		"locator": "phases/" + _phase + "/outputs/capture-" + label + ".png", 		"width": image.get_width(), 		"height": image.get_height(), 		"binding": _capture_binding_json(binding) 	}
	return _write_event("completed_frame_capture", "SAVED", payload)
func _capture_stage_ready(label: String) -> bool:
	if _phase == "continue":
		return label == "continue-rs" and _journey_confirmed and not _entry_ready.is_empty() and _phase_b_comparison_matches_current()
	if _phase != "new" or not _journey_confirmed: return false
	match label:
		"new-rs": return _route_index == 1 and not _returned_to_station
		"gh-entry": return _route_index == 2
		"sentinel-encounter":
			var snapshot: Variant = _main.get("snapshot")
			var encounter: Variant = snapshot.get("sentinelEncounter") if snapshot is Dictionary else null
			return _route_index == 10 and not _sentinel_defeat_confirmed and encounter is Dictionary and encounter.get("actorId") == "gh_sentinel_arena_sentinel_01" and not encounter.get("phase") in ["dead", "inactive"]
		"gh-exit-extraction": return _extraction_confirmed and not _return_gate_confirmed
		"returned-rs-saved": return _save_confirmed and _returned_to_station and _return_gate_confirmed
	return false
func _capture_scene_matches(label: String, binding: Dictionary) -> bool:
	if not _capture_stage_ready(label): return false
	for item in _capture_plan:
		if item.id == label: return binding.scene.get("sceneId") == item.sceneId
	return false
func _capture_bindings_match(a: Dictionary, b: Dictionary) -> bool:
	for key in _route.captureBinding:
		if not a.has(key) or not b.has(key): return false
	if not a.snapshot is Dictionary or not b.snapshot is Dictionary: return false
	if a.snapshot != b.snapshot or not P.same(a.snapshot, b.snapshot): return false
	if a.scene != b.scene or a.current_key != b.current_key or a.scene_hash != b.scene_hash: return false
	if a.camera != b.camera or a.viewport_rect != b.viewport_rect: return false
	return true
func _capture_binding_json(binding: Dictionary) -> Dictionary:
	return { 		"snapshot": binding.snapshot.duplicate(true), 		"scene": binding.scene.duplicate(true), 		"current_key": binding.current_key, 		"scene_hash": binding.scene_hash, 		"cameraIdentity": str(binding.camera.get_instance_id()), 		"viewport_rect": _rect_array(binding.viewport_rect) 	}
func _queue_captures_for_scene(scene_id: String) -> void:
	for item in _capture_plan:
		var label: String = item.id
		if item.sceneId == scene_id and _capture_stage_ready(label) and not _capture_done.has(label) and not _capture_pending.has(label):
			_capture_pending[label] = true
			_capture_queue.append(label)
	_drain_capture_queue()
func _drain_capture_queue() -> void:
	if _capture_running or _capture_queue.is_empty() or _stopped: return
	_capture_running = true
	var label: String = _capture_queue.pop_front()
	call_deferred("_capture", label)
func _finish_capture_queue() -> void:
	_capture_running = false
	for label in _capture_pending.keys():
		if _capture_done.has(label): _capture_pending.erase(label)
	if _capture_pending.size() > _capture_queue.size():
		for label in _capture_pending.keys():
			if not _capture_done.has(label) and not label in _capture_queue:
				_capture_pending.erase(label)
				break
	_drain_capture_queue()
	_maybe_finish_phase()
func _captures_complete(before_save: bool = false) -> bool:
	for item in _capture_plan:
		if before_save and item.id == "returned-rs-saved": continue
		if not _capture_done.has(item.id): return false
	return true
func _request_close_once(reason: String) -> void:
	if _close_requested: return
	_close_requested = true
	_write_event("close_requested", "REQUESTED", {"reason": reason})
	if not is_instance_valid(_main) or not is_instance_valid(_rpc):
		_write_event("close_requested", "OWNED_MAIN_UNAVAILABLE", {"reason": reason})
		return
	if not _rpc.get("authenticated") or _main.get("uncertain") or not str(_main.get("critical")).is_empty():
		_write_event("close_requested", "MAIN_GUARD_REFUSED", {"reason": reason})
		return
	if _main.get("close_attempted"):
		_write_event("close_requested", "MAIN_ALREADY_CLOSING", {"reason": reason})
		return
	_main.call("_submit_close")
func _stop_unknown(reason: String) -> void:
	if _stopped: return
	_stopped = true
	_stop_outcome = "UNKNOWN"
	_write_event("run_stopped", "UNKNOWN", {"reason": reason})
	if is_instance_valid(_main) and is_instance_valid(_rpc) and _rpc.get("authenticated") and not _main.get("uncertain") and str(_main.get("critical")).is_empty() and not _main.get("close_attempted"): _request_close_once("unknown:" + reason)
func _fail_known(reason: String) -> void:
	if _stopped: return
	_stopped = true
	_stop_outcome = "FAILED"
	_write_event("run_stopped", "FAILED", {"reason": reason})
	if is_instance_valid(_main) and is_instance_valid(_rpc) and _rpc.get("authenticated") and not _main.get("uncertain") and str(_main.get("critical")).is_empty() and not _main.get("close_attempted"): _request_close_once("failed:" + reason)
func _write_event(kind: String, outcome: String, payload: Dictionary = {}) -> bool:
	if not _attached or not DirAccess.dir_exists_absolute(_outputs_dir): return false
	if _event_count >= MAX_EVENT_COUNT or _event_bytes >= MAX_EVENT_FILE_BYTES: return false
	_event_sequence += 1
	var utc := Time.get_datetime_string_from_system(true)
	if not utc.ends_with("Z"): utc += "Z"
	var event := { 		"schema": EVENT_SCHEMA, 		"runId": _run_id, 		"phase": _phase, 		"sequence": _event_sequence, 		"utc": utc, 		"monotonicMs": maxi(0, Time.get_ticks_msec() - _phase_started_ms), 		"kind": kind, 		"outcome": outcome, 		"manifestSha256": _manifest_sha256, 		"sourceBaselineReference": _source_reference, 		"payload": payload.duplicate(true) 	}
	var bytes := (JSON.stringify(event) + "\n").to_utf8_buffer()
	if bytes.is_empty() or bytes.size() > MAX_EVENT_LINE_BYTES or _event_bytes + bytes.size() > MAX_EVENT_FILE_BYTES: return false
	var output := FileAccess.open(_events_path, FileAccess.READ_WRITE) if FileAccess.file_exists(_events_path) else FileAccess.open(_events_path, FileAccess.WRITE)
	if output == null: return false
	output.seek_end()
	output.store_buffer(bytes)
	output.flush()
	var ok := output.get_error() == OK
	output.close()
	if not ok: return false
	_event_count += 1
	_event_bytes += bytes.size()
	return true
func _safe_association(association: Dictionary) -> Dictionary:
	var out: Dictionary = {}
	for key in ["purpose", "generation", "context", "expectedCommand", "actionKind", "exitAfter", "neutral", "choice", "page"]:
		if association.has(key):
			var value: Variant = association[key]
			if value is Dictionary or value is Array: out[key] = value.duplicate(true)
			elif typeof(value) in [TYPE_STRING, TYPE_INT, TYPE_FLOAT, TYPE_BOOL] or value == null: out[key] = value
	var target: Variant = association.get("target")
	if target is Dictionary: out["target"] = { 			"entityId": target.get("entityId", ""), 			"kind": target.get("kind", ""), 			"active": target.get("active", false) 		}
	var token_value: Variant = association.get("token")
	if token_value is Dictionary: out["tokenIdentity"] = _token_identity(token_value)
	var before: Variant = association.get("before")
	if before is Dictionary and P.snapshot(before): out["beforeSnapshot"] = before.duplicate(true)
	return out
func _safe_label(value: String) -> bool:
	if value.is_empty() or value.length() > 64: return false
	for character in value:
		if not character in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.-": return false
	return true
func _valid_field_path(value: String) -> bool:
	if value.is_empty() or value.length() > 256 or value.begins_with(".") or value.ends_with(".") or ".." in value: return false
	for character in value:
		if not character in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.": return false
	return true
func _absolute_path(value: String) -> bool:
	if value.is_empty() or value.contains("\\") or value.length() < 4: return false
	if value[1] != ":" or value[2] != "/" or value.simplify_path() != value: return false
	for component in value.substr(3).split("/"):
		if component.is_empty() or component.ends_with(".") or component.ends_with(" "): return false
		var folded := component.to_upper()
		if folded in ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"]: return false
	return true
func _same_path(a: String, b: String) -> bool:
	return a.trim_suffix("/") == b.trim_suffix("/")
func _valid_session_token(value: String) -> bool:
	if value.length() < 32 or value.length() > 128: return false
	for character in value:
		if not character in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_": return false
	return true
func _canonical_uuid(value: Variant) -> bool:
	if typeof(value) != TYPE_STRING: return false
	var regex := RegEx.new()
	if regex.compile("^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$") != OK: return false
	return regex.search(value) != null
func _hex_digest(value: Variant) -> bool:
	if typeof(value) != TYPE_STRING or value.length() != 64: return false
	for character in value:
		if not character in "0123456789abcdef": return false
	return true
func _hex_oid(value: Variant) -> bool:
	if typeof(value) != TYPE_STRING or value.length() != 40: return false
	for character in value:
		if not character in "0123456789abcdef": return false
	return true
func _positive_integer(value: Variant) -> bool:
	return (typeof(value) == TYPE_INT or typeof(value) == TYPE_FLOAT) and is_finite(float(value)) and float(value) > 0.0 and floor(float(value)) == float(value)
func _positive_limit(source: Dictionary, alternatives: Array) -> bool:
	for key in alternatives:
		if source.has(key):
			var value: Variant = source[key]
			return (typeof(value) == TYPE_INT or typeof(value) == TYPE_FLOAT) and is_finite(float(value)) and float(value) > 0.0
	return false
func _phase_limit_ms() -> int:
	var limits: Dictionary = _config.get("limits", {})
	var seconds: Variant = limits.get("phaseSeconds", 0)
	return int(float(seconds) * 1000.0) if typeof(seconds) in [TYPE_INT, TYPE_FLOAT] and is_finite(float(seconds)) else 1
func _context_key(snapshot: Dictionary) -> String:
	if not P.context(snapshot): return ""
	return str(snapshot.worldId) + ":" + str(snapshot.sceneId) + ":" + str(snapshot.worldEpoch)
func _token_key(token_value: Variant) -> String:
	if not token_value is Dictionary or not P.token(token_value): return ""
	return str(token_value.generation) + ":" + str(token_value.worldId) + ":" + str(token_value.sceneId) + ":" + str(token_value.worldEpoch)
func _token_identity(token_value: Dictionary) -> Dictionary:
	return { 		"generation": token_value.get("generation"), 		"worldId": token_value.get("worldId"), 		"sceneId": token_value.get("sceneId"), 		"worldEpoch": token_value.get("worldEpoch") 	}
func _rect_array(value: Variant) -> Array:
	if not value is Rect2: return []
	return [value.position.x, value.position.y, value.size.x, value.size.y]
func _snapshot_signature(snapshot: Dictionary) -> String:
	var beacon: Variant = snapshot.get("greyHiveBeacon", {})
	var sentinel: Variant = snapshot.get("sentinelEncounter", {})
	var progress := _hive_progress(snapshot)
	return str([ 		_context_key(snapshot), 		snapshot.get("authorityRevision"), 		snapshot.get("ackSeq"), 		snapshot.get("player", {}).get("currentHp"), 		snapshot.get("progression", {}).get("eventSeq"), 		progress.get("completedEvents", []), 		progress.get("completed", false), 		progress.get("firstCompletion", false), 		progress.get("hiveChoice", ""), 		beacon.get("state", "") if beacon is Dictionary else "", 		beacon.get("legacyCompletedWithoutReceipt", null) if beacon is Dictionary else null, 		sentinel.get("phase", "") if sentinel is Dictionary else "" 	])
func _has_native_events(value: Variant) -> bool:
	if value is Array: return not value.is_empty()
	if value is Dictionary:
		for key in ["events", "rows", "cues", "items"]:
			var rows: Variant = value.get(key)
			if rows is Array and not rows.is_empty(): return true
		return value.has("event") or value.has("eventId")
	return false
func _wait_initial_viewport() -> bool:
	if _stopped or _initial_viewport_validated or _journey_confirmed or _journey_request_count != 0 or not _last_frame_key.is_empty(): return false
	if _pending_live_diagnostic.get("branch") != "viewport_shape_or_size": return false
	if _main.get("journey_started") != false or _renderer.get("viewport_rect") != Rect2(): return false
	var snapshot: Dictionary = _main.get("snapshot")
	if snapshot.get("entryToken") != null: return false
	var status_value: Variant = _main.get("status")
	if status_value is Dictionary and not status_value.is_empty():
		if not P.status(status_value) or (P.same(status_value, snapshot) and status_value.get("entryToken") != null): return false
	var actual_rect: Rect2 = _renderer.call("get_viewport_rect")
	if not is_finite(actual_rect.position.x) or not is_finite(actual_rect.position.y) or not is_finite(actual_rect.size.x) or not is_finite(actual_rect.size.y) or actual_rect.size.x <= 0.0 or actual_rect.size.y <= 0.0: return false
	var elapsed: int = Time.get_ticks_msec() - _phase_started_ms
	var startup_ms: float = float(_config.limits.startupSeconds) * 1000.0
	return elapsed >= 0 and float(elapsed) < minf(startup_ms, float(_phase_limit_ms()))

func _validate_live(diagnose: bool = false) -> bool:
	if diagnose: _pending_live_diagnostic.clear()
	if not is_instance_valid(_main) or not is_instance_valid(_renderer): return _live_reject(diagnose, "owners", {"mainValid": is_instance_valid(_main), "rendererValid": is_instance_valid(_renderer)} if diagnose else {})
	var snapshot: Variant = _main.get("snapshot")
	if not snapshot is Dictionary: return _live_reject(diagnose, "main_snapshot_type", {"snapshotType": typeof(snapshot)} if diagnose else {})
	if snapshot.is_empty():
		if diagnose and _journey_confirmed: _live_reject(true, "main_snapshot_empty_after_journey", {"journeyConfirmed": true})
		return not _journey_confirmed
	if not P.snapshot(snapshot): return _live_reject(diagnose, "main_snapshot_invalid", {"mainContext": _live_diag_context(snapshot)} if diagnose else {})
	var renderer_snapshot: Variant = _renderer.get("snapshot")
	if not renderer_snapshot is Dictionary or not P.snapshot(renderer_snapshot) or not P.same(snapshot, renderer_snapshot): return _live_reject(diagnose, "renderer_snapshot_or_context", {"mainContext": _live_diag_context(snapshot), "rendererContext": _live_diag_context(renderer_snapshot), "rendererValid": P.snapshot(renderer_snapshot)} if diagnose else {})
	var catalog: Variant = _main.get("catalog")
	if catalog == null or not is_instance_valid(catalog): return _live_reject(diagnose, "catalog_invalid", {} if diagnose else {})
	var scene: Variant = catalog.get("current")
	if not scene is Dictionary or scene.is_empty(): return _live_reject(diagnose, "catalog_scene_empty_or_type", {"sceneType": typeof(scene), "sceneEmpty": scene.is_empty() if scene is Dictionary else true} if diagnose else {})
	if scene.get("worldId") != snapshot.get("worldId") or scene.get("sceneId") != snapshot.get("sceneId"): return _live_reject(diagnose, "catalog_scene_context", {"expected": _live_diag_context(snapshot), "actual": _live_diag_context(scene)} if diagnose else {})
	if not catalog.get("manifest") is Dictionary or not catalog.manifest.has(snapshot.sceneId): return _live_reject(diagnose, "catalog_manifest_scene", {"manifestType": typeof(catalog.get("manifest")), "sceneMember": catalog.manifest.has(snapshot.sceneId) if catalog.get("manifest") is Dictionary else false} if diagnose else {})
	if _renderer.get("scene") != scene: return _live_reject(diagnose, "renderer_scene", {"expected": _live_diag_context(scene), "actual": _live_diag_context(_renderer.get("scene")), "sceneEqual": false} if diagnose else {})
	if typeof(_renderer.get("current_key")) != TYPE_STRING or _renderer.get("current_key") != _context_key(snapshot): return _live_reject(diagnose, "renderer_key", {"expected": _live_diag_text(_context_key(snapshot)), "actual": _live_diag_text(_renderer.get("current_key"))} if diagnose else {})
	var scene_hash: Variant = _renderer.get("scene_hash")
	if not _hex_digest(scene_hash) or catalog.get("source_hash") != scene_hash: return _live_reject(diagnose, "scene_hash", {"expected": catalog.get("source_hash") if _hex_digest(catalog.get("source_hash")) else null, "actual": scene_hash if _hex_digest(scene_hash) else null} if diagnose else {})
	var camera: Variant = _renderer.get("camera")
	if camera == null or not is_instance_valid(camera): return _live_reject(diagnose, "camera_invalid", {"cameraValid": false} if diagnose else {})
	var stored_rect: Variant = _renderer.get("viewport_rect")
	var actual_rect: Rect2 = _renderer.call("get_viewport_rect")
	if not stored_rect is Rect2 or stored_rect.size.x <= 0.0 or stored_rect.size.y <= 0.0 or actual_rect.size.x <= 0.0 or actual_rect.size.y <= 0.0: return _live_reject(diagnose, "viewport_shape_or_size", {"stored": _live_diag_rect(stored_rect), "actual": _live_diag_rect(actual_rect)} if diagnose else {})
	if not is_finite(actual_rect.position.x) or not is_finite(actual_rect.position.y) or not is_finite(actual_rect.size.x) or not is_finite(actual_rect.size.y): return _live_reject(diagnose, "viewport_nonfinite", {"stored": _live_diag_rect(stored_rect), "actual": _live_diag_rect(actual_rect)} if diagnose else {})
	if stored_rect != actual_rect: return _live_reject(diagnose, "viewport_changed", {"stored": _live_diag_rect(stored_rect), "actual": _live_diag_rect(actual_rect)} if diagnose else {})
	var entry_token: Variant = snapshot.get("entryToken")
	if entry_token != null and (not P.token(entry_token) or not P.same(entry_token, snapshot)): return _live_reject(diagnose, "entry_token", {"present": true, "valid": P.token(entry_token), "contextMatches": P.same(entry_token, snapshot) if entry_token is Dictionary else false} if diagnose else {})
	var status_value: Variant = _main.get("status")
	if status_value is Dictionary and not status_value.is_empty():
		if not P.status(status_value): return _live_reject(diagnose, "status_invalid", {"statusContext": _live_diag_context(status_value)} if diagnose else {})
		if P.same(status_value, snapshot):
			var status_token: Variant = status_value.get("entryToken")
			if status_token != null and (not P.token(status_token) or not P.same(status_token, snapshot)): return _live_reject(diagnose, "status_entry_token", {"present": true, "valid": P.token(status_token), "contextMatches": P.same(status_token, snapshot) if status_token is Dictionary else false} if diagnose else {})
	return true
func _live_reject(diagnose: bool, branch: String, details: Dictionary) -> bool:
	if diagnose and not _live_diagnostic_logged:
		_pending_live_diagnostic = {"diagnostic": "GH_LIVE_CONTEXT_V1", "phase": _phase, "branch": branch, "details": details}
	return false
func _emit_live_diagnostic(callsite: String) -> void:
	if _live_diagnostic_logged or _pending_live_diagnostic.is_empty(): return
	_live_diagnostic_logged = true
	var record: Dictionary = _pending_live_diagnostic.duplicate(true)
	record["callsite"] = callsite
	var encoded: String = JSON.stringify(record)
	if encoded.to_utf8_buffer().size() > 2048:
		encoded = JSON.stringify({"diagnostic": "GH_LIVE_CONTEXT_V1", "phase": _phase, "branch": record.branch, "callsite": callsite, "detailsOmitted": true})
	_pending_live_diagnostic.clear()
	print("GH_LIVE_CONTEXT_DIAGNOSTIC " + encoded)
func _live_diag_context(value: Variant) -> Dictionary:
	if not value is Dictionary: return {"type": typeof(value)}
	return {"worldId": _live_diag_text(value.get("worldId")), "sceneId": _live_diag_text(value.get("sceneId")), "worldEpoch": value.get("worldEpoch") if P.counter(value.get("worldEpoch")) else null}
func _live_diag_text(value: Variant) -> String:
	return value if P.ident(value) else "<invalid>"
func _live_diag_rect(value: Variant) -> Array:
	if not value is Rect2: return []
	var components: Array = []
	for component in [value.position.x, value.position.y, value.size.x, value.size.y]:
		components.append(component if is_finite(float(component)) else null)
	return components
func _safe_association_context(association: Dictionary) -> Dictionary:
	var value: Variant = association.get("context", {})
	return value.duplicate(true) if value is Dictionary else {}
