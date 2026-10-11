// face_compare_score.mjs — 【立绘脸 billboard 原型】体素版 vs 立绘脸版 qwen3.7-flash 相似度对比
// 用法: node face_compare_score.mjs
// 送检：
//   * 「体素脸版」= render_zhenghai_ {front,side,back} + pc_zhengzha.png 立绘
//   * 「立绘脸版」= render_zhenghaibillboard_ {front,side,back} + pc_zhengzha.png 立绘
// 各跑一次 qwen3.7-flash，解析相似度总分，做对比结论。
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SHOTS = path.join(__dirname, 'shots_qc');           // 送检用降采样图（防上游超时）
const LIHUI = path.join(SHOTS, 'pc_zhengzha.png');
const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 3000;
const RETRY_SLEEP = 20000;
const MAX_ATTEMPTS = 8;
const sleep = ms => new Promise(r => setTimeout(r, ms));

const VIEWS = ['front', 'side', 'back'];

function makePrompt(label) {
  return `前3张是角色郑吒的「${label}」体素建模三视图（第1张正面、第2张侧面、第3张背面），第4张是该角色的 2D 立绘（半身像：黑色短发、浓眉、深灰T恤、肌肉男）。
请按「体素建模与 2D 立绘的相似度」打分，中文简洁：
1. 相似度总分（0-100 分，100=完全一样）：给出一个明确数字。
2. 三视图各方向表现（正面/侧面/背面分别哪里像、哪里不像）。
3. 分维度打分：发/发型、服装/颜色、体型/身材、五官/脸型，各给一个 0-100 数字并一句说明。
4. 跟上一版比，脸的改变（如果是立绘脸版：2D 立绘头脸贴到体素身上，脸是不是明显更像立绘、更不像方块脸）。
5. 还剩哪些细节差距（具体列出）。
6. 是否值得继续（仅立绘脸版回答）：口头给出“值得/不值得继续做立绘脸”.`;
}

