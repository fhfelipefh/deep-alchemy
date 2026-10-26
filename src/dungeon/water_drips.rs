use avian3d::prelude::*;
use bevy::{
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    render::render_asset::RenderAssetUsages,
};

use crate::{
    alchemy::{
        surface_fluid::create_teardrop_droplet_mesh, AlchemicalEffectKind, PotionPuddle,
        PotionProjectile, ThermalGeyser,
    },
    dungeon::modular_kit::spawn_water_ripple,
    player::camera::Player,
    render::particles::{fastrand, Particle, ParticleType},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterPuddleState {
    Liquid,
    Frozen,
    Boiling,
    Evaporated,
}

#[derive(Component, Debug, Clone)]
pub struct CavernWaterPuddle {
    pub state: WaterPuddleState,
    pub radius: f32,
    pub original_radius: f32,
    pub base_y: f32,
    pub freeze_timer: f32,
    pub boil_timer: f32,
    pub regen_timer: f32,
    pub step_ripple_cooldown: f32,
}

#[derive(Component, Debug, Clone)]
pub struct CeilingDripPoint {
    pub timer: f32,
    pub drip_interval: f32,
    pub puddle_entity: Option<Entity>,
    pub drop_fall_speed: f32,
}

#[derive(Component, Debug, Clone)]
pub struct FallingWaterDrop {
    pub velocity_y: f32,
    pub target_y: f32,
    pub origin: Vec3,
    pub puddle_entity: Option<Entity>,
}

pub fn create_stalactite_mesh(
    top_radius: f32,
    bottom_radius: f32,
    height: f32,
    segments: usize,
) -> Mesh {
    let segments = segments.max(6);
    let v_steps = 5;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity((v_steps + 1) * (segments + 1));
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity((v_steps + 1) * (segments + 1));
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity((v_steps + 1) * (segments + 1));
    let mut indices: Vec<u32> = Vec::with_capacity(v_steps * segments * 6 + segments * 3);

    for v in 0..=v_steps {
        let t = v as f32 / v_steps as f32;
        let (y, r_t) = if v == 0 {
            (0.12, top_radius * 1.85)
        } else {
            let t_eff = (v - 1) as f32 / (v_steps - 1) as f32;
            (-t_eff * height, top_radius + (bottom_radius - top_radius) * t_eff.powf(1.4))
        };

        for s in 0..=segments {
            let u = s as f32 / segments as f32;
            let angle = u * std::f32::consts::TAU;
            let noise = 1.0 + 0.12 * (angle * 3.0 + t * 4.0).sin() + 0.08 * (angle * 5.0).cos();
            let r = r_t * noise;
            let px = angle.cos() * r;
            let pz = angle.sin() * r;

            positions.push([px, y, pz]);
            let norm = Vec3::new(angle.cos() * 0.9, 0.25, angle.sin() * 0.9).normalize();
            normals.push([norm.x, norm.y, norm.z]);
            uvs.push([u, t]);
        }
    }

    let stride = (segments + 1) as u32;
    for v in 0..v_steps as u32 {
        for s in 0..segments as u32 {
            let i0 = v * stride + s;
            let i1 = i0 + 1;
            let i2 = (v + 1) * stride + s;
            let i3 = i2 + 1;

            indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }
    }

    let tip_center_idx = positions.len() as u32;
    positions.push([0.0, -height - 0.04, 0.0]);
    normals.push([0.0, -1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    let bottom_ring_start = v_steps as u32 * stride;
    for s in 0..segments as u32 {
        let next = s + 1;
        indices.extend_from_slice(&[
            tip_center_idx,
            bottom_ring_start + s,
            bottom_ring_start + next,
        ]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_realistic_cavern_water_mesh(
    base_radius: f32,
    segments: usize,
    center_world: Vec3,
) -> Mesh {
    let segments = segments.max(24);
    let in_water = crate::dungeon::modular_kit::is_in_lake_water(center_world);
    let center_terr = if in_water {
        0.18
    } else {
        crate::dungeon::modular_kit::get_grotto_terrain_height(center_world.x, center_world.z)
    };

    let ring_steps = 3;
    let total_verts = 1 + segments * ring_steps;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(total_verts);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(total_verts);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(total_verts);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(total_verts);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * (3 + (ring_steps - 1) * 6));

    positions.push([0.0, 0.024, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);
    colors.push([0.75, 0.90, 0.98, 0.85]);

    let radii_factors = [0.45, 0.78, 1.00];
    let y_offsets = [0.019, 0.012, 0.003];
    let alpha_factors = [0.65, 0.35, 0.00];
    let normal_tilts = [0.06, 0.14, 0.28];

    for ((&r_fac, &y_base), (&alpha_val, &n_tilt)) in radii_factors
        .iter()
        .zip(y_offsets.iter())
        .zip(alpha_factors.iter().zip(normal_tilts.iter()))
    {
        for s in 0..segments {
            let theta = (s as f32 / segments as f32) * std::f32::consts::TAU;
            let pert = 1.0
                + 0.07 * (theta * 3.0 + 0.5).sin()
                + 0.04 * (theta * 5.0 + 1.2).cos()
                + 0.02 * (theta * 7.0).sin();
            let r = base_radius * pert * r_fac;
            let x = r * theta.cos();
            let z = r * theta.sin();
            let dy = if in_water {
                0.0
            } else {
                let terr_h = crate::dungeon::modular_kit::get_grotto_terrain_height(
                    center_world.x + x,
                    center_world.z + z,
                );
                (terr_h - center_terr).clamp(-0.06, 0.06)
            };

            positions.push([x, y_base + dy * 0.75, z]);
            let norm = Vec3::new(theta.cos() * n_tilt, 1.0 - n_tilt * 0.5, theta.sin() * n_tilt)
                .normalize();
            normals.push([norm.x, norm.y, norm.z]);
            uvs.push([0.5 + x / (base_radius * 2.2), 0.5 + z / (base_radius * 2.2)]);
            colors.push([0.70, 0.88, 0.96, alpha_val]);
        }
    }

    let seg_u32 = segments as u32;
    for s in 0..seg_u32 {
        let next_s = (s + 1) % seg_u32;
        indices.extend_from_slice(&[0, next_s + 1, s + 1]);
    }

    for step in 0..(ring_steps - 1) as u32 {
        let inner_start = 1 + step * seg_u32;
        let outer_start = 1 + (step + 1) * seg_u32;
        for s in 0..seg_u32 {
            let next_s = (s + 1) % seg_u32;
            let i0 = inner_start + s;
            let i1 = inner_start + next_s;
            let o0 = outer_start + s;
            let o1 = outer_start + next_s;

            indices.extend_from_slice(&[i0, i1, o1]);
            indices.extend_from_slice(&[i0, o1, o0]);
        }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_wet_stone_underlayer_mesh(
    base_radius: f32,
    segments: usize,
    center_world: Vec3,
) -> Mesh {
    let segments = segments.max(24);
    let in_water = crate::dungeon::modular_kit::is_in_lake_water(center_world);
    let center_terr = if in_water {
        0.18
    } else {
        crate::dungeon::modular_kit::get_grotto_terrain_height(center_world.x, center_world.z)
    };

    let total_radius = base_radius * 1.16;
    let ring_steps = 2;
    let total_verts = 1 + segments * ring_steps;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(total_verts);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(total_verts);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(total_verts);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(total_verts);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * (3 + (ring_steps - 1) * 6));

    positions.push([0.0, 0.010, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);
    colors.push([0.05, 0.07, 0.09, 0.68]);

    let radii_factors = [0.65, 1.00];
    let y_offsets = [0.008, 0.002];
    let alpha_factors = [0.42, 0.00];

    for ((&r_fac, &y_base), &alpha_val) in radii_factors
        .iter()
        .zip(y_offsets.iter())
        .zip(alpha_factors.iter())
    {
        for s in 0..segments {
            let theta = (s as f32 / segments as f32) * std::f32::consts::TAU;
            let pert = 1.0
                + 0.08 * (theta * 3.0 + 0.4).sin()
                + 0.05 * (theta * 5.0 + 1.1).cos()
                + 0.02 * (theta * 7.0).sin();
            let r = total_radius * pert * r_fac;
            let x = r * theta.cos();
            let z = r * theta.sin();
            let dy = if in_water {
                0.0
            } else {
                let terr_h = crate::dungeon::modular_kit::get_grotto_terrain_height(
                    center_world.x + x,
                    center_world.z + z,
                );
                (terr_h - center_terr).clamp(-0.06, 0.06)
            };

            positions.push([x, y_base + dy * 0.75, z]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([0.5 + x / (total_radius * 2.2), 0.5 + z / (total_radius * 2.2)]);
            colors.push([0.05, 0.07, 0.09, alpha_val]);
        }
    }

    let seg_u32 = segments as u32;
    for s in 0..seg_u32 {
        let next_s = (s + 1) % seg_u32;
        indices.extend_from_slice(&[0, next_s + 1, s + 1]);
    }

    let inner_start = 1;
    let outer_start = 1 + seg_u32;
    for s in 0..seg_u32 {
        let next_s = (s + 1) % seg_u32;
        let i0 = inner_start + s;
        let i1 = inner_start + next_s;
        let o0 = outer_start + s;
        let o1 = outer_start + next_s;

        indices.extend_from_slice(&[i0, i1, o1]);
        indices.extend_from_slice(&[i0, o1, o0]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn spawn_cavern_water_drip_pair(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    floor_pos: Vec3,
    ceiling_pos: Vec3,
    radius: f32,
    interval: f32,
) -> (Entity, Entity) {
    let water_mesh = meshes.add(create_realistic_cavern_water_mesh(
        radius,
        32,
        floor_pos,
    ));
    let wet_stone_mesh = meshes.add(create_wet_stone_underlayer_mesh(
        radius,
        32,
        floor_pos,
    ));

    let water_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.04, 0.14, 0.20, 0.42),
        emissive: LinearRgba::new(0.002, 0.008, 0.012, 0.02),
        perceptual_roughness: 0.02,
        reflectance: 0.98,
        metallic: 0.0,
        cull_mode: None,
        double_sided: true,
        alpha_mode: AlphaMode::Blend,
        depth_bias: 560.0,
        ..default()
    });

    let wet_stone_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.35,
        reflectance: 0.40,
        metallic: 0.0,
        cull_mode: None,
        alpha_mode: AlphaMode::Blend,
        depth_bias: 520.0,
        ..default()
    });

    let puddle_entity = commands
        .spawn((
            Name::new("Cavern_Water_Puddle"),
            CavernWaterPuddle {
                state: WaterPuddleState::Liquid,
                radius,
                original_radius: radius,
                base_y: floor_pos.y,
                freeze_timer: 0.0,
                boil_timer: 0.0,
                regen_timer: 0.0,
                step_ripple_cooldown: 0.0,
            },
            Mesh3d(water_mesh),
            MeshMaterial3d(water_mat),
            Transform::from_translation(floor_pos),
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Cavern_Puddle_Wet_Ground"),
                Mesh3d(wet_stone_mesh),
                MeshMaterial3d(wet_stone_mat),
                Transform::from_xyz(0.0, -0.005, 0.0),
            ));
        })
        .id();

    let stalactite_mesh = meshes.add(create_stalactite_mesh(0.18, 0.035, 0.65, 8));
    let sub_stalactite_mesh = meshes.add(create_stalactite_mesh(0.08, 0.02, 0.35, 6));
    let ceiling_rock_mesh = meshes.add(crate::dungeon::modular_kit::create_faceted_stone_mesh(
        7,
        5,
        Vec3::new(0.65, 0.40, 0.65),
        0.5,
    ));

    let stalactite_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.14, 0.18, 0.22),
        perceptual_roughness: 0.88,
        metallic: 0.0,
        reflectance: 0.15,
        ..default()
    });

    let drip_tip_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.55, 0.88, 1.0, 0.85),
        emissive: LinearRgba::new(0.05, 0.20, 0.35, 0.30),
        perceptual_roughness: 0.08,
        reflectance: 0.95,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let tip_mesh = meshes.add(Sphere::new(0.032).mesh().uv(6, 6));

    let ceiling_entity = commands
        .spawn((
            Name::new("Ceiling_Water_Drip_Point"),
            CeilingDripPoint {
                timer: fastrand::f32() * interval,
                drip_interval: interval,
                puddle_entity: Some(puddle_entity),
                drop_fall_speed: 0.0,
            },
            Mesh3d(stalactite_mesh),
            MeshMaterial3d(stalactite_mat.clone()),
            Transform::from_translation(ceiling_pos),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Ceiling_Rock_Anchor"),
                Mesh3d(ceiling_rock_mesh),
                MeshMaterial3d(stalactite_mat.clone()),
                Transform::from_xyz(0.0, 0.20, 0.0),
            ));
            parent.spawn((
                Name::new("Secondary_Stalactite_A"),
                Mesh3d(sub_stalactite_mesh.clone()),
                MeshMaterial3d(stalactite_mat.clone()),
                Transform::from_xyz(-0.16, 0.04, 0.12).with_scale(Vec3::splat(0.65)),
            ));
            parent.spawn((
                Name::new("Secondary_Stalactite_B"),
                Mesh3d(sub_stalactite_mesh),
                MeshMaterial3d(stalactite_mat),
                Transform::from_xyz(0.14, 0.06, -0.10).with_scale(Vec3::splat(0.48)),
            ));
            parent.spawn((
                Name::new("Drip_Tip_Droplet"),
                Mesh3d(tip_mesh),
                MeshMaterial3d(drip_tip_mat),
                Transform::from_xyz(0.0, -0.68, 0.0),
            ));
        })
        .id();

    (ceiling_entity, puddle_entity)
}

