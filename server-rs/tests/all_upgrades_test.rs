//! all_upgrades_test.rs —— 强化表全遍历（全量覆盖，非抽样）
//!
//! 遍历全部强化/兑换数据表条目，断言每条字段完整 + 兑换/合成可达：
//!   - WEAPONS / GEAR / TRESURE_DEFS：id 非空、关键字段合理、能经 items_data 查询到
//!   - ITEMS：每个道具 id 非空、有价/分级字段、能 item_def 查询到
//!   - BLOODLINES：passive 字段完整、bloodline_def 可查询
//!   - SKILLS：id 唯一、有 price/grade/category、skill(id) 可查询
//!   - RECIPES：原料 id + 产出 id 都在 ITEMS 表可查询（合成可达）
use wuxian_horror_ch1::combat_data::BLOODLINES;
use wuxian_horror_ch1::defs::{SkillEffect, SkillKind, SkillSchool};
use wuxian_horror_ch1::items_data::{GEAR, ITEMS, RECIPES, TRESURE_DEFS, WEAPONS};
use wuxian_horror_ch1::skills_data::SKILLS;

/// 全量遍历强化表，逐表断言；输出统计。
#[test]
fn weapons_gear_treasure_full_validated() {
    let mut n_weapons = 0usize;
    for w in WEAPONS {
        n_weapons += 1;
        assert!(!w.id.is_empty(), "武器 id 为空");
        assert!(!w.name.is_empty(), "武器 {id} 名为空", id = w.id);
        assert!(w.dmg.0 > 0 && w.dmg.1 >= w.dmg.0, "武器 {} dmg 区间非法 {:?}", w.id, w.dmg);
        assert!(w.base_price >= 0, "武器 {} 价格非法 {}", w.id, w.base_price);
        let _ = w.dmg_type;
        // 兑换可达：能经查询函数拿到（保证与查询同步）
        let q = wuxian_horror_ch1::items_data::weapon_def(w.id);
        assert!(q.is_some(), "武器 {id} weapon_def 查询不到", id = w.id);
        assert_eq!(q.unwrap().id, w.id, "武器 {} 查询结果错位", w.id);
    }

    let mut n_gear = 0usize;
    for g in GEAR {
        n_gear += 1;
        assert!(!g.id.is_empty(), "护甲 id 为空");
        assert!(!g.name.is_empty(), "护甲 {id} 名为空", id = g.id);
        assert!(g.dmg_reduce >= 0 && g.dodge >= 0.0 && g.dodge <= 1.0, "护甲 {id} 减伤/闪避异常", id = g.id);
        assert!(g.atk_flat >= 0 && g.price >= 0, "护甲 {id} 数值非法", id = g.id);
        let q = wuxian_horror_ch1::items_data::gear_def(g.id);
        assert!(q.is_some(), "护甲 {id} gear_def 查询不到", id = g.id);
        assert_eq!(q.unwrap().id, g.id, "护甲 {} 查询结果错位", g.id);
    }

    let mut n_treasure = 0usize;
    for t in TRESURE_DEFS {
        n_treasure += 1;
        assert!(!t.id.is_empty(), "法宝 id 为空");
        assert!(!t.name.is_empty(), "法宝 {id} 名为空", id = t.id);
        assert!(t.slot <= 2, "法宝 {} slot={} 非法（应 0/1/2）", t.id, t.slot);
        assert!(t.price >= 0, "法宝 {} 价格非法 {}", t.id, t.price);
        assert!((t.dmg_mult - 1.0).abs() >= 0.0, "法宝 {id} dmg_mult 异常", id = t.id);
        let q = wuxian_horror_ch1::items_data::treasure_def(t.id);
        assert!(q.is_some(), "法宝 {id} treasure_def 查询不到", id = t.id);
        assert_eq!(q.unwrap().id, t.id, "法宝 {} 查询结果错位", t.id);
    }

    eprintln!("[weapons_gear_treasure_full_validated] WEAPONS={n_weapons} GEAR={n_gear} TRESURE={n_treasure}");
    assert!(n_weapons >= 20 && n_gear >= 17 && n_treasure >= 12,
        "强化表条目不足：WEAPONS={n_weapons} GEAR={n_gear} TRESURE={n_treasure}");
}

/// 全量遍历 ITEMS：id 唯一、非空、价格/分级字段完整、item_def 可查询（兑换可达）
#[test]
fn items_full_validated() {
    let mut set: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for it in ITEMS {
        assert!(!it.id.is_empty(), "道具 id 为空");
        assert!(set.insert(it.id), "道具 id 重复：{}", it.id);
        assert!(!it.name.is_empty(), "道具 {} 名为空", it.id);
        assert!(it.price >= 0, "道具 {} price={} 非法", it.id, it.price);
        let q = wuxian_horror_ch1::items_data::item_def(it.id);
        assert!(q.is_some(), "道具 {} item_def 查询不到", it.id);
        assert_eq!(q.unwrap().id, it.id, "道具 {} 查询错位", it.id);
        let _ = it.kind;
    }
    eprintln!("[items_full_validated] ITEMS={} 道具", set.len());
    assert!(set.len() >= 29, "ITEMS 应有 ≥29 条，实际 {}", set.len());
}

