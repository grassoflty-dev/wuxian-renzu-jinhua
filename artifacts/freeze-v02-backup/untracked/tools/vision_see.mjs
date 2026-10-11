// vision_see.mjs — 单图 glm 视觉描述（快速）
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const fp = process.argv[2];
const PROMPT = process.argv[3] || '描述这张图：内容、布局、有没有明显问题（空白/错位/黑屏/破损）。中文简洁。';
const sleep = ms => new Promise(r => setTimeout(r, ms));
function loadKey() { const p = path.join(os.homedir(), '.dsh', '.credentials.yaml'); const m = fs.readFileSync(p, 'utf8').match(/TOKENRHYTHM_API_KEY:\s*(\S+)/); return m[1]; }
async function call(key, model) {
  const body = { model, messages: [{ role: 'user', content: [{ type: 'text', text: PROMPT }, { type: 'image_url', image_url: { url: 'data:image/png;base64,' + fs.readFileSync(fp).toString('base64') } }] }], max_tokens: 1200 };
  const res = await fetch('https://tokenrhythm.studio/v1/chat/completions', { method: 'POST', headers: { 'Authorization': 'Bearer ' + key, 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  if (res.status === 429 || res.status >= 500) return { retry: true };
  const t = await res.text();
  if (!res.ok) { if (/LITELLM|SERVICE_BUSY|timeout|MODEL_NOT_AVAILABLE/i.test(t)) return { retry: true }; throw new Error(t.slice(0,300)); }
  const d = JSON.parse(t); const m = d.choices?.[0]?.message || {}; return { retry: false, content: m.content || m.reasoning_content || '' };
}
(async () => {
  const key = loadKey();
  for (const model of ['deepseek-flash', 'qwen3.8-flash', 'glm-5.3-flash']) for (let a = 0; a < 4; a++) {
    const r = await call(key, model);
    if (!r.retry) { console.log('MODEL', model); console.log(r.content || '(empty)'); process.exit(0); }
    await sleep(15000);
  }
  console.log('ALL_MODELS_FAILED');
  process.exit(1);
})();
