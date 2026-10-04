import test from "node:test";
import assert from "node:assert/strict";
import { deriveEnhancementStatus, EnhancementStatusHud } from "../dist/ui/EnhancementStatusHud.js";

function snapshot(capabilityId, overrides = {}) {
  return {
    kind: "full", protocolVersion: 3, schemaVersion: "scene-v3/1", worldId: "grey_hive",
    sceneId: "gh_entry", checkpointId: null, worldEpoch: 1, serverTick: 10, authorityRevision: 10, ackSeq: 10,
    player: { entityId: "player", transform: { positionM: { xM: 1, yM: 0, zM: 2 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 78, maxHp: 100, currentEnergy: 40, maxEnergy: 50,
      facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: {
      schemaVersion: 1, firstEnhancementChoice: capabilityId ?? null,
      items: capabilityId ? [{ capabilityId, granted: true, selected: true, cooldownRemainingMs: 0 }] : [],
      exploredMap: { worldId: "grey_hive", playerPositionM: { xM: 1, yM: 0, zM: 2 }, rooms: [], connections: [], objectives: [] },
      enemyVitals: [],
      rearView: { granted: false, grantId: null, grantedAtRevision: null },
    },
    progression: { schemaVersion: 1, currentWorldId: "grey_hive", eventSeq: 1,
      worlds: [{ worldId: "grey_hive", completed: true, firstCompletion: true }] },
    ...overrides,
  };
}

test("no enhancement status is shown before Grey Hive first clear or without a selected granted choice", () => {
  const selected = snapshot("information.local_map_i");
  assert.equal(deriveEnhancementStatus({ ...selected, progression: { ...selected.progression, worlds: [] } }), null);
  assert.equal(deriveEnhancementStatus(snapshot(null)), null);
  const notGranted = snapshot("information.local_map_i");
  notGranted.capabilities.items[0].granted = false;
  assert.equal(deriveEnhancementStatus(notGranted), null);
  const notSelected = snapshot("information.local_map_i");
  notSelected.capabilities.items[0].selected = false;
  assert.equal(deriveEnhancementStatus(notSelected).availability, "已获得 · 请在能力管理中配置");
});

test("Local Map lists only room, connection, and objective data from the matching server projection", () => {
  const base = snapshot("information.local_map_i");
  const projected = snapshot("information.local_map_i", { capabilities: { ...base.capabilities,
    exploredMap: { ...base.capabilities.exploredMap,
      rooms: [{ roomId: "room-server-known", outlineM: [] }],
      connections: [{ connectionId: "door-server-known", fromRoomId: "A", toRoomId: "B" }],
      objectives: [{ objectiveId: "objective-server-known", positionM: { xM: 3, yM: 0, zM: 4 } }],
    },
  } });
  assert.deepEqual(deriveEnhancementStatus(projected), {
    capabilityId: "information.local_map_i", label: "局部地图", availability: "已授权 · 已接收探索投影",
    details: ["已探索区域：room-server-known", "已知通路：A → B", "已知目标：objective-server-known"],
  });
  const empty = deriveEnhancementStatus(base);
  assert.equal(empty.details.length, 0, "no map feature data is fabricated for an empty projection");
  const otherWorld = snapshot("information.local_map_i", { worldId: "mist_harbor", capabilities: base.capabilities });
  assert.equal(deriveEnhancementStatus(otherWorld).details.length, 0, "Grey Hive map data cannot leak across worlds");
});

test("Rear View reports only matching server authorization and never synthesizes a view or enemy data", () => {
  const base = snapshot("perception.rear_view_i");
  const denied = deriveEnhancementStatus(base);
  assert.equal(denied.availability, "尚未收到服务端授权");
  assert.deepEqual(denied.details, []);
  base.capabilities.rearView = { granted: true, grantId: "perception.rear_view_i", grantedAtRevision: 9 };
  const authorized = deriveEnhancementStatus(base);
  assert.equal(authorized.availability, "服务端授权有效");
  assert.deepEqual(authorized.details, [], "authorization metadata is not presented as a rendered rear-view image");
  base.capabilities.rearView.grantId = "forged-id";
  assert.equal(deriveEnhancementStatus(base).availability, "尚未收到服务端授权");
});

test("Regeneration shows authorization and only the current authoritative HP snapshot", () => {
  const base = snapshot("body.regeneration_i");
  assert.deepEqual(deriveEnhancementStatus(base), {
    capabilityId: "body.regeneration_i", label: "再生能力", availability: "已授权",
    details: ["HP（权威快照）：78 / 100"],
  });
  const changed = snapshot("body.regeneration_i", { player: { ...base.player, currentHp: 63 } });
  assert.equal(deriveEnhancementStatus(changed).details[0], "HP（权威快照）：63 / 100");
});

test("enhancement HUD replaces stale projections and reset clears state on Hub or session exit", () => {
  const originalDocument = globalThis.document;
  globalThis.document = { createElement: tag => ({ tag, textContent: "" }) };
  try {
    const root = { hidden: true, children: [], renders: 0, replaceChildren(...children) { this.renders++; this.children = children; } };
    const hud = new EnhancementStatusHud(root);
    hud.apply(snapshot("body.regeneration_i"));
    hud.apply(snapshot("body.regeneration_i"));
    assert.equal(root.hidden, false);
    assert.match(root.children[0].textContent, /再生能力/);
    assert.match(root.children[1].textContent, /78 \/ 100/);
    assert.equal(root.renders, 1, "unchanged snapshots do not churn the accessible panel");
    hud.apply(snapshot(null));
    assert.equal(root.hidden, true);
    assert.deepEqual(root.children, []);
    hud.apply(snapshot("perception.rear_view_i"));
    hud.reset();
    assert.equal(root.hidden, true);
    assert.deepEqual(root.children, []);
  } finally {
    globalThis.document = originalDocument;
  }
});


test("equipment-composed topology is never labelled as actual exploration history", () => {
  const value = snapshot("information.local_map_i");
  value.capabilities.mapTopologyAuthorized = true;
  value.capabilities.exploredMap.rooms = [{ roomId: "remotely-revealed", outlineM: [] }];
  const before = structuredClone(value);
  assert.deepEqual(deriveEnhancementStatus(value).details, ["获准地图区域：remotely-revealed"]);
  assert.deepEqual(value, before);
});
