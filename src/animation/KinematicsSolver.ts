export interface Vec3 {
  x: number;
  y: number;
  z: number;
}

export class KinematicsSolver {
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

  public static solveSpineChain(
    joints: Vec3[],
    segmentLength: number,
    maxTurnRad: number = Math.PI / 4
  ): void {
    if (joints.length < 2) return;

    for (let i = 1; i < joints.length; i++) {
      const constrained = this.applyDistanceConstraint(joints[i - 1], joints[i], segmentLength);

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

  public static solveFabrik2Bone(
    root: Vec3,
    joints: Vec3[],
    boneLen: number,
    target: Vec3,
    iterations: number = 3
  ): void {
    if (joints.length < 3) return;

    joints[2] = { ...target };

    for (let it = 0; it < iterations; it++) {
      joints[1] = this.applyDistanceConstraint(joints[2], joints[1], boneLen);
      joints[0] = this.applyDistanceConstraint(joints[1], joints[0], boneLen);

      joints[0] = { ...root };
      joints[1] = this.applyDistanceConstraint(joints[0], joints[1], boneLen);
      joints[2] = this.applyDistanceConstraint(joints[1], joints[2], boneLen);
    }
  }

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

  public static computeSquashAndStretch(deformationY: number): { sx: number; sy: number; sz: number } {
    const sy = Math.max(0.3, Math.min(2.0, 1.0 + deformationY));
    const horizontalScale = 1.0 / Math.sqrt(sy);
    return {
      sx: horizontalScale,
      sy,
      sz: horizontalScale,
    };
  }
}
