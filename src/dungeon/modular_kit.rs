use crate::{
    alchemy::{AlchemicalEffectKind, AlchemicalIngredient, ThermalGeyser},
    dungeon::asset_loader::{SceneAssetInstance, SceneAssetRegistry},
    player::interaction::Interactable,
    render::{
        cel_moss_material::CelShadedMossMaterial,
        particles::{ParticleEmitter, ParticleType},
        water_material::WaterMaterial,
    },
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, MeshAabb, PrimitiveTopology};
use bevy::render::primitives::Aabb;
use bevy::render::render_asset::RenderAssetUsages;

#[derive(Component)]
pub struct WaterfallMarker;

#[derive(Component)]
pub struct KeyShadowLight;

#[derive(Component, Clone)]
pub struct WaterfallBaseMesh {
    pub base_positions: Vec<[f32; 3]>,
    pub base_normals: Vec<[f32; 3]>,
}

#[derive(Component)]
pub struct CavernObstacle;

#[derive(Component, Debug, Clone)]
pub struct BenchObstacle;

#[derive(Component)]
pub struct TallGrassClump;

#[derive(Component, Debug, Clone)]
pub struct GrassBending {
    pub base_rotation: Quat,
    pub current_bend: Vec2,
    pub bend_velocity: Vec2,
}

#[derive(Component, Debug, Clone)]
pub struct WaterRipple {
    pub elapsed: f32,
    pub max_lifetime: f32,
    pub initial_radius: f32,
    pub max_radius: f32,
    pub initial_alpha: f32,
}

#[derive(Component, Debug, Clone)]
pub struct BioluminescentMushroom {
    pub base_intensity: f32,
    pub base_color: Color,
    pub phase: f32,
    pub surge_timer: f32,
}

#[derive(Component)]
pub struct MushroomCap;

#[derive(Component, Debug, Clone)]
pub struct MushroomOrganicBreathing {
    pub base_scale: Vec3,
    pub base_rotation: Quat,
    pub phase_offset: f32,
}

#[derive(Component)]
pub struct MushroomSporeAttached;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum ObstacleShape {
    Cylinder { radius: f32, height: f32 },
    Cuboid { half_extents: Vec3 },
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct LakeWaterPool;

pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        let anim_config = crate::dungeon::vegetation_animation::load_vegetation_animations_from_disk(
            "assets/data/vegetation_animations.ron",
        );
        app.insert_resource(anim_config)
            .init_resource::<SceneAssetRegistry>()
            .add_systems(
                Startup,
                (
                    setup_cavern_world,
                    crate::dungeon::generator::spawn_procedural_dungeon,
                ),
            )
        .add_systems(
            Update,
            (
                animate_waterfall_mesh,
                mark_imported_tall_grass,
                attach_mushroom_spore_emitters,
                mark_imported_flammable_props,
                update_water_ripples,
                update_bioluminescent_mushrooms,
                crate::dungeon::flora::animate_dandelions,
                crate::dungeon::vegetation_animation::update_vegetation_foliage_animation,
            ),
        );
    }
}

pub fn mark_imported_tall_grass(
    mut commands: Commands,
    named_entities: Query<(Entity, &Name, Option<&Transform>, Option<&Mesh3d>), Added<Name>>,
    meshes: Option<Res<Assets<Mesh>>>,
) {
    for (entity, name, tf_opt, mesh_opt) in &named_entities {
        let name_str = name.as_str();
        if name_str.contains("TallGrass") && !name_str.contains("Mound") {
            let base_rot = tf_opt.map(|t| t.rotation).unwrap_or(Quat::IDENTITY);
            let center_offset = mesh_opt
                .and_then(|h| meshes.as_ref().and_then(|m| m.get(&h.0)))
                .and_then(|m| m.compute_aabb())
                .map(|a| Vec3::from(a.center))
                .unwrap_or(Vec3::ZERO);

            let deformer_opt = mesh_opt
                .and_then(|h| meshes.as_ref().and_then(|m| m.get(&h.0)))
                .and_then(|m| {
                    if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(pos)) =
                        m.attribute(Mesh::ATTRIBUTE_POSITION)
                    {
                        let hash = (entity.index() as f32 * 0.6180339) % 6.28318;
                        crate::dungeon::vegetation_animation::FoliageMeshDeformer::new(pos, hash)
                    } else {
                        None
                    }
                });

            let mut ent_cmd = commands.entity(entity);
            ent_cmd.insert((
                TallGrassClump,
                GrassBending {
                    base_rotation: base_rot,
                    current_bend: Vec2::ZERO,
                    bend_velocity: Vec2::ZERO,
                },
                crate::alchemy::Flammable::with_duration(2.5).with_center_offset(center_offset),
            ));
            if let Some(deformer) = deformer_opt {
                ent_cmd.insert(deformer);
            }
        }
    }
}

