//! 黄金示例副本 tests（确定性，不碰随机战斗）
use wuxian_horror_ch1::scenes;

#[test]
fn yinxiang_scenes_exist() {
    assert!(scenes::scene("yx_00").is_some());
    assert!(scenes::scene("yx_01").is_some());
}

#[test]
fn yinxiang_fights_exist() {
    assert!(scenes::fight_cfg("yx_boss").is_some());
}

#[test]
fn yinxiang_self_consistent() {
    for (id, _c) in wuxian_horror_ch1::scenes_yinxiang::yinxiang_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
}

/* =====================================================================
   测试补全（test2_ 前缀：调查点/道具 / 结算分支 / BOSS 战结算）
   ===================================================================== */
use wuxian_horror_ch1::engine;
use wuxian_horror_ch1::state::GameState;

fn t2_pick(st: &GameState, keyword: &str) -> i32 {
    let scene = scenes::scene(&st.scene_id).expect("scene");
    let visible: Vec<_> = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(st))).collect();
    for (i, c) in visible.iter().enumerate() {
        if c.label.contains(keyword) {
            return i as i32;
        }
    }
    panic!("scene {} 未找到含「{}」的选项；可见: {:?}",
        st.scene_id, keyword, visible.iter().map(|c| c.label).collect::<Vec<_>>());
}

fn t2_choose(st: &mut GameState, keyword: &str) {
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    let idx = t2_pick(st, keyword);
    engine::choose(st, idx, &mut deaths);
}

/// 补1：调查点/道具 —— 三处奇观调查置 flag 与道具、两位 NPC 对话链路，逐项断言。
#[test]
fn test2_investigation_points_and_items() {
    let mut st = GameState::new();
    st.hp = 200;
    st.san = 100;
    let base = st.points;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "yx_hub", &mut deaths);
    assert_eq!(st.scene_id, "yx_hub");

    // 外部甲板 · 损伤之窗：取「银之泪」道具 + yx_saw_tear flag +15 点
    t2_choose(&mut st, "登上外部甲板 · 损伤之窗");
    assert_eq!(st.scene_id, "yx_pt_hull");
    t2_choose(&mut st, "取下一枚结晶鳞片");
    assert!(st.flag("yx_saw_tear"), "应置 yx_saw_tear");
    assert!(st.inventory.iter().any(|i| i == "yx_souvenir_tear"), "应收得银之泪道具");
    assert_eq!(st.points, base + 15, "取银之泪应 +15 点");

    // 阵亡机师 · 无声纪念碑：置 yx_hero_read -5 San +20 点
    t2_choose(&mut st, "阅读阵亡机师");
    assert_eq!(st.scene_id, "yx_pt_hero");
    t2_choose(&mut st, "郑重读完这段告白");
    assert!(st.flag("yx_hero_read"), "应置 yx_hero_read");
    assert_eq!(st.san, 95, "读告白应 San-5");
    assert_eq!(st.points, base + 15 + 20, "读告白应 +20 点");

    // 主引擎舱 · 死去的火焰：置 yx_engine_core +15 点
    t2_choose(&mut st, "下至主引擎舱");
    assert_eq!(st.scene_id, "yx_pt_engine");
    t2_choose(&mut st, "记录这团冰晶之核");
    assert!(st.flag("yx_engine_core"), "应置 yx_engine_core");
    assert_eq!(st.points, base + 15 + 20 + 15, "记录引擎应 +15 点");

    // 维修师对话 → 遗愿：置 yx_warn_banzhang +15 点
    t2_choose(&mut st, "倾听维修师机械");
    assert_eq!(st.scene_id, "yx_n_mech");
    t2_choose(&mut st, "问它为何不离开");
    assert_eq!(st.scene_id, "yx_n_mech2");
    t2_choose(&mut st, "记下这个名字与那句忠告");
    assert!(st.flag("yx_warn_banzhang"), "应置 yx_warn_banzhang");
    assert_eq!(st.points, base + 15 + 20 + 15 + 10 + 15, "问询应 +10，记遗愿应再 +15 点");
}

