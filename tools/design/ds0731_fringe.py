# -*- coding: utf-8 -*-
"""ds0731_fringe.py — 检测渲染图中模型轮廓内侧的白边/毛边(接缝伪影)。
用法: python ds0731_fringe.py <png>
对模型 mask(非白)的每条轮廓边界, 向内扫描 1..6px, 统计近白像素
(min channel > 235)的占比 —— 高占比 = 白边/毛边明显。"""
import sys
from PIL import Image
import numpy as np

def main():
    f = sys.argv[1]
    a = np.asarray(Image.open(f).convert('RGB')).astype(int)
    H, W = a.shape[:2]
    mx = a.max(axis=2)
    model = mx < 246
    if model.sum() == 0:
        print('NO MODEL'); return
    # 模型内缩 1..6 层的近白像素
    inner = model.copy()
    for ring in range(1, 7):
        # 当前边界 = inner 的 4 邻域外是背景/或模型外
        boundary = inner & ~(
            np.pad(inner[1:, :], ((0, 1), (0, 0)), constant_values=0)[:, :] &
            np.pad(inner[:-1, :], ((1, 0), (0, 0)), constant_values=0)[:, :] &
            np.pad(inner[:, 1:], ((0, 0), (0, 1)), constant_values=0)[:, :] &
            np.pad(inner[:, :-1], ((0, 0), (1, 0)), constant_values=0)[:, :]
        )
        # 4 邻域腐蚀: 去掉边界层
        eroded = inner.copy()
        eroded[1:, :] &= inner[:-1, :]; eroded[:-1, :] &= inner[1:, :]
        eroded[:, 1:] &= inner[:, :-1]; eroded[:, :-1] &= inner[:, 1:]
        if boundary.sum() == 0:
            break
        mn = a[:, :, :3].min(axis=2)
        near_white = mn > 235
        frac = float(near_white[boundary].mean())
        inner = eroded
        print(f'  ring {ring}: {int(boundary.sum())}px, near-white {frac * 100:.1f}%')

if __name__ == '__main__':
    main()
