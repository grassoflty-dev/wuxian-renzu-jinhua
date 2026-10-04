import test from "node:test";
import assert from "node:assert/strict";
import { deriveMistHarborStatus } from "../dist/ui/components/MistHarborStatusPanel.js";
import { deriveCorePanel } from "../dist/ui/CorePanelModel.js";

function snapshot(overrides = {}) {
  return {
    worldId: "mist_harbor", sceneId: "mh_tidal_warehouse",
    progression: { currentWorldId: "mist_harbor", worlds: [
      { worldId: "mist_harbor", completedEvents: [] },
      { worldId: "grey_hive", completed: true, firstCompletion: true, completedEvents: ["hive_power"] },
    ] },
    objectives: [{ objectiveId: "current_scene_objective", state: "active" }],
    ...overrides,
  };
}

function hazard(entityId, kind, active) {
  return { entityId, kind, active,
    transform: { positionM: { xM: 16, yM: 0, zM: 8 }, yawRad: 0 } };
}

const signalHazard = active => hazard("mh_signal_interference_region", "signal_interference_zone", active);
const waterHazard = active => hazard("mh_water_depth_region", "water_depth_slowdown", active);

test("Mist Harbor milestones use exact same-world Rust events", () => {
  const initial = deriveMistHarborStatus(snapshot());
  assert.match(initial.westBeacon, /待完成/);
  assert.match(initial.eastBeacon, /待完成/);
  assert.match(initial.signal, /待完成/);

  const partial = deriveMistHarborStatus(snapshot({ progression: {
    currentWorldId: "mist_harbor", worlds: [
      { worldId: "grey_hive", completedEvents: ["mist_beacon_east", "mist_signal"] },
      { worldId: "mist_harbor", completedEvents: ["mist_beacon_west", "mist_beacon_west_extra"] },
    ],
  } }));
  assert.match(partial.westBeacon, /已完成/);
  assert.match(partial.eastBeacon, /待完成/);
  assert.match(partial.signal, /待完成/);

  const complete = deriveMistHarborStatus(snapshot({ progression: {
    currentWorldId: "mist_harbor", worlds: [
      { worldId: "mist_harbor", completedEvents: ["mist_beacon_west", "mist_beacon_east", "mist_signal"] },
    ],
  } }));
  assert.deepEqual([complete.westBeacon, complete.eastBeacon, complete.signal].map(value => /已完成/.test(value)),
    [true, true, true]);
  assert.ok(![complete.westBeacon, complete.eastBeacon, complete.signal].some(value => /声呐|导航|已解锁/.test(value)));
});

test("missing, malformed, ambiguous, and wrong-world progress stays unconfirmed", () => {
  const invalid = [
    undefined,
    { currentWorldId: "mist_harbor", worlds: [] },
    { currentWorldId: "grey_hive", worlds: [{ worldId: "mist_harbor", completedEvents: ["mist_signal"] }] },
    { currentWorldId: "mist_harbor", worlds: [{ worldId: "mist_harbor" }] },
    { currentWorldId: "mist_harbor", worlds: [{ worldId: "mist_harbor", completedEvents: ["mist_signal", 1] }] },
    { currentWorldId: "mist_harbor", worlds: [
      { worldId: "mist_harbor", completedEvents: ["mist_signal"] },
      { worldId: "mist_harbor", completedEvents: [] },
    ] },
  ];
  for (const progression of invalid) {
    const status = deriveMistHarborStatus(snapshot({ progression }));
    assert.ok([status.westBeacon, status.eastBeacon, status.signal].every(value => /未确认/.test(value)));
  }
  assert.equal(deriveMistHarborStatus(snapshot({ worldId: "return_station" })), null);
  assert.equal(deriveMistHarborStatus(snapshot({ worldId: "grey_hive" })), null);
});

