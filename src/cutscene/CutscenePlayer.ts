/**
 * Reprodutor da Cutscene 2D Narrativa (Motion Comic / Estilo Anime).
 * Atua como loading mask ativa e não-pulável enquanto o gerador WFC e o Dual Grid
 * constroem o sistema de cavernas subterrâneas em background.
 * Totalmente diegética e sem elementos de texto.
 */

export class CutscenePlayer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private isPlaying: boolean = false;
  private progress: number = 0; // 0.0 a 1.0 ao longo das fases
  private durationSeconds: number = 11.0;
  private onCompleteCallback: (() => void) | null = null;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d')!;
    this.resize();
    window.addEventListener('resize', () => this.resize());
  }

  private resize(): void {
    this.canvas.width = window.innerWidth;
    this.canvas.height = window.innerHeight;
  }

  public play(loadingTask: Promise<void>, onComplete: () => void): void {
    this.isPlaying = true;
    this.progress = 0;
    this.onCompleteCallback = onComplete;
    this.canvas.style.display = 'block';

    let lastTime = performance.now();
    let generationDone = false;

    loadingTask.then(() => {
      generationDone = true;
    });

    const loop = (currentTime: number) => {
      if (!this.isPlaying) return;

      const delta = (currentTime - lastTime) / 1000;
      lastTime = currentTime;

      // Avança a linha do tempo narrativa
      this.progress += delta / this.durationSeconds;

      // Se a cutscene terminou mas o mapa ainda está carregando, segura no último painel da praia
      if (this.progress >= 1.0) {
        if (generationDone) {
          this.endCutscene();
          return;
        } else {
          this.progress = 0.99; // Segura na praia até a geração concluir
        }
      }

      this.render();
      requestAnimationFrame(loop);
    };

    requestAnimationFrame(loop);
  }

  private endCutscene(): void {
    this.isPlaying = false;
    this.canvas.style.display = 'none';
    if (this.onCompleteCallback) {
      this.onCompleteCallback();
    }
  }

  private render(): void {
    const w = this.canvas.width;
    const h = this.canvas.height;
    const ctx = this.ctx;

    ctx.save();
    ctx.fillStyle = '#080604';
    ctx.fillRect(0, 0, w, h);

    // Fases da narrativa:
    // 0.00 - 0.22 : Cena 1 - Entrando na pirâmide com tocha/lanterna e avistando o altar
    // 0.22 - 0.38 : Cena 2 - Pegando a estátua de ouro no pedestal
    // 0.38 - 0.48 : Cena 3 - Tensão de 1 segundo ("nada ocorreu...")
    // 0.48 - 0.62 : Cena 4 - Chão se abrindo, queda e visão para cima do teto se fechando
    // 0.62 - 0.82 : Cena 5 - Arrastado pela correnteza violenta do rio subterrâneo
    // 0.82 - 1.00 : Cena 6 - Acordando na praia escura, lanterna acendendo e revelando os fungos

    if (this.progress < 0.22) {
      this.renderScene1PyramidEntry(ctx, w, h, this.progress / 0.22);
    } else if (this.progress < 0.38) {
      this.renderScene2TakeIdol(ctx, w, h, (this.progress - 0.22) / 0.16);
    } else if (this.progress < 0.48) {
      this.renderScene3SilencePause(ctx, w, h, (this.progress - 0.38) / 0.10);
    } else if (this.progress < 0.62) {
      this.renderScene4TrapdoorFall(ctx, w, h, (this.progress - 0.48) / 0.14);
    } else if (this.progress < 0.82) {
      this.renderScene5WaterRapids(ctx, w, h, (this.progress - 0.62) / 0.20);
    } else {
      this.renderScene6CaveBeach(ctx, w, h, (this.progress - 0.82) / 0.18);
    }

    // Moldura estilo Anime / Mangá Cinema
    this.renderCinematicBars(ctx, w, h);

    ctx.restore();
  }

  // Cena 1: Entrada na Pirâmide
  private renderScene1PyramidEntry(ctx: CanvasRenderingContext2D, w: number, h: number, t: number): void {
    // Fundo de pedra arenito egípcio
    const grad = ctx.createLinearGradient(0, 0, w, h);
    grad.addColorStop(0, '#2d1808');
    grad.addColorStop(1, '#0e0703');
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, w, h);

    // Colunas de pedra da pirâmide em perspectiva
    ctx.fillStyle = '#1c0f05';
    ctx.beginPath();
    ctx.moveTo(w * 0.1, h);
    ctx.lineTo(w * 0.25, 0);
    ctx.lineTo(w * 0.35, 0);
    ctx.lineTo(w * 0.25, h);
    ctx.fill();

    ctx.beginPath();
    ctx.moveTo(w * 0.9, h);
    ctx.lineTo(w * 0.75, 0);
    ctx.lineTo(w * 0.65, 0);
    ctx.lineTo(w * 0.75, h);
    ctx.fill();

    // Pedestal ao fundo brilhando
    const altarX = w * 0.5;
    const altarY = h * 0.52;
    ctx.fillStyle = '#442810';
    ctx.fillRect(altarX - 40, altarY, 80, 100);

    // Brilho dourado da estátua ao fundo
    const glow = ctx.createRadialGradient(altarX, altarY - 20, 5, altarX, altarY - 20, 80 + Math.sin(t * 10) * 10);
    glow.addColorStop(0, 'rgba(255, 215, 0, 0.9)');
    glow.addColorStop(1, 'rgba(255, 180, 0, 0)');
    ctx.fillStyle = glow;
    ctx.beginPath();
    ctx.arc(altarX, altarY - 20, 80, 0, Math.PI * 2);
    ctx.fill();

    // Silhueta do Alquimista caminhando em direção ao altar (chapéu, mochila grande, pochete)
    const charX = w * (0.35 + t * 0.1);
    const charY = h * 0.72;
    this.drawAlchemistSilhouette(ctx, charX, charY, 0.9);
  }

  // Cena 2: Pegando a estátua dourada
  private renderScene2TakeIdol(ctx: CanvasRenderingContext2D, w: number, h: number, t: number): void {
    const cx = w * 0.5;
    const cy = h * 0.5;

    // Foco em primeiro plano no altar e na estátua
    ctx.fillStyle = '#180d05';
    ctx.fillRect(0, 0, w, h);

    // Pedestal
    ctx.fillStyle = '#3a200a';
    ctx.fillRect(cx - 90, cy + 50, 180, 150);

    // Estátua de ouro resplandecente
    ctx.save();
    ctx.translate(cx, cy);
    ctx.fillStyle = '#ffd700';
    ctx.beginPath();
    ctx.ellipse(0, 0, 35, 55, 0, 0, Math.PI * 2); // Corpo
    ctx.fill();
    ctx.beginPath();
    ctx.arc(0, -50, 22, 0, Math.PI * 2); // Cabeça
    ctx.fill();

    // Raio de luz dourada
    const beam = ctx.createRadialGradient(0, -20, 10, 0, -20, 160);
    beam.addColorStop(0, 'rgba(255, 230, 100, 0.6)');
    beam.addColorStop(1, 'rgba(255, 200, 0, 0)');
    ctx.fillStyle = beam;
    ctx.arc(0, -20, 160, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();

    // Mão enluvada de couro do alquimista se aproximando e tocando a estátua
    const handX = cx - 180 + t * 150;
    const handY = cy - 10;
    ctx.fillStyle = '#4a2c14';
    ctx.beginPath();
    ctx.ellipse(handX, handY, 30, 20, Math.PI / 6, 0, Math.PI * 2);
    ctx.fill();
    // Manga da camisa cáqui
    ctx.fillStyle = '#8b6b43';
    ctx.fillRect(handX - 120, handY - 18, 110, 36);
  }

  // Cena 3: Pausa tensa de 1 segundo ("nada ocorreu...")
  private renderScene3SilencePause(ctx: CanvasRenderingContext2D, w: number, h: number, _t: number): void {
    const cx = w * 0.5;
    const cy = h * 0.5;
    ctx.fillStyle = '#100802';
    ctx.fillRect(0, 0, w, h);

    // Close-up nos olhos do alquimista sob a aba do chapéu estilo Indiana Jones
    ctx.fillStyle = '#e8ba95';
    ctx.fillRect(cx - 200, cy - 80, 400, 160);

    // Sombra da aba do chapéu cobrindo a testa
    ctx.fillStyle = '#26160c';
    ctx.beginPath();
    ctx.moveTo(cx - 220, cy - 100);
    ctx.lineTo(cx + 220, cy - 80);
    ctx.lineTo(cx + 220, cy - 20);
    ctx.lineTo(cx - 220, cy - 10);
    ctx.closePath();
    ctx.fill();

    // Olhos tensos observando
    ctx.fillStyle = '#ffffff';
    ctx.fillRect(cx - 110, cy, 50, 25);
    ctx.fillRect(cx + 60, cy, 50, 25);

    // Íris anime
    ctx.fillStyle = '#2c5a75';
    ctx.beginPath();
    ctx.arc(cx - 85, cy + 12, 12, 0, Math.PI * 2);
    ctx.arc(cx + 85, cy + 12, 12, 0, Math.PI * 2);
    ctx.fill();

    // Gota de suor estilizada anime escorrendo
    ctx.fillStyle = '#7ac5f0';
    ctx.beginPath();
    ctx.arc(cx + 140, cy + 15, 6, 0, Math.PI * 2);
    ctx.fill();
  }

  // Cena 4: Chão se abre, queda e visão para cima do teto se fechando
  private renderScene4TrapdoorFall(ctx: CanvasRenderingContext2D, w: number, h: number, t: number): void {
    const cx = w * 0.5;
    const cy = h * 0.5;

    // Fundo negro do abismo
    ctx.fillStyle = '#020202';
    ctx.fillRect(0, 0, w, h);

    // Câmera olhando para cima em direção à abertura do teto que está se fechando
    // Abertura quadrada com luz da pirâmide ficando menor
    const openSize = Math.max(0, (1 - t) * 220);
    if (openSize > 2) {
      // Luz solar amarela/dourada entrando pelo quadrado
      ctx.fillStyle = '#ffe082';
      ctx.fillRect(cx - openSize * 0.5, cy - openSize * 0.5, openSize, openSize);

      // Portas de pedra fechando horizontalmente
      ctx.fillStyle = '#3a2312';
      // Porta esquerda
      ctx.fillRect(cx - 150, cy - openSize * 0.5, 150 - openSize * 0.5, openSize);
      // Porta direita
      ctx.fillRect(cx + openSize * 0.5, cy - openSize * 0.5, 150, openSize);
    }

    // Linhas de velocidade de queda livre vertical (Anime speedlines)
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.4)';
    ctx.lineWidth = 2;
    for (let i = 0; i < 15; i++) {
      const rx = (Math.sin(i * 32.5 + t * 20) * 0.5 + 0.5) * w;
      const ry = (Math.cos(i * 12.3 + t * 40) * 0.5 + 0.5) * h;
      ctx.beginPath();
      ctx.moveTo(rx, ry);
      ctx.lineTo(rx, ry + 80);
      ctx.stroke();
    }
  }

  // Cena 5: Alquimista arrastado por correnteza subterrânea violenta
  private renderScene5WaterRapids(ctx: CanvasRenderingContext2D, w: number, h: number, t: number): void {
    // Água azul escura turbulenta
    const grad = ctx.createLinearGradient(0, 0, w, h);
    grad.addColorStop(0, '#041829');
    grad.addColorStop(1, '#020912');
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, w, h);

    // Ondas e correntezas de espuma desenhadas com curvas de Bézier
    ctx.strokeStyle = 'rgba(100, 200, 255, 0.6)';
    ctx.lineWidth = 4;
    for (let i = 0; i < 8; i++) {
      const y = h * (0.2 + i * 0.1) + Math.sin(t * 15 + i) * 15;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.bezierCurveTo(w * 0.3, y - 40, w * 0.6, y + 40, w, y);
      ctx.stroke();
    }

    // Silhueta do Alquimista girando e sendo levado pela água (mochila e chapéu visíveis)
    ctx.save();
    ctx.translate(w * (0.2 + t * 0.6), h * (0.5 + Math.sin(t * 20) * 0.1));
    ctx.rotate(Math.sin(t * 12) * 0.4);
    this.drawAlchemistSilhouette(ctx, 0, 0, 0.8);
    // Bolhas de ar ao redor
    ctx.fillStyle = 'rgba(200, 240, 255, 0.8)';
    for (let b = 0; b < 6; b++) {
      ctx.beginPath();
      ctx.arc(Math.sin(b * 5) * 40, Math.cos(b * 3) * 40, 5 + b, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.restore();
  }

  // Cena 6: Despertando na praia da caverna, lanterna acende e revela cogumelos brilhantes
  private renderScene6CaveBeach(ctx: CanvasRenderingContext2D, w: number, h: number, t: number): void {
    // Escuridão profunda da caverna
    ctx.fillStyle = '#040406';
    ctx.fillRect(0, 0, w, h);

    // Margem de areia / praia subterrânea
    ctx.fillStyle = '#141416';
    ctx.beginPath();
    ctx.moveTo(0, h * 0.7);
    ctx.quadraticCurveTo(w * 0.5, h * 0.65, w, h * 0.8);
    ctx.lineTo(w, h);
    ctx.lineTo(0, h);
    ctx.closePath();
    ctx.fill();

    // Alquimista deitado na margem, começando a se levantar
    const charX = w * 0.45;
    const charY = h * 0.72;
    this.drawAlchemistSilhouette(ctx, charX, charY, 0.85);

    // Feixe cônico de luz da lanterna de cabeça acendendo (spotlight flickers on)
    const lanternFlicker = t > 0.3 ? (Math.sin(t * 30) > -0.2 ? 1.0 : 0.4) : t * 2;
    if (lanternFlicker > 0) {
      const cone = ctx.createRadialGradient(charX + 20, charY - 35, 10, charX + 220, charY - 20, 260);
      cone.addColorStop(0, `rgba(255, 245, 200, ${0.85 * lanternFlicker})`);
      cone.addColorStop(1, 'rgba(255, 230, 150, 0)');
      ctx.fillStyle = cone;
      ctx.beginPath();
      ctx.moveTo(charX + 20, charY - 35);
      ctx.lineTo(charX + 280, charY - 120);
      ctx.lineTo(charX + 320, charY + 60);
      ctx.closePath();
      ctx.fill();
    }

    // Cogumelos e musgos bioluminescentes brilhando nas paredes e chão ao redor
    this.drawBioluminescentMushrooms(ctx, w * 0.2, h * 0.75, 0x00ffff, t);
    this.drawBioluminescentMushrooms(ctx, w * 0.75, h * 0.68, 0x3388ff, t);
    this.drawBioluminescentMushrooms(ctx, w * 0.85, h * 0.76, 0x00ffcc, t);
  }

  private drawAlchemistSilhouette(ctx: CanvasRenderingContext2D, x: number, y: number, scale: number): void {
    ctx.save();
    ctx.translate(x, y);
    ctx.scale(scale, scale);

    // Corpo
    ctx.fillStyle = '#22160d';
    ctx.fillRect(-15, -45, 30, 50);

    // Mochila grande nas costas
    ctx.fillStyle = '#1c2214';
    ctx.fillRect(-35, -42, 22, 38);

    // Pochete utilitária na frente
    ctx.fillStyle = '#2e1c10';
    ctx.fillRect(10, -15, 14, 12);

    // Cabeça
    ctx.fillStyle = '#e2aa82';
    ctx.beginPath();
    ctx.arc(0, -58, 14, 0, Math.PI * 2);
    ctx.fill();

    // Chapéu de aba larga Indiana Jones
    ctx.fillStyle = '#3a2211';
    ctx.beginPath();
    ctx.ellipse(0, -68, 28, 6, 0, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillRect(-14, -80, 28, 14);

    // Lanterna de cabeça metálica na testa
    ctx.fillStyle = '#d4af37';
    ctx.fillRect(8, -66, 8, 6);

    ctx.restore();
  }

  private drawBioluminescentMushrooms(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    _colorHex: number,
    t: number
  ): void {
    ctx.save();
    ctx.translate(x, y);

    const pulse = 0.8 + Math.sin(t * 8 + x) * 0.2;

    // Halo de luz
    const glow = ctx.createRadialGradient(0, -15, 2, 0, -15, 35 * pulse);
    glow.addColorStop(0, 'rgba(0, 255, 240, 0.7)');
    glow.addColorStop(1, 'rgba(0, 255, 240, 0)');
    ctx.fillStyle = glow;
    ctx.beginPath();
    ctx.arc(0, -15, 35 * pulse, 0, Math.PI * 2);
    ctx.fill();

    // Caule
    ctx.strokeStyle = '#d0e0e8';
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.moveTo(0, 0);
    ctx.quadraticCurveTo(-4, -10, 0, -20);
    ctx.stroke();

    // Chapéu brilhante
    ctx.fillStyle = '#00f5ff';
    ctx.beginPath();
    ctx.ellipse(0, -20, 10, 6, 0, Math.PI, 0);
    ctx.fill();

    ctx.restore();
  }

  private renderCinematicBars(ctx: CanvasRenderingContext2D, w: number, h: number): void {
    const barHeight = Math.floor(h * 0.08);
    ctx.fillStyle = '#000000';
    ctx.fillRect(0, 0, w, barHeight);
    ctx.fillRect(0, h - barHeight, w, barHeight);
  }
}
