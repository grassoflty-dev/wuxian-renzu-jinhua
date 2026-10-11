//! 《无限曙光 · 我们是传奇》位面 · 夜城·孤独之城 任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 YECHENG_SCENES 并入 scenes::scene()、把 yecheng_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_YECHENG（id="yecheng"），保证 engine::goto / engine::choose 能解析 yc_* 场景
//! 与 yc_* 战斗（见 tools/design/yecheng_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① 地图完整性（3 层各 26×40；出生点/spawn；调查点与传送门可走）
//!   ② 白日线：开场→安全屋调查→无线电/样本（it_yc_blood）→入夜
//!   ③ 战斗表：夜魔游荡者/前锋/BOSS 进化体之王数值断言 + 分发闭环
//!   ④ 主线链：入夜→遭遇战→实验室取样本→深入夜色→巢心决战夜王→结局（sp_grade D）
use wuxian_horror_ch1::worlds;
use wuxian_horror_ch1::{engine, state::GameState, state::Mode};

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_YECHENG)
        .expect("夜城世界已注册（合并阶段）")
}

fn pick(st: &GameState, keyword: &str) -> i32 {
    let scene = wuxian_horror_ch1::scenes::scene(&st.scene_id).expect("scene");
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

fn step(st: &mut GameState, deaths: &mut Vec<(&'static str, &'static str)>, keyword: &str) {
    let idx = pick(st, keyword);
    engine::choose(st, idx, deaths);
    println!(
        "STEP [{keyword}] → {} (hp={} san={} pts={} night={} deep={} res={} bossdown={})",
        st.scene_id,
        st.hp,
        st.san,
        st.points,
        st.flag("yc_nightfall") as u8,
        st.flag("yc_deep_night") as u8,
        st.flag("yc_research") as u8,
        st.flag("yc_boss_down") as u8
    );
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

/// 驱策原生 FightCfg 战：连续攻击（index 0 = attack）直到脱离战斗态。
fn fight_to_win(st: &mut GameState, deaths: &mut Vec<(&'static str, &'static str)>, guard: usize) {
    let mut g = 0;
    while matches!(st.mode, Mode::Fight) && g < guard {
        g += 1;
        engine::choose(st, 0, deaths);
    }
    assert!(
        g < guard,
        "原生战斗未在 guard 轮内结束 scene={} mode={:?}",
        st.scene_id,
        st.mode
    );
}

/// 重置为健康前期状态：高 HP + 高 SAN + 军刀（方便连续遭遇战与 BOSS 战）。
fn fresh(st: &mut GameState) {
    st.hp = 500;
    st.san = 100;
    st.weapon = Some(wuxian_horror_ch1::state::Weapon::Sword);
}

/* ---------------- ① 地图完整性 ---------------- */
#[test]
fn yecheng_map_integrity() {
    let w = world();
    assert_eq!(w.floors.len(), 3, "夜城应为 3 层");
    for (fi, map) in w.floors.iter().enumerate() {
        assert_eq!(map.len(), 26, "floor{fi} rows != 26");
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (8, 4), "出生点应为 L1 白昼安全屋 (8,4)");
    // 所有调查点必须可走；传送门起点/落点必须落在地板
    for p in w.points {
        assert!(
            worlds::walkable(w, p.floor, p.x, p.y),
            "调查点 {}@L{}:({},{}) 不可走动",
            p.id,
            p.floor + 1,
            p.x,
            p.y
        );
    }
    for pt in w.portals {
        assert!(
            worlds::walkable(w, pt.floor, pt.x, pt.y),
            "传送门 {}@L{}:({},{}) 不可走动",
            pt.id,
            pt.floor + 1,
            pt.x,
            pt.y
        );
        assert!(
            worlds::walkable(w, pt.to_floor, pt.tx, pt.ty),
            "传送门 {} 落点 L{}:({},{}) 不可走动",
            pt.id,
            pt.to_floor + 1,
            pt.tx,
            pt.ty
        );
    }
    println!(
        "MAP INTEGRITY OK · floors={} points={} portals={} enemies={} zones={}",
        w.floors.len(),
        w.points.len(),
        w.portals.len(),
        w.enemies.len(),
        w.zones.len()
    );
}

/* ---------------- ② 场景 / 战斗表存在性 ---------------- */
#[test]
fn yecheng_scenes_exist() {
    for id in [
        "yc_00",
        "yc_l1_hub",
        "yc_l2_hub",
        "yc_l3_hub",
        "yc_20_dusk",
        "yc_f_slave",
        "yc_f_hunter",
        "yc_boss_intro",
        "yc_boss_round",
        "yc_boss_win",
        "yc_settle",
        "yc_42_card",
        "yc_50_death",
    ] {
        assert!(
            wuxian_horror_ch1::scenes::scene(id).is_some(),
            "场景 {id} 应存在"
        );
    }
}

#[test]
fn yecheng_fights_exist() {
    assert!(
        wuxian_horror_ch1::scenes::fight_cfg("yc_e_slave").is_some(),
        "fight yc_e_slave 应存在"
    );
    assert!(
        wuxian_horror_ch1::scenes::fight_cfg("yc_e_hunter").is_some(),
        "fight yc_e_hunter 应存在"
    );
    assert!(
        wuxian_horror_ch1::scenes::fight_cfg("yc_boss").is_some(),
        "fight yc_boss 应存在"
    );
    assert_eq!(
        wuxian_horror_ch1::scenes::fight_cfg("yc_e_slave")
            .unwrap()
            .hp,
        55,
        "夜魔·游荡者 HP 应为 55"
    );
    assert_eq!(
        wuxian_horror_ch1::scenes::fight_cfg("yc_e_hunter")
            .unwrap()
            .hp,
        80,
        "夜魔·前锋 HP 应为 80"
    );
    assert_eq!(
        wuxian_horror_ch1::scenes::fight_cfg("yc_boss").unwrap().hp,
        150,
        "夜魔之王 HP 应为 150"
    );
}

/// 自洽：yecheng_figths() 表内所有 fight id 都能被全局 fight_cfg 解析（分发闭环）
#[test]
fn yecheng_self_consistent() {
    for (id, _c) in wuxian_horror_ch1::scenes_yecheng::yecheng_figths() {
        assert!(
            wuxian_horror_ch1::scenes::fight_cfg(id).is_some(),
            "fight {id} 分发闭环"
        );
    }
    // YECHENG_SCENES 表内所有带 fight_id 的场景必须能被全局 fight_cfg 解析
    for s in wuxian_horror_ch1::scenes_yecheng::YECHENG_SCENES {
        if let Some(fid) = s.fight_id {
            assert!(
                wuxian_horror_ch1::scenes::fight_cfg(fid).is_some(),
                "场景 {} 的 fight_id {fid} 分发闭环",
                s.id
            );
        }
    }
}

/* ---------------- ③ 战斗表完备 ---------------- */
#[test]
fn yecheng_fight_table_complete() {
    let fights = wuxian_horror_ch1::scenes_yecheng::yecheng_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in ["yc_e_slave", "yc_e_hunter", "yc_boss"] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    let boss = fights
        .iter()
        .find(|(k, _)| *k == "yc_boss")
        .map(|(_, c)| c)
        .unwrap();
    assert_eq!(boss.hp, 150, "夜魔之王 HP 应为 150");
    assert_eq!(boss.reward, 400, "夜魔之王奖励 400");
    assert_eq!(boss.rage_at, Some(60), "夜魔之王狂暴阈 60");
    println!("FIGHT TABLE OK · fights={}", fights.len());
}

/* ---------------- ④ 主线链：白日 → 入夜 → 遭遇战 → 样本 → 巢心决战 → 结局 ---------------- */
#[test]
fn yecheng_main_line_night() {
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "yc_00", &mut deaths);
    assert_eq!(st.scene_id, "yc_00");
    let base_pts = st.points;

    // 白日线：安全屋调查（无线电留痕）
    step(&mut st, &mut deaths, "环顾安全屋"); // → yc_l1_hub
    step(&mut st, &mut deaths, "坏掉的无线电"); // → yc_01_radio
    step(&mut st, &mut deaths, "修好无线电"); // yc_radio_fixed → yc_l1_hub
    assert!(st.flag("yc_radio_fixed"), "应置 yc_radio_fixed");

    // 入夜（落日斜巷过渡 → 置 yc_nightfall）
    step(&mut st, &mut deaths, "走出安全屋"); // → yc_20_dusk
    step(&mut st, &mut deaths, "攥紧手电走进夜色"); // go_night → yc_l2_hub · yc_nightfall
    assert!(st.flag("yc_nightfall"), "应置 yc_nightfall");

    // 遭遇战：夜魔游荡者（原生 fight）
    step(&mut st, &mut deaths, "暗处响动"); // → yc_f_slave
    fight_to_win(&mut st, &mut deaths, 60);
    assert_eq!(
        st.scene_id, "yc_l2_hub",
        "游荡者胜后应回 yc_l2_hub，实际 {}",
        st.scene_id
    );

    // 实验室取样本（it_yc_blood + yc_research）
    step(&mut st, &mut deaths, "研究员地下实验室"); // → yc_08_lab
    step(&mut st, &mut deaths, "取走夜魔血液样本"); // → yc_l2_hub · yc_research + it_yc_blood
    assert!(st.flag("yc_research"), "应置 yc_research");
    assert!(
        st.inventory.iter().any(|i| i == "it_yc_blood"),
        "应有夜魔血液样本 it_yc_blood"
    );

    // 深入夜色 → L3 巢心
    step(&mut st, &mut deaths, "下夜魔巢"); // go_deep_night → yc_l3_hub
    assert!(st.flag("yc_deep_night"), "应置 yc_deep_night");
    assert_eq!(st.scene_id, "yc_l3_hub");

    // 决战夜魔之王（选择驱动 BOSS）
    step(&mut st, &mut deaths, "决战夜魔之王"); // → yc_boss_intro
    step(&mut st, &mut deaths, "出手破夜王"); // start_boss → yc_boss_round
    let mut guard = 0;
    while st.scene_id == "yc_boss_round" && guard < 80 {
        guard += 1;
        step(&mut st, &mut deaths, "重击夜王");
    }
    assert_eq!(
        st.scene_id, "yc_boss_win",
        "决战胜利应回 yc_boss_win，实际 {}",
        st.scene_id
    );
    assert!(st.flag("yc_boss_down"), "应置 yc_boss_down");
    assert!(
        st.inventory.iter().any(|i| i == "it_yc_fang"),
        "应有战利 it_yc_fang"
    );

    // 结算
    step(&mut st, &mut deaths, "对照废墟记录"); // → yc_settle
    step(&mut st, &mut deaths, "掀巢之路"); // route_settle → yc_42_card sp_grade=D
    assert_eq!(st.sp_grade, Some('D'), "掀巢结算应写 sp_grade=D");
    assert!(
        st.points > base_pts + 60,
        "结束后点数应包含样本+战利等加成，delta={}",
        st.points - base_pts
    );
    println!(
        "MAIN LINE OK · seg pts delta {} · deaths={:?}",
        st.points - base_pts,
        deaths
    );
}
