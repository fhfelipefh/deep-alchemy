import { Vec3 } from '../animation/KinematicsSolver';
import { BiomeType } from '../dungeon/DungeonTypes';
import { PotionType } from '../alchemy/PotionSystem';

export enum CritterType {
  SLIME = 'SLIME',
  CAVE_WOLF = 'CAVE_WOLF',
  GLOW_BEETLE = 'GLOW_BEETLE',
}

export enum AIState {
  PATROL = 'PATROL',
  ALERT = 'ALERT',
  CHASE = 'CHASE',
  FLEE = 'FLEE',
  GLUED = 'GLUED',
  FROZEN = 'FROZEN',
}

export interface CritterConfig {
  type: CritterType;
  preferredBiome: BiomeType;
  baseSpeed: number;
  detectionRadius: number;
  attackDamage: number;
  maxHealth: number;
}

export const CRITTER_CONFIGS: Record<CritterType, CritterConfig> = {
  [CritterType.SLIME]: {
    type: CritterType.SLIME,
    preferredBiome: BiomeType.FLOODED,
    baseSpeed: 2.2,
    detectionRadius: 8.0,
    attackDamage: 2,
    maxHealth: 15,
  },
  [CritterType.CAVE_WOLF]: {
    type: CritterType.CAVE_WOLF,
    preferredBiome: BiomeType.GEOTHERMAL,
    baseSpeed: 4.8,
    detectionRadius: 12.0,
    attackDamage: 3,
    maxHealth: 25,
  },
  [CritterType.GLOW_BEETLE]: {
    type: CritterType.GLOW_BEETLE,
    preferredBiome: BiomeType.FUNGAL_GLOW,
    baseSpeed: 1.8,
    detectionRadius: 6.0,
    attackDamage: 1,
    maxHealth: 10,
  },
};

export class CritterInstance {
  public id: string;
  public config: CritterConfig;
  public position: Vec3;
  public velocity: Vec3 = { x: 0, y: 0, z: 0 };
  public rotation: number = 0;
  public state: AIState = AIState.PATROL;
  public currentHealth: number;
  public isAlive: boolean = true;

  public glueTimer: number = 0;
  public frozenTimer: number = 0;
  public burningTimer: number = 0;

  public patrolWaypoints: Vec3[] = [];
  public currentWaypointIndex: number = 0;

  constructor(id: string, type: CritterType, initialPos: Vec3) {
    this.id = id;
    this.config = CRITTER_CONFIGS[type];
    this.position = { ...initialPos };
    this.currentHealth = this.config.maxHealth;

    this.patrolWaypoints = [
      { x: initialPos.x, y: initialPos.y, z: initialPos.z },
      { x: initialPos.x + 3.0, y: initialPos.y, z: initialPos.z + 2.0 },
      { x: initialPos.x - 2.0, y: initialPos.y, z: initialPos.z - 3.0 },
    ];
  }

  public applyPotionEffect(potionType: PotionType, inWater: boolean = false): void {
    switch (potionType) {
      case PotionType.GLUE:
        this.glueTimer = 5.0;
        this.state = AIState.GLUED;
        break;

      case PotionType.ICE:
        this.frozenTimer = 4.5;
        this.state = AIState.FROZEN;
        break;

      case PotionType.FIRE:
        this.burningTimer = 3.5;
        this.takeDamage(6);
        if (this.config.type === CritterType.CAVE_WOLF) {
          this.state = AIState.FLEE;
        }
        break;

      case PotionType.SHOCK: {
        const shockDamage = inWater ? 16 : 8;
        this.takeDamage(shockDamage);
        break;
      }

      case PotionType.LIGHT:
        if (this.config.type === CritterType.GLOW_BEETLE) {
          this.state = AIState.ALERT;
        }
        break;
    }
  }

  public takeDamage(amount: number): void {
    if (!this.isAlive) return;
    this.currentHealth = Math.max(0, this.currentHealth - amount);
    if (this.currentHealth <= 0) {
      this.isAlive = false;
    }
  }

  public update(deltaTime: number, playerPos: Vec3): void {
    if (!this.isAlive) return;

    if (this.frozenTimer > 0) {
      this.frozenTimer -= deltaTime;
      if (this.frozenTimer <= 0 && this.state === AIState.FROZEN) {
        this.state = AIState.PATROL;
      }
      return;
    }

    if (this.glueTimer > 0) {
      this.glueTimer -= deltaTime;
      if (this.glueTimer <= 0 && this.state === AIState.GLUED) {
        this.state = AIState.PATROL;
      }
    }

    if (this.burningTimer > 0) {
      this.burningTimer -= deltaTime;
      this.takeDamage(2 * deltaTime);
    }

    const dx = playerPos.x - this.position.x;
    const dz = playerPos.z - this.position.z;
    const distToPlayer = Math.hypot(dx, dz);

    if (this.state !== AIState.GLUED && this.state !== AIState.FROZEN && this.state !== AIState.FLEE) {
      if (distToPlayer <= this.config.detectionRadius) {
        this.state = AIState.CHASE;
      } else if (distToPlayer > this.config.detectionRadius * 1.5) {
        this.state = AIState.PATROL;
      }
    }

    let effectiveSpeed = this.config.baseSpeed;
    if (this.state === AIState.GLUED) {
      effectiveSpeed *= 0.2;
    }

    if (this.state === AIState.CHASE) {
      const angle = Math.atan2(dz, dx);
      this.rotation = angle;
      this.velocity.x = Math.cos(angle) * effectiveSpeed;
      this.velocity.z = Math.sin(angle) * effectiveSpeed;

      this.position.x += this.velocity.x * deltaTime;
      this.position.z += this.velocity.z * deltaTime;
    } else if (this.state === AIState.FLEE) {
      const angle = Math.atan2(-dz, -dx);
      this.rotation = angle;
      this.velocity.x = Math.cos(angle) * effectiveSpeed;
      this.velocity.z = Math.sin(angle) * effectiveSpeed;

      this.position.x += this.velocity.x * deltaTime;
      this.position.z += this.velocity.z * deltaTime;
    } else if (this.state === AIState.PATROL) {
      const targetWp = this.patrolWaypoints[this.currentWaypointIndex];
      const wx = targetWp.x - this.position.x;
      const wz = targetWp.z - this.position.z;
      const wpDist = Math.hypot(wx, wz);

      if (wpDist < 0.5) {
        this.currentWaypointIndex = (this.currentWaypointIndex + 1) % this.patrolWaypoints.length;
      } else {
        const angle = Math.atan2(wz, wx);
        this.rotation = angle;
        this.position.x += Math.cos(angle) * (effectiveSpeed * 0.5) * deltaTime;
        this.position.z += Math.sin(angle) * (effectiveSpeed * 0.5) * deltaTime;
      }
    }
  }
}
