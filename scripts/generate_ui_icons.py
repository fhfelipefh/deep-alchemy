"""
Generate stylized alchemical UI icons:
1. Alchemical Hearts: full red heart with black ink outline and glossy shine, empty heart silhouette.
2. Alchemical Potion Flasks: round spherical glass flasks with wooden cork, neck ring, translucent glass highlight, and vibrant glowing liquid (green, red, yellow, blue, white).
3. Hotbar Frames: normal slot frame and gold ornate active slot frame.
"""
import math
import os
from PIL import Image, ImageDraw, ImageFilter

os.makedirs("assets/textures/ui", exist_ok=True)

def create_heart_icon(size=64, full=True):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # Scale coordinates
    s = size / 64.0
    
    # Heart path using cubic bezier / polygon points
    # Parametric heart curve: x = 16 sin^3(t), y = -(13 cos(t) - 5 cos(2t) - 2 cos(3t) - cos(4t))
    pts = []
    num_pts = 100
    cx = size / 2.0
    cy = size / 2.0 + 2.0 * s
    scale = s * 1.55
    
    for i in range(num_pts):
        t = 2 * math.pi * i / num_pts
        x = 16 * (math.sin(t) ** 3)
        y = -(13 * math.cos(t) - 5 * math.cos(2*t) - 2 * math.cos(3*t) - math.cos(4*t))
        pts.append((cx + x * scale, cy + y * scale))
        
    # Draw shadow / thick ink outline
    outline_color = (15, 20, 28, 255)
    for ox in range(-3, 4):
        for oy in range(-3, 4):
            if ox*ox + oy*oy <= 9:
                shifted = [(x + ox, y + oy) for x, y in pts]
                draw.polygon(shifted, fill=outline_color)
                
    if full:
        # Gradient fill (crimson to bright ruby red)
        fill_color = (235, 35, 55, 255)
        draw.polygon(pts, fill=fill_color)
        
        # Inner dark gradient at bottom
        bottom_pts = [(x, y) for x, y in pts if y > cy + 4*s]
        if len(bottom_pts) > 2:
            draw.polygon(bottom_pts, fill=(160, 15, 30, 200))
            
        # Top-left specular highlight shine (crescent/oval)
        shine_pts = [
            (cx - 10*s, cy - 8*s),
            (cx - 5*s, cy - 14*s),
            (cx - 12*s, cy - 13*s),
            (cx - 16*s, cy - 8*s),
        ]
        draw.polygon(shine_pts, fill=(255, 200, 210, 220))
        # Small white dot
        draw.ellipse([cx - 12*s, cy - 12*s, cx - 8*s, cy - 8*s], fill=(255, 255, 255, 240))
    else:
        # Empty heart dark slate interior
        draw.polygon(pts, fill=(35, 42, 52, 230))
        # Faint inner edge
        inner_pts = [(cx + (x - cx) * 0.82, cy + (y - cy) * 0.82) for x, y in pts]
        draw.polygon(inner_pts, fill=(20, 25, 32, 255))
        
    return img

