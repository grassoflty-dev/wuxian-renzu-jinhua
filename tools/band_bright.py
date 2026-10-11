# -*- coding: utf-8 -*-
"""模拟 zone2d 的像素管线（提亮+22 / 对比 1.22），比较上身区域与腿部贴图的亮度分布。"""
import sys
import numpy as np
from PIL import Image

def pipeline_band(arr):
    a = arr.astype(np.float32)
    al = a[:,:,3]
    m = al > 128
    rgb = a[:,:,:3]
    avg = rgb.mean(axis=2, keepdims=True)
    out = np.clip(avg + (rgb - avg)*1.22 + 22, 0, 255)
    return out[m]

port = np.array(Image.open(sys.argv[1]).convert("RGBA"))
legs = np.array(Image.open(sys.argv[2]).convert("RGBA"))
# 上身：立绘顶部 54%
cut = int(port.shape[0]*0.54)
up = pipeline_band(port[:cut])
lo = pipeline_band(legs)
for name, v in (("upper(torso)", up), ("legs sprite", lo)):
    print("%-14s n=%6d mean=%6.1f p10=%5.1f p50=%5.1f p90=%5.1f" % (
        name, v.size, v.mean(), np.percentile(v,10), np.percentile(v,50), np.percentile(v,90)))
print("比值 legs/upper mean = %.2f" % (lo.mean()/up.mean()))
