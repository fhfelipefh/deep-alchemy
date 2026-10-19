---
name: game-feel-animation-pillars
description: Metodologia técnica e matriz de auditoria baseada nos 12 Princípios de Animação em Jogos (Chris Totten), Game Design Pillars (Carles Homs / Max Pears) e Game Feel com os 3 Cs e Presença (Celia Hodent / Steve Swink). Use SEMPRE que projetar, refinar, auditar ou implementar sistemas de movimentação, arremesso, combate, feedback sensorial (juice), câmera em primeira/terceira pessoa, animações procedurais ou para avaliar o que precisa ser melhorado na experiência do jogador.
---

# Game Feel, Animação para Jogos e Pilares de Design

Um jogo pode possuir mecânicas matematicamente perfeitas, código limpo e arquitetura sólida, mas ainda assim parecer morto, travado ou desconectado para o jogador. A diferença entre um sistema funcional e uma experiência memorável é o **Game Feel** (a sensação táctil da interação virtual), a aplicação consciente dos **12 Princípios de Animação Adaptados para Jogos** e a fidelidade estrita aos **Game Design Pillars** (Pilares de Design).

Esta skill consolida os fundamentos teóricos e a implementação prática para diagnosticar e elevar a qualidade interativa de qualquer projeto de jogo, com ênfase em engines modernas e arquitetura code-first (Bevy, Rust, shaders WGSL).

---

## 1. Os Três Fundamentos do Game Feel (Celia Hodent & Steve Swink)

O *Game Feel* trata do **prazer intrínseco de interagir com o jogo** ("Aspecto Brinquedo" / *Toy Aspect*), independente de metas, vitórias ou pontuação. É a razão pela qual pular com Mario em um cenário vazio ou girar a câmera em um ambiente imersivo já é gratificante por si só.

```
                  ┌────────────────────────────────────────┐
                  │               GAME FEEL                │
                  │       (Sensação Táctil Virtual)        │
                  └───────────────────┬────────────────────┘
                                      │
             ┌────────────────────────┴────────────────────────┐
             ▼                                                 ▼
   ┌───────────────────┐                             ┌───────────────────┐
   │    OS TRÊS Cs     │                             │    A PRESENÇA     │
   │                   │                             │                   │
   │  1. Camera        │                             │  1. Física        │
   │  2. Control       │                             │  2. Narrativa     │
   │  3. Character     │                             │  3. Emocional     │
   └───────────────────┘                             └───────────────────┘
```

### 1.1. Os Três Cs

1. **Camera (Câmera)**:
   - Define a relação espacial e o tom emocional do jogador com o mundo.
   - Câmeras em primeira pessoa com FOV estreito (60°-70°) geram claustrofobia e tensão; FOV dinâmico (85°-100°) com *kick* durante aceleração/corrida transmite velocidade e libertação.
   - Deve responder a acelerações com amortecimento sutil (*camera lag* controlado), inclinação lateral em curvas (*camera roll* de 0.5° a 1.5°) e oscilação orgânica (*head bobbing* em curvas de Lissajous), sem induzir cinetose (*motion sickness*).

2. **Control (Controles)**:
   - O elo direto entre a intenção cognitiva do jogador e a resposta do sistema.
   - O controle deve ser tão polido e responsivo que a interface desapareça da percepção: o jogador não pensa "vou apertar Espaço", ele pensa "vou saltar".
   - **Tolerâncias de Ouro de Controle**:
     - **Input Latency**: Resposta visual no frame subsequente ao acionamento do botão (< 16ms a 60 FPS).
     - **Coyote Time**: Janela de tolerância (70ms a 120ms) que permite ao jogador saltar logo após ter deixado uma plataforma ou borda no ar.
     - **Jump Buffering**: Registro prévio do comando de salto (100ms a 150ms antes de tocar o solo) que é executado no frame exato da aterrissagem.
     - **Step Offset Global**: Capacidade de transpor irregularidades e pequenas pedras de até 25cm sem exigir salto nem travar o deslizamento horizontal.
     - **Air Momentum**: Preservação da inércia vetorial de corrida ao sair do chão, sem desaceleração artificial pelo atrito do ar.

3. **Character (Personagem / Avatar)**:
   - Comunicação imediata de massa, intenção, vulnerabilidade e agilidade através da silhueta, animações e feedback sonoro.
   - O modelo visual do personagem deve espelhar fielmente suas capacidades mecânicas: peso no passo, recuo nas mãos ao golpear e deformação física nas frenagens.

