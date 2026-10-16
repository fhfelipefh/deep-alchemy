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
    roughness: options.roughness ?? 0.6,
    metalness: 0.1,
    emissive: options.emissive ?? 0x000000,
    emissiveIntensity: options.emissiveIntensity ?? 0,
    wireframe: options.wireframe ?? false,
  });
}
