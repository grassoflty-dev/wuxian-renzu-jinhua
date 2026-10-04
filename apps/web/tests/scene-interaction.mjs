import test from "node:test";
import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { dispatchInteractable, interactionErrorText, isCurrentSceneInteractionResult } from "../dist/game/SceneInteraction.js";
import { deriveHudState } from "../dist/ui/Hud.js";

const testsDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(testsDir, "../../..");
const compiledScenesDir = resolve(repoRoot, "content/scenes/compiled");
const RELEASE_SCENE_INTERACTION_KINDS = {
  gh_entry_maintenance: ["terminal"],
  gh_power_room: ["power_console"],
  gh_gate_a: ["door_panel"],
  gh_central_shaft: ["facility_log"],
  gh_lockdown: ["lockdown_terminal"],
  gh_bio_isolation: ["bio_log_terminal"],
  gh_gate_b: ["gate_b_panel"],
  gh_exit: ["extraction_console"],
  gh_deep_decon: ["environment_control"],
  gh_beacon: ["beacon_collect", "beacon_mount"],
  gh_sentinel_arena: ["facility_log"],
};
const ADDITIONAL_SCENE_INTERACTION_KINDS = {
  rs_core_room: ["world_gate_marker", "grey_hive_entry_marker", "world_gate_marker", "world_gate_marker", "mission_terminal_marker",
    "capability_terminal_marker", "save_rest_terminal_marker", "storage_inventory_marker"],
  mh_fog_pier: ["world_entry_beacon_marker", "entry_context_marker", "foghorn_direction_marker"],
  mh_tidal_warehouse: ["beacon", "beacon_caretaker_log_marker"],
  mh_signal_yard: ["signal_interference_hint_marker", "dialogue_content_marker"],
  mh_drowned_quay: ["water_depth_slowdown_marker"],
  mh_breakwater: ["beacon"],
  mh_pump_station: ["pump_control"],
  mh_resonance_tower: ["terminal", "dialogue_content_marker", "capability_staged_marker"],
  mh_warden_arena: [],
  mh_extraction: ["world_exit", "world_completion_staged_marker",
    "return_station_revisit_staged_marker"],
  cw_entry_foundry: ["entry_context_static_marker", "factory_shift_staged_marker"],
  cw_pressure_hall: ["valve_control", "three_valve_sequence_staged_marker",
    "pressure_ui_staged_marker"],
  cw_conveyor_bridge: ["moving_surface_staged_marker"],
  cw_boiler_chamber: ["environment_control"],
  cw_gear_shaft: [],
  cw_furnace_heart: ["environment_control", "static_dialogue_marker"],
  cw_forged_guard_arena: ["arena_shutter_staged_marker"],
  cw_regulator_core: ["terminal", "boss_phase_sequence_staged_marker",
    "coolant_valve"],
  cw_shutdown_exit: ["terminal", "air_step_grant_staged_marker",
    "exit_portal_staged_marker", "world_completion_staged_marker",
    "optional_enemy_wave_staged_marker", "static_epilogue_dialogue_marker"],
};
const RELEASE_ORDINARY_KINDS = new Set([
  "terminal", "power_console", "door_panel", "facility_log", "lockdown_terminal",
  "bio_log_terminal", "gate_b_panel", "extraction_console", "beacon", "pump_control", "valve_control",
  "coolant_valve",
]);
const STATIC_MARKER_KINDS = new Set([
  "decon_valve_static_marker", "beacon_deploy_static_marker", "beacon_storage_mount_static_marker",
  "world_exit",
]);

function item(kind, overrides = {}) {
  return { entityId: "route-a", kind, active: true,
    transform: { positionM: { xM: 0, yM: 0, zM: 0 }, yawRad: 0 }, ...overrides };
}
function result(worldEpoch = 4, overrides = {}) {
  return { applied: true, errorCode: null,
    snapshot: { kind: "full", protocolVersion: 3, worldId: "grey_hive", sceneId: "grey_hive", worldEpoch, ...overrides } };
}

