//! 《魔戒·摩瑞亚矿坑》任务世界（slug：moruiya）：3 层
//! F1 西闸门与矮人故居 / F2 长书库与无底阶梯 / F3 卡扎督姆桥与王厅宝库。
//! 设计依据 design/zhttty_universe/wuxian_kongbu/moruiya.md §3（坐标/门禁/传送门为落地权威）。
//! 本文件为纯静态世界数据（地图 + 调查点 + NPC + 敌人 + 传送门 + 门禁），剧情/战斗见 scenes_moruiya.rs。
//! 引擎网格 40 宽×26 高/层；ASCII 为"示意蓝图"，§3.2-3.4 区域划分表坐标为落地权威。
//! 出生点 P 在 F1 西闸门内侧 (12,1)；身后 G1 西闸门 (3,5) 为单向死锁（进得来出不去，呼应电影里监视者封门）。
//!
//! 设计口径对照：
//! - POINTS ids 全 `mo_p_` 前缀，route 引用 scenes_moruiya.rs 场景 id（mo_*）。
//! - ENEMIES ids 全 `mo_e_` 前缀，fight 引用 moruiya_figths() 表 id（goblin_* / watcher / balrog 等）。
//! - NPCS ids 全 `mo_n_` 前缀（甘道夫 F1、吉姆利 F2、波罗莫 F1）。
//! - ZONES ids 全 `mo_z_` 前缀（watcher 湖池 / bridge_mid 桥中段 / troll 巨魔口）。
//! - PORTALS：P3/P4 楼梯双向对、P1 矿车单向、P2 塌方缝隙单向。
//! - GATES：G1 西闸门 / G2 柱厅大门 / G3 塌方 / G4 书库大门 / G5 宝库门 / G6 东门。
//!
//! 地图采用开放连通的落点式布局（§3 层内为连通地下室厅，调查点/门禁/传送门均以对象表承载），
//! 关键坐标全部落于可行走格，保证从出生点可达；`I` 为火把/石柱/石棺装饰。

use crate::maps;

/// F1 西闸门与矮人故居（40×26）。出生点 P(12,1)。开放柱厅，左下湖池监视者区、月台、石梯向 F2。
pub static MORUIYA_F1_MAP: &[&str] = &[
"########################################",
"#I.....I....P....I.............I......I#",
"#.....##....###......###......####..####",
"#.....#.#........#......#......#....#..#",
"#.....#.#..I....#......#......#.....#..#",
"#..G..#.#........#......#......#....#..#",
"#.....#.#........#..I...#......#....#..#",
"#.....#.#....I..#......#......#.....#..#",
"#.....#.#........#......#......#....#..#",
"#.....##...G...###.....##......#....#..#",
"#..........I....#..........#.....#....##",
"#......I.......#....I.....#......#....##",
"#......#.......#..........#......#....##",
"#......#.......#..........#..G...#....##",
"#......#.......#..........#......#S...##",
"#.....#..I.....#...........#.....#....##",
"#.....#.........#....I.....#.....#....##",
"#.....#.........#..........#.....#....##",
"#.....#........##..####....#.....#....##",
"#.....#....................#.....#....##",
"#.....#..............C.....#.....#....##",
"#.....#......W.............#.....#....##",
"#.....#####################......#....##",
"#................................#....##",
"#I.....I.....I......I......I.....I....I#",
"########################################",
];

/// F2 长书库与无底阶梯（40×26）。上行点 U(1,1)←F1；书库大厅（巴林墓/书）、书库大门 G4、无底阶梯区、密室、楼梯 S→F3。
pub static MORUIYA_F2_MAP: &[&str] = &[
"########################################",
"#U.....I....I....I......I.............C#",
"#......#........#..........#..........##",
"#......#...I....#....I.....#...I......##",
"#......#........#..........#..........##",
"#......###...####...######............##",
"#...............#..........#..........##",
"#......I.......#....I.....#....I......##",
"#...............#..........#..........##",
"#...............##...###....#####....###",
"#..................#.................###",
"#.......I.........#.......I.........####",
"#..................#.................###",
"#.....##........###................#..##",
"#..............#..........#......#....##",
"#......I.......#....I.....#......#....##",
"#..............#..........#......#....##",
"#......#########..........####...#....##",
"#.....................................##",
"#......I.....I.....I..........I......###",
"#.....................................##",
"#......I.....I.....I......I.....I.....##",
"#.....................................##",
"#......I.....I.....I......I.....I.....##",
"#I.....I.....I......I......I.....I....I#",
"########################################",
];

