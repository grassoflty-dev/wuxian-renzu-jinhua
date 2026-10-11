//! 《星河战队 · 脑虫巢穴》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 XINGHE_SCENES 并入 scenes::scene()、把 xinghe_figths() 并入 scenes::fight_cfg()、
//! 并登记 WORLD_XINGHE / `mod scenes_xinghe`，保证 engine::goto / engine::choose / engine::fight_actions
//! 能解析 xh_* 场景与 xh_* 战斗（见 tools/design/xinghe_impl_log.md ★外部依赖）。
//! 测试内容：①3 层地图可达性 + 单向传送闭环 ②主线链→多波次→脑虫战胜利→sp_grade B
//! ③多波次增员链（波次 fight 存在 + 逐波推进清场）。
//! 注：脑虫战为「选择驱动遭遇」，用 step 点选项驱动；普通敌人/波次用 drive_fight（引擎回合制）。
use wuxian_horror_ch1::{engine, state::{GameState, Mode}};

fn pick(st: &GameState, keyword: &str) -> i32 {
    let scene = wuxian_horror_ch1::scenes::scene(&st.scene_id).expect("scene");
    let visible: Vec<_> = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(st))).collect();
    for (i, c) in visible.iter().enumerate() {
        if c.label.contains(keyword) {
            return i as i32;
        }
    }
    panic!("scene {} 未找到含「{}」的选项；可见: {:?}",
        st.scene_id, keyword, visible.iter().map(|c| c.label).collect::<Vec<_>>());
}

fn step(st: &mut GameState, deaths: &mut Vec<(&'static str, &'static str)>, keyword: &str) {
    let idx = pick(st, keyword);
    engine::choose(st, idx, deaths);
    assert!(st.hp >= 0 && st.san >= 0, "数值越界 scene={}", st.scene_id);
}

/// 驱动当前战场直到战斗结束（引擎 Fight 模式）。
/// 途中出现「基因锁觉醒」覆盖卡（__resume_fight__）会自动点「睁眼」回到战场，避免战斗被觉醒打断而卡死。
fn drive_fight(st: &mut GameState, deaths: &mut Vec<(&'static str, &'static str)>) {
    for _ in 0..300 {
        match &st.mode {
            Mode::AwaitCard(c) => {
                // 基因锁觉醒卡：按钮路由为 __resume_fight__，点「睁眼」续战；死/结算则返回
                let resume_idx = c.buttons.iter().position(|(_, r)| r == "__resume_fight__");
                match resume_idx {
                    Some(i) => { engine::choose(st, i as i32, deaths); }
                    None => return,
                }
            }
            Mode::Fight => {
                let acts = engine::fight_actions(st);
                let idx = if let Some(i) = acts.iter().position(|a| *a == "finisher") { i } else { 0 };
                engine::choose(st, idx as i32, deaths);
            }
            _ => return,
        }
    }
    panic!("战斗未在限定回合内结束");
}

/// 若当前在「休息回复点」，点休整回满血并推进到下一战
fn step_past_rest(st: &mut GameState, deaths: &mut Vec<(&'static str, &'static str)>) {
    while st.scene_id.starts_with("xh_rest_") {
        let idx = pick(st, "休整回满血");
        engine::choose(st, idx, deaths);
    }
}

fn fresh(st: &mut GameState) {
    st.hp = 500;
    st.san = 100;
    // 星河为阵地战设定：近卫斧（Drive 波次最强 DPS），驱动 Fight 波次稳定清场
    st.weapon = Some(wuxian_horror_ch1::state::Weapon::Axe);
}

