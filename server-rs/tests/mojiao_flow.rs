//! 《侠行天下 · 魔教总坛·血月坛》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 MOJIAO_SCENES 并入 scenes::scene()、把 mojiao_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_MOJIAO（id="mojiao"），保证 engine::goto / engine::choose 能解析 mj_* 场景
//! 与 mj_* 战斗（见 tools/design/mojiao_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① 四层地图可达性（出生点 P、各层调查点/传送门可走动）
//!   ② 主线链：闯坛（取令牌）→ 前殿（护法得令牌）→ 血池试炼 → 教主战胜利 → 抉择结局 → 结算
//!   ③ 单向传送闭环 + 战斗表完整性（BOSS 数值）
use wuxian_horror_ch1::{engine, state::GameState};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_MOJIAO).expect("魔教总坛世界已注册（合并阶段）")
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

fn step(st: &mut GameState, keyword: &str) {
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    let idx = pick(st, keyword);
    engine::choose(st, idx, &mut deaths);
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} flag_lord={})",
        st.scene_id, st.hp, st.san, st.points, st.flag("mj_lord_down") as u8);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

/// 重置为健康前期状态：高 HP + 血月试炼需残卷可镇血
fn fresh_combat(st: &mut GameState) {
    st.hp = 800;
    st.san = 100;
}

fn crate_set_item(st: &mut GameState, item: &str) {
    if !st.inventory.iter().any(|i| i == item) {
        st.inventory.push(item.to_string());
    }
}

/// ① 四层地图可达性
#[test]
fn mojiao_maps_reachable() {
    let w = world();
    // 四层地图每行恰 40 字符
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    // 出生点应为 L1 血月山道 (27,24)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (27, 24), "出生点应为 L1 血月山道 (27,24)");
    // 各层调查点须可走动
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "点 {}@L{}({},{}) 不可走动", p.id, p.floor + 1, p.x, p.y);
    }
    // 敌人/传送门起点可走动
    for e in w.enemies {
        assert!(worlds::walkable(w, e.floor, e.x, e.y), "敌人 {}@L{}({},{}) 不可走动", e.id, e.floor + 1, e.x, e.y);
    }
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.floor, pt.x, pt.y), "传送门 {}@L{} 不可走动", pt.id, pt.floor + 1);
    }
    println!("MAP REACHABLE OK · floors={} points={} enemies={} portals={}", w.floors.len(), w.points.len(), w.enemies.len(), w.portals.len());
}

