/** Passive test telemetry only. It never creates an input, command or renderer tick. */
export const INPUT_PROGRESS_BINDING = "__CURRENT_WEB_ACCEPTED_INPUT__";

// Self-contained because the same function is serialized before production boot.
export function createInputProgressPublisher(deliver) {
  let pending = null;
  let inFlight = false;
  let disposed = false;
  let delivered = 0;
  let failures = 0;
  let coalesced = 0;
  const increment = value => Math.min(Number.MAX_SAFE_INTEGER, value + 1);
  const drain = () => {
    if (disposed || inFlight || !pending) return;
    const record = pending;
    pending = null;
    inFlight = true;
    Promise.resolve().then(() => {
      if (!disposed) return deliver(record);
    }).then(() => { if (!disposed) delivered = increment(delivered); })
      .catch(() => { if (!disposed) failures = increment(failures); })
      .finally(() => { inFlight = false; drain(); });
  };
  return {
    publish(record) {
      if (disposed) return;
      if (pending) coalesced = increment(coalesced);
      pending = Object.freeze({ ...record });
      drain();
    },
    dispose() { disposed = true; pending = null; },
    diagnostics() { return { inFlight: Number(inFlight), pending: Number(Boolean(pending)), disposed, delivered, failures, coalesced }; },
  };
}

export function createInputProgressStore(streamId) {
  if (typeof streamId !== "string" || !/^[a-zA-Z0-9-]{16,80}$/.test(streamId)) throw new Error("E_INPUT_OBSERVER_STREAM");
  let latest = null;
  let documentId = null;
  let lastCheckpoint = null;
  let disposed = false;
  let accepted = 0;
  let rejected = 0;
  const increment = value => Math.min(Number.MAX_SAFE_INTEGER, value + 1);
  const positive = value => Number.isSafeInteger(value) && value > 0;
  const identity = value => typeof value === "string" && /^[a-zA-Z0-9-]{16,80}$/.test(value);
  const checkpointValid = value => value && typeof value === "object" && !Array.isArray(value)
    && Object.keys(value).sort().join() === "documentId,inputCount,worldEpoch"
    && identity(value.documentId) && positive(value.worldEpoch)
    && Number.isSafeInteger(value.inputCount) && value.inputCount >= 0;
  const keys = ["ackSeq", "authorityRevision", "documentId", "inputCount", "serverTick", "streamId", "worldEpoch"];
  const validate = record => record && typeof record === "object" && !Array.isArray(record)
    && Object.keys(record).sort().join() === keys.join()
    && record.streamId === streamId && identity(record.documentId) && record.documentId === documentId
    && [record.worldEpoch, record.ackSeq, record.inputCount, record.serverTick, record.authorityRevision].every(positive)
    && record.authorityRevision >= record.serverTick;
  return {
    arm(checkpoint) {
      if (disposed || !checkpointValid(checkpoint)) throw new Error("E_INPUT_OBSERVER_CHECKPOINT");
      if (checkpoint.documentId !== documentId) {
        documentId = checkpoint.documentId;
        latest = null;
        lastCheckpoint = null;
      }
      if ((lastCheckpoint && (checkpoint.worldEpoch < lastCheckpoint.worldEpoch || checkpoint.inputCount < lastCheckpoint.inputCount))
        || (latest && (checkpoint.worldEpoch < latest.worldEpoch || checkpoint.inputCount < latest.inputCount))) {
        throw new Error("E_INPUT_OBSERVER_CHECKPOINT_REGRESSION");
      }
      lastCheckpoint = Object.freeze({ ...checkpoint });
      return lastCheckpoint;
    },
    accept(record) {
      if (disposed) return false;
      if (!validate(record) || (latest && (
        record.worldEpoch < latest.worldEpoch || record.inputCount <= latest.inputCount
        || (record.worldEpoch === latest.worldEpoch && (record.ackSeq <= latest.ackSeq
          || record.serverTick <= latest.serverTick || record.authorityRevision <= latest.authorityRevision))
      ))) { rejected = increment(rejected); return false; }
      latest = Object.freeze({ ...record });
      accepted = increment(accepted);
      return true;
    },
    countAfter(checkpoint) {
      if (!checkpointValid(checkpoint)) {
        throw new Error("E_INPUT_OBSERVER_CHECKPOINT");
      }
      // Checkpoint is sampled once from the actual idle browser before New or
      // Continue. A delayed old-page/old-epoch notification can never satisfy it.
      return !disposed && checkpoint.documentId === documentId && latest && latest.worldEpoch > checkpoint.worldEpoch
        && latest.inputCount > checkpoint.inputCount ? latest.inputCount : 0;
    },
    diagnostics() { return { latest, documentId, disposed, accepted, rejected }; },
    dispose() { disposed = true; latest = null; documentId = null; lastCheckpoint = null; },
  };
}
