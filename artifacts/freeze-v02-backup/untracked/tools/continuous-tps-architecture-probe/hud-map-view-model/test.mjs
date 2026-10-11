import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createViewModelAdapter, hasUnsafeOutput } from "./view-model.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "../../..");
const resultPath = path.join(root, "tools/continuous-tps-architecture-probe/results/hud-map-view-model-tp15a1.json");
const scenarios = [];

function hash(value) {
  return crypto.createHash("sha256").update(JSON.stringify(value)).digest("hex");
}

function clone(value) {
  return structuredClone(value);
}

function finite(value) {
  return typeof value === "number" && Number.isFinite(value);
}

function deepFrozen(value, seen = new WeakSet()) {
  if (!value || typeof value !== "object") return true;
  if (seen.has(value)) return true;
  if (!Object.isFrozen(value)) return false;
  seen.add(value);
  return Object.values(value).every((child) => deepFrozen(child, seen));
}

function world({ epoch = 1, tick = 1, ackSeq = tick, x = 2, y = 0, z = 3, vx = 0, vy = 0, vz = 0, objectives = null, choiceState = null, result = null, ...extra } = {}) {
  return {
    protocol: "continuous-ipc",
    version: 1,
    worldEpoch: epoch,
    serverTick: tick,
    ackSeq,
    serverTimeMs: tick * 1000 / 30,
    worldId: "grey_hive",
    player: {
      transform: { positionM: { xM: x, yM: y, zM: z }, yawRad: 0.25 },
      velocityMps: { xM: vx, yM: vy, zM: vz },
      hp: 80,
      maxHp: 100,
      energy: 25,
      maxEnergy: 50,
      scrap: 12,
    },
    actors: [],
    doors: [],
    objectives: objectives ?? [
      { id: "hive_power", title: "恢复主电", done: true, optional: false },
      { id: "hive_lockdown", title: "解除隔离", done: false, optional: false },
      { id: "hive_choice", title: "白芷的请求", done: false, optional: true },
    ],
    checkpoint: "gh_cp_post_bio",
    choiceState,
    result,
    ...extra,
  };
}

function capability(epoch = 1, tick = 1, effective = []) {
  return { protocol: "continuous-capability", version: 1, worldId: "grey_hive", worldEpoch: epoch, tick, effective, baseline: ["ui.accessibility", "controls.basic"] };
}

function rearGrant(grantId, source = "equipment", tier = "C") {
  return { grantId, capabilityId: "perception.rear_view", source, tier, scope: "persistent", worldId: "grey_hive", worldEpoch: 1, parameters: { distanceM: 8, yawFovDeg: 120, resolutionTier: "low" } };
}

function scene({ formal = true, candidateOnly = false, hashValue = "scene-hash", expectedHash = "scene-hash", layout = "v7", expectedLayout = "v7", assetAvailable = true } = {}) {
  return {
    worldId: "grey_hive",
    formalSceneReady: formal,
    candidateOnly,
    sceneHash: hashValue,
    expectedSceneHash: expectedHash,
    layoutVersion: layout,
    expectedLayoutVersion: expectedLayout,
    room: "GH-B05",
    doors: [{ id: "door-b", label: "隔离门 B", xM: 1, yM: 0, zM: 2, open: false }],
    markers: [{ id: "cp-post-bio", label: "恢复点", xM: 3, yM: 0, zM: 4, kind: "checkpoint" }],
    assetAvailable,
  };
}

function record(name, input, expected, actual, pass, events = []) {
  scenarios.push({
    name,
    input,
    events,
    expected,
    actual,
    pass,
    stateHash: hash(actual),
    evidenceLevel: "isolated_immutable_serialized_view_model",
  });
}

function simpleView(vm) {
  return {
    connection: vm.connection,
    hud: vm.hud,
    objective: vm.objective,
    map: vm.map,
    capabilities: vm.capabilities,
    accessibility: vm.accessibility,
    player: vm.player,
    diagnostics: vm.diagnostics,
    frozen: deepFrozen(vm),
    unsafe: hasUnsafeOutput(vm),
  };
}

