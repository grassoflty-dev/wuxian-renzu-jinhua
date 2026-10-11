# -*- coding: utf-8 -*-
"""analyze_special2.py — 深层亮度分布分析。"""
import os
import numpy as np
from PIL import Image

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1"
IMG = os.path.join(BASE, "server-rs", "ui", "assets", "img")

NAMES = [
    "item_wp_fantian_yin.png",
    "item_silver_bullet.png",
    "item_wp_madoushu_grimoire.png",
    "item_wp_wang_zhicai.png",
    "item_wp_sheng_jian_mj.png",
    "item_wp_liumai_jian.png",
    "item_wp_orbital_gun.png",
]

for name in NAMES:
    a = np.asarray(Image.open(os.path.join(IMG, name)).convert("RGBA")).astype(float)
    rgb = a[..., :3]
    lum = rgb.mean(axis=2)
    line = []
    for t in (15, 30, 60, 120, 200):
        line.append(">%d:%.1f%%" % (t, (lum > t).mean() * 100))
    print(name, " ".join(line))
    mask = lum > 15
    if mask.any():
        ys, xs = np.where(mask)
        print("   bbox x[%d-%d] y[%d-%d]" % (xs.min(), xs.max(), ys.min(), ys.max()))
    # also check edge brightness avg (1px ring)
    edge_ring = np.concatenate([lum[0, :], lum[-1, :], lum[:, 0], lum[:, -1]])
    print("   edge-ring avg lum=%.1f max=%.1f" % (edge_ring.mean(), edge_ring.max()))
