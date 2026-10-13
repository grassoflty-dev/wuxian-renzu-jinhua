extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
var manifest: Dictionary = {}
var current: Dictionary = {}
var source_hash = ''
func _init() -> void:
	var f = FileAccess.open('res://data/scene_manifest.json',FileAccess.READ)
	if f != null:
		var value = JSON.parse_string(f.get_as_text())
		f.close()
		if value is Dictionary and value.size()==30:
			manifest = value
func build(world: String, scene: String) -> bool:
	current.clear()
	source_hash = ''
	if not manifest.has(scene):
		return false
	var value = manifest[scene]
	if not value is Dictionary or value.get('path')!='res://data/scenes/'+scene+'.json' or not P.counter(value.get('bytes'),1,1048576) or typeof(value.get('sha256'))!=TYPE_STRING or value.sha256.length()!=64:
		return false
	for c in value.sha256:
		if not c in '0123456789abcdef':
			return false
	var pin: Dictionary = value
	var f = FileAccess.open(pin.path,FileAccess.READ)
	if f == null or f.get_length()!=pin.bytes:
		return false
	var bytes = f.get_buffer(f.get_length())
	f.close()
	var hash = HashingContext.new()
	hash.start(HashingContext.HASH_SHA256)
	hash.update(bytes)
	if hash.finish().hex_encode()!=pin.sha256 or bytes.get_string_from_utf8().to_utf8_buffer()!=bytes:
		return false
	var scene_data = JSON.parse_string(bytes.get_string_from_utf8())
	if not scene_data is Dictionary or scene_data.get('schemaVersion')!=1 or scene_data.get('sceneId')!=scene or scene_data.get('worldId')!=world or not scene_data.get('boundsM') is Dictionary:
		return false
	var b: Dictionary = scene_data.boundsM
	if not P.number(b.get('x'),0,4096) or not P.number(b.get('z'),0,4096) or not P.number(b.get('width'),0.001,4096) or not P.number(b.get('depth'),0.001,4096):
		return false
	for field in ['collision','doors','interactions','hazards','objectives','occluders']:
		if not P.rows(scene_data.get(field,[]),2048):
			return false
	current = scene_data
	source_hash = pin.sha256
	return true
func contains(v: Vector3) -> bool:
	if current.is_empty():
		return false
	var b: Dictionary = current.boundsM
	return v.x>=b.x and v.x<=b.x+b.width and v.z>=b.z and v.z<=b.z+b.depth and v.y>=0 and v.y<=3.001
