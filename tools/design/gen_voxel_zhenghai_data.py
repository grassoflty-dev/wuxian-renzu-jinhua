# gen_voxel_zhenghai_data.py — 郑吒高分辨率体素建模数据生成器（Python 版，标准库，无外部依赖）
# ============================================================================
# 与 tools/design/gen_voxel_zhenghai_data.mjs 为同一流水线的 Python 移植（本环境无 Python
# 运行时，数据文件由 mjs 版实际生成；本文件为规范交付物，参数与逻辑完全一致）。
#
# 流水线：
#   1) 解析 makehuman_base.obj（y-up 四边形）→ 三角形网格，应用 muscle target 顶点位移
#   2) 缩放至 H=2.3 世界高、脚底贴地；解剖地标检测（颈/肩/手/裆/踝/下颌）
#   3) 头部放大（贴合立绘大头比例）+ 手臂垂放手术（臂降至立绘拳位）
#   4) 纵向比例 warp（mesh 关键点 → 立绘关键点，立绘 v 自上而下实测校准）
#   5) 网格体素化：粗网格（全身 SV=0.055，表面+内部填充）＋ 头部细网格（SVH=0.0275）
#   6) 立绘剪影贴合（trim 超出立绘轮廓的体素列）
#   7) 逐体素取色：按暴露面方向投影采样正/侧/背立绘（run 夹取 + 部位·v区中位色回退）
#   8) 烘焙方向明暗（MC 风格面着色，无光照依赖）→ 输出 voxel_zhenghai_data.js
#
# 用法：python gen_voxel_zhenghai_data.py
# 输出：server-rs/ui/js/voxel_zhenghai_data.js
import json
import os
import struct
import sys
import zlib

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
OBJ = os.path.join(ROOT, 'tools', 'model_task_pkg', 'human_ref', 'makehuman_base.obj')
TARGET = os.path.join(ROOT, 'tools', 'model_task_pkg', 'human_ref', 'makehuman_targets',
                      'universal-male-young-maxmuscle-averageweight.target')
IMG = os.path.join(ROOT, 'server-rs', 'ui', 'assets', 'img')
OUT = os.path.join(ROOT, 'server-rs', 'ui', 'js', 'voxel_zhenghai_data.js')

# ---------------- 参数 ----------------
H = 2.3
SV = 0.055
SVH = SV / 2
HS_XZ, HS_Z, HS_Y = 1.28, 1.18, 1.28
ARM_RAD = 0.185
ARM_HAND_Y = 1.07
TRIM_MARGIN = 0.7
FILL_V0, FILL_V1 = 0.215, 0.33
# 立绘关键点（bbox v 自上而下，立绘像素实测校准；世界 y 由 vOf 逆映射计算）
ART_V = {'brow': 0.145, 'eye': 0.16, 'mouth': 0.18, 'chin': 0.19, 'shoulder': 0.228, 'crotch': 0.546, 'ankle': 0.927}
SHADE = {'+y': 1.10, '-y': 0.88, '+z': 1.03, '-z': 0.94, '+x': 0.99, '-x': 0.99}
PRIORITY = ['+z', '-z', '+x', '-x', '+y', '-y']
FACE_BIT = {'+x': 1, '-x': 2, '+y': 4, '-y': 8, '+z': 16, '-z': 32}


def clamp(v, a, b):
    return a if v < a else (b if v > b else v)


def smooth(t):
    t = clamp(t, 0.0, 1.0)
    return t * t * (3 - 2 * t)


# ---------------- OBJ / target 解析 ----------------
def parse_obj(path):
    verts, faces = [], []
    for line in open(path, encoding='utf-8'):
        line = line.strip()
        if line.startswith('v '):
            t = line.split()
            verts.append([float(t[1]), float(t[2]), float(t[3])])
        elif line.startswith('f '):
            t = line.split()[1:]
            faces.append([int(s.split('/')[0]) - 1 for s in t])
    tris = []
    for f in faces:
        for i in range(1, len(f) - 1):
            tris.append([f[0], f[i], f[i + 1]])
    return verts, tris


def parse_target(path):
    out = []
    for line in open(path, encoding='utf-8'):
        line = line.strip()
        if not line or line.startswith('#'):
            continue
        t = line.split()
        out.append([int(t[0]), float(t[1]), float(t[2]), float(t[3])])
    return out


