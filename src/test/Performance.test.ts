import { describe, it, expect } from 'vitest';
import * as THREE from 'three';
import { WFCGenerator } from '../dungeon/WFCGenerator';
import { DualGridResolver } from '../dungeon/DualGrid';
import { DungeonRenderer } from '../render/DungeonRenderer';
import { KinematicsSolver } from '../animation/KinematicsSolver';
import { CritterInstance, CritterType } from '../entities/CritterAI';
import { PlayerState } from '../entities/PlayerState';
import { PlayerRenderer } from '../render/PlayerRenderer';
import { PotionInventory, PotionType, POTION_CONFIGS } from '../alchemy/PotionSystem';
import { BiomeType, CellType } from '../dungeon/DungeonTypes';

describe('Suíte de Testes de Performance & Análise de Gargalos', () => {
  it('mede e valida o tempo de geração procedural da dungeon (WFC + Pathfinding >= 500m)', () => {
    const runs = 3;
    const times: number[] = [];

    for (let i = 0; i < runs; i++) {
      const generator = new WFCGenerator({
        width: 90,
        height: 90,
        cellSize: 2.0,
        minExitDistanceMeters: 500.0,
        seed: 1000 + i * 37,
      });

      const start = performance.now();
      const dungeon = generator.generate();
      const duration = performance.now() - start;

      times.push(duration);
      expect(dungeon.shortestPathLengthMeters).toBeGreaterThanOrEqual(500.0);
    }

    const avgTime = times.reduce((a, b) => a + b, 0) / runs;
    expect(avgTime).toBeLessThan(350);
  });

  it('mede e valida o throughput do Dual Grid Resolver para 8.100 células', () => {
    const size = 90;
    const grid: CellType[][] = Array.from({ length: size }, () =>
      Array.from({ length: size }, () => (Math.random() < 0.6 ? CellType.CAVE : CellType.ROCK))
    );
    const biomes: BiomeType[][] = Array.from({ length: size }, () =>
      Array.from({ length: size }, () => BiomeType.NORMAL_CAVE)
    );

    const start = performance.now();
    const quads = DualGridResolver.resolve(grid, biomes);
    const duration = performance.now() - start;

    expect(quads.length).toBe((size - 1) * (size - 1));
    expect(duration).toBeLessThan(25);
  });

  it('mede a consolidação de geometria e garante redução de 99% nas draw calls da GPU', () => {
    const generator = new WFCGenerator({
      width: 90,
      height: 90,
      cellSize: 2.0,
      minExitDistanceMeters: 500.0,
      seed: 888,
    });
    const dungeon = generator.generate();

    const scene = new THREE.Scene();
    const renderer = new DungeonRenderer(scene);

    const start = performance.now();
    renderer.buildDungeon(dungeon);
    const duration = performance.now() - start;

    const totalMeshes = renderer.dungeonGroup.children.length + renderer.waterGroup.children.length;

    expect(duration).toBeLessThan(850);
    expect(totalMeshes).toBeLessThanOrEqual(25);
    expect(renderer.animatedLights.length).toBeLessThanOrEqual(5);

    renderer.clear();
  });

  it('mede o throughput matemático da Cinemática Procedural e FABRIK (> 1.000.000 ops/s)', () => {
    const iterations = 50000;

    const startDist = performance.now();
    let currentPos = { x: 0, y: 0, z: 0 };
    for (let i = 0; i < iterations; i++) {
      currentPos = KinematicsSolver.applyDistanceConstraint(
        { x: i * 0.1, y: 0, z: 0 },
        currentPos,
        1.5
      );
    }
    const distDuration = performance.now() - startDist;
    expect(distDuration).toBeLessThan(45);

    const startAngle = performance.now();
    let angle = 0;
    for (let i = 0; i < iterations; i++) {
      angle = KinematicsSolver.clampAngleDiff(angle, (i % 6) - 3, 0.4);
    }
    const angleDuration = performance.now() - startAngle;
    expect(angleDuration).toBeLessThan(35);

    const startSquash = performance.now();
    let volumeSum = 0;
    for (let i = 0; i < iterations; i++) {
      const def = (i % 20) / 20;
      const s = KinematicsSolver.computeSquashAndStretch(def);
      volumeSum += s.sx * s.sy * s.sz;
    }
    const squashDuration = performance.now() - startSquash;
    expect(squashDuration).toBeLessThan(35);
    expect(volumeSum / iterations).toBeCloseTo(1.0, 4);

    const startFabrik = performance.now();
    const root = { x: 0, y: 0, z: 0 };
    const joints = [
      { x: 0, y: 0, z: 0 },
      { x: 1, y: 0, z: 0 },
      { x: 2, y: 0, z: 0 },
    ];
    for (let i = 0; i < 10000; i++) {
      const target = { x: 1.0 + Math.sin(i) * 0.5, y: Math.cos(i) * 0.5, z: 0 };
      KinematicsSolver.solveFabrik2Bone(root, joints, 1.0, target, 3);
    }
    const fabrikDuration = performance.now() - startFabrik;
    expect(fabrikDuration).toBeLessThan(40);
  });

  it('mede e garante que a I.A. de 100 monstros em simultâneo consome menos de 1,5ms por tick', () => {
    const critters: CritterInstance[] = [];
    for (let i = 0; i < 100; i++) {
      const type = i % 2 === 0 ? CritterType.SLIME : CritterType.CAVE_WOLF;
      critters.push(
        new CritterInstance(`critter_${i}`, type, {
          x: (i % 10) * 8,
          y: 0,
          z: Math.floor(i / 10) * 8,
        })
      );
    }

    const playerPos = { x: 40, y: 0, z: 40 };
    const frames = 60;

    const start = performance.now();
    for (let f = 0; f < frames; f++) {
      for (let c = 0; c < critters.length; c++) {
        critters[c].update(0.016, playerPos);
      }
    }
    const totalDuration = performance.now() - start;
    const avgPerFrame = totalDuration / frames;

    expect(avgPerFrame).toBeLessThan(1.5);
  });

  it('mede a eficiência de detecção de colisão de poções em área contra hordas de monstros', () => {
    const critters: CritterInstance[] = [];
    for (let i = 0; i < 80; i++) {
      critters.push(
        new CritterInstance(`mob_${i}`, CritterType.SLIME, {
          x: 20 + (i % 9) * 2,
          y: 0,
          z: 20 + Math.floor(i / 9) * 2,
        })
      );
    }

    const potionThrows = 1000;
    const radius = POTION_CONFIGS[PotionType.ICE].effectRadius;

    const start = performance.now();
    let hitCount = 0;
    for (let t = 0; t < potionThrows; t++) {
      const impactPos = { x: 25 + (t % 5), z: 25 + (t % 5) };
      for (let c = 0; c < critters.length; c++) {
        const dist = Math.hypot(critters[c].position.x - impactPos.x, critters[c].position.z - impactPos.z);
        if (dist <= radius) {
          hitCount++;
        }
      }
    }
    const duration = performance.now() - start;
    const avgPerThrow = duration / potionThrows;

    expect(hitCount).toBeGreaterThan(0);
    expect(avgPerThrow).toBeLessThan(0.08);
  });

  it('mede a latência de cálculo de câmera em Primeira Pessoa e alinhamento de vetores', () => {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(65, 16 / 9, 0.1, 100);
    const renderer = new PlayerRenderer(camera, scene);
    const player = new PlayerState();

    const iterations = 5000;
    const start = performance.now();

    let lastFwdLen = 0;
    let lastRightLen = 0;

    for (let i = 0; i < iterations; i++) {
      renderer.rotateView(0.01, -0.005);
      const fwd = renderer.getForwardVector();
      const right = renderer.getHorizontalRight();
      renderer.update(player, 0.016, true);
      lastFwdLen = fwd.lengthSq();
      lastRightLen = right.lengthSq();
    }

    const duration = performance.now() - start;
    const avgPerTick = duration / iterations;

    expect(lastFwdLen).toBeCloseTo(1.0, 3);
    expect(lastRightLen).toBeCloseTo(1.0, 3);
    expect(avgPerTick).toBeLessThan(0.04);
  });

  it('valida que o orçamento total de processamento de um tick completo está abaixo de 4ms (Budget de 60 FPS = 16,6ms)', () => {
    const generator = new WFCGenerator({
      width: 90,
      height: 90,
      cellSize: 2.0,
      minExitDistanceMeters: 500.0,
      seed: 999,
    });
    const dungeon = generator.generate();

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(65, 16 / 9, 0.1, 100);
    const playerRenderer = new PlayerRenderer(camera, scene);
    const inventory = new PotionInventory();
    const player = new PlayerState();

    const critters: CritterInstance[] = [];
    for (let i = 0; i < 50; i++) {
      critters.push(
        new CritterInstance(`critter_${i}`, CritterType.CAVE_WOLF, {
          x: 20 + i * 2,
          y: 0,
          z: 20 + (i % 5) * 3,
        })
      );
    }

    const frames = 100;
    const start = performance.now();

    for (let f = 0; f < frames; f++) {
      player.update(0.016);
      playerRenderer.update(player, 0.016, true);
      inventory.updateCooldowns(0.016);

      for (let c = 0; c < critters.length; c++) {
        critters[c].update(0.016, player.position);
      }
    }

    const duration = performance.now() - start;
    const avgFrameTime = duration / frames;

    expect(avgFrameTime).toBeLessThan(3.5);
  });
});
