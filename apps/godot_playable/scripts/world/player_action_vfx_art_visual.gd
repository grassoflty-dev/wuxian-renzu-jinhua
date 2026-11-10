extends RefCounted

# The caller owns state, visibility, timing, camera and legacy fallback.
const PATH := "res://assets/vfx/player_attack_pulse_pierce_atlas_local_r1.png"
const SIZE := Vector2(1536.0, 1024.0)
const STEPS := 24
var _attempted := false
var _texture: Texture2D

func _valid_atlas(candidate: Texture2D) -> bool:
	if candidate==null or candidate.get_size()!=SIZE: return false
	var image:=candidate.get_image()
	return image!=null and not image.is_empty() and image.get_size()==Vector2i(1536,1024) and image.get_format()==Image.FORMAT_RGBA8

func _atlas() -> Texture2D:
	if _attempted: return _texture
	_attempted=true
	var imported: Texture2D=load(PATH) as Texture2D
	if _valid_atlas(imported):
		_texture=imported
		return _texture
	var raw:=Image.new()
	if raw.load(PATH)==OK and not raw.is_empty() and raw.get_size()==Vector2i(1536,1024) and raw.get_format()==Image.FORMAT_RGBA8:
		var candidate:=ImageTexture.create_from_image(raw)
		if _valid_atlas(candidate): _texture=candidate
	return _texture
# project accepts a Vector3 and returns a Vector2 in canvas-local coordinates.
# kind: basic, pulse, pierce; phase: windup, active, recovery.
# range_m and line_half_width_m are world units; opacity includes recovery fade.
# Returns false without drawing if any input, projection or resource is invalid.
func draw_effect(canvas: CanvasItem, project: Callable, origin: Vector3, facing: Vector2, kind: String, phase: String, range_m: float, line_half_width_m: float, pulse_radius_factor: float, opacity: float) -> bool:
	if not is_instance_valid(canvas) or not project.is_valid() or not origin.is_finite() or not facing.is_finite() or facing.length_squared() < 0.000001:
		return false
	if kind not in ["basic", "pulse", "pierce"] or phase not in ["windup", "active", "recovery"]:
		return false
	if not is_finite(range_m) or range_m <= 0.0 or not is_finite(line_half_width_m) or line_half_width_m < 0.0 or (kind == "pierce" and line_half_width_m <= 0.0) or not is_finite(pulse_radius_factor) or pulse_radius_factor <= 0.0 or not is_finite(opacity) or opacity < 0.0 or opacity > 1.0:
		return false
	var tex := _atlas()
	if tex == null:
		return false
	var direction := facing.normalized()
	var lateral := Vector2(-direction.y, direction.x)
	var points := PackedVector2Array()
	var uvs := PackedVector2Array()
	var cell_x := {"basic": 0.0, "pulse": 512.0, "pierce": 1024.0}[kind] as float
	var cell_y := 0.0 if phase == "windup" else 512.0
	var local_points: Array[Vector2] = []
	var local_uvs: Array[Vector2] = []
	if kind == "pierce":
		local_points = [Vector2(0.0, -line_half_width_m), Vector2(range_m, -line_half_width_m), Vector2(range_m, line_half_width_m), Vector2(0.0, line_half_width_m)]
		local_uvs = [Vector2(20, 168), Vector2(500, 168), Vector2(500, 304), Vector2(20, 304)]
	else:
		var radius := range_m if kind == "basic" else range_m * pulse_radius_factor
		local_points.append(Vector2.ZERO)
		local_uvs.append(Vector2(208, 256) if kind == "basic" else Vector2(256, 256))
		var start := -PI / 3.0 if kind == "basic" else -PI
		var sweep := 2.0 * PI / 3.0 if kind == "basic" else TAU
		for i in range(STEPS + 1):
			var angle := start + sweep * float(i) / float(STEPS)
			var unit := Vector2(cos(angle), sin(angle))
			local_points.append(unit * radius)
			local_uvs.append((Vector2(208, 256) if kind == "basic" else Vector2(256, 256)) + unit * 290.0)
	for i in range(local_points.size()):
		var p := local_points[i]
		var world_xz := direction * p.x + lateral * p.y
		var result: Variant = project.call(origin + Vector3(world_xz.x, 0.0, world_xz.y))
		if not result is Vector2 or not (result as Vector2).is_finite():
			return false
		points.append(result)
		var uv := local_uvs[i]
		uvs.append(Vector2(cell_x + clampf(uv.x, 0.5, 511.5), cell_y + clampf(uv.y, 0.5, 511.5)) / SIZE)
	var colors := PackedColorArray([Color(1, 1, 1, opacity)])
	if kind == "pierce":
		canvas.draw_polygon(points, colors, uvs, tex)
	else:
		for i in range(1, STEPS + 1):
			canvas.draw_polygon(PackedVector2Array([points[0], points[i], points[i + 1]]), colors, PackedVector2Array([uvs[0], uvs[i], uvs[i + 1]]), tex)
	return true
