# -*- coding: utf-8 -*-
"""clean_edge_final.py — 彻底清理人物轮廓外圈的暗色/黑边像素(视觉黑边根治)
对主体(alpha=255)的最外圈像素:
  1. 若该像素亮度 < 90(暗色), 且它是"轮廓凸出"点(3x3 邻域 alpha=255 少于 5 个, 即薄/尖端),
     直接置为透明(收缩边缘, 防止浅底显黑边);
  2. 其余暗色边缘像素做轻微提亮(×1.35 钳制255), 保留轮廓但不再"黑"。
重复 2 轮收缩, 清理突出毛刺。
用法: python clean_edge_final.py <in.png> <out.png>
"""
import sys
try:
    from PIL import Image
    import numpy as np
except ImportError:
    print("NEED PIL+numpy"); sys.exit(1)

def main():
    src, dst = sys.argv[1], sys.argv[2]
    im = Image.open(src).convert("RGBA")
    a = np.asarray(im).astype(np.float32)
    alpha = a[:, :, 3]
    rgb = a[:, :, :3]
    h, w = alpha.shape
    for _ in range(2):  # 2 轮
        pad = np.pad(alpha, 1, mode='constant')
        # 3x3 邻域 alpha=255 计数
        cnt = np.zeros((h, w))
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                cnt += (pad[1+dy:1+dy+h, 1+dx:1+dx+w] == 255)
        lum = rgb.mean(axis=2)
        # 边缘暗色且薄/尖(邻域实心少) → 收缩透明
        shrink = (alpha == 255) & (lum < 90) & (cnt < 5)
        # 边缘暗色但较实心 → 提亮
        lighten = (alpha == 255) & (lum < 90) & (cnt >= 5)
        alpha[shrink] = 0
        rgb[lighten] = np.clip(rgb[lighten] * 1.35, 0, 255)
    a[:, :, :3] = rgb
    a[alpha == 0, :3] = 255
    a[:, :, 3] = alpha
    Image.fromarray(a.astype(np.uint8), "RGBA").save(dst)
    print("cleaned edge -> %s" % dst)

if __name__ == "__main__":
    main()
