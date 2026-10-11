//! 《天蛇族地下实验室 · 零号基地》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 TIANSHE_SCENES 并入 scenes::scene()、把 tianshe_figths() 并入 scenes::fight_cfg()、
//! 把 WORLD_TIANSHE 定为 "tianshe" 并注册进 worlds::WORLDS，且新增 `mod scenes_tianshe;`（见 tools/design/tianshe_impl_log.md ★外部依赖）。
//! 本文件为「合并后」才可编译运行的最终形态；未合并时编译失败属预期，不在此处理。
//! 测试内容：
//!   ① L1 地图可达性（出生点走遍 F1 全部调查点）
//!   ② 主线链：开场→囚笼→基因区→初蛇基因库→族长穆拉巴战（逼退）→初蛇聚合体战（样本共鸣）→胜利→结算
//!   ③ 8 张残页收集计数 → ts_archive_all 置位 → 结算 +200
//!   ④ 无初蛇基因样本 → 灭世蜕皮 3 回合倒计时灭团
//!   ⑤ 持初蛇基因样本 → 样本共鸣终局分支
//!   ⑥ 战斗表完整性（15 场 / BOSS 数值）
use wuxian_horror_ch1::{engine, state::{GameState, Mode}};
use wuxian_horror_ch1::worlds;

/// 在当前场景可见选项里按 label 关键词定位下标（选项文本/cond 与 scenes_tianshe.rs 一致）
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
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} sp={:?})", st.scene_id, st.hp, st.san, st.points, st.sp_grade);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

/// 驱动一场引擎战斗（Mode::Fight）直到 finish/结算（finisher 立即可用则直接终结）。
fn drive_fight(st: &mut GameState, deaths: &mut Vec<(&'static str, &'static str)>) {
    for _ in 0..300 {
        match &st.mode {
            Mode::Fight => {
                let acts = engine::fight_actions(st);
                let idx = if let Some(i) = acts.iter().position(|a| *a == "finisher") { i } else { 0 };
                engine::choose(st, idx as i32, deaths);
            }
            Mode::AwaitCard(c) => {
                // 异常/死亡卡片视为战斗结束；结算卡片（逼退转场无卡片）不应出现
                if !c.good {
                    return;
                }
                engine::choose(st, 0, deaths);
            }
            _ => return,
        }
    }
    panic!("战斗未在限定回合内结束 scene={}", st.scene_id);
}

