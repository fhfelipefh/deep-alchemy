use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

#[derive(Component, Debug, Clone)]
pub struct DandelionPlant {
    pub sway_phase: f32,
    pub base_yaw: f32,
    pub base_rotation: Quat,
    pub base_translation: Vec3,
    pub pivot: Vec3,
}

#[derive(Resource)]
pub struct DandelionMaterials {
    pub stem_green: Handle<StandardMaterial>,
    pub leaf_green: Handle<StandardMaterial>,
    pub bract_green: Handle<StandardMaterial>,
    pub receptacle_green: Handle<StandardMaterial>,
    pub floret_yellow: Handle<StandardMaterial>,
    pub disk_orange: Handle<StandardMaterial>,
    pub pappus_white: Handle<StandardMaterial>,
}

pub fn setup_dandelion_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mats = DandelionMaterials {
        stem_green: materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.58, 0.20),
            perceptual_roughness: 0.78,
            ..default()
        }),
        leaf_green: materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.50, 0.16),
            perceptual_roughness: 0.80,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        bract_green: materials.add(StandardMaterial {
            base_color: Color::srgb(0.14, 0.46, 0.14),
            perceptual_roughness: 0.82,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        receptacle_green: materials.add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.54, 0.18),
            perceptual_roughness: 0.75,
            ..default()
        }),
        floret_yellow: materials.add(StandardMaterial {
            base_color: Color::srgb(0.98, 0.88, 0.08),
            emissive: LinearRgba::new(0.38, 0.30, 0.0, 1.0),
            perceptual_roughness: 0.50,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        disk_orange: materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.55, 0.04),
            emissive: LinearRgba::new(0.28, 0.14, 0.0, 1.0),
            perceptual_roughness: 0.60,
            ..default()
        }),
        pappus_white: materials.add(StandardMaterial {
            base_color: Color::srgba(0.95, 0.97, 1.0, 0.82),
            perceptual_roughness: 0.90,
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
    };
    commands.insert_resource(mats);
}

pub fn spawn_dandelion_field(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &DandelionMaterials,
    positions: &[(f32, f32, u32)],
) {
    for &(x, z, seed) in positions {
        spawn_dandelion(commands, meshes, mats, Vec3::new(x, 0.0, z), seed);
    }
}

