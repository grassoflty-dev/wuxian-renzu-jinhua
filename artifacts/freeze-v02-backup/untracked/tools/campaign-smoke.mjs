#!/usr/bin/env node

/**
 * Long-lived smoke/acceptance runner for the campaign probe.
 *
 * The probe is the only game-facing boundary used here.  The runner never
 * reconstructs CampaignState, writes a save, or relies on private Rust fields.
 * Every route is recomputed from the public map view.
 */

import { spawn } from 'node:child_process';
import readline from 'node:readline';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(SCRIPT_DIR, '..');
const PROBE_NAME = process.platform === 'win32' ? 'campaign-probe.exe' : 'campaign-probe';
const DEFAULT_PROBE = path.join(REPO_ROOT, 'server-rs', 'target', 'debug', PROBE_NAME);

const WORLD_ORDER = ['grey_hive', 'mist_harbor', 'clockworks'];
const RESPONSE_TIMEOUT_MS = 5000;
const CLOSE_TIMEOUT_MS = 300;
const MAX_ACTIONS_PER_WORLD = 5000;
const MAX_ROUTE_STEPS = 6000;
const MAX_COMBAT_ROUNDS = 160;
const MAX_DEATH_TICKS = 120;

const CARDINAL = [
  { dx: 1, dy: 0 },
  { dx: -1, dy: 0 },
  { dx: 0, dy: 1 },
  { dx: 0, dy: -1 },
];
const EIGHT_DIRECTIONS = [
  { dx: 1, dy: 0 },
  { dx: -1, dy: 0 },
  { dx: 0, dy: 1 },
  { dx: 0, dy: -1 },
  { dx: 1, dy: 1 },
  { dx: 1, dy: -1 },
  { dx: -1, dy: 1 },
  { dx: -1, dy: -1 },
];

function isRecord(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function cloneJson(value) {
  return JSON.parse(JSON.stringify(value));
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function parseArgs(argv) {
  let probe = DEFAULT_PROBE;
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--help' || arg === '-h') {
      return { help: true };
    }
    if (arg === '--probe') {
      if (index + 1 >= argv.length) throw new Error('--probe needs a path');
      probe = argv[++index];
      continue;
    }
    if (arg.startsWith('--probe=')) {
      probe = arg.slice('--probe='.length);
      if (probe.length === 0) throw new Error('--probe needs a path');
      continue;
    }
    throw new Error(`unknown argument: ${arg}`);
  }
  return { help: false, probe: path.resolve(process.cwd(), probe) };
}

class SmokeError extends Error {
  constructor(message, target = null, details = null) {
    super(message);
    this.name = 'CampaignSmokeError';
    this.target = target;
    this.details = details;
  }
}

class ProbeClient {
  constructor(executable) {
    this.executable = executable;
    this.child = spawn(executable, [], {
      cwd: REPO_ROOT,
      windowsHide: true,
      // stderr is deliberately not collected or included in reports.
      stdio: ['pipe', 'pipe', 'ignore'],
    });
    this.pending = null;
    this.queue = Promise.resolve();
    this.failure = null;
    this.closing = false;
    this.closed = false;

    this.reader = readline.createInterface({
      input: this.child.stdout,
      crlfDelay: Infinity,
    });
    this.reader.on('line', (line) => this.onLine(line));
    this.child.once('error', (error) => {
      this.fail(new Error(`probe process error: ${error.message}`));
    });
    this.child.once('exit', (code, signal) => {
      if (this.closing) return;
      const suffix = signal ? `signal=${signal}` : `code=${code}`;
      this.fail(new Error(`probe exited before response (${suffix})`));
    });
  }

  fail(error) {
    if (!this.failure) this.failure = error;
    if (this.pending) {
      const pending = this.pending;
      this.pending = null;
      clearTimeout(pending.timer);
      pending.reject(this.failure);
    }
  }

  onLine(line) {
    if (line.trim() === '') {
      this.fail(new Error('probe emitted an empty NDJSON line'));
      return;
    }
    if (!this.pending) {
      this.fail(new Error('probe emitted an unsolicited NDJSON response'));
      return;
    }
    const pending = this.pending;
    this.pending = null;
    clearTimeout(pending.timer);
    try {
      pending.resolve(JSON.parse(line));
    } catch (error) {
      pending.reject(new Error(`probe emitted invalid JSON: ${error.message}`));
    }
  }

  request(action, args = {}) {
    const run = () => this.requestOne(action, args);
    const next = this.queue.then(run, run);
    this.queue = next.catch(() => undefined);
    return next;
  }

  requestOne(action, args) {
    if (this.failure) return Promise.reject(this.failure);
    if (this.closed) return Promise.reject(new Error('probe client is closed'));
    const line = `${JSON.stringify({ action, args })}\n`;
    return new Promise((resolve, reject) => {
      let settled = false;
      const finish = (callback, value) => {
        if (settled) return;
        settled = true;
        if (this.pending?.finish === finish) this.pending = null;
        clearTimeout(timer);
        callback(value);
      };
      const timer = setTimeout(() => {
        const error = new Error(`probe response timeout after ${RESPONSE_TIMEOUT_MS}ms`);
        finish(reject, error);
        this.fail(error);
        try {
          this.child.kill();
        } catch {
          // The request already has a diagnostic failure; cleanup is best effort.
        }
      }, RESPONSE_TIMEOUT_MS);
      this.pending = { resolve: (value) => finish(resolve, value), reject: (error) => finish(reject, error), finish, timer };
      try {
        this.child.stdin.write(line, (error) => {
          if (error) finish(reject, error);
        });
      } catch (error) {
        finish(reject, error);
      }
    });
  }

  async close() {
    if (this.closed) return;
    this.closing = true;
    this.closed = true;
    this.reader.close();
    if (this.child.exitCode !== null || this.child.signalCode !== null) return;
    const closed = new Promise((resolve) => this.child.once('close', resolve));
    try {
      this.child.stdin.end();
    } catch {
      // The child may already have exited.
    }
    await Promise.race([closed, sleep(CLOSE_TIMEOUT_MS)]);
    if (this.child.exitCode === null && this.child.signalCode === null) {
      try {
        this.child.kill();
      } catch {
        // Best-effort process cleanup only.
      }
    }
  }
}

let probeClient = null;
let state = null;
let lastAction = null;
let activeTarget = null;
let activeRun = null;
let movesSinceTick = 0;

