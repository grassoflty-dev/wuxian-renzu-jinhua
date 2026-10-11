//! 动态难度缩放（design: tools/design/dynamic_scaling_design.md）
//! 副本敌人强度 = 主角当前强度 × 副本难度系数。
//!
//! 原则：
//! - FightCfg 表数值 = 基准值（难度系数 1.0 时的数值），**不改表**，只在实例化 Fight 时缩放。
//! - 主角强度因子 P 为**软上限**（对数衰减，见 `power_factor`）：越强增长越慢但不封死。
//! - **奖励超额**：reward 用超额难度系数（`difficulty_scale_excess`），难度越高回报率相对强度越高。
//! - **固定难度挑战关**：`WorldData.fixed_difficulty == true` 的世界，其战斗强度不随主角缩放
//!   （绝对强度锚点，验证"变强"），reward 也保留固定/超额梯度。
//! - `st.scaling_enabled == false` 时 `fight_scale` 恒 1.0（测试安全阀，保证既有 flow 数值回归全绿）。
//! - San 惩罚 / 环境伤害不缩放（场景机制，非敌人强度）。

use crate::defs::FightCfg;
use crate::state::{Fight, GameState};

/// 主角当前武器攻击贡献：现武器伤害中值 `(dmg.0+dmg.1)/2`；
/// 用户序：新装武器格（equipment.weapon）→ 旧 Weapon 枚举 → 无武器缺省 6。
/// 强化 +N（item_equipment_system_design.md §1.1/§4.3）：dmg 每级下限 +2、上限 +3，
/// 故中值逐级 +2.5。此处把 enhance 计入中值（向上取保护整数）。
pub fn weapon_atk(st: &GameState) -> i32 {
    if let Some(ws) = &st.equipment.weapon {
        if let Some(w) = crate::items_data::weapon_def(&ws.id) {
            let lo = w.dmg.0 + (ws.enhance as i32) * 2;
            let hi = w.dmg.1 + (ws.enhance as i32) * 3;
            return (lo + hi) / 2;
        }
    }
    if let Some(w) = st.weapon {
        let (a, b) = w.dmg();
        return (a + b) / 2;
    }
    6
}

/// 装备常驻攻击追加总和（护甲 + 饰品 + 法宝三格；无装备 0）。
pub fn equipped_atk_flat(st: &GameState) -> i32 {
    let mut a = 0;
    let eq = &st.equipment;
    if let Some(armor) = &eq.armor {
        if let Some(g) = crate::items_data::gear_def(armor) { a += g.atk_flat; }
    }
    if let Some(acc) = &eq.accessory {
        if let Some(g) = crate::items_data::gear_def(acc) { a += g.atk_flat; }
    }
    for t in eq.treasure.iter().flatten() {
        if let Some(td) = crate::items_data::treasure_def(t) { a += td.atk_flat; }
    }
    a
}

/// 主角强度（§二公式，单调增长；不新增字段）：
/// hp/20 + 武器攻击 + 基因锁阶×8 + 修真境界×6 + 装备攻击 + 技能数(上限 +20)。
pub fn power(st: &GameState) -> i32 {
    (st.hp as f32 / 20.0) as i32
        + weapon_atk(st)
        + (st.gene_stage as i32) * 8
        + (st.cultivation_stage as i32) * 6
        + equipped_atk_flat(st)
        + (st.skills.len() as i32).min(20)
}

/// 主角强度因子 P（软上限，对数衰减，任务一）：
/// `p = power / 25.0`；`P = (1.0 + p).log2()`，保底 0.6，**无硬上限 4.0**。
/// - power=25 → 1.0、100 → 2.32、1000 → 5.36、10000 → 8.64（log2 衰减，越强增长越慢但不封死）。
pub fn power_factor(st: &GameState) -> f32 {
    let p = power(st) as f32 / 25.0;
    (1.0 + p).log2().max(0.6)
}

/// 副本难度系数 D（战斗 hp/dmg/rage 用，§三）：1→0.8, 2→1.0, 3→1.3, 4→1.6, 5→2.0；其余回退 1.0。
pub fn difficulty_scale(d: usize) -> f32 {
    match d {
        1 => 0.8,
        2 => 1.0,
        3 => 1.3,
        4 => 1.6,
        5 => 2.0,
        _ => 1.0,
    }
}

/// 副本难度**超额**系数 Dr（奖励 reward 用，任务二）：
/// 难度 1→0.8、2→1.0、3→1.4、4→1.9、5→2.6（越高回报率相对强度越高，鼓励越级挑战）；其余回退 1.0。
pub fn difficulty_scale_excess(d: usize) -> f32 {
    match d {
        1 => 0.8,
        2 => 1.0,
        3 => 1.4,
        4 => 1.9,
        5 => 2.6,
        _ => 1.0,
    }
}

