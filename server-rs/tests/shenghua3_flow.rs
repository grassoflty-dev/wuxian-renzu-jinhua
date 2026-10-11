//! 《生化危机·伊芙琳·浣熊市地下》任务世界 · 集成测试（确定性、不碰随机战斗）。
//! 只依赖 scenes_shenghua3::shenghua3_figths() + scenes::scene/fight_cfg（主线合并后分发闭环）。
//! 结构参考 tools/design/impl_template.md 三节模板；不依赖 find_world/walkable（主线 merge 职责）。

use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::scenes_shenghua3;

/* ---------------- ① 场景存在性 ---------------- */
#[test]
fn sh3_scenes_exist() {
    // 开场 + 三层 hub + BOSS round + 第二选择 + 结算卡 + 死亡卡
    assert!(scenes::scene("sh3_00").is_some(), "开场缺");
    assert!(scenes::scene("sh3_01_l1_hub").is_some(), "L1 hub 缺");
    assert!(scenes::scene("sh3_03_l2_hub").is_some(), "L2 hub 缺");
    assert!(scenes::scene("sh3_04_l3_hub").is_some(), "L3 hub 缺");
    assert!(scenes::scene("sh3_30_final_choice").is_some(), "第二选择缺");
    assert!(scenes::scene("sh3_09_boss_round").is_some(), "BOSS round 缺");
    assert!(scenes::scene("sh3_42_card").is_some(), "结算卡缺");
    assert!(scenes::scene("sh3_50_death").is_some(), "死亡卡缺");
    // 三个开放结局分支
    assert!(scenes::scene("sh3_31_spare").is_some(), "宽赦结局缺");
    assert!(scenes::scene("sh3_32_feed").is_some(), "喂给结局缺");
    assert!(scenes::scene("sh3_33_blowup").is_some(), "引爆结局缺");
}

/* ---------------- ② 战斗表完整性 & 分发闭环 ---------------- */
#[test]
fn sh3_fights_exist() {
    assert!(scenes::fight_cfg("sh3_boss").is_some(), "追踪者 BOSS 缺");
    assert!(scenes::fight_cfg("sh3_fight_l1").is_some());
    assert!(scenes::fight_cfg("sh3_fight_l2").is_some());
    assert!(scenes::fight_cfg("sh3_fight_l3").is_some());
    let boss = scenes::fight_cfg("sh3_boss").expect("boss");
    assert_eq!(boss.hp, 260, "追踪者·复仇女神 HP 应 260");
}

/* ---------------- ③ 自洽：figths() 表所有 id 分发闭环 ---------------- */
#[test]
fn sh3_self_consistent() {
    for (id, _c) in scenes_shenghua3::shenghua3_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
    // 证据 flag 链关联的门禁场景存在（结构自检）
    assert!(scenes::scene("sh3_10_sewage_ok").is_some());
    assert!(scenes::scene("sh3_40_death_sewage").is_some());
}

