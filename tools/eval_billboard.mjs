// eval_billboard.mjs — eval billboard version
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const sleep = ms => new Promise(r => setTimeout(r, ms));
function loadKey() {
  return fs.readFileSync(path.join(os.homedir(), '.dsh', '.credentials.yaml'), 'utf8').match(/TOKENRHYTHM_API_KEY:\s*(\S+)/)[1];
}
function dataUrl(fp) { return { type: 'image_url', image_url: { url: `data:image/png;base64,${fs.readFileSync(fp).toString('base64')}` } }; }
async function callOnce(key, parts) {
  const res = await fetch(ENDPOINT, {
    method: 'POST', headers: { 'Authorization': `Bearer ${key}`, 'Content-Type': 'application/json' },
    body: JSON.stringify({ model: MODEL, messages: [{ role: 'user', content: parts }], max_tokens: 2000 }),
  });
  const text = await res.text();
  if (res.status === 429 || res.status >= 500) return { retry: true, content: '' };
  if (!res.ok) throw new Error(`API ${res.status}: ${text.slice(0,300)}`);
  const d = JSON.parse(text); const msg = d.choices?.[0]?.message || {};
  return { retry: false, content: typeof msg.content === 'string' ? msg.content : '' };
}
async function main() {
  const key = loadKey();
  const portrait = path.resolve(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');
  const views = ['front', 'side', 'back', 'face'];
  const parts = [
    { type: 'text', text: '图1=郑吒立绘。图2-5=体素身体+立绘头脸billboard混搭版(头是2D sprite,身体是3D voxel)。请评估：\n1. 头身比是否接近立绘？\n2. 头部用了立绘脸后整体像立绘的程度提升多少？\n3. 还有什么问题？\n4. 打分(1-10)。' },
    dataUrl(portrait),
  ];
  for (const v of views) parts.push(dataUrl(path.join(__dirname, 'shots', `render_zhenghai_billboard_${v}.png`)));
  for (let a = 0; a < 5; a++) {
    try { const r = await callOnce(key, parts); if (!r.retry) { console.log(r.content); return; } } catch (e) { if (a===4) throw e; }
    console.error(`retry ${a+1}...`); await sleep(15000);
  }
}
main().catch(e => { console.error('FATAL', e.message); process.exit(1); });
