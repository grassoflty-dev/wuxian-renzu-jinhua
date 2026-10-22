extends SceneTree
# SOURCE ONLY / FUTURE COMMAND NOT_RUN; separate execution authority required.
# godot --headless --path "C:/Users/GWL/Desktop/wuxian-renzu-jinhua/apps/godot_playable" --script res://tests/ui/baizhi_choice_hud_contract.gd
const P = preload('res://scripts/bridge/protocol.gd')
const Content = preload('res://scripts/ui/content_model.gd')
const Hud = preload('res://scripts/ui/play_status_model.gd')
const Fixture = preload('res://tests/ui/decon_hazard_text_contract.gd')
const BASE = ['✓ 主电恢复','○ 隔离封锁解除','○ 灰巢撤离']
const CHOICES = {'taken':'已记录的白芷选择：我会帮你离开这里。','left':'已记录的白芷选择：我不能带你走。'}
var _failures: int = 0

func _initialize() -> void:
	_choices()
	_progress_gates()
	_envelope_gates()
	_other_worlds()
	print('BAIZHI_CHOICE_HUD_CONTRACT failures='+str(_failures))
	quit(0 if _failures==0 else 1)

static func _fixture(choice: Variant='unresolved') -> Dictionary:
	var s: Dictionary = Fixture._fixture()
	s.progression.worlds = [{'worldId':'grey_hive','visitId':1,'cycleId':1,
		'revisitCount':0,'completed':false,'firstCompletion':false,
		'completedEvents':['hive_power'],'hiveChoice':choice}]
	return s

func _check(ok: bool, label: String) -> void:
	if not ok:
		_failures += 1
		push_error(label)

func _expect(label: String, s: Dictionary, lines: Array, eligible: bool=true,
		valid: bool=true, objectives: Array=[], action: Dictionary={}) -> void:
	var before: Dictionary = s.duplicate(true)
	_check(P.snapshot(s)==valid,label+'/snapshot validity')
	var expected: Dictionary = {'objectives':objectives,'milestones':lines,'action':action}
	for repeat in range(2):
		_check(Hud.project(s,eligible)==expected,label+'/projection/'+str(repeat))
	_check(s==before,label+'/input unchanged')

func _choices() -> void:
	var baseline: Dictionary = _fixture()
	_check(P.snapshot(baseline),'baseline full snapshot')
	_check(not Content.progress(baseline,'grey_hive').is_empty(),'baseline valid progression')
	_expect('baseline three milestones',baseline,BASE)
	for choice in CHOICES:
		var s: Dictionary = _fixture(choice)
		_check(not s.has('npcs') and not s.has('baizhi') and s.actors.is_empty(),'no NPC/HP prerequisite')
		_expect(choice+' exactly once',s,BASE+[CHOICES[choice]])
	for choice in ['unresolved','unknown','Taken','taken ',null,true,1,1.0,[],{},&'taken']:
		_expect('choice type/value '+str(choice),_fixture(choice),BASE)
	var s: Dictionary = _fixture()
	s.progression.worlds[0].erase('hiveChoice')
	_expect('missing choice',s,BASE)
	s = _fixture('taken')
	s.baizhi = {'schemaVersion':1,'choice':'left','available':false,'canInteract':false}
	s.npcs = []
	_expect('unavailable NPC does not override recorded decision',s,BASE+[CHOICES.taken])
	s.npcs = [{'entityId':'gh_bz_whitezhi_v1','entityType':'npc.baizhi','position':[8,0,10.5],'yawRad':PI,'interactable':false}]
	_expect('NPC row without HP does not override decision',s,BASE+[CHOICES.taken])
	s = _fixture('left')
	s.progression.worlds[0].completedEvents = ['hive_power','hive_lockdown','hive_extraction']
	_expect('all original event states',s,['✓ 主电恢复','✓ 隔离封锁解除','✓ 灰巢撤离',CHOICES.left])
	s = _fixture('taken')
	s.objectives = [{'objectiveId':'gh_restore_main_power','state':'active','positionM':s.player.transform.positionM.duplicate(true)}]
	s.player.actionState = 'pulse'
	s.player.actionPresentation = {'requestId':1,'phase':'active','durationMs':100,'elapsedMs':25,'rangeM':1,'lineHalfWidthM':0}
	_expect('objective/action preserved',s,BASE+[CHOICES.taken],true,true,
		['恢复主电 · 待完成'],{'kind':'pulse','phase':'active','remainingMs':75,'text':'执行 · 75ms'})

