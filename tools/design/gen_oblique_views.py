# -*- coding: utf-8 -*-
"""gen_oblique_views.py — 生成郑吒 45° 斜视全身立绘(正面/背面之间的斜视参考, 解决"斜视全靠插值")
用法: python gen_oblique_views.py
输出: tools/design/raw_enemy/pc_zhengzha_oblique{45,135}.png (768x1024 纯黑底)
"""
import json, os, re, sys, time, urllib.request, urllib.error

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gen_wan import gen

OUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "raw_enemy")
os.makedirs(OUT_DIR, exist_ok=True)

BASE = ("恐怖生存游戏主角郑吒的全身{view}立绘，完整全身入镜（头顶到脚底），居中构图，整体照明充足，主体清晰明亮，"
        "年轻中国男青年（约25岁），黑色短发，面容坚毅冷峻、五官清晰可辨，穿深灰蓝色紧身T恤与深色战术长裤，腰系战术腰带，"
        "肌肉型身材约7头身、宽肩窄腰，笔直站立，"
        "全身被明亮的冷白色轮廓光（rim light）环绕：头发、双肩、{rim}都有一圈清晰明亮的冷白轮廓线，与纯黑背景完全分离，"
        "主体位于画面中央，头顶接近画面上缘但留出少量纯黑空隙，脚底贴近画面底缘、双腿完整修长、双脚稳稳踩在画面底部，"
        "画面下方仅留约十分之一的纯黑空隙，底部绝无地面、阴影、倒影，"
        "背景为纯黑色(#000000)，无任何场景元素，无地面、无投影、无道具、无杂物，画面中仅主体居中；"
        "人物受明亮冷白主光正面照明，全身从头到脚均匀受光、皮肤与衣物细节清晰可辨，与纯黑背景强烈明暗分离，恐怖写实风格，细节清晰")

OBLIQUE45 = BASE.format(
    view="右前方45度斜视立绘（人物右转45度，同时可见正面约六成与右侧面约四成：右脸、右耳、右侧肩臂、右侧身体侧缘、右侧腿缘、右脚外侧）",
    rim="右侧肩臂外缘、右腿外侧、衣摆下缘")
OBLIQUE135 = BASE.format(
    view="左后方45度斜视立绘（人物左转135度，同时可见背面约六成与左侧面约四成：左后脑、左肩胛、左侧背腰、左侧臀腿、左脚外侧）",
    rim="左侧肩臂外缘、左腿外侧、衣摆下缘")

def main():
    items = [
        ("pc_zhengzha_oblique45", OBLIQUE45),
        ("pc_zhengzha_oblique135", OBLIQUE135),
    ]
    ok_all = True
    for name, prompt in items:
        out = os.path.join(OUT_DIR, name + ".png")
        print("GEN %s ..." % name, flush=True)
        ok = gen(prompt, "768x1024", out)
        print("DONE %s -> %s (%s)" % (name, out, "OK" if ok else "FAIL"), flush=True)
        ok_all = ok_all and ok
        time.sleep(3)
    print("BATCH SUMMARY: %s" % ("ALL OK" if ok_all else "SOME FAILED"), flush=True)
    sys.exit(0 if ok_all else 1)

if __name__ == "__main__":
    main()
