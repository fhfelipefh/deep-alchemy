"""
Generate stylized alchemical UI icons:
1. Alchemical Hearts: full red heart with black ink outline and glossy shine, empty heart silhouette.
2. Alchemical Potion Flasks (5 distinct silhouettes, colors, and styles):
   - Slot 1: Classic Alchemical Sphere (Green Spore / Agility) with cork & twine knot, rising bubbles.
   - Slot 2: Conical Erlenmeyer Flask (Red Vitality / Blood) with brass neck ring, wax seal & diagonal facet shine.
   - Slot 3: Heavy Chamfered Square Flask (Yellow Sulfur / Fire) with bronze corner brackets, rivets & screw cap.
   - Slot 4: Curvilinear Teardrop / Pear Amphora (Blue Mana / Deep Water) with flared lip, cut sapphire stopper & arcane waves.
   - Slot 5: Hexagonal Cut Gemstone Vial (White Astral Ether / Purity) with crystal finial, silver filigree & 4-point star sparkle.
3. Hotbar Frames: normal slot frame and gold ornate active slot frame.
"""
import math
import os
from PIL import Image, ImageDraw

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

def create_flask_green_spherical(size=96):
    """Style 1: Classic Alchemical Sphere (Green Spore/Agility)"""
    s = 2.0  # Supersample 2x
    W = int(size * s)
    img = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    cx, cy = W / 2.0, W / 2.0 + 8.0 * s
    r = 26.0 * s
    nw, nh = 9.0 * s, 18.0 * s
    neck_top = cy - r + 3.0 * s - nh
    cork_h = 10.0 * s
    outline_col = (12, 16, 24, 255)
    
    # Outer black ink outline
    for ox in range(-5, 6):
        for oy in range(-5, 6):
            if ox*ox + oy*oy <= 25:
                draw.ellipse([cx - r + ox, cy - r + oy, cx + r + ox, cy + r + oy], fill=outline_col)
                draw.rectangle([cx - nw + ox, neck_top + oy, cx + nw + ox, cy - r + 6*s + oy], fill=outline_col)
                draw.polygon([
                    (cx - nw*0.85 + ox, neck_top + oy),
                    (cx + nw*0.85 + ox, neck_top + oy),
                    (cx + nw*1.05 + ox, neck_top - cork_h + oy),
                    (cx - nw*1.05 + ox, neck_top - cork_h + oy),
                ], fill=outline_col)
                draw.ellipse([cx - nw*1.05 + ox, neck_top - cork_h - 4*s + oy, cx + nw*1.05 + ox, neck_top - cork_h + 3*s + oy], fill=outline_col)

    # Glass Interior Backing
    draw.ellipse([cx - r + 4*s, cy - r + 4*s, cx + r - 4*s, cy + r - 4*s], fill=(20, 32, 28, 220))
    draw.rectangle([cx - nw + 3*s, neck_top + 3*s, cx + nw - 3*s, cy - r + 6*s], fill=(25, 38, 32, 220))
    
    # Liquid (Green Spore)
    liq_img = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    liq_draw = ImageDraw.Draw(liq_img)
    lr = r - 5.0 * s
    liq_draw.ellipse([cx - lr, cy - lr, cx + lr, cy + lr], fill=(42, 205, 80, 255))
    
    # Bottom shading
    liq_draw.ellipse([cx - lr*0.9, cy + lr*0.2, cx + lr*0.9, cy + lr*0.95], fill=(22, 135, 50, 240))
    
    # Cut liquid top (65% full)
    liq_y = cy - 2.0 * s
    liq_draw.rectangle([0, 0, W, liq_y], fill=(0, 0, 0, 0))
    # Glowing Meniscus
    mw = lr * math.sqrt(max(0.1, 1.0 - ((liq_y - cy)/lr)**2))
    liq_draw.ellipse([cx - mw, liq_y - 4*s, cx + mw, liq_y + 4*s], fill=(135, 255, 160, 255))
    
    img = Image.alpha_composite(img, liq_img)
    draw = ImageDraw.Draw(img)
    
    # Rising Spore Bubbles
    draw.ellipse([cx + 6*s, cy + 4*s, cx + 11*s, cy + 9*s], fill=(200, 255, 215, 240), outline=outline_col, width=int(1.5*s))
    draw.ellipse([cx - 8*s, cy + 12*s, cx - 4*s, cy + 16*s], fill=(200, 255, 215, 220), outline=outline_col, width=int(1.5*s))
    draw.ellipse([cx + 2*s, cy + 16*s, cx + 5*s, cy + 19*s], fill=(200, 255, 215, 200))
    
    # Glass Cel-Shading Specular Highlights
    draw.arc([cx - r + 7*s, cy - r + 7*s, cx + r - 7*s, cy + r - 7*s], start=115, end=205, fill=(255, 255, 255, 240), width=int(4*s))
    draw.ellipse([cx - r*0.55, cy - r*0.55, cx - r*0.25, cy - r*0.25], fill=(255, 255, 255, 230))
    
    # Neck Rim
    draw.ellipse([cx - nw - 2*s, neck_top - 2*s, cx + nw + 2*s, neck_top + 4*s], fill=(140, 210, 180, 255), outline=outline_col, width=int(2*s))
    
    # Cork Stopper with Twine
    draw.polygon([
        (cx - nw*0.8, neck_top),
        (cx + nw*0.8, neck_top),
        (cx + nw, neck_top - cork_h),
        (cx - nw, neck_top - cork_h),
    ], fill=(160, 105, 60, 255))
    draw.ellipse([cx - nw, neck_top - cork_h - 3*s, cx + nw, neck_top - cork_h + 3*s], fill=(195, 135, 80, 255))
    # Twine band around neck
    draw.line([cx - nw*1.1, neck_top + 6*s, cx + nw*1.1, neck_top + 6*s], fill=(215, 175, 105, 255), width=int(2.5*s))
    draw.line([cx - nw*1.1, neck_top + 10*s, cx + nw*1.1, neck_top + 10*s], fill=(185, 145, 85, 255), width=int(2.5*s))
    draw.ellipse([cx + nw*0.8, neck_top + 8*s, cx + nw*1.4, neck_top + 14*s], fill=(215, 175, 105, 255), outline=outline_col, width=int(1.5*s))
    
    return img.resize((size, size), Image.Resampling.LANCZOS)

