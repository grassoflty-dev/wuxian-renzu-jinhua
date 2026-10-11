//! 《末世死城·人类防线》任务世界（sl2 moshi）：
//! 4 层（F1 城墙与外街 / F2 城内医院与军火库 / F3 地下指挥所 / F4 炮台观测台）。
//! 设计依据 design/zhttty_universe/wuxian_weilai/moshi_shoucheng.md §3。
//! 本文件为纯静态世界数据（地图 + 调查点 + NPC + 敌人 + 传送门 + 门禁），剧情/战斗见 scenes_moshi.rs。
//! 引擎网格 40 宽×26 高/层；ASCII 为"示意蓝图"，§3 区域划分表的坐标为落地权威。
//! 出生点 P 在 F1 城墙平台 (5,6)；传送门/门禁坐标全部照抄 §3 传送门接线与门禁表。

use crate::maps;

/// ---- F1 城墙与外街（出生层） ----
/// 每行固定 40 字符（# 边框 + 38 格内景）。开放连通：调查点/装饰以坐标落位，落地以 §3 区域表为准。
/// F1 城墙与外街采用开放连通结构（无纵向全封墙）：出生点 P(5,6) 出生在最上方守城平台带，
/// 内街大部为可走动地板，稀疏 `#` 仅作建筑破片装饰，保证所有调查点/门禁/传送门可步行到达。
/// 关键坐标全照抄 §3B 区域表（T=机枪阵地 / C=下城阶梯化原位 / G=门 / I=装饰）。
pub static MOSHI_F1_MAP: &[&str] = &[
"########################################",
"#......................................#",
"#......................................#",
"#......................................#",
"########################################",
"#.....I................................#",
"#.....P......I.........................#",
"#........I.............................#",
"######.#################################",
"#.I....................................#",
"#.....I................................#",
"#.........I............................#",
"#.............I........................#",
"#.................I....................#",
"#.....................I................#",
"#.........................I............#",
"#.............................I........#",
"#.................................I....#",
"#.....................................I#",
"#......................................#",
"#......................................#",
"#......................................#",
"#......................................#",
"#......................................#",
"#......................................#",
"########################################",
];
// 注：出生带 y5 行 = 第 6 行（索引 5）；区域表键坐标（P(5,6)/机枪(12,6)(28,6)/门(24,6)/传动门(34,23)/配给站(6,22)等）
// 均在开放连通区；P 标记（出生点）严格放 (5,6)。

/// ---- F2 城内医院与军火库 ----
pub static MOSHI_F2_MAP: &[&str] = &[
"########################################",
"##I##########I##########I##########I####",
"##I########I##########I#########I#######",
"##..............................II....##",
"##..#######...........................##",
"##..#######...###.....................##",
"##..#######...###.....................##",
"##..#######...........................##",
"##..............#######...............##",
"##.........................G..........##",
"##....................................##",
"##..............TT....................##",
"##............I.......................##",
"##....................................##",
"##..............................II....##",
"##....................................##",
"##....#######.........................##",
"##...#########........................##",
"##...#########........................##",
"##..G........##########...............##",
"##..C.........##########..............##",
"##...........##########...............##",
"##......C......##########.............##",
"##...................................C##",
"##....................................##",
"########################################",
];

/// ---- F3 地下指挥所 ----
pub static MOSHI_F3_MAP: &[&str] = &[
"########################################",
"##I#######I#########I########I##########",
"##..#########.........................##",
"##..#########..#######................##",
"##....................................##",
"##..............###...................##",
"##....................................##",
"##..##########........................##",
"##..##########....####................##",
"##..##########....####................##",
"##....................................##",
"##....................................##",
"##...........TT.......................##",
"##....................................##",
"##..............GG....................##",
"##..#######...........................##",
"##..#######...........................##",
"##....................................##",
"##......##########....................##",
"##......##########......C.............##",
"##....................................##",
"##....................................##",
"##........................#######.....##",
"##......................G.............##",
"##....................................##",
"########################################",
];

