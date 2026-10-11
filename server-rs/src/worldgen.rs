//! 程序化超大地图生成器（worldgen）：为任务世界生成「蜂巢设施」风格的大地图，
//! 替代手写小 tiles。输出 `Vec<String>`（每行字符串，'#'=墙，'.'=可走，'P'=出生点），
//! 与现有 `WorldData.floors: &[&[&str]]` 的 tile 格式完全兼容。
//!
//! 设计：随机房间 + 走廊（可按任务要求替换为 BSP 二叉树划分），保证连通。
//! - 确定性：以世界种子（slug hash → u64）做随机源，`StdRng` 种子固定，同一副本每次生成一致。
//! - 尺寸：默认 640x640（可参数化 width/height），生成几十到上百个更大的房间与更长的走廊。
//! - 性能：仅一次生成（上层 OnceLock 缓存），BFS/标注均为 O(width*height)，几十毫秒量级。
//! - 区域划分：按「出生点 BFS 距离」把可走格标为 入口区(Entrance)/探索区(Explore)/核心区(Core)/BOSS区(Boss)，
//!   供后续敌人/道具/传送门/剧情点填充使用；各区域提供锚点坐标。
//!
//! 元素填充（敌人/道具/传送门/剧情点）本轮不做，仅在 `GenResult` 中暴露区域锚点并标注 TODO。

use std::collections::VecDeque;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;

/// 区域标记（供元素填充按区域落位）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Region {
    /// 入口区：靠近出生点，适合指引/过渡、轻量互动点。
    Entrance,
    /// 探索区：地图主体，适合常规敌人/补给点。
    Explore,
    /// 核心区：中后段，适合精英/BOSS 前置机关/关键剧情点。
    Core,
    /// BOSS区：距出生点最远的角落房间，适合 BOSS 战/终局传送门。
    Boss,
}

/// 生成参数。
pub struct GenConfig {
    pub width: usize,
    pub height: usize,
    /// 期望房间数量（实际依布局可能略少）。
    pub room_count: usize,
    /// 随机数种子（同一 seed → 同一地图）。
    pub seed: u64,
}

impl GenConfig {
    /// 默认超大地图：640x640，几十到上百个更大的房间。
    pub fn hub640(seed: u64) -> Self {
        Self { width: 640, height: 640, room_count: 96, seed }
    }

    /// 供小尺寸/单测使用（原 64x64 默认，保留可参数化入口）。
    pub fn hub64(seed: u64) -> Self {
        Self { width: 64, height: 64, room_count: 22, seed }
    }
}

/// 一次生成的完整结果。
pub struct GenResult {
    /// 地图行（每行定长 width），与现有 tiles 格式兼容。
    pub tiles: Vec<String>,
    pub width: usize,
    pub height: usize,
    /// 与 tiles 同尺寸：每个格子的区域标记（墙格统一置为 Explore 占位，仅 '.' 格有意义）。
    pub regions: Vec<Vec<Region>>,
    /// 出生点（已标 'P'）。
    pub spawn: (usize, usize),
    /// BOSS 锚点（最远房间中心格）。
    pub boss_room: (usize, usize),
    /// 全部房间中心格。
    pub room_centers: Vec<(usize, usize)>,
    /// 入口区锚点（即出生点）。
    pub entrance_anchor: (usize, usize),
    /// 核心区锚点（若干）。
    pub core_anchors: Vec<(usize, usize)>,
    /// BOSS 区锚点（即为 boss_room）。
    pub boss_anchor: (usize, usize),
}