pub fn spawn_default_cavern_water_drips(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let drip_configs = [
        (
            Vec3::new(1.05, 0.02, 6.40),
            Vec3::new(1.05, 4.40, 6.40),
            1.05,
            2.8,
        ),
        (
            Vec3::new(-1.05, 0.02, 9.50),
            Vec3::new(-1.05, 4.40, 9.50),
            1.10,
            3.4,
        ),
        (
            Vec3::new(1.00, 0.02, 13.50),
            Vec3::new(1.00, 4.40, 13.50),
            0.95,
            2.3,
        ),
        (
            Vec3::new(-1.00, 0.02, 17.50),
            Vec3::new(-1.00, 4.40, 17.50),
            1.00,
            3.8,
        ),
    ];

    for (floor_pos, ceiling_pos, radius, interval) in drip_configs {
        spawn_cavern_water_drip_pair(
            commands,
            meshes,
            materials,
            floor_pos,
            ceiling_pos,
            radius,
            interval,
        );
    }
}

pub fn update_ceiling_drip_points(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut drip_points: Query<(&Transform, &mut CeilingDripPoint)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (transform, mut point) in drip_points.iter_mut() {
        point.timer += dt;
        if point.timer >= point.drip_interval {
            point.timer = 0.0;
            let spawn_pos = transform.translation - Vec3::Y * 0.72;
            let target_y = 0.02;

            let drop_mesh =
                meshes.add(create_teardrop_droplet_mesh(0.038, 8, 6));
            let drop_mat = materials.add(StandardMaterial {
                base_color: Color::srgba(0.65, 0.90, 1.0, 0.88),
                emissive: LinearRgba::new(0.10, 0.40, 0.65, 0.50),
                perceptual_roughness: 0.10,
                reflectance: 0.90,
                alpha_mode: AlphaMode::Blend,
                unlit: false,
                cull_mode: None,
                ..default()
            });

            commands.spawn((
                Name::new("Falling_Ceiling_Water_Drop"),
                FallingWaterDrop {
                    velocity_y: -0.3,
                    target_y,
                    origin: spawn_pos,
                    puddle_entity: point.puddle_entity,
                },
                Mesh3d(drop_mesh),
                MeshMaterial3d(drop_mat),
                Transform::from_translation(spawn_pos)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
            ));
        }
    }
}

