extends RefCounted

const DROWNED_PATH := 'res://assets/enemies/mist_harbor/drowned_temporal_local_r1.png'
const TIDEBOUND_PATH := 'res://assets/enemies/mist_harbor/tidebound_temporal_local_r1.png'
const DROWNED_SIZE := Vector2(1024,1536)
const TIDEBOUND_SIZE := Vector2(1024,1536)
var _textures: Dictionary = {}
var _attempted: Dictionary = {}
var _scope := ''
var _previous: Dictionary = {}

func _counter(v: Variant) -> bool:
	if not (v is int or v is float): return false
	var n=float(v)
	return is_finite(n) and n>=0 and n<=9007199254740991.0 and floor(n)==n

func _texture(family: String) -> Texture2D:
	if _attempted.has(family): return _textures.get(family)
	_attempted[family]=true
	var path: String = DROWNED_PATH if family=='drowned' else TIDEBOUND_PATH
	var size: Vector2 = DROWNED_SIZE if family=='drowned' else TIDEBOUND_SIZE
	if size.x<=0 or size.y<=0: return null
	var texture: Texture2D = null
	if ResourceLoader.exists(path): texture=load(path) as Texture2D
	if texture==null:
		var image=Image.new()
		if image.load(path)==OK and image.get_format()==Image.FORMAT_RGBA8: texture=ImageTexture.create_from_image(image)
	if texture!=null and texture.get_size()==size:
		var image=texture.get_image()
		if image!=null and image.get_format()==Image.FORMAT_RGBA8: _textures[family]=texture
	return _textures.get(family)

func draw_body(canvas: CanvasItem, family: String, scope: String, tick: int, server_ms: int, actor: Dictionary, foot: Vector2, ppm: float, projected_events: Array, animate: bool) -> bool:
	if family not in ['drowned','tidebound'] or not is_instance_valid(canvas) or scope.is_empty() or tick<0 or server_ms<0 or not foot.is_finite() or not is_finite(ppm) or ppm<=0: return false
	if scope!=_scope: _scope=scope;_previous.clear()
	var transform=actor.get('transform')
	if not transform is Dictionary or not transform.get('positionM') is Dictionary: _previous.clear();return false
	var position=transform.positionM
	for key in ['xM','yM','zM']:
		if not (position.get(key) is int or position.get(key) is float) or not is_finite(float(position[key])): _previous.clear();return false
	var p=Vector3(float(position.xM),float(position.yM),float(position.zM))
	var state='';var attack=null;var serial=0;var remaining=0
	if family=='drowned':
		var animation=actor.get('animationPresentation')
		if animation is Dictionary and animation.get('state') in ['idle','chasing','windup','cooldown'] and _counter(animation.get('remainingMs')) and _counter(animation.get('attackSerial')):
			state=animation.state;attack=animation.get('currentAttack');serial=int(animation.attackSerial);remaining=int(animation.remainingMs)
			if attack!=null and attack!='heavy_melee' or state=='windup' and (remaining==0 or attack!='heavy_melee') or state=='cooldown' and (remaining==0 or attack!=null) or state in ['idle','chasing'] and (remaining!=0 or attack!=null): state=''
	var moved=false;var release=false
	if not _previous.is_empty():
		if tick<_previous.tick or server_ms<_previous.time: _previous.clear();return false
		if tick==_previous.tick and server_ms==_previous.time:
			if p!=_previous.position or state!=_previous.state or serial!=_previous.serial or remaining!=_previous.remaining or attack!=_previous.attack: _previous.clear();return false
			moved=_previous.moved;release=_previous.release
		elif tick>_previous.tick and server_ms>_previous.time:
			if family=='drowned' and state!='' and _previous.state!='' and serial<_previous.serial: _previous.clear();return false
			moved=p.distance_squared_to(_previous.position)>0.00000001 and (family=='tidebound' or state=='chasing' and _previous.state=='chasing')
			release=family=='drowned' and state=='cooldown' and _previous.state=='windup' and _previous.attack=='heavy_melee' and serial==_previous.serial+1
		else: _previous.clear();return false
	_previous={'tick':tick,'time':server_ms,'position':p,'state':state,'serial':serial,'remaining':remaining,'attack':attack,'moved':moved,'release':release}
	var cell=0
	if animate:
		if moved: cell=int(server_ms/166)%2
		if family=='drowned':
			if state=='windup': cell=2
			elif release: cell=3
		else:
			for event in projected_events:
				if event.kind=='EnemyAttackTelegraph': cell=2
				elif event.kind in ['EnemyChargeMotion','EnemyAttackImpact']: cell=3
	var texture=_texture(family)
	if texture==null: return false
	var size=texture.get_size()/2.0
	var factor=(1.72 if family=='drowned' else 1.85)*ppm/size.y
	if not is_finite(factor) or factor<=0: return false
	var source=Rect2(Vector2(cell%2,floori(float(cell)/2.0))*size,size)
	canvas.draw_texture_rect_region(texture,Rect2(foot-Vector2(size.x*0.5,size.y*0.96)*factor,size*factor),source,Color.WHITE,false,true)
	return true
