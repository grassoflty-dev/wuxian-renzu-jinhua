import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '..', '..', '..');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'babylon-formal-render-consumption-boundary-validator-nb109.json');
const libPath = path.join(root, 'server-rs', 'src', 'lib.rs');
const libBeforeBytes = fs.readFileSync(libPath);
const libBefore = { sha256: crypto.createHash('sha256').update(libBeforeBytes).digest('hex').toUpperCase(), bytes: libBeforeBytes.length };
const gitBefore = spawnSync('git',['status','--short','--','server-rs/src/lib.rs'],{cwd:root,encoding:'utf8'}).stdout.trim();

const nodeIds = ['html_entry','babylon_engine_scene','snapshot_adapter','entity_mesh_camera_hud_projection','render_loop','resync_dispose'];
const edgePairs = [['html_entry','babylon_engine_scene'],['babylon_engine_scene','snapshot_adapter'],['snapshot_adapter','entity_mesh_camera_hud_projection'],['entity_mesh_camera_hud_projection','render_loop'],['render_loop','resync_dispose']];
const graph = (id, o={}) => ({ graphId:`synthetic-${id}`, schemaVersion:'nb109-render-boundary-v1', activeRenderer:'Babylon', backendWiringResolved:false, snapshot:{worldId:'grey-hive-v3',epoch:1,tick:10,revision:10,mode:'full',identity:'snapshot-hash-1'}, nodes:nodeIds.map(id=>({id,kind:id,authority:id==='snapshot_adapter'?'RustView':'BabylonPresentation'})), edges:edgePairs.map(([from,to],i)=>({id:`edge-${i+1}`,from,to,schemaVersion:'world-view-v3',worldId:'grey-hive-v3',epoch:1,tick:10,revision:10})), fullSnapshot:true, deltaAfterFull:true, resync:true, dispose:true, unknownEntity:'reject', ...o });
const positives = [graph('full'),graph('delta',{snapshot:{worldId:'grey-hive-v3',epoch:1,tick:10,revision:10,mode:'delta',identity:'snapshot-hash-1'}}),graph('resync'),graph('scene-reset',{sceneReset:true}),graph('dispose',{dispose:true}),graph('hud-projection',{hud:true})];
const negatives=[]; const neg=(id,code,mutate)=>{const g=graph(`negative-${id}`);mutate(g);negatives.push({id,expected:code,graph:g});};
neg('N01','THREE_RENDERER_REACHABLE',g=>{g.activeRenderer='Three';});
neg('N02','SIDE2D_RENDERER_REACHABLE',g=>{g.activeRenderer='Side2D';});
neg('N03','CANVAS_FALLBACK_REACHABLE',g=>{g.legacyFallback='Canvas';});
neg('N04','JS_MOVEMENT_AUTHORITY',g=>{g.jsAuthority='movement';});
neg('N05','JS_COLLISION_AUTHORITY',g=>{g.jsAuthority='collision';});
neg('N06','JS_AI_AUTHORITY',g=>{g.jsAuthority='ai';});
neg('N07','JS_WORLD_TRANSITION_AUTHORITY',g=>{g.jsAuthority='worldTransition';});
neg('N08','JS_SAVE_RESTORE_AUTHORITY',g=>{g.jsAuthority='saveRestore';});
neg('N09','FAKE_REVISION',g=>{g.snapshot.revision=99;});
neg('N10','FAKE_TICK',g=>{g.snapshot.tick=99;});
neg('N11','MOCK_SNAPSHOT',g=>{g.snapshot.source='mock';});
neg('N12','STALE_SNAPSHOT',g=>{g.snapshot.stale=true;});
neg('N13','DELTA_GAP',g=>{g.snapshot.mode='delta';g.deltaAfterFull=false;});
neg('N14','RESYNC_MISSING',g=>{g.resync=false;});
neg('N15','DUPLICATE_SNAPSHOT',g=>{g.snapshot.duplicate=true;});
neg('N16','UNKNOWN_ENTITY_ACCEPTED',g=>{g.unknownEntity='accept';});
neg('N17','UNKNOWN_COMPONENT_ACCEPTED',g=>{g.unknownComponent='accept';});
neg('N18','SCENE_NOT_DISPOSED',g=>{g.dispose=false;});
neg('N19','DOUBLE_RENDERER',g=>{g.renderers=['Babylon','Three'];});
neg('N20','UNKNOWN_CONSUMER',g=>{g.edges.push({id:'unknown',from:'snapshot_adapter',to:'unknown_consumer',schemaVersion:'world-view-v3',worldId:'grey-hive-v3',epoch:1,tick:11,revision:11});});
neg('N21','SNAPSHOT_IDENTITY_MISMATCH',g=>{g.edges[2].revision=99;});
neg('N22','BACKEND_WIRING_UNRESOLVED',g=>{g.backendWiringResolved=true;});

