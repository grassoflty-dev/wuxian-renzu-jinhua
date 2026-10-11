//! 战斗体系数据层（包 A）：组合联动（Synergy）规则表 + 检测 helper。
//! 背景：把「血统 × 武器 × 法宝 × 技能」的特定暗合组合从纯堆数值升级为「构筑感」——
//! 命中联动规则时，在玩家攻击段由 engine.rs 的 synergy_check 调用追加效果并记录特殊样式日志。
//! 全部联动规则**幂等、确定性**：只读表 + 当前装配，不引入新的随机字段/存档字段。
//! id 一律取自 combat_data::BLOODLINES / items_data::{WEAPONS, TRESURE_DEFS, SKILLS} 真实 id。
//! 仅供数据与检测，结算副作用由 engine 包 B 按 result 应用。

use crate::defs::Equipment;
use crate::state::GameState;

// ============================================================================
// 联动效果类型（本轮实现攻击段可确定性追加的：固定伤害 / 吸血 / 二者兼有）
// ============================================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SynergyEffect {
    /// 追加固定真实伤害（在装备倍率/弱点乘算之前、护甲扣减之后由 engine 统一折入 base）
    FlatDmg(i32),
    /// 追加吸血量（命中时回复 hp，封顶 100）
    Leech(i32),
    /// 追加伤害 + 吸血
    FlatDmgLeech(i32, i32),
}

impl SynergyEffect {
    pub fn flat(&self) -> i32 {
        match self {
            SynergyEffect::FlatDmg(n) => *n,
            SynergyEffect::Leech(_) => 0,
            SynergyEffect::FlatDmgLeech(n, _) => *n,
        }
    }
    pub fn leech(&self) -> i32 {
        match self {
            SynergyEffect::FlatDmg(_) => 0,
            SynergyEffect::Leech(n) => *n,
            SynergyEffect::FlatDmgLeech(_, m) => *m,
        }
    }
}

// ============================================================================
// 联动规则条目
// ============================================================================
#[derive(Clone, Copy, Debug)]
pub struct SynergyDef {
    /// 规则唯一 id
    pub id: &'static str,
    /// 血统要求；None = 不限血统
    pub bloodline: Option<&'static str>,
    /// 主手武器 id 要求（equipment.weapon）；None = 不限
    pub weapon: Option<&'static str>,
    /// 法宝 id 要求（equipment.treasure 三格任一命中）；None = 不限
    pub treasure: Option<&'static str>,
    /// 已购技能 id 要求（st.skills）；None = 不限
    pub skill: Option<&'static str>,
    /// 修真境界门槛（st.cultivation_stage）；0 = 不限
    pub cultivation_min: u8,
    /// 触发效果
    pub effect: SynergyEffect,
    /// 提示文案（原名「提示文案」，engine 按此 + 数值拼装 synergy 样式日志）
    pub msg: &'static str,
}

