# -*- coding: utf-8 -*-
"""ds0731_check.py — DS0731 渲染截图的量化自查(无视觉, 纯像素统计) v2。
用法: python ds0731_check.py <view> <png>
指标:
  1. 轮廓平滑度: 单连通 mask 外轮廓的尖角数/周长比; top/bottom 尖角少 = 圆润连续。
  2. 内部空洞: 背景像素被模型包围(洪泛外标记), 只统计 >=8px 的空洞(中空/纸片露馅)。
  3. 百叶窗/碎片: 模型区内每行分段数 >3 的行数(斜视条纹)。
  4. 明暗突变: 模型区内亮度梯度超阈值的像素占比(顶部/底部应低)。
同时输出 hole mask 可视化 PNG 到 shots/diag_ds0731/ 供识图子代理复核。
"""
import sys, os
from PIL import Image
import numpy as np

def clean_mask(model):
    """阈值 + 闭运算(去反锯齿斑点) + 最大连通域。"""
    h, w = model.shape
    # 闭运算: 膨胀2 + 腐蚀2 (4 邻域)
    def dilate(m, r):
        for _ in range(r):
            e = m.copy()
            e[1:, :] |= m[:-1, :]; e[:-1, :] |= m[1:, :]
            e[:, 1:] |= m[:, :-1]; e[:, :-1] |= m[:, 1:]
            m = e
        return m
    def erode(m, r):
        for _ in range(r):
            e = m.copy()
            e[1:, :] &= m[:-1, :]; e[:-1, :] &= m[1:, :]
            e[:, 1:] &= m[:, :-1]; e[:, :-1] &= m[:, 1:]
            m = e
        return m
    m = dilate(model.copy(), 2)
    m = erode(m, 2)
    # 最大连通域(4 邻域)
    visited = np.zeros((h, w), dtype=bool)
    best = None; best_size = 0
    ys, xs = np.where(m)
    for y, x in zip(ys, xs):
        if visited[y, x]: continue
        comp = []
        st = [(y, x)]; visited[y, x] = True
        while st:
            yy, xx = st.pop(); comp.append((yy, xx))
            for dy, dx in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
                ny, nx = yy + dy, xx + dx
                if 0 <= ny < h and 0 <= nx < w and m[ny, nx] and not visited[ny, nx]:
                    visited[ny, nx] = True; st.append((ny, nx))
        if len(comp) > best_size:
            best_size = len(comp); best = comp
    out = np.zeros((h, w), dtype=bool)
    if best:
        for (y, x) in best: out[y, x] = True
    return out

def trace_boundary(mask):
    """Moore 追踪外轮廓, 回到起点终止。返回轮廓点列表。"""
    h, w = mask.shape
    ys, xs = np.where(mask)
    if len(ys) == 0: return []
    # 起点: 最上行最左的边界像素
    top_y = ys.min()
    row_ys = ys[ys == top_y]
    sx = int(xs[np.where(ys == top_y)[0]].min())
    sy = int(top_y)
    # 找到起点处的一个出方向(顺时针)
    dirs = [(1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1), (0, 1), (1, 1)]  # 顺时针
    contour = []
    cx, cy = sx, sy
    # 起始方向: 从 12 点方向开始顺时针找第一个在 mask 内的 8 邻域
    start_dir = 0
    for di in range(8):
        dx, dy = dirs[di]
        nx, ny = cx + dx, cy + dy
        if 0 <= nx < w and 0 <= ny < h and mask[ny, nx]:
            start_dir = (di + 6) % 8  # 反向作为前进起点
            break
    prev_dir = start_dir
    while True:
        contour.append((cx, cy))
        if len(contour) > 200000:
            break
        # 从 prev_dir 顺时针扫描(先右转)
        found = False
        for di in range(8):
            d = (prev_dir + di) % 8
            dx, dy = dirs[d]
            nx, ny = cx + dx, cy + dy
            if 0 <= nx < w and 0 <= ny < h and mask[ny, nx]:
                cx, cy = nx, ny
                prev_dir = (d + 4) % 8  # 回退方向
                found = True
                break
        if not found:
            break
        if cx == sx and cy == sy:
            contour.append((cx, cy))
            break
    return contour

