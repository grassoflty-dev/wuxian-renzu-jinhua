extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const Projection = preload('res://scripts/world/projection.gd')
const PlayerIdle = preload('res://scripts/world/cenyao_idle_visual.gd')
# Current body consumers all constrain their frame to this canvas and anchor.
# This is a conservative canvas envelope, never an opaque-pixel silhouette.
const VALVE_POSITIONS = {
	'cw_pressure_valve_visual_01':Vector3(8,0,7),
	'cw_pressure_valve_visual_02':Vector3(12,0,7),
	'cw_pressure_valve_visual_03':Vector3(16,0,7)
}
const FADE_MIN: float = 0.25
const FADE_RATE_PER_SECOND: float = 3.75 # Full 0.75 opacity excursion in 200 ms.
const FADE_GAP_MAX_MS: int = 250
var _fade_owner: Array = []
var _fade_alpha: Dictionary = {}
var _fade_target: Dictionary = {}
var _fade_last_ms: Dictionary = {}
const ID = 'runtime2d.world.clockworks.pressure_pipe_valve.v1'
const PATH = 'res://assets/props/clockworks/pressure_pipe_valve.png'
const SHA = 'cb411faf57aacc7be0c116b3806b64dac0f75a9a421875b2215f54e9447a6e3a'
var attempted = false
var texture: Texture2D
func _pair(value, x: float, y: float) -> bool:
	return value is Array and value.size()==2 and P.number(value[0],x,x) and P.number(value[1],y,y)
func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open('res://data/cw_pressure_valve.internal-use.json',FileAccess.READ)
	if file==null:
		return
	if file.get_length()<=0 or file.get_length()>8192:
		file.close()
		return
	var bytes = file.get_buffer(file.get_length())
	file.close()
	if bytes.get_string_from_utf8().to_utf8_buffer()!=bytes:
		return
	var m = JSON.parse_string(bytes.get_string_from_utf8())
	if not m is Dictionary or m.get('schema')!='wuxian.cw-pressure-valve.internal-use/v1' or typeof(m.get('registeredSourceReuse'))!=TYPE_BOOL or not m.registeredSourceReuse or m.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or m.get('GodotDisplay')!='NOT_RUN' or m.get('publicRelease')!='NOT_ESTABLISHED':
		return
	var row = m.get('asset')
	if not row is Dictionary or row.get('path')!=PATH or row.get('assetId')!=ID or row.get('sourceId')!=ID or row.get('sourceAdmission')!='release_approved' or row.get('sourceRuntimeStatus')!='ADOPT' or row.get('format')!='RGBA8' or not P.counter(row.get('bytes'),1,4194304) or row.bytes!=1384057 or row.get('sha256')!=SHA or not _pair(row.get('canvas'),1210,1179) or not _pair(row.get('anchor'),0.5,1) or not P.number(row.get('heightM'),2.2,2.2):
		return
	var png_file = FileAccess.open(PATH,FileAccess.READ)
	if png_file==null:
		return
	if png_file.get_length()!=1384057:
		png_file.close()
		return
	var png = png_file.get_buffer(png_file.get_length())
	png_file.close()
	var hash = HashingContext.new()
	hash.start(HashingContext.HASH_SHA256)
	hash.update(png)
	if hash.finish().hex_encode()!=SHA:
		return
	var image = Image.new()
	if image.load_png_from_buffer(png)!=OK or image.get_width()!=1210 or image.get_height()!=1179 or image.get_format()!=Image.FORMAT_RGBA8:
		return
	texture = ImageTexture.create_from_image(image)
func canvas_rect(foot: Vector2, ppm: float) -> Rect2:
	if not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(ppm) or ppm<=0:
		return Rect2()
	var factor: float = 2.2*ppm/1179.0
	var size: Vector2 = Vector2(1210,1179)*factor
	var origin: Vector2 = foot-Vector2(605,1179)*factor
	if not is_finite(factor) or factor<=0 or not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0 or not is_finite(origin.x) or not is_finite(origin.y):
		return Rect2()
	return Rect2(origin,size)

