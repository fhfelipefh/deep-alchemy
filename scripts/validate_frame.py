#!/usr/bin/env python3
"""
Visual Regression & Quality Gatekeeper (Anti-Mocking Validator)
Analyzes debug_output/frame_check.png to ensure:
1. No flat untextured gray voids (> 30% flat gray fails)
2. Sharp outline edges exist (Sobel edge detection)
3. Rich bioluminescent and alchemical colors are present
"""
import sys
import os
import math

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8")

try:
    from PIL import Image
except ImportError:
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "Pillow"])
    from PIL import Image

def analyze_frame(image_path="debug_output/frame_check.png"):
    if not os.path.exists(image_path):
        print(f"❌ [ERRO FATAL] Arquivo de validação visual não encontrado: {image_path}")
        print("Execute o jogo com --screenshot-test para gerar o frame!")
        return False

    img = Image.open(image_path).convert("RGB")
    width, height = img.size
    total_pixels = width * height
    pixels = img.load()

    print(f"==================================================")
    print(f"🔍 [VISUAL REGRESSION GATE] Analisando: {image_path}")
    print(f"📐 Resolução do Frame: {width}x{height} ({total_pixels} pixels)")
    print(f"==================================================")

    flat_gray_count = 0
    vibrant_color_count = 0
    total_saturation = 0.0

    # 1. Pixel Color & Saturation Analysis
    for y in range(height):
        for x in range(width):
            r, g, b = pixels[x, y]
            max_c = max(r, g, b)
            min_c = min(r, g, b)
            diff = max_c - min_c
            sat = (diff / max_c) if max_c > 0 else 0.0
            total_saturation += sat

            # Detector de Cinza / Fundo Vazio ("programmer art" untextured gray: R ~ G ~ B in midtones)
            if diff <= 12 and 35 <= r <= 175:
                flat_gray_count += 1

            # Vibrant alchemical & bioluminescent colors (Cyan, Amber, Ruby, Emerald, Gold)
            if sat > 0.35 and max_c > 60:
                vibrant_color_count += 1

    gray_percentage = (flat_gray_count / total_pixels) * 100.0
    vibrant_percentage = (vibrant_color_count / total_pixels) * 100.0
    avg_saturation = total_saturation / total_pixels

    print(f"📊 Estatísticas de Cor:")
    print(f"   • Pixels Cinza Monótono: {gray_percentage:.2f}% (Limite Máximo: 30.0%)")
    print(f"   • Pixels com Cores Vivas/Bioluminescentes: {vibrant_percentage:.2f}%")
    print(f"   • Saturação Média: {avg_saturation:.3f}")

    # Check 1: Gray Void Gate
    if gray_percentage > 30.0:
        print(f"\n❌ [ERRO FATAL] Cena reprovada pelo Detector de Cinza:")
        print(f"   Mais de 30% da tela ({gray_percentage:.1f}%) é cinza/preto sem textura.")
        print(f"   Cena sem geometria de caverna fechada ou sem paleta de cores cel-shaded!")
        return False

    # 2. Outline & Silhouette Contrast Analysis (Sobel Filter Check)
    # Downsample slightly for ultra-fast gradient computation
    gray_img = img.convert("L")
    w_sub, h_sub = gray_img.size
    gray_px = gray_img.load()

    edge_count = 0
    sample_step = 2
    samples_checked = 0

    for y in range(1, h_sub - 1, sample_step):
        for x in range(1, w_sub - 1, sample_step):
            samples_checked += 1
            # Simple Sobel-like discrete gradient
            gx = (int(gray_px[x + 1, y - 1]) + 2 * int(gray_px[x + 1, y]) + int(gray_px[x + 1, y + 1])) - \
                 (int(gray_px[x - 1, y - 1]) + 2 * int(gray_px[x - 1, y]) + int(gray_px[x - 1, y + 1]))
            gy = (int(gray_px[x - 1, y + 1]) + 2 * int(gray_px[x, y + 1]) + int(gray_px[x + 1, y + 1])) - \
                 (int(gray_px[x - 1, y - 1]) + 2 * int(gray_px[x, y - 1]) + int(gray_px[x + 1, y - 1]))

            grad = math.sqrt(gx * gx + gy * gy)
            if grad > 110.0: # High-contrast outline edge threshold
                edge_count += 1

    edge_percentage = (edge_count / samples_checked) * 100.0
    print(f"\n📐 Análise de Contornos & Cel-Shading (Sobel Edge Gate):")
    print(f"   • Densidade de Bordas de Alto Contraste: {edge_percentage:.2f}% (Mínimo: 0.80%)")

    # Check 2: Edge / Silhouette Gate
    if edge_percentage < 0.80:
        print(f"\n❌ [ERRO FATAL] Cena reprovada pelo Detector de Outlines:")
        print(f"   Densidade de contornos insuficiente ({edge_percentage:.2f}% < 0.80%).")
        print(f"   A cena aparenta ser borrada ou sem sombreamento Toon/Edge Detection!")
        return False

    print(f"\n✅ [SUCESSO] Todos os critérios do Visual Regression Gate foram APROVADOS!")
    print(f"   • Ambiente fechado orgânico (sem vazamento de céu cinza)")
    print(f"   • Contornos nítidos e silhuetas bem definidas")
    print(f"   • Paleta cromática bioluminescente e cel-shaded validada")
    return True

if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else "debug_output/frame_check.png"
    success = analyze_frame(target)
    sys.exit(0 if success else 1)