/* ---------------- ① 地图可达性 ---------------- */
#[test]
fn tianshe_l1_map_reachable() {
    let w = worlds::find_world(worlds::WORLD_TIANSHE).expect("天蛇世界已注册（合并阶段）");

    // 断言四层地图每行恰 40 字符（防作者笔误导致格子偏移成墙）
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }

    // 出生点应为 L1 (1,1)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (1, 1), "出生点应为 L1 (1,1)");

    // L1 关键调查点本身可走动（非墙）
    for p in w.points {
        if p.floor == 0 {
            assert!(worlds::walkable(w, 0, p.x, p.y), "L1 点 {}@({},{}) 不可走动", p.id, p.x, p.y);
        }
    }

    // BFS 验证 L1 全层连通（出生点可达所有 L1 可走动格 → 关键调查点可达）
    let reachable = flood_fill(w, 0, sx, sy);
    for p in w.points {
        if p.floor == 0 {
            assert!(reachable.contains(&(p.x, p.y)), "L1 点 {}@({},{}) 从出生点不可达", p.id, p.x, p.y);
        }
    }
    println!("L1 map reachability OK · reachable={} pts_on_floor0={}",
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

/* ---------------- ② 主线链：开场→囚笼→基因区→初蛇基因库→战族长→逼退→聚合体→胜利→结算 ---------------- */
#[test]
fn tianshe_main_line_boss_win() {
    let mut st = GameState::new();
    // 抬高血条以稳定通过全链多场战斗（守护战/零号试做型/穆拉巴累积反击），聚焦主线程达终点；
    // San 基线抬高，避免穆拉巴狂暴 San-15 与事件扣减把主线带回崩溃结局。
    st.hp = 2000; st.san = 150;
    // 进入副本前玩家已取得武器（引擎裸拳 base=0 打不动敌人），给一柄军刀以便真实推进引擎战斗。
    st.weapon = Some(wuxian_horror_ch1::state::Weapon::Sword);
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    // 幕1 开场：观察环境 → 幕2 狱友
    engine::goto(&mut st, "ts_open", &mut deaths);
    assert_eq!(st.scene_id, "ts_open");
    step(&mut st, &mut deaths, "观察环境");     // wake_calm + mirror_1 → ts_act2_cell
    assert!(st.flag("mirror_1"));

    // 幕2 狱友：先与阿莲交谈建信任（后面救她）
    step(&mut st, &mut deaths, "和阿莲交谈");    // trust_alien_1 + intel_1 → ts_act3_convoy
    assert!(st.flag("trust_alien_1"));

    // 幕3 越狱夜：趁乱夺令 → 监工室蛇哨卫战 → 取令牌
    step(&mut st, &mut deaths, "趁乱夺令");      // → ts_guard_fight
    drive_fight(&mut st, &mut deaths);            // 战败蛇哨卫 tianshe_guard → ts_guard_win
    assert_eq!(st.scene_id, "ts_guard_win", "哨卫战后应进监工室结算；实际 {}", st.scene_id);
    step(&mut st, &mut deaths, "循令牌下电梯");  // token_route + mirror_2 + item_guard_token → ts_act4_pool
    assert!(st.inventory.iter().any(|i| i == "item_guard_token"), "应取得监工令牌 G1");

    // 幕4 血池之下（miniboss）：先救阿莲（得草药麻醉剂）→ 对决零号试做型
    step(&mut st, &mut deaths, "先救流水线上的阿莲"); // alien_saved + item_anesthetic → ts_zero_fight
    assert!(st.inventory.iter().any(|i| i == "item_anesthetic"), "救阿莲应得草药麻醉剂");
    drive_fight(&mut st, &mut deaths);            // 战败 zero_proto → ts_zero_win
    assert_eq!(st.scene_id, "ts_zero_win", "零号试做型战后应进结算；实际 {}", st.scene_id);
    step(&mut st, &mut deaths, "继续深入基因区"); // zero_prototype_killed → ts_act5_temple

    // 幕5 初蛇基因库：调查祭坛取初蛇血契 + 初蛇基因样本
    step(&mut st, &mut deaths, "调查祭坛取血契"); // G4 血契 + 基因样本 → ts_act6_core
    assert!(st.inventory.iter().any(|i| i == "item_chushe_sample"), "应取得初蛇基因样本");

    // 幕6 零号核心：揭穿真相 → 族长穆拉巴（阶段一）
    let pts_before = st.points;
    step(&mut st, &mut deaths, "揭穿他");        // zero_plan_known → ts_boss1_fight
    assert!(st.flag("zero_plan_known"));
    drive_fight(&mut st, &mut deaths);            // 穆拉巴 HP<150 triggers finisher「逼退」→ ts_boss1_retreat
    assert_eq!(st.scene_id, "ts_boss1_retreat", "阶段一应以「弃战献祭·逼退」转场；实际 {}", st.scene_id);

    // 阶段二：进熔炉 → 初蛇聚合体，强攻至 HP<100 → 样本共鸣终结
    step(&mut st, &mut deaths, "熔炉核心");      // Dyn(start_snake) → ts_boss2_round，此处才置 boss1_retreated
    assert_eq!(st.scene_id, "ts_boss2_round", "应进入阶段二回合；实际 {}", st.scene_id);
    assert!(st.flag("boss1_retreated"), "进入阶段二后应置 boss1_retreated");
    let mut guard = 0;
    while st.scene_id == "ts_boss2_round" && guard < 60 {
        guard += 1;
        // 持样本且 HP<100 时「样本共鸣」选项出现 → 直接终结；此前用重击压低血量
        let hp = st.fight.as_ref().map(|f| f.hp).unwrap_or(540);
        if hp < 100 {
            step(&mut st, &mut deaths, "样本共鸣");
        } else {
            step(&mut st, &mut deaths, "重击");
        }
    }
    assert_eq!(st.scene_id, "ts_boss2_win", "样本共鸣应进入胜利结算；实际 {}", st.scene_id);
    assert!(st.flag("sample_resonance"), "胜利应置样本共鸣 flag");

    // 胜利结算：拾取战利品 → 结算场景
    step(&mut st, &mut deaths, "拾取战利品");    // boss2_defeated +800 + item → ts_finale
    assert!(st.flag("boss2_defeated"));
    step(&mut st, &mut deaths, "主通道撤离");    // Dyn(route_finalize) → ts_finish
    assert_eq!(st.scene_id, "ts_finish", "结算场景应是 ts_finish；实际 {}", st.scene_id);

    assert!(st.points > pts_before, "全程应累计点数（pts_before={} now={}）", pts_before, st.points);
    assert_eq!(st.sp_grade, Some('D'), "支线评级应为 D");
    assert!(st.flag("dungeon_cleared"), "结算应置 dungeon_cleared");
    assert!(st.inventory.iter().any(|i| i == "item_core_crystal"), "胜利应得零号核心结晶");

    println!("MAIN LINE OK · points={} pts_before={} grade={:?} deaths={:?}", st.points, pts_before, st.sp_grade, deaths);
}

/* ---------------- ③ 残页收集计数 → 结算 +200 ---------------- */
#[test]
fn tianshe_archive_collection() {
    let mut st = GameState::new();
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    // 八张残页场景（含选项文本里固定关键词「收下残页」）
    let archive_scenes = [
        ("ts_roster", "名册"), ("ts_flow", "流水线"), ("ts_prayer", "祈祷"),
        ("ts_mother", "母体"), ("ts_seal", "符箓"), ("ts_escape", "越狱"),
        ("ts_name", "名字"), ("ts_heart", "心跳"),
    ];
    for (sid, _kw) in archive_scenes {
        engine::goto(&mut st, sid, &mut deaths);
        assert_eq!(st.scene_id, sid);
        let before = st.points;
        step(&mut st, &mut deaths, "收下");       // 每页 Points+30 → 回 ts_hall
        assert!(st.points >= before + 30, "残页 {sid} 应 +30；be={before} now={}", st.points);
        assert_eq!(st.scene_id, "ts_hall", "残页收集后应回探索间");
    }

    // 满 8 张 → aggregate flag ts_archive_all 已置
    assert!(st.flag("ts_archive_all"), "集齐 8 张应置 ts_archive_all");

    // 进入结算：走主通道撤离时 PointsIfFlag(ts_archive_all,200) 触发 → 点数额外 +200
    engine::goto(&mut st, "ts_finale", &mut deaths);
    let pts_before_finalize = st.points;
    step(&mut st, &mut deaths, "主通道撤离");     // route_finalize → ts_finish
    assert_eq!(st.scene_id, "ts_finish");
    assert!(st.points >= pts_before_finalize + 200,
        "集齐 8 张结算应 +200（be={} now={}）", pts_before_finalize, st.points);
    assert_eq!(st.sp_grade, Some('D'));

    println!("ARCHIVE COLLECTION OK · points={} finalize_delta={}",
        st.points, st.points - pts_before_finalize);
}

/* ---------------- ④ 无初蛇基因样本 → 灭世蜕皮 3 回合倒计时灭团 ---------------- */
#[test]
fn tianshe_no_sample_shed_wipe() {
    let mut st = GameState::new();
    st.hp = 500; st.san = 100;                 // 提高血量避免反击致死，聚焦倒计时灭团逻辑
    // 手工构建阶段二 Fight（HP<100 无样本 → shed 分支）。无样本：inventory 不含 item_chushe_sample。
    st.fight = Some(wuxian_horror_ch1::state::Fight {
        id: "apocalypse_snake".to_string(),
        name: "初蛇基因聚合体·灭世之蛇幼体".to_string(),
        hp: 60, max_hp: 540,
        dmg: (26, 40), reward: 800, reward_why: "终结灭世之蛇幼体".to_string(),
        raged: false, rage_at: Some(200), guard_turn: false,
        pending_log: vec![],
    });

    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "ts_boss2_round", &mut deaths);
    assert_eq!(st.scene_id, "ts_boss2_round");

    // 连续 3 次重击：HP<100 无样本 → 倒计时 1→2→3，第 3 次返回灭团
    step(&mut st, &mut deaths, "重击");          // ts_apoc_1
    assert!(st.flag("ts_apoc_1"));
    assert_eq!(st.scene_id, "ts_boss2_round");
    step(&mut st, &mut deaths, "重击");          // ts_apoc_2
    assert!(st.flag("ts_apoc_2"));
    assert_eq!(st.scene_id, "ts_boss2_round");
    step(&mut st, &mut deaths, "重击");          // 第 3 次 → ts_boss2_wipe
    assert!(st.flag("ts_apoc_3"), "第 3 回合应置 ts_apoc_3");
    assert_eq!(st.scene_id, "ts_boss2_wipe", "无样本 3 回合应灭团；实际 {}", st.scene_id);

    // 灭团死亡档案
    assert!(!deaths.is_empty(), "灭团应记录死亡档案");
    assert!(deaths.iter().any(|(t, _)| t.contains("灭世蜕皮")), "死亡档案应含灭世蜕皮；got {:?}", deaths);
    println!("SHED WIPE OK · deaths={:?} flags=apoc3={} hp={}", deaths, st.flag("ts_apoc_3"), st.hp);
}

