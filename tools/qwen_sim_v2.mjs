// qwen_sim_v2.mjs — 「立绘与建模 相似度」复评打分（v2，三视图）
// 判图：3 张三视图（正/侧/背）体素建模截图（tools/shots/sim_v2_front.png / sim_v2_side.png / sim_v2_back.png）
//       + 1 张 2D 立绘（郑吒，pc_zhengzha.png），一次性全部送检给 qwen3.7-flash（tokenrhythm）。
// 用法: node qwen_sim_v2.mjs
// 输出: tools/sim_v2_report.md（qwen 完整打分结论原文）
// 原始回复存档: tools/sim_v2_raw_responses/
// 凭据: ~/.dsh/.credentials.yaml 的 TOKENRHYTHM_API_KEY ; 端点 https://tokenrhythm.studio/v1/chat/completions
// 注意: 429 退避 15s×5；回复兼容 content / reasoning_content；max_tokens=3000
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SHOTS = ['sim_v2_front.png', 'sim_v2_side.png', 'sim_v2_back.png'];
const LIHUI = path.join(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');
const MD_OUT = path.join(__dirname, 'sim_v2_report.md');
const RAWDIR = path.join(__dirname, 'sim_v2_raw_responses');

const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 3000;
const RETRY_SLEEP = 15000;
const MAX_ATTEMPTS = 5;

const sleep = ms => new Promise(r => setTimeout(r, ms));

// 判图指令（三视图相似度打分，量化；一条调用含 3 建模三视图 + 1 立绘，共 4 张）
const JUDGE_PROMPT = `前三张是郑吒体素建模的三视图（正面/侧面/背面），第四张是郑吒立绘。请按「建模与立绘的相似度」打分，中文简洁：
1. 相似度总分（0-100 分，100=完全一样）：给出一个明确数字。
2. 三视图各方向的表现（正面/侧面/背面分别哪里像、哪里不像）。
3. 相比上一轮（35分：发色反了/黑煤球/无五官/坦克体型），本轮发色、服装分段、五官、体型各改善了多少。
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
    if (res.status === 429 || res.status >= 500 || /LITELLM_UNAVAILABLE|SERVICE_BUSY|暂时不可用|服务繁忙|overloaded|GATEWAY-TIME|timeout|timed out|MODEL_NOT_AVAILABLE/i.test(text)) {
      return { retry: true, reason: `${res.status}`, content: '', reasoning: '' };
    }
    throw new Error(`API error ${res.status}: ${text.slice(0, 500)}`);
  }
  let data;
  try { data = JSON.parse(text); } catch { throw new Error('bad json: ' + text.slice(0, 300)); }
  const msg = data.choices?.[0]?.message || {};
  const content = typeof msg.content === 'string' ? msg.content : '';
  const reasoning = typeof msg.reasoning_content === 'string' ? msg.reasoning_content : '';
  return { retry: false, reason: '', content, reasoning };
}

async function callWithRetry(key, imageParts, tag) {
  for (let attempt = 0; attempt < MAX_ATTEMPTS; attempt++) {
    try {
      const r = await callOnce(key, imageParts);
      if (!r.retry) return r;
      console.error(`  [${tag}] attempt ${attempt + 1}: transient fault ${r.reason}, backing off 15s ...`);
    } catch (e) {
      if (attempt === MAX_ATTEMPTS - 1) throw e;
      console.error(`  [${tag}] attempt ${attempt + 1}: ${e.message}, backing off 15s ...`);
    }
    await sleep(RETRY_SLEEP);
  }
  throw new Error(`[${tag}] still failing after ${MAX_ATTEMPTS} attempts`);
}

async function main() {
  const key = loadKey();
  if (!fs.existsSync(LIHUI)) { console.error('missing 立绘 ' + LIHUI); process.exit(2); }

  const shots = [];
  for (const s of SHOTS) {
    const fp = path.join(__dirname, 'shots', s);
    if (!fs.existsSync(fp)) { console.error(`missing ${fp}`); process.exit(2); }
    const size = fs.statSync(fp).size;
    if (size <= 50 * 1024) console.warn(`⚠️  ${s} 只有 ${size}B（要求 >50KB）`);
    shots.push({ file: s, path: fp, size });
  }
  const lihuiSize = fs.statSync(LIHUI).size;
  console.log('--- 送检 3 三视图建模 + 1 立绘 ---');
  for (const s of shots) console.log(`  tools/shots/${s.file} (${s.size}B)`);
  console.log(`  立绘 ${LIHUI} (${lihuiSize}B)`);
  console.log(`  送检顺序：前3建模（正/侧/背） + 第4立绘`);

  fs.mkdirSync(RAWDIR, { recursive: true });
  const t0 = Date.now();

  const parts = [...shots.map(s => dataUrl(s.path)), dataUrl(LIHUI)];
  let raw;
  try { raw = await callWithRetry(key, parts, 'sim_v2'); }
  catch (e) { console.error('FATAL: ' + e.message); process.exit(1); }

  const content = (raw.content || '').trim();
  const reasoning = (raw.reasoning || '').trim();
  const useReasoning = content.length < 60 && reasoning.length > content.length;
  const body = useReasoning ? reasoning : content;

  fs.writeFileSync(path.join(RAWDIR, 'sim_v2_content.txt'), content || '', 'utf8');
  fs.writeFileSync(path.join(RAWDIR, 'sim_v2_reasoning.txt'), reasoning || '', 'utf8');
  fs.writeFileSync(path.join(RAWDIR, 'sim_v2_authoritative.txt'), body || '', 'utf8');
  console.log(`  len(content)=${content.length} len(reasoning)=${reasoning.length} useReasoning=${useReasoning}`);

  // 解析评分数字：优先「相似度总分/得分/最终分数」后的数字
  let score = '';
  let mm = String(body).match(/(?:相似度总分|最终(?:分数|得分)|相似度得分)[^0-9]{0,10}[：:]?\s*(\d{1,3})\s*分/);
  if (!mm) mm = String(body).match(/(?:总分|得分|评分).{0,6}[：:]?\s*(\d{1,3})\s*分/);
  if (mm) score = mm[1];

  const L = [];
  L.push('# 立绘与体素建模「相似度」三视图复评报告（v2）');
  L.push('');
  L.push(`- 生成时间：${new Date().toISOString()}`);
  L.push(`- 判图模型：${MODEL}（tokenrhythm OpenAI 兼容视觉接口，max_tokens=${MAX_TOKENS}）`);
  L.push('- 送检：3 张游戏内 3D 体素人物建模三视图（正/侧/背）+ 1 张 2D 立绘（郑吒，pc_zhengzha.png）');
  L.push('- 判定维度：建模与立绘的相似度（0-100）');
  for (const s of shots) L.push(`  - \`tools/shots/${s.file}\`（${s.size}B）`);
  L.push(`  - 立绘 \`server-rs/ui/assets/img/pc_zhengzha.png\`（${lihuiSize}B）`);
  L.push('- 上一轮基线：35 分（发色反了/黑煤球/无五官/坦克体型）');
  L.push(`- 原始回复存档：tools/sim_v2_raw_responses/`);
  L.push('');
  L.push('## qwen3.7-flash 完整打分结论（原文）');
  L.push('');
  L.push('```');
  L.push(body || '（模型未返回内容）');
  L.push('```');
  if (useReasoning) {
    L.push('');
    L.push(`> ⚠️ content 过短(len=${content.length})，以上以 reasoning_content 兜底为打分结论。`);
  }
  if (score) {
    L.push('');
    L.push(`> **解析到相似度总分：${score}/100**`);
  }
  fs.writeFileSync(MD_OUT, L.join('\n'), 'utf8');

  const secs = ((Date.now() - t0) / 1000).toFixed(1);
  console.log('--- DONE ---');
  console.log(`elapsed=${secs}s wrote ${MD_OUT}`);
  console.log('--- 打分结论原文 ---');
  console.log(body);
}

main().catch(e => { console.error('FATAL', e.message); process.exit(1); });
