//! 道具表（item §1.3/1.4/1.5；战斗/兑换/门禁/合成查此）+ 合成表（item §4.3 ①）+ 计数 helper。
//! 由 items_data 拆分而来；inventory 计数用方案 A 拆 id（零侵入 Vec<String>）。

use crate::defs::{ItemDef, ItemEffect, ItemSub, Recipe};
use crate::state::GameState;

pub const ITEMS: &[ItemDef] = &[
    // ---- 消耗品（可堆叠）----
    ItemDef { id: "item_medkit", name: "强效医疗包", kind: ItemSub::Heal, stack: true, usable_in_fight: true, effect: ItemEffect::Heal(50), price: 700, need_grade: None },
    ItemDef { id: "item_bandage", name: "紧急绷带", kind: ItemSub::Heal, stack: true, usable_in_fight: true, effect: ItemEffect::Heal(18), price: 220, need_grade: None },
    ItemDef { id: "item_sedative", name: "镇静剂", kind: ItemSub::San, stack: true, usable_in_fight: true, effect: ItemEffect::San(20), price: 320, need_grade: None },
    ItemDef { id: "item_bottle_water", name: "口袋圣水", kind: ItemSub::Heal, stack: true, usable_in_fight: true, effect: ItemEffect::Heal(8), price: 180, need_grade: None },
    ItemDef { id: "item_holy_water", name: "圣水", kind: ItemSub::Throw, stack: true, usable_in_fight: true, effect: ItemEffect::Throw { dmg_over_time: None, weak: Some("holy"), flat_dmg: 0 }, price: 400, need_grade: None },
    ItemDef { id: "item_silver_bullet", name: "银弹", kind: ItemSub::Throw, stack: true, usable_in_fight: true, effect: ItemEffect::Throw { dmg_over_time: None, weak: Some("silver"), flat_dmg: 0 }, price: 520, need_grade: None },
    ItemDef { id: "item_torch", name: "火把", kind: ItemSub::Throw, stack: true, usable_in_fight: true, effect: ItemEffect::Throw { dmg_over_time: Some((0, 3)), weak: Some("fire"), flat_dmg: 0 }, price: 260, need_grade: None },
    ItemDef { id: "item_lure", name: "诱水剂", kind: ItemSub::Throw, stack: true, usable_in_fight: true, effect: ItemEffect::Throw { dmg_over_time: None, weak: None, flat_dmg: 60 }, price: 480, need_grade: None },
    ItemDef { id: "item_grenade", name: "燃烧手雷", kind: ItemSub::Throw, stack: true, usable_in_fight: true, effect: ItemEffect::Throw { dmg_over_time: None, weak: None, flat_dmg: 70 }, price: 900, need_grade: None },
    ItemDef { id: "item_quzhen_fu", name: "驱邪符", kind: ItemSub::CharmProt, stack: true, usable_in_fight: true, effect: ItemEffect::Charm { immune_death: true, cure_debuff: false }, price: 1500, need_grade: None },
    ItemDef { id: "item_jiezhou_fu", name: "解咒符", kind: ItemSub::CharmAnti, stack: true, usable_in_fight: true, effect: ItemEffect::Charm { immune_death: false, cure_debuff: true }, price: 600, need_grade: None },
    ItemDef { id: "item_antidote", name: "净化血清", kind: ItemSub::Heal, stack: true, usable_in_fight: true, effect: ItemEffect::Charm { immune_death: false, cure_debuff: true }, price: 420, need_grade: None },
    ItemDef { id: "it_qixue_dan", name: "气血丹", kind: ItemSub::Heal, stack: true, usable_in_fight: false, effect: ItemEffect::Heal(80), price: 480, need_grade: None },
    ItemDef { id: "ammo_crate", name: "弹药盒", kind: ItemSub::Ammo, stack: false, usable_in_fight: false, effect: ItemEffect::Ammo, price: 150, need_grade: None },
    ItemDef { id: "gj_grenade", name: "军用手雷", kind: ItemSub::Throw, stack: true, usable_in_fight: false, effect: ItemEffect::Throw { dmg_over_time: None, weak: None, flat_dmg: 50 }, price: 200, need_grade: None },
    ItemDef { id: "item_anesthetic", name: "麻醉剂", kind: ItemSub::Throw, stack: true, usable_in_fight: false, effect: ItemEffect::None, price: 200, need_grade: None },
    // ---- 强化石 / 新材料（战斗强化消耗品 + 合成原料）----
    ItemDef { id: "it_enhance_stone", name: "普通强化石", kind: ItemSub::Heal, stack: true, usable_in_fight: false, effect: ItemEffect::None, price: 1500, need_grade: None },
    ItemDef { id: "it_enhance_stone_hi", name: "高级强化石", kind: ItemSub::Heal, stack: true, usable_in_fight: false, effect: ItemEffect::None, price: 6000, need_grade: Some('B') },
    ItemDef { id: "it_em_core", name: "电磁炮核心", kind: ItemSub::Reliquary, stack: false, usable_in_fight: false, effect: ItemEffect::None, price: 5000, need_grade: Some('B') },
    ItemDef { id: "it_blood_essence", name: "血族精血", kind: ItemSub::Reliquary, stack: false, usable_in_fight: false, effect: ItemEffect::None, price: 4200, need_grade: Some('B') },
    ItemDef { id: "it_treasure_frag", name: "法宝碎片", kind: ItemSub::Reliquary, stack: false, usable_in_fight: false, effect: ItemEffect::None, price: 2600, need_grade: Some('B') },
    ItemDef { id: "it_core_crystal", name: "核心晶石", kind: ItemSub::Reliquary, stack: true, usable_in_fight: false, effect: ItemEffect::None, price: 2400, need_grade: Some('C') },
    // ---- 圣物凭证：防御型（复用现 engine 掉落 id + 自创兑换券）----
    ItemDef { id: "it_soul_shard", name: "灵魂碎片", kind: ItemSub::Reliquary, stack: true, usable_in_fight: false, effect: ItemEffect::None, price: 500, need_grade: Some('D') },
    ItemDef { id: "it_genome_alpha", name: "基因样本·α", kind: ItemSub::Reliquary, stack: false, usable_in_fight: false, effect: ItemEffect::None, price: 2000, need_grade: Some('C') },
    ItemDef { id: "it_core_sample", name: "能量核心残片", kind: ItemSub::Reliquary, stack: false, usable_in_fight: false, effect: ItemEffect::None, price: 1800, need_grade: Some('C') },
    ItemDef { id: "it_cross_key", name: "圣徽钥匙", kind: ItemSub::Reliquary, stack: false, usable_in_fight: false, effect: ItemEffect::None, price: 900, need_grade: Some('D') },
    ItemDef { id: "it_cross", name: "圣徽", kind: ItemSub::Reliquary, stack: false, usable_in_fight: false, effect: ItemEffect::None, price: 1200, need_grade: Some('D') },
    ItemDef { id: "it_secret_key", name: "秘匣钥匙", kind: ItemSub::Reliquary, stack: false, usable_in_fight: false, effect: ItemEffect::None, price: 700, need_grade: Some('D') },
    ItemDef { id: "it_box_mi", name: "密匣信物", kind: ItemSub::Quest, stack: false, usable_in_fight: false, effect: ItemEffect::None, price: 300, need_grade: Some('D') },
    // ---- 任务/剧情（纯 flag 载体，不占战斗数值）----
    ItemDef { id: "it_vault_pass", name: "宝库通行证", kind: ItemSub::Quest, stack: false, usable_in_fight: false, effect: ItemEffect::None, price: 0, need_grade: None },
    // ---- 增量扩充（第 4 批 · 数据子代理追加实用消耗品）----
    ItemDef { id: "item_stim_shot", name: "战术兴奋剂", kind: ItemSub::Heal, stack: true, usable_in_fight: true, effect: ItemEffect::Heal(20), price: 300, need_grade: None },
    ItemDef { id: "item_sanitizer", name: "净化喷雾", kind: ItemSub::CharmAnti, stack: true, usable_in_fight: true, effect: ItemEffect::Charm { immune_death: false, cure_debuff: true }, price: 500, need_grade: None },
    ItemDef { id: "item_rope", name: "攀爬绳索", kind: ItemSub::Quest, stack: true, usable_in_fight: false, effect: ItemEffect::None, price: 150, need_grade: None },
    ItemDef { id: "item_flare", name: "信号弹", kind: ItemSub::Throw, stack: true, usable_in_fight: true, effect: ItemEffect::Throw { dmg_over_time: None, weak: None, flat_dmg: 25 }, price: 350, need_grade: None },
];

