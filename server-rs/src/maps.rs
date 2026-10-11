//! 开放世界：蜂巢 2D 俯视地图（多楼层，符合电影《生化危机》+《无限恐怖》原著）
//! 楼层: 0=F1入口层(站台/消毒/齿轮电梯) 1=F2实验层(B区/实验室) 2=F3核心层(红后/激光) 3=F4底层(水道/站台/列车)
//! 地形 ASCII: '#'=墙 '.'=地板 'P'=玩家出生 'I'=设备装饰
//! 对象（调查点/敌人/NPC/副本入口/传送门）独立定义，带坐标与 floor。
//! 注: 本文件由 tools/map_gen.py 生成布局，人工修改布局时请重跑生成器或保持 40 宽等宽。

pub const MAP_W: usize = 40;
pub const MAP_H: usize = 26;
pub const FLOORS: usize = 4;

pub static FLOOR_NAMES: [&str; 4] = ["F1 入口层 · 列车站台", "F2 实验层 · B区", "F3 核心层 · 红后机房", "F4 底层 · 水道站台"];

pub static F1_MAP: &[&str] = &[
    "########################################",
    "#P.....................................#",
    "#......................................#",
    "#..............#...#.....I.....I.......#",
    "#.......I...I..##.##...................#",
    "#..............#...#...................#",
    "#.......I...I..#...#...................#",
    "#..............#...#...................#",
    "#..............##.##...................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "####.############.#########.############",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#........................I.......I.....#",
    "#......................................#",
    "#########.####################.#########",
    "#......................................#",
    "#......................................#",
    "####################.###################",
];

pub static F2_MAP: &[&str] = &[
    "########################################",
    "#............#............#............#",
    "#..##.###....#.....##.###.#.........####",
    "#.#.....#....#.....#....#.#.........#..#",
    "#.#.....#..........#....#...........#..#",
    "#.#.....#....#.....#....#.#............#",
    "#.#.....#....#.....#....#.#.........#..#",
    "#.###.###....#.....##.###.#.........#..#",
    "#............#............#.........#..#",
    "###.########.###########.#########.#####",
    "#............#............#............#",
    "#............#............#...I........#",
    "#............#.##.##......#.....##.###.#",
    "#....I...I.....#...#............#....#.#",
    "#............#.#...#......#.....#....#.#",
    "#............#.#...#......#.....#....#.#",
    "#............#.##.##......#.....##.###.#",
    "#............#............#............#",
    "######.##############.##########.#######",
    "#............#............#............#",
    "#............#........I...#.I..........#",
    "#.........................#............#",
    "#............#.........................#",
    "#.........I..#............#............#",
    "#............#............#............#",
    "########################################",
];

pub static F3_MAP: &[&str] = &[
    "########################################",
    "#......................................#",
    "#.#####.######...................#######",
    "#.#..........#...................#....##",
    "#.#..........#...................#....##",
    "#.#...............######.#####...#....##",
    "#.#..........#....#..........#...#....##",
    "#.#..........#....#.I......I.#...###.###",
    "#.#####.######..I.#..........#.........#",
    "#.................#..........#.........#",
    "#.................#.....I....#.........#",
    "####.###...............................#",
    "##.....#..I.......#..........#.........#",
    "##.....#..........#..........#..I......#",
    "##.....#..........#.........I#.........#",
    "##.....#..........#..........#.........#",
    "##.....#..........#..........#.........#",
    "####.###..........######.#####.........#",
    "#......................................#",
    "#......................................#",
    "#.................................I....#",
    "#.............######.#########.#####...#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "########################################",
];

pub static F4_MAP: &[&str] = &[
    "########################################",
    "#......................................#",
    "#......................................#",
    "#...##.##.........................####.#",
    "#...#...#.........................#..#.#",
    "#...#...#...I.....I...............#..#.#",
    "#...#...#.........................#..#.#",
    "#...##.##............................#.#",
    "#.................................#..#.#",
    "#.................................#..#.#",
    "###.##########.############.#######.####",
    "#.................................#..#.#",
    "#.................................#....#",
    "#........................I........#..#.#",
    "#.................................#..#.#",
    "#.................................#..#.#",
    "#.................................#..#.#",
    "#.................................#..#.#",
    "#.................................#..#.#",
    "#..................................###.#",
    "######.############.##########.#########",
    "#......................................#",
    "#..........I............I..............#",
    "#...........................####.####..#",
    "#......................................#",
    "########################################",
];

