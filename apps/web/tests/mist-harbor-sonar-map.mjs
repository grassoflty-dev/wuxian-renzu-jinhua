import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { deriveMistHarborSonarMap, renderMistHarborSonarMap } from "../dist/ui/components/MistHarborSonarMap.js";

const testDir = dirname(fileURLToPath(import.meta.url));
const point = (x, z) => ({ xM: x, yM: 0, zM: z });
const room = (roomId, minX, maxX) => ({ roomId,
  outlineM: [point(minX, 0), point(maxX, 0), point(maxX, 10), point(minX, 10)] });
const localMap = () => ({ capabilityId: "information.local_map_i", granted: true, selected: true });
function snapshot() {
  return {
    worldId: "mist_harbor", sceneId: "mh_fog_pier",
    capabilities: { items: [localMap()], exploredMap: {
      worldId: "mist_harbor", playerPositionM: point(5, 5),
      rooms: [room("known-pier", 0, 10), room("known-warehouse", 10, 20)],
      connections: [{ connectionId: "known-passage", fromRoomId: "known-pier", toRoomId: "known-warehouse" }],
      objectives: [{ objectiveId: "mapped-beacon", positionM: point(17, 5) }],
    } },
    objectives: [{ objectiveId: "secret-unmapped", positionM: point(99, 99) }],
    interactables: [{ entityId: "static-signal-marker", kind: "signal_interference_static_marker",
      transform: { positionM: point(88, 88) }, active: true }],
    hazards: [],
  };
}
const clone = value => structuredClone(value);

test("the shared mission panel only adds sonar for Mist Harbor", async () => {
  const presenter = await readFile(resolve(testDir, "../src/ui/CoreUiPresenter.ts"), "utf8");
  assert.match(presenter, /this\.current === "mission" && this\.snapshot\?\.worldId === "mist_harbor"/);
  assert.match(presenter, /renderMistHarborSonarMap\(sonar, this\.snapshot\)/);
  assert.equal(deriveMistHarborSonarMap({ ...snapshot(), worldId: "grey_hive" }), null);
});

test("exactly one granted and selected Local Map row is required", () => {
  for (const items of [[], [{ ...localMap(), granted: false }], [{ ...localMap(), selected: false }],
    [localMap(), localMap()], [localMap(), { ...localMap(), selected: false }]]) {
    const value = snapshot(); value.capabilities.items = items;
    assert.equal(deriveMistHarborSonarMap(value).status, "unauthorized");
    assert.deepEqual(deriveMistHarborSonarMap(value).markers, []);
  }
});

test("empty, foreign, and malformed exploration never draws an inferred chart", () => {
  const changes = [
    value => { delete value.capabilities.exploredMap; },
    value => { value.capabilities.exploredMap.worldId = "grey_hive"; },
    value => { value.capabilities.exploredMap.rooms = []; },
    value => { value.capabilities.exploredMap.rooms[0].outlineM[0].xM = Number.NaN; },
    value => { value.capabilities.exploredMap.rooms[0].outlineM[0].yM = Infinity; },
    value => { value.capabilities.exploredMap.rooms[0].outlineM = [point(0, 0), point(1, 1), point(2, 2)]; },
    value => { value.capabilities.exploredMap.rooms.push(clone(value.capabilities.exploredMap.rooms[0])); },
    value => { value.capabilities.exploredMap.connections[0].toRoomId = "unexplored-room"; },
    value => { value.capabilities.exploredMap.playerPositionM.zM = Number.NaN; },
    value => { value.capabilities.exploredMap.objectives[0].positionM.xM = Number.NaN; },
  ];
  for (const change of changes) {
    const value = snapshot(); change(value);
    const view = deriveMistHarborSonarMap(value);
    assert.equal(view.status, "unexplored");
    assert.deepEqual([view.rooms, view.connections, view.markers], [[], [], []]);
  }
});

test("ready chart is a read-only copy of known rooms, links, player, and mapped objectives", () => {
  const value = snapshot();
  const before = clone(value);
  const view = deriveMistHarborSonarMap(value);
  assert.equal(view.status, "ready");
  assert.deepEqual(view.rooms.map(item => item.id), ["known-pier", "known-warehouse"]);
  assert.equal(view.connections.length, 1);
  assert.deepEqual(view.player, { x: 5, z: 5 });
  assert.deepEqual(view.markers, [{ kind: "objective", position: { x: 17, z: 5 } }]);
  assert.deepEqual(value, before);
});

test("Signal Yard active hazard becomes an area cue; missing or malformed hazard suppresses exact markers", () => {
  const value = snapshot();
  value.sceneId = "mh_signal_yard";
  const hazard = { entityId: "mh_signal_interference_region", kind: "signal_interference_zone",
    active: true, transform: { positionM: point(16, 7.5), yawRad: 0 } };
  value.hazards = [hazard];
  assert.deepEqual(deriveMistHarborSonarMap(value).markers,
    [{ kind: "signal_region", position: { x: 16, z: 7.5 } }]);
  value.hazards[0].active = false;
  assert.deepEqual(deriveMistHarborSonarMap(value).markers,
    [{ kind: "objective", position: { x: 17, z: 5 } }]);
  for (const change of [
    next => { next.hazards = []; },
    next => { next.hazards.push(clone(next.hazards[0])); },
    next => { next.hazards[0].kind = "signal_interference_static_marker"; },
    next => { next.hazards[0].active = "true"; },
    next => { delete next.hazards[0].transform; },
    next => { next.hazards[0].transform.positionM.xM = Number.NaN; },
    next => { next.hazards[0].transform.yawRad = Number.NaN; },
    next => { next.hazards[0].transform.yawRad = Infinity; },
  ]) {
    for (const active of [true, false]) {
      const next = snapshot(); next.sceneId = "mh_signal_yard";
      next.hazards = [{ ...clone(hazard), active }];
      change(next);
      assert.deepEqual(deriveMistHarborSonarMap(next).markers, [],
        `malformed hazard must hide ${active ? "area cue" : "exact target"}`);
    }
  }
});

