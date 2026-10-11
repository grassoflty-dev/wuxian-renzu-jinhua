//! 《异形4·奥瑞迦号》任务世界（slug：yiying，全局 id 常量 WORLD_YIYING 由主线注册）。
//! 3 层：L1 船员生活区 / L2 实验室与孵化室 / L3 引擎区与皇后巢穴。
//! 设计依据 design/zhttty_universe/wuxian_kongbu/yiying.md §3（区域划分表坐标为权威）。
//! 本文件为纯静态世界数据（地图 + 调查点 + NPC + 敌人 + 传送门 + 门禁），剧情/战斗见 scenes_yiying.rs。
//! 引擎网格 40 宽×26 高/层；ASCII 为"示意蓝图"，区域划分表的坐标为落地权威（装饰位允许 ±1）。
//! 所有对象 id 前缀 `yiy_`（设计文档 §3 顶部注明可实现为 `yiy_`，研究对象表亦用 `yiy_`）。

use crate::maps;

/* =====================================================================
   待主线排期的素材替换清单（本副本无专属 bg/立绘素材）
   ---------------------------------------------------------------------
   [背景图 bg]（先引用已部署图，注明"待替换"）
   - 船舱走廊/主控室（L1~L3 通用、Father 主控室）→ 现用 `img_corridor.png`（待换：冷金属幽蓝走廊/屏幕墙，目标 `yiy_bg_corridor.png`）
   - 主神/贝蒂号货舱（开场）→ 现用 `img_zhangjie.png`（待换：太空货舱，目标 `yiy_bg_cargo.png`）
   - 孵化室/卵区（L2 B5）→ 现用 `img_horde.png`（待换：荧光绿黏液壁+半透膜卵苞，目标 `yiy_bg_incubator.png`）
   - 皇后巢穴（L3 C4）→ 现用 `img_laser.png`（待换：骨白穹顶+卵房"胚胎教堂"，目标 `yiy_bg_quee_nest.png`）
   - 反应堆/终局（L3 C2/引擎桥）→ 现用 `img_redqueen.png`（待换：幽蓝等离子反应堆+红闪警报，目标 `yiy_bg_reactor.png`）
   [敌人立绘]（复用，均可"小改"）
   - 抱脸虫 / 破胸体 / 巢穴抱脸虫群 → 复用 `img_licker.png`（小改：偏黄褐/浅黄、缩放 0.6、去长舌；目标 `enemy_yiy_facehugger.png`）
   - 异形工兵（worker1/worker2/pack）→ 复用 `img_hunter.png`（待换：直立异形士兵；目标 `enemy_yiy_worker.png`）
   - 异形哨兵（sentinel1/sentinel2）→ 复用 `img_hunter.png`（待换：士兵+粗尾枪+肩甲；目标 `enemy_yiy_sentinel.png`）
   - 异形猎手（hunter）→ 复用 `img_hunter.png`（待换：瘦高无眼骷髅+超长四肢；目标 `enemy_yiy_hunter.png`）
   - 异形皇后（BOSS）→ 复用 `img_licker.png`（大号/放大；待换：巨冠皇后；目标 `enemy_yiy_queen.png`）
   【备注】§9 美术需求清单完整版见设计文档；阶段一先"复用小改"跑通全流程。
   ===================================================================== */

/* =====================================================================
   L1 船员生活区（教学层）· §3.1
   区域：A1 顶舱管道层 / A2 宿舍区 / A3 医疗区 / A4 餐厅 / A5 主走廊 / A6 机修区 / A7 登陆坞 / A8 气闸区
   ===================================================================== */
pub static YIYING_F1_MAP: &[&str] = &[
"########################################",
"#.....G...II..............II...........#",
"#......................................#",
"#..##.........I......I.......I...##....#",
"#..##....I........I...........I....##..#",
"#..##.........I.......I.......I...##...#",
"#..##.....I.......I.......I......I..##.#",
"#.......................X..............#",
"#....##############################....#",
"#..I....I......I......I......I.........#",
"#........I...I...I....I....I...........#",
"#......................................#",
"#.......E..............................#",
"#.......................E..............#",
"#..............................X.......#",
"#......................................#",
"#..II............II..............II....#",
"#...................G.P................#",
"#....II..............II................#",
"#......................................#",
"#..II..........II...............II.....#",
"#......................................#",
"#......................................#",
"#......................................#",
"#......................................#",
"########################################",
];

