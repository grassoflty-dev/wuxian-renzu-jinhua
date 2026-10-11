//! 《死亡开端 · 旧神遗迹 · 沉没神殿》集成测试（shenmiao 副本）。
//! 依赖主线已把 SHENMIAO_SCENES 并入 scenes::scene()、把 shenmiao_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_SHENMIAO（合并后运行本测试）。
//! 覆盖：① 三层地图可达与坐标合法；② 调度接线；③ 直达 BOSS 的选择驱动相位闪现可击败。
use wuxian_horror_ch1::{engine, scenes, state::{GameState, Mode}};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_SHENMIAO).expect("沉没神殿世界已注册（合并阶段）")
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

/* ---------------- ① 三层地图可达（行宽 40 + 出生点可走 + 表坐标合法） ---------------- */
#[test]
fn shenmiao_map_reachable() {
    let w = world();
    assert_eq!(w.id, wuxian_horror_ch1::worlds::WORLD_SHENMIAO);
    assert_eq!(w.floors.len(), 3, "沉没神殿应为三层");
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
fn shenmiao_dispatch_wired() {
    assert!(scenes::scene("sm_00").is_some(), "sm_00 应在 scenes::scene() 可解析");
    assert!(scenes::scene("sm_37_ending").is_some(), "sm_37_ending（真相结局）应可解析");
    assert!(scenes::scene("sm_40_card").is_some(), "sm_40_card（结算卡片）应可解析");
    assert!(scenes::fight_cfg("sm_oldgod_spawn").is_some(), "sm_oldgod_spawn（旧神眷属）应可解析");
    println!("DISPATCH WIRED OK");
}

/* ---------------- ③ 直达 BOSS：选择驱动相位闪现可确定性击败 ---------------- */
#[test]
fn shenmiao_boss_deterministic() {
    let mut st = GameState::new();
    st.hp = 2000;
    st.san = 100;
    engine::goto(&mut st, "sm_00", &mut Vec::new());
    assert_eq!(st.scene_id, "sm_00");

    // 序 → F1 hub
    step(&mut st, "在石阶上站稳，向里走");     // → sm_10_f1
    // 拾旧神祭器 → 开倒置门环 → 进 F2
    step(&mut st, "逆泳之池");                 // → sm_14_pool
    step(&mut st, "捞出旧神祭器");              // it_shenmiao_reliquary → sm_10_f1
    assert!(st.inventory.iter().any(|i| i == "it_shenmiao_reliquary"), "应得旧神祭器");
    step(&mut st, "向倒置门扉游去");            // → sm_16_gate
    step(&mut st, "（踏入颠倒回廊）");          // → sm_20_f2
    // F2 → F3 沉眠神龛
    step(&mut st, "绕过无面石像");              // → sm_26_enter_f3
    step(&mut st, "（踏进沉眠神龛）");          // → sm_30_f3
    // 直面 BOSS
    step(&mut st, "【直面旧神眷属】");          // → sm_35_boss
    step(&mut st, "【直面开战】");              // start_spawn → sm_35_boss_round
    assert!(st.flag("sm_spawn_start"), "应置 BOSS 开始 flag");

    // 相位闪现决战：固定数值，确定性。
    // HP200；重击 34（未狂暴时被反噬 16），防御免伤；≤90 后狂暴（反噬 26）。
    step(&mut st, "重击");                      // 200→166
    step(&mut st, "重击");                      // →132
    step(&mut st, "防御");                      // 免伤
    step(&mut st, "重击");                      // →98
    step(&mut st, "重击");                      // →64（本回合结束后仍未狂暴：rage 在回合首判定）
    step(&mut st, "重击");                      // →30，回合首判定 hp≤90 → 进入狂暴相位
    assert!(st.fight.as_ref().map(|f| f.raged).unwrap_or(false), "hp≤90 后应进入狂暴相位");
    step(&mut st, "重击");                      // →0 → 胜
    assert!(st.flag("sm_spawn_dead"), "击败后应置 sm_spawn_dead");
    assert!(st.inventory.iter().any(|i| i == "it_shenmiao_ash"), "应得旧神残灰");

    // 安魂 → 真相结局 → 结算卡片
    step(&mut st, "把祭器放进空神龛");          // → sm_37_ending
    step(&mut st, "顺流而上，离开倒流之海");    // sm_route_settle → sm_40_card
    assert!(matches!(st.mode, Mode::AwaitCard(_)), "应落在结算卡片");
    assert!(st.sp_grade.is_some(), "结算应写评级");
    println!("BOSS DETERMINISTIC OK · pts={} grade={:?} scene={}", st.points, st.sp_grade, st.scene_id);
}