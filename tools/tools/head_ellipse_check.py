# head_ellipse_check.py — 头部轮廓椭圆拟合验收(DS0731 头部椭球化硬性标准)
# 用法: python head_ellipse_check.py [head45|head135]  (默认 head45)
# 输出: 椭圆拟合归一化残差 + 最大转角。要求: 残差<0.02, 转角<30°(排除发尖后)
from PIL import Image
import numpy as np
import sys

base = r'C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots'
view = sys.argv[1] if len(sys.argv) > 1 else 'head45'
name = 'render_sprite_ds0731_zhenghai_' + view

img = np.array(Image.open(base + '\\' + name + '.png').convert('RGB')).astype(np.float32)
H, W, _ = img.shape
bg = np.all(img > 235, axis=2)
fg = ~bg
ys, xs = np.where(fg)
if len(xs) == 0:
    print('NO fg'); raise SystemExit
x0, x1, y0, y1 = xs.min(), xs.max(), ys.min(), ys.max()
print('%s fg bbox x %d..%d y %d..%d (w=%d h=%d)' % (name, x0, x1, y0, y1, x1-x0+1, y1-y0+1))

# 头部区: 顶部 12%~32% (避开纯发尖和颈部)
ta = y0 + int((y1-y0)*0.12)
tb = y0 + int((y1-y0)*0.32)
pts = []
for y in range(ta, tb):
    f = fg[y]
    if f.any():
        xx = np.where(f)[0]
        pts.append((y, int(xx.min())))
        pts.append((y, int(xx.max())))

if len(pts) < 20:
    print('too few points'); raise SystemExit
P = np.array(pts, float)
yc = P[:,0].mean(); xc = P[:,1].mean()

# 椭圆拟合(最小二乘): Ax^2 + B y^2 + C x y + D x + E y + F = 0 (令 F=1)
X, Y = P[:,1]-xc, P[:,0]-yc
A = np.vstack([X*X, Y*Y, X*Y, X, Y]).T
b = -np.ones(len(X))
coef, _, _, _ = np.linalg.lstsq(A, b, rcond=None)
a2_, b2_, c_, d_, e_ = coef
# 椭圆判别: B>0, 且二次型正定近似
# 计算每点到拟合椭圆的代数残差(归一化)
res = np.abs(A @ coef + 1)
rel = res / max(1e-9, np.abs(b).max())
rms = np.sqrt(np.mean(rel**2))

# 转角: 相邻三点角度(经 3 点中值平滑)
def smooth_pts(pl):
    arr = np.array(pl)
    out = []
    for i in range(len(arr)):
        lo = max(0, i-2); hi = min(len(arr), i+3)
        out.append((arr[i,0], np.median(arr[lo:hi,1])))
    return out

def corner(pl):
    s = smooth_pts(pl)
    angs = []
    for i in range(1, len(s)-1):
        v1 = np.array([s[i][1]-s[i-1][1], s[i][0]-s[i-1][0]], float)
        v2 = np.array([s[i+1][1]-s[i][1], s[i+1][0]-s[i][0]], float)
        n1 = np.linalg.norm(v1); n2 = np.linalg.norm(v2)
        if n1 < 1e-9 or n2 < 1e-9: continue
        c = np.clip(np.dot(v1,v2)/(n1*n2), -1, 1)
        angs.append(np.degrees(np.arccos(c)))
    return (max(angs) if angs else 0), (np.mean(angs) if angs else 0)

# 左右轮廓分开算转角
left = [(y, x) for y, x in pts if x == min(px for yy, px in pts if yy == y)] if False else None
byrow = {}
for y, x in pts:
    byrow.setdefault(y, []).append(x)
lrows = sorted((y, min(v)) for y, v in byrow.items())
rrows = sorted((y, max(v)) for y, v in byrow.items())
ml, al = corner(lrows)
mr, ar = corner(rrows)

ok_r = rms < 0.02
ok_l = ml < 30 and mr < 30
print('--- 椭圆拟合(排除发尖区) ---')
print('椭圆残差(归一化 RMS): %.4f   %s' % (rms, 'PASS(椭球弧线)' if ok_r else 'FAIL(非椭球!)'))
print('左轮廓最大转角 %.1f° / 右轮廓 %.1f°   %s' % (ml, mr, 'PASS' if ok_l else 'FAIL(有棱角)'))
print('验收: %s' % ('PASS 头部已是平滑椭球' if (ok_r and ok_l) else 'FAIL 头部仍非椭球'))
print('拟合参数: a2=%.4f b2=%.4f c_xy=%.4f' % (a2_, b2_, c_))
