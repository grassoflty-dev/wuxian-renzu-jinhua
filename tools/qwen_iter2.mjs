// qwen_iter2.mjs — 生化世界 3D 地图截图（迭代2）判图评估
// 调用 tokenrhythm qwen3.7-flash 视觉模型，遍历 tools/shots/iter2_1.png ~ iter2_3.png
// 判图指令聚焦本轮改进点（黑屏占比/天空背景/光照/环境/人物/结论），中文简洁按 6 行。
// 用法: node qwen_iter2.mjs
// 输出: tools/iter2_report.md
// 凭据: ~/.dsh/.credentials.yaml 的 TOKENRHYTHM_API_KEY
// 端点: https://tokenrhythm.studio/v1/chat/completions (OpenAI 兼容)
// 注意: 429 退避 15s×5；回复兼容 content / reasoning_content；max_tokens=2000
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SHOTS = ['iter2_1.png', 'iter2_2.png', 'iter2_3.png'];
const MD_OUT = path.join(__dirname, 'iter2_report.md');
const RAWDIR = path.join(__dirname, 'shots_raw_responses_iter2');

const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 2000;
const RETRY_SLEEP = 15000;
const MAX_ATTEMPTS = 5;

const sleep = ms => new Promise(r => setTimeout(r, ms));

// 判图指令（聚焦本轮改进点）
const JUDGE_PROMPT = `这是游戏3D地图截图。评估，中文简洁按6行：
黑屏占比: （画面纯黑虚空的百分比，相比之前45-60%是否大幅下降）
天空背景: （地图外是否有渐变天空/穹顶，还是纯黑）
光照: （是否还死黑，有无立体感）
环境: （地面墙壁材质装饰丰富度，蜂巢氛围）
人物: （体素人是否完整、是否还悬浮）
结论: （PASS/WARN/FAIL + 一句话最该改的地方）`;

function loadKey() {
  const p = path.join(os.homedir(), '.dsh', '.credentials.yaml');
  const raw = fs.readFileSync(p, 'utf8');
  const m = raw.match(/TOKENRHYTHM_API_KEY:\s*(\S+)/);
  if (!m) throw new Error('TOKENRHYTHM_API_KEY not found');
  return m[1];
}

