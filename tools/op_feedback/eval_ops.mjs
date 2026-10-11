// ============================================================================
// eval_ops.mjs —— 用 qwen3.7-flash 逐操作评估「操作→画面反馈」
//
// 读取 ops_result.json + 截图, 对每个操作把 before/after 两张图 + 操作描述发给
// qwen3.7-flash(tokenrhythm API, 模型名不带前缀), 让其判断执行操作后画面是否
// 出现对应反馈, 并给四档评分: 有明显反馈 / 微弱反馈 / 无反馈 / 反馈异常。
// 输出 eval_report.md(每操作: 预期反馈、实际画面描述、评分、结论) + 汇总。
//
// 用法: node tools/op_feedback/eval_ops.mjs
// 硬约束: max_tokens>=3000, 429 退避 15s×5, data URL 传图, 兼容 reasoning_content
// ============================================================================
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const RESULT = path.join(__dirname, 'ops_result.json');
const SHOTS = path.join(__dirname, 'shots');
const REPORT = path.join(__dirname, 'eval_report.md');
const ENDPOINT = 'https://tokenrhythm.studio/v1/chat/completions';
const MODEL = 'qwen3.7-flash';
const MAX_TOKENS = 3000;
const RETRY_SLEEP = 15000;
const MAX_ATTEMPTS = 5;
const sleep = ms => new Promise(r => setTimeout(r, ms));

// 操作预期反馈(复用 operations_list.md)
const EXPECTED = {
  '01': '标题页 → 世界地图模式: 界面切换, HUD/地图/新手引导出现',
  '02': '新手引导弹层关闭, 露出世界地图',
  '03': '玩家角色沿格子位移(场景/位置变化)',
  '04': '第三人称相机推近/拉远(画面缩放)',
  '05': '相机水平旋转(视角转动)',
  '06': '交互后出现剧情场景/兑换卡片/提示(worldMsg)',
  '07': '弹出大地图覆盖层(bigmap)',
  '08': '切换到主神空间(跨世界地图变化)',
  '09': '世界地图 → 3D 战斗副本(Zone3D 场景+敌人)',
  '10': '角色攻击动画+武器特效, 敌人受击/掉血',
  '11': '技能施法特效+冷却, 敌人掉血(未装备则提示)',
  '11b': '技能施法特效+冷却, 敌人掉血(未装备则提示)',
  '12': '角色闪避位移+残影, 闪避提示',
  '13': '角色在副本地面移动(相机跟随)',
  '14': '弹出技能面板(技能分类列表)',
  '15': '弹出设置面板(分辨率/帧率按钮)',
  '16': '画面画质/清晰度变化(720p)',
  '16b': '画面画质/清晰度变化(1440p)',
  '17': '渲染帧率受限(画面刷新变慢)',
  '18': '弹出死亡档案卡片(记录或提示)',
};

