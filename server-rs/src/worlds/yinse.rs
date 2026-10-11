//! 《银色大地 · 地灵族机界遗迹》任务世界（Z2）静态数据：4 层地图 + 调查点 + 敌人 + NPC + 区域 + 传送门 + 门禁。
//! 设计依据 design/zhttty_universe/honghuang_li/yinse_dadi.md §3（区域划分表 + 传送门接线表 + 门禁设计）为落地权威。
//! 本文件为纯静态世界数据；剧情/战斗见 scenes_yinse.rs（YINSE_SCENES + yinse_figths）。
//! 引擎网格 40 宽×26 高/层；ASCII 为"示意蓝图"，§3.1~§3.4 区域划分表的坐标为落地权威。
//! 出生点 P 在 L1 降落点 (2,13)（PointDef pt_drop 撤离信标位）。
//! 圣位演出红线：东天二皇投影 / 龙族高圣尸骸 / 瓦罗"圣位本体"只做演出（Overlay/cine），不入 enemies 数值表。

use crate::maps;

/// 每行固定 40 字符（# 边框 + 38 格内景）。L1 白银荒原 · 地表尸骸战场（§3.1）。
pub static YINSE_F1_MAP: &[&str] = &[
"########################################",
"#......................................#",
"#....I......II............I............#",
"#......I..............II........I......#",
"#..........I..........................I#",
"#..###......I..............###.........#",
"#............########..................#",
"#.............#....I....#..............#",
"#.............#....#........II.........#",
"#.............###...I...###............#",
"#.................I........I...........#",
"#......................I...............#",
"#.....II.........I......II.............#",
"#.P.....I..........I...................#",
"#....II............I....II.............#",
"#........I..............I..............#",
"#..............I........I..............#",
"#..........###........###..............#",
"#..........#...I...#...................#",
"#..........####################........#",
"#..........#....I...#..................#",
"#..........#........#..................#",
"#......................................#",
"#..........##########.............I....#",
"#..........#...................I.......#",
"########################################",
];

/// L2 地灵族都市遗迹（§3.2）。
pub static YINSE_F2_MAP: &[&str] = &[
"########################################",
"#......................................#",
"#......####..........#####........##...#",
"#......#...I..#........#####....I......#",
"#......#..........#........##..........#",
"#......######......###............I....#",
"#......#....I......#......#............#",
"#......#..................#............#",
"#......######..............######......#",
"#.............I........I...............#",
"#............###............I..........#",
"#............#...I.....#....I..........#",
"#............#........#................#",
"#............#....I...#...........I....#",
"#............###........###............#",
"#........................I.............#",
"#........II.........I......I...........#",
"#.....I..............##.....II.........#",
"#.....#####........##......##..........#",
"#.....#......I...#......#..............#",
"#.....#........................#.......#",
"#.....#####......####.....I....####....#",
"#......................................#",
"#..........I......##########...........#",
"#............I.........................#",
"########################################",
];

/// L3 机界升华工厂（§3.3）。
pub static YINSE_F3_MAP: &[&str] = &[
"########################################",
"#......................................#",
"#....II......I........I.......II.......#",
"#........I......##.........##..........#",
"#........#....I..##.....I..............#",
"#........#........##....##.............#",
"#........#...I.....##....##......I.....#",
"#........#........##........##.........#",
"#........#######...##.....I..##........#",
"#.................I....................#",
"#......................................#",
"#.....I..II....##.....II...............#",
"#........##....##.....##...............#",
"#........##............##..............#",
"#........##....I.....##................#",
"#..........#####..........##...........#",
"#............#....I....#...............#",
"#............#.....##....#.............#",
"#............#.......I....#............#",
"#............########......####........#",
"#....................I.................#",
"#......................................#",
"#.....II....I..........................#",
"#........I....##########....I..........#",
"#..........I..........I................#",
"########################################",
];

