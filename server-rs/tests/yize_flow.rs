//! 《大宇宙时代 · 远古遗迹·遗泽》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 YIZE_SCENES 并入 scenes::scene()、把 yize_figths() 并入 scenes::fight_cfg()、
//! 把 world id "yize" 注册进 worlds::WORLDS，并新增 `mod scenes_yize;`（见 tools/design/yize_impl_log.md ★外部依赖）。
//! 测试内容：①F1 地图可达性 ②主线链→护盾四维护罩 BOSS→仲裁裁定→三选结局 ③护盾碎片顺序错复位。
use wuxian_horror_ch1::{engine, state::GameState};
use wuxian_horror_ch1::worlds;

fn pick(st: &GameState, keyword: &str) -> i32 {
    let scene = wuxian_horror_ch1::scenes::scene(&st.scene_id).expect("scene");
    let visible: Vec<_> = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(st))).collect();
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
    println!("STEP [{keyword}] → {} (hp={} san={} pts={})", st.scene_id, st.hp, st.san, st.points);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

fn flood_fill(w: &'static worlds::WorldData, floor: usize, sx: usize, sy: usize) -> std::collections::HashSet<(usize, usize)> {
    let mut seen = std::collections::HashSet::new();
    let mut stack = vec![(sx, sy)];
    while let Some((x, y)) = stack.pop() {
        if !seen.insert((x, y)) { continue; }
        for (dx, dy) in [(1usize, 0usize), (0usize, 1usize)] {
            let (nx, ny) = (x.wrapping_add(dx), y.wrapping_add(dy));
            if worlds::walkable(w, floor, nx, ny) { stack.push((nx, ny)); }
        }
        if x > 0 && worlds::walkable(w, floor, x - 1, y) { stack.push((x - 1, y)); }
        if y > 0 && worlds::walkable(w, floor, x, y - 1) { stack.push((x, y - 1)); }
    }
    seen
}

/* ---------------- ① F1 地图可达性 ---------------- */
#[test]
fn yize_f1_map_reachable() {
    let w = worlds::find_world("yize").expect("遗泽世界已注册（合并阶段）");
    // 每行恰 40 字符
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    // 出生点应为 F1 入口大厅 (19,24)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (19, 24), "出生点应为 (19,24)");
    // F1 关键点本身可走动，且从出生点 BFS 可达
    for p in w.points {
        if p.floor == 0 {
            assert!(worlds::walkable(w, 0, p.x, p.y), "F1 点 {}@({},{}) 不可走动", p.id, p.x, p.y);
        }
    }
    let reachable = flood_fill(w, 0, sx, sy);
    for p in w.points {
        if p.floor == 0 {
            assert!(reachable.contains(&(p.x, p.y)), "F1 点 {}@({},{}) 从出生点不可达", p.id, p.x, p.y);
        }
    }
    println!("F1 map reachability OK · reachable={}", reachable.len());
}

