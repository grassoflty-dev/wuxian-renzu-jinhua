# -*- coding: utf-8 -*-
"""批量 floodfill 抠 item 图标（不使用 --hole-solid）。

流程：
  1. 边框一致性检测：取整个外圈 ring(6px 内缩) 的像素，
     计算中位背景色，统计 >=95% 边框像素距中位色 <=8 且边框方差低 → 判为纯色背景。
     否则判为"复杂/渐变背景"，标需特殊处理，不硬抠。
  2. 纯色 → 调 cutout_floodfill 的 cutout()（--zero_rgb=True 保证透明 RGB==0，
     不开 hole_solid，内部孔洞走 ramp 渐变保留），覆盖前备份到 _bak_alpha\。
  3. 数值验证：透明占比、不透明占比(10~60%)、透明像素 RGB 是否全 0、主体完整(bounding box)。
"""
import os, json, sys, shutil
import numpy as np
from PIL import Image

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1"
IMG = os.path.join(BASE, "server-rs", "ui", "assets", "img")
BAK = os.path.join(IMG, "_bak_alpha")
sys.path.insert(0, os.path.join(BASE, "tools"))
from cutout_floodfill import cutout

NONW = ["item_antidote","item_bandage","item_bottle_water","item_core",
        "item_fragment","item_grenade","item_health","item_holy",
        "item_holy_water","item_jiezhou_fu","item_medkit","item_quzhen_fu",
        "item_rune","item_sedative","item_silver_bullet","item_stone","item_torch"]
WPN = ["item_wp_antimatter_round","item_wp_arcan_staff","item_wp_beam_saber",
       "item_wp_beiming_jian","item_wp_dagou_bang","item_wp_dragon_lance",
       "item_wp_dugu_jiujian","item_wp_electromag_gun","item_wp_excalibur_holy",
       "item_wp_fantian_yin","item_wp_feijian_qingyun"]
TARGETS = NONW + WPN

RING = 6            # 边框检测内缩宽度
BG_DIST = 8.0       # 判为背景的距离阈值
BORDER_BG_RATIO = 0.95  # 边框像素中近背景占比阈值

def border_bg_stats(rgb):
    """返回: median_bg, 边框近背景占比, 边框全方差, 边框距离均值。"""
    h, w = rgb.shape[:2]
    # 构建外圈掩膜（内缩 RING 之外）
    mask = np.ones((h, w), dtype=bool)
    mask[RING:h-RING, RING:w-RING] = False
    ring = rgb[mask]  # (N,3)
    med = np.median(ring, axis=0)
    d = np.sqrt(((ring - med) ** 2).sum(axis=1))
    near = float((d <= BG_DIST).sum()) / ring.shape[0]
    var = float(ring.std())
    return med, near, var, float(d.mean())

def opaque_ratio(alpha):
    return float((alpha >= 250).sum()) / alpha.size * 100

def check_transparent_rgb(img_array):
    alpha = img_array[..., 3]
    rgb = img_array[..., :3]
    trans = alpha <= 5
    if trans.sum() == 0:
        return True, 0
    bad = int((rgb[trans] != 0).any(axis=1).sum())
    return bad == 0, bad

def analyze(fname):
    path = os.path.join(IMG, fname)
    im = Image.open(path).convert("RGBA")
    a = np.asarray(im)
    rgb = a[..., :3].astype(np.float64)
    alpha = a[..., 3]
    med, near, var, dmean = border_bg_stats(rgb)
    solid = near >= BORDER_BG_RATIO and var <= 6.0
    return {
        "bg": [int(x) for x in med],
        "border_near": near,
        "border_std": var,
        "border_dmean": dmean,
        "solid": bool(solid),
        "opaque_pct": opaque_ratio(alpha),
    }