# ---------------- 三角形 vs AABB（Akenine-Möller SAT）----------------
def cross(a, b):
    return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def dot(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def tri_box_overlap(bc, bh, v0, v1, v2):
    v0x, v0y, v0z = v0[0] - bc[0], v0[1] - bc[1], v0[2] - bc[2]
    v1x, v1y, v1z = v1[0] - bc[0], v1[1] - bc[1], v1[2] - bc[2]
    v2x, v2y, v2z = v2[0] - bc[0], v2[1] - bc[1], v2[2] - bc[2]
    if min(v0x, v1x, v2x) > bh[0] or max(v0x, v1x, v2x) < -bh[0]:
        return False
    if min(v0y, v1y, v2y) > bh[1] or max(v0y, v1y, v2y) < -bh[1]:
        return False
    if min(v0z, v1z, v2z) > bh[2] or max(v0z, v1z, v2z) < -bh[2]:
        return False
    e0 = [v1x - v0x, v1y - v0y, v1z - v0z]
    e1 = [v2x - v1x, v2y - v1y, v2z - v1z]
    e2 = [v0x - v2x, v0y - v2y, v0z - v2z]
    n = cross(e0, e1)
    if abs(dot(n, [v0x, v0y, v0z])) > bh[0] * abs(n[0]) + bh[1] * abs(n[1]) + bh[2] * abs(n[2]):
        return False
    axes = [[1, 0, 0], [0, 1, 0], [0, 0, 1]]
    for e in (e0, e1, e2):
        for ax in axes:
            l = cross(e, ax)
            if l[0] == 0 and l[1] == 0 and l[2] == 0:
                continue
            p0, p1, p2 = dot(l, [v0x, v0y, v0z]), dot(l, [v1x, v1y, v1z]), dot(l, [v2x, v2y, v2z])
            r = bh[0] * abs(l[0]) + bh[1] * abs(l[1]) + bh[2] * abs(l[2])
            if min(p0, p1, p2) > r or max(p0, p1, p2) < -r:
                return False
    return True


# ---------------- PNG 解码（8bit RGBA，非隔行）----------------
def decode_png(path):
    buf = open(path, 'rb').read()
    pos, idat = 8, []
    w = h = 0
    while pos < len(buf):
        ln = struct.unpack('>I', buf[pos:pos + 4])[0]
        typ = buf[pos + 4:pos + 8].decode('ascii')
        if typ == 'IHDR':
            w, h = struct.unpack('>II', buf[pos + 8:pos + 16])
        elif typ == 'IDAT':
            idat.append(buf[pos + 8:pos + 8 + ln])
        elif typ == 'IEND':
            break
        pos += 12 + ln
    raw = zlib.decompress(b''.join(idat))
    ch = 4
    stride = w * ch
    out = bytearray(w * h * 4)
    prev = bytearray(stride)
    p = 0
    for y in range(h):
        ft = raw[p]
        p += 1
        row = raw[p:p + stride]
        p += stride
        cur = bytearray(stride)
        for i in range(stride):
            a = cur[i - ch] if i >= ch else 0
            b = prev[i]
            c = prev[i - ch] if i >= ch else 0
            v = row[i]
            if ft == 1:
                v = (v + a) & 255
            elif ft == 2:
                v = (v + b) & 255
            elif ft == 3:
                v = (v + ((a + b) >> 1)) & 255
            elif ft == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                v = (v + pr) & 255
            cur[i] = v
        prev = cur
        for x in range(w):
            s = x * ch
            o = (y * w + x) * 4
            out[o] = cur[s]
            out[o + 1] = cur[s + 1]
            out[o + 2] = cur[s + 2]
            out[o + 3] = cur[s + 3]
    return {'w': w, 'h': h, 'd': out}


def dehalo(img):
    d, n = img['d'], img['w'] * img['h']
    gr = gg = gb = gc = 0
    for i in range(n):
        a = d[i * 4 + 3]
        if 8 < a < 90:
            gr += d[i * 4]
            gg += d[i * 4 + 1]
            gb += d[i * 4 + 2]
            gc += 1
    if gc < 50:
        return
    gr, gg, gb = gr / gc, gg / gc, gb / gc
    for j in range(n):
        o = j * 4
        al = d[o + 3]
        if al == 0:
            continue
        if al < 60:
            d[o + 3] = 0
            continue
        if al < 245:
            t = al / 255
            it = 1 - t
            d[o] = max(0, min(255, (d[o] - gr * it) / t))
            d[o + 1] = max(0, min(255, (d[o + 1] - gg * it) / t))
            d[o + 2] = max(0, min(255, (d[o + 2] - gb * it) / t))


def cc_clean(img):
    d, W, H = img['d'], img['w'], img['h']
    N = W * H
    th = 0.15 * 255
    visited = bytearray(N)
    comps = []
    for i in range(N):
        if visited[i] or d[i * 4 + 3] <= th:
            continue
        stack = [i]
        visited[i] = 1
        px = []
        while stack:
            p = stack.pop()
            px.append(p)
            x, y = p % W, p // W
            for np_, dx, dy in ((p - 1, -1, 0), (p + 1, 1, 0), (p - W, 0, -1), (p + W, 0, 1)):
                if 0 <= x + dx < W and 0 <= y + dy < H and not visited[np_] and d[np_ * 4 + 3] > th:
                    visited[np_] = 1
                    stack.append(np_)
        comps.append(px)
    if not comps:
        return
    mx = max(len(c) for c in comps)
    keep = max(150, mx * 0.04)
    for c in comps:
        if len(c) >= keep:
            continue
        for q in c:
            d[q * 4 + 3] = 0


def erode_alpha(img, rad):
    d, W, H = img['d'], img['w'], img['h']
    a = bytearray(d[i * 4 + 3] for i in range(W * H))
    for _ in range(rad):
        b = bytearray(W * H)
        for y in range(H):
            for x in range(W):
                p = y * W + x
                m = a[p]
                if x > 0 and a[p - 1] < m:
                    m = a[p - 1]
                if x < W - 1 and a[p + 1] < m:
                    m = a[p + 1]
                if y > 0 and a[p - W] < m:
                    m = a[p - W]
                if y < H - 1 and a[p + W] < m:
                    m = a[p + W]
                b[p] = m
        a = b
    for k in range(W * H):
        d[k * 4 + 3] = a[k]


def detect_bbox(img):
    d, W, H = img['d'], img['w'], img['h']
    minX, minY, maxX, maxY = W, H, -1, -1
    for y in range(H):
        for x in range(W):
            if d[(y * W + x) * 4 + 3] > 15:
                minX, maxX = min(minX, x), max(maxX, x)
                minY, maxY = min(minY, y), max(maxY, y)
    if maxX < 0:
        return None
    return {'left': minX / W, 'right': (maxX + 1) / W, 'top': minY / H, 'bottom': (maxY + 1) / H}


def row_runs(img, bb, gap=3):
    d, W, H = img['d'], img['w'], img['h']
    x0, x1 = round(bb['left'] * W), round(bb['right'] * W) - 1
    y0, y1 = round(bb['top'] * H), round(bb['bottom'] * H) - 1
    hn = y1 - y0 + 1
    rows = []
    for ry in range(hn):
        py = y0 + ry
        runs = []
        l, gc = -1, 0
        for x in range(x0, x1 + 1):
            on = d[(py * W + x) * 4 + 3] > 127
            if on:
                if l < 0:
                    l = x
                gc = 0
            elif l >= 0:
                gc += 1
                if gc > gap:
                    if x - gc - l >= 3:
                        runs.append([l - x0, x - gc - x0])
                    l, gc = -1, 0
        if l >= 0 and x1 - l + 1 >= 3:
            runs.append([l - x0, x1 - x0])
        rows.append(runs)
    return {'wn': x1 - x0 + 1, 'hn': hn, 'rows': rows}


def sample(img, bb, u, v):
    d, W, H = img['d'], img['w'], img['h']
    fx = (bb['left'] + u * (bb['right'] - bb['left'])) * W - 0.5
    fy = (bb['top'] + v * (bb['bottom'] - bb['top'])) * H - 0.5
    x0 = max(0, min(W - 1, int(fx)))
    y0 = max(0, min(H - 1, int(fy)))
    x1 = min(W - 1, x0 + 1)
    y1 = min(H - 1, y0 + 1)
    tx = max(0.0, min(1.0, fx - x0))
    ty = max(0.0, min(1.0, fy - y0))
    out = []
    for c in range(3):
        v00 = d[(y0 * W + x0) * 4 + c]
        v10 = d[(y0 * W + x1) * 4 + c]
        v01 = d[(y1 * W + x0) * 4 + c]
        v11 = d[(y1 * W + x1) * 4 + c]
        out.append((v00 * (1 - tx) + v10 * tx) * (1 - ty) + (v01 * (1 - tx) + v11 * tx) * ty)
    return out


def px_at(img, px, py):
    x, y = int(px), int(py)
    if x < 0 or x >= img['w'] or y < 0 or y >= img['h']:
        return None
    o = (y * img['w'] + x) * 4
    return [img['d'][o], img['d'][o + 1], img['d'][o + 2], img['d'][o + 3]]


# ---------------- 体素化（表面 + 内部填充）----------------
def voxelize(verts, tris, sv, x0, y0, z0, nx, ny, nz):
    occ = bytearray(nx * ny * nz)
    bh = [sv / 2, sv / 2, sv / 2]
    eps = sv * 0.08  # 边界 epsilon：三角形恰贴体素边界（脚底 y=0）仍标记相邻单元

    def cell(x, y, z):
        return int((x - x0) // sv), int((y - y0) // sv), int((z - z0) // sv)

    for tri in tris:
        a, b, c = verts[tri[0]], verts[tri[1]], verts[tri[2]]
        ix0, iy0, iz0 = cell(min(a[0], b[0], c[0]) - eps, min(a[1], b[1], c[1]) - eps, min(a[2], b[2], c[2]) - eps)
        ix1, iy1, iz1 = cell(max(a[0], b[0], c[0]) + eps, max(a[1], b[1], c[1]) + eps, max(a[2], b[2], c[2]) + eps)
        for iz in range(max(0, iz0), min(nz - 1, iz1) + 1):
            for iy in range(max(0, iy0), min(ny - 1, iy1) + 1):
                for ix in range(max(0, ix0), min(nx - 1, ix1) + 1):
                    bc = [x0 + (ix + 0.5) * sv, y0 + (iy + 0.5) * sv, z0 + (iz + 0.5) * sv]
                    bhe = [bh[0] + sv * 0.02, bh[1] + sv * 0.02, bh[2] + sv * 0.02]
                    if tri_box_overlap(bc, bhe, a, b, c):
                        occ[iz * ny * nx + iy * nx + ix] = 1
    # 内部填充：边界 flood fill 空单元
    filled = bytearray(nx * ny * nz)
    q = []

    def push(ix, iy, iz):
        p = iz * ny * nx + iy * nx + ix
        if occ[p] or filled[p]:
            return
        filled[p] = 1
        q.append(p)

    for ix in range(nx):
        for iy in range(ny):
            push(ix, iy, 0)
            push(ix, iy, nz - 1)
    for iz in range(nz):
        for iy in range(ny):
            push(0, iy, iz)
            push(nx - 1, iy, iz)
    for iz in range(nz):
        for ix in range(nx):
            push(ix, 0, iz)
            push(ix, ny - 1, iz)
    while q:
        p = q.pop()
        iz, rem = p // (ny * nx), p % (ny * nx)
        iy, ix = rem // nx, rem % nx
        if ix > 0:
            push(ix - 1, iy, iz)
        if ix < nx - 1:
            push(ix + 1, iy, iz)
        if iy > 0:
            push(ix, iy - 1, iz)
        if iy < ny - 1:
            push(ix, iy + 1, iz)
        if iz > 0:
            push(ix, iy, iz - 1)
        if iz < nz - 1:
            push(ix, iy, iz + 1)
    for i in range(nx * ny * nz):
        if not filled[i]:
            occ[i] = 1
    return occ


# ---------------- 主流程 ----------------
def main():
    verts, tris = parse_obj(OBJ)
    tgt = parse_target(TARGET)
    print('OBJ verts=%d tris=%d target=%d' % (len(verts), len(tris), len(tgt)))
    for i, dx, dy, dz in tgt:
        if 0 <= i < len(verts):
            verts[i][0] += dx
            verts[i][1] += dy
            verts[i][2] += dz
    y_min = min(v[1] for v in verts)
    y_max = max(v[1] for v in verts)
    s = H / (y_max - y_min)
    for v in verts:
        v[0] *= s
        v[1] = (v[1] - y_min) * s
        v[2] *= s

    # 地标检测
    BAND = 0.02
    bands = []
    for y in range(0, 1000):
        yy = y * BAND
        if yy > H:
            break
        mx = mz = cw = 0
        n = 0
        for v in verts:
            if v[1] < yy - BAND / 2 or v[1] > yy + BAND / 2:
                continue
            n += 1
            ax = abs(v[0])
            if ax > mx:
                mx, mz = ax, v[2]
            if ax < 0.25 and ax > cw:
                cw = ax
        bands.append({'y': yy, 'n': n, 'mx': mx, 'mz': mz, 'cw': cw})
    neck_y, neck_w = 1.95, 1e9
    for b in bands:
        if 1.5 < b['y'] < 2.2 and b['n'] > 15 and b['cw'] < neck_w:
            neck_w, neck_y = b['cw'], b['y']
    arm_trace = [b for b in bands if 1.15 < b['y'] < 1.95 and b['mx'] > 0.30]
    hand_b, sh_b = arm_trace[0], arm_trace[-1]
    shoulder_y, shoulder_x = sh_b['y'], sh_b['mx']
    hand_y, hand_x, hand_z = hand_b['y'], hand_b['mx'], hand_b['mz']
    crotch_y = 1.05
    found = []
    for b in bands:
        if b['y'] < 0.3 or b['y'] > hand_y - 0.15 or b['n'] < 40:
            continue
        xs = sorted(v[0] for v in verts if abs(v[1] - b['y']) <= BAND / 2 and abs(v[0]) < 0.5)
        blobs = []
        for x in xs:
            if blobs and x - blobs[-1][1] < 0.09:
                blobs[-1][1] = x
            else:
                blobs.append([x, x])
        big = [b2 for b2 in blobs if b2[1] - b2[0] > 0.05]
        if len(big) >= 2 and abs(big[0][0] + big[0][1] - big[1][0] - big[1][1]) > 0.1:
            found.append(b['y'])
    if found:
        crotch_y = found[-1]
    ankle_y = 0.09
    jaw_y = neck_y + 0.10
    for b in bands:
        if neck_y + 0.01 < b['y'] < neck_y + 0.45 and b['cw'] > 0.075:
            jaw_y = b['y']
            break
    print('landmarks neck=%.3f shoulder=(%.3f,%.3f) hand=(%.3f,%.3f,%.3f) crotch=%.3f ankle=%.3f jaw=%.3f'
          % (neck_y, shoulder_x, shoulder_y, hand_x, hand_y, hand_z, crotch_y, ankle_y, jaw_y))

    # 立绘（提前加载，供 ART 关键点/取色）
    def load_portrait(fname):
        img = decode_png(os.path.join(IMG, fname))
        dehalo(img)
        cc_clean(img)
        erode_alpha(img, 2)
        bb = detect_bbox(img)
        runs = row_runs(img, bb)
        return {'img': img, 'bb': bb, 'runs': runs}

    fr = load_portrait('pc_zhengzha.png')
    sd = load_portrait('pc_zhengzha_side.png')
    bk = load_portrait('pc_zhengzha_back.png')
    # ART 关键点世界坐标：vOf 逆映射（vOf(y)=bb.top+(1-y/H)*(bb.bottom-bb.top)）
    ART = {}
    for k, vv in ART_V.items():
        ART[k] = H * (1 - (vv - fr['bb']['top']) / (fr['bb']['bottom'] - fr['bb']['top']))

    # 头部放大（|x|<0.22 头颈区）
    neck_cy, neck_cz, fall = neck_y - 0.02, 0.10, 0.14
    for v in verts:
        if v[1] < neck_cy - fall or abs(v[0]) > 0.22:
            continue
        f = smooth((v[1] - (neck_cy - fall)) / fall)
        v[0] *= 1 + (HS_XZ - 1) * f
        v[1] = neck_cy + (v[1] - neck_cy) * (1 + (HS_Y - 1) * f)
        v[2] = neck_cz + (v[2] - neck_cz) * (1 + (HS_Z - 1) * f)
    ymx = max(v[1] for v in verts)
    if ymx > H:
        k = H / ymx
        for v in verts:
            v[1] *= k
    ymn = min(v[1] for v in verts)
    if ymn < 0:
        for v in verts:
            v[1] -= ymn
    jaw_y = neck_y + (jaw_y - neck_y) * (1 + (HS_Y - 1) * 0.5)
    # 头面特征检测（头放大后，warp 前）：下巴/嘴/眼/眉 用脸前部 maxZ 剖面
    prof = []
    yy = 1.55
    while yy <= H:
        mz = -1
        for v in verts:
            if abs(v[1] - yy) > 0.005 or abs(v[0]) > 0.09 or v[2] <= 0.05:
                continue
            if v[2] > mz:
                mz = v[2]
        if mz > 0:
            prof.append((yy, mz))
        yy += 0.01
    eye_y, eye_max = 2.0, -1
    for p in prof:
        if 1.85 < p[0] < 2.15 and p[1] > eye_max:
            eye_max, eye_y = p[1], p[0]
    jaw_min, jaw2_y = 1e9, jaw_y
    for p in prof:
        if 1.70 < p[0] < eye_y - 0.02 and p[1] < jaw_min:
            jaw_min, jaw2_y = p[1], p[0]
    mouth_min, mouth_y = 1e9, (jaw2_y + eye_y) / 2
    for p in prof:
        if jaw2_y + 0.01 < p[0] < eye_y - 0.01 and p[1] < mouth_min:
            mouth_min, mouth_y = p[1], p[0]
    brow_y = eye_y + 0.05
    for p in prof:
        if eye_y + 0.005 < p[0] < eye_y + 0.12 and p[1] >= eye_max - 0.02:
            brow_y = p[0]
            break
    if abs(jaw2_y - jaw_y) > 0.02:
        jaw_y = jaw2_y
    print('head features: jaw=%.3f mouth=%.3f eye=%.3f brow=%.3f' % (jaw_y, mouth_y, eye_y, brow_y))

    # 手臂垂放手术（两遍：核心 + 外缘衰减，不拽躯干）
    arm_len = ((hand_x - shoulder_x) ** 2 + (hand_y - shoulder_y) ** 2 + (hand_z - shoulder_z) ** 2) ** 0.5
    shift = shoulder_y - arm_len - ARM_HAND_Y
    print('arm surgery len=%.3f shift=%.3f' % (arm_len, shift))
    for side in (-1, 1):
        sx, sy, sz = side * shoulder_x, shoulder_y, 0.30
        d = [side * (hand_x - shoulder_x) / arm_len, (hand_y - shoulder_y) / arm_len, (hand_z - shoulder_z) / arm_len]
        moved = [False] * len(verts)
        for vi, v in enumerate(verts):
            if v[1] < hand_y - 0.10 or v[1] > shoulder_y + 0.16:
                continue
            if side * v[0] < 0.19:
                continue
            vx, vy, vz = v[0] - sx, v[1] - sy, v[2] - sz
            t = vx * d[0] + vy * d[1] + vz * d[2]
            if t < -0.10 or t > arm_len + 0.12:
                continue
            rx, ry, rz = vx - d[0] * t, vy - d[1] * t, vz - d[2] * t
            rl = (rx * rx + ry * ry + rz * rz) ** 0.5
            if rl > 0.14:
                continue
            b = smooth(clamp((t - 0.06) / 0.22, 0, 1))
            v[0] += (sx + rx - v[0]) * b
            v[1] += (sy - t - shift * b - v[1]) * b
            v[2] += (sz + rz - v[2]) * b
            moved[vi] = True
        for vi, v in enumerate(verts):
            if moved[vi]:
                continue
            if v[1] < hand_y - 0.10 or v[1] > shoulder_y + 0.16:
                continue
            if side * v[0] < 0.19:
                continue
            vx, vy, vz = v[0] - sx, v[1] - sy, v[2] - sz
            t = vx * d[0] + vy * d[1] + vz * d[2]
            if t < -0.10 or t > arm_len + 0.12:
                continue
            rx, ry, rz = vx - d[0] * t, vy - d[1] * t, vz - d[2] * t
            rl = (rx * rx + ry * ry + rz * rz) ** 0.5
            if rl < 0.14 or rl > 0.25:
                continue
            damp = smooth((0.25 - rl) / 0.11)
            b = smooth(clamp((t - 0.06) / 0.22, 0, 1)) * damp
            v[0] += (sx + rx - v[0]) * b
            v[1] += (sy - t - shift * b - v[1]) * b
            v[2] += (sz + rz - v[2]) * b

    # 纵向比例 warp（mesh 关键点 → 立绘关键点；含头部内部五官关键点）
    key = [[0.0, 0.0], [ankle_y, ART['ankle']], [crotch_y, ART['crotch']],
           [shoulder_y, ART['shoulder']], [jaw_y, ART['chin']], [mouth_y, ART['mouth']],
           [eye_y, ART['eye']], [brow_y, ART['brow']], [H, H]]
    key.sort(key=lambda k: k[0])

    def warp_y(y):
        if y <= key[0][0]:
            return key[0][1]
        for i in range(len(key) - 1):
            if key[i][0] <= y <= key[i + 1][0]:
                t = (y - key[i][0]) / max(1e-6, key[i + 1][0] - key[i][0])
                return key[i][1] + t * (key[i + 1][1] - key[i][1])
        return y

    for v in verts:
        v[1] = warp_y(v[1])

    # 脚底平板（封闭足弓，贴合立绘平底军靴）：两脚各 2 三角
    def add_slab(x0, x1, z0, z1):
        b = len(verts)
        verts.extend([[x0, 0.004, z0], [x1, 0.004, z0], [x1, 0.004, z1], [x0, 0.004, z1]])
        tris.extend([[b, b + 1, b + 2], [b, b + 2, b + 3]])

    add_slab(-0.40, -0.08, -0.09, 0.30)
    add_slab(0.08, 0.40, -0.09, 0.30)

    f_asp = (fr['bb']['right'] - fr['bb']['left']) * fr['img']['w'] / ((fr['bb']['bottom'] - fr['bb']['top']) * fr['img']['h'])
    s_asp = (sd['bb']['right'] - sd['bb']['left']) * sd['img']['w'] / ((sd['bb']['bottom'] - sd['bb']['top']) * sd['img']['h'])
    b_asp = (bk['bb']['right'] - bk['bb']['left']) * bk['img']['w'] / ((bk['bb']['bottom'] - bk['bb']['top']) * bk['img']['h'])
    fig_w, fig_d, wb = H * f_asp, H * s_asp, H * b_asp

    def v_of(bb, y):
        return bb['top'] + (1 - y / H) * (bb['bottom'] - bb['top'])

    def row_of(runs, v):
        return clamp(int(v * runs['hn']), 0, runs['hn'] - 1)

    def rows_u(runs):
        return [[[r[0] / runs['wn'], r[1] / runs['wn']] for r in rd] if rd else None for rd in runs['rows']]

    def rows_x(runs, fw):
        return [[[(r[0] / runs['wn'] - 0.5) * fw, (r[1] / runs['wn'] - 0.5) * fw] for r in rd] if rd else None
                for rd in runs['rows']]

    fr_u, fr_x = rows_u(fr['runs']), rows_x(fr['runs'], fig_w)
    sd_u = rows_u(sd['runs'])
    bk_u = [[[1 - r[1], 1 - r[0]] for r in rd] if rd else None for rd in rows_u(bk['runs'])]

    # 粗网格体素化
    cut_y = ART['shoulder']
    x_min_g = min(v[0] for v in verts) - SV
    x_max_g = max(v[0] for v in verts) + SV
    y_min_g, y_max_g = -SV, max(v[1] for v in verts) + SV
    z_min_g = min(v[2] for v in verts) - SV
    z_max_g = max(v[2] for v in verts) + SV
    nx = int((x_max_g - x_min_g) // SV) + 1
    ny = int((y_max_g - y_min_g) // SV) + 1
    nz = int((z_max_g - z_min_g) // SV) + 1
    occ_a = voxelize(verts, tris, SV, x_min_g, y_min_g, z_min_g, nx, ny, nz)
    body = []
    for iz in range(nz):
        for iy in range(ny):
            for ix in range(nx):
                if not occ_a[iz * ny * nx + iy * nx + ix]:
                    continue
                cy = y_min_g + (iy + 0.5) * SV
                if cy >= cut_y:
                    continue
                body.append([ix, iy, iz])
    # 头部细网格
    hx0, hx1, hy0 = -0.24, 0.24, cut_y
    hy1 = max(v[1] for v in verts) + SVH * 2
    hz0 = min(v[2] for v in verts if v[1] > hy0 - 0.1) - SVH * 2
    hz1 = max(v[2] for v in verts if v[1] > hy0 - 0.1) + SVH * 2
    hnx = int((hx1 - hx0) // SVH) + 1
    hny = int((hy1 - hy0) // SVH) + 1
    hnz = int((hz1 - hz0) // SVH) + 1
    occ_b = voxelize(verts, tris, SVH, hx0, hy0, hz0, hnx, hny, hnz)
    head = []
    for iz in range(hnz):
        for iy in range(hny):
            for ix in range(hnx):
                if occ_b[iz * hny * hnx + iy * hnx + ix]:
                    head.append([ix, iy, iz])
    voxels = []
    for ix, iy, iz in body:
        voxels.append({'x': x_min_g + (ix + 0.5) * SV, 'y': y_min_g + (iy + 0.5) * SV,
                       'z': z_min_g + (iz + 0.5) * SV, 'size': SV, 'g': 'A'})
    for ix, iy, iz in head:
        voxels.append({'x': hx0 + (ix + 0.5) * SVH, 'y': hy0 + (iy + 0.5) * SVH,
                       'z': hz0 + (iz + 0.5) * SVH, 'size': SVH, 'g': 'B'})
    print('voxels total=%d body=%d head=%d' % (len(voxels), len(body), len(head)))

    # 剪影 trim
    removed = [False] * len(voxels)
    trim_cnt = 0
    for i, vx in enumerate(voxels):
        v = v_of(fr['bb'], vx['y'])
        if v < 0.012 or v > 0.99:
            continue
        row = fr_x[row_of(fr['runs'], v)]
        if not row:
            continue
        m = vx['size'] * TRIM_MARGIN
        mn = min(r[0] for r in row) - m
        mx = max(r[1] for r in row) + m
        if vx['x'] < mn or vx['x'] > mx:
            removed[i] = True
            trim_cnt += 1
    all_voxels = [vx for i, vx in enumerate(voxels) if not removed[i]]
    print('trim removed=%d' % trim_cnt)

    # 部位标注（同 mjs：|x| 阈值 + 层主分量）
    part_of = ['torso'] * len(all_voxels)
    for i, vx in enumerate(all_voxels):
        if vx['g'] == 'B':
            part_of[i] = 'head' if abs(vx['x']) < 0.20 else ('armL' if vx['x'] < 0 else 'armR')
    by_layer = {}
    for i, vx in enumerate(all_voxels):
        if vx['g'] != 'A':
            continue
        by_layer.setdefault(round(vx['y'] / SV), []).append(i)
    for key, idxs in by_layer.items():
        y_mid = (key + 0.5) * SV
        cols = {}
        for i in idxs:
            cols.setdefault(round(all_voxels[i]['x'] / SV), []).append(i)
        col_list = sorted(cols)
        col_comp = [-1] * len(col_list)
        comp_cols = []
        nc = 0
        for k in range(len(col_list)):
            if col_comp[k] >= 0:
                continue
            stack, members = [k], []
            col_comp[k] = nc
            while stack:
                kk = stack.pop()
                members.append(kk)
                for dd in (1, -1):
                    j = kk + dd
                    if 0 <= j < len(col_list) and col_comp[j] < 0 and abs(col_list[j] - col_list[kk]) == 1:
                        col_comp[j] = nc
                        stack.append(j)
            comp_cols.append(members)
            nc += 1
        comp_cx = [sum(col_list[k] for k in m) / len(m) for m in comp_cols]
        abs_cx = [abs(c) for c in comp_cx]
        main_idx = abs_cx.index(min(abs_cx))
        for k in range(len(col_list)):
            is_main = col_comp[k] == main_idx
            cxw = (col_list[k] + 0.5) * SV
            if y_mid > crotch_y:
                part = 'torso' if is_main else ('armL' if cxw < 0 and abs(cxw) > 0.21 else ('armR' if abs(cxw) > 0.21 else 'torso'))
            elif y_mid > ankle_y:
                part = 'legs' if is_main else ('legL' if cxw < 0 and abs(cxw) > 0.27 else ('legR' if abs(cxw) > 0.27 else 'legs'))
            else:
                part = 'feet' if is_main else ('footL' if cxw < 0 and abs(cxw) > 0.24 else ('footR' if abs(cxw) > 0.24 else 'feet'))
            for i in cols[col_list[k]]:
                part_of[i] = part

    # 面方向
    def faces_for(vx):
        faces = []
        occ = occ_a if vx['g'] == 'A' else occ_b
        nxi, nyi, nzi = (nx, ny, nz) if vx['g'] == 'A' else (hnx, hny, hnz)
        x0i, y0i, z0i = (x_min_g, y_min_g, z_min_g) if vx['g'] == 'A' else (hx0, hy0, hz0)
        svv = vx['size']
        ix, iy, iz = round((vx['x'] - x0i) / svv), round((vx['y'] - y0i) / svv), round((vx['z'] - z0i) / svv)

        def at(a, b, c):
            if a < 0 or a >= nxi or b < 0 or b >= nyi or c < 0 or c >= nzi:
                return 0
            return occ[c * nyi * nxi + b * nxi + a]

        if not at(ix + 1, iy, iz):
            faces.append('+x')
        if not at(ix - 1, iy, iz):
            faces.append('-x')
        if not at(ix, iy + 1, iz):
            faces.append('+y')
        if not at(ix, iy - 1, iz):
            faces.append('-y')
        if not at(ix, iy, iz + 1):
            faces.append('+z')
        if not at(ix, iy, iz - 1):
            faces.append('-z')
        return faces

    for vx in all_voxels:
        vx['faces'] = faces_for(vx)

    # 取色（同 mjs：主面采样 + 部位·v区中位回退 + 方向明暗）
    def zone_of(part, v):
        if part == 'head':
            if v < 0.095:
                return 'hair'
            if v < 0.115:
                return 'forehead'
            if v < 0.14:
                return 'brow'
            if v < 0.16:
                return 'eye'
            if v < 0.185:
                return 'nose'
            if v < 0.195:
                return 'mouth'
            if v < 0.20:
                return 'chin'
            return 'neck'
        if part == 'torso':
            return 'shirt'
        if part in ('armL', 'armR'):
            return 'sleeve' if v < 0.28 else ('forearm' if v < 0.48 else 'fist')
        if part in ('legL', 'legR'):
            return 'pants' if v < 0.92 else 'boot'
        if part in ('footL', 'footR'):
            return 'boot'
        return 'other'

    head_z_ext = {}
    for vx in all_voxels:
        if vx['g'] != 'B':
            continue
        k = round(vx['y'] / SVH)
        e = head_z_ext.setdefault(k, [vx['z'], vx['z']])
        e[0] = min(e[0], vx['z'])
        e[1] = max(e[1], vx['z'])

    def clamp_u(u, row, inset):
        if not row:
            return u
        best, bd = row[0], 1e9
        for r in row:
            d0, d1 = abs(u - r[0]), abs(u - r[1])
            d = min(d0, d1)
            if d < bd:
                bd, best = d, r
        l, r2 = best[0] + inset, best[1] - inset
        if l > r2:
            return (best[0] + best[1]) / 2
        return l if u < l else (r2 if u > r2 else u)

    def side_u(vx):
        vs = v_of(sd['bb'], vx['y'])
        row = sd_u[row_of(sd['runs'], vs)]
        us = vx['z'] / fig_d + 0.5
        if row:
            l = row[0][0] + 2 / sd['runs']['wn']
            r2 = row[0][1] - 2 / sd['runs']['wn']
            if vx['g'] == 'B':
                e = head_z_ext.get(round(vx['y'] / SVH))
                if e and e[1] > e[0] + 1e-6:
                    t = clamp((vx['z'] - e[0]) / (e[1] - e[0]), 0, 1)
                    return row[0][0] + t * (row[0][1] - row[0][0])
            us = (row[0][0] + row[0][1]) / 2 if l > r2 else (l if us < l else (r2 if us > r2 else us))
        return us

    def sample_by_face(vx, face):
        vf = v_of(fr['bb'], vx['y'])
        inset = 2 / fr['runs']['wn']
        if face == '+z':
            u = clamp_u(vx['x'] / fig_w + 0.5, fr_u[row_of(fr['runs'], vf)], inset)
            return sample(fr['img'], fr['bb'], u, vf)
        if face == '-z':
            vb = v_of(bk['bb'], vx['y'])
            ub = clamp_u(0.5 - vx['x'] / wb, bk_u[row_of(bk['runs'], vb)], 2 / bk['runs']['wn'])
            return sample(bk['img'], bk['bb'], ub, vb)
        if face in ('+x', '-x'):
            return sample(sd['img'], sd['bb'], side_u(vx), v_of(sd['bb'], vx['y']))
        u = clamp_u(vx['x'] / fig_w + 0.5, fr_u[row_of(fr['runs'], vf)], inset)
        return sample(fr['img'], fr['bb'], u, vf)

    def sample_alpha(vx, dom, vf):
        img = bk['img'] if dom == '-z' else (sd['img'] if dom in ('+x', '-x') else fr['img'])
        bb = bk['bb'] if dom == '-z' else (sd['bb'] if dom in ('+x', '-x') else fr['bb'])
        uu = 0.5 - vx['x'] / wb if dom == '-z' else (side_u(vx) if dom in ('+x', '-x') else vx['x'] / fig_w + 0.5)
        fx = (bb['left'] + clamp(uu, 0, 1) * (bb['right'] - bb['left'])) * img['w'] - 0.5
        fy = (bb['top'] + vf * (bb['bottom'] - bb['top'])) * img['h'] - 0.5
        p = px_at(img, fx, fy)
        return p[3] if p else 0

    col = [None] * len(all_voxels)
    zone_samples = {}
    for i, vx in enumerate(all_voxels):
        if not vx['faces']:
            continue
        dom = next(p for p in PRIORITY if p in vx['faces'])
        col[i] = sample_by_face(vx, dom)
        vf = v_of(fr['bb'], vx['y'])
        key = part_of[i] + ':' + zone_of(part_of[i], vf)
        if sample_alpha(vx, dom, vf) > 100:
            zone_samples.setdefault(key, []).append(col[i])
    zone_median = {}
    for k, arr in zone_samples.items():
        med = [sorted(a[c] for a in arr)[len(arr) // 2] for c in range(3)]
        zone_median[k] = med

    out = []
    for i, vx in enumerate(all_voxels):
        face_m = 0
        for f in vx['faces']:
            face_m |= FACE_BIT[f]
        key = part_of[i] + ':' + zone_of(part_of[i], v_of(fr['bb'], vx['y']))
        if not vx['faces']:
            rgb = zone_median.get(key, [120, 120, 120])
        else:
            dom = next(p for p in PRIORITY if p in vx['faces'])
            c = col[i]
            vf = v_of(fr['bb'], vx['y'])
            if sample_alpha(vx, dom, vf) <= 100:
                c = zone_median.get(key, c)
            sh = SHADE[dom] * (0.985 + 0.045 * clamp(vf, 0, 1))
            rgb = [clamp(round(x * sh), 0, 255) for x in c]
        out += [round(vx['x'], 3), round(vx['y'], 3), round(vx['z'], 3), rgb[0], rgb[1], rgb[2], face_m, 1 if vx['g'] == 'B' else 0]

    meta = {
        'h': H, 'sv': SV, 'svh': SVH, 'count': len(all_voxels),
        'figW': round(fig_w, 4), 'figD': round(fig_d, 4), 'wb': round(wb, 4),
        'layers': round(H / SV),
        'colors': {k: '#' + ''.join('%02x' % round(c) for c in v) for k, v in zone_median.items()},
    }
    js = ('// voxel_zhenghai_data.js — 郑吒体素模型数据（由 tools/design/gen_voxel_zhenghai_data.py 生成）\n'
          '// 纯颜色体素，无贴图。v 数组每 8 项一个体素：[x, y, z, r, g, b, faceMask, size]\n'
          '//   size：0=身体体素(边长%s) 1=头部细体素(边长%s)\n'
          'window.VoxelZhenghaiData = %s;\n' % (SV, SVH, json.dumps({'meta': meta, 'v': out})))
    with open(OUT, 'w', encoding='utf-8') as f:
        f.write(js)
    print('WROTE %s (%d voxels)' % (OUT, len(all_voxels)))


if __name__ == '__main__':
    main()
