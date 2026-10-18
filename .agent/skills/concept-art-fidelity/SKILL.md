---
name: concept-art-fidelity
description: Metodologia obrigatória de Technical Artist para traduzir concept arts e referências visuais em geometria 3D procedural, shaders de computação gráfica e composição de cena sem programar no escuro. Use SEMPRE que criar ou ajustar cenários 3D, rochas, vegetação, iluminação ou quando o usuário solicitar fidelidade à concept art/artwork.
---

# Pipeline de Tradução de Concept Art para Código (Technical Artist)

Agentes de IA frequentemente falham ao traduzir concept arts para o jogo porque interpretam imagens de forma semântica ("tem uma pedra, um banco, uma cachoeira") e instanciam primitivas básicas no código (cubos, cilindros lisos e planos retos), resultando em cenários blocados, artificiais e lavados.

Esta skill define o protocolo obrigatório em 4 técnicas para fazer o agente "enxergar" detalhes geométricos e traduzi-los em código de renderização e modelagem procedural.

---

## Técnica 1: O Loop de "Visual Diff" (Comparação Lado a Lado)

Nunca codifique cenários "no escuro" ou apenas com memória textual. O agente deve comparar o estado atual renderizado da cena contra a imagem de referência:

1. **Gere o Frame Atual:** Execute `cargo run -- --screenshot-test` para capturar a visão real da câmera em `debug_output/frame_check.png`.
2. **Execute a Análise Comparativa:** Execute `python scripts/visual_diff_blueprint.py debug_output/frame_check.png`.
3. **Inspeção Multimodal Lado a Lado:** Examine a imagem gerada comparando com o concept art e responda aos 3 eixos fundamentais:
   - **Enquadramento e FOV:** A câmera está permitindo ver a caverna ampla em arco ou há geometrias bloqueando a visão central?
   - **Oclusão Indevida:** Existem objetos desproporcionais ou planos retos artificiais na linha de visão?
   - **Equilíbrio de Contraste:** As sombras estão profundas e os elementos bioluminescentes vibrantes, ou a cena está cinza e lavada?

---

## Técnica 2: Tradução de Detalhes Artísticos em Regras de Shaders e Geometria

Traduza qualquer descrição artística em primitivas matemáticas de computação gráfica:

### 1. Musgo Orgânico no Topo (World-Space Normal $N_y > 0.6$)
- **Erro comum:** Pintar objetos inteiros de verde ou usar cubos com texturas genéricas.
- **Regra Técnica:** O shader deve ler a normal da superfície no espaço de mundo (`world_normal.y`). Sempre que $N_y > 0.6$ (superfícies horizontais voltadas para cima), aplica-se uma transição (via `smoothstep` combinado a ruído) para o verde-musgo (`#246e34`). As laterais ($N_y \le 0.6$) mantêm a pedra cinza-escura (`#26333a`) ou a madeira rústica (`#362215`).

### 2. Rochas Facetadas vs. Paredes Lisas (Proibição de Cuboides Puros)
- **Erro comum:** Instanciar `Cuboid` ou cubos lisos esticados como paredes de pedra.
- **Regra Técnica:** Toda rocha deve ser gerada a partir de icosaedros ou esferas subdivididas com deslocamento de vértices por ruído tridimensional (Perlin/Simplex/Voronoi) e normais planas (`poly.use_smooth = False` ou flat shading) para criar facetas angulares naturais. Sobreponha dezenas de blocos em camadas para formar penhascos orgânicos.

### 3. Contornos Pretos (Ink Lines) e Toon Shading
- **Erro comum:** Usar iluminação PBR fotorrealista contínua com gradientes cinzentos.
- **Regra Técnica:** Use Toon Shading com 2 a 3 faixas bem marcadas (stepping na iluminação difusa) e detecção de bordas via filtro Sobel em pós-processamento, isolando superfícies líquidas e fluidas para não gerar linhas pretas artificiais no meio de quedas d'água.

### 4. Caminho de Terra Batida vs. Laterais Gramadas
- **Erro comum:** Colocar faixas cinzas ou retângulos beges sobrepostos.
- **Regra Técnica:** No centro inferior ($|X| < 2.5$), o terreno é terra batida marrom quente (`#472914` a `#5c3826`), delimitada por seixos fluviais soltos e rochas angulares de transição nas bordas, dando lugar a bancos de grama densa nas laterais ($|X| > 2.5$).

