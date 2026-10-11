"""
v9 Cutout: combined saturation + brightness matte + binary mask.

Strategy: Instead of distance-to-bg, use HSV space:
- Background pixels near-black have low V value AND low S value (or undefined)
- Character pixels (even dark ones) should have higher V or detectable S
- Create saturation-weighted matte that's less affected by rim glow
"""
from PIL import Image
import numpy as np


def rgb_to_hsv(r, g, b):
    """Vectorized RGB to HSV conversion."""
    r = r / 255.0
    g = g / 255.0
    b = b / 255.0
    
    cmax = np.maximum(np.maximum(r, g), b)
    cmin = np.minimum(np.minimum(r, g), b)
    diff = cmax - cmin
    
    # Value
    v = cmax
    
    # Saturation
    s = np.where(cmax > 0, diff / cmax, 0)
    
    # Hue (ignored for now)
    h = np.zeros_like(diff)
    mask_r = (cmax == r) & (diff > 0)
    mask_g = (cmax == g) & (diff > 0)
    mask_b = (cmax == b) & (diff > 0)
    h[mask_r] = ((g[mask_r] - b[mask_r]) / diff[mask_r]) % 6
    h[mask_g] = 2 + (b[mask_g] - r[mask_g]) / diff[mask_g]
    h[mask_b] = 4 + (r[mask_b] - g[mask_b]) / diff[mask_b]
    h = h * 60
    
    return h, s, v


