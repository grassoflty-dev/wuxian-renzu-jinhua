# -*- coding: utf-8 -*-
"""analyze_special.py — 分析复杂背景图标的背景特征，指导抠图策略。"""
import sys, os
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

def auto_bg(rgb, patch=8):
    h, w = rgb.shape[:2]
    p = min(patch, h // 2, w // 2)
    corners = np.concatenate([
        rgb[0:p, 0:p].reshape(-1, 3),
        rgb[0:p, w - p:w].reshape(-1, 3),
        rgb[h - p:h, 0:p].reshape(-1, 3),
        rgb[h - p:h, w - p:w].reshape(-1, 3),
    ])
    return np.median(corners, axis=0)

for name in NAMES:
    p = os.path.join(IMG, name)
    if not os.path.exists(p):
        print("MISSING", name); continue
    img = Image.open(p).convert("RGBA")
    arr = np.asarray(img).astype(np.float64)
    rgb = arr[..., :3]
    h, w = rgb.shape[:2]
    bg = auto_bg(rgb)
    # corner patches brightness
    p8 = 8
    corners = {
        "TL": rgb[0:p8, 0:p8].reshape(-1,3).mean(axis=0),
        "TR": rgb[0:p8, w-p8:w].reshape(-1,3).mean(axis=0),
        "BL": rgb[h-p8:h, 0:p8].reshape(-1,3).mean(axis=0),
        "BR": rgb[h-p8:h, w-p8:w].reshape(-1,3).mean(axis=0),
    }
    d = np.sqrt(((rgb - bg) ** 2).sum(axis=2))
    # edge residue: how much of each edge is "near bg" (d<=8)
    edge = {
        "top": (d[0:1, :] <= 8).mean() * 100,
        "bottom": (d[-1:, :] <= 8).mean() * 100,
        "left": (d[:, 0:1] <= 8).mean() * 100,
        "right": (d[:, -1:] <= 8).mean() * 100,
    }
    # brightness histogram summary
    lum = rgb.mean(axis=2)
    th10 = np.percentile(lum, 10)
    th50 = np.percentile(lum, 50)
    th90 = np.percentile(lum, 90)
    alpha_now = arr[..., 3]
    has_alpha = bool((alpha_now < 250).any())
    print("="*70)
    print(name)
    print("  size=%dx%d bg=%s" % (w, h, str(tuple(int(x) for x in bg))))
    print("  corners:", {k: tuple(int(x) for x in v) for k, v in corners.items()})
    print("  edge-nearbg%%:", {k: round(v,1) for k, v in edge.items()})
    print("  lum p10/p50/p90: %.0f / %.0f / %.0f" % (th10, th50, th90))
    print("  existing alpha present:", has_alpha)
