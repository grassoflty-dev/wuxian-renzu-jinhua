//! 《侏罗纪公园 · 失序乐园》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 JULUOJI_SCENES 并入 scenes::scene()、把 juluoji_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_JULUOJI（id="juluoji"），保证 engine::goto / engine::choose / engine::fight
//! 能解析 jl_* 场景与 jl_* 战斗（见 tools/design/juluoji_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① 地图可达性（出生点 P(1,20) 走遍 L1 关键调查点 + 传送门起点落脚）
//!   ② 主线链→霸王龙战（园区断电→丛林→围栏）→胜利→B 级结算
//!   ③ 恐龙追击：EnemyDef.radius 拉近断言（迅猛龙/剑龙/霸王龙追击半径），保证进场贴脸
use wuxian_horror_ch1::{engine, state::GameState};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_JULUOJI).expect("侏罗纪公园世界已注册（合并阶段）")
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
    println!("STEP [{keyword}] → {} (hp={} san={} pts={})", st.scene_id, st.hp, st.san, st.points);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

fn crate_set_item(st: &mut GameState, item: &str) {
    if !st.inventory.iter().any(|i| i == item) {
        st.inventory.push(item.to_string());
    }
}

/* ---------------- ① 地图可达性 ---------------- */
#[test]
fn juluoji_map_reachable() {
    let w = world();
    // 三层地图每行恰 40 字符
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    // 出生点应为 L1 园区 (1,20)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (1, 20), "出生点应为 L1 园区 (1,20)");
    // 各层调查点必须可走动（开放式岛图 + 追击半径）
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "点 {}@({},{})L{} 不可走动", p.id, p.x, p.y, p.floor + 1);
    }
    // 传送门起点必须落在地板
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.floor, pt.x, pt.y), "传送门 {}@L{}:{} 不可走动", pt.id, pt.floor + 1, pt.x);
    }
    println!("map reachability OK · floors={} points={} enemies={} gates={} portals={}",
        w.floors.len(), w.points.len(), w.enemies.len(), w.gates.len(), w.portals.len());
}

/* ---------------- ② 主线链 → 霸王龙战 → B 级结算 ---------------- */
#[test]
fn juluoji_main_line_trex_win() {
    let mut st = GameState::new();
    st.hp = 400;
    crate_set_item(&mut st, "it_wrench");
    crate_set_item(&mut st, "it_bait_meat");
    crate_set_item(&mut st, "it_wire_cutters");
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "jl_00", &mut deaths);
    assert_eq!(st.scene_id, "jl_00");
    step(&mut st, &mut deaths, "查看断电控制台");    // → jl_02_powerlog
    assert!(st.flag("jl_powerlog"));
    step(&mut st, &mut deaths, "记下跌电点");          // → jl_01

    // 园区：售货亭冰柜取午餐肉 → 进丛林
    step(&mut st, &mut deaths, "售货亭冰柜");          // → jl_03_bait
    step(&mut st, &mut deaths, "撬开冷藏门");          // 需扳手(已备) → jl_01
    assert!(st.inventory.iter().any(|i| i == "it_bait_meat"), "应有午餐肉");
    step(&mut st, &mut deaths, "穿过电击围栏侧门");     // → jl_02_arrive_jungle

    // 丛林：抛午餐肉免战引导 → hub
    step(&mut st, &mut deaths, "扔出午餐肉引开");       // → jl_02_jungle_hub
    assert!(st.flag("jl_used_bait"));
    step(&mut st, &mut deaths, "撕抓树痕");            // → jl_07_marks
    step(&mut st, &mut deaths, "抽走铁丝");            // 得铁丝钳
    assert!(st.inventory.iter().any(|i| i == "it_wire_cutters"), "应有铁丝钳");
    step(&mut st, &mut deaths, "穿越密林窄道");         // → jl_03_arrive_fence

    // 围栏区：主电闸 → 决战霸王龙 → 胜利
    step(&mut st, &mut deaths, "围栏主电闸");           // → jl_09_fuse
    step(&mut st, &mut deaths, "合拢主电闸");           // fence_power
    assert!(st.flag("fence_power"));
    step(&mut st, &mut deaths, "仰望霸王龙");            // → jl_13_trex
    step(&mut st, &mut deaths, "直面霸王龙");            // start_trex → jl_trex_round

    let pts_before = st.points;
    let mut guard = 0;
    while st.scene_id == "jl_trex_round" && guard < 60 {
        guard += 1;
        step(&mut st, &mut deaths, "瞄准跛脚重击");
    }
    assert_eq!(st.scene_id, "jl_13_trex_down", "霸王龙胜利应回 jl_13_trex_down，实际 {}", st.scene_id);
    assert!(st.flag("jl_trex_down"), "应置 jl_trex_down");
    assert_eq!(st.sp_grade, Some('B'), "霸王龙胜利应写 sp_grade=B");
    assert!(st.points > pts_before, "胜利后点数应增加");
    assert!(st.inventory.iter().any(|i| i == "it_trex_tooth"), "应有恐龙牙饰");

    // 结算
    step(&mut st, &mut deaths, "走向撤离台");            // → jl_11_exit
    step(&mut st, &mut deaths, "踏入撤离光柱");          // → jl_12_card
    assert_eq!(st.scene_id, "jl_12_card", "胜利结算回结算卡片");
    println!("MAIN LINE OK · points={} (delta {}) · deaths={:?}", st.points, st.points - pts_before, deaths);
}

