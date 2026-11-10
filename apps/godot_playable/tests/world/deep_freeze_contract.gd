extends SceneTree
# Offline contract for the client freezes fixed by chief 23:
#  (1) `gh_central_shaft`: the lift frame cannot be composed when the authority publishes no
#      moving-support view at all; that optional overlay must not lock input while the player
#      stands on the authored floor inside the scene envelope (`floor_without_support`).
#  (2) floor-route scenes: with NO authoritative occupied region the client plans against the
#      nearest authored region, and still fails closed for genuinely unready geometry.
#  (3) the pure hold-recovery decision table (`scripts/ui/hold_recovery.gd`).
# Run: godot --headless --path apps/godot_playable --script res://tests/world/deep_freeze_contract.gd
const P = preload('res://scripts/bridge/protocol.gd')
const RegionsScript = preload('res://scripts/world/gh_floor_route_region_visual.gd')
const HoldRecovery = preload('res://scripts/ui/hold_recovery.gd')
const SHAFT_HASH = '6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832'
const LOCK_HASH = '23e5d5d9f37f6f44b1f8b9105aab656424778ef5edf0e775bbc364143fe1b30c'
const SHAFT_CROSS = 'grey_hive.gh_central_shaft.upper_active_info_v1.cross_transit'
const LOCK_WEST = 'grey_hive.gh_lockdown.dense_info_v1.west_transit'
var regions
var failures: Array[String] = []

func _init() -> void:
	regions = RegionsScript.new()
	_shaft_cases()
	_lockdown_fallback_cases()
	_hold_recovery_cases()
	if failures.is_empty():
		print('DEEP-FREEZE-CONTRACT PASS')
		quit(0)
	else:
		for row in failures:
			print('DEEP-FREEZE-CONTRACT FAIL '+row)
		print('DEEP-FREEZE-CONTRACT FAIL count=%d' % failures.size())
		quit(1)

func _ok(name: String, condition: bool) -> void:
	if not condition:
		failures.append(name)

func _point(x: float, y: float, z: float) -> Dictionary:
	return {'xM':x,'yM':y,'zM':z}

func _transform(x: float, y: float, z: float) -> Dictionary:
	return {'positionM':_point(x,y,z),'yawRad':0.0}

func _fixture(scene_id: String, x: float, z: float, y: float = 0.0) -> Dictionary:
	return {
		'kind':'full','protocolVersion':3,'schemaVersion':'freeze-v02-interfaces/1.2',
		'worldId':'grey_hive','sceneId':scene_id,'checkpointId':null,
		'worldEpoch':6,'serverTick':120,'authorityRevision':9,'ackSeq':0,'entryToken':null,
		'player':{'entityId':'player','transform':_transform(x,y,z),'velocityMps':_point(0,0,0),
			'actionState':'idle','currentHp':100,'maxHp':100,'currentEnergy':100,'maxEnergy':100,
			'facingX':0.0,'facingZ':1.0,'aimX':0.0,'aimZ':1.0},
		'actors':[],'doors':[],'interactables':[],'hazards':[],'objectives':[],
		'capabilities':{'schemaVersion':1,'items':[]},
		'progression':{'schemaVersion':1,'eventSeq':1,'currentWorldId':'grey_hive','worlds':[]},
		'denseEntityRegions':{},
		'denseOccupiedRegion':null,
		'currentVisibleRegions':null
	}

func _scene(path: String) -> Dictionary:
	var decoded = JSON.parse_string(FileAccess.get_file_as_string(path))
	return decoded if decoded is Dictionary else {}

