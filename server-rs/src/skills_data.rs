//! 战斗体系数据层（包 A）：SKILLS 技能库静态表（149 条合并：现有 33 + 扩充 76 + 基因锁 15 + 修真 22）。
//! 仅数据铺装与查询，不含战斗结算逻辑（由包 B 分发 SkillEffect）。
//! 数值 / 文案均「建议值·可调」，自创模仿主神兑换语气，非照抄受版权原文。
//! id 前缀：sk_ww_ (武道) / sk_gene_ / skx_gene_ (基因) / sk_vamp|skx_blood (血统) /
//!         sk_holy_|skx_holy_ (圣光) / sk_tech_|skx_tech_ (科技) / sk_nt_|skx_nt_ (超能NT) /
//!         sk_meme_|skx_meme_ (模因) / sk_util_|skx_util_ (通用) / cu_*|skx_xiu_* (修真)。

use crate::defs::{
    SkillCost, SkillDef, SkillEffect, SkillKind, SkillSchool,
};

// ============================================================================
// SKILLS · 全表（按 id 升序物理合并，线性查找）
// ============================================================================
pub const SKILLS: &[SkillDef] = &[
    // ================= A′① 修真·功法 (cu_gong_*) =================
    SkillDef {
        id: "cu_gong_hecheng", name: "合道归元气", school: SkillSchool::Xiu,
        desc: "返虚气海无边——吐纳之间，三十三层天罡尽归丹田。",
        price: 40000, need_grade: Some('S'), need_bloodline: None, need_qi: None,
        need_stage: None, need_cultivation: Some(6), kind: SkillKind::Passive,
        cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 0, san_resist: 0, qi_max: 100, hp_max: 60, per_turn_qi: 6 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_gong_dantian", name: "丹田淬体诀", school: SkillSchool::Xiu,
        desc: "丹田如炉，肉身不坏。", price: 2800, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(2),
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 4, san_resist: 0, qi_max: 0, hp_max: 30, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_gong_daoyin", name: "太乙导引法", school: SkillSchool::Xiu,
        desc: "导引天地精气——战罢回气，血肉重铸。", price: 22000, need_grade: Some('A'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(5),
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 0, san_resist: 0, qi_max: 0, hp_max: 20, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_gong_hunhei", name: "混元神罡", school: SkillSchool::Xiu,
        desc: "元婴之神护道周全。", price: 12000, need_grade: Some('A'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(4),
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 8, san_resist: 8, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_gong_qiling", name: "吐故纳新功", school: SkillSchool::Xiu,
        desc: "呼吸间真气生生不息——练气心法之基。", price: 1200, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(1),
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 0, san_resist: 3, qi_max: 40, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_gong_zhuque", name: "朱雀心火法", school: SkillSchool::Xiu,
        desc: "心火自燃，气海翻腾。", price: 6000, need_grade: Some('B'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(3),
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 6, leech: 0, dodge_bonus: 0.0, dmg_reduce: 0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 4 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= A′② 修真·禁制 (cu_jin_*) =================
    SkillDef {
        id: "cu_jin_dingji", name: "定身符", school: SkillSchool::Xiu,
        desc: "定身一回合，任其挣扎。", price: 2800, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(2),
        kind: SkillKind::Active, cost: SkillCost::Qi(7),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 1, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_jin_kunxian", name: "困仙禁制", school: SkillSchool::Xiu,
        desc: "禁制锁身，两回合攻击必中。", price: 1300, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(1),
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::DebuffEnemy { no_dodge: 2, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_jin_lingri", name: "临日封印", school: SkillSchool::Xiu,
        desc: "封印敌狂暴与负面转化。", price: 7000, need_grade: Some('B'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(3),
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_jin_ruyin", name: "如印封天", school: SkillSchool::Xiu,
        desc: "封印敌全部增益两回合，诛邪灭灵。", price: 52000, need_grade: Some('S'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(6),
        kind: SkillKind::Active, cost: SkillCost::Qi(18),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: Some((12, 3)), dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_jin_xianzhi", name: "禁灵罩", school: SkillSchool::Xiu,
        desc: "封敌方神通加成，我方心神稳固。", price: 26000, need_grade: Some('A'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(5),
        kind: SkillKind::Active, cost: SkillCost::Qi(14),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_jin_zhenmo", name: "镇魔阵", school: SkillSchool::Xiu,
        desc: "阵法镇压：敌输出减半两回合并持续灼伤。", price: 14000, need_grade: Some('A'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(4),
        kind: SkillKind::Active, cost: SkillCost::Qi(12),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: Some((8, 3)), dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= A′③ 修真·神通 (cu_shen_*) =================
    SkillDef {
        id: "cu_shen_danhuo", name: "丹火·流焰", school: SkillSchool::Xiu,
        desc: "丹火焚敌，对弱火 ×1.5。", price: 6000, need_grade: Some('B'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(3),
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::Striking { dmg: (30, 40), ignore_armor: false, hits: 1, weak: Some(&["fire"]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_shen_daoying", name: "返虚倒影", school: SkillSchool::Xiu,
        desc: "夺敌一式增益为己用三回合。", price: 50000, need_grade: Some('S'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(6),
        kind: SkillKind::Active, cost: SkillCost::Qi(18),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_shen_dunjian", name: "御剑遁法", school: SkillSchool::Xiu,
        desc: "身随剑走，护体如壁。", price: 3000, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(2),
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 1, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: true, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_shen_fankui", name: "反叩天地", school: SkillSchool::Xiu,
        desc: "神意化身，下一击 ×1.8。", price: 28000, need_grade: Some('A'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(5),
        kind: SkillKind::Active, cost: SkillCost::Qi(16),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_shen_hunse", name: "摄魂术", school: SkillSchool::Xiu,
        desc: "摄其魂魄，敌下一轮不动。", price: 12000, need_grade: Some('A'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(4),
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 1, dmg_over_time: None, dmg: (12, 18) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_shen_jianren", name: "剑丸·一线银丝", school: SkillSchool::Xiu,
        desc: "一线银丝破甲而行，对 armor>0 敌 ×1.3。", price: 1500, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(1),
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::Striking { dmg: (16, 24), ignore_armor: false, hits: 1, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_shen_jianyu", name: "万剑归宗·雏形", school: SkillSchool::Xiu,
        desc: "剑丸满银丝，万剑齐发。", price: 15000, need_grade: Some('A'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(4),
        kind: SkillKind::Active, cost: SkillCost::Qi(14),
        effect: SkillEffect::Striking { dmg: (24, 30), ignore_armor: false, hits: 3, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_shen_tiandai", name: "天地同游", school: SkillSchool::Xiu,
        desc: "一瞬合道之击——无视护甲，克制全乘。", price: 80000, need_grade: Some('S'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(7),
        kind: SkillKind::Active, cost: SkillCost::Qi(22),
        effect: SkillEffect::Striking { dmg: (50, 70), ignore_armor: true, hits: 1, weak: Some(&["fire", "electric", "holy", "silver"]) },
        per_fight_uses: None, cooldown: 0, flying: true, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_shen_yufeng", name: "御风行", school: SkillSchool::Xiu,
        desc: "身法+吐息，一回合闪避大涨。", price: 2600, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(2),
        kind: SkillKind::Active, cost: SkillCost::Qi(5),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.30, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: true, detect_bonus: 0,
    },
    SkillDef {
        id: "cu_shen_zhangxin", name: "掌心雷", school: SkillSchool::Xiu,
        desc: "掌心雷动，对机械/量子 ×1.5。", price: 1400, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: Some(1),
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::Striking { dmg: (14, 20), ignore_armor: false, hits: 1, weak: Some(&["electric"]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= B 现有·武道内功 (sk_ww_*, 6) =================
    SkillDef {
        id: "sk_ww_bopo", name: "破军一击", school: SkillSchool::Wushu,
        desc: "刚猛破防——对 grief/armor>0 敌 ×1.5。", price: 1100, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(60), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::Striking { dmg: (20, 28), ignore_armor: true, hits: 1, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_ww_jiuzhuan", name: "九转归元", school: SkillSchool::Wushu,
        desc: "爆发绝学，连击两段。", price: 1600, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(40), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(18),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 20 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_ww_liantui", name: "连环腿法", school: SkillSchool::Wushu,
        desc: "三连腿影，可叠暴击。", price: 700, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::Striking { dmg: (6, 9), ignore_armor: false, hits: 3, weak: Some(&[]) },
        per_fight_uses: Some(2), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_ww_wenxin", name: "问心一剑", school: SkillSchool::Wushu,
        desc: "绝学——那一剑无物可御，无视护甲直斩。", price: 1500, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(40), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(12),
        effect: SkillEffect::Striking { dmg: (30, 40), ignore_armor: true, hits: 1, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_ww_wumian", name: "无面心法", school: SkillSchool::Wushu,
        desc: "内功心法加强版——气海愈深，心神愈稳。", price: 1400, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(40), need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 0, san_resist: 4, qi_max: 30, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_ww_wuxiang", name: "无相步", school: SkillSchool::Wushu,
        desc: "身法绝学——守势提升下一轮闪避。", price: 1100, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(60), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.25, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= C 现有·基因锁 (sk_gene_*, 3，含 berserk 保留) =================
    SkillDef {
        id: "sk_gene_berserk", name: "本能爆发", school: SkillSchool::Gene,
        desc: "解放锁率的最后一瞬爆发——舍闪避换狂攻。", price: 3000, need_grade: Some('B'),
        need_bloodline: None, need_qi: None, need_stage: Some(3), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 15 },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_focus", name: "猎手凝视", school: SkillSchool::Gene,
        desc: "锁住猎物的重心——接下来两回合攻击必中。", price: 1600, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: Some(2), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::DebuffEnemy { no_dodge: 2, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 2,
    },
    SkillDef {
        id: "sk_gene_instinct", name: "战斗直觉", school: SkillSchool::Gene,
        desc: "身体先于剑至——一阶被动加成。", price: 2000, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: Some(1), need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 4, leech: 0, dodge_bonus: 0.08, dmg_reduce: 0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= D 现有·基因锁三/四阶新条目 (基因锁文档，15) =================
    SkillDef {
        id: "sk_gene_evade", name: "极限闪避", school: SkillSchool::Gene,
        desc: "让脊柱替你躲开死亡——下次伤势免疫。", price: 1200, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: Some(1), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 1.0, atk_flat: 0 },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_stagger", name: "撕咬反扑", school: SkillSchool::Gene,
        desc: "以伤换伤的起始——受击后反扑。", price: 900, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: Some(1), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::Striking { dmg: (8, 12), ignore_armor: false, hits: 1, weak: Some(&[]) },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_harden", name: "肌肉硬化", school: SkillSchool::Gene,
        desc: "骨肉如铁，免疫失衡。", price: 2000, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: Some(2), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 10, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_bone", name: "骨骼重构", school: SkillSchool::Gene,
        desc: "把断裂的尺骨重新焊回原处。", price: 2400, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: Some(2), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::SelfBuff { hp: 25, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_power", name: "力量爆发", school: SkillSchool::Gene,
        desc: "爆发一瞬，下一击 +15，舍却回避。", price: 2200, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: Some(2), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 15 },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_regen", name: "再生", school: SkillSchool::Gene,
        desc: "肌肉掌控常驻恢复。", price: 3000, need_grade: Some('B'),
        need_bloodline: None, need_qi: None, need_stage: Some(2), need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 3, san_resist: 0, qi_max: 0, hp_max: 20, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_psych", name: "精神冲击", school: SkillSchool::Gene,
        desc: "以念头砸进敌人太阳穴。", price: 4000, need_grade: Some('B'),
        need_bloodline: None, need_qi: None, need_stage: Some(3), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 1, dmg_over_time: None, dmg: (8, 14) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_wall", name: "念动壁", school: SkillSchool::Gene,
        desc: "意志凝壁，吸收下一次全额伤害。", price: 4200, need_grade: Some('B'),
        need_bloodline: None, need_qi: None, need_stage: Some(3), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 25, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_scan", name: "心灵扫描", school: SkillSchool::Gene,
        desc: "读透敌人，心神略安。", price: 3600, need_grade: Some('B'),
        need_bloodline: None, need_qi: None, need_stage: Some(3), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::SelfBuff { hp: 0, san: 5, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 2,
    },
    SkillDef {
        id: "sk_gene_link", name: "精神链接", school: SkillSchool::Gene,
        desc: "联结队友，协力作战。", price: 3800, need_grade: Some('B'),
        need_bloodline: None, need_qi: None, need_stage: Some(3), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(5),
        effect: SkillEffect::SelfBuff { hp: 0, san: 10, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_field", name: "规则领域", school: SkillSchool::Gene,
        desc: "撑开小范围领域，攻防一体。", price: 8000, need_grade: Some('A'),
        need_bloodline: None, need_qi: None, need_stage: Some(4), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(15),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.15, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_timesense", name: "时间感", school: SkillSchool::Gene,
        desc: "时停视角——下一击必中且敌不可反击。", price: 7500, need_grade: Some('A'),
        need_bloodline: None, need_qi: None, need_stage: Some(4), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(14),
        effect: SkillEffect::DebuffEnemy { no_dodge: 1, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_fold", name: "空间折叠·初级", school: SkillSchool::Gene,
        desc: "折叠空间，无视护盾护甲直击。", price: 8600, need_grade: Some('A'),
        need_bloodline: None, need_qi: None, need_stage: Some(4), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(16),
        effect: SkillEffect::Striking { dmg: (30, 38), ignore_armor: true, hits: 1, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_liber", name: "基因锁完全解放", school: SkillSchool::Gene,
        desc: "燃尽此身的至强一瞬——战后力竭。", price: 12000, need_grade: Some('A'),
        need_bloodline: None, need_qi: None, need_stage: Some(4), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 30 },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_gene_apex", name: "规则承载", school: SkillSchool::Gene,
        desc: "四阶常驻加成——靠近圣位的余温。", price: 9000, need_grade: Some('A'),
        need_bloodline: None, need_qi: None, need_stage: Some(4), need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 8, leech: 0, dodge_bonus: 0.03, dmg_reduce: 4, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= E 现有·血统 (sk_vamp/sk_wolf/sk_zuwu/sk_zhanshi, 5) =================
    SkillDef {
        id: "sk_vamp_frenzy", name: "血之狂潮", school: SkillSchool::Blood,
        desc: "渴意倾泻，撕裂并把敌之命汲为己有。", price: 2600, need_grade: Some('C'),
        need_bloodline: Some("vampire"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::Striking { dmg: (18, 26), ignore_armor: false, hits: 2, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_vamp_mist", name: "血雾遁形", school: SkillSchool::Blood,
        desc: "化作血雾，一回合闪避暴涨。", price: 2000, need_grade: Some('C'),
        need_bloodline: Some("vampire"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(5),
        effect: SkillEffect::SelfBuff { hp: 0, san: 5, guard: 0, dodge_bonus: 0.30, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: true, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_wolf_rend", name: "狂暴撕咬", school: SkillSchool::Blood,
        desc: "月下兽性，撕裂非机械之敌。", price: 3000, need_grade: Some('B'),
        need_bloodline: Some("werewolf"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::Striking { dmg: (28, 38), ignore_armor: false, hits: 1, weak: Some(&[]) },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_zuwu_iron", name: "祖巫不灭身", school: SkillSchool::Blood,
        desc: "血脉如铁，肉身不灭。", price: 3600, need_grade: Some('B'),
        need_bloodline: Some("zuwu"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 8, san_resist: 0, qi_max: 0, hp_max: 20, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_zhanshi_light", name: "圣职者之誓", school: SkillSchool::Blood,
        desc: "圣誓加身，心神稳固兼养攻势。", price: 2400, need_grade: Some('C'),
        need_bloodline: Some("zhanshi_blood"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 3, leech: 0, dodge_bonus: 0.0, dmg_reduce: 0, san_resist: 6, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= F 现有·圣光 (sk_holy_*, 4) =================
    SkillDef {
        id: "sk_holy_burst", name: "圣光术", school: SkillSchool::Holy,
        desc: "圣火驱暗——对亡灵 ×1.5，抚平心神。", price: 1600, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(20), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::Striking { dmg: (20, 28), ignore_armor: false, hits: 1, weak: Some(&["holy"]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_holy_water", name: "圣水祝福", school: SkillSchool::Holy,
        desc: "以圣水涤净怨灵之影。", price: 900, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Item("item_holy_water"),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: Some((8, 3)), dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_holy_purify", name: "圣印驱散", school: SkillSchool::Holy,
        desc: "净化诅咒，免疫一次即死。", price: 1400, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(20), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::SelfBuff { hp: 0, san: 12, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_holy_veil", name: "圣光护佑", school: SkillSchool::Holy,
        desc: "圣光常驻庇佑。", price: 1800, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(20), need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 3, san_resist: 8, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= G 现有·科技 (sk_tech_*, 4) =================
    SkillDef {
        id: "sk_tech_emp", name: "电磁脉冲", school: SkillSchool::Tech,
        desc: "蓝白电弧炸开，硅基忠诚在 0.4 秒内烧毁。", price: 2400, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(20), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::Striking { dmg: (18, 26), ignore_armor: false, hits: 1, weak: Some(&["electric"]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_tech_nano", name: "纳米修复", school: SkillSchool::Tech,
        desc: "纳米愈合躯体。", price: 1800, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(10), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(5),
        effect: SkillEffect::SelfBuff { hp: 25, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_tech_ballistic", name: "弹道预判", school: SkillSchool::Tech,
        desc: "必中之枪，下一枪 ×1.3。", price: 1800, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::DebuffEnemy { no_dodge: 1, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_tech_scanner", name: "战术扫描", school: SkillSchool::Tech,
        desc: "首发命中强化，读弱词显示。", price: 1600, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 6, leech: 0, dodge_bonus: 0.0, dmg_reduce: 0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 3,
    },
    // ================= H 现有·超能NT (sk_nt_*, 4) =================
    SkillDef {
        id: "sk_nt_precog", name: "灾祸预知", school: SkillSchool::Nt,
        desc: "预知折射一次伤势。", price: 1800, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.5, atk_flat: 0 },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 3,
    },
    SkillDef {
        id: "sk_nt_empathy", name: "读心感应", school: SkillSchool::Nt,
        desc: "读透弱词，知己知彼。", price: 1600, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(20), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(4),
        effect: SkillEffect::SelfBuff { hp: 0, san: 5, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 2,
    },
    SkillDef {
        id: "sk_nt_telekinetic", name: "念动力场", school: SkillSchool::Nt,
        desc: "思维凝成看不见的壁。", price: 2600, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 25, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_nt_push", name: "精神冲击", school: SkillSchool::Nt,
        desc: "指挥冲击，敌下一轮不动。", price: 2400, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 1, dmg_over_time: None, dmg: (8, 14) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= I 现有·模因 (sk_meme_*, 3) =================
    SkillDef {
        id: "sk_meme_mark", name: "道德印记", school: SkillSchool::Meme,
        desc: "烙上『注定被击败』的名字。", price: 1800, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(10), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_meme_link", name: "心灵链接", school: SkillSchool::Meme,
        desc: "联通心神，互为援手。", price: 1600, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(10), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(5),
        effect: SkillEffect::SelfBuff { hp: 0, san: 10, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_meme_seal", name: "概念封锁", school: SkillSchool::Meme,
        desc: "划掉『暴走』一句。", price: 3000, need_grade: Some('B'),
        need_bloodline: None, need_qi: None, need_stage: Some(2), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(12),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= J 现有·通用 (sk_util_*, 4) =================
    SkillDef {
        id: "sk_util_inspect", name: "洞察侦查", school: SkillSchool::Util,
        desc: "看破敌方一切底细。", price: 800, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: Some(2), cooldown: 0, flying: false, detect_bonus: 4,
    },
    SkillDef {
        id: "sk_util_bandage", name: "紧急包扎", school: SkillSchool::Util,
        desc: "先在伤口上做手脚，活下来再说。", price: 900, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Item("item_bandage"),
        effect: SkillEffect::SelfBuff { hp: 18, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: Some(2), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_util_morale", name: "振奋咆哮", school: SkillSchool::Util,
        desc: "一声咆哮，士气大振。", price: 1000, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::SelfBuff { hp: 0, san: 8, guard: 0, dodge_bonus: 0.0, atk_flat: 6 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "sk_util_antidote", name: "净化血清", school: SkillSchool::Util,
        desc: "解除毒与灼烧并补血。", price: 1100, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Item("item_antidote"),
        effect: SkillEffect::SelfBuff { hp: 8, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= K 扩充·武道 (skx_ww_*, 16) =================
    SkillDef {
        id: "skx_ww_anqi", name: "袖里飞蝗", school: SkillSchool::Wushu,
        desc: "暗器先制，先发制人。", price: 900, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Item("item_飛蝗"),
        effect: SkillEffect::Striking { dmg: (12, 18), ignore_armor: false, hits: 2, weak: Some(&[]) },
        per_fight_uses: Some(2), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_bagua", name: "八卦游身掌", school: SkillSchool::Wushu,
        desc: "游身走位，攻守两立。", price: 800, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::Striking { dmg: (14, 20), ignore_armor: false, hits: 2, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_bopi", name: "破甲铁臂", school: SkillSchool::Wushu,
        desc: "铁臂千钧，破甲进击。", price: 1200, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(45), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::Striking { dmg: (18, 26), ignore_armor: true, hits: 1, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_cangjian", name: "藏锋于鞘", school: SkillSchool::Wushu,
        desc: "蓄力待发，一击扬眉。", price: 1000, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(40), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 12 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_dianxue", name: "定穴指", school: SkillSchool::Wushu,
        desc: "点中要穴，下轮行动减弱。", price: 900, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(30), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 1, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_fenjin", name: "分筋错骨手", school: SkillSchool::Wushu,
        desc: "卸掉关节，让怨恨都做不完。", price: 900, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(40), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::Striking { dmg: (10, 16), ignore_armor: false, hits: 1, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_jingangzhao", name: "金钟罩", school: SkillSchool::Wushu,
        desc: "放下金钟，护体一回合。", price: 1400, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(50), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(12),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 18, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_jueming", name: "绝命一击", school: SkillSchool::Wushu,
        desc: "把闪避后路全押上——那是决心本身。", price: 2200, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(70), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(16),
        effect: SkillEffect::Striking { dmg: (35, 48), ignore_armor: false, hits: 1, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_lianhuan", name: "连环穿掌", school: SkillSchool::Wushu,
        desc: "连击穿掌，可叠暴击。", price: 800, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(35), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::Striking { dmg: (7, 10), ignore_armor: false, hits: 3, weak: Some(&[]) },
        per_fight_uses: Some(2), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_neigong", name: "周天养气诀", school: SkillSchool::Wushu,
        desc: "气走周天，再养一层。", price: 1300, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(60), need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 0, san_resist: 2, qi_max: 25, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_qise", name: "七弦无形剑", school: SkillSchool::Wushu,
        desc: "音剑克心魔，读 san 弱化。", price: 1800, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(60), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(12),
        effect: SkillEffect::Striking { dmg: (15, 22), ignore_armor: false, hits: 1, weak: Some(&["holy"]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_shuangji", name: "双影腿法", school: SkillSchool::Wushu,
        desc: "腿影翻飞，攻守两立。", price: 800, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::Striking { dmg: (9, 13), ignore_armor: false, hits: 2, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_suxin", name: "素心剑", school: SkillSchool::Wushu,
        desc: "心剑两利，无视部分甲。", price: 1200, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(40), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(9),
        effect: SkillEffect::Striking { dmg: (18, 24), ignore_armor: false, hits: 1, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_taxue", name: "踏雪无痕", school: SkillSchool::Wushu,
        desc: "踏雪无痕，闪避大涨。", price: 900, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(30), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::SelfBuff { hp: 0, san: 3, guard: 0, dodge_bonus: 0.35, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: true, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_tiebu", name: "铁布衫", school: SkillSchool::Wushu,
        desc: "硬气功常驻，克钝击。", price: 1000, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(30), need_stage: Some(1), need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 6, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_zhenqi", name: "真气护体", school: SkillSchool::Wushu,
        desc: "真气入周天，护体绵密。", price: 1200, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(50), need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.05, dmg_reduce: 0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= L 扩充·血统 (skx_vamp/skx_wolf/skx_zuwu/skx_zhanshi/skx_gauss, 9) =================
    SkillDef {
        id: "skx_vamp_shadow", name: "暗影之潮", school: SkillSchool::Blood,
        desc: "蝠翼穿夜的偷袭技。", price: 2400, need_grade: Some('C'),
        need_bloodline: Some("vampire"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::Striking { dmg: (16, 22), ignore_armor: false, hits: 1, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_vamp_drain", name: "生命汲尽", school: SkillSchool::Blood,
        desc: "渴血三成，克疲敌。", price: 2600, need_grade: Some('C'),
        need_bloodline: Some("vampire"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::Striking { dmg: (12, 18), ignore_armor: false, hits: 1, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_vamp_coffin", name: "血棺沉眠", school: SkillSchool::Blood,
        desc: "退回血棺，把伤势留给黎明清算。", price: 2200, need_grade: Some('C'),
        need_bloodline: Some("vampire"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::SelfBuff { hp: 22, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_wolf_howl", name: "苍狼长啸", school: SkillSchool::Blood,
        desc: "吼摄心魄，敌闪避下降。", price: 2600, need_grade: Some('C'),
        need_bloodline: Some("werewolf"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: None, dmg: (8, 12) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_wolf_primal", name: "野性直觉", school: SkillSchool::Blood,
        desc: "兽性预判，先敌而动。", price: 2200, need_grade: Some('C'),
        need_bloodline: Some("werewolf"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 3, leech: 0, dodge_bonus: 0.05, dmg_reduce: 0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_zuwu_maul", name: "祖巫撼地", school: SkillSchool::Blood,
        desc: "撼地范围，克兽潮。", price: 3600, need_grade: Some('B'),
        need_bloodline: Some("zuwu"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(12),
        effect: SkillEffect::Striking { dmg: (22, 30), ignore_armor: false, hits: 1, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_zuwu_totem", name: "祖巫图腾", school: SkillSchool::Blood,
        desc: "图腾庇佑，坦线续航。", price: 3200, need_grade: Some('B'),
        need_bloodline: Some("zuwu"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 3, san_resist: 0, qi_max: 15, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_zhanshi_bless", name: "圣职祝福", school: SkillSchool::Blood,
        desc: "圣祷安抚，克 SAN 侵蚀。", price: 2200, need_grade: Some('C'),
        need_bloodline: Some("zhanshi_blood"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::SelfBuff { hp: 0, san: 10, guard: 6, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_gauss_sync", name: "纳米同频", school: SkillSchool::Blood,
        desc: "纳米血统与机械共感（预留 gauss_cyber）。", price: 2000, need_grade: Some('C'),
        need_bloodline: Some("gauss_cyber"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 4, leech: 0, dodge_bonus: 0.02, dmg_reduce: 0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= M 扩充·圣光 (skx_holy_*, 8) =================
    SkillDef {
        id: "skx_holy_aura", name: "圣光领域", school: SkillSchool::Holy,
        desc: "圣光庇佑常驻。", price: 2400, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(50), need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 2, san_resist: 6, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_holy_bless", name: "圣辉祝福", school: SkillSchool::Holy,
        desc: "诸事顺遂一回合。", price: 1800, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(20), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(7),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 10, dodge_bonus: 0.10, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_holy_flash", name: "圣光闪", school: SkillSchool::Holy,
        desc: "对亡灵克制一击。", price: 2000, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::Striking { dmg: (18, 24), ignore_armor: false, hits: 1, weak: Some(&["holy"]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_holy_heal", name: "柔和圣光", school: SkillSchool::Holy,
        desc: "圣祷回复。", price: 2200, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(20), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::SelfBuff { hp: 30, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_holy_judge", name: "圣裁之刃", school: SkillSchool::Holy,
        desc: "纯粹圣光，克不死。", price: 3000, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(40), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(12),
        effect: SkillEffect::Striking { dmg: (22, 30), ignore_armor: false, hits: 1, weak: Some(&["holy"]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_holy_purge", name: "至净驱散", school: SkillSchool::Holy,
        desc: "净化主场，清多目标异常。", price: 2600, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(30), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(9),
        effect: SkillEffect::SelfBuff { hp: 0, san: 10, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_holy_resurrect", name: "祈愿圣歌", school: SkillSchool::Holy,
        desc: "临终祷词——致死保留一点生命。", price: 3600, need_grade: Some('B'),
        need_bloodline: None, need_qi: Some(30), need_stage: Some(1), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(15),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_holy_smite", name: "圣言冲击", school: SkillSchool::Holy,
        desc: "驱暗圣言，叠灼垢。", price: 1800, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: Some((6, 3)), dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= N 扩充·科技 (skx_tech_*, 8) =================
    SkillDef {
        id: "skx_tech_beacon", name: "信标锁定", school: SkillSchool::Tech,
        desc: "炮击必中。", price: 2000, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::DebuffEnemy { no_dodge: 2, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 3,
    },
    SkillDef {
        id: "skx_tech_drone", name: "哨戒无人机", school: SkillSchool::Tech,
        desc: "部署棱镜，揭穿隐遁之敌。", price: 2600, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(30), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_tech_emp_grenade", name: "EMP 手雷", school: SkillSchool::Tech,
        desc: "电磁一体化，克机械。", price: 2400, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Item("emp_grenade"),
        effect: SkillEffect::Striking { dmg: (20, 28), ignore_armor: false, hits: 1, weak: Some(&["electric"]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_tech_gauss", name: "高斯点射", school: SkillSchool::Tech,
        desc: "高斯弹头穿甲。", price: 2600, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Item("gauss_battery"),
        effect: SkillEffect::Striking { dmg: (20, 26), ignore_armor: true, hits: 1, weak: Some(&[]) },
        per_fight_uses: Some(2), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_tech_nanocoat", name: "纳米镀膜", school: SkillSchool::Tech,
        desc: "纳米愈合/镀层护体。", price: 2000, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(10), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(5),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 12, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_tech_overcharge", name: "过载强化", school: SkillSchool::Tech,
        desc: "纳米过载，下一击爆增。", price: 2400, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(10), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(9),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 14 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_tech_quantum", name: "量子扰动", school: SkillSchool::Tech,
        desc: "量子纠缠锁敌。", price: 3000, need_grade: Some('B'),
        need_bloodline: None, need_qi: Some(40), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(12),
        effect: SkillEffect::DebuffEnemy { no_dodge: 1, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_tech_rail", name: "电磁弹道", school: SkillSchool::Tech,
        desc: "高斯电磁，克机械。", price: 2200, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::Striking { dmg: (14, 20), ignore_armor: false, hits: 1, weak: Some(&["electric"]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= O 扩充·超能NT (skx_nt_*, 9) =================
    SkillDef {
        id: "skx_nt_bp", name: "黑科技蓝图", school: SkillSchool::Nt,
        desc: "解锁一次强化蓝图。", price: 2000, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Item("blueprint"),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 6 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_nt_hyperspace", name: "超维视界", school: SkillSchool::Nt,
        desc: "四维推演，封死退路。", price: 3600, need_grade: Some('B'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::DebuffEnemy { no_dodge: 2, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 5,
    },
    SkillDef {
        id: "skx_nt_micro", name: "微操弹道", school: SkillSchool::Nt,
        desc: "显微镜手眼，必中连击。", price: 2200, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::Striking { dmg: (12, 18), ignore_armor: false, hits: 2, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_nt_omen", name: "凶兆推演", school: SkillSchool::Nt,
        desc: "预言其必受创。", price: 2000, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 6 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 2,
    },
    SkillDef {
        id: "skx_nt_read", name: "恶意感应", school: SkillSchool::Nt,
        desc: "读意锁敌一回合。", price: 2000, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(4),
        effect: SkillEffect::DebuffEnemy { no_dodge: 1, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 2,
    },
    SkillDef {
        id: "skx_nt_seeker", name: "预感法则", school: SkillSchool::Nt,
        desc: "预知，闪避大涨一回合。", price: 1800, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(4),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.30, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 2,
    },
    SkillDef {
        id: "skx_nt_shield", name: "思维力场", school: SkillSchool::Nt,
        desc: "思维化盾 +25。", price: 3200, need_grade: Some('B'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 25, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_nt_song", name: "灵魂之歌", school: SkillSchool::Nt,
        desc: "歌声回魂。", price: 2000, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::SelfBuff { hp: 16, san: 6, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_nt_sympathy", name: "共鸣同调", school: SkillSchool::Nt,
        desc: "共鸣敌我节奏。", price: 1800, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(5),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 8 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= P 扩充·模因 (skx_meme_*, 7) =================
    SkillDef {
        id: "skx_meme_cursefeed", name: "诅咒回收", school: SkillSchool::Meme,
        desc: "把诅咒逆用为生机。", price: 2600, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(10), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::SelfBuff { hp: 10, san: 4, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_meme_echo", name: "印记回响", school: SkillSchool::Meme,
        desc: "概念传染毗邻目标。", price: 2200, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(10), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_meme_forget", name: "遗忘低语", school: SkillSchool::Meme,
        desc: "侵蚀其战术，令其忘誓。", price: 2600, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(10), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: Some((4, 3)), dmg: (0, 0) },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_meme_mindshield", name: "心膜铸盾", school: SkillSchool::Meme,
        desc: "对抗模因污染。", price: 2000, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 2, san_resist: 5, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_meme_narrate", name: "命名即缚", school: SkillSchool::Meme,
        desc: "它的结局已被写下。", price: 2800, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(9),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_meme_overwrite", name: "覆盖叙事", school: SkillSchool::Meme,
        desc: "划掉『暴走』，改写『停了一瞬』。", price: 3600, need_grade: Some('B'),
        need_bloodline: None, need_qi: Some(20), need_stage: Some(2), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(14),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: Some(1), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_meme_wyrm", name: "概念缠绕", school: SkillSchool::Meme,
        desc: "概念缠足，攻击必中。", price: 2200, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(7),
        effect: SkillEffect::DebuffEnemy { no_dodge: 2, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= Q 扩充·通用 (skx_util_*, 9) =================
    SkillDef {
        id: "skx_util_fieldmed", name: "战场急救", school: SkillSchool::Util,
        desc: "含止血的战场急救。", price: 900, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Item("纱布"),
        effect: SkillEffect::SelfBuff { hp: 18, san: 4, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: Some(2), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_util_heirloom", name: "祖传医术", school: SkillSchool::Util,
        desc: "交换留下的常驻医术。", price: 1200, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 0, san_resist: 0, qi_max: 0, hp_max: 8, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_util_insect", name: "尸味掩体", school: SkillSchool::Util,
        desc: "把自己腌进腐肉味，骗开巡逻。", price: 900, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Item("腐肉"),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.40, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_util_rally", name: "号令集结", school: SkillSchool::Util,
        desc: "鼓舞士气。", price: 1000, need_grade: None,
        need_bloodline: None, need_qi: Some(5), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(4),
        effect: SkillEffect::SelfBuff { hp: 0, san: 5, guard: 0, dodge_bonus: 0.0, atk_flat: 8 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_util_rations", name: "干粮补给", school: SkillSchool::Util,
        desc: "嚼几口干粮，去掉半饥饿。", price: 800, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Item("口粮"),
        effect: SkillEffect::SelfBuff { hp: 20, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_util_retreat", name: "战术撤退", school: SkillSchool::Util,
        desc: "撤出战场，高难保命。", price: 1200, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.50, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_util_scout", name: "敌情摸底", school: SkillSchool::Util,
        desc: "摸底敌情，知己知彼。", price: 800, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: Some(2), cooldown: 0, flying: false, detect_bonus: 4,
    },
    SkillDef {
        id: "skx_util_spotter", name: "鹰眼标记", school: SkillSchool::Util,
        desc: "指挥集火一回合。", price: 800, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::DebuffEnemy { no_dodge: 1, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 6,
    },
    SkillDef {
        id: "skx_util_trap", name: "绊索陷阱", school: SkillSchool::Util,
        desc: "布下绊索，绊其下一步。", price: 900, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Item("绊索"),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 1, dmg_over_time: None, dmg: (6, 10) },
        per_fight_uses: Some(2), cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= R 扩充·修真交叉 (skx_xiu_*, 7) =================
    SkillDef {
        id: "skx_xiu_flags", name: "镇魂旗阵", school: SkillSchool::Xiu,
        desc: "阵法压阵，敌闪避全消。", price: 2800, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(50), need_stage: None, need_cultivation: Some(2),
        kind: SkillKind::Active, cost: SkillCost::Qi(12),
        effect: SkillEffect::DebuffEnemy { no_dodge: 1, stun: 0, dmg_over_time: None, dmg: (6, 10) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_xiu_formation", name: "五行守御阵", school: SkillSchool::Xiu,
        desc: "五行兜底，阵法常驻。", price: 3200, need_grade: Some('B'),
        need_bloodline: None, need_qi: Some(60), need_stage: None, need_cultivation: Some(3),
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 4, san_resist: 4, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_xiu_pill", name: "培元丹", school: SkillSchool::Xiu,
        desc: "丹药疗伤，恢复大档。", price: 1200, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Item("培元丹"),
        effect: SkillEffect::SelfBuff { hp: 40, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: Some(2), cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_xiu_seal", name: "镇山水印", school: SkillSchool::Xiu,
        desc: "法宝镇压狂暴。", price: 3600, need_grade: Some('B'),
        need_bloodline: None, need_qi: Some(30), need_stage: None, need_cultivation: Some(2),
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_xiu_spirit", name: "御灵印", school: SkillSchool::Xiu,
        desc: "临时召灵协同作战。", price: 3000, need_grade: Some('B'),
        need_bloodline: None, need_qi: Some(30), need_stage: Some(1), need_cultivation: Some(2),
        kind: SkillKind::Active, cost: SkillCost::Qi(12),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 12, dodge_bonus: 0.0, atk_flat: 10 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_xiu_talisman", name: "火符·炎爆", school: SkillSchool::Xiu,
        desc: "对再生/生物 ×1.3（弱火）。", price: 1600, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(30), need_stage: None, need_cultivation: Some(1),
        kind: SkillKind::Active, cost: SkillCost::Item("火符"),
        effect: SkillEffect::Striking { dmg: (20, 28), ignore_armor: false, hits: 1, weak: Some(&["fire"]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_xiu_talisman_cling", name: "雷符·天罡", school: SkillSchool::Xiu,
        desc: "对机械/幽魂克制。", price: 1800, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(30), need_stage: None, need_cultivation: Some(1),
        kind: SkillKind::Active, cost: SkillCost::Item("雷符"),
        effect: SkillEffect::Striking { dmg: (18, 26), ignore_armor: false, hits: 1, weak: Some(&["electric", "holy"]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= S 扩充·基因通用 (skx_gene_*, 3) =================
    SkillDef {
        id: "skx_gene_overclock", name: "短暂过载", school: SkillSchool::Gene,
        desc: "把锁率拉到极限——一瞬的刃。", price: 1600, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: Some(1), need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 12 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_gene_regen", name: "锁率再生", school: SkillSchool::Gene,
        desc: "基因锁自愈常驻。", price: 2400, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: Some(2), need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 0, san_resist: 0, qi_max: 0, hp_max: 8, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_gene_sense", name: "锁定反应", school: SkillSchool::Gene,
        desc: "解放锁率的条件反射。", price: 1800, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: Some(1), need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.04, dmg_reduce: 0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    // ================= T 扩充·数据子代理追加（第 4 批 · 15 条，全表唯一） =================
    SkillDef {
        id: "skx_ww_konghou", name: "控鹤擒龙手", school: SkillSchool::Wushu,
        desc: "主神兑换——出手便锁敌身形两回合，武者的手比兵器更早学会不落空。",
        price: 1500, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(40), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::DebuffEnemy { no_dodge: 2, stun: 0, dmg_over_time: None, dmg: (8, 12) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_jinggang", name: "降魔金刚臂", school: SkillSchool::Wushu,
        desc: "主神兑换——臂若金刚，无视护甲直捣，兑换之后便没有砸不开的门。",
        price: 2200, need_grade: Some('C'),
        need_bloodline: None, need_qi: Some(50), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(12),
        effect: SkillEffect::Striking { dmg: (24, 32), ignore_armor: true, hits: 1, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_ww_yanhui", name: "燕回身法", school: SkillSchool::Wushu,
        desc: "主神兑换——身似燕回，下一轮闪避大涨，风里也留不住你的衣角。",
        price: 1300, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(40), need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.30, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_gene_rhythm", name: "锁率呼吸法", school: SkillSchool::Gene,
        desc: "主神兑换——呼吸之间锁率微振，久战不竭的底气。",
        price: 2600, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: Some(2), need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 3, leech: 0, dodge_bonus: 0.0, dmg_reduce: 3, san_resist: 0, qi_max: 0, hp_max: 10, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_blood_acid", name: "强酸喷吐", school: SkillSchool::Blood,
        desc: "主神兑换——异形之喉喷出蚀骨强酸，甲胄也挡不住这口恶念。",
        price: 3000, need_grade: Some('B'),
        need_bloodline: Some("xenomorph_bloodline"), need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::Striking { dmg: (24, 32), ignore_armor: true, hits: 1, weak: Some(&[]) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_holy_ward", name: "圣光壁垒", school: SkillSchool::Holy,
        desc: "主神兑换——圣光凝壁，替你挡下这一轮的全部恶意。",
        price: 2600, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::SelfBuff { hp: 0, san: 4, guard: 15, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_tech_target", name: "制导锁定", school: SkillSchool::Tech,
        desc: "主神兑换——弹头与目标之间只隔一段直线距离，余下的交给制导。",
        price: 1800, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(6),
        effect: SkillEffect::DebuffEnemy { no_dodge: 2, stun: 0, dmg_over_time: None, dmg: (0, 0) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 3,
    },
    SkillDef {
        id: "skx_tech_weave", name: "纳米织网", school: SkillSchool::Tech,
        desc: "主神兑换——纳米丝线织进皮下，钝击落到你身上都会先软三分。",
        price: 2400, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.0, dmg_reduce: 5, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_tech_repair", name: "自动修复单元", school: SkillSchool::Tech,
        desc: "主神兑换——皮下微机械开始工作，破损的血肉被重新焊回原处。",
        price: 2000, need_grade: Some('D'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::SelfBuff { hp: 22, san: 0, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_nt_gravity", name: "重力场压制", school: SkillSchool::Nt,
        desc: "主神兑换——把重力拧成一根看不见的绳索，压得敌人抬不起手。",
        price: 2800, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(10),
        effect: SkillEffect::DebuffEnemy { no_dodge: 0, stun: 1, dmg_over_time: None, dmg: (8, 12) },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_nt_premonition", name: "临危预兆", school: SkillSchool::Nt,
        desc: "主神兑换——危机未至，寒意先至，你总比死亡早一步侧身。",
        price: 2600, need_grade: Some('C'),
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Passive, cost: SkillCost::None,
        effect: SkillEffect::Passive { atk_flat: 0, leech: 0, dodge_bonus: 0.06, dmg_reduce: 0, san_resist: 3, qi_max: 0, hp_max: 0, per_turn_qi: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_util_silent", name: "潜行匿踪", school: SkillSchool::Util,
        desc: "主神兑换——脚步落进影子，杀意也学会了屏息。",
        price: 1000, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::None,
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.35, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_util_brew", name: "简易炼金", school: SkillSchool::Util,
        desc: "主神兑换——把路边的草与铁锈搅成一剂能吊命的药，野外从不缺原料。",
        price: 1100, need_grade: None,
        need_bloodline: None, need_qi: None, need_stage: None, need_cultivation: None,
        kind: SkillKind::Active, cost: SkillCost::Qi(5),
        effect: SkillEffect::SelfBuff { hp: 14, san: 8, guard: 0, dodge_bonus: 0.0, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_xiu_qianji", name: "千机傀儡术", school: SkillSchool::Xiu,
        desc: "主神兑换——木石为骨、灵丝为脉，傀儡替你挡下的每一击都算你的。",
        price: 3000, need_grade: Some('B'),
        need_bloodline: None, need_qi: Some(40), need_stage: None, need_cultivation: Some(2),
        kind: SkillKind::Active, cost: SkillCost::Qi(12),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 14, dodge_bonus: 0.0, atk_flat: 8 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
    SkillDef {
        id: "skx_xiu_dunzhou", name: "遁地咒", school: SkillSchool::Xiu,
        desc: "主神兑换——土行术初窥，一个翻身便从敌人脚下溜走三丈。",
        price: 1800, need_grade: Some('D'),
        need_bloodline: None, need_qi: Some(30), need_stage: None, need_cultivation: Some(1),
        kind: SkillKind::Active, cost: SkillCost::Qi(8),
        effect: SkillEffect::SelfBuff { hp: 0, san: 0, guard: 0, dodge_bonus: 0.40, atk_flat: 0 },
        per_fight_uses: None, cooldown: 0, flying: false, detect_bonus: 0,
    },
];

// ============================================================================
// 查询 helper
// ============================================================================

/// 按 id 查技能（O(n) 线性查找）
pub fn skill(id: &str) -> Option<&'static SkillDef> {
    SKILLS.iter().find(|s| s.id == id)
}

/// 技能「大类」中文名（14 类），供「大类熟练度」记账与前端强化/装备面板分组。
///
/// 归类依据（确定性，按优先级）+ 落点说明：
/// - 学派直挂：Gene→基因类、Tech→科技类、Blood→血统类、Holy→光明魔法类。
/// - Xiu（修真）细分：名字含「剑/御剑/剑丸/万剑」→剑仙类；含火冰雷风炎电心火→元素魔法类；其余→法修类。
/// - Wushu（武道）按名：剑/藏锋→剑类；拳掌腿指爪格斗近战铁臂金钟铁布护体等→体修类；其余→通用类。
/// - 其余学派（Nt 灵能 / Meme 模因 / Util 通用）默认「通用类」，仅个别名命中关键词时归相应类
///   （如「诅咒回收」→黑暗魔法类、「微操弹道」→枪械类）。
/// - 当前技能库无刀/太刀/镰刀与激光/光束/光剑条目，故「刀类」「激光类」暂为空（类别已定义备用）。
pub fn skill_category(sk: &SkillDef) -> &'static str {
    use crate::defs::SkillSchool::*;
    match sk.school {
        Gene => return "基因类",
        Tech => return "科技类",
        Blood => return "血统类",
        Holy => return "光明魔法类",
        Xiu => {
            if contains(sk.name, &["剑", "御剑", "剑丸", "万剑", "飞剑"]) {
                return "剑仙类";
            }
            // 符箓/阵法/法宝术（含 火符/雷符/五行阵）→ 法修类，优先于元素判定。
            // 注意不用裸「丹」作关键字（"丹火"是元素神通，"培元丹"才是丹药），丹药靠兜底法修承接。
            if contains(sk.name, &["符", "阵", "印", "诀", "功", "罡", "术", "禁", "封印", "旗", "图"]) {
                return "法修类";
            }
            if contains(sk.name, &["火", "冰", "雷", "风", "炎", "电", "心火", "丹火", "五行"]) {
                return "元素魔法类";
            }
            return "法修类";
        }
        _ => {}
    }
    let n = sk.name;
    if contains(n, &["剑", "藏锋", "长剑"]) { return "剑类"; }
    if contains(n, &["刀", "太刀", "镰刀"]) { return "刀类"; }
    if contains(n, &["火", "冰", "雷", "风", "炎", "电"]) { return "元素魔法类"; }
    if contains(n, &["暗", "死灵", "诅咒", "堕落", "冥", "亡", "腐"]) { return "黑暗魔法类"; }
    if contains(n, &["枪", "炮", "狙击", "弹药", "弹道", "点射"]) { return "枪械类"; }
    if contains(n, &["激光", "光束", "光剑", "能量"]) { return "激光类"; }
    if contains(n, &["拳", "腿", "掌", "指", "爪", "格斗", "近战", "铁臂", "金钟", "铁布", "护体", "臂", "罩", "衫", "手", "身"]) { return "体修类"; }
    "通用类"
}

/// 名字是否命中任一关键词（子串匹配，中文简称）
fn contains(name: &'static str, keys: &[&str]) -> bool {
    keys.iter().any(|k| name.contains(k))
}

/// 按 st.skills 已购 id 列表查表，返回 SkillDef 引用列表
pub fn skills_owned(st: &crate::state::GameState) -> Vec<&'static SkillDef> {
    st.skills.iter().filter_map(|id| skill(id)).collect()
}

/// 技能可用判定（拥有 && 前置满足）。结算包 B 的 fight_actions 亦据此。
pub fn skill_usable_in_fight(st: &crate::state::GameState, sk: &SkillDef) -> bool {
    if sk.kind != SkillKind::Active {
        return false;
    }
    if let Some(b) = sk.need_bloodline {
        if st.bloodline.as_deref() != Some(b) {
            return false;
        }
    }
    if let Some(g) = sk.need_stage {
        if crate::combat_data::gene_stage_of(st) < g {
            return false;
        }
    }
    if let Some(cs) = sk.need_cultivation {
        if st.cultivation_stage < cs {
            return false;
        }
    }
    if let Some(m) = sk.need_qi {
        if st.qi_max < m {
            return false;
        }
    }
    // 消耗可行性（仅在包 B 执行时真正扣减；此处只判资源上限）
    match &sk.cost {
        SkillCost::Qi(n) if (st.qi as u32) < *n => return false,
        _ => {}
    }
    true
}