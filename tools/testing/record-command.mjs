import { spawnSync } from 'node:child_process';
import { mkdirSync, openSync, closeSync, readFileSync, writeFileSync, renameSync, readSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
// Hash logs in fixed-size chunks so noisy output does not allocate the whole file.
function describeLog(path, name) {
  const fd = openSync(path, 'r'), hash = createHash('sha256'), buffer = Buffer.alloc(65536);
  let bytes = 0;
  try {
    for (let count; (count = readSync(fd, buffer, 0, buffer.length, null)) > 0;) { hash.update(buffer.subarray(0, count)); bytes += count; }
  } finally { closeSync(fd); }
  return { path: name, bytes, sha256: hash.digest('hex') };
}
const errorInfo = error => error ? { name: error.name, code: error.code ?? null, message: error.message } : null;

// Records command outcome, not test acceptance. Empty output never determines success.
// Intended for bounded, non-server commands; timeout kills only the direct child.
export function recordCommand({ command, cwd, sourceSha, inputs, outputDir, timeoutMs = 30000 }) {
  if (!Array.isArray(command) || !command.length || command.some(x => typeof x !== 'string') || !command[0]) throw new TypeError('command must be a non-empty argv array');
  if (!/^[a-f0-9]{40}$/.test(sourceSha)) throw new TypeError('sourceSha must be an exact Git SHA');
  if (!Array.isArray(inputs) || !inputs.length || inputs.some(x => typeof x.path !== 'string' || !/^[a-f0-9]{64}$/.test(x.sha256))) throw new TypeError('verified input manifest required');
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs < 1) throw new TypeError('positive timeoutMs required');
  cwd = resolve(cwd); outputDir = resolve(outputDir);
  // Never overwrite an earlier run, including a partial run.
  mkdirSync(outputDir);
  const metadataPath = resolve(outputDir, 'metadata.json');
  const metadata = { schemaVersion: 1, sourceSha, sourceScope: 'manifest-only; not a full clean checkout claim', inputs, command, cwd,
    runtime: { node: process.version, platform: process.platform, arch: process.arch }, timeoutMs,
    status: 'UNKNOWN', exitCode: null, signal: null, naturalExit: null, error: null, commandStartedAt: null, commandEndedAt: null,
    elapsedMs: null, timingMeaning: 'recorder observation immediately before/after spawnSync', stdout: null, stderr: null };
  const save = () => { writeFileSync(metadataPath + '.tmp', JSON.stringify(metadata, null, 2) + '\n'); renameSync(metadataPath + '.tmp', metadataPath); };
  const verify = () => inputs.every(input => sha256(readFileSync(resolve(cwd, input.path))) === input.sha256);
  let out, err;
  save();
  try {
    if (!verify()) throw new Error('Input identity mismatch before command');
    out = openSync(resolve(outputDir, 'stdout.log'), 'wx');
    err = openSync(resolve(outputDir, 'stderr.log'), 'wx');
    metadata.commandStartedAt = new Date().toISOString(); save();
    const start = process.hrtime.bigint();
    let result;
    try { result = spawnSync(command[0], command.slice(1), { cwd, shell: false, stdio: ['ignore', out, err], timeout: timeoutMs, killSignal: 'SIGKILL', windowsHide: true }); }
    finally { metadata.commandEndedAt = new Date().toISOString(); metadata.elapsedMs = Number(process.hrtime.bigint() - start) / 1e6; }
    metadata.exitCode = Number.isInteger(result.status) ? result.status : null;
    metadata.signal = result.signal ?? null;
    metadata.error = errorInfo(result.error);
    metadata.naturalExit = metadata.exitCode !== null && !result.error && !metadata.signal ? true : result.error || metadata.signal ? false : null;
    metadata.status = result.error?.code === 'ETIMEDOUT' ? 'TIMEOUT' : result.error ? 'EXECUTION_ERROR' : metadata.signal ? 'SIGNALED' : metadata.exitCode === 0 ? 'EXIT_ZERO' : metadata.exitCode !== null ? 'EXIT_NONZERO' : 'UNKNOWN';
    // Persist exit/timing before optional identity checks or log inspection.
    save();
    metadata.inputsUnchanged = verify();
    if (!metadata.inputsUnchanged) metadata.status = 'INPUT_CHANGED';
  } catch (error) {
    metadata.recordingError = errorInfo(error);
    metadata.status = 'RECORDING_ERROR';
  } finally {
    for (const fd of [out, err]) if (fd !== undefined) closeSync(fd);
    for (const stream of ['stdout', 'stderr']) {
      try { metadata[stream] = describeLog(resolve(outputDir, stream + '.log'), stream + '.log'); }
      catch (error) { metadata[stream] = { unavailable: true, error: errorInfo(error) }; metadata.status = 'RECORDING_ERROR'; }
    }
    save();
  }
  return metadata;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const result = recordCommand(JSON.parse(readFileSync(process.argv[2], 'utf8')));
    console.log(JSON.stringify({ status: result.status, exitCode: result.exitCode }));
    process.exitCode = result.status === 'EXIT_ZERO' ? 0 : Number.isInteger(result.exitCode) && result.exitCode > 0 && result.exitCode <= 255 ? result.exitCode : 1;
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
