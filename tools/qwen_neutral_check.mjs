// qwen_neutral_check.mjs — 中立验收: 不带"体素/方块"预设词, 直接看"像不像精致3D角色"
// 用法: node qwen_neutral_check.mjs <glm|ds>
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const which = process.argv[2] === 'glm' ? 'glm' : 'ds';
const PREFIX = which === 'glm' ? 'render_zhenghai_' : 'render_ds_zhenghai_';
const SHOTS = ['front', 'side', 'back', 'face'].map(v => path.join(__dirname, 'shots', `${PREFIX}${v}.png`));
const LIHUI = path.join(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');

const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 3000;
const RETRY_SLEEP = 15000;
const MAX_ATTEMPTS = 5;
const sleep = ms => new Promise(r => setTimeout(r, ms));

const JUDGE_PROMPT = `前四张图是一个3D角色的展示(1正面 2侧面 3背面 4脸部特写),第5张是该角色的2D参考立绘(黑色短发、粗眉、深炭色T恤、战术长裤、肌肉型7头身、宽肩窄腰)。
这是一个"低多边形但追求精致"的角色模型,1000+ 多边形用于精细轮廓。
请客观评估(不预设任何风格):
1. 模型整体是否精致?像"精心制作的3D角色"还是"粗糙的低模"?(0-100)
2. 肌肉/身体轮廓是否饱满圆润?(0-100)
3. 明暗过渡是否平滑自然?(0-100)
4. 服装材质质感如何?像布料吗?(0-100)
5. 与2D立绘的整体相似度(0-100)
6. 五官清晰度/还原度(0-100)
7. 身体比例(0-100)
8. 最明显的2-3个缺点(具体位置)
请直说,不用客气。中文输出。`;

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
  console.log(`=== qwen 中立验收 (${which} 通道) ===`);
  console.log(r.content || r.reasoning || '(no content)');
})();
