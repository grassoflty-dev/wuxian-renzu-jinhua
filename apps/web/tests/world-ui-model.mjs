import test from "node:test";
import assert from "node:assert/strict";
import { Container } from "pixi.js";
import { WorldRenderer } from "../dist/renderer/WorldRenderer.js";
import { projectWorldUi, WORLD_UI_INTERACTION_RANGE_M } from "../dist/renderer/WorldUiModel.js";
import { projectWorldPoint } from "../dist/renderer/CameraModel.js";

function snapshot() {
  const positionM = { xM: 0, yM: 0, zM: 0 };
  return {
    kind: "full", protocolVersion: 3, schemaVersion: "scene-v3/1", worldId: "grey_hive", sceneId: "gh_power",
    checkpointId: null, worldEpoch: 2, serverTick: 1, authorityRevision: 1, ackSeq: 0,
    player: { entityId: "player", transform: { positionM, yawRad: 0 }, velocityMps: positionM,
      currentHp: 1, maxHp: 1, currentEnergy: 1, maxEnergy: 1, facingX: 1, facingZ: 0, aimX: 1, aimZ: 0, actionState: "idle" },
    actors: [], doors: [], hazards: [],
    interactables: [
      { entityId: "near", kind: "terminal", active: true, transform: { positionM: { xM: 1, yM: 0, zM: 0 }, yawRad: 0 } },
      { entityId: "other_near", kind: "terminal", active: true, transform: { positionM: { xM: 2, yM: 0, zM: 0 }, yawRad: 0 } },
      { entityId: "inactive", kind: "terminal", active: false, transform: { positionM: { xM: 1, yM: 0, zM: 0 }, yawRad: 0 } },
      { entityId: "far", kind: "terminal", active: true, transform: { positionM: { xM: WORLD_UI_INTERACTION_RANGE_M + 1, yM: 0, zM: 0 }, yawRad: 0 } },
      { entityId: "unknown", kind: "unknown_kind", active: true, transform: { positionM, yawRad: 0 } },
    ],
    objectives: [], capabilities: { schemaVersion: 1, items: [],
      exploredMap: { worldId: "grey_hive", playerPositionM: positionM, rooms: [], connections: [],
        objectives: [{ objectiveId: "known", positionM: { xM: 8, yM: 0, zM: 3 } }] } },
    progression: { schemaVersion: 1, currentWorldId: "grey_hive", eventSeq: 0, worlds: [] },
  };
}

const camera = { width: 800, height: 600, origin: { xM: 0, yM: 0, zM: 0 } };

test("only the nearest actionable target inside the authoritative prompt range gets an F marker", () => {
  const markers = projectWorldUi(snapshot(), camera);
  assert.deepEqual(markers.map(marker => marker.id), ["interaction:near"]);
  assert.equal(markers[0].kind, "interaction");
});

test("known objective points appear only with selected granted local map projection for this world", () => {
  const value = snapshot();
  value.capabilities.items.push({ capabilityId: "information.local_map_i", granted: true, selected: true });
  assert.deepEqual(projectWorldUi(value, camera).map(marker => marker.id), ["interaction:near", "known-objective:known"]);
  value.capabilities.items[0].granted = false;
  assert.deepEqual(projectWorldUi(value, camera).map(marker => marker.id), ["interaction:near"]);
  value.capabilities.items[0].granted = true;
  value.capabilities.exploredMap.worldId = "mist_harbor";
  assert.deepEqual(projectWorldUi(value, camera).map(marker => marker.id), ["interaction:near"]);
});

test("authorized map markers can be distant hints but never become interaction prompts", () => {
  const value = snapshot();
  value.capabilities.items.push({ capabilityId: "information.local_map_i", granted: true, selected: true });
  const marker = projectWorldUi(value, camera).find(item => item.kind === "known_objective");
  assert.ok(marker);
  assert.equal(marker.id, "known-objective:known");
  assert.ok(marker.screenX > camera.width / 2);
});

function harborSnapshot(active = true) {
  const value = snapshot();
  value.worldId = "mist_harbor";
  value.sceneId = "mh_signal_yard";
  value.capabilities.exploredMap.worldId = "mist_harbor";
  value.capabilities.items.push({ capabilityId: "information.local_map_i", granted: true, selected: true });
  value.hazards.push({ entityId: "mh_signal_interference_region", kind: "signal_interference_zone", active,
    transform: { positionM: { xM: 16, yM: 0, zM: 7.5 }, yawRad: 0 } });
  return value;
}

test("active authoritative signal zone replaces precise objectives with one area hint while F stays independent", () => {
  const value = harborSnapshot(true);
  const markers = projectWorldUi(value, camera);
  assert.deepEqual(markers.map(marker => marker.id), ["interaction:near", "signal-region:mh_signal_interference_region"]);
  const area = markers.find(marker => marker.kind === "signal_region");
  const projected = projectWorldPoint(value.hazards[0].transform.positionM, camera);
  assert.equal(area.screenX, projected.x);
  assert.equal(area.screenY, projected.y);
  assert.equal(area.footY, projected.footY);
});