pub fn update_falling_water_drops(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut drops: Query<(Entity, &mut Transform, &mut FallingWaterDrop)>,
    mut puddles: Query<(&Transform, &CavernWaterPuddle), Without<FallingWaterDrop>>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (entity, mut transform, mut drop) in drops.iter_mut() {
        drop.velocity_y -= 9.81 * dt;
        transform.translation.y += drop.velocity_y * dt;

        if transform.translation.y <= drop.target_y {
            commands.entity(entity).despawn();

            let impact_pos =
                Vec3::new(transform.translation.x, drop.target_y, transform.translation.z);

            if let Some(puddle_ent) = drop.puddle_entity {
                if let Ok((puddle_tf, puddle)) = puddles.get_mut(puddle_ent) {
                    match puddle.state {
                        WaterPuddleState::Liquid => {
                            let p_pos = puddle_tf.translation;
                            let drop_dist = Vec2::new(impact_pos.x - p_pos.x, impact_pos.z - p_pos.z).length();
                            let max_allowed_r = (puddle.radius - drop_dist - 0.14).clamp(0.08, puddle.radius * 0.52);

                            spawn_water_ripple(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                Vec3::new(impact_pos.x, puddle.base_y + 0.024, impact_pos.z),
                                0.06,
                                max_allowed_r,
                                1.3,
                                0.45,
                            );

                            commands.spawn((
                                Mesh3d(meshes.add(create_teardrop_droplet_mesh(0.035, 8, 6))),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: Color::LinearRgba(LinearRgba::new(
                                        0.40, 0.75, 0.98, 0.45,
                                    )),
                                    alpha_mode: AlphaMode::Blend,
                                    unlit: false,
                                    perceptual_roughness: 0.05,
                                    reflectance: 0.95,
                                    ..default()
                                })),
                                Transform::from_translation(impact_pos + Vec3::Y * 0.04),
                                Particle {
                                    lifetime: 0.0,
                                    max_lifetime: 0.7,
                                    velocity: Vec3::new(
                                        (fastrand::f32() - 0.5) * 0.8,
                                        0.9 + fastrand::f32() * 0.6,
                                        (fastrand::f32() - 0.5) * 0.8,
                                    ),
                                    p_type: ParticleType::WaterSplash,
                                },
                            ));
                        }
                        WaterPuddleState::Frozen => {
                            commands.spawn((
                                Mesh3d(meshes.add(Sphere::new(0.025).mesh().uv(6, 6))),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: Color::srgba(0.85, 0.95, 1.0, 0.75),
                                    alpha_mode: AlphaMode::Blend,
                                    unlit: false,
                                    perceptual_roughness: 0.20,
                                    reflectance: 0.80,
                                    ..default()
                                })),
                                Transform::from_translation(impact_pos + Vec3::Y * 0.03),
                                Particle {
                                    lifetime: 0.0,
                                    max_lifetime: 0.5,
                                    velocity: Vec3::new(
                                        (fastrand::f32() - 0.5) * 0.6,
                                        0.4 + fastrand::f32() * 0.4,
                                        (fastrand::f32() - 0.5) * 0.6,
                                    ),
                                    p_type: ParticleType::WaterSplash,
                                },
                            ));
                        }
                        WaterPuddleState::Boiling | WaterPuddleState::Evaporated => {
                            commands.spawn((
                                Mesh3d(meshes.add(Sphere::new(0.06).mesh().uv(6, 6))),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: Color::srgba(0.90, 0.96, 1.0, 0.35),
                                    alpha_mode: AlphaMode::Blend,
                                    unlit: true,
                                    ..default()
                                })),
                                Transform::from_translation(impact_pos + Vec3::Y * 0.05),
                                Particle {
                                    lifetime: 0.0,
                                    max_lifetime: 1.2,
                                    velocity: Vec3::new(
                                        (fastrand::f32() - 0.5) * 0.3,
                                        0.8 + fastrand::f32() * 0.5,
                                        (fastrand::f32() - 0.5) * 0.3,
                                    ),
                                    p_type: ParticleType::Steam,
                                },
                            ));
                        }
                    }
                }
            }
        }
    }
}

