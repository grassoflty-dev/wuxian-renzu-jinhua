import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const LABEL='SOURCE_CONTRACT_NOT_POWERSHELL_GDSCRIPT_OR_RUNTIME';
const root=new URL('../../',import.meta.url);
const G='apps/godot_playable/tests/world/foundry_short_faces_owned_observation.gd';
const L='tools/foundry-short-faces-observation/launch.ps1';
const T='tools/foundry-short-faces-observation/contract.test.mjs';
const S='server-rs/tests/fixtures/earned-cw-foundry-entry-v6.json';
const C='apps/godot_playable/data/scenes/cw_entry_foundry.json';
const P='apps/godot_playable/scripts/bridge/protocol.gd';
const permitted=new Set([G,L,T,S,C,P]);
const read=p=>{assert(permitted.has(p));return readFileSync(new URL(p,root),'utf8');};
const source=Object.fromEntries([...permitted].map(p=>[p,read(p)]));

function lex(s,ps=false){
  const code=s.split(''),raw=s.split(''),strings=[];
  const blank=(a,b,both)=>{for(let k=a;k<b;k++)if(s[k]!=='\n'&&s[k]!=='\r'){code[k]=' ';if(both)raw[k]=' ';}};
  for(let i=0;i<s.length;){
    if(ps&&s.startsWith('<#',i)){const j=s.indexOf('#>',i+2);assert(j>=0,'unterminated block comment');blank(i,j+2,true);i=j+2;continue;}
    if(s[i]==='#'){let j=s.indexOf('\n',i);if(j<0)j=s.length;blank(i,j,true);i=j;continue;}
    if(s[i]==="'"||s[i]==='"'){
      const a=i,q=s[i++];let value='',closed=false;
      if(ps){assert.equal(q,"'",'controller uses single-quoted literals');assert.notEqual(s[a-1],'@');}
      assert(!s.startsWith(q.repeat(3),a),'no triple-quoted literals');
      while(i<s.length){
        if(s[i]===q){if(ps&&s[i+1]===q){value+=q;i+=2;continue;}i++;closed=true;break;}
        if(!ps&&s[i]==='\\'){assert(i+1<s.length);value+=s[i+1];i+=2;}else value+=s[i++];
      }
      assert(closed,'unterminated literal');strings.push({a,b:i,value});blank(a,i,false);continue;
    }
    i++;
  }
  return {code:code.join(''),raw:raw.join(''),strings};
}
function end(code,a){
  const stack=[],pairs={')':'(',']':'[','}':'{'};
  for(let i=a;i<code.length;i++){const c=code[i];if('([{'.includes(c))stack.push(c);else if(')]}'.includes(c)){assert.equal(stack.pop(),pairs[c],'balanced source');if(!stack.length)return i+1;}}
  assert.fail('unclosed source block');
}
const ps=lex(source[L],true),gd=lex(source[G]);
function psFunctions(){
  const result=new Map();
  for(const m of ps.code.matchAll(/^function\s+([\w-]+)\s*\{/gmi)){const a=ps.code.indexOf('{',m.index),b=end(ps.code,a);assert(!result.has(m[1]));result.set(m[1],{a,b,code:ps.code.slice(a,b),raw:ps.raw.slice(a,b)});}
  return result;
}
const pf=psFunctions();
function p(name){assert(pf.has(name),`missing ${name}`);return pf.get(name);}
function g(name){
  const starts=[...gd.code.matchAll(/^func\s+(\w+)\s*\(/gm)],i=starts.findIndex(m=>m[1]===name);assert(i>=0,`missing ${name}`);
  const a=starts[i].index,b=starts[i+1]?.index??gd.code.length;return {a,b,code:gd.code.slice(a,b),raw:gd.raw.slice(a,b)};
}
function has(body,token){const i=body.raw.indexOf(token);assert(i>=0,`missing ${token}`);return i;}
function order(body,...tokens){const at=tokens.map(t=>has(body,t));for(let i=1;i<at.length;i++)assert(at[i]>at[i-1],tokens.join(' -> '));}
function calls(body,name){return [...body.code.matchAll(new RegExp(`\\b${name}\\b`,'g'))].length;}
function reachable(body,seen=new Set()){for(const name of pf.keys())if(calls(body,name)&&!seen.has(name)){seen.add(name);reachable(p(name),seen);}return seen;}
function gdJson(name){const m=new RegExp(`^const\\s+${name}\\b[^=\\n]*=`,'m').exec(gd.code);assert(m);const a=gd.code.indexOf('[',m.index+m[0].length);assert(a>=0);return JSON.parse(gd.raw.slice(a,end(gd.code,a)));}
const declaration=/\$script:ManifestSpecJson\s*=/.exec(ps.raw);assert(declaration);
const literal=ps.strings.find(s=>s.a>=declaration.index+declaration[0].length);assert(literal);assert(!ps.raw.slice(declaration.index+declaration[0].length,literal.a).trim());
const spec=JSON.parse(literal.value),flow=gdJson('OBS_FLOW');
const roots=['schema','runId','sourceHead','sourcePins','artifacts','seed','scene','resources','optionalAssets','paths','limits','deadlineUtc'];

test(`${LABEL}: fixed text/interface inputs`,()=>{
  assert.equal(Buffer.byteLength(source[S]),14242);assert.equal(Buffer.byteLength(source[C]),5551);assert.equal(typeof JSON.parse(source[S]),'object');
  const scene=JSON.parse(source[C]);assert(Array.isArray(scene.collision));
  for(const name of ['snapshot','receipt','success','status'])assert(new RegExp(`\\bfunc\\s+${name}\\s*\\(`).test(source[P]));
  assert.equal(spec.schema,'wuxian.foundry-owned-observation/v1');assert.deepEqual(spec.rootKeys,roots);
  assert.deepEqual(spec.pinKeys,['path','bytes','sha256']);assert.deepEqual(spec.resourceKeys,['path','bytes','sha256','presence','required','purposeRef']);
  assert.deepEqual(spec.seed,{path:S,bytes:14242,sha256:'233275e296937b66956159f6bd89f9e6d08130298fd3ebfef84e52363158a0fb'});
  assert.deepEqual(spec.scene,{path:C,bytes:5551,sha256:'d90f852367b998cf5874b82fed583f1254664d4a88a8175bb3c94524b8f496b1'});
  assert.deepEqual(spec.limits,{pairs:1,photos:4,rawPipeBytes:1048576,naturalSeconds:120,drainSeconds:30,disposeSeconds:5});
  for(const name of ['Read-Manifest','Assert-Manifest','Assert-ExactKeys','Assert-Pin','Assert-ResourceRow','Resolve-Plan','Assert-OrdinaryNamespace','Assert-RunAuthority','Read-Receipt','Assert-ReceiptBinding','Assert-NotExpired','Assert-PurposeApproval','Assert-RunPins','Assert-HeadIndex','Assert-TextPins','Assert-ArtifactPins','Assert-ResourcePins','Assert-SeedPins','New-OwnedNamespace','Materialize-Seed','Get-NativeArguments','Get-RpcArguments','Get-GodotArguments','Start-OwnedProcess','Assert-ProcessIdentity','Assert-ReadyOwner','Pump-Pipes','Assert-Drained','Wait-OwnedClosure','Invoke-OwnedFallback','Invoke-OwnedRun','Invoke-Observation'])assert(pf.has(name),'missing controller function '+name);
  assert(source[G].trimEnd().split('\n').length+source[L].trimEnd().split('\n').length<=900);
  assert(source[T].trimEnd().split('\n').length<=220);
  for(const m of source[T].matchAll(/from\s+['"]([^'"]+)['"]/g))assert(m[1].startsWith('node:'));
});

test(`${LABEL}: Plan return precedes Run effects`,()=>{
  assert(/\$Mode\s*=\s*'Plan'/.test(ps.raw));
  const entry=p('Invoke-Observation'),m=/if\s*\(\s*\$Mode\s+-eq\s+'Plan'\s*\)\s*\{/i.exec(entry.raw);assert(m);
  const a=entry.code.indexOf('{',m.index),b=end(entry.code,a);assert(/\breturn\s+\$plan\b/i.test(entry.code.slice(a,b)));
  assert(!/\bfinally\b|\btrap\b/i.test(entry.code));
  const before={code:entry.code.slice(0,b),raw:entry.raw.slice(0,b)},effects=/\.Start\s*\(|\.Kill\s*\(|CreateDirectory|New-Item|FileStream|CreateNew|ComputeHash|ReadAllBytes|\.Write(?:All|\s*\()/i;
  assert(!effects.test(before.code));for(const name of reachable(before))assert(!effects.test(p(name).code),`Plan effect in ${name}`);
  order({raw:entry.raw.slice(b)},'Assert-RunAuthority','Assert-RunPins','Invoke-OwnedRun');
  order(p('Assert-RunAuthority'),'Read-Receipt','Assert-ReceiptBinding','Assert-NotExpired','Assert-PurposeApproval');
  for(const token of ['$script:ManifestSpecJson','Assert-ExactKeys','Assert-Pin','Assert-ResourceRow'])has(p('Assert-Manifest'),token);
  has(p('Assert-Manifest'),"$pin.path.Contains(':')");has(p('Assert-Manifest'),"$pin.path -match '\\.(png|jpe?g");has(p('Assert-ReceiptBinding'),'exclusive_runtime_profile');has(p('Assert-ReceiptBinding'),"@('leaseId','owner','runId','projectRoot','scope','deadlineUtc')");
  for(const token of ['GetFullPath','ReparsePoint'])has(p('Assert-OrdinaryNamespace'),token);
});

test(`${LABEL}: prestart arguments, environment and retained ownership`,()=>{
  const start=p('Start-OwnedProcess');order(start,'[System.Diagnostics.Process]::new()','.Start()');
  for(const key of ['APPDATA','LOCALAPPDATA','TEMP','TMP','WX_FOUNDRY_OBS_MANIFEST','WX_FOUNDRY_OBS_MANIFEST_SHA256'])order(start,`$psi.Environment['${key}']`,'.Start()');
  assert(p('Get-RpcArguments').raw.replace(/\s/g,'').includes("@('--v1-ready',$Ready,'--v1-token',$Token)"));
  assert(p('Get-NativeArguments').raw.replace(/\s/g,'').includes("@('--session-dir',$Session,'--save-dir',$Saves,'--session-token',$Token)"));
  for(const token of ['--path','--script','--log-file','Get-RpcArguments'])has(p('Get-GodotArguments'),token);
  const run=p('Invoke-OwnedRun');order(run,'New-OwnedNamespace','Materialize-Seed','Start-OwnedProcess','Assert-ReadyOwner','Wait-OwnedClosure');
  assert.equal(calls(run,'Start-OwnedProcess'),2);assert(!/\b(?:while|foreach|for|do)\b/i.test(run.code),'no pair retry loop');
  for(const token of ['StartTime','MainModule','CommandLine','StartInfo.ArgumentList','Record.Arguments'])has(p('Assert-ProcessIdentity'),token);
  for(const token of ['Assert-ProcessIdentity','OwningProcess','sessionDir','saveDir','token'])has(p('Assert-ReadyOwner'),token);
  order(p('Assert-ReadyOwner'),'Pump-Pipes','Get-NetTCPConnection');
});

test(`${LABEL}: bounded EOF/disposal and guarded fallback source`,()=>{
  for(const token of ['ReadAsync','rawPipeBytes','Overflow','Fault','Eof'])has(p('Pump-Pipes'),token);
  assert(/(?:Result|count|read)\s*-eq\s*0\b/i.test(p('Pump-Pipes').raw),'EOF must follow zero read');
  order(p('Wait-OwnedClosure'),'WaitForExit','Assert-Drained','ExitCode','.Dispose()');
  order(p('Invoke-OwnedFallback'),'Assert-RunAuthority','Assert-ProcessIdentity','.Kill(');
  assert(!/\$env:|Remove-Item|Stop-Process|Start-Process|Invoke-Expression|Add-Type|Set-Alias/i.test(ps.code));
  assert(!/WriteAllText|WriteAllBytes|FileMode\]::Create\b|FileMode\.Create\b|Out-File|Set-Content/i.test(ps.code));
  for(const m of ps.code.matchAll(/\.Start\s*\(/g))assert(m.index>=p('Start-OwnedProcess').a&&m.index<p('Start-OwnedProcess').b);
  for(const m of ps.code.matchAll(/\.Kill\s*\(/g))assert(m.index>=p('Invoke-OwnedFallback').a&&m.index<p('Invoke-OwnedFallback').b);
});

test(`${LABEL}: production receipt and same-key draw sequencing source`,()=>{
  order(g('_initialize'),'_verify_profile','_load_production');
  for(const token of ['OS.get_user_data_dir','globalize_path','WuxianV1Campaign141'])has(g('_verify_profile'),token);
  order(g('_send_once'),'sent.has','sent[method] = true','rpc.request');order(g('_advance'),'OBS_FLOW','state = row.next','_effect');
  order(g('_on_response'),'expected.id','expected.method','expected.association','_validate_receipt','_advance');
  for(const token of ['P.receipt','P.success','P.snapshot'])has(g('_validate_receipt'),token);
  has(g('_validate_status'),'P.status');has(g('_validate_status'),'paused');
  order(g('_on_frame'),'WAIT_FRAME','_assert_prepared_binding','_advance');has(g('_continue_once'),"'formal_continue'");
  order(g('_bind_camera'),'renderer.camera','camera.get_script() != projection_script','camera.origin = recipe.origin');
  for(const token of ["'formal_scene_ready'","'token'","'remainPaused':true"])has(g('_ready_once'),token);
  order(g('_status_once'),'_commit_final',"'bridge_session_status'");has(g('_wait_final_draw'),'RenderingServer.frame_post_draw');
  assert(!/frame_committed|reported_key/.test(g('_wait_final_draw').raw));assert(!/\breported_key\b/.test(gd.code));
  order(g('_capture_final'),'_wait_final_draw','_assert_live_bindings','_save_frame_once');
  order(g('_save_frame_once'),'FileAccess.file_exists','get_texture','save_png');order(g('_begin_close'),'close_started = true','_close_once');
  has(g('_close_once'),"'session_close'");has(g('_fail'),"'UNKNOWN'");has(g('_fail'),'_begin_close');
  assert(!/\b_advance\s*\(|\brpc\.request\s*\(/.test(g('_fail').code));
  const send=g('_send_once');for(const m of gd.code.matchAll(/\brpc\.request\s*\(/g))assert(m.index>=send.a&&m.index<send.b,'RPC request is centralized');
});

test(`${LABEL}: source-bound one-shot transition model`,()=>{
  const expected=[['AUTH','connected','WAIT_CONTINUE','continue'],['WAIT_CONTINUE','continue_ok','WAIT_FRAME','commit'],['WAIT_FRAME','prepared_frame','WAIT_READY','ready'],['WAIT_READY','ready_ok','WAIT_STATUS','status'],['WAIT_STATUS','status_ok','CAPTURING','capture'],['CAPTURING','captures_done','WAIT_CLOSE','close'],['WAIT_CLOSE','close_ok','DONE','quit']];
  assert.deepEqual(flow.map(r=>[r.state,r.event,r.next,r.action]),expected);
  const run=events=>{let state='AUTH',closed=false;const actions=[];for(const event of events){if(event==='unknown'){state='UNKNOWN';if(!closed){actions.push('close');closed=true;}continue;}if(state==='UNKNOWN')continue;const rows=flow.filter(r=>r.state===state&&r.event===event);if(rows.length!==1)throw Error('illegal or repeated transition');state=rows[0].next;actions.push(rows[0].action);if(rows[0].action==='close')closed=true;}return {state,actions};};
  const normal=run(expected.map(r=>r[1]));assert.equal(normal.state,'DONE');assert.deepEqual(normal.actions,['continue','commit','ready','status','capture','close','quit']);
  assert.throws(()=>run(['connected','connected']));const failed=run(['connected','unknown','unknown','continue_ok','captures_done']);
  assert.equal(failed.state,'UNKNOWN');assert.deepEqual(failed.actions,['continue','close']);
});

test(`${LABEL}: variable resource-purpose refusal model`,()=>{
  const acceptable=rows=>Array.isArray(rows)&&rows.length>0&&rows.every(r=>r&&typeof r==='object'&&Object.keys(r).sort().join('|')===[...spec.resourceKeys].sort().join('|')&&typeof r.required==='boolean'&&['present','absent'].includes(r.presence)&&typeof r.path==='string'&&r.path.length>0&&Number.isInteger(r.bytes)&&r.bytes>0&&/^[a-f0-9]{64}$/.test(r.sha256)&&(!r.required||r.presence==='present')&&(r.presence==='absent'?r.purposeRef===null:typeof r.purposeRef==='string'&&r.purposeRef.length>0));
  const unique=rows=>new Set(rows.map(r=>r.path.toLowerCase())).size===rows.length;
  const accepts=rows=>acceptable(rows)&&unique(rows);
  const row={path:'res://qualified.png',bytes:1,sha256:'a'.repeat(64),presence:'present',required:true,purposeRef:'MAIN:exact-purpose'};
  const second={...row,path:'res://second.png',purposeRef:'MAIN:second-purpose'};
  assert(accepts([row]));assert(accepts([row,second]));assert(!accepts([]));assert(!accepts([row,row]));
  assert(!accepts([{...row,purposeRef:null}]));assert(!accepts([{...row,presence:'absent',purposeRef:null}]));
  assert(accepts([{...row,presence:'absent',required:false,purposeRef:null}]));assert(!accepts([{...row,required:'true'}]));
  has(p('Assert-Manifest'),'$Manifest.resources.Count -lt 1');has(p('Assert-Manifest'),'Duplicate resource path');has(p('Assert-Manifest'),'ToLowerInvariant');
  for(const token of ['purposeApprovals','read','decode','display'])has(p('Assert-PurposeApproval'),token);
  has(p('Assert-PurposeApproval'),"Test-DeepEqual $approval.operations @('read','decode','display')");
});
