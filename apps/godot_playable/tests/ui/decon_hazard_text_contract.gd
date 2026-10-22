extends SceneTree
# SOURCE ONLY. Proposed command NOT_RUN; separate future execution authority required.
# godot --headless --path apps/godot_playable --script res://tests/ui/decon_hazard_text_contract.gd
const P = preload('res://scripts/bridge/protocol.gd')
const WorldMechanism = preload('res://scripts/ui/world_mechanism_model.gd')
const BASE = ['信标：尚未收取']
const PHASES = {'warning':'预警','active':'危险','recovery':'间歇','suppressed':'已抑制'}
const SPECS = [
	['gh_decon_steam_jet_zone','steam_jet','pressure','蒸汽',[[7,4],[10,4],[10,6.5],[7,6.5]]],
	['gh_decon_mist_zone','decon_mist','toxin','消杀雾',[[14,9],[18,9],[18,14],[14,14]]]
]
var _failures: int = 0

func _initialize() -> void:
	_phases_and_geometry()
	_hazard_rejections()
	_context_and_schema()
	_beacons()
	print('DECON_HAZARD_TEXT_CONTRACT failures='+str(_failures))
	quit(0 if _failures==0 else 1)

func _expect(label: String, s: Dictionary, expected: Array, eligible: Variant=true, valid: bool=true) -> void:
	if P.snapshot(s)!=valid:
		_fail(label+': fixture validity')
		return
	var before: Dictionary = s.duplicate(true)
	if WorldMechanism.project(s,eligible)!=expected:
		_fail(label+': projection')
	if s!=before:
		_fail(label+': input mutated')

func _fail(label: String) -> void:
	_failures += 1
	push_error(label)

static func _point(x: float, y: float, z: float) -> Dictionary:
	return {'xM':x,'yM':y,'zM':z}

static func _transform(x: float, y: float, z: float) -> Dictionary:
	return {'positionM':_point(x,y,z),'yawRad':0}

static func _fixture(which: int=0, phase: String='active') -> Dictionary:
	var spec: Array = SPECS[which]
	var polygon: Array = spec[4]
	var x: float = (polygon[0][0]+polygon[2][0])*0.5
	var z: float = (polygon[0][1]+polygon[2][1])*0.5
	var hazard: Dictionary = {
		'entityId':spec[0],'kind':spec[1],'transform':_transform(x,0,z),
		'active':phase=='active','phaseActive':true,'polygonM':polygon.duplicate(true),
		'environment':{'phase':phase,'remainingMs':1,'exposureBps':0,'tag':spec[2]}
	}
	return {
		'kind':'full','protocolVersion':3,'schemaVersion':'freeze-v02-interfaces/1.2',
		'worldId':'grey_hive','sceneId':'gh_deep_decon','checkpointId':null,
		'worldEpoch':1,'serverTick':1,'authorityRevision':1,'ackSeq':0,'entryToken':null,
		'player':{'entityId':'player','transform':_transform(x,0,z),'velocityMps':_point(0,0,0),
			'actionState':'idle','currentHp':100,'maxHp':100,'currentEnergy':100,'maxEnergy':100,
			'facingX':0,'facingZ':1,'aimX':0,'aimZ':1},
		'actors':[],'doors':[],'interactables':[],'hazards':[hazard],'objectives':[],
		'capabilities':{'schemaVersion':1,'items':[]},
		'progression':{'schemaVersion':1,'eventSeq':0,'currentWorldId':'grey_hive','worlds':[]},
		'greyHiveBeacon':{'state':'uncollected','legacyCompletedWithoutReceipt':false}
	}

