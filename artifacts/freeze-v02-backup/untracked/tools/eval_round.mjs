// eval_round.mjs — 评估当前渲染效果
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const sleep = ms => new Promise(r => setTimeout(r, ms));
function loadKey() {
  const p = path.join(os.homedir(), '.dsh', '.credentials.yaml');
  return fs.readFileSync(p, 'utf8').match(/TOKENRHYTHM_API_KEY:\s*(\S+)/)[1];
}
function dataUrl(fp) {
  return { type: 'image_url', image_url: { url: `data:image/png;base64,${fs.readFileSync(fp).toString('base64')}` } };
}
async function callOnce(key, messages) {
  const res = await fetch(ENDPOINT, {
    method: 'POST',
    headers: { 'Authorization': `Bearer ${key}`, 'Content-Type': 'application/json' },
    body: JSON.stringify({ model: MODEL, messages, max_tokens: 4000 }),
  });
  const text = await res.text();
  if (res.status === 429 || res.status >= 500) return { retry: true, content: '' };
  if (!res.ok) throw new Error(`API ${res.status}: ${text.slice(0, 300)}`);
  const d = JSON.parse(text);
  const msg = d.choices?.[0]?.message || {};
  return { retry: false, content: typeof msg.content === 'string' ? msg.content : '' };
}
async function main() {
  const label = process.argv[2] || 'Round 1';
  const key = loadKey();
  const portrait = path.resolve(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');
  const views = ['front', 'side', 'back', 'face'];
  const parts = [
    { type: 'text', text: `【这是${label}优化后的渲染】图1=郑吒立绘(目标参考)，图2=正面，图3=侧面，图4=背面，图5=脸部特写。请评估：\n1. 体型是否更像立绘（宽肩细腰、头身比、腿长）？\n2. 染色是否有改进？手臂/腿部是否也被染上颜色了？还是仍然只有躯干有色？\n3. 是否还存在大块"塑料玩具"平涂色？有没有明暗光影了？\n4. 还有什么明显问题？\n5. 请按1-10给"像立绘的程度"打分，并说出最需要继续优化的2-3点。` },
    dataUrl(portrait),
  ];
  for (const v of views) parts.push(dataUrl(path.join(__dirname, 'shots', `render_zhenghai_${v}.png`)));
  for (let a = 0; a < 5; a++) {
    try {
      const r = await callOnce(key, [{ role: 'user', content: parts }]);
      if (!r.retry) { console.log(r.content); return; }
    } catch (e) { if (a === 4) throw e; }
    console.error(`retry ${a+1}...`);
    await sleep(15000);
  }
}
main().catch(e => { console.error('FATAL', e.message); process.exit(1); });
