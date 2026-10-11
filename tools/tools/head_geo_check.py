# head_geo_check.py — DS0731 头部椭球化客观像素检测(不受模型偏见影响)
# 检测: ①头部轮廓平滑度(有无直线段/尖角) ②脸部水平扫描颜色连续性(有无断层/黑线)
from PIL import Image
import numpy as np

base = r'C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots'
name = 'render_sprite_ds0731_zhenghai_head45'

img = np.array(Image.open(base + '\\' + name + '.png').convert('RGB')).astype(np.float32)
H, W, _ = img.shape
print('image', img.shape)

# 背景 = 白(或接近白); 前景 = 非白
bg = np.all(img > 235, axis=2)
fg = ~bg
ys, xs = np.where(fg)
if len(xs) == 0:
    print('NO foreground'); raise SystemExit
x0, x1, y0, y1 = xs.min(), xs.max(), ys.min(), ys.max()
print('fg bbox x %d..%d y %d..%d  (w=%d h=%d)' % (x0, x1, y0, y1, x1-x0+1, y1-y0+1))

# ---- ① 头部轮廓平滑度: 取 bbox 上 25% 区域(头部) 逐行左右轮廓点, 检查拐角 ----
head_y1 = y0 + int((y1-y0)*0.28)  # 头部区域: 顶部 28%
rows_l, rows_r = [], []
for y in range(y0, head_y1):
    f = fg[y]
    if f.any():
        xx = np.where(f)[0]
        rows_l.append((y, int(xx.min())))
        rows_r.append((y, int(xx.max())))

def corner_score(pts):
    """相邻三点夹角尖锐度: 返回最大转角(度) 和 平均转角"""
    if len(pts) < 5: return 0, 0
    angs = []
    for i in range(1, len(pts)-1):
        y0p, x0p = pts[i-1]; y1p, x1p = pts[i]; y2p, x2p = pts[i+1]
        v1 = np.array([x1p-x0p, y1p-y0p], float); v2 = np.array([x2p-x1p, y2p-y1p], float)
        n1 = np.linalg.norm(v1); n2 = np.linalg.norm(v2)
        if n1 < 1e-6 or n2 < 1e-6: continue
        c = np.clip(np.dot(v1, v2)/(n1*n2), -1, 1)
        angs.append(np.degrees(np.arccos(c)))
    return (max(angs) if angs else 0), (np.mean(angs) if angs else 0)

# 平滑度: 拟合二次曲线看残差(椭球弧线应贴近抛物线/圆)
def fit_resid(pts, xcol, ycol):
    x = np.array([p[xcol] for p in pts], float); y = np.array([p[ycol] for p in pts], float)
    A = np.vstack([y*y, y, np.ones_like(y)]).T
    if A.shape[0] < 6: return -1
    try:
        coef, _, _, _ = np.linalg.lstsq(A, x, rcond=None)
        pred = A @ coef
        r = np.sqrt(np.mean((x-pred)**2))
        return r / max(1e-6, (x.max()-x.min()))
    except Exception:
        return -1

ml, al = corner_score(rows_l); mr, ar = corner_score(rows_r)
rl = fit_resid(rows_l, 1, 0); rr = fit_resid(rows_r, 1, 0)
print('--- 头部轮廓(顶部28%区) ---')
print('左轮廓: 最大转角 %.1f° 平均转角 %.1f° 二次拟合残差 %.4f' % (ml, al, rl))
print('右轮廓: 最大转角 %.1f° 平均转角 %.1f° 二次拟合残差 %.4f' % (mr, ar, rr))
print('判断: 转角 >25° 或 残差 >0.03 => 有棱角/非平滑弧线')
print('  左轮廓 %s | 右轮廓 %s' % ('棱角!' if ml > 25 or rl > 0.03 else '平滑弧线', '棱角!' if mr > 25 or rr > 0.03 else '平滑弧线'))

# ---- ② 脸部水平扫描颜色连续性: 头部中部(脸)一行, 找最大跳变 ----
mid_y = y0 + int((head_y1 - y0) * 0.55)  # 头部上部 55% 处(眼睛/脸颊行)
f = fg[mid_y]
if f.any():
    xx = np.where(f)[0]
    fx0, fx1 = int(xx.min()), int(xx.max())
    row = img[mid_y, fx0:fx1+1]
    # 相邻像素颜色差(欧氏距离)
    d = np.sqrt(((row[1:] - row[:-1])**2).sum(axis=1))
    mx = d.max()
    mx_i = int(np.argmax(d))
    # 大跳变阈值: 颜色差 > 60 (255 尺度)
    jumps = np.where(d > 60)[0]
    print('--- 脸部水平扫描(y=%d, x %d..%d) ---' % (mid_y, fx0, fx1))
    print('最大相邻色差 %.1f @x=%d(相对位置 %.2f)' % (mx, fx0+mx_i, (fx0+mx_i-fx0)/(fx1-fx0)))
    print('跳变>60 位置数: %d' % len(jumps))
    if len(jumps):
        print('  跳变位置(相对):', [round((fx0+j-fx0)/(fx1-fx0), 2) for j in jumps[:12]])
    print('判断: 若脸部中部(0.2~0.8)存在 >60 跳变 => 有断层/黑线')
    mid_jumps = [j for j in jumps if 0.2 < (fx0+j-fx0)/(fx1-fx0) < 0.8]
    print('  脸部中部跳变: %s' % ('有断层!' if mid_jumps else '无断层(连续)'))
else:
    print('mid row no fg')
