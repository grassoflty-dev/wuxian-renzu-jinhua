# organize_portraits.py — 把人物/怪物立绘按实体整理成独立文件夹
# 用法: python organize_portraits.py [--preview|--copy]
# 分组规则:
#   1. 剥离变体后缀(_cut/_final/_clean/_v\d+/_c\d+/_r\d+/_wan\d+/_on_xxx/_peeled/_bright/_rim/_seal 等)
#   2. 剥离类型前缀(enemy_/pc_/npc_/boss_/item_/gear_/bl_/tr_/scene_) 归一化主名, 便于 raw 无前缀文件归组
#   3. 排除背景/场景/纹理类(纯 bg/scene/tex/map 或 *_bg*)
# 输出: portraits_library/<实体>/<原文件名>
import os, re, shutil, sys

ROOT = r'C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1'
IMG = os.path.join(ROOT, 'server-rs', 'ui', 'assets', 'img')
RAW = os.path.join(ROOT, 'tools', 'design', 'raw_enemy')
PARTS = os.path.join(RAW, 'parts')
OUT = os.path.join(ROOT, 'portraits_library')

KNOWN_SUFFIX = {'cut', 'final', 'clean', 'peeled', 'bright', 'norim', 'rim', 'seal', 'fix',
                'edge', 'core', 'back', 'side', 'front', 'face', 'sprite', 'show', 'preview',
                'test', 'probe', 'on_black', 'on_gray', 'on_white', 'c', 'r', 'wan', 'v', 'n'}

def strip_suffix(name):
    """稳健循环剥离: 支持复合后缀(_c10cut/_on_black/_oblique45) + 普通 _word<数字>"""
    base = name
    if base.lower().endswith('.png'):
        base = base[:-4]
    while True:
        # 复合1: _c10cut / _c10cut4n / _v25clean 等 数字+动作词粘连
        m = re.search(r'_(c|r|v)\d+(cut|final|clean|fix|seal)[a-z0-9]*$', base)
        if m:
            base = base[:m.start()]
            continue
        # 复合2: _on_black / _on_gray / _on_white
        m = re.search(r'_on_(black|gray|white)$', base)
        if m:
            base = base[:m.start()]
            continue
        # 复合3: _oblique45 / _oblique
        m = re.search(r'_oblique\d*$', base)
        if m:
            base = base[:m.start()]
            continue
        # 普通: _word 或 _word<数字>
        m = re.search(r'_([a-z]+)(\d*)$', base)
        if m and m.group(1) in KNOWN_SUFFIX:
            base = base[:m.start()]
            continue
        break
    return base

PREFIX = re.compile(r'^(enemy|pc|npc|boss|item|gear|bl|tr|scene|img)_', re.I)

# 拼写别名 → 正确实体主名 (pc_wan* = 郑吒的 wan 生成版本; zhengha 拼写少一个 z)
ALIAS = {'zhengha': 'zhengzha', 'pc': 'zhengzha'}

def strip_prefix(name):
    m = PREFIX.match(name)
    return name[m.end():] if m else name

def is_background(name):
    b = strip_prefix(strip_suffix(name))
    if b in ('bg', 'scene', 'tex', 'map', 'icon', 'title', 'grain', 'logo'):
        return True
    if re.search(r'_bg($|_)', name, re.I) or re.match(r'^bg_', name, re.I):
        return True
    if re.search(r'_(bg|scene|backdrop)\.', name, re.I):
        return True
    return False

def collect(d):
    out = []
    if not os.path.isdir(d):
        return out
    for fn in sorted(os.listdir(d)):
        if fn.lower().endswith('.png') and not fn.startswith('~'):
            out.append(os.path.join(d, fn))
    return out

files = collect(IMG) + collect(RAW) + collect(PARTS)
groups = {}   # 主名 -> {原名 -> 路径}
for fp in files:
    name = os.path.basename(fp)
    stem = strip_suffix(name)
    if is_background(stem):
        continue
    main = strip_prefix(stem)  # 归一化主名
    main = ALIAS.get(main, main)
    groups.setdefault(main, {})[name] = fp

# 展示
items = sorted(groups.items())
print('实体总数: %d' % len(items))
multi = [(k, v) for k, v in items if len(v) > 1]
print('多文件实体 %d 个:' % len(multi))
for k, v in multi:
    print('  [%s] (%d):' % (k, len(v)))
    for n in sorted(v):
        print('      %s' % n)

if '--preview' in sys.argv or len(sys.argv) < 2:
    print('\n(预览模式, 用 --copy 执行复制)')
    sys.exit(0)

# 执行复制
os.makedirs(OUT, exist_ok=True)
for main, v in items:
    d = os.path.join(OUT, main)
    os.makedirs(d, exist_ok=True)
    for name, fp in v.items():
        shutil.copy2(fp, os.path.join(d, name))
print('DONE: %d 实体 → %s (%d 文件)' % (len(items), OUT, sum(len(v) for v in groups.values())))
