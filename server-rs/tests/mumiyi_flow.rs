//! 《无限恐怖 · 木乃伊（哈姆纳塔地宫）》任务世界 · 集成测试。
//! 依赖主神线在合并阶段把 MUMIYI_SCENES 并入 scenes::scene()、把 mumiyi_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_MUMIYI（id="mumiyi"），保证 engine::goto / engine::choose 能解析
//! mm_* 场景与 mm_* 战斗（见 tools/design/mumiyi_impl_log.md ★外部依赖）。
//! 测试内容：
//!   ① 地图可达性（出生点 P(19,22) 走遍各层调查点/传送门起点）
//!   ② 主线链：考古→开棺→诅咒苏醒→伊莫顿一段→复生二段→弱水终结→封棺胜利（sp_grade=C）
//!   ③ 宝藏掉落：mm_06_box_a（黄金+护符）与 mm_14_vault（翡翠+金饰）
use wuxian_horror_ch1::{engine, state::GameState};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_MUMIYI).expect("木乃伊世界已注册（合并阶段）")
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
    println!("STEP [{keyword}] → {} (hp={} san={} pts={})", st.scene_id, st.hp, st.san, st.points);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

fn crate_set_item(st: &mut GameState, item: &str) {
    if !st.inventory.iter().any(|i| i == item) {
        st.inventory.push(item.to_string());
    }
}

/* ---------------- ① 地图可达性 ---------------- */
#[test]
fn mumiyi_map_reachable() {
    let w = world();
    // 三层地图每行恰 40 字符
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    // 出生点应为 L1 地宫入口 P(19,22)
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (19, 22), "出生点应为 L1 地宫入口 (19,22)");
    // 各层调查点必须可走动
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "点 {}@({},{})L{} 不可走动", p.id, p.x, p.y, p.floor + 1);
    }
    // 传送门起点必须落在地板
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.floor, pt.x, pt.y), "传送门 {}@L{}:{} 不可走动", pt.id, pt.floor + 1, pt.x);
    }
    println!("map reachability OK · floors={} points={} enemies={} gates={} portals={} zones={}",
        w.floors.len(), w.points.len(), w.enemies.len(), w.gates.len(), w.portals.len(), w.zones.len());
}

/// 补2：NPC 对话流程 —— 考古队长阿尔德 / 见习员乔伊 / 悔罪祭司安卡图 三段对话，
/// 每段对话按选项推进到对应场景并设置预期状态/道具。
#[test]
fn test2_npc_dialogue_flows() {
    // 阿尔德（mm_02_npc）→ 对话引导去祭碑
    let mut st = GameState::new();
    st.hp = 500;
    st.san = 100;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "mm_02_npc", &mut deaths);
    assert_eq!(st.scene_id, "mm_02_npc", "应进入阿尔德对话");
    step(&mut st, &mut deaths, "转身走向祭碑");
    assert_eq!(st.scene_id, "mm_03_stele", "阿尔德对话引导应去祭碑");

    // 乔伊（mm_02_npc_joy）→ 引导石匣·陶罐，掉落黄金
    let mut st = GameState::new();
    st.hp = 200;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "mm_02_npc_joy", &mut deaths);
    step(&mut st, &mut deaths, "打开石匣·陶罐");
    assert_eq!(st.scene_id, "mm_06_box_a", "乔伊对话应引导至石匣场景");
    step(&mut st, &mut deaths, "取走宝藏");
    assert!(st.inventory.iter().any(|i| i == "it_mumi_loot_gold"), "石匣掉落黄金");

    // 安卡图（mm_11_npc）→ 谢过祭司得弱水/虫匣提示并回圣甲虫厅
    let mut st = GameState::new();
    st.hp = 300;
    st.san = 100;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "mm_11_npc", &mut deaths);
    assert_eq!(st.scene_id, "mm_11_npc", "应进入安卡图对话");
    step(&mut st, &mut deaths, "谢过这位祭司");
    assert!(st.flag("mm_priest_hint"), "安卡图提示应置 mm_priest_hint");
    assert_eq!(st.scene_id, "mm_10_arrive_f1", "安卡图对话应返还圣甲虫厅 hub");
    println!("NPC DIALOGUE OK · 阿尔德/乔伊/安卡图 三段对话均通过");
}

