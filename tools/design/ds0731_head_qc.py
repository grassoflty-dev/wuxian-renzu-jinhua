# -*- coding: utf-8 -*-
"""ds0731_head_qc.py — 头部斜视专项视觉 QC(qwen3.7-flash)。
重点: 脸颊弧线是否平滑(椭球感)、有无"立方体两面贴图"拼接感、正/侧脸过渡、后脑。"""
import sys, os, re, json, time, urllib.request, urllib.error
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ds0731_qc_vision as q

PROMPT = """你是 3D 角色模型质检员。这是《郑吒》角色模型(剪影视觉外壳+三视图立绘皮肤算法, 白底)的头部斜视特写渲染图(45° 前侧或 135° 后侧)。
请专门检查**头部是否是真实椭球/球面建模**, 逐项打分(0-100):
1. 椭球感: 头部轮廓(颅顶/脸颊/下巴)是否呈平滑弧线, 有无方角/棱线/"立方体"感
2. 脸颊弧线: 斜视时脸颊(颧骨→腮→下巴)是否有自然的曲面转折, 还是一段"平面"直接切到侧脸
3. 贴图拼接: 正脸与侧脸交界处是否有明显"两张平面贴图拼接"的边界/折痕/颜色断层
4. 五官立体: 斜视时鼻子/眉眼是否有立体感(曲面上的自然透视), 还是平面贴图上的平移
5. 后脑(135° 时): 后脑/后颈是否与前脸球面对称圆润, 有无凹陷或平面
输出 JSON(不要其他内容): {"item_scores": {"椭球感": n, "脸颊弧线": n, "贴图拼接": n, "五官立体": n, "后脑": n}, "total": n, "issues": [...], "praise": [...]}"""

def main():
    key = q.read_key()
    shots = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots"
    views = sys.argv[1].split(',') if len(sys.argv) > 1 else ['head45', 'head135', 'face', 'profile']
    out = {}
    for v in views:
        p = os.path.join(shots, f'render_sprite_ds0731_zhenghai_{v}.png')
        if not os.path.exists(p):
            print(v, 'MISSING'); continue
        print(f'--- {v} ---', flush=True)
        body = {
            "model": q.MODEL,
            "messages": [{"role": "user", "content": [
                {"type": "text", "text": PROMPT + "\n直接输出 JSON, 不要任何分析过程文字。"},
                {"type": "image_url", "image_url": {"url": q.img_data_url(p)}},
            ]}],
            "max_tokens": 8000,
            "response_format": {"type": "json_object"},
        }
        req = urllib.request.Request(q.BASE, data=json.dumps(body).encode(),
                                     headers={"Content-Type": "application/json", "Authorization": "Bearer " + key})
        text = None
        for attempt in range(6):
            try:
                with urllib.request.urlopen(req, timeout=120) as r:
                    resp = json.loads(r.read().decode())
                msg = resp["choices"][0]["message"]
                text = msg.get("content") or msg.get("reasoning_content") or ""
                break
            except urllib.error.HTTPError as e:
                if (e.code == 429 or e.code == 504 or e.code == 502) and attempt < 5:
                    time.sleep(15 * (attempt + 1)); continue
                text = f"HTTP {e.code}: {e.read().decode()[:200]}"; break
            except Exception as e:
                text = f"ERR {e}"; break
        out[v] = text
        m = re.search(r'\{.*\}', text, re.S)
        if m:
            try:
                j = json.loads(m.group(0))
                print('total:', j.get('total'), '| items:', j.get('item_scores'))
                for i in j.get('issues', [])[:5]: print('  -', i[:160])
                for i in j.get('praise', [])[:3]: print('  +', i[:120])
            except Exception:
                print('raw:', text[:400])
        else:
            print('raw:', text[:400])
        time.sleep(1)
    with open(os.path.join(shots, 'ds0731_head_qc.json'), 'w', encoding='utf-8') as f:
        json.dump(out, f, ensure_ascii=False, indent=1)

if __name__ == '__main__':
    main()
