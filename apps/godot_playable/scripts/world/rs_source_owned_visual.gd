extends RefCounted

const P = preload('res://scripts/bridge/protocol.gd')
const STEEL = 'res://assets/materials/cw_pressure_end_steel_local_r1.png'
const FACADE = 'res://assets/props/return_station/rs_orthographic_facade_local_r1.png'
const FACADE_R2 = 'res://assets/props/return_station/rs_orthographic_facade_local_r2.png'
const DECK_ATLAS = 'res://assets/props/return_station/rs_walk_deck_material_atlas_local_r1.png'
const DECK_ATLAS_R2 = 'res://assets/props/return_station/rs_walk_deck_material_atlas_local_r2.png'
const DECK_ATLAS_R3 = 'res://assets/props/return_station/rs_walk_deck_material_atlas_local_r3.png'
const DISTANT_SHELL = 'res://assets/props/return_station/rs_distant_shell_local_r1.png'
const DISTANT_SHELL_R2 = 'res://assets/props/return_station/rs_distant_shell_local_r2.png'
const AMBIENT_BACKDROP = 'res://assets/props/return_station/rs_ambient_backdrop_local_r1.png'
const AMBIENT_BACKDROP_R2 = 'res://assets/props/return_station/rs_ambient_backdrop_local_r2.png'
const OWNERS = {
	'north_bulkhead':Rect2(0,0,24,0.5), 'south_bulkhead':Rect2(0,15.5,24,0.5),
	'west_bulkhead_north':Rect2(0,0.5,0.5,4.5), 'west_bulkhead_south':Rect2(0,10,0.5,5.5),
	'east_bulkhead_north':Rect2(23.5,0.5,0.5,4.5), 'east_bulkhead_south':Rect2(23.5,10,0.5,5.5),
	'service_island_north':Rect2(9.5,6.5,5,1), 'service_island_south':Rect2(9.5,8.5,5,1),
}
var attempted := false
var ready := false
var steel: Texture2D
var facade: Texture2D
var facade_repeat_m := 0.0
var atlas_attempted := false
var atlas: Texture2D
var atlas_variant := 'none'
var shell_attempted := false
var shell_texture: Texture2D
var backdrop_attempted := false
var backdrop_texture: Texture2D
var probe_enabled := OS.get_environment('WUXIAN_RS_MATERIAL_PROBE')=='1'
var probe_active := false
var probe_done := false
var probe_phase := ''
var probe_viewport := Rect2()
var probe_counts := {'context':0,'shell':0,'floor':0,'island':0,'shell_viewport_intersections':0}
var probe_deck_import := 'not_tried'
var probe_deck_raw := 'not_tried'
var probe_deck_r2_import := 'not_tried'
var probe_deck_r2_raw := 'not_tried'
var probe_deck_r3_import := 'not_tried'
var probe_deck_r3_raw := 'not_tried'
var probe_shell_import := 'not_tried'
var probe_shell_raw := 'not_tried'
var probe_deck_selected := 'not_selected'
var probe_shell_selected := 'not_selected'
var probe_stage := 'not_started'
var probe_required_failure := 'required_resources'
var probe_shell_count := 0

func _texture(path: String) -> Texture2D:
	var tex: Texture2D
	if ResourceLoader.exists(path,'Texture2D'):
		tex=ResourceLoader.load(path,'Texture2D') as Texture2D
	if not is_instance_valid(tex):
		var image=Image.new()
		if image.load(path)==OK:
			if is_instance_valid(image) and not image.is_empty(): image.generate_mipmaps()
			tex=ImageTexture.create_from_image(image)
	return tex

func _load_once() -> bool:
	if attempted: return ready
	attempted=true
	steel=_texture(STEEL)
	facade=_texture(FACADE_R2)
	if not is_instance_valid(facade): facade=_texture(FACADE)
	if not is_instance_valid(steel) or steel.get_size()!=Vector2(1254,1254):
		if probe_enabled and not probe_done: probe_required_failure='required_steel'
		return false
	if not is_instance_valid(facade):
		if probe_enabled and not probe_done: probe_required_failure='required_facade'
		return false
	var image: Image = facade.get_image()
	if not is_instance_valid(image) or image.is_empty() or image.get_width()!=1536 or image.get_height()!=1024 or image.get_format()!=Image.FORMAT_RGB8 or image.detect_alpha()!=Image.ALPHA_NONE:
		if probe_enabled and not probe_done: probe_required_failure='required_facade'
		return false
	facade_repeat_m=4.0*float(image.get_width())/float(image.get_height())
	ready=is_finite(facade_repeat_m) and facade_repeat_m>0.0
	return ready

