import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const planPath = path.join(root, 'docs', 'THREE_WORLD_FORMAL_ROLLOUT_ACCEPTANCE.md');
const plan = fs.readFileSync(planPath, 'utf8');
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const states = ['concept_only', 'contract_ready', 'blockout_candidate', 'runtime_candidate', 'first_playable_candidate', 'formally_verified', 'release_candidate', 'released'];
const worlds = ['grey_hive', 'mist_harbor', 'clockworks'];
const forbidden = ['NB-' + '37', 'NB-' + '39', 'NB-' + '40', 'TP-' + '18D2'];

check('state_vocabulary', states.every((x) => plan.includes(x)) && plan.includes('No shared artifact may advance all worlds'), 'world state machine and independent advancement');
check('three_world_records', worlds.every((x) => plan.includes('`' + x + '`')), 'all three worlds have independent status records');
check('conservative_status', plan.includes('`grey_hive` | `blockout_candidate`') && plan.includes('`mist_harbor` | `concept_only`') && plan.includes('`clockworks` | `concept_only`'), 'Grey Hive is first candidate; others remain concept-only');
check('evidence_and_gaps', ['Evidence actually available', 'Explicitly not proven', 'formal collider/nav export', 'independent smoke'].every((x) => plan.includes(x)), 'real evidence and gaps are separated');
check('common_gates', ['Architecture Gate', 'Rust authority', 'Tauri transport', 'Save v3', 'Grid exclusion', 'Asset/license', 'Packaged smoke'].every((x) => plan.includes(x)), 'common gates defined');
check('grey_specific', ['250–350m', '500–800m', 'Gate A', 'Gate B', 'Beacon', 'Sentinel', 'gh_cp_post_bio'].every((x) => plan.includes(x)), 'Grey Hive-specific gates defined');
check('other_worlds_tbd', ['required categories, not invented layouts', 'Their room names, dimensions, routes', 'remain TBD'].every((x) => plan.includes(x)), 'Mist Harbor and Clockworks remain conservative');
check('shared_vs_specific', ['Shared-system versus world-content acceptance', 'Each world must prove separately', 'generic fallback theme'].every((x) => plan.includes(x)), 'shared shell cannot imply world completion');
check('rollout_order', ['Grey Hive formal verification', 'Mist Harbor contract', 'Clockworks contract', 'may proceed in parallel'].every((x) => plan.includes(x)), 'rollout dependencies and asset parallelism defined');
check('save_switching', ['Save v3 contains semantic campaign progression', 'old epoch inputs are rejected', 'no cross-world state leakage', 'source save'].every((x) => plan.includes(x)), 'save/campaign/cross-world switching acceptance');
check('independent_release', ['own manifest/hash', 'Grid-exclusion scan', 'rollback result', 'owner signoff'].every((x) => plan.includes(x)), 'per-world regression/rollback/release signoff');
check('future_template', ['The product is not capped at three worlds', 'world_id', 'release signoff'].every((x) => plan.includes(x)), 'future-world template prevents 3/3 cap');
check('blocked_conclusion', plan.includes('three_world_rollout_acceptance_ready_current_worlds_not_formally_verified') && plan.includes('no world is released'), 'current conclusion remains blocked');
check('no_implicit_acceptance', ['never', 'never a world release candidate', 'cannot satisfy another world'].every((x) => plan.includes(x)), 'no implicit shell/theme/placeholder acceptance');
check('no_forbidden_inputs', forbidden.every((x) => !plan.includes(x)), 'no unapproved package or task references');
check('no_media', plan.includes('asset/provenance/license') && plan.includes('No image/audio/video bytes'), 'only asset gate categories, no media operation');

const result = {
  schema: 'new_baseline_three_world_formal_rollout_acceptance_nb42_v1',
  status: checks.every((x) => x.ok) ? 'three_world_rollout_acceptance_ready_current_worlds_not_formally_verified' : 'three_world_rollout_acceptance_invalid',
  conclusion: checks.every((x) => x.ok) ? 'three_world_rollout_acceptance_ready_current_worlds_not_formally_verified' : 'three_world_rollout_acceptance_invalid',
  gatePass: false,
  worlds: {
    grey_hive: { status: 'blockout_candidate', formallyVerified: false, releaseCandidate: false },
    mist_harbor: { status: 'concept_only', formallyVerified: false, releaseCandidate: false },
    clockworks: { status: 'concept_only', formallyVerified: false, releaseCandidate: false },
  },
  commonGateCount: 7,
  independentWorldCount: 3,
  futureWorldTemplate: true,
  checks,
  scope: { formalCodeChanged: false, assetsChanged: false, ciChanged: false, mediaTouched: false, unverifiedPackagesConsumed: false },
};
const outArg = process.argv.indexOf('--out');
if (outArg >= 0 && process.argv[outArg + 1]) {
  fs.mkdirSync(path.dirname(path.resolve(process.argv[outArg + 1])), { recursive: true });
  fs.writeFileSync(path.resolve(process.argv[outArg + 1]), `${JSON.stringify(result, null, 2)}\n`);
}
if (!checks.every((x) => x.ok)) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));
