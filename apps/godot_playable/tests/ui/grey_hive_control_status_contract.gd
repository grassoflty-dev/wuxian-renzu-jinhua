extends SceneTree
# SOURCE ONLY: not executed; separate future execution authority required.
const P = preload('res://scripts/bridge/protocol.gd')
const WorldMechanism = preload('res://scripts/ui/world_mechanism_model.gd')
const Decon = preload('res://tests/ui/decon_hazard_text_contract.gd')
const BASE = ['信标：尚未收取']
const SPECS = [
	['gh_power_room','gh_power_console','power_console','主电控制',7],
	['gh_lockdown','gh_lockdown_terminal','lockdown_terminal','隔离控制',8]
]
const BEACONS = [
	['uncollected',false,'尚未收取'],['carried',false,'已携带'],['mounted',false,'已部署'],
	['uncollected',true,'收取回执未确认'],['carried',true,'状态未确认'],['unknown',false,'状态未确认'],['mounted',true,'状态未确认'],['uncollected',1,'状态未确认']
]
var _failures: int = 0
func _initialize() -> void:
	for spec in SPECS:
		_controls(spec)
		_rejections(spec)
		_contexts(spec)
	_exit_controls()
	print('GREY_HIVE_CONTROL_STATUS_CONTRACT failures='+str(_failures))
	quit(0 if _failures==0 else 1)
func _check(ok: bool, label: String) -> void:
	if not ok:
		_failures += 1
		push_error(label)
func _expect(label: String, s: Dictionary, expected: Array, eligible: Variant=true, valid: bool=true) -> void:
	var before: Dictionary = s.duplicate(true)
	var fixture_ok: bool = P.snapshot(s)==valid
	_check(fixture_ok,label+': fixture validity')
	if fixture_ok:
		_check(WorldMechanism.project(s,eligible)==expected,label+': projection')
	_check(s==before,label+': input mutated')
static func _row(id: String, kind: String, active: bool, x: float, z: float) -> Dictionary:
	return {'entityId':id,'kind':kind,'active':active,'transform':Decon._transform(x,0,z)}
static func _fixture(spec: Array) -> Dictionary:
	var s: Dictionary = Decon._fixture()
	s.worldId = 'grey_hive'
	s.sceneId = spec[0]
	s.player.transform = Decon._transform(2,0,spec[4])
	s.hazards = []
	s.interactables = [_row(spec[1],spec[2],true,4,spec[4])]
	return s
func _controls(spec: Array) -> void:
	for beacon in BEACONS:
		for mode in ['active','inactive','wrong kind','missing','unknown ID']:
			var s: Dictionary = _fixture(spec)
			s.greyHiveBeacon = {'state':beacon[0],'legacyCompletedWithoutReceipt':beacon[1]}
			var expected: Array = ['信标：'+beacon[2]]
			if mode=='inactive':
				s.interactables[0].active = false
			if mode=='wrong kind':
				s.interactables[0].kind = 'world_gate'
			if mode=='unknown ID':
				s.interactables[0].entityId = 'unrelated_control'
			if mode=='missing':
				s.interactables = []
			if mode in ['active','inactive','wrong kind']:
				expected.append(spec[3]+'：'+{'active':'当前启用','inactive':'当前未启用','wrong kind':'状态未确认'}[mode])
			_expect(spec[0]+'/'+str(beacon)+'/'+mode,s,expected)
func _rejections(spec: Array) -> void:
	var s: Dictionary = _fixture(spec)
	for rows in [null,{},[null],[false],[{}]]:
		s.interactables = rows
		_expect(spec[0]+'/malformed rows/'+str(rows),s,BASE,true,false)
	s = _fixture(spec)
	s.interactables.append(s.interactables[0].duplicate(true))
	_expect(spec[0]+'/duplicate ID',s,BASE,true,false)
	for field in ['kind','active','entityId','transform']:
		s = _fixture(spec)
		s.interactables[0].erase(field)
		_expect(spec[0]+'/missing row field/'+field,s,BASE,true,false)
	for mutation in [['kind',1],['active',1],['active',null],['entityId',false],['transform',null],['transform',{'positionM':{'xM':4,'yM':true,'zM':spec[4]},'yawRad':0}],['transform',{'positionM':{'xM':4,'yM':0,'zM':spec[4]},'yawRad':true}]]:
		s = _fixture(spec)
		s.interactables[0][mutation[0]] = mutation[1]
		_expect(spec[0]+'/invalid row/'+str(mutation),s,BASE,true,false)
	for mutation in [['kind','delta',[]],['schemaVersion','invalid',[]],['protocolVersion',true,[]],['serverTick',-1,[]],['entryToken',true,[]],['player',{},BASE],['checkpointId',true,BASE],['actors',[null],BASE],['hazards',[false],BASE],['capabilities',{},BASE]]:
		s = _fixture(spec)
		s[mutation[0]] = mutation[1]
		_expect(spec[0]+'/invalid full snapshot/'+str(mutation),s,mutation[2],true,false)
	s = _fixture(spec)
	s.erase('checkpointId')
	_expect(spec[0]+'/missing checkpoint',s,BASE,true,false)
func _contexts(spec: Array) -> void:
	for eligible in [false,null,0,1,'true',[],{}]:
		_expect(spec[0]+'/eligibility/'+str(eligible),_fixture(spec),[],eligible)
	var s: Dictionary = _fixture(spec)
	s.entryToken = {'worldId':'grey_hive','sceneId':spec[0],'worldEpoch':1,'generation':1}
	_expect(spec[0]+'/pending entry',s,[])
	for scene in ['gh_beacon','gh_lockdown' if spec[0]=='gh_power_room' else 'gh_power_room']:
		s = _fixture(spec)
		s.sceneId = scene
		_expect(spec[0]+'/other room/'+scene,s,BASE)
	s = _fixture(spec)
	s.worldId = 'return_station'
	s.sceneId = 'rs_core_room'
	s.progression.currentWorldId = 'return_station'
	_expect(spec[0]+'/other world',s,[])
func _exit_controls() -> void:
	var s: Dictionary = _fixture(SPECS[0])
	s.sceneId = 'gh_exit'
	s.player.transform = Decon._transform(2,0,8)
	s.interactables = [_row('gh_exit_extraction_console','extraction_console',true,8,8),_row('gh_extraction_return_to_rs','world_gate',false,8,8)]
	for pair in [[true,false,'撤离控制：当前启用','归航门：当前未启用'],[false,true,'撤离控制：当前未启用','归航门：当前启用']]:
		s.interactables[0].active = pair[0]
		s.interactables[1].active = pair[1]
		_expect('existing exit/'+str(pair),s,BASE+[pair[2],pair[3]])
	s.interactables = []
	_expect('existing exit missing rows',s,BASE+['撤离控制：状态未确认','归航门：状态未确认'])