### 1.2. As Três Dimensões da Presença

1. **Presença Física**: A sensação de que o controle desapareceu e o corpo virtual do jogador habita o espaço tridimensional.
2. **Presença Narrativa**: O ambiente reage a todas as ações: vidros quebram em fragmentos balísticos, líquidos espalham-se em poças tingidas persistentes, galhos dobram-se e tochas tremulam com o deslocamento de ar.
3. **Presença Emocional**: O jogador cria apego ao espaço ou às entidades; o ambiente possui mistério, perigo ou acolhimento através de contrastes de iluminação e som.

---

## 2. Game Design Pillars (Carles Homs & Max Pears)

Os Pilares de Design são os 3 a 5 conceitos basilares e emoções centrais que determinam o escopo, tom e identidade mecânica do jogo.

```
       [ Nova Ideia / Mecânica / Polimento ]
                         │
                         ▼
             ┌───────────────────────┐
             │  FILTRO DOS PILARES   │
             └───────────┬───────────┘
                         │
         ┌───────────────┴───────────────┐
         ▼                               ▼
 [ Reforça o Pilar? ]          [ Contradiz ou Dilui? ]
         │                               │
         ▼                               ▼
    [ IMPLEMENTAR ]                 [ DESCARTAR ]
```

### 2.1. Funções Críticas dos Pilares
- **Filtro de Decisão**: Nenhuma mecânica entra no projeto por capricho. Cada sistema (poções, movimentação, iluminação) deve justificar sua existência reforçando diretamente pelo menos um pilar.
- **Prevenção de Feature Creep**: Limita a expansão desordenada do escopo, garantindo que os recursos implementados sejam profundos e polidos em vez de superficiais e numerosos.
- **Alinhamento Criativo**: Garante que programadores, artistas e agentes de IA mantenham a mesma visão conceitual rigorosa.

### 2.2. Pilares de Referência para Deep Alchemy
1. **Física & Alquimia Táctil Sistêmica**: Todo frasco quebra fisicamente, fluidos interagem no solo, efeitos térmicos e geysers afetam alvos por proximidade contínua no espaço 3D.
2. **Exploração Orgânica & Atmosfera Bioluminescente**: Cavernas misteriosas com iluminação de cogumelos, materiais cel-shaded marcados por contornos e plantas reativas sem ruído visual excessivo.
3. **Fisicalidade em Primeira Pessoa Responsiva**: Movimentação com preservação de momento, peso corporal, mira balística calculada e resposta imediata sem lentidão artificial.

---

## 3. Os 12 Princípios de Animação Adaptados para Jogos (Chris Totten)

Os 12 princípios clássicos de animação de Frank Thomas e Ollie Johnston (*The Illusion of Life*) foram concebidos para mídia linear (filme). Nos jogos, a interatividade exige uma reinterpretação técnica profunda:

