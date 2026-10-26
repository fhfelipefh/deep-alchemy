# Créditos

Este arquivo reúne os créditos das pessoas, ferramentas e recursos utilizados na produção do jogo **Vitriol**, desenvolvido para a IFPF Game Jam.

## 1. Equipe de desenvolvimento

| Nome | Função ou contribuição |
|---|---|
| Felipe | Programação, Game Design, Shaders WGSL, Síntese de Áudio Procedural e Integração de Sistemas |

## 2. Recursos de terceiros

| Recurso | Autor ou criador | Fonte ou link | Licença |
|---|---|---|---|
| Bevy Engine 0.15.3 | Bevy Engine Contributors | https://bevyengine.org/ | MIT / Apache 2.0 |
| Avian3D 0.2.1 (Física 3D) | Jondolf | https://github.com/Jondolf/avian | MIT / Apache 2.0 |
| Fonte Roboto / Fira | Google Fonts / Christian Robertson | https://fonts.google.com/ | SIL Open Font License 1.1 |
| Efeitos Sonoros (SFX) | Autoral (Síntese DSP Procedural via Código Rust) | `src/audio/synth.rs` | Licença Aberta MIT / Autoral |
| Modelos 3D e Shaders | Autoral (Blender + WGSL Cel-shading / Sobel) | `assets/models/`, `assets/shaders/` | Licença Aberta MIT / Autoral |

*Nota: Conforme as diretrizes da IFPF Game Jam, nenhum asset comercial protegido ou extraído (ripped) foi utilizado.*

## 3. Ferramentas e tecnologias

- **Engine:** Rust + Bevy Engine 0.15.3 (arquitetura ECS e renderizador nativo)
- **Física 3D:** Avian3D 0.2.1 (colisores convexos, rigid bodies dinâmicos e raycasting espacial)
- **Linguagens de programação:** Rust (edição 2021) e WGSL (WebGPU Shading Language)
- **Ferramentas de criação:** Blender (modelagem low-poly autoral e exportação glTF 2.0 / GLB)
- **Áudio:** Sintetizador de áudio procedural via código matemático em Rust (`src/audio/synth.rs`), gerando ondas senoidais, ruído pseudo-aleatório filtrado, chirps exponenciais e envelopes ADSR
- **Plataformas-alvo:** Windows 64-bit e Android

## 4. Agradecimentos

Agradecimentos aos organizadores da **Primeira IFPF Game Jam** (IFSUL - Campus Passo Fundo) pela oportunidade e incentivo ao desenvolvimento independente de jogos, e à comunidade open-source de Rust e Bevy.

---

Todos os créditos foram registrados de acordo com as informações disponíveis sobre os recursos utilizados.
