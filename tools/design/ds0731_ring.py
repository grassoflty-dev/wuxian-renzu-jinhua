# -*- coding: utf-8 -*-
"""ds0731_ring.py — 沿径向线扫描俯视图的亮度剖面, 找肩部浅色环。
用法: python ds0731_ring.py <top_png>"""
import sys
from PIL import Image
import numpy as np

def main():
    f = sys.argv[1]
    a = np.asarray(Image.open(f).convert('RGB')).astype(float)
    gray = a.mean(axis=2)
    H, W = gray.shape
    mx = a.max(axis=2)
    model = mx < 246
    ys, xs = np.where(model)
    if len(ys) == 0:
        print('no model'); return
    cx = int(xs.mean()); cy = int(ys.mean())
    print(f'center ({cx},{cy}) bbox x[{xs.min()},{xs.max()}] y[{ys.min()},{ys.max()}]')
    # 沿 8 个方向的径向线, 从中心到模型边界, 输出每 5% 的亮度
    for ang in range(0, 360, 45):
        rad = np.radians(ang)
        dx, dy = np.cos(rad), np.sin(rad)
        prof = []
        maxr = int(np.hypot(W, H))
        for t in range(0, maxr, 3):
            x = int(cx + dx * t); y = int(cy + dy * t)
            if 0 <= x < W and 0 <= y < H and model[y, x]:
                prof.append((t, gray[y, x]))
            elif prof:
                break
        if len(prof) < 10: continue
        pts = np.array(prof)
        # 采样 8 个点
        samples = []
        for q in range(1, 9):
            idx = min(len(pts) - 1, int(len(pts) * q / 8))
            samples.append((int(pts[idx, 0]), int(pts[idx, 1])))
        print(f'ang {ang:3d}deg: ' + ' '.join(f'{r}:{int(v)}' for r, v in samples))

if __name__ == '__main__':
    main()
