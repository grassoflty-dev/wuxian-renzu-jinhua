# -*- coding: utf-8 -*-
import sys
from PIL import Image, ImageDraw
a = Image.open(sys.argv[1]).convert("RGB")   # before montage
b = Image.open(sys.argv[2]).convert("RGB")   # after montage
pad = 16
W = max(a.width, b.width)
canvas = Image.new("RGB", (W + pad*2, a.height + b.height + 90), (12, 14, 18))
d = ImageDraw.Draw(canvas)
d.text((pad, 8), "BEFORE (程序化梯形腿 / 旧版)", fill=(255,120,120))
canvas.paste(a, (pad, 30))
d.text((pad, a.height + 50), "AFTER (立绘下半身像素蒙皮 / 新版)", fill=(120,255,160))
canvas.paste(b, (pad, a.height + 72))
canvas.save(sys.argv[3])
print("saved", sys.argv[3], canvas.size)
