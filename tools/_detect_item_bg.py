# -*- coding: utf-8 -*-
"""检测 28 张 item 首图四角背景是否纯色，判断能否走 floodfill。
纯色: 四个角块(8x8 内缩)逐像素与角块中位色距离 <=2 且角块内 std 极小。
输出: 每张的判定 + 背景色 + 当前 alpha 概况。
"""
import os, sys, json
import numpy as np
from PIL import Image

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1"
IMG = os.path.join(BASE, "server-rs", "ui", "assets", "img")

NONW = ["item_antidote","item_bandage","item_bottle_water","item_core",
        "item_fragment","item_grenade","item_health","item_holy",
        "item_holy_water","item_jiezhou_fu","item_medkit","item_quzhen_fu",
        "item_rune","item_sedative","item_silver_bullet","item_stone",
        "item_torch"]
WPN = ["item_wp_antimatter_round","item_wp_arcan_staff","item_wp_beam_saber",
       "item_wp_beiming_jian","item_wp_dagou_bang","item_wp_dragon_lance",
       "item_wp_dugu_jiujian","item_wp_electromag_gun","item_wp_excalibur_holy",
       "item_wp_fantian_yin","item_wp_feijian_qingyun"]
TARGETS = NONW + WPN

def corner_analysis(rgb, patch=8):
    h, w = rgb.shape[:2]
    p = patch
    corners = {
        "tl": rgb[0:p, 0:p],
        "tr": rgb[0:p, w-p:w],
        "bl": rgb[h-p:h, 0:p],
        "br": rgb[h-p:h, w-p:w],
    }
    res = {}
    for name, blk in corners.items():
        med = np.median(blk.reshape(-1,3), axis=0)
        dist = np.sqrt(((blk.reshape(-1,3) - med)**2).sum(axis=1))
        res[name] = {
            "bg": [int(x) for x in med],
            "maxd": float(dist.max()),
            "mean": float(dist.mean()),
            "std": float(blk.reshape(-1,3).std()),
        }
    return res

def analyze(fname):
    path = os.path.join(IMG, fname)
    im = Image.open(path).convert("RGBA")
    a = np.asarray(im)
    rgb = a[..., :3].astype(np.float64)
    alpha = a[..., 3]
    h, w = rgb.shape[:2]
    cr = corner_analysis(rgb)
    # 判定纯色: 所有角块 maxd<=4 且 std<=4
    corner_vals = [v["bg"] for v in cr.values()]
    same_corner = all(np.array_equal(corner_vals[0], c) for c in corner_vals)
    solid = all(v["maxd"] <= 4.0 and v["std"] <= 4.5 for v in cr.values())
    # 四角是否相同→纯色背景；若四角各不同→可能即主体占满(无背景)或非纯背景
    total = alpha.size
    info = {
        "file": fname,
        "size": f"{w}x{h}",
        "bg": corner_vals[0] if same_corner else None,
        "solid": bool(solid),
        "same_corner": bool(same_corner),
        "corners": cr,
        "opaque": float((alpha>=250).sum())/total*100,
        "semi": float(((alpha>5)&(alpha<250)).sum())/total*100,
        "trans": float((alpha<=5).sum())/total*100,
        "all_opaque": bool((alpha>=250).sum()==total),
    }
    return info

out = {}
for f in TARGETS:
    fname = f + ".png"
    try:
        out[fname] = analyze(fname)
    except Exception as e:
        out[fname] = {"error": str(e)}

with open(os.path.join(BASE, "tools", "_detect_item_bg.json"), "w", encoding="utf-8") as fh:
    json.dump(out, fh, ensure_ascii=False, indent=2)

# 打印简洁表
print(f"{'file':38s} {'solid':5s} {'same_c':6s} {'all_op':6s} {'opq%':>6s} {'bg':12s}")
for f, v in out.items():
    if "error" in v:
        print(f"{f:38s} ERROR {v['error']}")
        continue
    bg = str(tuple(v["bg"])) if v["bg"] else "-"
    print(f"{f:38s} {str(v['solid']):5s} {str(v['same_corner']):6s} {str(v['all_opaque']):6s} {v['opaque']:6.1f} {bg}")
print("done")