/// 复用现有 engine 真实掉落的门禁/剧情道具（不一一列入 ITEMS 计量表，属非堆叠剧情载体）
pub const QUEST_ITEM_IDS: &[&str] = &[
    "adrenaline", "firstaid", "lab_badge", "yiy_key_med", "yiy_pulse", "yiy_em_restraint",
    "yiy_embryo_sample", "item_ghost_key", "item_toshio_key", "item_buddha", "item_diary",
    "item_cat_food", "item_chushe_sample", "item_chushe_blood", "item_core_crystal",
    "item_gene_card", "item_seal_pass", "item_diling", "item_lou_bone", "item_jiche",
    "item_key", "beast_core", "mithril_key", "mithril_block", "elven_cloak", "it_gear_key",
    "it_gear_token_a", "it_gear_token_b", "it_gear_token_c", "it_pivot_key", "it_mo_ling_a",
    "item_record_1", "item_letter", "corpse_key", "data_chip", "beam_core", "item_hope_light",
    "item_shoucheng_letter",
];

pub const RECIPES: &[Recipe] = &[
    Recipe { result: "it_core_crystal", ingredients: &["it_soul_shard", "it_core_sample"] },
    Recipe { result: "it_cross", ingredients: &["it_cross_key", "item_holy_water"] },
    // ---- 增量扩充（第 2 批 · 新材料合成）----
    Recipe { result: "it_em_core", ingredients: &["it_core_crystal", "beam_core"] },
    Recipe { result: "it_blood_essence", ingredients: &["item_chushe_blood", "it_soul_shard"] },
    Recipe { result: "it_treasure_frag", ingredients: &["it_core_sample", "it_soul_shard"] },
    Recipe { result: "tr_blood_banner", ingredients: &["it_treasure_frag", "it_blood_essence"] },
    Recipe { result: "it_enhance_stone", ingredients: &["it_core_sample", "it_soul_shard"] },
    Recipe { result: "it_enhance_stone_hi", ingredients: &["it_enhance_stone", "it_em_core"] },
    // ---- 增量扩充（第 4 批 · 数据子代理追加配方）----
    Recipe { result: "item_sanitizer", ingredients: &["item_bottle_water", "item_sedative"] },
    Recipe { result: "item_flare", ingredients: &["item_torch", "it_core_crystal"] },
];

