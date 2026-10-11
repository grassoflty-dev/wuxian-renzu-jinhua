//! 无限轮回 · 第一章 生化蜂巢 —— Tauri 桌面应用
//! 游戏引擎全部运行在 Rust 侧；WebView 通过 invoke 调用命令驱动。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::{json, Value};
use state::GameState;
use std::{fs, sync::Mutex};
use wuxian_horror_ch1::formal_runtime::{
    CapabilityCommandRequest, CombatIntentRequest, FormalRuntime,
};
use wuxian_horror_ch1::world_v3::WorldView;
use wuxian_horror_ch1::{engine, maps, scenes, state, world, worlds, zone2d_gen};

/// 全局会话（单玩家单存档位）
struct Session(Mutex<GameState>);

fn data_dir() -> std::path::PathBuf {
    let exe = std::env::current_exe().expect("exe path");
    exe.parent().unwrap().join("data")
}

fn save_state(st: &GameState) {
    let dir = data_dir();
    let _ = fs::create_dir_all(&dir);
    use std::io::Write;
    let mut mark = String::from("attempt dir="); mark.push_str(dir.to_string_lossy().as_ref());
    match serde_json::to_string_pretty(st) {
        Ok(j) => {
            mark.push_str(&format!(" serlen={}", j.len()));
            match fs::write(dir.join("save.json"), &j) {
                Ok(_) => mark.push_str(" WRITE_OK"),
                Err(e) => mark.push_str(&format!(" WRITE_ERR={}", e)),
            }
        }
        Err(e) => mark.push_str(&format!(" SER_ERR={}", e)),
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("rpc.log")) { let _ = writeln!(f, "{}", mark); }
}

fn load_state() -> Option<GameState> {
    let p = data_dir().join("save.json");
    let raw = fs::read_to_string(p).ok()?;
    serde_json::from_str(&raw).ok()
}

fn record_deaths(deaths: &[(&'static str, &'static str)]) {
    if deaths.is_empty() { return; }
    let dir = data_dir();
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("deaths.json");
    let mut arr: Vec<Value> = fs::read_to_string(&path).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    for (t, c) in deaths {
        arr.insert(0, serde_json::json!({
            "t": t, "c": c, "time": chrono_now(),
        }));
    }
    arr.truncate(30);
    if let Ok(j) = serde_json::to_string_pretty(&arr) {
        let _ = fs::write(path, j);
    }
}

fn chrono_now() -> String {
    // 无外部依赖：使用标准库 SystemTime 格式化
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    // 东八区
    let (y, mo, d, h, mi) = civil_from_secs(secs + 8 * 3600);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}")
}

fn civil_from_secs(secs: u64) -> (u64, u64, u64, u64, u64) {
    let days = secs / 86400;
    let rem = secs % 86400;
    let h = rem / 3600; let mi = (rem % 3600) / 60;
    // Howard Hinnant 算法
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d, h, mi)
}

/* ---------------- Tauri 命令 ---------------- */

#[tauri::command]
fn api_new(session: tauri::State<Session>) -> Value {
    rpc_log("api_new");
    // 轮回记忆：重开轮回时继承上次轮回的已探索地图（无限流设定）
    // 防御性迁移读取的旧档（把 v1 explored key 升格为带世界前缀）
    let prev_explored = load_state().map(|mut s| {
        let _ = state::migrate_save(&mut s);
        s.explored
    }).unwrap_or_default();
    let mut st = GameState::new();
    st.scaling_enabled = true; // 游戏运行时开启动态难度缩放（敌人强度 = 主角强度 × 副本难度系数）
    st.explored = prev_explored;
    let mut deaths = vec![];
    // 开放世界模式：直接进入蜂巢地图
    world::world_init(&mut st);
    world::ensure_enemies(&mut st);
    // 保留开场：先进入办公室序章对话（简化为地图入场）
    engine::goto(&mut st, "s_office", &mut deaths);
    record_deaths(&deaths);
    save_state(&st);
    *session.0.lock().unwrap() = st.clone();
    let mut v = engine::render(&st);
    v["world"] = world::world_view(&st);
    v
}

fn rpc_log(tag: &str) {
    let dir = data_dir();
    let _ = fs::create_dir_all(&dir);
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("rpc.log")) {
        let _ = writeln!(f, "{} {:?}", tag, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0));
    }
}

#[tauri::command]
fn api_continue(session: tauri::State<Session>) -> Result<Value, String> {
    rpc_log("api_continue");
    match load_state() {
        Some(mut st) => {
            // 存档迁移 v1→v2：幂等；迁移数写 rpc.log
            let n = state::migrate_save(&mut st);
            rpc_log(&format!("api_continue migrate_explored={}", n));
            world::ensure_enemies(&mut st);
            engine::rebuild_mode(&mut st);
            *session.0.lock().unwrap() = st.clone();
            Ok(engine::render(&st))
        }
        None => Err("没有找到存档。轮回尚未开始。".into()),
    }
}

