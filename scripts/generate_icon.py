"""
Generate high-fidelity MINE//SHIFT: BEYOND application icon.
Palette: Soft Circuit Obsidian (#121216, #1A1A24) with Neon Violet (#8B5CF6) and Emerald Mine (#10B981) highlights.
"""

import os
from PIL import Image, ImageDraw

def create_mineshift_icon():
    sizes = [(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (16, 16)]
    images = []

    for size in sizes:
        w, h = size
        img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        draw = ImageDraw.Draw(img)

        # Base tile (rounded bevel)
        pad = max(1, w // 16)
        r = max(2, w // 8)
        # Gradient background
        bg_color = (24, 24, 32, 255)
        border_color = (139, 92, 246, 255) # Neon Violet #8B5CF6
        draw.rounded_rectangle([pad, pad, w - pad, h - pad], radius=r, fill=bg_color, outline=border_color, width=max(1, w // 32))

        # Core Mine Symbol (geometric hex star)
        cx, cy = w // 2, h // 2
        core_r = max(2, w // 4)
        mine_fill = (16, 185, 129, 255) # Emerald #10B981
        draw.ellipse([cx - core_r, cy - core_r, cx + core_r, cy + core_r], fill=mine_fill)

        # Spikes / Traces
        spike_len = max(2, w // 3)
        spike_w = max(1, w // 24)
        draw.line([cx - spike_len, cy, cx + spike_len, cy], fill=border_color, width=spike_w)
        draw.line([cx, cy - spike_len, cx, cy + spike_len], fill=border_color, width=spike_w)

        # Inner highlight
        inner_r = max(1, core_r // 3)
        draw.ellipse([cx - inner_r, cy - inner_r, cx + inner_r, cy + inner_r], fill=(255, 255, 255, 230))

        images.append(img)

    os.makedirs("assets", exist_ok=True)
    # Save 256x256 PNG
    images[0].save("assets/icon.png", format="PNG")
    # Save multi-size ICO
    images[0].save("assets/app.ico", format="ICO", sizes=[(s[0], s[1]) for s in sizes])
    print("[ICON] Generated assets/icon.png and multi-resolution assets/app.ico")

if __name__ == "__main__":
    create_mineshift_icon()
