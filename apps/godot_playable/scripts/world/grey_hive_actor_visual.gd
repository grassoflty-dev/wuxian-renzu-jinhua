extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const WorldProjection = preload('res://scripts/world/projection.gd')
const ENTRY_SCENE_SHA = '5deb6c01b2ad81144144ae2526c06f8c337839e8d880aeb1b3af3a225afe787b'
const CONFIG = {
	'maintenance':{'id':'runtime2d.enemy.infected_maintenance_worker.v1','path':'res://assets/enemies/grey_hive/maintenance_sheet.png','bytes':1870188,'sha':'f7ee9460d18be46a8aea81fda2e566b29e5ac7b80d9ad2b20fd58f2f7e7a8168','source_sha':'6b7055df24acfa58edfdf0342c67efb60f9fac892c80110d449700bf7021477d','canvas':[1447,1086],'rect':[17,3,428,522],'foot':[266,511],'height':1.72},
	'security':{'id':'runtime2d.enemy.infected_security.v1','path':'res://assets/enemies/grey_hive/security_sheet.png','bytes':1827592,'sha':'cee3c171a14d82317e98ef00f39c3341a16364931dd8d56ec0ced330bcbecf66','source_sha':'7bd9d4cf19f36d189cdf81530c3dee48450179696a7dbceb24eaf375714de4e4','canvas':[1407,1072],'rect':[14,0,332,542],'foot':[190,537],'height':1.72},
	'sentinel':{'id':'runtime2d.enemy.sentinel.base.v1','path':'res://assets/enemies/grey_hive/sentinel_sheet.png','bytes':2160789,'sha':'c490ace4eb8ec9bf27e176359aefb8c3cd36c55a0651452be39ef25e4864df1d','source_sha':'f7ced75a0fcf35afacf796f23630a40c309682c86ef67af79de5414a8a16fbe2','canvas':[1442,1072],'rect':[10,0,386,541],'foot':[205,531],'height':2.6}
}
var attempted = false
var textures: Dictionary = {}
var _entry_shadow_offsets: Array = []
func _integers(value, expected: Array) -> bool:
	if not value is Array or value.size()!=expected.size():
		return false
	for n in range(expected.size()):
		if not P.counter(value[n],0,4096) or value[n]!=expected[n]:
			return false
	return true
func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open('res://data/grey_hive_actor.internal-use.json',FileAccess.READ)
	if file==null:
		return
	if file.get_length()<=0 or file.get_length()>16384:
		file.close()
		return
	var bytes = file.get_buffer(file.get_length())
	file.close()
	if bytes.get_string_from_utf8().to_utf8_buffer()!=bytes:
		return
	var mapping = JSON.parse_string(bytes.get_string_from_utf8())
	if not mapping is Dictionary or mapping.get('schema')!='wuxian.grey-hive-actor.internal-use/v1' or typeof(mapping.get('registeredSourceReuse'))!=TYPE_BOOL or not mapping.registeredSourceReuse or mapping.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or mapping.get('GodotDisplay')!='NOT_RUN' or mapping.get('publicRelease')!='NOT_ESTABLISHED' or not mapping.get('assets') is Dictionary or mapping.assets.size()!=3:
		return
	for key in CONFIG:
		var c: Dictionary = CONFIG[key]
		var row = mapping.assets.get(key)
		if not row is Dictionary or row.get('path')!=c.path or row.get('assetId')!=c.id or not P.counter(row.get('bytes'),1,4194304) or row.bytes!=c.bytes or row.get('sha256')!=c.sha or typeof(row.get('temporary_static_actor'))!=TYPE_BOOL or not row.temporary_static_actor or not _integers(row.get('canvas'),c.canvas) or not _integers(row.get('rect'),c.rect) or not _integers(row.get('absoluteFoot'),c.foot) or not P.number(row.get('visualHeightM'),c.height,c.height):
			continue
		var source = row.get('source')
		if not source is Dictionary or source.get('admission')!='release_approved' or source.get('runtimeStatus')!='ADOPT' or source.get('sha256')!=c.source_sha:
			continue
		if c.rect[2]<=0 or c.rect[3]<=0 or c.rect[0]+c.rect[2]>c.canvas[0] or c.rect[1]+c.rect[3]>c.canvas[1] or c.foot[0]<c.rect[0] or c.foot[0]>=c.rect[0]+c.rect[2] or c.foot[1]<c.rect[1] or c.foot[1]>=c.rect[1]+c.rect[3]:
			continue
		var png_file = FileAccess.open(c.path,FileAccess.READ)
		if png_file==null:
			continue
		if png_file.get_length()!=c.bytes:
			png_file.close()
			continue
		var png = png_file.get_buffer(png_file.get_length())
		png_file.close()
		var hash = HashingContext.new()
		if hash.start(HashingContext.HASH_SHA256)!=OK or hash.update(png)!=OK or hash.finish().hex_encode()!=c.sha:
			continue
		var image = Image.new()
		if image.load_png_from_buffer(png)!=OK or image.get_width()!=c.canvas[0] or image.get_height()!=c.canvas[1] or image.get_format()!=Image.FORMAT_RGBA8:
			continue
		var texture = ImageTexture.create_from_image(image)
		if texture!=null:
			textures[key] = texture
