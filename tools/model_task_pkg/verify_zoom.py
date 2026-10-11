# verify_zoom.py — 放大检查清洗后边缘:alpha 形状 vs 颜色残留
import numpy as np
from PIL import Image
import os

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\model_task_pkg"
OUT = os.path.join(BASE, "shots", "diag")

for name, (cx, cy) in [("pc_zhengzha_side", (400, 500)), ("pc_zhengzha_back", (300, 350))]:
    im = Image.open(os.path.join(OUT, name + "_cleaned.png")).convert("RGBA")
    a = np.asarray(im)
    crop = a[cy-90:cy+90, cx-90:cx+90]
    # 白底合成
    rgb = crop[..., :3].astype(np.float32)
    al = crop[..., 3:4].astype(np.float32) / 255.0
    comp = (rgb * al + 255 * (1 - al)).astype(np.uint8)
    Image.fromarray(comp).resize((360, 360), Image.NEAREST).save(os.path.join(OUT, name + "_zoom_comp.png"))
    # alpha 本身
    Image.fromarray(crop[..., 3]).resize((360, 360), Image.NEAREST).save(os.path.join(OUT, name + "_zoom_alpha.png"))
    print(name, "zoom saved, edge alpha uniq:", np.unique(crop[..., 3]).tolist()[:10])
