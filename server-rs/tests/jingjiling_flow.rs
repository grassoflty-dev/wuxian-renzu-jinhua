//! 黄金示例副本 tests（确定性，不碰随机战斗）
use wuxian_horror_ch1::scenes;

#[test]
fn jingjiling_scenes_exist() {
    assert!(scenes::scene("jj_00").is_some());
    assert!(scenes::scene("jj_01").is_some());
}

#[test]
fn jingjiling_fights_exist() {
    assert!(scenes::fight_cfg("jj_boss").is_some());
}

#[test]
fn jingjiling_self_consistent() {
    for (id, _c) in wuxian_horror_ch1::scenes_jingjiling::jingjiling_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
}