func canvas_rect(key: String, foot: Vector2, ppm: float) -> Rect2:
	if not CONFIG.has(key) or not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(ppm) or ppm<=0:
		return Rect2()
	var c: Dictionary = CONFIG[key]
	var factor = float(c.height)*ppm/float(c.rect[3])
	var size = Vector2(c.rect[2],c.rect[3])*factor
	var origin = foot-Vector2(c.foot[0]-c.rect[0],c.foot[1]-c.rect[1])*factor
	if not is_finite(factor) or factor<=0 or not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0 or not is_finite(origin.x) or not is_finite(origin.y):
		return Rect2()
	var body_rect = Rect2(origin,size)
	if not is_finite(body_rect.end.x) or not is_finite(body_rect.end.y):
		return Rect2()
	return body_rect
# GH-ENTRY-WORKER-CONTACT-SOURCE-20261022-001/r1
func _prepare_entry_shadow_offsets() -> bool:
	if not _entry_shadow_offsets.is_empty():
		return _entry_shadow_offsets.size() == 32
	var offsets: Array = []
	for index in range(32):
		var angle: float = TAU * float(index) / 32.0
		var x: float = cos(angle) * 0.25
		var z: float = sin(angle) * 0.16
		if not is_finite(x) or not is_finite(z):
			return false
		offsets.append([x,z])
	if offsets.size() != 32:
		return false
	_entry_shadow_offsets = offsets
	return true
