//! 《魔戒·摩瑞亚矿坑》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 MORUIYA_SCENES 并入 scenes::scene()、把 moruiya_figths() 并入 scenes::fight_cfg()，
//! 保证 engine::goto / engine::choose / engine::fight_actions 能解析 mo_* 场景与 goblin_*/watcher/balrog 战斗
//! （见 tools/design/moruiya_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① F1 地图可达性（出生点 P(12,1) 走遍 F1 关键调查点）
//!   ② 主线链：西闸门→柱厅石板→书库→卡扎督姆桥→炎魔胜利(断桥坠渊)→结算 flag
//!   ③ 双结局·甘道夫式牺牲（sp_grade=B、mo_sacrifice_done）
//!   ④ 持续 SAN 光环：火焰形态每回合 San 扣减断言
//!   ⑤ 战斗表完整性（9+2 别名）
use wuxian_horror_ch1::{engine, state::{Fight, GameState}};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_MORUIYA).expect("摩瑞亚世界已注册（合并阶段）")
}

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
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} rune={} book={} cleared={})",
        st.scene_id, st.hp, st.san, st.points,
        st.flag("mo_rune_decoded") as u8, st.flag("mo_book_read") as u8, st.flag("mo_cleared") as u8);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

/* ---------------- ① F1 地图可达性 ---------------- */
#[test]
fn moruiya_f1_map_reachable() {
    let w = world();

    // 断言三层地图每行恰 40 字符
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }

    // 出生点应为西闸门内侧 (12,1)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (12, 1), "出生点应为西闸门内侧 (12,1)");

    // F1 关键调查点必须可走动
    for p in w.points {
        if p.floor == 0 {
            assert!(worlds::walkable(w, 0, p.x, p.y), "F1 点 {}@({},{}) 不可走动", p.id, p.x, p.y);
        }
    }
    // BFS 验证 F1 全层连通（出生点可达所有 F1 可走动调查点）
    let reachable = flood_fill(w, 0, sx, sy);
    for p in w.points {
        if p.floor == 0 {
            assert!(reachable.contains(&(p.x, p.y)), "F1 点 {}@({},{}) 从出生点不可达", p.id, p.x, p.y);
        }
    }
    // 门禁 G1/G2/G3 落点本身可走动（软锁由 map_objs 控制，地图格子应为 .)
    for g in w.gates {
        if g.floor == 0 {
            assert!(worlds::walkable(w, 0, g.x, g.y), "F1 门禁 {}@({},{}) 不可走动", g.id, g.x, g.y);
        }
    }
    println!("F1 map reachability OK · reachable={} pts_on_floor0={}",
        reachable.len(), w.points.iter().filter(|p| p.floor == 0).count());
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

/* ---------------- ② 主线链：西闸门→柱厅→书库→桥→炎魔胜利(断桥坠渊)→结算 ---------------- */
#[test]
fn moruiya_main_line_bridge_break() {
    let mut st = GameState::new();
    st.hp = 1000; st.san = 100; // 高血量以便在持续 SAN 光环下撑到断桥阈值（战术性）
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "mo_01_gate", &mut deaths);
    assert_eq!(st.scene_id, "mo_01_gate");

    // 跳过监视者：后路断绝
    step(&mut st, &mut deaths, "立刻冲入闸门");
    assert!(st.flag("mo_gate_sealed"), "G1 封死后路旗标应置");

    // 柱厅石板解密
    step(&mut st, &mut deaths, "研读石板");       // mo_02_hall → mo_rune
    step(&mut st, &mut deaths, "拨动石钮");        // mo_rune → mo_02_hall（解密成功）
    assert!(st.flag("mo_rune_decoded"));
    assert!(st.flag("mo_side_rune"));

    // 书库·巴林之墓（读完整本，解锁书库大门 + 鼓声伏击由主线接线）
    // 注：楼层/廊道穿梭由开放世界传送门(P3/P4)承载，此处场景集成直接落到书库入口
    engine::goto(&mut st, "mo_book", &mut deaths);
    step(&mut st, &mut deaths, "读完《马扎布尔之书》");
    assert!(st.flag("mo_book_read"), "读完书应解锁书库大门 +150");
    assert!(st.flag("mo_side_book"));

    // 卡扎督姆桥·迎战炎魔
    engine::goto(&mut st, "mo_bridge_desc", &mut deaths);
    step(&mut st, &mut deaths, "迎向炎魔");         // start_balrog → mo_boss_round
    assert!(st.fight.is_some(), "应进入炎魔战");

    // 模拟战斗推进到断桥阈值 HP<81（战术性，直接驱动结局分支；实战由连击/重击逐步削血）
    let f = st.fight.as_mut().unwrap();
    f.hp = 78;
    let pts_before = st.points;
    step(&mut st, &mut deaths, "斩断桥索");         // → mo_ending_survive
    assert!(st.flag("mo_side_survive"), "断桥坠渊支线 F 旗标应置");
    assert!(st.flag("mo_cleared"), "应解锁东门/结算");
    assert!(st.points > pts_before, "炎魔胜利后点数应增加");
    assert!(!st.flag("mo_side_sacrifice"), "断桥与牺牲互斥");

    // 东门·黎明 + 完成结算卡
    step(&mut st, &mut deaths, "向东门");           // → mo_exit（PointsIfFlag +200）
    step(&mut st, &mut deaths, "推开东门");         // → mo_done
    assert_eq!(st.scene_id, "mo_done");
    println!("MAIN LINE OK · points={} pts_before={} cleared={}",
        st.points, pts_before, st.flag("mo_cleared") as u8);
    println!("DEATHS: {:?}", deaths);
}

