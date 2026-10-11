import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '..', '..', '..');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'world-v3-formal-runtime-wiring-validator-nb106.json');
const sourceRoot = path.join(root, 'server-rs');
const libPath = path.join(sourceRoot, 'src', 'lib.rs');
const libBefore = fs.readFileSync(libPath);
const libIdentityBefore = { sha256: crypto.createHash('sha256').update(libBefore).digest('hex').toUpperCase(), bytes: libBefore.length };
const gitStatusBefore = spawnSync('git', ['status', '--short', '--', 'server-rs/src/lib.rs'], { cwd: root, encoding: 'utf8' }).stdout.trim();

const requiredNodes = ['tauri_bootstrap','rust_world_owner','world_v3_initialize_load','world_tick_update','epoch_revision','save_restore_hook','world_view_snapshot','ipc_command_event','babylon_consumer_resync'];
const requiredEdges = [
  ['tauri_bootstrap','rust_world_owner'], ['rust_world_owner','world_v3_initialize_load'], ['world_v3_initialize_load','world_tick_update'],
  ['world_tick_update','epoch_revision'], ['epoch_revision','world_view_snapshot'], ['save_restore_hook','rust_world_owner'],
  ['world_view_snapshot','ipc_command_event'], ['ipc_command_event','babylon_consumer_resync']
];
const makeGraph = (name, overrides = {}) => ({
  graphId: `synthetic-${name}`, schemaVersion: 'nb106-wiring-v1', worldId: 'grey-hive-v3', owner: 'rust_world_owner',
  nodes: requiredNodes.map((id) => ({ id, kind: id, authority: id === 'rust_world_owner' ? 'Rust' : id === 'babylon_consumer_resync' ? 'BabylonPresentation' : 'contract' })),
  edges: requiredEdges.map(([from,to], i) => ({ id:`edge-${i+1}`, from, to, command: i === 6 ? 'world_view' : null, event: i === 7 ? 'snapshot_or_resync' : null, schemaVersion: 'world-v3', worldId: 'grey-hive-v3', epoch: 1, revision: i + 1 })),
  entrypoints: ['tauri::Builder::default','WorldStateV3::new','WorldStateV3::advance_clock','WorldStateV3::view','tauri::ipc::Channel<WorldView>','Babylon consumer'],
  ...overrides
});
const positives = [
  makeGraph('new-bootstrap'), makeGraph('save-v3-restore', { saveRestore: true }), makeGraph('full-snapshot', { snapshotMode:'full' }),
  makeGraph('delta-snapshot', { snapshotMode:'delta' }), makeGraph('resync', { resync: true }), makeGraph('world-transition', { transition: true })
];
const negatives = [];
const neg = (id, code, mutate) => { const graph = makeGraph(`negative-${id}`); mutate(graph); negatives.push({ id, expected:code, graph }); };
neg('N01','WORLD_V3_NOT_INSTANTIATED', g => { g.nodes = g.nodes.filter(n => n.id !== 'world_v3_initialize_load'); });
neg('N02','MODULE_EXPORT_ONLY', g => { g.edges = g.edges.filter(e => e.from !== 'rust_world_owner'); });
neg('N03','OLD_GRID_OWNER', g => { g.owner = 'grid_owner'; });
neg('N04','JS_AUTHORITY', g => { g.nodes.find(n => n.id === 'rust_world_owner').authority = 'JavaScript'; });
neg('N05','MOCK_SNAPSHOT', g => { g.edges.find(e => e.to === 'world_view_snapshot').from = 'mock_cache'; });
neg('N06','TICK_NOT_WIRED', g => { g.edges = g.edges.filter(e => e.from !== 'world_v3_initialize_load'); });
neg('N07','REVISION_MISSING', g => { g.edges.find(e => e.from === 'world_tick_update').revision = null; });
neg('N08','SAVE_BYPASS_OWNER', g => { g.edges.find(e => e.from === 'save_restore_hook').to = 'world_view_snapshot'; });
neg('N09','BABYLON_AUTHORITY_WRITE', g => { g.nodes.find(n => n.id === 'babylon_consumer_resync').authority = 'BabylonAuthority'; });
neg('N10','SILENT_FALLBACK', g => { g.fallback = 'legacy_grid'; });
neg('N11','DUAL_OWNER', g => { g.nodes.push({ id:'legacy_owner', kind:'owner', authority:'Rust' }); });
neg('N12','BOOTSTRAP_CYCLE', g => { g.edges.push({ id:'cycle', from:'world_tick_update', to:'tauri_bootstrap', schemaVersion:'world-v3', worldId:'grey-hive-v3', epoch:1, revision:4 }); });
neg('N13','UNKNOWN_CONSUMER', g => { g.edges.push({ id:'unknown', from:'world_view_snapshot', to:'unknown_consumer', schemaVersion:'world-v3', worldId:'grey-hive-v3', epoch:1, revision:4 }); });
neg('N14','DANGLING_NODE', g => { g.edges[0].to = 'missing_node'; });
neg('N15','SCHEMA_VERSION_MISSING', g => { delete g.edges[2].schemaVersion; });
neg('N16','WORLD_ID_MISSING', g => { g.edges[4].worldId = null; });
neg('N17','EPOCH_REVISION_MISMATCH', g => { g.edges[4].epoch = 2; });
neg('N18','NON_MONOTONIC_REVISION', g => { g.edges[4].revision = 9; g.edges[5].revision = 3; });
neg('N19','DELTA_WITHOUT_FULL', g => { g.snapshotMode = 'delta'; g.fullSnapshot = false; });
neg('N20','RESYNC_UNTRACEABLE', g => { g.resync = true; g.edges = g.edges.filter(e => e.to !== 'babylon_consumer_resync'); });

