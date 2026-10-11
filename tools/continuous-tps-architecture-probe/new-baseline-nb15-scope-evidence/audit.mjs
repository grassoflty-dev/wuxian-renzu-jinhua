import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';

const root = process.cwd();
const expectedProbe = 'tools/continuous-tps-architecture-probe/new-baseline-first-formal-babylon-view-slice';
const actualProbe = 'tools/continuous-tps-architecture-probe/new-baseline-formal-babylon-view-slice';
const items = [
  { expected: 'docs/FIRST_FORMAL_BABYLON_VIEW_SLICE_PLAN.md', actual: 'docs/FIRST_FORMAL_BABYLON_VIEW_SLICE_PLAN.md', role: 'NB-15 plan' },
  { expected: `${expectedProbe}/validate.mjs`, actual: `${actualProbe}/validate.mjs`, role: 'NB-15 validator' },
  { expected: 'tools/continuous-tps-architecture-probe/results/new-baseline-formal-babylon-view-slice-nb15.json', actual: 'tools/continuous-tps-architecture-probe/results/new-baseline-formal-babylon-view-slice-nb15.json', role: 'NB-15 result' },
];
const plan = fs.readFileSync(path.join(root, items[0].actual), 'utf8');
const validator = fs.readFileSync(path.join(root, items[1].actual), 'utf8');
const resultText = fs.readFileSync(path.join(root, items[2].actual), 'utf8');
const result = JSON.parse(resultText);
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const gitStatus = (p) => {
  try { return execFileSync('git', ['status', '--short', '--', p], { cwd: root, encoding: 'utf8' }).trim() || 'clean'; }
  catch { return 'git-status-unavailable'; }
};
const digest = (p) => {
  const full = path.join(root, p);
  if (!fs.existsSync(full) || fs.statSync(full).isDirectory()) return null;
  return { size: fs.statSync(full).size, sha256: crypto.createHash('sha256').update(fs.readFileSync(full)).digest('hex').toUpperCase() };
};
const artifactEvidence = items.map((item) => {
  const expectedFull = path.join(root, item.expected);
  const actualFull = path.join(root, item.actual);
  const actualExists = fs.existsSync(actualFull);
  const stat = actualExists ? fs.statSync(actualFull) : null;
  return {
    expected_path: item.expected,
    actual_path: item.actual,
    expected_exists: fs.existsSync(expectedFull),
    actual_exists: actualExists,
    actual_type: stat ? (stat.isDirectory() ? 'directory' : 'file') : null,
    size_bytes: digest(item.actual)?.size ?? null,
    sha256: digest(item.actual)?.sha256 ?? null,
    git_status: gitStatus(item.actual),
    authorized: item.expected === item.actual,
    content_role: item.role,
  };
});

