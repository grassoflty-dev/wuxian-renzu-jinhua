extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const SCENE_PATH = 'res://data/scenes/cw_gear_shaft.json'
const SCENE_BYTES = 8965
const SCENE_SHA256 = '91afdb035c3df824806884c49d1e8f71034ff8f51b7bf90280bf71594896fea9'
const IMAGE_PATH = 'res://assets/environment/cw_gear_shaft_exterior_r2.png'
const IMAGE_BYTES = 1974147
const IMAGE_SHA256 = 'c5f5ba676bd694207d429e72ac0a4e24df39f0b5285de8139706caa65d3b1810'

var _scene_attempted: bool = false
var _canonical_scene: Dictionary = {}
var _image_attempted: bool = false
var _texture: CanvasTexture

func draw_backdrop(canvas: CanvasItem, snapshot: Dictionary, scene_data: Dictionary, current_key: String, scene_hash: String, camera, viewport: Rect2) -> bool:
	if not _valid_binding(canvas,snapshot,scene_data,current_key,scene_hash,camera,viewport):
		return false
	if not _load_scene_once() or _canonical_scene != scene_data:
		return false
	_load_image_once()
	if _texture == null or not is_instance_valid(_texture):
		return false
	if not _valid_binding(canvas,snapshot,scene_data,current_key,scene_hash,camera,viewport) or _canonical_scene != scene_data:
		return false
	var ratio: float = float(camera.ppm) / 48.0
	var anchor: Vector2 = camera.project(Vector3.ZERO)
	var position: Vector2 = anchor + Vector2(-284.0,50.0) * ratio
	var size: Vector2 = Vector2(1536.0,1024.0) * 0.5 * ratio
	if not is_finite(ratio) or ratio <= 0.0 or not _finite2(anchor) or not _finite2(position) or not _finite2(size) or size.x <= 0.0 or size.y <= 0.0 or not _finite2(position + size):
		return false
	canvas.draw_texture_rect(_texture,Rect2(position,size),false,Color(1.0,1.0,1.0,0.55))
	return true

func _valid_binding(canvas: CanvasItem, snapshot: Dictionary, scene_data: Dictionary, current_key: String, scene_hash: String, camera, viewport: Rect2) -> bool:
	if not is_instance_valid(canvas) or not canvas.is_inside_tree() or canvas.is_queued_for_deletion() or not _valid_viewport(viewport) or viewport != canvas.get_viewport_rect():
		return false
	if canvas.get('viewport_rect') != viewport or canvas.get('snapshot') != snapshot or canvas.get('scene') != scene_data or canvas.get('camera') != camera:
		return false
	if canvas.get('current_key') != current_key or canvas.get('scene_hash') != scene_hash:
		return false
	if not P.snapshot(snapshot) or snapshot.get('entryToken') != null or snapshot.get('worldId') != 'clockworks' or snapshot.get('sceneId') != 'cw_gear_shaft' or not P.counter(snapshot.get('worldEpoch'),1):
		return false
	if current_key != 'clockworks:cw_gear_shaft:' + str(snapshot.get('worldEpoch')) or scene_hash != SCENE_SHA256:
		return false
	if scene_data.get('worldId') != 'clockworks' or scene_data.get('sceneId') != 'cw_gear_shaft' or not P.counter(scene_data.get('schemaVersion'),1,1):
		return false
	var bounds = scene_data.get('boundsM')
	if not bounds is Dictionary or bounds.size() != 4 or not P.number(bounds.get('x'),0.0,0.0) or not P.number(bounds.get('z'),0.0,0.0) or not P.number(bounds.get('width'),24.0,24.0) or not P.number(bounds.get('depth'),16.0,16.0):
		return false
	return _valid_camera(camera,viewport)

func _valid_camera(camera, viewport: Rect2) -> bool:
	if not camera is RefCounted or not is_instance_valid(camera) or camera.get_script() != ProjectionScript:
		return false
	var origin = camera.get('origin')
	var center = camera.get('screen_center')
	var camera_size = camera.get('viewport')
	var initialized = camera.get('initialized')
	var ppm = camera.get('ppm')
	if not origin is Vector3 or not center is Vector2 or not camera_size is Vector2 or typeof(initialized) != TYPE_BOOL or not initialized:
		return false
	if typeof(ppm) != TYPE_INT and typeof(ppm) != TYPE_FLOAT:
		return false
	if not is_finite(float(ppm)) or float(ppm) <= 0.0 or not is_finite(origin.x) or not is_finite(origin.y) or not is_finite(origin.z) or not _finite2(center) or not _finite2(camera_size):
		return false
	return camera_size == viewport.size

func _valid_viewport(viewport: Rect2) -> bool:
	return _finite2(viewport.position) and _finite2(viewport.size) and _finite2(viewport.end) and viewport.size.x > 0.0 and viewport.size.y > 0.0

func _finite2(value: Vector2) -> bool:
	return is_finite(value.x) and is_finite(value.y)

func _load_scene_once() -> bool:
	if _scene_attempted:
		return not _canonical_scene.is_empty()
	_scene_attempted = true
	var scene_file = FileAccess.open(SCENE_PATH,FileAccess.READ)
	if scene_file == null:
		return false
	if scene_file.get_length() != SCENE_BYTES:
		scene_file.close()
		return false
	var bytes: PackedByteArray = scene_file.get_buffer(SCENE_BYTES)
	scene_file.close()
	if not _sha_matches(bytes,SCENE_SHA256):
		return false
	var text: String = bytes.get_string_from_utf8()
	if text.to_utf8_buffer() != bytes:
		return false
	var parsed = JSON.parse_string(text)
	if not parsed is Dictionary or parsed.is_empty():
		return false
	_canonical_scene = parsed
	return true

func _load_image_once() -> void:
	if _image_attempted:
		return
	_image_attempted = true
	var image_file = FileAccess.open(IMAGE_PATH,FileAccess.READ)
	if image_file == null:
		return
	if image_file.get_length() != IMAGE_BYTES:
		image_file.close()
		return
	var bytes: PackedByteArray = image_file.get_buffer(IMAGE_BYTES)
	image_file.close()
	if not _sha_matches(bytes,IMAGE_SHA256):
		return
	var image: Image = Image.new()
	if image.load_png_from_buffer(bytes) != OK or image.get_width() != 1536 or image.get_height() != 1024 or image.get_format() != Image.FORMAT_RGB8:
		return
	var image_texture: ImageTexture = ImageTexture.create_from_image(image)
	if image_texture == null or image_texture.get_width() != 1536 or image_texture.get_height() != 1024:
		return
	var canvas_texture: CanvasTexture = CanvasTexture.new()
	canvas_texture.diffuse_texture = image_texture
	canvas_texture.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
	canvas_texture.texture_repeat = CanvasItem.TEXTURE_REPEAT_DISABLED
	_texture = canvas_texture

func _sha_matches(bytes: PackedByteArray, expected: String) -> bool:
	var hash: HashingContext = HashingContext.new()
	return hash.start(HashingContext.HASH_SHA256) == OK and hash.update(bytes) == OK and hash.finish().hex_encode() == expected
