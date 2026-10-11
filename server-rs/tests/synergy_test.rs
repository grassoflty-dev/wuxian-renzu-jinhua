//! 组合联动（Synergy）系统集成测试：规则表数据完整性 + 检测确定性 + 伤害/吸血结算。
//! 任一类子断言都只走公开 API（synergies::{SYNERGIES, synergy_check, synergy_flat,
//!   synergy_leech, synergy_line}）与真实装配（bloodline / equipment / cultivation / skills）。
//! 文件名称 synergy_test.rs（任务约定词尾）。
use wuxian_horror_ch1::defs::WeaponSlot;
use wuxian_horror_ch1::synergies::{
    SynergyDef, SynergyEffect, SYNERGIES, synergy_check, synergy_flat, synergy_leech,
    synergy_line, synergy_matches,
};
use wuxian_horror_ch1::combat_data::bloodline_def;
use wuxian_horror_ch1::items_data::{treasure_def, weapon_def};
use wuxian_horror_ch1::state::{GameState, Mode, Weapon};
use wuxian_horror_ch1::{engine, scenes, power};

// 便捷构造：装备主手武器 + 法宝，返回 st。
fn build(bloodline: Option<&str>, weapon: Option<&str>, treasure: Option<&str>) -> GameState {
    let mut st = GameState::new();
    st.bloodline = bloodline.map(|s| s.to_string());
    if let Some(w) = weapon {
        st.equipment.weapon = Some(WeaponSlot { id: w.to_string(), enhance: 0 });
    }
    if let Some(t) = treasure {
        let slot = treasure_def(t).map(|td| td.slot as usize).unwrap_or(0);
        st.equipment.treasure[slot.min(2)] = Some(t.to_string());
    }
    st
}

// ============================================================================
// 1. 数据完整性：每一条联动规则的 id 都在对应静态表真实存在
// ============================================================================
#[test]
fn every_synergy_ids_are_real() {
    assert!(SYNERGIES.len() >= 6, "联动规则应 ≥6 条，实际 {}", SYNERGIES.len());
    for d in SYNERGIES {
        if let Some(bl) = d.bloodline {
            assert!(bloodline_def(bl).is_some(), "联动 {} 引用的血统 {} 应存在", d.id, bl);
        }
        if let Some(w) = d.weapon {
            assert!(weapon_def(w).is_some(), "联动 {} 引用的武器 {} 应存在", d.id, w);
        }
        if let Some(t) = d.treasure {
            assert!(treasure_def(t).is_some(), "联动 {} 引用的法宝 {} 应存在", d.id, t);
        }
    }
}

/// 至少存在 6 条独立血统联动（本轮设计 10 条）
#[test]
fn at_least_six_bloodline_rules() {
    let blood_rules = SYNERGIES.iter().filter(|d| d.bloodline.is_some()).count();
    assert!(blood_rules >= 6, "血统联动应 ≥6 条，实际 {blood_rules}");
}

// ============================================================================
// 2. 检测确定性：命中 / 未命中 均由装配决定，纯函数可断言
// ============================================================================
#[test]
fn dragon_ball_echo_detects() {
    // 未装配血统时不应命中
    let none = build(None, None, Some("tr_longzu_shengyi"));
    assert!(synergy_check(&none).is_empty());
    // 只配血统、缺法宝 → 不命中
    let no_t = build(Some("saiyan_bloodline"), None, None);
    assert!(synergy_check(&no_t).is_empty());
    // 双条件齐备 → 唯一命中龙珠共鸣
    let st = build(Some("saiyan_bloodline"), None, Some("tr_longzu_shengyi"));
    let hit = synergy_check(&st);
    assert_eq!(hit.len(), 1);
    assert_eq!(hit[0].id, "dragon_ball_echo");
    assert_eq!(synergy_flat(&st), 10);
    assert_eq!(synergy_leech(&st), 0);
}

#[test]
fn shinigami_zanjing_detects() {
    let mut st = build(Some("shinigami_bloodline"), Some("wp_zanjingdao_he"), None);
    assert_eq!(synergy_check(&st)[0].id, "shinigami_zanjing");
    assert_eq!(synergy_flat(&st), 14);
    // 换一把非斩魄刀武器 → 不再命中
    st.equipment.weapon = Some(WeaponSlot { id: "wp_axe".to_string(), enhance: 0 });
    assert!(synergy_check(&st).is_empty());
}

#[test]
fn vampire_feast_flat_and_leech() {
    let st = build(Some("vampire"), Some("wpn_bloodsaber"), None);
    let hit = synergy_check(&st);
    assert!(!hit.is_empty());
    assert_eq!(synergy_flat(&st), 6);
    assert_eq!(synergy_leech(&st), 4);
    assert!(hit.iter().any(|d| d.id == "vampire_feast"));
}

/// 修真境界门槛：yujian_benming 需要 cultivation_stage>=1
#[test]
fn yujian_benming_requires_cultivation() {
    let mut st = build(None, None, Some("cu_bab_benming_fejian"));
    st.cultivation_stage = 0;
    assert!(synergy_check(&st).is_empty(), "未修真不应触发御剑");
    st.cultivation_stage = 2;
    let hit = synergy_check(&st);
    assert_eq!(hit.len(), 1);
    assert_eq!(hit[0].id, "yujian_benming");
    assert_eq!(synergy_flat(&st), 8);
    assert_eq!(synergy_leech(&st), 4);
}

