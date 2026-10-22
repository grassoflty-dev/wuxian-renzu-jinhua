extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const PATH = 'res://assets/props/grey_hive/supply_crates_sheet.png'
const ID = 'runtime2d.prop.supply_crates.v1'
const SHA = '0dbad1364c14649b021f20510b9faf970ff317e6202131bd4cb2c32648b34b00'
const ContactProjection = preload('res://scripts/world/projection.gd')
const CONTACT_SCENE_SHA256 = '3cab424771d10799ee82a453a72117affb0ebca09dd814f25308be70310da9ee'
const CUTOUT = [[250,0],[539,20],[539,440],[295,549],[15,420],[15,20]]
var attempted = false
var texture: Texture2D
func _integers(value, expected: Array) -> bool:
	if not value is Array or value.size()!=expected.size():
		return false
	for n in expected.size():
		if not P.counter(value[n],0,4096) or value[n]!=expected[n]:
			return false
	return true
func prepare_resources() -> void:
	_load_once()
func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open('res://data/gh_supply_crates.internal-use.json',FileAccess.READ)
	if file==null:
		return
	if file.get_length()<=0 or file.get_length()>8192:
		file.close()
		return
	var bytes = file.get_buffer(file.get_length())
	file.close()
	if bytes.get_string_from_utf8().to_utf8_buffer()!=bytes:
		return
	var m = JSON.parse_string(bytes.get_string_from_utf8())
	if not m is Dictionary or m.get('schema')!='wuxian.gh-supply-crates.internal-use/v1' or typeof(m.get('registeredSourceReuse'))!=TYPE_BOOL or not m.registeredSourceReuse or m.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or m.get('GodotDisplay')!='NOT_RUN' or m.get('publicRelease')!='NOT_ESTABLISHED':
		return
	var row = m.get('asset')
	if not row is Dictionary or row.get('path')!=PATH or row.get('assetId')!=ID or row.get('sourceId')!=ID or row.get('sourceAdmission')!='release_approved' or row.get('sourceRuntimeStatus')!='ADOPT' or row.get('format')!='RGBA8' or not P.counter(row.get('bytes'),1,4194304) or row.bytes!=2067549 or row.get('sha256')!=SHA or not _integers(row.get('canvas'),[1238,1233]) or not _integers(row.get('region'),[0,640,540,550]) or not _integers(row.get('anchor'),[270,550]) or not P.number(row.get('heightM'),1,1):
		return
	var cutout = row.get('cutout')
	if not cutout is Array or cutout.size()!=6:
		return
	for n in 6:
		if not _integers(cutout[n],CUTOUT[n]) or cutout[n][0]>=540 or cutout[n][1]>=550:
			return
	var png_file = FileAccess.open(PATH,FileAccess.READ)
	if png_file==null:
		return
	if png_file.get_length()!=2067549:
		png_file.close()
		return
	var png = png_file.get_buffer(png_file.get_length())
	png_file.close()
	var hash = HashingContext.new()
	hash.start(HashingContext.HASH_SHA256)
	hash.update(png)
	if hash.finish().hex_encode()!=SHA:
		return
	var image = Image.new()
	if image.load_png_from_buffer(png)!=OK or image.get_width()!=1238 or image.get_height()!=1233 or image.get_format()!=Image.FORMAT_RGBA8:
		return
	texture = ImageTexture.create_from_image(image)
static func _contact_finite2(value: Variant) -> bool:
	return value is Vector2 and is_finite(value.x) and is_finite(value.y)

static func _contact_poly_valid(points: PackedVector2Array) -> bool:
	if points.size()<3:
		return false
	var origin: Vector2 = points[0]
	var winding = 0
	for index in points.size():
		var a: Vector2 = points[index]
		var b: Vector2 = points[(index+1)%points.size()]
		var c: Vector2 = points[(index+2)%points.size()]
		var edge = b-a
		var turn = edge.cross(c-b)
		if not _contact_finite2(a) or not _contact_finite2(b) or not _contact_finite2(c) or not _contact_finite2(edge) or edge==Vector2.ZERO or not is_finite(turn) or turn==0.0:
			return false
		if winding==0:
			winding = 1 if turn>0.0 else -1
		elif (turn>0.0)!=(winding>0):
			return false
	var area = 0.0
	for index in range(1,points.size()-1):
		var term: float = (points[index]-origin).cross(points[index+1]-origin)
		if not is_finite(term):
			return false
		area += term
		if not is_finite(area):
			return false
	return area!=0.0