func _piece(camera, world: Array, uv: PackedVector2Array, tex: Texture2D) -> Dictionary:
	if world.size()!=4 or uv.size()!=4 or not is_instance_valid(tex): return {}
	var points:=PackedVector2Array()
	for position in world:
		var point: Vector2 = camera.project(position)
		if not point.is_finite(): return {}
		points.append(point)
	var area:=0.0
	for i in range(4): area+=points[i].cross(points[(i+1)%4])
	if not is_finite(area) or absf(area)<0.000001: return {}
	return {'points':points,'uv':uv,'texture':tex}

func _ground(camera, r: Rect2, height: float) -> Dictionary:
	var uv=PackedVector2Array([r.position/2.0,Vector2(r.end.x,r.position.y)/2.0,r.end/2.0,Vector2(r.position.x,r.end.y)/2.0])
	return _piece(camera,[Vector3(r.position.x,height,r.position.y),Vector3(r.end.x,height,r.position.y),Vector3(r.end.x,height,r.end.y),Vector3(r.position.x,height,r.end.y)],uv,steel)

func _valid_atlas_image(image: Image) -> bool:
	return is_instance_valid(image) and not image.is_empty() and image.get_width()==2172 and image.get_height()==724 and image.get_format()==Image.FORMAT_RGB8 and image.detect_alpha()==Image.ALPHA_NONE

func _atlas_ready() -> bool:
	if not atlas_attempted:
		atlas_attempted=true
		if probe_enabled: probe_deck_r3_import='absent'
		if ResourceLoader.exists(DECK_ATLAS_R3,'Texture2D'):
			var r3_imported=ResourceLoader.load(DECK_ATLAS_R3,'Texture2D') as Texture2D
			if is_instance_valid(r3_imported) and _valid_atlas_image(r3_imported.get_image()): atlas=r3_imported
			if probe_enabled: probe_deck_r3_import='accepted' if is_instance_valid(atlas) else 'unqualified'
		if not is_instance_valid(atlas):
			var r3_raw=Image.new()
			var r3_decoded=r3_raw.load(DECK_ATLAS_R3)==OK
			if r3_decoded and _valid_atlas_image(r3_raw): atlas=ImageTexture.create_from_image(r3_raw)
			if probe_enabled: probe_deck_r3_raw='decode_failed' if not r3_decoded else ('accepted' if is_instance_valid(atlas) else 'unqualified')
		if is_instance_valid(atlas) and atlas_variant=='none': atlas_variant='r3'
		if not is_instance_valid(atlas):
			if probe_enabled: probe_deck_r2_import='absent'
			if ResourceLoader.exists(DECK_ATLAS_R2,'Texture2D'):
				var r2_imported=ResourceLoader.load(DECK_ATLAS_R2,'Texture2D') as Texture2D
				if is_instance_valid(r2_imported) and _valid_atlas_image(r2_imported.get_image()): atlas=r2_imported
				if probe_enabled: probe_deck_r2_import='accepted' if is_instance_valid(atlas) else 'unqualified'
			if not is_instance_valid(atlas):
				var r2_raw=Image.new()
				var r2_decoded=r2_raw.load(DECK_ATLAS_R2)==OK
				if r2_decoded and _valid_atlas_image(r2_raw): atlas=ImageTexture.create_from_image(r2_raw)
				if probe_enabled: probe_deck_r2_raw='decode_failed' if not r2_decoded else ('accepted' if is_instance_valid(atlas) else 'unqualified')
			if is_instance_valid(atlas): atlas_variant='r2'
		if not is_instance_valid(atlas) and probe_enabled: probe_deck_import='absent'
		if not is_instance_valid(atlas) and ResourceLoader.exists(DECK_ATLAS,'Texture2D'):
			var imported=ResourceLoader.load(DECK_ATLAS,'Texture2D') as Texture2D
			if is_instance_valid(imported) and _valid_atlas_image(imported.get_image()): atlas=imported
			if probe_enabled: probe_deck_import='accepted' if is_instance_valid(atlas) else 'unqualified'
		if not is_instance_valid(atlas):
			var raw=Image.new()
			var decoded=raw.load(DECK_ATLAS)==OK
			if decoded and _valid_atlas_image(raw): atlas=ImageTexture.create_from_image(raw)
			if probe_enabled: probe_deck_raw='decode_failed' if not decoded else ('accepted' if is_instance_valid(atlas) else 'unqualified')
		if is_instance_valid(atlas) and atlas_variant=='none': atlas_variant='r1'
	return is_instance_valid(atlas)

