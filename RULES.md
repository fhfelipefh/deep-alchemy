# DIRETRIZES DE ARQUITETURA E ENGENHARIA (SKYRIM-LIKE & TRUE FIRST-PERSON)

Você é o Engenheiro de Jogos Principal deste projeto. O jogo é um RPG de exploração e alquimia em Primeira Pessoa Real (True First-Person) com densidade geométrica e interatividade estilo Skyrim, porém com pipeline visual Cel-Shaded detalhado. Siga estritamente as regras abaixo:

## 1. Geometria Densa, Modularidade e Otimização (Padrão RPG Imersivo)
* **Malhas Detalhadas com LOD Obrigatório:** Os modelos 3D (rochas, ruínas, flora, equipamentos) devem ter geometria rica e orgânica (Mid-to-High Poly), suportando silhuetas complexas. Todo asset de cenário ou prop complexo DEVE ser estruturado para suportar grupos de LOD (LOD0 para perto, LOD1, LOD2 para distância) e *Occlusion Culling*.
* **Kits Modulares de Dungeon (Estilo Creation Engine):** Cavernas e estruturas nunca devem ser um bloco único gigante. Arquiteture o level design usando Kits Modulares encaixáveis no grid (Paredes, Cantos, Pilares, Tetos, Transições de Bioma Termal), permitindo *Frustum Culling* eficiente peça por peça.
* **Colisores Compostos e Física de Props:** Como itens (poções, ferramentas, baús) interagem com o mundo, use colisores compostos (múltiplas primitivas ou *Convex Hulls* ajustados) para garantir física realista ao rolar no chão ou colidir, separando a malha de renderização (alta densidade) da malha física.
* **Integridade de Malha para Shaders:** Em malhas de alta densidade, mantenha *Vertex Normals* limpas e mapeamento UV sem sobreposição indesejada para que o pós-processamento de contornos (Depth/Normal Edge Detection) e mapas de detalhes internos funcionem sem artefatos visuais ("sujeira" nas linhas).

## 2. Sistema de Jogador: "True First-Person" (Corpo Inteiro, Reflexos e Morte)
* **Malha Única de Corpo Inteiro (Full-Body Awareness):** NUNCA use o truque antigo de "apenas braços flutuantes na câmera". O jogador (Alquimista Explorador) DEVE possuir o modelo 3D completo com mochila, pochetes e chapéu instanciado no mundo o tempo todo.
* **Visibilidade em Reflexos e Sombras:** Configure as *Render Layers* / *Shadow Casting* da cabeça e acessórios para `Shadows Only` (ou visível apenas para *Reflection Probes* / Planar Reflections / Espelhos d'água) enquanto em 1ª pessoa. Assim, a câmera não atravessa (clipping) o próprio chapéu/rosto, mas o corpo inteiro aparece perfeitamente nos reflexos da água da caverna e projeta sombra completa no chão.
* **Câmera de Morte e Ragdoll Físico:** O esqueleto (Rig) do jogador deve possuir um *Physical Asset / Ragdoll* configurado (articulações com limites de ângulo para corpo e mochila). Ao morrer (HP <= 0):
  1. Desative o controlador de movimento e o *Animator* principal.
  2. Torne a cabeça/chapéu 100% visíveis na camada principal da câmera.
  3. Ative a física de *Ragdoll* aplicando o vetor de força do último dano recebido.
  4. Transicione suavemente a câmera para uma visão externa (3ª pessoa/orbital) focando no personagem caindo e ficando deitado no chão da caverna.

## 3. Persistência de Mundo e Interação Estilo "Elder Scrolls"
* **Separação entre Definição Estática e Estado Dinâmico:**
  * *Dado Estático (Imutável):* ID do item, modelo 3D, textura, peso, efeitos alquímicos base (carregados de JSON/Data Assets).
  * *Estado de Instância (Mutável/Serializável):* Posição $(X,Y,Z)$ atual na caverna, rotação, se já foi colhido, quantidade no frasco. Isso permite salvar/carregar onde cada item caiu no chão.
* **Interação por Raycast Central:** O sistema de coleta (plantas, poções, baús) opera via Raycast a partir do centro da câmera, verificando a interface/componente `IInteractable`. Objetos pequenos podem ser puxados para o inventário ou manipulados fisicamente no espaço 3D.
* **Química e Alquimia Sistêmica:** Poções arremessadas ou ingeridas não usam código *hardcoded* por inimigo. Elas aplicam `ActiveEffects` (Magias/Efeitos Alquímicos com duração e magnitude) em qualquer entidade ou superfície que possua um `EffectReceiverComponent` (seja o próprio jogador, um Cogumelo inimigo ou um Gêiser termal).

## 4. Pipeline Visual: Cel-Shading de Alta Densidade + Água Reflexiva
* **Reflexos na Água e Superfícies:** Águas de lagoas e áreas termais devem combinar o estilo *Toon* (espuma nas bordas via Depth Buffer) com reflexos em tempo real (Planar Reflections ou Screen Space Reflections - SSR + Reflection Probes) para refletir o ambiente bioluminescente e a silhueta completa do Alquimista quando ele passar perto da margem.
* **Iluminação Atmosférica + Toon:** Mantenha suporte a *Volumetric Fog* (névoa dos gêiseres e profundidade da caverna) e múltiplas fontes de luz locais (cogumelos e poções na pochete) utilizando *Forward+* ou *Deferred Rendering* adaptado para quantizar a rampa de luz sem perder o clima sombrio de exploração.

## 5. Código Limpo, Performance e Zero-Gargalo
* **Proibição de Alocação no Loop Principal:** NUNCA aloque memória na Heap, faça I/O ou buscas caras de cena dentro do `Update/Tick`.
* **Object Pooling para Efeitos e Projéteis:** Frascos quebrados, partículas de vapor, esporos e decalques de poções no chão devem ser reciclados via *Object Pool*.
* **Desacoplamento por Eventos:** A UI (corações de vida e hotbar de poções) e o sistema de som reagem exclusivamente a eventos disparados pelo estado do jogador e do mundo, sem acoplamento direto.

---

## 📜 6. Diretrizes Gerais da Game Jam (Alquimia e Bichinhos)
* **Plataforma Exclusiva:** Desktop (PC).
* **Sem Elementos Textuais em Jogo:** Apenas a tela de menu possui texto (máximo 4 palavras). O jogo em si é 100% diegético e visual.
* **Origem dos Assets:** Todos os modelos 3D e texturas devem ser autorais criados via Blender / Workbench.
* **Escala do Mundo:** Altura do jogador = 1,80m. A saída do labirinto procedural nunca pode estar a menos de 500m de distância a pé da praia de início.
* **Cutscene Inicial 2D:** Exibe o alquimista na pirâmide pegando a estátua de ouro, o chão abrindo, caindo pelo rio subterrâneo até acordar na praia da caverna. Atua como máscara de carregamento.
