import { PRNG } from '../core/PRNG';
import { BiomeType, CellType, ClutterItem, DungeonMap } from './DungeonTypes';
import { DualGridResolver } from './DualGrid';
import { Pathfinding, Point2D } from './Pathfinding';

export interface GeneratorOptions {
  width?: number;
  height?: number;
  cellSize?: number;
  minExitDistanceMeters?: number;
  seed?: number;
}

export class WFCGenerator {
  private prng: PRNG;
  private width: number;
  private height: number;
  private cellSize: number;
  private minExitDistanceMeters: number;

  constructor(options: GeneratorOptions = {}) {
    this.width = options.width ?? 90;
    this.height = options.height ?? 90;
    this.cellSize = options.cellSize ?? 2.0;
    this.minExitDistanceMeters = options.minExitDistanceMeters ?? 500.0;
    this.prng = new PRNG(options.seed ?? Date.now());
  }

  public generate(): DungeonMap {
    let attempts = 0;
    const maxAttempts = 20;

    while (attempts < maxAttempts) {
      attempts++;
      const dungeon = this.tryGenerateOnce();
      if (dungeon && dungeon.shortestPathLengthMeters >= this.minExitDistanceMeters) {
        return dungeon;
      }
    }

    return this.generateGuaranteedLongDungeon();
  }

  private tryGenerateOnce(): DungeonMap | null {
    const grid: CellType[][] = Array.from({ length: this.height }, () =>
      Array.from({ length: this.width }, () => CellType.ROCK)
    );

    const biomes: BiomeType[][] = Array.from({ length: this.height }, () =>
      Array.from({ length: this.width }, () => BiomeType.NORMAL_CAVE)
    );

    const startPos: Point2D = { x: 6, z: this.height - 7 };

    for (let dz = -3; dz <= 3; dz++) {
      for (let dx = -3; dx <= 3; dx++) {
        const px = startPos.x + dx;
        const pz = startPos.z + dz;
        if (px >= 1 && px < this.width - 1 && pz >= 1 && pz < this.height - 1) {
          grid[pz][px] = CellType.BEACH_START;
          biomes[pz][px] = BiomeType.BEACH;
        }
      }
    }

    const waypoints: Point2D[] = [
      startPos,
      { x: Math.floor(this.width * 0.82), z: Math.floor(this.height * 0.8) },
      { x: Math.floor(this.width * 0.18), z: Math.floor(this.height * 0.65) },
      { x: Math.floor(this.width * 0.82), z: Math.floor(this.height * 0.5) },
      { x: Math.floor(this.width * 0.18), z: Math.floor(this.height * 0.35) },
      { x: Math.floor(this.width * 0.82), z: Math.floor(this.height * 0.2) },
      { x: Math.floor(this.width * 0.18), z: 6 },
    ];

    for (let i = 0; i < waypoints.length - 1; i++) {
      this.carveOrganicTunnel(grid, waypoints[i], waypoints[i + 1]);
    }

    this.carveSecondaryChambers(grid);
    this.assignBiomes(grid, biomes);

    const exitPos = waypoints[waypoints.length - 1];
    grid[exitPos.z][exitPos.x] = CellType.EXIT;

    const path = Pathfinding.findShortestPath(grid, startPos, exitPos);
    if (!path) return null;

    const shortestPathLengthMeters = Pathfinding.calculatePathLengthMeters(path, this.cellSize);
    const dualQuads = DualGridResolver.resolve(grid, biomes, this.prng.getSeed());
    const clutter = this.generateClutter(grid, biomes);

    return {
      width: this.width,
      height: this.height,
      cellSize: this.cellSize,
      grid,
      biomes,
      startPos,
      exitPos,
      shortestPathLengthMeters,
      dualQuads,
      clutter,
    };
  }

