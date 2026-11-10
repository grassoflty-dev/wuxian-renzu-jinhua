extends RefCounted
# Information-zone membership only. Existing structural rendering stays owned by its scene.
const P = preload('res://scripts/bridge/protocol.gd')
const SOURCE = 'res://data/gh_floor_route_regions.local.json'
const HASHES = {
	'gh_entry_maintenance':['4bf7a69a72e714a8de36c683f8b04fa7b1c89bbb9a358b2735dc7d2775915eaf'],
	'gh_central_shaft':['6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832'],
	'gh_lockdown':['23e5d5d9f37f6f44b1f8b9105aab656424778ef5edf0e775bbc364143fe1b30c','718489a1ea4ec4622ccdcefc8459a1dc1bd4f66ccf26abc813b6a06519197850'],
	'gh_gate_b':['a82c10a6fa50129c48267e255d2ca86f4ebaca43e0e0da81b639ee88228312c1'],
	'gh_sentinel_arena':['3e3e3a52dca003ec13c7b3698317f6a32627091eeda833a49896910e521f45bb']}
var data: Dictionary = {}
var attempted = false

func _load() -> bool:
	if not attempted:
		attempted=true
		var decoded = JSON.parse_string(FileAccess.get_file_as_string(SOURCE))
		if decoded is Dictionary: data=decoded
	return data.get('schema')=='gh-floor-route-info/v1' and data.get('variants') is Dictionary and data.variants.size()==13

func scope(snapshot:Dictionary,hash:String) -> bool:
	if snapshot.get('worldId')!='grey_hive' or not HASHES.has(snapshot.get('sceneId')) or not hash in HASHES[snapshot.sceneId]: return false
	if snapshot.sceneId!='gh_central_shaft': return true
	if snapshot.get('shaftLiftDeferred')==true and snapshot.get('shaftLift')==null and snapshot.get('supportScene')==null: return true
	var support=snapshot.get('supportScene')
	if snapshot.get('shaftLift') is Dictionary and support is Dictionary and support.get('rider') is Dictionary and P.point(support.rider.get('positionM')): return true
	var visible=snapshot.get('currentVisibleRegions')
	return visible is Array and not visible.is_empty() and typeof(visible[0])==TYPE_STRING and visible[0].begins_with('grey_hive.gh_central_shaft.upper_')

func requires_floor_region(snapshot:Dictionary) -> bool:
	if snapshot.get('sceneId')!='gh_central_shaft': return true
	# Lack of a fresh service-floor observation hides information, not movement.
	if snapshot.get('shaftLowerService') is Dictionary and (not snapshot.get('currentVisibleRegions') is Array or snapshot.currentVisibleRegions.is_empty()): return false
	if snapshot.get('shaftLiftDeferred')==true and snapshot.get('shaftLift')==null and snapshot.get('supportScene')==null: return false
	var support=snapshot.get('supportScene')
	if not support is Dictionary or not support.get('rider') is Dictionary: return true
	var mode=support.rider.get('mode')
	if mode in ['surface','airborne','traversal']: return false
	return true

