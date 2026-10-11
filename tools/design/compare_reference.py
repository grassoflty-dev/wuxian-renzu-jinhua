"""Compare cutout outputs against the already-good front reference sprite."""
from PIL import Image
import numpy as np

def alpha_stats(path):
    arr = np.array(Image.open(path).convert('RGBA'))
    a = arr[:,:,3]
    return {
        'min': int(a.min()), 'max': int(a.max()),
        'mean': float(a.mean()), 'median': float(np.median(a)),
        'std': float(a.std()),
        'trans_128': (a < 128).mean() * 100,
        'opaque_128': (a >= 128).mean() * 100,
        'edge': ((a > 0) & (a < 128)).mean() * 100,
    }

def rgb_quality(path):
    """Check RGB distribution in opaque area - higher means brighter/more colorful."""
    arr = np.array(Image.open(path).convert('RGBA'))
    mask = arr[:,:,3] >= 128
    r, g, b = arr[mask][:,:3].mean(axis=0)
    med_r, med_g, med_b = np.median(arr[mask][:,:3], axis=0)
    return {'r': round(r,1), 'g': round(g,1), 'b': round(b,1), 
            'med_r': round(med_r,1), 'med_g': round(med_g,1), 'med_b': round(med_b,1)}

print("=== ALPHA DISTRIBUTION COMPARISON ===")
print(f"{'Sprite':<45} {'Trans%':>8} {'Opaque%':>8} {'Edge%':>6} {'A-mean':>8} {'A-std':>7}")
print("-"*90)

sprites = [
    ("pc_zhengzha.png (front ref)", r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\server-rs\ui\assets\img\pc_zhengzha.png"),
    ("pc_zhengzha_side_clean.png", r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side_clean.png"),
    ("pc_zhengzha_back_clean.png", r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back_clean.png"),
]

for name, path in sprites:
    stats = alpha_stats(path)
    print(f"{name:<45} {stats['trans_128']:>7.2f}% {stats['opaque_128']:>7.2f}% {stats['edge']:>5.2f}% {stats['mean']:>7.1f} {stats['std']:>6.1f}")

print("\n=== RGB COLOR COMPARISON (opaque area) ===")
print(f"{'Sprite':<45} {'Mean R':>7} {'Mean G':>7} {'Mean B':>7} {'Med R':>7} {'Med G':>7} {'Med B':>7}")
print("-"*90)

for name, path in sprites:
    colors = rgb_quality(path)
    print(f"{name:<45} {colors['r']:>6.1f} {colors['g']:>6.1f} {colors['b']:>6.1f} {colors['med_r']:>6.1f} {colors['med_g']:>6.1f} {colors['med_b']:>6.1f}")

print("\n=== EDGE PIXEL RGB ANALYSIS ===")
for name, path in sprites:
    arr = np.array(Image.open(path).convert('RGBA'))
    edge_mask = (arr[:,:,3] > 0) & (arr[:,:,3] < 128)
    n_edge = edge_mask.sum()
    if n_edge > 0:
        edge_rgb = arr[edge_mask][:,:3]
        print(f"\n{name}:")
        print(f"  Edge pixels: {n_edge:,}")
        print(f"  Edge mean RGB: ({edge_rgb[:,0].mean():.1f}, {edge_rgb[:,1].mean():.1f}, {edge_rgb[:,2].mean():.1f})")
        brightness = edge_rgb.mean(axis=-1)
        dark_edges = (brightness < 15).sum() / n_edge * 100
        light_edges = (brightness > 200).sum() / n_edge * 100
        print(f"  Dark edges (<15 bright): {dark_edges:.1f}%  |  Light edges (>200 bright): {light_edges:.1f}%")
        
        if dark_edges > 50:
            print(f"  [NOTE] Many edge pixels are dark - could indicate dark halo residue")
        elif light_edges > 50:
            print(f"  [NOTE] Many edge pixels are light - could indicate white rim light")
        else:
            print(f"  [OK] Edge pixels are mixed gray tones (anti-aliasing only)")