function diagnostics(target = activeTarget, extra = null) {
  const map = state?.map;
  const player = state?.player;
  return {
    mode: state?.mode ?? null,
    last_event: state?.last_event ?? null,
    player: player
      ? {
          hp: player.hp,
          max_hp: player.max_hp,
          energy: player.energy,
          max_energy: player.max_energy,
          medkits: player.medkits,
          reward_points: player.reward_points,
          side_story_tokens: player.side_story_tokens,
          build: player.build,
        }
      : null,
    position: map ? { x: map.px, y: map.py, world_id: map.world_id } : null,
    target: target ?? null,
    last_action: lastAction,
    run: activeRun
      ? {
          id: activeRun.id,
          kind: activeRun.kind,
          actions: activeRun.actions,
          ticks: activeRun.ticks,
          combat: activeRun.combat,
        }
      : null,
    enemies: map?.enemies?.map((enemy) => ({
      id: enemy.id,
      x: enemy.x,
      y: enemy.y,
      hp: enemy.hp,
      alive: enemy.alive,
      telegraph: enemy.telegraph,
    })) ?? null,
    ...(extra ? { detail: extra } : {}),
  };
}

function fail(message, target = activeTarget, extra = null) {
  throw new SmokeError(message, target, diagnostics(target, extra));
}

function expect(condition, message, target = activeTarget, extra = null) {
  if (!condition) fail(message, target, extra);
}

function validateView(value) {
  expect(isRecord(value), 'probe response is not a JSON object');
  expect(typeof value.mode === 'string', 'probe response has no public mode');
  expect(typeof value.last_event === 'string', 'probe response has no public last_event');
  expect(isRecord(value.player), 'probe response has no public player view');
  expect(Array.isArray(value.worlds), 'probe response has no public worlds view');
  return value;
}

function publicMutationFingerprint(value) {
  const copy = cloneJson(value);
  copy.last_event = '__ignored_last_event__';
  return JSON.stringify(copy);
}

function assertPublicUnchanged(before, after, label, target = activeTarget) {
  expect(
    publicMutationFingerprint(before) === publicMutationFingerprint(after),
    `${label} changed public state other than last_event`,
    target,
    { before: diagnosticsFromView(before), after: diagnosticsFromView(after) },
  );
}

function diagnosticsFromView(view) {
  return {
    mode: view?.mode ?? null,
    last_event: view?.last_event ?? null,
    player: view?.player ?? null,
    position: view?.map ? { x: view.map.px, y: view.map.py, world_id: view.map.world_id } : null,
  };
}

async function act(action, args = {}, target = activeTarget) {
  if (activeRun && activeRun.actions >= MAX_ACTIONS_PER_WORLD) {
    fail(`action budget exceeded (${MAX_ACTIONS_PER_WORLD})`, target);
  }
  lastAction = { action, args: cloneJson(args) };
  if (activeRun) {
    activeRun.actions += 1;
    if (action === 'tick') activeRun.ticks += 1;
  }
  try {
    const response = validateView(await probeClient.request(action, args));
    state = response;
    return response;
  } catch (error) {
    if (error instanceof SmokeError) throw error;
    throw new SmokeError(`probe action ${action} failed: ${error.message}`, target, diagnostics(target));
  }
}

function beginRun(id, kind = 'world') {
  activeRun = {
    id,
    kind,
    actions: 0,
    ticks: 0,
    combat: {
      attacks: 0,
      skillUses: 0,
      pulseUses: 0,
      pierceUses: 0,
      guardUses: 0,
    },
    optional_replay_skipped: false,
    decision: null,
  };
  movesSinceTick = 0;
  activeTarget = { world_id: id, stage: kind === 'world' ? 'enter' : kind };
}

function getMap(target = activeTarget) {
  expect(isRecord(state?.map), 'expected an in-world public map', target);
  const map = state.map;
  expect(Number.isInteger(map.w) && Number.isInteger(map.h), 'map dimensions are invalid', target);
  expect(Array.isArray(map.tiles) && map.tiles.length === map.h, 'map tiles are incomplete', target);
  for (const row of map.tiles) {
    expect(typeof row === 'string' && row.length === map.w, 'map tile row has the wrong width', target);
  }
  return map;
}

function positionOfObject(map, id) {
  const collections = [map.points, map.npcs, map.gates, map.portals];
  for (const collection of collections) {
    const object = collection?.find((item) => item?.id === id);
    if (object) return { x: object.x, y: object.y, object };
  }
  return null;
}

function allMapObjects(map) {
  return [
    ...(map.points ?? []),
    ...(map.npcs ?? []),
    ...(map.gates ?? []),
    ...(map.portals ?? []),
  ].filter((item) => typeof item?.id === 'string' && Number.isInteger(item.x) && Number.isInteger(item.y));
}

function distanceSq(ax, ay, bx, by) {
  const dx = ax - bx;
  const dy = ay - by;
  return dx * dx + dy * dy;
}

function inInteractRange(map, x, y) {
  return distanceSq(map.px, map.py, x, y) <= 2;
}

function tileAt(map, x, y) {
  if (x < 0 || y < 0 || x >= map.w || y >= map.h) return null;
  return map.tiles[y]?.[x] ?? null;
}

function isWalkable(map, x, y) {
  return tileAt(map, x, y) === '.';
}

function canStep(map, x, y, dx, dy) {
  if (dx === 0 && dy === 0) return false;
  if (!isWalkable(map, x + dx, y + dy)) return false;
  if (dx !== 0 && dy !== 0) {
    return isWalkable(map, x + dx, y) && isWalkable(map, x, y + dy);
  }
  return true;
}

function cellKey(x, y) {
  return `${x}:${y}`;
}

function bfs(map, goal, blocked = new Set()) {
  const start = { x: map.px, y: map.py };
  if (goal(start.x, start.y)) return [];
  const queue = [start];
  const visited = new Set([cellKey(start.x, start.y)]);
  const previous = new Map();

  for (let cursor = 0; cursor < queue.length; cursor += 1) {
    const current = queue[cursor];
    for (const direction of CARDINAL) {
      const x = current.x + direction.dx;
      const y = current.y + direction.dy;
      const key = cellKey(x, y);
      if (visited.has(key) || blocked.has(key) || !canStep(map, current.x, current.y, direction.dx, direction.dy)) continue;
      visited.add(key);
      previous.set(key, { from: cellKey(current.x, current.y), dx: direction.dx, dy: direction.dy, x, y });
      if (goal(x, y)) {
        const path = [];
        let step = previous.get(key);
        while (step) {
          path.push(step);
          step = previous.get(step.from);
        }
        path.reverse();
        return path;
      }
      queue.push({ x, y });
    }
  }
  return null;
}

function liveEnemies(map) {
  return (map.enemies ?? []).filter((enemy) => enemy?.alive === true);
}

function enemyBlockSet(map, exceptId = null) {
  return new Set(
    liveEnemies(map)
      .filter((enemy) => enemy.id !== exceptId)
      .map((enemy) => cellKey(enemy.x, enemy.y)),
  );
}

