# check_part_views.py — 验证生成的部位特写图: 轮廓完整性/居中/纯黑背景/尺寸
from PIL import Image
import numpy as np
import os

base = r'C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\parts'
for name in ['part_head_front.png', 'part_head_side.png']:
    fp = os.path.join(base, name)
    img = np.array(Image.open(fp).convert('RGBA')).astype(np.float32)
    H, W, _ = img.shape
    rgb = img[..., :3]
    # 纯黑背景判定: 背景应接近黑
    dark = np.all(rgb < 30, axis=2)
    dark_ratio = dark.mean()
    # 主体: 非黑像素
    fg = ~dark
    ys, xs = np.where(fg)
    if len(xs) == 0:
        print(name, 'NO fg!'); continue
    x0, x1, y0, y1 = xs.min(), xs.max(), ys.min(), ys.max()
    bw, bh = x1-x0+1, y1-y0+1
    print('%s: %dx%d, 黑底占比 %.2f, 主体bbox x %d..%d y %d..%d (w=%d h=%d), 宽高比 %.2f, 主体占画幅 %.0f%%' % (
        name, W, H, dark_ratio, x0, x1, y0, y1, bw, bh, bw/bh, (bw*bh)/(W*H)*100))
    # 主体居中度
    cx = (x0+x1)/2 / W
    print('  水平居中 x=%.2f(0.5=居中)  垂直范围 y=%.2f~%.2f' % (cx, y0/H, y1/H))
    # 每行宽度剖面(轮廓是否有锯齿/断裂): 行宽突变检查
    widths = []
    for y in range(y0, y1+1):
        f = fg[y]
        if f.any():
            xx = np.where(f)[0]
            widths.append(xx.max()-xx.min()+1)
    if widths:
        w = np.array(widths, float)
        print('  行宽: min %d max %d 均值 %.0f 突变(>40%%跳变)次数 %d' % (
            w.min(), w.max(), w.mean(), int((np.abs(np.diff(w))/ (w[:-1]+1) > 0.4).sum())))
