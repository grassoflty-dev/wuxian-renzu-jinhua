//! 《咒怨》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 ZHOUYUAN_SCENES 并入 scenes::scene()、把 zhouyuan_figths() 并入 scenes::fight_cfg()，
//! 保证 engine::goto / engine::choose / engine::fight_actions 能解析 zy_* 场景与 b_* 战斗（见 tools/design/zhouyuan_impl_log.md ★外部依赖）。
//! 测试内容：①地图可达性（出生点走遍 F1 关键调查点）②主线链→BOSS 胜利→点数增加 ③BOSS 诅咒叠层→二重死 ④死亡档案触发。
use wuxian_horror_ch1::{engine, state::GameState};
use wuxian_horror_ch1::worlds;

fn pick(st: &GameState, keyword: &str) -> i32 {
    let scene = wuxian_horror_ch1::scenes::scene(&st.scene_id).expect("scene");
    // 若当前场景通过合并扩展可被解析但其 choices 应来自本世界表；bio 表与 zy_ 表二选一
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
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} cur={}/{}{} dead={:?})",
        st.scene_id, st.hp, st.san, st.points,
        st.flag("zy_curse_1") as u8, st.flag("zy_curse_2") as u8, st.flag("zy_curse_3") as u8, st.dead_team);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

/// 战斗：post-merge 后 fight_cfg 能解析 b_*；此处针对剧情中的引擎战斗（若有）

// 直接构造一个处于 Boss 领域回合的状态（用于诅咒叠层 / 狂暴测试）
fn enter_boss_round(st: &mut GameState, boss_hp: i32, curse_starts: usize) {
    // 主线合并后可通过 fight_cfg 拿到 b_kayako；测试里手工构建 Fight 最关键字段
    st.fight = Some(wuxian_horror_ch1::state::Fight {
        id: "b_kayako".to_string(),
        name: "伽椰子 · 怨念之源".to_string(),
        hp: boss_hp,
        max_hp: 140,
        dmg: (12, 18),
        reward: 500,
        reward_why: "击败伽椰子本体".to_string(),
        raged: boss_hp <= 56,
        rage_at: Some(40),
        guard_turn: false,
        pending_log: vec![],
    });
    for i in 1..=curse_starts {
        if i == 1 { st.set_flag("zy_curse_1"); }
        if i == 2 { st.set_flag("zy_curse_2"); }
        if i == 3 { st.set_flag("zy_curse_3"); }
    }
    let mut deaths = vec![];
    engine::goto(st, "zy_boss_round", &mut deaths);
    assert_eq!(st.scene_id, "zy_boss_round");
}

/* ---------------- ① 地图可达性 ---------------- */
#[test]
fn zhouyuan_f1_map_reachable() {
    let w = wuxian_horror_ch1::worlds::find_world(wuxian_horror_ch1::worlds::WORLD_ZHOUYUAN)
        .expect("咒怨世界已注册（合并阶段）");

    // 断言三层地图每行恰 40 字符（防作者笔误导致格子偏移成墙）
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }

    // 出生点应为玄关入口 (7,24)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (7, 24), "出生点应为玄关入口 (7,24)");
    // 关键点本身必须可走动（非墙）
    for p in w.points {
        if p.floor == 0 {
            assert!(worlds::walkable(w, 0, p.x, p.y), "F1 点 {}@({},{}) 不可走动", p.id, p.x, p.y);
        }
    }
    // BFS 验证 F1 全层连通（出生点可达所有 F1 可走动格 → 关键调查点可达）
    let reachable = flood_fill(w, 0, sx, sy);
    for p in w.points {
        if p.floor == 0 {
            let key = (p.x, p.y);
            assert!(reachable.contains(&key), "F1 点 {}@({},{}) 从出生点不可达", p.id, p.x, p.y);
        }
    }
    println!("F1 map reachability OK · reachable={} pts_on_floor0={}",
        reachable.len(), w.points.iter().filter(|p| p.floor == 0).count());
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

