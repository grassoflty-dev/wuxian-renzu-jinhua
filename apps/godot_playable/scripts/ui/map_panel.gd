extends Control
const P = preload('res://scripts/bridge/protocol.gd')
const V = preload('res://scripts/world/presentation.gd')
const CHANNELS = ['topology','terrain','connections','objectives','enemies','hazards','loot','npcs','secrets']
const CHANNEL_TAGS = {'enemies':'E','hazards':'H','loot':'L','npcs':'N','secrets':'S'}
const CATEGORY_LABELS = {'objective':'目标','hazard':'风险信息','other':'其他信息'}
const MAP_ATLAS_PATH = 'res://assets/ui/map/map_region_ui_local_r1.png'
const MAP_ATLAS_RECTS = [Rect2(64,64,290,290),Rect2(482,64,290,290),Rect2(844,204,380,40),Rect2(28,610,388,32),Rect2(520,520,214,214),Rect2(948,516,172,216),Rect2(80,916,280,244),Rect2(520,936,216,216),Rect2(904,916,256,256)]
var map_atlas: Texture2D
var map_atlas_attempted = false
var text_layout_key: Array = []
var text_layout: Dictionary = {}
var requested_map_height = -1.0
var text_scale = 1.0
var polygons: Array = []
var points: Array = []
var connections: Array = []
var junctions: Array = []
var vertical_link: Dictionary = {}
var height_reference: Dictionary = {}
var junction_replaced = {}
var range_rect = Rect2()
var player_point = Vector2.ZERO
var has_player_point = false
var legend: Array = []
var region_state_legend = false
var visible_regions = {}
var shroud_active = false
var accent = Color('#436679')
var message = '当前没有可显示的地图信息。'
func _map_polygon(v: Array) -> bool:
	if not P.rows(v,128) or v.size()<3:
		return false
	var area = 0.0
	for n in range(v.size()):
		var a = v[n]
		var b = v[(n+1)%v.size()]
		if not a is Array or a.size()!=2 or not b is Array or b.size()!=2 or not P.number(a[0],-4096,4096) or not P.number(a[1],-4096,4096) or not P.number(b[0],-4096,4096) or not P.number(b[1],-4096,4096):
			return false
		area += a[0]*b[1]-b[0]*a[1]
	return abs(area)>0.000001