func _shaft_cases() -> void:
	var scene := _scene('res://data/scenes/gh_central_shaft.json')
	_ok('shaft scene parses', not scene.is_empty())
	_ok('shaft bounds envelope', regions.bounds_contain(scene.get('boundsM'),Vector3(0.05,0.0,9.04)))
	_ok('shaft bounds rejects outside', not regions.bounds_contain(scene.get('boundsM'),Vector3(30.0,0.0,9.0)))
	_ok('shaft bounds rejects malformed', not regions.bounds_contain({},Vector3(1,0,1)))
	# The two observed standstill points: no support view, floor height, inside the envelope.
	for point in [[0.05,9.04],[23.99,9.12]]:
		var snapshot := _fixture('gh_central_shaft',point[0],point[1])
		_ok('shaft fixture valid %s' % str(point), P.snapshot(snapshot))
		_ok('shaft no-support floor accepted %s' % str(point), regions.floor_without_support(snapshot,scene)==true)
	# Fail-closed variants of the same predicate.
	var with_lift := _fixture('gh_central_shaft',0.05,9.04)
	with_lift.shaftLift = {'upperGateOpen':false}
	_ok('shaft rejects a published lift', regions.floor_without_support(with_lift,scene)==false)
	var with_support := _fixture('gh_central_shaft',0.05,9.04)
	with_support.supportScene = {'rider':{'mode':'floor','positionM':_point(0.05,0.0,9.04)}}
	_ok('shaft rejects a published support scene', regions.floor_without_support(with_support,scene)==false)
	var deferred := _fixture('gh_central_shaft',0.05,9.04)
	deferred.shaftLiftDeferred = true
	_ok('shaft rejects a deferred-floor proof state', regions.floor_without_support(deferred,scene)==false)
	var raised := _fixture('gh_central_shaft',12.0,8.0,1.0)
	_ok('shaft rejects another height', regions.floor_without_support(raised,scene)==false)
	var outside := _fixture('gh_central_shaft',30.0,8.0)
	_ok('shaft rejects outside the envelope', regions.floor_without_support(outside,scene)==false)
	var other_scene := _fixture('gh_lockdown',0.05,9.04)
	_ok('shaft predicate rejects another scene', regions.floor_without_support(other_scene,scene)==false)
	var no_support := _fixture('gh_central_shaft',0.05,9.04)
	_ok('shaft unknown support has no region scope', regions.scope(no_support,SHAFT_HASH)==false)
	_ok('shaft unknown support plans no strict region', regions.prepare(no_support,scene,SHAFT_HASH,[]).is_empty())
	# The strict shaft path still works when the authority publishes the lift/support view.
	var known := _fixture('gh_central_shaft',12.0,8.0)
	known.denseOccupiedRegion = SHAFT_CROSS
	known.currentVisibleRegions = [SHAFT_CROSS]
	known.shaftLift = {'upperGateOpen':false}
	known.supportScene = {'rider':{'mode':'floor','positionM':_point(12.0,0.0,8.0)}}
	var known_plan: Dictionary = regions.prepare(known,scene,SHAFT_HASH,[])
	_ok('strict shaft path plans one region', not known_plan.is_empty())
	_ok('strict shaft path region matches authority', str(known_plan.get('region',''))==SHAFT_CROSS)
	_ok('strict shaft path contains player', regions.contains(known_plan,Vector3(12.0,0.0,8.0)))
	_ok('strict shaft path is not a fallback', known_plan.get('fallback')!=true)
	_ok('strict shaft path takes no fallback shortcut', regions.prepare_nearest(known,scene,SHAFT_HASH,[]).is_empty())
	_ok('strict shaft path rejects no-support floor', regions.floor_without_support(known,scene)==false)

