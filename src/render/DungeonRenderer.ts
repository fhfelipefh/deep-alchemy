import * as THREE from 'three';
import { BiomeType, CanonicalPiece, ClutterItem, DungeonMap, DualQuad } from '../dungeon/DungeonTypes';
import { createCelMaterial } from './CelShader';

export class DungeonRenderer {
  public scene: THREE.Scene;
  public dungeonGroup: THREE.Group;
  public waterGroup: THREE.Group;
  public clutterGroup: THREE.Group;
  public steamParticles: THREE.Points[] = [];
  public animatedLights: { light: THREE.PointLight; baseIntensity: number; speed: number; offset: number }[] = [];

  private wallMaterials: Map<BiomeType, THREE.Material> = new Map();
  private floorMaterials: Map<BiomeType, THREE.Material> = new Map();
  private waterMaterial: THREE.MeshStandardMaterial;

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
      opacity: 0.65,
      roughness: 0.1,
      metalness: 0.2,
    });
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
      [BiomeType.BEACH]: 0x5a4838,
      [BiomeType.NORMAL_CAVE]: 0x2e2e32,
      [BiomeType.FLOODED]: 0x1c2830,
      [BiomeType.GEOTHERMAL]: 0x3d2018,
      [BiomeType.FUNGAL_GLOW]: 0x181c28,
    };

    const floorColors: Record<BiomeType, number> = {
      [BiomeType.BEACH]: 0x8a7356,
      [BiomeType.NORMAL_CAVE]: 0x3d3d42,
      [BiomeType.FLOODED]: 0x24323c,
      [BiomeType.GEOTHERMAL]: 0x48241a,
      [BiomeType.FUNGAL_GLOW]: 0x202434,
    };

    biomes.forEach((b) => {
      this.wallMaterials.set(b, createCelMaterial({ color: wallColors[b], roughness: 0.85 }));
      this.floorMaterials.set(b, createCelMaterial({ color: floorColors[b], roughness: 0.75 }));
    });
  }

  public buildDungeon(dungeon: DungeonMap): void {
    this.clear();
    const cellSize = dungeon.cellSize;

    // Constrói malhas baseadas nos DualQuads
    dungeon.dualQuads.forEach((quad) => {
      this.buildDualQuadMesh(quad, cellSize);
    });

    // Constrói Clutter (cogumelos luminosos, gêiseres, etc.)
    dungeon.clutter.forEach((item) => {
      this.buildClutterMesh(item);
    });

    // Constrói o feixe de luz da Saída (Exit Portal)
    this.buildExitBeacon(dungeon.exitPos, cellSize);
  }

  private buildDualQuadMesh(quad: DualQuad, size: number): void {
    const wx = (quad.x + 0.5) * size;
    const wz = (quad.z + 0.5) * size;
    const floorMat = this.floorMaterials.get(quad.biome)!;
    const wallMat = this.wallMaterials.get(quad.biome)!;

    if (quad.piece === CanonicalPiece.EMPTY) {
      // Rocha sólida (Parede contínua fechada)
      const wallGeo = new THREE.BoxGeometry(size, 4.0, size);
      const wallMesh = new THREE.Mesh(wallGeo, wallMat);
      wallMesh.position.set(wx, 2.0, wz);
      wallMesh.castShadow = true;
      wallMesh.receiveShadow = true;
      this.dungeonGroup.add(wallMesh);
      return;
    }

    // Piso caminhável
    const floorGeo = new THREE.PlaneGeometry(size, size);
    floorGeo.rotateX(-Math.PI / 2);
    const floorMesh = new THREE.Mesh(floorGeo, floorMat);
    floorMesh.position.set(wx, 0, wz);
    floorMesh.receiveShadow = true;
    this.dungeonGroup.add(floorMesh);

    // Teto rochoso abobadado
    const ceilingGeo = new THREE.PlaneGeometry(size, size);
    ceilingGeo.rotateX(Math.PI / 2);
    const ceilingMesh = new THREE.Mesh(ceilingGeo, wallMat);
    ceilingMesh.position.set(wx, 4.0, wz);
    this.dungeonGroup.add(ceilingMesh);

    // Se for peça de borda ou canto, ergue paredes correspondentes
    if (quad.piece === CanonicalPiece.STRAIGHT_EDGE) {
      // Parede em um dos lados
      const edgeGeo = new THREE.BoxGeometry(size, 4.0, 0.4);
      const edgeMesh = new THREE.Mesh(edgeGeo, wallMat);
      edgeMesh.position.set(wx, 2.0, wz - size * 0.4);
      edgeMesh.rotation.y = quad.rotation;
      edgeMesh.castShadow = true;
      edgeMesh.receiveShadow = true;
      this.dungeonGroup.add(edgeMesh);
    } else if (quad.piece === CanonicalPiece.CONVEX_CORNER || quad.piece === CanonicalPiece.CONCAVE_CORNER) {
      // Canto curvilíneo / coluna de quina
      const cornerGeo = new THREE.CylinderGeometry(size * 0.4, size * 0.4, 4.0, 8);
      const cornerMesh = new THREE.Mesh(cornerGeo, wallMat);
      cornerMesh.position.set(wx, 2.0, wz);
      cornerMesh.rotation.y = quad.rotation;
      cornerMesh.castShadow = true;
      cornerMesh.receiveShadow = true;
      this.dungeonGroup.add(cornerMesh);
    }

    // Se tiver água
    if (quad.waterHeight > 0) {
      const waterGeo = new THREE.PlaneGeometry(size, size);
      waterGeo.rotateX(-Math.PI / 2);
      const waterMesh = new THREE.Mesh(waterGeo, this.waterMaterial);
      waterMesh.position.set(wx, quad.waterHeight, wz);
      this.waterGroup.add(waterMesh);
    }
  }

  private buildClutterMesh(item: ClutterItem): void {
    if (item.type === 'MUSHROOM_CYAN' || item.type === 'MUSHROOM_BLUE') {
      const group = new THREE.Group();
      group.position.set(item.x, item.y, item.z);
      group.scale.setScalar(item.scale);

      // Caule
      const stemGeo = new THREE.CylinderGeometry(0.04, 0.06, 0.4, 6);
      const stemMat = createCelMaterial({ color: 0xcccccc });
      const stem = new THREE.Mesh(stemGeo, stemMat);
      stem.position.y = 0.2;
      group.add(stem);

      // Chapéu bioluminescente
      const capGeo = new THREE.SphereGeometry(0.18, 8, 6, 0, Math.PI * 2, 0, Math.PI * 0.5);
      const capColor = item.lightColor ?? 0x00ffff;
      const capMat = createCelMaterial({
        color: capColor,
        emissive: capColor,
        emissiveIntensity: 2.5,
      });
      const cap = new THREE.Mesh(capGeo, capMat);
      cap.position.y = 0.38;
      group.add(cap);

      // Luz pontual emitida pelo fungo
      const light = new THREE.PointLight(capColor, 1.2, 5.0, 2);
      light.position.set(0, 0.45, 0);
      group.add(light);

      this.animatedLights.push({
        light,
        baseIntensity: 1.2,
        speed: 2.0 + Math.random(),
        offset: Math.random() * Math.PI,
      });

      this.clutterGroup.add(group);
    } else if (item.type === 'GEYSER') {
      // Gêiser geotérmico
      const ringGeo = new THREE.TorusGeometry(0.5, 0.15, 6, 12);
      ringGeo.rotateX(Math.PI / 2);
      const ringMat = createCelMaterial({ color: 0x553322 });
      const ring = new THREE.Mesh(ringGeo, ringMat);
      ring.position.set(item.x, 0.1, item.z);
      this.clutterGroup.add(ring);

      // Luz quente alaranjada
      const warmLight = new THREE.PointLight(0xff6622, 1.5, 6.0, 2);
      warmLight.position.set(item.x, 0.5, item.z);
      this.clutterGroup.add(warmLight);
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

    // Coluna vertical de luz solar celestial atravessando o teto
    const beaconGeo = new THREE.CylinderGeometry(1.2, 1.8, 6.0, 16, 1, true);
    const beaconMat = new THREE.MeshBasicMaterial({
      color: 0xffea88,
      transparent: true,
      opacity: 0.45,
      side: THREE.DoubleSide,
    });
    const beacon = new THREE.Mesh(beaconGeo, beaconMat);
    beacon.position.set(wx, 3.0, wz);
    this.clutterGroup.add(beacon);

    // Luz brilhante da saída
    const exitLight = new THREE.PointLight(0xfff0aa, 3.5, 15.0, 2);
    exitLight.position.set(wx, 2.5, wz);
    this.clutterGroup.add(exitLight);
  }

  public update(_deltaTime: number, elapsedTime: number): void {
    // Pulsação suave das luzes dos fungos bioluminescentes
    this.animatedLights.forEach((item) => {
      item.light.intensity = item.baseIntensity + Math.sin(elapsedTime * item.speed + item.offset) * 0.35;
    });

    // Animação sutil de ondulação da água
    this.waterGroup.children.forEach((w, i) => {
      w.position.y = 0.25 + Math.sin(elapsedTime * 2.0 + i) * 0.03;
    });
  }

  public clear(): void {
    while (this.dungeonGroup.children.length > 0) {
      this.dungeonGroup.remove(this.dungeonGroup.children[0]);
    }
    while (this.waterGroup.children.length > 0) {
      this.waterGroup.remove(this.waterGroup.children[0]);
    }
    while (this.clutterGroup.children.length > 0) {
      this.clutterGroup.remove(this.clutterGroup.children[0]);
    }
    this.animatedLights = [];
  }
}