static func _contact_polygons_overlap(a: PackedVector2Array, b: PackedVector2Array) -> Variant:
	if not _contact_poly_valid(a) or not _contact_poly_valid(b):
		return null
	var origin: Vector2 = a[0]
	for source in [a,b]:
		for index in source.size():
			var edge: Vector2 = source[(index+1)%source.size()]-source[index]
			if not _contact_finite2(edge) or edge==Vector2.ZERO:
				return null
			var raw_axis = Vector2(-edge.y,edge.x)
			if not _contact_finite2(raw_axis):
				return null
			var axis_scale: float = maxf(absf(raw_axis.x),absf(raw_axis.y))
			if not is_finite(axis_scale) or axis_scale<=0.0:
				return null
			var axis: Vector2 = raw_axis/axis_scale
			if not _contact_finite2(axis) or axis==Vector2.ZERO:
				return null
			var a_min = INF
			var a_max = -INF
			var b_min = INF
			var b_max = -INF
			for point: Vector2 in a:
				var relative: Vector2 = point-origin
				if not _contact_finite2(relative):
					return null
				var value: float = relative.dot(axis)
				if not is_finite(value):
					return null
				a_min = minf(a_min,value)
				a_max = maxf(a_max,value)
			for point: Vector2 in b:
				var relative: Vector2 = point-origin
				if not _contact_finite2(relative):
					return null
				var value: float = relative.dot(axis)
				if not is_finite(value):
					return null
				b_min = minf(b_min,value)
				b_max = maxf(b_max,value)
			var a_span: float = a_max-a_min
			var b_span: float = b_max-b_min
			if not is_finite(a_min) or not is_finite(a_max) or not is_finite(b_min) or not is_finite(b_max) or not is_finite(a_span) or not is_finite(b_span) or a_span<=0.0 or b_span<=0.0:
				return null
			if a_max<=b_min or b_max<=a_min:
				return false
	return true

static func _contact_projection_valid(projection: RefCounted, viewport: Rect2) -> bool:
	if not is_instance_valid(projection) or projection.get_script()!=ContactProjection:
		return false
	var initialized: Variant = projection.get('initialized')
	var origin: Variant = projection.get('origin')
	var center: Variant = projection.get('screen_center')
	var size: Variant = projection.get('viewport')
	var ppm: Variant = projection.get('ppm')
	return typeof(initialized)==TYPE_BOOL and initialized and origin is Vector3 and is_finite(origin.x) and is_finite(origin.y) and is_finite(origin.z) and center is Vector2 and _contact_finite2(center) and size is Vector2 and _contact_finite2(size) and size==viewport.size and P.number(ppm,0.0,256.0) and float(ppm)>0.0

static func _contact_canvas_valid(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, key: String, scene_hash: String, projection: RefCounted, viewport: Rect2) -> bool:
	if not is_instance_valid(canvas) or not canvas.is_inside_tree() or not canvas.is_visible_in_tree() or canvas.is_queued_for_deletion():
		return false
	var stored: Variant = canvas.get('viewport_rect')
	return canvas.get('snapshot')==snapshot and canvas.get('scene')==scene and canvas.get('current_key')==key and canvas.get('scene_hash')==scene_hash and canvas.get('camera')==projection and stored is Rect2 and stored==viewport and viewport.position==Vector2.ZERO and _contact_finite2(viewport.position) and _contact_finite2(viewport.size) and viewport.size.x>0.0 and viewport.size.y>0.0 and canvas.get_viewport_rect()==viewport and canvas.get_global_transform()==Transform2D.IDENTITY and canvas.get_global_transform_with_canvas()==Transform2D.IDENTITY and _contact_projection_valid(projection,viewport)