pub struct PointDef { pub id: &'static str, pub name: &'static str, pub floor: usize, pub x: usize, pub y: usize, pub route: &'static str }
pub static POINTS: &[PointDef] = &[
    PointDef { id: "p_train_console", name: "列车控制台", floor: 0, x: 20, y: 17, route: "d_train_console" },
    PointDef { id: "p_luggage", name: "行李架", floor: 0, x: 3, y: 4, route: "d_luggage" },
    PointDef { id: "p_platform_map", name: "站台导览图", floor: 0, x: 21, y: 10, route: "d_platform_map" },
    PointDef { id: "p_decon_terminal", name: "消毒终端", floor: 0, x: 29, y: 10, route: "d_decon" },
    PointDef { id: "p_gate_lock", name: "大门密码锁", floor: 0, x: 33, y: 10, route: "d_entrance_gate" },
    PointDef { id: "p_kitchen_cabinet", name: "厨房急救箱", floor: 1, x: 6, y: 10, route: "d_adrenaline" },
    PointDef { id: "p_redqueen_terminal", name: "红后终端", floor: 1, x: 27, y: 8, route: "d_redqueen" },
    PointDef { id: "p_laser_schematic", name: "激光通道示意图", floor: 1, x: 21, y: 16, route: "d_schematic" },
    PointDef { id: "p_file_cabinet", name: "档案柜", floor: 1, x: 6, y: 21, route: "d_files" },
    PointDef { id: "p_med_cabinet", name: "药品柜", floor: 1, x: 9, y: 23, route: "d_meds" },
    PointDef { id: "p_sterile_lab", name: "无菌实验室", floor: 1, x: 12, y: 19, route: "s_b_sterile_lab" },
    PointDef { id: "p_kitchen", name: "厨房", floor: 1, x: 23, y: 17, route: "s_b_kitchen_after" },
    PointDef { id: "p_virus_vault", name: "病毒样本库", floor: 1, x: 33, y: 13, route: "s_virus_vault" },
    PointDef { id: "p_isolation", name: "隔离观察室", floor: 1, x: 37, y: 8, route: "s_isolation_room" },
    PointDef { id: "p_rq3_terminal", name: "红后终端(核心)", floor: 2, x: 24, y: 9, route: "d_redqueen" },
    PointDef { id: "p_server_array", name: "服务器阵列", floor: 2, x: 30, y: 5, route: "d_server" },
    PointDef { id: "p_main_console", name: "主控终端", floor: 2, x: 33, y: 12, route: "d_main_console" },
    PointDef { id: "p_safety_manual", name: "安全手册", floor: 2, x: 36, y: 8, route: "d_manual" },
    PointDef { id: "p_cooling_valve", name: "冷却回路阀组", floor: 2, x: 12, y: 22, route: "d_cooling_valve" },
    PointDef { id: "p_drain_gate", name: "排水闸", floor: 3, x: 6, y: 6, route: "d_drain_gate" },
    PointDef { id: "p_pipe_valve", name: "管道阀门", floor: 3, x: 29, y: 3, route: "d_pipe_valve" },
    PointDef { id: "p_firstaid", name: "站台急救点", floor: 3, x: 22, y: 21, route: "d_firstaid" },
    PointDef { id: "p_train_door", name: "列车车门开关", floor: 3, x: 33, y: 14, route: "d_train_door" },
    PointDef { id: "p_backup_power", name: "备用电源箱", floor: 3, x: 24, y: 24, route: "d_backup_power" },
];

pub struct EnemyDef { pub id: &'static str, pub name: &'static str, pub floor: usize, pub x: usize, pub y: usize, pub radius: usize, pub fight: &'static str }
pub static ENEMIES: &[EnemyDef] = &[
    EnemyDef { id: "e_f1_z1", name: "站台丧尸", floor: 0, x: 7, y: 6, radius: 3, fight: "zombie1_save" },
    EnemyDef { id: "e_f1_z2", name: "列车员丧尸", floor: 0, x: 14, y: 14, radius: 3, fight: "zombie1_far" },
    EnemyDef { id: "e_f1_z3", name: "站台巡逻丧尸", floor: 0, x: 32, y: 23, radius: 3, fight: "zombie1_far" },
    EnemyDef { id: "e_z1", name: "游荡丧尸", floor: 1, x: 25, y: 2, radius: 3, fight: "zombie1_save" },
    EnemyDef { id: "e_z2", name: "游荡丧尸", floor: 1, x: 30, y: 2, radius: 3, fight: "zombie1_save" },
    EnemyDef { id: "e_z3", name: "厨房丧尸", floor: 1, x: 14, y: 15, radius: 3, fight: "zombie1_far" },
    EnemyDef { id: "e_h1", name: "水道尸群", floor: 1, x: 25, y: 24, radius: 3, fight: "horde" },
    EnemyDef { id: "e_licker", name: "舔食者", floor: 1, x: 35, y: 22, radius: 3, fight: "licker" },
    EnemyDef { id: "e_f3_z1", name: "回廊感染者", floor: 2, x: 12, y: 18, radius: 3, fight: "zombie1_save" },
    EnemyDef { id: "e_f3_z2", name: "机房守卫", floor: 2, x: 28, y: 23, radius: 3, fight: "b_guard" },
    EnemyDef { id: "e_f3_z3", name: "核心层巡逻丧尸", floor: 2, x: 36, y: 21, radius: 3, fight: "zombie1_far" },
    EnemyDef { id: "e_f4_horde", name: "水道尸群", floor: 3, x: 4, y: 8, radius: 3, fight: "horde" },
    EnemyDef { id: "e_f4_z1", name: "管道丧尸", floor: 3, x: 13, y: 18, radius: 3, fight: "zombie1_save" },
    EnemyDef { id: "e_f4_z2", name: "底层潜伏丧尸", floor: 3, x: 15, y: 12, radius: 3, fight: "zombie1_far" },
    EnemyDef { id: "e_f4_boss", name: "舔食者·成年", floor: 3, x: 22, y: 23, radius: 3, fight: "licker" },
    EnemyDef { id: "e_f4_elite", name: "猎杀者·实验体", floor: 3, x: 25, y: 14, radius: 3, fight: "hunter_elite" },
];

pub struct NpcDef { pub id: &'static str, pub name: &'static str, pub floor: usize, pub x: usize, pub y: usize, pub talk: &'static str }
pub static NPCS: &[NpcDef] = &[
    NpcDef { id: "n_zhangjie", name: "张杰", floor: 0, x: 8, y: 3, talk: "s_world_zhangjie" },
    NpcDef { id: "n_rain", name: "蕾恩", floor: 1, x: 22, y: 16, talk: "s_world_rain" },
    NpcDef { id: "n_kaplan", name: "卡普兰", floor: 1, x: 25, y: 13, talk: "s_world_kaplan" },
    NpcDef { id: "n_yihao", name: "一号", floor: 1, x: 24, y: 11, talk: "s_world_yihao" },
    NpcDef { id: "n_rain_f3", name: "蕾恩(核心层)", floor: 2, x: 32, y: 4, talk: "s_world_rain" },
    NpcDef { id: "n_rain_f4", name: "蕾恩(站台)", floor: 3, x: 30, y: 12, talk: "s_world_rain" },
];

pub struct ZoneDef { pub id: &'static str, pub name: &'static str, pub floor: usize, pub x: usize, pub y: usize, pub kind: &'static str, pub ref_id: &'static str }
pub static ZONES: &[ZoneDef] = &[
    ZoneDef { id: "z_laser", name: "激光通道", floor: 1, x: 34, y: 21, kind: "puzzle", ref_id: "d_laser_room" },
    ZoneDef { id: "z_licker", name: "站台BOSS区", floor: 1, x: 34, y: 22, kind: "fight", ref_id: "licker" },
];

/// 传送门：物理单向（删除反向门即单向）。箱庭闭环：
/// F3→F4 走竖井（下得快），F4→F3 只能走 B 区爬梯（绕行捷径，单向向上），形成单向环。
pub struct PortalDef { pub id: &'static str, pub floor: usize, pub x: usize, pub y: usize, pub to_floor: usize, pub tx: usize, pub ty: usize }
pub static PORTALS: &[PortalDef] = &[
    PortalDef { id: "pt_elevator_down", floor: 0, x: 27, y: 4, to_floor: 1, tx: 2, ty: 2 },
    PortalDef { id: "pt_stairs_down", floor: 0, x: 3, y: 20, to_floor: 1, tx: 23, ty: 13 },
    PortalDef { id: "pt_elevator_up", floor: 1, x: 2, y: 2, to_floor: 0, tx: 27, ty: 4 },
    PortalDef { id: "pt_vlift_up", floor: 1, x: 23, y: 14, to_floor: 2, tx: 30, ty: 3 },
    PortalDef { id: "pt_vlift_down", floor: 2, x: 30, y: 3, to_floor: 1, tx: 23, ty: 14 },
    // 竖井下行 F3→F4（单向：F4 侧无反向门）
    PortalDef { id: "pt_shaft_down", floor: 2, x: 21, y: 14, to_floor: 3, tx: 21, ty: 5 },
    // 检修爬梯：F4→F3 单向上行（绕路捷径）
    PortalDef { id: "pt_ladder_up", floor: 3, x: 32, y: 19, to_floor: 2, tx: 32, ty: 3 },
    // 通风管：F1 下层站台侧 → F3 下层员工区（单向下行）。入口即 gate_vent 门禁格，解锁后踩上即传送。
    PortalDef { id: "pt_vent_down", floor: 0, x: 20, y: 21, to_floor: 2, tx: 14, ty: 23 },
];

/// 门禁（箱庭软锁）：锁捷径、绕行可达。解锁条件：物品（inventory）或环境态（flag）。
/// 状态存 st.map_objs[gate_id]。
pub struct GateDef {
    pub id: &'static str,
    pub name: &'static str,
    pub floor: usize,
    pub x: usize,
    pub y: usize,
    pub need_item: Option<&'static str>,
    pub need_flag: Option<&'static str>,
    pub lock_msg: &'static str,
    pub unlock_msg: &'static str,
}
pub static GATES: &[GateDef] = &[
    // G1 B区门禁：F2 北走廊直下南区(舔食者区)的捷径，需要「实验室员工卡」
    GateDef {
        id: "gate_b_area", name: "B区门禁", floor: 1, x: 32, y: 19,
        need_item: Some("lab_badge"), need_flag: None,
        lock_msg: "门禁的红灯冷冷地亮着。读卡器旁的铭牌写着：<em>【B区通行证 · 需实验室员工卡】</em>——没有它，你只能绕路。",
        unlock_msg: "「滴——」绿灯亮起，气密门滑开。B区捷径解锁。",
    },
    // G3 水闸铁门：F4 北水道路通向南区站台的近道，需要先排干水道（drain_done）
    GateDef {
        id: "gate_water_sluice", name: "水闸铁门", floor: 3, x: 32, y: 21,
        need_item: None, need_flag: Some("drain_done"),
        lock_msg: "厚重的防水闸门死死压着。透过观察窗，另一侧的积水泛着油光——<em>水位未排空前，这道门无法开启</em>。你得先去找到排水闸。",
        unlock_msg: "水位已经退去，闸门电机发出沉闷的轰鸣，缓缓升起。水道士的捷径通了。",
    },
    // G2 通风管格栅：F1 下层站台侧 → F3 下层的员工捷径（单向下行），需「实验室员工卡」
    GateDef {
        id: "gate_vent", name: "通风管格栅", floor: 0, x: 20, y: 21,
        need_item: Some("lab_badge"), need_flag: None,
        lock_msg: "通风井的格栅被螺栓焊死。铭牌写着：<em>【员工捷径通道 · 需实验室员工卡】</em>——拿到权限卡才能撬开格栅。",
        unlock_msg: "员工卡刷过读卡器「滴——」，格栅嗡地弹开：一股带着消毒水味的气流从井底涌了上来。",
    },
    // G4 B-09 供电闸门：F3 上层←→下层之间的窄口捷径，需先推上备用电源（backup_on）
    GateDef {
        id: "gate_b09", name: "B-09 供电闸门", floor: 2, x: 20, y: 21,
        need_item: None, need_flag: Some("backup_on"),
        lock_msg: "B-09 闸门纹丝不动，指示灯一片死寂。<em>供电中断，闸门锁死</em>——配电图上的编号 B-09 你还记得：去底层配电房推上备用电源箱。",
        unlock_msg: "备用电源的电流沿着线路扑来，B-09 闸门亮起绿灯，液压杆缓缓升起。F3 上下层的窄口捷径通了。",
    },
];

/// 查询某格的未解锁门禁（未解锁才挡路；已解锁等同普通地板）
pub fn gate_at(floor: usize, x: usize, y: usize) -> Option<&'static GateDef> {
    GATES.iter().find(|g| g.floor == floor && g.x == x && g.y == y)
}

/// 按 id 查门禁
pub fn gate_by_id(id: &str) -> Option<&'static GateDef> {
    GATES.iter().find(|g| g.id == id)
}
/// 查询 tile（按楼层）
pub fn tile(floor: usize, x: usize, y: usize) -> char {
    let map = match floor {
        0 => F1_MAP,
        1 => F2_MAP,
        2 => F3_MAP,
        _ => F4_MAP,
    };
    if y >= map.len() || x >= map[y].len() {
        return '#';
    }
    map[y].as_bytes()[x] as char
}

/// 该位置是否可行走（按楼层）
pub fn walkable(floor: usize, x: usize, y: usize) -> bool {
    let c = tile(floor, x, y);
    c != '#'
}

/// 出生点（F1 站台）
pub fn spawn() -> (usize, usize) {
    for (y, row) in F1_MAP.iter().enumerate() {
        if let Some(x) = row.find('P') {
            return (x, y);
        }
    }
    (1, 1)
}

// ============================================================================
// ★ 程序化超大地图（worldgen 生成，替代上方手写 26x40 小地图）
// ----------------------------------------------------------------------------
// 生化蜂巢（biohazard_ch1）主世界不再使用手写 maps::F1~F4_MAP（26x40），改用 `worldgen`
// 程序化生成四张 640x640 蜂巢设施风格超大地图。确定性由世界 slug "biohazard_ch1" 派生种子
// 保证（同一副本每次生成一致）。下方手写 `F1~F4_MAP` 仅作主题参考保留。
//
// ★坐标权威：maps::POINTS/ENEMIES/NPCS/ZONES/PORTALS/GATES 坐标为落地权威（仍在原小图范围，
//   即 x∈[0,40)、y∈[0,26)），此处统一把这些坐标格（以及传送落点格）凿为可走，保证对象/落点
//   在程序化大图上可达；首层出生点由 worldgen 置 'P' 并凿可走。
// ============================================================================

/// 程序化生成生化四层大地图并泄漏为 `'static`（供 WorldData.floors 引用）。
/// 子种子 = worldgen::seed_from_slug("biohazard_ch1") 与楼层号混合，保证每层不同但确定性。
pub fn biohazard_generated_floors() -> &'static [&'static [&'static str]] {
    use std::sync::OnceLock;
    use crate::worldgen;
    static FL: OnceLock<&'static [&'static [&'static str]]> = OnceLock::new();
    FL.get_or_init(|| {
        // 与 worlds/mod.rs::WORLD_BIOHAZARD == "biohazard_ch1" 保持一致（避免跨模块耦合）。
        let base = worldgen::seed_from_slug("biohazard_ch1");

        let mut floors: Vec<&'static [&'static str]> = Vec::new();
        for fi in 0..FLOORS as usize {
            let seed = base.wrapping_mul(6364136223846793005).wrapping_add(fi as u64 + 23);
            let cfg = worldgen::GenConfig::hub640(seed);
            let mut res = worldgen::generate_floor(&cfg);
            let tiles = &mut res.tiles;

            // ★ 撤销 worldgen 的随机出生点 'P'（出生点权威归下方盖章的小图内容区）。
            //   否则大图深处会残留一枚 'P'，且玩家会出生在远离对象簇数百格的荒墙里。
            if res.spawn.1 < tiles.len() && res.spawn.0 < tiles[res.spawn.1].len()
                && tiles[res.spawn.1].as_bytes()[res.spawn.0] == b'P'
            {
                let bytes = unsafe { tiles[res.spawn.1].as_bytes_mut() };
                bytes[res.spawn.0] = b'.';
            }

            // ★ 把原始手写小地图（40×26）「盖章」进大图左上角，作为内容区：对象坐标
            //   （x∈[0,40), y∈[0,26)）正是基于该小图设计，其内部本就连通且含出生点 'P'（仅 F1）。
            //   盖章后对象/门禁/传送门全部落在连通地板上，不再散落成孤立格。
            let small = match fi {
                0 => F1_MAP,
                1 => F2_MAP,
                2 => F3_MAP,
                _ => F4_MAP,
            };
            worldgen::stamp_map(tiles, small, 0, 0);

            // 某格强制可走（若为墙则凿开）——防御性：万一对象压在小图墙格上。
            let mut carve = |x: usize, y: usize| {
                worldgen::carve_walkable(tiles, x, y);
            };
            // 本层对象 / 传送落点坐标权威：全凿可走。
            for p in POINTS.iter().filter(|p| p.floor == fi) {
                carve(p.x, p.y);
            }
            for e in ENEMIES.iter().filter(|e| e.floor == fi) {
                carve(e.x, e.y);
            }
            for n in NPCS.iter().filter(|n| n.floor == fi) {
                carve(n.x, n.y);
            }
            for z in ZONES.iter().filter(|z| z.floor == fi) {
                carve(z.x, z.y);
            }
            for pt in PORTALS.iter().filter(|pt| pt.floor == fi) {
                carve(pt.x, pt.y); // 传送源格
            }
            for g in GATES.iter().filter(|g| g.floor == fi) {
                carve(g.x, g.y); // 门禁格
            }
            // 传送落点一律凿可走（目标层由 to_floor 决定，独立于当前 fi）。
            for pt in PORTALS.iter().filter(|pt| pt.to_floor == fi) {
                carve(pt.tx, pt.ty);
            }

            // ★ 全图连通：自内容区首个开放格（F1 即 'P'）出发，把程序化大图主区桥接进来，
            //   修复「对象被墙隔断、出生点不可达交互物」的阻断性 bug。
            let mut start = (1usize, 1usize);
            'scan: for (y, row) in tiles.iter().enumerate() {
                for (x, b) in row.bytes().enumerate() {
                    if b != b'#' {
                        start = (x, y);
                        break 'scan;
                    }
                }
            }
            worldgen::connect_all(tiles, start);

            // 泄漏每行与整层切片为 'static
            let rows: Vec<&'static str> = res
                .tiles
                .into_iter()
                .map(|s| -> &'static str { Box::leak(s.into_boxed_str()) })
                .collect();
            floors.push(Box::leak(rows.into_boxed_slice()));
        }
        Box::leak(floors.into_boxed_slice())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 自 start 出发做 4 邻域 BFS，返回每个格是否可达（'#' 视为墙）。
    fn bfs_reach(floors: &[&[&str]], floor: usize, start: (usize, usize)) -> Vec<Vec<bool>> {
        let h = floors[floor].len();
        let w = floors[floor][0].len();
        let mut seen = vec![vec![false; w]; h];
        let mut q = std::collections::VecDeque::new();
        if start.1 < h && start.0 < w {
            seen[start.1][start.0] = true;
            q.push_back(start);
        }
        let dirs = [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)];
        while let Some((cx, cy)) = q.pop_front() {
            for (dx, dy) in dirs {
                let (nx, ny) = (cx as i32 + dx, cy as i32 + dy);
                if nx < 0 || ny < 0 { continue; }
                let (nx, ny) = (nx as usize, ny as usize);
                if nx >= w || ny >= h || seen[ny][nx] { continue; }
                if floors[floor][ny].as_bytes()[nx] == b'#' { continue; }
                seen[ny][nx] = true;
                q.push_back((nx, ny));
            }
        }
        seen
    }

    /// 回归护栏：修复「对象被墙隔断、出生点不可达交互物」后，四层所有对象都必须自出生点可达。
    #[test]
    fn biohazard_objects_reachable_from_spawn() {
        let floors = biohazard_generated_floors();
        // 出生点应回到小图 F1 的 (1,1)
        let mut spawn = (1usize, 1usize);
        's: for (y, row) in floors[0].iter().enumerate() {
            if let Some(x) = row.find('P') {
                spawn = (x, y);
                break 's;
            }
        }
        assert_eq!(spawn, (1, 1), "出生点应为内容区 F1 的 (1,1)");

        for fi in 0..FLOORS {
            let seen = bfs_reach(floors, fi, spawn);
            for p in POINTS.iter().filter(|p| p.floor == fi) {
                assert!(seen[p.y][p.x], "点 {} ({},{}) 不可达", p.id, p.x, p.y);
            }
            for e in ENEMIES.iter().filter(|e| e.floor == fi) {
                assert!(seen[e.y][e.x], "敌人 {} ({},{}) 不可达", e.id, e.x, e.y);
            }
            for n in NPCS.iter().filter(|n| n.floor == fi) {
                assert!(seen[n.y][n.x], "NPC {} ({},{}) 不可达", n.id, n.x, n.y);
            }
            for z in ZONES.iter().filter(|z| z.floor == fi) {
                assert!(seen[z.y][z.x], "区域 {} ({},{}) 不可达", z.id, z.x, z.y);
            }
            for pt in PORTALS.iter().filter(|pt| pt.floor == fi) {
                assert!(seen[pt.y][pt.x], "传送门 {} ({},{}) 不可达", pt.id, pt.x, pt.y);
            }
            for g in GATES.iter().filter(|g| g.floor == fi) {
                assert!(seen[g.y][g.x], "门禁 {} ({},{}) 不可达", g.id, g.x, g.y);
            }
        }
    }
}
