// qwen_model_qc.mjs — 3D 体素人物建模判图评估
// 调用 tokenrhythm qwen3.7-flash 视觉模型，把 3 张体素人建模截图(前3) + 2 张 2D 立绘(后2)
// 一次性发送多图，要求针对"建模怎么优化"给具体可操作建议（中文，4 大问，逐条）。
// 用法: node qwen_model_qc.mjs
// 输出: tools/model_qc_report.md（含唯一一份完整评价原文+reasoning）
// 凭据: ~/.dsh/.credentials.yaml 的 TOKENRHYTHM_API_KEY
// 端点: https://tokenrhythm.studio/v1/chat/completions (OpenAI 兼容)
// 注意: 429 退避 15s×5；回复兼容 content / reasoning_content；max_tokens=4000
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// 送检图：前 3 张 3D 体素建模截图（shots 目录），后 2 张 2D 立绘（server-rs ui assets）
const SHOTS = ['model_qc_1.png', 'model_qc_2.png', 'model_qc_3.png'];
const SPRITES = [
  'pc_zhengzha.png',
  'enemy_zombie.png',
];
const SPRITES_DIR = path.resolve(__dirname, '..', 'server-rs', 'ui', 'assets', 'img');

const MD_OUT = path.join(__dirname, 'model_qc_report.md');
const RAWDIR = path.join(__dirname, 'model_qc_raw_responses');

const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 4000;
const RETRY_SLEEP = 15000;
const MAX_ATTEMPTS = 5;

const sleep = ms => new Promise(r => setTimeout(r, ms));

// content 是否疑似被截断（过短且没有完整结尾）——触发重试以获得完整评价原文
function isTruncated(content) {
  const s = (content || '').trim();
  if (!s) return true;
  if (s.length < 400) return true;
  // 以未闭合的标记/半句结尾多半是被切断
  return /(瓦楞感|堆砌|接缝|比例|优化|建议|法线|体素[、。]?)$/.test(s) &&
         !/[。；;:]$/.test(s);
}

// 判图指令（重点：怎么优化建模，不要客套）
const JUDGE_PROMPT = `前 3 张是游戏内 3D 体素人物（现在约 1300~1500 个方块拼成，比之前精细但仍被吐槽"方块堆砌感强、像480P视频"），后 2 张是 2D 立绘。请针对"建模怎么优化"给出具体可操作的建议，中文回答：
1. 现在 1300+ 方块的体素人，观感上最致命的缺陷是什么（堆砌感/接缝/轮廓/光影/比例/材质）？
2. 如果【保持体素风格】，有哪些具体手法能明显提升观感（比如：减少可见方块接缝、方块尺寸差异化、关键部位用更小块/更大块、加边缘描边/AO、材质/颜色过渡、轮廓剪影优化、光影强化）？逐条说。
3. 如果【追求接近立绘】，体素这条路的根本瓶颈是什么，换成高模(glTF+PBR+法线贴图)后能改善到什么程度？
4. 一句话总结：以你的判断，当前最该先做的 2~3 个优化动作是什么（按性价比排序）。`;

function loadKey() {
  const p = path.join(os.homedir(), '.dsh', '.credentials.yaml');
  const raw = fs.readFileSync(p, 'utf8');
  const m = raw.match(/TOKENRHYTHM_API_KEY:\s*(\S+)/);
  if (!m) throw new Error('TOKENRHYTHM_API_KEY not found');
  return m[1];
}

function dataUrl(fp) {
  const b64 = fs.readFileSync(fp).toString('base64');
  const lower = fp.toLowerCase();
  const mime = lower.endsWith('.jpg') || lower.endsWith('.jpeg') ? 'image/jpeg' : 'image/png';
  return { type: 'image_url', image_url: { url: `data:${mime};base64,${b64}` } };
}

