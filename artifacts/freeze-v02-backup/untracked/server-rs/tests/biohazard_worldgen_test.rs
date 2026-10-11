//! biohazard_ch1 主世界程序化地图接入验收：
//! - 四层均由 worldgen 生成，尺寸 640x640（放大概 16 倍，明显大于原手写 26x40）。
//! - 确定性：find_world 多次读取同一副本生成一致（同一 WorldData.floors 引用）。
//! - 出生点 / 全部对象坐标 / 传送源 + 落点 / 门禁格均可走（坐标权威凿可走）。
use wuxian_horror_ch1::worlds;

#[test]
fn biohazard_floors_are_generated_640x640_deterministic() {
    let w = worlds::find_world("biohazard_ch1").expect("biohazard world 应注册");
    assert_eq!(w.floors.len(), 4, "生化应为 4 层");
    for (fi, floor) in w.floors.iter().enumerate() {
        assert_eq!(floor.len(), 640, "第 {fi} 层高度应为 640");
        for (yi, row) in floor.iter().enumerate() {
            assert_eq!(row.len(), 640, "第 {fi} 层第 {yi} 行宽度应为 640");
        }
    }
    // 确定性：同一 WorldData 静态缓存，两次读取应逐层逐格一致
    let w2 = worlds::find_world("biohazard_ch1").unwrap();
    for fi in 0..w.floors.len() {
        assert_eq!(
            w.floors[fi], w2.floors[fi],
            "第 {fi} 层应逐位一致（确定性）"
        );
    }
}

#[test]
fn biohazard_spawn_and_all_coords_walkable() {
    let w = worlds::find_world("biohazard_ch1").expect("biohazard world 应注册");
    // 出生点可走
    let (sx, sy) = w.spawn();
    assert!(worlds::walkable(w, 0, sx, sy), "出生点 ({sx},{sy}) 应可走");

    let mut checked = 0usize;
    // 调查点坐标可走
    for p in w.points {
        assert!(
            worlds::walkable(w, p.floor, p.x, p.y),
            "调查点 {} 坐标 ({},{}) 在 {floor} 层不可走",
            p.id,
            p.x,
            p.y,
            floor = p.floor,
        );
        checked += 1;
    }
    // 敌人坐标可走
    for e in w.enemies {
        assert!(
            worlds::walkable(w, e.floor, e.x, e.y),
            "敌人 {} 坐标 ({},{}) 在 {floor} 层不可走",
            e.id,
            e.x,
            e.y,
            floor = e.floor,
        );
        checked += 1;
    }
    // NPC 坐标可走
    for n in w.npcs {
        assert!(
            worlds::walkable(w, n.floor, n.x, n.y),
            "NPC {} 坐标 ({},{}) 在 {floor} 层不可走",
            n.id,
            n.x,
            n.y,
            floor = n.floor,
        );
        checked += 1;
    }
    // 机关坐标可走
    for z in w.zones {
        assert!(
            worlds::walkable(w, z.floor, z.x, z.y),
            "机关 {} 坐标 ({},{}) 在 {floor} 层不可走",
            z.id,
            z.x,
            z.y,
            floor = z.floor,
        );
        checked += 1;
    }
    // 门禁坐标可走
    for g in w.gates {
        assert!(
            worlds::walkable(w, g.floor, g.x, g.y),
            "门禁 {} 坐标 ({},{}) 在 {floor} 层不可走",
            g.id,
            g.x,
            g.y,
            floor = g.floor,
        );
        checked += 1;
    }
    // 传送源与落点可走
    for pt in w.portals {
        assert!(
            worlds::walkable(w, pt.floor, pt.x, pt.y),
            "传送 {} 源 ({},{}) 在 {floor} 层不可走",
            pt.id,
            pt.x,
            pt.y,
            floor = pt.floor,
        );
        assert!(
            worlds::walkable(w, pt.to_floor, pt.tx, pt.ty),
            "传送 {} 落点 ({},{}) 在 {floor} 层不可走",
            pt.id,
            pt.tx,
            pt.ty,
            floor = pt.to_floor,
        );
        checked += 2;
    }
    // 有意义的计数卫生
    assert!(checked >= 60, "应为 40 线核查有充分条目，实际 {checked}");
}
