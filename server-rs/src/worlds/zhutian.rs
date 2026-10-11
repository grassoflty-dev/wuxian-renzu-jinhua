//! 主神空间世界（P1）：中央主神光柱圆台 + 西侧半圆广场 + 东侧传送门阵列 + 南侧兑换区 + 复活祭坛。
//! 单层 40×26，无战斗（enemies 空表）。跨世界网关见 worlds/mod.rs 的 GW_PORTALS。
//! 设计依据 multi_world_framework.md §6 P1 蓝本 / §5.1。

use crate::maps;

/// 主神空间 1 层地图（40×26 等宽 ASCII；#墙 .地板 P出生点 I装饰）
/// 布局定稿：中央直径 12 格主神光柱圆台（含主神交互点，P 出生在光柱南侧开口）；
/// 西侧半圆广场（张杰 NPC）；东侧传送门阵列（上=生化白门 gw_biohazard，下=咒怨灰绿门 gw_zhouyuan）；
/// 南侧兑换区（光球×3）；西南复活祭坛。
pub static ZHUTIAN_MAP: &[&str] = &[
"########################################",
"#......................................#",
"#......................................#",
"#......................................#",
"#......................................#",
"#......................................#",
"#...........................##...##....#",
"#...........................#.....#....#",
"#....#########......#####...#..I..#....#",
"#..................##...##..#.....#....#",
"#.................##.....##............#",
"#.....I...........#.......#............#",
"#..........I......#...I...#........I...#",
"#.................#.......#............#",
"#.................##.....##............#",
"#..................##...##.............#",
"#.....................P.....##...##....#",
"#...........................#.....#....#",
"#...........................#..I..#....#",
"#.................I..I..I...#.....#....#",
"#....####...I..........................#",
"#......I...............................#",
"#......................................#",
"#......................................#",
"#......................................#",
"########################################",
];

pub static ZHUTIAN_FLOOR_NAMES: &[&str] = &["主神空间 · 中央广场"];

/// 主神空间调查点：中央主神 + 南侧兑换区光球×3 + 西南复活祭坛
pub static POINTS: &[maps::PointDef] = &[
    maps::PointDef { id: "np_nexus_god", name: "主神 · 光柱", floor: 0, x: 22, y: 12, route: "s_nexus_god" },
    maps::PointDef { id: "np_exchange_strengthen", name: "兑换光球 · 强化", floor: 0, x: 18, y: 19, route: "s_nexus_exchange" },
    maps::PointDef { id: "np_exchange_gene", name: "兑换光球 · 基因锁", floor: 0, x: 21, y: 19, route: "s_nexus_exchange" },
    maps::PointDef { id: "np_exchange_bloodline", name: "兑换光球 · 血统", floor: 0, x: 24, y: 19, route: "s_nexus_exchange" },
    maps::PointDef { id: "np_nexus_altar", name: "复活祭坛", floor: 0, x: 7, y: 21, route: "s_nexus_resurrection" },
];

/// 主神空间 NPC：张杰（引导者）+ 中洲队核心队友（郑吒/楚轩/詹岚/赵樱空）
/// 全部 floor=0 主神广场，坐标错开摆位；talk 指向各自对话场景。
pub static NPCS: &[maps::NpcDef] = &[
    maps::NpcDef { id: "n_zhangjie_nexus", name: "张杰", floor: 0, x: 7, y: 11, talk: "s_nexus_zhangjie" },
    maps::NpcDef { id: "n_zhengzha_nexus", name: "郑吒", floor: 0, x: 12, y: 10, talk: "s_nexus_zhengzha" },
    maps::NpcDef { id: "n_chuxuan_nexus", name: "楚轩", floor: 0, x: 16, y: 9, talk: "s_nexus_chuxuan" },
    maps::NpcDef { id: "n_zhanlan_nexus", name: "詹岚", floor: 0, x: 14, y: 13, talk: "s_nexus_zhanlan" },
    maps::NpcDef { id: "n_zhaoyingkong_nexus", name: "赵樱空", floor: 0, x: 8, y: 14, talk: "s_nexus_zhaoyingkong" },
];

/// 主神空间单层无层内切层传送门；跨世界网关（gw_*）在 worlds/mod.rs GW_PORTALS。
pub static PORTALS: &[maps::PortalDef] = &[];

/// 主神空间门禁软锁：咒怨门（P1 暂不解锁，需 P3 接入）
pub static GATES: &[maps::GateDef] = &[
    maps::GateDef {
        id: "gz_zhouyuan", name: "咒怨之门（封印）",
        floor: 0, x: 31, y: 16,
        need_item: None,
        need_flag: Some("zy_unlocked"), // P1 无任何路径置此 flag → 恒锁（"暂不解锁"）
        lock_msg: "灰绿色的封印纹路渗着寒意——通往《咒怨》的门尚未向轮回者开启。",
        unlock_msg: "咒怨之门缓缓开启……（P3 接入）",
    },
];