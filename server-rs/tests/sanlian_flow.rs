//! 黄金示例副本 tests（确定性，不碰随机战斗）
use wuxian_horror_ch1::scenes;

#[test]
fn sanlian_scenes_exist() {
    assert!(scenes::scene("sl_00").is_some());
    assert!(scenes::scene("sl_01").is_some());
}

#[test]
fn sanlian_fights_exist() {
    assert!(scenes::fight_cfg("sl_boss").is_some());
}

#[test]
fn sanlian_self_consistent() {
    for (id, _c) in wuxian_horror_ch1::scenes_sanlian::sanlian_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
}