func _atlas_uv(cell: int, u: float, v: float) -> Vector2:
	return Vector2((4.5+724.0*cell+715.0*u)/2172.0,(4.5+715.0*v)/724.0)

func _atlas_ground(camera, r: Rect2, height: float, cell: int, period: float) -> Array:
	var result: Array=[]
	var z=r.position.y
	while z<r.end.y-0.00001:
		var dz=minf(period,r.end.y-z)
		var x=r.position.x
		while x<r.end.x-0.00001:
			var dx=minf(period,r.end.x-x)
			var uv=PackedVector2Array([_atlas_uv(cell,0,0),_atlas_uv(cell,dx/period,0),_atlas_uv(cell,dx/period,dz/period),_atlas_uv(cell,0,dz/period)])
			var piece=_piece(camera,[Vector3(x,height,z),Vector3(x+dx,height,z),Vector3(x+dx,height,z+dz),Vector3(x,height,z+dz)],uv,atlas)
			if piece.is_empty(): return []
			result.append(piece)
			x+=dx
		z+=dz
	return result

func _atlas_front(camera, r: Rect2, height: float, index: int) -> Array:
	var corners=[r.position,Vector2(r.end.x,r.position.y),r.end,Vector2(r.position.x,r.end.y)]
	var front=0
	for i in range(4):
		if corners[i].x+corners[i].y>corners[front].x+corners[front].y: front=i
	var a: Vector2=corners[front]
	var b: Vector2=corners[[posmod(front-1,4),posmod(front+1,4)][index]]
	var length=a.distance_to(b)
	var result: Array=[]
	var s=0.0
	while s<length-0.00001:
		var next=minf(length,s+1.0)
		var p=a.lerp(b,s/length)
		var q=a.lerp(b,next/length)
		var uv=PackedVector2Array([_atlas_uv(2,0,1.0-height),_atlas_uv(2,next-s,1.0-height),_atlas_uv(2,next-s,1),_atlas_uv(2,0,1)])
		var piece=_piece(camera,[Vector3(p.x,height,p.y),Vector3(q.x,height,q.y),Vector3(q.x,0,q.y),Vector3(p.x,0,p.y)],uv,atlas)
		if piece.is_empty(): return []
		result.append(piece)
		s=next
	return result

func _structural_fascia(camera) -> Array:
	if atlas_variant!='r2' and atlas_variant!='r3': return []
	var result: Array=[]
	for span in [['x',16.0,0.0,24.0],['z',24.0,0.5,5.0],['z',24.0,10.0,15.5]]:
		var position: float=span[2]
		var end: float=span[3]
		while position<end-0.00001:
			var next=minf(end,position+1.0)
			var a=Vector3(position,0,span[1]) if span[0]=='x' else Vector3(span[1],0,position)
			var b=Vector3(next,0,span[1]) if span[0]=='x' else Vector3(span[1],0,next)
			var uv=PackedVector2Array([_atlas_uv(2,0,0),_atlas_uv(2,next-position,0),_atlas_uv(2,next-position,0.75),_atlas_uv(2,0,0.75)])
			var piece=_piece(camera,[a,b,b+Vector3(0,-0.75,0),a+Vector3(0,-0.75,0)],uv,atlas)
			if piece.is_empty(): return []
			result.append(piece)
			position=next
	return result

