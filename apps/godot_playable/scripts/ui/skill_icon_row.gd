extends HBoxContainer
const LABELS = {'pulse':'Q · 脉冲','guard':'E · 防御','pierce':'R · 穿刺','dash':'Shift · 冲刺'}
var slots = {}
func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	for id in ['pulse','guard','pierce','dash']:
		var box = VBoxContainer.new()
		box.name = id.capitalize()
		box.mouse_filter = Control.MOUSE_FILTER_IGNORE
		add_child(box)
		var icon = TextureRect.new()
		icon.name = 'Icon'
		icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
		icon.mouse_filter = Control.MOUSE_FILTER_IGNORE
		icon.focus_mode = Control.FOCUS_NONE
		icon.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
		icon.texture_repeat = CanvasItem.TEXTURE_REPEAT_DISABLED
		icon.custom_minimum_size = Vector2(56,56)
		box.add_child(icon)
		var text = Label.new()
		text.text = LABELS[id]
		text.mouse_filter = Control.MOUSE_FILTER_IGNORE
		box.add_child(text)
		var phase = Label.new()
		phase.mouse_filter = Control.MOUSE_FILTER_IGNORE
		phase.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		phase.custom_minimum_size.x = 56
		phase.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
		box.add_child(phase)
		slots[id] = {'icon':icon,'text':text,'phase':phase}
func apply_admitted_mapping(mapping: Dictionary) -> void:
	# Explicit future intake only. No private lookup, implicit preload or admission from existence.
	for id in slots:
		slots[id].icon.texture = null
		var row = mapping.get(id)
		var expected = 'res://assets/ui/skills/'+id+'/icon_128.png'
		if not row is Dictionary or typeof(row.get('admitted'))!=TYPE_BOOL or not row.admitted or row.get('path')!=expected or typeof(row.get('sha256'))!=TYPE_STRING or row.sha256.length()!=64 or typeof(row.get('sourceRevision'))!=TYPE_STRING or row.sourceRevision.is_empty() or row.get('approval')!='INTERNAL_SOURCE_ONLY' or row.get('runtimeAdmission')!='NOT_VALIDATED' or row.get('upstreamRuntimeAdmission')!='NOT_ADMITTED' or typeof(row.get('releaseEligible'))!=TYPE_BOOL or row.releaseEligible:
			continue
		if FileAccess.file_exists(expected) and FileAccess.get_sha256(expected)==row.sha256:
			var file = FileAccess.open(expected,FileAccess.READ)
			if file == null:
				continue
			if file.get_length()>65536:
				file.close()
				continue
			var bytes = file.get_buffer(file.get_length())
			file.close()
			var image = Image.new()
			if image.load_png_from_buffer(bytes)==OK and image.get_width()==128 and image.get_height()==128 and image.get_format()==Image.FORMAT_RGBA8:
				var texture = ImageTexture.create_from_image(image)
				if texture != null:
					slots[id].icon.texture = texture
func update_display(scale: float, action: String, eligible: bool, status: Dictionary = {}) -> void:
	for id in slots:
		slots[id].icon.custom_minimum_size = Vector2(56,56)*scale
		slots[id].text.add_theme_font_size_override('font_size',int(16*scale))
		slots[id].text.text = LABELS[id]+(' · 当前' if action==id else '')+(' · 暂停输入' if not eligible else '')
		slots[id].phase.add_theme_font_size_override('font_size',int(14*scale))
		slots[id].phase.custom_minimum_size.x = 56*scale
		slots[id].phase.text = status.text if eligible and action==id and status.get('kind')==id and typeof(status.get('text'))==TYPE_STRING else ''
