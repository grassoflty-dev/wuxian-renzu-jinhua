//! 开放世界核心：2D 地图移动/碰撞/楼层切换/敌人状态/调查交互/3D 副本会话
use crate::maps::EnemyDef;
use crate::state::GameState;
use crate::worlds;
use serde_json::{json, Value};


/// 轮回记忆开图半径（已探索区域可见半径，单位格）
const REVEAL_RADIUS: usize = 4;

/// 玩家出生时初始化世界状态（敌人全部存活、该世界出生点）
pub fn world_init(st: &mut GameState) {
    let world = worlds::find_world(&st.world_id).unwrap();
    let (x, y) = world.spawn();
    st.px = x;
    st.py = y;
    st.floor = 0;
    // 开放世界模式：默认配发消防斧（可探索/战斗）
    if st.weapon.is_none() {
        st.weapon = Some(crate::state::Weapon::Axe);
    }
    for e in world.enemies {
        st.enemies_alive.insert(e.id.to_string(), true);
    }
    // 地图对象默认 false（未获得/未解锁）
    for p in world.points {
        st.map_objs.entry(p.id.to_string()).or_insert(false);
    }
    for z in world.zones {
        st.map_objs.entry(z.id.to_string()).or_insert(false);
    }
    // 门禁默认锁定
    for g in world.gates {
        st.map_objs.entry(g.id.to_string()).or_insert(false);
    }
    // 出生点周围开图（轮回记忆未覆盖的新区域）
    reveal_around(st, REVEAL_RADIUS);
}

/// 标记玩家周围 radius 格为已探索（含墙格判定由渲染端做，这里仅存坐标集合）
fn reveal_around(st: &mut GameState, radius: usize) {
    let (px, py) = (st.px, st.py);
    let r = radius as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            let x = px as i32 + dx;
            let y = py as i32 + dy;
            if x < 0 || y < 0 { continue; }
            let (x, y) = (x as usize, y as usize);
            let world = worlds::find_world(&st.world_id).unwrap();
            // 行数 = 高度（h），首行字符长度 = 宽度（w）；越界则跳过
            let h = world.floors[st.floor].len();
            let w = if h > 0 { world.floors[st.floor][0].as_bytes().len() } else { 0 };
            if x >= w || y >= h { continue; }
            st.explored.insert(format!("{}:{}:{}:{}", st.world_id, st.floor, x, y));
        }
    }
}

/// 蛛丝拉拽（蜘蛛侠血统 · 世界地图机动）：点击目标墙格（高处建筑/墙面），
/// 校验射程(曼哈顿 ≤10)与视线(Bresenham 中间无墙)后，把玩家拉到目标旁最近开阔格并探索。
/// 返回 Ok(落点) 或 Err(原因)。血统校验由 main 层负责。
pub fn try_web_zip(st: &mut GameState, tx: i32, ty: i32) -> Result<(usize, usize), &'static str> {
    let world = worlds::find_world(&st.world_id).unwrap();
    let h = world.floors[st.floor].len();
    let w = if h > 0 { world.floors[st.floor][0].as_bytes().len() } else { 0 };
    if tx < 0 || ty < 0 || tx >= w as i32 || ty >= h as i32 { return Err("out_of_bounds"); }
    let target_c = world.floors[st.floor][ty as usize].as_bytes()[tx as usize] as char;
    if target_c != '#' { return Err("not_wall"); }
    let (px, py) = (st.px as i32, st.py as i32);
    if (tx - px).abs() + (ty - py).abs() > 10 { return Err("too_far"); }
    // Bresenham 视线：中间格不得为墙（目标本身为墙，允许）
    let (mut x0, mut y0) = (px, py);
    let (dx, dy) = ((tx - px).abs(), -(ty - py).abs());
    let (sx, sy) = (if px < tx { 1 } else { -1 }, if py < ty { 1 } else { -1 });
    let mut err = dx + dy;
    loop {
        if x0 == tx && y0 == ty { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
        if x0 == tx && y0 == ty { break; }
        if x0 < 0 || y0 < 0 || x0 >= w as i32 || y0 >= h as i32 { return Err("blocked"); }
        let c = world.floors[st.floor][y0 as usize].as_bytes()[x0 as usize] as char;
        if c == '#' { return Err("blocked"); }
    }
    // 落点：目标 8 邻域中最近的 '.'/'P'/'I' 开阔格
    let mut best: Option<(usize, usize)> = None;
    let mut best_d = i32::MAX;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 { continue; }
            let (nx, ny) = (tx + dx, ty + dy);
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 { continue; }
            let c = world.floors[st.floor][ny as usize].as_bytes()[nx as usize] as char;
            if c == '.' || c == 'P' || c == 'I' {
                let d = (nx - px).abs() + (ny - py).abs();
                if d < best_d { best_d = d; best = Some((nx as usize, ny as usize)); }
            }
        }
    }
    let Some((lx, ly)) = best else { return Err("no_landing"); };
    st.px = lx;
    st.py = ly;
    reveal_around(st, REVEAL_RADIUS);
    // 落点触发传送门 → 切层（同 try_move）
    for pt in world.portals {
        if pt.floor == st.floor && pt.x == lx && pt.y == ly {
            st.floor = pt.to_floor;
            st.px = pt.tx;
            st.py = pt.ty;
            reveal_around(st, REVEAL_RADIUS);
            break;
        }
    }
    Ok((st.px, st.py))
}

