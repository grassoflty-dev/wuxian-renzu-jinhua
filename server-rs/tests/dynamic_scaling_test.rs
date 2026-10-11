//! 动态难度缩放（tools/design/dynamic_scaling_design.md §六）纯函数 + 集成测试。
//! 覆盖 power()/power_factor()/difficulty_scale()/fight_scale()/scaled_fight()。
//! 文件命名为 dynamic_scaling_test.rs（任务约定词尾）。
use wuxian_horror_ch1::combat_data::set_gene_stage;
use wuxian_horror_ch1::defs::{Equipment, WeaponSlot};
use wuxian_horror_ch1::power;
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::state::GameState;
use wuxian_horror_ch1::worlds;

/// 基准：fresh GameState 的 power（无武器/装备/技能/境界）
fn fresh_power() -> i32 {
    let st = GameState::new();
    // (hp/20)=5 + weapon_atk(默认 6) = 11
    assert_eq!(power::power(&st), 11);
    11
}

/// 装备一把已知中值武器 → weapon_atk 与 power 上升
#[test]
fn power_weapon_raises() {
    let mut st = GameState::new();
    let p0 = power::power(&st);
    // 问心·青锋剑 dmg(30,46) 中值 38
    st.equipment.weapon = Some(WeaponSlot { id: "wp_cu_ju".to_string(), enhance: 0 });
    let wa = power::weapon_atk(&st);
    assert_eq!(wa, 38, "青锋剑中值 (30+46)/2 = 38");
    assert_eq!(power::power(&st), p0 + 38 - 6, "power 增加 (38 - 默认6)");
}

/// power() 对装备 atk_flat 加成敏感（护甲/饰品/法宝）
#[test]
fn power_equipped_atk_flat_raises() {
    let mut st = GameState::new();
    let p0 = power::power(&st);
    // 蛮力指环 accessory atk_flat=6
    st.equipment.accessory = Some("access_strength_ring".to_string());
    assert_eq!(power::equipped_atk_flat(&st), 6, "饰品攻击加成 6");
    // 本命飞剑·青锋 treasure atk_flat=8（直接装配 equipment.treasure[0]）
    st.equipment.treasure[0] = Some("cu_bab_benming_fejian".to_string());
    assert_eq!(power::equipped_atk_flat(&st), 6 + 8, "饰品+法宝攻击加成 14");
    assert_eq!(power::power(&st), p0 + 14, "power 随装备攻击上升");
}

/// power() 单调：基因锁阶 / 修真境界 / 技能数 各贡献
#[test]
fn power_monotonic() {
    let mut st = GameState::new();
    let p0 = fresh_power();
    // 基因锁 4 阶 → +32
    set_gene_stage(&mut st, 4);
    st.cultivation_stage = 7; // 合道 +42
    for i in 0..30 {
        st.skills.push(format!("skx_test_{i}")); // 技能贡献上限 +20
    }
    assert_eq!(power::power(&st), p0 + 32 + 42 + 20, "power 随成长单调增加");
    assert!(power::power(&st) > p0);
}

/// power_factor：P = 软上限（对数衰减）。(1 + power/25).log2()，保底 0.6，**无硬上限 4.0**。
#[test]
fn power_factor_soft_cap() {
    // 低 power → 保底 0.6
    let mut low = GameState::new(); // power=11 → 11/25=0.44 → log2(1.44)=0.526 < 0.6
    assert_eq!(power::power_factor(&low), 0.6);
    // 中 power → 仍低于 0.6 → clamp 0.6
    low.skills.push("x".to_string()); // power=12 → 0.48 → log2(1.48)=0.566 → clamp 0.6
    assert_eq!(power::power_factor(&low), 0.6);
    // 高 power：不再硬 clamp 到 4.0，而是 log2 对数衰减（>4.0 无上限）
    let mut hi = GameState::new();
    hi.hp = 800;
    hi.equipment.weapon = Some(WeaponSlot { id: "wp_gravity_collapse".to_string(), enhance: 0 }); // +52
    set_gene_stage(&mut hi, 4); // +32
    hi.cultivation_stage = 7; // +42
    hi.equipment.accessory = Some("access_strength_ring".to_string()); // +6
    for i in 0..30 { hi.skills.push(format!("skx_{i}")); } // +20
    // power = 40+52+32+42+6+20 = 192 → p=192/25=7.68 → (1+7.68).log2() = log2(8.68) ≈ 3.118
    let expected = (1.0 + 192.0_f32 / 25.0).log2();
    assert!((power::power_factor(&hi) - expected).abs() < 1e-4, "软上限 log2(8.68)≈3.118，实际 {}", power::power_factor(&hi));
    assert!(power::power_factor(&hi) > 3.0 && power::power_factor(&hi) < 4.0, "不再硬钳到 4.0");
    // 极高的 power 仍继续增长（无硬上限）：hp=20000 → power≈1006 → p≈40.24 → log2(41.24)≈5.37 > 4.0
    let mut huge = GameState::new();
    huge.hp = 20_000; // hp/20=1000, 默认武器+6 → power≈1006
    let exp_huge = (1.0 + power::power(&huge) as f32 / 25.0).log2();
    assert!((power::power_factor(&huge) - exp_huge).abs() < 1e-4);
    assert!(power::power_factor(&huge) > 4.0, "无硬上限，power 很大时应超过旧的 4.0 封顶");
}

