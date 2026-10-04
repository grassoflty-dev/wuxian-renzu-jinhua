import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { PumpSceneFeedbackModel, PUMP_SCENE_FEEDBACK_TEMPORARY_VISUAL } from "../dist/renderer/PumpSceneFeedbackModel.js";

const snapshot = (overrides = {}) => ({
  kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
  worldId: "mist_harbor", sceneId: "mh_pump_station", checkpointId: null,
  worldEpoch: 4, serverTick: 8, authorityRevision: 8, ackSeq: 8,
  player: { entityId: "player", transform: { positionM: { xM: 14, yM: 0, zM: 7 }, yawRad: 0 },
    velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 100, maxHp: 100,
    currentEnergy: 50, maxEnergy: 50, facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
  actors: [], doors: [], interactables: [], hazards: [], objectives: [],
  capabilities: { schemaVersion: 1, items: [] },
  progression: { schemaVersion: 1, currentWorldId: "mist_harbor", eventSeq: 0, worlds: [] },
  mistHarborPump: { state: "ready" },
  ...overrides,
});

test("the three authoritative states project distinct, discrete local feedback", () => {
  const model = new PumpSceneFeedbackModel();
  const ready = model.project(snapshot(), 100, false);
  assert.equal(ready.state, "ready");
  assert.equal(ready.indicatorColor, 0xffb34d);
  assert.equal(ready.machineOffsetX, 0);
  assert.equal(ready.flowAlpha, 0);

  const drainingStart = model.project(snapshot({ mistHarborPump: { state: "draining" } }), 200, false);
  const draining = model.project(snapshot({ mistHarborPump: { state: "draining" } }), 300, false);
  assert.equal(draining.state, "draining");
  assert.equal(draining.indicatorColor, 0x61ddcf);
  assert.notEqual(draining.machineOffsetX, 0);
  assert.ok(draining.flowAlpha > 0);
  assert.equal(draining.motionPhase, 100 / 1_350);
  assert.equal(drainingStart.motionPhase, 0);

  const drained = model.project(snapshot({ mistHarborPump: { state: "drained" } }), 400, false);
  const drainedLater = model.project(snapshot({ mistHarborPump: { state: "drained" } }), 10_000, false);
  assert.equal(drained.state, "drained");
  assert.equal(drained.flowAlpha, 0);
  assert.equal(drained.machineOffsetX, 0);
  assert.equal(drained.indicatorColor, 0x61ddcf);
  assert.deepEqual(drainedLater, drained);
  assert.equal(PUMP_SCENE_FEEDBACK_TEMPORARY_VISUAL, true);
});

test("old, malformed, and foreign snapshots fail closed and reset active feedback", () => {
  const model = new PumpSceneFeedbackModel();
  assert.notEqual(model.project(snapshot({ mistHarborPump: { state: "draining" } }), 10, false), null);
  for (const stale of [
    snapshot({ mistHarborPump: undefined }),
    snapshot({ mistHarborPump: { state: "half-drained" } }),
    snapshot({ mistHarborPump: { state: 7 } }),
    snapshot({ worldId: "grey_hive" }),
    snapshot({ sceneId: "mh_drowned_quay" }),
    snapshot({ protocolVersion: 2 }),
    snapshot({ worldEpoch: Number.NaN }),
  ]) {
    assert.equal(model.project(stale, 20, false), null);
  }
  const restarted = model.project(snapshot({ mistHarborPump: { state: "draining" } }), 500, false);
  assert.equal(restarted.motionPhase, 0);
});

test("scene and epoch changes reset the animation phase without carrying stale state", () => {
  const model = new PumpSceneFeedbackModel();
  model.project(snapshot({ mistHarborPump: { state: "draining" } }), 100, false);
  assert.ok(model.project(snapshot({ mistHarborPump: { state: "draining" } }), 250, false).motionPhase > 0);
  assert.equal(model.project(snapshot({ sceneId: "mh_breakwater", mistHarborPump: { state: "draining" } }), 300, false), null);
  assert.equal(model.project(snapshot({ worldEpoch: 5, mistHarborPump: { state: "draining" } }), 2_000, false).motionPhase, 0);
  assert.equal(model.project(snapshot({ worldEpoch: 6, mistHarborPump: { state: "draining" } }), 3_000, true).motionPhase, 0);
});

test("reduced motion leaves the state indicator static and suppresses machinery and flow motion", () => {
  const model = new PumpSceneFeedbackModel();
  const first = model.project(snapshot({ mistHarborPump: { state: "draining" } }), 100, true);
  const later = model.project(snapshot({ mistHarborPump: { state: "draining" } }), 4_000, true);
  assert.equal(first.motionPhase, 0);
  assert.equal(first.machineOffsetX, 0);
  assert.equal(first.flowAlpha, 0);
  assert.equal(first.indicatorColor, 0x61ddcf);
  assert.deepEqual(later, first);
});

test("duplicate snapshot projection is deterministic and renderer feedback stays presentation-only", async () => {
  const model = new PumpSceneFeedbackModel();
  const value = snapshot({ mistHarborPump: { state: "draining" } });
  assert.deepEqual(model.project(value, 1_250, false), model.project(value, 1_250, false));
  const renderer = await readFile(new URL("../src/renderer/WorldRenderer.ts", import.meta.url), "utf8");
  assert.match(renderer, /this\.pumpSceneFeedbackModel\.project\(snapshot, frameTimeMs, reduceFogMotion\)/);
  assert.match(renderer, /renderPumpSceneFeedback\(pumpFeedback, camera\)/);
  assert.match(renderer, /xM: 17, yM: 0, zM: 7/);
  assert.match(renderer, /xM: 17, yM: 0, zM: 8/);
  assert.match(renderer, /get\("L4_DYNAMIC_PROPS"\)/);
  assert.match(renderer, /get\("L7_VFX"\)/);
  assert.match(renderer, /graphic\.destroy\(\)/);
  const feedback = renderer.slice(renderer.indexOf("  private renderPumpSceneFeedback("), renderer.indexOf("  private clearPumpSceneFeedback("));
  assert.doesNotMatch(feedback, /waterRipple|wetFloor|exploredMap|soundCue|playSound/i);
});
