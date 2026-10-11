//! 《侠行天下 · 藏经阁》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 CANGJING_SCENES 并入 scenes::scene()、把 cangjingge_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_CANGJING（id="cangjingge"），保证 engine::goto / engine::choose 能解析 cj_* 场景
//! 与 cj_* 战斗（见 tools/design/cangjingge_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① 地图可达性（出生点 P(14,20) 走遍各层调查点 / 传送门落点）
//!   ② 主线链：山门→一楼(残卷/书灵/守经人指引)→书梯→二楼(取檀木秘钥)→秘籍塔→BOSS 胜利→结局→结算
//!   ③ 单向传送闭环：静态断言 p_cj_back 为唯一"回跳"（to_floor < floor）传送门
//!   ④ 禁书三分支结局（研读/重封/焚书）+ 免战解脱（出示檀木信物）
use wuxian_horror_ch1::{engine, state::GameState};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_CANGJING).expect("藏经阁世界已注册（合并阶段）")
}

fn pick(st: &GameState, keyword: &str) -> i32 {
    let scene = wuxian_horror_ch1::scenes::scene(&st.scene_id).expect("scene");
    let visible: Vec<_> = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(st))).collect();
    for (i, c) in visible.iter().enumerate() {
        if c.label.contains(keyword) {
            return i as i32;
        }
    }
    panic!("scene {} 未找到含「{}」的选项；可见选项: {:?}",
        st.scene_id, keyword, visible.iter().map(|c| c.label).collect::<Vec<_>>());
}

fn step(st: &mut GameState, deaths: &mut Vec<(&'static str, &'static str)>, keyword: &str) {
    let idx = pick(st, keyword);
    engine::choose(st, idx, deaths);
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} clue1={} clue2={})",
        st.scene_id, st.hp, st.san, st.points,
        st.flag("keeper_clue_1") as u8, st.flag("keeper_clue_2") as u8);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

fn crate_set_item(st: &mut GameState, item: &str) {
    if !st.inventory.iter().any(|i| i == item) {
        st.inventory.push(item.to_string());
    }
}

/// 重置为健康前期状态：高 HP（供 BOSS 选择驱动多回合）
fn fresh(st: &mut GameState) {
    st.hp = 500;
    st.san = 100;
    // 直接备好绝学密室信物（供免战分支）
    crate_set_item(st, "it_tan_token");
    crate_set_item(st, "it_tan_key");
}

/* ---------------- ① 地图可达性 ---------------- */
#[test]
fn cangjingge_map_reachable() {
    let w = world();
    // 每层地图每行恰 40 字符
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    // 出生点应为 L0 经堂 (14,20)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (14, 20), "出生点应为 L0 经堂 (14,20)");
    // 各调查点必须可走动
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "点 {}@floor{}:({},{}) 不可走动", p.id, p.floor, p.x, p.y);
    }
    // 单向门起点必须落在地板
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.floor, pt.x, pt.y), "传送门 {}@floor{}:({},{}) 不可走动", pt.id, pt.floor, pt.x, pt.y);
    }
    println!("map reachability OK · points={} portals={} gates={}", w.points.len(), w.portals.len(), w.gates.len());
}

/* ---------------- ② 主线链 → 入魔守阁僧 BOSS → 结局 → 结算 ---------------- */
#[test]
fn cangjingge_main_line_boss_win() {
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "cj_00", &mut deaths);
    assert_eq!(st.scene_id, "cj_00");
    step(&mut st, &mut deaths, "直入经堂");          // → cj_01（此测试走正路，先取铜铃）
    // 取铜铃 → G1 经堂内门
    step(&mut st, &mut deaths, "铜铃法台");          // → cj_06_bell
    step(&mut st, &mut deaths, "取下铜铃");          // it_tongling → cj_01
    assert!(st.inventory.iter().any(|i| i == "it_tongling"), "应有铜铃");
    step(&mut st, &mut deaths, "进经堂内门");         // → cj_10_arrive_floor1

    // 一楼：集三卷残卷（甲/乙/丙）
    step(&mut st, &mut deaths, "残卷·甲");           // → cj_11_scroll_a
    step(&mut st, &mut deaths, "取走残卷·甲");        // route_miju_collect → cj_10_arrive_floor1
    step(&mut st, &mut deaths, "残卷·乙");
    step(&mut st, &mut deaths, "取走残卷·乙");
    step(&mut st, &mut deaths, "残卷·丙");
    step(&mut st, &mut deaths, "取走残卷·丙");        // 三卷齐 → it_miju_full
    assert!(st.inventory.iter().any(|i| i == "it_miju_full"), "三卷应合成真解 it_miju_full");

    // 守经人手札 → 守经人指引（G2）
    step(&mut st, &mut deaths, "守经人手札");         // → cj_13_keeper_note
    step(&mut st, &mut deaths, "合上手札");           // route_keeper_clue → set clue2 → cj_10_arrive_floor1
    assert!(st.flag("keeper_clue_2"), "守经人指引应置 keeper_clue_2");

    // 书梯（G2 → 二楼）
    step(&mut st, &mut deaths, "中央书梯");           // → cj_14_lift
    step(&mut st, &mut deaths, "升梯至二楼禁书库");   // p_cj_2 单向 → cj_14_arrive_floor2

    // 二楼：取檀木秘钥（G3）
    step(&mut st, &mut deaths, "锁钥架 · 檀木匣");    // → cj_17_tan_key
    step(&mut st, &mut deaths, "取走檀木秘钥");       // it_tan_key → cj_14_arrive_floor2
    assert!(st.inventory.iter().any(|i| i == "it_tan_key"), "应有檀木秘钥");

    // 禁书库走廊西口 → 秘籍塔（p_cj_4）
    step(&mut st, &mut deaths, "禁书库走廊西口");     // → cj_19_arrive_tower

    // BOSS 决战：入魔守阁僧（选择驱动）
    let pts0 = st.points;
    step(&mut st, &mut deaths, "迎战入魔守阁僧");     // → cj_22_shouge
    step(&mut st, &mut deaths, "逼近塔台");           // start_shouge → cj_shouge_round

    let mut guard = 0;
    while st.scene_id == "cj_shouge_round" && guard < 30 {
        guard += 1;
        step(&mut st, &mut deaths, "重击");
    }
    assert_eq!(st.scene_id, "cj_23_shouge_down", "守阁僧胜利应回 cj_23_shouge_down，实际 {}", st.scene_id);
    assert!(st.flag("cj_shouge_down"), "应置 cj_shouge_down");
    assert_eq!(st.sp_grade, Some('D'), "守阁僧胜利应写 sp_grade=D");
    assert!(st.points > pts0, "胜利后点数应增加");

    // 结局分支：研读禁书 → 结算
    step(&mut st, &mut deaths, "走向经匣石台");       // → cj_30_box
    step(&mut st, &mut deaths, "研读禁书");           // → cj_31_exit
    assert!(st.flag("cj_book_read") && st.flag("cj_book_choice"));
    step(&mut st, &mut deaths, "踏入撤离阵");         // → cj_32_card（sp_grade=D 兜底）
    println!("MAIN LINE OK · points={} (delta {}) · deaths={:?}", st.points, st.points - pts0, deaths);
}

