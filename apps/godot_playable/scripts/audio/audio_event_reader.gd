extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const V = preload('res://scripts/world/presentation.gd')
const E = preload('res://scripts/world/event_view.gd')
const L = preload('res://scripts/audio/audio_cue_library.gd')
var context_key = ''
var cursor = 0
var seeded = false
var pending: Array = []
var terminals: Dictionary = {}
var last_tick = 0
func suspend() -> void:
 pending.clear()
 terminals.clear()
 seeded = false
func sync(s: Dictionary, generation: int, eligible: bool) -> void:
 var key = str(generation)+':'+str(s.get('worldEpoch'))+':'+str(s.get('worldId'))+':'+str(s.get('sceneId'))
 if key!=context_key or s.get('serverTick',0)<last_tick:
  context_key = key
  cursor = 0
  suspend()
 last_tick = int(s.get('serverTick',0))
 if not eligible:
  suspend()
func envelope(e) -> bool:
 return e is Dictionary and P.counter(e.get('protocolVersion'),2,2) and P.counter(e.get('eventId'),1) and P.counter(e.get('worldEpoch')) and P.counter(e.get('serverTick')) and P.ident(e.get('kind')) and P.point(e.get('positionM')) and P.number(e.get('directionRad'),-10000,10000) and P.number(e.get('radiusM'),0,10000) and P.number(e.get('intensity'),0,2) and (not e.has('durationMs') or P.counter(e.durationMs,1,60000))
func actor(id, s: Dictionary) -> Dictionary:
 var matches: Array = s.get('actors',[]).filter(func(a): return a.get('entityId')==id)
 return matches[0] if matches.size()==1 else {}
func identity(v) -> bool:
 if typeof(v)!=TYPE_STRING or v.is_empty() or v.length()>256:
  return false
 for c in v:
  if not c in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.:/-':
   return false
 return true
