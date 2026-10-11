# -*- coding: utf-8 -*-
"""vision_analyze.py — 调 qwen3.7-flash(tokenrhythm) base64 逐张判图，输出主体/背景结构分析。

用法: <python> vision_analyze.py
输出逐张 JSON 文本到 tools\_vision_out\ 下，命名 <name>.txt。
"""
import base64, io, json, os, sys, time
import urllib.request
from PIL import Image

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1"
IMG = os.path.join(BASE, "server-rs", "ui", "assets", "img")
OUTD = os.path.join(BASE, "tools", "_vision_out")
os.makedirs(OUTD, exist_ok=True)

API_KEY = "sk_tr_ZKpYni9Ske1UDTEBQwX3J8G5OElreEleFdnF1hyjOH4"
URL = "https://tokenrhythm.studio/v1/chat/completions"

PROMPT = (
    "你是游戏素材视觉质检。下面是一张 768x768 的游戏物品图标，背景近似纯黑，"
    "主体是较亮的物品/武器，可能带亮晕辉光、可能主体贴到画面边缘。我们需要抠成透明背景。"
    "请用中文回答：\n"
    "1) 主物品形状/类别，位于画面大致哪块区域（用中心坐标或象限描述）；是否占满哪些边。\n"
    "2) 背景是否均匀近黑；辉光是否延伸到画面四边/四角。\n"
    "3) 主体主色是什么；主体上有哪些明显偏暗的部位（暗部在阈值抠图时会被当背景误删，要标出来）。\n"
    "4) 主体与背景边界亮暗对比强不强。\n"
    "5) 四角 9x9 区域是纯黑还是有亮色。\n"
    "只给结论，不要建议。"
)

def b64(imgpath, max_dim=512):
    im = Image.open(imgpath).convert("RGB")
    im.thumbnail((max_dim, max_dim), Image.LANCZOS)
    buf = io.BytesIO()
    im.save(buf, "PNG")
    return base64.b64encode(buf.getvalue()).decode()

def ask(name):
    data_url = "data:image/png;base64," + b64(os.path.join(IMG, name))
    payload = {
        "model": "qwen3.7-flash",
        "messages": [
            {"role": "user", "content": [
                {"type": "image_url", "image_url": {"url": data_url}},
                {"type": "text", "text": PROMPT},
            ]},
        ],
        "max_tokens": 4000,
    }
    req = urllib.request.Request(
        URL, data=json.dumps(payload).encode(),
        headers={"Content-Type": "application/json", "Authorization": "Bearer " + API_KEY},
    )
    last = None
    for attempt in range(5):
        try:
            with urllib.request.urlopen(req, timeout=120) as resp:
                obj = json.loads(resp.read().decode())
            msg = obj["choices"][0]["message"]
            content = msg.get("content")
            rr = msg.get("reasoning_content")
            text = content or rr or ""
            return text.strip()
        except Exception as e:
            last = e
            print("  attempt %d err: %r" % (attempt, e), flush=True)
            time.sleep(15 * (attempt + 1))
    return "ERROR: %r" % (last,)

def main():
    names = sys.argv[1:] if len(sys.argv) > 1 else [
        "item_wp_fantian_yin.png", "item_silver_bullet.png",
        "item_wp_madoushu_grimoire.png", "item_wp_wang_zhicai.png",
        "item_wp_sheng_jian_mj.png", "item_wp_liumai_jian.png",
        "item_wp_orbital_gun.png"]
    for name in names:
        print("== %s ==" % name, flush=True)
        try:
            txt = ask(name)
        except Exception as e:
            txt = "ERROR %r" % (e,)
        out = os.path.join(OUTD, name.rsplit(".", 1)[0] + ".txt")
        with open(out, "w", encoding="utf-8") as f:
            f.write(txt)
        print(txt, flush=True)
        time.sleep(1)

if __name__ == "__main__":
    main()
