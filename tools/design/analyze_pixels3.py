"""Check: is this a solid RGBA with no actual transparency?"""
from PIL import Image
import numpy as np

def quick_check(path):
    img = Image.open(path)
    print(f"\n{'='*60}")
    print(f"File: {path}")
    print(f"Mode: {img.mode}, Size: {img.size}")
    
    arr = np.array(img)
    a = arr[:,:,3] if arr.shape[2] == 4 else np.full(arr.shape[:2], 255, dtype=np.uint8)
    
    # Is there ANY transparency?
    print(f"Alpha min={a.min()}, max={a.max()}")
    unique_a = np.unique(a)
    print(f"Unique alpha values ({len(unique_a)}): {unique_a[:30]}...")
    
    # If mode is RGB (no alpha channel at all):
    if arr.shape[2] == 3:
        print(">>> IMAGE IS RGB MODE - NO ALPHA CHANNEL AT ALL")
        print(">>> The 'black background' was never meant to be transparent")
        return
    
    # Check brightness distribution
    r, g, b = arr[:,:,0].astype(float), arr[:,:,1].astype(float), arr[:,:,2].astype(float)
    brightness = (r + g + b) / 3
    
    # What percentage is essentially black (brightness < 20)?
    black_px = (brightness < 20).sum()
    total = brightness.size
    print(f"Black-ish pixels (bright < 20): {black_px:,} ({black_px/total*100:.1f}%)")
    
    # Count by brightness ranges
    for lo in [0, 10, 20, 30, 50, 70, 100, 150, 200]:
        hi = lo + 10
        mask = (brightness >= lo) & (brightness < hi)
        cnt = mask.sum()
        if cnt > 0:
            alpha_avg = a[mask].mean()
            print(f"  Bright {lo}-{hi}: {cnt:>7,} px, mean alpha={alpha_avg:.0f}")
    
    # Find non-black pixels - these ARE the character
    char_mask = brightness >= 30
    print(f"\nCharacter pixels (brightness >= 30): {char_mask.sum():,} ({char_mask.mean()*100:.1f}%)")
    
    if char_mask.any():
        char_r, char_g, char_b = r[char_mask].mean(), g[char_mask].mean(), b[char_mask].mean()
        print(f"  Character avg RGB: ({char_r:.1f}, {char_g:.1f}, {char_b:.1f})")
        
        # Brightness histogram of CHARACTER pixels only
        cb = brightness[char_mask]
        for lo in [30, 50, 70, 90, 120, 160, 200, 240]:
            mask_c = (cb >= lo) & (cb < lo+30)
            cnt = mask_c.sum()
            if cnt > 0:
                pct = cnt / char_mask.sum() * 100
                print(f"    Char bright {lo}-{lo+30}: {pct:.1f}%")

quick_check(r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side.png")
quick_check(r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back.png")
quick_check(r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\server-rs\ui\assets\img\pc_zhengzha.png")