/// 由世界 slug 派生确定性种子（稳定字符串 hash）。
pub fn seed_from_slug(slug: &str) -> u64 {
    let mut h: u64 = 14695981039346656037;
    for &b in slug.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

/// 把某个格强制设为可走（'#'→'.'）。用于固定传送落点/对象落位格。
pub fn carve_walkable(tiles: &mut [String], x: usize, y: usize) {
    if let Some(row) = tiles.get_mut(y) {
        if x < row.len() && row.as_bytes()[x] == b'#' {
            let bytes = unsafe { row.as_bytes_mut() };
            bytes[x] = b'.';
        }
    }
}

/// 把某个格设为出生点 'P'（若当前墙则先凿开）。
pub fn carve_spawn(tiles: &mut [String], x: usize, y: usize) {
    let Some(row) = tiles.get_mut(y) else { return };
    if x >= row.len() { return; }
    if byte_at(row, x) == b'#' {
        let bytes = unsafe { row.as_bytes_mut() };
        bytes[x] = b'.';
    }
    // 置 'P'
    let bytes = unsafe { row.as_bytes_mut() };
    bytes[x] = b'P';
}

/// 把一张手写小地图「盖章」进大图指定偏移处（覆盖式）。用于把内容区（对象坐标权威即基于该
/// 小图设计，内部本就连通且含出生点 'P'）放回程序化大图：对象/门禁/传送门将落在连通地板上，
/// 而非散落成孤立的单格。小图通常 40×26，覆盖大图左上角（ox,oy）。
pub fn stamp_map(tiles: &mut [String], small: &[&str], ox: usize, oy: usize) {
    for (sy, row) in small.iter().enumerate() {
        let ty = oy + sy;
        let Some(dst) = tiles.get_mut(ty) else { break };
        let bytes = unsafe { dst.as_bytes_mut() };
        for (sx, &b) in row.as_bytes().iter().enumerate() {
            let tx = ox + sx;
            if tx >= bytes.len() { break; }
            bytes[tx] = b;
        }
    }
}

/// 保证从 `start` 出发、地图上所有开放格（非 '#'）连通：反复找到最近的不可达开放格，
/// 并从距其最近的已连通格凿一条 L 形走廊连回，直到无孤立开放格（或达安全迭代上限）。
/// 用于把「盖章内容区」与程序化大图主区桥接，修复对象/出生点被墙隔断的问题。
pub fn connect_all(tiles: &mut [String], start: (usize, usize)) {
    let h = tiles.len();
    if h == 0 { return; }
    let w = tiles[0].len();
    let mut guard = 0usize;
    loop {
        let dist = bfs_dist(tiles, start);
        // 任意一个不可达的开放格
        let mut target: Option<(usize, usize)> = None;
        'outer: for y in 0..h {
            for x in 0..w {
                if char_at(&tiles[y], x) != '#' && dist[y][x] == usize::MAX {
                    target = Some((x, y));
                    break 'outer;
                }
            }
        }
        let Some((tx, ty)) = target else { break };
        // 距该格最近的已连通格（曼哈顿）
        let mut src: Option<(usize, usize)> = None;
        let mut bd = usize::MAX;
        for y in 0..h {
            for x in 0..w {
                if dist[y][x] < usize::MAX {
                    let d = x.abs_diff(tx) + y.abs_diff(ty);
                    if d < bd { bd = d; src = Some((x, y)); }
                }
            }
        }
        let Some((sx, sy)) = src else { break };
        carve_corridor(tiles, sx, sy, tx, ty);
        guard += 1;
        if guard > 128 { break; }
    }
}

fn byte_at(s: &str, x: usize) -> u8 {
    s.as_bytes().get(x).copied().unwrap_or(b'#')
}

fn char_at(s: &str, x: usize) -> char {
    byte_at(s, x) as char
}

/// 由地图边长推导房间尺寸范围：随地图放大而增大，且始终保证 min<=max。
/// `m_div`/`max_div`/`hard_div` 为缩放除数：分别控制下限/上限/硬上限。
fn scaled_room_bounds(len: usize, min_div: usize, max_div: usize, hard_div: usize) -> (usize, usize) {
    let floor = 4usize;
    // 目标下限/上限（随尺寸增大，但钳制到合理上限，避免房间过大）
    let min = (len / min_div).max(floor);
    let max = (len / max_div).max(floor + 1);
    let hard_max = (len / hard_div).max(min);
    let mut hi = max.min(hard_max);
    if hi < min {
        hi = min.min(len / 2).max(floor);
    }
    (min.min(hi), hi)
}

