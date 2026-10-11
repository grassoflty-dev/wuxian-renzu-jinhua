# -*- coding: utf-8 -*-
"""fix_edge_black2.py — 彻底去边缘黑/暗边(更激进):
1. 半透明边缘像素(0<alpha<255) → unpremultiply 去黑背景色
2. 再对边缘做一次"亮度提升 + 轻微收缩":
   - 若 unpremultiply 后仍很暗(<60)且 alpha 低(<160), 说明是"黑色背景 + 主体暗轮廓"混合,
     直接把该像素 alpha 降到 0(收缩边缘, 宁可少一圈也不留暗边)
   - 其余边缘像素 alpha 提到 255(钉死, 防羽化边缘在浅底显暗)
用法: python fix_edge_black2.py <in.png> <out.png>
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
    # 1) unpremultiply
    if edge.any():
        k = 255.0 / np.maximum(alpha[edge], 1)[:, None]
        rgb[edge] = np.clip(rgb[edge] * k, 0, 255)
    # 2) 收缩: unpremultiply 后仍暗(<70)且半透明(<170) → 视为黑底暗边, alpha 归零
    lum = rgb.mean(axis=2)
    dark_edge = edge & (lum < 70) & (alpha < 170)
    alpha[dark_edge] = 0
    # 3) 剩余半透明边缘钉到 255(防浅底显暗)
    remain = (alpha > 0) & (alpha < 255)
    alpha[remain] = 255
    # 4) 完全透明像素 RGB 清白
    a[:, :, :3] = rgb
    a[alpha == 0, :3] = 255
    a[:, :, 3] = alpha
    Image.fromarray(a.astype(np.uint8), "RGBA").save(dst)
    print("fixed v2: shrink %d dark-edge px, pin %d remain px -> %s" % (int(dark_edge.sum()), int(remain.sum()), dst))

if __name__ == "__main__":
    main()
