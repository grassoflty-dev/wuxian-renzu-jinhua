//! 敌人战斗修饰数据层（combat §2.1）。
//! 仅数据铺装与查询 helper，不含任何战斗结算逻辑（结算由包 B 做）。
//! 全部数值「建议值·可调」，拷贝自 tools/design/* 设计文档。

use crate::defs::BattleMods;

// ============================================================================
// BATTLE_MODS · 敌人战斗修饰（combat §2.1；key by fight_id，旁附零侵入 FIGHTS）
// 初始行按 combat §2.1 各副本→训练词映射示例。
// ============================================================================
pub const BATTLE_MODS: &[(&'static str, BattleMods)] = &[
    ("b_thirst_king", BattleMods {
        weak_fire: 30, weak_electric: 30, armor: 0, regen: Some((15, 3)), aura: 0,
        post_kill: Some((0, "绿潮蚀伤")), waves: &[], no_dodge: false,
    }),
    ("balrog", BattleMods {
        weak_fire: 0, weak_electric: 0, armor: 0, regen: None, aura: 6,
        post_kill: None, waves: &[], no_dodge: false,
    }),
    ("b_kayako", BattleMods {
        weak_fire: 0, weak_electric: 0, armor: 0, regen: None, aura: 5,
        post_kill: None, waves: &[], no_dodge: true,
    }),
];

/// 按 fight_id 查战斗修饰
pub fn battle_mods(id: &str) -> Option<&'static BattleMods> {
    BATTLE_MODS.iter().find(|(fid, _)| *fid == id).map(|(_, m)| m)
}