/* ---------------- ② 主线链：开场 → 调查 → BOSS 胜利 → 点数增加 ---------------- */
#[test]
fn zhouyuan_main_line_boss_win() {
    let mut st = GameState::new();
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "zy_01", &mut deaths);
    assert_eq!(st.scene_id, "zy_01");

    step(&mut st, &mut deaths, "检查那双儿童雨鞋");   // zy_01 → zy_02，支线 zy_shoe_checked +10
    assert!(st.flag("zy_shoe_checked"));

    step(&mut st, &mut deaths, "佛龛");                 // zy_02 → zy_03_butsudan
    step(&mut st, &mut deaths, "取走供品碟里的猫粮");    // 领取 item_cat_food
    assert!(st.inventory.iter().any(|i| i == "item_cat_food"));

    step(&mut st, &mut deaths, "继续探索");             // zy_03_back → zy_02
    step(&mut st, &mut deaths, "直接上二楼");           // → zy_04（楼梯口俊雄）

    // 俊雄引路：走安全线（持有猫粮）
    step(&mut st, &mut deaths, "跟猫走");               // → zy_11_diary（安全落地阁楼旧皮箱）
    assert!(st.flag("zy_cat_trust"));
    assert!(st.flag("zy_cat_safe"));

    // 阁楼：取日记（真相）
    step(&mut st, &mut deaths, "记住真相");             // → zy_12
    assert!(st.flag("zy_diary_truth"));

    // 下地下室（G3 已满足真相）
    step(&mut st, &mut deaths, "下地下室");             // → zy_13_basement
    step(&mut st, &mut deaths, "走向结界圈");           // → zy_14_well
    step(&mut st, &mut deaths, "迎向结界核心");         // → zy_15_fight

    // 决战：先布置仪式（需佛珠 + 真相）→ 直接决战
    // 为简化主线，这里直接用「直接决战」进入 BOSS；仪式( +200)在独立测试中覆盖
    step(&mut st, &mut deaths, "直接决战");             // start_kayako → zy_boss_round

    let pts_before = st.points;
    // 逐回合强攻，直到 BOSS 结算
    let mut guard = 0;
    while st.scene_id == "zy_boss_round" && guard < 30 {
        guard += 1;
        step(&mut st, &mut deaths, "重击");
    }
    assert!(st.scene_id.starts_with("zy_16_win"), "BOSS 结算应在 zy_16_win，实际 {}", st.scene_id);
    // 无仪式 → 强杀结算卡片（不 +200 / 无 item / 无 sp_grade）
    assert!(st.flag("zy_strongkill"));
    assert!(st.points > pts_before, "胜利后点数应增加");
    assert_eq!(st.sp_grade, None, "强杀无支线评级");
    assert!(!st.inventory.iter().any(|i| i == "item_talisman"), "强杀无驱邪符");

    println!("MAIN LINE OK · points={} pts_before={} curses={}{}{}",
        st.points, pts_before, st.flag("zy_curse_1") as u8, st.flag("zy_curse_2") as u8, st.flag("zy_curse_3") as u8);
    println!("DEATHS: {:?}", deaths);
}

/* ---------------- ③ 诅咒叠层 → 二重死 ---------------- */
#[test]
fn zhouyuan_curse_triple_is_death() {
    let mut st = GameState::new();
    st.hp = 100; st.san = 100;
    // 本场景「狂暴」= BOSS HP≤56（狂暴阈值 40%=56，见 route_boss_attack / boss_raged）。
    // 把手动构造的 BOSS 血量定在阈值 56：已有两层诅咒 → 本回「重击(34-46)」先攒第 3 层，
    // 且 BOSS 不会被一回击倒（余血 10-22 > 0），于是诅咒叠满 → 二重死。
    enter_boss_round(&mut st, 56, 2);
    assert!(boss_raged_flag(&st));
    let mut deaths = vec![];
    // 强攻：BOSS HP 56 不会在 1 回被击倒（34-46 伤害余血 10-22）→ 攒满 3 层 → 二重死
    let idx = pick(&st, "重击");
    engine::choose(&mut st, idx, &mut deaths);
    assert!(st.flag("zy_curse_3"), "第三层诅咒应已叠满");
    assert_eq!(st.scene_id, "zy_17_lose_curse", "诅咒叠满应进入二重死；实际 {}", st.scene_id);
    assert!(!deaths.is_empty(), "应记录死亡档案");
    let tag = deaths.iter().find(|(t, _)| t.contains("二重死"));
    assert!(tag.is_some(), "死亡档案应含 d3 二重死：{:?}", deaths);
    println!("CURSE DEATH OK · deaths={:?}", deaths);
}

fn boss_raged_flag(st: &GameState) -> bool {
    st.fight.as_ref().map(|f| f.hp <= 56 && f.raged).unwrap_or(false)
}

/* ---------------- ④ 死亡档案（壁纸强开 / SAN 归零） ---------------- */
#[test]
fn zhouyuan_wallpaper_death_archive() {
    let mut st = GameState::new();
    st.san = 50;
    let mut deaths = vec![];
    engine::goto(&mut st, "zy_05_wallpaper", &mut deaths);
    assert_eq!(st.scene_id, "zy_05_wallpaper");
    let idx = pick(&st, "夺路而逃");
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(st.scene_id, "zy_17_lose_wall", "强开壁纸逃跑应死于 d2");
    assert!(!deaths.is_empty());
    assert!(deaths.iter().any(|(t, _)| t.contains("壁纸")), "死亡档案应含 d2 壁纸里的人形");
    println!("WALLPAPER DEATH OK · deaths={:?}", deaths);
}

/* ---------------- ⑤ 战斗表完整性（战斗数 / BOSS 数值） ---------------- */
#[test]
fn zhouyuan_fight_table_complete() {
    let fights = wuxian_horror_ch1::scenes_zhouyuan::zhouyuan_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in ["b_servant", "b_shade", "b_shade2", "b_toshio", "b_shade3", "b_kayako_shade", "b_kayako"] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    let kayako = fights.iter().find(|(k, _)| *k == "b_kayako").map(|(_, c)| c).unwrap();
    assert_eq!(kayako.hp, 140, "BOSS HP 应为 140");
    assert_eq!(kayako.reward, 500, "BOSS 奖励基准 500");
    // 狂暴 40%：HP≤56 触发
    assert_eq!(kayako.rage_at, Some(40));
    println!("FIGHT TABLE OK · {} 场", fights.len());
}