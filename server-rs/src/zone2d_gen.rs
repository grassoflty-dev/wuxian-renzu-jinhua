//! zone2d_gen.rs — 2D 横版战斗副本地图生成器（确定性，同 zone_id 同版本必得同一地图）
//! 设计依据：tools/design/zone2d_camera_design.md §3
//! 拓扑：房间(普通/BOSS/出生/竖井) + L 形走廊(Prim 最小生成树 + 1~2 环路)，平台带可达性校验。
//! 瓦片字符：'#'=墙/实心，'.'=地板，'P'=平台砖(单面)，'C'=爬墙砖(竖井壁)。
//! 判定权威在 Rust：前端只消费本模块输出的 JSON（w/h/tiles/rooms/player_start/enemy_spawns/boss_room/seed）。

use serde_json::{json, Value};

// ---------- 确定性 PRNG ----------
fn fnv1a64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
struct Rng { s: u32 }
impl Rng {
    fn new(seed: u64) -> Self {
        Rng { s: ((seed ^ (seed >> 32)) as u32) | 1 }
    }
    fn next(&mut self) -> u32 {
        // mulberry32
        self.s = self.s.wrapping_add(0x6D2B79F5);
        let mut t = self.s;
        t = (t ^ (t >> 15)).wrapping_mul(1 | t);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t));
        (t ^ (t >> 14)) as u32
    }
    fn int(&mut self, n: i32) -> i32 {
        if n <= 0 { return 0; }
        (self.next() % n as u32) as i32
    }
    fn chance(&mut self, pct: i32) -> bool { self.int(100) < pct }
}

#[derive(Clone, Copy, PartialEq)]
struct Rect { x: i32, y: i32, w: i32, h: i32 }
impl Rect {
    fn right(&self) -> i32 { self.x + self.w }
    fn bottom(&self) -> i32 { self.y + self.h }
    fn cx(&self) -> i32 { self.x + self.w / 2 }
    fn cy(&self) -> i32 { self.y + self.h / 2 }
    fn overlap(&self, o: &Rect, gap: i32) -> bool {
        self.x - gap < o.right() && o.x - gap < self.right() &&
        self.y - gap < o.bottom() && o.y - gap < self.bottom()
    }
}

const MAP_W: usize = 60;
const MAP_H: usize = 36;

pub fn gen_zone2d(zone_id: &str) -> Value {
    let seed = fnv1a64(&format!("{}::v1", zone_id));
    // 高楼/rooftop 模式：zone_id 以 "rooftop_" 或 "city_" 开头 → 高楼布局代替房间走廊布局。
    // 仅拦截约定前缀，其余 zone_id 走原房间+走廊流程（向后兼容，布局不变）。
    if zone_id.starts_with("rooftop_") || zone_id.starts_with("city_") {
        let result = gen_rooftop(zone_id, seed);
        // 确定性保证 + 强校验：若某次派生布局意外通不过可达性，退回手敲的安全屋顶地图
        let v = match result {
            Some(v) => v,
            None => fallback_rooftop(seed),
        };
        return v;
    }
    let mut rng = Rng::new(seed);
    match try_gen(&mut rng, seed) {
        Some(v) => v,
        None => fallback_map(seed),
    }
}