func _front(camera, r: Rect2, height: float, index: int) -> Dictionary:
	var corners=[r.position,Vector2(r.end.x,r.position.y),r.end,Vector2(r.position.x,r.end.y)]
	var front=0
	for i in range(4):
		if corners[i].x+corners[i].y>corners[front].x+corners[front].y: front=i
	var neighbors=[posmod(front-1,4),posmod(front+1,4)]
	var a: Vector2 = corners[front]
	var b: Vector2 = corners[neighbors[index]]
	var u0: float = (a.x+a.y)/2.0
	var u1: float = (b.x+b.y)/2.0
	return _piece(camera,[Vector3(a.x,height,a.y),Vector3(b.x,height,b.y),Vector3(b.x,0,b.y),Vector3(a.x,0,a.y)],PackedVector2Array([Vector2(u0,0),Vector2(u1,0),Vector2(u1,height/2.0),Vector2(u0,height/2.0)]),steel)

func _facade_plane(camera, axis: String, fixed: float, start: float, end: float) -> Array:
	var result: Array = []
	var position: float = start
	while position<end-0.00001:
		var cell: float = floorf(position/facade_repeat_m)*facade_repeat_m
		var next: float = minf(end,cell+facade_repeat_m)
		if next<=position: return []
		var u0: float = (position-cell)/facade_repeat_m
		var u1: float = (next-cell)/facade_repeat_m
		var a=Vector3(position,0.375,fixed) if axis=='x' else Vector3(fixed,0.375,position)
		var b=Vector3(next,0.375,fixed) if axis=='x' else Vector3(fixed,0.375,next)
		var piece=_piece(camera,[a,b,b+Vector3(0,4,0),a+Vector3(0,4,0)],PackedVector2Array([Vector2(u0,1),Vector2(u1,1),Vector2(u1,0),Vector2(u0,0)]),facade)
		if piece.is_empty(): return []
		result.append(piece)
		position=next
	return result

func _valid_shell_image(image: Image) -> bool:
	return is_instance_valid(image) and not image.is_empty() and image.get_width()==1254 and image.get_height()==1254 and image.get_format()==Image.FORMAT_RGB8 and image.detect_alpha()==Image.ALPHA_NONE

func _shell_ready() -> bool:
	if not shell_attempted:
		shell_attempted=true
		if ResourceLoader.exists(DISTANT_SHELL_R2,'Texture2D'):
			var r2_imported=ResourceLoader.load(DISTANT_SHELL_R2,'Texture2D') as Texture2D
			if is_instance_valid(r2_imported) and _valid_shell_image(r2_imported.get_image()): shell_texture=r2_imported
		if not is_instance_valid(shell_texture):
			var r2_raw=Image.new()
			var r2_decoded=r2_raw.load(DISTANT_SHELL_R2)==OK
			if r2_decoded and _valid_shell_image(r2_raw): shell_texture=ImageTexture.create_from_image(r2_raw)
		if not is_instance_valid(shell_texture):
			if probe_enabled: probe_shell_import='absent'
			if ResourceLoader.exists(DISTANT_SHELL,'Texture2D'):
				var imported=ResourceLoader.load(DISTANT_SHELL,'Texture2D') as Texture2D
				if is_instance_valid(imported) and _valid_shell_image(imported.get_image()): shell_texture=imported
				if probe_enabled: probe_shell_import='accepted' if is_instance_valid(shell_texture) else 'unqualified'
			if not is_instance_valid(shell_texture):
				var raw=Image.new()
				var decoded=raw.load(DISTANT_SHELL)==OK
				if decoded and _valid_shell_image(raw): shell_texture=ImageTexture.create_from_image(raw)
				if probe_enabled: probe_shell_raw='decode_failed' if not decoded else ('accepted' if is_instance_valid(shell_texture) else 'unqualified')
	return is_instance_valid(shell_texture)

func _distant_shell_plane(camera, axis: String, fixed: float, start: float, end: float, base_y: float = 0.375, phase: float = 0.0) -> Array:
	var result: Array=[]
	var position=start
	while position<end-0.00001:
		var cell=floorf((position-phase)/6.0)*6.0+phase
		var next=minf(end,cell+6.0)
		if next<=position: return []
		var u0=(position-cell)/6.0
		var u1=(next-cell)/6.0
		var a=Vector3(position,base_y,fixed) if axis=='x' else Vector3(fixed,base_y,position)
		var b=Vector3(next,base_y,fixed) if axis=='x' else Vector3(fixed,base_y,next)
		var piece=_piece(camera,[a,b,b+Vector3(0,6,0),a+Vector3(0,6,0)],PackedVector2Array([Vector2(u0,1),Vector2(u1,1),Vector2(u1,0),Vector2(u0,0)]),shell_texture)
		if piece.is_empty(): return []
		result.append(piece)
		position=next
	return result

