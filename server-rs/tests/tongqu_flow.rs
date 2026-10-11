//! 《侠行天下 · 通衢古镇 · 夜雨镖局》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 TONGQU_SCENES 并入 scenes::scene()、把 tongqu_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_TONGQU（id="tongqu"），保证 engine::goto / engine::choose 能解析 tq_* 场景
//! 与 tqf_* 战斗（见 tools/design/tongqu_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① 地图可达性：三层附图 40×26、出生点 (14,20)、POINTS/PORTALS 坐标可走动
//!   ② 主线链：镇门 → 市井 → 客栈追凶 → 镇尾古宅 → 雪夜伏击 BOSS 胜利 → 结算（sp_grade=D）
//!   ③ 三线汇流分支：黑店 / 追凶 / 护镖三线各置其 flag，并验证 BOSS「揭面指认」终结路径
use wuxian_horror_ch1::{engine, state::GameState, state::Mode};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_TONGQU).expect("通衢古镇世界已注册（合并阶段）")
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
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} trace={} dark={} biaoju={} grade={:?})",
        st.scene_id, st.hp, st.san, st.points,
        st.flag("tq_trace_2") as u8, st.flag("tq_heidian_truth") as u8,
        st.flag("tq_biaoju_trust") as u8, st.sp_grade);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

/// 驱动标准 Fight（fight_id 场景，索引 0=攻击）直到败敌、退回 Normal 场景
fn fight_through(st: &mut GameState, deaths: &mut Vec<(&'static str, &'static str)>) {
    let mut guard = 0;
    while matches!(st.mode, Mode::Fight) && guard < 80 {
        guard += 1;
        engine::choose(st, 0, deaths);
    }
    assert!(!matches!(st.mode, Mode::Fight), "战斗未终结于 {guard} 回合");
}

/// 前置健康弹幕环境（标准 fight 用军刀保证 index 0=attack 造成伤害；BOSS 选择驱动不受武器影响）
fn fresh_combat(st: &mut GameState) {
    st.hp = 800;
    st.san = 100;
    st.weapon = Some(wuxian_horror_ch1::state::Weapon::Sword);
    st.inventory.clear();
}

/* ---------------- ① 地图可达性 ---------------- */
#[test]
fn tongqu_map_reachable() {
    let w = world();
    for (fi, map) in w.floors.iter().enumerate() {
        assert_eq!(map.len(), 26, "floor{fi} 行数 != 26");
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (14, 20), "出生点应为 L1 镇门 (14,20)");
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "点 {}@({},{}) 不可走动", p.id, p.x, p.y);
    }
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.floor, pt.x, pt.y), "传送门 {}@L{}:{} 不可走动", pt.id, pt.floor + 1, pt.y);
    }
    println!("MAP OK · points={} enemies={} portals={} zones={}", w.points.len(), w.enemies.len(), w.portals.len(), w.zones.len());
}

/* ---------------- ② 主线链 → 雪夜伏击胜利 → 结算 ---------------- */
#[test]
fn tongqu_main_line_boss_win() {
    let mut st = GameState::new();
    st.world_id = wuxian_horror_ch1::worlds::WORLD_TONGQU.to_string();
    fresh_combat(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "tq_00", &mut deaths);
    assert_eq!(st.scene_id, "tq_00");
    step(&mut st, &mut deaths, "闻一闻官道的泥泞");   // → tq_10_track（雨后车辙）
    step(&mut st, &mut deaths, "记下车辙");            // 置 tq_trace_1 → tq_01
    assert!(st.flag("tq_trace_1"), "雨后车辙应置 tq_trace_1");
    step(&mut st, &mut deaths, "进市井长街");          // → tq_02

    // 追凶线：客栈 → 后檐截探子（tqf_tuishun 战斗）→ 押纲清单
    step(&mut st, &mut deaths, "客栈追凶");            // → tq_22_inn
    step(&mut st, &mut deaths, "追到后檐");            // → tq_22_inn_fight（fight_id）
    fight_through(&mut st, &mut deaths);               // 败 tqf_tuishun
    assert_eq!(st.scene_id, "tq_22_inn_win", "应回 tq_22_inn_win，实际 {}", st.scene_id);
    step(&mut st, &mut deaths, "收起押纲清单");         // → tq_22_self，置 tq_trace_2
    assert!(st.flag("tq_trace_2"), "追凶应置 tq_trace_2");
    step(&mut st, &mut deaths, "循线索出镇尾");         // → tq_03

    // 决战：雪夜伏击（选择驱动，Mode::Normal）
    step(&mut st, &mut deaths, "直面雪夜伏击");         // → tq_boss_enter
    step(&mut st, &mut deaths, "逼近头领");            // start_boss → tq_boss_round
    let mut rounds = 0;
    while st.scene_id == "tq_boss_round" && rounds < 60 {
        rounds += 1;
        step(&mut st, &mut deaths, "重击");
    }
    assert_eq!(st.scene_id, "tq_boss_win", "头领胜利应回 tq_boss_win，实际 {}", st.scene_id);
    assert!(st.flag("tq_boss_down"), "应置 tq_boss_down");
    assert_eq!(st.sp_grade, Some('D'), "头领胜利应写 sp_grade=D");
    assert!(st.inventory.iter().any(|i| i == "it_tq_secret_letter"), "应有密信 it_tq_secret_letter");

    // 结算
    step(&mut st, &mut deaths, "走向供桌");             // → tq_33_exit
    step(&mut st, &mut deaths, "踏入撤离阵");           // → tq_34_card（结算）
    assert_eq!(st.scene_id, "tq_34_card", "应回结算卡片，实际 {}", st.scene_id);
    println!("MAIN LINE OK · points={} rounds={} deaths={:?}", st.points, rounds, deaths);
}

