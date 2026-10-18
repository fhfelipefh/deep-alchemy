---
name: cel-shading-3d-pipeline
description: Pipeline obrigatório para geração de assets 3D via Blender headless, UI em SVG e validação visual contra Concept Art. Use SEMPRE que criar ou modificar cenas 3D, shaders ou UI.
---

# Pipeline de Arte 3D, Shaders e Auditoria Visual

Você é PROIBIDO de modelar cenários, cogumelos, rochas ou ícones de UI usando primitivas nativas da engine (`Cuboid`, `Cylinder`, `Cone`, `Plane` ou retângulos de UI). Siga estritamente este workflow de 3 passos:

## Passo 1: Geração de Assets 3D via Blender Headless (`bpy`)
Sempre que precisar de um modelo 3D (caverna, parede rochosa, cogumelo curvo, cachoeira volumétrica, frasco):
1. Escreva um script Python para o Blender em `scripts/blender_gen_<asset>.py` usando `import bpy` e `bmesh`.
2. No script do Blender:
   - Use modificadores reais: `SUBSURF` (Subdivision Surface), `DISPLACE` (com texturas `VORONOI` ou `CLOUDS` para rochas e terreno) e `SIMPLE_DEFORM` (Bend/Taper para curvar caules de cogumelos).
   - Atribua materiais com cores base ricas e canal `Emission` nos objetos bioluminescentes.
   - Exporte como `.glb` para `assets/models/<asset>.glb` via `bpy.ops.export_scene.gltf()`.
3. Execute no terminal: `blender -b -P scripts/blender_gen_<asset>.py`.
   - *Nota no Windows:* `& "C:\Program Files\Blender Foundation\blender-5.4.0\blender.exe" -b -P scripts/blender_gen_<asset>.py`

## Passo 2: Geração de UI via SVG / Rasterização
Nunca desenhe quadrados coloridos para representar itens ou vida.
1. Gere arquivos `.svg` com paths vetoriais detalhados (Coração estilizado com borda preta e brilho; Frascos redondos de vidro com rolha e líquido colorido).
2. Converta para `.png` em `assets/ui/` usando Python (`cairosvg`, `resvg` ou `Pillow`) e carregue como `ImageNode` na UI.

## Passo 3: O "Visual Gatekeeper" (Validação de Screenshot Obrigatória)
Antes de encerrar sua tarefa, você DEVE rodar a validação visual:
1. Execute o jogo no modo captura: `cargo run -- --screenshot-test` (que salva `debug_output/latest_frame.png`).
2. Execute o script validador: `python scripts/validate_visuals.py debug_output/latest_frame.png`.
   - O script irá REPROVAR o build se:
     a) Mais de 20% dos pixels do terço inferior forem pretos (`RGB < 18,18,18` -> erro de chão sem luz).
     b) Não houver bordas pretas de alto contraste detectáveis (ausência de Outline Shader).
     c) Não houver pixels ciano/verde de alta luminosidade (ausência de Bloom/Emissive nos cogumelos).
3. **Inspeção Multimodal:** Use sua ferramenta de leitura de arquivos/imagem na `debug_output/latest_frame.png` e compare com `assets/reference_concept.jpg`. Se parecer "programmer art" (formas geométricas duras, sem contornos de quadrinhos ou cores mortas), corrija e repita o loop ANTES de responder ao usuário.
