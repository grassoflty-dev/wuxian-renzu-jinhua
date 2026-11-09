extends RefCounted
const UiArt = preload('res://scripts/ui/ui_art_texture.gd')
const StyleBButtonArt = preload('res://scripts/ui/style_b_button_art.gd')
static var choice_material_attempted = false
static var choice_materials: Dictionary = {}
static var tooltip_material_attempted = false
static var tooltip_material: AtlasTexture
static var range_accent_attempted = false
static var range_accent_material: AtlasTexture
const BASE = {'background':'#0B121A','panel':'#16202A','panelRaised':'#1B2934','border':'#344652','text':'#E7EDF2','textMuted':'#8D9AA5','system':'#C4BDB8','accent':'#D9AD58','danger':'#E1504C','success':'#55C89C','focus':'#E5DDD6'}
static func install_tooltip_skin(control: Control) -> void:
	if not tooltip_material_attempted:
		tooltip_material_attempted = true
		tooltip_material = UiArt.atlas('res://assets/ui/global_ui_panel_b.png',Rect2(400,300,128,128))
	if tooltip_material == null:
		return
	var prior = control.get_theme_stylebox('panel','TooltipPanel')
	if prior == null:
		return
	var local_theme: Theme
	if control.theme == null:
		local_theme = Theme.new()
	else:
		local_theme = control.theme.duplicate() as Theme
	if local_theme == null:
		return
	var style = StyleBoxTexture.new()
	style.texture = tooltip_material
	style.content_margin_left = prior.get_margin(SIDE_LEFT)
	style.content_margin_top = prior.get_margin(SIDE_TOP)
	style.content_margin_right = prior.get_margin(SIDE_RIGHT)
	style.content_margin_bottom = prior.get_margin(SIDE_BOTTOM)
	local_theme.set_stylebox('panel','TooltipPanel',style)
	control.theme = local_theme
static func install_save_name_skin(entry: LineEdit) -> void:
	if not is_instance_valid(entry) or tooltip_material == null:
		return
	if not entry.has_theme_stylebox('normal','LineEdit') or not entry.has_theme_stylebox('read_only','LineEdit'):
		return
	var prior_normal = entry.get_theme_stylebox('normal','LineEdit')
	var prior_read_only = entry.get_theme_stylebox('read_only','LineEdit')
	if prior_normal == null or prior_read_only == null:
		return
	var normal_skin = StyleBoxTexture.new()
	var read_only_skin = StyleBoxTexture.new()
	for pair in [[normal_skin,prior_normal],[read_only_skin,prior_read_only]]:
		var skin: StyleBoxTexture = pair[0]
		var prior: StyleBox = pair[1]
		skin.texture = tooltip_material
		skin.content_margin_left = prior.get_margin(SIDE_LEFT)
		skin.content_margin_top = prior.get_margin(SIDE_TOP)
		skin.content_margin_right = prior.get_margin(SIDE_RIGHT)
		skin.content_margin_bottom = prior.get_margin(SIDE_BOTTOM)
	entry.add_theme_stylebox_override('normal',normal_skin)
	entry.add_theme_stylebox_override('read_only',read_only_skin)
static func _choice_materials() -> Dictionary:
	if not choice_material_attempted:
		choice_material_attempted = true
		var source = StyleBButtonArt.get_sheet()
		if source != null and source.get_width()>=1300 and source.get_height()>=712:
			for pair in [['normal',114],['hover',281],['pressed',449],['disabled',616]]:
				var atlas = AtlasTexture.new()
				atlas.atlas = source
				atlas.region = Rect2(236,pair[1],1064,96)
				choice_materials[pair[0]] = atlas
	return choice_materials

static func _choice_style(texture: Texture2D, prior: StyleBox) -> StyleBoxTexture:
	var style = StyleBoxTexture.new()
	style.texture = texture
	style.content_margin_left = prior.get_margin(SIDE_LEFT)
	style.content_margin_top = prior.get_margin(SIDE_TOP)
	style.content_margin_right = prior.get_margin(SIDE_RIGHT)
	style.content_margin_bottom = prior.get_margin(SIDE_BOTTOM)
	return style

