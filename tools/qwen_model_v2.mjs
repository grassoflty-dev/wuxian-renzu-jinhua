// qwen_model_v2.mjs — 3D 体素人物建模「三板斧」复评
// 三板斧：①头做大/肩收窄/躯干瘦/腿加长 ②方块接缝加 AO 暗缝 ③抗锯齿 MSAA8x。
// 判图：逐张送 3 张新截图（model_v2_1~3.png）给 qwen3.7-flash（tokenrhythm），按指定 6 行结构复评
//       三尺是否改善了"480P感/方块堆砌感"，并给出整体改善百分比。
// 用法: node qwen_model_v2.mjs
// 输出: tools/model_v2_report.md（每张 6 行 + 整体百分比）
// 凭据: ~/.dsh/.credentials.yaml 的 TOKENRHYTHM_API_KEY ; 端点 https://tokenrhythm.studio/v1/chat/completions
// 注意: 429 退避 15s×5；回复兼容 content / reasoning_content；max_tokens=4000
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SHOTS = ['model_v2_1.png', 'model_v2_2.png', 'model_v2_3.png'];
const MD_OUT = path.join(__dirname, 'model_v2_report.md');
const RAWDIR = path.join(__dirname, 'model_v2_raw_responses');

const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 4000;
const RETRY_SLEEP = 15000;
const MAX_ATTEMPTS = 5;

const sleep = ms => new Promise(r => setTimeout(r, ms));

// 判图指令（复评，聚焦三板斧是否见效；6 行结构）
const JUDGE_PROMPT = `这是游戏 3D 体素人物（刚做了三板斧优化：①头做大肩收窄躯干瘦腿加长的比例调整 ②方块接缝加 AO 暗缝 ③抗锯齿 MSAA8x）。请复评，中文简洁按 6 行：
比例: （头肩躯干腿比例是否协调、像人了？还是仍失调）
堆砌感: （方块接缝的 AO 暗缝是否让堆砌感下降、体积感是否出来了？）
锯齿: （抗锯齿是否让边缘平滑、480P感是否下降？）
观感: （整体相比之前'像480P视频'改善了多少，用百分比）
剩余缺陷: （还有哪些明显问题）
结论: （PASS/WARN/FAIL + 一句话最该继续改的地方）`;

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

function escMD(s) { return (s || '').replace(/\|/g, '\\|').replace(/\r?\n/g, '  \n'); }
function fmt6(content) {
  // 若模型返回了带「比例:/堆砌感:...」的行，尽量原样保留；否则整体贴出。
  const s = (content || '').trim();
  if (!s) return '（模型未返回可解析内容）';
  return '```\n' + s + '\n```';
}

async function main() {
  const key = loadKey();
  const shots = [];
  for (const s of SHOTS) {
    const fp = path.join(__dirname, 'shots', s);
    if (!fs.existsSync(fp)) { console.error(`missing ${fp}`); process.exit(2); }
    const size = fs.statSync(fp).size;
    shots.push({ file: s, path: fp, size });
  }
  console.log('--- 送检 ' + shots.length + ' 张 v2 截图 ---');
  for (const s of shots) console.log(`  tools/shots/${s.file} (${s.size}B)`);

  fs.mkdirSync(RAWDIR, { recursive: true });
  const t0 = Date.now();

  // 真正"每个字符串"索引: 逐张（每张单独调用，避免多图互相稀释/顺序偏差）
  const results = [];
  for (let i = 0; i < shots.length; i++) {
    const s = shots[i];
    const tag = 'shot' + (i + 1);
    console.log(`... 判图 ${s.file} ...`);
    let raw;
    try { raw = await callWithRetry(key, [dataUrl(s.path)], tag); }
    catch (e) { console.error('FATAL ' + tag + ': ' + e.message); results.push({ file: s.file, error: e.message }); continue; }
    const content = (raw.content || '').trim();
    const reasoning = (raw.reasoning || '').trim();
    // content 兼容解析：若 content 过短疑似被截断而 reasoning 完整，用 reasoning 兜底
    const useReasoning = content.length < 60 && reasoning.length > content.length;
    const body = useReasoning ? reasoning : content;
    results.push({ file: s.file, content, reasoning, body, size: s.size, useReasoning });
    fs.writeFileSync(path.join(RAWDIR, `${tag}_content.txt`), content || '', 'utf8');
    fs.writeFileSync(path.join(RAWDIR, `${tag}_reasoning.txt`), reasoning || '', 'utf8');
    fs.writeFileSync(path.join(RAWDIR, `${tag}_authoritative.txt`), body || '', 'utf8');
    console.log(`  len(content)=${content.length} len(reasoning)=${reasoning.length} useReasoning=${useReasoning}`);
  }

  const L = [];
  L.push('# 3D 体素人物建模「三板斧」复评报告');
  L.push('');
  L.push(`- 生成时间：${new Date().toISOString()}`);
  L.push(`- 判图模型：${MODEL}（tokenrhythm OpenAI 兼容视觉接口，max_tokens=${MAX_TOKENS}）`);
  L.push('- 复评对象：头做大/肩收窄/躯干瘦/腿加长 + 方块接缝 AO 暗缝 + 抗锯齿 MSAA8x');
  L.push(`- 送检图片：${shots.length} 张`);
  for (const s of shots) L.push(`  - \`tools/shots/${s.file}\`（${s.size}B）`);
  L.push(`- 原始回复存档：tools/model_v2_raw_responses/`);
  L.push('');
  L.push('## 逐张复评结论（qwen3.7-flash）');
  L.push('');
  const totalPct = [];
  for (const r of results) {
    if (r.error) { L.push(`### ${r.file} — ❌ 判图失败`); L.push(''); L.push(r.error); L.push(''); continue; }
    L.push(`### ${r.file}（${r.size}B）`);
    L.push('');
    L.push(fmt6(r.body));
    L.push('');
    // 尝试统计观感百分比
    const m = String(r.body).match(/(\d{1,3})(?:\s*%|\.?\s*百分比|%\s*改善)/);
    if (m) totalPct.push(parseInt(m[1], 10));
    if (r.useReasoning) L.push(`> ⚠️ content 过短(len=${r.content.length})，以 reasoning_content 兜底为判图结论。`);
    L.push('');
  }
  L.push('## 整体改善评估');
  L.push('');
  if (totalPct.length) {
    const avg = Math.round(totalPct.reduce((a, b) => a + b, 0) / totalPct.length);
    L.push(`- qwen 对 ${totalPct.length} 张给出观感改善百分比，平均约 **${avg}%**；`);
    L.push(`- 逐张百分比：${totalPct.join('、')}。`);
    L.push('');
    L.push(`> **整体结论**：三板斧让"像480P视频/方块堆砌感"平均改善约 **${avg}%**。`);
  } else {
    L.push('- 未从判图文本中解析到明确的观感百分比（复评原文见上方逐张段落）。');
  }
  fs.writeFileSync(MD_OUT, L.join('\n'), 'utf8');

  const secs = ((Date.now() - t0) / 1000).toFixed(1);
  console.log('--- DONE ---');
  console.log(`elapsed=${secs}s wrote ${MD_OUT}`);
}

main().catch(e => { console.error('FATAL', e.message); process.exit(1); });