// 单次 API 调用
async function callOnce(key, b64) {
  const body = {
    model: MODEL,
    messages: [{
      role: 'user',
      content: [
        { type: 'text', text: JUDGE_PROMPT },
        { type: 'image_url', image_url: { url: `data:image/png;base64,${b64}` } },
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
    if (res.status === 429 || res.status >= 500 || /LITELLM_UNAVAILABLE|SERVICE_BUSY|暂时不可用|服务繁忙|overloaded|GATEWAY-TIME|timeout|timed out/i.test(text)) {
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

// 带退避重试：429/5xx 瞬时故障退避 RETRY_SLEEP，最多 MAX_ATTEMPTS 次
async function callWithRetry(key, b64, fname) {
  for (let attempt = 0; attempt < MAX_ATTEMPTS; attempt++) {
    try {
      const r = await callOnce(key, b64);
      if (!r.retry) return r;
      console.error(`  [${fname}] attempt ${attempt + 1}: transient fault ${r.reason}, backing off 15s ...`);
    } catch (e) {
      // 网络/JSON 级错误也按瞬时处理重试，最后一次抛出
      if (attempt === MAX_ATTEMPTS - 1) throw e;
      console.error(`  [${fname}] attempt ${attempt + 1}: ${e.message}, backing off 15s ...`);
    }
    await sleep(RETRY_SLEEP);
  }
  throw new Error(`[${fname}] still failing after ${MAX_ATTEMPTS} attempts`);
}

// 解析模型回复为 6 行（黑屏占比/天空背景/光照/环境/人物/结论)；结论含 PASS/WARN/FAIL 判定
function parseReply(text) {
  const src = (text || '');
  const clean = v => v.trim().replace(/^\d+[\.、]\s*/, '').trim();
  const pick = label => {
    const m = src.match(new RegExp(`^\\s*${label}\\s*[:：]\\s*(.*)$`, 'm'));
    return m ? clean(m[1]) : '';
  };
  return {
    black: pick('黑屏占比'),
    sky: pick('天空背景'),
    light: pick('光照'),
    env: pick('环境'),
    char: pick('人物'),
    verdict: pick('结论'),
  };
}

function normalizeVerdict(v) {
  const s = (v || '').toUpperCase();
  if (s.includes('FAIL')) return 'FAIL';
  if (s.includes('WARN')) return 'WARN';
  if (/严重|死黑|缺失|破损|黑屏|不行|特别差|悬浮|悬空/i.test(s)) return 'FAIL';
  if (/小问题|一般|略|轻微|勉强|还行|尚未/i.test(s)) return 'WARN';
  return 'PASS';
}

function escMD(s) { return (s || '').replace(/\|/g, '\\|').replace(/\r?\n/g, ' / '); }

async function main() {
  const key = loadKey();
  const results = [];
  const t0 = Date.now();

  for (const f of SHOTS) {
    const fp = path.join(__dirname, 'shots', f);
    if (!fs.existsSync(fp)) { console.error(`missing ${fp}`); results.push({ file: f, ok: false, err: 'missing file' }); continue; }
    const b64 = fs.readFileSync(fp).toString('base64');
    let resp;
    try {
      resp = await callWithRetry(key, b64, f);
    } catch (e) {
      console.error(`FATAL ${f}: ${e.message}`);
      results.push({ file: f, ok: false, err: e.message });
      continue;
    }
    const reply = resp.content || resp.reasoning || '';
    fs.mkdirSync(RAWDIR, { recursive: true });
    fs.writeFileSync(path.join(RAWDIR, f.replace(/\.png$/, '') + '.txt'), reply, 'utf8');
    fs.writeFileSync(path.join(RAWDIR, f.replace(/\.png$/, '') + '.reasoning.txt'), resp.reasoning || '', 'utf8');

    const parsed = parseReply(reply);
    const verdict = normalizeVerdict(parsed.verdict);
    results.push({ file: f, ok: true, ...parsed, verdict });
    console.log(`[${f}] -> ${verdict}`);
    await sleep(300 + Math.floor(Math.random() * 300));
  }

  // 写 MD 报告（含每张 6 行结论原文）
  const L = [];
  L.push('# 生化世界 3D 地图截图 · 迭代2 判图报告');
  L.push('');
  L.push(`- 生成时间：${new Date().toISOString()}`);
  L.push(`- 判图模型：${MODEL}（tokenrhythm，OpenAI 兼容视觉接口）`);
  L.push(`- 覆盖截图：${SHOTS.length} 张（iter2_1 ~ iter2_3，3 视角）`);
  L.push(`- 原始回复存档：tools/shots_raw_responses_iter2/`);
  L.push('');
  L.push('## 逐张 6 行结论（原文）');
  L.push('');
  for (const r of results) {
    L.push(`### ${r.file}`);
    L.push('');
    if (!r.ok) { L.push(`**调用失败**：${escMD(r.err)}`); L.push(''); continue; }
    L.push(`- 黑屏占比: ${escMD(r.black)}`);
    L.push(`- 天空背景: ${escMD(r.sky)}`);
    L.push(`- 光照: ${escMD(r.light)}`);
    L.push(`- 环境: ${escMD(r.env)}`);
    L.push(`- 人物: ${escMD(r.char)}`);
    L.push(`- 结论: ${escMD(r.verdict)}`);
    L.push('');
  }

  L.push('## 汇总');
  L.push('');
  L.push('| 文件 | 判定 |');
  L.push('| --- | --- |');
  for (const r of results) {
    L.push(`| \`${r.file}\` | ${r.ok ? '**' + escMD(r.verdict) + '**' : '错误'} |`);
  }
  L.push('');
  L.push('> 说明：结论为归一化判定（PASS/WARN/FAIL）；各字段为模型判图的中文原文（含"结论"整句原文）。');
  fs.writeFileSync(MD_OUT, L.join('\n'), 'utf8');

  const secs = ((Date.now() - t0) / 1000).toFixed(1);
  console.log('--- DONE ---');
  console.log(`elapsed=${secs}s wrote ${MD_OUT}`);
}

main().catch(e => { console.error('FATAL', e.message); process.exit(1); });
