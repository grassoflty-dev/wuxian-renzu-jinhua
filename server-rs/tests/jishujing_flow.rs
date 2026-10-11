//! 《无限恐怖 · 弗莱迪归来》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 JISHUJING_SCENES 并入 scenes::scene()、把 jishujing_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_JISHUJING（id="jishujing"），保证 engine::goto / engine::choose 能解析
//! jj2_* 场景与 jj2_* 战斗（见 tools/design/jishujing_impl_log.md ★外部依赖）。
//! 测试只依赖本副本的 scenes_<slug> 表 + 全局 scenes::scene / scenes::fight_cfg，不碰 WorldData/find_world。
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::scenes_jishujing;

/// ① 核心场景全存在（开场 / 三层 hub / BOSS / 结算卡 / 死亡卡）
#[test]
fn jj2_scenes_exist() {
    assert!(scenes::scene("jj2_00").is_some(), "开场 jj2_00");
    assert!(scenes::scene("jj2_l1_hub").is_some(), "L1 hub");
    assert!(scenes::scene("jj2_l2_hub").is_some(), "L2 hub");
    assert!(scenes::scene("jj2_l3_hub").is_some(), "L3 hub");
    assert!(scenes::scene("jj2_boss").is_some(), "BOSS 铺垫");
    assert!(scenes::scene("jj2_boss_round").is_some(), "BOSS round");
    assert!(scenes::scene("jj2_end_flee").is_some(), "开放结局·挣脱");
    assert!(scenes::scene("jj2_end_sleep").is_some(), "开放结局·共眠");
    assert!(scenes::scene("jj2_end_share").is_some(), "开放结局·交给同伴");
    assert!(scenes::scene("jj2_42_card").is_some(), "结算卡");
    assert!(scenes::scene("jj2_50_death").is_some(), "死亡卡");
}

/// ② 战斗表完整：选择驱动 BOSS 梦魇弗莱迪 HP=210
#[test]
fn jj2_fights_exist() {
    assert!(scenes::fight_cfg("jj2_boss").is_some(), "jj2_boss fight 已分发");
    assert_eq!(scenes::fight_cfg("jj2_boss").unwrap().hp, 210, "弗莱迪 HP 应为 210");
    let all = scenes_jishujing::jishujing_figths();
    let ids: std::collections::HashSet<&str> = all.iter().map(|(k, _)| *k).collect();
    assert!(ids.contains("jj2_fight_l1"), "L1 影子战");
    assert!(ids.contains("jj2_fight_l2"), "L2 怪物战");
    assert!(ids.contains("jj2_fight_l3"), "L3 蒸汽鬼战");
    assert!(ids.contains("jj2_boss"), "梦魇弗莱迪");
    println!("FIGHTS OK · {} 场（BOSS HP={}）", all.len(), all.iter().find(|(k, _)| *k == "jj2_boss").map(|(_, c)| c.hp).unwrap());
}

/// ③ 自身闭合：jishujing_figths() 里的每个 fight id 都由全局 scenes::fight_cfg 分发闭环
#[test]
fn jj2_self_consistent() {
    for (id, _c) in scenes_jishujing::jishujing_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
    println!("SELF CONSISTENT OK · {} fight 闭环", scenes_jishujing::jishujing_figths().len());
}