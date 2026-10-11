# -*- coding: utf-8 -*-
import os
import numpy as np
from PIL import Image

d = r'server-rs\ui\assets\img'
rows = []
for f in sorted(os.listdir(d)):
    if not (f.startswith('item_wp_') and f.endswith('.png')):
        continue
    p = os.path.join(d, f)
    try:
        a = np.asarray(Image.open(p).convert('RGBA')).astype(np.int32)
    except Exception as e:
        rows.append((f, 'ERR', str(e), None, None))
        continue
    al = a[..., 3]
    h, w = a.shape[:2]
    opaque = (al >= 250).sum() / al.size * 100
    transparent = (al <= 5).sum() / al.size * 100
    corners = [tuple(a[3, 3, :3]), tuple(a[3, w - 4, :3]),
               tuple(a[h - 4, 3, :3]), tuple(a[h - 4, w - 4, :3])]
    same = len(set(corners)) == 1
    rows.append((f, opaque, transparent, same, corners[0]))

print('%-30s %8s %8s %5s %s' % ('file', 'opaque%', 'trans%', 'same', 'corner'))
for r in rows:
    print('%-30s %8s %8s %5s %s' % (r[0], r[1], r[2], r[3], r[4]))
