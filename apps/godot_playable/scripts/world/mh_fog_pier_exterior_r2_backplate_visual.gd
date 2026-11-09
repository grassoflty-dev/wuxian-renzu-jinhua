extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const SCENE_RESOURCE = "res://data/scenes/mh_fog_pier.json"
const SCENE_BYTES = 7671
const SCENE_SHA256 = "1f12ceae6565dcfb8f3b9f497840f8027748e6109208e788ea6d7579fff00946"
const MAPPING_RESOURCE = "res://data/mh_fog_pier_exterior_backplate_20261027_r2.internal-use.json"
const TEXTURE_RESOURCE = "res://assets/props/mist_harbor/mh_fog_pier_exterior_backplate_20261027_r2.png"
const PNG_BYTES = 2069499
const PNG_SHA256 = "ae21e5372b8f236865d13de5f0b2eee81adc32d9ee3328f434e59959ecf5cb2c"

var _scene_attempted: bool = false
var _canonical_scene: Dictionary = {}
var _attempted: bool = false
var _texture: CanvasTexture

func draw_backplate(canvas: CanvasItem, snapshot: Dictionary, scene_data: Dictionary, current_key: String, scene_hash: String, viewport: Rect2) -> bool:
	if not _valid_binding(canvas, snapshot, scene_data, current_key, scene_hash, viewport):
		return false
	if not _load_canonical_scene_once() or _canonical_scene != scene_data:
		return false
	_load_once()
	if _texture == null or not is_instance_valid(_texture):
		return false
	if not _valid_binding(canvas, snapshot, scene_data, current_key, scene_hash, viewport) or _canonical_scene != scene_data:
		return false
	var scale: float = maxf(viewport.size.x / 1536.0, viewport.size.y / 1024.0)
	var size: Vector2 = Vector2(1536.0, 1024.0) * scale
	var position: Vector2 = viewport.position + (viewport.size - size) * 0.5
	if not is_finite(scale) or scale <= 0.0 or not is_finite(size.x) or not is_finite(size.y) or not is_finite(position.x) or not is_finite(position.y):
		return false
	canvas.draw_texture_rect(_texture, Rect2(position, size), false, Color.WHITE)
	return true

func _valid_binding(canvas: CanvasItem, snapshot: Dictionary, scene_data: Dictionary, current_key: String, scene_hash: String, viewport: Rect2) -> bool:
	if not is_instance_valid(canvas) or not canvas.is_inside_tree() or not _valid_viewport(viewport) or viewport != canvas.get_viewport_rect():
		return false
	var live_viewport = canvas.get("viewport_rect")
	var live_snapshot = canvas.get("snapshot")
	var live_scene = canvas.get("scene")
	if not live_viewport is Rect2 or live_viewport != viewport or not live_snapshot is Dictionary or live_snapshot != snapshot or not live_scene is Dictionary or live_scene != scene_data:
		return false
	if canvas.get("current_key") != current_key or canvas.get("scene_hash") != scene_hash:
		return false
	if not P.snapshot(snapshot) or snapshot.get("worldId") != "mist_harbor" or snapshot.get("sceneId") != "mh_fog_pier" or not P.counter(snapshot.get("worldEpoch"), 1):
		return false
	if current_key != "mist_harbor:mh_fog_pier:" + str(snapshot.get("worldEpoch")) or scene_hash != SCENE_SHA256:
		return false
	if scene_data.get("worldId") != "mist_harbor" or scene_data.get("sceneId") != "mh_fog_pier" or not P.counter(scene_data.get("schemaVersion"), 1, 1):
		return false
	var bounds = scene_data.get("boundsM")
	if not bounds is Dictionary or bounds.size() != 4:
		return false
	return P.number(bounds.get("x"), 0.0, 0.0) and P.number(bounds.get("z"), 0.0, 0.0) and P.number(bounds.get("width"), 24.0, 24.0) and P.number(bounds.get("depth"), 16.0, 16.0)

func _valid_viewport(viewport: Rect2) -> bool:
	return is_finite(viewport.position.x) and is_finite(viewport.position.y) and is_finite(viewport.size.x) and is_finite(viewport.size.y) and viewport.size.x > 0.0 and viewport.size.y > 0.0

