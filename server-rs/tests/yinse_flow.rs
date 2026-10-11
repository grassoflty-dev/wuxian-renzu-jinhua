//! 《银色大地 · 地灵族机界遗迹》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 YINSE_SCENES 并入 scenes::scene()、把 yinse_figths() 并入 scenes::fight_cfg()、
//! 把 WORLD_YINSE 注册进 worlds::WORLDS，并新增 `mod scenes_yinse;`（见 tools/design/yinse_impl_log.md ★外部依赖）。
//! 测试内容：①L1 地图可达性 ②主线链→一段 BOSS 胜利→转场→二段胜利→结算 ③顺序机关链错序惩罚 ④残响演出文本不触发战斗。
use wuxian_horror_ch1::{engine, state::{GameState, Mode}};
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

/* ---------------- ① L1 地图可达性 ---------------- */
#[test]
fn yinse_f1_map_reachable() {
    let w = worlds::find_world(worlds::WORLD_YINSE).expect("银色大地世界已注册（合并阶段）");
    // 每行恰 40 字符
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    // 出生点应为降落点 (2,13)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (2, 13), "出生点应为降落点 (2,13)");
    // L1 关键点本身可走动
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
    println!("L1 map reachability OK · reachable={}", reachable.len());
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

/* ---------------- ② 主线链：救人→髅→机关链→库房→真相→BOSS 两段→结算 ---------------- */
#[test]
fn yinse_main_line_boss_two_phase() {
    let mut st = GameState::new();
    st.hp = 600; st.san = 100; // 抬高血条以稳定扛过 1/2 段 BOSS 累积反击（否则随机反击可致死 → ys_lose_r1）
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "ys_01_drop", &mut deaths);
    assert_eq!(st.scene_id, "ys_01_drop");

    // Act1 救人（冲上去 → 连战 → 收阿桑）
    step(&mut st, &mut deaths, "冲上去救人");          // → ys_03_asang_save
    step(&mut st, &mut deaths, "战斗");                // → ys_03_asang_win
    step(&mut st, &mut deaths, "收入队");              // → ys_04_trench，此处才置 ys_save_asang
    assert!(st.flag("ys_save_asang"));

    // Act2 战潮王·髅：迎战 → 战斗 → 得方解石（在「走向电梯井」结算步取得）
    step(&mut st, &mut deaths, "迎战");                // → ys_05_lou_enter
    step(&mut st, &mut deaths, "战斗");                // → ys_05_lou_win
    step(&mut st, &mut deaths, "走向电梯井");          // 结算：加 item_diling → ys_05_ele1
    assert!(st.inventory.iter().any(|i| i == "item_diling"), "髅亡应掉地灵方解石");

    // G1 电梯井下行
    step(&mut st, &mut deaths, "开电梯井");            // ys_05_ele1 → ys_05_ele1_open
    step(&mut st, &mut deaths, "下行 L2");             // → ys_06_city

    // Act3 机关链：master→B→C 顺序
    step(&mut st, &mut deaths, "去配电塔总控");         // → ys_06_power_master
    step(&mut st, &mut deaths, "拉下总控");             // → ys_06_power_mid
    step(&mut st, &mut deaths, "去配电点 B");           // → ys_06_power_b
    step(&mut st, &mut deaths, "拉下配电点 B");         // → ys_06_power_c
    step(&mut st, &mut deaths, "拉下配电点 C");         // → ys_06_power_done
    assert!(st.flag("ys_l2_power_restored"));
    step(&mut st, &mut deaths, "前去符文闸门 G2");        // → ys_06_gate2
    step(&mut st, &mut deaths, "前往居民骸骨长街");       // → ys_06_city

    // 小枢入队 + 库房取机核碎片
    step(&mut st, &mut deaths, "去居民骸骨长街");       // → ys_07_home_bones
    step(&mut st, &mut deaths, "凝视片刻");              // San-8 → ys_06_city
    assert!(st.san <= 92);
    step(&mut st, &mut deaths, "找小枢");               // → ys_07_xiaoshu
    step(&mut st, &mut deaths, "轻抚她");               // → 收小枢 + item_key
    step(&mut st, &mut deaths, "前往隐藏库房");         // → ys_08_vault
    step(&mut st, &mut deaths, "取走三神兵");           // → 取碎片
    assert!(st.inventory.iter().any(|i| i == "item_jiche"), "库房应得机核碎片");

    // Act4 工厂：生产线 + 升华装置真相回放（G3）
    step(&mut st, &mut deaths, "前往 L3 工厂");         // → ys_09_factory
    step(&mut st, &mut deaths, "去升华装置启动间");      // G3 有碎片
    step(&mut st, &mut deaths, "洞悉真相");             // → waro_truth, core_open
    assert!(st.flag("ys_waro_truth"));

    // 进 L4 决战祭坛
    step(&mut st, &mut deaths, "前行 L4");              // → ys_12_tomb

    // Act5 转折：听他说完（真相）→ 正常进决战
    step(&mut st, &mut deaths, "听他说完");             // → ys_12_truth_talk
    step(&mut st, &mut deaths, "记下真相");             // → ys_13_fight_r1

    // 一段 BOSS：检查符文打断（已 l2_power_restored）→ 开战
    step(&mut st, &mut deaths, "检查祭坛符文");         // → ys_13_cast_check (power restored)
    step(&mut st, &mut deaths, "注入都市电能");         // → start_waro_r1 并 set cast_stopped
    assert!(st.flag("ys_waro_cast_stopped"));
    // 逐轮强攻直到一段胜利 → 转场
    let mut guard = 0;
    while st.scene_id == "ys_13_round_r1" && guard < 40 {
        guard += 1;
        step(&mut st, &mut deaths, "重击");
    }
    assert_eq!(st.scene_id, "ys_waR0_cast", "一段胜利应进入转场演出；实际 {}", st.scene_id);

    // 转场演出（残响文本，不触发战斗）→ 放二段
    step(&mut st, &mut deaths, "凝望投影");             // → start_waro_r2 → ys_14_fight_r2
    assert!(!matches!(st.mode, Mode::Fight), "转场演出不应触发引擎战斗模式");

    // 二段战
    step(&mut st, &mut deaths, "开始决战");             // → ys_14_round_r2
    guard = 0;
    while st.scene_id == "ys_14_round_r2" && guard < 40 {
        guard += 1;
        step(&mut st, &mut deaths, "重击");
    }
    assert!(st.scene_id.starts_with("ys_15_ending"), "二段胜利应进入结局；实际 {}", st.scene_id);
    assert!(st.flag("ys_waro_defeated"));
    assert_eq!(st.sp_grade, Some('D'), "胜利应给 D 级支线评级");
    assert!(st.inventory.iter().any(|i| i == "item_walo_tear"), "胜利应掉 瓦罗之泪");

    println!("MAIN LINE OK · points={} ending={} deaths={:?}", st.points, st.scene_id, deaths);
}

