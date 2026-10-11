# -*- coding: utf-8 -*-
"""vision_qa.py — 把候选抠图合成到棋盘背景+qwen判图。
对每个候选：读 RGBA，合成到棋盘（保留 alpha），调 qwen3.7-flash 判主体是否完整/有无被吃/有无残留亮边。
输出 tools\_vision_out\qa_<name>.txt
"""
import base64, io, json, os, sys, time
import numpy as np
from PIL import Image, ImageDraw
import urllib.request

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1"
SWP = os.path.join(BASE, "tools", "_sweep")
OUTD = os.path.join(BASE, "tools", "_vision_out")
os.makedirs(OUTD, exist_ok=True)
API_KEY = "sk_tr_ZKpYni9Ske1UDTEBQwX3J8G5OElreEleFdnF1hyjOH4"
URL = "https://tokenrhythm.studio/v1/chat/completions"

QA_PROMPT = (
    "下面是把一张游戏物品图标抠成透明背景后、合成到灰色棋盘格背景上的结果。"
    "请只判断 4 件事并简短回答(中文)：\n"
    "1.物品主体是否完整（有没有某部分被挖空/被切除/只剩残片）。若有请指出位置。\n"
    "2.主体外部是否残留明显亮边/光晕/碎屑(多余的半透明或亮色区域)。\n"
    "3.主体内部是否有不该有的黑洞/镂空。\n"
    "4.整体作为掉落物图标是否合格(合格/不合格/勉强)。\n"
    "结论请用『结论:xxx』开头。"
)

def checker(im, cell=32, grayA=96, grayB=160):
    arr = np.asarray(im.convert("RGBA"))
    h, w = arr.shape[:2]
    bg = np.zeros((h, w, 3), dtype=np.uint8)
    cols = [np.array([grayA, grayA, grayA], dtype=np.uint8),
            np.array([grayB, grayB, grayB], dtype=np.uint8)]
    for y in range(h):
        for x in range(w):
            bg[y, x] = cols[(y // cell + x // cell) % 2]
    a = arr[..., 3:4].astype(np.float32) / 255.0
    out = (arr[..., :3].astype(np.float32) * a + bg.astype(np.float32) * (1 - a)).astype(np.uint8)
    return Image.fromarray(out, "RGB")

def b64(img, max_dim=512):
    cp = img.copy(); cp.thumbnail((max_dim, max_dim), Image.LANCZOS)
    buf = io.BytesIO(); cp.save(buf, "PNG")
    return base64.b64encode(buf.getvalue()).decode()

def ask(data_url):
    payload = {"model": "qwen3.7-flash",
               "messages": [{"role": "user", "content": [
                   {"type": "image_url", "image_url": {"url": data_url}},
                   {"type": "text", "text": QA_PROMPT}]}],
               "max_tokens": 2000}
    req = urllib.request.Request(URL, data=json.dumps(payload).encode(),
                                 headers={"Content-Type": "application/json",
                                          "Authorization": "Bearer " + API_KEY})
    last = None
    for attempt in range(5):
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                obj = json.loads(r.read().decode())
            m = obj["choices"][0]["message"]
            return (m.get("content") or m.get("reasoning_content") or "").strip()
        except Exception as e:
            last = e; print("   attempt%d err %r" % (attempt, e), flush=True); time.sleep(15 * (attempt + 1))
    return "ERROR %r" % (last,)

def main():
    names = sys.argv[1:] or ["item_wp_fantian_yin", "item_silver_bullet",
                             "item_wp_madoushu_grimoire", "item_wp_wang_zhicai",
                             "item_wp_sheng_jian_mj", "item_wp_liumai_jian",
                             "item_wp_orbital_gun"]
    for name in names:
        p = os.path.join(SWP, "cand_%s.png" % name)
        if not os.path.exists(p):
            print("missing", p); continue
        cmp = checker(Image.open(p))
        pngbytes = io.BytesIO()
        cmp.thumbnail((512, 512), Image.LANCZOS)
        cmp.save(pngbytes, "PNG")
        data_url = "data:image/png;base64," + base64.b64encode(pngbytes.getvalue()).decode()
        print("== qa %s ==" % name, flush=True)
        txt = ask(data_url)
        with open(os.path.join(OUTD, "qa_%s.txt" % name), "w", encoding="utf-8") as f:
            f.write(txt)
        print(txt, flush=True)
        time.sleep(1)

if __name__ == "__main__":
    main()