/// ---- F4 炮台观测台（最终层） ----
pub static MOSHI_F4_MAP: &[&str] = &[
"########################################",
"##...............I....................##",
"##....................................##",
"##......#######.......................##",
"##......#######.......................##",
"##....................................##",
"##....................II..............##",
"##....................................##",
"##........................I...........##",
"##....................................##",
"##....................................##",
"##......................TT............##",
"##....................................##",
"##............I.......................##",
"##....................................##",
"##..#######...........................##",
"##..#######...........................##",
"##....................................##",
"##................I...................##",
"##....................................##",
"##........TT..........................##",
"##....................................##",
"##..............C.....................##",
"##....................................##",
"##....................................##",
"########################################",
];

pub static MOSHI_FLOOR_NAMES: &[&str] = &[
    "F1 城墙与外街 · 人类防线的第一面墙",
    "F2 城内医院与军火库 · 弹尽粮绝",
    "F3 地下指挥所 · 轨道授权",
    "F4 炮台观测台 · 最终防线",
];

/// 调查点：全部 id 以 ms_ 前缀，route 引用 scenes_moshi.rs 的 MOSHI_SCENES 场景 id。
pub static POINTS: &[maps::PointDef] = &[
    // ---- F1 城墙与外街 ----
    maps::PointDef { id: "ms_p_siren", name: "防空警报塔", floor: 0, x: 10, y: 13, route: "ms_siren" },
    maps::PointDef { id: "ms_p_bus", name: "废弃巴士（撬棍①）", floor: 0, x: 29, y: 15, route: "ms_bus" },
    maps::PointDef { id: "ms_p_supply", name: "军需配给站", floor: 0, x: 6, y: 22, route: "ms_supply" },
    maps::PointDef { id: "ms_p_gunpost_a", name: "城墙机枪阵地·东", floor: 0, x: 12, y: 6, route: "ms_00" },
    maps::PointDef { id: "ms_p_gunpost_b", name: "城墙机枪阵地·西", floor: 0, x: 28, y: 6, route: "ms_00" },
    // ---- F2 城内医院与军火库 ----
    maps::PointDef { id: "ms_p_ward", name: "病房（office_key ①）", floor: 1, x: 20, y: 6, route: "ms_ward" },
    maps::PointDef { id: "ms_p_linen", name: "医院杂物间（撬棍②）", floor: 1, x: 8, y: 12, route: "ms_linen" },
    maps::PointDef { id: "ms_p_pharmacy", name: "药房（急救包）", floor: 1, x: 11, y: 4, route: "ms_pharmacy" },
    maps::PointDef { id: "ms_p_captain", name: "上尉办公室", floor: 1, x: 26, y: 4, route: "ms_captain" },
    maps::PointDef { id: "ms_p_cell", name: "电梯控制柜", floor: 1, x: 31, y: 16, route: "ms_cell" },
    // ---- F3 地下指挥所 ----
    maps::PointDef { id: "ms_p_comms", name: "通讯阵列", floor: 2, x: 24, y: 7, route: "ms_comms" },
    maps::PointDef { id: "ms_p_reactor", name: "反应堆配电室", floor: 2, x: 14, y: 16, route: "ms_reactor" },
    // ---- F4 炮台观测台 ----
    maps::PointDef { id: "ms_p_ammo_lift", name: "弹药升降井（补弹药）", floor: 3, x: 6, y: 20, route: "ms_f4_arrive" },
    maps::PointDef { id: "ms_p_howitzer", name: "巨型主炮残件", floor: 3, x: 16, y: 16, route: "ms_f4_arrive" },
    maps::PointDef { id: "ms_p_scope", name: "观测镜", floor: 3, x: 17, y: 11, route: "ms_scope" },
    maps::PointDef { id: "ms_p_beacon", name: "轨道信标塔", floor: 3, x: 26, y: 6, route: "ms_beacon" },
];

