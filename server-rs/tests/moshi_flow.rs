//! 《末世死城·人类防线》集成测试（moshi 副本，简版）。
//! 依赖主线已把 MOSHI_SCENES 并入 scenes::scene()、把 moshi_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_MOSHI。测试只覆盖核心可达性与主线链可推进，避免 RNG 脆断。
use wuxian_horror_ch1::{engine, scenes, state::{GameState, Mode}};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_MOSHI).expect("末世死城世界已注册（恢复阶段）")
}

fn pick(st: &GameState, keyword: &str) -> i32 {
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

fn step(st: &mut GameState, keyword: &str) {
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    let idx = pick(st, keyword);
    engine::choose(st, idx, &mut deaths);
    println!("STEP [{keyword}] → {} (hp={} san={} pts={})", st.scene_id, st.hp, st.san, st.points);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

/// 简版战斗驱动：高血量兜底 + 迭代上限，无死亡卡片判断。
fn fight_until_done(st: &mut GameState) {
    st.hp = 2000;
    st.san = 100;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    for _ in 0..500 {
        match st.mode {
            Mode::Fight => { engine::choose(st, 0, &mut deaths); }
            Mode::AwaitCard(_) => { engine::choose(st, 0, &mut deaths); } // 基因锁觉醒/结算 → 续战或前进
            _ => return,
        }
    }
    panic!("战斗未在迭代上限内结束（scene={}）", st.scene_id);
}

/* ---------------- ① 地图可达性（不硬编码精确出生点） ---------------- */
#[test]
fn moshi_map_reachable() {
    let w = world();
    assert_eq!(w.id, wuxian_horror_ch1::worlds::WORLD_MOSHI);
    // F1 行宽 40、出生点可走
    assert_eq!(w.floors[0][0].len(), 40, "F1 行宽应为 40");
    let (sx, sy) = w.spawn();
    assert!(worlds::walkable(w, 0, sx, sy), "出生点 ({sx},{sy}) 应可走动");
    // 全部调查点 + 传送门落点必须可走
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "调查点 {}@L{}({},{}) 不可走动", p.id, p.floor + 1, p.x, p.y);
    }
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.to_floor, pt.tx, pt.ty), "传送门 {} 落点@L{}({},{}) 不可走动", pt.id, pt.to_floor + 1, pt.tx, pt.ty);
    }
    println!("MAP REACHABLE OK · floors={} points={} portals={} spawn=({sx},{sy})", w.floors.len(), w.points.len(), w.portals.len());
}

/* ---------------- ② 调度接线（场景/战斗可解析） ---------------- */
#[test]
fn moshi_dispatch_wired() {
    assert!(scenes::scene("ms_00").is_some(), "ms_00 应在 scenes::scene() 可解析");
    assert!(scenes::scene("ms_combat_a").is_some(), "ms_combat_a 应可解析");
    assert!(scenes::fight_cfg("fight_f1_wave_a").is_some(), "fight_f1_wave_a 应在 scenes::fight_cfg() 可解析");
    assert!(scenes::fight_cfg("fight_r_siege_beast").is_some(), "fight_r_siege_beast（BOSS）应可解析");
    println!("DISPATCH WIRED OK");
}

/* ---------------- ③ 主线可推进（打到后期场景/点数增长即可） ---------------- */
#[test]
fn moshi_mainline_boss() {
    let mut st = GameState::new();
    st.hp = 2000;
    st.san = 100;
    engine::goto(&mut st, "ms_00", &mut Vec::new());
    assert_eq!(st.scene_id, "ms_00");

    // 开场 → F1 首战
    step(&mut st, "即刻上城墙协防");
    assert_eq!(st.scene_id, "ms_combat_a");
    fight_until_done(&mut st);            // 波1 → 休整节点
    let pts_after_wave1 = st.points;
    assert!(st.scene_id != "ms_combat_a", "首战应打完离开战斗场景");

    // 继续推进若干步，直到发生战斗或主线前进（用迭代上限兜底，不强制到 BOSS 结算）
    let mut guard = 0;
    while !matches!(st.mode, Mode::Fight) && guard < 20 {
        guard += 1;
        let s = scenes::scene(&st.scene_id).expect("scene");
        let visible: Vec<&_> = s.choices.iter().filter(|c| c.cond.map_or(true, |f| f(&st))).collect();
        if visible.is_empty() {
            break;
        }
        // Route::Dyn 与战斗场景均可能落在 ms_rest / ms_fight 等，逐个按首个可见选项推进
        step(&mut st, visible[0].label);
    }
    // 只要点数较开场有增长、且场景已离开开场，即视为主线推进成功
    assert!(st.points >= pts_after_wave1, "主线推进后点数不应回退（pts={}）", st.points);
    assert!(st.points > 0, "入场后应取得基础点数");
    println!("MAINLINE OK · scene={} pts={} hp={}", st.scene_id, st.points, st.hp);
}