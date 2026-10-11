"""Pixel analysis of Zhengzha character sprites — edge structure diagnosis."""
from PIL import Image
import numpy as np

def analyze(path):
    img = Image.open(path).convert('RGBA')
    arr = np.array(img)
    h, w, _ = arr.shape
    print(f"\n{'='*60}")
    print(f"File: {path}")
    print(f"Size: {w}x{h}, Shape: {arr.shape}")
    
    # Separate channels
    r, g, b, a = arr[:,:,0], arr[:,:,1], arr[:,:,2], arr[:,:,3]
    
    # Overall stats
    mask_bg = a == 0
    mask_fg = a > 128
    mask_edge = (a > 0) & (a < 255)
    
    if np.any(mask_fg):
        fg_r, fg_g, fg_b = r[mask_fg].mean(), g[mask_fg].mean(), b[mask_fg].mean()
        print(f"Foreground (a>128): R={fg_r:.1f} G={fg_g:.1f} B={fg_b:.1f}")
        med = np.median(arr[mask_fg][:,:3])
        print(f"  Median RGB value: {med:.1f}")
    
    if np.any(mask_bg):
        bg_r, bg_g, bg_b = r[mask_bg].mean(), g[mask_bg].mean(), b[mask_bg].mean()
        print(f"Background (a=0):  R={bg_r:.1f} G={bg_g:.1f} B={bg_b:.1f}")
    
    if np.any(mask_edge):
        er, eg, eb = r[mask_edge].mean(), g[mask_edge].mean(), b[mask_edge].mean()
        print(f"Edge pixels (0<a<255): R={er:.1f} G={eg:.1f} B={eb:.1f}, count={mask_edge.sum()}")
    
    # Edge column scan: look at alpha values going inward from each side
    print("\nAlpha bands along horizontal center line:")
    cy = h // 2
    row_a = a[cy, :]
    # Find transitions
    transitions = []
    for x in range(1, w):
        da = int(row_a[x]) - int(row_a[x-1])
        if abs(da) > 10:
            transitions.append((x, int(row_a[x-1]), int(row_a[x]), da))
    
    if len(transitions) >= 4:
        for i in range(0, len(transitions)-1, 2):
            if i+1 < len(transitions):
                t1, t2 = transitions[i], transitions[i+1]
                left_x = t1[0]
                right_x = t2[0]
                mid_alpha = int((row_a[left_x] + row_a[right_x]) / 2)
                # Sample RGB at various alpha levels going inward
                sample_points = [left_x + int((right_x-left_x)*p/10) for p in range(0, 11, 2)]
                vals = [(int(x), int(arr[cy, x, :3].tolist())) for x in sample_points]
                print(f"  Left edge ~{left_x}: {vals}")
                break
    
    # More detailed: scan top-to-bottom at center x
    cx = w // 2
    col_a = a[:, cx]
    print(f"\nAlpha band vertical at center x={cx}:")
    for y in [cy - 100, cy - 50, cy, cy + 50, cy + 100]:
        if 0 <= y < h:
            print(f"  y={y}: A={int(col_a[y])}, RGB=({arr[y,cx,0]}, {arr[y,cx,1]}, {arr[y,cx,2]})")
    
    # Key analysis: find regions where we go from dark background through edge layers to foreground
    # Check a few points along the rightmost edge transition
    print("\n--- Edge layer analysis ---")
    if len(transitions) >= 4:
        # Get left-edge region
        for i in range(0, len(transitions)-1, 2):
            t_enter = transitions[i]
            t_exit = transitions[i+1]
            enter_x, exit_x = t_enter[0], t_exit[0]
            
            if exit_x - enter_x > 3:  # Only if there's an actual edge region
                print(f"\nLeft-side edge zone: x={enter_x} to {exit_x} ({exit_x-enter_x}px wide)")
                print("Rows (alpha -> RGB):")
                for dy in [-30, -15, 0, 15, 30]:
                    yy = min(max(cy + dy, 0), h-1)
                    row = a[yy, :]
                    seg = row[enter_x:exit_x+1]
                    rgb_seg = arr[yy, enter_x:exit_x+1, :3].astype(float)
                    if len(seg) > 0 and seg.max() > 0:
                        avg_rgb = rgb_seg.mean(axis=0)
                        print(f"  y_offset+{dy:+d}: alpha_vals={[int(v) for v in seg[:5]]}..., RGB_avg=({avg_rgb[0]:.0f},{avg_rgb[1]:.0f},{avg_rgb[2]:.0f})")
                break
    
    # Check how many distinct "dark ring" layers exist
    print("\n--- Checking for dark ring layers at boundary ---")
    # Get bounding box of visible pixels
    coords = np.column_stack(np.where(a > 0))
    if len(coords) > 0:
        min_y, min_x = coords.min(axis=0)
        max_y, max_x = coords.max(axis=0)
        print(f"BBox: ({min_x},{min_y}) to ({max_x},{max_y}), size {max_x-min_x+1}x{max_y-min_y+1}")
        
        # Look at 5px-wide bands just inside the bbox
        for offset in [0, 2, 5, 8, 12, 15]:
            if min_x + offset < max_x:
                test_r = a[max_y-offset, min_x:min_x+20].mean()
                test_g = a[max_y-offset, min_x:min_x+20].mean() 
                rgb_mean = arr[max_y-offset, min_x:min_x+20, :3].mean()
                print(f"  {offset}px inside bottom edge: meanA={test_r:.0f}, meanRGB={rgb_mean:.0f}")

    return arr


# Analyze all three images
side_arr = analyze(r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side.png")
back_arr = analyze(r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back.png")

print("\n\n========== COMPARISON ========== ")
print("If back median is much lower than side, the back sprite is inherently darker.")
