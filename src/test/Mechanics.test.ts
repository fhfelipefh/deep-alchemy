import { describe, it, expect } from 'vitest';
import { PRNG } from '../core/PRNG';
import { DualGridResolver, MASK_TO_PIECE } from '../dungeon/DualGrid';
import { CanonicalPiece, CellType, BiomeType } from '../dungeon/DungeonTypes';
import { Pathfinding } from '../dungeon/Pathfinding';
import { WFCGenerator } from '../dungeon/WFCGenerator';
import { KinematicsSolver } from '../animation/KinematicsSolver';
import { PotionInventory, PotionType, POTION_CONFIGS } from '../alchemy/PotionSystem';
import { PlayerState } from '../entities/PlayerState';
import { CritterInstance, CritterType, AIState } from '../entities/CritterAI';

describe('PRNG Determinístico', () => {
  it('gera sequência idêntica para sementes idênticas', () => {
    const prng1 = new PRNG(4242);
    const prng2 = new PRNG(4242);

    for (let i = 0; i < 20; i++) {
      expect(prng1.next()).toBe(prng2.next());
    }
  });

  it('respeita limites inteiros e de float', () => {
    const prng = new PRNG(999);
    for (let i = 0; i < 100; i++) {
      const val = prng.nextInt(5, 10);
      expect(val).toBeGreaterThanOrEqual(5);
      expect(val).toBeLessThanOrEqual(10);
    }
  });
});

describe('Dual Grid & As 6 Peças Canônicas (Stålberg)', () => {
  it('cobre todas as 16 máscaras possíveis mapeando rigorosamente para as 6 peças canônicas', () => {
    for (let mask = 0; mask < 16; mask++) {
      const mapping = MASK_TO_PIECE[mask];
      expect(mapping).toBeDefined();
      expect(Object.values(CanonicalPiece)).toContain(mapping.piece);
    }
  });

  it('valida peças canônicas universais específicas', () => {
    expect(MASK_TO_PIECE[0].piece).toBe(CanonicalPiece.EMPTY);
    expect(MASK_TO_PIECE[1].piece).toBe(CanonicalPiece.CONVEX_CORNER);
    expect(MASK_TO_PIECE[3].piece).toBe(CanonicalPiece.STRAIGHT_EDGE);
    expect(MASK_TO_PIECE[5].piece).toBe(CanonicalPiece.DIAGONAL_SADDLE);
    expect(MASK_TO_PIECE[14].piece).toBe(CanonicalPiece.CONCAVE_CORNER);
    expect(MASK_TO_PIECE[15].piece).toBe(CanonicalPiece.FULL_OPEN);
  });

  it('calcula dimensões corretas do Dual Grid (N-1 x M-1)', () => {
    const grid: CellType[][] = [
      [CellType.ROCK, CellType.ROCK, CellType.ROCK],
      [CellType.ROCK, CellType.CAVE, CellType.ROCK],
      [CellType.ROCK, CellType.ROCK, CellType.ROCK],
    ];
    const biomes: BiomeType[][] = [
      [BiomeType.NORMAL_CAVE, BiomeType.NORMAL_CAVE, BiomeType.NORMAL_CAVE],
      [BiomeType.NORMAL_CAVE, BiomeType.NORMAL_CAVE, BiomeType.NORMAL_CAVE],
      [BiomeType.NORMAL_CAVE, BiomeType.NORMAL_CAVE, BiomeType.NORMAL_CAVE],
    ];

    const quads = DualGridResolver.resolve(grid, biomes);
    expect(quads.length).toBe(4);
  });
});

describe('Pathfinding & Distância da Saída (>= 500m)', () => {
  it('calcula distância euclidiana/manhattan em metros baseado no tamanho da célula', () => {
    const path = [
      { x: 0, z: 0 },
      { x: 1, z: 0 },
      { x: 2, z: 0 },
    ];
    const dist = Pathfinding.calculatePathLengthMeters(path, 2.0);
    expect(dist).toBe(4.0);
  });

  it('garante que a saída gerada pelo WFCGenerator está a no mínimo 500 metros de caminhada', () => {
    const generator = new WFCGenerator({
      width: 90,
      height: 90,
      cellSize: 2.0,
      minExitDistanceMeters: 500.0,
      seed: 777,
    });

    const dungeon = generator.generate();

    expect(dungeon).toBeDefined();
    expect(dungeon.shortestPathLengthMeters).toBeGreaterThanOrEqual(500.0);
    expect(dungeon.dualQuads.length).toBeGreaterThan(0);
    expect(dungeon.clutter.length).toBeGreaterThan(0);
  });
});

