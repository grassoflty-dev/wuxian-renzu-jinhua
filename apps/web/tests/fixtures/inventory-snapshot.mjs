export function inventorySnapshot(overrides = {}) {
  return {
    kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2", worldId: "grey_hive",
    sceneId: "gh_entry", checkpointId: null, worldEpoch: 4, serverTick: 10, authorityRevision: 10, ackSeq: 1,
    player: { entityId: "player", transform: { positionM: { xM: 1, yM: 0, zM: 2 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 78, maxHp: 100, currentEnergy: 40, maxEnergy: 50,
      facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: [] },
    progression: { schemaVersion: 1, currentWorldId: "grey_hive", eventSeq: 1, worlds: [] },
    build: { schemaVersion: 1, revision: 2, items: [{ itemId: "rear_view_lens", quantity: 2, slotId: "lens",
      label: "权威镜片", description: "由服务端提供的说明", equipped: false, releaseEligible: false }], equipment: [] },
    ...overrides,
  };
}
export function equippedSnapshot(source = inventorySnapshot()) {
  return { ...source, authorityRevision: source.authorityRevision + 1,
    build: { ...source.build, revision: source.build.revision + 1,
      items: source.build.items.map(item => ({ ...item, equipped: true })),
      equipment: [{ slotId: "lens", itemId: "rear_view_lens", label: "权威镜片" }] } };
}
export function receipt(snapshot = equippedSnapshot(), overrides = {}) {
  return { commandId: "build:test", applied: true, alreadyApplied: false, errorCode: null,
    worldEpoch: snapshot.worldEpoch, serverTick: snapshot.serverTick, authorityRevision: snapshot.authorityRevision,
    snapshot, ...overrides };
}
