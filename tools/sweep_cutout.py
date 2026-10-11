# -*- coding: utf-8 -*-
"""sweep_cutout.py — 对每张图尝试多组 cutout_floodfill 阈值参数，报告透明占比/四角/边框残留，用于选参。

用法: <python> sweep_cutout.py <name> [阈值1 阈值2 ...]
每个阈值输出到 tools\_sweep\<name>_t<阈值>.png 并打印统计。
"""
import os, sys, subprocess
import numpy as np
from PIL import Image

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1"
TOOLS = os.path.join(BASE, "tools")
IMG = os.path.join(BASE, "server-rs", "ui", "assets", "img")
SWEEP = os.path.join(TOOLS, "_sweep")
SCRIPT = os.path.join(TOOLS, "cutout_floodfill.py")
PY = os.path.join(TOOLS, ".venv-assets", "Scripts", "python.exe")
os.makedirs(SWEEP, exist_ok=True)

def stats(png):
    a = np.asarray(Image.open(png).convert("RGBA"))
    al = a[..., 3].astype(int)
    total = al.size
    trans = (al <= 5).mean() * 100
    semi = ((al > 5) & (al < 250)).mean() * 100
    opq = (al >= 250).mean() * 100
    rgb = a[..., :3]
    trans_rgb_nonzero = ((al <= 5) & (rgb.sum(axis=2) > 0)).sum()
    # corners (12x12)
    h, w = a.shape[:2]
    p = 12
    corners = [al[0:p, 0:p], al[0:p, w-p:w], al[h-p:h, 0:p], al[h-p:h, w-p:w]]
    corner_opaque_idx = [int((c > 5).mean() * 100) for c in corners]
    # border residue: opaque pixels on 1px border
    border = np.concatenate([al[0, :], al[-1, :], al[:, 0], al[:, -1]])
    border_opq = (border > 5).mean() * 100
    return dict(trans=trans, semi=semi, opq=opq, corner=corner_opaque_idx,
                border_opq=border_opq, trans_rgb_nonzero=int(trans_rgb_nonzero))

def run(name, thr, bg, extra=()):
    src = os.path.join(IMG, name)
    dst = os.path.join(SWEEP, "%s_t%g.png" % (name, thr))
    cmd = [PY, SCRIPT, src, dst,
           "--zero-rgb", "--bg", bg, "--seal", "2", "--closing", "2",
           "--feather", "2", "--fix-holes", "--hole-channel", "3", "--conn", "8"] + list(extra)
    subprocess.run(cmd, shell=True if os.name == "nt" else False,
                   capture_output=True, text=True)
    try:
        return stats(dst)
    except Exception as e:
        return {"err": str(e)}

def main():
    name = sys.argv[1]
    thrs = [float(x) for x in sys.argv[2:]] or [8, 12, 16, 20, 26, 34, 44]
    bgs = {
        "item_wp_fantian_yin.png": "3,4,2",
        "item_silver_bullet.png": "4,4,3",
        "item_wp_madoushu_grimoire.png": "4,5,3",
        "item_wp_wang_zhicai.png": "4,5,4",
        "item_wp_sheng_jian_mj.png": "3,4,2",
        "item_wp_liumai_jian.png": "4,5,3",
        "item_wp_orbital_gun.png": "3,4,2",
    }
    bg = bgs[name]
    print("== %s bg=%s ==" % (name, bg))
    for thr in thrs:
        s = run(name, thr, bg)
        if "err" in s:
            print("  t%-3g ERR %s" % (thr, s["err"])); continue
        print("  t%-3g trans=%.1f%% semi=%.1f%% opq=%.1f%% corners=%s border_opq=%.1f%% rgb_nz_in_trans=%d"
              % (thr, s["trans"], s["semi"], s["opq"], s["corner"], s["border_opq"], s["trans_rgb_nonzero"]))

if __name__ == "__main__":
    main()