/// 尝试移动一格（当前楼层）；返回 (成功, 目标tile字符, 触发的对象id可选, 是否触发传送门)
/// 对象 id 编码：敌人=e:<id>、未解锁门禁=gate:<id>
pub fn try_move(st: &mut GameState, dx: i32, dy: i32) -> (bool, char, Option<String>, bool) {
    let world = worlds::find_world(&st.world_id).unwrap();
    let nx = st.px as i32 + dx;
    let ny = st.py as i32 + dy;
    if nx < 0 || ny < 0 {
        return (false, '#', None, false);
    }
    let (nx, ny) = (nx as usize, ny as usize);
    let c = worlds::tile(world, st.floor, nx, ny);
    if !worlds::walkable(world, st.floor, nx, ny) {
        return (false, c, None, false);
    }
    // 门禁：未解锁则挡路（返回 gate:<id> 供前端提示）
    if let Some(g) = world.gates.iter().find(|g| g.floor == st.floor && g.x == nx && g.y == ny) {
        let unlocked = st.map_objs.get(g.id).copied().unwrap_or(false);
        if !unlocked {
            return (false, 'G', Some(format!("gate:{}", g.id)), false);
        }
    }
    st.px = nx;
    st.py = ny;
    reveal_around(st, REVEAL_RADIUS);
    // 碰到敌人 → 返回敌人 id
    for e in world.enemies {
        if e.floor == st.floor && e.x == nx && e.y == ny
            && st.enemies_alive.get(e.id).copied().unwrap_or(false) {
            return (true, c, Some(e.id.to_string()), false);
        }
    }
    // 触发传送门 → 楼层切换（P0 不处理 to_world，PortalDef 无该字段）
    for pt in world.portals {
        if pt.floor == st.floor && pt.x == nx && pt.y == ny {
            st.floor = pt.to_floor;
            st.px = pt.tx;
            st.py = pt.ty;
            reveal_around(st, REVEAL_RADIUS);
            // 层内无敌人直接触发：返回传送信息
            return (true, c, None, true);
        }
    }
    (true, c, None, false)
}