test("scene routes, trigger, and world gate dispatch to matching authoritative commands", async () => {
  const calls = [];
  const client = Object.fromEntries(["sceneTransition", "sceneCheckpoint", "sceneTrigger", "worldGate"].map(method =>
    [method, async (id, worldEpoch) => { calls.push({ method, id, worldEpoch }); return result(worldEpoch + (method === "sceneTransition" ? 1 : 0)); }])) ;
  await dispatchInteractable(client, item("scene_transition"), 4);
  await dispatchInteractable(client, item("scene_checkpoint", { entityId: "checkpoint-a" }), 7);
  await dispatchInteractable(client, item("scene_trigger", { entityId: "trigger-a" }), 9);
  await dispatchInteractable(client, item("world_gate", { entityId: "rs_world_gate_to_gh" }), 11);
  assert.deepEqual(calls, [
    { method: "sceneTransition", id: "route-a", worldEpoch: 4 },
    { method: "sceneCheckpoint", id: "checkpoint-a", worldEpoch: 7 },
    { method: "sceneTrigger", id: "trigger-a", worldEpoch: 9 },
    { method: "worldGate", id: "rs_world_gate_to_gh", worldEpoch: 11 },
  ]);
});

test("the live Return Station F target dispatches the Rust gate route only from its source scene", async () => {
  const gate = item("world_gate", {
    entityId: "rs_world_gate_marker",
    transform: { positionM: { xM: 20, yM: 0, zM: 8 }, yawRad: 0 },
  });
  const snapshot = {
    worldId: "return_station", sceneId: "rs_core_room", worldEpoch: 12,
    player: { currentHp: 10, maxHp: 10, currentEnergy: 10, maxEnergy: 10,
      actionState: "idle", transform: { positionM: { xM: 20, yM: 0, zM: 8 }, yawRad: 0 } },
    interactables: [gate], doors: [], objectives: [],
  };
  const hud = deriveHudState(snapshot);
  assert.equal(hud.interactionId, "rs_world_gate_marker");
  const target = snapshot.interactables.find(entry => entry.entityId === hud.interactionId);
  assert.equal(target, gate);
  const calls = [];
  const client = {
    worldGate: async (id, epoch) => { calls.push(["worldGate", id, epoch]); return result(epoch + 1); },
    interact: async id => { calls.push(["interact", id]); return result(); },
    sceneTransition: async (id, epoch) => { calls.push(["sceneTransition", id, epoch]); return result(); },
  };
  await dispatchInteractable(client, target, snapshot.worldEpoch,
    { worldId: snapshot.worldId, sceneId: snapshot.sceneId });
  assert.deepEqual(calls, [["worldGate", "rs_world_gate_to_gh", 12]]);

  for (const source of [
    { worldId: "grey_hive", sceneId: "rs_core_room" },
    { worldId: "return_station", sceneId: "gh_exit" },
    undefined,
  ]) {
    await assert.rejects(dispatchInteractable(client, gate, 12, source), /E_WORLD_GATE_SOURCE_MISMATCH/);
  }
  assert.deepEqual(calls, [["worldGate", "rs_world_gate_to_gh", 12]]);
  await dispatchInteractable(client, { ...gate, kind: "terminal" }, 12,
    { worldId: "return_station", sceneId: "rs_core_room" });
  await dispatchInteractable(client, { ...gate, kind: "scene_transition" }, 12,
    { worldId: "return_station", sceneId: "rs_core_room" });
  await dispatchInteractable(client, { ...gate, entityId: "gh_extraction_return_to_rs" }, 12,
    { worldId: "grey_hive", sceneId: "gh_exit" });
  assert.deepEqual(calls.slice(1), [
    ["interact", "rs_world_gate_marker"],
    ["sceneTransition", "rs_world_gate_marker", 12],
    ["worldGate", "gh_extraction_return_to_rs", 12],
  ]);
});

test("Mist Harbor entry and extraction gates require their exact authoritative source scene", async () => {
  const calls = [];
  const client = { worldGate: async (id, epoch) => { calls.push([id, epoch]); return result(epoch + 1); } };
  const entry = item("world_gate", { entityId: "rs_mh_world_gate_marker" });
  await dispatchInteractable(client, entry, 21, { worldId: "return_station", sceneId: "rs_core_room" });
  const exit = item("world_gate", { entityId: "mh_extraction_return_to_rs" });
  await dispatchInteractable(client, exit, 22, { worldId: "mist_harbor", sceneId: "mh_extraction" });
  assert.deepEqual(calls, [["rs_world_gate_to_mh", 21], ["mh_extraction_return_to_rs", 22]]);
  await assert.rejects(dispatchInteractable(client, entry, 21, { worldId: "mist_harbor", sceneId: "mh_extraction" }),
    /E_WORLD_GATE_SOURCE_MISMATCH/);
  await assert.rejects(dispatchInteractable(client, exit, 22, { worldId: "mist_harbor", sceneId: "mh_breakwater" }),
    /E_WORLD_GATE_SOURCE_MISMATCH/);
  assert.equal(interactionErrorText("E_MH_EXTRACTION_REQUIRED"),
    "先完成雾港西侧航标、东侧航标和信号目标，再撤离。");
});

