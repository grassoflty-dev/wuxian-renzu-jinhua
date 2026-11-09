extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const V = preload('res://scripts/world/presentation.gd')
const PATH = 'res://assets/enemies/grey_hive/sentinel_temporal_local_r1.png'
# Actual returned original; activation still requires the independent purpose fit.
const WIDTH = 1024
const HEIGHT = 1536
const PIVOT = Vector2(195.0/386.0,0.91)
var attempted := false
var atlas: Texture2D
var scope := ''
var sample: Dictionary = {}

func reset() -> void:
	scope=''
	sample.clear()

func suspend() -> void:
	if not sample.is_empty(): sample['can_advance']=false

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
		if image==null or image.is_empty() or image.get_format()!=Image.FORMAT_RGBA8 or image.detect_alpha()==Image.ALPHA_NONE or image.get_width()!=WIDTH or image.get_height()!=HEIGHT or WIDTH<2 or HEIGHT<2 or WIDTH%2!=0 or HEIGHT%2!=0 or atlas==null or atlas.get_size()!=Vector2(WIDTH,HEIGHT):
			atlas=null
	return atlas!=null

func ready() -> bool:
	return _load_once()

func canvas_rect(foot:Vector2,ppm:float) -> Rect2:
	if WIDTH<2 or HEIGHT<2 or not foot.is_finite() or not is_finite(ppm) or ppm<=0.0: return Rect2()
	var cell=Vector2(WIDTH/2.0,HEIGHT/2.0)
	var factor=2.6*ppm/cell.y
	var size=cell*factor
	return Rect2(foot-Vector2(PIVOT.x*size.x,PIVOT.y*size.y),size)

# The caller has already validated scene, actor, full-body visibility and foot.
func draw_body(canvas:CanvasItem,s:Dictionary,actor:Dictionary,scene_hash:String,foot:Vector2,ppm:float,animate:bool) -> bool:
	if not is_instance_valid(canvas) or not P.snapshot(s) or s.get('worldId')!='grey_hive' or s.get('sceneId')!='gh_sentinel_arena' or not P.counter(s.get('worldEpoch'),1) or not P.counter(s.get('serverTick')) or not P.counter(s.get('serverTimeMs')) or scene_hash.is_empty() or actor.get('entityId')!='gh_sentinel_arena_sentinel_01' or actor.get('entityType')!='enemy.grey_hive.sentinel' or actor.get('actorKind')!='sentinel' or actor.get('active')!=true or not P.transform(actor.get('transform')):
		reset()
		return false
	var encounter=V.sentinel(s)
	if encounter.is_empty() or encounter.phase in ['dead','inactive'] or not _load_once():
		reset()
		return false
	var bounds=canvas_rect(foot,ppm)
	if not bounds.has_area(): reset(); return false
	var next_scope=scene_hash+':'+str(s.worldEpoch)+':'+actor.entityId
	if scope!=next_scope:
		reset()
		scope=next_scope
	var position=P.vector(actor.transform.positionM)
	var render_frame=Engine.get_process_frames()
	var chosen:=0
	var elapsed:=0
	if not sample.is_empty():
		if sample.get('frame',-2)<render_frame-1 or s.serverTick<sample.tick or s.serverTimeMs<sample.time:
			reset()
			scope=next_scope
		elif s.serverTick==sample.tick:
			if s.serverTimeMs!=sample.time or position!=sample.position or encounter.phase!=sample.phase or encounter.attackSerial!=sample.serial or encounter.remainingMs!=sample.remaining:
				reset()
				return false
			chosen=sample.cell
			elapsed=sample.elapsed
		elif animate and sample.get('can_advance')==true and encounter.phase=='chase' and sample.phase=='chase' and encounter.attackSerial==sample.serial and position!=sample.position:
			elapsed=int(sample.elapsed)+int(s.serverTimeMs)-int(sample.time)
			chosen=1+int(floor(float(elapsed)*6.0/1000.0))%2
	if encounter.phase in ['light_windup','heavy_windup','charge_windup','charge']: chosen=3
	elif encounter.phase!='chase': chosen=0
	if not animate and encounter.phase=='chase' and (sample.is_empty() or sample.get('tick')!=s.serverTick): chosen=0; elapsed=0
	sample={'tick':s.serverTick,'time':s.serverTimeMs,'position':position,'phase':encounter.phase,'serial':encounter.attackSerial,'remaining':encounter.remainingMs,'cell':chosen,'elapsed':elapsed,'frame':render_frame,'can_advance':animate}
	var cell=Vector2(WIDTH/2.0,HEIGHT/2.0)
	var source=Rect2(Vector2((chosen%2)*cell.x,int(chosen/2)*cell.y),cell)
	canvas.draw_texture_rect_region(atlas,bounds,source,Color.WHITE)
	return true
