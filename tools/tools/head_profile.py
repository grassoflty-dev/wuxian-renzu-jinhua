# head_profile.py — 打印立绘顶部宽度剖面(每 6 行), 目视定位头/颈/肩
from PIL import Image
import numpy as np

base = r'C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\server-rs\ui\assets\img'
fr = np.array(Image.open(base + r'\pc_zhengzha.png').convert('RGBA'))
sd = np.array(Image.open(base + r'\pc_zhengzha_side.png').convert('RGBA'))

def bbox(a):
    al = a[..., 3] > 15
    ys, xs = np.where(al)
    return xs.min(), xs.max(), ys.min(), ys.max()

def profile(a, bx0, bx1, by0, by1, step=6, maxrows=180):
    print(' y_abs | y%  | width | x-range')
    for y in range(by0, min(by1, by0 + maxrows), step):
        al = a[y, bx0:bx1 + 1, 3] > 15
        if al.any():
            xs = np.where(al)[0] + bx0
            w = xs.max() - xs.min() + 1
            pct = (y - by0) / (by1 - by0 + 1) * 100
            print('%5d | %4.1f | %5d | %d..%d' % (y, pct, w, xs.min(), xs.max()))
        else:
            print('%5d | %4.1f |     0 |' % (y, (y - by0) / (by1 - by0 + 1) * 100))

fx0, fx1, fy0, fy1 = bbox(fr)
sx0, sx1, sy0, sy1 = bbox(sd)
print('=== FRONT top profile (bbox y %d..%d) ===' % (fy0, fy1))
profile(fr, fx0, fx1, fy0, fy1, step=4, maxrows=140)
print()
print('=== SIDE top profile (bbox y %d..%d) ===' % (sy0, sy1))
profile(sd, sx0, sx1, sy0, sy1, step=4, maxrows=140)
