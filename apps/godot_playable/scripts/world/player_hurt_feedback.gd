extends RefCounted
## Optional presentation-only transplant of web CombatPresentationLayer's hurt cue.
## Call only for current EventView.project() output, after any admitted flipbook.
## No queue, stored event, transform write, atlas, new timer or gameplay authority.
const P = preload('res://scripts/bridge/protocol.gd')
const DURATION_MS = 260
const BASE_PIXELS_PER_METER = 48.0

static func draw_accepted(canvas: CanvasItem, event: Dictionary, snapshot: Dictionary, contact: Vector2, pixels_per_meter: float, reduced_motion: bool, now_ms: int) -> bool:
	# True means handled, including an accepted event that expired between
	# EventView.project() and this draw. Never flash its legacy cross afterward.
	# False leaves the renderer's existing fallback untouched.
	if event.get('kind')!='Damaged' or not P.counter(event.get('protocolVersion'),2,2):
		return false
	var feedback = event.get('combatFeedback')
	if not feedback is Dictionary or feedback.get('outcome')!='player_hurt' or feedback.get('targetId')!='player' or typeof(feedback.get('sourceId'))!=TYPE_STRING:
		return false
	if not P.context(snapshot) or feedback.get('worldId')!=snapshot.worldId or feedback.get('sceneId')!=snapshot.sceneId:
		return false
	if not P.counter(event.get('eventId'),1) or not P.counter(event.get('worldEpoch')) or event.worldEpoch!=snapshot.worldEpoch or not P.counter(event.get('serverTick')) or not P.counter(snapshot.get('serverTick')) or event.serverTick>snapshot.serverTick:
		return false
	if not P.counter(event.get('durationMs'),DURATION_MS,DURATION_MS) or not P.number(event.get('_duration'),DURATION_MS,DURATION_MS) or not P.number(event.get('_initial_age'),0,DURATION_MS) or not P.counter(event.get('_accepted')) or now_ms<event._accepted:
		return false
	# Same age expression as EventView. Reading the accepted timestamp does not
	# start/restart a clock; source expiry and cancellation remain upstream.
	var age_ms = maxf((snapshot.serverTick-event.serverTick)*1000.0/60.0,float(event._initial_age)+float(now_ms-event._accepted))
	if age_ms>=DURATION_MS:
		return true
	if not is_instance_valid(canvas) or not is_finite(contact.x) or not is_finite(contact.y) or not is_finite(pixels_per_meter) or pixels_per_meter<=0.0:
		return false
	var scale = pixels_per_meter/BASE_PIXELS_PER_METER
	# contact is camera.project(event.positionM), the historical resolved root.
	# The web's fixed screen-space lift is a badge offset, not a torso socket.
	var center = contact-Vector2(0,18)*scale
	if not is_finite(center.x) or not is_finite(center.y) or not is_finite(14.0*scale):
		return false
	var progress = clampf(age_ms/float(DURATION_MS),0.0,1.0)
	var opacity = 1.0 if reduced_motion else 1.0-progress*0.55
	var backing = Color('#101821')
	backing.a = 0.85*opacity
	var outline = Color('#ff8c86')
	outline.a = opacity
	# Same six vertices and 2.5 px stroke as the web player_hurt branch.
	# No enemy-hit spread, body tint, shake, particles or hit-region semantics.
	var points = PackedVector2Array([
		center+Vector2(-11,-4)*scale,
		center+Vector2(-4,-10)*scale,
		center+Vector2(1,-2)*scale,
		center+Vector2(8,-9)*scale,
		center+Vector2(11,4)*scale,
		center+Vector2(3,11)*scale,
		center+Vector2(-11,-4)*scale
	])
	canvas.draw_circle(center,14.0*scale,backing)
	canvas.draw_polyline(points,outline,2.5*scale,true)
	return true
