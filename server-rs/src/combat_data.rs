//! 战斗体系数据层（包 A）：修真境界 / 战斗内道具 静态表。
//! 基因锁 → gene.rs；血统 → bloodlines.rs；敌人战斗修饰 → enemies.rs；均在此 re-export。
//! 仅数据铺装与查询 helper，不含任何战斗结算逻辑（结算由包 B 做）。
//! 全部数值「建议值·可调」，拷贝自 tools/design/* 设计文档。

pub use crate::bloodlines::*;
pub use crate::enemies::*;
pub use crate::gene::*;

use crate::defs::{CultivationStageCfg, FightItemEffect, ItemEffect, ItemSub};
use crate::state::GameState;

// ============================================================================
// CULTIVATION_STAGES · 修真境界（cultivation §1.1）
// 0 未修真（凡体）；1 练气 ~ 7 合道；合道以上只演出不进 fight 数值
// ============================================================================
const SP_ZERO: crate::defs::StagePassive = crate::defs::StagePassive {
    dmg_reduce: 0, san_resist: 0, per_turn_qi: 0, atk_flat: 0, hp_max: 0,
    ignore_death: false, dodge_bonus: 0.0,
};

pub const CULTIVATION_STAGES: &[CultivationStageCfg] = &[
    CultivationStageCfg {
        stage: 1, name: "练气期", need_points: 3000, need_grade: Some('D'), prev: None,
        qi_max_cap: 120, hp_regen: 8, san_regen: 5,
        passive: crate::defs::StagePassive { san_resist: 3, ..SP_ZERO },
    },
    CultivationStageCfg {
        stage: 2, name: "筑基期", need_points: 7000, need_grade: Some('C'), prev: Some(1),
        qi_max_cap: 240, hp_regen: 12, san_regen: 8,
        passive: crate::defs::StagePassive { dmg_reduce: 4, ..SP_ZERO },
    },
    CultivationStageCfg {
        stage: 3, name: "金丹期", need_points: 15000, need_grade: Some('B'), prev: Some(2),
        qi_max_cap: 400, hp_regen: 18, san_regen: 0,
        passive: crate::defs::StagePassive { per_turn_qi: 4, ..SP_ZERO },
    },
    CultivationStageCfg {
        stage: 4, name: "元婴期", need_points: 30000, need_grade: Some('A'), prev: Some(3),
        qi_max_cap: 600, hp_regen: 0, san_regen: 0,
        passive: SP_ZERO,
    },
    CultivationStageCfg {
        stage: 5, name: "化神期", need_points: 55000, need_grade: Some('A'), prev: Some(4),
        qi_max_cap: 850, hp_regen: 30, san_regen: 0,
        passive: crate::defs::StagePassive { ignore_death: true, ..SP_ZERO },
    },
    CultivationStageCfg {
        stage: 6, name: "返虚期", need_points: 90000, need_grade: Some('S'), prev: Some(5),
        qi_max_cap: 1100, hp_regen: 0, san_regen: 0,
        passive: crate::defs::StagePassive { san_resist: 2, ..SP_ZERO },
    },
    CultivationStageCfg {
        stage: 7, name: "合道期（顶点）", need_points: 140000, need_grade: Some('S'), prev: Some(6),
        qi_max_cap: 1400, hp_regen: 100, san_regen: 100,
        passive: SP_ZERO,
    },
];

// ============================================================================
// FIGHT_ITEMS · 战斗内道具白名单（combat §2.3 方案 A；id → 效果）
// ============================================================================
pub const FIGHT_ITEMS: &[(&'static str, FightItemEffect)] = &[
    ("item_medkit", FightItemEffect { kind: ItemSub::Heal, effect: ItemEffect::Heal(50) }),
    ("item_bandage", FightItemEffect { kind: ItemSub::Heal, effect: ItemEffect::Heal(18) }),
    ("item_sedative", FightItemEffect { kind: ItemSub::San, effect: ItemEffect::San(20) }),
    ("item_bottle_water", FightItemEffect {
        kind: ItemSub::Heal, effect: ItemEffect::Heal(8),
    }),
    ("item_holy_water", FightItemEffect {
        kind: ItemSub::Throw, effect: ItemEffect::Throw { dmg_over_time: None, weak: Some("holy"), flat_dmg: 0 },
    }),
    ("item_silver_bullet", FightItemEffect {
        kind: ItemSub::Throw, effect: ItemEffect::Throw { dmg_over_time: None, weak: Some("silver"), flat_dmg: 0 },
    }),
    ("item_torch", FightItemEffect {
        kind: ItemSub::Throw, effect: ItemEffect::Throw { dmg_over_time: Some((0, 3)), weak: Some("fire"), flat_dmg: 0 },
    }),
    ("item_lure", FightItemEffect {
        kind: ItemSub::Throw, effect: ItemEffect::Throw { dmg_over_time: None, weak: None, flat_dmg: 60 },
    }),
    ("item_grenade", FightItemEffect {
        kind: ItemSub::Throw, effect: ItemEffect::Throw { dmg_over_time: None, weak: None, flat_dmg: 70 },
    }),
    ("item_quzhen_fu", FightItemEffect {
        kind: ItemSub::CharmProt, effect: ItemEffect::Charm { immune_death: true, cure_debuff: false },
    }),
    ("item_jiezhou_fu", FightItemEffect {
        kind: ItemSub::CharmAnti, effect: ItemEffect::Charm { immune_death: false, cure_debuff: true },
    }),
    ("item_antidote", FightItemEffect {
        kind: ItemSub::CharmAnti, effect: ItemEffect::Charm { immune_death: false, cure_debuff: true },
    }),
];

/// 按修真境界阶查表
pub fn cultivation_stage_cfg(n: u8) -> Option<&'static CultivationStageCfg> {
    CULTIVATION_STAGES.iter().find(|c| c.stage == n)
}

/// 当前境界修真 qi_max 天棚档（未修真 0）
pub fn qi_max_cap_of(st: &GameState) -> i32 {
    cultivation_stage_cfg(st.cultivation_stage).map(|c| c.qi_max_cap).unwrap_or(0)
}