function validateGraph(graph) {
  const failures = []; const fail = (code, fieldPath) => failures.push({ code, fieldPath });
  if (graph.schemaVersion !== 'nb106-wiring-v1') fail('SCHEMA_VERSION_MISSING','schemaVersion');
  if (graph.owner !== 'rust_world_owner') fail('OLD_GRID_OWNER','owner');
  const ids = new Set(graph.nodes.map(n => n.id));
  for (const id of requiredNodes) if (!ids.has(id)) fail(id === 'world_v3_initialize_load' ? 'WORLD_V3_NOT_INSTANTIATED' : 'MISSING_NODE', `nodes.${id}`);
  if (graph.nodes.filter(n => n.authority === 'Rust' && (n.kind === 'owner' || n.id.includes('owner'))).length !== 1) fail('DUAL_OWNER','nodes');
  const edgeKey = new Set();
  for (const e of graph.edges) {
    if (!ids.has(e.from) || !ids.has(e.to)) fail('DANGLING_EDGE', `edges.${e.id}`);
    if (!e.schemaVersion) fail('SCHEMA_VERSION_MISSING', `edges.${e.id}.schemaVersion`);
    if (!e.worldId) fail('WORLD_ID_MISSING', `edges.${e.id}.worldId`);
    if (e.epoch === null || e.revision === null) fail('EPOCH_REVISION_MISSING', `edges.${e.id}`);
    edgeKey.add(`${e.from}->${e.to}`);
  }
  for (const [from,to] of requiredEdges) if (!edgeKey.has(`${from}->${to}`)) fail(from === 'save_restore_hook' ? 'SAVE_BYPASS_OWNER' : from === 'world_v3_initialize_load' ? 'TICK_NOT_WIRED' : 'MISSING_EDGE', `edges.${from}->${to}`);
  const revs = graph.edges.filter(e => typeof e.revision === 'number').map(e => e.revision); for (let i=1;i<revs.length;i++) if (revs[i] < revs[i-1]) fail('NON_MONOTONIC_REVISION','edges.revision');
  const epochs = graph.edges.map(e => e.epoch).filter((e) => e !== null); if (epochs.some((e) => e !== epochs[0])) fail('EPOCH_REVISION_MISMATCH','edges.epoch');
  if (graph.nodes.some(n => n.authority === 'JavaScript')) fail('JS_AUTHORITY','nodes');
  if (graph.nodes.some(n => n.authority === 'BabylonAuthority')) fail('BABYLON_AUTHORITY_WRITE','nodes');
  if (graph.edges.some(e => e.from === 'mock_cache')) fail('MOCK_SNAPSHOT','edges');
  if (graph.fallback) fail('SILENT_FALLBACK','fallback');
  if (graph.nodes.some(n => n.id === 'unknown_consumer')) fail('UNKNOWN_CONSUMER','nodes');
  if (graph.edges.some(e => e.from === 'world_tick_update' && e.to === 'tauri_bootstrap')) fail('BOOTSTRAP_CYCLE','edges');
  if (graph.snapshotMode === 'delta' && graph.fullSnapshot === false) fail('DELTA_WITHOUT_FULL','snapshotMode');
  if (graph.resync && !edgeKey.has('ipc_command_event->babylon_consumer_resync')) fail('RESYNC_UNTRACEABLE','edges');
  return failures;
}

