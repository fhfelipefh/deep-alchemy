import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { AIState, CritterInstance, CritterType } from '../entities/CritterAI';
import { KinematicsSolver } from '../animation/KinematicsSolver';
import { createCelMaterial } from './CelShader';

interface CritterVisual {
  instance: CritterInstance;
  group: THREE.Group;
  bodyMesh: THREE.Object3D;
  iceCubeMesh?: THREE.Mesh;
  animTimer: number;
}

export class CritterRenderer {
  public scene: THREE.Scene;
  public critterGroup: THREE.Group;
  private visuals: Map<string, CritterVisual> = new Map();
  private slimeTemplate: THREE.Object3D | null = null;
  private wolfTemplate: THREE.Object3D | null = null;

  constructor(scene: THREE.Scene) {
    this.scene = scene;
    this.critterGroup = new THREE.Group();
    this.scene.add(this.critterGroup);

    this.preloadTemplates();
  }

  private preloadTemplates(): void {
    const loader = new GLTFLoader();
    loader.load('models/slime.glb', (gltf) => {
      this.slimeTemplate = gltf.scene;
    });
    loader.load('models/cave_wolf.glb', (gltf) => {
      this.wolfTemplate = gltf.scene;
    });
  }

  public registerCritter(instance: CritterInstance): void {
    const group = new THREE.Group();
    group.position.set(instance.position.x, instance.position.y, instance.position.z);

    let bodyMesh: THREE.Object3D;

    if (instance.config.type === CritterType.SLIME) {
      if (this.slimeTemplate) {
        bodyMesh = this.slimeTemplate.clone(true);
      } else {
        // Fallback procedural
        const geo = new THREE.SphereGeometry(0.45, 12, 10);
        const mat = createCelMaterial({ color: 0x33cc66, roughness: 0.3 });
        bodyMesh = new THREE.Mesh(geo, mat);
        bodyMesh.position.y = 0.4;
      }
    } else if (instance.config.type === CritterType.CAVE_WOLF) {
      if (this.wolfTemplate) {
        bodyMesh = this.wolfTemplate.clone(true);
      } else {
        const geo = new THREE.BoxGeometry(0.4, 0.4, 0.7);
        const mat = createCelMaterial({ color: 0x5a6070, roughness: 0.7 });
        bodyMesh = new THREE.Mesh(geo, mat);
        bodyMesh.position.y = 0.35;
      }
    } else {
      // Glow Beetle
      const geo = new THREE.SphereGeometry(0.25, 8, 6);
      const mat = createCelMaterial({ color: 0x00ffff, emissive: 0x00ffff, emissiveIntensity: 1.5 });
      bodyMesh = new THREE.Mesh(geo, mat);
      bodyMesh.position.y = 0.25;
    }

    bodyMesh.castShadow = true;
    bodyMesh.receiveShadow = true;
    group.add(bodyMesh);

    // Bloco de Gelo para quando congelado
    const iceGeo = new THREE.BoxGeometry(1.0, 1.0, 1.0);
    const iceMat = new THREE.MeshStandardMaterial({
      color: 0x88e0ff,
      transparent: true,
      opacity: 0.7,
      roughness: 0.1,
    });
    const iceCube = new THREE.Mesh(iceGeo, iceMat);
    iceCube.position.y = 0.5;
    iceCube.visible = false;
    group.add(iceCube);

    this.critterGroup.add(group);
    this.visuals.set(instance.id, {
      instance,
      group,
      bodyMesh,
      iceCubeMesh: iceCube,
      animTimer: Math.random() * 10,
    });
  }

  public update(deltaTime: number): void {
    for (const [id, visual] of this.visuals.entries()) {
      const critter = visual.instance;
      if (!critter.isAlive) {
        this.critterGroup.remove(visual.group);
        this.visuals.delete(id);
        continue;
      }

      visual.animTimer += deltaTime;

      // Sincroniza posição e rotação
      visual.group.position.set(critter.position.x, critter.position.y, critter.position.z);
      visual.group.rotation.y = critter.rotation;

      // Status visual de gelo
      if (visual.iceCubeMesh) {
        visual.iceCubeMesh.visible = critter.state === AIState.FROZEN;
      }

      // Animação procedural por criatura
      if (critter.config.type === CritterType.SLIME) {
        // Squash and Stretch procedural em tempo real
        const hopFrequency = critter.state === AIState.CHASE ? 8.0 : 4.0;
        const hopPhase = Math.sin(visual.animTimer * hopFrequency);
        const squash = KinematicsSolver.computeSquashAndStretch(hopPhase * 0.35);

        visual.bodyMesh.scale.set(squash.sx, squash.sy, squash.sz);
        visual.group.position.y = Math.max(0, hopPhase * 0.3);
      } else if (critter.config.type === CritterType.CAVE_WOLF) {
        // Balanço sutil de respiração e trote
        const trotSpeed = critter.state === AIState.CHASE ? 12.0 : 6.0;
        visual.bodyMesh.rotation.z = Math.sin(visual.animTimer * trotSpeed) * 0.08;
      }
    }
  }

  public clear(): void {
    while (this.critterGroup.children.length > 0) {
      this.critterGroup.remove(this.critterGroup.children[0]);
    }
    this.visuals.clear();
  }
}