/// 补2：结算分支 —— 三分支（看景 / 带纪念 / 停留）各自置 flag、点数规则与关键道具。
#[test]
fn test2_settlement_branches() {
    // 看景结局：置 yx_end_view；读过银之泪时经 PointsIfFlag 加 30
    let mut st = GameState::new();
    st.hp = 200;
    st.san = 100;
    st.set_flag("yx_saw_tear");
    engine::goto(&mut st, "yx_end_view", &mut vec![]);
    let p = st.points;
    t2_choose(&mut st, "回到主神空间");
    assert!(st.flag("yx_end_view"), "看景结局应置 yx_end_view");
    assert_eq!(st.points, p + 40 + 30, "看景结局 +40，读过银之泪再 +30");

    // 带纪念结局：置 yx_end_souv + 收 yx_souvenir_tear
    let mut st = GameState::new();
    st.hp = 200;
    engine::goto(&mut st, "yx_end_souv", &mut vec![]);
    let p = st.points;
    t2_choose(&mut st, "回到主神空间");
    assert!(st.flag("yx_end_souv"), "带纪念结局应置 yx_end_souv");
    assert_eq!(st.points, p + 40, "带纪念结局 +40");
    assert!(st.inventory.iter().any(|i| i == "yx_souvenir_tear"), "带纪念应持有银之泪");

    // 停留结局：置 yx_end_stay；读过英雄告白时经 PointsIfFlag 加 30
    let mut st = GameState::new();
    st.hp = 200;
    st.set_flag("yx_hero_read");
    engine::goto(&mut st, "yx_end_stay", &mut vec![]);
    let p = st.points;
    t2_choose(&mut st, "灯熄后回到主神空间");
    assert!(st.flag("yx_end_stay"), "停留结局应置 yx_end_stay");
    assert_eq!(st.points, p + 40 + 30, "停留结局 +40，读过告白再 +30");
    // 三分支收束到同一结算卡
    assert_eq!(st.scene_id, "yx_card", "结算分支应收束到 yx_card");
}

/// 补3：BOSS 战结算 —— 银色舰长（HP 250）重击压制后 +500 点 / sp_grade D / 战利品道具 / flag。
#[test]
fn test2_boss_victory_settlement() {
    let mut st = GameState::new();
    st.hp = 900;
    st.san = 100;
    st.world_id = "yinxiang".to_string();
    let base = st.points;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    // 迎战银色舰长（start_boss 初始化 yx_boss Fight HP250）
    engine::goto(&mut st, "yx_01", &mut deaths);
    t2_choose(&mut st, "迎战");
    assert_eq!(st.scene_id, "yx_round", "应进入 BOSS 回合");
    assert!(st.fight.is_some(), "应初始化 BOSS 战斗");

    // 重击（30 伤害）打满 250 → 约 9 轮，期间防御可免伤
    let mut guard = 0;
    while st.scene_id == "yx_round" && guard < 50 {
        guard += 1;
        if guard % 3 == 0 {
            let idx = t2_pick(&st, "防御");
            engine::choose(&mut st, idx, &mut deaths);
        } else {
            let idx = t2_pick(&st, "重击");
            engine::choose(&mut st, idx, &mut deaths);
        }
    }
    assert!(guard < 50, "BOSS 未在 guard 轮内击破");
    assert_eq!(st.scene_id, "yx_card", "击败 BOSS 应进结算卡");
    assert!(st.flag("yx_boss_down"), "应置 yx_boss_down");
    assert_eq!(st.sp_grade, Some('D'), "击杀 BOSS 应写 sp_grade=D");
    assert_eq!(st.points, base + 500, "击败 BOSS 应 +500 点");
    assert!(st.inventory.iter().any(|i| i == "yx_reward"), "应掉落战利品 yx_reward");
}
