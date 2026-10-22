extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')
const WALLS = {
	'beacon_wall_north':{'polygon':[[0,0],[24,0],[24,0.5],[0,0.5]],'inner':[[0,0.5],[24,0.5]],'inside':Vector2(0,-0.04)},
	'beacon_wall_south':{'polygon':[[0,15.5],[24,15.5],[24,16],[0,16]],'inner':[[0,15.5],[24,15.5]],'inside':Vector2(0,0.04)},
	'beacon_wall_west_north':{'polygon':[[0,0.5],[0.5,0.5],[0.5,6.5],[0,6.5]],'inner':[[0.5,0.5],[0.5,6.5]],'inside':Vector2(-0.04,0)},
	'beacon_wall_west_south':{'polygon':[[0,9.5],[0.5,9.5],[0.5,15.5],[0,15.5]],'inner':[[0.5,9.5],[0.5,15.5]],'inside':Vector2(-0.04,0)},
	'beacon_wall_east_north':{'polygon':[[23.5,0.5],[24,0.5],[24,6.5],[23.5,6.5]],'inner':[[23.5,0.5],[23.5,6.5]],'inside':Vector2(0.04,0)},
	'beacon_wall_east_south':{'polygon':[[23.5,9.5],[24,9.5],[24,15.5],[23.5,15.5]],'inner':[[23.5,9.5],[23.5,15.5]],'inside':Vector2(0.04,0)}
}
const EXPOSED_FACES = {
	'beacon_wall_north': [[[24,0],[24,0.5]]],
	'beacon_wall_south': [[[0,16],[24,16]],[[24,15.5],[24,16]]],
	'beacon_wall_west_north': [[[0,6.5],[0.5,6.5]]],
	'beacon_wall_west_south': [],
	'beacon_wall_east_north': [[[24,0.5],[24,6.5]],[[23.5,6.5],[24,6.5]]],
	'beacon_wall_east_south': [[[24,9.5],[24,15.5]]]
}
const REFERENCES = {
	'beacon_approved_floor':['runtime2d.grey_hive.floor_tiles.v1','visual.floor',Vector3(12,0,8)],
	'beacon_folded_approved':['runtime2d.prop.beacon.folded.v1','visual.props_dynamic',Vector3(8,0,8)],
	'beacon_mounted_approved':['runtime2d.prop.beacon.deployed.v1','visual.props_dynamic',Vector3(17,0,8)]
}

func _v2(value: Variant) -> bool:
	return typeof(value)==TYPE_VECTOR2 and is_finite(value.x) and is_finite(value.y)

func _v3(value: Variant) -> bool:
	return typeof(value)==TYPE_VECTOR3 and is_finite(value.x) and is_finite(value.y) and is_finite(value.z)

func _position(value: Variant, expected: Vector3) -> bool:
	return value is Array and value.size()==3 and P.number(value[0],expected.x,expected.x) and P.number(value[1],expected.y,expected.y) and P.number(value[2],expected.z,expected.z)

func _polygon(value: Variant, expected: Array) -> bool:
	if not value is Array or value.size()!=expected.size():
		return false
	for index: int in range(expected.size()):
		var point: Variant = value[index]
		var target: Array = expected[index]
		if not point is Array or point.size()!=2 or not P.number(point[0],float(target[0]),float(target[0])) or not P.number(point[1],float(target[1]),float(target[1])):
			return false
	return true