function attackVector(map, enemy, maxDistance = 2) {
  for (const direction of EIGHT_DIRECTIONS) {
    for (let distance = 1; distance <= maxDistance; distance += 1) {
      if (enemy.x === map.px + direction.dx * distance && enemy.y === map.py + direction.dy * distance) {
        return { dx: direction.dx, dy: direction.dy, distance };
      }
    }
  }
  return null;
}

function attackGoalForEnemy(enemy) {
  return (x, y) => EIGHT_DIRECTIONS.some((direction) =>
    [1, 2].some((distance) => enemy.x === x + direction.dx * distance && enemy.y === y + direction.dy * distance),
  );
}

function pierceVector(map, enemy) {
  return attackVector(map, enemy, 3);
}

function imminentHazards(map) {
  const hazards = [];
  for (const enemy of liveEnemies(map)) {
    const telegraph = enemy.telegraph;
    if (telegraph && Number.isInteger(telegraph.remaining) && telegraph.remaining <= 1) {
      hazards.push({ x: telegraph.x, y: telegraph.y, radius: telegraph.radius ?? 0, source: enemy.id });
    }
  }
  for (const zone of map.zones ?? []) {
    const telegraph = zone.telegraph;
    if (telegraph && Number.isInteger(telegraph.remaining) && telegraph.remaining <= 1) {
      hazards.push({ x: telegraph.x, y: telegraph.y, radius: telegraph.radius ?? 0, source: zone.kind });
    }
  }
  return hazards;
}

function hazardContains(hazards, x, y) {
  return hazards.some((hazard) => distanceSq(x, y, hazard.x, hazard.y) <= hazard.radius * hazard.radius);
}

function liveEnemyContains(map, x, y) {
  return liveEnemies(map).some((enemy) => enemy.x === x && enemy.y === y);
}

function simulatedDashLanding(map, dx, dy) {
  let x = map.px;
  let y = map.py;
  let moved = 0;
  for (let step = 0; step < 2; step += 1) {
    if (!canStep(map, x, y, dx, dy)) break;
    x += dx;
    y += dy;
    moved += 1;
  }
  return { x, y, moved };
}

async function rawMove(dx, dy, target) {
  const map = getMap(target);
  const before = { x: map.px, y: map.py };
  expect(canStep(map, before.x, before.y, dx, dy), 'runner attempted a non-legal move', target, { dx, dy, before });
  await act('move', { dx, dy }, target);
  const afterMap = getMap(target);
  expect(afterMap.px === before.x + dx && afterMap.py === before.y + dy, 'valid move did not land on its requested adjacent cell', target, { dx, dy, before, after: { x: afterMap.px, y: afterMap.py } });
  movesSinceTick += 1;
  if (movesSinceTick >= 2) {
    await tickOnce(target, { allowDodge: false });
  }
}

async function rawDash(dx, dy, target) {
  const map = getMap(target);
  const before = { x: map.px, y: map.py };
  const simulated = simulatedDashLanding(map, dx, dy);
  expect(simulated.moved > 0, 'runner attempted a non-legal dash', target, { dx, dy, before });
  await act('dash', { dx, dy }, target);
  const afterMap = getMap(target);
  const moved = distanceSq(before.x, before.y, afterMap.px, afterMap.py) > 0;
  expect(moved, 'valid dash did not move the player', target, { dx, dy, before, after: { x: afterMap.px, y: afterMap.py } });
  movesSinceTick += 1;
  if (movesSinceTick >= 2) {
    await tickOnce(target, { allowDodge: false });
  }
}

async function dodgeImminent(target, desired = null) {
  const map = getMap(target);
  const hazards = imminentHazards(map);
  if (hazards.length === 0) return false;
  const currentUnsafe = hazardContains(hazards, map.px, map.py);
  const desiredUnsafe = desired ? hazardContains(hazards, desired.x, desired.y) : false;
  if (!currentUnsafe && !desiredUnsafe) return false;

  // If the planned adjacent landing is already outside the fixed telegraph,
  // use that legal step as the dodge so route progress is not lost.
  if (desired && !desiredUnsafe) {
    const dx = Math.sign(desired.x - map.px);
    const dy = Math.sign(desired.y - map.py);
    if (canStep(map, map.px, map.py, dx, dy)) {
      await rawMove(dx, dy, { ...target, stage: 'telegraph-dodge-progress' });
      expect(state.mode === 'world', 'telegraph dodge caused an unexpected death', target);
      return true;
    }
  }

  const candidates = EIGHT_DIRECTIONS
    .map((direction) => ({ ...direction, x: map.px + direction.dx, y: map.py + direction.dy }))
    .filter((candidate) => canStep(map, map.px, map.py, candidate.dx, candidate.dy))
    .filter((candidate) => !hazardContains(hazards, candidate.x, candidate.y))
    .sort((a, b) => {
      const aEnemy = liveEnemyContains(map, a.x, a.y) ? 1 : 0;
      const bEnemy = liveEnemyContains(map, b.x, b.y) ? 1 : 0;
      return aEnemy - bEnemy;
    });
  if (candidates.length > 0) {
    await rawMove(candidates[0].dx, candidates[0].dy, { ...target, stage: 'telegraph-dodge' });
    expect(state.mode === 'world', 'telegraph dodge caused an unexpected death', target);
    return true;
  }

  for (const direction of EIGHT_DIRECTIONS) {
    const landing = simulatedDashLanding(map, direction.dx, direction.dy);
    if (landing.moved > 0 && !hazardContains(hazards, landing.x, landing.y)) {
      await rawDash(direction.dx, direction.dy, { ...target, stage: 'telegraph-dash' });
      expect(state.mode === 'world', 'telegraph dash caused an unexpected death', target);
      return true;
    }
  }
  fail('no legal move or dash could leave the imminent telegraph landing zone', target, { hazards, position: { x: map.px, y: map.py } });
}

async function tickOnce(target, { allowDodge = true } = {}) {
  expect(state?.mode === 'world', 'tick requested outside world mode', target);
  if (allowDodge) await dodgeImminent(target);
  expect(state?.mode === 'world', 'tick prelude left world mode', target);
  await act('tick', {}, target);
  movesSinceTick = 0;
  return state.mode === 'world';
}

async function tickCombat(target, count) {
  for (let index = 0; index < count; index += 1) {
    const alive = await tickOnce(target);
    expect(alive, 'player died during a normal combat cooldown tick', target);
  }
}

async function moveOne(dx, dy, target) {
  const map = getMap(target);
  const desired = { x: map.px + dx, y: map.py + dy };
  if (await dodgeImminent(target, desired)) return false;
  await rawMove(dx, dy, target);
  expect(state.mode === 'world', 'player died during a normal move', target);
  return true;
}