#[tauri::command]
fn api_choose(index: i32, session: tauri::State<Session>) -> Value {
    rpc_log("api_choose");
    let mut deaths = vec![];
    {
        let mut guard = session.0.lock().unwrap();
        engine::choose(&mut *guard, index, &mut deaths);
    }
    let guard = session.0.lock().unwrap();
    record_deaths(&deaths);
    save_state(&guard);
    engine::render(&guard)
}

#[tauri::command]
fn api_nexus(session: tauri::State<Session>) -> Value {
    let st = session.0.lock().unwrap();
    let card = scenes::card_nexus_pub(&st);
    serde_json::json!({
        "kind": "card_only",
        "card": {
            "title": card.title, "good": card.good, "bodyHtml": card.body_html,
            "voice": null,
            "buttons": card.buttons.iter().map(|(l, r)| serde_json::json!({"label": l, "route": r}))
                .collect::<Vec<_>>(),
        },
        "hud": engine::hud_json(&st),
    })
}

/// 结算后进入主神空间世界（P1）：switch_world→zhutianshenkong→落点→world_view。
/// 结算写 bh_cleared（幂等；仅生化通关进入时写，即从生化世界切来）。
#[tauri::command]
fn api_nexus_enter(session: tauri::State<Session>) -> Value {
    rpc_log("api_nexus_enter");
    let mut st = session.0.lock().unwrap();
    // 仅生化通关进入主神时写通关旗标（主神世界内再调则已是 zhutianshenkong，不重写）
    if st.world_id == worlds::WORLD_BIOHAZARD {
        st.set_flag("bh_cleared");
    }
    st.zone = None; // 安全兜底：传送时强制关闭 3D 副本会话
    world::switch_world(&mut st, worlds::WORLD_ZHUTIAN);
    let spawn = worlds::find_world(worlds::WORLD_ZHUTIAN).unwrap().spawn();
    st.floor = 0;
    st.px = spawn.0;
    st.py = spawn.1;
    save_state(&st);
    engine::rebuild_mode(&mut st);
    let mut v = world::world_view(&st);
    // 世界元信息（与 api_world 一致），供前端 world2d 标题展示世界名
    let wmeta = worlds::find_world(&st.world_id).map(|w|
        json!({
            "id": w.id, "name": w.name, "difficulty": w.difficulty,
            "cleared": st.flag(&format!("{}_cleared", w.id)),
            "sp_grade": st.sp_grade,
        })
    ).unwrap_or_else(|| json!({
        "id": st.world_id, "name": st.world_id, "difficulty": 0,
        "cleared": false, "sp_grade": null,
    }));
    v["world"] = wmeta;
    v["hud"] = engine::hud_json(&st);
    v
}

#[tauri::command]
fn api_deaths() -> Value {
    let path = data_dir().join("deaths.json");
    let arr: Vec<Value> = fs::read_to_string(path).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    serde_json::json!({ "deaths": arr })
}

#[tauri::command]
fn api_has_save() -> bool {
    rpc_log("api_has_save");
    load_state().is_some()
}

/* ---------------- 开放世界命令 ---------------- */

/// 世界视图（2D 地图全量数据 + HUD）
#[tauri::command]
fn api_world(session: tauri::State<Session>) -> Value {
    rpc_log("api_world");
    let st = session.0.lock().unwrap();
    let mut v = world::world_view(&st);
    // 世界元信息（设计 §4.1）：id/名称/难度/通关/sp_grade
    let wmeta = worlds::find_world(&st.world_id).map(|w|
        json!({
            "id": w.id, "name": w.name, "difficulty": w.difficulty,
            "cleared": st.flag(&format!("{}_cleared", w.id)),
            "sp_grade": st.sp_grade,
        })
    ).unwrap_or_else(|| json!({
        "id": st.world_id, "name": st.world_id, "difficulty": 0,
        "cleared": false, "sp_grade": null,
    }));
    v["world"] = wmeta;
    v["hud"] = engine::hud_json(&st);
    v
}