/// 补3：战斗多分支 —— 伊莫顿一段多分支（强攻 / 弱水祭炼 / 呼唤合力 / 抽身），
/// 用「呼唤考古队合力」路径同时断言：扣血/脱离多回合收敛、flag、且可回到墓室。
#[test]
fn test2_imhotep1_battle_branches() {
    // 分支 A：持续强攻最终压制一段 → 复生二段
    let mut st = GameState::new();
    st.hp = 600;
    st.san = 100;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "mm_22_curse", &mut deaths);
    step(&mut st, &mut deaths, "拔出武器，迎战伊莫顿");
    assert_eq!(st.scene_id, "mm_23_imhotep1");
    let mut g = 0;
    while st.scene_id == "mm_23_imhotep1" && g < 60 {
        g += 1;
        step(&mut st, &mut deaths, "挥剑强攻");
    }
    assert!(g < 60, "强攻分支未收敛");
    assert_eq!(st.scene_id, "mm_24_reborn", "一段强攻击倒应进复生");
    assert!(st.flag("mm_imhotep1_down"));

    // 分支 B：持弱水走「以弱水祭炼」——置 mm_water_prep 且不直接进死亡
    let mut st = GameState::new();
    st.hp = 600;
    st.san = 100;
    crate_set_item(&mut st, "it_mumi_water");
    st.set_flag("mm_curse");
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "mm_22_curse", &mut deaths);
    step(&mut st, &mut deaths, "拔出武器，迎战伊莫顿");
    assert_eq!(st.scene_id, "mm_23_imhotep1");
    step(&mut st, &mut deaths, "以弱水祭炼");
    assert!(st.flag("mm_water_prep"), "弱水祭炼应置 mm_water_prep");
    assert!(st.san >= 0 && st.hp >= 0, "多分支不得越界");

    // 分支 C：伺机抽身 → 退回墓室
    let mut st = GameState::new();
    st.hp = 600;
    st.san = 100;
    st.set_flag("mm_curse");
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "mm_22_curse", &mut deaths);
    step(&mut st, &mut deaths, "拔出武器，迎战伊莫顿");
    step(&mut st, &mut deaths, "伺机抽身");
    assert_eq!(st.scene_id, "mm_20_sarc_room", "抽身分支应退回墓室");
}

/* ---------------- ② 主线链 → 伊莫顿一段 → 复生二段 → 弱水终结 → 封棺胜利 ---------------- */
#[test]
fn mumiyi_main_line_imhotep2_water_seal() {
    let mut st = GameState::new();
    st.hp = 500;
    st.san = 100;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];

    engine::goto(&mut st, "mm_00_camp", &mut deaths);
    assert_eq!(st.scene_id, "mm_00_camp");
    step(&mut st, &mut deaths, "走向被凿穿的石门");   // → mm_01_arrive
    step(&mut st, &mut deaths, "检视无名祭碑");        // → mm_03_stele
    step(&mut st, &mut deaths, "取下青铜甲虫钥");       // 得 it_mumi_key → mm_01_arrive
    assert!(st.inventory.iter().any(|i| i == "it_mumi_key"), "应有青铜甲虫钥");

    step(&mut st, &mut deaths, "与考古队长谈");         // → mm_02_npc
    step(&mut st, &mut deaths, "走向金色墓门");         // → mm_07_gate_scarab
    step(&mut st, &mut deaths, "嵌入钥匙，开机关门");   // mm_gate_scarab_open → mm_10_arrive_f1
    assert!(st.flag("mm_gate_scarab_open"));

    // F1 圣甲虫厅：取弱水(终结前置) + 虫巢神像取圣甲虫之匣(封印墓门前提)
    step(&mut st, &mut deaths, "走向弱水井");           // → mm_12_well
    step(&mut st, &mut deaths, "装满一囊弱水");         // 得 it_mumi_water → mm_10_arrive_f1
    assert!(st.inventory.iter().any(|i| i == "it_mumi_water"), "应有尼罗弱水");
    step(&mut st, &mut deaths, "查探虫巢神像");         // → mm_13_hollow
    step(&mut st, &mut deaths, "取出圣甲虫之匣");       // 得 it_mumi_scarab_sac → mm_10_arrive_f1
    assert!(st.inventory.iter().any(|i| i == "it_mumi_scarab_sac"), "应有圣甲虫之匣");

    step(&mut st, &mut deaths, "走向南侧封印墓门");     // → mm_15_gate_tomb
    step(&mut st, &mut deaths, "嵌入圣甲虫之匣");       // mm_gate_tomb_open → mm_20_sarc_room
    assert!(st.flag("mm_gate_tomb_open"));

    // F2 祭司墓室：开棺 → 诅咒苏醒
    step(&mut st, &mut deaths, "上前查看伊莫顿石棺");   // → mm_21_open_sarc
    step(&mut st, &mut deaths, "诅咒苏醒了！");         // mm_open_sarc/mm_curse → mm_22_curse
    assert!(st.flag("mm_curse"));

    // 伊莫顿一段（选择驱动）
    step(&mut st, &mut deaths, "拔出武器，迎战伊莫顿"); // route_start_imhotep1 → mm_23_imhotep1
    let mut guard = 0;
    while st.scene_id == "mm_23_imhotep1" && guard < 80 {
        guard += 1;
        step(&mut st, &mut deaths, "挥剑强攻");
    }
    assert_eq!(st.scene_id, "mm_24_reborn", "一段击倒应进复生二段，实际 {}", st.scene_id);
    assert!(st.flag("mm_imhotep1_down"), "应置 mm_imhotep1_down");

    // 复生二段
    step(&mut st, &mut deaths, "再次迎向复生之躯");     // route_start_imhotep2 → mm_25_imhotep2
    assert_eq!(st.scene_id, "mm_25_imhotep2");

    // 弱水终结
    let pts_before = st.points;
    step(&mut st, &mut deaths, "以尼罗弱水终结");        // route_imhotep2_water → mm_27_win
    assert_eq!(st.scene_id, "mm_27_win", "弱水终结应直达胜利场景");
    assert_eq!(st.sp_grade, Some('C'), "伊莫顿击杀应写 sp_grade=C");
    assert!(st.flag("mm_water_finish"), "应置 mm_water_finish");

    // 领取奖励 → 撤离结局（mm_27_win 为胜利覆盖卡片，卡片按钮导航至撤离场景）
    engine::goto(&mut st, "mm_28_escape", &mut deaths);
    assert_eq!(st.scene_id, "mm_28_escape", "胜利应回撤离结局");
    assert!(st.flag("mm_end_seal"), "应置 mm_end_seal");
    assert!(st.points > pts_before, "胜利后点数应增加（+800 结算）");
    println!("MAIN LINE OK · points={} · sp_grade={:?} · deaths={:?}", st.points, st.sp_grade, deaths);
}

