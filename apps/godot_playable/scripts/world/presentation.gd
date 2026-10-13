extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
static func signal_positions(a: Dictionary, s: Dictionary) -> Array:
	var counts = {'mh_tidal_warehouse':1,'mh_signal_yard':3,'mh_breakwater':2,'mh_resonance_tower':4}
	if s.worldId!='mist_harbor' or a.entityType!='enemy.mist_harbor.signal_wraith' or a.actorKind!='enemy':
		return []
	var known = false
	for i in range(counts.get(s.sceneId,0)):
		if a.entityId==s.sceneId+'_signal_wraith_0'+str(i+1):
			known = true
	var v = a.get('signalPerception')
	if not known or not v is Dictionary or v.size()!=3 or typeof(v.get('precise'))!=TYPE_BOOL or not P.number(v.get('uncertaintyRadiusM'),0,2) or not P.rows(v.get('positionsM'),2) or v.positionsM.size()!=(1 if v.precise else 2):
		return []
	var center = P.vector(a.transform.positionM)
	for p in v.positionsM:
		if not P.point(p) or abs(float(p.yM)-center.y)>0.0001:
			return []
	if v.precise:
		if v.uncertaintyRadiusM!=0 or P.vector(v.positionsM[0]).distance_to(center)>0.0001:
			return []
	else:
		var first = P.vector(v.positionsM[0])
		var second = P.vector(v.positionsM[1])
		if v.uncertaintyRadiusM<=0 or abs(first.distance_to(second)-v.uncertaintyRadiusM)>0.0001 or ((first+second)*0.5).distance_to(center)>0.0001:
			return []
	return v.positionsM
static func swarm_members(a: Dictionary) -> Array:
	var members = a.get('members')
	if not P.rows(members,4) or members.size()!=4:
		return []
	var center = P.vector(a.transform.positionM)
	var alive = false
	for n in range(4):
		var m = members[n]
		if not m is Dictionary or m.get('memberId')!=a.entityId+'/member/'+str(n+1) or typeof(m.get('active'))!=TYPE_BOOL or not P.point(m.get('positionM')) or not P.number(m.get('radiusM'),0.001,1):
			return []
		var pos = P.vector(m.positionM)
		if abs(pos.y-center.y)>0.0001 or Vector2(pos.x-center.x,pos.z-center.z).length()+m.radiusM>1.0001:
			return []
		alive = alive or m.active
	return members if a.active==alive else []
static func actor_positions(a: Dictionary, s: Dictionary) -> Array:
	if a.entityType=='enemy.mist_harbor.signal_wraith':
		return signal_positions(a,s)
	if a.entityType=='enemy.grey_hive.swarm':
		var out: Array = []
		for m in swarm_members(a):
			if m.active:
				out.append(m.positionM)
		return out
	return [a.transform.positionM]
static func mapping(s: Dictionary) -> bool:
	var c: Dictionary = s.capabilities
	if c.has('acousticMappingAuthorized'):
		return typeof(c.acousticMappingAuthorized)==TYPE_BOOL and c.acousticMappingAuthorized
	return P.granted(s,'perception.acoustic_mapping_i',true)
static func support(v, s: Dictionary, scene_hash: String) -> Dictionary:
	if not v is Dictionary or not P.counter(v.get('schemaVersion'),1,1) or not P.context(v) or not P.counter(v.get('serverTick')) or not P.counter(v.get('authorityRevision')) or not P.same(v,s) or v.get('serverTick')!=s.serverTick or v.get('authorityRevision')!=s.authorityRevision or v.get('sceneSourceSha256')!=scene_hash or not P.counter(v.get('serverTimeMs')) or not P.rows(v.get('poses'),64):
		return {}
	var out = {}
	var previous = ''
	for pose in v.poses:
		if not pose is Dictionary or not P.ident(pose.get('supportId')) or pose.supportId<=previous or not pose.get('phase') in ['lower_hold','rising','upper_hold','falling','stationary'] or not P.number(pose.get('heightM'),0,3) or not P.number(pose.get('velocityMps'),-12,12) or not P.counter(pose.get('phaseElapsedMs'),0,60000):
			return {}
		if pose.phase=='rising' and pose.velocityMps<=0 or pose.phase=='falling' and pose.velocityMps>=0 or pose.phase in ['lower_hold','upper_hold','stationary'] and pose.velocityMps!=0 or pose.phase=='stationary' and pose.phaseElapsedMs!=0:
			return {}
		previous = pose.supportId
		out[pose.supportId] = pose
	var rider = v.get('rider')
	if not rider is Dictionary or not P.point(rider.get('positionM')) or P.vector(rider.positionM).distance_to(P.vector(s.player.transform.positionM))>0.0001 or not P.number(rider.get('radiusM'),0.001,4) or not rider.get('mode') in ['surface','floor','airborne','traversal']:
		return {}
	if rider.mode=='surface':
		if not out.has(rider.get('supportId')) or abs(float(rider.positionM.yM)-out[rider.supportId].heightM)>0.0001:
			return {}
	elif rider.get('supportId')!=null or rider.mode=='floor' and abs(float(rider.positionM.yM))>0.0001:
		return {}
	return out
