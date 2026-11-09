extends RefCounted

const PATHS = {
	'gh': 'res://assets/props/grey_hive/gh_entry_whole_local_r1.png',
	'mh': 'res://assets/props/mist_harbor/mh_breakwater_whole_local_r1.png',
	'cw': 'res://assets/props/clockworks/cw_foundry_whole_local_r1.png',
	'pressure': 'res://assets/props/clockworks/cw_pressure_hall_whole_scene_local_r1.png',
	'fog_pier': 'res://assets/props/mist_harbor/mh_fog_pier_whole_scene_local_r1.png',
	'extraction': 'res://assets/props/mist_harbor/mh_extraction_static_master_local_r1.png',
	'shaft': 'res://assets/props/grey_hive/gh_central_shaft_static_master_local_r1.png',
	'decon': 'res://assets/props/grey_hive/gh_deep_decon_whole_scene_local_r1.png',
	'boiler': 'res://assets/props/clockworks/cw_boiler_static_master_local_r1.png',
	'gear': 'res://assets/props/clockworks/cw_gear_lower_static_master_local_r1.png',
	'heart': 'res://assets/props/clockworks/cw_furnace_heart_whole_scene_local_r1.png',
	'sentinel': 'res://assets/props/grey_hive/gh_sentinel_static_master_local_r1.png',
	'conveyor': 'res://assets/props/clockworks/cw_conveyor_whole_scene_local_r1.png',
	'regulator': 'res://assets/props/clockworks/cw_furnace_heart_whole_scene_local_r1.png',
	'forged_arena': 'res://assets/props/clockworks/cw_forged_guard_whole_scene_local_r2.png',
	'shutdown': 'res://assets/props/clockworks/cw_shutdown_exit_static_master_local_r1.png'
}
var attempted: Dictionary = {}
var textures: Dictionary = {}

func draw_master(canvas: CanvasItem, key: String, viewport: Rect2, camera = null, anchor = null) -> bool:
	if not is_instance_valid(canvas) or not PATHS.has(key) or not is_finite(viewport.position.x) or not is_finite(viewport.position.y) or not is_finite(viewport.size.x) or not is_finite(viewport.size.y) or viewport.size.x<=0 or viewport.size.y<=0: return false
	if not attempted.has(key):
		attempted[key] = true
		var path: String = PATHS[key]
		var texture: Texture2D
		if ResourceLoader.exists(path,'Texture2D'):
			var loaded = ResourceLoader.load(path,'Texture2D')
			if loaded is Texture2D:
				var imported_image: Image = loaded.get_image()
				if is_instance_valid(imported_image) and imported_image.get_width()==1536 and imported_image.get_height()==1024 and imported_image.get_format()==Image.FORMAT_RGB8:
					texture = loaded
		if not is_instance_valid(texture):
			var image = Image.new()
			var bytes = FileAccess.get_file_as_bytes(path)
			if image.load_png_from_buffer(bytes)==OK and image.get_width()==1536 and image.get_height()==1024 and image.get_format()==Image.FORMAT_RGB8:
				texture = ImageTexture.create_from_image(image)
		if is_instance_valid(texture): textures[key] = texture
	if not textures.has(key) or not is_instance_valid(textures[key]): return false
	var scale = maxf(viewport.size.x/1536.0,viewport.size.y/1024.0)
	var extent = Vector2(1536,1024)*scale
	var origin = viewport.position+(viewport.size-extent)*0.5
	if camera != null and anchor is Vector3 and camera.initialized:
		var projected: Vector2 = camera.project(anchor)
		if not is_finite(projected.x) or not is_finite(projected.y): return false
		origin += projected-viewport.get_center()
	if not is_finite(scale) or scale<=0 or not is_finite(origin.x) or not is_finite(origin.y): return false
	canvas.draw_texture_rect(textures[key],Rect2(origin,extent),false,Color.WHITE)
	return true