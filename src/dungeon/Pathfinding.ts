import { CellType } from './DungeonTypes';

export interface Point2D {
  x: number;
  z: number;
}

export class Pathfinding {
  /**
   * Encontra o menor caminho caminhável entre dois pontos usando BFS.
   * Retorna a lista de pontos ordenados do início ao destino, ou null se não houver caminho.
   */
  public static findShortestPath(
    grid: CellType[][],
    start: Point2D,
    goal: Point2D
  ): Point2D[] | null {
    const height = grid.length;
    const width = grid[0].length;

    if (
      start.x < 0 || start.x >= width || start.z < 0 || start.z >= height ||
      goal.x < 0 || goal.x >= width || goal.z < 0 || goal.z >= height
    ) {
      return null;
    }

    if (grid[start.z][start.x] === CellType.ROCK || grid[goal.z][goal.x] === CellType.ROCK) {
      return null;
    }

    const queue: Point2D[] = [start];
    const visited = new Set<string>();
    const parent = new Map<string, Point2D>();

    const key = (p: Point2D) => `${p.x},${p.z}`;
    visited.add(key(start));

    const directions = [
      { x: 0, z: -1 }, // Norte
      { x: 1, z: 0 },  // Leste
      { x: 0, z: 1 },  // Sul
      { x: -1, z: 0 }, // Oeste
    ];

    let found = false;

    while (queue.length > 0) {
      const current = queue.shift()!;
      if (current.x === goal.x && current.z === goal.z) {
        found = true;
        break;
      }

      for (const dir of directions) {
        const nx = current.x + dir.x;
        const nz = current.z + dir.z;

        if (nx >= 0 && nx < width && nz >= 0 && nz < height) {
          const neighborKey = `${nx},${nz}`;
          if (!visited.has(neighborKey) && grid[nz][nx] !== CellType.ROCK) {
            visited.add(neighborKey);
            parent.set(neighborKey, current);
            queue.push({ x: nx, z: nz });
          }
        }
      }
    }

    if (!found) return null;

    // Reconstrói o caminho
    const path: Point2D[] = [];
    let curr: Point2D | undefined = goal;
    while (curr) {
      path.push(curr);
      curr = parent.get(key(curr));
    }
    path.reverse();
    return path;
  }

  /**
   * Calcula a distância em metros do trajeto considerando o tamanho da célula.
   */
  public static calculatePathLengthMeters(path: Point2D[], cellSize: number = 2.0): number {
    if (path.length <= 1) return 0;
    let distance = 0;
    for (let i = 1; i < path.length; i++) {
      const dx = (path[i].x - path[i - 1].x) * cellSize;
      const dz = (path[i].z - path[i - 1].z) * cellSize;
      distance += Math.hypot(dx, dz);
    }
    return distance;
  }
}