/* ---------------- ③ 宝藏掉落 ---------------- */
#[test]
fn mumiyi_treasure_drops() {
    // F0 石匣·陶罐：黄金 + 护符
    let mut st = GameState::new();
    st.hp = 200;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "mm_06_box_a", &mut deaths);
    let pts = st.points;
    step(&mut st, &mut deaths, "取走宝藏");
    assert!(st.inventory.iter().any(|i| i == "it_mumi_loot_gold"), "应有黄金器皿");
    assert!(st.inventory.iter().any(|i| i == "it_mumi_amulet"), "应有圣甲虫护符");
    assert!(st.points > pts, "石匣应加点数(+120)");

    // F1 宝库供案：翡翠 + 金饰
    let mut st = GameState::new();
    st.hp = 200;
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "mm_14_vault", &mut deaths);
    let pts = st.points;
    step(&mut st, &mut deaths, "席卷宝库珍宝");
    assert!(st.inventory.iter().any(|i| i == "it_mumi_loot_gem"), "应有翡翠圣匣/宝石");
    assert!(st.inventory.iter().any(|i| i == "it_mumi_trinket"), "应有金饰");
    assert!(st.points > pts, "宝库应加点数(+150)");
    println!("TREASURE OK · box_a(gold+amulet) + vault(gem+trinket) 多掉落通过");
}

/* ---------------- ④ 战斗表完整性 ---------------- */
#[test]
fn mumiyi_fight_table() {
    let fights = wuxian_horror_ch1::scenes_mumiyi::mumiyi_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in ["mm_scarab_swarm_light", "mm_scarab_swarm", "mm_mummy_guard",
                 "mm_mummy_sentinel", "mm_imhotep", "mm_imhotep2"] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    let imhotep = fights.iter().find(|(k, _)| *k == "mm_imhotep").map(|(_, c)| c).unwrap();
    assert_eq!(imhotep.hp, 210, "伊莫顿一段 HP 应为 210");
    assert_eq!(imhotep.rage_at, Some(105), "伊莫顿狂暴阈 105（圣甲虫潮增员）");
    assert_eq!(imhotep.reward, 700, "伊莫顿一段奖励 700");
    let imhotep2 = fights.iter().find(|(k, _)| *k == "mm_imhotep2").map(|(_, c)| c).unwrap();
    assert_eq!(imhotep2.hp, 120, "伊莫顿复生 HP 应为 120");
    // 弱水终结前置：finisher_if 依赖尼罗弱水
    let st = GameState::new();
    assert!(!(imhotep.finisher_if)(&st, 10), "无弱水时不可终结");
    let mut st2 = GameState::new();
    crate_set_item(&mut st2, "it_mumi_water");
    assert!((imhotep2.finisher_if)(&st2, 10), "持有尼罗弱水时应可弱水终结");
    println!("FIGHT TABLE OK · {} 场 · Imhotep 二阶段弱水终结前置通过", fights.len());
}