function loadKey() {
  const p = path.join(os.homedir(), '.dsh', '.credentials.yaml');
  const raw = fs.readFileSync(p, 'utf8');
  const m = raw.match(/TOKENRHYTHM_API_KEY:\s*(\S+)/);
  if (!m) throw new Error('TOKENRHYTHM_API_KEY not found');
  return m[1];
}
function dataUrl(fp, label) {
  return { type: 'image_url', image_url: { url: `data:image/png;base64,${fs.readFileSync(fp).toString('base64')}` } };
}
async function callOnce(key, imageParts, prompt) {
  const body = { model: MODEL, messages: [{ role: 'user', content: [{ type: 'text', text: prompt }, ...imageParts] }], max_tokens: MAX_TOKENS };
  const res = await fetch(ENDPOINT, {
    method: 'POST',
    headers: { 'Authorization': `Bearer ${key}`, 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  const text = await res.text();
  if (res.status === 429 || res.status >= 500 || /LITELLM_UNAVAILABLE|SERVICE_BUSY|overloaded|GATEWAY-TIME|timeout|timed out|MODEL_NOT_AVAILABLE/i.test(text)) {
    return { retry: true, content: '', reasoning: '' };
  }
  if (!res.ok) throw new Error(`API ${res.status}: ${text.slice(0, 400)}`);
  const d = JSON.parse(text);
  const msg = d.choices?.[0]?.message || {};
  return { retry: false, content: typeof msg.content === 'string' ? msg.content : '', reasoning: typeof msg.reasoning_content === 'string' ? msg.reasoning_content : '' };
}
async function callWithRetry(key, parts, tag, prompt) {
  for (let a = 0; a < MAX_ATTEMPTS; a++) {
    try {
      const r = await callOnce(key, parts, prompt);
      if (!r.retry) return r;
      console.error(`  [${tag}] attempt ${a + 1}: transient, backoff 15s`);
    } catch (e) {
      if (a === MAX_ATTEMPTS - 1) throw e;
      console.error(`  [${tag}] attempt ${a + 1}: ${e.message}, backoff 15s`);
    }
    await sleep(RETRY_SLEEP);
  }
  throw new Error(`[${tag}] still failing`);
}

function parseScore(body) {
  let m = String(body).match(/(?:相似度总分|最终(?:分数|得分)|相似度得分|总分)[^0-9]{0,10}[：:]?\s*(\d{1,3})\s*分/);
  if (!m) m = String(body).match(/(?:得分|评分).{0,6}[：:]?\s*(\d{1,3})\s*分/);
  return m ? Number(m[1]) : null;
}
function parseDimScore(body, key) {
  // key: 发/发型|服装|体型|五官/脸型
  const re = new RegExp(key + '[^0-9]{0,12}[:：]?\\s*(\\d{1,3})\\s*分');
  const m = String(body).match(re);
  return m ? Number(m[1]) : null;
}

async function scoreVariant(key, label, prefix) {
  const parts = [];
  for (const v of VIEWS) {
    const fp = path.join(SHOTS, `${prefix}${v}.png`);
    if (!fs.existsSync(fp)) throw new Error('missing ' + fp);
    parts.push(dataUrl(fp, label));
  }
  parts.push(dataUrl(LIHUI, '立绘'));
  const prompt = makePrompt(label);
  console.log(`\n===== 送检「${label}」${prefix}{front,side,back}.png + 立绘 =====`);
  const raw = await callWithRetry(key, parts, label, prompt);
  const body = ((raw.content || '') + '\n' + (raw.reasoning || '')).trim();
  return { label, prefix, body };
}

async function main() {
  const key = loadKey();
  const voxel = await scoreVariant(key, '体素脸版', 'render_zhenghai_');
  const bill = await scoreVariant(key, '立绘脸版', 'render_zhenghaibillboard_');

  const out = [`# 郑吒「立绘脸 billboard」vs「体素脸」qwen3.7-flash 相似度对比`, ''];
  const rows = [voxel, bill];
  const parsed = {};
  for (const r of rows) {
    parsed[r.label] = {
      score: parseScore(r.body),
      dims: { hair: parseDimScore(r.body, '发'), cloth: parseDimScore(r.body, '服装'),
              body: parseDimScore(r.body, '体型'), face: parseDimScore(r.body, '五官') },
    };
    out.push(`## ${r.label}（${r.prefix}{front,side,back}.png + 立绘）`);
    out.push(`- 相似度总分：${parsed[r.label].score ?? 'N/A'}/100`);
    out.push(`- 分维度：发=${parsed[r.label].dims.hair ?? 'N/A'} 服装=${parsed[r.label].dims.cloth ?? 'N/A'} 体型=${parsed[r.label].dims.body ?? 'N/A'} 五官/脸型=${parsed[r.label].dims.face ?? 'N/A'}`);
    out.push('', '### 原文', '', '```', r.body, '```', '');
  }
  // 对比结论
  const a = parsed['体素脸版'], b = parsed['立绘脸版'];
  out.push('---');
  out.push('## 对比结论');
  out.push(`- 体素脸版总分 ${a.score}/100 vs 立绘脸版总分 ${b.score}/100`);
  if (a.face != null && b.face != null) out.push(`- 五官/脸型：体素脸版 ${a.face} vs 立绘脸版 ${b.face}（差 ${b.face - a.face}）`);
  if (a.score != null && b.score != null) out.push(`- 总分差：${b.score - a.score}（正=立绘脸更高）`);

  fs.mkdirSync(path.join(__dirname, 'audit_reports'), { recursive: true });
  fs.mkdirSync(path.join(__dirname, 'audit_raw_responses'), { recursive: true });
  const md = path.join(__dirname, 'audit_reports', 'zhenghai_facebillboard_compare.md');
  fs.writeFileSync(md, out.join('\n'), 'utf8');
  console.log('\n\nWROTE ' + md);
  // 打印精简结果
  for (const r of rows) {
    console.log(`\n[${r.label}] score=${parsed[r.label].score} dims=${JSON.stringify(parsed[r.label].dims)}`);
    console.log(r.body);
  }
}
main().catch(e => { console.error('FATAL', e.message); process.exit(1); });
