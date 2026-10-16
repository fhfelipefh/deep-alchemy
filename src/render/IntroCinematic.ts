import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';

export class IntroCinematic {
  public isPlaying: boolean = false;
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private cinematicGroup: THREE.Group;
  private idolMesh: THREE.Object3D | null = null;
  private idolMixer: THREE.AnimationMixer | null = null;
  private timer: number = 0;
  private readonly duration: number = 3.6;
  private onCompleteCallback: (() => void) | null = null;

  constructor(scene: THREE.Scene, camera: THREE.PerspectiveCamera) {
    this.scene = scene;
    this.camera = camera;
    this.cinematicGroup = new THREE.Group();
    this.scene.add(this.cinematicGroup);

    const loader = new GLTFLoader();
    loader.load('models/golden_idol.glb', (gltf) => {
      this.idolMesh = gltf.scene;
      this.idolMesh.scale.set(2.0, 2.0, 2.0);
      this.cinematicGroup.add(this.idolMesh);

      if (gltf.animations.length > 0) {
        this.idolMixer = new THREE.AnimationMixer(this.idolMesh);
        this.idolMixer.clipAction(gltf.animations[0]).play();
      }
    });

    this.cinematicGroup.visible = false;
  }

  public play(playerPos: THREE.Vector3, onComplete: () => void): void {
    this.isPlaying = true;
    this.timer = 0;
    this.onCompleteCallback = onComplete;
    this.cinematicGroup.visible = true;

    if (this.idolMesh) {
      this.idolMesh.position.set(playerPos.x, playerPos.y + 1.2, playerPos.z + 2.5);
    }

    this.camera.position.set(playerPos.x, playerPos.y + 1.8, playerPos.z + 4.5);
    this.camera.lookAt(playerPos.x, playerPos.y + 1.2, playerPos.z + 2.5);
  }

  public update(deltaTime: number, playerPos: THREE.Vector3): void {
    if (!this.isPlaying) return;

    this.timer += deltaTime;
    if (this.idolMixer) {
      this.idolMixer.update(deltaTime);
    }

    const t = Math.min(1.0, this.timer / this.duration);

    if (t < 0.35) {
      this.camera.position.set(
        playerPos.x + Math.sin(t * 8) * 0.4,
        playerPos.y + 1.6,
        playerPos.z + 4.0 - t * 2.0
      );
      this.camera.lookAt(playerPos.x, playerPos.y + 1.2, playerPos.z + 2.0);
    } else if (t < 0.65) {
      const shake = (Math.random() - 0.5) * 0.08;
      this.camera.position.set(
        playerPos.x + shake,
        playerPos.y + 2.8 + shake,
        playerPos.z + 2.0
      );
      this.camera.lookAt(playerPos.x, playerPos.y, playerPos.z);
    } else {
      const settleT = (t - 0.65) / 0.35;
      const camTargetPos = new THREE.Vector3(playerPos.x, playerPos.y + 2.2, playerPos.z - 4.0);
      this.camera.position.lerp(camTargetPos, settleT);
      this.camera.lookAt(playerPos.x, playerPos.y + 1.4, playerPos.z);
    }

    if (this.timer >= this.duration) {
      this.isPlaying = false;
      this.cinematicGroup.visible = false;
      if (this.onCompleteCallback) {
        this.onCompleteCallback();
      }
    }
  }
}
