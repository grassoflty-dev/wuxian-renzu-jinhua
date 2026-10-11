//! 《无限恐怖·寄生前夜》任务世界 · 3 个确定性集成用例。
//! 只依赖本副本自己的 scenes 文件（scenes_jishengqianye::jishengqianye_figths()）+ 全局 scenes::scene/fight_cfg（主线合并后分发）。
//! 不依赖 find_world / walkable / WorldData / 随机战斗（见 tools/design/jishengqianye_impl_log.md ★外部依赖）。
use wuxian_horror_ch1::scenes;

/// ① 关键场景可被 scenes::scene 解析（主线把 JISHENGQIANYE_SCENES 并入后成立）。
#[test]
fn jishengqianye_scenes_exist() {
    assert!(scenes::scene("js_00").is_some(), "开场场景 js_00 应可分发");
    assert!(scenes::scene("js_f3_bossgate").is_some(), "深室决战入口应可分发");
    assert!(scenes::scene("js_boss_round").is_some(), "BOSS 回合场景应可分发");
    assert!(scenes::scene("js_card").is_some(), "结算卡应可分发");
    assert!(scenes::scene("js_death").is_some(), "死亡卡应可分发");
}

/// ② BOSS 战斗表可被 scenes::fight_cfg 解析，且数值符合设计（HP 200 / 霸者阶段增回阈 80 / 奖励 500）。
#[test]
fn jishengqianye_boss_cfg() {
    let cfg = scenes::fight_cfg("wc_jq_boss").expect("线粒体聚合体战斗表应可分发");
    assert_eq!(cfg.hp, 200, "聚合体 HP 应为 200");
    assert_eq!(cfg.dmg, (16, 26), "聚合体受击范围 (16,26)");
    assert_eq!(cfg.reward, 500, "聚合体奖励 500");
    assert_eq!(cfg.rage_at, Some(80), "霸者阶段（增回）触发阈 80");
    assert!(scenes::fight_cfg("wc_jq_intro_prop").is_some(), "开场傀儡战斗应可分发");
    assert!(scenes::fight_cfg("wc_jq_hound").is_some(), "线粒体猎犬战斗应可分发");
}

/// ③ 分发闭环：表中每个 fight id 都应能经 scenes::fight_cfg 查回（自一致）。
#[test]
fn jishengqianye_self_consistent() {
    let fights = wuxian_horror_ch1::scenes_jishengqianye::jishengqianye_figths();
    assert!(!fights.is_empty(), "战斗表非空");
    for (id, _c) in fights {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
    // 场景集合：任一 Route::To / fight 引用应指向自有的 js_* 场景或战斗表（烟雾检查主链完整性）
    let scenes_list = wuxian_horror_ch1::scenes_jishengqianye::JISHENGQIANYE_SCENES;
    assert!(scenes_list.iter().any(|s| s.id == "js_00"), "应有开场 js_00");
    assert!(scenes_list.iter().any(|s| s.id == "js_boss_round"), "应有 BOSS 回合");
    assert!(scenes_list.iter().any(|s| s.id == "js_card"), "应有结算卡");
    println!("SCENES={} FIGHTS={}", scenes_list.len(), fights.len());
}