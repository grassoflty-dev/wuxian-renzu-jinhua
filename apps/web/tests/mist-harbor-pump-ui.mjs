import test from "node:test";
import assert from "node:assert/strict";
import { deriveHudState } from "../dist/ui/Hud.js";
import { deriveMistHarborPumpStatus } from "../dist/ui/components/MistHarborPumpStatus.js";
import { dispatchInteractable, interactionErrorText } from "../dist/game/SceneInteraction.js";

function snapshot(overrides = {}) {
  const control = {
    entityId: "mh_pump_control_primary", kind: "pump_control", active: true,
    transform: { positionM: { xM: 17, yM: 0, zM: 7 }, yawRad: 0 },
  };
  return {
    kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "mist_harbor", sceneId: "mh_pump_station", checkpointId: null,
    worldEpoch: 4, serverTick: 8, authorityRevision: 8, ackSeq: 8,
    player: { entityId: "player", transform: { positionM: { xM: 14.5, yM: 0, zM: 7 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 100, maxHp: 100,
      currentEnergy: 50, maxEnergy: 50, facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [], doors: [], interactables: [control], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: [] },
    progression: { schemaVersion: 1, currentWorldId: "mist_harbor", eventSeq: 0, worlds: [] },
    mistHarborPump: { state: "ready" },
    ...overrides,
  };
}

test("the authored control is an F target only when the Rust snapshot permits it", () => {
  const ready = snapshot();
  assert.equal(deriveHudState(ready).interactionId, "mh_pump_control_primary");
  assert.equal(deriveHudState(ready).interactionText, "[F] 启动排水泵");
  assert.equal(deriveHudState(snapshot({ player: { ...ready.player,
    transform: { positionM: { xM: 14.499, yM: 0, zM: 7 }, yawRad: 0 } } })).interactionId, null);
  assert.equal(deriveHudState(snapshot({ interactables: [{ ...ready.interactables[0], active: false }] })).interactionId, null);
  assert.equal(deriveHudState(snapshot({ sceneId: "mh_breakwater" })).interactionId, null);
  assert.equal(deriveHudState(snapshot({ worldId: "grey_hive" })).interactionId, null);
  assert.equal(deriveHudState(snapshot({ interactables: [{ ...ready.interactables[0], entityId: "forged_pump" }] })).interactionId, null);
  assert.equal(deriveHudState(snapshot({ mistHarborPump: { state: "draining" } })).interactionId, null);
  assert.equal(deriveHudState(snapshot({ mistHarborPump: { state: "drained" } })).interactionId, null);
  assert.equal(deriveHudState(snapshot({ mistHarborPump: { state: "half" } })).interactionId, null);
  // The bridge fix remains useful against the previous Rust snapshot, which did not expose a pump field.
  assert.equal(deriveHudState(snapshot({ mistHarborPump: undefined })).interactionId, "mh_pump_control_primary");
});

test("the station shows only discrete authoritative states and hides missing or stale projections", () => {
  const cases = [
    ["ready", "排水泵待命"], ["draining", "排水进行中"], ["drained", "排水系统已完成"],
  ];
  for (const [state, text] of cases) {
    const view = deriveMistHarborPumpStatus(snapshot({ mistHarborPump: { state } }));
    assert.deepEqual(view, { state, text });
    assert.doesNotMatch(text, /%|0\.25|倒计时/);
  }
  assert.equal(deriveMistHarborPumpStatus(snapshot({ mistHarborPump: undefined })), null);
  assert.equal(deriveMistHarborPumpStatus(snapshot({ mistHarborPump: { state: "half" } })), null);
  assert.equal(deriveMistHarborPumpStatus(snapshot({ sceneId: "mh_drowned_quay" })), null);
  assert.equal(deriveMistHarborPumpStatus(snapshot({ worldId: "return_station" })), null);
  // A resumed/revisited first frame must use the saved Rust state, not a local timer.
  assert.equal(deriveMistHarborPumpStatus(snapshot({ worldEpoch: 9, mistHarborPump: { state: "drained" } }))?.state, "drained");
});

test("F dispatch sends the exact formal interaction and preserves server failure feedback", async () => {
  const calls = [];
  const current = snapshot();
  const client = { interact: async id => { calls.push(id); return { applied: true, snapshot: current }; } };
  await dispatchInteractable(client, current.interactables[0], current.worldEpoch,
    { worldId: current.worldId, sceneId: current.sceneId });
  assert.deepEqual(calls, ["mh_pump_control_primary"]);
  await assert.rejects(dispatchInteractable(client, { ...current.interactables[0], active: false }, current.worldEpoch),
    /E_INTERACTION_INACTIVE/);
  assert.equal(interactionErrorText("E_PUMP_EastBeaconIncomplete"), "先激活东侧航标，再启动排水泵。");
});
