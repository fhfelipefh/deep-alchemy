import * as THREE from 'three';
import { PlayerState } from './entities/PlayerState';
import { PotionInventory, PotionType, POTION_CONFIGS } from './alchemy/PotionSystem';
import { WFCGenerator } from './dungeon/WFCGenerator';
import { DungeonMap, CellType } from './dungeon/DungeonTypes';
import { DungeonRenderer } from './render/DungeonRenderer';
import { PlayerRenderer } from './render/PlayerRenderer';
import { CritterRenderer } from './render/CritterRenderer';
import { PotionRenderer } from './render/PotionRenderer';
import { CritterInstance, CritterType } from './entities/CritterAI';
import { DiegeticHUD } from './ui/DiegeticHUD';
import { MobileControls } from './ui/MobileControls';
import { StartMenu } from './ui/StartMenu';
import { CutscenePlayer } from './cutscene/CutscenePlayer';

class GameApp {
  private canvas3D: HTMLCanvasElement;
  private canvasCutscene: HTMLCanvasElement;
  private renderer: THREE.WebGLRenderer;
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;

  private dungeonRenderer: DungeonRenderer;
  private playerRenderer: PlayerRenderer;
  private critterRenderer: CritterRenderer;
  private potionRenderer: PotionRenderer;

  private player: PlayerState;
  private inventory: PotionInventory;
  private hud: DiegeticHUD;
  private mobileControls: MobileControls;
  private cutscenePlayer: CutscenePlayer;

  private currentDungeon: DungeonMap | null = null;
  private critters: CritterInstance[] = [];
  private isGameActive: boolean = false;
  private keysDown: Set<string> = new Set();

  constructor() {
    this.canvas3D = document.getElementById('game-canvas') as HTMLCanvasElement;
    this.canvasCutscene = document.getElementById('cutscene-canvas') as HTMLCanvasElement;

    // 1. Configuração do Three.js WebGL com suporte a sombras realistas
    this.renderer = new THREE.WebGLRenderer({
      canvas: this.canvas3D,
      antialias: true,
      powerPreference: 'high-performance',
    });
    this.renderer.setSize(window.innerWidth, window.innerHeight);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 0.95;

    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x040406); // Escuridão profunda da caverna
    this.scene.fog = new THREE.FogExp2(0x040406, 0.045); // Névoa volumétrica

    this.camera = new THREE.PerspectiveCamera(
      65,
      window.innerWidth / window.innerHeight,
      0.1,
      100
    );

    // 2. Luz ambiente mínima (a caverna depende da lanterna e dos fungos)
    const ambientLight = new THREE.AmbientLight(0x101218, 0.35);
    this.scene.add(ambientLight);

    // 3. Subsistemas de renderização
    this.dungeonRenderer = new DungeonRenderer(this.scene);
    this.playerRenderer = new PlayerRenderer(this.camera);
    this.critterRenderer = new CritterRenderer(this.scene);
    this.potionRenderer = new PotionRenderer(this.scene);
    this.scene.add(this.playerRenderer.group);

    // 4. Lógica de jogo e HUD
    this.player = new PlayerState();
    this.inventory = new PotionInventory();
    this.hud = new DiegeticHUD();
    this.hud.hide(); // Oculta até o fim da cutscene
    this.mobileControls = new MobileControls();
    this.cutscenePlayer = new CutscenePlayer(this.canvasCutscene);

    this.setupInputs();
    this.setupResize();

    // 5. Menu Inicial (única tela com texto)
    new StartMenu(() => this.startNewRun());

