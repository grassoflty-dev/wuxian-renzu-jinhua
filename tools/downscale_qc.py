# -*- coding: utf-8 -*-
"""downscale_qc.py — 将送检图降采样（最长边 ~768px），避免 glm 4图+高token 请求上游超时(UPSTREAM_TIMEOUT)。
输出到 tools/shots_qc/（同名），供 face_compare_score.mjs 送检。
"""
import os
import numpy as np
from PIL import Image

BASE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.abspath(os.path.join(BASE, "shots"))
DST = os.path.abspath(os.path.join(BASE, "shots_qc"))
os.makedirs(DST, exist_ok=True)

TARGET_MAX = 640
files = [
    "render_zhenghai_front.png", "render_zhenghai_side.png", "render_zhenghai_back.png",
    "render_zhenghaibillboard_front.png", "render_zhenghaibillboard_side.png", "render_zhenghaibillboard_back.png",
    "render_zhenghai_face.png", "render_zhenghaibillboard_face.png",
]

def resize(fp, out):
    im = Image.open(fp)
    w, h = im.size
    mx = max(w, h)
    if mx > TARGET_MAX:
        s = TARGET_MAX / mx
        im = im.resize((int(round(w*s)), int(round(h*s))), Image.LANCZOS)
    im.save(out, "PNG")
    return im.size, os.path.getsize(out)

ti = os.path.abspath(os.path.join(BASE, "..", "server-rs", "ui", "assets", "img", "pc_zhengzha.png"))
to = os.path.join(DST, "pc_zhengzha.png")
size, nbytes = resize(ti, to)
print("立绘 -> %s %s %dB" % (to, size, nbytes))

for f in files:
    sp = os.path.join(SRC, f)
    if not os.path.exists(sp):
        print("MISSING", sp); continue
    out = os.path.join(DST, f)
    size, nbytes = resize(sp, out)
    print("%s -> %s %dB" % (f, size, nbytes))