/* =====================================================================
   测试补全（test2_ 前缀：调查点置 flag / 环境机关门禁 / BOSS 战结算）
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

/// 补1：调查点置 flag —— 取走三份证据（水样 / 信条 / 孵化日志），
/// 每取一份置对应 sh3_ev_* flag；集齐触发 sh3_evidence_full +200 点，并解锁对应门禁 flag。
#[test]
fn test2_investigation_collects_evidence_flags_and_unlocks() {
    let mut st = GameState::new();
    st.hp = 500;
    st.san = 100;
    let base = st.points;

    // L1 水样
    engine::goto(&mut st, "sh3_01_l1_hub", &mut vec![]);
    t2_choose(&mut st, "地下水闸门");
    assert_eq!(st.scene_id, "sh3_02_gate");
    t2_choose(&mut st, "取走水样");
    assert!(st.flag("sh3_ev_sample"), "应置 sh3_ev_sample");
    assert_eq!(st.scene_id, "sh3_01_l1_hub", "取完样本回 L1 hub");

    // L2 信条（兼解锁门禁 sh3_creed_read）
    engine::goto(&mut st, "sh3_03_l2_hub", &mut vec![]);
    t2_choose(&mut st, "审讯室门");
    assert_eq!(st.scene_id, "sh3_03_cell");
    t2_choose(&mut st, "取走伞公司信条");
    assert!(st.flag("sh3_ev_creed"), "应置 sh3_ev_creed");
    assert!(st.flag("sh3_creed_read"), "读通信条应置 sh3_creed_read（解 L3 井道门禁）");
    assert_eq!(st.scene_id, "sh3_03_l2_hub");

    // L3 孵化日志（置 sh3_log_read）
    engine::goto(&mut st, "sh3_04_l3_hub", &mut vec![]);
    t2_choose(&mut st, "停尸间");
    assert_eq!(st.scene_id, "sh3_04_morgue");
    t2_choose(&mut st, "取走孵化日志");
    assert!(st.flag("sh3_ev_log"), "应置 sh3_ev_log");
    assert!(st.flag("sh3_log_read"), "读孵化日志应置 sh3_log_read");

    // 三份集齐 → evidence_full +200
    assert!(st.flag("sh3_evidence_full"), "三份证据集齐应置 sh3_evidence_full");
    assert_eq!(st.points, base + 200, "集齐证据应 +200 点");

    // 门禁可达性：井道（需信条）与孵化室终端门在集齐后可进
    engine::goto(&mut st, "sh3_03_l2_hub", &mut vec![]);
    t2_choose(&mut st, "沿井道下楼");
    assert_eq!(st.scene_id, "sh3_03_gateway");
    t2_choose(&mut st, "通过气闸进实验室");
    assert_eq!(st.scene_id, "sh3_04_l3_hub", "读通信条后应可通过井道门禁到 L3");
}

/// 补2：环境机关 / 门禁 —— 未查水质踩污水渠 → 生化感染死亡卡；先查证则安然 +30 点并置安全 flag。
#[test]
fn test2_environment_mechanism_sewage_death_or_safe() {
    // 未查水质 → 感染死亡
    let mut st = GameState::new();
    st.hp = 500;
    st.san = 100;
    st.world_id = "shenghua3".to_string();
    engine::goto(&mut st, "sh3_10_sewage", &mut vec![]);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    let idx = t2_pick(&st, "污染防治");
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(st.scene_id, "sh3_40_death_sewage", "未查水质应跳生化感染死亡卡");
    assert_eq!(st.san, 90, "进污水渠先扣 San-10");

    // 先查证（sh3_water_checked）→ 安然过渠：+30 点 + 置 sh3_sign_sew_ok
    let mut st = GameState::new();
    st.hp = 500;
    st.san = 100;
    st.set_flag("sh3_water_checked");
    st.world_id = "shenghua3".to_string();
    st.points = 40;
    engine::goto(&mut st, "sh3_10_sewage", &mut vec![]);
    // zone_sewage：查证则 route_sewage_ok（+30）
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    let idx = t2_pick(&st, "污染防治");
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(st.scene_id, "sh3_01_l1_hub", "查证后应安然返回 L1 hub");
    assert!(st.flag("sh3_sign_sew_ok"), "查证后应置 sh3_sign_sew_ok");
    assert_eq!(st.points, 40 + 30, "安然过渠应 +30 点");
}

/// 补3：BOSS 战结算 —— 追踪者·复仇女神击败后进第二选择，再选宽赦结局 → 结算卡，
/// 断言 flag（sh3_boss_down / sh3_end_spare）、points+500、sp_grade=D、关键道具 sh3_item_prototype。
#[test]
fn test2_boss_victory_settlement_with_ending() {
    let mut st = GameState::new();
    st.hp = 800;
    st.san = 100;
    st.world_id = "shenghua3".to_string();
    let base = st.points;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    // 开场 → L1 hub → 直接跳到 L3 hub 触发 BOSS（走 sh3_boss_start）
    engine::goto(&mut st, "sh3_04_l3_hub", &mut deaths);
    t2_choose(&mut st, "走向孵化室");
    assert_eq!(st.scene_id, "sh3_09_boss_round", "应进入 BOSS 回合场景");
    assert!(st.fight.is_some(), "应初始化 BOSS 战斗");

    // 重击（index 0 = 34 伤害）打满 260 HP → 约 8 轮
    let mut guard = 0;
    while st.scene_id == "sh3_09_boss_round" && guard < 40 {
        guard += 1;
        engine::choose(&mut st, 0, &mut deaths);
    }
    assert!(guard < 40, "BOSS 未在 guard 轮内被击破");
    assert_eq!(st.scene_id, "sh3_30_final_choice", "击败 BOSS 应进第二选择");
    assert!(st.flag("sh3_boss_down"), "应置 sh3_boss_down");
    assert_eq!(st.sp_grade, Some('D'), "BOSS 击杀应写 sp_grade=D");
    assert_eq!(st.points, base + 500, "击败 BOSS 应 +500 点");
    assert!(st.inventory.iter().any(|i| i == "sh3_item_prototype"), "应掉落原型道具 sh3_item_prototype");

    // 第二选择 · 宽赦
    let pts = st.points;
    t2_choose(&mut st, "宽赦幸存者");
    assert_eq!(st.scene_id, "sh3_31_spare", "宽赦应进 sh3_31_spare");
    assert!(st.flag("sh3_end_spare"), "应置 sh3_end_spare");
    t2_choose(&mut st, "见证重启");
    assert_eq!(st.scene_id, "sh3_42_card", "应进结算卡");
    assert_eq!(st.points, pts + 150, "宽赦结局应再 +150 点");
}