func _context(s: Variant, scene: Variant, key: Variant, projection: Variant, viewport: Variant) -> bool:
	if not s is Dictionary or not P.snapshot(s) or not scene is Dictionary:
		return false
	if s.get('worldId')!='grey_hive' or s.get('sceneId')!='gh_beacon' or scene.get('worldId')!='grey_hive' or scene.get('sceneId')!='gh_beacon' or not P.counter(s.get('worldEpoch'),1):
		return false
	if typeof(key)!=TYPE_STRING or key!='grey_hive:gh_beacon:'+str(s.worldEpoch) or not P.counter(scene.get('schemaVersion'),1,1):
		return false
	var bounds: Variant = scene.get('boundsM')
	if not bounds is Dictionary or bounds.size()!=4 or not P.number(bounds.get('x'),0,0) or not P.number(bounds.get('z'),0,0) or not P.number(bounds.get('width'),24,24) or not P.number(bounds.get('depth'),16,16):
		return false
	var collisions: Variant = scene.get('collision')
	if not P.rows(collisions,6) or collisions.size()!=6:
		return false
	var wall_ids: Dictionary = {}
	for row_value: Variant in collisions:
		if not row_value is Dictionary:
			return false
		var row: Dictionary = row_value
		var id: Variant = row.get('id')
		if typeof(id)!=TYPE_STRING or not WALLS.has(id) or wall_ids.has(id) or not _polygon(row.get('polygon'),WALLS[id].polygon):
			return false
		wall_ids[id] = true
	var presentation: Variant = scene.get('presentation')
	if not presentation is Dictionary:
		return false
	var sprites: Variant = presentation.get('sprites')
	if not P.rows(sprites,64):
		return false
	var sprite_ids: Dictionary = {}
	var reference_count: int = 0
	for row_value: Variant in sprites:
		if not row_value is Dictionary:
			return false
		var row: Dictionary = row_value
		var id: Variant = row.get('id')
		if not P.ident(id) or sprite_ids.has(id):
			return false
		sprite_ids[id] = true
		if REFERENCES.has(id):
			var expected: Array = REFERENCES[id]
			if row.get('assetId')!=expected[0] or row.get('layer')!=expected[1] or not _position(row.get('position'),expected[2]):
				return false
			reference_count += 1
	if reference_count!=3 or not projection is RefCounted or not is_instance_valid(projection) or projection.get_script()!=ProjectionScript:
		return false
	if not _v3(projection.get('origin')) or not _v2(projection.get('screen_center')) or not _v2(projection.get('viewport')) or not P.number(projection.get('ppm'),0.000001,256) or typeof(projection.get('initialized'))!=TYPE_BOOL or not projection.initialized:
		return false
	var size: Vector2 = projection.viewport
	if size.x<=0 or size.y<=0 or typeof(viewport)!=TYPE_RECT2:
		return false
	var rect: Rect2 = viewport
	return _v2(rect.position) and _v2(rect.size) and _v2(rect.end) and rect.size.x>0 and rect.size.y>0

func _canvas(canvas: Variant) -> bool:
	return canvas is CanvasItem and is_instance_valid(canvas) and canvas.is_inside_tree() and not canvas.is_queued_for_deletion()

func _faces_bound(canvas: CanvasItem, snapshot: Variant, scene: Variant, key: Variant, projection: Variant, viewport: Variant) -> bool:
	var actual_snapshot: Variant = canvas.get('snapshot')
	var actual_scene: Variant = canvas.get('scene')
	var actual_key: Variant = canvas.get('current_key')
	var actual_camera: Variant = canvas.get('camera')
	var actual_viewport: Variant = canvas.get('viewport_rect')
	return actual_snapshot is Dictionary and actual_snapshot == snapshot and actual_scene is Dictionary and actual_scene == scene and typeof(actual_key)==TYPE_STRING and actual_key==key and actual_camera is RefCounted and actual_camera==projection and typeof(actual_viewport)==TYPE_RECT2 and actual_viewport==viewport

func _exposed_quad(projection: Variant, segment_value: Variant, relief: Vector2) -> PackedVector2Array:
	if not segment_value is Array or segment_value.size()!=2:
		return PackedVector2Array()
	var coordinates: Array = []
	for endpoint_value: Variant in segment_value:
		if not endpoint_value is Array or endpoint_value.size()!=2 or not P.number(endpoint_value[0],0.0,24.0) or not P.number(endpoint_value[1],0.0,16.0):
			return PackedVector2Array()
		coordinates.append(endpoint_value)
	var ground: PackedVector2Array = _project(projection,coordinates)
	if ground.size()!=2:
		return PackedVector2Array()
	var quad: PackedVector2Array = PackedVector2Array([ground[0],ground[1],ground[1]-relief,ground[0]-relief])
	var twice_area: float = 0.0
	for index: int in range(quad.size()):
		var next_index: int = (index+1)%quad.size()
		twice_area += quad[index].x*quad[next_index].y-quad[next_index].x*quad[index].y
	if not is_finite(twice_area) or twice_area==0.0:
		return PackedVector2Array()
	return quad

func _project(projection: Variant, coordinates: Array) -> PackedVector2Array:
	var points: PackedVector2Array = PackedVector2Array()
	for source: Array in coordinates:
		var point: Variant = projection.call('project',Vector3(float(source[0]),0,float(source[1])))
		if not _v2(point):
			return PackedVector2Array()
		points.append(point)
	return points

