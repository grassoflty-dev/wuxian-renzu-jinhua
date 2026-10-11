//! 黄金示例副本 tests（确定性，不碰随机战斗）
use wuxian_horror_ch1::scenes;

#[test]
fn xingjichuanqi_scenes_exist() {
    assert!(scenes::scene("xj_00").is_some());
    assert!(scenes::scene("xj_01").is_some());
}

#[test]
fn xingjichuanqi_fights_exist() {
    assert!(scenes::fight_cfg("xj_boss").is_some());
}

#[test]
fn xingjichuanqi_self_consistent() {
    for (id, _c) in wuxian_horror_ch1::scenes_xingjichuanqi::xingjichuanqi_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
}