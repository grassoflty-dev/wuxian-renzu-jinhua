extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const CONFIG = {"gate_b":{"derived":"assets/derived/asset-pipeline-v1/sprites/png/runtime2d.prop.gate_b.v1.png","bytes":2397332,"sha256":"fdad9675bb90b37bc3c452773934536918fa56f4bef2b6f95c10a9823e2a67f6","source":"batch02/07_runtime2d.prop.gate_b.v1.png","path":"res://assets/props/grey_hive/gate_b.png","states":{"closed":[0,0,1448,1073]},"sourceSha256":"433a56c7960d19e09fb77ea55aa1787197e4a2eecc12face0ab29d5eaac39450","sourceBytes":2442085,"heightM":2.6,"canvas":[1448,1073],"id":"runtime2d.prop.gate_b.v1"},"power_console":{"derived":"assets/derived/asset-pipeline-v1/sprites/png/runtime2d.prop.power_console.off_on.v1.png","bytes":2311441,"sha256":"740eaaf0d0b9c0b1ba3b9f2c6650ffc3e270e70e15d0d979f2637df4092f048c","source":"batch01/06_runtime2d.prop.power_console.off_on.v1.png","path":"res://assets/props/grey_hive/power_console_sheet.png","states":{"off":[24,2,718,918],"on":[752,2,696,918]},"sourceSha256":"4cc2443664940a54e776e4cfd8d682b8477a1848dc420b2df4990d8d398f9961","sourceBytes":2348980,"heightM":1.45,"canvas":[1448,1018],"id":"runtime2d.prop.power_console.off_on.v1"},"gate_a":{"derived":"assets/derived/asset-pipeline-v1/sprites/png/runtime2d.prop.gate_a.closed_open.v1.png","bytes":1386982,"sha256":"0fa3981a0abfa5a1f7a5d4b75033f392e46f1b91f901605dcacf415c796e1b7b","source":"batch01/07_runtime2d.prop.gate_a.closed_open.v1.png","path":"res://assets/props/grey_hive/gate_a_sheet.png","states":{"closed":[0,216,690,625],"open":[724,216,683,625]},"sourceSha256":"7d4b352063775818773e3bbda6ae1296645e09a84f2a2940ae17075be8c00fed","sourceBytes":1429841,"heightM":2.6,"canvas":[1407,1039],"id":"runtime2d.prop.gate_a.closed_open.v1"}}
var attempted = false
var textures: Dictionary = {}
func _equal_numbers(v, expected: Array) -> bool:
	if not v is Array or v.size()!=expected.size():
		return false
	for n in range(expected.size()):
		if not P.number(v[n],expected[n],expected[n]):
			return false
	return true
func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open('res://data/grey_hive_state_props.internal-use.json',FileAccess.READ)
	if file==null:
		return
	if file.get_length()==0 or file.get_length()>16384:
		file.close()
		return
	var bytes = file.get_buffer(file.get_length())
	file.close()
	if bytes.get_string_from_utf8().to_utf8_buffer()!=bytes:
		return
	var map = JSON.parse_string(bytes.get_string_from_utf8())
	if not map is Dictionary or map.get('schema')!='wuxian.grey-hive-state-props.internal-use/v1' or typeof(map.get('registeredSourceReuse'))!=TYPE_BOOL or not map.registeredSourceReuse or map.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or map.get('GodotDisplay')!='NOT_RUN' or map.get('publicRelease')!='NOT_ESTABLISHED' or not map.get('assets') is Dictionary or map.assets.size()!=3:
		return
	for key in CONFIG:
		var c: Dictionary = CONFIG[key]
		var row = map.assets.get(key)
		if not row is Dictionary or row.get('assetId')!=c.id or row.get('sourceId')!=c.id or row.get('path')!=c.path or row.get('source')!=c.source or row.get('derived')!=c.derived or not P.counter(row.get('bytes'),c.bytes,c.bytes) or c.bytes>4194304 or row.get('sha256')!=c.sha256 or not P.counter(row.get('sourceBytes'),c.sourceBytes,c.sourceBytes) or row.get('sourceSha256')!=c.sourceSha256 or row.get('admission')!='release_approved' or row.get('runtimeStatus')!='ADOPT' or typeof(row.get('registeredSourceReuse'))!=TYPE_BOOL or not row.registeredSourceReuse or row.get('format')!='RGBA8' or not _equal_numbers(row.get('canvas'),c.canvas) or not _equal_numbers(row.get('anchor'),[0.5,1]) or not P.number(row.get('heightM'),c.heightM,c.heightM) or not row.get('states') is Dictionary or row.states.size()!=c.states.size():
			continue
		var valid = true
		for state in c.states:
			var rect: Array = c.states[state]
			if not _equal_numbers(row.states.get(state),rect) or rect[0]<0 or rect[1]<0 or rect[2]<=0 or rect[3]<=0 or rect[0]+rect[2]>c.canvas[0] or rect[1]+rect[3]>c.canvas[1]:
				valid = false
		if not valid:
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
		if hash.start(HashingContext.HASH_SHA256)!=OK or hash.update(png)!=OK or hash.finish().hex_encode()!=c.sha256:
			continue
		var image = Image.new()
		if image.load_png_from_buffer(png)!=OK or image.get_width()!=c.canvas[0] or image.get_height()!=c.canvas[1] or image.get_format()!=Image.FORMAT_RGBA8:
			continue
		var texture = ImageTexture.create_from_image(image)
		if texture!=null:
			textures[key] = texture
func body_height(key: String, ppm: float) -> float:
	return float(CONFIG[key].heightM)*ppm if CONFIG.has(key) and is_finite(ppm) and ppm>0 else 0.0
func draw_body(canvas: CanvasItem, key: String, state: String, foot: Vector2, ppm: float) -> bool:
	if not CONFIG.has(key) or not CONFIG[key].states.has(state) or not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(ppm) or ppm<=0:
		return false
	_load_once()
	if not textures.has(key):
		return false
	var c: Dictionary = CONFIG[key]
	var region: Array = c.states[state]
	var factor = float(c.heightM)*ppm/float(region[3])
	var size = Vector2(region[2],region[3])*factor
	var origin = foot-Vector2(region[2]*0.5,region[3])*factor
	if not is_finite(factor) or factor<=0 or not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0 or not is_finite(origin.x) or not is_finite(origin.y):
		return false
	var texture: ImageTexture = textures[key]
	canvas.draw_texture_rect_region(texture,Rect2(origin,size),Rect2(region[0],region[1],region[2],region[3]),Color.WHITE,false,true)
	return true