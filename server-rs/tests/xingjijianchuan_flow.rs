//! 黄金示例副本 tests（确定性，不碰随机战斗）
use wuxian_horror_ch1::scenes;

#[test]
fn xingjijianchuan_scenes_exist() {
    assert!(scenes::scene("xjj_00").is_some());
    assert!(scenes::scene("xjj_01").is_some());
}

#[test]
fn xingjijianchuan_fights_exist() {
    assert!(scenes::fight_cfg("xjj_boss").is_some());
}

#[test]
fn xingjijianchuan_self_consistent() {
    for (id, _c) in wuxian_horror_ch1::scenes_xingjijianchuan::xingjijianchuan_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
}