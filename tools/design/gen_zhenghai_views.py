# -*- coding: utf-8 -*-
"""gen_zhenghai_views.py — 用 wan2.7-image 生成郑吒侧面/背面全身立绘(纯 prompt, 该端点不支持参考图)。
用于 SpriteStacking 侧面分层贴合真实轮廓。
输出: tools/design/raw_enemy/pc_zhengzha_{side,back}.png (768x1024 纯黑底, 与正立绘同管线)
"""
import json, os, re, sys, time, urllib.request, urllib.error

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gen_wan import gen

OUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "raw_enemy")
os.makedirs(OUT_DIR, exist_ok=True)

# 与 pc_zhengzha.png 立绘一致的郑吒设定, 只改视角
BASE = ("恐怖生存游戏主角郑吒的全身{view}立绘，完整全身入镜（头顶到脚底），居中构图，整体照明充足，主体清晰明亮，"
        "年轻中国男青年（约25岁），黑色短发，面容坚毅冷峻、五官清晰可辨，穿深灰蓝色紧身T恤与深色战术长裤，腰系战术腰带，"
        "肌肉型身材约7头身、宽肩窄腰，笔直站立，"
        "全身被明亮的冷白色轮廓光（rim light）环绕：头发、双肩、{rim}都有一圈清晰明亮的冷白轮廓线，与纯黑背景完全分离，"
        "主体位于画面中央，头顶接近画面上缘但留出少量纯黑空隙，脚底贴近画面底缘、双腿完整修长、双脚稳稳踩在画面底部，"
        "画面下方仅留约十分之一的纯黑空隙，底部绝无地面、阴影、倒影，"
        "背景为纯黑色(#000000)，无任何场景元素，无地面、无投影、无道具、无杂物，画面中仅主体居中；"
        "人物受明亮冷白主光正面照明，全身从头到脚均匀受光、皮肤与衣物细节清晰可辨，与纯黑背景强烈明暗分离，恐怖写实风格，细节清晰")

SIDE = BASE.format(
    view="侧面立绘（人物完全转向右侧，只露右侧侧面轮廓：侧脸、耳、肩线、手臂前后缘、腿前后缘、鞋侧面）",
    rim="手臂前后缘、双腿前后缘、衣摆下缘")
BACK = BASE.format(
    view="背面立绘（人物背对镜头，只露后背：后脑、肩胛、背部、后腰、臀、双腿背面、鞋后跟；绝对不露脸）",
    rim="双肩外侧、躯干两侧、双腿外侧、衣摆下缘")

def main():
    items = [
        ("pc_zhengzha_side", SIDE),
        ("pc_zhengzha_back", BACK),
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
