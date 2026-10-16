import { Vec3 } from '../animation/KinematicsSolver';

export enum PotionType {
  GLUE = 'GLUE',
  FIRE = 'FIRE',
  SHOCK = 'SHOCK',
  ICE = 'ICE',
  LIGHT = 'LIGHT',
}

export interface PotionDefinition {
  type: PotionType;
  colorHex: number;
  effectRadius: number;
  durationSeconds: number;
  cooldownSeconds: number;
}

export const POTION_CONFIGS: Record<PotionType, PotionDefinition> = {
  [PotionType.GLUE]: {
    type: PotionType.GLUE,
    colorHex: 0x88cc22,
    effectRadius: 3.5,
    durationSeconds: 6.0,
    cooldownSeconds: 1.0,
  },
  [PotionType.FIRE]: {
    type: PotionType.FIRE,
    colorHex: 0xff4411,
    effectRadius: 3.0,
    durationSeconds: 4.0,
    cooldownSeconds: 1.2,
  },
  [PotionType.SHOCK]: {
    type: PotionType.SHOCK,
    colorHex: 0xffee22,
    effectRadius: 3.0,
    durationSeconds: 2.0,
    cooldownSeconds: 1.5,
  },
  [PotionType.ICE]: {
    type: PotionType.ICE,
    colorHex: 0x33ddff,
    effectRadius: 3.2,
    durationSeconds: 5.0,
    cooldownSeconds: 1.2,
  },
  [PotionType.LIGHT]: {
    type: PotionType.LIGHT,
    colorHex: 0xffffff,
    effectRadius: 10.0,
    durationSeconds: 15.0,
    cooldownSeconds: 0.8,
  },
};

export interface ActivePotionArea {
  id: string;
  type: PotionType;
  position: Vec3;
  radius: number;
  remainingTime: number;
  conductiveInWater: boolean;
}

export interface PotionProjectile {
  id: string;
  type: PotionType;
  position: Vec3;
  velocity: Vec3;
  alive: boolean;
}

export class PotionInventory {
  private activeSlotIndex: number = 0;
  private slots: PotionType[] = [
    PotionType.GLUE,
    PotionType.FIRE,
    PotionType.SHOCK,
    PotionType.ICE,
    PotionType.LIGHT,
  ];
  private cooldowns: Map<PotionType, number> = new Map();

  constructor() {
    for (const type of this.slots) {
      this.cooldowns.set(type, 0);
    }
  }

  public getSlots(): PotionType[] {
    return [...this.slots];
  }

  public getActiveSlotIndex(): number {
    return this.activeSlotIndex;
  }

  public getActivePotion(): PotionType {
    return this.slots[this.activeSlotIndex];
  }

  public selectSlot(index: number): boolean {
    if (index >= 0 && index < this.slots.length) {
      this.activeSlotIndex = index;
      return true;
    }
    return false;
  }

  public canThrow(type: PotionType): boolean {
    return (this.cooldowns.get(type) ?? 0) <= 0;
  }

  public triggerThrow(type: PotionType): boolean {
    if (!this.canThrow(type)) return false;
    const config = POTION_CONFIGS[type];
    this.cooldowns.set(type, config.cooldownSeconds);
    return true;
  }

  public updateCooldowns(deltaTime: number): void {
    for (const [type, cd] of this.cooldowns.entries()) {
      if (cd > 0) {
        this.cooldowns.set(type, Math.max(0, cd - deltaTime));
      }
    }
  }

  public getCooldownPercent(type: PotionType): number {
    const current = this.cooldowns.get(type) ?? 0;
    const max = POTION_CONFIGS[type].cooldownSeconds;
    if (max <= 0) return 0;
    return Math.min(1, Math.max(0, current / max));
  }
}
