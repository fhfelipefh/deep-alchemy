import * as THREE from 'three';

export function createCelMaterial(options: {
  color: THREE.ColorRepresentation;
  roughness?: number;
  emissive?: THREE.ColorRepresentation;
  emissiveIntensity?: number;
  wireframe?: boolean;
}): THREE.MeshStandardMaterial {
  return new THREE.MeshStandardMaterial({
    color: options.color,
    roughness: options.roughness ?? 0.5,
    metalness: 0.1,
    emissive: options.emissive ?? 0x12151c,
    emissiveIntensity: options.emissiveIntensity ?? 0.35,
    wireframe: options.wireframe ?? false,
  });
}