def analyze(path, view, diag_dir):
    im = Image.open(path).convert('RGB')
    a = np.asarray(im).astype(int)
    H, W = a.shape[:2]
    mx = a.max(axis=2)
    model_raw = mx < 246
    model = clean_mask(model_raw)
    if model.sum() == 0:
        print('NO MODEL'); return
    # ---- 1. 空洞 ----
    bg = ~model
    ext = np.zeros((H, W), dtype=bool)
    q = []
    for x in range(W):
        if bg[0, x] and not ext[0, x]: ext[0, x] = True; q.append((0, x))
        if bg[H - 1, x] and not ext[H - 1, x]: ext[H - 1, x] = True; q.append((H - 1, x))
    for y in range(H):
        if bg[y, 0] and not ext[y, 0]: ext[y, 0] = True; q.append((y, 0))
        if bg[y, W - 1] and not ext[y, W - 1]: ext[y, W - 1] = True; q.append((y, W - 1))
    while q:
        y, x = q.pop()
        for dy, dx in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
            ny, nx = y + dy, x + dx
            if 0 <= ny < H and 0 <= nx < W and bg[ny, nx] and not ext[ny, nx]:
                ext[ny, nx] = True; q.append((ny, nx))
    holes = bg & ~ext
    # 只统计 >=8px 的空洞
    visited = np.zeros((H, W), dtype=bool)
    comps = []
    ys, xs = np.where(holes)
    for y, x in zip(ys, xs):
        if visited[y, x]: continue
        st = [(y, x)]; visited[y, x] = True; sz = 0; pts = []
        while st:
            yy, xx = st.pop(); sz += 1; pts.append((yy, xx))
            for dy, dx in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
                ny, nx = yy + dy, xx + dx
                if 0 <= ny < H and 0 <= nx < W and holes[ny, nx] and not visited[ny, nx]:
                    visited[ny, nx] = True; st.append((ny, nx))
        if sz >= 8: comps.append((sz, pts))
    big_comps = [c for c in comps if c[0] >= 60]
    hole_px = sum(c[0] for c in comps)
    # 大空洞位置(bbox 与归一化坐标)
    hole_locs = []
    for (sz, pts) in big_comps:
        hy = [p[0] for p in pts]; hx = [p[1] for p in pts]
        hole_locs.append((sz, (min(hx), min(hy), max(hx), max(hy)),
                          (round(min(hx) / W, 2), round(min(hy) / H, 2), round(max(hx) / W, 2), round(max(hy) / H, 2))))
    # 保存空洞可视化
    os.makedirs(diag_dir, exist_ok=True)
    hm = Image.new('RGB', (W, H), (255, 255, 255))
    hpx = hm.load()
    for (sz, pts) in comps:
        col = (255, 0, 0) if sz >= 60 else (255, 170, 0)
        for (y, x) in pts: hpx[x, y] = col
    hm.save(os.path.join(diag_dir, f'holes_{view}.png'))
    # ---- 2. 轮廓尖角 ----
    contour = trace_boundary(model)
    corners = 0
    if len(contour) > 30:
        pts = np.array(contour, dtype=float)
        angles = []
        n = len(pts)
        for i in range(0, n, 3):
            j = (i + 3) % n; k = (i - 3) % n
            v1 = pts[i] - pts[k]; v2 = pts[j] - pts[i]
            l1 = np.linalg.norm(v1) + 1e-9; l2 = np.linalg.norm(v2) + 1e-9
            c = np.clip(np.dot(v1, v2) / (l1 * l2), -1, 1)
            angles.append(np.degrees(np.arccos(c)))
        ang = np.array(angles)
        corners = int((ang > 60).sum())
        # 凸包比: 轮廓点集凸包面积 / mask 面积(凸包用简单投影: 逐行范围占比的凸包近似)
    else:
        corners = -1
    # ---- 3. 每行分段 ----
    seg_counts = []
    ys2, xs2 = np.where(model)
    ymin, ymax = ys2.min(), ys2.max()
    for y in range(ymin, ymax + 1):
        row = model[y]
        if not row.any(): continue
        nseg = 0
        prev = False
        for v in row:
            if v and not prev: nseg += 1
            prev = v
        seg_counts.append(nseg)
    seg_counts = np.array(seg_counts)
    frag_rows = int((seg_counts > 3).sum())
    max_seg = int(seg_counts.max())
    # ---- 4. 明暗突变 ----
    gray = a.mean(axis=2)
    gy, gx = np.gradient(gray.astype(float))
    gmag = np.sqrt(gx * gx + gy * gy)
    inner = model.copy()
    for _ in range(2):
        e = inner.copy()
        e[1:, :] &= inner[:-1, :]; e[:-1, :] &= inner[1:, :]
        e[:, 1:] &= inner[:, :-1]; e[:, :-1] &= inner[:, 1:]
        inner = e
    inner_px = int(inner.sum())
    harsh = float((gmag[inner] > 80).sum()) if inner_px else 0
    harsh_ratio = harsh / inner_px if inner_px else 0
    lum_std = float(gray[inner].std()) if inner_px else 0

    print(f'== {view} == {os.path.basename(path)}')
    print(f'  model px: {int(model.sum())}  bbox: x[{int(xs2.min())},{int(xs2.max())}] y[{ymin},{ymax}]')
    print(f'  holes(>=8px): {len(comps)} comps, {hole_px} px; big(>=60px): {len(big_comps)}  ({"OK" if len(big_comps) == 0 else "!! 中空"})')
    for (sz, bb, nb) in hole_locs:
        print(f'    big hole: {sz}px bbox{bb} norm{nb}')
    # ---- 轮廓平滑度(逐行边界): 粗糙度 + 行跨度填充率(凸性) ----
    ys2, xs2 = np.where(model)
    ymin, ymax = ys2.min(), ys2.max()
    Ls = []; Rs = []
    fills = []
    for y in range(ymin, ymax + 1):
        row = model[y]
        if not row.any(): continue
        cols = np.where(row)[0]
        Ls.append(cols.min()); Rs.append(cols.max())
        fills.append(len(cols) / (cols.max() - cols.min() + 1))
    Ls = np.array(Ls, dtype=float); Rs = np.array(Rs, dtype=float)
    d2L = np.abs(np.diff(Ls, 2)) if len(Ls) > 3 else np.array([0])
    d2R = np.abs(np.diff(Rs, 2)) if len(Rs) > 3 else np.array([0])
    rough = float((d2L.mean() + d2R.mean()) / 2)
    convexity = float(np.mean(fills))
    smooth_ok = rough < 1.5 and convexity > 0.90
    print(f'  boundary: roughness {rough:.2f}px, row-fill {convexity * 100:.1f}%  ({"圆润连续" if smooth_ok else "!! 边界不平/有凹口"})')
    print(f'  row segments: max {max_seg}, rows>3seg: {frag_rows}  ({"无条纹" if frag_rows == 0 else "!! 碎片/条纹"})')
    print(f'  luminance: std {lum_std:.1f}, harsh-edge {harsh_ratio * 100:.2f}%  ({"明暗平滑" if harsh_ratio < 0.03 else "注意明暗"})')

if __name__ == '__main__':
    view = sys.argv[1]
    png = sys.argv[2]
    diag = os.path.join(os.path.dirname(png), 'diag_ds0731')
    analyze(png, view, diag)
