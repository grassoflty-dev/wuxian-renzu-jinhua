# -*- coding: utf-8 -*-
"""asset_alpha_detect.py — 盘点立绘/图标资产的透明通道情况。

扫描 server-rs/ui/assets/img/ 下 pc_*.png / enemy_*.png / item_*.png，
统计每个图的：
  - 是否有 alpha 通道（RGBA 还是 RGB）
  - 透明像素占比（alpha<10 的像素比例）
  - 四角 / 边缘是否透明（判断背景是否已抠掉）
输出 tools/asset_alpha_report.md 总表 + 本脚本同目录的 asset_alpha_results.json
（供补抠脚本增量使用）。

用法:
  <python> asset_alpha_detect.py [--img <目录>] [--out <报告md>] [--json <结果json>] [--pattern pc|enemy|item|all]
"""
import argparse
import glob
import json
import os

from PIL import Image
import numpy as np

PROJ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TOOLS = os.path.dirname(os.path.abspath(__file__))
DEFAULT_IMG = os.path.join(PROJ, "server-rs", "ui", "assets", "img")
DEFAULT_OUT = os.path.join(TOOLS, "asset_alpha_report.md")
DEFAULT_JSON = os.path.join(TOOLS, "asset_alpha_results.json")


def classify_name(fn):
    if fn.startswith("pc_"):
        return "pc"
    if fn.startswith("enemy_"):
        return "enemy"
    if fn.startswith("item_"):
        return "item"
    return "other"