pub fn attach_mushroom_spore_emitters(
    mut commands: Commands,
    mushroom_caps: Query<
        (Entity, Option<&Name>, Option<&Mesh3d>, Option<&Transform>),
        (
            Or<(With<MushroomCap>, With<Name>)>,
            Without<MushroomSporeAttached>,
        ),
    >,
    meshes: Option<Res<Assets<Mesh>>>,
) {
    for (entity, name_opt, mesh_opt, tf_opt) in &mushroom_caps {
        let is_target = if let Some(name) = name_opt {
            let s = name.as_str();
            (s.contains("Cap") && (s.contains("Shroom") || s.contains("Mushroom")))
                || s.starts_with("Mushroom_Cap")
        } else {
            true
        };

        if is_target {
            let center_offset = mesh_opt
                .and_then(|h| meshes.as_ref().and_then(|m| m.get(&h.0)))
                .and_then(|m| m.compute_aabb())
                .map(|a| Vec3::from(a.center))
                .unwrap_or(Vec3::ZERO);

            let base_scale = tf_opt.map(|t| t.scale).unwrap_or(Vec3::ONE);
            let base_rotation = tf_opt.map(|t| t.rotation).unwrap_or(Quat::IDENTITY);
            let phase = (entity.index() as f32 * 0.6180339) % 6.28318;

            commands
                .entity(entity)
                .insert((
                    Visibility::default(),
                    InheritedVisibility::default(),
                    MushroomCap,
                    MushroomSporeAttached,
                    MushroomOrganicBreathing {
                        base_scale,
                        base_rotation,
                        phase_offset: phase,
                    },
                    BioluminescentMushroom {
                        base_intensity: 1.0,
                        base_color: Color::srgb(0.12, 0.96, 0.88),
                        phase: 0.0,
                        surge_timer: 0.0,
                    },
                    crate::alchemy::Flammable::with_duration(3.5).with_center_offset(center_offset),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Transform::from_xyz(center_offset.x, center_offset.y - 0.06, center_offset.z),
                        ParticleEmitter {
                            spawn_timer: 0.0,
                            spawn_rate: 0.22,
                            particle_type: ParticleType::GlowSpore,
                        },
                    ));
                });
        }
    }
}

pub fn mark_imported_flammable_props(
    mut commands: Commands,
    named_entities: Query<
        (Entity, &Name, Option<&Mesh3d>, Option<&Transform>),
        (
            Added<Name>,
            Without<crate::alchemy::Flammable>,
            Without<TallGrassClump>,
            Without<MushroomCap>,
        ),
    >,
    meshes: Option<Res<Assets<Mesh>>>,
) {
    for (entity, name, mesh_opt, tf_opt) in &named_entities {
        let s = name.as_str();
        let (duration, is_flammable) = if (s.contains("Stem") && (s.contains("Shroom") || s.contains("Mushroom")))
            || (s.contains("Shroom") || s.contains("Mushroom"))
        {
            (3.5, true)
        } else if s.contains("Fern") {
            (2.2, true)
        } else if s.contains("Vine") {
            (2.8, true)
        } else if s.contains("Wall_Cyan_Spore") {
            (1.8, true)
        } else if s.contains("Dandelion") {
            (2.0, true)
        } else {
            (0.0, false)
        };

        if is_flammable {
            let center_offset = mesh_opt
                .and_then(|h| meshes.as_ref().and_then(|m| m.get(&h.0)))
                .and_then(|m| m.compute_aabb())
                .map(|a| Vec3::from(a.center))
                .unwrap_or(Vec3::ZERO);

            let mut ent_cmd = commands.entity(entity);
            ent_cmd.insert(
                crate::alchemy::Flammable::with_duration(duration).with_center_offset(center_offset),
            );

            if s.contains("Dandelion") {
                let hash = (entity.index() as f32 * 0.6180339) % 6.28318;
                let rot = tf_opt.map(|t| t.rotation).unwrap_or(Quat::IDENTITY);
                let yaw = rot.to_euler(EulerRot::YXZ).0;
                ent_cmd.insert(crate::dungeon::flora::DandelionPlant {
                    sway_phase: hash,
                    base_yaw: yaw,
                    base_rotation: rot,
                });
            } else if (s.contains("Stem") && (s.contains("Shroom") || s.contains("Mushroom")))
                || (s.contains("Shroom") || s.contains("Mushroom"))
            {
                let base_scale = tf_opt.map(|t| t.scale).unwrap_or(Vec3::ONE);
                let base_rotation = tf_opt.map(|t| t.rotation).unwrap_or(Quat::IDENTITY);
                let phase = (entity.index() as f32 * 0.6180339) % 6.28318;
                ent_cmd.insert(MushroomOrganicBreathing {
                    base_scale,
                    base_rotation,
                    phase_offset: phase,
                });
            }
        }
    }
}

pub fn keep_tall_grass_clear_of_obstacles(
    mut _grass_query: Query<
        (
            &mut Transform,
            &GlobalTransform,
            Option<&crate::alchemy::Flammable>,
            Option<&Aabb>,
        ),
        With<TallGrassClump>,
    >,
    _obstacles: Query<
        (&GlobalTransform, &ObstacleShape),
        (With<CavernObstacle>, Without<TallGrassClump>),
    >,
) {
}

pub fn update_grass_bending(
    _time: Res<Time>,
    _player_query: Query<&GlobalTransform, (With<crate::player::camera::Player>, Without<TallGrassClump>)>,
    mut _grass_query: Query<
        (
            &mut Transform,
            &GlobalTransform,
            &mut GrassBending,
            Option<&crate::alchemy::Flammable>,
            Option<&Aabb>,
        ),
        With<TallGrassClump>,
    >,
) {
}

pub fn create_water_ring_mesh(radius: f32, inner_ratio: f32, segments: usize) -> Mesh {
    let inner_radius = radius * inner_ratio;
    let num_verts = (segments + 1) * 2;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_verts);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_verts);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_verts);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 6);

    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        positions.push([cos_a * radius, 0.0, sin_a * radius]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + 0.5 * cos_a, 0.5 + 0.5 * sin_a]);

        positions.push([cos_a * inner_radius, 0.0, sin_a * inner_radius]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + 0.5 * inner_ratio * cos_a, 0.5 + 0.5 * inner_ratio * sin_a]);
    }

    for i in 0..segments as u32 {
        let o1 = i * 2;
        let i1 = i * 2 + 1;
        let o2 = (i + 1) * 2;
        let i2 = (i + 1) * 2 + 1;

        indices.push(o1);
        indices.push(o2);
        indices.push(i1);

        indices.push(i1);
        indices.push(o2);
        indices.push(i2);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn spawn_water_ripple(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    initial_radius: f32,
    max_radius: f32,
    lifetime: f32,
    alpha: f32,
) {
    let ring_mesh = meshes.add(create_water_ring_mesh(1.0, 0.80, 32));
    let ring_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.85, 0.96, 1.0, alpha),
        emissive: LinearRgba::new(0.20, 0.65, 0.85, alpha * 0.45),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        depth_bias: 450.0,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        WaterRipple {
            elapsed: 0.0,
            max_lifetime: lifetime,
            initial_radius,
            max_radius,
            initial_alpha: alpha,
        },
        Mesh3d(ring_mesh),
        MeshMaterial3d(ring_mat),
        Transform::from_translation(position).with_scale(Vec3::new(initial_radius, 1.0, initial_radius)),
    ));
}

