# -*- coding: utf-8 -*-
"""ds0731_arm_qc.py — 针对斜视手臂的专项视觉 QC(qwen3.7-flash)。
重点: 手臂是否独立于躯干、粗细是否正常(不含身体投影)、有无突兀/粘连。"""
import sys, os, re, json, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ds0731_qc_vision as q

PROMPT = """你是 3D 角色模型质检员。这是一张《郑吒》3D 角色模型(剪影视觉外壳+三视图立绘皮肤算法, 白底)的渲染截图, 视角是 45° 斜视/上半身特写。
请专门检查**手臂与躯干的关系**, 逐项打分(0-100):
1. 手臂独立性: 手臂是否与躯干明显分离、能看到手臂轮廓(而非与躯干粘连成一大块)
2. 手臂粗细: 手臂宽度是否接近真实手臂(上臂≈躯干宽度的 1/4~1/3), 是否明显过粗(看起来像身体投影/躯干宽度)
3. 手臂形状: 是否圆润有体积(非扁片/纸片), 上臂→前臂→手是否自然过渡
4. 肩部: 肩峰(三角肌)到手臂的连接是否平滑, 有无突兀凸起或"大头柱"感
5. 躯干: 躯干与手臂之间的间隙是否正确(能看到独立肢体而非贴在一起)
输出 JSON(不要其他内容): {"item_scores": {"手臂独立": n, "手臂粗细": n, "手臂形状": n, "肩部": n, "间隙": n}, "total": n, "issues": [...], "praise": [...]}"""

def main():
    key = q.read_key()
    shots = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots"
    views = sys.argv[1].split(',') if len(sys.argv) > 1 else ['upper', 'side', 'front', 'profile']
    out = {}
    for v in views:
        p = os.path.join(shots, f'render_sprite_ds0731_zhenghai_{v}.png')
        if not os.path.exists(p):
            print(v, 'MISSING'); continue
        print(f'--- {v} ---', flush=True)
        # 用专项 prompt
        body = {
            "model": q.MODEL,
            "messages": [{"role": "user", "content": [
                {"type": "text", "text": PROMPT},
                {"type": "image_url", "image_url": {"url": q.img_data_url(p)}},
            ]}],
            "max_tokens": 4000,
        }
        import urllib.request, urllib.error
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
    with open(os.path.join(shots, 'ds0731_arm_qc.json'), 'w', encoding='utf-8') as f:
        json.dump(out, f, ensure_ascii=False, indent=1)

if __name__ == '__main__':
    main()
