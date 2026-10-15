extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const TEXTURE_PATH = 'res://assets/world/rs_core_floor_base_r2.png'
enum State { UNTRIED, FAILED, READY }
var state = State.UNTRIED
# Retain the imported texture for deferred CanvasItem drawing.
var texture: Texture2D
func matches_context(snapshot: Dictionary, scene: Dictionary) -> bool:
	if snapshot.get('worldId')!='return_station' or snapshot.get('sceneId')!='rs_core_room' or scene.get('worldId')!='return_station' or scene.get('sceneId')!='rs_core_room':
		return false
	var bounds = scene.get('boundsM')
	return bounds is Dictionary and P.number(bounds.get('x'),0,0) and P.number(bounds.get('z'),0,0) and P.number(bounds.get('width'),24,24) and P.number(bounds.get('depth'),16,16)
func _load_once() -> void:
	if state!=State.UNTRIED:
		return
	state = State.FAILED
	# Use Godot's imported-resource remapping, including in exported builds.
	if not ResourceLoader.exists(TEXTURE_PATH,'Texture2D'):
		return
	var candidate = ResourceLoader.load(TEXTURE_PATH,'Texture2D')
	if not candidate is Texture2D or candidate.get_width()!=1536 or candidate.get_height()!=1024:
		return
	texture = candidate
	state = State.READY
func draw_floor(canvas: CanvasItem, snapshot: Dictionary, scene: Dictionary, points: PackedVector2Array) -> bool:
	# Other scenes/bounds must not trigger resource IO, even after a successful load.
	if canvas==null or not matches_context(snapshot,scene) or points.size()!=4:
		return false
	for point in points:
		if not is_finite(point.x) or not is_finite(point.y):
			return false
	_load_once()
	if state!=State.READY:
		return false
	# The renderer supplies the existing y=0 floor corners in this same order.
	# Whole-image coverage once; inherit LINEAR filtering and REPEAT_DISABLED.
	var uvs = PackedVector2Array([Vector2(0,0),Vector2(1,0),Vector2(1,1),Vector2(0,1)])
	canvas.draw_colored_polygon(points,Color.WHITE,uvs,texture)
	return true
