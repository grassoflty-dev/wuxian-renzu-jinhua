# verify_clean.py — 复现 zz3d.js 清洗流水线,白底合成输出供人工检查抠图质量
import numpy as np
from PIL import Image
import os

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\model_task_pkg"
OUT = os.path.join(BASE, "shots", "diag")
os.makedirs(OUT, exist_ok=True)

def erode_alpha(a, rad):
    m = a.copy()
    for _ in range(rad):
        up = np.roll(m, 1, axis=0); up[0, :] = 255
        dn = np.roll(m, -1, axis=0); dn[-1, :] = 255
        lf = np.roll(m, 1, axis=1); lf[:, 0] = 255
        rt = np.roll(m, -1, axis=1); rt[:, -1] = 255
        m = np.minimum(np.minimum(up, dn), np.minimum(lf, rt))
    return m

def dilate_mask(t, rad):
    m = t.copy()
    for _ in range(rad):
        up = np.roll(m, 1, axis=0); up[0, :] = 0
        dn = np.roll(m, -1, axis=0); dn[-1, :] = 0
        lf = np.roll(m, 1, axis=1); lf[:, 0] = 0
        rt = np.roll(m, -1, axis=1); rt[:, -1] = 0
        m = m | up | dn | lf | rt
    return m

for name in ["pc_zhengzha_side", "pc_zhengzha_back"]:
    im = Image.open(os.path.join(BASE, "assets", name + ".png")).convert("RGBA")
    a = np.asarray(im).astype(np.int32)
    rgb, alpha = a[..., :3], a[..., 3]

    # hardCut(200)
    alpha2 = np.where(alpha < 200, 0, alpha).astype(np.int32)
    # erode 4: 剥掉黑雾致密芯(蓝灰 60-100, alpha 200-255 的斑块) + 白 rim 外圈
    alpha2 = erode_alpha(alpha2, 6)
    # median 3x3 ×2: 多数表决抚平黑雾噪声造成的毛刺边
    for _ in range(2):
        acc = (alpha2 > 0).astype(np.int32)
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                if dy == 0 and dx == 0:
                    continue
                acc += (np.roll(alpha2, (dy, dx), axis=(0, 1)) > 0).astype(np.int32)
        alpha2 = np.where(acc >= 5, 255, 0).astype(np.int32)
    # inpaintRim(16): 贴边 16px 内的蓝白 rim 像素,用非 rim 内部邻域色逐轮填充(不改 alpha,不留洞)
    t = (alpha2 < 60)
    near = dilate_mask(t, 16)
    mn = rgb.min(axis=2)
    bluish = (rgb[..., 2] >= rgb[..., 0] - 10)
    rimMask = near & (alpha2 > 0) & (mn > 130) & bluish
    rgb2 = rgb.copy()
    filled = np.zeros(rimMask.shape, bool)
    for _ in range(16):
        # 已可用色源:非 rim 的不透明像素 + 已填充像素
        srcok = ((alpha2 > 0) & ~rimMask) | filled
        # 对未填充 rim 像素,若有任一 8 邻域 srcok,则取其平均色
        acc = np.zeros((*rimMask.shape, 3), np.float64)
        cnt = np.zeros(rimMask.shape, np.float64)
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                if dy == 0 and dx == 0:
                    continue
                sh_ok = np.roll(srcok, (dy, dx), axis=(0, 1))
                sh_rgb = np.roll(rgb2, (dy, dx), axis=(0, 1))
                w = sh_ok & rimMask & ~filled
                acc[w] += sh_rgb[w]
                cnt[w] += 1
        upd = (cnt > 0)
        if not (upd & rimMask & ~filled).any():
            break
        w2 = upd & rimMask & ~filled
        rgb2[w2] = (acc[w2] / cnt[w2][:, None])
        filled[w2] = True
    # softClamp(208, 0.18)
    over = rgb2 > 208
    rgb2[over] = 208 + ((rgb2[over] - 208) * 0.18)
    # deRim(±5px, bright-white/cyan, >55)
    src = rgb2
    bright = src.mean(axis=2)
    isWhite = src.min(axis=2) > 185
    isCyan = (src[..., 2] > 195) & ((src[..., 2] - src[..., 0]) > 35)
    cand = (isWhite | isCyan) & (src[..., 2] >= src[..., 0] - 12) & (alpha2 > 40)
    outs = []
    for dy, dx in [(0, 5), (0, -5), (5, 0), (-5, 0)]:
        sh = np.roll(src, (dy, dx), axis=(0, 1))
        sha = np.roll(alpha2, (dy, dx), axis=(0, 1))
        outs.append((sha > 40, sh.mean(axis=2), sh))
    dimmest = np.full(bright.shape, 1e9)
    dimcol = src.copy()
    cnt = np.zeros(bright.shape, bool)
    for ok, br, sh in outs:
        upd = ok & (br < dimmest)
        dimmest[upd] = br[upd]
        dimcol[upd] = sh[upd]
        cnt |= ok
    rim = cand & cnt & ((bright - dimmest) > 55)
    rgb2[rim] = dimcol[rim]

    out = np.dstack([rgb2, alpha2]).astype(np.uint8)
    Image.fromarray(out, "RGBA").save(os.path.join(OUT, name + "_cleaned.png"))
    # 白底合成
    comp = np.ones_like(rgb2) * 255
    af = (alpha2[..., None] / 255.0)
    comp = rgb2 * af + comp * (1 - af)
    H, W = comp.shape[:2]
    ys, xs = np.where(alpha2 > 8)
    pad = 30
    crop = comp[max(0, ys.min()-pad):min(H, ys.max()+pad), max(0, xs.min()-pad):min(W, xs.max()+pad)]
    Image.fromarray(crop.astype(np.uint8)).save(os.path.join(OUT, name + "_cleaned_onwhite.png"))
    print(name, "cleaned px:", int((alpha2 > 0).sum()))
print("saved")
