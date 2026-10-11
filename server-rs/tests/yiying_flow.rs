//! 《异形4·奥瑞迦号》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 YIYING_SCENES 并入 scenes::scene()、把 yiying_figths() 并入 scenes::fight_cfg()，
//! 并在 lib.rs 声明 `pub mod scenes_yiying`、worlds/mod.rs 注册 WORLD_YIYING（="yiying"）WorldData
//! （见 tools/design/yiying_impl_log.md ★外部依赖）。本文件按合并后接口编写。
//! 测试内容：①L1 地图可达（出生点走遍 L1 关键调查点）②主线链：开场→孵化室→皇后胜利→结算(sp_grade=C)
//! ③寄生倒计时未取样→破胸死亡（「摇篮曲」）一条 ④Father 断电前置影响终结技条件一条。
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
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} par={}/{}{} dead={:?})",
        st.scene_id, st.hp, st.san, st.points,
        st.flag("yiy_parasite_1") as u8, st.flag("yiy_parasite_2") as u8, st.flag("yiy_parasite_3") as u8, st.dead_team);
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

/* ---------------- ① L1 地图可达性 ---------------- */
#[test]
fn yiying_l1_map_reachable() {
    let w = wuxian_horror_ch1::worlds::find_world("yiying")
        .expect("异形世界已注册（合并阶段）");

    // 断言三层地图每行恰 40 字符（防作者笔误导致格子偏移成墙）
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "yiying floor{fi} row{r} len != 40: {row}");
        }
    }

    // 出生点应为 P (22,17)（登陆坞）
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (22, 17), "出生点应为登陆坞 (22,17)");

    // 本层关键点本身必须可走动（非墙）
    for p in w.points {
        if p.floor == 0 {
            assert!(worlds::walkable(w, 0, p.x, p.y), "L1 点 {}@({},{}) 不可走动", p.id, p.x, p.y);
        }
    }
    // BFS 验证 L1 全层连通（出生点可达所有 L1 可走动格 → 关键调查点可达）
    let reachable = flood_fill(w, 0, sx, sy);
    for p in w.points {
        if p.floor == 0 {
            let key = (p.x, p.y);
            assert!(reachable.contains(&key), "L1 点 {}@({},{}) 从出生点不可达", p.id, p.x, p.y);
        }
    }
    println!("L1 map reachability OK · reachable={} pts_on_floor0={}",
        reachable.len(), w.points.iter().filter(|p| p.floor == 0).count());
}