| Princípio Clássico | Adaptação para Jogos | Implementação Técnica em Código/Shaders |
| :--- | :--- | :--- |
| **1. Squash and Stretch** | Deformação elástica preservando volume sem alterar a hitbox física central. | Escala da malha visual por fator $(1+s, 1-s, 1+s)$ no pouso/impacto mantendo o colisor rígido intacto. |
| **2. Anticipation** | Telegrafo e janela de leitura para tomada de decisão do jogador. | Ataques inimigos com windup legível (20-40 frames). Ações do jogador com telegrafo instantâneo (1-3 frames) para não gerar sensação de atraso. |
| **3. Staging** | Clareza absoluta da ação principal contra o fundo. | Silhuetas cel-shaded com Sobel Edge, iluminação pontual em pontos de interesse e enquadramento limpo da câmera sem obstruções em primeiro plano. |
| **4. Straight Ahead vs. Pose-to-Pose** | Combinação de poses-chave (FSM/Layers) com simulação física procedural em tempo real. | Poses de animação interpoladas para estados padrão, somadas a Ragdoll, molas de inércia e cinemática inversa (FABRIK) em tempo real. |
| **5. Follow-Through & Overlapping** | Movimento residual defasado de partes soltas (cabelo, chapéu, adornos, líquidos). | Simulação de molas amortecidas (*spring-damper*) $F = -kx - cv$ nos vértices ou ossos acessórios após a parada do corpo principal. |
| **6. Slow In and Slow Out** | Curvas de aceleração/desaceleração não-lineares. | Uso de interpolações cúbicas e sigmóides (`smoothstep`, `ease_out_quad`) para transições de velocidade e movimentação da câmera. |
| **7. Arcs** | Trajetórias orgânicas curvilíneas e parabólicas. | Arremesso balístico com gravidade vetorial real ($y = y_0 + v_0 t - \frac{1}{2}gt^2$); head bobbing em curva sinusoidal em 8. |
| **8. Secondary Action** | Efeitos adicionais que amplificam a ação primária sem roubar o foco. | Partículas de esporos subindo nos passos, distorções de calor ao redor de poções térmicas e gotas pingando de frascos. |
| **9. Timing and Spacing** | Densidade de frames que define peso, agilidade e impacto dramático. | Ajuste milimétrico de startups, hitboxes ativas e frames de recuperação; uso de *Hitstop* (pausa de 2 a 4 frames no impacto crítico). |
| **10. Exaggeration** | Amplificação de eventos físicos para legibilidade e impacto sensorial. | Screen shake de alta frequência com decaimento exponencial; FOV kick na corrida; estilhaços de vidro com impulsos dispersos. |
| **11. Solid Drawing** | Volume tridimensional convincente, profundidade e equilíbrio visual. | Modelos com topologia limpa e facetas planas (*flat shading* intencional em rochas); iluminação consistente sem artefatos de clipping. |
| **12. Appeal** | Carisma, coerência artística e identidade estilística magnética. | Paleta de cores bioluminescente contrastante, interface integrada ao universo (HUD rústico em molduras vivas) e áudio com personalidade. |

---

## 4. O Kit de Ferramentas de Polimento Sensorial ("Juice")

O "Juice" é o conjunto de respostas audiovisuais simultâneas geradas por uma única ação básica, transformando entradas simples em sensações táteis viscerais:

```
[ Ação: Impacto de Poção / Salto / Colisão ]
                      │
   ┌──────────────────┼──────────────────┬──────────────────┐
   ▼                  ▼                  ▼                  ▼
[ Partículas ]   [ Screen Shake ]    [ Deformação ]     [ Áudio & Luz ]
Fragmentos de    Tremor direcional   Squash visual      Clarão emissivo
vidro e poça     com decaimento      temporário na      e som com tom
persistente      exponencial         malha              randômico
```

1. **Screen Shake Físico**:
   - Nunca use ruído uniforme linear. O tremor deve ter direção relacionada ao impacto e decair via curva exponencial:
     $$\text{Offset}(t) = \mathbf{D} \cdot A_0 \cdot e^{-\lambda t} \cdot \sin(2\pi f t)$$
     onde $\mathbf{D}$ é o vetor direcional normalizado do impacto, $A_0$ a amplitude inicial e $\lambda$ a taxa de amortecimento.
2. **Hitstop / Micro-Freeze**:
   - Em impactos de alta energia, congele o tempo de animação e física da entidade atingida por 30ms a 70ms. Isso confere sensação de densidade e resistência material.
3. **Persistência Material**:
   - Impactos deixam marcas: poças líquidas que duram mais de 60 segundos, decalques de queimadura ou gelo, e fragmentos que colidem com o solo antes de desaparecer.
4. **Variação Harmônica de Áudio**:
   - Nunca toque o mesmo som de forma idêntica. Varie o *pitch* entre $\pm 8\%$ e adicione camadas sutis de graves ou agudos baseados na força da colisão.

---

## 5. Matriz de Auditoria e Diagnóstico para Jogos

Sempre que avaliar sistemas de jogo existentes ou planejar novas implementações, execute o seguinte checklist de 5 fases:

### Fase 1: Auditoria de Controles e Locomoção
- [ ] O avatar reage no frame imediatamente posterior ao input?
- [ ] Existe *Coyote Time* (70-120ms) na borda de penhascos e plataformas?
- [ ] Há *Jump Buffering* (100-150ms) antes de tocar o solo?
- [ ] O *Step Offset* permite subir obstáculos pequenos (pedras, raízes, degraus de até 25cm) sem exigir salto?
- [ ] O *Air Momentum* é preservado ao pular durante a corrida rápida?

