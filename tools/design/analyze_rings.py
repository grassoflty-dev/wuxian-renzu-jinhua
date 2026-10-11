# -*- coding: utf-8 -*-
"""analyze_rings.py — 分析人物边缘的圈层结构: 从透明区向主体内逐层扫描, 统计每层平均颜色/亮度。
用法: python analyze_rings.py <img.png>
"""
import sys
try:
    from PIL import Image
    import numpy as np
except ImportError:
    print("NEED PIL+numpy"); sys.exit(1)

def main():
    f = sys.argv[1]
    im = Image.open(f).convert("RGBA")
    a = np.asarray(im)
    alpha = a[:,:,3]
    rgb = a[:,:,:3]
    h, w = alpha.shape
    body = alpha == 255
    # 逐层从透明边界向主体内部侵蚀, 每层输出该层像素的平均 RGB/亮度
    pad = np.pad(body, 1, mode='constant')
    for layer in range(10):
        edge = np.zeros((h,w), dtype=bool)
        for dy,dx in [(-1,0),(1,0),(0,-1),(0,1)]:
            edge |= body & ~pad[1+dy:1+dy+h, 1+dx:1+dx+w]
        n = int(edge.sum())
        if n == 0: break
        e = rgb[edge]
        lum = e.mean(axis=1)
        print("layer %d: %d px, 亮度 %d, RGB %s" % (layer, n, round(lum.mean()), e.mean(axis=0).astype(int).tolist()))
        body[edge] = False
        pad = np.pad(body, 1, mode='constant')

if __name__ == "__main__":
    main()
