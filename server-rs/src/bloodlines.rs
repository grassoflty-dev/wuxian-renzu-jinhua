//! 血统数据层（combat §1.2；vampire 迁入，被动对现 is_vampire 内联行为等价）＋血统进阶/强度入口转发。
//! 仅数据铺装与查询 helper，不含任何战斗结算逻辑（结算由包 B 做）。
//! 全部数值「建议值·可调」，拷贝自 tools/design/* 设计文档。

use crate::defs::{BloodlineDef, BloodlinePassive};
use crate::state::GameState;

pub use crate::bloodline_tiers::{
    bloodline_advance_cost, bloodline_strength, bloodline_tier_info, current_bloodline_id,
    current_bloodline_tier, current_contribution, has_next_tier, try_advance_bloodline,
    AdvanceError, BloodlineTierCfg, BLOODLINE_TIERS,
};

// ============================================================================
// BLOODLINES · 血统池（combat §1.2；vampire 迁入，被动对现 is_vampire 内联行为等价）
// 现有内联：vampire → 受击减 3（reduce3）、命中吸血 4（leech4）
// ============================================================================
pub const BLOODLINES: &[BloodlineDef] = &[
    BloodlineDef {
        id: "vampire",
        name: "初级吸血鬼血统",
        desc: "夜属于你——每一次撕裂都把敌人的生命汲为己有。",
        passive: BloodlinePassive {
            atk_flat: 0, leech_on_hit: 4, dodge_bonus: 0.0, dmg_reduce: 3, san_resist: 0,
            rage_bonus_atk: 0, label: "攻击吸血·受击减3",
        },
    },
    BloodlineDef {
        id: "werewolf",
        name: "狼人血统",
        desc: "远祖的兽性在月下苏醒——近身撕裂，狂暴时更凶。",
        passive: BloodlinePassive {
            atk_flat: 8, leech_on_hit: 0, dodge_bonus: 0.05, dmg_reduce: 2, san_resist: 0,
            rage_bonus_atk: 10, label: "攻+8·狂暴+10·减2",
        },
    },
    BloodlineDef {
        id: "zuwu",
        name: "祖巫血脉",
        desc: "远古其骨：肉身即壁垒，兽潮炎魔亦难破。",
        passive: BloodlinePassive {
            atk_flat: 0, leech_on_hit: 0, dodge_bonus: 0.0, dmg_reduce: 10, san_resist: 0,
            rage_bonus_atk: 0, label: "受击减10 高坦",
        },
    },
    BloodlineDef {
        id: "zhanshi_blood",
        name: "圣光圣职者血脉",
        desc: "圣祷加持心神，克制咒怨与日光光环。",
        passive: BloodlinePassive {
            atk_flat: 0, leech_on_hit: 0, dodge_bonus: 0.0, dmg_reduce: 0, san_resist: 8,
            rage_bonus_atk: 0, label: "SAN抗+8",
        },
    },
    BloodlineDef {
        id: "gauss_cyber",
        name: "纳米血统·共振",
        desc: "与机械共感（科技侧预留）。",
        passive: BloodlinePassive {
            atk_flat: 4, leech_on_hit: 0, dodge_bonus: -0.01, dmg_reduce: 6, san_resist: 0,
            rage_bonus_atk: 0, label: "攻+4·减6·闪-0.01",
        },
    },
    // ---- 增量扩充（第 2 批 · 天使/恶魔/龙族/机械义体）----
    BloodlineDef {
        id: "angel_bloodline",
        name: "天使血统",
        desc: "圣光凝筑双翼，心志如炽——暗秽退散，濒死亦能历光重生。",
        passive: BloodlinePassive {
            atk_flat: 0, leech_on_hit: 0, dodge_bonus: 0.0, dmg_reduce: 4, san_resist: 12,
            rage_bonus_atk: 0, label: "SAN抗+12·受击减4",
        },
    },
    BloodlineDef {
        id: "demon_bloodline",
        name: "恶魔血统",
        desc: "以血祭炼，以命换力——每一次撕裂都从敌人处夺回生机。",
        passive: BloodlinePassive {
            atk_flat: 12, leech_on_hit: 6, dodge_bonus: 0.0, dmg_reduce: 0, san_resist: 0,
            rage_bonus_atk: 15, label: "攻+12·吸血6·狂暴+15",
        },
    },
    BloodlineDef {
        id: "dragon_bloodline",
        name: "龙族血统",
        desc: "龙魂入髓，钢鳞覆皮——本体即为最凶之器，鳞甲坚不可摧。",
        passive: BloodlinePassive {
            atk_flat: 6, leech_on_hit: 0, dodge_bonus: 0.0, dmg_reduce: 14, san_resist: 0,
            rage_bonus_atk: 0, label: "攻+6·受击减14 高坦",
        },
    },
    BloodlineDef {
        id: "cyber_prosthetic",
        name: "机械义体血统",
        desc: "机械器官与神经同频——身负义体，闪避如电。",
        passive: BloodlinePassive {
            atk_flat: 8, leech_on_hit: 0, dodge_bonus: 0.08, dmg_reduce: 8, san_resist: 0,
            rage_bonus_atk: 0, label: "攻+8·减8·闪+0.08",
        },
    },
    // ==================== 增量扩充（第 3 批 · +10 血统 · 动漫/小说） ====================
    BloodlineDef {
        id: "saiyan_bloodline",
        name: "赛亚人血统",
        desc: "濒死愈强，怒意化气——每一次倒下都让下一次站起更凶悍。",
        passive: BloodlinePassive {
            atk_flat: 12, leech_on_hit: 0, dodge_bonus: 0.02, dmg_reduce: 4, san_resist: 0,
            rage_bonus_atk: 25, label: "攻+12·减4·狂暴+25",
        },
    },
    BloodlineDef {
        id: "sharingan_bloodline",
        name: "写轮眼血统",
        desc: "眼眸映照万象，洞察先机——闪避如影，一眼看破杀意。",
        passive: BloodlinePassive {
            atk_flat: 6, leech_on_hit: 0, dodge_bonus: 0.15, dmg_reduce: 0, san_resist: 4,
            rage_bonus_atk: 10, label: "攻+6·闪+0.15·SAN抗+4",
        },
    },
    BloodlineDef {
        id: "hollow_bloodline",
        name: "虚化血统",
        desc: "面具之下是深渊——每一次命中都撕扯敌人生机。",
        passive: BloodlinePassive {
            atk_flat: 14, leech_on_hit: 8, dodge_bonus: -0.02, dmg_reduce: 0, san_resist: 0,
            rage_bonus_atk: 15, label: "攻+14·吸血8·狂暴+15",
        },
    },
    BloodlineDef {
        id: "saint_bloodline",
        name: "圣斗士血统",
        desc: "第七感觉醒，小宇宙燃烧——圣衣之魂护佑己身。",
        passive: BloodlinePassive {
            atk_flat: 8, leech_on_hit: 0, dodge_bonus: 0.05, dmg_reduce: 10, san_resist: 6,
            rage_bonus_atk: 0, label: "攻+8·减10·闪+0.05",
        },
    },
    BloodlineDef {
        id: "shinigami_bloodline",
        name: "死神血统",
        desc: "持有斩魄刀之力，灵压如渊——出手即割裂万物。",
        passive: BloodlinePassive {
            atk_flat: 12, leech_on_hit: 0, dodge_bonus: 0.04, dmg_reduce: 6, san_resist: 0,
            rage_bonus_atk: 0, label: "攻+12·减6·闪+0.04",
        },
    },
    BloodlineDef {
        id: "quincy_bloodline",
        name: "灭却师血统",
        desc: "灵子圣弓在手，神圣灭矢——对秽物穿透尤厉。",
        passive: BloodlinePassive {
            atk_flat: 16, leech_on_hit: 0, dodge_bonus: 0.0, dmg_reduce: 2, san_resist: 0,
            rage_bonus_atk: 0, label: "攻+16·减2",
        },
    },
    BloodlineDef {
        id: "uchiha_bloodline",
        name: "宇智波血脉",
        desc: "瞳术天之骄子，火遁贯穿——怒火化作蚀骨炎刀。",
        passive: BloodlinePassive {
            atk_flat: 10, leech_on_hit: 4, dodge_bonus: 0.06, dmg_reduce: 0, san_resist: 0,
            rage_bonus_atk: 10, label: "攻+10·吸血4·闪+0.06",
        },
    },
    BloodlineDef {
        id: "senju_bloodline",
        name: "千手血脉",
        desc: "森罗万象之体，愈战愈勇——受创愈多，回气愈盛。",
        passive: BloodlinePassive {
            atk_flat: 0, leech_on_hit: 0, dodge_bonus: 0.0, dmg_reduce: 12, san_resist: 4,
            rage_bonus_atk: 0, label: "减12·SAN抗+4 高坦",
        },
    },
    BloodlineDef {
        id: "otsutsuki_bloodline",
        name: "大筒木血脉",
        desc: "星天之血，神之之力——承受与撕裂并存，近乎规则之躯。",
        passive: BloodlinePassive {
            atk_flat: 14, leech_on_hit: 6, dodge_bonus: 0.0, dmg_reduce: 12, san_resist: 6,
            rage_bonus_atk: 0, label: "攻+14·吸血6·减12",
        },
    },
    BloodlineDef {
        id: "mitsurugi_bloodline",
        name: "鬼灭呼吸·日之呼吸",
        desc: "太阳之呼吸流转，斩鬼之器——对暗秽炎克双绝。",
        passive: BloodlinePassive {
            atk_flat: 12, leech_on_hit: 0, dodge_bonus: 0.03, dmg_reduce: 6, san_resist: 0,
            rage_bonus_atk: 8, label: "攻+12·减6·闪+0.03·狂暴+8",
        },
    },
    // ==================== 增量扩充（第 4 批 · 数据子代理追加 · zhttty 系题材） ====================
    BloodlineDef {
        id: "blood_gene_avatar",
        name: "基因锁·觉醒者血脉",
        desc: "主神兑换——锁率与血脉同炉，狂怒与濒死都成了可燃烧的柴。",
        passive: BloodlinePassive {
            atk_flat: 10, leech_on_hit: 2, dodge_bonus: 0.04, dmg_reduce: 4, san_resist: 0,
            rage_bonus_atk: 12, label: "攻+10·吸血2·减4·狂暴+12",
        },
    },
    BloodlineDef {
        id: "xenomorph_bloodline",
        name: "异形血统",
        desc: "主神兑换——黑壳覆身，酸血蚀骨，你已不在「人类」的名册上。",
        passive: BloodlinePassive {
            atk_flat: 8, leech_on_hit: 8, dodge_bonus: 0.05, dmg_reduce: 6, san_resist: 0,
            rage_bonus_atk: 8, label: "攻+8·吸血8·减6·闪+0.05",
        },
    },
    BloodlineDef {
        id: "spiderman_bloodline",
        name: "蜘蛛侠血统",
        desc: "主神兑换——蛛丝如臂指使：点击/指向高处建筑与墙面喷射蛛丝，将你拉拽过去；身法飘忽，闪避如风。",
        passive: BloodlinePassive {
            atk_flat: 6, leech_on_hit: 0, dodge_bonus: 0.08, dmg_reduce: 0, san_resist: 0,
            rage_bonus_atk: 0, label: "闪+0.08·攻+6·蛛丝拉拽",
        },
    },
];

/// 按血统 id 查表（Option<&BloodlineDef>）
pub fn bloodline_def(id: &str) -> Option<&'static BloodlineDef> {
    BLOODLINES.iter().find(|b| b.id == id)
}

/// 按当前角色血统查表
pub fn bloodline_of(st: &GameState) -> Option<&'static BloodlineDef> {
    match st.bloodline.as_deref() {
        Some(id) => bloodline_def(id),
        None => None,
    }
}
