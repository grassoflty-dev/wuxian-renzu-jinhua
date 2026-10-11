import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// ---------- config ----------
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const CREDS = 'C:/Users/PUBLIC_USER/.dsh/.credentials.yaml';
const BASE = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const OUT_MD = path.join(__dirname, 'compare_report.md');

// ordered material: first two are 3D voxel, rest are 2D sprites
const IMAGES = [
  { path: path.join(__dirname, 'shots', 'fight_z1.png'), label: '3D 战斗体素人', group: '3D' },
  { path: path.join(__dirname, 'shots', 'iter2_2.png'),  label: '3D 地图体素人', group: '3D' },
  { path: path.join(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png'), label: '2D 玩家郑吒立绘', group: '2D' },
  { path: path.join(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'enemy_zombie.png'), label: '2D 丧尸立绘', group: '2D' },
  { path: path.join(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'enemy_hunter.png'), label: '2D 猎手立绘', group: '2D' },
];

const PROMPT = `请对比这些游戏素材的视觉质量。前 2 张是游戏内 3D 体素建模（方块拼成的人物）截图，后几张是 2D 角色立绘（AI 生成插画）。请详细对比，中文回答：
1. 3D 体素建模的视觉质量如何（精度、细节、五官、服饰、观感）
2. 2D 立绘的视觉质量如何（精度、细节、五官、服饰、观感）
3. 两者的差距有多大（用 1-10 分分别打分，说明差距体现在哪些方面）
4. 如果要把 3D 建模提升到接近立绘的观感，最缺的是什么（细节/材质/比例/光影？）`;

// ---------- helpers ----------
function getApiKey() {
  const raw = fs.readFileSync(CREDS, 'utf8');
  const re = /TOKENRHYTHM_API_KEY\s*:\s*"?([^"\s]+)"?/;
  const m = raw.match(re);
  if (!m) throw new Error('TOKENRHYTHM_API_KEY not found in ' + CREDS);
  return m[1].trim();
}

function toDataUrl(filePath) {
  const buf = fs.readFileSync(filePath);
  return `data:image/png;base64,${buf.toString('base64')}`;
}

// retry with 429 backoff 15s * 5
async function chat(dataUrl, apiKey, prompt, retriesLeft = 5) {
  const body = {
    model: MODEL,
    max_tokens: 4000,
    messages: [
      {
        role: 'user',
        content: [
          { type: 'text', text: prompt },
          { type: 'image_url', image_url: { url: dataUrl } },
        ],
      },
    ],
  };
  const res = await fetch(BASE, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${apiKey}`,
    },
    body: JSON.stringify(body),
  });
  if (res.status === 429) {
    if (retriesLeft <= 0) throw new Error('429 exhausted retries');
    await new Promise((r) => setTimeout(r, 15000));
    return chat(dataUrl, apiKey, prompt, retriesLeft - 1);
  }
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`HTTP ${res.status}: ${text.slice(0, 500)}`);
  }
  const json = await res.json();
  // content may appear in "content" or "reasoning_content"
  const msg = json?.choices?.[0]?.message;
  const content = msg?.content || msg?.reasoning_content || '';
  return content;
}

// ---------- run ----------
const apiKey = getApiKey();

// collect existing images only
const avail = IMAGES.map((im) => ({
  ...im,
  exists: fs.existsSync(im.path),
}));
const existing2d = avail
  .filter((im) => im.group === '2D')
  .filter((im) => im.exists);
if (existing2d.length === 0) {
  console.error('No 2D sprite images found to compare.');
  process.exit(1);
}

const textImages = [{ type: 'text', text: PROMPT }];
for (const im of avail.filter((i) => i.exists)) {
  console.error(`[attach] ${im.label} (${im.group}) size=${fs.statSync(im.path).size}`);
  textImages.push({ type: 'image_url', image_url: { url: toDataUrl(im.path) } });
}

const body = {
  model: MODEL,
  max_tokens: 4000,
  messages: [{ role: 'user', content: textImages }],
};

console.error(`[call] sending ${avail.filter((i)=>i.exists).length} images to ${MODEL} (single grouped call)`);

let result = '';
try {
  const res = await fetch(BASE, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${apiKey}` },
    body: JSON.stringify(body),
  });
  if (res.status === 429) {
    console.error('[retry] 429, backing off 15s (up to 5 retries)');
    result = await chat2_with_backoff(textImages, apiKey);
  } else if (!res.ok) {
    const t = await res.text();
    throw new Error(`HTTP ${res.status}: ${t.slice(0, 500)}`);
  } else {
    const json = await res.json();
    const msg = json?.choices?.[0]?.message;
    result = msg?.content || msg?.reasoning_content || '';
  }
} catch (e) {
  console.error('[FAIL]', e.message);
  process.exitCode = 1;
  result = `\n> 调用失败：${e.message}\n`;
}

async function chat2_with_backoff(contentParts, apiKey, retries = 5) {
  let lastErr;
  for (let i = 1; i <= retries; i++) {
    await new Promise((r) => setTimeout(r, 15000));
    try {
      const res = await fetch(BASE, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${apiKey}` },
        body: JSON.stringify({ model: MODEL, max_tokens: 4000, messages: [{ role: 'user', content: contentParts }] }),
      });
      if (res.status === 429) {
        lastErr = new Error('429 again');
        continue;
      }
      if (!res.ok) {
        const t = await res.text();
        throw new Error(`HTTP ${res.status}: ${t.slice(0, 300)}`);
      }
      const json = await res.json();
      const msg = json?.choices?.[0]?.message;
      return msg?.content || msg?.reasoning_content || '';
    } catch (e) {
      lastErr = e;
    }
  }
  throw lastErr || new Error('backoff exhausted');
}

// ---------- write report ----------
const md = [];
md.push('# 3D 体素建模 vs 2D 立绘 视觉质量对比报告');
md.push('');
md.push(`- 调用模型：${MODEL}`);
md.push('- 对比方式：前 2 张为游戏内 3D 体素人截图，后 3 张为 AI 生成 2D 角色立绘，一次性送入 qwen 判图');
md.push('- 生成时间：' + new Date().toLocaleString('zh-CN', { timeZone: 'Asia/Shanghai' }));
md.push('');
md.push('## 送检素材');
md.push('');
md.push('| 分组 | 标签 | 文件 | 存在 |');
md.push('|------|------|------|------|');
for (const im of avail) {
  md.push(`| ${im.group} | ${im.label} | \`${path.basename(im.path)}\` | ${im.exists ? '✓' : '✗ 不存在'} |`);
}
md.push('');
md.push('## qwen 对比结论（原文）');
md.push('');
md.push(result);
md.push('');
fs.writeFileSync(OUT_MD, md.join('\n'), 'utf8');
console.error('[done] wrote report to ' + OUT_MD);
console.log(md.join('\n'));
