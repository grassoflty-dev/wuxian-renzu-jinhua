extends Control
const P = preload('res://scripts/bridge/protocol.gd')
const V = preload('res://scripts/world/presentation.gd')
var snapshot: Dictionary = {}
var text_scale = 1.0
var polygons: Array = []
var points: Array = []
var connections: Array = []
var range_rect = Rect2()
var message = '地图权限尚未授予'
func set_snapshot(s: Dictionary) -> void:
	snapshot = s
	polygons.clear()
	points.clear()
	connections.clear()
	message = '地图权限尚未授予'
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
				polygons.append({'p':outline,'color':Color('#436679')})
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
					points.append({'p':Vector2(o.positionM.xM,o.positionM.zM),'label':o.objectiveId})
	var knowledge = c.get('mapKnowledge')
	if knowledge is Dictionary:
		if P.rows(knowledge.get('regions'),512):
			for region in knowledge.regions:
				if not region is Dictionary or not P.ident(region.get('id')) or not region.get('channel') in ['topology','terrain','connections','objectives','enemies','hazards','loot','npcs','secrets'] or not region.get('polygon') is Dictionary or not P.rows(region.polygon.get('vertices'),128):
					continue
				var p: Array = []
				for v in region.polygon.vertices:
					if not v is Dictionary or not P.number(v.get('x'),0,4096) or not P.number(v.get('y'),0,4096):
						p.clear()
						break
					p.append([v.x,v.y])
				if P.polygon(p):
					polygons.append({'p':p,'color':Color('#426563')})
		if P.rows(knowledge.get('features'),512):
			for feature in knowledge.features:
				var v = feature.get('position') if feature is Dictionary else null
				if feature is Dictionary and feature.get('channel') in ['topology','terrain','connections','objectives','enemies','hazards','loot','npcs','secrets'] and feature.get('knowledge') in ['explored_only','detected_only','known','full'] and v is Dictionary and P.number(v.get('x'),0,4096) and P.number(v.get('y'),0,4096):
					points.append({'p':Vector2(v.x,v.y),'label':str(feature.get('knowledge','已感知'))})
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
	lo = lo.min(Vector2(player.xM,player.zM))
	hi = hi.max(Vector2(player.xM,player.zM))
	range_rect = Rect2(lo,(hi-lo).max(Vector2.ONE))
	queue_redraw()
func _map(p: Vector2) -> Vector2:
	var scale = minf((size.x-40)/range_rect.size.x,(size.y-70)/range_rect.size.y)
	return Vector2(20,50)+(p-range_rect.position)*scale
func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO,size),Color('#101a22'))
	draw_string(ThemeDB.fallback_font,Vector2(16,25),message,HORIZONTAL_ALIGNMENT_LEFT,-1,int(18*text_scale),Color('#e7edf2'))
	if polygons.is_empty() and points.is_empty():
		return
	for item in polygons:
		var poly = PackedVector2Array()
		for p in item.p:
			poly.append(_map(Vector2(p[0],p[1])))
		draw_colored_polygon(poly,item.color)
	for line in connections:
		draw_line(_map(line.from),_map(line.to),Color('#7d9ca8'),2,true)
	for item in points:
		draw_circle(_map(item.p),4,Color('#d5c788'))
	var player = snapshot.player.transform.positionM
	draw_circle(_map(Vector2(player.xM,player.zM)),6,Color('#85dfd0'))
