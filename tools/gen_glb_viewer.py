# gen_glb_viewer.py — 把各方案生成的 GLB 转成 {positions,indices,colors} JSON，供本地 three.js 查看器用
# 用法: 用 D:\img23d_alt\venv 的 python 运行（有 trimesh/numpy）
import trimesh, numpy as np, json, os

MODELS = [
    ("SF3D薄壳",        r"D:\sf3d\output\mesh.glb"),
    ("视觉外壳多视图",  r"D:\img23d_alt\mv\multiview_mesh.glb"),
    ("TRELLIS2灰",      r"D:\trellis\outputs\pc_zhengzha_1024.glb"),
    ("Hunyuan3D灰",     r"D:\trellis\hunyuan-out\zhengzha_shape.glb"),
    ("Hunyuan3D上色",   r"D:\trellis\hunyuan-out\zhengzha_colored.glb"),
]
OUT_DIR = r"D:\trellis\viewer"
OUT_JS = os.path.join(OUT_DIR, "models.js")
os.makedirs(OUT_DIR, exist_ok=True)
MAX_VERTS = 250000  # 降面上限，避免 JSON 过大

def load_mesh(path):
    m = trimesh.load(path, force="mesh")
    if isinstance(m, trimesh.Scene):
        # 场景 → 合并成单个 mesh
        parts = list(m.geometry.values())
        m = trimesh.util.concatenate(parts)
    return m

def decimate(m):
    if len(m.vertices) <= MAX_VERTS:
        return m
    reduction = 1.0 - MAX_VERTS / len(m.vertices)
    try:
        return m.simplify_quadric_decimation(reduction)  # 新版 trimesh: 0-1 缩减比例
    except Exception as e1:
        try:
            return m.simplify_quadric_decimation(MAX_VERTS * 2)  # 旧版: 目标面数
        except Exception as e2:
            print("  decimate failed:", e1, "|", e2)
            return m

def vertex_colors(m):
    vis = m.visual
    # 1) 已有顶点色
    if hasattr(vis, "vertex_colors") and vis.vertex_colors is not None:
        return np.asarray(vis.vertex_colors, dtype=float)[:, :3] / 255.0
    # 2) 有 UV + 纹理 → 采样纹理到顶点色
    try:
        if hasattr(vis, "uv") and vis.uv is not None and getattr(vis, "material", None) is not None:
            colored = vis.to_color()
            return np.asarray(colored.vertex_colors, dtype=float)[:, :3] / 255.0
    except Exception as e:
        print("  to_color failed:", e)
    return None

data = {}
for name, path in MODELS:
    if not os.path.exists(path):
        print("MISSING", name, path); continue
    print("load", name)
    m = load_mesh(path)
    m = decimate(m)
    # 归一化：居中 + 高度缩放到 1
    v = np.asarray(m.vertices, dtype=float)
    v = v - v.mean(axis=0)
    h = v[:, 1].max() - v[:, 1].min()
    v = v / (h + 1e-6)
    cols = vertex_colors(m)
    if cols is None:
        # 无颜色 → 按法线做简单明暗（朝上亮、朝下暗），便于看形状
        n = np.asarray(m.vertex_normals, dtype=float)
        lam = 0.45 + 0.55 * np.clip(n[:, 1], -1, 1)
        cols = np.stack([lam, lam, lam], axis=1)
    elif len(cols) != len(v):
        # 颜色数不匹配（抽样后）→ 重采样到顶点数
        cols = np.resize(cols, (len(v), 3))
    data[name] = {
        "positions": v.reshape(-1).round(4).tolist(),
        "indices": np.asarray(m.faces, dtype=int).reshape(-1).tolist(),
        "colors": np.clip(cols, 0, 1).reshape(-1).round(3).tolist(),
    }
    print(f"  {name}: {len(v)} verts, {len(m.faces)} faces, colors={cols.shape}")

js = "window.MODELS=" + json.dumps(data) + ";"
with open(OUT_JS, "w", encoding="utf-8") as f:
    f.write(js)
print("wrote", OUT_JS, os.path.getsize(OUT_JS)//1024, "KB")