function validate(g){
  const f=[];const fail=(code,fieldPath)=>f.push({code,fieldPath});
  if(g.schemaVersion!=='nb109-render-boundary-v1')fail('SCHEMA_MISMATCH','schemaVersion');
  if(g.activeRenderer!=='Babylon')fail(`${g.activeRenderer?.toUpperCase()||'UNKNOWN'}_RENDERER_REACHABLE`,'activeRenderer');
  if(g.legacyFallback)fail('CANVAS_FALLBACK_REACHABLE','legacyFallback');
  if(g.jsAuthority)fail(`JS_${String(g.jsAuthority).toUpperCase()}_AUTHORITY`,'jsAuthority');
  const ids=new Set(g.nodes.map(n=>n.id));for(const n of nodeIds)if(!ids.has(n))fail('MISSING_RENDER_NODE',`nodes.${n}`);
  const keys=new Set();for(const e of g.edges){if(!ids.has(e.from)||!ids.has(e.to))fail('UNKNOWN_CONSUMER',`edges.${e.id}`);if(!e.schemaVersion)fail('SCHEMA_MISMATCH',`edges.${e.id}.schemaVersion`);if(!e.worldId)fail('WORLD_ID_MISSING',`edges.${e.id}.worldId`);keys.add(`${e.from}->${e.to}`);}
  for(const [a,b] of edgePairs)if(!keys.has(`${a}->${b}`))fail('UNRESOLVED_RENDER_EDGE',`${a}->${b}`);
  const s=g.snapshot;if(!s||s.worldId!=='grey-hive-v3')fail('WORLD_ID_MISMATCH','snapshot.worldId');if(!Number.isInteger(s?.epoch)||!Number.isInteger(s?.tick)||!Number.isInteger(s?.revision))fail('SNAPSHOT_IDENTITY_MISSING','snapshot');if(s?.source==='mock')fail('MOCK_SNAPSHOT','snapshot.source');if(s?.stale)fail('STALE_SNAPSHOT','snapshot.stale');if(s?.duplicate)fail('DUPLICATE_SNAPSHOT','snapshot.duplicate');if(s?.revision!==s?.tick)fail('FAKE_REVISION_OR_TICK','snapshot');
  if(s?.mode==='delta'&&!g.deltaAfterFull)fail('DELTA_GAP','deltaAfterFull');if(!g.resync)fail('RESYNC_MISSING','resync');if(g.unknownEntity==='accept')fail('UNKNOWN_ENTITY_ACCEPTED','unknownEntity');if(g.unknownComponent==='accept')fail('UNKNOWN_COMPONENT_ACCEPTED','unknownComponent');if(!g.dispose)fail('SCENE_NOT_DISPOSED','dispose');if(Array.isArray(g.renderers)&&new Set(g.renderers).size>1)fail('DOUBLE_RENDERER','renderers');if(g.edges.some(e=>e.revision!==s.revision&&e.to==='entity_mesh_camera_hud_projection'))fail('SNAPSHOT_IDENTITY_MISMATCH','edges');if(g.backendWiringResolved)fail('BACKEND_WIRING_UNRESOLVED','backendWiringResolved');return f;
}

