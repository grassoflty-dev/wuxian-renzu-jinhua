"""Thorough programmatic quality check of cutout outputs."""
from PIL import Image
import numpy as np

def quality_check(input_path, output_path, label):
    print(f"\n{'='*60}")
    print(f"QUALITY CHECK: {label}")
    print(f"{'='*60}")
    
    orig = np.array(Image.open(input_path).convert('RGBA'))
    clean = np.array(Image.open(output_path).convert('RGBA'))
    h, w, _ = clean.shape
    
    oa, ca = orig[:,:,3], clean[:,:,3]
    orgb, crgb = orig[:,:,:3], clean[:,:,:3]
    
    # 1. Check transparency distribution
    print(f"\n--- Transparency ---")
    print(f"  Transparent (a<128): {(ca < 128).sum():,} ({(ca<128).mean()*100:.1f}%)")
    print(f"  Opaque (a>=128):     {(ca >= 128).sum():,} ({(ca>=128).mean()*100:.1f}%)")
    edge_px = ((ca > 0) & (ca < 128)).sum()
    print(f"  Edge (1<=a<=127):    {edge_px:,} ({edge_px/(h*w)*100:.2f}%)")
    
    # 2. Check for dark residue outside figure bounds
    coords = np.column_stack(np.where(ca >= 128))
    if len(coords) == 0:
        print("  [FAIL] No opaque pixels!")
        return
    
    min_y, min_x = coords.min(axis=0)
    max_y, max_x = coords.max(axis=0)
    print(f"\n--- Bounding Box ---")
    print(f"  Figure: ({min_x},{min_y}) to ({max_x},{max_y}), size {max_x-min_x+1}x{max_y-min_y+1}")
    
    # Check margin zones around bbox for dark residue
    margins = [(0, min_y), (min_y, min_y+5), (max_y, max_y+5), 
               (-20, min_x), (min_x, min_x+5), (max_x, max_x+5)]
    
    issues_found = []
    for dir_name, y1, y2, x1, x2 in [
        ("above", 0, min_y, 0, w),
        ("below", max_y, h, 0, w),
        ("left", min_y, max_y+1, 0, min_x),
        ("right", min_y, max_y+1, max_x, w),
    ]:
        if y2 <= y1 or x2 <= x1:
            continue
        region_a = ca[max(y1,0):min(y2,h), max(x1,0):min(x2,w)]
        opaque_in_region = (region_a >= 128).sum()
        dark_in_region = ((region_a < 128) & (orgb[max(y1,0):min(y2,h), max(x1,0):min(x2,w)].mean(axis=-1) < 20)).sum()
        
        if opaque_in_region > 0:
            issues_found.append(f"{dir_name}: {opaque_in_region} opaque stray pixels")
        print(f"  {dir_name:8s} (outside): {opaque_in_region} opaque px, {dark_in_region} dark px")
    
    # 3. Compare character RGB against original (should be same where alpha is high)
    overlap_mask = (ca >= 200) & (oa >= 200)
    if overlap_mask.any():
        rgb_diff = np.sqrt(((crgb[overlap_mask].astype(float) - orgb[overlap_mask].astype(float)) ** 2).sum(axis=-1))
        mean_diff = rgb_diff.mean()
        max_diff = rgb_diff.max()
        print(f"\n--- Color Fidelity ---")
        print(f"  Mean RGB diff (high-alpha overlap): {mean_diff:.2f}")
        print(f"  Max RGB diff (high-alpha overlap):  {max_diff:.1f}")
        if mean_diff > 5:
            print(f"  [WARN] High average color difference!")
    
    # 4. Edge analysis: look at alpha transition width
    print(f"\n--- Edge Transition Quality ---")
    edge_mask = (ca > 0) & (ca < 128)
    if edge_mask.sum() > 0:
        # Check edge pixel brightness
        edge_alpha_mean = ca[edge_mask].mean()
        edge_rgb_mean = crgb[edge_mask].mean(axis=(0,1))
        print(f"  Edge alpha mean: {edge_alpha_mean:.1f}")
        print(f"  Edge RGB mean:   ({edge_rgb_mean[0]:.0f}, {edge_rgb_mean[1]:.0f}, {edge_rgb_mean[2]:.0f})")
        
        # Are edge pixels mostly semi-transparent black (artifact) or smooth gradient?
        # Good edges should have alpha smoothly varying and RGB close to character colors
        low_alpha_edge = ca[(ca > 0) & (ca < 64)]
        if len(low_alpha_edge) > 0:
            avg_low = low_alpha_edge.mean()
            print(f"  Low-alpha edge (1-63) mean alpha: {avg_low:.1f} (lower = smoother)")
    
    # 5. Summary verdict
    print(f"\n--- VERDICT ---")
    all_ok = True
    
    # Check 1: Clean edges
    below_count = (ca[max_y:, :].flatten() >= 128).sum()
    above_count = (ca[:min_y, :].flatten() >= 128).sum()
    if above_count > 10 or below_count > 10:
        print(f"  [FAIL] Stray opaque pixels outside figure")
        all_ok = False
    else:
        print(f"  [PASS] No stray pixels outside figure boundary")
    
    # Check 2: Edge count reasonable
    total_fig_pixels = (ca >= 128).sum()
    edge_ratio = edge_px / total_fig_pixels if total_fig_pixels > 0 else 999
    if edge_ratio > 0.1:
        print(f"  [WARN] High edge-to-body ratio: {edge_ratio:.1%}")
    else:
        print(f"  [PASS] Clean edge ratio: {edge_ratio:.1%}")
    
    # Check 3: Color preserved
    if mean_diff > 10:
        print(f"  [WARN] Significant color change in body")
    else:
        print(f"  [PASS] Body colors well-preserved")
    
    # Overall
    if all_ok:
        print(f"\n  >>> OVERALL: {'PASS' if edge_ratio <= 0.1 else 'PASS_WITH_WARNINGS'}")
    else:
        print(f"\n  >>> OVERALL: NEEDS_REVIEW")
    
    return {"all_ok": all_ok, "edge_ratio": edge_ratio, "mean_diff": float(mean_diff)}


# Run checks
side_result = quality_check(
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side_clean.png",
    "SIDE SPRITE"
)

back_result = quality_check(
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back_clean.png",
    "BACK SPRITE"
)

print(f"\n\n{'='*60}")
print("SUMMARY")
print(f"{'='*60}")
print(f"Side:  OK={side_result['all_ok']}, edge_ratio={side_result['edge_ratio']:.3%}, color_diff={side_result['mean_diff']:.1f}")
print(f"Back:  OK={back_result['all_ok']}, edge_ratio={back_result['edge_ratio']:.3%}, color_diff={back_result['mean_diff']:.1f}")
