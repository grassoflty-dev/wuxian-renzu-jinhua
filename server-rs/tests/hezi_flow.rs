//! 《异位面 · 倒影界》集成测试（hezi 副本，简版）。
//! 依赖主线已把 HEZI_SCENES 并入 scenes::scene()、把 hezi_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_HEZI。测试覆盖三层可达、调度接线与开放结局可到达。
use wuxian_horror_ch1::{engine, scenes, state::{GameState, Mode}};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_HEZI).expect("异位面·倒影界世界已注册（恢复阶段）")
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

/* ---------------- ① 三层地图可达（行宽 40 + 出生点可走） ---------------- */
#[test]
fn hezi_map_reachable() {
    let w = world();
    assert_eq!(w.id, wuxian_horror_ch1::worlds::WORLD_HEZI);
    assert_eq!(w.floors.len(), 3, "倒影界应为 3 层");
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    let (sx, sy) = w.spawn();
    assert!(worlds::walkable(w, 0, sx, sy), "出生点 ({sx},{sy}) 应可走动");
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "调查点 {}@L{}({},{}) 不可走动", p.id, p.floor + 1, p.x, p.y);
    }
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.to_floor, pt.tx, pt.ty), "传送门 {} 落点@L{}({},{}) 不可走动", pt.id, pt.to_floor + 1, pt.tx, pt.ty);
    }
    println!("MAP REACHABLE OK · floors={} points={} portals={} spawn=({sx},{sy})", w.floors.len(), w.points.len(), w.portals.len());
}

/* ---------------- ② 调度接线 ---------------- */
#[test]
fn hezi_dispatch_wired() {
    assert!(scenes::scene("hz_00").is_some(), "hz_00 应在 scenes::scene() 可解析");
    assert!(scenes::scene("hz_40_ending").is_some(), "hz_40_ending（开放结局）应可解析");
    assert!(scenes::fight_cfg("hz_gentle_herd").is_some(), "hz_gentle_herd 应在 scenes::fight_cfg() 可解析");
    assert!(scenes::fight_cfg("hz_guardian").is_some(), "hz_guardian（界域守护兽）应可解析");
    println!("DISPATCH WIRED OK");
}

/* ---------------- ③ 开放结局可到达（守护兽友好交流） ---------------- */
#[test]
fn hezi_open_ending() {
    let mut st = GameState::new();
    st.hp = 2000;
    st.san = 100;
    engine::goto(&mut st, "hz_00", &mut Vec::new());
    assert_eq!(st.scene_id, "hz_00");

    // 序 → F1 hub
    step(&mut st, "慢些走入");            // → hz_01
    // 拾棱光石 → 开微光幕 → 进 F2 荧光石林
    step(&mut st, "棱光石滩");            // → hz_04_pebble
    step(&mut st, "拾起棱光石");           // it_yijie_crystal → hz_01
    assert!(st.inventory.iter().any(|i| i == "it_yijie_crystal"), "应得棱光石");
    step(&mut st, "走向石林（需棱光石）");   // → hz_04_gate
    step(&mut st, "（踏入荧光石林）");       // → hz_10_arrive
    // F2 → F3 倒悬星海
    step(&mut st, "走向倒悬星海");          // → hz_20_arrive
    // 面向守护兽 → 友好交流
    step(&mut st, "面向界域守护兽");        // → hz_30_guardian
    step(&mut st, "朝它挥手问好");          // → hz_31_friendly
    step(&mut st, "收下异界标本");           // guardian_friendly → hz_32_guardian_peace
    assert!(st.flag("hz_guardian_peace"), "应置守护兽友好的 flag");
    step(&mut st, "（转身，继续你的漫游或收尾）"); // → hz_40_ending
    // 开放结局
    step(&mut st, "带走标本");             // → hz_41_exit
    assert!(st.flag("hz_ending_done"), "开放结局应置结束 flag");
    step(&mut st, "（这一程，也到说再见的时候了）"); // Route::Dyn(settle) → hz_42_card
    assert!(matches!(st.mode, Mode::AwaitCard(_)), "应落在探界结算卡片");
    assert!(st.sp_grade.is_some(), "探索结算应写评级");
    println!("OPEN ENDING OK · pts={} grade={:?} scene={}", st.points, st.sp_grade, st.scene_id);
}