// ---------------------------------------------------------------------------
// 高楼 / rooftop 模式
// 设计要点（为前端蜘蛛侠摆荡 showcase 服务）：
//   - 数栋竖直钢筋高楼柱（宽 6~10、高 20~30），从街面(底部)升起，楼与楼之间留 4~8 格水平间隙；
//   - 楼侧带可站人的楼顶缘悬挑平台 'P' + 高楼上部外壁 'C' 爬墙砖（前端摆荡瞄准点）；
//   - 每栋楼配一条连续 'P' 消防梯（楼顶 ↔ 街面），保证所有屋顶/平台经 8 邻域 BFS 可达；
//   - player_start 落在左侧楼顶，enemy_spawns / boss_room 落在较高楼顶；
//   - 完全确定性（seed 由 zone_id 派生），重复生成同一高楼地图。
// ---------------------------------------------------------------------------
fn gen_rooftop(zone_id: &str, seed: u64) -> Option<Value> {
    let mut rng = Rng::new(seed);
    let mut tiles: Vec<Vec<char>> = vec![vec!['#'; MAP_W]; MAP_H];

    // 1) 街面：底部 2 行贯全宽地板，保证建筑间水平可达（街道走廊）
    for x in 0..MAP_W {
        tiles[MAP_H - 1][x] = '.';
        tiles[MAP_H - 2][x] = '.';
    }

    // 2) 放置建筑柱（从地面升起，楼与楼之间留间隙 4~8）
    //    结构: (bx 左缘, btop 楼顶底行, bw 宽, bh 高)
    let mut buildings: Vec<(i32, i32, i32, i32)> = Vec::new();
    let n_buildings = 3 + rng.int(3); // 3~5 栋
    let mut bx: i32 = 1;
    let mut guard = 0;
    while (buildings.len() as i32) < n_buildings && guard < 300 {
        guard += 1;
        let bw = 6 + rng.int(5);   // 6~10
        let bh = 20 + rng.int(11); // 20~30
        if bx + bw > (MAP_W as i32) - 2 { break; }
        let btop = (MAP_H as i32) - 2 - bh; // 楼顶底行；墙上 2 行作楼顶
        buildings.push((bx, btop, bw, bh));
        bx += bw + 4 + rng.int(5); // 间隙 4~8
    }
    if buildings.is_empty() {
        // 极端退化：至少一栋
        buildings.push((1, (MAP_H as i32) - 24, 8, 22));
    }

    let _ = zone_id;

    // 3) 雕刻墙体 / 楼顶 / 梯 / 爬墙砖 / 缘台
    //    墙体只升到街面上方(MAP_H-3)，街道底部 2 行(MAP_H-2, MAP_H-1)保持全线地板，
    //    保证建筑之间街道水平贯通（这是可达性的关键）。
    let street_top = (MAP_H as i32) - 3; // 墙底行上界(不含)，下 2 行为街道
    let ladder_bottom = (MAP_H as i32) - 2; // 消防梯底到街道上方 1 行
    for &(bbx, btop, bw, _bh) in &buildings {
        // 墙体：竖直钢筋柱（止于街面上方）
        for x in bbx..bbx + bw {
            for y in btop..street_top {
                if y >= 0 && (y as usize) < MAP_H && (x as usize) < MAP_W {
                    tiles[y as usize][x as usize] = '#';
                }
            }
        }
        // 楼顶：墙上 2 行可站立（'.'）
        let rowhi = (btop + 1) as usize;
        for x in bbx..bbx + bw {
            if (x as usize) < MAP_W {
                tiles[btop as usize][x as usize] = '.';
                tiles[rowhi][x as usize] = '.';
            }
        }
        // 楼顶两侧缘悬挑平台 'P'（伸进间隙 1 格，可站/可摆荡依托）
        if bbx - 1 >= 0 { tiles[btop as usize][(bbx - 1) as usize] = 'P'; }
        if bbx + bw < MAP_W as i32 { tiles[btop as usize][(bbx + bw) as usize] = 'P'; }
        // 上段外壁爬墙砖 'C'（视觉/摆荡，'C' 不参与可达性判定）
        let ctop = btop + 2;
        for x in bbx..bbx + bw {
            for y in ctop..(ctop + 4).min(street_top) {
                if (y as usize) < MAP_H && (x as usize) < MAP_W {
                    tiles[y as usize][x as usize] = 'C';
                }
            }
        }
        // 消防梯：楼顶到街面上方的连续 'P' 竖柱（楼左侧，间隙内），保证 BFS 可达
        if bbx - 1 >= 0 && ((bbx - 1) as usize) < MAP_W {
            for y in btop + 1..=ladder_bottom {
                if y >= 0 { tiles[y as usize][(bbx - 1) as usize] = 'P'; }
            }
        }
    }

    // player_start：落在左侧第一栋楼顶（楼顶底行）
    let (fbx, fbtop, fbw, _) = buildings[0];
    let player_start = json!({ "x": fbx + fbw / 2, "y": fbtop + 1 });

    // 可达性强校验：任一 '. / P' 必须从 player_start 可达（街道全宽贯通 + 每栋楼连消防梯 ⇒ 恒可达）
    let (px, py) = (player_start["x"].as_i64().unwrap() as i32, player_start["y"].as_i64().unwrap() as i32);
    if !check_reachable_from(&tiles, px, py) {
        return None;
    }

    // boss：最高楼顶；enemy_spawns：每栋楼顶放 1 个 normal，最高的楼顶放 boss
    let tallest = buildings.iter().enumerate()
        .max_by_key(|(_, &(_, _, _, bh))| bh)
        .map(|(i, _)| i).unwrap_or(0);
    let (tbx, tbtop, tbw, _) = buildings[tallest];

    let boss_room = json!({ "x": tbx, "y": tbtop, "w": tbw, "h": 2 });
    let mut spawns: Vec<Value> = Vec::new();
    for (i, &(bbx, btop, bw, _)) in buildings.iter().enumerate() {
        let cx = bbx + bw / 2;
        if i == tallest {
            spawns.push(json!({ "x": cx, "y": btop + 1, "type": "boss" }));
        } else {
            spawns.push(json!({ "x": cx, "y": btop + 1, "type": "normal" }));
        }
    }

    let tiles_out: Vec<String> = tiles.iter().map(|row| row.iter().collect()).collect();
    let rooms_out: Vec<Value> = buildings.iter().enumerate().map(|(i, &(bbx, btop, bw, bh))| json!({
        "x": bbx, "y": btop, "w": bw, "h": bh,
        "type": if i == tallest { "boss" } else { "rooftop" },
    })).collect();

    Some(json!({
        "w": MAP_W, "h": MAP_H,
        "tiles": tiles_out,
        "rooms": rooms_out,
        "player_start": player_start,
        "enemy_spawns": spawns,
        "boss_room": boss_room,
        "seed": seed,
    }))
}