/* =====================================================================
   L2 实验室与孵化室（核心段）· §3.2
   区域：B1 到达厅 / B2 主控室（Father）/ B3 生物实验室 / B4 卵库/孵化室入口 / B5 孵化室（卵区）/ B6 管道出口
   ===================================================================== */
pub static YIYING_F2_MAP: &[&str] = &[
"########################################",
"#......................................#",
"#...G.........II..............II.......#",
"#....II...........II.......#######.....#",
"#..II...........II.......###..I..##....#",
"#....II.......I......###......I...##...#",
"#..II......I........###......I.....##..#",
"#....II....I.......#######.......I.....#",
"#..........P....#######................#",
"#.....................H................#",
"#......................................#",
"#...II.........II............II........#",
"#.....II.......L..........L......II....#",
"#......................................#",
"#...............................E......#",
"#.....II........L...........L.....II...#",
"#.............................L........#",
"#......................................#",
"#..II..............II..............II..#",
"#......................................#",
"#......................................#",
"#....II..........II..........II........#",
"#.....G.............G.............G....#",
"#......................................#",
"#...II....II............II....II.......#",
"########################################",
];

/* =====================================================================
   L3 引擎区与皇后巢穴（终局）· §3.3
   区域：C1 电梯到达厅 / C2 引擎控制桥 / C3 反应堆管道区 / C4 皇后巢穴 / C5 贝蒂号对接舱 / C6 管道出口
   ===================================================================== */
pub static YIYING_F3_MAP: &[&str] = &[
"########################################",
"#......................................#",
"#........II...........II.....#####.....#",
"#..................G......#####..I.....#",
"#.............................I........#",
"#....II...........II.....#####.........#",
"#..II.............II...........#######.#",
"#...........II...............##....##..#",
"#......................................#",
"#......................................#",
"#...#############################......#",
"#......................................#",
"#....II...........II.........II........#",
"#......................................#",
"#...........E..........................#",
"#.......II...........II........II......#",
"#.......B..............................#",
"#...II...........II.........II.........#",
"#......................................#",
"#......................................#",
"#......II..........II.........II.......#",
"#......................................#",
"#...............................G......#",
"#......................................#",
"#....II..........II..........II........#",
"########################################",
];

pub static YIYING_FLOOR_NAMES: &[&str] = &[
    "L1 船员生活区",
    "L2 实验室 · 孵化室",
    "L3 引擎区 · 皇后巢穴",
];

/// 调查点：全部 id `yiy_` 前缀，route 引用 scenes_yiying.rs 的 YIYING_SCENES 场景 id。
pub static POINTS: &[maps::PointDef] = &[
    // ---- L1 船员生活区 ----
    maps::PointDef { id: "yiy_p_corpse", name: "餐厅 · 第一具破尸", floor: 0, x: 18, y: 10, route: "yiy_s2_corpse" },
    maps::PointDef { id: "yiy_p_airlock", name: "气闸机关", floor: 0, x: 33, y: 14, route: "yiy_s_airlock" },
    // ---- L2 实验室与孵化室 ----
    maps::PointDef { id: "yiy_p_father", name: "Father 主控终端", floor: 1, x: 30, y: 5, route: "yiy_s3_father" },
    maps::PointDef { id: "yiy_p_medlab", name: "生物实验室 · 医疗舱（寄生手术点）", floor: 1, x: 14, y: 7, route: "yiy_s_medlab" },
    maps::PointDef { id: "yiy_p_sample", name: "取样台", floor: 1, x: 12, y: 8, route: "yiy_s_sample" },
    maps::PointDef { id: "yiy_p_lab_security", name: "安全柜（安保脉冲枪）", floor: 1, x: 4, y: 3, route: "yiy_s_lab_security" },
    maps::PointDef { id: "yiy_p_lab_chest", name: "生物实验室 · 物资箱", floor: 1, x: 6, y: 5, route: "yiy_s_lab_chest" },
    maps::PointDef { id: "yiy_p_nest_burn", name: "卵区 · 烧巢点", floor: 1, x: 30, y: 16, route: "yiy_s_nest_fire" },
    // ---- L3 引擎区与皇后巢穴 ----
    maps::PointDef { id: "yiy_p_reactor", name: "引擎桥 · 引爆总闸", floor: 2, x: 30, y: 4, route: "yiy_s7_evac" },
    maps::PointDef { id: "yiy_p_pipe", name: "冷却管道支路（终结技触发点）", floor: 2, x: 12, y: 8, route: "yiy_s_pipe" },
];