/// 生成一层地图。
pub fn generate_floor(cfg: &GenConfig) -> GenResult {
    let w = cfg.width.max(8);
    let h = cfg.height.max(8);
    let mut rng = StdRng::seed_from_u64(cfg.seed);

    // 全墙初始化
    let mut tiles: Vec<String> = (0..h).map(|_| "#".repeat(w)).collect();
    // 强留 1 格墙边界，避免出生点/传送落点贴边越界
    let (max_x, max_y) = (w - 1, h - 1);

    // ---- 生成若干矩形房间（蜂巢大厅/实验室/通道）----
    let room_half = cfg.room_count.max(3);
    let mut room_centers: Vec<(usize, usize)> = Vec::new();
    let mut rooms: Vec<(usize, usize, usize, usize)> = Vec::new(); // (x0,y0,x1,y1)
    let margin = 1;
    // 房间尺寸随地图规模缩放：大图生成「几十到上百格」的大房间，小图退化为 4~11 格。
    let (rw_min, rw_max) = scaled_room_bounds(w, 24, 12, 6);
    let (rh_min, rh_max) = scaled_room_bounds(h, 32, 16, 8);
    let mut attempts = 0;
    let max_attempts = room_half * 60;
    while rooms.len() < room_half && attempts < max_attempts {
        attempts += 1;
        let rw = rng.gen_range(rw_min..=rw_max);
        let rh = rng.gen_range(rh_min..=rh_max);
        if rw + 2 > w || rh + 2 > h {
            continue;
        }
        let rx0 = rng.gen_range(margin..=max_x.saturating_sub(rw));
        let ry0 = rng.gen_range(margin..=max_y.saturating_sub(rh));
        let (rx1, ry1) = (rx0 + rw - 1, ry0 + rh - 1);
        // 允许少量重叠；直接凿空该矩形（墙体行/列保留 1 格内缩，保证房间完整）
        for y in ry0..=ry1 {
            for x in rx0..=rx1 {
                let row_bytes = unsafe { tiles[y].as_bytes_mut() };
                if row_bytes[x] != b'P' {
                    row_bytes[x] = b'.';
                }
            }
        }
        let cx = (rx0 + rx1) / 2;
        let cy = (ry0 + ry1) / 2;
        rooms.push((rx0, ry0, rx1, ry1));
        room_centers.push((cx, cy));
    }

    // ---- 走廊连接：顺序连接保证全图连通（每房间通向之前某随机房间，形成树 + 少量环）----
    if rooms.len() > 1 {
        // 建立连接列表：每条边 (来自, 去往)。首房间固定为根，其余每房连向一个随机已存在房间。
        let mut edges: Vec<(usize, usize)> = Vec::new();
        let mut tree: Vec<usize> = vec![0];
        for i in 1..rooms.len() {
            let parent = tree.choose(&mut rng).copied().unwrap_or(0);
            edges.push((parent, i));
            tree.push(i);
        }
        // 额外走廊（增加环与连通冗余）
        let extra = (rooms.len() / 3).max(1);
        for _ in 0..extra {
            let a = rng.gen_range(0..rooms.len());
            let b = rng.gen_range(0..rooms.len());
            if a != b {
                edges.push((a, b));
            }
        }
        for (a, b) in edges {
            let (ax, ay) = room_centers[a];
            let (bx, by) = room_centers[b];
            carve_corridor(&mut tiles, ax, ay, bx, by);
        }
    }

    // 出生点：取首房间中心（尽可能靠左下=入口侧），force 'P'
    let spawn = if let Some(&(sx, sy)) = room_centers.first() {
        (sx.min(max_x), sy.min(max_y))
    } else {
        (1, 1)
    };
    carve_spawn(&mut tiles, spawn.0, spawn.1);

    // ---- BFS 距离场（自出生点，沿可走格）----
    let dist = bfs_dist(&tiles, spawn);

    // BOSS 房：找距出生点最远的房间中心（其床在 dist 可达范围内）
    let reachable_centers: Vec<(usize, usize, usize)> = room_centers
        .iter()
        .flat_map(|&(cx, cy)| {
            let d = dist.get(cy).and_then(|r| r.get(cx)).copied();
            if let Some(dv) = d {
                if dv < usize::MAX {
                    return Some((cx, cy, dv));
                }
            }
            None
        })
        .collect();
    let farthest = reachable_centers.iter().max_by_key(|&&(_, _, d)| d);
    let boss_room = match farthest {
        Some(&(cx, cy, _)) => (cx, cy),
        None => *room_centers.last().unwrap_or(&spawn),
    };

    // ---- 区域标注 ----
    let maxd = dist
        .iter()
        .flatten()
        .filter(|&&d| d < usize::MAX)
        .copied()
        .max()
        .unwrap_or(1)
        .max(1);
    let mut regions: Vec<Vec<Region>> = Vec::with_capacity(h);
    for y in 0..h {
        let mut row = Vec::with_capacity(w);
        for x in 0..w {
            let d = dist[y][x];
            let r = if d >= usize::MAX {
                Region::Explore // 墙/不可达占位
            } else {
                let f = d as f32 / maxd as f32;
                if f < 0.30 {
                    Region::Entrance
                } else if f < 0.55 {
                    Region::Core
                } else if f >= 0.85 {
                    Region::Boss
                } else {
                    Region::Explore
                }
            };
            row.push(r);
        }
        regions.push(row);
    }
    // 强制 BOSS 房中心标为 Boss（保证 BOSS 锚点成立）
    if let Some(row) = regions.get_mut(boss_room.1) {
        if let Some(c) = row.get_mut(boss_room.0) {
            *c = Region::Boss;
        }
    }
    // 强制出生点区域为 Entrance
    if let Some(row) = regions.get_mut(spawn.1) {
        if let Some(c) = row.get_mut(spawn.0) {
            *c = Region::Entrance;
        }
    }
    // 收集核心区锚点
    let core_anchors = collect_region_anchors(&regions, Region::Core, 3);
    let entrance_anchor = spawn;
    let boss_anchor = boss_room;

    GenResult {
        tiles,
        width: w,
        height: h,
        regions,
        spawn,
        boss_room,
        room_centers,
        entrance_anchor,
        core_anchors,
        boss_anchor,
    }
}

