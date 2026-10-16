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
- **Controles**:
  - Teclado 1 a 5 (ou toque direto no slot da HUD móvel) para selecionar a poção ativa.
  - Clique esquerdo / Botão de ação para arremessar.
  - Movimentação: WASD / Joystick Virtual touch.
  - Correr: `Ctrl` / Toque prolongado.
  - Pular: `Espaço` / Botão de pulo.

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
