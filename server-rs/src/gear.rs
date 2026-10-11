//! 护甲/饰品旁附表（item §1.2；slot=Armor|Accessory）
//! 由 items_data 拆分而来，仅数据铺装与 gear_def 查询 helper。

use crate::defs::{GearDef, GearSlot};

pub const GEAR: &[GearDef] = &[
    GearDef { id: "gear_police_vest", name: "警用防弹背心", slot: GearSlot::Armor, dmg_reduce: 4, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 900, need_grade: Some('D') },
    GearDef { id: "gear_kevlar", name: "凯夫拉防弹衣", slot: GearSlot::Armor, dmg_reduce: 7, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 2200, need_grade: Some('C') },
    GearDef { id: "gear_elven_cloak", name: "精灵斗篷", slot: GearSlot::Armor, dmg_reduce: 5, atk_flat: 0, dodge: 0.05, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 2800, need_grade: Some('C') },
    GearDef { id: "gear_mithril_vault", name: "秘银护甲", slot: GearSlot::Armor, dmg_reduce: 10, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 6000, need_grade: Some('B') },
    GearDef { id: "gear_nano_vest", name: "纳米作战服", slot: GearSlot::Armor, dmg_reduce: 12, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 9000, need_grade: Some('A') },
    GearDef { id: "access_strength_ring", name: "蛮力指环", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 6, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 1200, need_grade: Some('D') },
    GearDef { id: "access_agility_boots", name: "追风靴", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 0, dodge: 0.06, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 1300, need_grade: Some('D') },
    GearDef { id: "access_san_locket", name: "安魂吊坠", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 6, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 1600, need_grade: Some('C') },
    GearDef { id: "access_qi_belt", name: "聚气腰带", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 30, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 1800, need_grade: Some('C') },
    // ---- 增量扩充（第 2 批 · 高 tier 减伤/闪避/SAN 抗/qi_max/tech_shield）----
    GearDef { id: "gear_adamant_cuirass", name: "精金胸甲", slot: GearSlot::Armor, dmg_reduce: 15, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 20, per_turn_qi: 0, dmg_mult: 1.0, price: 14000, need_grade: Some('A') },
    GearDef { id: "gear_void_leak", name: "虚无织物衣", slot: GearSlot::Armor, dmg_reduce: 9, atk_flat: 0, dodge: 0.08, san_resist: 4, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 11000, need_grade: Some('A') },
    GearDef { id: "gear_zero_absorb", name: "绝对零度护甲", slot: GearSlot::Armor, dmg_reduce: 18, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 26000, need_grade: Some('S') },
    GearDef { id: "gear_sanctum_plate", name: "圣域板甲", slot: GearSlot::Armor, dmg_reduce: 12, atk_flat: 0, dodge: 0.0, san_resist: 10, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 15000, need_grade: Some('A') },
    GearDef { id: "access_hades_cloak", name: "幽冥披风", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 0, dodge: 0.12, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 8800, need_grade: Some('B') },
    GearDef { id: "access_will_anchor", name: "意志锚链", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 14, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 9600, need_grade: Some('B') },
    GearDef { id: "access_tianting_belt", name: "天庭灵气腰带", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 80, hp_max: 0, per_turn_qi: 4, dmg_mult: 1.0, price: 18000, need_grade: Some('A') },
    GearDef { id: "access_nano_tech_shield", name: "纳米护盾核心", slot: GearSlot::Accessory, dmg_reduce: 6, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 12000, need_grade: Some('A') },
    // ==================== 增量扩充（第 3 批 · +30 护甲/饰品 · 各风格） ====================
    // ---- 战甲 / 圣衣 / 机甲（Armor）----
    GearDef { id: "gear_shengclothes_shooter", name: "射手座黄金圣衣", slot: GearSlot::Armor, dmg_reduce: 20, atk_flat: 8, dodge: 0.05, san_resist: 6, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 30000, need_grade: Some('S') },
    GearDef { id: "gear_nano_mecha_suit", name: "纳米战甲·机甲", slot: GearSlot::Armor, dmg_reduce: 16, atk_flat: 6, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 20, per_turn_qi: 0, dmg_mult: 1.0, price: 20000, need_grade: Some('A') },
    GearDef { id: "gear_leidun_armor", name: "雷霆铠甲", slot: GearSlot::Armor, dmg_reduce: 14, atk_flat: 4, dodge: 0.0, san_resist: 0, qi_max: 20, hp_max: 0, per_turn_qi: 3, dmg_mult: 1.0, price: 17000, need_grade: Some('A') },
    GearDef { id: "gear_longlin_jia", name: "龙鳞逆甲", slot: GearSlot::Armor, dmg_reduce: 22, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 30, per_turn_qi: 0, dmg_mult: 1.0, price: 32000, need_grade: Some('S') },
    GearDef { id: "gear_shengguang_fapao", name: "圣光法袍", slot: GearSlot::Armor, dmg_reduce: 10, atk_flat: 0, dodge: 0.0, san_resist: 18, qi_max: 0, hp_max: 0, per_turn_qi: 4, dmg_mult: 1.0, price: 16000, need_grade: Some('A') },
    GearDef { id: "gear_tian_yi", name: "神炁天衣", slot: GearSlot::Armor, dmg_reduce: 12, atk_flat: 6, dodge: 0.08, san_resist: 8, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 18000, need_grade: Some('A') },
    GearDef { id: "gear_azote_panzhi", name: "奥术织纹布甲", slot: GearSlot::Armor, dmg_reduce: 8, atk_flat: 8, dodge: 0.0, san_resist: 5, qi_max: 0, hp_max: 0, per_turn_qi: 4, dmg_mult: 1.0, price: 13000, need_grade: Some('B') },
    GearDef { id: "gear_wh_warframe", name: "战争框架·重装", slot: GearSlot::Armor, dmg_reduce: 19, atk_flat: 10, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 28000, need_grade: Some('A') },
    GearDef { id: "gear_ice_dragon_scale", name: "冰霜巨龙鳞甲", slot: GearSlot::Armor, dmg_reduce: 17, atk_flat: 0, dodge: 0.0, san_resist: 5, qi_max: 0, hp_max: 25, per_turn_qi: 0, dmg_mult: 1.0, price: 24000, need_grade: Some('A') },
    GearDef { id: "gear_shadow_cloak_armor", name: "暗影皮甲", slot: GearSlot::Armor, dmg_reduce: 11, atk_flat: 4, dodge: 0.10, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 15000, need_grade: Some('B') },
    // ---- 饰品 / 护符（Accessory）----
    GearDef { id: "access_divine_ring", name: "神圣婚戒", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 12, dodge: 0.0, san_resist: 4, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.1, price: 14000, need_grade: Some('A') },
    GearDef { id: "access_frost_amulet", name: "冰封护符", slot: GearSlot::Accessory, dmg_reduce: 4, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 9000, need_grade: Some('B') },
    GearDef { id: "access_lightning_core", name: "雷电核心吊坠", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 6, dodge: 0.0, san_resist: 0, qi_max: 40, hp_max: 0, per_turn_qi: 3, dmg_mult: 1.0, price: 10000, need_grade: Some('B') },
    GearDef { id: "access_soul_bind", name: "魂之锁结", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 12, qi_max: 0, hp_max: 20, per_turn_qi: 0, dmg_mult: 1.0, price: 11000, need_grade: Some('B') },
    GearDef { id: "access_dragon_seal", name: "龙纹玉玺", slot: GearSlot::Accessory, dmg_reduce: 5, atk_flat: 8, dodge: 0.0, san_resist: 4, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 16000, need_grade: Some('A') },
    GearDef { id: "access_saint_bracelet", name: "圣斗士手环", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 10, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 12000, need_grade: Some('B') },
    GearDef { id: "access_truth_seeker", name: "求真透镜·感知", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 0, dodge: 0.06, san_resist: 6, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 9800, need_grade: Some('B') },
    GearDef { id: "access_tianxuan_jing", name: "先天玄光镜", slot: GearSlot::Accessory, dmg_reduce: 6, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 60, hp_max: 0, per_turn_qi: 5, dmg_mult: 1.0, price: 20000, need_grade: Some('A') },
    GearDef { id: "access_wuxin_shaer", name: "无心神砂·定魂", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 16, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 13000, need_grade: Some('B') },
    GearDef { id: "access_devil_contract", name: "恶魔契约徽记", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 14, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.15, price: 18000, need_grade: Some('A') },
    // ---- 中低 tier 补（Armor / Accessory 混合）----
    GearDef { id: "gear_iron_warplate", name: "玄铁重甲", slot: GearSlot::Armor, dmg_reduce: 9, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 5200, need_grade: Some('C') },
    GearDef { id: "gear_silk_robe", name: "唐锦法衣", slot: GearSlot::Armor, dmg_reduce: 5, atk_flat: 4, dodge: 0.04, san_resist: 3, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 3600, need_grade: Some('C') },
    GearDef { id: "gear_black_tech_suit", name: "黑色科技紧身衣", slot: GearSlot::Armor, dmg_reduce: 8, atk_flat: 6, dodge: 0.06, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 7000, need_grade: Some('B') },
    GearDef { id: "gear_holy_plate_armor", name: "圣骑士板甲", slot: GearSlot::Armor, dmg_reduce: 12, atk_flat: 4, dodge: 0.0, san_resist: 6, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 9800, need_grade: Some('B') },
    GearDef { id: "access_silver_cross", name: "秘银圣十字", slot: GearSlot::Accessory, dmg_reduce: 3, atk_flat: 0, dodge: 0.0, san_resist: 8, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 6800, need_grade: Some('C') },
    GearDef { id: "access_moon_pendant", name: "月华坠饰", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 4, dodge: 0.05, san_resist: 4, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 6200, need_grade: Some('C') },
    GearDef { id: "access_qi_obsidian", name: "聚灵黑曜石", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 50, hp_max: 0, per_turn_qi: 2, dmg_mult: 1.0, price: 7600, need_grade: Some('B') },
    GearDef { id: "access_werewolf_claw", name: "狼王獠牙挂坠", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 8, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 8000, need_grade: Some('B') },
    GearDef { id: "access_ice_heart", name: "冰晶之心", slot: GearSlot::Accessory, dmg_reduce: 2, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 10, per_turn_qi: 0, dmg_mult: 1.0, price: 7000, need_grade: Some('B') },
    GearDef { id: "access_phoenix_feather", name: "涅槃凤羽", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 6, dodge: 0.0, san_resist: 4, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.1, price: 15000, need_grade: Some('A') },
    // ---- 增量扩充（第 4 批 · 数据子代理追加 2 甲 1 饰）----
    GearDef { id: "gear_wolf_skin", name: "狼皮重裘", slot: GearSlot::Armor, dmg_reduce: 8, atk_flat: 2, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 10, per_turn_qi: 0, dmg_mult: 1.0, price: 5800, need_grade: Some('C') },
    GearDef { id: "gear_star_robe", name: "星辉法袍", slot: GearSlot::Armor, dmg_reduce: 6, atk_flat: 4, dodge: 0.04, san_resist: 6, qi_max: 20, hp_max: 0, per_turn_qi: 2, dmg_mult: 1.0, price: 11800, need_grade: Some('B') },
    GearDef { id: "access_blood_charm", name: "血月护符", slot: GearSlot::Accessory, dmg_reduce: 0, atk_flat: 5, dodge: 0.0, san_resist: 4, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, price: 5600, need_grade: Some('C') },
];

/// 按装备 id 查表（护甲/饰品）
pub fn gear_def(id: &str) -> Option<&'static GearDef> {
    GEAR.iter().find(|g| g.id == id)
}
