"""Quick check V=22 variant."""
from PIL import Image
import numpy as np

base = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy"

for sp, path in [("side", f"{base}\\pc_zhengzha_side.png"), 
                  ("back", f"{base}\\pc_zhengzha_back.png")]:
    arr = np.array(Image.open(path).convert('RGBA'))
    h, w = arr.shape[:2]
    r = arr[:,:,0]/255.0; g = arr[:,:,1]/255.0; b = arr[:,:,2]/255.0
    cmax = np.maximum(np.maximum(r,g),b)
    
    for t in [20, 22, 24, 25]:
        body = (cmax >= t/255.0).astype(bool)
        # 3x erosion
        em = body.astype(np.float64)
        for i in range(3):
            pad = np.pad(em, 1, mode='constant', constant_values=0)
            n = np.ones_like(em)
            for dy in (-1,0,1):
                for dx in (-1,0,1):
                    n = n * pad[1+dy:h+1+dy, 1+dx:w+1+dx]
            em = n
        n_body = int(em.sum())
        pct = n_body/(h*w)*100
        print(f"  {sp} V={t}: {n_body:>7,} px ({pct:.1f}%)")