/* ---------------- ① 地图可达性 + 单向闭环 ---------------- */
#[test]
fn xinghe_map_reachable() {
    let w = wuxian_horror_ch1::worlds::find_world(wuxian_horror_ch1::worlds::WORLD_XINGHE)
        .expect("世界已注册（合并阶段）");
    // 三层地图每行恰 40 字符
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    assert_eq!(w.floors.len(), 3, "星河应为 3 层");
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (5, 14), "出生点应为 L1 登陆场 (5,14)");
    // 全部调查点/敌人/NPC/Zone/传送门落点坐标可走动
    for p in w.points {
        assert!(wuxian_horror_ch1::worlds::walkable(w, p.floor, p.x, p.y),
            "{}-调查点不可走动", p.id);
    }
    for e in w.enemies {
        assert!(wuxian_horror_ch1::worlds::walkable(w, e.floor, e.x, e.y),
            "{}-敌人不可走动", e.id);
    }
    for z in w.zones {
        assert!(wuxian_horror_ch1::worlds::walkable(w, z.floor, z.x, z.y),
            "{}-Zone 不可走动", z.id);
    }
    for pt in w.portals {
        assert!(wuxian_horror_ch1::worlds::walkable(w, pt.floor, pt.x, pt.y),
            "{}-传送门不可走动", pt.id);
    }
    for g in w.gates {
        assert!(wuxian_horror_ch1::worlds::walkable(w, g.floor, g.x, g.y),
            "{}-门禁不可走动", g.id);
    }
    // 单向闭环：仅 p_xh_4 是回跳门（to_floor < floor）；前向 p_xh_1/2/3 层级递增
    let back: Vec<&str> = w.portals.iter().filter(|p| p.to_floor < p.floor).map(|p| p.id).collect();
    assert_eq!(back.len(), 1, "应仅一扇回跳门；实际 {:?}", back);
    assert_eq!(back[0], "p_xh_4", "唯一回跳应为 p_xh_4 塌井");
    let fwd: Vec<&str> = w.portals.iter().filter(|p| p.to_floor > p.floor).map(|p| p.id).collect();
    assert!(fwd.contains(&"p_xh_1") && fwd.contains(&"p_xh_2") && fwd.contains(&"p_xh_3"),
        "前向单向门 p_xh_1/2/3 应齐备：{:?}", fwd);
    println!("MAP OK · floors={} points={} enemies={} portals={} gates={}",
        w.floors.len(), w.points.len(), w.enemies.len(), w.portals.len(), w.gates.len());
}

/* ---------------- ② 主线链 → 多波次 → 脑虫战胜利 → sp_grade B ---------------- */
#[test]
fn xinghe_main_line_brain_win() {
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    let pts0 = st.points;
    engine::goto(&mut st, "xh_00", &mut deaths);
    assert_eq!(st.scene_id, "xh_00");
    // L1 调查：取装甲动力格（G1 破题 + 武装）
    step(&mut st, &mut deaths, "查看登陆舱残骸");      // → xh_02_craft
    step(&mut st, &mut deaths, "取下装甲动力格");      // → xh_00
    assert!(st.inventory.iter().any(|i| i == "it_xh_armor"));

    // 登陆场多波次增员：波1→(休整)→波2→(休整)→波3→xh_01
    step(&mut st, &mut deaths, "迎战登陆场虫群");      // → xh_combat_wave1
    drive_fight(&mut st, &mut deaths);
    step_past_rest(&mut st, &mut deaths);               // xh_rest_wave1 → xh_combat_wave2
    assert_eq!(st.scene_id, "xh_combat_wave2", "波1胜应进波2");
    drive_fight(&mut st, &mut deaths);
    step_past_rest(&mut st, &mut deaths);               // xh_rest_wave2 → xh_combat_wave3
    assert_eq!(st.scene_id, "xh_combat_wave3", "波2胜应进波3");
    drive_fight(&mut st, &mut deaths);
    assert_eq!(st.scene_id, "xh_01", "波3清场应进 L1 hub");

    // 地洞（预设已解读虫纹，直接深坑垂降进 L3）
    st.set_flag("xh_rune_read");
    engine::goto(&mut st, "xh_10_arrive_tunnel", &mut deaths);
    step(&mut st, &mut deaths, "深坑垂降");            // → xh_14_drop
    step(&mut st, &mut deaths, "深坑垂降");            // → xh_20_arrive_nest

    // 巢穴：取巢膜残片 + 观察脑波频段（终结前置）
    step(&mut st, &mut deaths, "剐取脑虫巢膜残片");    // → xh_22_membrane
    step(&mut st, &mut deaths, "收下巢膜残片");        // → xh_20
    assert!(st.inventory.iter().any(|i| i == "it_xh_membrane"));
    step(&mut st, &mut deaths, "巢膜观察窗窥视");      // → xh_21_observe
    step(&mut st, &mut deaths, "记下脑波频段");        // → xh_20
    assert!(st.flag("xh_brain_trace"));

    // BOSS 决战（选择驱动）
    // 前置重返健康：波次战后 HP 已被压到 S 边，满血应战脑虫，避免高频触须被秒
    st.hp = 500;
    st.san = 100;
    step(&mut st, &mut deaths, "破膜逼近脑虫高台");    // → xh_30_brain
    step(&mut st, &mut deaths, "逼近脑虫");            // start_brain → xh_brain_round
    assert_eq!(st.scene_id, "xh_brain_round");
    let mut guard = 0;
    while st.scene_id == "xh_brain_round" && guard < 60 {
        guard += 1;
        step(&mut st, &mut deaths, "突击步枪连射");
    }
    assert_eq!(st.scene_id, "xh_41_settle", "脑虫应被歼灭回结算，实际 {}", st.scene_id);
    assert!(st.flag("xh_brain_down"), "应置 xh_brain_down");
    assert_eq!(st.sp_grade, Some('B'), "脑虫胜利应写 sp_grade=B");
    assert!(st.points > pts0, "胜利后点数应增加");
    assert!(st.inventory.iter().any(|i| i == "it_xh_brain_core"), "应掉脑虫晶核");

    // 结算卡
    step(&mut st, &mut deaths, "走向撤离阵");
    assert_eq!(st.scene_id, "xh_42_card", "结算应进卡片");
    println!("MAIN LINE OK · points={} (delta {}) · sp_grade={:?} · deaths={:?}",
        st.points, st.points - pts0, st.sp_grade, deaths);
}

