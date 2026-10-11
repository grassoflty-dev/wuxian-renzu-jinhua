# -*- coding: utf-8 -*-
"""fix_edge_black.py — 修复抠图边缘黑边: 半透明边缘像素带黑色背景色 → unpremultiply 去黑。
对 0<alpha<255 的像素: 把 RGB 从"被黑背景稀释的色"还原为"纯前景色":
  pure = rgb / (alpha/255)  (钳制到 255)
这样半透明边缘显示在任意背景上都干净(不再是黑边)。
用法: python fix_edge_black.py <in.png> <out.png>
"""
import sys
try:
    from PIL import Image
    import numpy as np
except ImportError as e:
    print("NEED PIL+numpy:", e); sys.exit(1)

def main():
    src, dst = sys.argv[1], sys.argv[2]
    im = Image.open(src).convert("RGBA")
    a = np.asarray(im).astype(np.float32)
    rgb = a[:, :, :3]
    alpha = a[:, :, 3]
    edge = (alpha > 0) & (alpha < 255)
    if edge.any():
        k = 255.0 / np.maximum(alpha[edge], 1)[:, None]     # unpremultiply 系数
        new = rgb[edge] * k
        rgb[edge] = np.clip(new, 0, 255)
        # 完全透明像素 RGB 清白(防预览深底显黑)
    a[:, :, :3] = rgb
    a[alpha == 0, :3] = 255
    Image.fromarray(a.astype(np.uint8), "RGBA").save(dst)
    print("fixed edge black: %d px -> %s" % (int(edge.sum()), dst))

if __name__ == "__main__":
    main()
