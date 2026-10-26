# Termo de Uso de Inteligência Artificial

Este documento declara como ferramentas de Inteligência Artificial (IA) foram utilizadas durante o desenvolvimento deste projeto, submetido à IFPF Game Jam.

## 1. Uso de Inteligência Artificial

A equipe declara que foi feito o uso de ferramentas de inteligência artificial (IA) como assistente de programação durante o desenvolvimento do projeto **Vitriol** para a IFPF Game Jam.

## 2. Ferramentas utilizadas

| Ferramenta | Finalidade | Etapa ou recurso do projeto |
|---|---|---|
| Antigravity (Google DeepMind) | Assistência na escrita de código, refatoração e testes | Arquitetura em Rust com Bevy 0.15 e Avian3D |
| Assistentes de Código (LLM) | Auxílio com matemática procedural | Geração de masmorra procedural (WFC e Dual Grid) |
| Assistentes de Código (LLM) | Auxílio na sintaxe de shaders | Shaders WGSL cel-shaded e contornos Sobel |
| Assistentes de Código (LLM) | Depuração e testes unitários | Suíte de testes automatizados (`tests/unit_tests.rs`) |

## 3. Declaração e Conformidade Estrita com as Regras da Jam

1. **Código-fonte:** O uso de IA restringiu-se estritamente à programação e depuração lógica, conforme permitido pelo Item 1 das Diretrizes da IFPF Game Jam ("IA na Programação").
2. **Assets Audiovisuais (Regra 2 - Proibição de IA Generativa):** Em estrito cumprimento à regra oficial de que *"é terminantemente proibido o uso de IA generativa para criar arte (sprites, modelos 3D, texturas, ilustrações, concept art, interfaces), música, efeitos sonoros (SFX) ou dublagens"*:
   - **Nenhum asset visual ou sonoro foi gerado por IA generativa.**
   - Os modelos 3D low-poly foram construídos no **Blender** de forma autoral ou gerados por geometria procedural.
   - A interface do usuário (UI) e a história em quadrinhos da introdução foram construídas 100% in-game através da hierarquia de nós vetoriais e Bevy UI nativa.
   - Todos os efeitos sonoros (SFX) foram gerados via **código de síntese matemática DSP em Rust** (`src/audio/synth.rs`), sem qualquer modelo de IA de áudio ou samples protegidos por direitos autorais.

Declaramos que as informações apresentadas neste documento são verdadeiras e que o uso de ferramentas de Inteligência Artificial no projeto foi informado de maneira transparente e em total conformidade com o edital da Game Jam.