def create_flask_icon(size=96, liquid_color=(76, 217, 100), glow_color=(120, 255, 140)):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    s = size / 96.0
    cx = size / 2.0
    cy = size / 2.0 + 4.0 * s
    
    # Flask dimensions:
    # Round bottom sphere radius
    r = 26.0 * s
    # Neck width & height
    nw = 10.0 * s
    nh = 18.0 * s
    neck_top = cy - r + 4.0 * s - nh
    cork_h = 9.0 * s
    
    # 1. Outer Ink Outline (thick black border)
    # Circle outline
    outline_col = (12, 16, 24, 255)
    for ox in range(-3, 4):
        for oy in range(-3, 4):
            if ox*ox + oy*oy <= 9:
                draw.ellipse([cx - r + ox, cy - r + oy, cx + r + ox, cy + r + oy], fill=outline_col)
                draw.rectangle([cx - nw + ox, neck_top + oy, cx + nw + ox, cy - r + 8*s + oy], fill=outline_col)
                draw.rectangle([cx - nw - 2*s + ox, neck_top - cork_h + oy, cx + nw + 2*s + ox, neck_top + oy], fill=outline_col)
                
    # 2. Glass Body Background (Dark transparent / translucent)
    draw.ellipse([cx - r + 2*s, cy - r + 2*s, cx + r - 2*s, cy + r - 2*s], fill=(25, 35, 45, 210))
    draw.rectangle([cx - nw + 2*s, neck_top + 2*s, cx + nw - 2*s, cy - r + 8*s], fill=(30, 42, 55, 210))
    
    # 3. Liquid inside sphere (half/two-thirds full)
    # Liquid surface Y
    liq_y = cy - 2.0 * s
    # Mask for liquid inside circle
    liquid_img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    liq_draw = ImageDraw.Draw(liquid_img)
    # Liquid circle
    lr = r - 4.0 * s
    liq_draw.ellipse([cx - lr, cy - lr, cx + lr, cy + lr], fill=(*liquid_color, 255))
    # Cut off top of liquid
    liq_draw.rectangle([0, 0, size, liq_y], fill=(0, 0, 0, 0))
    # Liquid meniscus ellipse
    meniscus_w = lr * math.sqrt(max(0.1, 1.0 - ((liq_y - cy)/lr)**2))
    liq_draw.ellipse([cx - meniscus_w, liq_y - 4*s, cx + meniscus_w, liq_y + 4*s], fill=(*glow_color, 255))
    
    # Composite liquid
    img = Image.alpha_composite(img, liquid_img)
    draw = ImageDraw.Draw(img)
    
    # 4. Glass Specular Highlights (Cel-shaded crescent on left and edge rim)
    # White highlight arc on bottom left
    draw.arc([cx - r + 5*s, cy - r + 5*s, cx + r - 5*s, cy + r - 5*s], start=120, end=210, fill=(255, 255, 255, 230), width=int(3*s))
    # Glass shine oval on top left
    draw.ellipse([cx - r*0.6, cy - r*0.6, cx - r*0.3, cy - r*0.3], fill=(255, 255, 255, 220))
    draw.ellipse([cx - r*0.65, cy - r*0.2, cx - r*0.5, cy - r*0.05], fill=(255, 255, 255, 180))
    
    # 5. Neck lip ring
    draw.ellipse([cx - nw - 1*s, neck_top - 1*s, cx + nw + 1*s, neck_top + 4*s], fill=(160, 195, 220, 240))
    draw.ellipse([cx - nw, neck_top, cx + nw, neck_top + 3*s], fill=(30, 42, 55, 240))
    
    # 6. Wooden Cork Stopper
    cork_col = (165, 110, 65, 255)
    cork_top_col = (195, 135, 80, 255)
    draw.polygon([
        (cx - nw*0.8, neck_top + 1*s),
        (cx + nw*0.8, neck_top + 1*s),
        (cx + nw*0.9, neck_top - cork_h),
        (cx - nw*0.9, neck_top - cork_h)
    ], fill=cork_col)
    # Cork top rounded cap
    draw.ellipse([cx - nw*0.9, neck_top - cork_h - 2*s, cx + nw*0.9, neck_top - cork_h + 3*s], fill=cork_top_col)
    
    # 7. Subtle alchemical bubble inside liquid
    draw.ellipse([cx + 6*s, cy + 6*s, cx + 10*s, cy + 10*s], fill=(255, 255, 255, 200))
    draw.ellipse([cx - 4*s, cy + 14*s, cx - 1*s, cy + 17*s], fill=(255, 255, 255, 180))
    
    return img

def create_slot_frame(size=80, active=False):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    s = size / 80.0
    
    # Dark alchemical stone background
    draw.rounded_rectangle([4*s, 4*s, size - 4*s, size - 4*s], radius=10*s, fill=(18, 22, 28, 220))
    
    if active:
        # Ornate Glowing Gold Border (3-layer filigree)
        # Outer glow
        draw.rounded_rectangle([2*s, 2*s, size - 2*s, size - 2*s], radius=12*s, outline=(255, 215, 60, 255), width=int(3*s))
        draw.rounded_rectangle([6*s, 6*s, size - 6*s, size - 6*s], radius=8*s, outline=(245, 175, 25, 200), width=int(2*s))
        # Corner gem accents
        for cx in [7*s, size - 7*s]:
            for cy in [7*s, size - 7*s]:
                draw.rectangle([cx - 2*s, cy - 2*s, cx + 2*s, cy + 2*s], fill=(255, 245, 180, 255))
    else:
        # Slate bronze/iron border
        draw.rounded_rectangle([3*s, 3*s, size - 3*s, size - 3*s], radius=11*s, outline=(10, 14, 18, 255), width=int(3*s))
        draw.rounded_rectangle([5*s, 5*s, size - 5*s, size - 5*s], radius=9*s, outline=(70, 78, 88, 200), width=int(2*s))
        
    return img

if __name__ == "__main__":
    # Generate Hearts
    create_heart_icon(64, full=True).save("assets/textures/ui/heart_full.png")
    create_heart_icon(64, full=False).save("assets/textures/ui/heart_empty.png")
    
    # Generate 5 Potion Flasks matching reference
    flasks = [
        ("flask_green.png", (45, 210, 85), (140, 255, 160)),    # 1. Agility / Spore
        ("flask_red.png", (235, 45, 60), (255, 140, 150)),      # 2. Vitality
        ("flask_yellow.png", (245, 195, 30), (255, 235, 130)),  # 3. Sulfur / Thermal
        ("flask_blue.png", (30, 160, 245), (140, 225, 255)),    # 4. Deep Water / Mana
        ("flask_white.png", (220, 230, 245), (255, 255, 255)),  # 5. Purified Essence
    ]
    for name, liq, glow in flasks:
        create_flask_icon(96, liq, glow).save(f"assets/textures/ui/{name}")
        
    # Generate Slot Frames
    create_slot_frame(80, active=False).save("assets/textures/ui/slot_normal.png")
    create_slot_frame(80, active=True).save("assets/textures/ui/slot_gold_active.png")
    
    print(">>> Successfully generated all stylized UI textures in assets/textures/ui/")
