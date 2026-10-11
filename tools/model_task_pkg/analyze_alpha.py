# analyze_alpha.py — 诊断侧/背立绘 alpha 与边缘"黑影"分布
import numpy as np
from PIL import Image
import os

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\model_task_pkg"
OUT = os.path.join(BASE, "shots", "diag")
os.makedirs(OUT, exist_ok=True)

for name in ["pc_zhengzha_side", "pc_zhengzha_back"]:
    im = Image.open(os.path.join(BASE, "assets", name + ".png")).convert("RGBA")
    a = np.asarray(im).astype(np.int32)
    H, W = a.shape[:2]
    alpha = a[..., 3]
    rgb = a[..., :3]
    print(f"== {name} {W}x{H}")
    # alpha 直方图
    hist, edges = np.histogram(alpha, bins=[0, 8, 30, 60, 100, 150, 200, 230, 250, 256])
    print("  alpha hist [<8,<30,<60,<100,<150,<200,<230,<250,=255]:", hist.tolist())
    # 高 alpha 主体的颜色统计 vs 半透明区颜色统计
    body = alpha > 200
    mist = (alpha > 8) & (alpha <= 200)
    if body.sum():
        print("  body px:", int(body.sum()), "mean rgb:", rgb[body].mean(axis=0).round(1).tolist())
    if mist.sum():
        print("  mist px:", int(mist.sum()), "mean rgb:", rgb[mist].mean(axis=0).round(1).tolist(),
              "mean alpha:", round(float(alpha[mist].mean()), 1))
    # mist 里"黑影"(暗色)与"白晕"(亮色)比例
    dark = mist & (rgb.max(axis=2) < 90)
    lite = mist & (rgb.min(axis=2) > 160)
    print("  mist-dark px:", int(dark.sum()), " mist-light px:", int(lite.sum()))
    # 暗色黑影的高 alpha(>=150)部分 —— 现行 hardCut(150) 会留下的
    darkhi = dark & (alpha >= 150)
    print("  dark&alpha>=150 px:", int(darkhi.sum()), "(hardCut150 后仍残留的黑影)")
    dark200 = dark & (alpha >= 200)
    print("  dark&alpha>=200 px:", int(dark200.sum()), "(阈值200 后仍残留的)")
    # 保存 alpha 可视化 + 主体 mask 可视化
    Image.fromarray(alpha.astype(np.uint8)).save(os.path.join(OUT, name + "_alpha.png"))
    Image.fromarray((body * 255).astype(np.uint8)).save(os.path.join(OUT, name + "_mask200.png"))
    # 用白底合成原图,看边缘真实观感
    comp = np.ones((H, W, 3), np.float32) * 255
    af = alpha[..., None] / 255.0
    comp = rgb * af + comp * (1 - af)
    Image.fromarray(comp.astype(np.uint8)).save(os.path.join(OUT, name + "_onwhite.png"))
    # 边缘裁剪图(人物 bbox 外扩 40px)
    ys, xs = np.where(alpha > 8)
    x0, x1, y0, y1 = xs.min(), xs.max(), ys.min(), ys.max()
    pad = 40
    crop = comp[max(0, y0-pad):min(H, y1+pad), max(0, x0-pad):min(W, x1+pad)]
    Image.fromarray(crop.astype(np.uint8)).save(os.path.join(OUT, name + "_edgecrop.png"))
    print("  bbox(a>8):", (int(x0), int(y0), int(x1), int(y0 and y1)))
print("saved to", OUT)
