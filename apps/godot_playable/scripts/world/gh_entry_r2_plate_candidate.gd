extends RefCounted

# PROVISIONAL_NO_OCCLUDER_SORTING: placement is not a ground/contact fit.
# Disabled by default; this candidate does not alter scene authority or walkability.
const P = preload('res://scripts/bridge/protocol.gd')
const SETTING = 'wuxian/visual_candidates/gh_entry_r2'
const IMAGE_PATH = 'res://assets/props/grey_hive/gh_entry_whole_map_20261101_r2.png'
const IMAGE_BYTES = 2943000
const IMAGE_SHA256 = '550b5ebb8a70decd94f454ab0416ac8ce7722338168243181be824f6571fd045'
const SCENE_SHA256 = '5deb6c01b2ad81144144ae2526c06f8c337839e8d880aeb1b3af3a225afe787b'
const ORIGIN_PIXEL = Vector2(780,140)
const REFERENCE_PPM = 40.0

var attempted = false
var texture: ImageTexture

func draw_plate(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, key: String, scene_hash: String, camera) -> bool:
	var enabled = ProjectSettings.get_setting(SETTING,false)
	if typeof(enabled)!=TYPE_BOOL or not enabled:
		return false
	if not is_instance_valid(canvas) or not P.snapshot(snapshot) or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_entry_maintenance':
		return false
	if scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_entry_maintenance' or scene_hash!=SCENE_SHA256 or key!='grey_hive:gh_entry_maintenance:'+str(snapshot.worldEpoch):
		return false
	if camera==null or not is_finite(camera.ppm) or camera.ppm<=0.0:
		return false
	_load_once()
	if not is_instance_valid(texture):
		return false
	var scale: float = camera.ppm / REFERENCE_PPM
	var position: Vector2 = camera.project(Vector3.ZERO) - ORIGIN_PIXEL * scale
	if not is_finite(position.x) or not is_finite(position.y) or not is_finite(scale) or scale<=0.0:
		return false
	var old_filter = canvas.texture_filter
	var old_repeat = canvas.texture_repeat
	canvas.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
	canvas.texture_repeat = CanvasItem.TEXTURE_REPEAT_DISABLED
	canvas.draw_texture_rect(texture,Rect2(position,Vector2(1536,1024)*scale),false)
	canvas.texture_filter = old_filter
	canvas.texture_repeat = old_repeat
	return true

func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open(IMAGE_PATH,FileAccess.READ)
	if file==null:
		return
	if file.get_length()!=IMAGE_BYTES:
		file.close()
		return
	var bytes: PackedByteArray = file.get_buffer(IMAGE_BYTES)
	file.close()
	var hash = HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256)!=OK or hash.update(bytes)!=OK or hash.finish().hex_encode()!=IMAGE_SHA256:
		return
	var image = Image.new()
	if image.load_png_from_buffer(bytes)!=OK or image.get_width()!=1536 or image.get_height()!=1024 or image.get_format()!=Image.FORMAT_RGB8:
		return
	texture = ImageTexture.create_from_image(image)