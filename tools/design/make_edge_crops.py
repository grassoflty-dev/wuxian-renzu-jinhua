"""Create zoomed edge-crop composites showing the exact boundary area."""
from PIL import Image, ImageDraw
import numpy as np

def make_edge_zoom(orig_path, clean_path, label):
    """Zoom into character edges to see halo quality."""
    orig = np.array(Image.open(orig_path).convert('RGBA'))
    clean = np.array(Image.open(clean_path).convert('RGBA'))
    
    # Find opaque region in clean image
    ca = clean[:,:,3]
    coords = np.column_stack(np.where(ca >= 128))
    c_min_y, c_min_x = coords.min(axis=0)
    c_max_y, c_max_x = coords.max(axis=0)
    
    # Create zoom panels for each side
    padding = 40
    zoom_w = 200
    zoom_h = 200
    
    results = []
    
    # Top-left corner
    cx1 = max(0, c_min_x - padding)
    cy1 = max(0, c_min_y - padding)
    crop_tl = orig[cy1:cy1+zoom_h, cx1:cx1+zoom_w]
    clean_tl = clean[cy1:cy1+zoom_h, cx1:cx1+zoom_w]
    
    # Bottom-right corner
    cx2 = min(orig.shape[1]-zoom_w, c_max_x - zoom_w + padding)
    cy2 = min(orig.shape[0]-zoom_h, c_max_y - zoom_h + padding)
    crop_br = orig[cy2:cy2+zoom_h, cx2:cx2+zoom_w]
    clean_br = clean[cy2:cy2+zoom_h, cx2:cx2+zoom_w]
    
    def composite_crop(crop, bg_color):
        h, w, _ = crop.shape
        bg_arr = np.zeros((h, w, 4), dtype=np.uint8)
        bg_arr[:,:,:3] = bg_color
        bg_arr[:,:,3] = 255
        fa = crop[:,:,3].astype(float) / 255.0
        result = (bg_arr[:,:,:3].astype(float) * (1-fa[...,None]) + 
                  crop[:,:,:3].astype(float) * fa[...,None]).astype(np.uint8)
        return Image.fromarray(result, 'RGB')
    
    # Composite layout: top row = original, bottom row = clean; col1=black bg, col2=white bg
    panels = [
        ("orig_black", composite_crop(crop_tl, (0,0,0))),
        ("clean_black", composite_crop(clean_tl, (0,0,0))),
        ("orig_white", composite_crop(crop_tl, (255,255,255))),
        ("clean_white", composite_crop(clean_tl, (255,255,255))),
        ("orig_black", composite_crop(crop_br, (0,0,0))),
        ("clean_black", composite_crop(clean_br, (0,0,0))),
        ("orig_white", composite_crop(crop_br, (255,255,255))),
        ("clean_white", composite_crop(clean_br, (255,255,255))),
    ]
    
    labels = ["ORIG", "CLEAN", "ORIG", "CLEAN", "ORIG", "CLEAN", "ORIG", "CLEAN"]
    bgs = ["BKG", "BKG", "WT", "WT", "BKG", "BKG", "WT", "WT"]
    
    # Arrange as 2 rows x 4 cols
    cell_size = (200, 100)
    grid_w = cell_size[0] * 4
    grid_h = cell_size[1] * 2
    
    grid = Image.new('RGB', (grid_w, grid_h))
    for i, (name, panel) in enumerate(panels):
        r, c = divmod(i, 4)
        resized = panel.resize(cell_size, Image.LANCZOS)
        
        # Add border
        bordered = Image.new('RGB', (cell_size[0]+2, cell_size[1]+2), (128,128,128))
        bordered.paste(resized, (1, 1))
        
        # Add labels
        label_img = bordered.copy()
        draw = ImageDraw.Draw(label_img)
        draw.rectangle([0, 0, bordered.width-1, 14], fill=(0,0,0))
        # Text labels would require font... skip for simplicity
        
        grid.paste(bordered, (c*cell_size[0], r*cell_size[1]))
    
    out_path = f"C:\\Users\\PUBLIC_USER\\Desktop\\itwillclaude\\games\\wuxian-horror-ch1\\tools\\design\\raw_enemy\\edge_zoom_{label}.png"
    grid.save(out_path, 'PNG')
    print(f"[+] {label}: edge zoom saved to {out_path}")
    return out_path


make_edge_zoom(
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_side_clean.png",
    "SIDE"
)

make_edge_zoom(
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back.png",
    r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy\pc_zhengzha_back_clean.png",
    "BACK"
)

print("Done.")
