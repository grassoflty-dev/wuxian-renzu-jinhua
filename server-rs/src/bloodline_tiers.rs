//! 血统进阶路线（血统阶位 + 熟练度强度）数据层。
//!
//! 数值方案（已定，照此实现）：
//! - 每个血统一条「进阶路线」，每阶 `{ tier, name, base, k, need_points, need_flag }`。
//! - 主基调 ×10 递增：阶1 初级 base=100 k=+1；阶2 中级 base=1000 k=+2 need_points=5000；
//!   阶3 高级 base=10000 k=+4 need_points=50000。
//! - 个别 S 级血统（赛亚人/大筒木）base 上浮 1.5~2×，保持 ×10 递增主基调。
//!
//! 本轮只给 `vampire`（血族）与 `saiyan_bloodline`（赛亚人）补完整 3 阶示例；
//! 其余 17 个血统走「通用默认进阶线」（后续另拆子代理批量补各血统专属阶位数据）。
//!
//! 强度公式（见 combat_data::bloodline_strength）：
//!   strength = base + contribution
//!   contribution 每次「使用当前血统相关技能/攻击」+k（当前阶系数），进阶砍半、保底不下降。

use crate::state::GameState;

/// 血统单一阶位配置。数值「建议值·可调」；全静态表。
#[derive(Clone, Copy, Debug)]
pub struct BloodlineTierCfg {
    pub tier: u8,              // 阶位 1=初级 2=中级 3=高级（可扩 4）
    pub name: &'static str,    // 阶名（血统专属）
    pub base: i32,             // 该阶初始强度（不含熟练度贡献）
    pub k: i32,                // 熟练度系数：每次使用相关技能/攻击贡献 +k
    pub need_points: i64,      // 进阶所需点数（升到本阶的花费）
    pub need_flag: Option<&'static str>, // 支线剧情权限旗标（None=无剧情门槛）
}

