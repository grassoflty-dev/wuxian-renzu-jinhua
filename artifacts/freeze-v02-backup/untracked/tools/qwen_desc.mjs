// qwen_desc.mjs — 详细描述《无限轮回·灾变篇 第一章》4 张 3D 地图截图
// 调用 tokenrhythm qwen3.7-flash 视觉模型（模型名不带 tokenrhythm/ 前缀）
// 用法: node qwen_desc.mjs
// 输入: tools/shots/desc_1.png ~ desc_4.png
// 输出: tools/map_desc_report.md （每张截图的完整详细描述原文）
// 凭据: ~/.dsh/.credentials.yaml 的 TOKENRHYTHM_API_KEY
// 端点: https://tokenrhythm.studio/v1/chat/completions (OpenAI 兼容)
// 注意: 429/5xx 退避 15s×5；回复兼容 content / reasoning_content；data URL base64 传图；max_tokens=4000
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SHOTS_DIR = path.join(__dirname, 'shots');
const MD_OUT = path.join(__dirname, 'map_desc_report.md');

const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 4000;

const sleep = ms => new Promise(r => setTimeout(r, ms));

// 判图指令：详细描述，不要简短结论
const DESCRIBE_PROMPT = `请详细描述这张游戏 3D 地图截图，尽量具体、客观，逐项说：
1. 整体构图与视角（俯视/平视/近距离，画面主体是什么）
2. 人物（体素方块人）的造型、配色、在画面中的比例和清晰度
3. 地面的材质纹理、颜色、有没有变化
4. 墙壁的材质纹理、颜色、有没有变化
5. 环境里有哪些物体/装饰（管道/箱子/灯光/血迹等，具体列出来）
6. 光照和整体氛围（色调、明暗、有没有立体感）
7. 你作为观者，觉得画面哪里显得粗糙、简陋、单调或违和（如实指出，不要客套）`;

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
        { type: 'text', text: DESCRIBE_PROMPT },
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
  const text = await res.text();
  if (res.status === 429 || res.status >= 500 || /LITELLM_UNAVAILABLE|SERVICE_BUSY|暂时不可用|服务繁忙|overloaded|GATEWAY-TIME|timeout|timed out/i.test(text)) {
    return { retry: true, reason: `${res.status}`, error: '', content: '', reasoning: '' };
  }
  if (!res.ok) {
    return { retry: false, reason: `${res.status}`, error: `API error ${res.status}: ${text.slice(0, 500)}`, content: '', reasoning: '' };
  }
  let data;
  try { data = JSON.parse(text); } catch { return { retry: false, reason: 'json', error: 'bad json: ' + text.slice(0, 300), content: '', reasoning: '' }; }
  const msg = data.choices?.[0]?.message || {};
  const content = typeof msg.content === 'string' ? msg.content : '';
  const reasoning = typeof msg.reasoning_content === 'string' ? msg.reasoning_content : '';
  return { retry: false, reason: '', error: '', content, reasoning };
}

// 带退避的重试：瞬时故障退避 15s×5
async function callWithRetry(key, b64, fname) {
  for (let attempt = 0; attempt < 5; attempt++) {
    const r = await callOnce(key, b64);
    if (!r.retry) {
      if (r.error) throw new Error(r.error);
      return r;
    }
    console.error(`  [${fname}] attempt ${attempt + 1}: transient fault ${r.reason}, backing off 15s ...`);
    await sleep(15000);
  }
  throw new Error(`[${fname}] still failing after 5 attempts`);
}

function buildMarkdown(reports) {
  const L = [];
  L.push('# 《无限轮回·灾变篇 第一章》3D 地图截图详细描述');
  L.push('');
  L.push(`- 生成时间：${new Date().toISOString()}`);
  L.push(`- 判图模型：${MODEL}（tokenrhythm，OpenAI 兼容视觉接口，max_tokens=${MAX_TOKENS}）`);
  L.push(`- 覆盖截图：${reports.length} 张（tools/shots/desc_1.png ~ desc_4.png）`);
  L.push('');
  L.push('> 判图指令要求模型客观、具体地逐项描述画面，并如实指出粗糙、简陋、单调或违和之处。以下为每张截图的完整详细描述原文。');
  L.push('');
  for (const r of reports) {
    L.push(`## ${r.file}`);
    L.push('');
    if (r.error) {
      L.push(`> ❌ 调用失败：${r.error}`);
      L.push('');
      continue;
    }
    const text = (r.description || '').trim();
    L.push('```');
    L.push(text);
    L.push('```');
    L.push('');
  }
  return L.join('\n');
}

async function main() {
  if (!fs.existsSync(SHOTS_DIR)) { console.error(`shots dir not found: ${SHOTS_DIR}`); process.exit(2); }
  const files = ['desc_1.png', 'desc_2.png', 'desc_3.png', 'desc_4.png'];
  const key = loadKey();
  const reports = [];
  const t0 = Date.now();

  for (const f of files) {
    const fp = path.join(SHOTS_DIR, f);
    if (!fs.existsSync(fp)) {
      console.error(`missing ${fp}`);
      reports.push({ file: f, description: '', error: '截图文件缺失' });
      continue;
    }
    const b64 = fs.readFileSync(fp).toString('base64');
    let resp;
    try {
      resp = await callWithRetry(key, b64, f);
    } catch (e) {
      console.error(`FATAL ${f}: ${e.message}`);
      reports.push({ file: f, description: '', error: '调用失败: ' + e.message });
      continue;
    }
    // 优先 content，其次 reasoning_content
    const description = (resp.content || resp.reasoning || '').trim();
    reports.push({ file: f, description, error: '' });
    console.log(`[${f}] got description (${description.length} chars)`);
    // 每张间 sleep 300~600ms 避免限流
    await sleep(300 + Math.floor(Math.random() * 300));
  }

  fs.writeFileSync(MD_OUT, buildMarkdown(reports), 'utf8');

  const secs = ((Date.now() - t0) / 1000).toFixed(1);
  console.log('--- DONE ---');
  console.log(`wrote ${MD_OUT} in ${secs}s`);
  for (const r of reports) {
    console.log(`  ${r.file}: ${r.error ? 'ERROR ' + r.error : (r.description.length + ' chars')}`);
  }
}

main().catch(e => { console.error('FATAL', e.message); process.exit(1); });