static func _contact_context_valid(snapshot: Dictionary, scene: Dictionary, key: Variant, scene_hash: Variant) -> bool:
	if not P.snapshot(snapshot) or not snapshot.has('entryToken') or snapshot.get('entryToken')!=null or snapshot.get('worldId')!='grey_hive' or snapshot.get('sceneId')!='gh_beacon' or not P.counter(snapshot.get('worldEpoch'),1):
		return false
	var epoch: Variant = snapshot.get('worldEpoch')
	var bounds: Variant = scene.get('boundsM')
	return typeof(key)==TYPE_STRING and key=='grey_hive:gh_beacon:'+str(epoch) and typeof(scene_hash)==TYPE_STRING and scene_hash==CONTACT_SCENE_SHA256 and scene.get('worldId')=='grey_hive' and scene.get('sceneId')=='gh_beacon' and bounds is Dictionary and P.number(bounds.get('x'),0.0,0.0) and P.number(bounds.get('z'),0.0,0.0) and P.number(bounds.get('width'),24.0,24.0) and P.number(bounds.get('depth'),16.0,16.0)

static func _contact_crate_row_valid(scene: Dictionary) -> bool:
	var presentation: Variant = scene.get('presentation')
	if not presentation is Dictionary:
		return false
	var sprites: Variant = presentation.get('sprites')
	if not sprites is Array:
		return false
	var sprite_rows: Array = sprites
	if not P.rows(sprite_rows,2048) or not P.unique(sprite_rows,'id'):
		return false
	var found = 0
	for row in sprites:
		if row.get('id')!='beacon_service_crates':
			continue
		found += 1
		var position: Variant = row.get('position')
		if row.get('assetId')!=ID or row.get('layer')!='visual.props_back' or not position is Array or position.size()!=3 or not P.number(position[0],5.0,5.0) or not P.number(position[1],0.0,0.0) or not P.number(position[2],11.5,11.5):
			return false
	return found==1

static func _contact_collision_clear(world: PackedVector2Array, scene: Dictionary) -> bool:
	var rows: Variant = scene.get('collision')
	if not rows is Array:
		return false
	var collision_rows: Array = rows
	if not P.rows(collision_rows,128) or collision_rows.size()!=6 or not P.unique(collision_rows,'id'):
		return false
	for row in collision_rows:
		var raw: Variant = row.get('polygon')
		if not raw is Array or raw.size()<3 or raw.size()>128:
			return false
		var pairs: Array = raw
		var obstacle = PackedVector2Array()
		for pair in pairs:
			if not pair is Array or pair.size()!=2 or not P.number(pair[0],0.0,4096.0) or not P.number(pair[1],0.0,4096.0):
				return false
			obstacle.append(Vector2(float(pair[0]),float(pair[1])))
		if not _contact_poly_valid(obstacle):
			return false
		var overlap: Variant = _contact_polygons_overlap(world,obstacle)
		if typeof(overlap)!=TYPE_BOOL or overlap:
			return false
	return true

static func _contact_color_valid(color: Color) -> bool:
	return is_finite(color.r) and is_finite(color.g) and is_finite(color.b) and is_finite(color.a) and color.r>=0.0 and color.r<=1.0 and color.g>=0.0 and color.g<=1.0 and color.b>=0.0 and color.b<=1.0 and color.a>=0.0 and color.a<=1.0

