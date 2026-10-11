#!/usr/bin/env node

/**
 * AG-01 isolated architecture probe.
 *
 * This file intentionally does not import the game, Tauri, Babylon, Cargo
 * crates, or any player save. It measures deterministic toy models only.
 */

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { performance } from "node:perf_hooks";

const ROOT = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));

function parseArgs(argv) {
  const result = {};
  for (let i = 0; i < argv.length; i += 1) {
    const token = argv[i];
    if (token.startsWith("--")) result[token.slice(2)] = argv[i + 1] ?? true;
  }
  return result;
}

function percentile(values, p) {
  if (!values.length) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const index = (sorted.length - 1) * p;
  const lower = Math.floor(index);
  const upper = Math.ceil(index);
  if (lower === upper) return sorted[lower];
  return sorted[lower] + (sorted[upper] - sorted[lower]) * (index - lower);
}

function round(value, digits = 6) {
  const scale = 10 ** digits;
  return Math.round(value * scale) / scale;
}

function environment() {
  return {
    node: process.version,
    platform: process.platform,
    arch: process.arch,
    cpu_count: os.cpus().length,
    cpu_model: os.cpus()[0]?.model ?? "unknown",
    cwd: process.cwd(),
    probe_root: ROOT,
  };
}

function deterministicWork(seed, iterations = 600) {
  let value = seed;
  for (let i = 0; i < iterations; i += 1) {
    value = (value * 1.000001 + i * 0.0000001) % 997.0;
  }
  return value;
}

function tickRateProbe(rate) {
  const dt = 1 / rate;
  const frameHz = 120;
  const frameDt = 1 / frameHz;
  let accumulator = 0;
  let tickCount = 0;
  let droppedTicks = 0;
  let maxBacklog = 0;
  let maxTicksPerFrame = 0;
  let paused = false;
  let pauseResumed = false;
  const durationsUs = [];
  const virtualFrames = 120 * 12;
  const tickStart = performance.now();
  for (let frame = 0; frame < virtualFrames; frame += 1) {
    const pauseWindow = frame >= 360 && frame < 600;
    if (pauseWindow) {
      paused = true;
      accumulator = 0;
      continue;
    }
    if (paused) {
      paused = false;
      pauseResumed = true;
    }
    const jitter = ((frame * 17) % 5 - 2) * 0.0004;
    accumulator += Math.max(0, frameDt + jitter);
    const before = accumulator;
    let steps = 0;
    const maxCatchUp = 5;
    while (accumulator >= dt && steps < maxCatchUp) {
      const started = performance.now();
      deterministicWork(tickCount + rate, 600);
      const ended = performance.now();
      durationsUs.push((ended - started) * 1000);
      accumulator -= dt;
      tickCount += 1;
      steps += 1;
    }
    if (accumulator >= dt) {
      const excess = Math.floor(accumulator / dt);
      droppedTicks += excess;
      accumulator -= excess * dt;
    }
    maxBacklog = Math.max(maxBacklog, before);
    maxTicksPerFrame = Math.max(maxTicksPerFrame, steps);
  }
  const elapsedMs = performance.now() - tickStart;
  return {
    rate_hz: rate,
    fixed_dt_ms: round(dt * 1000, 6),
    virtual_duration_s: 12,
    virtual_ticks: tickCount,
    measured_probe_wall_ms: round(elapsedMs, 3),
    tick_work_mean_us: round(durationsUs.reduce((a, b) => a + b, 0) / durationsUs.length, 3),
    tick_work_p95_us: round(percentile(durationsUs, 0.95), 3),
    max_accumulator_ms: round(maxBacklog * 1000, 3),
    max_ticks_per_render_frame: maxTicksPerFrame,
    dropped_ticks_after_cap: droppedTicks,
    pause_resume_observed: pauseResumed,
    pause_policy: "reset accumulator while paused; discard excess beyond maxCatchUp",
    status: droppedTicks === 0 && pauseResumed ? "measured_candidate" : "needs_review",
  };
}

function inputAt(timeMs) {
  if (timeMs < 2000) return { x: 0, z: 1 };
  if (timeMs < 4000) return { x: 0.70710678, z: 0.70710678 };
  if (timeMs < 5000) return { x: 0, z: 0 };
  return { x: -1, z: 0 };
}

function add(a, b) {
  return { x: a.x + b.x, z: a.z + b.z };
}