async function recoverIfNeeded(target) {
  if (state?.mode !== 'world') return;
  const player = state.player;
  if (!Number.isFinite(player?.hp) || !Number.isFinite(player?.max_hp)) return;
  const ratio = player.hp / Math.max(1, player.max_hp);
  if (ratio > 0.56) return;

  const healTarget = { ...target, stage: 'recover' };
  await dodgeImminent(healTarget);
  if (state.player.medkits > 0 && state.player.hp < state.player.max_hp) {
    const beforeHp = state.player.hp;
    const beforeMedkits = state.player.medkits;
    await act('heal', {}, healTarget);
    expect(state.mode === 'world', 'heal left world mode unexpectedly', healTarget);
    expect(state.player.hp > beforeHp && state.player.medkits === beforeMedkits - 1, 'heal did not restore HP and consume exactly one medkit', healTarget);
    await tickCombat(healTarget, 1);
    return;
  }

  const guard = state.skills?.[1];
  if (guard?.ready && state.player.energy >= guard.cost && state.player.hp < state.player.max_hp) {
    const beforeEnergy = state.player.energy;
    await act('skill', { slot: 1 }, healTarget);
    expect(state.mode === 'world', 'guard skill left world mode unexpectedly', healTarget);
    expect(state.player.energy === beforeEnergy - guard.cost, 'guard skill did not consume its public energy cost', healTarget);
    if (activeRun) activeRun.combat.guardUses += 1;
    await tickCombat(healTarget, 1);
  }
}

function enemyById(map, id) {
  return map.enemies?.find((enemy) => enemy.id === id) ?? null;
}

function findEnemyPath(map, enemy) {
  return bfs(map, attackGoalForEnemy(enemy), enemyBlockSet(map));
}

async function approachEnemy(id, target) {
  for (let steps = 0; steps < MAX_ROUTE_STEPS; steps += 1) {
    const map = getMap(target);
    const enemy = enemyById(map, id);
    if (!enemy || !enemy.alive) return;
    const attack = attackVector(map, enemy, 2);
    const pierce = pierceVector(map, enemy);
    const pulseReady = !activeRun.combat.pulseUses && state.skills?.[0]?.ready && state.player.energy >= (state.skills?.[0]?.cost ?? Number.MAX_SAFE_INTEGER);
    const pierceReady = pierce && state.skills?.[2]?.ready && state.player.energy >= (state.skills?.[2]?.cost ?? Number.MAX_SAFE_INTEGER);
    if (attack || pierceReady || (pulseReady && distanceSq(map.px, map.py, enemy.x, enemy.y) <= 6)) return;
    await recoverIfNeeded({ ...target, enemy_id: id, stage: 'approach-enemy' });
    const updatedMap = getMap(target);
    const updatedEnemy = enemyById(updatedMap, id);
    if (!updatedEnemy || !updatedEnemy.alive) return;
    const route = findEnemyPath(updatedMap, updatedEnemy);
    expect(route && route.length > 0, 'no BFS route to a legal attack position for enemy', { ...target, enemy_id: id, stage: 'approach-enemy' }, { enemy: updatedEnemy });
    await moveOne(route[0].dx, route[0].dy, { ...target, enemy_id: id, stage: 'approach-enemy' });
  }
  fail('enemy approach exceeded route step limit', { ...target, enemy_id: id, stage: 'approach-enemy' });
}

function hpSnapshot(map) {
  return new Map((map.enemies ?? []).map((enemy) => [enemy.id, { hp: enemy.hp, alive: enemy.alive }]));
}

function anyEnemyDamaged(before, afterMap) {
  for (const enemy of afterMap.enemies ?? []) {
    const prior = before.get(enemy.id);
    if (prior && (enemy.hp < prior.hp || (prior.alive && !enemy.alive))) return true;
  }
  return false;
}

async function fightEnemy(id, target) {
  for (let round = 0; round < MAX_COMBAT_ROUNDS; round += 1) {
    const map = getMap(target);
    const enemy = enemyById(map, id);
    if (!enemy || !enemy.alive) return;
    const combatTarget = { ...target, enemy_id: id, enemy_name: enemy.name, stage: 'combat' };
    activeTarget = combatTarget;
    await recoverIfNeeded(combatTarget);
    expect(state.mode === 'world', 'normal combat reached dead mode', combatTarget);

    const currentMap = getMap(combatTarget);
    const currentEnemy = enemyById(currentMap, id);
    if (!currentEnemy || !currentEnemy.alive) return;
    const closeEnemies = liveEnemies(currentMap).filter((candidate) => distanceSq(currentMap.px, currentMap.py, candidate.x, candidate.y) <= 6);
    const pulse = state.skills?.[0];
    if (!activeRun.combat.pulseUses && closeEnemies.length > 0 && pulse?.ready && state.player.energy >= pulse.cost) {
      if (await dodgeImminent(combatTarget)) continue;
      const before = hpSnapshot(currentMap);
      await act('skill', { slot: 0 }, combatTarget);
      expect(state.mode === 'world', 'pulse skill left world mode unexpectedly', combatTarget);
      expect(anyEnemyDamaged(before, getMap(combatTarget)), 'pulse skill produced no public enemy damage', combatTarget, { enemy: currentEnemy });
      activeRun.combat.pulseUses += 1;
      activeRun.combat.skillUses += 1;
      await tickCombat(combatTarget, 1);
      continue;
    }

    const pierceDirection = pierceVector(currentMap, currentEnemy);
    const pierce = state.skills?.[2];
    if (pierceDirection && pierce?.ready && state.player.energy >= pierce.cost) {
      if (await dodgeImminent(combatTarget)) continue;
      const beforeHp = currentEnemy.hp;
      await act('skill', { slot: 2, dx: pierceDirection.dx, dy: pierceDirection.dy }, combatTarget);
      const afterEnemy = enemyById(getMap(combatTarget), id);
      expect(!afterEnemy || !afterEnemy.alive || afterEnemy.hp < beforeHp, 'pierce skill produced no public damage', combatTarget, { beforeHp, afterEnemy });
      activeRun.combat.pierceUses += 1;
      activeRun.combat.skillUses += 1;
      await tickCombat(combatTarget, 1);
      continue;
    }

    const attackDirection = attackVector(currentMap, currentEnemy, 2);
    if (attackDirection) {
      if (await dodgeImminent(combatTarget)) continue;
      const beforeHp = currentEnemy.hp;
      await act('attack', { dx: attackDirection.dx, dy: attackDirection.dy }, combatTarget);
      const afterEnemy = enemyById(getMap(combatTarget), id);
      expect(!afterEnemy || !afterEnemy.alive || afterEnemy.hp < beforeHp, 'attack produced no public damage', combatTarget, { beforeHp, afterEnemy });
      activeRun.combat.attacks += 1;
      // The contract under test requires two explicit 250ms ticks after attack.
      await tickCombat(combatTarget, 2);
      continue;
    }

    await approachEnemy(id, combatTarget);
  }
  fail('combat round limit exceeded', { ...target, enemy_id: id, stage: 'combat' });
}

