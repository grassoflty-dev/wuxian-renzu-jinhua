//! 黄金示例副本 tests（确定性，不碰随机战斗）
use wuxian_horror_ch1::scenes;

#[test]
fn hangu_scenes_exist() {
    assert!(scenes::scene("hg_00").is_some());
    assert!(scenes::scene("hg_01").is_some());
}

#[test]
fn hangu_fights_exist() {
    assert!(scenes::fight_cfg("hg_boss").is_some());
}

#[test]
fn hangu_self_consistent() {
    for (id, _c) in wuxian_horror_ch1::scenes_hangu::hangu_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
}