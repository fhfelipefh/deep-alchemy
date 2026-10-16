import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';

export class IntroCinematic {
  public isPlaying: boolean = false;
  public onCompleteCallback: (() => void) | null = null;
  public scene: THREE.Scene;
  public camera: THREE.PerspectiveCamera;
  private canvas2D: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D | null = null;
  private timer: number = 0;
  private readonly duration: number = 7.8;

  constructor(scene: THREE.Scene, camera: THREE.PerspectiveCamera) {
    this.scene = scene;
    this.camera = camera;

    this.canvas2D = document.createElement('canvas');
    this.canvas2D.id = 'cinematic-2d';
    this.canvas2D.style.position = 'fixed';
    this.canvas2D.style.top = '0';
    this.canvas2D.style.left = '0';
    this.canvas2D.style.width = '100vw';
    this.canvas2D.style.height = '100vh';
    this.canvas2D.style.zIndex = '90';
    this.canvas2D.style.pointerEvents = 'none';
    this.canvas2D.style.display = 'none';

    document.body.appendChild(this.canvas2D);
    this.ctx = this.canvas2D.getContext('2d');

    this.resize();
    window.addEventListener('resize', () => this.resize());

    try {
      const loader = new GLTFLoader();
      loader.load('models/golden_idol.glb', () => {});
    } catch {}
  }

  private resize(): void {
    if (!this.canvas2D) return;
    this.canvas2D.width = window.innerWidth;
    this.canvas2D.height = window.innerHeight;
  }

  public play(playerPos: THREE.Vector3, onComplete: () => void): void {
    this.isPlaying = true;
    this.timer = 0;
    this.onCompleteCallback = onComplete;
    this.resize();
    this.canvas2D.style.display = 'block';
    this.camera.position.set(playerPos.x, playerPos.y + 1.68, playerPos.z);
  }

  public update(deltaTime: number, _playerPos: THREE.Vector3): void {
    if (!this.isPlaying || !this.ctx) return;

    this.timer += deltaTime;
    const progress = Math.min(1.0, this.timer / this.duration);

    const w = this.canvas2D.width;
    const h = this.canvas2D.height;

    this.ctx.clearRect(0, 0, w, h);

    if (progress < 0.26) {
      this.drawPyramidApproach(w, h, progress / 0.26);
    } else if (progress < 0.44) {
      this.drawIdolTension(w, h, (progress - 0.26) / 0.18);
    } else if (progress < 0.62) {
      this.drawFloorCollapse(w, h, (progress - 0.44) / 0.18);
    } else if (progress < 0.82) {
      this.drawSubterraneanRapids(w, h, (progress - 0.62) / 0.2);
    } else {
      this.drawBeachAwakening(w, h, (progress - 0.82) / 0.18);
    }

    if (this.timer >= this.duration) {
      this.endCutscene();
    }
  }

  private drawPyramidApproach(w: number, h: number, t: number): void {
    const ctx = this.ctx!;
    const grad = ctx.createLinearGradient(0, 0, 0, h);
    grad.addColorStop(0, '#1c130c');
    grad.addColorStop(0.65, '#2e1c12');
    grad.addColorStop(1, '#0e0906');
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, w, h);

    ctx.fillStyle = '#422a18';
    for (let row = 0; row < 5; row++) {
      const y = h * 0.15 + row * 45;
      for (let col = -1; col < 12; col++) {
        const x = col * 120 + ((row % 2) * 60);
        ctx.fillRect(x, y, 114, 40);
      }
    }

    const pedestalX = w * 0.72;
    const pedestalY = h * 0.68;
    ctx.fillStyle = '#5c4328';
    ctx.fillRect(pedestalX - 45, pedestalY, 90, 110);
    ctx.fillRect(pedestalX - 55, pedestalY, 110, 18);

