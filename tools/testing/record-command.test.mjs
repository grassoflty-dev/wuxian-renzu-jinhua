import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { recordCommand } from './record-command.mjs';
const digest = text => createHash('sha256').update(text).digest('hex');
function fixture() {
  const cwd = mkdtempSync(join(tmpdir(), 'command-evidence-test-'));
  writeFileSync(join(cwd, 'input.txt'), 'fixed');
  return { command: [process.execPath, '-e', ''], cwd, sourceSha: 'a'.repeat(40), inputs: [{ path: 'input.txt', sha256: digest('fixed') }], outputDir: join(cwd, 'run'), timeoutMs: 3000 };
}
test('empty successful stdout/stderr preserves identity, actual exit and observed times', () => {
  const config = fixture(), result = recordCommand(config);
  assert.equal(result.status, 'EXIT_ZERO'); assert.equal(result.exitCode, 0); assert.equal(result.naturalExit, true);
  for (const stream of ['stdout', 'stderr']) { assert.equal(result[stream].bytes, 0); assert.equal(result[stream].sha256, digest('')); }
  assert.deepEqual(result.command, config.command); assert.equal(result.sourceSha, config.sourceSha);
  assert.ok(result.commandStartedAt); assert.ok(result.commandEndedAt); assert.ok(result.elapsedMs >= 0);
  assert.deepEqual(JSON.parse(readFileSync(join(config.outputDir, 'metadata.json'))), result);
});
test('empty nonzero output cannot become a pass', () => {
  const config = fixture(); config.command[2] = 'process.exit(7)';
  const result = recordCommand(config); assert.equal(result.exitCode, 7); assert.equal(result.status, 'EXIT_NONZERO');
});
test('raw stdout and stderr remain separate, including non-UTF8 bytes', () => {
  const config = fixture(); config.command[2] = 'process.stdout.write(Buffer.from([0,255,10]));process.stderr.write("error\\n")';
  const result = recordCommand(config);
  assert.equal(result.stdout.bytes, 3); assert.equal(result.stderr.bytes, 6);
  assert.deepEqual(readFileSync(join(config.outputDir, 'stdout.log')), Buffer.from([0,255,10]));
});
test('spawn failure preserves unknown exit instead of zero', () => {
  const config = fixture(); config.command = [join(config.cwd, 'missing-executable')];
  const result = recordCommand(config); assert.equal(result.exitCode, null); assert.equal(result.status, 'EXECUTION_ERROR'); assert.equal(result.error.code, 'ENOENT');
});
test('timeout is explicit and is never a successful natural exit', () => {
  const config = fixture(); config.command[2] = 'setInterval(()=>{}, 1000)'; config.timeoutMs = 100;
  const result = recordCommand(config); assert.equal(result.status, 'TIMEOUT'); assert.equal(result.naturalExit, false); assert.equal(result.exitCode, null); assert.equal(result.error.code, 'ETIMEDOUT');
});
test('preflight mismatch prevents execution and preserves UNKNOWN exit metadata', () => {
  const config = fixture(); config.inputs[0].sha256 = '0'.repeat(64);
  const result = recordCommand(config); assert.equal(result.status, 'RECORDING_ERROR'); assert.equal(result.exitCode, null); assert.equal(result.commandStartedAt, null);
});
test('post-command input mutation invalidates evidence even with exit zero', () => {
  const config = fixture(); config.command[2] = 'require("node:fs").writeFileSync("input.txt", "changed")';
  const result = recordCommand(config); assert.equal(result.status, 'INPUT_CHANGED'); assert.equal(result.exitCode, 0); assert.equal(result.inputsUnchanged, false);
});
test('existing evidence is never overwritten', () => {
  const config = fixture(); recordCommand(config); assert.throws(() => recordCommand(config), { code: 'EEXIST' });
});

test('missing captured log invalidates recording while preserving child exit', { skip: process.platform === 'win32' }, () => {
  const config = fixture();
  config.command[2] = `require("node:fs").unlinkSync(${JSON.stringify(join(config.outputDir, 'stdout.log'))})`;
  const result = recordCommand(config);
  assert.equal(result.status, 'RECORDING_ERROR'); assert.equal(result.exitCode, 0); assert.equal(result.stdout.unavailable, true);
});
test('large output is captured and hashed without whole-log buffering', () => {
  const config = fixture(); config.command[2] = 'process.stdout.write(Buffer.alloc(4 * 1024 * 1024, 65))';
  const result = recordCommand(config); assert.equal(result.status, 'EXIT_ZERO'); assert.equal(result.stdout.bytes, 4 * 1024 * 1024);
  assert.equal(result.stdout.sha256, digest(Buffer.alloc(4 * 1024 * 1024, 65)));
});