def analyze(path):
    """返回该图的 alpha 统计 dict。"""
    with Image.open(path) as im:
        mode = im.mode
        has_alpha = "A" in mode
        w, h = im.size
        # 转 RGBA 统一分析
        rgba = im.convert("RGBA")
        arr = np.asarray(rgba).astype(np.uint16)
    alpha = arr[..., 3]
    n = alpha.size
    # 透明像素(alpha<=10)比例
    trans_ratio = float((alpha <= 10).sum()) / n * 100.0
    # 半透明像素(11..249)比例
    semi_ratio = float(((alpha > 10) & (alpha < 250)).sum()) / n * 100.0
    opaque_ratio = float((alpha >= 250).sum()) / n * 100.0
    # alpha 通道的取值分布
    a_min = int(alpha.min())
    a_max = int(alpha.max())
    a_unique = int(np.unique(alpha).size)
    # 四角 8x8 块是否透明
    p = min(8, h // 2, w // 2)
    corners = [
        alpha[0:p, 0:p],
        alpha[0:p, w - p:w],
        alpha[h - p:h, 0:p],
        alpha[h - p:h, w - p:w],
    ]
    corner_trans = [float((c <= 10).sum()) / c.size * 100.0 for c in corners]
    corner_all_trans = all(x == 100.0 for x in corner_trans)
    # 边框(四边 2px 条带)透明比例
    edge = np.concatenate([alpha[0:2, :].ravel(), alpha[h - 2:h, :].ravel(),
                           alpha[:, 0:2].ravel(), alpha[:, w - 2:w].ravel()])
    edge_trans_ratio = float((edge <= 10).sum()) / edge.size * 100.0
    edge_all_trans = edge_trans_ratio == 100.0

    # 结论
    no_alpha = not has_alpha
    if no_alpha or (not corner_all_trans and not edge_all_trans):
        if trans_ratio < 1.0:
            verdict = "需抠图"      # 完全不透明背景
        elif corner_all_trans:
            verdict = "可直接billboard"   # 四角透明但背景未全抠?保守再看
        else:
            verdict = "需抠图"
    else:
        if trans_ratio >= 5.0:
            verdict = "可直接billboard"
        else:
            verdict = "需抠图"

    # 细化：无alpha通道 / 有alpha但背景不透明 → 需抠
    if no_alpha:
        verdict = "需抠图"  # RGB 无透明通道，背景必不透明
    elif corner_all_trans and edge_all_trans and trans_ratio >= 5.0:
        verdict = "可直接billboard"
    else:
        # 有alpha但某些角度模糊：以四角 & 边缘是否全部透明为准
        verdict = "可直接billboard" if corner_all_trans and edge_all_trans else "需抠图"

    return {
        "file": os.path.basename(path),
        "mode": mode,
        "has_alpha": has_alpha,
        "size": "%dx%d" % (w, h),
        "trans_ratio": round(trans_ratio, 2),
        "semi_ratio": round(semi_ratio, 2),
        "opaque_ratio": round(opaque_ratio, 2),
        "alpha_min": a_min,
        "alpha_max": a_max,
        "corner_trans": [round(x, 1) for x in corner_trans],
        "corner_all_trans": corner_all_trans,
        "edge_trans_ratio": round(edge_trans_ratio, 2),
        "edge_all_trans": edge_all_trans,
        "verdict": verdict,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--img", default=DEFAULT_IMG)
    ap.add_argument("--out", default=DEFAULT_OUT)
    ap.add_argument("--json", default=DEFAULT_JSON)
    ap.add_argument("--pattern", default="all", choices=("pc", "enemy", "item", "all"))
    args = ap.parse_args()

    files = []
    for pref in ("pc", "enemy", "item"):
        if args.pattern in ("all", pref):
            files += sorted(glob.glob(os.path.join(args.img, "%s_*.png" % pref)))

    results = [analyze(f) for f in files]
    res_by_cat = {}
    for r in results:
        res_by_cat.setdefault(r["file"][: r["file"].index("_")], []).append(r)

    # 汇总
    cats = {}
    for r in results:
        c = classify_name(r["file"])
        cats.setdefault(c, {"total": 0, "billboard": 0, "cutout": 0,
                            "special": 0, "cutout_files": [], "billboard_files": []})
        cats[c]["total"] += 1
        if r["verdict"] == "可直接billboard":
            cats[c]["billboard"] += 1
            cats[c]["billboard_files"].append(r["file"])
        else:
            cats[c]["cutout"] += 1
            cats[c]["cutout_files"].append(r["file"])

    total = len(results)
    # 写 JSON
    with open(args.json, "w", encoding="utf-8") as f:
        json.dump({"total": total, "cats": cats, "results": results},
                  f, ensure_ascii=False, indent=2)

    # 写 markdown
    lines = []
    lines.append("# 立绘/图标 Alpha 透明通道盘点报告\n")
    lines.append("> 生成时间：%s" % _now())
    lines.append("> 分析目录：`%s`" % args.img)
    lines.append("> 判定逻辑：有 alpha 通道 + 四角全透明 + 边框全透明 + 透明占比≥5% → 可直接 billboard；否则需抠图。\n")
    lines.append("## 汇总\n")
    lines.append("| 类别 | 总数 | 可直接billboard | 需抠图 |")
    lines.append("|------|------|----------------|--------|")
    order = ("pc", "enemy", "item")
    for c in order:
        x = cats.get(c, {"total": 0, "billboard": 0, "cutout": 0})
        lines.append("| %s | %d | %d | %d |" % (c, x["total"], x["billboard"], x["cutout"]))
    x_all = cats
    import collections
    agg = collections.Counter()
    for c in order:
        for k in ("total", "billboard", "cutout"):
            agg[k] += cats.get(c, {}).get(k, 0)
    lines.append("| **合计** | **%d** | **%d** | **%d** |" % (agg["total"], agg["billboard"], agg["cutout"]))
    lines.append("")

    lines.append("## 明细\n")
    lines.append("| 类别 | 文件名 | 模式 | 尺寸 | 透明占比% | 半透明% | 不透明% | 四角透明 | 边框透明 | 结论 |")
    lines.append("|------|--------|------|------|-----------|---------|---------|----------|----------|------|")
    for r in sorted(results, key=lambda x: (x["file"], )):
        c = classify_name(r["file"])
        corner = ("全透明" if r["corner_all_trans"] else
                  "部分(%s)" % "/".join(str(x) for x in r["corner_trans"]))
        edge = ("全透明" if r["edge_all_trans"] else "%.0f%%" % r["edge_trans_ratio"])
        lines.append("| %s | `%s` | %s | %s | %.1f | %.1f | %.1f | %s | %s | **%s** |" % (
            c, r["file"], r["mode"], r["size"],
            r["trans_ratio"], r["semi_ratio"], r["opaque_ratio"],
            corner, edge, r["verdict"]))
    lines.append("")

    # 需抠图清单
    lines.append("## 需抠图清单\n")
    for c in order:
        files_c = cats.get(c, {}).get("cutout_files", [])
        if files_c:
            lines.append("### %s（%d 张）\n" % (c, len(files_c)))
            for fn in files_c:
                lines.append("- `%s`" % fn)
            lines.append("")

    with open(args.out, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))

    print("scanned=%d report=%s json=%s" % (total, args.out, args.json), flush=True)
    print("SUMMARY:", json.dumps(cats, ensure_ascii=False), flush=True)


def _now():
    import datetime
    return datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")


if __name__ == "__main__":
    main()
