extends RefCounted
# Packaged builds archive only the IMPORTED texture; the raw .png is not in the
# .pck. So the imported resource is the primary path and Image.load is a
# development-only fallback. Never gate the resource on ResourceLoader.exists():
# inside an exported package exists(path, 'Texture2D') can answer false while
# load(path) still returns the CompressedTexture2D.
static var load_sources: Dictionary = {}

static func load_texture(path: String) -> Texture2D:
	var loaded: Resource = ResourceLoader.load(path)
	if loaded is Texture2D:
		load_sources[path] = 'resource'
		return loaded
	loaded = ResourceLoader.load(path, 'Texture2D')
	if loaded is Texture2D:
		load_sources[path] = 'resource'
		return loaded
	var image := Image.new()
	if image.load(path) == OK and not image.is_empty():
		load_sources[path] = 'image'
		return ImageTexture.create_from_image(image)
	load_sources[path] = 'none'
	return null

static func atlas(path: String, region: Rect2) -> AtlasTexture:
	var texture := load_texture(path)
	if texture == null or texture.get_width() < region.end.x or texture.get_height() < region.end.y:
		return null
	var result = AtlasTexture.new()
	result.atlas = texture
	result.region = region
	return result