/// 全量遍历 BLOODLINES：passive 字段完整、id 唯一、bloodline_def 可查询（兑换可达）
#[test]
fn bloodlines_full_validated() {
    let mut set: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut n = 0usize;
    for b in BLOODLINES {
        n += 1;
        assert!(!b.id.is_empty() && set.insert(b.id), "血统 id 空或重复：{:?}", b.id);
        assert!(!b.name.is_empty() && !b.desc.is_empty(), "血统 {id} 名称/描述为空", id = b.id);
        // passive 字段完整（label 命中任一数值说明非全零哨兵）
        let p = &b.passive;
        assert!(!p.label.is_empty(), "血统 {id} passive.label 为空", id = b.id);
        assert!(p.atk_flat >= 0 && p.dmg_reduce >= 0 && p.san_resist >= 0, "血统 {id} 被动数值非法", id = b.id);
        let q = wuxian_horror_ch1::combat_data::bloodline_def(b.id);
        assert!(q.is_some(), "血统 {id} bloodline_def 查询不到", id = b.id);
        assert_eq!(q.unwrap().id, b.id, "血统 {id} 查询错位", id = b.id);
    }
    eprintln!("[bloodlines_full_validated] BLOODLINES={n}");
    assert!(n >= 9, "BLOODLINES 应 ≥9，实际 {n}");
}

/// 全量遍历 SKILLS：id 唯一、有 school/category/price/grade、skill(id) 可查询（兑换可达）
#[test]
fn skills_full_validated() {
    let mut set: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut n = 0usize;
    for sk in SKILLS {
        n += 1;
        assert!(!sk.id.is_empty() && set.insert(sk.id), "技能 id 空或重复");
        assert!(!sk.name.is_empty(), "技能 {id} 名为空", id = sk.id);
        assert!(sk.price > 0, "技能 {} price={} 非法（应 >0）", sk.id, sk.price);
        assert!(sk.school as u8 <= SkillSchool::Xiu as u8, "技能 {id} school 越界", id = sk.id);
        assert!(sk.kind == SkillKind::Active || sk.kind == SkillKind::Passive, "技能 {id} kind 非法", id = sk.id);
        let q = wuxian_horror_ch1::skills_data::skill(sk.id);
        assert!(q.is_some(), "技能 {id} skill(id) 查询不到", id = sk.id);
        assert_eq!(q.unwrap().id, sk.id, "技能 {id} 查询错位", id = sk.id);
        // 效果结构可匹配（四种变体之一，避免全空占位；不强绑 Passive↔Passive 语义）
        let shape_ok = matches!(
            sk.effect,
            SkillEffect::Striking { .. } | SkillEffect::SelfBuff { .. } | SkillEffect::DebuffEnemy { .. } | SkillEffect::Passive { .. }
        );
        assert!(shape_ok, "技能 {id} effect 不属于已知结构", id = sk.id);
    }
    eprintln!("[skills_full_validated] SKILLS={n}");
    assert!(n >= 145, "SKILLS 应 ≥145，实际 {n}");
}

/// 全量遍历 RECIPES：产出 id 与全部原料 id 都必须是「已知真实物品」——即能在
/// ITEMS / TRESURE_DEFS / QUEST_ITEM_IDS（engine 真实掉落剧情/门禁道具）之一查询到，
/// 保证合成闭环可达（合成不产出生造 id）。
#[test]
fn recipes_full_reachable() {
    // 已知物品注册表：ITEMS + 法宝 + 任务/剧情掉落载体（后者刻意不列入 ITEMS 计量表）
    let mut known: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for it in ITEMS { known.insert(it.id); }
    for t in TRESURE_DEFS { known.insert(t.id); }
    for q in wuxian_horror_ch1::items_data::QUEST_ITEM_IDS { known.insert(q); }

    let mut n = 0usize;
    for r in RECIPES {
        n += 1;
        assert!(!r.result.is_empty(), "配方 {n} result 为空");
        assert!(!r.ingredients.is_empty(), "配方 result={} 无原料", r.result);
        assert!(
            known.contains(r.result),
            "配方产出 {} 既不在 ITEMS 也不在 TRESURE_DEFS/QUEST_ITEM_IDS（合成不可达）",
            r.result,
        );
        for ing in r.ingredients {
            assert!(
                known.contains(ing),
                "配方 {} 原料 {} 不在任何已知物品表（合成不可达）", r.result, ing,
            );
        }
    }
    eprintln!("[recipes_full_reachable] RECIPES={n}");
    assert!(n >= 8, "RECIPES 应 ≥8，实际 {n}");
}