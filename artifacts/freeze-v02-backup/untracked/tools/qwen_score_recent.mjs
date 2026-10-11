// qwen_score_recent.mjs — 用 qwen3.7-flash 直接评分最新一轮郑吒渲染(不走子代理,作为备份通道)
// 用法: node qwen_score_recent.mjs
// 输入: tools/shots/render_zhenghai_{front,side,back,face}.png + 立绘 pc_zhengzha.png
// 输出: 控制台报告
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

const JUDGE_PROMPT = `前四张图是《郑吒》体素染色建模的干净展示图(第1张正面全身、第2张侧面全身、第3张背面全身、第4张正面脸部特写),第5张是郑吒的2D立绘(黑色短发、粗眉、深炭色T恤、战术长裤、肌肉型约7头身、宽肩窄腰)。
请按"建模与立绘的相似度"打分,中文简洁:
1. 相似度总分(0-100,100=完全一样):给一个明确数字。
2. 正面/侧面/背面三视图各方向哪里像、哪里不像。
3. 发型、服装颜色、分段、五官、体型各打几分并说明。
4. 还剩哪些细节差距(具体列出,供下一轮优化)。
5. 下一轮最该先改的 3 个细节。`;

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
async function callOnce(key, imageParts) {
  const body = {
    model: MODEL,
    messages: [{ role: 'user', content: [{ type: 'text', text: JUDGE_PROMPT }, ...imageParts] }],
    max_tokens: MAX_TOKENS,
  };
  const res = await fetch(ENDPOINT, {
    method: 'POST',
    headers: { 'Authorization': `Bearer ${key}`, 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (res.status === 429) return { retry: true, reason: '429', content: '', reasoning: '' };
  const text = await res.text();
  if (!res.ok) {
    if (res.status === 429 || res.status >= 500 || /LITELLM_UNAVAILABLE|SERVICE_BUSY|overloaded|GATEWAY-TIME|timeout|timed out|MODEL_NOT_AVAILABLE/i.test(text)) {
      return { retry: true, reason: `${res.status}`, content: '', reasoning: '' };
    }
    throw new Error(`API error ${res.status}: ${text.slice(0, 500)}`);
  }
  let data; try { data = JSON.parse(text); } catch { throw new Error('bad json'); }
  const msg = data.choices?.[0]?.message || {};
  return { retry: false, reason: '', content: typeof msg.content === 'string' ? msg.content : '', reasoning: typeof msg.reasoning_content === 'string' ? msg.reasoning_content : '' };
}
async function callWithRetry(key, parts, tag) {
  for (let a = 0; a < MAX_ATTEMPTS; a++) {
    const r = await callOnce(key, parts);
    if (!r.retry) return r;
    console.log(`[${tag}] attempt ${a + 1} retry (${r.reason})`);
    await sleep(RETRY_SLEEP);
  }
  return { retry: false, reason: 'giveup', content: '', reasoning: '' };
}
(async () => {
  const key = loadKey();
  for (const fp of [...SHOTS, LIHUI]) {
    if (!fs.existsSync(fp)) { console.error('MISSING:', fp); process.exit(1); }
  }
  const parts = [...SHOTS, LIHUI].map(dataUrl);
  const r = await callWithRetry(key, parts, 'qwen3.7-flash');
  console.log('=== qwen3.7-flash 最新渲染评分 ===');
  console.log(r.content || r.reasoning || '(no content)');
})();