pub fn spawn_dandelion(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &DandelionMaterials,
    base_pos: Vec3,
    seed: u32,
) {

    let mut s = seed as u64 * 6364136223846793005u64 + 1442695040888963407u64;
    let mut rng = move || -> f32 {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((s >> 32) as u32 as f32) / (u32::MAX as f32)
    };

    let scale      = 0.72 + rng() * 0.58;
    let stem_h     = 0.36 + rng() * 0.26;
    let lean_x     = (rng() - 0.5) * 0.20;
    let lean_z     = (rng() - 0.5) * 0.20;
    let sway_phase = rng() * std::f32::consts::TAU;
    let base_yaw   = rng() * std::f32::consts::TAU;
    let is_bloom   = rng() < 0.62;

    let stem_tip_local = Vec3::new(lean_x * stem_h, stem_h, lean_z * stem_h);

    let root_tf = Transform::from_translation(base_pos)
        .with_rotation(Quat::from_rotation_y(base_yaw))
        .with_scale(Vec3::splat(scale));

    commands
        .spawn((
            DandelionPlant {
                sway_phase,
                base_yaw,
                base_rotation: Quat::from_rotation_y(base_yaw),
                base_translation: base_pos,
                pivot: Vec3::ZERO,
            },
            root_tf,
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            crate::alchemy::Flammable::with_duration(2.0),
        ))
        .with_children(|plant| {

            let leaf_count = 6 + (rng() * 5.0) as usize;
            for i in 0..leaf_count {
                let angle = (i as f32 / leaf_count as f32) * std::f32::consts::TAU
                    + rng() * 0.4;
                let leaf_len = (0.16 + rng() * 0.14) / scale;
                let lobe_seed = rng();
                let leaf_mesh = create_leaf_mesh(leaf_len, (0.048 + rng() * 0.02) / scale, lobe_seed);
                let tilt = -0.20 - rng() * 0.18;
                plant.spawn((
                    Mesh3d(meshes.add(leaf_mesh)),
                    MeshMaterial3d(mats.leaf_green.clone()),
                    Transform::from_xyz(0.0, 0.002 / scale, 0.0)
                        .with_rotation(
                            Quat::from_rotation_y(angle) * Quat::from_rotation_x(tilt)
                        ),
                ));
            }

            let stem_r = (0.016 + rng() * 0.006) / scale;
            let stem_mesh = create_stem_mesh(stem_h, stem_r, 10, lean_x, lean_z);
            plant.spawn((
                Mesh3d(meshes.add(stem_mesh)),
                MeshMaterial3d(mats.stem_green.clone()),
                Transform::IDENTITY,
            ));

            let bract_h = (0.08 + rng() * 0.04) / scale;
            let bract_r = (0.05 + rng() * 0.02) / scale;
            let bract_mesh = create_bract_mesh(bract_r, bract_h, 12);
            plant.spawn((
                Mesh3d(meshes.add(bract_mesh)),
                MeshMaterial3d(mats.bract_green.clone()),
                Transform::from_translation(stem_tip_local),
            ));

            let rec_r = if is_bloom { 0.055 / scale } else { 0.044 / scale };
            let rec_mesh = create_receptacle_mesh(rec_r, 20);
            plant.spawn((
                Mesh3d(meshes.add(rec_mesh)),
                MeshMaterial3d(mats.receptacle_green.clone()),
                Transform::from_translation(stem_tip_local),
            ));

            let head_origin = stem_tip_local + Vec3::new(0.0, 0.006 / scale, 0.0);

            if is_bloom {

                let ray_count = 24 + (rng() * 16.0) as u32;
                let ray_len   = (0.072 + rng() * 0.028) / scale;
                let ray_w     = (0.014 + rng() * 0.006) / scale;
                for i in 0..ray_count {
                    let angle = (i as f32 / ray_count as f32) * std::f32::consts::TAU
                        + rng() * 0.1;
                    let tilt  = 0.22 + rng() * 0.18;
                    let rot   = Quat::from_rotation_y(angle) * Quat::from_rotation_x(-tilt);
                    plant.spawn((
                        Mesh3d(meshes.add(create_ray_floret_mesh(ray_len, ray_w))),
                        MeshMaterial3d(mats.floret_yellow.clone()),
                        Transform::from_translation(head_origin).with_rotation(rot),
                    ));
                }

                plant.spawn((
                    Mesh3d(meshes.add(create_disk_dome_mesh(0.030 / scale, 14))),
                    MeshMaterial3d(mats.disk_orange.clone()),
                    Transform::from_translation(head_origin + Vec3::new(0.0, 0.014 / scale, 0.0)),
                ));
            } else {

                let globe_r    = (0.095 + rng() * 0.025) / scale;
                let seed_count = 90 + (rng() * 70.0) as u32;
                let golden = std::f32::consts::PI * (3.0 - 5_f32.sqrt());

                for i in 0..seed_count {
                    let t     = i as f32 / seed_count as f32;
                    let incl  = (1.0 - 2.0 * t).acos();
                    let azim  = golden * i as f32;
                    let dir   = Vec3::new(
                        incl.sin() * azim.cos(),
                        incl.cos().abs() + 0.2,
                        incl.sin() * azim.sin(),
                    ).normalize();

                    if dir.y < -0.05 { continue; }

                    let achene_base = head_origin + dir * globe_r * 0.25;
                    let achene_tip  = head_origin + dir * globe_r;

                    plant.spawn((
                        Mesh3d(meshes.add(create_achene_mesh(achene_base, achene_tip, 0.005 / scale))),
                        MeshMaterial3d(mats.stem_green.clone()),
                        Transform::IDENTITY,
                    ));
                    plant.spawn((
                        Mesh3d(meshes.add(create_pappus_mesh(achene_tip, dir, 0.032 / scale, 7))),
                        MeshMaterial3d(mats.pappus_white.clone()),
                        Transform::IDENTITY,
                    ));
                }
            }
        });
}

