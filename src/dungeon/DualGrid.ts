import { CanonicalPiece, CellType, DualQuad, BiomeType } from './DungeonTypes';

export interface PieceMapping {
  piece: CanonicalPiece;
  rotation: number;
}

export const MASK_TO_PIECE: Record<number, PieceMapping> = {
  0:  { piece: CanonicalPiece.EMPTY, rotation: 0 },
  1:  { piece: CanonicalPiece.CONVEX_CORNER, rotation: 0 },
  2:  { piece: CanonicalPiece.CONVEX_CORNER, rotation: Math.PI / 2 },
  4:  { piece: CanonicalPiece.CONVEX_CORNER, rotation: Math.PI },
  8:  { piece: CanonicalPiece.CONVEX_CORNER, rotation: Math.PI * 1.5 },
  3:  { piece: CanonicalPiece.STRAIGHT_EDGE, rotation: 0 },
  6:  { piece: CanonicalPiece.STRAIGHT_EDGE, rotation: Math.PI / 2 },
  12: { piece: CanonicalPiece.STRAIGHT_EDGE, rotation: Math.PI },
  9:  { piece: CanonicalPiece.STRAIGHT_EDGE, rotation: Math.PI * 1.5 },
  5:  { piece: CanonicalPiece.DIAGONAL_SADDLE, rotation: 0 },
  10: { piece: CanonicalPiece.DIAGONAL_SADDLE, rotation: Math.PI / 2 },
  14: { piece: CanonicalPiece.CONCAVE_CORNER, rotation: 0 },
  13: { piece: CanonicalPiece.CONCAVE_CORNER, rotation: Math.PI / 2 },
  11: { piece: CanonicalPiece.CONCAVE_CORNER, rotation: Math.PI },
  7:  { piece: CanonicalPiece.CONCAVE_CORNER, rotation: Math.PI * 1.5 },
  15: { piece: CanonicalPiece.FULL_OPEN, rotation: 0 },
};

export class DualGridResolver {
  public static isWalkable(cell: CellType): boolean {
    return cell !== CellType.ROCK;
  }

  public static hashCoord(x: number, z: number, seed: number = 42): number {
    let h = (x * 374761393 + z * 668265263 + seed) >>> 0;
    h = Math.imul(h ^ (h >>> 13), 1274126177);
    return (h ^ (h >>> 16)) >>> 0;
  }

  public static resolve(
    primaryGrid: CellType[][],
    biomeGrid: BiomeType[][],
    seed: number = 1337
  ): DualQuad[] {
    const height = primaryGrid.length;
    if (height < 2) return [];
    const width = primaryGrid[0].length;
    if (width < 2) return [];

    const dualQuads: DualQuad[] = [];

    for (let z = 0; z < height - 1; z++) {
      for (let x = 0; x < width - 1; x++) {
        const cNW = this.isWalkable(primaryGrid[z][x]);
        const cNE = this.isWalkable(primaryGrid[z][x + 1]);
        const cSE = this.isWalkable(primaryGrid[z + 1][x + 1]);
        const cSW = this.isWalkable(primaryGrid[z + 1][x]);

        const mask = (cNW ? 1 : 0) |
                     (cNE ? 2 : 0) |
                     (cSE ? 4 : 0) |
                     (cSW ? 8 : 0);

        const mapping = MASK_TO_PIECE[mask];
        const hash = this.hashCoord(x, z, seed);
        const variantIndex = hash % 4;

        const quadBiome = biomeGrid[z][x] || BiomeType.NORMAL_CAVE;
        const isWater = primaryGrid[z][x] === CellType.WATER ||
                        primaryGrid[z][x + 1] === CellType.WATER ||
                        primaryGrid[z + 1][x + 1] === CellType.WATER ||
                        primaryGrid[z + 1][x] === CellType.WATER;

        dualQuads.push({
          x,
          z,
          piece: mapping.piece,
          rotation: mapping.rotation,
          biome: quadBiome,
          variantIndex,
          mask,
          corners: [cNW, cNE, cSE, cSW],
          waterHeight: isWater ? 0.3 : 0.0,
        });
      }
    }

    return dualQuads;
  }
}
