//! 多世界框架：世界数据注册表（静态数据编译期常驻）+ 当前世界查询
//! 设计依据 §5.1：world_id 常量 + WorldData 定义 + WORLDS 注册表 + find_world。
//! P0 注册生化（BIOHAZARD）；P1 主神（ZHUTIAN）与 GW_PORTALS；P3 咒怨（ZHOUYUAN）。
mod cangjingge;
mod hezi;
mod jianzhong;
mod jiguancheng;
mod juluoji;
mod mojiao;
mod moruiya;
mod moshi;
mod mumiyi;
mod sishen;
mod tianshe;
mod tianting;
mod tongqu;
mod wulin;
mod xinghe;
mod yinse;
mod yiying;
mod zhutian;
mod zhouyuan;
mod shaqiu;
mod yize;
mod poxiao;
mod tiexue;
mod yecheng;

use crate::maps;

pub const WORLD_BIOHAZARD: &str = "biohazard_ch1";
pub const WORLD_ZHUTIAN: &str = "zhutianshenkong"; // P1 主神空间
pub const WORLD_ZHOUYUAN: &str = "zhuyuan";         // P3 咒怨
pub const WORLD_MOSHI: &str = "moshi_shoucheng";    // 末世死城·人类防线
pub const WORLD_YINSE: &str = "yinse_dadi";         // 银色大地·地灵族机界遗迹
pub const WORLD_YIYING: &str = "yiying";            // 异形4·奥瑞迦号
pub const WORLD_TIANSHE: &str = "tianshe";          // 天蛇族地下实验室
pub const WORLD_JIGUAN: &str = "jiguancheng";       // 侠行天下·机关城核心
pub const WORLD_MORUIYA: &str = "moruiya";          // 魔戒·摩瑞亚矿坑
pub const WORLD_CANGJING: &str = "cangjingge";      // 侠行天下·藏经阁·绝学之争
pub const WORLD_JIANZHONG: &str = "jianzhong";      // 侠行天下·剑冢禁地
pub const WORLD_TONGQU: &str = "tongqu";            // 侠行天下·通衢古镇·夜雨镖局
pub const WORLD_JULUOJI: &str = "juluoji";          // 侏罗纪公园
pub const WORLD_XINGHE: &str = "xinghe";            // 星河异形·巢穴
pub const WORLD_SISHEN: &str = "sishen";            // 死神来了·机场危机
pub const WORLD_MUMIYI: &str = "mumiyi";            // 木乃伊·哈姆纳塔地宫
pub const WORLD_MOJIAO: &str = "mojiao";            // 魔教总坛·血月
pub const WORLD_WULIN: &str = "wulin";              // 侠行天下·武林大会
pub const WORLD_TIANTING: &str = "tianting";        // 洪荒天庭·被封印的天庭残境
pub const WORLD_HEZI: &str = "hezi";                // 盒壁层·异位面（倒影界）
pub const WORLD_SHAQIU: &str = "shaqiu";            // 沙丘魔海·坠毁之星
pub const WORLD_YIZE: &str = "yize";                // 远古遗迹·遗泽
pub const WORLD_POXIAO: &str = "poxiao";            // 破晓封锁区
pub const WORLD_TIEXUE: &str = "tiexue";            // 铁血·地底金字塔
pub const WORLD_YECHENG: &str = "yecheng";          // 无限曙光·我们是传奇位面·夜城·孤独之城

/// 跨世界网关（P1 主神→生化 打通，主神→咒怨 占位 P3）。
/// maps::PortalDef 无 to_world 字段（P2 才加），且本阶段不改 maps.rs，
/// 故跨世界网关独立于各世界对象表，承载目标世界与落点。available=false = 占位不可交互。
pub struct WorldGateway {
    pub id: &'static str,          // gw_biohazard / gw_zhouyuan（全局唯一）
    pub from_world: &'static str,  // 网关所在世界
    pub floor: usize,
    pub x: usize,
    pub y: usize,
    pub to_world: &'static str,
    pub to_floor: usize,
    pub tx: usize,
    pub ty: usize,
    pub available: bool,           // false = 占位（不可交互）
}