func reset_fade() -> bool:
	var changed: bool = not _fade_owner.is_empty() or not _fade_alpha.is_empty()
	_fade_owner.clear()
	_fade_alpha.clear()
	_fade_target.clear()
	_fade_last_ms.clear()
	return changed

func _position_exact(value: Variant, expected: Vector3) -> bool:
	return value is Array and value.size()==3 and P.number(value[0],expected.x,expected.x) and P.number(value[1],expected.y,expected.y) and P.number(value[2],expected.z,expected.z)

func _rect_valid(rect: Rect2) -> bool:
	return is_finite(rect.position.x) and is_finite(rect.position.y) and is_finite(rect.size.x) and is_finite(rect.size.y) and rect.size.x>0.0 and rect.size.y>0.0

func _fade_context(prop: Dictionary, snapshot: Dictionary, scene: Dictionary, key: String, camera, viewport: Rect2, gate: Dictionary, lifetime: Array, now_ms: int) -> bool:
	if now_ms<0 or not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.get('worldId')!='clockworks' or snapshot.get('sceneId')!='cw_pressure_hall' or not P.counter(snapshot.get('worldEpoch'),1):
		return false
	if scene.get('worldId')!='clockworks' or scene.get('sceneId')!='cw_pressure_hall' or not P.counter(scene.get('schemaVersion'),1,1) or key!='clockworks:cw_pressure_hall:'+str(snapshot.worldEpoch):
		return false
	if not P.counter(snapshot.player.get('currentHp'),1,4294967295):
		return false
	if not P.same(gate,snapshot) or gate.get('serverTick')!=snapshot.serverTick or gate.get('authorityRevision')!=snapshot.authorityRevision or gate.get('player_id')!=snapshot.player.entityId:
		return false
	for flag: String in ['presentation_allowed','visible','reduced_motion','higher_priority_body']:
		if typeof(gate.get(flag))!=TYPE_BOOL:
			return false
	if not gate.presentation_allowed or not gate.visible or lifetime.size()!=2 or not P.counter(lifetime[0]) or not P.counter(lifetime[1]):
		return false
	if not camera is RefCounted or camera.get_script()!=Projection or not camera.initialized or not P.number(camera.ppm,0.001,3072):
		return false
	if not _rect_valid(viewport) or viewport.position!=Vector2.ZERO or camera.viewport!=viewport.size or not camera.origin is Vector3 or not camera.screen_center is Vector2:
		return false
	if not is_finite(camera.origin.x) or not is_finite(camera.origin.y) or not is_finite(camera.origin.z) or not is_finite(camera.screen_center.x) or not is_finite(camera.screen_center.y):
		return false
	if PlayerIdle.CANVAS!=Vector2(336,560) or PlayerIdle.ANCHOR!=Vector2(168,551):
		return false
	var presentation: Variant = scene.get('presentation')
	if not presentation is Dictionary:
		return false
	var rows: Variant = presentation.get('sprites')
	if not P.rows(rows,2048) or not P.unique(rows,'id') or not VALVE_POSITIONS.has(prop.get('id')):
		return false
	var count: int = 0
	var matched: bool = false
	for row: Dictionary in rows:
		if not VALVE_POSITIONS.has(row.id) and row.get('assetId')!=ID:
			continue
		if not VALVE_POSITIONS.has(row.id) or row.size()!=4 or row.get('assetId')!=ID or row.get('layer')!='visual.props_dynamic' or not _position_exact(row.get('position'),VALVE_POSITIONS[row.id]):
			return false
		count += 1
		if row.id==prop.id:
			matched = row==prop
	return count==3 and matched