func set_snapshot(s: Dictionary) -> void:
	polygons.clear()
	points.clear()
	connections.clear()
	junctions.clear()
	vertical_link.clear()
	height_reference.clear()
	junction_replaced.clear()
	legend.clear()
	region_state_legend = false
	visible_regions.clear()
	shroud_active = false
	range_rect = Rect2()
	player_point = Vector2.ZERO
	has_player_point = false
	accent = Color('#436679')
	message = '当前没有可显示的地图信息。'
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
	var current_ids = _validated_room_ids(c.get('mapCurrentRoomIds'))
	var explored_ids = _validated_room_ids(c.get('mapExploredRoomIds'))
	# Region-level visibility is authored by the authority for the dense hive
	# scenes; the map only mirrors it. No field means no shroud at all, so every
	# scene without region data keeps the previous rendering byte for byte.
	visible_regions = _validated_room_ids(s.get('currentVisibleRegions'))
	shroud_active = not visible_regions.is_empty()
	var region_ids = {}
	if topology and map is Dictionary and map.get('worldId')==s.worldId and P.point(map.get('playerPositionM')) and P.rows(map.get('rooms'),256) and P.unique(map.rooms,'roomId'):
		player_point = Vector2(map.playerPositionM.xM,map.playerPositionM.zM)
		has_player_point = true
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
			if _map_polygon(outline):
				var state = 'current' if current_ids.has(room.roomId) else ('visible' if _region_visible(room.roomId) else ('explored' if explored_ids.has(room.roomId) else 'neutral'))
				polygons.append({'p':outline,'color':accent,'id':room.roomId,'state':state})
				if state!='neutral': region_state_legend = true
				region_ids[room.roomId] = true
				centers[room.roomId] = center/outline.size()
		if P.rows(map.get('connections'),512) and P.unique(map.connections,'connectionId'):
			for link in map.connections:
				if link.get('fromRoomId')!=link.get('toRoomId') and centers.has(link.get('fromRoomId')) and centers.has(link.get('toRoomId')):
					connections.append({'id':link.connectionId,'from':centers[link.fromRoomId],'to':centers[link.toRoomId],'from_id':link.fromRoomId,'to_id':link.toRoomId})
		var known = c.get('mapKnownJunctions')
		if P.rows(known,64):
			var pairs = {}
			for entry in known:
				if not entry is Dictionary or not P.ident(entry.get('fromRoomId')) or not P.ident(entry.get('toRoomId')) or entry.fromRoomId==entry.toRoomId or not centers.has(entry.fromRoomId) or not centers.has(entry.toRoomId) or not P.number(entry.get('x'),-4096,4096) or not P.number(entry.get('z'),-4096,4096): continue
				var ids = [entry.fromRoomId,entry.toRoomId]
				ids.sort()
				var key = ids[0]+'|'+ids[1]
				if pairs.has(key): continue
				pairs[key] = true
				junctions.append(Vector2(entry.x,entry.z))
				if P.rows(entry.get('replacedConnectionIds'),512):
					for id in entry.replacedConnectionIds:
						if not P.ident(id): continue
						for line in connections:
							if line.id==id and ((line.from_id==entry.fromRoomId and line.to_id==entry.toRoomId) or (line.from_id==entry.toRoomId and line.to_id==entry.fromRoomId)):
								junction_replaced[id]=true
		# This relation is independent of flat junctions and never contributes
		# the other floor to the visible map bounds.
		var links=c.get('mapKnownVerticalLinks')
		var lift=s.get('shaftLift')
		var support=s.get('supportScene')
		if P.granted(s,'information.local_map_i',true) and s.worldId=='grey_hive' and s.sceneId=='gh_central_shaft' and P.rows(links,1) and links.size()==1 and s.get('shaftLiftDeferred')==false and lift is Dictionary and support is Dictionary and P.context(lift) and P.same(lift,s) and P.context(support) and P.same(support,s) and support.get('rider') is Dictionary and support.rider.get('mode')=='floor' and P.point(support.rider.get('positionM')) and support.rider.positionM==s.player.transform.positionM and P.counter(s.get('serverTick')) and P.counter(s.get('authorityRevision')) and P.counter(s.get('serverTimeMs')) and lift.get('serverTick')==s.serverTick and support.get('serverTick')==s.serverTick and lift.get('authorityRevision')==s.authorityRevision and support.get('authorityRevision')==s.authorityRevision and lift.get('serverTimeMs')==s.serverTimeMs and support.get('serverTimeMs')==s.serverTimeMs:
			var link=links[0]
			var upper='grey_hive.gh_central_shaft.upper_active_info_v1.north_service:part3'
			var lower='grey_hive.gh_central_shaft.lower_info_v1.front_landing_corridor:part0'
			var sha='6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832'
			if link is Dictionary and link.size()==7 and link.get('supportId')=='gh_shaft_two_stop_lift' and link.get('sceneSourceSha256')==sha and lift.get('sceneSourceSha256')==sha and support.get('sceneSourceSha256')==sha and P.number(link.get('x')) and P.number(link.get('z')) and centers.has(link.get('currentRoomId')):
				var is_upper=link.get('currentLayerId')=='upper_y0' and link.get('currentRoomId') in [upper,'grey_hive.gh_central_shaft.upper_active_info_v1.north_service:part6'] and link.get('otherRoomId')==lower and float(support.rider.positionM.yM)==0.0 and is_equal_approx(float(link.x),43.5) and is_equal_approx(float(link.z),4.15)
				var is_lower=link.get('currentLayerId')=='lower_y_minus4' and link.get('currentRoomId') in [lower,'grey_hive.gh_central_shaft.lower_info_v1.front_landing_corridor:part1'] and link.get('otherRoomId')==upper and float(support.rider.positionM.yM)==-4.0 and is_equal_approx(float(link.x),3.5) and is_equal_approx(float(link.z),5.15)
				if is_upper or is_lower: vertical_link={'p':Vector2(link.x,link.z),'upper':is_upper}
		var height_refs=c.get('mapKnownHeightReferences')
		if P.granted(s,'information.local_map_i',true) and s.worldId=='clockworks' and P.rows(height_refs,1) and height_refs.size()==1:
			var ref=height_refs[0]
			if ref is Dictionary and ref.size()==6 and P.number(ref.get('x')) and P.number(ref.get('z')) and centers.has(ref.get('currentRoomId')) and explored_ids.has(ref.get('currentRoomId')):
				var forward=s.sceneId=='cw_gear_shaft' and ref.get('kind')=='gear_to_furnace' and ref.get('sceneSourceSha256')=='91afdb035c3df824806884c49d1e8f71034ff8f51b7bf90280bf71594896fea9' and ref.get('frameId')=='cw_early_v1' and ref.get('currentRoomId')=='mhcw_v1:clockworks:cw_gear_shaft:upper_static:cw_gear_furnace_landing:part0' and is_equal_approx(float(ref.x),119.0) and is_equal_approx(float(ref.z),8.0)
				var back=s.sceneId=='cw_furnace_heart' and ref.get('kind')=='furnace_return_to_gear_ground' and ref.get('sceneSourceSha256')=='f16f3941767aed71023cd8b9ce75642345a103eabf5c77682e358c96c2e219d5' and ref.get('frameId')=='cw_later_base_v1' and ref.get('currentRoomId')=='mhcw_v1:clockworks:cw_furnace_heart:ground:facility_floor:part0' and is_equal_approx(float(ref.x),1.5) and is_equal_approx(float(ref.z),8.0)
				if forward or back: height_reference={'p':Vector2(ref.x,ref.z),'forward':forward}
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
				if not region is Dictionary or not P.ident(region.get('id')) or region_ids.has(region.get('id')) or not region.get('channel') in CHANNELS or not region.get('polygon') is Dictionary or not P.rows(region.polygon.get('vertices'),128):
					continue
				var p: Array = []
				for v in region.polygon.vertices:
					if not v is Dictionary or not P.number(v.get('x'),-4096,4096) or not P.number(v.get('y'),-4096,4096):
						p.clear()
						break
					p.append([v.x,v.y])
				if _map_polygon(p):
					polygons.append({'p':p,'color':accent,'id':region.id,'state':'neutral'})
					region_ids[region.id] = true
		if P.rows(knowledge.get('features'),512):
			for feature in knowledge.features:
				var v = feature.get('position') if feature is Dictionary else null
				if feature is Dictionary and feature.get('channel') in CHANNELS and feature.get('knowledge') in ['explored_only','detected_only','known','full'] and v is Dictionary and P.number(v.get('x'),-4096,4096) and P.number(v.get('y'),-4096,4096):
					points.append({'p':Vector2(v.x,v.y),'category':_category(feature.channel),'channel':feature.channel,'knowledge':feature.knowledge})
	if polygons.is_empty() and points.is_empty():
		queue_redraw()
		return
	message = '本世界地图 · 未知区域隐藏'
	if not vertical_link.is_empty(): message += ' · 当前图层：上层' if vertical_link.upper else ' · 当前图层：下层'
	var lo = Vector2(INF,INF)
	var hi = Vector2(-INF,-INF)
	for polygon in polygons:
		for p in polygon.p:
			lo = lo.min(Vector2(p[0],p[1]))
			hi = hi.max(Vector2(p[0],p[1]))
	for p in points:
		lo = lo.min(p.p)
		hi = hi.max(p.p)
	if has_player_point:
		lo = lo.min(player_point)
		hi = hi.max(player_point)
	range_rect = Rect2(lo,(hi-lo).max(Vector2.ONE))
	for category in ['objective','hazard','other']:
		for item in points:
			if item.category==category:
				legend.append(category)
				break
	queue_redraw()
