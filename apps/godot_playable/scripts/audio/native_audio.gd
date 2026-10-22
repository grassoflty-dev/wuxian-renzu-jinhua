extends Node
const P = preload('res://scripts/bridge/protocol.gd')
const L = preload('res://scripts/audio/audio_cue_library.gd')
const Reader = preload('res://scripts/audio/audio_event_reader.gd')
const SkillMedia = preload('res://scripts/audio/skill_audio_media.gd')
const RATE = 22050
const CACHE_LIMIT = 25165824
# Nine possible outputs share fixed headroom. Each recipe tone is capped at .22.
const HEADROOM = 1.0/9.0
var reader
var skill_media
var cache_source: Dictionary = {}
var cache: Dictionary = {}
var cache_bytes = 0
var voices: Array = []
var ambient: AudioStreamPlayer
var ambient_id = ''
var muted = false
var volume = 0.35
var focused = true
var world_enabled = false
var output_context = ''
var failed = false
var status_text = '临时原生合成音频；输出尚未验证。仅当前会话。'
func _ready() -> void:
 reader = Reader.new()
 skill_media = SkillMedia.new()
func configure(mute_value, volume_value) -> bool:
 if typeof(mute_value)!=TYPE_BOOL or not P.number(volume_value,0,1):
  return false
 muted = mute_value
 volume = float(volume_value)
 if muted or volume==0:
  stop_all()
 for v in voices:
  v.player.volume_linear = volume*HEADROOM
 if is_instance_valid(ambient):
  ambient.volume_linear = volume*HEADROOM
 return true
func suspend_world() -> void:
 world_enabled = false
 if reader!=null:
  reader.suspend()
 for v in voices.duplicate():
  if not v.ui:
   release_voice(v)
 stop_ambient()
func stop_all() -> void:
 suspend_world()
 for v in voices.duplicate():
  release_voice(v)
func stop_ambient() -> void:
 if is_instance_valid(ambient):
  ambient.stop()
  ambient.queue_free()
 ambient = null
 ambient_id = ''
func release_voice(v: Dictionary) -> void:
 v.player.stop()
 v.player.queue_free()
 voices.erase(v)
func accept_events(rows, s: Dictionary, generation: int, eligible: bool) -> void:
 if not reader.accept(rows,s,generation,eligible and not muted and volume>0 and focused and not failed):
  suspend_world()
func update(s: Dictionary, scene: Dictionary, generation: int, eligible: bool, has_focus: bool) -> void:
 focused = has_focus
 if not focused:
  stop_all()
 var enabled = eligible and focused and not muted and volume>0 and not failed and not s.is_empty() and not scene.is_empty()
 reader.sync(s,generation,enabled)
 if output_context!=reader.context_key or not enabled:
  suspend_world()
 output_context = reader.context_key
 world_enabled = enabled
 for v in voices.duplicate():
  if not v.player.playing or Time.get_ticks_msec()>=v.until or not v.ui and (not enabled or not reader.authorized(v.row.event,s,scene) or reader.age(v.row,s)>=v.get('duration',0.0)):
   release_voice(v)
 if not enabled:
  return
 for row in reader.dispatch(s,scene):
  if row.stop:
   for v in voices.duplicate():
    if not v.ui and row.key!='' and v.row.key==row.key:
     release_voice(v)
  if row.cue!='' and reader.age(row,s)<row.duration:
   var pan = 0.0
   if row.event.kind.begins_with('ResonanceWarden'):
    var d = P.vector(row.event.positionM)-P.vector(s.player.transform.positionM)
    var distance = Vector2(d.x,d.z).length()
    pan = clampf((d.x-d.z)/(sqrt(2.0)*distance),-1,1) if distance>0 else 0.0
   play_cue(row.cue,row,reader.age(row,s),pan,false)
 # A one-shot/cache failure may invalidate the earlier enabled decision.
 if not output_allowed(true):
  stop_ambient()
  return
 var desired = ''
 if s.worldId=='return_station' and s.sceneId=='rs_core_room':
  desired = 'bgm.return_station'
 elif s.worldId=='grey_hive' and scene.get('worldId')=='grey_hive' and scene.get('sceneId')==s.sceneId:
  desired = 'audio.gh.ambient.facility'
 if desired!=ambient_id:
  stop_ambient()
  if desired!='':
   var wav = stream_for(desired,true,0)
   if wav!=null and output_allowed(true):
    ambient = AudioStreamPlayer.new()
    add_child(ambient)
    ambient.stream = wav
    ambient.volume_linear = volume*HEADROOM
    if not output_allowed(true):
     stop_ambient()
     return
    ambient.play()
    ambient_id = desired
    if not ambient.has_stream_playback():
     audio_failure()
func ui_cue(kind: String, has_focus: bool) -> void:
 if not has_focus or muted or volume<=0 or failed or not kind in ['confirm','cancel','error','tab']:
  return
 play_cue('audio.ui.'+kind,{},0,0,true)
