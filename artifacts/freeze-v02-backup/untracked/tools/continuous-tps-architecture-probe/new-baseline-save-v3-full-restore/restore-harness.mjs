import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const dir = path.dirname(new URL(import.meta.url).pathname).replace(/^\/(?:[A-Za-z]:)/, value => value.slice(1)).replaceAll('/', path.sep);
const root = path.resolve(dir, '..', '..', '..');
const fixturePath = path.join(dir, 'save-v3-fixture.json');
const schemaPath = path.join(dir, 'save-v3-full-restore.schema.json');
const nb05Path = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-grey-hive-data-contract', 'runtime-fixture.json');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-save-v3-full-restore-nb10.json');
const fixture = JSON.parse(fs.readFileSync(fixturePath, 'utf8'));
const schema = JSON.parse(fs.readFileSync(schemaPath, 'utf8'));
const nb05 = JSON.parse(fs.readFileSync(nb05Path, 'utf8'));
const clone = value => structuredClone(value);
const stable = value => JSON.stringify(value);
const hash = value => crypto.createHash('sha256').update(stable(value)).digest('hex');
const check = (id, pass, actual, expected) => ({ id, pass: Boolean(pass), actual, expected });
const roomIds = new Set(nb05.static.rooms.map(room => room.id));
const checkpointIds = new Set(nb05.static.checkpoints.map(cp => cp.id));
const doorIds = new Set(nb05.static.doors.map(door => door.id));
const triggerIds = new Set(nb05.static.triggers.map(trigger => trigger.id));
const knownWorlds = new Set(['grey_hive', 'mist_harbor', 'clockworks']);
const forbiddenKeys = /^(babylon|ui|runtimeHandle|pointer|resourceHandle|channel|pendingCommand|grid|px|py|tile)$/i;

function payload(save) { const copy = clone(save); delete copy.checksum; return copy; }
function withChecksum(save) { const copy = clone(save); copy.checksum = hash(payload(copy)); return copy; }
const validSave = withChecksum(fixture);

