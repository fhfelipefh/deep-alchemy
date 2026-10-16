export enum CellType {
  ROCK = 0,
  CAVE = 1,
  WATER = 2,
  GEOTHERMAL = 3,
  FUNGAL = 4,
  BEACH_START = 5,
  EXIT = 6,
}

export enum BiomeType {
  BEACH = 'BEACH',
  NORMAL_CAVE = 'NORMAL_CAVE',
  FLOODED = 'FLOODED',
  GEOTHERMAL = 'GEOTHERMAL',
  FUNGAL_GLOW = 'FUNGAL_GLOW',
}

export enum CanonicalPiece {
  EMPTY = 0,
  CONVEX_CORNER = 1,
  STRAIGHT_EDGE = 2,
  DIAGONAL_SADDLE = 3,
  CONCAVE_CORNER = 4,
  FULL_OPEN = 5,
}

export interface DualQuad {
  x: number;
  z: number;
  piece: CanonicalPiece;
  rotation: number;
  biome: BiomeType;
  variantIndex: number;
  mask: number;
  corners: [boolean, boolean, boolean, boolean];
  waterHeight: number;
}

export interface ClutterItem {
  id: string;
  type: 'MUSHROOM_BLUE' | 'MUSHROOM_CYAN' | 'GLOW_MOSS' | 'STALACTITE' | 'GEYSER' | 'ROCK_PILE';
  x: number;
  y: number;
  z: number;
  scale: number;
  rotation: number;
  lightIntensity?: number;
  lightColor?: number;
}

export interface DungeonMap {
  width: number;
  height: number;
  cellSize: number;
  grid: CellType[][];
  biomes: BiomeType[][];
  startPos: { x: number; z: number };
  exitPos: { x: number; z: number };
  shortestPathLengthMeters: number;
  dualQuads: DualQuad[];
  clutter: ClutterItem[];
}
