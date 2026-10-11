# verify_bodymask.py — 细节(梯度)驱动的身体分割:烟雾模糊无边缘,身体有轮廓线/褶皱强边缘
import numpy as np
from PIL import Image
import os
from collections import deque

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\model_task_pkg"
OUT = os.path.join(BASE, "shots", "diag")

def dilate(m, rad):
    for _ in range(rad):
        up = np.roll(m, 1, 0); up[0, :] = 0
        dn = np.roll(m, -1, 0); dn[-1, :] = 0
        lf = np.roll(m, 1, 1); lf[:, 0] = 0
        rt = np.roll(m, -1, 1); rt[:, -1] = 0
        m = m | up | dn | lf | rt
    return m

def erode(m, rad):
    for _ in range(rad):
        up = np.roll(m, 1, 0); up[0, :] = 1
        dn = np.roll(m, -1, 0); dn[-1, :] = 1
        lf = np.roll(m, 1, 1); lf[:, 0] = 1
        rt = np.roll(m, -1, 1); rt[:, -1] = 1
        m = m & up & dn & lf & rt
    return m

def largest_cc(m):
    H, W = m.shape
    lab = np.zeros((H, W), np.int32)
    best_size, best_lab, cur = 0, 0, 0
    visited = np.zeros((H, W), bool)
    for sy in range(0, H, 1):
        for sx in range(0, W, 1):
            if not m[sy, sx] or visited[sy, sx]:
                continue
            cur += 1
            q = deque([(sy, sx)])
            visited[sy, sx] = True
            size = 0
            while q:
                y, x = q.popleft()
                lab[y, x] = cur
                size += 1
                for dy, dx in ((1,0),(-1,0),(0,1),(0,-1)):
                    ny, nx = y+dy, x+dx
                    if 0 <= ny < H and 0 <= nx < W and m[ny, nx] and not visited[ny, nx]:
                        visited[ny, nx] = True
                        q.append((ny, nx))
            if size > best_size:
                best_size, best_lab = size, cur
    return lab == best_lab, best_size

def fill_holes(m):
    # 从边界洪泛标记"外部",孔=未到达的 0
    H, W = m.shape
    ext = np.zeros((H, W), bool)
    q = deque()
    for x in range(W):
        for y in (0, H-1):
            if not m[y, x] and not ext[y, x]:
                ext[y, x] = True; q.append((y, x))
    for y in range(H):
        for x in (0, W-1):
            if not m[y, x] and not ext[y, x]:
                ext[y, x] = True; q.append((y, x))
    while q:
        y, x = q.popleft()
        for dy, dx in ((1,0),(-1,0),(0,1),(0,-1)):
            ny, nx = y+dy, x+dx
            if 0 <= ny < H and 0 <= nx < W and not m[ny, nx] and not ext[ny, nx]:
                ext[ny, nx] = True; q.append((ny, nx))
    return m | ~ext  # 外部=1,孔也变 1

for name in ["pc_zhengzha_side", "pc_zhengzha_back"]:
    im = Image.open(os.path.join(BASE, "assets", name + ".png")).convert("RGBA")
    a = np.asarray(im).astype(np.int32)
    rgb, alpha = a[..., :3], a[..., 3]
    H, W = alpha.shape
    # 梯度幅度(灰度,4 邻域最大差)
    g = rgb.mean(axis=2)
    gx = np.abs(np.roll(g, -1, 1) - g); gx[:, -1] = 0
    gy = np.abs(np.roll(g, -1, 0) - g); gy[-1, :] = 0
    grad = np.maximum(gx, gy)
    for thr in [45]:
        strong = grad > thr
        rough = dilate(strong, 5)
        cc, sz = largest_cc(rough)
        filled = fill_holes(cc)
        # 与 alpha 主体相交(去远端噪点),再腐蚀 1 收边
        mask = filled & (alpha > 100)
        mask = erode(mask, 1)
        out_a = np.where(mask, alpha, 0).astype(np.uint8)
        # 白底合成
        comp = np.ones((H, W, 3), np.float32) * 255
        af = (out_a[..., None] / 255.0).astype(np.float32)
        comp = rgb * af + comp * (1 - af)
        ys, xs = np.where(out_a > 8)
        pad = 30
        crop = comp[max(0, ys.min()-pad):min(H, ys.max()+pad), max(0, xs.min()-pad):min(W, xs.max()+pad)]
        Image.fromarray(crop.astype(np.uint8)).save(os.path.join(OUT, name + "_bodymask_onwhite.png"))
        Image.fromarray((mask * 255).astype(np.uint8)).save(os.path.join(OUT, name + "_bodymask.png"))
        print(name, "thr", thr, "bodymask px:", int(mask.sum()), "cc size:", sz)
print("saved")