/// 在 (ax,ay) 与 (bx,by) 之间凿一条 L 形（或矩形）走廊，保证两格连通。
fn carve_corridor(tiles: &mut [String], ax: usize, ay: usize, bx: usize, by: usize) {
    // 先横向再纵向
    let x0 = ax.min(bx);
    let x1 = ax.max(bx);
    for x in x0..=x1.min(tiles[0].len() - 1) {
        set_floor(tiles, x, ay);
    }
    let y0 = ay.min(by);
    let y1 = ay.max(by);
    for y in y0..=y1.min(tiles.len() - 1) {
        set_floor(tiles, bx, y);
    }
}

fn set_floor(tiles: &mut [String], x: usize, y: usize) {
    if let Some(row) = tiles.get_mut(y) {
        if x < row.len() {
            if char_at(row, x) != b'P' as char {
                let bytes = unsafe { row.as_bytes_mut() };
                bytes[x] = b'.';
            }
        }
    }
}

fn bfs_dist(tiles: &[String], start: (usize, usize)) -> Vec<Vec<usize>> {
    let h = tiles.len();
    let w = tiles.first().map(|r| r.len()).unwrap_or(0);
    let mut dist = vec![vec![usize::MAX; w]; h];
    let mut q = VecDeque::new();
    if h == 0 || w == 0 {
        return dist;
    }
    dist[start.1][start.0] = 0;
    q.push_back(start);
    let dirs = [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)];
    while let Some((cx, cy)) = q.pop_front() {
        let d = dist[cy][cx];
        for (dx, dy) in dirs.iter() {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            let (nx, ny) = (nx as usize, ny as usize);
            if nx >= w || ny >= h {
                continue;
            }
            if char_at(&tiles[ny], nx) == '#' {
                continue;
            }
            if dist[ny][nx] <= d + 1 {
                continue;
            }
            dist[ny][nx] = d + 1;
            q.push_back((nx, ny));
        }
    }
    dist
}

/// 收集某一区域若干个代表性锚点（按出现顺序前 n 个）。
fn collect_region_anchors(
    regions: &[Vec<Region>],
    which: Region,
    n: usize,
) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for (y, row) in regions.iter().enumerate() {
        for (x, &r) in row.iter().enumerate() {
            if r == which {
                out.push((x, y));
                if out.len() >= n {
                    return out;
                }
            }
        }
    }
    out
}

/// 判定全图可走格是否连通（自出生点）。供测试/自检。
pub fn is_connected(result: &GenResult) -> bool {
    let dist = bfs_dist(&result.tiles, result.spawn);
    for y in 0..result.height {
        for x in 0..result.width {
            if char_at(&result.tiles[y], x) != '#' && dist[y][x] == usize::MAX {
                return false;
            }
        }
    }
    true
}

