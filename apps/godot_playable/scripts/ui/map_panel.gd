extends Control
const P = preload('res://scripts/bridge/protocol.gd')
const V = preload('res://scripts/world/presentation.gd')
const CHANNELS = ['topology','terrain','connections','objectives','enemies','hazards','loot','npcs','secrets']
const CHANNEL_TAGS = {'enemies':'E','hazards':'H','loot':'L','npcs':'N','secrets':'S'}
const CATEGORY_LABELS = {'objective':'目标','hazard':'风险信息','other':'其他信息'}
var text_scale = 1.0
var polygons: Array = []
var points: Array = []
var connections: Array = []
var range_rect = Rect2()
var player_point = Vector2.ZERO
var legend: Array = []
var accent = Color('#436679')
var message = '地图权限尚未授予'
func set_snapshot(s: Dictionary) -> void:
	polygons.clear()
	points.clear()
	connections.clear()
	legend.clear()
	range_rect = Rect2()
	player_point = Vector2.ZERO
	accent = Color('#436679')
	message = '地图权限尚未授予'
	if s.is_empty() or not P.context(s) or not s.get('capabilities') is Dictionary or not s.get('player') is Dictionary or not s.player.get('transform') is Dictionary or not P.point(s.player.transform.get('positionM')) or not P.rows(s.get('actors'),2048) or not P.rows(s.get('hazards'),2048):
		queue_redraw()
		return
	for field in ['actors','hazards']:
		if not P.unique(s[field],'entityId'):
			queue_redraw()
			return
		for row in s[field]:
			if typeof(row.get('active'))!=TYPE_BOOL or field=='actors' and typeof(row.get('entityType'))!=TYPE_STRING:
				queue_redraw()
				return
	accent = {'mist_harbor':Color('#426563'),'clockworks':Color('#796043')}.get(s.worldId,accent)
	var topology = P.granted(s,'information.local_map_i',true)
	var effective = s.capabilities.get('mapTopologyAuthorized')
	if s.capabilities.has('mapTopologyAuthorized'):
		topology = typeof(effective)==TYPE_BOOL and effective
	var knowledge_available = s.capabilities.get('mapKnowledge') is Dictionary
	if not topology and not V.mapping(s) and not knowledge_available:
		queue_redraw()
		return
	var c: Dictionary = s.capabilities
	var map = c.get('exploredMap')
	if topology and map is Dictionary and map.get('worldId')==s.worldId and P.point(map.get('playerPositionM')) and P.rows(map.get('rooms'),256) and P.unique(map.rooms,'roomId'):
		var centers = {}
		for room in map.rooms:
			if not P.rows(room.get('outlineM'),128):
				continue
			var outline: Array = []
			var center = Vector2.ZERO
			for vertex in room.outlineM:
				if not P.point(vertex):
					outline.clear()
					break
				outline.append([vertex.xM,vertex.zM])
				center += Vector2(vertex.xM,vertex.zM)
			if P.polygon(outline):
				polygons.append({'p':outline,'color':accent})
				centers[room.roomId] = center/outline.size()
		if P.rows(map.get('connections'),512) and P.unique(map.connections,'connectionId'):
			for link in map.connections:
				if link.get('fromRoomId')!=link.get('toRoomId') and centers.has(link.get('fromRoomId')) and centers.has(link.get('toRoomId')):
					connections.append({'from':centers[link.fromRoomId],'to':centers[link.toRoomId]})
		# Objectives are drawn only from the current authorized map projection, never disk geometry.
		var interference = false
		for a in s.actors:
			if a.active and a.entityType=='enemy.mist_harbor.signal_wraith' and (V.signal_positions(a,s).is_empty() or not a.signalPerception.precise):
				interference = true
		var signal_count = 0
		for h in s.hazards:
			if h.entityId=='mh_signal_interference_region':
				signal_count += 1
			if h.entityId=='mh_signal_interference_region' and h.active:
				interference = true
		if s.sceneId=='mh_signal_yard' and signal_count!=1:
			interference = true
		if not interference and P.rows(map.get('objectives'),256) and P.unique(map.objectives,'objectiveId'):
			for o in map.objectives:
				if o is Dictionary and P.ident(o.get('objectiveId')) and P.point(o.get('positionM')):
					points.append({'p':Vector2(o.positionM.xM,o.positionM.zM),'category':'objective','channel':'objectives','knowledge':'known'})
	var knowledge = c.get('mapKnowledge')
	if knowledge is Dictionary:
		if P.rows(knowledge.get('regions'),512):
			for region in knowledge.regions:
				if not region is Dictionary or not P.ident(region.get('id')) or not region.get('channel') in CHANNELS or not region.get('polygon') is Dictionary or not P.rows(region.polygon.get('vertices'),128):
					continue
				var p: Array = []
				for v in region.polygon.vertices:
					if not v is Dictionary or not P.number(v.get('x'),0,4096) or not P.number(v.get('y'),0,4096):
						p.clear()
						break
					p.append([v.x,v.y])
				if P.polygon(p):
					polygons.append({'p':p,'color':accent})
		if P.rows(knowledge.get('features'),512):
			for feature in knowledge.features:
				var v = feature.get('position') if feature is Dictionary else null
				if feature is Dictionary and feature.get('channel') in CHANNELS and feature.get('knowledge') in ['explored_only','detected_only','known','full'] and v is Dictionary and P.number(v.get('x'),0,4096) and P.number(v.get('y'),0,4096):
					points.append({'p':Vector2(v.x,v.y),'category':_category(feature.channel),'channel':feature.channel,'knowledge':feature.knowledge})
	if polygons.is_empty() and points.is_empty():
		message = '尚无已探索区域；不展示未授权拓扑'
		queue_redraw()
		return
	message = '当前权威地图知识 · 未知区域保持隐藏'
	var lo = Vector2(INF,INF)
	var hi = Vector2(-INF,-INF)
	for polygon in polygons:
		for p in polygon.p:
			lo = lo.min(Vector2(p[0],p[1]))
			hi = hi.max(Vector2(p[0],p[1]))
	for p in points:
		lo = lo.min(p.p)
		hi = hi.max(p.p)
	var player = s.player.transform.positionM
	player_point = Vector2(player.xM,player.zM)
	lo = lo.min(Vector2(player.xM,player.zM))
	hi = hi.max(Vector2(player.xM,player.zM))
	range_rect = Rect2(lo,(hi-lo).max(Vector2.ONE))
	for category in ['objective','hazard','other']:
		for item in points:
			if item.category==category:
				legend.append(category)
				break
	queue_redraw()
