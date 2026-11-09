extends SceneTree

var checks: int = 0
var failures: int = 0
func check(ok: bool, label: String) -> void:
	checks += 1
	if not ok:
		failures += 1
		printerr('FAIL '+label)
func source(path: String) -> String:
	return FileAccess.get_file_as_string(path).replace('\r\n','\n')
func function_text(text: String, name: String) -> String:
	var start: int = text.find('func '+name+'(')
	if start<0: return ''
	var end: int = text.find('\nfunc ',start+1)
	return text.substr(start) if end<0 else text.substr(start,end-start)
func compile_text(text: String) -> GDScript:
	var script := GDScript.new()
	script.source_code = text
	check(script.reload()==OK,'extracted actual functions compile')
	return script
func _initialize() -> void:
	var renderer: String = source('res://scripts/world/renderer.gd')
	var power: String = source('res://scripts/world/gh_power_environment_visual.gd')
	var cenyao: String = source('res://scripts/world/cenyao_idle_visual.gd')
	for path in ['renderer','gh_power_environment_visual','cenyao_idle_visual']:
		var full_script: Script = load('res://scripts/world/'+path+'.gd')
		check(full_script is GDScript and full_script.can_instantiate(),'FULL resource compile '+path)
	var methods: Array[String] = ['_gh_gate_a_open_barrier','_gh_gate_b_open_barrier','_gh_lockdown_open_seal','_gh_exit_open_barrier']
	var bodies: String = function_text(renderer,'_gh_state_context')+'\n'+function_text(renderer,'_gh_state_authored_position')+'\n'
	var all_present: bool = true
	var draw_source: String = function_text(renderer,'_draw')
	for method in methods:
		var body: String = function_text(renderer,method)
		check(not body.is_empty(),'actual helper present '+method)
		check(draw_source.count('\t\t\tif '+method+'(wall):\n\t\t\t\tcontinue\n')==1,'actual collision fallback skip wiring '+method)
		all_present = all_present and not body.is_empty()
		bodies += body+'\n'
	if all_present:
		var script: GDScript = compile_text("extends RefCounted\nconst P=preload('res://scripts/bridge/protocol.gd')\nconst Content=preload('res://scripts/ui/content_model.gd')\nvar scene: Dictionary={}\nvar snapshot: Dictionary={}\nvar current_key: String=''\n"+bodies)
		if script.can_instantiate():
			var actual = script.new()
			for index in range(methods.size()):
				var id: String = ['gh_gate_a','gh_gate_b','gh_lockdown','gh_exit'][index]
				var wall_id: String = ['gh_gate_a_closed_barrier','gh_gate_b_gate_barrier','gh_lockdown_bio_seal','exit_sealed_extraction_barrier'][index]
				var scene: Dictionary = JSON.parse_string(source('res://data/scenes/'+id+'.json'))
				var walls: Array = scene.collision.filter(func(row): return row.id==wall_id)
				check(walls.size()==1,'canonical authored barrier '+id)
				var wall: Dictionary = walls[0]
				var progress: Dictionary = {'worldId':'grey_hive','completed':false,'firstCompletion':false,'revisitCount':0,'visitId':1,'cycleId':1,'completedEvents':[]}
				var snapshot: Dictionary = {'worldId':'grey_hive','sceneId':id,'worldEpoch':3,'doors':[], 'progression':{'schemaVersion':1,'eventSeq':0,'currentWorldId':'grey_hive','worlds':[progress]}}
				if index<2:
					var authored: Dictionary = scene.doors[0]
					var p: Array = authored.position
					snapshot.doors=[{'doorId':id,'open':false,'locked':true,'transform':{'positionM':{'xM':p[0],'yM':p[1],'zM':p[2]},'yawRad':0.0}}]
				actual.scene=scene
				actual.snapshot=snapshot
				actual.current_key='grey_hive:'+id+':3'
				check(not actual.call(methods[index],wall),'closed keeps strip '+id)
				if index<2:
					snapshot.doors[0].open=true
					snapshot.doors[0].locked=false
				else:
					progress.completedEvents=[wall.requiresEvent]
				check(actual.call(methods[index],wall),'open suppresses strip '+id)
				actual.current_key='grey_hive:'+id+':2'
				check(not actual.call(methods[index],wall),'stale epoch rejected '+id)
				actual.current_key='grey_hive:'+id+':3'
				var forged: Dictionary = wall.duplicate(true)
				forged.polygon[0][0]+=0.25
				check(not actual.call(methods[index],forged),'noncanonical strip rejected '+id)
				if index<2:
					snapshot.doors[0].locked=true
					check(not actual.call(methods[index],wall),'locked native door retains strip '+id)
					snapshot.doors[0].locked=false
					snapshot.doors.append(snapshot.doors[0].duplicate(true))
					check(not actual.call(methods[index],wall),'duplicate native door rejected '+id)
				else:
					progress.visitId=0
					check(not actual.call(methods[index],wall),'unvisited event cannot open '+id)
	# Run actual packet selection with only geometry/layout observation boundaries inert.
	var power_header: String = '''extends RefCounted
const WALLS=[]
const MAX_PACKETS=160
var valid=true
var light_ok=true
func _ensure_layout(_a,_b,_c): return valid
func _projection_valid(_p): return valid
func _pipe_anchor_valid(_s): return false
func _add_wall(_a,_b,_c): return true
func _build_outer_wall_packet(_a,_b,_c,_d,_e): return null
func _add_pipe_detail(_a,_b): return true
func _add_contact_shadow(_a,_b): return true
func _add_task_light(p,_projection):
	p.append({'role':'task_light'})
	return true
func _add_ground_ellipse(p,_id,role,_center,_rx,_rz,_color,_projection,_source):
	p.append({'role':role})
	return light_ok
'''
	var power_script: GDScript = compile_text(power_header+function_text(power,'build_packets')+function_text(power,'_add_off_emergency_light'))
	if power_script.can_instantiate():
		var actual = power_script.new()
		check(actual.build_packets({},{},'',null,'off')==[{'role':'off_emergency_light'}],'power OFF reaches actual emergency light branch')
		check(actual.build_packets({},{},'',null,'on')==[{'role':'task_light'}],'power ON retains task light only')
		check(actual.build_packets({},{},'',null,'unknown').is_empty(),'unknown light does not invent lighting')
		actual.light_ok=false
		check(actual.build_packets({},{},'',null,'off').is_empty(),'emergency construction failure fails closed')
		actual.valid=false
		check(actual.build_packets({},{},'',null,'off').is_empty(),'invalid layout/projection stays rejected')
	var pin_line: String = ''
	for line in function_text(cenyao,'draw_entry_contact_shadow').split('\n'):
		if line.begins_with("\tif scene.get('worldId')") and line.contains('scene_hash!='): pin_line=line
	check(not pin_line.is_empty(),'actual contact scene pin guard present')
	var manifest: Dictionary = JSON.parse_string(source('res://data/scene_manifest.json'))
	var expected: String = manifest.gh_entry_maintenance.sha256
	var pin: String = pin_line.get_slice("scene_hash!='",1).get_slice("'",0)
	check(pin.length()==64 and pin==expected,'CenYao pin is 64 chars and matches admitted scene catalog')
	if not pin_line.is_empty():
		var pin_script: GDScript = compile_text('extends RefCounted\nfunc accepts(scene,snapshot,scene_hash,current_key):\n'+pin_line+'\n\t\treturn false\n\treturn true\n')
		if pin_script.can_instantiate():
			var actual = pin_script.new()
			var scene: Dictionary = {'worldId':'grey_hive','sceneId':'gh_entry_maintenance'}
			check(actual.accepts(scene,{'worldEpoch':3},expected,'grey_hive:gh_entry_maintenance:3'),'canonical scene hash admitted')
			check(not actual.accepts(scene,{'worldEpoch':3},expected+'0','grey_hive:gh_entry_maintenance:3'),'wrong hash rejected')
			check(not actual.accepts(scene,{'worldEpoch':3},expected,'grey_hive:gh_entry_maintenance:2'),'stale contact epoch rejected')
			check(not actual.accepts({'worldId':'mist_harbor','sceneId':'gh_entry_maintenance'},{'worldEpoch':3},expected,'grey_hive:gh_entry_maintenance:3'),'wrong world rejected')
	var contact: String = function_text(cenyao,'draw_entry_contact_shadow')
	for guard in ["renderer_script.resource_path!='res://scripts/world/renderer.gd'","host_script.resource_path!='res://scripts/ui/v1_main.gd'",'native_snapshot_value!=snapshot','not P.snapshot(snapshot)','snapshot.entryToken!=null']:
		check(contact.contains(guard),'strict contact identity guard retained '+guard)
	print('GH_VISUAL_STATE_CONTRACT checks='+str(checks)+' failures='+str(failures))
	quit(1 if failures else 0)