function loadKey() {
  const p = path.join(os.homedir(), '.dsh', '.credentials.yaml');
  const raw = fs.readFileSync(p, 'utf8');
  const m = raw.match(/TOKENRHYTHM_API_KEY:\s*(\S+)/);
  if (!m) throw new Error('TOKENRHYTHM_API_KEY not found');
  return m[1];
}
function imgPart(fp) {
  const b64 = fs.readFileSync(fp).toString('base64');
  return { type: 'image_url', image_url: { url: `data:image/png;base64,${b64}` } };
}
async function callOnce(key, parts) {
  const body = { model: MODEL, messages: [{ role: 'user', content: [{ type: 'text', text: parts.text }, ...parts.images] }], max_tokens: MAX_TOKENS };
  const res = await fetch(ENDPOINT, { method: 'POST', headers: { 'Authorization': `Bearer ${key}`, 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  if (res.status === 429) return { retry: true, reason: '429', content: '' };
  const text = await res.text();
  if (!res.ok) {
    if (res.status === 429 || res.status >= 500 || /LITELLM_UNAVAILABLE|SERVICE_BUSY|overloaded|GATEWAY-TIME|timeout|timed out|MODEL_NOT_AVAILABLE|429/i.test(text)) return { retry: true, reason: `${res.status}`, content: '' };
    throw new Error(`API error ${res.status}: ${text.slice(0, 400)}`);
  }
  let data; try { data = JSON.parse(text); } catch { throw new Error('bad json'); }
  const msg = data.choices?.[0]?.message || {};
  return { retry: false, reason: '', content: typeof msg.content === 'string' && msg.content.trim() ? msg.content : (typeof msg.reasoning_content === 'string' ? msg.reasoning_content : '') };
}
async function callWithRetry(key, parts) {
  for (let a = 0; a < MAX_ATTEMPTS; a++) {
    const r = await callOnce(key, parts);
    if (!r.retry) return r;
    console.log(`  [retry ${a + 1}/5] ${r.reason}, 等 ${RETRY_SLEEP}ms`);
    await sleep(RETRY_SLEEP);
  }
  return { retry: false, reason: 'giveup', content: '' };
}
// 从 qwen 回答中解析评分档
function parseRating(text) {
  const t = String(text || '');
  const map = { '明显反馈': '有明显反馈', '微弱反馈': '微弱反馈', '无反馈': '无反馈', '反馈异常': '反馈异常' };
  let hit = '';
  // 先匹配评分词
  for (const k of ['明显反馈', '微弱反馈', '无反馈', '反馈异常']) if (t.includes(k)) { hit = map[k]; break; }
  return { rating: hit || '未判定', raw: t.slice(0, 900) };
}

(async () => {
  if (!fs.existsSync(RESULT)) { console.error('缺少 ops_result.json, 先运行 run_ops.mjs'); process.exit(1); }
  const data = JSON.parse(fs.readFileSync(RESULT, 'utf8'));
  const ops = data.ops || [];
  const key = loadKey();
  const evals = [];
  console.log('评估 ' + ops.length + ' 个操作...');

  for (const op of ops) {
    const before = path.join(SHOTS, op.before + '.png');
    const after = path.join(SHOTS, op.after + '.png');
    const bothExist = fs.existsSync(before) && fs.existsSync(after);
    const opName = op.name || op.id;
    const expected = EXPECTED[op.id] || '存在对应画面变化';
    if (!bothExist) {
      evals.push({ id: op.id, name: opName, expected, desc: '截图缺失', rating: '无反馈', verdict: '截图缺失,无法评估' });
      console.log(`  [${op.id}] ${opName} 截图缺失,跳过`);
      continue;
    }
    const prompt = `你是一名游戏"操作→画面反馈"质量检测员。下面两张图是执行【${opName}(${op.id})】前后各截取的游戏画面截图:第一张是操作前,第二张是操作后。\n操作说明:${op.desc}。\n预期反馈:${expected}。\n执行状态:${op.status}(fail 表示操作未能真正执行,应按无变化判断;pass 表示操作已派发)。\n\n请对比两张截图,回答:执行该操作后,画面是否出现了对应的视觉反馈(例如角色/视角/位置变化、弹窗/面板出现、敌人受击/掉血、特效、场景切换等)?\n严格按照以下格式回答:\n评分:明显反馈 / 微弱反馈 / 无反馈 / 反馈异常(四选一)\n画面说明:用一两句中文描述两张图的关键差异,没有变化就写"两张图几乎一致"。\n结论:简短判断该操作画面反馈是否正常,是否存在缺失或异常。`;

    const text = prompt;
    const images = [imgPart(before), imgPart(after)];
    const r = await callWithRetry(key, { text, images });
    if (!r.content) {
      evals.push({ id: op.id, name: opName, expected, desc: '评估失败', rating: '未判定', verdict: 'API 未返回内容' });
      console.log(`  [${op.id}] ${opName} => 评估失败(空) ${r.reason}`);
      continue;
    }
    const { rating, raw } = parseRating(r.content);
    evals.push({ id: op.id, name: opName, expected, desc: `${op.desc}(执行状态:${op.status})`, rating, verdict: raw, raw });
    console.log(`  [${op.id}] ${opName} => ${rating}`);
    await sleep(400); // 轻微限速
  }

  // 汇总
  const count = n => evals.filter(e => e.rating === n).length;
  const summary = {
    明显反馈: count('有明显反馈'), 微弱反馈: count('微弱反馈'), 无反馈: count('无反馈'), 反馈异常: count('反馈异常'), 未判定: count('未判定'),
  };

  // 写报告
  let md = `# 无限轮回 · 操作-画面反馈评估报告(qwen3.7-flash)\n\n`;
  md += `> 评估模型: ${MODEL} via tokenrhythm · 逐操作前后截图对比判画面反馈\n`;
  md += `> 操作总数: ${evals.length} · 截图对: ${evals.filter(e => e.desc !== '截图缺失,无法评估').length}\n\n`;
  md += `## 汇总\n\n| 结论 | 数量 |\n|---|---|\n`;
  for (const k of ['有明显反馈', '微弱反馈', '无反馈', '反馈异常', '未判定']) md += `| ${k} | ${summary[k]} |\n`;
  md += `\n## 逐操作明细\n\n| ID | 操作 | 执行状态 | 预期反馈 | 评分 | 结论/画面说明 |\n|---|---|---|---|---|---|\n`;
  // 格式: 合并 verdict 首段
  for (const e of evals) {
    const verdictFirst = String(e.verdict).split('\n').filter(Boolean).join(' ').slice(0, 130);
    md += `| ${e.id} | ${e.name} | ${e.desc.includes('执行状态:') ? e.desc.match(/执行状态:(\w+)/)?.[1] || '' : ''} | ${e.expected} | ${e.rating} | ${verdictFirst} |\n`;
  }
  md += `\n---\n\n## 评估详情\n\n`;
  for (const e of evals) {
    md += `### ${e.id} · ${e.name}\n`;
    md += `- 预期反馈: ${e.expected}\n`;
    md += `- 评分: **${e.rating}**\n`;
    md += `- 画面说明/结论:\n\n> ${String(e.verdict).trim().replace(/\n/g, '\n> ')}\n\n`;
  }
  md += `\n---\n*报告由 eval_ops.mjs 生成。截图: ${path.basename(SHOTS)}/*。*\n`;
  fs.writeFileSync(REPORT, md, 'utf8');
  fs.writeFileSync(path.join(__dirname, 'eval_result.json'), JSON.stringify({ summary, evals }, null, 2), 'utf8');

  console.log('\n===== 评估完成 =====');
  for (const k of ['有明显反馈', '微弱反馈', '无反馈', '反馈异常', '未判定']) console.log(`  ${k}: ${summary[k]}`);
  console.log('报告输出: ' + REPORT);
  process.exit(summary['反馈异常'] ? 2 : 0);
})().catch(e => { console.error('FATAL', e); process.exit(2); });
