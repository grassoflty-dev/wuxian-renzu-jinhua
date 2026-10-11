//! 副本条件触发（网关 need_item + need_flag）—— 集成测试
//! 覆盖：条件网关需道具+线索齐备 / 无条件网关直接开放 / 锁定文案提示缺口。
use wuxian_horror_ch1::state::GameState;
use wuxian_horror_ch1::worlds;

#[test]
fn conditional_gateway_requires_item_and_flag() {
    let mut st = GameState::new();
    let gw = worlds::gw_portal_by_id("gw_tianting").expect("天霆网关应存在");
    assert!(!worlds::gateway_unlocked(gw, &st), "缺道具+线索应锁定");
    st.inventory.push("item_seal_pass".to_string());
    assert!(!worlds::gateway_unlocked(gw, &st), "仅道具、缺线索仍应锁定");
    st.set_flag("bh_cleared");
    assert!(worlds::gateway_unlocked(gw, &st), "道具+线索齐备应解锁");
}

#[test]
fn unconditional_gateway_open() {
    let st = GameState::new();
    let gw = worlds::gw_portal_by_id("gw_biohazard").expect("生化网关应存在");
    assert!(worlds::gateway_unlocked(gw, &st), "无条件网关应直接开放");
}

#[test]
fn locked_msg_mentions_requirements() {
    let gw = worlds::gw_portal_by_id("gw_tianting").unwrap();
    let msg = worlds::gateway_locked_msg(gw);
    assert!(msg.contains("封印通行证"), "锁定文案应提示缺道具：{msg}");
    assert!(msg.contains("生化蜂巢"), "锁定文案应提示缺线索：{msg}");
}
