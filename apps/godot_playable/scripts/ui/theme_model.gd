extends RefCounted
const BASE = {'background':'#0B121A','panel':'#16202A','panelRaised':'#1B2934','border':'#344652','text':'#E7EDF2','textMuted':'#8D9AA5','system':'#58D7F0','accent':'#D9AD58','danger':'#E1504C','success':'#55C89C','focus':'#9FEEFF'}
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
