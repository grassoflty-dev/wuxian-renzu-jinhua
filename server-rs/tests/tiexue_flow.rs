//! 《无限曙光 · 铁血·地底金字塔》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 TIEXUE_SCENES 并入 scenes::scene()、把 tiexue_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_TIEXUE（id="tiexue"），保证 engine::goto / engine::choose 能解析 tx_* 场景
//! 与 tx_* 战斗（见 tools/design/tiexue_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① 地图可达性（出生点 P(1,1)、各层调查点/传送门落点可走动）+ 战斗表完整性
//!   ② 主线链（结盟线）：L1 取腕刃→G2→祭坛石板(altar_key)→G3→L3→与铁血结盟→
//!      皇后战【铁血·肩炮助战】终结→拾取神性→回归圣门→结算
//!   ③ 可战可和（猎杀线）+ 酸液终结：伏击猎杀铁血→胜利(predator_hunted)；无结盟时
//!      踩祭坛酸液机关→皇后战【祭坛酸液喷口】终结→胜利
use wuxian_horror_ch1::{engine, state::GameState};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_TIEXUE).expect("铁血世界已注册（合并阶段）")
}

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
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} floor={})",
        st.scene_id, st.hp, st.san, st.points, st.floor);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

fn crate_set_item(st: &mut GameState, item: &str) {
    if !st.inventory.iter().any(|i| i == item) {
        st.inventory.push(item.to_string());
    }
}

/// 取当前场景（供判断某选项是否存在）
fn scene_of(st: &GameState) -> &'static wuxian_horror_ch1::defs::SceneDef {
    wuxian_horror_ch1::scenes::scene(&st.scene_id).expect("scene")
}

/// 重置为健康前期状态：高 HP 供 BOSS 选择驱动多回合同调。
fn fresh(st: &mut GameState) {
    st.hp = 500;
    st.san = 100;
}

/* ---------------- ① 地图可达性 + 战斗表完整性 ---------------- */
#[test]
fn tiexue_map_reachable() {
    let w = world();
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    // 出生点应为 L1 冰原 (1,1)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (1, 1), "出生点应为 L1 (1,1)");
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "点 {}@floor{}:({},{}) 不可走动", p.id, p.floor, p.x, p.y);
    }
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.floor, pt.x, pt.y), "传送门 {}@floor{}:({},{}) 不可走动", pt.id, pt.floor, pt.x, pt.y);
    }
    // 单向下潜链路：T1(L1→L2) / T3(L2→L3)，T5 为 L2 内单向捷径，回退出走 tx_p_return 承载。
    let pts = w.portals;
    assert!(pts.iter().any(|p| p.id == "p_tx_l1l2" && p.to_floor > p.floor), "应存在下潜门 p_tx_l1l2");
    assert!(pts.iter().any(|p| p.id == "p_tx_l2l3" && p.to_floor > p.floor), "应存在下潜门 p_tx_l2l3");
    // 战斗表完整性
    let fights = wuxian_horror_ch1::scenes_tiexue::tiexue_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in ["tx_facehugger", "tx_alien_scout", "tx_alien_drone", "tx_alien_warrior",
                 "tx_alien_guard", "tx_iron_predator", "tx_alien_queen"] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    let queen = fights.iter().find(|(k, _)| *k == "tx_alien_queen").map(|(_, c)| c).unwrap();
    assert_eq!(queen.hp, 200, "皇后 HP 首版落地 200");
    assert_eq!(queen.rage_at, Some(100), "皇后狂暴阈 100");
    let pred = fights.iter().find(|(k, _)| *k == "tx_iron_predator").map(|(_, c)| c).unwrap();
    assert_eq!(pred.hp, 150, "铁血 HP 150");
    println!("MAP + FIGHT TABLE OK · floors={} points={} portals={} fights={}",
        w.floors.len(), w.points.len(), w.portals.len(), fights.len());
}

