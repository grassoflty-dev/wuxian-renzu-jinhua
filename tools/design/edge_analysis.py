"""Comprehensive edge quality analysis."""
import sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
from PIL import Image
import numpy as np

def analyze_edge_quality(orig_path, clean_path, label):
    print(f"\n{'='*70}")
    print(f"EDGE QUALITY ANALYSIS: {label}")
    print(f"{'='*70}")
    
    orig = np.array(Image.open(orig_path).convert('RGBA'))
    clean = np.array(Image.open(clean_path).convert('RGBA'))
    h, w, _ = clean.shape
    
    oa, ca = orig[:,:,3], clean[:,:,3]
    orgb, crgb = orig[:,:,:3].astype(float), clean[:,:,:3].astype(float)
    
    # 1. Body sizing
    orig_coords = np.column_stack(np.where(oa >= 200))
    clean_coords = np.column_stack(np.where(ca >= 200))
    
    shrink_info = {}
    if len(orig_coords) > 0:
        o_min_y, o_min_x = orig_coords.min(axis=0)
        o_max_y, o_max_x = orig_coords.max(axis=0)
        o_w, o_h = o_max_x - o_min_x + 1, o_max_y - o_min_y + 1
        
        print(f"\n--- Body Sizing ---")
        print(f"  Original bbox: ({o_min_x},{o_min_y})-({o_max_x},{o_max_y}), {o_w}x{o_h}px")
        
        shrink_info['o_min_x'] = int(o_min_x)
        shrink_info['o_max_x'] = int(o_max_x)
        shrink_info['o_min_y'] = int(o_min_y)
        shrink_info['o_max_y'] = int(o_max_y)
        
    if len(clean_coords) > 0:
        c_min_y, c_min_x = clean_coords.min(axis=0)
        c_max_y, c_max_x = clean_coords.max(axis=0)
        c_w, c_h = c_max_x - c_min_x + 1, c_max_y - c_min_y + 1
        print(f"  Clean bbox:    ({c_min_x},{c_min_y})-({c_max_x},{c_max_y}), {c_w}x{c_h}px")
        
        shrink_info['c_min_x'] = int(c_min_x)
        shrink_info['c_max_x'] = int(c_max_x)
        shrink_info['c_min_y'] = int(c_min_y)
        shrink_info['c_max_y'] = int(c_max_y)
        
        dx_l = o_min_x - c_min_x
        dx_r = o_max_x - c_max_x
        dx_t = o_min_y - c_min_y
        dx_b = c_max_y - o_max_y
        
        print(f"  Shrunk by: left={dx_l:+d}, right={dx_r:+d}, top={dx_t:+d}, bottom={dx_b:+d} px")
        total_shrink = abs(dx_l) + abs(dx_r) + abs(dx_t) + abs(dx_b)
        print(f"  Total shrink: {total_shrink}px across all edges")
        if total_shrink < 10:
            print("[PASS] Minimal body shrinkage")
        elif total_shrink < 30:
            print("[WARN] Minor: Some edge tightening detected")
        else:
            print("[FAIL] Significant body shrinkage")
            
        shrink_info['total_shrink'] = int(total_shrink)
    
    # 2. Edge artifact detection
    total_stragglers = [0]
    print(f"\n--- Edge Artifact Detection ---")
    
    if 'c_min_y' in shrink_info:
        c_min_x = shrink_info['c_min_x']
        c_max_x = shrink_info['c_max_x']
        c_min_y = shrink_info['c_min_y']
        c_max_y = shrink_info['c_max_y']
        
        for margin_label, y1, y2, x1, x2 in [
            ("top", 0, c_min_y, 0, w),
            ("bottom", c_max_y, h, 0, w),
            ("left", c_min_y, c_max_y+1, 0, c_min_x),
            ("right", c_min_y, c_max_y+1, c_max_x, w),
        ]:
            region_a = ca[max(y1,0):min(y2,h), max(x1,0):min(x2,w)]
            opaque_in_region = (region_a >= 64).sum()
            dark_sub_pixel = ((region_a > 0) & (region_a < 64)).sum()
            total_stragglers[0] += opaque_in_region
            
            status = "CLEAN" if opaque_in_region == 0 else f"{opaque_in_region} stragglers"
            if dark_sub_pixel > 0:
                status += f" + {dark_sub_pixel} sub-pixel"
            print(f"  {margin_label:>8s}: alpha>=64={opaque_in_region:>5d}, 0<a<64={dark_sub_pixel:>5d} -> [{status}]")
    
    total_stray = total_stragglers[0]
    
    # 3. Alpha matte smoothness
    edge_mask = (ca > 0) & (ca < 128)
    n_edge = int(edge_mask.sum())
    
    print(f"\n--- Matte Smoothness ---")
    if n_edge > 0:
        edge_values = ca[edge_mask]
        print(f"  Edge pixel count: {n_edge:,} ({n_edge/(h*w)*100:.3f}%)")
        print(f"  Alpha range: [{edge_values.min()}, {edge_values.max()}]")
        print(f"  Alpha std: {edge_values.std():.1f}")
        print(f"  Alpha mean: {edge_values.mean():.1f}")
        
        low_alpha_pct = float(((edge_values < 64).sum()) / n_edge * 100)
        print(f"  Low-alpha pixels (<64): {low_alpha_pct:.1f}%")
        
        if edge_values.std() < 15 and low_alpha_pct > 90:
            print("[PASS] Smooth anti-aliasing edge (tight distribution)")
        elif edge_values.std() < 40:
            print("[MODERATE] Slightly wider edge transition")
        else:
            print("[WORRY] Wide alpha distribution suggests inconsistent matte")
        
        matte_info = {'std': float(edge_values.std()), 'low_pct': low_alpha_pct}
    else:
        matte_info = {'std': 999, 'low_pct': 0}
    
    # 4. Color preservation
    print(f"\n--- Color Preservation ---")
    overlap_high = (ca >= 220) & (oa >= 220)
    color_diff = 0
    if overlap_high.any():
        rgb_diff = np.sqrt(((crgb[overlap_high] - orgb[overlap_high]) ** 2).sum(axis=-1))
        color_diff = float(rgb_diff.mean())
        print(f"  Overlap pixels: {overlap_high.sum():,}")
        print(f"  Mean diff: {color_diff:.2f}, Max diff: {rgb_diff.max():.1f}")
        print(f"  Median diff: {np.median(rgb_diff):.2f}")
        
        bad_pixels = rgb_diff > 30
        if bad_pixels.sum() > overlap_high.sum() * 0.01:
            pct = bad_pixels.sum()/overlap_high.sum()*100
            print(f"[WARN] {pct:.1f}% body pixels changed color significantly")
        else:
            print("[PASS] Colors well-preserved (< 1% significant change)")
    
    # 5. Remote transparency test
    print(f"\n--- Remote Transparency Test ---")
    np.random.seed(42)
    good_remote = 0
    bad_remote = 0
    bad_samples = []
    for _ in range(100):
        sx, sy = np.random.randint(0, w), np.random.randint(0, h)
        min_dist = float('inf')
        if len(clean_coords) > 0:
            dists = np.sqrt((clean_coords[:,1]-sx)**2 + (clean_coords[:,0]-sy)**2)
            min_dist = float(dists.min())
        
        if min_dist > 100:
            a_val = int(ca[sy, sx])
            if a_val < 5:
                good_remote += 1
            else:
                bad_remote += 1
                if bad_remote <= 5:
                    r_orig = orig[sy, sx]
                    bad_samples.append(f"  Stray at ({sx},{sy}): alpha={a_val}, RGB=({r_orig[0]},{r_orig[1]},{r_orig[2]})")
    
    print(f"  Remote samples (>100px from body): {good_remote} clean, {bad_remote} dirty")
    for s in bad_samples:
        print(s)
    if bad_remote == 0:
        print("[PASS] All remote areas properly transparent")
    else:
        print(f"[WARN] {bad_remote} stray opaque pixels in remote regions")
    
    # 6. Overall verdict
    print(f"\n--- OVERALL VERDICT ---")
    score = 0
    issues = []
    
    ts = shrink_info.get('total_shrink', 999)
    if ts < 30:
        score += 1
    else:
        issues.append("body shrinkage")
    
    if total_stray < 100:
        score += 1
    else:
        issues.append(f"edge stragglers ({total_stray})")
    
    std = matte_info['std']
    if std < 40:
        score += 1
    else:
        issues.append("inconsistent matte")
    
    cd = color_diff
    if cd < 5:
        score += 1
    else:
        issues.append("color shift")
    
    if bad_remote == 0:
        score += 1
    else:
        issues.append("remote artifacts")
    
    if score >= 4:
        print(f"PASSED: {score}/5 checks OK")
        if issues:
            for i in issues:
                print(f"  Minor: {i}")
    elif score >= 3:
        print(f"MARGINAL: {score}/5 checks OK")
        for i in issues:
            print(f"  Issue: {i}")
    else:
        print(f"FAILED: Only {score}/5 checks OK")
        for i in issues:
            print(f"  Problem: {i}")


# Run both sprites
analyze_edge_quality(
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side_clean.png",
    "SIDE SPRITE"
)

analyze_edge_quality(
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back_clean.png",
    "BACK SPRITE"
)
