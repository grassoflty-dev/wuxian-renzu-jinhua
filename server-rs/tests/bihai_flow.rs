//! 《无限恐怖 · 深海阴影》集成测试（确定性用例，不碰随机战斗）。
//! 依赖主神线在合并阶段把 BIHAI_SCENES 并入 scenes::scene()、把 bihai_figths() 并入 scenes::fight_cfg()，
//! 并在 lib.rs 注册 `pub mod scenes_bihai;`、worlds/mod.rs 注册 WORLD_BIHAI（见 tools/design/bihai_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① bh_scenes_exist：开场/boss/结局/结算卡/死亡卡 等关键场景都在。
//!   ② bh_fights_exist：选择驱动 BOSS「深渊邪物」HP==230。
//!   ③ bh_self_consistent：bihai_figths() 每个 fight 都能经 scenes::fight_cfg() 分发闭环。
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::scenes_bihai;

#[test]
fn bh_scenes_exist() {
    assert!(scenes::scene("bh_00").is_some());
    assert!(scenes::scene("bh_l1_hub").is_some());
    assert!(scenes::scene("bh_l2_hub").is_some());
    assert!(scenes::scene("bh_l3_hub").is_some());
    assert!(scenes::scene("bh_boss").is_some());
    assert!(scenes::scene("bh_boss_round").is_some());
    assert!(scenes::scene("bh_end_choice").is_some());
    assert!(scenes::scene("bh_win_escape").is_some());
    assert!(scenes::scene("bh_win_sacrifice").is_some());
    assert!(scenes::scene("bh_win_assimilate").is_some());
    assert!(scenes::scene("bh_42_card").is_some());
    assert!(scenes::scene("bh_50_death").is_some());
}

#[test]
fn bh_fights_exist() {
    assert!(scenes::fight_cfg("bh_boss").is_some());
    assert!(scenes::fight_cfg("bh_fight_l1").is_some());
    assert!(scenes::fight_cfg("bh_fight_l2").is_some());
    assert!(scenes::fight_cfg("bh_fight_l3").is_some());
    // 选择驱动 BOSS「深渊邪物」HP 230
    assert_eq!(scenes::fight_cfg("bh_boss").unwrap().hp, 230);
}

#[test]
fn bh_self_consistent() {
    for (id, _c) in scenes_bihai::bihai_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
}