# head_geo_check2.py — 改进版: 排除头发尖干扰, 只测头骨轮廓 + 脸颊断层
from PIL import Image
import numpy as np

base = r'C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots'
name = 'render_sprite_ds0731_zhenghai_head45'

img = np.array(Image.open(base + '\\' + name + '.png').convert('RGB')).astype(np.float32)
H, W, _ = img.shape
bg = np.all(img > 235, axis=2)
fg = ~bg
ys, xs = np.where(fg)
if len(xs) == 0:
    print('NO fg'); raise SystemExit
x0, x1, y0, y1 = xs.min(), xs.max(), ys.min(), ys.max()
print('fg bbox x %d..%d y %d..%d (w=%d h=%d)' % (x0, x1, y0, y1, x1-x0+1, y1-y0+1))

# ---- ① 头骨轮廓(排除顶部发尖): 顶部 14%~30% 区, 中值平滑 ----
top_a = y0 + int((y1-y0)*0.14)
top_b = y0 + int((y1-y0)*0.30)
rows_l, rows_r = [], []
for y in range(top_a, top_b):
    f = fg[y]
    if f.any():
        xx = np.where(f)[0]
        rows_l.append((y, int(xx.min())))
        rows_r.append((y, int(xx.max())))

def smooth(pts, k=3):
    out = []
    xs_ = [p[1] for p in pts]
    for i in range(len(pts)):
        lo = max(0, i-k); hi = min(len(pts), i+k+1)
        out.append((pts[i][0], int(np.median(xs_[lo:hi]))))
    return out

def corner_score(pts):
    if len(pts) < 6: return 0, 0
    angs = []
    for i in range(1, len(pts)-1):
        _, x0p = pts[i-1]; _, x1p = pts[i]; _, x2p = pts[i+1]
        v1 = np.array([x1p-x0p, 1], float); v2 = np.array([x2p-x1p, 1], float)
        n1 = np.linalg.norm(v1); n2 = np.linalg.norm(v2)
        if n1 < 1e-6 or n2 < 1e-6: continue
        c = np.clip(np.dot(v1, v2)/(n1*n2), -1, 1)
        angs.append(np.degrees(np.arccos(c)))
    return (max(angs) if angs else 0), (np.mean(angs) if angs else 0)

def fit_resid(pts):
    y = np.array([p[0] for p in pts], float); x = np.array([p[1] for p in pts], float)
    if len(y) < 8: return -1
    A = np.vstack([y*y, y, np.ones_like(y)]).T
    try:
        coef, _, _, _ = np.linalg.lstsq(A, x, rcond=None)
        r = np.sqrt(np.mean((x - A@coef)**2))
        return r / max(1e-6, x.max()-x.min())
    except Exception:
        return -1

sl = smooth(rows_l); sr = smooth(rows_r)
ml, al = corner_score(sl); mr, ar = corner_score(sr)
rl = fit_resid(sl); rr = fit_resid(sr)
print('--- 头骨轮廓(顶部14-30%, 中值平滑) ---')
print('左: 最大转角 %.1f° 平均 %.1f° 残差 %.4f  %s' % (ml, al, rl, '棱角!' if ml > 40 or rl > 0.035 else '平滑弧线'))
print('右: 最大转角 %.1f° 平均 %.1f° 残差 %.4f  %s' % (mr, ar, rr, '棱角!' if mr > 40 or rr > 0.035 else '平滑弧线'))

# ---- ② 脸颊断层: 在脸颊高度(头 45%~60%)扫描, 只测"非头发/非背景"像素的突变 ----
scan_y = y0 + int((y1-y0)*0.24)   # 额头稍下
scan_y2 = y0 + int((y1-y0)*0.32)  # 眼睛/颧骨行
for sy in (scan_y, scan_y2):
    f = fg[sy]
    if not f.any():
        print('y=%d no fg' % sy); continue
    xx = np.where(f)[0]
    fx0, fx1 = int(xx.min()), int(xx.max())
    row = img[sy, fx0:fx1+1]
    lum = row.mean(axis=1)
    d = np.sqrt(((row[1:] - row[:-1])**2).sum(axis=1))
    mx = d.max()
    # 排除两端(背景/头发边缘), 只测中间 30-70%
    n = len(d)
    seg = d[int(n*0.3):int(n*0.7)]
    segmx = seg.max()
    segjumps = np.where(seg > 80)[0]
    print('--- 扫描 y=%d x %d..%d: 全宽最大色差 %.1f | 中段(30-70%%)最大 %.1f, >80跳变 %d处 %s' % (
        sy, fx0, fx1, mx, segmx, len(segjumps), '断层!' if segmx > 110 else '连续'))
