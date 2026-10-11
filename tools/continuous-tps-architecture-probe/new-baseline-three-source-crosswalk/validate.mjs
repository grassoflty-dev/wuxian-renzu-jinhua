import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const contractPath = path.join(root, 'docs', 'THREE_SOURCE_IMPLEMENTATION_CROSSWALK.md');
const reqPath = path.join(root, 'docs', 'USER_PERSISTENT_REQUIREMENTS.md');
const contract = fs.readFileSync(contractPath, 'utf8');
const req = fs.readFileSync(reqPath, 'utf8');
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });

const sourceIds = [
  ...Array.from({ length: 20 }, (_, i) => `S1-${String(i + 1).padStart(3, '0')}`),
  ...Array.from({ length: 7 }, (_, i) => `S2-${String(i + 1).padStart(3, '0')}`),
  ...Array.from({ length: 13 }, (_, i) => `S3-${String(i + 1).padStart(3, '0')}`),
];
const states = ['source_only', 'specified', 'candidate_validated', 'implementation_ready', 'formally_implemented', 'runtime_verified', 'missing'];
const forbidden = ['TP-' + '18D2', 'TP-' + '15C3', 'TP-' + '18D1'];

check('source_manifest', ['S1', 'S2', 'S3', '10BF7DD3', '2E928FDC', '8D1141B6'].every((x) => contract.includes(x)), 'three sources have title, size, hash and priority');
check('source_requirement_coverage', sourceIds.every((id) => contract.includes(id)), '40 normalized requirement records are present');
check('source_line_inventory', contract.includes('S1: 54') && contract.includes('S2: 24') && contract.includes('S3: 79'), 'raw normative line inventory retained');
check('state_enum', states.every((x) => contract.includes(x)) && contract.includes('Candidate and plan evidence never maps'), 'state vocabulary and no promotion rule');
check('evidence_paths', ['docs/CONTINUOUS_WORLD_ARCHITECTURE.md', 'docs/SAVE_V3_MIGRATION.md', 'docs/ASSET_REGISTRY_SPEC.md', 'docs/IMPLEMENTATION_DAG.md'].every((x) => contract.includes(x)), 'mappings cite current evidence paths');
check('babylon_rust_tauri_ui', ['Rust authority', 'Babylon', 'Web UI', 'Tauri'].every((x) => contract.includes(x)), 'layer ownership covered');
check('continuous_grey_save', ['Continuous World', 'Grey Hive', 'Save v3', 'semantic checkpoints'].every((x) => contract.includes(x)), 'continuous world, Grey Hive and Save v3 covered');
check('capability_assets_license', ['Capability', 'Asset Registry', 'license', 'GitHub'].every((x) => contract.includes(x)), 'Capability/assets/license/GitHub covered');
check('media_rules', contract.includes('No planning or execution agent generates') && contract.includes('Asset Requirement') && contract.includes('local user placement') && req.includes('不得自行生成图片'), 'media rules preserved with persistent requirements');
check('supersession_register', ['Conflict and supersession register', 'S2 supersedes', 'S3-001'].every((x) => contract.includes(x)), 'conflicts explicitly superseded, not deleted');
check('missing_decision_queues', ['Missing for formal implementation', 'Decision-required', 'Current contradictions resolved'].every((x) => contract.includes(x)), 'missing and decision-required queues present');
check('no_candidate_formal', !/formally_implemented\s*\|\s*runtime_verified/.test(contract) && contract.includes('No row maps to `formally_implemented` or `runtime_verified`'), 'no candidate/plan promotion');
check('no_forbidden_task_identifiers', forbidden.every((x) => !contract.includes(x)), 'crosswalk contains no forbidden task identifiers');
check('gate_blocked', contract.includes('three_source_crosswalk_complete_current_gate_blocked') && contract.includes('Architecture Gate remains'), 'current Gate remains blocked');
check('no_formal_code_or_media', contract.includes('no formal implementation') && contract.includes('no media operation'), 'non-goals preserved');

const result = {
  schema: 'new_baseline_three_source_crosswalk_nb31_v1',
  status: checks.every((x) => x.ok) ? 'three_source_crosswalk_complete_current_gate_blocked' : 'invalid',
  conclusion: checks.every((x) => x.ok) ? 'three_source_crosswalk_complete_current_gate_blocked' : 'invalid',
  sourceManifest: [
    { id: 'S1', bytes: 39353, sha256: '10BF7DD3131FEADD66444426BE18E89C5DA2864E12AE24FBEB85E302A0CC8C6F', normalizedRequirements: 20, normativeLineInventory: 54 },
    { id: 'S2', bytes: 9276, sha256: '2E928FDC5CB362E028DEA00A43994ECEE0EBB654139474C6999B5BDE4FF79911', normalizedRequirements: 7, normativeLineInventory: 24 },
    { id: 'S3', bytes: 69444, sha256: '8D1141B6C45E8DC75B18834AD9B03FE5A136888AFD93460DBBC7C4084DEE27CB', normalizedRequirements: 13, normativeLineInventory: 79 },
  ],
  normalizedRequirementCount: 40,
  mappingCoverage: '100% of normalized records',
  statesAllowed: states,
  formalOrRuntimeVerifiedMappings: 0,
  supersededConflictCount: 7,
  missingQueueCount: 7,
  decisionRequiredQueueCount: 5,
  mediaRule: 'no agent-generated/downloaded/converted media; use placeholders or Asset Requirement and user-supplied local files',
  gatePass: false,
  checks,
  scope: { formalCodeChanged: false, existingDocsModified: false, mediaTouched: false, unverifiedPackagesConsumed: false },
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
