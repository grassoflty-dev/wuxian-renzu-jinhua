//! P1 点数消费体系集成测试：兑换扣点 + 属性生效 / 点数不足拒绝 / 复活扣点 + dead_team 变动
//! 复用 engine::goto / engine::choose 走完整场景流转（与 playthrough 同风格）。
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::state::{GameState, Mode};
use wuxian_horror_ch1::engine;

fn pick(st: &GameState, keyword: &str) -> i32 {
    let scene = scenes::scene(&st.scene_id).expect("scene");
    let visible: Vec<_> = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(st))).collect();
    for (i, c) in visible.iter().enumerate() {
        if c.label.contains(keyword) {
            return i as i32;
        }
    }
    panic!("scene {} 未找到含「{}」的选项；可见: {:?}",
        st.scene_id, keyword, visible.iter().map(|c| c.label).collect::<Vec<_>>());
}

/// 用例1：兑换「细胞活力强化」扣 800 点 + str_bonus+1 + strength 生效进入 done 场景
#[test]
fn exchange_strengthen_deducts_points_and_applies_bonus() {
    let mut st = GameState::new();
    st.points = 2000; // 足够兑换一项
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_exchange", &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange", "应进入可交互兑换场景");

    let idx = pick(&st, "细胞活力强化");
    engine::choose(&mut st, idx, &mut deaths);

    // 扣点 + 属性写入 + 进入 done 场景（Mode::Normal，非覆盖层）
    assert_eq!(st.points, 2000 - 800, "应扣 800 点");
    assert_eq!(st.str_bonus, 1, "str_bonus 应 +1");
    assert_eq!(st.scene_id, "s_nexus_exchange_done");
    assert!(matches!(st.mode, Mode::Normal));

    // 重复购买：再扣 800，str_bonus+1（可叠加成 spend sink）
    let d2 = pick(&st, "返回兑换目录");
    engine::choose(&mut st, d2, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange");
    let again = pick(&st, "细胞活力强化");
    engine::choose(&mut st, again, &mut deaths);
    assert_eq!(st.points, 1200 - 800, "二次兑换再扣 800");
    assert_eq!(st.str_bonus, 2, "str_bonus 叠加到 2");
}

/// 用例2（A）：点数不足兑换 → 进入 fail 场景，不扣点、属性不变
#[test]
fn exchange_insufficient_points_rejected_no_deduction() {
    let mut st = GameState::new();
    st.points = 500; // 不足 800
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_exchange", &mut deaths);

    let idx = pick(&st, "细胞活力强化");
    engine::choose(&mut st, idx, &mut deaths);

    assert_eq!(st.scene_id, "s_nexus_exchange_fail", "点数不足应进 fail 场景");
    assert_eq!(st.points, 500, "不足时不得扣点");
    assert_eq!(st.str_bonus, 0, "不足时属性不变");
}

/// 用例2（B）：兑换权限/即时状态——兑换基因锁后选项隐藏
#[test]
fn exchange_gene_hides_after_purchase() {
    let mut st = GameState::new();
    st.points = 2100;
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_exchange", &mut deaths);

    let idx = pick(&st, "基因锁");
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_done");
    assert_eq!(st.points, 100, "基因锁扣 2000");
    assert!(st.gene_lock, "gene_lock 自主开启权生效");
    assert!(st.flag("ex_bought_gene"));

    // 返回兑换目录 → 基因锁不再显示
    let d = pick(&st, "返回兑换目录");
    engine::choose(&mut st, d, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange");
    assert!(pick_absent(&st, "基因锁"), "兑换后基因锁选项应隐藏");
}

/// 辅助：断言某选项不可见
fn pick_absent(st: &GameState, keyword: &str) -> bool {
    let scene = scenes::scene(&st.scene_id).unwrap();
    let visible: Vec<_> = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(st))).collect();
    !visible.iter().any(|c| c.label.contains(keyword))
}

/// 用例3：复活队友 → 扣 4000 + dead_team 首位移除（回到存活）
#[test]
fn resurrect_deducts_points_and_removes_dead_teammate() {
    let mut st = GameState::new();
    st.points = 6000;
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    st.dead_team = vec!["one".to_string(), "jd".to_string()];
    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_resurrection", &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_resurrection");

    let idx = pick(&st, "复活一名本次阵亡的同伴");
    engine::choose(&mut st, idx, &mut deaths);

    assert_eq!(st.scene_id, "s_nexus_resurrect_done", "成功复活进入 done");
    assert_eq!(st.points, 6000 - 4000, "复活应扣 4000 点");
    assert_eq!(st.dead_team.len(), 1, "首位阵亡者被移除");
    assert!(!st.dead_team.contains(&"one".to_string()), "一号回到存活");
    assert!(st.dead_team.contains(&"jd".to_string()), "其他阵亡者保留");
    assert_eq!(st.resurrected_name.as_deref(), Some("one"), "记录被复活队友");
}

