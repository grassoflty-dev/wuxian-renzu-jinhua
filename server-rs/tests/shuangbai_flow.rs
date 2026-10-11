//! 《死亡开端·霜白村》流程测试：3 个确定性用例（不碰随机战斗）
//! 注意：需主线在 lib.rs 注册 `pub mod scenes_shuangbai;` 后方可编译运行（合并后运行）。

use wuxian_horror_ch1::worlds;
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::scenes_shuangbai;

#[test]
fn shuangbai_map_reachable() {
    let w = worlds::find_world(worlds::WORLD_SHUANGBAI).expect("世界已注册");
    // 两层 40×26
    assert_eq!(w.floors.len(), 2, "共两层");
    for f in 0..w.floors.len() {
        assert_eq!(w.floors[f].len(), 26, "floor {f} 高26");
        for row in w.floors[f] {
            assert_eq!(row.len(), 40, "floor {f} 行宽40");
        }
    }
    // 出生点可走
    let (px, py) = w.spawn();
    assert!(worlds::walkable(w, 0, px, py), "出生点可走");
    // 关键调查点(NPC/敌人/调查点)均应落在可走格
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "调查点 {} 可走", p.id);
    }
    for e in w.enemies {
        assert!(worlds::walkable(w, e.floor, e.x, e.y), "敌人 {} 可走", e.id);
    }
    for n in w.npcs {
        assert!(worlds::walkable(w, n.floor, n.x, n.y), "NPC {} 可走", n.id);
    }
    // 传送门入口可走
    for p in w.portals {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "传送门 {} 入口可走", p.id);
    }
}

#[test]
fn shuangbai_dispatch_wired() {
    // 关键场景可分发
    for id in ["sb_00", "sb_01", "sb_03", "sb_12", "sb_boss_round", "sb_settle", "sb_hub", "sb_death"] {
        assert!(scenes::scene(id).is_some(), "场景 {id} 已注册");
    }
    // 三场战斗分发闭环（先看到的是 fight_cfg 存在）
    for id in ["sb_fight_grey", "sb_fight_warped", "sb_boss"] {
        assert!(scenes::fight_cfg(id).is_some(), "战斗 {id} 已注册");
    }
}

#[test]
fn shuangbai_fight_table_complete() {
    assert!(scenes_shuangbai::shuangbai_figths().len() >= 3, "至少三场战斗");
    for (id, _) in scenes_shuangbai::shuangbai_figths() {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
}