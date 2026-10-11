//! 装备系统集成测试：装备穿戴（weapon/armor/accessory/treasure 装配）→ 角色加成变化；
//! 武器强化 +N 后战力（power）变化。
//! 复用公开观测口径：power::weapon_atk / power::equipped_atk_flat / power::power。
//! 文件名称 equipment_test.rs（任务约定词尾）。
use wuxian_horror_ch1::defs::{Equipment, WeaponSlot};
use wuxian_horror_ch1::engine;
use wuxian_horror_ch1::power;
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::state::GameState;
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

/// 装配主手武器协议 id：weapon_atk 读 highest（equipment.weapon 优先），无则回落旧 Weapon 枚举/缺省 6
#[test]
fn weapon_equip_changes_weapon_atk() {
    let mut st = GameState::new();
    assert_eq!(power::weapon_atk(&st), 6, "无武器缺省 6");

    // 高斯手枪 dmg(20,30) 中值 25
    st.equipment.weapon = Some(WeaponSlot { id: "wp_gauss".to_string(), enhance: 0 });
    assert_eq!(power::weapon_atk(&st), 25, "高斯中值 (20+30)/2");

    // 换更高 tier → weapon_atk 继续涨
    st.equipment.weapon = Some(WeaponSlot { id: "wp_quantum_core".to_string(), enhance: 0 }); // dmg(34,50) → 42
    assert_eq!(power::weapon_atk(&st), 42, "量子核心振荡剑中值 (34+50)/2");
}

/// 武器强化 +N：power::weapon_atk 与 power 随 enhance 上升（设计 item §4.3：dmg 每级 +2/+3）
#[test]
fn weapon_enhance_raises_power() {
    let mut st = GameState::new();
    st.equipment.weapon = Some(WeaponSlot { id: "wp_gauss".to_string(), enhance: 0 });
    let p0 = power::power(&st);
    let a0 = power::weapon_atk(&st);
    assert_eq!(a0, 25);

    // 强化 +2
    st.equipment.weapon.as_mut().unwrap().enhance = 2;
    let a2 = power::weapon_atk(&st);
    assert!(a2 > a0, "强化 +2 后武器攻击应更高（当前 a0={a0} a2={a2}）");
    assert_eq!(power::power(&st), p0 + (a2 - a0), "power 随强化同步上升");
}

/// 护甲/饰品装备：atk_flat 加成并入 power（equipped_atk_flat 汇总护甲+饰品+法宝三格）
#[test]
fn armor_accessory_atk_flat_raises_power() {
    let mut st = GameState::new();
    let base = power::equipped_atk_flat(&st);
    assert_eq!(base, 0, "空装备攻击加成 0");

    // 精灵斗篷 armor：dmg_reduce5 dodge0.05（无 atk_flat）→ equipped_atk_flat 不变但 slot 落位
    st.equipment.armor = Some("gear_elven_cloak".to_string());
    // 蛮力指环 accessory：atk_flat 6
    st.equipment.accessory = Some("access_strength_ring".to_string());
    assert_eq!(power::equipped_atk_flat(&st), 6, "饰品攻击加成 6");
    assert!(power::power(&st) > power::power(&GameState::new()), "装备后 power 上升");

    // 护甲有 atk_flat 的（秘银 —— 无；用纳米核心 accessory 606）请参考格；此处校验累加
    st.equipment.armor = Some("gear_nano_vest".to_string()); // atk 0
    assert_eq!(power::equipped_atk_flat(&st), 6, "护甲无 atk 时保持饰品加成");
}

/// 法宝装配（equipment.treasure[slot]）→ equipped_atk_flat 累加（设计 §2.2 装配权威）
#[test]
fn treasure_equip_raises_equipped_atk_flat() {
    let mut st = GameState::new();
    st.equipment.treasure[0] = Some("cu_bab_benming_fejian".to_string()); // 本命飞剑 atk_flat 8
    assert_eq!(power::equipped_atk_flat(&st), 8, "法宝格攻击加成 8");

    st.equipment.treasure[1] = Some("tr_taixu_shield".to_string()); // 太虚玄光镜 atk_flat 0
    st.equipment.treasure[2] = Some("cu_bab_hunyuan_lu".to_string()); // 混元炉 atk_flat 0
    assert_eq!(power::equipped_atk_flat(&st), 8, "防御/辅助法宝不计攻击");
    let p = power::power(&st);
    assert!(p > power::power(&GameState::new()), "装配法宝使 power 上升");
}

/// 集成：置入护甲/饰品/法宝三格后 power 显著增长，且与拆分口径一致
#[test]
fn full_equipment_raises_power() {
    let mut st = GameState::new();
    let p0 = power::power(&st);
    st.equipment.weapon = Some(WeaponSlot { id: "wp_gauss".to_string(), enhance: 1 });
    st.equipment.armor = Some("gear_kevlar".to_string());
    st.equipment.accessory = Some("access_strength_ring".to_string());
    st.equipment.treasure[0] = Some("tr_zhuxian_calendar".to_string()); // atk_flat 16
    st.equipment.treasure[1] = Some("cu_bab_hudun_fu".to_string());

    let wa = power::weapon_atk(&st);
    let ea = power::equipped_atk_flat(&st);
    assert!(wa > 6, "高级武器拉高 weapon_atk");
    assert_eq!(ea, 6 + 16, "饰品6 + 法宝16 攻击加成");
    assert!(power::power(&st) > p0, "全套装备使 power 上升");
}

/// 结构完整性：equipment 各槽位类型字段存在且可构造（编译期契约）
#[test]
fn equipment_struct_shape() {
    let mut eq = Equipment::default();
    assert!(eq.weapon.is_none() && eq.armor.is_none() && eq.accessory.is_none());
    assert_eq!(eq.treasure, [None, None, None]);
    eq.weapon = Some(WeaponSlot { id: "wp_katana".to_string(), enhance: 3 });
    eq.armor = Some("gear_police_vest".to_string());
    eq.accessory = Some("access_agility_boots".to_string());
    eq.treasure[0] = Some("cu_bab_benming_fejian".to_string());
    let mut st = GameState::new();
    st.equipment = eq;
    assert_eq!(st.equipment.weapon.as_ref().unwrap().enhance, 3);
}

/// 集成：主神道具铺购买法宝（本命飞剑·青锋）→ equipment.treasure[slot] 装配 + equipped_atk_flat 生效
/// （bug 回归：此前法宝只入 st.treasures 拥有标记，未写入装配权威 equipment.treasure，加成不生效）
#[test]
fn buy_treasure_wires_into_equipment_treasure() {
    let mut st = GameState::new();
    st.points = 20_000;
    st.sp_grade = Some('C'); // cu_bab_benming_fejian 需 C
    st.world_id = worlds::WORLD_ZHUTIAN.to_string();
    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_exchange", &mut deaths);
    let shop = pick(&st, "道具铺");
    engine::choose(&mut st, shop, &mut deaths);

    let idx = pick(&st, "本命飞剑·青锋");
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_done", "购买法宝成功");
    assert_eq!(st.equipment.treasure[0].as_deref(), Some("cu_bab_benming_fejian"),
        "法宝应装配到 equipment.treasure[0]");
    assert!(st.treasures.contains(&"cu_bab_benming_fejian".to_string()), "拥有标记保留");
    assert_eq!(power::equipped_atk_flat(&st), 8, "法宝攻击加成 8 生效");
    assert_eq!(power::power(&st), power::power(&GameState::new()) + 8, "power 随法宝上升");
}