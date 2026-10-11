import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const plan = fs.readFileSync(path.join(root, 'docs', 'NEW_BASELINE_ARTIFACT_CONTRACT_AUDIT.md'), 'utf8');
const audit = JSON.parse(fs.readFileSync(path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-artifact-contract-audit', 'audit.json'), 'utf8'));
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const allowedClassifications = new Set(['confirmed_conforming','report_path_mismatch','scope_mismatch','missing_artifact','duplicate_or_orphan_artifact','unverified_report','interrupted_or_denylisted','not_enough_evidence']);
const requiredFields = ['stableAuditId','packageId','assignedAllowlist','reportedPaths','observedPaths','expectedResultPath','actualResultPath','validatorOutputTarget','classification','evidence','violations','remediationRequired','gateEffect','status','originalReportVerified','scopeConformance','pathConformance','schemaConformance'];
const packageIds = Array.from({ length: 51 }, (_, i) => `NB-${String(i + 1).padStart(2, '0')}`);
const records = audit.packages;
const byId = (id) => records.find((x) => x.packageId === id);

check('package_coverage', packageIds.every((id) => Boolean(byId(id))) && records.filter((x) => packageIds.includes(x.packageId)).length === 51, 'NB-01 through NB-51 registered exactly once');
check('package_id_unique', records.map((x) => x.packageId).length === new Set(records.map((x) => x.packageId)).size && records.every((x) => packageIds.includes(x.packageId) || ['NB-15R1','NB-18R1'].includes(x.packageId)), 'unique package IDs');
check('contract_fields', records.every((x) => requiredFields.every((k) => Object.hasOwn(x, k))), 'artifact contract fields');
check('classification_valid', records.every((x) => allowedClassifications.has(x.classification)), 'classification vocabulary');
check('mismatch_nb49', byId('NB-49').classification === 'report_path_mismatch' && byId('NB-49').expectedResultPath !== byId('NB-49').actualResultPath && byId('NB-49').assignedAllowlist.includes(byId('NB-49').expectedResultPath) && byId('NB-49').reportedPaths.includes(byId('NB-49').actualResultPath), 'NB-49 assigned/reported mismatch preserved');
check('mismatch_nb50', byId('NB-50').classification === 'report_path_mismatch' && byId('NB-50').expectedResultPath !== byId('NB-50').actualResultPath && byId('NB-50').assignedAllowlist.includes(byId('NB-50').expectedResultPath) && byId('NB-50').reportedPaths.includes(byId('NB-50').actualResultPath), 'NB-50 assigned/reported mismatch preserved');
check('mismatch_not_normalized', byId('NB-49').expectedResultPath !== byId('NB-49').actualResultPath && byId('NB-50').expectedResultPath !== byId('NB-50').actualResultPath && plan.includes('不自动归一化'), 'path spelling is compared literally');
check('quarantine', ['NB-22','NB-27','NB-31','NB-37','NB-42'].every((id) => byId(id).classification === 'unverified_report' && byId(id).originalReportVerified === false && byId(id).status === 'quarantined'), 'unverified packages remain quarantined');
check('nb15_boundary', byId('NB-15').classification === 'scope_mismatch' && byId('NB-15R1').classification === 'unverified_report' && plan.includes('NB-15') && plan.includes('NB-15R1'), 'NB-15 historical scope risk retained');
check('nb18_boundary', ['NB-18','NB-18R1'].every((id) => byId(id).originalReportVerified === false) && plan.includes('transient old output name'), 'NB-18 path repair not promoted');
check('denylist', audit.denylist.includes('TP-15C3') && audit.denylist.includes('TP-18D1') && audit.denylist.includes('TP-01R11') && audit.nonexistent.includes('TP-18D2') && !audit.evidenceReads.includes('TP-15C3') && !audit.evidenceReads.includes('TP-18D1') && !audit.evidenceReads.includes('TP-01R11'), 'denylisted/interrupted inputs not consumed');
check('not_conforming_unverified', records.filter((x) => !x.originalReportVerified).every((x) => x.classification !== 'confirmed_conforming'), 'unverified cannot be conforming');
check('gate_unchanged', records.every((x) => x.gateEffect === 'unchanged') && plan.includes('不改变任何 gate 结论'), 'gate effect unchanged');
check('flags', Object.values(audit.flags).every((x) => x === false), 'historical/runtime/media/network flags false');
check('forbidden_scope', !plan.includes('TP-18D2 内容') && plan.includes('TP-18D2 是 nonexistent') && !plan.includes('读取 TP-15C3'), 'forbidden content not consumed');

const counts = Object.fromEntries([...allowedClassifications].map((c) => [c, records.filter((x) => x.classification === c).length]));
const result = {
  schema: 'new_baseline_artifact_contract_audit_nb52_v1',
  status: 'new_baseline_artifact_contract_audit_complete_gate_unchanged',
  conclusion: 'new_baseline_artifact_contract_audit_complete_gate_unchanged',
  artifactContractAccepted: false,
  historicalFilesModified: false,
  runtimeExecuted: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaModelCalled: false,
  network: false,
  packageCount: records.length,
  classificationCounts: counts,
  remediationQueueCount: audit.remediationQueue.length,
  checks,
};
process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
if (checks.some((x) => !x.ok)) process.exitCode = 1;
