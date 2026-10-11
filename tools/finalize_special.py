# -*- coding: utf-8 -*-
"""finalize_special.py — 对候选结果做最终清理(清 2px 外边框)，并验证/写回原路径。

用法: <python> finalize_special.py
读取 tools\_sweep\cand_<name>.png → 清 2px 外边框(透明+RGB0) → 存原图路径(覆盖)。
打印每张最终统计与是否写回。
"""
import os
import numpy as np
from PIL import Image

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1"
SWP = os.path.join(BASE, "tools", "_sweep")
IMG = os.path.join(BASE, "server-rs", "ui", "assets", "img")

NAMES = ["item_wp_fantian_yin", "item_silver_bullet",
         "item_wp_madoushu_grimoire", "item_wp_wang_zhicai",
         "item_wp_sheng_jian_mj", "item_wp_liumai_jian",
         "item_wp_orbital_gun"]

def stats(a):
    al = a[..., 3].astype(int)
    rgb = a[..., :3]
    total = al.size
    trans = (al <= 5).mean() * 100
    semi = ((al > 5) & (al < 250)).mean() * 100
    opq = (al >= 250).mean() * 100
    trgb = int(((al <= 5) & (rgb.sum(axis=2) > 0)).sum())
    h, w = a.shape[:2]
    p = 12
    corners = [int((al[0:p, 0:p] > 5).mean() * 100),
               int((al[0:p, w - p:w] > 5).mean() * 100),
               int((al[h - p:h, 0:p] > 5).mean() * 100),
               int((al[h - p:h, w - p:w] > 5).mean() * 100)]
    border = np.concatenate([al[0, :], al[-1, :], al[:, 0], al[:, -1]])
    border_opq = (border > 5).mean() * 100
    return dict(trans=trans, semi=semi, opq=opq, corners=corners,
                border_opq=border_opq, trgb=trgb)

def main():
    for name in NAMES:
        cand = os.path.join(SWP, "cand_%s.png" % name)
        dst = os.path.join(IMG, "%s.png" % name)
        if not os.path.exists(cand):
            print("%s: NO candidate, skip" % name); continue
        a = np.asarray(Image.open(cand).convert("RGBA")).copy()
        # 清 2px 外边框（透明 + RGB0）
        a[:2, :, :] = 0
        a[-2:, :, :] = 0
        a[:, :2, :] = 0
        a[:, -2:, :] = 0
        s = stats(a)
        # 验证主体占比在合理范围
        flag = "OK" if (10 <= s["opq"] <= 60) else ("THIN-ICON" if 2 <= s["opq"] < 10 else "LOW")
        # 覆盖原文件（备份已在 _bak_alpha）
        Image.fromarray(a, "RGBA").save(dst, "PNG")
        print("%-28s trans=%.1f%% semi=%.1f%% opq=%.1f%% corners=%s border_opq=%.2f%% trgb=%d [%s] => %s"
              % (name, s["trans"], s["semi"], s["opq"], s["corners"],
                 s["border_opq"], s["trgb"], flag, os.path.basename(dst)), flush=True)

if __name__ == "__main__":
    main()
