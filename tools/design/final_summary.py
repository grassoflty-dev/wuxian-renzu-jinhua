"""Final comparison: original vs clean edge analysis."""
from PIL import Image
import numpy as np

base = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\design\raw_enemy"
front_ref = r"C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\server-rs\ui\assets\img\pc_zhengzha.png"

print("=== FINAL COMPARISON ===")
for label, path in [
    ("Front ref", front_ref),
    ("Side v25 clean", f"{base}\\pc_zhengzha_side_clean.png"),
    ("Back v25 clean", f"{base}\\pc_zhengzha_back_clean.png"),
]:
    arr = np.array(Image.open(path).convert('RGBA'))
    h, w = arr.shape[:2]
    
    # Alpha stats
    n_trans = int((arr[:,:,3]==0).sum())
    n_body = int((arr[:,:,3]==255).sum())
    n_edge = int(((arr[:,:,3]>0) & (arr[:,:,3]<255)).sum())
    
    # Body RGB stats  
    body_mask = arr[:,:,3] >= 128
    if body_mask.any():
        body_rgb_mean = arr[body_mask,:3].mean(axis=0)
        body_bright = body_rgb_mean.mean()
    else:
        body_rgb_mean = [0,0,0]
        body_bright = 0
    
    print(f"\n{label}:")
    print(f"  Alpha: trans={n_trans:,}({n_trans/(h*w)*100:.1f}%), "
          f"body={n_body:,}({n_body/(h*w)*100:.1f}%), edge={n_edge}")
    print(f"  Body mean RGB: ({body_rgb_mean[0]:.0f},{body_rgb_mean[1]:.0f},{body_rgb_mean[2]:.0f}) bright={body_bright:.0f}")

# Key insight: compare with what we had before
print("\n\n=== EVOLUTION OF APPROACHES ===")
approaches = {
    "Original": {"side_px": 118934, "back_px": 207044},  # From first cut attempt
    "v5 sigmoid": {"side_px": 121292, "edge_px": 290, "back_px": 206808, "edge_px_back": 644},
    "v25 binary": {"side_px": 73374, "edge_px": 0, "back_px": 176162, "edge_px_back": 0},
}

print("Key difference: v25 eliminates ALL semi-transparent edge pixels")
print("(previously ~290 side / ~644 back edge pixels had wrong colors)")
print("Tradeoff: some dark body pixels may be excluded (threshold limitation)")
print()
print("The rim glow issue is INHERENT to wan2.7 sprites:")
print("- Characters are extremely dark (median RGB~5)")
print("- Rim glow has similar darkness → can't be separated by color alone")
print("- V≥25 captures brightest body pixels; dimmer rim gets trimmed")