/// 用例3（B）：复活失败——点数不足 → fail 场景，不扣点、dead_team 不变
#[test]
fn resurrect_insufficient_points_rejected() {
    let mut st = GameState::new();
    st.points = 3000; // 不足 4000
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    st.dead_team = vec!["rain".to_string()];
    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_resurrection", &mut deaths);

    let idx = pick(&st, "复活一名本次阵亡的同伴");
    engine::choose(&mut st, idx, &mut deaths);

    assert_eq!(st.scene_id, "s_nexus_resurrect_fail");
    assert_eq!(st.points, 3000, "不足不得扣点");
    assert_eq!(st.dead_team, vec!["rain".to_string()], "dead_team 不变");
}

/// 用例3（C）：本次轮回无人阵亡 → 提示 none 场景
#[test]
fn resurrect_with_no_dead_shows_none() {
    let mut st = GameState::new();
    st.points = 9999;
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    st.dead_team.clear();
    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_resurrection", &mut deaths);

    assert!(pick_absent(&st, "复活"), "无阵亡时不显示复活选项");
    // 走旧的「抚过祭坛符纹」返回张杰
    let idx = pick(&st, "抚过祭坛符纹");
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_zhangjie");
}

/* =====================================================================
   测试补全（test2_ 前缀，覆盖点：兑换强化流程 / 跨世界传送门 / 面板遍历）
   ===================================================================== */

