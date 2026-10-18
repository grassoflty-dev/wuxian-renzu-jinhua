extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const PATH = 'res://assets/props/grey_hive/supply_crates_sheet.png'
const ID = 'runtime2d.prop.supply_crates.v1'
const SHA = '0dbad1364c14649b021f20510b9faf970ff317e6202131bd4cb2c32648b34b00'
const CUTOUT = [[250,0],[539,20],[539,440],[295,549],[15,420],[15,20]]
var attempted = false
var texture: Texture2D
func _integers(value, expected: Array) -> bool:
	if not value is Array or value.size()!=expected.size():
		return false
	for n in expected.size():
		if not P.counter(value[n],0,4096) or value[n]!=expected[n]:
			return false
	return true
func prepare_resources() -> void:
	_load_once()
func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open('res://data/gh_supply_crates.internal-use.json',FileAccess.READ)
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
	if not m is Dictionary or m.get('schema')!='wuxian.gh-supply-crates.internal-use/v1' or typeof(m.get('registeredSourceReuse'))!=TYPE_BOOL or not m.registeredSourceReuse or m.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or m.get('GodotDisplay')!='NOT_RUN' or m.get('publicRelease')!='NOT_ESTABLISHED':
		return
	var row = m.get('asset')
	if not row is Dictionary or row.get('path')!=PATH or row.get('assetId')!=ID or row.get('sourceId')!=ID or row.get('sourceAdmission')!='release_approved' or row.get('sourceRuntimeStatus')!='ADOPT' or row.get('format')!='RGBA8' or not P.counter(row.get('bytes'),1,4194304) or row.bytes!=2067549 or row.get('sha256')!=SHA or not _integers(row.get('canvas'),[1238,1233]) or not _integers(row.get('region'),[0,640,540,550]) or not _integers(row.get('anchor'),[270,550]) or not P.number(row.get('heightM'),1,1):
		return
	var cutout = row.get('cutout')
	if not cutout is Array or cutout.size()!=6:
		return
	for n in 6:
		if not _integers(cutout[n],CUTOUT[n]) or cutout[n][0]>=540 or cutout[n][1]>=550:
			return
	var png_file = FileAccess.open(PATH,FileAccess.READ)
	if png_file==null:
		return
	if png_file.get_length()!=2067549:
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
	if image.load_png_from_buffer(png)!=OK or image.get_width()!=1238 or image.get_height()!=1233 or image.get_format()!=Image.FORMAT_RGBA8:
		return
	texture = ImageTexture.create_from_image(image)
func draw_body(canvas: CanvasItem, foot: Vector2, ppm: float) -> bool:
	if not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(ppm) or ppm<=0:
		return false
	var factor = ppm/550.0
	if not is_finite(factor) or factor<=0:
		return false
	_load_once()
	if texture==null:
		return false
	var points = PackedVector2Array()
	var uvs = PackedVector2Array()
	for pair in CUTOUT:
		var q = Vector2(pair[0],pair[1])
		var point = foot+(q-Vector2(270,550))*factor
		var uv = (q+Vector2(0,640))/Vector2(1238,1233)
		if not is_finite(point.x) or not is_finite(point.y) or not is_finite(uv.x) or not is_finite(uv.y):
			return false
		points.append(point)
		uvs.append(uv)
	canvas.draw_colored_polygon(points,Color.WHITE,uvs,texture)
	return true