/// 全局跨世界网关表（world_static，不属任何单世界；§2.6/§4.2）。
/// 主神→生化：落点 = 生化 F1 出生点 (1,1)；主神→咒怨：P3 启用，占位。
pub static GW_PORTALS: &[WorldGateway] = &[
    WorldGateway {
        id: "gw_biohazard",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 8,
        to_world: WORLD_BIOHAZARD, to_floor: 0, tx: 1, ty: 1,
        available: true,
    },
    WorldGateway {
        id: "gw_zhouyuan",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 18,
        to_world: WORLD_ZHOUYUAN, to_floor: 0, tx: 7, ty: 24, // 落点 = 咒怨 F1 玄关入口 P
        available: true, // 咒怨 P3 已接入（2026-08-27 合并）
    },
    // —— 4 新副本网关（主神→各副本，落点=各副本出生点 P；主神地图 room 层不摆新门，
    // 交互按 objId 经 gw_portal_by_id 直接触发，不依赖 tile 可达）——
    WorldGateway {
        id: "gw_moshi",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 9,
        to_world: WORLD_MOSHI, to_floor: 0, tx: 6, ty: 6, // 落点 = 末世 F1 城墙平台出生点 P(6,6)（地图实际落位）
        available: true,
    },
    WorldGateway {
        id: "gw_yinse",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 17,
        to_world: WORLD_YINSE, to_floor: 0, tx: 2, ty: 13, // 落点 = 银色 L1 降落点 P(2,13)
        available: true,
    },
    WorldGateway {
        id: "gw_yiying",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 19,
        to_world: WORLD_YIYING, to_floor: 0, tx: 22, ty: 17, // 落点 = 异形4 F1 生活区出生点 P(22,17)
        available: true,
    },
    WorldGateway {
        id: "gw_tianshe",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 20,
        to_world: WORLD_TIANSHE, to_floor: 0, tx: 1, ty: 1, // 落点 = 天蛇 L1 出生点 P(1,1)
        available: true,
    },
    WorldGateway {
        id: "gw_jiguancheng",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 21,
        to_world: WORLD_JIGUAN, to_floor: 0, tx: 14, ty: 20, // 落点 = 机关城 L1 城门出生点 P(14,20)
        available: true,
    },
    WorldGateway {
        id: "gw_moruiya",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 22,
        to_world: WORLD_MORUIYA, to_floor: 0, tx: 12, ty: 1, // 落点 = 摩瑞亚 F1 西闸门内侧出生点 P(12,1)
        available: true,
    },
    // —— 第二批 9 副本网关（主神→各副本，落点=各副本出生点 P）——
    WorldGateway {
        id: "gw_cangjingge",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 23,
        to_world: WORLD_CANGJING, to_floor: 0, tx: 14, ty: 20, // 落点 = 藏经阁 L0 经堂出生点 P(14,20)
        available: true,
    },
    WorldGateway {
        id: "gw_jianzhong",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 24,
        to_world: WORLD_JIANZHONG, to_floor: 0, tx: 20, ty: 24, // 落点 = 剑冢 L1 山门古道入口 P(20,24)
        available: true,
    },
    WorldGateway {
        id: "gw_tongqu",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 25,
        to_world: WORLD_TONGQU, to_floor: 0, tx: 14, ty: 20, // 落点 = 通衢 L1 镇门出生点 P(14,20)
        available: true,
    },
    WorldGateway {
        id: "gw_juluoji",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 26,
        to_world: WORLD_JULUOJI, to_floor: 0, tx: 1, ty: 20, // 落点 = 侏罗纪 L1 园区出生点 P(1,20)
        available: true,
    },
    WorldGateway {
        id: "gw_xinghe",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 27,
        to_world: WORLD_XINGHE, to_floor: 0, tx: 5, ty: 14, // 落点 = 星河 L1 登陆场出生点 P(5,14)
        available: true,
    },
    WorldGateway {
        id: "gw_sishen",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 28,
        to_world: WORLD_SISHEN, to_floor: 0, tx: 20, ty: 5, // 落点 = 死神 L1 候机大厅出生点 P(20,5)
        available: true,
    },
    WorldGateway {
        id: "gw_mumiyi",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 29,
        to_world: WORLD_MUMIYI, to_floor: 0, tx: 19, ty: 22, // 落点 = 木乃伊 F0 地宫入口出生点 P(19,22)
        available: true,
    },
    WorldGateway {
        id: "gw_mojiao",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 30,
        to_world: WORLD_MOJIAO, to_floor: 0, tx: 27, ty: 24, // 落点 = 魔教 L1 血月山道出生点 P(27,24)
        available: true,
    },
    WorldGateway {
        id: "gw_wulin",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 31,
        to_world: WORLD_WULIN, to_floor: 0, tx: 18, ty: 20, // 落点 = 武林大会 L1 山门出生点 P(18,20)
        available: true,
    },
    // —— 第三批 2 副本网关（主神→各副本，落点=各副本出生点 P）——
    WorldGateway {
        id: "gw_tianting",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 32,
        to_world: WORLD_TIANTING, to_floor: 0, tx: 2, ty: 13, // 落点 = 洪荒天庭 L1 坠落点出生点 P(2,13)
        available: true,
    },
    WorldGateway {
        id: "gw_hezi",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 33,
        to_world: WORLD_HEZI, to_floor: 0, tx: 1, ty: 1, // 落点 = 异位面·倒影界 F1 倒映平原出生点 P(1,1)
        available: true,
    },
    // —— 第四批 4 副本网关（主神→各副本，落点=各副本出生点 P）——
    WorldGateway {
        id: "gw_shaqiu",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 34,
        to_world: WORLD_SHAQIU, to_floor: 0, tx: 4, ty: 14, // 落点 = 沙丘 F1 坠毁穿梭机残骸出生点 P(4,14)
        available: true,
    },
    WorldGateway {
        id: "gw_yize",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 35,
        to_world: WORLD_YIZE, to_floor: 0, tx: 19, ty: 24, // 落点 = 遗泽 F1 入口大厅出生点 P(19,24)
        available: true,
    },
    WorldGateway {
        id: "gw_poxiao",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 36,
        to_world: WORLD_POXIAO, to_floor: 0, tx: 4, ty: 24, // 落点 = 破晓 L1 封锁城区街道出生点 P(4,24)
        available: true,
    },
    WorldGateway {
        id: "gw_tiexue",
        from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 37,
        to_world: WORLD_TIEXUE, to_floor: 0, tx: 1, ty: 1, // 落点 = 铁血 L1 冰层营地出生点 P(1,1)
        available: true,
    },
    WorldGateway { id: "gw_tiexue2", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 38, to_world: WORLD_TIEXUE2, to_floor: 0, tx: 1, ty: 1, available: true },
    WorldGateway { id: "gw_xingjichuanqi", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 39, to_world: WORLD_XINGJICHUANQI, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_xinhuangfang", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 40, to_world: WORLD_XINHUANGFANG, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_huanxiongshi", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 41, to_world: WORLD_HUANXIONGSHI, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_mengguijie", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 42, to_world: WORLD_MENGGUIJIE, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_siwuzhen", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 43, to_world: WORLD_SIWUZHEN, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_jingjiling", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 44, to_world: WORLD_JINGJILING, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_shenmiao", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 45, to_world: WORLD_SHENMIAO, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_shuangbai", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 46, to_world: WORLD_SHUANGBAI, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_dashengtang", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 47, to_world: WORLD_DASHENGTANG, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_daliexi", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 48, to_world: WORLD_DALIEXI, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_poxu", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 49, to_world: WORLD_POXU, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_hangu", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 50, to_world: WORLD_HANGU, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_panbu", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 51, to_world: WORLD_PANBU, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_diweidu", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 52, to_world: WORLD_DIWEIDU, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_sanlian", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 53, to_world: WORLD_SANLIAN, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_wujin", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 54, to_world: WORLD_WUJIN, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_yizhong", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 55, to_world: WORLD_YIZHONG, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_jishengqianye", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 56, to_world: WORLD_JISHENGQIANYE, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_miwu", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 57, to_world: WORLD_MIWU, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_xingchen", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 58, to_world: WORLD_XINGCHEN, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_yinxiang", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 59, to_world: WORLD_YINXIANG, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_nuoya", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 60, to_world: WORLD_NUOYA, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_lanshan", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 61, to_world: WORLD_LANSHAN, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_shourongsuo", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 62, to_world: WORLD_SHOURONGSUO, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_tianwang", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 63, to_world: WORLD_TIANWANG, to_floor: 0, tx: 20, ty: 3, available: true },
    WorldGateway { id: "gw_xingjijianchuan", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 64, to_world: WORLD_XINGJIJIANCHUAN, to_floor: 0, tx: 20, ty: 3, available: true },
    // —— 第六批 6 副本网关（主神→各副本，落点=各副本出生点 P）——
    WorldGateway { id: "gw_xingjichuanqi2", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 65, to_world: WORLD_XINGJICHUANQI2, to_floor: 0, tx: 5, ty: 4, available: true },   // 落点 = 灰雾之心 L1 迷雾矿洞出生点 P(5,4)
    WorldGateway { id: "gw_jialebi", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 66, to_world: WORLD_JIALEBI, to_floor: 0, tx: 5, ty: 4, available: true },            // 落点 = 黑珍珠 L1 甲板出生点 P(5,4)
    WorldGateway { id: "gw_shenghua3", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 67, to_world: WORLD_SHENGHUA3, to_floor: 0, tx: 5, ty: 4, available: true },        // 落点 = 浣熊市地下 L1 下水道出生点 P(5,4)
    WorldGateway { id: "gw_jishujing", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 68, to_world: WORLD_JISHUJING, to_floor: 0, tx: 20, ty: 22, available: true },     // 落点 = 弗莱迪归来 L1 榆树街出生点 P(20,22)
    WorldGateway { id: "gw_baisun", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 69, to_world: WORLD_BAISUN, to_floor: 0, tx: 8, ty: 3, available: true },              // 落点 = 死神来了2 L1 停车场出生点 P(8,3)
    WorldGateway { id: "gw_bihai", from_world: WORLD_ZHUTIAN, floor: 0, x: 31, y: 70, to_world: WORLD_BIHAI, to_floor: 0, tx: 20, ty: 5, available: true },               // 落点 = 深海阴影 L1 潜水器舱出生点 P(20,5)
];

/// 查询当前世界的（有序）跨世界网关，含占位。由 api_world_interact 网关路由使用。
pub fn gw_portal_by_id(id: &str) -> Option<&'static WorldGateway> {
    GW_PORTALS.iter().find(|g| g.id == id)
}

/// 交互查找：对象 id 命中当前世界对象/网关之一。由 api_world_interact 网关/portal 分支使用。
pub fn gw_portal_in(world_id: &str, floor: usize, x: usize, y: usize) -> Option<&'static WorldGateway> {
    GW_PORTALS.iter().find(|g| g.from_world == world_id && g.floor == floor && g.x == x && g.y == y)
}

/// 一个世界的全部静态数据（P0 生化引用现有 maps 表，不搬家）
pub struct WorldData {
    pub id: &'static str,
    pub name: &'static str,
    pub difficulty: u8,
    /// 固定难度挑战关标记（任务三）：true 的少数挑战关（隐藏 BOSS / 周本 / 挑战塔）战斗强度不随主角缩放，
    /// fight_scale 用「固定基准」而非随主角强度，绝对强度固定——主角不够强打不过、够强碾压（验证"变强"锚点）。
    /// 默认 false；详见 power.rs::base_scale / fight_scale。
    pub fixed_difficulty: bool,
    pub initial_scene: &'static str,
    pub floors: &'static [&'static [&'static str]],
    pub floor_names: &'static [&'static str],
    pub points: &'static [maps::PointDef],
    pub enemies: &'static [maps::EnemyDef],
    pub npcs: &'static [maps::NpcDef],
    pub zones: &'static [maps::ZoneDef],
    pub portals: &'static [maps::PortalDef],
    pub gates: &'static [maps::GateDef],
}

impl WorldData {
    /// 出生点：在首层找到 'P' 标记，缺省回退 (1,1)
    pub fn spawn(&self) -> (usize, usize) {
        if let Some(row) = self.floors.first() {
            for (y, r) in (*row).iter().enumerate() {
                if let Some(x) = (*r).find('P') {
                    return (x, y);
                }
            }
        }
        (1, 1)
    }
}

