# Regeln & Diretrizes do Jogo - IFPF Game Jam (Alquimia e Bichinhos)

Este documento registra todas as regras obrigatórias e especificações de design estabelecidas para o desenvolvimento deste projeto.

---

## 📜 1. Diretrizes Fundamentais de Design

1. **Ausência Total de Texto em Jogo (Diegetic Design)**:
   - O jogo **NÃO possui elementos textuais**, exceto na tela inicial de menu.
   - Todo o feedback de jogabilidade, tutoriais e status do jogador são puramente visuais e diegéticos.
   - **HUD Visual**:
     - Canto superior esquerdo: Vida indicada por corações estilizados (mecânica visual inspirada em Minecraft).
     - Centro inferior: Hotbar de poções composta por slots quadrados, contendo um número identificador discreto no canto superior esquerdo do slot (atalho do teclado/toque) e a ilustração/ícone 3D único do frasco de cada poção.

2. **Estilo Artístico**:
   - Estética **Anime / Cartoon** combinada com **sombras realistas**, iluminação de contato e ambient occlusion.
   - Shading estilizado (Cel/Toon Shading) com iluminação dinâmica (lanterna de cabeça, fungos e musgos bioluminescentes).
   - **Origem dos Assets**: Todos os modelos 3D, malhas e assets artísticos devem ser autorais, criados via **Blender** ou **Workbench**, sem uso de IA generativa de imagens ou modelos 3D (conforme regulamento oficial da Game Jam).

3. **Ciclo de Jogo e Rejogabilidade**:
   - Estrutura clássica: Início (Cutscene/Praia), Meio (Exploração e Sobrevivência no Labirinto) e Fim (Saída da Caverna).
   - **Alta Rejogabilidade**: Cada nova partida gera um sistema de cavernas subterrâneas totalmente inédito e aleatório.

4. **Escala e Métricas do Mundo**:
   - Altura do jogador: **1,80m** (unidade de referência: 1.0 unidade = 1 metro).
   - **Distância Mínima da Saída**: O algoritmo de geração procedural calcula a distância do trajeto caminhável (passos do jogador) e garante que a saída **NUNCA fique a menos de 500 metros** do ponto inicial.

5. **Câmera e Perspectiva em Primeira Pessoa (First-Person Perspective)**:
   - A visão de jogabilidade é estritamente em **PRIMEIRA PESSOA** (câmera alinhada na altura dos olhos a 1,68m).
   - O corpo do jogador não obstrui a visão: a malha do personagem é mantida oculta durante a exploração na primeira pessoa, evitando qualquer clipping ou bloqueio de tela.
   - A lanterna direcional frontal e a lanterna radial 360° acompanham em tempo real a rotação do olhar (yaw e pitch) e os passos com sutil head-bobbing.
   - O arremesso de poções segue o vetor 3D exato para onde o jogador está olhando.

---

## 🎬 2. Cutscene Inicial 2D e Loading Oculto

- **Formato**: Animação 2D estilizada (Anime/Cartoon).
- **Enredo**:
  1. O alquimista/explorador (traje estilo Indiana Jones, mochila grande nas costas, pochete/bolsa utilitária) adentra uma pirâmide antiga.
  2. Ele alcança o pedestal e pega uma estátua dourada.
  3. Pausa tensa de 1 segundo (sensação de que nada deu errado).
  4. O chão se abre sob seus pés!
  5. Câmera em perspectiva olhando para cima: a abertura do teto se fecha na escuridão.
  6. Corte dinâmico: o alquimista sendo arrastado pela correnteza violenta de um rio subterrâneo.
  7. O fluxo de água o cospe em uma margem de areia / pequena praia subterrânea.
- **Transição de Controle**:
  - O jogador assume o controle em 3D exatamente no instante em que ele se levanta na praia da caverna (ponto inicial fixo da geração).
  - **Inviolabilidade da Cutscene**: A cutscene atua como **tela de carregamento ativa** (loading mask) enquanto a Wave Function Collapse (WFC), o Dual Grid e o pathfinding calculam o labirinto de 500m+ em background. Por isso, **não pode ser pulada**.

---

