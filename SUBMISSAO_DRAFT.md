# Rascunho de Submissão - IFPF Game Jam

Este documento contém todos os dados prontos e formatados para preencher a **Issue de Submissão** no repositório oficial da maratona:
👉 **[Abrir Nova Issue de Submissão na IFPF Game Jam](https://github.com/o-jonathan/IFPF_Game-Jam/issues/new?template=submiss%C3%A3o.yaml)**

---

### Título da Issue:
```
[SUBMISSÃO] Vitriol
```

---

### Campos do Formulário:

#### 1. Nome do jogo:
```
Vitriol
```

#### 2. Nome da equipe:
```
[Preencha com o nome da sua equipe cadastrada no painel IFSul]
```

#### 3. Repositório do projeto:
```
https://github.com/fhfelipefh/deep-alchemy
```
*(Caso o link do seu repositório no GitHub seja outro, substitua pelo link público do seu repositório)*

#### 4. Uso de Inteligência Artificial:
```
Sim, utilizamos IA na programação
```

#### 5. Assets externos:
```
Sim, utilizamos assets externos permitidos
```
*(Fontes Google Fonts sob licença aberta SIL OFL 1.1 e bibliotecas open-source MIT/Apache 2.0 listadas em CREDITOS.md)*

#### 6. Declarações obrigatórias:
Marque todas as caixas de seleção:
- [x] **Li e aceito o regulamento da IFPF Game Jam.**
- [x] **Minha equipe possui no máximo 5 integrantes.**
- [x] **O projeto possui uma versão jogável pronta para avaliação.**
- [x] **Não utilizamos IA generativa para criar arte, modelos 3D, texturas, música, efeitos sonoros ou dublagens.**
- [x] **Não utilizamos propriedade intelectual ou assets protegidos sem autorização.**
- [x] **Todos os assets externos permitidos estão devidamente creditados.**
- [x] **Declaramos corretamente o uso de IA na programação, quando aplicável.**

---

### Informações Complementares para a Banca Avaliadora:

- **Tema da Jam atendido:** *Alquimia & Bichinhos* — o jogador é um alquimista em primeira pessoa que manipula poções elementais (fogo, gelo, terra, ácido, vapor), acende tochas apagadas para se guiar, resolve quebra-cabeças no lago subterrâneo e interage com criaturas da masmorra (Sproutlings e Golem) que possuem rotas de patrulha e IA reativa com esquiva.
- **História em Quadrinhos Inicial (HQ):** Cutscene de introdução em 3 quadrinhos in-game (sem IA) retratando a queda da floresta no poço da cachoeira subterrânea.
- **Opções de Áudio e Volume:** Menu de configurações completo com controle de volume sonoro de 0% a 100% via botões passo a passo (`-` e `+`), aplicando dinamicamente a todos os efeitos sonoros.
- **Efeitos Sonoros Procedurais:** Todos os sons (arremesso de poção, impacto/estilhaço de vidro, splash na água, combustão de fogo, cristalização de gelo, piados dos monstrinhos e clique na interface) foram sintetizados via código matemático em Rust (`src/audio/synth.rs`).
- **Arquivos Obrigatórios na Raiz:**
  - [`TERMO_IA.md`](TERMO_IA.md) — Declaração de transparência de uso de IA na programação.
  - [`CREDITOS.md`](CREDITOS.md) — Créditos da equipe e licenças de recursos.