def cutout_v9(input_path, output_path):
    print(f"\n{'='*60}")
    print(f"V9 CUTOUT: {output_path}")
    
    img = Image.open(input_path).convert('RGBA')
    arr = np.array(img, dtype=np.float64)
    h, w, _ = arr.shape
    
    # ── 1. Analyze in HSV ──
    r = arr[:,:,0]; g = arr[:,:,1]; b = arr[:,:,2]
    hsv_h, hsv_s, hsv_v = rgb_to_hsv(r, g, b)
    
    # BG color from corners
    margin = 80
    corner_data = np.vstack([arr[:margin,:margin].reshape(-1,4)[:,:3],
                              arr[:margin,-margin:].reshape(-1,4)[:,:3],
                              arr[-margin:,:margin].reshape(-1,4)[:,:3],
                              arr[-margin:,-margin:].reshape(-1,4)[:,:3]])
    bg_color = corner_data.mean(axis=0)
    bg_r, bg_g, bg_b = bg_color[0], bg_color[1], bg_color[2]
    _, bg_s, bg_v = rgb_to_hsv(bg_r, bg_g, bg_b)
    
    print(f"  BG: RGB=({bg_color[0]:.1f},{bg_color[1]:.1f},{bg_color[2]:.1f}), S={bg_s:.3f}, V={bg_v:.3f}")
    
    # Histograms for S and V
    print(f"\n  Value histogram:")
    for lo in [0, 0.05, 0.1, 0.15, 0.2, 0.3, 0.4, 0.5, 0.7, 0.9]:
        hi = min(lo + 0.1, 1.0)
        mask = (hsv_v >= lo) & (hsv_v < hi)
        if mask.any():
            count = mask.sum()
            avg_s = hsv_s[mask].mean()
            print(f"    V [{lo:.2f},{hi:.2f}): {count:>7,} px, avg_sat={avg_s:.3f}")
    
    print(f"\n  Saturation histogram:")
    for lo in [0, 0.02, 0.05, 0.1, 0.2, 0.3, 0.5]:
        hi = min(lo + 0.05, 1.0)
        mask = (hsv_s >= lo) & (hsv_s < hi)
        if mask.any():
            count = mask.sum()
            avg_v = hsv_v[mask].mean()
            print(f"    S [{lo:.2f},{hi:.2f}): {count:>7,} px, avg_val={avg_v:.3f}")
    
    # ── 2. Try combining brightness + saturation for body detection ──
    # Body pixels: slightly brighter than BG OR saturated
    # Glow pixels: bright but unsaturated (near-white)
    
    dist_from_bg = np.sqrt(np.sum((arr[:,:,:3].astype(float) - bg_color)**2, axis=2))
    
    # Combine: weight by both distance and saturation
    # High-distance pixels are candidate body
    # Among those, high-saturation = real body; low-saturation = possible glow
    score = dist_from_bg / 255.0 * 0.5 + hsv_s * 0.3 + hsv_v * 0.2
    
    # Threshold on score
    body_mask = (score >= 0.08).astype(bool)
    n_body = int(body_mask.sum())
    print(f"\n  Combined score threshold: {body_mask.sum():,} body pixels")
    
    # Also try pure V threshold
    body_v = (hsv_v >= 0.08).astype(bool)
    print(f"  V>=0.08 only: {body_v.sum():,} pixels")
    
    # Binary erosion
    eroded = body_mask.astype(np.float64)
    for i in range(3):
        padded = np.pad(eroded, 1, mode='constant', constant_values=0)
        new_m = np.ones_like(eroded)
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                new_m = new_m * padded[1+dy:h+1+dy, 1+dx:w+1+dx]
        before = int(eroded.sum())
        eroded = new_m
        after = int(eroded.sum())
        print(f"    Eroded #{i+1}: removed={before-after:,}, remaining={after:,}")
    
    # Build alpha matte (binary)
    alpha_matte = eroded.astype(bool)
    
    # Defringe
    filled = body_mask & ~alpha_matte
    fy, fx = np.where(filled)
    if len(fy) > 0:
        ya, xa = fy.astype(int), fx.astype(int)
        for step in range(6):
            ny_up = np.clip(ya-1,0,h-1); ny_dn = np.clip(ya+1,0,h-1)
            nx_lt = np.clip(xa-1,0,w-1); nx_rt = np.clip(xa+1,0,w-1)
            mvals = np.column_stack([
                alpha_matte[ny_up,xa], alpha_matte[ny_dn,xa],
                alpha_matte[ya,nx_lt], alpha_matte[ya,nx_rt]])
            best_d = np.argmax(mvals, axis=1)
            moving = mvals[np.arange(len(ya)), best_d] < 0.5
            if not moving.any(): break
            ya_new = np.copy(ya); xa_new = np.copy(xa)
            for d_idx in range(4):
                p = (best_d == d_idx) & moving
                yy = [ny_up, ny_dn, ya, ya][d_idx]
                xx = [ya, ya, nx_lt, nx_rt][d_idx]
                ya_new[p] = np.clip(yy[p], 0, h-1)
                xa_new[p] = np.clip(xx[p], 0, w-1)
            ya = ya_new; xa = xa_new
        arr[fy, fx, :3] = arr[ya, xa, :3].astype(float)
        print(f"  Defringed {len(fy)} boundary pixels")
    
    # Output
    out_arr = np.zeros((h, w, 4), dtype=np.uint8)
    out_arr[:, :, :3] = 255
    out_arr[alpha_matte, :3] = arr[alpha_matte, :3].astype(np.uint8)
    out_arr[alpha_matte, 3] = 255
    
    n_trans = int((out_arr[:,:,3]==0).sum())
    n_body_final = int((out_arr[:,:,3]==255).sum())
    print(f"\n  Final: transparent={n_trans:,} ({n_trans/(h*w)*100:.1f}%), opaque={n_body_final:,} ({n_body_final/(h*w)*100:.1f}%)")
    
    # Check edge pixel brightness (should be none since all are binary)
    print("  All-binary alpha (no semi-transparent edge)")
    
    img_out = Image.fromarray(out_arr, 'RGBA')
    img_out.save(output_path, 'PNG')
    print(f"  Saved\n")
    
    return alpha_matte


# ═══ SIDE ═══
cutout_v9(
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side_clean.png"
)

# ═══ BACK ═══  
cutout_v9(
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back_clean.png"
)

print("Done!")