/// F3 卡扎督姆桥与王厅宝库（40×26）。上行点 U(1,1)←F2；桥中段 ZoneDef；东门 G6；宝库支线右下。
pub static MORUIYA_F3_MAP: &[&str] = &[
"########################################",
"#U......I....I.......I........I........#",
"#......#........#..........#..........##",
"#......#...I....#....I.....#...I......##",
"#......#........#..........#..........##",
"#......###...####...######............##",
"#...............#..........#..........##",
"#......I.......#....I.....#....I......##",
"#...............#..........#..........##",
"#...............##...###....#####....###",
"#..................#.................###",
"#.......I.........#.......I.........####",
"#..................#.................###",
"#....I............................I....#",
"#....##############################....#",
"#...###..........................###...#",
"#.....#................................#",
"#.....##############################...#",
"#.....................................##",
"#......I.....I.....I............########",
"#...............................#I....##",
"#......I.....I.....I......I.....#.....##",
"#...............................#I....##",
"#......I.....I.....I......I.....########",
"#I.....I.....I......I......I.....I....I#",
"########################################",
];

pub static MORUIYA_FLOOR_NAMES: &[&str] = &[
    "F1 西闸门与矮人故居",
    "F2 长书库与无底阶梯",
    "F3 卡扎督姆桥与王厅宝库",
];

/// 调查点：全部 id 以 mo_ 前缀，route 引用 scenes_moruiya.rs 的 MORUIYA_SCENES 场景 id。
/// 对应设计 §3.2-3.4 关键调查坐标（石板/塌方/湖岸/巴林之书/阶梯陷阱×3/密室宝箱/桥裂隙/秘银宝箱）。
pub static POINTS: &[maps::PointDef] = &[
    // ---- F1 ----
    maps::PointDef { id: "mo_p_lake", name: "西闸门 · 湖岸", floor: 0, x: 13, y: 21, route: "mo_lake" },
    maps::PointDef { id: "mo_p_slab", name: "柱厅 · 刻字石板", floor: 0, x: 26, y: 10, route: "mo_rune" },
    maps::PointDef { id: "mo_p_collapse", name: "北廊 · 塌方", floor: 0, x: 11, y: 9, route: "mo_collapse" },
    maps::PointDef { id: "mo_p_cart", name: "月台 · 矿车补给箱", floor: 0, x: 21, y: 20, route: "mo_cart" },
    maps::PointDef { id: "mo_p_stairs", name: "楼梯口 · 下 F2", floor: 0, x: 34, y: 14, route: "mo_02_hall" },
    // ---- F2 ----
    maps::PointDef { id: "mo_p_book", name: "书库 · 石棺/《马扎布尔之书》", floor: 1, x: 20, y: 10, route: "mo_book" },
    maps::PointDef { id: "mo_p_trap1", name: "无底阶梯 · 一步踏空·一", floor: 1, x: 10, y: 20, route: "mo_stair" },
    maps::PointDef { id: "mo_p_trap2", name: "无底阶梯 · 一步踏空·二", floor: 1, x: 22, y: 22, route: "mo_stair" },
    maps::PointDef { id: "mo_p_trap3", name: "无底阶梯 · 一步踏空·三", floor: 1, x: 34, y: 20, route: "mo_stair" },
    maps::PointDef { id: "mo_p_chest", name: "密室 · 秘银钥匙石宝箱", floor: 1, x: 4, y: 24, route: "mo_chest" },
    maps::PointDef { id: "mo_p_stairs2", name: "楼梯口 · 下 F3", floor: 1, x: 31, y: 23, route: "mo_book" },
    // ---- F3 ----
    maps::PointDef { id: "mo_p_crack", name: "桥中段 · 裂隙", floor: 2, x: 22, y: 13, route: "mo_bridge_desc" },
    maps::PointDef { id: "mo_p_vault_chest", name: "王厅宝库 · 秘银宝箱", floor: 2, x: 33, y: 21, route: "mo_vault" },
];

