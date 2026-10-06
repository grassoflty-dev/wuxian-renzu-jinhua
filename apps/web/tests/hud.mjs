import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { deriveHudState, HudPresenter } from "../dist/ui/Hud.js";
import { projectWorldUi } from "../dist/renderer/WorldUiModel.js";

globalThis.document = { createElement: () => element(), baseURI: "https://example.test/" };
const testsDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(testsDir, "../../..");

function snapshot(overrides = {}) {
  return {
    kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "grey_hive", sceneId: "gh_entry_maintenance", checkpointId: null,
    worldEpoch: 8, serverTick: 10, authorityRevision: 10, ackSeq: 10,
    player: { entityId: "player", transform: { positionM: { xM: 1, yM: 0, zM: 1 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 80, maxHp: 100, currentEnergy: 25, maxEnergy: 50,
      facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [], doors: [], interactables: [], hazards: [],
    objectives: [{ objectiveId: "restore_power", state: "active", positionM: { xM: 4, yM: 0, zM: 1 } }],
    capabilities: { schemaVersion: 1, items: [] },
    progression: { schemaVersion: 1, currentWorldId: "grey_hive", eventSeq: 0, worlds: [] },
    ...overrides,
  };
}

function element() {
  return { textContent: "", hidden: true, style: {}, attributes: {}, children: [],
    setAttribute(name, value) { this.attributes[name] = value; },
    replaceChildren(...children) { this.children = children; },
    append(child) { this.children.push(child); } };
}
function presenter() {
  const elements = {
    portrait: element(), actionState: element(), skills: element(), objectivePanel: element(), interactionPrompt: element(),
    hpValue: element(), hpFill: element(), energyValue: element(), energyFill: element(),
    world: element(), objectives: element(), interaction: element(), doorStatus: element(),
    pumpStatus: element(), environmentStatus: element(),
    feedback: element(), pauseButton: element(), pauseOverlay: element(),
    pauseTitle: element(), pauseDetail: element(), resumeButton: element(),
  };
  return { elements, hud: new HudPresenter(elements) };
}

test("HUD follows v3 snapshot vitals, world, scene, objectives, and later authoritative changes", () => {
  const first = deriveHudState(snapshot());
  assert.equal(first.hpText, "80 / 100");
  assert.equal(first.hpRatio, 0.8);
  assert.equal(first.energyText, "25 / 50");
  assert.equal(first.worldLabel, "灰巢设施");
  assert.equal("sceneId" in first, false);
  assert.deepEqual(first.objectives, [{ id: "restore_power", state: "active" }]);

  const changed = deriveHudState(snapshot({
    worldId: "clockworks", sceneId: "cw_factory", worldEpoch: 9,
    player: { ...snapshot().player, currentHp: 0, currentEnergy: 50 },
    objectives: [{ objectiveId: "restore_power", state: "complete", positionM: { xM: 4, yM: 0, zM: 1 } }],
  }));
  assert.equal(changed.worldLabel, "钟骨工厂");
  assert.equal("sceneId" in changed, false);
  assert.equal(changed.hpRatio, 0);
  assert.equal(changed.energyRatio, 1);
  assert.deepEqual(changed.objectives, [{ id: "restore_power", state: "complete" }]);
});

test("interaction target respects the server's 2.5 m 3D boundary and active state", () => {
  const interactable = { entityId: "console", kind: "power_console", transform: { positionM: { xM: 3.5, yM: 0, zM: 1 }, yawRad: 0 }, active: true };
  assert.equal(deriveHudState(snapshot({ interactables: [interactable] })).interactionId, "console");
  assert.equal(deriveHudState(snapshot({ interactables: [{ ...interactable, transform: { positionM: { xM: 3.5001, yM: 0, zM: 1 }, yawRad: 0 } }] })).interactionId, null);
  assert.equal(deriveHudState(snapshot({ interactables: [{ ...interactable, transform: { positionM: { xM: 1, yM: 2.5001, zM: 1 }, yawRad: 0 } }] })).interactionId, null);
  assert.equal(deriveHudState(snapshot({ interactables: [{ ...interactable, active: false }] })).interactionId, null);
});

test("Grey Hive tutorial terminal prompt follows existing range, focus and authority, with no permanent marker", async () => {
  const scene = JSON.parse(await readFile(resolve(repoRoot, "content/scenes/compiled/gh_entry_maintenance.json"), "utf8"));
  const authored = scene.interactions.find(item => item.id === "gh_entry_tutorial_terminal");
  assert.equal(authored.kind, "terminal");
  assert.equal(authored.event, null);
  assert.equal(authored.assetId, undefined);
  assert.ok(!scene.presentation.sprites.some(item => item.id === authored.id));
  const terminal = { entityId: authored.id, kind: authored.kind, active: true,
    transform: { positionM: { xM: authored.position[0], yM: authored.position[1], zM: authored.position[2] }, yawRad: 0 } };
  const value = snapshot({ interactables: [terminal] });
  const position = value.player.transform.positionM;
  const camera = { width: 1280, height: 720, origin: terminal.transform.positionM };
  const { hud, elements } = presenter();
  const observe = (visible) => {
    const state = deriveHudState(value);
    assert.equal(state.interactionText, visible ? "[F] 查看教程终端" : "");
    assert.equal(state.interactionId, visible ? terminal.entityId : null);
    hud.apply(value);
    assert.equal(elements.interactionPrompt.hidden, !visible);
    assert.equal(elements.interactionPrompt.textContent, state.interactionText);
    // World-UI death lifecycle is separate; this text-only fix preserves its policy.
    if (value.player.currentHp > 0) {
      assert.deepEqual(projectWorldUi(value, camera).filter(item => item.kind === "interaction").map(item => item.id),
        visible ? [`interaction:${terminal.entityId}`] : []);
    }
  };
  Object.assign(position, { xM: 4, yM: 0, zM: 10 }); observe(false);
  position.zM = 9.5; observe(true); // The same inclusive 2.5 m boundary as every F target.
  position.zM = 9.5001; observe(false);
  Object.assign(position, { xM: 4, yM: 2.5001, zM: 7 }); observe(false);
  position.yM = 0; observe(true);
  terminal.active = false; observe(false);
  terminal.active = true; value.player.currentHp = 0; observe(false);
  value.player.currentHp = 80; observe(true);
  const closer = { entityId: "another_terminal", kind: "terminal", active: true,
    transform: { positionM: { xM: 4, yM: 0, zM: 8 }, yawRad: 0 } };
  position.zM = 8; value.interactables.push(closer);
  assert.equal(deriveHudState(value).interactionId, closer.entityId);
  assert.equal(deriveHudState(value).interactionText, "F 交互 · 终端");
  value.interactables.reverse(); assert.equal(deriveHudState(value).interactionId, closer.entityId);
  value.interactables = [terminal]; value.sceneId = "gh_power_room";
  assert.equal(deriveHudState(value).interactionText, "F 交互 · 终端");
  value.sceneId = "gh_entry_maintenance"; value.worldId = "return_station";
  assert.equal(deriveHudState(value).interactionText, "F 交互 · 终端");
  value.worldId = "grey_hive"; observe(true);
  hud.reset(); assert.equal(elements.interactionPrompt.hidden, true);
  assert.equal(elements.interactionPrompt.textContent, "");
});

test("compiled Clockworks pressure hall valves expose one nearby pressure-valve F target", async () => {
  const compiled = JSON.parse(await readFile(resolve(repoRoot,
    "content/scenes/compiled/cw_pressure_hall.json"), "utf8"));
  const expectedValveIds = [
    "cw_pressure_valve_01_staged",
    "cw_pressure_valve_02_staged",
    "cw_pressure_valve_03_staged",
  ];
  const authoredValves = compiled.interactions.filter(entry => entry.kind === "valve_control");
  assert.deepEqual(authoredValves.map(entry => entry.id).sort(), expectedValveIds);
  assert.deepEqual(
    compiled.interactionAggregates.find(entry => entry.event === "clockworks_valves")?.memberIds,
    expectedValveIds,
  );

  const toInteractable = entry => ({
    entityId: entry.id,
    kind: entry.kind,
    active: true,
    transform: { positionM: { xM: entry.position[0], yM: entry.position[1], zM: entry.position[2] }, yawRad: 0 },
  });
  const sceneSnapshot = interactables => snapshot({
    worldId: "clockworks", sceneId: "cw_pressure_hall", interactables,
  });
  const nearestAuthoredValve = toInteractable(authoredValves[0]);
  nearestAuthoredValve.transform.positionM = { xM: 3.5, yM: 0, zM: 1 };
  const inRange = deriveHudState(sceneSnapshot([nearestAuthoredValve]));
  assert.equal(inRange.interactionId, expectedValveIds[0]);
  assert.equal(inRange.interactionText, "F 交互 · 压力阀");

  assert.equal(deriveHudState(sceneSnapshot([{ ...nearestAuthoredValve, active: false }])).interactionId, null);
  assert.equal(deriveHudState(sceneSnapshot([{
    ...nearestAuthoredValve,
    transform: { positionM: { xM: 3.5001, yM: 0, zM: 1 }, yawRad: 0 },
  }])).interactionId, null);
  assert.equal(deriveHudState(sceneSnapshot([{
    ...nearestAuthoredValve,
    transform: { positionM: { xM: 1, yM: 2.5001, zM: 1 }, yawRad: 0 },
  }])).interactionId, null);
});

test("nearest pressure valve wins deterministically and staged markers never claim F", async () => {
  const compiled = JSON.parse(await readFile(resolve(repoRoot,
    "content/scenes/compiled/cw_pressure_hall.json"), "utf8"));
  const valveIds = compiled.interactions.filter(entry => entry.kind === "valve_control")
    .map(entry => entry.id).sort();
  const stagedMarkerKinds = ["three_valve_sequence_staged_marker", "pressure_ui_staged_marker"];
  assert.deepEqual(compiled.interactions.filter(entry => stagedMarkerKinds.includes(entry.kind))
    .map(entry => entry.kind).sort(), stagedMarkerKinds.slice().sort());
  const near = (entityId, kind, xM = 2, zM = 1) => ({
    entityId, kind, active: true,
    transform: { positionM: { xM, yM: 0, zM }, yawRad: 0 },
  });
  const sceneSnapshot = interactables => snapshot({
    worldId: "clockworks", sceneId: "cw_pressure_hall", interactables,
  });
  const tiedValves = [near(valveIds[1], "valve_control"), near(valveIds[0], "valve_control")];
  assert.equal(deriveHudState(sceneSnapshot(tiedValves)).interactionId, valveIds[0]);
  const closestValve = near(valveIds[2], "valve_control", 1.5, 1);
  const markers = stagedMarkerKinds.map((kind, index) => near(
    compiled.interactions.find(entry => entry.kind === kind).id, kind, 1.01 + index * 0.01, 1,
  ));
  assert.equal(deriveHudState(sceneSnapshot([...markers, tiedValves[0], closestValve])).interactionId, valveIds[2]);
  for (const marker of markers) {
    const markerOnly = deriveHudState(sceneSnapshot([marker]));
    assert.equal(markerOnly.interactionId, null);
    assert.equal(markerOnly.interactionText, "");
  }
});

test("extraction console has a clear label, while authored static markers are never F-key targets", () => {
  const near = kind => ({ entityId: kind, kind, transform: { positionM: { xM: 2, yM: 0, zM: 1 }, yawRad: 0 }, active: true });
  const extraction = deriveHudState(snapshot({ interactables: [near("extraction_console")] }));
  assert.equal(extraction.interactionId, "extraction_console");
  assert.equal(extraction.interactionText, "F 交互 · 撤离控制台");

  for (const kind of ["decon_valve_static_marker", "beacon_deploy_static_marker", "beacon_storage_mount_static_marker",
    "world_gate_marker", "signal_interference_hint_marker", "water_depth_slowdown_marker"]) {
    const markerOnly = deriveHudState(snapshot({ interactables: [near(kind)] }));
    assert.equal(markerOnly.interactionId, null, `${kind} must not be presented as an F action`);
    assert.equal(markerOnly.interactionText, "");
    const markerBesideAction = deriveHudState(snapshot({ interactables: [near(kind), near("extraction_console")] }));
    assert.equal(markerBesideAction.interactionId, "extraction_console", `${kind} must not mask a real action`);
  }
});

test("authored Mist Harbor beacons show a nearby F prompt only while Rust marks them active", () => {
  const beacon = { entityId: "mh_west_beacon", kind: "beacon", active: true,
    transform: { positionM: { xM: 2, yM: 0, zM: 1 }, yawRad: 0 } };
  const current = snapshot({ worldId: "mist_harbor", sceneId: "mh_tidal_warehouse", interactables: [beacon] });
  assert.equal(deriveHudState(current).interactionId, "mh_west_beacon");
  assert.equal(deriveHudState(current).interactionText, "F 交互 · 雾港航标");
  assert.equal(deriveHudState(snapshot({ ...current, interactables: [{ ...beacon, active: false }] })).interactionId, null);
  assert.equal(deriveHudState(snapshot({ ...current, interactables: [{ ...beacon,
    transform: { positionM: { xM: 3.6, yM: 0, zM: 1 }, yawRad: 0 } }] })).interactionId, null);
});

test("pump status renders from each authoritative frame and clears on scene change or reset", () => {
  const { elements, hud } = presenter();
  const current = snapshot({ worldId: "mist_harbor", sceneId: "mh_pump_station",
    mistHarborPump: { state: "draining" } });
  hud.apply(current);
  assert.equal(elements.pumpStatus.hidden, false);
  assert.equal(elements.pumpStatus.textContent, "排水进行中");
  assert.equal(elements.pumpStatus.attributes["data-pump-state"], "draining");
  hud.apply(snapshot({ ...current, worldEpoch: 12, mistHarborPump: { state: "drained" } }));
  assert.equal(elements.pumpStatus.textContent, "排水系统已完成");
  hud.apply(snapshot({ ...current, sceneId: "mh_drowned_quay" }));
  assert.equal(elements.pumpStatus.hidden, true);
  hud.reset();
  assert.equal(elements.pumpStatus.textContent, "");
  assert.equal(elements.pumpStatus.attributes["data-pump-state"], "");
});

test("only the native world-gate affordance is exposed as F, with return gated to the extraction route", () => {
  const gate = { entityId: "rs_world_gate_marker", kind: "world_gate", active: true,
    transform: { positionM: { xM: 20, yM: 0, zM: 8 }, yawRad: 0 } };
  const atGate = snapshot({
    worldId: "return_station", sceneId: "rs_core_room",
    player: { ...snapshot().player, transform: { positionM: { xM: 18, yM: 0, zM: 8 }, yawRad: 0 } },
    interactables: [gate],
  });
  assert.equal(deriveHudState(atGate).interactionId, "rs_world_gate_marker");
  assert.equal(deriveHudState(atGate).interactionText, "F 交互 · 前往灰巢设施");

  const returnGate = deriveHudState(snapshot({ interactables: [{ ...gate,
    entityId: "gh_extraction_return_to_rs",
    transform: { positionM: { xM: 2, yM: 0, zM: 1 }, yawRad: 0 },
  }] }));
  assert.equal(returnGate.interactionText, "F 交互 · 撤离后返回归航站");

  const extractionConsole = { entityId: "gh_exit_extraction_console", kind: "extraction_console", active: false,
    transform: { positionM: { xM: 2, yM: 0, zM: 1 }, yawRad: 0 } };
  const noCompetingF = deriveHudState(snapshot({ interactables: [extractionConsole, {
    ...gate, entityId: "gh_extraction_return_to_rs",
    transform: extractionConsole.transform,
  }] }));
  assert.equal(noCompetingF.interactionId, "gh_extraction_return_to_rs");
});

test("Mist Harbor gate labels stay distinct and staged exits do not become F actions", () => {
  const gate = { entityId: "rs_mh_world_gate_marker", kind: "world_gate", active: true,
    transform: { positionM: { xM: 20, yM: 0, zM: 12 }, yawRad: 0 } };
  const near = snapshot({ worldId: "return_station", sceneId: "rs_core_room",
    player: { ...snapshot().player, transform: { positionM: { xM: 20, yM: 0, zM: 12 }, yawRad: 0 } },
    interactables: [gate] });
  assert.equal(deriveHudState(near).interactionId, gate.entityId);
  assert.equal(deriveHudState(near).interactionText, "F 交互 · 前往雾港余烬");

  const exit = { ...gate, entityId: "mh_extraction_return_to_rs",
    transform: { positionM: { xM: 20, yM: 0, zM: 8 }, yawRad: 0 } };
  assert.equal(deriveHudState(snapshot({ worldId: "mist_harbor", sceneId: "mh_extraction",
    player: { ...snapshot().player, transform: { positionM: exit.transform.positionM, yawRad: 0 } },
    interactables: [exit] })).interactionText,
  "F 交互 · 撤离后返回归航站");
  for (const kind of ["world_exit", "world_completion_staged_marker", "return_station_revisit_staged_marker"]) {
    assert.equal(deriveHudState(snapshot({ worldId: "mist_harbor", sceneId: "mh_extraction",
      interactables: [{ ...exit, kind }] })).interactionId, null);
  }
});

test("the native Return Station save/rest affordance is an F target only while active and nearby", () => {
  const terminal = { entityId: "rs_save_rest_terminal_marker", kind: "save_rest_terminal", active: true,
    transform: { positionM: { xM: 8, yM: 0, zM: 12 }, yawRad: 0 } };
  const near = snapshot({ worldId: "return_station", sceneId: "rs_core_room",
    player: { ...snapshot().player, transform: { positionM: { xM: 8, yM: 0, zM: 12 }, yawRad: 0 } },
    interactables: [terminal] });
  assert.equal(deriveHudState(near).interactionId, terminal.entityId);
  assert.equal(deriveHudState(near).interactionText, "F 交互 · 休整并保存");
  assert.equal(deriveHudState(snapshot({ ...near, interactables: [{ ...terminal, active: false }] })).interactionId, null);
});

test("the native mission marker is a distinct F target, not a generic marker action", () => {
  const terminal = { entityId: "rs_mission_terminal_marker", kind: "mission_terminal", active: true,
    transform: { positionM: { xM: 8, yM: 0, zM: 4 }, yawRad: 0 } };
  const current = snapshot({
    worldId: "return_station", sceneId: "rs_core_room",
    player: { ...snapshot().player, transform: { positionM: { xM: 8, yM: 0, zM: 4 }, yawRad: 0 } },
    interactables: [terminal],
  });
  assert.equal(deriveHudState(current).interactionId, terminal.entityId);
  assert.equal(deriveHudState(current).interactionText, "F 交互 · 任务状态");
  assert.equal(deriveHudState(snapshot({ interactables: [{ ...terminal, active: false }] })).interactionId, null);
});

test("the native capability marker is a distinct F target only while active and nearby", () => {
  const terminal = { entityId: "rs_capability_terminal_marker", kind: "capability_terminal", active: true,
    transform: { positionM: { xM: 15, yM: 0, zM: 4 }, yawRad: 0 } };
  const current = snapshot({
    worldId: "return_station", sceneId: "rs_core_room",
    player: { ...snapshot().player, transform: { positionM: { xM: 15, yM: 0, zM: 4 }, yawRad: 0 } },
    interactables: [terminal],
  });
  assert.equal(deriveHudState(current).interactionId, terminal.entityId);
  assert.equal(deriveHudState(current).interactionText, "F 交互 · 能力状态");
  assert.equal(deriveHudState(snapshot({ interactables: [{ ...terminal, active: false }] })).interactionId, null);
});

test("door feedback uses authoritative open and locked flags only when the player is nearby", () => {
  const door = { doorId: "gate", transform: { positionM: { xM: 3, yM: 0, zM: 1 }, yawRad: 0 }, open: false, locked: true };
  const locked = deriveHudState(snapshot({ doors: [door] }));
  assert.equal(locked.nearbyDoorText, "门锁闭");
  assert.equal(locked.interactionText, "");
  assert.equal(deriveHudState(snapshot({ doors: [{ ...door, open: true, locked: false }] })).nearbyDoorText, "门已开启");
  assert.equal(deriveHudState(snapshot({ doors: [{ ...door, transform: { positionM: { xM: 3.5001, yM: 0, zM: 1 }, yawRad: 0 } }] })).nearbyDoorText, "");
});

test("empty objectives, absent targets, invalid maxima, and out-of-range meters remain safe", () => {
  const state = deriveHudState(snapshot({
    objectives: [],
    player: { ...snapshot().player, currentHp: 25, maxHp: 0, currentEnergy: 80, maxEnergy: 50 },
  }));
  assert.equal(state.hpText, "— / —");
  assert.equal(state.hpRatio, 0);
  assert.equal(state.energyText, "50 / 50");
  assert.equal(state.energyRatio, 1);
  assert.deepEqual(state.objectives, []);
  assert.equal(state.interactionId, null);
  assert.equal(state.interactionText, "");
});

test("HUD presenter refreshes authoritative state, exposes pause state, and clears at scene exit", () => {
  const { hud, elements } = presenter();
  hud.apply(snapshot({ interactables: [{ entityId: "console", kind: "power_console", transform: { positionM: { xM: 3, yM: 0, zM: 1 }, yawRad: 0 }, active: true }] }));
  assert.equal(elements.hpValue.textContent, "80 / 100");
  assert.equal(elements.interaction.textContent, "F 交互 · 电力控制台");
  hud.setPauseStatus("pausing");
  assert.equal(elements.pauseOverlay.hidden, false);
  assert.equal(elements.pauseTitle.textContent, "正在请求暂停…");
  assert.notEqual(elements.pauseTitle.textContent, "旅程已暂停");
  hud.setPauseStatus("paused");
  assert.equal(elements.pauseTitle.textContent, "旅程已暂停");
  hud.setPauseStatus("error", "ipc pause disconnected");
  assert.equal(elements.pauseTitle.textContent, "暂停状态未确认");
  assert.match(elements.pauseDetail.textContent, /ipc pause disconnected/);
  assert.equal(elements.pauseButton.textContent, "重试确认");
  hud.setFeedback("交互已完成");
  hud.reset();
  assert.equal(elements.hpValue.textContent, "— / —");
  assert.equal(elements.world.textContent, "—");
  assert.equal(elements.objectivePanel.hidden, true);
  assert.equal(elements.feedback.textContent, "");
  assert.equal(elements.pauseOverlay.hidden, true);
  assert.equal(elements.pauseButton.textContent, "暂停");
});


test("environment controls and non-color HUD status follow projected authority and clear on scene exit",()=>{
 const hazard={entityId:"heat",kind:"heat_zone",active:false,phaseActive:true,polygonM:[[1,1],[3,1],[3,3],[1,3]],
  environment:{tag:"heat",phase:"warning",remainingMs:900,exposureBps:4500}};
 const interaction={entityId:"coolant",kind:"environment_control",active:true,transform:{positionM:{xM:1,yM:0,zM:1},yawRad:0}};
 const input=snapshot({hazards:[hazard],interactables:[interaction]});
 const {elements,hud}=presenter();const state=hud.apply(input);
 assert.equal(state.interactionId,"coolant");assert.match(state.interactionText,/环境调节/);
 assert.match(elements.environmentStatus.textContent,/热区.*预警.*热量 45%/);assert.equal(elements.environmentStatus.hidden,false);
 assert.doesNotMatch(elements.environmentStatus.textContent,/0\.9秒/);
 hud.apply(snapshot());assert.equal(elements.environmentStatus.hidden,true);
 hud.apply(input);hud.reset();assert.equal(elements.environmentStatus.textContent,"");assert.equal(elements.environmentStatus.hidden,true);
});


test("ordinary HUD omits scene IDs and idle while retaining localized active and unknown status", () => {
  const { elements, hud } = presenter();
  // Any HTML write to a HUD element must fail, even if a future field is malformed.
  for (const value of Object.values(elements)) {
    Object.defineProperty(value, "innerHTML", { set() { throw new Error("HUD must use textContent"); } });
  }
  for (const [worldId, label] of [["return_station", "归航站"], ["grey_hive", "灰巢设施"],
    ["mist_harbor", "雾港余烬"], ["clockworks", "钟骨工厂"]]) {
    const state = hud.apply(snapshot({ worldId, sceneId: "rs_core_room" }));
    assert.equal(elements.world.textContent, label);
    assert.equal(elements.actionState.textContent, "");
    assert.equal(elements.actionState.hidden, true);
    assert.equal("sceneId" in state, false);
  }
  const labels = { primaryAttack: "普通攻击", dash: "短距冲刺", pulse: "环形扫描",
    guard: "弧盾防御", pierce: "线性穿透", walk: "行走", run: "奔跑", hit: "受击",
    death: "已倒下", interact: "交互中", contextTraversal: "通行中" };
  for (const [actionState, label] of Object.entries(labels)) {
    hud.apply(snapshot({ player: { ...snapshot().player, actionState } }));
    assert.equal(elements.actionState.textContent, label);
    assert.equal(elements.actionState.hidden, false);
  }
  for (const untrusted of ["future_internal_state", "", "<img src=x onerror=alert(1)>", "constructor", "__proto__"]) {
    const state = hud.apply(snapshot({ worldId: untrusted, sceneId: untrusted,
      player: { ...snapshot().player, actionState: untrusted } }));
    assert.equal(elements.world.textContent, "未知世界");
    assert.equal(elements.actionState.textContent, "状态未知");
    assert.equal(elements.actionState.hidden, false);
    assert.equal("sceneId" in state, false);
  }
  hud.apply(snapshot({ player: { ...snapshot().player, currentHp: 0, actionState: "idle" } }));
  assert.equal(elements.actionState.textContent, "已倒下");
  assert.equal(elements.actionState.hidden, false);
  hud.reset();
  assert.equal(elements.actionState.textContent, "");
  assert.equal(elements.actionState.hidden, true);
  hud.apply(snapshot({ worldId: "return_station", worldEpoch: 10, sceneId: "rs_core_room",
    player: { ...snapshot().player, currentHp: 65, currentEnergy: 10 } }));
  assert.equal(elements.world.textContent, "归航站");
  assert.equal(elements.actionState.hidden, true);
  assert.equal(elements.hpValue.textContent, "65 / 100");
  assert.equal(elements.hpFill.attributes["aria-valuenow"], "65");
  assert.equal(elements.energyValue.textContent, "10 / 50");
  assert.equal(elements.energyFill.attributes["aria-valuenow"], "20");
  hud.apply(snapshot({ player: { ...snapshot().player, actionState: "guard" } }));
  assert.equal(elements.actionState.hidden, false);
  hud.apply(snapshot({ player: { ...snapshot().player, velocityMps: { xM: 2, yM: 0, zM: 0 } } }));
  assert.equal(elements.actionState.hidden, true, "moving idle must not retain an old guard label");
});