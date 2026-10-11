import { readFileSync, mkdirSync, createWriteStream } from 'node:fs';
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

function textOf(content) {
  const out = [];
  (function walk(n) {
    if (n == null) return;
    if (typeof n === 'string') { out.push(n); return; }
    if (Array.isArray(n)) { for (const x of n) walk(x); return; }
    if (typeof n === 'object') {
      if (n.type === 'text' && typeof n.text === 'string') out.push(n.text);
      else for (const k of Object.keys(n)) walk(n[k]);
    }
  })(content);
  return out.join(' ').replace(/\r/g, '').replace(/\n+/g, '⏎');
}

const buf = readFileSync(process.argv[2]);
const outDir = process.argv[3];
mkdirSync(outDir, { recursive: true });
const { frames } = scanZstdFrames(buf);

const userStream = createWriteStream(`${outDir}/user_messages.txt`, { flags: 'w' });
const asstStream = createWriteStream(`${outDir}/assistant_messages.txt`, { flags: 'w' });
const toolStream = createWriteStream(`${outDir}/tool_calls.txt`, { flags: 'w' });
const toolCount = new Map();
let userCount = 0, asstCount = 0;

for (let fi = 0; fi < frames.length; fi++) {
  const f = frames[fi];
  let plain; try { plain = zstdDecompressSync(buf.subarray(f.start, f.end)).toString('utf8'); } catch { continue; }
  for (const ln of plain.split('\n')) {
    if (!ln) continue;
    let ev; try { ev = JSON.parse(ln); } catch { continue; }
    const t = ev.type, d = ev.data;
    if (!d || typeof d !== 'object') continue;
    if (t === 'user/message') {
      if (d.source && d.source.kind === 'user' && d.role === 'user') {
        const txt = textOf(d.content);
        if (txt.trim()) { userStream.write(`### U${userCount} #${ev.seq} @${ev.time}\n${txt}\n\n`); userCount++; }
      }
    } else if (t === 'assistant/message') {
      const m = d.message;
      if (m && m.role === 'assistant' && Array.isArray(m.content)) {
        const parts = [];
        for (const c of m.content) {
          if (c && c.type === 'text' && typeof c.text === 'string') parts.push(c.text);
        }
        const txt = parts.join(' ').replace(/\r/g, '').replace(/\n+/g, '⏎');
        if (txt.trim()) { asstStream.write(`### A${asstCount} #${ev.seq} @${ev.time}\n${txt}\n\n`); asstCount++; }
      }
    } else if (t === 'tool/call') {
      const name = d.name || '?';
      toolCount.set(name, (toolCount.get(name) || 0) + 1);
      let argsStr = '';
      try { const a = JSON.parse(d.arguments || '{}'); argsStr = (a.command || a.description || a.pattern || a.query || a.prompt || a.objective || '').toString(); }
      catch { argsStr = (d.arguments || '').slice(0, 120); }
      toolStream.write(`${name}\t${argsStr.replace(/\n/g, ' ').slice(0, 200)}\n`);
    }
  }
  if (fi % 25000 === 0) console.log('progress', fi, '/', frames.length, 'user', userCount, 'asst', asstCount);
}
userStream.end(); asstStream.end(); toolStream.end();

let tOut = '';
for (const [k, v] of [...toolCount.entries()].sort((a, b) => b[1] - a[1])) tOut += `${v}\t${k}\n`;
createWriteStream(`${outDir}/tool_tally.txt`, { flags: 'w' }).end(tOut);
console.log('DONE user', userCount, 'asst', asstCount);
console.log(tOut);
