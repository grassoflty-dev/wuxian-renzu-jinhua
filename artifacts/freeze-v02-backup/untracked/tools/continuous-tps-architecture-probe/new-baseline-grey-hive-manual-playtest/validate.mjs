import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const own = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-grey-hive-manual-playtest');
const planPath = path.join(root, 'docs', 'GREY_HIVE_MANUAL_PLAYTEST_PROTOCOL.md');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-grey-hive-manual-playtest-nb40.json');
const plan = fs.readFileSync(planPath, 'utf8');
const checks = [];
function check(id, pass, actual, expected) { checks.push({ id, pass, actual, expected }); }
const cohorts = ['C1 first-time player','C2 action-game familiar','C3 project-familiar','C4 accessibility/input variation'];
const sessions = ['S01','S02','S03','S04','S05','S06','S07','S08','S09','S10'];
const dimensions = ['Movement and camera','targeting','spatial orientation','objective clarity','Scan targets','Stealth visibility','route had a comfortable rhythm','checkpoint recovery','UI information density','without depending on audio alone'];
const forbidden = ['TP-18'+'D2','TP-15'+'C3','TP-18'+'D1','TP-01'+'R11'];
check('scope_grey_hive', /only `grey_hive`/.test(plan) && /no other world/.test(plan), 'Grey Hive only', true);
check('build_prereqs', /P0 automated\s+and packaged checks/.test(plan) && ['immutable build hash','content hash','restore points','moderator script'].every((x) => plan.includes(x)) && /[Cc]lean Save v3/.test(plan), 'build and clean-save prerequisites', true);
check('cohorts', cohorts.every((x) => plan.includes(x)), cohorts, 4);
check('sessions', sessions.every((x) => plan.includes(x)) && sessions.length === 10, sessions.length, 10);
check('session_fields', ['objective','setup','moderator script','prohibited hints','observation fields','completion','elapsed time','error count','death count','subjective 1–5 scores','stop condition'].every((x) => plan.includes(x)) && /hint\/help\s+count/.test(plan), 'structured session fields', true);
check('dimensions', plan.includes('Movement and camera') && plan.includes('targeting') && /where I was and which route choices/.test(plan) && plan.includes('objective clarity') && plan.includes('Scan targets') && plan.includes('Stealth visibility') && plan.includes('route had a comfortable rhythm') && plan.includes('checkpoint recovery') && plan.includes('UI information density') && plan.includes('without depending on audio alone'), dimensions, 'all experience dimensions');
check('likert', plan.includes('1–5 Likert') && plan.includes('median') && plan.includes('percentile'), 'scale and aggregation', true);
check('thresholds', ['80%','3.5/5','insufficient_sample','P0/P1 qualitative blocker'].every((x) => plan.includes(x)), 'thresholds without current data', true);
check('observation_subjective_split', /Observations are factual/.test(plan) && /subjective fields\s+are verbatim response/.test(plan) && plan.includes('observation') && plan.includes('subjective'), 'separate fields', true);
check('bias_hints_randomization', /read the script verbatim/.test(plan) && /leading adjectives/.test(plan) && /rotate session\s+order/.test(plan) && /record all hints/.test(plan) && plan.includes('prohibited hints'), 'bias/hint/order controls', true);
check('technical_boundary', ['cannot override a blocked P0','automated/packaged evidence','placeholder into an approved final asset','candidate contract'].every((x) => plan.includes(x)), 'manual cannot replace technical gate', true);
check('telemetry_separation', ['route telemetry reference','route length, FPS, tick time and state correctness come only'].every((x) => plan.includes(x)), 'telemetry separate from feeling', true);
check('issue_taxonomy', ['observation','subjective','bug','design_issue','content_issue','stability_issue','P0/P1/P2'].every((x) => plan.includes(x)), 'issue classes and IDs', true);
check('privacy', plan.includes('pseudonymous ID') && /no name, contact, face, voice/.test(plan) && /Consent covers notes/.test(plan) && /optional future\s+media separately/.test(plan), 'privacy contract', true);
check('media_flags', ['no screenshot','video','audio','model call','network upload','media read','media generation'].every((x) => plan.includes(x)), 'media prohibited', true);
check('exit_states', ['ready_for_limited_playable_review','needs_iteration','blocked_by_technical_gate','insufficient_sample','protocol_ready_not_executed'].every((x) => plan.includes(x)), 'exit states', true);
check('zero_execution', ['playtestExecuted=false','sampleCount=0','formalPlayableAccepted=false','mediaCaptured=false','mediaBytesRead=false','network=false','modelCalled=false'].every((x) => plan.includes(x)), 'not executed/no fabricated metrics', true);
check('negative_fixtures', ['starting without P0','moderator answer leakage','small sample treated as pass','human override','placeholder treated as final','missing build hash','unauthorized media capture'].every((x) => plan.includes(x)), 'negative fixtures', true);
check('route_gate_contract', ['250–350m','500–800m','hive_lockdown'].every((x) => plan.includes(x)) && /optional choice/.test(plan) && /NPC\/choice semantics\s+persist independently/.test(plan), 'route/Gate boundary', true);
check('forbidden_identifiers', !forbidden.some((x) => plan.includes(x)), 'forbidden IDs absent', true);
check('allowlist', plan.includes('docs/GREY_HIVE_MANUAL_PLAYTEST_PROTOCOL.md') && plan.includes('new-baseline-grey-hive-manual-playtest') && plan.includes('new-baseline-grey-hive-manual-playtest-nb40.json'), 'allowlisted targets', true);

const passed = checks.filter((x) => x.pass).length;
const payload = {
  schema: 'new_baseline_grey_hive_manual_playtest_nb40_v1',
  status: passed === checks.length ? 'grey_hive_manual_playtest_protocol_ready_not_executed' : 'invalid',
  conclusion: passed === checks.length ? 'grey_hive_manual_playtest_protocol_ready_not_executed' : 'invalid',
  world: 'grey_hive',
  playtestExecuted: false,
  sampleCount: 0,
  formalPlayableAccepted: false,
  mediaCaptured: false,
  mediaBytesRead: false,
  network: false,
  modelCalled: false,
  cohorts: ['C1','C2','C3','C4'],
  sessions: { total: 10, ids: sessions },
  dimensions,
  scale: 'Likert 1-5 plus not_applicable',
  thresholds: { coreObjectiveIndependentRate: 0.8, coreSessionCompletionRate: 0.8, medianScore: 3.5, cohortMedianFloor: 3, p0QualitativeBlocker: true },
  exits: ['ready_for_limited_playable_review','needs_iteration','blocked_by_technical_gate','insufficient_sample','protocol_ready_not_executed'],
  currentExit: 'protocol_ready_not_executed',
  technicalBoundary: 'manual experience cannot override automated/packaged P0',
  telemetrySubjectiveSeparated: true,
  privacy: 'pseudonymous tester IDs; no unnecessary personal data',
  checks,
  summary: { total: checks.length, passed, failed: checks.length - passed },
  rollback: 'delete only NB-40 allowlisted protocol, validator and result; no game/data/media changed'
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, JSON.stringify(payload, null, 2) + '\n');
console.log(JSON.stringify({ status: payload.status, total: checks.length, passed, failed: checks.length - passed, resultPath }, null, 2));
if (payload.status === 'invalid') process.exitCode = 1;
