extends RefCounted
const PATH='res://assets/characters/baizhi/baizhi_static_local_r1.png'
var attempted:=false
var texture: Texture2D

func _load_once() -> bool:
	if attempted: return is_instance_valid(texture)
	attempted=true
	var candidate: Texture2D
	if ResourceLoader.exists(PATH,'Texture2D'): candidate=ResourceLoader.load(PATH,'Texture2D') as Texture2D
	if not is_instance_valid(candidate):
		var image=Image.new()
		if image.load(PATH)==OK and image.get_width()==1024 and image.get_height()==1536 and image.get_format()==Image.FORMAT_RGBA8:
			candidate=ImageTexture.create_from_image(image)
	if is_instance_valid(candidate):
		var image: Image = candidate.get_image()
		if is_instance_valid(image) and image.get_width()==1024 and image.get_height()==1536 and image.get_format()==Image.FORMAT_RGBA8: texture=candidate
	return is_instance_valid(texture)

func draw_body(canvas: CanvasItem, npc: Dictionary, camera) -> bool:
	if not is_instance_valid(canvas) or camera==null or not camera.initialized or not is_finite(camera.ppm) or camera.ppm<=0 or npc.get('entityId')!='gh_bz_whitezhi_v1' or not npc.get('position') is Array or npc.position.size()!=3 or not _load_once(): return false
	var foot: Vector2 = camera.project(Vector3(npc.position[0],npc.position[1],npc.position[2]))
	var factor: float = 1.7*camera.ppm/1454.0
	if not foot.is_finite() or not is_finite(factor) or factor<=0: return false
	canvas.draw_texture_rect(texture,Rect2(foot-Vector2(505,1475)*factor,Vector2(1024,1536)*factor),false,Color.WHITE)
	return true
