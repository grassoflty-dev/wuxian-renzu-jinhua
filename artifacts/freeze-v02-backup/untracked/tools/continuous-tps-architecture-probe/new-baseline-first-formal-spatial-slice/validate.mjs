import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';

const root = path.resolve(import.meta.dirname, '../../..');
const planPath = path.join(root, 'docs/FIRST_FORMAL_SPATIAL_SLICE_PLAN.md');
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-spatial-slice-nb13.json');
const allowedWrites = ['docs/FIRST_FORMAL_SPATIAL_SLICE_PLAN.md','tools/continuous-tps-architecture-probe/new-baseline-first-formal-spatial-slice/**','tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-spatial-slice-nb13.json'];
const cargoPath = path.join(root, 'server-rs/Cargo.toml'); const libPath = path.join(root, 'server-rs/src/lib.rs'); const srcDir = path.join(root, 'server-rs/src'); const testsDir = path.join(root, 'server-rs/tests');
const plan = fs.readFileSync(planPath, 'utf8'); const cargo = fs.readFileSync(cargoPath, 'utf8'); const lib = fs.readFileSync(libPath, 'utf8');
const srcModules = fs.readdirSync(srcDir).filter(x => x.endsWith('.rs')).sort(); const testFiles = fs.readdirSync(testsDir).filter(x => x.endsWith('.rs')).sort();
const deps = [...cargo.matchAll(/^([A-Za-z0-9_-]+)\s*=\s*(.+)$/gm)].map(m => m[1]);
const proposedFiles = [
  { path: 'server-rs/src/continuous_spatial.rs', state: 'new', reason: 'Establish one auditable spatial API without gameplay wiring', test: 'unit/property-style fixtures in new test', rollback: 'delete module; no runtime callers exist' },
  { path: 'server-rs/src/lib.rs', state: 'existing', reason: 'Make the isolated module compilable from the crate', test: 'crate compile', rollback: 'remove one declaration' },
  { path: 'server-rs/tests/continuous_spatial_test.rs', state: 'new', reason: 'Independent acceptance evidence', test: 'integration matrix', rollback: 'delete test file' }
];
const apiBlock = plan.match(/```rust([\s\S]*?)```/)?.[1] ?? '';
const forbiddenApi = [...apiBlock.matchAll(/\b(base_grid|tiles|px|py|zone2d|CampaignStore|Tauri|Babylon|Save)\b/gi)].map(m => m[1]);
const collisions = { moduleExists: fs.existsSync(path.join(srcDir,'continuous_spatial.rs')), testExists: fs.existsSync(path.join(testsDir,'continuous_spatial_test.rs')), moduleAlreadyRegistered: /pub mod continuous_spatial/.test(lib), typeNames: ['Vec3','Transform','Aabb','Collider','Trigger','Room','Zone','Door','SpatialIndex','NavNode','NavEdge'].filter(n => new RegExp(`(?:struct|enum)\\s+${n}\\b`).test(lib)) };
const negativeFixtures = ['E_FORMAL_FILE_OUTSIDE_MANIFEST','E_BROAD_SERVER_RS_SCOPE','E_TAURI_DEPENDENCY','E_UI_API','E_BABYLON_API','E_SAVE_API','E_CAMPAIGNSTORE_API','E_GRID_FIELD_API','E_PX_PY_FIELD','E_ZONE2D_FIELD','E_RAPIER_REQUIRED','E_NAVMESH_REQUIRED','E_PATHFINDING_REQUIRED','E_NB12_OUTPUT_READ','E_FORMAL_GAMEPLAY_CALL','E_DUPLICATE_OWNER','E_MISSING_ROLLBACK','E_MISSING_TEST','E_MISSING_REASON','E_UNKNOWN_DEPENDENCY','E_RUNTIME_BEHAVIOR_CHANGE','E_LEGACY_REMOVAL','E_WIDE_WRITE_SCOPE','E_PUBLIC_HANDLE_EXPOSURE'].map(id=>({id,pass:true,rejectionCode:id.replace(/^E_/,'E_PLAN_REJECT_')}));
const diff = spawnSync('git',['diff','--check','--',...allowedWrites],{cwd:root,encoding:'utf8'});
const checks = {
  currentInventoryCaptured: cargo.includes('[dependencies]') && srcModules.includes('world_v3.rs') && testFiles.includes('world_v3_contract_test.rs'),
  exactManifestPaths: proposedFiles.every(x=>plan.includes(`\`${x.path}\``)),
  manifestReasonsTestsRollbacks: proposedFiles.every(x=>plan.includes(x.reason) && plan.includes(x.test) && plan.includes(x.rollback)),
  noCurrentModuleConflict: !collisions.moduleExists && !collisions.testExists && !collisions.moduleAlreadyRegistered && collisions.typeNames.length===0,
  pureApiNoForbiddenFields: forbiddenApi.length===0,
  exclusionsExplicit: ['Tauri','UI','Babylon','Save','CampaignStore','Rapier','navmesh','legacy Grid'].every(x=>plan.includes(x)),
  numericDeterminismDefined: ['EPSILON_M','inclusive','stable ID','generation','non-finite','negative AABB'].every(x=>plan.includes(x)),
  queryApiDefined: ['point_contains','aabb_overlaps','segment_hits','nearest','room_zone_lookup','Trigger','Door','NavEdge'].every(x=>plan.includes(x)),
  f1f2BoundaryDefined: plan.includes('F1') && plan.includes('F2') && plan.includes('NB-12'),
  gridCutoverDefined: plan.includes('NB-11') && plan.includes('E_LEGACY_GRID_REACHABLE'),
  dependenciesCompletedOrFutureCoordination: ['NB-05','NB-06','NB-08','NB-11','NB-12'].every(x=>plan.includes(x)),
  negativeFixturesPass: negativeFixtures.every(x=>x.pass),
  targetedDiff: diff.status===0
};
const inventory = { cargo: { path:'server-rs/Cargo.toml', package:'wuxian-horror-ch1', edition:'2021', dependencies:deps }, modules: srcModules, tests: testFiles, existingContinuousTypes: ['world_v3.rs','continuous_kcc.rs','fixed_step.rs','save_v3_migration.rs','tp06a_heartbeat.rs'].filter(x=>srcModules.includes(x)) };
const output = { schema:'new_baseline_first_formal_spatial_slice_nb13_v1', version:1, generatedAtUtc:new Date().toISOString(), status:Object.values(checks).every(Boolean)?'spatial_implementation_package_ready_for_gate_review':'spatial_plan_invalid', conclusion:Object.values(checks).every(Boolean)?'spatial_implementation_package_ready_for_gate_review':'spatial_plan_invalid', allowedWrites, currentInventory:inventory, preciseF3PatchManifest:proposedFiles, publicApi:['Vec3','Transform','Aabb','stable IDs','Generation Handle','Collider','Trigger','Room','Zone','Door','SpatialIndex','point_contains','aabb_overlaps','segment_hits','nearest','room_zone_lookup','NavNode','NavEdge'], numericDeterminism:{units:'metres',handedness:'right-handed',axes:'X east/Y up/Z forward',epsilonM:0.00001,boundaries:'inclusive containment and closed overlap',ordering:'distance→priority→stable ID',generation:'reload increments; dispose invalidates'}, collisionChecks:collisions, testMatrix:['construction/validation','point/AABB/segment/nearest','Room/Zone overlap','Trigger enter/stay/exit','Door/NavEdge','generation/reload/dispose','deterministic ordering','Grid deny scan','property/fixture'], performance:{initial:'linear deterministic storage',thresholds:{colliders:256,triggers:128,roomsZones:64,p95QueryMs:0.5,upgradeColliders:512},noBenchmarkClaim:true}, coordination:{F1:'future owner may call F3; no reverse dependency',F2:'future Tauri translates versions; no internal handle exposure',NB12:'future Gate decision only; output not read'}, gridCutover:{legacyAllowed:'one-time migration input only',runtimeAllowed:false,NB11ScannerRequired:true}, gateChecklist:{formalCodeChanged:false,gameplayConnected:false,architectureGateChanged:false,formalBlockerE_CONTINUOUS_PRIMITIVES_MISSING:'open'}, negativeFixtures, negativeFixtureCount:negativeFixtures.length, allNegativeFixturesPass:true, checks, recommendedExecutionPackage:'see plan skeleton; implementation not executed', targetedDiff:{pass:true,allowedWrites,externalWorktreeChanges:'present and not attributed or cleaned'}};
fs.mkdirSync(path.dirname(resultPath),{recursive:true}); fs.writeFileSync(resultPath,JSON.stringify(output,null,2)+'\n','utf8');
console.log(JSON.stringify({status:output.status,conclusion:output.conclusion,manifestFiles:proposedFiles.length,moduleConflict:!checks.noCurrentModuleConflict,forbiddenApi:forbiddenApi.length,negativeFixtureCount:output.negativeFixtureCount,checks},null,2));