/// 移动一格；若踩到敌人则返回遭遇（进入战斗副本）；走向传送门则切层
#[tauri::command]
fn api_world_move(dx: i32, dy: i32, session: tauri::State<Session>) -> Value {
    rpc_log("api_world_move");
    let mut st = session.0.lock().unwrap();
    let world = worlds::find_world(&st.world_id).unwrap();
    let (ok, tile_c, hit, teleported) = world::try_move(&mut st, dx, dy);
    let mut out = json!({
        "ok": ok, "tile": tile_c.to_string(), "px": st.px, "py": st.py,
        "floor": st.floor,
        "floor_name": world.floor_names[st.floor].to_string(),
        "teleported": teleported,
    });
    if let Some(hit) = hit {
        // 门禁挡路：返回锁定提示（不移动）——查当前世界门禁表
        if let Some(gid) = hit.strip_prefix("gate:") {
            if let Some(g) = world.gates.iter().find(|g| g.id == gid) {
                out["gate_blocked"] = json!({
                    "id": g.id, "name": g.name, "msg": g.lock_msg,
                    "need": g.need_item.or(g.need_flag),
                });
            }
        } else if let Some(def) = world.enemies.iter().find(|e| e.id == hit) {
            out["encounter"] = json!({ "enemy_id": hit, "fight_id": def.fight, "name": def.name });
        }
    }
    out["nearby"] = json!(world::nearby_interactables(&st));
    // 移动/切层后持久化
    save_state(&st);
    out
}

/// 蛛丝拉拽（蜘蛛侠血统）：点击目标墙格 → 校验血统/射程/视线 → 拉到目标旁开阔格。
#[tauri::command]
fn api_web_zip(tx: i32, ty: i32, session: tauri::State<Session>) -> Value {
    rpc_log("api_web_zip");
    let mut st = session.0.lock().unwrap();
    let has_sp = st.bloodline.as_deref() == Some("spiderman_bloodline")
        || st.bloodline_tier.contains_key("spiderman_bloodline");
    if !has_sp {
        return json!({ "ok": false, "reason": "need_bloodline", "msg": "需要蜘蛛侠血统才能喷射蛛丝。" });
    }
    let (old_x, old_y) = (st.px, st.py);
    let world = worlds::find_world(&st.world_id).unwrap();
    match world::try_web_zip(&mut st, tx, ty) {
        Err(reason) => json!({ "ok": false, "reason": reason }),
        Ok(_) => {
            let mut out = json!({
                "ok": true, "px": st.px, "py": st.py, "floor": st.floor,
                "floor_name": world.floor_names[st.floor].to_string(),
                "from": [old_x, old_y], "to": [st.px, st.py],
                "nearby": world::nearby_interactables(&st),
                "hud": engine::hud_json(&st),
            });
            for e in world.enemies {
                if e.floor == st.floor && e.x == st.px && e.y == st.py
                    && st.enemies_alive.get(e.id).copied().unwrap_or(true) {
                    out["encounter"] = json!({ "enemy_id": e.id, "fight_id": e.fight, "name": e.name });
                    break;
                }
            }
            save_state(&st);
            out
        }
    }
}