function scan(){const files=[];const dirs=[path.join(root,'server-rs','ui')];const walk=d=>{for(const e of fs.readdirSync(d,{withFileTypes:true}).sort((a,b)=>a.name.localeCompare(b.name))){const p=path.join(d,e.name);if(e.isDirectory())walk(p);else if(/\.(html|js)$/.test(e.name))files.push(p);}};dirs.forEach(walk);const rules=[['html_entry',/index\.html$/],['babylon_import',/babylon|BABYLON/],['babylon_usage',/Babylon/],['three_import',/three\.min\.js|THREE/],['side2d',/world2d|zone2d|World2D/],['canvas_renderer',/getContext\(["']2d|createElement\(["']canvas/],['snapshot_consumer',/snapshot|WorldView|worldId|revision/],['resync',/resync|stale|gap/],['render_loop',/requestAnimationFrame|render\(/],['authority_like_write',/(?:revision|tick|epoch|collision|aiState|worldTransition|saveRestore)\s*[+\-]?=/],['legacy_fallback',/World3D\s*=\s*window\.World25D\s*\|\|\s*window\.World2D|fallbackApi\s*=\s*window\.World2D/]];const out=[];for(const file of files){const rel=path.relative(root,file).replaceAll('\\','/');const lines=fs.readFileSync(file,'utf8').split(/\r?\n/);for(const [symbol,re] of rules)lines.forEach((text,i)=>{if(re.test(text))out.push({symbol,path:rel,line:i+1,text:text.trim().slice(0,180)});});}return out.sort((a,b)=>a.path.localeCompare(b.path)||a.line-b.line||a.symbol.localeCompare(b.symbol));}
const staticInventory=scan();const positiveResults=positives.map(g=>{const failures=validate(g);return{graphId:g.graphId,passed:!failures.length,failures};});const negativeResults=negatives.map(({id,expected,graph})=>{const failures=validate(graph);return{id,expectedRejectionCode:expected,rejected:failures.length>0,firstFailure:failures[0]??null,rejectionCodes:[...new Set(failures.map(x=>x.code))]};});
const libAfterBytes=fs.readFileSync(libPath);const libAfter={sha256:crypto.createHash('sha256').update(libAfterBytes).digest('hex').toUpperCase(),bytes:libAfterBytes.length};const gitAfter=spawnSync('git',['status','--short','--','server-rs/src/lib.rs'],{cwd:root,encoding:'utf8'}).stdout.trim();
const babylonFormalEntryDetected=staticInventory.some(x=>x.symbol==='babylon_import'||x.symbol==='babylon_usage');const legacyFormalFallbackDetected=staticInventory.some(x=>x.symbol==='legacy_fallback')||staticInventory.some(x=>x.symbol==='three_import');const jsAuthorityDetected=staticInventory.some(x=>x.symbol==='authority_like_write');
const result={packageId:'NB-109',positiveCount:positives.length,negativeCount:negatives.length,positivePassed:positiveResults.filter(x=>x.passed).length,negativeRejected:negativeResults.filter(x=>x.rejected).length,positiveResults,negativeResults,staticInventory,scanConclusion:{babylonFormalEntryDetected,legacyFormalFallbackDetected,jsAuthorityDetected,backendWiringResolved:false,fullGameSwitchedToBabylon:false},babylonFormalEntryDetected,legacyFormalFallbackDetected,jsAuthorityDetected,backendWiringResolved:false,fullGameSwitchedToBabylon:false,renderBoundaryValidatorImplemented:true,compilerValidated:false,runtimeValidated:false,packagedRuntimeValidated:false,sourceModified:false,formalAccepted:false,blockerClosed:false,architectureGateOpened:false,mediaGenerated:false,networkUsed:false,sourceIdentity:{libBefore,libAfter,unchanged:libBefore.sha256===libAfter.sha256&&libBefore.bytes===libAfter.bytes},gitStatus:{command:'git status --short -- server-rs/src/lib.rs',before:gitBefore,after:gitAfter,unchanged:gitBefore===gitAfter},commands:['node --check tools/continuous-tps-architecture-probe/babylon-formal-render-consumption-boundary-validator/validate.mjs','node tools/continuous-tps-architecture-probe/babylon-formal-render-consumption-boundary-validator/validate.mjs','git diff --check -- docs/BABYLON_FORMAL_RENDER_CONSUMPTION_BOUNDARY_VALIDATOR.md tools/continuous-tps-architecture-probe/babylon-formal-render-consumption-boundary-validator tools/continuous-tps-architecture-probe/results/babylon-formal-render-consumption-boundary-validator-nb109.json'],resultStatus:positiveResults.every(x=>x.passed)&&negativeResults.every(x=>x.rejected)&&libBefore.sha256===libAfter.sha256?'complete':'failed',notes:['static inventory is not compiler or runtime evidence','Babylon entry status does not imply full game switch','backend World v3 wiring remains unresolved']};
fs.mkdirSync(path.dirname(resultPath),{recursive:true});fs.writeFileSync(resultPath,`${JSON.stringify(result,null,2)}\n`,'utf8');console.log(JSON.stringify({status:result.resultStatus,positivePassed:result.positivePassed,negativeRejected:result.negativeRejected,inventory:staticInventory.length,babylonFormalEntryDetected,legacyFormalFallbackDetected,jsAuthorityDetected,libUnchanged:result.sourceIdentity.unchanged},null,2));if(result.resultStatus==='failed')process.exit(1);
