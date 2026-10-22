extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const ContactGeometry = preload('res://scripts/world/gh_supply_crate_visual.gd')
const CONFIG = {
	'rs_terminal':{'id':'runtime2d.world.returnstation.terminal.v1','width':1071,'height':1218,'nominal':72},
	'beacon_folded':{'id':'runtime2d.prop.beacon.folded.v1','width':1226,'height':1005,'nominal':36},
	'beacon_deployed':{'id':'runtime2d.prop.beacon.deployed.v1','width':1333,'height':1054,'nominal':72}
}
var attempted = false
var textures: Dictionary = {}
func _load_once() -> void:
	if attempted:
		return
	attempted = true
	var file = FileAccess.open('res://data/registered_props.internal-use.json',FileAccess.READ)
	if file == null:
		return
	if file.get_length()>16384:
		file.close()
		return
	var bytes = file.get_buffer(file.get_length())
	file.close()
	if bytes.get_string_from_utf8().to_utf8_buffer()!=bytes:
		return
	var map = JSON.parse_string(bytes.get_string_from_utf8())
	if not map is Dictionary or map.get('schema')!='wuxian.registered-props.internal-use/v1' or typeof(map.get('registeredSourceReuse'))!=TYPE_BOOL or not map.registeredSourceReuse or map.get('approval')!='INTERNAL_REGISTERED_SOURCE_REUSE' or map.get('GodotDisplay')!='NOT_RUN' or map.get('publicRelease')!='NOT_ESTABLISHED' or not map.get('assets') is Dictionary or map.assets.size()!=3:
		return
	for key in CONFIG:
		var row = map.assets.get(key)
		var c = CONFIG[key]
		var path = 'res://assets/props/'+key+'.png'
		if not row is Dictionary or row.get('path')!=path or row.get('assetId')!=c.id or not P.counter(row.get('bytes'),1,4194304) or not _hash(row.get('sha256')) or not _pair(row.get('canvas'),c.width,c.height) or not _pair(row.get('anchor'),0.5,1.0) or not P.number(row.get('nominalHeight'),c.nominal,c.nominal) or not P.number(row.get('referencePpm'),48,48):
			continue
		var png_file = FileAccess.open(path,FileAccess.READ)
		if png_file == null:
			continue
		if png_file.get_length()!=row.bytes:
			png_file.close()
			continue
		var png = png_file.get_buffer(png_file.get_length())
		png_file.close()
		var hash = HashingContext.new()
		hash.start(HashingContext.HASH_SHA256)
		hash.update(png)
		if hash.finish().hex_encode()!=row.sha256:
			continue
		var image = Image.new()
		if image.load_png_from_buffer(png)!=OK or image.get_width()!=c.width or image.get_height()!=c.height or image.get_format()!=Image.FORMAT_RGBA8:
			continue
		var texture = ImageTexture.create_from_image(image)
		if texture != null:
			textures[key] = texture
func _pair(v, first: float, second: float) -> bool:
	return v is Array and v.size()==2 and P.number(v[0],first,first) and P.number(v[1],second,second)
func _hash(v) -> bool:
	if typeof(v)!=TYPE_STRING or v.length()!=64:
		return false
	for c in v:
		if not c in '0123456789abcdef':
			return false
	return true
func canvas_height(key: String, ppm: float) -> float:
	return float(CONFIG[key].nominal)*(ppm/48.0) if CONFIG.has(key) and is_finite(ppm) and ppm>0 else 0.0
static func _contact_beacon_row_valid(scene: Dictionary, art_id: String, asset_id: String, expected: Vector3) -> bool:
	var presentation: Variant = scene.get('presentation')
	if not presentation is Dictionary:
		return false
	var sprites: Variant = presentation.get('sprites')
	if not sprites is Array:
		return false
	var sprite_rows: Array = sprites
	if not P.rows(sprite_rows,2048) or not P.unique(sprite_rows,'id'):
		return false
	var matches = sprite_rows.filter(func(row): return row is Dictionary and row.get('id')==art_id and row.get('assetId')==asset_id and row.get('layer')=='visual.props_dynamic')
	if matches.size()!=1:
		return false
	var position: Variant = matches[0].get('position')
	return position is Array and position.size()==3 and P.number(position[0],expected.x,expected.x) and P.number(position[1],0.0,0.0) and P.number(position[2],expected.z,expected.z)