/// 获取与玩家相邻（含同层）的可交互对象
pub fn nearby_interactables(st: &GameState) -> Vec<Value> {
    let world = worlds::find_world(&st.world_id).unwrap();
    let (x, y, fl) = (st.px, st.py, st.floor);
    let mut out = vec![];
    let mut push = |id: &str, name: &str, kind: &str, dx: i32, dy: i32| {
        out.push(json!({"id": id, "name": name, "kind": kind, "dx": dx, "dy": dy}));
    };
    for p in world.points {
        if p.floor != fl { continue; }
        if manhattan(x, y, p.x, p.y) <= 1 {
            push(p.id, p.name, "point", p.x as i32 - x as i32, p.y as i32 - y as i32);
        }
    }
    for n in world.npcs {
        if n.floor != fl { continue; }
        if manhattan(x, y, n.x, n.y) <= 1 {
            push(n.id, n.name, "npc", n.x as i32 - x as i32, n.y as i32 - y as i32);
        }
    }
    for z in world.zones {
        if z.floor != fl { continue; }
        if manhattan(x, y, z.x, z.y) <= 1 {
            push(z.id, z.name, "zone", z.x as i32 - x as i32, z.y as i32 - y as i32);
        }
    }
    for pt in world.portals {
        if pt.floor != fl { continue; }
        if manhattan(x, y, pt.x, pt.y) <= 1 {
            push(pt.id, "垂直通道", "portal", pt.x as i32 - x as i32, pt.y as i32 - y as i32);
        }
    }
    for g in world.gates {
        if g.floor != fl { continue; }
        if manhattan(x, y, g.x, g.y) <= 1 {
            push(g.id, g.name, "gate", g.x as i32 - x as i32, g.y as i32 - y as i32);
        }
    }
    // 跨世界网关（GW_PORTALS）：视为 portal 就近项，供 E/点击交互
    for gw in worlds::GW_PORTALS {
        if gw.from_world != st.world_id || gw.floor != fl { continue; }
        if manhattan(x, y, gw.x, gw.y) <= 1 {
            push(gw.id, gateway_label(gw), "portal", gw.x as i32 - x as i32, gw.y as i32 - y as i32);
        }
    }
    out
}

/// 网关显示名：可用返回目标世界名，占位标注咒怨封印
fn gateway_label(gw: &worlds::WorldGateway) -> &'static str {
    match gw.id {
        "gw_biohazard" => "传送门 · 生化蜂巢",
        "gw_zhouyuan" => "传送门 · 咒怨（封印）",
        _ => "传送门",
    }
}

fn manhattan(x1: usize, y1: usize, x2: usize, y2: usize) -> usize {
    (x1 as i32 - x2 as i32).unsigned_abs() as usize + (y1 as i32 - y2 as i32).unsigned_abs() as usize
}

/// 地图视图（前端 2D 渲染用，只含当前楼层数据）
pub fn world_view(st: &GameState) -> Value {
    let world = worlds::find_world(&st.world_id).unwrap();
    let fl = st.floor;
    let mut enemies = vec![];
    for e in world.enemies {
        if e.floor != fl { continue; }
        let alive = st.enemies_alive.get(e.id).copied().unwrap_or(false);
        enemies.push(json!({
            "id": e.id, "name": e.name, "x": e.x, "y": e.y,
            "radius": e.radius, "alive": alive,
        }));
    }
    let points: Vec<Value> = world.points.iter().filter(|p| p.floor == fl).map(|p| {
        json!({"id": p.id, "name": p.name, "x": p.x, "y": p.y, "done": st.map_objs.get(p.id).copied().unwrap_or(false)})
    }).collect();
    let npcs: Vec<Value> = world.npcs.iter().filter(|n| n.floor == fl).map(|n| {
        json!({"id": n.id, "name": n.name, "x": n.x, "y": n.y})
    }).collect();
    let zones: Vec<Value> = world.zones.iter().filter(|z| z.floor == fl).map(|z| {
        json!({"id": z.id, "name": z.name, "x": z.x, "y": z.y, "kind": z.kind, "ref": z.ref_id})
    }).collect();
    let mut portals: Vec<Value> = world.portals.iter().filter(|p| p.floor == fl).map(|p| {
        json!({"id": p.id, "x": p.x, "y": p.y, "to_floor": p.to_floor})
    }).collect();
    // 跨世界网关并入 portals 渲染（前端按传送门绘制）
    for gw in worlds::GW_PORTALS {
        if gw.from_world == st.world_id && gw.floor == fl {
            portals.push(json!({"id": gw.id, "x": gw.x, "y": gw.y, "to_floor": gw.to_floor, "to_world": gw.to_world}));
        }
    }
    let gates: Vec<Value> = world.gates.iter().filter(|g| g.floor == fl).map(|g| {
        json!({"id": g.id, "name": g.name, "x": g.x, "y": g.y,
               "locked": !st.map_objs.get(g.id).copied().unwrap_or(false),
               "need": g.need_item.map(|i| i.to_string()).or_else(|| g.need_flag.map(|f| f.to_string()))})
    }).collect();
    let cur_floor = world.floors.get(fl).copied().unwrap_or(&world.floors[world.floors.len() - 1]);
    let tiles = cur_floor;
    let h = cur_floor.len();
    let w = if h > 0 { cur_floor[0].len() } else { 0 };
    // 本轮层内已探索格子（"x:y" 列表，供前端迷雾渲染）—— 轮回记忆清单
    // 注意: 用 strip_prefix 只裁一次，trim_start_matches 会把 "1:1:5" 误裁成 "5"
    let prefix = format!("{}:{}:", st.world_id, fl);
    let explored: Vec<String> = st.explored.iter()
        .filter(|k| k.starts_with(&prefix))
        .map(|k| k.strip_prefix(&prefix).unwrap_or(k).to_string())
        .collect();
    json!({
        "w": w, "h": h, "floor": fl,
        "floor_name": world.floor_names[fl],
        "tiles": tiles,
        "px": st.px, "py": st.py,
        "enemies": enemies, "points": points, "npcs": npcs, "zones": zones, "portals": portals, "gates": gates,
        "inventory": st.inventory,
        "nearby": nearby_interactables(st),
        "explored": explored,
    })
}

