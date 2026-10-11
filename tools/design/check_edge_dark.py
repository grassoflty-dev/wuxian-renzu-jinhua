# -*- coding: utf-8 -*-
"""check_edge_dark.py — 检查图片主体边缘是否有暗色/黑边(无 scipy)
对每个 alpha=255 且 4-邻域有 alpha=0 的边缘像素,统计其 RGB 亮度。
"""
import sys
try:
    from PIL import Image
    import numpy as np
except ImportError:
    print("NEED PIL+numpy"); sys.exit(1)

def main():
    for f in sys.argv[1:]:
        im = Image.open(f).convert("RGBA")
        a = np.asarray(im)
        alpha = a[:,:,3]; rgb = a[:,:,:3]
        h, w = alpha.shape
        pad = np.pad(alpha, 1, mode='constant')
        edge = np.zeros((h, w), dtype=bool)
        for dy, dx in [(-1,0),(1,0),(0,-1),(0,1)]:
            edge |= (alpha==255) & (pad[1+dy:1+dy+h, 1+dx:1+dx+w]==0)
        n = int(edge.sum())
        if n:
            er = rgb[edge]
            lum = er.mean(axis=1)
            dark = int((lum < 60).sum())
            print("%s: 主体边缘 %d px, 暗边(<60) %d, 平均RGB %s" % (f.split(chr(92))[-1], n, dark, er.mean(axis=0).astype(int).tolist()))
        else:
            print("%s: 无主体边缘" % f.split(chr(92))[-1])

if __name__ == "__main__":
    main()
