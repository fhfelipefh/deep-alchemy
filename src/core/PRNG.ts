/**
 * Gerador de números pseudo-aleatórios determinístico baseado em Mulberry32.
 * Garante que sementes idênticas gerem dungeons e posicionamentos idênticos,
 * essencial para testes unitários e reprodutibilidade de partidas.
 */
export class PRNG {
  private state: number;

  constructor(seed: number = 1337) {
    this.state = seed >>> 0;
  }

  /**
   * Retorna um número de ponto flutuante no intervalo [0, 1)
   */
  public next(): number {
    let t = (this.state += 0x6d2b79f5);
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  }

  /**
   * Retorna um inteiro no intervalo [min, max] (inclusivo)
   */
  public nextInt(min: number, max: number): number {
    return Math.floor(this.next() * (max - min + 1)) + min;
  }

  /**
   * Retorna um float no intervalo [min, max)
   */
  public nextFloat(min: number, max: number): number {
    return min + this.next() * (max - min);
  }

  /**
   * Retorna um valor booleano com base em uma probabilidade [0, 1]
   */
  public nextBool(probability: number = 0.5): boolean {
    return this.next() < probability;
  }

  /**
   * Escolhe um elemento aleatório de uma lista
   */
  public choose<T>(items: T[]): T {
    if (items.length === 0) {
      throw new Error("Cannot choose from empty array");
    }
    return items[Math.floor(this.next() * items.length)];
  }

  public getSeed(): number {
    return this.state;
  }
}
