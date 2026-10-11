// qwen_glm_final.mjs — 直接调 tokenrhythm API 验收 glm 通道方块强化最终效果
// 用法: node qwen_glm_final.mjs
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));

const SHOTS = ['front', 'side', 'back', 'face'].map(v => path.join(__dirname, 'shots', `render_zhenghai_${v}.png`));
const LIHUI = path.join(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');

const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 3000;
const RETRY_SLEEP = 15000;
const MAX_ATTEMPTS = 5;
const sleep = ms => new Promise(r => setTimeout(r, ms));

const JUDGE_PROMPT = `前四张图是《郑吒》体素建模的干净展示(1正面 2侧面 3背面 4脸部特写),第5张是2D立绘(黑色短发、粗眉、深炭色T恤、战术长裤、肌肉型7头身、宽肩窄腰)。
本轮做了"方块级细节强化":顶点色方块明度系统(每方块上排顶点×1.06/下排×0.90 纵向渐变)、高度明度梯度(顶亮底暗)、环向涟漪±4.5%、材质开vertexColors、肌肉起伏加强(二头肌/胸腹zTaper)。
请验收:
1. **方块感(0-100,核心)**:放大看躯干/手臂/裤子的连续面——能否逐格数出方块?相邻方块是否有明度色差/接缝?横向分界线和竖向分块是否都清晰?还是整片同色?
2. 整体相似度(0-100) 3. 五官(0-100) 4. 服装配色(0-100) 5. 身体比例(0-100) 6. 肌肉/衣褶凹凸体积感(0-100)
7. 是否有"砖砌感过强"或"斑驳噪点"问题?(明度调制是否过度)
中文输出,简洁,具体位置观察。不要臆造,看图说话。`;

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
  console.log('=== qwen 方块感验收 (glm 通道) ===');
  console.log(r.content || r.reasoning || '(no content)');
})();
