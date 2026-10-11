# scan_portraits.py — 扫描立绘并按实体分组(预览, 不复制)
from PIL import Image
import numpy as np
import os, re

IMG = r'C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\server-rs\ui\assets\img'
RAW = r'C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy'

SUFFIX = re.compile(r'(_cut\d*|_final\d*|_clean\d*|_v\d+|_c\d+|_rim\d*|_peeled|_bright|_fix\d*|_seal\d+|_on_(black|gray|white)|_norim|_cut)$', re.I)

def entity(name):
    """从文件名提取实体名(去掉后缀变体)"""
    base = name
    while True:
        m = SUFFIX.search(base)
        if not m:
            break
        base = base[:m.start()]
    return base

def scan(d, skip_common=True):
    groups = {}
    for fn in sorted(os.listdir(d)):
        if not fn.lower().endswith('.png'):
            continue
        ent = entity(fn)
        # 排除背景/场景图(非人物/怪物)
        if re.match(r'^(bg|scene|map|tex|icon|title|grain)_', ent, re.I):
            continue
        groups.setdefault(ent, []).append(fn)
    return groups

for label, d in [('assets/img', IMG), ('raw_enemy', RAW)]:
    g = scan(d)
    print('=== %s: %d 实体, %d 文件 ===' % (label, len(g), sum(len(v) for v in g.values())))
    # 多文件实体(有变体的)
    multi = {k: v for k, v in g.items() if len(v) > 1}
    print('多文件实体 %d 个:' % len(multi))
    for k in sorted(multi)[:60]:
        print('  %s (%d): %s' % (k, len(multi[k]), ', '.join(multi[k])))
    singles = [k for k, v in g.items() if len(v) == 1]
    print('单文件实体 %d 个(示例前20): %s' % (len(singles), ', '.join(sorted(singles)[:20])))
    print()