// ============================================================================
// SYNERGIES · 联动规则表（10 条）
// 血统 id 一律真实存在（combat_data::BLOODLINES）；
// 武器/法宝 id 一律真实存在（items_data::{WEAPONS, TRESURE_DEFS}）。
// ============================================================================
pub const SYNERGIES: &[SynergyDef] = &[
    // 1. 龙珠共鸣：赛亚人在手，七龙珠随血脉共振。
    SynergyDef {
        id: "dragon_ball_echo",
        bloodline: Some("saiyan_bloodline"),
        weapon: None,
        treasure: Some("tr_longzu_shengyi"),
        skill: None,
        cultivation_min: 0,
        effect: SynergyEffect::FlatDmg(10),
        msg: "龙珠共鸣",
    },
    // 2. 写轮·破绽看破：宇智波之瞳洞悉牛头虚刃的破绽。
    SynergyDef {
        id: "sharingan_broken",
        bloodline: Some("uchiha_bloodline"),
        weapon: Some("wp_niutou_ren"),
        treasure: None,
        skill: None,
        cultivation_min: 0,
        effect: SynergyEffect::FlatDmg(12),
        msg: "写轮·破绽看破",
    },
    // 3. 死神·卍解斩魄：灵压注入斩魄刀，卍解沸腾。
    SynergyDef {
        id: "shinigami_zanjing",
        bloodline: Some("shinigami_bloodline"),
        weapon: Some("wp_zanjingdao_he"),
        treasure: None,
        skill: None,
        cultivation_min: 0,
        effect: SynergyEffect::FlatDmg(14),
        msg: "死神·卍解斩魄",
    },
    // 4. 御剑·本命青锋：修真者以本命飞剑御剑追袭（需入修真一境以上）。
    SynergyDef {
        id: "yujian_benming",
        bloodline: None,
        weapon: None,
        treasure: Some("cu_bab_benming_fejian"),
        skill: None,
        cultivation_min: 1,
        effect: SynergyEffect::FlatDmgLeech(8, 4),
        msg: "御剑·本命青锋",
    },
    // 5. 血族渴宴：吸血鬼握血戮剑，渴宴彻底展开。
    SynergyDef {
        id: "vampire_feast",
        bloodline: Some("vampire"),
        weapon: Some("wpn_bloodsaber"),
        treasure: None,
        skill: None,
        cultivation_min: 0,
        effect: SynergyEffect::FlatDmgLeech(6, 4),
        msg: "血族渴宴",
    },
    // 6. 龙裔·屠龙之心：龙之血性呼应屠龙之枪与八荒龙印。
    SynergyDef {
        id: "dragon_kill",
        bloodline: Some("dragon_bloodline"),
        weapon: Some("wp_dragon_lance"),
        treasure: Some("tr_bahuang_longyin"),
        skill: None,
        cultivation_min: 0,
        effect: SynergyEffect::FlatDmg(16),
        msg: "龙裔·屠龙之心",
    },
    // 7. 灭却·寒晶圣矢：灭却师之血令寒冰精灵长弓凝出圣矢。
    SynergyDef {
        id: "quincy_arrow",
        bloodline: Some("quincy_bloodline"),
        weapon: Some("wp_hanbing_gong"),
        treasure: None,
        skill: None,
        cultivation_min: 0,
        effect: SynergyEffect::FlatDmg(10),
        msg: "灭却·寒晶圣矢",
    },
    // 8. 恶魔契约·至尊戒：至尊戒响应恶魔血统的贪婪。
    SynergyDef {
        id: "demon_ring",
        bloodline: Some("demon_bloodline"),
        weapon: None,
        treasure: Some("tr_mo_jie_jiujie"),
        skill: None,
        cultivation_min: 0,
        effect: SynergyEffect::FlatDmg(12),
        msg: "恶魔契约·至尊戒",
    },
    // 9. 虚化·牛头之怒：虚化血统撕开牛头虚刃的疯狂。
    SynergyDef {
        id: "hollow_niu",
        bloodline: Some("hollow_bloodline"),
        weapon: Some("wp_niutou_ren"),
        treasure: None,
        skill: None,
        cultivation_min: 0,
        effect: SynergyEffect::FlatDmg(14),
        msg: "虚化·牛头之怒",
    },
    // 10. 大筒木·北冥吸星：大筒木之血驱动北冥神功·吸星剑。
    SynergyDef {
        id: "otsutsuki_bei",
        bloodline: Some("otsutsuki_bloodline"),
        weapon: Some("wp_beiming_jian"),
        treasure: None,
        skill: None,
        cultivation_min: 0,
        effect: SynergyEffect::FlatDmgLeech(10, 6),
        msg: "大筒木·北冥吸星",
    },
];

// ============================================================================
// 查询 helper（纯读表，供 engine 包 B 在玩家攻击段调用）
// ============================================================================

fn has_weapon(eq: &Equipment, id: &str) -> bool {
    eq.weapon.as_ref().map(|w| w.id.as_str() == id).unwrap_or(false)
}
fn has_treasure(eq: &Equipment, id: &str) -> bool {
    eq.treasure.iter().flatten().any(|t| t.as_str() == id)
}

/// 单条规则是否命中当前装配（确定性：纯读表 + 当前状态）
pub fn synergy_matches(st: &GameState, def: &SynergyDef) -> bool {
    if let Some(bl) = def.bloodline {
        if st.bloodline.as_deref() != Some(bl) {
            return false;
        }
    }
    if let Some(w) = def.weapon {
        if !has_weapon(&st.equipment, w) {
            return false;
        }
    }
    if let Some(t) = def.treasure {
        if !has_treasure(&st.equipment, t) {
            return false;
        }
    }
    if let Some(sk) = def.skill {
        if !st.skills.iter().any(|s| s == sk) {
            return false;
        }
    }
    if def.cultivation_min > 0 && st.cultivation_stage < def.cultivation_min {
        return false;
    }
    true
}

/// 命中检测（engine 在玩家攻击段调用）：返回当前命中的全部联动规则。
pub fn synergy_check(st: &GameState) -> Vec<&'static SynergyDef> {
    SYNERGIES.iter().filter(|d| synergy_matches(st, d)).collect()
}

/// 命中规则的追加伤害总和（纯读表，确定性）
pub fn synergy_flat(st: &GameState) -> i32 {
    synergy_check(st).iter().map(|d| d.effect.flat()).sum()
}

/// 命中规则的吸血量总和（纯读表，确定性）
pub fn synergy_leech(st: &GameState) -> i32 {
    synergy_check(st).iter().map(|d| d.effect.leech()).sum()
}

/// 把单条命中规则的一行 synergy 样式日志拼装出来（engine 逐条 push_log）。
/// 返回 (flat, leech, 文案) 三元组，便于 engine 同时应用数值并写日志。
pub fn synergy_line(
    def: &SynergyDef,
) -> (i32, i32, String) {
    let flat = def.effect.flat();
    let leech = def.effect.leech();
    let detail = match (flat > 0, leech > 0) {
        (true, true) => format!("追加伤害 +{flat} · 吸血 +{leech}"),
        (true, false) => format!("追加伤害 +{flat}"),
        (false, true) => format!("吸血 +{leech}"),
        (false, false) => "联动触发".to_string(),
    };
    (
        flat,
        leech,
        format!("<span class='synergy'>✦ 组合联动「{}」——{}</span>", def.msg, detail),
    )
}