func _paint(canvas: CanvasItem, commands: Array[Dictionary]) -> bool:
	if commands.is_empty() or commands.size()>64:
		return false
	for command: Dictionary in commands:
		var points: PackedVector2Array = command.points
		var colors: PackedColorArray = command.colors
		if points.size()<2 or colors.is_empty() or colors.size()!=1 and colors.size()!=points.size():
			return false
		for point: Vector2 in points:
			if not _v2(point):
				return false
		for color: Color in colors:
			if not is_finite(color.r) or not is_finite(color.g) or not is_finite(color.b) or not is_finite(color.a):
				return false
		if command.kind=='line' and not P.number(command.get('width'),0.000001,8):
			return false
	for command: Dictionary in commands:
		var points: PackedVector2Array = command.points
		var colors: PackedColorArray = command.colors
		if command.kind=='polygon':
			canvas.draw_polygon(points,colors)
		else:
			canvas.draw_polyline(points,colors[0],float(command.width),true)
	return true

func draw_floor(canvas: Variant, snapshot: Variant, scene: Variant, current_key: Variant, projection: Variant, viewport: Variant) -> bool:
	if not _canvas(canvas) or not _context(snapshot,scene,current_key,projection,viewport):
		return false
	var commands: Array[Dictionary] = []
	var base: Color = Color('#273a37')
	for column: int in range(6):
		for row: int in range(4):
			var x: float = float(column)*4.0
			var z: float = float(row)*4.0
			var corners: Array = [[x,z],[x+4,z],[x+4,z+4],[x,z+4]]
			var colors: PackedColorArray = PackedColorArray()
			var delta: float = 0.0
			for corner: Array in corners:
				var px: float = float(corner[0])
				var pz: float = float(corner[1])
				var edge: float = 1.0-smoothstep(0.0,2.0,minf(minf(px,24.0-px),minf(pz,16.0-pz)))
				var nx: float = (px-12.5)/6.0
				var nz: float = (pz-8.0)/3.5
				var bay: float = 1.0-smoothstep(0.0,1.0,sqrt(nx*nx+nz*nz))
				var value: float = delta-0.020*edge+0.018*bay
				colors.append(Color(base.r+value,base.g+value,base.b+value,1))
			commands.append({'kind':'polygon','points':_project(projection,corners),'colors':colors})
	var width: float = 0.65*float(projection.ppm)/48.0
	var joint: PackedColorArray = PackedColorArray([Color(0.05,0.08,0.075,0.10)])
	for x: int in [4,8,12,16,20]:
		commands.append({'kind':'line','points':_project(projection,[[x,0],[x,16]]),'colors':joint,'width':width})
	for z: int in [4,8,12]:
		commands.append({'kind':'line','points':_project(projection,[[0,z],[24,z]]),'colors':joint,'width':width})
	return _paint(canvas,commands)

func draw_wall(canvas: Variant, snapshot: Variant, scene: Variant, current_key: Variant, projection: Variant, wall: Variant, viewport: Variant) -> bool:
	if not _canvas(canvas) or not _context(snapshot,scene,current_key,projection,viewport) or not wall is Dictionary or not WALLS.has(wall.get('id')):
		return false
	var spec: Dictionary = WALLS[wall.id]
	if not _polygon(wall.get('polygon'),spec.polygon):
		return false
	var a: Vector2 = Vector2(float(spec.inner[0][0]),float(spec.inner[0][1]))
	var b: Vector2 = Vector2(float(spec.inner[1][0]),float(spec.inner[1][1]))
	var inside: Vector2 = spec.inside
	var ground: PackedVector2Array = _project(projection,spec.inner)
	var cap: PackedVector2Array = _project(projection,spec.polygon)
	if ground.size()!=2 or cap.size()!=4:
		return false
	var relief: Vector2 = Vector2(0,12.0*float(projection.ppm)/48.0)
	var face: PackedVector2Array = PackedVector2Array([ground[0],ground[1],ground[1]-relief,ground[0]-relief])
	var contact_b: Vector2 = b
	if wall.id in ['beacon_wall_west_south','beacon_wall_east_south']:
		# Native y-height of the existing relief; south-cap overlap is terminal.
		var south_cap_height: float = relief.y / float(projection.ppm)
		contact_b.y -= south_cap_height
		var clipped_ground: PackedVector2Array = _project(projection,[[contact_b.x,contact_b.y]])
		if clipped_ground.size()!=1:
			return false
		face = PackedVector2Array([ground[0],clipped_ground[0],ground[1]-relief,ground[0]-relief])
	for index: int in range(cap.size()):
		cap[index] -= relief
	var outline: PackedVector2Array = cap.duplicate()
	outline.append(cap[0])
	var commands: Array[Dictionary] = [
		{'kind':'polygon','points':_project(projection,[[a.x,a.y],[contact_b.x,contact_b.y],[contact_b.x+inside.x,contact_b.y+inside.y],[a.x+inside.x,a.y+inside.y]]),'colors':PackedColorArray([Color(0.02,0.06,0.045,0.12)])},
		{'kind':'polygon','points':face,'colors':PackedColorArray([Color('#26352f')])},
		{'kind':'polygon','points':cap,'colors':PackedColorArray([Color('#35483e')])},
		{'kind':'line','points':outline,'colors':PackedColorArray([Color(0.42745098,0.50980392,0.46666667,0.35)]),'width':float(projection.ppm)/48.0}
	]
	if _faces_bound(canvas,snapshot,scene,current_key,projection,viewport):
		var exposed_values: Variant = EXPOSED_FACES.get(wall.get('id'))
		if not exposed_values is Array:
			return false
		for face_index: int in range(exposed_values.size()):
			var segment_value: Variant = exposed_values[exposed_values.size()-1-face_index]
			var quad: PackedVector2Array = _exposed_quad(projection,segment_value,relief)
			if quad.size()!=4:
				return false
			commands.insert(commands.size()-2,{'kind':'polygon','points':quad,'colors':PackedColorArray([Color('#26352f')])})
	return _paint(canvas,commands)

