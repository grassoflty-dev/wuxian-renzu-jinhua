//! 《无限恐怖 · 死神来了》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 SISHEN_SCENES 并入 scenes::scene()、把 sishen_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_SISHEN（id="sishen"），保证 engine::goto / engine::choose 能解析 ss_* 场景
//! 与 ss_* 战斗（见 tools/design/sishen_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① L1 地图可达性（出生点 P(20,5) 走遍 L1 调查点 / 传送门起点可踩）
//!   ② 命运清单改命链：三征兆观测 → ss_fate_rewritten；预判征兆后环境机关免死存活
//!   ③ 环境机关死亡档案：未预判征兆进入环境机关 → 跳死亡档案「意外身故」
use wuxian_horror_ch1::{engine, state::GameState};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_SISHEN).expect("死神来了世界已注册（合并阶段）")
}

/// 在当前 scene 中找到含 keyword 的可选子选项（考虑 cond）
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
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} fall={} boom={} shock={} fate={})",
        st.scene_id, st.hp, st.san, st.points,
        st.flag("ss_foresee_fall") as u8, st.flag("ss_foresee_explosion") as u8,
        st.flag("ss_foresee_shock") as u8, st.flag("ss_fate_rewritten") as u8);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

fn fresh(st: &mut GameState) {
    st.hp = 200;
    st.san = 100;
}

/* ---------------- ① L1 地图可达性 ---------------- */
#[test]
fn sishen_l1_map_reachable() {
    let w = world();
    // 三层地图每行恰 40 字符
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    // 出生点应为候机大厅 (20,5)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (20, 5), "出生点应为 L1 候机大厅 (20,5)");
    // L1 全部调查点必须可走动
    for p in w.points {
        if p.floor == 0 {
            assert!(worlds::walkable(w, 0, p.x, p.y), "L1 点 {}@({},{}) 不可走动", p.id, p.x, p.y);
        }
    }
    // 传送门起点必须落在地板（可踩触发传送）
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.floor, pt.x, pt.y), "传送门 {}@L{}:{} 不可走动", pt.id, pt.floor + 1, pt.y);
    }
    println!("L1 map reachability OK · points={} portals={} zones={}", w.points.len(), w.portals.len(), w.zones.len());
}

/* ---------------- ② 命运清单改命链 ---------------- */
#[test]
fn sishen_fate_rewrite_chain() {
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    // 观测三征兆（通过场景改命路由）
    engine::goto(&mut st, "ss_02_escalator", &mut deaths);   // 坠落征兆
    step(&mut st, &mut deaths, "识破坠落征兆");
    assert!(st.flag("ss_foresee_fall"), "坠落征兆应已置");

    engine::goto(&mut st, "ss_03_truck", &mut deaths);       // 爆炸征兆
    step(&mut st, &mut deaths, "识破爆炸征兆");
    assert!(st.flag("ss_foresee_explosion"), "爆炸征兆应已置");

    engine::goto(&mut st, "ss_04_fuse", &mut deaths);        // 触电征兆
    step(&mut st, &mut deaths, "识破触电征兆");
    assert!(st.flag("ss_foresee_shock"), "触电征兆应已置");
    assert!(st.flag("ss_fate_rewritten"), "三征兆齐备应置 ss_fate_rewritten");

    // 预判后进入环境机关：免死存活（改命）
    let pts_before = st.points;
    engine::goto(&mut st, "ss_10_death_fall", &mut deaths);  // 坠落机关
    step(&mut st, &mut deaths, "逼近护栏");                   // zone_fall → 存活分支
    assert_eq!(st.scene_id, "ss_01_l1_hub", "预判坠落征兆后应改命免死回到 L1 hub");
    assert!(st.flag("ss_sign_fall_dodged"), "应置坠落改命标志");
    assert!(st.points > pts_before, "改命应加点数");

    engine::goto(&mut st, "ss_11_death_boom", &mut deaths);  // 爆炸机关
    step(&mut st, &mut deaths, "逼近泄漏核心");
    assert_eq!(st.scene_id, "ss_03_l2_hub", "预判爆炸征兆后应改命免死回到 L2 hub");

    engine::goto(&mut st, "ss_12_death_shock", &mut deaths); // 触电机关
    step(&mut st, &mut deaths, "触碰带电的积水");
    assert_eq!(st.scene_id, "ss_04_l3_hub", "预判触电征兆后应改命免死回到 L3 hub");
    assert_eq!(deaths.len(), 0, "全改命路径不应有死亡档案");

    // 经命运清单结算 → 走向撤离光柱 → sp_grade=D 结算卡片
    engine::goto(&mut st, "ss_20_settle", &mut deaths);
    assert_eq!(st.scene_id, "ss_20_settle");
    step(&mut st, &mut deaths, "撤离光柱");
    assert_eq!(st.scene_id, "ss_21_card", "结算应落 ss_21_card 撤离卡片");
    println!("FATE REWRITE CHAIN OK · pts={} deaths={:?}", st.points, deaths);
}

/* ---------------- ③ 环境机关死亡档案 · 意外身故 ---------------- */
#[test]
fn sishen_environment_death_archive() {
    // 未预判坠落征兆 → 进入坠落机关 → 死亡档案「意外身故」
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "ss_10_death_fall", &mut deaths);
    step(&mut st, &mut deaths, "逼近护栏");                   // zone_fall → ss_40_death_fall
    assert_eq!(st.scene_id, "ss_40_death_fall",
        "未预判坠落征兆应跳死亡档案 ss_40_death_fall，实际 {}", st.scene_id);
    let scene = wuxian_horror_ch1::scenes::scene("ss_40_death_fall").expect("death scene");
    let ov = scene.overlay.as_ref().expect("死亡档案应有 overlay");
    let (title, cause) = ov.death.expect("死亡档案应有 death 记录");
    assert_eq!(cause, "未预判坠落征兆，坠下候机大厅楼层", "死因应为「意外身故」类文案");
    assert!(!st.flag("ss_fate_rewritten"), "死亡路径不应置命运改写");
    println!("ENV DEATH ARCHIVE OK · title={title} cause={cause}");
}

/* ---------------- ④ 战斗表完整性（使者象征战 1 场）---------------- */
#[test]
fn sishen_fight_table_complete() {
    let fights = wuxian_horror_ch1::scenes_sishen::sishen_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    assert!(ids.contains("ss_emissary"), "象征战表应含 ss_emissary");
    let emissary = fights.iter().find(|(k, _)| *k == "ss_emissary").map(|(_, c)| c).unwrap();
    assert!(emissary.hp > 0, "使者 HP 应大于 0");
    println!("FIGHT TABLE OK · {} 场（使者象征战 HP={}）", fights.len(), emissary.hp);
}