extends RefCounted
# Pure decision table for leaving a terminal client hold (`uncertain`). The caller passes
# only facts it already validates; nothing here mutates, requests or resends anything.
# Keeping the rule side-effect free lets an offline contract test pin it.
static func ready(f: Dictionary) -> bool:
	if typeof(f)!=TYPE_DICTIONARY: return false
	if f.get('uncertain')!=true or f.get('journey_started')!=true: return false
	if f.get('error_page')!=true or f.get('panel_pending')!=false: return false
	if f.get('authenticated')!=true or f.get('focused')!=true: return false
	if f.get('critical_empty')!=true: return false
	if f.get('snapshot_valid')!=true or f.get('status_valid')!=true or f.get('context_same')!=true: return false
	if f.get('entry_token_absent')!=true: return false
	if f.get('paused')!=false or f.get('alive')!=true: return false
	if f.get('preview_absent')!=true or f.get('ready_absent')!=true or f.get('ticket_absent')!=true: return false
	if f.get('renderer_valid')!=true or f.get('dense_ready')!=true: return false
	for key in ['hold_at_ms','last_snapshot_at','last_status_at','now_ms']:
		var value = f.get(key)
		if typeof(value)!=TYPE_INT and typeof(value)!=TYPE_FLOAT: return false
		if not is_finite(float(value)) or float(value)<0.0: return false
	if float(f.hold_at_ms)<=0.0: return false
	# The confirming pair must be NEWER than the hold itself: late pre-hold polls do not count.
	if float(f.last_snapshot_at)<=float(f.hold_at_ms) or float(f.last_status_at)<=float(f.hold_at_ms): return false
	if float(f.now_ms)-float(f.last_snapshot_at)>=1000.0 or float(f.now_ms)-float(f.last_status_at)>=1000.0: return false
	return true
