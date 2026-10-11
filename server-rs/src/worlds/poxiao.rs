//! 《无限曙光 · 破晓封锁区》（poxiao）：封锁城区街道 → 地下排水道与叛军据点 → 黎明尖塔。3 层。
//! 设计依据 design/zhttty_universe/wuxian_shuguang/shixue_poxiao.md（§3 地图 / §5 BOSS / §6 剧情 / §10 实现风险）。
//! 世界方向：嗜血破晓世界观下的黎明之城——人类与血族对峙、沉沦者失控的末世 D 级绝境任务。
//! 钩子：「太阳快出来了——这对某些人，是末日。」
//! 三方势力抉择（人类叛军 / 温和血族 / 中立独行）互斥 flag；日光倒计时与日光射线终结均用剧情 flag 降级，零新引擎。
//! 本文件为纯静态世界数据，剧情/战斗见 scenes_poxiao.rs。
//! 引擎网格 40 宽×26 高/层；ASCII 为示意蓝图，POINTS/ENEMIES/NPCS/ZONES/PORTALS/GATES 坐标为落地权威。
//! 出生点 P 在 L1 (4,24)；单向进深（L1→L2→L3）+ 一条 L2→L1 回程捷径缝合回溯。

use crate::maps;

/// 每行固定 40 字符。L1 封锁城区街道：紫夜深巷、教会收容所、废弃血站、钟楼广场、十字路口、地铁口。
pub static POXIAO_F1_MAP: &[&str] = &[
"########################################",
"#..........................I...........#",
"#.#########........I........############",
"#.#.......#.................#..........#",
"#.#..I....#.................#..I...I...#",
"#.#.......#.................#.....I....#",
"#.#.......#....I............#..........#",
"#.#.......#.............I...#..........#",
"#.#.......#.................#..........#",
"#.####.####.................#.##########",
"#.............................I.....I..#",
"#.............I..........I.............#",
"#........................I........I....#",
"#...........I..............I...........#",
"#..#######...............#######.......#",
"#..#.....#...............#.....#.......#",
"#..#..I..#...............#.....#....I..#",
"#..###.###...............###.#.#.......#",
"#............I.........................#",
"#.......................I..............#",
"#...........I............I.............#",
"#..IIIII..........................I....#",
"#..IIIII...............................#",
"#..IIIII...............................#",
"#...P..................................#",
"########################################",
];

/// L2 地下排水道与叛军据点：叛军据点、主排水道、泵房、深水渠巢穴、发电机房、货运电梯。
pub static POXIAO_F2_MAP: &[&str] = &[
"########################################",
"#......................................#",
"#..############........................#",
"#..#..........#........................#",
"#..#..I.......#........................#",
"#..#..........#........................#",
"#..#..........#........................#",
"#..#....I.....#........................#",
"#..#..........#........................#",
"#..###.########........................#",
"#......................................#",
"#...........I....I.....................#",
"#......................................#",
"#..........I.......I...................#",
"#..######...................#########..#",
"#..#....#...................#.......#..#",
"#..#.I..#...................#...I...#..#",
"#..#..I.#...................#.......#..#",
"#..##.###...................#.......#..#",
"#...........................#.##.####..#",
"#......................................#",
"#.....I.......I........................#",
"#......................................#",
"#................................P.....#",
"#......................................#",
"########################################",
];

/// L3 黎明尖塔：尖塔大堂、中层实验区/档案室、顶层决战平台（玻璃穹顶 + 镜阵列 + BOSS）。
pub static POXIAO_F3_MAP: &[&str] = &[
"########################################",
"#......................................#",
"#.....############################.....#",
"#.....#.....I....................#.....#",
"#.....#...........I.....I........#.....#",
"#.....#..........................#.....#",
"#.....##.#####################.###.....#",
"#......................................#",
"#....##########..........##########....#",
"#....#........#..........#........#....#",
"#....#.I......#..........#.....I..#....#",
"#....#..I.....#..........#........#....#",
"#....#........#..........#........#....#",
"#....#........#..........#........#....#",
"#....#........#..........#........#....#",
"#....###.######..........###.#....#....#",
"#......................................#",
"#......................................#",
"#...........I..............I...........#",
"#......................................#",
"#.......##########..############.......#",
"#.......#......................#.......#",
"#....I..#......................#...I...#",
"#.......#......................#.......#",
"#.......##########P.############.......#",
"########################################",
];

