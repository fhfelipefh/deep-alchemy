import sys
from PIL import Image
import numpy as np

def validate_frame(image_path):
    img = Image.open(image_path).convert("RGB")
    arr = np.array(img, dtype=np.float32)
    h, w, _ = arr.shape

    errors = []

    # 1. Checa o "Chao Preto Absoluto" (Terco inferior da tela)
    bottom_third = arr[int(h * 0.66):, :, :]
    black_pixels = np.mean(np.all(bottom_third < 18, axis=-1))
    if black_pixels > 0.20:
        errors.append(f"REPROVADO [Chao Escuro]: {black_pixels*100:.1f}% da parte inferior esta preta! Adicione luzes/textura no chao.")

    # 2. Checa a Bioluminescencia Ciano/Verde (Cogumelos e Agua)
    bright_cyan = np.mean((arr[:, :, 1] > 180) & (arr[:, :, 2] > 180) & (arr[:, :, 0] < 120))
    if bright_cyan < 0.03:
        errors.append(f"REPROVADO [Sem Bioluminescencia]: Apenas {bright_cyan*100:.2f}% da tela tem brilho ciano/turquesa vibrante.")

    # 3. Checa se a cena esta "Cinza / Monocromatica" (Arte de programador)
    r, g, b = arr[:,:,0], arr[:,:,1], arr[:,:,2]
    saturation = (np.maximum(np.maximum(r, g), b) - np.minimum(np.minimum(r, g), b)) / (np.maximum(np.maximum(r, g), b) + 1e-5)
    if np.mean(saturation) < 0.30:
        errors.append(f"REPROVADO [Cores Mortas/Cinzas]: Saturacao media ({np.mean(saturation):.2f}) muito baixa para Cel-Shading.")

    if errors:
        print("\n".join(errors))
        sys.exit(1)
    
    print("APROVADO nas metricas basicas! Agora faca a comparacao visual multimodal com a Concept Art.")
    sys.exit(0)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Uso: python validate_visuals.py <caminho_do_screenshot.png>")
        sys.exit(1)
    validate_frame(sys.argv[1])
