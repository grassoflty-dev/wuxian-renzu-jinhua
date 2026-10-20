extends RefCounted

# Candidate decorative water-speed contrast, not hydraulic simulation or drain progress.
# No emitter/outlet, net transport, local timer, state cache, or gameplay changes.
const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const SCENE_SHA: String = '557c180648bc8c73621b444dd108f2207535750947e9a7738a5716dd1b2a5437'
const CENTERS: Array[Vector2] = [Vector2(19,6.3),Vector2(20.8,6.3),Vector2(19,8),Vector2(20.8,8),Vector2(19,9.7),Vector2(20.8,9.7)]
const INK: Color = Color(0.28,0.56,0.62,0.065)
const COLLISION: Array = [{"id":"pump_wall_north","polygon":[[0.0,0.0],[24.0,0.0],[24.0,0.5],[0.0,0.5]]},{"id":"pump_wall_south","polygon":[[0.0,15.5],[24.0,15.5],[24.0,16.0],[0.0,16.0]]},{"id":"pump_wall_west_north","polygon":[[0.0,0.5],[0.5,0.5],[0.5,6.0],[0.0,6.0]]},{"id":"pump_wall_west_south","polygon":[[0.0,9.5],[0.5,9.5],[0.5,15.5],[0.0,15.5]]},{"id":"pump_wall_east_north","polygon":[[23.5,0.5],[24.0,0.5],[24.0,6.0],[23.5,6.0]]},{"id":"pump_wall_east_south","polygon":[[23.5,9.5],[24.0,9.5],[24.0,15.5],[23.5,15.5]]}]
const TERRAIN: Array = [{"id":"mh_pump_east_channel_water","polygon":[[18.0,5.5],[23.5,5.5],[23.5,10.5],[18.0,10.5]],"tag":"terrain.water_deep"}]
const TRANSITIONS: Array = [{"id":"mh_pump_station_to_breakwater","polygon":[[0.0,6.5],[2.0,6.5],[2.0,9.5],[0.0,9.5]],"spawnId":"mh_breakwater_spawn","toSceneId":"mh_breakwater"},{"id":"mh_pump_station_to_tower","polygon":[[22.0,6.5],[24.0,6.5],[24.0,9.5],[22.0,9.5]],"spawnId":"mh_resonance_tower_spawn","toSceneId":"mh_resonance_tower"}]
const WALKABLE: Array = [{"id":"mh_pump_station_walkable_area","polygon":[[0.5,0.5],[23.5,0.5],[23.5,15.5],[0.5,15.5]]}]
const PRESENTATION: Dictionary = {"cameraProfile":"oblique_default","layers":[{"id":"visual.floor","zGroup":0},{"id":"visual.props_back","zGroup":10},{"id":"visual.props_dynamic","zGroup":20},{"id":"visual.vfx_markers","zGroup":30}],"sprites":[{"assetId":"runtime2d.world.mistharbor.water_edge_stone.v1","id":"approved_pump_station_stone_water_module","layer":"visual.floor","position":[12.0,0.0,8.0]},{"assetId":"runtime2d.world.mistharbor.water_edge.v1","id":"pump_station_water_edge","layer":"visual.props_back","position":[12.0,0.0,14.5]},{"assetId":"runtime2d.world.mistharbor.fog_bank.v1","id":"pump_station_fog_bank","layer":"visual.props_back","position":[18.0,0.0,3.0]}]}

func _same(actual: Variant, expected: Variant) -> bool:
	if expected is Dictionary:
		if not actual is Dictionary or actual.size()!=expected.size():
			return false
		for key: Variant in expected:
			if not actual.has(key) or not _same(actual[key],expected[key]):
				return false
		return true
	if expected is Array:
		if not actual is Array or actual.size()!=expected.size():
			return false
		for index: int in range(expected.size()):
			if not _same(actual[index],expected[index]):
				return false
		return true
	if typeof(expected) in [TYPE_INT,TYPE_FLOAT]:
		return P.number(actual,float(expected),float(expected))
	return typeof(actual)==typeof(expected) and actual==expected

func _finite2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y) and absf(value.x)<=10000000.0 and absf(value.y)<=10000000.0

