# check_oblique.py — 验证 45° 斜视图: 轮廓完整性 + 与正视/侧视的宽度剖面对比
from PIL import Image
import numpy as np
import os

base = r'C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy'
files = ['pc_zhengzha_front.png', 'pc_zhengzha_side.png', 'pc_zhengzha_back.png',
         'pc_zhengzha_oblique45.png', 'pc_zhengzha_oblique135.png']

data = {}
for name in files:
    fp = os.path.join(base, name)
    if not os.path.exists(fp):
        print(name, 'MISSING'); continue
    img = np.array(Image.open(fp).convert('RGBA')).astype(np.float32)
    H, W, _ = img.shape
    rgb = img[..., :3]
    dark = np.all(rgb < 30, axis=2)
    fg = ~dark
    ys, xs = np.where(fg)
    if len(xs) == 0:
        print(name, 'NO fg'); continue
    x0, x1, y0, y1 = xs.min(), xs.max(), ys.min(), ys.max()
    bw, bh = x1-x0+1, y1-y0+1
    # 行宽剖面(每 5% 高度取最大行宽)
    prof = []
    for pct in range(5, 100, 5):
        y = y0 + int(bh*pct/100)
        f = fg[y]
        if f.any():
            xx = np.where(f)[0]
            prof.append(xx.max()-xx.min()+1)
        else:
            prof.append(0)
    data[name] = (bw, bh, prof)
    print('%s: bbox w=%d h=%d 宽高比 %.2f 黑底%.2f' % (name, bw, bh, bw/bh, dark.mean()))
    print('   行宽剖面(5%%..95%%): %s' % prof)

# 头部宽度对比(剖面第 1 项 = 10% 高度)
print()
print('=== 头部宽度(10%%高度) vs 肩宽(25%%高度) vs 腿宽(75%%高度) 对比 ===')
for name, (bw, bh, prof) in data.items():
    if len(prof) >= 15:
        head = prof[1]   # 10%
        shoulder = prof[4]  # 25%
        leg = prof[14]   # 75%
        print('%s: 头 %d / 肩 %d / 腿 %d  (头:肩 = %.2f)' % (name, head, shoulder, leg, head/max(1, shoulder)))