test("outside the authoritative zone precise objective positions remain available", () => {
  assert.deepEqual(projectWorldUi(harborSnapshot(false), camera).map(marker => marker.id),
    ["interaction:near", "known-objective:known"]);
});

test("selected Local Map keeps an approximate signal area visible before any objective is mapped", () => {
  const value = harborSnapshot();
  value.capabilities.exploredMap.objectives = [];
  assert.deepEqual(projectWorldUi(value, camera).map(marker => marker.id),
    ["interaction:near", "signal-region:mh_signal_interference_region"]);
});

test("missing, duplicate, wrong-kind, and malformed signal hazards fail closed", () => {
  const cases = [
    value => { value.hazards = []; },
    value => { value.hazards.push({ ...value.hazards[0] }); },
    value => { value.hazards[0].kind = "signal_interference_static_marker"; },
    value => { value.hazards[0].transform.positionM.xM = Number.NaN; },
    value => { value.hazards[0].active = undefined; },
  ];
  for (const mutate of cases) {
    const value = harborSnapshot();
    mutate(value);
    assert.deepEqual(projectWorldUi(value, camera).map(marker => marker.id), ["interaction:near"]);
  }
});

test("the signal rule is scoped to Mist Harbor Signal Yard", () => {
  const wrongWorld = harborSnapshot();
  wrongWorld.worldId = "grey_hive";
  wrongWorld.capabilities.exploredMap.worldId = "grey_hive";
  assert.deepEqual(projectWorldUi(wrongWorld, camera).map(marker => marker.id),
    ["interaction:near", "known-objective:known"]);
  const wrongScene = harborSnapshot();
  wrongScene.sceneId = "mh_drowned_quay";
  assert.deepEqual(projectWorldUi(wrongScene, camera).map(marker => marker.id),
    ["interaction:near", "known-objective:known"]);
});

test("an unavailable or unselected Local Map reveals neither exact objectives nor signal area", () => {
  for (const mutate of [
    value => { value.capabilities.items[0].granted = false; },
    value => { value.capabilities.items[0].selected = false; },
    value => { value.capabilities.items = []; },
  ]) {
    const value = harborSnapshot();
    mutate(value);
    assert.deepEqual(projectWorldUi(value, camera).map(marker => marker.id), ["interaction:near"]);
  }
});

function addEnemyVital(value, tier = "healthy", id = "enemy-01") {
  value.actors = [{ entityId: id, entityType: "enemy.grey_hive.infected", actorKind: "enemy", active: true,
    transform: { positionM: { xM: 3, yM: 0, zM: 4 }, yawRad: 0 } }];
  value.capabilities.enemyVitals = [{ entityId: id, tier }];
  return value;
}

function enemyVitalMarkers(value, actorPositions) {
  return projectWorldUi(value, camera, actorPositions).filter(marker => marker.kind === "enemy_vital");
}

test("Enemy Vitals needs the granted capability and maps only the four server tiers to text", () => {
  const value = addEnemyVital(snapshot(), "healthy");
  assert.deepEqual(enemyVitalMarkers(value).map(marker => marker.label), []);
  value.capabilities.items.push({ capabilityId: "information.enemy_vitals_basic", granted: false, selected: false });
  assert.deepEqual(enemyVitalMarkers(value).map(marker => marker.label), []);
  value.capabilities.items[0].granted = true;

  const labels = [
    ["healthy", "Healthy"],
    ["wounded", "Wounded"],
    ["severelyWounded", "Severe"],
    ["critical", "Critical"],
  ];
  for (const [tier, label] of labels) {
    value.capabilities.enemyVitals[0].tier = tier;
    assert.deepEqual(enemyVitalMarkers(value).map(marker => marker.label), [label]);
  }
});

test("Enemy Vitals fails closed for missing, duplicate, malformed, or unmatched records", () => {
  const mutations = [
    value => { delete value.capabilities.enemyVitals; },
    value => { value.capabilities.enemyVitals = null; },
    value => { value.capabilities.enemyVitals[0].entityId = ""; },
    value => { value.capabilities.enemyVitals[0].tier = "1347 / 2000"; },
    value => { value.capabilities.enemyVitals[0].tier = "toString"; },
    value => { value.capabilities.enemyVitals.push({ ...value.capabilities.enemyVitals[0] }); },
    value => { value.actors.push({ ...value.actors[0] }); },
    value => { value.actors[0].active = false; },
    value => { value.actors[0].actorKind = "player"; },
    value => { value.actors[0].entityId = value.player.entityId; },
    value => { value.actors = []; },
    value => { value.actors[0].transform.positionM.xM = Number.NaN; },
    value => { value.actors[0].entityId = "different-actor"; },
  ];
  for (const mutate of mutations) {
    const value = snapshot();
    value.capabilities.items.push({ capabilityId: "information.enemy_vitals_basic", granted: true, selected: false });
    addEnemyVital(value);
    mutate(value);
    assert.deepEqual(enemyVitalMarkers(value), [], "malformed or ambiguous vitals must not render");
  }
});