func _validated_room_ids(value) -> Dictionary:
	var result = {}
	if not P.rows(value,256): return result
	for id in value:
		if not P.ident(id) or result.has(id): return {}
		result[id] = true
	return result
func _region_visible(room_id: String) -> bool:
	if room_id.is_empty() or visible_regions.is_empty():
		return false
	if visible_regions.has(room_id):
		return true
	for region in visible_regions:
		if room_id.begins_with(str(region)+':'):
			return true
	return false
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
func _apply_map_minimum_height() -> void:
	if requested_map_height>0.0 and not is_equal_approx(custom_minimum_size.y,requested_map_height):
		custom_minimum_size.y = requested_map_height

func _layout_text(ui_scale: float, atlas_mode: bool) -> Dictionary:
	var font = ThemeDB.fallback_font
	var pad = 20.0*ui_scale
	var width = size.x-2.0*pad
	var gutter = 28.0*ui_scale
	var row_width = width-gutter
	if font==null or not is_finite(width) or row_width<=0.0: return {}
	var entries: Array[String] = []
	if not atlas_mode: entries.append('地图简化显示：当前与历史、特殊连接的样式未显示')
	if not polygons.is_empty() or not points.is_empty():
		if has_player_point: entries.append('你在这里（当前图层）')
		if atlas_mode and region_state_legend: entries.append('角标+粗边：当前可见 · 斜纹：已知或此前探索')
		if atlas_mode and shroud_active and region_state_legend: entries.append('亮边：当前视野内 · 暗纱：视野之外')
		var detected = false
		var known = false
		var channels = {}
		for item in points:
			if item.knowledge=='detected_only': detected = true
			else: known = true
			if CHANNEL_TAGS.has(item.channel): channels[item.channel] = true
		if detected: entries.append('空心：仅感知')
		if known: entries.append('实心：已知信息')
		if atlas_mode and not junctions.is_empty(): entries.append('连接符：已知连接，通行以现场为准')
		if atlas_mode and not vertical_link.is_empty(): entries.append(('升降连接：下层（已知）' if vertical_link.upper else '升降连接：上层（已知）')+' · 通行以现场为准')
		if atlas_mode and not height_reference.is_empty(): entries.append(('高差连接（已知）' if height_reference.forward else '返回连接（已知）')+' · 通行以现场为准')
		for category in legend: entries.append(CATEGORY_LABELS[category])
		for pair in [['enemies','E 敌人'],['hazards','H 风险信息'],['loot','L 战利品'],['npcs','N NPC'],['secrets','S 秘密']]:
			if channels.has(pair[0]): entries.append(pair[1])
	var key = [size.x,ui_scale,atlas_mode,message,entries]
	if key==text_layout_key: return text_layout
	var flags = TextServer.BREAK_MANDATORY|TextServer.BREAK_WORD_BOUND|TextServer.BREAK_GRAPHEME_BOUND
	var header_size = int(18*ui_scale)
	var footer_size = int(14*ui_scale)
	var header_h = font.get_multiline_string_size(message,HORIZONTAL_ALIGNMENT_LEFT,width,header_size,-1,flags).y
	var rows: Array = []
	var footer_h = 0.0
	for entry in entries:
		var h = font.get_multiline_string_size(entry,HORIZONTAL_ALIGNMENT_LEFT,row_width,footer_size,-1,flags).y
		var sample = 'player' if entry=='你在这里（当前图层）' else ''
		for category in legend:
			if entry==CATEGORY_LABELS[category]: sample = category
		rows.append({'text':entry,'height':h,'sample':sample})
		footer_h += h+4.0*ui_scale
	text_layout_key = key.duplicate(true)
	text_layout = {'pad':pad,'width':width,'gutter':gutter,'row_width':row_width,'header':header_h,'rows':rows,'footer':footer_h,'flags':flags,'atlas':atlas_mode}
	var needed = maxf(440.0*ui_scale,2.0*pad+header_h+160.0*ui_scale+footer_h)
	if not is_equal_approx(requested_map_height,needed):
		requested_map_height = needed
		call_deferred('_apply_map_minimum_height')
	return text_layout