func _category(channel: String) -> String:
	if channel=='objectives':
		return 'objective'
	if channel in ['enemies','hazards']:
		return 'hazard'
	return 'other'
func _channel_tag(channel: String) -> String:
	return str(CHANNEL_TAGS.get(channel,''))
func _has_channel_tags() -> bool:
	for item in points:
		if item is Dictionary and not _channel_tag(str(item.get('channel',''))).is_empty():
			return true
	return false
func _fit(ui_scale: float) -> Dictionary:
	if not is_finite(size.x) or not is_finite(size.y) or not is_finite(range_rect.position.x) or not is_finite(range_rect.position.y) or not is_finite(range_rect.size.x) or not is_finite(range_rect.size.y) or range_rect.size.x<=0 or range_rect.size.y<=0:
		return {}
	var pad = 20*ui_scale
	var header = 48*ui_scale
	# Fixed generic legend rows are bounded independently of payload strings.
	var footer = (legend.size()+2)*22*ui_scale+pad+(44*ui_scale if _has_channel_tags() else 0.0)
	var area = Rect2(Vector2(pad,header),Vector2(size.x-2*pad,size.y-header-footer))
	if area.size.x<32*ui_scale or area.size.y<32*ui_scale:
		return {}
	var factor = minf(area.size.x/range_rect.size.x,area.size.y/range_rect.size.y)
	if not is_finite(factor) or factor<=0:
		return {}
	var origin = area.position+(area.size-range_rect.size*factor)*0.5
	if not is_finite(origin.x) or not is_finite(origin.y):
		return {}
	return {'factor':factor,'origin':origin,'legend_y':size.y-footer+22*ui_scale}
func _map(p: Vector2, fit: Dictionary) -> Vector2:
	return fit.origin+(p-range_rect.position)*fit.factor
func _marker(p: Vector2, category: String, hollow: bool, ui_scale: float) -> void:
	var radius = 5*ui_scale
	var color = {'objective':Color('#e4cd79'),'hazard':Color('#e69b83'),'other':Color('#acb5dd')}[category]
	var vertices = PackedVector2Array()
	if category=='objective':
		vertices = PackedVector2Array([p+Vector2(0,-radius),p+Vector2(radius,0),p+Vector2(0,radius),p+Vector2(-radius,0)])
	elif category=='hazard':
		vertices = PackedVector2Array([p+Vector2(0,-radius),p+Vector2(radius,radius),p+Vector2(-radius,radius)])
	else:
		vertices = PackedVector2Array([p+Vector2(-radius,-radius),p+Vector2(radius,-radius),p+Vector2(radius,radius),p+Vector2(-radius,radius)])
	if not hollow:
		draw_colored_polygon(vertices,color)
	vertices.append(vertices[0])
	draw_polyline(vertices,color,1.5*ui_scale,true)
