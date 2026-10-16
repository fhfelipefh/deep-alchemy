import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { AIState, CritterInstance, CritterType } from '../entities/CritterAI';
import { createCelMaterial } from './CelShader';

interface CritterVisual {
  instance: CritterInstance;
  group: THREE.Group;
  bodyMesh: THREE.Object3D;
  iceCubeMesh?: THREE.Mesh;
  mixer?: THREE.AnimationMixer;
}

export class CritterRenderer {
  public scene: THREE.Scene;
  public critterGroup: THREE.Group;
  private visuals: Map<string, CritterVisual> = new Map();
  private slimeTemplate: THREE.Object3D | null = null;
  private slimeClip: THREE.AnimationClip | null = null;
  private wolfTemplate: THREE.Object3D | null = null;
  private wolfClip: THREE.AnimationClip | null = null;

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
      if (gltf.animations.length > 0) {
        this.slimeClip = gltf.animations[0];
      }
    });
    loader.load('models/cave_wolf.glb', (gltf) => {
      this.wolfTemplate = gltf.scene;
      if (gltf.animations.length > 0) {
        this.wolfClip = gltf.animations[0];
      }
    });
  }

  public registerCritter(instance: CritterInstance): void {
    const group = new THREE.Group();
    group.position.set(instance.position.x, instance.position.y, instance.position.z);

    let bodyMesh: THREE.Object3D;
    let mixer: THREE.AnimationMixer | undefined;

    if (instance.config.type === CritterType.SLIME) {
      if (this.slimeTemplate) {
        bodyMesh = this.slimeTemplate.clone(true);
        if (this.slimeClip) {
          mixer = new THREE.AnimationMixer(bodyMesh);
          mixer.clipAction(this.slimeClip).play();
        }
      } else {
        const geo = new THREE.SphereGeometry(0.45, 12, 10);
        const mat = createCelMaterial({ color: 0x33cc66, roughness: 0.3 });
        bodyMesh = new THREE.Mesh(geo, mat);
        bodyMesh.position.y = 0.4;
      }
    } else if (instance.config.type === CritterType.CAVE_WOLF) {
      if (this.wolfTemplate) {
        bodyMesh = this.wolfTemplate.clone(true);
        if (this.wolfClip) {
          mixer = new THREE.AnimationMixer(bodyMesh);
          mixer.clipAction(this.wolfClip).play();
        }
      } else {
        const geo = new THREE.BoxGeometry(0.4, 0.4, 0.7);
        const mat = createCelMaterial({ color: 0x5a6070, roughness: 0.7 });
        bodyMesh = new THREE.Mesh(geo, mat);
        bodyMesh.position.y = 0.35;
      }
    } else {
      const geo = new THREE.SphereGeometry(0.25, 8, 6);
      const mat = createCelMaterial({ color: 0x00ffff, emissive: 0x00ffff, emissiveIntensity: 1.5 });
      bodyMesh = new THREE.Mesh(geo, mat);
      bodyMesh.position.y = 0.25;
    }

    bodyMesh.castShadow = true;
    bodyMesh.receiveShadow = true;
    group.add(bodyMesh);

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
      mixer,
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

      visual.group.position.set(critter.position.x, critter.position.y, critter.position.z);
      visual.group.rotation.y = critter.rotation;

      if (visual.iceCubeMesh) {
        visual.iceCubeMesh.visible = critter.state === AIState.FROZEN;
      }

      if (visual.mixer && critter.state !== AIState.FROZEN) {
        visual.mixer.update(deltaTime);
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