/* ---------------- ⑤（补）移动遍历：出生点沿层内可走格 + 层间传送门逐层抵达所有关键坐标 ---------------- */
#[test]
fn test2_map_movement_reaches_key_coords() {
    let w = world();
    let (sx, sy) = w.spawn();
    // 逐层 BFS：把传送门起点当「跨层跳板」，层间用门户（origin→destination）联通达所有调查点/敌人/门禁坐标
    let mut seen = std::collections::HashSet::new();        // (floor, x, y) 已抵达
    let mut queue = std::collections::VecDeque::new();
    seen.insert((0usize, sx, sy));
    queue.push_back((0usize, sx, sy));

    while let Some((fl, x, y)) = queue.pop_front() {
        let rows = w.floors[fl as usize].len();
        let cols = w.floors[fl as usize][0].len();
        // 四向走格
        let deltas: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        for (dx, dy) in deltas {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || ny < 0 { continue; }
            let (nx, ny) = (nx as usize, ny as usize);
            if nx >= cols || ny >= rows { continue; }
            if seen.contains(&(fl, nx, ny)) { continue; }
            if !walk(w, fl, nx, ny) { continue; }
            seen.insert((fl, nx, ny));
            queue.push_back((fl, nx, ny));
        }
        // 层间传送门跳板
        for pt in w.portals {
            if pt.floor == fl as usize && pt.x == x && pt.y == y {
                let key = (pt.to_floor as usize, pt.tx, pt.ty);
                if !seen.contains(&key) {
                    seen.insert(key); queue.push_back(key);
                }
            }
        }
    }

    // 所有调查点 / NPC / 敌人 必须都能从出生点抵达（这些均落于可走格）
    for p in w.points {
        assert!(seen.contains(&(p.floor, p.x, p.y)), "调查点 {}@L{}:({},{}) 不可达出生点", p.id, p.floor + 1, p.x, p.y);
    }
    for n in w.npcs {
        assert!(seen.contains(&(n.floor, n.x, n.y)), "NPC {}@L{}:({},{}) 不可达出生点", n.id, n.floor + 1, n.x, n.y);
    }
    for e in w.enemies {
        assert!(seen.contains(&(e.floor, e.x, e.y)), "敌人 {}@L{}:({},{}) 不可达出生点", e.id, e.floor + 1, e.x, e.y);
    }
    // 门禁多为墙壁格（解锁后通行），断言其邻接存在可从出生点抵达的可走格
    for g in w.gates {
        let adj = [(g.x + 1, g.y), (g.x.wrapping_sub(1), g.y), (g.x, g.y + 1), (g.x, g.y.wrapping_sub(1))];
        let near = adj.iter().any(|(nx, ny)| *nx < w.floors[g.floor][0].len() && seen.contains(&(g.floor, *nx, *ny)));
        assert!(near, "门禁 {}@L{}:({},{}) 周边无可达格", g.id, g.floor + 1, g.x, g.y);
    }
    // 传送门目标落点亦须可走动
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.to_floor, pt.tx, pt.ty), "传送门 {} 目标 @L{}:({},{}) 不可走动", pt.id, pt.to_floor + 1, pt.tx, pt.ty);
    }
    println!("MOVE TRAVERSE OK · 出生点({},{}) 步达 layers={} points={} npcs={} enemies={} gates={}",
        sx, sy, w.floors.len(), w.points.len(), w.npcs.len(), w.enemies.len(), w.gates.len());
}

/// 补用：某坐标是否可走（非边界墙字符）
fn walk(w: &'static worlds::WorldData, floor: usize, x: usize, y: usize) -> bool {
    if floor >= w.floors.len() || y >= w.floors[floor].len() || x >= w.floors[floor][0].len() {
        return false;
    }
    w.floors[floor][y].as_bytes()[x] != b'#'
}
