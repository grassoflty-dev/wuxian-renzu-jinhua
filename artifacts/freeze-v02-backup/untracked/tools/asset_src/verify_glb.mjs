#!/usr/bin/env node
/**
 * verify_glb.mjs - dependency-free structural validator + animation inspector for .glb / .gltf
 *
 * Checks:
 *   - GLB container: magic 'glTF', version 2, declared length == file size, chunk layout
 *   - JSON chunk parses, asset.version == "2.0"
 *   - every bufferView / accessor reference resolves inside the BIN chunk
 *   - animations[] names + per-clip duration (seconds) + channel/sampler counts
 *   - skins[] joint counts, mesh/primitive triangle counts, material count
 *   - POSITION accessor min/max bounding box (for scale + up-axis inspection)
 *   - images: embedded (bufferView) vs external (uri)
 *
 * Usage: node verify_glb.mjs <file-or-dir> [more...]
 *        node verify_glb.mjs --json <file-or-dir>     (machine readable only)
 */

import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, extname, basename } from 'node:path';

const GLB_MAGIC = 0x46546c67;      // 'glTF' little-endian
const CHUNK_JSON = 0x4e4f534a;     // 'JSON'
const CHUNK_BIN  = 0x004e4942;     // 'BIN\0'

const COMPONENT_BYTES = { 5120:1, 5121:1, 5122:2, 5123:2, 5125:4, 5126:4 };
const TYPE_COMPONENTS  = { SCALAR:1, VEC2:2, VEC3:3, VEC4:4, MAT2:4, MAT3:9, MAT4:16 };

function walk(p, out = []) {
  const st = statSync(p);
  if (st.isDirectory()) {
    for (const e of readdirSync(p)) walk(join(p, e), out);
  } else if (['.glb', '.gltf'].includes(extname(p).toLowerCase())) {
    out.push(p);
  }
  return out;
}

function parseGlb(buf) {
  const errors = [];
  const warnings = [];
  if (buf.length < 12) { errors.push('file shorter than 12-byte GLB header'); return { errors, warnings, json: null, bin: null }; }
  const magic = buf.readUInt32LE(0);
  const version = buf.readUInt32LE(4);
  const declaredLength = buf.readUInt32LE(8);
  if (magic !== GLB_MAGIC) errors.push('bad magic 0x' + magic.toString(16) + ' (expected 0x46546c67 "glTF")');
  if (version !== 2) errors.push('unexpected container version ' + version + ' (expected 2)');
  if (declaredLength !== buf.length) errors.push('declared length ' + declaredLength + ' != actual ' + buf.length);

  let offset = 12;
  let json = null;
  let bin = null;
  const chunks = [];
  while (offset + 8 <= buf.length) {
    const cl = buf.readUInt32LE(offset);
    const ct = buf.readUInt32LE(offset + 4);
    const body = buf.subarray(offset + 8, offset + 8 + cl);
    chunks.push({ type: ct === CHUNK_JSON ? 'JSON' : ct === CHUNK_BIN ? 'BIN' : '0x' + ct.toString(16), length: cl });
    if (cl % 4 !== 0) warnings.push('chunk length ' + cl + ' is not 4-byte aligned');
    if (ct === CHUNK_JSON) {
      try { json = JSON.parse(body.toString('utf8')); }
      catch (e) { errors.push('JSON chunk does not parse: ' + e.message); }
    } else if (ct === CHUNK_BIN) {
      bin = body;
    }
    offset += 8 + cl;
  }
  if (!json) errors.push('no JSON chunk found');
  if (offset !== buf.length) warnings.push('trailing bytes after last chunk: ' + (buf.length - offset));
  return { errors, warnings, json, bin, chunks, version, declaredLength };
}

