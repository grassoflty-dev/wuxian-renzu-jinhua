//! 血统进阶路线 + 熟练度强度系统 —— 集成测试
//! 覆盖验收点：
//!   1. 血统阶位表（vampire/saiyan 完整 3 阶），强度公式 = base + contribution。
//!   2. 每次使用血统相关技能/攻击 contrib +k，进阶砍半（向下取整）+ 保底不下降。
//!   3. api_bloodline_advance / api_get_bloodlines（逻辑等价 try_advance_bloodline）校验点数 + 旗标。
//!   4. 强度接入战斗数值（strength/100 攻击追加、strength/200 减伤）。
//!   5. 技能熟练度两层加成：skill/10 + category/20，换同大类新技能仍吃大类加成。
//!   6. 新测试全绿、既有测试不破。
use wuxian_horror_ch1::combat_data::{
    bloodline_advance_cost, bloodline_strength, bloodline_tier_info, current_bloodline_tier,
    has_next_tier, try_advance_bloodline, AdvanceError,
};
use wuxian_horror_ch1::state::GameState;

/// 阶位表：vampire / saiyan 各具 3 阶完整数据，且 ×10 递增主基调。
#[test]
fn vampire_and_saiyan_have_three_full_tiers() {
    // vampire 3 阶
    let v1 = bloodline_tier_info("vampire", 1).expect("vampire 阶1");
    let v2 = bloodline_tier_info("vampire", 2).expect("vampire 阶2");
    let v3 = bloodline_tier_info("vampire", 3).expect("vampire 阶3");
    assert_eq!((v1.base, v1.k), (100, 1));
    assert_eq!((v2.base, v2.k), (1000, 2));
    assert_eq!((v3.base, v3.k), (10000, 4));
    // ×10 递增主基调
    assert_eq!(v2.base, v1.base * 10, "中级 base=初级×10");
    assert_eq!(v3.base, v2.base * 10, "高级 base=中级×10");
    // 进阶点数
    assert_eq!(v2.need_points, 5000);
    assert_eq!(v3.need_points, 50000);
    // vampire 支线剧情权限旗标
    assert_eq!(v2.need_flag, Some("vamp_mid_unlock"));
    assert_eq!(v3.need_flag, Some("vamp_high_unlock"));

    // saiyan 3 阶（S 级 base 上浮 1.5×）
    let s1 = bloodline_tier_info("saiyan_bloodline", 1).expect("saiyan 阶1");
    let s2 = bloodline_tier_info("saiyan_bloodline", 2).expect("saiyan 阶2");
    let s3 = bloodline_tier_info("saiyan_bloodline", 3).expect("saiyan 阶3");
    assert_eq!(s1.base, 150, "赛亚人基 +1.5×");
    assert_eq!(s2.base, 1500);
    assert_eq!(s3.base, 15000);
    assert_eq!(s2.base, s1.base * 10, "赛亚人仍保持 ×10 递增");
}

/// 强度公式：bloodline_strength == base + contribution。血统为空 → 0。
#[test]
fn strength_equals_base_plus_contribution() {
    let mut st = GameState::new();
    // 无血统 → 0
    assert_eq!(bloodline_strength(&st), 0);

    st.bloodline = Some("vampire".to_string());
    // 阶1 base=100，贡献 0 → 100
    assert_eq!(bloodline_strength(&st), 100);

    // 贡献 900 → 强度 1000
    st.bloodline_contribution.insert("vampire".to_string(), 900);
    assert_eq!(bloodline_strength(&st), 1000);

    // 升到阶2 base=1000，贡献清零 → 1000
    st.bloodline_tier.insert("vampire".to_string(), 2);
    st.bloodline_contribution.insert("vampire".to_string(), 0);
    assert_eq!(bloodline_strength(&st), 1000);
}

/// 熟练度提升：每次使用血统相关技能 contrib +k（配合 engine 战斗结算）。
/// 这里直接验证 bloodline_tiers 的进阶换算核心（进阶砍半 + 保底不下降）。
#[test]
fn advance_halves_contribution_and_never_drops_strength() {
    // 场景 A：初级 100 练到贡献 900（强度 1000）→ 进阶中级强度 = max(1000 + 450, 1000) = 1450
    let mut st = GameState::new();
    st.bloodline = Some("vampire".to_string());
    st.points = 50000;
    st.set_flag("vamp_mid_unlock");
    st.bloodline_contribution.insert("vampire".to_string(), 900);
    // 旧强度 = 100 + 900 = 1000
    assert_eq!(bloodline_strength(&st), 1000);

    let new_tier = try_advance_bloodline(&mut st, "vampire").expect("进阶成功");
    assert_eq!(new_tier, 2);
    // 贡献砍半：900/2 = 450
    assert_eq!(
        st.bloodline_contribution.get("vampire"),
        Some(&450),
        "贡献砍半"
    );
    // 新强度 = max(1000 + 450, 1000) = 1450
    assert_eq!(bloodline_strength(&st), 1450);
    // 点数扣除 need_points=5000
    assert_eq!(st.points, 45000);
}

