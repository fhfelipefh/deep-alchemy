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
    pub base_translation: Vec3,
    pub pivot: Vec3,
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
            .init_resource::<crate::dungeon::fauna::AmbientFaunaSpawner>()
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
                mark_imported_lake_features,
                update_water_ripples,
                update_bioluminescent_mushrooms,
                (
                    crate::dungeon::fauna::update_spider_webs,
                    crate::dungeon::fauna::update_ambient_insects,
                ).chain(),
                crate::dungeon::fauna::update_ambient_fauna_spawner,
                (
                    crate::dungeon::water_drips::update_ceiling_drip_points,
                    crate::dungeon::water_drips::update_falling_water_drops,
                    crate::dungeon::water_drips::update_puddle_player_interaction,
                    crate::dungeon::water_drips::update_puddle_temperature_reactions,
                    crate::dungeon::water_drips::update_puddle_boiling_visuals,
                ).chain(),
                crate::dungeon::flora::animate_dandelions,
                crate::dungeon::vegetation_animation::update_vegetation_foliage_animation,
                (
                    crate::dungeon::creatures::update_creature_patrol_and_movement,
                    crate::dungeon::creatures::update_creature_ai_perception_and_aim_dodge,
                    crate::dungeon::creatures::update_creature_potion_interactions,
                ).chain(),
                crate::dungeon::generator::update_forest_victory_trigger,
            ),
        )
        .add_systems(
            Update,
            (
                crate::dungeon::traps::update_cracked_wall_nests,
                crate::dungeon::traps::update_emergent_creatures,
                crate::dungeon::traps::update_dungeon_traps,
                crate::dungeon::traps::update_creatures_trap_tactics,
                crate::dungeon::traps::update_potion_traps_and_nests_interactions,
            ),
        );
    }
}