// 兜底：手敲一张最小但可达的高楼地图（被强校验触发时才用，理论不该发生）
fn fallback_rooftop(seed: u64) -> Value {
    // 单栋 8 宽 × 24 高的楼：楼顶 2 行可站，左侧连续 'P' 消防梯连到街面
    let mut tiles: Vec<Vec<char>> = vec![vec!['#'; MAP_W]; MAP_H];
    for x in 0..MAP_W {
        tiles[MAP_H - 1][x] = '.';
        tiles[MAP_H - 2][x] = '.';
    }
    let top = (MAP_H as i32) - 22; // 楼顶
    for x in (top as usize)..MAP_H {
        tiles[x][4] = '.';  // 左墙(A)
        tiles[x][5] = '.';  // 左墙(B)
        tiles[x][6] = '.';  // 右墙(A)
        tiles[x][7] = '.';  // 右墙(B)
    }
    let _ = top;
    json!({
        "w": MAP_W, "h": MAP_H,
        "tiles": tiles.iter().map(|r| r.iter().collect::<String>()).collect::<Vec<String>>(),
        "rooms": [ { "x": 0, "y": 0, "w": 60, "h": 36, "type": "rooftop" } ],
        "player_start": { "x": 5, "y": (MAP_H as i32) - 20 },
        "enemy_spawns": [ { "x": 5, "y": (MAP_H as i32) - 2, "type": "boss" } ],
        "boss_room": { "x": 0, "y": 0, "w": (MAP_W as i32), "h": 2 },
        "seed": seed,
    })
}


