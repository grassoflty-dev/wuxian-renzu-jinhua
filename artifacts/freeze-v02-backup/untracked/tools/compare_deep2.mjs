// compare_deep2.mjs — 简洁版深度对比
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
    body: JSON.stringify({ model: MODEL, messages, max_tokens: 6000 }),
  });
  const text = await res.text();
  if (res.status === 429 || res.status >= 500) return { retry: true, content: '' };
  if (!res.ok) throw new Error(`API ${res.status}: ${text.slice(0, 300)}`);
  const d = JSON.parse(text);
  const msg = d.choices?.[0]?.message || {};
  return { retry: false, content: typeof msg.content === 'string' ? msg.content : '' };
}
async function main() {
  const key = loadKey();
  const portrait = path.resolve(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');
  const front = path.join(__dirname, 'shots', 'render_zhenghai_front.png');
  const side = path.join(__dirname, 'shots', 'render_zhenghai_side.png');
  const parts = [
    { type: 'text', text: '图1:郑吒立绘(目标参考)。图2:体素方块人正面。图3:体素方块人侧面。体素方案：用BoxGeometry方块(约1480块)拼成人形空心环/椭球，然后遍历每个方块世界坐标(x,y)→线性映射到立绘(u,v)取平均像素颜色染方块，侧面按X距离暗化45%。用户反馈"没啥区别，还是方块人"。请从技术角度找出3个最根本的问题，以及对应的修复方向（要具体，比如"头宽从0.48改到0.38"、"染色算法从正面投影改为按面朝向分别采样"）。简洁列出：\n\n问题1（体型）：... 修复方向：...\n问题2（染色）：... 修复方向：...\n问题3（几何/方块）：... 修复方向：...\n额外建议：...' },
    dataUrl(portrait),
    dataUrl(front),
    dataUrl(side),
  ];
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
