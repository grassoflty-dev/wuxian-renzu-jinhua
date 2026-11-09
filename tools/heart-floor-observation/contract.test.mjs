import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const fixturePath = resolve(repo, 'apps/godot_playable/tests/world/heart_floor_owned_observation.gd');
const launcherPath = resolve(repo, 'tools/heart-floor-observation/launch.ps1');
const contractPath = fileURLToPath(import.meta.url);
const fixture = readFileSync(fixturePath, 'utf8');
const launcher = readFileSync(launcherPath, 'utf8');
const contract = readFileSync(contractPath, 'utf8');
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const lineCount = text => text.split(/\r?\n/).length;

function block(source, startText, endText) {
  const start = source.indexOf(startText);
  assert.notEqual(start, -1, `missing source anchor: ${startText}`);
  const end = source.indexOf(endText, start + startText.length);
  assert.notEqual(end, -1, `missing closing anchor: ${endText}`);
  return source.slice(start, end);
}

test('owned Heart observation sources are bounded, fail-closed and non-executing by default', () => {
  assert.ok(lineCount(fixture) + lineCount(launcher) <= 800);
  assert.ok(lineCount(contract) <= 180);
  assert.match(launcher, /\[string\]\$Mode\s*=\s*'PlanOnly'/);
  assert.match(launcher, /sideEffects=\$false/);
  assert.match(launcher, /resourceAllowlist\.Count\s*-ne\s*16/);
  assert.match(launcher, /resourcePurposeBindingReference/);
  assert.match(launcher, /dirtyStatusSha256|gitStatusSha256/);
  assert.match(launcher, /'nativeGodotPairs'|nativeGodotPairs/);
  assert.match(launcher, /captureFrameLimit\s*-ne\s*2/);
  assert.match(launcher, /FileMode\]::CreateNew/);
  assert.doesNotMatch(launcher, /Remove-Item|taskkill|Stop-Process|Get-Process\s/);

  const dispatch = launcher.slice(launcher.indexOf("if ($Mode -eq 'PlanOnly')"));
  const planOnly = dispatch.indexOf("if ($Mode -eq 'PlanOnly')");
  const manifestGate = dispatch.indexOf('Assert-SourceAndManifest $ManifestPath');
  const runCall = dispatch.indexOf('Invoke-Run $manifestValue');
  assert.ok(planOnly >= 0 && manifestGate > planOnly && runCall > manifestGate);

  const manifestGateBody = block(launcher, 'function Assert-SourceAndManifest', 'function New-PrivateFile');
  assert.match(manifestGateBody, /ExpectedManifestSha256/);
  assert.match(manifestGateBody, /resourcePurposeBindingSha256/);
  assert.match(manifestGateBody, /sourceHead/);
  assert.match(manifestGateBody, /dirtyInputHashes/);
  assert.match(manifestGateBody, /Get-GitStatusDigest/);
  assert.match(manifestGateBody, /RUNTIME_SOURCE_PIN_REQUIRED/);
  assert.match(manifestGateBody, /RESOURCE_PIN_CHANGED/);
  assert.match(manifestGateBody, /RUN_DEADLINE_EXPIRED/);

  const startBody = block(launcher, 'function Start-OwnedProcess', 'function Assert-OwnedProcess');
  const envBind = startBody.indexOf('$info.Environment[$name]');
  const argBind = startBody.indexOf('$info.ArgumentList.Add');
  const processStart = startBody.indexOf('$process.Start()');
  assert.ok(envBind >= 0 && argBind >= 0 && processStart > envBind && processStart > argBind);
  assert.match(startBody, /CopyToAsync/);
  const stopBody = block(launcher, 'function Stop-OwnedProcess', 'function Complete-OwnedProcess');
  const identityCheck = stopBody.indexOf('Assert-OwnedProcess $Owned');
  const exactKill = stopBody.indexOf('$Owned.Process.Kill()');
  assert.ok(identityCheck >= 0 && exactKill > identityCheck);
  assert.match(stopBody, /EXACT_RETAINED_PROCESS_IDENTITY/);

  const runBody = block(launcher, 'function Invoke-Run', "if ($Mode -eq 'PlanOnly')");
  const seedCheck = runBody.indexOf('SEED_IDENTITY_MISMATCH');
  const firstDirectoryCreate = runBody.indexOf('[IO.Directory]::CreateDirectory');
  const seedWrite = runBody.indexOf('New-PrivateFile $savePath');
  const nativeStart = runBody.indexOf('Start-OwnedProcess $Manifest.native.path');
  const godotStart = runBody.indexOf('Start-OwnedProcess $Manifest.engine.path');
  assert.ok(seedCheck >= 0 && firstDirectoryCreate > seedCheck && seedWrite > firstDirectoryCreate);
  assert.ok(nativeStart > seedWrite && godotStart > nativeStart);
  assert.match(runBody, /NATIVE_CLOSE_TIMEOUT/);
  assert.match(runBody, /process\.json/);

  const observe = block(fixture, 'func _observe() -> bool:', 'func _verify_profile_binding() -> bool:');
  const profileGuard = observe.indexOf('if not _verify_profile_binding()');
  const rpcCreate = observe.indexOf('_rpc = RpcClientScript.new()');
  const rendererCreate = observe.indexOf('_renderer = RendererScript.new()');
  const continueCall = observe.indexOf("await _rpc_call('formal_continue'");
  const readyCommit = observe.indexOf('_bind_viewport(Vector2i(1920,1080)');
  const readyCall = observe.indexOf("await _rpc_call('formal_scene_ready'");
  const wholeCapture = observe.indexOf("_capture('whole-floor.png'");
  const entranceCapture = observe.indexOf("_capture('entrance-joint.png'");
  assert.ok(profileGuard >= 0 && rpcCreate > profileGuard && rendererCreate > profileGuard);
  assert.ok(continueCall > profileGuard && readyCommit > continueCall && readyCall > readyCommit);
  assert.ok(wholeCapture > readyCall && entranceCapture > wholeCapture);
  assert.equal((fixture.match(/_rpc_call\('formal_continue'/g) || []).length, 1);
  assert.equal((fixture.match(/_rpc_call\('formal_scene_ready'/g) || []).length, 1);
  assert.equal((fixture.match(/_rpc_call\('session_close'/g) || []).length, 1);
  assert.match(fixture, /P\.snapshot\(prepared\)/);
  assert.match(fixture, /P\.token\(value\.get\('entryToken'\)\)/);
  assert.match(fixture, /catalog\.build\('clockworks','cw_furnace_heart'\)/);
  assert.match(fixture, /HEART_SCENE_SHA256/);
  assert.match(fixture, /RenderingServer\.frame_post_draw/);
  assert.match(fixture, /status\.get\('paused'\)!=true/);
  assert.match(fixture, /status\.get\('entryToken'\)!=null/);
  assert.doesNotMatch(fixture, /worldEpoch\s*==\s*76/);

  const binding = block(fixture, 'func _verify_profile_binding() -> bool:', 'func _valid_heart_snapshot');
  assert.match(binding, /OS\.get_user_data_dir\(\)/);
  assert.match(binding, /ProjectSettings\.globalize_path\('user:\/\/'\)/);
  assert.match(binding, /actual_os==expected and actual_user==expected/);
  const capture = block(fixture, 'func _capture(', 'func _write_context');
  assert.match(capture, /MAX_CAPTURES/);
  assert.match(capture, /get_root\(\)\.get_texture\(\)\.get_image\(\)/);
  assert.match(capture, /image\.save_png\(destination\)/);
  assert.match(capture, /FileAccess\.file_exists\(destination\)/);
  assert.match(fixture, /'whole-floor\.png'/);
  assert.match(fixture, /'entrance-joint\.png'/);

  const canonical = readFileSync(resolve(repo, 'apps/godot_playable/data/scenes/cw_furnace_heart.json'));
  const seed = readFileSync(resolve(repo, 'server-rs/tests/fixtures/earned-cw-furnace-entry-v6.json'));
  assert.equal(canonical.length, 8424);
  assert.equal(digest(canonical), 'f16f3941767aed71023cd8b9ce75642345a103eabf5c77682e358c96c2e219d5');
  assert.equal(seed.length, 17801);
  assert.equal(digest(seed), 'd246c77b29c417d94220f7450408655912ea17fefabbecb32fac01c2a15793c2');

  const quotedPath = launcherPath.replaceAll("'", "''");
  const parseCommand = `$tokens=$null;$errors=$null;[System.Management.Automation.Language.Parser]::ParseFile('${quotedPath}',[ref]$tokens,[ref]$errors)|Out-Null;if($errors.Count){$errors|ForEach-Object{[Console]::Error.WriteLine($_.ToString())};exit 1};exit 0`;
  const parsed = spawnSync('powershell.exe', ['-NoLogo','-NoProfile','-NonInteractive','-Command',parseCommand], { encoding:'utf8', timeout:15000, maxBuffer:1048576 });
  assert.equal(parsed.error, undefined, parsed.error?.message);
  assert.equal(parsed.status, 0, `${parsed.stdout}\n${parsed.stderr}`);
});