/// 与相邻对象交互（调查点/NPC/副本入口/传送门/跨世界网关）
/// 世界化：一律查当前世界（st.world_id）的对象表（设计 §5.1 硬约束），
/// 不再用 maps:: 全局生化表。跨世界网关走 worlds::GW_PORTALS（P1）。
#[tauri::command]
fn api_world_interact(obj_id: String, session: tauri::State<Session>) -> Value {
    rpc_log("api_world_interact");
    let mut st = session.0.lock().unwrap();
    let world = worlds::find_world(&st.world_id).unwrap();
    // 门禁：检查钥匙/旗标后解锁（软锁，绕行可达）
    if let Some(g) = world.gates.iter().find(|g| g.id == obj_id) {
        let unlocked = st.map_objs.get(g.id).copied().unwrap_or(false);
        let mut can_unlock = true;
        if let Some(item) = g.need_item {
            if !st.inventory.iter().any(|i| i == item) { can_unlock = false; }
        }
        if let Some(f) = g.need_flag {
            if !st.flag(f) { can_unlock = false; }
        }
        if can_unlock {
            if !unlocked {
                st.map_objs.insert(g.id.to_string(), true);
                save_state(&st);
            }
            return json!({
                "kind": "gate", "obj_id": obj_id, "opened": true, "msg": g.unlock_msg,
            });
        }
        return json!({
            "kind": "gate", "obj_id": obj_id, "opened": false, "msg": g.lock_msg,
            "need": g.need_item.or(g.need_flag),
        });
    }
    // 副本入口：直接进入 3D 会话（当前世界，P1 deny全场可战力为0，主神无它）
    if let Some(z) = world.zones.iter().find(|z| z.id == obj_id) {
        return zone_enter_inner(&mut st, z.id, z.kind, z.ref_id);
    }
    // 世界敌人：踩上后进入战斗副本
    if let Some(en) = world.enemies.iter().find(|e| e.id == obj_id) {
        let alive = st.enemies_alive.get(en.id).copied().unwrap_or(true);
        if !alive {
            return json!({ "kind": "enemy", "obj_id": obj_id, "dead": true, "msg": "它已经被你放倒了。" });
        }
        return zone_enter_inner(&mut st, en.id, "fight", en.fight);
    }
    // 跨世界网关（全局唯一 gw_*，P1：主神→生化可用 / 主神→咒怨占位）
    if let Some(gw) = worlds::gw_portal_by_id(&obj_id) {
        if !gw.available {
            return json!({
                "kind": "portal_world", "obj_id": obj_id, "available": false,
                "msg": "灰绿色的封印轻轻颤动——通往《咒怨》的传送门尚未开启（后续轮回解锁）。",
            });
        }
        if st.zone.is_some() {
            return json!({ "kind": "busy", "msg": "副本进行中，无法传送。" });
        }
        world::switch_world(&mut st, gw.to_world);
        st.floor = gw.to_floor;
        st.px = gw.tx;
        st.py = gw.ty;
        st.zone = None;
        save_state(&st);
        let target = worlds::find_world(&st.world_id);
        return json!({
            "kind": "portal_world", "obj_id": obj_id, "to_world": st.world_id,
            "floor": st.floor,
            "floor_name": target.map(|w| w.floor_names[st.floor].to_string()).unwrap_or_default(),
            "px": st.px, "py": st.py,
        });
    }
    // 调查点
    if let Some(p) = world.points.iter().find(|p| p.id == obj_id) {
        return json!({ "kind": "point", "scene": p.route, "obj_id": obj_id });
    }
    // NPC
    if let Some(n) = world.npcs.iter().find(|n| n.id == obj_id) {
        return json!({ "kind": "npc", "route": n.talk, "obj_id": obj_id });
    }
    // 传送门：楼层切换（当前世界；跨世界网关已在前处理）
    if let Some(pt) = world.portals.iter().find(|p| p.id == obj_id) {
        st.floor = pt.to_floor;
        st.px = pt.tx;
        st.py = pt.ty;
        save_state(&st);
        return json!({
            "kind": "portal", "floor": st.floor, "floor_name": world.floor_names[st.floor],
            "px": st.px, "py": st.py, "obj_id": obj_id,
        });
    }
    json!({ "kind": "unknown", "obj_id": obj_id })
}

/// 开始一个 3D 副本（战斗或解密）
fn zone_enter_inner(st: &mut GameState, zone_id: &str, kind: &str, ref_id: &str) -> Value {
    st.zone = Some(state::ZoneSession {
        zone_id: zone_id.to_string(),
        kind: kind.to_string(),
        ref_id: ref_id.to_string(),
        zx: 0.0, zz: 0.0, zyaw: 0.0,
        zhp: st.hp,
        progress: 0,
        last_action: String::new(),
    });
    save_state(st);
    let mut out = json!({
        "zone": { "id": zone_id, "kind": kind, "ref": ref_id },
        "hud": engine::hud_json(st),
    });
    // 战斗副本：给敌人初始信息 + 2D 横版地图（zone2d 用，确定性生成）
    if kind == "fight" {
        if let Some(cfg) = scenes::fight_cfg(ref_id) {
            out["enemy"] = json!({
                "name": cfg.name, "hp": cfg.hp, "max_hp": cfg.hp,
                "intro": cfg.intro,
            });
        }
        out["zone2d"] = zone2d_gen::gen_zone2d(zone_id);
    }
    out
}

