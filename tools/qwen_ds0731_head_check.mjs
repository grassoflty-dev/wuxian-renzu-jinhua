// qwen_ds0731_head_check.mjs — DS0731 头部椭球化专项验收(head45/head135 斜视特写)
// 用法: node qwen_ds0731_head_check.mjs
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PREFIX = 'render_sprite_ds0731_zhenghai_';
const SHOTS = ['head45', 'head135', 'upper', 'face', 'side'].map(v => path.join(__dirname, 'shots', `${PREFIX}${v}.png`));
const LIHUI = path.join(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');

const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 4000;
const RETRY_SLEEP = 15000;
const MAX_ATTEMPTS = 5;
const sleep = ms => new Promise(r => setTimeout(r, ms));

const JUDGE_PROMPT = `前五张是一个3D角色(郑吒,黑色短发/深色T恤)的头部与上半身渲染:1头部45度左斜视特写 2头部135度右后斜视特写 3上半身60度斜视 4脸部正面特写 5正侧面。第6张是原始2D立绘(正面)。
这个模型头部按"真实椭球体"建模(三维椭圆曲面,非方形),皮肤沿椭球曲面包裹。
请逐项验收(打分+一句话理由):
1. **头部几何立体感(0-100,最重要)**:看第1/2张,头部轮廓是圆润的球面/椭球弧线(像真实人头),还是方角/棱线/盒状?头顶到脸颊到下巴的轮廓是否平滑弧线?
2. **斜视脸部贴图连续性(0-100)**:第1张45度斜视时,脸颊/颧骨→侧脸的颜色过渡是否连续?有没有明显的贴图接缝/颜色断层/重影/两条棱?
3. **五官自然度(0-100)**:第1/4张的眼睛/眉毛/嘴是否正常贴在人脸位置,无错位/拉伸变形?
4. **多视角一致性(0-100)**:第1/2/3/5张同一头部在不同角度是否一致连续,无跳变?
5. **主要缺点(具体到视角和部位)**
中文输出,简洁,不要臆造,看图说话。`;

function loadKey() {
  const p = path.join(os.homedir(), '.dsh', '.credentials.yaml');
  const raw = fs.readFileSync(p, 'utf8');
  const m = raw.match(/TOKENRHYTHM_API_KEY:\s*(\S+)/);
  if (!m) throw new Error('TOKENRHYTHM_API_KEY not found');
  return m[1];
}
function dataUrl(fp) {
  const b64 = fs.readFileSync(fp).toString('base64');
  return { type: 'image_url', image_url: { url: `data:image/png;base64,${b64}` } };
}
async function callOnce(key, parts) {
  const body = { model: MODEL, messages: [{ role: 'user', content: [{ type: 'text', text: JUDGE_PROMPT }, ...parts] }], max_tokens: MAX_TOKENS };
  const res = await fetch(ENDPOINT, { method: 'POST', headers: { 'Authorization': `Bearer ${key}`, 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  if (res.status === 429) return { retry: true, reason: '429', content: '', reasoning: '' };
  const text = await res.text();
  if (!res.ok) {
    if (res.status === 429 || res.status >= 500 || /LITELLM_UNAVAILABLE|SERVICE_BUSY|overloaded|GATEWAY-TIME|timeout|timed out|MODEL_NOT_AVAILABLE|Authentication/i.test(text)) return { retry: true, reason: `${res.status}`, content: '', reasoning: '' };
    throw new Error(`API error ${res.status}: ${text.slice(0, 500)}`);
  }
  let data; try { data = JSON.parse(text); } catch { throw new Error('bad json'); }
  const msg = data.choices?.[0]?.message || {};
  return { retry: false, reason: '', content: typeof msg.content === 'string' ? msg.content : '', reasoning: typeof msg.reasoning_content === 'string' ? msg.reasoning_content : '' };
}
async function callWithRetry(key, parts) {
  for (let a = 0; a < MAX_ATTEMPTS; a++) {
    const r = await callOnce(key, parts);
    if (!r.retry) return r;
    console.log(`attempt ${a + 1} retry (${r.reason})`);
    await sleep(RETRY_SLEEP);
  }
  return { retry: false, reason: 'giveup', content: '', reasoning: '' };
}
(async () => {
  const key = loadKey();
  for (const fp of [...SHOTS, LIHUI]) if (!fs.existsSync(fp)) { console.error('MISSING:', fp); process.exit(1); }
  const r = await callWithRetry(key, [...SHOTS, LIHUI].map(dataUrl));
  console.log('=== qwen DS0731 头部椭球专项验收 ===');
  console.log(r.content || r.reasoning || '(no content)');
})();
