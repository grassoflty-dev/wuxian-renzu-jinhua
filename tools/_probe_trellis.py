import trimesh, numpy as np
m = trimesh.load(r'D:\trellis\outputs\pc_zhengzha_1024.glb', force='mesh')
print('type:', type(m).__name__)
print('verts', len(m.vertices), 'faces', len(m.faces))
v = m.visual
print('visual kind:', v.kind)
print('has uv:', v.uv is not None, 'uv shape:', v.uv.shape if v.uv is not None else None)
print('material:', getattr(v, 'material', None))
print('has vertex_colors:', getattr(v, 'vertex_colors', None) is not None)
try:
    c = v.to_color()
    print('to_color kind:', c.kind)
    if c.vertex_colors is not None:
        print('vertex_colors mean RGB:', np.asarray(c.vertex_colors)[:, :3].mean(axis=0).round(1))
    else:
        print('to_color vertex_colors None')
except Exception as e:
    print('to_color err:', repr(e))