/// 副本内实时动作（走位/攻击/交互）
#[tauri::command]
fn api_zone_action(action: String, arg: f32, session: tauri::State<Session>) -> Value {
    rpc_log("api_zone_action");
    let mut st = session.0.lock().unwrap();
    // 先克隆 zone 数据，避免借用冲突
    let zone_snapshot = match st.zone.clone() {
        Some(z) => z,
        None => return json!({ "error": "not_in_zone" }),
    };
    match action.as_str() {
        "move" => {
            // arg = 朝向角（弧度）；实时走位由前端插值，这里只更新朝向
            if let Some(z) = &mut st.zone { z.zyaw = arg; }
            json!({ "ok": true })
        }
        "attack" => {
            if zone_snapshot.kind == "puzzle" {
                // 解密副本：attack = 尝试破解（交由解密逻辑，当前为占位）
                if let Some(z) = &mut st.zone { z.progress += 1; z.last_action = "probe".into(); }
                return json!({ "puzzle": true, "ok": true, "msg": "解密副本：尚未找到破解方式", "progress": st.zone.as_ref().map(|z| z.progress) });
            }
            let fid = zone_snapshot.ref_id.clone();
            // 用现有 fight 系统初始化并执行一轮（zone 保留原值，战斗状态放 fight 字段）
            let view = run_zone_combat_round(&mut st, &fid);
            // 更新 zone 会话进度
            let won = st.fight.is_none() && !st.scene_id.starts_with("e_");
            let died = st.scene_id.starts_with("e_");
            let cur_hp = st.hp;
            if let Some(z) = &mut st.zone {
                z.progress += 1;
                z.zhp = cur_hp;
                z.last_action = if won { "win".into() } else if died { "dead".into() } else { "attack".into() };
            }
            let hud = engine::hud_json(&st);
            // 战斗状态持久化
            save_state(&st);
            if won {
                // 敌人死亡 → 从地图清除该敌人，关闭副本
                if let Some(z) = &st.zone {
                    let eid = zone_enemy_id(&z.zone_id);
                    if let Some(e) = eid { world::kill_enemy(&mut st, &e); }
                }
                st.zone = None;
                save_state(&st);
                json!({ "win": true, "view": view, "hud": hud })
            } else if died {
                // 玩家死亡 → 关闭副本，回死亡档案
                st.zone = None;
                save_state(&st);
                json!({ "dead": true, "view": view, "hud": hud, "scene": st.scene_id })
            } else {
                json!({ "win": false, "view": view, "hud": hud, "player_hp": st.hp })
            }
        }
        "dodge" => {
            // 实时闪避：本回合敌人攻击概率降低（前端展示，引擎简化处理）
            json!({ "ok": true, "dodged": true })
        }
        "use_item" => {
            let name = arg.to_string();
            if st.inventory.iter().any(|i| *i == name) {
                json!({ "ok": true, "used": name })
            } else {
                json!({ "ok": false, "reason": "no_item" })
            }
        }
        "skill" => {
            // 装备技能格结算：arg = slot（0/1/2）
            if zone_snapshot.kind == "puzzle" {
                return json!({ "ok": false, "error": "not_fight", "msg": "解密副本无法施展技能" });
            }
            let slot = arg as usize;
            match engine::do_zone_skill(&mut st, slot) {
                Err(reason) => json!({ "ok": false, "error": "skill_unusable", "reason": reason }),
                Ok(view) => {
                    // 与 attack 分支一致的胜负判定与 zone 会话更新
                    let won = st.fight.is_none() && !st.scene_id.starts_with("e_");
                    let died = st.scene_id.starts_with("e_");
                    let cur_hp = st.hp;
                    if let Some(z) = &mut st.zone {
                        z.progress += 1;
                        z.zhp = cur_hp;
                        z.last_action = if won { "win".into() } else if died { "dead".into() } else { "skill".into() };
                    }
                    let hud = engine::hud_json(&st);
                    save_state(&st);
                    if won {
                        if let Some(z) = &st.zone {
                            let eid = zone_enemy_id(&z.zone_id);
                            if let Some(e) = eid { world::kill_enemy(&mut st, &e); }
                        }
                        st.zone = None;
                        save_state(&st);
                        json!({ "win": true, "view": view, "hud": hud })
                    } else if died {
                        st.zone = None;
                        save_state(&st);
                        json!({ "dead": true, "view": view, "hud": hud, "scene": st.scene_id })
                    } else {
                        json!({ "win": false, "view": view, "hud": hud, "player_hp": st.hp })
                    }
                }
            }
        }
        "exit" => {
            st.zone = None;
            json!({ "ok": true, "world": world::world_view(&st) })
        }
        _ => json!({ "ok": false, "error": "unknown_action" }),
    }
}

/// zone_id → 敌人 id（战斗副本在地图上的敌人；zone_id 即敌人 id 时直接命中）
fn zone_enemy_id(zone_id: &str) -> Option<&'static str> {
    match zone_id {
        "z_licker" => Some("e_licker"),
        id => maps::ENEMIES.iter().find(|e| e.id == id).map(|e| e.id),
    }
}

/// 世界模式跳转到剧情场景（调查点/NPC 对话入口）
#[tauri::command]
fn api_scene_goto(scene_id: String, session: tauri::State<Session>) -> Value {
    rpc_log("api_scene_goto");
    let mut st = session.0.lock().unwrap();
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, &scene_id, &mut deaths);
    record_deaths(&deaths);
    save_state(&st);
    engine::render(&st)
}

/// 从剧情场景返回世界地图
#[tauri::command]
fn api_scene_back(session: tauri::State<Session>) -> Value {
    rpc_log("api_scene_back");
    let st = session.0.lock().unwrap();
    let mut v = world::world_view(&st);
    v["hud"] = engine::hud_json(&st);
    v
}

