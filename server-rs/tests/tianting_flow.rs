//! 《洪荒天庭》高难任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 TIANTING_SCENES 并入 scenes::scene()、把 tianting_figths() 并入 scenes::fight_cfg()、
//! 把 WORLD_TIANTING 注册进 worlds::WORLDS，并新增 `mod scenes_tianting;`（见 tools/design/tianting_impl_log.md ★外部依赖）。
//! 测试内容：①L1 地图可达性 ②主线链→神将 BOSS 两段→东天二皇演出（不战斗）→结局分支 ③演出红线：东天二皇 fight_id None / 不初始化 Fight / 不进战斗模式。
use wuxian_horror_ch1::{engine, state::{GameState, Mode}};
use wuxian_horror_ch1::worlds;

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

/* ---------------- ① L1 地图可达性 ---------------- */
#[test]
fn tianting_f1_map_reachable() {
    let w = worlds::find_world(worlds::WORLD_TIANTING).expect("洪荒天庭世界已注册（合并阶段）");
    // 每行恰 40 字符
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    // 出生点应为坠落点 (2,13)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (2, 13), "出生点应为坠落点 (2,13)");
    // L1 关键点本身可走动
    for p in w.points {
        if p.floor == 0 {
            assert!(worlds::walkable(w, 0, p.x, p.y), "L1 点 {}@({},{}) 不可走动", p.id, p.x, p.y);
        }
    }
    // BFS 验证 L1 全层连通（出生点可达所有 L1 可走动格 → 关键调查点可达）
    let reachable = flood_fill(w, 0, sx, sy);
    for p in w.points {
        if p.floor == 0 {
            assert!(reachable.contains(&(p.x, p.y)), "L1 点 {}@({},{}) 从出生点不可达", p.id, p.x, p.y);
        }
    }
    println!("L1 map reachability OK · reachable={}", reachable.len());
}

fn flood_fill(w: &'static worlds::WorldData, floor: usize, sx: usize, sy: usize) -> std::collections::HashSet<(usize, usize)> {
    let mut seen = std::collections::HashSet::new();
    let mut stack = vec![(sx, sy)];
    while let Some((x, y)) = stack.pop() {
        if !seen.insert((x, y)) { continue; }
        for (dx, dy) in [(1usize, 0usize), (0usize, 1usize)] {
            let (nx, ny) = (x.wrapping_add(dx), y.wrapping_add(dy));
            if worlds::walkable(w, floor, nx, ny) { stack.push((nx, ny)); }
        }
        if x > 0 && worlds::walkable(w, floor, x - 1, y) { stack.push((x - 1, y)); }
        if y > 0 && worlds::walkable(w, floor, x, y - 1) { stack.push((x, y - 1)); }
    }
    seen
}

