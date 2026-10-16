import * as THREE from 'three';
import { ActivePotionArea, PotionProjectile, PotionType, POTION_CONFIGS } from '../alchemy/PotionSystem';
import { Vec3 } from '../animation/KinematicsSolver';

export class PotionRenderer {
  public scene: THREE.Scene;
  public group: THREE.Group;
  private projectiles: { data: PotionProjectile; mesh: THREE.Mesh }[] = [];
  private activeAreas: { data: ActivePotionArea; mesh: THREE.Mesh; light?: THREE.PointLight }[] = [];

  constructor(scene: THREE.Scene) {
    this.scene = scene;
    this.group = new THREE.Group();
    this.scene.add(this.group);
  }

  public spawnProjectile(
    type: PotionType,
    startPos: Vec3,
    direction: THREE.Vector3,
    speed: number = 14.0
  ): void {
    const config = POTION_CONFIGS[type];
    const geo = new THREE.DodecahedronGeometry(0.14);
    const mat = new THREE.MeshBasicMaterial({ color: config.colorHex });
    const mesh = new THREE.Mesh(geo, mat);

    mesh.position.set(startPos.x, startPos.y + 1.2, startPos.z);
    this.group.add(mesh);

    const projectile: PotionProjectile = {
      id: `proj_${Date.now()}_${Math.random()}`,
      type,
      position: { x: mesh.position.x, y: mesh.position.y, z: mesh.position.z },
      velocity: {
        x: direction.x * speed,
        y: direction.y * speed + 3.0, // Impulso vertical para arco parabólico
        z: direction.z * speed,
      },
      alive: true,
    };

    this.projectiles.push({ data: projectile, mesh });
  }

  public update(
    deltaTime: number,
    onImpact: (type: PotionType, pos: Vec3) => void
  ): void {
    const gravity = -18.0;

    // Atualiza projéteis
    for (let i = this.projectiles.length - 1; i >= 0; i--) {
      const p = this.projectiles[i];
      p.data.velocity.y += gravity * deltaTime;

      p.data.position.x += p.data.velocity.x * deltaTime;
      p.data.position.y += p.data.velocity.y * deltaTime;
      p.data.position.z += p.data.velocity.z * deltaTime;

      p.mesh.position.set(p.data.position.x, p.data.position.y, p.data.position.z);
      p.mesh.rotation.x += 10 * deltaTime;
      p.mesh.rotation.y += 8 * deltaTime;

      // Colisão com o chão (y <= 0.1)
      if (p.data.position.y <= 0.1) {
        p.data.position.y = 0.1;
        this.group.remove(p.mesh);
        this.projectiles.splice(i, 1);

        onImpact(p.data.type, p.data.position);
        this.createActiveArea(p.data.type, p.data.position);
      }
    }

    // Atualiza poças / áreas ativas
    for (let i = this.activeAreas.length - 1; i >= 0; i--) {
      const area = this.activeAreas[i];
      area.data.remainingTime -= deltaTime;

      // Pulso visual
      const scale = 1.0 + Math.sin(area.data.remainingTime * 6) * 0.05;
      area.mesh.scale.set(scale, scale, 1);

      if (area.data.remainingTime <= 0) {
        this.group.remove(area.mesh);
        if (area.light) {
          this.group.remove(area.light);
        }
        this.activeAreas.splice(i, 1);
      }
    }
  }

  private createActiveArea(type: PotionType, pos: Vec3): void {
    const config = POTION_CONFIGS[type];
    const geo = new THREE.CircleGeometry(config.effectRadius, 24);
    geo.rotateX(-Math.PI / 2);

    const mat = new THREE.MeshBasicMaterial({
      color: config.colorHex,
      transparent: true,
      opacity: 0.5,
      side: THREE.DoubleSide,
    });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.position.set(pos.x, 0.05, pos.z);
    this.group.add(mesh);

    let light: THREE.PointLight | undefined;
    if (type === PotionType.LIGHT) {
      // Luz intensa de longa duração
      light = new THREE.PointLight(0xffffff, 4.0, 20.0, 2);
      light.position.set(pos.x, 1.8, pos.z);
      this.group.add(light);
    } else if (type === PotionType.FIRE) {
      light = new THREE.PointLight(0xff4411, 2.0, 8.0, 2);
      light.position.set(pos.x, 0.8, pos.z);
      this.group.add(light);
    }

    const activeArea: ActivePotionArea = {
      id: `area_${Date.now()}_${Math.random()}`,
      type,
      position: { ...pos },
      radius: config.effectRadius,
      remainingTime: config.durationSeconds,
      conductiveInWater: type === PotionType.SHOCK,
    };

    this.activeAreas.push({ data: activeArea, mesh, light });
  }

  public clear(): void {
    this.projectiles.forEach((p) => this.group.remove(p.mesh));
    this.activeAreas.forEach((a) => {
      this.group.remove(a.mesh);
      if (a.light) this.group.remove(a.light);
    });
    this.projectiles = [];
    this.activeAreas = [];
  }
}