func _progress_gates() -> void:
	var s: Dictionary = _fixture('taken')
	s.progression.worlds[0].visitId = 0
	s.progression.worlds[0].completedEvents = []
	_expect('unvisited Grey Hive',s,[])
	s = _fixture('taken')
	s.progression.worlds.append(s.progression.worlds[0].duplicate(true))
	_expect('duplicate world',s,[])
	for mutation in [['visitId',true],['cycleId',0],['revisitCount',-1],['completed',1],
			['firstCompletion',null],['completedEvents',['unknown']],
			['completedEvents',['hive_power','hive_power']]]:
		s = _fixture('taken')
		s.progression.worlds[0][mutation[0]] = mutation[1]
		_expect('invalid progress row '+str(mutation),s,[])
	for mutation in [['schemaVersion',true],['eventSeq',-1],['worlds',null]]:
		s = _fixture('taken')
		s.progression[mutation[0]] = mutation[1]
		_expect('invalid progression '+str(mutation),s,[],true,false)
	s = _fixture('taken')
	s.progression.currentWorldId = 'mist_harbor'
	_expect('current world mismatch keeps event milestones',s,BASE)
	s = _fixture('taken')
	s.entryToken = {'worldId':s.worldId,'sceneId':s.sceneId,'worldEpoch':s.worldEpoch,'generation':1}
	_expect('pending entry keeps event milestones',s,BASE)
	_expect('ineligible',_fixture('taken'),[],false)

func _envelope_gates() -> void:
	for mutation in [['kind','delta'],['schemaVersion','invalid'],['serverTick',-1]]:
		var s: Dictionary = _fixture('taken')
		s[mutation[0]] = mutation[1]
		_expect('invalid snapshot keeps milestones '+str(mutation),s,BASE,true,false)
	var s: Dictionary = _fixture('taken')
	s.player.erase('currentHp')
	_expect('missing player HP keeps milestones',s,BASE,true,false)
	s = _fixture('taken')
	s.erase('checkpointId')
	_expect('missing checkpoint keeps milestones',s,BASE,true,false)
	for mutation in [['protocolVersion',true],['worldEpoch',true]]:
		s = _fixture('taken')
		s[mutation[0]] = mutation[1]
		_expect('project numeric type guard '+str(mutation),s,[],true,false)
	for mutation in [['worldId','unknown_world'],['sceneId','unknown_scene']]:
		s = _fixture('taken')
		s[mutation[0]] = mutation[1]
		_expect('project known context guard '+str(mutation),s,[])

func _other_worlds() -> void:
	for spec in [
		['mist_harbor','mh_fog_pier','mist_beacon_west',['✓ 西侧航标','○ 东侧航标','○ 信号定位']],
		['clockworks','cw_entry_foundry','clockworks_valves',['✓ 压力阀门','○ 调节者核心','○ 停机结算']]
	]:
		var s: Dictionary = _fixture('taken')
		var row: Dictionary = s.progression.worlds[0].duplicate(true)
		row.worldId = spec[0]
		row.completedEvents = [spec[2]]
		row.erase('hiveChoice')
		s.progression.worlds.append(row)
		s.worldId = spec[0]
		s.sceneId = spec[1]
		s.progression.currentWorldId = spec[0]
		_expect(spec[0]+' no Grey Hive choice leak',s,spec[3])
	var s: Dictionary = _fixture('taken')
	s.worldId = 'return_station'
	s.sceneId = 'rs_core_room'
	s.progression.currentWorldId = 'return_station'
	_expect('Return Station keeps existing summary only',s,['灰巢设施 · 未完成'])
	s.progression.worlds[0].visitId = 0
	s.progression.worlds[0].completedEvents = []
	s.progression.worlds[0].hiveChoice = 'unresolved'
	_expect('Return Station valid unvisited summary',s,['灰巢设施 · 未完成'])