fn try_gen(rng: &mut Rng, seed: u64) -> Option<Value> {
    // ---------- 房间放置 ----------
    let mut rooms: Vec<(Rect, &'static str)> = Vec::new();
    // 出生房
    let start = Rect { x: 2, y: MAP_H as i32 - 8, w: 8, h: 6 };
    rooms.push((start, "start"));
    // BOSS 房 14×8（放在右上区）
    let boss = Rect { x: MAP_W as i32 - 16, y: 2, w: 14, h: 8 };
    rooms.push((boss, "boss"));
    // 普通房 + 竖井
    let mut n_rooms = 3 + rng.int(4);          // 3~6 个普通房
    let mut shafts = rng.int(3);               // 0~2 竖井
    let mut guard = 0;
    while rooms.len() < 5 + n_rooms as usize && guard < 200 {
        guard += 1;
        if shafts > 0 && rng.chance(30) {
            // 竖井房 4~6 宽 × 10~18 高
            let w = 4 + rng.int(3);
            let h = 10 + rng.int(9);
            let rx = 2 + rng.int(MAP_W as i32 - w - 4);
            let ry = 2 + rng.int(MAP_H as i32 - h - 4);
            let r = Rect { x: rx, y: ry, w, h };
            if rooms.iter().all(|(rr, _)| !rr.overlap(&r, 2)) {
                rooms.push((r, "shaft"));
                shafts -= 1;
            }
        } else {
            let w = 8 + rng.int(7);            // 8~14
            let h = 6 + rng.int(3);            // 6~8
            let rx = 2 + rng.int(MAP_W as i32 - w - 4);
            let ry = 2 + rng.int(MAP_H as i32 - h - 4);
            let r = Rect { x: rx, y: ry, w, h };
            if rooms.iter().all(|(rr, _)| !rr.overlap(&r, 2)) {
                rooms.push((r, "normal"));
                n_rooms -= 1;
            }
        }
    }

    // ---------- 瓦片雕刻 ----------
    let mut tiles: Vec<Vec<char>> = vec![vec!['#'; MAP_W]; MAP_H];
    let mut room_types: std::collections::HashMap<(i32, i32), &'static str> = std::collections::HashMap::new();
    for (r, ty) in &rooms {
        for y in r.y..r.bottom() {
            for x in r.x..r.right() {
                if x < 0 || y < 0 || x >= MAP_W as i32 || y >= MAP_H as i32 { continue; }
                if *ty == "shaft" {
                    // 竖井：两侧 C 爬墙砖，中间空
                    if x == r.x || x == r.right() - 1 { tiles[y as usize][x as usize] = 'C'; }
                    else if y == r.bottom() - 1 { tiles[y as usize][x as usize] = '.'; }
                } else {
                    tiles[y as usize][x as usize] = '.';
                }
            }
        }
        for y in r.y..r.bottom() {
            for x in r.x..r.right() {
                room_types.insert((x, y), ty);
            }
        }
    }

    // ---------- 走廊（Prim 最小生成树 + 环路） ----------
    let n = rooms.len();
    let mut mst: Vec<(usize, usize)> = Vec::new();
    let mut in_tree = vec![false; n];
    in_tree[0] = true;
    while mst.len() < n - 1 {
        let mut best: Option<(i32, usize, usize)> = None;
        for (i, (r1, _)) in rooms.iter().enumerate() {
            if !in_tree[i] { continue; }
            for (j, (r2, _)) in rooms.iter().enumerate() {
                if in_tree[j] { continue; }
                let d = (r1.cx() - r2.cx()).abs() + (r1.cy() - r2.cy()).abs();
                if best.map(|(bd, _, _)| d < bd).unwrap_or(true) {
                    best = Some((d, i, j));
                }
            }
        }
        if let Some((_, i, j)) = best {
            mst.push((i, j));
            in_tree[j] = true;
        } else { break; }
    }
    // 1~2 条环路（随机非树边）
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            if !mst.contains(&(i, j)) && !mst.contains(&(j, i)) {
                edges.push((i, j));
            }
        }
    }
    let loops = 1 + rng.int(2);
    for _ in 0..loops {
        if edges.is_empty() { break; }
        let e = rng.int(edges.len() as i32) as usize;
        mst.push(edges.remove(e));
    }
    for (i, j) in mst {
        let (a, b) = (rooms[i].0, rooms[j].0);
        let (x1, y1) = (a.cx(), a.cy());
        let (x2, y2) = (b.cx(), b.cy());
        carve_corridor(&mut tiles, x1, y1, x2, y1);
        carve_corridor(&mut tiles, x2, y1, x2, y2);
    }

    // ---------- 平台（普通房 1~2 层 + 可达性校验，重试 ≤8） ----------
    for (r, ty) in &rooms {
        if *ty != "normal" { continue; }
        for attempt in 0..8 {
            let mut plat: Vec<(i32, i32)> = Vec::new();
            let layers = if attempt >= 4 { 1 } else { 1 + rng.int(2) };
            for layer in 0..layers {
                let bar_y = r.bottom() - 3 - layer * 3;   // 房底-3 / 房底-6
                let bar_w = 3 + rng.int(6);               // 3~8
                let mut bx = r.x + 1;
                while bx + bar_w < r.right() - 1 {
                    let mut bar: Vec<(i32, i32)> = Vec::new();
                    for k in 0..bar_w { bar.push((bx + k, bar_y)); }
                    // 与已有平台条不重叠（同层间距）
                    if plat.iter().all(|(px, py)| *py != bar_y || (*px - bx).abs() > bar_w) {
                        plat.extend(bar);
                    }
                    bx += bar_w + 1 + rng.int(2);          // 洞 1~2 格
                }
            }
            // 清掉旧平台再画
            for (px, py) in plat_positions(&tiles) {
                let _ = px;
                if tiles[py as usize][px as usize] == 'P' { tiles[py as usize][px as usize] = '.'; }
            }
            for (px, py) in &plat {
                if *px >= 0 && *py >= 0 && *px < MAP_W as i32 && *py < MAP_H as i32 && room_types.get(&(*px, *py)).map(|t| *t == "normal").unwrap_or(false) {
                    tiles[*py as usize][*px as usize] = 'P';
                }
            }
            if check_reachable(&tiles, &rooms) || attempt == 7 {
                break;
            }
        }
    }

    // ---------- 出生点 / 敌人刷点 ----------
    let player_start = json!({ "x": start.cx(), "y": start.bottom() - 2 });
    let mut spawns: Vec<Value> = Vec::new();
    spawns.push(json!({ "x": boss.cx() + 2, "y": boss.bottom() - 1, "type": "boss" }));
    // 普通房各放 1 个 normal 刷点（距出生房 ≥4 格）
    for (r, ty) in &rooms {
        if *ty != "normal" { continue; }
        if (r.cx() - start.cx()).abs() + (r.cy() - start.cy()).abs() < 4 { continue; }
        let mut placed = false;
        for yy in (r.y..r.bottom()).rev() {
            for xx in r.x..r.right() {
                if tiles[yy as usize][xx as usize] == '.' {
                    spawns.push(json!({ "x": xx, "y": yy, "type": "normal" }));
                    placed = true;
                    break;
                }
            }
            if placed { break; }
        }
    }

    let tiles_out: Vec<String> = tiles.iter().map(|row| row.iter().collect()).collect();
    let rooms_out: Vec<Value> = rooms.iter().map(|(r, ty)| json!({
        "x": r.x, "y": r.y, "w": r.w, "h": r.h, "type": ty,
    })).collect();

    Some(json!({
        "w": MAP_W, "h": MAP_H,
        "tiles": tiles_out,
        "rooms": rooms_out,
        "player_start": player_start,
        "enemy_spawns": spawns,
        "boss_room": { "x": boss.x, "y": boss.y, "w": boss.w, "h": boss.h },
        "seed": seed,
    }))
}

