//! 《无限恐怖 · 命运清单 · 第二端》任务世界（world slug: baisun，前缀 bs_）：3 层。
//! 地图为主世界示范：由 `worldgen` 程序化生成 3 张 640×640 蜂巢设施超大地图（替代原手写 40×26 tiles）。
//! 设计依据 design/zhttty_universe/00_INDEX_EXPANSION.md §1.1「死神来了」系列 + research 第二端骨架。
//! 本文件为纯静态世界数据（地图 + 调查点 + 环境机关 + 同伴 NPC + 环境死亡区 + 传送门 + 门禁），剧情见 scenes_baisun.rs。
//! 主题定位：「规则流机关」。死神无实体，以「连环意外」收命；玩家在场景里
//!   「识破征兆」置 flag（bs_foresee_*）→ 进对应机关免死改命；未识破则死亡档案「意外身故」。
//! 三征兆齐备置 bs_fate_rewritten。仅 L3 保留 1 个「死神·使者」选择驱动象征战（HP150）。
//! 敌人极少/无实体：环境机关以 ZoneDef(kind=env) 表演致命意外（停车场坠物/商场电梯夹/电影院火灾逃生）；
//!   仅 L3 保留 1 个「逃生梯巡查员（异样）」作使者象征战的一次性代理。出生点 P 由 worldgen 在 L1 入口区置位。
//! ★坐标权威：下方 POINTS/ZONES/NPC/GATES/PORTALS 坐标为落地权威，地图格由 worldgen 程序化生成（含这些坐标格，
//!   由 baisun_generated_floors 统一凿为可走，保证对象可达；传送落点同理可走）。

use crate::maps;
use crate::worldgen;

// ============================================================================
// ★ 程序化超大地图（worldgen 生成，替代下方手写小地图）
// ----------------------------------------------------------------------------
// baisun 主世界不再使用手写 40x26 tiles，改用 `worldgen` 程序化生成三张 640x640 蜂巢
// 设施风格超大地图（L1 医院停车场，L2 室内商场合订结构，L3 电影院逃生梯），确定性由
// 世界 slug "baisun" 派生种子保证（同一副本每次生成一致）。下方手写 `BAISUN_L*x_MAP`
// 仅作主题参考保留（不再被 WorldData.floors 引用；pub 无 dead_code 告警）。
//
// 元素填充 TODO（本轮仅做地图生成，敌人/道具/传送门/剧情点按区域落位留待下一轮）：
//   - 入口区 region::Entrance → 出生点 L1 停车场（bs_p_l1_* 出生征兆观测）、引导性互动。
//   - 探索区 region::Explore   → 常规敌人、补给道具、支线剧情点。
//   - 核心区 region::Core      → 精英/前置机关（bs_z_l1_drop / bs_z_l2_elev / bs_z_l3_fire 关联征兆）。
//   - BOSS区 region::Boss      → L3 最远角落「死神·使者」象征战（bs_boss，现有 bs_e_l3_watcher
//                              与 bs_z_l3_emissary 已据此落位）。
// ============================================================================

