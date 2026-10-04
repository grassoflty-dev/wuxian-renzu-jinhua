import test from "node:test";
import assert from "node:assert/strict";
import { missionTerminalSummary } from "../dist/game/MissionTerminal.js";

function snapshot({ completed = false, firstCompletion = false, gateActive = true } = {}) {
  return {
    protocolVersion: 3,
    interactables: [{ entityId: "rs_world_gate_marker", kind: "world_gate", active: gateActive }],
    progression: { worlds: [
      { worldId: "grey_hive", completed, firstCompletion },
      { worldId: "mist_harbor", completed: false, firstCompletion: false },
      { worldId: "clockworks", completed: false, firstCompletion: false },
    ] },
  };
}

test("mission terminal presents the Rust route state and does not invent MH/CW access", () => {
  assert.equal(missionTerminalSummary(snapshot()),
    "灰巢设施：入口可用；首次撤离：尚未完成。雾港余烬：灰巢首次撤离后解锁。钟骨工厂：状态未报告。");
  assert.equal(missionTerminalSummary(snapshot({ completed: true, firstCompletion: true, gateActive: false })),
    "灰巢设施：入口当前不可用；首次撤离：已完成。雾港余烬：已解锁 · 前往归航站进入。钟骨工厂：状态未报告。");
});

test("missing server projections are described as unreported rather than guessed", () => {
  assert.equal(missionTerminalSummary({ protocolVersion: 3, interactables: [], progression: { worlds: [] } }),
    "灰巢设施：入口状态未报告；首次撤离：状态未报告。雾港余烬：灰巢首次撤离后解锁。钟骨工厂：状态未报告。");
});
