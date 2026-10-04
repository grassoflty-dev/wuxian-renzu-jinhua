import { createInputProgressPublisher, INPUT_PROGRESS_BINDING } from "./input-progress.mjs";

/** Synthetic UI-only authority. No movement, combat, route unlocks or disk saves.
 * Self-contained so addInitScript can install the identical model before boot.
 */
export function createMockBridge(options = {}, observeAcceptedInput = null, documentId = null) {
  const clone = value => structuredClone(value);
  const state = { calls: [], totalCalls: 0, ledgerDropped: 0, unexpected: [], unexpectedCount: 0, epoch: 1, tick: 1, ack: 0, paused: false,
    current: null, slots: [], saved: {}, blocked: {}, failures: {} };
  const pending = new Map();
  let observerErrors = 0;
  let lastObservedInput = null;
  let notifiedEpoch = null;
  const knownCommands = ["formal_snapshot", "formal_list_save_slots", "formal_new", "formal_return",
    "formal_pause", "formal_resume", "formal_presentation_events", "formal_sound_cues",
    "formal_submit_input", "formal_submit_action", "formal_save_slot", "formal_continue_slot"];
  const observations = Object.fromEntries(knownCommands.map(command => [command,
    { count: 0, firstAtMs: null, lastAtMs: null }]));
  const now = () => typeof performance === "undefined" ? Date.now() : performance.now();
  const highRate = new Set(["formal_submit_input", "formal_presentation_events", "formal_sound_cues"]);
  const controlCalls = Object.fromEntries(knownCommands.filter(command => !highRate.has(command))
    .map(command => [command, []]));
  const firstInputs = [];
  const increment = value => Math.min(Number.MAX_SAFE_INTEGER, value + 1);
  // Match authored player entries; these are presentation fixtures, not disk saves.
  const spawnPositions = {
    rs_core_room: { xM: 4, yM: 0, zM: 8 },
    gh_entry_maintenance: { xM: 2, yM: 0, zM: 7 },
  };
  const makeSnapshot = (worldId = "return_station", sceneId = "rs_core_room") => ({
    kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId, sceneId, checkpointId: null, worldEpoch: state.epoch, serverTick: state.tick,
    authorityRevision: state.tick, ackSeq: state.ack,
    player: { entityId: "player", transform: { positionM: clone(spawnPositions[sceneId] ?? { xM: 0, yM: 0, zM: 0 }), yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 80, maxHp: 100,
      currentEnergy: 60, maxEnergy: 100, facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: clone(options.capabilityItems ?? []) },
    progression: { schemaVersion: 1, currentWorldId: worldId, eventSeq: 0, worlds: [] },
  });
  const slot = (slotId, valid = true, readOnly = false) => ({
    slotId, displayName: slotId === "valid-slot" ? "当前有效存档" : slotId === "legacy-slot" ? "旧版只读存档" : "损坏存档",
    updatedAtMs: 1000, worldId: "grey_hive", checkpointId: null, playerPositionM: null,
    currentHp: 80, maxHp: 100, currentEnergy: 60, maxEnergy: 100, gateOpen: false,
    completedEvents: [], readOnly, valid, errorCode: valid ? null : "E_SAVE_CORRUPT",
  });
  if (options.withSlots !== false) {
    state.slots = [slot("valid-slot"), slot("legacy-slot", true, true), slot("corrupt-slot", false)];
    state.saved["valid-slot"] = makeSnapshot("grey_hive", "gh_entry_maintenance");
    state.saved["legacy-slot"] = makeSnapshot("grey_hive", "gh_entry_maintenance");
  }
  state.current = makeSnapshot(options.worldId, options.sceneId);
  const advance = (newEpoch = false) => {
    if (newEpoch) { state.epoch++; state.ack = 0; }
    state.tick++;
    state.current = { ...state.current, worldEpoch: state.epoch, serverTick: state.tick,
      authorityRevision: state.tick, ackSeq: state.ack };
  };
  const receipt = commandId => ({ commandId, applied: true, alreadyApplied: false,
    errorCode: null, worldEpoch: state.current.worldEpoch, serverTick: state.current.serverTick,
    authorityRevision: state.current.authorityRevision, snapshot: clone(state.current) });
  const bridge = {
    metadata: { currentWindow: { label: "main" } },
    async invoke(command, args = {}) {
      // Counts only actual bridge invocations, including blocked/rejected ones,
      // exactly matching the pre-existing invocation ledger assertion semantics.
      const call = { command, args: clone(args) };
      state.totalCalls = increment(state.totalCalls);
      if (state.calls.length === 512) { state.calls.shift(); state.ledgerDropped = increment(state.ledgerDropped); }
      state.calls.push(call);
      if (Object.hasOwn(controlCalls, command)) {
        const examples = controlCalls[command];
        if (examples.length === 8) examples.shift();
        examples.push(call);
      }
      if (command === "formal_submit_input" && firstInputs.length < 3) firstInputs.push(call);
      if (Object.hasOwn(observations, command)) {
        const row = observations[command];
        row.count = Math.min(Number.MAX_SAFE_INTEGER, row.count + 1);
        row.firstAtMs ??= now(); row.lastAtMs = now();
      }
      if (state.blocked[command]) await new Promise((resolve, reject) => {
        const queue = pending.get(command) ?? []; queue.push({ resolve, reject }); pending.set(command, queue);
      });
      if (state.failures[command]) {
        const message = state.failures[command]; delete state.failures[command]; throw new Error(message);
      }
      switch (command) {
        case "formal_snapshot": return clone(state.current);
        case "formal_list_save_slots": return clone(state.slots);
        case "formal_new":
          state.current = makeSnapshot(); advance(true); state.paused = false; return receipt("new");
        case "formal_return":
          state.current = makeSnapshot(); advance(true); state.paused = false; return receipt("return");
        case "formal_pause":
          state.paused = true; advance(); return receipt("pause");
        case "formal_resume":
          state.paused = false; advance(); return receipt("resume");
        case "formal_presentation_events":
        case "formal_sound_cues": return [];
        case "formal_submit_input": {
          if (state.paused) throw new Error("E_MOCK_INPUT_WHILE_PAUSED");
          if (args.sample?.worldEpoch !== state.epoch) throw new Error("E_MOCK_INPUT_EPOCH");
          const priorAck = state.ack;
          state.ack = args.sample.seq; advance();
          const result = receipt("input");
          const sample = args.sample;
          // Qualify observations only after the actual invocation has produced
          // its successful receipt. No notification changes mock outcomes,
          // counters, input dispatch or simulation state.
          const valid = sample.protocol === "continuous-input" && sample.protocolVersion === 2
            && Number.isSafeInteger(sample.worldEpoch) && sample.worldEpoch > 0
            && Number.isSafeInteger(sample.seq) && sample.seq > priorAck
            && Number.isSafeInteger(sample.clientTimeMs) && sample.clientTimeMs >= 0
            && [sample.moveX, sample.moveZ, sample.aimX, sample.aimZ].every(axis => Number.isFinite(axis) && Math.abs(axis) <= 1)
            && (!lastObservedInput || lastObservedInput.worldEpoch !== sample.worldEpoch
              || sample.clientTimeMs >= lastObservedInput.clientTimeMs)
            && result.applied && !result.alreadyApplied && result.errorCode === null
            && result.worldEpoch === sample.worldEpoch && result.snapshot.worldEpoch === sample.worldEpoch
            && result.snapshot.ackSeq === sample.seq
            && result.serverTick === result.snapshot.serverTick
            && result.authorityRevision === result.snapshot.authorityRevision;
          if (valid) {
            lastObservedInput = { worldEpoch: sample.worldEpoch, clientTimeMs: sample.clientTimeMs };
            // Readiness only needs the first qualified input in each epoch.
            // Raw invocation counters remain exact and independently readable.
            if (observeAcceptedInput && notifiedEpoch !== result.worldEpoch) {
              notifiedEpoch = result.worldEpoch;
              try { observeAcceptedInput(Object.freeze({ documentId, worldEpoch: result.worldEpoch, ackSeq: result.snapshot.ackSeq,
                inputCount: observations.formal_submit_input.count, serverTick: result.serverTick,
                authorityRevision: result.authorityRevision })); }
              catch { observerErrors = increment(observerErrors); }
            }
          }
          return result;
        }
        case "formal_submit_action":
          if (state.paused) throw new Error("E_MOCK_ACTION_WHILE_PAUSED");
          advance(); return receipt("action");
        case "formal_save_slot": {
          if (!state.paused) throw new Error("E_MOCK_SAVE_REQUIRES_PAUSE");
          const existing = state.slots.find(item => item.slotId === args.slotId);
          if ((!args.create && (!existing?.valid || existing.readOnly)) || (args.create && existing)) {
            throw new Error("E_MOCK_INVALID_SAVE_TARGET");
          }
          state.saved[args.slotId] = clone(state.current);
          const summary = { ...slot(args.slotId), displayName: args.displayName, worldId: state.current.worldId };
          state.slots = [...state.slots.filter(item => item.slotId !== args.slotId), summary];
          return receipt("save-slot");
        }
        case "formal_continue_slot": {
          const summary = state.slots.find(item => item.slotId === args.slotId);
          if (!summary?.valid || !state.saved[args.slotId]) throw new Error("E_MOCK_INVALID_CONTINUE_TARGET");
          state.current = clone(state.saved[args.slotId]); advance(true); state.paused = false; return receipt("continue-slot");
        }
        default:
          state.unexpectedCount = increment(state.unexpectedCount);
          if (state.unexpected.length < 64) state.unexpected.push(command);
          throw new Error(`E_CURRENT_WEB_UNMAPPED_IPC:${command}`);
      }
    },
  };
  return { evidenceKind: "current-web-production-bundle-mocked-ipc", nativeGameplayAcceptance: false, bridge, state,
    count(command) { return Object.hasOwn(observations, command) ? observations[command].count : 0; },
    inputCheckpoint() { return { documentId, worldEpoch: state.epoch, inputCount: observations.formal_submit_input.count }; },
    callsFor(command) { return highRate.has(command) ? state.calls.filter(call => call.command === command)
      : Object.hasOwn(controlCalls, command) ? controlCalls[command] : []; },
    captureEvidence() {
      // Keep bounded examples and exact monotonic counts, rather than serializing
      // thousands of repeated samples during teardown of an already slow page.
      return { observations: clone(observations), totalCalls: state.totalCalls, observerErrors,
        ledgerDropped: state.ledgerDropped,
        calls: clone([...Object.values(controlCalls).flat(), ...firstInputs, ...state.calls.slice(-32)]),
        callLedgerSampled: true, unexpected: state.unexpected.slice(0, 64),
        unexpectedCount: state.unexpectedCount };
    },
    block(command) { state.blocked[command] = true; },
    release(command) {
      delete state.blocked[command];
      for (const request of pending.get(command) ?? []) request.resolve();
      pending.delete(command);
    },
    failNext(command, message = "E_MOCK_REQUEST_REJECTED") { state.failures[command] = message; },
  };
}

export function mockBootstrapScript(options = {}, streamId = null) {
  if (streamId !== null && (typeof streamId !== "string" || !/^[a-zA-Z0-9-]{16,80}$/.test(streamId))) throw new Error("E_INPUT_OBSERVER_STREAM");
  return `(() => {
    const streamId = ${JSON.stringify(streamId)};
    const documentId = streamId ? window.crypto.randomUUID() : null;
    const publisher = streamId ? (${createInputProgressPublisher.toString()})(record =>
      window[${JSON.stringify(INPUT_PROGRESS_BINDING)}]({ ...record, streamId })) : null;
    const mock = (${createMockBridge.toString()})(${JSON.stringify(options)}, publisher ? record => publisher.publish(record) : null, documentId);
    window.isTauri = true; window.__TAURI_INTERNALS__ = mock.bridge;
    window.__CURRENT_WEB_MOCK__ = mock;
    window.__CURRENT_WEB_INPUT_OBSERVER__ = publisher ? Object.freeze({ dispose: publisher.dispose, diagnostics: publisher.diagnostics }) : null;
  })();`;
}
