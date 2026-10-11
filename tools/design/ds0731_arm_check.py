# -*- coding: utf-8 -*-
"""ds0731_arm_check.py — 斜视/正面图中手臂独立的量化检查。
用法: python ds0731_arm_check.py <png> <view>
  view=front: 在躯干区检测 [左臂, 躯干, 右臂] runs, 量手臂宽度
  view=upper/side: 45° 斜视, 检测上臂高度处 runs 结构:
    若 [臂, 隙, 躯干, 隙, 臂] → 手臂独立 ✓; 若手臂与躯干粘连 → 手臂吞身体 ✗
输出: 手臂屏幕宽度 + 与躯干宽度的比值(圆柱手臂 ≈ 正面投影宽度)。"""
import sys
import numpy as np
from PIL import Image

def segs_of_row(row):
    segs = []
    prev = False
    for x, v in enumerate(row):
        if v and not prev:
            segs.append([x, x])
        if v:
            segs[-1][1] = x
        prev = v
    return segs

def main():
    f, view = sys.argv[1], sys.argv[2]
    a = np.asarray(Image.open(f).convert('RGB')).astype(int)
    H, W = a.shape[:2]
    mx = a.max(axis=2)
    model = mx < 246
    ys, xs = np.where(model)
    y0, y1 = ys.min(), ys.max()
    h = y1 - y0 + 1
    print(f'{view}: model bbox y[{y0},{y1}] x[{xs.min()},{xs.max()}]')
    if view == 'front':
        # 上臂区 v=0.35-0.45
        for frac in [0.35, 0.40, 0.45]:
            y = y0 + int(h * frac)
            segs = segs_of_row(model[y])
            print(f'  v={frac}: segs={[(s[1]-s[0]+1, s[0], s[1]) for s in segs]}')
        # 平均手臂/躯干宽度
        armw = []; torw = []
        for frac in np.arange(0.34, 0.50, 0.01):
            y = y0 + int(h * frac)
            segs = segs_of_row(model[y])
            if len(segs) == 3:
                armw.append(segs[0][1] - segs[0][0] + 1)
                armw.append(segs[2][1] - segs[2][0] + 1)
                torw.append(segs[1][1] - segs[1][0] + 1)
        if armw:
            print(f'  front arm width avg {np.mean(armw):.1f}px, torso {np.mean(torw):.1f}px, arm/torso = {np.mean(armw)/np.mean(torw):.3f}')
    else:
        # 斜视: 上臂区(v=0.25-0.45, 头肩下方到胸), 统计每行 segs 数
        frag = 0; five = 0; three = 0; two = 0; one = 0
        armw = []; torw = []
        for frac in np.arange(0.26, 0.48, 0.01):
            y = y0 + int(h * frac)
            segs = segs_of_row(model[y])
            n = len(segs)
            if n >= 5: five += 1; armw.append(min(s[1]-s[0]+1 for s in segs[:2])); torw.append(segs[2][1]-segs[2][0]+1)
            elif n == 3: three += 1
            elif n == 2: two += 1
            elif n == 1: one += 1
            else: frag += 1
        total = five + three + two + one + frag
        print(f'  rows: 5seg(臂独立)={five}, 3seg={three}, 2seg={two}, 1seg(粘连)={one}, >5={frag}  (of {total})')
        if armw:
            print(f'  oblique arm width avg {np.mean(armw):.1f}px, torso {np.mean(torw):.1f}px, arm/torso = {np.mean(armw)/np.mean(torw):.3f}')
        # 抽样几行展示
        for frac in [0.30, 0.36, 0.42]:
            y = y0 + int(h * frac)
            segs = segs_of_row(model[y])
            print(f'  v={frac}: n={len(segs)} segs={[(s[1]-s[0]+1) for s in segs]}')

if __name__ == '__main__':
    main()
