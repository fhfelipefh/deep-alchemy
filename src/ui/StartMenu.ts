export class StartMenu {
  private overlay: HTMLDivElement;
  private rotateNotice: HTMLDivElement;
  private onStartCallback: () => void;

  constructor(onStart: () => void) {
    this.onStartCallback = onStart;

    this.rotateNotice = document.createElement('div');
    this.rotateNotice.id = 'rotate-notice';
    this.setupRotateNotice();
    document.body.appendChild(this.rotateNotice);

    this.overlay = document.createElement('div');
    this.overlay.id = 'start-menu';
    this.setupMenuOverlay();
    this.renderContent();
    document.body.appendChild(this.overlay);

    this.checkOrientation();
    window.addEventListener('resize', () => this.checkOrientation());
    window.addEventListener('orientationchange', () => this.checkOrientation());
  }

  private setupRotateNotice(): void {
    this.rotateNotice.style.position = 'fixed';
    this.rotateNotice.style.top = '0';
    this.rotateNotice.style.left = '0';
    this.rotateNotice.style.width = '100vw';
    this.rotateNotice.style.height = '100vh';
    this.rotateNotice.style.background = '#0a0806';
    this.rotateNotice.style.zIndex = '9999';
    this.rotateNotice.style.display = 'none';
    this.rotateNotice.style.flexDirection = 'column';
    this.rotateNotice.style.alignItems = 'center';
    this.rotateNotice.style.justifyContent = 'center';
    this.rotateNotice.style.color = '#fff';
    this.rotateNotice.style.fontFamily = 'system-ui, -apple-system, sans-serif';
    this.rotateNotice.style.textAlign = 'center';
    this.rotateNotice.style.padding = '24px';
    this.rotateNotice.style.boxSizing = 'border-box';

    this.rotateNotice.innerHTML = `
      <div style="font-size: 48px; margin-bottom: 16px; animation: rotatePhone 1.6s infinite ease-in-out;">📱 ➔ 🔄</div>
      <div style="font-size: 18px; font-weight: bold; color: #ffd700; margin-bottom: 8px;">GIRE O DISPOSITIVO</div>
      <div style="font-size: 14px; color: #c4b5a2; max-width: 320px; line-height: 1.4;">
        Este jogo foi desenvolvido exclusivamente para ser jogado na orientação horizontal (Landscape).
      </div>
      <style>
        @keyframes rotatePhone {
          0% { transform: rotate(0deg); }
          50% { transform: rotate(90deg); }
          100% { transform: rotate(0deg); }
        }
      </style>
    `;
  }

  private checkOrientation(): void {
    const isPortrait = window.innerHeight > window.innerWidth && window.innerWidth < 768;
    this.rotateNotice.style.display = isPortrait ? 'flex' : 'none';
  }

  private setupMenuOverlay(): void {
    this.overlay.style.position = 'fixed';
    this.overlay.style.top = '0';
    this.overlay.style.left = '0';
    this.overlay.style.width = '100vw';
    this.overlay.style.height = '100vh';
    this.overlay.style.background = 'radial-gradient(ellipse at center, #1e1710 0%, #080604 100%)';
    this.overlay.style.display = 'flex';
    this.overlay.style.alignItems = 'center';
    this.overlay.style.justifyContent = 'center';
    this.overlay.style.zIndex = '100';
    this.overlay.style.color = '#fff';
    this.overlay.style.fontFamily = 'system-ui, -apple-system, sans-serif';
    this.overlay.style.padding = 'clamp(8px, 2vh, 20px)';
    this.overlay.style.boxSizing = 'border-box';
    this.overlay.style.overflowY = 'auto';
  }

  private renderContent(): void {
    this.overlay.innerHTML = `
      <div style="
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
        gap: clamp(12px, 3vw, 28px);
        width: 100%;
        max-width: 900px;
        max-height: 94vh;
        align-items: center;
        background: rgba(18, 14, 10, 0.7);
        border: 1px solid rgba(255, 215, 0, 0.25);
        border-radius: 16px;
        padding: clamp(14px, 3vh, 24px);
        box-shadow: 0 10px 40px rgba(0,0,0,0.8), inset 0 0 20px rgba(255, 215, 0, 0.05);
        backdrop-filter: blur(8px);
      ">
        <div style="display: flex; flex-direction: column; justify-content: center;">
          <div style="font-size: 11px; text-transform: uppercase; letter-spacing: 2.5px; color: #ffd700; font-weight: bold; margin-bottom: 6px;">
            IFPF Game Jam 2026 • Alquimia & Bichinhos
          </div>
          <h1 style="font-size: clamp(24px, 4.5vw, 38px); line-height: 1.1; margin: 0 0 10px 0; color: #ffffff; text-shadow: 0 2px 14px rgba(255, 215, 0, 0.5);">
            ALQUIMIA DAS PROFUNDEZAS
          </h1>
          <p style="font-size: clamp(12px, 1.8vh, 14px); line-height: 1.4; color: #c4b5a2; margin: 0 0 16px 0;">
            Sobreviva ao colapso da pirâmide e escape do vasto sistema de cavernas subterrâneas usando suas poções alquímicas contra as criaturas do abismo.
          </p>
          <button id="btn-start-game" style="
            background: linear-gradient(135deg, #ffd700 0%, #e6a100 100%);
            border: none;
            color: #120e0a;
            padding: clamp(12px, 2.5vh, 16px) 28px;
            font-size: clamp(16px, 2.5vh, 19px);
            font-weight: 800;
            border-radius: 28px;
            cursor: pointer;
            box-shadow: 0 6px 20px rgba(255, 215, 0, 0.45);
            transition: transform 0.15s, box-shadow 0.15s;
            width: fit-content;
          ">
            INICIAR EXPEDIÇÃO
          </button>
        </div>

        <div style="
          background: rgba(0, 0, 0, 0.4);
          border: 1px solid rgba(255, 255, 255, 0.1);
          border-radius: 12px;
          padding: clamp(10px, 2vh, 16px);
          font-size: clamp(11px, 1.6vh, 12.5px);
          color: #a89a88;
          line-height: 1.5;
          display: flex;
          flex-direction: column;
          gap: 6px;
        ">
          <div style="font-weight: bold; color: #ffd700; font-size: 12px; margin-bottom: 2px;">
            🎮 CONTROLES (HORIZONTAL):
          </div>
          <div>
            <span style="color: #fff; font-weight: bold;">PC:</span> [WASD] Mover | [Ctrl] Correr | [Espaço] Pular | [1..5] Poções | [Clique] Arremessar
          </div>
          <div>
            <span style="color: #fff; font-weight: bold;">Mobile:</span> Joystick virtual esquerdo | Botões táteis [↑] [⚡] | Toque nos frascos inferiores e na tela
          </div>
          <div style="margin-top: 4px; padding-top: 6px; border-top: 1px solid rgba(255,255,255,0.08); font-size: 11px; color: #7d7265;">
            * Sem áudio por diretriz. Experiência 100% visual e diegética.
          </div>
        </div>
      </div>
    `;

    const btn = this.overlay.querySelector('#btn-start-game') as HTMLButtonElement;
    btn.addEventListener('mouseenter', () => {
      btn.style.transform = 'scale(1.04)';
      btn.style.boxShadow = '0 8px 25px rgba(255, 215, 0, 0.6)';
    });
    btn.addEventListener('mouseleave', () => {
      btn.style.transform = 'scale(1.0)';
      btn.style.boxShadow = '0 6px 20px rgba(255, 215, 0, 0.45)';
    });

    btn.addEventListener('click', async () => {
      try {
        if ('orientation' in screen && 'lock' in screen.orientation) {
          await (screen.orientation as any).lock('landscape').catch(() => {});
        }
      } catch {}

      this.hide();
      this.onStartCallback();
    });
  }

  public hide(): void {
    this.overlay.style.display = 'none';
  }

  public show(): void {
    this.overlay.style.display = 'flex';
  }
}
