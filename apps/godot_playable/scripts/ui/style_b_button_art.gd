extends Control
const UiArt = preload('res://scripts/ui/ui_art_texture.gd')
static var sheet: Texture2D
static var attempted = false
var panel_owner
var font_scale = 1.0
var button: Button
var active = false
var state_code = -1
var last_width = -1.0
var original_minimum: Vector2
var saved_styles = {}
var empty_style: StyleBoxEmpty
var expected_height = 0.0

static func get_sheet() -> Texture2D:
	if not attempted:
		attempted = true
		var atlas = UiArt.atlas('res://assets/ui/global_ui_button_states_b.png',Rect2(0,0,1536,1024))
		if atlas != null:
			sheet = atlas.atlas
	return sheet
func _ready() -> void:
	button = get_parent() as Button
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	show_behind_parent = true
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	get_sheet()
	original_minimum = button.custom_minimum_size
	for key in ['normal','hover','pressed','hover_pressed','disabled','focus']:
		saved_styles[key] = button.get_theme_stylebox(key) if button.has_theme_stylebox_override(key) else null
	empty_style = StyleBoxEmpty.new()
	button.resized.connect(_width_changed)
	_width_changed()

func _width_changed() -> void:
	if not is_instance_valid(button):
		return
	var width = button.size.x
	if absf(width-last_width)<0.5:
		return
	last_width = width
	var font = button.get_theme_font('font')
	var font_size = button.get_theme_font_size('font_size')
	if sheet == null or not is_finite(width) or width<160.0*font_scale or font == null or font_size<=0:
		_restore()
		return
	expected_height = maxf(52.0*font_scale,(font.get_height(font_size)+6.0*font_scale)*176.0/96.0)
	var q = expected_height/176.0
	var middle = (width-144.0*q)/1080.0
	if middle<=0:
		_restore()
		return
	var horizontal = maxf(72.0*q+8.0*middle,24.0*font_scale)
	empty_style.content_margin_left = horizontal
	empty_style.content_margin_right = horizontal
	empty_style.content_margin_top = 40.0*q
	empty_style.content_margin_bottom = 40.0*q
	button.custom_minimum_size.y = expected_height
	for key in ['normal','hover','pressed','hover_pressed','disabled','focus']:
		button.add_theme_stylebox_override(key,empty_style)
	active = true
	call_deferred('_verify_geometry')
	queue_redraw()

func _verify_geometry() -> void:
	if active and is_instance_valid(button) and (absf(button.size.y-expected_height)>1.0 or button.size.x<160.0*font_scale):
		_restore()

func _restore() -> void:
	if not is_instance_valid(button):
		return
	active = false
	button.custom_minimum_size = original_minimum
	for key in ['normal','hover','pressed','hover_pressed','disabled','focus']:
		if saved_styles[key] == null:
			button.remove_theme_stylebox_override(key)
		else:
			button.add_theme_stylebox_override(key,saved_styles[key])
	queue_redraw()

func _process(_delta: float) -> void:
	if not active or not is_instance_valid(button):
		return
	var draw_mode = button.get_draw_mode()
	var disabled = button.disabled or draw_mode == BaseButton.DRAW_DISABLED or (is_instance_valid(panel_owner) and panel_owner.pending)
	var code = 3 if disabled else (2 if draw_mode == BaseButton.DRAW_PRESSED or draw_mode == BaseButton.DRAW_HOVER_PRESSED else (1 if draw_mode == BaseButton.DRAW_HOVER else 0))
	if not disabled and button.has_focus():
		code += 4
	if code!=state_code:
		state_code = code
		queue_redraw()

func _draw() -> void:
	if not active or sheet == null or not is_instance_valid(button):
		return
	var width = size.x
	var height = size.y
	var q = height/176.0
	var cap = 72.0*q
	if width<=2.0*cap:
		return
	var row = 74
	var base = state_code % 4 if state_code>=0 else 0
	if base==1:
		row = 241
	elif base==2:
		row = 409
	elif base==3:
		row = 576
	_draw_horizontal(row,0,176,0.0,height,cap,width)
	if state_code>=4:
		_draw_horizontal(746,0,40,0.0,40.0*q,cap,width)
		_draw_horizontal(746,136,40,136.0*q,40.0*q,cap,width)
		draw_texture_rect_region(sheet,Rect2(0,40.0*q,cap,96.0*q),Rect2(156,786,72,96))
		draw_texture_rect_region(sheet,Rect2(width-cap,40.0*q,cap,96.0*q),Rect2(1308,786,72,96))

func _draw_horizontal(row: int, source_y: int, source_h: int, dest_y: float, dest_h: float, cap: float, width: float) -> void:
	draw_texture_rect_region(sheet,Rect2(0,dest_y,cap,dest_h),Rect2(156,row+source_y,72,source_h))
	draw_texture_rect_region(sheet,Rect2(cap,dest_y,width-2.0*cap,dest_h),Rect2(228,row+source_y,1080,source_h))
	draw_texture_rect_region(sheet,Rect2(width-cap,dest_y,cap,dest_h),Rect2(1308,row+source_y,72,source_h))
