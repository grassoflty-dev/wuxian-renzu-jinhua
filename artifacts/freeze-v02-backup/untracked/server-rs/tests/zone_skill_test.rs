//! 3D 副本（zone）技能格结算 — 集成测试
//! 覆盖：engine::do_zone_skill 触发装备技能（差分伤害 + 熟练度 +1）；
//!       slot 越界/未装备 → Err；真气不足 → Err 不扣血；技能击杀 → win。
use wuxian_horror_ch1::engine;
use wuxian_horror_ch1::state::{self, GameState, Mode};

/// 构造一个已进入战斗副本（zone）的初始状态
fn zone_state(fight_ref: &str) -> GameState {
    let mut st = GameState::new();
    st.weapon = Some(state::Weapon::Axe); // 渲染 Fight 视图需持有武器（action_label 会 unwrap）
    st.zone = Some(state::ZoneSession {
        zone_id: "e_licker".to_string(),
        kind: "fight".to_string(),
        ref_id: fight_ref.to_string(),
        zx: 0.0,
        zz: 0.0,
        zyaw: 0.0,
        zhp: st.hp,
        progress: 0,
        last_action: String::new(),
    });
    st
}

/// 造一个不死玩家的「靶子」战斗：敌方 hp=10000（打不死）、dmg(0,0)（打不死玩家）
fn spawn_training_fight(st: &mut GameState, id: &str, hp: i32) {
    st.hp = 10000;
    st.fight = Some(state::Fight {
        id: id.to_string(),
        name: "训练靶".to_string(),
        hp,
        max_hp: hp,
        dmg: (0, 0),
        reward: 0,
        reward_why: String::new(),
        raged: false,
        rage_at: None,
        guard_turn: false,
        pending_log: vec![],
    });
    st.mode = Mode::Fight;
}

/// 装备 1 个主动技能后，do_zone_skill 触发技能 → 敌人血量下降 + skill/category 熟练度 +1。
/// 命中判定有随机性（82% 命中），故循环多次直到命中；熟练度 +1 是确定性行为。
#[test]
fn zone_skill_strikes_and_increments_proficiency() {
    let mut st = zone_state("horde");
    st.skills.push("sk_ww_liantui".to_string()); // 连环腿法（体修类，Active，cost None，Striking 3 hits）
    st.equipped_skills.push("sk_ww_liantui".to_string());
    st.qi = 1000;
    st.qi_max = 1000;
    spawn_training_fight(&mut st, "horde", 10000);

    let mut hit = false;
    for _ in 0..200 {
        engine::do_zone_skill(&mut st, 0).expect("技能应可施展");
        if st.fight.as_ref().unwrap().hp < 10000 {
            hit = true;
            break;
        }
    }
    assert!(hit, "多次施展应至少命中一次");
    assert!(
        st.skill_proficiency
            .get("sk_ww_liantui")
            .copied()
            .unwrap_or(0)
            >= 1,
        "技能熟练度应 +1"
    );
    assert!(
        st.category_proficiency.get("体修类").copied().unwrap_or(0) >= 1,
        "大类熟练度应 +1"
    );
}

/// slot 越界 / 未装备 → Err
#[test]
fn zone_skill_invalid_slot_errors() {
    let mut st = zone_state("horde");
    st.skills.push("sk_ww_liantui".to_string());
    // 未装备任何技能
    assert!(engine::do_zone_skill(&mut st, 0).is_err(), "空装备格应报错");
    // 只装备 1 个，slot 1/2 越界
    st.equipped_skills.push("sk_ww_liantui".to_string());
    assert!(
        engine::do_zone_skill(&mut st, 1).is_err(),
        "越界 slot 应报错"
    );
    assert!(
        engine::do_zone_skill(&mut st, 2).is_err(),
        "越界 slot 应报错"
    );
    assert!(
        engine::do_zone_skill(&mut st, 0).is_ok(),
        "合法 slot 应成功"
    );
}

/// 技能真气不足 → Err，且不扣血、不推进（敌人/玩家血量不变、熟练度不增）
#[test]
fn zone_skill_insufficient_qi_errors_without_damage() {
    let mut st = zone_state("horde");
    st.skills.push("sk_ww_wenxin".to_string()); // 问心一剑：cost Qi(12)，need_qi Some(40)
    st.equipped_skills.push("sk_ww_wenxin".to_string());
    st.qi = 5;
    st.qi_max = 40; // qi_max 满足 need_qi(40)，但当前真气不足 12
    st.hp = 100;
    st.fight = Some(state::Fight {
        id: "horde".to_string(),
        name: "尸潮".to_string(),
        hp: 100,
        max_hp: 100,
        dmg: (0, 0),
        reward: 0,
        reward_why: String::new(),
        raged: false,
        rage_at: None,
        guard_turn: false,
        pending_log: vec![],
    });
    st.mode = Mode::Fight;

    let enemy_hp_before = st.fight.as_ref().unwrap().hp;
    let player_hp_before = st.hp;
    let err = engine::do_zone_skill(&mut st, 0).expect_err("真气不足应报错");
    assert!(
        err.contains("真气不足"),
        "错误信息应说明真气不足，实际：{err}"
    );
    assert_eq!(
        st.fight.as_ref().unwrap().hp,
        enemy_hp_before,
        "敌人血量不变"
    );
    assert_eq!(st.hp, player_hp_before, "玩家血量不变");
    assert_eq!(
        st.skill_proficiency.get("sk_ww_wenxin"),
        None,
        "未施展不应累计熟练度"
    );
}

/// 技能击杀敌人 → 战斗胜利（fight 清空 + 场景不进入死亡结局）
#[test]
fn zone_skill_kill_enemy_returns_win() {
    let mut st = zone_state("horde");
    st.skills.push("sk_ww_wenxin".to_string()); // dmg(30,40) 无视护甲，hp=10 一击必杀
    st.equipped_skills.push("sk_ww_wenxin".to_string());
    st.qi = 1000;
    st.qi_max = 1000;

    let mut won = false;
    for _ in 0..200 {
        spawn_training_fight(&mut st, "horde", 10);
        st.scene_id = "s_office".to_string();
        engine::do_zone_skill(&mut st, 0).expect("技能应可施展");
        if st.fight.is_none() && !st.scene_id.starts_with("e_") {
            won = true;
            break;
        }
    }
    assert!(won, "技能击杀敌人后应判定为胜利");
}