func authorized(e: Dictionary, s: Dictionary, scene: Dictionary) -> bool:
 if not envelope(e) or s.is_empty() or scene.is_empty() or e.worldEpoch!=s.worldEpoch or scene.get('worldId')!=s.worldId or scene.get('sceneId')!=s.sceneId:
  return false
 var b: Dictionary = scene.boundsM
 var p = P.vector(e.positionM)
 if p.x<b.x or p.x>b.x+b.width or p.z<b.z or p.z>b.z+b.depth or p.y<0 or p.y>3.001:
  return false
 if e.kind in E.COMBAT:
  var f = e.get('combatFeedback')
  if not f is Dictionary or f.get('worldId')!=s.worldId or f.get('sceneId')!=s.sceneId or f.get('outcome')!=E.COMBAT[e.kind] or not P.counter(e.get('durationMs'),1,2000) or f.get('requestId')!=null and not P.counter(f.requestId,1):
   return false
  if f.outcome=='rejected':
   return f.get('sourceId')=='player' and P.counter(f.get('requestId'),1) and identity(f.get('reason'))
  if not identity(f.get('sourceId')) or not identity(f.get('targetId')):
   return false
  if f.outcome=='absorbed':
   return f.targetId=='player' and not actor(f.sourceId,s).is_empty() and f.get('reason') in ['invulnerable','damage_reduced_to_zero']
  if f.get('reason')!=null:
   return false
  var a = actor(f.targetId if f.outcome=='enemy_hit' else f.sourceId,s)
  if a.is_empty() or not a.get('actorKind') in ['enemy','sentinel'] or not P.point(a.get('transform',{}).get('positionM')):
   return false
  if a.entityType=='enemy.mist_harbor.signal_wraith' and (V.signal_positions(a,s).is_empty() or not a.signalPerception.precise):
   return false
  return f.sourceId=='player' and f.targetId!='player' if f.outcome=='enemy_hit' else f.targetId=='player'
 if e.kind in E.ENEMY:
  var a = actor(e.get('actorId'),s)
  if a.is_empty() or a.actorKind!='enemy' or not P.point(a.transform.positionM) or not P.counter(e.get('durationMs'),1,60000) or not P.number(e.radiusM,0.001,64):
   return false
  if e.kind=='EnemyDeath' and a.active or e.kind not in ['EnemyDeath','EnemyMemberDisperse'] and not a.active:
   return false
  var allowed = {'enemy.clockworks.pressure_drone':['pressure_shot'],'enemy.clockworks.furnace_hound':['bite','leap'],'enemy.grey_hive.brute':['charge','slam'],'enemy.grey_hive.swarm':['lunge'],'enemy.mist_harbor.tidebound':['tide_swing','tide_charge'],'enemy.mist_harbor.signal_wraith':['signal_shot']}
  if not allowed.has(a.entityType) or not enemy_source(a,s,scene):
   return false
  var attack = e.get('attackKind')
  if attack!=null and not attack in allowed[a.entityType]:
   return false
  var attack_row = e.kind in ['EnemyAttackTelegraph','EnemyAttackImpact'] or 'Motion' in e.kind
  if attack_row and attack==null or e.has('rangeM') and not P.number(e.rangeM,0,128) or e.kind=='EnemyAttackTelegraph' and not P.number(e.get('rangeM'),0.001,128):
   return false
  if attack=='tide_swing':
   if not P.number(e.get('halfAngleRad'),0.001,PI):
    return false
  elif e.get('halfAngleRad')!=null:
   return false
  if attack in ['tide_swing','tide_charge'] and not P.number(e.get('rangeM'),0.001,128):
   return false
  if a.entityType=='enemy.mist_harbor.signal_wraith' and (V.signal_positions(a,s).is_empty() or attack_row and e.kind!='EnemyAttackImpact' and not a.signalPerception.precise):
   return false
  if e.kind in ['EnemyMemberHit','EnemyMemberDisperse']:
   if attack!=null:
    return false
   for m in V.swarm_members(a):
    if m.memberId==e.get('memberId') and m.active==(e.kind=='EnemyMemberHit'):
     return true
   return false
  if e.get('memberId')!=null:
   return false
  var motion = {'PressureShotMotion':'pressure_shot','FurnaceHoundLeapMotion':'leap','SignalShotMotion':'signal_shot','EnemyLungeMotion':'lunge'}
  if motion.has(e.kind) and attack!=motion[e.kind] or e.kind=='EnemyChargeMotion' and not attack in ['charge','tide_charge']:
   return false
  if e.kind=='SignalBlink' and (a.entityType!='enemy.mist_harbor.signal_wraith' or attack!=null or not P.number(e.get('rangeM'),0.001,2)):
   return false
  return true
 if e.kind.begins_with('Sentinel'):
  var v = V.sentinel(s)
  var phase = {'SentinelAttackWindup':'light_windup','SentinelHeavyWindup':'heavy_windup','SentinelChargeWindup':'charge_windup'}.get(e.kind,'')
  return not v.is_empty() and phase!='' and v.phase==phase and e.get('actorId')==v.actorId and P.counter(e.get('attackId')) and e.attackId==v.attackSerial and P.vector(e.positionM).distance_to(P.vector(v.warning.originM))<=0.001 and abs(e.directionRad-v.warning.directionRad)<=0.001 and abs(e.radiusM-v.warning.radiusM)<=0.001 and P.number(e.get('rangeM')) and abs(e.rangeM-v.warning.rangeM)<=0.001
 if e.kind.begins_with('ResonanceWarden'):
  var a = actor('mh_resonance_warden_staged_marker',s)
  return L.PRESENTATION_AUDIO_IDS.has(e.kind) and s.worldId=='mist_harbor' and s.sceneId=='mh_warden_arena' and not a.is_empty() and a.entityType=='runtime2d.enemy.mistharbor.signal_wraith.v2' and a.active==(e.kind!='ResonanceWardenDeath')
 return e.kind in ['AttackStarted','DashStarted','PulseCast','GuardStarted','GuardEnded','PierceStarted']
