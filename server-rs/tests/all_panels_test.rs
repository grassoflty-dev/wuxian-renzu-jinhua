//! all_panels_test.rs —— 主神空间面板全遍历（全量覆盖，非抽样）
//!
//! 遍历 scenes::SCENES 中全部 s_nexus_* 前缀面板/分页场景：
//!   - 每个面板 scenes::scene(id) 可解析（面板可达）
//!   - 每个面板 choices 非空；
//!   - 每个 Route::To 的目标场景可解析（面板间跳转可达）
//!
//! 并断言结算卡 / 死亡卡关键字段：
//!   - 每个 overlay 卡片按钮非空
//!   - 至少存在一张含 __enter_nexus__ 回去路由的卡片（主神衔接可达）
use wuxian_horror_ch1::defs::{OverlayDef, Route};
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::state::{Card, GameState};

const NEXUS_PREFIX: &str = "s_nexus";

/// 汇总一张覆盖层卡片的按钮信息用于断言
fn render_card(ov: &OverlayDef) -> Card {
    // 用一份干净存档渲染（按钮/路由存在性与存档值无关地检）
    (ov.card)(&GameState::new())
}

#[test]
fn all_nexus_panels_reachable() {
    let mut total_panels = 0usize;
    let mut total_route_targets = 0usize;
    let mut broken_targets = Vec::new();

    for scene in scenes::SCENES {
        if !scene.id.starts_with(NEXUS_PREFIX) {
            continue;
        }
        total_panels += 1;
        // 面板可达：中央注册表能解析
        assert!(
            scenes::scene(scene.id).is_some(),
            "主神面板 {id} scenes::scene 不可解析", id = scene.id,
        );
        // 选项非空：非覆盖层面板必须有选项；覆盖层（弹卡）面板允许空 choices（卡由 overlay.card 承载）
        if scene.overlay.is_none() {
            assert!(
                !scene.choices.is_empty(),
                "主神面板 {id} choices 为空且无 overlay（面板既无选项也无卡片）", id = scene.id,
            );
        }
        // 面板间跳转：Route::To 目标可解析（Dyn 运行时求值无法静态断言，跳过）
        for c in scene.choices {
            if let Route::To(target) = c.route {
                total_route_targets += 1;
                if scenes::scene(target).is_none() {
                    // 特例：允许跳到非本面板但全局注册过的场景即可
                    broken_targets.push(format!("[{}] -> {target}", scene.id));
                }
            }
        }
    }

    eprintln!(
        "[all_nexus_panels_reachable] 面板（s_nexus_*）={total_panels} 跳转目标={total_route_targets} 坏目标={}",
        broken_targets.len()
    );
    assert!(total_panels >= 30, "主神面板应 ≥30，实际 {total_panels}");
    assert!(
        broken_targets.is_empty(),
        "存在不可解析的面板跳转目标：{:?}", broken_targets,
    );
}

#[test]
fn all_overlay_cards_have_buttons() {
    let mut overlays_with_enter = 0usize;
    let mut total_cards = 0usize;
    let mut empty_buttons = Vec::new();

    for scene in scenes::SCENES {
        if let Some(ov) = &scene.overlay {
            total_cards += 1;
            let card = render_card(ov);
            if card.buttons.is_empty() {
                empty_buttons.push(scene.id);
            }
            let has_enter = card.buttons.iter().any(|(_, r)| r == "__enter_nexus__");
            if has_enter {
                overlays_with_enter += 1;
            }
        }
    }

    eprintln!(
        "[all_overlay_cards_have_buttons] 覆盖层卡片={total_cards} 含__enter_nexus__={overlays_with_enter} 空按钮={}",
        empty_buttons.len()
    );
    assert!(
        empty_buttons.is_empty(),
        "以下覆盖层卡片按钮为空：{:?}", empty_buttons,
    );
    assert!(
        overlays_with_enter >= 1,
        "应至少一张覆盖层卡片含 __enter_nexus__ 回去路由（主神衔接卡）",
    );
}

/// 结算/死亡卡结构契约：s_settle 结算卡必有按钮且含 __enter_nexus__，
/// 遍历所有 overlay 场景一次性断言（比单测更全量）。
#[test]
fn settlement_and_death_cards_contract() {
    let mut found_settle = false;
    for scene in scenes::SCENES {
        let Some(ov) = &scene.overlay else { continue };
        let card = render_card(ov);
        assert!(
            !card.title.is_empty(),
            "覆盖层场景 {id} 卡片标题为空", id = scene.id,
        );
        assert!(
            !card.body_html.is_empty(),
            "覆盖层场景 {id} 卡片内容为空", id = scene.id,
        );
        if scene.id == "s_settle" {
            found_settle = true;
            assert!(
                card.buttons.iter().any(|(_, r)| r == "__enter_nexus__"),
                "结算卡 s_settle 无 __enter_nexus__ 路由",
            );
        }
    }
    assert!(found_settle, "未找到 s_settle 结算覆盖层场景");
}