    // Inicia o loop de animação
    this.startMainLoop();
  }

  private setupInputs(): void {
    window.addEventListener('keydown', (e) => {
      this.keysDown.add(e.code);

      // Atalhos de seleção de poção (1 a 5)
      if (['Digit1', 'Digit2', 'Digit3', 'Digit4', 'Digit5'].includes(e.code)) {
        const slotIdx = parseInt(e.code.replace('Digit', '')) - 1;
        this.inventory.selectSlot(slotIdx);
      }
    });

    window.addEventListener('keyup', (e) => {
      this.keysDown.delete(e.code);
    });

    // Clique do mouse para arremessar poção
    this.canvas3D.addEventListener('pointerdown', (e) => {
      if (!this.isGameActive || e.button !== 0) return;
      this.throwCurrentPotion();
    });

    // Toque nos slots da HUD seleciona o slot
    this.hud.setOnSlotSelected((index) => {
      this.inventory.selectSlot(index);
    });
  }

  private setupResize(): void {
    window.addEventListener('resize', () => {
      this.camera.aspect = window.innerWidth / window.innerHeight;
      this.camera.updateProjectionMatrix();
      this.renderer.setSize(window.innerWidth, window.innerHeight);
    });
  }

  private startNewRun(): void {
    this.isGameActive = false;
    this.hud.hide();

    // Tarefa assíncrona de geração procedural executada durante a cutscene 2D
    const dungeonTask = new Promise<void>((resolve) => {
      setTimeout(() => {
        const generator = new WFCGenerator({
          width: 90,
          height: 90,
          cellSize: 2.0,
          minExitDistanceMeters: 500.0,
          seed: Math.floor(Math.random() * 1000000),
        });

        this.currentDungeon = generator.generate();
        this.dungeonRenderer.buildDungeon(this.currentDungeon);

        // Posiciona o jogador exatamente na praia inicial
        const startWorldX = (this.currentDungeon.startPos.x + 0.5) * this.currentDungeon.cellSize;
        const startWorldZ = (this.currentDungeon.startPos.z + 0.5) * this.currentDungeon.cellSize;

        this.player.position = { x: startWorldX, y: 0, z: startWorldZ };
        this.player.currentHealth = 20;
        this.player.isDead = false;
        this.player.hasReachedExit = false;

        // Gera os bichinhos patrulhando as cavernas
        this.spawnCaveCritters(this.currentDungeon);

        resolve();
      }, 50);
    });

    // Executa a cutscene 2D narrativa (loading mask)
    this.cutscenePlayer.play(dungeonTask, () => {
      // O jogador assume o controle exatamente no momento em que chega à praia da caverna
      this.isGameActive = true;
      this.hud.show();
    });
  }

  private spawnCaveCritters(dungeon: DungeonMap): void {
    this.critters = [];
    this.critterRenderer.clear();

    const height = dungeon.grid.length;
    const width = dungeon.grid[0].length;
    let critterCount = 0;

    for (let z = 10; z < height - 10; z += 6) {
      for (let x = 10; x < width - 10; x += 6) {
        const cell = dungeon.grid[z][x];
        if (cell === CellType.ROCK) continue;

        let type: CritterType;

        if (cell === CellType.WATER) {
          type = CritterType.SLIME;
        } else if (cell === CellType.GEOTHERMAL) {
          type = CritterType.CAVE_WOLF;
        } else if (cell === CellType.FUNGAL) {
          type = CritterType.GLOW_BEETLE;
        } else {
          type = Math.random() < 0.5 ? CritterType.SLIME : CritterType.CAVE_WOLF;
        }

        const worldX = (x + 0.5) * dungeon.cellSize;
        const worldZ = (z + 0.5) * dungeon.cellSize;

        const critter = new CritterInstance(`critter_${++critterCount}`, type, {
          x: worldX,
          y: 0,
          z: worldZ,
        });

        this.critters.push(critter);
        this.critterRenderer.registerCritter(critter);
      }
    }
  }

  private throwCurrentPotion(): void {
    const activeType = this.inventory.getActivePotion();
    if (!this.inventory.triggerThrow(activeType)) {
      return; // Cooldown ativo
    }

    // Calcula a direção para onde o alquimista está olhando
    const forward = new THREE.Vector3(0, 0, 1);
    forward.applyAxisAngle(new THREE.Vector3(0, 1, 0), this.playerRenderer.group.rotation.y);

    this.potionRenderer.spawnProjectile(activeType, this.player.position, forward);
  }

  private handlePotionImpact(type: PotionType, pos: { x: number; y: number; z: number }): void {
    const config = POTION_CONFIGS[type];

    // Verifica se impactou em célula de água
    let inWater = false;
    if (this.currentDungeon) {
      const gx = Math.floor(pos.x / this.currentDungeon.cellSize);
      const gz = Math.floor(pos.z / this.currentDungeon.cellSize);
      if (
        gz >= 0 && gz < this.currentDungeon.grid.length &&
        gx >= 0 && gx < this.currentDungeon.grid[0].length
      ) {
        inWater = this.currentDungeon.grid[gz][gx] === CellType.WATER;
      }
    }

    // Aplica efeitos nos monstros dentro do raio
    for (const critter of this.critters) {
      if (!critter.isAlive) continue;
      const dist = Math.hypot(critter.position.x - pos.x, critter.position.z - pos.z);
      if (dist <= config.effectRadius) {
        critter.applyPotionEffect(type, inWater);
      }
    }
  }

  private updatePlayer(deltaTime: number): THREE.Vector3 {
    let inputX = 0;
    let inputZ = 0;

    // Teclado
    if (this.keysDown.has('KeyA') || this.keysDown.has('ArrowLeft')) inputX -= 1;
    if (this.keysDown.has('KeyD') || this.keysDown.has('ArrowRight')) inputX += 1;
    if (this.keysDown.has('KeyW') || this.keysDown.has('ArrowUp')) inputZ += 1;
    if (this.keysDown.has('KeyS') || this.keysDown.has('ArrowDown')) inputZ -= 1;

    // Joystick Mobile Android
    if (Math.hypot(this.mobileControls.state.moveX, this.mobileControls.state.moveZ) > 0.1) {
      inputX = this.mobileControls.state.moveX;
      inputZ = -this.mobileControls.state.moveZ;
    }

    // Corrida (Ctrl ou Botão Mobile)
    const isRunning =
      this.keysDown.has('ControlLeft') ||
      this.keysDown.has('ControlRight') ||
      this.mobileControls.state.isRunning;
    this.player.isRunning = isRunning;

    const currentSpeed = isRunning ? this.player.runSpeed : this.player.walkSpeed;

    const moveDirection = new THREE.Vector3(inputX, 0, inputZ);
    if (moveDirection.lengthSq() > 0.01) {
      moveDirection.normalize();

      const nextX = this.player.position.x + moveDirection.x * currentSpeed * deltaTime;
      const nextZ = this.player.position.z + moveDirection.z * currentSpeed * deltaTime;

      // Colisão com paredes da caverna
      if (this.canWalkTo(nextX, nextZ)) {
        this.player.position.x = nextX;
        this.player.position.z = nextZ;
      } else if (this.canWalkTo(nextX, this.player.position.z)) {
        this.player.position.x = nextX; // Desliza no eixo X
      } else if (this.canWalkTo(this.player.position.x, nextZ)) {
        this.player.position.z = nextZ; // Desliza no eixo Z
      }
    }

    // Pulo (Espaço ou Botão Mobile)
    const wantsJump = this.keysDown.has('Space') || this.mobileControls.state.isJumping;
    if (wantsJump && this.player.isGrounded) {
      this.player.velocity.y = this.player.jumpForce;
      this.player.isGrounded = false;
    }

    // Gravidade
    if (!this.player.isGrounded) {
      this.player.velocity.y -= 15.0 * deltaTime;
      this.player.position.y += this.player.velocity.y * deltaTime;
      if (this.player.position.y <= 0) {
        this.player.position.y = 0;
        this.player.velocity.y = 0;
        this.player.isGrounded = true;
      }
    }

    this.player.update(deltaTime);

    // Checa se alcançou a saída
    if (this.currentDungeon && !this.player.hasReachedExit) {
      const exitWorldX = (this.currentDungeon.exitPos.x + 0.5) * this.currentDungeon.cellSize;
      const exitWorldZ = (this.currentDungeon.exitPos.z + 0.5) * this.currentDungeon.cellSize;
      const distToExit = Math.hypot(this.player.position.x - exitWorldX, this.player.position.z - exitWorldZ);

      if (distToExit < 2.5) {
        this.player.hasReachedExit = true;
        // Reinicia a expedição após vitória brilhante
        setTimeout(() => this.startNewRun(), 3000);
      }
    }

    return moveDirection;
  }

  private canWalkTo(worldX: number, worldZ: number): boolean {
    if (!this.currentDungeon) return true;
    const gx = Math.floor(worldX / this.currentDungeon.cellSize);
    const gz = Math.floor(worldZ / this.currentDungeon.cellSize);

    if (
      gz < 0 || gz >= this.currentDungeon.grid.length ||
      gx < 0 || gx >= this.currentDungeon.grid[0].length
    ) {
      return false;
    }

    return this.currentDungeon.grid[gz][gx] !== CellType.ROCK;
  }

  private startMainLoop(): void {
    let lastTime = performance.now();

    const animate = (currentTime: number) => {
      requestAnimationFrame(animate);

      const deltaTime = Math.min(0.05, (currentTime - lastTime) / 1000);
      lastTime = currentTime;

      if (this.isGameActive) {
        // 1. Atualiza Alquimista e Câmera
        const moveDir = this.updatePlayer(deltaTime);
        this.playerRenderer.update(this.player, deltaTime, moveDir);

        // 2. Atualiza Cooldowns e HUD
        this.inventory.updateCooldowns(deltaTime);
        this.hud.update(this.player, this.inventory);

        // 3. Atualiza I.A. dos Bichinhos
        for (const critter of this.critters) {
          critter.update(deltaTime, this.player.position);

          // Ataque ao jogador por proximidade
          if (critter.isAlive && critter.state === 'CHASE') {
            const dist = Math.hypot(
              critter.position.x - this.player.position.x,
              critter.position.z - this.player.position.z
            );
            if (dist < 1.1) {
              this.player.takeDamage(critter.config.attackDamage);
              if (this.player.isDead) {
                setTimeout(() => this.startNewRun(), 2500);
              }
            }
          }
        }
        this.critterRenderer.update(deltaTime);

        // 4. Atualiza Projéteis e Poças Alquímicas
        this.potionRenderer.update(deltaTime, (type, pos) => {
          this.handlePotionImpact(type, pos);
        });

        // 5. Atualiza Caverna (Luzes pulsantes e água)
        this.dungeonRenderer.update(deltaTime, currentTime / 1000);
      }

      this.renderer.render(this.scene, this.camera);
    };

    requestAnimationFrame(animate);
  }
}

// Inicializa a aplicação ao carregar o DOM
window.addEventListener('DOMContentLoaded', () => {
  new GameApp();
});
