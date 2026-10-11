// compare_deep.mjs — 深度对比：立绘 vs 4视图体素渲染，找出根本问题
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const sleep = ms => new Promise(r => setTimeout(r, ms));

function loadKey() {
  const p = path.join(os.homedir(), '.dsh', '.credentials.yaml');
  const m = fs.readFileSync(p, 'utf8').match(/TOKENRHYTHM_API_KEY:\s*(\S+)/);
  if (!m) throw new Error('no key');
  return m[1];
}
function dataUrl(fp) {
  return { type: 'image_url', image_url: { url: `data:image/png;base64,${fs.readFileSync(fp).toString('base64')}` } };
}
async function callOnce(key, messages) {
  const res = await fetch(ENDPOINT, {
    method: 'POST',
    headers: { 'Authorization': `Bearer ${key}`, 'Content-Type': 'application/json' },
    body: JSON.stringify({ model: MODEL, messages, max_tokens: 4000 }),
  });
  const text = await res.text();
  if (res.status === 429 || res.status >= 500 || /LITELLM_UNAVAILABLE|SERVICE_BUSY|overloaded|GATEWAY-TIME|timeout|timed out/i.test(text)) {
    return { retry: true, content: '' };
  }
  if (!res.ok) throw new Error(`API ${res.status}: ${text.slice(0, 500)}`);
  const d = JSON.parse(text);
  const msg = d.choices?.[0]?.message || {};
  const c = typeof msg.content === 'string' ? msg.content : '';
  const r = typeof msg.reasoning_content === 'string' ? msg.reasoning_content : '';
  return { retry: false, content: c, reasoning: r };
}

async function main() {
  const key = loadKey();
  const portrait = path.resolve(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');
  const views = ['front', 'side', 'back', 'face'];
  const parts = [];
  parts.push({ type: 'text', text: '【图1：原始立绘 pc_zhengzha.png（郑吒全身立绘，2D动漫写实风男性角色）——这是参考目标】' });
  parts.push(dataUrl(portrait));
  parts.push({ type: 'text', text: '【图2-5：体素方块人（Minecraft风）3D建模渲染图，纯白背景】\n图2: 正面全身\n图3: 侧面\n图4: 背面\n图5: 脸部特写\n\n这些是用 THREE.js BoxGeometry 方块（约1480块）拼成的人形，然后把立绘逐像素投影染色到每个方块上（取立绘对应位置的平均像素颜色染方块，侧面按X距离做了暗化）。当前用户反馈："感觉没啥区别"——意思是染完色之后看起来还是方块人、和立绘差距很大。\n\n请仔细对比图1与图2-5，从建模优化角度深度分析**根本问题**（不要只说"有点方块感"，要说清原因）：\n\n1. **体型比例**：和立绘比，头身比、肩宽/腰细比例（倒三角程度）、手臂粗细/长度、腿长比例——差距在哪？是头太大？腰不够细？腿太短？肩不够宽？给出具体方向。\n2. **方块感/体素感的根源**：为什么染色后还是方块人？是方块太大/太少导致曲面完全不像人？是环/层数量不够使躯干/四肢看起来像圆柱筒而非人体？是方块接缝/排列方式的问题？\n3. **染色投影的根本缺陷**：当前方案是"遍历每个方块的世界坐标(x,y)→线性映射到立绘(u,v)→5×5平均采样取色染方块"。这个方案有什么致命问题？比如：正面立绘的颜色投到侧面方块上是否合理？立绘的阴影/高光被当成本色投影后是否导致脏色？四肢末端被裁剪后是否只有躯干染上色？\n4. **头部**：当前是体素椭球头+Canvas像素画脸+前面贴一个Plane平面脸贴片，和立绘的黑发/浓眉/严肃脸差在哪？头发是不是太少/太短？脸贴片角度对不对？\n5. **衣服细节**：立绘的深炭灰T恤、战术裤、腰带、裤袋、靴子——体素上有多少能辨认出来？为什么有战术装备方块还是不像？\n6. **优化优先级排序**：如果只能改3件事（按影响大小排序），具体该改什么？给出可操作方向（比如"把躯干环数从12提到X""改染色算法为XXX"等）。\n\n请用中文回答，直说问题，不用客套。分析要深入到几何/染色算法层面。' });
  for (const v of views) {
    parts.push(dataUrl(path.join(__dirname, 'shots', `render_zhenghai_${v}.png`)));
  }
  let r = { retry: true, content: '', reasoning: '' };
  for (let a = 0; a < 5; a++) {
    try { r = await callOnce(key, [{ role: 'user', content: parts }]); if (!r.retry) break; } catch (e) { if (a === 4) throw e; }
    console.error(`  retry ${a + 1} (429/5xx) ...`);
    await sleep(15000);
  }
  const body = (r.content || '').trim() || (r.reasoning || '').trim();
  console.log('=== DEEP COMPARISON ===\n');
  console.log(body);
}
main().catch(e => { console.error('FATAL', e.message); process.exit(1); });
