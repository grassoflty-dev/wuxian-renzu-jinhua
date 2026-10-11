# -*- coding: utf-8 -*-
"""Sample real colors from Zheng Zha's standing portrait (pc_zhengzha.png).
Region layout based on the portrait's opaque bbox: head near top, legs at bottom.
Outputs a palette markdown table (average RGB + hex + dominant color) and prints JSON."""
import sys, json
from collections import Counter
from PIL import Image

png = r"server-rs\ui\assets\img\pc_zhengzha.png"
im = Image.open(png).convert("RGBA")
W, H = im.size
pix = im.load()

# Alpha mask of opaque subject
def opaque():
    ys = []
    for y in range(0, H, 2):
        for x in range(0, W, 2):
            if pix[x, y][3] > 40:
                ys.append(y)
    return (min(ys), max(ys)) if ys else (0, H)

top, bottom = opaque()
print(f"size={W}x{H} opaque_bbox_y=({top},{bottom})", file=sys.stderr)

def sample(x0, x1, y0, y1, name):
    """Average + dominant color over a rectangular region. Restrict to opaque (alpha>120) pixels."""
    acc = [0, 0, 0]; n = 0
    c = Counter()
    for yy in range(y0, y1):
        for xx in range(x0, x1):
            if xx < 0 or xx >= W or yy < 0 or yy >= H:
                continue
            r, g, b, a = pix[xx, yy]
            if a < 120:
                continue
            acc[0] += r; acc[1] += g; acc[2] += b; n += 1
            # quantize for mode
            c[(r // 16, g // 16, b // 16)] += 1
    if n == 0:
        return {"name": name, "avg": None, "hex": None, "mode": None, "n": 0}
    avg = [c // n for c in acc]
    # un-quantize mode bin center
    mr, mg, mb = c.most_common(1)[0][0]
    mode = [mr * 16 + 8, mg * 16 + 8, mb * 16 + 8]
    return {"name": name, "avg": avg, "hex": "#%02x%02x%02x" % tuple(avg),
            "mode": mode, "modehex": "#%02x%02x%02x" % tuple(mode), "n": n}

# ---- locate body zones from bbox ----
# Opaque spans x=[bx0,bx1], y=[top,bottom]. Use body-wide horizontal slices.
bw = 541 - 227            # ~314 subject width
cx = (227 + 541) / 2      # center x
bodyH = bottom - top

def pct(lo, hi):
    return (int(top + bodyH * lo), int(top + bodyH * hi))

# Rough anatomical proportions for a standing portrait with head at top:
#   hair crown zone
y_hair0, y_hair1 = pct(0.02, 0.09)
#   face/skin
y_face0, y_face1 = pct(0.13, 0.21)
#   neck/chest skin
y_neck0, y_neck1 = pct(0.21, 0.27)
#   upper chest / shoulders shirt
y_chest0, y_chest1 = pct(0.27, 0.40)
#   T-shirt torso
y_shirt0, y_shirt1 = pct(0.40, 0.58)
#   belt / waist
y_belt0, y_belt1 = pct(0.60, 0.68)
#   upper legs / pants
y_pants0, y_pants1 = pct(0.70, 0.86)
#   lower legs
y_pantlo0, y_pantlo1 = pct(0.86, 0.98)

def center_band(lo, hi, width_pct=0.34):
    """A horizontally-centered band of given fraction of subject width."""
    return (int(cx - bw * width_pct / 2), int(cx + bw * width_pct / 2), lo, hi)

# T-shirt torso: wider band (includes shoulders/blocks)
def band_custom(lo, hi, wfrac):
    return (int(cx - bw * wfrac / 2), int(cx + bw * wfrac / 2), lo, hi)

regions = {
    "hair":   band_custom(*pct(0.03, 0.08), 0.40),
    "face":   band_custom(*pct(0.14, 0.20), 0.30),
    "neck":   band_custom(*pct(0.21, 0.26), 0.24),
    "chest":  band_custom(*pct(0.28, 0.40), 0.80),
    "shirt":  band_custom(*pct(0.42, 0.56), 0.80),
    "belt":   band_custom(*pct(0.61, 0.67), 0.44),
    "pants":  band_custom(*pct(0.71, 0.85), 0.44),
    "pantlo": band_custom(*pct(0.87, 0.97), 0.40),
}

out = []
for name, (x0, x1, y0, y1) in regions.items():
    r = sample(x0, x1, y0, y1, name)
    out.append(r)
    print(json.dumps(r), file=sys.stderr)

md = ["# 郑吒立绘像素取色表", "",
      f"来源 `server-rs/ui/assets/img/pc_zhengzha.png`（{W}x{H}，不透明主体 y∈[{top},{bottom}]）", "",
      "| 部位 | 平均RGB | 十六进制 | 主色(众数) | 采样像素 |",
      "|---|---|---|---|---|"]
for r in out:
    md.append(f"| {r['name']} | ({r['avg'][0]}, {r['avg'][1]}, {r['avg'][2]}) | {r['hex']} | {r['modehex']} | {r['n']} |")
md.append("")
md.append("> 平均=区域不透明像素算数平均 RGB；主色=16 级量化后的众数 bin 中心。")

with open(r"tools\zhengha_palette.md", "w", encoding="utf-8") as f:
    f.write("\n".join(md))
print("wrote tools/zhengha_palette.md")