func _load_canonical_scene_once() -> bool:
	if _scene_attempted:
		return not _canonical_scene.is_empty()
	_scene_attempted = true
	var scene_file = FileAccess.open(SCENE_RESOURCE, FileAccess.READ)
	if scene_file == null:
		return false
	if scene_file.get_length() != SCENE_BYTES:
		scene_file.close()
		return false
	var bytes: PackedByteArray = scene_file.get_buffer(SCENE_BYTES)
	scene_file.close()
	if not _sha_matches(bytes, SCENE_SHA256):
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
	var mapping_file = FileAccess.open(MAPPING_RESOURCE, FileAccess.READ)
	if mapping_file == null:
		return
	var mapping_length: int = mapping_file.get_length()
	if mapping_length <= 0 or mapping_length > 3000:
		mapping_file.close()
		return
	var mapping_bytes: PackedByteArray = mapping_file.get_buffer(mapping_length)
	mapping_file.close()
	var text: String = mapping_bytes.get_string_from_utf8()
	if text.to_utf8_buffer() != mapping_bytes or not _valid_mapping(JSON.parse_string(text)):
		return
	var image_file = FileAccess.open(TEXTURE_RESOURCE, FileAccess.READ)
	if image_file == null:
		return
	if image_file.get_length() != PNG_BYTES:
		image_file.close()
		return
	var png_bytes: PackedByteArray = image_file.get_buffer(PNG_BYTES)
	image_file.close()
	if not _sha_matches(png_bytes, PNG_SHA256):
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
	if value.get("schema") != "wuxian.mh-fog-pier-exterior-r2-backplate.internal-use/v1" or value.get("resourceId") != "ART-MH-FOG-PIER-EXTERIOR-BACKPLATE-20261027-001" or value.get("revision") != "r2":
		return false
	if value.get("approval") != "ACCEPT_LIMITED_STATIC_MH_FOG_PIER_R2_BACKGROUND_USE" or value.get("GodotDisplay") != "NOT_RUN" or value.get("publicRelease") != "NOT_ESTABLISHED":
		return false
	if value.get("fallback") != "EXISTING_MH_R1_BACKPLATE_THEN_CLEAR_AND_FLOOR" or value.get("limitations") != "UNREGISTERED_NONINTERACTIVE_RUNTIME_NOT_RUN":
		return false
	var asset = value.get("asset")
	var composition = value.get("composition")
	var installation = value.get("installation")
	if not asset is Dictionary or asset.size() != 6 or not composition is Dictionary or composition.size() != 6 or not installation is Dictionary or installation.size() != 9:
		return false
	if asset.get("path") != TEXTURE_RESOURCE or asset.get("bytes") != PNG_BYTES or asset.get("sha256") != PNG_SHA256 or asset.get("width") != 1536 or asset.get("height") != 1024 or asset.get("format") != "RGB8":
		return false
	if composition.get("space") != "SCREEN_SPACE_BACKGROUND_ONLY" or composition.get("fit") != "CENTERED_COVER_FULL_VIEWPORT" or composition.get("uv") != "FULL_ONCE" or composition.get("filter") != "LINEAR" or composition.get("repeat") != "DISABLED" or composition.get("tint") != "WHITE_OPAQUE":
		return false
	if installation.get("worldId") != "mist_harbor" or installation.get("sceneId") != "mh_fog_pier" or not P.counter(installation.get("sceneSchemaVersion"), 1, 1) or installation.get("sceneSha256") != SCENE_SHA256:
		return false
	if installation.get("canonicalScenePath") != SCENE_RESOURCE or installation.get("canonicalSceneBytes") != SCENE_BYTES or installation.get("nonInteractive") != true or installation.get("unregistered") != true:
		return false
	var bounds = installation.get("boundsM")
	if not bounds is Array or bounds.size() != 4:
		return false
	return P.number(bounds[0], 0.0, 0.0) and P.number(bounds[1], 0.0, 0.0) and P.number(bounds[2], 24.0, 24.0) and P.number(bounds[3], 16.0, 16.0)
