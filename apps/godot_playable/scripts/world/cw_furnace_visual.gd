extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const CONFIG = {
	'furnace_v1':{'id':'runtime2d.world.clockworks.furnace.v1','width':1193,'height':1254,'bytes':1938854,'sha256':'4a36e98fec3057bac6e59e01bcd2287ee5e412071a5396731c50082a351aec44'},
	'furnace_v2':{'id':'runtime2d.world.clockworks.furnace.v2','width':1165,'height':1240,'bytes':1584389,'sha256':'207e2b4c1759e49bfd03613958f86787586410dc0abe6da526f34679f17cc74d'}
}
var attempted = false
var textures: Dictionary = {}
func _pair(v, x: float, y: float) -> bool:
	return v is Array and v.size()==2 and P.number(v[0],x,x) and P.number(v[1],y,y)
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
		if not row is Dictionary or row.get('key')!=key or row.get('path')!=path or row.get('assetId')!=c.id or row.get('sourceId')!=c.id or row.get('sourceAdmission')!='release_approved' or row.get('sourceRuntimeStatus')!='ADOPT' or row.get('format')!='RGBA8' or not P.counter(row.get('bytes'),1,4194304) or row.bytes!=c.bytes or row.get('sha256')!=c.sha256 or not _pair(row.get('canvas'),c.width,c.height) or not _pair(row.get('anchor'),0.5,1) or not P.number(row.get('heightM'),4,4):
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
func draw_body(canvas: CanvasItem, key: String, foot: Vector2, ppm: float) -> bool:
	if not CONFIG.has(key) or not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(ppm) or ppm<=0:
		return false
	var c: Dictionary = CONFIG[key]
	var factor = 4.0*ppm/float(c.height)
	var size = Vector2(c.width,c.height)*factor
	var origin = foot-Vector2(c.width*0.5,c.height)*factor
	if not is_finite(factor) or factor<=0 or not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0 or not is_finite(origin.x) or not is_finite(origin.y):
		return false
	_load_once()
	if not textures.has(key):
		return false
	canvas.draw_texture_rect(textures[key],Rect2(origin,size),false,Color.WHITE)
	return true
