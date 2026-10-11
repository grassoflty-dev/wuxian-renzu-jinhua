# -*- coding: utf-8 -*-
"""legs_only_compare.py — before/after 的「腰部以下」放大对比：每行一个版本，每列一个动作。"""
import sys, os
from PIL import Image, ImageDraw
D, DST = sys.argv[1], sys.argv[2]
names = ["01_idle", "02_walk_0", "02_walk_1", "02_walk_2", "03_jump"]
K = 6                      # 1 art px = 6 物理像素（zoom=2 × 3x 截图）
AX0, AX1, AY0, AY1 = 18, 46, 50, 104
panels = {}
for tag in ("before", "after"):
    for n in names:
        f = os.path.join(D, "%s_crop_%s.png" % (tag, n))
        im = Image.open(f).convert("RGB")
        W, H = im.size
        fx, fy = W / 2.0, H * 0.84
        box = (int(fx + (AX0 - 32) * K), int(fy + (AY0 - 104) * K), int(fx + (AX1 - 32) * K), int(fy + (AY1 - 104) * K))
        panels[(tag, n)] = im.crop(box)
cw = panels[("after", names[0])].width; chh = panels[("after", names[0])].height
pad, lab = 10, 22
Wc = pad + len(names) * (cw + pad)
Hc = lab + 2 * (lab + chh + pad) + pad
canvas = Image.new("RGB", (Wc, Hc), (14, 17, 22))
d = ImageDraw.Draw(canvas)
y = pad
for tag, label, col in (("before", "BEFORE  (procedural trapezoid legs)", (255, 130, 130)),
                        ("after",  "AFTER   (legs skinned from the portrait)", (130, 255, 170))):
    d.text((pad, y), label, fill=col); y += lab
    x = pad
    for n in names:
        canvas.paste(panels[(tag, n)], (x, y))
        d.text((x + 2, y - 14), n, fill=(255, 220, 80))
        x += cw + pad
    y += chh + pad
canvas.save(DST)
print("saved", DST, canvas.size)
