extends RefCounted
const PATHS = ['res://assets/characters/cenyao/walk_east/frame_01.png','res://assets/characters/cenyao/walk_east/frame_02.png','res://assets/characters/cenyao/walk_east/frame_03.png','res://assets/characters/cenyao/walk_east/frame_04.png']
var attempted := false
var frames: Array[Texture2D] = []
func _canvas_valid(value) -> bool:
	if not value is Array or value.size()!=2: return false
	for i in range(2):
		if typeof(value[i])!=TYPE_INT and typeof(value[i])!=TYPE_FLOAT: return false
		if not is_finite(float(value[i])): return false
	return value[0]==971 and value[1]==1620
func _load_once() -> void:
	if attempted: return
	attempted = true
	var f = FileAccess.open('res://data/cenyao_east_walk.local.json',FileAccess.READ)
	if f == null: return
	var m = JSON.parse_string(f.get_as_text())
	f.close()
	if not m is Dictionary or m.get('schema')!='wuxian.cenyao-east-walk.local/v1' or not _canvas_valid(m.get('canvas')) or m.get('anchor')!=[0.5,0.984375] or m.get('nominal_height')!=104 or m.get('reference_ppm')!=48 or m.get('fps')!=6 or m.get('frames')!=PATHS: return
	var loaded: Array[Texture2D] = []
	for item in PATHS:
		var file = FileAccess.open(item,FileAccess.READ)
		if file == null: return
		var bytes = file.get_buffer(file.get_length())
		file.close()
		var image = Image.new()
		if image.load_png_from_buffer(bytes)!=OK or image.get_width()!=971 or image.get_height()!=1620 or image.get_format()!=Image.FORMAT_RGBA8: return
		var texture = ImageTexture.create_from_image(image)
		if texture == null: return
		loaded.append(texture)
	if loaded.size()==4: frames = loaded
func draw_body(canvas: CanvasItem, foot: Vector2, ppm: float, seconds: float) -> bool:
	if not is_finite(ppm) or ppm<=0.0 or not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(seconds) or seconds<0.0: return false
	_load_once()
	if frames.size()!=4: return false
	var factor = (104.0/1620.0)*(ppm/48.0)
	if not is_finite(factor) or factor<=0.0: return false
	var size = Vector2(971,1620)*factor
	canvas.draw_texture_rect(frames[int(floor(seconds*6.0))%4],Rect2(foot-Vector2(0.5,0.984375)*size,size),false,Color.WHITE)
	return true
