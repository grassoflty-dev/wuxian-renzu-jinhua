import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { deriveCorePanel } from "../dist/ui/CorePanelModel.js";
import { missionTerminalSummary } from "../dist/game/MissionTerminal.js";

// The presentation must follow the existing Rust policy, never change it.
const catalog = JSON.parse(readFileSync(new URL("../../../server-rs/data/world_progression_v1.json", import.meta.url), "utf8"));
const limit = catalog.worlds.find(world => world.worldId === "mist_harbor").revisitPolicy.maxRevisits;
const events = ["mist_beacon_west", "mist_beacon_east", "mist_signal"];

function snapshot({ worldId = "return_station", count = 0, gateActive, completed = true, greyHiveCleared = true } = {}) {
  return {
    kind: "full", protocolVersion: 3, worldId,
    sceneId: { return_station: "rs_core_room", clockworks: "cw_entry_foundry", grey_hive: "gh_exit", mist_harbor: "mh_extraction" }[worldId],
    interactables: gateActive === undefined ? [] : [
      { entityId: "rs_mh_world_gate_marker", kind: "world_gate", active: gateActive },
    ],
    progression: { worlds: [
      { worldId: "grey_hive", completed: greyHiveCleared, firstCompletion: greyHiveCleared },
      { worldId: "mist_harbor", completed, firstCompletion: completed,
        visitId: count + 1, cycleId: 1, revisitCount: count, completedEvents: completed ? [...events] : events.slice(0, 1) },
      { worldId: "clockworks", completed: false, firstCompletion: false, revisitCount: 0, completedEvents: [] },
    ] },
  };
}

function labels(view) {
  const network = deriveCorePanel("world_network", view).rows.find(row => row.label === "雾港余烬").value;
  const terminal = missionTerminalSummary(view).match(/雾港余烬：([^。]+)。/)[1];
  return [network, terminal];
}

function assertLabels(view, expected) {
  const before = structuredClone(view);
  assert.deepEqual(labels(view), [expected, expected]);
  assert.deepEqual(view, before, "labels do not mutate authoritative progress or gates");
}

test("MH first clear and last available revisit stay completed without a scene-local gate", () => {
  for (const count of [0, limit - 1]) {
    for (const worldId of ["return_station", "clockworks", "grey_hive"]) {
      assertLabels(snapshot({ count, worldId }), "已完成 · 前往归航站查询复访");
    }
  }
});

test("MH active and paused gates describe current availability, never inferred exhaustion", () => {
  for (const count of [0, limit - 1]) {
    assertLabels(snapshot({ count, gateActive: true }), "已完成 · 限次复访入口可用");
    assertLabels(snapshot({ count, gateActive: false }), "已完成 · 入口当前不可用");
  }
});

test("MH exhausted status requires the authoritative count to reach the existing limit", () => {
  for (const worldId of ["return_station", "clockworks", "grey_hive"]) {
    for (const gateActive of [undefined, false, true]) {
      assertLabels(snapshot({ worldId, count: limit, gateActive }), "已完成 · 复访次数用尽");
    }
  }
});

test("missing or malformed MH counts cannot claim exhaustion or an available revisit", () => {
  for (const count of [undefined, null, "0", "2", false, -1, 0.5, limit + 1, Number.NaN, Infinity, Number.MAX_SAFE_INTEGER + 1]) {
    for (const gateActive of [undefined, false, true]) {
      const view = snapshot({ gateActive });
      if (count === undefined) delete view.progression.worlds[1].revisitCount;
      else view.progression.worlds[1].revisitCount = count;
      assertLabels(view, "已完成 · 复访状态未报告");
    }
  }
});

test("MH current-world objective progress remains current even on the final revisit", () => {
  for (const completed of [false, true]) {
    for (const count of [0, limit - 1, limit]) {
      assert.deepEqual(labels(snapshot({ worldId: "mist_harbor", count, completed })), [
        "当前世界", `当前世界 · 主线目标 ${completed ? 3 : 1}/3`,
      ]);
    }
  }
});

test("MH locked and first-entry states do not depend on a not-yet-used revisit counter", () => {
  for (const count of [0, undefined, null]) {
    for (const [options, expected] of [
      [{ greyHiveCleared: false }, "灰巢首次撤离后解锁"],
      [{ greyHiveCleared: true }, "已解锁 · 前往归航站进入"],
      [{ gateActive: true }, "归航站入口可用"],
    ]) {
      const view = snapshot({ completed: false, ...options });
      view.progression.worlds[1].revisitCount = count;
      assertLabels(view, expected);
    }
  }
});