/// 敌人（异形谱系：▸抱脸虫→破胸体→工兵→哨兵→猎手→皇后）。fight 引用 yiying_figths() 表里的 id。
pub static ENEMIES: &[maps::EnemyDef] = &[
    // ---- L1 船员生活区 ----
    maps::EnemyDef { id: "yiy_e_worker1", name: "异形工兵 · 初现", floor: 0, x: 8, y: 13, radius: 4, fight: "f_yiy_worker1" },
    maps::EnemyDef { id: "yiy_e_worker1b", name: "异形工兵 · 巡逻", floor: 0, x: 24, y: 13, radius: 4, fight: "f_yiy_worker1" },
    // ---- L2 实验室与孵化室 ----
    maps::EnemyDef { id: "yiy_e_worker2", name: "异形工兵 · 巡逻（主控室门）", floor: 1, x: 26, y: 6, radius: 4, fight: "f_yiy_worker2" },
    maps::EnemyDef { id: "yiy_e_workerpack", name: "工兵伏击群", floor: 1, x: 22, y: 12, radius: 4, fight: "f_yiy_workerpack" },
    maps::EnemyDef { id: "yiy_e_sentinel1", name: "异形哨兵 · 镇守孵化室", floor: 1, x: 32, y: 14, radius: 4, fight: "f_yiy_sentinel1" },
    // ---- L3 引擎区与皇后巢穴 ----
    maps::EnemyDef { id: "yiy_e_sentinel2", name: "异形哨兵 · 驻守引擎桥", floor: 2, x: 26, y: 6, radius: 4, fight: "f_yiy_sentinel2" },
    maps::EnemyDef { id: "yiy_e_hunter", name: "异形猎手（精英）", floor: 2, x: 12, y: 15, radius: 4, fight: "f_yiy_hunter" },
    maps::EnemyDef { id: "yiy_e_queenhold", name: "巢穴抱脸虫群", floor: 2, x: 8, y: 20, radius: 4, fight: "f_yiy_queenhold" },
];

/// NPC：Father 终端 + 可存活队友（考尔/约翰纳/普维斯/克里斯蒂）。
pub static NPCS: &[maps::NpcDef] = &[
    maps::NpcDef { id: "yiy_n_father", name: "Father（船载 AI）", floor: 1, x: 30, y: 6, talk: "yiy_s_father_npc" },
    maps::NpcDef { id: "yiy_n_call", name: "考尔（队友）", floor: 0, x: 24, y: 17, talk: "yiy_s_call" },
    maps::NpcDef { id: "yiy_n_johnner", name: "约翰纳（队友）", floor: 0, x: 19, y: 19, talk: "yiy_s_johnner" },
];

/// 特殊区域：皇后战区 + 气闸环境区 + 管道过热区（kind=env 表示环境终结技触发区）。
pub static ZONES: &[maps::ZoneDef] = &[
    maps::ZoneDef { id: "yiy_z_queen", name: "皇后巢穴 · 决战", floor: 2, x: 8, y: 16, kind: "fight", ref_id: "f_yiy_queen" },
    maps::ZoneDef { id: "yiy_z_airlock", name: "气闸环境区（鼓舞击杀）", floor: 0, x: 33, y: 14, kind: "env_kill", ref_id: "yiy_s_airlock" },
    maps::ZoneDef { id: "yiy_z_pipe", name: "反应堆管道区（过热熔毁）", floor: 2, x: 12, y: 8, kind: "env", ref_id: "yiy_s_pipe" },
];

/// 传送门（§3，物理单向：`PortalDef` 仅在起点侧定义，反向无门即单向）。
pub static PORTALS: &[maps::PortalDef] = &[
    // L1 顶舱通风管 (6,2) → L2 后舱 (34,22)（单向捷径，需 g_yiy_vents_lock 解锁）
    maps::PortalDef { id: "yiy_pt_vents1", floor: 0, x: 6, y: 2, to_floor: 1, tx: 34, ty: 22 },
    // L1 电梯井 (20,17) ↔ L2 到达厅 (20,4)（双向：成对两门）
    maps::PortalDef { id: "yiy_pt_elv1_up", floor: 0, x: 20, y: 17, to_floor: 1, tx: 20, ty: 4 },
    maps::PortalDef { id: "yiy_pt_elv1_down", floor: 1, x: 20, y: 4, to_floor: 0, tx: 20, ty: 17 },
    // L2 管道出口 (6,22) → L3 引擎区上层 (34,4)（单向，暴露在猎手领地的高危捷径）
    maps::PortalDef { id: "yiy_pt_vents2", floor: 1, x: 6, y: 22, to_floor: 2, tx: 34, ty: 4 },
    // L2 货运电梯 (20,22) ↔ L3 电梯到达厅 (20,3)（双向）
    maps::PortalDef { id: "yiy_pt_elv2_up", floor: 1, x: 20, y: 22, to_floor: 2, tx: 20, ty: 3 },
    maps::PortalDef { id: "yiy_pt_elv2_down", floor: 2, x: 20, y: 3, to_floor: 1, tx: 20, ty: 22 },
];

