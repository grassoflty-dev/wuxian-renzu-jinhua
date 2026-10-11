//! 《无限恐怖 · 黑珍珠》任务世界 · 确定性集成测试（不碰随机战斗）。
//! 依赖主神线在合并阶段把 JIALEBI_SCENES 并入 scenes::scene()、把 jialebi_figths() 并入 scenes::fight_cfg()，
//! 并在 worlds/mod.rs 注册 WORLD_JIALEBI（id="jialebi"）；本测试只验证：
//!   ① 关键场景 id 已分发到 scenes::scene()
//!   ② BOSS jb_boss（HP 220）与相关战斗已分发到 scenes::fight_cfg()
//!   ③ jialebi_figths() 内的所有 fight id 都真正可被 scenes::fight_cfg() 解析（自洽闭环）
//! （见 tools/design/jialebi_impl_log.md ★外部依赖。）
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::scenes_jialebi;

/* ---------------- ① 关键场景存在 ---------------- */
#[test]
fn jialebi_scenes_exist() {
    assert!(scenes::scene("jb_00").is_some(), "开场 jb_00 应存在");
    assert!(scenes::scene("jb_l1_hub").is_some(), "L1 hub 应存在");
    assert!(scenes::scene("jb_l2_hub").is_some(), "L2 hub 应存在");
    assert!(scenes::scene("jb_l3_hub").is_some(), "L3 hub 应存在");
    assert!(scenes::scene("jb_boss_area").is_some(), "BOSS 铺垫应存在");
    assert!(scenes::scene("jb_boss_round").is_some(), "BOSS round 应存在");
    assert!(scenes::scene("jb_boss_win").is_some(), "BOSS 胜利应存在");
    assert!(scenes::scene("jb_ending").is_some(), "开放结局应存在");
    assert!(scenes::scene("jb_42_card").is_some(), "结算卡应存在");
    assert!(scenes::scene("jb_50_death").is_some(), "BOSS 死亡卡应存在");
    println!("SCENES EXIST OK · 10 个关键场景已分发");
}

/* ---------------- ② 战斗存在：BOSS HP 220 ---------------- */
#[test]
fn jialebi_fights_exist() {
    assert!(scenes::fight_cfg("jb_boss").is_some(), "BOSS jb_boss 应可查");
    let boss = scenes::fight_cfg("jb_boss").expect("jb_boss 已并入 fight_cfg");
    assert_eq!(boss.hp, 220, "亡灵船长·巴博萨 HP 应为 220");
    for id in ["jb_fight_l1", "jb_fight_l2", "jb_fight_l3"] {
        assert!(scenes::fight_cfg(id).is_some(), "小怪战 {id} 应可查");
    }
    println!("FIGHTS EXIST OK · BOSS HP={} l1/l2/l3 小怪战均分发", boss.hp);
}

/* ---------------- ③ 自洽：本副本战斗表内所有 fight id 可分发给 scenes::fight_cfg ---------------- */
#[test]
fn jialebi_self_consistent() {
    for (id, _c) in scenes_jialebi::jialebi_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
    let n = scenes_jialebi::jialebi_figths().len();
    println!("SELF-CONSISTENT OK · 共 {n} 场战斗全部闭环");
}