func prepare(snapshot:Dictionary,scene:Dictionary,hash:String,events:Array) -> Dictionary:
	if not _load() or not P.snapshot(snapshot) or not scope(snapshot,hash) or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or not snapshot.get('denseEntityRegions') is Dictionary: return {}
	var visible=snapshot.get('currentVisibleRegions')
	if not visible is Array or visible.size()!=1 or typeof(visible[0])!=TYPE_STRING or snapshot.get('denseOccupiedRegion')!=visible[0]: return {}
	var matches: Array = []
	for key in data.variants:
		var variant=data.variants[key]
		if not variant is Dictionary or variant.get('scene')!=snapshot.sceneId or not variant.get('parents') is Array: continue
		if snapshot.sceneId=='gh_central_shaft':
			var lift=snapshot.get('shaftLift');var support=snapshot.get('supportScene')
			if snapshot.get('shaftLiftDeferred')==true and lift==null and support==null:
				if key!='shaft_deferred' or snapshot.player.transform.positionM.yM!=0.0: continue
			else:
				if not lift is Dictionary or not support is Dictionary or not support.get('rider') is Dictionary or support.rider.get('mode')!='floor' or not P.point(support.rider.get('positionM')) or support.rider.positionM!=snapshot.player.transform.positionM: continue
				if support.rider.positionM.yM==0.0:
					if typeof(lift.get('upperGateOpen'))!=TYPE_BOOL or key!=('shaft_active_open' if lift.upperGateOpen else 'shaft_active_closed'): continue
				elif support.rider.positionM.yM==-4.0:
					if typeof(lift.get('lowerGateOpen'))!=TYPE_BOOL or key!=('shaft_lower_open' if lift.lowerGateOpen else 'shaft_lower_closed'): continue
				else: continue
		if snapshot.sceneId=='gh_lockdown' and (key.begins_with('lockdown_legacy_') != (hash==HASHES.gh_lockdown[1])): continue
		for parent in variant.parents:
			if parent is Dictionary and parent.get('id')==visible[0] and parent.get('available')==true: matches.append({'key':key,'variant':variant,'parent':parent})
	# Current native region and selected event phase must agree; duplicate phase rows
	# are resolved only by the actual effective state, never by arbitrary ordering.
	if snapshot.sceneId in ['gh_gate_b','gh_lockdown']:
		var suffix='_open' if events.has('hive_lockdown') else '_closed'
		matches=matches.filter(func(m): return str(m.key).ends_with(suffix))
	if matches.size()!=1: return {}
	var selected:Dictionary=matches[0]
	if not scene.get('collision') is Array or not P.unique(scene.collision,'id') or not selected.variant.get('owners') is Array: return {}
	var count=0
	for owner in selected.variant.owners:
		if not owner is Dictionary or owner.get('role')!='collision_owner' or owner.get('id') in ['well_west','well_east','well_north','upper_landing_gate']: continue
		var rect=owner.get('rect_x0_z0_x1_z1')
		if not rect is Array or rect.size()!=4: return {}
		var rows=scene.collision.filter(func(r): return r is Dictionary and r.get('id')==owner.get('id'))
		if rows.size()!=1 or rows[0].get('polygon')!=[[rect[0],rect[1]],[rect[2],rect[1]],[rect[2],rect[3]],[rect[0],rect[3]]] or rows[0].get('requiresEvent')!=owner.get('requiresEvent') or rows[0].get('requiresActorFirstKill')!=null: return {}
		count+=1
	if count!=scene.collision.size(): return {}
	var parts: Array = []
	for p in selected.parent.get('parts_xz',[]):
		if not p is Array or p.size()!=4 or not p[0] is Array or not p[2] is Array or p[0].size()!=2 or p[2].size()!=2: return {}
		if not P.number(p[0][0]) or not P.number(p[0][1]) or not P.number(p[2][0]) or not P.number(p[2][1]): return {}
		var a=Vector2(p[0][0],p[0][1]);var b=Vector2(p[2][0],p[2][1])
		# Corner identity is compared in float64, exactly as the JSON writes it. Reading
		# the corners back through Vector2 first truncates them to float32, so 6.35/9.65
		# no longer equal their own literal (gh_lockdown west_transit) even though the
		# rectangle is exact -- which emptied the whole region plan and froze the client
		# at the spawn point. Same four-corner predicate, no float truncation, and the
		# Rect2 below is still built exactly as before.
		var x0=float(p[0][0]);var z0=float(p[0][1]);var x1=float(p[2][0]);var z1=float(p[2][1])
		if x1<=x0 or z1<=z0 or p[1]!=[x1,z0] or p[3]!=[x0,z1]: return {}
		parts.append(Rect2(a,b-a))
	return {'region':visible[0],'parts':parts,'floor_y':float(selected.variant.get('floor_y',0.0))} if not parts.is_empty() else {}

func contains(plan:Dictionary,p:Vector3) -> bool:
	if not p.is_finite() or p.y!=plan.get('floor_y'): return false
	for rect in plan.get('parts',[]):
		if rect is Rect2 and p.x>=rect.position.x and p.x<=rect.end.x and p.z>=rect.position.y and p.z<=rect.end.y: return true
	return false
