extends RefCounted
## Isolated candidate: presentation only. No file I/O, clock, event or gameplay authority.
## Caller owns source admission/hash checks, clip start, cancellation and draw ordering.
const SCHEMA = 'wuxian.frame-atlas-candidate/v1'
const MAX_CLIPS = 128
const MAX_FRAMES = 128
var _texture: Texture2D
var _size = Vector2.ZERO
var _clips: Dictionary = {}

func clear() -> void:
	_texture = null
	_size = Vector2.ZERO
	_clips.clear()

func _number(value, low: float, high: float) -> bool:
	return (typeof(value)==TYPE_INT or typeof(value)==TYPE_FLOAT) and is_finite(float(value)) and float(value)>=low and float(value)<=high

func _pair(value, low: float, high: float) -> bool:
	return value is Array and value.size()==2 and _number(value[0],low,high) and _number(value[1],low,high)

func _integer(value, low: int, high: int) -> bool:
	return _number(value,low,high) and float(value)==floor(float(value))

func _identifier(value) -> bool:
	if typeof(value)!=TYPE_STRING or value.is_empty() or value.length()>64:
		return false
	for c in value:
		if not c in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-':
			return false
	return true

func configure(metadata: Dictionary, texture: Texture2D) -> bool:
	# Fail closed: a failed reconfiguration cannot keep an old atlas alive.
	clear()
	if texture==null or metadata.get('schema')!=SCHEMA:
		return false
	var size = metadata.get('atlas_size')
	if not _pair(size,1,8192) or not _integer(size[0],1,8192) or not _integer(size[1],1,8192):
		return false
	var atlas_size = Vector2(float(size[0]),float(size[1]))
	if texture.get_size()!=atlas_size:
		return false
	var clips = metadata.get('clips')
	if not clips is Array or clips.is_empty() or clips.size()>MAX_CLIPS:
		return false
	var checked: Dictionary = {}
	for clip in clips:
		if not clip is Dictionary or not _identifier(clip.get('id')) or not _identifier(clip.get('direction')) or typeof(clip.get('loop'))!=TYPE_BOOL:
			return false
		var key: String = clip.id+':'+clip.direction
		if checked.has(key):
			return false
		var frames = clip.get('frames')
		if not frames is Array or frames.is_empty() or frames.size()>MAX_FRAMES:
			return false
		var compiled: Array = []
		var total = 0.0
		var canvas_size = Vector2.ZERO
		var root = Vector2.ZERO
		for frame in frames:
			if not frame is Dictionary or not _number(frame.get('duration_ms'),1,60000):
				return false
			var rect = frame.get('rect')
			if not rect is Array or rect.size()!=4:
				return false
			for n in range(4):
				if not _integer(rect[n],0 if n<2 else 1,8192):
					return false
			var source = Rect2(float(rect[0]),float(rect[1]),float(rect[2]),float(rect[3]))
			if source.end.x>atlas_size.x or source.end.y>atlas_size.y or not _pair(frame.get('anchor'),0,8192):
				return false
			var anchor = Vector2(float(frame.anchor[0]),float(frame.anchor[1]))
			if anchor.x>source.size.x or anchor.y>source.size.y:
				return false
			# Full, registered canvases only. Never fit each silhouette independently.
			if compiled.is_empty():
				canvas_size = source.size
				root = anchor
			elif source.size!=canvas_size or anchor!=root:
				return false
			# Repeating an identical atlas region is not a new motion frame.
			for prior in compiled:
				if prior.rect==source:
					return false
			total += float(frame.duration_ms)
			if total>60000:
				return false
			compiled.append({'rect':source,'anchor':anchor,'end_ms':total})
		checked[key] = {'frames':compiled,'duration_ms':total,'loop':clip.loop}
	_texture = texture
	_size = atlas_size
	_clips = checked
	return true

func sample(clip_id: String, direction: String, elapsed_ms: float, require_motion: bool = true) -> Dictionary:
	if _texture==null or not is_finite(elapsed_ms) or elapsed_ms<0 or not _clips.has(clip_id+':'+direction):
		return {}
	var clip: Dictionary = _clips[clip_id+':'+direction]
	if require_motion and clip.frames.size()<2:
		return {}
	var time = elapsed_ms
	if clip.loop:
		time = fmod(time,float(clip.duration_ms))
	elif time>=float(clip.duration_ms):
		return {}
	for index in range(clip.frames.size()):
		var frame: Dictionary = clip.frames[index]
		if time<float(frame.end_ms):
			return {'rect':frame.rect,'anchor':frame.anchor,'frame_index':index,'duration_ms':clip.duration_ms}
	return {}

func draw_clip(canvas: CanvasItem, clip_id: String, direction: String, elapsed_ms: float, foot: Vector2, uniform_scale: float, tint: Color = Color.WHITE, require_motion: bool = true) -> bool:
	if not is_instance_valid(canvas) or not is_finite(foot.x) or not is_finite(foot.y) or not is_finite(uniform_scale) or uniform_scale<=0 or uniform_scale>64:
		return false
	for component in [tint.r,tint.g,tint.b,tint.a]:
		if not is_finite(component) or component<0 or component>1:
			return false
	var packet = sample(clip_id,direction,elapsed_ms,require_motion)
	if packet.is_empty() or _texture.get_size()!=_size:
		return false
	var rect: Rect2 = packet.rect
	canvas.draw_texture_rect_region(_texture,Rect2(foot-packet.anchor*uniform_scale,rect.size*uniform_scale),rect,tint,false,true)
	return true