{
  const input = { worldView: world({ tick: 1, choiceState: "unresolved" }), capabilityView: capability(), sceneMetadata: scene(), uiPreferences: { subtitles: true, fontScale: 1.25, colorBlindMode: "deuteranopia", contrast: "high", keybinds: { interact: "F" } }, receivedAtMs: 0 };
  const before = clone(input);
  const vm = createViewModelAdapter().accept(input);
  const actual = simpleView(vm);
  record("normal_snapshot", input, { status: "accepted", map: "available", gate: "B", accessibilityIndependent: true, immutable: true }, actual, vm.connection.status === "accepted" && vm.map.status === "available" && vm.objective.gatePrompt.gate === "B" && vm.accessibility.fontScale === 1.25 && vm.capabilities.active["perception.rear_view"] === null && actual.frozen && !actual.unsafe && JSON.stringify(input) === JSON.stringify(before), ["accept valid WorldView", "derive HUD/objective/map/accessibility", "freeze output"]);
}

{
  const adapter = createViewModelAdapter();
  adapter.accept({ worldView: world({ tick: 1 }), sceneMetadata: scene() });
  const vm = adapter.accept({ worldView: world({ tick: 1, x: 90 }), sceneMetadata: scene() });
  record("duplicate_and_out_of_order_tick", { ticks: [1, 1], positions: [2, 90] }, { diagnostic: "duplicate_or_out_of_order_tick", stateTick: 1 }, { diagnostic: vm.connection.diagnostic, stateTick: vm.connection.tick, positionX: vm.player.positionM.xM }, vm.connection.diagnostic.code === "duplicate_or_out_of_order_tick" && vm.connection.tick === 1 && vm.player.positionM.xM === 2, ["accept tick 1", "reject duplicate tick without mutation"]);
}

{
  const adapter = createViewModelAdapter();
  adapter.accept({ worldView: world({ epoch: 3, tick: 4 }), sceneMetadata: scene() });
  const vm = adapter.accept({ worldView: world({ epoch: 2, tick: 99, x: 99 }), sceneMetadata: scene() });
  record("stale_epoch", { acceptedEpoch: 3, incomingEpoch: 2 }, { diagnostic: "stale_epoch", stateEpoch: 3 }, { diagnostic: vm.connection.diagnostic, stateEpoch: vm.connection.worldEpoch, positionX: vm.player.positionM.xM }, vm.connection.diagnostic.code === "stale_epoch" && vm.connection.worldEpoch === 3 && vm.player.positionM.xM === 2, ["accept epoch 3", "drop epoch 2"]);
}

{
  const adapter = createViewModelAdapter();
  adapter.accept({ worldView: world({ tick: 1, ackSeq: 8 }), sceneMetadata: scene() });
  const vm = adapter.accept({ worldView: world({ tick: 2, ackSeq: 7, x: 44 }), sceneMetadata: scene() });
  record("ack_rollback", { acceptedAck: 8, incomingAck: 7 }, { diagnostic: "ack_regression", stateTick: 1 }, { diagnostic: vm.connection.diagnostic, stateTick: vm.connection.tick, ackSeq: vm.connection.ackSeq }, vm.connection.diagnostic.code === "ack_regression" && vm.connection.tick === 1 && vm.connection.ackSeq === 8, ["accept ack 8", "reject ack rollback"]);
}

{
  const adapter = createViewModelAdapter();
  adapter.accept({ worldView: world({ epoch: 1, tick: 5 }), capabilityView: capability(1, 5, [rearGrant("old")]), sceneMetadata: scene() });
  const vm = adapter.accept({ worldView: world({ epoch: 2, tick: 1, x: 20 }), capabilityView: undefined, sceneMetadata: scene() });
  record("world_replacement", { oldEpoch: 1, newEpoch: 2, oldRearView: true }, { status: "world_replaced", rearView: false, epoch: 2 }, { status: vm.connection.status, epoch: vm.connection.worldEpoch, rearView: vm.capabilities.availability["perception.rear_view"], diagnostic: vm.connection.diagnostic }, vm.connection.status === "world_replaced" && vm.connection.worldEpoch === 2 && vm.capabilities.availability["perception.rear_view"] === false, ["accept epoch 1", "replace world epoch", "clear world-scoped capability"]);
}

