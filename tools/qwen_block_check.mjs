// qwen_block_check.mjs — 直接调 tokenrhythm API 验证 DS 通道方块感(不走子代理 read_image,路由不稳定时可靠)
// 用法: node qwen_block_check.mjs
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));

const SHOTS = ['front', 'side', 'back', 'face'].map(v => path.join(__dirname, 'shots', `render_ds_zhenghai_${v}.png`));
const LIHUI = path.join(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');

const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 3000;
const RETRY_SLEEP = 15000;
const MAX_ATTEMPTS = 5;
const sleep = ms => new Promise(r => setTimeout(r, ms));

const JUDGE_PROMPT = `前四张图是《郑吒》体素建模的干净展示(1正面 2侧面 3背面 4脸部特写),第5张是2D立绘(黑色短发、粗眉、深炭色T恤、战术长裤、肌肉型7头身、宽肩窄腰)。
本轮刚做了"方块级细节强化":每块方块按位置加了明度调制(顶光±10% + 环向相邻方块明度交替±6% + 抖动),染色保留方块差异,肌肉起伏加强。
请重点验收"方块感"(用户最关心):放大看躯干/手臂/裤子的连续面——
1. 能否逐格数出方块?相邻方块是否有明度色差/接缝?(给出具体位置观察,如"上臂可见深浅交替"或"躯干仍然一整块色")
2. 方块感评分(0-100)
3. 整体相似度(0-100) 4. 五官(0-100) 5. 服装配色(0-100) 6. 身体比例(0-100)
7. 肌肉/衣褶是否有凹凸体积感?
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
  console.log('=== qwen 方块感验收 (DS 通道) ===');
  console.log(r.content || r.reasoning || '(no content)');
})();
