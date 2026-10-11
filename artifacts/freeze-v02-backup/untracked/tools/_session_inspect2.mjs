import { readFileSync } from 'node:fs';
import { zstdDecompressSync } from 'node:zlib';
const ZSTD_MAGIC = 4247762216;
function scanZstdFrames(buffer, maxFrames = Infinity) {
  const frames = []; let offset = 0;
  while (offset < buffer.length) {
    const start = offset;
    if (buffer.length - offset < 4) return { frames, tornStart: start };
    if (buffer.readUInt32LE(offset) !== ZSTD_MAGIC) throw new Error('bad magic');
    offset += 4;
    if (offset === buffer.length) return { frames, tornStart: start };
    const descriptor = buffer.readUInt8(offset); offset += 1;
    const csf = descriptor >>> 6, ss = (descriptor & 32) !== 0, ck = (descriptor & 4) !== 0, df = descriptor & 3;
    const db = df === 3 ? 4 : df, csb = csf === 0 ? (ss ? 1 : 0) : (1 << csf);
    offset += (ss ? 0 : 1) + db + csb;
    for (;;) {
      const bh = buffer.readUIntLE(offset, 3); offset += 3;
      const last = (bh & 1) !== 0, bt = (bh >>> 1) & 3, bs = bh >>> 3;
      const pb = bt === 1 ? 1 : bs; offset += pb;
      if (last) break;
    }
    if (ck) offset += 4;
    frames.push({ start, end: offset });
    if (frames.length === maxFrames) return { frames };
  }
  return { frames };
}
const buf = readFileSync(process.argv[2]);
const { frames } = scanZstdFrames(buf);
const want = new Set(['user/message','assistant/message','tool/call','tool/result','assistant/chunk','text-chunks']);
const samples = {};
for (let fi = 0; fi < frames.length; fi++) {
  const f = frames[fi];
  const plain = zstdDecompressSync(buf.subarray(f.start, f.end)).toString('utf8');
  for (const ln of plain.split('\n')) {
    if (!ln) continue;
    let ev; try { ev = JSON.parse(ln); } catch { continue; }
    const t = ev.type;
    if (!want.has(t)) continue;
    if (!samples[t]) samples[t] = [];
    if (samples[t].length < 2) samples[t].push(ev);
  }
}
for (const t of Object.keys(samples)) {
  console.log('===== ' + t + ' =====');
  for (const ev of samples[t]) {
    console.log(JSON.stringify(ev).slice(0, 900));
    console.log('---');
  }
}
