# -*- coding: utf-8 -*-
"""peel_edges.py — 从人物主体外圈剥离 N 层像素(全部变透明), 清除厚暗边/黑边残留。
原理: 从 alpha=255 的边界向内逐层剥离, 每层无条件置透明(不管颜色)。
层数按需: 侧面约 8 层, 背面约 12 层(看 analyze_rings 的暗层数)。
用法: python peel_edges.py <in.png> <out.png> <layers>
"""
import sys
try:
    from PIL import Image
    import numpy as np
except ImportError:
    print("NEED PIL+numpy"); sys.exit(1)

def main():
    src, dst = sys.argv[1], sys.argv[2]
    layers = int(sys.argv[3]) if len(sys.argv) > 3 else 8
    im = Image.open(src).convert("RGBA")
    a = np.asarray(im)
    alpha = a[:,:,3].copy()
    rgb = a[:,:,:3].copy()
    h, w = alpha.shape
    body = alpha == 255
    peeled = 0
    for _ in range(layers):
        pad = np.pad(body, 1, mode='constant')
        edge = np.zeros((h,w), dtype=bool)
        for dy,dx in [(-1,0),(1,0),(0,-1),(0,1)]:
            edge |= body & ~pad[1+dy:1+dy+h, 1+dx:1+dx+w]
        n = int(edge.sum())
        if n == 0: break
        body[edge] = False
        alpha[edge] = 0
        peeled += n
    # 透明像素 RGB 清白
    rgb[alpha == 0] = 255
    out = np.dstack([rgb, alpha]).astype(np.uint8)
    Image.fromarray(out, "RGBA").save(dst)
    print("peeled %d px over %d layers -> %s" % (peeled, layers, dst))

if __name__ == "__main__":
    main()