pub fn mark_imported_lake_features(
    mut commands: Commands,
    named_entities: Query<(Entity, &Name), Added<Name>>,
) {
    for (entity, name) in &named_entities {
        let name_str = name.as_str();
        if name_str == "Lake_Fine_Rim"
            || name_str.starts_with("WaterRipple_")
            || name_str == "WaterRipple_1"
            || name_str == "WaterRipple_2"
            || name_str == "WaterRipple_3"
        {
            commands.entity(entity).despawn();
        }
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
            let tf_pos = tf_opt.map(|t| t.translation).unwrap_or(Vec3::ZERO);
            let center_offset = mesh_opt
                .and_then(|h| meshes.as_ref().and_then(|m| m.get(&h.0)))
                .and_then(|m| m.compute_aabb())
                .map(|a| Vec3::from(a.center))
                .unwrap_or(Vec3::ZERO);
            let world_center = tf_pos + center_offset;
            if is_in_lake_water(world_center)
                || (world_center.x * world_center.x + (world_center.z - (-8.5)) * (world_center.z - (-8.5))).sqrt() < 6.2
            {
                commands.entity(entity).despawn();
                continue;
            }

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
                crate::alchemy::Flammable::with_duration(2.5)
                    .with_center_offset(center_offset)
                    .with_ignition_temperature(15.0),
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
            let base_translation = tf_opt.map(|t| t.translation).unwrap_or(Vec3::ZERO);
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
                        base_translation,
                        pivot: center_offset,
                        phase_offset: phase,
                    },
                    BioluminescentMushroom {
                        base_intensity: 1.0,
                        base_color: Color::srgb(0.12, 0.96, 0.88),
                        phase: 0.0,
                        surge_timer: 0.0,
                    },
                    crate::alchemy::Flammable::with_duration(3.5)
                        .with_center_offset(center_offset)
                        .with_ignition_temperature(22.0),
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

            let threshold = if s.contains("Dandelion") {
                15.0
            } else if s.contains("Fern") {
                18.0
            } else if s.contains("Vine") {
                20.0
            } else if s.contains("Stem") {
                35.0
            } else {
                20.0
            };

            let mut ent_cmd = commands.entity(entity);
            ent_cmd.insert(
                crate::alchemy::Flammable::with_duration(duration)
                    .with_center_offset(center_offset)
                    .with_ignition_temperature(threshold),
            );

            if s.contains("Dandelion") {
                let hash = (entity.index() as f32 * 0.6180339) % 6.28318;
                let rot = tf_opt.map(|t| t.rotation).unwrap_or(Quat::IDENTITY);
                let trans = tf_opt.map(|t| t.translation).unwrap_or(Vec3::ZERO);
                let yaw = rot.to_euler(EulerRot::YXZ).0;
                let min_y = mesh_opt
                    .and_then(|h| meshes.as_ref().and_then(|m| m.get(&h.0)))
                    .and_then(|m| m.compute_aabb())
                    .map(|a| a.center[1] - a.half_extents[1])
                    .unwrap_or(center_offset.y);
                let pivot = Vec3::new(center_offset.x, min_y, center_offset.z);
                ent_cmd.insert(crate::dungeon::flora::DandelionPlant {
                    sway_phase: hash,
                    base_yaw: yaw,
                    base_rotation: rot,
                    base_translation: trans,
                    pivot,
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
    let mid_radius = (radius + inner_radius) * 0.5;
    let num_verts = (segments + 1) * 3;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_verts);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_verts);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_verts);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(num_verts);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 12);

    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        positions.push([cos_a * radius, 0.0, sin_a * radius]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + 0.5 * cos_a, 0.5 + 0.5 * sin_a]);
        colors.push([0.65, 0.85, 0.98, 0.0]);

        positions.push([cos_a * mid_radius, 0.003, sin_a * mid_radius]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([
            0.5 + 0.5 * (mid_radius / radius) * cos_a,
            0.5 + 0.5 * (mid_radius / radius) * sin_a,
        ]);
        colors.push([0.80, 0.94, 1.0, 1.0]);

        positions.push([cos_a * inner_radius, 0.0, sin_a * inner_radius]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([
            0.5 + 0.5 * inner_ratio * cos_a,
            0.5 + 0.5 * inner_ratio * sin_a,
        ]);
        colors.push([0.65, 0.85, 0.98, 0.0]);
    }

    for i in 0..segments as u32 {
        let o1 = i * 3;
        let m1 = i * 3 + 1;
        let i1 = i * 3 + 2;

        let o2 = (i + 1) * 3;
        let m2 = (i + 1) * 3 + 1;
        let i2 = (i + 1) * 3 + 2;

        indices.push(o1);
        indices.push(o2);
        indices.push(m1);

        indices.push(m1);
        indices.push(o2);
        indices.push(m2);

        indices.push(m1);
        indices.push(m2);
        indices.push(i1);

        indices.push(i1);
        indices.push(m2);
        indices.push(i2);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
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
    let ring_mesh = meshes.add(create_water_ring_mesh(1.0, 0.70, 32));
    let ring_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.50, 0.78, 0.95, alpha * 0.60),
        emissive: LinearRgba::new(0.01, 0.04, 0.06, alpha * 0.12),
        unlit: false,
        perceptual_roughness: 0.03,
        reflectance: 0.98,
        alpha_mode: AlphaMode::Blend,
        depth_bias: 575.0,
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
            let fade = (1.0 - progress).powi(2);
            let alpha = ripple.initial_alpha * fade;
            mat.base_color = Color::srgba(0.50, 0.78, 0.95, alpha * 0.60);
            mat.emissive = LinearRgba::new(0.01, 0.04, 0.06, alpha * 0.12);
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
        Option<&crate::alchemy::Flammable>,
    ), Without<crate::player::camera::Player>>,
    mut passive_shrooms: Query<(
        &MushroomOrganicBreathing,
        &mut Transform,
        Option<&crate::alchemy::Flammable>,
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

    for (_entity, gt, mut shroom, light_opt, mut breathing_opt, flam_opt) in mushrooms.iter_mut() {
        let is_burning = flam_opt.map(|f| f.is_burning).unwrap_or(false);
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

        if !is_burning {
            if let Some((breathing, ref mut tf)) = breathing_opt {
                let squash_amp = if is_surging {
                    m_prof.idle_squash_amplitude * 1.5
                } else {
                    m_prof.idle_squash_amplitude
                };
                let sway_amp = if is_surging {
                    m_prof.idle_sway_amplitude * 1.4
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

                if breathing.pivot != Vec3::ZERO {
                    let rot_scale = tf.rotation * (tf.scale * breathing.pivot);
                    tf.translation = breathing.base_translation + breathing.pivot - rot_scale;
                }
            }
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

    for (breathing, mut tf, flam_opt) in passive_shrooms.iter_mut() {
        if flam_opt.map(|f| f.is_burning).unwrap_or(false) {
            continue;
        }
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

        if breathing.pivot != Vec3::ZERO {
            let rot_scale = tf.rotation * (tf.scale * breathing.pivot);
            tf.translation = breathing.base_translation + breathing.pivot - rot_scale;
        }
    }
}

pub fn setup_cavern_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<SceneAssetRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    mut moss_materials: ResMut<Assets<CelShadedMossMaterial>>,
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
            color: Color::srgb(0.20, 0.72, 0.85),
            intensity: 3200.0,
            range: 11.0,
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
            color: Color::srgb(0.22, 0.70, 0.82),
            intensity: 1400.0,
            range: 9.5,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 4.0, 2.2),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.18, 0.75, 0.88),
            intensity: 1200.0,
            range: 9.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 4.6, 2.9),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.95, 0.72, 0.40),
            intensity: 2200.0,
            range: 8.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 2.8, 6.8),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.92, 0.68, 0.35),
            intensity: 2400.0,
            range: 10.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 2.6, 9.5),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.90, 0.65, 0.32),
            intensity: 2400.0,
            range: 14.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 2.6, 17.5),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.95, 0.70, 0.36),
            intensity: 2800.0,
            range: 14.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 2.6, 25.5),
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

    for (name, x, y, z, hx, hy, hz) in [
        ("Collider_Left_Terrace_1", -6.49, 0.61, -6.82, 1.65, 0.82, 1.31),
        ("Collider_Left_Terrace_2", -6.78, 1.47, -7.81, 1.37, 0.89, 1.21),
        ("Collider_Left_Terrace_3", -7.00, 2.52, -8.79, 1.36, 1.06, 1.24),
        ("Collider_Left_Terrace_4", -7.37, 3.74, -9.81, 1.21, 1.13, 1.13),
        ("Collider_Left_Terrace_B0", -6.99, 0.45, 1.98, 1.55, 0.66, 1.39),
        ("Collider_Left_Terrace_B1", -6.80, 0.47, 0.50, 1.49, 0.69, 1.31),
        ("Collider_Left_Terrace_B2", -6.60, 0.50, -1.01, 1.56, 0.77, 1.34),
        ("Collider_Left_Terrace_B3", -6.47, 0.54, -2.61, 1.32, 0.74, 1.26),
        ("Collider_Left_Terrace_B4", -6.61, 0.57, -4.38, 1.56, 0.80, 1.38),
        ("Collider_Left_Terrace_Low", -5.80, 0.44, -5.79, 1.12, 0.56, 1.07),
        ("Collider_Left_Terrace_M0", -7.51, 1.45, 1.21, 1.43, 0.87, 1.25),
        ("Collider_Left_Terrace_M1", -7.28, 1.62, -0.62, 1.52, 0.86, 1.16),
        ("Collider_Left_Terrace_M2", -7.19, 1.86, -2.51, 1.29, 0.81, 1.08),
        ("Collider_Left_Terrace_M3", -7.21, 2.07, -4.39, 1.45, 0.97, 1.22),
        ("Collider_Right_Terrace_1", 6.53, 0.62, -6.81, 1.56, 0.79, 1.48),
        ("Collider_Right_Terrace_2", 6.78, 1.47, -7.78, 1.43, 0.98, 1.26),
        ("Collider_Right_Terrace_3", 6.99, 2.53, -8.80, 1.43, 1.08, 1.21),
        ("Collider_Right_Terrace_4", 7.41, 3.74, -9.82, 1.39, 1.16, 1.07),
        ("Collider_Right_Terrace_B0", 7.01, 0.45, 2.00, 1.37, 0.67, 1.22),
        ("Collider_Right_Terrace_B1", 6.78, 0.47, 0.53, 1.52, 0.74, 1.20),
        ("Collider_Right_Terrace_B2", 6.61, 0.51, -1.00, 1.64, 0.76, 1.40),
        ("Collider_Right_Terrace_B3", 6.50, 0.53, -2.61, 1.38, 0.69, 1.28),
        ("Collider_Right_Terrace_Low", 5.80, 0.44, -5.80, 1.11, 0.54, 1.17),
        ("Collider_Right_Terrace_M0", 7.49, 1.46, 1.23, 1.44, 0.87, 1.15),
        ("Collider_Right_Terrace_M1", 7.31, 1.63, -0.61, 1.51, 0.91, 1.24),
        ("Collider_Right_Terrace_M2", 7.19, 1.87, -2.50, 1.40, 0.94, 1.24),
        ("Collider_Right_Terrace_M3", 7.22, 2.07, -4.40, 1.58, 0.99, 1.29),
    ] {
        commands.spawn((
            Name::new(name),
            CavernObstacle,
            ObstacleShape::Cuboid {
                half_extents: Vec3::new(hx, hy, hz),
            },
            Transform::from_xyz(x, y, z),
            RigidBody::Static,
            Collider::cuboid(hx * 2.0, hy * 2.0, hz * 2.0),
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
        Mesh3d(meshes.add(create_water_disk_mesh(5.24, 96))),
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
        Transform::from_xyz(0.0, 0.065, -8.5),
    ));

    let cavern_rock_mat = standard_materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.16, 0.20),
        perceptual_roughness: 0.88,
        metallic: 0.0,
        reflectance: 0.15,
        ..default()
    });

    let _shoreline_moss_mat = moss_materials.add(CelShadedMossMaterial {
        base_color: LinearRgba::new(0.036, 0.088, 0.134, 1.0),
        moss_color: LinearRgba::new(0.025, 0.075, 0.065, 1.0),
        light_dir: Vec4::new(0.25, 0.85, -0.45, 0.0),
        moss_params: Vec4::new(0.70, 3.50, 0.15, 3.0),
    });

    let mesh_boulder = meshes.add(create_faceted_stone_mesh(7, 4, Vec3::new(0.68, 0.44, 0.62), 0.4));
    let mesh_pebble = meshes.add(create_faceted_stone_mesh(6, 4, Vec3::new(0.40, 0.20, 0.36), 0.8));
    let mesh_flat_stepping = meshes.add(create_faceted_stone_mesh(7, 4, Vec3::new(0.55, 0.15, 0.48), 1.2));

    let shore_stone_configs = [
        (-1.45, 0.035, -3.35, 0.52, 0.4, 1),
        (-0.95, 0.025, -3.42, 0.38, -0.6, 2),
        (-1.95, 0.045, -3.48, 0.62, 0.8, 0),
        (1.10, 0.030, -3.38, 0.44, 0.5, 2),
        (1.65, 0.040, -3.45, 0.58, -0.3, 1),
        (2.25, 0.055, -3.65, 0.68, 0.7, 0),
        (-0.40, 0.015, -3.55, 0.36, 0.2, 2),
        (0.45, 0.018, -3.58, 0.38, -0.4, 2),
        (-3.10, 0.075, -4.20, 0.85, 0.3, 0),
        (-3.85, 0.090, -4.95, 0.95, -0.5, 0),
        (-4.55, 0.110, -5.90, 1.05, 0.4, 0),
        (-4.95, 0.125, -7.10, 1.10, -0.6, 0),
        (-5.10, 0.130, -8.30, 1.05, 0.5, 0),
        (-4.80, 0.115, -9.55, 0.95, -0.4, 0),
        (-4.20, 0.095, -10.75, 0.90, 0.3, 0),
        (3.05, 0.075, -4.25, 0.85, -0.4, 0),
        (3.80, 0.090, -5.00, 0.95, 0.5, 0),
        (4.50, 0.110, -5.95, 1.05, -0.3, 0),
        (4.90, 0.125, -7.15, 1.10, 0.6, 0),
        (5.05, 0.130, -8.35, 1.05, -0.5, 0),
        (4.75, 0.115, -9.60, 0.95, 0.4, 0),
        (4.15, 0.095, -10.80, 0.90, -0.3, 0),
        (-2.80, 0.080, -12.40, 0.85, 0.6, 0),
        (-1.50, 0.065, -13.15, 0.75, -0.4, 1),
        (0.00, 0.060, -13.40, 0.80, 0.2, 0),
        (1.50, 0.065, -13.15, 0.75, 0.5, 1),
        (2.80, 0.080, -12.40, 0.85, -0.5, 0),
    ];

    for (x, y, z, scale, rot_y, mesh_type) in shore_stone_configs {
        let mesh_handle = match mesh_type {
            0 => mesh_boulder.clone(),
            1 => mesh_pebble.clone(),
            _ => mesh_flat_stepping.clone(),
        };
        commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(cavern_rock_mat.clone()),
            Transform::from_xyz(x, y, z)
                .with_scale(Vec3::splat(scale))
                .with_rotation(Quat::from_rotation_y(rot_y)),
        ));
    }

    let mesh_cliff_boulder = meshes.add(create_faceted_stone_mesh(7, 4, Vec3::new(1.05, 0.90, 1.00), 0.3));
    let mesh_cliff_pillar = meshes.add(create_faceted_stone_mesh(6, 5, Vec3::new(0.80, 1.45, 0.85), 0.8));
    let mesh_cliff_shelf = meshes.add(create_faceted_stone_mesh(7, 4, Vec3::new(1.30, 0.60, 1.10), 1.2));
    let mesh_arch_keystone = meshes.add(create_faceted_stone_mesh(7, 4, Vec3::new(0.95, 0.70, 0.85), 0.5));
    let mesh_cliff_accent = meshes.add(create_faceted_stone_mesh(6, 4, Vec3::new(0.65, 0.55, 0.60), 1.6));

    let portal_wall_stone_configs = [
        (0.00, 5.42, 3.96, 1.15, 0.10, 0),
        (-0.85, 5.32, 3.96, 1.05, -0.25, 3),
        (0.85, 5.32, 3.96, 1.05, 0.30, 3),
        (-1.75, 4.95, 3.95, 1.10, 0.40, 0),
        (1.75, 4.95, 3.95, 1.10, -0.45, 0),
        (0.00, 6.45, 4.05, 1.35, 0.50, 2),
        (-1.25, 6.35, 4.00, 1.25, -0.40, 0),
        (1.25, 6.35, 4.00, 1.25, 0.35, 0),
        (0.00, 7.50, 4.15, 1.40, 0.20, 2),
        (-1.50, 7.40, 4.10, 1.30, -0.30, 0),
        (1.50, 7.40, 4.10, 1.30, 0.25, 0),
        (-2.85, 0.50, 3.85, 1.15, 0.25, 0),
        (-3.95, 0.55, 3.80, 1.25, -0.40, 2),
        (-5.20, 0.50, 3.80, 1.30, 0.30, 0),
        (-6.50, 0.55, 3.85, 1.25, -0.50, 2),
        (-7.60, 0.60, 3.85, 1.30, 0.40, 0),
        (-2.90, 1.90, 3.90, 1.10, -0.30, 1),
        (-4.10, 2.05, 3.85, 1.30, 0.45, 2),
        (-5.40, 2.15, 3.80, 1.25, -0.35, 0),
        (-6.70, 2.10, 3.85, 1.25, 0.40, 2),
        (-2.85, 3.35, 3.95, 1.15, 0.35, 0),
        (-4.25, 3.50, 3.85, 1.35, -0.45, 2),
        (-5.60, 3.60, 3.80, 1.25, 0.50, 1),
        (-6.90, 3.55, 3.85, 1.20, -0.25, 2),
        (-2.70, 4.60, 4.00, 1.15, -0.40, 0),
        (-3.95, 4.80, 3.95, 1.30, 0.35, 2),
        (-5.35, 4.90, 3.85, 1.25, -0.50, 0),
        (-6.70, 4.95, 3.90, 1.20, 0.45, 2),
        (-3.50, 6.10, 4.05, 1.30, -0.35, 0),
        (-5.10, 6.20, 4.00, 1.35, 0.40, 2),
        (-6.60, 6.15, 4.00, 1.25, -0.30, 0),
        (2.85, 0.50, 3.85, 1.15, -0.30, 0),
        (4.00, 0.55, 3.80, 1.25, 0.45, 2),
        (5.25, 0.50, 3.80, 1.30, -0.35, 0),
        (6.55, 0.55, 3.85, 1.25, 0.55, 2),
        (7.65, 0.60, 3.85, 1.30, -0.40, 0),
        (2.90, 1.90, 3.90, 1.10, 0.35, 1),
        (4.15, 2.05, 3.85, 1.30, -0.40, 2),
        (5.45, 2.15, 3.80, 1.25, 0.30, 0),
        (6.75, 2.10, 3.85, 1.25, -0.45, 2),
        (2.85, 3.35, 3.95, 1.15, -0.40, 0),
        (4.30, 3.50, 3.85, 1.35, 0.50, 2),
        (5.65, 3.60, 3.80, 1.25, -0.45, 1),
        (6.95, 3.55, 3.85, 1.20, 0.30, 2),
        (2.70, 4.60, 4.00, 1.15, 0.45, 0),
        (4.00, 4.80, 3.95, 1.30, -0.30, 2),
        (5.40, 4.90, 3.85, 1.25, 0.45, 0),
        (6.75, 4.95, 3.90, 1.20, -0.40, 2),
        (3.55, 6.10, 4.05, 1.30, 0.40, 0),
        (5.15, 6.20, 4.00, 1.35, -0.35, 2),
        (6.65, 6.15, 4.00, 1.25, 0.35, 0),
        (-2.45, 0.18, 3.65, 0.65, 0.50, 4),
        (2.45, 0.18, 3.65, 0.65, -0.45, 4),
        (-3.35, 0.90, 3.65, 0.70, -0.35, 4),
        (3.35, 0.90, 3.65, 0.70, 0.40, 4),
    ];

    for (x, y, z, scale, rot_y, mesh_type) in portal_wall_stone_configs {
        let mesh_handle = match mesh_type {
            0 => mesh_cliff_boulder.clone(),
            1 => mesh_cliff_pillar.clone(),
            2 => mesh_cliff_shelf.clone(),
            3 => mesh_arch_keystone.clone(),
            _ => mesh_cliff_accent.clone(),
        };
        commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(cavern_rock_mat.clone()),
            Transform::from_xyz(x, y, z)
                .with_scale(Vec3::splat(scale))
                .with_rotation(Quat::from_rotation_y(rot_y)),
        ));
    }

    for x_off in [-3.6, -2.4, -1.2, 0.0, 1.2, 2.4, 3.6] {
        commands.spawn((
            Transform::from_xyz(x_off, 0.06, -8.55),
            ParticleEmitter {
                spawn_timer: 0.0,
                spawn_rate: 0.08,
                particle_type: ParticleType::WaterSplash,
            },
        ));
        commands.spawn((
            Transform::from_xyz(x_off * 0.8, 0.10, -8.7),
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
        crate::alchemy::Flammable::with_duration(3.0).with_ignition_temperature(20.0),
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
        crate::alchemy::Flammable::with_duration(4.5)
            .with_ignition_temperature(50.0)
            .with_cooling_rate(4.0),
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

    crate::dungeon::water_drips::spawn_default_cavern_water_drips(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
    );

    let fauna_assets = crate::dungeon::fauna::setup_ambient_fauna(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
    );

    let initial_mushrooms = [
        Vec3::new(-3.4, 2.4, -2.8),
        Vec3::new(4.1, 2.4, -3.3),
    ];
    let initial_puddles = [
        (Vec3::new(1.05, 0.02, 6.40), None),
        (Vec3::new(-1.05, 0.02, 9.50), None),
    ];
    let initial_webs = [
        (Vec3::new(1.85, 4.10, 5.50), Quat::from_rotation_y(0.0), 0.75, true),
        (Vec3::new(-1.85, 0.35, 8.20), Quat::from_rotation_y(std::f32::consts::PI), 0.65, true),
        (Vec3::new(-1.85, 4.15, 12.00), Quat::from_rotation_y(std::f32::consts::PI), 0.70, true),
        (Vec3::new(3.80, 0.40, -1.20), Quat::from_rotation_y(1.57), 0.60, false),
    ];
    crate::dungeon::fauna::spawn_initial_ambient_fauna(
        &mut commands,
        &fauna_assets,
        &initial_mushrooms,
        &initial_puddles,
        &initial_webs,
    );

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

pub fn create_faceted_stone_mesh(u_segs: usize, v_segs: usize, scale: Vec3, aspect_shift: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let u_steps = u_segs.max(6);
    let v_steps = v_segs.max(3);

    let apex = Vec3::new(0.0, scale.y * 1.02, 0.0);
    let bottom = Vec3::new(0.0, -scale.y * 0.28, 0.0);

    let mut temp_pos = Vec::new();

    for iv in 0..v_steps {
        let v_frac = (iv as f32 + 0.5) / v_steps as f32;
        let phi = 0.22 + v_frac * (std::f32::consts::PI - 0.44);
        let y = phi.cos();
        let r_sin = phi.sin();

        for iu in 0..u_steps {
            let u_frac = iu as f32 / u_steps as f32;
            let theta = u_frac * std::f32::consts::TAU;

            let x = theta.cos() * r_sin;
            let z = theta.sin() * r_sin;

            let oblong = 1.0 + (theta * 2.0 + aspect_shift).cos() * 0.15;
            let p = Vec3::new(
                x * oblong * scale.x,
                (y * oblong * scale.y).max(-scale.y * 0.28),
                z * oblong * scale.z,
            );
            temp_pos.push(p);
        }
    }

    let mut vert_idx = 0u32;

    for iu in 0..u_steps {
        let next_u = (iu + 1) % u_steps;
        let p0 = temp_pos[iu];
        let p1 = temp_pos[next_u];

        let v_a = p0 - apex;
        let v_b = p1 - apex;
        let n = v_b.cross(v_a).normalize_or_zero();

        positions.push([apex.x, apex.y, apex.z]);
        positions.push([p1.x, p1.y, p1.z]);
        positions.push([p0.x, p0.y, p0.z]);
        normals.push([n.x, n.y, n.z]);
        normals.push([n.x, n.y, n.z]);
        normals.push([n.x, n.y, n.z]);
        uvs.push([0.5, 0.5]);
        uvs.push([1.0, 0.0]);
        uvs.push([0.0, 0.0]);
        indices.push(vert_idx);
        indices.push(vert_idx + 1);
        indices.push(vert_idx + 2);
        vert_idx += 3;
    }

    for iv in 0..(v_steps - 1) {
        for iu in 0..u_steps {
            let next_u = (iu + 1) % u_steps;

            let p00 = temp_pos[iv * u_steps + iu];
            let p10 = temp_pos[iv * u_steps + next_u];
            let p01 = temp_pos[(iv + 1) * u_steps + iu];
            let p11 = temp_pos[(iv + 1) * u_steps + next_u];

            let n1 = (p10 - p00).cross(p01 - p00).normalize_or_zero();
            positions.push([p00.x, p00.y, p00.z]);
            positions.push([p10.x, p10.y, p10.z]);
            positions.push([p01.x, p01.y, p01.z]);
            normals.push([n1.x, n1.y, n1.z]);
            normals.push([n1.x, n1.y, n1.z]);
            normals.push([n1.x, n1.y, n1.z]);
            uvs.push([0.0, 0.0]);
            uvs.push([1.0, 0.0]);
            uvs.push([0.0, 1.0]);
            indices.push(vert_idx);
            indices.push(vert_idx + 1);
            indices.push(vert_idx + 2);
            vert_idx += 3;

            let n2 = (p11 - p10).cross(p01 - p11).normalize_or_zero();
            positions.push([p10.x, p10.y, p10.z]);
            positions.push([p11.x, p11.y, p11.z]);
            positions.push([p01.x, p01.y, p01.z]);
            normals.push([n2.x, n2.y, n2.z]);
            normals.push([n2.x, n2.y, n2.z]);
            normals.push([n2.x, n2.y, n2.z]);
            uvs.push([1.0, 0.0]);
            uvs.push([1.0, 1.0]);
            uvs.push([0.0, 1.0]);
            indices.push(vert_idx);
            indices.push(vert_idx + 1);
            indices.push(vert_idx + 2);
            vert_idx += 3;
        }
    }

    let last_ring = (v_steps - 1) * u_steps;
    for iu in 0..u_steps {
        let next_u = (iu + 1) % u_steps;
        let p0 = temp_pos[last_ring + iu];
        let p1 = temp_pos[last_ring + next_u];

        let v_a = p0 - bottom;
        let v_b = p1 - bottom;
        let n = v_a.cross(v_b).normalize_or_zero();

        positions.push([bottom.x, bottom.y, bottom.z]);
        positions.push([p0.x, p0.y, p0.z]);
        positions.push([p1.x, p1.y, p1.z]);
        normals.push([n.x, n.y, n.z]);
        normals.push([n.x, n.y, n.z]);
        normals.push([n.x, n.y, n.z]);
        uvs.push([0.5, 0.5]);
        uvs.push([0.0, 1.0]);
        uvs.push([1.0, 1.0]);
        indices.push(vert_idx);
        indices.push(vert_idx + 1);
        indices.push(vert_idx + 2);
        vert_idx += 3;
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
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
    let h_bottom = -0.45;
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

    let lake_floor = if dist_lake < lake_radius {
        let u = dist_lake / lake_radius;
        let funnel_depth = (1.0 - u).max(0.0).powf(1.75) * 3.15 + (1.0 - u) * 0.35;
        0.04 - funnel_depth
    } else if dist_lake < lake_radius + 1.8 {
        let rim_blend = (dist_lake - lake_radius) / 1.8;
        0.04 + rim_blend * 0.03
    } else {
        0.04
    };

    let lateral_ramp = if x.abs() > 2.5 && z <= 4.0 {
        let x_abs = x.abs();
        let lateral_rise = if x_abs < 5.5 {
            ((x_abs - 2.5) / 3.0) * 1.05
        } else {
            1.05 + ((x_abs - 5.5) / 2.5).min(1.0) * 0.85
        };

        if z > -4.0 {
            let fade = ((4.0 - z) / 2.0).clamp(0.0, 1.0);
            lateral_rise * fade
        } else if z > -7.5 {
            let t = (z - (-7.5)) / 3.5;
            lateral_rise * t.clamp(0.2, 1.0)
        } else if x_abs > 6.0 {
            (lateral_rise * 0.85).max(0.10)
        } else {
            0.0
        }
    } else {
        0.0
    };

    lake_floor + lateral_ramp
}

pub fn get_grotto_terrain_normal(x: f32, z: f32) -> Vec3 {
    let eps = 0.08;
    let h_r = get_grotto_terrain_height(x + eps, z);
    let h_l = get_grotto_terrain_height(x - eps, z);
    let h_u = get_grotto_terrain_height(x, z + eps);
    let h_d = get_grotto_terrain_height(x, z - eps);

    let dh_dx = (h_r - h_l) / (2.0 * eps);
    let dh_dz = (h_u - h_d) / (2.0 * eps);

    Vec3::new(-dh_dx, 1.0, -dh_dz).normalize()
}

pub fn is_in_lake_water(pos: Vec3) -> bool {
    let dx = pos.x;
    let dz = pos.z - (-8.5);
    let dist = (dx * dx + dz * dz).sqrt();
    dist <= 5.35 && pos.y <= 0.22
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
