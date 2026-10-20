extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const CONFIG = {
	'furnace_v1':{'id':'runtime2d.world.clockworks.furnace.v1','width':1193,'height':1254,'anchor':[0.5,1.0],'heightM':4.0,'bytes':1938854,'sha256':'4a36e98fec3057bac6e59e01bcd2287ee5e412071a5396731c50082a351aec44'},
	'furnace_v2':{'id':'runtime2d.world.clockworks.furnace.v2','width':1165,'height':1240,'anchor':[0.5,1.0],'heightM':4.0,'bytes':1584389,'sha256':'207e2b4c1759e49bfd03613958f86787586410dc0abe6da526f34679f17cc74d'}
}
var attempted = false
var textures: Dictionary = {}
# Exact nonzero-alpha source-pixel runs, prepared only from each admitted CONFIG RGBA.
# This is body occupancy, not a furnace-opening mask or an animation resource.
var _alpha_runs_by_key: Dictionary = {}
const PLAYER_OVERLAP_ALPHA: float = 0.25
const PLAYER_FADE_RATE: float = 3.75 # 0.75 opacity excursion / 200 ms (spec 150-250 ms).
const PLAYER_FADE_GAP_MAX_MS: int = 250
var _fade_owner: Array = []
var _fade_alpha: float = 1.0
var _fade_target: float = 1.0
var _fade_at_ms: int = -1
var _fade_frame_ms: int = -1
var _fade_seen: bool = false
var _fade_reduced_motion: bool = true
func _pair(v, x: float, y: float) -> bool:
	return v is Array and v.size()==2 and P.number(v[0],x,x) and P.number(v[1],y,y)
func prepare_resources() -> void:
	_load_once()
func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open('res://data/cw_furnaces.internal-use.json',FileAccess.READ)
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
	if not m is Dictionary or m.get('schema')!='wuxian.cw-furnaces.internal-use/v1' or typeof(m.get('registeredSourceReuse'))!=TYPE_BOOL or not m.registeredSourceReuse or m.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or m.get('GodotDisplay')!='NOT_RUN' or m.get('publicRelease')!='NOT_ESTABLISHED' or not m.get('assets') is Dictionary or m.assets.size()!=2:
		return
	for key in CONFIG:
		var c: Dictionary = CONFIG[key]
		var row = m.assets.get(key)
		var path = 'res://assets/props/clockworks/'+key+'.png'
		if not row is Dictionary or row.get('key')!=key or row.get('path')!=path or row.get('assetId')!=c.id or row.get('sourceId')!=c.id or row.get('sourceAdmission')!='release_approved' or row.get('sourceRuntimeStatus')!='ADOPT' or row.get('format')!='RGBA8' or not P.counter(row.get('bytes'),1,4194304) or row.bytes!=c.bytes or row.get('sha256')!=c.sha256 or not _pair(row.get('canvas'),c.width,c.height) or not _pair(row.get('anchor'),c.anchor[0],c.anchor[1]) or not P.number(row.get('heightM'),c.heightM,c.heightM):
			continue
		var png_file = FileAccess.open(path,FileAccess.READ)
		if png_file==null:
			continue
		if png_file.get_length()!=c.bytes:
			png_file.close()
			continue
		var png = png_file.get_buffer(png_file.get_length())
		png_file.close()
		var hash = HashingContext.new()
		hash.start(HashingContext.HASH_SHA256)
		hash.update(png)
		if hash.finish().hex_encode()!=c.sha256:
			continue
		var image = Image.new()
		if image.load_png_from_buffer(png)!=OK or image.get_width()!=c.width or image.get_height()!=c.height or image.get_format()!=Image.FORMAT_RGBA8:
			continue
		var texture = ImageTexture.create_from_image(image)
		if texture!=null:
			textures[key] = texture
			_cache_alpha_runs(key,image)
func canvas_rect(key: String, foot: Vector2, ppm: float) -> Rect2:
	if not CONFIG.has(key) or not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(ppm) or ppm<=0:
		return Rect2()
	var c: Dictionary = CONFIG[key]
	var anchor_value = c.get('anchor')
	if not anchor_value is Array or anchor_value.size()!=2 or not P.number(anchor_value[0],0,1) or not P.number(anchor_value[1],0,1) or not P.number(c.get('heightM'),0.001,100):
		return Rect2()
	var factor = float(c.heightM)*ppm/float(c.height)
	var size = Vector2(c.width,c.height)*factor
	var anchor = Vector2(anchor_value[0],anchor_value[1])
	var origin = foot-Vector2(c.width*anchor.x,c.height*anchor.y)*factor
	if not is_finite(factor) or factor<=0 or not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0 or not is_finite(origin.x) or not is_finite(origin.y):
		return Rect2()
	var rect = Rect2(origin,size)
	if not is_finite(rect.end.x) or not is_finite(rect.end.y):
		return Rect2()
	return rect
func _cache_alpha_runs(key: String, image: Image) -> void:
	var rows: Array = []
	# Once per admitted key: v1 1193*1254, v2 1165*1240 source pixels.
	for y in range(image.get_height()):
		var runs: PackedInt32Array = PackedInt32Array()
		var start: int = -1
		for x in range(image.get_width()):
			var occupied: bool = image.get_pixel(x,y).a>0.0
			if occupied and start<0:
				start = x
			elif not occupied and start>=0:
				runs.append(start)
				runs.append(x)
				start = -1
		if start>=0:
			runs.append(start)
			runs.append(image.get_width())
		rows.append(runs)
	_alpha_runs_by_key[key] = rows