func _phases_and_geometry() -> void:
	for which in range(SPECS.size()):
		var spec: Array = SPECS[which]
		var line: String = spec[3]+'：危险'
		for phase in PHASES:
			_expect(spec[0]+'/'+phase,_fixture(which,phase),BASE+[spec[3]+'：'+PHASES[phase]])
			_expect(spec[0]+'/ineligible/'+phase,_fixture(which,phase),[],false)
		_expect(spec[0]+'/nonboolean eligible',_fixture(which),[],1)
		for y in [0.0,1.0,1.0001,-0.0001]:
			var s: Dictionary = _fixture(which)
			s.player.transform.positionM.yM = y
			_expect(spec[0]+'/height/'+str(y),s,BASE+([line] if y>=0 and y<=1 else []))
		var a: Array = spec[4][0]
		var b: Array = spec[4][2]
		var cx: float = (a[0]+b[0])*0.5
		var cz: float = (a[1]+b[1])*0.5
		var width: float = b[0]-a[0]
		var depth: float = b[1]-a[1]
		# Native cross-product tolerance, not a normalized distance tolerance.
		var points: Array = [
			[a[0],a[1],true],[b[0],a[1],true],[b[0],b[1],true],[a[0],b[1],true],
			[cx,a[1],true],[b[0],cz,true],[cx,b[1],true],[a[0],cz,true],
			[cx,a[1]-0.000005/width,true],[cx,a[1]-0.00002/width,false],
			[a[0]-0.000005/depth,cz,true],[a[0]-0.00002/depth,cz,false],
			[a[0]-0.00002,a[1],false],[cx,a[1]-1,false]
		]
		for n in range(points.size()):
			var s: Dictionary = _fixture(which)
			s.player.transform.positionM = _point(points[n][0],0,points[n][1])
			_expect(spec[0]+'/edge/'+str(n),s,BASE+([line] if points[n][2] else []))

func _hazard_rejections() -> void:
	for which in range(SPECS.size()):
		var wrong_polygon: Array = SPECS[which][4].duplicate(true)
		wrong_polygon[0][0] -= 0.25
		var mutations: Array = [
			['entityId','unrelated_zone'],['kind',SPECS[1-which][1]],
			['polygonM',wrong_polygon],['heightRangeM',[0,1]],
			['heightRangeM',null],['phaseActive',false],['phaseActive',1],['active',false],
			['environment',null]
		]
		for n in range(mutations.size()):
			var s: Dictionary = _fixture(which)
			s.hazards[0][mutations[n][0]] = mutations[n][1]
			_expect(str(which)+'/hazard rejection/'+str(n),s,BASE)
		for field in ['environment','polygonM','phaseActive']:
			var s: Dictionary = _fixture(which)
			s.hazards[0].erase(field)
			_expect(str(which)+'/missing/'+field,s,BASE)
		for mutation in [['tag',SPECS[1-which][2]],['phase','unknown'],['remainingMs',0],['remainingMs',true],['exposureBps',1]]:
			var s: Dictionary = _fixture(which)
			s.hazards[0].environment[mutation[0]] = mutation[1]
			_expect(str(which)+'/environment/'+str(mutation),s,BASE)

func _context_and_schema() -> void:
	var s: Dictionary = _fixture()
	s.entryToken = {'worldId':'grey_hive','sceneId':'gh_deep_decon','worldEpoch':1,'generation':1}
	_expect('pending entry',s,[])
	s = _fixture()
	s.sceneId = 'gh_beacon'
	_expect('other room',s,BASE)
	s = _fixture()
	s.worldId = 'return_station'
	s.sceneId = 'rs_core_room'
	s.progression.currentWorldId = 'return_station'
	_expect('other world',s,[])
	for mutation in [['kind','delta'],['schemaVersion','invalid'],['protocolVersion',true],['serverTick',-1]]:
		s = _fixture()
		s[mutation[0]] = mutation[1]
		_expect('invalid envelope/'+str(mutation),s,[],true,false)
	s = _fixture()
	s.player.maxHp = 0
	_expect('invalid player preserves beacon',s,BASE,true,false)
	s = _fixture()
	s.erase('checkpointId')
	_expect('missing checkpoint preserves beacon',s,BASE,true,false)

func _beacons() -> void:
	var cases: Array = [
		['uncollected',false,'信标：尚未收取'],['carried',false,'信标：已携带'],
		['mounted',false,'信标：已部署'],['uncollected',true,'信标：收取回执未确认'],
		['carried',true,'信标：状态未确认']
	]
	for which in range(SPECS.size()):
		for item in cases:
			var s: Dictionary = _fixture(which)
			s.greyHiveBeacon = {'state':item[0],'legacyCompletedWithoutReceipt':item[1]}
			_expect('beacon/'+str(which)+'/'+str(item),s,[item[2],SPECS[which][3]+'：危险'])
			s.hazards[0].erase('environment')
			_expect('beacon with absent environment/'+str(which)+'/'+str(item),s,[item[2]])
		for invalid in [null,{}, {'state':'unknown','legacyCompletedWithoutReceipt':false}, {'state':'mounted','legacyCompletedWithoutReceipt':false,'extra':0}]:
			var s: Dictionary = _fixture(which)
			s.greyHiveBeacon = invalid
			_expect('invalid beacon/'+str(which)+'/'+str(invalid),s,['信标：状态未确认',SPECS[which][3]+'：危险'])
