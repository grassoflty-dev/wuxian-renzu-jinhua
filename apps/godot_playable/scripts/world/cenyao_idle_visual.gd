extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const DIRECTIONS = ['east','south_east','south','south_west','west','north_west','north','north_east']
const CANVAS = Vector2(336,560)
const ANCHOR = Vector2(168,551)
var attempted = false
var textures: Dictionary = {}
var _entry_shadow_offsets: Array = []

func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open('res://data/cenyao_idle.internal-use.json',FileAccess.READ)
	if file == null:
		return
	if file.get_length()>16384:
		file.close()
		return
	var bytes = file.get_buffer(file.get_length())
	file.close()
	if bytes.get_string_from_utf8().to_utf8_buffer()!=bytes:
		return
	var mapping = JSON.parse_string(bytes.get_string_from_utf8())
	if not mapping is Dictionary or mapping.get('schema')!='wuxian.cenyao-idle.internal-use/v1' or mapping.get('assetId')!='runtime2d.actor.cenyao.base.v1' or typeof(mapping.get('registeredSourceReuse'))!=TYPE_BOOL or not mapping.registeredSourceReuse or mapping.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or mapping.get('GodotDisplay')!='NOT_RUN' or mapping.get('publicRelease')!='NOT_ESTABLISHED':
		return
	if not _pair(mapping.get('canvas'),336,560) or not _pair(mapping.get('footAnchor'),168,551) or not P.counter(mapping.get('nominalCanvasHeight'),104,104) or not P.counter(mapping.get('referencePpm'),48,48) or not mapping.get('frames') is Dictionary or mapping.frames.size()!=8:
		return
	for direction in DIRECTIONS:
		var row = mapping.frames.get(direction)
		var expected = 'res://assets/characters/cenyao/idle/cenyao_idle_'+direction+'.png'
		if not row is Dictionary or row.get('path')!=expected or not P.counter(row.get('bytes'),1,4194304) or not _hash(row.get('sha256')) or row.get('state')!='idle' or not P.counter(row.get('frameCount'),1,1):
			continue
		var image_file = FileAccess.open(expected,FileAccess.READ)
		if image_file == null:
			continue
		if image_file.get_length()!=row.bytes:
			image_file.close()
			continue
		var png = image_file.get_buffer(image_file.get_length())
		image_file.close()
		var hash = HashingContext.new()
		hash.start(HashingContext.HASH_SHA256)
		hash.update(png)
		if hash.finish().hex_encode()!=row.sha256:
			continue
		var image = Image.new()
		if image.load_png_from_buffer(png)!=OK or image.get_width()!=336 or image.get_height()!=560 or image.get_format()!=Image.FORMAT_RGBA8:
			continue
		var texture = ImageTexture.create_from_image(image)
		if texture != null:
			textures[direction] = texture

func _pair(value, first: int, second: int) -> bool:
	return value is Array and value.size()==2 and P.counter(value[0],first,first) and P.counter(value[1],second,second)

func _hash(value) -> bool:
	if typeof(value)!=TYPE_STRING or value.length()!=64:
		return false
	for c in value:
		if not c in '0123456789abcdef':
			return false
	return true

# BEGIN additive GH Entry player contact shadow
func _entry_contact_polygon_valid(points: Array) -> bool:
	if points.size()<3 or points.size()>128: return false
	var area=0.0
	var winding=0.0
	var origin=Vector2.ZERO
	for i in range(points.size()):
		var av: Variant=points[i]; var bv: Variant=points[(i+1)%points.size()]; var cv: Variant=points[(i+2)%points.size()]
		if not av is Vector2 or not bv is Vector2 or not cv is Vector2: return false
		var a: Vector2=av; var b: Vector2=bv; var c: Vector2=cv
		if not is_finite(a.x) or not is_finite(a.y) or not is_finite(b.x) or not is_finite(b.y) or not is_finite(c.x) or not is_finite(c.y) or a==b: return false
		var turn: float=(b-a).cross(c-b)
		if not is_finite(turn) or turn==0.0: return false
		if winding==0.0: winding=signf(turn)
		elif turn*winding<=0.0: return false
		if i==0: origin=a
		if i<points.size()-2: area+=(b-origin).cross(c-origin)
	return is_finite(area) and area!=0.0

