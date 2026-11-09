extends Control
const UiArt = preload('res://scripts/ui/ui_art_texture.gd')
# Optional caller-supplied opaque art; no resource is loaded by this view.
var art: Texture2D = null
var world_title: String = '正在进入世界'
# Optional authoritative caller-supplied lore; empty by default.
var short_lore: String = ''
var matte: Texture2D = null
func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	matte = UiArt.atlas('res://assets/ui/global_ui_panel_b.png',Rect2(400,300,128,128))
	visible = false
	set_process(false)
func begin_loading(title: String) -> void:
	short_lore = ''
	world_title = title if not title.is_empty() else '正在进入世界'
	visible = true
	queue_redraw()
func end_loading() -> void:
	visible = false
	set_process(false)
func set_world_title(value: String) -> void:
	world_title = value if not value.is_empty() else '正在进入世界'
	queue_redraw()
func set_short_lore(value: String) -> void:
	short_lore = value.strip_edges()
	queue_redraw()
func set_loading_art(value: Texture2D) -> void:
	art = value
	queue_redraw()
func _draw() -> void:
	if size.x<=0.0 or size.y<=0.0:
		return
	if is_instance_valid(matte):
		draw_texture_rect(matte,Rect2(Vector2.ZERO,size),false)
	else:
		draw_rect(Rect2(Vector2.ZERO,size),Color('#111e29'))
	if is_instance_valid(art):
		var source = art.get_size()
		if source.x>0.0 and source.y>0.0 and is_finite(source.x) and is_finite(source.y):
			var scale_factor = minf(size.x/source.x,size.y/source.y)
			var fitted = source*scale_factor
			draw_texture_rect(art,Rect2((size-fitted)*0.5,fitted),false)
	# Bottom safe band scales down on small controls; art remains centered CONTAIN.
	var band_height = minf(128.0,size.y)
	var band_scale = band_height/128.0
	var padding = minf(32.0,size.x*0.1)
	var band = Rect2(Vector2(0,size.y-band_height),Vector2(size.x,band_height))
	if is_instance_valid(matte):
		draw_texture_rect(matte,band,false)
	else:
		draw_rect(band,Color(0.02,0.04,0.06,0.84))
	var font = ThemeDB.fallback_font
	var text_width = maxf(0.0,size.x-2.0*padding)
	if font != null and text_width>0.0:
		var title_size = maxi(1,roundi(20.0*band_scale))
		var lore_size = maxi(1,roundi(16.0*band_scale))
		draw_multiline_string(font,Vector2(padding,size.y-100.0*band_scale),world_title,HORIZONTAL_ALIGNMENT_LEFT,text_width,title_size,1,Color('#dce7ec'))
		if not short_lore.is_empty():
			draw_multiline_string(font,Vector2(padding,size.y-66.0*band_scale),short_lore,HORIZONTAL_ALIGNMENT_LEFT,text_width,lore_size,2,Color('#c2d2da'))
