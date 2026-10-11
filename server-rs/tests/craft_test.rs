//! 合成系统集成测试：RECIPES 消耗原料 → 产出（inventory 变化）。
//! 复用 items_data 公开 helper（count_item/has_item/consume_item/add_item_counted）+ 主神合成工坊场景流转。
//! 文件名称 craft_test.rs。
//!
//! 注：真实原料 id 以 ITEMS 表为准 `it_core_sample`（能量核心残片）；RECIPES 中曾误写
//! `item_core_sample`（经本测试暴露并修正）。`it_core_crystal` 为配方中间产物（进入 RECIPES/二次配方，
//! 不在 ITEMS 计量表，属合成链内部 id）。
use wuxian_horror_ch1::engine;
use wuxian_horror_ch1::items_data;
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

fn seed(st: &mut GameState, id: &str, n: usize) {
    for _ in 0..n {
        items_data::add_item_counted(st, id);
    }
}

fn held(st: &GameState, id: &str) -> usize {
    items_data::count_item(st, id)
}

/// RECIPES 表注册完整性：每张配方 result 唯一、原料非空
#[test]
fn recipes_registered_wellformed() {
    assert!(!items_data::RECIPES.is_empty());
    let mut seen = std::collections::HashSet::new();
    for r in items_data::RECIPES {
        assert!(seen.insert(r.result), "配方 result 重复: {}", r.result);
        assert!(!r.ingredients.is_empty(), "配方 {} 原料为空", r.result);
    }
}

/// 每个配方原料要么是 ITEMS/TREASURE 有定义的实体道具，要么是另一张配方的产出（内部中间物）
#[test]
fn recipe_ingredients_defined_or_produced() {
    let produced: Vec<&str> = items_data::RECIPES.iter().map(|r| r.result).collect();
    let defined = |id: &str| {
        items_data::item_def(id).is_some()
            || items_data::treasure_def(id).is_some()
            || items_data::QUEST_ITEM_IDS.contains(&id)
    };
    for r in items_data::RECIPES {
        for &ing in r.ingredients {
            let ok = defined(ing) || produced.contains(&ing);
            assert!(ok, "配方 {} 原料 {} 既无定义也非中间产物", r.result, ing);
        }
    }
}

/// 纯 helper：消耗原料 → 计数客观减少，产物计数 +1（等价 recipe_build 内层）
#[test]
fn craft_helper_consume_then_produce() {
    let mut st = GameState::new();
    // 炼制 it_treasure_frag：it_core_sample + it_soul_shard
    seed(&mut st, "it_core_sample", 1);
    seed(&mut st, "it_soul_shard", 1);
    assert_eq!(held(&st, "it_core_sample"), 1);
    assert_eq!(held(&st, "it_soul_shard"), 1);

    for &i in ["it_core_sample", "it_soul_shard"].iter() {
        assert!(items_data::consume_item(&mut st, i), "应能消耗 {}", i);
    }
    assert_eq!(held(&st, "it_core_sample"), 0, "原料已消耗");
    assert_eq!(held(&st, "it_soul_shard"), 0);

    items_data::add_item_counted(&mut st, "it_treasure_frag");
    assert_eq!(held(&st, "it_treasure_frag"), 1, "产物 +1");
}

/// 组装/拆 id 计数契约：同一 base 多枚叠加用 `base_k`，consume 尾删一份
#[test]
fn stackable_count_contract() {
    let mut st = GameState::new();
    seed(&mut st, "it_soul_shard", 3);
    assert_eq!(held(&st, "it_soul_shard"), 3, "三枚灵魂碎片");
    assert!(items_data::consume_item(&mut st, "it_soul_shard"));
    assert_eq!(held(&st, "it_soul_shard"), 2, "消耗一枚剩两枚");
    // 非堆叠唯一物
    seed(&mut st, "it_core_sample", 2); // 定义 stack:false → 去重唯一
    assert_eq!(held(&st, "it_core_sample"), 1, "非堆叠只持一枚");
}

