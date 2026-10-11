// final_cmp.mjs — 最终轮对比：立绘 vs 最新渲染，给出改进总结
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const sleep = ms => new Promise(r => setTimeout(r, 15000));

const PROMPT = `有两张图。左图是郑吒全身立绘（原画），右图是根据该立绘用体素(Minecraft方块)生成的染色版正面渲染图。

请逐条回答（中文）：
1. 上衣颜色是否接近立绘？(蓝灰/深灰/暖灰?)
2. 裤子颜色是否与上衣可区分？(比上衣更深吗?)
3. 皮肤/手臂颜色是否正常肤色? 还是发白?
4. 身体两侧（手臂、躯干侧面）是否看起来比正面暗一些? 立体感有无改善?
5. 头部与躯干的衔接是否自然? 有无突兀白色断层?

最后给一个一句话结论：这次改动相比原始方案有没有明显改善?`;

function dataUrl(fp) {
  return { type: 'image_url', image_url: { url: `data:image/png;base64,${fs.readFileSync(fp).toString('base64')}` } };
}

async function callOnce(key, parts) {
  const res = await fetch(ENDPOINT, {
    method: 'POST',
    headers: { 'Authorization': `Bearer ${key}`, 'Content-Type': 'application/json' },
    body: JSON.stringify({ model: MODEL, messages: [{ role: 'user', content: [{ type: 'text', text: PROMPT }, ...parts] }], max_tokens: 2000 }),
  });
  const text = await res.text();
  if (res.status === 429 || res.status >= 500 || /overloaded|GATEWAY-TIME|timeout/i.test(text)) return { retry: true };
  if (!res.ok) throw new Error(`API ${res.status}: ${text.slice(0, 300)}`);
  const d = JSON.parse(text);
  const msg = d.choices?.[0]?.message || {};
  return { retry: false, content: msg.content || '' };
}

async function main() {
  const p = path.join(os.homedir(), '.dsh', '.credentials.yaml');
  const key = fs.readFileSync(p, 'utf8').match(/TOKENRHYTHM_API_KEY:\s*(\S+)/)[1];

  // Find portrait
  const portraitPath = path.join(__dirname, '..', 'server-rs\\ui\\assets\\img\\pc_zhengzha.png');
  if (!fs.existsSync(portraitPath)) throw new Error('no portrait: ' + portraitPath);

  const renderFront = path.join(__dirname, 'shots', 'render_zhenghai_front.png');
  if (!fs.existsSync(renderFront)) throw new Error('no render: ' + renderFront);

  const parts = [dataUrl(portraitPath), dataUrl(renderFront)];

  let r = { retry: true };
  for (let a = 0; a < 5; a++) {
    r = await callOnce(key, parts);
    if (!r.retry) break;
    console.error(`  retry ${a+1} ...`);
    await sleep(15000);
  }

  console.log(r.content || '(empty response)');
}
main().catch(e => { console.error('FATAL:', e.message); process.exit(1); });
