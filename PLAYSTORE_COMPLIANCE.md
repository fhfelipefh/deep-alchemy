# Guia de Conformidade e Publicação na Google Play Store
## Vitriol

Este documento descreve a infraestrutura de controles touch screen e os requisitos técnicos implementados para compilação Android e publicação em conformidade com as diretrizes da **Google Play Store**.

---

## 🎮 1. Sistema de Controles Touch Screen

A aplicação possui suporte nativo para dispositivos móveis com controles virtuais diegéticos, estilizados com bordas douradas e ardósia translúcida em conformidade com a identidade visual Cel-Shaded do jogo:

1. **Analógico Virtual (Virtual Joystick - Lado Esquerdo)**:
   - Posicionado no canto inferior esquerdo com base circular e manípulo reativo.
   - Suporta movimentação analógica suave de 360 graus.
   - **Auto-Sprint Integrado**: Puxar o analógico além de 80% do raio ativa a corrida automaticamente, consumindo a barra de stamina conforme as regras de gameplay.
   - Ao soltar o toque, o manípulo retorna suavemente à posição central e o movimento cessa.

2. **Giro de Câmera e Olhar (Swipe / Drag - Lado Direito)**:
   - Qualquer toque/arraste no semicampo direito que não esteja sobre botões de ação controla a rotação da câmera (Yaw do jogador e Pitch da câmera).
   - Suporta multi-toque real: o jogador pode andar com o polegar esquerdo enquanto move a câmera e salta com o polegar direito em simultâneo.

3. **Botões de Ação Táteis (Cluster Inferior Direito)**:
   - **▲ Pular**: Ativa o buffer de pulo e verificação de solo.
   - **✦ Arremessar**: Lança a poção ativa do slot selecionado.
   - **◎ Mirar**: Ativa o arco balístico e trajetória preditiva da poção.
   - **E Interagir / Coletar**: Coleta ervas e ingredientes alquímicos destacados pela mira.
   - **▼ Agachar**: Alterna o estado de agachamento, reduzindo a altura do colisor e velocidade.

4. **Hotbar Tátil (Centro Inferior)**:
   - Toque direto nos slots 1 a 5 da barra inferior para trocar de poção instantaneamente, com animação de pop e borda iluminada ativa.

5. **Botão de Pausa e Menu (Topo Direito)**:
   - Botão `❚❚` posicionado no canto superior direito para alternar entre o jogo e o menu de pausa em dispositivos móveis que não possuem tecla Escape física.
   - Suporte nativo ao gesto de voltar do Android (Back gesture / KeyCode::BrowserBack).

---

## 📱 2. Conformidade com os Requisitos da Google Play Store

| Requisito Play Store | Status | Especificação Implementada |
|---|---|---|
| **Target SDK 34/35+** | ✅ Conforme | `targetSdkVersion = 35` (Android 15) configurado no `build.gradle.kts` e `Cargo.toml`. |
| **Min SDK** | ✅ Conforme | `minSdkVersion = 26` (Android 8.0 Oreo), garantindo suporte a Vulkan 1.1 e 64-bit. |
| **Suporte Obrigatório a 64-bit** | ✅ Conforme | Compilação nativa com ABI filters para `arm64-v8a` (obrigatório) e `x86_64` (Chromebooks e Google Play Games para PC). |
| **Formato de Distribuição .AAB** | ✅ Conforme | Tarefa Gradle `bundleRelease` configurada para gerar o **Android App Bundle (`.aab`)**. |
| **Permissões Mínimas (Zero Permissões Perigosas)** | ✅ Conforme | O `AndroidManifest.xml` não solicita permissões intrusivas (câmera, microfone, contatos, localização ou armazenamento externo), evitando reprovação no processo de revisão da Play Store. |
| **Orientation & Cutout (Notch/Bordas)** | ✅ Conforme | `sensorLandscape` com `windowLayoutInDisplayCutoutMode = shortEdges` para preenchimento de ponta a ponta sem barras pretas laterais. |
| **Hardware Features Declaradas** | ✅ Conforme | Declaração de `android.hardware.touchscreen`, `android.hardware.vulkan.version` e `android.glEsVersion` no manifesto. |
| **Ícone de Alta Resolução (512x512)** | ✅ Conforme | Imagem de alta resolução gerada em `artworks/playstore_icon_512.png` pronta para o Play Console. |
| **Gráfico de Recursos (Feature Graphic 1024x500)** | ✅ Conforme | Banner promocional de 1024x500 gerado em `artworks/playstore_feature_graphic.png`. |
| **Ícones Adaptativos (Adaptive Icons)** | ✅ Conforme | `ic_launcher.xml` e `ic_launcher_round.xml` configurados com foreground vetorial e background em `res/mipmap-anydpi-v26/`. |