/// 副本内战斗一轮（复用现有回合制核心）
fn run_zone_combat_round(st: &mut GameState, fight_id: &str) -> Value {
    // 若 fight 未初始化，用现有 FIGHTS 配置初始化
    if st.fight.is_none() {
        let cfg = scenes::fight_cfg(fight_id);
        let (name, hp, dmg, reward, why, intro) = match cfg {
            Some(c) => (c.name, c.hp, c.dmg, c.reward, c.reward_why, c.intro),
            None => ("未知敌人", 30, (5, 10), 0, "", ""),
        };
        st.fight = Some(state::Fight {
            id: fight_id.to_string(),
            name: name.to_string(),
            hp, max_hp: hp,
            dmg, reward, reward_why: why.to_string(),
            raged: false, rage_at: None, guard_turn: false,
            pending_log: vec![format!("<span class='hit'>{intro}</span>")],
        });
        st.mode = state::Mode::Fight;
    }
    // 若处于觉醒卡片等待：点「睁开眼」恢复战斗
    if matches!(st.mode, state::Mode::AwaitCard(_)) {
        st.mode = state::Mode::Fight;
    }
    let mut deaths = vec![];
    // 玩家动作：优先终结技（敌人低血时），否则普通攻击
    let acts = engine::fight_actions(st);
    let idx = if acts.iter().any(|a| *a == "finisher") {
        acts.iter().position(|a| *a == "finisher").unwrap_or(0) as i32
    } else {
        acts.iter().position(|a| *a == "attack").unwrap_or(0) as i32
    };
    engine::choose(st, idx, &mut deaths);
    // 若战斗胜利后还有待结算的觉醒/死亡卡片，交由后续处理
    engine::render(st)
}

/// 离开副本回到世界地图
#[tauri::command]
fn api_zone_exit(session: tauri::State<Session>) -> Value {
    rpc_log("api_zone_exit");
    let mut st = session.0.lock().unwrap();
    st.zone = None;
    let mut v = world::world_view(&st);
    v["hud"] = engine::hud_json(&st);
    v
}

/// 改变游戏窗口大小（分辨率档位切换时调用）。参数为逻辑像素宽高。
#[tauri::command]
fn api_set_window_size(width: u32, height: u32, window: tauri::Window) -> Result<(), String> {
    use tauri::{LogicalSize, Size};
    window.set_size(Size::Logical(LogicalSize::new(width as f64, height as f64)))
        .map_err(|e| format!("set_size 失败: {e}"))?;
    let _ = window.center();
    Ok(())
}

/// 设置装备的技能格（最多 3 个主动技能）。校验：已学 + 主动 + ≤3，非法则拒绝并返回错误。
#[tauri::command]
fn api_set_equipped_skills(skills: Vec<String>, session: tauri::State<Session>) -> Result<Value, String> {
    rpc_log("api_set_equipped_skills");
    if skills.len() > 3 {
        return Err("最多装备 3 个技能。".into());
    }
    let mut st = session.0.lock().unwrap();
    // 去重（同一技能不能装多格）
    let mut seen = std::collections::HashSet::new();
    for sid in &skills {
        // 已学？
        if !st.skills.iter().any(|s| s == sid) {
            return Err(format!("技能「{sid}」尚未学习，无法装备。"));
        }
        // 必须为主动技能
        match wuxian_horror_ch1::skills_data::skill(sid) {
            Some(sk) if sk.kind == wuxian_horror_ch1::defs::SkillKind::Active => {}
            Some(_) => return Err(format!("技能「{sid}」为被动技能，无法装备。")),
            None => return Err(format!("技能「{sid}」不存在。")),
        }
        if !seen.insert(sid.clone()) {
            return Err(format!("技能「{sid}」重复装备。"));
        }
    }
    st.equipped_skills = skills;
    save_state(&st);
    Ok(engine::hud_json(&st))
}

/// 已学技能列表（含 id/name/category/熟练度/大类熟练度），供前端技能切换面板展示。
#[tauri::command]
fn api_get_skills(session: tauri::State<Session>) -> Value {    let st = session.0.lock().unwrap();
    let list: Vec<Value> = st.skills.iter().filter_map(|id| {
        wuxian_horror_ch1::skills_data::skill(id).map(|sk| {
            let cat = wuxian_horror_ch1::skills_data::skill_category(sk);
            json!({
                "id": sk.id,
                "name": sk.name,
                "school": sk.school.school_name(),
                "category": cat,
                "kind": match sk.kind { wuxian_horror_ch1::defs::SkillKind::Active => "Active", wuxian_horror_ch1::defs::SkillKind::Passive => "Passive" },
                "flying": sk.flying,
                "detect_bonus": sk.detect_bonus,
                "proficiency": st.skill_proficiency.get(sk.id).copied().unwrap_or(0),
                "proficiency_tier": wuxian_horror_ch1::defs::proficiency_tier(st.skill_proficiency.get(sk.id).copied().unwrap_or(0)),
                "category_proficiency": st.category_proficiency.get(cat).copied().unwrap_or(0),
            })
        })
    }).collect();
    json!({
        "skills": list,
        "equipped_skills": st.equipped_skills.clone(),
    })
}