test("delayed Return Station gate receipts retain the source epoch and session freshness boundary", async () => {
  let finish;
  const source = { worldId: "return_station", sceneId: "rs_core_room", worldEpoch: 14 };
  const call = dispatchInteractable({ worldGate: () => new Promise(resolve => { finish = resolve; }) },
    item("world_gate", { entityId: "rs_world_gate_marker" }), source.worldEpoch, source);
  finish(result(15, { worldId: "grey_hive", sceneId: "gh_entry_maintenance", serverTick: 1,
    authorityRevision: 1 }));
  const receipt = await call;
  const current = { ...source, serverTick: 10, authorityRevision: 10 };
  assert.equal(isCurrentSceneInteractionResult(true, current, source, receipt), true);
  assert.equal(isCurrentSceneInteractionResult(false, current, source, receipt), false);
  assert.equal(isCurrentSceneInteractionResult(true, { ...current, worldEpoch: 15 }, source, receipt), false);
  assert.equal(isCurrentSceneInteractionResult(true, { ...current, sceneId: "gh_exit" }, source, receipt), false);
});

test("the mission terminal dispatches its dedicated query with the snapshot epoch", async () => {
  const calls = [];
  const client = {
    missionTerminalStatus: async (id, worldEpoch) => {
      calls.push({ id, worldEpoch });
      return result(worldEpoch);
    },
  };
  await dispatchInteractable(client, item("mission_terminal", { entityId: "rs_mission_terminal_marker" }), 31);
  assert.deepEqual(calls, [{ id: "rs_mission_terminal_marker", worldEpoch: 31 }]);
  assert.equal(interactionErrorText("E_MISSION_TERMINAL_OUT_OF_RANGE"), "请靠近归航站任务终端。");
  await assert.rejects(dispatchInteractable(client, item("mission_terminal", { active: false }), 31), /E_INTERACTION_INACTIVE/);
});

test("the capability terminal dispatches a read-only v3 query with the snapshot epoch", async () => {
  const calls = [];
  const client = {
    capabilityTerminalStatus: async (id, worldEpoch) => {
      calls.push({ id, worldEpoch });
      return result(worldEpoch);
    },
  };
  await dispatchInteractable(client, item("capability_terminal", { entityId: "rs_capability_terminal_marker" }), 32);
  assert.deepEqual(calls, [{ id: "rs_capability_terminal_marker", worldEpoch: 32 }]);
  assert.equal(interactionErrorText("E_CAPABILITY_TERMINAL_OUT_OF_RANGE"), "请靠近归航站能力终端。");
  await assert.rejects(dispatchInteractable(client, item("capability_terminal", { active: false }), 32), /E_INTERACTION_INACTIVE/);
});

test("Return Station save/rest target dispatches only the dedicated epoch-bound command", async () => {
  const calls = [];
  const client = { ...Object.fromEntries(["interact", "sceneTransition", "sceneCheckpoint", "sceneTrigger", "worldGate"]
    .map(method => [method, async () => { throw new Error(`unexpected ${method}`); }])),
  saveRestTerminal: async (id, worldEpoch) => { calls.push({ id, worldEpoch }); return result(worldEpoch); } };
  await dispatchInteractable(client, item("save_rest_terminal", { entityId: "rs_save_rest_terminal_marker" }), 23);
  assert.deepEqual(calls, [{ id: "rs_save_rest_terminal_marker", worldEpoch: 23 }]);
  assert.equal(interactionErrorText("E_REST_TERMINAL_COMBAT_ACTIVE"), "战斗状态结束后才能休整并保存。");
  await assert.rejects(dispatchInteractable(client, item("save_rest_terminal", { active: false }), 23), /E_INTERACTION_INACTIVE/);
});