/// L4 银色核心 · 瓦罗之墓（§3.4）。
pub static YINSE_F4_MAP: &[&str] = &[
"########################################",
"#......................................#",
"#....II..........I........I............#",
"#........I............II...............#",
"#..........##........I....##...........#",
"#..........##......########..##........#",
"#..........##................##........#",
"#..........##...I............##........#",
"#..........##.................##.......#",
"#..........##.................##.......#",
"#..........#########..........##########",
"#.....................##...............#",
"#....................I.................#",
"#.....................##...............#",
"#..........#########..........##########",
"#..........##.................##.......#",
"#..........##...I............##........#",
"#..........##................##........#",
"#..........###........I....###.........#",
"#..............I........I..............#",
"#......................................#",
"#........I......I.....II...............#",
"#............I.........................#",
"#......................................#",
"#..........I....................I......#",
"########################################",
];

pub static YINSE_FLOOR_NAMES: &[&str] = &[
    "L1 白银荒原 · 地表尸骸战场",
    "L2 地灵族都市遗迹",
    "L3 机界升华工厂",
    "L4 银色核心 · 瓦罗之墓",
];

/// 调查点：全部 id 以 ys_ 前缀，route 引用 scenes_yinse.rs 的 YINSE_SCENES 场景 id。
/// §3 各层点：pt_drop（撤离信标）/dragon_pit（龙尸坑）/war_flags（北废墟战旗）/l2_power_master|b|c（机关链）/
/// l2_home_bones（居民骸骨长街）/l2_vault（隐藏库房）/l3_assembly_line（生产线）/l3_rift_lever（裂缝拉杆机关链末端）/
/// l4_stele（瓦罗石碑）。
pub static POINTS: &[maps::PointDef] = &[
    // ---- L1 ----
    maps::PointDef { id: "ys_pt_drop", name: "撤离信标", floor: 0, x: 2, y: 13, route: "ys_evac_beacon" },
    maps::PointDef { id: "ys_dragon_pit", name: "龙尸坑 · 地灵方解石", floor: 0, x: 30, y: 6, route: "ys_01_drop" },
    maps::PointDef { id: "ys_war_flags", name: "北废墟 · 战旗与情报", floor: 0, x: 17, y: 15, route: "ys_02_war_flags" },
    // ---- L2 ----
    maps::PointDef { id: "ys_power_master", name: "配电塔总控", floor: 1, x: 4, y: 7, route: "ys_06_power_master" },
    maps::PointDef { id: "ys_power_b", name: "配电点 B", floor: 1, x: 24, y: 9, route: "ys_06_power_b" },
    maps::PointDef { id: "ys_power_c", name: "配电点 C", floor: 1, x: 16, y: 22, route: "ys_06_power_c" },
    maps::PointDef { id: "ys_home_bones", name: "居民骸骨长街", floor: 1, x: 16, y: 16, route: "ys_07_home_bones" },
    maps::PointDef { id: "ys_l2_vault", name: "隐藏库房", floor: 1, x: 10, y: 21, route: "ys_08_vault" },
    // ---- L3 ----
    maps::PointDef { id: "ys_assembly_line", name: "三神兵生产线", floor: 2, x: 20, y: 18, route: "ys_09_assembly_line" },
    maps::PointDef { id: "ys_l3_rift_lever", name: "裂缝机关拉杆", floor: 2, x: 32, y: 12, route: "ys_11_rift_lever" },
    // ---- L4 ----
    maps::PointDef { id: "ys_l4_stele", name: "瓦罗石碑", floor: 3, x: 6, y: 3, route: "ys_12_stele" },
];

