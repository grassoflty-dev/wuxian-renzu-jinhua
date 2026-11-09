import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { win32 } from 'node:path';

const source = readFileSync(new URL('./run-owned.ps1', import.meta.url), 'utf8').replace(/\r\n/g, '\n');
const adapterSource = readFileSync(new URL('../../apps/godot_playable/tests/campaign/grey_hive_first_playable.gd', import.meta.url), 'utf8').replace(/\r\n/g, '\n');
const nativeProducerSource = readFileSync(new URL('../../server-rs/src/godot_main.rs', import.meta.url), 'utf8');
const rpcClientSource = readFileSync(new URL('../../apps/godot_playable/scripts/bridge/rpc_client.gd', import.meta.url), 'utf8');
const interfaceMatch = source.match(/^\$GH_INTERFACE_JSON\s*=\s*@'\n([\s\S]*?)\n'@/m);
assert.ok(interfaceMatch, 'complete fixed interface literal');
const I = JSON.parse(interfaceMatch[1]);
const adapterInterfaceMatch = adapterSource.match(/^const\s+GH_INTERFACE_JSON\s*=\s*"""\n([\s\S]*?)\n"""/m);
assert.ok(adapterInterfaceMatch, 'complete adapter interface literal');
assert.deepEqual(JSON.parse(adapterInterfaceMatch[1]), I, 'runner and adapter share one exact interface');
const escape = value => value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

function body(name) {
  const re = new RegExp('^function\\s+' + escape(name) + '\\s*\\{', 'm');
  const match = re.exec(source);
  assert.ok(match, 'function exists: ' + name);
  const begin = source.indexOf('{', match.index);
  let depth = 0, state = 'code';
  for (let i = begin; i < source.length; i++) {
    const c = source[i], n = source[i + 1];
    if (state === 'line') { if (c === '\n') state = 'code'; continue; }
    if (state === 'block') { if (c === '#' && n === '>') { state = 'code'; i++; } continue; }
    if (state === 'single') { if (c === "'" && n === "'") { i++; continue; } if (c === "'") state = 'code'; continue; }
    if (state === 'double') { if (c === '`') { i++; continue; } if (c === '"') state = 'code'; continue; }
    if (c === '<' && n === '#') { state = 'block'; i++; continue; }
    if (c === '#') { state = 'line'; continue; }
    if (c === "'") { state = 'single'; continue; }
    if (c === '"') { state = 'double'; continue; }
    if (c === '{') depth++;
    if (c === '}' && --depth === 0) return source.slice(begin + 1, i);
  }
  assert.fail('unbalanced function: ' + name);
}

function ordered(text, anchors) {
  let cursor = 0;
  for (const anchor of anchors) {
    const at = text.indexOf(anchor, cursor);
    assert.ok(at >= 0, 'positive scoped anchor: ' + anchor);
    cursor = at + anchor.length;
  }
}
function has(text, pattern) { assert.match(text, pattern); }
function exactKeys(value, keys) {
  return value && typeof value === 'object' && !Array.isArray(value) &&
    Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value, key));
}
// Models QualifiedManifestSha256 being bound to Read-Manifest's byte digest.
const expectedManifestDigest = 'a'.repeat(64);
function canRun(m, suppliedLeaseId = m.lease?.id, qualifiedManifestDigest = expectedManifestDigest) {
  return m.schema === I.manifestSchema && m.driver === I.driver &&
    exactKeys(m, I.manifestKeys) && exactKeys(m.authority, I.runRequires) &&
    I.runRequires.every(k => m.authority[k]?.state === 'QUALIFIED' &&
      typeof m.authority[k]?.reference === 'string' && m.authority[k].reference.length > 0) &&
    m.lease.maxPairs === 2 && m.lease.id === suppliedLeaseId &&
    qualifiedManifestDigest === expectedManifestDigest &&
    /^[a-f0-9]{64}$/.test(qualifiedManifestDigest) && Array.isArray(m.unresolved) && m.unresolved.length === 0 &&
    m.operator?.available === true && I.baizhiChoices.includes(m.operator?.baizhiChoice) && m.operator.baizhiChoice !== 'unresolved' &&
    m.acceptance?.baizhiChoice === m.operator.baizhiChoice &&
    Array.isArray(m.resources) && m.resources.length > 0 &&
    new Set(m.resources.map(r => r.path)).size === m.resources.length &&
    m.resources.every(r => exactKeys(r, I.resourceKeys) && r.qualificationState === 'QUALIFIED' && r.purposeReference &&
      r.presence === 'present' && (r.required === true || r.required === false));
}
function qualifiedModel() {
  const choice = 'taken';
  return { schema:I.manifestSchema, runId:'00000000-0000-4000-8000-000000000001', driver:I.driver,
    authority:Object.fromEntries(I.runRequires.map(k => [k,{state:'QUALIFIED',reference:'external:'+k}])),
    sourceHead:'a'.repeat(40), source:{}, artifacts:{},
    resources:[{path:'res://one.png',bytes:1,sha256:'a'.repeat(64),presence:'present',required:true,
      consumer:'adapter',readPhase:'new',purposeReference:'external:purpose',qualificationState:'QUALIFIED',distinction:'display'}],
    optionalAssets:[], namespace:{}, lease:{id:'lease-example',owner:'operator',firstUtc:'2026-10-22T00:00:00Z',
      reserveUtc:'2026-10-22T00:10:00Z',hardUtc:'2026-10-22T00:20:00Z',maxPairs:2,fallbackScope:'owned'},
    operator:{reference:'external:operator',available:true,baizhiChoice:choice},
    limits:{}, acceptance:{baizhiChoice:choice}, unresolved:[], deadlineUtc:'2026-10-22T00:20:00Z' };
}
function contained(base,candidate) {
  const relative = win32.relative(base,candidate);
  return relative.length > 0 && relative !== '..' && !relative.startsWith('..\\') && !win32.isAbsolute(relative);
}