static func install_native_button_backgrounds(button: Button) -> void:
	if not is_instance_valid(button):
		return
	var materials = _choice_materials()
	if materials.size()!=4:
		return
	var keys = ['normal','hover','pressed','hover_pressed','disabled']
	var prior = {}
	for key in keys:
		if not button.has_theme_stylebox(key):
			return
		var style = button.get_theme_stylebox(key)
		if style == null:
			return
		prior[key] = style
	var replacement = {}
	for key in keys:
		var material_key = 'pressed' if key=='hover_pressed' else key
		replacement[key] = _choice_style(materials[material_key],prior[key])
	for key in keys:
		button.add_theme_stylebox_override(key,replacement[key])

static func install_choice_skin(choice: OptionButton) -> void:
	if not is_instance_valid(choice):
		return
	var materials = _choice_materials()
	if materials.size()!=4:
		return
	install_native_button_backgrounds(choice)
	var popup = choice.get_popup()
	if not is_instance_valid(popup) or tooltip_material == null:
		return
	if popup.has_theme_stylebox_override('panel') or popup.has_theme_stylebox_override('hover'):
		return
	if not popup.has_theme_stylebox('panel','PopupMenu') or not popup.has_theme_stylebox('hover','PopupMenu'):
		return
	var popup_panel = popup.get_theme_stylebox('panel','PopupMenu')
	var popup_hover = popup.get_theme_stylebox('hover','PopupMenu')
	if popup_panel == null or popup_hover == null:
		return
	var popup_theme: Theme
	if popup.theme == null:
		popup_theme = Theme.new()
	else:
		popup_theme = popup.theme.duplicate() as Theme
	if popup_theme == null:
		return
	popup_theme.set_stylebox('panel','PopupMenu',_choice_style(tooltip_material,popup_panel))
	popup_theme.set_stylebox('hover','PopupMenu',_choice_style(materials.hover,popup_hover))
	popup.theme = popup_theme
static func _range_accent_texture() -> AtlasTexture:
	if not range_accent_attempted:
		range_accent_attempted = true
		range_accent_material = UiArt.atlas('res://assets/ui/global_ui_energy.png',Rect2(704,494,128,32))
	return range_accent_material

static func install_range_track(control: Control, style_name: String, vertical: bool) -> void:
	if not is_instance_valid(control):
		return
	_install_range_material(control,style_name,vertical,tooltip_material)
	if control.get_class()=='HSlider' and control.get_script()==null and style_name=='slider' and not vertical:
		_install_range_material(control,'grabber_area',false,_range_accent_texture())
	elif control.get_class()=='VScrollBar' and control.get_script()==null and style_name=='scroll' and vertical:
		_install_range_material(control,'grabber',true,_range_accent_texture())

static func _install_range_material(control: Control, style_name: String, vertical: bool, material: Texture2D) -> void:
	if not is_instance_valid(control) or material == null or not control.has_theme_stylebox(style_name):
		return
	var prior = control.get_theme_stylebox(style_name)
	if prior == null or not prior is StyleBoxFlat or prior.get_script() != null or prior.get_class() != 'StyleBoxFlat':
		return
	var flat: StyleBoxFlat = prior
	if flat.shadow_size != 0 or flat.shadow_offset != Vector2.ZERO:
		return
	var margins = []
	for side in [SIDE_LEFT,SIDE_TOP,SIDE_RIGHT,SIDE_BOTTOM]:
		var expansion = flat.get_expand_margin(side)
		var margin = flat.get_margin(side)
		if not is_finite(expansion) or expansion != 0.0 or not is_finite(margin) or margin<0.0:
			return
		margins.append(margin)
	var old_size = flat.get_minimum_size()
	if not is_finite(old_size.x) or not is_finite(old_size.y) or old_size.x<0.0 or old_size.y<0.0:
		return
	if (vertical and old_size.x<=0.0) or (not vertical and old_size.y<=0.0):
		return
	var skin = StyleBoxTexture.new()
	skin.texture = material
	skin.content_margin_left = margins[0]
	skin.content_margin_top = margins[1]
	skin.content_margin_right = margins[2]
	skin.content_margin_bottom = margins[3]
	var new_size = skin.get_minimum_size()
	if not is_finite(new_size.x) or not is_finite(new_size.y) or not is_equal_approx(old_size.x,new_size.x) or not is_equal_approx(old_size.y,new_size.y):
		return
	if (vertical and new_size.x<=0.0) or (not vertical and new_size.y<=0.0):
		return
	control.add_theme_stylebox_override(style_name,skin)
