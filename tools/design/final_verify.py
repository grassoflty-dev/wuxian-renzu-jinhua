"""Final verification of selected v25 outputs."""
from PIL import Image
import numpy as np

base = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy"

for sp in ["side", "back"]:
    path = f"{base}\\pc_zhengzha_{sp}_clean.png"
    arr = np.array(Image.open(path).convert('RGBA'))
    h, w = arr.shape[:2]
    
    n_trans = int((arr[:,:,3]==0).sum())
    n_opaque = int((arr[:,:,3]==255).sum())
    n_edge = int(((arr[:,:,3]>0) & (arr[:,:,3]<255)).sum())
    
    print(f"\n{'='*50}")
    print(f"{sp.upper()} clean.png")
    print(f"{'='*50}")
    print(f"  Transparent (alpha=0): {n_trans:,} ({n_trans/(h*w)*100:.1f}%)")
    print(f"  Opaque (alpha=255):    {n_opaque:,} ({n_opaque/(h*w)*100:.1f}%)")
    print(f"  Semi-alpha edge:       {n_edge} (should be 0)")
    
    # Check for dark residue
    if n_edge == 0:
        print("  [PASS] All pixels are either fully transparent or opaque")
    
    # Bounding box
    coords = np.column_stack(np.where(arr[:,:,3] >= 128))
    if len(coords) > 0:
        c_min = coords.min(axis=0)
        c_max = coords.max(axis=0)
        print(f"  BBox: ({c_min[0]},{c_min[1]}) to ({c_max[0]},{c_max[1]})")
        print(f"  Size: {c_max[0]-c_min[0]+1}x{c_max[1]-c_min[1]+1}px")
        
        # Stray check
        above = (arr[:c_min[1], :, 3] >= 128).sum()
        below = (arr[c_max[1]+1:, :, 3] >= 128).sum()
        left = (arr[:, :c_min[0], 3] >= 128).sum()
        right = (arr[:, c_max[0]+1:, 3] >= 128).sum()
        total_stray = above + below + left + right
        status = "PASS" if total_stray == 0 else "FAIL"
        print(f"  Stragglers outside bbox: {total_stray} [{status}]")

# Compare with front reference
front_path = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\server-rs\ui\assets\img\pc_zhengzha.png"
front_arr = np.array(Image.open(front_path).convert('RGBA'))
n_front_body = int((front_arr[:,:,3]==255).sum())
print(f"\nFront ref body pixels: {n_front_body:,} ({n_front_body/(768*1024)*100:.1f}%)")

# Summary ratios
for sp in ["side", "back"]:
    path = f"{base}\\pc_zhengzha_{sp}_clean.png"
    arr = np.array(Image.open(path).convert('RGBA'))
    n_sp = int((arr[:,:,3]==255).sum())
    ratio = n_sp / n_front_body * 100
    label = "reasonable" if ratio > 40 else "too_small"
    print(f"  {sp}: {n_sp:,} px = {ratio:.0f}% of front ref size [{label}]")