func _entry_contact_separated(a: Array, b: Array, axis: Vector2) -> Variant:
	var axis_length: float=axis.length_squared()
	if a.is_empty() or b.is_empty() or not is_finite(axis.x) or not is_finite(axis.y) or not is_finite(axis_length) or axis_length<=0.0: return null
	var first: Variant=a[0]
	if not first is Vector2: return null
	var origin: Vector2=first
	if not is_finite(origin.x) or not is_finite(origin.y): return null
	var ranges: Array[Vector2]=[]
	for polygon in [a,b]:
		var low: float=INF; var high: float=-INF
		for value in polygon:
			if not value is Vector2: return null
			var point: Vector2=value; var delta: Vector2=point-origin
			if not is_finite(point.x) or not is_finite(point.y) or not is_finite(delta.x) or not is_finite(delta.y): return null
			var projected: float=delta.dot(axis)
			if not is_finite(projected): return null
			low=minf(low,projected); high=maxf(high,projected)
		var span: float=high-low
		if not is_finite(low) or not is_finite(high) or not is_finite(span) or span<=0.0: return null
		ranges.append(Vector2(low,high))
	return ranges[0].y<=ranges[1].x or ranges[1].y<=ranges[0].x

func _entry_contact_positive_overlap(a: Array, b: Array) -> Variant:
	if not _entry_contact_polygon_valid(a) or not _entry_contact_polygon_valid(b): return null
	for axis in [Vector2.RIGHT,Vector2.DOWN]:
		var separated: Variant=_entry_contact_separated(a,b,axis)
		if separated!=false: return separated
	for shape in [a,b]:
		for i in range(shape.size()):
			var p: Vector2=shape[i]; var q: Vector2=shape[(i+1)%shape.size()]; var edge: Vector2=q-p
			var separated: Variant=_entry_contact_separated(a,b,Vector2(-edge.y,edge.x))
			if separated!=false: return separated
	return true

func _prepare_entry_shadow_offsets() -> bool:
	if not _entry_shadow_offsets.is_empty():
		return _entry_shadow_offsets.size()==32
	var offsets: Array = []
	for i in range(32):
		var angle: float = TAU*float(i)/32.0
		var x: float = cos(angle)*0.28
		var z: float = sin(angle)*0.18
		if not is_finite(x) or not is_finite(z):
			return false
		offsets.append([x,z])
	if offsets.size()!=32:
		return false
	_entry_shadow_offsets = offsets
	return true