---

## 🚀 3. Instruções de Compilação para Android

### Pré-requisitos
1. **Rust com Alvos Android**:
   ```bash
   rustup target add aarch64-linux-android x86_64-linux-android
   ```
2. **Android NDK & SDK**:
   Instale o Android Studio com NDK (r26 ou superior) e configure as variáveis de ambiente:
   - `ANDROID_HOME` (caminho do Android SDK)
   - `ANDROID_NDK_HOME` (caminho do NDK, ex: `$ANDROID_HOME/ndk/26.x.x`)
3. **Instalação do cargo-ndk ou cargo-apk**:
   ```bash
   cargo install cargo-ndk
   # ou
   cargo install cargo-apk
   ```

### Método 1: Gerando o Android App Bundle (.aab) para a Google Play Store
Execute o script automatizado:
```powershell
.\scripts\build_android.ps1 -Target bundle -Mode release
```
Ou no Linux/macOS:
```bash
./scripts/build_android.sh release bundle
```
O arquivo gerado para upload na Google Play Console estará em:
`android/app/build/outputs/bundle/release/app-release.aab`

### Método 2: Gerando APK para testes e sideload direto
```powershell
.\scripts\build_android.ps1 -Target apk -Mode release
```
O APK instalável estará em:
`android/app/build/outputs/apk/release/app-release.apk`

---

## 🔑 4. Geração de Chave de Assinatura (Keystore) para a Play Store

Antes de enviar para a Google Play Store, gere sua chave de assinatura de produção:

```bash
keytool -genkeypair -v -keystore release-key.jks -keyalg RSA -keysize 2048 -validity 10000 -alias deepalchemy
```

Adicione as variáveis de ambiente ou configure no `android/app/build.gradle.kts`:
```kotlin
signingConfigs {
    create("release") {
        storeFile = file(System.getenv("KEYSTORE_PATH") ?: "release-key.jks")
        storePassword = System.getenv("KEYSTORE_PASSWORD")
        keyAlias = System.getenv("KEY_ALIAS") ?: "deepalchemy"
        keyPassword = System.getenv("KEY_PASSWORD")
    }
}
```

---

## 📋 5. Checklist de Envio na Google Play Console

1. **Conta de Desenvolvedor**: Acessar [Google Play Console](https://play.google.com/console).
2. **Criar Novo App**:
   - Nome: *Vitriol*
   - Idioma padrão: *Português (Brasil)*
   - Tipo de app: *Jogo*
   - Modelo: *Gratuito*
3. **Configuração de Presença na Loja**:
   - Ícone do app: Fazer upload de `artworks/playstore_icon_512.png`.
   - Imagem de recurso (Feature Graphic): Fazer upload de `artworks/playstore_feature_graphic.png`.
   - Screenshots: Capturar telas em formato paisagem (16:9) usando o modo `--screenshot-test` ou imagens do jogo.
4. **Classificação de Conteúdo**:
   - Responder ao questionário IARC (Jogo de exploração e alquimia em cavernas, sem violência explícita).
5. **Público-alvo**:
   - Definir faixa etária (Geral / 10+).
6. **Política de Privacidade**:
   - Como o jogo não coleta dados pessoais, utilize uma declaração padrão de aplicativo offline sem coleta de telemetria de usuário.
7. **Lançamento (Release)**:
   - Fazer upload do arquivo `app-release.aab` na faixa de **Teste Fechado (Closed Testing)** ou **Produção**.
