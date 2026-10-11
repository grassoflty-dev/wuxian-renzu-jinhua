// DISABLED: historical cloud entry point; not a local model adapter.
// Fail closed on execution AND import, before credentials, requests or output writes.
// No environment variable / CLI override is permitted.
throw Object.assign(new Error('LOCAL_MEDIA_ONLY: cloud image generation is disabled; use a separately verified local deployment. No cloud fallback.'), { code: 'LOCAL_MEDIA_ONLY' });

// Historical implementation retained below; unreachable under the current policy.
// 图片生成 CLI：node img_gen.mjs <prompt> <outPath> [size]
// 用基元律动 wan2.7-image；支持尺寸 768x768/1024x1024/768x1024/1024x768/1280x720/720x1280/1328x1328
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
const key = fs.readFileSync(path.join(os.homedir(), '.dsh', '.credentials.yaml'), 'utf8').match(/TOKENRHYTHM_API_KEY:\s*(\S+)/)[1];
const [prompt, out, size] = process.argv.slice(2);
if (!prompt || !out) { console.log('usage: node img_gen.mjs <prompt> <outPath> [size]'); process.exit(1); }
const res = await fetch('https://tokenrhythm.studio/v1/images/generations', {
  method: 'POST',
  headers: { 'Authorization': 'Bearer ' + key, 'Content-Type': 'application/json' },
  body: JSON.stringify({ model: 'wan2.7-image', prompt, n: 1, size: size || '768x1024' })
});
const t = await res.text();
if (!res.ok) { console.log('HTTP ' + res.status + ' :: ' + t.slice(0, 400)); process.exit(1); }
const d = JSON.parse(t);
const url = d.data?.[0]?.url;
if (!url) { console.log('no url: ' + t.slice(0, 300)); process.exit(1); }
const bin = await fetch(url);
const buf = Buffer.from(await bin.arrayBuffer());
fs.mkdirSync(path.dirname(out), { recursive: true });
fs.writeFileSync(out, buf);
console.log('OK ' + out + ' ' + buf.length + ' bytes');