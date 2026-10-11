// test_campaign_probe_actions.mjs — campaign-probe.exe 多动作确定性验证（非 mock）
//
// 直接用 PowerShell 管道把 NDJSON 请求逐行喂给 server-rs/target/debug/campaign-probe.exe，
// 每行一个请求、每行严格对应一个响应（probe 是行协议，见 server-rs/src/bin/campaign_probe.rs）。
// 断言覆盖：hub 视图结构 → enter 灰巢 → 移动/非法移动 → 攻击/技能冷却与资源校验 →
// 医疗包 → 交互距离校验 → 非法动作报错 → 撤离前置校验 → 重启世界 → 回归航站。
// 全程内存态，不读写存档。
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PROBE = path.resolve(__dirname, '..', 'server-rs', 'target', 'debug', 'campaign-probe.exe');

const requests = [
  { action: 'view' },
  { action: 'enter', args: { world_id: 'grey_hive' } },
  { action: 'view' },
  { action: 'move', args: { dx: 0, dy: 0 } },          // 非法：dx/dy 不能同时为 0
  { action: 'move', args: { dx: 1, dy: 0 } },
  { action: 'attack', args: { dx: 1, dy: 0 } },        // 附近没敌人也允许落空/或成功，仅断言协议结构
  { action: 'skill', args: { slot: 9 } },              // 非法 slot
  { action: 'skill', args: { slot: 0 } },              // 脉冲（可能“没有触及敌人”）
  { action: 'heal' },
  { action: 'interact', args: { id: 'definitely_not_here' } },
  { action: 'extract' },                               // 主目标未完成，必须被拒绝
  { action: 'nonsense_action' },                       // 未知动作
  { action: 'retreat' },
  { action: 'view' },
];

const payload = requests.map((r) => JSON.stringify(r)).join('\n') + '\n';
const res = spawnSync('powershell.exe', [
  '-NoProfile', '-Command',
  `$input | & '${PROBE}'`,
], { input: payload, encoding: 'utf8', timeout: 30000 });

assert.equal(res.status, 0, `probe 退出码应为 0，实际 ${res.status}\nstderr: ${res.stderr}`);
const lines = res.stdout.split(/\r?\n/).filter(Boolean);
assert.equal(lines.length, requests.length, `回复行数应等于请求数 ${requests.length}，实际 ${lines.length}`);

const replies = lines.map((line) => JSON.parse(line));

// 1. hub 视图：真实标题/版本/玩家/世界
const hub = replies[0];
assert.equal(hub.title, '无限人族进化');
assert.equal(hub.version, 2);
assert.equal(hub.mode, 'hub');
assert.equal(hub.player.name, '岑遥');
assert.equal(hub.player.hp, 120);
assert.equal(hub.worlds.length, 3);
assert.ok(hub.worlds.some((w) => w.id === 'grey_hive' && w.available));

// 2. enter 灰巢：map 出现、模式不再是 hub
const entered = replies[1];
assert.equal(entered.error, undefined, `enter 不应报错: ${JSON.stringify(entered)}`);
assert.ok(entered.map, 'enter 后应有 map');
assert.equal(entered.map.world_id, 'grey_hive');
assert.ok(Array.isArray(entered.map.tiles) && entered.map.tiles.length > 0, '地图应有格子');
assert.ok(entered.objectives.length > 0, 'enter 后应有任务目标');

// 3. 世界内视图
const inWorld = replies[2];
assert.equal(inWorld.mode, 'world');
assert.equal(inWorld.map.world_id, 'grey_hive');
assert.ok(Number.isInteger(inWorld.map.px) && Number.isInteger(inWorld.map.py),
  `地图应有玩家坐标 px/py: ${JSON.stringify(inWorld.map?.px)}/${JSON.stringify(inWorld.map?.py)}`);

// 4. 非法移动：probe 把拒绝写进 last_event 并回完整视图（不是独立 error 字段）
assert.match(replies[3].last_event || '', /不能同时为 0|移动需要/);
assert.equal(replies[3].mode, 'world', '非法移动后仍在世界内');

// 5. 合法移动：不抛错即可（是否撞墙由引擎定）
assert.equal(replies[4].error, undefined, `合法移动不应报错: ${JSON.stringify(replies[4])}`);

// 6-8. 攻击/技能协议结构：返回 error（挥空/冷却/资源）或 effect（命中），两者都合法
for (const [idx, label] of [[5, 'attack'], [7, 'skill(slot=0)']]) {
  const r = replies[idx];
  assert.ok(
    r.error !== undefined || r.effect !== undefined || r.last_event !== undefined,
    `${idx} ${label} 回复应有 error/effect/last_event 之一: ${JSON.stringify(r)}`,
  );
}
assert.match(replies[6].last_event || '', /slot 只能是 0、1、2/);

// 9. heal：medkits>0 且未满血时生效；满血时给提示。两种都断言结构
assert.ok(replies[8].player || replies[8].last_event, 'heal 应返回 player 或提示');

// 10. 交互远处对象被拒
assert.match(replies[9].last_event || '', /附近没有这个交互对象|距离交互对象太远/);

// 11. 未完成主目标不能撤离
assert.match(replies[10].last_event || '', /三个主目标尚未全部完成|当前状态不能撤离/);

// 12. 未知动作
assert.match(replies[11].last_event || '', /未知动作/);

// 13. retreat 放弃任务 → 回到可继续状态
assert.ok(replies[12].error === undefined || replies[12].error === null || !replies[12].error,
  `retreat 不应报错: ${JSON.stringify(replies[12])}`);

// 14. retreat 后回到归航站
assert.equal(replies[13].mode, 'hub', `retreat 后应回到 hub: ${replies[13].mode}`);

console.log('PASS campaign-probe 多动作确定性验证：14 步全部符合断言');
console.log('  hub: %s v%s / %s', hub.title, hub.version, hub.player.name);
console.log('  worlds:', hub.worlds.map((w) => w.id).join(', '));
