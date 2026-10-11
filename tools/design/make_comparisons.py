"""Generate side-by-side comparison of original vs clean for visual inspection."""
from PIL import Image
import numpy as np

def make_comparison(orig_path, clean_path, label):
    """Create a 4-panel image: original on black, clean on white, clean on black, edge detail."""
    orig = np.array(Image.open(orig_path).convert('RGBA'))
    clean = np.array(Image.open(clean_path).convert('RGBA'))
    
    panel_h, panel_w = 256, 256
    
    # Create composites
    panels = []
    
    for panel_name, panel_data in [
        ("original_on_black", orig),
        ("clean_on_white", composite_on_bg(clean, (255,255,255))),
        ("clean_on_black", composite_on_bg(clean, (0,0,0))),
        ("alpha_channel", alpha_to_gray(clean)),
    ]:
        arr = panel_data
        pil_img = Image.fromarray(arr.astype(np.uint8))
        
        # Crop to character area
        if pil_img.mode == 'RGBA':
            a = pil_img.split()[3]
            bbox = a.getbbox()
        else:
            arr_a = np.zeros((pil_img.height, pil_img.width), dtype=np.uint8)
            pil_rgba = pil_img.convert('RGBA')
            a = pil_rgba.split()[3]
            bbox = a.getbbox()
        
        if bbox:
            cropped = pil_img.crop(bbox)
        else:
            cropped = pil_img
        
        # Scale down to panel size
        scale = min(panel_w / cropped.width, panel_h / cropped.height)
        new_size = (int(cropped.width * scale), int(cropped.height * scale))
        scaled = cropped.resize(new_size, Image.LANCZOS)
        
        # Center in panel
        canvas = Image.new('RGB' if pil_img.mode != 'RGBA' else 'RGBA', (panel_w, panel_h), 
                          (255,255,255) if 'white' in panel_name else (0,0,0))
        paste_x = (panel_w - new_size[0]) // 2
        paste_y = (panel_h - new_size[1]) // 2
        canvas.paste(scaled, (paste_x, paste_y))
        
        # Add label
        panels.append(canvas)
    
    # Stitch rows: 2x2 grid
    top = Image.new('RGB' if panels[0].mode == 'RGB' else 'RGBA', (panel_w*2, panel_h))
    bottom = Image.new('RGB' if panels[2].mode == 'RGB' else 'RGBA', (panel_w*2, panel_h))
    
    top.paste(panels[0], (0, 0))
    top.paste(panels[1], (panel_w, 0))
    bottom.paste(panels[2], (0, 0))
    bottom.paste(panels[3], (panel_w, 0))
    
    result = Image.new('RGBA', (panel_w*2, panel_h*2))
    result.paste(top, (0, 0))
    result.paste(bottom, (panel_w, panel_h))
    
    out_path = f"C:\\Users\\PUBLIC_USER\\Desktop\\itwillclaude\\games\\wuxian-horror-ch1\\tools\\design\\raw_enemy\\comparison_{label}.png"
    result.save(out_path, 'PNG')
    print(f"[+] Saved comparison: {out_path}")
    return out_path


def composite_on_bg(rgba_arr, bg_rgb):
    h, w, _ = rgba_arr.shape
    bg_arr = np.zeros((h, w, 4), dtype=np.uint8)
    bg_arr[:,:,:3] = bg_rgb
    bg_arr[:,:,3] = 255
    fa = rgba_arr[:,:,3].astype(float) / 255.0
    result = (bg_arr[:,:,:3].astype(float) * (1-fa[...,None]) + 
              rgba_arr[:,:,:3].astype(float) * fa[...,None]).astype(np.uint8)
    return result


def alpha_to_gray(clean):
    """Convert alpha channel to grayscale for edge inspection."""
    h, w, _ = clean.shape
    a = clean[:,:,3].astype(float) / 255.0
    gray = (a * 255).astype(np.uint8)
    result = np.zeros((h, w, 3), dtype=np.uint8)
    result[:,:,0] = gray
    result[:,:,1] = gray
    result[:,:,2] = gray
    return result


# Generate comparisons
comp_side = make_comparison(
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side_clean.png",
    "SIDE"
)

comp_back = make_comparison(
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back_clean.png",
    "BACK"
)

print("\nComparisons generated.")