pub fn update_water_ripples(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ripples: Query<(
        Entity,
        &mut WaterRipple,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    for (entity, mut ripple, mut tf, mat_handle) in ripples.iter_mut() {
        ripple.elapsed += dt;
        if ripple.elapsed >= ripple.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = (ripple.elapsed / ripple.max_lifetime).clamp(0.0, 1.0);
        let current_radius = ripple.initial_radius + (ripple.max_radius - ripple.initial_radius) * (1.0 - (1.0 - progress).powi(2));
        tf.scale = Vec3::new(current_radius, 1.0, current_radius);

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let alpha = ripple.initial_alpha * (1.0 - progress);
            mat.base_color = Color::srgba(0.85, 0.96, 1.0, alpha);
        }
    }
}

pub fn update_bioluminescent_mushrooms(
    time: Res<Time>,
    config: Option<Res<crate::dungeon::vegetation_animation::VegetationAnimationConfig>>,
    player_query: Query<&GlobalTransform, (With<crate::player::camera::Player>, Without<BioluminescentMushroom>)>,
    puddles_query: Query<&GlobalTransform, (With<crate::alchemy::PotionPuddle>, Without<BioluminescentMushroom>)>,
    mut mushrooms: Query<(
        Entity,
        &GlobalTransform,
        &mut BioluminescentMushroom,
        Option<&mut PointLight>,
        Option<(&MushroomOrganicBreathing, &mut Transform)>,
    ), Without<crate::player::camera::Player>>,
    mut passive_shrooms: Query<(
        &MushroomOrganicBreathing,
        &mut Transform,
    ), (Without<BioluminescentMushroom>, Without<crate::player::camera::Player>)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let default_prof = crate::dungeon::vegetation_animation::MushroomAnimationProfile::default();
    let m_prof = config
        .as_ref()
        .map(|c| &c.mushroom_profile)
        .unwrap_or(&default_prof);

    let elapsed = time.elapsed_secs();
    let player_pos = player_query.get_single().map(|gt| gt.translation()).unwrap_or(Vec3::splat(9999.0));

    for (_entity, gt, mut shroom, light_opt, mut breathing_opt) in mushrooms.iter_mut() {
        let shroom_pos = gt.translation();

        let dist_to_player = shroom_pos.distance(player_pos);
        if dist_to_player < m_prof.player_reaction_radius {
            shroom.surge_timer = shroom.surge_timer.max(2.5);
        }

        for p_gt in puddles_query.iter() {
            if p_gt.translation().distance(shroom_pos) < 4.0 {
                shroom.surge_timer = shroom.surge_timer.max(3.0);
                break;
            }
        }

        let is_surging = shroom.surge_timer > 0.0;
        if is_surging {
            shroom.surge_timer = (shroom.surge_timer - dt).max(0.0);
        }

        let frequency = if is_surging {
            m_prof.idle_pulse_speed * m_prof.player_reaction_multiplier
        } else {
            m_prof.idle_pulse_speed
        };
        shroom.phase += std::f32::consts::TAU * frequency * dt;

        let wave = shroom.phase.sin();
        let amp = if is_surging { 0.40 } else { 0.18 };
        let intensity_mult = 1.0 + wave * amp;

        if let Some(mut light) = light_opt {
            light.intensity = shroom.base_intensity * intensity_mult;
            let hue_shift = (wave * 0.08).clamp(-0.1, 0.1);
            let base_srgb = shroom.base_color.to_srgba();
            light.color = Color::srgb(
                (base_srgb.red + hue_shift * 0.1).clamp(0.05, 0.4),
                (base_srgb.green + hue_shift * 0.05).clamp(0.7, 1.0),
                (base_srgb.blue + hue_shift * 0.12).clamp(0.7, 1.0),
            );
        }

        if let Some((breathing, ref mut tf)) = breathing_opt {
            let squash_amp = if is_surging {
                m_prof.idle_squash_amplitude * 1.6
            } else {
                m_prof.idle_squash_amplitude
            };
            let sway_amp = if is_surging {
                m_prof.idle_sway_amplitude * 1.5
            } else {
                m_prof.idle_sway_amplitude
            };

            let phase = elapsed * frequency * std::f32::consts::TAU + breathing.phase_offset;
            let w = phase.sin();
            let squash = w * squash_amp;

            tf.scale = Vec3::new(
                breathing.base_scale.x * (1.0 - squash * 0.5),
                breathing.base_scale.y * (1.0 + squash),
                breathing.base_scale.z * (1.0 - squash * 0.5),
            );

            let pitch = (phase * 0.7).cos() * sway_amp;
            let roll = (phase * 0.5).sin() * (sway_amp * 0.75);
            tf.rotation = breathing.base_rotation * Quat::from_rotation_x(pitch) * Quat::from_rotation_z(roll);
        }

        if is_surging && wave > 0.95 && crate::render::particles::fastrand::f32() < 0.25 {
            commands.spawn((
                Transform::from_translation(shroom_pos + Vec3::new(
                    (crate::render::particles::fastrand::f32() - 0.5) * 0.6,
                    -0.15,
                    (crate::render::particles::fastrand::f32() - 0.5) * 0.6,
                )),
                ParticleEmitter {
                    spawn_timer: 0.0,
                    spawn_rate: 0.04,
                    particle_type: ParticleType::GlowSpore,
                },
                crate::alchemy::AlchemyTransientEmitter {
                    elapsed: 0.0,
                    max_lifetime: 1.5,
                },
            ));
        }
    }

    for (breathing, mut tf) in passive_shrooms.iter_mut() {
        let phase = elapsed * m_prof.idle_pulse_speed * std::f32::consts::TAU + breathing.phase_offset;
        let w = phase.sin();
        let squash = w * m_prof.idle_squash_amplitude;

        tf.scale = Vec3::new(
            breathing.base_scale.x * (1.0 - squash * 0.5),
            breathing.base_scale.y * (1.0 + squash),
            breathing.base_scale.z * (1.0 - squash * 0.5),
        );

        let pitch = (phase * 0.7).cos() * m_prof.idle_sway_amplitude;
        let roll = (phase * 0.5).sin() * (m_prof.idle_sway_amplitude * 0.75);
        tf.rotation = breathing.base_rotation * Quat::from_rotation_x(pitch) * Quat::from_rotation_z(roll);
    }
}