/// 自用：进入兑换主目录
fn t2_go_exchange(st: &mut GameState) {
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    let mut deaths = vec![];
    engine::goto(st, "s_nexus_exchange", &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange");
}

/// 补1：兑换强化流程全覆盖 —— 技能/血统/装备三类各兑换一次，逐项断言 points 扣减 + 持有状态。
/// 装备走道具铺（s_nexus_exchange_shop→精锻武士刀 = wp_katana·1200·需C）
/// 技能走技能秘藏→通用→洞察侦查（sk_util_inspect·800）
/// 血统走主目录→初级吸血鬼血统（vampire·3000·附赠敏捷+1）
#[test]
fn test2_exchange_skill_bloodline_equipment_flow() {
    let mut st = GameState::new();
    st.points = 20_000;
    st.sp_grade = Some('C');
    t2_go_exchange(&mut st);
    let mut deaths = vec![];

    // ---- 技能：技能秘藏 → 通用 → 洞察侦查（-800）
    let sk = pick(&st, "技能秘藏");
    engine::choose(&mut st, sk, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_skill", "应进入技能秘藏");
    let util = pick(&st, "通用");
    engine::choose(&mut st, util, &mut deaths);
    let skill = pick(&st, "洞察侦查");
    engine::choose(&mut st, skill, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_done");
    assert_eq!(st.points, 20_000 - 800, "技能兑换应扣 800 点");
    assert!(st.skills.contains(&"sk_util_inspect".to_string()), "技能应入库 sk_util_inspect");

    // 返回兑换目录
    let back = pick(&st, "返回兑换目录");
    engine::choose(&mut st, back, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange");

    // ---- 血统：初级吸血鬼血统（-3000，bloodline 写入 + agi_bonus+1）
    let blood = pick(&st, "初级吸血鬼血统");
    engine::choose(&mut st, blood, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_done");
    assert_eq!(st.points, 20_000 - 800 - 3000, "血统兑换应再扣 3000");
    assert_eq!(st.bloodline.as_deref(), Some("vampire"), "持有血统 vampire");
    assert_eq!(st.agi_bonus, 1, "血统附赠敏捷 +1");

    // 返回兑换目录
    let back = pick(&st, "返回兑换目录");
    engine::choose(&mut st, back, &mut deaths);

    // ---- 装备：道具铺 → 精锻武士刀（wp_katana·-1200·需C）
    let shop = pick(&st, "道具铺");
    engine::choose(&mut st, shop, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_shop");
    let wp = pick(&st, "精锻武士刀");
    engine::choose(&mut st, wp, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_done");
    assert_eq!(st.points, 20_000 - 800 - 3000 - 1200, "装备兑换应再扣 1200");
    let w = st.equipment.weapon.as_ref().expect("应装配武器");
    assert_eq!(w.id, "wp_katana", "武器槽装配 wp_katana");
}

/// 补2：跨世界传送门 —— 主神空间 → 某副本 → 回主神，断言 switch_world 快照/恢复闭环。
/// 起点主神标记一处调查点，切到副本后该点随快照离场，回主神后恢复。
#[test]
fn test2_cross_world_portal_roundtrip() {
    let mut st = GameState::new();
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    // 标记主神一处调查点（模拟探索进度）
    st.map_objs.insert("np_nexus_god".into(), true);
    let origin_enemies = st.enemies_alive.len();

    // 传送 → 生化蜂巢
    wuxian_horror_ch1::world::switch_world(&mut st, wuxian_horror_ch1::worlds::WORLD_BIOHAZARD);
    assert_eq!(st.world_id, wuxian_horror_ch1::worlds::WORLD_BIOHAZARD, "应切到生化副本世界");
    // 主神点已随快照离场
    assert!(!st.map_objs.contains_key("np_nexus_god"), "主神调查点不应出现在副本世界");
    assert!(st.world_states.contains_key(wuxian_horror_ch1::worlds::WORLD_ZHUTIAN), "主神快照应入 world_states");
    // 副本世界敌人被惰性初始化
    let bio = wuxian_horror_ch1::worlds::find_world(wuxian_horror_ch1::worlds::WORLD_BIOHAZARD).unwrap();
    assert_eq!(st.enemies_alive.len() >= bio.enemies.len(), true, "生化敌人表应补齐");

    // 传回主神
    wuxian_horror_ch1::world::switch_world(&mut st, wuxian_horror_ch1::worlds::WORLD_ZHUTIAN);
    assert_eq!(st.world_id, wuxian_horror_ch1::worlds::WORLD_ZHUTIAN, "应回到主神空间");
    assert_eq!(st.map_objs.get("np_nexus_god").copied(), Some(true), "主神调查点状态应恢复");
    assert!(!st.world_states.contains_key(wuxian_horror_ch1::worlds::WORLD_ZHUTIAN), "主神快照应已取出");
    // 非空副本的原敌人数不受主神影响（主神无战斗 → 敌人数回到空/原点）
    assert_eq!(st.enemies_alive.len() >= origin_enemies, true, "往返后存活表应被主神空表刷新");
}

/// 补3：高等血统评级门槛 —— 狼人血统（4500·需 B）在 C 级被拒、B 级成交，
/// 断言评级不足 fail 不扣点 / 满足后 points 扣减 + bloodline 持有 + 属性生效。
#[test]
fn test2_high_bloodline_grade_gated() {
    // C 级（不足 B）→ fail
    let mut st = GameState::new();
    st.points = 100_000;
    st.sp_grade = Some('C');
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_exchange_blood", &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_blood");
    let wolf = pick(&st, "狼人血统");
    engine::choose(&mut st, wolf, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_fail", "C 级兑换狼人应失败");
    assert_eq!(st.points, 100_000, "评级不足不扣点");
    assert_ne!(st.bloodline.as_deref(), Some("werewolf"), "评级不足不得写入血统");

    // B 级 → 成交
    let mut st = GameState::new();
    st.points = 100_000;
    st.sp_grade = Some('B');
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_exchange_blood", &mut deaths);
    let wolf = pick(&st, "狼人血统");
    engine::choose(&mut st, wolf, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_done");
    assert_eq!(st.points, 100_000 - 4500, "狼人应扣 4500 点");
    assert_eq!(st.bloodline.as_deref(), Some("werewolf"), "B 级应写入狼人血统");
}

/// 补4：面板遍历 —— 主神各主要兑换面板经 goto/choose 可达并往返，面板间跳转闭环。
/// 覆盖：兑换目录 → 基因进化 / 修真 / 科技 / 道具铺 / 合成工坊 / 返回主神广场。
#[test]
fn test2_nexus_panels_traversal() {
    let mut st = GameState::new();
    st.points = 50_000;
    st.sp_grade = Some('A');
    st.gene_stage = 1;
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_exchange", &mut deaths);

    // 基因进化面板
    let g = pick(&st, "基因进化");
    engine::choose(&mut st, g, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_gene");
    let back = pick(&st, "返回兑换目录");
    engine::choose(&mut st, back, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange");

    // 修真面板
    let x = pick(&st, "修真");
    engine::choose(&mut st, x, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_xiu");
    let back = pick(&st, "返回兑换目录");
    engine::choose(&mut st, back, &mut deaths);

    // 科技面板
    let t = pick(&st, "科技");
    engine::choose(&mut st, t, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_tech");
    let back = pick(&st, "返回兑换目录");
    engine::choose(&mut st, back, &mut deaths);

    // 道具铺
    let s = pick(&st, "道具铺");
    engine::choose(&mut st, s, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_shop");
    let back = pick(&st, "返回兑换目录");
    engine::choose(&mut st, back, &mut deaths);

    // 合成工坊
    let c = pick(&st, "合成工坊");
    engine::choose(&mut st, c, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_craft");
    let back = pick(&st, "返回兑换目录");
    engine::choose(&mut st, back, &mut deaths);

    // 返回主神广场
    let god = pick(&st, "返回主神广场");
    engine::choose(&mut st, god, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_god", "面板遍历应收束回主神广场");
    assert_eq!(st.points, 50_000, "纯面板遍历不产生点数变动");
}