static func environment(h: Dictionary) -> Dictionary:
	var e = h.get('environment')
	var tags = {'heat_zone':'heat','steam_jet':'pressure','decon_mist':'toxin'}
	if not e is Dictionary or not e.get('phase') in ['warning','active','recovery','suppressed'] or not tags.has(h.kind) or tags[h.kind]!=e.get('tag') or not P.counter(e.get('remainingMs'),1,60000) or not P.counter(e.get('exposureBps'),0,10000) or e.tag!='heat' and e.exposureBps!=0 or e.phase=='warning' and e.remainingMs>10000 or h.active!=(e.phase=='active') or h.get('phaseActive')!=true or not P.polygon(h.get('polygonM')):
		return {}
	return e
static func sentinel(s: Dictionary) -> Dictionary:
	if s.worldId!='grey_hive' or s.sceneId!='gh_sentinel_arena':
		return {}
	var matches: Array = []
	for a in s.actors:
		if a.entityId=='gh_sentinel_arena_sentinel_01' or a.entityType=='enemy.grey_hive.sentinel':
			matches.append(a)
	var v = s.get('sentinelEncounter')
	if matches.size()!=1 or matches[0].entityId!='gh_sentinel_arena_sentinel_01' or matches[0].entityType!='enemy.grey_hive.sentinel' or matches[0].actorKind!='sentinel' or not v is Dictionary or v.size()!=6 or v.get('actorId')!=matches[0].entityId or not P.point(v.get('positionM')) or P.vector(v.positionM).distance_to(P.vector(matches[0].transform.positionM))>0.001 or not P.counter(v.get('attackSerial')) or not P.counter(v.get('remainingMs'),0,60000) or not v.get('phase') in ['chase','light_windup','heavy_windup','charge_windup','charge','recover','hit','stagger','dead','inactive']:
		return {}
	if (v.phase in ['dead','inactive'])==matches[0].active:
		return {}
	if v.phase in ['chase','dead','inactive'] and v.remainingMs!=0 or v.phase=='hit' and v.remainingMs>134 or v.phase=='stagger' and v.remainingMs>884 or v.phase=='recover' and v.remainingMs>500:
		return {}
	if v.phase in ['light_windup','heavy_windup','charge_windup','charge']:
		var cap = {'light_windup':250,'heavy_windup':400,'charge_windup':884,'charge':600}[v.phase]
		var w = v.get('warning')
		if v.remainingMs<=0 or v.remainingMs>cap or not w is Dictionary or w.size()!=5 or not P.point(w.get('originM')) or not P.number(w.get('directionRad'),-PI-0.001,PI+0.001) or not P.number(w.get('radiusM'),0.001,3) or not P.number(w.get('rangeM'),0,3.601):
			return {}
		if v.phase in ['light_windup','heavy_windup']:
			if w.get('shape')!='circle' or abs(w.radiusM-(1.8 if v.phase=='light_windup' else 2.2))>0.001 or w.rangeM!=0 or w.directionRad!=0 or P.vector(w.originM).distance_to(P.vector(v.positionM))>0.001:
				return {}
		else:
			if w.get('shape')!='corridor' or abs(w.radiusM-0.6)>0.001 or w.rangeM<=0 or v.attackSerial<=0:
				return {}
			var d = P.vector(v.positionM)-P.vector(w.originM)
			var along = d.x*sin(w.directionRad)+d.z*cos(w.directionRad)
			var side = d.x*cos(w.directionRad)-d.z*sin(w.directionRad)
			if abs(side)>0.001 or along < -0.001 or along>w.rangeM+0.001 or abs(d.y)>0.001 or v.phase=='charge_windup' and d.length()>0.001:
				return {}
	elif v.get('warning')!=null:
		return {}
	return v