function analyseJson(json, bin, filePath) {
  const errors = [];
  const warnings = [];
  const info = {};

  if (json.asset?.version !== '2.0') errors.push('asset.version is ' + JSON.stringify(json.asset?.version) + ' (expected "2.0")');
  info.generator = json.asset?.generator ?? null;
  info.copyright = json.asset?.copyright ?? null;

  const accessors = json.accessors ?? [];
  const bufferViews = json.bufferViews ?? [];
  const meshes = json.meshes ?? [];

  // ---- bufferView / accessor bounds ----
  for (let i = 0; i < bufferViews.length; i++) {
    const bv = bufferViews[i];
    const bufDef = (json.buffers ?? [])[bv.buffer];
    if (!bufDef) { errors.push('bufferView[' + i + '] references missing buffer ' + bv.buffer); continue; }
    const avail = bv.buffer === 0 && bin ? bin.length : (bufDef.byteLength ?? 0);
    const end = (bv.byteOffset ?? 0) + (bv.byteLength ?? 0);
    if (end > avail) errors.push('bufferView[' + i + '] overruns buffer (' + end + ' > ' + avail + ')');
  }
  for (let i = 0; i < accessors.length; i++) {
    const a = accessors[i];
    if (a.bufferView === undefined) continue;
    const bv = bufferViews[a.bufferView];
    if (!bv) { errors.push('accessor[' + i + '] references missing bufferView ' + a.bufferView); continue; }
    const cBytes = COMPONENT_BYTES[a.componentType];
    const cCount = TYPE_COMPONENTS[a.type];
    if (!cBytes || !cCount) { errors.push('accessor[' + i + '] bad componentType/type'); continue; }
    const stride = bv.byteStride ?? cBytes * cCount;
    const need = (a.count - 1) * stride + cBytes * cCount;
    if (need > bv.byteLength) errors.push('accessor[' + i + '] needs ' + need + ' bytes but bufferView has ' + bv.byteLength);
  }

  // ---- triangles / primitives ----
  let triangles = 0, vertices = 0, primitives = 0, skinnedPrims = 0;
  for (const mesh of meshes) {
    for (const p of mesh.primitives ?? []) {
      primitives++;
      const posAcc = p.attributes?.POSITION;
      if (posAcc !== undefined && accessors[posAcc]) vertices += accessors[posAcc].count;
      if (p.indices !== undefined && accessors[p.indices]) triangles += Math.floor(accessors[p.indices].count / 3);
      else if (posAcc !== undefined && accessors[posAcc]) triangles += Math.floor(accessors[posAcc].count / 3);
      if (p.attributes?.JOINTS_0 !== undefined) skinnedPrims++;
      const mode = p.mode ?? 4;
      if (mode !== 4 && mode !== undefined) warnings.push('primitive mode ' + mode + ' (not TRIANGLES) in mesh "' + (mesh.name ?? '?') + '"');
    }
  }
  info.meshes = meshes.length;
  info.primitives = primitives;
  info.skinnedPrimitives = skinnedPrims;
  info.triangles = triangles;
  info.vertices = vertices;
  info.materials = (json.materials ?? []).length;
  info.nodes = (json.nodes ?? []).length;

  // ---- bbox from POSITION accessors ----
  const bbox = { min: [Infinity, Infinity, Infinity], max: [-Infinity, -Infinity, -Infinity] };
  let hasBbox = false;
  for (const mesh of meshes) {
    for (const p of mesh.primitives ?? []) {
      const a = accessors[p.attributes?.POSITION];
      if (a?.min && a?.max) {
        hasBbox = true;
        for (let k = 0; k < 3; k++) {
          bbox.min[k] = Math.min(bbox.min[k], a.min[k]);
          bbox.max[k] = Math.max(bbox.max[k], a.max[k]);
        }
      }
    }
  }
  if (hasBbox) {
    info.bbox = {
      min: bbox.min.map(v => +v.toFixed(4)),
      max: bbox.max.map(v => +v.toFixed(4)),
      size: bbox.max.map((v, i) => +(v - bbox.min[i]).toFixed(4)),
      heightY: +(bbox.max[1] - bbox.min[1]).toFixed(4)
    };
  } else {
    warnings.push('no accessor min/max on POSITION - cannot infer scale');
  }

  // ---- skins ----
  const skins = json.skins ?? [];
  info.skins = skins.length;
  info.joints = skins.reduce((s, k) => s + (k.joints?.length ?? 0), 0);
  if (skins.length && info.skinnedPrimitives === 0) warnings.push('skins present but no primitive has JOINTS_0');
  if (info.skinnedPrimitives && skins.length === 0) warnings.push('JOINTS_0 present but no skins[] entry');

  // ---- animations ----
  const animations = json.animations ?? [];
  info.animationCount = animations.length;
  info.animations = animations.map((anim, idx) => {
    let duration = 0;
    let chans = anim.channels?.length ?? 0;
    const paths = new Set();
    const targets = new Set();
    for (const ch of anim.channels ?? []) {
      paths.add(ch.target?.path);
      if (ch.target?.node !== undefined) targets.add(ch.target.node);
      const sampler = anim.samplers?.[ch.sampler];
      if (!sampler) { errors.push('animation[' + idx + '] channel references missing sampler'); continue; }
      const input = accessors[sampler.input];
      if (input?.max?.[0] !== undefined) duration = Math.max(duration, input.max[0]);
    }
    const samplerCount = anim.samplers?.length ?? 0;
    let keys = 0;
    for (const s of anim.samplers ?? []) keys += accessors[s.input]?.count ?? 0;
    void samplerCount;
    return {
      index: idx,
      name: anim.name ?? null,
      durationSec: +duration.toFixed(4),
      channels: chans,
      samplers: samplerCount,
      totalKeys: keys,
      paths: [...paths].sort(),
      animatedNodes: targets.size
    };
  });
  if (animations.length === 0) warnings.push('no animations[] - static model');

  // ---- images / textures ----
  const images = json.images ?? [];
  info.images = images.length;
  info.imagesEmbedded = images.filter(i => i.bufferView !== undefined).length;
  info.imagesExternal = images.filter(i => i.uri !== undefined).length;
  const externalUris = [...new Set(images.filter(i => i.uri).map(i => i.uri))];
  info.externalImageUris = externalUris;
  if (externalUris.length) warnings.push('references ' + externalUris.length + ' external image file(s): ' + externalUris.join(', '));

  // ---- extensions ----
  info.extensionsUsed = json.extensionsUsed ?? [];
  info.extensionsRequired = json.extensionsRequired ?? [];
  for (const e of info.extensionsRequired) {
    if (!['KHR_materials_unlit', 'KHR_texture_transform', 'KHR_mesh_quantization'].includes(e)) {
      warnings.push('requires extension ' + e + ' - confirm Babylon.js loader support');
    }
  }
  return { errors, warnings, info };
}