function chooseEnemyForClear(map) {
  const candidates = liveEnemies(map)
    .map((enemy) => ({ enemy, route: findEnemyPath(map, enemy) }))
    .filter((candidate) => candidate.route !== null)
    .sort((a, b) => a.route.length - b.route.length);
  return candidates[0] ?? null;
}

async function clearAllEnemies(target) {
  for (let count = 0; count < 20; count += 1) {
    const map = getMap(target);
    const alive = liveEnemies(map);
    if (alive.length === 0) return;
    const selected = chooseEnemyForClear(map);
    expect(selected, 'an alive enemy has no public BFS route to an attack position', { ...target, stage: 'clear-enemies' }, { alive });
    const enemyTarget = { ...target, enemy_id: selected.enemy.id, enemy_name: selected.enemy.name, stage: 'clear-enemies' };
    await approachEnemy(selected.enemy.id, enemyTarget);
    await fightEnemy(selected.enemy.id, enemyTarget);
  }
  fail('enemy clear loop exceeded its finite limit', { ...target, stage: 'clear-enemies' });
}

async function travelToObject(id, target, { intercept = true } = {}) {
  for (let step = 0; step < MAX_ROUTE_STEPS; step += 1) {
    const map = getMap(target);
    const objectPosition = positionOfObject(map, id);
    expect(objectPosition, 'target object disappeared from the public map view', { ...target, object_id: id }, { id });
    if (inInteractRange(map, objectPosition.x, objectPosition.y)) return;

    if (intercept) {
      const nearbyEnemy = liveEnemies(map)
        .filter((enemy) => distanceSq(map.px, map.py, enemy.x, enemy.y) <= 6)
        .sort((a, b) => distanceSq(map.px, map.py, a.x, a.y) - distanceSq(map.px, map.py, b.x, b.y))[0];
      if (nearbyEnemy) {
        await approachEnemy(nearbyEnemy.id, { ...target, object_id: id, stage: 'encounter' });
        await fightEnemy(nearbyEnemy.id, { ...target, object_id: id, stage: 'encounter' });
        continue;
      }
    }

    const goal = (x, y) => distanceSq(x, y, objectPosition.x, objectPosition.y) <= 2;
    const route = bfs(map, goal, enemyBlockSet(map));
    if (!route) {
      const unblocked = bfs(map, goal);
      expect(unblocked, 'no public BFS route to target within 1.5 interaction range', { ...target, object_id: id, stage: 'route' }, { object: objectPosition, map_revision: map.revision });
      const enemyOnRoute = liveEnemies(map).find((enemy) => unblocked.some((node) => node.x === enemy.x && node.y === enemy.y));
      expect(enemyOnRoute, 'BFS route became unavailable without an enemy chokepoint', { ...target, object_id: id, stage: 'route' }, { object: objectPosition, route_length: unblocked.length });
      await approachEnemy(enemyOnRoute.id, { ...target, object_id: id, stage: 'encounter' });
      await fightEnemy(enemyOnRoute.id, { ...target, object_id: id, stage: 'encounter' });
      continue;
    }
    expect(route.length > 0, 'route planner returned an empty route outside interaction range', { ...target, object_id: id });
    await moveOne(route[0].dx, route[0].dy, { ...target, object_id: id, stage: 'move' });
  }
  fail('route step limit exceeded', { ...target, object_id: id, stage: 'route' });
}

function mainObjectivesFromView(view, target) {
  const objectives = (view.objectives ?? []).filter((objective) => objective.optional !== true);
  expect(objectives.length >= 3, 'world view does not expose three ordered main objectives', target, { objectives });
  return objectives;
}

function decisionEntry(view, worldId) {
  const decisions = view?.decisions;
  if (isRecord(decisions)) {
    return Object.prototype.hasOwnProperty.call(decisions, worldId)
      ? { found: true, value: decisions[worldId] }
      : { found: false, value: null };
  }
  if (Array.isArray(decisions)) {
    const entry = decisions.find((candidate) =>
      candidate?.world_id === worldId || candidate?.worldId === worldId || candidate?.id === worldId,
    );
    if (entry) {
      return {
        found: true,
        value: entry.choice ?? entry.option_id ?? entry.decision ?? entry.value ?? entry,
      };
    }
  }
  return { found: false, value: null };
}

function decisionsSnapshot(view) {
  return cloneJson(view?.decisions ?? null);
}

function sameDecisions(before, after) {
  return JSON.stringify(before) === JSON.stringify(after);
}

async function interactMainPoint(objective, target) {
  activeTarget = { ...target, objective_id: objective.id, objective_title: objective.title, stage: 'objective' };
  const map = getMap(activeTarget);
  const point = map.points?.find((candidate) => candidate.id === objective.id);
  expect(point, 'ordered objective has no matching public map point', activeTarget, { objective });
  await travelToObject(point.id, activeTarget);
  const arrived = getMap(activeTarget);
  expect(inInteractRange(arrived, point.x, point.y), 'player is not within 1.5 of objective point after BFS', activeTarget, { point, position: { x: arrived.px, y: arrived.py } });
  await act('interact', { id: point.id }, activeTarget);
  expect(state.mode === 'world', 'main objective interaction left world mode unexpectedly', activeTarget);
  const completed = state.objectives?.find((candidate) => candidate.id === objective.id);
  expect(completed?.done === true, 'main objective interaction did not mark its public objective done', activeTarget, { objective: completed });
  await tickCombat(activeTarget, 1);
}