/* ---------------- ⑤ 持初蛇基因样本 → 样本共鸣终局分支 ---------------- */
#[test]
fn tianshe_with_sample_resonance() {
    let mut st = GameState::new();
    st.hp = 300; st.san = 100;
    st.inventory.push("item_chushe_sample".into()); // 持样本
    st.fight = Some(wuxian_horror_ch1::state::Fight {
        id: "apocalypse_snake".to_string(),
        name: "初蛇基因聚合体·灭世之蛇幼体".to_string(),
        hp: 60, max_hp: 540,
        dmg: (26, 40), reward: 800, reward_why: "终结灭世之蛇幼体".to_string(),
        raged: false, rage_at: Some(200), guard_turn: false,
        pending_log: vec![],
    });

    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "ts_boss2_round", &mut deaths);
    assert_eq!(st.scene_id, "ts_boss2_round");

    // 持样本 + HP<100 → 「样本共鸣」可见且直接终结
    step(&mut st, &mut deaths, "样本共鸣");
    assert_eq!(st.scene_id, "ts_boss2_win", "样本共鸣应直接进入胜利；实际 {}", st.scene_id);
    assert!(st.flag("sample_resonance"), "应置 sample_resonance");
    assert_eq!(st.sp_grade, None, "共鸣胜利前不写支线评级（结算时才置 D）");

    // 可继续推进：拾取战利品 → 结算
    step(&mut st, &mut deaths, "拾取战利品");
    step(&mut st, &mut deaths, "主通道撤离");
    assert_eq!(st.scene_id, "ts_finish");
    assert_eq!(st.sp_grade, Some('D'));
    println!("SAMPLE RESONANCE OK · ending={} deaths={:?}", st.scene_id, deaths);
}

