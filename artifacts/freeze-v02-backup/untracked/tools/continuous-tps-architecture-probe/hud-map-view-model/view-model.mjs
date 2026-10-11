const PROTOCOL = "continuous-ipc";
const WORLD_VERSION = 1;
const CAPABILITY_PROTOCOL = "continuous-capability";
const CAPABILITY_VERSION = 1;
const DEFAULTS = Object.freeze({
  fontScale: 1,
  contrast: "normal",
  colorBlindMode: "none",
  subtitles: true,
  keybinds: Object.freeze({ interact: "E", map: "M", rearView: "V" }),
});
const SOURCE_PRIORITY = Object.freeze({
  system: 100,
  progression: 80,
  equipment: 60,
  bloodline: 50,
  world: 40,
  temporary: 20,
});

function diagnostic(code, detail = null) {
  return detail === null ? { code } : { code, detail };
}

function finite(value) {
  return typeof value === "number" && Number.isFinite(value);
}

function integer(value) {
  return Number.isInteger(value) && value >= 0;
}

function object(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function boundedString(value, fallback = "", max = 512) {
  return typeof value === "string" && value.length <= max ? value : fallback;
}

function clone(value) {
  return value === undefined ? undefined : structuredClone(value);
}

function deepFreeze(value, seen = new WeakSet()) {
  if (!value || typeof value !== "object" || seen.has(value)) return value;
  seen.add(value);
  for (const child of Object.values(value)) deepFreeze(child, seen);
  return Object.freeze(value);
}

function immutable(value) {
  return deepFreeze(clone(value));
}

function emptyVm(overrides = {}) {
  return immutable({
    schema: "hud_map_view_model_tp15a1_v1",
    immutable: true,
    connection: {
      status: "unavailable",
      accepted: false,
      worldEpoch: null,
      tick: null,
      ackSeq: null,
      correctionFlag: false,
      sampleMode: "none",
      diagnostic: diagnostic("world_snapshot_unavailable"),
    },
    hud: {
      hp: null,
      maxHp: null,
      energy: null,
      maxEnergy: null,
      scrap: null,
      interactionPrompt: null,
      subtitle: null,
      textFallback: true,
    },
    objective: {
      current: null,
      list: [],
      checkpoint: null,
      result: null,
      gatePrompt: null,
    },
    map: {
      status: "unavailable",
      candidateOnly: false,
      room: null,
      doors: [],
      markers: [],
      diagnostic: diagnostic("formal_scene_unavailable"),
      textFallback: true,
    },
    capabilities: {
      availability: {},
      active: {},
      errors: [diagnostic("capability_view_missing")],
    },
    accessibility: {
      subtitles: DEFAULTS.subtitles,
      fontScale: DEFAULTS.fontScale,
      colorBlindMode: DEFAULTS.colorBlindMode,
      contrast: DEFAULTS.contrast,
      keybinds: clone(DEFAULTS.keybinds),
    },
    player: {
      positionM: null,
      yawRad: null,
      velocityMps: null,
      renderMode: "none",
      correctionFlag: false,
    },
    diagnostics: [diagnostic("world_snapshot_unavailable")],
    ...overrides,
  });
}

function numberOrNull(value) {
  return finite(value) ? value : null;
}

function sanitizeVec(value, names) {
  if (!object(value)) return null;
  const output = {};
  for (const name of names) {
    if (!finite(value[name])) return null;
    output[name] = value[name];
  }
  return output;
}

function sanitizeWorld(worldView) {
  if (!object(worldView)) return { ok: false, diagnostics: [diagnostic("world_snapshot_missing")] };
  if (worldView.protocol !== PROTOCOL || worldView.version !== WORLD_VERSION) {
    return { ok: false, diagnostics: [diagnostic("world_schema_mismatch")] };
  }
  if (!integer(worldView.worldEpoch) || !integer(worldView.serverTick) || !integer(worldView.ackSeq)) {
    return { ok: false, diagnostics: [diagnostic("world_clock_invalid")] };
  }
  if (typeof worldView.worldId !== "string" || worldView.worldId.length === 0 || worldView.worldId.length > 128) {
    return { ok: false, diagnostics: [diagnostic("world_id_invalid")] };
  }
  const player = object(worldView.player) ? worldView.player : {};
  const transform = object(player.transform) ? player.transform : {};
  const velocity = object(player.velocityMps) ? player.velocityMps : {};
  const numericGroups = [
    [transform.positionM, ["xM", "yM", "zM"]],
    [velocity, ["xM", "yM", "zM"]],
    [transform, ["yawRad"]],
    [player, ["hp", "maxHp", "energy", "maxEnergy", "scrap"]],
    [worldView, ["serverTimeMs"]],
  ];
  if (numericGroups.some(([value, names]) => object(value) && names.some((name) => value[name] !== undefined && value[name] !== null && !finite(value[name])))) {
    return { ok: false, diagnostics: [diagnostic("world_non_finite")] };
  }
  const position = sanitizeVec(transform.positionM, ["xM", "yM", "zM"]);
  const velocityMps = sanitizeVec(velocity, ["xM", "yM", "zM"]);
  const yawRad = finite(transform.yawRad) ? transform.yawRad : null;
  const diagnostics = [];
  if (!position || !velocityMps || yawRad === null) diagnostics.push(diagnostic("player_transform_unavailable"));
  const arrays = ["actors", "doors", "objectives"];
  for (const key of arrays) {
    if (worldView[key] !== undefined && !Array.isArray(worldView[key])) diagnostics.push(diagnostic(`world_${key}_invalid`));
    if (Array.isArray(worldView[key]) && worldView[key].length > 256) return { ok: false, diagnostics: [diagnostic("input_oversize", key)] };
  }
  const hp = object(player) ? player : {};
  return {
    ok: true,
    value: {
      worldId: worldView.worldId,
      worldEpoch: worldView.worldEpoch,
      tick: worldView.serverTick,
      ackSeq: worldView.ackSeq,
      serverTimeMs: finite(worldView.serverTimeMs) ? worldView.serverTimeMs : worldView.serverTick * (1000 / 60),
      player: {
        positionM: position,
        yawRad,
        velocityMps,
        hp: numberOrNull(hp.hp),
        maxHp: numberOrNull(hp.maxHp),
        energy: numberOrNull(hp.energy),
        maxEnergy: numberOrNull(hp.maxEnergy),
        scrap: numberOrNull(hp.scrap),
      },
      interactionPrompt: boundedString(worldView.interactionPrompt, null),
      subtitle: boundedString(worldView.subtitle, null, 2048),
      objective: object(worldView.objective)
        ? { id: boundedString(worldView.objective.id, "unknown", 96), title: boundedString(worldView.objective.title, "当前目标", 256), done: worldView.objective.done === true }
        : null,
      objectives: Array.isArray(worldView.objectives)
        ? worldView.objectives.slice(0, 256).filter((item) => object(item)).map((item) => ({
          id: boundedString(item.id, "unknown", 96),
          title: boundedString(item.title, boundedString(item.id, "未命名目标", 96), 256),
          done: item.done === true,
          optional: item.optional === true,
        }))
        : [],
      checkpoint: boundedString(worldView.checkpoint ?? null, null),
      choiceState: boundedString(worldView.choiceState ?? null, null, 32),
      result: object(worldView.result)
        ? { title: boundedString(worldView.result.title, "", 256), summary: boundedString(worldView.result.summary, "", 2048), rank: boundedString(worldView.result.rank, "", 32), map: worldView.result.map === null ? null : undefined }
        : null,
      doors: sanitizeEntries(worldView.doors, "door"),
    },
    diagnostics,
  };
}

function sanitizePreferences(preferences) {
  const input = object(preferences) ? preferences : {};
  const fontScale = finite(input.fontScale) ? Math.min(2, Math.max(0.75, input.fontScale)) : DEFAULTS.fontScale;
  const keybinds = object(input.keybinds) ? input.keybinds : {};
  return {
    subtitles: input.subtitles !== false,
    fontScale,
    colorBlindMode: boundedString(input.colorBlindMode, DEFAULTS.colorBlindMode, 32),
    contrast: boundedString(input.contrast, DEFAULTS.contrast, 32),
    keybinds: {
      interact: boundedString(keybinds.interact, DEFAULTS.keybinds.interact, 16),
      map: boundedString(keybinds.map, DEFAULTS.keybinds.map, 16),
      rearView: boundedString(keybinds.rearView, DEFAULTS.keybinds.rearView, 16),
    },
  };
}

function sanitizeCapability(capabilityView, world) {
  const errors = [];
  if (!object(capabilityView)) return { effective: [], errors: [diagnostic("capability_view_missing")] };
  if (capabilityView.protocol !== CAPABILITY_PROTOCOL || capabilityView.version !== CAPABILITY_VERSION) {
    return { effective: [], errors: [diagnostic("capability_schema_mismatch")] };
  }
  if (capabilityView.worldId !== world.worldId || capabilityView.worldEpoch !== world.worldEpoch) {
    return { effective: [], errors: [diagnostic("capability_stale_epoch")] };
  }
  if (!integer(capabilityView.tick)) return { effective: [], errors: [diagnostic("capability_tick_invalid")] };
  if (capabilityView.tick > world.tick) return { effective: [], errors: [diagnostic("capability_tick_ahead")] };
  if (!Array.isArray(capabilityView.effective)) return { effective: [], errors: [...errors, diagnostic("capability_effective_invalid")] };
  if (capabilityView.effective.length > 128) return { effective: [], errors: [diagnostic("input_oversize", "capabilities")] };
  const effective = [];
  for (const grant of capabilityView.effective) {
    if (!object(grant) || typeof grant.capabilityId !== "string" || typeof grant.grantId !== "string") {
      errors.push(diagnostic("capability_grant_invalid"));
      continue;
    }
    if (grant.capabilityId === "perception.rear_view" && grant.worldEpoch !== undefined && grant.worldEpoch !== world.worldEpoch) {
      errors.push(diagnostic("rear_view_stale_epoch"));
      continue;
    }
    effective.push({
      capabilityId: boundedString(grant.capabilityId, "", 96),
      grantId: boundedString(grant.grantId, "", 128),
      source: boundedString(grant.source, "", 32),
      tier: boundedString(grant.tier, "", 32),
      scope: boundedString(grant.scope, "", 32),
      worldId: boundedString(grant.worldId, world.worldId, 128),
      worldEpoch: grant.worldEpoch === undefined ? world.worldEpoch : grant.worldEpoch,
      parameters: object(grant.parameters) ? {
        distanceM: finite(grant.parameters.distanceM) ? grant.parameters.distanceM : undefined,
        yawFovDeg: finite(grant.parameters.yawFovDeg) ? grant.parameters.yawFovDeg : undefined,
        resolutionTier: boundedString(grant.parameters.resolutionTier, "", 32),
      } : {},
    });
  }
  effective.sort((a, b) => {
    const priority = (SOURCE_PRIORITY[b.source] ?? 0) - (SOURCE_PRIORITY[a.source] ?? 0);
    return priority || String(a.tier ?? "").localeCompare(String(b.tier ?? "")) || a.grantId.localeCompare(b.grantId);
  });
  return { effective, errors };
}

function capabilityModel(capabilityView, world) {
  const sanitized = sanitizeCapability(capabilityView, world);
  const availability = {};
  const active = {};
  for (const grant of sanitized.effective) {
    if (!availability[grant.capabilityId]) availability[grant.capabilityId] = true;
    if (!active[grant.capabilityId]) active[grant.capabilityId] = clone(grant);
  }
  for (const id of ["perception.rear_view", "map.local", "mobility.dash"]) {
    if (availability[id] === undefined) availability[id] = false;
    if (active[id] === undefined) active[id] = null;
  }
  return { availability, active, errors: sanitized.errors };
}

function objectiveModel(world) {
  const objectives = world.objectives.filter((item) => object(item)).map((item) => ({
    id: boundedString(item.id, "unknown", 96),
    title: boundedString(item.title, boundedString(item.id, "未命名目标", 96), 256),
    done: item.done === true,
    optional: item.optional === true,
  }));
  const current = world.objective && object(world.objective)
    ? { id: boundedString(world.objective.id, "unknown", 96), title: boundedString(world.objective.title, "当前目标", 256), done: world.objective.done === true }
    : objectives.find((item) => !item.done && !item.optional) ?? objectives.find((item) => !item.done) ?? null;
  let gatePrompt = null;
  const power = objectives.find((item) => item.id === "hive_power");
  const lockdown = objectives.find((item) => item.id === "hive_lockdown");
  if (power && !power.done) gatePrompt = { gate: "A", condition: "hive_power", text: "恢复主电以解锁 A 区" };
  else if (lockdown && !lockdown.done) gatePrompt = { gate: "B", condition: "hive_lockdown", text: "解除隔离以解锁 B 区" };
  return { current, list: objectives, checkpoint: world.checkpoint, result: world.result, choiceState: world.choiceState, gatePrompt };
}

function sanitizeEntries(entries, kind) {
  if (!Array.isArray(entries)) return [];
  return entries.slice(0, 256).filter((entry) => object(entry)).map((entry) => {
    const output = { id: boundedString(entry.id, `${kind}-unknown`, 96), label: boundedString(entry.label ?? entry.name, "", 256) };
    for (const field of ["xM", "yM", "zM", "x", "y", "z"]) if (finite(entry[field])) output[field] = entry[field];
    if (entry.open !== undefined) output.open = entry.open === true;
    if (entry.kind !== undefined) output.kind = boundedString(entry.kind, "", 64);
    return output;
  });
}

function mapModel(scene, world) {
  if (world.result && world.result.map === null) return { status: "unavailable", candidateOnly: false, room: null, doors: [], markers: [], diagnostic: diagnostic("terminal_result_no_map"), textFallback: true };
  if (!object(scene)) return { status: "unavailable", candidateOnly: false, room: null, doors: [], markers: [], diagnostic: diagnostic("scene_metadata_missing"), textFallback: true };
  if (scene.worldId !== undefined && scene.worldId !== world.worldId) return { status: "unavailable", candidateOnly: false, room: null, doors: [], markers: [], diagnostic: diagnostic("scene_world_mismatch"), textFallback: true };
  if (scene.expectedSceneHash !== undefined && scene.sceneHash !== scene.expectedSceneHash) return { status: "unavailable", candidateOnly: false, room: null, doors: [], markers: [], diagnostic: diagnostic("scene_hash_mismatch"), textFallback: true };
  if (scene.expectedLayoutVersion !== undefined && scene.layoutVersion !== scene.expectedLayoutVersion) return { status: "unavailable", candidateOnly: false, room: null, doors: [], markers: [], diagnostic: diagnostic("scene_layout_mismatch"), textFallback: true };
  const candidateOnly = scene.candidateOnly === true || scene.formalSceneReady !== true;
  const status = candidateOnly ? "candidate" : "available";
  const diagnosticCode = scene.formalSceneReady === true ? null : "candidate_scene_not_formal";
  return {
    status,
    candidateOnly,
    room: boundedString(scene.room ?? scene.roomId, null, 96),
    doors: sanitizeEntries(scene.doors, "door"),
    markers: sanitizeEntries(scene.markers, "marker"),
    diagnostic: diagnosticCode ? diagnostic(diagnosticCode) : null,
    textFallback: scene.assetAvailable === false,
  };
}

function renderPlayer(state, nowMs, policy) {
  if (!state.latest || !state.latest.world.player.positionM) return { positionM: null, yawRad: null, velocityMps: null, renderMode: "none", correctionFlag: false };
  const latest = state.latest;
  const previous = state.previous;
  if (!previous || previous === latest || !finite(nowMs)) return { ...clone(latest.world.player), renderMode: "snap", correctionFlag: false };
  const duration = latest.receivedAtMs - previous.receivedAtMs;
  const alpha = duration > 0 ? Math.min(1, Math.max(0, (nowMs - previous.receivedAtMs) / duration)) : 1;
  let renderMode = "interpolation";
  let position = {
    xM: previous.world.player.positionM.xM + (latest.world.player.positionM.xM - previous.world.player.positionM.xM) * alpha,
    yM: previous.world.player.positionM.yM + (latest.world.player.positionM.yM - previous.world.player.positionM.yM) * alpha,
    zM: previous.world.player.positionM.zM + (latest.world.player.positionM.zM - previous.world.player.positionM.zM) * alpha,
  };
  if (nowMs > latest.receivedAtMs && policy.allowExtrapolation === true) {
    const dtMs = Math.min(nowMs - latest.receivedAtMs, policy.horizonMs);
    const predicted = {
      xM: latest.world.player.positionM.xM + latest.world.player.velocityMps.xM * dtMs / 1000,
      yM: latest.world.player.positionM.yM + latest.world.player.velocityMps.yM * dtMs / 1000,
      zM: latest.world.player.positionM.zM + latest.world.player.velocityMps.zM * dtMs / 1000,
    };
    const correction = Math.hypot(predicted.xM - latest.world.player.positionM.xM, predicted.yM - latest.world.player.positionM.yM, predicted.zM - latest.world.player.positionM.zM);
    if (correction <= policy.correctionMeters) {
      position = predicted;
      renderMode = "extrapolation";
      return { ...clone(latest.world.player), positionM: position, renderMode, correctionFlag: false, extrapolationMs: dtMs, correctionMeters: correction };
    }
    return { ...clone(latest.world.player), renderMode: "correction", correctionFlag: true, correctionMeters: correction };
  }
  return { ...clone(latest.world.player), positionM: position, renderMode, correctionFlag: false, interpolationAlpha: alpha };
}

function viewModel(state, status, diagnosticCode, nowMs = null) {
  const world = state.latest?.world;
  if (!world) return emptyVm({ connection: { ...emptyVm().connection, status, diagnostic: diagnostic(diagnosticCode) } });
  const prefs = state.preferences;
  const cap = capabilityModel(state.latest.capabilityView, world);
  const map = mapModel(state.latest.scene, world);
  const objectives = objectiveModel(world);
  const player = nowMs === null ? renderPlayer(state, world.serverTimeMs, state.policy) : renderPlayer(state, nowMs, state.policy);
  const diagnostics = [...state.latest.diagnostics, ...(map.diagnostic ? [map.diagnostic] : []), ...cap.errors];
  return immutable({
    schema: "hud_map_view_model_tp15a1_v1",
    immutable: true,
    connection: {
      status,
      accepted: status === "accepted" || status === "world_replaced",
      worldEpoch: world.worldEpoch,
      tick: world.tick,
      ackSeq: world.ackSeq,
      correctionFlag: player.correctionFlag,
      sampleMode: player.renderMode,
      diagnostic: diagnosticCode ? diagnostic(diagnosticCode) : null,
    },
    hud: {
      hp: world.player.hp,
      maxHp: world.player.maxHp,
      energy: world.player.energy,
      maxEnergy: world.player.maxEnergy,
      scrap: world.player.scrap,
      interactionPrompt: world.interactionPrompt,
      subtitle: prefs.subtitles ? world.subtitle : null,
      textFallback: true,
    },
    objective: objectives,
    map,
    capabilities: cap,
    accessibility: clone(prefs),
    player,
    diagnostics,
  });
}

export function createViewModelAdapter(options = {}) {
  const policy = {
    allowExtrapolation: options.allowExtrapolation === true,
    horizonMs: finite(options.horizonMs) && options.horizonMs >= 0 ? Math.min(options.horizonMs, 50) : 50,
    correctionMeters: finite(options.correctionMeters) && options.correctionMeters >= 0 ? Math.min(options.correctionMeters, 0.25) : 0.25,
  };
  const state = { latest: null, previous: null, preferences: sanitizePreferences(null), policy, capabilityEpoch: null, capabilityTick: null, capabilityView: null };

  function accept(input) {
    if (!object(input)) return emptyVm({ connection: { ...emptyVm().connection, status: "rejected", diagnostic: diagnostic("input_schema_invalid") } });
    state.preferences = sanitizePreferences(input.uiPreferences);
    const worldResult = sanitizeWorld(input.worldView);
    if (!worldResult.ok) return state.latest ? viewModel(state, "rejected", worldResult.diagnostics[0].code) : emptyVm({ accessibility: state.preferences, connection: { ...emptyVm().connection, status: "rejected", diagnostic: worldResult.diagnostics[0] } });
    const world = worldResult.value;
    const receivedAtMs = finite(input.receivedAtMs) ? input.receivedAtMs : world.serverTimeMs;
    if (!finite(receivedAtMs) || receivedAtMs < 0) return state.latest ? viewModel(state, "rejected", "received_time_invalid") : emptyVm({ accessibility: state.preferences, connection: { ...emptyVm().connection, status: "rejected", diagnostic: diagnostic("received_time_invalid") } });
    if (state.latest && world.worldEpoch < state.latest.world.worldEpoch) return viewModel(state, "rejected", "stale_epoch");
    if (state.latest && world.worldEpoch === state.latest.world.worldEpoch && world.ackSeq < state.latest.world.ackSeq) return viewModel(state, "rejected", "ack_regression");
    if (state.latest && world.worldEpoch === state.latest.world.worldEpoch && world.tick <= state.latest.world.tick) return viewModel(state, "rejected", "duplicate_or_out_of_order_tick");
    const replaced = state.latest && world.worldEpoch > state.latest.world.worldEpoch;
    let capabilityView = input.capabilityView;
    const candidateCapabilityTick = object(capabilityView) && integer(capabilityView.tick) ? capabilityView.tick : null;
    const candidateCapabilityEpoch = object(capabilityView) && integer(capabilityView.worldEpoch) ? capabilityView.worldEpoch : null;
    const capabilityStale = !replaced
      && candidateCapabilityTick !== null
      && candidateCapabilityEpoch === state.capabilityEpoch
      && state.capabilityTick !== null
      && candidateCapabilityTick < state.capabilityTick;
    if (replaced) {
      state.capabilityEpoch = world.worldEpoch;
      state.capabilityTick = null;
      state.capabilityView = null;
    } else if (capabilityStale) {
      capabilityView = state.capabilityView;
    } else if (candidateCapabilityTick !== null && candidateCapabilityEpoch === world.worldEpoch) {
      state.capabilityEpoch = world.worldEpoch;
      state.capabilityTick = candidateCapabilityTick;
      state.capabilityView = capabilityView;
    }
    state.previous = replaced ? null : state.latest;
    state.latest = { world, capabilityView, scene: input.sceneMetadata, diagnostics: [...worldResult.diagnostics, ...(capabilityStale ? [diagnostic("capability_stale_tick")] : [])], receivedAtMs };
    return viewModel(state, replaced ? "world_replaced" : "accepted", replaced ? "world_replaced" : null, receivedAtMs);
  }

  function render(nowMs) {
    if (!state.latest) return emptyVm({ accessibility: state.preferences, connection: { ...emptyVm().connection, status: "unavailable", diagnostic: diagnostic("no_snapshot") } });
    return viewModel(state, "accepted", null, nowMs);
  }

  return Object.freeze({ accept, render });
}

export function hasUnsafeOutput(value) {
  const seen = new WeakSet();
  function visit(node) {
    if (typeof node === "function" || typeof node === "bigint" || typeof node === "symbol") return true;
    if (!node || typeof node !== "object") return false;
    if (seen.has(node)) return false;
    seen.add(node);
    return Object.entries(node).some(([key, child]) => /path|bytes|dom|canvas|webgl|gpu|handle/i.test(key) || visit(child));
  }
  return visit(value);
}

export const __test = Object.freeze({ sanitizeWorld, mapModel, objectiveModel, sanitizePreferences });