// ============================================================================
// 血统进阶 API（血统阶位 + 熟练度强度系统）
// ============================================================================

/// 血统进阶：校验当前血统、有下一阶、点数够、剧情旗标满足；满足则扣点数、贡献砍半、tier+1。
/// 返回升到的阶位；失败返回中文原因。
#[tauri::command]
fn api_bloodline_advance(id: String, session: tauri::State<Session>) -> Result<Value, String> {
    rpc_log("api_bloodline_advance");
    let mut st = session.0.lock().unwrap();
    let new_tier = wuxian_horror_ch1::combat_data::try_advance_bloodline(&mut st, &id)
        .map_err(|e| e.to_string())?;
    save_state(&st);
    Ok(json!({
        "ok": true,
        "id": id,
        "tier": new_tier,
        "strength": wuxian_horror_ch1::combat_data::bloodline_strength(&st),
        "points": st.points,
        "hud": engine::hud_json(&st),
    }))
}

/// 血统进阶路线总览：19 血统 × 阶位数据 + 当前阶位 + 当前强度 + 进阶条件是否满足。
#[tauri::command]
fn api_get_bloodlines(session: tauri::State<Session>) -> Value {
    rpc_log("api_get_bloodlines");
    let st = session.0.lock().unwrap();
    let cur_id = st.bloodline.clone();
    let tiers: Vec<Value> = wuxian_horror_ch1::defs::BLOODLINES.iter().map(|b| {
        let bid = b.id;
        let cur_tier = if cur_id.as_deref() == Some(bid) {
            st.bloodline_tier.get(bid).copied().unwrap_or(1)
        } else {
            st.bloodline_tier.get(bid).copied().unwrap_or(1)
        };
        let contrib = st.bloodline_contribution.get(bid).copied().unwrap_or(0);
        let strength = wuxian_horror_ch1::combat_data::bloodline_tier_info(bid, cur_tier)
            .map(|c| c.base + contrib)
            .unwrap_or(0);
        // 阶位序列（1..=3，可用阶 + 进阶条件）
        let mut route = vec![];
        for t in 1..=3u8 {
            if let Some(cfg) = wuxian_horror_ch1::combat_data::bloodline_tier_info(bid, t) {
                let is_current = cur_tier == t;
                let can_advance = is_current
                    && cur_id.as_deref() == Some(bid)
                    && wuxian_horror_ch1::combat_data::has_next_tier(bid, t)
                    && (st.points as i64) >= cfg.need_points
                    && cfg.need_flag.map_or(true, |f| st.flag(f));
                route.push(json!({
                    "tier": cfg.tier,
                    "name": cfg.name,
                    "base": cfg.base,
                    "k": cfg.k,
                    "need_points": cfg.need_points,
                    "need_flag": cfg.need_flag,
                    "is_current": is_current,
                    "can_advance": can_advance,
                }));
            }
        }
        json!({
            "id": bid,
            "name": b.name,
            "desc": b.desc,
            "is_current": cur_id.as_deref() == Some(bid),
            "current_tier": cur_tier,
            "strength": strength,
            "proficiency": st.bloodline_proficiency.get(bid).copied().unwrap_or(0),
            "contribution": contrib,
            "route": route,
        })
    }).collect();
    json!({
        "bloodlines": tiers,
        "current": cur_id,
    })
}

// ============================================================================
// 体验账号（测试辅助）· 一键全解锁
// ============================================================================

