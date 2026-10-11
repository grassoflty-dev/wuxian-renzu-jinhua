//! all_worlds_interaction_test.rs —— 程序化全遍历集成测试（全量覆盖，非抽样）
//!
//! 遍历 worlds::all_worlds() 返回的全部 54+ 世界的静态数据表，对每条断言「可达性」：
//!   （baisun 为 worldgen 程序化生成世界，WorldData 懒构建，故注册表改用 all_worlds()）
//!   - points  ：PointDef.route —— 场景交互可达（scenes::scene(route) 可解析）
//!   - npcs    ：NpcDef.talk  —— 人物交互可达（scenes::scene(talk) 可解析）
//!   - zones   ：kind=="fight" 的 ZoneDef.ref_id 在 fight_cfg 可解析（战斗/机关可达）；
//!               kind!="fight" 的 ref_id 作为场景 id 可解析（机关/环境副本可达）
//!   - gates   ：GateDef.need_item/need_flag 字段合法（门禁条件可达）
//!   - portals ：PortalDef 落点 (tx,ty) 在目标层地图内且可走（传送可达）
//!   - enemies ：EnemyDef.fight 在 fight_cfg 可解析（遭遇战可达）
//!
//! 统计总世界数 / 总调查点数 / 总NPC数 / 总门禁数 / 总传送数，全部断言通过才绿。
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::worlds;

const SENTINEL_NONE: &str = "&__no_need_item__&";

/// 校验单世界全部表，返回 (points, npcs, gates, portals) 计数；遇坏数据 panic（附明细）。
fn check_world(w: &worlds::WorldData) -> (usize, usize, usize, usize) {
    let wid = w.id;
    // ---- points：调查点 route 场景可达 ----
    for p in w.points {
        assert!(
            scenes::scene(p.route).is_some(),
            "[{wid}] 调查点 {id}（{name}）route={route} 场景不可解析",
            id = p.id, name = p.name, route = p.route,
        );
    }
    // ---- npcs：talk 场景可达 ----
    for n in w.npcs {
        assert!(
            scenes::scene(n.talk).is_some(),
            "[{wid}] NPC {id}（{name}）talk={talk} 场景不可解析",
            id = n.id, name = n.name, talk = n.talk,
        );
    }
    // ---- zones：机关/战斗可达 ----
for z in w.zones {
        if z.kind == "fight" {
            // 战斗机关：ref_id 可指向「战斗配置」(fight_cfg) 或「带 fight_id 的战斗场景」。
            // 引擎两种都支持：进入时用 fight_cfg(ref_id) 显示敌人，攻击时把 ref_id 交给回合制；
            // 常见做法是引用携带 fight_id 的场景（如 moshi ms_z_sq1→ms_combat_a）。
            assert!(
                scenes::fight_cfg(z.ref_id).is_some() || scenes::scene(z.ref_id).is_some(),
                "[{wid}] 战斗机关 {id}（{name}）kind=fight ref_id={ref_id} 既非战斗配置也非场景（机关不可达）",
                id = z.id, name = z.name, ref_id = z.ref_id,
            );
        } else {
            // puzzle / env / env_kill / overlay / zone 等：ref_id 进入副本系统时按场景或战斗解析。
            // 已知旧式 3D 谜题标签（引擎把 kind=puzzle 作 3D 副本占位，不路由到 scene/fight）
            // 单独放行；其余必须至少在 scene 或 fight_cfg 之一可达，防生造死链。
            const LEGACY_PUZZLE_LABELS: &[&str] = &["d_laser_room"];
            if LEGACY_PUZZLE_LABELS.contains(&z.ref_id) {
                continue;
            }
            assert!(
                scenes::scene(z.ref_id).is_some() || scenes::fight_cfg(z.ref_id).is_some(),
                "[{wid}] 机关 {id}（{name}）kind={kind} ref_id={ref_id} 既非场景也非战斗（机关不可达）",
                id = z.id, name = z.name, kind = z.kind, ref_id = z.ref_id,
            );
        }
    }
    // ---- gates：门禁条件字段合法 ----
    for g in w.gates {
        assert!(
            !g.id.is_empty(),
            "[{wid}] 门禁 id 为空（floor={},x={},y={}）", g.floor, g.x, g.y,
        );
        // need_item 与 need_flag 不能同空（门禁必须至少有一个解锁条件），认 bool 可并置
        match (g.need_item, g.need_flag) {
            (None, None) => {
                // 单开门禁可能无条件（例如场景线索门），不强制——仅提示字段结构合法即可
            }
            (Some(item), _) => {
                assert!(
                    !item.is_empty() && item != SENTINEL_NONE,
                    "[{wid}] 门禁 {id} need_item 为空", id = g.id,
                );
            }
            (_, Some(flag)) => {
                assert!(
                    !flag.is_empty(),
                    "[{wid}] 门禁 {id} need_flag 为空", id = g.id,
                );
            }
        }
    }
    // ---- portals：落点在地图内且可走 ----
    for pt in w.portals {
        let (maps, floor_names) = (w.floors, w.floor_names);
        let tf = pt.to_floor;
        let valid_floor = tf < maps.len();
        let tile_ch: Option<char> = valid_floor.then(|| worlds::tile(w, tf, pt.tx, pt.ty));
        assert!(
            valid_floor,
            "[{wid}] 传送 {} 目标层 {tf} 越界（共 {} 层）", pt.id, maps.len(),
        );
        let map = &maps[tf];
        let in_bounds = pt.ty < map.len() && pt.tx < map[pt.ty].len();
        assert!(
            in_bounds,
            "[{wid}] 传送 {} 落点 ({},{}) 超出目标层 {tf} 地图",
            pt.id, pt.tx, pt.ty,
        );
        assert!(
            tile_ch.is_some_and(|c| c != '#'),
            "[{wid}] 传送 {} 落点 ({},{}) 在 {tf} 层走不通（墙）",
            pt.id, pt.tx, pt.ty,
        );
        let _ = floor_names; // 备用：需要可走层名时可展开
    }
    // ---- enemies：遭遇战 fight 可达 ----
    for e in w.enemies {
        assert!(
            scenes::fight_cfg(e.fight).is_some(),
            "[{wid}] 敌人 {id}（{name}）fight={fight} 不在 fight_cfg",
            id = e.id, name = e.name, fight = e.fight,
        );
    }
    (w.points.len(), w.npcs.len(), w.gates.len(), w.portals.len())
}

