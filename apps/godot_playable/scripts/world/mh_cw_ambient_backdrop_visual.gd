extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ASSETS = {
	'mist_harbor:mh_tidal_warehouse':'res://assets/props/mist_harbor/mh_tidal_warehouse_ambient_local_r1.png',
	'clockworks:cw_entry_foundry':'res://assets/props/clockworks/cw_entry_foundry_ambient_local_r1.png',
}
var attempted: Dictionary = {}
var textures: Dictionary = {}

func _valid_image(image: Image) -> bool:
	return is_instance_valid(image) and not image.is_empty() and image.get_width()==1536 and image.get_height()==1024 and image.get_format()==Image.FORMAT_RGB8 and image.detect_alpha()==Image.ALPHA_NONE

func _texture_for(owner: String) -> Texture2D:
	if attempted.has(owner): return textures.get(owner)
	attempted[owner]=true
	var path: String=ASSETS[owner]
	var texture: Texture2D
	if ResourceLoader.exists(path,'Texture2D'):
		var imported=ResourceLoader.load(path,'Texture2D') as Texture2D
		if is_instance_valid(imported) and _valid_image(imported.get_image()): texture=imported
	if not is_instance_valid(texture):
		var raw=Image.new()
		if raw.load(path)==OK and _valid_image(raw): texture=ImageTexture.create_from_image(raw)
	if is_instance_valid(texture): textures[owner]=texture
	return texture

func _point(u: float, v: float) -> Vector3:
	return Vector3((u-64.0)/2.0,-32.0-v,(-64.0-u)/2.0)

func prepare(snapshot: Dictionary, scene: Dictionary, key: String, camera, current_plan: Dictionary) -> Dictionary:
	if not P.snapshot(snapshot) or not is_instance_valid(camera) or not camera.initialized or not is_finite(camera.ppm) or camera.ppm<=0.0: return {}
	var owner=str(snapshot.worldId)+':'+str(snapshot.sceneId)
	if not ASSETS.has(owner) or scene.get('worldId')!=snapshot.worldId or scene.get('sceneId')!=snapshot.sceneId or key!=owner+':'+str(snapshot.worldEpoch): return {}
	if current_plan.is_empty() or not current_plan.get('floor') is Array or current_plan.floor.is_empty(): return {}
	var texture=_texture_for(owner)
	if not is_instance_valid(texture): return {}
	var bounds=Vector4(-68.0,76.0,-35.5,60.5) if owner.begins_with('mist_harbor:') else Vector4(-28.0,36.0,-46.0/3.0,82.0/3.0)
	var world=[_point(bounds.x,bounds.z),_point(bounds.y,bounds.z),_point(bounds.y,bounds.w),_point(bounds.x,bounds.w)]
	var points:=PackedVector2Array()
	for p in world:
		var projected: Vector2=camera.project(p)
		if not projected.is_finite(): return {}
		points.append(projected)
	var area=0.0
	for i in range(4): area+=points[i].cross(points[(i+1)%4])
	if not is_finite(area) or absf(area)<0.000001: return {}
	return {'points':points,'texture':texture}

func draw(canvas: CanvasItem, packet: Dictionary) -> void:
	if not is_instance_valid(canvas) or not packet.get('points') is PackedVector2Array or not is_instance_valid(packet.get('texture')): return
	var previous=canvas.texture_repeat
	canvas.texture_repeat=CanvasItem.TEXTURE_REPEAT_DISABLED
	canvas.draw_polygon(packet.points,PackedColorArray([Color.WHITE]),PackedVector2Array([Vector2(0,0),Vector2(1,0),Vector2(1,1),Vector2(0,1)]),packet.texture)
	canvas.texture_repeat=previous
