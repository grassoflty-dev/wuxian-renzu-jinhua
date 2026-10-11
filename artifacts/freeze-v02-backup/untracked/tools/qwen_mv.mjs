// qwen_mv.mjs — 体素建模「独立展示」相似度评分（脱离游戏场景）
// 判图：4 张干净建模图（全身正/侧/背 + 正面脸部特写）+ 1 张 2D 立绘（郑吒），一次性送检 qwen3.7-flash。
// 用法: node qwen_mv.mjs
// 输出: tools/mv_report.md ; 原始回复存档 tools/mv_raw_responses/
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SHOTS = ['mv_front.png', 'mv_side.png', 'mv_back.png', 'mv_face.png'];
const LIHUI = path.join(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');
const MD_OUT = path.join(__dirname, 'mv_report.md');
const RAWDIR = path.join(__dirname, 'mv_raw_responses');

const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 3000;
const RETRY_SLEEP = 15000;
const MAX_ATTEMPTS = 5;
const sleep = ms => new Promise(r => setTimeout(r, ms));

const JUDGE_PROMPT = `前四张是郑吒体素建模的干净展示图（第1张正面全身、第2张侧面全身、第3张背面全身、第4张正面脸部特写），第五张是郑吒的 2D 立绘。请按「建模与立绘的相似度」打分，中文简洁：
1. 相似度总分（0-100 分，100=完全一样）：给出一个明确数字。
2. 三视图各方向的表现（正面/侧面/背面分别哪里像、哪里不像）。
3. 发色、服装颜色/分段、五官、体型 各打几分并说明。
4. 还剩哪些细节差距（具体列出，供下一轮优化）。
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
    try {
      const r = await callOnce(key, parts);
      if (!r.retry) return r;
      console.error(`  [${tag}] attempt ${a + 1}: transient ${r.reason}, backoff 15s ...`);
    } catch (e) {
      if (a === MAX_ATTEMPTS - 1) throw e;
      console.error(`  [${tag}] attempt ${a + 1}: ${e.message}, backoff 15s ...`);
    }
    await sleep(RETRY_SLEEP);
  }
  throw new Error(`[${tag}] still failing`);
}

async function main() {
  const key = loadKey();
  if (!fs.existsSync(LIHUI)) { console.error('missing 立绘 ' + LIHUI); process.exit(2); }
  const shots = [];
  for (const s of SHOTS) {
    const fp = path.join(__dirname, 'shots', s);
    if (!fs.existsSync(fp)) { console.error(`missing ${fp}`); process.exit(2); }
    shots.push({ file: s, path: fp, size: fs.statSync(fp).size });
  }
  console.log('--- 送检 4 建模视图 + 1 立绘 ---');
  for (const s of shots) console.log(`  tools/shots/${s.file} (${s.size}B)`);
  console.log(`  立绘 ${LIHUI} (${fs.statSync(LIHUI).size}B)`);

  fs.mkdirSync(RAWDIR, { recursive: true });
  const t0 = Date.now();
  const parts = [...shots.map(s => dataUrl(s.path)), dataUrl(LIHUI)];
  const raw = await callWithRetry(key, parts, 'mv');

  const content = (raw.content || '').trim();
  const reasoning = (raw.reasoning || '').trim();
  const useReasoning = content.length < 60 && reasoning.length > content.length;
  const body = useReasoning ? reasoning : content;

  fs.writeFileSync(path.join(RAWDIR, 'mv_content.txt'), content || '', 'utf8');
  fs.writeFileSync(path.join(RAWDIR, 'mv_reasoning.txt'), reasoning || '', 'utf8');

  let score = '';
  let mm = String(body).match(/(?:相似度总分|最终(?:分数|得分)|相似度得分)[^0-9]{0,10}[：:]?\s*(\d{1,3})\s*分/);
  if (!mm) mm = String(body).match(/(?:总分|得分|评分).{0,6}[：:]?\s*(\d{1,3})\s*分/);
  if (mm) score = mm[1];

  const L = [];
  L.push('# 体素建模「独立展示」相似度评分报告');
  L.push('');
  L.push(`- 生成时间：${new Date().toISOString()}`);
  L.push(`- 判图模型：${MODEL}（tokenrhythm 视觉接口，max_tokens=${MAX_TOKENS}）`);
  L.push('- 送检：4 张干净建模图（全身正/侧/背 + 正面脸部特写）+ 1 张 2D 立绘');
  L.push('- 展示方式：脱离游戏场景，纯白背景 + 标准三点白光');
  for (const s of shots) L.push(`  - \`tools/shots/${s.file}\`（${s.size}B）`);
  L.push(`  - 立绘 \`server-rs/ui/assets/img/pc_zhengzha.png\``);
  L.push('');
  L.push('## qwen3.7-flash 完整打分结论（原文）');
  L.push('');
  L.push('```');
  L.push(body || '（模型未返回内容）');
  L.push('```');
  if (useReasoning) L.push('\n> ⚠️ content 过短，以上以 reasoning_content 兜底。');
  if (score) L.push(`\n> **解析到相似度总分：${score}/100**`);
  fs.writeFileSync(MD_OUT, L.join('\n'), 'utf8');

  console.log(`  len(content)=${content.length} len(reasoning)=${reasoning.length} useReasoning=${useReasoning}`);
  console.log('--- DONE ---');
  console.log(`elapsed=${((Date.now() - t0) / 1000).toFixed(1)}s wrote ${MD_OUT}`);
  console.log('--- 打分结论原文 ---');
  console.log(body);
}
main().catch(e => { console.error('FATAL', e.message); process.exit(1); });