function scale(a, factor) {
  return { x: a.x * factor, z: a.z * factor };
}

function distance(a, b) {
  return Math.hypot(a.x - b.x, a.z - b.z);
}

function positionAt(timeMs) {
  const cutovers = [0, 2000, 4000, 5000, timeMs].filter((value) => value >= 0 && value <= timeMs).sort((a, b) => a - b);
  let position = { x: 0, z: 0 };
  for (let i = 1; i < cutovers.length; i += 1) {
    const startMs = cutovers[i - 1];
    const endMs = cutovers[i];
    position = add(position, scale(inputAt((startMs + endMs) / 2), speedForIpc * (endMs - startMs) / 1000));
  }
  return position;
}

const speedForIpc = 4.5;

function ipcCase(renderHz, injectedDelayMs) {
  const durationMs = 8000;
  const tickHz = 60;
  const inputHz = 30;
  const renderDt = 1000 / renderHz;
  const tickDt = 1000 / tickHz;
  const sendDt = 1000 / inputHz;
  const inputEvents = [];
  let seq = 0;
  for (let time = 0; time <= durationMs + 0.01; time += sendDt) {
    const input = inputAt(time);
    const event = { seq: ++seq, client_time_ms: time, input };
    inputEvents.push(event);
  }
  const snapshots = [];
  let latestSeq = 0;
  for (let serverTime = 0; serverTime <= durationMs + 0.01; serverTime += tickDt) {
    while (latestSeq + 1 < inputEvents.length && inputEvents[latestSeq + 1].client_time_ms <= serverTime) latestSeq += 1;
    const snapshot = {
      serverTime,
      arrivalMs: serverTime + injectedDelayMs + ((Math.floor(serverTime / tickDt) % 4) - 1.5) * 1.5,
      lastProcessedSeq: inputEvents[latestSeq]?.seq ?? 0,
      position: positionAt(serverTime),
    };
    snapshots.push(snapshot);
  }
  snapshots.sort((a, b) => a.arrivalMs - b.arrivalMs);

  let visual = { x: 0, z: 0 };
  let latestInput = inputAt(0);
  let arrivalIndex = 0;
  let nextInputIndex = 0;
  const corrections = [];
  let lastProcessedSeq = 0;
  const correctionEvents = [];
  for (let renderTime = 0; renderTime <= durationMs + 0.01; renderTime += renderDt) {
    while (nextInputIndex < inputEvents.length && inputEvents[nextInputIndex].client_time_ms <= renderTime) {
      latestInput = inputEvents[nextInputIndex].input;
      nextInputIndex += 1;
    }
    visual = add(visual, scale(latestInput, speedForIpc * renderDt / 1000));

    while (arrivalIndex < snapshots.length && snapshots[arrivalIndex].arrivalMs <= renderTime) {
      const snapshot = snapshots[arrivalIndex];
      if (snapshot.lastProcessedSeq > lastProcessedSeq) {
        const correction = distance(visual, snapshot.position);
        corrections.push(correction);
        correctionEvents.push({ seq: snapshot.lastProcessedSeq, server_time_ms: snapshot.serverTime, correction_m: correction });
        visual = { ...snapshot.position };
        lastProcessedSeq = snapshot.lastProcessedSeq;
      }
      arrivalIndex += 1;
    }
  }
  return {
    render_hz: renderHz,
    injected_network_delay_ms: injectedDelayMs,
    duration_s: durationMs / 1000,
    input_hz: inputHz,
    snapshot_hz: tickHz,
    input_messages_sent: inputEvents.length,
    render_frames: Math.floor(durationMs / renderDt) + 1,
    corrections_observed: corrections.length,
    max_correction_m: round(Math.max(...corrections), 6),
    p95_correction_m: round(percentile(corrections, 0.95), 6),
    mean_correction_m: round(corrections.reduce((a, b) => a + b, 0) / corrections.length, 6),
    last_processed_seq: lastProcessedSeq,
    correction_sample: correctionEvents.slice(0, 5).map((item) => ({ ...item, correction_m: round(item.correction_m, 6) })),
    model_note: "visual prediction integrates latest input; delayed snapshot snaps to authoritative position; no replay is assumed",
  };
}

function collisionRectHit(position, radius, rect) {
  const closestX = Math.max(rect.minX, Math.min(position.x, rect.maxX));
  const closestZ = Math.max(rect.minZ, Math.min(position.z, rect.maxZ));
  return Math.hypot(position.x - closestX, position.z - closestZ) < radius - 1e-9;
}

