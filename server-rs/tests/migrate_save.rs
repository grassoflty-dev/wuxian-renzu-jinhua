//! 存档迁移 v1→v2 集成测试（P0 验收项）
//! 全部使用内存 JSON 构造，不写盘。依赖 serde_json（Cargo.toml 已含）。
use std::collections::BTreeSet;
use wuxian_horror_ch1::state::{self, GameState, WorldRuntime};
use wuxian_horror_ch1::world;

fn v1_json() -> serde_json::Value {
    serde_json::json!({
        "hp": 82, "san": 60, "points": 430, "weapon": "Axe", "ammo": 4,
        "gene_lock": false, "gene_lock_used": false,
        "flags": { "A": true, "B1": true, "B2": false, "C": false, "drain_done": true },
        "dead_team": ["one"],
        "scene_id": "s_office", "laser_fails": 1, "fight": null,
        "px": 14, "py": 9, "floor": 2,
        "inventory": ["lab_badge", "shotgun_shell"],
        "map_objs": { "gate_b_area": true, "gate_vent": false },
        "enemies_alive": { "e_f4_boss": false, "e_f1_z1": true },
        "zone": null,
        "explored": ["2:14:9", "0:3:3"],
        "mode": "Normal", "pending_death": null, "settle_total": 0, "settle_rank": "D"
    })
}

/// 用例1：完整 v1 存档 → migrate → 生化进度一个不丢
#[test]
fn migrate_v1_save_complete() {
    // 反序列化 v1 形态：新字段走 default（world_id=default_world_id=biohazard_ch1，save_version=0）
    let mut st: GameState = serde_json::from_value(v1_json()).expect("v1 json 应可反序列化");
    assert_eq!(st.world_id, "biohazard_ch1", "缺省 world_id 回填生化");
    assert_eq!(st.save_version, 0, "v1 存档 save_version 缺省为 0");
    assert!(st.sp_grade.is_none());
    assert!(st.world_states.is_empty());

    let rewritten = state::migrate_save(&mut st);
    // R5 版本号
    assert_eq!(st.save_version, 2);
    assert_eq!(rewritten, 2, "两条 v1 explored key 被重写");
    // R2 探索迷雾：key 前插世界前缀，数量不变
    assert_eq!(st.explored.len(), 2);
    assert!(st.explored.contains(&"biohazard_ch1:2:14:9".to_string()));
    assert!(st.explored.contains(&"biohazard_ch1:0:3:3".to_string()));
    assert!(!st.explored.contains(&"2:14:9".to_string()));
    // 楼层与坐标保留
    assert_eq!(st.floor, 2);
    assert_eq!(st.px, 14);
    assert_eq!(st.py, 9);
    // 旗标/道具/地图对象/敌人/队友/数值一个不丢
    assert!(st.flag("A") && st.flag("B1") && !st.flag("B2") && st.flag("drain_done"));
    assert!(st.inventory.iter().any(|i| i == "lab_badge"));
    assert!(st.inventory.iter().any(|i| i == "shotgun_shell"));
    assert_eq!(st.map_objs.get("gate_b_area"), Some(&true));
    assert_eq!(st.map_objs.get("gate_vent"), Some(&false));
    assert_eq!(st.enemies_alive.get("e_f4_boss"), Some(&false));
    assert_eq!(st.enemies_alive.get("e_f1_z1"), Some(&true));
    assert!(st.dead_team.iter().any(|d| d == "one"));
    assert_eq!(st.points, 430);
    assert!(st.sp_grade.is_none(), "v1 无 sp_grade，迁移后仍为 None");
}

