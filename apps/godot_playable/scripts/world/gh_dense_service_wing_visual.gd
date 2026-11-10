extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
# Embedded design witness prevents client/source geometry drift; this is not a visibility rule.
const EXPECTED_JSON = "{\"schema\":\"gh-service-wing-dense/v1\",\"worldId\":\"grey_hive\",\"floorId\":\"gh_service_wing_y0\",\"note\":\"Authored design coordinates; visibility is not specified.\",\"offsets\":{\"gh_power_room\":[0,0],\"gh_gate_a\":[20,0]},\"heightM\":0.25,\"nominalPlayerRadiusM\":0.35,\"walls\":[{\"id\":\"dense_power_north_0\",\"sceneId\":\"gh_power_room\",\"rect\":[0.5,5,3,5.5]},{\"id\":\"dense_power_north_1\",\"sceneId\":\"gh_power_room\",\"rect\":[5,5,10,5.5]},{\"id\":\"dense_power_north_2\",\"sceneId\":\"gh_power_room\",\"rect\":[12,5,16,5.5]},{\"id\":\"dense_power_north_3\",\"sceneId\":\"gh_power_room\",\"rect\":[18,5,19.5,5.5]},{\"id\":\"dense_gate_north_0\",\"sceneId\":\"gh_gate_a\",\"rect\":[0.5,5,4,5.5]},{\"id\":\"dense_gate_north_1\",\"sceneId\":\"gh_gate_a\",\"rect\":[6,5,9.5,5.5]},{\"id\":\"dense_gate_north_2\",\"sceneId\":\"gh_gate_a\",\"rect\":[10.5,5,14,5.5]},{\"id\":\"dense_gate_north_3\",\"sceneId\":\"gh_gate_a\",\"rect\":[16,5,19.5,5.5]},{\"id\":\"dense_power_south_0\",\"sceneId\":\"gh_power_room\",\"rect\":[0.5,8.5,3,9]},{\"id\":\"dense_power_south_1\",\"sceneId\":\"gh_power_room\",\"rect\":[5,8.5,10,9]},{\"id\":\"dense_power_south_2\",\"sceneId\":\"gh_power_room\",\"rect\":[12,8.5,16,9]},{\"id\":\"dense_power_south_3\",\"sceneId\":\"gh_power_room\",\"rect\":[18,8.5,19.5,9]},{\"id\":\"dense_gate_south_0\",\"sceneId\":\"gh_gate_a\",\"rect\":[0.5,8.5,4,9]},{\"id\":\"dense_gate_south_1\",\"sceneId\":\"gh_gate_a\",\"rect\":[6,8.5,9.5,9]},{\"id\":\"dense_gate_south_2\",\"sceneId\":\"gh_gate_a\",\"rect\":[10.5,8.5,14,9]},{\"id\":\"dense_gate_south_3\",\"sceneId\":\"gh_gate_a\",\"rect\":[16,8.5,19.5,9]},{\"id\":\"dense_power_ncross_8_a\",\"sceneId\":\"gh_power_room\",\"rect\":[7.75,0.5,8.25,2]},{\"id\":\"dense_power_ncross_8_b\",\"sceneId\":\"gh_power_room\",\"rect\":[7.75,4.7,8.25,5]},{\"id\":\"dense_power_scross_8_a\",\"sceneId\":\"gh_power_room\",\"rect\":[7.75,8.5,8.25,9]},{\"id\":\"dense_power_scross_8_b\",\"sceneId\":\"gh_power_room\",\"rect\":[7.75,12.5,8.25,13.5]},{\"id\":\"dense_power_ncross_14_a\",\"sceneId\":\"gh_power_room\",\"rect\":[13.75,0.5,14.25,2]},{\"id\":\"dense_power_ncross_14_b\",\"sceneId\":\"gh_power_room\",\"rect\":[13.75,4.7,14.25,5]},{\"id\":\"dense_power_scross_14_a\",\"sceneId\":\"gh_power_room\",\"rect\":[13.75,8.5,14.25,9]},{\"id\":\"dense_power_scross_14_b\",\"sceneId\":\"gh_power_room\",\"rect\":[13.75,12.5,14.25,13.5]}],\"regions\":[{\"id\":\"power_nw\",\"name\":\"管线检修间\",\"kind\":\"functional_bay\",\"sceneId\":\"gh_power_room\",\"polygon\":[[0.5,0.5],[8,0.5],[8,5.5],[0.5,5.5]]},{\"id\":\"power_nm\",\"name\":\"配电检查间\",\"kind\":\"functional_bay\",\"sceneId\":\"gh_power_room\",\"polygon\":[[8,0.5],[14,0.5],[14,5.5],[8,5.5]]},{\"id\":\"power_ne\",\"name\":\"分配维护间\",\"kind\":\"functional_bay\",\"sceneId\":\"gh_power_room\",\"polygon\":[[14,0.5],[19.5,0.5],[19.5,5.5],[14,5.5]]},{\"id\":\"power_sw\",\"name\":\"值守休整间\",\"kind\":\"functional_bay\",\"sceneId\":\"gh_power_room\",\"polygon\":[[0.5,8.5],[8,8.5],[8,13.5],[0.5,13.5]]},{\"id\":\"power_sm\",\"name\":\"器材整备间\",\"kind\":\"functional_bay\",\"sceneId\":\"gh_power_room\",\"polygon\":[[8,8.5],[14,8.5],[14,13.5],[8,13.5]]},{\"id\":\"power_se\",\"name\":\"维护储物间\",\"kind\":\"functional_bay\",\"sceneId\":\"gh_power_room\",\"polygon\":[[14,8.5],[19.5,8.5],[19.5,13.5],[14,13.5]]},{\"id\":\"gate_nw\",\"name\":\"闸前线缆间\",\"kind\":\"functional_bay\",\"sceneId\":\"gh_gate_a\",\"polygon\":[[20.5,0.5],[29.5,0.5],[29.5,5.5],[20.5,5.5]]},{\"id\":\"gate_sw\",\"name\":\"闸前检查间\",\"kind\":\"functional_bay\",\"sceneId\":\"gh_gate_a\",\"polygon\":[[20.5,8.5],[29.5,8.5],[29.5,13.5],[20.5,13.5]]},{\"id\":\"gate_ne\",\"name\":\"闸后维护间\",\"kind\":\"functional_bay\",\"sceneId\":\"gh_gate_a\",\"polygon\":[[30.5,0.5],[39.5,0.5],[39.5,5.5],[30.5,5.5]]},{\"id\":\"gate_se\",\"name\":\"闸后物资间\",\"kind\":\"functional_bay\",\"sceneId\":\"gh_gate_a\",\"polygon\":[[30.5,8.5],[39.5,8.5],[39.5,13.5],[30.5,13.5]]},{\"id\":\"corr_power_w\",\"name\":\"供电西段走廊\",\"kind\":\"corridor_section\",\"sceneId\":\"gh_power_room\",\"polygon\":[[0,5.5],[8,5.5],[8,8.5],[0,8.5]]},{\"id\":\"corr_power_m\",\"name\":\"供电中段走廊\",\"kind\":\"corridor_section\",\"sceneId\":\"gh_power_room\",\"polygon\":[[8,5.5],[14,5.5],[14,8.5],[8,8.5]]},{\"id\":\"corr_power_e\",\"name\":\"供电东段走廊\",\"kind\":\"corridor_section\",\"sceneId\":\"gh_power_room\",\"polygon\":[[14,5.5],[20,5.5],[20,8.5],[14,8.5]]},{\"id\":\"corr_gate_w\",\"name\":\"闸前主走廊\",\"kind\":\"corridor_section\",\"sceneId\":\"gh_gate_a\",\"polygon\":[[20,5.5],[29.5,5.5],[29.5,8.5],[20,8.5]]},{\"id\":\"corr_gate_threshold\",\"name\":\"受控闸口\",\"kind\":\"existing_controlled_threshold\",\"sceneId\":\"gh_gate_a\",\"polygon\":[[29.5,5.5],[30.5,5.5],[30.5,8.5],[29.5,8.5]]},{\"id\":\"corr_gate_e\",\"name\":\"闸后主走廊\",\"kind\":\"corridor_section\",\"sceneId\":\"gh_gate_a\",\"polygon\":[[30.5,5.5],[40,5.5],[40,8.5],[30.5,8.5]]}],\"edges\":[{\"id\":\"main_0\",\"from\":\"corr_power_w\",\"to\":\"corr_power_m\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"main_1\",\"from\":\"corr_power_m\",\"to\":\"corr_power_e\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"main_2\",\"from\":\"corr_power_e\",\"to\":\"corr_gate_w\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"main_3\",\"from\":\"corr_gate_w\",\"to\":\"corr_gate_threshold\",\"bidirectional\":true,\"requiresEvent\":\"hive_power\"},{\"id\":\"main_4\",\"from\":\"corr_gate_threshold\",\"to\":\"corr_gate_e\",\"bidirectional\":true,\"requiresEvent\":\"hive_power\"},{\"id\":\"access_power_nw\",\"from\":\"corr_power_w\",\"to\":\"power_nw\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"access_power_nm\",\"from\":\"corr_power_m\",\"to\":\"power_nm\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"access_power_ne\",\"from\":\"corr_power_e\",\"to\":\"power_ne\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"access_power_sw\",\"from\":\"corr_power_w\",\"to\":\"power_sw\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"access_power_sm\",\"from\":\"corr_power_m\",\"to\":\"power_sm\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"access_power_se\",\"from\":\"corr_power_e\",\"to\":\"power_se\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"access_gate_nw\",\"from\":\"corr_gate_w\",\"to\":\"gate_nw\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"access_gate_sw\",\"from\":\"corr_gate_w\",\"to\":\"gate_sw\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"access_gate_ne\",\"from\":\"corr_gate_e\",\"to\":\"gate_ne\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"access_gate_se\",\"from\":\"corr_gate_e\",\"to\":\"gate_se\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"loop_north_0\",\"from\":\"power_nw\",\"to\":\"power_nm\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"loop_north_1\",\"from\":\"power_nm\",\"to\":\"power_ne\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"loop_south_0\",\"from\":\"power_sw\",\"to\":\"power_sm\",\"bidirectional\":true,\"requiresEvent\":null},{\"id\":\"loop_south_1\",\"from\":\"power_sm\",\"to\":\"power_se\",\"bidirectional\":true,\"requiresEvent\":null}]}"
const DATA_PATH = 'res://data/gh_service_wing_dense.local.json'
const HASHES = {'gh_power_room':'3d05edeca4a119bc7f2db7aa094041a7ad61380b00015a613fda787025e67854','gh_gate_a':'3c62f3c35a2c9f2e935707e7a93c34a8d6f5696ed730b8487c4efe0b6d1ee484'}
var attempted = false
var layout: Dictionary = {}
func _load() -> bool:
	if attempted: return not layout.is_empty()
	attempted = true
	var f = FileAccess.open(DATA_PATH,FileAccess.READ)
	if f==null: return false
	var raw = f.get_as_text()
	f.close()
	var value = JSON.parse_string(raw)
	var expected = JSON.parse_string(EXPECTED_JSON)
	if not value is Dictionary or not expected is Dictionary or value!=expected: return false
	layout = value
	return true