describe('Cinemática Procedural & FABRIK', () => {
  it('mantém distância física estrita em restrições de distância com proteção a singularidade', () => {
    const anchor = { x: 0, y: 0, z: 0 };
    const follower = { x: 10, y: 0, z: 0 };
    const targetDist = 2.5;

    const result = KinematicsSolver.applyDistanceConstraint(anchor, follower, targetDist);
    const dist = Math.hypot(result.x - anchor.x, result.y - anchor.y, result.z - anchor.z);
    expect(dist).toBeCloseTo(targetDist, 5);

    const singularity = KinematicsSolver.applyDistanceConstraint(anchor, { ...anchor }, targetDist);
    const singDist = Math.hypot(singularity.x - anchor.x, singularity.y - anchor.y, singularity.z - anchor.z);
    expect(singDist).toBeCloseTo(targetDist, 5);
  });

  it('limita diferença angular para evitar dobras e colapso de vértebras', () => {
    const baseAngle = 0;
    const extremeTargetAngle = Math.PI;
    const maxTurn = Math.PI / 6;

    const clamped = KinematicsSolver.clampAngleDiff(baseAngle, extremeTargetAngle, maxTurn);
    expect(Math.abs(clamped - baseAngle)).toBeLessThanOrEqual(maxTurn + 1e-6);
  });

  it('preserva volume em deformação squash and stretch (Sx * Sy * Sz = 1)', () => {
    const deformationY = 0.5;
    const scales = KinematicsSolver.computeSquashAndStretch(deformationY);
    const volume = scales.sx * scales.sy * scales.sz;
    expect(volume).toBeCloseTo(1.0, 5);
  });

  it('FABRIK de 2 ossos conecta raiz ao efetuador com convergência', () => {
    const root = { x: 0, y: 0, z: 0 };
    const joints = [
      { x: 0, y: 0, z: 0 },
      { x: 1, y: 0, z: 0 },
      { x: 2, y: 0, z: 0 },
    ];
    const target = { x: 1.5, y: 0.5, z: 0 };
    KinematicsSolver.solveFabrik2Bone(root, joints, 1.0, target, 4);

    expect(joints[0].x).toBeCloseTo(root.x, 3);
    expect(joints[0].y).toBeCloseTo(root.y, 3);
  });
});

describe('Sistema Alquímico de Poções & HUD sem texto', () => {
  it('inicializa com 5 slots de poções e permite troca de atalho', () => {
    const inventory = new PotionInventory();
    expect(inventory.getSlots().length).toBe(5);
    expect(inventory.getActivePotion()).toBe(PotionType.GLUE);

    inventory.selectSlot(1);
    expect(inventory.getActivePotion()).toBe(PotionType.FIRE);

    inventory.selectSlot(4);
    expect(inventory.getActivePotion()).toBe(PotionType.LIGHT);
  });

  it('aplica cooldown e previne arremessos consecutivos imediatos', () => {
    const inventory = new PotionInventory();
    const canThrow1 = inventory.triggerThrow(PotionType.FIRE);
    expect(canThrow1).toBe(true);

    const canThrowImmediate = inventory.triggerThrow(PotionType.FIRE);
    expect(canThrowImmediate).toBe(false);

    inventory.updateCooldowns(POTION_CONFIGS[PotionType.FIRE].cooldownSeconds + 0.1);
    expect(inventory.canThrow(PotionType.FIRE)).toBe(true);
  });
});

describe('Estado do Jogador & Corações Estilo Minecraft', () => {
  it('possui altura estrita de 1,80m e calcula corações cheios/meios/vazios', () => {
    const player = new PlayerState();
    expect(player.height).toBe(1.80);

    let hearts = player.getDisplayHearts();
    expect(hearts.full).toBe(10);
    expect(hearts.half).toBe(0);
    expect(hearts.empty).toBe(0);

    player.takeDamage(3);
    hearts = player.getDisplayHearts();
    expect(hearts.full).toBe(8);
    expect(hearts.half).toBe(1);
    expect(hearts.empty).toBe(1);
  });

  it('respeita invulnerabilidade temporária (i-frames)', () => {
    const player = new PlayerState();
    player.takeDamage(4);
    const damagedAgain = player.takeDamage(4);
    expect(damagedAgain).toBe(false);
  });
});

describe('I.A. dos Bichinhos & Reações Alquímicas', () => {
  it('passa de patrulha para perseguição ao se aproximar do jogador', () => {
    const slime = new CritterInstance('slime_1', CritterType.SLIME, { x: 10, y: 0, z: 10 });
    expect(slime.state).toBe(AIState.PATROL);

    slime.update(0.1, { x: 12, y: 0, z: 10 });
    expect(slime.state).toBe(AIState.CHASE);
  });

  it('reage adequadamente às poções (cola desacelera, gelo congela, choque dobra em água)', () => {
    const wolf = new CritterInstance('wolf_1', CritterType.CAVE_WOLF, { x: 0, y: 0, z: 0 });
    wolf.applyPotionEffect(PotionType.ICE);
    expect(wolf.state).toBe(AIState.FROZEN);

    const wolfWater = new CritterInstance('wolf_water', CritterType.CAVE_WOLF, { x: 0, y: 0, z: 0 });
    const initialHp = wolfWater.currentHealth;
    wolfWater.applyPotionEffect(PotionType.SHOCK, true);
    expect(wolfWater.currentHealth).toBe(initialHp - 16);
  });
});