/// 程序化生成 baisun 三层大地图并泄漏为 `'static`（供 WordData.floors 引用）。
/// 子种子 = worldgen::seed_from_slug("baisun") 与楼层号混合，保证每层不同但确定性。
pub fn baisun_generated_floors() -> &'static [&'static [&'static str]] {
    use std::sync::OnceLock;
    static FL: OnceLock<&'static [&'static [&'static str]]> = OnceLock::new();
    FL.get_or_init(|| {
        let base = worldgen::seed_from_slug(super::WORLD_BAISUN);

        let mut floors: Vec<&'static [&'static str]> = Vec::new();
        for fi in 0..3usize {
            let seed = base.wrapping_mul(6364136223846793005).wrapping_add(fi as u64 + 11);
            let cfg = worldgen::GenConfig::hub640(seed);
            let mut res = worldgen::generate_floor(&cfg);
            let tiles = &mut res.tiles;

            // ★ 撤销 worldgen 的随机出生点 'P'（出生点权威归下方盖章的小图内容区）。
            if res.spawn.1 < tiles.len() && res.spawn.0 < tiles[res.spawn.1].len()
                && tiles[res.spawn.1].as_bytes()[res.spawn.0] == b'P'
            {
                let bytes = unsafe { tiles[res.spawn.1].as_bytes_mut() };
                bytes[res.spawn.0] = b'.';
            }

            // ★ 把原始手写小地图（40×26）「盖章」进大图左上角作为内容区：对象坐标即基于
            //   该小图设计，内部本就连通且含出生点 'P'（仅 L1），盖章后对象全部落在连通地板上。
            let small = match fi {
                0 => BAISUN_L1_MAP,
                1 => BAISUN_L2_MAP,
                _ => BAISUN_L3_MAP,
            };
            worldgen::stamp_map(tiles, small, 0, 0);

            // ★ 固定本世界对象/传送落点的格为可走，保证坐标权威并在新地图上可达（防御性）。
            let mut carve = |pts: &[(usize, usize)]| {
                for &(x, y) in pts {
                    worldgen::carve_walkable(tiles, x, y);
                }
            };
            let (l1, l2, l3) = (
                [
                    (18, 3), (18, 8), (30, 10), (18, 14), // bs_p_l1_*点
                    (33, 15),                            // bs_n_l1_guard
                    (34, 18),                            // bs_z_l1_drop
                    (37, 22),                            // p_bs_1 源
                ],
                [
                    (10, 4), (10, 9), (10, 14), (20, 4), // bs_p_l2_*点
                    (33, 10),                            // bs_n_l2_clerk
                    (10, 20),                            // bs_z_l2_elev
                    (2, 3),                              // p_bs_1 落点
                    (37, 22),                            // p_bs_2 源
                    (10, 22),                            // bs_g2 门禁
                ],
                [
                    (8, 8), (18, 8), (18, 18), (27, 8),  // bs_p_l3_*点
                    (32, 14),                            // bs_n_l3_janitor
                    (18, 22),                            // bs_z_l3_fire
                    (34, 22),                            // bs_z_l3_emissary
                    (30, 20),                            // bs_e_l3_watcher
                    (2, 3),                              // p_bs_2 落点
                    (30, 4),                             // bs_g3 门禁
                ],
            );
            match fi {
                0 => carve(&l1),
                1 => carve(&l2),
                _ => carve(&l3),
            }

            // ★ 全图连通：自内容区首个开放格（L1 即 'P'）出发，桥接程序化大图主区。
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

/// 每行固定 40 字符。L1 医院停车场：升降机边立柱 / 吊机摇臂 / 焦黑轿车 / 湿滑警戒带 / 出口升降梯。
#[allow(dead_code)] // 已被 worldgen 生成地图取代，保留作主题参考
pub static BAISUN_L1_MAP: &[&str] = &[
"########################################",
"#......................................#",
"#....I.................................#",
"#.......P.........I....................#",
"#......................................#",
"#........................I.............#",
"#..................##.....##...........#",
"#......................................#",
"#...........I..........................#",
"#......................................#",
"#.............................I........#",
"#......................................#",
"#...........................I..........#",
"#...................................#..#",
"#.................I.................#..#",
"#...................................#..#",
"#......................................#",
"#...................................##.#",
"#.................................I.##.#",
"#...................................##.#",
"#....##................................#",
"#..............##......................#",
"#....................................I.#",
"#......................................#",
"#......................................#",
"########################################",
];

/// L2 室内商场合订结构：扶梯连廊 / 停用电梯轿厢 / 扶梯扶手 / 商场风道 / 货运电梯间。
#[allow(dead_code)]
pub static BAISUN_L2_MAP: &[&str] = &[
"########################################",
"#......................................#",
"#......................................#",
"#......................................#",
"#.........I.........I..................#",
"#......................................#",
"#......................................#",
"#......................................#",
"#......................................#",
"#.........I.............##.............#",
"#.......................##.......I.....#",
"#.......................##.............#",
"#......................................#",
"#....###...............................#",
"#.........I............................#",
"#......................................#",
"#......................................#",
"#.............................##.......#",
"#.....I.......................##.......#",
"#.............................##.......#",
"#.........I...I........................#",
"#......................................#",
"#....................................I.#",
"#......................................#",
"#......................................#",
"########################################",
];

/// L3 电影院逃生梯：放映室 / 灭火器柜 / 洒水喷头 / 逃生指示牌 / 防火门 / 楼梯间（使者象征战）。
#[allow(dead_code)]
pub static BAISUN_L3_MAP: &[&str] = &[
"########################################",
"#......................................#",
"#......................................#",
"#......................................#",
"#.......I..............................#",
"#......................................#",
"#......................................#",
"#......................................#",
"#.......I.........I........I.....I.....#",
"#......................................#",
"#......................................#",
"#......................................#",
"#......................................#",
"#.......I..............................#",
"#...............................I......#",
"#......................................#",
"#......................................#",
"#................................##....#",
"#.................I....................#",
"#......................................#",
"#................................##....#",
"#......................................#",
"#.................I.......##......I....#",
"#......................................#",
"#......................................#",
"########################################",
];

pub static BAISUN_FLOOR_NAMES: &[&str] = &[
    "L1 医院停车场",
    "L2 室内商场合订结构",
    "L3 电影院逃生梯",
];

/// 调查点：全部 id 以 bs_p_ 前缀，route 引用 scenes_baisun.rs 的 BAISUN_SCENES 场景 id。
/// 这些点就是「死亡征兆」的观测锚：预判征兆 → flag 改写命运（避免连环死亡）。
pub static POINTS: &[maps::PointDef] = &[
    // ---- L1 医院停车场 ----
    maps::PointDef { id: "bs_p_l1_crane", name: "吊机摇臂", floor: 0, x: 18, y: 3, route: "bs_01_crane" },
    maps::PointDef { id: "bs_p_l1_slip", name: "湿滑警戒带", floor: 0, x: 18, y: 8, route: "bs_01_slip" },
    maps::PointDef { id: "bs_p_l1_car", name: "焦黑轿车", floor: 0, x: 30, y: 10, route: "bs_01_car" },
    maps::PointDef { id: "bs_p_l1_box", name: "悬空铁箱", floor: 0, x: 18, y: 14, route: "bs_01_box" },
    // ---- L2 室内商场合订结构 ----
    maps::PointDef { id: "bs_p_l2_handrail", name: "扶梯扶手", floor: 1, x: 10, y: 4, route: "bs_04_handrail" },
    maps::PointDef { id: "bs_p_l2_cabinet", name: "停用电梯轿厢", floor: 1, x: 10, y: 9, route: "bs_04_cabinet" },
    maps::PointDef { id: "bs_p_l2_sign", name: "超载报警灯", floor: 1, x: 10, y: 14, route: "bs_04_sign" },
    maps::PointDef { id: "bs_p_l2_duct", name: "商场风道", floor: 1, x: 20, y: 4, route: "bs_04_duct" },
    // ---- L3 电影院逃生梯 ----
    maps::PointDef { id: "bs_p_l3_projector", name: "放映室胶卷", floor: 2, x: 8, y: 8, route: "bs_07_projector" },
    maps::PointDef { id: "bs_p_l3_extinguisher", name: "灭火器柜", floor: 2, x: 18, y: 8, route: "bs_07_extinguisher" },
    maps::PointDef { id: "bs_p_l3_sprinkler", name: "洒水喷头", floor: 2, x: 18, y: 18, route: "bs_07_sprinkler" },
    maps::PointDef { id: "bs_p_l3_sign", name: "逃生指示牌", floor: 2, x: 27, y: 8, route: "bs_07_sign" },
];

/// 敌人：极少「机关形」象征物。满共识——死神无实体，不主动袭击；
/// 仅 L3 楼梯间保留 1 尊「逃生梯巡查员（异样）」作使者象征战的一次性代理（守卫形立绘复用）。
/// 该 fight 指向选择驱动 BOSS（bs_boss，HP150），玩家踏入即进入 bs_garage 象征战对话。
pub static ENEMIES: &[maps::EnemyDef] = &[
    maps::EnemyDef { id: "bs_e_l3_watcher", name: "逃生梯巡查员（异样）", floor: 2, x: 30, y: 20, radius: 3, fight: "bs_boss" },
];

/// 同伴 NPC：停车场保安 / 商场客服 / 电影院清洁工。——玩家识破征兆、改命的「证人与提示」。
pub static NPCS: &[maps::NpcDef] = &[
    maps::NpcDef { id: "bs_n_l1_guard", name: "停车场保安", floor: 0, x: 33, y: 15, talk: "bs_02_guard" },
    maps::NpcDef { id: "bs_n_l2_clerk", name: "商场客服", floor: 1, x: 33, y: 10, talk: "bs_05_clerk" },
    maps::NpcDef { id: "bs_n_l3_janitor", name: "电影院清洁工", floor: 2, x: 32, y: 14, talk: "bs_08_janitor" },
];

/// 特殊区域（ZoneDef）：环境机关「意外死亡」表演区（停车场坠物/商场电梯夹/电影院火灾逃生）。
///   进入 ref_id 场景即触发 San 惩罚 +（未识破征兆时）致死死亡档案「意外身故」；
///   识破过对应征兆（bs_foresee_*）则进入「改写命运」免死分支。L3 楼梯间为唯一 fight（使者象征战）。
///   ★引擎 maps.rs/main.rs 不支持 kind="env"（env 机关 attack 会走 run_zone_combat_round 做无名战斗），
///   故 3 个规则流 env 机关已降级为 kind="puzzle"（剧情仍经 POINTS/场景链 bs_10/11/12 可达）。
pub static ZONES: &[maps::ZoneDef] = &[
    // L1 吊机下方吊物区：坠物意外（征兆 bs_foresee_drop，规则流引擎无 kind=env，降级为 puzzle）
    maps::ZoneDef { id: "bs_z_l1_drop", name: "吊机坠物区", floor: 0, x: 34, y: 18, kind: "puzzle", ref_id: "bs_10_drop" },
    // L2 停用电梯轿厢：电梯夹意外（征兆 bs_foresee_elev，降级为 puzzle）
    maps::ZoneDef { id: "bs_z_l2_elev", name: "电梯轿厢夹缝", floor: 1, x: 10, y: 20, kind: "puzzle", ref_id: "bs_11_elev" },
    // L3 放映室外走廊：火灾逃生意外（征兆 bs_foresee_fire，降级为 puzzle）
    maps::ZoneDef { id: "bs_z_l3_fire", name: "逃生梯失火段", floor: 2, x: 18, y: 22, kind: "puzzle", ref_id: "bs_12_fire" },
    // L3 楼梯间 · 死神·使者（唯一象征战，规则流主线可完全绕过）
    maps::ZoneDef { id: "bs_z_l3_emissary", name: "死神·使者（象征战）", floor: 2, x: 34, y: 22, kind: "fight", ref_id: "bs_boss" },
];

/// 传送门（副本单向逐步推进）：L1 出口升降梯 → L2 商场连廊 → L3 电影院逃生梯。
pub static PORTALS: &[maps::PortalDef] = &[
    // p_bs_1 L1 停车场出口升降梯 → L2 商场内部连廊（单向上）
    maps::PortalDef { id: "p_bs_1", floor: 0, x: 37, y: 22, to_floor: 1, tx: 2, ty: 3 },
    // p_bs_2 L2 货运电梯 → L3 电影院逃生梯入口（单向）
    maps::PortalDef { id: "p_bs_2", floor: 1, x: 37, y: 22, to_floor: 2, tx: 2, ty: 3 },
];

/// 门禁（GateDef 软锁）：G2 货运电梯轿厢门（需已识破电梯征兆）/ G3 逃生梯防火门（需已识破火灾征兆）。
pub static GATES: &[maps::GateDef] = &[
    // G2 货运电梯轿厢门：需已识破电梯征兆才可借货梯直上 L3；否则绕行商场中庭险路
    maps::GateDef {
        id: "bs_g2", name: "货运电梯轿厢门", floor: 1, x: 10, y: 22,
        need_item: None, need_flag: Some("bs_foresee_elev"),
        lock_msg: "锈住的货梯门缝里漏出被绞断的防坠链——<em>未识破「电梯夹」征兆前，踏入轿厢就是死路</em>。",
        unlock_msg: "你已识破电梯夹缝的征兆，避开那道会咬人的门缝。货梯「咣」地停稳，铁丝网门朝你敞开一条直上通道。",
    },
    // G3 逃生梯防火门：需已识破火灾征兆才借消防通道脱险；否则强穿失火段
    maps::GateDef {
        id: "bs_g3", name: "逃生梯防火门", floor: 2, x: 30, y: 4,
        need_item: None, need_flag: Some("bs_foresee_fire"),
        lock_msg: "防火门被高温烤得发红，门把冒着白烟——<em>未识破「火灾」征兆前，推开它就是把自己送进去</em>。",
        unlock_msg: "你已识破影院火灾的征兆，绕开冒烟的走廊。防火门背后，消防通道的冷风让你活了下来。",
    },
];