/* ---------------- ③ 三线汇流分支 ---------------- */
#[test]
fn tongqu_three_branches() {
    let mut st = GameState::new();
    st.world_id = wuxian_horror_ch1::worlds::WORLD_TONGQU.to_string();
    fresh_combat(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    // 黑店线：黑店暗门 → 后厨隔板 → 打手战 → 供词 → tq_heidian_truth
    engine::goto(&mut st, "tq_21_dark_shop", &mut deaths);
    step(&mut st, &mut deaths, "推开后厨的隔板");       // → tq_21_dark_fight
    fight_through(&mut st, &mut deaths);               // 败 tqf_thug
    assert_eq!(st.scene_id, "tq_21_dark_win", "黑店应胜至 tq_21_dark_win，实际 {}", st.scene_id);
    step(&mut st, &mut deaths, "记下黑店供词");         // 置 tq_heidian_truth → tq_02
    assert!(st.flag("tq_heidian_truth"), "黑店线应置 tq_heidian_truth");
    println!("BLACK-SHOP OK · scene={} pts={}", st.scene_id, st.points);

    // 追凶线（从市井 hub 出发）置 tq_trace_2
    let mut st2 = GameState::new();
    st2.world_id = wuxian_horror_ch1::worlds::WORLD_TONGQU.to_string();
    fresh_combat(&mut st2);
    let mut d2: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st2, "tq_22_inn", &mut d2);
    step(&mut st2, &mut d2, "追到后檐");
    fight_through(&mut st2, &mut d2);
    step(&mut st2, &mut d2, "收起押纲清单");
    assert!(st2.flag("tq_trace_2"), "追凶线应置 tq_trace_2");
    println!("PURSUIT OK · pts={}", st2.points);

    // 护镖线：沈镖头处接委托（需追凶或黑店佐证）→ 过招护院 → tq_biaoju_trust
    let mut st3 = GameState::new();
    st3.world_id = wuxian_horror_ch1::worlds::WORLD_TONGQU.to_string();
    fresh_combat(&mut st3);
    let mut d3: Vec<(&'static str, &'static str)> = vec![];
    st3.set_flag("tq_trace_2"); // 供文已具备追凶佐证
    engine::goto(&mut st3, "tq_20_biaoju", &mut d3);
    step(&mut st3, &mut d3, "接护镖委托");              // → tq_20_huyuan_fight（护院战）
    assert!(st3.flag("tq_biaoju_trust"), "接委托应置 tq_biaoju_trust");
    assert!(st3.inventory.iter().any(|i| i == "it_tq_biaoju_token"), "应有镖局信物");
    step(&mut st3, &mut d3, "过招");                    // → tq_20_huyuan_fight2（fight_id）
    fight_through(&mut st3, &mut d3);
    assert_eq!(st3.scene_id, "tq_20_huyuan_win", "护院应胜至 tq_20_huyuan_win，实际 {}", st3.scene_id);
    step(&mut st3, &mut d3, "踏进内院");                // → tq_02
    println!("BIAOJU OK · pts={}", st3.points);

    // BOSS「揭面指认」结局路径：已识黑店（tq_heidian_truth）→ 狂暴后可「揭面指认」→ 胜利
    let mut stb = GameState::new();
    stb.world_id = wuxian_horror_ch1::worlds::WORLD_TONGQU.to_string();
    fresh_combat(&mut stb);
    let mut db: Vec<(&'static str, &'static str)> = vec![];
    stb.set_flag("tq_heidian_truth");
    engine::goto(&mut stb, "tq_boss_enter", &mut db);
    step(&mut stb, &mut db, "逼近头领");               // start_boss → tq_boss_round
    // 把 BOSS 打到狂暴（HP≤70）
    let mut loopguard = 0;
    while stb.scene_id == "tq_boss_round"
        && stb.fight.as_ref().map(|f| !f.raged).unwrap_or(false)
        && loopguard < 40 {
        loopguard += 1;
        step(&mut stb, &mut db, "重击");
    }
    assert!(stb.fight.as_ref().map(|f| f.raged).unwrap_or(false) || stb.scene_id != "tq_boss_round",
        "BOSS 应进入狂暴或已结束");
    // 狂暴后「揭面指认」（识破黑店 · 40 固伤），随后补刀至胜利
    if stb.scene_id == "tq_boss_round" {
        step(&mut stb, &mut db, "揭面指认");
    }
    let mut endguard = 0;
    while stb.scene_id == "tq_boss_round" && endguard < 40 {
        endguard += 1;
        step(&mut stb, &mut db, "重击");
    }
    assert_eq!(stb.scene_id, "tq_boss_win", "揭面指认后应胜至 tq_boss_win，实际 {}", stb.scene_id);
    assert!(stb.flag("tq_boss_down"), "应置 tq_boss_down");
    assert_eq!(stb.sp_grade, Some('D'), "应写 sp_grade=D");
    println!("BRANCH-BOSS OK · pts={} scene={}", stb.points, stb.scene_id);
}