extends RefCounted
# The caller owns scene/spawn/visibility checks and passes authoritative time.
const PATH := 'res://assets/enemies/clockworks/prime_regulator_temporal_local_r1.png'
const EXPECTED_SIZE := Vector2(1254,1254)
const TYPE := 'runtime2d.enemy.clockworks.forged_guard.v2'
var _texture: Texture2D
var _attempted := false
var _scope := ''
var _previous: Dictionary = {}

func reset() -> void:
	_scope=''
	_previous.clear()

func _counter(v: Variant) -> bool:
	if not (v is int or v is float): return false
	var n=float(v)
	return is_finite(n) and n>=0.0 and n<=9007199254740991.0 and floor(n)==n

func _load_once() -> Texture2D:
	if _attempted: return _texture
	_attempted=true
	var texture: Texture2D = null
	if ResourceLoader.exists(PATH): texture=load(PATH) as Texture2D
	if texture==null:
		var image=Image.new()
		if image.load(PATH)==OK and image.get_format()==Image.FORMAT_RGBA8:
			texture=ImageTexture.create_from_image(image)
	if texture!=null and EXPECTED_SIZE.x>0 and texture.get_size()==EXPECTED_SIZE:
		var image=texture.get_image()
		if image!=null and image.get_format()==Image.FORMAT_RGBA8: _texture=texture
	return _texture

func draw_body(canvas:CanvasItem,scene_key:String,epoch:int,tick:int,server_ms:int,actor:Dictionary,foot:Vector2,ppm:float,expected_id:String='cw_prime_regulator',animate_gait:bool=true) -> bool:
	if not is_instance_valid(canvas) or scene_key.is_empty() or epoch<=0 or tick<0 or server_ms<0 or not foot.is_finite() or not is_finite(ppm) or ppm<=0: return false
	var scope=scene_key+':'+str(epoch)+':'+expected_id
	if scope!=_scope: reset();_scope=scope
	if expected_id.is_empty() or actor.get('entityId')!=expected_id or actor.get('entityType')!=TYPE or actor.get('active')!=true: reset();return false
	var transform=actor.get('transform');var animation=actor.get('animationPresentation')
	if not transform is Dictionary or not transform.get('positionM') is Dictionary or not animation is Dictionary: reset();return false
	var p=transform.positionM
	for key in ['xM','yM','zM']:
		if not (p.get(key) is int or p.get(key) is float) or not is_finite(float(p[key])): reset();return false
	var position=Vector3(float(p.xM),float(p.yM),float(p.zM))
	var state=animation.get('state');var remaining=animation.get('remainingMs');var serial=animation.get('attackSerial');var attack=animation.get('currentAttack')
	if state not in ['idle','chasing','windup','cooldown'] or not _counter(remaining) or not _counter(serial): reset();return false
	if attack!=null and attack not in ['heavy_melee','pressure_wave']: reset();return false
	if expected_id!='cw_prime_regulator' and attack=='pressure_wave': reset();return false
	if state=='windup' and (int(remaining)==0 or attack==null) or state=='cooldown' and (int(remaining)==0 or attack!=null) or state in ['idle','chasing'] and (int(remaining)!=0 or attack!=null): reset();return false
	var moved=false;var release=false
	if not _previous.is_empty():
		if tick<_previous.tick or server_ms<_previous.time: reset();return false
		if tick==_previous.tick and server_ms==_previous.time:
			if _previous.pos!=position or _previous.state!=state or _previous.remaining!=int(remaining) or _previous.serial!=int(serial) or _previous.attack!=attack: reset();return false
			moved=_previous.moved;release=_previous.release
		elif tick>_previous.tick and server_ms>_previous.time:
			if int(serial)<_previous.serial: reset();return false
			if state=='windup' and _previous.state=='windup' and attack!=_previous.attack: reset();return false
			moved=state=='chasing' and _previous.state=='chasing' and position.distance_squared_to(_previous.pos)>0.00000001
			release=state=='cooldown' and _previous.state=='windup' and _previous.attack in ['heavy_melee','pressure_wave'] and int(serial)==_previous.serial+1
		else: reset();return false
	_previous={'tick':tick,'time':server_ms,'pos':position,'state':state,'remaining':int(remaining),'serial':int(serial),'attack':attack,'moved':moved,'release':release}
	var cell=-1
	if moved and animate_gait: cell=int(server_ms/166)%2
	elif state=='windup': cell=2
	elif release: cell=3
	if cell<0: return false
	var texture=_load_once()
	if texture==null: return false
	var source_size=texture.get_size()/2.0
	var scale=1.72*ppm/source_size.y
	if not is_finite(scale) or scale<=0: return false
	var source=Rect2(Vector2(cell%2,floori(float(cell)/2.0))*source_size,source_size)
	var pivot=Vector2(source_size.x*0.5,source_size.y*0.97)
	canvas.draw_texture_rect_region(texture,Rect2(foot-pivot*scale,source_size*scale),source,Color.WHITE,false,true)
	return true