/// 生化危机·蜂巢（biohazard_ch1）：主世界（P0）。
/// ★ 地图改用 `worldgen` 程序化生成（BIOHAZARD 为接入程序化大地图的主世界示范，方法与
///   baisun_world() 一致）：四层的 640×640 tile 在首次访问时 `Box::leak` 为 `'static` 并缓存，
///   WorldData 亦懒构建后泄漏。points/enemies/npcs/zones/portals/gates 仍用 maps:: 的静态表
///   （坐标仍在小图范围，生成大图时由 maps::biohazard_generated_floors() 按坐标权威凿为可走）。
fn biohazard_world() -> &'static WorldData {
    use std::sync::OnceLock;
    static B: OnceLock<&'static WorldData> = OnceLock::new();
    B.get_or_init(|| {
        Box::leak(Box::new(WorldData {
            id: WORLD_BIOHAZARD,
            name: "生化危机·蜂巢",
            difficulty: 1,
            fixed_difficulty: false,
            initial_scene: "s_office",
            floors: maps::biohazard_generated_floors(),
            floor_names: &maps::FLOOR_NAMES,
            points: maps::POINTS,
            enemies: maps::ENEMIES,
            npcs: maps::NPCS,
            zones: maps::ZONES,
            portals: maps::PORTALS,
            gates: maps::GATES,
        }))
    })
}

/// 主神空间（P1）：单层可玩枢纽世界，无战斗（enemies 空表）。
static ZHUTIAN: WorldData = WorldData {
    id: WORLD_ZHUTIAN,
    name: "主神空间",
    difficulty: 0,
    fixed_difficulty: false,
    initial_scene: "s_nexus",
    floors: &[zhutian::ZHUTIAN_MAP],
    floor_names: zhutian::ZHUTIAN_FLOOR_NAMES,
    points: zhutian::POINTS,
    enemies: &[],
    npcs: zhutian::NPCS,
    zones: &[],
    portals: zhutian::PORTALS,
    gates: zhutian::GATES,
};

/// 咒怨（P3）：佐伯家凶宅 3 层，SAN 第二血条 + 伽椰子 BOSS（worlds/zhouyuan.rs + scenes_zhouyuan.rs）。
static ZHOUYUAN: WorldData = WorldData {
    id: WORLD_ZHOUYUAN,
    name: "咒怨 · 佐伯家",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "zy_01",
    floors: &[zhouyuan::ZHOUYUAN_F1_MAP, zhouyuan::ZHOUYUAN_F2_MAP, zhouyuan::ZHOUYUAN_F3_MAP],
    floor_names: zhouyuan::ZHOUYUAN_FLOOR_NAMES,
    points: zhouyuan::POINTS,
    enemies: zhouyuan::ENEMIES,
    npcs: zhouyuan::NPCS,
    zones: zhouyuan::ZONES,
    portals: zhouyuan::PORTALS,
    gates: zhouyuan::GATES,
};

/// 末世死城（sl2 moshi）：4 层人类防线，兽潮生存（worlds/moshi.rs + scenes_moshi.rs）。
static MOSHI: WorldData = WorldData {
    id: WORLD_MOSHI,
    name: "末世死城·人类防线",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "ms_00",
    floors: &[moshi::MOSHI_F1_MAP, moshi::MOSHI_F2_MAP, moshi::MOSHI_F3_MAP, moshi::MOSHI_F4_MAP],
    floor_names: moshi::MOSHI_FLOOR_NAMES,
    points: moshi::POINTS,
    enemies: moshi::ENEMIES,
    npcs: moshi::NPCS,
    zones: moshi::ZONES,
    portals: moshi::PORTALS,
    gates: moshi::GATES,
};

/// 银色大地（yinse）：4 层地灵族机界遗迹（worlds/yinse.rs + scenes_yinse.rs）。
static YINSE: WorldData = WorldData {
    id: WORLD_YINSE,
    name: "银色大地·地灵族机界遗迹",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "ys_01_drop",
    floors: &[yinse::YINSE_F1_MAP, yinse::YINSE_F2_MAP, yinse::YINSE_F3_MAP, yinse::YINSE_F4_MAP],
    floor_names: yinse::YINSE_FLOOR_NAMES,
    points: yinse::POINTS,
    enemies: yinse::ENEMIES,
    npcs: yinse::NPCS,
    zones: yinse::ZONES,
    portals: yinse::PORTALS,
    gates: yinse::GATES,
};

/// 异形4（yiying）：3 层奥瑞迦号（worlds/yiying.rs + scenes_yiying.rs）。
static YIYING: WorldData = WorldData {
    id: WORLD_YIYING,
    name: "异形4·奥瑞迦号",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "yiy_s0_arrive",
    floors: &[yiying::YIYING_F1_MAP, yiying::YIYING_F2_MAP, yiying::YIYING_F3_MAP],
    floor_names: yiying::YIYING_FLOOR_NAMES,
    points: yiying::POINTS,
    enemies: yiying::ENEMIES,
    npcs: yiying::NPCS,
    zones: yiying::ZONES,
    portals: yiying::PORTALS,
    gates: yiying::GATES,
};

/// 天蛇（tianshe）：4 层地下实验室（L 前缀，非 F；worlds/tianshe.rs + scenes_tianshe.rs）。
static TIANSHE: WorldData = WorldData {
    id: WORLD_TIANSHE,
    name: "天蛇族地下实验室",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "ts_open",
    floors: &[tianshe::TIANSHE_L1_MAP, tianshe::TIANSHE_L2_MAP, tianshe::TIANSHE_L3_MAP, tianshe::TIANSHE_L4_MAP],
    floor_names: tianshe::TIANSHE_FLOOR_NAMES,
    points: tianshe::POINTS,
    enemies: tianshe::ENEMIES,
    npcs: tianshe::NPCS,
    zones: tianshe::ZONES,
    portals: tianshe::PORTALS,
    gates: tianshe::GATES,
};

/// 侠行天下·机关城核心（jiguancheng）：4 层机关秘城（worlds/jiguancheng.rs + scenes_jiguancheng.rs）。
static JIGUAN: WorldData = WorldData {
    id: WORLD_JIGUAN,
    name: "侠行天下·机关城核心",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "jg_00",
    floors: &[jiguancheng::JIGUAN_L1_MAP, jiguancheng::JIGUAN_L2_MAP, jiguancheng::JIGUAN_L3_MAP, jiguancheng::JIGUAN_L4_MAP],
    floor_names: jiguancheng::JIGUAN_FLOOR_NAMES,
    points: jiguancheng::POINTS,
    enemies: jiguancheng::ENEMIES,
    npcs: jiguancheng::NPCS,
    zones: jiguancheng::ZONES,
    portals: jiguancheng::PORTALS,
    gates: jiguancheng::GATES,
};

/// 魔戒·摩瑞亚矿坑（moruiya）：3 层矮人遗迹（worlds/moruiya.rs + scenes_moruiya.rs）。
static MORUIYA: WorldData = WorldData {
    id: WORLD_MORUIYA,
    name: "魔戒·摩瑞亚矿坑",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "mo_01_gate",
    floors: &[moruiya::MORUIYA_F1_MAP, moruiya::MORUIYA_F2_MAP, moruiya::MORUIYA_F3_MAP],
    floor_names: moruiya::MORUIYA_FLOOR_NAMES,
    points: moruiya::POINTS,
    enemies: moruiya::ENEMIES,
    npcs: moruiya::NPCS,
    zones: moruiya::ZONES,
    portals: moruiya::PORTALS,
    gates: moruiya::GATES,
};

/// 侠行天下·藏经阁（cangjingge）：4 层绝学之争（L 前缀；worlds/cangjingge.rs + scenes_cangjingge.rs）。
static CANGJING: WorldData = WorldData {
    id: WORLD_CANGJING,
    name: "侠行天下·藏经阁·绝学之争",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "cj_00",
    floors: &[cangjingge::CANGJING_L0_MAP, cangjingge::CANGJING_L1_MAP, cangjingge::CANGJING_L2_MAP, cangjingge::CANGJING_L3_MAP],
    floor_names: cangjingge::CANGJING_FLOOR_NAMES,
    points: cangjingge::POINTS,
    enemies: cangjingge::ENEMIES,
    npcs: cangjingge::NPCS,
    zones: cangjingge::ZONES,
    portals: cangjingge::PORTALS,
    gates: cangjingge::GATES,
};

