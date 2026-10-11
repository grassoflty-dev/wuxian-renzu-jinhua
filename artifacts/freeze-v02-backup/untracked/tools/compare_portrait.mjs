// compare_portrait.mjs — 对比立绘 vs 体素渲染，找出染色具体问题
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const name = process.argv[2] || 'zhenghai';
const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const sleep = ms => new Promise(r => setTimeout(r, 15000));

// 两张图一起做对比判断
const PROMPT = `有两组图片。第一张是《无限恐怖》主角郑吒的立绘（全身原画），第二张是根据该立绘生成的体素(Minecraft方块风)3D建模正面渲染图。

请对比分析，列出体素渲染相对于立绘的具体不足和问题（中文）：
1. 颜色准确性：上衣T恤颜色、裤子颜色与立绘是否一致？色差多少？
2. 立体感：体素身体有无明暗渐变（受光面亮、背光面暗）？还是纯平色看起来像纸片贴纸？
3. 错位：身体各部位在体素上的位置是否与立绘对应部位对齐？（肩线、腰线、裤腰等）
4. 头身衔接：头部(白色方块脸+黑色寸头)与身体(深色T恤)之间是否有突兀的分界线或断层？
5. 细节丢失：立绘中衣服褶皱/肌肉轮廓/腰带/口袋等细节，在体素上能否看到？
6. 整体观感：这个体素角色从正面看是否像立绘中的郑吒？哪些特征最缺失？

请具体、客观地指出问题，以便我修改代码改进。`;

function dataUrl(fp) {
  return { type: 'image_url', image_url: { url: `data:image/png;base64,${fs.readFileSync(fp).toString('base64')}` } };
}

async function callOnce(key, parts) {
  const res = await fetch(ENDPOINT, {
    method: 'POST',
    headers: { 'Authorization': `Bearer ${key}`, 'Content-Type': 'application/json' },
    body: JSON.stringify({ model: MODEL, messages: [{ role: 'user', content: [{ type: 'text', text: PROMPT }, ...parts] }], max_tokens: 3000 }),
  });
  const text = await res.text();
  if (res.status === 429 || res.status >= 500 || /LITELLM_UNAVAILABLE|SERVICE_BUSY|overloaded|GATEWAY-TIME|timeout|timed out/i.test(text)) {
    return { retry: true, content: '', reasoning: '' };
  }
  if (!res.ok) throw new Error(`API ${res.status}: ${text.slice(0, 300)}`);
  const d = JSON.parse(text);
  const msg = d.choices?.[0]?.message || {};
  return { retry: false, content: typeof msg.content === 'string' ? msg.content : '', reasoning: typeof msg.reasoning_content === 'string' ? msg.reasoning_content : '' };
}

async function main() {
  const p = path.join(os.homedir(), '.dsh', '.credentials.yaml');
  const key = fs.readFileSync(p, 'utf8').match(/TOKENRHYTHM_API_KEY:\s*(\S+)/)[1];

  // 立绘路径（用server-rs/ui/assets下的那个，渲染使用的源文件）
  const portraitPath = path.join(__dirname, '..', 'server-rs\\ui\\assets\\img\\pc_zhengzha.png');
  const renderFront = path.join(__dirname, 'shots', `render_${name}_front.png`);

  console.error('Loading:', portraitPath);
  console.error('Loading:', renderFront);

  if (!fs.existsSync(portraitPath)) {
    // 试试另一个可能的路径
    const alt = path.join(__dirname, 'design', 'raw_snapshot_2007', 'pc_zhengzha.png');
    if (fs.existsSync(alt)) {
      console.error('Found alt:', alt);
    }
  }

  const parts = [];

  if (fs.existsSync(portraitPath)) {
    parts.push(dataUrl(portraitPath));
  } else {
    // Try alternate paths
    const candidates = [
      path.join(__dirname, 'design', 'raw_snapshot_2007', 'pc_zhengzha.png'),
      path.join(__dirname, 'design', 'backup_cutout', 'pc_zhengzha.png'),
      path.join(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png'),
    ];
    for (const c of candidates) {
      if (fs.existsSync(c)) {
        console.error('Using alt path:', c);
        parts.push(dataUrl(c));
        break;
      }
    }
    if (parts.length === 0) throw new Error('Cannot find pc_zhengzha.png');
  }

  parts.push(dataUrl(renderFront));

  let r = { retry: true, content: '', reasoning: '' };
  for (let a = 0; a < 5; a++) {
    try {
      r = await callOnce(key, parts);
      if (!r.retry) break;
    } catch (e) {
      if (a === 4) throw e;
    }
    console.error(`  retry ${a + 1} (429/5xx) ...`);
    await sleep(15000);
  }

  const body = (r.content || '').trim() || (r.reasoning || '').trim();
  console.log(body);
}

main().catch(e => { console.error('FATAL', e.message); process.exit(1); });
