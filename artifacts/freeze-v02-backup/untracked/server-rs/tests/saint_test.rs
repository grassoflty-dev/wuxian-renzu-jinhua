//! 五阶圣人（基因锁第五阶）达成条件 —— 集成测试
//! 覆盖：
//!   1. 三满层判定：实力（合道7 + 基因四阶4）/ 心性（六维加权境界≥道心凝练）/ 气运（完美通关≥FORTUNE_MAX）。
//!   2. Eff::Mind 记录心性维度增减（1~10 夹取）+ 临在/坚毅的行为测量。
//!   3. 五阶圣人兑换：可见性（cond_gene5）+ 达成解锁 / 未达成隐藏。
//!   4. S 评级结算计入气运（幂等防重复计数）。
use wuxian_horror_ch1::combat_data;
use wuxian_horror_ch1::defs::{self, Eff};
use wuxian_horror_ch1::engine;
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::state::GameState;

/// 六维心性全部拉到满（境界 = 心灵之光·明照）
fn max_mind(st: &mut GameState) {
    for dim in ["guanzhao", "sheli", "cibei", "linzai", "jianyi", "guanyi"] {
        st.mind_shift(dim, 9); // 1 → 10
    }
}

/// 造一个「实力满层 + 气运满层 + 心性满层」齐备的准圣人状态
fn near_saint() -> GameState {
    let mut st = GameState::new();
    st.cultivation_stage = 7; // 合道（修真满层）
    combat_data::set_gene_stage(&mut st, 4); // 规则之门（基因四阶）
    st.perfect_clears = defs::FORTUNE_MAX; // 气运满层
    max_mind(&mut st); // 心性满层（六维 10）
    st
}

/// 在场景里按关键词找可见选项索引（与既有测试 pick 同口径）
fn pick(st: &GameState, keyword: &str) -> i32 {
    let scene = scenes::scene(&st.scene_id).expect("scene");
    let visible: Vec<_> = scene
        .choices
        .iter()
        .filter(|c| c.cond.map_or(true, |f| f(st)))
        .collect();
    for (i, c) in visible.iter().enumerate() {
        if c.label.contains(keyword) {
            return i as i32;
        }
    }
    panic!(
        "scene {} 未找到含「{}」的选项；可见: {:?}",
        st.scene_id,
        keyword,
        visible.iter().map(|c| c.label).collect::<Vec<_>>()
    );
}

#[test]
fn power_maxed_requires_he_dao_and_gene_four() {
    let mut st = GameState::new();
    st.cultivation_stage = 7;
    combat_data::set_gene_stage(&mut st, 4);
    assert!(combat_data::power_maxed(&st), "合道+四阶应为实力满层");
    st.cultivation_stage = 6;
    assert!(!combat_data::power_maxed(&st), "未到合道不算实力满层");
    st.cultivation_stage = 7;
    combat_data::set_gene_stage(&mut st, 3);
    assert!(!combat_data::power_maxed(&st), "未到四阶不算实力满层");
}

#[test]
fn fortune_maxed_by_perfect_clears() {
    let mut st = GameState::new();
    st.perfect_clears = defs::FORTUNE_MAX - 1;
    assert!(!combat_data::fortune_maxed(&st), "差一次不算气运满层");
    st.perfect_clears = defs::FORTUNE_MAX;
    assert!(combat_data::fortune_maxed(&st));
}

#[test]
fn mind_maxed_requires_dao_xin_realm() {
    let mut st = GameState::new();
    // 默认六维全 1 → realm 0（凡心），不满层
    assert!(!combat_data::mind_maxed(&st), "凡心（默认 1）不算心性满层");
    // 全部拉到 8 → weighted 8 ≥ 7 → realm 3（道心·凝练）
    for dim in ["guanzhao", "sheli", "cibei", "linzai", "jianyi", "guanyi"] {
        st.mind_shift(dim, 7); // 1 → 8
    }
    assert!(combat_data::mind_maxed(&st), "六维 8 应为道心·凝练满层");
}