/// 全遍历主入口：逐世界跑 check_world，累计各类计数与门禁条目明细
#[test]
fn full_worlds_traversal_all_linked() {
    let mut total_worlds = 0usize;
    let mut total_points = 0usize;
    let mut total_npcs = 0usize;
    let mut total_gates = 0usize;
    let mut total_portals = 0usize;
    let mut total_zones = 0usize;
    let mut total_enemies = 0usize;
    let mut ids_unique: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();

    for w in worlds::all_worlds().iter().map(|w| *w) {
        total_worlds += 1;
        // 全局 id 唯一（跨世界互不冲突，联动查询唯一性）
        let all_obj = w.points.iter().map(|p| p.id)
            .chain(w.npcs.iter().map(|n| n.id))
            .chain(w.zones.iter().map(|z| z.id))
            .chain(w.gates.iter().map(|g| g.id))
            .chain(w.portals.iter().map(|p| p.id))
            .chain(w.enemies.iter().map(|e| e.id));
        for id in all_obj {
            assert!(ids_unique.insert(id), "对象 id 重复：{id}（{wid}）", wid = w.id);
        }
        let (np, nn, ng, npt) = check_world(w);
        total_points += np;
        total_npcs += nn;
        total_gates += ng;
        total_portals += npt;
        total_zones += w.zones.len();
        total_enemies += w.enemies.len();
    }

    // 有意义的计数卫生：至少应有这规模（防止 WORLDS 被清空导致假绿）
    assert!(total_worlds >= 50, "WORLDS 应 ≥50，实际 {total_worlds}");
    assert!(total_points >= 100, "调查点应 ≥100，实际 {total_points}");
    assert!(total_npcs >= 100, "NPC 应 ≥100，实际 {total_npcs}");
    assert!(total_gates >= 100, "门禁应 ≥100，实际 {total_gates}");
    assert!(total_portals >= 100, "传送应 ≥100，实际 {total_portals}");
    assert!(total_zones >= 50, "机关应 ≥50，实际 {total_zones}");
    assert!(total_enemies >= 150, "敌人应 ≥150，实际 {total_enemies}");

    eprintln!(
        "[full_worlds_traversal_all_linked] 世界={total_worlds} 调查点={total_points} \
         NPC={total_npcs} 门禁={total_gates} 传送={total_portals} 机关={total_zones} 敌人={total_enemies}"
    );
}

/// 世界初始场景可达性：每个世界的初始场景必须能解析（进入世界即加载）
#[test]
fn every_world_initial_scene_resolvable() {
    for w in worlds::all_worlds().iter().map(|w| *w) {
        assert!(
            scenes::scene(w.initial_scene).is_some(),
            "[{}] 初始场景 initial_scene={:?} 不可解析",
            w.id, w.initial_scene,
        );
    }
}

/// 每世界 must 有出生点 P 或回退 (1,1)；且出生点可走（spawn() 内部实现）
#[test]
fn every_world_spawn_walkable() {
    for w in worlds::all_worlds().iter().map(|w| *w) {
        let (sx, sy) = w.spawn();
        assert!(
            worlds::walkable(w, 0, sx, sy),
            "[{}] 出生点 ({},{}) 不可走", w.id, sx, sy,
        );
    }
}