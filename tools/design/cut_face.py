# -*- coding: utf-8 -*-
"""cut_face.py — 从郑吒半身立绘 pc_zhengzha.png 抠出「头+脸」区域，
输出透明背景 head/face sprite（RGBA，透明像素 RGB=0，头顶/下巴完整）。

裁剪策略（针对此立绘）：
  * 头顶 = 第一个有内容行（略留白）。
  * 头宽 = 「头顶..头顶+140px」带内最大内容宽度（头冠+脸颊，不含肩）。
  * 肩线 = 向下第一个宽度 >= 1.5×头宽 的行（进入肩膀）。
  * 下边框 = 肩线下方一点点（含一点肩颈衔接）。
  * 横向以头/颈中心对齐，宽按目标 3:4。

用法: uv run --with pillow --with numpy python cut_face.py
输出: ../tools/shots/zhengha_face_sprite.png
"""
import os
import numpy as np
from PIL import Image

BASE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.abspath(os.path.join(BASE, "..", "..", "server-rs", "ui", "assets", "img", "pc_zhengzha.png"))
OUT = os.path.abspath(os.path.join(BASE, "..", "shots", "zhengha_face_sprite.png"))

TARGET_AR = 0.75  # w/h 目标 ≈3:4

img = Image.open(SRC).convert("RGBA")
a = np.asarray(img).astype(np.int32)
alpha = a[..., 3]
sub = alpha > 5
h, w = alpha.shape
rows = sub.sum(axis=1)

ys, xs = np.where(sub)
top = int(ys.min())

# 头宽：头顶..头顶+140 带
head_band_end = min(int(ys.max()), top + 140)
head_w = 0
for y in range(top, head_band_end):
    if rows[y] > 0:
        e = np.where(sub[y])[0]
        head_w = max(head_w, e.max() - e.min() + 1)

# 肩线：向下第一个宽度 >= 1.5*头宽
shoulder_row = None
for y in range(top, int(ys.max())):
    if rows[y] > 3:
        e = np.where(sub[y])[0]
        if e.max() - e.min() + 1 >= 1.5 * head_w:
            shoulder_row = y
            break
if shoulder_row is None:
    shoulder_row = int(ys.max())

y0 = max(0, top - 3)                       # 头顶略留白
y1 = min(int(ys.max()), shoulder_row + 6)  # 下到肩线下方一点（含肩颈衔接）

# 横向中心：y0..y1 带内容中心的中位
centers = []
for y in range(y0, y1 + 1):
    if rows[y] > 0:
        e = np.where(sub[y])[0]
        centers.append((e.min() + e.max()) / 2.0)
cx = int(round(float(np.median(centers))))

crop_h = y1 - y0 + 1
crop_w = int(round(crop_h * TARGET_AR))
crop_w = max(crop_w, int(round(head_w * 1.15)))
if crop_w % 2 == 1:
    crop_w += 1

x0 = cx - crop_w // 2
x1 = x0 + crop_w - 1
if x0 < 0:
    x1 -= x0; x0 = 0
if x1 >= w:
    x1 = w - 1; x0 = x1 - crop_w + 1

crop = a[y0:y1 + 1, x0:x1 + 1].copy()
al = crop[..., 3]
tr = al <= 5
if tr.any():
    crop[tr, 0] = 0; crop[tr, 1] = 0; crop[tr, 2] = 0
out_img = Image.fromarray(crop.astype(np.uint8), "RGBA")

dat = np.asarray(out_img)
dal = dat[..., 3]
drgb = dat[..., :3]
opaque = dal >= 250
opx, opy = np.where(opaque)
report = {
    "out": OUT,
    "crop_box": (x0, y0, x1, y1),
    "out_size": (crop.shape[1], crop.shape[0]),
    "opaque_pct": round(float(opaque.mean() * 100), 2),
    "semi_pct": round(float(((dal > 5) & (dal < 250)).mean() * 100), 2),
    "trans_pct": round(float((dal <= 5).mean() * 100), 2),
    "trans_rgb_all_zero": bool((drgb[dal <= 5] == 0).all()),
    "opaque_content_bbox": (int(opx.min()), int(opy.min()), int(opx.max()), int(opy.max())),
}
os.makedirs(os.path.dirname(OUT), exist_ok=True)
out_img.save(OUT, "PNG")
print("WROTE", OUT)
for k, v in report.items():
    print("  %s = %s" % (k, v))