func occlusion_alpha(prop: Dictionary, snapshot: Dictionary, scene: Dictionary, key: String, camera, viewport: Rect2, gate: Dictionary, lifetime: Array, now_ms: int) -> float:
	# Called only from the existing selected-prop draw branch, after its culling.
	# Fresh gate/context checks happen on every emitted valve, not only on process.
	if not _fade_context(prop,snapshot,scene,key,camera,viewport,gate,lifetime,now_ms):
		reset_fade()
		return 1.0
	_load_once()
	if texture==null:
		reset_fade()
		return 1.0
	var ppm: float = float(camera.ppm)
	var player_world: Vector3 = P.vector(snapshot.player.transform.positionM)
	var valve_world: Vector3 = VALVE_POSITIONS[prop.id]
	var player_foot: Vector2 = camera.project(player_world)
	var valve_foot: Vector2 = camera.project(valve_world)
	var valve_rect: Rect2 = canvas_rect(valve_foot,ppm)
	var factor: float = (104.0/560.0)*(ppm/48.0)
	var player_rect: Rect2 = Rect2(player_foot-PlayerIdle.ANCHOR*factor,PlayerIdle.CANVAS*factor)
	# Include the existing circular player marker and line fallback, not VFX or UI.
	var marker_radius: float = 12.0*ppm/48.0
	player_rect = player_rect.merge(Rect2(player_foot-Vector2.ONE*marker_radius,Vector2.ONE*marker_radius*2.0))
	if not _rect_valid(valve_rect) or not _rect_valid(player_rect):
		reset_fade()
		return 1.0
	var player_ground: Vector2 = camera.project(Vector3(player_world.x,0.0,player_world.z))
	var valve_ground: Vector2 = camera.project(Vector3(valve_world.x,0.0,valve_world.z))
	if not is_finite(player_ground.y) or not is_finite(valve_ground.y):
		reset_fade()
		return 1.0
	# Exactly the same comparator as renderer.objects.sort_custom, including ties.
	var valve_after_player: bool = snapshot.player.entityId<prop.id if is_equal_approx(player_ground.y,valve_ground.y) else player_ground.y<valve_ground.y
	var visible_player: Rect2 = player_rect.intersection(viewport)
	var visible_valve: Rect2 = valve_rect.intersection(viewport)
	var overlaps: bool = _rect_valid(visible_player) and _rect_valid(visible_valve) and visible_player.intersects(visible_valve)
	var target: float = FADE_MIN if valve_after_player and overlaps else 1.0
	var owner: Array = [key,snapshot.player.entityId,lifetime[0],lifetime[1],viewport,ppm]
	if owner!=_fade_owner:
		reset_fade()
		_fade_owner = owner.duplicate()
	var id: String = prop.id
	var previous: float = float(_fade_alpha.get(id,1.0))
	var last_ms: int = int(_fade_last_ms.get(id,now_ms))
	var elapsed_ms: int = now_ms-last_ms
	if not is_finite(previous) or previous<FADE_MIN or previous>1.0 or elapsed_ms<0 or elapsed_ms>FADE_GAP_MAX_MS:
		previous = 1.0
		elapsed_ms = 0
	# A newly observed overlap/release starts now, never retroactively.
	if not _fade_target.has(id) or float(_fade_target[id])!=target:
		elapsed_ms = 0
	var result: float = target if gate.reduced_motion else move_toward(previous,target,float(elapsed_ms)*0.001*FADE_RATE_PER_SECOND)
	_fade_alpha[id] = result
	_fade_target[id] = target
	_fade_last_ms[id] = now_ms
	return result

func draw_body(canvas: CanvasItem, foot: Vector2, ppm: float, opacity: float = 1.0) -> bool:
	var rectangle: Rect2 = canvas_rect(foot,ppm)
	if not _rect_valid(rectangle):
		return false
	_load_once()
	if texture==null:
		reset_fade()
		return false
	var alpha: float = opacity if is_finite(opacity) and opacity>=FADE_MIN and opacity<=1.0 else 1.0
	canvas.draw_texture_rect(texture,rectangle,false,Color(1.0,1.0,1.0,alpha))
	return true