fn create_stem_mesh(height: f32, radius: f32, segs: usize, lean_x: f32, lean_z: f32) -> Mesh {
    let radial = 8usize;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for seg in 0..=segs {
        let t  = seg as f32 / segs as f32;
        let cx = lean_x * t * t * height;
        let cy = t * height;
        let cz = lean_z * t * t * height;
        let r  = radius * (1.0 - t * 0.32);

        for ri in 0..radial {
            let angle = (ri as f32 / radial as f32) * std::f32::consts::TAU;
            let nx = angle.cos();
            let nz = angle.sin();
            positions.push([cx + nx * r, cy, cz + nz * r]);
            normals.push([nx, 0.05, nz]);
        }
    }

    let r = radial as u32;
    for seg in 0..segs as u32 {
        for ri in 0..r {
            let next = (ri + 1) % r;
            let b = seg * r;
            let t = (seg + 1) * r;
            indices.extend_from_slice(&[b + ri, t + ri, b + next]);
            indices.extend_from_slice(&[b + next, t + ri, t + next]);
        }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}

fn create_leaf_mesh(length: f32, width: f32, lobe_seed: f32) -> Mesh {
    let steps = 22usize;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let y = t * length;

        let envelope = (t * std::f32::consts::PI).sin().powf(0.55);
        let lobe = 1.0 + 0.38 * (t * 9.0 + lobe_seed * std::f32::consts::TAU).sin();
        let hw = width * envelope * lobe;
        let z_curl = t * t * 0.045;

        positions.push([-hw, y, z_curl]);
        positions.push([hw, y, z_curl]);
        normals.push([0.0, 0.15, 1.0]);
        normals.push([0.0, 0.15, 1.0]);
    }

    for i in 0..steps as u32 {
        let b = i * 2;

        indices.extend_from_slice(&[b, b + 2, b + 1]);
        indices.extend_from_slice(&[b + 1, b + 2, b + 3]);

        indices.extend_from_slice(&[b, b + 1, b + 2]);
        indices.extend_from_slice(&[b + 1, b + 3, b + 2]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}

fn create_bract_mesh(outer_r: f32, droop_h: f32, count: u32) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let next_a = ((i + 1) as f32 / count as f32) * std::f32::consts::TAU;
        let mid_a  = (angle + next_a) * 0.5;

        let in_r = outer_r * 0.28;
        let bl = Vec3::new(angle.cos() * in_r, 0.0, angle.sin() * in_r);
        let br = Vec3::new(next_a.cos() * in_r, 0.0, next_a.sin() * in_r);

        let tip = Vec3::new(mid_a.cos() * outer_r, -droop_h, mid_a.sin() * outer_r);

        let normal = (br - bl).cross(tip - bl).normalize_or_zero();
        let base_idx = positions.len() as u32;
        positions.extend_from_slice(&[bl.to_array(), br.to_array(), tip.to_array()]);
        normals.extend_from_slice(&[normal.to_array(); 3]);
        indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
        indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 1]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}

