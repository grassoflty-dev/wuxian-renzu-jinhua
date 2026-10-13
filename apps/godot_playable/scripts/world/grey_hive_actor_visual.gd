extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const CONFIG = {
	'maintenance':{'id':'runtime2d.enemy.infected_maintenance_worker.v1','path':'res://assets/enemies/grey_hive/maintenance_sheet.png','bytes':1870188,'sha':'f7ee9460d18be46a8aea81fda2e566b29e5ac7b80d9ad2b20fd58f2f7e7a8168','source_sha':'6b7055df24acfa58edfdf0342c67efb60f9fac892c80110d449700bf7021477d','canvas':[1447,1086],'rect':[17,3,428,522],'foot':[266,511],'height':1.72},
	'security':{'id':'runtime2d.enemy.infected_security.v1','path':'res://assets/enemies/grey_hive/security_sheet.png','bytes':1827592,'sha':'cee3c171a14d82317e98ef00f39c3341a16364931dd8d56ec0ced330bcbecf66','source_sha':'7bd9d4cf19f36d189cdf81530c3dee48450179696a7dbceb24eaf375714de4e4','canvas':[1407,1072],'rect':[14,0,332,542],'foot':[190,537],'height':1.72},
	'sentinel':{'id':'runtime2d.enemy.sentinel.base.v1','path':'res://assets/enemies/grey_hive/sentinel_sheet.png','bytes':2160789,'sha':'c490ace4eb8ec9bf27e176359aefb8c3cd36c55a0651452be39ef25e4864df1d','source_sha':'f7ced75a0fcf35afacf796f23630a40c309682c86ef67af79de5414a8a16fbe2','canvas':[1442,1072],'rect':[10,0,386,541],'foot':[205,531],'height':2.6}
}
var attempted = false
var textures: Dictionary = {}
func _integers(value, expected: Array) -> bool:
	if not value is Array or value.size()!=expected.size():
		return false
	for n in range(expected.size()):
		if not P.counter(value[n],0,4096) or value[n]!=expected[n]:
			return false
	return true
func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open('res://data/grey_hive_actor.internal-use.json',FileAccess.READ)
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
	if not mapping is Dictionary or mapping.get('schema')!='wuxian.grey-hive-actor.internal-use/v1' or typeof(mapping.get('registeredSourceReuse'))!=TYPE_BOOL or not mapping.registeredSourceReuse or mapping.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or mapping.get('GodotDisplay')!='NOT_RUN' or mapping.get('publicRelease')!='NOT_ESTABLISHED' or not mapping.get('assets') is Dictionary or mapping.assets.size()!=3:
		return
	for key in CONFIG:
		var c: Dictionary = CONFIG[key]
		var row = mapping.assets.get(key)
		if not row is Dictionary or row.get('path')!=c.path or row.get('assetId')!=c.id or not P.counter(row.get('bytes'),1,4194304) or row.bytes!=c.bytes or row.get('sha256')!=c.sha or typeof(row.get('temporary_static_actor'))!=TYPE_BOOL or not row.temporary_static_actor or not _integers(row.get('canvas'),c.canvas) or not _integers(row.get('rect'),c.rect) or not _integers(row.get('absoluteFoot'),c.foot) or not P.number(row.get('visualHeightM'),c.height,c.height):
			continue
		var source = row.get('source')
		if not source is Dictionary or source.get('admission')!='release_approved' or source.get('runtimeStatus')!='ADOPT' or source.get('sha256')!=c.source_sha:
			continue
		if c.rect[2]<=0 or c.rect[3]<=0 or c.rect[0]+c.rect[2]>c.canvas[0] or c.rect[1]+c.rect[3]>c.canvas[1] or c.foot[0]<c.rect[0] or c.foot[0]>=c.rect[0]+c.rect[2] or c.foot[1]<c.rect[1] or c.foot[1]>=c.rect[1]+c.rect[3]:
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
	var factor = float(c.height)*ppm/float(c.rect[3])
	var size = Vector2(c.rect[2],c.rect[3])*factor
	var origin = foot-Vector2(c.foot[0]-c.rect[0],c.foot[1]-c.rect[1])*factor
	if not is_finite(factor) or factor<=0 or not is_finite(origin.x) or not is_finite(origin.y) or not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0:
		return false
	canvas.draw_texture_rect_region(textures[key],Rect2(origin,size),Rect2(c.rect[0],c.rect[1],c.rect[2],c.rect[3]),Color.WHITE,false,true)
	return true