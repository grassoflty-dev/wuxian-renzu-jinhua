extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const ProjectionScript = preload('res://scripts/world/projection.gd')

const PACKET_KIND = 'mh_fog_pier_ground_v1'
const WORLD_ID = 'mist_harbor'
const SCENE_ID = 'mh_fog_pier'
const WALKABLE_ID = 'mh_fog_pier_walkable_area'
const STONE_ID = 'approved_stone_water_module'
const STONE_ASSET = 'runtime2d.world.mistharbor.water_edge_stone.v1'
const FILL_COLORS = [Color('#243943'), Color('#283d46'), Color('#2b4049')]
const SEAM_COLOR = Color('#192d36')
const SEAM_ALPHA = 0.45
const ROW_COUNT = 15
const COLUMN_COUNT = 12
const CELL_COUNT = 180

func _finite_vector2(value) -> bool:
	return typeof(value) == TYPE_VECTOR2 and is_finite(value.x) and is_finite(value.y)

func _finite_rect2(value, positive: bool = false) -> bool:
	if typeof(value) != TYPE_RECT2:
		return false
	var rect: Rect2 = value
	return is_finite(rect.position.x) and is_finite(rect.position.y) and is_finite(rect.size.x) and is_finite(rect.size.y) and (rect.size.x > 0.0 and rect.size.y > 0.0 if positive else rect.size.x >= 0.0 and rect.size.y >= 0.0)

func _finite_color(value) -> bool:
	return typeof(value) == TYPE_COLOR and is_finite(value.r) and is_finite(value.g) and is_finite(value.b) and is_finite(value.a)

func _valid_bounds(scene: Dictionary) -> bool:
	var bounds = scene.get('boundsM')
	if not bounds is Dictionary or bounds.size() != 4:
		return false
	return P.number(bounds.get('x'), 0.0, 0.0) and P.number(bounds.get('z'), 0.0, 0.0) and P.number(bounds.get('width'), 24.0, 24.0) and P.number(bounds.get('depth'), 16.0, 16.0)

func _same_point2(value, x: float, z: float) -> bool:
	return value is Array and value.size() == 2 and P.number(value[0], x, x) and P.number(value[1], z, z)

func _valid_walkable(scene: Dictionary) -> bool:
	var polygons = scene.get('walkablePolygons')
	if not polygons is Array or polygons.size() != 1:
		return false
	var row = polygons[0]
	if not row is Dictionary or row.size() != 2 or row.get('id') != WALKABLE_ID:
		return false
	var polygon = row.get('polygon')
	return polygon is Array and polygon.size() == 4 and _same_point2(polygon[0], 0.5, 0.5) and _same_point2(polygon[1], 23.5, 0.5) and _same_point2(polygon[2], 23.5, 15.5) and _same_point2(polygon[3], 0.5, 15.5)

func _valid_authored_floor(scene: Dictionary) -> bool:
	var presentation = scene.get('presentation')
	if not presentation is Dictionary:
		return false
	var sprites = presentation.get('sprites')
	if not sprites is Array or sprites.size() > 64:
		return false
	var seen_ids: Dictionary = {}
	var candidate_count = 0
	for row in sprites:
		if not row is Dictionary:
			return false
		var id = row.get('id')
		var asset_id = row.get('assetId')
		var layer = row.get('layer')
		var position = row.get('position')
		if not P.ident(id) or seen_ids.has(id) or not P.ident(asset_id) or not P.ident(layer) or not position is Array or position.size() != 3:
			return false
		for coordinate in position:
			if not P.number(coordinate):
				return false
		seen_ids[id] = true
		if id == STONE_ID or asset_id == STONE_ASSET:
			candidate_count += 1
			if row.size() != 4 or id != STONE_ID or asset_id != STONE_ASSET or layer != 'visual.floor' or not P.number(position[0], 12.0, 12.0) or not P.number(position[1], 0.0, 0.0) or not P.number(position[2], 8.0, 8.0):
				return false
	return candidate_count == 1

func _valid_projection(projection) -> bool:
	if not projection is RefCounted or projection.get_script() != ProjectionScript:
		return false
	return P.number(projection.get('ppm'), 0.000001, 256.0)

func _valid_context(snapshot, scene, current_key, projection) -> bool:
	if not snapshot is Dictionary or not P.snapshot(snapshot) or snapshot.get('worldId') != WORLD_ID or snapshot.get('sceneId') != SCENE_ID or not P.counter(snapshot.get('worldEpoch'), 1):
		return false
	if typeof(current_key) != TYPE_STRING or current_key != WORLD_ID + ':' + SCENE_ID + ':' + str(snapshot.worldEpoch):
		return false
	if not scene is Dictionary or not P.counter(scene.get('schemaVersion'), 1, 1) or scene.get('worldId') != WORLD_ID or scene.get('sceneId') != SCENE_ID:
		return false
	return _valid_bounds(scene) and _valid_walkable(scene) and _valid_authored_floor(scene) and _valid_projection(projection)

func _project_point(projection, x: float, z: float):
	var point = projection.call('project', Vector3(x, 0.0, z))
	return point if _finite_vector2(point) else null

func _cell_id(row: int, column: int) -> String:
	return 'mh_fog_pier_ground_%02d_%02d' % [row, column]