pub static POXIAO_FLOOR_NAMES: &[&str] = &[
    "L1 封锁城区街道",
    "L2 地下排水道与叛军据点",
    "L3 黎明尖塔",
];

/// 调查点：全部 id 以 px_p_ 前缀，route 引用 scenes_poxiao.rs 的 POXIAO_SCENES 场景 id。
pub static POINTS: &[maps::PointDef] = &[
    // ---- L1 封锁城区街道 ----
    maps::PointDef { id: "px_p_altar", name: "圣坛笔记", floor: 0, x: 6, y: 4, route: "px_altar" },
    maps::PointDef { id: "px_p_bloodbank", name: "血站终端", floor: 0, x: 33, y: 4, route: "px_bloodbank" },
    maps::PointDef { id: "px_p_belltower", name: "钟楼调查", floor: 0, x: 18, y: 2, route: "px_belltower" },
    maps::PointDef { id: "px_p_apt_letter", name: "空屋信件", floor: 0, x: 6, y: 15, route: "px_apt_letter" },
    maps::PointDef { id: "px_p_store_relic", name: "店主遗物", floor: 0, x: 26, y: 16, route: "px_store_relic" },
    maps::PointDef { id: "px_p_ruin", name: "坍塌废墟", floor: 0, x: 5, y: 22, route: "px_ruin" },
    // ---- L2 地下排水道与叛军据点 ----
    maps::PointDef { id: "px_p_commpost", name: "通讯台", floor: 1, x: 8, y: 6, route: "px_rebels" },
    maps::PointDef { id: "px_p_armory", name: "军械库", floor: 1, x: 12, y: 4, route: "px_armory" },
    maps::PointDef { id: "px_p_valve_console", name: "泵房控制台", floor: 1, x: 4, y: 15, route: "px_pump_console" },
    maps::PointDef { id: "px_p_valve_a", name: "阀门 A", floor: 1, x: 5, y: 16, route: "px_valve_a" },
    maps::PointDef { id: "px_p_valve_b", name: "阀门 B", floor: 1, x: 6, y: 17, route: "px_valve_b" },
    maps::PointDef { id: "px_p_nest", name: "巢穴调查", floor: 1, x: 20, y: 16, route: "px_nest" },
    maps::PointDef { id: "px_p_generator", name: "发电机", floor: 1, x: 32, y: 16, route: "px_generator" },
    // ---- L3 黎明尖塔 ----
    maps::PointDef { id: "px_p_reception", name: "接待台", floor: 2, x: 19, y: 23, route: "px_reception" },
    maps::PointDef { id: "px_p_lab_log", name: "实验记录", floor: 2, x: 9, y: 13, route: "px_lab_log" },
    maps::PointDef { id: "px_p_archive", name: "机密档案", floor: 2, x: 29, y: 13, route: "px_archive" },
    maps::PointDef { id: "px_p_mirror_l", name: "左镜", floor: 2, x: 12, y: 3, route: "px_mirror_l" },
    maps::PointDef { id: "px_p_mirror_r", name: "右镜", floor: 2, x: 24, y: 4, route: "px_mirror_r" },
    maps::PointDef { id: "px_p_mirror_c", name: "主控镜阵", floor: 2, x: 18, y: 4, route: "px_mirror_c" },
];