/// 敌人（兽潮；fight 引 scenes_moshi.rs 的 moshi_figths() 表的 fight id）。
/// 驻守分布照抄 §4：radius 巡逻半径建议 2（兽潮小队 r3）。
pub static ENEMIES: &[maps::EnemyDef] = &[
    // F1 城墙与外街
    maps::EnemyDef { id: "ms_e_beast1", name: "兽兵", floor: 0, x: 8, y: 6, radius: 2, fight: "fight_f1_beast" },
    maps::EnemyDef { id: "ms_e_beast2", name: "兽兵", floor: 0, x: 10, y: 13, radius: 2, fight: "fight_f1_beast" },
    maps::EnemyDef { id: "ms_e_beast3", name: "兽兵", floor: 0, x: 26, y: 11, radius: 2, fight: "fight_f1_beast" },
    maps::EnemyDef { id: "ms_e_leaper1", name: "跳扑兽", floor: 0, x: 16, y: 17, radius: 2, fight: "fight_f1_leaper" },
    maps::EnemyDef { id: "ms_e_leaper2", name: "跳扑兽", floor: 0, x: 7, y: 15, radius: 2, fight: "fight_f1_leaper" },
    maps::EnemyDef { id: "ms_e_pack1", name: "兽潮小队", floor: 0, x: 22, y: 16, radius: 3, fight: "fight_f1_pack" },
    // F2 城内医院与军火库
    maps::EnemyDef { id: "ms_e_mutant1", name: "医疗变异体", floor: 1, x: 10, y: 18, radius: 2, fight: "fight_f2_mutant" },
    maps::EnemyDef { id: "ms_e_mutant2", name: "医疗变异体", floor: 1, x: 22, y: 21, radius: 2, fight: "fight_f2_mutant" },
    maps::EnemyDef { id: "ms_e_beast4", name: "兽兵", floor: 1, x: 12, y: 13, radius: 2, fight: "fight_f1_beast" },
    maps::EnemyDef { id: "ms_e_pack2", name: "兽潮小队", floor: 1, x: 18, y: 15, radius: 3, fight: "fight_f1_pack" },
    maps::EnemyDef { id: "ms_e_stalker", name: "潜行猎兽", floor: 1, x: 31, y: 12, radius: 2, fight: "fight_f2_stalker" },
    // F3 地下指挥所
    maps::EnemyDef { id: "ms_e_burrower1", name: "掘地兽", floor: 2, x: 8, y: 9, radius: 2, fight: "fight_f3_burrower" },
    maps::EnemyDef { id: "ms_e_burrower2", name: "掘地兽", floor: 2, x: 27, y: 15, radius: 2, fight: "fight_f3_burrower" },
    maps::EnemyDef { id: "ms_e_pack3", name: "兽潮·深部", floor: 2, x: 12, y: 20, radius: 2, fight: "fight_f3_pack" },
    maps::EnemyDef { id: "ms_e_pack4", name: "兽潮·深部", floor: 2, x: 22, y: 13, radius: 2, fight: "fight_f3_pack" },
    maps::EnemyDef { id: "ms_e_vanguard1", name: "高阶兽兵", floor: 2, x: 34, y: 10, radius: 2, fight: "fight_f3_vanguard" },
    maps::EnemyDef { id: "ms_e_vanguard2", name: "高阶兽兵", floor: 2, x: 25, y: 21, radius: 2, fight: "fight_f3_vanguard" },
    // F4 炮台观测台
    maps::EnemyDef { id: "ms_e_dog1", name: "兽潮·决死", floor: 3, x: 10, y: 21, radius: 2, fight: "fight_f4_pack" },
    maps::EnemyDef { id: "ms_e_dog2", name: "兽潮·决死", floor: 3, x: 28, y: 17, radius: 2, fight: "fight_f4_pack" },
    maps::EnemyDef { id: "ms_e_dog3", name: "兽潮·决死", floor: 3, x: 12, y: 6, radius: 2, fight: "fight_f4_pack" },
    maps::EnemyDef { id: "ms_e_dog4", name: "兽潮·决死", floor: 3, x: 28, y: 6, radius: 2, fight: "fight_f4_pack" },
];