static func _contact_select_beacon(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary) -> Dictionary:
	var beacon: Variant = snapshot.get('greyHiveBeacon')
	if not beacon is Dictionary or beacon.size()!=2 or typeof(beacon.get('state'))!=TYPE_STRING or typeof(beacon.get('legacyCompletedWithoutReceipt'))!=TYPE_BOOL or beacon.legacyCompletedWithoutReceipt or not beacon.state in ['uncollected','mounted']:
		return {}
	var folded: bool = beacon.state=='uncollected'
	var key = 'beacon_folded' if folded else 'beacon_deployed'
	var marker_id = 'gh_beacon_deploy_marker' if folded else 'gh_beacon_storage_mount_marker'
	var art_id = 'beacon_folded_approved' if folded else 'beacon_mounted_approved'
	var asset_id = 'runtime2d.prop.beacon.folded.v1' if folded else 'runtime2d.prop.beacon.deployed.v1'
	var expected = Vector3(8.0,0.0,8.0) if folded else Vector3(17.0,0.0,8.0)
	if not _contact_beacon_row_valid(scene,art_id,asset_id,expected) or not canvas.has_method('_ground_beacon_body'):
		return {}
	var matches = snapshot.interactables.filter(func(row): return row is Dictionary and row.get('entityId')==marker_id)
	if matches.size()!=1 or not P.transform(matches[0].get('transform')):
		return {}
	var marker_position: Vector3 = P.vector(matches[0].transform.positionM)
	var selected: Variant = canvas.call('_ground_beacon_body')
	if not selected is Dictionary or selected.size()!=4 or selected.get('id')!='beacon-body:'+marker_id or selected.get('kind')!='beacon_body' or not selected.get('p') is Vector3 or selected.p!=marker_position or marker_position!=expected:
		return {}
	var data: Variant = selected.get('data')
	if not data is Dictionary or data.size()!=1 or data.get('key')!=key:
		return {}
	return {'key':key,'center':marker_position}

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
	if not ContactGeometry._contact_context_valid(snapshot,scene,current_key,scene_hash) or not ContactGeometry._contact_canvas_valid(canvas,snapshot,scene,current_key,scene_hash,projection,viewport):
		return false
	var selection = _contact_select_beacon(canvas,snapshot,scene)
	if selection.is_empty():
		return false
	var key: String = selection.key
	var cached: Variant = textures.get(key)
	var center: Variant = selection.center
	if not center is Vector3 or center.y!=0.0 or not cached is Texture2D or not is_instance_valid(cached):
		return false
	var radii = Vector2(0.20,0.13) if key=='beacon_folded' else Vector2(0.28,0.18)
	var tint = Color(0.015,0.020,0.022,0.08)
	var snapshot_frame = snapshot.duplicate(true)
	var scene_frame = scene.duplicate(true)
	var camera_frame = {'origin':projection.get('origin'),'center':projection.get('screen_center'),'viewport':projection.get('viewport'),'ppm':projection.get('ppm'),'initialized':projection.get('initialized')}
	var plan: Dictionary = ContactGeometry._contact_build_plan(Vector2(center.x,center.z),radii,tint,scene,projection,viewport)
	var final_selection = _contact_select_beacon(canvas,snapshot,scene)
	if plan.is_empty() or final_selection!=selection or snapshot_frame!=snapshot or scene_frame!=scene or projection.get('origin')!=camera_frame.origin or projection.get('screen_center')!=camera_frame.center or projection.get('viewport')!=camera_frame.viewport or projection.get('ppm')!=camera_frame.ppm or projection.get('initialized')!=camera_frame.initialized or not ContactGeometry._contact_context_valid(snapshot,scene,current_key,scene_hash) or not ContactGeometry._contact_canvas_valid(canvas,snapshot,scene,current_key,scene_hash,projection,viewport) or textures.get(key)!=cached or not cached is Texture2D or not is_instance_valid(cached):
		return false
	canvas.draw_colored_polygon(plan.points,plan.color)
	return true

func draw_body(canvas: CanvasItem, key: String, foot: Vector2, ppm: float) -> bool:
	if not CONFIG.has(key) or not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(ppm) or ppm<=0:
		return false
	_load_once()
	if not textures.has(key):
		return false
	var c = CONFIG[key]
	var factor = (float(c.nominal)/float(c.height))*(ppm/48.0)
	if not is_finite(factor) or factor<=0:
		return false
	canvas.draw_texture_rect(textures[key],Rect2(foot-Vector2(c.width*0.5,c.height)*factor,Vector2(c.width,c.height)*factor),false,Color.WHITE)
	return true