pub fn walkable_char(c: char) -> bool {
    c != '#'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_same_seed_same_map() {
        let cfg = GenConfig::hub640(seed_from_slug("baisun"));
        let a = generate_floor(&cfg);
        let b = generate_floor(&cfg);
        assert_eq!(a.width, 640);
        assert_eq!(a.height, 640);
        assert_eq!(a.tiles, b.tiles, "同种子应生成完全相同地图");
        assert_eq!(a.regions, b.regions);
    }

    #[test]
    fn different_seed_different_map() {
        let a = generate_floor(&GenConfig::hub640(seed_from_slug("world_x")));
        let b = generate_floor(&GenConfig::hub640(seed_from_slug("world_y")));
        assert_ne!(a.tiles, b.tiles, "不同种子应生成不同地图");
    }

    #[test]
    fn size_is_configurable() {
        let cfg = GenConfig { width: 64, height: 64, room_count: 22, seed: 1 };
        let r = generate_floor(&cfg);
        assert_eq!(r.width, 64);
        assert_eq!(r.height, 64);
        assert_eq!(r.tiles.len(), 64);
        for row in &r.tiles {
            assert_eq!(row.len(), 64, "每行必须等宽");
        }
    }

    #[test]
    fn scale_up_is_much_larger() {
        let cfg = GenConfig { width: 640, height: 640, room_count: 96, seed: 7 };
        let r = generate_floor(&cfg);
        assert_eq!(r.width, 640);
        assert_eq!(r.height, 640);
        assert_eq!(r.tiles.len(), 640);
        for row in &r.tiles {
            assert_eq!(row.len(), 640, "每行必须等宽");
        }
        assert!(r.room_centers.len() > 40, "640x640 应生成大量房间");
    }

    #[test]
    fn spawn_present_and_walkable() {
        let r = generate_floor(&GenConfig::hub640(seed_from_slug("baisun")));
        assert_eq!(
            char_at(&r.tiles[r.spawn.1], r.spawn.0),
            'P',
            "出生点应为 'P'"
        );
    }

    #[test]
    fn floor_is_connected() {
        let r = generate_floor(&GenConfig::hub640(seed_from_slug("baisun")));
        assert!(is_connected(&r), "地图所有可走格应连通");
    }

    #[test]
    fn stamp_and_connect_makes_objects_reachable() {
        // 模拟 biohazard/baisun 的修复：大图对象本会散落为孤立格，盖章小图内容区 + connect_all
        // 后，出生点 'P' 与所有对象格都必须与全图连通。
        let mut res = generate_floor(&GenConfig::hub640(seed_from_slug("biohazard_ch1")));
        // 撤销 worldgen 的随机 'P'
        if res.tiles[res.spawn.1].as_bytes()[res.spawn.0] == b'P' {
            let bytes = unsafe { res.tiles[res.spawn.1].as_bytes_mut() };
            bytes[res.spawn.0] = b'.';
        }
        let small: &[&str] = &[
            "##########",
            "#P.......#",
            "#........#",
            "#..#.....#",
            "#........#",
            "##########",
        ];
        stamp_map(&mut res.tiles, small, 0, 0);
        // 两个对象格（防御性凿开，模拟对象坐标权威）
        carve_walkable(&mut res.tiles, 2, 2);
        carve_walkable(&mut res.tiles, 7, 3);
        let start = (1, 1);
        connect_all(&mut res.tiles, start);

        let dist = bfs_dist(&res.tiles, start);
        for y in 0..res.height {
            for x in 0..res.width {
                if char_at(&res.tiles[y], x) != '#' {
                    assert_ne!(dist[y][x], usize::MAX, "开放格 ({x},{y}) 应可达");
                }
            }
        }
        assert_ne!(dist[2][2], usize::MAX, "对象格 (2,2) 应可达");
        assert_ne!(dist[3][7], usize::MAX, "对象格 (7,3) 应可达");
    }

    #[test]
    fn regions_are_marked() {
        let r = generate_floor(&GenConfig::hub640(seed_from_slug("baisun")));
        // 出生点区域 = Entrance，BOSS 锚点区域 = Boss
        assert_eq!(r.regions[r.spawn.1][r.spawn.0], Region::Entrance);
        assert_eq!(r.regions[r.boss_room.1][r.boss_room.0], Region::Boss);
        // 各区域锚点非空
        assert!(!r.core_anchors.is_empty());
        assert_ne!(r.boss_room, r.spawn, "BOSS 房应远离出生点");
    }
}
