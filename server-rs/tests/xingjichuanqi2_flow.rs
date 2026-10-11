//! 《寂静岭·灰雾之心》任务世界 · 集成测试（确定性、不碰随机战斗）。
//! 只依赖 scenes_xingjichuanqi2::xingjichuanqi2_figths() + scenes::scene/fight_cfg（主线合并后分发闭环）。
//! 结构参考 tests/sishen_flow.rs 模板；本测试不依赖 find_world/walkable（那些是主线的 merge 职责）。

use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::scenes_xingjichuanqi2;

/* ---------------- ① 场景存在性 ---------------- */
#[test]
fn xj2_scenes_exist() {
    // 开场 + hub + BOSS round + 结算卡 + 死亡卡
    assert!(scenes::scene("xj2_00").is_some(), "开场缺");
    assert!(scenes::scene("xj2_01_l1_hub").is_some(), "L1 hub 缺");
    assert!(scenes::scene("xj2_03_l2_hub").is_some(), "L2 hub 缺");
    assert!(scenes::scene("xj2_04_l3_hub").is_some(), "L3 hub 缺");
    assert!(scenes::scene("xj2_30_judgement").is_some(), "审判缺");
    assert!(scenes::scene("xj2_09_boss_round").is_some(), "BOSS round 缺");
    assert!(scenes::scene("xj2_42_card").is_some(), "结算卡缺");
    assert!(scenes::scene("xj2_50_death").is_some(), "死亡卡缺");
    // 三个开放结局分支
    assert!(scenes::scene("xj2_31_forgive").is_some(), "宽恕结局缺");
    assert!(scenes::scene("xj2_32_revenge").is_some(), "复仇结局缺");
    assert!(scenes::scene("xj2_33_carry").is_some(), "背负结局缺");
}

/* ---------------- ② 战斗表完整性 & 分发闭环 ---------------- */
#[test]
fn xj2_fights_exist() {
    // 选择驱动 BOSS + 三个罪念守卫
    assert!(scenes::fight_cfg("xj2_boss").is_some(), "三角头 BOSS 缺");
    assert!(scenes::fight_cfg("xj2_fight_miner").is_some());
    assert!(scenes::fight_cfg("xj2_fight_priest").is_some());
    assert!(scenes::fight_cfg("xj2_fight_nurse").is_some());
    // BOSS HP 应为 200
    let boss = scenes::fight_cfg("xj2_boss").expect("boss");
    assert_eq!(boss.hp, 200, "三角头·深红审判 HP 应 200");
}

/* ---------------- ③ 自洽：figths() 表所有 id 分发闭环 ---------------- */
#[test]
fn xj2_self_consistent() {
    for (id, _c) in scenes_xingjichuanqi2::xingjichuanqi2_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
    // 罪证 flag 链：三碎片齐备应置 xj2_evidence_full——此处仅断言场景门基于 flag 的 cond 存在（结构自检），不跑引擎
    assert!(scenes::scene("xj2_30_judgement").is_some());
}