  private generateGuaranteedLongDungeon(): DungeonMap {
    const grid: CellType[][] = Array.from({ length: this.height }, () =>
      Array.from({ length: this.width }, () => CellType.ROCK)
    );

    const biomes: BiomeType[][] = Array.from({ length: this.height }, () =>
      Array.from({ length: this.width }, () => BiomeType.NORMAL_CAVE)
    );

    const startPos: Point2D = { x: 6, z: this.height - 7 };
    for (let dz = -3; dz <= 3; dz++) {
      for (let dx = -3; dx <= 3; dx++) {
        const px = startPos.x + dx;
        const pz = startPos.z + dz;
        if (px >= 1 && px < this.width - 1 && pz >= 1 && pz < this.height - 1) {
          grid[pz][px] = CellType.BEACH_START;
          biomes[pz][px] = BiomeType.BEACH;
        }
      }
    }

    const numRows = Math.floor((this.height - 16) / 8);
    let currentX = startPos.x;
    let currentZ = startPos.z - 3;

    for (let row = 0; row < numRows; row++) {
      const targetX = row % 2 === 0 ? this.width - 8 : 8;
      const stepX = targetX > currentX ? 1 : -1;
      while (currentX !== targetX) {
        currentX += stepX;
        this.carveBrush(grid, currentX, currentZ, 1);
      }
      if (row < numRows - 1) {
        for (let s = 0; s < 8; s++) {
          currentZ -= 1;
          this.carveBrush(grid, currentX, currentZ, 1);
        }
      }
    }

    const exitPos: Point2D = { x: currentX, z: currentZ };
    grid[exitPos.z][exitPos.x] = CellType.EXIT;

    this.assignBiomes(grid, biomes);

    const path = Pathfinding.findShortestPath(grid, startPos, exitPos)!;
    const shortestPathLengthMeters = Pathfinding.calculatePathLengthMeters(path, this.cellSize);
    const dualQuads = DualGridResolver.resolve(grid, biomes, this.prng.getSeed());
    const clutter = this.generateClutter(grid, biomes);

    return {
      width: this.width,
      height: this.height,
      cellSize: this.cellSize,
      grid,
      biomes,
      startPos,
      exitPos,
      shortestPathLengthMeters,
      dualQuads,
      clutter,
    };
  }

  private carveOrganicTunnel(grid: CellType[][], p1: Point2D, p2: Point2D): void {
    let cx = p1.x;
    let cz = p1.z;

    while (cx !== p2.x || cz !== p2.z) {
      this.carveBrush(grid, cx, cz, this.prng.nextInt(1, 2));

      const chooseHorizontal = this.prng.next() < 0.6;
      if (chooseHorizontal && cx !== p2.x) {
        cx += p2.x > cx ? 1 : -1;
      } else if (cz !== p2.z) {
        cz += p2.z > cz ? 1 : -1;
      } else if (cx !== p2.x) {
        cx += p2.x > cx ? 1 : -1;
      }

      if (this.prng.next() < 0.2) {
        const jitterZ = cz + this.prng.nextInt(-1, 1);
        const jitterX = cx + this.prng.nextInt(-1, 1);
        if (jitterX >= 2 && jitterX < this.width - 2 && jitterZ >= 2 && jitterZ < this.height - 2) {
          this.carveBrush(grid, jitterX, jitterZ, 1);
        }
      }
    }
    this.carveBrush(grid, p2.x, p2.z, 2);
  }

  private carveSecondaryChambers(grid: CellType[][]): void {
    const numChambers = this.prng.nextInt(8, 14);
    for (let c = 0; c < numChambers; c++) {
      const rx = this.prng.nextInt(8, this.width - 8);
      const rz = this.prng.nextInt(8, this.height - 8);
      const radius = this.prng.nextInt(3, 5);

      for (let dz = -radius; dz <= radius; dz++) {
        for (let dx = -radius; dx <= radius; dx++) {
          if (dx * dx + dz * dz <= radius * radius) {
            const x = rx + dx;
            const z = rz + dz;
            if (x >= 2 && x < this.width - 2 && z >= 2 && z < this.height - 2) {
              if (grid[z][x] === CellType.ROCK) {
                grid[z][x] = CellType.CAVE;
              }
            }
          }
        }
      }
    }
  }