func _draw_map_text(layout: Dictionary, ui_scale: float) -> void:
	var font = ThemeDB.fallback_font
	var pad: float = layout.pad
	if size.y>=pad+layout.header:
		draw_multiline_string(font,Vector2(pad,pad+font.get_ascent(int(18*ui_scale))),message,HORIZONTAL_ALIGNMENT_LEFT,layout.width,int(18*ui_scale),-1,Color('#e7edf2'),layout.flags)
	if size.x-2.0*pad<32.0*ui_scale or size.y<2.0*pad+layout.header+160.0*ui_scale+layout.footer: return
	var y: float = size.y-pad-layout.footer
	for row in layout.rows:
		var sample_point = Vector2(pad+layout.gutter*0.5,y+minf(row.height,14.0*ui_scale)*0.5)
		if row.sample=='player':
			if layout.atlas: _icon(sample_point,4,6*ui_scale)
			else:
				draw_circle(sample_point,5*ui_scale,Color('#85dfd0'),false,2*ui_scale,true)
				draw_circle(sample_point,2*ui_scale,Color('#85dfd0'))
		elif not row.sample.is_empty():
			if layout.atlas: _marker(sample_point,row.sample,false,ui_scale)
			else: _legacy_marker(sample_point,row.sample,false,ui_scale)
		draw_multiline_string(font,Vector2(pad+layout.gutter,y+font.get_ascent(int(14*ui_scale))),row.text,HORIZONTAL_ALIGNMENT_LEFT,layout.row_width,int(14*ui_scale),-1,Color('#e7edf2'),layout.flags)
		y += row.height+4.0*ui_scale