/// 门禁（§3）：Father 三处判定门禁（均判 `yiy_father_off`，统一在 GateDef 层判 flag，避免散落硬编码）。
pub static GATES: &[maps::GateDef] = &[
    // L1 医疗区门：需医疗钥匙卡（软化门；钥匙卡来自餐厅破尸调查「搜身」）
    maps::GateDef {
        id: "g_yiy_med", name: "医疗区门禁", floor: 0, x: 24, y: 5,
        need_item: Some("yiy_key_med"), need_flag: None,
        lock_msg: "医疗区的气密门紧锁着，指示灯一片死红。读卡器提示：<em>【医疗钥匙卡】</em>——去餐厅那具破尸身上搜一搜。",
        unlock_msg: "钥匙卡「滴」过读卡器，医疗区气密门滑开。药械与冷柜的白光涌出来。",
    },
    // L1 顶部通风管口：需 Father 关停才可进入
    maps::GateDef {
        id: "g_yiy_vents_lock", name: "通风管格栅（Father 锁定）", floor: 0, x: 6, y: 2,
        need_item: None, need_flag: Some("yiy_father_off"),
        lock_msg: "通风管的格栅被 Father 的锁定机构焊死——除非关闭船载 AI，否则没有任何工具能撬开它。",
        unlock_msg: "Father 的锁定机构「嗡——」地失效，通风管格栅咔哒弹开，露出一条通往实验室后舱的黑暗滑道。",
    },
    // L2 孵化室门禁：需 Father 关停
    maps::GateDef {
        id: "g_yiy_incubator", name: "孵化室门禁", floor: 1, x: 22, y: 10,
        need_item: None, need_flag: Some("yiy_father_off"),
        lock_msg: "孵化室的双层气密门仍受 Father 控制，指示灯齐红——必须关停 AI 才能手动开启。",
        unlock_msg: "Father 失联后，孵化室门锁的机构失压，气密门嘎吱地滑开，卵区那股黏稠的腐甜味扑面而来。",
    },
    // L2 主控室门禁：需 Father 关停 或 携带安保脉冲枪（强开）
    maps::GateDef {
        id: "g_yiy_lab", name: "主控室门禁（Father 核心）", floor: 1, x: 25, y: 5,
        need_item: None, need_flag: None, // need_flag 由主线合并阶段按 (yiy_father_off or yiy_pulse) 判；此处留 None 待 Dyn 接线（见 ★外部依赖）
        lock_msg: "主控室的装甲闸门重逾千斤，读卡器注明 Father 专属权限。手动闸阀被焊死——要么先关停 Father，要么用大功率电磁脉冲枪强启。",
        unlock_msg: "主控室闸门在液压声中向两侧退开，露出幽蓝的屏幕墙。Father 的冷冽女声从深处响起。",
    },
    // L3 引擎桥门禁：需 Father 关停
    maps::GateDef {
        id: "g_yiy_reactor", name: "引擎桥门禁", floor: 2, x: 25, y: 5,
        need_item: None, need_flag: Some("yiy_father_off"),
        lock_msg: "引擎桥的防爆门锁死，紧急制动阀仍由 Father 的冗余协议把持——先关停 AI，这道门才会让路。",
        unlock_msg: "防爆门在泄压声中轰然开启，反物质反应堆的幽蓝辉光扑面而来——终局在即。",
    },
    // L3 皇后巢穴门禁：需 Father 关停 或 携带安保脉冲枪
    maps::GateDef {
        id: "g_yiy_queen", name: "皇后巢穴闸门", floor: 2, x: 22, y: 15,
        need_item: None, need_flag: None, // need_flag 由主线合并阶段按 (yiy_father_off or yiy_pulse) 判；此处留 None 待 Dyn 接线（见 ★外部依赖）
        lock_msg: "巢穴闸门的血肉与金属交缠结死，门边是一面碎裂的安保面板——除非 Father 已失联，或你有大功率电磁脉冲枪强制轰开。",
        unlock_msg: "闸门在刺耳的金属撕裂声中轰然倒下，卵房的黏腻黑暗向外涌。深处，那个巨大的心跳声愈发清晰。",
    },
];