static func install_hud_information_panel(panel: PanelContainer) -> void:
	if not is_instance_valid(panel) or tooltip_material == null or not panel.has_theme_stylebox_override('panel') or not panel.has_theme_stylebox('panel'):
		return
	var prior = panel.get_theme_stylebox('panel')
	if prior == null or not prior is StyleBoxFlat or prior.get_script() != null or prior.get_class() != 'StyleBoxFlat':
		return
	var flat: StyleBoxFlat = prior
	if flat.shadow_size != 0 or flat.shadow_offset != Vector2.ZERO:
		return
	var margins = []
	for side in [SIDE_LEFT,SIDE_TOP,SIDE_RIGHT,SIDE_BOTTOM]:
		var expansion = flat.get_expand_margin(side)
		var margin = flat.get_margin(side)
		if not is_finite(expansion) or expansion != 0.0 or not is_finite(margin) or margin<0.0:
			return
		margins.append(margin)
	var old_size = flat.get_minimum_size()
	if not is_finite(old_size.x) or not is_finite(old_size.y) or old_size.x<0.0 or old_size.y<0.0:
		return
	var skin = StyleBoxTexture.new()
	skin.texture = tooltip_material
	skin.content_margin_left = margins[0]
	skin.content_margin_top = margins[1]
	skin.content_margin_right = margins[2]
	skin.content_margin_bottom = margins[3]
	var new_size = skin.get_minimum_size()
	if not is_finite(new_size.x) or not is_finite(new_size.y) or not is_equal_approx(old_size.x,new_size.x) or not is_equal_approx(old_size.y,new_size.y):
		return
	panel.add_theme_stylebox_override('panel',skin)
static func palette(world: String) -> Dictionary:
	var colors = BASE.duplicate()
	if world=='mist_harbor':
		colors.merge({'background':'#091721','panel':'#112936','panelRaised':'#1A3946','border':'#365866','textMuted':'#9AB4BE','system':'#82C9D4','accent':'#D8B879','focus':'#B2F3F5'},true)
	elif world=='clockworks':
		colors.merge({'background':'#17120E','panel':'#282018','panelRaised':'#382B1D','border':'#62503A','textMuted':'#B9A88D','system':'#D4A25D','accent':'#E08C45','focus':'#FFE0A1'},true)
	return colors
static func apply(control: Control, world: String) -> void:
	var colors = palette(world)
	var theme = Theme.new()
	theme.set_color('font_color','Label',Color(colors.text))
	theme.set_color('font_color','Button',Color(colors.text))
	for state in ['normal','hover','pressed','disabled','focus']:
		var style = StyleBoxFlat.new()
		style.bg_color = Color(colors.panelRaised if state in ['hover','pressed'] else colors.panel)
		style.border_color = Color(colors.focus if state=='focus' else colors.border)
		style.set_border_width_all(1)
		style.set_corner_radius_all(2)
		style.content_margin_left = 12
		style.content_margin_right = 12
		style.content_margin_top = 6
		style.content_margin_bottom = 6
		theme.set_stylebox(state,'Button',style)
	control.theme = theme
	install_tooltip_skin(control)
	if control.has_method('_install_skill_button_backgrounds'):
		control.call('_install_skill_button_backgrounds')