/* ---------------- ③ 顺序机关链错序惩罚（先动 C → 触发电偶战斗） ---------------- */
#[test]
fn yinse_powerchain_wrong_order_triggers_golem() {
    let mut st = GameState::new();
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "ys_06_power_master", &mut deaths);
    step(&mut st, &mut deaths, "拉下总控");             // master done
    step(&mut st, &mut deaths, "去配电点 B");           // → ys_06_power_b
    // 错序：先动 C → route_wrong_order 触发电偶战斗场景
    step(&mut st, &mut deaths, "先动 C");              // → ys_06_golem_fight
    assert_eq!(st.scene_id, "ys_06_golem_fight", "错序应触发电偶战斗；实际 {}", st.scene_id);
    assert!(st.flag("ys_wrong_order_golem"));
    assert!(st.san <= 96, "错序不应触发 San 惩罚以外的叠加（此处已 -4）");
    println!("POWERCHAIN WRONG-ORDER OK · golem_fight");
}

/* ---------------- ④ 残响演出文本不触发战斗（圣位演出红线） ---------------- */
#[test]
fn yinse_huang_yaji_cast_is_playout_not_fight() {
    let mut st = GameState::new();
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    // 预置一段 BOSS 状态，模拟一段胜利后的转场 scene
    engine::goto(&mut st, "ys_waR0_cast", &mut deaths);
    assert_eq!(st.scene_id, "ys_waR0_cast");
    assert!(st.fight.is_none(), "转场演出不应初始化 Fight（圣位不可战）");
    assert!(!matches!(st.mode, Mode::Fight), "演出场景不应进入战斗模式");
    // 演出仅推进剧情文本 → 不产生任何死亡档案 / 不触发 fight_id
    let scene = wuxian_horror_ch1::scenes::scene("ys_waR0_cast").unwrap();
    assert!(scene.fight_id.is_none(), "演出场景 fight_id 应为 None");
    println!("HUANG-YAJI CAST PLAYLIST OK · 无战斗、无 fight_id");
}