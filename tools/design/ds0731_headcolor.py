# -*- coding: utf-8 -*-
"""ds0731_headcolor.py — 检测头部斜视图的脸颊颜色连续性(找拼接断层/棱线)。
用法: python ds0731_headcolor.py <png> [y_frac]
在指定高度(默认头部中部)水平扫描模型区, 找相邻像素色差跳变点(>35)。
平滑椭球贴图应无跳变; 面拼接会有跳变。"""
import sys
import numpy as np
from PIL import Image

def main():
    f = sys.argv[1]
    y_frac = float(sys.argv[2]) if len(sys.argv) > 2 else 0.5
    a = np.asarray(Image.open(f).convert('RGB')).astype(int)
    H, W = a.shape[:2]
    mx = a.max(axis=2)
    model = mx < 246
    ys, xs = np.where(model)
    if len(ys) == 0:
        print('NO MODEL'); return
    y0, y1 = ys.min(), ys.max()
    y = y0 + int((y1 - y0) * y_frac)
    row = model[y]
    cols = np.where(row)[0]
    if len(cols) < 10:
        print('row empty'); return
    xl, xr = cols.min(), cols.max()
    rgb = a[y, xl:xr + 1]
    # 相邻像素色差
    diff = np.abs(np.diff(rgb.astype(float), axis=0)).max(axis=1)
    jumps = np.where(diff > 35)[0]
    print(f'{f.split(chr(92))[-1].split("/")[-1]} y={y} (frac {y_frac}): row x[{xl},{xr}] len={len(rgb)}')
    print(f'  color jumps(>35): {len(jumps)}  ({"平滑连续" if len(jumps) <= 3 else "!! 有断层/棱线"})')
    if len(jumps):
        # 显示跳变位置(相对头部中心)
        cx = (xl + xr) // 2
        for j in jumps[:20]:
            print(f'    jump at x={xl + j} (rel {xl + j - cx:+d}px) diff={diff[j]:.0f}')
    # 最大色差
    print(f'  max adjacent diff: {diff.max():.0f}')

if __name__ == '__main__':
    main()
