# -*- coding: utf-8 -*-
"""vision_final — 对最终写回的原图做一次性视觉终检（合成棋盘）。
用法: <python> vision_final.py [name...]
"""
import base64, io, json, os, sys, time
import numpy as np
from PIL import Image
import urllib.request

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1"
IMG = os.path.join(BASE, "server-rs", "ui", "assets", "img")
OUTD = os.path.join(BASE, "tools", "_vision_out")
os.makedirs(OUTD, exist_ok=True)
API_KEY = "sk_tr_ZKpYni9Ske1UDTEBQwX3J8G5OElreEleFdnF1hyjOH4"
URL = "https://tokenrhythm.studio/v1/chat/completions"

PROMPT = ("把游戏物品图标抠成透明背景后合成到灰色棋盘格上的图。只给一行结论，"
          "格式『结论: 合格』或『结论: 需人工蒙版』，不要解释。"
          "合格=主体完整未切缺、无明显碍眼光晕残留、内部无穿透棋盘格的洞。")

def proc(name):
    p = os.path.join(IMG, name + ".png")
    im = Image.open(p).convert("RGBA"); a = np.asarray(im); h, w = a.shape[:2]
    bg = np.zeros((h, w, 3), dtype=np.uint8)
    grey = np.array([128, 128, 128]);
    for y in range(h):
        for x in range(w):
            bg[y, x] = grey
    # checker via array
    bg = (bg // 2)
    ch = np.array([[[96, 96, 96], [160, 160, 160]]], dtype=np.int32)
    bg = np.zeros((h, w, 3), dtype=np.int32)
    for y in range(h):
        for x in range(w):
            bg[y, x] = (96, 96, 96) if ((y // 32 + x // 32) % 2 == 0) else (160, 160, 160)
    al = a[..., 3:4].astype(np.float32) / 255.0
    out = (a[..., :3].astype(np.float32) * al + bg.astype(np.float32) * (1 - al)).astype(np.uint8)
    imc = Image.fromarray(out, "RGB"); imc.thumbnail((512, 512), Image.LANCZOS)
    buf = io.BytesIO(); imc.save(buf, "PNG")
    du = "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode()
    payload = {"model": "qwen3.7-flash",
               "messages": [{"role": "user", "content": [
                   {"type": "image_url", "image_url": {"url": du}},
                   {"type": "text", "text": PROMPT}]}], "max_tokens": 60}
    req = urllib.request.Request(URL, data=json.dumps(payload).encode(),
                                 headers={"Content-Type": "application/json",
                                          "Authorization": "Bearer " + API_KEY})
    for at in range(5):
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                o = json.loads(r.read().decode())
            m = o["choices"][0]["message"]
            return (m.get("content") or m.get("reasoning_content") or "").strip()[:200]
        except Exception as e:
            if at == 4:
                return "ERROR %r" % (e,)
            time.sleep(15 * (at + 1))

def main():
    names = sys.argv[1:] or ["item_wp_fantian_yin", "item_silver_bullet",
                             "item_wp_madoushu_grimoire", "item_wp_wang_zhicai",
                             "item_wp_sheng_jian_mj", "item_wp_liumai_jian",
                             "item_wp_orbital_gun"]
    for n in names:
        print("FINAL", n, "=>", flush=True)
        t = proc(n)
        print("   ", t, flush=True)
        with open(os.path.join(OUTD, "final_%s.txt" % n), "w", encoding="utf-8") as f:
            f.write(t)
        time.sleep(1)

if __name__ == "__main__":
    main()
