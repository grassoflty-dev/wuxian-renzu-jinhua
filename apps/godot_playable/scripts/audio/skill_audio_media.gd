extends RefCounted
const P = preload('res://scripts/bridge/protocol.gd')
const CONFIG = {"audio.skill.pierce.cast":{"rawParentSha256":"fda7c776520134a84d79e13f60b1cbb853bb9e5ff052b27d48932fe3792746b3","event":"PierceStarted","maxMs":200,"rawParentId":"originals/pierce_raw.wav","path":"res://assets/audio/skills/pierce_cast.wav"},"audio.player.dash":{"rawParentSha256":"363ecf57156615083fa3d23eba121c05abeb3bae78bda2afc3df32e8eb6b4f44","event":"DashStarted","maxMs":300,"rawParentId":"originals/dash_raw.wav","path":"res://assets/audio/skills/dash_start.wav"},"audio.skill.guard.start":{"rawParentSha256":"26483f50d62dbd7280fd8939d252f8fdf6e7f8bd39e83e7a1842b2eb31b5b048","event":"GuardStarted","maxMs":350,"rawParentId":"originals/guard_raw.wav","path":"res://assets/audio/skills/guard_start.wav"},"audio.skill.pulse.cast":{"rawParentSha256":"fa8e024085d4c1709ecaffd5cb62b15542f986d47029588748d86c6a8be46026","event":"PulseCast","maxMs":500,"rawParentId":"originals/pulse_raw.wav","path":"res://assets/audio/skills/pulse_cast.wav"}}
const RATE = 22050
const FILE_LIMIT = 1048576
var mapping_attempted = false
var selected_rows: Dictionary = {}
var attempted: Dictionary = {}
# Immutable buffers only. World/event/permission state belongs to native_audio/reader.
var buffers: Dictionary = {}
func _hash(v) -> bool:
	if typeof(v)!=TYPE_STRING or v.length()!=64:
		return false
	for c in v:
		if not c in '0123456789abcdef':
			return false
	return true
func _read_mapping_once() -> void:
	if mapping_attempted:
		return
	mapping_attempted = true
	var file = FileAccess.open('res://data/skill_audio.internal-use.json',FileAccess.READ)
	if file==null:
		return
	if file.get_length()==0 or file.get_length()>16384:
		file.close()
		return
	var bytes = file.get_buffer(file.get_length())
	file.close()
	if bytes.get_string_from_utf8().to_utf8_buffer()!=bytes:
		return
	var map = JSON.parse_string(bytes.get_string_from_utf8())
	if not map is Dictionary or map.get('schema')!='wuxian.skill-audio.internal-use/v1' or typeof(map.get('internalSourceSelected'))!=TYPE_BOOL or map.get('GodotPlayback')!='NOT_RUN' or map.get('publicRelease')!='NOT_GRANTED' or map.get('commercialRelease')!='NOT_ESTABLISHED' or not map.get('assets') is Dictionary:
		return
	if not map.internalSourceSelected:
		# This operation installs only the empty unselected mapping.
		return
	if map.get('sourceStatus')!='INTERNAL_CANDIDATE_SOURCE_SELECTION' or map.get('upstreamAdmission')!='NOT_ADMITTED' or map.get('use')!='PRIVATE_NONCOMMERCIAL_ONLY' or map.assets.is_empty() or map.assets.size()>4:
		return
	for cue in map.assets:
		if not CONFIG.has(cue):
			return
	selected_rows = map.assets
func _tag(bytes: PackedByteArray, at: int, tag: String) -> bool:
	return at>=0 and at+4<=bytes.size() and bytes.slice(at,at+4).get_string_from_ascii()==tag