/* ---------------- ③ 双结局 · 甘道夫式牺牲（sp_grade=B） ---------------- */
#[test]
fn moruiya_dual_sacrifice_ending() {
    let mut st = GameState::new();
    st.hp = 1000; st.san = 100;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "mo_bridge_desc", &mut deaths);
    step(&mut st, &mut deaths, "迎向炎魔");
    assert!(st.fight.is_some());

    // 任意阶段可直接选「让甘道夫断桥」
    step(&mut st, &mut deaths, "让甘道夫断桥");      // → mo_ending_sacrifice
    assert!(st.flag("mo_side_sacrifice"), "甘道夫牺牲支线 G 旗标应置");
    assert!(st.flag("mo_sacrifice_done"));
    assert!(st.flag("mo_cleared"));
    assert_eq!(st.sp_grade, Some('B'), "甘道夫式牺牲应产出 B 级支线 sp_grade=B");
    assert!(!st.flag("mo_side_survive"), "断桥与牺牲互斥");
    assert!(st.inventory.iter().any(|i| i == "mithril_schematic"), "牺牲线传说掉落应含秘银护甲图纸");

    step(&mut st, &mut deaths, "向东门");            // → mo_exit（PointsIfFlag +300）
    println!("SACRIFICE ENDING OK · points={} sp_grade={:?}", st.points, st.sp_grade);
}

/* ---------------- ④ 持续 SAN 光环：火焰形态每回合 San 扣减 ---------------- */
#[test]
fn moruiya_balrog_san_aura() {
    let mut st = GameState::new();
    st.hp = 1000;
    st.san = 100;
    // 构造一个已进入火焰形态（HP≤135）的炎魔实例
    st.fight = Some(Fight {
        id: "b_balrog".to_string(),
        name: "炎魔 · 都灵的克星".to_string(),
        hp: 100, max_hp: 270, dmg: (18, 26),
        reward: 600, reward_why: "击败炎魔".to_string(),
        raged: true, rage_at: Some(135), guard_turn: false, pending_log: vec![],
    });
    let mut deaths = vec![];
    engine::goto(&mut st, "mo_boss_round", &mut deaths);
    assert_eq!(st.scene_id, "mo_boss_round");

    let san_before = st.san;
    // 选「后撤重整」（dmg=0，只走 SAN 光环）：火焰形态 San-6 + 全场高温 San-3 = -9/回
    let idx = pick(&st, "后撤重整");
    engine::choose(&mut st, idx, &mut deaths);
    assert!(st.san < san_before, "火焰形态持续 SAN 光环应扣减理智");
    assert_eq!(st.san, (san_before - 9).max(0), "火焰形态回合理智应 -9（3+6）");

    // 再走一回合高电平：非火焰形态（HP>135）只扣高温 -3
    let f = st.fight.as_mut().unwrap();
    f.hp = 160; f.raged = false;
    let san2 = st.san;
    engine::goto(&mut st, "mo_boss_round", &mut deaths);
    let idx2 = pick(&st, "后撤重整");
    engine::choose(&mut st, idx2, &mut deaths);
    assert_eq!(st.san, (san2 - 3).max(0), "普通形态回合理智应 -3（全场高温）");

    println!("SAN AURA OK · fire -9/回, normal -3/回");
}

/* ---------------- ⑤ 战斗表完整性 ---------------- */
#[test]
fn moruiya_fight_table_complete() {
    let fights = wuxian_horror_ch1::scenes_moruiya::moruiya_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in [
        "goblin_scout", "goblin_pack", "goblin_raider", "drum_ambush",
        "orc_captain", "cave_troll", "orc_guard", "watcher", "balrog",
        "b_watcher", "b_balrog",
    ] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    let balrog = fights.iter().find(|(k, _)| *k == "balrog").map(|(_, c)| c).unwrap();
    assert_eq!(balrog.hp, 270, "炎魔 HP 应为 270");
    assert_eq!(balrog.reward, 600, "炎魔奖励基准 600");
    assert_eq!(balrog.dmg, (18, 26), "炎魔 dmg 应为 (18,26)");
    assert_eq!(balrog.rage_at, Some(135), "炎魔狂暴应在 HP≤135（50%）触发火焰形态");
    let watcher = fights.iter().find(|(k, _)| *k == "watcher").map(|(_, c)| c).unwrap();
    assert_eq!(watcher.hp, 145, "监视者 HP 应为 145");
    let troll = fights.iter().find(|(k, _)| *k == "cave_troll").map(|(_, c)| c).unwrap();
    assert_eq!(troll.hp, 100, "洞穴巨魔 HP 应为 100");
    println!("FIGHT TABLE OK · {} 场（含别名）", fights.len());
}