### Fase 2: Auditoria de Câmera e Percepção
- [ ] O FOV ajusta-se dinamicamente às mudanças de velocidade (kick de sprint)?
- [ ] O *head bobbing* segue uma trajetória orgânica suave em vez de um movimento linear rígido?
- [ ] Não há geometrias obstruindo ou cobrindo o centro da visão (decalques mal projetados ou polígonos na frente da lente)?
- [ ] O plano de corte próximo (*Near Clip Plane*) evita que a câmera atravesse malhas de primeiro plano?

### Fase 3: Auditoria de Projéteis e Trajetórias
- [ ] A mira balística reflete exatamente a trajetória física calculada com gravidade real?
- [ ] O indicador de mira projeta o ponto de impacto no solo ou obstáculo via *raycast sweep*?
- [ ] O projétil possui propagação contínua livre de autocolisão no frame de nascimento?
- [ ] Ao atingir o alvo, o projétil gera efeitos secundários imediatos (estilhaços balísticos, poça líquida persistente, transferência de efeitos alquímicos)?

### Fase 4: Auditoria de Legibilidade Visual e Staging
- [ ] Os contornos de pós-processamento (Sobel Edge) reforçam a silhueta sem poluir quedas d'água ou superfícies transparentes?
- [ ] A iluminação direciona o olhar do jogador para os pontos de interesse (luz guia bioluminescente)?
- [ ] A paleta de cores equilibra sombras profundas e emissão vibrante, evitando tons cinzentos e lavados?

### Fase 5: Auditoria de Coerência com os Pilares
- [ ] A mecânica sob teste reforça a identidade central do jogo ou é uma distração superficial?
- [ ] O sistema opera de forma sistêmica (interage com outros sistemas de física, elementos e alquimia)?
- [ ] O retorno sensorial justifica o tempo e a atenção despendidos pelo jogador?

---

## 6. Exemplo de Implementação: Pulo Orgânico com Gravidade Assimétrica e Squash em Bevy Rust

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct JumperLocomotion {
    pub velocity_y: f32,
    pub is_grounded: bool,
    pub jump_speed: f32,
    pub gravity_up: f32,
    pub gravity_down: f32,
    pub coyote_timer: f32,
    pub buffer_timer: f32,
}

#[derive(Component)]
pub struct VisualSquash {
    pub current_scale: Vec3,
    pub target_scale: Vec3,
    pub recovery_speed: f32,
}

pub fn update_organic_jump(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut JumperLocomotion, &mut VisualSquash)>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut jumper, mut squash) in &mut query {
        if jumper.is_grounded {
            jumper.coyote_timer = 0.10;
        } else {
            jumper.coyote_timer = (jumper.coyote_timer - dt).max(0.0);
        }

        if keys.just_pressed(KeyCode::Space) {
            jumper.buffer_timer = 0.12;
        } else {
            jumper.buffer_timer = (jumper.buffer_timer - dt).max(0.0);
        }

        if jumper.buffer_timer > 0.0 && jumper.coyote_timer > 0.0 {
            jumper.velocity_y = jumper.jump_speed;
            jumper.is_grounded = false;
            jumper.coyote_timer = 0.0;
            jumper.buffer_timer = 0.0;
            squash.current_scale = Vec3::new(0.85, 1.25, 0.85);
        }

        let effective_gravity = if jumper.velocity_y > 0.0 {
            jumper.gravity_up
        } else {
            jumper.gravity_down
        };

        if !jumper.is_grounded {
            jumper.velocity_y -= effective_gravity * dt;
            transform.translation.y += jumper.velocity_y * dt;

            if transform.translation.y <= 0.0 {
                transform.translation.y = 0.0;
                jumper.velocity_y = 0.0;
                jumper.is_grounded = true;
                squash.current_scale = Vec3::new(1.30, 0.70, 1.30);
            }
        }

        squash.current_scale = squash.current_scale.lerp(squash.target_scale, squash.recovery_speed * dt);
        transform.scale = squash.current_scale;
    }
}
```

---

## 7. Diretriz de Uso Obrigatório

Sempre que o usuário solicitar:
- Melhorias na "jogabilidade", "sensação de peso", "game feel" ou "fluidez";
- Refatoração ou auditoria de movimentação, arremesso de itens ou combate;
- Adição de novos efeitos visuais, câmeras ou partículas;

Consulte esta skill para identificar as lacunas sensoriais, validar o alinhamento com os Pilares de Design e aplicar os 12 Princípios de Animação em Jogos.
