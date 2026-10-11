// qwen_desc_model.mjs — 用 qwen3.7-flash 客观描述单个体素建模渲染图（不打分）
// 用法: node qwen_desc_model.mjs <name>   读取 tools/shots/render_<name>_{front,side,back,face}.png
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const name = process.argv[2] || 'zhenghai';
const views = ['front', 'side', 'back', 'face'];
const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const sleep = ms => new Promise(r => setTimeout(r, ms));

const PROMPT = `这是《无限恐怖》主角郑吒的体素（Minecraft方块风）3D建模渲染图，共4张：第1张正面全身、第2张侧面、第3张背面、第4张正面脸部特写，纯白背景。
请客观、具体地描述你看到的建模（中文，分点）：
1. 发型和发色
2. 脸：五官形态（眉/眼/鼻/嘴）、肤色、有没有胡子
3. 上半身衣服：颜色、款式、领口、胸前有无纹理或凸起
4. 裤子和腰带：颜色、有没有口袋/绑带
5. 体型：胖瘦、头身比、肩宽、手臂、腿
6. 鞋子
7. 整体观感一句话（像不像肌肉壮汉/硬汉）
不要打分，只客观描述，具体到颜色和形状。`;

function loadKey() {
  const p = path.join(os.homedir(), '.dsh', '.credentials.yaml');
  const m = fs.readFileSync(p, 'utf8').match(/TOKENRHYTHM_API_KEY:\s*(\S+)/);
  if (!m) throw new Error('no key');
  return m[1];
}
function dataUrl(fp) {
  return { type: 'image_url', image_url: { url: `data:image/png;base64,${fs.readFileSync(fp).toString('base64')}` } };
}
async function callOnce(key, parts) {
  const res = await fetch(ENDPOINT, {
    method: 'POST',
    headers: { 'Authorization': `Bearer ${key}`, 'Content-Type': 'application/json' },
    body: JSON.stringify({ model: MODEL, messages: [{ role: 'user', content: [{ type: 'text', text: PROMPT }, ...parts] }], max_tokens: 3000 }),
  });
  const text = await res.text();
  if (res.status === 429 || res.status >= 500 || /LITELLM_UNAVAILABLE|SERVICE_BUSY|overloaded|GATEWAY-TIME|timeout|timed out/i.test(text)) {
    return { retry: true, content: '', reasoning: '' };
  }
  if (!res.ok) throw new Error(`API ${res.status}: ${text.slice(0, 300)}`);
  const d = JSON.parse(text);
  const msg = d.choices?.[0]?.message || {};
  return { retry: false, content: typeof msg.content === 'string' ? msg.content : '', reasoning: typeof msg.reasoning_content === 'string' ? msg.reasoning_content : '' };
}

async function main() {
  const key = loadKey();
  const parts = views.map(v => {
    const fp = path.join(__dirname, 'shots', `render_${name}_${v}.png`);
    if (!fs.existsSync(fp)) throw new Error('missing ' + fp);
    return dataUrl(fp);
  });
  let r = { retry: true, content: '', reasoning: '' };
  for (let a = 0; a < 5; a++) {
    try { r = await callOnce(key, parts); if (!r.retry) break; } catch (e) { if (a === 4) throw e; }
    console.error(`  retry ${a + 1} (429/5xx) ...`);
    await sleep(15000);
  }
  const body = (r.content || '').trim() || (r.reasoning || '').trim();
  console.log(body);
}
main().catch(e => { console.error('FATAL', e.message); process.exit(1); });