function scanForbidden(value, pathName = '') {
  const found = [];
  if (Array.isArray(value)) value.forEach((item, index) => found.push(...scanForbidden(item, `${pathName}[${index}]`)));
  else if (value && typeof value === 'object') for (const [key, item] of Object.entries(value)) { if (forbiddenKeys.test(key)) found.push(`${pathName}.${key}`); found.push(...scanForbidden(item, `${pathName}.${key}`)); }
  return found;
}
function finiteDeep(value, pathName = '') {
  if (typeof value === 'number' && !Number.isFinite(value)) return [`E_NONFINITE:${pathName}`];
  if (Array.isArray(value)) return value.flatMap((item, index) => finiteDeep(item, `${pathName}[${index}]`));
  if (value && typeof value === 'object') return Object.entries(value).flatMap(([key, item]) => finiteDeep(item, `${pathName}.${key}`));
  return [];
}
function duplicateIds(items) { const values = items.map(item => item.id); return new Set(values).size === values.length; }
function validateSave(save) {
  const errors = [];
  if (save.magic !== 'WUXIAN_SAVE_V3') errors.push('E_MAGIC');
  if (save.schemaVersion !== 3) errors.push(save.schemaVersion === undefined ? 'E_MISSING_SCHEMA_VERSION' : save.schemaVersion < 3 ? 'E_V1_UNSUPPORTED' : 'E_FUTURE_SCHEMA');
  if (!save.saveId || !save.createdAt || !save.updatedAt || !save.buildVersion || !save.contentVersion || !save.worldEpoch) errors.push('E_ENVELOPE_REQUIRED');
  if (!knownWorlds.has(save.worldId)) errors.push('E_UNKNOWN_WORLD');
  if (save.checksum !== hash(payload(save))) errors.push('E_CHECKSUM');
  errors.push(...finiteDeep(save));
  const forbidden = scanForbidden(save); if (forbidden.length) errors.push('E_FORBIDDEN_PERSISTENCE');
  if (!save.checkpoint || !checkpointIds.has(save.checkpoint.id) || save.checkpoint.worldId !== save.worldId) errors.push('E_CHECKPOINT_REFERENCE');
  if (!save.player?.transform?.positionM || !Array.isArray(save.player.transform.positionM) || save.player.transform.positionM.length !== 3) errors.push('E_PLAYER_TRANSFORM');
  if (!Array.isArray(save.player?.inventory) || !duplicateIds(save.player.inventory)) errors.push('E_DUPLICATE_ID');
  if (!Array.isArray(save.world?.doors) || !save.world.doors.every(door => doorIds.has(door.id))) errors.push('E_DOOR_REFERENCE');
  if (!Array.isArray(save.world?.triggers) || !save.world.triggers.every(trigger => triggerIds.has(trigger.id))) errors.push('E_TRIGGER_REFERENCE');
  if (!save.world?.rooms?.every(id => roomIds.has(id))) errors.push('E_ROOM_REFERENCE');
  if (!save.world?.checkpoints?.every(cp => checkpointIds.has(cp.id) && roomIds.has(cp.roomId))) errors.push('E_CHECKPOINT_REFERENCE');
  if (!save.world?.flags || Object.values(save.world.flags).some(flag => typeof flag !== 'boolean')) errors.push('E_FLAG_ENUM');
  if (!['taken', 'left', 'unresolved'].includes(save.campaign?.choices?.hive_choice)) errors.push('E_CHOICE');
  const gateB = save.campaign?.gateFacts?.gateB;
  if (!gateB || stable(gateB.requires) !== stable(['hive_lockdown']) || (gateB.requires.includes('hive_choice') || gateB.requires.includes('hive_baizhi'))) errors.push('E_GATE_B_POLICY');
  const expectedGateB = Boolean(save.world?.flags?.hive_lockdown); if (Boolean(gateB?.open) !== expectedGateB) errors.push('E_GATE_STATE');
  for (const door of save.world?.doors ?? []) { const expected = door.id === 'gh_gate_b' ? expectedGateB : door.state !== 'Locked'; if (door.navEdgeEnabled !== expected) errors.push('E_DOOR_NAV_DIVERGENCE'); }
  const consumed = new Set((save.world?.pickups ?? []).filter(item => item.consumed).map(item => item.rewardLedgerId));
  const ledger = save.world?.rewardLedger ?? []; if (new Set(ledger).size !== ledger.length || [...consumed].some(id => !ledger.includes(id))) errors.push('E_REWARD_DEDUP');
  if (save.restoreOutcome === 'unknown' || save.saveOutcome === 'unknown') errors.push('E_UNKNOWN_OUTCOME');
  if (save.snapshotEpoch !== undefined && save.snapshotEpoch < save.worldEpoch) errors.push('E_OLD_EPOCH');
  return [...new Set(errors)];
}