/// 侠行天下·剑冢禁地（jianzhong）：4 层试炼之地（L 前缀；worlds/jianzhong.rs + scenes_jianzhong.rs）。
static JIANZHONG: WorldData = WorldData {
    id: WORLD_JIANZHONG,
    name: "侠行天下·剑冢禁地",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "jz_00",
    floors: &[jianzhong::JIANZHONG_L1_MAP, jianzhong::JIANZHONG_L2_MAP, jianzhong::JIANZHONG_L3_MAP, jianzhong::JIANZHONG_L4_MAP],
    floor_names: jianzhong::JIANZHONG_FLOOR_NAMES,
    points: jianzhong::POINTS,
    enemies: jianzhong::ENEMIES,
    npcs: jianzhong::NPCS,
    zones: jianzhong::ZONES,
    portals: jianzhong::PORTALS,
    gates: jianzhong::GATES,
};

/// 侠行天下·通衢古镇（tongqu）：3 层夜雨镖局（L 前缀；worlds/tongqu.rs + scenes_tongqu.rs）。
static TONGQU: WorldData = WorldData {
    id: WORLD_TONGQU,
    name: "侠行天下·通衢古镇·夜雨镖局",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "tq_00",
    floors: &[tongqu::TONGQU_L1_MAP, tongqu::TONGQU_L2_MAP, tongqu::TONGQU_L3_MAP],
    floor_names: tongqu::TONGQU_FLOOR_NAMES,
    points: tongqu::POINTS,
    enemies: tongqu::ENEMIES,
    npcs: tongqu::NPCS,
    zones: tongqu::ZONES,
    portals: tongqu::PORTALS,
    gates: tongqu::GATES,
};

/// 侏罗纪公园（juluoji）：3 层恐龙园区（L 前缀；worlds/juluoji.rs + scenes_juluoji.rs）。
static JULUOJI: WorldData = WorldData {
    id: WORLD_JULUOJI,
    name: "侏罗纪公园",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "jl_00",
    floors: &[juluoji::JULUOJI_L1_MAP, juluoji::JULUOJI_L2_MAP, juluoji::JULUOJI_L3_MAP],
    floor_names: juluoji::JULUOJI_FLOOR_NAMES,
    points: juluoji::POINTS,
    enemies: juluoji::ENEMIES,
    npcs: juluoji::NPCS,
    zones: juluoji::ZONES,
    portals: juluoji::PORTALS,
    gates: juluoji::GATES,
};

/// 星河异形·巢穴（xinghe）：3 层登陆迁徙（L 前缀；worlds/xinghe.rs + scenes_xinghe.rs）。
static XINGHE: WorldData = WorldData {
    id: WORLD_XINGHE,
    name: "星河异形·巢穴",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "xh_00",
    floors: &[xinghe::XINGHE_L1_MAP, xinghe::XINGHE_L2_MAP, xinghe::XINGHE_L3_MAP],
    floor_names: xinghe::XINGHE_FLOOR_NAMES,
    points: xinghe::POINTS,
    enemies: xinghe::ENEMIES,
    npcs: xinghe::NPCS,
    zones: xinghe::ZONES,
    portals: xinghe::PORTALS,
    gates: xinghe::GATES,
};

/// 死神来了（sishen）：3 层机场危机（L 前缀；worlds/sishen.rs + scenes_sishen.rs）。
static SISHEN: WorldData = WorldData {
    id: WORLD_SISHEN,
    name: "死神来了·机场危机",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "ss_00",
    floors: &[sishen::SISHEN_L1_MAP, sishen::SISHEN_L2_MAP, sishen::SISHEN_L3_MAP],
    floor_names: sishen::SISHEN_FLOOR_NAMES,
    points: sishen::POINTS,
    enemies: sishen::ENEMIES,
    npcs: sishen::NPCS,
    zones: sishen::ZONES,
    portals: sishen::PORTALS,
    gates: sishen::GATES,
};

/// 木乃伊·哈姆纳塔地宫（mumiyi）：3 层法老墓室（F 前缀，表名带 MUMIYI_；worlds/mumiyi.rs + scenes_mumiyi.rs）。
static MUMIYI: WorldData = WorldData {
    id: WORLD_MUMIYI,
    name: "木乃伊·哈姆纳塔地宫",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "mm_00_camp",
    floors: &[mumiyi::MUMIYI_F0_MAP, mumiyi::MUMIYI_F1_MAP, mumiyi::MUMIYI_F2_MAP],
    floor_names: mumiyi::MUMIYI_FLOOR_NAMES,
    points: mumiyi::MUMIYI_POINTS,
    enemies: mumiyi::MUMIYI_ENEMIES,
    npcs: mumiyi::MUMIYI_NPCS,
    zones: mumiyi::MUMIYI_ZONES,
    portals: mumiyi::MUMIYI_PORTALS,
    gates: mumiyi::MUMIYI_GATES,
};

/// 魔教总坛·血月（mojiao）：4 层邪教圣地（L 前缀；worlds/mojiao.rs + scenes_mojiao.rs）。
static MOJIAO: WorldData = WorldData {
    id: WORLD_MOJIAO,
    name: "魔教总坛·血月",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "mj_00",
    floors: &[mojiao::MOJIAO_L1_MAP, mojiao::MOJIAO_L2_MAP, mojiao::MOJIAO_L3_MAP, mojiao::MOJIAO_L4_MAP],
    floor_names: mojiao::MOJIAO_FLOOR_NAMES,
    points: mojiao::POINTS,
    enemies: mojiao::ENEMIES,
    npcs: mojiao::NPCS,
    zones: mojiao::ZONES,
    portals: mojiao::PORTALS,
    gates: mojiao::GATES,
};

/// 侠行天下·武林大会（wulin）：4 层群雄会武（L 前缀；worlds/wulin.rs + scenes_wulin.rs）。
static WULIN: WorldData = WorldData {
    id: WORLD_WULIN,
    name: "侠行天下·武林大会",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "wl_00",
    floors: &[wulin::WULIN_L1_MAP, wulin::WULIN_L2_MAP, wulin::WULIN_L3_MAP, wulin::WULIN_L4_MAP],
    floor_names: wulin::WULIN_FLOOR_NAMES,
    points: wulin::POINTS,
    enemies: wulin::ENEMIES,
    npcs: wulin::NPCS,
    zones: wulin::ZONES,
    portals: wulin::PORTALS,
    gates: wulin::GATES,
};

/// 洪荒天庭（tianting）：4 层被封印的天庭残境，高难世界（L 前缀；worlds/tianting.rs + scenes_tianting.rs）。
static TIANTING: WorldData = WorldData {
    id: WORLD_TIANTING,
    name: "洪荒天庭 · 倒悬的王座",
    difficulty: 3,
    // 固定难度挑战关示例：高难度 BOSS 世界，绝对强度不随主角缩放——不够强打不过、够强碾压（验证"变强"锚点）。
    fixed_difficulty: true,
    initial_scene: "tt_01_drop",
    floors: &[tianting::TIANTING_F1_MAP, tianting::TIANTING_F2_MAP, tianting::TIANTING_F3_MAP, tianting::TIANTING_F4_MAP],
    floor_names: tianting::TIANTING_FLOOR_NAMES,
    points: tianting::POINTS,
    enemies: tianting::ENEMIES,
    npcs: tianting::NPCS,
    zones: tianting::ZONES,
    portals: tianting::PORTALS,
    gates: tianting::GATES,
};

/// 盒壁层·异位面（hezi）：3 层倒影界·开放漫游展示世界（F 前缀；worlds/hezi.rs + scenes_hezi.rs）。
static HEZI: WorldData = WorldData {
    id: WORLD_HEZI,
    name: "异位面 · 倒影界",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "hz_00",
    floors: &[hezi::HEZI_F1_MAP, hezi::HEZI_F2_MAP, hezi::HEZI_F3_MAP],
    floor_names: hezi::HEZI_FLOOR_NAMES,
    points: hezi::POINTS,
    enemies: hezi::ENEMIES,
    npcs: hezi::NPCS,
    zones: hezi::ZONES,
    portals: hezi::PORTALS,
    gates: hezi::GATES,
};

