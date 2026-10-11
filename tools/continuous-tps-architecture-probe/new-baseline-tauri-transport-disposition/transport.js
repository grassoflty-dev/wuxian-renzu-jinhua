export const ERROR = Object.freeze({
  TAURI_UNAVAILABLE: 'E_TAURI_UNAVAILABLE', TAURI_CAPABILITY: 'E_TAURI_CAPABILITY',
  HTTP_NOT_EXPLICIT: 'E_HTTP_NOT_EXPLICIT', HTTP_HEALTH: 'E_HTTP_HEALTH', HTTP_CAPABILITY: 'E_HTTP_CAPABILITY',
  HTTP_AUTHORITY: 'E_HTTP_AUTHORITY', HTTP_SCHEMA: 'E_HTTP_SCHEMA', REQUEST_ID: 'E_REQUEST_ID',
  OLD_GENERATION: 'E_OLD_GENERATION', ACK_ORDER: 'E_ACK_ORDER', NON_IDEMPOTENT_RETRY: 'E_NON_IDEMPOTENT_RETRY', UNKNOWN_SAVE: 'E_SAVE_UNKNOWN'
});

export function tauriReady(tauri) {
  return Boolean(tauri && typeof tauri.invoke === 'function' && typeof tauri.channel === 'function' && tauri.capability?.schemaVersion === 1 && tauri.capability?.authority === 'rust');
}

export function selectTransport({ mode = 'packaged', tauri = null, http = null } = {}) {
  if (tauri && !tauriReady(tauri)) return { ok: false, error: ERROR.TAURI_CAPABILITY };
  if (tauriReady(tauri)) return { ok: true, kind: 'tauri', reason: 'verified_invoke_channel' };
  if (mode === 'packaged') return { ok: false, error: ERROR.TAURI_UNAVAILABLE };
  if (!http?.explicit) return { ok: false, error: ERROR.HTTP_NOT_EXPLICIT };
  if (http.health !== true) return { ok: false, error: ERROR.HTTP_HEALTH };
  if (http.capability !== true) return { ok: false, error: ERROR.HTTP_CAPABILITY };
  if (http.authority !== 'rust') return { ok: false, error: ERROR.HTTP_AUTHORITY };
  if (http.schemaVersion !== 1) return { ok: false, error: ERROR.HTTP_SCHEMA };
  return { ok: true, kind: 'http-dev-test', reason: 'explicit_verified_handshake' };
}

export function newRequest(kind, requestId, idempotencyKey = null) {
  if (!requestId || (kind === 'command' && !idempotencyKey)) return { ok: false, error: ERROR.REQUEST_ID };
  return { ok: true, requestId, idempotencyKey, kind, retrySafe: kind !== 'command' || Boolean(idempotencyKey) };
}

export function acceptResponse(state, response) {
  if (response.requestId !== state.requestId) return { ok: false, error: ERROR.REQUEST_ID };
  if (response.generation !== state.generation || response.worldEpoch !== state.worldEpoch) return { ok: false, error: ERROR.OLD_GENERATION };
  if (response.ackSeq < state.ackSeq) return { ok: false, error: ERROR.ACK_ORDER };
  return { ok: true, ackSeq: response.ackSeq };
}

export function retryPolicy(kind, outcome) {
  if (kind === 'query') return { retry: outcome === 'timeout', reissue: true };
  if (kind === 'command') return { retry: false, error: ERROR.NON_IDEMPOTENT_RETRY, requireUserReconcile: outcome === 'unknown' || outcome === 'timeout' };
  if (kind === 'save') return { retry: false, error: ERROR.UNKNOWN_SAVE, requireUserReconcile: outcome === 'unknown' };
  return { retry: false };
}

export function acceptChannel(state, message) {
  if (!state.handshaken) return { ok: false, error: 'E_CHANNEL_HANDSHAKE' };
  if (message.generation !== state.generation || message.worldEpoch !== state.worldEpoch) return { ok: false, error: ERROR.OLD_GENERATION };
  if (message.serverTick <= state.serverTick) return { ok: false, error: ERROR.ACK_ORDER };
  return { ok: true, serverTick: message.serverTick };
}