/// difficulty_scale：D 映射表
#[test]
fn difficulty_scale_table() {
    assert_eq!(power::difficulty_scale(1), 0.8);
    assert_eq!(power::difficulty_scale(2), 1.0);
    assert_eq!(power::difficulty_scale(3), 1.3);
    assert_eq!(power::difficulty_scale(4), 1.6);
    assert_eq!(power::difficulty_scale(5), 2.0);
    // 未知难度（含 0 世界难度占位）回退 1.0
    assert_eq!(power::difficulty_scale(0), 1.0);
    assert_eq!(power::difficulty_scale(6), 1.0);
    assert_eq!(power::difficulty_scale(99), 1.0);
}

/// scaling_enabled=false 安全阀：fight_scale 恒 1.0（价值观测的既有 flow 回归）
#[test]
fn fight_scale_valve_when_disabled() {
    let mut st = GameState::new();
    st.scaling_enabled = false;
    st.world_id = worlds::WORLD_BIOHAZARD.to_string(); // D=0.8
    // 即使高 power 也不缩放
    st.gene_stage = 4;
    st.cultivation_stage = 7;
    st.hp = 800;
    assert_eq!(power::fight_scale(&st), 1.0, "关闭缩放时恒 1.0");
}

/// fight_scale = D × P（开启状态，生化 difficulty=1 → D=0.8）
#[test]
fn fight_scale_multiplicative() {
    let mut st = GameState::new();
    st.scaling_enabled = true;
    st.world_id = worlds::WORLD_BIOHAZARD.to_string();
    // 低 power → factor 0.6 ; D=0.8 → scale 0.48
    assert_eq!(power::power_factor(&st), 0.6);
    assert!((power::fight_scale(&st) - 0.8 * 0.6).abs() < 1e-6);
    // 高 power → factor≈3.118 → scale = 0.8×3.118 = 2.494（软上限，不再是 4.0 封顶）
    let mut hi = GameState::new();
    hi.scaling_enabled = true;
    hi.world_id = worlds::WORLD_BIOHAZARD.to_string();
    hi.hp = 800;
    hi.equipment.weapon = Some(WeaponSlot { id: "wp_gravity_collapse".to_string(), enhance: 0 });
    set_gene_stage(&mut hi, 4);
    hi.cultivation_stage = 7;
    hi.equipment.accessory = Some("access_strength_ring".to_string());
    for i in 0..30 { hi.skills.push(format!("skx_{i}")); }
    let p_hi = (1.0 + 192.0_f32 / 25.0).log2(); // ≈3.118
    assert!((power::power_factor(&hi) - p_hi).abs() < 1e-4);
    assert!((power::fight_scale(&hi) - 0.8 * p_hi).abs() < 1e-4);
}

