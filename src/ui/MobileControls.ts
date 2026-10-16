/**
 * Controles de Toque para Android e Dispositivos Móveis Web.
 * Joystick virtual responsivo para movimentação + botões de pulo e corrida.
 */

export interface MobileInputState {
  moveX: number; // -1.0 a 1.0
  moveZ: number; // -1.0 a 1.0
  isJumping: boolean;
  isRunning: boolean;
  throwTriggered: boolean;
}

export class MobileControls {
  private container: HTMLDivElement;
  private joystickBase: HTMLDivElement;
  private joystickThumb: HTMLDivElement;
  private jumpBtn: HTMLDivElement;
  private runBtn: HTMLDivElement;

  private isDraggingJoystick: boolean = false;
  private joystickTouchId: number | null = null;
  private baseCenter = { x: 0, y: 0 };
  private maxRadius = 45;

  public state: MobileInputState = {
    moveX: 0,
    moveZ: 0,
    isJumping: false,
    isRunning: false,
    throwTriggered: false,
  };

  constructor() {
    this.container = document.createElement('div');
    this.container.id = 'mobile-controls';
    this.container.style.position = 'fixed';
    this.container.style.top = '0';
    this.container.style.left = '0';
    this.container.style.width = '100%';
    this.container.style.height = '100%';
    this.container.style.pointerEvents = 'none';
    this.container.style.userSelect = 'none';
    this.container.style.touchAction = 'none';
    this.container.style.zIndex = '40';

    // 1. Base do Joystick Virtual (Canto inferior esquerdo)
    this.joystickBase = document.createElement('div');
    this.joystickBase.style.position = 'absolute';
    this.joystickBase.style.bottom = '40px';
    this.joystickBase.style.left = '40px';
    this.joystickBase.style.width = '110px';
    this.joystickBase.style.height = '110px';
    this.joystickBase.style.borderRadius = '50%';
    this.joystickBase.style.background = 'rgba(255, 255, 255, 0.12)';
    this.joystickBase.style.border = '2px solid rgba(255, 255, 255, 0.35)';
    this.joystickBase.style.pointerEvents = 'auto';
    this.joystickBase.style.display = 'flex';
    this.joystickBase.style.alignItems = 'center';
    this.joystickBase.style.justifyContent = 'center';

    // Manopla móvel
    this.joystickThumb = document.createElement('div');
    this.joystickThumb.style.width = '46px';
    this.joystickThumb.style.height = '46px';
    this.joystickThumb.style.borderRadius = '50%';
    this.joystickThumb.style.background = 'rgba(255, 255, 255, 0.6)';
    this.joystickThumb.style.boxShadow = '0 0 10px rgba(0,0,0,0.5)';
    this.joystickThumb.style.transform = 'translate(0px, 0px)';
    this.joystickThumb.style.pointerEvents = 'none';
    this.joystickBase.appendChild(this.joystickThumb);

    this.container.appendChild(this.joystickBase);

    // 2. Botão de Pulo (Canto inferior direito)
    this.jumpBtn = this.createActionButton('↑', '35px', '30px');
    this.container.appendChild(this.jumpBtn);

    // 3. Botão de Corrida (Ctrl equivalente)
    this.runBtn = this.createActionButton('⚡', '110px', '30px');
    this.container.appendChild(this.runBtn);

    this.setupTouchListeners();
    document.body.appendChild(this.container);

    // Esconde por padrão se for desktop puro sem toque
    const isTouchDevice = 'ontouchstart' in window || navigator.maxTouchPoints > 0;
    if (!isTouchDevice) {
      this.container.style.opacity = '0.4'; // Deixa sutil para testes com mouse se desejado
    }
  }

  private createActionButton(symbol: string, right: string, bottom: string): HTMLDivElement {
    const btn = document.createElement('div');
    btn.style.position = 'absolute';
    btn.style.right = right;
    btn.style.bottom = bottom;
    btn.style.width = '58px';
    btn.style.height = '58px';
    btn.style.borderRadius = '50%';
    btn.style.background = 'rgba(20, 20, 26, 0.75)';
    btn.style.border = '2px solid rgba(255, 255, 255, 0.4)';
    btn.style.display = 'flex';
    btn.style.alignItems = 'center';
    btn.style.justifyContent = 'center';
    btn.style.fontSize = '22px';
    btn.style.color = '#ffffff';
    btn.style.pointerEvents = 'auto';
    btn.style.boxShadow = '0 4px 10px rgba(0,0,0,0.4)';
    btn.innerText = symbol;
    return btn;
  }

  private setupTouchListeners(): void {
    // Joystick Touch
    this.joystickBase.addEventListener('pointerdown', (e) => {
      this.isDraggingJoystick = true;
      this.joystickTouchId = e.pointerId;
      const rect = this.joystickBase.getBoundingClientRect();
      this.baseCenter = {
        x: rect.left + rect.width / 2,
        y: rect.top + rect.height / 2,
      };
      this.updateJoystick(e.clientX, e.clientY);
      this.joystickBase.setPointerCapture(e.pointerId);
    });

    window.addEventListener('pointermove', (e) => {
      if (this.isDraggingJoystick && e.pointerId === this.joystickTouchId) {
        this.updateJoystick(e.clientX, e.clientY);
      }
    });

    const releaseJoystick = (e: PointerEvent) => {
      if (e.pointerId === this.joystickTouchId) {
        this.isDraggingJoystick = false;
        this.joystickTouchId = null;
        this.state.moveX = 0;
        this.state.moveZ = 0;
        this.joystickThumb.style.transform = 'translate(0px, 0px)';
      }
    };

    window.addEventListener('pointerup', releaseJoystick);
    window.addEventListener('pointercancel', releaseJoystick);

    // Botão de Pulo
    this.jumpBtn.addEventListener('pointerdown', () => {
      this.state.isJumping = true;
      this.jumpBtn.style.background = 'rgba(255, 215, 0, 0.6)';
    });
    this.jumpBtn.addEventListener('pointerup', () => {
      this.state.isJumping = false;
      this.jumpBtn.style.background = 'rgba(20, 20, 26, 0.75)';
    });

    // Botão de Corrida (Toggle ou Hold)
    this.runBtn.addEventListener('pointerdown', () => {
      this.state.isRunning = !this.state.isRunning;
      this.runBtn.style.background = this.state.isRunning
        ? 'rgba(255, 120, 0, 0.7)'
        : 'rgba(20, 20, 26, 0.75)';
    });
  }

  private updateJoystick(clientX: number, clientY: number): void {
    const dx = clientX - this.baseCenter.x;
    const dy = clientY - this.baseCenter.y;
    const dist = Math.hypot(dx, dy);

    if (dist === 0) {
      this.state.moveX = 0;
      this.state.moveZ = 0;
      this.joystickThumb.style.transform = 'translate(0px, 0px)';
      return;
    }

    const clampedDist = Math.min(this.maxRadius, dist);
    const nx = dx / dist;
    const ny = dy / dist;

    this.joystickThumb.style.transform = `translate(${nx * clampedDist}px, ${ny * clampedDist}px)`;

    // Normalizado (-1 a 1)
    this.state.moveX = nx * (clampedDist / this.maxRadius);
    this.state.moveZ = ny * (clampedDist / this.maxRadius);
  }
}