/// NPC：守军/医师/指挥官引导线索。
pub static NPCS: &[maps::NpcDef] = &[
    maps::NpcDef { id: "ms_n_minuteman", name: "民兵队长", floor: 0, x: 16, y: 6, talk: "ms_00_minuteman" },
    maps::NpcDef { id: "ms_n_medic", name: "军医", floor: 1, x: 12, y: 9, talk: "ms_medic_win" },
    maps::NpcDef { id: "ms_n_comander", name: "老指挥官", floor: 2, x: 20, y: 9, talk: "ms_comander" },
];

/// 特殊区域：多波次战场（kind=fight）+ BOSS 战区（kind=fight）+ 反应堆机关（puzzle）。
pub static ZONES: &[maps::ZoneDef] = &[
    // F1 城门口广场 · 多波兽潮（战斗场景链 起点 ms_combat_a）
    maps::ZoneDef { id: "ms_z_sq1", name: "城门口广场 · 兽潮第一波", floor: 0, x: 22, y: 16, kind: "fight", ref_id: "ms_combat_a" },
    // F2 中央广场 · 兽潮增援
    maps::ZoneDef { id: "ms_z_sq2", name: "中央广场 · 兽潮增援", floor: 1, x: 20, y: 15, kind: "fight", ref_id: "ms_medic_fight" },
    // F3 电梯闸 · 最后的守军
    maps::ZoneDef { id: "ms_z_gate3", name: "电梯闸 · 最后的守军", floor: 2, x: 30, y: 23, kind: "fight", ref_id: "ms_f3_arrive" },
    // F3 反应堆配电室 · 机关
    maps::ZoneDef { id: "ms_z_reactor", name: "反应堆配电室", floor: 2, x: 14, y: 16, kind: "puzzle", ref_id: "ms_reactor" },
    // F4 顶层观测甲板 · BOSS 决战
    maps::ZoneDef { id: "ms_z_siege", name: "狂化攻城巨兽 · 决战", floor: 3, x: 20, y: 9, kind: "fight", ref_id: "ms_f4_boss" },
];

/// 传送门（§3，物理单向：`PortalDef` 仅在起点侧定义，反向无门即单向）。
pub static PORTALS: &[maps::PortalDef] = &[
    // F1 → F2：城墙升降梯(下)，主线下行，单向
    maps::PortalDef { id: "ms_pt_f1_f2", floor: 0, x: 34, y: 23, to_floor: 1, tx: 6, ty: 20 },
    // F2 → F1：回程电梯（需 flag cell_restored 才出现，见 GATES p_f2_back_f1）
    maps::PortalDef { id: "ms_pt_f2_f1", floor: 1, x: 36, y: 14, to_floor: 0, tx: 30, ty: 6 },
    // F2 → F3：机要楼梯(下)，主线下行，单向
    maps::PortalDef { id: "ms_pt_f2_f3", floor: 1, x: 34, y: 22, to_floor: 2, tx: 5, ty: 18 },
    // F3 → F2：回程货梯（需 flag power_restored）
    maps::PortalDef { id: "ms_pt_f3_f2", floor: 2, x: 36, y: 19, to_floor: 1, tx: 36, ty: 14 },
    // F3 → F4：指挥所→炮台专用电梯(上)，受 gate_cc_elevator 控制，主线登顶
    maps::PortalDef { id: "ms_pt_f3_f4", floor: 2, x: 30, y: 23, to_floor: 3, tx: 14, ty: 23 },
];