/// 敌人：fight 引用 poxiao_figths() 表里的 id（立绘复用：guard→守卫/血族、hunter→沉沦者）。
pub static ENEMIES: &[maps::EnemyDef] = &[
    // ---- L1 ----
    maps::EnemyDef { id: "px_e1_v1", name: "平民吸血鬼", floor: 0, x: 8, y: 12, radius: 3, fight: "pc_vamp_civil" },
    maps::EnemyDef { id: "px_e1_v2", name: "平民吸血鬼", floor: 0, x: 26, y: 11, radius: 3, fight: "pc_vamp_civil" },
    maps::EnemyDef { id: "px_e1_guard1", name: "血站守卫", floor: 0, x: 31, y: 10, radius: 3, fight: "pc_guard" },
    maps::EnemyDef { id: "px_e1_guard2", name: "血站守卫", floor: 0, x: 24, y: 5, radius: 3, fight: "pc_guard" },
    maps::EnemyDef { id: "px_e1_deg", name: "沉沦者", floor: 0, x: 16, y: 8, radius: 3, fight: "pc_degenerate" },
    maps::EnemyDef { id: "px_e1_horde1", name: "沉沦者群", floor: 0, x: 19, y: 16, radius: 3, fight: "pc_degenerate_horde" },
    // ---- L2 ----
    maps::EnemyDef { id: "px_e2_patrol1", name: "血族巡逻队", floor: 1, x: 28, y: 12, radius: 3, fight: "pc_vamp_patrol" },
    maps::EnemyDef { id: "px_e2_patrol2", name: "血族巡逻队", floor: 1, x: 14, y: 13, radius: 3, fight: "pc_vamp_patrol" },
    maps::EnemyDef { id: "px_e2_elite", name: "嗜血沉沦者·精英", floor: 1, x: 30, y: 15, radius: 3, fight: "pc_elite" },
    maps::EnemyDef { id: "px_e2_nest1", name: "巢穴沉沦者", floor: 1, x: 19, y: 15, radius: 3, fight: "pc_degenerate" },
    maps::EnemyDef { id: "px_e2_nest2", name: "巢穴沉沦者", floor: 1, x: 21, y: 17, radius: 3, fight: "pc_degenerate" },
    // ---- L3 ----
    maps::EnemyDef { id: "px_e3_guard1", name: "尖塔卫队", floor: 2, x: 10, y: 5, radius: 3, fight: "pc_spire_guard" },
    maps::EnemyDef { id: "px_e3_guard2", name: "尖塔卫队", floor: 2, x: 26, y: 5, radius: 3, fight: "pc_spire_guard" },
    maps::EnemyDef { id: "px_e3_guard3", name: "尖塔卫队", floor: 2, x: 29, y: 12, radius: 3, fight: "pc_spire_guard" },
    maps::EnemyDef { id: "px_e3_boss", name: "高级沉沦者·格里高尔", floor: 2, x: 18, y: 5, radius: 3, fight: "pc_boss_gregor" },
];

/// NPC：道尔顿 / 奥黛丽 / 埃尔维斯 / 埃德加（talk 走 px_* 剧情线）。
pub static NPCS: &[maps::NpcDef] = &[
    maps::NpcDef { id: "px_n_dalton", name: "爱德华·道尔顿", floor: 0, x: 6, y: 5, talk: "px_dalton" },
    maps::NpcDef { id: "px_n_audrey", name: "奥黛丽·班尼特", floor: 1, x: 9, y: 7, talk: "px_rebels" },
    maps::NpcDef { id: "px_n_elvis", name: "埃尔维斯", floor: 1, x: 5, y: 5, talk: "px_elvis" },
    maps::NpcDef { id: "px_n_edgar", name: "埃德加·冯·豪森", floor: 2, x: 30, y: 5, talk: "px_edgar_deal" },
];

/// 特殊区域：L1 十字路口沉沦者群 Zone + L3 实验笼挣脱沉沦者 Zone。
pub static ZONES: &[maps::ZoneDef] = &[
    maps::ZoneDef { id: "px_z_l1_cross", name: "十字路口 · 伏击", floor: 0, x: 19, y: 13, kind: "fight", ref_id: "pc_degenerate_horde" },
    maps::ZoneDef { id: "px_z_l3_cage", name: "实验笼 · 挣脱", floor: 2, x: 8, y: 11, kind: "fight", ref_id: "pc_degenerate" },
];