func _fit(ui_scale: float, layout: Dictionary) -> Dictionary:
	if not is_finite(size.x) or not is_finite(size.y) or not is_finite(range_rect.position.x) or not is_finite(range_rect.position.y) or not is_finite(range_rect.size.x) or not is_finite(range_rect.size.y) or range_rect.size.x<=0 or range_rect.size.y<=0:
		return {}
	var pad = 20*ui_scale
	var header = pad+layout.header+pad
	var footer = layout.footer+pad
	var area = Rect2(Vector2(pad,header),Vector2(size.x-2*pad,size.y-header-footer))
	if has_player_point: area = area.grow(-13.0*ui_scale)
	if area.size.x<32*ui_scale or area.size.y<32*ui_scale:
		return {}
	var factor = minf(area.size.x/range_rect.size.x,area.size.y/range_rect.size.y)
	if not is_finite(factor) or factor<=0:
		return {}
	var origin = area.position+(area.size-range_rect.size*factor)*0.5
	if not is_finite(origin.x) or not is_finite(origin.y):
		return {}
	return {'factor':factor,'origin':origin,'legend_y':size.y-footer,'header_y':header}
func _map(p: Vector2, fit: Dictionary) -> Vector2:
	return fit.origin+(p-range_rect.position)*fit.factor
func _atlas_cell(index: int) -> Rect2:
	return MAP_ATLAS_RECTS[index]
func _icon(p: Vector2, index: int, extent: float) -> void:
	var source = _atlas_cell(index)
	var icon_size = source.size*(2.0*extent/maxf(source.size.x,source.size.y))
	draw_texture_rect_region(map_atlas,Rect2(p-icon_size*0.5,icon_size),source,Color.WHITE)
func _strip(a: Vector2, b: Vector2, index: int, width: float, opacity: float = 1.0) -> void:
	var d = b-a
	if d.length_squared()<=0.000001:
		return
	var n = Vector2(-d.y,d.x).normalized()*width*0.5
	var rect = _atlas_cell(index)
	var sz = map_atlas.get_size()
	var uv = PackedVector2Array([rect.position/sz,Vector2(rect.end.x,rect.position.y)/sz,rect.end/sz,Vector2(rect.position.x,rect.end.y)/sz])
	draw_polygon(PackedVector2Array([a-n,b-n,b+n,a+n]),PackedColorArray([Color(1,1,1,opacity)]),uv,map_atlas)
func _hatch_polygon(poly: PackedVector2Array, color: Color, spacing: float, width: float) -> void:
	if poly.size()<3 or not is_finite(spacing) or spacing<=0.0:
		return
	var lo = poly[0]
	var hi = poly[0]
	for p in poly:
		lo = lo.min(p)
		hi = hi.max(p)
	var step = spacing*sqrt(2.0)
	var offset = lo.x+lo.y
	var offset_end = hi.x+hi.y
	while offset<=offset_end:
		# Even-odd spans of the diagonal x+y=offset, clipped to the room polygon.
		var xs: Array[float] = []
		for i in range(poly.size()):
			var a = poly[i]
			var b = poly[(i+1)%poly.size()]
			var fa = a.x+a.y-offset
			var fb = b.x+b.y-offset
			if (fa<=0.0 and fb>0.0) or (fb<=0.0 and fa>0.0):
				xs.append(a.x+(b.x-a.x)*(fa/(fa-fb)))
		xs.sort()
		var k = 0
		while k+1<xs.size():
			draw_line(Vector2(xs[k],offset-xs[k]),Vector2(xs[k+1],offset-xs[k+1]),color,width,true)
			k += 2
		offset += step
func _marker(p: Vector2, category: String, hollow: bool, ui_scale: float) -> void:
	var index: int = 8 if hollow else {'objective':5,'hazard':6,'other':7}[category]
	_icon(p,index,7*ui_scale)