def create_flask_red_conical(size=96):
    """Style 2: Triangular Conical Erlenmeyer Flask (Red Vitality/Healing)"""
    s = 2.0
    W = int(size * s)
    img = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    cx = W / 2.0
    base_y = W / 2.0 + 31.0 * s
    top_y = base_y - 42.0 * s
    base_w = 32.0 * s
    neck_w = 8.5 * s
    neck_top = top_y - 14.0 * s
    cork_h = 10.0 * s
    outline_col = (14, 16, 24, 255)
    
    body_pts = [
        (cx - base_w, base_y),
        (cx + base_w, base_y),
        (cx + neck_w + 3*s, top_y),
        (cx + neck_w, top_y),
        (cx + neck_w, neck_top),
        (cx - neck_w, neck_top),
        (cx - neck_w, top_y),
        (cx - neck_w - 3*s, top_y),
    ]
    
    # Outer black ink outline
    for ox in range(-5, 6):
        for oy in range(-5, 6):
            if ox*ox + oy*oy <= 25:
                shifted = [(x + ox, y + oy) for x, y in body_pts]
                draw.polygon(shifted, fill=outline_col)
                draw.polygon([
                    (cx - neck_w*0.85 + ox, neck_top + oy),
                    (cx + neck_w*0.85 + ox, neck_top + oy),
                    (cx + neck_w*1.1 + ox, neck_top - cork_h + oy),
                    (cx - neck_w*1.1 + ox, neck_top - cork_h + oy),
                ], fill=outline_col)
                draw.ellipse([cx - neck_w*1.1 + ox, neck_top - cork_h - 4*s + oy, cx + neck_w*1.1 + ox, neck_top - cork_h + 3*s + oy], fill=outline_col)

    # Glass Interior Backing
    inner_pts = [
        (cx - base_w + 4*s, base_y - 4*s),
        (cx + base_w - 4*s, base_y - 4*s),
        (cx + neck_w - 2*s, top_y),
        (cx + neck_w - 2*s, neck_top + 2*s),
        (cx - neck_w + 2*s, neck_top + 2*s),
        (cx - neck_w + 2*s, top_y),
    ]
    draw.polygon(inner_pts, fill=(35, 18, 22, 220))
    
    # Liquid (Red Vitality)
    liq_img = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    liq_draw = ImageDraw.Draw(liq_img)
    
    liq_y = base_y - 27.0 * s
    t = (base_y - liq_y) / (base_y - top_y)
    curr_w = (base_w - 4*s) * (1.0 - t) + (neck_w - 2*s) * t
    
    liq_pts = [
        (cx - base_w + 5*s, base_y - 5*s),
        (cx + base_w - 5*s, base_y - 5*s),
        (cx + curr_w, liq_y),
        (cx - curr_w, liq_y),
    ]
    liq_draw.polygon(liq_pts, fill=(235, 30, 50, 255))
    # Bottom deep crimson shadow
    liq_draw.polygon([
        (cx - base_w + 6*s, base_y - 5*s),
        (cx + base_w - 6*s, base_y - 5*s),
        (cx + base_w - 10*s, base_y - 14*s),
        (cx - base_w + 10*s, base_y - 14*s),
    ], fill=(160, 15, 30, 240))
    
    # Straight Meniscus with vibrant ruby shine
    liq_draw.line([cx - curr_w, liq_y, cx + curr_w, liq_y], fill=(255, 140, 155, 255), width=int(4*s))
    
    img = Image.alpha_composite(img, liq_img)
    draw = ImageDraw.Draw(img)
    
    # Cel-Shaded Facet Specular Highlight (Angled diagonal glass band)
    draw.polygon([
        (cx - base_w + 10*s, base_y - 8*s),
        (cx - base_w + 16*s, base_y - 8*s),
        (cx - neck_w + 1*s, top_y + 4*s),
        (cx - neck_w - 3*s, top_y + 4*s),
    ], fill=(255, 255, 255, 210))
    # Base rim highlight
    draw.line([cx - base_w + 12*s, base_y - 6*s, cx + base_w - 12*s, base_y - 6*s], fill=(255, 255, 255, 120), width=int(2*s))
    
    # Brass Neck Ring & Lip
    draw.rectangle([cx - neck_w - 3*s, neck_top - 2*s, cx + neck_w + 3*s, neck_top + 4*s], fill=(225, 180, 55, 255), outline=outline_col, width=int(2*s))
    
    # Stopper with Wax Seal
    draw.polygon([
        (cx - neck_w*0.8, neck_top),
        (cx + neck_w*0.8, neck_top),
        (cx + neck_w, neck_top - cork_h),
        (cx - neck_w, neck_top - cork_h),
    ], fill=(180, 25, 40, 255))
    # Wax seal golden crest button on top
    draw.ellipse([cx - neck_w*0.9, neck_top - cork_h - 4*s, cx + neck_w*0.9, neck_top - cork_h + 3*s], fill=(245, 195, 60, 255), outline=outline_col, width=int(2*s))
    draw.ellipse([cx - 4*s, neck_top - cork_h - 1*s, cx + 4*s, neck_top - cork_h + 7*s], fill=(210, 20, 35, 255))
    
    return img.resize((size, size), Image.Resampling.LANCZOS)