/// 敌人信息查询（P0 仅生化已注册；签名兼容 main.rs api_world_move）
pub fn enemy_by_id(id: &str) -> Option<&'static EnemyDef> {
    worlds::find_world(worlds::WORLD_BIOHAZARD)
        .map(|w| w.enemies.iter().find(|e| e.id == id))
        .flatten()
}

/// 标记敌人死亡
pub fn kill_enemy(st: &mut GameState, id: &str) {
    if let Some(v) = st.enemies_alive.get_mut(id) {
        *v = false;
    }
}

/// 记录已完成的调查点
pub fn mark_point(st: &mut GameState, id: &str) {
    if let Some(v) = st.map_objs.get_mut(id) {
        *v = true;
    }
}

/// 道具加入物品栏（去重由调用方控制）
pub fn add_item(st: &mut GameState, item: &str) {
    if !st.inventory.iter().any(|i| i == item) {
        st.inventory.push(item.to_string());
    }
}

/// 初始化/补齐敌人存活表：逐个补缺，保证旧存档也能识别新增敌人
pub fn ensure_enemies(st: &mut GameState) {
    if let Some(world) = worlds::find_world(&st.world_id) {
        for e in world.enemies {
            st.enemies_alive.entry(e.id.to_string()).or_insert(true);
        }
    }
}

/// 切换世界：把当前顶层 map_objs/enemies_alive 快照进 world_states，再载入目标世界快照；惰性初始化。
/// 设计依据 §2.4/§5.2。st.floor/px/py 由调用方（传送门落点）覆写，本函数不改。
pub fn switch_world(st: &mut GameState, to: &str) {
    if st.world_id == to {
        return;
    }
    // 快照当前世界（若顶层的当前世界已被 snapshot 过则覆盖最新）
    st.world_states.insert(st.world_id.clone(), crate::state::WorldRuntime {
        map_objs: std::mem::take(&mut st.map_objs),
        enemies_alive: std::mem::take(&mut st.enemies_alive),
        entered: true,
    });
    // 载入目标世界快照（缺省空表 = 惰性初始化）
    if let Some(rt) = st.world_states.remove(to) {
        st.map_objs = rt.map_objs;
        st.enemies_alive = rt.enemies_alive;
    }
    st.world_id = to.to_string();
    ensure_enemies(st); // 按目标世界表补缺（惰性初始化主要入口）
}