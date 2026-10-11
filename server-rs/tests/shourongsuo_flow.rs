//! 黄金示例副本 tests（确定性，不碰随机战斗）
use wuxian_horror_ch1::scenes;

#[test]
fn shourongsuo_scenes_exist() {
    assert!(scenes::scene("sr_00").is_some());
    assert!(scenes::scene("sr_01").is_some());
}

#[test]
fn shourongsuo_fights_exist() {
    assert!(scenes::fight_cfg("sr_boss").is_some());
}

#[test]
fn shourongsuo_self_consistent() {
    for (id, _c) in wuxian_horror_ch1::scenes_shourongsuo::shourongsuo_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
}