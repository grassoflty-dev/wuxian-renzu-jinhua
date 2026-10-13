extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const CONFIG = {
	'rs_terminal':{'id':'runtime2d.world.returnstation.terminal.v1','width':1071,'height':1218,'nominal':72},
	'beacon_folded':{'id':'runtime2d.prop.beacon.folded.v1','width':1226,'height':1005,'nominal':36},
	'beacon_deployed':{'id':'runtime2d.prop.beacon.deployed.v1','width':1333,'height':1054,'nominal':72}
}
var attempted = false
var textures: Dictionary = {}
func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open('res://data/registered_props.internal-use.json',FileAccess.READ)
	if file == null:
		return
	if file.get_length()>16384:
		file.close()
		return
	var bytes = file.get_buffer(file.get_length())
	file.close()
	if bytes.get_string_from_utf8().to_utf8_buffer()!=bytes:
		return
	var map = JSON.parse_string(bytes.get_string_from_utf8())
	if not map is Dictionary or map.get('schema')!='wuxian.registered-props.internal-use/v1' or typeof(map.get('registeredSourceReuse'))!=TYPE_BOOL or not map.registeredSourceReuse or map.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or map.get('GodotDisplay')!='NOT_RUN' or map.get('publicRelease')!='NOT_ESTABLISHED' or not map.get('assets') is Dictionary or map.assets.size()!=3:
		return
	for key in CONFIG:
		var row = map.assets.get(key)
		var c = CONFIG[key]
		var path = 'res://assets/props/'+key+'.png'
		if not row is Dictionary or row.get('path')!=path or row.get('assetId')!=c.id or not P.counter(row.get('bytes'),1,4194304) or not _hash(row.get('sha256')) or not _pair(row.get('canvas'),c.width,c.height) or not _pair(row.get('anchor'),0.5,1.0) or not P.number(row.get('nominalHeight'),c.nominal,c.nominal) or not P.number(row.get('referencePpm'),48,48):
			continue
		var png_file = FileAccess.open(path,FileAccess.READ)
		if png_file == null:
			continue
		if png_file.get_length()!=row.bytes:
			png_file.close()
			continue
		var png = png_file.get_buffer(png_file.get_length())
		png_file.close()
		var hash = HashingContext.new()
		hash.start(HashingContext.HASH_SHA256)
		hash.update(png)
		if hash.finish().hex_encode()!=row.sha256:
			continue
		var image = Image.new()
		if image.load_png_from_buffer(png)!=OK or image.get_width()!=c.width or image.get_height()!=c.height or image.get_format()!=Image.FORMAT_RGBA8:
			continue
		var texture = ImageTexture.create_from_image(image)
		if texture != null:
			textures[key] = texture
func _pair(v, first: float, second: float) -> bool:
	return v is Array and v.size()==2 and P.number(v[0],first,first) and P.number(v[1],second,second)
func _hash(v) -> bool:
	if typeof(v)!=TYPE_STRING or v.length()!=64:
		return false
	for c in v:
		if not c in '0123456789abcdef':
			return false
	return true
func canvas_height(key: String, ppm: float) -> float:
	return float(CONFIG[key].nominal)*(ppm/48.0) if CONFIG.has(key) and is_finite(ppm) and ppm>0 else 0.0
func draw_body(canvas: CanvasItem, key: String, foot: Vector2, ppm: float) -> bool:
	if not CONFIG.has(key) or not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(ppm) or ppm<=0:
		return false
	_load_once()
	if not textures.has(key):
		return false
	var c = CONFIG[key]
	var factor = (float(c.nominal)/float(c.height))*(ppm/48.0)
	if not is_finite(factor) or factor<=0:
		return false
	canvas.draw_texture_rect(textures[key],Rect2(foot-Vector2(c.width*0.5,c.height)*factor,Vector2(c.width,c.height)*factor),false,Color.WHITE)
	return true