{
  const input = { worldView: world({ tick: 1, player: undefined, unknownField: { path: "must not leak" } }), sceneMetadata: undefined, capabilityView: undefined, uiPreferences: undefined };
  delete input.worldView.player;
  const vm = createViewModelAdapter().accept(input);
  const actual = simpleView(vm);
  record("partial_missing_unknown_fields", input, { status: "accepted", playerDiagnostic: "player_transform_unavailable", mapDiagnostic: "scene_metadata_missing", textFallback: true }, actual, vm.connection.status === "accepted" && vm.diagnostics.some((item) => item.code === "player_transform_unavailable") && vm.map.diagnostic.code === "scene_metadata_missing" && vm.hud.textFallback && !actual.unsafe, ["accept partial WorldView", "ignore unknown field", "retain text fallback"]);
}

{
  const nanVm = createViewModelAdapter().accept({ worldView: world({ tick: 1, x: NaN }), sceneMetadata: scene() });
  const infVm = createViewModelAdapter().accept({ worldView: world({ tick: Infinity }), sceneMetadata: scene() });
  record("nan_infinity_input", { nan: "player.positionM.xM", infinity: "serverTick" }, { nan: "world_non_finite", infinity: "world_clock_invalid" }, { nan: nanVm.connection.diagnostic.code, infinity: infVm.connection.diagnostic.code }, nanVm.connection.diagnostic.code === "world_non_finite" && infVm.connection.diagnostic.code === "world_clock_invalid", ["reject NaN", "reject Infinity"]);
}

{
  const oversized = world({ tick: 1, actors: Array.from({ length: 257 }, (_, index) => ({ id: String(index) })) });
  const vm = createViewModelAdapter().accept({ worldView: oversized, sceneMetadata: scene() });
  record("oversize_input", { actors: 257 }, { diagnostic: "input_oversize" }, { diagnostic: vm.connection.diagnostic }, vm.connection.diagnostic.code === "input_oversize", ["enforce bounded snapshot arrays"]);
}

{
  const vm = createViewModelAdapter().accept({ worldView: world({ tick: 1 }), sceneMetadata: scene({ formal: false, candidateOnly: true, assetAvailable: false }) });
  record("candidate_map_room_door_marker", { scene: "candidateOnly", room: "GH-B05", doors: 1, markers: 1 }, { mapStatus: "candidate", candidateOnly: true, textFallback: true }, { map: vm.map, diagnostics: vm.diagnostics }, vm.map.status === "candidate" && vm.map.candidateOnly && vm.map.room === "GH-B05" && vm.map.doors.length === 1 && vm.map.markers.length === 1 && vm.map.textFallback === true, ["accept candidate metadata", "retain candidateOnly flag", "use text fallback when asset unavailable"]);
}

{
  const badHash = createViewModelAdapter().accept({ worldView: world({ tick: 1 }), sceneMetadata: scene({ hashValue: "wrong" }) });
  const badLayout = createViewModelAdapter().accept({ worldView: world({ tick: 1 }), sceneMetadata: scene({ layout: "old-v1" }) });
  record("wrong_scene_hash_layout", { hash: "wrong", layout: "old-v1" }, { hash: "scene_hash_mismatch", layout: "scene_layout_mismatch" }, { hash: badHash.map.diagnostic.code, layout: badLayout.map.diagnostic.code }, badHash.map.status === "unavailable" && badHash.map.diagnostic.code === "scene_hash_mismatch" && badLayout.map.status === "unavailable" && badLayout.map.diagnostic.code === "scene_layout_mismatch", ["reject wrong scene hash", "reject wrong layout"]);
}

{
  const vm = createViewModelAdapter().accept({ worldView: world({ tick: 1, result: { title: "任务完成", summary: "返回归航站", rank: "A", map: null } }), sceneMetadata: scene() });
  record("completed_result_map_null", { result: "completed", map: null }, { mapStatus: "unavailable", diagnostic: "terminal_result_no_map", noCoordinates: true }, { map: vm.map }, vm.map.status === "unavailable" && vm.map.diagnostic.code === "terminal_result_no_map" && vm.map.room === null && vm.map.markers.length === 0, ["detect terminal result", "suppress map coordinates"]);
}

