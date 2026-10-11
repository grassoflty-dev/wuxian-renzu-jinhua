# -*- coding: utf-8 -*-
"""make_pc_legs.py — 从侧视立绘 pc_zhengzha_side.png 抠出「下半身腿部」贴图。

思路（方案A：复用立绘自身像素，画风天然一致）：
  1. 立绘描边是白色 rim light（亮度 >140），逐行取左右 rim 的 x 范围 →
     得到下半身真实轮廓（直接按 alpha 取会把 10~30px 的柔光晕一起算进去，导致贴图错位）。
  2. 沿「腿后缘」(rimL) 对齐并做线性拉直（腿在源图里略向后倾），
     保留靴尖向前自然前伸。按行裁掉 rim 之外的像素 → 顺带剔除远侧腿的柔光/残影。
  3. 输出 art px 尺寸 17 x 48（与上身 64x104 同尺度），4 倍超采样，交给前端
     makePortrait() 走与上身完全相同的像素管线（提亮/对比/锐化）。
"""
import sys, os
import numpy as np
from PIL import Image

SRC = sys.argv[1]
DST = sys.argv[2]

HIP, SOLE = 553, 1008          # 髋（= 立绘顶部 54% 处）与鞋底所在源行
PAD = 4                        # 轮廓外额外保留的源像素
LEG_W_ART, LEG_H_ART = 17, 48  # 目标 art px 尺寸
SS = 4                         # 超采样倍数

im = Image.open(SRC).convert("RGBA")
a = np.array(im)
al = a[:, :, 3].astype(np.float32)
br = a[:, :, :3].astype(np.float32).mean(axis=2)
rim = (al > 128) & (br > 140)          # 白描边（rim light）

H, W = al.shape

def rim_edges(y):
    xs = np.nonzero(rim[y])[0]
    if len(xs) == 0:
        return None
    return int(xs[0]), int(xs[-1])

left, right = {}, {}
for y in range(HIP, SOLE + 1):
    e = rim_edges(y)
    if e:
        left[y] = e[0]
        right[y] = e[1]

# 缺口用邻近行线性插值补齐
def fill(d):
    ys = sorted(d)
    for y in range(HIP, SOLE + 1):
        if y in d:
            continue
        lo = max([t for t in ys if t < y], default=None)
        hi = min([t for t in ys if t > y], default=None)
        if lo is None:
            d[y] = d[hi]
        elif hi is None:
            d[y] = d[lo]
        else:
            t = (y - lo) / (hi - lo)
            d[y] = d[lo] + (d[hi] - d[lo]) * t
    return d

fill(left); fill(right)

ys = np.arange(HIP, SOLE + 1, dtype=np.float64)
L = np.array([left[int(y)] for y in ys])
R = np.array([right[int(y)] for y in ys])
print("rim 左缘 min/max %.0f/%.0f  右缘 min/max %.0f/%.0f  最大宽度 %d" % (L.min(), L.max(), R.min(), R.max(), int((R - L).max())))

# 沿左缘线性拉直（去掉腿的整体后倾，保留靴尖前伸）
k, b = np.polyfit(ys, L, 1)
print("左缘拟合斜率 %.5f (每行 -%.3f px)" % (k, -k))

SCALE_ART = (SOLE - HIP) / float(LEG_H_ART)     # 源像素 / art px
STRIPW = int(round(LEG_W_ART * SCALE_ART))      # 条带源宽度
print("源条带宽度 %d px  →  art %.2f x %d" % (STRIPW, STRIPW / SCALE_ART, LEG_H_ART))

need = int(np.ceil((R - L).max())) + 2 * PAD
print("包含靴尖所需最小条带宽度 %d px（当前 %d）  %s" % (need, STRIPW, "OK" if need <= STRIPW else "!! 不足"))

