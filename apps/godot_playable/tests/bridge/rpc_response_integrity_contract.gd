extends SceneTree
# SOURCE ONLY: not executed. Off-tree clients; no connection, request send or retry.
const Rpc = preload('res://scripts/bridge/rpc_client.gd')
const P = preload('res://scripts/bridge/protocol.gd')
const Fixture = preload('res://tests/ui/decon_hazard_text_contract.gd')
const UNKNOWN = 'E_RPC_RESPONSE_LIMIT_MUTATION_UNKNOWN'
var failures: int = 0
var errors: Array = []
var replies: Array = []
var changes: Array = []
var sample: Dictionary = {}

func _initialize() -> void:
	sample = Fixture._fixture()
	_check(P.snapshot(sample),'existing fixture validates')
	_error_matrix()
	_out_of_order()
	_protocol_rejections()
	print('RPC_RESPONSE_INTEGRITY_CONTRACT failures='+str(failures))
	quit(0 if failures==0 else 1)

func _check(ok: bool, label: String) -> void:
	if not ok:
		failures += 1
		push_error(label)

func _client():
	errors.clear()
	replies.clear()
	changes.clear()
	var c = Rpc.new()
	c.connection_generation = 7
	c.next_id = 102
	c.authenticated = true
	c.hello_sent = true
	c.failed.connect(func(id, method, code, association): errors.append([id,method,code,association.duplicate(true)]))
	c.response.connect(func(id, method, result, association): replies.append([id,method,result,association.duplicate(true)]))
	c.connection_changed.connect(func(connected, code): changes.append([connected,code]))
	return c

func _association(id: int) -> Dictionary:
	return {'generation':7,'context':{'worldId':sample.worldId,'sceneId':sample.sceneId,
		'worldEpoch':sample.worldEpoch},'purpose':'action','actionKind':'pulse','sequence':id}

func _seed(c, id: int, method: String) -> Dictionary:
	var association: Dictionary = _association(id)
	# Same pending record fields as request(); its network gate is intentionally not used.
	c.pending[id] = {'method':method,'association':association.duplicate(true),
		'started':Time.get_ticks_msec(),'generation':c.connection_generation}
	return association

func _envelope(id: int, code: String) -> Dictionary:
	return {'protocol':Rpc.OUTER,'protocolVersion':1,'id':id,'ok':false,
		'result':null,'error':{'code':code}}

func _no_resend(c, label: String) -> void:
	_check(c.queue.is_empty() and c.queued_bytes==0 and c.next_id==102,label+'/no enqueue or new id')

func _error_matrix() -> void:
	for method in Rpc.METHODS:
		for code in [UNKNOWN,'E_ACTION_BUSY','E_RPC_RESPONSE_LIMIT',UNKNOWN+'x',UNKNOWN.to_lower(),'']:
			var c = _client()
			var association: Dictionary = _seed(c,101,method)
			var envelope: Dictionary = _envelope(101,code)
			var before: Dictionary = envelope.duplicate(true)
			c._handle(envelope)
			var label: String = method+'/'+code
			if method=='hello':
				_check(errors.is_empty() and replies.is_empty(),label+'/handshake has no application result')
				_check(c.transport_stopped and changes==[[false,'E_HANDSHAKE_REJECTED']],label+'/handshake unchanged')
			else:
				var expected: String = 'UNKNOWN_MUTATION' if not method in Rpc.READS and code==UNKNOWN else code
				_check(errors==[[101,method,expected,association]],label+'/original ownership and exact code')
				_check(replies.is_empty() and changes.is_empty() and not c.transport_stopped,label+'/only failure signal')
			_check(c.pending.is_empty() and envelope==before,label+'/consume once without mutating envelope')
			_no_resend(c,label)
			c.free()

func _out_of_order() -> void:
	var c = _client()
	var first: Dictionary = _seed(c,101,'formal_snapshot')
	var second: Dictionary = _seed(c,102,'formal_submit_action')
	var waiting: Dictionary = c.pending[101].duplicate(true)
	c._handle(_envelope(102,UNKNOWN))
	_check(errors==[[102,'formal_submit_action','UNKNOWN_MUTATION',second]],'later reply owns later request')
	_check(c.pending.size()==1 and c.pending.get(101)==waiting,'earlier pending ownership unchanged')
	_no_resend(c,'out of order mutation')
	var before: Dictionary = sample.duplicate(true)
	var success: Dictionary = _envelope(101,'')
	success.ok = true
	success.error = null
	success.result = sample
	c._handle(success)
	_check(replies==[[101,'formal_snapshot',sample,first]],'earlier success retains id method result association')
	_check(sample==before and c.pending.is_empty() and not c.transport_stopped,'success fixture preserved')
	_check(changes.is_empty() and errors.size()==1,'success adds no failure or connection event')
	_no_resend(c,'success')
	c._handle(success)
	_check(c.transport_stopped and changes==[[false,'E_RESPONSE_PROTOCOL']],'duplicate response rejected')
	_check(replies.size()==1 and errors.size()==1,'duplicate never redelivered')
	_no_resend(c,'duplicate')
	c.free()

func _protocol_rejections() -> void:
	for field in ['protocol','id','ok','error']:
		var c = _client()
		var association: Dictionary = _seed(c,101,'formal_snapshot')
		var bad: Dictionary = _envelope(101,UNKNOWN)
		match field:
			'protocol': bad.protocol = 'wrong'
			'id': bad.id = 999
			'ok': bad.ok = 1
			'error': bad.error = {'code':7}
		c._handle(bad)
		_check(c.transport_stopped and changes==[[false,'E_RESPONSE_PROTOCOL']],field+'/protocol rejection')
		var expected: Array = [[101,'formal_snapshot','E_RESPONSE_PROTOCOL',association]]
		_check(errors==expected and replies.is_empty(),field+'/existing pending-consumption semantics')
		_check(c.pending.is_empty(),field+'/no retained request')
		_no_resend(c,field)
		c.free()