{
  const outputs = [];
  for (const choiceState of ["taken", "left", "unresolved"]) {
    const vm = createViewModelAdapter().accept({ worldView: world({ tick: 1, choiceState, objectives: [{ id: "hive_power", title: "恢复主电", done: true, optional: false }, { id: "hive_lockdown", title: "解除隔离", done: true, optional: false }, { id: "hive_choice", title: "白芷的请求", done: choiceState !== "unresolved", optional: true }] }), sceneMetadata: scene() });
    outputs.push({ choiceState: vm.objective.choiceState, gatePrompt: vm.objective.gatePrompt });
  }
  record("choice_states_do_not_gate_gate_b", { choices: ["taken", "left", "unresolved"], lockdown: true }, { allPreserved: true, gatePrompt: null }, { outputs }, outputs.every((item) => ["taken", "left", "unresolved"].includes(item.choiceState)) && outputs.every((item) => item.gatePrompt === null), ["render all legal choice states", "derive Gate B from lockdown only"]);
}

{
  const adapter = createViewModelAdapter();
  const base = { worldView: world({ tick: 1 }), sceneMetadata: scene() };
  const noGrant = adapter.accept(base);
  const grant = adapter.accept({ ...base, worldView: world({ tick: 2 }), capabilityView: capability(1, 2, [rearGrant("rear-1")]) });
  const revoke = adapter.accept({ ...base, worldView: world({ tick: 3 }), capabilityView: capability(1, 3, []) });
  const regrant = adapter.accept({ ...base, worldView: world({ tick: 4 }), capabilityView: capability(1, 4, [rearGrant("rear-2", "progression", "A")]) });
  record("rear_view_no_grant_grant_revoke_regrant", { sequence: ["none", "grant", "revoke", "regrant"] }, { active: [false, true, false, true] }, { active: [noGrant.capabilities.availability["perception.rear_view"], grant.capabilities.availability["perception.rear_view"], revoke.capabilities.availability["perception.rear_view"], regrant.capabilities.availability["perception.rear_view"]] }, noGrant.capabilities.availability["perception.rear_view"] === false && grant.capabilities.active["perception.rear_view"].grantId === "rear-1" && revoke.capabilities.availability["perception.rear_view"] === false && regrant.capabilities.active["perception.rear_view"].grantId === "rear-2", ["no grant", "grant", "revoke", "regrant"]);
}

{
  const adapter = createViewModelAdapter();
  adapter.accept({ worldView: world({ tick: 1 }), capabilityView: capability(1, 1, [rearGrant("equipment", "equipment", "C")]), sceneMetadata: scene() });
  const high = adapter.accept({ worldView: world({ tick: 2 }), capabilityView: capability(1, 2, [rearGrant("equipment", "equipment", "C"), rearGrant("progression", "progression", "A")]), sceneMetadata: scene() });
  const revoked = adapter.accept({ worldView: world({ tick: 3 }), capabilityView: capability(1, 3, []), sceneMetadata: scene() });
  const late = adapter.accept({ worldView: world({ tick: 4 }), capabilityView: capability(1, 2, [rearGrant("late", "progression", "A")]), sceneMetadata: scene() });
  record("multi_source_and_late_callback", { priority: ["progression", "equipment"], lateTick: 2, revokedAtTick: 3 }, { winner: "progression", afterLateRevoke: false, error: "capability_stale_tick" }, { winner: high.capabilities.active["perception.rear_view"].grantId, afterRevoke: revoked.capabilities.availability["perception.rear_view"], afterLate: late.capabilities.availability["perception.rear_view"], diagnostics: late.capabilities.errors }, high.capabilities.active["perception.rear_view"].grantId === "progression" && revoked.capabilities.availability["perception.rear_view"] === false && late.capabilities.availability["perception.rear_view"] === false && late.diagnostics.some((item) => item.code === "capability_stale_tick"), ["sort multi-source capability", "revoke", "drop late callback"]);
}

{
  const input = { worldView: world({ tick: 1, subtitle: "保持安静" }), capabilityView: undefined, sceneMetadata: undefined, uiPreferences: { subtitles: true, fontScale: 1.5, colorBlindMode: "protanopia", contrast: "high", keybinds: { interact: "Q", map: "Tab", rearView: "V" } } };
  const vm = createViewModelAdapter().accept(input);
  record("accessibility_independent_of_capability", { capability: "missing", preferences: input.uiPreferences }, { subtitle: "保持安静", fontScale: 1.5, colorBlindMode: "protanopia", keybindInteract: "Q" }, { subtitle: vm.hud.subtitle, accessibility: vm.accessibility, capabilityErrors: vm.capabilities.errors }, vm.hud.subtitle === "保持安静" && vm.accessibility.fontScale === 1.5 && vm.accessibility.colorBlindMode === "protanopia" && vm.accessibility.keybinds.interact === "Q", ["render baseline subtitle", "render accessibility without capability"]);
}

