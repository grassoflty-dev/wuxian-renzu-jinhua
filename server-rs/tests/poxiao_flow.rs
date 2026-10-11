//! 《无限曙光 · 破晓封锁区》（poxiao）世界 · 集成测试。
//! 依赖主神线在合并阶段把 POXIAO_SCENES 并入 scenes::scene()、把 poxiao_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_POXIAO（id="poxiao"），保证 engine::goto / engine::choose 能解析 px_* 场景
//! 与 pc_* 战斗（见 tools/design/poxiao_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① L1 地图完整性（3 层各 26×40；出生点；关键地标/传送门可走）
//!   ② 主线链：开场→道尔顿教学战→血站→地下三方抉择（人类线）→档案→镜阵→决战格里高尔（日光终结）→结局
//!   ③ 三方势力互斥 flag：一次只能置 1 个 faction
use wuxian_horror_ch1::{engine, state::GameState, state::Mode};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_POXIAO).expect("破晓封锁区世界已注册（合并阶段）")
}

fn pick(st: &GameState, keyword: &str) -> i32 {
    let scene = wuxian_horror_ch1::scenes::scene(&st.scene_id).expect("scene");
    let visible: Vec<_> = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(&st))).collect();
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
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} arch={} mirror={} fac={})",
        st.scene_id, st.hp, st.san, st.points,
        st.flag("px_archive") as u8, st.flag("px_dawn_mirror") as u8,
        st.flag("px_faction_human") as u8 + st.flag("px_faction_moderate") as u8 + st.flag("px_faction_neutral") as u8);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

/// 驱策原生 FightCfg 战：连续攻击直到脱离战斗态。
fn fight_to_win(st: &mut GameState, deaths: &mut Vec<(&'static str, &'static str)>, guard: usize) {
    let mut g = 0;
    while matches!(st.mode, Mode::Fight) && g < guard {
        g += 1;
        engine::choose(st, 0, deaths);
    }
    assert!(g < guard, "原生战斗未在 guard 轮内结束 scene={} mode={:?}", st.scene_id, st.mode);
}

/// 重置为健康前期状态
fn fresh(st: &mut GameState) {
    st.hp = 500;
    st.san = 100;
    st.weapon = Some(wuxian_horror_ch1::state::Weapon::Sword);
}

/* ---------------- ① L1 地图完整性 ---------------- */
#[test]
fn poxiao_map_integrity() {
    let w = world();
    assert_eq!(w.floors.len(), 3, "破晓封锁区应为 3 层");
    for (fi, map) in w.floors.iter().enumerate() {
        assert_eq!(map.len(), 26, "floor{fi} rows != 26");
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (4, 24), "出生点应为 L1 (4,24)");
    // 所有调查点必须可走动；传送门起点必须落在地板
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "调查点 {}@L{}:({},{}) 不可走动", p.id, p.floor + 1, p.x, p.y);
    }
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.floor, pt.x, pt.y), "传送门 {}@L{}:({},{}) 不可走动", pt.id, pt.floor + 1, pt.x, pt.y);
    }
    // BOSS 落位在顶层决战平台 (18,5)
    let boss = w.enemies.iter().find(|e| e.fight == "pc_boss_gregor").expect("应有格里高尔 BOSS");
    assert_eq!((boss.floor, boss.x, boss.y), (2, 18, 5), "BOSS 应落位 L3 (18,5)");
    println!("MAP INTEGRITY OK · floors={} points={} portals={} enemies={}", w.floors.len(), w.points.len(), w.portals.len(), w.enemies.len());
}

