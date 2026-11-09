extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const TILE = 4.0
const MAX_AXIS = 16
const MATERIAL_PATH = 'res://assets/props/grey_hive/floor_tiles.png'
var material_attempted := false
var material_texture: Texture2D

func _material_pieces(camera, x0: float, z0: float, x1: float, z1: float) -> Array:
	var pieces: Array = []
	var x=x0
	while x<x1:
		var cx=floorf(x/2.0)*2.0
		var xe=minf(x1,cx+2.0)
		if xe<=x: return []
		var z=z0
		while z<z1:
			var cz=floorf(z/2.0)*2.0
			var ze=minf(z1,cz+2.0)
			if ze<=z: return []
			var points=_quad(camera,x,z,xe,ze)
			if points.size()!=4: return []
			var u0=(x-cx)/2.0; var u1=(xe-cx)/2.0; var v0=(z-cz)/2.0; var v1=(ze-cz)/2.0
			var uv:=PackedVector2Array()
			for f in [Vector2(u0,v0),Vector2(u1,v0),Vector2(u1,v1),Vector2(u0,v1)]:
				uv.append((Vector2(187.5,184.5)+Vector2(158,121)*f.x+Vector2(-160,121)*f.y)/Vector2(1434,980))
			pieces.append({'points':points,'uv':uv})
			z=ze
		x=xe
	return pieces

func prepare_material(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, current_key: String, camera, viewport: Rect2, back_packets: Array) -> Dictionary:
	if not is_instance_valid(canvas) or not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.worldId!='return_station' or snapshot.sceneId!='rs_core_room' or current_key!='return_station:rs_core_room:'+str(snapshot.worldEpoch):
		return {}
	if scene.get('worldId')!='return_station' or scene.get('sceneId')!='rs_core_room' or not scene.get('boundsM') is Dictionary or camera==null or not camera.initialized or not is_finite(camera.ppm) or camera.ppm<=0.0 or not _point_valid(viewport.position) or not _point_valid(viewport.size) or viewport.size.x<=0.0 or viewport.size.y<=0.0:
		return {}
	var bounds: Dictionary = scene.boundsM
	if not P.number(bounds.get('x'),0,0) or not P.number(bounds.get('z'),0,0) or not P.number(bounds.get('width'),24,24) or not P.number(bounds.get('depth'),16,16):
		return {}
	var base = back_packets.filter(func(packet): return packet is Dictionary and packet.get('id')=='rs:terminal-base:mission_terminal_art' and packet.get('role')=='terminal_base' and packet.get('filled')==true)
	if base.size()!=1 or not base[0].get('points') is PackedVector2Array or base[0].points.size()!=4:
		return {}
	var expected = _quad(camera,7.375,3.78,8.625,4.22)
	if expected.size()!=4: return {}
	for i in range(4):
		if expected[i].distance_to(base[0].points[i])>0.01: return {}
	var floor_pieces = _material_pieces(camera,0,0,24,16)
	var pad_pieces = _material_pieces(camera,7.375,3.78,8.625,4.22)
	if floor_pieces.size()!=96 or pad_pieces.size()!=4:
		return {}
	if not material_attempted:
		material_attempted = true
		if ResourceLoader.exists(MATERIAL_PATH):
			var loaded = load(MATERIAL_PATH)
			if loaded is Texture2D: material_texture = loaded
		if not is_instance_valid(material_texture):
			var image = Image.new()
			if image.load(MATERIAL_PATH)==OK and image.get_width()==1434 and image.get_height()==980:
				material_texture = ImageTexture.create_from_image(image)
	if not is_instance_valid(material_texture) or material_texture.get_width()!=1434 or material_texture.get_height()!=980:
		return {}
	return {'texture':material_texture,'floor':floor_pieces,'pad':pad_pieces}

func draw_material(canvas: CanvasItem, pieces: Array, texture: Texture2D) -> void:
	for piece in pieces:
		canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,texture)

func _point_valid(p: Vector2) -> bool:
	return is_finite(p.x) and is_finite(p.y)