fn create_receptacle_mesh(radius: f32, segments: usize) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);

    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        let y = -(x * x + z * z) / (radius * 10.0);
        positions.push([x, y, z]);
        normals.push([x * 0.5, radius * 3.0, z * 0.5]);
    }

    for i in 1..=segments as u32 {
        indices.extend_from_slice(&[0, i, i + 1]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}

fn create_ray_floret_mesh(length: f32, width: f32) -> Mesh {
    let steps = 8usize;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let y = t * length;

        let env = if t < 0.12 {
            t / 0.12
        } else if t > 0.85 {
            let x = (t - 0.85) / 0.15;
            1.0 - x * x * 0.40
        } else {
            1.0
        };
        let hw = width * 0.5 * env;
        let crease_z = hw * 0.10;

        positions.push([-hw, y, 0.0]);
        positions.push([ 0.0, y, crease_z]);
        positions.push([ hw, y, 0.0]);
        normals.push([-0.1, 0.3, 1.0]);
        normals.push([ 0.0, 0.5, 1.0]);
        normals.push([ 0.1, 0.3, 1.0]);
    }

    let s = 3u32;
    for i in 0..steps as u32 {
        let b = i * s;
        let t = (i + 1) * s;

        indices.extend_from_slice(&[b, t, b + 1,  b + 1, t, t + 1]);
        indices.extend_from_slice(&[b, b + 1, t,  b + 1, t + 1, t]);

        indices.extend_from_slice(&[b + 1, t + 1, b + 2,  b + 2, t + 1, t + 2]);
        indices.extend_from_slice(&[b + 1, b + 2, t + 1,  b + 2, t + 2, t + 1]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}

fn create_disk_dome_mesh(radius: f32, segments: usize) -> Mesh {
    let rings = 4usize;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    positions.push([0.0, radius * 0.72, 0.0]);
    normals.push([0.0, 1.0, 0.0]);

    for ring in 1..=rings {
        let phi = (ring as f32 / rings as f32) * std::f32::consts::FRAC_PI_2;
        let y = radius * phi.cos();
        let r = radius * phi.sin();
        for j in 0..segments {
            let theta = (j as f32 / segments as f32) * std::f32::consts::TAU;
            let x = r * theta.cos();
            let z = r * theta.sin();
            positions.push([x, y, z]);
            let n = Vec3::new(x, y, z).normalize();
            normals.push(n.to_array());
        }
    }

    for j in 0..segments as u32 {
        let next_j = (j + 1) % segments as u32;
        indices.extend_from_slice(&[0, 1 + j, 1 + next_j]);
    }

    for ring in 0..(rings - 1) as u32 {
        let base_curr = 1 + ring * segments as u32;
        let base_next = 1 + (ring + 1) * segments as u32;
        for j in 0..segments as u32 {
            let next_j = (j + 1) % segments as u32;
            indices.extend_from_slice(&[
                base_curr + j, base_next + j, base_curr + next_j,
                base_curr + next_j, base_next + j, base_next + next_j,
            ]);
        }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}

fn create_achene_mesh(origin: Vec3, tip: Vec3, thickness: f32) -> Mesh {
    let dir  = (tip - origin).normalize_or_zero();
    let perp = dir.cross(Vec3::Y).normalize_or_zero() * thickness;
    let positions = vec![
        (origin - perp).to_array(),
        (origin + perp).to_array(),
        (tip    - perp * 0.35).to_array(),
        (tip    + perp * 0.35).to_array(),
    ];
    let normals = vec![[0.0, 0.0, 1.0]; 4];
    let indices = Indices::U32(vec![0, 2, 1, 1, 2, 3, 0, 1, 2, 1, 3, 2]);
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(indices)
}

fn create_pappus_mesh(tip: Vec3, outward: Vec3, radius: f32, ray_count: u32) -> Mesh {
    let up      = outward.normalize();
    let right   = up.cross(Vec3::Z).normalize_or_zero();
    let forward = up.cross(right).normalize_or_zero();

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for i in 0..ray_count {
        let angle = (i as f32 / ray_count as f32) * std::f32::consts::TAU;
        let ray_dir = (right * angle.cos() + forward * angle.sin()).normalize_or_zero();
        let end     = tip + ray_dir * radius + up * radius * 0.22;
        let mid     = tip + ray_dir * radius * 0.6 + up * radius * 0.08;

        let base = positions.len() as u32;
        positions.extend_from_slice(&[tip.to_array(), end.to_array(), mid.to_array()]);
        normals.extend_from_slice(&[up.to_array(); 3]);
        indices.extend_from_slice(&[base, base + 1, base + 2]);
        indices.extend_from_slice(&[base, base + 2, base + 1]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn animate_dandelions(
    time: Res<Time>,
    config: Option<Res<crate::dungeon::vegetation_animation::VegetationAnimationConfig>>,
    player_query: Query<&GlobalTransform, (With<crate::player::camera::Player>, Without<DandelionPlant>)>,
    mut query: Query<(&DandelionPlant, &GlobalTransform, &mut Transform), Without<crate::player::camera::Player>>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 || dt > 0.1 {
        return;
    }
    let default_prof = crate::dungeon::vegetation_animation::DandelionAnimationProfile::default();
    let prof = config
        .as_ref()
        .map(|c| &c.dandelion_profile)
        .unwrap_or(&default_prof);

    let t = time.elapsed_secs();
    let player_pos_opt = player_query.get_single().map(|gt| gt.translation()).ok();

    for (plant, gt, mut tf) in query.iter_mut() {
        let idle_speed = prof.idle_sway_speed;
        let idle_x = (t * idle_speed + plant.sway_phase).sin() * prof.idle_pitch_amplitude
            + (t * (idle_speed * prof.secondary_frequency_mult) + plant.sway_phase * 0.6).cos()
                * (prof.idle_pitch_amplitude * prof.secondary_amplitude_mult);
        let idle_z = (t * (idle_speed * 0.85) + plant.sway_phase * 1.3).sin() * prof.idle_roll_amplitude;

        let mut push = Vec2::ZERO;
        if let Some(player_pos) = player_pos_opt {
            let d_pos = gt.translation();
            let to_d = Vec2::new(d_pos.x - player_pos.x, d_pos.z - player_pos.z);
            let dist = to_d.length();
            if dist < prof.player_push_radius && dist > 1e-4 {
                let factor = (1.0 - dist / prof.player_push_radius).clamp(0.0, 1.0);
                let push_dir = to_d / dist;
                push = push_dir * (factor * prof.player_push_strength);
            }
        }

        let sway_x = idle_x + push.x;
        let sway_z = idle_z + push.y;
        let base = if plant.base_rotation != Quat::IDENTITY {
            plant.base_rotation
        } else {
            Quat::from_rotation_y(plant.base_yaw)
        };
        let wind = Quat::from_rotation_x(sway_x) * Quat::from_rotation_z(sway_z);
        tf.rotation = base * wind;
        if plant.pivot != Vec3::ZERO {
            tf.translation = plant.base_translation + plant.pivot - (tf.rotation * plant.pivot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stem_mesh_topology() {
        let m = create_stem_mesh(0.5, 0.02, 8, 0.0, 0.0);
        assert_eq!(m.primitive_topology(), PrimitiveTopology::TriangleList);
        assert!(m.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(m.indices().is_some());
    }

    #[test]
    fn test_leaf_mesh_has_positions_and_indices() {
        let m = create_leaf_mesh(0.2, 0.06, 0.0);
        assert!(m.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(m.indices().is_some());
    }

    #[test]
    fn test_receptacle_connects_to_stem_tip() {

        let m = create_receptacle_mesh(0.06, 20);
        assert!(m.indices().is_some());
    }

    #[test]
    fn test_pappus_mesh_has_two_sided_faces() {
        let m = create_pappus_mesh(Vec3::new(0.0, 0.5, 0.0), Vec3::Y, 0.04, 8);
        assert!(m.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
    }
}
