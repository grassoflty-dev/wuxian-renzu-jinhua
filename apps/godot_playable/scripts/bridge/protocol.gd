extends RefCounted
# Native JSON type checks: Boolean never passes as a number; integral floats are accepted.
const SAFE = 9007199254740991.0
static func number(v, lo: float = -1.0e9, hi: float = 1.0e9) -> bool:
	return (typeof(v) == TYPE_INT or typeof(v) == TYPE_FLOAT) and is_finite(float(v)) and float(v) >= lo and float(v) <= hi
static func counter(v, lo: float = 0.0, hi: float = SAFE) -> bool:
	return number(v, lo, hi) and floor(float(v)) == float(v)
static func ident(v) -> bool:
	if typeof(v) != TYPE_STRING or v.is_empty() or v.length() > 128:
		return false
	for c in v:
		if not c in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.:-':
			return false
	return true
static func point(v) -> bool:
	return v is Dictionary and number(v.get('xM')) and number(v.get('yM'), -64, 64) and number(v.get('zM'))
static func vector(v) -> Vector3:
	return Vector3(float(v.xM), float(v.yM), float(v.zM))
static func transform(v) -> bool:
	return v is Dictionary and point(v.get('positionM')) and number(v.get('yawRad'), -10000, 10000)
static func rows(v, maximum: int) -> bool:
	return v is Array and v.size() <= maximum
static func unique(v: Array, key: String) -> bool:
	var seen = {}
	for row in v:
		if not row is Dictionary or not ident(row.get(key)) or seen.has(row[key]):
			return false
		seen[row[key]] = true
	return true
static func context(v) -> bool:
	return v is Dictionary and ident(v.get('worldId')) and ident(v.get('sceneId')) and counter(v.get('worldEpoch'))
static func same(a: Dictionary, b: Dictionary) -> bool:
	return a.get('worldId') == b.get('worldId') and a.get('sceneId') == b.get('sceneId') and a.get('worldEpoch') == b.get('worldEpoch')
static func token(v) -> bool:
	return context(v) and v.size()==4 and counter(v.get('generation'), 1)
static func snapshot(s) -> bool:
	if not s is Dictionary or s.get('kind') != 'full' or s.get('protocolVersion') != 3 or not counter(s.get('protocolVersion'), 3, 3) or s.get('schemaVersion') != 'freeze-v02-interfaces/1.2' or not context(s):
		return false
	if not s.has('checkpointId') or s.checkpointId!=null and typeof(s.checkpointId)!=TYPE_STRING:
		return false
	for k in ['worldEpoch', 'serverTick', 'authorityRevision', 'ackSeq']:
		if not counter(s.get(k)):
			return false
	var p = s.get('player')
	if not p is Dictionary or not ident(p.get('entityId')) or not transform(p.get('transform')) or not point(p.get('velocityMps')) or typeof(p.get('actionState')) != TYPE_STRING:
		return false
	for k in ['currentHp','maxHp','currentEnergy','maxEnergy']:
		if not counter(p.get(k), 0, 4294967295):
			return false
	if p.maxHp <= 0 or p.maxEnergy <= 0 or p.currentHp > p.maxHp or p.currentEnergy > p.maxEnergy:
		return false
	for k in ['facingX','facingZ','aimX','aimZ']:
		if not number(p.get(k), -1, 1):
			return false
	for field in ['actors','doors','interactables','hazards','objectives']:
		if not rows(s.get(field), 2048):
			return false
	for field in ['actors','interactables','hazards']:
		if not unique(s[field], 'entityId'):
			return false
		for row in s[field]:
			if not transform(row.get('transform')) or typeof(row.get('active')) != TYPE_BOOL:
				return false
			var k = 'entityType' if field == 'actors' else 'kind'
			if typeof(row.get(k)) != TYPE_STRING:
				return false
			if field == 'actors' and typeof(row.get('actorKind')) != TYPE_STRING:
				return false
	if not unique(s.doors, 'doorId') or not unique(s.objectives, 'objectiveId'):
		return false
	for d in s.doors:
		if not transform(d.get('transform')) or typeof(d.get('open')) != TYPE_BOOL or typeof(d.get('locked')) != TYPE_BOOL:
			return false
	for o in s.objectives:
		if typeof(o.get('state')) != TYPE_STRING or not point(o.get('positionM')):
			return false
	if not s.get('capabilities') is Dictionary or not s.get('progression') is Dictionary:
		return false
	if not counter(s.capabilities.get('schemaVersion'),1,1) or not counter(s.progression.get('schemaVersion'),1,1) or not counter(s.progression.get('eventSeq')) or not ident(s.progression.get('currentWorldId')) or not rows(s.progression.get('worlds'),16):
		return false
	if s.get('entryToken') != null and (not token(s.entryToken) or not same(s.entryToken, s)):
		return false
	return true