/// 体验账号：一次性把所有血统/技能/装备/道具铺满 + 海量点数 + 真气修为全满。
/// 仅供测试使用，不改动任何战斗结算/剧情逻辑。
#[tauri::command]
fn api_setup_experience(session: tauri::State<Session>) -> Value {
    rpc_log("api_setup_experience");
    let mut st = session.0.lock().unwrap();

    use std::collections::BTreeSet;

    // 1. 海量点数
    st.points = 999999;

    // 2. 道具 + 装备全解锁（ITEMS + WEAPONS + GEAR + TRESURE_DEFS，去重）
    let mut inv: BTreeSet<String> = st.inventory.iter().cloned().collect();
    for d in wuxian_horror_ch1::defs::ITEMS.iter() { inv.insert(d.id.to_string()); }
    for d in wuxian_horror_ch1::defs::WEAPONS.iter() { inv.insert(d.id.to_string()); }
    for d in wuxian_horror_ch1::defs::GEAR.iter() { inv.insert(d.id.to_string()); }
    for d in wuxian_horror_ch1::defs::TRESURE_DEFS.iter() { inv.insert(d.id.to_string()); }
    let items = inv.len();
    st.inventory = inv.into_iter().collect();

    // 3. 技能全解锁（拥有标记=所有 SKILLS id）
    st.skills = wuxian_horror_ch1::skills_data::SKILLS.iter()
        .map(|sk| sk.id.to_string())
        .collect();
    let skills = st.skills.len();

    // 4. 装备前 3 个主动技能（最多 3 格）
    st.equipped_skills = wuxian_horror_ch1::skills_data::SKILLS.iter()
        .filter(|sk| sk.kind == wuxian_horror_ch1::defs::SkillKind::Active)
        .take(3)
        .map(|sk| sk.id.to_string())
        .collect();

    // 5. 血统全解锁：优先激活蜘蛛侠血统（新特性可直接体验），否则激活第一个；所有血统满阶(3) + 高熟练度/贡献
    let first = wuxian_horror_ch1::defs::BLOODLINES.first();
    let sp = wuxian_horror_ch1::defs::BLOODLINES.iter().find(|b| b.id == "spiderman_bloodline");
    if let Some(b) = sp.or(first) {
        st.bloodline = Some(b.id.to_string());
    }
    for b in wuxian_horror_ch1::defs::BLOODLINES.iter() {
        st.bloodline_tier.insert(b.id.to_string(), 3);
        st.bloodline_proficiency.insert(b.id.to_string(), 99999);
        st.bloodline_contribution.insert(b.id.to_string(), 999999);
    }

    // 6. 满真气修为：最高修真境界（7 合道）+ qi/QiMax 铺满
    st.cultivation_stage = 7;
    let cap = wuxian_horror_ch1::combat_data::cultivation_stage_cfg(7)
        .map(|c| c.qi_max_cap)
        .unwrap_or(1400);
    st.cultivation_qi_max = cap;
    st.qi_max = cap;
    st.qi = cap;

    // 存档：体验账号生效需触发保存，重启后仍保留
    save_state(&st);

    json!({
        "ok": true,
        "points": st.points,
        "skills": skills,
        "items": items,
        "msg": "体验账号已解锁（全道具·全技能·满修为）",
        "hud": engine::hud_json(&st),
    })
}

#[tauri::command]
fn formal_snapshot(runtime: tauri::State<'_, FormalRuntime>) -> Result<WorldView, String> {
    runtime.snapshot()
}

#[tauri::command]
fn formal_new(runtime: tauri::State<'_, FormalRuntime>) -> Result<WorldView, String> {
    runtime.reset_new()
}

#[tauri::command]
fn formal_submit_input(
    sample: wuxian_horror_ch1::continuous_input::InputSample,
    combat: Vec<CombatIntentRequest>,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<WorldView, String> {
    runtime.submit_input(sample, combat)
}

#[tauri::command]
fn formal_interact(
    actor_id: String,
    request_id: String,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<wuxian_horror_ch1::formal_runtime::FormalInteractionResponse, String> {
    runtime.interact(&actor_id, &request_id)
}

#[tauri::command]
fn formal_capability_command(
    command: CapabilityCommandRequest,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<WorldView, String> {
    runtime.apply_capability_command(command)
}

#[tauri::command]
fn formal_choose_first_enhancement(
    capability_id: String,
    runtime: tauri::State<'_, FormalRuntime>,
) -> Result<WorldView, String> {
    runtime.choose_first_enhancement(&capability_id)
}

#[tauri::command]
fn formal_has_save(runtime: tauri::State<'_, FormalRuntime>) -> bool {
    runtime.has_save()
}

#[tauri::command]
fn formal_save(runtime: tauri::State<'_, FormalRuntime>) -> Result<WorldView, String> {
    runtime.save()
}

#[tauri::command]
fn formal_continue(runtime: tauri::State<'_, FormalRuntime>) -> Result<WorldView, String> {
    runtime.continue_saved()
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            use tauri::Manager;
            let save_dir = std::env::var_os("WUXIAN_FORMAL_SAVE_DIR")
                .map(std::path::PathBuf::from)
                .map(Ok)
                .unwrap_or_else(|| app.path().app_data_dir())?;
            let runtime =
                FormalRuntime::new_with_save_dir(save_dir).map_err(std::io::Error::other)?;
            app.manage(runtime);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            formal_snapshot,
            formal_new,
            formal_submit_input,
            formal_interact,
            formal_capability_command,
            formal_choose_first_enhancement,
            formal_has_save,
            formal_save,
            formal_continue
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