test("Enemy Vital markers follow current actor positions and coexist with interaction and map markers", () => {
  const value = addEnemyVital(snapshot(), "wounded");
  value.capabilities.items.push(
    { capabilityId: "information.enemy_vitals_basic", granted: true, selected: false },
    { capabilityId: "information.local_map_i", granted: true, selected: true },
  );
  const currentPosition = { xM: 5, yM: 0, zM: 6 };
  value.actors[0].transform.positionM = currentPosition;
  const interpolatedPosition = { xM: 8, yM: 0, zM: 9 };
  const actorPositions = new Map([["enemy-01", interpolatedPosition]]);
  const markers = projectWorldUi(value, camera, actorPositions);
  const vital = markers.find(marker => marker.kind === "enemy_vital");
  assert.ok(vital);
  const expected = projectWorldPoint(interpolatedPosition, camera);
  assert.equal(vital.screenX, expected.x);
  assert.equal(vital.screenY, expected.y);
  assert.notDeepEqual(interpolatedPosition, currentPosition);
  assert.deepEqual(enemyVitalMarkers(value, new Map([["enemy-01", { xM: Number.NaN, yM: 0, zM: 0 }]])), []);
  const resizedCamera = { ...camera, width: 1000, height: 700 };
  const resizedVital = projectWorldUi(value, resizedCamera).find(marker => marker.kind === "enemy_vital");
  const resizedExpected = projectWorldPoint(currentPosition, resizedCamera);
  assert.equal(resizedVital.screenX, resizedExpected.x);
  assert.equal(resizedVital.screenY, resizedExpected.y);
  assert.ok(markers.some(marker => marker.kind === "interaction"));
  assert.ok(markers.some(marker => marker.kind === "known_objective"));
});

test("WorldRenderer updates tier text and position, then releases vital labels when actors or epochs leave", async () => {
  const renderer = new WorldRenderer({});
  const layer = new Container();
  renderer.layerContainers.set("L8_WORLD_UI", layer);
  const value = snapshot();
  addEnemyVital(value);
  value.interactables = [];
  value.capabilities.items.push({ capabilityId: "information.enemy_vitals_basic", granted: true, selected: false });
  const firstPosition = { xM: 3, yM: 0, zM: 4 };
  renderer.actorPositionsForWorldUi = new Map([["enemy-01", firstPosition]]);
  renderer.renderWorldUi(value, camera);
  const record = renderer.worldUi.get("enemy-vital:enemy-01");
  assert.ok(record);
  assert.equal(record.label.text, "Healthy");
  assert.equal(layer.children.length, 1);
  const firstProjected = projectWorldPoint(firstPosition, camera);
  assert.deepEqual([record.container.position.x, record.container.position.y],
    [firstProjected.x, firstProjected.y - 34]);

  value.capabilities.enemyVitals[0].tier = "critical";
  const currentPosition = { xM: 7, yM: 0, zM: 2 };
  value.actors[0].transform.positionM = currentPosition;
  const interpolatedPosition = { xM: 10, yM: 0, zM: 5 };
  renderer.actorPositionsForWorldUi = new Map([["enemy-01", interpolatedPosition]]);
  renderer.renderWorldUi(value, camera);
  assert.equal(renderer.worldUi.get("enemy-vital:enemy-01"), record);
  assert.equal(record.label.text, "Critical");
  const currentProjected = projectWorldPoint(interpolatedPosition, camera);
  assert.deepEqual([record.container.position.x, record.container.position.y],
    [currentProjected.x, currentProjected.y - 34]);

  value.actors[0].active = false;
  renderer.renderWorldUi(value, camera);
  assert.equal(renderer.worldUi.size, 0);
  assert.equal(layer.children.length, 0);
  value.actors[0].active = true;
  renderer.renderWorldUi(value, camera);
  assert.equal(layer.children.length, 1);
  value.actors = [];
  renderer.renderWorldUi(value, camera);
  assert.equal(renderer.worldUi.size, 0);
  assert.equal(layer.children.length, 0);
  addEnemyVital(value, "critical");
  renderer.renderWorldUi(value, camera);
  assert.equal(layer.children.length, 1);
  value.capabilities.items[0].granted = false;
  renderer.renderWorldUi(value, camera);
  assert.equal(renderer.worldUi.size, 0);
  assert.equal(layer.children.length, 0);
  value.capabilities.items[0].granted = true;
  renderer.renderWorldUi(value, camera);
  assert.equal(layer.children.length, 1);
  renderer.expectSceneIdentity({ worldId: "grey_hive", sceneId: "gh_power", worldEpoch: 3 });
  await renderer.sceneInvalidationCleanup;
  assert.equal(renderer.worldUi.size, 0);
  assert.equal(layer.children.length, 0);
});