def create_flask_yellow_square(size=96):
    """Style 3: Heavy Chamfered Square Flask with Bronze Brackets (Yellow Sulfur/Thermal)"""
    s = 2.0
    W = int(size * s)
    img = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    cx = W / 2.0
    cy = W / 2.0 + 8.0 * s
    hw = 24.0 * s
    hh = 24.0 * s
    ch = 7.0 * s
    
    nw, nh = 10.0 * s, 14.0 * s
    neck_top = cy - hh - nh
    cap_h = 10.0 * s
    outline_col = (14, 16, 24, 255)
    
    body_pts = [
        (cx - hw + ch, cy - hh),
        (cx + hw - ch, cy - hh),
        (cx + hw, cy - hh + ch),
        (cx + hw, cy + hh - ch),
        (cx + hw - ch, cy + hh),
        (cx - hw + ch, cy + hh),
        (cx - hw, cy + hh - ch),
        (cx - hw, cy - hh + ch),
    ]
    
    # Outer black ink outline
    for ox in range(-5, 6):
        for oy in range(-5, 6):
            if ox*ox + oy*oy <= 25:
                shifted = [(x + ox, y + oy) for x, y in body_pts]
                draw.polygon(shifted, fill=outline_col)
                draw.rectangle([cx - nw + ox, neck_top + oy, cx + nw + ox, cy - hh + oy], fill=outline_col)
                draw.rectangle([cx - nw*1.2 + ox, neck_top - cap_h + oy, cx + nw*1.2 + ox, neck_top + oy], fill=outline_col)

    # Glass Interior Backing
    inner_pts = [
        (cx - hw + ch + 3*s, cy - hh + 4*s),
        (cx + hw - ch - 3*s, cy - hh + 4*s),
        (cx + hw - 4*s, cy - hh + ch + 3*s),
        (cx + hw - 4*s, cy + hh - ch - 3*s),
        (cx + hw - ch - 3*s, cy + hh - 4*s),
        (cx - hw + ch + 3*s, cy + hh - 4*s),
        (cx - hw + 4*s, cy + hh - ch - 3*s),
        (cx - hw + 4*s, cy - hh + ch + 3*s),
    ]
    draw.polygon(inner_pts, fill=(35, 28, 15, 220))
    
    # Liquid (Golden Amber Sulfur)
    liq_img = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    liq_draw = ImageDraw.Draw(liq_img)
    
    liq_y = cy - 6.0 * s
    liq_pts = [
        (cx - hw + 5*s, liq_y),
        (cx + hw - 5*s, liq_y),
        (cx + hw - 5*s, cy + hh - ch - 4*s),
        (cx + hw - ch - 4*s, cy + hh - 5*s),
        (cx - hw + ch + 4*s, cy + hh - 5*s),
        (cx - hw + 5*s, cy + hh - ch - 4*s),
    ]
    liq_draw.polygon(liq_pts, fill=(245, 185, 25, 255))
    liq_draw.rectangle([cx - hw + 6*s, cy + hh*0.3, cx + hw - 6*s, cy + hh - 6*s], fill=(185, 115, 15, 230))
    liq_draw.line([cx - hw + 5*s, liq_y, cx + hw - 5*s, liq_y], fill=(255, 240, 140, 255), width=int(4*s))
    
    img = Image.alpha_composite(img, liq_img)
    draw = ImageDraw.Draw(img)
    
    # Internal floating ember particles (diamond sparks)
    for px, py, sz in [(cx - 6*s, cy + 6*s, 3*s), (cx + 8*s, cy + 12*s, 4*s), (cx + 4*s, cy, 2.5*s)]:
        draw.polygon([(px, py - sz), (px + sz, py), (px, py + sz), (px - sz, py)], fill=(255, 255, 200, 255))
        
    # Bronze Reinforcement Straps & Corner Brackets
    metal_col = (175, 120, 50, 255)
    metal_dark = (115, 75, 30, 255)
    rivet_col = (255, 225, 120, 255)
    
    draw.rectangle([cx - hw - 1*s, cy + 2*s, cx + hw + 1*s, cy + 8*s], fill=metal_col, outline=outline_col, width=int(2*s))
    for rx in [cx - hw*0.6, cx, cx + hw*0.6]:
        draw.ellipse([rx - 2*s, cy + 3.5*s, rx + 2*s, cy + 6.5*s], fill=rivet_col)
        
    draw.polygon([(cx - hw, cy - hh + ch*1.5), (cx - hw + ch*1.5, cy - hh), (cx - hw + ch*0.5, cy - hh), (cx - hw, cy - hh + ch*0.5)], fill=metal_col)
    draw.polygon([(cx + hw, cy - hh + ch*1.5), (cx + hw - ch*1.5, cy - hh), (cx + hw - ch*0.5, cy - hh), (cx + hw, cy - hh + ch*0.5)], fill=metal_col)
    draw.polygon([(cx - hw, cy + hh - ch*1.5), (cx - hw + ch*1.5, cy + hh), (cx - hw + ch*0.5, cy + hh), (cx - hw, cy + hh - ch*0.5)], fill=metal_col)
    draw.polygon([(cx + hw, cy + hh - ch*1.5), (cx + hw - ch*1.5, cy + hh), (cx + hw - ch*0.5, cy + hh), (cx + hw, cy + hh - ch*0.5)], fill=metal_col)
    
    draw.rectangle([cx - hw + 8*s, cy - hh + 8*s, cx - hw + 12*s, cy + hh - 8*s], fill=(255, 255, 255, 200))
    
    draw.rectangle([cx - nw - 2*s, neck_top, cx + nw + 2*s, cy - hh], fill=metal_col, outline=outline_col, width=int(2*s))
    draw.rectangle([cx - nw*1.2, neck_top - cap_h, cx + nw*1.2, neck_top], fill=metal_col, outline=outline_col, width=int(2*s))
    draw.rectangle([cx - nw*1.2, neck_top - cap_h*0.6, cx + nw*1.2, neck_top - cap_h*0.3], fill=metal_dark)
    draw.arc([cx - 5*s, neck_top - cap_h - 7*s, cx + 5*s, neck_top - cap_h + 3*s], start=180, end=360, fill=metal_col, width=int(3*s))
    
    return img.resize((size, size), Image.Resampling.LANCZOS)