func _valid_backdrop_image(image: Image) -> bool:
	return is_instance_valid(image) and not image.is_empty() and image.get_width()==1536 and image.get_height()==1024 and image.get_format()==Image.FORMAT_RGB8 and image.detect_alpha()==Image.ALPHA_NONE

func _backdrop_ready() -> bool:
	if not backdrop_attempted:
		backdrop_attempted=true
		if ResourceLoader.exists(AMBIENT_BACKDROP_R2,'Texture2D'):
			var r2_imported=ResourceLoader.load(AMBIENT_BACKDROP_R2,'Texture2D') as Texture2D
			if is_instance_valid(r2_imported) and _valid_backdrop_image(r2_imported.get_image()): backdrop_texture=r2_imported
		if not is_instance_valid(backdrop_texture):
			var r2_raw=Image.new()
			if r2_raw.load(AMBIENT_BACKDROP_R2)==OK and _valid_backdrop_image(r2_raw):
				if is_instance_valid(r2_raw) and not r2_raw.is_empty(): r2_raw.generate_mipmaps()
				backdrop_texture=ImageTexture.create_from_image(r2_raw)
		if not is_instance_valid(backdrop_texture):
			if ResourceLoader.exists(AMBIENT_BACKDROP,'Texture2D'):
				var imported=ResourceLoader.load(AMBIENT_BACKDROP,'Texture2D') as Texture2D
				if is_instance_valid(imported) and _valid_backdrop_image(imported.get_image()): backdrop_texture=imported
			if not is_instance_valid(backdrop_texture):
				var raw=Image.new()
				if raw.load(AMBIENT_BACKDROP)==OK and _valid_backdrop_image(raw):
					if is_instance_valid(raw) and not raw.is_empty(): raw.generate_mipmaps()
					backdrop_texture=ImageTexture.create_from_image(raw)
	return is_instance_valid(backdrop_texture)