function analyseGltf(filePath) {
  const errors = [];
  const warnings = [];
  let json;
  try { json = JSON.parse(readFileSync(filePath, 'utf8')); }
  catch (e) { return { errors: ['JSON does not parse: ' + e.message], warnings, info: {} }; }
  const r = analyseJson(json, null, filePath);
  errors.push(...r.errors);
  warnings.push(...r.warnings);
  const buffers = json.buffers ?? [];
  for (const b of buffers) {
    if (b.uri && !b.uri.startsWith('data:')) warnings.push('external buffer: ' + b.uri);
  }
  if (buffers.some(b => b.uri === undefined)) errors.push('buffer without uri in a .gltf file');
  r.info.container = 'gltf (JSON + external .bin)';
  return { errors, warnings, info: r.info };
}

function run(filePath) {
  const buf = readFileSync(filePath);
  let res;
  if (extname(filePath).toLowerCase() === '.glb') {
    const c = parseGlb(buf);
    res = c.json ? analyseJson(c.json, c.bin, filePath) : { errors: c.errors, warnings: c.warnings, info: {} };
    res.errors = [...c.errors, ...res.errors];
    res.warnings = [...c.warnings, ...res.warnings];
    res.info.container = 'glb';
    res.info.chunks = c.chunks;
  } else {
    res = analyseGltf(filePath);
  }
  res.file = filePath;
  res.bytes = buf.length;
  res.ok = res.errors.length === 0;
  return res;
}

const argv = process.argv.slice(2);
const asJson = argv.includes('--json');
const targets = argv.filter(a => a !== '--json');
if (targets.length === 0) { console.error('usage: node verify_glb.mjs [--json] <file-or-dir> [...]'); process.exit(2); }

const files = [];
for (const t of targets) {
  const st = statSync(t);
  if (st.isDirectory()) files.push(...walk(t)); else files.push(t);
}
files.sort();

const reports = files.map(f => {
  try { return run(f); }
  catch (e) { return { file: f, ok: false, errors: ['unreadable: ' + e.message], warnings: [], info: {}, bytes: 0 }; }
});

if (asJson) {
  console.log(JSON.stringify(reports, null, 2));
} else {
  for (const r of reports) {
    console.log('='.repeat(78));
    console.log((r.ok ? '[PASS] ' : '[FAIL] ') + basename(r.file) + '   ' + r.bytes.toLocaleString() + ' bytes');
    console.log('  path: ' + r.file);
    const i = r.info;
    console.log('  container=' + i.container + '  generator=' + (i.generator ?? 'n/a'));
    console.log('  meshes=' + i.meshes + '  prims=' + i.primitives + '  tris=' + (i.triangles ?? 0).toLocaleString() +
                '  verts=' + (i.vertices ?? 0).toLocaleString() + '  materials=' + i.materials +
                '  nodes=' + i.nodes + '  skins=' + i.skins + '  joints=' + i.joints);
    if (i.bbox) console.log('  bbox size XYZ = [' + i.bbox.size.join(', ') + ']  heightY=' + i.bbox.heightY);
    console.log('  images=' + i.images + ' (embedded=' + i.imagesEmbedded + ', external=' + i.imagesExternal + ')');
    if (i.extensionsUsed?.length) console.log('  extensionsUsed: ' + i.extensionsUsed.join(', '));
    console.log('  ANIMATIONS (' + i.animationCount + '):');
    if (!i.animationCount) console.log('    (none)');
    for (const a of i.animations ?? []) {
      console.log('    [' + String(a.index).padStart(2) + '] "' + a.name + '"  ' + a.durationSec + 's  channels=' + a.channels +
                  '  keys=' + a.totalKeys + '  nodes=' + a.animatedNodes + '  paths=[' + a.paths.join(',') + ']');
    }
    for (const e of r.errors) console.log('  ERROR: ' + e);
    for (const w of r.warnings) console.log('  warn : ' + w);
  }
  const pass = reports.filter(r => r.ok).length;
  console.log('='.repeat(78));
  console.log('SUMMARY: ' + pass + '/' + reports.length + ' passed structural validation');
}
process.exit(reports.every(r => r.ok) ? 0 : 1);
