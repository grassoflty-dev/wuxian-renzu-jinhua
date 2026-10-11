// qwen_scene.mjs — 场景画面专项分析(地面/墙/物件/氛围)
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const fp = process.argv[2];
const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 3500;
const RETRY_SLEEP = 15000;
const sleep = ms => new Promise(r => setTimeout(r, ms));

const PROMPT = `这是游戏 3D 地图画面(俯视第三人称)。请逐项详细描述:
1. **地面/地板**: 颜色(全黑?灰色?有纹理?)、是否能看到地板方块/纹理、哪些区域黑。
2. **墙**: 颜色、样式(贴图?纯色方块?)、高度感。
3. **场景物件**: 画面里有哪些物体(角色/敌人/调查点/装饰)?有没有桌子、门、椅子、家具、树?逐个描述位置和样子。
4. **氛围**: 整体亮/暗?有没有光照?
5. **问题**: 这个场景看起来有什么问题(黑屏/粗糙/空洞/缺少物件)?
中文输出,看图说话,不要臆造。`;

function loadKey() {
  const raw = fs.readFileSync(path.join(os.homedir(), '.dsh', '.credentials.yaml'), 'utf8');
  const m = raw.match(/TOKENRHYTHM_API_KEY:\s*(\S+)/);
  if (!m) throw new Error('no key');
  return m[1];
}
async function callOnce(key, parts) {
  const body = { model: MODEL, messages: [{ role: 'user', content: [{ type: 'text', text: PROMPT }, ...parts] }], max_tokens: MAX_TOKENS };
  const res = await fetch(ENDPOINT, { method: 'POST', headers: { 'Authorization': `Bearer ${key}`, 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  if (res.status === 429) return { retry: true, content: '' };
  const text = await res.text();
  if (!res.ok) {
    if (res.status >= 500 || /UNAVAILABLE|BUSY|timeout/i.test(text)) return { retry: true, content: '' };
    throw new Error(`API ${res.status}: ${text.slice(0, 300)}`);
  }
  let d; try { d = JSON.parse(text); } catch { throw new Error('bad json'); }
  const msg = d.choices?.[0]?.message || {};
  return { retry: false, content: typeof msg.content === 'string' ? msg.content : (msg.reasoning_content || '') };
}
(async () => {
  if (!fs.existsSync(fp)) { console.error('MISSING', fp); process.exit(1); }
  const key = loadKey();
  const b64 = fs.readFileSync(fp).toString('base64');
  for (let a = 0; a < 5; a++) {
    const r = await callOnce(key, [{ type: 'image_url', image_url: { url: `data:image/png;base64,${b64}` } }]);
    if (!r.retry) { console.log(r.content); process.exit(0); }
    console.log(`retry ${a + 1}`);
    await sleep(RETRY_SLEEP);
  }
})().catch(e => { console.error('ERR', e.message); process.exit(1); });
