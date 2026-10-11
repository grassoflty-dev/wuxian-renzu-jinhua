"""Deep pixel analysis: histogram + alpha distribution."""
from PIL import Image
import numpy as np
from collections import Counter

def deep_analyze(path):
    img = Image.open(path).convert('RGBA')
    arr = np.array(img)
    h, w, _ = arr.shape
    r, g, b, a = arr[:,:,0], arr[:,:,1], arr[:,:,2], arr[:,:,3]
    
    print(f"\n{'='*70}")
    print(f"File: {path}")
    
    # Alpha histogram (buckets of 32)
    abuckets = [(i*32, min((i+1)*32-1, 255)) for i in range(8)]
    print("\nAlpha histogram:")
    total_px = h * w
    for lo, hi in abuckets:
        mask = (a >= lo) & (a <= hi)
        cnt = mask.sum()
        pct = cnt / total_px * 100
        avg_rgb = arr[mask][:,:3].mean(axis=(0,1)).round(1) if cnt > 0 else [0,0,0]
        print(f"  A={lo}-{hi:3d}: {cnt:>8d} ({pct:>6.2f}%)  meanRGB=({avg_rgb[0]:>5}, {avg_rgb[1]:>5}, {avg_rgb[2]:>5})")
    
    # Find the dominant "black" vs "character" separation
    # Look at brightness = mean(RGB)
    brightness = (r.astype(float) + g.astype(float) + b.astype(float)) / 3
    
    # How much of the image is pure black vs non-black?
    pure_black = (r < 5) & (g < 5) & (b < 5)
    print(f"\nPure black pixels (R,G,B < 5): {pure_black.sum():,} ({pure_black.mean()*100:.1f}%)")
    
    # Brightness histogram
    buckets_b = [5*i for i in range(20)]  # 0-100 brightness bins
    print("\nBrightness histogram (0-100 range):")
    for i, (lo, hi) in enumerate(zip(buckets_b[:-1], buckets_b[1:])):
        mask = (brightness >= lo) & (brightness < hi)
        cnt = mask.sum()
        pct = cnt / total_px * 100
        alpha_avg = a[mask].mean() if cnt > 0 else 0
        print(f"  Bright {lo:3d}-{hi:3d}: {cnt:>8d} ({pct:>6.2f}%)  meanA={alpha_avg:.0f}")
    
    # Check corner regions - are they all black?
    margin = 50
    corners = {
        'top-left': arr[:margin, :margin],
        'top-right': arr[:margin, -margin:],
        'bottom-left': arr[-margin:, :margin],
        'bottom-right': arr[-margin:, -margin:]
    }
    print("\nCorner regions (first/last 50px):")
    for name, patch in corners.items():
        pa = patch[:,:,3]
        pr, pg, pb = patch[:,:,0].mean(), patch[:,:,1].mean(), patch[:,:,2].mean()
        transparent = (pa < 128).sum() / pa.size * 100
        print(f"  {name}: meanA={pa.mean():.0f}, meanRGB=({pr:.0f},{pg:.0f},{pb:.0f}), transparent%={transparent:.1f}")
    
    # Is the image basically ALL black with the character drawn ON TOP?
    # Or does it have proper transparency?
    total_transparent = (a < 128).sum()
    total_opaque = (a > 200).sum()
    total_edge = ((a >= 128) & (a <= 200)).sum()
    print(f"\nTransparency breakdown:")
    print(f"  Transparent (a<128): {total_transparent:,} ({total_transparent/total_px*100:.1f}%)")
    print(f"  Opaque (a>200):      {total_opaque:,} ({total_opaque/total_px*100:.1f}%)")  
    print(f"  Transition (128<=a<=200): {total_edge:,} ({total_edge/total_px*100:.1f}%)")
    
    # Sample scanline at row 100 (likely top of character head or empty space)
    print(f"\nSample horizontal scanline at y=100 (first 30px):")
    for x in range(min(30, w)):
        print(f"  x={x}: A={int(a[100,x])}, RGB=({int(r[100,x])},{int(g[100,x])},{int(b[100,x])})")
    
    return arr


# Analyze both originals
side_arr = deep_analyze(r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side.png")
back_arr = deep_analyze(r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back.png")

# Also check the reference front sprite
deep_analyze(r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\server-rs\ui\assets\img\pc_zhengzha.png")
