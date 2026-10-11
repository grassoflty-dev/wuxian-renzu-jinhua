# -*- coding: utf-8 -*-
"""ds0731_probe_colors.py — 探测正面立绘颈部/边缘颜色(找青色 rim 线)与整体边缘像素统计。"""
import sys
from PIL import Image
import numpy as np

def main():
    base = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\server-rs\ui\assets\img"
    im = Image.open(base + '\\pc_zhengzha.png').convert('RGBA')
    a = np.asarray(im)
    H, W = a.shape[:2]
    al = a[:, :, 3]
    ys, xs = np.where(al > 15)
    bb = (int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max()))
    print('bbox:', bb)
    # 颈部区域 v=12-20% (y = bb.top + 0.12..0.20 * h)
    y0, y1 = bb[1] + int(0.12 * (bb[3] - bb[1])), bb[1] + int(0.20 * (bb[3] - bb[1]))
    print('neck y range:', y0, y1)
    for y in range(y0, y1 + 1, 12):
        row = a[y]
        on = row[:, 3] > 127
        if on.sum() == 0: continue
        xl, xr = int(np.where(on)[0].min()), int(np.where(on)[0].max())
        # 边缘像素(左右各 6px)与内部像素(中心 40%)
        edge = row[xl:xl + 6][:, :3], row[xr - 5:xr + 1][:, :3]
        cx0, cx1 = xl + int((xr - xl) * 0.30), xr - int((xr - xl) * 0.30)
        inner = row[cx0:cx1][:, :3]
        def stat(p):
            if len(p) == 0: return None
            m = p.mean(axis=0)
            return (int(m[0]), int(m[1]), int(m[2]), int(p.min()))
        print(f'y={y} x=[{xl},{xr}] left-edge={stat(edge[0])} right-edge={stat(edge[1])} inner={stat(inner)}')
    # 全图边缘像素统计: 蓝青(b > r+20)像素占比, 亮白占比
    on = al > 40
    rgb = a[:, :, :3].astype(int)
    cyan = on & (rgb[:, :, 2] > rgb[:, :, 0] + 20) & (rgb[:, :, 2] > 150)
    bright = on & (rgb.min(axis=2) > 200)
    print('opaque px:', int(on.sum()), 'cyan-ish:', int(cyan.sum()), 'bright-white:', int(bright.sum()))

if __name__ == '__main__':
    main()
