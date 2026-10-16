import * as THREE from 'three';
import { PlayerState } from './entities/PlayerState';
import { PotionInventory, PotionType, POTION_CONFIGS } from './alchemy/PotionSystem';
import { WFCGenerator } from './dungeon/WFCGenerator';
import { DungeonMap, CellType } from './dungeon/DungeonTypes';
import { DungeonRenderer } from './render/DungeonRenderer';
import { PlayerRenderer } from './render/PlayerRenderer';
import { CritterRenderer } from './render/CritterRenderer';
import { PotionRenderer } from './render/PotionRenderer';
import { IntroCinematic } from './render/IntroCinematic';
import { CritterInstance, CritterType } from './entities/CritterAI';
import { DiegeticHUD } from './ui/DiegeticHUD';
import { MobileControls } from './ui/MobileControls';
import { StartMenu } from './ui/StartMenu';

class GameApp {
  private canvas3D: HTMLCanvasElement;
  private renderer: THREE.WebGLRenderer;
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;

  private dungeonRenderer: DungeonRenderer;
  private playerRenderer: PlayerRenderer;
  private critterRenderer: CritterRenderer;
  private potionRenderer: PotionRenderer;
  private introCinematic: IntroCinematic;

  private player: PlayerState;
  private inventory: PotionInventory;
  private hud: DiegeticHUD;
  private mobileControls: MobileControls;

  private currentDungeon: DungeonMap | null = null;
  private critters: CritterInstance[] = [];
  private isGameActive: boolean = false;
  private keysDown: Set<string> = new Set();

  constructor() {
    this.canvas3D = document.getElementById('game-canvas') as HTMLCanvasElement;

    this.renderer = new THREE.WebGLRenderer({
      canvas: this.canvas3D,
      antialias: true,
      powerPreference: 'high-performance',
      preserveDrawingBuffer: true,
    });
    this.renderer.setSize(window.innerWidth, window.innerHeight);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFShadowMap;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.1;

    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x121722);
    this.scene.fog = new THREE.FogExp2(0x121722, 0.005);

    this.camera = new THREE.PerspectiveCamera(
      60,
      window.innerWidth / window.innerHeight,
      0.1,
      120
    );

    const ambientLight = new THREE.AmbientLight(0x75889e, 1.8);
    this.scene.add(ambientLight);

    const hemiLight = new THREE.HemisphereLight(0x90a8c0, 0x4d3d2e, 1.6);
    this.scene.add(hemiLight);

    const dirLight = new THREE.DirectionalLight(0xffeedd, 1.8);
    dirLight.position.set(20, 45, 10);
    dirLight.castShadow = true;
    dirLight.shadow.mapSize.width = 1024;
    dirLight.shadow.mapSize.height = 1024;
    this.scene.add(dirLight);

    this.dungeonRenderer = new DungeonRenderer(this.scene);
    this.playerRenderer = new PlayerRenderer(this.camera, this.scene);
    this.critterRenderer = new CritterRenderer(this.scene);
    this.potionRenderer = new PotionRenderer(this.scene);
    this.introCinematic = new IntroCinematic(this.scene, this.camera);
    this.scene.add(this.playerRenderer.group);

    this.player = new PlayerState();
    this.inventory = new PotionInventory();
    this.hud = new DiegeticHUD();
    this.hud.hide();
    this.mobileControls = new MobileControls();

    this.setupInputs();
    this.setupResize();

    (window as any).__APP__ = this;

