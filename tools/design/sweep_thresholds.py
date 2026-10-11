"""
Quick threshold sweep: generate cutouts at multiple brightness thresholds,
produce composites for each, delegate to vision agent for selection.
"""
from PIL import Image
import numpy as np


def make_binary_cutout(input_path, v_threshold, label):
    """Create binary (fully opaque/transparent) cutout at given V threshold."""
    img = Image.open(input_path).convert('RGBA')
    arr = np.array(img, dtype=np.float64)
    h, w, _ = arr.shape
    
    r, g, b = arr[:,:,0]/255.0, arr[:,:,1]/255.0, arr[:,:,2]/255.0
    cmax = np.maximum(np.maximum(r, g), b)
    # v = cmax already
    
    # Binary mask
    body_mask = (cmax >= v_threshold / 255.0).astype(bool)
    
    # Light binary erosion (3 iterations)
    eroded = body_mask.astype(np.float64)
    for i in range(3):
        padded = np.pad(eroded, 1, mode='constant', constant_values=0)
        new_m = np.ones_like(eroded)
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                new_m = new_m * padded[1+dy:h+1+dy, 1+dx:w+1+dx]
        eroded = new_m
    
    # Defringe boundary
    filled = body_mask & ~eroded.astype(bool)
    fy, fx = np.where(filled)
    if len(fy) > 0:
        ya, xa = fy.astype(int), fx.astype(int)
        for step in range(6):
            ny_up = np.clip(ya-1,0,h-1); ny_dn = np.clip(ya+1,0,h-1)
            nx_lt = np.clip(xa-1,0,w-1); nx_rt = np.clip(xa+1,0,w-1)
            mvals = np.column_stack([
                eroded[ny_up,xa], eroded[ny_dn,xa],
                eroded[ya,nx_lt], eroded[ya,nx_rt]])
            best_d = np.argmax(mvals, axis=1)
            moving = mvals[np.arange(len(ya)), best_d] < 0.5
            if not moving.any(): break
            ya_new, xa_new = ya.copy(), xa.copy()
            for d_idx in range(4):
                p = (best_d == d_idx) & moving
                yy = [ny_up, ny_dn, ya, ya][d_idx]
                xx = [ya, ya, nx_lt, nx_rt][d_idx]
                ya_new[p] = np.clip(yy[p], 0, h-1)
                xa_new[p] = np.clip(xx[p], 0, w-1)
            ya, xa = ya_new, xa_new
        arr[fy, fx, :3] = arr[ya, xa, :3].astype(float)
    
    # Build output
    out = np.zeros((h, w, 4), dtype=np.uint8)
    out[:, :, :3] = 255
    am = eroded.astype(bool)
    out[am, :3] = arr[am, :3].astype(np.uint8)
    out[am, 3] = 255
    
    return out


# Generate multiple variants for SIDE
side_in = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side.png"
back_in = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back.png"
out_dir = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy"

thresholds = [20, 25, 30, 35, 40]

for t in thresholds:
    print(f"\n--- Threshold V={t} ---")
    
    # Side
    out_side = make_binary_cutout(side_in, t, "SIDE")
    name_s = f"pc_zhengzha_side_v{t}"
    img_out = Image.fromarray(out_side, 'RGBA')
    img_out.save(f"{out_dir}\\{name_s}.png", 'PNG')
    n_body = int((out_side[:,:,3]==255).sum())
    print(f"  SIDE: {n_body:,} opaque px ({n_body/(768*1024)*100:.1f}%)")
    
    # Back
    out_back = make_binary_cutout(back_in, t, "BACK")
    name_b = f"pc_zhengzha_back_v{t}"
    img_out2 = Image.fromarray(out_back, 'RGBA')
    img_out2.save(f"{out_dir}\\{name_b}.png", 'PNG')
    n_body_b = int((out_back[:,:,3]==255).sum())
    print(f"  BACK: {n_body_b:,} opaque px ({n_body_b/(768*1024)*100:.1f}%)")

print("\n\nAll variants generated.")
print("Side outputs:")
for t in thresholds:
    print(f"  {out_dir}\\pc_zhengzha_side_v{t}.png")
print("Back outputs:")
for t in thresholds:
    print(f"  {out_dir}\\pc_zhengzha_back_v{t}.png")