func _positive_finite_rect(rect: Rect2) -> bool:
	return is_finite(rect.position.x) and is_finite(rect.position.y) and is_finite(rect.size.x) and is_finite(rect.size.y) and is_finite(rect.end.x) and is_finite(rect.end.y) and rect.size.x>0.0 and rect.size.y>0.0

func reset_player_fade() -> bool:
	var changed: bool = not _fade_owner.is_empty() or _fade_alpha!=1.0
	_fade_owner.clear()
	_fade_alpha = 1.0
	_fade_target = 1.0
	_fade_at_ms = -1
	_fade_frame_ms = -1
	_fade_seen = false
	_fade_reduced_motion = true
	return changed

func begin_player_fade_frame(owner: Array, now_ms: int, reduced_motion: bool) -> void:
	# No epoch/connection/visibility/camera or unconsumed-frame carryover.
	if owner.is_empty() or now_ms<0:
		reset_player_fade()
		return
	if owner!=_fade_owner or not _fade_seen or now_ms<_fade_frame_ms or now_ms-_fade_frame_ms>PLAYER_FADE_GAP_MAX_MS:
		reset_player_fade()
		_fade_owner = owner.duplicate(true)
		_fade_at_ms = now_ms
	_fade_frame_ms = now_ms
	_fade_reduced_motion = reduced_motion
	_fade_seen = false

func player_overlap_alpha(key: String, foot: Vector2, ppm: float, player_body: Rect2, viewport: Rect2) -> float:
	_load_once()
	if not CONFIG.has(key) or not textures.has(key) or not _alpha_runs_by_key.has(key) or _alpha_runs_by_key[key].size()!=CONFIG[key].height:
		reset_player_fade()
		return 1.0
	var target: float = _player_overlap_target(key,foot,ppm,player_body,viewport)
	if _fade_owner.is_empty() or _fade_frame_ms<0:
		return 1.0
	_fade_seen = true
	if _fade_reduced_motion:
		_fade_alpha = target
	elif target!=_fade_target:
		# First observation starts at the current alpha, never backdates an overlap.
		_fade_at_ms = _fade_frame_ms
	else:
		var elapsed_ms: int = _fade_frame_ms-_fade_at_ms
		if elapsed_ms<0 or elapsed_ms>PLAYER_FADE_GAP_MAX_MS:
			reset_player_fade()
			return 1.0
		_fade_alpha = move_toward(_fade_alpha,target,PLAYER_FADE_RATE*float(elapsed_ms)/1000.0)
	_fade_target = target
	_fade_at_ms = _fade_frame_ms
	return clampf(_fade_alpha,PLAYER_OVERLAP_ALPHA,1.0)

func _player_overlap_target(key: String, foot: Vector2, ppm: float, player_body: Rect2, viewport: Rect2) -> float:
	# Full PLAYER CANVAS is conservative; it is not an opaque player silhouette.
	# Unlike a canvas-only prop test, transparent furnace margins/holes do not qualify.
	if not CONFIG.has(key) or not P.number(ppm,0.001,3072) or not _positive_finite_rect(player_body) or not _positive_finite_rect(viewport):
		return 1.0
	var body: Rect2 = canvas_rect(key,foot,ppm)
	if not _positive_finite_rect(body):
		return 1.0
	var overlap: Rect2 = body.intersection(player_body).intersection(viewport)
	if not _positive_finite_rect(overlap):
		return 1.0
	_load_once()
	var c: Dictionary = CONFIG[key]
	if not textures.has(key) or not _alpha_runs_by_key.has(key) or _alpha_runs_by_key[key].size()!=c.height:
		return 1.0
	var rows: Array = _alpha_runs_by_key[key]
	var factor: float = body.size.y/float(c.height)
	var source_min: Vector2 = (overlap.position-body.position)/factor
	var source_max: Vector2 = (overlap.end-body.position)/factor
	if not is_finite(source_min.x) or not is_finite(source_min.y) or not is_finite(source_max.x) or not is_finite(source_max.y):
		return 1.0
	# Half-open source-pixel cells with positive-area overlap; no edge-touch fade.
	var x0: int = clampi(int(floor(source_min.x)),0,int(c.width))
	var x1: int = clampi(int(ceil(source_max.x)),0,int(c.width))
	var y0: int = clampi(int(floor(source_min.y)),0,int(c.height))
	var y1: int = clampi(int(ceil(source_max.y)),0,int(c.height))
	if x0>=x1 or y0>=y1:
		return 1.0
	for y in range(y0,y1):
		var runs: PackedInt32Array = rows[y]
		for i in range(0,runs.size(),2):
			if runs[i]>=x1:
				break
			if runs[i+1]>x0:
				return PLAYER_OVERLAP_ALPHA
	return 1.0

func draw_body(canvas: CanvasItem, key: String, foot: Vector2, ppm: float, opacity: float = 1.0) -> bool:
	var rect = canvas_rect(key,foot,ppm)
	if rect.size.x<=0 or rect.size.y<=0:
		return false
	_load_once()
	if not textures.has(key):
		return false
	var alpha: float = clampf(opacity,PLAYER_OVERLAP_ALPHA,1.0) if is_finite(opacity) else 1.0
	canvas.draw_texture_rect(textures[key],rect,false,Color(1.0,1.0,1.0,alpha))
	return true
