# head_ratio_check.py — 量化立绘头部宽深比(DS0731 头部盒子感根因)
from PIL import Image
import numpy as np

base = r'C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\server-rs\ui\assets\img'
fr = np.array(Image.open(base + r'\pc_zhengzha.png').convert('RGBA'))
sd = np.array(Image.open(base + r'\pc_zhengzha_side.png').convert('RGBA'))
print('front', fr.shape, 'side', sd.shape)

def bbox(a):
    al = a[..., 3] > 15
    ys, xs = np.where(al)
    return xs.min(), xs.max(), ys.min(), ys.max()

fx0, fx1, fy0, fy1 = bbox(fr)
sx0, sx1, sy0, sy1 = bbox(sd)
print('front bbox x', fx0, fx1, 'y', fy0, fy1, '=> W', fx1 - fx0 + 1, 'H', fy1 - fy0 + 1)
print('side  bbox x', sx0, sx1, 'y', sy0, sy1, '=> W', sx1 - sx0 + 1, 'H', sy1 - sy0 + 1)

def row_widths(a, y0, y1, x0, x1):
    out = []
    for y in range(y0, y1):
        al = a[y, x0:x1 + 1, 3] > 15
        if al.any():
            xs = np.where(al)[0] + x0
            out.append((y, int(xs.min()), int(xs.max())))
    return out

for name, a, (bx0, bx1, by0, by1) in [('front', fr, (fx0, fx1, fy0, fy1)), ('side', sd, (sx0, sx1, sy0, sy1))]:
    hh = by1 - by0 + 1
    best = None
    for t in range(2, 25):
        y0 = by0 + int(hh * t / 100)
        y1 = by0 + int(hh * (t + 8) / 100)
        r = row_widths(a, y0, y1, bx0, bx1)
        if len(r) >= 3:
            wmax = max(x1 - x0 for _, x0, x1 in r)
            if best is None or wmax > best[3]:
                best = (t, y0, y1, wmax, len(r))
    print(name, 'head scan (t%, y0,y1, maxW, rows):', best)
    if best:
        t, y0, y1, wmax, n = best
        r = row_widths(a, y0, y1, bx0, bx1)
        # 头部中心行宽
        mid = r[len(r) // 2]
        print('  mid row y=%d x %d..%d width=%d' % (mid[0], mid[1], mid[2], mid[2] - mid[1] + 1))

# 侧面头部深度(前额到后脑): 在侧面立绘顶部扫描 z 区间最大宽度
hh = sy1 - sy0 + 1
best_s = None
for t in range(2, 25):
    y0 = sy0 + int(hh * t / 100)
    y1 = sy0 + int(hh * (t + 8) / 100)
    r = row_widths(sd, y0, y1, sx0, sx1)
    if len(r) >= 3:
        wmax = max(x1 - x0 for _, x0, x1 in r)
        if best_s is None or wmax > best_s[3]:
            best_s = (t, y0, y1, wmax, len(r))
print('side head depth scan:', best_s)

# 比例: 正面头宽 vs 侧面头深(需换算到同一高度尺度)
if best and best_s:
    fw = best[3]  # 正面头宽 px
    sw = best_s[3]  # 侧面头深 px
    fh = (fy1 - fy0 + 1)
    sh = (sy1 - sy0 + 1)
    # 归一化: 正面按 fh 归一, 侧面按 sh 归一
    ratio = (fw / fh) / (sw / sh)
    print('front head W/H=%.3f, side head D/H=%.3f, 宽深比(同身高归一)=%.3f' % (fw / fh, sw / sh, ratio))
    print('=> 若 ratio > 1.6: 头部截面宽扁, 斜视呈盒子/饼状(用户反馈的根源)')
