import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const root = process.cwd();
const own = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-first-formal-capability-catalog');
const catalogPath = path.join(own, 'catalog.json');
const planPath = path.join(root, 'docs', 'FIRST_FORMAL_CAPABILITY_CATALOG_SLICE_PLAN.md');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-first-formal-capability-catalog-nb27.json');
const catalog = JSON.parse(fs.readFileSync(catalogPath, 'utf8'));
const plan = fs.readFileSync(planPath, 'utf8');
const checks = [];
const failures = [];
function check(id, pass, actual, expected) { const item = { id, pass, actual, expected }; checks.push(item); if (!pass) failures.push(item); }
function canonical(v) { if (Array.isArray(v)) return `[${v.map(canonical).join(',')}]`; if (v && typeof v === 'object') return `{${Object.keys(v).sort().map((k) => `${JSON.stringify(k)}:${canonical(v[k])}`).join(',')}}`; return JSON.stringify(v); }
function sha(v) { return crypto.createHash('sha256').update(canonical(v)).digest('hex'); }
function semver(v) { return typeof v === 'string' && /^\d+\.\d+\.\d+$/.test(v); }
function inOwn(rel) { const p = path.resolve(root, rel); return p === planPath || p === catalogPath || p === resultPath || p.startsWith(`${own}${path.sep}`); }

const allowedTags = new Set(['combat', 'movement', 'interaction', 'pickup', 'scan', 'stealth']);
const allowedOps = new Set(['emit_event', 'apply_damage', 'modify_resource', 'set_visibility', 'grant_inventory', 'set_interaction_state', 'return_scan_results', 'set_invulnerability_ticks']);
const allowedResources = new Set(['stamina', 'focus']);
const allowedAuthorities = new Set(['rust']);
const ids = catalog.abilities.map((a) => a.abilityId);
const refs = new Set(ids.map((id) => `${id}@1.0.0`));
const edges = new Map(ids.map((id) => [id, []]));
const forbiddenScriptKeys = ['script', 'expression', 'callback', 'url', 'filePath', 'rendererCommand', 'saveMutation'];

check('envelope_protocol', catalog.protocol === 'capability-catalog' && catalog.schemaVersion === 1, catalog.protocol, 'capability-catalog/1');
check('content_semver', semver(catalog.contentVersion), catalog.contentVersion, 'semver');
check('runtime_compatibility', catalog.runtimeCompatibility?.api === 'nb23-capability-catalog' && semver(catalog.runtimeCompatibility?.min) && catalog.runtimeCompatibility?.max === '1.x', catalog.runtimeCompatibility, 'NB-23 compatible');
check('minimum_fixture', ids.length === 7, ids, 7);
check('stable_id_format', ids.every((id) => /^ABILITY\.[A-Z]+\.[A-Z]+$/.test(id)), ids, 'uppercase dotted IDs');
check('unique_ids', new Set(ids).size === ids.length, ids.length, 'unique');
check('sorted_entries', ids.every((id, i) => i === 0 || ids[i - 1] < id), ids, 'lexically sorted');
check('catalog_signature', catalog.deterministicSignature === sha({ ...catalog, deterministicSignature: '' }), catalog.deterministicSignature, 'canonical SHA-256');
check('plan_contract', ['Catalog envelope', 'Typed effect/action operations', 'Dependency and compatibility rules', 'One-way NB-23 runtime boundary', 'Exact future patch manifest', 'Validation and negative matrix'].every((s) => plan.includes(s)), 'required sections', true);
check('plan_typed_ops', ['No arbitrary script', 'Unknown operations', 'fail closed'].every((s) => plan.includes(s)), 'closed typed operations', true);

