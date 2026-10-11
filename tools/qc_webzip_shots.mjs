// qc_webzip_shots.mjs — 用 tokenrhythm qwen3.7-flash（识图）检查《无限轮回》蛛丝拉拽验收截图
// 用法: node qc_webzip_shots.mjs
// 依赖: C:\Users\PUBLIC_USER\.dsh\.credentials.yaml 的 TOKENRHYTHM_API_KEY
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SHOTS = path.join(__dirname, 'shots');
const sleep = ms => new Promise(r => setTimeout(r, ms));

function loadKey() {
  const raw = fs.readFileSync('C:\\Users\\PUBLIC_USER\\.dsh\\.credentials.yaml', 'utf8');
  const m = raw.match(/TOKENRHYTHM_API_KEY\s*[:=]\s*"?([^"\r\n]+)/);
  if (!m) throw new Error('TOKENRHYTHM_API_KEY not found');
  return m[1].trim();
}

async function askVision(apiKey, files) {
  const images = files.map(f => {
    const b64 = fs.readFileSync(f).toString('base64');
    return { type: 'image_url', image_url: { url: 'data:image/png;base64,' + b64 } };
  });
  const body = {
    model: 'qwen3.7-flash',
    max_tokens: 4000,
    messages: [{
      role: 'user',
      content: [
        { type: 'text', text: `你是视觉质检子代理。请逐张检查以下 ${files.length} 张《无限轮回》2D 俯视像素世界地图的验收截图（功能：蜘蛛侠蛛丝拉拽），按顺序回答：

背景：世界是 2D 俯视像素地图（深色底，蜂巢工业风），墙壁是深色工业像素块，地板是蜂巢纹理，玩家是白色小三角/像素小人。每格 24 art px、显示放大 2 倍，相机以玩家为中心。

图1 webzip_1_enter.png：拉拽前——玩家站在一条竖直走廊里，上方不远处有一面墙。
图2 webzip_2_zip.png：拉拽中——玩家被白色蛛丝线从走廊中段拉向那面墙。预期可见：白色蛛丝线（从玩家连到墙上的命中点、可微微下垂的曲线）、墙命中点一个白色小圆点、玩家处于浮空/悬空弧线状态（与地面阴影分离）或已贴近墙边。

对每张图回答：
- A. 是否能看到 2D 俯视像素地图（地板/墙）？画面大致内容（玩家在哪、墙在哪、什么方向）？
- B. 是否能看到白色蛛丝线/曲线？从哪里连到哪里（方向）？命中点白色小圆点是否存在？
- C. 玩家角色是否浮空/悬空（与地面阴影分离）或明显靠近墙边？
- D. 是否有异常（黑屏、报错文字、大面积遮挡、渲染破损）？

最后明确结论：webzip_2_zip.png 中「蛛丝拉拽动画表现」（白色蛛丝线 + 玩家被拉向墙）是否可见？是/否 + 理由。请直接输出结构化中文文本，不要客套。` },
        ...images,
      ],
    }],
  };
  let lastErr = null;
  for (let attempt = 0; attempt < 5; attempt++) {
    const r = await fetch('https://tokenrhythm.studio/v1/chat/completions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: 'Bearer ' + apiKey },
      body: JSON.stringify(body),
    });
    if (r.status === 429) {
      lastErr = '429 rate limited';
      console.log('[QC] 429，退避 ' + (15 * (attempt + 1)) + 's …');
      await sleep(15000 * (attempt + 1));
      continue;
    }
    if (!r.ok) { lastErr = 'HTTP ' + r.status + ' ' + (await r.text()).slice(0, 300); break; }
    const j = await r.json();
    const msg = j.choices && j.choices[0] && j.choices[0].message;
    if (msg) {
      return (msg.content || '').trim() || ('[no content; reasoning: ' + String(msg.reasoning_content || '').slice(0, 500) + ']');
    }
    return JSON.stringify(j).slice(0, 800);
  }
  throw new Error('vision api failed: ' + lastErr);
}

(async () => {
  const apiKey = loadKey();
  const files = ['webzip_1_enter.png', 'webzip_2_zip.png'].map(f => path.join(SHOTS, f));
  for (const f of files) {
    if (!fs.existsSync(f)) throw new Error('missing ' + f);
  }
  const verdict = await askVision(apiKey, files);
  console.log('=== VISION VERDICT ===');
  console.log(verdict);
})().catch(e => { console.error('FATAL', e.message); process.exit(2); });