pub fn update_puddle_player_interaction(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut puddles: Query<(&Transform, &mut CavernWaterPuddle, &Visibility), Without<Player>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else {
        return;
    };

    for (puddle_tf, mut puddle, visibility) in puddles.iter_mut() {
        if puddle.state != WaterPuddleState::Liquid || *visibility == Visibility::Hidden {
            puddle.step_ripple_cooldown = (puddle.step_ripple_cooldown - dt).max(0.0);
            continue;
        }

        let dx = player_tf.translation.x - puddle_tf.translation.x;
        let dz = player_tf.translation.z - puddle_tf.translation.z;
        let dist = (dx * dx + dz * dz).sqrt();

        if dist <= puddle.radius {
            puddle.step_ripple_cooldown -= dt;
            if puddle.step_ripple_cooldown <= 0.0 {
                puddle.step_ripple_cooldown = 0.22;

                let dist_to_rim = puddle.radius - dist;
                let max_ripple_r = (dist_to_rim - 0.12).clamp(0.0, 0.38);

                if max_ripple_r >= 0.08 {
                    spawn_water_ripple(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        Vec3::new(
                            player_tf.translation.x,
                            puddle.base_y + 0.024,
                            player_tf.translation.z,
                        ),
                        0.08,
                        max_ripple_r,
                        1.0,
                        0.52,
                    );
                }

                commands.spawn((
                    Mesh3d(meshes.add(create_teardrop_droplet_mesh(0.040, 8, 6))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::LinearRgba(LinearRgba::new(0.35, 0.70, 0.95, 0.40)),
                        alpha_mode: AlphaMode::Blend,
                        unlit: false,
                        perceptual_roughness: 0.05,
                        reflectance: 0.95,
                        ..default()
                    })),
                    Transform::from_xyz(
                        player_tf.translation.x,
                        puddle.base_y + 0.035,
                        player_tf.translation.z,
                    ),
                    Particle {
                        lifetime: 0.0,
                        max_lifetime: 0.7,
                        velocity: Vec3::new(
                            (fastrand::f32() - 0.5) * 1.0,
                            0.7 + fastrand::f32() * 0.6,
                            (fastrand::f32() - 0.5) * 1.0,
                        ),
                        p_type: ParticleType::WaterSplash,
                    },
                ));
            }
        } else {
            puddle.step_ripple_cooldown = (puddle.step_ripple_cooldown - dt).max(0.0);
        }
    }
}

