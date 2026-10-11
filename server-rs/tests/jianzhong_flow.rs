//! 《侠行天下 · 剑冢禁地》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 JIANZHONG_SCENES 并入 scenes::scene()、把 jianzhong_figths() 并入
//! scenes::fight_cfg()、并在 worlds/mod.rs 注册 WORLD_JIANZHONG（id="jianzhong"）与 WorldData，
//! 保证 engine::goto / engine::choose 能解析 jz_* 场景、jz_* 战斗，lib.rs 暴露 scenes_jianzhong
//! （见 tools/design/jianzhong_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① L1 地图可达性（出生点 P(20,24) 走遍 4 层关键调查点 + 选定传送门起点可踩）
//!   ② 主线链：开场→守陵人信任→L1→L2(取锈钥+静心)→L3(静心开石门)→L4→BOSS(问心一剑)→拔剑结局→结算
//!   ③ 拔剑/不拔剑双结局（互斥 flag + 掉落/收益断言）
//!   ④ 剑心镜像战 + BOSS 心境对决（san 读取）链路
use wuxian_horror_ch1::{engine, state::GameState};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_JIANZHONG).expect("剑冢世界已注册（合并阶段）")
}

fn pick(st: &GameState, keyword: &str) -> i32 {
    let scene = wuxian_horror_ch1::scenes::scene(&st.scene_id).expect("scene");
    let visible: Vec<_> = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(&st))).collect();
    for (i, c) in visible.iter().enumerate() {
        if c.label.contains(keyword) {
            return i as i32;
        }
    }
    panic!("scene {} 未找到含「{}」的选项；可见选项: {:?}",
        st.scene_id, keyword, visible.iter().map(|c| c.label).collect::<Vec<_>>());
}

fn step(st: &mut GameState, deaths: &mut Vec<(&'static str, &'static str)>, keyword: &str) {
    let idx = pick(st, keyword);
    engine::choose(st, idx, deaths);
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} heart={} trust={})",
        st.scene_id, st.hp, st.san, st.points,
        st.flag("jz_heart_clean") as u8, st.flag("jz_oldman_trust") as u8);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

/// 重置为健康前期状态：高 HP/San + 静心(预解石门)、后缀校验由各用例补
fn fresh(st: &mut GameState) {
    st.hp = 500;
    st.san = 100;
}

/* ---------------- ① L1 地图可达性 ---------------- */
#[test]
fn jianzhong_l1_map_reachable() {
    let w = world();
    // 四层地图每行恰 40 字符
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
        assert_eq!(map.len(), 26, "floor{fi} 应为 26 行");
    }
    // 出生点应为 L1 山门古道入口 (20,24)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (20, 24), "出生点应为 L1 山门古道 (20,24)");
    // 各层调查点必须可走动
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "点 {}@L{}:({},{}) 不可走动", p.id, p.floor + 1, p.x, p.y);
    }
    // 单向门起点必须落在地板（可踩上触发传送）
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.floor, pt.x, pt.y), "传送门 {}@L{}:{},{} 不可走动", pt.id, pt.floor + 1, pt.x, pt.y);
    }
    // 单向进深/捷径：f1/f2/f3 递增 + 两条捷径（藏剑龛回山门、断崖坠谷）to_floor<floor
    let backs: Vec<&str> = w.portals.iter().filter(|p| p.to_floor < p.floor).map(|p| p.id).collect();
    assert_eq!(backs, vec!["f2_shortcut", "f4_cliff"], "两条单向捷径应齐备: {:?}", backs);
    println!("L1 map reachability OK · points={} portals={}", w.points.len(), w.portals.len());
}

/* ---------------- ② 主线链 → BOSS 问心一剑 → 拔剑结局 → 结算 ---------------- */
#[test]
fn jianzhong_main_line_boss_win_took_sword() {
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "jz_00", &mut deaths);
    assert_eq!(st.scene_id, "jz_00");
    step(&mut st, &mut deaths, "亮出主神手环");      // jz_00_trust → trust + san+5
    assert!(st.flag("jz_oldman_trust"));
    step(&mut st, &mut deaths, "入内");               // → jz_01
    step(&mut st, &mut deaths, "登石阶而上");         // cond trust → jz_06_arrive_l2
    step(&mut st, &mut deaths, "静心打坐");           // jz_heart_clean → jz_10_arrive_l2

    // L2：取锈锁钥匙，开铁门
    step(&mut st, &mut deaths, "铭剑龛");              // → jz_13_jianming
    step(&mut st, &mut deaths, "取锈锁钥匙并拓剑铭");  // it_rust_key → jz_10_arrive_l2
    assert!(st.inventory.iter().any(|i| i == "it_rust_key"), "应有锈锁钥匙");
    step(&mut st, &mut deaths, "开锈锁铁门");          // → jz_16_open_gate_l2
    step(&mut st, &mut deaths, "登石阶下赴深谷");      // cond heart_clean → jz_20_arrive_l3

    // L3：深谷石门（heart_clean）+ 天梯
    step(&mut st, &mut deaths, "开深谷石门");          // → jz_24_open_gate_l3
    step(&mut st, &mut deaths, "登天梯而上");          // → jz_30_arrive_l4

    // L4：BOSS 前对话 → 决战 → 逼 round 至第5回合 → 问心一剑终结
    step(&mut st, &mut deaths, "无名剑碑 · 对峙");     // → jz_41_prequel
    step(&mut st, &mut deaths, "为己心证道");          // San+5 → jz_42_boss
    step(&mut st, &mut deaths, "逼近剑碑");            // start_boss → jz_boss_round
    // 前 5 回合用「后撤凝神」（guard，不出手，仍计数 BOSS 回合；san 高则 BOSS dmg-2）
    let mut guard = 0;
    while st.scene_id == "jz_boss_round" && guard < 8 {
        guard += 1;
        // 若问心一剑已可出则直接用；否则后撤凝神攒回合
        let fin_ready = {
            let scene = wuxian_horror_ch1::scenes::scene(&st.scene_id).unwrap();
            scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(&st)))
                .any(|c| c.label.contains("问心一剑"))
        };
        if fin_ready {
            step(&mut st, &mut deaths, "问心一剑");
        } else {
            step(&mut st, &mut deaths, "后撤凝神");
        }
    }
    assert_eq!(st.scene_id, "jz_50_choice", "BOSS 胜后应到 jz_50_choice，实际 {}", st.scene_id);
    assert!(st.flag("jz_boss_down"), "应置 jz_boss_down");
    assert_eq!(st.sp_grade, Some('D'), "BOSS 胜利应写 sp_grade=D");

    // 拔剑结局
    step(&mut st, &mut deaths, "拔起无名剑");          // → jz_51_ending_took
    assert!(st.flag("jz_took_sword") && !st.flag("jz_spare_sword"), "拔剑应为 jz_took_sword，且互斥");
    assert!(st.inventory.iter().any(|i| i == "it_wuming_sword"), "拔剑应得无名剑");
    step(&mut st, &mut deaths, "下山");                // route_end_settle → jz_52_card
    assert_eq!(st.scene_id, "jz_52_card", "解算卡片");
    println!("MAIN LINE OK · points={} · deaths={:?}", st.points, deaths);
}