/// 沙丘魔海（shaqiu）：4 层开放探索生存（F 前缀；worlds/shaqiu.rs + scenes_shaqiu.rs）。
static SHAQIU: WorldData = WorldData {
    id: WORLD_SHAQIU,
    name: "沙丘魔海 · 坠毁之星",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "sq_00_intro",
    floors: &[shaqiu::SHAQIU_F1_MAP, shaqiu::SHAQIU_F2_MAP, shaqiu::SHAQIU_F3_MAP, shaqiu::SHAQIU_F4_MAP],
    floor_names: shaqiu::SHAQIU_FLOOR_NAMES,
    points: shaqiu::POINTS,
    enemies: shaqiu::ENEMIES,
    npcs: shaqiu::NPCS,
    zones: shaqiu::ZONES,
    portals: shaqiu::PORTALS,
    gates: shaqiu::GATES,
};

/// 远古遗迹·遗泽（yize）：4 层任务世界（F 前缀；worlds/yize.rs + scenes_yize.rs）。
static YIZE: WorldData = WorldData {
    id: WORLD_YIZE,
    name: "远古遗迹 · 遗泽",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "yz_01_arrive",
    floors: &[yize::YIZE_F1_MAP, yize::YIZE_F2_MAP, yize::YIZE_F3_MAP, yize::YIZE_F4_MAP],
    floor_names: yize::YIZE_FLOOR_NAMES,
    points: yize::POINTS,
    enemies: yize::ENEMIES,
    npcs: yize::NPCS,
    zones: yize::ZONES,
    portals: yize::PORTALS,
    gates: yize::GATES,
};

/// 破晓封锁区（poxiao）：3 层封锁城区/RPG（F 前缀但表名 POXIAO_；worlds/poxiao.rs + scenes_poxiao.rs）。
static POXIAO: WorldData = WorldData {
    id: WORLD_POXIAO,
    name: "破晓封锁区",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "px_00_open",
    floors: &[poxiao::POXIAO_F1_MAP, poxiao::POXIAO_F2_MAP, poxiao::POXIAO_F3_MAP],
    floor_names: poxiao::POXIAO_FLOOR_NAMES,
    points: poxiao::POINTS,
    enemies: poxiao::ENEMIES,
    npcs: poxiao::NPCS,
    zones: poxiao::ZONES,
    portals: poxiao::PORTALS,
    gates: poxiao::GATES,
};

/// 铁血·地底金字塔（tiexue）：3 层任务世界（L 前缀；worlds/tiexue.rs + scenes_tiexue.rs）。
static TIEXUE: WorldData = WorldData {
    id: WORLD_TIEXUE,
    name: "铁血 · 地底金字塔",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "tx_00_open",
    floors: &[tiexue::TIEXUE_L1_MAP, tiexue::TIEXUE_L2_MAP, tiexue::TIEXUE_L3_MAP],
    floor_names: tiexue::TIEXUE_FLOOR_NAMES,
    points: tiexue::POINTS,
    enemies: tiexue::ENEMIES,
    npcs: tiexue::NPCS,
    zones: tiexue::ZONES,
    portals: tiexue::PORTALS,
    gates: tiexue::GATES,
};

/// P0 生化；P1 主神；P3 咒怨；P4 六新副本；P5 第二批 9 副本；P6 第三批 2 副本；P7 第四批 4 副本
mod tiexue2;
mod xingjichuanqi;
mod xinhuangfang;
mod huanxiongshi;
mod mengguijie;
mod siwuzhen;
mod jingjiling;
mod shenmiao;
mod shuangbai;
mod dashengtang;
mod daliexi;
mod poxu;
mod hangu;
mod panbu;
mod diweidu;
mod sanlian;
mod wujin;
mod yizhong;
mod jishengqianye;
mod miwu;
mod xingchen;
mod yinxiang;
mod nuoya;
mod lanshan;
mod shourongsuo;
mod tianwang;
mod xingjijianchuan;

// —— 第六批 6 副本（P8）——
mod xingjichuanqi2;
mod jialebi;
mod shenghua3;
mod jishujing;
mod baisun;
mod bihai;

pub const WORLD_TIEXUE2: &str = "tiexue2";
pub const WORLD_XINGJICHUANQI: &str = "xingjichuanqi";
pub const WORLD_XINHUANGFANG: &str = "xinhuangfang";
pub const WORLD_HUANXIONGSHI: &str = "huanxiongshi";
pub const WORLD_MENGGUIJIE: &str = "mengguijie";
pub const WORLD_SIWUZHEN: &str = "siwuzhen";
pub const WORLD_JINGJILING: &str = "jingjiling";
pub const WORLD_SHENMIAO: &str = "shenmiao";
pub const WORLD_SHUANGBAI: &str = "shuangbai";
pub const WORLD_DASHENGTANG: &str = "dashengtang";
pub const WORLD_DALIEXI: &str = "daliexi";
pub const WORLD_POXU: &str = "poxu";
pub const WORLD_HANGU: &str = "hangu";
pub const WORLD_PANBU: &str = "panbu";
pub const WORLD_DIWEIDU: &str = "diweidu";
pub const WORLD_SANLIAN: &str = "sanlian";
pub const WORLD_WUJIN: &str = "wujin";
pub const WORLD_YIZHONG: &str = "yizhong";
pub const WORLD_JISHENGQIANYE: &str = "jishengqianye";
pub const WORLD_MIWU: &str = "miwu";
pub const WORLD_XINGCHEN: &str = "xingchen";
pub const WORLD_YINXIANG: &str = "yinxiang";
pub const WORLD_NUOYA: &str = "nuoya";
pub const WORLD_LANSHAN: &str = "lanshan";
pub const WORLD_SHOURONGSUO: &str = "shourongsuo";
pub const WORLD_TIANWANG: &str = "tianwang";
pub const WORLD_XINGJIJIANCHUAN: &str = "xingjijianchuan";
// —— 第六批 6 副本（P8）——
pub const WORLD_XINGJICHUANQI2: &str = "xingjichuanqi2";   // 星际传奇续·寂静岭·灰雾之心
pub const WORLD_JIALEBI: &str = "jialebi";                  // 无限恐怖·黑珍珠
pub const WORLD_SHENGHUA3: &str = "shenghua3";              // 无限恐怖·浣熊市地下
pub const WORLD_JISHUJING: &str = "jishujing";              // 无限恐怖·弗莱迪归来
pub const WORLD_BAISUN: &str = "baisun";                    // 无限恐怖·死神来了2
pub const WORLD_BIHAI: &str = "bihai";                      // 无限恐怖·深海阴影

static TIEXUE2: WorldData = WorldData {
    id: WORLD_TIEXUE2,
    name: "tiexue2",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "tx2_00_open",
    floors: &[tiexue2::TIEXUE2_F1_MAP, tiexue2::TIEXUE2_F2_MAP, tiexue2::TIEXUE2_F3_MAP],
    floor_names: tiexue2::TIEXUE2_FLOOR_NAMES,
    points: tiexue2::POINTS,
    enemies: tiexue2::ENEMIES,
    npcs: tiexue2::NPCS,
    zones: tiexue2::ZONES,
    portals: tiexue2::PORTALS,
    gates: tiexue2::GATES,
};

static XINGJICHUANQI: WorldData = WorldData {
    id: WORLD_XINGJICHUANQI,
    name: "xingjichuanqi",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "xj_00",
    floors: &[xingjichuanqi::XINGJICHUANQI_F1_MAP, xingjichuanqi::XINGJICHUANQI_F2_MAP, xingjichuanqi::XINGJICHUANQI_F3_MAP],
    floor_names: xingjichuanqi::XINGJICHUANQI_FLOOR_NAMES,
    points: xingjichuanqi::POINTS,
    enemies: xingjichuanqi::ENEMIES,
    npcs: xingjichuanqi::NPCS,
    zones: xingjichuanqi::ZONES,
    portals: xingjichuanqi::PORTALS,
    gates: xingjichuanqi::GATES,
};