test("known ordinary interaction still uses formal_interact and unknown or inactive kinds fail closed", async () => {
  const calls = [];
  const client = {
    interact: async id => { calls.push(["formal_interact", id]); return result(); },
    sceneTransition: async () => { calls.push(["transition"]); return result(); },
    sceneCheckpoint: async () => result(),
    sceneTrigger: async () => result(),
    worldGate: async () => result(),
  };
  await dispatchInteractable(client, item("power_console"), 4);
  await assert.rejects(dispatchInteractable(client, item("mystery_type"), 4), /E_INTERACTION_KIND_UNKNOWN:mystery_type/);
  await assert.rejects(dispatchInteractable(client, item("terminal", { active: false }), 4), /E_INTERACTION_INACTIVE/);
  assert.deepEqual(calls, [["formal_interact", "route-a"]]);
});

test("compiled scene catalog keeps static markers closed and routes confirmed scenes", async () => {
  const files = (await readdir(compiledScenesDir)).filter(name => name.endsWith(".json")).sort();
  const currentCatalog = {};
  const currentScenes = {};
  for (const file of files) {
    const definition = JSON.parse(await readFile(resolve(compiledScenesDir, file), "utf8"));
    currentCatalog[definition.sceneId] = [...new Set((definition.interactions ?? []).map(interaction => interaction.kind))].sort();
    currentScenes[definition.sceneId] = definition;
  }

  const currentSceneIds = Object.keys(currentCatalog).sort();
  const expectedSceneIds = [...Object.keys(RELEASE_SCENE_INTERACTION_KINDS),
    ...Object.keys(ADDITIONAL_SCENE_INTERACTION_KINDS)].sort();
  assert.deepEqual(currentSceneIds, expectedSceneIds);
  const arenaEnemySpawns = (currentScenes.cw_forged_guard_arena.spawns ?? [])
    .filter(spawn => spawn.kind === "enemy")
    .map(({ id, entityType, position }) => ({ id, entityType, position }));
  assert.deepEqual(arenaEnemySpawns, [{
    id: "cw_forged_guard_elite",
    entityType: "enemy.clockworks.forged_guard_elite",
    position: [12, 0, 7],
  }], "Arena contains exactly the authored Forged Guard elite enemy");
  const arenaShutterBlockers = (currentScenes.cw_forged_guard_arena.collision ?? [])
    .filter(collision => collision.requiresActorFirstKill !== undefined)
    .map(({ id, requiresActorFirstKill }) => ({ id, requiresActorFirstKill }));
  assert.deepEqual(arenaShutterBlockers, [
    { id: "cw_arena_shutter_west_blocker", requiresActorFirstKill: "cw_forged_guard_elite" },
    { id: "cw_arena_shutter_east_blocker", requiresActorFirstKill: "cw_forged_guard_elite" },
  ], "Arena shutter blockers are authored against the elite's first real kill");
  const warden = currentScenes.mh_warden_arena;
  assert.deepEqual(warden.spawns.filter(spawn=>spawn.kind==="enemy").map(({id,entityType,position})=>({id,entityType,position})),[{
    id:"mh_resonance_warden_staged_marker",entityType:"enemy.mist_harbor.resonance_warden",position:[18,0,8],
  }],"the authored Warden is a combat actor, never a clickable defeat marker");
  assert.equal(warden.interactions.some(item=>item.event),false);
  const regulator = currentScenes.cw_regulator_core;
  const regulatorSpawns = (regulator.spawns ?? []).filter(spawn =>
    spawn.id === "cw_prime_regulator" || spawn.entityType === "enemy.clockworks.prime_regulator");
  assert.deepEqual(regulatorSpawns.map(({ id, entityType, kind, position }) =>
    ({ id, entityType, kind, position })), [{
    id: "cw_prime_regulator",
    entityType: "enemy.clockworks.prime_regulator",
    kind: "enemy",
    position: [14, 0, 8],
  }], "core contains exactly the authored Prime Regulator");
  assert.deepEqual(regulator.interactions.filter(interaction => interaction.event != null).map(({ id, kind, event }) => ({ id, kind, event })),
    [{ id: "cw_regulator_core_console_staged", kind: "terminal", event: "clockworks_core" }],
    "only the authored core console can request progression; Rust requires real boss defeat");
  for (const [sceneId, kinds] of Object.entries(currentCatalog)) {
    const expectedKinds = RELEASE_SCENE_INTERACTION_KINDS[sceneId] ?? ADDITIONAL_SCENE_INTERACTION_KINDS[sceneId];
    assert.deepEqual(kinds, [...new Set(expectedKinds)].sort(), `compiled interaction kinds for ${sceneId}`);
  }
  assert.equal(Object.keys(RELEASE_SCENE_INTERACTION_KINDS).length, 11, "release design catalog includes the integrated eleven scenes");
  assert.equal(Object.keys(ADDITIONAL_SCENE_INTERACTION_KINDS).length, 19, "Return Station, native Mist Harbor, and staged Clockworks are cataloged");

  const calls = [];
  const client = {
    interact: async id => { calls.push(["formal_interact", id]); return result(); },
    sceneTransition: async id => { calls.push(["formal_scene_transition", id]); return result(5); },
    sceneCheckpoint: async id => { calls.push(["formal_scene_checkpoint", id]); return result(); },
    sceneTrigger: async id => { calls.push(["formal_scene_trigger", id]); return result(); },
    worldGate: async id => { calls.push(["formal_world_gate", id]); return result(5); },
  };
  for (const [sceneId, kinds] of Object.entries(RELEASE_SCENE_INTERACTION_KINDS)) {
    for (const kind of kinds) {
      if (STATIC_MARKER_KINDS.has(kind)) {
        await assert.rejects(dispatchInteractable(client, item(kind, { entityId: `${sceneId}:${kind}` }), 4),
          new RegExp(`E_INTERACTION_KIND_UNKNOWN:${kind}`));
        continue;
      }
      if (kind === "environment_control") {
        const id = currentScenes[sceneId].interactions.find(item => item.kind === kind).id;
        const epochClient = { ...client, environmentControl: async (id, epoch) => { calls.push(["formal_environment_control", id, epoch]); return result(epoch); } };
        const before = calls.length;
        await dispatchInteractable(epochClient, item(kind, { entityId: id }), 4);
        assert.deepEqual(calls.slice(before), [["formal_environment_control", id, 4]]);
        continue;
      }
      if (kind === "beacon_collect" || kind === "beacon_mount") {
        const id = currentScenes[sceneId].interactions.find(item => item.kind === kind).id;
        await dispatchInteractable(client, item(kind, {entityId:id}), 4, {worldId:"grey_hive",sceneId});
        continue;
      }
      assert.ok(RELEASE_ORDINARY_KINDS.has(kind), `ordinary interaction catalog missing ${kind}`);
      await dispatchInteractable(client, item(kind, { entityId: `${sceneId}:${kind}` }), 4);
    }
  }
  assert.deepEqual(calls, Object.entries(RELEASE_SCENE_INTERACTION_KINDS).flatMap(([sceneId, kinds]) =>
    kinds.flatMap(kind => kind === "environment_control"
      ? [["formal_environment_control", currentScenes[sceneId].interactions.find(item => item.kind === kind).id, 4]]
      : (kind === "beacon_collect" || kind === "beacon_mount") ? [["formal_interact", currentScenes[sceneId].interactions.find(item => item.kind === kind).id]]
      : RELEASE_ORDINARY_KINDS.has(kind) ? [["formal_interact", `${sceneId}:${kind}`]] : [])));

  for (const [sceneId, kinds] of Object.entries(ADDITIONAL_SCENE_INTERACTION_KINDS)) {
    for (const kind of kinds) {
      if (sceneId === "mh_resonance_tower" && kind === "terminal") {
        assert.ok(RELEASE_ORDINARY_KINDS.has(kind), "signal console uses the existing terminal kind");
        continue;
      }
      if ((sceneId === "mh_tidal_warehouse" || sceneId === "mh_breakwater") && kind === "beacon") {
        assert.ok(RELEASE_ORDINARY_KINDS.has(kind), "authored Mist Harbor beacons use the Rust interaction command");
        continue;
      }
      if (sceneId === "mh_pump_station" && kind === "pump_control") {
        assert.ok(RELEASE_ORDINARY_KINDS.has(kind), "authored pump control uses the Rust interaction command");
        continue;
      }
      if (["gh_deep_decon", "cw_boiler_chamber", "cw_furnace_heart"].includes(sceneId) && kind === "environment_control") {
        const control = currentScenes[sceneId].interactions.find(item => item.kind === kind);
        assert.ok(control.environmentControl); assert.equal(control.event, null);
        const before = calls.length;
        const epochClient={...client, environmentControl: async (id,epoch)=> { calls.push(["formal_environment_control",id,epoch]); return result(epoch); }};
        await dispatchInteractable(epochClient, item(kind, { entityId: control.id }), 4);
        assert.deepEqual(calls.slice(before), [["formal_environment_control", control.id,4]]);
        continue;
      }
      if (sceneId === "cw_pressure_hall" && kind === "valve_control") {
        assert.ok(RELEASE_ORDINARY_KINDS.has(kind), "authored Pressure Hall valves use the existing Rust interaction command");
        const before = calls.length;
        await dispatchInteractable(client, item(kind, { entityId: "cw_pressure_valve_01_staged" }), 4);
        assert.deepEqual(calls.slice(before), [["formal_interact", "cw_pressure_valve_01_staged"]]);
        continue;
      }
      if (sceneId === "cw_regulator_core" && kind === "coolant_valve") {
        const before = calls.length;
        const control = item(kind, { entityId: "cw_regulator_valve_furnace_link_staged" });
        await dispatchInteractable(client, control, 4, { worldId: "clockworks", sceneId });
        assert.deepEqual(calls.slice(before), [["formal_interact", control.entityId]]);
        await assert.rejects(dispatchInteractable(client, control, 4, { worldId: "clockworks", sceneId: "cw_boiler_chamber" }), /SOURCE_MISMATCH/);
        await assert.rejects(dispatchInteractable(client, item(kind, { entityId: "forged-valve" }), 4, { worldId: "clockworks", sceneId }), /SOURCE_MISMATCH/);
        continue;
      }
      if (sceneId === "cw_regulator_core" && kind === "terminal") {
        const before = calls.length;
        await dispatchInteractable(client, item(kind, { entityId: "cw_regulator_core_console_staged" }), 4);
        assert.deepEqual(calls.slice(before), [["formal_interact", "cw_regulator_core_console_staged"]]);
        continue;
      }
      if (sceneId === "cw_shutdown_exit" && kind === "terminal") {
        assert.ok(RELEASE_ORDINARY_KINDS.has(kind), "the Shutdown terminal uses the existing Rust interaction command");
        const before = calls.length;
        await dispatchInteractable(client, item(kind, { entityId: "cw_master_shutdown_staged" }), 4);
        assert.deepEqual(calls.slice(before), [["formal_interact", "cw_master_shutdown_staged"]]);
        continue;
      }
      await assert.rejects(dispatchInteractable(client, item(kind, { entityId: `${sceneId}:${kind}` }), 4),
        new RegExp(`E_INTERACTION_KIND_UNKNOWN:${kind}`));
    }
  }

  const west = JSON.parse(await readFile(resolve(compiledScenesDir, "mh_tidal_warehouse.json"), "utf8"))
    .interactions.find(interaction => interaction.id === "mh_west_beacon");
  const east = JSON.parse(await readFile(resolve(compiledScenesDir, "mh_breakwater.json"), "utf8"))
    .interactions.find(interaction => interaction.id === "mh_east_beacon");
  assert.deepEqual([west?.event, east?.event], ["mist_beacon_west", "mist_beacon_east"]);
  calls.length = 0;
  await dispatchInteractable(client, item(west.kind, { entityId: west.id }), 4);
  await dispatchInteractable(client, item(east.kind, { entityId: east.id }), 4);
  assert.deepEqual(calls, [["formal_interact", "mh_west_beacon"], ["formal_interact", "mh_east_beacon"]]);

  const pump = JSON.parse(await readFile(resolve(compiledScenesDir, "mh_pump_station.json"), "utf8"))
    .interactions.find(interaction => interaction.id === "mh_pump_control_primary");
  calls.length = 0;
  await dispatchInteractable(client, item(pump.kind, { entityId: pump.id }), 4);
  assert.deepEqual(calls, [["formal_interact", "mh_pump_control_primary"]]);

  calls.length = 0;
  await dispatchInteractable(client, item("scene_transition", { entityId: "route" }), 4);
  await dispatchInteractable(client, item("scene_checkpoint", { entityId: "checkpoint" }), 4);
  await dispatchInteractable(client, item("scene_trigger", { entityId: "trigger" }), 4);
  await dispatchInteractable(client, item("world_gate", { entityId: "rs_world_gate_to_gh" }), 4);
  assert.deepEqual(calls, [
    ["formal_scene_transition", "route"],
    ["formal_scene_checkpoint", "checkpoint"],
    ["formal_scene_trigger", "trigger"],
    ["formal_world_gate", "rs_world_gate_to_gh"],
  ]);
});

