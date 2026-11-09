extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const SCENE_PATH = 'res://data/scenes/mh_signal_yard.json'
const SCENE_BYTES = 8327
const SCENE_SHA256 = '14f708142f6d4c15b8abdb3671e5357f845d96228c5b9aed719e19895e3a66d8'
const MAPPING_PATH = 'res://data/mh_signal_yard_exterior_reuse_20261028_r2.internal-use.json'
const IMAGE_PATH = 'res://assets/props/mist_harbor/mh_fog_pier_exterior_backplate_20261027_r2.png'
const IMAGE_BYTES = 2069499
const IMAGE_SHA256 = 'ae21e5372b8f236865d13de5f0b2eee81adc32d9ee3328f434e59959ecf5cb2c'

var _scene_attempted: bool = false
var _canonical_scene: Dictionary = {}
var _attempted: bool = false
var _texture: CanvasTexture

func draw_backplate(canvas: CanvasItem, snapshot: Dictionary, scene_data: Dictionary, current_key: String, scene_hash: String, viewport: Rect2) -> bool:
	if not _valid_binding(canvas,snapshot,scene_data,current_key,scene_hash,viewport):
		return false
	if not _load_scene_once() or _canonical_scene != scene_data:
		return false
	_load_once()
	if _texture == null or not is_instance_valid(_texture):
		return false
	if not _valid_binding(canvas,snapshot,scene_data,current_key,scene_hash,viewport) or _canonical_scene != scene_data:
		return false
	var scale: float = maxf(viewport.size.x / 1536.0,viewport.size.y / 1024.0)
	var size: Vector2 = Vector2(1536.0,1024.0) * scale
	var position: Vector2 = viewport.position + (viewport.size - size) * 0.5
	if not is_finite(scale) or scale <= 0.0 or not _finite2(size) or not _finite2(position) or not _finite2(position + size):
		return false
	canvas.draw_texture_rect(_texture,Rect2(position,size),false,Color.WHITE)
	return true

func _valid_binding(canvas: CanvasItem, snapshot: Dictionary, scene_data: Dictionary, current_key: String, scene_hash: String, viewport: Rect2) -> bool:
	if not is_instance_valid(canvas) or not canvas.is_inside_tree() or canvas.is_queued_for_deletion() or not _valid_viewport(viewport) or viewport != canvas.get_viewport_rect():
		return false
	if canvas.get('viewport_rect') != viewport or canvas.get('snapshot') != snapshot or canvas.get('scene') != scene_data:
		return false
	if canvas.get('current_key') != current_key or canvas.get('scene_hash') != scene_hash:
		return false
	if not P.snapshot(snapshot) or snapshot.get('entryToken') != null or snapshot.get('worldId') != 'mist_harbor' or snapshot.get('sceneId') != 'mh_signal_yard' or not P.counter(snapshot.get('worldEpoch'),1):
		return false
	if current_key != 'mist_harbor:mh_signal_yard:' + str(snapshot.get('worldEpoch')) or scene_hash != SCENE_SHA256:
		return false
	if scene_data.get('worldId') != 'mist_harbor' or scene_data.get('sceneId') != 'mh_signal_yard' or not P.counter(scene_data.get('schemaVersion'),1,1):
		return false
	var bounds = scene_data.get('boundsM')
	if not bounds is Dictionary or bounds.size() != 4:
		return false
	return P.number(bounds.get('x'),0.0,0.0) and P.number(bounds.get('z'),0.0,0.0) and P.number(bounds.get('width'),24.0,24.0) and P.number(bounds.get('depth'),16.0,16.0)

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

func _load_once() -> void:
	if _attempted:
		return
	_attempted = true
	var mapping_file = FileAccess.open(MAPPING_PATH,FileAccess.READ)
	if mapping_file == null:
		return
	var length: int = mapping_file.get_length()
	if length <= 0 or length > 3000:
		mapping_file.close()
		return
	var mapping_bytes: PackedByteArray = mapping_file.get_buffer(length)
	mapping_file.close()
	var text: String = mapping_bytes.get_string_from_utf8()
	if text.to_utf8_buffer() != mapping_bytes or not _valid_mapping(JSON.parse_string(text)):
		return
	var image_file = FileAccess.open(IMAGE_PATH,FileAccess.READ)
	if image_file == null:
		return
	if image_file.get_length() != IMAGE_BYTES:
		image_file.close()
		return
	var png_bytes: PackedByteArray = image_file.get_buffer(IMAGE_BYTES)
	image_file.close()
	if not _sha_matches(png_bytes,IMAGE_SHA256):
		return
	var image: Image = Image.new()
	if image.load_png_from_buffer(png_bytes) != OK or image.get_width() != 1536 or image.get_height() != 1024 or image.get_format() != Image.FORMAT_RGB8:
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

func _valid_mapping(value) -> bool:
	if not value is Dictionary or value.size() != 11:
		return false
	if value.get('schema') != 'wuxian.mh-signal-yard-exterior-reuse.internal-use/v1' or value.get('resourceId') != 'ART-MH-FOG-PIER-EXTERIOR-BACKPLATE-20261027-001' or value.get('revision') != 'r2':
		return false
	if value.get('approval') != 'ACCEPT_LIMITED_STATIC_MH_SIGNAL_YARD_BACKGROUND_REUSE' or value.get('GodotDisplay') != 'NOT_RUN' or value.get('publicRelease') != 'NOT_ESTABLISHED' or value.get('fallback') != 'EXISTING_CLEAR_AND_FLOOR' or value.get('limitations') != 'UNREGISTERED_NONINTERACTIVE_RUNTIME_NOT_RUN':
		return false
	var asset = value.get('asset')
	var composition = value.get('composition')
	var installation = value.get('installation')
	if not asset is Dictionary or asset.size() != 6 or not composition is Dictionary or composition.size() != 6 or not installation is Dictionary or installation.size() != 9:
		return false
	if asset.get('path') != IMAGE_PATH or asset.get('bytes') != IMAGE_BYTES or asset.get('sha256') != IMAGE_SHA256 or asset.get('width') != 1536 or asset.get('height') != 1024 or asset.get('format') != 'RGB8':
		return false
	if composition.get('space') != 'SCREEN_SPACE_BACKGROUND_ONLY' or composition.get('fit') != 'CENTERED_COVER_FULL_VIEWPORT' or composition.get('uv') != 'FULL_ONCE' or composition.get('filter') != 'LINEAR' or composition.get('repeat') != 'DISABLED' or composition.get('tint') != 'WHITE_OPAQUE':
		return false
	if installation.get('worldId') != 'mist_harbor' or installation.get('sceneId') != 'mh_signal_yard' or not P.counter(installation.get('sceneSchemaVersion'),1,1) or installation.get('sceneSha256') != SCENE_SHA256:
		return false
	if installation.get('canonicalScenePath') != SCENE_PATH or installation.get('canonicalSceneBytes') != SCENE_BYTES or installation.get('nonInteractive') != true or installation.get('unregistered') != true:
		return false
	var bounds = installation.get('boundsM')
	if not bounds is Array or bounds.size() != 4:
		return false
	return P.number(bounds[0],0.0,0.0) and P.number(bounds[1],0.0,0.0) and P.number(bounds[2],24.0,24.0) and P.number(bounds[3],16.0,16.0)
