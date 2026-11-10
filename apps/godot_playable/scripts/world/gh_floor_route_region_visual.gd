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

# The common discriminator: the scene data that the region rows are pinned to, and a
# snapshot whose player transform is trustworthy. Never widens membership itself.
func _context_ok(snapshot:Dictionary,scene:Dictionary,hash:String) -> bool:
	if not _load() or not P.snapshot(snapshot) or not scope(snapshot,hash) or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or not snapshot.get('denseEntityRegions') is Dictionary: return false
	return P.transform(snapshot.player.get('transform'))

# Every authored variant of this scene at the player's *known* floor state. The phase rows
# are selected by the real lift/support view exactly as before; nothing here decides an
# occupied region.
func _variant_rows(snapshot:Dictionary,hash:String) -> Array:
	var out: Array = []
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
		out.append({'key':key,'variant':variant})
	return out

# Same rows, but a shaft support state the client cannot name does not erase the authored
# floor rows: the fallback plan still has to agree with the player's floor height, and the
# rows are only a *display/plan* approximation while the authority publishes no region.
func _fallback_variant_rows(snapshot:Dictionary,hash:String) -> Array:
	var out: Array = []
	for key in data.variants:
		var variant=data.variants[key]
		if not variant is Dictionary or variant.get('scene')!=snapshot.sceneId or not variant.get('parents') is Array: continue
		if snapshot.sceneId=='gh_central_shaft':
			var lift=snapshot.get('shaftLift');var support=snapshot.get('supportScene')
			if snapshot.get('shaftLiftDeferred')==true and lift==null and support==null:
				if key!='shaft_deferred': continue
			elif lift is Dictionary and support is Dictionary and support.get('rider') is Dictionary and support.rider.get('mode')=='floor' and P.point(support.rider.get('positionM')) and support.rider.positionM==snapshot.player.transform.positionM:
				if support.rider.positionM.yM==0.0:
					if typeof(lift.get('upperGateOpen'))!=TYPE_BOOL or key!=('shaft_active_open' if lift.upperGateOpen else 'shaft_active_closed'): continue
				elif support.rider.positionM.yM==-4.0:
					if typeof(lift.get('lowerGateOpen'))!=TYPE_BOOL or key!=('shaft_lower_open' if lift.lowerGateOpen else 'shaft_lower_closed'): continue
				else: continue
		if snapshot.sceneId=='gh_lockdown' and (key.begins_with('lockdown_legacy_') != (hash==HASHES.gh_lockdown[1])): continue
		out.append({'key':key,'variant':variant})
	return out

# Current native region and selected event phase must agree; duplicate phase rows are
# resolved only by the actual effective state, never by arbitrary ordering.
func _phase_matches(snapshot:Dictionary,events:Array,key:String) -> bool:
	if not snapshot.sceneId in ['gh_gate_b','gh_lockdown']: return true
	var suffix='_open' if events.has('hive_lockdown') else '_closed'
	return key.ends_with(suffix)

# The variant's collision owners must still be the authored ones. This is the same strict
# identity as before; a genuinely mismatched region stays unavailable instead of being
# approximated.
func _owners_ok(variant:Dictionary,scene:Dictionary) -> bool:
	if not scene.get('collision') is Array or not P.unique(scene.collision,'id') or not variant.get('owners') is Array: return false
	var count=0
	for owner in variant.owners:
		if not owner is Dictionary or owner.get('role')!='collision_owner' or owner.get('id') in ['well_west','well_east','well_north','upper_landing_gate']: continue
		var rect=owner.get('rect_x0_z0_x1_z1')
		if not rect is Array or rect.size()!=4: return false
		var rows=scene.collision.filter(func(r): return r is Dictionary and r.get('id')==owner.get('id'))
		if rows.size()!=1 or rows[0].get('polygon')!=[[rect[0],rect[1]],[rect[2],rect[1]],[rect[2],rect[3]],[rect[0],rect[3]]] or rows[0].get('requiresEvent')!=owner.get('requiresEvent') or rows[0].get('requiresActorFirstKill')!=null: return false
		count+=1
	return count==scene.collision.size()

func _parts(parent:Dictionary) -> Array:
	var parts: Array = []
	for p in parent.get('parts_xz',[]):
		if not p is Array or p.size()!=4 or not p[0] is Array or not p[2] is Array or p[0].size()!=2 or p[2].size()!=2: return []
		if not P.number(p[0][0]) or not P.number(p[0][1]) or not P.number(p[2][0]) or not P.number(p[2][1]): return []
		# Corner identity is compared in float64, exactly as the JSON writes it. Reading
		# the corners back through Vector2 first truncates them to float32, so 6.35/9.65
		# no longer equal their own literal (gh_lockdown west_transit) even though the
		# rectangle is exact -- which emptied the whole region plan and froze the client
		# at the spawn point. Same four-corner predicate, no float truncation, and the
		# Rect2 below is still built exactly as before.
		var x0=float(p[0][0]);var z0=float(p[0][1]);var x1=float(p[2][0]);var z1=float(p[2][1])
		if x1<=x0 or z1<=z0 or p[1]!=[x1,z0] or p[3]!=[x0,z1]: return []
		parts.append(Rect2(Vector2(p[0][0],p[0][1]),Vector2(p[2][0],p[2][1])-Vector2(p[0][0],p[0][1])))
	return parts

