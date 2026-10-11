# -*- coding: utf-8 -*-
"""打印游戏内渲染结果在 art 分辨率下的字符图（3x 截图 → 6 device px = 1 art px）。"""
import sys
import numpy as np
from PIL import Image
f = sys.argv[1]
X0, X1 = int(sys.argv[2]), int(sys.argv[3])   # art 列范围（0..64，中心 32）
Y0, Y1 = int(sys.argv[4]), int(sys.argv[5])   # art 行范围（0..104）
im = Image.open(f).convert("RGB")
a = np.array(im).astype(np.float32)
# 截图 3x 于 CSS，CSS = 2x art（zoom=2）→ 6 物理像素 = 1 art px
K = 6
# 角色在本裁剪里的位置：裁剪脚本以 (sx, sy) 为锚，sx 在裁剪宽度中央，sy 在 84% 高度
cw = a.shape[1]; ch = a.shape[0]
fx = cw / 2.0; fy = ch * 0.84          # 脚锚在该裁剪中的像素位置
ramp = " .:-=+*#%@"
print("裁剪尺寸", a.shape[1], a.shape[0], "脚锚", int(fx), int(fy))
for ay in range(Y0, Y1):
    y = int(round(fy + (ay - 104) * K + K / 2))
    line = ""
    for ax in range(X0, X1):
        x = int(round(fx + (ax - 32) * K + K / 2))
        if 0 <= x < cw and 0 <= y < ch:
            blk = a[max(0,y-2):y+3, max(0,x-2):x+3]
            v = blk.mean()
            line += ramp[min(9, int(v / 26))]
        else:
            line += "~"
    print("%3d|%s|" % (ay, line))
