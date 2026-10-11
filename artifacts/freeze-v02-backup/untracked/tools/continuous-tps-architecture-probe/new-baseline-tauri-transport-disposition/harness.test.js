import assert from 'node:assert/strict';
import { ERROR, selectTransport, tauriReady, newRequest, acceptResponse, retryPolicy, acceptChannel } from './transport.js';

const goodTauri = { invoke() {}, channel() {}, capability: { schemaVersion: 1, authority: 'rust' } };
const goodHttp = { explicit: true, health: true, capability: true, authority: 'rust', schemaVersion: 1 };
const cases = [];
function test(name, fn) { fn(); cases.push(name); }

test('packaged selects verified Tauri', () => assert.deepEqual(selectTransport({ mode: 'packaged', tauri: goodTauri }), { ok: true, kind: 'tauri', reason: 'verified_invoke_channel' }));
test('packaged without Tauri fails closed', () => assert.equal(selectTransport({ mode: 'packaged', http: goodHttp }).error, ERROR.TAURI_UNAVAILABLE));
test('invalid Tauri does not silently fall back', () => assert.equal(selectTransport({ mode: 'packaged', tauri: { invoke() {} }, http: goodHttp }).error, ERROR.TAURI_CAPABILITY));
test('dev HTTP requires explicit verified handshake', () => assert.equal(selectTransport({ mode: 'dev', http: goodHttp }).kind, 'http-dev-test'));
test('HTTP without explicit flag rejected', () => assert.equal(selectTransport({ mode: 'dev', http: { ...goodHttp, explicit: false } }).error, ERROR.HTTP_NOT_EXPLICIT));
test('HTTP missing health/capability/authority/schema rejected', () => { for (const patch of [{ health: false }, { capability: false }, { authority: 'other' }, { schemaVersion: 2 }]) assert.notEqual(selectTransport({ mode: 'test', http: { ...goodHttp, ...patch } }).ok, true); });
test('request IDs and command idempotency required', () => { assert.equal(newRequest('command', 'r1', 'i1').ok, true); assert.equal(newRequest('command', 'r1').error, ERROR.REQUEST_ID); });
test('action timeout is not blindly retried', () => assert.equal(retryPolicy('command', 'timeout').retry, false));
test('query timeout may retry and renderer retry is not command retry', () => { assert.equal(retryPolicy('query', 'timeout').retry, true); assert.equal(retryPolicy('command', 'renderer_retry').retry, false); });
test('response request id and epoch/generation are checked', () => { const s = { requestId: 'r1', generation: 2, worldEpoch: 3, ackSeq: 4 }; assert.equal(acceptResponse(s, { requestId: 'r2', generation: 2, worldEpoch: 3, ackSeq: 5 }).error, ERROR.REQUEST_ID); assert.equal(acceptResponse(s, { requestId: 'r1', generation: 1, worldEpoch: 3, ackSeq: 5 }).error, ERROR.OLD_GENERATION); });
test('channel handshake/order/reload are checked', () => { const s = { handshaken: true, generation: 2, worldEpoch: 3, serverTick: 4 }; assert.equal(acceptChannel(s, { generation: 1, worldEpoch: 3, serverTick: 5 }).error, ERROR.OLD_GENERATION); assert.equal(acceptChannel(s, { generation: 2, worldEpoch: 3, serverTick: 4 }).error, ERROR.ACK_ORDER); });
test('save unknown outcome requires reconciliation', () => assert.equal(retryPolicy('save', 'unknown').error, ERROR.UNKNOWN_SAVE));
test('none transport never becomes authority', () => assert.equal(selectTransport({ mode: 'dev' }).ok, false));
test('Tauri readiness requires invoke/channel/capability', () => assert.equal(tauriReady(goodTauri), true));

console.log(JSON.stringify({ passed: cases.length, cases, candidateOnly: true, networkOpened: false, realFetchCalled: false }));
