// qwen_sprite_check.mjs — 验收 SpriteStacking 2.5D 原型(正面还原度 + 侧面体积感)
// 用法: node qwen_sprite_check.mjs <ds|glm>   (默认 ds)
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const which = process.argv[2] === 'glm' ? 'glm' : 'ds';
const PREFIX = which === 'glm' ? 'render_sprite_glm_zhenghai_' : 'render_sprite_ds_zhenghai_';
const SHOTS = ['front', 'side', 'back', 'face'].map(v => path.join(__dirname, 'shots', `${PREFIX}${v}.png`));
const LIHUI = path.join(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');

const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 3000;
const RETRY_SLEEP = 15000;
const MAX_ATTEMPTS = 5;
const sleep = ms => new Promise(r => setTimeout(r, ms));

const JUDGE_PROMPT = `前四张图是一个"2.5D 立绘投影(SpriteStacking)"角色展示(1正面 2侧面 3背面 4脸部特写),第5张是原始2D立绘(黑色短发、粗眉、深炭色T恤、战术长裤、肌肉型7头身、宽肩窄腰)。
技术原理:把2D立绘沿深度堆叠成多层切片,正面看=立绘本身(细节像素级还原),侧面看=有厚度层次。
请验收:
1. **正面还原度(0-100,核心)**:正面是否和立绘一模一样?(发型/五官/手/脚细节是否精确还原,有没有残迹/虚线/边框/破损)
2. **侧面体积感(0-100)**:侧面是否看起来有实体厚度?(还是一张纸片?侧面是平滑层次还是奇怪的条纹?)
3. 背面效果(0-100):是否可接受?
4. 脸部特写清晰度(0-100)
5. 整体精致度(0-100):作为角色表现,是否比"方块人"更好?
6. 主要缺点(具体位置)
中文输出,简洁。不要臆造,看图说话。`;

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
  console.log(`=== qwen SpriteStacking 验收 (${which} 通道) ===`);
  console.log(r.content || r.reasoning || '(no content)');
})();
