# -*- coding: utf-8 -*-
"""ds0731h_check.py — DS0731H 人体底模通道客观检查。
1. front 躯干/手臂颜色(三态无混色)
2. upper/side 斜视手臂内侧凹陷(腋下/手臂间隙)
3. foot 脚部
4. 各视图空洞"""
import numpy as np
from PIL import Image
import sys, os

SHOTS = r'C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots'

def bbox(path):
    a = np.array(Image.open(path).convert('RGB')).astype(int)
    mx = a.max(axis=2)
    ys, xs = np.where(mx < 246)
    return a, (xs.min(), ys.min(), xs.max(), ys.max()) if len(xs) else None

print('=== 1. front colors ===')
a, bb = bbox(os.path.join(SHOTS, 'render_sprite_ds0731h_zhenghai_front.png'))
x0, y0, x1, y1 = bb
w, h = x1 - x0, y1 - y0
for fx in range(0, 100, 25):
    xa = x0 + int(w * fx / 100); xb = x0 + int(w * (fx + 12) / 100)
    reg = a[y0 + int(h * 0.42):y0 + int(h * 0.50), xa:xb].reshape(-1, 3)
    m = np.median(reg, axis=0).astype(int)
    print('  front x %d-%d%%: med %s' % (fx, fx + 12, tuple(m)))

print('=== 2. upper arm-pit / arm gap ===')
a2, bb2 = bbox(os.path.join(SHOTS, 'render_sprite_ds0731h_zhenghai_upper.png'))
x0, y0, x1, y1 = bb2
h = y1 - y0
# 上臂区(40-60% 高度)每行的段数(手臂与躯干分离=有间隙/凹陷)
for frac in [0.40, 0.48, 0.56]:
    y = y0 + int(h * frac)
    row = a2[y]
    nw = ~np.all(row > 230, axis=1)
    xs_ = np.where(nw)[0]
    if len(xs_) == 0:
        print('  y%d%%: empty' % int(frac * 100)); continue
    segs = []
    run = []
    for xx in xs_:
        if not run or xx == run[-1] + 1: run.append(xx)
        else:
            if len(run) >= 15: segs.append((run[0], run[-1]))
            run = [xx]
    if len(run) >= 15: segs.append((run[0], run[-1]))
    print('  upper y%d%%: segs=%s' % (int(frac * 100), [(s[1] - s[0] + 1) for s in segs]))

print('=== 3. foot ===')
a3, bb3 = bbox(os.path.join(SHOTS, 'render_sprite_ds0731h_zhenghai_foot.png'))
if bb3:
    x0, y0, x1, y1 = bb3
    print('  foot bbox w=%d h=%d model_px=%d' % (x1 - x0, y1 - y0, int((a3.max(axis=2) < 246).sum())))

print('=== 4. holes (top/bottom/profile/back) ===')
for v in ['top', 'bottom', 'profile', 'back', 'face']:
    p = os.path.join(SHOTS, 'render_sprite_ds0731h_zhenghai_%s.png' % v)
    a4 = np.array(Image.open(p).convert('RGB')).astype(int)
    mx = a4.max(axis=2)
    model = mx < 246
    ys, xs = np.where(model)
    if len(xs) == 0:
        print('  %s: EMPTY' % v); continue
    y0, y1, x0, x1 = ys.min(), ys.max(), xs.min(), xs.max()
    bg = ~model
    ext = np.zeros_like(bg)
    q = []
    for x in range(x1 + 1):
        if bg[0, x] and not ext[0, x]: ext[0, x] = True; q.append((0, x))
        if bg[y1, x] and not ext[y1, x]: ext[y1, x] = True; q.append((y1, x))
    for y in range(y1 + 1):
        if bg[y, 0] and not ext[y, 0]: ext[y, 0] = True; q.append((y, 0))
        if bg[y, x1] and not ext[y, x1]: ext[y, x1] = True; q.append((y, x1))
    while q:
        y, x = q.pop()
        for dy, dx in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
            ny, nx = y + dy, x + dx
            if 0 <= ny <= y1 and 0 <= nx <= x1 and bg[ny, nx] and not ext[ny, nx]:
                ext[ny, nx] = True; q.append((ny, nx))
    holes = (bg & ~ext).sum()
    print('  %s: holes=%d px' % (v, int(holes)))
