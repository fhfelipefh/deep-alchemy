# Vitriol (Deep Alchemy)

Jogo 3D em primeira pessoa desenvolvido para a **IFPF Game Jam** (Tema: *Alquimia e Bichinhos*).

---

## 🎮 Como Baixar e Jogar Imediatamente (Sem Instalação)

Para os avaliadores da Game Jam ou jogadores que desejam testar o jogo imediatamente no Windows (sem precisar ter Rust ou compiladores instalados):

1. **Baixe o arquivo compactado disponível na raiz deste repositório:**
   👉 **[`Vitriol-v0.1.0-windows-x64.zip`](Vitriol-v0.1.0-windows-x64.zip)** *(ou baixe via [GitHub Releases](https://github.com/fhfelipefh/deep-alchemy/releases/tag/v0.1.0))*
2. Extraia o arquivo `.zip` em qualquer pasta do seu computador.
3. Dê duplo clique no executável **`deep_alchemy.exe`**.
4. Divirta-se! O jogo iniciará com a introdução cinematográfica 3D e o menu principal.

---

## 🕹️ Controles

- **W, A, S, D:** Movimentação (caminhar)
- **Shift Esquerdo:** Correr (consome estamina)
- **Barra de Espaço:** Pular
- **Mouse:** Olhar ao redor (visão em primeira pessoa)
- **Botão Esquerdo do Mouse:** Arremessar poção alquímica
- **Botão Direito do Mouse:** Mirar com trajetória balística
- **1 a 5 ou Scroll do Mouse:** Selecionar frasco de poção na hotbar (Fogo, Gelo, Terra, Ácido, Vapor)
- **E:** Interagir (acender tochas, coletar reagentes)
- **ESC:** Pausar jogo e abrir menu de configurações (áudio, sensibilidade, campo de visão e tela cheia)

---

## 🌟 Pilares do Projeto

- **Tema da Jam:** *Alquimia & Bichinhos* — Manipule poções com reações físico-químicas em tempo real (evaporação, congelamento de poças, choque térmico, combustão) e interaja com as criaturas da caverna (Sproutlings, Emberlings, Glacelings) que aprendem seus padrões e reagem aos elementos.
- **Gráficos Cel-Shading + Sombras:** Estilo cartoon/anime com iluminação dinâmica, tochas interativas, shaders WGSL customizados e reflexos planares na água subterrânea.
- **Geração Procedural de Masmorras:** Sistema de cavernas subterrâneas e labirintos gerados via Wave Function Collapse (WFC) e Dual Grid, garantindo conectividade e caminhos navegáveis com saída abissal.
- **Síntese de Áudio Procedural DSP:** Todos os efeitos sonoros (arremesso de poção, estilhaço de vidro, splash na água, combustão, congelamento, cliques de UI) foram sintetizados matematicamente via código Rust em tempo de execução (`src/audio/synth.rs`).
- **Cutscene Cinematográfica 3D:** Animação de introdução in-game narrando a descida do alquimista pelo poço da cachoeira subterrânea.
- **100% Testado:** Suíte com 285 testes automatizados (unitários, integração, arquitetura e desempenho).

---

## 🛠️ Stack Tecnológica

- **Linguagem:** Rust (edição 2021)
- **Motor de Jogo:** Bevy Engine 0.15.3 (arquitetura ECS nativa)
- **Física 3D:** Avian3D 0.2.1 (colisões, corpos rígidos e raycasting)
- **Shaders:** WGSL (WebGPU Shading Language)
- **Modelagem 3D:** Blender (modelos low-poly autorais exportados em glTF 2.0 / GLB)
- **Áudio:** Síntese DSP Procedural em código nativo

---

## 📜 Documentações Oficiais da IFPF Game Jam

- [Termo de Uso de Inteligência Artificial](TERMO_IA.md)
- [Créditos e Licenças](CREDITOS.md)
