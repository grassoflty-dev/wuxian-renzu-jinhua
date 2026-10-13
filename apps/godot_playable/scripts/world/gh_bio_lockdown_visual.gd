extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const CONFIG = {
	'bio_pod':{'id':'runtime2d.prop.bio_pod.v1','width':1171,'height':1254,'height_m':2.4,'bytes':1371757,'sha256':'d71d51dea193527c71262ccb54d7d7b14d80e43346b017dede9a4122e2589588'},
	'medical_station':{'id':'runtime2d.prop.medical_station.v1','width':969,'height':1215,'height_m':1.8,'bytes':1418293,'sha256':'de4fdadfd8b9fc21afd1bf430a1da846129fa09b2a303a064eaedeb8552dc0f2'},
	'lockdown_terminal':{'id':'runtime2d.prop.lockdown_terminal.v1','width':1270,'height':1086,'height_m':1.4,'bytes':1870319,'sha256':'e917f9b4677d49e6d534b6b5f32b8756fb2934dbe4e87e76391c09ccd786361c'}
}
var attempted = false
var textures: Dictionary = {}
func _pair(value, first: float, second: float) -> bool:
	return value is Array and value.size()==2 and P.number(value[0],first,first) and P.number(value[1],second,second)
func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open('res://data/gh_bio_lockdown.internal-use.json',FileAccess.READ)
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
	if not mapping is Dictionary or mapping.get('schema')!='wuxian.gh-bio-lockdown.internal-use/v1' or typeof(mapping.get('registeredSourceReuse'))!=TYPE_BOOL or not mapping.registeredSourceReuse or mapping.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or mapping.get('GodotDisplay')!='NOT_RUN' or mapping.get('publicRelease')!='NOT_ESTABLISHED' or not mapping.get('assets') is Dictionary or mapping.assets.size()!=3:
		return
	for key in CONFIG:
		var row = mapping.assets.get(key)
		var c: Dictionary = CONFIG[key]
		var path = 'res://assets/props/grey_hive/'+key+'.png'
		if not row is Dictionary or row.get('path')!=path or row.get('assetId')!=c.id or row.get('sourceId')!=c.id or row.get('sourceAdmission')!='release_approved' or row.get('sourceRuntimeStatus')!='ADOPT' or row.get('format')!='RGBA8' or not P.counter(row.get('bytes'),1,4194304) or row.bytes!=c.bytes or row.get('sha256')!=c.sha256 or not _pair(row.get('canvas'),c.width,c.height) or not _pair(row.get('anchor'),0.5,1) or not P.number(row.get('heightM'),c.height_m,c.height_m):
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
	_load_once()
	if not textures.has(key):
		return false
	var c: Dictionary = CONFIG[key]
	var factor = float(c.height_m)*ppm/float(c.height)
	if not is_finite(factor) or factor<=0:
		return false
	canvas.draw_texture_rect(textures[key],Rect2(foot-Vector2(c.width*0.5,c.height)*factor,Vector2(c.width,c.height)*factor),false,Color.WHITE)
	return true
