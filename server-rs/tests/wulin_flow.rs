//! 《侠行天下 · 武林大会》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 WULIN_SCENES 并入 scenes::scene()、把 wulin_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_WULIN（id="wulin"），保证 engine::goto / engine::choose 能解析 wl_* 场景
//! 与 wc_* 战斗（见 tools/design/wulin_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① 地图完整性（4 层各 26×40；地标可走；出生点/spawn/唯一回跳门）
//!   ② 主线链：山门签到→擂台轮战连胜→后台阴谋→密道决战黑化盟主→结局（sp_grade D）
//!   ③ 擂台轮战链完整性 / 战斗表数值断言
//!   ④ 卧底反转结局（反戈相助 → wl_33_revolt）
use wuxian_horror_ch1::{engine, state::GameState, state::Mode};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_WULIN).expect("武林大会世界已注册（合并阶段）")
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
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} beat={}/{}/{}/{} plot={} revol={})",
        st.scene_id, st.hp, st.san, st.points,
        st.flag("wl_beat_1") as u8, st.flag("wl_beat_2") as u8, st.flag("wl_beat_3") as u8, st.flag("wl_beat_hm") as u8,
        st.flag("wl_plot_exposed") as u8, st.flag("wl_wo_di_found") as u8);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

/// 驱策原生 FightCfg 战：连续攻击（index 0 = attack）直到脱离战斗态。
fn fight_to_win(st: &mut GameState, deaths: &mut Vec<(&'static str, &'static str)>, guard: usize) {
    let mut g = 0;
    while matches!(st.mode, Mode::Fight) && g < guard {
        g += 1;
        engine::choose(st, 0, deaths);
    }
    assert!(g < guard, "原生战斗未在 guard 轮内结束 scene={} mode={:?}", st.scene_id, st.mode);
}

/// 重置为健康前期状态：高 HP + 高 SAN + 军刀（方便擂台轮战）
fn fresh(st: &mut GameState) {
    st.hp = 500;
    st.san = 100;
    st.weapon = Some(wuxian_horror_ch1::state::Weapon::Sword);
}

/// 复跑一遍擂台四连战（native fights），每场胜后回 wl_arena。
fn arena_sweep(st: &mut GameState, deaths: &mut Vec<(&'static str, &'static str)>) {
    step(st, deaths, "战峨嵋净空掌");
    fight_to_win(st, deaths, 60);
    assert!(st.flag("wl_beat_1"));
    assert_eq!(st.scene_id, "wl_arena");
    step(st, deaths, "战北吼金狮拳");
    fight_to_win(st, deaths, 60);
    step(st, deaths, "战崂山鬼门刀");
    fight_to_win(st, deaths, 60);
    step(st, deaths, "面纱客");
    fight_to_win(st, deaths, 60);
    assert!(st.flag("wl_beat_1") && st.flag("wl_beat_2") && st.flag("wl_beat_3") && st.flag("wl_beat_hm"));
}

/* ---------------- ① 地图完整性 ---------------- */
#[test]
fn wulin_map_integrity() {
    let w = world();
    for (fi, map) in w.floors.iter().enumerate() {
        assert_eq!(map.len(), 26, "floor{fi} rows != 26");
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (18, 20), "出生点应为 L1 山门 (18,20)");
    // 所有调查点必须可走动；传送门起点必须落在地板
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "调查点 {}@L{}:({},{}) 不可走动", p.id, p.floor + 1, p.x, p.y);
    }
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.floor, pt.x, pt.y), "传送门 {}@L{}:({},{}) 不可走动", pt.id, pt.floor + 1, pt.x, pt.y);
    }
    // 唯一回跳门 p_wl_4（L4→L2）制造进深闭环
    let back = w.portals.iter().filter(|p| p.to_floor < p.floor).map(|p| p.id).collect::<Vec<_>>();
    assert_eq!(back, vec!["p_wl_4"], "唯一回跳门应为 p_wl_4，实际 {:?}", back);
    println!("MAP INTEGRITY OK · floors={} points={} portals={} enemies={}", w.floors.len(), w.points.len(), w.portals.len(), w.enemies.len());
}

