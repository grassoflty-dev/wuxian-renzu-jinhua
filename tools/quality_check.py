#!/usr/bin/env python3
"""MC Voxel Item Quality Check - 逐张调用 qwen3.7-flash 视觉模型判图"""

import os, sys, json, base64, time, urllib.request

# ── config ──────────────────────────────────────────────────────────
PYTHON_EXE = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\.venv-assets\Scripts\python.exe"
API_URL    = "https://tokenrhythm.studio/v1/chat/completions"
API_KEY    = "sk_tr_4hgjNjV9IO5nF1xLU6RbdbUTPMZGlnBdLU3b7BoDYF4"
MODEL      = "qwen3.7-flash"

IMAGE_FILES = [
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots\render_voxel_item_item_health_orbit45.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots\render_voxel_item_item_health_top.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots\render_voxel_item_item_wp_tulong_dao_orbit45.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots\render_voxel_item_gear_gear_holy_plate_armor_orbit45.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots\render_voxel_item_tr_tr_bahuang_longyin_front.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots\render_voxel_item_bl_sharingan_bloodline_orbit45.png",
]
LABELS = [
    "道具·药瓶 (斜视)",
    "道具·药瓶 (俯视)",
    "武器·屠龙刀 (斜视)",
    "护甲·圣纹板甲 (斜视)",
    "法宝·八荒龙印 (正视)",
    "血统·写轮眼 (斜视)",
]

PROMPT = """这是MC体素方块建模的3D渲染。请评估：
(1)是否看起来由可见的方块/体素堆叠而成(MC风)？
(2)大致外形是否清晰、立体？
(3)纯白背景是否正确？
(4)有无像贴图/纹理照片的平滑渐变痕迹？
(5)主体颜色是否丰富真实？
请简短回答，每图2-3句中文。"""

TEMP_DIR = os.path.join(os.environ.get("TMPDIR", os.environ.get("TEMP", "/tmp")), "voxel_check")
os.makedirs(TEMP_DIR, exist_ok=True)


def image_to_data_url(path):
    """用 PIL 缩到 ~512x512 → 存 temp png → 读为 base64 data URL"""
    from PIL import Image
    import tempfile

    img = Image.open(path).convert("RGB")
    w, h = img.size
    s = min(w, h)
    crop = img.crop(((w - s) // 2, (h - s) // 2, (w + s) // 2, (h + s) // 2))
    crop = crop.resize((512, 512), Image.LANCZOS)

    # save as temp png
    tmp_path = os.path.join(TEMP_DIR, f"check_{os.path.basename(path)}.png")
    crop.save(tmp_path, "PNG")

    with open(tmp_path, "rb") as f:
        b64 = base64.b64encode(f.read()).decode()
    return f"data:image/png;base64,{b64}"


def call_vision(data_url, retry=5):
    payload = {
        "model": MODEL,
        "max_tokens": 4000,
        "messages": [{
            "role": "user",
            "content": [
                {"type": "text", "text": PROMPT},
                {"type": "image_url", "image_url": {"url": data_url}},
            ]
        }]
    }
    body = json.dumps(payload).encode()
    req = urllib.request.Request(API_URL, data=body, method="POST")
    req.add_header("Content-Type", "application/json")
    req.add_header("Authorization", f"Bearer {API_KEY}")

    for attempt in range(1, retry + 1):
        try:
            with urllib.request.urlopen(req, timeout=60) as resp:
                raw = resp.read().decode()
                result = json.loads(raw)
                # 兼容 content / reasoning_content
                text = result.get("content") or ""
                if not text:
                    rc = result.get("reasoning_content") or ""
                    if isinstance(rc, list):
                        text = "".join(r.get("text","") for r in rc if r.get("text"))
                    else:
                        text = rc
                # 也可能在 choices[0].message.content
                if not text:
                    choices = result.get("choices", [])
                    if choices:
                        msg = choices[0].get("message", {})
                        text = msg.get("content") or msg.get("reasoning_content", "")
                        if isinstance(text, list):
                            text = json.dumps(text, ensure_ascii=False)
                return text.strip() if text else "(空响应)"
        except urllib.error.HTTPError as e:
            if e.code == 429 and attempt < retry:
                print(f"  ⏳ 429 rate-limit, wait 15s (attempt {attempt}/{retry})", file=sys.stderr)
                time.sleep(15)
                continue
            return f"HTTP {e.code}: {e.reason}"
        except Exception as e:
            if attempt < retry:
                time.sleep(3)
                continue
            return f"ERROR: {e}"
    return "(重试耗尽)"


def main():
    # ensure stdout can print utf8
    import io, sys
    if hasattr(sys.stdout, 'buffer'):
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    if hasattr(sys.stderr, 'buffer'):
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

    results = []
    for i, (path, label) in enumerate(zip(IMAGE_FILES, LABELS), start=1):
        fname = os.path.basename(path)
        print(f"\n{'='*60}\n图{i}: {fname} -- {label}", flush=True)
        if not os.path.exists(path):
            print(f"  FAIL: file not found: {path}")
            results.append((fname, "FAIL: file not found", False))
            continue
        try:
            data_url = image_to_data_url(path)
        except Exception as e:
            print(f"  FAIL: PIL error: {e}")
            results.append((fname, f"PIL error: {e}", False))
            continue

        answer = call_vision(data_url)
        print(f"  OK: {answer}", flush=True)
        # 简单判断是否合格
        is_ok = "不合格" not in answer and "FAIL" not in answer and "error" not in answer.lower() and "not found" not in answer.lower()
        results.append((fname, answer, is_ok))

    # summary
    total = len(results)
    ok_count = sum(1 for _, _, ok in results if ok)
    fail_list = [(fn, ans) for fn, ans, ok in results if not ok]

    print(f"\n\n{'='*60}")
    print("MC VOXEL RENDERING -- QUALITY CHECK SUMMARY")
    print(f"{'='*60}")
    print(f"\nTotal: {total}, PASSED: {ok_count}, FAILED: {len(fail_list)}\n")
    for i, (fname, answer, ok) in enumerate(results, start=1):
        status = "PASS" if ok else "FAIL"
        preview = answer.replace("\n", " ")[:250]
        print(f"Image {i} [{status}] {fname}")
        print(f"  Verdict: {preview}")
        print()
    if fail_list:
        print("--- FAILURES ---")
        for fn, ans in fail_list:
            print(f"  * {fn}: {ans[:200].replace(chr(10),' ')}")

if __name__ == "__main__":
    main()
