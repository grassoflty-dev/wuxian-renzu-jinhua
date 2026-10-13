extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const DIRECTIONS = ['east','south_east','south','south_west','west','north_west','north','north_east']
const CANVAS = Vector2(336,560)
const ANCHOR = Vector2(168,551)
var attempted = false
var textures: Dictionary = {}

func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open('res://data/cenyao_idle.internal-use.json',FileAccess.READ)
	if file == null:
		return
	if file.get_length()>16384:
		file.close()
		return
	var bytes = file.get_buffer(file.get_length())
	file.close()
	if bytes.get_string_from_utf8().to_utf8_buffer()!=bytes:
		return
	var mapping = JSON.parse_string(bytes.get_string_from_utf8())
	if not mapping is Dictionary or mapping.get('schema')!='wuxian.cenyao-idle.internal-use/v1' or mapping.get('assetId')!='runtime2d.actor.cenyao.base.v1' or typeof(mapping.get('registeredSourceReuse'))!=TYPE_BOOL or not mapping.registeredSourceReuse or mapping.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or mapping.get('GodotDisplay')!='NOT_RUN' or mapping.get('publicRelease')!='NOT_ESTABLISHED':
		return
	if not _pair(mapping.get('canvas'),336,560) or not _pair(mapping.get('footAnchor'),168,551) or not P.counter(mapping.get('nominalCanvasHeight'),104,104) or not P.counter(mapping.get('referencePpm'),48,48) or not mapping.get('frames') is Dictionary or mapping.frames.size()!=8:
		return
	for direction in DIRECTIONS:
		var row = mapping.frames.get(direction)
		var expected = 'res://assets/characters/cenyao/idle/cenyao_idle_'+direction+'.png'
		if not row is Dictionary or row.get('path')!=expected or not P.counter(row.get('bytes'),1,4194304) or not _hash(row.get('sha256')) or row.get('state')!='idle' or not P.counter(row.get('frameCount'),1,1):
			continue
		var image_file = FileAccess.open(expected,FileAccess.READ)
		if image_file == null:
			continue
		if image_file.get_length()!=row.bytes:
			image_file.close()
			continue
		var png = image_file.get_buffer(image_file.get_length())
		image_file.close()
		var hash = HashingContext.new()
		hash.start(HashingContext.HASH_SHA256)
		hash.update(png)
		if hash.finish().hex_encode()!=row.sha256:
			continue
		var image = Image.new()
		if image.load_png_from_buffer(png)!=OK or image.get_width()!=336 or image.get_height()!=560 or image.get_format()!=Image.FORMAT_RGBA8:
			continue
		var texture = ImageTexture.create_from_image(image)
		if texture != null:
			textures[direction] = texture

func _pair(value, first: int, second: int) -> bool:
	return value is Array and value.size()==2 and P.counter(value[0],first,first) and P.counter(value[1],second,second)

func _hash(value) -> bool:
	if typeof(value)!=TYPE_STRING or value.length()!=64:
		return false
	for c in value:
		if not c in '0123456789abcdef':
			return false
	return true

func draw_body(canvas: CanvasItem, foot: Vector2, player: Dictionary, ppm: float) -> bool:
	# Static pose only: current authority chooses orientation; there is no animation clock.
	if not is_finite(ppm) or ppm<=0 or not is_finite(foot.x) or not is_finite(foot.y) or not P.counter(player.get('currentHp'),1,4294967295) or not P.transform(player.get('transform')) or not P.number(player.get('facingX'),-1,1) or not P.number(player.get('facingZ'),-1,1):
		return false
	var facing = Vector2(player.facingX-player.facingZ,(player.facingX+player.facingZ)*0.5)
	if not is_finite(facing.x) or not is_finite(facing.y) or facing.length_squared()==0:
		return false
	var sector = int(floor(atan2(facing.y,facing.x)/(PI/4.0)+0.5))
	var direction = DIRECTIONS[posmod(sector,8)]
	_load_once()
	if not textures.has(direction):
		return false
	var factor = (104.0/560.0)*(ppm/48.0)
	if not is_finite(factor) or factor<=0:
		return false
	canvas.draw_texture_rect(textures[direction],Rect2(foot-ANCHOR*factor,CANVAS*factor),false,Color.WHITE)
	return true