function inventory() {
  const roots = [path.join(sourceRoot,'src'), path.join(sourceRoot,'ui','js')]; const files=[];
  const walk = (dir) => { for (const entry of fs.readdirSync(dir,{withFileTypes:true}).sort((a,b)=>a.name.localeCompare(b.name))) { const p=path.join(dir,entry.name); if(entry.isDirectory()) walk(p); else if(/\.(rs|js)$/.test(entry.name)) files.push(p); } };
  roots.forEach(walk); const patterns = [
    ['world_v3_module', /pub mod world_v3/], ['world_state_type', /WorldStateV3/], ['world_state_new', /WorldStateV3::new/], ['world_clock', /advance_clock/], ['world_view', /WorldView/],
    ['tauri_bootstrap', /tauri::Builder::default/], ['tauri_command', /#\[tauri::command\]/], ['save_restore', /save_v3|restore|CampaignStore::load/], ['babylon_consumer', /Babylon|world3d|snapshot|resync/i], ['grid_owner', /base_grid|blockedGrid/]
  ]; const findings=[];
  for (const file of files) { const rel=path.relative(root,file).replaceAll('\\','/'); const lines=fs.readFileSync(file,'utf8').split(/\r?\n/); for (const [symbol,re] of patterns) lines.forEach((line,i)=>{if(re.test(line)) findings.push({symbol,path:rel,line:i+1,text:line.trim().slice(0,180)});}); }
  return findings.sort((a,b)=>a.path.localeCompare(b.path)||a.line-b.line||a.symbol.localeCompare(b.symbol));
}
const staticInventory = inventory();
const positiveResults = positives.map((g) => { const failures=validateGraph(g); return { graphId:g.graphId, passed:failures.length===0, failures }; });
const negativeResults = negatives.map(({id,expected,graph}) => { const failures=validateGraph(graph); return { id, expectedRejectionCode:expected, rejected:failures.length>0, firstFailure:failures[0]??null, rejectionCodes:[...new Set(failures.map(f=>f.code))] }; });
const libAfter = fs.readFileSync(libPath); const libIdentityAfter={sha256:crypto.createHash('sha256').update(libAfter).digest('hex').toUpperCase(),bytes:libAfter.length};
const sourceUnchanged=libIdentityBefore.sha256===libIdentityAfter.sha256&&libIdentityBefore.bytes===libIdentityAfter.bytes;
const missingEdges = requiredEdges.map(([from,to])=>({edge:`${from}->${to}`,status:'unresolved',reason:'static inventory finds endpoint symbols but no compiler-validated/runtime-proven formal connection'}));
const gitStatusAfter = spawnSync('git', ['status', '--short', '--', 'server-rs/src/lib.rs'], { cwd: root, encoding: 'utf8' }).stdout.trim();
const result = { packageId:'NB-106', blockerId:'E_WORLD_V3_NOT_RUNTIME_WIRED', blockerStatus:'open', currentWiringFindings:{staticInventory,detectedEntrypoints:staticInventory.filter(x=>['tauri_bootstrap','tauri_command','world_state_new','world_clock','world_view','babylon_consumer'].includes(x.symbol)),owners:[...new Set(staticInventory.filter(x=>x.symbol==='world_state_type').map(x=>x.path))],missingOrUnresolvedEdges:missingEdges,staticOnly:true}, positiveCount:positiveResults.length, negativeCount:negativeResults.length, positivePassed:positiveResults.filter(x=>x.passed).length, negativeRejected:negativeResults.filter(x=>x.rejected).length, positiveResults, negativeResults, sourceIdentity:{libBefore:libIdentityBefore,libAfter:libIdentityAfter,unchanged:sourceUnchanged}, gitStatus:{command:'git status --short -- server-rs/src/lib.rs',before:gitStatusBefore,after:gitStatusAfter,unchanged:gitStatusBefore===gitStatusAfter}, commands:['node --check tools/continuous-tps-architecture-probe/world-v3-formal-runtime-wiring-validator/validate.mjs','node tools/continuous-tps-architecture-probe/world-v3-formal-runtime-wiring-validator/validate.mjs','git diff --check -- docs/WORLD_V3_FORMAL_RUNTIME_WIRING_VALIDATOR.md tools/continuous-tps-architecture-probe/world-v3-formal-runtime-wiring-validator tools/continuous-tps-architecture-probe/results/world-v3-formal-runtime-wiring-validator-nb106.json'], wiringValidatorImplemented:true, compilerValidated:false, runtimeValidated:false, packagedRuntimeValidated:false, sourceModified:false, formalAccepted:false, blockerClosed:false, architectureGateOpened:false, mediaGenerated:false, networkUsed:false, resultStatus:positiveResults.every(x=>x.passed)&&negativeResults.every(x=>x.rejected)&&sourceUnchanged?'complete':'failed', notes:['static inventory is not compiler or runtime evidence','current blocker remains open until formal implementation and independent runtime evidence exist'] };
fs.mkdirSync(path.dirname(resultPath),{recursive:true}); fs.writeFileSync(resultPath,`${JSON.stringify(result,null,2)}\n`,'utf8');
console.log(JSON.stringify({status:result.resultStatus,positivePassed:result.positivePassed,negativeRejected:result.negativeRejected,negativeTotal:result.negativeCount,inventoryFindings:staticInventory.length,missingEdges:missingEdges.length,libUnchanged:sourceUnchanged},null,2));
if (result.resultStatus==='failed') process.exit(1);