/// 敌人（稀疏驻守：F1 半兽人斥候/巡逻队；F2 半兽人掠夺者/鼓声伏击、巨魔口；F3 半兽人禁卫）。
/// fight 引用 moruiya_figths() 表里的 id。watcher / balrog 走 ZoneDef 直战（可选遭遇/BOSS 战场），不在此驻守。
pub static ENEMIES: &[maps::EnemyDef] = &[
    // F1
    maps::EnemyDef { id: "mo_e_scout", name: "哥布林斥候", floor: 0, x: 29, y: 14, radius: 3, fight: "goblin_scout" },
    maps::EnemyDef { id: "mo_e_pack", name: "半兽人巡逻队", floor: 0, x: 8, y: 18, radius: 4, fight: "goblin_pack" },
    // F2
    maps::EnemyDef { id: "mo_e_raider", name: "半兽人掠夺者", floor: 1, x: 34, y: 6, radius: 3, fight: "goblin_raider" },
    maps::EnemyDef { id: "mo_e_ambush", name: "鼓声伏击·半兽人群", floor: 1, x: 25, y: 11, radius: 4, fight: "drum_ambush" },
    maps::EnemyDef { id: "mo_e_troll", name: "洞穴巨魔", floor: 1, x: 7, y: 18, radius: 3, fight: "cave_troll" },
    // F3
    maps::EnemyDef { id: "mo_e_guard1", name: "半兽人禁卫", floor: 2, x: 12, y: 16, radius: 3, fight: "orc_guard" },
    maps::EnemyDef { id: "mo_e_guard2", name: "半兽人禁卫", floor: 2, x: 14, y: 20, radius: 3, fight: "orc_guard" },
];

/// NPC：甘道夫（F1 引导）、波罗莫（F1）、吉姆利（F2 书库）。talk 引用 scenes_moruiya.rs 场景。
pub static NPCS: &[maps::NpcDef] = &[
    maps::NpcDef { id: "mo_n_gandalf", name: "甘道夫", floor: 0, x: 12, y: 3, talk: "mo_npc_gandalf" },
    maps::NpcDef { id: "mo_n_boromir", name: "波罗莫", floor: 0, x: 18, y: 12, talk: "mo_npc_boromir" },
    maps::NpcDef { id: "mo_n_gimli", name: "吉姆利", floor: 1, x: 22, y: 7, talk: "mo_npc_gimli" },
];

/// 特殊区域：watcher 湖池（可选 mini-BOSS）/ bridge_mid 桥中段（炎魔 BOSS 战场）/ troll 巨魔口（越级精英闲聊）。
pub static ZONES: &[maps::ZoneDef] = &[
    maps::ZoneDef { id: "mo_z_watcher", name: "水中监视者 · 湖池", floor: 0, x: 13, y: 21, kind: "fight", ref_id: "mo_lake" },
    maps::ZoneDef { id: "mo_z_troll", name: "无底阶梯口 · 巨魔", floor: 1, x: 7, y: 18, kind: "fight", ref_id: "mo_npc_troll" },
    maps::ZoneDef { id: "mo_z_bridge", name: "卡扎督姆桥 · 中段", floor: 2, x: 22, y: 13, kind: "fight", ref_id: "b_balrog" },
];

/// 传送门（§3.5，物理单向：PortalDef 仅在起点侧定义，反向无门即单向）。
pub static PORTALS: &[maps::PortalDef] = &[
    // P3 楼梯（双向）：F1 (34,14) ↔ F2 (1,1) —— 两条反向对实现双向
    maps::PortalDef { id: "mo_pt_stair_up", floor: 0, x: 34, y: 14, to_floor: 1, tx: 1, ty: 1 },
    maps::PortalDef { id: "mo_pt_stair_down", floor: 1, x: 1, y: 1, to_floor: 0, tx: 34, ty: 14 },
    // P4 楼梯（双向）：F2 (31,23) ↔ F3 (1,1)
    maps::PortalDef { id: "mo_pt_stair_up2", floor: 1, x: 31, y: 23, to_floor: 2, tx: 1, ty: 1 },
    maps::PortalDef { id: "mo_pt_stair_down2", floor: 2, x: 1, y: 1, to_floor: 1, tx: 31, ty: 23 },
    // P1 矿车下滑（单向）：F2 (38,1) 月台 → F1 (21,20) 月台，缆索单行不可上爬
    maps::PortalDef { id: "mo_pt_cart_slide", floor: 1, x: 38, y: 1, to_floor: 0, tx: 21, ty: 20 },
    // P2 塌方缝隙（单向）：F3 (6,20) 桥下 → F2 (4,24) 密室，缝隙过高不可爬回
    maps::PortalDef { id: "mo_pt_collapse_fall", floor: 2, x: 6, y: 20, to_floor: 1, tx: 4, ty: 24 },
];

