# -*- coding: utf-8 -*-
"""erase_dark_rim.py — 擦除立绘人物轮廓最外圈的深色描边(AI 生成自带的暗轮廓)。
思路: 对主体(不透明)像素, 逐层(最多 3 层)向内部侵蚀:
  若某像素是"当前外圈"(4-邻域存在透明像素)且亮度 < 阈值 → 置透明;
  同时记录已擦层数, 避免把整个深色衣服/头发擦掉。
阈值默认 85: 亮度低于此视为"暗轮廓"(皮肤/浅色衣亮度通常 >100)。
用法: python erase_dark_rim.py <in.png> <out.png> [lum_thresh] [max_layers]
"""
import sys
try:
    from PIL import Image
    import numpy as np
except ImportError:
    print("NEED PIL+numpy"); sys.exit(1)

def main():
    src, dst = sys.argv[1], sys.argv[2]
    th = float(sys.argv[3]) if len(sys.argv) > 3 else 85
    max_layers = int(sys.argv[4]) if len(sys.argv) > 4 else 3
    im = Image.open(src).convert("RGBA")
    a = np.asarray(im).astype(np.float32)
    alpha = a[:, :, 3]
    rgb = a[:, :, :3]
    h, w = alpha.shape
    erased = 0
    for _ in range(max_layers):
        pad = np.pad(alpha, 1, mode='constant')
        is_edge = np.zeros((h, w), dtype=bool)
        for dy, dx in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
            is_edge |= (alpha == 255) & (pad[1+dy:1+dy+h, 1+dx:1+dx+w] == 0)
        lum = rgb.mean(axis=2)
        target = is_edge & (lum < th)
        n = int(target.sum())
        if n == 0:
            break
        alpha[target] = 0
        erased += n
    a[:, :, :3] = rgb
    a[alpha == 0, :3] = 255
    a[:, :, 3] = alpha
    Image.fromarray(a.astype(np.uint8), "RGBA").save(dst)
    print("erased %d dark rim px (th=%d, %d layers) -> %s" % (erased, int(th), max_layers, dst))

if __name__ == "__main__":
    main()
