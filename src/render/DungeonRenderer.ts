import * as THREE from 'three';
import { mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { BiomeType, CanonicalPiece, ClutterItem, DungeonMap } from '../dungeon/DungeonTypes';
import { createCelMaterial } from './CelShader';

export class DungeonRenderer {
  public scene: THREE.Scene;
  public dungeonGroup: THREE.Group;
  public waterGroup: THREE.Group;
  public clutterGroup: THREE.Group;
  public animatedLights: { light: THREE.PointLight; baseIntensity: number; speed: number; offset: number }[] = [];
  public mixers: THREE.AnimationMixer[] = [];

  private wallMaterials: Map<BiomeType, THREE.Material> = new Map();
  private floorMaterials: Map<BiomeType, THREE.Material> = new Map();
  private waterMaterial: THREE.MeshStandardMaterial;
  private mushroomTemplate: THREE.Object3D | null = null;
  private mushroomClip: THREE.AnimationClip | null = null;
  private idolTemplate: THREE.Object3D | null = null;
  private idolClip: THREE.AnimationClip | null = null;
  private waterfallMesh: THREE.Mesh | null = null;
  private waterfallFoam: THREE.Mesh | null = null;

  constructor(scene: THREE.Scene) {
    this.scene = scene;
    this.dungeonGroup = new THREE.Group();
    this.waterGroup = new THREE.Group();
    this.clutterGroup = new THREE.Group();

    this.scene.add(this.dungeonGroup);
    this.scene.add(this.waterGroup);
    this.scene.add(this.clutterGroup);

    this.initMaterials();
    this.waterMaterial = new THREE.MeshStandardMaterial({
      color: 0x2288bb,
      transparent: true,
      opacity: 0.75,
      roughness: 0.1,
      metalness: 0.25,
    });

    this.loadTemplates();
  }

  private loadTemplates(): void {
    const loader = new GLTFLoader();
    try {
      loader.load('models/glowing_mushroom.glb', (gltf) => {
        this.mushroomTemplate = gltf.scene;
        if (gltf.animations.length > 0) {
          this.mushroomClip = gltf.animations[0];
        }
      });
    } catch {}

    try {
      loader.load('models/golden_idol.glb', (gltf) => {
        this.idolTemplate = gltf.scene;
        if (gltf.animations.length > 0) {
          this.idolClip = gltf.animations[0];
        }
      });
    } catch {}
  }

  private initMaterials(): void {
    const biomes = [
      BiomeType.BEACH,
      BiomeType.NORMAL_CAVE,
      BiomeType.FLOODED,
      BiomeType.GEOTHERMAL,
      BiomeType.FUNGAL_GLOW,
    ];

    const wallColors: Record<BiomeType, number> = {
      [BiomeType.BEACH]: 0x8a6e54,
      [BiomeType.NORMAL_CAVE]: 0x5a5e6b,
      [BiomeType.FLOODED]: 0x4f6b7c,
      [BiomeType.GEOTHERMAL]: 0x7d493a,
      [BiomeType.FUNGAL_GLOW]: 0x515b7c,
    };

    const floorColors: Record<BiomeType, number> = {
      [BiomeType.BEACH]: 0xdfbe88,
      [BiomeType.NORMAL_CAVE]: 0x6e727e,
      [BiomeType.FLOODED]: 0x577182,
      [BiomeType.GEOTHERMAL]: 0x945241,
      [BiomeType.FUNGAL_GLOW]: 0x586282,
    };

    biomes.forEach((b) => {
      this.wallMaterials.set(b, createCelMaterial({ color: wallColors[b], roughness: 0.7 }));
      this.floorMaterials.set(b, createCelMaterial({ color: floorColors[b], roughness: 0.6 }));
    });
  }

  public buildDungeon(dungeon: DungeonMap): void {
    this.clear();
    const cellSize = dungeon.cellSize;

    const floorBuckets = new Map<BiomeType, THREE.BufferGeometry[]>();
    const wallBuckets = new Map<BiomeType, THREE.BufferGeometry[]>();
    const ceilingGeos: THREE.BufferGeometry[] = [];
    const waterGeos: THREE.BufferGeometry[] = [];

    const biomes = [
      BiomeType.BEACH,
      BiomeType.NORMAL_CAVE,
      BiomeType.FLOODED,
      BiomeType.GEOTHERMAL,
      BiomeType.FUNGAL_GLOW,
    ];

    biomes.forEach((b) => {
      floorBuckets.set(b, []);
      wallBuckets.set(b, []);
    });

    const baseWallBox = new THREE.BoxGeometry(cellSize, 4.0, cellSize);
    const baseFloorPlane = new THREE.PlaneGeometry(cellSize, cellSize);
    baseFloorPlane.rotateX(-Math.PI / 2);
    const baseCeilingPlane = new THREE.PlaneGeometry(cellSize, cellSize);
    baseCeilingPlane.rotateX(Math.PI / 2);
    const baseEdgeBox = new THREE.BoxGeometry(cellSize, 4.0, 0.4);
    const baseCornerCyl = new THREE.CylinderGeometry(cellSize * 0.4, cellSize * 0.4, 4.0, 8);

    dungeon.dualQuads.forEach((quad) => {
      const wx = (quad.x + 0.5) * cellSize;
      const wz = (quad.z + 0.5) * cellSize;

      if (quad.piece === CanonicalPiece.EMPTY) {
        const wallGeo = baseWallBox.clone();
        wallGeo.translate(wx, 2.0, wz);
        wallBuckets.get(quad.biome)?.push(wallGeo);
        return;
      }

      const floorGeo = baseFloorPlane.clone();
      floorGeo.translate(wx, 0.0, wz);
      floorBuckets.get(quad.biome)?.push(floorGeo);

      const ceilingGeo = baseCeilingPlane.clone();
      ceilingGeo.translate(wx, 4.0, wz);
      ceilingGeos.push(ceilingGeo);

      if (quad.piece === CanonicalPiece.STRAIGHT_EDGE) {
        const edgeGeo = baseEdgeBox.clone();
        edgeGeo.translate(0, 0, -cellSize * 0.4);
        edgeGeo.rotateY(quad.rotation);
        edgeGeo.translate(wx, 2.0, wz);
        wallBuckets.get(quad.biome)?.push(edgeGeo);
      } else if (
        quad.piece === CanonicalPiece.CONVEX_CORNER ||
        quad.piece === CanonicalPiece.CONCAVE_CORNER
      ) {
        const cornerGeo = baseCornerCyl.clone();
        cornerGeo.rotateY(quad.rotation);
        cornerGeo.translate(wx, 2.0, wz);
        wallBuckets.get(quad.biome)?.push(cornerGeo);
      }

      if (quad.waterHeight > 0) {
        const waterGeo = baseFloorPlane.clone();
        waterGeo.translate(wx, quad.waterHeight, wz);
        waterGeos.push(waterGeo);
      }
    });

    baseWallBox.dispose();
    baseFloorPlane.dispose();
    baseCeilingPlane.dispose();
    baseEdgeBox.dispose();
    baseCornerCyl.dispose();

    biomes.forEach((b) => {
      const floors = floorBuckets.get(b);
      if (floors && floors.length > 0) {
        const mergedFloor = mergeGeometries(floors, false);
        const mesh = new THREE.Mesh(mergedFloor, this.floorMaterials.get(b)!);
        mesh.receiveShadow = true;
        this.dungeonGroup.add(mesh);
      }

      const walls = wallBuckets.get(b);
      if (walls && walls.length > 0) {
        const mergedWall = mergeGeometries(walls, false);
        const mesh = new THREE.Mesh(mergedWall, this.wallMaterials.get(b)!);
        mesh.receiveShadow = true;
        this.dungeonGroup.add(mesh);
      }
    });

    if (ceilingGeos.length > 0) {
      const mergedCeiling = mergeGeometries(ceilingGeos, false);
      const mesh = new THREE.Mesh(mergedCeiling, this.wallMaterials.get(BiomeType.NORMAL_CAVE)!);
      this.dungeonGroup.add(mesh);
    }

    if (waterGeos.length > 0) {
      const mergedWater = mergeGeometries(waterGeos, false);
      const mesh = new THREE.Mesh(mergedWater, this.waterMaterial);
      this.waterGroup.add(mesh);
    }

    dungeon.clutter.forEach((item) => {
      this.buildClutterMesh(item);
    });

    this.buildWaterfall(dungeon.startPos, cellSize);
    this.buildExitBeacon(dungeon.exitPos, cellSize);
  }

  private buildWaterfall(startPos: { x: number; z: number }, cellSize: number): void {
    const wx = (startPos.x + 0.5) * cellSize;
    const wz = (startPos.z + 5.5) * cellSize;

    const waterfallGeo = new THREE.PlaneGeometry(cellSize * 5.0, 7.5, 12, 16);
    const waterfallMat = new THREE.MeshStandardMaterial({
      color: 0x55ccff,
      emissive: 0x1177aa,
      emissiveIntensity: 0.9,
      transparent: true,
      opacity: 0.85,
      roughness: 0.08,
      metalness: 0.25,
      side: THREE.DoubleSide,
    });
    this.waterfallMesh = new THREE.Mesh(waterfallGeo, waterfallMat);
    this.waterfallMesh.position.set(wx, 3.6, wz - 0.2);
    this.waterGroup.add(this.waterfallMesh);

    const foamGeo = new THREE.RingGeometry(0.8, cellSize * 2.8, 20);
    foamGeo.rotateX(-Math.PI / 2);
    const foamMat = new THREE.MeshStandardMaterial({
      color: 0xffffff,
      emissive: 0x88eeff,
      emissiveIntensity: 1.4,
      transparent: true,
      opacity: 0.75,
      roughness: 0.2,
      side: THREE.DoubleSide,
    });
    this.waterfallFoam = new THREE.Mesh(foamGeo, foamMat);
    this.waterfallFoam.position.set(wx, 0.32, wz - 1.2);
    this.waterGroup.add(this.waterfallFoam);

    const sprayLight = new THREE.PointLight(0x77ddff, 2.5, 16.0, 1.2);
    sprayLight.position.set(wx, 2.0, wz - 1.5);
    this.clutterGroup.add(sprayLight);
  }

  private buildClutterMesh(item: ClutterItem): void {
    if (item.type === 'MUSHROOM_CYAN' || item.type === 'MUSHROOM_BLUE') {
      const group = new THREE.Group();
      group.position.set(item.x, item.y, item.z);
      group.scale.setScalar(item.scale);

      if (this.mushroomTemplate) {
        const shroom = this.mushroomTemplate.clone(true);
        group.add(shroom);
        if (this.mushroomClip) {
          const mixer = new THREE.AnimationMixer(shroom);
          mixer.clipAction(this.mushroomClip).play();
          this.mixers.push(mixer);
        }
      } else {
        const stemGeo = new THREE.CylinderGeometry(0.04, 0.06, 0.4, 6);
        const stemMat = createCelMaterial({ color: 0xcccccc });
        const stem = new THREE.Mesh(stemGeo, stemMat);
        stem.position.y = 0.2;
        group.add(stem);

        const capGeo = new THREE.SphereGeometry(0.18, 8, 6, 0, Math.PI * 2, 0, Math.PI * 0.5);
        const capColor = item.lightColor ?? 0x00ffff;
        const capMat = createCelMaterial({
          color: capColor,
          emissive: capColor,
          emissiveIntensity: 2.6,
        });
        const cap = new THREE.Mesh(capGeo, capMat);
        cap.position.y = 0.38;
        group.add(cap);
      }

      this.clutterGroup.add(group);
    } else if (item.type === 'GEYSER') {
      const ringGeo = new THREE.TorusGeometry(0.5, 0.15, 6, 12);
      ringGeo.rotateX(Math.PI / 2);
      const ringMat = createCelMaterial({
        color: 0x773318,
        emissive: 0xff4400,
        emissiveIntensity: 1.5,
      });
      const ring = new THREE.Mesh(ringGeo, ringMat);
      ring.position.set(item.x, 0.1, item.z);
      this.clutterGroup.add(ring);
    } else if (item.type === 'ROCK_PILE') {
      const rockGeo = new THREE.DodecahedronGeometry(0.3 * item.scale);
      const rockMat = createCelMaterial({ color: 0x44444a });
      const rock = new THREE.Mesh(rockGeo, rockMat);
      rock.position.set(item.x, 0.15, item.z);
      rock.rotation.set(item.rotation, item.rotation * 0.5, 0);
      this.clutterGroup.add(rock);
    }
  }

  private buildExitBeacon(exitPos: { x: number; z: number }, cellSize: number): void {
    const wx = (exitPos.x + 0.5) * cellSize;
    const wz = (exitPos.z + 0.5) * cellSize;

    if (this.idolTemplate) {
      const idol = this.idolTemplate.clone(true);
      idol.position.set(wx, 1.2, wz);
      idol.scale.set(1.5, 1.5, 1.5);
      this.clutterGroup.add(idol);
      if (this.idolClip) {
        const mixer = new THREE.AnimationMixer(idol);
        mixer.clipAction(this.idolClip).play();
        this.mixers.push(mixer);
      }
    }

    const beaconGeo = new THREE.CylinderGeometry(1.2, 1.8, 6.0, 16, 1, true);
    const beaconMat = new THREE.MeshBasicMaterial({
      color: 0xffea88,
      transparent: true,
      opacity: 0.5,
      side: THREE.DoubleSide,
    });
    const beacon = new THREE.Mesh(beaconGeo, beaconMat);
    beacon.position.set(wx, 3.0, wz);
    this.clutterGroup.add(beacon);

    const exitLight = new THREE.PointLight(0xfff0aa, 5.0, 20.0, 1.5);
    exitLight.position.set(wx, 2.5, wz);
    this.clutterGroup.add(exitLight);
  }

  public update(deltaTime: number, elapsedTime: number): void {
    this.animatedLights.forEach((item) => {
      item.light.intensity = item.baseIntensity + Math.sin(elapsedTime * item.speed + item.offset) * 0.4;
    });

    this.waterGroup.children.forEach((w, i) => {
      if (w !== this.waterfallMesh && w !== this.waterfallFoam) {
        w.position.y = 0.28 + Math.sin(elapsedTime * 2.0 + i) * 0.03;
      }
    });

    if (this.waterfallMesh) {
      const posAttr = this.waterfallMesh.geometry.attributes.position as THREE.BufferAttribute;
      for (let i = 0; i < posAttr.count; i++) {
        const vy = posAttr.getY(i);
        const vx = posAttr.getX(i);
        const wave = Math.sin(vy * 3.0 - elapsedTime * 8.0) * 0.12 + Math.cos(vx * 2.5 + elapsedTime * 5.0) * 0.05;
        posAttr.setZ(i, wave);
      }
      posAttr.needsUpdate = true;
    }

    if (this.waterfallFoam) {
      const foamScale = 1.0 + Math.sin(elapsedTime * 6.0) * 0.08;
      this.waterfallFoam.scale.set(foamScale, 1.0, foamScale);
    }

    this.mixers.forEach((m) => m.update(deltaTime));
  }

  public clear(): void {
    while (this.dungeonGroup.children.length > 0) {
      const child = this.dungeonGroup.children[0] as THREE.Mesh;
      if (child.geometry) child.geometry.dispose();
      this.dungeonGroup.remove(child);
    }
    while (this.waterGroup.children.length > 0) {
      const child = this.waterGroup.children[0] as THREE.Mesh;
      if (child.geometry) child.geometry.dispose();
      this.waterGroup.remove(child);
    }
    while (this.clutterGroup.children.length > 0) {
      this.clutterGroup.remove(this.clutterGroup.children[0]);
    }
    this.waterfallMesh = null;
    this.waterfallFoam = null;
    this.animatedLights = [];
    this.mixers = [];
  }
}