func play_cue(id: String, row: Dictionary, age_ms: float, pan: float, ui: bool) -> void:
 if not output_allowed(not ui):
  return
 if voices.size()>=8:
  release_voice(voices[0])
 var wav = stream_for(id,false,pan,'' if ui else row.event.kind)
 if wav==null or not output_allowed(not ui):
  return
 var player = AudioStreamPlayer.new()
 add_child(player)
 player.stream = wav
 player.volume_linear = volume*HEADROOM
 var remaining = L.duration_ms(id)-age_ms if ui else row.duration-age_ms
 if not ui and cache_source.get(id,{}).get('kind')=='selected_media' and cache_source[id].event==row.event.kind:
  remaining = float(cache_source[id].duration_ms)-age_ms
 if remaining<=0:
  player.queue_free()
  return
 voices.append({'player':player,'row':row,'ui':ui,'duration':remaining+age_ms,'until':Time.get_ticks_msec()+int(ceil(remaining))})
 player.play(age_ms/1000.0)
 if not player.has_stream_playback():
  audio_failure()
func audio_failure() -> void:
 failed = true
 status_text = '音频反馈暂不可用；游戏仍可继续。输出尚未验证。'
 stop_all()
func output_allowed(world: bool) -> bool:
 return not failed and not muted and focused and P.number(volume,0,1) and volume>0 and (not world or world_enabled)
func stream_for(id: String, looped: bool, pan: float, event: String = '') -> AudioStreamWAV:
 if not output_allowed(looped) or not L.AUDIO_CUE_LIBRARY.has(id) or not P.number(pan,-1,1):
  return null
 if not cache.has(id):
  var selected = skill_media.candidate(id,event) if not looped and skill_media!=null else {}
  var pcm: PackedByteArray
  var source = {'kind':'recipe','duration_ms':L.duration_ms(id),'event':''}
  if not selected.is_empty():
   pcm = selected.pcm
   source = {'kind':selected.kind,'duration_ms':selected.duration_ms,'event':selected.event}
  else:
   pcm = synthesize(id,looped)
  if pcm.is_empty() or cache_bytes+pcm.size()>CACHE_LIMIT:
   audio_failure()
   return null
  cache[id] = pcm
  cache_source[id] = source
  cache_bytes += pcm.size()
 var mono: PackedByteArray = cache[id]
 var data = mono
 if not looped:
  # Bounded per-voice stereo copy of cached base PCM, never a pan/volume cache.
  data = PackedByteArray()
  data.resize(mono.size()*2)
  var left = sqrt((1.0-pan)*0.5)
  var right = sqrt((1.0+pan)*0.5)
  for n in range(int(mono.size()/2)):
   var sample = int(mono[n*2]) | (int(mono[n*2+1])<<8)
   if sample>=32768:
    sample-=65536
   put_sample(data,n*4,int(sample*left))
   put_sample(data,n*4+2,int(sample*right))
 var wav = AudioStreamWAV.new()
 wav.format = AudioStreamWAV.FORMAT_16_BITS
 wav.mix_rate = RATE
 wav.stereo = not looped
 wav.data = data
 if looped:
  wav.loop_mode = AudioStreamWAV.LOOP_FORWARD
  wav.loop_begin = 0
  wav.loop_end = RATE*5
 return wav
func put_sample(data: PackedByteArray, offset: int, value: int) -> void:
 var bounded = clampi(value,-32767,32767)&65535
 data[offset] = bounded&255
 data[offset+1] = (bounded>>8)&255
func synthesize(id: String, looped: bool) -> PackedByteArray:
 # Called only by future runtime playback. No constructor/PCM at script load.
 var recipe: Dictionary = L.AUDIO_CUE_LIBRARY[id]
 var length_ms = 5000.0 if looped else L.duration_ms(id)
 if length_ms<=0 or length_ms>5000:
  return PackedByteArray()
 var frames = int(ceil(length_ms*RATE/1000.0))
 var data = PackedByteArray()
 data.resize(frames*2)
 for n in range(frames):
  var seconds = float(n)/RATE
  var mix = 0.0
  for tone in recipe.tones:
   var t = seconds-float(tone.offsetMs)/1000.0
   var duration = float(tone.durationMs)/1000.0
   if t<0 or not looped and t>=duration:
    continue
   var frequency = float(tone.frequencyHz)
   var end = float(tone.get('endFrequencyHz',frequency))
   var cycles = frequency*t
   if not looped and end!=frequency:
    var k = log(end/frequency)/duration
    cycles = frequency*(exp(k*t)-1.0)/k
   var phase = fposmod(cycles,1.0)
   var wave = sin(TAU*phase)
   if tone.waveform=='triangle':
    wave = 1.0-4.0*abs(phase-0.5)
   elif tone.waveform=='sawtooth':
    wave = 2.0*phase-1.0
   var gain = minf(float(recipe.peakGain),0.22)
   if not looped:
    var attack = minf(t/0.009,1.0)
    var release_at = maxf(0.01,duration-0.018)
    var envelope = attack
    if t>release_at:
     envelope = exp(log(0.0001/maxf(gain,0.0001))*(t-release_at)/(duration-release_at))
    gain *= envelope
   mix += wave*gain
  put_sample(data,n*2,int(clampf(mix,-1,1)*32767))
 return data
func _exit_tree() -> void:
 stop_all()
 cache.clear()
 cache_source.clear()
 skill_media = null
 cache_bytes = 0
