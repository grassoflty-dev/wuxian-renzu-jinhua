// qwen_see.mjs — 让 qwen 看图并定位界面异常(满宽左右拖动条)
// 用法: node qwen_see.mjs <图片路径> [问题描述]
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const fp = process.argv[2] || path.join(__dirname, 'shots', 'nexus_first_t1_afterEnter.png');
const ISSUE = process.argv[3] || '';

const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 3000;
const RETRY_SLEEP = 15000;
const MAX_ATTEMPTS = 5;
const sleep = ms => new Promise(r => setTimeout(r, ms));

const JUDGE_PROMPT = `这是一张游戏界面的截图。${ISSUE ? '用户反馈的问题:【' + ISSUE + '】。请判断截图里是否能看到这个问题的具体表现,并描述它的精确位置(上下左右相对百分比)、形状、颜色、尺寸。' : '请详细描述这个界面的内容:有哪些 UI 元素、布局、文字、按钮,特别是中上部的元素。'}
请具体描述:
1. 整体是什么界面
2. **中上部(垂直 15%-40% 区域)**:有什么元素?有没有一个横向的条/带/滚动条/横幅?它的精确位置(y 百分比范围)、左右范围、颜色、内容?
3. 这个条能否拖动(猜测依据:样式/位置/内容)?
4. 其他明显异常(错位/拉伸/空白/闪烁元素)
中文输出,简洁精确,看图说话,不要臆造。`;

function loadKey() {
  const p = path.join(os.homedir(), '.dsh', '.credentials.yaml');
  const raw = fs.readFileSync(p, 'utf8');
  const m = raw.match(/TOKENRHYTHM_API_KEY:\s*(\S+)/);
  if (!m) throw new Error('TOKENRHYTHM_API_KEY not found');
  return m[1];
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
  if (!fs.existsSync(fp)) { console.error('MISSING', fp); process.exit(1); }
  const key = loadKey();
  const b64 = fs.readFileSync(fp).toString('base64');
  const r = await callWithRetry(key, [{ type: 'image_url', image_url: { url: `data:image/png;base64,${b64}` } }]);
  console.log('=== qwen 界面检查: ' + path.basename(fp) + ' ===');
  console.log(r.content || r.reasoning || '(no content)');
})();