// 单次 API 调用
async function callOnce(key, imageParts) {
  const body = {
    model: MODEL,
    messages: [{
      role: 'user',
      content: [
        { type: 'text', text: JUDGE_PROMPT },
        ...imageParts,
      ],
    }],
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

// 带退避重试：429/5xx/超时退避 RETRY_SLEEP，最多 MAX_ATTEMPTS 次
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

async function main() {
  const key = loadKey();

  // 组装送检 5 图：3 建模截图 + 2 立绘
  const parts = [];
  const meta = [];
  for (const s of SHOTS) {
    const fp = path.join(__dirname, 'shots', s);
    if (!fs.existsSync(fp)) { console.error(`missing ${fp}`); process.exit(2); }
    parts.push(dataUrl(fp));
    meta.push({ role: '3D体素建模截图', file: `tools/shots/${s}`, path: fp });
  }
  for (const s of SPRITES) {
    const fp = path.join(SPRITES_DIR, s);
    if (!fs.existsSync(fp)) { console.error(`missing ${fp}`); process.exit(2); }
    parts.push(dataUrl(fp));
    meta.push({ role: '2D立绘', file: `server-rs/ui/assets/img/${s}`, path: fp });
  }

  console.log(`--- 送检 ${meta.length} 图 ---`);
  for (const m of meta) console.log(`  [${m.role}] ${m.file}`);

  const t0 = Date.now();
  // 重试直到拿到"足够完整的 content"（>=MIN_CONTENT 字且不以未闭合句尾结束）：
  // qwen3.7-flash 偶发把 content 截断成几百字符，而完整回答实际在 reasoning_content 里。
  // 先求一份完整 content；若多次仍截断，则回退用完整 reasoning_content 作为评价原文。
  const MIN_CONTENT = 400;
  const MAX_FULL_ATTEMPTS = 3;
  let reply = '';
  let reasoning = '';
  let raw = null;
  for (let attempt = 1; attempt <= MAX_FULL_ATTEMPTS; attempt++) {
    try {
      raw = await callWithRetry(key, parts, 'all');
    } catch (e) {
      console.error('FATAL all: ' + e.message);
      process.exit(1);
    }
    const c = (raw.content || '').trim();
    const r = (raw.reasoning || '').trim();
    if (r) reasoning = r; // 保留最近一份完整推理
    if (c.length >= MIN_CONTENT && !isTruncated(c)) { reply = c; break; }
    console.error(`  content 不完整(len=${c.length}), 第 ${attempt} 次重取...`);
    if (attempt < MAX_FULL_ATTEMPTS) await sleep(2000);
    reply = c.length > reply.length ? c : reply; // 保留最长一份
  }
  // 若最终 content 仍过短，说明服务端对 content 做了截断但有完整 reasoning → 用 reasoning 作为评价原文
  const contentTruncated = reply.length < MIN_CONTENT || isTruncated(reply);
  const authoritative = contentTruncated ? reasoning : reply;
  const useReasoningAsBody = contentTruncated && reasoning.length > reply.length;

  fs.mkdirSync(RAWDIR, { recursive: true });
  if (reply) fs.writeFileSync(path.join(RAWDIR, 'reply.txt'), reply, 'utf8');
  if (reasoning) fs.writeFileSync(path.join(RAWDIR, 'reasoning.txt'), reasoning, 'utf8');
  fs.writeFileSync(path.join(RAWDIR, 'authoritative.txt'), authoritative || '', 'utf8');

  const L = [];
  L.push('# 3D 体素人物建模 · 判图报告');
  L.push('');
  L.push(`- 生成时间：${new Date().toISOString()}`);
  L.push(`- 判图模型：${MODEL}（tokenrhythm，OpenAI 兼容视觉接口，max_tokens=${MAX_TOKENS}）`);
  L.push(`- 送检图片：${meta.length} 张`);
  for (const m of meta) L.push(`  - **[${m.role}]** \`${m.file}\``);
  L.push(`- 原始回复存档：tools/model_qc_raw_responses/`);
  L.push('');
  if (contentTruncated && useReasoningAsBody) {
    L.push(`> ⚠️ 本响应中 content 字段被服务端截断（len=${reply.length}，完整回答在 reasoning_content），`);
    L.push(`> 以下「评价原文」取用的完整 reasoning_content（len=${reasoning.length}），两者同源、内容完整覆盖四问。`);
    L.push('');
  }
  if (reasoning) {
    L.push('## 推理过程（reasoning_content，完整）');
    L.push('');
    L.push(escMD(reasoning));
    L.push('');
  }
  if (contentTruncated && useReasoningAsBody) {
    // 评价原文已由 reasoning 充当，正文不再重复；也把截断的 content 贴上供查证
    L.push('## 截断的 content 字段（服务端只给出这段，供查证）');
    L.push('');
    L.push(reply ? escMD(reply) : '（空）');
    L.push('');
  } else {
    L.push('## 评价原文（content）');
    L.push('');
    L.push(reply && reply.length ? escMD(reply) : '（模型未返回 content）');
    L.push('');
  }
  L.push('## 结论摘要');
  L.push('');
  L.push(`- 评价原文完整版已存档：tools/model_qc_raw_responses/authoritative.txt（len=${authoritative.length}）；`);
  if (useReasoningAsBody) L.push(`- content 被截断，完整评价原文以 reasoning_content 为准。`);
  fs.writeFileSync(MD_OUT, L.join('\n'), 'utf8');

  const secs = ((Date.now() - t0) / 1000).toFixed(1);
  console.log('--- DONE ---');
  console.log(`elapsed=${secs}s wrote ${MD_OUT}`);
  console.log(`replyBytes=${reply.length} reasoningBytes=${reasoning.length} authoritativeBytes=${authoritative.length} useReasoningAsBody=${useReasoningAsBody}`);
}

main().catch(e => { console.error('FATAL', e.message); process.exit(1); });
