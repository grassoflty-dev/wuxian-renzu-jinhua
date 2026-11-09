extends RefCounted
# Fixed-identity texture sharing only after each caller has validated its own source.
const ASSET_PATH = 'res://assets/props/return_station/bulkhead_tiles.png'
const ASSET_BYTES = 1673423
const ASSET_SHA = 'c5a18cae9a3ddc95c8515f6b342cb7ca3be12c23ff502ab5b3f16000bb4c9a5b'
static var _texture: ImageTexture
static func acquire(image: Image, asset_path: String, asset_bytes: int, asset_sha: String) -> ImageTexture:
	if asset_path!=ASSET_PATH or asset_bytes!=ASSET_BYTES or asset_sha!=ASSET_SHA or not is_instance_valid(image):
		return null
	if image.get_width()!=1418 or image.get_height()!=1028 or image.get_format()!=Image.FORMAT_RGBA8:
		return null
	if is_instance_valid(_texture):
		return _texture
	var candidate = ImageTexture.create_from_image(image)
	if not is_instance_valid(candidate):
		return null
	_texture = candidate
	return _texture