/// 固定难度挑战关的绝对强度锚点因子（任务三）：
/// fixed_difficulty 世界不随主角缩放，强度 = 纯以副本绝对难度系数为基准（低强度主角打不过、高强度主角可碾压）。
pub const FIXED_SCALE_FACTOR: f32 = 1.0;

/// 当前世界的 difficulty（从 st.world_id → worlds::find_world → .difficulty）。
/// world 未注册/未知时回退 usize::MAX 边界外的难度基准 1.0（difficulty_scale 的 _ 分支）。
fn current_difficulty(st: &GameState) -> usize {
    let d = crate::worlds::find_world(&st.world_id).map(|w| w.difficulty as usize).unwrap_or(0);
    d
}

/// 当前世界是否为固定难度挑战关（fixed_difficulty）。world 未注册视为 false。
fn current_fixed(st: &GameState) -> bool {
    crate::worlds::find_world(&st.world_id).map(|w| w.fixed_difficulty).unwrap_or(false)
}

/// 基础战斗缩放倍率（hp/dmg/rage 用）= D(副本难度系数) × P(主角强度因子)。
/// `fixed_difficulty` 世界用 FIXED_SCALE_FACTOR 替代 P（不随主角缩放）。
/// 关闭缩放开关（st.scaling_enabled == false）时恒 1.0（测试安全阀）。
fn base_scale(st: &GameState) -> f32 {
    if !st.scaling_enabled {
        return 1.0;
    }
    let d = current_difficulty(st);
    let fac = if current_fixed(st) { FIXED_SCALE_FACTOR } else { power_factor(st) };
    difficulty_scale(d) * fac
}

/// 奖励缩放倍率（reward 用）= 超额难度系数 Dr × P（任务二：难度越高回报率越高）。
/// reward 至少 ≥1 保底（见 scaled_fight）；`fixed_difficulty` 世界同样不随主角缩放但保留超额梯度。
fn reward_scale(st: &GameState) -> f32 {
    if !st.scaling_enabled {
        return 1.0;
    }
    let d = current_difficulty(st);
    let fac = if current_fixed(st) { FIXED_SCALE_FACTOR } else { power_factor(st) };
    difficulty_scale_excess(d) * fac
}

/// 整体缩放倍率 scale = D(副本难度系数) × P(主角强度因子)。
/// 关闭缩放开关（st.scaling_enabled == false）时恒 1.0（测试安全阀）。
pub fn fight_scale(st: &GameState) -> f32 {
    base_scale(st)
}

/// 按 scale 缩放 FightCfg → 实例化 Fight。
/// - id 由调用方给定（FightCfg 无 id 字段，构造点各自有权威 id）
/// - hp/max_hp = round(cfg.hp × hp_scale)
/// - dmg = (round(cfg.dmg.0 × hp_scale), round(cfg.dmg.1 × hp_scale))
/// - reward = max(1, round(cfg.reward × reward_scale))（reward 用超额系数，至少 ≥1 保底）
/// - rage_at = cfg.rage_at.map(|r| round(r × hp_scale))（与 hp 同源缩放）
/// - hp/dmg/rage_at 用 `base_scale`，reward 用 `reward_scale`（任务二奖励超额）。
/// - 其余字段照 cfg 填（name/reward_why/intro 不变）；pending_log 由调用方给定。
/// - 若 fight_id 命中楚轩战术提示表（`tactics::chuxuan_tactic`），在 pending_log 末尾追加一条楚轩分析。
pub fn scaled_fight(id: &str, cfg: &FightCfg, st: &GameState, pending_log: Vec<String>) -> Fight {
    let hs = base_scale(st);
    let rs = reward_scale(st);
    let hp = (cfg.hp as f32 * hs).round() as i32;
    let dmg = ((cfg.dmg.0 as f32 * hs).round() as i32, (cfg.dmg.1 as f32 * hs).round() as i32);
    let reward = (cfg.reward as f32 * rs).round() as i32;
    let reward = reward.max(1);
    let rage_at = cfg.rage_at.map(|r| (r as f32 * hs).round() as i32);
    // 楚轩智谋提示：命中 fight_id 则在敌人登场文案之后追加一条楚轩的战术分析。
    // 放在实例化 Fight 时做，天然实现同一场战斗只加一次（幂等），不引入存档字段、不随机。
    let mut pending_log = pending_log;
    if let Some(tactic) = crate::tactics::chuxuan_tactic_line(id) {
        pending_log.push(tactic);
    }
    Fight {
        id: id.to_string(),
        name: cfg.name.to_string(),
        hp,
        max_hp: hp,
        dmg,
        reward,
        reward_why: cfg.reward_why.to_string(),
        raged: false,
        rage_at,
        guard_turn: false,
        pending_log,
    }
}