test("Mission keeps Grey Hive and current-scene rows, adding Mist Harbor only in its world", () => {
  const mission = deriveCorePanel("mission", snapshot());
  assert.match(mission.subtitle, /雾港/);
  assert.deepEqual(mission.rows.map(row => row.label), [
    "当前世界", "灰巢首撤", "雾港西侧航标", "雾港东侧航标", "雾港信号",
    "当前信号干扰", "当前水域减速", "current_scene_objective",
  ]);
  assert.equal(mission.rows.at(-1).value, "active");
  assert.equal(mission.rows[1].value, "已完成");

  const markerOnly = snapshot({ progression: undefined, objectives: [
    { objectiveId: "mist_signal", state: "complete" },
  ] });
  assert.match(deriveCorePanel("mission", markerOnly).rows[4].value, /未确认/);
  assert.equal(deriveCorePanel("mission", markerOnly).rows.at(-1).value, "complete");

  const station = deriveCorePanel("mission", snapshot({ worldId: "return_station" }));
  assert.match(station.subtitle, /灰巢/);
  assert.ok(station.rows.every(row => !row.label.startsWith("雾港")));
  assert.ok(station.rows.every(row => !row.label.startsWith("当前信号") && !row.label.startsWith("当前水域")));
});

test("Signal Yard reports only the exact active Rust hazard as occupancy", () => {
  for (const active of [false, true]) {
    const value = deriveMistHarborStatus(snapshot({ sceneId: "mh_signal_yard", hazards: [signalHazard(active)] }));
    assert.match(value.signalInterference, active ? /区内/ : /区外/);
    assert.match(value.waterSlowdown, /不适用/);
    assert.doesNotMatch(value.signalInterference, /强度|百分比|水位|水深|通行/);
    const rows = deriveCorePanel("mission", snapshot({ sceneId: "mh_signal_yard", hazards: [signalHazard(active)] })).rows;
    assert.equal(rows.find(row => row.label === "当前信号干扰").value, value.signalInterference);
  }
});

test("Drowned Quay reports only the exact active Rust water slowdown hazard", () => {
  for (const active of [false, true]) {
    const value = deriveMistHarborStatus(snapshot({ sceneId: "mh_drowned_quay", hazards: [waterHazard(active)] }));
    assert.match(value.waterSlowdown, active ? /区内/ : /区外/);
    assert.match(value.signalInterference, /不适用/);
    assert.doesNotMatch(value.waterSlowdown, /强度|百分比|水位|水深|通行/);
    const rows = deriveCorePanel("mission", snapshot({ sceneId: "mh_drowned_quay", hazards: [waterHazard(active)] })).rows;
    assert.equal(rows.find(row => row.label === "当前水域减速").value, value.waterSlowdown);
  }
});

test("missing, duplicate, wrong-kind, malformed and wrong-id hazards stay unreported", () => {
  for (const [sceneId, matching, field] of [
    ["mh_signal_yard", signalHazard, "signalInterference"],
    ["mh_drowned_quay", waterHazard, "waterSlowdown"],
  ]) {
    const invalid = [
      undefined, [], [matching(true), matching(false)],
      [{ ...matching(true), entityId: "mh_cy_signal_01_static_marker" }],
      [{ ...matching(true), kind: "dialogue_content_marker" }],
      [{ ...matching(true), active: "true" }],
      [{ ...matching(true), transform: undefined }],
      [{ ...matching(true), transform: { positionM: { xM: NaN, yM: 0, zM: 8 }, yawRad: 0 } }],
      [{ ...matching(true), transform: { positionM: { xM: 16, yM: 0, zM: 8 }, yawRad: NaN } }],
    ];
    for (const hazards of invalid) {
      const status = deriveMistHarborStatus(snapshot({ sceneId, hazards }));
      assert.match(status[field], /未报告/);
    }
  }
});

test("wrong scene/world and non-hazard data cannot create an environment reading", () => {
  const unrelated = snapshot({ sceneId: "mh_tidal_warehouse", hazards: [signalHazard(true), waterHazard(true)],
    objectives: [{ objectiveId: "mist_signal", state: "complete" }],
    player: { velocityMps: { xM: 0, yM: 0, zM: 0 } },
    interactables: [{ entityId: "mh_cy_signal_01_static_marker", kind: "dialogue_content_marker" }],
  });
  const status = deriveMistHarborStatus(unrelated);
  assert.match(status.signalInterference, /不适用/);
  assert.match(status.waterSlowdown, /不适用/);
  assert.equal(deriveMistHarborStatus({ ...unrelated, worldId: "grey_hive", sceneId: "mh_signal_yard" }), null);
  const yard = deriveMistHarborStatus(snapshot({ sceneId: "mh_signal_yard", hazards: [],
    progression: { currentWorldId: "mist_harbor", worlds: [
      { worldId: "mist_harbor", completedEvents: ["mist_signal"] },
    ] },
  }));
  assert.match(yard.signal, /已完成/);
  assert.match(yard.signalInterference, /未报告/);
});