func _draw() -> void:
	if not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0:
		return
	if not map_atlas_attempted:
		map_atlas_attempted = true
		if ResourceLoader.exists(MAP_ATLAS_PATH):
			map_atlas = load(MAP_ATLAS_PATH) as Texture2D
			if map_atlas!=null:
				var image: Image = map_atlas.get_image()
				if image==null or image.get_format()!=Image.FORMAT_RGBA8 or image.get_width()!=1254 or image.get_height()!=1254 or map_atlas.get_width()!=1254 or map_atlas.get_height()!=1254:
					map_atlas = null
		if map_atlas==null:
			var original = Image.new()
			if original.load(MAP_ATLAS_PATH)==OK and original.get_width()==1254 and original.get_height()==1254 and original.get_format()==Image.FORMAT_RGBA8:
				map_atlas = ImageTexture.create_from_image(original)
	if map_atlas==null or map_atlas.get_width()!=1254 or map_atlas.get_height()!=1254:
		_draw_legacy()
		return
	draw_texture_rect_region(map_atlas,Rect2(Vector2.ZERO,size),_atlas_cell(0),Color.WHITE)
	var ui_scale = text_scale if P.number(text_scale,1,1.5) else 1.0
	var layout = _layout_text(ui_scale,true)
	if layout.is_empty(): return
	_draw_map_text(layout,ui_scale)
	if polygons.is_empty() and points.is_empty():
		return
	var fit = _fit(ui_scale,layout)
	if fit.is_empty():
		return
	for item in polygons:
		var poly = PackedVector2Array()
		var uv = PackedVector2Array()
		var fill = _atlas_cell(1)
		for p in item.p:
			var position = Vector2(p[0],p[1])
			poly.append(_map(position,fit))
			uv.append((fill.position+(position-range_rect.position)/range_rect.size*fill.size)/map_atlas.get_size())
		# Three tiers, highest first: the current room, rooms inside the
		# authority's current visible regions, then everything merely known.
		# Explored and neutral rooms share one dimmed, hatched treatment so a
		# known-but-not-current room never reads as more visible than an explored one.
		var tier = str(item.get('state','neutral'))
		var dim = tier!='current' and tier!='visible'
		var opacity = 1.0 if tier=='current' else (0.78 if tier=='visible' else 0.45)
		draw_polygon(poly,PackedColorArray([Color(1,1,1,opacity)]),uv,map_atlas)
		for i in range(poly.size()):
			_strip(poly[i],poly[(i+1)%poly.size()],2,(1.5 if dim else 2.0)*ui_scale,opacity)
		if dim:
			_hatch_polygon(poly,Color(0.90,0.94,0.97,0.30),8.0*ui_scale,1.0*ui_scale)
			# Per-room shroud: a veil clipped to the room outline, never a global
			# fade, and only when the snapshot actually carries region data.
			if shroud_active:
				draw_colored_polygon(poly,Color(0.02,0.035,0.05,0.32))
	# Current boundaries are drawn last so shared edges retain their meaning.
	for item in polygons:
		if item.state!='current': continue
		for i in range(item.p.size()):
			_strip(_map(Vector2(item.p[i][0],item.p[i][1]),fit),_map(Vector2(item.p[(i+1)%item.p.size()][0],item.p[(i+1)%item.p.size()][1]),fit),2,3*ui_scale)
		# A corner bracket marks the room shape itself, independent of outline weight.
		var corner = Vector2(INF,INF)
		for p in item.p:
			var mapped = _map(Vector2(p[0],p[1]),fit)
			if mapped.x+mapped.y<corner.x+corner.y: corner = mapped
		if not is_finite(corner.x) or not is_finite(corner.y): continue
		var arm = 9.0*ui_scale
		draw_line(corner,corner+Vector2(arm,0),Color(0.93,0.97,1.0,0.95),2.5*ui_scale,true)
		draw_line(corner,corner+Vector2(0,arm),Color(0.93,0.97,1.0,0.95),2.5*ui_scale,true)
	for line in connections:
		if junction_replaced.has(line.id): continue
		_strip(_map(line.from,fit),_map(line.to,fit),3,3*ui_scale)
	for junction in junctions:
		var p = _map(junction,fit)
		var a = p-Vector2(5,0)*ui_scale
		var b = p+Vector2(5,0)*ui_scale
		_strip(a,b,3,2*ui_scale,0.45)
		_strip(a-Vector2(0,3)*ui_scale,a+Vector2(0,3)*ui_scale,2,1.5*ui_scale,0.45)
		_strip(b-Vector2(0,3)*ui_scale,b+Vector2(0,3)*ui_scale,2,1.5*ui_scale,0.45)
	if not vertical_link.is_empty():
		var p=_map(vertical_link.p,fit)
		var a=p-Vector2(5,0)*ui_scale
		var b=p+Vector2(5,0)*ui_scale
		_strip(a,b,3,2*ui_scale,0.45)
		_strip(a-Vector2(0,3)*ui_scale,a+Vector2(0,3)*ui_scale,2,1.5*ui_scale,0.45)
		_strip(b-Vector2(0,3)*ui_scale,b+Vector2(0,3)*ui_scale,2,1.5*ui_scale,0.45)
	if not height_reference.is_empty():
		var p=_map(height_reference.p,fit)
		var a=p-Vector2(5,0)*ui_scale
		var b=p+Vector2(5,0)*ui_scale
		_strip(a,b,3,2*ui_scale,0.45)
		_strip(a-Vector2(0,3)*ui_scale,a+Vector2(0,3)*ui_scale,2,1.5*ui_scale,0.45)
		_strip(b-Vector2(0,3)*ui_scale,b+Vector2(0,3)*ui_scale,2,1.5*ui_scale,0.45)
	for item in points:
		_marker(_map(item.p,fit),item.category,item.knowledge=='detected_only',ui_scale)
		var channel_tag = _channel_tag(str(item.get('channel','')))
		if not channel_tag.is_empty():
			var tag_font = ThemeDB.fallback_font
			var tag_font_size = int(12*ui_scale)
			var tag_width = tag_font.get_string_size(channel_tag,HORIZONTAL_ALIGNMENT_LEFT,-1,tag_font_size).x
			var tag_position = _map(item.p,fit)+Vector2(6,-6)*ui_scale
			var tag_minimum_y = fit.header_y+tag_font.get_ascent(tag_font_size)
			var tag_maximum_y = fit.legend_y-tag_font.get_descent(tag_font_size)
			tag_position.x = clampf(tag_position.x,20*ui_scale,maxf(20*ui_scale,size.x-20*ui_scale-tag_width))
			tag_position.y = clampf(tag_position.y,tag_minimum_y,tag_maximum_y)
			draw_string(tag_font,tag_position,channel_tag,HORIZONTAL_ALIGNMENT_LEFT,tag_width,tag_font_size,Color('#ffffff'))
	if has_player_point:
		var player = _map(player_point,fit)
		_icon(player,4,12*ui_scale)
