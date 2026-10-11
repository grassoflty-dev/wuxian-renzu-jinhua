import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const ledgerPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-formal-runtime-gate-readiness', 'ledger.json');
const ledger = JSON.parse(fs.readFileSync(ledgerPath, 'utf8'));
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const requiredBlockers = ['E_LEGACY_GRID_REACHABLE','E_WORLD_V3_NOT_RUNTIME_WIRED','E_CONTINUOUS_PRIMITIVES_MISSING','E_API_FALLBACK_UNBACKED','E_NO_FULL_SAVE_V3_RUNTIME_RESTORE'];
const quarantine = ['NB-15R1','NB-22','NB-27','NB-31','NB-37','NB-42'];
const denyIds = ['TP-' + '15C3','TP-' + '18D1','TP-' + '01R11','TP-' + '18D2'];
const levels = ['planning_complete','static_validation_passed','candidate_implementation_exists','formal_runtime_wired','packaged_runtime_executed','manual_acceptance_complete'];

check('package_count', ledger.packages.length >= 25, `package count ${ledger.packages.length}`);
check('required_package_range', ['NB-04','NB-15R1','NB-22','NB-31','NB-37','NB-42','NB-44','NB-47'].every((x) => ledger.packages.some((p) => p.packageId === x)), 'NB-04 through NB-47 relevant packages represented');
check('evidence_levels', levels.every((x) => ledger.evidenceLevels.some((l) => l.id === x && Number.isInteger(l.rank))), 'six ordered evidence levels present');
check('no_level_promotion', ledger.packages.every((p) => !((p.artifactStatus === 'unverified-report' || p.artifactStatus === 'quarantined_unverified_report') && (p.runtimeImplemented || p.runtimeExecuted))), 'unverified/quarantined rows cannot claim runtime');
check('quarantine', quarantine.every((x) => ledger.quarantine.some((q) => q.packageId === x && q.gateEffect === 'none')) && ledger.quarantine.length === 6, 'six quarantined packages have no Gate effect');
check('denylist', denyIds.every((x) => ledger.denylist.some((d) => d.id === x && d.read === false && d.gateEffect === 'none')), 'interrupted outputs and forbidden marker denylisted');
check('blockers_open', requiredBlockers.every((x) => ledger.blockers.some((b) => b.id === x && b.status === 'open')) && ledger.blockers.length === 5, 'all five blockers remain open');
check('babylon_not_gate', ledger.packages.some((p) => p.packageId === 'NB-44' && p.formalGateEffect === 'verified_but_gate_blocked') && ledger.currentDecision.includes('blocked'), 'verified renderer evidence cannot close runtime blockers');
check('dag_nodes', ['F1','F2','F3','F4','F5','F6','F7','GREY_HIVE_ACCEPTANCE','THREE_WORLD_CAMPAIGN_ACCEPTANCE'].every((x) => ledger.gateDag.nodes.some((n) => n.id === x)), 'F1-F7 and world acceptance nodes present');
const nodeIds = new Set(ledger.gateDag.nodes.map((n) => n.id));
check('dag_edges_closed', ledger.gateDag.edges.every(([a,b]) => nodeIds.has(a) && nodeIds.has(b)), 'all DAG edges reference nodes');
check('dag_acyclic', !ledger.gateDag.edges.some(([a,b]) => a === b) && ledger.gateDag.edges.every(([a,b]) => !ledger.gateDag.edges.some(([c,d]) => c === b && d === a)), 'DAG has no self-edge or direct cycle');
check('entry_criteria', ['engineeringDecidable','originalReportVerification','userDecision','packagedExecution'].every((x) => Array.isArray(ledger.entryCriteria[x]) && ledger.entryCriteria[x].length > 0), 'first-code entry criteria separated by decision type');
check('fixed_decision', ledger.currentDecision === 'gate_blocked_evidence_and_runtime_closure_required', 'current decision fixed');
check('runtime_flags_false', ledger.formalImplementationGateOpen === false && ledger.formalRuntimeAccepted === false && ledger.runtimeExecuted === false, 'implementation/runtime flags false');
check('media_network_flags_false', ledger.mediaBytesRead === false && ledger.mediaCreated === false && ledger.mediaModelCalled === false && ledger.network === false, 'media/model/network flags false');

const result = {
  schema: 'new_baseline_formal_runtime_gate_readiness_nb47_v1',
  status: checks.every((x) => x.ok) ? 'formal_runtime_gate_readiness_ledger_valid_gate_blocked' : 'formal_runtime_gate_readiness_ledger_invalid',
  conclusion: checks.every((x) => x.ok) ? 'formal_runtime_gate_readiness_ledger_valid_gate_blocked' : 'formal_runtime_gate_readiness_ledger_invalid',
  packageCount: ledger.packages.length,
  verifiedReportCount: ledger.packages.filter((p) => p.artifactStatus === 'verified-report').length,
  unverifiedReportCount: ledger.packages.filter((p) => p.artifactStatus === 'unverified-report').length,
  quarantineCount: ledger.quarantine.length,
  blockerCount: ledger.blockers.length,
  dagNodeCount: ledger.gateDag.nodes.length,
  dagEdgeCount: ledger.gateDag.edges.length,
  currentDecision: ledger.currentDecision,
  formalImplementationGateOpen: ledger.formalImplementationGateOpen,
  formalRuntimeAccepted: ledger.formalRuntimeAccepted,
  runtimeExecuted: ledger.runtimeExecuted,
  mediaBytesRead: ledger.mediaBytesRead,
  mediaCreated: ledger.mediaCreated,
  mediaModelCalled: ledger.mediaModelCalled,
  network: ledger.network,
  checks,
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
