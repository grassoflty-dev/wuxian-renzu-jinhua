# -*- coding: utf-8 -*-
"""crop_players.py — 读 <tag>_boxes.json，把整屏截图裁成角色特写并放大，拼成蒙太奇。
用法: python crop_players.py <shotsdir> <tag> <outfile>
"""
import sys, os, json
from PIL import Image, ImageDraw
D = sys.argv[1]; TAG = sys.argv[2]; DST = sys.argv[3]
boxes = json.load(open(os.path.join(D, TAG + "_boxes.json"), encoding="utf-8"))
CW, CH, SS = 150, 290, 3          # 裁剪尺寸（CSS px）与放大倍数
panels = []
for b in boxes:
    src = os.path.join(D, "%s_%s.png" % (TAG, b["name"]))
    if not os.path.exists(src): continue
    im = Image.open(src).convert("RGB")
    # 截图物理像素 / CSS = im.width / vw
    k = im.width / float(b["vw"])
    x0 = int(round((b["sx"] - CW / 2) * k)); y0 = int(round((b["sy"] - CH * 0.84) * k))
    x1 = int(round((b["sx"] + CW / 2) * k)); y1 = int(round((b["sy"] + CH * 0.16) * k))
    x0 = max(0, x0); y0 = max(0, y0); x1 = min(im.width, x1); y1 = min(im.height, y1)
    c = im.crop((x0, y0, x1, y1))
    c = c.resize((c.width * SS // max(1, int(k)), c.height * SS // max(1, int(k))), Image.NEAREST)
    panels.append((b["name"] + " " + b["state"], c))
    c.save(os.path.join(D, "%s_crop_%s.png" % (TAG, b["name"])))
    import numpy as np
    arr = np.array(c).astype(np.float32)
    print("  crop %-14s mean=%6.1f  lit=%.3f" % (b["name"], arr.mean(), (arr.max(axis=2) > 45).mean()))
if not panels:
    print("no panels"); sys.exit(1)
pad = 10
h = max(p.height for _, p in panels)
tw = sum(p.width for _, p in panels) + pad * (len(panels) + 1)
canvas = Image.new("RGB", (tw, h + 34), (16, 20, 26))
d = ImageDraw.Draw(canvas)
x = pad
for name, p in panels:
    canvas.paste(p, (x, 28)); d.text((x + 4, 10), name, fill=(255, 220, 80)); x += p.width + pad
canvas.save(DST)
print("saved", DST, canvas.size, "panels", len(panels))