func enemy_source(a: Dictionary, s: Dictionary, scene: Dictionary) -> bool:
 if a.entityType in ['enemy.clockworks.pressure_drone','enemy.clockworks.furnace_hound']:
  if s.worldId!='clockworks':
   return false
  var matches: Array = scene.get('spawns',[]).filter(func(row): return row.get('id')==a.entityId and row.get('entityType')==a.entityType and row.get('kind')=='enemy')
  return matches.size()==1
 if a.entityType=='enemy.grey_hive.brute':
  return s.worldId=='grey_hive' and s.sceneId=='gh_gate_b' and a.entityId=='gh_gate_b_brute_01'
 if a.entityType=='enemy.grey_hive.swarm':
  return s.worldId=='grey_hive' and s.sceneId in ['gh_lockdown','gh_deep_decon'] and a.entityId in [s.sceneId+'_swarm_01',s.sceneId+'_swarm_02'] and not V.swarm_members(a).is_empty()
 if a.entityType=='enemy.mist_harbor.signal_wraith':
  return not V.signal_positions(a,s).is_empty()
 var count = {'mh_drowned_quay':1,'mh_breakwater':2,'mh_resonance_tower':1}.get(s.sceneId,0)
 for n in range(count):
  if s.worldId=='mist_harbor' and a.entityId==s.sceneId+'_tidebound_0'+str(n+1):
   return true
 return false
func accept(rows, s: Dictionary, generation: int, eligible: bool) -> bool:
 sync(s,generation,eligible)
 if not P.rows(rows,2048):
  suspend()
  return false
 var ordered: Array = []
 for e in rows:
  if not envelope(e):
   suspend()
   return false
  ordered.append(e)
 ordered.sort_custom(func(a,b): return a.eventId<b.eventId)
 var seed_only = not seeded or not eligible
 for e in ordered:
  if e.worldEpoch!=s.worldEpoch or e.eventId<=cursor:
   continue
  cursor = int(e.eventId)
  if seed_only:
   continue
  if pending.size()>=256:
   suspend()
   return false
  pending.append({'event':e.duplicate(true),'accepted':Time.get_ticks_msec(),'initial_age':maxf(0,(s.serverTick-e.serverTick)*1000.0/60)})
 seeded = eligible
 return true
func age(row: Dictionary, s: Dictionary) -> float:
 return maxf((s.serverTick-row.event.serverTick)*1000.0/60,row.initial_age+Time.get_ticks_msec()-row.accepted)
func dispatch(s: Dictionary, scene: Dictionary) -> Array:
 var out: Array = []
 var future: Array = []
 for row in pending:
  var e: Dictionary = row.event
  if e.serverTick>s.serverTick:
   if Time.get_ticks_msec()-row.accepted<1000:
    future.append(row)
   continue
  if not authorized(e,s,scene):
   continue
  var key: String = str(e.get('actorId','player' if e.kind=='GuardEnded' or e.kind=='GuardStarted' else ''))
  var terminal = e.kind in ['EnemyAttackImpact','EnemyStagger','EnemyDeath']
  if terminal:
   terminals[key] = e.serverTick
   if terminals.size()>256:
    terminals.erase(terminals.keys()[0])
  elif e.kind=='EnemyAttackTelegraph' or 'Motion' in e.kind:
   if terminals.get(key,-1)>=e.serverTick:
    continue
  row['cue'] = L.cue_for(e)
  row['key'] = key
  row['stop'] = terminal or 'Motion' in e.kind or e.kind in ['GuardEnded','EnemyAttackTelegraph']
  row['duration'] = minf(L.duration_ms(row.cue),float(e.get('durationMs',L.duration_ms(row.cue))))
  if row.stop or age(row,s)<row.duration:
   out.append(row)
 pending = future
 return out