async function interactOptionalNpc(worldId, target, testInvalidChoice, replay) {
  const optionalTarget = { ...target, stage: 'optional-decision' };
  const optionalObjective = state.objectives?.find((objective) => objective.optional === true);
  const persisted = decisionEntry(state, worldId);
  if (replay && (persisted.found || optionalObjective?.done === true)) {
    expect(
      persisted.found && optionalObjective?.done === true,
      'replay public view has an inconsistent persisted decision/optional-objective pair',
      optionalTarget,
      { persisted, optionalObjective, decisions: state.decisions },
    );
    expect(state.mode === 'world' && state.dialog === null, 'replay entered with a persisted decision but is not in a clean world view', optionalTarget, { dialog: state.dialog });
    const snapshot = decisionsSnapshot(state);
    activeRun.optional_replay_skipped = true;
    activeRun.decision = persisted.value;
    return { skipped: true, decision: persisted.value, decisions: snapshot };
  }

  const map = getMap(target);
  const npc = map.npcs?.[0];
  expect(npc && typeof npc.id === 'string', 'world has no public optional NPC to exercise', { ...target, stage: 'optional-npc' }, { npcs: map.npcs });
  const npcTarget = { ...target, npc_id: npc.id, npc_name: npc.name, stage: 'optional-npc' };
  await travelToObject(npc.id, npcTarget);
  const arrived = getMap(npcTarget);
  expect(inInteractRange(arrived, npc.x, npc.y), 'player is not within 1.5 of optional NPC after BFS', npcTarget, { npc, position: { x: arrived.px, y: arrived.py } });
  await act('interact', { id: npc.id }, npcTarget);
  expect(state.mode === 'dialog' && isRecord(state.dialog), 'optional NPC interaction did not open public dialog mode', npcTarget, { dialog: state.dialog });
  expect(Array.isArray(state.dialog.choices) && state.dialog.choices.length > 0, 'optional dialog has no public choices', npcTarget, { dialog: state.dialog });

  if (testInvalidChoice) {
    const before = state;
    await act('choice', { option_id: '__invalid_choice__' }, { ...npcTarget, stage: 'invalid-choice' });
    assertPublicUnchanged(before, state, 'invalid choice', { ...npcTarget, stage: 'invalid-choice' });
    expect(state.mode === 'dialog', 'invalid choice left dialog mode', npcTarget);
  }

  const choice = state.dialog.choices[0];
  expect(typeof choice.id === 'string' && choice.id.length > 0, 'first public dialog choice has no id', npcTarget, { choice });
  await act('choice', { option_id: choice.id }, npcTarget);
  expect(state.mode === 'world' && state.dialog === null, 'valid choice did not return to world mode', npcTarget);
  const chosenOptionalObjective = state.objectives?.find((objective) => objective.optional === true);
  expect(chosenOptionalObjective?.done === true, 'valid choice did not complete the optional public objective', npcTarget, { optionalObjective: chosenOptionalObjective });
  const stored = decisionEntry(state, worldId);
  expect(stored.found, 'valid choice did not persist a public world decision', npcTarget, { decisions: state.decisions });
  activeRun.decision = stored.value;
  return { skipped: false, decision: stored.value, decisions: decisionsSnapshot(state) };
}

function findDistantObject(map) {
  return allMapObjects(map)
    .filter((object) => distanceSq(map.px, map.py, object.x, object.y) > 2)
    .sort((a, b) => distanceSq(map.px, map.py, b.x, b.y) - distanceSq(map.px, map.py, a.x, a.y))[0] ?? null;
}

async function runWorld(worldId, { invalidChecks = false, replay = false } = {}) {
  beginRun(worldId, 'world');
  activeTarget = { world_id: worldId, replay, stage: 'enter' };
  await act('enter', { world_id: worldId }, activeTarget);
  expect(state.mode === 'world' && state.map?.world_id === worldId, 'enter did not start the requested world', activeTarget, { map: state.map });
  const sideStoryTokensBefore = state.player.side_story_tokens;
  if (replay) {
    expect(state.worlds?.find((world) => world.id === worldId)?.cleared === true, 'replay run did not enter a previously cleared world', activeTarget);
  }

  if (invalidChecks) {
    const beforeContinue = state;
    await act('continue', {}, { world_id: worldId, stage: 'invalid-continue-inworld' });
    assertPublicUnchanged(beforeContinue, state, 'continue in world', { world_id: worldId, stage: 'invalid-continue-inworld' });
    expect(state.mode === 'world', 'invalid continue in world changed mode', { world_id: worldId, stage: 'invalid-continue-inworld' });

    const distant = findDistantObject(getMap({ world_id: worldId, stage: 'invalid-distant-interact' }));
    expect(distant, 'world view has no object distant enough for invalid interact coverage', { world_id: worldId, stage: 'invalid-distant-interact' });
    const beforeInteract = state;
    await act('interact', { id: distant.id }, { world_id: worldId, object_id: distant.id, stage: 'invalid-distant-interact' });
    assertPublicUnchanged(beforeInteract, state, 'distant interact', { world_id: worldId, object_id: distant.id, stage: 'invalid-distant-interact' });
    expect(state.mode === 'world', 'invalid distant interact changed mode', { world_id: worldId, object_id: distant.id, stage: 'invalid-distant-interact' });
  }

  const objectives = mainObjectivesFromView(state, { world_id: worldId, stage: 'objectives' });
  const objectiveNames = (state.objectives ?? []).map((objective) => objective.title ?? objective.name ?? objective.id);
  await interactMainPoint(objectives[0], { world_id: worldId, replay, stage: 'main-1' });
  const optionalFlow = await interactOptionalNpc(worldId, { world_id: worldId, replay }, invalidChecks, replay);
  await interactMainPoint(objectives[1], { world_id: worldId, replay, stage: 'main-2' });
  await clearAllEnemies({ world_id: worldId, replay, stage: 'combat-clear' });
  await interactMainPoint(objectives[2], { world_id: worldId, replay, stage: 'main-3' });

  const finalMap = getMap({ world_id: worldId, replay, stage: 'exit' });
  const portal = finalMap.portals?.find((candidate) => candidate.kind === 'exit' || candidate.id === 'exit');
  expect(portal, 'world view has no public exit portal', { world_id: worldId, replay, stage: 'exit' }, { portals: finalMap.portals });
  await travelToObject(portal.id, { world_id: worldId, replay, portal_id: portal.id, stage: 'exit' }, { intercept: false });
  const exitMap = getMap({ world_id: worldId, replay, portal_id: portal.id, stage: 'extract' });
  expect(inInteractRange(exitMap, portal.x, portal.y), 'player is not within 1.5 of active exit before extract', { world_id: worldId, replay, portal_id: portal.id, stage: 'extract' }, { portal });
  await act('extract', {}, { world_id: worldId, replay, portal_id: portal.id, stage: 'extract' });
  expect(state.mode === 'result' && isRecord(state.result), 'extract did not produce public result mode', { world_id: worldId, replay, stage: 'extract' }, { result: state.result });
  expect(state.result.all_complete === (replay || worldId === 'clockworks'), 'result all_complete flag is inconsistent with ordered campaign progress', { world_id: worldId, replay, stage: 'extract' }, { result: state.result });
  expect(Number.isInteger(state.result.reward) && state.result.reward >= 0, 'result reward is not a public non-negative integer', { world_id: worldId, replay, stage: 'extract' }, { result: state.result });
  expect(Number.isInteger(state.result.side_story_tokens) && state.result.side_story_tokens >= 0, 'result did not expose a public side-story voucher award', { world_id: worldId, replay, stage: 'extract' }, { result: state.result });
  if (replay) {
    expect(state.result.reward === 0, 'replay extract awarded a duplicate world reward', { world_id: worldId, replay, stage: 'extract' }, { result: state.result });
    expect(state.result.side_story_tokens === 0, 'replay extract awarded duplicate first-clear vouchers', { world_id: worldId, replay, stage: 'extract' }, { result: state.result });
    expect(state.player.side_story_tokens === sideStoryTokensBefore, 'replay extract changed public side-story vouchers', { world_id: worldId, replay, stage: 'extract' }, { before: sideStoryTokensBefore, after: state.player.side_story_tokens });
    expect(state.result.stats?.replay === true, 'replay extract did not expose replay=true in public result stats', { world_id: worldId, replay, stage: 'extract' }, { stats: state.result.stats });
    if (optionalFlow.skipped) {
      expect(sameDecisions(optionalFlow.decisions, state.decisions), 'replay changed a persisted public decision while the dialog was skipped', { world_id: worldId, replay, stage: 'decision-invariance' }, { before: optionalFlow.decisions, after: state.decisions, decision: optionalFlow.decision });
    }
  } else {
    expect(state.result.reward > 0, 'first-clear extract awarded no public reward', { world_id: worldId, stage: 'extract' }, { result: state.result });
    const expectedVoucherAward = optionalFlow.skipped ? 1 : 2;
    expect(state.result.side_story_tokens === expectedVoucherAward, 'first-clear/optional voucher award did not follow the public rule', { world_id: worldId, stage: 'extract' }, { expected: expectedVoucherAward, result: state.result });
    expect(state.player.side_story_tokens === sideStoryTokensBefore + expectedVoucherAward, 'first-clear/optional vouchers did not increase the public player total', { world_id: worldId, stage: 'extract' }, { before: sideStoryTokensBefore, after: state.player.side_story_tokens });
  }
  const result = cloneJson(state.result);
  expect(activeRun.combat.attacks > 0, 'complete world flow did not exercise a public attack', { world_id: worldId, replay, stage: 'combat-report' }, { combat: activeRun.combat });
  expect(activeRun.combat.skillUses > 0, 'complete world flow did not exercise a public skill', { world_id: worldId, replay, stage: 'combat-report' }, { combat: activeRun.combat });
  expect(Number.isInteger(result.stats?.enemies_defeated) && result.stats.enemies_defeated > 0, 'complete world flow reported no public enemy kills', { world_id: worldId, replay, stage: 'combat-report' }, { stats: result.stats });

  return {
    id: worldId,
    replay,
    objective_names: objectiveNames,
    kills: result.stats?.enemies_defeated ?? null,
    damage_taken: result.stats?.damage_taken ?? null,
    hp: state.player.hp,
    reward: result.reward,
    side_story_tokens: result.side_story_tokens,
    ticks: activeRun.ticks,
    actions: activeRun.actions,
    rank: result.rank,
    all_complete: result.all_complete,
    decision: optionalFlow.decision,
    optional_replay_skipped: optionalFlow.skipped,
    combat: cloneJson(activeRun.combat),
  };
}