rows = []
for y in range(HIP, SOLE + 1):
    Lr = k * y + b                       # 拉直后的左缘基准
    x0 = Lr - PAD                        # 贴图左缘 = 腿后缘 - PAD
    row = np.zeros((STRIPW, 4), dtype=np.uint8)
    lo, hi = left[y] - PAD, right[y] + PAD
    for i in range(STRIPW):
        xx = int(round(x0 + i))
        if lo <= xx <= hi and 0 <= xx < W:
            row[i] = a[y, xx]
    rows.append(row)

arr = np.stack(rows)                     # (455, STRIPW, 4)
img = Image.fromarray(arr, "RGBA")
# 预乘 alpha 再缩放，避免透明像素(0,0,0,0)在 LANCZOS 下把边缘染黑
f = arr.astype(np.float32)
alw = f[:, :, 3:4] / 255.0
pre = np.concatenate([f[:, :, :3] * alw, f[:, :, 3:4]], axis=2).astype(np.uint8)
pres = np.array(Image.fromarray(pre, "RGBA").resize((LEG_W_ART * SS, LEG_H_ART * SS), Image.LANCZOS)).astype(np.float32)
pw = np.clip(pres[:, :, 3:4], 1, 255) / 255.0
unf = np.concatenate([np.clip(pres[:, :, :3] / pw, 0, 255), pres[:, :, 3:4]], axis=2)
# 明度对齐：源立绘的裤子本身就比上衣暗（管线后 mean 96 vs 118、中位 76 vs 104），
# 这里给腿部贴图一个整体提亮 GAIN_S/GAIN_O，使其与上身处于同一明度族（不改色相/描边/色块结构）。
# 前端 makePortrait 的线性处理 (+22 / 对比 1.22) 会把这组线性变换原样传递，故直接在源图上补偿。
GAIN_S, GAIN_O = 1.14, 7.0
op = unf[:, :, 3:4] > 0
unf[:, :, :3] = np.where(op, np.clip(unf[:, :, :3] * GAIN_S + GAIN_O, 0, 255), unf[:, :, :3])
# 注意：不要对腿部高光做限幅。立绘的白色 rim light 是"描边"来源——前端 makePortrait 会把
# 「近白且紧邻透明」的像素压成深色描边 (14,17,22,26)，上身靠这一步得到像素风深色轮廓。
# 若把 rim 限幅到 225 以下，这一步就不触发，腿部会只剩亮边、丢失深色描边，反而更像"贴纸"。
# 因此这里只做明度对齐，保留 rim 原样。
out = Image.fromarray(unf.astype(np.uint8), "RGBA")
os.makedirs(os.path.dirname(DST), exist_ok=True)
out.save(DST)
print("saved", DST, out.size)

# —— 供 zone2d.js 使用的常量（身体本地 art 坐标：画布 64 宽，中心为 0）——
ART_S = 104.0 / 1024.0                   # 立绘 → art px 全局比例
CROP_OFF = (768 - 64 / ART_S) / 2.0      # 立绘水平居中裁切偏移
hip_back_src = left[HIP] - PAD
hip_back_local = (hip_back_src - CROP_OFF) * ART_S - 32
thigh_front_src = right[603] + PAD
thigh_front_local = (thigh_front_src - CROP_OFF) * ART_S - 32
pivot = (thigh_front_local - hip_back_local) / 2.0
print("LEG_W_ART=%d LEG_H_ART=%d" % (LEG_W_ART, LEG_H_ART))
print("贴图左缘(腿后缘) 身体本地 x = %.2f" % hip_back_local)
print("大腿前缘     身体本地 x = %.2f" % thigh_front_local)
print("建议 LEG_PIVOT（贴图内髋关节 x, art px） = %.2f" % pivot)
print("建议 近腿铰点 hxNear = %.2f   远腿 hxFar = %.2f" % (hip_back_local + pivot, hip_back_local + pivot - 4.6))
print("膝/踝 建议行: 膝 %.1f  踝 %.1f" % ((728 - HIP) / SCALE_ART, (903 - HIP) / SCALE_ART))