class Element {
  constructor(tagName) { this.tagName = tagName; this.children = []; this.attributes = {}; this.textContent = ""; this.className = ""; }
  setAttribute(key, value) { this.attributes[key] = value; }
  append(...children) { this.children.push(...children); }
  replaceChildren(...children) { this.children = [...children]; }
}
const walk = (element, predicate) => [element, ...element.children.flatMap(child => walk(child, predicate))].filter(predicate);

test("rendering uses static radar bearings and never draws an unauthorised or static scene marker", () => {
  const previousDocument = globalThis.document;
  globalThis.document = { createElement: tag => new Element(tag), createElementNS: (_ns, tag) => new Element(tag) };
  try {
    const host = new Element("section");
    const value = snapshot();
    renderMistHarborSonarMap(host, value);
    assert.equal(walk(host, item => item.attributes.class === "mh-sonar-room").length, 2);
    assert.equal(walk(host, item => item.attributes.class === "mh-sonar-bearing").length, 4);
    assert.equal(walk(host, item => item.attributes.class === "mh-sonar-objective").length, 1);
    assert.equal(walk(host, item => item.attributes.class === "mh-sonar-player").length, 1);
    value.sceneId = "mh_signal_yard";
    renderMistHarborSonarMap(host, value);
    assert.equal(walk(host, item => item.attributes.class === "mh-sonar-objective").length, 0);
    assert.equal(walk(host, item => item.attributes.class === "mh-sonar-signal").length, 0);
    value.capabilities.exploredMap.rooms = [];
    renderMistHarborSonarMap(host, value);
    assert.equal(walk(host, item => item.tagName === "svg").length, 0);
    assert.ok(walk(host, item => item.textContent === "尚无已探索投影").length > 0);
  } finally { globalThis.document = previousDocument; }
});

test("sonar CSS supports contrast, reduced motion and desktop widths", async () => {
  const css = await readFile(resolve(testDir, "../src/visual-authority.css"), "utf8");
  assert.match(css, /html\[data-contrast="high"\] \.mh-sonar-panel/);
  assert.match(css, /html\[data-motion="reduced"\] \.mh-sonar-panel/);
  assert.match(css, /@media \(prefers-reduced-motion: reduce\)/);
  assert.match(css, /@media \(max-width: 1400px\)/);
  assert.match(css, /@media \(min-width: 1900px\)/);
  assert.match(css, /@media \(min-width: 2400px\)/);
});



test("resolved map authority displays equipment topology without granting Local Map", () => {
  const value = snapshot();
  value.capabilities.items = [{ ...localMap(), granted: false, selected: false }];
  value.capabilities.mapTopologyAuthorized = true;
  value.capabilities.exploredMap.objectives = [];
  const before = clone(value);
  assert.equal(deriveMistHarborSonarMap(value).status, "ready");
  assert.equal(deriveMistHarborSonarMap(value).rooms.length, 2);
  assert.deepEqual(deriveMistHarborSonarMap(value).markers, []);
  assert.deepEqual(value, before);
  for (const denied of [false, null, "true", 1]) {
    value.capabilities.items = [localMap()];
    value.capabilities.mapTopologyAuthorized = denied;
    assert.equal(deriveMistHarborSonarMap(value).status, "unauthorized");
  }
});

test("equipment-only topology never gains signal or objective knowledge from world hazards", () => {
  const value = snapshot();
  value.sceneId = "mh_signal_yard";
  value.capabilities.items = [];
  value.capabilities.mapTopologyAuthorized = true;
  value.capabilities.exploredMap.objectives = [];
  value.hazards = [{ entityId: "mh_signal_interference_region", kind: "signal_interference_zone",
    active: true, transform: { positionM: point(16, 7.5), yawRad: 0 } }];
  assert.equal(deriveMistHarborSonarMap(value).status, "ready");
  assert.deepEqual(deriveMistHarborSonarMap(value).markers, []);
  value.capabilities.mapTopologyAuthorized = false;
  assert.equal(deriveMistHarborSonarMap(value).status, "unauthorized");
});


test("selected permanent Local Map retains its existing approximate signal cue without a mapped objective", () => {
  const value = snapshot();
  value.sceneId = "mh_signal_yard";
  value.capabilities.mapTopologyAuthorized = true;
  value.capabilities.exploredMap.objectives = [];
  value.hazards = [{ entityId: "mh_signal_interference_region", kind: "signal_interference_zone",
    active: true, transform: { positionM: point(16, 7.5), yawRad: 0 } }];
  assert.deepEqual(deriveMistHarborSonarMap(value).markers,
    [{ kind: "signal_region", position: { x: 16, z: 7.5 } }]);
});
