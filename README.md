# Alquimia das Profundezas (Deep Alchemy)

Jogo web 3D desenvolvido para a **IFPF Game Jam** (Tema: *Alquimia e Bichinhos*), compatível com Desktop Web e Android Mobile Web.

---

## 🌟 Pilares do Projeto

- **Sem Texto em Jogo**: Toda a interface e comunicação com o jogador é puramente visual e diegética (corações no topo esquerdo estilo Minecraft, hotbar de poções com números de atalho e frascos únicos no centro inferior). Apenas o menu inicial possui texto.
- **Gráficos Cel-Shading Anime + Sombras Realistas**: Estilo cartoon/anime com iluminação dinâmica realista (lanterna de cabeça do alquimista, fungos e musgos bioluminescentes procedurais).
- **Geração Procedural de Cavernas (Dual Grid + WFC)**: Sistema de cavernas subterrâneas imenso e orgânico. A distância caminhável calculada até a saída **nunca é menor que 500 metros** (escala real: jogador com 1,80m).
- **Animações Procedurais (Argonaut Kinematics & FABRIK)**: Criaturas e bichinhos patrulham a caverna com movimentação e ciclos de marcha gerados por restrições geométricas e cinemática inversa.
- **Cutscene 2D Narrativa**: Cutscene inicial não-pulável que narra a queda do alquimista na pirâmide e mascara o tempo de geração do labirinto subterrâneo.
- **Pipeline de Modelagem Autoral**: Modelos e assets desenvolvidos com suporte a scripts de modelagem e exportação para **Blender**.
- **100% Testado**: Suíte de testes unitários automatizados para todas as mecânicas, regras de distância, resolução WFC e animações procedurais.

---

## 🛠️ Stack Tecnológica

- **Linguagem & Bundler**: TypeScript + Vite (garante inicialização ultrarrápida e 60 FPS em navegadores móveis Android e Desktop)
- **Motor Gráfico Web**: Three.js (WebGL 2 / WebGPU Ready) com materiais Cel-Shading customizados e sombras suaves (PCFSoftShadowMap)
- **Pipeline de Arte 3D**: Blender 4.5 / 5.4 (exportação glTF / GLB com animações e materiais)
- **Testes Unitários**: Vitest (execução ultraveloz de testes de mecânica, lógica de labirinto e IA)
- **Controles Híbridos**: Teclado/Mouse no PC + Joystick Virtual e botões táteis no Android Mobile

---

## 📜 Documentações de Referência
- [Regras e Especificações Oficiais](RULES.md)
- [Termo de Uso de IA](TERMO_USO_IA.md)
