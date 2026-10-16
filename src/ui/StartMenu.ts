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
      <div style="font-size: 52px; margin-bottom: 16px; animation: rotatePhone 1.6s infinite ease-in-out;">📱 ➔ 🔄</div>
      <div style="font-size: 20px; font-weight: bold; color: #ffd700; letter-spacing: 2px;">GIRE O DISPOSITIVO</div>
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
    this.overlay.style.background = 'radial-gradient(ellipse at center, rgba(30, 22, 14, 0.85) 0%, rgba(8, 9, 14, 0.98) 85%)';
    this.overlay.style.display = 'flex';
    this.overlay.style.alignItems = 'center';
    this.overlay.style.justifyContent = 'center';
    this.overlay.style.zIndex = '100';
    this.overlay.style.color = '#fff';
    this.overlay.style.fontFamily = 'system-ui, -apple-system, sans-serif';
    this.overlay.style.padding = '16px';
    this.overlay.style.boxSizing = 'border-box';
    this.overlay.style.overflow = 'hidden';
  }

  private renderContent(): void {
    this.overlay.innerHTML = `
      <div style="
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        text-align: center;
        width: 100%;
        max-width: 680px;
        box-sizing: border-box;
      ">
        <svg width="58" height="58" viewBox="0 0 64 64" fill="none" style="margin-bottom: clamp(10px, 2.5vh, 20px); filter: drop-shadow(0 0 14px rgba(255, 215, 0, 0.65));">
          <circle cx="32" cy="32" r="28" stroke="#ffd700" stroke-width="2" stroke-dasharray="4 3" opacity="0.8" />
          <polygon points="32,10 52,48 12,48" stroke="#ffd700" stroke-width="2.5" fill="none" />
          <polygon points="32,54 12,16 52,16" stroke="#e6a100" stroke-width="1.5" fill="none" opacity="0.6" />
          <circle cx="32" cy="32" r="5" fill="#ffd700" />
        </svg>

        <h1 style="
          font-size: clamp(24px, 5.2vw, 44px);
          font-weight: 900;
          letter-spacing: clamp(3px, 1vw, 7px);
          line-height: 1.15;
          margin: 0 0 clamp(18px, 4vh, 32px) 0;
          color: #ffffff;
          text-shadow: 0 0 25px rgba(255, 215, 0, 0.5), 0 4px 12px rgba(0,0,0,0.8);
          font-family: 'Cinzel', 'Trajan Pro', 'Georgia', serif, system-ui;
          text-transform: uppercase;
        ">
          ALQUIMIA DAS PROFUNDEZAS
        </h1>

        <button id="btn-start-game" style="
          background: linear-gradient(135deg, #ffd700 0%, #e6a100 100%);
          border: none;
          color: #110d08;
          min-width: clamp(170px, 26vw, 230px);
          height: clamp(50px, 8.5vh, 60px);
          font-size: clamp(16px, 2.6vh, 20px);
          font-weight: 900;
          letter-spacing: 4px;
          border-radius: 36px;
          cursor: pointer;
          box-shadow: 0 8px 28px rgba(255, 215, 0, 0.45), inset 0 1px 0 rgba(255, 255, 255, 0.6);
          transition: transform 0.18s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.18s ease;
          display: flex;
          align-items: center;
          justify-content: center;
          text-transform: uppercase;
          outline: none;
          touch-action: manipulation;
        ">
          JOGAR
        </button>
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

    btn.addEventListener('click', () => {
      try {
        if ('orientation' in screen && 'lock' in screen.orientation) {
          (screen.orientation as any).lock('landscape').catch(() => {});
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