static func receipt(r) -> bool:
	if not r is Dictionary or typeof(r.get('commandId')) != TYPE_STRING or typeof(r.get('applied')) != TYPE_BOOL or typeof(r.get('alreadyApplied')) != TYPE_BOOL or not snapshot(r.get('snapshot')):
		return false
	if r.applied and r.alreadyApplied or (r.get('errorCode') != null and typeof(r.errorCode) != TYPE_STRING):
		return false
	if (r.applied or r.alreadyApplied) and r.get('errorCode')!=null or not r.applied and not r.alreadyApplied and typeof(r.get('errorCode'))!=TYPE_STRING:
		return false
	for k in ['worldEpoch','serverTick','authorityRevision']:
		if not counter(r.get(k)) or r[k] != r.snapshot[k]:
			return false
	return true
static func success(r) -> bool:
	return receipt(r) and (r.applied or r.alreadyApplied) and r.get('errorCode') == null
static func status(v) -> bool:
	return context(v) and typeof(v.get('paused')) == TYPE_BOOL and counter(v.get('lastLifecycleCommandSequence')) and (v.get('entryToken') == null or token(v.entryToken)) and (v.get('enhancementTerminalContext') == null or v.enhancementTerminalContext is Dictionary)
static func capability_items(s: Dictionary) -> Array:
	var out: Array = []
	var a = s.get('capabilities', {}).get('items')
	if not rows(a, 128) or not unique(a, 'capabilityId'):
		return out
	for i in a:
		if typeof(i.get('granted')) != TYPE_BOOL or typeof(i.get('selected')) != TYPE_BOOL or not i.granted and i.selected or i.has('cooldownRemainingMs') and not counter(i.cooldownRemainingMs, 0, 600000):
			return []
		out.append(i)
	return out
static func granted(s: Dictionary, id: String, selected: bool = false) -> bool:
	for i in capability_items(s):
		if i.capabilityId == id and i.granted and (not selected or i.selected):
			return true
	return false
static func build(v) -> bool:
	if not v is Dictionary or not counter(v.get('schemaVersion'), 1, 1) or not counter(v.get('revision')) or not rows(v.get('items'),512) or not rows(v.get('equipment'),32) or not unique(v.items,'itemId') or not unique(v.equipment,'slotId'):
		return false
	var items = {}
	for i in v.items:
		if not counter(i.get('quantity'),1,4294967295) or not ident(i.get('slotId')) or typeof(i.get('label')) != TYPE_STRING or typeof(i.get('description')) != TYPE_STRING or typeof(i.get('equipped')) != TYPE_BOOL or typeof(i.get('releaseEligible')) != TYPE_BOOL:
			return false
		items[i.itemId] = i
	var used = {}
	for e in v.equipment:
		if not ident(e.get('itemId')) or used.has(e.itemId) or not items.has(e.itemId) or e.get('label') != items[e.itemId].label or e.slotId != items[e.itemId].slotId or not items[e.itemId].equipped:
			return false
		used[e.itemId] = true
	for i in v.items:
		if i.equipped != used.has(i.itemId):
			return false
	return true
static func ticket(t, expected: Dictionary) -> bool:
	return context(t) and t.size()==8 and same(t,expected) and ident(t.get('ownerId')) and counter(t.get('generation'),1) and counter(t.get('pauseCommandSequence'),1) and t.get('entityId') == 'gh_bz_whitezhi_v1' and t.get('interactionId') == 'gh_bz_first_contact'
static func polygon(v) -> bool:
	if not rows(v,128) or v.size() < 3:
		return false
	var area = 0.0
	for n in range(v.size()):
		var a = v[n]
		var b = v[(n+1)%v.size()]
		if not a is Array or a.size()!=2 or not number(a[0],0,4096) or not number(a[1],0,4096) or not b is Array or b.size()!=2 or not number(b[0],0,4096) or not number(b[1],0,4096):
			return false
		area += a[0]*b[1]-b[0]*a[1]
	return abs(area) > 0.000001
