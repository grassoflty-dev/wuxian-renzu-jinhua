# -*- coding: utf-8 -*-
"""ds0731h_qc.py — DS0731H 人体底模版关键视图 qwen 判图。"""
import sys, os, re, json, time, urllib.request, urllib.error
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ds0731_qc_vision as q

PROMPT = """你是 3D 角色模型质检员。这是《郑吒》角色 3D 模型渲染图(标准人体网格底模 + 立绘投影上色, 白底)。
请检查并打分(0-100):
1. 人体结构: 是否有真实人体解剖特征(腋下凹陷/手臂内侧/腿间/手指脚趾), 不是圆柱/剪影近似
2. 立绘贴合: 颜色是否来自立绘(躯干=上衣色, 手臂=肤色), 无混色错位
3. 平滑度: 曲面是否平滑, 有无棱线/撕裂/纸片感
4. 斜视(45°/60°): 手臂与躯干之间是否有真实凹陷与间隙, 非贴在一起的大块
输出 JSON: {"item_scores": {"人体结构": n, "立绘贴合": n, "平滑度": n, "斜视凹陷": n}, "total": n, "issues": [...], "praise": [...]}"""

def main():
    key = q.read_key()
    shots = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots"
    views = sys.argv[1].split(',') if len(sys.argv) > 1 else ['front', 'upper', 'side', 'foot']
    out = {}
    for v in views:
        p = os.path.join(shots, f'render_sprite_ds0731h_zhenghai_{v}.png')
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
                text = f"HTTP {e.code}"; break
            except Exception as e:
                text = f"ERR {e}"; break
        out[v] = text
        m = re.search(r'\{.*\}', text, re.S)
        if m:
            try:
                j = json.loads(m.group(0))
                print('total:', j.get('total'), '| items:', j.get('item_scores'))
                for i in j.get('issues', [])[:4]: print('  -', i[:150])
                for i in j.get('praise', [])[:2]: print('  +', i[:120])
            except Exception:
                print('raw:', text[:300])
        else:
            print('raw:', text[:300])
        time.sleep(1)
    with open(os.path.join(shots, 'ds0731h_qc.json'), 'w', encoding='utf-8') as f:
        json.dump(out, f, ensure_ascii=False, indent=1)

if __name__ == '__main__':
    main()
