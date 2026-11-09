extends RefCounted
static func atlas(path: String, region: Rect2) -> AtlasTexture:
	var texture: Texture2D = null
	if ResourceLoader.exists(path, 'Texture2D'):
		texture = ResourceLoader.load(path, 'Texture2D') as Texture2D
	if texture == null:
		var image = Image.new()
		if image.load(path) == OK:
			texture = ImageTexture.create_from_image(image)
	if texture == null or texture.get_width() < region.end.x or texture.get_height() < region.end.y:
		return null
	var result = AtlasTexture.new()
	result.atlas = texture
	result.region = region
	return result