def create_flask_blue_teardrop(size=96):
    """Style 4: Curvilinear Teardrop / Pear Amphora (Blue Deep Water/Mana)"""
    s = 2.0
    W = int(size * s)
    img = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    cx = W / 2.0
    base_cy = W / 2.0 + 16.0 * s
    r_base = 25.0 * s
    neck_w = 7.0 * s
    neck_top = base_cy - 46.0 * s
    stopper_h = 13.0 * s
    outline_col = (12, 16, 26, 255)
    
    num_pts = 60
    body_pts = []
    for i in range(num_pts):
        ang = -math.pi * 0.15 + (math.pi * 1.3) * (i / float(num_pts - 1))
        x = cx + r_base * math.cos(ang)
        y = base_cy + r_base * math.sin(ang)
        body_pts.append((x, y))
    body_pts.append((cx - neck_w, base_cy - 24.0 * s))
    body_pts.append((cx - neck_w, neck_top))
    body_pts.append((cx - neck_w - 3*s, neck_top - 3*s))
    body_pts.append((cx + neck_w + 3*s, neck_top - 3*s))
    body_pts.append((cx + neck_w, neck_top))
    body_pts.append((cx + neck_w, base_cy - 24.0 * s))
    
    for ox in range(-5, 6):
        for oy in range(-5, 6):
            if ox*ox + oy*oy <= 25:
                shifted = [(x + ox, y + oy) for x, y in body_pts]
                draw.polygon(shifted, fill=outline_col)
                draw.polygon([
                    (cx - 3*s + ox, neck_top + oy),
                    (cx + 3*s + ox, neck_top + oy),
                    (cx + neck_w*1.1 + ox, neck_top - stopper_h*0.6 + oy),
                    (cx + ox, neck_top - stopper_h + oy),
                    (cx - neck_w*1.1 + ox, neck_top - stopper_h*0.6 + oy),
                ], fill=outline_col)

    inner_pts = [(cx + (x - cx)*0.86, base_cy + (y - base_cy)*0.88) for x, y in body_pts if y > base_cy - 20.0*s]
    inner_pts.append((cx + neck_w - 2*s, neck_top + 2*s))
    inner_pts.append((cx - neck_w + 2*s, neck_top + 2*s))
    draw.polygon(inner_pts, fill=(15, 25, 42, 220))
    
    liq_img = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    liq_draw = ImageDraw.Draw(liq_img)
    
    liq_y = base_cy - 4.0 * s
    lr = r_base - 5.0 * s
    liq_draw.ellipse([cx - lr, base_cy - lr, cx + lr, base_cy + lr], fill=(25, 145, 250, 255))
    liq_draw.ellipse([cx - lr*0.85, base_cy + lr*0.2, cx + lr*0.85, base_cy + lr*0.95], fill=(12, 70, 160, 240))
    liq_draw.rectangle([0, 0, W, liq_y], fill=(0, 0, 0, 0))
    liq_draw.ellipse([cx - lr*0.95, liq_y - 5*s, cx + lr*0.95, liq_y + 5*s], fill=(140, 225, 255, 255))
    
    img = Image.alpha_composite(img, liq_img)
    draw = ImageDraw.Draw(img)
    
    draw.arc([cx - lr*0.6, base_cy - 1*s, cx + lr*0.6, base_cy + 12*s], start=30, end=150, fill=(180, 240, 255, 220), width=int(2.5*s))
    draw.arc([cx - lr*0.4, base_cy + 6*s, cx + lr*0.4, base_cy + 16*s], start=210, end=330, fill=(180, 240, 255, 180), width=int(2*s))
    
    draw.arc([cx - r_base + 7*s, base_cy - r_base + 7*s, cx + r_base - 7*s, base_cy + r_base - 7*s], start=125, end=195, fill=(255, 255, 255, 240), width=int(4*s))
    draw.ellipse([cx - r_base*0.5, base_cy - r_base*0.4, cx - r_base*0.25, base_cy - r_base*0.15], fill=(255, 255, 255, 220))
    
    draw.polygon([(cx - neck_w - 3*s, neck_top - 3*s), (cx + neck_w + 3*s, neck_top - 3*s), (cx + neck_w, neck_top + 3*s), (cx - neck_w, neck_top + 3*s)], fill=(190, 215, 240, 255), outline=outline_col, width=int(2*s))
    
    gem_pts = [
        (cx, neck_top - stopper_h),
        (cx + neck_w*0.95, neck_top - stopper_h*0.55),
        (cx + 3*s, neck_top),
        (cx - 3*s, neck_top),
        (cx - neck_w*0.95, neck_top - stopper_h*0.55),
    ]
    draw.polygon(gem_pts, fill=(40, 160, 255, 255), outline=outline_col, width=int(2*s))
    draw.polygon([(cx, neck_top - stopper_h), (cx - neck_w*0.5, neck_top - stopper_h*0.55), (cx, neck_top - stopper_h*0.35)], fill=(200, 240, 255, 240))
    
    return img.resize((size, size), Image.Resampling.LANCZOS)