func should_skip_mount_blockout(object: Variant, snapshot: Variant, scene: Variant, current_key: Variant, projection: Variant, viewport: Variant, drawn_bodies: Variant) -> bool:
	if not _context(snapshot,scene,current_key,projection,viewport) or not object is Dictionary or object.get('kind')!='prop' or object.get('id')!='beacon_storage_mount_blockout':
		return false
	var data: Variant = object.get('data')
	var foot: Variant = object.get('p')
	if not data is Dictionary or data.size()!=4 or data.get('id')!='beacon_storage_mount_blockout' or data.get('assetId')!='runtime2d.grey_hive.clutter.v1' or data.get('layer')!='visual.props_back' or not _position(data.get('position'),Vector3(17,0,8)) or not _v3(foot) or foot!=Vector3(17,0,8):
		return false
	var canonical_count: int = 0
	for row: Dictionary in scene.presentation.sprites:
		if row.get('id')=='beacon_storage_mount_blockout':
			if row.size()!=4 or row.get('assetId')!=data.assetId or row.get('layer')!=data.layer or not _position(row.get('position'),Vector3(17,0,8)):
				return false
			canonical_count += 1
	var beacon: Variant = snapshot.get('greyHiveBeacon')
	if canonical_count!=1 or not beacon is Dictionary or beacon.size()!=2 or beacon.get('state')!='mounted' or typeof(beacon.get('legacyCompletedWithoutReceipt'))!=TYPE_BOOL or beacon.legacyCompletedWithoutReceipt or not drawn_bodies is Dictionary:
		return false
	var expected: float = 72.0*(float(projection.ppm)/48.0)
	return P.number(drawn_bodies.get('beacon-body:gh_beacon_storage_mount_marker'),expected,expected)

# Candidate presentation calibration: one 600 ms symbolic beam/ring after an
# authenticated successful Mount receipt. This does not grant scan range.
func matches_deploy_context(snapshot: Variant, scene: Variant, current_key: Variant, projection: Variant, viewport: Variant) -> bool:
	if not _context(snapshot,scene,current_key,projection,viewport) or snapshot.get('entryToken')!=null or snapshot.player.currentHp<=0:
		return false
	var beacon: Variant = snapshot.get('greyHiveBeacon')
	if not beacon is Dictionary or beacon.size()!=2 or beacon.get('state')!='mounted' or typeof(beacon.get('legacyCompletedWithoutReceipt'))!=TYPE_BOOL or beacon.legacyCompletedWithoutReceipt:
		return false
	var interactions: Variant = scene.get('interactions')
	if not P.rows(interactions,64):
		return false
	var authored_count: int = 0
	for row_value: Variant in interactions:
		if not row_value is Dictionary:
			return false
		var row: Dictionary = row_value
		if row.get('id')=='gh_beacon_storage_mount_marker' or row.get('kind')=='beacon_mount':
			if row.get('id')!='gh_beacon_storage_mount_marker' or row.get('kind')!='beacon_mount' or not _position(row.get('position'),Vector3(17,0,8)) or row.get('event')!=null or row.get('environmentControl')!=null or not P.number(row.get('rangeM',2.5),2.5,2.5):
				return false
			authored_count += 1
	var native_count: int = 0
	for row_value: Variant in snapshot.interactables:
		if not row_value is Dictionary:
			return false
		var row: Dictionary = row_value
		if row.get('entityId')=='gh_beacon_storage_mount_marker' or row.get('kind')=='beacon_mount':
			if row.get('entityId')!='gh_beacon_storage_mount_marker' or row.get('kind')!='beacon_mount' or not P.transform(row.get('transform')) or P.vector(row.transform.positionM)!=Vector3(17,0,8):
				return false
			native_count += 1
	# The native mount becomes inactive after success; do not require active=true.
	return authored_count==1 and native_count==1