/// 门禁（§3.6）：箱庭软锁，解锁条件为道具/item 或 flag，状态存 st.map_objs[gate_id]。
pub static GATES: &[maps::GateDef] = &[
    // G1 西闸门：回程单一封锁（剧情后设置 flag mo_gate_sealed，无处可绕，呼应「后路断绝」）
    maps::GateDef {
        id: "mo_g_g1", name: "西闸门", floor: 0, x: 3, y: 5,
        need_item: None, need_flag: Some("mo_gate_sealed"),
        lock_msg: "西闸门内侧，湖水与触手把门缝封死了——后路断绝。只能向黑暗深处走。",
        unlock_msg: "（此门为单向死锁，任何条件均无法从内侧开启）",
    },
    // G2 柱厅大门：东厅直路，需 flag mo_rune_decoded（石板解密成功）
    maps::GateDef {
        id: "mo_g_g2", name: "柱厅大门", floor: 0, x: 29, y: 13,
        need_item: None, need_flag: Some("mo_rune_decoded"),
        lock_msg: "厚重的石门刻满矮人符文，石门吨位非人力可撼——但门槛上的凹槽，与石板上的咒文严丝合缝。",
        unlock_msg: "你按石板咒文的顺序转动石钮——轰隆一声，柱厅大门缓缓碾开。",
    },
    // G3 塌方：北廊捷径，需 flag mo_collapse_cleared（调查清理塌方，Hurt(-10)）
    maps::GateDef {
        id: "mo_g_g3", name: "塌方", floor: 0, x: 11, y: 9,
        need_item: None, need_flag: Some("mo_collapse_cleared"),
        lock_msg: "碎石封死了北廊的捷径。巨石堆成一道屏障，绕行南廊要多打一场遭遇战。",
        unlock_msg: "你撬开卡死的石梁，碎石如瀑落下——捷径通了，代价是体力被磨去一线。",
    },
    // G4 书库大门：无底阶梯段，需 flag mo_book_read（读完《马扎布尔之书》）
    maps::GateDef {
        id: "mo_g_g4", name: "书库大门", floor: 1, x: 25, y: 9,
        need_item: None, need_flag: Some("mo_book_read"),
        lock_msg: "书库大门的机簧锈死。《马扎布尔之书》的残页里似乎记着门锁的机要——没读完书，这道门开不了。",
        unlock_msg: "你按书中记载的机簧暗语转动门轴——书库大门轰然洞开，深井里的鼓声随之更近了。",
    },
    // G5 宝库门：王厅宝库（纯支线），需道具 mithril_key（F2 密室取得）
    maps::GateDef {
        id: "mo_g_g5", name: "王厅宝库", floor: 2, x: 28, y: 19,
        need_item: Some("mithril_key"), need_flag: None,
        lock_msg: "秘银包边的石门纹丝不动。锁孔的形状与你在书库密室捡到的钥匙石一模一样。",
        unlock_msg: "秘银钥匙石贴合锁孔，一声叮的脆响——王厅宝库向你们敞开。",
    },
    // G6 东门：主线终点，需 flag mo_cleared（炎魔任一结局后设置）
    maps::GateDef {
        id: "mo_g_g6", name: "东门", floor: 2, x: 36, y: 13,
        need_item: None, need_flag: Some("mo_cleared"),
        lock_msg: "东门的巨闸半掩在晨光里，但机杼纹丝不动——炎魔未灭，这门不会为你们打开。",
        unlock_msg: "炎魔已坠入深渊。东门机杼「轧」地转动，晨光如河涌进——你们做到了。",
    },
];