func _legacy_marker(p: Vector2, category: String, hollow: bool, ui_scale: float) -> void:
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
func _draw_legacy() -> void:
	if not is_finite(size.x) or not is_finite(size.y) or size.x<=0 or size.y<=0:
		return
	draw_rect(Rect2(Vector2.ZERO,size),Color('#101a22'))
	var ui_scale = text_scale if P.number(text_scale,1,1.5) else 1.0
	var layout = _layout_text(ui_scale,false)
	if layout.is_empty(): return
	_draw_map_text(layout,ui_scale)
	if polygons.is_empty() and points.is_empty():
		return
	var fit = _fit(ui_scale,layout)
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
		_legacy_marker(_map(item.p,fit),item.category,item.knowledge=='detected_only',ui_scale)
		var channel_tag = _channel_tag(str(item.get('channel','')))
		if not channel_tag.is_empty():
			var tag_font = ThemeDB.fallback_font
			var tag_font_size = int(12*ui_scale)
			var tag_width = tag_font.get_string_size(channel_tag,HORIZONTAL_ALIGNMENT_LEFT,-1,tag_font_size).x
			var tag_position = _map(item.p,fit)+Vector2(6,-6)*ui_scale
			var tag_minimum_y = fit.header_y+tag_font.get_ascent(tag_font_size)
			var tag_maximum_y = fit.legend_y-tag_font.get_descent(tag_font_size)
			tag_position.x = clampf(tag_position.x,20*ui_scale,maxf(20*ui_scale,size.x-20*ui_scale-tag_width))
			tag_position.y = clampf(tag_position.y,tag_minimum_y,tag_maximum_y)
			draw_string(tag_font,tag_position,channel_tag,HORIZONTAL_ALIGNMENT_LEFT,tag_width,tag_font_size,Color('#ffffff'))
	if has_player_point:
		var player = _map(player_point,fit)
		draw_circle(player,12*ui_scale,Color('#85dfd0'),false,2*ui_scale,true)
		draw_circle(player,2*ui_scale,Color('#85dfd0'))