func draw_entry_contact_shadows(canvas, snapshot, scene, current_key, scene_hash, projection, viewport) -> bool:
	if typeof(canvas) != TYPE_OBJECT or not is_instance_valid(canvas) or not canvas is CanvasItem:
		return false
	if not snapshot is Dictionary or not scene is Dictionary or typeof(current_key) != TYPE_STRING or typeof(scene_hash) != TYPE_STRING or not viewport is Rect2:
		return false
	if typeof(projection) != TYPE_OBJECT or not is_instance_valid(projection) or projection.get_script() != WorldProjection:
		return false
	var item: CanvasItem = canvas
	if not item.has_method('_grey_hive_body_key') or not item.has_method('actor_visible') or not item.is_inside_tree() or item.is_queued_for_deletion() or not item.visible or not item.is_visible_in_tree() or item.get_global_transform() != Transform2D.IDENTITY:
		return false
	if not P.snapshot(snapshot) or not snapshot.has('entryToken') or snapshot.get('entryToken') != null or not P.counter(snapshot.get('worldEpoch'),1) or snapshot.player.currentHp <= 0:
		return false
	if snapshot.get('worldId') != 'grey_hive' or snapshot.get('sceneId') != 'gh_entry_maintenance' or current_key != 'grey_hive:gh_entry_maintenance:' + str(snapshot.get('worldEpoch')) or scene_hash != ENTRY_SCENE_SHA:
		return false
	if scene.get('worldId') != 'grey_hive' or scene.get('sceneId') != 'gh_entry_maintenance' or item.get('snapshot') != snapshot or item.get('scene') != scene or item.get('current_key') != current_key or item.get('scene_hash') != scene_hash or item.get('camera') != projection:
		return false
	var actual_viewport: Rect2 = item.get_viewport_rect()
	if viewport.position != Vector2.ZERO or not is_finite(viewport.size.x) or not is_finite(viewport.size.y) or viewport.size.x <= 0.0 or viewport.size.y <= 0.0 or actual_viewport != viewport or item.get('viewport_rect') != viewport:
		return false
	if projection.get('initialized') != true:
		return false
	var camera_origin: Variant = projection.get('origin')
	var camera_center: Variant = projection.get('screen_center')
	var camera_viewport: Variant = projection.get('viewport')
	var camera_ppm: Variant = projection.get('ppm')
	if not camera_origin is Vector3 or not camera_center is Vector2 or not camera_viewport is Vector2 or not P.number(camera_ppm,0.000001,256):
		return false
	if not is_finite(camera_origin.x) or not is_finite(camera_origin.y) or not is_finite(camera_origin.z) or not is_finite(camera_center.x) or not is_finite(camera_center.y) or camera_viewport != viewport.size or camera_center != viewport.size * 0.5:
		return false
	var maintenance_texture: Variant = textures.get('maintenance')
	if not maintenance_texture is Texture2D or not is_instance_valid(maintenance_texture):
		return false
	if not _prepare_entry_shadow_offsets():
		return false
	var captured_snapshot: Dictionary = snapshot.duplicate(true)
	var captured_scene: Dictionary = scene.duplicate(true)
	var captured_key: String = current_key
	var captured_hash: String = scene_hash
	var captured_origin: Vector3 = camera_origin
	var captured_center: Vector2 = camera_center
	var captured_viewport: Vector2 = camera_viewport
	var captured_ppm: float = float(camera_ppm)
	var bounds: Variant = captured_scene.get('boundsM')
	if not bounds is Dictionary or bounds.get('x') != 0 or bounds.get('z') != 0 or bounds.get('width') != 20.0 or bounds.get('depth') != 14.0:
		return false
	var collision_rows: Variant = captured_scene.get('collision')
	if not P.rows(collision_rows,8) or collision_rows.size() != 5 or not P.unique(collision_rows,'id'):
		return false
	var collision_ids: Array = ['gh_entry_wall_north','gh_entry_wall_south','gh_entry_wall_west','gh_entry_wall_east_north','gh_entry_wall_east_south']
	var collision_bounds: Array = []
	for wall_id in collision_ids:
		var wall_matches: Array = collision_rows.filter(func(row): return row is Dictionary and row.get('id') == wall_id)
		if wall_matches.size() != 1:
			return false
		var polygon: Variant = wall_matches[0].get('polygon')
		if not polygon is Array or polygon.size() != 4:
			return false
		var low := Vector2(INF,INF)
		var high := Vector2(-INF,-INF)
		for vertex in polygon:
			if not vertex is Array or vertex.size() != 2 or not P.number(vertex[0],0,20) or not P.number(vertex[1],0,14):
				return false
			var point := Vector2(float(vertex[0]),float(vertex[1]))
			low = low.min(point)
			high = high.max(point)
		if low.x >= high.x or low.y >= high.y:
			return false
		collision_bounds.append(Rect2(low,high-low))
	var viewport_corners: Array = [Vector2.ZERO,Vector2(viewport.size.x,0),viewport.size,Vector2(0,viewport.size.y)]
	var worker_ids: Array = ['gh_entry_worker_01','gh_entry_worker_02']
	var plans: Array = []
	for worker_id in worker_ids:
		var actor_matches: Array = captured_snapshot.actors.filter(func(row): return row.get('entityId') == worker_id)
		if actor_matches.size() != 1:
			continue
		var actor: Dictionary = actor_matches[0]
		if actor.get('actorKind') != 'enemy' or actor.get('entityType') != 'runtime2d.enemy.infected_maintenance_worker.v1' or actor.get('active') != true or not P.transform(actor.get('transform')):
			continue
		var body_key: Variant = item.call('_grey_hive_body_key',actor)
		var visible: Variant = item.call('actor_visible',worker_id)
		if typeof(body_key) != TYPE_STRING or body_key != 'maintenance' or typeof(visible) != TYPE_BOOL or not visible:
			continue
		var position: Variant = actor.transform.get('positionM')
		if not P.point(position):
			continue
		var center := Vector2(float(position.xM),float(position.zM))
		var world_bounds := Rect2(center-Vector2(0.25,0.16),Vector2(0.5,0.32))
		if world_bounds.position.x < 0.0 or world_bounds.position.y < 0.0 or world_bounds.end.x > 20.0 or world_bounds.end.y > 14.0:
			continue
		var hits_collision := false
		for wall_bounds in collision_bounds:
			if world_bounds.position.x < wall_bounds.end.x and world_bounds.end.x > wall_bounds.position.x and world_bounds.position.y < wall_bounds.end.y and world_bounds.end.y > wall_bounds.position.y:
				hits_collision = true
				break
		if hits_collision:
			continue
		var shadow_color := Color(0.015,0.020,0.022,0.10)
		if not is_finite(shadow_color.r) or not is_finite(shadow_color.g) or not is_finite(shadow_color.b) or not is_finite(shadow_color.a) or shadow_color.r < 0.0 or shadow_color.g < 0.0 or shadow_color.b < 0.0 or shadow_color.a <= 0.0 or shadow_color.r > 1.0 or shadow_color.g > 1.0 or shadow_color.b > 1.0 or shadow_color.a > 1.0:
			continue
		var points := PackedVector2Array()
		var screen_low := Vector2(INF,INF)
		var screen_high := Vector2(-INF,-INF)
		var valid_points := true
		for index in range(32):
			var offset: Array = _entry_shadow_offsets[index]
			var world_point := Vector3(center.x + offset[0],0.0,center.y + offset[1])
			if not is_finite(world_point.x) or not is_finite(world_point.y) or not is_finite(world_point.z):
				valid_points = false
				break
			var screen_point: Vector2 = projection.project(world_point)
			if not is_finite(screen_point.x) or not is_finite(screen_point.y):
				valid_points = false
				break
			points.append(screen_point)
			screen_low = screen_low.min(screen_point)
			screen_high = screen_high.max(screen_point)
		if not valid_points or points.size() != 32 or screen_low.x >= screen_high.x or screen_low.y >= screen_high.y:
			continue
		var twice_area: float = 0.0
		var turn_direction: int = 0
		for index in range(32):
			var first: Vector2 = points[index]
			var second: Vector2 = points[(index+1)%32]
			var third: Vector2 = points[(index+2)%32]
			var edge := second-first
			var next_edge := third-second
			var turn: float = edge.cross(next_edge)
			if edge.length_squared() <= 0.0 or not is_finite(turn) or turn == 0.0:
				valid_points = false
				break
			var direction: int = 1 if turn > 0.0 else -1
			if turn_direction != 0 and direction != turn_direction:
				valid_points = false
				break
			turn_direction = direction
			twice_area += first.cross(second)
		if not valid_points or not is_finite(twice_area) or twice_area == 0.0:
			continue
		var axes: Array = [Vector2.RIGHT,Vector2.DOWN]
		for index in range(32):
			var edge: Vector2 = points[(index+1)%32]-points[index]
			axes.append(Vector2(-edge.y,edge.x))
		var outside_viewport := false
		for axis in axes:
			if not is_finite(axis.x) or not is_finite(axis.y) or axis.length_squared() <= 0.0:
				valid_points = false
				break
			var poly_low := INF
			var poly_high := -INF
			var view_low := INF
			var view_high := -INF
			for point in points:
				var value: float = point.dot(axis)
				if not is_finite(value):
					valid_points = false
					break
				poly_low = minf(poly_low,value)
				poly_high = maxf(poly_high,value)
			for corner in viewport_corners:
				var value: float = corner.dot(axis)
				if not is_finite(value):
					valid_points = false
					break
				view_low = minf(view_low,value)
				view_high = maxf(view_high,value)
			if not valid_points:
				break
			if poly_high <= view_low or view_high <= poly_low:
				outside_viewport = true
				break
		if not valid_points or outside_viewport:
			continue
		plans.append({'id':worker_id,'points':points,'color':shadow_color})
	if plans.is_empty() or plans.size() > 2:
		return false
	if not is_instance_valid(item) or not item.is_inside_tree() or item.is_queued_for_deletion() or not item.visible or not item.is_visible_in_tree() or item.get_global_transform() != Transform2D.IDENTITY:
		return false
	if snapshot != captured_snapshot or scene != captured_scene or current_key != captured_key or scene_hash != captured_hash or item.get('snapshot') != captured_snapshot or item.get('scene') != captured_scene or item.get('current_key') != captured_key or item.get('scene_hash') != captured_hash or item.get('camera') != projection:
		return false
	if item.get_viewport_rect() != viewport or item.get('viewport_rect') != viewport or viewport.position != Vector2.ZERO or projection.get_script() != WorldProjection or projection.get('initialized') != true or projection.get('origin') != captured_origin or projection.get('screen_center') != captured_center or projection.get('viewport') != captured_viewport or projection.get('ppm') != captured_ppm or textures.get('maintenance') != maintenance_texture:
		return false
	for plan in plans:
		var actor_matches: Array = captured_snapshot.actors.filter(func(row): return row.get('entityId') == plan.get('id'))
		if actor_matches.size() != 1 or item.call('_grey_hive_body_key',actor_matches[0]) != 'maintenance' or item.call('actor_visible',plan.get('id')) != true:
			return false
	for plan in plans:
		item.draw_colored_polygon(plan.get('points'),plan.get('color'))
	return true

func draw_body(canvas: CanvasItem, key: String, foot: Vector2, ppm: float) -> bool:
	var body_rect = canvas_rect(key,foot,ppm)
	if body_rect.size.x<=0 or body_rect.size.y<=0:
		return false
	_load_once()
	if not textures.has(key):
		return false
	var c: Dictionary = CONFIG[key]
	canvas.draw_texture_rect_region(textures[key],body_rect,Rect2(c.rect[0],c.rect[1],c.rect[2],c.rect[3]),Color.WHITE,false,true)
	return true