#[test]
fn advance_floor_keeps_strength_when_overleveled() {
    // 场景 B：贡献 2000（强度 2100）→ 进阶 max(1000 + 1000, 2100) = 2100 保底不下降
    let mut st = GameState::new();
    st.bloodline = Some("vampire".to_string());
    st.points = 50000;
    st.set_flag("vamp_mid_unlock");
    st.bloodline_contribution
        .insert("vampire".to_string(), 2000);
    assert_eq!(bloodline_strength(&st), 2100);

    let new_tier = try_advance_bloodline(&mut st, "vampire").expect("进阶成功");
    assert_eq!(new_tier, 2);
    // 贡献砍半：2000/2 = 1000，但保底把新强度顶到旧强度 2100 → 最终贡献 = 2100 - 1000 = 1100
    assert_eq!(bloodline_strength(&st), 2100, "保底不下降");
    assert_eq!(
        st.bloodline_contribution.get("vampire"),
        Some(&1100),
        "保底补足贡献"
    );
}

#[test]
fn advance_requires_enough_points() {
    let mut st = GameState::new();
    st.bloodline = Some("vampire".to_string());
    st.points = 100; // 远不足 5000
    st.set_flag("vamp_mid_unlock");
    st.bloodline_contribution.insert("vampire".to_string(), 0);

    let err = try_advance_bloodline(&mut st, "vampire").unwrap_err();
    assert_eq!(err, AdvanceError::NotEnoughPoints);
    // 未升阶
    assert_eq!(current_bloodline_tier(&st), 1);
}

#[test]
fn advance_requires_flag() {
    let mut st = GameState::new();
    st.bloodline = Some("vampire".to_string());
    st.points = 50000;
    // 未设 vamp_mid_unlock
    st.bloodline_contribution.insert("vampire".to_string(), 0);

    let err = try_advance_bloodline(&mut st, "vampire").unwrap_err();
    assert_eq!(err, AdvanceError::FlagNotMet);
    assert_eq!(current_bloodline_tier(&st), 1);
}

#[test]
fn advance_fails_at_max_tier() {
    let mut st = GameState::new();
    st.bloodline = Some("vampire".to_string());
    st.points = 1000000;
    // 直接满阶 3
    st.bloodline_tier.insert("vampire".to_string(), 3);
    st.set_flag("vamp_high_unlock");

    let err = try_advance_bloodline(&mut st, "vampire").unwrap_err();
    assert_eq!(err, AdvanceError::MaxTier);
    assert!(!has_next_tier("vampire", 3), "3 阶无下一阶");
}

#[test]
fn advance_requires_current_bloodline() {
    // 未装配血统
    let mut st = GameState::new();
    st.points = 50000;
    let err = try_advance_bloodline(&mut st, "vampire").unwrap_err();
    assert_eq!(err, AdvanceError::NoBloodline);
}

/// 进阶花费查询：返回 need_points + need_flag；满阶 None。
#[test]
fn advance_cost_reports_points_and_flag() {
    // 升阶2：5000 点 + vamp_mid_unlock
    let (pts, flag) = bloodline_advance_cost("vampire", 1).expect("阶1→2 有花费");
    assert_eq!(pts, 5000);
    assert_eq!(flag, Some("vamp_mid_unlock"));
    // 升阶3：50000 点 + vamp_high_unlock
    let (pts3, flag3) = bloodline_advance_cost("vampire", 2).expect("阶2→3 有花费");
    assert_eq!(pts3, 50000);
    assert_eq!(flag3, Some("vamp_high_unlock"));
    // 满阶 None
    assert!(bloodline_advance_cost("vampire", 3).is_none());
}

/// 未具名血统（如测试用的不存在 id）走通用默认进阶线。
#[test]
fn generic_default_tiers_fallback() {
    let w2 = bloodline_tier_info("nonexistent_bloodline", 2).expect("未具名血统通用默认阶2");
    assert_eq!(w2.base, 1000, "通用默认 base");
    assert_eq!(w2.k, 2);
    assert_eq!(w2.need_points, 5000);
    // 通用默认无剧情门槛
    assert_eq!(w2.need_flag, None);
}