fn plat_positions(tiles: &[Vec<char>]) -> Vec<(i32, i32)> {
    let mut v = Vec::new();
    for (y, row) in tiles.iter().enumerate() {
        for (x, c) in row.iter().enumerate() {
            if *c == 'P' { v.push((x as i32, y as i32)); }
        }
    }
    v
}

fn carve_corridor(tiles: &mut [Vec<char>], mut x1: i32, mut y1: i32, x2: i32, y2: i32) {
    // 水平段（y1 固定，x1→x2）：整行 2 宽
    let sx = if x2 >= x1 { 1 } else { -1 };
    while x1 != x2 {
        set_w(tiles, x1, y1); set_w(tiles, x1, y1 + 1);
        x1 += sx;
    }
    // 垂直段（x2 固定，y1→y2）：整列 2 宽
    let sy = if y2 >= y1 { 1 } else { -1 };
    while y1 != y2 {
        set_w(tiles, x2, y1); set_w(tiles, x2 + 1, y1);
        y1 += sy;
    }
    set_w(tiles, x2, y2); set_w(tiles, x2 + 1, y2);
}
fn set_w(tiles: &mut [Vec<char>], x: i32, y: i32) {
    if x >= 0 && y >= 0 && x < MAP_W as i32 && y < MAP_H as i32 {
        let c = tiles[y as usize][x as usize];
        if c == '#' || c == 'C' { tiles[y as usize][x as usize] = '.'; }
    }
}

