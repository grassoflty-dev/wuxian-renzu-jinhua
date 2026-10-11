# head_ratio_check2.py — 精确头部区域: 从顶往下找宽度鼓包(头顶→脸→下巴→颈)
from PIL import Image
import numpy as np

base = r'C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\server-rs\ui\assets\img'
fr = np.array(Image.open(base + r'\pc_zhengzha.png').convert('RGBA'))
sd = np.array(Image.open(base + r'\pc_zhengzha_side.png').convert('RGBA'))

def bbox(a):
    al = a[..., 3] > 15
    ys, xs = np.where(al)
    return xs.min(), xs.max(), ys.min(), ys.max()

def row_widths(a, x0, x1, y0, y1):
    out = []
    for y in range(y0, y1):
        al = a[y, x0:x1 + 1, 3] > 15
        if al.any():
            xs = np.where(al)[0] + x0
            out.append((y, int(xs.min()), int(xs.max()), int(xs.max() - xs.min() + 1)))
    return out

def find_head(a, bx0, bx1, by0, by1):
    """从 bbox 顶往下: 头部 = 宽度鼓包(先增后减), 到宽度跳增(肩膀)为止"""
    rw = row_widths(a, bx0, bx1, by0, by1)
    if not rw:
        return None
    rows = []
    prev_w = None
    for i, (y, l, r, w) in enumerate(rw):
        if prev_w is not None and w > prev_w * 1.45 and rows:
            # 宽度跳增 45% -> 到肩膀了, 头部结束
            break
        rows.append((y, l, r, w))
        prev_w = w
    return rows

for name, a, bb in [('front', fr, bbox(fr)), ('side', sd, bbox(sd))]:
    bx0, bx1, by0, by1 = bb
    hd = find_head(a, bx0, bx1, by0, by1)
    if not hd:
        print(name, 'no head rows'); continue
    ys = [r[0] for r in hd]
    ws = [r[3] for r in hd]
    iw = int(np.argmax(ws))
    y0, y1 = ys[0], ys[-1]
    print('%s head rows y=%d..%d (%d rows), maxW=%d at y=%d (x %d..%d)' % (
        name, y0, y1, len(hd), ws[iw], ys[iw], hd[iw][1], hd[iw][2]))
    print('  headH=%d  (bboxH=%d => %.1f%%)' % (y1 - y0 + 1, by1 - by0 + 1, (y1 - y0 + 1) / (by1 - by0 + 1) * 100))
    # 首行宽(头顶) 与 末行宽(下巴/颈)
    print('  top row w=%d, bottom row w=%d' % (ws[0], ws[-1]))

# 换算: 正面头宽 Wf_px, 侧面头深 Ds_px, 各自归一化到身高
fx0, fx1, fy0, fy1 = bbox(fr)
sx0, sx1, sy0, sy1 = bbox(sd)
fh = fy1 - fy0 + 1
sh = sy1 - sy0 + 1
hd_f = find_head(fr, fx0, fx1, fy0, fy1)
hd_s = find_head(sd, sx0, sx1, sy0, sy1)
if hd_f and hd_s:
    Wf = max(r[3] for r in hd_f)
    Ds = max(r[3] for r in hd_s)
    # 归一化: 头宽/身高 vs 头深/身高
    w_norm = Wf / fh
    d_norm = Ds / sh
    print('\nfront headW=%dpx (%.3f of height), side headD=%dpx (%.3f of height)' % (Wf, w_norm, Ds, d_norm))
    print('截面宽深比 xr:zrad = %.2f : 1 (归一化后)' % (w_norm / d_norm))
    print('=> xr=%.2f * figW, zrad=%.2f * figW(若figW≈figD)' % (w_norm / 2, d_norm / 2))