static func _contact_build_plan(center: Vector2, radii: Vector2, color: Color, scene: Dictionary, projection: RefCounted, viewport: Rect2) -> Dictionary:
	if not _contact_finite2(center) or not _contact_finite2(radii) or radii.x<=0.0 or radii.y<=0.0 or not _contact_color_valid(color):
		return {}
	var world = PackedVector2Array()
	var screen = PackedVector2Array()
	for index in 32:
		var angle = TAU*float(index)/32.0
		var point = center+Vector2(cos(angle)*radii.x,sin(angle)*radii.y)
		if not _contact_finite2(point) or point.x<0.0 or point.x>24.0 or point.y<0.0 or point.y>16.0:
			return {}
		var projected: Variant = projection.call('project',Vector3(point.x,0.0,point.y))
		if not _contact_finite2(projected):
			return {}
		world.append(point)
		screen.append(projected)
	if not _contact_poly_valid(world) or not _contact_poly_valid(screen) or not _contact_collision_clear(world,scene):
		return {}
	var visible = PackedVector2Array([viewport.position,viewport.position+Vector2(viewport.size.x,0.0),viewport.position+viewport.size,viewport.position+Vector2(0.0,viewport.size.y)])
	if not _contact_polygons_overlap(screen,visible):
		return {}
	return {'world':world,'points':screen,'color':color}

func draw_beacon_contact_shadow(canvas_value: Variant, snapshot_value: Variant, scene_value: Variant, current_key_value: Variant, scene_hash_value: Variant, projection_value: Variant, viewport_value: Variant) -> bool:
	if not canvas_value is CanvasItem or not is_instance_valid(canvas_value) or not snapshot_value is Dictionary or not scene_value is Dictionary or typeof(current_key_value)!=TYPE_STRING or typeof(scene_hash_value)!=TYPE_STRING or not projection_value is RefCounted or not is_instance_valid(projection_value) or not viewport_value is Rect2:
		return false
	var canvas: CanvasItem = canvas_value
	var snapshot: Dictionary = snapshot_value
	var scene: Dictionary = scene_value
	var current_key: String = current_key_value
	var scene_hash: String = scene_hash_value
	var projection: RefCounted = projection_value
	var viewport: Rect2 = viewport_value
	if not _contact_context_valid(snapshot,scene,current_key,scene_hash) or not _contact_canvas_valid(canvas,snapshot,scene,current_key,scene_hash,projection,viewport) or not _contact_crate_row_valid(scene) or texture==null or not is_instance_valid(texture):
		return false
	var texture_frame: Texture2D = texture
	var snapshot_frame = snapshot.duplicate(true)
	var scene_frame = scene.duplicate(true)
	var camera_frame = {'origin':projection.get('origin'),'center':projection.get('screen_center'),'viewport':projection.get('viewport'),'ppm':projection.get('ppm'),'initialized':projection.get('initialized')}
	var plan: Dictionary = _contact_build_plan(Vector2(5.0,11.5),Vector2(0.38,0.22),Color(0.015,0.020,0.022,0.09),scene,projection,viewport)
	if plan.is_empty() or snapshot_frame!=snapshot or scene_frame!=scene or projection.get('origin')!=camera_frame.origin or projection.get('screen_center')!=camera_frame.center or projection.get('viewport')!=camera_frame.viewport or projection.get('ppm')!=camera_frame.ppm or projection.get('initialized')!=camera_frame.initialized or not _contact_context_valid(snapshot,scene,current_key,scene_hash) or not _contact_canvas_valid(canvas,snapshot,scene,current_key,scene_hash,projection,viewport) or not _contact_crate_row_valid(scene) or texture!=texture_frame or not is_instance_valid(texture_frame):
		return false
	canvas.draw_colored_polygon(plan.points,plan.color)
	return true

func draw_body(canvas: CanvasItem, foot: Vector2, ppm: float) -> bool:
	if not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(ppm) or ppm<=0:
		return false
	var factor = ppm/550.0
	if not is_finite(factor) or factor<=0:
		return false
	_load_once()
	if texture==null:
		return false
	var points = PackedVector2Array()
	var uvs = PackedVector2Array()
	for pair in CUTOUT:
		var q = Vector2(pair[0],pair[1])
		var point = foot+(q-Vector2(270,550))*factor
		var uv = (q+Vector2(0,640))/Vector2(1238,1233)
		if not is_finite(point.x) or not is_finite(point.y) or not is_finite(uv.x) or not is_finite(uv.y):
			return false
		points.append(point)
		uvs.append(uv)
	canvas.draw_colored_polygon(points,Color.WHITE,uvs,texture)
	return true
