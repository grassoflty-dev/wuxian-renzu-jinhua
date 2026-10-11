//! 《心慌方》CUBE 集成测试（xinhuangfang 副本）。
//! 依赖主线已把 XINHUANGFANG_SCENES 并入 scenes::scene()、把 xinhuangfang_figths() 并入 scenes::fight_cfg()、
//! 并在 worlds/mod.rs 注册 WORLD_XINHUANGFANG（合并后运行）。测试覆盖三层地图可达、调度接线与机关流出口可到达。
use wuxian_horror_ch1::worlds;
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::scenes_xinhuangfang;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_XINHUANGFANG).expect("心慌方世界已注册（合并后）")
}

/* ---------------- ① 三层地图可达（行宽 40 + 出生点/调查点/传送落点可走） ---------------- */
#[test]
fn xinhuangfang_map_reachable() {
    let w = world();
    assert_eq!(w.id, wuxian_horror_ch1::worlds::WORLD_XINHUANGFANG);
    assert_eq!(w.floors.len(), 3, "心慌方应为 3 层");
    for (fi, map) in w.floors.iter().enumerate() {
        assert_eq!(map.len(), 26, "floor{fi} 应 26 行");
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    let (sx, sy) = w.spawn();
    assert!(worlds::walkable(w, 0, sx, sy), "出生点 ({sx},{sy}) 应可走动");
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "调查点 {}@L{}({},{}) 不可走动", p.id, p.floor + 1, p.x, p.y);
    }
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.to_floor, pt.tx, pt.ty), "传送门 {} 落点@L{}({},{}) 不可走动", pt.id, pt.to_floor + 1, pt.tx, pt.ty);
    }
    println!("MAP REACHABLE OK · floors={} points={} enemies={} portals={} spawn=({sx},{sy})",
        w.floors.len(), w.points.len(), w.enemies.len(), w.portals.len());
}

/* ---------------- ② 调度接线 ---------------- */
#[test]
fn xinhuangfang_dispatch_wired() {
    assert!(scenes::scene("xf_00").is_some(), "xf_00 应在 scenes::scene() 可解析");
    assert!(scenes::scene("xf_34_open").is_some(), "xf_34_open（出口之门开）应可解析");
    assert!(scenes::scene("xf_38_ending").is_some(), "xf_38_ending（开放结局）应可解析");
    assert!(scenes::fight_cfg("xf_guard").is_some(), "xf_guard（铁灰巡卫）应可解析");
    assert!(scenes::fight_cfg("xf_kanshi").is_some(), "xf_kanshi（考验者·可选战）应可解析");
    println!("DISPATCH WIRED OK");
}

/* ---------------- ③ 战斗表分发闭环 ---------------- */
#[test]
fn xinhuangfang_fight_table_complete() {
    let fights = scenes_xinhuangfang::xinhuangfang_figths();
    assert!(fights.len() > 0, "应有战斗表");
    for (id, _) in fights {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
    // 映射到地图敌人的 fight id 必须全部有 cfg
    let w = world();
    for e in w.enemies {
        assert!(scenes::fight_cfg(e.fight).is_some(), "地图敌人 {} 的 fight {} 缺 cfg", e.id, e.fight);
    }
    println!("FIGHT TABLE COMPLETE OK · {} 条战斗", fights.len());
}

/* ---------------- ④ 机关流出口可到达（数字对路三组 → 出口之门） ---------------- */
#[test]
fn xinhuangfang_trap_escape() {
    let mut st = wuxian_horror_ch1::state::GameState::new();
    st.hp = 2000;
    st.san = 100;
    wuxian_horror_ch1::engine::goto(&mut st, "xf_00", &mut Vec::new());
    assert_eq!(st.scene_id, "xf_00");

    fn pick(st: &wuxian_horror_ch1::state::GameState, keyword: &str) -> i32 {
        let scene = scenes::scene(&st.scene_id).expect("scene");
        let visible: Vec<_> = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(st))).collect();
        for (i, c) in visible.iter().enumerate() {
            if c.label.contains(keyword) {
                return i as i32;
            }
        }
        panic!("scene {} 未找到含「{}」的选项；可见: {:?}",
            st.scene_id, keyword, visible.iter().map(|c| c.label).collect::<Vec<_>>());
    }
    fn step(st: &mut wuxian_horror_ch1::state::GameState, keyword: &str) {
        let mut deaths: Vec<(&'static str, &'static str)> = vec![];
        let idx = pick(st, keyword);
        let before = st.scene_id.clone();
        wuxian_horror_ch1::engine::choose(st, idx, &mut deaths);
        println!("STEP [{keyword}] {} → {} (hp={} san={} pts={})", before, st.scene_id, st.hp, st.san, st.points);
        assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
    }

    // 复刻启动层投井下数字（跳过随机战斗，纯机关流路径）
    step(&mut st, "环顾这间房间");          // → xf_01_first
    step(&mut st, "取走纸上的数字批注");      // it_xf_num_note → xf_10_arrive
    // 数字线索（一）（二）（三）
    step(&mut st, "铁灰房间壹");            // xf_num_1
    step(&mut st, "记下 07 · 14");
    step(&mut st, "墙上画号");              // xf_num_2
    step(&mut st, "抄下这条笔记");
    step(&mut st, "褪色序列");              // xf_num_3
    step(&mut st, "推算出数列第七项");
    // 断言三组数字 flag 齐 → 可开画号门甲
    assert!(st.flag("xf_num_1") && st.flag("xf_num_2") && st.flag("xf_num_3"), "三组数字对路应齐");
    step(&mut st, "走向更深处");            // → xf_12_gate
    step(&mut st, "（推门踏入中层）");        // → xf_20_arrive
    // 出口之门需要 xf_nums_done（三组数字摆满）；机关流路径经门乙 → 出口层
    step(&mut st, "深入编号回廊");          // → xf_22_gate
    step(&mut st, "（踏入出口层）");          // → xf_30_arrive
    // 出口层数字线索（三）拼图 + 尝试开门
    step(&mut st, "编号墙");                // → xf_32_num
    step(&mut st, "取得第三组数字");
    step(&mut st, "尽头的门");              // → xf_33_gate
    step(&mut st, "嵌入三组数字");           // → xf_34_open
    step(&mut st, "（迈出最后一步）");         // xf_route_exit → xf_50_card
    assert!(st.flag("xf_ending_done"), "应置结局 flag");
    assert!(matches!(st.mode, wuxian_horror_ch1::state::Mode::AwaitCard(_)), "应落在探界结算卡片");
    assert!(st.sp_grade.is_some(), "结算应写评级");
    println!("TRAP ESCAPE OK · pts={} grade={:?} scene={}", st.points, st.sp_grade, st.scene_id);
}