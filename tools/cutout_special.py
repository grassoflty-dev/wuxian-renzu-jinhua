# -*- coding: utf-8 -*-
"""cutout_special.py — 复杂背景图标抠图（不依赖边界种子洪泛）。

背景为均匀近黑渐变 + 主体可能带暗部/辉光、边界可能被辉光覆盖（边界种子洪泛进不去）。
这里改为：把「距离背景色 within 阈值」的像素整体当作背景区域（全局，不经边界连通），
再用 floodfill 的 fix_holes 把「被不透明主体包围的暗部（主体自身暗部）」回填保留。

   警告：主体暗部若与外部背景/辉光(透明区)连通，会被一起切掉——这是同类图固有风险。

步骤:
  1. bg = 取图像最暗 ~30% 像素的中位色（近似远场背景，避开边界辉光/主体亮部）。
  2. near = 像素 RGB 到 bg 的欧氏距离 <= T （T 需覆盖背景渐变+边界辉光）。
  3. alpha = where(near, 0, 255)。
  4. fix_holes：对标 alpha==0 掩膜做 hole_channel 次闭运算后，再取「不接触图像边界」的
     连通域回填 ramp_alpha（或 --hole-solid 无条件 255）。注意：因为整个背景是大连通域且
     触边，主体暗部若与它连通则不会被回填。
  5. 边缘羽化 feather、闭运算 closing 清理。
  6. --zero-rgb 透明像素 RGB 置 0（默认开）。
  7. 输出透明 PNG。

用法:
  <python> cutout_special.py <输入png> <输出png> <T> [--bg R,G,B] [--feather N]
                   [--closing N] [--hole-channel N] [--hole-solid] [--no-fix-holes]
                   [--conn 4|8] [--keep-rgb]
"""
import argparse, os, sys
import numpy as np
from PIL import Image

try:
    import scipy.ndimage as ndi
    HAS_SCIPY = True
except Exception:
    HAS_SCIPY = False

from cutout_floodfill import (auto_bg, ramp_alpha, dilate_np, erode_np,
                              close_np, flood_mask, _shifts)

def cutout_special(src, dst, T, bg=None, feather=2, closing=1,
                   fix_holes=True, hole_channel=3, hole_solid=True,
                   conn=4, zero_rgb=True, dark_percentile=30.0):
    img = Image.open(src).convert("RGBA")
    arr = np.asarray(img).astype(np.float64)
    rgb = arr[..., :3]
    h, w = rgb.shape[:2]

    if bg is None:
        # 背景 ≈ 最暗 dark_percentile% 像素的中位色（避开边界辉光/主体亮部）
        lum = rgb.mean(axis=2)
        pthr = np.percentile(lum, dark_percentile)
        darkmask = lum <= pthr
        if darkmask.any():
            bg = np.median(rgb[darkmask], axis=0)
        else:
            bg = auto_bg(rgb)
    bg = np.asarray(bg, dtype=np.float64).reshape(3)

    d = np.sqrt(((rgb - bg) ** 2).sum(axis=2))
    near = d <= float(T)
    alpha = np.where(near, 0, 255).astype(np.uint8)

    # 边缘羽化：只对主体侧过渡带套渐变
    if feather > 0 and not near.all():
        inv = ~near
        ring = dilate_np(near, feather, conn) & inv
        if ring.any():
            ar = ramp_alpha(d)
            alpha[ring] = np.maximum(ar[ring], 8).astype(np.uint8)

    # 闭运算清理主体内部细小透明孔
    if closing > 0:
        sub_mask = alpha > 0
        closed = close_np(sub_mask, closing, conn) if not HAS_SCIPY \
            else ndi.binary_closing(sub_mask, structure=np.ones((3, 3), dtype=bool),
                                    iterations=closing)
        filled = closed & ~sub_mask
        if filled.any():
            alpha[filled] = 255

    # fix_holes: 回填「不接触图像边界」的内部暗部（主体自身凹坑/暗区）
    if fix_holes:
        zeros = alpha == 0
        if zeros.any():
            if hole_channel > 0:
                if HAS_SCIPY:
                    zc = ndi.binary_closing(zeros, structure=np.ones((3, 3), dtype=bool),
                                            iterations=hole_channel, border_value=1)
                else:
                    zc = close_np(zeros, hole_channel, conn)
                enclosed = zc & ~flood_mask(zc, conn)
            else:
                enclosed = zeros & ~flood_mask(zeros, conn)
            if enclosed.any():
                if hole_solid:
                    alpha[enclosed] = 255
                else:
                    ar = ramp_alpha(d)
                    alpha[enclosed] = ar[enclosed].astype(np.uint8)

    rgb8 = rgb.astype(np.uint8)
    if zero_rgb:
        rgb8 = np.where(alpha[..., None] > 0, rgb8, 0).astype(np.uint8)
    out = np.dstack([rgb8, alpha])
    Image.fromarray(out, "RGBA").save(dst, "PNG")

    chk = np.asarray(Image.open(dst).convert("RGBA"))
    al = chk[..., 3]
    total = al.size
    print("%s T=%g bg=%s trans=%.1f%% semi=%.1f%% opq=%.1f%% (scipy=%s)"
          % (os.path.basename(dst), T, str(tuple(int(x) for x in bg)),
             (al <= 5).mean() * 100, ((al > 5) & (al < 250)).mean() * 100,
             (al >= 250).mean() * 100, HAS_SCIPY), flush=True)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("paths", nargs="+")
    ap.add_argument("--bg", default=None)
    ap.add_argument("--feather", type=int, default=2)
    ap.add_argument("--closing", type=int, default=1)
    ap.add_argument("--no-fix-holes", dest="fix_holes", action="store_false", default=True)
    ap.add_argument("--hole-channel", type=int, default=3)
    ap.add_argument("--no-hole-solid", dest="hole_solid", action="store_false", default=True)
    ap.add_argument("--conn", type=int, default=4, choices=(4, 8))
    ap.add_argument("--keep-rgb", dest="zero_rgb", action="store_false", default=True)
    ap.add_argument("--dark-percentile", type=float, default=30.0)
    args = ap.parse_args()

    bg = tuple(int(x) for x in args.bg.split(",")) if args.bg else None
    p = list(args.paths)
    if len(p) >= 2 and os.path.isfile(p[0]):
        T = float(p[2]) if len(p) > 2 else 20.0
        cutout_special(p[0], p[1], T, bg=bg, feather=args.feather,
                       closing=args.closing, fix_holes=args.fix_holes,
                       hole_channel=args.hole_channel, hole_solid=args.hole_solid,
                       conn=args.conn, zero_rgb=args.zero_rgb,
                       dark_percentile=args.dark_percentile)
        print("done", flush=True)
    else:
        ap.print_help()

if __name__ == "__main__":
    main()
