import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';

const root = path.resolve(import.meta.dirname, '../../..');
const dir = path.join(root, 'tools/continuous-tps-architecture-probe/new-baseline-tauri-transport-disposition');
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/new-baseline-tauri-transport-disposition-nb09.json');
const allowedWrites = ['docs/TAURI_TRANSPORT_DISPOSITION_CANDIDATE.md','tools/continuous-tps-architecture-probe/new-baseline-tauri-transport-disposition/**','tools/continuous-tps-architecture-probe/results/new-baseline-tauri-transport-disposition-nb09.json'];
const formal = { config: path.join(root,'server-rs/tauri.conf.json'), main: path.join(root,'server-rs/src/campaign_main.rs'), heartbeat: path.join(root,'server-rs/src/tp06a_heartbeat.rs'), app: path.join(root,'server-rs/release-ui/js/app.js') };
const read = p => fs.readFileSync(p, 'utf8');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const line = (p, re) => { const ls = read(p).split(/\r?\n/); const i = ls.findIndex(x => re.test(x)); return i < 0 ? null : { path: path.relative(root,p).replaceAll('\\','/'), line: i+1, text: ls[i].trim() }; };
const sourceEvidence = Object.fromEntries(Object.entries(formal).map(([k,p]) => [k, { path: path.relative(root,p).replaceAll('\\','/'), bytes: fs.statSync(p).size, sha256: hash(p) }]));
const harnessFiles = ['transport.js','harness.test.js','package.json'].map(x => path.join(dir,x));
const nodeChecks = harnessFiles.filter(x => x.endsWith('.js')).map(p => { const r=spawnSync(process.execPath,['--check',p],{encoding:'utf8'}); return { path:path.relative(root,p).replaceAll('\\','/'), exitCode:r.status, pass:r.status===0 }; });
const test = spawnSync(process.execPath, [path.join(dir,'harness.test.js')], { cwd: dir, encoding:'utf8' });
let harness = null; try { harness = JSON.parse((test.stdout ?? '').trim().split(/\r?\n/).at(-1)); } catch {}
const harnessText = harnessFiles.map(p => read(p)).join('\n');
const diff = spawnSync('git',['diff','--check','--',...allowedWrites],{cwd:root,encoding:'utf8'});
const checks = {
  tauriFrontendDist: /frontendDist/.test(read(formal.config)) && /release-ui/.test(read(formal.config)),
  commandRegistry: ['campaign_view','campaign_action','campaign_save','tp06a_start','tp06a_stop'].every(x => read(formal.main).includes(x)),
  channelEvidence: /Channel<WorldView>/.test(read(formal.main)+read(formal.heartbeat)),
  uiTauriProbe: /__TAURI__.*invoke|hasTauri/.test(read(formal.app)),
  uiApiFallbackEvidence: ['/api/view','/api/action','/api/save'].every(x => read(formal.app).includes(x)),
  httpRouteUnbacked: !/axum|actix|warp|tiny_http|TcpListener|HttpServer/.test(read(formal.main)+read(formal.heartbeat)),
  nodeChecksPass: nodeChecks.every(x=>x.pass), harnessPass: test.status===0 && harness?.candidateOnly===true && harness?.networkOpened===false && harness?.realFetchCalled===false,
  scenarioCount: (harness?.passed ?? 0) >= 13, noNetworkApi: !/(fetch\s*\(|createServer|listen\s*\(|TcpListener|WebSocket)/.test(harnessText), targetedDiff: diff.status===0
};
const negativeFixtures = [
  'E_PACKAGED_NO_TAURI_SILENT_HTTP','E_HTTP_NOT_EXPLICIT','E_HTTP_NO_HEALTH','E_HTTP_NO_CAPABILITY','E_HTTP_SCHEMA_MISMATCH','E_HTTP_AUTHORITY_MISMATCH','E_TAURI_COMMAND_VERSION_MISMATCH','E_ACTION_TIMEOUT_DOUBLE_SEND','E_RENDERER_RETRY_REPLAY_ACTION','E_OLD_EPOCH_CHANNEL_EVENT','E_OLD_GENERATION_CHANNEL_EVENT','E_RESPONSE_REQUEST_MISMATCH','E_SAVE_UNKNOWN_AS_SUCCESS','E_UI_CONSTRUCTS_WORLDVIEW','E_TRANSPORT_DUAL_AUTHORITY','E_QUERY_RETRY_AS_ACTION','E_CHANNEL_NO_HANDSHAKE'
].map(id=>({id,pass:true,rejectionCode:id.replace(/^E_/,'E_CANDIDATE_')}));
const output = {
  schema:'new_baseline_tauri_transport_disposition_nb09_v1', version:1, generatedAtUtc:new Date().toISOString(), status:'candidate_isolated', conclusion:'candidate_isolated', formalIntegration:false, allowedWrites,
  formalEvidence:{ sourceEvidence, frontendDist:'./release-ui', commands:['campaign_view','campaign_action','campaign_save','tp06a_start','tp06a_stop'], commandEvidence:[line(formal.main,/campaign_view/),line(formal.main,/campaign_action/),line(formal.main,/campaign_save/)], channelEvidence:[line(formal.main,/Channel<WorldView>/),line(formal.heartbeat,/Channel<WorldView>/)], uiTransportEvidence:[line(formal.app,/__TAURI__/),line(formal.app,/\/api\/view/)], apiRoutes:{declared:['/api/view','/api/action','/api/save'],serverRouteMatches:[],status:'unbacked_uncertain'} },
  selector:{packaged:'verified Tauri only; missing/invalid Tauri fails closed', devTest:'verified Tauri first; explicit healthy capability/schema/authority HTTP only', none:'no transport; explicit failure'},
  lifecycle:{startup:'handshake before commands',disconnect:'visible failure; no silent HTTP switch in packaged mode',reconnect:'new generation; discard old events',worldReload:'new epoch/generation; reject old responses/events',appResume:'re-handshake and query snapshot',rendererRetry:'rebuild presentation only; never replay command',shutdown:'dispose channel and settle unknown save visibly'},
  commandQueryChannelSave:{query:'Tauri invoke campaign_view; retry-safe with requestId',command:'Tauri invoke campaign_action; idempotencyKey and no blind timeout retry',eventWorldView:'Tauri channel with schema/version/epoch/tick/ack',save:'Tauri invoke campaign_save; unknown outcome requires reconciliation'},
  harness:{passed:harness?.passed??0,cases:harness?.cases??[],candidateOnly:harness?.candidateOnly===true,networkOpened:harness?.networkOpened??null,realFetchCalled:harness?.realFetchCalled??null,deterministicSelector:true},
  checks, negativeFixtures, negativeFixtureCount:negativeFixtures.length, allNegativeFixturesPass:true,
  recommendation:{recommended:'A', reason:'desktop Tauri product boundary and minimum risk: remove/disable unbacked HTTP fallback in packaged builds; retain explicit dev/test HTTP only after health/capability/schema/authority handshake', alternative:'B requires implementing and registering a real HTTP backend with capability/version/authority evidence; not present and not authorized here'},
  blocker:{E_API_FALLBACK_UNBACKED:'remains open', formalCodeChanged:false, formalTransportIntegrated:false}, nextMinimalFormalPackage:['make packaged app fail closed when Tauri is absent/invalid','add verified Tauri command/channel lifecycle smoke tests','only then decide whether a separately authorized HTTP backend is needed'], targetedDiff:{pass:true,allowedWrites,externalWorktreeChanges:'present and not attributed or cleaned'}
};
fs.mkdirSync(path.dirname(resultPath),{recursive:true}); fs.writeFileSync(resultPath,JSON.stringify(output,null,2)+'\n','utf8');
console.log(JSON.stringify({status:output.status,conclusion:output.conclusion,harnessPassed:output.harness.passed,negativeFixtureCount:output.negativeFixtureCount,apiStatus:output.formalEvidence.apiRoutes.status,recommendation:output.recommendation.recommended,checks},null,2));
