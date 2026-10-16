import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { PlayerState } from '../entities/PlayerState';
import { createCelMaterial } from './CelShader';

export class PlayerRenderer {
  public group: THREE.Group;
  public headlamp: THREE.SpotLight;
  public radialLantern: THREE.PointLight;
  public headlampTarget: THREE.Object3D;
  public camera: THREE.PerspectiveCamera;
  public mixer: THREE.AnimationMixer | null = null;

  private characterMesh: THREE.Object3D | null = null;
  private legs: THREE.Mesh[] = [];
  private walkTime: number = 0;
  private walkAction: THREE.AnimationAction | null = null;

  constructor(camera: THREE.PerspectiveCamera, scene: THREE.Scene) {
    this.camera = camera;
    this.group = new THREE.Group();

    this.headlamp = new THREE.SpotLight(0xfff8ee, 7.5, 50.0, Math.PI / 2.6, 0.85, 1.1);
    this.headlamp.position.set(0, 1.65, 0.2);
    this.headlamp.castShadow = true;
    this.headlamp.shadow.mapSize.width = 1024;
    this.headlamp.shadow.mapSize.height = 1024;
    this.headlamp.shadow.bias = -0.001;

    this.radialLantern = new THREE.PointLight(0xfff0dc, 5.0, 24.0, 1.1);
    this.radialLantern.position.set(0, 1.45, 0.1);
    this.group.add(this.radialLantern);

    this.headlampTarget = new THREE.Object3D();
    scene.add(this.headlampTarget);
    this.headlamp.target = this.headlampTarget;
    this.group.add(this.headlamp);

    this.loadAlchemistModel();
  }

  private loadAlchemistModel(): void {
    const loader = new GLTFLoader();
    loader.load(
      'models/alchemist.glb',
      (gltf) => {
        this.characterMesh = gltf.scene;
        this.characterMesh.traverse((child) => {
          if ((child as THREE.Mesh).isMesh) {
            child.castShadow = true;
            child.receiveShadow = true;
          }
        });

        if (gltf.animations.length > 0) {
          this.mixer = new THREE.AnimationMixer(this.characterMesh);
          this.walkAction = this.mixer.clipAction(gltf.animations[0]);
          this.walkAction.play();
        }

        this.group.add(this.characterMesh);
      },
      undefined,
      () => {
        this.buildProceduralAlchemist();
      }
    );
  }

  private buildProceduralAlchemist(): void {
    const alchemist = new THREE.Group();

    const matClothes = createCelMaterial({ color: 0x8b6b43 });
    const matHat = createCelMaterial({ color: 0x3d2514 });
    const matPack = createCelMaterial({ color: 0x384a28 });
    const matSkin = createCelMaterial({ color: 0xf5c29b });
    const matGold = createCelMaterial({ color: 0xe6b800 });

    const torsoGeo = new THREE.CylinderGeometry(0.24, 0.22, 0.7, 8);
    const torso = new THREE.Mesh(torsoGeo, matClothes);
    torso.position.y = 1.05;
    torso.castShadow = true;
    alchemist.add(torso);

    const headGeo = new THREE.SphereGeometry(0.18, 8, 8);
    const head = new THREE.Mesh(headGeo, matSkin);
    head.position.y = 1.55;
    head.castShadow = true;
    alchemist.add(head);

    const brimGeo = new THREE.CylinderGeometry(0.36, 0.36, 0.03, 12);
    const brim = new THREE.Mesh(brimGeo, matHat);
    brim.position.y = 1.68;
    alchemist.add(brim);

    const topGeo = new THREE.CylinderGeometry(0.19, 0.2, 0.14, 10);
    const hatTop = new THREE.Mesh(topGeo, matHat);
    hatTop.position.y = 1.76;
    alchemist.add(hatTop);

    const lampGeo = new THREE.BoxGeometry(0.08, 0.08, 0.08);
    const lamp = new THREE.Mesh(lampGeo, matGold);
    lamp.position.set(0, 1.66, 0.2);
    alchemist.add(lamp);

    const packGeo = new THREE.BoxGeometry(0.32, 0.44, 0.22);
    const pack = new THREE.Mesh(packGeo, matPack);
    pack.position.set(0, 1.15, -0.22);
    pack.castShadow = true;
    alchemist.add(pack);

    const pouchGeo = new THREE.BoxGeometry(0.22, 0.12, 0.1);
    const pouch = new THREE.Mesh(pouchGeo, matHat);
    pouch.position.set(0.1, 0.75, 0.22);
    alchemist.add(pouch);

    const legGeo = new THREE.CylinderGeometry(0.09, 0.08, 0.7, 6);
    const leftLeg = new THREE.Mesh(legGeo, matClothes);
    leftLeg.position.set(-0.14, 0.35, 0);
    leftLeg.castShadow = true;
    alchemist.add(leftLeg);

    const rightLeg = new THREE.Mesh(legGeo, matClothes);
    rightLeg.position.set(0.14, 0.35, 0);
    rightLeg.castShadow = true;
    alchemist.add(rightLeg);

    this.legs = [leftLeg, rightLeg];
    this.characterMesh = alchemist;
    this.group.add(alchemist);
  }