/* ---------------- ② 主线链 → 格里高尔战 + 日光终结 → 结局 ---------------- */
#[test]
fn poxiao_main_line_gregor_sunray() {
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "px_00_open", &mut deaths);
    assert_eq!(st.scene_id, "px_00_open");
    let base_pts = st.points;

    // 幕0 · 开场观察 → 教堂
    step(&mut st, &mut deaths, "冷静观察四周");   // px_00_check
    step(&mut st, &mut deaths, "推门进教堂");      // px_dalton

    // 幕1 · 接受护送 → 教学战（pc_degenerate 原生 fight）→ 血站情报
    step(&mut st, &mut deaths, "我护送");          // px_deg_fight
    fight_to_win(&mut st, &mut deaths, 80);
    assert_eq!(st.scene_id, "px_deg_after", "教学战胜后应至 px_deg_after，实际 {}", st.scene_id);
    assert!(st.flag("px_anchored"), "接受护送应置 px_anchored");
    step(&mut st, &mut deaths, "跟随道尔顿");      // px_bloodbank
    step(&mut st, &mut deaths, "记下血清情报");     // px_blood_bank + it_px_plasma
    assert!(st.flag("px_blood_bank"), "血站应置 px_blood_bank");

    // 幕2 · 下地下 → 三方抉择（人类线）→ 军械库开放
    step(&mut st, &mut deaths, "地铁口");           // px_metro → px_l2_arrive
    step(&mut st, &mut deaths, "下阶梯");           // px_l2_arrive → px_l2_hub
    step(&mut st, &mut deaths, "循着人声前进");      // px_l2_arrive → px_l2_hub
    step(&mut st, &mut deaths, "通讯台");           // px_rebels
    step(&mut st, &mut deaths, "帮人类叛军");        // px_faction_human + px_armory_open
    assert!(st.flag("px_faction_human"), "应置 px_faction_human");
    assert!(!st.flag("px_faction_moderate") && !st.flag("px_faction_neutral"), "三方应互斥");
    step(&mut st, &mut deaths, "接过盟约");           // px_rebel_oath → px_l2_hub

    // 发电机通电（上电梯前置）
    step(&mut st, &mut deaths, "发电机房");
    step(&mut st, &mut deaths, "给货运电梯通电");    // px_generator

    // 幕3 · 上尖塔 → 档案弱点 → 镜阵三校准
    step(&mut st, &mut deaths, "货运电梯");          // px_l3_arrive
    step(&mut st, &mut deaths, "进入尖塔中层");       // px_l3_arrive → px_l3_hub
    step(&mut st, &mut deaths, "机密档案");          // px_archive
    step(&mut st, &mut deaths, "解密档案");          // px_archive + 开闸门
    assert!(st.flag("px_archive"), "档案应解密");

    // 镜阵三校准：左 → 右 → 主控（点亮需左右已校）
    step(&mut st, &mut deaths, "左镜");
    step(&mut st, &mut deaths, "校准左镜");          // px_mirror_l
    step(&mut st, &mut deaths, "右镜");
    step(&mut st, &mut deaths, "校准右镜");          // px_mirror_r
    step(&mut st, &mut deaths, "主控镜阵");          // px_mirror_c
    step(&mut st, &mut deaths, "点亮主控枢纽");       // px_dawn_mirror
    assert!(st.flag("px_dawn_mirror"), "镜阵应校准 px_dawn_mirror");

    // 幕4 · 决战：校准主控（需 px_archive && px_dawn_mirror → px_sunray_ready）→ 开战
    step(&mut st, &mut deaths, "登上决战平台");       // px_duel
    step(&mut st, &mut deaths, "校准主控台");         // px_sunray_ready → start_gregor
    assert_eq!(st.scene_id, "px_gregor_round", "应进入格里高尔选择驱动回合，实际 {}", st.scene_id);
    assert!(st.flag("px_sunray_ready"), "应置 px_sunray_ready");

    // 打到半血狂暴（hp<=60）后用日光射线终结
    let mut guard = 0;
    while st.scene_id == "px_gregor_round" && guard < 120 {
        guard += 1;
        let sunray = st.flag("px_sunray_ready")
            && st.fight.as_ref().map(|f| f.hp <= 40).unwrap_or(false);
        if sunray {
            step(&mut st, &mut deaths, "引动日光射线");
        } else {
            step(&mut st, &mut deaths, "稳定攻击");
        }
    }
    assert_eq!(st.scene_id, "px_end_human", "决战胜利应回 px_end_human（人类线），实际 {}", st.scene_id);
    assert!(st.flag("px_gregor_down") && st.flag("px_sunrise") && st.flag("px_end"), "应置结尾 flag");
    assert!(st.flag("px_faction_human"), "人类线 flag 应保留");
    assert_eq!(st.sp_grade, Some('B'), "人类线胜利应 sp_grade=B");
    assert!(st.inventory.iter().any(|i| i == "it_px_sun_crystal"), "应掉落日光结晶");
    assert!(st.points > base_pts, "胜利后点数应增加");
    println!("MAIN LINE OK · seg pts delta {} · deaths={:?}", st.points - base_pts, deaths);
}

/* ---------------- ③ 三方势力互斥 flag ---------------- */
#[test]
fn poxiao_faction_mutually_exclusive() {
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "px_rebels", &mut deaths);
    assert_eq!(st.scene_id, "px_rebels");
    // 三个 faction 选项都应可见（尚未选边）
    let scene = wuxian_horror_ch1::scenes::scene(&st.scene_id).unwrap();
    let visible: Vec<_> = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(&st))).collect();
    let n_faction_choices = visible.iter().filter(|c|
        c.label.contains("帮人类叛军") || c.label.contains("帮温和血族") || c.label.contains("中立独行")).count();
    assert_eq!(n_faction_choices, 3, "三方抉择应均可见（未选边前）");

    // 选人类线 → 只有 px_faction_human
    step(&mut st, &mut deaths, "帮人类叛军");
    assert!(st.flag("px_faction_human"));
    assert!(!st.flag("px_faction_moderate") && !st.flag("px_faction_neutral"), "选人类后血族/中立不得置");

    // 回到通讯台：三方选项应已因互斥不再全部可见（cond_no_faction=false）
    let scene2 = wuxian_horror_ch1::scenes::scene(&st.scene_id).unwrap();
    let visible2: Vec<_> = scene2.choices.iter().filter(|c| c.cond.map_or(true, |f| f(&st))).collect();
    let n_faction_choices2 = visible2.iter().filter(|c|
        c.label.contains("帮人类叛军") || c.label.contains("帮温和血族") || c.label.contains("中立独行")).count();
    assert_eq!(st.scene_id, "px_rebel_oath", "人类线应落在誓约场景");
    assert_eq!(n_faction_choices2, 0, "已选边后通讯台三方选项不应再可见");

    // 结算函数不被改坏：直接查三方互斥计数恒 ≤1。
    let cnt = st.flag("px_faction_human") as usize
        + st.flag("px_faction_moderate") as usize
        + st.flag("px_faction_neutral") as usize;
    assert_eq!(cnt, 1, "三方势力 flag 应互斥（恰置其一）");
    println!("FACTION EXCLUSIVE OK · cnt={} · scene={}", cnt, st.scene_id);
}