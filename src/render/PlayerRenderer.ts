import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { PlayerState } from '../entities/PlayerState';
import { createCelMaterial } from './CelShader';

export class PlayerRenderer {
  public group: THREE.Group;
  public headlamp: THREE.SpotLight;
  public headlampTarget: THREE.Object3D;
  public camera: THREE.PerspectiveCamera;

  private characterMesh: THREE.Object3D | null = null;
  private legs: THREE.Mesh[] = [];
  private walkTime: number = 0;

  constructor(camera: THREE.PerspectiveCamera) {
    this.camera = camera;
    this.group = new THREE.Group();

    // 1. Lanterna de Cabeça (Headlamp Spotlight cônico com sombras realistas)
    this.headlamp = new THREE.SpotLight(0xfff2d6, 3.8, 30.0, Math.PI / 4.5, 0.4, 1.8);
    this.headlamp.position.set(0, 1.65, 0.2); // Altura da testa (1,65m)
    this.headlamp.castShadow = true;
    this.headlamp.shadow.mapSize.width = 1024;
    this.headlamp.shadow.mapSize.height = 1024;
    this.headlamp.shadow.bias = -0.001;

    this.headlampTarget = new THREE.Object3D();
    this.headlampTarget.position.set(0, 1.4, 5.0); // Foco à frente
    this.headlamp.target = this.headlampTarget;

    this.group.add(this.headlamp);
    this.group.add(this.headlampTarget);

    // Carrega o modelo autoral exportado do Blender ou instancia malha procedural estilizada
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
        this.group.add(this.characterMesh);
      },
      undefined,
      (_err) => {
        // Fallback procedural idêntico caso executado em ambiente offline sem .glb
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

    // Tronco (altura base)
    const torsoGeo = new THREE.CylinderGeometry(0.24, 0.22, 0.7, 8);
    const torso = new THREE.Mesh(torsoGeo, matClothes);
    torso.position.y = 1.05;
    torso.castShadow = true;
    alchemist.add(torso);

    // Cabeça
    const headGeo = new THREE.SphereGeometry(0.18, 8, 8);
    const head = new THREE.Mesh(headGeo, matSkin);
    head.position.y = 1.55;
    head.castShadow = true;
    alchemist.add(head);

    // Chapéu de explorador Indiana Jones
    const brimGeo = new THREE.CylinderGeometry(0.36, 0.36, 0.03, 12);
    const brim = new THREE.Mesh(brimGeo, matHat);
    brim.position.y = 1.68;
    alchemist.add(brim);

    const topGeo = new THREE.CylinderGeometry(0.19, 0.2, 0.14, 10);
    const hatTop = new THREE.Mesh(topGeo, matHat);
    hatTop.position.y = 1.76;
    alchemist.add(hatTop);

    // Lanterna de cabeça
    const lampGeo = new THREE.BoxGeometry(0.08, 0.08, 0.08);
    const lamp = new THREE.Mesh(lampGeo, matGold);
    lamp.position.set(0, 1.66, 0.2);
    alchemist.add(lamp);

    // Mochila grande nas costas
    const packGeo = new THREE.BoxGeometry(0.32, 0.44, 0.22);
    const pack = new THREE.Mesh(packGeo, matPack);
    pack.position.set(0, 1.15, -0.22);
    pack.castShadow = true;
    alchemist.add(pack);

    // Pochete utilitária na cintura frontal
    const pouchGeo = new THREE.BoxGeometry(0.22, 0.12, 0.1);
    const pouch = new THREE.Mesh(pouchGeo, matHat);
    pouch.position.set(0.1, 0.75, 0.22);
    alchemist.add(pouch);

    // Pernas móveis para animação procedural de passos
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

  public update(player: PlayerState, deltaTime: number, moveDirection: THREE.Vector3): void {
    // Sincroniza posição no mundo
    this.group.position.set(player.position.x, player.position.y, player.position.z);

    // Rotação do personagem para a direção do movimento
    const isMoving = moveDirection.lengthSq() > 0.01;
    if (isMoving) {
      const targetRotation = Math.atan2(moveDirection.x, moveDirection.z);
      this.group.rotation.y = targetRotation;

      // Animação de passos das pernas
      const stepSpeed = player.isRunning ? 14 : 9;
      this.walkTime += deltaTime * stepSpeed;

      if (this.legs.length === 2) {
        this.legs[0].rotation.x = Math.sin(this.walkTime) * 0.45;
        this.legs[1].rotation.x = -Math.sin(this.walkTime) * 0.45;
      }
    } else {
      if (this.legs.length === 2) {
        this.legs[0].rotation.x = 0;
        this.legs[1].rotation.x = 0;
      }
    }

    // Câmera em terceira pessoa (Third-Person Over-the-Shoulder)
    // Câmera segue atrás do jogador a uma distância confortável (~3.8m atrás, ~2.2m de altura)
    const cameraOffset = new THREE.Vector3(0, 2.2, -3.8);
    cameraOffset.applyAxisAngle(new THREE.Vector3(0, 1, 0), this.group.rotation.y);

    const targetCameraPos = this.group.position.clone().add(cameraOffset);
    this.camera.position.lerp(targetCameraPos, 0.12);

    // Câmera olha para a altura do peito/cabeça do alquimista
    const lookTarget = this.group.position.clone().add(new THREE.Vector3(0, 1.5, 0));
    this.camera.lookAt(lookTarget);
  }
}