/* ---------------- ③ 多波次增员链 ---------------- */
#[test]
fn xinghe_wave_reinforce() {
    let fights = wuxian_horror_ch1::scenes_xinghe::xinghe_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in ["xh_wave_a", "xh_wave_b", "xh_wave_c", "xh_tunnel_swarm", "xh_brain",
                 "xh_warrior", "xh_swarm", "xh_giant"] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    // 波次链数值合理性（蜂巢基线：普通 HP≤60，波次增员 68-88，BOSS 220）
    let wave_a = fights.iter().find(|(k, _)| *k == "xh_wave_a").map(|(_, c)| c).unwrap();
    let wave_c = fights.iter().find(|(k, _)| *k == "xh_wave_c").map(|(_, c)| c).unwrap();
    let brain = fights.iter().find(|(k, _)| *k == "xh_brain").map(|(_, c)| c).unwrap();
    assert!(wave_a.hp >= 60 && wave_c.hp > wave_a.hp, "波次应逐波增强");
    assert_eq!(brain.hp, 220, "脑虫 HP 应为 220");
    assert_eq!(brain.rage_at, Some(100), "脑虫狂暴阈 100");

    // 波次链可逐波打穿（引擎回合制）
    let w = wuxian_horror_ch1::worlds::find_world(wuxian_horror_ch1::worlds::WORLD_XINGHE).unwrap();
    assert!(w.zones.iter().any(|z| z.id == "xh_z_l1_wave" && z.ref_id == "xh_combat_wave1"),
        "ZoneDef 波次链 l1_wave 应引 xh_combat_wave1");

    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "xh_combat_wave1", &mut deaths);
    drive_fight(&mut st, &mut deaths);
    step_past_rest(&mut st, &mut deaths);
    drive_fight(&mut st, &mut deaths);
    step_past_rest(&mut st, &mut deaths);
    drive_fight(&mut st, &mut deaths);
    assert!(st.scene_id == "xh_01", "逐波清场应收束于 xh_01；实际 {}", st.scene_id);
    println!("WAVE REINFORCE OK · chain xh_wave_a→b→c → xh_01");
}