func prepare(snapshot: Dictionary, scene: Dictionary, key: String, camera, environment, feet: Array) -> Dictionary:
	if probe_enabled and not probe_done:
		probe_deck_selected='not_selected'
		probe_shell_selected='not_selected'
		probe_stage='context'
		probe_shell_count=0
	if not P.snapshot(snapshot) or snapshot.get('worldId')!='return_station' or snapshot.get('sceneId')!='rs_core_room' or key!='return_station:rs_core_room:'+str(snapshot.get('worldEpoch')): return {}
	if probe_enabled and not probe_done: probe_stage='camera_layout_feet'
	if camera==null or not camera.initialized or not is_finite(camera.ppm) or camera.ppm<=0 or not environment.has_valid_layout('return_station','rs_core_room',scene) or feet.size()!=5: return {}
	if probe_enabled and not probe_done: probe_stage='required_resources'
	if not _load_once():
		if probe_enabled and not probe_done: probe_stage=probe_required_failure
		return {}
	if probe_enabled and not probe_done: probe_stage='base_packets'
	var back: Array = environment.build_packets(scene,snapshot.worldId,snapshot.sceneId,camera,feet,'back')
	var front: Array = environment.build_packets(scene,snapshot.worldId,snapshot.sceneId,camera,feet,'front')
	if back.is_empty() or front.is_empty(): return {}
	for id in ['mission_terminal_art','capability_terminal_art','storage_terminal_art','world_gate_terminal_art','save_rest_terminal_art']:
		var terminal_id='rs:terminal-base:'+id
		if back.filter(func(p): return p is Dictionary and p.get('id')==terminal_id and p.get('role')=='terminal_base').size()!=1: return {}
	var floor_piece=_ground(camera,Rect2(0,0,24,16),0.0)
	if floor_piece.is_empty(): return {}
	var tops: Dictionary = {}
	var fronts: Dictionary = {}
	for id in OWNERS:
		var top_id='rs:top:'+id
		var top_packets=back.filter(func(p): return p is Dictionary and p.get('id')==top_id and p.get('role')=='structure_top' and p.get('filled')==true)
		if top_packets.size()!=1: return {}
		var h: float = 1.0/6.0 if id.begins_with('service_island_') else 0.375
		var top=_ground(camera,OWNERS[id],h)
		if top.is_empty(): return {}
		tops[top_id]=top
		for i in range(2):
			var front_id='rs:front:'+id+':'+str(i)
			var front_packets=front.filter(func(p): return p is Dictionary and p.get('id')==front_id and p.get('role')=='front_face' and p.get('filled')==true)
			if front_packets.size()!=1: return {}
			var face=_front(camera,OWNERS[id],h,i)
			if face.is_empty(): return {}
			fronts[front_id]=face
	var context: Array = []
	for plane in [['x',0.5,0.5,23.5],['z',0.5,0.5,5.0],['z',0.5,10.0,15.5]]:
		var pieces=_facade_plane(camera,plane[0],plane[1],plane[2],plane[3])
		if pieces.is_empty(): return {}
		context.append_array(pieces)
	if probe_enabled and not probe_done:
		probe_stage='complete'
		probe_deck_selected='legacy'
		probe_shell_selected='none'
	# The optional decorative shell is independent of the material upgrade.
	var shell: Array=[]
	for plane in [['x',-2.0,0.5,23.5],['z',-2.0,0.5,2.45],['z',-2.0,10.0,15.5]]:
		var pieces=_facade_plane(camera,plane[0],plane[1],plane[2],plane[3])
		if pieces.is_empty(): shell.clear(); break
		shell.append_array(pieces)
	if _shell_ready():
		var upgraded_shell: Array=[]
		for plane in [['x',-2.0,-2.0,24.0],['z',-2.0,-2.0,2.45],['z',-2.0,10.0,16.0]]:
			var shell_pieces=_distant_shell_plane(camera,plane[0],plane[1],plane[2],plane[3])
			if shell_pieces.is_empty(): upgraded_shell.clear(); break
			upgraded_shell.append_array(shell_pieces)
		if not upgraded_shell.is_empty():
			shell=upgraded_shell
			if probe_enabled and not probe_done: probe_shell_selected='new'
		if not shell.is_empty():
			var peripheral=_distant_shell_plane(camera,'x',-8.0,16.0,32.0,0.0,16.0)
			if not peripheral.is_empty(): shell=peripheral+shell
	if probe_enabled and not probe_done:
		if not shell.is_empty() and probe_shell_selected!='new': probe_shell_selected='legacy'
		probe_shell_count=shell.size()
	if not shell.is_empty(): context=shell+context
	# Upgrade only when every atlas surface preflights.
	if _atlas_ready():
		var floor_upgrade=_atlas_ground(camera,Rect2(0,0,24,16),0.0,0,4.0)
		var top_upgrade: Dictionary={}
		var front_upgrade: Dictionary={}
		for id in ['service_island_north','service_island_south']:
			top_upgrade['rs:top:'+id]=_atlas_ground(camera,OWNERS[id],1.0/6.0,1,1.0)
			for i in range(2):
				front_upgrade['rs:front:'+id+':'+str(i)]=_atlas_front(camera,OWNERS[id],1.0/6.0,i)
		var complete=not floor_upgrade.is_empty()
		for owner_id in top_upgrade: complete=complete and not top_upgrade[owner_id].is_empty()
		for owner_id in front_upgrade: complete=complete and not front_upgrade[owner_id].is_empty()
		if complete:
			floor_piece=floor_upgrade
			if probe_enabled and not probe_done: probe_deck_selected='new'
			for owner_id in top_upgrade: tops[owner_id]=top_upgrade[owner_id]
			for owner_id in front_upgrade: fronts[owner_id]=front_upgrade[owner_id]
			if atlas_variant=='r2' or atlas_variant=='r3':
				var fascia=_structural_fascia(camera)
				if not fascia.is_empty(): context.append_array(fascia)
	var background: Dictionary={}
	if _backdrop_ready():
		# Screen-space cover: size the plane so its projection exactly fills the viewport.
		var vp: Vector2 = camera.viewport
		var pp: float = camera.ppm
		if is_finite(vp.x) and is_finite(vp.y) and vp.x>1.0 and vp.y>1.0 and is_finite(pp) and pp>0.0:
			var over: float = 2.0
			var y: float = -0.5
			var cx: float = camera.screen_center.x
			var cy: float = camera.screen_center.y
			var ox: float = camera.origin.x
			var oz: float = camera.origin.z
			var osum: float = (ox+oz)*0.5
			var u0: float = (0.0-over-cx)/pp+(ox-oz)
			var u1: float = (vp.x+over-cx)/pp+(ox-oz)
			var v0: float = (0.0-over-cy)/pp+(y-camera.origin.y)+osum
			var v1: float = (vp.y+over-cy)/pp+(y-camera.origin.y)+osum
			background=_piece(camera,[Vector3(v0+u0*0.5,y,v0-u0*0.5),Vector3(v0+u1*0.5,y,v0-u1*0.5),Vector3(v1+u1*0.5,y,v1-u1*0.5),Vector3(v1+u0*0.5,y,v1-u0*0.5)],PackedVector2Array([Vector2(0,0),Vector2(1,0),Vector2(1,1),Vector2(0,1)]),backdrop_texture)
	if background.is_empty() and _backdrop_ready():
		background=_piece(camera,[Vector3(-28,-0.5,4),Vector3(8,-0.5,-32),Vector3(8,-48.5,-32),Vector3(-28,-48.5,4)],PackedVector2Array([Vector2(0,0),Vector2(1,0),Vector2(1,1),Vector2(0,1)]),backdrop_texture)
	return {'floor':floor_piece,'tops':tops,'fronts':fronts,'context':context,'background':background,'facade_repeat_m':facade_repeat_m,
		'back_packets':back,'front_packets':front,'key':key,'epoch':snapshot.worldEpoch,'tick':snapshot.serverTick,
		'revision':snapshot.authorityRevision,'player':snapshot.player.entityId,'camera_origin':camera.origin,'camera_ppm':camera.ppm,'camera_center':camera.screen_center}