pub fn update_puddle_temperature_reactions(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut puddles: Query<(
        Entity,
        &Transform,
        &mut CavernWaterPuddle,
        &MeshMaterial3d<StandardMaterial>,
        &mut Visibility,
    ), (Without<PotionPuddle>, Without<ThermalGeyser>, Without<PotionProjectile>)>,
    potion_puddles: Query<(&Transform, &PotionPuddle)>,
    geysers: Query<(&Transform, &ThermalGeyser)>,
    projectiles: Query<(&Transform, &PotionProjectile)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (puddle_entity, puddle_tf, mut puddle, mat_handle, mut visibility) in puddles.iter_mut() {
        let p_pos = puddle_tf.translation;
        let mut has_frost = false;
        let mut has_heat = false;

        for (pp_tf, pp) in potion_puddles.iter() {
            let dist = p_pos.distance(pp_tf.translation);
            if dist <= puddle.radius + pp.radius + 0.4 {
                if pp.effects.iter().any(|e| e.kind == AlchemicalEffectKind::Frost) {
                    has_frost = true;
                }
                if pp.effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat) {
                    has_heat = true;
                }
            }
        }

        for (proj_tf, proj) in projectiles.iter() {
            let dist = p_pos.distance(proj_tf.translation);
            if dist <= puddle.radius + 1.2 {
                if proj.effects.iter().any(|e| e.kind == AlchemicalEffectKind::Frost) {
                    has_frost = true;
                }
                if proj.effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat) {
                    has_heat = true;
                }
            }
        }

        for (g_tf, g) in geysers.iter() {
            if g.is_erupting {
                let dist = p_pos.distance(g_tf.translation);
                if dist <= puddle.radius + g.heat_radius + 0.5 {
                    has_heat = true;
                }
            }
        }

        match puddle.state {
            WaterPuddleState::Liquid => {
                if has_frost {
                    puddle.state = WaterPuddleState::Frozen;
                    puddle.freeze_timer = 22.0;

                    if let Some(mat) = materials.get_mut(&mat_handle.0) {
                        mat.base_color = Color::srgba(0.82, 0.92, 0.98, 0.90);
                        mat.emissive = LinearRgba::new(0.06, 0.18, 0.28, 0.25);
                        mat.perceptual_roughness = 0.22;
                        mat.metallic = 0.04;
                        mat.reflectance = 0.80;
                    }

                    commands.entity(puddle_entity).insert((
                        RigidBody::Static,
                        Collider::cylinder(puddle.radius * 0.95, 0.08),
                    ));
                } else if has_heat {
                    puddle.state = WaterPuddleState::Boiling;
                    puddle.boil_timer = 2.8;
                }
            }
            WaterPuddleState::Frozen => {
                if has_heat {
                    puddle.state = WaterPuddleState::Liquid;
                    puddle.freeze_timer = 0.0;
                    commands.entity(puddle_entity).remove::<(RigidBody, Collider)>();

                    if let Some(mat) = materials.get_mut(&mat_handle.0) {
                        mat.base_color = Color::srgba(0.04, 0.14, 0.20, 0.42);
                        mat.emissive = LinearRgba::new(0.002, 0.008, 0.012, 0.02);
                        mat.perceptual_roughness = 0.02;
                        mat.metallic = 0.0;
                        mat.reflectance = 0.98;
                    }
                } else {
                    puddle.freeze_timer -= dt;
                    if puddle.freeze_timer <= 0.0 && !has_frost {
                        puddle.state = WaterPuddleState::Liquid;
                        commands.entity(puddle_entity).remove::<(RigidBody, Collider)>();

                        if let Some(mat) = materials.get_mut(&mat_handle.0) {
                            mat.base_color = Color::srgba(0.04, 0.14, 0.20, 0.42);
                            mat.emissive = LinearRgba::new(0.002, 0.008, 0.012, 0.02);
                            mat.perceptual_roughness = 0.02;
                            mat.metallic = 0.0;
                            mat.reflectance = 0.98;
                        }
                    }
                }
            }
            WaterPuddleState::Boiling => {
                puddle.boil_timer -= dt;

                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.10).mesh().uv(6, 6))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(0.92, 0.96, 1.0, 0.32),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_xyz(
                        p_pos.x + (fastrand::f32() - 0.5) * puddle.radius,
                        puddle.base_y + 0.08,
                        p_pos.z + (fastrand::f32() - 0.5) * puddle.radius,
                    ),
                    Particle {
                        lifetime: 0.0,
                        max_lifetime: 1.8,
                        velocity: Vec3::new(
                            (fastrand::f32() - 0.5) * 0.4,
                            1.2 + fastrand::f32() * 0.8,
                            (fastrand::f32() - 0.5) * 0.4,
                        ),
                        p_type: ParticleType::Steam,
                    },
                ));

                if puddle.boil_timer <= 0.0 {
                    puddle.state = WaterPuddleState::Evaporated;
                    puddle.regen_timer = 18.0;
                    *visibility = Visibility::Hidden;
                }
            }
            WaterPuddleState::Evaporated => {
                if !has_heat {
                    puddle.regen_timer -= dt;
                    if puddle.regen_timer <= 0.0 {
                        puddle.state = WaterPuddleState::Liquid;
                        *visibility = Visibility::Inherited;

                        if let Some(mat) = materials.get_mut(&mat_handle.0) {
                            mat.base_color = Color::srgba(0.04, 0.14, 0.20, 0.42);
                            mat.emissive = LinearRgba::new(0.002, 0.008, 0.012, 0.02);
                            mat.perceptual_roughness = 0.02;
                            mat.metallic = 0.0;
                            mat.reflectance = 0.98;
                        }
                    }
                } else {
                    puddle.regen_timer = 18.0;
                }
            }
        }
    }
}

pub fn update_puddle_boiling_visuals(
    mut puddles: Query<(&mut Transform, &CavernWaterPuddle)>,
) {
    for (mut transform, puddle) in puddles.iter_mut() {
        if puddle.state == WaterPuddleState::Boiling {
            let frac = (puddle.boil_timer / 2.8).clamp(0.05, 1.0);
            transform.scale = Vec3::new(frac, 1.0, frac);
        } else if puddle.state == WaterPuddleState::Liquid || puddle.state == WaterPuddleState::Frozen {
            transform.scale = Vec3::ONE;
        }
    }
}