---

## Técnica 3: Etapa de "Extração de Blueprint" Antes do Código

Nunca altere o código e analise a imagem na mesma etapa. Divida o processo obrigatoriamente em duas fases:

### Passo 1: Extração da Blueprint Técnica
Gere uma tabela técnica antes de programar, documentando:
- **Paleta Hexadecimal Exata:**
  - Água profunda / Água rasa / Espuma: `#05385c` / `#1cd2ea` / `#ebfbff`
  - Cogumelos bioluminescentes (Ciano / Pálido): `#15e8d8` / `#8be8f5`
  - Musgo verde aveludado: `#246e34`
  - Terra batida: `#472914`
  - Madeira rústica de tábuas: `#362215`
  - Sombra da caverna vs. Rocha iluminada: `#18222b` / `#36454e`
- **Tabela de Z-Depth e Posições (X, Y, Z):**
  - **Foreground ($Z \in [-1.0, -3.0]$):** Caminho de terra marrom no centro, pedras angulares de transição, banco de madeira escura com musgo no canto direito (`X \approx 2.6, Z \approx -2.1`).
  - **Midground ($Z \in [-5.0, -8.5]$):** Lagoa circular ampla com borda fina de pedra nivelada com a água; cachoeira central vertical descendente com névoa e gotículas de impacto.
  - **Background & Teto ($Z < -9.0$ e $Y > 15.0$):** Paredes rochosas facetadas com topos musgosos, vinhas verticais e arco superior escuro com estalactites pontiagudas.

### Passo 2: Implementação e Refatoração
Com os dados técnicos documentados no contexto, aplique as alterações nos scripts Blender (`scripts/build_grotto_scene.py`, `scripts/generate_alchemy_props.py`) e nos shaders WGSL (`assets/shaders/`).

---

## Técnica 4: Validação e Critérios de Conclusão (Definition of Done)

Uma tarefa de arte de cenário só é dada como concluída após:
1. `cargo run -- --screenshot-test` executado sem erros.
2. `python scripts/validate_frame.py debug_output/frame_check.png` aprovado:
   - Densidade de bordas Sobel $\ge 0.80\%$
   - Pixels cinza monótono $\le 30.0\%$
   - Pixels bioluminescentes vibrantes $\ge 15.0\%$
3. Visualização da imagem capturada (`view_file` em `frame_check.png` ou `frame_check_squeezed.png`) atestando a harmonia estética com a concept art.

---

## Técnica 5: Protocolo "Zero-Vision Loop First" e Redução de Tokens de Visão

Para economizar 75% a 90% dos tokens de visão e acelerar iterações de arte:

1. **Consulta via `scene_spec.json` (Zero-Vision):**
   - Em vez de reenviar imagens de referência a cada turno, consulte [`assets/data/scene_spec.json`](file:///C:/Users/fhgam/Documents/GitHub/gamelab/assets/data/scene_spec.json). Ele contém todas as paletas Hex, limites de câmera e especificações espaciais cacheadas pelo modelo com custo mínimo.
2. **Diagnóstico via Telemetria de Frustum (`camera_frustum_dump.json`):**
   - Durante `--screenshot-test`, a engine exporta [`debug_output/camera_frustum_dump.json`](file:///C:/Users/fhgam/Documents/GitHub/gamelab/debug_output/camera_frustum_dump.json).
   - O agente lê a posição das entidades, distâncias e diagnósticos de oclusão em **texto puro (<100 tokens)** sem gastar visão.
3. **Middleware Vision Squeezer:**
   - Quando a inspeção visual for necessária, execute `python scripts/vision_squeezer.py debug_output/frame_check.png`.
   - O script alinha a imagem aos tile boundaries do modelo (`640x352` ou crops de 512px em `debug_output/crops/`), reduzindo o custo de 2.200 para ~160-340 tokens por verificação.
4. **Automação de Viewport com Blender MCP:**
   - Utilize as ferramentas do servidor MCP `blender` (`get_scene_info`, `execute_blender_code`, `generate_3d`) para inspecionar nós de cena e gerar/importar geometrias sem codificar tentativas no escuro.

