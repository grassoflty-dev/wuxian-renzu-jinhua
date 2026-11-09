extends RefCounted
const BulkheadAtlasTexture = preload('res://scripts/world/bulkhead_atlas_texture.gd')
const P = preload('res://scripts/bridge/protocol.gd')
const MAPPING = 'res://data/gh_exit_sealed_barrier.internal-use.json'
const ASSET = 'res://assets/props/return_station/bulkhead_tiles.png'
const ASSET_SHA = 'c5a18cae9a3ddc95c8515f6b342cb7ca3be12c23ff502ab5b3f16000bb4c9a5b'
const SCENE_SHA = '6ab262467e1992feac94f27d8bf571dcf77ddbc8c1693a79c57b3aeebff0c54c'
const POLYGON = [[21.5,0.5],[22.5,0.5],[22.5,15.5],[21.5,15.5]]
var _attempted = false
var _texture: ImageTexture
var _shadow_offsets: Array = []

func matches_barrier(wall) -> bool:
	return wall is Dictionary and wall.size()==3 and wall.get('id')=='exit_sealed_extraction_barrier' and wall.get('requiresEvent')=='hive_extraction' and wall.get('polygon')==POLYGON

func build_packets(snapshot: Dictionary, scene: Dictionary, key: String, scene_hash: String, projection, open: bool) -> Array:
	if open or not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_exit':
		return []
	if scene_hash!=SCENE_SHA or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_exit' or not P.counter(snapshot.get('worldEpoch'),1) or key!='grey_hive:gh_exit:'+str(snapshot.worldEpoch):
		return []
	if projection == null or not projection is Object or not projection.has_method('project') or not P.number(projection.get('ppm'),0.000001,256.0):
		return []
	var collision = scene.get('collision')
	if not P.rows(collision,2048) or not P.unique(collision,'id'):
		return []
	var matches = collision.filter(func(row): return row.get('id')=='exit_sealed_extraction_barrier')
	if matches.size()!=1 or not matches_barrier(matches[0]):
		return []
	_load_once()
	if _texture==null or _shadow_offsets.size()!=16:
		return []
	var ppm: float = float(projection.get('ppm'))
	var uvs = PackedVector2Array([Vector2(0,0),Vector2(355.0/1418.0,0),Vector2(355.0/1418.0,480.0/1028.0),Vector2(0,480.0/1028.0)])
	var packets: Array = []
	for i in range(15):
		var a_world = Vector3(22,0,0.5+float(i))
		var b_world = Vector3(22,0,1.5+float(i))
		var a = projection.project(a_world)
		var b = projection.project(b_world)
		if not _finite_point(a) or not _finite_point(b) or b.distance_to(a+Vector2(-ppm,0.5*ppm))>0.01:
			return []
		var sx = -ppm/250.0
		var sy = (0.5*ppm-138.0*(ppm/250.0))/250.0
		var vertical = ppm/250.0
		var points = PackedVector2Array()
		for pixel in [Vector2(0,0),Vector2(355,0),Vector2(355,480),Vector2(0,480)]:
			var d = pixel-Vector2(50,322)
			var p = a+Vector2(sx*d.x,sy*d.x+vertical*d.y)
			if not _finite_point(p):
				return []
			points.append(p)
		var bounds = _bounds(points)
		if not _finite_rect(bounds):
			return []
		var shadow = PackedVector2Array()
		for offset in _shadow_offsets:
			var sp = projection.project(Vector3(22+offset[0],0,1.0+float(i)+offset[1]))
			if not _finite_point(sp):
				return []
			shadow.append(sp)
		var shadow_bounds = _bounds(shadow)
		if not _finite_rect(shadow_bounds):
			return []
		packets.append({'id':'exit_barrier_module_'+str(i).pad_zeros(2),'p':Vector3(22,0,1.0+float(i)),'points':points,'uvs':uvs,'body_rect':bounds,'shadow':shadow,'shadow_rect':shadow_bounds})
	return packets if packets.size()==15 else []

func draw_shadow(canvas: CanvasItem, packet: Dictionary, viewport: Rect2) -> void:
	if viewport.intersects(packet.shadow_rect):
		canvas.draw_colored_polygon(packet.shadow,Color(0.04,0.05,0.055,0.16))

func draw_body(canvas: CanvasItem, packet: Dictionary) -> void:
	var tint = Color(0.62,0.68,0.66,1.0)
	canvas.draw_polygon(packet.points,PackedColorArray([tint,tint,tint,tint]),packet.uvs,_texture)

func _load_once() -> void:
	if _attempted:
		return
	_attempted = true
	var mf = FileAccess.open(MAPPING,FileAccess.READ)
	if mf==null:
		return
	var mapping_bytes = mf.get_buffer(mf.get_length())
	mf.close()
	var mapping = JSON.parse_string(mapping_bytes.get_string_from_utf8())
	if not mapping is Dictionary or mapping.size()!=18 or mapping.get('world')!='grey_hive' or mapping.get('scene')!='gh_exit' or mapping.get('barrier')!='exit_sealed_extraction_barrier' or mapping.get('event')!='hive_extraction' or mapping.get('canvas')!=[1418,1028] or mapping.get('groundX')!=22 or mapping.get('startZ')!=0.5 or mapping.get('cadence')!=1 or mapping.get('schema')!='wuxian.gh-exit-sealed-barrier.internal-use/v1' or mapping.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or mapping.get('sceneSha256')!=SCENE_SHA or mapping.get('asset')!=ASSET or mapping.get('assetSha256')!=ASSET_SHA or mapping.get('assetBytes')!=1673423 or mapping.get('frame')!=[0,0,355,480] or mapping.get('sourceFeet')!=[[50,322],[300,460]] or mapping.get('modules')!=15 or mapping.get('scope')!='visualOnlyNoCollisionOrState':
		return
	var f = FileAccess.open(ASSET,FileAccess.READ)
	if f==null:
		return
	if f.get_length()!=1673423:
		f.close()
		return
	var bytes = f.get_buffer(f.get_length())
	f.close()
	var hash = HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256)!=OK or hash.update(bytes)!=OK or hash.finish().hex_encode()!=ASSET_SHA:
		return
	var image = Image.new()
	if image.load_png_from_buffer(bytes)!=OK or image.get_width()!=1418 or image.get_height()!=1028 or image.get_format()!=Image.FORMAT_RGBA8:
		return
	var candidate_texture = BulkheadAtlasTexture.acquire(image, ASSET, 1673423, ASSET_SHA)
	if candidate_texture==null:
		return
	var offsets: Array = []
	for k in range(16):
		var angle = TAU*float(k)/16.0
		var x: float = cos(angle)*0.18
		var z: float = sin(angle)*0.48
		if not is_finite(x) or not is_finite(z):
			return
		offsets.append([x,z])
	if offsets.size()!=16:
		return
	_shadow_offsets = offsets
	_texture = candidate_texture

func _finite_point(p) -> bool:
	return p is Vector2 and is_finite(p.x) and is_finite(p.y)
func _finite_rect(r: Rect2) -> bool:
	return _finite_point(r.position) and _finite_point(r.size) and _finite_point(r.end) and r.size.x>0 and r.size.y>0
func _bounds(points: PackedVector2Array) -> Rect2:
	if points.is_empty():
		return Rect2()
	var lo = points[0]
	var hi = points[0]
	for point in points:
		lo = lo.min(point)
		hi = hi.max(point)
	return Rect2(lo,hi-lo)