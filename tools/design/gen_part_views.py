# -*- coding: utf-8 -*-
"""gen_part_views.py — 为郑吒建模生成"部位多视角"参考图(逐部位独立建模用)
先测试生成: 头部正面特写 + 手臂侧面特写, 验证 wan2.7-image 部位特写质量与立绘贴合度。
用法: python gen_part_views.py [all]
"""
import json, os, re, sys, time, urllib.request, urllib.error

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gen_wan import gen

OUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "raw_enemy", "parts")
os.makedirs(OUT_DIR, exist_ok=True)

# 与 pc_zhengzha.png 一致的郑吒设定 (参照 gen_zhenghai_views.py)
CHAR = ("恐怖生存游戏主角郑吒{part}的{view}特写参考图，{detail}"
        "年轻中国男青年（约25岁），黑色短发，面容坚毅冷峻，穿深灰蓝色紧身T恤与深色战术长裤，"
        "肌肉型身材，皮肤偏小麦色，"
        "纯黑色背景(#000000)，无任何场景元素，无地面、无投影、无道具，"
        "明亮冷白主光照明，细节清晰，恐怖写实风格，与全身立绘完全一致的发型/衣着/肤色")

HEAD_F = CHAR.format(part="头部", view="正面",
    detail="头部完整入镜（头顶到下巴），居中对正，正对镜头，双眼直视前方，五官清晰：")
HEAD_S = CHAR.format(part="头部", view="正右侧面",
    detail="头部完整入镜（头顶到下巴），人物完全转向右侧只露右侧侧脸轮廓，侧脸/耳/下颌线清晰：")
HEAD_B = CHAR.format(part="头部", view="背面",
    detail="头部完整入镜（头顶到下巴），后脑勺正对镜头，绝对不露脸，后脑/头发走向清晰：")
HEAD_45 = CHAR.format(part="头部", view="右前45度斜视",
    detail="头部完整入镜（头顶到下巴），右前45度斜侧视角，露出右脸与右耳、颧骨/下颌立体感清晰：")
ARM_S = CHAR.format(part="右臂", view="正右侧面",
    detail="右臂单独完整入镜（肩头到指尖），手臂自然下垂贴体侧，侧面视角只露手臂前后缘轮廓，上臂/前臂/手部清晰：")
ARM_F = CHAR.format(part="右臂", view="正面",
    detail="右臂单独完整入镜（肩头到指尖），手臂自然下垂，正面视角，上臂/前臂/手部清晰：")
TORSO_S = CHAR.format(part="躯干", view="正右侧面",
    detail="躯干单独完整入镜（颈部到胯部），侧面视角只露躯干前后缘轮廓，胸/腰/腹曲线清晰：")
LEG_S = CHAR.format(part="右腿", view="正右侧面",
    detail="右腿单独完整入镜（胯部到脚底），侧面视角只露腿前后缘轮廓，大腿/膝/小腿/脚侧清晰：")
FOOT_S = CHAR.format(part="脚部", view="正右侧面",
    detail="右脚单独特写（脚踝到鞋底），侧面视角，鞋型/鞋底/脚踝清晰：")

ITEMS = [
    ("part_head_front", HEAD_F),
    ("part_head_side", HEAD_S),
    ("part_head_back", HEAD_B),
    ("part_head_45", HEAD_45),
    ("part_arm_side", ARM_S),
    ("part_arm_front", ARM_F),
    ("part_torso_side", TORSO_S),
    ("part_leg_side", LEG_S),
    ("part_foot_side", FOOT_S),
]

def main():
    only = sys.argv[1] if len(sys.argv) > 1 else "test"
    items = ITEMS if only == "all" else ITEMS[:2]  # 默认只测试前 2 张
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