function moveCircleWithSlide(start, desired, radius, rects) {
  let position = { ...start };
  const axisMoves = [
    { x: desired.x, z: 0 },
    { x: 0, z: desired.z },
  ];
  for (const move of axisMoves) {
    const length = Math.max(Math.abs(move.x), Math.abs(move.z));
    const steps = Math.max(1, Math.ceil(length / 0.01));
    for (let i = 1; i <= steps; i += 1) {
      const candidate = { x: position.x + move.x / steps, z: position.z + move.z / steps };
      if (rects.some((rect) => collisionRectHit(candidate, radius, rect))) break;
      position = candidate;
    }
  }
  return position;
}

function collisionScenario() {
  const wall = { minX: 3, maxX: 3.2, minZ: -4, maxZ: 4 };
  const radius = 0.45;
  const closed = moveCircleWithSlide({ x: 0, z: 0 }, { x: 6, z: 0 }, radius, [wall]);
  const slide = moveCircleWithSlide({ x: 0, z: 0 }, { x: 6, z: 3 }, radius, [wall]);
  const door = { minX: 2.5, maxX: 3.5, minZ: -1.75, maxZ: 1.75 };
  const doorStates = {};
  for (const state of ["Locked", "Unlocking", "Open"]) {
    const colliders = state === "Open" ? [] : [door];
    const result = moveCircleWithSlide({ x: 0, z: 0 }, { x: 6, z: 0 }, radius, colliders);
    doorStates[state] = { final_x: round(result.x, 4), collider_active: state !== "Open" };
  }

  let y = 0;
  let velocity = 5;
  let landed = false;
  const jumpDt = 1 / 120;
  for (let i = 0; i < 300; i += 1) {
    velocity -= 9.8 * jumpDt;
    y += velocity * jumpDt;
    if (y <= 0 && velocity < 0) {
      y = 0;
      velocity = 0;
      landed = true;
      break;
    }
  }
  const dash = moveCircleWithSlide({ x: 0, z: -2 }, { x: 8, z: 0 }, radius, [wall]);
  const checks = [
    { id: "capsule_wall_stop", pass: closed.x < 3 - radius + 0.02 && closed.x > 2.4 },
    { id: "capsule_slide", pass: slide.x < 3 - radius + 0.02 && slide.z > 2.5 },
    { id: "door_locked_collides", pass: doorStates.Locked.collider_active && doorStates.Locked.final_x < 2.1 },
    { id: "door_unlocking_still_collides", pass: doorStates.Unlocking.collider_active && doorStates.Unlocking.final_x < 2.1 },
    { id: "door_open_releases", pass: !doorStates.Open.collider_active && doorStates.Open.final_x > 5.5 },
    { id: "jump_lands", pass: landed && y === 0 },
    { id: "dash_stops_at_wall", pass: dash.x < 3 - radius + 0.02 },
  ];
  return {
    units: "metres; horizontal X/Z, vertical Y",
    model: "isolated axis-separated circle sweep against AABB; capsule approximated by horizontal radius",
    radius_m: radius,
    wall_result: { stop: { x: round(closed.x, 4), z: round(closed.z, 4) }, slide: { x: round(slide.x, 4), z: round(slide.z, 4) } },
    door_state_result: doorStates,
    jump_result: { landed, final_y_m: y, final_velocity_mps: velocity, dt_ms: jumpDt * 1000 },
    dash_result: { final_x: round(dash.x, 4), final_z: round(dash.z, 4) },
    checks,
    all_checks_pass: checks.every((check) => check.pass),
    limitations: [
      "This does not measure Rapier or the formal Rust collision implementation.",
      "Vertical capsule shape, slopes, moving platforms, camera collision and dynamic multi-body interaction are not modeled.",
    ],
  };
}

const rooms = [
  ["GH_A01", 0, 0, 10, 8], ["GH_A02", 14, 0, 18, 4], ["GH_A03", 32, 0, 16, 14],
  ["GH_GA", 32, 11, 5, 6], ["GH_B01", 32, 27, 24, 22], ["GH_B02", 11, 27, 12, 10],
  ["GH_B03", 56, 27, 20, 15], ["GH_B04", 56, 40, 8, 10], ["GH_GB", 32, 43, 6, 8],
  ["GH_C01", 32, 56, 18, 4], ["GH_C02", 32, 70, 18, 16], ["GH_C03", 49, 70, 10, 8],
].map(([id, x, z, width, depth]) => ({ id, x, z, width, depth, minX: x - width / 2, maxX: x + width / 2, minZ: z - depth / 2, maxZ: z + depth / 2 }));