def create_flask_white_crystal(size=96):
    """Style 5: Hexagonal Cut Gemstone Vial (White Astral Ether/Purity)"""
    s = 2.0
    W = int(size * s)
    img = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    cx = W / 2.0
    cy = W / 2.0 + 8.0 * s
    hw = 23.0 * s
    body_h = 30.0 * s
    neck_w = 7.5 * s
    neck_top = cy - body_h - 8.0 * s
    crystal_h = 16.0 * s
    outline_col = (14, 18, 28, 255)
    
    body_pts = [
        (cx, cy + body_h + 4.0 * s),
        (cx + hw, cy + body_h * 0.4),
        (cx + hw, cy - body_h * 0.5),
        (cx + neck_w + 3*s, cy - body_h),
        (cx + neck_w, neck_top),
        (cx - neck_w, neck_top),
        (cx - neck_w - 3*s, cy - body_h),
        (cx - hw, cy - body_h * 0.5),
        (cx - hw, cy + body_h * 0.4),
    ]
    
    for ox in range(-5, 6):
        for oy in range(-5, 6):
            if ox*ox + oy*oy <= 25:
                shifted = [(x + ox, y + oy) for x, y in body_pts]
                draw.polygon(shifted, fill=outline_col)
                draw.polygon([
                    (cx + ox, neck_top - crystal_h + oy),
                    (cx + neck_w*1.15 + ox, neck_top - crystal_h*0.5 + oy),
                    (cx + neck_w*0.7 + ox, neck_top + oy),
                    (cx - neck_w*0.7 + ox, neck_top + oy),
                    (cx - neck_w*1.15 + ox, neck_top - crystal_h*0.5 + oy),
                ], fill=outline_col)

    inner_pts = [(cx + (x - cx)*0.84, cy + (y - cy)*0.84) for x, y in body_pts]
    draw.polygon(inner_pts, fill=(26, 32, 45, 220))
    
    liq_img = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    liq_draw = ImageDraw.Draw(liq_img)
    
    liq_y = cy - 8.0 * s
    liq_pts = [
        (cx - hw + 4*s, liq_y),
        (cx + hw - 4*s, liq_y),
        (cx + hw - 4*s, cy + body_h * 0.38),
        (cx, cy + body_h + 1*s),
        (cx - hw + 4*s, cy + body_h * 0.38),
    ]
    liq_draw.polygon(liq_pts, fill=(230, 240, 252, 255))
    liq_draw.polygon([
        (cx - hw*0.7, cy + body_h * 0.4),
        (cx + hw*0.7, cy + body_h * 0.4),
        (cx, cy + body_h + 1*s),
    ], fill=(175, 195, 230, 230))
    
    liq_draw.line([cx - hw + 4*s, liq_y, cx + hw - 4*s, liq_y], fill=(255, 255, 255, 255), width=int(4*s))
    
    img = Image.alpha_composite(img, liq_img)
    draw = ImageDraw.Draw(img)
    
    draw.line([cx, liq_y, cx, cy + body_h + 1*s], fill=(200, 215, 245, 180), width=int(2*s))
    draw.line([(cx - hw + 4*s), cy + body_h*0.38, cx, cy + body_h + 1*s], fill=(200, 215, 245, 180), width=int(2*s))
    draw.line([(cx + hw - 4*s), cy + body_h*0.38, cx, cy + body_h + 1*s], fill=(200, 215, 245, 180), width=int(2*s))
    
    sx, sy, sr = cx, cy + 2.0 * s, 10.0 * s
    star_pts = [
        (sx, sy - sr),
        (sx + sr*0.25, sy - sr*0.25),
        (sx + sr, sy),
        (sx + sr*0.25, sy + sr*0.25),
        (sx, sy + sr),
        (sx - sr*0.25, sy + sr*0.25),
        (sx - sr, sy),
        (sx - sr*0.25, sy - sr*0.25),
    ]
    draw.polygon(star_pts, fill=(255, 255, 255, 255))
    draw.ellipse([sx - sr*0.2, sy - sr*0.2, sx + sr*0.2, sy + sr*0.2], fill=(255, 255, 255, 255))
    
    draw.polygon([
        (cx - hw + 4*s, cy - body_h * 0.45),
        (cx - hw + 10*s, cy - body_h * 0.45),
        (cx - neck_w - 1*s, cy - body_h + 2*s),
        (cx - neck_w - 4*s, cy - body_h + 2*s),
    ], fill=(255, 255, 255, 240))
    
    draw.rectangle([cx - neck_w - 2*s, neck_top, cx + neck_w + 2*s, neck_top + 4*s], fill=(215, 225, 240, 255), outline=outline_col, width=int(2*s))
    
    cap_pts = [
        (cx, neck_top - crystal_h),
        (cx + neck_w, neck_top - crystal_h*0.5),
        (cx + neck_w*0.7, neck_top),
        (cx - neck_w*0.7, neck_top),
        (cx - neck_w, neck_top - crystal_h*0.5),
    ]
    draw.polygon(cap_pts, fill=(240, 245, 255, 255), outline=outline_col, width=int(2*s))
    draw.polygon([(cx, neck_top - crystal_h), (cx - neck_w*0.4, neck_top - crystal_h*0.5), (cx, neck_top)], fill=(255, 255, 250))
    
    return img.resize((size, size), Image.Resampling.LANCZOS)

