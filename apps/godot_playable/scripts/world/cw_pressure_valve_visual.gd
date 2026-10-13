extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
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
func draw_body(canvas: CanvasItem, foot: Vector2, ppm: float) -> bool:
	if not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(ppm) or ppm<=0:
		return false
	var factor = 2.2*ppm/1179.0
	var size = Vector2(1210,1179)*factor
	var origin = foot-Vector2(605,1179)*factor
	if not is_finite(factor) or factor<=0 or not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0 or not is_finite(origin.x) or not is_finite(origin.y):
		return false
	_load_once()
	if texture==null:
		return false
	canvas.draw_texture_rect(texture,Rect2(origin,size),false,Color.WHITE)
	return true
