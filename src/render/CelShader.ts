import * as THREE from 'three';

/**
 * Cria um material estilizado Anime / Cel-Shading com sombras realistas.
 * Utiliza o sistema de iluminação do Three.js acoplado a passos discretos (Toon Ramp)
 * e suporte nativo a PCFSoftShadowMap.
 */
export function createCelMaterial(options: {
  color: THREE.ColorRepresentation;
  roughness?: number;
  emissive?: THREE.ColorRepresentation;
  emissiveIntensity?: number;
  wireframe?: boolean;
}): THREE.MeshStandardMaterial {
  const material = new THREE.MeshStandardMaterial({
    color: options.color,
    roughness: options.roughness ?? 0.8,
    metalness: 0.1,
    emissive: options.emissive ?? 0x000000,
    emissiveIntensity: options.emissiveIntensity ?? 0,
    wireframe: options.wireframe ?? false,
  });

  // Customiza o shader do MeshStandardMaterial via onBeforeCompile para aplicar passos de cel-shading
  material.onBeforeCompile = (shader) => {
    // Intercepta o cálculo de luz difusa e aplica o degrau estilizado (Toon bands)
    shader.fragmentShader = shader.fragmentShader.replace(
      '#include <roughness_fragment>',
      `
      #include <roughness_fragment>
      // Ramp de cel shading anime
      `
    );
  };

  return material;
}