pub fn setup_cavern_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<SceneAssetRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    _moss_materials: ResMut<Assets<CelShadedMossMaterial>>,
) {

    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.10, 0.20, 0.32),
        brightness: 380.0,
    });

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.75, 0.92, 1.0),
            illuminance: 6200.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(1.5, 22.0, -4.5).looking_at(Vec3::new(0.0, 1.2, -8.5), Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.96, 0.84, 0.48),
            intensity: 7500.0,
            range: 15.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 3.2, 0.8),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.20, 0.94, 0.88),
            intensity: 14500.0,
            range: 9.5,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(2.8, 1.3, -0.5),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.25, 0.70, 0.98),
            intensity: 38000.0,
            range: 24.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 5.5, -11.2),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.12, 0.96, 0.88),
            intensity: 22000.0,
            range: 16.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-3.4, 2.4, -2.8),
        BioluminescentMushroom {
            base_intensity: 22000.0,
            base_color: Color::srgb(0.12, 0.96, 0.88),
            phase: 0.0,
            surge_timer: 0.0,
        },
        KeyShadowLight,
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.12, 0.96, 0.88),
            intensity: 22000.0,
            range: 16.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.1, 2.4, -3.3),
        BioluminescentMushroom {
            base_intensity: 22000.0,
            base_color: Color::srgb(0.12, 0.96, 0.88),
            phase: 1.57,
            surge_timer: 0.0,
        },
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.35, 0.90, 1.0),
            intensity: 26000.0,
            range: 18.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 1.8, -10.2),
        KeyShadowLight,
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.40, 0.95, 1.0),
            intensity: 30000.0,
            range: 22.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 13.5, -11.2),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.24, 0.38, 0.55),
            intensity: 36000.0,
            range: 29.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 8.0, -11.5),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.25, 0.45, 0.65),
            intensity: 18000.0,
            range: 18.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-3.8, 4.2, -4.2),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.25, 0.45, 0.65),
            intensity: 18000.0,
            range: 18.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(3.8, 4.2, -4.2),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.18, 0.92, 0.98),
            intensity: 22000.0,
            range: 18.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 1.8, -8.5),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.20, 0.45, 0.65),
            intensity: 32000.0,
            range: 28.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 18.0, -5.5),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.17, 0.95, 0.88),
            intensity: 19000.0,
            range: 22.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-4.5, 17.0, -7.0),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.17, 0.95, 0.88),
            intensity: 19000.0,
            range: 22.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.5, 17.0, -7.0),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.55, 0.95, 1.0),
            intensity: 38000.0,
            range: 30.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 21.0, -9.5),
    ));

    commands.spawn((
        SceneAssetInstance {
            asset_key: "cavern_grotto".to_string(),
        },
        SceneRoot(registry.load_scene(&asset_server, "cavern_grotto")),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.22, 0.92, 0.96),
            intensity: 5200.0,
            range: 16.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 4.0, 2.2),
    ));

    for (name, x) in [
        ("Striated_Cliff_Left", -5.25),
        ("Striated_Cliff_Right", 5.25),
    ] {
        commands.spawn((
            Name::new(format!("Collider_{name}")),
            CavernObstacle,
            ObstacleShape::Cuboid {
                half_extents: Vec3::new(2.45, 5.25, 0.50),
            },
            Transform::from_xyz(x, 5.50, -11.45),
            RigidBody::Static,
            Collider::cuboid(4.9, 10.5, 1.0),
        ));
    }

    commands.spawn((
        Name::new("Collider_Floor_Path_Entrance"),
        Transform::from_xyz(0.0, -1.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(16.0, 2.0, 10.0),
    ));

    commands.spawn((
        Name::new("Collider_Floor_Shore_Left"),
        Transform::from_xyz(-6.5, -1.0, -8.5),
        RigidBody::Static,
        Collider::cuboid(6.0, 2.0, 18.0),
    ));

    commands.spawn((
        Name::new("Collider_Floor_Shore_Right"),
        Transform::from_xyz(6.5, -1.0, -8.5),
        RigidBody::Static,
        Collider::cuboid(6.0, 2.0, 18.0),
    ));

    commands.spawn((
        Name::new("Collider_Floor_Waterfall_Rear"),
        Transform::from_xyz(0.0, -1.0, -14.5),
        RigidBody::Static,
        Collider::cuboid(18.0, 2.0, 6.0),
    ));

    commands.spawn((
        Name::new("Collider_Floor_Lake_Bed"),
        Transform::from_xyz(0.0, -4.2, -8.5),
        RigidBody::Static,
        Collider::cuboid(12.0, 2.0, 12.0),
    ));

    commands.spawn((
        Name::new("Collider_Mushroom_Left_Large"),
        CavernObstacle,
        ObstacleShape::Cylinder {
            radius: 0.65,
            height: 2.70,
        },
        Transform::from_xyz(-3.50, 1.30, -3.30),
        RigidBody::Static,
        Collider::cylinder(0.65, 2.70),
    ));
    commands.spawn((
        Name::new("Collider_Mushroom_Left_Medium"),
        CavernObstacle,
        ObstacleShape::Cylinder {
            radius: 0.48,
            height: 1.90,
        },
        Transform::from_xyz(-2.89, 0.90, -2.52),
        RigidBody::Static,
        Collider::cylinder(0.48, 1.90),
    ));
    commands.spawn((
        Name::new("Collider_Mushroom_Left_Small"),
        CavernObstacle,
        ObstacleShape::Cylinder {
            radius: 0.38,
            height: 1.25,
        },
        Transform::from_xyz(-4.07, 0.60, -2.20),
        RigidBody::Static,
        Collider::cylinder(0.38, 1.25),
    ));

    commands.spawn((
        Name::new("Collider_Mushroom_Right_1"),
        CavernObstacle,
        ObstacleShape::Cylinder {
            radius: 0.48,
            height: 1.70,
        },
        Transform::from_xyz(4.17, 0.85, -3.53),
        RigidBody::Static,
        Collider::cylinder(0.48, 1.70),
    ));
    commands.spawn((
        Name::new("Collider_Mushroom_Right_2"),
        CavernObstacle,
        ObstacleShape::Cylinder {
            radius: 0.40,
            height: 1.40,
        },
        Transform::from_xyz(3.58, 0.70, -2.80),
        RigidBody::Static,
        Collider::cylinder(0.40, 1.40),
    ));
    commands.spawn((
        Name::new("Collider_Mushroom_Right_3"),
        CavernObstacle,
        ObstacleShape::Cylinder {
            radius: 0.32,
            height: 1.05,
        },
        Transform::from_xyz(4.73, 0.50, -3.98),
        RigidBody::Static,
        Collider::cylinder(0.32, 1.05),
    ));

    for (name, x, z, hx, hz) in [
        ("Collider_Rock_Terrace_Left_Fore", -6.4, 1.0, 1.6, 2.3),
        ("Collider_Rock_Terrace_Right_Fore", 6.4, 1.0, 1.6, 2.3),
        ("Collider_Rock_Terrace_Left_Mid", -6.1, -3.1, 1.6, 2.1),
        ("Collider_Rock_Terrace_Right_Mid", 6.1, -3.1, 1.6, 2.1),
        ("Collider_Rock_Terrace_Left_Back", -6.6, -6.8, 1.7, 1.9),
        ("Collider_Rock_Terrace_Right_Back", 6.6, -6.8, 1.7, 1.9),
        ("Collider_Rock_Terrace_Left_Deep", -6.4, -9.4, 1.6, 1.9),
        ("Collider_Rock_Terrace_Right_Deep", 6.4, -9.4, 1.6, 1.9),
    ] {
        commands.spawn((
            Name::new(name),
            CavernObstacle,
            ObstacleShape::Cuboid {
                half_extents: Vec3::new(hx, 3.5, hz),
            },
            Transform::from_xyz(x, 2.5, z),
            RigidBody::Static,
            Collider::cuboid(hx * 2.0, 7.0, hz * 2.0),
        ));
    }

    for (name, x, z, radius, height) in [
        ("Collider_Transition_Rock_Left_01", -2.4, -1.1, 0.58, 0.32),
        ("Collider_Transition_Rock_Left_02", -2.5, -2.3, 0.64, 0.32),
        ("Collider_Transition_Rock_Left_03", -2.7, -3.4, 0.70, 0.32),
        ("Collider_Transition_Rock_Right_01", 2.3, -1.0, 0.56, 0.32),
        ("Collider_Transition_Rock_Right_02", 2.4, -2.2, 0.62, 0.32),
        ("Collider_Transition_Rock_Right_03", 2.6, -3.3, 0.68, 0.32),
        ("Collider_Transition_Rock_Fore_Left_A", -2.3, 0.8, 0.56, 0.32),
        ("Collider_Transition_Rock_Fore_Left_B", -2.4, -0.2, 0.60, 0.32),
        ("Collider_Transition_Rock_Fore_Right_A", 2.3, 0.8, 0.54, 0.32),
        ("Collider_Transition_Rock_Fore_Right_B", 2.4, -0.2, 0.58, 0.32),
    ] {
        commands.spawn((
            Name::new(name),
            CavernObstacle,
            ObstacleShape::Cylinder { radius, height },
            Transform::from_xyz(x, height * 0.5, z),
            RigidBody::Static,
            Collider::cylinder(radius, height),
        ));
    }

    for (name, x, z, radius, height) in [
        ("Collider_Side_Rock_Left_Front", -5.15, -4.8, 1.15, 3.6),
        ("Collider_Side_Rock_Left_Low", -5.60, -5.8, 1.25, 3.6),
        ("Collider_Side_Rock_Left_Back", -5.70, -7.2, 1.25, 3.6),
        ("Collider_Waterfall_Cliff_Tower_Left", -4.65, -10.60, 1.65, 8.0),
        ("Collider_Side_Rock_Right_Front", 5.15, -4.8, 1.15, 3.6),
        ("Collider_Side_Rock_Right_Low", 5.60, -5.8, 1.25, 3.6),
        ("Collider_Side_Rock_Right_Back", 5.70, -7.2, 1.25, 3.6),
        ("Collider_Waterfall_Cliff_Tower_Right", 4.65, -10.60, 1.65, 8.0),
    ] {
        commands.spawn((
            Name::new(name),
            CavernObstacle,
            ObstacleShape::Cylinder { radius, height },
            Transform::from_xyz(x, height * 0.5, z),
            RigidBody::Static,
            Collider::cylinder(radius, height),
        ));
    }

    commands.spawn((
        Name::new("Collider_Canyon_Wall_Left"),
        CavernObstacle,
        ObstacleShape::Cuboid {
            half_extents: Vec3::new(1.4, 5.0, 9.0),
        },
        Transform::from_xyz(-7.8, 3.5, -4.5),
        RigidBody::Static,
        Collider::cuboid(2.8, 10.0, 18.0),
    ));
    commands.spawn((
        Name::new("Collider_Canyon_Wall_Right"),
        CavernObstacle,
        ObstacleShape::Cuboid {
            half_extents: Vec3::new(1.4, 5.0, 9.0),
        },
        Transform::from_xyz(7.8, 3.5, -4.5),
        RigidBody::Static,
        Collider::cuboid(2.8, 10.0, 18.0),
    ));

    commands.spawn((
        Name::new("Collider_Cavern_Wall_Rear_Left"),
        CavernObstacle,
        ObstacleShape::Cuboid {
            half_extents: Vec3::new(3.15, 4.0, 1.0),
        },
        Transform::from_xyz(-5.35, 3.0, 4.0),
        RigidBody::Static,
        Collider::cuboid(6.3, 8.0, 2.0),
    ));
    commands.spawn((
        Name::new("Collider_Cavern_Wall_Rear_Right"),
        CavernObstacle,
        ObstacleShape::Cuboid {
            half_extents: Vec3::new(3.15, 4.0, 1.0),
        },
        Transform::from_xyz(5.35, 3.0, 4.0),
        RigidBody::Static,
        Collider::cuboid(6.3, 8.0, 2.0),
    ));

    let (waterfall_mesh, waterfall_base) = create_waterfall_mesh(32, 48);
    commands.spawn((
        Mesh3d(meshes.add(waterfall_mesh)),
        MeshMaterial3d(water_materials.add(WaterMaterial {
            base_color: LinearRgba::new(0.10, 0.67, 0.78, 0.25),
            deep_color: LinearRgba::new(0.025, 0.20, 0.33, 0.36),
            foam_color: LinearRgba::new(0.82, 0.93, 0.96, 0.46),
            params: Vec4::new(0.0, 1.15, 1.0, 0.22),
            reflection_texture: None,
            reflection_params: Vec4::new(0.18, 0.0, 0.0, 0.0),
        })),
        WaterfallMarker,
        waterfall_base,
        bevy::pbr::NotShadowCaster,
        bevy::pbr::NotShadowReceiver,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(create_water_disk_mesh(5.85, 64))),
        MeshMaterial3d(water_materials.add(WaterMaterial {
            base_color: LinearRgba::new(0.10, 0.62, 0.70, 0.16),
            deep_color: LinearRgba::new(0.012, 0.055, 0.12, 0.88),
            foam_color: LinearRgba::new(0.84, 0.94, 0.96, 0.74),
            params: Vec4::new(0.0, 0.82, 0.0, 0.26),
            reflection_texture: None,
            reflection_params: Vec4::new(0.18, 0.045, 0.85, 0.70),
        })),
        bevy::pbr::NotShadowCaster,
        bevy::pbr::NotShadowReceiver,
        LakeWaterPool,
        Transform::from_xyz(0.0, 0.18, -8.5),
    ));

    for x_off in [-3.6, -2.4, -1.2, 0.0, 1.2, 2.4, 3.6] {
        commands.spawn((
            Transform::from_xyz(x_off, 0.22, -8.55),
            ParticleEmitter {
                spawn_timer: 0.0,
                spawn_rate: 0.08,
                particle_type: ParticleType::WaterSplash,
            },
        ));
        commands.spawn((
            Transform::from_xyz(x_off * 0.8, 0.28, -8.7),
            ParticleEmitter {
                spawn_timer: 0.0,
                spawn_rate: 0.10,
                particle_type: ParticleType::Steam,
            },
        ));
    }

    commands.spawn((
        Interactable {
            label: "Cyan Parasol Mushroom",
        },
        AlchemicalIngredient {
            id: "ing_cyan_spore".to_string(),
            name: "Glow Spore".to_string(),
            primary_effect: AlchemicalEffectKind::Luminescence,
            secondary_effect: AlchemicalEffectKind::Swiftness,
            harvested: false,
        },
        Transform::from_xyz(-3.6, 0.5, -3.2),
        GlobalTransform::default(),
        Collider::cylinder(0.4, 0.8),
        crate::alchemy::Flammable::with_duration(3.0),
    ));

    commands.spawn((
        Transform::from_xyz(-5.8, 0.5, -5.8),
        ThermalGeyser {
            eruption_timer: 0.0,
            eruption_interval: 3.5,
            is_erupting: true,
            heat_radius: 5.0,
        },
        ParticleEmitter {
            spawn_timer: 0.0,
            spawn_rate: 0.08,
            particle_type: ParticleType::Steam,
        },
    ));

    let bench_position = Vec3::new(2.90, 0.03, -1.05);
    let bench_rotation = Quat::from_rotation_y(-0.45);
    commands.spawn((
        Name::new("Rustic_Mossy_Bench"),
        SceneAssetInstance {
            asset_key: "prop_wooden_bench".to_string(),
        },
        SceneRoot(registry.load_scene(&asset_server, "prop_wooden_bench")),
        Transform::from_translation(bench_position).with_rotation(bench_rotation),
        crate::alchemy::Flammable::with_duration(4.5),
    ));

    let bench_colliders = [
        (
            "Collider_Bench_Seat",
            Vec3::new(0.0, 0.58, 0.0),
            Vec3::new(1.65, 0.18, 0.34),
        ),
        (
            "Collider_Bench_Backrest",
            Vec3::new(0.0, 1.00, 0.28),
            Vec3::new(1.65, 0.38, 0.12),
        ),
        (
            "Collider_Bench_Leg_FL",
            Vec3::new(-0.62, 0.28, 0.20),
            Vec3::new(0.14, 0.56, 0.14),
        ),
        (
            "Collider_Bench_Leg_FR",
            Vec3::new(0.62, 0.28, 0.20),
            Vec3::new(0.14, 0.56, 0.14),
        ),
        (
            "Collider_Bench_Leg_BL",
            Vec3::new(-0.62, 0.28, -0.20),
            Vec3::new(0.14, 0.56, 0.14),
        ),
        (
            "Collider_Bench_Leg_BR",
            Vec3::new(0.62, 0.28, -0.20),
            Vec3::new(0.14, 0.56, 0.14),
        ),
    ];
    for (name, local_offset, dimensions) in bench_colliders {
        commands.spawn((
            Name::new(name),
            CavernObstacle,
            BenchObstacle,
            ObstacleShape::Cuboid {
                half_extents: dimensions * 0.5,
            },
            Transform::from_translation(bench_position + bench_rotation * local_offset)
                .with_rotation(bench_rotation),
            RigidBody::Static,
            Collider::cuboid(dimensions.x, dimensions.y, dimensions.z),
        ));
    }

    info!("Masterwork Cel-Shaded Cavern Grotto initialized with Volumetric Waterfall, Organic Mushrooms, and Sobel Outlines!");
}