#[test]
fn mind_effect_shifts_and_clamps() {
    let mut st = GameState::new();
    assert_eq!(st.mind.cibei, 1, "默认慈悲 1");
    Eff::Mind("cibei", 3).apply(&mut st);
    assert_eq!(st.mind.cibei, 4, "慈悲 +3 应为 4");
    Eff::Mind("cibei", -100).apply(&mut st);
    assert_eq!(st.mind.cibei, 1, "慈悲应夹取到下限 1");
    Eff::Mind("cibei", 100).apply(&mut st);
    assert_eq!(st.mind.cibei, 10, "慈悲应夹取到上限 10");
    // 未知维度不崩、不变
    Eff::Mind("nope", 5).apply(&mut st);
    assert_eq!(st.mind.cibei, 10);
}

#[test]
fn deliberation_records_linzai() {
    let mut st = GameState::new();
    st.record_deliberation(1500);
    assert_eq!(st.mind.linzai, 1, "<3s 不加临在");
    st.record_deliberation(5000);
    assert_eq!(st.mind.linzai, 2, "≥3s 临在 +1");
    st.record_deliberation(9000);
    assert_eq!(st.mind.linzai, 4, "≥8s 临在 +2");
}

#[test]
fn saint_ready_requires_all_three() {
    let mut st = near_saint();
    assert!(combat_data::saint_ready(&st));
    st.perfect_clears = 0;
    assert!(!combat_data::saint_ready(&st), "气运不足不算圣人达成");
}

#[test]
fn saint_exchange_visible_and_unlocks() {
    let mut st = near_saint();
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_exchange_gene", &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_gene");
    let idx = pick(&st, "五阶");
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(combat_data::gene_stage_of(&st), 5, "兑换后应达五阶圣人");
    assert_eq!(st.scene_id, "s_nexus_exchange_done");
}

#[test]
fn saint_exchange_hidden_when_not_ready() {
    let mut st = GameState::new();
    st.cultivation_stage = 7;
    combat_data::set_gene_stage(&mut st, 4); // 实力满，但气运/心性未满
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_exchange_gene", &mut deaths);
    let scene = scenes::scene(&st.scene_id).expect("scene");
    let visible: Vec<_> = scene
        .choices
        .iter()
        .filter(|c| c.cond.map_or(true, |f| f(&st)))
        .collect();
    assert!(
        visible.iter().all(|c| !c.label.contains("五阶")),
        "未达成时五阶选项应隐藏"
    );
}

#[test]
fn s_rank_settlement_increments_perfect_clear() {
    let mut st = GameState::new();
    st.points = 2000;
    for k in [
        "A",
        "B1",
        "B2",
        "C",
        "decon_truth",
        "server_cooling",
        "nav_manual_cross",
    ] {
        st.set_flag(k);
    }
    let mut deaths = vec![];
    engine::goto(&mut st, "s_settle", &mut deaths);
    assert_eq!(st.settle_rank, 'S');
    assert_eq!(st.perfect_clears, 1, "S 评级结算应计一次完美通关");
    engine::goto(&mut st, "s_settle", &mut deaths);
    assert_eq!(st.perfect_clears, 1, "重复进入结算不应重复计数");
}

#[test]
fn choosing_moral_choice_shifts_cibei() {
    // 卡普兰：救 = 慈悲+1 舍离+1
    let mut st = GameState::new();
    let mut deaths = vec![];
    engine::goto(&mut st, "s_bhall", &mut deaths);
    let idx = pick(&st, "冲上去救卡普兰");
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(st.mind.cibei, 2, "救卡普兰应慈悲 +1");
    assert_eq!(st.mind.sheli, 2, "救卡普兰应舍离 +1");
}

#[test]
fn promise_fulfillment_shifts_guanyi() {
    // 许下承诺 → 兑现：贯一 +1
    let mut st = GameState::new();
    Eff::Promise("protect").apply(&mut st);
    assert!(st.flag("promise_protect"), "应记录承诺旗标");
    Eff::Fulfill("protect").apply(&mut st);
    assert_eq!(st.mind.guanyi, 2, "兑现承诺应贯一 +1");
    assert!(!st.flag("promise_protect"), "兑现后承诺旗标应清除");

    // 许下承诺 → 违背：贯一 -1（夹取到 1）
    let mut st2 = GameState::new();
    Eff::Promise("protect").apply(&mut st2);
    Eff::BreakPromise("protect").apply(&mut st2);
    assert_eq!(st2.mind.guanyi, 1, "违背承诺应贯一 -1（夹取到 1）");

    // 无承诺时兑现不生效
    let mut st3 = GameState::new();
    Eff::Fulfill("none").apply(&mut st3);
    assert_eq!(st3.mind.guanyi, 1, "无承诺时兑现不应生效");
}