  private carveBrush(grid: CellType[][], cx: number, cz: number, radius: number): void {
    for (let dz = -radius; dz <= radius; dz++) {
      for (let dx = -radius; dx <= radius; dx++) {
        if (dx * dx + dz * dz <= radius * radius + 0.5) {
          const x = cx + dx;
          const z = cz + dz;
          if (x >= 2 && x < this.width - 2 && z >= 2 && z < this.height - 2) {
            if (grid[z][x] === CellType.ROCK) {
              grid[z][x] = CellType.CAVE;
            }
          }
        }
      }
    }
  }

  private assignBiomes(grid: CellType[][], biomes: BiomeType[][]): void {
    for (let z = 0; z < this.height; z++) {
      for (let x = 0; x < this.width; x++) {
        if (grid[z][x] === CellType.ROCK) continue;
        if (grid[z][x] === CellType.BEACH_START) continue;

        const normalizedX = x / this.width;
        const normalizedZ = z / this.height;

        if (normalizedZ < 0.35 && normalizedX < 0.6) {
          biomes[z][x] = BiomeType.FLOODED;
          if (this.prng.next() < 0.4) {
            grid[z][x] = CellType.WATER;
          }
        } else if (normalizedX > 0.55 && normalizedZ > 0.45) {
          biomes[z][x] = BiomeType.GEOTHERMAL;
          grid[z][x] = CellType.GEOTHERMAL;
        } else if (normalizedZ > 0.5 && normalizedX < 0.45) {
          biomes[z][x] = BiomeType.FUNGAL_GLOW;
          grid[z][x] = CellType.FUNGAL;
        } else {
          biomes[z][x] = BiomeType.NORMAL_CAVE;
          grid[z][x] = CellType.CAVE;
        }
      }
    }
  }

  private generateClutter(grid: CellType[][], biomes: BiomeType[][]): ClutterItem[] {
    const clutter: ClutterItem[] = [];
    let idCounter = 0;

    for (let z = 2; z < this.height - 2; z += 2) {
      for (let x = 2; x < this.width - 2; x += 2) {
        const cell = grid[z][x];
        if (cell === CellType.ROCK) continue;

        const biome = biomes[z][x];
        const worldX = (x + 0.5) * this.cellSize;
        const worldZ = (z + 0.5) * this.cellSize;

        if (biome === BiomeType.FUNGAL_GLOW && this.prng.next() < 0.35) {
          const isCyan = this.prng.nextBool(0.6);
          clutter.push({
            id: `clutter_${++idCounter}`,
            type: isCyan ? 'MUSHROOM_CYAN' : 'MUSHROOM_BLUE',
            x: worldX + this.prng.nextFloat(-0.5, 0.5),
            y: 0,
            z: worldZ + this.prng.nextFloat(-0.5, 0.5),
            scale: this.prng.nextFloat(0.6, 1.4),
            rotation: this.prng.nextFloat(0, Math.PI * 2),
            lightIntensity: 1.5,
            lightColor: isCyan ? 0x00ffff : 0x4488ff,
          });
        } else if (biome === BiomeType.GEOTHERMAL && this.prng.next() < 0.2) {
          clutter.push({
            id: `clutter_${++idCounter}`,
            type: 'GEYSER',
            x: worldX,
            y: 0,
            z: worldZ,
            scale: this.prng.nextFloat(0.8, 1.2),
            rotation: this.prng.nextFloat(0, Math.PI * 2),
            lightIntensity: 0.8,
            lightColor: 0xffaa44,
          });
        } else if (this.prng.next() < 0.15) {
          clutter.push({
            id: `clutter_${++idCounter}`,
            type: this.prng.nextBool() ? 'ROCK_PILE' : 'GLOW_MOSS',
            x: worldX + this.prng.nextFloat(-0.4, 0.4),
            y: 0,
            z: worldZ + this.prng.nextFloat(-0.4, 0.4),
            scale: this.prng.nextFloat(0.5, 1.1),
            rotation: this.prng.nextFloat(0, Math.PI * 2),
          });
        }
      }
    }

    return clutter;
  }
}