static XINHUANGFANG: WorldData = WorldData {
    id: WORLD_XINHUANGFANG,
    name: "xinhuangfang",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "xf_00",
    floors: &[xinhuangfang::XINHUANGFANG_F1_MAP, xinhuangfang::XINHUANGFANG_F2_MAP, xinhuangfang::XINHUANGFANG_F3_MAP],
    floor_names: xinhuangfang::XINHUANGFANG_FLOOR_NAMES,
    points: xinhuangfang::POINTS,
    enemies: xinhuangfang::ENEMIES,
    npcs: xinhuangfang::NPCS,
    zones: xinhuangfang::ZONES,
    portals: xinhuangfang::PORTALS,
    gates: xinhuangfang::GATES,
};

static HUANXIONGSHI: WorldData = WorldData {
    id: WORLD_HUANXIONGSHI,
    name: "huanxiongshi",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "hx_00",
    floors: &[huanxiongshi::HUANXIONGSHI_F1_MAP, huanxiongshi::HUANXIONGSHI_F2_MAP, huanxiongshi::HUANXIONGSHI_F3_MAP],
    floor_names: huanxiongshi::HUANXIONGSHI_FLOOR_NAMES,
    points: huanxiongshi::POINTS,
    enemies: huanxiongshi::ENEMIES,
    npcs: huanxiongshi::NPCS,
    zones: huanxiongshi::ZONES,
    portals: huanxiongshi::PORTALS,
    gates: huanxiongshi::GATES,
};

static MENGGUIJIE: WorldData = WorldData {
    id: WORLD_MENGGUIJIE,
    name: "mengguijie",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "mg_00",
    floors: &[mengguijie::MENGGUIJIE_F1_MAP, mengguijie::MENGGUIJIE_F2_MAP, mengguijie::MENGGUIJIE_F3_MAP],
    floor_names: mengguijie::MENGGUIJIE_FLOOR_NAMES,
    points: mengguijie::POINTS,
    enemies: mengguijie::ENEMIES,
    npcs: mengguijie::NPCS,
    zones: mengguijie::ZONES,
    portals: mengguijie::PORTALS,
    gates: mengguijie::GATES,
};

static SIWUZHEN: WorldData = WorldData {
    id: WORLD_SIWUZHEN,
    name: "siwuzhen",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "sw_00",
    floors: &[siwuzhen::SIWUZHEN_F1_MAP, siwuzhen::SIWUZHEN_F2_MAP, siwuzhen::SIWUZHEN_F3_MAP],
    floor_names: siwuzhen::SIWUZHEN_FLOOR_NAMES,
    points: siwuzhen::POINTS,
    enemies: siwuzhen::ENEMIES,
    npcs: siwuzhen::NPCS,
    zones: siwuzhen::ZONES,
    portals: siwuzhen::PORTALS,
    gates: siwuzhen::GATES,
};

static JINGJILING: WorldData = WorldData {
    id: WORLD_JINGJILING,
    name: "jingjiling",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "jj_00",
    floors: &[jingjiling::JINGJILING_F1_MAP, jingjiling::JINGJILING_F2_MAP, jingjiling::JINGJILING_F3_MAP],
    floor_names: jingjiling::JINGJILING_FLOOR_NAMES,
    points: jingjiling::POINTS,
    enemies: jingjiling::ENEMIES,
    npcs: jingjiling::NPCS,
    zones: jingjiling::ZONES,
    portals: jingjiling::PORTALS,
    gates: jingjiling::GATES,
};

static SHENMIAO: WorldData = WorldData {
    id: WORLD_SHENMIAO,
    name: "shenmiao",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "sm_00",
    floors: &[shenmiao::SHENMIAO_F1_MAP, shenmiao::SHENMIAO_F2_MAP, shenmiao::SHENMIAO_F3_MAP],
    floor_names: shenmiao::SHENMIAO_FLOOR_NAMES,
    points: shenmiao::POINTS,
    enemies: shenmiao::ENEMIES,
    npcs: shenmiao::NPCS,
    zones: shenmiao::ZONES,
    portals: shenmiao::PORTALS,
    gates: shenmiao::GATES,
};

static SHUANGBAI: WorldData = WorldData {
    id: WORLD_SHUANGBAI,
    name: "shuangbai",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "sb_00",
    floors: &[shuangbai::SHUANGBAI_F1_MAP, shuangbai::SHUANGBAI_F2_MAP],
    floor_names: shuangbai::SHUANGBAI_FLOOR_NAMES,
    points: shuangbai::POINTS,
    enemies: shuangbai::ENEMIES,
    npcs: shuangbai::NPCS,
    zones: shuangbai::ZONES,
    portals: shuangbai::PORTALS,
    gates: shuangbai::GATES,
};

static DASHENGTANG: WorldData = WorldData {
    id: WORLD_DASHENGTANG,
    name: "dashengtang",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "ds_00",
    floors: &[dashengtang::DASHENGTANG_F1_MAP, dashengtang::DASHENGTANG_F2_MAP, dashengtang::DASHENGTANG_F3_MAP],
    floor_names: dashengtang::DASHENGTANG_FLOOR_NAMES,
    points: dashengtang::POINTS,
    enemies: dashengtang::ENEMIES,
    npcs: dashengtang::NPCS,
    zones: dashengtang::ZONES,
    portals: dashengtang::PORTALS,
    gates: dashengtang::GATES,
};

static DALIEXI: WorldData = WorldData {
    id: WORLD_DALIEXI,
    name: "daliexi",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "dl_00",
    floors: &[daliexi::DALIEXI_F1_MAP, daliexi::DALIEXI_F2_MAP, daliexi::DALIEXI_F3_MAP],
    floor_names: daliexi::DALIEXI_FLOOR_NAMES,
    points: daliexi::POINTS,
    enemies: daliexi::ENEMIES,
    npcs: daliexi::NPCS,
    zones: daliexi::ZONES,
    portals: daliexi::PORTALS,
    gates: daliexi::GATES,
};

static POXU: WorldData = WorldData {
    id: WORLD_POXU,
    name: "poxu",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "pv_00",
    floors: &[poxu::POXU_F1_MAP, poxu::POXU_F2_MAP, poxu::POXU_F3_MAP, poxu::POXU_F4_MAP],
    floor_names: poxu::POXU_FLOOR_NAMES,
    points: poxu::POINTS,
    enemies: poxu::ENEMIES,
    npcs: poxu::NPCS,
    zones: poxu::ZONES,
    portals: poxu::PORTALS,
    gates: poxu::GATES,
};

static HANGU: WorldData = WorldData {
    id: WORLD_HANGU,
    name: "hangu",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "hg_00",
    floors: &[hangu::HANGU_F1_MAP, hangu::HANGU_F2_MAP, hangu::HANGU_F3_MAP],
    floor_names: hangu::HANGU_FLOOR_NAMES,
    points: hangu::POINTS,
    enemies: hangu::ENEMIES,
    npcs: hangu::NPCS,
    zones: hangu::ZONES,
    portals: hangu::PORTALS,
    gates: hangu::GATES,
};

static PANBU: WorldData = WorldData {
    id: WORLD_PANBU,
    name: "panbu",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "pb_00",
    floors: &[panbu::PANBU_F1_MAP, panbu::PANBU_F2_MAP, panbu::PANBU_F3_MAP],
    floor_names: panbu::PANBU_FLOOR_NAMES,
    points: panbu::POINTS,
    enemies: panbu::ENEMIES,
    npcs: panbu::NPCS,
    zones: panbu::ZONES,
    portals: panbu::PORTALS,
    gates: panbu::GATES,
};

static DIWEIDU: WorldData = WorldData {
    id: WORLD_DIWEIDU,
    name: "diweidu",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "dw_00",
    floors: &[diweidu::DIWEIDU_F1_MAP, diweidu::DIWEIDU_F2_MAP, diweidu::DIWEIDU_F3_MAP],
    floor_names: diweidu::DIWEIDU_FLOOR_NAMES,
    points: diweidu::POINTS,
    enemies: diweidu::ENEMIES,
    npcs: diweidu::NPCS,
    zones: diweidu::ZONES,
    portals: diweidu::PORTALS,
    gates: diweidu::GATES,
};

