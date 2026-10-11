// qwen_refine_check.mjs — 验收"精致化"效果(去方块感后): 平滑度/轮廓圆润度/材质精致度
// 用法: node qwen_refine_check.mjs <ds|glm>   (默认 ds)
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

const JUDGE_PROMPT = `前四张图是《郑吒》体素建模的干净展示(1正面 2侧面 3背面 4脸部特写),第5张是2D立绘(黑色短发、粗眉、深炭色T恤、战术长裤、肌肉型7头身、宽肩窄腰)。
用户要求"不要体素/方块感,以精致为最高要求"——本轮刚做了"去方块化精致化":移除方块级明度调制(棋盘格/砖缝)、压淡AO、材质柔化(布料感)、四肢环层加密(截面9-10边)使轮廓圆润、躯干S曲线。
请按"精致度"验收(用户最新诉求,不再看方块感):
1. **明暗顺滑度(0-100)**:连续面(躯干/手臂/腿)的明暗是否平滑渐变?是否还有棋盘格/砖缝/斑驳噪点?还是"每一块都能数出来"?
2. **轮廓圆润度(0-100)**:肌肉(胸/腹/二头肌/大腿)是否饱满圆润?躯干胸→腰→胯是否有S形曲线?四肢粗细过渡是否连续无阶梯?
3. **材质精致度(0-100)**:T恤是否像布料(非石头/金属)?肤色是否平滑健康微光泽?
4. **整体精致度(0-100)**:综合观感像"精致低多边形角色"还是"Minecraft方块人"?
5. 整体相似度(0-100) 6. 五官(0-100) 7. 身体比例(0-100)
8. 仍存在的问题(具体位置)
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
  console.log(`=== qwen 精致度验收 (${which} 通道) ===`);
  console.log(r.content || r.reasoning || '(no content)');
})();
