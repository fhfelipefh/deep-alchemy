/**
 * Tela de Menu Inicial do Jogo.
 * ÚNICA tela com elementos textuais permitidos conforme especificação estrita do jogo.
 */

export class StartMenu {
  private overlay: HTMLDivElement;
  private onStartCallback: () => void;

  constructor(onStart: () => void) {
    this.onStartCallback = onStart;

    this.overlay = document.createElement('div');
    this.overlay.id = 'start-menu';
    this.overlay.style.position = 'fixed';
    this.overlay.style.top = '0';
    this.overlay.style.left = '0';
    this.overlay.style.width = '100vw';
    this.overlay.style.height = '100vh';
    this.overlay.style.background = 'radial-gradient(circle at center, #1c1510 0%, #080604 100%)';
    this.overlay.style.display = 'flex';
    this.overlay.style.flexDirection = 'column';
    this.overlay.style.alignItems = 'center';
    this.overlay.style.justifyContent = 'center';
    this.overlay.style.zIndex = '100';
    this.overlay.style.color = '#fff';
    this.overlay.style.fontFamily = 'system-ui, -apple-system, sans-serif';
    this.overlay.style.textAlign = 'center';
    this.overlay.style.padding = '20px';
    this.overlay.style.boxSizing = 'border-box';

    this.renderContent();
    document.body.appendChild(this.overlay);
  }

  private renderContent(): void {
    this.overlay.innerHTML = `
      <div style="max-width: 600px; width: 100%;">
        <div style="font-size: 13px; text-transform: uppercase; letter-spacing: 3px; color: #ffd700; margin-bottom: 8px; font-weight: bold;">
          IFPF Game Jam 2026 • Alquimia & Bichinhos
        </div>

        <h1 style="font-size: clamp(32px, 6vw, 52px); margin: 0 0 16px 0; color: #fff; text-shadow: 0 4px 20px rgba(255, 215, 0, 0.4); letter-spacing: 1px;">
          ALQUIMIA DAS PROFUNDEZAS
        </h1>

        <p style="font-size: 15px; line-height: 1.5; color: #bba993; margin-bottom: 28px;">
          Adentre as ruínas da pirâmide, sobreviva ao colapso e encontre a saída através de um vasto labirinto de cavernas subterrâneas utilizando suas poções alquímicas.
        </p>

        <button id="btn-start-game" style="
          background: linear-gradient(135deg, #ffd700 0%, #d49500 100%);
          border: none;
          color: #120e0a;
          padding: 16px 42px;
          font-size: 20px;
          font-weight: bold;
          border-radius: 30px;
          cursor: pointer;
          box-shadow: 0 6px 25px rgba(255, 215, 0, 0.4);
          transition: transform 0.15s, box-shadow 0.15s;
          margin-bottom: 30px;
        ">
          INICIAR EXPEDIÇÃO
        </button>

        <div style="
          background: rgba(255, 255, 255, 0.05);
          border: 1px solid rgba(255, 255, 255, 0.1);
          border-radius: 12px;
          padding: 16px;
          font-size: 13px;
          color: #9e9182;
          text-align: left;
          line-height: 1.6;
        ">
          <div style="font-weight: bold; color: #ffd700; margin-bottom: 6px;">🎮 GUIA DE CONTROLES:</div>
          <div>• <strong>PC Desktop:</strong> [WASD] Mover | [Ctrl] Correr | [Espaço] Pular | [1 a 5] Selecionar Poção | [Clique] Arremessar</div>
          <div style="margin-top: 4px;">• <strong>Android / Mobile:</strong> Joystick na tela para mover | Botões de Pulo e Corrida | Toque nos frascos da barra inferior para trocar e toque na tela para arremessar</div>
          <div style="margin-top: 8px; font-size: 11px; color: #736b63;">
            *Nota: Sem áudio por diretriz de desenvolvimento. A experiência em jogo é 100% visual e diegética.
          </div>
        </div>
      </div>
    `;

    const btn = this.overlay.querySelector('#btn-start-game') as HTMLButtonElement;
    btn.addEventListener('mouseenter', () => {
      btn.style.transform = 'scale(1.05)';
      btn.style.boxShadow = '0 8px 30px rgba(255, 215, 0, 0.6)';
    });
    btn.addEventListener('mouseleave', () => {
      btn.style.transform = 'scale(1.0)';
      btn.style.boxShadow = '0 6px 25px rgba(255, 215, 0, 0.4)';
    });

    btn.addEventListener('click', () => {
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