static SANLIAN: WorldData = WorldData {
    id: WORLD_SANLIAN,
    name: "sanlian",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "sl_00",
    floors: &[sanlian::SANLIAN_F1_MAP, sanlian::SANLIAN_F2_MAP],
    floor_names: sanlian::SANLIAN_FLOOR_NAMES,
    points: sanlian::POINTS,
    enemies: sanlian::ENEMIES,
    npcs: sanlian::NPCS,
    zones: sanlian::ZONES,
    portals: sanlian::PORTALS,
    gates: sanlian::GATES,
};

static WUJIN: WorldData = WorldData {
    id: WORLD_WUJIN,
    name: "wujin",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "wj_00",
    floors: &[wujin::WUJIN_F1_MAP, wujin::WUJIN_F2_MAP, wujin::WUJIN_F3_MAP],
    floor_names: wujin::WUJIN_FLOOR_NAMES,
    points: wujin::POINTS,
    enemies: wujin::ENEMIES,
    npcs: wujin::NPCS,
    zones: wujin::ZONES,
    portals: wujin::PORTALS,
    gates: wujin::GATES,
};

static YIZHONG: WorldData = WorldData {
    id: WORLD_YIZHONG,
    name: "yizhong",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "yz_00",
    floors: &[yizhong::YIZHONG_F1_MAP, yizhong::YIZHONG_F2_MAP, yizhong::YIZHONG_F3_MAP],
    floor_names: yizhong::YIZHONG_FLOOR_NAMES,
    points: yizhong::POINTS,
    enemies: yizhong::ENEMIES,
    npcs: yizhong::NPCS,
    zones: yizhong::ZONES,
    portals: yizhong::PORTALS,
    gates: yizhong::GATES,
};

static JISHENGQIANYE: WorldData = WorldData {
    id: WORLD_JISHENGQIANYE,
    name: "jishengqianye",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "js_00",
    floors: &[jishengqianye::JISHENGQIANYE_F1_MAP, jishengqianye::JISHENGQIANYE_F2_MAP, jishengqianye::JISHENGQIANYE_F3_MAP],
    floor_names: jishengqianye::JISHENGQIANYE_FLOOR_NAMES,
    points: jishengqianye::POINTS,
    enemies: jishengqianye::ENEMIES,
    npcs: jishengqianye::NPCS,
    zones: jishengqianye::ZONES,
    portals: jishengqianye::PORTALS,
    gates: jishengqianye::GATES,
};

static MIWU: WorldData = WorldData {
    id: WORLD_MIWU,
    name: "miwu",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "mw_00",
    floors: &[miwu::MIWU_F1_MAP, miwu::MIWU_F2_MAP, miwu::MIWU_F3_MAP],
    floor_names: miwu::MIWU_FLOOR_NAMES,
    points: miwu::POINTS,
    enemies: miwu::ENEMIES,
    npcs: miwu::NPCS,
    zones: miwu::ZONES,
    portals: miwu::PORTALS,
    gates: miwu::GATES,
};

static XINGCHEN: WorldData = WorldData {
    id: WORLD_XINGCHEN,
    name: "xingchen",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "xc_00",
    floors: &[xingchen::XINGCHEN_F1_MAP, xingchen::XINGCHEN_F2_MAP, xingchen::XINGCHEN_F3_MAP],
    floor_names: xingchen::XINGCHEN_FLOOR_NAMES,
    points: xingchen::POINTS,
    enemies: xingchen::ENEMIES,
    npcs: xingchen::NPCS,
    zones: xingchen::ZONES,
    portals: xingchen::PORTALS,
    gates: xingchen::GATES,
};

static YINXIANG: WorldData = WorldData {
    id: WORLD_YINXIANG,
    name: "yinxiang",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "yx_00",
    floors: &[yinxiang::YINXIANG_F1_MAP, yinxiang::YINXIANG_F2_MAP, yinxiang::YINXIANG_F3_MAP],
    floor_names: yinxiang::YINXIANG_FLOOR_NAMES,
    points: yinxiang::POINTS,
    enemies: yinxiang::ENEMIES,
    npcs: yinxiang::NPCS,
    zones: yinxiang::ZONES,
    portals: yinxiang::PORTALS,
    gates: yinxiang::GATES,
};

static NUOYA: WorldData = WorldData {
    id: WORLD_NUOYA,
    name: "nuoya",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "ny_00",
    floors: &[nuoya::NUOYA_F1_MAP, nuoya::NUOYA_F2_MAP],
    floor_names: nuoya::NUOYA_FLOOR_NAMES,
    points: nuoya::POINTS,
    enemies: nuoya::ENEMIES,
    npcs: nuoya::NPCS,
    zones: nuoya::ZONES,
    portals: nuoya::PORTALS,
    gates: nuoya::GATES,
};

static LANSHAN: WorldData = WorldData {
    id: WORLD_LANSHAN,
    name: "lanshan",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "ls_00",
    floors: &[lanshan::LANSHAN_F1_MAP, lanshan::LANSHAN_F2_MAP, lanshan::LANSHAN_F3_MAP],
    floor_names: lanshan::LANSHAN_FLOOR_NAMES,
    points: lanshan::POINTS,
    enemies: lanshan::ENEMIES,
    npcs: lanshan::NPCS,
    zones: lanshan::ZONES,
    portals: lanshan::PORTALS,
    gates: lanshan::GATES,
};

static SHOURONGSUO: WorldData = WorldData {
    id: WORLD_SHOURONGSUO,
    name: "shourongsuo",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "sr_00",
    floors: &[shourongsuo::SHOURONGSUO_F1_MAP, shourongsuo::SHOURONGSUO_F2_MAP, shourongsuo::SHOURONGSUO_F3_MAP],
    floor_names: shourongsuo::SHOURONGSUO_FLOOR_NAMES,
    points: shourongsuo::POINTS,
    enemies: shourongsuo::ENEMIES,
    npcs: shourongsuo::NPCS,
    zones: shourongsuo::ZONES,
    portals: shourongsuo::PORTALS,
    gates: shourongsuo::GATES,
};

static TIANWANG: WorldData = WorldData {
    id: WORLD_TIANWANG,
    name: "tianwang",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "tw_00",
    floors: &[tianwang::TIANWANG_F1_MAP, tianwang::TIANWANG_F2_MAP, tianwang::TIANWANG_F3_MAP],
    floor_names: tianwang::TIANWANG_FLOOR_NAMES,
    points: tianwang::POINTS,
    enemies: tianwang::ENEMIES,
    npcs: tianwang::NPCS,
    zones: tianwang::ZONES,
    portals: tianwang::PORTALS,
    gates: tianwang::GATES,
};

static XINGJIJIANCHUAN: WorldData = WorldData {
    id: WORLD_XINGJIJIANCHUAN,
    name: "xingjijianchuan",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "xjj_00",
    floors: &[xingjijianchuan::XINGJIJIANCHUAN_F1_MAP, xingjijianchuan::XINGJIJIANCHUAN_F2_MAP, xingjijianchuan::XINGJIJIANCHUAN_F3_MAP],
    floor_names: xingjijianchuan::XINGJIJIANCHUAN_FLOOR_NAMES,
    points: xingjijianchuan::POINTS,
    enemies: xingjijianchuan::ENEMIES,
    npcs: xingjijianchuan::NPCS,
    zones: xingjijianchuan::ZONES,
    portals: xingjijianchuan::PORTALS,
    gates: xingjijianchuan::GATES,
};

/// 星际传奇续·寂静岭·灰雾之心（xingjichuanqi2）：3 层罪与罚（L 前缀；worlds/xingjichuanqi2.rs + scenes_xingjichuanqi2.rs）。
static XINGJICHUANQI2: WorldData = WorldData {
    id: WORLD_XINGJICHUANQI2,
    name: "寂静岭2·灰雾之心",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "xj2_00",
    floors: &[xingjichuanqi2::XINGJICHUANQI2_L1_MAP, xingjichuanqi2::XINGJICHUANQI2_L2_MAP, xingjichuanqi2::XINGJICHUANQI2_L3_MAP],
    floor_names: xingjichuanqi2::XINGJICHUANQI2_FLOOR_NAMES,
    points: xingjichuanqi2::POINTS,
    enemies: xingjichuanqi2::ENEMIES,
    npcs: xingjichuanqi2::NPCS,
    zones: xingjichuanqi2::ZONES,
    portals: xingjichuanqi2::PORTALS,
    gates: xingjichuanqi2::GATES,
};