func prepare(snapshot: Dictionary, current: Dictionary, key: String, target: Dictionary, target_hash: String, offset: float, camera, texture) -> Array:
	if not _load() or not P.snapshot(snapshot) or snapshot.get('worldId')!='grey_hive' or current.get('worldId')!='grey_hive' or current.get('sceneId')!=snapshot.get('sceneId') or key!='grey_hive:'+str(snapshot.sceneId)+':'+str(snapshot.worldEpoch): return []
	if not target is Dictionary or target.get('worldId')!='grey_hive' or not HASHES.has(target.get('sceneId')) or HASHES[target.sceneId]!=target_hash or not camera.initialized or not is_instance_valid(texture): return []
	if not ((target.sceneId==snapshot.sceneId and offset==0.0) or (snapshot.sceneId=='gh_power_room' and target.sceneId=='gh_gate_a' and offset==20.0) or (snapshot.sceneId=='gh_gate_a' and target.sceneId=='gh_power_room' and offset==-20.0)): return []
	var escapes = snapshot.get('densePartitionEscapeIds')
	if not escapes is Array or escapes.size()>24: return []
	var seen = {}
	for id in escapes:
		if typeof(id)!=TYPE_STRING or seen.has(id): return []
		var matches=layout.walls.filter(func(w): return w.id==id and w.sceneId==snapshot.sceneId)
		if matches.size()!=1: return []
		seen[id]=true
	var result: Array = []
	for wall in layout.walls:
		if wall.sceneId!=target.sceneId: continue
		if target.sceneId==snapshot.sceneId and seen.has(wall.id):
			seen.erase(wall.id)
			continue
		var r: Array = wall.rect
		var x=float(r[0])+offset; var z=float(r[1]); var xmax=float(r[2])+offset; var zmax=float(r[3]); var part=0
		while x<xmax-0.00001:
			var nx=minf(xmax,(floor(x*2.0+0.00001)+1.0)*0.5)
			if nx<=x: return []
			var zz=z
			while zz<zmax-0.00001:
				var nz=minf(zmax,(floor(zz*2.0+0.00001)+1.0)*0.5)
				if nz<=zz: return []
				var u0=(x-floor(x*2.0+0.00001)*0.5)*2.0; var u1=u0+(nx-x)*2.0
				var v0=(zz-floor(zz*2.0+0.00001)*0.5)*2.0; var v1=v0+(nz-zz)*2.0
				var cap=PackedVector2Array([camera.project(Vector3(x,.25,zz)),camera.project(Vector3(nx,.25,zz)),camera.project(Vector3(nx,.25,nz)),camera.project(Vector3(x,.25,nz))])
				var capuv=PackedVector2Array([Vector2(u0,v0),Vector2(u1,v0),Vector2(u1,v1),Vector2(u0,v1)])
				var pieces: Array = [{'world':Rect2(x,zz,nx-x,nz-zz),'points':cap,'uv':capuv}]
				for edge in [[Vector2(x,zz),Vector2(nx,zz)],[Vector2(nx,zz),Vector2(nx,nz)],[Vector2(nx,nz),Vector2(x,nz)],[Vector2(x,nz),Vector2(x,zz)]]:
					var a: Vector2=edge[0]; var b: Vector2=edge[1]
					var u=(absf(b.x-a.x)+absf(b.y-a.y))*2.0
					var face=PackedVector2Array([camera.project(Vector3(a.x,0,a.y)),camera.project(Vector3(b.x,0,b.y)),camera.project(Vector3(b.x,.25,b.y)),camera.project(Vector3(a.x,.25,a.y))])
					pieces.append({'world':Rect2(minf(a.x,b.x),minf(a.y,b.y),absf(b.x-a.x),absf(b.y-a.y)),'points':face,'uv':PackedVector2Array([Vector2(0,1),Vector2(u,1),Vector2(u,.5),Vector2(0,.5)])})
				for piece in pieces:
					for pt in piece.points:
						if not is_finite(pt.x) or not is_finite(pt.y): return []
				result.append({'id':wall.id+':'+str(part),'p':Vector3((x+nx)*.5,0,(zz+nz)*.5),'world':Rect2(x,zz,nx-x,nz-zz),'pieces':pieces,'texture':texture})
				part+=1
				if part>128: return []
				zz=nz
			x=nx
	if target.sceneId==snapshot.sceneId and not seen.is_empty(): return []
	return result