func draw_piece(canvas: CanvasItem, piece: Dictionary, alpha: float = 1.0) -> bool:
	if not is_instance_valid(canvas) or piece.is_empty() or not is_finite(alpha): return false
	var previous=canvas.texture_repeat
	var previous_filter=canvas.texture_filter
	canvas.texture_repeat=CanvasItem.TEXTURE_REPEAT_ENABLED
	canvas.texture_filter=CanvasItem.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS
	canvas.draw_polygon(piece.points,PackedColorArray([Color(1,1,1,clampf(alpha,0.0,1.0))]),piece.uv,piece.texture)
	if probe_active and probe_counts.has(probe_phase):
		probe_counts[probe_phase]+=1
		if probe_phase=='shell':
			var bounds=Rect2(piece.points[0],Vector2.ZERO)
			for point in piece.points: bounds=bounds.expand(point)
			if bounds.intersects(probe_viewport): probe_counts.shell_viewport_intersections+=1
	canvas.texture_repeat=previous
	canvas.texture_filter=previous_filter
	return true

func probe_finish(outcome: String) -> void:
	if not probe_enabled or probe_done: return
	probe_active=false
	probe_done=true
	var selected_variant=atlas_variant if probe_deck_selected=='new' else probe_deck_selected
	print('RS_MATERIAL_PROBE '+JSON.stringify({'outcome':outcome,'required_stage':probe_stage,'deck_r2_import':probe_deck_r2_import,'deck_r2_raw':probe_deck_r2_raw,'deck_import':probe_deck_import,'deck_raw':probe_deck_raw,'deck_variant':selected_variant,'shell_import':probe_shell_import,'shell_raw':probe_shell_raw,'deck_selected':probe_deck_selected,'shell_selected':probe_shell_selected,'submissions':probe_counts}))

func draw_surface(canvas: CanvasItem, surface, alpha: float = 1.0) -> bool:
	if surface is Dictionary: return draw_piece(canvas,surface,alpha)
	if not surface is Array or surface.is_empty(): return false
	for piece in surface:
		if not draw_piece(canvas,piece,alpha): return false
	return true