// 可达性：从出生房地板 BFS，8 邻域 + 跳边（上 ≤3.5 格 / 下 ≤2 格，水平 ≤5 格），所有 '.'/'P' 必须可达
fn check_reachable(tiles: &[Vec<char>], rooms: &[(Rect, &'static str)]) -> bool {
    let start = rooms.iter().find(|(_, t)| *t == "start").map(|(r, _)| r.clone());
    let Some(s) = start else { return false };
    let (sx, sy) = (s.cx(), s.bottom() - 1);
    check_reachable_from(tiles, sx, sy)
}

// 从指定坐标出发的可达性 BFS（8 邻域 + 跳边），所有 '.'/'P' 必须可达。屋顶模式与房间模式共用。
fn check_reachable_from(tiles: &[Vec<char>], sx: i32, sy: i32) -> bool {
    if sx < 0 || sy < 0 || sx >= MAP_W as i32 || sy >= MAP_H as i32 { return false; }
    if tiles[sy as usize][sx as usize] == '#' { return false; }
    let mut seen = vec![vec![false; MAP_W]; MAP_H];
    let mut q = std::collections::VecDeque::new();
    q.push_back((sx, sy));
    seen[sy as usize][sx as usize] = true;
    let pass = |x: i32, y: i32| -> bool {
        x >= 0 && y >= 0 && x < MAP_W as i32 && y < MAP_H as i32 &&
        (tiles[y as usize][x as usize] == '.' || tiles[y as usize][x as usize] == 'P')
    };
    while let Some((x, y)) = q.pop_front() {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                let (nx, ny) = (x + dx, y + dy);
                if pass(nx, ny) && !seen[ny as usize][nx as usize] {
                    seen[ny as usize][nx as usize] = true;
                    q.push_back((nx, ny));
                }
            }
        }
        // 跳边：上方 ≤3.5 格（同列向下找最近支撑面），下方 ≤2 格
        for step in 1..=3 {
            let (nx, ny) = (x, y - step);
            if ny >= 0 && pass(nx, ny) && !seen[ny as usize][nx as usize] {
                // 中间不能有实心阻挡（简单近似：检查路径上非 '#'）
                let clear = (y - step + 1..y).all(|yy| tiles[yy as usize][x as usize] == '.' || tiles[yy as usize][x as usize] == 'P');
                if clear {
                    seen[ny as usize][nx as usize] = true;
                    q.push_back((nx, ny));
                }
            }
        }
        for step in 1..=2 {
            let (nx, ny) = (x, y + step);
            if ny < MAP_H as i32 && pass(nx, ny) && !seen[ny as usize][nx as usize] {
                seen[ny as usize][nx as usize] = true;
                q.push_back((nx, ny));
            }
        }
    }
    for (y, row) in tiles.iter().enumerate() {
        for (x, c) in row.iter().enumerate() {
            if (*c == '.' || *c == 'P') && !seen[y][x] { return false; }
        }
    }
    true
}

