extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const V = preload('res://scripts/world/presentation.gd')
const PATH = 'res://assets/enemies/grey_hive/swarm_member_temporal_local_r1.png'
var attempted := false
var atlas: Texture2D
var samples: Dictionary = {}
var seen: Dictionary = {}
var scope := ''

func reset() -> void:
	scope=''
	samples.clear()
	seen.clear()

func begin_frame() -> void:
	seen.clear()

func end_frame() -> void:
	for id in samples.keys():
		if not seen.has(id): samples.erase(id)

func _load_once() -> bool:
	if not attempted:
		attempted=true
		var image: Image
		if ResourceLoader.exists(PATH):
			var texture=load(PATH)
			if texture is Texture2D:
				image=texture.get_image()
				atlas=texture
		if image==null:
			image=Image.new()
			if image.load(PATH)==OK: atlas=ImageTexture.create_from_image(image)
		if image==null or image.is_empty() or image.get_format()!=Image.FORMAT_RGBA8 or image.detect_alpha()==Image.ALPHA_NONE:
			atlas=null
		elif image.get_width()!=1254 or image.get_height()!=1254 or atlas==null or atlas.get_size()!=Vector2(1254,1254):
			atlas=null
	return atlas!=null

# Caller supplies only its current EventView.project rows and independently
# validates parent and member visibility. No attack timing is reconstructed here.
func draw_body(canvas:CanvasItem,s:Dictionary,actor:Dictionary,member:Dictionary,foot:Vector2,ppm:float,events:Array,animate:bool) -> bool:
	if not is_instance_valid(canvas) or not P.snapshot(s) or not P.counter(s.get('worldEpoch'),1) or not P.counter(s.get('serverTick')) or not P.counter(s.get('serverTimeMs')) or not foot.is_finite() or not is_finite(ppm) or ppm<=0.0:
		reset()
		return false
	var key=str(s.worldId)+':'+str(s.sceneId)+':'+str(s.worldEpoch)
	if key!=scope:
		reset()
		scope=key
	var id=actor.get('entityId')
	var mid=member.get('memberId')
	if actor.get('entityType')!='enemy.grey_hive.swarm' or actor.get('actorKind')!='enemy' or actor.get('active')!=true or not P.ident(id) or typeof(mid)!=TYPE_STRING or not mid.begins_with(id+'/member/') or member.get('active')!=true or not P.point(member.get('positionM')) or not V.swarm_members(actor).has(member):
		if mid is String: samples.erase(mid)
		return false
	if not _load_once(): return false
	seen[mid]=true
	var pos=P.vector(member.positionM)
	var frame=Engine.get_process_frames()
	var now=Time.get_ticks_msec()
	var previous=samples.get(mid,{})
	var moved:=false
	var start: int = now
	if animate and previous is Dictionary and previous.get('actor')==id and previous.get('frame',-2)>=frame-1 and previous.get('tick',-1)<=s.serverTick and previous.get('time',-1)<=s.serverTimeMs:
		if previous.tick==s.serverTick:
			if previous.get('position')!=pos: samples.erase(mid); return false
			moved=previous.get('moving')==true
			start=int(previous.get('start',now))
		else:
			moved=previous.get('position') is Vector3 and previous.position!=pos
			if moved and previous.get('moving')==true: start=int(previous.get('start',now))
	if animate: samples[mid]={'actor':id,'tick':s.serverTick,'time':s.serverTimeMs,'position':pos,'moving':moved,'start':start,'frame':frame}
	else: samples.erase(mid)
	var lunge:=false
	for event in (events if animate else []):
		if event is Dictionary and event.get('kind')=='EnemyLungeMotion' and event.get('attackKind')=='lunge' and event.get('actorId')==id and event.get('worldEpoch')==s.worldEpoch and P.counter(event.get('serverTick')) and event.serverTick<=s.serverTick:
			lunge=true
			break
	var cell=3 if lunge else (1+int(floor(float(now-start)*6.0/1000.0))%2 if moved else 0)
	var size=atlas.get_size()
	var cell_size=size/2.0
	var source=Rect2(Vector2((cell%2)*cell_size.x,int(cell/2)*cell_size.y),cell_size)
	var height=maxf(0.6,0.9)*ppm
	var display=Vector2(height*cell_size.x/cell_size.y,height)
	if not display.is_finite() or display.x<=0.0 or display.y<=0.0: return false
	canvas.draw_texture_rect_region(atlas,Rect2(foot-Vector2(display.x*0.5,display.y*0.84),display),source,Color.WHITE)
	return true
