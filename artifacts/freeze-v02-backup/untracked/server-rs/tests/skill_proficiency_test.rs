//! 技能熟练度系统 + 3 技能格装备/切换 + 技能大类（category）— 集成测试
//! 覆盖：使用技能后 skill_proficiency/category_proficiency +1；
//!       熟练度分级 proficiency_tier；
//!       技能大类 skill_category（14 类中文名）；
//!       装备格契约（equipped_skills ≤3 与合法校验）；
//!       旧存档 serde default 迁移兼容（缺字段不崩）。
use wuxian_horror_ch1::defs;
use wuxian_horror_ch1::engine;
use wuxian_horror_ch1::power;
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::skills_data::{self, skill_category};
use wuxian_horror_ch1::state::{GameState, Mode};

/// 熟练度分级：0-9=0、10-29=1、30-59=2、60+=3
#[test]
fn proficiency_tier_boundaries() {
    assert_eq!(defs::proficiency_tier(0), 0);
    assert_eq!(defs::proficiency_tier(9), 0);
    assert_eq!(defs::proficiency_tier(10), 1);
    assert_eq!(defs::proficiency_tier(29), 1);
    assert_eq!(defs::proficiency_tier(30), 2);
    assert_eq!(defs::proficiency_tier(59), 2);
    assert_eq!(defs::proficiency_tier(60), 3);
    assert_eq!(defs::proficiency_tier(1000), 3, "60+ 恒为 3 级");
}

/// 技能大类：四大学派直挂 + 修真/武道关键词细分
#[test]
fn skill_category_rules() {
    // 学派直挂
    assert_eq!(skill_category(sk("sk_gene_regen")), "基因类");
    assert_eq!(skill_category(sk("sk_tech_nano")), "科技类");
    assert_eq!(skill_category(sk("sk_vamp_frenzy")), "血统类");
    assert_eq!(skill_category(sk("sk_holy_burst")), "光明魔法类");
    // 修真：御剑→剑仙类，火雷→元素魔法类，其余→法修类
    assert_eq!(skill_category(sk("cu_shen_jianyu")), "剑仙类");
    assert_eq!(skill_category(sk("cu_shen_zhangxin")), "元素魔法类"); // 掌心雷
    assert_eq!(skill_category(sk("cu_jin_dingji")), "法修类"); // 定身符
                                                               // 武道：剑/体修细分
    assert_eq!(skill_category(sk("sk_ww_wenxin")), "剑类"); // 问心一剑
    assert_eq!(skill_category(sk("sk_ww_liantui")), "体修类"); // 连环腿法
    assert_eq!(skill_category(sk("sk_util_inspect")), "通用类"); // 通用
    assert_eq!(skill_category(sk("skx_meme_cursefeed")), "黑暗魔法类"); // 诅咒回收
}

fn sk(id: &str) -> &'static wuxian_horror_ch1::defs::SkillDef {
    skills_data::skill(id).expect("skill exists")
}

/// 飞行/浮空标签：御剑/御风/腾云/轻功/化雾浮空 → flying=true；其余 false
#[test]
fn flying_tag_on_flight_skills() {
    assert!(sk("cu_shen_dunjian").flying, "御剑遁法");
    assert!(sk("cu_shen_yufeng").flying, "御风行");
    assert!(sk("cu_shen_tiandai").flying, "天地同游");
    assert!(sk("skx_ww_taxue").flying, "踏雪无痕");
    assert!(sk("sk_vamp_mist").flying, "血雾遁形");
    // 非飞行技能保持 false
    assert!(!sk("sk_ww_liantui").flying, "连环腿法不飞");
    assert!(!sk("sk_gene_regen").flying);
    // 全表 flying 恰好 5 个
    let n = skills_data::SKILLS.iter().filter(|s| s.flying).count();
    assert_eq!(n, 5, "flying 技能数应为 5，实际 {n}");
}

/// 远程探测视野加成：鹰眼/扫描/侦查/感应/预知等 detect_bonus>0；其余 0
#[test]
fn detect_bonus_on_scouting_skills() {
    let cases: Vec<(&str, u32)> = vec![
        ("skx_util_spotter", 6),  // 鹰眼标记
        ("skx_nt_hyperspace", 5), // 超维视界
        ("sk_util_inspect", 4),   // 洞察侦查
        ("skx_util_scout", 4),    // 敌情摸底
        ("sk_tech_scanner", 3),   // 战术扫描
        ("sk_nt_precog", 3),      // 灾祸预知
        ("skx_tech_beacon", 3),   // 信标锁定
        ("sk_gene_scan", 2),      // 心灵扫描
        ("sk_nt_empathy", 2),     // 读心感应
        ("skx_nt_read", 2),       // 恶意感应
        ("skx_nt_seeker", 2),     // 预感法则
        ("skx_nt_omen", 2),       // 凶兆推演
        ("sk_gene_focus", 2),     // 猎手凝视
    ];
    for (id, want) in cases {
        assert_eq!(sk(id).detect_bonus, want, "{id} detect_bonus 应为 {want}");
    }
    // 非探测技能为 0
    assert_eq!(sk("sk_ww_liantui").detect_bonus, 0);
    assert_eq!(sk("sk_gene_regen").detect_bonus, 0);
    // 全表 detect_bonus>0 恰好 13 个
    let n = skills_data::SKILLS
        .iter()
        .filter(|s| s.detect_bonus > 0)
        .count();
    assert_eq!(n, 14, "detect_bonus>0 技能数应为 14，实际 {n}");
}

/// 全表 146 条技能都能归入 14 类之一（无 panics，返回值非空）
#[test]
fn every_skill_has_nonempty_category() {
    for s in skills_data::SKILLS {
        let c = skill_category(s);
        assert!(!c.is_empty(), "{} 归类不应为空", s.id);
    }
    assert_eq!(skills_data::SKILLS.len(), 161, "技能总数保持 161");
}