/// 静态阶位表：key=(血统 id, tier)。仅 vampire / saiyan_bloodline 具名铺装，
/// 其余血统由 `default_tiers(id)` 运行时生成通用默认线（见下）。
pub const BLOODLINE_TIERS: &[(&'static str, BloodlineTierCfg)] = &[
    // ---------------- vampire · 血族（完整 3 阶示例） ----------------
    ("vampire", BloodlineTierCfg {
        tier: 1, name: "初级血族", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("vampire", BloodlineTierCfg {
        tier: 2, name: "中级血族", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("vamp_mid_unlock"),
    }),
    ("vampire", BloodlineTierCfg {
        tier: 3, name: "高级血族", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("vamp_high_unlock"),
    }),
    // ---------------- saiyan_bloodline · 赛亚人（S 级，base 上浮 1.5×）----
    ("saiyan_bloodline", BloodlineTierCfg {
        tier: 1, name: "初级赛亚人", base: 150, k: 1, need_points: 0, need_flag: None,
    }),
    ("saiyan_bloodline", BloodlineTierCfg {
        tier: 2, name: "中级赛亚人（超级赛亚人）", base: 1500, k: 2, need_points: 5000,
        need_flag: Some("saiyan_mid_unlock"),
    }),
    ("saiyan_bloodline", BloodlineTierCfg {
        tier: 3, name: "高级赛亚人（超级赛亚人2）", base: 15000, k: 4, need_points: 50000,
        need_flag: Some("saiyan_high_unlock"),
    }),
    // ---------------- werewolf · 狼人血统 ----------------
    ("werewolf", BloodlineTierCfg {
        tier: 1, name: "月狼", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("werewolf", BloodlineTierCfg {
        tier: 2, name: "狂暴狼人", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("werewolf_mid_unlock"),
    }),
    ("werewolf", BloodlineTierCfg {
        tier: 3, name: "狼王", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("werewolf_high_unlock"),
    }),
    // ---------------- zuwu · 祖巫血脉 ----------------
    ("zuwu", BloodlineTierCfg {
        tier: 1, name: "巫骨", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("zuwu", BloodlineTierCfg {
        tier: 2, name: "祖巫体魄", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("zuwu_mid_unlock"),
    }),
    ("zuwu", BloodlineTierCfg {
        tier: 3, name: "十二祖巫", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("zuwu_high_unlock"),
    }),
    // ---------------- zhanshi_blood · 圣光圣职者血脉 ----------------
    ("zhanshi_blood", BloodlineTierCfg {
        tier: 1, name: "圣光士", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("zhanshi_blood", BloodlineTierCfg {
        tier: 2, name: "圣殿骑士", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("zhanshi_blood_mid_unlock"),
    }),
    ("zhanshi_blood", BloodlineTierCfg {
        tier: 3, name: "圣光圣使", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("zhanshi_blood_high_unlock"),
    }),
    // ---------------- cyber_prosthetic · 机械义体血统 ----------------
    ("cyber_prosthetic", BloodlineTierCfg {
        tier: 1, name: "义体初段", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("cyber_prosthetic", BloodlineTierCfg {
        tier: 2, name: "神经义体", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("cyber_prosthetic_mid_unlock"),
    }),
    ("cyber_prosthetic", BloodlineTierCfg {
        tier: 3, name: "机械化核心", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("cyber_prosthetic_high_unlock"),
    }),
    // ---------------- gauss_cyber · 纳米血统·共振 ----------------
    ("gauss_cyber", BloodlineTierCfg {
        tier: 1, name: "纳米共鸣", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("gauss_cyber", BloodlineTierCfg {
        tier: 2, name: "共振核心", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("gauss_cyber_mid_unlock"),
    }),
    ("gauss_cyber", BloodlineTierCfg {
        tier: 3, name: "纳米主宰", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("gauss_cyber_high_unlock"),
    }),
    // ---------------- angel_bloodline · 天使血统 ----------------
    ("angel_bloodline", BloodlineTierCfg {
        tier: 1, name: "天使·圣翼", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("angel_bloodline", BloodlineTierCfg {
        tier: 2, name: "炽天使", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("angel_bloodline_mid_unlock"),
    }),
    ("angel_bloodline", BloodlineTierCfg {
        tier: 3, name: "大天使", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("angel_bloodline_high_unlock"),
    }),
    // ---------------- demon_bloodline · 恶魔血统（S 级，base 上浮 1.5×）----
    ("demon_bloodline", BloodlineTierCfg {
        tier: 1, name: "恶魔·低阶契约", base: 150, k: 1, need_points: 0, need_flag: None,
    }),
    ("demon_bloodline", BloodlineTierCfg {
        tier: 2, name: "高阶恶魔", base: 1500, k: 2, need_points: 5000,
        need_flag: Some("demon_bloodline_mid_unlock"),
    }),
    ("demon_bloodline", BloodlineTierCfg {
        tier: 3, name: "深渊魔王", base: 15000, k: 4, need_points: 50000,
        need_flag: Some("demon_bloodline_high_unlock"),
    }),
    // ---------------- dragon_bloodline · 龙族血统（S 级，base 上浮 1.5×）----
    ("dragon_bloodline", BloodlineTierCfg {
        tier: 1, name: "龙血觉醒", base: 150, k: 1, need_points: 0, need_flag: None,
    }),
    ("dragon_bloodline", BloodlineTierCfg {
        tier: 2, name: "龙魂钢鳞", base: 1500, k: 2, need_points: 5000,
        need_flag: Some("dragon_bloodline_mid_unlock"),
    }),
    ("dragon_bloodline", BloodlineTierCfg {
        tier: 3, name: "龙皇降世", base: 15000, k: 4, need_points: 50000,
        need_flag: Some("dragon_bloodline_high_unlock"),
    }),
    // ---------------- sharingan_bloodline · 写轮眼血统 ----------------
    ("sharingan_bloodline", BloodlineTierCfg {
        tier: 1, name: "写轮眼·单勾玉", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("sharingan_bloodline", BloodlineTierCfg {
        tier: 2, name: "写轮眼·万花筒", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("sharingan_bloodline_mid_unlock"),
    }),
    ("sharingan_bloodline", BloodlineTierCfg {
        tier: 3, name: "写轮眼·轮回眼", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("sharingan_bloodline_high_unlock"),
    }),
    // ---------------- hollow_bloodline · 虚化血统 ----------------
    ("hollow_bloodline", BloodlineTierCfg {
        tier: 1, name: "虚化·部分假面", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("hollow_bloodline", BloodlineTierCfg {
        tier: 2, name: "虚化·半面具", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("hollow_bloodline_mid_unlock"),
    }),
    ("hollow_bloodline", BloodlineTierCfg {
        tier: 3, name: "完全虚化", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("hollow_bloodline_high_unlock"),
    }),
    // ---------------- saint_bloodline · 圣斗士血统 ----------------
    ("saint_bloodline", BloodlineTierCfg {
        tier: 1, name: "青铜圣衣", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("saint_bloodline", BloodlineTierCfg {
        tier: 2, name: "白银圣衣", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("saint_bloodline_mid_unlock"),
    }),
    ("saint_bloodline", BloodlineTierCfg {
        tier: 3, name: "黄金圣衣", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("saint_bloodline_high_unlock"),
    }),
    // ---------------- shinigami_bloodline · 死神（斩魄刀之力 / 灵压） ----------------
    ("shinigami_bloodline", BloodlineTierCfg {
        tier: 1, name: "死神·始解", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("shinigami_bloodline", BloodlineTierCfg {
        tier: 2, name: "死神·卍解", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("shinigami_mid_unlock"),
    }),
    ("shinigami_bloodline", BloodlineTierCfg {
        tier: 3, name: "死神·王印", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("shinigami_high_unlock"),
    }),
    // ---------------- quincy_bloodline · 灭却师（灵子圣弓 / 神圣灭矢） ----------------
    ("quincy_bloodline", BloodlineTierCfg {
        tier: 1, name: "灭却师·初羽", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("quincy_bloodline", BloodlineTierCfg {
        tier: 2, name: "灭却师·圣诗", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("quincy_mid_unlock"),
    }),
    ("quincy_bloodline", BloodlineTierCfg {
        tier: 3, name: "灭却师·夺魂", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("quincy_high_unlock"),
    }),
    // ---------------- uchiha_bloodline · 宇智波（写轮眼 / 火遁） ----------------
    ("uchiha_bloodline", BloodlineTierCfg {
        tier: 1, name: "写轮眼·单勾玉", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("uchiha_bloodline", BloodlineTierCfg {
        tier: 2, name: "写轮眼·万花筒", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("uchiha_mid_unlock"),
    }),
    ("uchiha_bloodline", BloodlineTierCfg {
        tier: 3, name: "写轮眼·轮回眼", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("uchiha_high_unlock"),
    }),
    // ---------------- senju_bloodline · 千手（森罗万象 / 木遁） ----------------
    ("senju_bloodline", BloodlineTierCfg {
        tier: 1, name: "千手·初醒", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("senju_bloodline", BloodlineTierCfg {
        tier: 2, name: "千手·木遁", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("senju_mid_unlock"),
    }),
    ("senju_bloodline", BloodlineTierCfg {
        tier: 3, name: "千手·森罗万象", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("senju_high_unlock"),
    }),
    // ---------------- otsutsuki_bloodline · 大筒木（S 级，base 上浮 1.5×）----
    ("otsutsuki_bloodline", BloodlineTierCfg {
        tier: 1, name: "大筒木·始源", base: 150, k: 1, need_points: 0, need_flag: None,
    }),
    ("otsutsuki_bloodline", BloodlineTierCfg {
        tier: 2, name: "大筒木·神格", base: 1500, k: 2, need_points: 5000,
        need_flag: Some("otsutsuki_mid_unlock"),
    }),
    ("otsutsuki_bloodline", BloodlineTierCfg {
        tier: 3, name: "大筒木·灭界", base: 15000, k: 4, need_points: 50000,
        need_flag: Some("otsutsuki_high_unlock"),
    }),
    // ---------------- mitsurugi_bloodline · 鬼灭呼吸·日之呼吸（斩鬼之器）---------
    ("mitsurugi_bloodline", BloodlineTierCfg {
        tier: 1, name: "日之呼吸·壹之型", base: 100, k: 1, need_points: 0, need_flag: None,
    }),
    ("mitsurugi_bloodline", BloodlineTierCfg {
        tier: 2, name: "日之呼吸·捌之型", base: 1000, k: 2, need_points: 5000,
        need_flag: Some("mitsurugi_mid_unlock"),
    }),
    ("mitsurugi_bloodline", BloodlineTierCfg {
        tier: 3, name: "日之呼吸·太阳极", base: 10000, k: 4, need_points: 50000,
        need_flag: Some("mitsurugi_high_unlock"),
    }),
];

/// 通用默认进阶线（其余 17 个血统，主基调 ×10 递增）。
/// 阶名拼接血统名，后续子代理批量补专属阶位时替换为具名条目即可，查询逻辑不变。
fn generic_tier_name(_id: &str, tier: u8) -> &'static str {
    // 只有三档，用静态字面量避免堆分配（id 暂未用于阶名，保持通用）。
    match tier {
        1 => "初级血脉",
        2 => "中级血脉",
        3 => "高级血脉",
        _ => "顶点血脉",
    }
}

/// 取某血统某阶的配置：先查具名表，未命中则回退通用默认线（所有血统都可进阶）。
pub fn bloodline_tier_info(id: &str, tier: u8) -> Option<BloodlineTierCfg> {
    if let Some((_, cfg)) = BLOODLINE_TIERS.iter().find(|(bid, c)| *bid == id && c.tier == tier) {
        return Some(*cfg);
    }
    // 通用默认：tier 1..=3；更高的阶（4+）默认不给（后续扩充）。
    if !(1..=3).contains(&tier) {
        return None;
    }
    let (base, k) = match tier {
        1 => (100, 1),
        2 => (1000, 2),
        3 => (10000, 4),
        _ => return None,
    };
    Some(BloodlineTierCfg {
        tier,
        name: generic_tier_name(id, tier),
        base,
        k,
        need_points: match tier {
            1 => 0,
            2 => 5000,
            3 => 50000,
            _ => 0,
        },
        // 通用默认线不带剧情门槛（专属线届时再补 need_flag）。
        need_flag: None,
    })
}

/// 血统当前是否有下一阶（tier+1）。
pub fn has_next_tier(id: &str, tier: u8) -> bool {
    bloodline_tier_info(id, tier + 1).is_some()
}

/// 进阶花费查询：返回 (need_points, need_flag)。无下一阶 → None（满阶）。
pub fn bloodline_advance_cost(id: &str, cur_tier: u8) -> Option<(i64, Option<&'static str>)> {
    let next = bloodline_tier_info(id, cur_tier + 1)?;
    Some((next.need_points, next.need_flag))
}

/// 当前血统 id（None=无血统）。
pub fn current_bloodline_id(st: &GameState) -> Option<&str> {
    st.bloodline.as_deref()
}

/// 当前血统阶位（默认 1；无血统 → 0）。
pub fn current_bloodline_tier(st: &GameState) -> u8 {
    match st.bloodline.as_deref() {
        Some(id) => st.bloodline_tier.get(id).copied().unwrap_or(1),
        None => 0,
    }
}

/// 当前血统阶位内贡献值（默认 0）。
pub fn current_contribution(st: &GameState) -> i32 {
    match st.bloodline.as_deref() {
        Some(id) => st.bloodline_contribution.get(id).copied().unwrap_or(0),
        None => 0,
    }
}

/// 当前血统强度：base + contribution。血统为空 → 0。
///
/// 公式（验收点 1/2）：
///   strength(id) = tier.base + contribution
///   contribution 每次使用血统相关技能/攻击 +k（当前阶系数）。
pub fn bloodline_strength(st: &GameState) -> i32 {
    let id = match current_bloodline_id(st) {
        Some(id) => id,
        None => return 0,
    };
    let tier = current_bloodline_tier(st);
    let cfg = match bloodline_tier_info(id, tier) {
        Some(c) => c,
        None => return 0,
    };
    let contrib = current_contribution(st);
    cfg.base + contrib
}

/// 进阶失败/成功的枚举原因（供 API 与测试统一判读）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdvanceError {
    /// 未装配血统
    NoBloodline,
    /// 血统不存在于进阶路线
    UnknownBloodline,
    /// 已满阶（无下一阶）
    MaxTier,
    /// 点数不足
    NotEnoughPoints,
    /// 剧情权限旗标不满足
    FlagNotMet,
}

impl std::fmt::Display for AdvanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            AdvanceError::NoBloodline => "未装配任何血统。",
            AdvanceError::UnknownBloodline => "该血统无进阶路线。",
            AdvanceError::MaxTier => "已是最高阶位，无法继续进阶。",
            AdvanceError::NotEnoughPoints => "点数不足，无法进阶。",
            AdvanceError::FlagNotMet => "支线剧情权限未满足，无法进阶。",
        };
        write!(f, "{msg}")
    }
}

/// 进阶：number 指定血统 id；校验当前血统、有下一阶、点数够、need_flag 满足。
/// 满足则：扣点数、当前阶内贡献砍半（向下取整）、tier+1、换新 base。
///
/// 进阶换算（砍半 + 保底不下降）：
///   旧贡献_old、旧强度 = old_base + old_contrib
///   新贡献 = old_contrib / 2（向下取整）
///   新强度 = max(new_base + 新贡献, 旧强度)   // 保底不下降
/// 直接存贡献值（bloodline_contribution），避免反推熟练度的整除丢失。
pub fn try_advance_bloodline(st: &mut GameState, id: &str) -> Result<u8, AdvanceError> {
    let cur_id = current_bloodline_id(st).ok_or(AdvanceError::NoBloodline)?;
    if cur_id != id {
        // 进阶指定的不是当前血统 → 视为不匹配（当前互斥单血统）
        return Err(AdvanceError::NoBloodline);
    }
    let cur_tier = current_bloodline_tier(st);
    let old_cfg = bloodline_tier_info(id, cur_tier).ok_or(AdvanceError::UnknownBloodline)?;
    let next_cfg = bloodline_tier_info(id, cur_tier + 1).ok_or(AdvanceError::MaxTier)?;

    // 点数校验
    if (st.points as i64) < next_cfg.need_points {
        return Err(AdvanceError::NotEnoughPoints);
    }
    // 剧情权限校验（None=无门槛）
    if let Some(f) = next_cfg.need_flag {
        if !st.flag(f) {
            return Err(AdvanceError::FlagNotMet);
        }
    }

    // 旧贡献/旧强度
    let old_contrib = current_contribution(st);
    let old_strength = old_cfg.base + old_contrib;
    // 贡献砍半（向下取整）
    let new_contrib = old_contrib / 2;
    // 新强度保底不下降
    let new_strength = (next_cfg.base + new_contrib).max(old_strength);
    // 保底生效时（新 base+砍半贡献 < 旧强度），把贡献补足到保底强度对应的量。
    // 补足贡献值 = new_strength - new_base（保持 strength = base + contribution 不变式）。
    let final_contrib = new_strength - next_cfg.base;

    // 扣点数、写贡献、升阶
    st.points = st.points.saturating_sub(next_cfg.need_points as i32);
    st.bloodline_contribution.insert(id.to_string(), final_contrib);
    st.bloodline_tier.insert(id.to_string(), next_cfg.tier);

    Ok(next_cfg.tier)
}

