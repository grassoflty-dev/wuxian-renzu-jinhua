"""Generate white/black/gray composites for all threshold variants."""
from PIL import Image
import numpy as np

def composite_rgba(rgba_arr, bg_rgb):
    h, w = rgba_arr.shape[:2]
    bg = np.zeros((h, w, 4), dtype=np.uint8)
    bg[:,:,:3] = bg_rgb; bg[:,:,3] = 255
    fa = rgba_arr[:,:,3].astype(float)/255.0
    return ((bg[:,:,:3].astype(float)*(1-fa[...,None]) + 
             rgba_arr[:,:,:3].astype(float)*fa[...,None]).astype(np.uint8))

dirs = [r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy"]
base = dirs[0]
thresholds = [25, 30]
sprites = ["side", "back"]

for t in thresholds:
    for sp in sprites:
        path = f"{base}\\pc_zhengzha_{sp}_v{t}.png"
        try:
            arr = np.array(Image.open(path).convert('RGBA'))
            for bg_name, bg_rgb in [("white",(255,255,255)),("black",(0,0,0)),("gray",(128,128,128))]:
                comp = composite_rgba(arr, bg_rgb)
                out_path = path.replace(".png", f"_on_{bg_name}.png")
                Image.fromarray(comp, 'RGB').save(out_path, 'PNG')
            print(f"[+] {sp}_v{t}: done")
        except Exception as e:
            print(f"[-] {sp}_v{t}: {e}")

# Also composite the current clean files
for sp in sprites:
    path = f"{base}\\pc_zhengzha_{sp}_clean.png"
    try:
        arr = np.array(Image.open(path).convert('RGBA'))
        for bg_name, bg_rgb in [("white",(255,255,255)),("black",(0,0,0))]:
            comp = composite_rgba(arr, bg_rgb)
            out_path = path.replace(".png", f"_on_{bg_name}.png")
            Image.fromarray(comp, 'RGB').save(out_path, 'PNG')
        print(f"[+] {sp}_clean (updated): done")
    except Exception as e:
        print(f"[-] {sp}_clean: {e}")

print("\nAll composites generated.")
