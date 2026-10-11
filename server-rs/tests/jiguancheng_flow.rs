//! 《侠行天下 · 机关城核心》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 JIGUAN_SCENES 并入 scenes::scene()、把 jiguancheng_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_JIGUAN（id="jiguancheng"），保证 engine::goto / engine::choose 能解析 jg_* 场景
//! 与 jc_* 战斗（见 tools/design/jiguancheng_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① L1 地图可达性（出生点 P(14,20) 走遍 L1 关键调查点）
//!   ② 主线链：城门→工坊三启→回廊(取枢机钥)→枢机桥→核心→巨像胜利→结算
//!   ③ 单向传送闭环：静态断言 p_jc_4 为唯一"回跳"（to_floor < floor）传送门
//!   ④ 密匣三分支结局（开启/封存/毁匣各断言）
//!   ⑤ 战斗表完整性（敌人数值 / BOSS HP160 狂暴70 三令终结）
use wuxian_horror_ch1::{engine, state::GameState};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_JIGUAN).expect("机关城世界已注册（合并阶段）")
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
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} clear={} ling={})",
        st.scene_id, st.hp, st.san, st.points,
        st.flag("gear_puzzle_clear") as u8, st.flag("mo_ling_broken") as u8);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

/// 重置为健康前期状态：高 HP + 轻功(齿轮碾阵减半) + 三枢机令（供「以令夺枢」终结）
fn fresh_combat(st: &mut GameState) {
    st.hp = 500;
    st.san = 100;
    st.set_flag("jg_qinggong");
    crate_set_item(st, "it_gear_token_a");
    crate_set_item(st, "it_gear_token_b");
    crate_set_item(st, "it_gear_token_c");
}

fn crate_set_item(st: &mut GameState, item: &str) {
    if !st.inventory.iter().any(|i| i == item) {
        st.inventory.push(item.to_string());
    }
}

/* ---------------- ① L1 地图可达性 ---------------- */
#[test]
fn jiguancheng_l1_map_reachable() {
    let w = world();
    // 四层地图每行恰 40 字符
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    // 出生点应为外城广场入口 (14,20)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (14, 20), "出生点应为 L1 外城广场 (14,20)");
    // L1 关键调查点必须可走动
    for p in w.points {
        if p.floor == 0 {
            assert!(worlds::walkable(w, 0, p.x, p.y), "L1 点 {}@({},{}) 不可走动", p.id, p.x, p.y);
        }
    }
    // 单向门起点必须落在地板（可踩上触发传送）
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.floor, pt.x, pt.y), "传送门 {}@L{}:{} 不可走动", pt.id, pt.floor + 1, pt.y);
    }
    println!("L1 map reachability OK · points={} portals={}", w.points.len(), w.portals.len());
}

/* ---------------- ② 主线链 → 巨像胜利 → 结算 ---------------- */
#[test]
fn jiguancheng_main_line_colossus_win() {
    let mut st = GameState::new();
    fresh_combat(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "jg_00", &mut deaths);
    assert_eq!(st.scene_id, "jg_00");
    step(&mut st, &mut deaths, "查看断碑");          // → jg_01
    assert!(st.flag("jc_stele_scan"));
    step(&mut st, &mut deaths, "进城门闸");           // → jg_02_arrive_workshop
    assert!(st.flag("jc_stele_scan"));

    // 工坊三启
    step(&mut st, &mut deaths, "甲");                 // gear_sw_a
    step(&mut st, &mut deaths, "拔动机关");           // route_gear_pivot → 工坊
    step(&mut st, &mut deaths, "乙");                 // gear_sw_b
    step(&mut st, &mut deaths, "拧动机关");           // → 工坊
    step(&mut st, &mut deaths, "丙");                 // gear_sw_c
    step(&mut st, &mut deaths, "压动机关");           // → 工坊，三启齐发
    assert!(st.flag("gear_puzzle_clear"), "三启应置 gear_puzzle_clear");
    assert!(st.flag("jg_pivot_gate"), "三启应置 jg_pivot_gate");
    assert!(st.inventory.iter().any(|i| i == "it_gear_token_a"), "应有枢机令甲");

    // 升降梯（G2 已三启）→ L3 回廊
    step(&mut st, &mut deaths, "上升降梯");           // → jg_02_lift
    step(&mut st, &mut deaths, "升梯");               // → jg_03_arrive_corridor

    // 回廊：取枢机钥（G3 前提）
    step(&mut st, &mut deaths, "暗格");               // → jg_13_pivot_key
    step(&mut st, &mut deaths, "取走枢机钥");
    assert!(st.inventory.iter().any(|i| i == "it_pivot_key"), "应有枢机钥");
    step(&mut st, &mut deaths, "上枢机桥");           // → jg_02_pivot_bridge
    step(&mut st, &mut deaths, "过枢机桥");           // → jg_21_arrive_core

    // 决战：巨像（选择驱动）
    let pts_before = st.points;
    step(&mut st, &mut deaths, "迎战枢机巨像");       // → jg_22_colossus
    step(&mut st, &mut deaths, "逼近巨像");           // start_colossus → jg_colossus_round

    let mut guard = 0;
    while st.scene_id == "jg_colossus_round" && guard < 30 {
        guard += 1;
        step(&mut st, &mut deaths, "重击");
    }
    assert_eq!(st.scene_id, "jg_23_colossus_down", "巨像胜利应回 jg_23_colossus_down，实际 {}", st.scene_id);
    assert!(st.flag("jc_colossus_down"), "应置 jc_colossus_down");
    assert_eq!(st.sp_grade, Some('D'), "巨像胜利应写 sp_grade=D");
    assert!(st.points > pts_before, "胜利后点数应增加");
    assert!(st.inventory.iter().any(|i| i == "it_wuxue_map"), "应有上古武学图谱残页");

    // 密匣之择 → 结算
    step(&mut st, &mut deaths, "走向密匣石台");        // → jg_30_box
    step(&mut st, &mut deaths, "开启密匣");            // → jg_31_exit
    assert!(st.flag("jc_box_choice"));
    assert!(st.flag("jc_box_open"));
    println!("MAIN LINE OK · points={} (delta {}) · deaths={:?}", st.points, st.points - pts_before, deaths);
}