func _build_cells(projection) -> Array:
	var cells: Array = []
	for row in range(ROW_COUNT):
		var start_x = 0.5 if row % 2 == 0 else -0.5
		var z1 = 0.5 + float(row)
		var z2 = z1 + 1.0
		for column in range(COLUMN_COUNT):
			var x1 = maxf(0.5, start_x + 2.0 * float(column))
			var x2 = minf(23.5, start_x + 2.0 * float(column) + 2.0)
			if x2 <= x1:
				return []
			var points: Array = []
			for corner in [Vector2(x1, z1), Vector2(x2, z1), Vector2(x2, z2), Vector2(x1, z2)]:
				var screen = _project_point(projection, corner.x, corner.y)
				if screen == null:
					return []
				points.append(screen)
			var screen_min = points[0]
			var screen_max = points[0]
			for point in points:
				screen_min = screen_min.min(point)
				screen_max = screen_max.max(point)
			var seams: Array = []
			if z1 > 0.5:
				seams.append({'a': points[0], 'b': points[1]})
			if x1 > 0.5:
				seams.append({'a': points[0], 'b': points[3]})
			cells.append({
				'id': _cell_id(row, column),
				'row': row,
				'column': column,
				'worldRect': Rect2(Vector2(x1, z1), Vector2(x2 - x1, z2 - z1)),
				'polygon': points,
				'screenBounds': Rect2(screen_min, screen_max - screen_min),
				'fill': FILL_COLORS[(column + 2 * row) % FILL_COLORS.size()],
				'seams': seams
			})
	return cells if cells.size() == CELL_COUNT else []

func build_packet(snapshot, scene, current_key, projection) -> Dictionary:
	if not _valid_context(snapshot, scene, current_key, projection):
		return {}
	var ppm = float(projection.get('ppm'))
	var cells = _build_cells(projection)
	if cells.size() != CELL_COUNT:
		return {}
	return {
		'kind': PACKET_KIND,
		'worldId': WORLD_ID,
		'sceneId': SCENE_ID,
		'worldEpoch': snapshot.worldEpoch,
		'currentKey': current_key,
		'ppm': ppm,
		'seamColor': SEAM_COLOR,
		'seamAlpha': SEAM_ALPHA,
		'seamWidth': clampf(0.7 * ppm / 48.0, 0.35, 1.0),
		'cells': cells
	}

func is_packet_valid(packet, snapshot, scene, current_key, projection) -> bool:
	if not _valid_context(snapshot, scene, current_key, projection):
		return false
	if not packet is Dictionary or packet.size() != 10 or packet.get('kind') != PACKET_KIND or packet.get('worldId') != WORLD_ID or packet.get('sceneId') != SCENE_ID or packet.get('worldEpoch') != snapshot.worldEpoch or packet.get('currentKey') != current_key:
		return false
	var ppm = float(projection.get('ppm'))
	if not P.number(packet.get('ppm'), ppm, ppm) or packet.get('seamColor') != SEAM_COLOR or not P.number(packet.get('seamAlpha'), SEAM_ALPHA, SEAM_ALPHA) or not P.number(packet.get('seamWidth'), clampf(0.7 * ppm / 48.0, 0.35, 1.0), clampf(0.7 * ppm / 48.0, 0.35, 1.0)):
		return false
	var cells = packet.get('cells')
	if not cells is Array or cells.size() != CELL_COUNT:
		return false
	var expected = _build_cells(projection)
	if expected.size() != CELL_COUNT:
		return false
	for index in range(CELL_COUNT):
		var cell = cells[index]
		var wanted = expected[index]
		if not cell is Dictionary or cell.size() != 8 or typeof(cell.get('id')) != TYPE_STRING or not P.counter(cell.get('row'), 0, ROW_COUNT - 1) or not P.counter(cell.get('column'), 0, COLUMN_COUNT - 1):
			return false
		if not _finite_rect2(cell.get('worldRect'), true) or not cell.get('polygon') is Array or cell.polygon.size() != 4 or not _finite_rect2(cell.get('screenBounds'), true) or not _finite_color(cell.get('fill')) or not cell.get('seams') is Array:
			return false
		for point in cell.polygon:
			if not _finite_vector2(point):
				return false
		for seam in cell.seams:
			if not seam is Dictionary or seam.size() != 2 or not _finite_vector2(seam.get('a')) or not _finite_vector2(seam.get('b')):
				return false
		if cell != wanted:
			return false
	return true

func draw_packet(canvas, packet, snapshot, scene, current_key, projection, viewport: Rect2) -> bool:
	if not canvas is CanvasItem:
		return false
	if not _finite_rect2(viewport, true) or not is_packet_valid(packet, snapshot, scene, current_key, projection):
		return false
	var seam_width = float(packet.seamWidth)
	var visible_cells: Array = []
	for cell in packet.cells:
		var bounds: Rect2 = cell.screenBounds
		if bounds.grow(seam_width).intersects(viewport):
			visible_cells.append(cell)
	for cell in visible_cells:
		canvas.draw_colored_polygon(PackedVector2Array(cell.polygon), cell.fill)
	var seam = packet.seamColor
	seam.a = float(packet.seamAlpha)
	for cell in visible_cells:
		for segment in cell.seams:
			canvas.draw_line(segment.a, segment.b, seam, seam_width, true)
	return true