/* ---------------- ③ 恐龙追击（EnemyDef.radius 拉近） ---------------- */
#[test]
fn juluoji_chase_radius() {
    let w = world();
    // 追击半径必须拉近：迅猛龙≥5、剑龙群≥4、霸王龙≥6 —— 进场即贴脸触发战斗
    let raptor = w.enemies.iter().find(|e| e.id == "jl_e_l1_raptor1").expect("迅猛龙存在");
    assert!(raptor.radius >= 5, "迅猛龙追击半径应≥5（拉近贴脸），实际 {}", raptor.radius);
    let stego = w.enemies.iter().find(|e| e.id == "jl_e_l1_stego").expect("剑龙群存在");
    assert!(stego.radius >= 4, "剑龙群追击半径应≥4，实际 {}", stego.radius);
    let trex = w.enemies.iter().find(|e| e.id == "jl_e_l3_trex").expect("霸王龙存在");
    assert!(trex.radius >= 6, "霸王龙追击半径应≥6（全场贴脸），实际 {}", trex.radius);
    // 追击半径应显著大于普通敌人的默认半径（默认≤3），体现"恐龙追击"
    assert!(trex.radius > 3 && raptor.radius > 3 && stego.radius > 3, "追击半径应全部 >3");
    println!("CHASE RADIUS OK · raptor={} stego={} trex={}", raptor.radius, stego.radius, trex.radius);
}

/* ---------------- ④ 战斗表完整性 ---------------- */
#[test]
fn juluoji_fight_table() {
    let fights = wuxian_horror_ch1::scenes_juluoji::juluoji_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in ["jl_raptor", "jl_stego", "jl_raptor_pack", "jl_stego2", "jl_trex"] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    let trex = fights.iter().find(|(k, _)| *k == "jl_trex").map(|(_, c)| c).unwrap();
    assert_eq!(trex.hp, 260, "霸王龙 HP 应为 260");
    assert_eq!(trex.rage_at, Some(100), "霸王龙狂暴阈 100");
    assert_eq!(trex.reward, 650, "霸王龙奖励 650");
    let raptor = fights.iter().find(|(k, _)| *k == "jl_raptor").map(|(_, c)| c).unwrap();
    assert_eq!(raptor.hp, 46, "迅猛龙 HP 46");
    println!("FIGHT TABLE OK · {} 场", fights.len());
}