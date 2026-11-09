extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const Catalog = preload('res://scripts/world/scene_catalog.gd')
const Content = preload('res://scripts/ui/content_model.gd')
const NEW_SHA = 'b32ad8d1204e9eaf45512b0e2d5ea22e08f5bd09ec0b5e31980e41336c45ee09'
const OLD_SHA = '6ab262467e1992feac94f27d8bf571dcf77ddbc8c1693a79c57b3aeebff0c54c'
const WALLS = ['exit_wall_north','exit_wall_south','exit_wall_west_north','exit_wall_west_south','exit_wall_east_north','exit_wall_east_south']
var catalogs: Dictionary = {}
var attempted := false
var pipe: Texture2D
var clutter: Texture2D

func _catalog(hash: String):
	if not hash in [NEW_SHA,OLD_SHA]: return null
	if not catalogs.has(hash):
		var c=Catalog.new()
		if not c.build('grey_hive','gh_exit',hash): return null
		catalogs[hash]=c
	return catalogs[hash]

func _coords(actual: Variant, expected: Array) -> bool:
	if not actual is Array or actual.size()!=expected.size(): return false
	for i in range(expected.size()):
		if expected[i] is Array:
			if not _coords(actual[i],expected[i]): return false
		elif not P.number(actual[i],float(expected[i]),float(expected[i])): return false
	return true

func _image(path: String, size: Vector2i) -> Image:
	if not ResourceLoader.exists(path,'Texture2D'): return null
	var t=ResourceLoader.load(path,'Texture2D')
	if not t is Texture2D or t.get_size()!=Vector2(size): return null
	var img=t.get_image()
	if img==null or img.is_empty() or img.get_size()!=size or img.get_format()!=Image.FORMAT_RGBA8: return null
	return img

func _load_props(legacy) -> bool:
	if not attempted:
		attempted=true
		var a=_image('res://assets/props/return_station/service_pipes.png',Vector2i(1414,1047))
		var b=_image('res://assets/props/grey_hive/clutter.png',Vector2i(1428,1061))
		if a==null or b==null: return false
		var ac=legacy._isolate_component(a,Rect2i(0,219,388,270),Vector2i(100,320),46171,-1)
		var bc=legacy._isolate_component(b,Rect2i(40,716,347,287),Vector2i(150,800),73359,168)
		if ac==null or bc==null: return false
		pipe=ImageTexture.create_from_image(ac);clutter=ImageTexture.create_from_image(bc)
	return pipe!=null and clutter!=null

func _prop(project: Callable, legacy, texture: Texture2D, id: String, a: Vector2, b: Vector2, wa: Vector3, wb: Vector3, pos: Vector3, shadow_radii: Vector2, shadow_alpha: float, max_rotation: float, max_extent: float, ppm: float) -> Dictionary:
	var sa=project.call(wa);var sb=project.call(wb)
	if not sa is Vector2 or not sb is Vector2 or not sa.is_finite() or not sb.is_finite() or a.distance_to(b)<=0.0: return {}
	var factor: float=sa.distance_to(sb)/a.distance_to(b)
	var rotation: float=(sb-sa).angle()-(b-a).angle()
	var origin: Vector2=sa-a.rotated(rotation)*factor
	var bounds: Rect2=legacy._transformed_bounds(origin,rotation,Vector2(factor,factor),Rect2(Vector2.ZERO,texture.get_size()))
	var shadow: PackedVector2Array=legacy._ellipse_points(project.get_object(),pos,shadow_radii,24)
	if not is_finite(factor) or factor<=0 or not is_finite(rotation) or absf(rotation)>max_rotation or not origin.is_finite() or not bounds.has_area() or maxf(bounds.size.x,bounds.size.y)>max_extent*ppm/48.0+0.01 or shadow.size()!=24: return {}
	for p in shadow:
		if not p.is_finite(): return {}
	return {'id':id,'p':pos,'texture':texture,'origin':origin,'rotation':rotation,'scale':Vector2(factor,factor),'shadow':shadow,'alpha':shadow_alpha}

func _edges(rects: Array, axis: String, limit: float) -> Array:
	var values: Array=[]
	var v:=0.0
	while v<=limit+0.00001:
		values.append(v)
		v+=0.5
	for row in rects:
		var r: Rect2=row.rect
		values.append(r.position.x if axis=='x' else r.position.y)
		values.append(r.end.x if axis=='x' else r.end.y)
	values.sort()
	var out: Array=[]
	for value in values:
		if out.is_empty() or absf(float(value)-float(out.back()))>0.00001: out.append(value)
	return out

func _owner(rects: Array, x: float, z: float) -> Dictionary:
	var selected: Dictionary={}
	for row in rects:
		var r: Rect2=row.rect
		if x>r.position.x and x<r.end.x and z>r.position.y and z<r.end.y and (selected.is_empty() or row.height>selected.height): selected=row
	return selected

func _piece(art, project: Callable, world: Array, uv: PackedVector2Array, id: String, anchor: Vector3) -> Dictionary:
	var result: Array=[]
	if not art._piece(result,project,world,uv,art._steel,{'owner':id,'anchor':anchor}) or result.size()!=1: return {}
	return result[0]