/* ---------------- ③ 单向传送闭环 ---------------- */
#[test]
fn jiguancheng_one_way_portal_closure() {
    let w = world();
    let portals = w.portals;
    // 存在回跳秘道 p_jc_4（L3→L2，to_floor < floor 的唯一一扇）缝合闭环
    let back = portals.iter().filter(|p| p.to_floor < p.floor).collect::<Vec<_>>();
    assert_eq!(back.len(), 1, "应仅有一扇回跳门（to_floor<floor）制造闭环；实际 {:?}",
        back.iter().map(|p| p.id).collect::<Vec<_>>());
    assert_eq!(back[0].id, "p_jc_4", "唯一回跳应为 p_jc_4 秘道滑轮");
    // 前向多道单向门：p_jc_1/2/3/5 层级递增（物理单向，无反向门）
    let fwd: Vec<&str> = portals.iter().filter(|p| p.to_floor > p.floor).map(|p| p.id).collect();
    assert!(fwd.contains(&"p_jc_1") && fwd.contains(&"p_jc_2") && fwd.contains(&"p_jc_3") && fwd.contains(&"p_jc_5"),
        "单向进深门 p_jc_1/2/3/5 应齐备：{:?}", fwd);
    println!("ONE-WAY CLOSURE OK · 前向门 {} 扇, 回跳门 1 扇(p_jc_4)", fwd.len());
}

/* ---------------- ④ 密匣三分支结局 ---------------- */
#[test]
fn jiguancheng_box_three_branches() {
    // 开启密匣
    let mut st = GameState::new();
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "jg_30_box", &mut deaths);
    step(&mut st, &mut deaths, "开启密匣");
    assert!(st.flag("jc_box_open") && st.flag("jc_box_choice"));
    assert!(st.inventory.iter().any(|i| i == "it_cross_box_open"), "开启密匣应得跨界密匣(开)");

    // 以墨令封匣
    let mut st = GameState::new();
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    crate_set_item(&mut st, "it_mo_ling_full");
    engine::goto(&mut st, "jg_30_box", &mut deaths);
    step(&mut st, &mut deaths, "以墨令封匣");
    assert!(st.flag("jc_box_seal") && st.flag("jc_box_choice"));
    assert!(st.inventory.iter().any(|i| i == "it_cross_box_sealed"), "封匣应得跨界密匣(封)");

    // 毁匣
    let mut st = GameState::new();
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "jg_30_box", &mut deaths);
    let pts = st.points;
    step(&mut st, &mut deaths, "毁匣");
    assert!(st.flag("jc_box_destroy") && st.flag("jc_box_choice"));
    assert!(st.points > pts, "毁匣应加点数");
    println!("BOX 3-BRANCH OK · open/seal/destroy 全通过");
}

/* ---------------- ⑤ 战斗表完整性 ---------------- */
#[test]
fn jiguancheng_fight_table_complete() {
    let fights = wuxian_horror_ch1::scenes_jiguancheng::jiguancheng_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in ["jc_sentry", "jc_guard1", "jc_guard2", "jc_bowman", "jc_gearbest",
                 "jc_guard3", "jc_gearbest2", "jc_bowwall", "jc_guard4", "jc_colossus", "jc_keeper"] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    let colossus = fights.iter().find(|(k, _)| *k == "jc_colossus").map(|(_, c)| c).unwrap();
    assert_eq!(colossus.hp, 160, "巨像 HP 应为 160");
    assert_eq!(colossus.rage_at, Some(70), "巨像狂暴阈 70");
    assert_eq!(colossus.reward, 550, "巨像奖励 550");
    let keeper = fights.iter().find(|(k, _)| *k == "jc_keeper").map(|(_, c)| c).unwrap();
    assert_eq!(keeper.hp, 140, "守城人 HP 应为 140");
    println!("FIGHT TABLE OK · {} 场", fights.len());
}