/// 敌人（§4 敌人表，按层；fight 引用 yinse_figths() 表里的 id）。
/// 圣位红线：龙族高圣 / 东天二皇 / 瓦罗论本体一律不入此表（只做演出场景）。
pub static ENEMIES: &[maps::EnemyDef] = &[
    // ---- L1 白银荒原 ----
    maps::EnemyDef { id: "ys_e_scav1", name: "古兽人拾荒者", floor: 0, x: 6, y: 8, radius: 3, fight: "ws_scav" },
    maps::EnemyDef { id: "ys_e_scav2", name: "古兽人拾荒者", floor: 0, x: 14, y: 21, radius: 3, fight: "ws_scav" },
    maps::EnemyDef { id: "ys_e_brute", name: "战潮碎骨者", floor: 0, x: 26, y: 3, radius: 3, fight: "ws_brute" },
    maps::EnemyDef { id: "ys_e_worm1", name: "银色机械蠕虫", floor: 0, x: 32, y: 14, radius: 2, fight: "ws_worm" },
    maps::EnemyDef { id: "ys_e_worm2", name: "银色机械蠕虫", floor: 0, x: 11, y: 23, radius: 2, fight: "ws_worm" },
    maps::EnemyDef { id: "ys_e_worm3", name: "银色机械蠕虫", floor: 0, x: 24, y: 20, radius: 2, fight: "ws_worm" },
    // ---- L2 都市遗迹 ----
    maps::EnemyDef { id: "ys_e_servant1", name: "失控地灵机仆", floor: 1, x: 14, y: 3, radius: 3, fight: "ws_servant" },
    maps::EnemyDef { id: "ys_e_servant2", name: "失控地灵机仆", floor: 1, x: 30, y: 8, radius: 3, fight: "ws_servant" },
    maps::EnemyDef { id: "ys_e_servant3", name: "失控地灵机仆", floor: 1, x: 8, y: 20, radius: 3, fight: "ws_servant" },
    maps::EnemyDef { id: "ys_e_servant4", name: "失控地灵机仆", floor: 1, x: 26, y: 19, radius: 3, fight: "ws_servant" },
    maps::EnemyDef { id: "ys_e_golem1", name: "符文电偶", floor: 1, x: 21, y: 4, radius: 3, fight: "ws_golem" },
    maps::EnemyDef { id: "ys_e_golem2", name: "符文电偶", floor: 1, x: 12, y: 18, radius: 3, fight: "ws_golem" },
    maps::EnemyDef { id: "ys_e_fused1", name: "机械缝合体", floor: 1, x: 19, y: 16, radius: 3, fight: "ws_fused" },
    maps::EnemyDef { id: "ys_e_fused2", name: "机械缝合体", floor: 1, x: 7, y: 13, radius: 3, fight: "ws_fused" },
    // ---- L3 升华工厂 ----
    maps::EnemyDef { id: "ys_e_guard1", name: "生产线守卫机仆", floor: 2, x: 10, y: 8, radius: 3, fight: "ws_guardline" },
    maps::EnemyDef { id: "ys_e_guard2", name: "生产线守卫机仆", floor: 2, x: 20, y: 15, radius: 3, fight: "ws_guardline" },
    maps::EnemyDef { id: "ys_e_guard3", name: "生产线守卫机仆", floor: 2, x: 29, y: 18, radius: 3, fight: "ws_guardline" },
    maps::EnemyDef { id: "ys_e_pupa", name: "低纬度灾厄之蛹", floor: 2, x: 33, y: 6, radius: 2, fight: "ws_pupa" },
    maps::EnemyDef { id: "ys_e_snake1", name: "裂缝银蛇", floor: 2, x: 30, y: 5, radius: 2, fight: "ws_abyss_snake" },
    maps::EnemyDef { id: "ys_e_snake2", name: "裂缝银蛇", floor: 2, x: 35, y: 7, radius: 2, fight: "ws_abyss_snake" },
    // ---- L4 瓦罗之墓 ----
    maps::EnemyDef { id: "ys_e_warden1", name: "机界守墓枢机", floor: 3, x: 12, y: 8, radius: 3, fight: "ws_warden" },
    maps::EnemyDef { id: "ys_e_warden2", name: "机界守墓枢机", floor: 3, x: 30, y: 15, radius: 3, fight: "ws_warden" },
    maps::EnemyDef { id: "ys_e_worm_r1", name: "银蚴残余", floor: 3, x: 7, y: 21, radius: 2, fight: "ws_worm" },
    maps::EnemyDef { id: "ys_e_worm_r2", name: "银蚴残余", floor: 3, x: 33, y: 19, radius: 2, fight: "ws_worm" },
];