/// 按道具 id 查表
pub fn item_def(id: &str) -> Option<&'static ItemDef> {
    ITEMS.iter().find(|it| it.id == id)
}

// ---------- inventory 计数（方案 A：同 id 多枚拆 `id_k`，零侵入 Vec<String>） ----------

/// 是否为某 base 的拆 id 计数组件：`base` 本身 或 `base_<数字>`（方案 A 拆 id）。
/// 仅需数字后缀，避免与「真名含 _ 前缀的独立 id」相撞（如 it_enhance_stone vs it_enhance_stone_hi）。
fn matches_base(entry: &str, base: &str) -> bool {
    if entry == base {
        return true;
    }
    let Some(rest) = entry.strip_prefix(&format!("{base}_")) else { return false; };
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
}

/// 统计 base id 持有量（命中 base 本身或 base_N 前缀）
pub fn count_item(st: &GameState, base: &str) -> usize {
    st.inventory.iter().filter(|i| matches_base(i, base)).count()
}

/// 计数别名（保持可读性）
pub fn count_items(st: &GameState, base: &str) -> usize {
    count_item(st, base)
}

/// 是否有该道具（base 或 base_N 前缀命中）
pub fn has_item(st: &GameState, base: &str) -> bool {
    count_item(st, base) > 0
}

/// 无堆叠 / 剧情 / 圣物唯一物：去重唯一保留现 add_item 语义；调用方决定走哪个入口。
/// 追加计数到位：可堆叠消耗品 → push `{base}_{count}`；唯一物 → 去重 push base。
/// 若可堆叠则累加（用 item_def.max_stack 判定，缺表条目按可堆叠处理）。
pub fn add_item_counted(st: &mut GameState, item: &str) {
    let stackable = item_def(item).map(|d| d.stack).unwrap_or(true);
    if !stackable {
        if !st.inventory.iter().any(|i| i == item) {
            st.inventory.push(item.to_string());
        }
        return;
    }
    let k = count_item(st, item) + 1;
    st.inventory.push(format!("{item}_{k}"));
}

/// 消耗一份 base 道具（尾部移除一个 base 或 base_N 组件）；返回是否成功
pub fn consume_item(st: &mut GameState, base: &str) -> bool {
    let pos = st.inventory.iter().rposition(|i| matches_base(i, base));
    match pos {
        Some(p) => {
            st.inventory.remove(p);
            true
        }
        None => false,
    }
}
