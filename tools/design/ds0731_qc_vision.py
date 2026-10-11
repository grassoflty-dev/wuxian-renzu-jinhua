# -*- coding: utf-8 -*-
"""ds0731_qc_vision.py — 用 qwen3.7-flash(tokenrhythm API) 对 DS0731 渲染做视觉质检。
用法: python ds0731_qc_vision.py <view1,view2,...>
输出: 每视图 0-100 打分 + 问题描述, 重点检查:
  接缝虚线 / 双脸重影 / 颈部青色线 / 百叶窗条纹 / 俯视空洞与方块感 /
  仰视中空与纸片感 / 斜视平滑 / 明暗连续性
"""
import sys, os, base64, re, json, time, urllib.request

CRED = r"C:\Users\PUBLIC_USER\.dsh\.credentials.yaml"
SHOTS = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots"
BASE = "https://tokenrhythm.studio/v1/chat/completions"
MODEL = "qwen3.7-flash"

def read_key():
    txt = open(CRED, encoding='utf-8').read()
    m = re.search(r'TOKENRHYTHM_API_KEY\s*[:=]\s*["\']?([^"\'\s]+)', txt)
    if not m:
        raise RuntimeError('no TOKENRHYTHM_API_KEY in ' + CRED)
    return m.group(1)

def img_data_url(path, max_side=1024):
    from PIL import Image
    im = Image.open(path).convert('RGB')
    w, h = im.size
    if max(w, h) > max_side:
        s = max_side / max(w, h)
        im = im.resize((int(w * s), int(h * s)), Image.LANCZOS)
    buf = __import__('io').BytesIO()
    im.save(buf, format='PNG')  # 无损, 避免 JPEG 色带干扰判图
    b64 = base64.b64encode(buf.getvalue()).decode()
    return f"data:image/png;base64,{b64}"

PROMPT = """你是 3D 角色模型质检员。这是一张《郑吒》3D 角色模型(由"剪影视觉外壳+三视图立绘皮肤"算法构建, 白底)的渲染截图。
请严格按以下 7 项逐项检查并打分(每项 0-100), 最后给出总分(0-100):
1. 轮廓/体积: 是否有方块感、纸片感、切片感、生硬平面(重点)
2. 接缝: 皮肤贴图边缘是否有虚线接缝、裁切白边、拼接缝
3. 面部: 是否有双脸重影、脸型扭曲、"立体派"生硬拼接、五官错位
4. 颈部: 颈部是否有青色/蓝灰色细线、光晕残留
5. 镂空/中空: 是否能看到模型内部空洞、白色透光(中空/纸片)
6. 条纹: 是否有百叶窗条纹、栅栏感、周期性明暗条纹
7. 明暗: 明暗过渡是否平滑自然、有无明显色带/硬边
输出格式(JSON, 不要其他内容):
{"item_scores": {"轮廓体积": n, "接缝": n, "面部": n, "颈部": n, "镂空中空": n, "条纹": n, "明暗": n}, "total": n, "issues": ["...", ...], "praise": ["...", ...]}"""

def call(key, data_url):
    body = {
        "model": MODEL,
        "messages": [
            {"role": "user", "content": [
                {"type": "text", "text": PROMPT},
                {"type": "image_url", "image_url": {"url": data_url}},
            ]}
        ],
        "max_tokens": 4000,
    }
    req = urllib.request.Request(BASE, data=json.dumps(body).encode(),
                                 headers={"Content-Type": "application/json", "Authorization": "Bearer " + key})
    for attempt in range(6):
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                resp = json.loads(r.read().decode())
            msg = resp["choices"][0]["message"]
            text = msg.get("content") or msg.get("reasoning_content") or ""
            return text
        except urllib.error.HTTPError as e:
            if (e.code == 429 or e.code == 504 or e.code == 502) and attempt < 5:
                time.sleep(15 * (attempt + 1))
                continue
            return f"HTTP {e.code}: {e.read().decode()[:300]}"
        except Exception as e:
            return f"ERR {e}"

def main():
    key = read_key()
    views = sys.argv[1].split(',') if len(sys.argv) > 1 else ['front', 'side', 'profile', 'back', 'face', 'top', 'bottom']
    out = {}
    for v in views:
        p = os.path.join(SHOTS, f'render_sprite_ds0731_zhenghai_{v}.png')
        if not os.path.exists(p):
            print(v, 'MISSING'); continue
        print(f'--- QC {v} ---', flush=True)
        text = call(key, img_data_url(p))
        out[v] = text
        # 尝试解析 JSON
        m = re.search(r'\{.*\}', text, re.S)
        if m:
            try:
                j = json.loads(m.group(0))
                print('total:', j.get('total'), '| items:', j.get('item_scores'))
                for i in j.get('issues', [])[:6]: print('  -', i)
                for i in j.get('praise', [])[:3]: print('  +', i)
            except Exception:
                print('raw:', text[:400])
        else:
            print('raw:', text[:400])
        time.sleep(1)
    with open(os.path.join(SHOTS, 'ds0731_qc_vision.json'), 'w', encoding='utf-8') as f:
        json.dump(out, f, ensure_ascii=False, indent=1)
    print('saved ds0731_qc_vision.json')

if __name__ == '__main__':
    main()