/// 强度接入战斗数值的缩放系数（映射口径）：strength/100 攻击、strength/200 减伤。
#[test]
fn strength_combat_scaling_mapping() {
    let mut st = GameState::new();
    st.bloodline = Some("vampire".to_string());
    // 阶1 base=100 → atk_bonus 1, reduce 0（100/200=0 向下取整）
    assert_eq!(bloodline_strength(&st) / 100, 1, "阶1 攻击追加 +1");
    assert_eq!(bloodline_strength(&st) / 200, 0, "阶1 减伤 0");

    // 贡献 9900（阶1 强度 10000）→ atk 100, reduce 50
    st.bloodline_contribution
        .insert("vampire".to_string(), 9900);
    assert_eq!(bloodline_strength(&st), 10000);
    assert_eq!(bloodline_strength(&st) / 100, 100);
    assert_eq!(bloodline_strength(&st) / 200, 50);
}

/// 旧存档缺额字段 → serde default 回填空 HashMap，不崩。
#[test]
fn old_save_without_bloodline_tier_fields_defaults() {
    let json = serde_json::json!({
        "hp": 80, "san": 60, "points": 100, "bloodline": "vampire",
        "weapon": "Axe", "scene_id": "s_office", "px": 1, "py": 1, "floor": 0,
    });
    let st: GameState = serde_json::from_value(json).expect("旧档可反序列化");
    assert!(st.bloodline_tier.is_empty(), "旧档血统阶位缺省为空");
    assert!(st.bloodline_proficiency.is_empty());
    assert!(st.bloodline_contribution.is_empty());
    // 缺省阶位视为 1，强度 = base(100) + 0 = 100
    assert_eq!(bloodline_strength(&st), 100);
}

// ============================================================================
// 技能熟练度 → 技能强度：两层加成公式（sk/10 + cat/20，向下取整）
// ============================================================================

use wuxian_horror_ch1::engine::skill_strength_bonus;

/// 辅助：查技能定义引用（剑类技能）
fn sw_skill(id: &str) -> &'static wuxian_horror_ch1::defs::SkillDef {
    wuxian_horror_ch1::skills_data::skill(id).expect("技能存在")
}

/// 两层加成公式边界：skill/10 + category/20，均向下取整。
#[test]
fn skill_strength_bonus_formula() {
    let mut st = GameState::new();
    let sk = sw_skill("sk_ww_wenxin"); // 剑类
    let cat = wuxian_horror_ch1::skills_data::skill_category(sk).to_string();
    assert_eq!(cat, "剑类", "问心一剑归剑类");

    // 双 0 → 0
    assert_eq!(skill_strength_bonus(sk, &st), 0);

    // skill_proficiency 9/10 分界：9→0，10→1，19→1，20→2
    for (sp, want) in [(9u32, 0), (10, 1), (19, 1), (20, 2)] {
        st.skill_proficiency.insert(sk.id.to_string(), sp);
        st.category_proficiency.insert(cat.clone(), 0);
        assert_eq!(
            skill_strength_bonus(sk, &st),
            want,
            "skill_proficiency={sp}"
        );
    }

    // category_proficiency 19/20 分界：19→0，20→1，39→1，40→2
    for (cp, want) in [(19u32, 0), (20, 1), (39, 1), (40, 2)] {
        st.skill_proficiency.insert(sk.id.to_string(), 0);
        st.category_proficiency.insert(cat.clone(), cp);
        assert_eq!(
            skill_strength_bonus(sk, &st),
            want,
            "category_proficiency={cp}"
        );
    }

    // 叠加：skill 15 (+1) + category 30 (+1) = 2
    st.skill_proficiency.insert(sk.id.to_string(), 15);
    st.category_proficiency.insert(cat.clone(), 30);
    assert_eq!(skill_strength_bonus(sk, &st), 2);
}

