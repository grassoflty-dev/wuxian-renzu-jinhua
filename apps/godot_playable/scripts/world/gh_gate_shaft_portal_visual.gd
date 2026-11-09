extends RefCounted

# A pinned secondary catalog and owner-issued preparation are both required.
# Only the two-metre ground strip inside the selected upper opening is drawn.
const Catalog = preload('res://scripts/world/scene_catalog.gd')
const P = preload('res://scripts/bridge/protocol.gd')
var catalogs: Dictionary = {}
var attempted: Dictionary = {}

func prepare(snapshot: Dictionary, scene: Dictionary, key: String, camera, shaft_art) -> Dictionary:
	if not P.snapshot(snapshot) or snapshot.worldId!='grey_hive' or not snapshot.sceneId in ['gh_gate_a','gh_central_shaft']: return {}
	if scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or key!='grey_hive:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch) or not camera.initialized: return {}
	var record = snapshot.get('shaftUpperNeighbor')
	if not record is Dictionary or record.size()!=7 or record.get('sourceSceneId')!=snapshot.sceneId or not P.counter(record.get('worldEpoch')) or not P.counter(record.get('serverTick')) or not P.counter(record.get('authorityRevision')) or record.worldEpoch!=snapshot.worldEpoch or record.serverTick!=snapshot.serverTick or record.authorityRevision!=snapshot.authorityRevision: return {}
	var target: String = 'gh_central_shaft' if snapshot.sceneId=='gh_gate_a' else 'gh_gate_a'
	if record.get('targetSceneId')!=target or typeof(record.get('shaftDeferred'))!=TYPE_BOOL: return {}
	if not attempted.has(target):
		attempted[target]=true
		var catalog = Catalog.new()
		if catalog.build('grey_hive',target): catalogs[target]=catalog
	var catalog = catalogs.get(target)
	if catalog==null or catalog.source_hash!=record.get('targetSceneSourceSha256'): return {}
	var b = catalog.current.get('boundsM')
	var width: int = 24 if target=='gh_central_shaft' else 20
	var depth: int = 16 if target=='gh_central_shaft' else 14
	if not b is Dictionary or b.get('x')!=0 or b.get('z')!=0 or b.get('width')!=width or b.get('depth')!=depth: return {}
	var strip := Rect2(0,7,2,2) if target=='gh_central_shaft' else Rect2(18,6,2,2)
	# Do not paint across any actual static collider in this opening.
	var collisions = catalog.current.get('collision')
	if not collisions is Array: return {}
	for row in collisions:
		if not row is Dictionary or not row.get('polygon') is Array or row.polygon.size()!=4: return {}
		var polygon: Array = row.polygon
		if not polygon[0] is Array or not polygon[2] is Array or polygon[0].size()!=2 or polygon[2].size()!=2: return {}
		for point in polygon:
			if not point is Array or point.size()!=2 or not P.number(point[0]) or not P.number(point[1]): return {}
		var x0: float = INF
		var x1: float = -INF
		var z0: float = INF
		var z1: float = -INF
		for point in polygon:
			x0=minf(x0,float(point[0])); x1=maxf(x1,float(point[0]))
			z0=minf(z0,float(point[1])); z1=maxf(z1,float(point[1]))
		if x0>=x1 or z0>=z1: return {}
		for point in polygon:
			if float(point[0]) not in [x0,x1] or float(point[1]) not in [z0,z1]: return {}
		if Rect2(x0,z0,x1-x0,z1-z0).intersects(strip): return {}
	if not shaft_art._load_once(): return {}
	var offset := Vector3(20,0,-1) if target=='gh_central_shaft' else Vector3(-20,0,1)
	var project: Callable = func(p: Vector3) -> Vector2: return camera.project(p+offset)
	var pieces: Array = []
	var phase := Vector2(0,-1) if target=='gh_central_shaft' else Vector2.ZERO
	if not shaft_art._floor_rect(pieces,project,strip,0.0,{'floor':'upper','role':'ground'},phase): return {}
	if pieces.is_empty(): return {}
	return {'ground':pieces,'selection':{'floor':'upper','regions':['upper']}}