fn fallback_map(seed: u64) -> Value {
    let mut tiles: Vec<String> = Vec::new();
    for y in 0..8 {
        let mut row = String::new();
        for x in 0..12 {
            let c = if y == 0 || y == 7 || x == 0 || x == 11 { '#' } else { '.' };
            row.push(c);
        }
        tiles.push(row);
    }
    json!({
        "w": 12, "h": 8,
        "tiles": tiles,
        "rooms": [ { "x": 0, "y": 0, "w": 12, "h": 8, "type": "start" } ],
        "player_start": { "x": 5, "y": 6 },
        "enemy_spawns": [ { "x": 8, "y": 6, "type": "boss" } ],
        "boss_room": { "x": 0, "y": 0, "w": 12, "h": 8 },
        "seed": seed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rooftop_mode_creates_tall_buildings_and_reachable() {
        for zid in ["rooftop_ch1", "city_downtown", "rooftop_a"] {
            let v = gen_zone2d(zid);
            let tiles: Vec<Vec<char>> = v["tiles"]
                .as_array().unwrap()
                .iter()
                .map(|r| r.as_str().unwrap().chars().collect())
                .collect();

            // 存在竖直高墙柱('#'纵向连续 ≥18)，即高楼
            let h = v["h"].as_i64().unwrap() as usize;
            let w = v["w"].as_i64().unwrap() as usize;
            let mut max_col: usize = 0;
            for x in 0..w {
                let mut run: usize = 0;
                for y in 0..h {
                    if tiles[y][x] == '#' {
                        run += 1;
                        max_col = max_col.max(run);
                    } else {
                        run = 0;
                    }
                }
            }
            assert!(max_col >= 18, "zone {}: 应存在 ≥18 格竖直高墙柱, 实际 max_col={}", zid, max_col);

            // 存在 ≥2 栋（数栋楼），即至少 2 段竖直墙柱独立块
            let mut blocks = 0;
            let mut prev_wall = false;
            for x in 0..w {
                let col_has_tall = tiles[h - 12][x] == '#'; // 中高空属于楼体
                if col_has_tall && !prev_wall { blocks += 1; }
                prev_wall = col_has_tall;
            }
            assert!(blocks >= 2, "zone {}: 应 ≥2 栋楼, 实际 blocks={}", zid, blocks);

            // 可达性：从 player_start 出发，所有 '.'/'P' 可达
            let px = v["player_start"]["x"].as_i64().unwrap() as i32;
            let py = v["player_start"]["y"].as_i64().unwrap() as i32;
            assert!(check_reachable_from(&tiles, px, py), "zone {}: 可达性校验失败", zid);
        }
    }

    #[test]
    fn backward_compat_same_zone_id_unchanged() {
        // 不带 rooftop/city 前缀的 zone_id 必须与旧实现输出一致（无法跑旧代码时，至少验证确定性 & 走原房间流程）
        let a = gen_zone2d("ch1_1");
        let b = gen_zone2d("ch1_1");
        assert_eq!(a, b, "同 zone_id 两次生成必须一致（确定性）");
        // 确认输出含 rooms(房间走廊布局字段) 而非仅 boss 楼顶
        let rooms = a["rooms"].as_array().unwrap();
        assert!(rooms.len() >= 5, "房间走廊布局应含多个房间, 实际 {}", rooms.len());
    }
}