def run_one(fname, info):
    path = os.path.join(IMG, fname)
    out = {"file": fname, "kind": info["kind"], "bg": info["bg"],
           "border_near": info["border_near"], "border_std": info["border_std"]}
    if info["kind"] != "solid":
        out["status"] = "needs-special"
        out["reason"] = info["reason"]
        return out
    # 备份
    os.makedirs(BAK, exist_ok=True)
    bak_path = os.path.join(BAK, fname)
    if not os.path.exists(bak_path):
        shutil.copy2(path, bak_path)
        out["backup"] = bak_path
    else:
        out["backup"] = bak_path + " (已存在,不覆盖)"
    try:
        # zero_rgb=True 保证透明像素 RGB==0；hole_solid=False 保留内部孔洞渐变
        st = cutout(path, path, threshold=6.0, gap=0, seal=2, closing=1,
                    feather=2, fix_holes=True, bg=info["bg"], conn=4,
                    zero_rgb=True, hole_channel=2, hole_solid=False)
    except Exception as e:
        out["status"] = "error"
        out["message"] = str(e)
        return out
    # 验证
    chk = np.asarray(Image.open(path).convert("RGBA"))
    al = chk[..., 3]
    opq = opaque_ratio(al)
    trans_rgb_ok, bad = check_transparent_rgb(chk)
    out["opaque_pct"] = opq
    out["semi_pct"] = float(((al>5)&(al<250)).sum())/al.size*100
    out["trans_pct"] = float((al<=5).sum())/al.size*100
    out["trans_rgb_zero"] = trans_rgb_ok
    out["trans_rgb_bad"] = bad
    # 主体完整性: 不透明像素的 bounding box 比较居中且尺寸占比合理
    ys, xs = np.where(al > 5)
    bbox = (int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max()))
    out["bbox"] = bbox
    w = al.shape[1]; h = al.shape[0]
    out["bbox_cover"] = round((bbox[2]-bbox[0])*(bbox[3]-bbox[1])/(h*w)*100, 1)
    # 判定
    out["status"] = "success"
    out["checks"] = []
    if opq < 10 or opq > 60:
        out["checks"].append(f"opaque {opq:.1f}% 超出 10~60%")
    if not trans_rgb_ok:
        out["checks"].append(f"透明像素 RGB 非全 0({bad}px)")
    if not out["checks"]:
        out["checks"].append("通过")
    return out

def main():
    os.makedirs(BAK, exist_ok=True)
    summary = {"solid": [], "special": []}
    results = []
    for f in TARGETS:
        fname = f + ".png"
        st = analyze(fname)
        if st["solid"]:
            info = {"kind": "solid", "bg": st["bg"],
                    "border_near": st["border_near"], "border_std": st["border_std"]}
        else:
            info = {"kind": "special", "bg": st["bg"],
                    "border_near": st["border_near"], "border_std": st["border_std"],
                    "reason": f"边框近背景占比 {st['border_near']*100:.1f}%<95% 或方差 {st['border_std']:.1f}>6"}
        r = run_one(fname, info)
        (summary["solid"] if r["status"] == "success" else summary["special"]).append(fname)
        results.append(r)
    with open(os.path.join(BASE, "tools", "_cutout_item_run.json"), "w", encoding="utf-8") as fh:
        json.dump(results, fh, ensure_ascii=False, indent=2)
    # 打印
    for r in results:
        if r.get("status") == "error":
            print(f"[{r['status']:10s}] {r['file']} ERROR {r.get('message')}")
        else:
            print(f"[{r['status']:10s}] {r['file']} bg={tuple(r['bg'])} "
                  f"border_near={r.get('border_near',0)*100:.1f}% "
                  f"opq={r.get('opaque_pct',float('nan')):.1f}% "
                  f"rgb0={r.get('trans_rgb_zero')} bbox_cover={r.get('bbox_cover')} "
                  f"checks={r.get('checks')}")
    print("SOLID:", len(summary["solid"]))
    print("SPECIAL:", len(summary["special"]))

if __name__ == "__main__":
    main()