/// ② 主线链：闯坛 → 前殿 → 血池试炼 → 教主战胜利 → 抉择 → 结算
#[test]
fn mojiao_main_line_lord_win() {
    let mut st = GameState::new();
    fresh_combat(&mut st);
    // 预置残卷（保证可走镇血终结路线），快速通关
    crate_set_item(&mut st, "it_mj_tome");

    engine::goto(&mut st, "mj_00", &mut Vec::new());
    assert_eq!(st.scene_id, "mj_00");
    step(&mut st, "拾级而上");          // → mj_01

    // 闯坛：取血月令牌 → 登坛牌坊（G1）
    step(&mut st, "查看血月刻石");        // → mj_02_moon_mark
    step(&mut st, "取下血月令牌");        // → mj_01, got it_mj_moon_pass
    assert!(st.inventory.iter().any(|i| i == "it_mj_moon_pass"), "应有血月令牌");
    step(&mut st, "登坛牌坊");            // → mj_04_gate
    step(&mut st, "踏入总坛前殿");        // → mj_05_arrive_qian (符号需匹配 label)

    // 前殿：会护法（得红衣令牌）→ 前殿正门（G2）
    step(&mut st, "会一会红衣护法");      // → mj_06_hufa_fight (fight)
    engine::goto(&mut st, "mj_06_hufa_win", &mut Vec::new()); // 结算护法战 → 胜场景
    step(&mut st, "拾取红衣令牌");        // → mj_05_arrive_qian, got hufa_token
    assert!(st.inventory.iter().any(|i| i == "it_mj_hufa_token"), "应有红衣令牌");
    step(&mut st, "前殿正门");            // → mj_06_gate
    step(&mut st, "入血池殿密梯");        // → mj_10_arrive_pool

    // 血池试炼（miniboss 选择驱动）
    step(&mut st, "踏血池试炼");          // → mj_20_trial
    step(&mut st, "迎战傀儡主");          // start_pool → mj_pool_round
    let mut guard = 0;
    while st.scene_id == "mj_pool_round" && guard < 30 {
        guard += 1;
        step(&mut st, "重击");
    }
    assert_eq!(st.scene_id, "mj_21_trial_done", "血池试炼应回 mj_21_trial_done，实际 {}", st.scene_id);
    assert!(st.flag("mj_pool_clear"), "应置 mj_pool_clear");
    assert!(st.inventory.iter().any(|i| i == "it_mj_pool_key"), "应得赤血钥");
    step(&mut st, "潜身拾起赤血钥");      // → mj_10_arrive_pool
    step(&mut st, "血池殿门");            // → mj_22_gate_pool
    step(&mut st, "入教主密梯");          // → mj_22_arrive_lord

    // 教主战（选择驱动 HP200）
    let pts_before = st.points;
    step(&mut st, "迎战魔教教主");        // → mj_30_lord
    step(&mut st, "迎战血月尊者");        // start_lord → mj_lord_round
    let mut guard2 = 0;
    while st.scene_id == "mj_lord_round" && guard2 < 60 {
        guard2 += 1;
        step(&mut st, "重击");
    }
    assert_eq!(st.scene_id, "mj_31_lord_down", "教主战应回 mj_31_lord_down，实际 {}", st.scene_id);
    assert!(st.flag("mj_lord_down"), "应置 mj_lord_down");
    assert_eq!(st.sp_grade, Some('D'), "教主胜利应写 sp_grade=D");
    assert!(st.points > pts_before, "胜利后点数应增加");

    // 抉择结局 → 结算
    step(&mut st, "走向血月尊者的遗愿");  // → mj_40_ending
    step(&mut st, "焚毁魔教总坛");        // → mj_41_exit, mj_end_destroy
    assert!(st.flag("mj_end_destroy") && st.flag("mj_ending_done"), "应置焚坛结局");
    step(&mut st, "踏入撤离阵");          // mj_route_exit_settle → mj_42_card
    assert_eq!(st.scene_id, "mj_42_card", "结算卡片应达");
    println!("MAIN LINE OK · points={} (delta {})", st.points, st.points - pts_before);
}

/// ③ 单向闭环 + 战斗表完整性
#[test]
fn mojiao_portal_closure_and_fights() {
    let w = world();
    // 唯一回跳门 p_mj_4（to_floor < floor）
    let back: Vec<_> = w.portals.iter().filter(|p| p.to_floor < p.floor).collect();
    assert_eq!(back.len(), 1, "应只有 p_mj_4 回跳；实际 {:?}", back.iter().map(|p| p.id).collect::<Vec<_>>());
    assert_eq!(back[0].id, "p_mj_4");
    // 前向单向门 p_mj_1/2/3
    let fwd: Vec<&str> = w.portals.iter().filter(|p| p.to_floor > p.floor).map(|p| p.id).collect();
    assert!(fwd.contains(&"p_mj_1") && fwd.contains(&"p_mj_2") && fwd.contains(&"p_mj_3"),
        "应有 p_mj_1/2/3：{:?}", fwd);

    // 战斗表完整性
    let fights = wuxian_horror_ch1::scenes_mojiao::mojiao_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in ["mj_jiaozhong", "mj_sentry", "mj_yingwei", "mj_hufa", "mj_kulei",
                 "mj_hufa2", "mj_pool_boss", "mj_jiaozhu_guard", "mj_jiaozhu"] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    let pool = fights.iter().find(|(k, _)| *k == "mj_pool_boss").map(|(_, c)| c).unwrap();
    assert_eq!(pool.hp, 120, "血池傀儡主 HP 120");
    assert_eq!(pool.rage_at, Some(60), "狂暴阈 60");
    let lord = fights.iter().find(|(k, _)| *k == "mj_jiaozhu").map(|(_, c)| c).unwrap();
    assert_eq!(lord.hp, 200, "教主 HP 200");
    assert_eq!(lord.rage_at, Some(100), "教主狂暴阈 100");
    assert_eq!(lord.reward, 200, "教主奖励 200");
    println!("CLOSURE + FIGHT TABLE OK · fights={}", fights.len());
}