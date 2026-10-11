//! 任务系统 —— 集成测试
//! 覆盖：task_view 按世界过滤 / current_task 主线未完成 / flag 完成判定 / 主神空间无任务。
use wuxian_horror_ch1::state::GameState;
use wuxian_horror_ch1::tasks;

#[test]
fn biohazard_tasks_present_and_current() {
    let mut st = GameState::new();
    st.world_id = wuxian_horror_ch1::worlds::WORLD_BIOHAZARD.to_string();
    let view = tasks::task_view(&st);
    assert_eq!(view.len(), 8, "生化蜂巢应含 1 主线 + 7 支线/隐藏");
    assert!(view.iter().all(|t| !t.done), "新局任务应全部未完成");
    assert_eq!(
        tasks::current_task(&st),
        Some("主线 · 进入蜂巢，关闭红后，活着回到地面")
    );
    assert_eq!(view[0].kind, "main", "首条应为主线");
}

#[test]
fn flags_mark_tasks_done() {
    let mut st = GameState::new();
    st.world_id = wuxian_horror_ch1::worlds::WORLD_BIOHAZARD.to_string();
    st.set_flag("A");
    st.set_flag("bh_cleared");
    let view = tasks::task_view(&st);
    assert!(
        view.iter().any(|t| t.id == "bh_A" && t.done),
        "支线A 应完成"
    );
    assert!(
        view.iter().any(|t| t.id == "bh_main" && t.done),
        "主线应完成"
    );
    assert_eq!(tasks::current_task(&st), None, "主线完成后无当前主线任务");
}

#[test]
fn nexus_has_no_tasks() {
    let mut st = GameState::new();
    st.world_id = wuxian_horror_ch1::worlds::WORLD_ZHUTIAN.to_string();
    assert!(tasks::task_view(&st).is_empty(), "主神空间无任务");
    assert_eq!(tasks::current_task(&st), None);
}
