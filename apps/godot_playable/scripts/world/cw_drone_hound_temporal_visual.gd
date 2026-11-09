extends RefCounted

# Authoritative actor position is always the image anchor; events only select a pose.
const DRONE_PATH := 'res://assets/actors/cw_pressure_drone_temporal_local_r1.png'
const HOUND_PATH := 'res://assets/actors/cw_furnace_hound_temporal_local_r1.png'
const DRONE_SIZE := Vector2(1254,1254)
const HOUND_SIZE := Vector2(1254,1254)
var _textures: Dictionary = {}
var _attempted: Dictionary = {}
var _previous: Dictionary = {}
var _scope := ''

func _texture(family: String) -> Texture2D:
	if _attempted.has(family): return _textures.get(family)
	_attempted[family] = true
	var path: String = DRONE_PATH if family=='pressure_drone' else HOUND_PATH
	var expected: Vector2 = DRONE_SIZE if family=='pressure_drone' else HOUND_SIZE
	if expected.x<=0 or expected.y<=0: return null
	var texture: Texture2D = null
	if ResourceLoader.exists(path): texture=load(path) as Texture2D
	if texture==null:
		var image=Image.new()
		if image.load(path)==OK and image.get_format()==Image.FORMAT_RGBA8: texture=ImageTexture.create_from_image(image)
	if texture!=null and texture.get_size()==expected:
		var image=texture.get_image()
		if image!=null and image.get_format()==Image.FORMAT_RGBA8: _textures[family]=texture
	return _textures.get(family)

func draw_body(canvas: CanvasItem, family: String, scope: String, tick: int, server_ms: int, actor: Dictionary, foot: Vector2, ppm: float, projected_events: Array, animate: bool) -> bool:
	if family not in ['pressure_drone','furnace_hound'] or not is_instance_valid(canvas) or scope.is_empty() or tick<0 or server_ms<0 or not foot.is_finite() or not is_finite(ppm) or ppm<=0: return false
	if scope!=_scope: _scope=scope; _previous.clear()
	var position=actor.get('transform',{}).get('positionM',{})
	if not position is Dictionary: _previous.clear(); return false
	for key in ['xM','yM','zM']:
		if not (position.get(key) is int or position.get(key) is float) or not is_finite(float(position[key])): _previous.clear(); return false
	var p=Vector3(float(position.xM),float(position.yM),float(position.zM))
	var moved=false
	if not _previous.is_empty():
		if tick<_previous.tick or server_ms<_previous.time: _previous.clear(); return false
		if tick==_previous.tick and server_ms==_previous.time:
			if p!=_previous.position: _previous.clear(); return false
			moved=_previous.moved
		elif tick>_previous.tick and server_ms>_previous.time:
			moved=p.distance_squared_to(_previous.position)>0.00000001
		else: _previous.clear(); return false
	_previous={'tick':tick,'time':server_ms,'position':p,'moved':moved}
	var cell=0
	if animate:
		if moved: cell=int(server_ms/166)%2
		for event in projected_events:
			if event.kind=='EnemyAttackTelegraph': cell=2
			elif event.kind in ['PressureShotMotion','FurnaceHoundLeapMotion','EnemyAttackImpact']: cell=3
	var texture=_texture(family)
	if texture==null: return false
	var size=texture.get_size()/2.0
	var metres=(0.9 if family=='pressure_drone' else 1.0)
	var factor=maxf(metres,1.1)*ppm/size.y
	if not is_finite(factor) or factor<=0: return false
	var source=Rect2(Vector2(cell%2,floori(float(cell)/2.0))*size,size)
	canvas.draw_texture_rect_region(texture,Rect2(foot-Vector2(size.x*0.5,size.y*0.92)*factor,size*factor),source,Color.WHITE,false,true)
	return true
