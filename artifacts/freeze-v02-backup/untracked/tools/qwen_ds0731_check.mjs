// qwen_ds0731_check.mjs — DS0731 部位级分离版专项验收(斜视头部/手臂独立/多视角平滑)
// 用法: node qwen_ds0731_check.mjs
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PREFIX = 'render_sprite_ds0731_zhenghai_';
const SHOTS = ['front', 'upper', 'side', 'profile'].map(v => path.join(__dirname, 'shots', `${PREFIX}${v}.png`));
const LIHUI = path.join(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');

const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 4000;
const RETRY_SLEEP = 15000;
const MAX_ATTEMPTS = 5;
const sleep = ms => new Promise(r => setTimeout(r, ms));

const JUDGE_PROMPT = `前四张图是同一个3D角色(郑吒,黑色短发/深炭色T恤/战术长裤/肌肉型)从不同视角的渲染:1正面 2约60度左斜视(上半身) 3正侧面 4约45度斜侧。第5张是原始2D立绘。
这个模型用"三视图视觉外壳"重建(按部位独立建模:头/躯干/左右臂/左右腿)。
请逐项验收(打分+一句话理由):
1. **斜视头部(0-100,最重要)**:看第2/4张的头部——头部是圆润的立体球面/椭球面(有自然的弧线过渡),还是像"盒子两个面各贴一张图"(正面平面+侧面平面,中间有棱线/方角)?
2. **斜视脸部贴图(0-100)**:斜视时脸颊/颧骨到侧脸是否有连续过渡?有没有看到明显的贴图边界/接缝/重影?
3. **手臂独立(0-100)**:手臂是否粗细正常(约躯干1/4)、与躯干之间有正确空隙?还是手臂管过粗(像含了身体投影)?
4. **多视角平滑(0-100)**:四个视角整体是否平滑连续,有无突兀台阶/空洞/棱角?
5. **主要缺点**:具体到部位(头部/手臂/躯干/腿)和视角。
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
  console.log('=== qwen DS0731 部位分离版验收 ===');
  console.log(r.content || r.reasoning || '(no content)');
})();