/* ---------------- ② 主线链：开场 → 孵化室 → 皇后胜利 → 结算 ---------------- */
#[test]
fn yiying_main_line_queen_win() {
    let mut st = GameState::new();
    // 玩家放落地为异形世界
    st.world_id = "yiying".into();
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "yiy_s0_arrive", &mut deaths);
    assert_eq!(st.scene_id, "yiy_s0_arrive");

    // 开场：检查装备 → 主走廊
    step(&mut st, &mut deaths, "检查装备");
    assert!(st.weapon.is_some());

    // 主走廊 → 餐厅破尸 → 先搜尸取钥匙卡 → 战斗 → 战斗后
    step(&mut st, &mut deaths, "去餐厅");
    step(&mut st, &mut deaths, "先搜尸");
    assert!(st.inventory.iter().any(|i| i == "yiy_key_med"), "搜尸应获得医疗钥匙卡");
    step(&mut st, &mut deaths, "战斗");
    assert_eq!(st.scene_id, "yiy_s2_win");

    // 下楼到 L2
    step(&mut st, &mut deaths, "去电梯井下楼");
    step(&mut st, &mut deaths, "下到 L2");
    assert_eq!(st.scene_id, "yiy_s_l2_arrive");

    // 取安保脉冲枪（解锁主控室/巢穴门双条件的前置）
    step(&mut st, &mut deaths, "检查到达厅安全柜");
    step(&mut st, &mut deaths, "取走安保脉冲枪");
    assert!(st.inventory.iter().any(|i| i == "yiy_pulse"));

    // 进主控室 → 假装顺从关停 Father（智慧路线 +100）
    step(&mut st, &mut deaths, "进主控室");
    step(&mut st, &mut deaths, "假装顺从");
    assert!(st.flag("yiy_father_off"), "智慧路线应置 yiy_father_off");
    assert_eq!(st.scene_id, "yiy_s3_after", "Father 关闭后进入反馈场景");

    // 前往孵化室；就地理结被寄生者（跳过倒计时，保持主线性）
    step(&mut st, &mut deaths, "前往孵化室");
    step(&mut st, &mut deaths, "就地了结他");
    assert_eq!(st.scene_id, "yiy_s5_nest");
    assert!(st.dead_team.iter().any(|d| d == "purvis"), "就地理结普维斯");

    // 下楼到 L3 → 进反应堆管道区 → 主动挑衅皇后（Father 已关，前置满足）
    step(&mut st, &mut deaths, "乘货运电梯下 L3");
    assert_eq!(st.scene_id, "yiy_s_l3_arrive");
    step(&mut st, &mut deaths, "进反应堆管道区");
    step(&mut st, &mut deaths, "主动挑衅皇后");
    // 抵达皇后巢穴预告 → 引向管道过熟熔毁
    step(&mut st, &mut deaths, "引向管道 · 过热熔毁");
    step(&mut st, &mut deaths, "进入决战");

    // BOSS 回合 → 引向管道 · 过热熔毁（环境终结，father_off 前置满足）
    let pts_before = st.points;
    step(&mut st, &mut deaths, "引向管道 · 过热熔毁");
    assert_eq!(st.scene_id, "yiy_queen_win", "管道过热熔毁应环境终结于 yiy_queen_win");
    assert!(st.flag("yiy_queen_final"));
    assert!(st.flag("yiy_queen_plan"), "环境终结应置支线 flag yiy_queen_plan");
    assert!(st.points > pts_before, "皇后胜利后点数应增加");

    // 打扫战场 → 女王胜利卡（sp_grade = C）
    step(&mut st, &mut deaths, "打扫战场");
    assert_eq!(st.scene_id, "yiy_queen_win_card2");
    assert_eq!(st.sp_grade, Some('C'), "本副本产出 C 级支线剧情");
    assert!(st.inventory.iter().any(|i| i == "yiy_gauss_blueprint"), "皇后掉落高斯图纸（二选一）");

    // 结算：主线给 +400 后进入 yiy_settle，评级卡确认 C 级
    st.points += 400; // 主线撤离奖励（简化为直接入账）
    st.set_flag("yiy_final");
    engine::goto(&mut st, "yiy_settle", &mut deaths);
    assert_eq!(st.scene_id, "yiy_settle");
    assert_eq!(st.sp_grade, Some('C'));

    println!("MAIN LINE OK · points={} sp_grade={:?} queen_plan={} father_off={}",
        st.points, st.sp_grade, st.flag("yiy_queen_plan"), st.flag("yiy_father_off"));
    println!("DEATHS: {:?}", deaths);
}

/* ---------------- ③ 寄生倒计时未取样 → 破胸死亡 ---------------- */
#[test]
fn yiying_parasite_timeout_death() {
    let mut st = GameState::new();
    st.world_id = "yiying".into();
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "yiy_s4_incubator", &mut deaths);
    // 带他冲医疗舱取样 → 启动寄生倒计时（置 yiy_infected + yiy_parasite_1）
    step(&mut st, &mut deaths, "带他冲医疗舱取样");
    assert!(st.flag("yiy_infected"));
    assert!(st.flag("yiy_parasite_1"));
    assert_eq!(st.scene_id, "yiy_s5_med");

    // 连续三次「先安顿一下再走（多耗一幕）」：幕数推进 1→2→3→破胸
    step(&mut st, &mut deaths, "先安顿一下再走"); // →parasite_2
    assert!(st.flag("yiy_parasite_2"));
    step(&mut st, &mut deaths, "先安顿一下再走"); // →parasite_3
    assert!(st.flag("yiy_parasite_3"));
    step(&mut st, &mut deaths, "先安顿一下再走"); // →破胸死亡

    assert_eq!(st.scene_id, "yiy_dead_parasite", "第 3 幕未取样应破胸死亡；实际 {}", st.scene_id);
    assert!(st.flag("yiy_parasite_dead"));
    assert!(!deaths.is_empty(), "应记录死亡档案");
    let tag = deaths.iter().find(|(t, _)| t.contains("摇篮曲"));
    assert!(tag.is_some(), "死亡档案应含「摇篮曲」：{:?}", deaths);
    println!("PARASITE DEATH OK · deaths={:?}", deaths);
}