const doors = [
  ["A01-A02", "GH_A01", "GH_A02", "x", 2.8, "Open"], ["A02-A03", "GH_A02", "GH_A03", "x", 3.2, "Open"],
  ["hive_gate_a", "GH_A03", "GH_GA", "z", 3.5, "Locked"], ["GA-B01", "GH_GA", "GH_B01", "z", 0, "Locked"],
  ["B01-B02", "GH_B01", "GH_B02", "x", 2.8, "Open"], ["B01-B03", "GH_B01", "GH_B03", "x", 3.5, "Open"],
  ["B03-B04", "GH_B03", "GH_B04", "z", 2.6, "Open"], ["hive_gate_b", "GH_B01", "GH_GB", "z", 3.5, "Locked"],
  ["GB-C01", "GH_GB", "GH_C01", "z", 0, "Locked"], ["C01-C02", "GH_C01", "GH_C02", "z", 4, "Open"],
  ["C02-C03", "GH_C02", "GH_C03", "x", 3, "Open"],
].map(([id, a, b, axis, width, initial]) => ({ id, a, b, axis, width, initial }));

function axisGap(a, b, axis) {
  if (axis === "x") return Math.max(0, b.minX - a.maxX, a.minX - b.maxX);
  return Math.max(0, b.minZ - a.maxZ, a.minZ - b.maxZ);
}

function crossOverlap(a, b, axis) {
  if (axis === "x") return Math.min(a.maxZ, b.maxZ) - Math.max(a.minZ, b.minZ);
  return Math.min(a.maxX, b.maxX) - Math.max(a.minX, b.minX);
}

function layoutScenario() {
  const roomById = new Map(rooms.map((room) => [room.id, room]));
  const roomChecks = rooms.map((room) => ({ id: room.id, width_m: room.width, depth_m: room.depth, narrowest_m: Math.min(room.width, room.depth) }));
  const doorAudit = doors.map((door) => {
    const a = roomById.get(door.a);
    const b = roomById.get(door.b);
    return {
      id: door.id,
      connection: `${door.a} -> ${door.b}`,
      initial: door.initial,
      axis: door.axis,
      logical_door_width_m: door.width || null,
      rectangle_gap_m: round(axisGap(a, b, door.axis), 3),
      cross_overlap_m: round(crossOverlap(a, b, door.axis), 3),
      geometry_join_status: axisGap(a, b, door.axis) === 0 ? "touches" : "gap_requires_door_/corridor_geometry",
    };
  });
  const minX = Math.min(...rooms.map((room) => room.minX));
  const maxX = Math.max(...rooms.map((room) => room.maxX));
  const minZ = Math.min(...rooms.map((room) => room.minZ));
  const maxZ = Math.max(...rooms.map((room) => room.maxZ));
  const mainline = ["GH_A01", "GH_A02", "GH_A03", "GH_GA", "GH_B01", "GH_GB", "GH_C01", "GH_C02", "GH_C03"];
  let mainlineDistance = 0;
  const segments = [];
  for (let i = 1; i < mainline.length; i += 1) {
    const a = roomById.get(mainline[i - 1]);
    const b = roomById.get(mainline[i]);
    const distanceM = Math.hypot(a.x - b.x, a.z - b.z);
    mainlineDistance += distanceM;
    segments.push({ from: a.id, to: b.id, center_distance_m: round(distanceM, 3) });
  }
  const graphInitial = {
    upstream_to_downstream_blocked: true,
    gate_a_edge: "hive_gate_a locked; no graph edge from GH_A03 to GH_B01",
    gate_b_edge: "hive_gate_b locked; no graph edge from GH_B01 to GH_GB/GH_C01",
    geometric_bypass_proven: false,
  };
  return {
    room_count: rooms.length,
    rooms: roomChecks,
    doors: doorAudit,
    bounds_m: { min_x: minX, max_x: maxX, min_z: minZ, max_z: maxZ, size_x: maxX - minX, size_z: maxZ - minZ },
    target_bounds_from_plan_m: { x: "60..80", z: "50..70" },
    mainline_center_path: { segments, total_distance_m: round(mainlineDistance, 3), target_m: "180..280" },
    corridor_width_findings: {
      ordinary_minimum_proposed_m: 4,
      ordinary_target_m: ">=2.6",
      battle_target_m: ">=3.5",
      sentinel_arena_min_dimension_m: 16,
      sentinel_arena_target_m: "16..20",
    },
    graph_initial: graphInitial,
    status: "evidence_insufficient_for_layout_freeze",
    limitations: [
      "Rectangles are taken from the proposal table; no rendered blockout or collider bake was changed or loaded.",
      "Door rectangles/door-throat geometry are not specified in the layout, so a touch or gap cannot prove a traversable connection.",
      "Center-to-center mainline distance is a lower-bound proxy, not a measured walkable route.",
    ],
  };
}