test("locked, unknown-kind, and out-of-range failures have actionable visible feedback", () => {
  assert.equal(interactionErrorText("E_SCENE_SENTINEL_FIRST_KILL_REQUIRED"), "先击败哨卫，再前往信标室。");
  assert.equal(interactionErrorText("E_SCENE_RUNTIME_ProgressionLocked"), "路线尚未解锁，现场状态未改变。");
  assert.equal(interactionErrorText("E_INTERACTION_KIND_UNKNOWN:future_kind"), "无法识别此目标类型，已阻止操作。");
  assert.equal(interactionErrorText("E_SCENE_RUNTIME_OutOfRange"), "距离目标太远，请靠近后再试。");
  assert.equal(interactionErrorText("E_SCENE_FUTURE_GATE_CODE"), "现场操作失败：E_SCENE_FUTURE_GATE_CODE");
});

test("locked route results remain visible data and delayed receipts cannot cross session or epoch boundaries", async () => {
  const locked = { ...result(), applied: false, errorCode: "E_SCENE_RUNTIME_ProgressionLocked" };
  const lockedResult = await dispatchInteractable({
    interact: async () => result(), sceneTransition: async () => locked,
    sceneCheckpoint: async () => locked, sceneTrigger: async () => locked,
    worldGate: async () => locked,
  }, item("scene_transition"), 4);
  assert.equal(lockedResult.applied, false);
  assert.match(lockedResult.errorCode, /ProgressionLocked/);
  assert.equal(interactionErrorText(lockedResult.errorCode), "路线尚未解锁，现场状态未改变。");
  const sentinelGate = { ...result(), applied: false, errorCode: "E_SCENE_SENTINEL_FIRST_KILL_REQUIRED" };
  assert.equal(sentinelGate.applied, false);
  assert.equal(interactionErrorText(sentinelGate.errorCode), "先击败哨卫，再前往信标室。");

  let finish;
  const delayed = dispatchInteractable({
    interact: async () => result(), sceneTransition: () => new Promise(resolve => { finish = resolve; }),
    sceneCheckpoint: async () => result(), sceneTrigger: async () => result(),
    worldGate: async () => result(),
  }, item("scene_transition"), 4);
  const oldReceipt = result(5, { sceneId: "gh_gate_a" });
  finish(oldReceipt);
  const receipt = await delayed;
  const source = { worldId: "grey_hive", sceneId: "grey_hive", worldEpoch: 4 };
  assert.equal(isCurrentSceneInteractionResult(true, { ...source, serverTick: 2, authorityRevision: 2 }, source, receipt), true);
  assert.equal(isCurrentSceneInteractionResult(false, { ...source, serverTick: 2, authorityRevision: 2 }, source, receipt), false);
  assert.equal(isCurrentSceneInteractionResult(true, { worldId: "grey_hive", sceneId: "gh_gate_a", worldEpoch: 5, serverTick: 2, authorityRevision: 2 }, source, receipt), false);
  assert.equal(isCurrentSceneInteractionResult(true, { ...source, serverTick: 2, authorityRevision: 2 }, source, result(3)), false);
  assert.equal(isCurrentSceneInteractionResult(true, { ...source, serverTick: 6, authorityRevision: 6 }, source,
    { ...result(), snapshot: { ...result().snapshot, serverTick: 5, authorityRevision: 5 } }), false);
});


test("Regulator phase valve uses the exact source epoch and inactive phase never dispatches", async () => {
  const calls = [];
  const client = { interact: async (id, epoch) => { calls.push([id, epoch]); return result(epoch); } };
  const control = item("coolant_valve", { entityId: "cw_regulator_valve_furnace_link_staged" });
  const source = { worldId: "clockworks", sceneId: "cw_regulator_core" };
  await dispatchInteractable(client, control, 57, source);
  assert.deepEqual(calls, [[control.entityId, 57]]);
  await assert.rejects(dispatchInteractable(client, { ...control, active: false }, 58, source), /INACTIVE/);
  await assert.rejects(dispatchInteractable(client, control, 58), /SOURCE_MISMATCH/);
  assert.equal(calls.length, 1);
});