pub fn create_water_disk_mesh(radius: f32, segments: usize) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(segments + 2);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(segments + 2);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(segments + 2);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 3);

    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        positions.push([x, 0.0, z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);
    }

    for i in 1..=segments as u32 {
        indices.push(0);
        indices.push(i + 1);
        indices.push(i);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn create_waterfall_mesh(u_segs: usize, v_segs: usize) -> (Mesh, WaterfallBaseMesh) {
    let num_verts = (u_segs + 1) * (v_segs + 1);
    let mut positions = Vec::with_capacity(num_verts);
    let mut normals = Vec::with_capacity(num_verts);
    let mut uvs = Vec::with_capacity(num_verts);
    let mut indices = Vec::with_capacity(u_segs * v_segs * 12);

    let h_top = 17.5;
    let h_bottom = 0.06;
    let z_top = -10.8;
    let z_bottom = -8.5;
    let width_top = 4.2;
    let width_bottom = 6.4;

    for iv in 0..=v_segs {
        let v_frac = iv as f32 / v_segs as f32;
        let y_calc = h_bottom + (1.0 - v_frac) * (h_top - h_bottom);

        let curr_z_center = z_top + v_frac.powf(1.4) * (z_bottom - z_top);
        let curr_width = width_top + v_frac * (width_bottom - width_top);

        for iu in 0..=u_segs {
            let u_frac = iu as f32 / u_segs as f32;
            let u_norm = (u_frac - 0.5) * 2.0;
            let x = u_norm * (curr_width * 0.5);

            let convex_arc = (1.0 - u_norm * u_norm) * 0.52;

            let fluting = (u_norm * std::f32::consts::PI * 8.0).sin() * 0.08;
            let z = curr_z_center + convex_arc + fluting;

            let bottom_scallop = if iv == v_segs {
                (u_frac * std::f32::consts::PI).sin() * 0.08
            } else {
                0.0
            };
            let y = y_calc - bottom_scallop;

            positions.push([x, y, z]);

            let nx = -u_norm * 0.45;
            let ny = 0.15;
            let nz = 1.0;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            normals.push([nx / len, ny / len, nz / len]);

            uvs.push([u_frac, v_frac]);
        }
    }

    for iv in 0..v_segs as u32 {
        for iu in 0..u_segs as u32 {
            let row_stride = (u_segs + 1) as u32;
            let v1 = iv * row_stride + iu;
            let v2 = iv * row_stride + iu + 1;
            let v3 = (iv + 1) * row_stride + iu + 1;
            let v4 = (iv + 1) * row_stride + iu;

            indices.push(v1);
            indices.push(v4);
            indices.push(v2);

            indices.push(v2);
            indices.push(v4);
            indices.push(v3);

            indices.push(v1);
            indices.push(v2);
            indices.push(v4);

            indices.push(v2);
            indices.push(v3);
            indices.push(v4);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    let base_mesh = WaterfallBaseMesh {
        base_positions: positions,
        base_normals: normals,
    };

    (mesh, base_mesh)
}

pub fn animate_waterfall_mesh(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(&Mesh3d, &WaterfallBaseMesh), With<WaterfallMarker>>,
) {
    let t = time.elapsed_secs();
    for (mesh_handle, base) in &query {
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            let mut new_positions = Vec::with_capacity(base.base_positions.len());
            for (pos, norm) in base.base_positions.iter().zip(base.base_normals.iter()) {
                let wave =
                    (pos[1] * 4.0 - t * 5.0).sin() * 0.05 + (pos[0] * 3.0 + t * 2.5).cos() * 0.025;
                let displaced = [
                    pos[0] + norm[0] * wave,
                    pos[1] + norm[1] * wave * 0.5,
                    pos[2] + norm[2] * wave,
                ];
                new_positions.push(displaced);
            }
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_positions);
        }
    }
}

pub fn get_grotto_terrain_height(x: f32, z: f32) -> f32 {
    let lake_center_x = 0.0;
    let lake_center_z = -8.5;
    let lake_radius = 5.3;
    let dx = x - lake_center_x;
    let dz = z - lake_center_z;
    let dist_lake = (dx * dx + dz * dz).sqrt();

    if dist_lake < lake_radius {
        let u = dist_lake / lake_radius;
        let funnel_depth = (1.0 - u).max(0.0).powf(1.75) * 3.10 + (1.0 - u) * 0.35;
        0.04 - funnel_depth
    } else if dist_lake < lake_radius + 1.8 {
        let rim_blend = (dist_lake - lake_radius) / 1.8;
        0.04 + rim_blend * 0.03
    } else {
        0.04
    }
}

pub fn is_in_lake_water(pos: Vec3) -> bool {
    let dx = pos.x;
    let dz = pos.z - (-8.5);
    let dist = (dx * dx + dz * dz).sqrt();
    dist <= 5.80 && pos.y <= 0.22
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_dungeon_plugin_build() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Mesh>();
        app.init_asset::<StandardMaterial>();
        app.init_asset::<WaterMaterial>();
        app.init_asset::<bevy::scene::Scene>();
        app.add_plugins(DungeonPlugin);
        assert!(app.is_plugin_added::<DungeonPlugin>());
    }

    #[test]
    fn test_setup_cavern_world_spawns_environment() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Mesh>();
        app.init_asset::<StandardMaterial>();
        app.init_asset::<WaterMaterial>();
        app.init_asset::<CelShadedMossMaterial>();
        app.init_asset::<bevy::scene::Scene>();
        app.init_resource::<SceneAssetRegistry>();
        app.add_systems(Startup, setup_cavern_world);
        app.update();

        let mut light_query = app.world_mut().query::<&PointLight>();
        let light_count = light_query.iter(app.world()).count();
        assert!(light_count >= 9, "Must spawn at least 9 point lights for bioluminescence, waterfall and high ceiling vault");

        let mut obstacle_query = app
            .world_mut()
            .query::<(&CavernObstacle, &Collider, &RigidBody)>();
        let obstacle_count = obstacle_query.iter(app.world()).count();
        assert!(obstacle_count >= 9, "Must spawn at least 9 static colliders for mushroom stems, rock terraces and rustic mossy bench");

        let mut emitter_query = app.world_mut().query::<&ParticleEmitter>();
        let emitter_count = emitter_query.iter(app.world()).count();
        assert!(
            emitter_count >= 4,
            "Must spawn particle emitters for waterfall foam and mushroom spores"
        );

        let mut scene_query = app.world_mut().query::<&SceneRoot>();
        let scene_count = scene_query.iter(app.world()).count();
        assert!(
            scene_count >= 2,
            "Must spawn cavern_kit and alchemy_props scenes"
        );

        let mut waterfall_query = app.world_mut().query::<&WaterfallMarker>();
        let waterfall_count = waterfall_query.iter(app.world()).count();
        assert_eq!(waterfall_count, 1, "Must spawn dynamic waterfall entity");
    }

    #[test]
    fn test_modular_kit_asset_registry_loaded_and_mapped() {
        let registry = SceneAssetRegistry::default();
        assert!(registry.get_path("env_cave_wall").is_some());
        assert!(registry.get_path("prop_wooden_bench").is_some());
        assert!(registry.get_path("cavern_grotto").is_some());
    }

    #[test]
    fn test_create_water_disk_mesh_attributes_and_topology() {
        let mesh = create_water_disk_mesh(5.0, 32);
        assert_eq!(mesh.primitive_topology(), PrimitiveTopology::TriangleList);
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
        assert!(mesh.indices().is_some());
    }

    #[test]
    fn test_create_waterfall_mesh_attributes_and_curvature() {
        let (mesh, base) = create_waterfall_mesh(16, 24);
        assert_eq!(mesh.primitive_topology(), PrimitiveTopology::TriangleList);
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
        assert!(mesh.indices().is_some());
        assert_eq!(base.base_positions.len(), (16 + 1) * (24 + 1));
        assert_eq!(base.base_normals.len(), (16 + 1) * (24 + 1));
    }

    #[test]
    fn test_mushroom_cap_spore_emitter_dynamic_attachment_and_repositioning() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, attach_mushroom_spore_emitters);

        let shroom = app
            .world_mut()
            .spawn((
                Name::new("Left_Shroom_Large_Cap"),
                Transform::from_xyz(-3.5, 2.0, -3.5),
            ))
            .id();

        app.update();

        let mut query = app.world_mut().query::<(&ParticleEmitter, &Parent)>();
        let attached = query.iter(app.world()).any(|(emitter, parent)| {
            parent.get() == shroom && emitter.particle_type == ParticleType::GlowSpore
        });
        assert!(
            attached,
            "Must dynamically attach GlowSpore particle emitter to newly spawned mushroom cap"
        );

        let mut tf_query = app.world_mut().query::<&mut Transform>();
        if let Ok(mut tf) = tf_query.get_mut(app.world_mut(), shroom) {
            tf.translation = Vec3::new(10.0, 4.0, 10.0);
        }

        let new_shroom = app
            .world_mut()
            .spawn((MushroomCap, Transform::from_xyz(5.0, 3.0, 5.0)))
            .id();

        app.update();

        let mut query = app.world_mut().query::<(&ParticleEmitter, &Parent)>();
        let attached_new = query.iter(app.world()).any(|(emitter, parent)| {
            parent.get() == new_shroom && emitter.particle_type == ParticleType::GlowSpore
        });
        assert!(
            attached_new,
            "Must dynamically attach GlowSpore emitter to newly added MushroomCap"
        );
    }
}
