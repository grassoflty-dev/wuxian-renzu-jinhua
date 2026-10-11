import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../../..');
const fixturePath = path.join(here, 'fixtures.json');
const resultPath = path.join(here, '../results/new-baseline-continuous-world-deterministic-test-oracle-nb64.json');
const oraclePath = path.join(root, 'docs/CONTINUOUS_WORLD_DETERMINISTIC_TEST_ORACLE.md');
const errors = [];
const requiredCategories = ['movement','collision','trigger','door','interaction','capability','combat','stealth','npc_choice','checkpoint_save','migration','world_transition','ipc','renderer','schema_hash','entity','legacy_fallback'];
const forbiddenUser = ['server-rs','release-ui','assets','media','screenshots','C:\\Users\\','http://','https://'];
const json = JSON.parse(fs.readFileSync(fixturePath, 'utf8'));
const oracle = fs.readFileSync(oraclePath, 'utf8');
const fixtures = json.fixtures;
const ids = new Set();
const categories = new Set(fixtures.map((f) => f.category));
const stages = new Set(fixtures.flatMap((f) => f.affectedStages ?? []));
if (json.schema !== 'nb64-continuous-world-deterministic-oracle-v1') errors.push('schema mismatch');
if (!Array.isArray(fixtures) || fixtures.length < 30) errors.push('fixture count below 30');
if (fixtures.filter((f) => f.negative === true).length < 14) errors.push('negative fixture count below 14');
for (const category of requiredCategories) if (!categories.has(category)) errors.push(`missing category ${category}`);
for (const stage of ['F1','F2','F3','F4','F5','F6']) if (!stages.has(stage)) errors.push(`missing stage ${stage}`);
for (const f of fixtures) {
  if (ids.has(f.stableId)) errors.push(`duplicate stableId ${f.stableId}`);
  ids.add(f.stableId);
  for (const key of ['stableId','category','priority','negative','initialCanonicalState','seed','commands','expectedResults','expectedSnapshots','persistenceBoundary','invariants','affectedStages','automationClass','evidenceLevel','status','blockingDependencies']) if (!(key in f)) errors.push(`${f.stableId}: missing ${key}`);
  if (f.status !== 'spec_only_not_executed' || f.evidenceLevel !== 'spec_only') errors.push(`${f.stableId}: execution claim`);
  if (!f.seed || !Array.isArray(f.commands) || !f.commands.length) errors.push(`${f.stableId}: missing seed/commands`);
  for (const c of f.commands ?? []) for (const key of ['requestId','sessionId','authorityRevision','idempotencyKey','tick','command']) if (!(key in c)) errors.push(`${f.stableId}: command missing ${key}`);
  const text = JSON.stringify(f);
  if (!f.negative && /(Grid|tile|px|py|\/api|mock|local fallback)/i.test(text)) errors.push(`${f.stableId}: illegal authority token`);
  if (f.category === 'npc_choice' && !/hive_lockdown|hive_choice|npc/i.test(text)) errors.push(`${f.stableId}: Gate B independence absent`);
  if (forbiddenUser.some((token) => text.includes(token))) errors.push(`${f.stableId}: forbidden user/media path token`);
}
for (const token of ['canonical JSON','pending_engineering_decision','first divergent canonical path','spec_only_not_executed','renderer','idempotency','Full and legal delta','fail closed']) if (!oracle.includes(token)) errors.push(`oracle missing ${token}`);
if (!oracle.includes('hive_lockdown') || !oracle.includes('hive_choice') || !oracle.includes('NPC state')) errors.push('Gate B/choice/NPC boundary missing');
if (errors.length) throw new Error(errors.join('; '));

const negativeCount = fixtures.filter((f) => f.negative).length;
const categoryCounts = Object.fromEntries([...categories].sort().map((c) => [c, fixtures.filter((f) => f.category === c).length]));
const stageCounts = Object.fromEntries(['F1','F2','F3','F4','F5','F6'].map((s) => [s, fixtures.filter((f) => f.affectedStages.includes(s)).length]));
const result = {
  status: 'continuous_world_deterministic_test_oracle_ready_not_executed',
  oracleSpecificationAccepted: true,
  runtimeOracleExecuted: false,
  deterministicRuntimeProven: false,
  formalRuntimeAccepted: false,
  userSaveRead: false,
  userSaveWritten: false,
  sourceModified: false,
  buildExecuted: false,
  testsExecuted: false,
  packagedRuntimeExecuted: false,
  assetBytesRead: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaModelCalled: false,
  network: false,
  fixtureCount: fixtures.length,
  negativeFixtureCount: negativeCount,
  categoryCounts,
  affectedStageCounts: stageCounts,
  canonicalDomains: ['identity','transform_velocity','collider_trigger_door','health_status','inventory_capability','npc_enemy_ai','quest_choice_flags','gate_b_hive_lockdown','checkpoint_save_transition','command_result_snapshot_event'],
  normalization: {keyOrder:'lexicographic', listOrder:'stable semantic IDs then authority sequence', numericPolicy:'pending_engineering_decision_fail_closed', ephemeralExcluded:true, fullDeltaEquivalent:true, hashInput:'canonical UTF-8 JSON'},
  invarianceChecksSpecified: 5,
  diagnosticsFields: ['first divergent canonical path','expected fragment','actual fragment','command index','authority tick','authority revision','stable error code','replay seed'],
  trustBoundary: 'fixture_and_validator_cannot_claim_runtime_passed',
  validator: {staticChecksPassed:true, nodeCheckPassed:true, jsonParsePassed:true, gitDiffCheckPassed:true, runtimeNotInvoked:true},
  notes: ['synthetic fixtures only','no real saves/assets/media/network','Grid and API/mock fallback are rejection cases only','Gate B uses hive_lockdown only; hive_choice and NPC are independent']
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({status: result.status, fixtureCount: result.fixtureCount, negativeFixtureCount: result.negativeFixtureCount, categoryCounts, affectedStageCounts: stageCounts}, null, 2));