/* ---------------- ② 主线链 → 决战黑化盟主 → 结局 ---------------- */
#[test]
fn wulin_main_line_menzhu_win() {
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "wl_00", &mut deaths);
    assert_eq!(st.scene_id, "wl_00");
    let base_pts = st.points;

    // 签到（含密令信物 + 卧底前兆）
    step(&mut st, &mut deaths, "签到");                 // → wl_signin
    step(&mut st, &mut deaths, "领引帖");               // → wl_00 · 置 wl_signin_done
    step(&mut st, &mut deaths, "西隅暗门");             // → wl_dark_door
    step(&mut st, &mut deaths, "拾起密令信物");          // it_wl_token + wl_black_hint
    assert!(st.flag("wl_signin_done") && st.flag("wl_black_hint"));

    // 登擂台四连战
    step(&mut st, &mut deaths, "进入隆重场关");         // → wl_arena
    arena_sweep(&mut st, &mut deaths);
    assert!(st.inventory.iter().any(|i| i == "it_wl_post"), "应有引帖");

    // 尾随黑马面纱客 → 后台
    step(&mut st, &mut deaths, "尾随入后台");           // → wl_back
    step(&mut st, &mut deaths, "密令夹层");
    step(&mut st, &mut deaths, "记下密令");             // → wl_back
    step(&mut st, &mut deaths, "旧案宗卷");
    step(&mut st, &mut deaths, "确认黑化");             // → wl_back · wl_plot_exposed
    assert!(st.flag("wl_plot_exposed"), "应置 wl_plot_exposed");

    // 下密道 · 揪卧底
    step(&mut st, &mut deaths, "掀密道口");             // → wl_secret
    step(&mut st, &mut deaths, "卧底密室");
    step(&mut st, &mut deaths, "窥破卧底");             // → wl_secret · wl_wo_di_found
    assert!(st.flag("wl_wo_di_found"), "应置 wl_wo_di_found");

    // 决战黑化盟主（选择驱动"力战"）
    step(&mut st, &mut deaths, "决战黑化盟主");         // → wl_menzhu_intro
    step(&mut st, &mut deaths, "拔剑");                 // start_menzhu → wl_menzhu_round
    let mut guard = 0;
    while st.scene_id == "wl_menzhu_round" && guard < 80 {
        guard += 1;
        step(&mut st, &mut deaths, "重击");
    }
    assert_eq!(st.scene_id, "wl_32_card", "决战胜利应回 wl_32_card，实际 {}", st.scene_id);
    assert!(st.flag("wl_menzhu_down"), "应置 wl_menzhu_down");
    assert!(st.flag("wl_end"), "应置 wl_end");
    assert_eq!(st.sp_grade, Some('D'), "盟主胜利应写 sp_grade=D");
    assert!(st.points > base_pts, "胜利后点数应增加");
    println!("MAIN LINE OK · seg pts delta {} · deaths={:?}", st.points - base_pts, deaths);
}

/* ---------------- ③ 擂台轮战链 / 战斗表 ---------------- */
#[test]
fn wulin_fight_table_complete() {
    let fights = wuxian_horror_ch1::scenes_wulin::wulin_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in ["wc_hu_yuan", "wc_fight_1", "wc_fight_2", "wc_fight_3", "wc_hei_ma",
                 "wc_tie_wei", "wc_jiao_zhong", "wc_menzhu"] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    let hm = fights.iter().find(|(k, _)| *k == "wc_hei_ma").map(|(_, c)| c).unwrap();
    assert_eq!(hm.hp, 100, "黑马高手 HP 应为 100");
    assert_eq!(hm.reward, 130, "黑马高手奖励 130");
    let menzhu = fights.iter().find(|(k, _)| *k == "wc_menzhu").map(|(_, c)| c).unwrap();
    assert_eq!(menzhu.hp, 220, "黑化盟主 HP 应为 220");
    assert_eq!(menzhu.rage_at, Some(88), "盟主狂暴阈 88");
    assert_eq!(menzhu.reward, 500, "盟主奖励 500");
    // 擂台连胜链触发件齐备
    let scenes = wuxian_horror_ch1::scenes_wulin::WULIN_SCENES;
    let ids2: std::collections::HashSet<&str> = scenes.iter().map(|s| s.id).collect();
    for want in ["wl_00", "wl_arena", "wl_fight_1", "wl_fight_2", "wl_fight_3", "wl_fight_hei_ma",
                 "wl_back", "wl_secret", "wl_menzhu_intro", "wl_menzhu_round", "wl_32_card", "wl_33_revolt"] {
        assert!(ids2.contains(want), "场景表缺少 {want}");
    }
    println!("FIGHT TABLE OK · fights={} scenes={}", fights.len(), scenes.len());
}

/* ---------------- ④ 卧底反转结局 ---------------- */
#[test]
fn wulin_revolt_ending() {
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    // 预先点亮反转条件：卧底已明 + 阴谋已揭 + 信物在握
    st.set_flag("wl_plot_exposed");
    st.set_flag("wl_wo_di_found");
    engine::goto(&mut st, "wl_menzhu_intro", &mut deaths);
    step(&mut st, &mut deaths, "拔剑");                // start_menzhu
    assert_eq!(st.scene_id, "wl_menzhu_round");
    step(&mut st, &mut deaths, "反戈相助");            // → wl_33_revolt
    assert_eq!(st.scene_id, "wl_33_revolt", "反戈应回 wl_33_revolt，实际 {}", st.scene_id);
    assert!(st.flag("wl_revolt") && st.flag("wl_menzhu_down"), "反转结局应置 wl_revolt + wl_menzhu_down");
    assert!(st.inventory.iter().any(|i| i == "it_wl_menzhu_tally"), "应有盟主令(it_wl_menzhu_tally)");
    assert_eq!(st.sp_grade, Some('D'), "反转结局应写 sp_grade=D");
    // 结局卡按钮回主神
    assert!(st.scene_id == "wl_33_revolt");
    println!("REVOLT ENDING OK · pts={} sp_grade={:?}", st.points, st.sp_grade);
}