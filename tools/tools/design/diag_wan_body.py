# 体检 wan2.7-image 生成的 768x1024 立绘:
#   1) 纯黑背景占比 / 是否消干净
#   2) 主体 bbox / y_extent / 高度占比
#   3) bottom 8% 是否有内容(脚是否到底)
#   4) 主体是否连通(粗分块:头部区/中段/下段非零占比)
import sys
from PIL import Image
import numpy as np

path = sys.argv[1] if len(sys.argv) > 1 else "design/raw_enemy/wan_test1.png"
im = Image.open(path).convert("RGB")
a = np.asarray(im).astype(np.int16)
H, W, _ = a.shape
lum = a.mean(axis=2)

dark_bg = lum < 24
bg_ratio = dark_bg.mean()

body = lum >= 40
ys, xs = np.where(body)
if len(ys) == 0:
    print(f"NO BODY {path}")
    sys.exit(0)

y0, y1 = ys.min(), ys.max()
x0, x1 = xs.min(), xs.max()
bh = y1 - y0 + 1
bw = x1 - x0 + 1
print(f"size={W}x{H} bg_dark_ratio={bg_ratio:.3f}")
print(f"body_ratio={body.mean():.3f} bbox=({x0},{y0})-({x1},{y1}) w={bw} h={bh}")
print(f"y_extent_top={y0/H:.3f} bottom={y1/H:.3f} body_h_ratio={bh/H:.3f}")
print(f"top_gap_px={y0} bottom_gap_px={H-1-y1}")

# bottom 8% 内容
b8 = body[int(H*0.92):, :]
print(f"bottom8%_content_ratio={b8.mean():.3f}")

# 分段非零占比(上/中/下三段)
for name, sl in [("head_seg", slice(0, H//3)), ("mid_seg", slice(H//3, 2*H//3)), ("low_seg", slice(2*H//3, H))]:
    seg = body[sl]
    print(f"{name}_ratio={seg.mean():.3f}")

# 中心列(避免背景边缘误判)
cx = W // 2
col = body[:, max(0, cx-80):cx+80]
print(f"center_col_ratio={col.mean():.3f}")