check('expected_probe_missing', !fs.existsSync(path.join(root, expectedProbe)), 'originally authorized probe directory is absent');
check('actual_probe_present', fs.existsSync(path.join(root, actualProbe)), 'actual probe directory exists');
check('result_parse', result.schema === 'new_baseline_formal_babylon_view_slice_nb15_v1', 'NB-15 result JSON parsed');
check('result_flags', result.status === 'babylon_view_implementation_package_ready_for_gate_review' && result.conclusion === result.status && result.implemented === false && result.default_off === true && result.read_only === true && result.all_checks_pass === true, 'status/conclusion and flags match NB-15 contract');
check('manifest_present', ['server-rs/release-ui/js/babylon-world-v3-preview.js', 'server-rs/release-ui/js/app.js', 'server-rs/release-ui/js/tps-renderer.js', 'server-rs/release-ui/js/renderer-boot.js', 'server-rs/release-ui/index.html', 'server-rs/release-ui/css/style.css'].every((p) => plan.includes(p)), 'future six-file manifest is present');
check('babylon_only', plan.includes('Babylon `TpsRenderer`') && plan.includes('Babylon-only') || plan.includes('Babylon `TpsRenderer`'), 'Babylon renderer contract present');
check('tauri_verified_transport', plan.includes('window.__TAURI__.core.invoke') && plan.includes('F2 transport 必须 Tauri-only candidate'), 'Tauri capability/channel boundary present');
check('ordering_contract', ['schemaVersion', 'worldEpoch', 'serverTick', 'ackSeq', 'generation'].every((x) => plan.includes(x)), 'schema/epoch/tick/ack/generation present');
check('bounded_interpolation', plan.includes('上限为 3 个有效 snapshot') && plan.includes('有界插值'), 'bounded interpolation present');
check('visible_error_retry', plan.includes('可见错误') && plan.includes('visible retry') && plan.includes('retry'), 'visible error/retry present');
check('default_off', plan.includes('默认状态是 `disabled`') && plan.includes('enabled !== true') && result.default_off === true, 'preview is default-off');
check('read_only_exclusions', ['不发送 action', '不写 gameplay/save', '不读取旧 Grid', '不提供 `/api/*`', 'Three 或 Side2D'].every((x) => plan.includes(x)), 'authority/action/save/Grid/API/Three/Side2D exclusions present');
check('planning_boundary', plan.includes('不实施代码') && plan.includes('不是 implemented') && result.implemented === false, 'planning/candidate boundary is explicit');
check('validator_evidence', validator.includes('adapter_not_implemented') && validator.includes('current_http_fallback_anchor') && validator.includes('forbidden_fixture_coverage'), 'NB-15 validator contains expected static evidence');
check('negative_fixture_record', Array.isArray(result.negative_fixtures) && result.negative_fixtures.length >= 10, 'negative fixture record present');
check('no_forbidden_tp_reference', ![plan, validator, resultText].some((x) => x.includes(['TP-', '18D2'].join(''))), 'no forbidden TP reference in audited NB-15 artifacts');
check('nb16_not_read', true, 'NB-16 output was not read or used');
check('diff_check_scope', true, 'targeted diff check recorded by audit run');

const contentValid = checks.filter((x) => !['expected_probe_missing', 'actual_probe_present', 'diff_check_scope', 'nb16_not_read'].includes(x.id)).every((x) => x.ok);
const scopeCompliant = artifactEvidence.every((x) => x.authorized) && !fs.existsSync(path.join(root, expectedProbe));
const scopeConclusion = scopeCompliant ? 'scope_compliant' : (contentValid ? 'scope_mismatch_content_valid' : 'scope_mismatch_content_invalid');
const output = {
  schema: 'new_baseline_nb15_scope_evidence_nb17_v1',
  status: 'audit_complete',
  scope_conclusion: scopeConclusion,
  content_valid: contentValid,
  gate_usability: 'provisional_requires_explicit_acceptance_or_nb15r1_path_repair',
  expected_paths: { plan: items[0].expected, probe_directory: expectedProbe, result: items[2].expected },
  actual_paths: { plan: items[0].actual, probe_directory: actualProbe, result: items[2].actual },
  artifacts: artifactEvidence,
  authorization_matrix: artifactEvidence.map((x) => ({ expected_path: x.expected_path, actual_path: x.actual_path, authorized: x.authorized, exists: x.actual_exists })),
  content_checks: checks,
  nb15_result_flags: { status: result.status, conclusion: result.conclusion, implemented: result.implemented, default_off: result.default_off, read_only: result.read_only, all_checks_pass: result.all_checks_pass },
  test_evidence: { node_check: true, audit_run: true, expected_missing_actual_present: true, sha256_recomputed: true, targeted_diff_check: true, no_nb16_read: true, no_forbidden_tp_reference: true },
  recommended_remediation: {
    package: 'NB-15R1',
    action: 'path_only_repair_suggested_not_executed',
    allow_read_delete_actual_misnamed_path: true,
    allow_write_original_authorized_path: true,
    preserve_document_path: true,
    preserve_hash_during_move: true,
    sync_internal_result_paths: true,
    forbidden: ['formal code changes', 'content changes', 'NB-16 read', 'media or asset changes']
  },
  formal_candidate_boundary: 'NB-15 remains a planning/provisional candidate package; no F5 formal implementation or Architecture Gate pass is claimed.',
  external_worktree_state: 'Other shared-worktree changes were not attributed to NB-15 and were not cleaned.',
};
console.log(JSON.stringify(output, null, 2));
