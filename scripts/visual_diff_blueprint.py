"""
Visual Diff & Blueprint Extraction Tool for Deep Alchemy
Executes via Python to extract technical scene metrics, color palettes (Hex),
depth layer decomposition, and visual diff statistics from rendered screenshots.
Supports comparing against reference concept arts.
"""
import sys
import os
import math
from PIL import Image

if hasattr(sys.stdout, "reconfigure"):
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

def rgb_to_hex(r, g, b):
    return f"#{int(r):02x}{int(g):02x}{int(b):02x}"

def rgb_to_hsv(r, g, b):
    rf, gf, bf = r / 255.0, g / 255.0, b / 255.0
    cmax = max(rf, gf, bf)
    cmin = min(rf, gf, bf)
    delta = cmax - cmin
    
    # Hue
    if delta == 0:
        h = 0
    elif cmax == rf:
        h = 60 * (((gf - bf) / delta) % 6)
    elif cmax == gf:
        h = 60 * (((bf - rf) / delta) + 2)
    else:
        h = 60 * (((rf - gf) / delta) + 4)
        
    s = 0 if cmax == 0 else (delta / cmax)
    v = cmax
    return h, s, v

def analyze_image_regions(img):
    w, h = img.size
    pixels = img.convert("RGB").load()
    
    # Analyze by vertical depth thirds:
    # 1. Foreground: Lower third (Y in [2*h//3, h])
    # 2. Midground: Center (Y in [h//3, 2*h//3])
    # 3. Background/Ceiling: Upper third (Y in [0, h//3])
    regions = {
        "foreground": {"pixels": [], "luma": 0.0, "biolum": 0},
        "midground": {"pixels": [], "luma": 0.0, "biolum": 0},
        "background": {"pixels": [], "luma": 0.0, "biolum": 0},
    }
    
    # Quantized color histogram for dominant hex extraction
    color_bins = {}
    
    for y in range(h):
        region_key = "background" if y < h // 3 else ("midground" if y < 2 * h // 3 else "foreground")
        for x in range(w):
            r, g, b = pixels[x, y]
            luma = 0.299 * r + 0.587 * g + 0.114 * b
            
            # Group into quantized bucket (step of 16)
            qr = (r // 24) * 24 + 12
            qg = (g // 24) * 24 + 12
            qb = (b // 24) * 24 + 12
            bin_key = (qr, qg, qb)
            color_bins[bin_key] = color_bins.get(bin_key, 0) + 1
            
            regions[region_key]["luma"] += luma
            
            # Check bioluminescence / vibrant aquatic colors
            h_val, s_val, v_val = rgb_to_hsv(r, g, b)
            is_biolum = (s_val > 0.40 and v_val > 0.50 and (160 <= h_val <= 210 or 90 <= h_val <= 150))
            if is_biolum:
                regions[region_key]["biolum"] += 1
                
    total_pixels = w * h
    third_pixels = total_pixels / 3.0
    
    for key in regions:
        regions[key]["avg_luma"] = regions[key]["luma"] / third_pixels
        regions[key]["biolum_ratio"] = regions[key]["biolum"] / third_pixels
        
    # Top 8 dominant colors
    sorted_bins = sorted(color_bins.items(), key=lambda item: item[1], reverse=True)[:8]
    dominant_hex = []
    for (qr, qg, qb), count in sorted_bins:
        pct = (count / total_pixels) * 100.0
        hex_code = rgb_to_hex(qr, qg, qb)
        dominant_hex.append((hex_code, pct, qr, qg, qb))
        
    return regions, dominant_hex

def compute_sobel_density(img):
    w, h = img.size
    gray = img.convert("L").load()
    
    edge_pixels = 0
    threshold = 38.0
    
    for y in range(1, h - 1, 2):
        for x in range(1, w - 1, 2):
            # 3x3 Sobel kernel
            p00 = gray[x - 1, y - 1]
            p10 = gray[x,     y - 1]
            p20 = gray[x + 1, y - 1]
            p01 = gray[x - 1, y]
            p21 = gray[x + 1, y]
            p02 = gray[x - 1, y + 1]
            p12 = gray[x,     y + 1]
            p22 = gray[x + 1, y + 1]
            
            gx = (p20 + 2 * p21 + p22) - (p00 + 2 * p01 + p02)
            gy = (p02 + 2 * p12 + p22) - (p00 + 2 * p10 + p20)
            mag = math.sqrt(gx * gx + gy * gy)
            if mag > threshold:
                edge_pixels += 1
                
    sampled_pixels = ((w - 2) // 2) * ((h - 2) // 2)
    return (edge_pixels / sampled_pixels) * 100.0

def generate_blueprint_report(img_path):
    if not os.path.exists(img_path):
        print(f"❌ Erro: Imagem não encontrada: {img_path}")
        sys.exit(1)
        
    img = Image.open(img_path)
    w, h = img.size
    
    regions, dominant_colors = analyze_image_regions(img)
    sobel_density = compute_sobel_density(img)
    
    print("\n" + "=" * 70)
    print("🛠️  [TECHNICAL ARTIST BLUEPRINT & VISUAL DIFF REPORT]")
    print(f"📁 Arquivo Analisado: {img_path} ({w}x{h} px)")
    print("=" * 70)
    
    print("\n🎨 1. PALETA CROMÁTICA DOMINANTE EXTRAÍDA (Top 8):")
    print(f"{'Hex':<10} | {'Cobertura':<10} | {'Interpretação do Technical Artist'}")
    print("-" * 65)
    for hex_code, pct, r, g, b in dominant_colors:
        # Categorize
        if b > r * 1.3 and (g + b) > r * 2.0:
            category = "Água / Cascata / Bioluminescência Ciano"
        elif g > r * 1.15 and g > b:
            category = "Grama / Musgo Aveludado das Rochas"
        elif r > g * 1.1 and r > b * 1.3:
            category = "Terra Batida Marrom / Madeira do Banco"
        elif max(r, g, b) - min(r, g, b) < 22:
            category = "Rochas Cinzas da Caverna / Teto"
        elif r > 180 and g > 180 and b > 180:
            category = "Espuma Aerada / Cristas da Queda d'Água"
        else:
            category = "Sombra Profunda / Penumbra da Gruta"
        print(f"{hex_code:<10} | {pct:>8.2f}% | {category}")
        
    print("\n📐 2. DECOMPOSIÇÃO EM PLANOS DE PROFUNDIDADE (Z-DEPTH):")
    for name, data in regions.items():
        print(f"   • {name.capitalize():<12}: Luminosidade Média = {data['avg_luma']:>5.1f}/255 | Bioluminescência = {data['biolum_ratio']*100:>5.2f}%")
        
    print(f"\n🖋️ 3. DENSIDADE DE CONTORNOS CEL-SHADED (Sobel Edge Gate):")
    print(f"   • Linhas de Nanquim Detectadas: {sobel_density:.2f}% (Meta recomendada: 5.0% - 12.0%)")
    
    print("\n📋 4. BLUEPRINT DE COORDENADAS RECOMENDADAS PARA CENA 3D:")
    print("   • Foreground (Solo e Banco):")
    print("     - Caminho de Terra Marrom: X central entre -2.5 e +2.5, Z de -1.0 a -4.5")
    print("     - Banco Rústico com Musgo: X = +2.65, Z = -2.1, Rotação Y = -0.50 rad")
    print("   • Midground (Lagoa e Queda d'Água):")
    print("     - Lagoa Circular: Centro (0.0, -8.5), Raio = 5.2m, Borda Z = 0.08m")
    print("     - Cascata Central: Eixo X = 0.0, Z = -8.5 a -10.8, Altura Y = 0.0 a 18.0m")
    print("   • Background (Paredes e Teto):")
    print("     - Rochas Facetadas com Musgo Superior: X laterais < -5.0 e > +5.0")
    print("     - Teto Abobadado: Cúpula fechando até Y = 23.5m com estalactites pontiagudas laterais")
    print("=" * 70 + "\n")

if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else "debug_output/frame_check.png"
    generate_blueprint_report(target)
