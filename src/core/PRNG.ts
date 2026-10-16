export class PRNG {
  private state: number;

  constructor(seed: number = 1337) {
    this.state = seed >>> 0;
  }

  public next(): number {
    let t = (this.state += 0x6d2b79f5);
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  }

  public nextInt(min: number, max: number): number {
    return Math.floor(this.next() * (max - min + 1)) + min;
  }

  public nextFloat(min: number, max: number): number {
    return min + this.next() * (max - min);
  }

  public nextBool(probability: number = 0.5): boolean {
    return this.next() < probability;
  }

  public getSeed(): number {
    return this.state;
  }
}