/// NPC：阿桑（人族劫掠队遗孤）+ 小枢（友善地灵族遗民）。talk 引用 scenes_yinse.rs 场景。
pub static NPCS: &[maps::NpcDef] = &[
    maps::NpcDef { id: "ys_n_asang", name: "阿桑（人族劫掠队遗孤）", floor: 0, x: 9, y: 4, talk: "ys_03_asang" },
    maps::NpcDef { id: "ys_n_xiaoshu", name: "小枢（地灵族遗民）", floor: 1, x: 33, y: 5, talk: "ys_07_xiaoshu" },
];

/// 特殊区域（§3）：两段 BOSS 战区 + 顺序机关链 + 升华装置启动间 + 低纬度裂缝 + 决战祭坛 + 东天二皇投影（演出）。
/// kind: "fight"=BOSS/战斗区；"puzzle"=机关；"overlay"=演出（不可战）。
pub static ZONES: &[maps::ZoneDef] = &[
    // L1 战潮王·髅 miniboss（条件触发）
    maps::ZoneDef { id: "ys_z_mini_lou", name: "战潮王 · 髅战场", floor: 0, x: 21, y: 9, kind: "fight", ref_id: "ws_lou" },
    // L2 配电机关链（master→B→C 顺序）
    maps::ZoneDef { id: "ys_z_powerchain", name: "三配电点机关链", floor: 1, x: 4, y: 7, kind: "puzzle", ref_id: "ys_06_power_master" },
    // L2 银蚴巢群（南区污水渠）
    maps::ZoneDef { id: "ys_z_nest", name: "银蚴巢群", floor: 1, x: 31, y: 21, kind: "fight", ref_id: "ws_nest" },
    // L3 升华装置启动间（真相回放 / G3 门禁内）
    maps::ZoneDef { id: "ys_z_sublime", name: "升华装置启动间", floor: 2, x: 13, y: 13, kind: "puzzle", ref_id: "ys_10_sublime" },
    // L3 低纬度裂缝（灾厄区 / 机关链末端拉杆）
    maps::ZoneDef { id: "ys_z_rift", name: "低纬度裂缝", floor: 2, x: 32, y: 6, kind: "puzzle", ref_id: "ys_11_rift_lever" },
    // L4 BOSS 决战祭坛（进场触发两段式）
    maps::ZoneDef { id: "ys_z_waro", name: "决战祭坛 · 瓦罗残响", floor: 3, x: 20, y: 12, kind: "fight", ref_id: "ws_waro_r1" },
    // L4 东天二皇投影（演出，不可战）
    maps::ZoneDef { id: "ys_z_huang", name: "东天二皇投影（演出）", floor: 3, x: 20, y: 5, kind: "overlay", ref_id: "ys_waR0_cast" },
];