    ctx.save();
    ctx.shadowColor = '#ffe066';
    ctx.shadowBlur = 24 + Math.sin(t * 8) * 8;
    ctx.fillStyle = '#ffcc00';
    ctx.beginPath();
    ctx.arc(pedestalX, pedestalY - 28, 22, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillRect(pedestalX - 16, pedestalY - 14, 32, 28);
    ctx.restore();

    const alchemistX = w * 0.15 + t * (w * 0.38);
    const alchemistY = h * 0.68;
    this.drawAlchemistFigure(ctx, alchemistX, alchemistY, false, t);

    ctx.fillStyle = 'rgba(0,0,0,0.85)';
    ctx.fillRect(0, 0, w, h * 0.08);
    ctx.fillRect(0, h * 0.92, w, h * 0.08);
  }

  private drawIdolTension(w: number, h: number, t: number): void {
    const ctx = this.ctx!;
    ctx.fillStyle = '#110c08';
    ctx.fillRect(0, 0, w, h);

    const zoom = 1.0 + t * 0.12;
    ctx.save();
    ctx.translate(w / 2, h / 2);
    ctx.scale(zoom, zoom);
    ctx.translate(-w / 2, -h / 2);

    const cx = w * 0.52;
    const cy = h * 0.52;

    ctx.fillStyle = '#4a321f';
    ctx.fillRect(cx - 70, cy + 50, 140, 150);

    const lift = Math.min(32, t * 45);
    ctx.save();
    ctx.shadowColor = '#ffd700';
    ctx.shadowBlur = 36;
    ctx.fillStyle = '#ffe044';
    ctx.beginPath();
    ctx.arc(cx, cy - lift, 38, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillRect(cx - 28, cy - lift + 18, 56, 42);
    ctx.restore();

    ctx.fillStyle = '#e8a87c';
    ctx.fillRect(cx - 75, cy - lift + 4, 52, 20);

    ctx.restore();

    ctx.fillStyle = 'rgba(0,0,0,0.88)';
    ctx.fillRect(0, 0, w, h * 0.08);
    ctx.fillRect(0, h * 0.92, w, h * 0.08);
  }

  private drawFloorCollapse(w: number, h: number, t: number): void {
    const ctx = this.ctx!;
    const shakeX = (Math.random() - 0.5) * 14 * (1 - t * 0.5);
    const shakeY = (Math.random() - 0.5) * 14 * (1 - t * 0.5);

    ctx.save();
    ctx.translate(shakeX, shakeY);

    ctx.fillStyle = '#080503';
    ctx.fillRect(-20, -20, w + 40, h + 40);

    const holeSize = Math.max(10, (1 - t) * (w * 0.35));
    const cx = w * 0.5;
    const cy = h * 0.45;

    ctx.save();
    ctx.shadowColor = '#ffbb55';
    ctx.shadowBlur = 40 * (1 - t);
    ctx.fillStyle = `rgba(255, 180, 70, ${Math.max(0, 1 - t * 1.1)})`;
    ctx.fillRect(cx - holeSize / 2, cy - holeSize / 2, holeSize, holeSize);
    ctx.restore();

    ctx.fillStyle = '#3a2414';
    const slabDist = t * (w * 0.28);
    ctx.fillRect(cx - holeSize / 2 - 120 + slabDist, cy - holeSize / 2 - 20, 120, holeSize + 40);
    ctx.fillRect(cx + holeSize / 2 - slabDist, cy - holeSize / 2 - 20, 120, holeSize + 40);

    ctx.restore();

    ctx.fillStyle = 'rgba(0,0,0,0.92)';
    ctx.fillRect(0, 0, w, h * 0.08);
    ctx.fillRect(0, h * 0.92, w, h * 0.08);
  }

  private drawSubterraneanRapids(w: number, h: number, t: number): void {
    const ctx = this.ctx!;
    ctx.fillStyle = '#0a121c';
    ctx.fillRect(0, 0, w, h);

    ctx.fillStyle = '#060b12';
    ctx.beginPath();
    ctx.moveTo(0, 0);
    ctx.lineTo(w * 0.25, 0);
    ctx.lineTo(w * 0.15, h);
    ctx.lineTo(0, h);
    ctx.fill();

    ctx.beginPath();
    ctx.moveTo(w * 0.78, 0);
    ctx.lineTo(w, 0);
    ctx.lineTo(w, h);
    ctx.lineTo(w * 0.85, h);
    ctx.fill();

    const waterGrad = ctx.createLinearGradient(w * 0.2, 0, w * 0.8, h);
    waterGrad.addColorStop(0, '#154160');
    waterGrad.addColorStop(0.5, '#226b96');
    waterGrad.addColorStop(1, '#0e2b40');
    ctx.fillStyle = waterGrad;
    ctx.fillRect(w * 0.2, 0, w * 0.6, h);

    ctx.strokeStyle = '#62badb';
    ctx.lineWidth = 3;
    for (let i = 0; i < 18; i++) {
      const yPos = ((t * 1200 + i * 55) % (h + 80)) - 40;
      const xPos = w * 0.28 + Math.sin(yPos * 0.02 + i) * (w * 0.18);
      ctx.beginPath();
      ctx.moveTo(xPos, yPos);
      ctx.lineTo(xPos + (Math.sin(i) * 20), yPos + 40);
      ctx.stroke();
    }

    const alchemistY = ((t * 1.2) * h) % h;
    const alchemistX = w * 0.48 + Math.sin(alchemistY * 0.03) * 35;
    ctx.save();
    ctx.translate(alchemistX, alchemistY);
    ctx.rotate(Math.sin(t * 14) * 0.35);

    ctx.fillStyle = '#6b4f2c';
    ctx.fillRect(-22, -18, 44, 36);
    ctx.fillStyle = '#3a2312';
    ctx.beginPath();
    ctx.ellipse(0, -24, 26, 9, 0, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillRect(-14, -36, 28, 14);

    ctx.fillStyle = '#e8a87c';
    ctx.beginPath();
    ctx.arc(0, -14, 12, 0, Math.PI * 2);
    ctx.fill();

    ctx.fillStyle = '#2d3d22';
    ctx.fillRect(-16, 12, 32, 22);

    ctx.restore();

    ctx.fillStyle = 'rgba(0,0,0,0.92)';
    ctx.fillRect(0, 0, w, h * 0.08);
    ctx.fillRect(0, h * 0.92, w, h * 0.08);
  }

  private drawBeachAwakening(w: number, h: number, t: number): void {
    const ctx = this.ctx!;
    ctx.fillStyle = '#080c14';
    ctx.fillRect(0, 0, w, h);

    const sandGrad = ctx.createLinearGradient(0, h * 0.5, 0, h);
    sandGrad.addColorStop(0, '#594a3a');
    sandGrad.addColorStop(0.4, '#c2a886');
    sandGrad.addColorStop(1, '#826d54');
    ctx.fillStyle = sandGrad;
    ctx.beginPath();
    ctx.moveTo(0, h * 0.65);
    ctx.quadraticCurveTo(w * 0.45, h * 0.55, w, h * 0.72);
    ctx.lineTo(w, h);
    ctx.lineTo(0, h);
    ctx.fill();

    const alchemistX = w * 0.42;
    const alchemistY = h * 0.72;

    this.drawAlchemistFigure(ctx, alchemistX, alchemistY, true, t);

    if (t > 0.35) {
      const lampIntensity = Math.min(1.0, (t - 0.35) / 0.3);
      ctx.save();
      const beamGrad = ctx.createRadialGradient(
        alchemistX + 12,
        alchemistY - 48,
        4,
        alchemistX + 180,
        alchemistY - 20,
        340
      );
      beamGrad.addColorStop(0, `rgba(255, 248, 220, ${0.95 * lampIntensity})`);
      beamGrad.addColorStop(0.3, `rgba(255, 230, 160, ${0.55 * lampIntensity})`);
      beamGrad.addColorStop(1, 'rgba(255, 230, 160, 0)');

      ctx.fillStyle = beamGrad;
      ctx.beginPath();
      ctx.moveTo(alchemistX + 12, alchemistY - 48);
      ctx.lineTo(alchemistX + 420, alchemistY - 140);
      ctx.lineTo(alchemistX + 460, alchemistY + 120);
      ctx.closePath();
      ctx.fill();
      ctx.restore();
    }

    const shroomGlow = Math.min(1.0, t * 1.5);
    this.drawGlowMushroom2D(ctx, w * 0.18, h * 0.68, '#00ffff', shroomGlow);
    this.drawGlowMushroom2D(ctx, w * 0.24, h * 0.71, '#3388ff', shroomGlow);
    this.drawGlowMushroom2D(ctx, w * 0.76, h * 0.78, '#00e5ff', shroomGlow);
    this.drawGlowMushroom2D(ctx, w * 0.84, h * 0.82, '#00ffff', shroomGlow);

    if (t > 0.8) {
      const fadeAlpha = (t - 0.8) / 0.2;
      ctx.fillStyle = `rgba(18, 23, 34, ${fadeAlpha})`;
      ctx.fillRect(0, 0, w, h);
    }

    ctx.fillStyle = 'rgba(0,0,0,0.92)';
    ctx.fillRect(0, 0, w, h * 0.08);
    ctx.fillRect(0, h * 0.92, w, h * 0.08);
  }

  private drawAlchemistFigure(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    isKneeling: boolean,
    t: number
  ): void {
    ctx.save();
    ctx.translate(x, y);

    if (!isKneeling) {
      ctx.fillStyle = '#6b4f2c';
      ctx.fillRect(-14, -58, 28, 38);

      ctx.fillStyle = '#2d3d22';
      ctx.fillRect(-28, -55, 14, 30);

      ctx.fillStyle = '#3a2312';
      ctx.fillRect(8, -32, 12, 10);

      ctx.fillStyle = '#5c4328';
      const legWalk = Math.sin(t * 18) * 12;
      ctx.fillRect(-12, -20, 10, 20 + legWalk);
      ctx.fillRect(2, -20, 10, 20 - legWalk);

      ctx.fillStyle = '#e8a87c';
      ctx.beginPath();
      ctx.arc(0, -68, 12, 0, Math.PI * 2);
      ctx.fill();

      ctx.fillStyle = '#3a2312';
      ctx.beginPath();
      ctx.ellipse(0, -74, 24, 7, 0, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillRect(-12, -86, 24, 13);

      ctx.fillStyle = '#ffd700';
      ctx.fillRect(6, -73, 5, 5);
    } else {
      ctx.fillStyle = '#6b4f2c';
      ctx.fillRect(-18, -42, 36, 28);

      ctx.fillStyle = '#2d3d22';
      ctx.fillRect(-28, -40, 12, 24);

      ctx.fillStyle = '#5c4328';
      ctx.fillRect(-14, -14, 26, 14);

      ctx.fillStyle = '#e8a87c';
      ctx.beginPath();
      ctx.arc(8, -50, 12, 0, Math.PI * 2);
      ctx.fill();

      ctx.fillStyle = '#3a2312';
      ctx.beginPath();
      ctx.ellipse(8, -56, 22, 6, 0.2, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillRect(-2, -66, 20, 11);

      ctx.fillStyle = '#ffd700';
      ctx.fillRect(16, -56, 6, 6);
    }

    ctx.restore();
  }

  private drawGlowMushroom2D(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    color: string,
    alpha: number
  ): void {
    ctx.save();
    ctx.translate(x, y);

    ctx.fillStyle = `rgba(200, 210, 220, ${alpha * 0.9})`;
    ctx.fillRect(-2, -18, 4, 18);

    ctx.shadowColor = color;
    ctx.shadowBlur = 18 * alpha;
    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.arc(0, -20, 10, Math.PI, 0, false);
    ctx.fill();

    ctx.restore();
  }

  public skip(): void {
    this.endCutscene();
  }

  private endCutscene(): void {
    if (!this.isPlaying) return;
    this.isPlaying = false;
    this.canvas2D.style.display = 'none';
    if (this.ctx) {
      this.ctx.clearRect(0, 0, this.canvas2D.width, this.canvas2D.height);
    }
    if (this.onCompleteCallback) {
      this.onCompleteCallback();
    }
  }
}