/* ---------------- ② 主线链：断碑→残页→神桥→星宿→封神台真相→BOSS 两段→演出→结局 ---------------- */
#[test]
fn tianting_main_line_boss_two_phase() {
    let mut st = GameState::new();
    st.hp = 600; st.san = 100; // 抬高血条以稳定扛过 1/2 段 BOSS 累积反击（否则随机反击可致死）
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "tt_01_drop_land", &mut deaths);
    assert_eq!(st.scene_id, "tt_01_drop_land");

    // 天条断碑 → 残页一
    step(&mut st, &mut deaths, "上前查看天条断碑");     // → tt_03_stele
    step(&mut st, &mut deaths, "揭下封神榜残页");       // → tt_02_gate，得 残页一
    assert!(st.flag("tt_fengshen_p1"), "断碑应得 残页一");

    // 穿越南天门 G1 → 神桥
    step(&mut st, &mut deaths, "穿越南天门");           // G1 → tt_05_bridge
    step(&mut st, &mut deaths, "与录事官残魂对话");      // → tt_05_bridge_lushi
    step(&mut st, &mut deaths, "谢过，前往云海断碑");    // → tt_06_cloud_stele
    step(&mut st, &mut deaths, "取走封神榜残页");        // → tt_05_bridge，得 残页二
    assert!(st.flag("tt_fengshen_p2"), "云海断碑应得 残页二");

    // 星宿残阵 → 点亮 → 走向封神台
    step(&mut st, &mut deaths, "去星宿残阵");           // → tt_07_stars
    step(&mut st, &mut deaths, "重排二十八宿残碑");      // → tt_07_stars_done，tt_stars_lit
    assert!(st.flag("tt_stars_lit"));
    step(&mut st, &mut deaths, "步向封神台方向");        // → tt_08_fengshen

    // 封神台：核心残页三 → 拼接残页洞悉真相
    step(&mut st, &mut deaths, "查看封神台核心");         // → tt_08_fengshen_core
    step(&mut st, &mut deaths, "抢下封神榜残页");         // → tt_09_truth_wall，得 残页三
    assert!(st.flag("tt_fengshen_p3"), "封神台应得 残页三");
    step(&mut st, &mut deaths, "拼接三张残页");           // → tt_11_fengshen_truth，洞悉真相
    assert!(st.flag("tt_fengshen_truth"), "应洞悉封神真相");

    // 封神台结界 → 迎战一形态
    step(&mut st, &mut deaths, "赴封神台结界");          // → tt_13_fight_gate
    step(&mut st, &mut deaths, "迎战");                  // → start_shenjiang_r1 → tt_14_round_r1

    // 一段战：逐轮重击直到胜利 → 进入东天二皇投影演出
    let mut guard = 0;
    while st.scene_id == "tt_14_round_r1" && guard < 40 {
        guard += 1;
        step(&mut st, &mut deaths, "重击");              // 唯一匹配「重击（强攻）」，不匹配打断/终结
    }
    assert_eq!(st.scene_id, "tt_huang_cast", "一段胜利应进入东天二皇演出；实际 {}", st.scene_id);
    // 圣位红线：演出场景不进引擎战斗模式
    assert!(!matches!(st.mode, Mode::Fight), "东天二皇演出不应触发引擎战斗模式");

    // 二形态
    step(&mut st, &mut deaths, "凝望倒悬王座");          // → start_shenjiang_r2 → tt_15_r2
    step(&mut st, &mut deaths, "开始决战");              // → tt_15_round_r2
    guard = 0;
    while st.scene_id == "tt_15_round_r2" && guard < 40 {
        guard += 1;
        step(&mut st, &mut deaths, "重击");
    }
    assert_eq!(st.scene_id, "tt_17_choice", "二段胜利应进入结局抉择；实际 {}", st.scene_id);

    // 抉择 → 结局（揭封神榜真相）
    step(&mut st, &mut deaths, "揭开封神榜真相");        // → tt_16_ending_unmask
    step(&mut st, &mut deaths, "踏入撤离传送门");        // → settle_A → tt_18_settle
    assert_eq!(st.sp_grade, Some('A'), "高难副本结局应给 A 级支线评级");
    assert!(st.flag("tt_cleared"));

    println!("MAIN LINE OK · points={} ending={} deaths={:?}", st.points, st.scene_id, deaths);
}

/* ---------------- ③ 东天二皇演出红线：投影只演出、不触发战斗 ---------------- */
#[test]
fn tianting_huang_cast_is_playout_not_fight() {
    let mut st = GameState::new();
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "tt_huang_cast", &mut deaths);
    assert_eq!(st.scene_id, "tt_huang_cast");
    // 圣位红线：东天二皇不可战——不初始化 Fight、不进战斗模式
    assert!(st.fight.is_none(), "东天二皇演出不应初始化 Fight（圣位不可战）");
    assert!(!matches!(st.mode, Mode::Fight), "演出场景不应进入战斗模式");
    let scene = wuxian_horror_ch1::scenes::scene("tt_huang_cast").unwrap();
    assert!(scene.fight_id.is_none(), "东天二皇演出场景 fight_id 应为 None");
    println!("TIANTING HUANG-YAJI CAST PLAYLIST OK · 无战斗、无 fight_id");
}