## 🕳️ 3. Geração Procedural da Caverna (Skill: `procedural-dungeon-generator`)

- **Arquitetura Dual Grid & WFC**:
  - Separação estrita entre Camada Lógica (WFC de grafos/células) e Camada Visual (Dual Grid com 6 peças canônicas universais).
  - Deformação proporcional orgânica (Stålberg's quad grid) para eliminar ângulos retos e rigidez mecânica.
  - Multi-tile Pattern Matching para grandes salões, abismos e lagos.
- **Biomas Subterrâneos Aleatórios**:
  - **Setores Inundados / Alagados**: Cavernas com água rasa e profunda, estalactites gotejantes.
  - **Setores Geotérmicos / Fontes Termais**: Gêiseres liberando vapor quente e água termal borbulhante.
  - **Bosques de Fungos Bioluminescentes**: Setores escuros iluminados naturalmente por cogumelos e musgos brilhantes procedurais.
- **Iluminação e Visibilidade**:
  - A caverna é primariamente escura.
  - Fontes de luz: Lanterna de cabeça do explorador (Spotlight móvel cônico) + emissão dos fungos e musgos nas paredes e solo.

---

## 🐾 4. Bichinhos e Monstros Fofos (Skill: `procedural-animation`)

- **Temática da Jam**: Alquimia e Bichinhos.
- **Tipos de Criaturas**:
  - Slimes saltitantes (deformação squash & stretch procedural em tempo real).
  - Pequenos lobos / canídeos da caverna com cauda procedural e marcha reativa.
  - Criaturas luminescentes com apêndices dinâmicos via restrições de distância e FABRIK.
- **Comportamento e Distribuição Ecológica**:
  - Cada espécie possui preferência de habitat (ex: slimes aquáticos nos setores alagados; criaturinhas térmicas perto dos gêiseres).
  - I.A. autônoma com rotas de patrulha e detecção de proximidade do jogador para encontros orgânicos.

---

## 🧪 5. Sistema Alquímico de Poções

- O jogador arremessa ou usa frascos de poções para interagir com o ambiente e criaturas:
  - **Poção de Cola**: Cria poças viscosas que desaceleram ou prendem monstros temporariamente.
  - **Poção de Fogo**: Acende áreas inflamáveis, queima obstáculos vegetais ou afasta certas criaturas.
  - **Poção de Gelo**: Congela poças d'água para criar plataformas sólidas ou imobiliza alvos.
  - **Poção de Choque**: Conduz corrente elétrica através de trechos alagados, atingindo múltiplos monstros.
  - **Poção de Luz**: Projeta um orbe luminescente de longa duração para iluminar grandes cavernas.
- **Controles Estilo Skyrim (FPS Imersivo)**:
  - **Pointer Lock Automático**: Ao iniciar ou clicar na tela, o jogo captura o cursor do mouse (`requestPointerLock`).
  - **Mouse Look Fluido**: Movimentar o mouse gira a visão livremente (yaw e pitch) em 360°, sem precisar clicar ou arrastar.
  - **Clique Esquerdo**: Arremessa imediatamente a poção equipada na direção do olhar.
  - **Roda do Mouse (Wheel)**: Alterna ciclicamente entre os 5 frascos de poção da hotbar.
  - **Teclado 1 a 5**: Seleção direta do slot de poção correspondente.
  - **Movimentação**: `W` (frente), `S` (trás), `A` (passo lateral esquerdo), `D` (passo lateral direito).
  - **Correr (Sprint)**: `Shift` ou `Ctrl` para corrida veloz contínua.
  - **Pular**: `Espaço`.
  - **Mobile**: Joystick virtual à esquerda para movimentação; toque e arraste no lado direito para girar a visão; toque rápido para arremesso; botões táteis dedicados para pulo e sprint.

---

## 🧪 6. Testes Unitários Obrigatórios

- **100% das mecânicas lógicas e geradores devem conter testes automatizados**:
  - Algoritmo de geração da dungeon (WFC, conectividade sem deadlocks, garantia de distância >= 500m até a saída).
  - Dual Grid e indexação das 6 peças canônicas.
  - Sistema de restrições de distância e FABRIK das animações procedurais.
  - Sistema de dano, corações de vida e consumo de poções.
  - Patrulha e estados da I.A. dos bichinhos.

---

## 🔒 7. Privacidade do Repositório

- O repositório deve permanecer estritamente **local e privado** até ordem explícita do usuário.
- Nenhum push ou sincronização remota deve ser efetuada sem consentimento direto.

---

## 🎨 8. Regra do Menu Minimalista (Limite Estrito de 4 Palavras)

- **Contagem Máxima de Palavras**: A tela de menu não pode conter mais do que **4 palavras no total** em toda a interface.
  - Exemplo canônico: Título `"ALQUIMIA DAS PROFUNDEZAS"` (3 palavras) + Botão `"JOGAR"` (1 palavra).
- **Sem Parágrafos ou Textos Explicativos**: Nenhum subtítulo, texto de tutorial ou parágrafo de controles pode estar presente no menu.
- **Aparência e Design (Impeccable Craft)**: Design minimalista refinado, tipografia elegante com brilho âmbar/alquímico sutil, vidro translúcido com `backdrop-filter: blur`, 100% responsivo para landscape mobile sem scrollbars ou transbordamento.

---

## 📱 9. Arquitetura Mobile First e Controles Táteis

- **Orientação Exclusivamente Horizontal (Landscape)**:
  - O jogo bloqueia e adverte o usuário com tela cheia caso o dispositivo esteja em orientação vertical (retrato).
- **Métricas de Toque**:
  - Todos os alvos de toque interativos (joystick, botões de ação de pulo e corrida, slots da hotbar) devem respeitar a dimensão mínima de 44x44px (idealmente 48x48px).
- **Isolamento Multi-touch e Gestos**:
  - `touch-action: none` e `user-scalable=no` para prevenir zoom e scroll acidentais do navegador mobile.
  - Suporte a multi-touch desvinculado: o jogador pode mover o joystick analógico com o polegar esquerdo enquanto pula, corre ou arremessa poções com o polegar direito sem interferência.
- **Safe Area Insets**:
  - Interface respeita `env(safe-area-inset-*)` em entalhes (notches) e barras dinâmicas.

---

## 💡 10. Iluminação Subterrânea e Espalhamento Amplo de Luz

- **Lanterna de Cabeça Híbrida**:
  - Feixe Direcional Frontal (Spotlight): cone amplo (abertura > 65°), alcance de 45m com penumbra suave para visualização distante.
  - Luz Radial de Proximidade (Point Light 360°): raio de 18-20m ao redor do explorador para banhar o piso sob os pés, paredes laterais e teto, eliminando qualquer sensação de "cone cego" ou escuridão total adjacente.
- **Luz Ambiente Subterrânea**:
  - Preenchimento difuso suave que impede que áreas não iluminadas diretamente virem buracos negros (#000000), preservando o contraste e relevo das cavernas.
- **Bioluminescência Ativa**:
  - Cogumelos e musgos emitem luz vívida em tons de ciano e azul, servindo como faróis visuais para navegação natural no labirinto.


---

## ⚡ 11. Otimização de Performance WebGL e Geometria Agrupada

- **Fusão de Geometria (Geometry Batching via BufferGeometryUtils)**:
  - Todas as milhares de peças do Dual Grid (pisos, paredes, tetos e lâminas d'água) são pré-transformadas e fundidas em um número reduzido de `BufferGeometry` consolidadas por bioma antes da renderização.
  - Isso reduz o volume de draw calls da GPU em mais de 99% (de mais de 16.000 chamadas por quadro para menos de 15 chamadas), eliminando travamentos de CPU/GPU e garantindo 60 FPS consistentes em dispositivos mobile e computadores com gráficos integrados.
- **Eficiência de Iluminação Dinâmica**:
  - Eliminação de dezenas de `PointLight` estáticas por cogumelo em favor de materiais cel-shading com alta intensidade emissiva (`emissiveIntensity: 2.6`).
  - Prevenção do estouro de uniformes e shaders WebGL, mantendo iluminação nítida, contrastada e performática com a lanterna de cabeça direcional e luz radial do alquimista.