/// 集成：主神合成工坊合成「能量核心残片」（it_core_crystal）
/// 需原料 it_soul_shard + it_core_sample
#[test]
fn exchange_craft_core_crystal() {
    let mut st = GameState::new();
    st.world_id = worlds::WORLD_ZHUTIAN.to_string();
    seed(&mut st, "it_soul_shard", 1);
    seed(&mut st, "it_core_sample", 1);
    assert_eq!(held(&st, "it_core_crystal"), 0);

    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_exchange", &mut deaths);
    let craft = pick(&st, "合成工坊");
    engine::choose(&mut st, craft, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_craft");

    let idx = pick(&st, "合成：能量核心残片");
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_done", "原料齐备应合成成功");
    assert_eq!(held(&st, "it_core_crystal"), 1, "产出核心晶石");
    assert_eq!(held(&st, "it_soul_shard"), 0, "灵魂碎片消耗");
    assert_eq!(held(&st, "it_core_sample"), 0, "能量核心残片消耗");
}

/// 集成：合成电磁炮核心（it_em_core = it_core_crystal + beam_core）
#[test]
fn exchange_craft_em_core() {
    let mut st = GameState::new();
    st.world_id = worlds::WORLD_ZHUTIAN.to_string();
    seed(&mut st, "it_core_crystal", 1);
    seed(&mut st, "beam_core", 1);

    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_exchange", &mut deaths);
    let craft = pick(&st, "合成工坊");
    engine::choose(&mut st, craft, &mut deaths);
    let idx = pick(&st, "合成：电磁炮核心");
    engine::choose(&mut st, idx, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_done");
    assert_eq!(held(&st, "it_em_core"), 1, "产出电磁炮核心");
    assert_eq!(held(&st, "it_core_crystal"), 0, "核心晶石消耗");
    assert_eq!(held(&st, "beam_core"), 0, "光束核心消耗");
}

/// 集成：二级合成链 —— 先合普通强化石（it_enhance_stone），再合高级强化石（it_enhance_stone_hi）
#[test]
fn exchange_craft_enh_stone_chain() {
    let mut st = GameState::new();
    st.world_id = worlds::WORLD_ZHUTIAN.to_string();
    // 普通强化石 = it_core_sample + it_soul_shard
    seed(&mut st, "it_core_sample", 1);
    seed(&mut st, "it_soul_shard", 1);
    // 高级 = 普通强化石 + 电磁炮核心
    seed(&mut st, "it_em_core", 1);

    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_exchange", &mut deaths);
    let craft = pick(&st, "合成工坊");
    engine::choose(&mut st, craft, &mut deaths);

    // 普通强化石可见（原料齐）
    let a = pick(&st, "合成：普通强化石");
    engine::choose(&mut st, a, &mut deaths);
    assert_eq!(held(&st, "it_enhance_stone"), 1, "普通强化石 +1");

    // 回目录→工坊→高级强化石
    let back = pick(&st, "返回兑换目录");
    engine::choose(&mut st, back, &mut deaths);
    let craft = pick(&st, "合成工坊");
    engine::choose(&mut st, craft, &mut deaths);
    let b = pick(&st, "合成：高级强化石");
    engine::choose(&mut st, b, &mut deaths);
    assert_eq!(st.scene_id, "s_nexus_exchange_done", "高级强化石合成成功");
    assert_eq!(held(&st, "it_enhance_stone"), 0, "普通强化石被消耗");
    assert_eq!(held(&st, "it_enhance_stone_hi"), 1, "高级强化石产出 1");
}

/// 集成：原料不足时该合选项不可见（cond 门控）
#[test]
fn exchange_craft_hidden_without_materials() {
    let mut st = GameState::new();
    st.world_id = worlds::WORLD_ZHUTIAN.to_string();
    let mut deaths = vec![];
    engine::goto(&mut st, "s_nexus_exchange", &mut deaths);
    let craft = pick(&st, "合成工坊");
    engine::choose(&mut st, craft, &mut deaths);

    let scene = scenes::scene(&st.scene_id).unwrap();
    let has_any_craft = scene.choices.iter().any(|c| {
        c.label.contains("合成：") && c.cond.map_or(true, |f| f(&st))
    });
    assert!(!has_any_craft, "无原料时任何合成选项都不应可见");
}