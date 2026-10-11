//! 《无限曙光 · 生化危机·浣熊市》地面城市战副本 · 集成测试。
//! 依赖主神线在合并阶段把 HUANXIONGSHI_SCENES 并入 scenes::scene()、
//! 把 huanxiongshi_figths() 并入 scenes::fight_cfg()（并注册 crate::scenes_huanxiongshi mod）、
//! 并在 worlds/mod.rs 注册 WORLD_HUANXIONGSHI（id="huanxiongshi"），
//! 保证 engine::goto / engine::choose 能解析 hx_* 场景与 hx_* 战斗（见 tools/design/huanxiongshi_impl_log.md ★外部依赖）。
//! 测试内容（3 确定性用例，避开随机原生战斗，仅走选择驱动与剧情链）：
//!   ① 地图可达：三层每行 40、出生点可走、POINTS/敌人/角色/战圈/传送门/门禁坐标可行走
//!   ② 分发接线：scene("hx_00") / scene("hx_settle_card") 可解析；fight_cfg("hx_tyrant") HP200 可选
//!   ③ 主神线：序 → 警局取钥匙 → 街道 → 城郊 → 恐惧核弹 → 抉择(徒步逃离) → 结算卡片(sp_grade=D)
use wuxian_horror_ch1::{engine, scenes, state::GameState};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_HUANXIONGSHI).expect("浣熊市世界已注册（合并阶段）")
}

fn pick(st: &GameState, keyword: &str) -> i32 {
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

fn step(st: &mut GameState, keyword: &str) {
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    let idx = pick(st, keyword);
    engine::choose(st, idx, &mut deaths);
    println!("STEP [{keyword}] → {} (hp={} san={} pts={} floor={}sp={:?})",
        st.scene_id, st.hp, st.san, st.points, st.floor, st.sp_grade);
    assert!(st.san >= 0 && st.hp >= 0, "数值越界 scene={}", st.scene_id);
}

/* ---------------- ① 地图可达性 ---------------- */
#[test]
fn huanxiongshi_map_reachable() {
    let w = world();
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
        }
    }
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (1, 1), "出生点应为 F1 警局 (1,1)，实际 ({sx},{sy})");
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "调查点 {}@L{}({},{}) 不可走动", p.id, p.floor + 1, p.x, p.y);
    }
    for e in w.enemies {
        assert!(worlds::walkable(w, e.floor, e.x, e.y), "敌人 {}@L{}({},{}) 不可走动", e.id, e.floor + 1, e.x, e.y);
    }
    for n in w.npcs {
        assert!(worlds::walkable(w, n.floor, n.x, n.y), "角色 {}@L{} 不可走动", n.id, n.floor + 1);
    }
    for z in w.zones {
        assert!(worlds::walkable(w, z.floor, z.x, z.y), "战圈 {}@L{}({},{}) 不可走动", z.id, z.floor + 1, z.x, z.y);
    }
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.floor, pt.x, pt.y), "传送门 {}@L{}({},{}) 不可走动", pt.id, pt.floor + 1, pt.x, pt.y);
        assert!(worlds::walkable(w, pt.to_floor, pt.tx, pt.ty), "传送门 {} 落点@L{}({},{}) 不可走动", pt.id, pt.to_floor + 1, pt.tx, pt.ty);
    }
    for g in w.gates {
        assert!(worlds::walkable(w, g.floor, g.x, g.y), "门禁 {}@L{}({},{}) 不可走动", g.id, g.floor + 1, g.x, g.y);
    }
    println!("MAP REACHABLE OK · floors={} points={} enemies={} npcs={} zones={} portals={} gates={} spawn=({sx},{sy})",
        w.floors.len(), w.points.len(), w.enemies.len(), w.npcs.len(), w.zones.len(), w.portals.len(), w.gates.len());
}

/* ---------------- ② 分发接线（scene + fight） ---------------- */
#[test]
fn huanxiongshi_dispatch_wired() {
    assert!(scenes::scene("hx_00").is_some(), "缺 hx_00");
    assert!(scenes::scene("hx_f1_hub").is_some(), "缺 hx_f1_hub");
    assert!(scenes::scene("hx_settle_card").is_some(), "缺 hx_settle_card");
    assert!(scenes::scene("hx_nuke_death").is_some(), "缺 hx_nuke_death");
    // 战斗表
    let fights = wuxian_horror_ch1::scenes_huanxiongshi::huanxiongshi_figths();
    assert!(!fights.is_empty(), "战斗表不得为空");
    for (id, _) in fights {
        assert!(scenes::fight_cfg(id).is_some(), "fight {id} 分发闭环");
    }
    let tyr = fights.iter().find(|(k, _)| *k == "hx_tyrant").map(|(_, c)| c).unwrap();
    assert_eq!(tyr.hp, 200, "暴君 HP200 首版落地");
    assert_eq!(tyr.rage_at, Some(80), "暴君狂暴阈 80");
    assert_eq!(tyr.reward, 400, "暴君奖励 400");
    println!("DISPATCH WIRED OK · scenes hx_* + fights={} (tyrant hp={})", fights.len(), tyr.hp);
}

/* ---------------- ③ 主神线：秩序取钥匙 → 城郊 → 抉择逃离 → 结算 ---------------- */
#[test]
fn huanxiongshi_mainline_escape() {
    let mut st = GameState::new();
    st.hp = 300;
    st.san = 100;

    engine::goto(&mut st, "hx_00", &mut Vec::new());
    assert_eq!(st.scene_id, "hx_00");

    step(&mut st, "走上前，查看警局大厅");   // → hx_f1_hub
    assert_eq!(st.scene_id, "hx_f1_hub");

    // 撬开保险柜 → 得哨所钥匙 + 手枪
    step(&mut st, "撬开枪械保险柜");         // → hx_f1_lockup
    step(&mut st, "取走手枪与哨所钥匙");       // → hx_f1_hub, 得 it_hx_gatekey
    assert!(st.inventory.iter().any(|i| i == "it_hx_gatekey"), "应得哨所钥匙");

    // 突围至街道
    step(&mut st, "从警局侧门突围");          // → hx_f2_arrive
    step(&mut st, "（进入街道枢纽）");          // → hx_f2_hub

    // 冲向城郊路障（需哨所钥匙）
    step(&mut st, "冲向城郊路障");            // → hx_f3_arrive
    step(&mut st, "（探查城郊）");              // → hx_f3_hub
    assert_eq!(st.scene_id, "hx_f3_hub");

    // 核弹倒计时室 → 立即撤离 → 抉择
    step(&mut st, "走向核弹倒计时室");         // → hx_nuke_room
    step(&mut st, "立即撤离");                // → hx_ending_choice

    // 徒步逃离
    step(&mut st, "徒步冲出封锁线");           // → hx_settle_card
    assert_eq!(st.scene_id, "hx_settle_card", "应回结算卡片，实际 {}", st.scene_id);
    assert!(matches!(st.mode, wuxian_horror_ch1::state::Mode::AwaitCard(_)), "应落在结算卡片");
    assert_eq!(st.sp_grade, Some('D'), "应写 D 级评级");
    assert!(st.points >= 400, "逃离应得退出点数");

    println!("MAIN LINE (取钥匙 → 城郊 → 逃离 → 结算) OK · pts={} grade={:?} scene={}", st.points, st.sp_grade, st.scene_id);
}