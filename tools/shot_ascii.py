# -*- coding: utf-8 -*-
import sys
from PIL import Image
import numpy as np
im = Image.open(sys.argv[1]).convert("RGB")
print("size", im.size)
a = np.array(im).astype(np.float32).mean(axis=2)
H, W = a.shape
# 粗粒度亮度图
gh, gw = 36, 104
sy, sx = H//gh, W//gw
ramp = " .:-=+*#%@"
print("max brightness", a.max())
for j in range(gh):
    line = ""
    for i in range(gw):
        blk = a[j*sy:(j+1)*sy, i*sx:(i+1)*sx]
        line += ramp[min(9, int(blk.mean()/26))]
    print("%2d|%s|" % (j, line))
# 亮像素分布
for thr in (60, 90, 120, 150, 180):
    m = a > thr
    if m.sum() == 0: print("thr", thr, "none"); continue
    ys, xs = np.nonzero(m)
    print("thr %3d n=%6d  x %d..%d  y %d..%d" % (thr, m.sum(), xs.min(), xs.max(), ys.min(), ys.max()))
