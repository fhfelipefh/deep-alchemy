import { Vec3 } from '../animation/KinematicsSolver';

export class PlayerState {
  public position: Vec3;
  public velocity: Vec3;
  public height: number = 1.80; // 1,80m conforme especificação estrita
  public maxHearts: number = 10; // 10 corações (estilo Minecraft)
  public currentHealth: number = 20; // 2 pontos por coração = 20 HP
  public isRunning: boolean = false;
  public isGrounded: boolean = true;
  public invulnerabilityTimer: number = 0;
  public isDead: boolean = false;
  public hasReachedExit: boolean = false;

  public readonly walkSpeed: number = 4.2;  // m/s
  public readonly runSpeed: number = 6.8;   // m/s (Ctrl ativado)
  public readonly jumpForce: number = 5.2;  // m/s pulo

  constructor(initialPos: Vec3 = { x: 0, y: 0.9, z: 0 }) {
    this.position = { ...initialPos };
    this.velocity = { x: 0, y: 0, z: 0 };
  }

  public takeDamage(amount: number): boolean {
    if (this.invulnerabilityTimer > 0 || this.isDead) {
      return false;
    }

    this.currentHealth = Math.max(0, this.currentHealth - amount);
    this.invulnerabilityTimer = 1.0; // 1 segundo de i-frames

    if (this.currentHealth <= 0) {
      this.isDead = true;
    }
    return true;
  }

  public heal(amount: number): void {
    if (this.isDead) return;
    this.currentHealth = Math.min(this.maxHearts * 2, this.currentHealth + amount);
  }

  public getDisplayHearts(): { full: number; half: number; empty: number } {
    const totalHalfHearts = Math.round(this.currentHealth);
    const full = Math.floor(totalHalfHearts / 2);
    const half = totalHalfHearts % 2;
    const empty = this.maxHearts - full - half;
    return { full, half, empty };
  }

  public update(deltaTime: number): void {
    if (this.invulnerabilityTimer > 0) {
      this.invulnerabilityTimer = Math.max(0, this.invulnerabilityTimer - deltaTime);
    }
  }
}
