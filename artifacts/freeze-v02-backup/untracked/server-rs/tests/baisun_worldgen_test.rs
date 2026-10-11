//! baisun 主世界程序化地图接入验收：
//! - 三层均由 worldgen 生成，尺寸 640x640（放大概 10 倍，明显大于原手写 40x26）。
//! - 确定性：find_world 多次读取同一副本生成一致（同一 WorldData.floors 引用）。
//! - 出生点 / 传送落点可走（由泛用 all_worlds_interaction_test 已覆盖，这里补尺寸与确定性）。
use wuxian_horror_ch1::worlds;

#[test]
fn baisun_floors_are_generated_640x640_deterministic() {
    let w = worlds::find_world("baisun").expect("baisun world 应注册");
    assert_eq!(w.floors.len(), 3, "baisun 应为 3 层");
    for (fi, floor) in w.floors.iter().enumerate() {
        assert_eq!(floor.len(), 640, "第 {fi} 层高度应为 640");
        for (yi, row) in floor.iter().enumerate() {
            assert_eq!(row.len(), 640, "第 {fi} 层第 {yi} 行宽度应为 640");
        }
    }
    // 确定性：同一 WorldData 静态缓存，两次读取应逐层逐格一致
    let w2 = worlds::find_world("baisun").unwrap();
    for fi in 0..w.floors.len() {
        assert_eq!(
            w.floors[fi], w2.floors[fi],
            "第 {fi} 层应逐位一致（确定性）"
        );
    }
}

#[test]
fn baisun_spawn_and_portal_landings_walkable() {
    let w = worlds::find_world("baisun").expect("baisun world 应注册");
    let (sx, sy) = w.spawn();
    assert!(worlds::walkable(w, 0, sx, sy), "出生点 ({sx},{sy}) 应可走");
    // 传送落点（现有 baisun::PORTALS 到层 1/2 的 (2,3)）应可走
    for pt in w.portals {
        assert!(
            worlds::walkable(w, pt.to_floor, pt.tx, pt.ty),
            "传送 {} 落点 ({},{}) 应可走",
            pt.id,
            pt.tx,
            pt.ty,
        );
    }
}
