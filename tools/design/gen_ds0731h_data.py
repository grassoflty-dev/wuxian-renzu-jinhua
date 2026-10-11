# -*- coding: utf-8 -*-
"""gen_ds0731h_data.py — 解析 MakeHuman base.obj + maxmuscle target, 生成:
1. server-rs/ui/js/sprite_ds0731h_data.js(内联顶点/面/target 位移数组, CC0)
2. 打印底模 y 轴结构(头/颈/肩/腰/胯/膝分界)供体型标定
"""
import os, sys

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\model_task_pkg\human_ref\makehuman_base.obj"
TGT = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\model_task_pkg\human_ref\makehuman_targets\universal-male-young-maxmuscle-averageweight.target"
OUT = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\server-rs\ui\js\sprite_ds0731h_data.js"

# ---- 解析 OBJ ----
v = []
f = []
y_hist = {}  # 每 0.5 单位的顶点数(y 分布)
for line in open(BASE, encoding='utf-8', errors='replace'):
    if line.startswith('v '):
        p = line.split()
        x, y, z = float(p[1]), float(p[2]), float(p[3])
        v.append((x, y, z))
        key = round(y * 2) / 2
        y_hist[key] = y_hist.get(key, 0) + 1
    elif line.startswith('f '):
        parts = line.split()[1:]
        idx = []
        for pt in parts:
            iv = int(pt.split('/')[0]) - 1
            idx.append(iv)
        f.append(idx)

# ---- 解析 target ----
t = []
for line in open(TGT, encoding='utf-8', errors='replace'):
    line = line.strip()
    if not line or line.startswith('#'):
        continue
    p = line.split()
    t.append((int(p[0]), float(p[1]), float(p[2]), float(p[3])))

print('v:', len(v), 'f:', len(f), 'target:', len(t))

# ---- y 轴结构探测(0.5 单位直方图) ----
print('\n=== y histogram (per 0.5) ===')
ys_sorted = sorted(y_hist.keys())
for key in ys_sorted:
    if key < -9 or key > 9:
        continue
    bar = '#' * min(60, y_hist[key] // 40)
    print('%6.1f | %6d %s' % (key, y_hist[key], bar))

# ---- 输出 JS(紧凑数字数组) ----
def fmt_nums(nums, per_line=12, prec=4):
    out = []
    line = []
    for n in nums:
        line.append(str(n))
        if len(line) >= per_line:
            out.append('  ' + ', '.join(line) + ',')
            line = []
    if line:
        out.append('  ' + ', '.join(line) + ',')
    return '\n'.join(out)

with open(OUT, 'w', encoding='utf-8') as fo:
    fo.write('/* sprite_ds0731h_data.js — MakeHuman base.obj(CC0) + maxmuscle target(CC0) 内联数据.\n')
    fo.write(' * 来源: makehumancommunity/makehuman, CC0 1.0 Universal(文件头双声明)。\n')
    fo.write(' * v: 19158*3 float(y-up, 高 16.95); f: 18486*4 int(四边形, 1-based→0-based);\n')
    fo.write(' * target: 6843*4 int/float(顶点索引, dx, dy, dz 位移)。*/\n')
    fo.write('window.DS0731H_DATA = {\n')
    fo.write('  v: [\n')
    flat_v = [round(c, 4) for p in v for c in p]
    fo.write(fmt_nums(flat_v) + '\n')
    fo.write('  ],\n  f: [\n')
    flat_f = [i for quad in f for i in quad]
    fo.write(fmt_nums(flat_f) + '\n')
    fo.write('  ],\n  target: [\n')
    flat_t = [c for row in t for c in row]
    fo.write(fmt_nums(flat_t, prec=3) + '\n')
    fo.write('  ]\n};\n')

print('\nwrote', OUT, os.path.getsize(OUT), 'B')