/// 无限恐怖·黑珍珠（jialebi）：3 层航海冒险展示世界（L 前缀；worlds/jialebi.rs + scenes_jialebi.rs）。
static JIALEBI: WorldData = WorldData {
    id: WORLD_JIALEBI,
    name: "无限恐怖·黑珍珠",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "jb_00",
    floors: &[jialebi::JIALEBI_L1_MAP, jialebi::JIALEBI_L2_MAP, jialebi::JIALEBI_L3_MAP],
    floor_names: jialebi::JIALEBI_FLOOR_NAMES,
    points: jialebi::POINTS,
    enemies: jialebi::ENEMIES,
    npcs: jialebi::NPCS,
    zones: jialebi::ZONES,
    portals: jialebi::PORTALS,
    gates: jialebi::GATES,
};

/// 无限恐怖·浣熊市地下（shenghua3）：3 层生化幸存（worlds/shenghua3.rs + scenes_shenghua3.rs）。
static SHENGHUA3: WorldData = WorldData {
    id: WORLD_SHENGHUA3,
    name: "无限恐怖·浣熊市地下",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "sh3_00",
    floors: &[shenghua3::SHENGHUA3_L1_MAP, shenghua3::SHENGHUA3_L2_MAP, shenghua3::SHENGHUA3_L3_MAP],
    floor_names: shenghua3::SHENGHUA3_FLOOR_NAMES,
    points: shenghua3::POINTS,
    enemies: shenghua3::ENEMIES,
    npcs: shenghua3::NPCS,
    zones: shenghua3::ZONES,
    portals: shenghua3::PORTALS,
    gates: shenghua3::GATES,
};

/// 无限恐怖·弗莱迪归来（jishujing）：3 层梦境惊悚展示世界（L 前缀；worlds/jishujing.rs + scenes_jishujing.rs）。
static JISHUJING: WorldData = WorldData {
    id: WORLD_JISHUJING,
    name: "无限恐怖·弗莱迪归来",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "jj2_00",
    floors: &[jishujing::JISHUJING_L1_MAP, jishujing::JISHUJING_L2_MAP, jishujing::JISHUJING_L3_MAP],
    floor_names: jishujing::JISHUJING_FLOOR_NAMES,
    points: jishujing::POINTS,
    enemies: jishujing::ENEMIES,
    npcs: jishujing::NPCS,
    zones: jishujing::ZONES,
    portals: jishujing::PORTALS,
    gates: jishujing::GATES,
};

/// 无限恐怖·死神来了2（baisun）：3 层规则流机关（L 前缀；worlds/baisun.rs + scenes_baisun.rs）。
/// ★ 地图改用 `worldgen` 程序化生成（baisun 为示范世界），故不能为 `static` 字面量：
///   生成的三层 tile 在首次访问时 `Box::leak` 为 `'static` 并缓存，WorldData 亦懒构建后泄漏。
fn baisun_world() -> &'static WorldData {
    use std::sync::OnceLock;
    static B: OnceLock<&'static WorldData> = OnceLock::new();
    B.get_or_init(|| {
        Box::leak(Box::new(WorldData {
            id: WORLD_BAISUN,
            name: "无限恐怖·死神来了2",
            difficulty: 2,
            fixed_difficulty: false,
            initial_scene: "bs_00",
            floors: baisun::baisun_generated_floors(),
            floor_names: baisun::BAISUN_FLOOR_NAMES,
            points: baisun::POINTS,
            enemies: baisun::ENEMIES,
            npcs: baisun::NPCS,
            zones: baisun::ZONES,
            portals: baisun::PORTALS,
            gates: baisun::GATES,
        }))
    })
}

/// 无限恐怖·深海阴影（bihai）：3 层克苏鲁世界展示（L 前缀；worlds/bihai.rs + scenes_bihai.rs）。
static BIHAI: WorldData = WorldData {
    id: WORLD_BIHAI,
    name: "无限恐怖·深海阴影",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "bh_00",
    floors: &[bihai::BIHAI_L1_MAP, bihai::BIHAI_L2_MAP, bihai::BIHAI_L3_MAP],
    floor_names: bihai::BIHAI_FLOOR_NAMES,
    points: bihai::POINTS,
    enemies: bihai::ENEMIES,
    npcs: bihai::NPCS,
    zones: bihai::ZONES,
    portals: bihai::PORTALS,
    gates: bihai::GATES,
};

/// 无限曙光·我们是传奇位面·夜城·孤独之城（yecheng）：3 层昼夜生存世界展示。
/// 「原创致敬」拓展（design 见 tools/design/yecheng_impl_log.md）。
static YECHENG: WorldData = WorldData {
    id: WORLD_YECHENG,
    name: "夜城·孤独之城",
    difficulty: 2,
    fixed_difficulty: false,
    initial_scene: "yc_00",
    floors: &[yecheng::YECHENG_L1_MAP, yecheng::YECHENG_L2_MAP, yecheng::YECHENG_L3_MAP],
    floor_names: yecheng::YECHENG_FLOOR_NAMES,
    points: yecheng::POINTS,
    enemies: yecheng::ENEMIES,
    npcs: yecheng::NPCS,
    zones: yecheng::ZONES,
    portals: yecheng::PORTALS,
    gates: yecheng::GATES,
};

/// 全世界注册表（懒初始化；baisun 为程序化生成世界）。
/// 保持 `&'static [&'static WorldData]` 返回类型，与既有 `WORLDS.iter()` 遍历兼容。
pub fn all_worlds() -> &'static [&'static WorldData] {
    static SLICE: std::sync::OnceLock<&'static [&'static WorldData]> = std::sync::OnceLock::new();
    SLICE.get_or_init(|| {
        let list: Vec<&'static WorldData> = vec![
            biohazard_world(), &ZHUTIAN, &ZHOUYUAN, &MOSHI, &YINSE, &YIYING, &TIANSHE, &JIGUAN, &MORUIYA,
            &CANGJING, &JIANZHONG, &TONGQU, &JULUOJI, &XINGHE, &SISHEN, &MUMIYI, &MOJIAO, &WULIN,
            &TIANTING, &HEZI, &SHAQIU, &YIZE, &POXIAO, &TIEXUE,
            &TIEXUE2,
            &XINGJICHUANQI,
            &XINHUANGFANG,
            &HUANXIONGSHI,
            &MENGGUIJIE,
            &SIWUZHEN,
            &JINGJILING,
            &SHENMIAO,
            &SHUANGBAI,
            &DASHENGTANG,
            &DALIEXI,
            &POXU,
            &HANGU,
            &PANBU,
            &DIWEIDU,
            &SANLIAN,
            &WUJIN,
            &YIZHONG,
            &JISHENGQIANYE,
            &MIWU,
            &XINGCHEN,
            &YINXIANG,
            &NUOYA,
            &LANSHAN,
            &SHOURONGSUO,
            &TIANWANG,
            &XINGJIJIANCHUAN,
            // —— 第六批 6 副本（P8）——
            &XINGJICHUANQI2,
            &JIALEBI,
            &SHENGHUA3,
            &JISHUJING,
            baisun_world(),
            &BIHAI,
            &YECHENG,
        ];
        Box::leak(list.into_boxed_slice())
    })
}

pub fn find_world(id: &str) -> Option<&'static WorldData> {
    all_worlds().iter().find(|w| w.id == id).copied()
}

/// 显式携带 world 的地图查询（设计 §5.1 硬约束：杜绝 st.floor 裸查询）
pub fn tile(w: &WorldData, floor: usize, x: usize, y: usize) -> char {
    let map = w.floors.get(floor).copied().unwrap_or(&w.floors[w.floors.len() - 1]);
    if y >= map.len() || x >= (*map)[y].len() {
        return '#';
    }
    (*map)[y].as_bytes()[x] as char
}

pub fn walkable(w: &WorldData, floor: usize, x: usize, y: usize) -> bool {
    tile(w, floor, x, y) != '#'
}