test('SOURCE_CONTRACT_NOT_POWERSHELL_OR_RUNTIME: fixed interface and refusal model', () => {
  assert.ok(source.trimEnd().split('\n').length<=1100);
  assert.equal(I.schema,'wuxian.gh-first-playable.interface/v2');
  assert.equal(I.manifestSchema,'wuxian.gh-first-playable.manifest/v2');
  assert.deepEqual(I.manifestKeys, ['schema','runId','driver','authority','sourceHead','source','artifacts','resources','optionalAssets','namespace','lease','operator','limits','acceptance','unresolved','deadlineUtc']);
  assert.deepEqual(I.requiredSourcePins, ['apps/godot_playable/scripts/ui/v1_main.gd','apps/godot_playable/scripts/ui/panels.gd','apps/godot_playable/scripts/world/renderer.gd','apps/godot_playable/tests/campaign/grey_hive_first_playable.gd','tools/grey-hive-first-playable/run-owned.ps1','tools/grey-hive-first-playable/contract.test.mjs']);
  assert.deepEqual(I.phaseConfigKeys, ['schema','phase','manifestSha256','manifest','sessionDir','saveDir','expectedUserDir','sessionTokenSha256']);
  assert.deepEqual(I.eventKeys, ['schema','runId','phase','sequence','utc','monotonicMs','kind','outcome','manifestSha256','sourceBaselineReference','payload']);
  assert.deepEqual(I.terminalPayloadKeys, ['authenticated','association','closing','connectionGeneration']);
  assert.equal(I.eventTransport,'phase-output-file');
  assert.equal(I.eventFileName,'events.ndjson');
  assert.equal(I.terminalKind,'session_close');
  assert.equal(I.terminalOutcome,'CONFIRMED');
  assert.deepEqual(I.phaseOrder,['new','continue']);
  assert.deepEqual(I.rpcArguments,['--v1-ready','--v1-token']);
  assert.equal(I.saveRelative,'session/saves/formal-save-v6.json');
  assert.equal(I.driver,'operator');
  const good=qualifiedModel(); assert.equal(canRun(good),true);
  for (const key of I.runRequires) { const bad=structuredClone(good); bad.authority[key].state='UNRESOLVED'; assert.equal(canRun(bad),false,key); }
  let bad=structuredClone(good); bad.resources[0].qualificationState='UNRESOLVED'; assert.equal(canRun(bad),false);
  bad=structuredClone(good); bad.operator.baizhiChoice='unresolved'; bad.acceptance.baizhiChoice='unresolved'; assert.equal(canRun(bad),false);
  assert.equal(canRun(good, 'other-lease'), false);
  assert.equal(canRun(good, good.lease.id, 'b'.repeat(64)), false);
  bad=structuredClone(good); bad.resources=[]; assert.equal(canRun(bad),false);
  bad=structuredClone(good); bad.resources.push({...bad.resources[0]}); assert.equal(canRun(bad),false);
  bad=structuredClone(good); bad.extra='not-in-manifest-v2'; assert.equal(canRun(bad),false);
});

