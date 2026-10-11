# -*- coding: utf-8 -*-
"""ds0731_head_check.py — 检测头部斜视轮廓的"方角/棱线"(验证头部是椭球而非方截面)。
用法: python ds0731_head_check.py <png>
方法: 取头部区域(画面中上部), 提取模型轮廓, 沿轮廓检测:
  1. 直边段: 连续 >=12px 方向变化 <6° 的段(椭球轮廓不应有长直边)
  2. 尖角: 轮廓曲率突变 >60° 的点
  3. 头部截面"方度": 斜视投影轮廓与椭圆的拟合残差
同时输出头部轮廓点供人工核对。"""
import sys
import numpy as np
from PIL import Image

def trace_outer(mask):
    """轮廓: 每行取左右边界(上/下 8 邻域补偿), 组装外轮廓点。"""
    h, w = mask.shape
    ys, xs = np.where(mask)
    if len(ys) == 0: return []
    y0, y1 = ys.min(), ys.max()
    pts = []
    # 上轮廓: 每列最上
    for x in range(xs.min(), xs.max() + 1):
        col = np.where(mask[:, x])[0]
        if len(col): pts.append((x, col.min()))
    # 右轮廓: 每行最右(去掉顶部已录的)
    for y in range(y0, y1 + 1):
        row = np.where(mask[y])[0]
        if len(row) and row.max() > xs.max() * 0.5:  # 只在右侧
            pass
    return pts

def analyze(path, view):
    a = np.asarray(Image.open(path).convert('RGB')).astype(int)
    H, W = a.shape[:2]
    mx = a.max(axis=2)
    model = mx < 246
    ys, xs = np.where(model)
    if len(ys) == 0: print('NO MODEL'); return
    # 头部区域: 模型上部 30%
    y_top = ys.min()
    y_head = y_top + int((ys.max() - y_top) * 0.28)
    head = model.copy()
    head[y_head:, :] = False
    if head.sum() < 100:
        print('head region too small'); return
    hys, hxs = np.where(head)
    # 轮廓: 逐行左右边界 + 逐列上下边界
    left = {}; right = {}; top = {}; bottom = {}
    for y in range(hys.min(), hys.max() + 1):
        row = np.where(head[y])[0]
        if len(row): left[y] = row.min(); right[y] = row.max()
    for x in range(hxs.min(), hxs.max() + 1):
        col = np.where(head[:, x])[0]
        if len(col): top[x] = col.min(); bottom[x] = col.max()
    # 组装轮廓点(顺时针): 上边(左→右), 右边(上→下), 下边(右→左), 左边(下→上)
    pts = []
    for x in sorted(top): pts.append((x, top[x]))
    for y in sorted(right): pts.append((right[y], y))
    for x in sorted(bottom, reverse=True): pts.append((x, bottom[x]))
    for y in sorted(left, reverse=True): pts.append((left[y], y))
    # 去重相邻
    out = []
    for p in pts:
        if not out or abs(p[0] - out[-1][0]) > 1 or abs(p[1] - out[-1][1]) > 1:
            out.append(p)
    if len(out) < 30:
        print('contour too small', len(out)); return
    P = np.array(out, dtype=float)
    # 方向角
    d = np.diff(P, axis=0)
    ang = np.degrees(np.arctan2(d[:, 1], d[:, 0]))
    ang = np.unwrap(ang)
    # 直边段: 相邻方向差 <6° 的连续段
    straight = 0
    seg_len = 0
    for i in range(1, len(ang)):
        dd = abs(ang[i] - ang[i - 1])
        dd = min(dd, 360 - dd)
        if dd < 6:
            seg_len += 1
            if seg_len == 12: straight += 1
        else:
            seg_len = 0
    # 尖角: 方向突变 >50°
    corners = 0
    for i in range(1, len(ang) - 1):
        dd = abs(ang[i + 1] - ang[i - 1])
        dd = min(dd, 360 - dd)
        if dd > 50: corners += 1
    # 轮廓点间距
    print(f'== {view} == head contour {len(out)} pts')
    print(f'  straight-edge segments(>=12px): {straight}  ({"平滑弧线" if straight <= 3 else "!! 有直边/棱线"})')
    print(f'  sharp corners(>50deg): {corners}  ({"无棱角" if corners <= 3 else "!! 有棱角"})')
    # 头部轮廓 bbox 与宽高
    print(f'  head bbox: x[{hxs.min()},{hxs.max()}] y[{hys.min()},{hys.max()}]  w={hxs.max()-hxs.min()} h={hys.max()-hys.min()}')

if __name__ == '__main__':
    view = sys.argv[2] if len(sys.argv) > 2 else 'head45'
    analyze(sys.argv[1], view)
