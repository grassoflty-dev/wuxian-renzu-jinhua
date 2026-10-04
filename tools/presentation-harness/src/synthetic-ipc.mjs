export function makeSyntheticView(overrides = {}) {
  const position = overrides.positionM || { xM: 0, yM: 0, zM: 3 };
  const open = overrides.gateOpen === true;
  const revision = overrides.authorityRevision ?? (open ? 2 : 1);
  const view = {
    protocol: "continuous-ipc",
    version: 1,
    schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "grey_hive",
    worldEpoch: 1,
    serverTick: overrides.serverTick ?? revision - 1,
    authorityRevision: revision,
    ackSeq: overrides.ackSeq ?? Math.max(0, revision - 1),
    serverTimeMs: overrides.serverTimeMs ?? revision * 50,
    player: {
      entityId: "player",
      transform: { positionM: position, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 },
      currentHp: 100,
      maxHp: 100,
      currentEnergy: 100,
      maxEnergy: 100
    },
    actors: [{
      entityId: "gh_power_console",
      actorKind: "power_console",
      transform: { positionM: { xM: 2, yM: 0, zM: 3 }, yawRad: 0 },
      active: !open
    }],
    doors: [{
      doorId: "gh_gate_a",
      transform: { positionM: { xM: 0, yM: 0, zM: 8 }, yawRad: 0 },
      open,
      locked: !open
    }],
    capabilities: {
      schemaVersion: 1,
      items: [],
      exploredMap: {
        worldId: "grey_hive",
        playerPositionM: position,
        rooms: [{
          roomId: "synthetic-room",
          outlineM: [
            { xM: -4, yM: 0, zM: -3 }, { xM: 4, yM: 0, zM: -3 },
            { xM: 4, yM: 0, zM: 10 }, { xM: -4, yM: 0, zM: 10 }
          ]
        }],
        connections: [],
        objectives: [{ objectiveId: "hive_power", positionM: { xM: 2, yM: 0, zM: 3 } }]
      },
      enemyVitals: [],
      rearView: { granted: false, grantId: null, grantedAtRevision: null }
    },
    progression: {
      schemaVersion: 1,
      currentWorldId: "grey_hive",
      eventSeq: open ? 1 : 0,
      worlds: [{
        worldId: "grey_hive",
        completed: false,
        firstCompletion: false,
        visitId: 1,
        cycleId: 1,
        revisitCount: 0,
        completedEvents: open ? ["hive_power"] : []
      }]
    }
  };
  return view;
}

export function makeSyntheticSnapshot(view) {
  return {
    kind: "full",
    schemaVersion: view.schemaVersion,
    worldId: view.worldId,
    worldEpoch: view.worldEpoch,
    serverTick: view.serverTick,
    authorityRevision: view.authorityRevision,
    ackSeq: view.ackSeq,
    view
  };
}

export function makeSyntheticReceipt(commandId, view, outcome = {}) {
  return {
    commandId,
    applied: outcome.applied ?? true,
    alreadyApplied: outcome.alreadyApplied ?? false,
    errorCode: outcome.errorCode ?? null,
    worldEpoch: view.worldEpoch,
    serverTick: view.serverTick,
    authorityRevision: view.authorityRevision,
    snapshot: makeSyntheticSnapshot(view)
  };
}

export function syntheticBootstrapScript() {
  const hub = makeSyntheticView();
  const powerBefore = makeSyntheticView();
  const powerAfter = makeSyntheticView({ gateOpen: true, authorityRevision: 2, serverTick: 1, ackSeq: 1 });
  return `(() => {
    const hub = ${JSON.stringify(hub)};
    const powerBefore = ${JSON.stringify(powerBefore)};
    const powerAfter = ${JSON.stringify(powerAfter)};
    let current = powerBefore;
    const calls = [];
    const wrap = view => ({ kind: "full", schemaVersion: view.schemaVersion, worldId: view.worldId,
      worldEpoch: view.worldEpoch, serverTick: view.serverTick, authorityRevision: view.authorityRevision,
      ackSeq: view.ackSeq, view });
    const receipt = (commandId, view, outcome = {}) => ({ commandId, applied: outcome.applied ?? true,
      alreadyApplied: false, errorCode: outcome.errorCode ?? null, worldEpoch: view.worldEpoch,
      serverTick: view.serverTick, authorityRevision: view.authorityRevision, snapshot: wrap(view) });
    const invoke = async (command, args = {}) => {
      calls.push({ command, args });
      if (command === "formal_snapshot") return wrap(hub);
      if (command === "formal_list_save_slots") return [];
      if (command === "formal_new") { current = powerBefore; return receipt(command, current); }
      if (command === "formal_submit_input") return receipt(command, current);
      if (command === "formal_interact") { current = powerAfter; return { applied: true, alreadyApplied: false, errorCode: null, events: ["hive_power"], receipt: receipt(command, current), view: current }; }
      return Promise.reject(new Error("E_SYNTHETIC_IPC_UNMAPPED:" + command));
    };
    window.__PRESENTATION_HARNESS__ = { calls, source: "inline-synthetic-preflight", getCurrent: () => current };
    window.__TAURI__ = { core: { invoke } };
  })();`;
}