/* ---------------- ③ 单向传送闭环 ---------------- */
#[test]
fn cangjingge_one_way_portal_closure() {
    let w = world();
    let back = w.portals.iter().filter(|p| p.to_floor < p.floor).collect::<Vec<_>>();
    assert_eq!(back.len(), 1, "应仅有一扇回跳门（to_floor<floor）制造闭环；实际 {:?}",
        back.iter().map(|p| p.id).collect::<Vec<_>>());
    assert_eq!(back[0].id, "p_cj_back", "唯一回跳应为 p_cj_back 经阁密道");
    let fwd: Vec<&str> = w.portals.iter().filter(|p| p.to_floor > p.floor).map(|p| p.id).collect();
    assert!(fwd.contains(&"p_cj_1") && fwd.contains(&"p_cj_2") && fwd.contains(&"p_cj_4"),
        "单向进深门 p_cj_1/2/4 应齐备：{:?}", fwd);
    println!("ONE-WAY CLOSURE OK · 前向门 {} 扇, 回跳门 1 扇(p_cj_back)", fwd.len());
}

/* ---------------- ④ 禁书三分支结局 + 免战解脱 ---------------- */
#[test]
fn cangjingge_ending_three_branches_and_freed() {
    // 免战解脱：出示檀木信物（狂暴后）
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "cj_22_shouge", &mut deaths);
    step(&mut st, &mut deaths, "逼近塔台");           // start_shouge
    let mut iter = 0;
    while st.scene_id == "cj_shouge_round" && iter < 30 {
        // 先打到狂暴（HP≤60），再出示信物
        if st.flag("cj_shouge_freed") { break; }
        if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
            step(&mut st, &mut deaths, "【出示檀木信物】");
        } else {
            step(&mut st, &mut deaths, "重击");
        }
        iter += 1;
    }
    assert_eq!(st.scene_id, "cj_24_shouge_freed", "免战应回 cj_24_shouge_freed，实际 {}", st.scene_id);
    assert!(st.flag("cj_shouge_freed"), "应置 cj_shouge_freed");
    assert_eq!(st.sp_grade, Some('D'));

    // 研读禁书
    let mut st = GameState::new();
    engine::goto(&mut st, "cj_30_box", &mut deaths);
    step(&mut st, &mut deaths, "研读禁书");
    assert!(st.flag("cj_book_read") && st.flag("cj_book_choice"));

    // 以铁页重封
    let mut st = GameState::new();
    engine::goto(&mut st, "cj_30_box", &mut deaths);
    step(&mut st, &mut deaths, "以铁页重封");
    assert!(st.flag("cj_book_seal") && st.flag("cj_book_choice"));

    // 焚毁禁书
    let mut st = GameState::new();
    engine::goto(&mut st, "cj_30_box", &mut deaths);
    let pts = st.points;
    step(&mut st, &mut deaths, "焚毁禁书");
    assert!(st.flag("cj_book_burn") && st.flag("cj_book_choice"));
    assert!(st.points > pts, "焚书应加点数");
    println!("ENDING 3-BRANCH + FREED OK · read/seal/burn/freed 全通过");
}

/* ---------------- ⑤ 战斗表完整性 ---------------- */
#[test]
fn cangjingge_fight_table_complete() {
    let fights = wuxian_horror_ch1::scenes_cangjingge::cangjingge_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in ["cj_guard1", "cj_zhikui", "cj_guard2", "cj_hunter", "cj_guard3", "cj_guard4",
                 "cj_shuling", "cj_shouge", "cj_xinmo"] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    let shouge = fights.iter().find(|(k, _)| *k == "cj_shouge").map(|(_, c)| c).unwrap();
    assert_eq!(shouge.hp, 150, "守阁僧 HP 应为 150");
    assert_eq!(shouge.rage_at, Some(60), "守阁僧狂暴阈 60");
    assert_eq!(shouge.reward, 300, "守阁僧奖励 300");
    let xinmo = fights.iter().find(|(k, _)| *k == "cj_xinmo").map(|(_, c)| c).unwrap();
    assert_eq!(xinmo.hp, 90, "心魔 HP 应为 90");
    println!("FIGHT TABLE OK · {} 场", fights.len());
}