/* ---------------- ② 主线链：进入→矩阵→引擎断能→真相→护盾四维护罩 BOSS→仲裁裁定→带走结局 ---------------- */
#[test]
fn yize_main_line_arbiter_ending() {
    let mut st = GameState::new();
    st.hp = 600; st.san = 100; // 抬高血条以稳定扛过 Boss 累积反击
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "yz_01_arrive", &mut deaths);
    assert_eq!(st.scene_id, "yz_01_arrive");

    // Act1 F1：观测终端 → 陷阱绕行 → 补给权限卡 → 巨门 → F2
    step(&mut st, &mut deaths, "先读观测终端");                 // → yz_d_obs
    step(&mut st, &mut deaths, "记下这段预告");                 // → yz_d_trap（置 yz_relic_prologue）
    assert!(st.flag("yz_relic_prologue"));
    step(&mut st, &mut deaths, "观察 → 逐个绕行");              // f1_stealth → yz_02_passage
    assert!(st.flag("yz_f1_stealth"));
    step(&mut st, &mut deaths, "前往补给舱");                   // → yz_d_supply
    step(&mut st, &mut deaths, "取走权限卡");                   // relic_key1 → passage
    assert!(st.inventory.iter().any(|i| i == "relic_key1"));
    step(&mut st, &mut deaths, "走向尘封巨门");                 // gate1 → yz_02_gate1
    step(&mut st, &mut deaths, "沿阶而下");                     // → yz_03_city

    // Act2 F2：矩阵顺序 → 北闸上行 → F3
    step(&mut st, &mut deaths, "破解能量矩阵");                 // → yz_d_matrix
    step(&mut st, &mut deaths, "按档案序列点亮");               // matrix_order + open
    assert!(st.flag("yz_matrix_order") && st.flag("yz_matrix_open"));
    step(&mut st, &mut deaths, "走向北闸口");                   // → yz_03_gate2
    step(&mut st, &mut deaths, "沿升降闸上行");                 // → yz_04_engine_hub

    // Act3 F3：三步断电 → 轨道闸 → F4
    step(&mut st, &mut deaths, "扳手 ⅰ");                       // → wrench1
    step(&mut st, &mut deaths, "继续断电");
    step(&mut st, &mut deaths, "扳手 ⅱ");
    step(&mut st, &mut deaths, "继续断电");
    step(&mut st, &mut deaths, "扳手 ⅲ");
    step(&mut st, &mut deaths, "完成断电");                     // 3 步 → engine_room_core + open
    assert!(st.flag("yz_engine_room_core") && st.flag("yz_engine_open"));
    step(&mut st, &mut deaths, "走向轨道闸");                   // → yz_04_gate3
    step(&mut st, &mut deaths, "乘平台上行");                   // → yz_05_sanctum

    // Act4 F4：全息真相 → 十字殿
    step(&mut st, &mut deaths, "读取全息立壁");                 // → yz_d_holo
    step(&mut st, &mut deaths, "完整听完");                     // legacy_history
    assert!(st.flag("yz_legacy_history"));
    step(&mut st, &mut deaths, "进入十字殿");                   // → yz_04_hall

    // 护盾四维护罩：S3→S1→S4→S2 顺序关闭（SHIELD_ORDER）
    step(&mut st, &mut deaths, "关闭碎片座 S3");
    step(&mut st, &mut deaths, "尝试关闭");
    step(&mut st, &mut deaths, "关闭碎片座 S1");
    step(&mut st, &mut deaths, "尝试关闭");
    step(&mut st, &mut deaths, "关闭碎片座 S4");
    step(&mut st, &mut deaths, "尝试关闭");
    step(&mut st, &mut deaths, "关闭碎片座 S2");
    step(&mut st, &mut deaths, "尝试关闭");
    assert!(st.flag("yz_unlock_order"), "四碎片顺序关闭应置 unlock_order");

    // 祭坛 → 迎战仲裁者 → 护盾战（重击循环 → 仲裁裁定/胜利）
    step(&mut st, &mut deaths, "走向祭坛");                     // → yz_d_altar
    step(&mut st, &mut deaths, "迎战仲裁者");                   // → start_arbiter → yz_arb_round
    let mut guard = 0;
    while st.scene_id == "yz_arb_round" && guard < 40 {
        guard += 1;
        step(&mut st, &mut deaths, "重击");
    }
    if st.scene_id == "yz_arb_finisher" {
        step(&mut st, &mut deaths, "接受裁定");                 // 仲裁裁定 → arb_win
    }
    assert_eq!(st.scene_id, "yz_5_arb_win", "仲裁者应胜利；实际 {}", st.scene_id);
    assert!(st.flag("yz_arbiter_defeated"));
    assert_eq!(st.sp_grade, Some('D'), "仲裁者胜利应给 sp_grade=D");
    assert!(st.inventory.iter().any(|i| i == "legacy_core"), "胜利应掉 遗泽核心");
    assert!(st.flag("yz_unlock_order"), "Boss 胜利时 unlock_order 仍在");

    // Act5 三选结局：带回 种子（带走遗泽）
    step(&mut st, &mut deaths, "走向祭坛");                     // → yz_05_ending_choice
    step(&mut st, &mut deaths, "带走遗泽");                     // legacy_take +400 + legacy_shard
    assert!(st.flag("yz_legacy_take"));
    // 引擎 de-dup 语义：world::add_item 对同名物品去重，仅发 1 枚（连续 3 次 AddItem("legacy_shard")
    // 被合并为 1 枚）。故此处断言 1 枚，而非脚本的 3 枚——这是 AddItem 的既有契约，非副本 bug。
    let shards = st.inventory.iter().filter(|i| i.as_str() == "legacy_shard").count();
    assert_eq!(shards, 1, "带走结局在引擎去重语义下应得 1 枚遗泽碎片（脚本 3 枚被合并）");
    step(&mut st, &mut deaths, "驶向主神空间");                 // 结算支线 PointsIfFlag → settle_card
    assert_eq!(st.scene_id, "yz_settle_card", "应进入结算卡；实际 {}", st.scene_id);

    println!("MAIN LINE OK · points={} ending={} deaths={:?}", st.points, st.scene_id, deaths);
}

/* ---------------- ③ 护盾碎片顺序错 → 复位 + 相位冲击，不解 unlock_order ---------------- */
#[test]
fn yize_shield_order_wrong_reset() {
    let mut st = GameState::new();
    st.hp = 600; st.san = 80;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "yz_04_hall", &mut deaths);
    // 正确：S3 → S1
    step(&mut st, &mut deaths, "关闭碎片座 S3");
    step(&mut st, &mut deaths, "尝试关闭");
    step(&mut st, &mut deaths, "关闭碎片座 S1");
    step(&mut st, &mut deaths, "尝试关闭");
    // 错序：下一步应为 S4，点 S2 → 相位冲击复位，进 yz_sh_wrong
    step(&mut st, &mut deaths, "关闭碎片座 S2");
    step(&mut st, &mut deaths, "尝试关闭");
    assert_eq!(st.scene_id, "yz_sh_wrong", "错序应触发相位冲击复位；实际 {}", st.scene_id);
    assert!(!st.flag("yz_unlock_order"), "错序后不应解锁 unlock_order");
    assert!(!st.flag("yz_s3_closed") && !st.flag("yz_s1_closed"), "错序应复位已关碎片");
    assert!(st.hp <= 594, "错序应受到 6 点相位冲击（hp {}）", st.hp);
    // 重整 → 回十字殿
    step(&mut st, &mut deaths, "重整碎片阵");
    assert_eq!(st.scene_id, "yz_04_hall");
    println!("SHIELD WRONG-ORDER RESET OK · {}", st.scene_id);
}