    new StartMenu(() => this.startNewRun());
    this.startMainLoop();
  }

  private setupInputs(): void {
    window.addEventListener('keydown', (e) => {
      this.keysDown.add(e.code);

      if (['Digit1', 'Digit2', 'Digit3', 'Digit4', 'Digit5'].includes(e.code)) {
        const slotIdx = parseInt(e.code.replace('Digit', '')) - 1;
        this.inventory.selectSlot(slotIdx);
      }
    });

    window.addEventListener('keyup', (e) => {
      this.keysDown.delete(e.code);
    });

    this.canvas3D.addEventListener('pointerdown', (e) => {
      if (!this.isGameActive || e.button !== 0) return;
      this.throwCurrentPotion();
    });

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

    const generator = new WFCGenerator({
      width: 90,
      height: 90,
      cellSize: 2.0,
      minExitDistanceMeters: 500.0,
      seed: Math.floor(Math.random() * 1000000),
    });

    this.currentDungeon = generator.generate();
    this.dungeonRenderer.buildDungeon(this.currentDungeon);

    const startWorldX = (this.currentDungeon.startPos.x + 0.5) * this.currentDungeon.cellSize;
    const startWorldZ = (this.currentDungeon.startPos.z + 0.5) * this.currentDungeon.cellSize;

    this.player.position = { x: startWorldX, y: 0, z: startWorldZ };
    this.player.currentHealth = 20;
    this.player.isDead = false;
    this.player.hasReachedExit = false;

    this.spawnCaveCritters(this.currentDungeon);

    const initialAngle = Math.PI;
    this.playerRenderer.snapCamera(this.player, initialAngle);

    const playerPosVec = new THREE.Vector3(startWorldX, 0, startWorldZ);
    this.introCinematic.play(playerPosVec, () => {
      this.isGameActive = true;
      this.hud.show();
      this.playerRenderer.snapCamera(this.player, initialAngle);
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
      return;
    }

    const forward = new THREE.Vector3(0, 0, 1);
    forward.applyAxisAngle(new THREE.Vector3(0, 1, 0), this.playerRenderer.group.rotation.y);

    this.potionRenderer.spawnProjectile(activeType, this.player.position, forward);
  }

  private handlePotionImpact(type: PotionType, pos: { x: number; y: number; z: number }): void {
    const config = POTION_CONFIGS[type];

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

    if (this.keysDown.has('KeyA') || this.keysDown.has('ArrowLeft')) inputX -= 1;
    if (this.keysDown.has('KeyD') || this.keysDown.has('ArrowRight')) inputX += 1;
    if (this.keysDown.has('KeyW') || this.keysDown.has('ArrowUp')) inputZ -= 1;
    if (this.keysDown.has('KeyS') || this.keysDown.has('ArrowDown')) inputZ += 1;

    if (Math.hypot(this.mobileControls.state.moveX, this.mobileControls.state.moveZ) > 0.1) {
      inputX = this.mobileControls.state.moveX;
      inputZ = this.mobileControls.state.moveZ;
    }

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

      if (this.canWalkTo(nextX, nextZ)) {
        this.player.position.x = nextX;
        this.player.position.z = nextZ;
      } else if (this.canWalkTo(nextX, this.player.position.z)) {
        this.player.position.x = nextX;
      } else if (this.canWalkTo(this.player.position.x, nextZ)) {
        this.player.position.z = nextZ;
      }
    }

    const wantsJump = this.keysDown.has('Space') || this.mobileControls.state.isJumping;
    if (wantsJump && this.player.isGrounded) {
      this.player.velocity.y = this.player.jumpForce;
      this.player.isGrounded = false;
    }

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

    if (this.currentDungeon && !this.player.hasReachedExit) {
      const exitWorldX = (this.currentDungeon.exitPos.x + 0.5) * this.currentDungeon.cellSize;
      const exitWorldZ = (this.currentDungeon.exitPos.z + 0.5) * this.currentDungeon.cellSize;
      const distToExit = Math.hypot(this.player.position.x - exitWorldX, this.player.position.z - exitWorldZ);

      if (distToExit < 2.5) {
        this.player.hasReachedExit = true;
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

      if (this.introCinematic.isPlaying) {
        const playerPosVec = new THREE.Vector3(
          this.player.position.x,
          this.player.position.y,
          this.player.position.z
        );
        this.introCinematic.update(deltaTime, playerPosVec);
      } else if (this.isGameActive) {
        const moveDir = this.updatePlayer(deltaTime);
        this.playerRenderer.update(this.player, deltaTime, moveDir);

        this.inventory.updateCooldowns(deltaTime);
        this.hud.update(this.player, this.inventory);

        for (const critter of this.critters) {
          critter.update(deltaTime, this.player.position);

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

        this.potionRenderer.update(deltaTime, (type, pos) => {
          this.handlePotionImpact(type, pos);
        });

        this.dungeonRenderer.update(deltaTime, currentTime / 1000);
      }

      this.renderer.render(this.scene, this.camera);
    };

    requestAnimationFrame(animate);
  }
}

window.addEventListener('DOMContentLoaded', () => {
  new GameApp();
});
