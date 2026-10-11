# -*- coding: utf-8 -*-
"""strip_white_rim.py — 去除 AI 生成立绘的白色/淡蓝轮廓光描边(纯黑背景素材专用)。
原理: 白色描边是"贴近黑色背景但亮度极高"的过渡带像素。对每个像素:
  若 亮度>thresh 且 像素与图像边缘/黑背景连通(其 8-邻域里有近背景像素) → 压暗到背景色。
这避免误伤主体内部的高光(如 T 恤高光、皮肤高光——它们周围不是黑背景)。
用法: python strip_white_rim.py <in.png> <out.png> [lum_thresh] [bg_dist]
"""
import sys
try:
    from PIL import Image
    import numpy as np
except ImportError as e:
    print("NEED PIL+numpy:", e); sys.exit(1)

def main():
    src, dst = sys.argv[1], sys.argv[2]
    lum_th = float(sys.argv[3]) if len(sys.argv) > 3 else 110   # 亮度阈值
    bg_d = float(sys.argv[4]) if len(sys.argv) > 4 else 40      # "邻域近背景"判定距离
    im = Image.open(src).convert("RGB")
    a = np.asarray(im, dtype=np.float32)
    h, w, _ = a.shape
    corners = np.concatenate([a[:8,:8].reshape(-1,3), a[:8,-8:].reshape(-1,3),
                              a[-8:,:8].reshape(-1,3), a[-8:,-8:].reshape(-1,3)], axis=0)
    bg = np.median(corners, axis=0)
    dist = np.sqrt(((a - bg) ** 2).sum(axis=2))      # 每像素到背景色距离
    lum = a.mean(axis=2)
    near_bg = dist < bg_d                              # 近背景掩膜
    # 8-邻域扩张的近背景(numpy 手写 3x3 膨胀, 无 scipy)
    pad = np.pad(near_bg, 1, mode='constant', constant_values=True)  # 边缘视为近背景
    near8 = np.zeros_like(near_bg)
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            near8 |= pad[1+dy:1+dy+h, 1+dx:1+dx+w]
    # 白边候选 = 高亮 & 邻近(8-邻域)有近背景像素
    rim = (lum > lum_th) & near8
    out = a.copy()
    ratio = np.where(rim[...,None], 0.06, 1.0)
    out = bg[None,None,:] + (a - bg[None,None,:]) * ratio
    out = np.clip(out, 0, 255).astype(np.uint8)
    Image.fromarray(out).save(dst)
    print("stripped %d rim px -> %s" % (int(rim.sum()), dst))

if __name__ == "__main__":
    main()
