// ⚠️【废弃标注】3D 地图质检配套脚本（prompt 面向 3D 体素地图），游戏已切 World2D；截图评估请用 probe_world2d.mjs + qwen_map_eval.mjs。
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
// 密钥从 ~/.dsh/.credentials.yaml 读取（原硬编码密钥已移除）
const API_KEY = (() => {
  const raw = fs.readFileSync(path.join(os.homedir(), '.dsh', '.credentials.yaml'), 'utf8');
  const m = raw.match(/TOKENRHYTHM_API_KEY:\s*(\S+)/);
  if (!m) throw new Error('TOKENRHYTHM_API_KEY not found in ~/.dsh/.credentials.yaml');
  return m[1];
})();

const JUDGE_PROMPT = `这是游戏《无限轮回》3D 地图画面截图。请评估视觉质量，中文简洁，严格按 5 行输出：
人物造型: （体素人是否空心/头是否浮空/脖子是否与身体连接/整体是否完整连贯）
对比度: （蓝衣人物在地面/环境上是否清晰可见）
环境: （地面/墙壁是否单调，有无多种材质和装饰，生化蜂巢工业氛围）
精细度: （头/躯干/四肢细节是否足够）
结论: （PASS/WARN/FAIL + 一句话问题，无问题写无）`;

const shotsDir = path.join(__dirname, 'shots');
const files = ['map_qc_1.png', 'map_qc_2.png', 'map_qc_3.png'];

function sleep(ms) {
  return new Promise(r => setTimeout(r, ms));
}

async function callWithRetry(fetchBody) {
  let lastErr;
  for (let attempt = 1; attempt <= 5; attempt++) {
    try {
      const resp = await fetch(ENDPOINT, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${API_KEY}`,
        },
        body: JSON.stringify(fetchBody),
      });
      if (!resp.ok && resp.status !== 429 && resp.status >= 500) {
        // retry on 429 and server 5xx
        lastErr = new Error(`HTTP ${resp.status}: ${resp.statusText}`);
      } else if (resp.ok) {
        return await resp.json();
      } else {
        lastErr = new Error(`HTTP ${resp.status}: ${resp.statusText}`);
      }
    } catch (e) {
      lastErr = e;
    }
    console.log(`  [retry] attempt ${attempt} failed: ${lastErr.message}, backing off 15s...`);
    await sleep(15000);
  }
  throw lastErr;
}

async function analyzeOne(fileName) {
  const filePath = path.join(shotsDir, fileName);
  const b64 = fs.readFileSync(filePath).toString('base64');
  const dataUrl = `data:image/png;base64,${b64}`;

  const body = {
    model: MODEL,
    messages: [
      {
        role: 'user',
        content: [
          { type: 'text', text: JUDGE_PROMPT },
          { type: 'image_url', image_url: { url: dataUrl } },
        ],
      },
    ],
    max_tokens: 4000,
  };

  console.log(`\n=== ${fileName} (${(b64.length * 0.75 / 1024).toFixed(0)} KB) ===`);
  const data = await callWithRetry(body);

  // 兼容解析：优先 content，退回 reasoning_content
  let text = '';
  try {
    const msg = data?.choices?.[0]?.message;
    text = msg?.content || msg?.reasoning_content || '';
  } catch (e) {
    text = '';
  }
  if (!text.trim()) {
    text = `[解析失败] raw: ${JSON.stringify(data).slice(0, 500)}`;
  }
  return { fileName, text };
}

async function main() {
  const results = [];
  for (const f of files) {
    const r = await analyzeOne(f);
    results.push(r);
  }

  // write report
  const lines = [];
  lines.push('# 地图质检报告（qwen3.7-flash 视觉评估）');
  lines.push('');
  lines.push(`生成时间：${new Date().toISOString()}`);
  lines.push('');
  for (const { fileName, text } of results) {
    lines.push(`## ${fileName}`);
    lines.push('');
    lines.push('```');
    lines.push(text.trim());
    lines.push('```');
    lines.push('');
  }

  const reportPath = path.join(__dirname, 'map_qc_report.md');
  fs.writeFileSync(reportPath, lines.join('\n'), 'utf8');
  console.log(`\n\nReport written to ${reportPath}`);
  console.log(JSON.stringify(results, null, 2));
}

main().catch(e => {
  console.error('FATAL:', e);
  process.exit(1);
});
