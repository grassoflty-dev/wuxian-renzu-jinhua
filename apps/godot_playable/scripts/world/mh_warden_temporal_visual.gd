extends RefCounted
# Separate Warden identity and visibility are owned by the caller.
const PATH := 'res://assets/enemies/mist_harbor/signal_wraith_temporal_local_r1.png'
var _texture: Texture2D
var _attempted := false
var _scope := ''
var _previous: Dictionary = {}

func reset() -> void:
	_scope='';_previous.clear()

func _load_once() -> Texture2D:
	if _attempted: return _texture
	_attempted=true
	var texture: Texture2D = null
	if ResourceLoader.exists(PATH): texture=load(PATH) as Texture2D
	if texture==null:
		var image=Image.new()
		if image.load(PATH)==OK and image.get_format()==Image.FORMAT_RGBA8: texture=ImageTexture.create_from_image(image)
	if texture!=null and texture.get_size()==Vector2(1254,1254):
		var image=texture.get_image()
		if image!=null and image.get_format()==Image.FORMAT_RGBA8: _texture=texture
	return _texture

func select_cell(scope: String, tick: int, server_ms: int, position: Vector3, entitled_warning: bool, animate: bool) -> int:
	if scope.is_empty() or tick<0 or server_ms<0 or not position.is_finite(): reset();return -1
	if scope!=_scope: reset();_scope=scope
	var moved=false
	if not _previous.is_empty():
		if tick<_previous.tick or server_ms<_previous.time: reset();return -1
		if tick==_previous.tick and server_ms==_previous.time:
			if position!=_previous.position: reset();return -1
			moved=_previous.moved
		elif tick>_previous.tick and server_ms>_previous.time:
			moved=position.distance_squared_to(_previous.position)>0.00000001
		else: reset();return -1
	_previous={'tick':tick,'time':server_ms,'position':position,'moved':moved}
	var cell=-1
	if animate and entitled_warning: cell=2
	elif animate and moved: cell=int(server_ms/166)%2
	return cell

func draw_body(canvas: CanvasItem, scope: String, tick: int, server_ms: int, position: Vector3, foot: Vector2, ppm: float, entitled_warning: bool, animate: bool) -> bool:
	if not is_instance_valid(canvas) or not foot.is_finite() or not is_finite(ppm) or ppm<=0: reset();return false
	var cell=select_cell(scope,tick,server_ms,position,entitled_warning,animate)
	if cell<0: return false
	var texture=_load_once()
	if texture==null: return false
	var factor=1.72*ppm/600.0
	if not is_finite(factor) or factor<=0: return false
	canvas.draw_texture_rect_region(texture,Rect2(foot-Vector2(384,610)*factor,Vector2(627,627)*factor),Rect2(Vector2((cell%2)*627,floori(float(cell)/2.0)*627),Vector2(627,627)),Color.WHITE,false,true)
	return true
