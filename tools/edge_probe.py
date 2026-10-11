# -*- coding: utf-8 -*-
"""edge_probe.py — 探查边界像素分布，判断能否作为 flood 种子。"""
import os
import numpy as np
from PIL import Image

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1"
IMG = os.path.join(BASE, "server-rs", "ui", "assets", "img")
NAMES = ["item_wp_fantian_yin.png", "item_silver_bullet.png",
         "item_wp_madoushu_grimoire.png", "item_wp_wang_zhicai.png",
         "item_wp_sheng_jian_mj.png", "item_wp_liumai_jian.png",
         "item_wp_orbital_gun.png"]

for name in NAMES:
    a = np.asarray(Image.open(os.path.join(IMG, name)).convert("RGBA")).astype(float)
    rgb = a[..., :3]
    lum = rgb.mean(axis=2)
    top = lum[0, :]
    bot = lum[-1, :]
    lef = lum[:, 0]
    rig = lum[:, -1]
    def stat(v, label):
        m = v.mean()
        print("    %s: avg=%.1f min=%.1f max=%.1f dark%%(=<=10)=%.1f%%" %
              (label, m, v.min(), v.max(), (v <= 10).mean() * 100))
    print(name)
    stat(top, "top")
    stat(bot, "bottom")
    stat(lef, "left")
    stat(rig, "right")
