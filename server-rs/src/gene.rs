//! 基因锁数据层（基因锁 §1.1~1.4）。
//! 仅数据铺装与查询 helper，不含任何战斗结算逻辑（结算由包 B 做）。
//! 全部数值「建议值·可调」，拷贝自 tools/design/* 设计文档。

use crate::defs::GeneStageCfg;
use crate::state::GameState;

// ============================================================================
// GENE_STAGES · 基因锁各阶（gene_lock §1.1~1.4，口径与 combat §1.1 一致）
// 一阶D/二阶B/三阶A/四阶A顶；濒死 horn：hp<=阈值额外叠（§1.1~1.4 建议值）
// ============================================================================
pub const GENE_STAGES: &[GeneStageCfg] = &[
    GeneStageCfg {
        stage: 1,
        name: "战斗本能",
        sp_grade: Some('D'),
        atk: (6, 12),
        dodge: 0.15,
        dmg_reduce: 4,
        finisher_bonus: 0,
        hp_low_threshold: 30,
        awakening_cost_san: 8,
    },
    GeneStageCfg {
        stage: 2,
        name: "肌肉掌控",
        sp_grade: Some('B'),
        atk: (12, 20),
        dodge: 0.18,
        dmg_reduce: 8,
        finisher_bonus: 15,
        hp_low_threshold: 25,
        awakening_cost_san: 8,
    },
    GeneStageCfg {
        stage: 3,
        name: "精神领域",
        sp_grade: Some('A'),
        atk: (20, 30),
        dodge: 0.20,
        dmg_reduce: 12,
        finisher_bonus: 25,
        hp_low_threshold: 20,
        awakening_cost_san: 10,
    },
    GeneStageCfg {
        stage: 4,
        name: "规则之门",
        sp_grade: Some('A'), // 顶
        atk: (30, 40),
        dodge: 0.22,
        dmg_reduce: 16,
        finisher_bonus: 40,
        hp_low_threshold: 15,
        awakening_cost_san: 15,
    },
];

/// 读当前基因锁阶：max(gene_stage, gene_lock 布尔视图)。engine/scenes 唯一入口。
pub fn gene_stage_of(st: &GameState) -> u8 {
    if st.gene_lock && st.gene_stage < 1 { 1 } else { st.gene_stage }
}

/// 写基因锁阶：gene_stage=n 且 gene_lock=(n>=1)。engine/scenes 唯一写入口。
pub fn set_gene_stage(st: &mut GameState, n: u8) {
    st.gene_stage = n;
    st.gene_lock = n >= 1;
}

/// 基因锁各阶查询：按 gene_stage_of(st) 查表
pub fn gene_stage_cfg(st: &GameState) -> Option<&'static GeneStageCfg> {
    GENE_STAGES.iter().find(|c| c.stage == gene_stage_of(st))
}
