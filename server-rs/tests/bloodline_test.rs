//! 血统系统集成测试：BLOODLINES 九条血统在表 + 被动数值与字段一致 + bloodline_of 装配读取。
//! 纯函数走 combat_data 公开查询（bloodline_def / bloodline_of），数据完整性断言。
//! 文件名称 bloodline_test.rs（任务约定词尾）。
use wuxian_horror_ch1::combat_data::{bloodline_def, bloodline_of};
use wuxian_horror_ch1::defs::BLOODLINES;
use wuxian_horror_ch1::state::GameState;

/// 九条血统全部注册（vampire/werewolf/zuwu/zhanshi_blood/gauss_cyber/angel/demon/dragon/cyber）
#[test]
fn all_9_bloodlines_registered() {
    let expected: &[&str] = &[
        "vampire", "werewolf", "zuwu", "zhanshi_blood", "gauss_cyber",
        "angel_bloodline", "demon_bloodline", "dragon_bloodline", "cyber_prosthetic",
    ];
    for id in expected {
        assert!(bloodline_def(id).is_some(), "血统 {id} 应注册于 BLOODLINES");
    }
    // 表内条数 ≥9（血统持续扩充，不硬编码精确数量）
    assert!(BLOODLINES.len() >= 9, "BLOODLINES 应 ≥9 条，实际 {}", BLOODLINES.len());
}

/// 表索引自洽：bloodline_def(id) 能取回同名血统定义（字段一致）
#[test]
fn bloodline_def_backs_the_table() {
    for def in BLOODLINES {
        let got = bloodline_def(def.id).expect(def.id);
        assert_eq!(got.id, def.id, "bloodline_def 返回同条血统");
        assert_eq!(got.name, def.name, "bloodline_def 名称一致");
    }
}

/// vampire 被动基数（设计：吸血4 / 受击减3）
#[test]
fn vampire_passive_base() {
    let b = bloodline_def("vampire").unwrap();
    assert_eq!(b.passive.leech_on_hit, 4, "吸血鬼命中吸血 4");
    assert_eq!(b.passive.dmg_reduce, 3, "吸血鬼受击减 3");
    assert_eq!(b.passive.atk_flat, 0);
    assert_eq!(b.passive.dodge_bonus, 0.0);
}

/// werewolf 被动（攻+8 / 狂暴+10 / 减2 / 闪+0.05）
#[test]
fn werewolf_passive() {
    let b = bloodline_def("werewolf").unwrap();
    assert_eq!(b.passive.atk_flat, 8);
    assert_eq!(b.passive.rage_bonus_atk, 10);
    assert_eq!(b.passive.dmg_reduce, 2);
    assert!((b.passive.dodge_bonus - 0.05).abs() < 1e-9);
}

/// zuwu 高坦（受击减10）
#[test]
fn zuwu_passive_tank() {
    let b = bloodline_def("zuwu").unwrap();
    assert_eq!(b.passive.dmg_reduce, 10);
    assert_eq!(b.passive.atk_flat, 0);
}

/// zhanshi 圣职（SAN抗+8）
#[test]
fn zhanshi_san_resist() {
    let b = bloodline_def("zhanshi_blood").unwrap();
    assert_eq!(b.passive.san_resist, 8);
}

/// gauss_cyber 纳米（攻+4 / 减6）
#[test]
fn gauss_cyber_passive() {
    let b = bloodline_def("gauss_cyber").unwrap();
    assert_eq!(b.passive.atk_flat, 4);
    assert_eq!(b.passive.dmg_reduce, 6);
}

/// angel 天使（SAN抗+12 / 减4）
#[test]
fn angel_passive() {
    let b = bloodline_def("angel_bloodline").unwrap();
    assert_eq!(b.passive.san_resist, 12);
    assert_eq!(b.passive.dmg_reduce, 4);
}

/// demon 恶魔（攻+12 / 吸血6 / 狂暴+15）
#[test]
fn demon_passive() {
    let b = bloodline_def("demon_bloodline").unwrap();
    assert_eq!(b.passive.atk_flat, 12);
    assert_eq!(b.passive.leech_on_hit, 6);
    assert_eq!(b.passive.rage_bonus_atk, 15);
}

/// dragon 龙族（攻+6 / 受击减14）
#[test]
fn dragon_passive() {
    let b = bloodline_def("dragon_bloodline").unwrap();
    assert_eq!(b.passive.atk_flat, 6);
    assert_eq!(b.passive.dmg_reduce, 14);
}

/// cyber_prosthetic 机械义体（攻+8 / 减8 / 闪+0.08）
#[test]
fn cyber_prosthetic_passive() {
    let b = bloodline_def("cyber_prosthetic").unwrap();
    assert_eq!(b.passive.atk_flat, 8);
    assert_eq!(b.passive.dmg_reduce, 8);
    assert!((b.passive.dodge_bonus - 0.08).abs() < 1e-9);
}

/// bloodline_of：装配后读回；未装配 None
#[test]
fn bloodline_of_reads_assignment() {
    let mut st = GameState::new();
    assert!(bloodline_of(&st).is_none(), "未装配血统为 None");
    st.bloodline = Some("vampire".to_string());
    assert_eq!(bloodline_of(&st).unwrap().id, "vampire");
    st.bloodline = Some("dragon_bloodline".to_string());
    assert_eq!(bloodline_of(&st).unwrap().id, "dragon_bloodline");
}

/// 每条血统都有 label（HUD/兑换文案）——数据完整性
#[test]
fn every_bloodline_has_label() {
    for def in BLOODLINES {
        assert!(!def.passive.label.is_empty(), "血统 {}", def.id);
    }
}

/// 被动数值自洽：至少一条正向加成（攻击/吸血/减伤/闪避/SAN抗至少占一）
#[test]
fn passive_never_all_zero() {
    for def in BLOODLINES {
        let p = &def.passive;
        let any =
            p.atk_flat != 0 || p.leech_on_hit != 0 || p.dmg_reduce != 0
            || p.san_resist != 0 || p.rage_bonus_atk != 0 || p.dodge_bonus.abs() > 1e-9;
        assert!(any, "血统 {} 被动应至少一项非零", def.id);
    }
}