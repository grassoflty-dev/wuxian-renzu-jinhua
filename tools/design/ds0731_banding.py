# -*- coding: utf-8 -*-
"""ds0731_banding.py — 检测渲染图中的周期性横向条纹(百叶窗/色带)。
用法: python ds0731_banding.py <png> [x0 x1 y0 y1]
对指定区域(默认全图模型区)做逐行亮度谱分析: 计算每行平均亮度, 对行序列做
自相关/FFT, 报告是否存在显著周期分量。"""
import sys
from PIL import Image
import numpy as np

def main():
    f = sys.argv[1]
    im = Image.open(f).convert('RGB')
    a = np.asarray(im).astype(float)
    H, W = a.shape[:2]
    gray = a.mean(axis=2)
    if len(sys.argv) >= 6:
        x0, x1, y0, y1 = map(int, sys.argv[2:6])
    else:
        # 模型区(非白)
        mx = a.max(axis=2)
        model = mx < 246
        ys, xs = np.where(model)
        if len(ys) == 0:
            print('no model'); return
        x0, x1 = xs.min(), xs.max(); y0, y1 = ys.min(), ys.max()
    region = gray[y0:y1, x0:x1]
    rowmean = region.mean(axis=1)
    n = len(rowmean)
    if n < 16:
        print('region too small'); return
    # 去趋势
    x = np.arange(n)
    trend = np.polyval(np.polyfit(x, rowmean, 2), x)
    det = rowmean - trend
    # 自相关找周期
    det0 = det - det.mean()
    ac = np.correlate(det0, det0, 'full')[n - 1:]
    ac = ac / (ac[0] + 1e-9)
    # 找 3..n/2 范围内的显著峰
    best = (0, 0)
    for p in range(3, n // 2):
        if ac[p] > best[1]:
            best = (p, ac[p])
    # FFT 峰值
    spec = np.abs(np.fft.rfft(det0))
    freqs = np.fft.rfftfreq(n)
    nonz = freqs > 0
    idx = np.argmax(spec[nonz])
    dom_freq = freqs[nonz][idx]
    dom_period = 1 / dom_freq if dom_freq > 0 else 0
    dom_power = spec[nonz][idx] / (spec.sum() + 1e-9)
    print(f'{f.split(chr(92))[-1].split("/")[-1]}: region x[{x0},{x1}] y[{y0},{y1}]')
    print(f'  rowmean: mean {rowmean.mean():.1f} std {rowmean.std():.1f}')
    print(f'  autocorr peak: period {best[0]}px corr {best[1]:.3f}  ({"!! 显著周期(条纹)" if best[1] > 0.25 and best[0] >= 3 else "无明显周期"})')
    print(f'  FFT dominant: period {dom_period:.1f}px power-share {dom_power * 100:.1f}%  ({"!! 强周期分量" if dom_power > 0.15 and dom_period < n / 2 else "OK"})')
    # 中段行亮度曲线采样, 供人工核对
    for frac in [0.25, 0.5, 0.75]:
        yy = y0 + int((y1 - y0) * frac)
        seg = gray[yy, x0:x1]
        print(f'  row y={yy}: mean {seg.mean():.1f}, min {seg.min():.0f}, max {seg.max():.0f}')

if __name__ == '__main__':
    main()