func draw_entry_contact_shadow(canvas, snapshot, scene, current_key, scene_hash, projection, viewport) -> bool:
	if not canvas is CanvasItem or not snapshot is Dictionary or not scene is Dictionary or typeof(current_key)!=TYPE_STRING or typeof(scene_hash)!=TYPE_STRING or not projection is RefCounted or typeof(viewport)!=TYPE_RECT2:
		return false
	if not is_instance_valid(projection):
		return false
	if not is_instance_valid(canvas) or not canvas.is_inside_tree() or canvas.is_queued_for_deletion() or not canvas.is_visible_in_tree():
		return false
	var renderer_script: Variant = canvas.get_script()
	if not renderer_script is Script or renderer_script.resource_path!='res://scripts/world/renderer.gd':
		return false
	var host: Variant = canvas.get_parent()
	if not host is Node or not is_instance_valid(host) or not host.is_inside_tree() or host.is_queued_for_deletion():
		return false
	var host_script: Variant = host.get_script()
	if not host_script is Script or host_script.resource_path!='res://scripts/ui/v1_main.gd' or host.get('renderer')!=canvas:
		return false
	var native_snapshot_value: Variant = canvas.get('snapshot')
	var native_scene_value: Variant = canvas.get('scene')
	var native_key_value: Variant = canvas.get('current_key')
	var native_hash_value: Variant = canvas.get('scene_hash')
	var native_projection_value: Variant = canvas.get('camera')
	var stored_viewport_value: Variant = canvas.get('viewport_rect')
	var host_snapshot_value: Variant = host.get('snapshot')
	if not native_snapshot_value is Dictionary or not native_scene_value is Dictionary or typeof(native_key_value)!=TYPE_STRING or typeof(native_hash_value)!=TYPE_STRING or native_projection_value!=projection or typeof(stored_viewport_value)!=TYPE_RECT2 or not host_snapshot_value is Dictionary:
		return false
	if native_snapshot_value!=snapshot or native_scene_value!=scene or native_key_value!=current_key or native_hash_value!=scene_hash or host_snapshot_value!=snapshot:
		return false
	if not P.snapshot(snapshot) or not snapshot.has('entryToken') or snapshot.entryToken!=null or not P.counter(snapshot.get('worldEpoch'),1) or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_entry_maintenance' or snapshot.player.currentHp<=0:
		return false
	if scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_entry_maintenance' or scene_hash!='5deb6c01b2ad81144144ae2526c06f8c337839e8d880aeb1b3af3a225afe787b' or current_key!='grey_hive:gh_entry_maintenance:'+str(snapshot.worldEpoch):
		return false
	if not canvas.has_method('actor_visible') or canvas.call('actor_visible','player')!=true: return false
	var projection_script: Variant = projection.get_script()
	if not projection_script is Script or projection_script.resource_path!='res://scripts/world/projection.gd':
		return false
	var initialized_value: Variant = projection.get('initialized')
	var origin_value: Variant = projection.get('origin')
	var center_value: Variant = projection.get('screen_center')
	var projection_viewport_value: Variant = projection.get('viewport')
	var ppm_value: Variant = projection.get('ppm')
	if typeof(initialized_value)!=TYPE_BOOL or not initialized_value or not origin_value is Vector3 or not center_value is Vector2 or not projection_viewport_value is Vector2 or (typeof(ppm_value)!=TYPE_INT and typeof(ppm_value)!=TYPE_FLOAT):
		return false
	var origin: Vector3 = origin_value
	var screen_center: Vector2 = center_value
	var projection_viewport: Vector2 = projection_viewport_value
	var ppm: float = float(ppm_value)
	if not is_finite(origin.x) or not is_finite(origin.y) or not is_finite(origin.z) or not is_finite(screen_center.x) or not is_finite(screen_center.y) or not is_finite(projection_viewport.x) or not is_finite(projection_viewport.y) or not is_finite(ppm) or ppm<=0.0 or ppm>256.0:
		return false
	var actual_viewport: Rect2 = canvas.get_viewport_rect()
	var global_transform: Transform2D = canvas.get_global_transform()
	var canvas_transform: Transform2D = canvas.get_canvas_transform()
	if viewport.position!=Vector2.ZERO or viewport.size.x<=0.0 or viewport.size.y<=0.0 or actual_viewport!=viewport or stored_viewport_value!=viewport or projection_viewport!=viewport.size or global_transform!=Transform2D.IDENTITY or canvas_transform!=Transform2D.IDENTITY:
		return false
	var bounds_value: Variant = scene.get('boundsM')
	var collision_value: Variant = scene.get('collision')
	if not bounds_value is Dictionary or not collision_value is Array or collision_value.size()!=5:
		return false
	if not P.number(bounds_value.get('x'),0.0,0.0) or not P.number(bounds_value.get('z'),0.0,0.0) or not P.number(bounds_value.get('width'),20.0,20.0) or not P.number(bounds_value.get('depth'),14.0,14.0):
		return false
	var captured_snapshot: Dictionary = snapshot.duplicate(true)
	var captured_scene: Dictionary = scene.duplicate(true)
	var captured_key: String = current_key
	var captured_hash: String = scene_hash
	var captured_origin: Vector3 = origin
	var captured_center: Vector2 = screen_center
	var captured_camera_viewport: Vector2 = projection_viewport
	var captured_ppm: float = ppm
	var captured_viewport: Rect2 = viewport
	var captured_collisions: Array = captured_scene.get('collision')
	var player_value: Variant = captured_snapshot.get('player')
	if not player_value is Dictionary or not P.transform(player_value.get('transform')):
		return false
	var player: Dictionary = player_value
	var facing: Vector2 = Vector2(float(player.facingX)-float(player.facingZ),(float(player.facingX)+float(player.facingZ))*0.5)
	if not is_finite(facing.x) or not is_finite(facing.y) or facing.length_squared()==0.0:
		return false
	var sector: int = int(floor(atan2(facing.y,facing.x)/(PI/4.0)+0.5))
	var direction: String = DIRECTIONS[posmod(sector,8)]
	var cached_texture: Variant = textures.get(direction)
	if not cached_texture is Texture2D or not is_instance_valid(cached_texture):
		return false
	if not _prepare_entry_shadow_offsets():
		return false
	var native_position: Vector3 = P.vector(player.transform.positionM)
	var world_center: Vector3 = Vector3(native_position.x,0.0,native_position.z)
	if not is_finite(world_center.x) or not is_finite(world_center.z):
		return false
	var world_points: Array[Vector2] = []
	var screen_points: Array[Vector2] = []
	for i in range(32):
		var offset: Array = _entry_shadow_offsets[i]
		var point: Vector3 = Vector3(world_center.x+offset[0],0.0,world_center.z+offset[1])
		if not is_finite(point.x) or not is_finite(point.y) or not is_finite(point.z) or point.x<0.0 or point.x>20.0 or point.z<0.0 or point.z>14.0:
			return false
		world_points.append(Vector2(point.x,point.z))
		var projected: Vector2 = projection.project(point)
		if not is_finite(projected.x) or not is_finite(projected.y):
			return false
		screen_points.append(projected)
	if not _entry_contact_polygon_valid(world_points) or not _entry_contact_polygon_valid(screen_points):
		return false
	for collision_row_value in captured_collisions:
		if not collision_row_value is Dictionary:
			return false
		var collision_row: Dictionary = collision_row_value
		var polygon_value: Variant = collision_row.get('polygon')
		if not polygon_value is Array or polygon_value.size()!=4:
			return false
		var collision_points: Array[Vector2] = []
		for coordinate_value in polygon_value:
			if not coordinate_value is Array or coordinate_value.size()!=2 or not P.number(coordinate_value[0],0.0,20.0) or not P.number(coordinate_value[1],0.0,14.0):
				return false
			collision_points.append(Vector2(float(coordinate_value[0]),float(coordinate_value[1])))
		var collision_overlap: Variant=_entry_contact_positive_overlap(world_points,collision_points)
		if not _entry_contact_polygon_valid(collision_points) or collision_overlap!=false: return false
	var viewport_points: Array[Vector2] = [viewport.position,viewport.position+Vector2(viewport.size.x,0.0),viewport.position+viewport.size,viewport.position+Vector2(0.0,viewport.size.y)]
	if _entry_contact_positive_overlap(screen_points,viewport_points)!=true: return false
	var tint: Color = Color(0.015,0.020,0.022,0.10)
	if not is_finite(tint.r) or not is_finite(tint.g) or not is_finite(tint.b) or not is_finite(tint.a) or tint.r<0.0 or tint.r>1.0 or tint.g<0.0 or tint.g>1.0 or tint.b<0.0 or tint.b>1.0 or tint.a<=0.0 or tint.a>1.0:
		return false
	if not is_instance_valid(canvas) or not canvas.is_inside_tree() or canvas.is_queued_for_deletion() or not canvas.is_visible_in_tree() or not is_instance_valid(host) or host.is_queued_for_deletion() or host.get('renderer')!=canvas or canvas.get('snapshot')!=captured_snapshot or canvas.get('scene')!=captured_scene or canvas.get('current_key')!=captured_key or canvas.get('scene_hash')!=captured_hash or canvas.get('camera')!=projection or canvas.get('viewport_rect')!=captured_viewport or host.get('snapshot')!=captured_snapshot or canvas.get_viewport_rect()!=captured_viewport or canvas.get_global_transform()!=Transform2D.IDENTITY or canvas.get_canvas_transform()!=Transform2D.IDENTITY or not canvas.has_method('actor_visible') or canvas.call('actor_visible','player')!=true:
		return false
	if not is_instance_valid(projection) or projection.get('initialized')!=true or projection.get('origin')!=captured_origin or projection.get('screen_center')!=captured_center or projection.get('viewport')!=captured_camera_viewport or projection.get('ppm')!=captured_ppm or textures.get(direction)!=cached_texture or not is_instance_valid(cached_texture):
		return false
	# True reports only this optional shadow paint; it does not gate the player body.
	canvas.draw_colored_polygon(PackedVector2Array(screen_points),tint)
	return true
# END additive GH Entry player contact shadow

func draw_body(canvas: CanvasItem, foot: Vector2, player: Dictionary, ppm: float) -> bool:
	# Static pose only: current authority chooses orientation; there is no animation clock.
	if not is_finite(ppm) or ppm<=0 or not is_finite(foot.x) or not is_finite(foot.y) or not P.counter(player.get('currentHp'),1,4294967295) or not P.transform(player.get('transform')) or not P.number(player.get('facingX'),-1,1) or not P.number(player.get('facingZ'),-1,1):
		return false
	var facing = Vector2(player.facingX-player.facingZ,(player.facingX+player.facingZ)*0.5)
	if not is_finite(facing.x) or not is_finite(facing.y) or facing.length_squared()==0:
		return false
	var sector = int(floor(atan2(facing.y,facing.x)/(PI/4.0)+0.5))
	var direction = DIRECTIONS[posmod(sector,8)]
	_load_once()
	if not textures.has(direction):
		return false
	var factor = (104.0/560.0)*(ppm/48.0)
	if not is_finite(factor) or factor<=0:
		return false
	canvas.draw_texture_rect(textures[direction],Rect2(foot-ANCHOR*factor,CANVAS*factor),false,Color.WHITE)
	return true