/// 门禁（§3，箱庭软锁，解锁条件为道具/item 或 flag，状态存 st.map_objs[gate_id]）。
pub static GATES: &[maps::GateDef] = &[
    // gate_city_gate：城墙平台东段↔西段
    maps::GateDef {
        id: "ms_g_city_gate", name: "城门闸", floor: 0, x: 24, y: 6,
        need_item: None, need_flag: Some("city_gate_welded"),
        lock_msg: "城门闸被兽潮顶得哐哐作响——只有焊死它，才能封住城墙东段到西段的直线火线。",
        unlock_msg: "闸门在火里焊死成一整块铁板，城墙东西两段连成一条防线。",
    },
    // gate_rubble：内街东↔西近道（需撬棍）
    maps::GateDef {
        id: "ms_g_rubble", name: "废墟近道", floor: 0, x: 12, y: 20,
        need_item: Some("crowbar"), need_flag: None,
        lock_msg: "坍塌的废墟堵死了这条近道。《撬棍》还在街上某处。",
        unlock_msg: "你撬开几块混凝土，露出一条贯通城墙内侧的近道。",
    },
    // gate_hospital_east：急诊大厅↔广场近道（需 flag medic_trusted）
    maps::GateDef {
        id: "ms_g_hospital_east", name: "医院东门", floor: 1, x: 28, y: 10,
        need_item: None, need_flag: Some("medic_trusted"),
        lock_msg: "医院东门反锁着。门缝里传来低声的走动——需要军医的信任才肯开门。",
        unlock_msg: "军医从内侧打开东门：「救下了我们的护士……谢了。进去吧。」",
    },
    // gate_elevator_south：回程电梯（F2→F1 捷径，需 flag cell_restored）
    maps::GateDef {
        id: "ms_g_elevator_south", name: "电梯闸·南", floor: 1, x: 36, y: 14,
        need_item: None, need_flag: Some("cell_restored"),
        lock_msg: "电梯轿厢的灯灭着，控制柜上的熔丝烧断了。没电，电梯纹丝不动。",
        unlock_msg: "恢复供电后，电梯指示灯亮起，轿厢缓缓升了上来。",
    },
    // gate_armory：军火库安全门（需 item keycard_armory）
    maps::GateDef {
        id: "ms_g_armory", name: "军火库安全门", floor: 1, x: 30, y: 19,
        need_item: Some("keycard_armory"), need_flag: None,
        lock_msg: "厚重的军火库安全门纹丝不动，读卡器亮着红灯。需要一把钥匙卡。",
        unlock_msg: "门锁「咔」地亮起绿灯，军火库缓缓打开，成排的重火力露出冷光。",
    },
    // gate_reactor：反应堆机房（需 item crowbar）
    maps::GateDef {
        id: "ms_g_reactor", name: "反应堆机房门", floor: 2, x: 16, y: 15,
        need_item: Some("crowbar"), need_flag: None,
        lock_msg: "反应堆机房的防辐射门被变形卡死，需要撬棍才能别开。",
        unlock_msg: "你撬开门，防辐射门「嘶」地泄压打开——配电室里冒出一股焦糊味。",
    },
    // gate_freight：F3→F2 回程货梯（需 flag power_restored）
    maps::GateDef {
        id: "ms_g_freight", name: "指挥所货梯", floor: 2, x: 36, y: 19,
        need_item: None, need_flag: Some("power_restored"),
        lock_msg: "货梯的按钮一排全灭——地下指挥所停电了。",
        unlock_msg: "电力恢复，货梯指示灯苏醒，轿厢开始下降。",
    },
    // gate_cc_elevator：上 F4 的主线门禁（需 flag orbital_auth）
    maps::GateDef {
        id: "ms_g_cc_elevator", name: "防爆电梯门", floor: 2, x: 30, y: 23,
        need_item: None, need_flag: Some("orbital_auth"),
        lock_msg: "军事级防爆电梯被多重授权锁死。老指挥官说过：只有轨道授权才能登顶炮台。",
        unlock_msg: "授权码核对通过，防爆电梯门向两侧滑开，露出一路向上的轨道。",
    },
];