async function runDeathRestartCheck(worldId) {
  beginRun(`death:${worldId}`, 'death-restart');
  activeTarget = { world_id: worldId, stage: 'death-enter' };
  await act('enter', { world_id: worldId }, activeTarget);
  expect(state.mode === 'world', 'death/restart check could not enter world', activeTarget);
  const spawnPosition = { x: state.map.px, y: state.map.py };
  const selected = liveEnemies(getMap(activeTarget)).find((enemy) => enemy.kind === 'chaser') ?? liveEnemies(getMap(activeTarget))[0];
  expect(selected, 'death/restart check found no public live enemy', activeTarget, { enemies: state.map.enemies });
  const enemyTarget = { world_id: worldId, enemy_id: selected.id, enemy_name: selected.name, stage: 'death-approach' };
  await approachEnemy(selected.id, enemyTarget);
  const nearMap = getMap(enemyTarget);
  const nearEnemy = enemyById(nearMap, selected.id);
  expect(nearEnemy && nearEnemy.alive && distanceSq(nearMap.px, nearMap.py, nearEnemy.x, nearEnemy.y) <= 6, 'death/restart check did not approach a live enemy', enemyTarget, { enemy: nearEnemy });

  let deathTicks = 0;
  while (state.mode === 'world' && deathTicks < MAX_DEATH_TICKS) {
    await act('tick', {}, { ...enemyTarget, stage: 'death-tick' });
    deathTicks += 1;
  }
  expect(state.mode === 'dead' && state.player.hp === 0, 'finite enemy tick sequence did not verify the dead state', { ...enemyTarget, stage: 'death-result' }, { deathTicks, hp: state.player.hp });
  const deadPosition = state.map ? { x: state.map.px, y: state.map.py } : null;

  await act('restart', {}, { world_id: worldId, stage: 'restart' });
  expect(state.mode === 'world' && state.map?.world_id === worldId, 'restart did not return to world mode', { world_id: worldId, stage: 'restart' });
  expect(state.player.hp === state.player.max_hp, 'restart did not restore full public HP', { world_id: worldId, stage: 'restart' });
  expect(state.map.px === spawnPosition.x && state.map.py === spawnPosition.y, 'restart did not reset to the public world spawn position', { world_id: worldId, stage: 'restart' }, { spawnPosition, position: { x: state.map.px, y: state.map.py } });
  const restartHp = state.player.hp;
  await act('retreat', {}, { world_id: worldId, stage: 'death-retreat' });
  expect(state.mode === 'hub', 'death/restart cleanup did not return to hub', { world_id: worldId, stage: 'death-retreat' });
  return { world_id: worldId, ticks_to_dead: deathTicks, dead_hp: 0, dead_position: deadPosition, restart_hp: restartHp };
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (options.help) {
    console.log('Usage: node tools/campaign-smoke.mjs [--probe PATH]');
    return;
  }

  probeClient = new ProbeClient(options.probe);
  state = await act('new', {});
  expect(state.mode === 'hub', 'new did not establish hub mode');
  expect(state.player.reward_points === 30, 'new did not expose the required initial 30 reward points', null, { player: state.player });
  expect(state.player.side_story_tokens === 0, 'new did not expose zero initial side-story vouchers', null, { player: state.player });
  expect(!Object.prototype.hasOwnProperty.call(state.player, 'level'), 'new public player view still exposes level', null, { player: state.player });

  const beforeUnknownEnter = state;
  await act('enter', { world_id: '__unknown_world__' }, { stage: 'invalid-enter-unknown' });
  assertPublicUnchanged(beforeUnknownEnter, state, 'enter unknown', { stage: 'invalid-enter-unknown' });
  expect(state.mode === 'hub', 'invalid enter unknown changed hub mode', { stage: 'invalid-enter-unknown' });

  const armorExchange = state.exchanges?.find((exchange) =>
    exchange.reward_cost <= state.player.reward_points && exchange.token_cost <= state.player.side_story_tokens && /reinforced|armor|suit|护甲/i.test(`${exchange.id} ${exchange.name}`),
  ) ?? state.exchanges?.find((exchange) => exchange.reward_cost <= state.player.reward_points && exchange.token_cost <= state.player.side_story_tokens);
  expect(armorExchange, 'no affordable public armor-like exchange was exposed from the initial 30 reward points', { stage: 'exchange' }, { exchanges: state.exchanges });
  expect(armorExchange.category === 'equipment' && armorExchange.grade === 'D', 'initial exchange did not expose the expected D-grade equipment metadata', { stage: 'exchange' }, { exchange: armorExchange });
  const beforeUpgrade = cloneJson(state);
  await act('exchange', { id: armorExchange.id }, { stage: 'exchange', exchange_id: armorExchange.id });
  expect(state.player.reward_points === beforeUpgrade.player.reward_points - armorExchange.reward_cost, 'exchange did not deduct exactly its public reward-point cost', { stage: 'exchange', exchange_id: armorExchange.id }, { before: beforeUpgrade.player, after: state.player });
  expect(state.player.side_story_tokens === beforeUpgrade.player.side_story_tokens - armorExchange.token_cost, 'exchange did not deduct exactly its public voucher cost', { stage: 'exchange', exchange_id: armorExchange.id }, { before: beforeUpgrade.player, after: state.player });
  const exchanged = state.exchanges.find((exchange) => exchange.id === armorExchange.id);
  expect(exchanged?.owned === true, 'successful exchange did not mark the public entry as owned', { stage: 'exchange', exchange_id: armorExchange.id }, { before: armorExchange, after: exchanged });
  const pointsAfterExchange = state.player.reward_points;
  const tokensAfterExchange = state.player.side_story_tokens;
  await act('exchange', { id: armorExchange.id }, { stage: 'exchange-duplicate', exchange_id: armorExchange.id });
  expect(state.player.reward_points === pointsAfterExchange && state.player.side_story_tokens === tokensAfterExchange, 'duplicate exchange changed public resources', { stage: 'exchange-duplicate', exchange_id: armorExchange.id }, { player: state.player });
  expect(/重复兑换|已经兑换/.test(state.last_event), 'duplicate exchange did not produce a public failure event', { stage: 'exchange-duplicate', exchange_id: armorExchange.id }, { last_event: state.last_event });

  await act('equip', { id: 'echo' }, { stage: 'equip-echo' });
  expect(state.player.build === 'vanguard', 'unowned echo lineage was incorrectly equipped', { stage: 'equip-echo' }, { player: state.player });
  await act('equip', { id: 'vanguard' }, { stage: 'equip-vanguard' });
  expect(state.player.build === 'vanguard', 'vanguard is not accepted as a public build', { stage: 'equip-vanguard' }, { player: state.player });
  expect(state.player.reward_points === pointsAfterExchange && state.player.side_story_tokens === tokensAfterExchange, 'build switching changed public exchange resources', { stage: 'equip' }, { player: state.player });

  const worlds = [];
  for (let index = 0; index < WORLD_ORDER.length; index += 1) {
    const worldId = WORLD_ORDER[index];
    const report = await runWorld(worldId, { invalidChecks: index === 0, replay: false });
    worlds.push(report);
    expect(report.all_complete === (index === WORLD_ORDER.length - 1), 'ordered first-clear result has the wrong all_complete value', { world_id: worldId, stage: 'first-clear-result' }, { report });
    if (index < WORLD_ORDER.length - 1) {
      await act('continue', {}, { world_id: worldId, stage: 'continue-to-hub' });
      expect(state.mode === 'hub', 'continue did not return from result to hub between worlds', { world_id: worldId, stage: 'continue-to-hub' });
    }
  }

  const firstClearRewardPoints = state.player.reward_points;
  const firstClearVouchers = state.player.side_story_tokens;
  await act('continue', {}, { world_id: 'clockworks', stage: 'continue-after-all-complete' });
  expect(state.mode === 'hub', 'continue after all-complete result did not return to hub', { stage: 'continue-after-all-complete' });
  const deathRestart = await runDeathRestartCheck('grey_hive');
  expect(state.player.reward_points === firstClearRewardPoints && state.player.side_story_tokens === firstClearVouchers, 'death/restart check changed settled exchange resources', { stage: 'death-reward-invariance' }, { before: { reward_points: firstClearRewardPoints, side_story_tokens: firstClearVouchers }, after: { reward_points: state.player.reward_points, side_story_tokens: state.player.side_story_tokens } });

  const replayRewardPointsBefore = state.player.reward_points;
  const replayVouchersBefore = state.player.side_story_tokens;
  const replay = await runWorld('grey_hive', { invalidChecks: false, replay: true });
  expect(state.player.reward_points === replayRewardPointsBefore, 'full replay changed public reward points despite reward=0', { world_id: 'grey_hive', stage: 'replay-reward-invariance' }, { before: replayRewardPointsBefore, after: state.player.reward_points });
  expect(state.player.side_story_tokens === replayVouchersBefore, 'full replay changed public side-story vouchers despite zero voucher award', { world_id: 'grey_hive', stage: 'replay-voucher-invariance' }, { before: replayVouchersBefore, after: state.player.side_story_tokens });

  expect(worlds.length === 3, 'not all ordered worlds produced a report', { stage: 'final' }, { worlds });
  expect(worlds.every((report) => report.reward > 0), 'one or more first-clear worlds had no reward', { stage: 'final' }, { worlds });
  expect(replay.reward === 0 && replay.all_complete === true, 'replay did not preserve all_complete=true with zero duplicate reward', { stage: 'final' }, { replay });

  console.log(JSON.stringify({
    ok: true,
    worlds,
    replay,
    death_restart: deathRestart,
    final_all_complete: replay.all_complete,
  }));
}

let exitCode = 0;
try {
  await main();
} catch (error) {
  exitCode = 1;
  const details = error instanceof SmokeError ? error.details : diagnostics();
  console.log(JSON.stringify({
    ok: false,
    error: error?.message ?? String(error),
    mode: details?.mode ?? state?.mode ?? null,
    last_event: details?.last_event ?? state?.last_event ?? null,
    player: details?.player ?? state?.player ?? null,
    position: details?.position ?? (state?.map ? { x: state.map.px, y: state.map.py, world_id: state.map.world_id } : null),
    target: error instanceof SmokeError ? error.target : activeTarget,
    detail: details,
  }));
} finally {
  if (probeClient) await probeClient.close();
}
process.exitCode = exitCode;
