# extract_head_contour.py — 提取头部轮廓为黑白线图(供 qwen 纯几何判读)
# 用法: python extract_head_contour.py <head45|head135> <输出.png>
from PIL import Image, ImageDraw
import numpy as np
import sys

base = r'C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots'
view = sys.argv[1] if len(sys.argv) > 1 else 'head45'
name = 'render_sprite_ds0731_zhenghai_' + view
outp = sys.argv[2] if len(sys.argv) > 2 else (base + '\\contour_' + view + '.png')

img = np.array(Image.open(base + '\\' + name + '.png').convert('RGB')).astype(np.float32)
H, W, _ = img.shape
bg = np.all(img > 235, axis=2)
fg = ~bg
ys, xs = np.where(fg)
if len(xs) == 0:
    print('NO fg'); raise SystemExit
x0, x1, y0, y1 = xs.min(), xs.max(), ys.min(), ys.max()
print('fg bbox x %d..%d y %d..%d' % (x0, x1, y0, y1))

# 头部区: 顶部 6%~36% (完整头顶弧+两侧上部, 排除颈部)
ta = y0 + int((y1-y0)*0.06)
tb = y0 + int((y1-y0)*0.36)

# 提取每行左右轮廓点(3px 中值平滑)
byrow = {}
for y in range(ta, tb):
    f = fg[y]
    if f.any():
        xx = np.where(f)[0]
        byrow[y] = (int(xx.min()), int(xx.max()))

# 画布: 白色背景, 黑色轮廓点连线
pad = 8
w = (x1 - x0) + pad * 2
h = (tb - ta) + pad * 2
cv = Image.new('RGB', (w, h), (255, 255, 255))
dr = ImageDraw.Draw(cv)

def pts_smooth(vals, k=2):
    arr = np.array(vals, float)
    return [int(np.median(arr[max(0,i-k):min(len(arr), i+k+1)])) for i in range(len(arr))]

ys_ = sorted(byrow.keys())
lefts = pts_smooth([byrow[y][0] for y in ys_])
rights = pts_smooth([byrow[y][1] for y in ys_])
pts_l = [(l - x0 + pad, y - ta + pad) for y, l in zip(ys_, lefts)]
pts_r = [(r - x0 + pad, y - ta + pad) for y, r in zip(ys_, rights)]

# 画轮廓(线宽 4)
dr.line(pts_l, fill=(0, 0, 0), width=4)
dr.line(pts_r, fill=(0, 0, 0), width=4)
# 画采样点(小圆)
for p in pts_l[::3]: dr.ellipse([p[0]-3, p[1]-3, p[0]+3, p[1]+3], fill=(0, 0, 0))
for p in pts_r[::3]: dr.ellipse([p[0]-3, p[1]-3, p[0]+3, p[1]+3], fill=(0, 0, 0))

cv.save(outp)
print('saved', outp, w, 'x', h, ' rows', len(ys_))
