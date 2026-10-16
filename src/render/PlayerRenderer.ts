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
  public characterMesh: THREE.Object3D | null = null;

  public yaw: number = 0;
  public pitch: number = 0;
  public eyeHeight: number = 1.68;

  private walkTime: number = 0;
  private walkAction: THREE.AnimationAction | null = null;

  constructor(camera: THREE.PerspectiveCamera, scene: THREE.Scene) {
    this.camera = camera;
    this.group = new THREE.Group();

    this.headlamp = new THREE.SpotLight(0xfff8ee, 7.5, 50.0, Math.PI / 2.6, 0.85, 1.1);
    this.headlamp.position.set(0, this.eyeHeight, 0);
    this.headlamp.castShadow = true;
    this.headlamp.shadow.mapSize.width = 1024;
    this.headlamp.shadow.mapSize.height = 1024;
    this.headlamp.shadow.bias = -0.001;

    this.radialLantern = new THREE.PointLight(0xfff0dc, 5.0, 24.0, 1.1);
    this.radialLantern.position.set(0, 1.45, 0);
    this.group.add(this.radialLantern);

    this.headlampTarget = new THREE.Object3D();
    scene.add(this.headlampTarget);
    this.headlamp.target = this.headlampTarget;
    this.group.add(this.headlamp);

    this.loadAlchemistModel();
  }

  private loadAlchemistModel(): void {
    try {
      const loader = new GLTFLoader();
      loader.load(
        'models/alchemist.glb',
        (gltf) => {
          this.characterMesh = gltf.scene;
          this.characterMesh.visible = false;
          this.characterMesh.traverse((child) => {
            if ((child as THREE.Mesh).isMesh) {
              child.castShadow = false;
              child.receiveShadow = false;
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
    } catch {
      this.buildProceduralAlchemist();
    }
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
    alchemist.add(torso);

    const headGeo = new THREE.SphereGeometry(0.18, 8, 8);
    const head = new THREE.Mesh(headGeo, matSkin);
    head.position.y = 1.55;
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
    alchemist.add(pack);

    const pouchGeo = new THREE.BoxGeometry(0.22, 0.12, 0.1);
    const pouch = new THREE.Mesh(pouchGeo, matHat);
    pouch.position.set(0.1, 0.75, 0.22);
    alchemist.add(pouch);

    const legGeo = new THREE.CylinderGeometry(0.09, 0.08, 0.7, 6);
    const leftLeg = new THREE.Mesh(legGeo, matClothes);
    leftLeg.position.set(-0.14, 0.35, 0);
    alchemist.add(leftLeg);

    const rightLeg = new THREE.Mesh(legGeo, matClothes);
    rightLeg.position.set(0.14, 0.35, 0);
    alchemist.add(rightLeg);

    this.characterMesh = alchemist;
    this.characterMesh.visible = false;
    this.group.add(alchemist);
  }

  public snapCamera(player: PlayerState, initialAngle: number = 0): void {
    this.yaw = initialAngle;
    this.pitch = 0;
    this.group.position.set(player.position.x, player.position.y, player.position.z);
    this.updateCameraTransform(player, 0);
  }

  public rotateView(deltaYaw: number, deltaPitch: number): void {
    this.yaw += deltaYaw;
    this.pitch = Math.max(-Math.PI / 2.6, Math.min(Math.PI / 2.6, this.pitch + deltaPitch));
  }

  public getForwardVector(): THREE.Vector3 {
    const cosPitch = Math.cos(this.pitch);
    return new THREE.Vector3(
      Math.sin(this.yaw) * cosPitch,
      Math.sin(this.pitch),
      Math.cos(this.yaw) * cosPitch
    ).normalize();
  }

  public getHorizontalForward(): THREE.Vector3 {
    return new THREE.Vector3(Math.sin(this.yaw), 0, Math.cos(this.yaw)).normalize();
  }

  public getHorizontalRight(): THREE.Vector3 {
    const fwd = this.getHorizontalForward();
    const up = new THREE.Vector3(0, 1, 0);
    return new THREE.Vector3().crossVectors(fwd, up).normalize();
  }

  public update(player: PlayerState, deltaTime: number, isMoving: boolean): void {
    this.group.position.set(player.position.x, player.position.y, player.position.z);

    let headBobY = 0;
    if (isMoving) {
      const stepSpeed = player.isRunning ? 13 : 9;
      this.walkTime += deltaTime * stepSpeed;
      headBobY = Math.sin(this.walkTime * 2) * 0.035;

      if (this.mixer) {
        this.mixer.timeScale = player.isRunning ? 1.6 : 1.0;
        this.mixer.update(deltaTime);
      }
    } else {
      this.walkTime = 0;
    }

    this.updateCameraTransform(player, headBobY);
  }

  private updateCameraTransform(player: PlayerState, headBobY: number): void {
    const eyeY = player.position.y + this.eyeHeight + headBobY;
    this.camera.position.set(player.position.x, eyeY, player.position.z);

    const cosPitch = Math.cos(this.pitch);
    const dirX = Math.sin(this.yaw) * cosPitch;
    const dirY = Math.sin(this.pitch);
    const dirZ = Math.cos(this.yaw) * cosPitch;

    const targetX = this.camera.position.x + dirX * 10;
    const targetY = this.camera.position.y + dirY * 10;
    const targetZ = this.camera.position.z + dirZ * 10;

    this.camera.lookAt(targetX, targetY, targetZ);

    this.radialLantern.position.set(player.position.x, player.position.y + 1.4, player.position.z);
    this.headlamp.position.copy(this.camera.position);
    this.headlampTarget.position.set(
      this.camera.position.x + dirX * 15,
      this.camera.position.y + dirY * 15,
      this.camera.position.z + dirZ * 15
    );
  }
}
