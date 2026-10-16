import { CanonicalPiece, CellType, DualQuad, BiomeType } from './DungeonTypes';

export interface PieceMapping {
  piece: CanonicalPiece;
  rotation: number;
}

/**
 * Mapeamento das 16 combinações binárias dos 4 cantos para as 6 peças canônicas.
 * Ordem dos bits: Bit 0 = NW, Bit 1 = NE, Bit 2 = SE, Bit 3 = SW
 */
export const MASK_TO_PIECE: Record<number, PieceMapping> = {
  0:  { piece: CanonicalPiece.EMPTY, rotation: 0 },
  1:  { piece: CanonicalPiece.CONVEX_CORNER, rotation: 0 },               // NW
  2:  { piece: CanonicalPiece.CONVEX_CORNER, rotation: Math.PI / 2 },     // NE
  4:  { piece: CanonicalPiece.CONVEX_CORNER, rotation: Math.PI },         // SE
  8:  { piece: CanonicalPiece.CONVEX_CORNER, rotation: Math.PI * 1.5 },   // SW

  3:  { piece: CanonicalPiece.STRAIGHT_EDGE, rotation: 0 },               // Norte (NW + NE)
  6:  { piece: CanonicalPiece.STRAIGHT_EDGE, rotation: Math.PI / 2 },     // Leste (NE + SE)
  12: { piece: CanonicalPiece.STRAIGHT_EDGE, rotation: Math.PI },         // Sul (SE + SW)
  9:  { piece: CanonicalPiece.STRAIGHT_EDGE, rotation: Math.PI * 1.5 },   // Oeste (SW + NW)

  5:  { piece: CanonicalPiece.DIAGONAL_SADDLE, rotation: 0 },             // NW + SE
  10: { piece: CanonicalPiece.DIAGONAL_SADDLE, rotation: Math.PI / 2 },   // NE + SW

  14: { piece: CanonicalPiece.CONCAVE_CORNER, rotation: 0 },              // Sem NW (NE+SE+SW)
  13: { piece: CanonicalPiece.CONCAVE_CORNER, rotation: Math.PI / 2 },    // Sem NE (NW+SE+SW)
  11: { piece: CanonicalPiece.CONCAVE_CORNER, rotation: Math.PI },        // Sem SE (NW+NE+SW)
  7:  { piece: CanonicalPiece.CONCAVE_CORNER, rotation: Math.PI * 1.5 },  // Sem SW (NW+NE+SE)

  15: { piece: CanonicalPiece.FULL_OPEN, rotation: 0 },
};

/**
 * Avalia o Dual Grid a partir da matriz primária.
 * O Dual Grid possui dimensões (width - 1) x (height - 1) e é deslocado em +0.5 células.
 */
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
        const variantIndex = hash % 4; // 4 variações estéticas por peça canônica

        // Determina o bioma predominante dos 4 cantos
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
