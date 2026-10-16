/**
 * Tipos fundamentais e enums para a geração da caverna e do sistema de biomas.
 */

export enum CellType {
  ROCK = 0,         // Rocha sólida intransponível
  CAVE = 1,         // Chão de caverna normal (pedra/areia)
  WATER = 2,        // Setor alagado / corrente de água subterrânea
  GEOTHERMAL = 3,   // Setor vulcânico / fontes termais e gêiseres
  FUNGAL = 4,       // Bosque de fungos e musgos bioluminescentes
  BEACH_START = 5,  // Praia inicial (onde o jogador acorda após a cutscene)
  EXIT = 6,         // A saída misteriosa da caverna
}

export enum BiomeType {
  BEACH = 'BEACH',
  NORMAL_CAVE = 'NORMAL_CAVE',
  FLOODED = 'FLOODED',
  GEOTHERMAL = 'GEOTHERMAL',
  FUNGAL_GLOW = 'FUNGAL_GLOW',
}

/**
 * As 6 peças canônicas do Dual Grid (Stålberg Architecture)
 */
export enum CanonicalPiece {
  EMPTY = 0,        // 0 cantos preenchidos (Espaço fechado de rocha)
  CONVEX_CORNER = 1,// 1 canto preenchido (Canto convexo suave)
  STRAIGHT_EDGE = 2,// 2 cantos adjacentes (Parede reta contínua)
  DIAGONAL_SADDLE=3,// 2 cantos opostos (Cruzamento / Sela)
  CONCAVE_CORNER= 4,// 3 cantos preenchidos (Canto côncavo suave)
  FULL_OPEN = 5,    // 4 cantos preenchidos (Salão aberto contínuo)
}

export interface DualQuad {
  x: number;         // Coordenada no Dual Grid (mundo x = (x + 0.5) * cellSize)
  z: number;         // Coordenada no Dual Grid (mundo z = (z + 0.5) * cellSize)
  piece: CanonicalPiece;
  rotation: number;  // Rotação em radianos (0, PI/2, PI, 3*PI/2)
  biome: BiomeType;
  variantIndex: number;
  mask: number;      // 0 a 15
  corners: [boolean, boolean, boolean, boolean]; // NW, NE, SE, SW
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
  cellSize: number;       // Metros por célula primária (ex: 2.0m)
  grid: CellType[][];
  biomes: BiomeType[][];
  startPos: { x: number; z: number };
  exitPos: { x: number; z: number };
  shortestPathLengthMeters: number;
  dualQuads: DualQuad[];
  clutter: ClutterItem[];
}