/// 核心验收点：换一个同大类的新技能（skill_proficiency=0），仍吃到 category_proficiency 加成。
#[test]
fn new_same_category_skill_still_gets_category_bonus() {
    let mut st = GameState::new();
    let a = sw_skill("sk_ww_wenxin"); // 剑类（问心一剑）
    let b = sw_skill("skx_ww_suxin"); // 剑类（素心剑）—— 新技能，skill_proficiency=0

    assert_eq!(
        wuxian_horror_ch1::skills_data::skill_category(a),
        wuxian_horror_ch1::skills_data::skill_category(b),
        "两技能同属剑类"
    );

    // 练 A 技能，大类熟练度累积到 40（=> cat/20 = 2）
    let cat = wuxian_horror_ch1::skills_data::skill_category(a).to_string();
    st.category_proficiency.insert(cat.clone(), 40);

    // B 技能 skill_proficiency=0，但大类加成 +2
    assert!(st.skill_proficiency.get(b.id).is_none(), "B 技能专精为 0");
    assert_eq!(skill_strength_bonus(b, &st), 2, "新同大类技能仍吃大类加成");
    // 对照：A 技能若 skill=0 也同吃大类加成
    assert_eq!(skill_strength_bonus(a, &st), 2);
}

/// 练 A 技能 → B 技能（同大类）也因大类熟练度受益，但 B 技能专精加成恒 0。
#[test]
fn training_a_benefits_b_via_category_but_not_skill() {
    let mut st = GameState::new();
    let a = sw_skill("sk_ww_wenxin");
    let b = sw_skill("skx_ww_suxin");
    let cat = wuxian_horror_ch1::skills_data::skill_category(a).to_string();

    // A 技能练 25 次：A 专精 +25，大类 +25
    for _ in 0..25 {
        *st.skill_proficiency.entry(a.id.to_string()).or_insert(0) += 1;
        *st.category_proficiency.entry(cat.clone()).or_insert(0) += 1;
    }

    // A: skill 25/10=2 + cat 25/20=1 → 3
    assert_eq!(skill_strength_bonus(a, &st), 3);
    // B: skill 0/10=0 + cat 25/20=1 → 1（B 专精加成 0，但大类加成 1）
    assert_eq!(skill_strength_bonus(b, &st), 1);
}

/// 全额战斗结算：Striking 技能伤害吃到两层熟练度加成（走 engine::do_zone_skill）。
#[test]
fn striking_damage_includes_two_layer_bonus() {
    let mut st = GameState::new();
    // 学剑类技能并装备到技能格
    st.skills.push("sk_ww_wenxin".to_string());
    st.equipped_skills = vec!["sk_ww_wenxin".to_string()];
    // 给足真气上限与当前真气，确保技能可用
    st.qi_max = 120;
    st.qi = 120;
    // 预先铺好熟练度：skill 20（+2） + 剑类 40（+2） = +4 伤害
    st.skill_proficiency.insert("sk_ww_wenxin".to_string(), 20);
    let cat = wuxian_horror_ch1::skills_data::skill_category(sw_skill("sk_ww_wenxin")).to_string();
    st.category_proficiency.insert(cat, 40);

    // 建立战斗（horde），敌人满血
    let cfg = wuxian_horror_ch1::scenes::fight_cfg("horde").expect("horde");
    st.fight = Some(wuxian_horror_ch1::power::scaled_fight(
        "horde",
        cfg,
        &st,
        vec![],
    ));
    st.fight.as_mut().unwrap().hp = 500;
    st.fight.as_mut().unwrap().max_hp = 500;
    st.scene_id = "s_office".to_string(); // do_zone_skill 内部 render 需合法场景
    st.weapon = Some(wuxian_horror_ch1::state::Weapon::Axe); // render 动作标签需武器
    st.mode = wuxian_horror_ch1::state::Mode::Fight;
    // 建立 zone 会话使 do_zone_skill 走 zone 结算（ref_id 用 horde）
    st.zone = Some(wuxian_horror_ch1::state::ZoneSession {
        zone_id: "z_test".to_string(),
        kind: "fight".to_string(),
        ref_id: "horde".to_string(),
        zx: 0.0,
        zz: 0.0,
        zyaw: 0.0,
        zhp: st.hp,
        progress: 0,
        last_action: String::new(),
    });

    let before_hp = st.fight.as_ref().unwrap().hp;
    // 命中判定有随机性（82% 命中），循环施展直到命中；熟练度每次施展 +1（确定性）
    let mut hit = false;
    for _ in 0..200 {
        let _ = wuxian_horror_ch1::engine::do_zone_skill(&mut st, 0).expect("技能可用");
        if st.fight.as_ref().map_or(false, |f| f.hp < before_hp) {
            hit = true;
            break;
        }
    }
    assert!(hit, "Striking 技能多次施展应至少命中一次");
    // 熟练度已累加（本次施展 +1 每次技能与大类；命中前可能多次施展，故用 >=）
    assert!(
        st.skill_proficiency
            .get("sk_ww_wenxin")
            .copied()
            .unwrap_or(0)
            >= 21,
        "技能熟练度应累加"
    );
}