func _draw() -> void:
	if not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0:
		return
	draw_rect(Rect2(Vector2.ZERO,size),Color('#101a22'))
	var ui_scale = text_scale if P.number(text_scale,1,1.5) else 1.0
	var font_size = int(18*ui_scale)
	var pad = 16*ui_scale
	if size.y>=30*ui_scale and size.x>2*pad:
		draw_string(ThemeDB.fallback_font,Vector2(pad,25*ui_scale),message,HORIZONTAL_ALIGNMENT_LEFT,size.x-2*pad,font_size,Color('#e7edf2'))
	if polygons.is_empty() and points.is_empty():
		return
	var fit = _fit(ui_scale)
	if fit.is_empty():
		return
	for item in polygons:
		var poly = PackedVector2Array()
		for p in item.p:
			poly.append(_map(Vector2(p[0],p[1]),fit))
		draw_colored_polygon(poly,item.color)
	for line in connections:
		draw_line(_map(line.from,fit),_map(line.to,fit),accent.lightened(0.4),2*ui_scale,true)
	for item in points:
		_marker(_map(item.p,fit),item.category,item.knowledge=='detected_only',ui_scale)
		var channel_tag = _channel_tag(str(item.get('channel','')))
		if not channel_tag.is_empty():
			var tag_font = ThemeDB.fallback_font
			var tag_font_size = int(12*ui_scale)
			var tag_width = tag_font.get_string_size(channel_tag,HORIZONTAL_ALIGNMENT_LEFT,-1,tag_font_size).x
			var tag_position = _map(item.p,fit)+Vector2(6,-6)*ui_scale
			var tag_minimum_y = 48*ui_scale+tag_font.get_ascent(tag_font_size)
			var tag_maximum_y = fit.legend_y-22*ui_scale-tag_font.get_descent(tag_font_size)
			tag_position.x = clampf(tag_position.x,20*ui_scale,maxf(20*ui_scale,size.x-20*ui_scale-tag_width))
			tag_position.y = clampf(tag_position.y,tag_minimum_y,tag_maximum_y)
			draw_string(tag_font,tag_position,channel_tag,HORIZONTAL_ALIGNMENT_LEFT,tag_width,tag_font_size,Color('#ffffff'))
	var player = _map(player_point,fit)
	draw_circle(player,7*ui_scale,Color('#85dfd0'),false,2*ui_scale,true)
	draw_circle(player,2*ui_scale,Color('#85dfd0'))
	var legend_y: float = fit.legend_y
	for category in legend:
		_marker(Vector2(24*ui_scale,legend_y-5*ui_scale),category,false,ui_scale)
		draw_string(ThemeDB.fallback_font,Vector2(40*ui_scale,legend_y),CATEGORY_LABELS[category],HORIZONTAL_ALIGNMENT_LEFT,maxf(0,size.x-56*ui_scale),int(14*ui_scale),Color('#e7edf2'))
		legend_y += 22*ui_scale
	if _has_channel_tags():
		draw_string(ThemeDB.fallback_font,Vector2(pad,legend_y),'E 敌人 · H 风险信息 · L 战利品',HORIZONTAL_ALIGNMENT_LEFT,maxf(0,size.x-56*ui_scale),int(14*ui_scale),Color('#e7edf2'))
		legend_y += 22*ui_scale
		draw_string(ThemeDB.fallback_font,Vector2(pad,legend_y),'N NPC · S 秘密',HORIZONTAL_ALIGNMENT_LEFT,maxf(0,size.x-56*ui_scale),int(14*ui_scale),Color('#e7edf2'))
		legend_y += 22*ui_scale
	draw_string(ThemeDB.fallback_font,Vector2(pad,legend_y),'青色环：当前位置',HORIZONTAL_ALIGNMENT_LEFT,maxf(0,size.x-2*pad),int(14*ui_scale),Color('#85dfd0'))
	legend_y += 22*ui_scale
	draw_string(ThemeDB.fallback_font,Vector2(pad,legend_y),'空心：仅感知 · 实心：已知信息',HORIZONTAL_ALIGNMENT_LEFT,maxf(0,size.x-2*pad),int(14*ui_scale),Color('#e7edf2'))