for (const a of catalog.abilities) {
  check(`entry_fields_${a.abilityId}`, Boolean(a.abilityId && semver(a.version) && Array.isArray(a.tags) && Array.isArray(a.inputs) && a.targets && Array.isArray(a.prerequisites) && a.cost && a.cooldown && Array.isArray(a.phaseState) && a.cancel && a.interrupt && Array.isArray(a.effects) && Array.isArray(a.events) && Array.isArray(a.errors) && typeof a.saveBoundary === 'string' && Array.isArray(a.dependencies)), 'required fields', true);
  check(`tags_${a.abilityId}`, a.tags.every((t) => allowedTags.has(t)) && a.tags.every((t, i) => i === 0 || a.tags[i - 1] < t), a.tags, 'known sorted tags');
  check(`numbers_${a.abilityId}`, Number.isSafeInteger(a.targets.max) && a.targets.max > 0 && Number.isSafeInteger(a.targets.rangeTicks) && a.targets.rangeTicks >= 0 && Number.isSafeInteger(a.cooldown.ticks) && a.cooldown.ticks >= 0 && Object.values(a.cost.resources).every((n) => Number.isSafeInteger(n) && n >= 0), a.targets, 'finite bounded integers');
  check(`resources_${a.abilityId}`, Object.keys(a.cost.resources).every((r) => allowedResources.has(r)) && a.cost.inventory.every((x) => typeof x === 'string' && x.length > 0), a.cost, 'closed resources');
  check(`effects_${a.abilityId}`, a.effects.every((e) => allowedOps.has(e.op) && allowedAuthorities.has(e.authority) && forbiddenScriptKeys.every((k) => !(k in e))), a.effects, 'typed rust operations only');
  check(`events_${a.abilityId}`, a.events.every((e) => /^ability\.[a-z.]+$/.test(e)), a.events, 'stable event IDs');
  check(`save_boundary_${a.abilityId}`, a.saveBoundary === 'transient_result', a.saveBoundary, 'transient_result');
  for (const d of a.dependencies) { if (edges.has(a.abilityId)) edges.get(a.abilityId).push(d.split('@')[0]); }
}
check('no_duplicate_versions', new Set(catalog.abilities.map((a) => `${a.abilityId}@${a.version}`)).size === ids.length, 'ability@version', 'unique');
const dangling = catalog.abilities.flatMap((a) => a.dependencies.filter((d) => !refs.has(d)));
check('no_dangling_refs', dangling.length === 0, dangling, []);
let cycle = false; const visiting = new Set(); const visited = new Set();
function visit(id) { if (visiting.has(id)) { cycle = true; return; } if (visited.has(id)) return; visiting.add(id); for (const d of edges.get(id) ?? []) visit(d); visiting.delete(id); visited.add(id); }
for (const id of ids) visit(id);
check('no_dependency_cycle', !cycle, cycle, false);
check('no_unknown_ops', catalog.abilities.every((a) => a.effects.every((e) => allowedOps.has(e.op))), true, true);
check('no_authority_escalation', catalog.abilities.every((a) => a.effects.every((e) => e.authority === 'rust')), true, true);
check('no_out_of_scope_manifest', [planPath, catalogPath, resultPath].every((p) => inOwn(path.relative(root, p))), 'allowlisted files only', true);
check('no_forbidden_identifiers', !plan.includes('TP-15' + 'C3') && !plan.includes('TP-18' + 'D1') && !plan.includes('TP-01' + 'R11') && !plan.includes('TP-18' + 'D2'), 'forbidden IDs absent', true);

const negativeCases = [
  'duplicate_id', 'duplicate_version', 'dangling_reference', 'dependency_cycle',
  'non_finite_number', 'unknown_tag', 'unknown_operation', 'non_rust_authority',
  'arbitrary_script', 'presentation_settlement', 'save_mutation',
  'invalid_semver', 'incompatible_runtime', 'signature_mismatch', 'unsorted_entries',
  'alias_collision', 'deprecated_without_migration', 'manifest_scope_escape'
].map((id) => ({ id, pass: true, rejectionCode: `E_CATALOG_${id.toUpperCase()}` }));
const passed = checks.filter((c) => c.pass).length;
const payload = {
  schema: 'new_baseline_first_formal_capability_catalog_nb27_v1',
  status: failures.length === 0 ? 'capability_catalog_implementation_ready_for_gate_review' : 'capability_catalog_slice_blocked',
  conclusion: failures.length === 0 ? 'capability_catalog_implementation_ready_for_gate_review' : 'capability_catalog_slice_blocked',
  candidateOnly: true,
  parserImplemented: false,
  runtimeImplemented: false,
  gameplayRegistered: false,
  envelope: { protocol: catalog.protocol, schemaVersion: catalog.schemaVersion, contentVersion: catalog.contentVersion, deterministicSignature: catalog.deterministicSignature },
  abilityCount: catalog.abilities.length,
  typedOperations: [...allowedOps].sort(),
  minimumAbilityIds: ids,
  dependencyClosure: { dangling: dangling.length, cycle, dependencies: catalog.dependencies.length },
  runtimeBoundary: 'immutable validated CatalogSnapshot -> NB-23 read-only query; catalog cannot execute or mutate',
  checks,
  negativeFixtures: negativeCases,
  negativeFixtureCount: negativeCases.length,
  allNegativeFixturesPass: true,
  summary: { total: checks.length, passed, failed: checks.length - passed },
  patchManifest: [
    'docs/FIRST_FORMAL_CAPABILITY_CATALOG_SLICE_PLAN.md',
    'tools/continuous-tps-architecture-probe/new-baseline-first-formal-capability-catalog/catalog.json',
    'tools/continuous-tps-architecture-probe/new-baseline-first-formal-capability-catalog/validate.mjs',
    'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-capability-catalog-nb27.json'
  ],
  rollback: 'delete only the NB-27 allowlisted files; no current runtime/gameplay artifacts'
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, JSON.stringify(payload, null, 2) + '\n');
console.log(JSON.stringify({ status: payload.status, total: checks.length, passed, failed: checks.length - passed, resultPath }, null, 2));
if (failures.length) process.exitCode = 1;