func prepare(snapshot: Dictionary, scene: Dictionary, key: String, hash: String, camera, art, legacy, terminal, crate, neighbor: Dictionary = {}) -> Dictionary:
	if not P.snapshot(snapshot) or snapshot.get('worldId')!='grey_hive' or not P.counter(snapshot.get('worldEpoch'),1): return {}
	if neighbor.is_empty():
		if snapshot.get('sceneId')!='gh_exit' or key!='grey_hive:gh_exit:'+str(snapshot.worldEpoch) or snapshot.get('utilitySceneSourceSha256')!=hash: return {}
	else:
		if snapshot.get('sceneId')!='gh_beacon' or key!='grey_hive:gh_beacon:'+str(snapshot.worldEpoch) or snapshot.get('beaconExitNeighbor')!=neighbor: return {}
		if neighbor.get('sourceSceneId')!='gh_beacon' or neighbor.get('targetSceneId')!='gh_exit' or neighbor.get('targetSceneSourceSha256')!=hash or neighbor.get('worldEpoch')!=snapshot.worldEpoch or neighbor.get('serverTick')!=snapshot.serverTick or neighbor.get('authorityRevision')!=snapshot.authorityRevision: return {}
	var token=snapshot.get('entryToken')
	if token!=null and (not P.token(token) or not P.same(token,snapshot)): return {}
	var catalog=_catalog(hash)
	if catalog==null or catalog.source_hash!=hash or catalog.current!=scene or not art._load_once() or not _load_props(legacy): return {}
	terminal._load_once();crate._load_once()
	if not terminal.textures.get('lockdown_terminal') is Texture2D or not crate.texture is Texture2D: return {}
	if not P.rows(scene.get('collision'),64) or scene.collision.size()!=(17 if hash==NEW_SHA else 7) or not P.unique(scene.collision,'id') or not P.rows(scene.get('interactions'),64) or not P.unique(scene.interactions,'id'): return {}
	var seals=scene.collision.filter(func(row): return row is Dictionary and row.get('id')=='exit_sealed_extraction_barrier')
	var consoles=scene.interactions.filter(func(row): return row is Dictionary and row.get('id')=='gh_exit_extraction_console')
	if seals.size()!=1 or seals[0].size()!=3 or seals[0].get('requiresEvent')!='hive_extraction' or not _coords(seals[0].get('polygon'),[[21.5,0.5],[22.5,0.5],[22.5,15.5],[21.5,15.5]]): return {}
	if consoles.size()!=1 or consoles[0].size()!=4 or consoles[0].get('kind')!='extraction_console' or consoles[0].get('event')!='hive_extraction' or not _coords(consoles[0].get('position'),[8,0,8]): return {}
	var progress=snapshot.get('progression')
	if not progress is Dictionary or progress.get('currentWorldId')!='grey_hive' or not P.rows(progress.get('worlds'),16) or not P.unique(progress.worlds,'worldId'): return {}
	var actual_progress=Content.progress(snapshot,'grey_hive')
	if actual_progress.is_empty() or not P.counter(actual_progress.get('visitId'),1): return {}
	var open: bool='hive_extraction' in actual_progress.completedEvents
	var sprites=scene.get('presentation',{}).get('sprites')
	if not P.rows(sprites,64) or not P.unique(sprites,'id'): return {}
	for spec in [['exit_service_pipes','runtime2d.grey_hive.pipes_cables.v1',[9,0,3],'visual.props_back'],['exit_approved_clutter','runtime2d.grey_hive.clutter.v1',[5,0,11.5],'visual.props_back'],['exit_extraction_console_approved','runtime2d.prop.lockdown_terminal.v1',[8,0,8],'visual.props_dynamic'],['exit_console_crates','runtime2d.prop.supply_crates.v1',[18,0,11.5],'visual.props_back']]:
		var rows=sprites.filter(func(row): return row is Dictionary and row.get('id')==spec[0])
		if rows.size()!=1 or rows[0].get('assetId')!=spec[1] or rows[0].get('layer')!=spec[3] or not _coords(rows[0].get('position'),spec[2]): return {}
	var project: Callable=Callable(camera,'project')
	var prop1=_prop(project,legacy,pipe,'exit_service_pipes',Vector2(50,126),Vector2(325,263),Vector3(8.5,0,3),Vector3(9.5,0,3),Vector3(9,0,3),Vector2(0.6,0.16),0.18,0.025,80,camera.ppm)
	var prop2=_prop(project,legacy,clutter,'exit_approved_clutter',Vector2(10,213),Vector2(329,182),Vector3(4.625,0,12),Vector3(5.375,0,11),Vector3(5,0,11.5),Vector2(0.5,0.55),0.16,0.06,96,camera.ppm)
	if prop1.is_empty() or prop2.is_empty(): return {}
	var grounding: Dictionary={}
	for spec in [['exit_extraction_console_approved',Vector3(8,0,8),Vector2(0.40,0.22)],['exit_console_crates',Vector3(18,0,11.5),Vector2(0.30,0.18)]]:
		var points: PackedVector2Array=legacy._ellipse_points(camera,spec[1],spec[2],24)
		if points.size()!=24: return {}
		for point in points:
			if not point.is_finite(): return {}
		grounding[spec[0]]=points
	var rects: Array=[]
	for row in scene.collision:
		if not row is Dictionary or not row.get('id') is String: return {}
		if row.id=='exit_sealed_extraction_barrier' and open: continue
		if row.id!='exit_sealed_extraction_barrier' and (row.get('requiresEvent')!=null or row.get('requiresActorFirstKill')!=null): return {}
		var r: Rect2=art._rect(row.get('polygon'))
		if not r.has_area(): return {}
		var height: float=0.75 if row.id=='exit_sealed_extraction_barrier' else (0.5 if row.id in WALLS else 0.25)
		rects.append({'id':row.id,'rect':r,'height':height})
	var floor: Array=[]
	if not art._floor_rect(floor,project,Rect2(0,0,24,16),0.0,{'role':'ground'}) or floor.is_empty(): return {}
	var xs=_edges(rects,'x',24.0);var zs=_edges(rects,'z',16.0)
	var pieces: Array=[];var seen: Dictionary={}
	for xi in range(xs.size()-1):
		for zi in range(zs.size()-1):
			var x0: float=xs[xi];var x1: float=xs[xi+1];var z0: float=zs[zi];var z1: float=zs[zi+1]
			var owner=_owner(rects,(x0+x1)*0.5,(z0+z1)*0.5)
			if owner.is_empty(): continue
			seen[owner.id]=true
			var h: float=owner.height;var anchor=Vector3((x0+x1)*0.5,0,(z0+z1)*0.5)
			var u0: float=fposmod(x0,0.5)/0.5;var u1: float=u0+(x1-x0)/0.5
			var v0: float=fposmod(z0,0.5)/0.5;var v1: float=v0+(z1-z0)/0.5
			var cap=_piece(art,project,[Vector3(x0,h,z0),Vector3(x1,h,z0),Vector3(x1,h,z1),Vector3(x0,h,z1)],PackedVector2Array([Vector2(u0,v0),Vector2(u1,v0),Vector2(u1,v1),Vector2(u0,v1)]),owner.id,anchor)
			if cap.is_empty(): return {}
			pieces.append(cap)
			var checks=[Vector2(x0-0.0001,(z0+z1)*0.5),Vector2(x1+0.0001,(z0+z1)*0.5),Vector2((x0+x1)*0.5,z0-0.0001),Vector2((x0+x1)*0.5,z1+0.0001)]
			var edges=[[Vector2(x0,z0),Vector2(x0,z1)],[Vector2(x1,z0),Vector2(x1,z1)],[Vector2(x0,z0),Vector2(x1,z0)],[Vector2(x0,z1),Vector2(x1,z1)]]
			for side in range(4):
				var adjacent=_owner(rects,checks[side].x,checks[side].y)
				var low: float=0.0 if adjacent.is_empty() else adjacent.height
				if low>=h: continue
				var a: Vector2=edges[side][0];var b: Vector2=edges[side][1]
				var start: float=a.y if side<2 else a.x;var finish: float=b.y if side<2 else b.x
				var f0: float=fposmod(start,0.5)/0.5;var f1: float=f0+(finish-start)/0.5
				while low<h-0.00001:
					var high: float=minf(h,(floor(low/0.5)+1.0)*0.5)
					if high<=low: return {}
					var vv0: float=1.0-fposmod(low,0.5)/0.5;var vv1: float=vv0-(high-low)/0.5
					var face=_piece(art,project,[Vector3(a.x,low,a.y),Vector3(b.x,low,b.y),Vector3(b.x,high,b.y),Vector3(a.x,high,a.y)],PackedVector2Array([Vector2(f0,vv0),Vector2(f1,vv0),Vector2(f1,vv1),Vector2(f0,vv1)]),owner.id,anchor)
					if face.is_empty(): return {}
					pieces.append(face);low=high
	if seen.size()!=rects.size() or pieces.is_empty(): return {}
	return {'floor':floor,'pieces':pieces,'props':[prop1,prop2],'grounding':grounding,'hash':hash,'open':open,'visit':actual_progress.visitId}

func draw_piece(canvas: CanvasItem, piece: Dictionary) -> bool:
	if not is_instance_valid(canvas) or piece.is_empty(): return false
	canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,piece.texture)
	return true

func draw_prop(canvas: CanvasItem, packet: Dictionary) -> bool:
	if not is_instance_valid(canvas) or packet.is_empty(): return false
	canvas.draw_set_transform(packet.origin,packet.rotation,packet.scale)
	canvas.draw_texture_rect(packet.texture,Rect2(Vector2.ZERO,packet.texture.get_size()),false,Color.WHITE)
	canvas.draw_set_transform(Vector2.ZERO,0.0,Vector2.ONE)
	return true
