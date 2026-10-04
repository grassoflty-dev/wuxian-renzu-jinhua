import test from "node:test";
import assert from "node:assert/strict";
import { deriveGreyHiveStatus } from "../dist/ui/components/GreyHiveStatusPanel.js";

const position = (xM, zM) => ({ xM, yM: 0, zM });
const exploredMap = {
  worldId: "grey_hive",
  playerPositionM: position(2, 2),
  rooms: [{ roomId: "power_room", outlineM: [position(0, 0), position(4, 0), position(4, 4), position(0, 4)] }],
  connections: [],
  objectives: [{ objectiveId: "gh_restore_main_power", positionM: position(3, 3) }],
};

function snapshot(overrides = {}) {
  return {
    worldId: "grey_hive",
    objectives: [
      { objectiveId: "gh_restore_main_power", state: "active" },
      { objectiveId: "gh_lockdown_objective", state: "complete" },
    ],
    capabilities: { items: [], exploredMap },
    ...overrides,
  };
}

test("Grey Hive cards use only current-scene authoritative objectives", () => {
  const view = deriveGreyHiveStatus(snapshot());
  assert.match(view.power, /待完成/);
  assert.match(view.containment, /已完成/);
  const absent = deriveGreyHiveStatus(snapshot({ objectives: [], progression: { worlds: [{ completedEvents: ["hive_power"] }] } }));
  assert.match(absent.power, /未投影/);
  assert.match(absent.containment, /未投影/);
  assert.equal(deriveGreyHiveStatus(snapshot({ worldId: "mist_harbor" })), null);
});

test("global Grey Hive status comes from matching Rust completed events", () => {
  const progression = completedEvents => ({ currentWorldId: "grey_hive", worlds: [
    { worldId: "grey_hive", completedEvents },
  ] });
  const powered = deriveGreyHiveStatus(snapshot({ objectives: [], progression: progression(["hive_power"]) }));
  assert.match(powered.power, /已恢复/);
  assert.match(powered.containment, /待解除/);
  const cleared = deriveGreyHiveStatus(snapshot({ objectives: [], progression: progression(["hive_power", "hive_lockdown"]) }));
  assert.match(cleared.containment, /已解除/);
  const inconsistent = deriveGreyHiveStatus(snapshot({
    objectives: [{ objectiveId: "gh_restore_main_power", state: "active" }],
    progression: { currentWorldId: "mist_harbor", worlds: [{ worldId: "grey_hive", completedEvents: ["hive_power"] }] },
  }));
  assert.match(inconsistent.power, /待完成/);
  const malformed = deriveGreyHiveStatus(snapshot({ progression: progression(["hive_power", 9]) }));
  assert.match(malformed.power, /待完成/);
});

test("facility map requires selected grant and a same-world valid exploration projection", () => {
  assert.equal(deriveGreyHiveStatus(snapshot()).map, null);
  const grant = { capabilityId: "information.local_map_i", granted: true, selected: true };
  assert.equal(deriveGreyHiveStatus(snapshot({ capabilities: { items: [{ ...grant, selected: false }], exploredMap } })).map, null);
  assert.equal(deriveGreyHiveStatus(snapshot({ capabilities: { items: [grant], exploredMap: { ...exploredMap, worldId: "mist_harbor" } } })).map, null);
  const authorized = deriveGreyHiveStatus(snapshot({ capabilities: { items: [grant], exploredMap } }));
  assert.match(authorized.mapStatus, /已授权/);
  assert.deepEqual(authorized.map.rooms.map(room => room.id), ["power_room"]);
  assert.deepEqual(authorized.map.player, { x: 2, z: 2 });
  const badMap = { ...exploredMap, rooms: [{ roomId: "bad", outlineM: [position(0, 0), position(Infinity, 1), position(1, 1)] }] };
  const invalid = deriveGreyHiveStatus(snapshot({ capabilities: { items: [grant], exploredMap: badMap } }));
  assert.equal(invalid.map, null);
  assert.match(invalid.mapStatus, /无有效探索投影/);
});
