/**
 * Vetor 3D leve para cálculos de cinemática procedural e restrições geométricas.
 */
export interface Vec3 {
  x: number;
  y: number;
  z: number;
}

export class KinematicsSolver {
  /**
   * Aplica restrição de distância estrita entre âncora (anchor) e seguidor (follower).
   * Proteção matemática contra singularidade (distância < 10^-6).
   */
  public static applyDistanceConstraint(
    anchor: Vec3,
    follower: Vec3,
    targetDist: number
  ): Vec3 {
    const dx = follower.x - anchor.x;
    const dy = follower.y - anchor.y;
    const dz = follower.z - anchor.z;
    const distSq = dx * dx + dy * dy + dz * dz;

    if (distSq < 1e-12) {
      // Singularidade: afasta no eixo Z padrão
      return {
        x: anchor.x,
        y: anchor.y,
        z: anchor.z + targetDist,
      };
    }

    const dist = Math.sqrt(distSq);
    const scale = targetDist / dist;

    return {
      x: anchor.x + dx * scale,
      y: anchor.y + dy * scale,
      z: anchor.z + dz * scale,
    };
  }

  /**
   * Limita a rotação angular máxima no plano horizontal (XZ) entre dois segmentos adjacentes,
   * impedindo o colapso e auto-interseção das vértebras de cobras, lobos e caudas.
   */
  public static clampAngleDiff(
    baseAngle: number,
    targetAngle: number,
    maxTurnRad: number
  ): number {
    let diff = Math.atan2(Math.sin(targetAngle - baseAngle), Math.cos(targetAngle - baseAngle));
    if (diff > maxTurnRad) diff = maxTurnRad;
    else if (diff < -maxTurnRad) diff = -maxTurnRad;
    return baseAngle + diff;
  }

  /**
   * Resolve uma cadeia espinhal de N vértebras puxada pela cabeça (Head-driven).
   */
  public static solveSpineChain(
    joints: Vec3[],
    segmentLength: number,
    maxTurnRad: number = Math.PI / 4
  ): void {
    if (joints.length < 2) return;

    for (let i = 1; i < joints.length; i++) {
      // 1. Aplica restrição de distância física
      const constrained = this.applyDistanceConstraint(joints[i - 1], joints[i], segmentLength);

      // 2. Aplica restrição angular para suavidade orgânica se houver segmento anterior
      if (i >= 2) {
        const prevAngle = Math.atan2(
          joints[i - 1].z - joints[i - 2].z,
          joints[i - 1].x - joints[i - 2].x
        );
        const currAngle = Math.atan2(
          constrained.z - joints[i - 1].z,
          constrained.x - joints[i - 1].x
        );
        const clampedAngle = this.clampAngleDiff(prevAngle, currAngle, maxTurnRad);

        joints[i].x = joints[i - 1].x + Math.cos(clampedAngle) * segmentLength;
        joints[i].y = constrained.y;
        joints[i].z = joints[i - 1].z + Math.sin(clampedAngle) * segmentLength;
      } else {
        joints[i].x = constrained.x;
        joints[i].y = constrained.y;
        joints[i].z = constrained.z;
      }
    }
  }

  /**
   * Algoritmo FABRIK para membros de 2 ossos (Quadril/Ombro -> Joelho/Cotovelo -> Pé).
   */
  public static solveFabrik2Bone(
    root: Vec3,
    joints: Vec3[], // [ombro, joelho, pé]
    boneLen: number,
    target: Vec3,
    iterations: number = 3
  ): void {
    if (joints.length < 3) return;

    // Inicializa o efetuador final no alvo
    joints[2] = { ...target };

    for (let it = 0; it < iterations; it++) {
      // 1. Passe Forward (Pé -> Joelho -> Ombro)
      joints[1] = this.applyDistanceConstraint(joints[2], joints[1], boneLen);
      joints[0] = this.applyDistanceConstraint(joints[1], joints[0], boneLen);

      // 2. Passe Backward (Ombro ancorado no corpo -> Joelho -> Pé)
      joints[0] = { ...root };
      joints[1] = this.applyDistanceConstraint(joints[0], joints[1], boneLen);
      joints[2] = this.applyDistanceConstraint(joints[1], joints[2], boneLen);
    }
  }

  /**
   * Interpolação Bézier quadrática para o arco de elevação do pé em passos reativos.
   */
  public static evaluateStepArc(p0: Vec3, p1: Vec3, stepHeight: number, t: number): Vec3 {
    const clampedT = Math.max(0, Math.min(1, t));
    const pMid: Vec3 = {
      x: (p0.x + p1.x) * 0.5,
      y: Math.max(p0.y, p1.y) + stepHeight,
      z: (p0.z + p1.z) * 0.5,
    };

    const oneMinusT = 1 - clampedT;
    const a = oneMinusT * oneMinusT;
    const b = 2 * oneMinusT * clampedT;
    const c = clampedT * clampedT;

    return {
      x: a * p0.x + b * pMid.x + c * p1.x,
      y: a * p0.y + b * pMid.y + c * p1.y,
      z: a * p0.z + b * pMid.z + c * p1.z,
    };
  }

  /**
   * Deformação Squash & Stretch volumétrico para slimes com preservação de volume (Vx * Vy * Vz = 1).
   */
  public static computeSquashAndStretch(deformationY: number): { sx: number; sy: number; sz: number } {
    // sy varia entre 0.4 (esmagado no chão) e 1.6 (esticado no ar)
    const sy = Math.max(0.3, Math.min(2.0, 1.0 + deformationY));
    const horizontalScale = 1.0 / Math.sqrt(sy);
    return {
      sx: horizontalScale,
      sy,
      sz: horizontalScale,
    };
  }
}
