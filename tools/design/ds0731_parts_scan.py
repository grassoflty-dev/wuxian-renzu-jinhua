# -*- coding: utf-8 -*-
"""ds0731_parts_scan.py — 细化扫描正/侧立绘的过渡区 runs 结构, 供部位分离设计。
输出: 每 1.5% 高度的 runs; 标注 头/颈/肩/躯干臂区/腿区 边界。"""
import numpy as np
from PIL import Image

def runs_of_row(a, y, thresh=127, gap=2):
    W = a.shape[1]
    al = a[y, :, 3]
    on = al > thresh
    out = []
    l = -1; g = 0
    for x in range(W):
        if on[x]:
            if l < 0: l = x
            g = 0
        elif l >= 0:
            g += 1
            if g > gap:
                if x - g - l >= 3: out.append((l, x - g - 1))
                l = -1; g = 0
    if l >= 0 and W - l >= 3: out.append((l, W - 1))
    return out

base = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\server-rs\ui\assets\img"
a = np.asarray(Image.open(base + r'\pc_zhengzha.png').convert('RGBA'))
al = a[:, :, 3]
ys, xs = np.where(al > 15)
bb = (int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max()))
H = bb[3] - bb[1] + 1
print(f'front bbox: {bb}, height {H}')

print('--- 头/颈/肩过渡 (v 0-35%) ---')
for frac in range(0, 36, 2):
    y = bb[1] + int(H * frac / 100)
    rs = runs_of_row(a, y)
    if rs:
        w = max(r[1]-r[0] for r in rs)
        print(f'v={frac:2d}% y={y}: {len(rs)} run(s) {rs} maxW={w}')

print('--- 躯干/腿过渡 (v 48-66%) ---')
for frac in range(48, 68, 2):
    y = bb[1] + int(H * frac / 100)
    rs = runs_of_row(a, y)
    if rs:
        w = max(r[1]-r[0] for r in rs)
        print(f'v={frac:2d}% y={y}: {len(rs)} run(s) {rs} maxW={w}')

# 3-run 区躯干边界统计
print('--- 躯干边界 (3-run 行) ---')
gl = []; gr = []
for frac in range(30, 55, 1):
    y = bb[1] + int(H * frac / 100)
    rs = runs_of_row(a, y)
    if len(rs) == 3:
        gl.append(rs[1][0]); gr.append(rs[1][1])
if gl:
    import statistics
    print('torso L: median', int(statistics.median(gl)), 'range', min(gl), max(gl))
    print('torso R: median', int(statistics.median(gr)), 'range', min(gr), max(gr))