func _deploy_segment_visible(a: Vector2, b: Vector2, rect: Rect2) -> bool:
	if rect.has_point(a) or rect.has_point(b):
		return true
	var corners: PackedVector2Array = PackedVector2Array([rect.position,Vector2(rect.end.x,rect.position.y),rect.end,Vector2(rect.position.x,rect.end.y)])
	for index: int in range(4):
		if Geometry2D.segment_intersects_segment(a,b,corners[index],corners[(index+1)%4])!=null:
			return true
	return false

func draw_deploy_cue(canvas: Variant, snapshot: Variant, scene: Variant, current_key: Variant, projection: Variant, viewport: Variant, age_ms: Variant, drawn_bodies: Variant) -> bool:
	if not _canvas(canvas) or not matches_deploy_context(snapshot,scene,current_key,projection,viewport) or not P.number(age_ms,0,600) or float(age_ms)>=600.0 or not drawn_bodies is Dictionary:
		return false
	var ppm: float = float(projection.ppm)
	var expected: float = 72.0*(ppm/48.0)
	if not P.number(drawn_bodies.get('beacon-body:gh_beacon_storage_mount_marker'),expected,expected):
		return false
	var foot: Variant = projection.call('project',Vector3(17,0,8))
	if not _v2(foot):
		return false
	var t: float = float(age_ms)/600.0
	var half_width: float = 0.04*ppm
	var height: float = 2.0*ppm
	var beam: PackedVector2Array = PackedVector2Array([foot+Vector2(-half_width,0),foot+Vector2(half_width,0),foot+Vector2(half_width,-height),foot+Vector2(-half_width,-height)])
	for point: Vector2 in beam:
		if not _v2(point):
			return false
	var beam_area: float = (beam[1]-beam[0]).cross(beam[3]-beam[0])
	if not is_finite(beam_area) or beam_area==0.0:
		return false
	var ring: PackedVector2Array = PackedVector2Array()
	var radius: float = 0.3+0.7*t
	for index: int in range(32):
		var angle: float = TAU*float(index)/32.0
		var point: Variant = projection.call('project',Vector3(17.0+radius*cos(angle),0,8.0+radius*sin(angle)))
		if not _v2(point):
			return false
		ring.append(point)
	ring.append(ring[0])
	for index: int in range(32):
		var length_squared: float = ring[index].distance_squared_to(ring[index+1])
		if not is_finite(length_squared) or length_squared<=0.0:
			return false
	var width: float = ppm/48.0
	if not P.number(width,0.000001,8):
		return false
	var beam_rect: Rect2 = Rect2(beam[3],beam[1]-beam[3])
	var ring_view: Rect2 = viewport.grow(width*0.5)
	if not _v2(beam_rect.position) or not _v2(beam_rect.size) or not _v2(beam_rect.end) or not _v2(ring_view.position) or not _v2(ring_view.size) or not _v2(ring_view.end):
		return false
	var visible: bool = beam_rect.intersects(viewport)
	for index: int in range(32):
		if _deploy_segment_visible(ring[index],ring[index+1],ring_view):
			visible = true
	if not visible:
		return false
	var cyan: Color = Color('#b9eef2')
	var commands: Array[Dictionary] = [
		{'kind':'polygon','points':beam,'colors':PackedColorArray([Color(cyan.r,cyan.g,cyan.b,0.12*(1.0-t))])},
		{'kind':'line','points':ring,'colors':PackedColorArray([Color(cyan.r,cyan.g,cyan.b,0.18*(1.0-t))]),'width':width}
	]
	# _paint validates both complete commands and all colors before the first draw.
	return _paint(canvas,commands)