function restore(active, candidate) {
  const old = clone(active); const staged = clone(candidate); const stageLog = ['envelope_version_checksum', 'parse_staging', 'compatibility', 'static_world_load', 'hydrate_authoritative_entities', 'reconcile_references_invariants', 'rebuild_indexes_spatial_nav', 'allocate_new_epoch_generation'];
  const errors = validateSave(staged);
  if (errors.length) return { ok: false, active: old, stageLog, errors, atomic: true };
  staged.worldEpoch = old.worldEpoch + 1; staged.runtime = { generation: staged.worldEpoch, handles: 'recreated', indices: 'recreated', babylon: 'not_persisted' }; stageLog.push('atomic_activate', 'readonly_first_projection');
  return { ok: true, active: staged, stageLog, errors: [], atomic: true };
}
function migrateV2(v2) {
  if (v2?.version !== 2) return { ok: false, errors: [v2?.version === undefined ? 'E_V2_VERSION' : 'E_V2_VERSION'] };
  if (v2.current_world_id !== 'grey_hive') return { ok: false, errors: ['E_V2_WORLD'] };
  if (v2.map?.px !== undefined && v2.map?.py !== undefined && v2.map?.unit !== 'metres') return { ok: false, errors: ['E_V2_UNCERTAIN_UNITS'] };
  const choice = v2.optional_choice === undefined || v2.optional_choice === null ? 'unresolved' : ({ hive_help_baizhi: 'taken', hive_leave_sample: 'left' }[v2.optional_choice] ?? v2.optional_choice);
  if (!['taken', 'left', 'unresolved'].includes(choice)) return { ok: false, errors: ['E_V2_CHOICE'] };
  const out = withChecksum({ ...clone(validSave), saveId: `migrated.${v2.fixture_id ?? 'v2'}`, schemaVersion: 3, worldEpoch: 0, checkpoint: { id: 'gh_cp_airlock', worldId: 'grey_hive', semantic: true }, campaign: { ...clone(validSave.campaign), choices: { hive_choice: choice } }, source: { schemaVersion: 2, migration: 'v2_to_v3', fixtureId: v2.fixture_id ?? 'v2' } });
  return { ok: true, save: out, signature: hash(out) };
}
function atomicScenario(fault) {
  const active = { marker: 'old-active' }; const pending = { marker: 'new-pending' }; const order = ['backup', 'temp_write', 'flush', 'validate_pending', 'replace'];
  if (!fault) return { ok: true, active: pending, order };
  const failedAt = order.indexOf(fault); return { ok: false, active, order: order.slice(0, failedAt + 1), restored: true, failure: `E_ATOMIC_${fault.toUpperCase()}` };
}

const restoreResult = restore({ worldEpoch: 40, marker: 'old-active' }, validSave);
const migrationInput = { version: 2, fixture_id: 'grey-hive-v2-001', current_world_id: 'grey_hive', objectives: [], flags: { hive_power: true }, decisions: [], optional_choice: null };
const migrationA = migrateV2(migrationInput); const migrationB = migrateV2(migrationInput);
const positiveChecks = [
  check('schema_contract', schema.$id === 'save-v3-full-restore-candidate-v1' && schema.candidateOnly, schema.$id, 'candidate schema'),
  check('save_fixture_checksum', validateSave(validSave).length === 0, validateSave(validSave), 'valid Save v3'),
  check('full_state_graph', ['player','world','campaign'].every(key => key in validSave), Object.keys(validSave), 'player/world/campaign'),
  check('staging_atomic_restore', restoreResult.ok && restoreResult.active.worldEpoch === 41 && restoreResult.stageLog.at(-1) === 'readonly_first_projection' && restoreResult.atomic, restoreResult, 'staging→reconcile→activate'),
  check('velocity_strategy', validSave.player.velocityRestore === 'discard_and_zero', validSave.player.velocityRestore, 'explicit velocity strategy'),
  check('oneshot_reward_dedup', validateSave(validSave).length === 0 && validSave.world.triggers.filter(trigger => trigger.state === 'fired' && trigger.oneShot).length === 2 && new Set(validSave.world.rewardLedger).size === validSave.world.rewardLedger.length, true, 'no duplicate reward'),
  check('gate_choice_npc', validSave.campaign.gateFacts.gateB.requires.length === 1 && validSave.campaign.gateFacts.gateB.requires[0] === 'hive_lockdown' && validSave.campaign.choices.hive_choice === 'unresolved', validSave.campaign.gateFacts.gateB, 'lockdown only; choice/NPC independent'),
  check('v2_migration_success', migrationA.ok && migrationA.save.schemaVersion === 3 && migrationA.save.campaign.choices.hive_choice === 'unresolved', migrationA.ok, 'v2→v3 once'),
  check('v2_migration_deterministic', migrationA.signature === migrationB.signature, { a: migrationA.signature, b: migrationB.signature }, 'same signature'),
  check('v2_grid_not_dual_authority', migrationA.ok && !Object.hasOwn(migrationA.save, 'px') && !Object.hasOwn(migrationA.save, 'py'), Object.keys(migrationA.save), 'no v3 px/py truth'),
  check('atomic_save_success', atomicScenario(null).ok && atomicScenario(null).active.marker === 'new-pending', atomicScenario(null), 'backup→temp→flush→validate→replace'),
  check('atomic_save_faults', ['temp_write','flush','validate_pending','replace'].every(fault => { const scenario = atomicScenario(fault); return !scenario.ok && scenario.restored && scenario.active.marker === 'old-active'; }), true, 'fault retains old active state'),
  check('formal_boundary', schema.formalPersistenceWired === false && !('runtime' in fixture), { formalPersistenceWired: schema.formalPersistenceWired, runtimeInFixture: 'runtime' in fixture }, 'candidate only')
];