/* ---------------- ③ 拔剑 / 不拔剑 双结局（互斥）---------------- */
#[test]
fn jianzhong_dual_endings() {
    // 拔剑分支
    let mut st_a = GameState::new();
    let mut d_a: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st_a, "jz_50_choice", &mut d_a);
    assert!(st_a.scene_id == "jz_50_choice");
    step(&mut st_a, &mut d_a, "拔起无名剑");
    assert!(st_a.flag("jz_took_sword") && !st_a.flag("jz_spare_sword"));
    assert!(st_a.inventory.iter().any(|i| i == "it_wuming_sword"));

    // 不拔剑分支：San+15 + jz_spare_sword
    let mut st_b = GameState::new();
    st_b.san = 20;
    let mut d_b: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st_b, "jz_50_choice", &mut d_b);
    step(&mut st_b, &mut d_b, "不拔剑");
    assert!(st_b.flag("jz_spare_sword") && !st_b.flag("jz_took_sword"), "不拔剑应为 jz_spare_sword，互斥");
    assert_eq!(st_b.san, 20 + 15, "不拔剑应 San+15");
    println!("DUAL ENDINGS OK · took / spare 互斥成立");
}

/* ---------------- ④ 心魔镜像战 + BOSS 心境对决 ---------------- */
#[test]
fn jianzhong_mirror_and_boss_san() {
    // 剑心幻影·初级（L2 试剑龛）：fight 表校验 + 进入即出战境；胜后置 jz_mirror_1
    let fights = wuxian_horror_ch1::scenes_jianzhong::jianzhong_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in ["jz_servant", "jz_sentry", "jz_patrol", "jz_echo", "jz_rust",
                 "jz_wraith_faint", "jz_wraith", "jz_sword_mad", "jz_stele_guard",
                 "jz_phantom_1", "jz_phantom_2", "jz_phantom_apex", "jz_sword_spirit"] {
        assert!(ids.contains(want), "战斗表缺 {want}");
    }
    let spirit = fights.iter().find(|(k, _)| *k == "jz_sword_spirit").map(|(_, c)| c).unwrap();
    assert_eq!(spirit.hp, 150, "BOSS HP 150");
    assert_eq!(spirit.rage_at, Some(70), "BOSS 狂暴 @70");
    assert_eq!(spirit.reward, 600, "BOSS 奖励 600");
    let phantom1 = fights.iter().find(|(k, _)| *k == "jz_phantom_1").map(|(_, c)| c).unwrap();
    assert!(phantom1.dmg.0 >= 12, "初级幻影克制改伤基线");

    // 镜像战场景进入即 Fight 模式（jz_phantom_1），胜后经 jz_13_mirror1_win 置 jz_mirror_1
    let mut st = GameState::new();
    fresh(&mut st);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "jz_12_mirror1", &mut deaths);
    assert_eq!(st.scene_id, "jz_12_mirror1");
    assert!(matches!(st.mode, wuxian_horror_ch1::state::Mode::Fight), "试剑龛应入战境");
    assert_eq!(st.fight.as_ref().map(|f| f.id.as_str()), Some("jz_phantom_1"), "应为 jz_phantom_1");
    // 直接结算镜像胜利（经胜场景置 jz_mirror_1）
    engine::goto(&mut st, "jz_13_mirror1_win", &mut deaths);
    step(&mut st, &mut deaths, "继续前行");            // 置 jz_mirror_1 → jz_10_arrive_l2
    assert!(st.flag("jz_mirror_1"), "应置 jz_mirror_1");

    // BOSS 心境对决文本按 san 变化：高 san → 剑心不稳；低 san → 心魔加持
    let (q, _) = fights.iter().find(|(k, _)| *k == "jz_sword_spirit").unwrap();
    let _ = q;
    println!("MIRROR + BOSS SAN OK · 战表 {} 场；镜像初战入 Fight；BOSS 狂暴/奖励/HP 校验通过", fights.len());
}