func _lockdown_fallback_cases() -> void:
	var scene := _scene('res://data/scenes/gh_lockdown.json')
	_ok('lockdown scene parses', not scene.is_empty())
	# (10.5,3.0) is the authored 1 m wall gap between the northwest and northeast services:
	# inside the scene envelope, outside every authored region part.
	var snapshot := _fixture('gh_lockdown',10.5,3.0)
	_ok('lockdown fixture valid', P.snapshot(snapshot))
	_ok('lockdown region scope is live', regions.scope(snapshot,LOCK_HASH)==true)
	_ok('lockdown strict prepare has no region', regions.prepare(snapshot,scene,LOCK_HASH,[]).is_empty())
	var plan: Dictionary = regions.prepare_nearest(snapshot,scene,LOCK_HASH,[])
	_ok('lockdown fallback plan exists', not plan.is_empty())
	_ok('lockdown fallback marked', plan.get('fallback')==true)
	_ok('lockdown fallback distance is measured', float(plan.get('fallback_distance_m',-1.0))>0.0)
	_ok('lockdown fallback region is authored', str(plan.get('region','')).begins_with('grey_hive.gh_lockdown.dense_info_v1.'))
	_ok('lockdown fallback does not claim containment', not regions.contains(plan,Vector3(10.5,0.0,3.0)))
	# Fail-closed variants.
	var occupied := _fixture('gh_lockdown',10.5,3.0)
	occupied.denseOccupiedRegion = LOCK_WEST
	occupied.currentVisibleRegions = [LOCK_WEST]
	_ok('lockdown occupied snapshot: no fallback', regions.prepare_nearest(occupied,scene,LOCK_HASH,[]).is_empty())
	var outside := _fixture('gh_lockdown',30.0,3.0)
	_ok('lockdown outside envelope: no fallback', regions.prepare_nearest(outside,scene,LOCK_HASH,[]).is_empty())
	var raised := _fixture('gh_lockdown',10.5,3.0,1.0)
	_ok('lockdown wrong floor: no fallback', regions.prepare_nearest(raised,scene,LOCK_HASH,[]).is_empty())
	var contradictory := _fixture('gh_lockdown',10.5,3.0)
	contradictory.currentVisibleRegions = [LOCK_WEST]
	_ok('lockdown visible region without occupied: no fallback', regions.prepare_nearest(contradictory,scene,LOCK_HASH,[]).is_empty())
	var unready := _scene('res://data/scenes/gh_lockdown.json')
	unready.collision[0].polygon[0][0] = float(unready.collision[0].polygon[0][0])+0.25
	_ok('lockdown genuinely unready geometry: no fallback', regions.prepare_nearest(_fixture('gh_lockdown',10.5,3.0),unready,LOCK_HASH,[]).is_empty())
	_ok('lockdown other scene sha: no fallback', regions.prepare_nearest(_fixture('gh_lockdown',10.5,3.0),scene,'00',[]).is_empty())
	# The strict floor-route path still plans the authoritative region.
	var strict := _fixture('gh_lockdown',5.0,8.0)
	strict.denseOccupiedRegion = LOCK_WEST
	strict.currentVisibleRegions = [LOCK_WEST]
	var strict_plan: Dictionary = regions.prepare(strict,scene,LOCK_HASH,[])
	_ok('strict lockdown path plans one region', not strict_plan.is_empty())
	_ok('strict lockdown region matches authority', str(strict_plan.get('region',''))==LOCK_WEST)
	_ok('strict lockdown contains player', regions.contains(strict_plan,Vector3(5.0,0.0,8.0)))
	_ok('strict lockdown is not a fallback', strict_plan.get('fallback')!=true)

func _hold_facts() -> Dictionary:
	return {
		'uncertain':true,'journey_started':true,
		'error_page':true,'panel_pending':false,
		'authenticated':true,'focused':true,'critical_empty':true,
		'snapshot_valid':true,'status_valid':true,'context_same':true,
		'entry_token_absent':true,'paused':false,'alive':true,
		'preview_absent':true,'ready_absent':true,'ticket_absent':true,
		'renderer_valid':true,'dense_ready':true,
		'hold_at_ms':1000,'last_snapshot_at':1400,'last_status_at':1500,'now_ms':1600}

func _hold_recovery_cases() -> void:
	var base := _hold_facts()
	_ok('hold recovery accepts a fresh post-hold pair', HoldRecovery.ready(base)==true)
	var rejections := [
		['not held','uncertain',false],['no journey','journey_started',false],
		['wrong page','error_page',false],['panel pending','panel_pending',true],
		['not authenticated','authenticated',false],['not focused','focused',false],
		['blocking request','critical_empty',false],
		['invalid snapshot','snapshot_valid',false],['invalid status','status_valid',false],
		['context mismatch','context_same',false],['entry token','entry_token_absent',false],
		['paused','paused',true],['dead','alive',false],
		['exit preview','preview_absent',false],['pending entry','ready_absent',false],
		['baizhi ticket','ticket_absent',false],['no renderer','renderer_valid',false],
		['dense plan not ready','dense_ready',false]]
	for row in rejections:
		var facts := _hold_facts()
		facts[row[1]] = row[2]
		_ok('hold recovery rejects '+str(row[0]), HoldRecovery.ready(facts)==false)
	var zero := _hold_facts()
	zero.hold_at_ms = 0
	_ok('hold recovery rejects missing hold time', HoldRecovery.ready(zero)==false)
	var stale_snapshot := _hold_facts()
	stale_snapshot.last_snapshot_at = 1000
	_ok('hold recovery rejects pre-hold snapshot', HoldRecovery.ready(stale_snapshot)==false)
	var stale_status := _hold_facts()
	stale_status.last_status_at = 900
	_ok('hold recovery rejects pre-hold status', HoldRecovery.ready(stale_status)==false)
	var stale_age := _hold_facts()
	stale_age.now_ms = 2400
	_ok('hold recovery rejects stale evidence', HoldRecovery.ready(stale_age)==false)
	var string_time := _hold_facts()
	string_time.now_ms = '1700'
	_ok('hold recovery rejects malformed time', HoldRecovery.ready(string_time)==false)
	_ok('hold recovery rejects non-dictionary', HoldRecovery.ready({})==false)