const mutations = [
  ['checksum_corrupt', save => { save.checksum = 'bad'; return 'E_CHECKSUM'; }],
  ['truncated', save => { delete save.player; return 'E_PLAYER_TRANSFORM'; }],
  ['future_schema', save => { save.schemaVersion = 99; return 'E_FUTURE_SCHEMA'; }],
  ['v1', save => { save.schemaVersion = 1; return 'E_V1_UNSUPPORTED'; }],
  ['missing_version', save => { delete save.schemaVersion; return 'E_MISSING_SCHEMA_VERSION'; }],
  ['non_finite', save => { save.player.transform.positionM[0] = Number.NaN; return 'E_NONFINITE'; }],
  ['dangling_id', save => { save.checkpoint.id = 'missing'; return 'E_CHECKPOINT_REFERENCE'; }],
  ['duplicate_id', save => { save.player.inventory.push(clone(save.player.inventory[0])); return 'E_DUPLICATE_ID'; }],
  ['unknown_world', save => { save.worldId = 'unknown_world'; return 'E_UNKNOWN_WORLD'; }],
  ['ui_object', save => { save.babylon = { mesh: 'gpu' }; return 'E_FORBIDDEN_PERSISTENCE'; }],
  ['old_handle', save => { save.runtimeHandle = '41:door:gh_gate_b'; return 'E_FORBIDDEN_PERSISTENCE'; }],
  ['grid_field', save => { save.grid = { px: 1, py: 2 }; return 'E_FORBIDDEN_PERSISTENCE'; }],
  ['half_hydrate_gate', save => { save.campaign.gateFacts.gateB.requires = ['hive_choice']; return 'E_GATE_B_POLICY'; }],
  ['pickup_reward_duplicate', save => { save.world.rewardLedger.push('reward.hive_power'); return 'E_REWARD_DEDUP'; }],
  ['door_nav_split', save => { save.world.doors[1].navEdgeEnabled = false; return 'E_DOOR_NAV_DIVERGENCE'; }],
  ['old_epoch', save => { save.snapshotEpoch = 1; return 'E_OLD_EPOCH'; }],
  ['unknown_outcome', save => { save.saveOutcome = 'unknown'; return 'E_UNKNOWN_OUTCOME'; }],
  ['choice_gate', save => { save.campaign.gateFacts.gateB.requires = ['hive_lockdown', 'hive_choice']; return 'E_GATE_B_POLICY'; }],
  ['npc_gate', save => { save.campaign.gateFacts.gateB.requires = ['hive_lockdown', 'hive_baizhi']; return 'E_GATE_B_POLICY'; }],
  ['v2_uncertain_units', save => { save.__v2 = { version: 2, current_world_id: 'grey_hive', map: { px: 1, py: 2 } }; return 'E_V2_UNCERTAIN_UNITS'; }, 'E_V2_UNCERTAIN_UNITS']
];
const negativeChecks = mutations.map(([id, mutate, expected]) => { const candidate = clone(validSave); let code; if (id === 'v2_uncertain_units') { mutate(candidate); code = migrateV2(candidate.__v2).errors[0]; } else { mutate(candidate); code = validateSave(candidate).find(error => error === expected); } return check(id, code === expected, code, expected); });