/* ---------------- ④ Father 断电前置影响终结技条件 ---------------- */
#[test]
fn yiying_pipe_finisher_needs_father_off() {
    // 场景 A：未关停 Father → 管道过热熔毁选项不可用，直接用管道则死于「硫磺与蒸汽」
    let mut st = GameState::new();
    st.world_id = "yiying".into();
    st.hp = 100; st.san = 100;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "yiy_s_pipe", &mut deaths);
    assert_eq!(st.scene_id, "yiy_s_pipe");

    // Father 未关 → 「主动挑衅皇后(引向管道)」被 cond 隐藏（finisher 前置不满足）
    let scene = wuxian_horror_ch1::scenes::scene(&st.scene_id).unwrap();
    let visible = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(&st))).collect::<Vec<_>>();
    assert!(
        !visible.iter().any(|c| c.label.contains("主动挑衅皇后")),
        "未关 Father 时不得提供「主动挑衅皇后」终结选项"
    );
    // 直接扑向管道 → 高温蒸汽灭团
    step(&mut st, &mut deaths, "直接扑向管道");
    assert_eq!(st.scene_id, "yiy_dead_pipe", "未关 Father 用管道应死于「硫磺与蒸汽」");
    assert!(deaths.iter().any(|(t, _)| t.contains("硫磺与蒸汽")));

    // 场景 B：关停 Father → 「主动挑衅皇后」可见（finisher 前置满足）
    let mut st2 = GameState::new();
    st2.world_id = "yiying".into();
    st2.hp = 100; st2.san = 100;
    st2.set_flag("yiy_father_off");
    let mut deaths2: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st2, "yiy_s_pipe", &mut deaths2);
    let scene2 = wuxian_horror_ch1::scenes::scene(&st2.scene_id).unwrap();
    let visible2 = scene2.choices.iter().filter(|c| c.cond.map_or(true, |f| f(&st2))).collect::<Vec<_>>();
    assert!(
        visible2.iter().any(|c| c.label.contains("主动挑衅皇后")),
        "关停 Father 后应提供「主动挑衅皇后」终结选项"
    );
    // 选「主动挑衅皇后」可推进到女王前置（yiy_queen_pipe 已置）
    let idx = pick(&st2, "主动挑衅皇后");
    engine::choose(&mut st2, idx, &mut deaths2);
    assert!(st2.flag("yiy_queen_pipe"), "选唤起玉石俱焚后应置 yiy_queen_pipe");
    println!("PIPE FINISHER GATE OK · no_father=dead_pipe · with_father=queen_pipe");
}

/* ---------------- fight 表完整性（BOSS 数值） ---------------- */
#[test]
fn yiying_fight_table_complete() {
    let fights = wuxian_horror_ch1::scenes_yiying::yiying_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in [
        "f_yiy_facehugger", "f_yiy_chestburst", "f_yiy_worker1", "f_yiy_worker2",
        "f_yiy_workerpack", "f_yiy_sentinel1", "f_yiy_sentinel2", "f_yiy_hunter",
        "f_yiy_queenhold", "f_yiy_queen",
    ] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    let queen = fights.iter().find(|(k, _)| *k == "f_yiy_queen").map(|(_, c)| c).unwrap();
    assert_eq!(queen.hp, 200, "皇后 HP 应为 200（蜂巢+30% 至精英/BOSS 档）");
    assert_eq!(queen.reward, 450, "皇后奖励基准 450");
    assert_eq!(queen.rage_at, Some(35), "皇后狂暴 35%");
    // 蜂巢+30%：工兵 HP70 于蜂巢普通档
    let worker = fights.iter().find(|(k, _)| *k == "f_yiy_worker1").map(|(_, c)| c).unwrap();
    assert_eq!(worker.hp, 70, "工兵 HP 70 对应蜂巢普通+30% 上浮");
    println!("FIGHT TABLE OK · {} 场", fights.len());
}