function saveSchemaScenario() {
  const requiredV2 = [
    "version", "mode", "title", "subtitle", "player", "worlds", "upgrades", "skills", "objectives", "dialog", "result", "last_event", "effect", "map", "decisions", "current_world_id", "flags", "spoils_claimed", "run_reward_allowed", "run_scrap", "enemies_defeated", "run_ticks", "run_damage_taken", "optional_choice", "shield", "attack_cooldown", "steam_phase", "spawn_protected",
  ];
  const mapV2 = ["world_id", "world_name", "w", "h", "floor", "tiles", "px", "py", "revision", "explored", "enemies", "npcs", "points", "zones", "portals", "gates", "props", "nearby"];
  const proposedV3 = ["schema_version", "world_id", "player.transform", "player.velocity", "player.grounded", "player.motor_state", "entities", "doors", "room_states", "explored_rooms", "discovered_pois", "objective_progress", "checkpoint_id", "last_processed_input_seq"];
  return {
    real_save_accessed: false,
    current_v2: {
      source: "server-rs/src/campaign_store.rs:318-348, 364-430, 768-877",
      version_required: 2,
      title_identifiers_accepted: ["无限人族进化", "余界：归航协议"],
      valid_world_ids: ["grey_hive", "mist_harbor", "clockworks"],
      required_root_fields: requiredV2,
      map_required_fields: mapV2,
      position_representation: "integer map.px/map.py validated against ASCII tiles",
      recovery_boundary: "invalid read falls back to default in memory and reports warning; writes are paused until explicit new",
    },
    proposed_v3: {
      status: "proposal_only_not_implemented",
      required_fields: proposedV3,
      migration_rule: "semantic checkpoint mapping by world/objective/gate stage; never linear-scale px/py",
      checkpoint_candidates: { grey_hive: ["gh_airlock_spawn", "gh_power_checkpoint", "gh_central_checkpoint", "gh_rescue_checkpoint"] },
      invalid_transform_recovery_order: ["nearest legal navigation point", "current room checkpoint", "world spawn"],
      write_policy_to_decide: ["file name and title marker", "whether formal v2 compatibility is promised", "backup retention and pending/bak recovery contract"],
    },
    status: "evidence_insufficient_for_v3_decision",
    limitations: ["No private or real save file was opened, copied, modified or generated.", "A schema audit cannot prove migration idempotence or recovery behavior without a disposable fixture and an approved v3 contract."],
  };
}

function resultFor(scenario) {
  if (scenario === "tick") return { scenario, environment: environment(), runs: [tickRateProbe(30), tickRateProbe(60)] };
  if (scenario === "ipc") {
    const cases = [];
    for (const renderHz of [60, 120]) for (const delay of [0, 16, 50, 120]) cases.push(ipcCase(renderHz, delay));
    return { scenario, environment: environment(), cases };
  }
  if (scenario === "collision") return { scenario, environment: environment(), ...collisionScenario() };
  if (scenario === "layout") return { scenario, environment: environment(), ...layoutScenario() };
  if (scenario === "save-schema") return { scenario, environment: environment(), ...saveSchemaScenario() };
  throw new Error(`unknown scenario: ${scenario}`);
}

const args = parseArgs(process.argv.slice(2));
if (!args.scenario || !args.out) {
  console.error("Usage: node run.mjs --scenario <tick|ipc|collision|layout|save-schema> --out <json>");
  process.exitCode = 2;
} else {
  const output = path.resolve(process.cwd(), String(args.out));
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, `${JSON.stringify(resultFor(String(args.scenario)), null, 2)}\n`, "utf8");
  console.log(`AG-01 ${args.scenario}: ${output}`);
}