func _plan(row:Dictionary,region:String,parts:Array,fallback:bool,distance:float) -> Dictionary:
	var variant:Dictionary=row.variant
	var plan={'region':region,'parts':parts,'floor_y':float(variant.get('floor_y',0.0))}
	if fallback:
		plan['fallback']=true
		plan['fallback_distance_m']=distance
	return plan

func prepare(snapshot:Dictionary,scene:Dictionary,hash:String,events:Array) -> Dictionary:
	if not _context_ok(snapshot,scene,hash): return {}
	var visible=snapshot.get('currentVisibleRegions')
	if not visible is Array or visible.size()!=1 or typeof(visible[0])!=TYPE_STRING or snapshot.get('denseOccupiedRegion')!=visible[0]: return {}
	var matches: Array = []
	for row in _variant_rows(snapshot,hash):
		for parent in row.variant.parents:
			if parent is Dictionary and parent.get('id')==visible[0] and parent.get('available')==true: matches.append({'key':row.key,'variant':row.variant,'parent':parent})
	if snapshot.sceneId in ['gh_gate_b','gh_lockdown']:
		matches=matches.filter(func(m): return _phase_matches(snapshot,events,str(m.key)))
	if matches.size()!=1: return {}
	var selected:Dictionary=matches[0]
	if not _owners_ok(selected.variant,scene): return {}
	var parts=_parts(selected.parent)
	if parts.is_empty(): return {}
	return _plan(selected,visible[0],parts,false,0.0)

# The authority can publish no occupied region at all while the player still stands inside
# the authored scene envelope (observed at (0.05,9.04) and (23.99,9.12) in
# gh_central_shaft: the moving-support proof does not cover that spot, so the whole
# information layer goes absent and `dense_region_ready()` froze every key). Movement is
# still authoritative, so the client plans against the nearest authored region whose row
# is itself fully valid. Anything genuinely unready (missing/mismatched geometry, unknown
# floor) still returns {} and still freezes.
func prepare_nearest(snapshot:Dictionary,scene:Dictionary,hash:String,events:Array) -> Dictionary:
	if not _context_ok(snapshot,scene,hash): return {}
	if snapshot.get('denseOccupiedRegion')!=null: return {}
	var visible=snapshot.get('currentVisibleRegions')
	if visible!=null and not (visible is Array and visible.is_empty()): return {}
	var p=P.vector(snapshot.player.transform.positionM)
	if not bounds_contain(scene.get('boundsM'),p): return {}
	var best: Dictionary={}
	var best_distance: float=INF
	for row in _fallback_variant_rows(snapshot,hash):
		if not _phase_matches(snapshot,events,str(row.key)): continue
		if not _owners_ok(row.variant,scene): continue
		if float(row.variant.get('floor_y',0.0))!=p.y: continue
		for parent in row.variant.parents:
			if not parent is Dictionary or parent.get('available')!=true or typeof(parent.get('id'))!=TYPE_STRING: continue
			var parts=_parts(parent)
			if parts.is_empty(): continue
			for rect in parts:
				var dx=maxf(maxf(rect.position.x-p.x,0.0),p.x-rect.end.x)
				var dz=maxf(maxf(rect.position.y-p.z,0.0),p.z-rect.end.y)
				var d=Vector2(dx,dz).length()
				if d<best_distance:
					best_distance=d
					best={'key':row.key,'variant':row.variant,'parent':parent,'parts':parts}
	if best.is_empty(): return {}
	return _plan(best,best.parent.id,best.parts,true,best_distance)

func contains(plan:Dictionary,p:Vector3) -> bool:
	if not p.is_finite() or p.y!=plan.get('floor_y'): return false
	for rect in plan.get('parts',[]):
		if rect is Rect2 and p.x>=rect.position.x and p.x<=rect.end.x and p.z>=rect.position.y and p.z<=rect.end.y: return true
	return false

# The authored scene envelope only (`boundsM`, the same rectangle the catalog validates).
func bounds_contain(bounds:Variant,p:Vector3) -> bool:
	if not bounds is Dictionary or not p.is_finite(): return false
	if not P.number(bounds.get('x'),0,4096) or not P.number(bounds.get('z'),0,4096) or not P.number(bounds.get('width'),0.001,4096) or not P.number(bounds.get('depth'),0.001,4096): return false
	return p.x>=float(bounds.x) and p.x<=float(bounds.x)+float(bounds.width) and p.z>=float(bounds.z) and p.z<=float(bounds.z)+float(bounds.depth)

# A missing moving-support view is an information gap, not a missing floor: when the
# authority publishes no lift/support view at all while the player stands on the authored
# floor inside the scene envelope, the optional lift overlay cannot be composed. That must
# not turn into an input lock (observed at (0.05,9.04) / (23.99,9.12) in gh_central_shaft:
# `dense_region_ready()` stayed false for 75-210 s while every key was refused). Nothing
# outside this exact shape is widened: a published support view, a deferred-floor proof,
# another scene, another height or a position outside the envelope all still fail closed.
func floor_without_support(snapshot:Dictionary,scene:Dictionary) -> bool:
	if not P.snapshot(snapshot) or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_central_shaft': return false
	if snapshot.get('shaftLift')!=null or snapshot.get('supportScene')!=null or snapshot.get('shaftLiftDeferred')==true: return false
	if not scene is Dictionary or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId: return false
	var p=P.vector(snapshot.player.transform.positionM)
	if not p.is_finite() or not (p.y==0.0 or p.y==-4.0): return false
	return bounds_contain(scene.get('boundsM'),p)
