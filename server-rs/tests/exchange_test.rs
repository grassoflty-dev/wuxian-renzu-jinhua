//! 主神兑换集成测试：武器/护甲/饰品/血统/技能买入后 state 变化
//! （points 扣减；equipment/skills/bloodline/inventory 增加）。
//! 复用 engine::goto/choose 走完整场景流转，风格同 nexus_exchange.rs。
//! 文件名称 exchange_test.rs。
use wuxian_horror_ch1::engine;
use wuxian_horror_ch1::power;
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::state::{GameState, Mode};
use wuxian_horror_ch1::worlds;

fn pick(st: &GameState, keyword: &str) -> i32 {
    let scene = scenes::scene(&st.scene_id).expect("scene");
    let visible: Vec<_> = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(st))).collect();
    for (i, c) in visible.iter().enumerate() {
        if c.label.contains(keyword) {
            return i as i32;
        }
    }
    panic!("scene {} 未找到「{}」；可见: {:?}",
        st.scene_id, keyword,
        visible.iter().map(|c| c.label).collect::<Vec<_>>());
}

/// 进入兑换主目录（s_nexus_exchange）
fn go_exchange(st: &mut GameState) {
    st.world_id = worlds::WORLD_ZHUTIAN.to_string();
    let mut deaths = vec![];
    engine::goto(st, "s_nexus_exchange", &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange");
}

/// 进入道具铺
fn go_shop(st: &mut GameState) {
    go_exchange(st);
    let mut deaths = vec![];
    let idx = pick(st, "道具铺");
    engine::choose(st, idx, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_shop");
}

/// 用例1：购买武器精锻武士刀（wp_katana, 1200·C）→ points 扣减 + equipment.weapon 装配 + 攻击提升
#[test]
fn buy_weapon_updates_equipment_and_power() {
    let mut st = GameState::new();
    st.points = 5000;
    st.sp_grade = Some('C');
    go_shop(&mut st);

    let before_atk = power::weapon_atk(&st); // 无武器默认 6
    let idx = pick(&st, "精锻武士刀");
    let mut deaths = vec![];
    engine::choose(&mut st, idx, &mut deaths);

    assert_eq!(st.scene_id, "s_nexus_exchange_done", "应兑换成功");
    assert_eq!(st.points, 5000 - 1200, "扣 1200 点");
    let w = st.equipment.weapon.as_ref().expect("应装配武器");
    assert_eq!(w.id, "wp_katana");
    assert_eq!(w.enhance, 0, "新武器默认 +0");
    assert!(power::weapon_atk(&st) > before_atk, "武器攻击提升");
    assert!(matches!(st.mode, Mode::Normal));
}

/// 用例2：购买护甲警用防弹背心（gear_police_vest, 900·D）→ equipment.armor 装配 + 扣点
#[test]
fn buy_armor_updates_equipment() {
    let mut st = GameState::new();
    st.points = 3000;
    st.sp_grade = Some('D');
    go_shop(&mut st);

    let idx = pick(&st, "警用防弹背心");
    let mut deaths = vec![];
    engine::choose(&mut st, idx, &mut deaths);

    assert_eq!(st.scene_id, "s_nexus_exchange_done");
    assert_eq!(st.points, 3000 - 900, "扣 900 点");
    assert_eq!(st.equipment.armor.as_deref(), Some("gear_police_vest"), "护甲槽装配");
}

/// 用例3：购买饰品蛮力指环（access_strength_ring, 1200·D）→ equipment.accessory 装配 + 攻击加成
#[test]
fn buy_accessory_updates_equipment_and_power() {
    let mut st = GameState::new();
    st.points = 4000;
    st.sp_grade = Some('D');
    let p0 = power::power(&st);
    go_shop(&mut st);

    let idx = pick(&st, "蛮力指环");
    let mut deaths = vec![];
    engine::choose(&mut st, idx, &mut deaths);

    assert_eq!(st.scene_id, "s_nexus_exchange_done");
    assert_eq!(st.points, 4000 - 1200, "扣 1200 点");
    assert_eq!(st.equipment.accessory.as_deref(), Some("access_strength_ring"));
    assert_eq!(power::equipped_atk_flat(&st), 6, "饰品攻击加成 6");
    assert_eq!(power::power(&st), p0 + 6, "power 随饰品上升");
}

/// 用例4：购买血统初级吸血鬼（3000）→ bloodline 写入 + points 扣减 + agi_bonus+1
#[test]
fn buy_bloodline_sets_state() {
    let mut st = GameState::new();
    st.points = 5000;
    go_exchange(&mut st);

    let idx = pick(&st, "初级吸血鬼血统");
    let mut deaths = vec![];
    engine::choose(&mut st, idx, &mut deaths);

    assert_eq!(st.scene_id, "s_nexus_exchange_done");
    assert_eq!(st.points, 5000 - 3000, "扣 3000 点");
    assert_eq!(st.bloodline.as_deref(), Some("vampire"), "血统写入");
    assert_eq!(st.agi_bonus, 1, "附赠敏捷 +1");
}

/// 用例5：购买技能洞察侦查（sk_util_inspect, 800）→ skills 增加 + points 扣减
#[test]
fn buy_skill_updates_skills() {
    let mut st = GameState::new();
    st.points = 5000;
    go_exchange(&mut st);

    // 技能秘藏 → 通用 → 洞察侦查
    let mut deaths = vec![];
    let sk = pick(&st, "技能秘藏");
    engine::choose(&mut st, sk, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_skill");
    let util = pick(&st, "通用");
    engine::choose(&mut st, util, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_skcat_util");

    let idx = pick(&st, "洞察侦查");
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_done");
    assert_eq!(st.points, 5000 - 800, "技能扣 800 点");
    assert!(st.skills.contains(&"sk_util_inspect".to_string()), "技能入库");
}

/// 用例6：血统/技能购买二次防重复（技能已购 → fail 不扣点）
#[test]
fn buy_skill_duplicate_rejected() {
    let mut st = GameState::new();
    st.points = 5000;
    st.skills.push("sk_util_inspect".to_string());
    go_exchange(&mut st);

    let mut deaths = vec![];
    let sk = pick(&st, "技能秘藏");
    engine::choose(&mut st, sk, &mut deaths);
    let util = pick(&st, "通用");
    engine::choose(&mut st, util, &mut deaths);

    let idx = pick(&st, "洞察侦查");
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_fail", "已购技能二次购买应失败");
    assert_eq!(st.points, 5000, "不重复扣点");
}

/// 用例7：点数不足购买武器 → fail，不装配、不扣点
#[test]
fn buy_underfunded_rejected() {
    let mut st = GameState::new();
    st.points = 100; // 不足 1200
    st.sp_grade = Some('C');
    go_shop(&mut st);

    let idx = pick(&st, "精锻武士刀");
    let mut deaths = vec![];
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_fail");
    assert_eq!(st.points, 100, "不足不扣点");
    assert!(st.equipment.weapon.is_none(), "不装配");
}

/// 用例8：评级门槛不足购买高级武器 → fail（需 B，仅 C）
#[test]
fn buy_grade_gated_rejected() {
    let mut st = GameState::new();
    st.points = 100_000;
    st.sp_grade = Some('C'); // 电磁脉冲枪需 B
    go_shop(&mut st);

    // 电磁脉冲枪 wp_emi 需 B → C 时买不到（选项仍可见，但 route 内拒）
    let idx = pick(&st, "电磁脉冲枪");
    let mut deaths = vec![];
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_fail", "评级不足应失败");
    assert!(st.equipment.weapon.is_none());
    assert_eq!(st.points, 100_000, "不扣点");
}