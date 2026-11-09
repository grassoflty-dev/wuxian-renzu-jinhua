extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const MAPPING_RESOURCE = "res://data/gh_entry_exterior_backplate_20261027_r1.internal-use.json"
const TEXTURE_RESOURCE = "res://assets/props/grey_hive/gh_entry_exterior_backplate_20261027_r1.png"
const PNG_BYTES = 2211708
const PNG_SHA256 = "1c936f9c07abe19a63b4277e2173817625905225b1ac55ff894d7969edec803c"
const SCENE_SHA256 = "4bf7a69a72e714a8de36c683f8b04fa7b1c89bbb9a358b2735dc7d2775915eaf"

var _attempted: bool = false
var _texture: CanvasTexture

func draw_backplate(canvas: CanvasItem, snapshot: Dictionary, scene_data: Dictionary, current_key: String, scene_hash: String, viewport: Rect2) -> bool:
	if not is_instance_valid(canvas) or not _valid_context(snapshot, scene_data, current_key, scene_hash) or not _valid_viewport(viewport):
		return false
	var scale: float = maxf(viewport.size.x / 1536.0, viewport.size.y / 1024.0)
	var size: Vector2 = Vector2(1536.0, 1024.0) * scale
	var position: Vector2 = viewport.position + (viewport.size - size) * 0.5
	if not is_finite(scale) or scale <= 0.0 or not is_finite(size.x) or not is_finite(size.y) or not is_finite(position.x) or not is_finite(position.y):
		return false
	_load_once()
	if _texture == null or not is_instance_valid(_texture):
		return false
	var previous_filter = canvas.texture_filter
	var previous_repeat = canvas.texture_repeat
	canvas.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
	canvas.texture_repeat = CanvasItem.TEXTURE_REPEAT_DISABLED
	canvas.draw_texture_rect(_texture, Rect2(position, size), false)
	canvas.texture_filter = previous_filter
	canvas.texture_repeat = previous_repeat
	return true

func _valid_viewport(viewport: Rect2) -> bool:
	return is_finite(viewport.position.x) and is_finite(viewport.position.y) and is_finite(viewport.size.x) and is_finite(viewport.size.y) and viewport.size.x > 0.0 and viewport.size.y > 0.0

func _valid_context(snapshot: Dictionary, scene_data: Dictionary, current_key: String, scene_hash: String) -> bool:
	if not P.snapshot(snapshot) or snapshot.get("worldId") != "grey_hive" or snapshot.get("sceneId") != "gh_entry_maintenance":
		return false
	if scene_data.get("worldId") != "grey_hive" or scene_data.get("sceneId") != "gh_entry_maintenance" or not P.counter(scene_data.get("schemaVersion"), 1, 1):
		return false
	if not P.counter(snapshot.get("worldEpoch"), 1) or current_key != "grey_hive:gh_entry_maintenance:" + str(snapshot.get("worldEpoch")) or scene_hash != SCENE_SHA256:
		return false
	var bounds = scene_data.get("boundsM")
	if not bounds is Dictionary or bounds.size() != 4:
		return false
	return P.number(bounds.get("x"), 0.0, 0.0) and P.number(bounds.get("z"), 0.0, 0.0) and P.number(bounds.get("width"), 20.0, 20.0) and P.number(bounds.get("depth"), 14.0, 14.0)

func _load_once() -> void:
	if _attempted:
		return
	_attempted = true
	var mapping_file = FileAccess.open(MAPPING_RESOURCE, FileAccess.READ)
	if mapping_file == null:
		return
	var mapping_length: int = mapping_file.get_length()
	if mapping_length <= 0 or mapping_length > 8192:
		mapping_file.close()
		return
	var mapping_bytes: PackedByteArray = mapping_file.get_buffer(mapping_length)
	mapping_file.close()
	var mapping_text: String = mapping_bytes.get_string_from_utf8()
	if mapping_text.to_utf8_buffer() != mapping_bytes or not _valid_mapping(JSON.parse_string(mapping_text)):
		return
	var image_file = FileAccess.open(TEXTURE_RESOURCE, FileAccess.READ)
	if image_file == null:
		return
	if image_file.get_length() != PNG_BYTES:
		image_file.close()
		return
	var png_bytes: PackedByteArray = image_file.get_buffer(PNG_BYTES)
	image_file.close()
	var hash: HashingContext = HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256) != OK or hash.update(png_bytes) != OK or hash.finish().hex_encode() != PNG_SHA256:
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

func _valid_mapping(value) -> bool:
	if not value is Dictionary or value.size() != 9:
		return false
	if value.get("schema") != "wuxian.gh-entry-exterior-backplate.internal-use/v1" or value.get("resourceId") != "ART-GH-ENTRY-MAINTENANCE-EXTERIOR-BACKPLATE-20261027-001" or value.get("revision") != "r1":
		return false
	if value.get("approval") != "ACCEPT_LIMITED_STATIC_GH_ENTRY_BACKGROUND_USE" or value.get("GodotDisplay") != "NOT_RUN" or value.get("publicRelease") != "NOT_ESTABLISHED":
		return false
	var asset = value.get("asset")
	var composition = value.get("composition")
	var installation = value.get("installation")
	if not asset is Dictionary or asset.size() != 6 or not composition is Dictionary or composition.size() != 4 or not installation is Dictionary or installation.size() != 6:
		return false
	if asset.get("path") != TEXTURE_RESOURCE or asset.get("bytes") != PNG_BYTES or asset.get("sha256") != PNG_SHA256 or asset.get("width") != 1536 or asset.get("height") != 1024 or asset.get("format") != "RGB8":
		return false
	if composition.get("space") != "SCREEN_SPACE_BACKGROUND_ONLY" or composition.get("fit") != "CENTERED_COVER_FULL_VIEWPORT" or composition.get("filter") != "LINEAR" or composition.get("repeat") != "DISABLED":
		return false
	if installation.get("worldId") != "grey_hive" or installation.get("sceneId") != "gh_entry_maintenance" or not P.counter(installation.get("sceneSchemaVersion"), 1, 1) or installation.get("sceneSha256") != SCENE_SHA256 or installation.get("nonInteractive") != true:
		return false
	var bounds = installation.get("boundsM")
	if not bounds is Array or bounds.size() != 4:
		return false
	return P.number(bounds[0], 0.0, 0.0) and P.number(bounds[1], 0.0, 0.0) and P.number(bounds[2], 20.0, 20.0) and P.number(bounds[3], 14.0, 14.0)