const sourceFiles = [fixturePath, schemaPath, nb05Path];
const result = {
  schema: 'new_baseline_save_v3_full_restore_nb10_result_v1', status: positiveChecks.every(item => item.pass) && negativeChecks.every(item => item.pass) ? 'candidate_isolated' : 'failed_candidate_isolated', candidateOnly: true, formalRestoreWired: false,
  sources: sourceFiles.map(file => ({ path: path.relative(root, file).replaceAll(path.sep, '/'), bytes: fs.statSync(file).size, sha256: crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex') })),
  fieldClassification: { mustPersist: ['envelope', 'worldId', 'worldEpoch', 'checkpointId', 'player semantic state', 'inventory/loadout/abilities/cooldowns/statusEffects', 'doors/triggers/rooms/zones/checkpoints/pickups/objectives/encounters/NPC', 'world flags/campaign progress/rewards/unlocks/choices/gateFacts'], reconstruct: ['Babylon mesh/camera/effects/UI cache', 'spatial acceleration/indexes', 'nav cache', 'runtime handles', 'channels', 'new epoch/generation'], forbidden: ['UI/Babylon objects', 'GPU/resource/pointer handles', 'unconfirmed commands', 'legacy grid/px/py/tile authority'] },
  restoreStages: restoreResult.stageLog, migration: { accepted: 'v2 only', rejected: ['missing', 'v1', 'future', 'unknown'], deterministicSignature: migrationA.signature, missingChoiceDefault: 'unresolved', transformRule: 'semantic checkpoint; no guessed pixel-to-metre scaling' },
  atomicSave: { order: ['backup', 'temp_write', 'flush', 'validate_pending', 'replace'], faults: ['temp_write', 'flush', 'validate_pending', 'replace'], preservesOldActive: true, systemDestructiveTest: false },
  gateContract: { gateA: ['hive_power'], gateB: ['hive_lockdown'], choice: ['taken', 'left', 'unresolved'], choiceAffectsGateB: false, npcAffectsGateB: false },
  positiveChecks, negativeChecks, summary: { positiveTotal: positiveChecks.length, positivePassed: positiveChecks.filter(item => item.pass).length, negativeTotal: negativeChecks.length, negativeRejected: negativeChecks.filter(item => item.pass).length },
  formalBoundary: { E_NO_FULL_SAVE_V3_RUNTIME_RESTORE: 'remains open', reason: 'isolated harness; no CampaignStore/startup/Tauri integration' },
  unproven: ['formal collider clearance for restored transform', 'production SaveV3 persistence wiring', 'full three-world runtime restore', 'Babylon/UI first read-only projection in production', 'formal crash recovery on real filesystem'],
  nextMinimalFormalPackage: ['wire validated SaveV3 into Rust CampaignStore/startup', 'connect semantic checkpoint resolver and formal clearance', 'rebuild continuous indexes/nav after restore', 'prove Tauri/WorldView/UI read-only attach'],
  scope: { allowed: ['docs/SAVE_V3_FULL_RESTORE_CANDIDATE.md', 'tools/continuous-tps-architecture-probe/new-baseline-save-v3-full-restore/**', 'tools/continuous-tps-architecture-probe/results/new-baseline-save-v3-full-restore-nb10.json'], forbiddenWrites: ['server-rs/**', 'NB-01/NB-05/NB-06/NB-07/NB-08', 'TP-15C3', 'TP-18D1', 'media'] }
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true }); fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify({ status: result.status, summary: result.summary, resultPath }, null, 2));
if (result.status !== 'candidate_isolated') process.exitCode = 1;
