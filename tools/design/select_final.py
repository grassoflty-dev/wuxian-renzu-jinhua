"""Select best variant and produce final clean outputs."""
import numpy as np
from PIL import Image

base = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy"

# Compare opaque pixel counts across thresholds
import os
for sp in ["side", "back"]:
    print(f"\n{sp.upper()}:")
    for t in [20, 25, 30, 35, 40]:
        path = f"{base}\\pc_zhengzha_{sp}_v{t}.png"
        arr = np.array(Image.open(path).convert('RGBA'))
        n_body = int((arr[:,:,3]==255).sum())
        n_trans = int((arr[:,:,3]==0).sum())
        pct = n_body / (768*1024) * 100
        print(f"  V={t:>2d}: {n_body:>7,} px ({pct:.1f}%)")

# Select: v25 seems good — retains most body while reducing edge pixels
# Copy selected to _clean.png
for sp, t in [("side", 25), ("back", 25)]:
    src = f"{base}\\pc_zhengzha_{sp}_v{t}.png"
    dst = f"{base}\\pc_zhengzha_{sp}_clean.png"
    
    # Read and verify
    orig = np.array(Image.open(src).convert('RGBA'))
    n_body = int((orig[:,:,3]==255).sum())
    n_edge = int(((orig[:,:,3]>0) & (orig[:,:,3]<255)).sum())
    
    # Defringe check: all pixels should be either fully transparent or fully opaque
    assert n_edge == 0, f"Found {n_edge} edge pixels!"
    
    # Copy to final
    img = Image.open(src).convert('RGBA')
    img.save(dst, 'PNG')
    print(f"\n[+] Copied {sp}_v{t} -> {sp}_clean.png ({n_body:,} opaque px, 0 edge pixels)")

print("\nFinal outputs ready for visual QC:")
print(f"  {base}\\pc_zhengzha_side_clean.png")
print(f"  {base}\\pc_zhengzha_back_clean.png")
