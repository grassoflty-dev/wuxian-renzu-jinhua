//! 《无限恐怖 · 命运清单 · 第二端》任务世界 · 确定性用例。
//! 依赖主神线在合并阶段把 BAISUN_SCENES 并入 scenes::scene()、把 baisun_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_BAISUN（id="baisun"），保证 engine::goto / engine::choose 能解析 bs_* 场景
//! 与 bs_* 战斗（见 tools/design/baisun_impl_log.md ★外部依赖）。
//! 测试只依赖 scenes_baisun 表 + 全局 scenes::scene / scenes::fight_cfg；不触碰 WorldData / walkable / spawn
//! （那是主线合并后的职责）。用例全部确定性，不碰随机战斗。
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::scenes_baisun;

/// 开场 + 三层 hub 必须存在
#[test]
fn bs_scenes_exist() {
    assert!(scenes::scene("bs_00").is_some(), "开场 bs_00 应存在");
    assert!(scenes::scene("bs_l1_hub").is_some(), "L1 hub bs_l1_hub 应存在");
    assert!(scenes::scene("bs_l2_hub").is_some(), "L2 hub bs_l2_hub 应存在");
    assert!(scenes::scene("bs_l3_hub").is_some(), "L3 hub bs_l3_hub 应存在");
    // 结局相关场景也应可达
    assert!(scenes::scene("bs_settle").is_some());
    assert!(scenes::scene("bs_42_card").is_some());
    assert!(scenes::scene("bs_50_death").is_some());
    assert!(scenes::scene("bs_boss_round").is_some());
}

/// 死神·使者象征战：fight bs_boss 存在，HP=150
#[test]
fn bs_fights_exist() {
    assert!(scenes::fight_cfg("bs_boss").is_some(), "fight bs_boss 应存在");
    assert_eq!(scenes::fight_cfg("bs_boss").unwrap().hp, 150, "死神·使者 HP 应为 150");
}

/// 自洽：baisun_figths() 表内所有 fight id 都能被全局 fight_cfg 解析（分发闭环）
#[test]
fn bs_self_consistent() {
    for (id, _c) in scenes_baisun::baisun_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
}

/// 第四用例（可选）：三征兆齐备 → bs_fate_rewritten 的判定可通过 baisun_figths 与场景静态表间接校验。
/// 这里用确定性方式抽查规则流关键场景 id 均在表中（feature 只读，无状态副作用）。
#[test]
fn bs_rule_flow_scenes_present() {
    // 三条环境机关线对应的「征兆观测」与「机关致死/改命」入口场景都必须定义
    let required = [
        "bs_01_crane", "bs_01_slip", "bs_01_box", "bs_10_drop",   // L1 坠物线
        "bs_04_handrail", "bs_04_cabinet", "bs_04_sign", "bs_11_elev", // L2 电梯夹线
        "bs_07_projector", "bs_07_extinguisher", "bs_07_sprinkler", "bs_12_fire", // L3 火灾线
    ];
    for id in required {
        assert!(scenes::scene(id).is_some(), "规则流关键场景 {id} 应存在");
    }
    // 战斗表应恰好包含唯一象征战 bs_boss
    assert_eq!(scenes_baisun::baisun_figths().len(), 1, "baisun 战斗表应只有死神·使者 1 场");
}