/// scaled_fight：关闭缩放 → Fight 数值 == 基准；开启 → hp/dmg/reward 按 scale 放大
#[test]
fn scaled_fight_applies_scale() {
    let cfg = scenes::fight_cfg("horde").expect("horde 战斗表");
    assert_eq!(cfg.hp, 55);
    assert_eq!(cfg.dmg, (11, 17));
    assert_eq!(cfg.reward, 20);

    // 关闭缩放 → 1:1
    let mut off = GameState::new();
    off.scaling_enabled = false;
    off.world_id = worlds::WORLD_BIOHAZARD.to_string();
    let f = power::scaled_fight("horde", cfg, &off, vec![]);
    assert_eq!(f.hp, 55);
    assert_eq!(f.max_hp, 55);
    assert_eq!(f.dmg, (11, 17));
    assert_eq!(f.reward, 20);
    // 关闭缩放：rage_at 保持基准（horde rage_at=Some(25)）
    assert_eq!(f.rage_at, Some(25));

    // 开启 + 高 power → p_hi≈3.118；hp_scale = D(0.8)×P ≈ 2.494（reward 因 D=1 超额系数同为 0.8，reward_scale 等同 hp_scale）
    let mut hi = GameState::new();
    hi.scaling_enabled = true;
    hi.world_id = worlds::WORLD_BIOHAZARD.to_string();
    hi.hp = 800;
    hi.equipment.weapon = Some(WeaponSlot { id: "wp_gravity_collapse".to_string(), enhance: 0 });
    set_gene_stage(&mut hi, 4);
    hi.cultivation_stage = 7;
    hi.equipment.accessory = Some("access_strength_ring".to_string());
    for i in 0..30 { hi.skills.push(format!("skx_{i}")); }
    let p_hi = (1.0 + 192.0_f32 / 25.0).log2();
    let hs = 0.8 * p_hi;
    assert!((power::fight_scale(&hi) - hs).abs() < 1e-4);
    let f = power::scaled_fight("horde", cfg, &hi, vec![]);
    assert_eq!(f.hp, (55.0_f32 * hs).round() as i32);
    assert_eq!(f.max_hp, f.hp);
    assert_eq!(f.dmg, ((11.0_f32 * hs).round() as i32, (17.0_f32 * hs).round() as i32));
    assert_eq!(f.reward, ((20.0_f32 * 0.8 * p_hi).round() as i32).max(1), "reward 用超额系数（难度1超额=0.8）且 ≥1 保底");
    assert_eq!(f.id, "horde");
}

/// scaled_fight：rage_at 也同源缩放（有 rage 的副本）
#[test]
fn scaled_fight_scales_rage_at() {
    let cfg = scenes::fight_cfg("horde").unwrap(); // rage_at=Some(25)
    assert_eq!(cfg.rage_at, Some(25));
    let mut st = GameState::new();
    st.scaling_enabled = true;
    st.world_id = worlds::WORLD_BIOHAZARD.to_string();
    // 手动给出一个稳定 power → factor 用低值 0.6 → scale = 0.8*0.6 = 0.48
    // rage_at = round(25*0.48) = round(12.0) = 12
    let f = power::scaled_fight("horde", cfg, &st, vec![]);
    assert_eq!(f.rage_at, Some((25.0_f32 * 0.48).round() as i32));
    assert!(f.rage_at.unwrap() < 25, "高难缩放下狂暴阈值按 hp 同比例下修");
}

/// 防御性：缩放不破坏整数健壮性 —— 极大 power 边界不 panic / 不上溢
#[test]
fn scaled_fight_never_panics() {
    let cfg = scenes::fight_cfg("licker").unwrap(); // hp=112 dmg(15,22) reward 500
    let mut st = GameState::new();
    st.scaling_enabled = true;
    st.hp = 1_000_000;
    st.gene_stage = 4;
    st.cultivation_stage = 7;
    st.equipment = Equipment::default();
    let _ = power::scaled_fight("licker", cfg, &st, vec![]);
    let _ = power::scaled_fight("licker_y", cfg, &st, vec![]);
}

/// difficulty_scale_excess：奖励超额系数表（任务二）
#[test]
fn difficulty_scale_excess_table() {
    assert_eq!(power::difficulty_scale_excess(1), 0.8);
    assert_eq!(power::difficulty_scale_excess(2), 1.0);
    assert_eq!(power::difficulty_scale_excess(3), 1.4);
    assert_eq!(power::difficulty_scale_excess(4), 1.9);
    assert_eq!(power::difficulty_scale_excess(5), 2.6);
    // 其余（含 0 / 未注册世界）回退 1.0
    assert_eq!(power::difficulty_scale_excess(0), 1.0);
    assert_eq!(power::difficulty_scale_excess(6), 1.0);
    assert_eq!(power::difficulty_scale_excess(99), 1.0);
    // 难度越高回报率相对强度越高（越级挑战鼓励）：超额系数 ≥ 战斗系数，d≥3 严格更高
    for d in [1usize, 2, 3, 4, 5] {
        assert!(power::difficulty_scale_excess(d) >= power::difficulty_scale(d), "d={d} 超额系数应 ≥ 战斗系数");
    }
    assert!(power::difficulty_scale_excess(3) > power::difficulty_scale(3), "d=3 超额 1.4>1.3");
    assert!(power::difficulty_scale_excess(4) > power::difficulty_scale(4), "d=4 超额 1.9>1.6");
    assert!(power::difficulty_scale_excess(5) > power::difficulty_scale(5), "d=5 超额 2.6>2.0");
}