func _parse_pcm(bytes: PackedByteArray, frames: int) -> PackedByteArray:
	if bytes.size()<44 or bytes.size()>FILE_LIMIT or not _tag(bytes,0,'RIFF') or not _tag(bytes,8,'WAVE') or bytes.decode_u32(4)!=bytes.size()-8:
		return PackedByteArray()
	var offset = 12
	var fmt_at = -1
	var data_at = -1
	var data_size = 0
	while offset<bytes.size():
		if offset+8>bytes.size():
			return PackedByteArray()
		var chunk_size = int(bytes.decode_u32(offset+4))
		var begin = offset+8
		var end = begin+chunk_size
		var padded = end+(chunk_size%2)
		if end<begin or end>bytes.size() or padded>bytes.size():
			return PackedByteArray()
		if _tag(bytes,offset,'fmt '):
			if fmt_at!=-1 or not chunk_size in [16,18]:
				return PackedByteArray()
			fmt_at = begin
			if chunk_size==18 and bytes.decode_u16(begin+16)!=0:
				return PackedByteArray()
		elif _tag(bytes,offset,'data'):
			if data_at!=-1 or chunk_size<=0 or chunk_size%2!=0:
				return PackedByteArray()
			data_at = begin
			data_size = chunk_size
		offset = padded
	if offset!=bytes.size() or fmt_at<0 or data_at<0 or data_size!=frames*2:
		return PackedByteArray()
	if bytes.decode_u16(fmt_at)!=1 or bytes.decode_u16(fmt_at+2)!=1 or bytes.decode_u32(fmt_at+4)!=RATE or bytes.decode_u32(fmt_at+8)!=44100 or bytes.decode_u16(fmt_at+12)!=2 or bytes.decode_u16(fmt_at+14)!=16:
		return PackedByteArray()
	for n in range(frames):
		var sample = int(bytes.decode_u16(data_at+n*2))
		if sample>=32768:
			sample-=65536
		if abs(sample)>8192:
			return PackedByteArray()
	return bytes.slice(data_at,data_at+data_size)
func candidate(cue: String, event: String) -> Dictionary:
	if not CONFIG.has(cue) or CONFIG[cue].event!=event:
		return {}
	if attempted.has(cue):
		return buffers.get(cue,{})
	attempted[cue] = true
	_read_mapping_once()
	var row = selected_rows.get(cue)
	var c: Dictionary = CONFIG[cue]
	if not row is Dictionary or typeof(row.get('internalSourceSelected'))!=TYPE_BOOL or not row.internalSourceSelected or row.get('upstreamAdmission')!='NOT_ADMITTED' or row.get('use')!='PRIVATE_NONCOMMERCIAL_ONLY' or row.get('cue')!=cue or row.get('event')!=c.event or row.get('path')!=c.path or row.get('rawParentId')!=c.rawParentId or row.get('rawParentSha256')!=c.rawParentSha256 or not P.counter(row.get('bytes'),44,FILE_LIMIT) or not _hash(row.get('sha256')) or not P.counter(row.get('frames'),1,floor(float(c.maxMs)*RATE/1000.0)) or not P.counter(row.get('sampleRate'),RATE,RATE) or not P.counter(row.get('channels'),1,1) or not P.counter(row.get('bitsPerSample'),16,16) or row.get('format')!='PCM16' or typeof(row.get('loop'))!=TYPE_BOOL or row.loop or not P.number(row.get('maxMs'),c.maxMs,c.maxMs) or not P.number(row.get('peakMax'),0.25,0.25):
		return {}
	var file = FileAccess.open(c.path,FileAccess.READ)
	if file==null:
		return {}
	if file.get_length()!=row.bytes:
		file.close()
		return {}
	var bytes = file.get_buffer(file.get_length())
	file.close()
	var hash = HashingContext.new()
	if hash.start(HashingContext.HASH_SHA256)!=OK or hash.update(bytes)!=OK or hash.finish().hex_encode()!=row.sha256:
		return {}
	var pcm = _parse_pcm(bytes,int(row.frames))
	if pcm.is_empty():
		return {}
	var duration = float(row.frames)*1000.0/RATE
	if not is_finite(duration) or duration<=0 or duration>c.maxMs:
		return {}
	buffers[cue] = {'pcm':pcm,'kind':'selected_media','duration_ms':duration,'event':c.event}
	return buffers[cue]