test('SOURCE_CONTRACT: Plan has no launch or materialization path', () => {
  assert.match(source, /\$Mode\s*=\s*'Plan'/);
  const plan=body('Get-Plan'); has(plan,/Read-Manifest/);
  assert.doesNotMatch(plan,/Start-Owned|New-Owned|Add-Type|CreateDirectory|CreateNew|Get-FileHash/);
  const dispatch=source.match(/^# BEGIN GH_DISPATCH\n([\s\S]*?)^# END GH_DISPATCH$/m);
  assert.ok(dispatch,'isolated dispatch region');
  ordered(dispatch[1],["$Mode -eq 'Plan'",'Get-Plan','exit 0','Invoke-OwnedRun']);
  const auth=body('Assert-RunAuthorization');
  for (const value of ['runRequires','QualifiedManifestSha256','LeaseId']) assert.ok(auth.includes(value),value);
  assert.doesNotMatch(source,/Invoke-Expression|Start-Job|Start-Process/);
});

test('SOURCE_CONTRACT: prestart qualification and child-only environment', () => {
  const config=body('Write-PhaseConfig');
  for (const field of ['phaseConfigSchema','manifestSha256','manifest=','sessionDir=','saveDir=','expectedUserDir=','sessionTokenSha256=']) assert.ok(config.includes(field),field);
  const events=body('Read-PhaseEvents');
  for (const field of ['eventFileName','eventKeys','evidenceSchema','manifestSha256','sourceBaselineReference','maxEventLineBytes','maxEventFileBytes']) assert.ok(events.includes(field),field);
  const strictJson=body('Read-StrictJson');
  ordered(strictJson,['JsonDocument]::Parse','Test-DuplicateJsonKeys',
    'ConvertFrom-Json -InputObject $text -AsHashtable -Depth 100 -DateKind String']);
  ordered(events,['JsonDocument]::Parse','Test-DuplicateJsonKeys',
    'ConvertFrom-Json -InputObject $line -AsHashtable -Depth 32 -DateKind String']);
  for (const field of ['firstUtc','reserveUtc','hardUtc'])
    assert.ok(body('Assert-Manifest').includes('Assert-String $Manifest.lease.'+field+" 'lease."+field+"'"));
  for (const token of ['$event.utc -isnot [string]','phase event timestamp is not canonical UTC','phase event timestamp is not UTC'])
    assert.ok(events.includes(token),token);
  for (const token of ['$stampPattern','[TimeSpan]::Zero','$reserve -le $first','$hard -le $reserve'])
    assert.ok(body('Assert-OrdinaryNamespace').includes(token),token);
  has(body('Assert-SessionClose'),/terminalPayloadKeys/);
  ordered(body('Invoke-OwnedRun'),['Assert-RunAuthorization','Assert-PurposeCoverage','Assert-SourcePins',
    'Assert-ArtifactPins','Assert-FreshNamespace','New-OwnedNamespace','Start-OwnedPair',
    'Wait-OwnedPairClosed','Confirm-OwnedSave','Archive-OwnedReady','Start-OwnedPair','Wait-OwnedPairClosed']);
  const env=body('New-ChildEnvironment');
  for (const key of ['APPDATA','LOCALAPPDATA','TEMP','TMP',...I.environment]) assert.ok(env.includes(key),key);
  assert.doesNotMatch(source,/\$env:[A-Za-z_][A-Za-z_0-9]*\s*=/i);
  ordered(body('Start-OwnedProcess'),['.Environment[','Process]::new','.Start(']);
  ordered(body('Start-OwnedPair'),['Start-OwnedProcess','Assert-ReadyIdentity','Write-PhaseConfig',
    'New-ChildEnvironment','Start-OwnedProcess']);
  const native=body('Get-NativeArguments');
  for (const token of ['--session-dir','--save-dir','--session-token']) assert.ok(native.includes(token));
  ordered(native,['--session-dir','$Session','--save-dir','$Saves','--session-token','$Token']);
  const rpc=body('Get-RpcArguments'); ordered(rpc,['$i.rpcArguments[0]','$Ready','$i.rpcArguments[1]','$Token']);
  const godot=body('Get-GodotArguments');
  for (const token of ['--path','--rendering-method','gl_compatibility','--audio-driver','Dummy','--log-file','--script','--']) assert.ok(godot.includes(token));
  const ready=body('Assert-ReadyIdentity');
  ordered(ready,['Assert-OwnedIdentity','Assert-ReadyOwner']);
  const identity=body('Assert-ProcessIdentity')+body('Start-OwnedProcess');
  for (const field of ['StartTime','MainModule','CommandLine']) assert.ok(identity.includes(field),field);
  const owner=body('Assert-ReadyOwner');
  for (const field of ['processId','sessionDir','saveDir','127.0.0.1','OwningProcess','sessionToken']) assert.ok(owner.includes(field),field);
  assert.ok(owner.includes('[string]$Ready.sessionToken -cne $Token'));
  assert.doesNotMatch(owner, /\$Ready\.token\b/);
  assert.ok(nativeProducerSource.includes('"sessionToken":token'));
  assert.ok(rpcClientSource.includes("ready.get('sessionToken')"));
  assert.match(nativeProducerSource, /\(32\.\.=128\)/);
  assert.ok(nativeProducerSource.includes("b.is_ascii_alphanumeric()||b==b'-'"));
  assert.ok(rpcClientSource.includes('v.length()<32 or v.length()>128'));
  assert.ok(rpcClientSource.includes('abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-'));
  const creation=body('Get-ProcessCreationUtc');
  for (const marker of ['$Value -is [DateTime]','$Value.Kind -eq [DateTimeKind]::Unspecified','$Value.ToUniversalTime()',
    '$Value -is [DateTimeOffset]','$Value.UtcDateTime','\\A[0-9]{14}\\.[0-9]{6}[+-][0-9]{3}\\z',
    'ManagementDateTimeConverter]::ToDateTime','Stop-Run 40']) assert.ok(creation.includes(marker),marker);
  assert.doesNotMatch(creation, /\bGet-CimInstance\b|\bGet-Process\s/);
  const processIdentity=body('Assert-ProcessIdentity');
  assert.ok(processIdentity.includes('Get-ProcessCreationUtc $native.CreationDate'));
  assert.ok(processIdentity.includes('TotalSeconds) -gt 2.0'));
  const generatedToken=body('New-SessionToken');
  for (const marker of ['[byte[]]::new(32)','RandomNumberGenerator]::Create()','[System.BitConverter]::ToString($bytes)',
    ".Replace('-','').ToLowerInvariant()"]) assert.ok(generatedToken.includes(marker),marker);
  assert.doesNotMatch(generatedToken,/ToBase64String/);
  const ns=body('Assert-FreshNamespace'); has(ns,/ReparsePoint/);
  const base='C:\\private\\run';
  assert.equal(contained(base,base+'\\id\\session'),true);
  assert.equal(contained(base,'C:\\private\\runner\\id'),false);
  assert.equal(contained(base,'D:\\elsewhere'),false);
});

test('SOURCE_CONTRACT: closure and exact ready archival precede second pair', () => {
  const closed=body('Wait-OwnedPairClosed');
  for (const field of ['ExitCode','Eof','Overflow','Fault','Dispose']) assert.ok(closed.includes(field),field);
  ordered(body('Archive-OwnedReady'),['Assert-ReadyRecordSnapshot','[System.IO.File]::Move','Assert-ReadyRecordSnapshot']);
  const fallback=body('Invoke-OwnedFallback'); ordered(fallback,['Assert-FallbackAuthorization','Assert-OwnedIdentity','.Kill(']);
  for (const name of ['Assert-Manifest','Assert-PurposeCoverage']) assert.ok(body(name).length>0);
  const manifest = body('Assert-Manifest');
  for (const label of ['new:new-rs:rs_core_room','new:gh-entry:gh_entry_maintenance','new:sentinel-encounter:gh_sentinel_arena','new:gh-exit-extraction:gh_exit','new:returned-rs-saved:rs_core_room','continue:continue-rs:rs_core_room']) assert.ok(manifest.includes(label),label);
  assert.ok(manifest.includes("$Manifest.limits.captureCap -ne 6"));
  assert.ok(manifest.includes("($actualCaptures -join '|') -cne ($expectedCaptures -join '|')"));
  assert.ok(body('Read-PhaseEvents').includes("$Pair.Phase -ceq 'new' -and $info.Length -gt 67108864"));
  const captures = body('Assert-SessionClose');
  assert.ok(captures.includes("$sequence -ge [long]$terminal[0].sequence"));
  assert.ok(captures.includes("$sequence -le [long]$summary[0].sequence"));
  assert.ok(captures.includes("$saveReceipt[0].payload.association.exitAfter -ne $false"));
  assert.ok(captures.includes("$sequence -le [long]$extraction[0].sequence"));
  assert.doesNotMatch(source,/Stop-Process|Remove-Item|Kill\(\s*\$true\s*\)|--user-data-dir/);
  assert.doesNotMatch(source,/Copy-Item.*(?:scripts|project\.godot)/i);
  for (const name of ['Get-HeadOid','Assert-SourcePins','Assert-ArtifactPins','Assert-ResourcePins']) assert.ok(body(name).length>0);
});