/// 固定难度挑战关：fixed_difficulty 世界不随主角缩放（任务三）——
/// 洪荒天庭 difficulty=3、fixed_difficulty=true：无论主角多强/多弱，fight_scale 恒 = D(1.3)×固定因子 1.0。
#[test]
fn fixed_difficulty_no_player_scaling() {
    // 弱主角
    let mut weak = GameState::new();
    weak.scaling_enabled = true;
    weak.world_id = worlds::WORLD_TIANTING.to_string();
    // 强主角（高 hp + 满技能 + 装备）
    let mut strong = GameState::new();
    strong.scaling_enabled = true;
    strong.world_id = worlds::WORLD_TIANTING.to_string();
    strong.hp = 20_000; // hp/20=1000 → factor (1+40).log2()≈5.36（远超旧的 4.0 封顶）
    strong.gene_stage = 4;
    strong.cultivation_stage = 7;
    assert!(power::power_factor(&strong) > 4.0, "主角足够强（示意无硬上限）");
    // 弱主角 factor 0.6，强主角 factor>4，但固定难度关 fight_scale 相同（不乘 power_factor）
    assert!((power::fight_scale(&weak) - 1.3).abs() < 1e-4, "弱主角固定关 scale=1.3, 实际 {}", power::fight_scale(&weak));
    assert!((power::fight_scale(&strong) - 1.3).abs() < 1e-4, "强主角固定关 scale 仍=1.3, 实际 {}", power::fight_scale(&strong));
    // 对比非固定世界：强主角 fight_scale 会因软上限显著上升（验证 fixed 才冻结）
    let mut normal_strong = strong.clone();
    normal_strong.world_id = worlds::WORLD_JIGUAN.to_string(); // difficulty=2, 非固定
    assert!(power::fight_scale(&normal_strong) > 1.0, "非固定难世界强主角 scale 随主角上升");
}

/// 固定难度挑战关 reward 用超额系数且不随主角缩放（任务二 × 任务三）：
/// 天庭神将(一形态) tf_shenjiang_r1：hp=260, reward=600；d=3 → hp_scale=D1.3×固定1.0=1.3，reward_scale=超额1.4×固定1.0=1.4。
#[test]
fn fixed_difficulty_reward_uses_excess() {
    let cfg = scenes::fight_cfg("tf_shenjiang_r1").expect("天庭神将战斗表");
    assert_eq!(cfg.hp, 260);
    assert_eq!(cfg.reward, 600);
    // 弱主角（确保固定关 reward 不随 power 压低 / 抬高）
    let mut weak = GameState::new();
    weak.scaling_enabled = true;
    weak.world_id = worlds::WORLD_TIANTING.to_string();
    let f = power::scaled_fight("tf_shenjiang_r1", cfg, &weak, vec![]);
    assert_eq!(f.hp, (260.0_f32 * 1.3).round() as i32, "固定关 hp = D1.3 × 固定1.0");
    assert_eq!(f.reward, (600.0_f32 * 1.4).round() as i32, "reward = 超额1.4 × 固定1.0（≥1 保底）");
    // 强主角 reward 不变（不乘 power_factor）
    let mut strong = GameState::new();
    strong.scaling_enabled = true;
    strong.world_id = worlds::WORLD_TIANTING.to_string();
    strong.hp = 20_000;
    strong.gene_stage = 4;
    strong.cultivation_stage = 7;
    let f2 = power::scaled_fight("tf_shenjiang_r1", cfg, &strong, vec![]);
    assert_eq!(f2.reward, f.reward, "固定关 reward 不随主角强度变化");
}

/// reward 保底 ≥1：极小 reward 的战斗不因缩放归零
#[test]
fn reward_never_below_one() {
    // licker reward=500 过大；改用一把 reward 很小的战斗验证保底
    let cfg = scenes::fight_cfg("horde").unwrap(); // reward=20
    let mut st = GameState::new();
    st.scaling_enabled = true;
    st.world_id = worlds::WORLD_ZHOUYUAN.to_string(); // d=2, 非固定, factor 0.6 → reward_scale=0.6 → 20*0.6=12 ≥1（>0）
    let f = power::scaled_fight("horde", cfg, &st, vec![]);
    assert!(f.reward >= 1, "reward 保底 ≥1，实际 {}", f.reward);
}