/* ---------------- ② 主线链：结盟线 + 肩炮终结 + 结算 ---------------- */
#[test]
fn tiexue_main_line_alliance_shoulder_finisher() {
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "tx_00_open", &mut deaths);
    assert_eq!(st.scene_id, "tx_00_open");
    step(&mut st, &mut deaths, "检查冻尸");       // → tx_11_frozen
    step(&mut st, &mut deaths, "卸下铁血腕刃");    // iron_blade → 置 frozen_predator + tx_g2_open → L1 hub
    assert!(st.inventory.iter().any(|i| i == "iron_blade"), "应有铁血腕刃");
    assert!(st.flag("frozen_predator") && st.flag("tx_g2_open"), "冻尸/腕刃应置 frozen_predator+tx_g2_open");

    step(&mut st, &mut deaths, "下冰阶");          // → tx_15_l1_to_l2
    step(&mut st, &mut deaths, "沿冰阶下行");      // → L2 hub
    assert_eq!(st.scene_id, "tx_20_l2_hub");

    step(&mut st, &mut deaths, "核心墓室");        // cond tx_g2_open → tx_33_core
    step(&mut st, &mut deaths, "解读祭坛石板");    // altar_key +50 → L2 hub
    assert!(st.flag("altar_key"), "应置 altar_key");

    step(&mut st, &mut deaths, "祭坛圣门");        // cond altar_key → tx_35_l2_to_l3
    step(&mut st, &mut deaths, "坠入圣殿");        // → L3 hub
    assert_eq!(st.scene_id, "tx_30_l3_hub");

    // 与铁血结盟
    step(&mut st, &mut deaths, "铁血·成年礼战士"); // → tx_40_predator
    step(&mut st, &mut deaths, "归还腕刃");        // → tx_41_alliance
    assert!(st.flag("predator_alliance"), "应置 predator_alliance");
    step(&mut st, &mut deaths, "回圣殿");          // → tx_30_l3_hub

    // 迎战皇后（选择驱动）→ 打至狂暴 → 肩炮终结
    step(&mut st, &mut deaths, "碎裂圣像");        // → tx_55_queen_start
    step(&mut st, &mut deaths, "迎战皇后");        // start_queen → tx_queen_round
    let mut guard = 0;
    while st.scene_id == "tx_queen_round" && guard < 40 {
        guard += 1;
        let raged = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
        if raged && st.flag("predator_alliance") {
            if scene_of(&st).choices.iter().any(|c| c.label.contains("肩炮")) {
                step(&mut st, &mut deaths, "肩炮");  // finisher_shoulder
                continue;
            }
        }
        step(&mut st, &mut deaths, "重击");
    }
    assert_eq!(st.scene_id, "tx_60_queen_win", "皇后肩炮终结应回 tx_60_queen_win，实际 {}", st.scene_id);
    assert!(st.flag("queen_defeated"), "应置 queen_defeated");
    assert!(st.inventory.iter().any(|i| i == "death_divinity_shard"), "应有死亡神性颗粒");
    assert!(st.flag("tx_queen_shoulder"), "应走肩炮终结触发表记");

    step(&mut st, &mut deaths, "握紧感受");        // → tx_61_shard (+10 San)
    step(&mut st, &mut deaths, "走向回归圣门");     // → tx_90_exit
    step(&mut st, &mut deaths, "踏入圣门");        // route_exit_settle → tx_95_card
    assert_eq!(st.scene_id, "tx_95_card", "结算应回 tx_95_card，实际 {}", st.scene_id);
    assert!(st.settle_total > 0, "应写入结算总分");
    println!("MAIN LINE (结盟·肩炮) OK · pts={} total={} deaths={:?}", st.points, st.settle_total, deaths);
}

/* ---------------- ③ 可战可和（猎杀线）+ 酸液终结 ---------------- */
#[test]
fn tiexue_hunt_and_acid_finisher() {
    // 猎杀线：伏击猎杀铁血 → predator_hunted
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    crate_set_item(&mut st, "iron_blade");
    crate_set_item(&mut st, "predator_wristblade_elite");

    engine::goto(&mut st, "tx_40_predator", &mut deaths);
    step(&mut st, &mut deaths, "伏击猎杀");        // start_predator → tx_predator_round
    let mut g = 0;
    while st.scene_id == "tx_predator_round" && g < 40 {
        g += 1;
        if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
            step(&mut st, &mut deaths, "腕刃连斩");
        } else {
            step(&mut st, &mut deaths, "重击");
        }
    }
    assert_eq!(st.scene_id, "tx_42_hunted", "猎杀应回 tx_42_hunted，实际 {}", st.scene_id);
    assert!(st.flag("predator_hunted"), "应置 predator_hunted");
    assert!(st.inventory.iter().any(|i| i == "predator_wristblade_elite"), "应有猎场之礼腕刃");

    // 酸液终结：无结盟，踩祭坛酸液机关 → 皇后战【祭坛酸液喷口】终结
    let mut st2 = GameState::new();
    fresh(&mut st2);
    let mut deaths2: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st2, "tx_30_l3_hub", &mut deaths2);
    step(&mut st2, &mut deaths2, "酸液喷口");      // → tx_50_acid_puzzle
    step(&mut st2, &mut deaths2, "踩左侧石板");     // step1
    step(&mut st2, &mut deaths2, "踩右侧石板");     // → tx_acid_primed
    assert!(st2.flag("tx_acid_primed"), "酸液机关应置 tx_acid_primed");
    step(&mut st2, &mut deaths2, "不踩了");        // → L3 hub

    step(&mut st2, &mut deaths2, "迎战皇后");      // → tx_55_queen_start
    step(&mut st2, &mut deaths2, "迎战皇后");      // start_queen → tx_queen_round
    let mut g2 = 0;
    while st2.scene_id == "tx_queen_round" && g2 < 40 {
        g2 += 1;
        if st2.flag("tx_acid_primed") {
            if scene_of(&st2).choices.iter().any(|c| c.label.contains("酸液")) {
                step(&mut st2, &mut deaths2, "酸液喷口");  // finisher_acid
                continue;
            }
        }
        step(&mut st2, &mut deaths2, "重击");
    }
    assert_eq!(st2.scene_id, "tx_60_queen_win", "皇后酸液终结应回 tx_60_queen_win，实际 {}", st2.scene_id);
    assert!(st2.flag("queen_defeated") && st2.flag("tx_queen_acid"), "应置 queen_defeated + tx_queen_acid(酸液线)");
    assert!(!st2.flag("predator_alliance") && !st2.flag("predator_hunted"), "非结盟线不应置任何 predator 结盟/猎杀 flag");
    println!("HUNT (猎杀线) + ACID FINISHER OK · pts={} pts2={}", st.points, st2.points);
}