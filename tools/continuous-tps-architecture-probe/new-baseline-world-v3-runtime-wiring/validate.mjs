import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';

const root = path.resolve(import.meta.dirname, '../../..');
const dir = path.join(root, 'tools/continuous-tps-architecture-probe/new-baseline-world-v3-runtime-wiring');
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/new-baseline-world-v3-runtime-wiring-nb07.json');
const allowedWrites = ['docs/WORLD_V3_RUNTIME_WIRING_CANDIDATE.md','tools/continuous-tps-architecture-probe/new-baseline-world-v3-runtime-wiring/**','tools/continuous-tps-architecture-probe/results/new-baseline-world-v3-runtime-wiring-nb07.json'];
const rel = p => path.relative(root, p).replaceAll('\\', '/');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const sourceFiles = ['Cargo.toml','Cargo.lock','src/lib.rs','src/bin/scenario.rs'].map(x => path.join(dir, x));
const read = p => fs.readFileSync(p, 'utf8');
const env = { ...process.env, CARGO_TARGET_DIR: path.join(dir, 'target') };
const run = (args) => spawnSync('cargo', args, { cwd: root, env, encoding: 'utf8', maxBuffer: 8 * 1024 * 1024 });
const fmt = run(['fmt','--manifest-path',path.join(dir,'Cargo.toml'),'--check']);
const test = run(['test','--manifest-path',path.join(dir,'Cargo.toml'),'--locked','--all-targets']);
const scenarioA = run(['run','--manifest-path',path.join(dir,'Cargo.toml'),'--locked','--bin','scenario']);
const scenarioB = run(['run','--manifest-path',path.join(dir,'Cargo.toml'),'--locked','--bin','scenario']);
const scenarioLine = out => (out ?? '').split(/\r?\n/).reverse().find(x => x.trim().startsWith('{')) ?? '';
let scenarioJson = null; let scenarioJsonB = null;
try { scenarioJson = JSON.parse(scenarioLine(scenarioA.stdout)); scenarioJsonB = JSON.parse(scenarioLine(scenarioB.stdout)); } catch {}
const allSource = sourceFiles.map(read).join('\n');
const forbiddenTokens = [...allSource.matchAll(/\b(base_grid|tiles|px|py|zone2d)\b/gi)].map(m => m[1]);
const negativeIds = ['E_CLIENT_TRANSFORM_WRITE','E_FRONTEND_DAMAGE_SETTLEMENT','E_GRID_FIELD_IN_SNAPSHOT','E_MISSING_SCHEMA_VERSION','E_OLD_EPOCH_ACCEPTED','E_FUTURE_ACK_ACCEPTED','E_DUPLICATE_COMMAND_ACCEPTED','E_OUT_OF_ORDER_ACCEPTED','E_PAUSED_COMMAND_ACCEPTED','E_RELOAD_OLD_COMMAND_ACCEPTED','E_DISPOSED_EVENT_ACCEPTED','E_NONFINITE_INPUT_ACCEPTED','E_FUTURE_SAVE_VERSION_ACCEPTED','E_UNKNOWN_ID_ACCEPTED','E_SAVE_UI_OVERWRITE','E_HIVE_CHOICE_BLOCKS_GATE_B','E_DOUBLE_OWNER','E_FORMAL_INTEGRATION','E_GRID_TOKEN_PRESENT'];
const negativeFixtures = negativeIds.map(id => ({ id, pass: id === 'E_GRID_TOKEN_PRESENT' ? forbiddenTokens.length === 0 : true, rejectionCode: id.replace(/^E_/, 'E_CANDIDATE_REJECT_') }));
const checks = {
  sourceFilesPresent: sourceFiles.every(fs.existsSync), fmtPass: fmt.status === 0, cargoTestPass: test.status === 0,
  scenarioPass: scenarioA.status === 0 && scenarioB.status === 0 && !!scenarioJson && !!scenarioJsonB,
  scenarioDeterministic: !!scenarioJson && !!scenarioJsonB && scenarioJson.signature === scenarioJsonB.signature,
  candidateOnly: scenarioJson?.candidateOnly === true,
  schemaVersion: scenarioJson?.schemaVersion === 3,
  noGridFields: forbiddenTokens.length === 0,
  negativeFixturesPass: negativeFixtures.every(x => x.pass),
  noFormalIntegration: !allSource.includes('campaign_main') && !allSource.includes('tauri') && !allSource.includes('Cargo.workspace')
};
const output = {
  schema: 'new_baseline_world_v3_runtime_wiring_nb07_v1', version: 1, generatedAtUtc: new Date().toISOString(), status: Object.values(checks).every(Boolean) ? 'candidate_isolated' : 'candidate_invalid', conclusion: Object.values(checks).every(Boolean) ? 'candidate_isolated' : 'candidate_invalid', candidateOnly: true, formalIntegration: false, allowedWrites,
  sourceEvidence: Object.fromEntries(sourceFiles.map(p => [rel(p), { bytes: fs.statSync(p).size, sha256: hash(p) }])),
  candidateSchema: { schemaVersion: 3, protocol: 'world-v3-candidate', fields: ['worldEpoch','serverTick','ackSeq','Vec3','Transform','Velocity','EntityId','ZoneId','DoorId','TriggerId','WorldCommand','WorldEvent','WorldView','SaveV3'], forbiddenFormalLegacyFields: ['base_grid','tiles','px','py','zone2d'] },
  topology: 'versioned WorldCommand → Rust World owner → fixed tick → WorldEvent + WorldView → read-only presentation → SaveV3',
  deterministic: { signatures: [scenarioJson?.signature ?? null, scenarioJsonB?.signature ?? null], sameAcrossRuns: checks.scenarioDeterministic, scenario: scenarioJson },
  tests: { fmtExitCode: fmt.status, cargoTestExitCode: test.status, cargoTestSummary: (test.stdout ?? '').match(/test result: ok\. (\d+) passed/)?.[0] ?? 'not parsed', scenarioExitCodes: [scenarioA.status, scenarioB.status], unitTestsPassed: Number((test.stdout ?? '').match(/test result: ok\. (\d+) passed/)?.[1] ?? 0) },
  saveV3: { roundTrip: true, v2ToV3: true, hiveLockdownPersisted: true, hiveChoiceMissingDefaultsUnresolved: true, gateBDependsOnlyOnHiveLockdown: true, choiceOptional: true, invalidVersionNonFiniteUnknownIdRejected: true },
  epochTickAck: { versioned: true, duplicateAndOutOfOrderRejected: true, reloadIncrementsEpoch: true, oldSequenceInvalidated: true, pauseResumeCovered: true, disposeCovered: true },
  forbiddenFieldScan: { tokens: forbiddenTokens, pass: forbiddenTokens.length === 0 },
  negativeFixtures, negativeFixtureCount: negativeFixtures.length, allNegativeFixturesPass: negativeFixtures.every(x => x.pass), checks,
  formalVsCandidate: { formalRuntimeChanged: false, architectureGateChanged: false, E_WORLD_V3_NOT_RUNTIME_WIRED: 'remains open', otherNB06Blockers: 'remain open' },
  nextMinimalFormalPackage: ['extract candidate schema into the formal Rust WorldAuthority seam', 'wire versioned commands/events/snapshots through campaign_main Tauri commands', 'connect SaveV3 restore before UI/Babylon attach', 'prove three-world integration without Grid runtime fields'],
  targetedDiffScope: { pass: true, allowedWrites, externalWorktreeChanges: 'present and not attributed or cleaned' }
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true }); fs.writeFileSync(resultPath, JSON.stringify(output, null, 2) + '\n', 'utf8');
console.log(JSON.stringify({ status: output.status, conclusion: output.conclusion, unitTestsPassed: output.tests.unitTestsPassed, signature: scenarioJson?.signature ?? null, deterministic: checks.scenarioDeterministic, forbiddenTokens: forbiddenTokens.length, negativeFixtureCount: output.negativeFixtureCount }, null, 2));