/// 用例2：仅重写旧前缀键、保已前缀键、不双前缀、幂等
#[test]
fn migrate_rewrites_explored_prefix_only_and_keeps_prefixed() {
    // 混入旧 key "1:5:5" 与已前缀 key "biohazard_ch1:0:1:5"
    let explored: BTreeSet<String> = ["1:5:5".to_string(), "biohazard_ch1:0:1:5".to_string()]
        .into_iter().collect();
    let mut st: GameState = serde_json::from_value(serde_json::json!({
        "world_id": "biohazard_ch1", "save_version": 0,
        "explored": explored, "px": 1, "py": 1, "floor": 0,
    })).expect("反序列化");
    let n = state::migrate_save(&mut st);
    assert_eq!(n, 1, "仅 1 条旧键被重写");
    assert_eq!(st.explored.len(), 2, "总数不变");
    assert!(st.explored.contains(&"biohazard_ch1:1:5:5".to_string()), "旧键前插");
    assert!(!st.explored.contains(&"1:5:5".to_string()), "旧键不再保留裸格式");
    assert!(st.explored.contains(&"biohazard_ch1:0:1:5".to_string()), "已前缀键不变");
    assert!(!st.explored.contains(&"biohazard_ch1:biohazard_ch1:0:1:5".to_string()), "不双前缀");

    // 幂等：save_version 已 2 → 二次迁移不改写
    let before: BTreeSet<String> = st.explored.iter().cloned().collect();
    let n2 = state::migrate_save(&mut st);
    assert_eq!(n2, 0, "幂等屏障：已 v2 不重复迁移");
    assert_eq!(st.explored, before, "二次迁移 explored 完全不变");
}

/// 用例3：新轮回默认值与 serde default 回填
#[test]
fn migrate_default_world_id_and_new_game_default() {
    let st = GameState::new();
    assert_eq!(st.world_id, "biohazard_ch1");
    assert_eq!(st.save_version, 2);
    assert!(st.world_states.is_empty());
    assert!(st.sp_grade.is_none());

    // 空结构反序列化 → 新字段走 default（world_id 经 default_world_id 回填）
    let empty: GameState = serde_json::from_value(serde_json::json!({})).expect("空 json");
    assert_eq!(empty.world_id, "biohazard_ch1", "serde default 回填 world_id");
    assert_eq!(empty.save_version, 0, "save_version 缺省 0");
    assert!(empty.sp_grade.is_none());
    assert!(empty.world_states.is_empty());
}

/// 可选加分：switch_world 快照/载入运行时状态
#[test]
fn switch_world_snapshots_and_restores_runtime() {
    let mut st = GameState::new();
    st.world_id = "biohazard_ch1".to_string();
    st.map_objs.insert("gate_b_area".to_string(), true);
    st.enemies_alive.insert("e_f4_boss".to_string(), true);

    // 切到未注册世界 zhutianshenkong
    world::switch_world(&mut st, "zhutianshenkong");
    assert_eq!(st.world_id, "zhutianshenkong");
    assert!(st.map_objs.is_empty(), "zhutian 未注册 → 空表");
    assert!(st.enemies_alive.is_empty(), "zhutian 未注册 → 空表");
    // 原世界快照已存
    let snap = st.world_states.get("biohazard_ch1").expect("快照存在");
    assert!(snap.entered);
    assert_eq!(snap.map_objs.get("gate_b_area"), Some(&true));
    assert_eq!(snap.enemies_alive.get("e_f4_boss"), Some(&true));

    // 切回生化 → 运行时状态恢复 + ensure_enemies 补全敌人
    world::switch_world(&mut st, "biohazard_ch1");
    assert_eq!(st.world_id, "biohazard_ch1");
    assert_eq!(st.map_objs.get("gate_b_area"), Some(&true), "map_objs 恢复");
    assert_eq!(st.enemies_alive.get("e_f4_boss"), Some(&true), "敌人状态恢复");
    // ensure_enemies 补缺所有生化敌人（e_f1_z1 等原缺省被补为 true）
    assert_eq!(st.enemies_alive.get("e_f1_z1"), Some(&true));

    // WorldRuntime 可直接构造/序列化
    let rt = WorldRuntime {
        map_objs: Default::default(),
        enemies_alive: Default::default(),
        entered: true,
    };
    let _j = serde_json::to_string(&rt).expect("WorldRuntime 可序列化");
}