func _quad(camera, x0: float, z0: float, x1: float, z1: float) -> PackedVector2Array:
	var result = PackedVector2Array()
	for point in [Vector3(x0,0,z0),Vector3(x1,0,z0),Vector3(x1,0,z1),Vector3(x0,0,z1)]:
		var screen: Vector2 = camera.project(point)
		if not _point_valid(screen):
			return PackedVector2Array()
		result.append(screen)
	var sign_value: float = 0.0
	for i in range(4):
		var cross: float = (result[(i+1)%4]-result[i]).cross(result[(i+2)%4]-result[(i+1)%4])
		if not is_finite(cross) or absf(cross)<0.001 or (sign_value!=0.0 and sign_value*cross<=0.0):
			return PackedVector2Array()
		sign_value = cross
	return result

func draw_floor(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, current_key: String, camera, viewport: Rect2) -> void:
	if not is_instance_valid(canvas) or not P.snapshot(snapshot) or snapshot.get('entryToken')!=null or snapshot.worldId!='return_station' or snapshot.sceneId!='rs_core_room' or current_key!='return_station:rs_core_room:'+str(snapshot.worldEpoch):
		return
	if scene.get('worldId')!='return_station' or scene.get('sceneId')!='rs_core_room' or not scene.get('boundsM') is Dictionary or camera==null or not camera.initialized:
		return
	if not is_finite(camera.ppm) or camera.ppm<=0.0 or not _point_valid(viewport.position) or not _point_valid(viewport.size) or viewport.size.x<=0.0 or viewport.size.y<=0.0:
		return
	var bounds: Dictionary = scene.boundsM
	for key in ['x','z','width','depth']:
		if typeof(bounds.get(key)) not in [TYPE_INT,TYPE_FLOAT] or not is_finite(float(bounds[key])):
			return
	var x: float = float(bounds.x); var z: float = float(bounds.z)
	var width: float = float(bounds.width); var depth: float = float(bounds.depth)
	if width<=0.3 or depth<=0.3 or width>TILE*MAX_AXIS or depth>TILE*MAX_AXIS or not is_finite(x+width) or not is_finite(z+depth):
		return
	var nx: int = ceili(width/TILE); var nz: int = ceili(depth/TILE)
	if nx<1 or nz<1 or nx>MAX_AXIS or nz>MAX_AXIS or nx*nz>256 or nx+nz-2>128:
		return
	var plates: Array = []
	for ix in range(nx):
		for iz in range(nz):
			var x0: float = x+float(ix)*TILE; var z0: float = z+float(iz)*TILE
			var quad: PackedVector2Array = _quad(camera,x0,z0,minf(x+width,x0+TILE),minf(z+depth,z0+TILE))
			if quad.size()!=4:
				return
			plates.append({'points':quad,'odd':(ix+iz)%2==1})
	var seams: Array = []
	for ix in range(1,nx):
		var sx: float = x+float(ix)*TILE
		var a: Vector2 = camera.project(Vector3(sx,0,z)); var b: Vector2 = camera.project(Vector3(sx,0,z+depth))
		if not _point_valid(a) or not _point_valid(b):
			return
		seams.append([a,b])
	for iz in range(1,nz):
		var sz: float = z+float(iz)*TILE
		var a: Vector2 = camera.project(Vector3(x,0,sz)); var b: Vector2 = camera.project(Vector3(x+width,0,sz))
		if not _point_valid(a) or not _point_valid(b):
			return
		seams.append([a,b])
	var inset: PackedVector2Array = _quad(camera,x+0.15,z+0.15,x+width-0.15,z+depth-0.15)
	if inset.size()!=4:
		return
	inset.append(inset[0])
	for plate in plates:
		var points: PackedVector2Array = plate.points
		var rect = Rect2(points[0],Vector2.ZERO)
		for point in points:
			rect = rect.expand(point)
		if rect.intersects(viewport):
			canvas.draw_colored_polygon(points,Color('#29434d') if plate.odd else Color('#263f49'))
	for line in seams:
		canvas.draw_line(line[0],line[1],Color('#1d333d'),1.0,true)
	canvas.draw_polyline(inset,Color('#35505a'),1.0,true)