def create_slot_frame(size=80, active=False):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    s = size / 80.0
    
    # Dark alchemical stone background
    draw.rounded_rectangle([4*s, 4*s, size - 4*s, size - 4*s], radius=10*s, fill=(18, 22, 28, 220))
    
    if active:
        # Ornate Glowing Gold Border (3-layer filigree)
        draw.rounded_rectangle([2*s, 2*s, size - 2*s, size - 2*s], radius=12*s, outline=(255, 215, 60, 255), width=int(3*s))
        draw.rounded_rectangle([6*s, 6*s, size - 6*s, size - 6*s], radius=8*s, outline=(245, 175, 25, 200), width=int(2*s))
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
    
    # Generate 5 Distinct Potion Flasks (each with unique shape, color, stopper & effects)
    print("Generating 5 distinct stylized alchemical potion flasks...")
    create_flask_green_spherical(96).save("assets/textures/ui/flask_green.png")
    create_flask_red_conical(96).save("assets/textures/ui/flask_red.png")
    create_flask_yellow_square(96).save("assets/textures/ui/flask_yellow.png")
    create_flask_blue_teardrop(96).save("assets/textures/ui/flask_blue.png")
    create_flask_white_crystal(96).save("assets/textures/ui/flask_white.png")
        
    # Generate Slot Frames
    create_slot_frame(80, active=False).save("assets/textures/ui/slot_normal.png")
    create_slot_frame(80, active=True).save("assets/textures/ui/slot_gold_active.png")
    
    print(">>> Successfully generated all stylized UI textures in assets/textures/ui/")