/// 传送门（物理单向：PortalDef 仅在起点侧定义，反向无门即单向）。
pub static PORTALS: &[maps::PortalDef] = &[
    // p_px_1 L1 地铁口 → L2 到达点（单向向下）
    maps::PortalDef { id: "p_px_1", floor: 0, x: 33, y: 21, to_floor: 1, tx: 33, ty: 23 },
    // p_px_2 L1 回程：L2 到达点同格反向 → L1 地铁口（单向向上；回溯捷径）
    maps::PortalDef { id: "p_px_2", floor: 1, x: 33, y: 23, to_floor: 0, tx: 33, ty: 21 },
    // p_px_3 L2 货运电梯 → L3 尖塔大堂（单向向上，需 poxiao_generator 通电）
    maps::PortalDef { id: "p_px_3", floor: 1, x: 5, y: 21, to_floor: 2, tx: 18, ty: 24 },
];

/// 门禁（GateDef 软锁）：G1 军械库 / G2 水闸 / G3 电梯闸 / G4+G5 顶层闸门（决战前置）。
pub static GATES: &[maps::GateDef] = &[
    // G1 军械库：需血浆样本×2 或 人类路线 flag
    maps::GateDef {
        id: "px_g1_armory", name: "叛军军械库", floor: 1, x: 11, y: 4,
        need_item: None, need_flag: Some("px_armory_open"),
        lock_msg: "军械库的钢板门焊死了。铭牌：<em>【需血浆样本×2 或 人类阵营许可】</em>——你没有凑齐兑换物或阵营资格，撬不动它。",
        unlock_msg: "铁门轰然解锁。你推开军械库，弹箱与圣水弹药码放整齐，弹药味扑面而来。",
    },
    // G2 水闸：通往深水渠巢穴，需阀门谜题完成 flag
    maps::GateDef {
        id: "px_g2_sluice", name: "泵房水闸", floor: 1, x: 17, y: 16,
        need_item: None, need_flag: Some("px_valves"),
        lock_msg: "厚重的防水水闸死死压着。透过有机玻璃，另一侧积水翻着浊光——<em>阀门未全部排干前，这道闸无法升起</em>。",
        unlock_msg: "泵房电机轰鸣，水位退去，水闸缓缓升起——深水渠巢穴的捷径通了。",
    },
    // G3 电梯闸：货运电梯需发电机通电 flag
    maps::GateDef {
        id: "px_g3_elevator", name: "货运电梯闸", floor: 1, x: 4, y: 21,
        need_item: None, need_flag: Some("px_generator"),
        lock_msg: "货运电梯的电力闸刀被拉开了，指示灯一片死寂。<em>电梯未通电，无法上行尖塔</em>——先去发电机房推上电源。",
        unlock_msg: "备用电流沿着线路扑来，电梯指示灯亮起，轿厢门「嘀」地滑开，通往尖塔的上升通道开启。",
    },
    // G4/G5 顶层闸门：决战前置，需档案弱点情报 flag
    maps::GateDef {
        id: "px_g4_top_west", name: "顶层西闸门", floor: 2, x: 8, y: 6,
        need_item: None, need_flag: Some("px_archive"),
        lock_msg: "顶层闸门的电子锁闪烁红光：<em>【需要权限档案】</em>——你还未解密机密档案，无法进入决战平台。",
        unlock_msg: "你出示解密的档案权限，西闸门液压杆升起，镜阵冷光自决战平台泻出。",
    },
    maps::GateDef {
        id: "px_g5_top_east", name: "顶层东闸门", floor: 2, x: 30, y: 6,
        need_item: None, need_flag: Some("px_archive"),
        lock_msg: "顶层闸门的电子锁闪烁红光：<em>【需要权限档案】</em>——你还未解密机密档案，无法进入决战平台。",
        unlock_msg: "你出示解密的档案权限，东闸门液压杆升起，决战平台的穹顶透出黎明前的微光。",
    },
];