func _context(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, projection: Variant, viewport: Variant, gate: Variant) -> bool:
	if not canvas is CanvasItem or not is_instance_valid(canvas) or not canvas.is_inside_tree() or canvas.is_queued_for_deletion():
		return false
	if not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.player.currentHp<=0:
		return false
	if snapshot.worldId!='mist_harbor' or snapshot.sceneId!='mh_pump_station' or not P.counter(snapshot.worldEpoch,1):
		return false
	if typeof(key)!=TYPE_STRING or key!='mist_harbor:mh_pump_station:'+str(snapshot.worldEpoch) or scene_hash!=SCENE_SHA:
		return false
	if not scene is Dictionary or scene.get('worldId')!='mist_harbor' or scene.get('sceneId')!='mh_pump_station':
		return false
	if not P.counter(scene.get('schemaVersion'),1,1) or not _same(scene.get('boundsM'),{'x':0,'z':0,'width':24,'depth':16}):
		return false
	if not _same(scene.get('collision'),COLLISION) or not _same(scene.get('terrainRegions'),TERRAIN) or not _same(scene.get('transitions'),TRANSITIONS) or not _same(scene.get('hazards'),[]) or not _same(scene.get('occluders'),[]) or not _same(scene.get('walkablePolygons'),WALKABLE) or not _same(scene.get('presentation'),PRESENTATION):
		return false
	var pump: Variant = snapshot.get('mistHarborPump')
	if not pump is Dictionary or pump.size()!=1 or typeof(pump.get('state'))!=TYPE_STRING or pump.get('state') not in ['ready','draining']:
		return false
	if canvas.get('snapshot')!=snapshot or canvas.get('scene')!=scene or canvas.get('current_key')!=key or canvas.get('scene_hash')!=scene_hash:
		return false
	if not gate is Dictionary or not P.same(gate,snapshot) or gate.get('serverTick')!=snapshot.serverTick or gate.get('authorityRevision')!=snapshot.authorityRevision or gate.get('player_id')!=snapshot.player.entityId:
		return false
	for field: String in ['presentation_allowed','visible','reduced_motion']:
		if typeof(gate.get(field))!=TYPE_BOOL:
			return false
	if not gate.presentation_allowed or not gate.visible or gate.reduced_motion:
		return false
	if not projection is RefCounted or not is_instance_valid(projection) or projection.get_script()!=ProjectionScript or canvas.get('camera')!=projection:
		return false
	if not viewport is Rect2 or canvas.get('viewport_rect')!=viewport or viewport!=canvas.get_viewport_rect():
		return false
	var origin: Variant = projection.get('origin')
	var center: Variant = projection.get('screen_center')
	var size: Variant = projection.get('viewport')
	if not origin is Vector3 or not center is Vector2 or not size is Vector2:
		return false
	if typeof(projection.get('initialized'))!=TYPE_BOOL or not projection.get('initialized') or not P.number(projection.get('ppm'),0.001,256):
		return false
	return is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and _finite2(center) and _finite2(size) and _finite2(viewport.position) and _finite2(viewport.size) and _finite2(viewport.end) and viewport.position==Vector2.ZERO and viewport.size==size and size.x>0 and size.y>0

func draw_flow(canvas: Variant, snapshot: Variant, scene: Variant, key: Variant, scene_hash: Variant, projection: Variant, viewport: Variant, gate: Variant) -> int:
	if not _context(canvas,snapshot,scene,key,scene_hash,projection,viewport,gate):
		return 0
	# Only presentation speed uses state. Native state alone determines completion.
	var period: int = 90 if snapshot.mistHarborPump.state=='draining' else 360
	var tick: int = int(snapshot.serverTick)%period
	var plans: Array[Dictionary] = []
	for index: int in range(CENTERS.size()):
		var phase: float = TAU*(float(tick)/float(period)+float(index)/6.0)
		# Opposite ends deform symmetrically around a fixed center. No outlet direction.
		var span: float = 0.42+0.06*sin(phase)
		var bend: float = 0.04*cos(phase)
		if not is_finite(span) or span<0.359999 or span>0.480001 or not is_finite(bend) or absf(bend)>0.040001:
			return 0
		var offsets: Array[Vector2] = [Vector2(-span,-bend-0.018),Vector2(span,bend-0.018),Vector2(span,bend+0.018),Vector2(-span,-bend+0.018)]
		var points: PackedVector2Array = PackedVector2Array()
		for offset: Vector2 in offsets:
			var ground: Vector2 = CENTERS[index]+offset
			# Full footprint bound, not center-only clipping. It excludes Tower entry.
			if not _finite2(ground) or ground.x<18.5 or ground.x>21.3 or ground.y<6.2 or ground.y>9.8:
				return 0
			var projected: Variant = projection.call('project',Vector3(ground.x,0.0,ground.y))
			if not projected is Vector2 or not _finite2(projected):
				return 0
			points.append(projected)
		if points.size()!=4:
			return 0
		var bounds: Rect2 = Rect2(points[0],Vector2.ZERO)
		for point: Vector2 in points:
			bounds = bounds.expand(point)
		if not _finite2(bounds.position) or not _finite2(bounds.end) or bounds.size.x<=0.0 or bounds.size.y<=0.0:
			return 0
		plans.append({'points':points,'bounds':bounds})
	# No canvas submission before complete six-polygon planning and context recheck.
	if plans.size()!=6 or not _context(canvas,snapshot,scene,key,scene_hash,projection,viewport,gate):
		return 0
	var submitted: int = 0
	for plan: Dictionary in plans:
		if plan.bounds.intersects(viewport):
			canvas.draw_colored_polygon(plan.points,INK)
			submitted += 1
	return submitted
