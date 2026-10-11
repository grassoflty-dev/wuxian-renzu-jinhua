import { readFileSync } from 'node:fs';
import { zstdDecompressSync } from 'node:zlib';

const ZSTD_MAGIC = 4247762216;
function scanZstdFrames(buffer, maxFrames = Infinity) {
  const frames = [];
  let offset = 0;
  while (offset < buffer.length) {
    const start = offset;
    if (buffer.length - offset < 4) return { frames, tornStart: start };
    if (buffer.readUInt32LE(offset) !== ZSTD_MAGIC) throw new Error(`bad magic at ${offset}`);
    offset += 4;
    if (offset === buffer.length) return { frames, tornStart: start };
    const descriptor = buffer.readUInt8(offset); offset += 1;
    if ((descriptor & 24) !== 0) throw new Error('reserved bit');
    const contentSizeFlag = descriptor >>> 6;
    const singleSegment = (descriptor & 32) !== 0;
    const checksum = (descriptor & 4) !== 0;
    const dictionaryFlag = descriptor & 3;
    const dictionaryBytes = dictionaryFlag === 3 ? 4 : dictionaryFlag;
    const contentSizeBytes = contentSizeFlag === 0 ? (singleSegment ? 1 : 0) : (1 << contentSizeFlag);
    const remainingHeaderBytes = (singleSegment ? 0 : 1) + dictionaryBytes + contentSizeBytes;
    if (buffer.length - offset < remainingHeaderBytes) return { frames, tornStart: start };
    offset += remainingHeaderBytes;
    for (;;) {
      if (buffer.length - offset < 3) return { frames, tornStart: start };
      const blockHeader = buffer.readUIntLE(offset, 3); offset += 3;
      const lastBlock = (blockHeader & 1) !== 0;
      const blockType = (blockHeader >>> 1) & 3;
      const blockSize = blockHeader >>> 3;
      if (blockType === 3) throw new Error('reserved block type');
      const payloadBytes = blockType === 1 ? 1 : blockSize;
      if (buffer.length - offset < payloadBytes) return { frames, tornStart: start };
      offset += payloadBytes;
      if (lastBlock) break;
    }
    if (checksum) { if (buffer.length - offset < 4) return { frames, tornStart: start }; offset += 4; }
    frames.push({ start, end: offset });
    if (frames.length === maxFrames) return { frames };
  }
  return { frames };
}

const buf = readFileSync(process.argv[2]);
const { frames, tornStart } = scanZstdFrames(buf);
console.log('frames:', frames.length, 'tornStart:', tornStart, 'fileBytes:', buf.length);

let all = '';
for (const f of frames.slice(0, 6)) {
  const plain = zstdDecompressSync(buf.subarray(f.start, f.end)).toString('utf8');
  all += plain;
  console.log(`=== frame ${f.start}-${f.end} plainBytes=${plain.length}`);
}
// print first few lines of the concatenated plaintext
const lines = all.split('\n');
console.log('TOTAL lines in first 6 frames:', lines.length);
for (let i = 0; i < Math.min(10, lines.length); i++) {
  let o = null; try { o = JSON.parse(lines[i]); } catch (e) { o = { err: String(e) }; }
  const s = JSON.stringify(o);
  console.log(`[${i}] keys=${o && !o.err ? Object.keys(o).join(',') : 'ERR'}`);
  console.log(s.slice(0, 500));
}