  public snapCamera(player: PlayerState, initialAngle: number = 0): void {
    this.group.rotation.y = initialAngle;
    this.group.position.set(player.position.x, player.position.y, player.position.z);

    const forward = new THREE.Vector3(0, 0, 1).applyAxisAngle(new THREE.Vector3(0, 1, 0), this.group.rotation.y);
    this.headlampTarget.position.copy(this.group.position).add(forward.clone().multiplyScalar(6));

    const offset = new THREE.Vector3(0, 2.2, -3.2);
    offset.applyAxisAngle(new THREE.Vector3(0, 1, 0), this.group.rotation.y);

    this.camera.position.copy(this.group.position).add(offset);
    const lookTarget = this.group.position.clone().add(forward.clone().multiplyScalar(3.5)).add(new THREE.Vector3(0, 1.4, 0));
    this.camera.lookAt(lookTarget);
  }

  public update(player: PlayerState, deltaTime: number, moveDirection: THREE.Vector3): void {
    this.group.position.set(player.position.x, player.position.y, player.position.z);

    const isMoving = moveDirection.lengthSq() > 0.01;
    if (isMoving) {
      const targetRotation = Math.atan2(moveDirection.x, moveDirection.z);
      this.group.rotation.y = targetRotation;

      if (this.mixer) {
        this.mixer.timeScale = player.isRunning ? 1.6 : 1.0;
        this.mixer.update(deltaTime);
      } else {
        const stepSpeed = player.isRunning ? 14 : 9;
        this.walkTime += deltaTime * stepSpeed;
        if (this.legs.length === 2) {
          this.legs[0].rotation.x = Math.sin(this.walkTime) * 0.45;
          this.legs[1].rotation.x = -Math.sin(this.walkTime) * 0.45;
        }
      }
    } else {
      if (this.legs.length === 2) {
        this.legs[0].rotation.x = 0;
        this.legs[1].rotation.x = 0;
      }
    }

    const forward = new THREE.Vector3(0, 0, 1).applyAxisAngle(new THREE.Vector3(0, 1, 0), this.group.rotation.y);
    this.headlampTarget.position.copy(this.group.position).add(forward.clone().multiplyScalar(6));

    const cameraOffset = new THREE.Vector3(0, 2.2, -3.2);
    cameraOffset.applyAxisAngle(new THREE.Vector3(0, 1, 0), this.group.rotation.y);

    const targetCameraPos = this.group.position.clone().add(cameraOffset);
    this.camera.position.lerp(targetCameraPos, 0.2);

    const lookTarget = this.group.position.clone().add(forward.clone().multiplyScalar(3.5)).add(new THREE.Vector3(0, 1.4, 0));
    this.camera.lookAt(lookTarget);
  }
}