/// 新状态三字段默认空
#[test]
fn new_state_has_empty_proficiency_and_equipped() {
    let st = GameState::new();
    assert!(st.skill_proficiency.is_empty());
    assert!(st.category_proficiency.is_empty());
    assert!(st.equipped_skills.is_empty());
}

/// 旧存档缺省三字段 → serde default 回填空，不崩（migrate 兼容）
#[test]
fn old_save_without_proficiency_fields_migrates() {
    let json = serde_json::json!({
        "hp": 80, "san": 60, "points": 100,
        "skills": ["sk_util_inspect"],
        "weapon": "Axe", "scene_id": "s_office",
        "px": 1, "py": 1, "floor": 0,
    });
    let mut st: GameState = serde_json::from_value(json).expect("旧档可反序列化");
    assert!(st.skill_proficiency.is_empty(), "旧档技能熟练度缺省为空");
    assert!(st.category_proficiency.is_empty(), "旧档大类熟练度缺省为空");
    assert!(st.equipped_skills.is_empty(), "旧档装备格缺省为空");
    let n = wuxian_horror_ch1::state::migrate_save(&mut st);
    assert_eq!(n, 0, "无 explored 旧键时迁移返回 0");
    assert!(
        st.skill_proficiency.is_empty(),
        "迁移后仍为空（无历史数据）"
    );
}

/// 使用主动技能后：skill_proficiency +1 且 category_proficiency（该大类）+1
#[test]
fn using_skill_increments_skill_and_category_proficiency() {
    let mut st = GameState::new();
    // 学习一个零成本主动技能 sk_util_inspect（通用类，Active，cost None）
    st.skills.push("sk_util_inspect".to_string());
    assert!(st.category_proficiency.get("通用类").is_none());

    let cfg = scenes::fight_cfg("horde").expect("horde");
    st.fight = Some(power::scaled_fight("horde", cfg, &st, vec![]));
    st.fight.as_mut().unwrap().hp = 500;
    st.fight.as_mut().unwrap().max_hp = 500;
    st.mode = Mode::Fight;

    // 动作键为 "sk_<skill_id>"，id 本身含 sk_ 前缀 → "sk_sk_util_inspect"
    let idx = engine::fight_actions(&st)
        .iter()
        .position(|a| a == "sk_sk_util_inspect")
        .expect("应能列出 sk_util_inspect 动作");
    engine::choose(&mut st, idx as i32, &mut vec![]);

    assert_eq!(
        st.skill_proficiency.get("sk_util_inspect"),
        Some(&1),
        "技能熟练度应为 1"
    );
    assert_eq!(
        st.category_proficiency.get("通用类"),
        Some(&1),
        "通用类熟练度应为 1"
    );
}

/// 连续使用两次（体修类技能）：技能与「体修类」大类熟练度均累计到 2
#[test]
fn repeated_use_accumulates_category_proficiency() {
    let mut st = GameState::new();
    st.skills.push("sk_ww_liantui".to_string()); // 连环腿法（体修类，cost None，Active）

    for _ in 0..2 {
        let cfg = scenes::fight_cfg("horde").expect("horde");
        st.fight = Some(power::scaled_fight("horde", cfg, &st, vec![]));
        st.fight.as_mut().unwrap().hp = 500;
        st.fight.as_mut().unwrap().max_hp = 500;
        st.mode = Mode::Fight;
        let idx = engine::fight_actions(&st)
            .iter()
            .position(|a| a == "sk_sk_ww_liantui")
            .expect("动作存在");
        engine::choose(&mut st, idx as i32, &mut vec![]);
        if st.fight.is_some() {
            st.fight = None;
        }
        st.mode = Mode::Normal;
    }

    assert_eq!(st.skill_proficiency.get("sk_ww_liantui"), Some(&2));
    assert_eq!(
        st.category_proficiency.get("体修类"),
        Some(&2),
        "体修类熟练度累计到 2"
    );
}

/// 装备格契约：equipped_skills 最多 3 个且全为主动技能
#[test]
fn equipped_skills_holds_at_most_three_active() {
    let mut st = GameState::new();
    st.skills = vec![
        "sk_util_inspect".to_string(),
        "sk_util_morale".to_string(),
        "sk_ww_liantui".to_string(),
    ];
    st.equipped_skills = vec![
        "sk_util_inspect".to_string(),
        "sk_util_morale".to_string(),
        "sk_ww_liantui".to_string(),
    ];
    assert!(st.equipped_skills.len() <= 3, "最多 3 个");
    for sid in &st.equipped_skills {
        let s = wuxian_horror_ch1::skills_data::skill(sid).expect("技能存在");
        assert_eq!(
            s.kind,
            wuxian_horror_ch1::defs::SkillKind::Active,
            "{sid} 应为主动"
        );
    }
}

/// hud_json 含 equipped_skills 与 skill/category 熟练度字段
#[test]
fn hud_contains_equipped_and_proficiency() {
    let mut st = GameState::new();
    st.skills.push("sk_ww_liantui".to_string());
    st.equipped_skills.push("sk_ww_liantui".to_string());
    st.skill_proficiency.insert("sk_ww_liantui".to_string(), 5);
    st.category_proficiency.insert("体修类".to_string(), 5);

    let hud = engine::hud_json(&st);
    assert_eq!(hud["equipped_skills"], serde_json::json!(["sk_ww_liantui"]));
    assert_eq!(
        hud["skill_proficiency"]["sk_ww_liantui"],
        serde_json::json!(5)
    );
    assert_eq!(hud["category_proficiency"]["体修类"], serde_json::json!(5));
}