/// 多条件联动：龙裔·屠龙之心需血统+武器+法宝三者齐备
#[test]
fn dragon_kill_requires_three() {
    let two = build(Some("dragon_bloodline"), Some("wp_dragon_lance"), None);
    assert!(synergy_check(&two).is_empty(), "缺法宝时不命中");
    let three = build(Some("dragon_bloodline"), Some("wp_dragon_lance"), Some("tr_bahuang_longyin"));
    let hit = synergy_check(&three);
    assert_eq!(hit.len(), 1);
    assert_eq!(hit[0].id, "dragon_kill");
    assert_eq!(synergy_flat(&three), 16);
}

/// synergy_matches 对空武器/血统装配稳妥返回 false，不 panic
#[test]
fn synergy_matches_never_panics_on_empty() {
    let st = GameState::new();
    for d in SYNERGIES {
        let _ = synergy_matches(&st, d);
    }
    // 引用一条规则实体做显式断言
    let sample: &SynergyDef = &SYNERGIES[0];
    assert!(!synergy_matches(&st, sample));
}

// ============================================================================
// 3. synergy_line 文案带 synergy 样式标签
// ============================================================================
#[test]
fn synergy_line_has_special_markup() {
    let st = build(Some("demon_bloodline"), None, Some("tr_mo_jie_jiujie"));
    let def = synergy_check(&st)[0];
    let (flat, leech, line) = synergy_line(def);
    let _ = flat; let _ = leech;
    assert!(line.contains("<span class='synergy'>"), "联动日志应带 synergy 样式: {line}");
    assert!(line.contains(def.msg), "联动日志应含规则文案: {line}");
}

/// effect 数值可读性：同步枚举行为（flat/leech 访问器）
#[test]
fn effect_accessors() {
    assert_eq!(SynergyEffect::FlatDmg(7).flat(), 7);
    assert_eq!(SynergyEffect::FlatDmg(7).leech(), 0);
    assert_eq!(SynergyEffect::Leech(3).flat(), 0);
    assert_eq!(SynergyEffect::Leech(3).leech(), 3);
    assert_eq!(SynergyEffect::FlatDmgLeech(5, 2).flat(), 5);
    assert_eq!(SynergyEffect::FlatDmgLeech(5, 2).leech(), 2);
}

// ============================================================================
// 4. 集成：真实 fight_turn 一回合命中后，联动日志出现 + 敌人血量下降
// ============================================================================
/// 反复普攻，直到至少出现一次命中（命中率 0.82，50 次内几乎必然命中），
/// 断言出现 `<span class='synergy'>` 日志，并确认命中那回合敌人 HP 减少。
#[test]
fn fight_turn_fires_synergy_on_hit() {
    // 大筒木 + 北冥吸星剑：联动追加招 10 + 吸血 6，最利于观测伤害下降
    let mut st = build(Some("otsutsuki_bloodline"), Some("wp_beiming_jian"), None);
    st.weapon = Some(Weapon::Sword); // 主手攻击动作走军刀（多段）
    st.world_id = wuxian_horror_ch1::worlds::WORLD_BIOHAZARD.to_string();
    st.scaling_enabled = false;
    // 直接进入一场真实战斗（horde 表：fight_cfg 可解析；hp 抬到 500 以便多刀观测非击杀命中）
    let cfg = scenes::fight_cfg("horde").expect("horde");
    st.fight = Some(power::scaled_fight("horde", cfg, &st, vec![]));
    st.fight.as_mut().unwrap().hp = 500;
    st.fight.as_mut().unwrap().max_hp = 500;
    st.mode = Mode::Fight;

    let mut saw_synergy_log = false;
    let mut saw_hp_drop = false;
    let mut fights = 0;
    while st.fight.is_some() && fights < 400 {
        fights += 1;
        let cur_hp = st.fight.as_ref().map(|f| f.hp);
        // 每次取「普攻」动作的下标（finisher 出现时它会插在最前，不能依赖固定 0）
        let atk_idx = engine::fight_actions(&st).iter().position(|a| a == "attack");
        let atk_idx = atk_idx.expect("应有普攻动作");
        engine::choose(&mut st, atk_idx as i32, &mut vec![]);
        let new_hp = st.fight.as_ref().map(|f| f.hp);
        let logs: Vec<String> = st.fight.as_ref().map(|f| f.pending_log.clone()).unwrap_or_default();
        if logs.iter().any(|l| l.contains("<span class='synergy'>")) {
            saw_synergy_log = true;
        }
        if let (Some(a), Some(b)) = (cur_hp, new_hp) {
            if b < a { saw_hp_drop = true; }
        }
        // 敌人死亡后 fight_win 会清空 fight；若仍存活则继续下一刀
    }
    assert!(saw_synergy_log, "攻击命中时应出现组合联动 synergy 日志");
    assert!(saw_hp_drop, "联动追加伤害命中回合应使敌人 HP 下降");
}

// ============================================================================
// 5. 血统 id 与规则血统字段一致（防手误拼错 id）
// ============================================================================
#[test]
fn bloodline_ids_in_rules_match_table() {
    // 规则里用到的每个血统都能在表里取回同名被动（间接保证 id 拼写正确）
    for d in SYNERGIES {
        if let Some(bl) = d.bloodline {
            let b = bloodline_def(bl).expect(bl);
            assert_eq!(b.id, bl, "规则血统 {} 应与表内 id 一致", bl);
        }
    }
}
