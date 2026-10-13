extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const CONFIG = {"drowned":{"path":"res://assets/enemies/mist_harbor/drowned.png","sha":"752ac830587fc957d5695cd7714c57e584f40ec1a8f511f324ee491adaee7f2d","id":"runtime2d.enemy.mistharbor.drowned.v2","source_sha":"1b57a57c4fc7dc1d853b82cc1c6a3525e97af5853006ad17a8e25d0de6ca993e","canvas":[1221,1254],"bytes":1128162,"height":1.72,"source_bytes":1157176},"forged_guard":{"path":"res://assets/enemies/clockworks/forged_guard.png","sha":"b41cdc83c587805d0beee9d48668a9328001882595e0ab97fa69347d9bcf8cb4","id":"runtime2d.enemy.clockworks.forged_guard.v2","source_sha":"5c542927516e2e393eccada493e5ca4e20d4bc5d75db8431d57f306910c5d0c0","canvas":[1226,1254],"bytes":1300721,"height":1.72,"source_bytes":1330037},"signal_wraith":{"path":"res://assets/enemies/mist_harbor/signal_wraith.png","sha":"942a294e86076bfa91762113cd9bb14c81db329baabf5804ba9138942cc7166b","id":"runtime2d.enemy.mistharbor.signal_wraith.v2","source_sha":"fcaad0d40dc3a9dca4c24d147c656d4c751b6a729d265ae3f60d421fe69dd055","canvas":[1236,1254],"bytes":1440594,"height":1.72,"source_bytes":1470333}}
var attempted = false
var textures: Dictionary = {}
func _canvas(v, expected: Array) -> bool:
	return v is Array and v.size()==2 and P.counter(v[0],1,4096) and P.counter(v[1],1,4096) and v==expected
func _anchor(v) -> bool:
	return v is Array and v.size()==2 and P.number(v[0],0.5,0.5) and P.number(v[1],1,1)
func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open('res://data/mh_cw_actor.internal-use.json',FileAccess.READ)
	if file==null:
		return
	if file.get_length()<=0 or file.get_length()>16384:
		file.close()
		return
	var bytes = file.get_buffer(file.get_length())
	file.close()
	if bytes.get_string_from_utf8().to_utf8_buffer()!=bytes:
		return
	var mapping = JSON.parse_string(bytes.get_string_from_utf8())
	if not mapping is Dictionary or mapping.get('schema')!='wuxian.mh-cw-actor.internal-use/v1' or typeof(mapping.get('registeredSourceReuse'))!=TYPE_BOOL or not mapping.registeredSourceReuse or mapping.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or mapping.get('GodotDisplay')!='NOT_RUN' or mapping.get('publicRelease')!='NOT_ESTABLISHED' or not mapping.get('assets') is Dictionary or mapping.assets.size()!=3:
		return
	for key in CONFIG:
		var c: Dictionary = CONFIG[key]
		var row = mapping.assets.get(key)
		if not row is Dictionary or row.get('path')!=c.path or row.get('assetId')!=c.id or not P.counter(row.get('bytes'),1,4194304) or row.bytes!=c.bytes or row.get('sha256')!=c.sha or typeof(row.get('temporary_static_actor'))!=TYPE_BOOL or not row.temporary_static_actor or not _canvas(row.get('canvas'),c.canvas) or not _anchor(row.get('anchor')) or not P.number(row.get('visualHeightM'),c.height,c.height):
			continue
		var source = row.get('source')
		if not source is Dictionary or source.get('assetId')!=c.id or source.get('admission')!='release_approved' or source.get('runtimeStatus')!='ADOPT' or source.get('sha256')!=c.source_sha or not P.counter(source.get('bytes'),1,4194304) or source.bytes!=c.source_bytes:
			continue
		var png_file = FileAccess.open(c.path,FileAccess.READ)
		if png_file==null:
			continue
		if png_file.get_length()!=c.bytes:
			png_file.close()
			continue
		var png = png_file.get_buffer(png_file.get_length())
		png_file.close()
		var hash = HashingContext.new()
		if hash.start(HashingContext.HASH_SHA256)!=OK or hash.update(png)!=OK or hash.finish().hex_encode()!=c.sha:
			continue
		var image = Image.new()
		if image.load_png_from_buffer(png)!=OK or image.get_width()!=c.canvas[0] or image.get_height()!=c.canvas[1] or image.get_format()!=Image.FORMAT_RGBA8:
			continue
		var texture = ImageTexture.create_from_image(image)
		if texture!=null:
			textures[key] = texture
func draw_body(canvas: CanvasItem, key: String, foot: Vector2, ppm: float) -> bool:
	if not CONFIG.has(key) or not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(ppm) or ppm<=0:
		return false
	_load_once()
	if not textures.has(key):
		return false
	var c: Dictionary = CONFIG[key]
	var factor = float(c.height)*ppm/float(c.canvas[1])
	var size = Vector2(c.canvas[0],c.canvas[1])*factor
	var origin = foot-Vector2(c.canvas[0]*0.5,c.canvas[1])*factor
	if not is_finite(factor) or factor<=0 or not is_finite(origin.x) or not is_finite(origin.y) or not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0:
		return false
	canvas.draw_texture_rect(textures[key],Rect2(origin,size),false,Color.WHITE)
	return true