/* ---------------- ⑥ 战斗表完整性（15 场 + BOSS 数值） ---------------- */
#[test]
fn tianshe_fight_table_complete() {
    let fights = wuxian_horror_ch1::scenes_tianshe::tianshe_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in [
        "tianshe_guard", "tianshe_hound", "cell_rioter",
        "snake_overseer", "blood_swallower", "licker_x", "zero_proto",
        "snake_guard", "chushe_tentacle", "rabid_guard",
        "royal_guard", "wangxue_exp", "nest_tentacle",
        "mulaba", "apocalypse_snake",
    ] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    assert_eq!(fights.len(), 15, "天蛇战斗表应为 15 场，实际 {}", fights.len());

    let mulaba = fights.iter().find(|(k, _)| *k == "mulaba").map(|(_, c)| c).unwrap();
    assert_eq!(mulaba.hp, 360, "族长穆拉巴 HP 应为 360");
    assert_eq!(mulaba.reward, 500, "族长穆拉巴奖励基准 500");
    let snake = fights.iter().find(|(k, _)| *k == "apocalypse_snake").map(|(_, c)| c).unwrap();
    assert_eq!(snake.hp, 540, "初蛇聚合体 HP 应为 540");
    assert_eq!(snake.reward, 800, "初蛇聚合体奖励基准 800");

    println!("FIGHT TABLE OK · {} 场", fights.len());
}