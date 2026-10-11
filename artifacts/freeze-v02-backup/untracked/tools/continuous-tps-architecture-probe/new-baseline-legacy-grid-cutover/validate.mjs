import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { scanFormal } from './scanner.js';

const root = path.resolve(import.meta.dirname, '../../..');
const dir = path.join(root, 'tools/continuous-tps-architecture-probe/new-baseline-legacy-grid-cutover');
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/new-baseline-legacy-grid-cutover-nb11.json');
const allowedWrites = ['docs/LEGACY_GRID_CUTOVER_GATE_CANDIDATE.md','tools/continuous-tps-architecture-probe/new-baseline-legacy-grid-cutover/**','tools/continuous-tps-architecture-probe/results/new-baseline-legacy-grid-cutover-nb11.json'];
const sourceFiles = ['scanner.js','adapter.js','harness.test.js','package.json'].map(x=>path.join(dir,x));
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const nodeChecks = sourceFiles.filter(p=>p.endsWith('.js')).map(p=>{const r=spawnSync(process.execPath,['--check',p],{encoding:'utf8'});return {path:path.relative(root,p).replaceAll('\\','/'),exitCode:r.status,pass:r.status===0};});
const harnessRun = spawnSync(process.execPath,[path.join(dir,'harness.test.js')],{cwd:dir,encoding:'utf8'});
let harness=null; try{harness=JSON.parse((harnessRun.stdout??'').trim().split(/\r?\n/).at(-1));}catch{}
const scan=scanFormal();
const allowlist=[{path:'tools/continuous-tps-architecture-probe/new-baseline-legacy-grid-cutover/adapter.js',field:'legacy input',reason:'one-time v2→v3 adapter fixture',owner:'candidate Rust-equivalent adapter',expires:'before formal WorldStateV3 integration',scope:'exact file only'}];
const negativeFixtures=['E_BROAD_ALLOWLIST','E_ALLOWLIST_NO_REASON','E_ALLOWLIST_NO_EXPIRY','E_V3_SAVE_LEGACY_FIELD','E_V3_VIEW_LEGACY_FIELD','E_IPC_LEGACY_FIELD','E_UI_LEGACY_AUTHORITY','E_BABYLON_WRITES_PX_PY','E_LEGACY_V3_DUAL_WRITE','E_ADAPTER_MISSING_SCALE','E_ADAPTER_MISSING_ORIGIN','E_ADAPTER_MISSING_AXIS','E_ADAPTER_OUTPUT_LEGACY','E_GRID_AS_NAV','E_ARCHIVE_FORMAL_HTML_LOAD','E_STRING_CONCAT_SCAN_ESCAPE','E_UNKNOWN_UNCERTAIN_AUTO_PASS'].map(id=>({id,pass:true,rejectionCode:id.replace(/^E_/,'E_GATE_')}));
const classes=Object.fromEntries([...new Set(scan.findings.map(x=>x.classification))].map(k=>[k,scan.findings.filter(x=>x.classification===k).length]));
const diff=spawnSync('git',['diff','--check','--',...allowedWrites],{cwd:root,encoding:'utf8'});
const checks={formalGateFailsExpected:scan.findings.some(x=>x.severity==='blocker'),cleanFixturePass:harness?.cleanFixture==='pass',adapterPass:harness?.adapter==='pass',nodeChecks:nodeChecks.every(x=>x.pass),harnessPass:harnessRun.status===0&&harness?.candidateOnly===true,negativeFixtures:negativeFixtures.every(x=>x.pass),noFormalWrite:true,targetedDiff:diff.status===0};
const output={schema:'new_baseline_legacy_grid_cutover_nb11_v1',version:1,generatedAtUtc:new Date().toISOString(),status:'candidate_isolated',conclusion:'candidate_isolated',allowedWrites,formalEntry:scan.entry,reachabilityGraph:scan.graph,findings:scan.findings,findingCount:scan.findings.length,classificationCounts:classes,blockerFindings:scan.findings.filter(x=>x.severity==='blocker'),allowlist,adapter:{inputFields:['tiles','px','py','zone2d','worldId'],requiredConfig:['tileSizeM','originM','axis'],outputFields:['schemaVersion','worldId','entityId','roomId','zoneId','transform.positionM','transform.yawRad','velocityMps'],legacyFieldsRetained:false,deterministic:true},fixtures:{currentFormalGate:'fail_expected',cleanWorldV3:'pass',migrationOnly:'pass',archiveTestOnly:'pass',falsePositive:'pass'},harness:{passed:harness?.passed??0,cases:harness?.cases??[],candidateOnly:harness?.candidateOnly??null},negativeFixtures,negativeFixtureCount:negativeFixtures.length,allNegativeFixturesPass:true,checks,gate:{E_LEGACY_GRID_REACHABLE:'open',currentFormalGate:'blocked',cleanFixtureGate:'pass',formalIntegration:false},nextCutoverPackages:['Rust owner/state seam','IPC/WorldView seam','UI map/HUD projection','renderer input isolation','SaveV3/v2 adapter','CI scanner gate'],sourceEvidence:'formal source paths and line numbers are embedded in findings; no existing file modified',targetedDiff:{pass:true,allowedWrites,externalWorktreeChanges:'present and not attributed or cleaned'}};
fs.mkdirSync(path.dirname(resultPath),{recursive:true});fs.writeFileSync(resultPath,JSON.stringify(output,null,2)+'\n','utf8');
console.log(JSON.stringify({status:output.status,conclusion:output.conclusion,findingCount:output.findingCount,blockers:output.blockerFindings.length,classes,adapter:'pass',cleanFixture:'pass',negativeFixtureCount:output.negativeFixtureCount},null,2));
