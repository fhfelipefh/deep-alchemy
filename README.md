# Vitriol (Deep Alchemy)

Jogo 3D em primeira pessoa desenvolvido para a **IFPF Game Jam** (Tema: *Alquimia e Bichinhos*).

---

## 🎮 Como Baixar e Jogar Imediatamente (Sem Instalação)

Para os avaliadores da Game Jam ou jogadores que desejam testar o jogo imediatamente no Windows (sem precisar ter Rust ou compiladores instalados):

1. **Baixe a versão mais recente na página de [GitHub Releases](https://github.com/fhfelipefh/deep-alchemy/releases/latest).** O repositório mantém somente a release atual; os arquivos grandes do jogo não ficam duplicados na raiz do código.
2. Extraia o arquivo `.zip` em qualquer pasta do seu computador.
3. Dê duplo clique no executável **`deep_alchemy.exe`**.
4. Divirta-se! O jogo iniciará com uma introdução cinematográfica 3D e o menu principal.

Também é possível jogar no navegador em [Vitriol Web](https://fhfelipefh.github.io/deep-alchemy/). Ao abrir, a versão WebAssembly carrega os mesmos menus e a mesma introdução do desktop; na primeira visita o navegador baixa o jogo e seus recursos.

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