{
  const adapter = createViewModelAdapter({ allowExtrapolation: true, horizonMs: 50, correctionMeters: 0.25 });
  adapter.accept({ worldView: world({ tick: 1, x: 0, vx: 3 }), receivedAtMs: 0, sceneMetadata: scene() });
  adapter.accept({ worldView: world({ tick: 2, x: 0.1, vx: 3 }), receivedAtMs: 33.333, sceneMetadata: scene() });
  const interpolation = adapter.render(16.666);
  const extrapolation = adapter.render(49.999);
  const correctionAdapter = createViewModelAdapter({ allowExtrapolation: true, horizonMs: 50, correctionMeters: 0.25 });
  correctionAdapter.accept({ worldView: world({ tick: 1, x: 0, vx: 30 }), receivedAtMs: 0, sceneMetadata: scene() });
  correctionAdapter.accept({ worldView: world({ tick: 2, x: 0, vx: 30 }), receivedAtMs: 33.333, sceneMetadata: scene() });
  const correction = correctionAdapter.render(49.999);
  record("bounded_interpolation_extrapolation_correction", { snapshotHz: 30, renderHz: 60, horizonMs: 50, correctionMeters: 0.25 }, { interpolation: "interpolation", extrapolation: "extrapolation", correction: true }, { interpolation: interpolation.player, extrapolation: extrapolation.player, correction: correction.player }, interpolation.player.renderMode === "interpolation" && extrapolation.player.renderMode === "extrapolation" && finite(extrapolation.player.positionM.xM) && correction.player.renderMode === "correction" && correction.player.correctionFlag === true, ["accept 30Hz snapshot 1", "accept 30Hz snapshot 2", "render 60Hz midpoint", "bounded extrapolation", "flag correction beyond bound"]);
}

const output = {
  schema: "continuous_hud_map_view_model_tp15a1_v1",
  status: scenarios.every((scenario) => scenario.pass) ? "passed_candidate_isolated" : "blocked_probe_failure",
  machinePass: scenarios.every((scenario) => scenario.pass),
  candidateOnly: true,
  outputContract: {
    immutableJsonOnly: true,
    noCommands: true,
    noAuthority: true,
    noDomWebglCanvas: true,
    forbiddenOutputKinds: ["function", "DOM", "WebGL", "Canvas", "GPU handle", "file path", "media bytes"],
  },
  inputSchema: { worldView: "serialized WorldView-like JSON", capabilityView: "serialized CapabilityView-like JSON", sceneMetadata: "scene export metadata only", uiPreferences: "accessibility and keybind preferences" },
  viewModelSchema: { sections: ["connection", "hud", "objective", "map", "capabilities", "accessibility", "player", "diagnostics"], epochTickAck: true, correctionFlag: true },
  scenarios,
  policies: {
    gates: { gateA: "hive_power only", gateB: "hive_lockdown only; choice/NPC excluded" },
    rearView: "only valid same-epoch grant; revoke and epoch replacement remove active status immediately",
    accessibility: "subtitles, scalable text, color/contrast and keybinds are baseline and capability-independent",
    map: "formal scene required for available; candidate scene remains candidateOnly; missing/wrong/terminal scene is unavailable with text fallback",
    interpolation: { snapshotHz: 30, renderHz: 60, maxExtrapolationMs: 50, maxCorrectionMeters: 0.25, authoritativeStateMutation: false },
  },
  formalGaps: [
    "formal UI DOM/render integration is not changed or verified",
    "formal scene/map export remains unavailable; all map candidate coordinates are non-formal",
    "CapabilityView is an isolated serialized contract; no formal Rust capability resolver exists",
    "TP-14 campaign/input wiring, visual accessibility review, renderer performance and human usability remain untested",
  ],
  gateConclusion: { tp15: "blocked", relatedGates: "blocked", architecture: "no_go", candidate: "passed_candidate_isolated" },
};

fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(output, null, 2)}\n`);
console.log(`TP-15A1 ${output.machinePass ? "passed" : "failed"}: ${scenarios.length} scenarios; result=${resultPath}`);
if (!output.machinePass) process.exitCode = 1;
