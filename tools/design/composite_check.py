"""Create composite images with transparent PNGs shown on light AND dark backgrounds."""
from PIL import Image
import numpy as np

def composite_on_bg(path, bg_color_name, bg_rgb):
    """Overlay RGBA image on solid color background."""
    rgba = Image.open(path).convert('RGBA')
    arr = np.array(rgba)
    
    # Create background
    bg_arr = np.zeros((arr.shape[0], arr.shape[1], 4), dtype=np.uint8)
    bg_arr[:, :, :3] = bg_rgb
    bg_arr[:, :, 3] = 255
    
    # Composite: alpha-blend
    fg_alpha = arr[:,:,3].astype(float) / 255.0
    result = (bg_arr[:,:,:3].astype(float) * (1 - fg_alpha[...,None]) + 
              arr[:,:,:3].astype(float) * fg_alpha[...,None]).astype(np.uint8)
    
    return Image.fromarray(result, 'RGB')

# Files to check
files = [
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side_clean.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back_clean.png",
]

labels = ["SIDE", "BACK"]

for path, label in zip(files, labels):
    white = composite_on_bg(path, "white", (255, 255, 255))
    black = composite_on_bg(path, "black", (0, 0, 0))
    gray = composite_on_bg(path, "gray", (128, 128, 128))
    
    out_path_white = path.replace(".png", "_on_white.png")
    out_path_black = path.replace(".png", "_on_black.png")
    out_path_gray = path.replace(".png", "_on_gray.png")
    
    white.save(out_path_white, 'PNG')
    black.save(out_path_black, 'PNG')
    gray.save(out_path_gray, 'PNG')
    
    print(f"[+] {label}: saved {out_path_white}, {out_path_black}, {out_path_gray}")

print("\nDone. All composite images ready for visual check.")
