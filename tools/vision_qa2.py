# -*- coding: utf-8 -*-
"""vision_qa2.py — 严格二选一终检（只让模型给结论词）。
用法: <python> vision_qa2.py <name>...
输出 _vision_out\qa2_<name>.txt
"""
import base64, io, json, os, sys, time
import numpy as np
from PIL import Image
import urllib.request

BASE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1"
SWP = os.path.join(BASE, "tools", "_sweep")
OUTD = os.path.join(BASE, "tools", "_vision_out")
os.makedirs(OUTD, exist_ok=True)
API_KEY = "sk_tr_ZKpYni9Ske1UDTEBQwX3J8G5OElreEleFdnF1hyjOH4"
URL = "https://tokenrhythm.studio/v1/chat/completions"

PROMPT = ("这张图是把游戏物品图标抠成透明背景后合成到灰色棋盘格上的结果。"
          "请只回答一个短结论，格式严格为：『结论: 合格』或『结论: 需人工蒙版』。"
          "判定标准：物品主体完整、没有把主体的一部分切掉、外部没有把背景的亮边/光晕残留为碍眼的白圈或碎屑、内部没有透出棋盘的洞。"

          "如果主体完整且外部只是很细的干净轮廓边(原图自带描边)或极少量可忽略的羽化，判『合格』；"
          "如果主体明显被切、有碍眼残留光晕、或明显缺块，判『需人工蒙版』。不要解释，只给结论行。")

def checker(im):
    arr = np.asarray(im.convert("RGBA")); h, w = arr.shape[:2]
    from PIL import Image as I
    bg = I.new("RGB", (w, h), (128, 128, 128))
    # checker
    bp = bg.load()
    for y in range(h):
        for x in range(w):
            bp[x, y] = (96, 96, 96) if ((y // 32 + x // 32) % 2 == 0) else (160, 160, 160)
    a = arr[..., 3:4].astype(float) / 255.0
    out = (arr[..., :3].astype(float) * a + np.asarray(bg).astype(float) * (1 - a)).astype('uint8')
    return I.fromarray(out, "RGB")

def ask(data_url):
    payload = {"model": "qwen3.7-flash",
               "messages": [{"role": "user", "content": [
                   {"type": "image_url", "image_url": {"url": data_url}},
                   {"type": "text", "text": PROMPT}]}], "max_tokens": 40}
    req = urllib.request.Request(URL, data=json.dumps(payload).encode(),
                                 headers={"Content-Type": "application/json",
                                          "Authorization": "Bearer " + API_KEY})
    last = None
    for a in range(5):
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                o = json.loads(r.read().decode())
            m = o["choices"][0]["message"]
            return (m.get("content") or m.get("reasoning_content") or "").strip()[:120]
        except Exception as e:
            last = e; time.sleep(15 * (a + 1))
    return "ERROR %r" % last

def main():
    for name in sys.argv[1:] or []:
        p = os.path.join(SWP, "cand_%s.png" % name)
        if not os.path.exists(p):
            print("missing", p); continue
        cmp = checker(Image.open(p)); cmp.thumbnail((512, 512), Image.LANCZOS)
        buf = io.BytesIO(); cmp.save(buf, "PNG")
        du = "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode()
        print("== qa2 %s ==" % name, flush=True)
        for _ in range(3):  # 3 次投票取多数
            t = ask(du)
            print("  ", t, flush=True)
            with open(os.path.join(OUTD, "qa2_%s.txt" % name), "a", encoding="utf-8") as f:
                f.write(t + "\n")
            time.sleep(1)

if __name__ == "__main__":
    main()
