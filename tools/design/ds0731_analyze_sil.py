# -*- coding: utf-8 -*-
"""ds0731_analyze_sil.py — 分析正/侧/背立绘的逐行剪影结构, 供 DS0731 建模器设计。
用法: python ds0731_analyze_sil.py <front|side|back>
输出: 每 4% 高度一行的 runs / zrange / bbox。
"""
import sys
from PIL import Image
import numpy as np

def runs_of_row(a, y, thresh=127, gap=2):
    W = a.shape[1]
    al = a[y, :, 3]
    on = al > thresh
    out = []
    l = -1; g = 0
    for x in range(W):
        if on[x]:
            if l < 0: l = x
            g = 0
        elif l >= 0:
            g += 1
            if g > gap:
                if x - g - l >= 3: out.append((l, x - g - 1))
                l = -1; g = 0
    if l >= 0 and W - l >= 3: out.append((l, W - 1))
    return out

def main():
    which = sys.argv[1] if len(sys.argv) > 1 else 'front'
    base = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\server-rs\ui\assets\img"
    fn = {'front': 'pc_zhengzha.png', 'side': 'pc_zhengzha_side.png', 'back': 'pc_zhengzha_back.png'}[which]
    im = Image.open(base + '\\' + fn).convert('RGBA')
    a = np.asarray(im)
    H, W = a.shape[:2]
    print(f'{fn}: {W}x{H}')
    # bbox alpha>15
    al = a[:, :, 3]
    ys, xs = np.where(al > 15)
    bb = (xs.min(), ys.min(), xs.max(), ys.max())
    print('bbox alpha>15:', bb)
    # 采样行结构
    for frac in range(0, 101, 4):
        y = int(bb[1] + (bb[3] - bb[1]) * frac / 100)
        if which == 'front':
            rs = runs_of_row(a, y)
            print(f'v={frac:3d}% y={y}: runs={rs if rs else None}')
        else:
            on = a[y, :, 3] > 127
            xs2 = np.where(on)[0]
            rng = (int(xs2.min()), int(xs2.max())) if xs2.size else None
            # 是否多段
            rs = runs_of_row(a, y)
            print(f'v={frac:3d}% y={y}: zrange={rng} runs={len(rs)}')

if __name__ == '__main__':
    main()