func draw(canvas: CanvasItem, record: Dictionary) -> void:
	for piece in record.pieces: canvas.draw_polygon(piece.points,PackedColorArray([Color.WHITE]),piece.uv,record.texture)

# Only the player's current region is exposed here; this adds no FOV or unexplored-map knowledge.
func current_region(snapshot: Dictionary) -> Dictionary:
	if not _load() or not P.snapshot(snapshot) or snapshot.get('worldId')!='grey_hive' or not snapshot.sceneId in ['gh_power_room','gh_gate_a'] or not snapshot.get('densePartitionEscapeIds') is Array: return {}
	var selected = snapshot.get('denseOccupiedRegion')
	if typeof(selected)!=TYPE_STRING or snapshot.get('currentVisibleRegions')!=[selected]: return {}
	var rows = layout.regions.filter(func(row): return row is Dictionary and row.get('id')==selected and row.get('sceneId')==snapshot.sceneId)
	if rows.size()!=1: return {}
	var point=P.vector(snapshot.player.transform.positionM)
	# The bounds below are JSON float64. Rebuilding the player coordinate through Vector3
	# (single precision in the shipping build) and then comparing it with < / > excludes a
	# player standing exactly on a region's min corner: f32(6.2)=6.199999809265137 < 6.2.
	# Take the same coordinate from the snapshot's float64 metres; the inclusive bounds are
	# unchanged (nothing is loosened, only the width of the measured value).
	var pm = snapshot.player.transform.positionM
	var px: float = float(pm.xM) if pm is Dictionary else float(point.x)
	var pz: float = float(pm.zM) if pm is Dictionary else float(point.z)
	var global_x: float = px + (20.0 if snapshot.sceneId=='gh_gate_a' else 0.0)
	var global_z: float = pz
	var poly: Array=rows[0].polygon
	if poly.size()!=4: return {}
	var x0=float(poly[0][0]); var z0=float(poly[0][1]); var x1=float(poly[2][0]); var z1=float(poly[2][1])
	if not point.is_finite() or point.y!=0.0 or global_x<x0 or global_x>x1 or global_z<z0 or global_z>z1: return {}
	return {'id':rows[0].id,'name':rows[0].name,'rect':Rect2(x0,z0,x1-x0,z1-z0)}

# Entity classification uses the same stable lexical shared-edge tie break as authority.
func same_region(snapshot: Dictionary, position: Vector3) -> bool:
	var current: Dictionary = current_region(snapshot)
	if current.is_empty() or not position.is_finite() or position.y!=0.0: return false
	var gx=position.x+(20.0 if snapshot.sceneId=='gh_gate_a' else 0.0)
	var candidates: Array[String] = []
	for row in layout.regions:
		if row.sceneId!=snapshot.sceneId: continue
		var poly: Array=row.polygon
		if poly.size()!=4: return false
		if gx>=float(poly[0][0]) and gx<=float(poly[2][0]) and position.z>=float(poly[0][1]) and position.z<=float(poly[2][1]): candidates.append(row.id)
	if candidates.is_empty(): return false
	candidates.sort()
	return candidates[0]==current.id
