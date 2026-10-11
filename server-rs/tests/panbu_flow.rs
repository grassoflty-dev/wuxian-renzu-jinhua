//! 副本 tests · 盘部落圣遗之夜（panbu / pb_，确定性，不碰随机战斗）
use wuxian_horror_ch1::scenes;

#[test]
fn panbu_scenes_exist() {
    assert!(scenes::scene("pb_00").is_some());
    assert!(scenes::scene("pb_01").is_some());
    assert!(scenes::scene("pb_round").is_some());
}

#[test]
fn panbu_fights_exist() {
    assert!(scenes::fight_cfg("pb_boss").is_some());
}

#[test]
fn panbu_self_consistent() {
    for (id, _c) in wuxian_horror_ch1::scenes_panbu::panbu_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
}