/// 传送门（§3.5 接线表；物理单向：仅在起点侧定义 PortalDef 即单向）。
pub static PORTALS: &[maps::PortalDef] = &[
    // G1 电梯井下行：L1(34,20) → L2(36,4)
    maps::PortalDef { id: "ys_pt_down1", floor: 0, x: 34, y: 20, to_floor: 1, tx: 36, ty: 4 },
    // 回程捷径·货运吊索：L2(4,21) → L1(34,2)，单向上行（需 G2 已开）
    maps::PortalDef { id: "ys_pt_up1", floor: 1, x: 4, y: 21, to_floor: 0, tx: 34, ty: 2 },
    // 地铁/管线下行：L2(36,21) → L3(36,4)，单向
    maps::PortalDef { id: "ys_pt_down2", floor: 1, x: 36, y: 21, to_floor: 2, tx: 36, ty: 4 },
    // 逃生梯上行捷径：L3(6,21) → L2(36,21)，单向
    maps::PortalDef { id: "ys_pt_up2", floor: 2, x: 6, y: 21, to_floor: 1, tx: 36, ty: 21 },
    // 主升降井下行：L3(24,23) → L4(24,22)，单向（G4 判定）
    maps::PortalDef { id: "ys_pt_down3", floor: 2, x: 24, y: 23, to_floor: 3, tx: 24, ty: 22 },
    // 低纬度裂缝传送：L3(32,12) → L4(33,7)，单向（需 flag rift_open，san-15 走此路）
    maps::PortalDef { id: "ys_pt_rift", floor: 2, x: 32, y: 12, to_floor: 3, tx: 33, ty: 7 },
    // 撤离传送门：L4(20,23) → 主神空间（BOSS 胜利后激活；Route 由 scenes 处理）
    maps::PortalDef { id: "ys_pt_exit", floor: 3, x: 20, y: 23, to_floor: 0, tx: 2, ty: 13 },
];

/// 门禁（§3.6）：G1 电梯井 / G2 符文闸门 / G3 升华装置启动间 / G4 瓦罗之墓主门。
/// 状态存 st.map_objs[gate_id]。
pub static GATES: &[maps::GateDef] = &[
    // G1 电梯井：L1(33,21)，需道具"地灵方解石"（龙尸坑调查获得）
    maps::GateDef {
        id: "ys_g_ele1", name: "电梯井门禁", floor: 0, x: 33, y: 21,
        need_item: Some("item_diling"), need_flag: None,
        lock_msg: "电梯井的铁闸死死咬合。闸缝里卡着一枚银白结晶体——仿佛在向携带<b>地灵方解石</b>的人现身。",
        unlock_msg: "你将地灵方解石贴上台面的浅槽，「咔」一声轻响，铁闸升起，黑暗的电梯井向下延伸。",
    },
    // G2 符文闸门：L2(28,12)，需 flag l2_power_restored（顺序机关链）
    maps::GateDef {
        id: "ys_g_runegate", name: "符文闸门", floor: 1, x: 28, y: 12,
        need_item: None, need_flag: Some("ys_l2_power_restored"),
        lock_msg: "半扇巨型符文闸门横亘当路，表面的地灵符文黯淡无光——<em>供电未恢复，闸门死锁</em>。",
        unlock_msg: "全城灯光次第亮起，符文闸门上的光芒自下而上流淌，轰然升开。都市中心大道、隐藏库房与回程吊索全部畅通。",
    },
    // G3 升华装置启动间：L3(13,13)，需道具"三神兵·机核碎片"
    maps::GateDef {
        id: "ys_g_sublime", name: "升华装置启动间", floor: 2, x: 13, y: 13,
        need_item: Some("item_jiche"), need_flag: None,
        lock_msg: "升华装置的门由熔金封死。一枚不规则的机核碎片形状嵌在门心——<em>需要三神兵·机核碎片嵌入</em>。",
        unlock_msg: "机核碎片嵌入熔金凹槽，门心泛起幽蓝圣光。机界升华装置沉睡了两百年的转子，开始缓慢转动。",
    },
    // G4 瓦罗之墓主门：L3(24,23)，需 flag waro_truth 或道具"三神兵·机核碎片"
    maps::GateDef {
        id: "ys_g_core", name: "瓦罗之墓主门", floor: 2, x: 24, y: 23,
        need_item: None, need_flag: Some("ys_core_open"), // truth OR 机核碎片 → 场景设此 flag（见 ys_10_sublime / ys_11_rift_lever）
        lock_msg: "通往银色核心的主升降井被圣力封死。碑文刻着：<em>「持机核者，或知我名者，方可下行。」</em>",
        unlock_msg: "圣力感应到你洞悉的真相或怀中的机核，封禁缓缓瓦解，主升降井的轰隆声自深处传来。",
    },
];