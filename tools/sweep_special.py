# -*- coding: utf-8 -*-
"""sweep_special.py — cutout_special 阈值扫描，报告透明占比/四角/边框残留/透明区内残留RGB。"""
import os, sys, subprocess
import numpy as np
from PIL import Image

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1"
TOOLS = os.path.join(BASE, "tools")
IMG = os.path.join(BASE, "server-rs", "ui", "assets", "img")
SWEEP = os.path.join(TOOLS, "_sweep")
SCRIPT = os.path.join(TOOLS, "cutout_special.py")
PY = os.path.join(TOOLS, ".venv-assets", "Scripts", "python.exe")
os.makedirs(SWEEP, exist_ok=True)

def stats(png):
    a = np.asarray(Image.open(png).convert("RGBA"))
    al = a[..., 3].astype(int)
    trans = (al <= 5).mean() * 100
    semi = ((al > 5) & (al < 250)).mean() * 100
    opq = (al >= 250).mean() * 100
    rgb = a[..., :3]
    trgb = ((al <= 5) & (rgb.sum(axis=2) > 0)).sum()
    h, w = a.shape[:2]
    p = 12
    corners = [al[0:p, 0:p], al[0:p, w-p:w], al[h-p:h, 0:p], al[h-p:h, w-p:w]]
    corner_opq = [int((c > 5).mean() * 100) for c in corners]
    border = np.concatenate([al[0, :], al[-1, :], al[:, 0], al[:, -1]])
    border_opq = (border > 5).mean() * 100
    return dict(trans=trans, semi=semi, opq=opq, corner=corner_opq,
                border_opq=border_opq, trgb=int(trgb))

def run(name, T, bg=None, extra=()):
    src = os.path.join(IMG, name)
    dst = os.path.join(SWEEP, "%s_s_t%g.png" % (name, T))
    cmd = [PY, SCRIPT, src, dst, str(T), "--closing", "2", "--feather", "2"]
    if bg:
        cmd += ["--bg", bg]
    cmd += list(extra)
    subprocess.run(cmd, shell=True if os.name == "nt" else False,
                   capture_output=True, text=True)
    try:
        return stats(dst)
    except Exception as e:
        return {"err": str(e)}

def main():
    name = sys.argv[1]
    thrs = [float(x) for x in sys.argv[2:]] or [8, 12, 16, 22, 30, 40, 55, 75]
    bgs = {
        "item_wp_fantian_yin.png": "3,4,2",
        "item_silver_bullet.png": "4,4,3",
        "item_wp_madoushu_grimoire.png": "4,5,3",
        "item_wp_wang_zhicai.png": "4,5,4",
        "item_wp_sheng_jian_mj.png": "3,4,2",
        "item_wp_liumai_jian.png": "4,5,3",
        "item_wp_orbital_gun.png": "3,4,2",
    }
    bg = bgs.get(name)
    print("== %s ==" % name)
    for T in thrs:
        s = run(name, T, bg=bg)
        if "err" in s:
            print("  T%-3g ERR %s" % (T, s["err"])); continue
        print("  T%-3g trans=%.1f%% semi=%.1f%% opq=%.1f%% corners=%s border_opq=%.1f%% trgb=%d"
              % (T, s["trans"], s["semi"], s["opq"], s["corner"], s["border_opq"], s["trgb"]))

if __name__ == "__main__":
    main()
