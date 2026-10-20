use crate::{
    player::camera::{LocomotionState, Player, PlayerCamera, PlayerState},
    ui::{hud::ActiveHotbarSlot, menu::GameState},
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Component)]
pub enum AlchemicalEffectKind {
    ThermalHeat,
    Vaporize,
    Luminescence,
    Vitality,
    Swiftness,
    Frost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEffect {
    pub kind: AlchemicalEffectKind,
    pub duration: f32,
    pub elapsed: f32,
    pub magnitude: f32,
}

#[derive(Component, Debug, Default)]
pub struct EffectReceiverComponent {
    pub active_effects: Vec<ActiveEffect>,
}

#[allow(dead_code)]
impl EffectReceiverComponent {
    pub fn apply_effect(&mut self, kind: AlchemicalEffectKind, duration: f32, magnitude: f32) {
        if let Some(existing) = self.active_effects.iter_mut().find(|e| e.kind == kind) {
            existing.duration = existing.duration.max(duration);
            existing.magnitude = existing.magnitude.max(magnitude);
            existing.elapsed = 0.0;
        } else {
            self.active_effects.push(ActiveEffect {
                kind,
                duration,
                elapsed: 0.0,
                magnitude,
            });
        }
    }

    pub fn has_effect(&self, kind: AlchemicalEffectKind) -> bool {
        self.active_effects.iter().any(|e| e.kind == kind)
    }
}

#[derive(Component, Debug)]
#[allow(dead_code)]
pub struct AlchemicalIngredient {
    pub id: String,
    pub name: String,
    pub primary_effect: AlchemicalEffectKind,
    pub secondary_effect: AlchemicalEffectKind,
    pub harvested: bool,
}

#[derive(Component, Debug)]
pub struct PotionProjectile {
    pub velocity: Vec3,
    pub effects: Vec<ActiveEffect>,
    pub lifetime: f32,
    pub color: Color,
    pub slot: usize,
}

#[derive(Component, Debug)]
pub struct TrajectoryAimIndicator;

#[derive(Component, Debug, Clone)]
pub struct PotionPuddle {
    pub color: Color,
    pub elapsed: f32,
    pub max_lifetime: f32,
    pub radius: f32,
    pub effects: Vec<ActiveEffect>,
    pub is_wall: bool,
}

#[derive(Component, Debug)]
pub struct GlassShardParticle {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub ground_y: f32,
    pub settled: bool,
    pub color: Color,
}

#[derive(Component, Debug)]
#[allow(dead_code)]
pub struct PotionFlask {
    pub id: String,
    pub effects: Vec<ActiveEffect>,
    pub liquid_color: Color,
    pub volume: f32,
}

#[derive(Component, Debug)]
pub struct ThermalGeyser {
    pub eruption_timer: f32,
    pub eruption_interval: f32,
    pub is_erupting: bool,
    pub heat_radius: f32,
}

pub struct AlchemyPlugin;

impl Plugin for AlchemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_active_effects,
                update_thermal_geysers,
                update_potion_aiming
                    .run_if(resource_exists::<ButtonInput<MouseButton>>)
                    .run_if(resource_exists::<SpatialQueryPipeline>)
                    .run_if(resource_exists::<Assets<StandardMaterial>>),
                spawn_potion_projectile
                    .run_if(resource_exists::<ButtonInput<MouseButton>>)
                    .run_if(resource_exists::<Assets<StandardMaterial>>),
                update_potion_projectiles
                    .run_if(resource_exists::<SpatialQueryPipeline>)
                    .run_if(resource_exists::<Assets<StandardMaterial>>),
                update_potion_puddles.run_if(resource_exists::<Assets<StandardMaterial>>),
                update_glass_shards.run_if(resource_exists::<Assets<StandardMaterial>>),
            ),
        );
    }
}

pub fn calculate_aim_trajectory_and_impact(
    origin: Vec3,
    initial_velocity: Vec3,
    gravity: f32,
    steps: usize,
    step_time: f32,
    spatial_query: Option<(&SpatialQuery, &SpatialQueryFilter)>,
) -> (Vec<Vec3>, Option<(Vec3, Vec3)>) {
    let mut points = Vec::with_capacity(steps + 1);
    let mut current_pos = origin;
    let mut current_vel = initial_velocity;
    points.push(current_pos);
    let mut hit_info = None;

    for _ in 0..steps {
        let next_vel = current_vel + Vec3::new(0.0, -gravity * step_time, 0.0);
        let displacement = (current_vel + next_vel) * 0.5 * step_time;
        let segment_len = displacement.length();

        if let Some((sq, filter)) = spatial_query {
            if segment_len > 1e-4 {
                let dir = displacement / segment_len;
                if let Some(hit) = sq.cast_ray(
                    current_pos,
                    Dir3::new(dir).unwrap_or(Dir3::NEG_Y),
                    segment_len,
                    true,
                    filter,
                ) {
                    if hit.distance > 0.001 {
                        let impact_pos = current_pos + dir * hit.distance;
                        points.push(impact_pos);
                        hit_info = Some((impact_pos, hit.normal));
                        break;
                    }
                }
            }
        }

        current_pos += displacement;
        current_vel = next_vel;
        points.push(current_pos);
        if current_pos.y < -5.0 {
            break;
        }
    }

    (points, hit_info)
}

pub fn calculate_aim_trajectory(
    origin: Vec3,
    initial_velocity: Vec3,
    gravity: f32,
    steps: usize,
    step_time: f32,
    spatial_query: Option<&SpatialQuery>,
) -> Vec<Vec3> {
    let filter = SpatialQueryFilter::default();
    let sq_tuple = spatial_query.map(|sq| (sq, &filter));
    let (pts, _) = calculate_aim_trajectory_and_impact(
        origin,
        initial_velocity,
        gravity,
        steps,
        step_time,
        sq_tuple,
    );
    pts
}

pub fn apply_throw_inaccuracy(
    base_direction: Vec3,
    is_sprinting: bool,
    time_seed: f32,
) -> Vec3 {
    if !is_sprinting {
        return base_direction.normalize_or_zero();
    }
    let pseudo_rand_x = (time_seed * 127.1 + 311.7).sin() * 43758.547;
    let pseudo_rand_y = (time_seed * 269.5 + 183.3).cos() * 24634.63;
    let fract_x = (pseudo_rand_x.fract() - 0.5) * 2.0;
    let fract_y = (pseudo_rand_y.fract() - 0.5) * 2.0;
    let right = base_direction.cross(Vec3::Y).normalize_or_zero();
    let up = right.cross(base_direction).normalize_or_zero();
    let inaccuracy_amount = 0.20;
    let perturbed = base_direction + right * (fract_x * inaccuracy_amount) + up * (fract_y * inaccuracy_amount);
    perturbed.normalize_or_zero()
}

pub fn selected_potion_effects(slot: usize) -> (Color, Vec<ActiveEffect>) {
    let (color, kind, magnitude) = match slot {
        2 => (
            Color::srgb(0.95, 0.12, 0.16),
            AlchemicalEffectKind::ThermalHeat,
            25.0,
        ),
        3 => (
            Color::srgb(0.95, 0.72, 0.12),
            AlchemicalEffectKind::Vitality,
            30.0,
        ),
        4 => (
            Color::srgb(0.12, 0.42, 0.95),
            AlchemicalEffectKind::Frost,
            20.0,
        ),
        5 => (
            Color::srgb(0.92, 0.95, 1.0),
            AlchemicalEffectKind::Luminescence,
            18.0,
        ),
        _ => (
            Color::srgb(0.12, 0.92, 0.42),
            AlchemicalEffectKind::Swiftness,
            20.0,
        ),
    };
    (
        color,
        vec![ActiveEffect {
            kind,
            duration: 5.0,
            elapsed: 0.0,
            magnitude,
        }],
    )
}

pub fn mix_potion_colors(c1: Color, c2: Color) -> Color {
    let s1 = c1.to_srgba();
    let s2 = c2.to_srgba();
    let r = (s1.red + s2.red) * 0.5;
    let g = (s1.green + s2.green) * 0.5;
    let b = (s1.blue + s2.blue) * 0.5;
    let max_in = (s1.red.max(s1.green).max(s1.blue) + s2.red.max(s2.green).max(s2.blue)) * 0.5;
    let max_mix = r.max(g).max(b).max(0.001);
    let boost = (max_in / max_mix).min(1.4);
    Color::srgb(
        (r * boost).clamp(0.0, 1.0),
        (g * boost).clamp(0.0, 1.0),
        (b * boost).clamp(0.0, 1.0),
    )
}

pub fn merge_potion_effects(base: &mut Vec<ActiveEffect>, incoming: &[ActiveEffect]) {
    for inc in incoming {
        if let Some(existing) = base.iter_mut().find(|e| e.kind == inc.kind) {
            existing.magnitude = (existing.magnitude + inc.magnitude) * 0.85;
            existing.duration = existing.duration.max(inc.duration);
            existing.elapsed = 0.0;
        } else {
            base.push(inc.clone());
        }
    }
    let has_heat = base.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat);
    let has_frost = base.iter().any(|e| e.kind == AlchemicalEffectKind::Frost);
    if has_heat && has_frost && !base.iter().any(|e| e.kind == AlchemicalEffectKind::Vaporize) {
        base.push(ActiveEffect {
            kind: AlchemicalEffectKind::Vaporize,
            duration: 8.0,
            elapsed: 0.0,
            magnitude: 40.0,
        });
    }
}

pub fn update_potion_aiming(
    game_state: Option<Res<State<GameState>>>,
    mouse: Res<ButtonInput<MouseButton>>,
    slot: Option<Res<ActiveHotbarSlot>>,
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    player: Query<&PlayerState>,
    player_entities: Query<Entity, With<Player>>,
    spatial_query: SpatialQuery,
    aim_indicators: Query<Entity, With<TrajectoryAimIndicator>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if let Some(state) = game_state {
        if *state.get() != GameState::InGame {
            for entity in aim_indicators.iter() {
                commands.entity(entity).despawn();
            }
            return;
        }
    }

    let is_aiming = mouse.pressed(MouseButton::Right) && player.iter().all(|s| *s == PlayerState::Active);

    for entity in aim_indicators.iter() {
        commands.entity(entity).despawn();
    }

    if !is_aiming {
        return;
    }

    let Ok(camera_tf) = camera.get_single() else {
        return;
    };

    let slot_index = slot.as_deref().map(|s| s.0).unwrap_or(1);
    let (color, _) = selected_potion_effects(slot_index);
    let forward = *camera_tf.forward();
    let origin = camera_tf.translation() + forward * 0.45 + Vec3::new(0.0, -0.12, 0.0);
    let impulse_dir = (forward + Vec3::new(0.0, 0.22, 0.0)).normalize();
    let initial_velocity = impulse_dir * 16.0;

    let excluded: Vec<Entity> = player_entities.iter().collect();
    let filter = SpatialQueryFilter::default().with_excluded_entities(excluded);

    let (points, hit_info) = calculate_aim_trajectory_and_impact(
        origin,
        initial_velocity,
        9.8,
        28,
        0.045,
        Some((&spatial_query, &filter)),
    );

    let bead_mesh = meshes.add(Sphere::new(0.028).mesh().uv(8, 6));
    let bead_mat = materials.add(StandardMaterial {
        base_color: color.with_alpha(0.85),
        emissive: color.to_linear() * 2.2,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    for pt in points.into_iter().skip(1) {
        commands.spawn((
            TrajectoryAimIndicator,
            Mesh3d(bead_mesh.clone()),
            MeshMaterial3d(bead_mat.clone()),
            Transform::from_translation(pt),
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ));
    }

    if let Some((hit_pos, hit_normal)) = hit_info {
        let decal_mesh = meshes.add(create_flat_circle_mesh(0.35, 24));
        let decal_mat = materials.add(StandardMaterial {
            base_color: color.with_alpha(0.88),
            emissive: color.to_linear() * 2.0,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let rotation = Quat::from_rotation_arc(Vec3::Y, hit_normal);
        commands.spawn((
            TrajectoryAimIndicator,
            Mesh3d(decal_mesh),
            MeshMaterial3d(decal_mat),
            Transform::from_translation(hit_pos + hit_normal * 0.015).with_rotation(rotation),
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ));
    }
}

pub fn spawn_potion_projectile(
    game_state: Option<Res<State<GameState>>>,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    slot: Option<Res<ActiveHotbarSlot>>,
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    player_query: Query<(&PlayerState, Option<&LocomotionState>), With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if let Some(state) = game_state {
        if *state.get() != GameState::InGame {
            return;
        }
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok((player_state, locomotion_opt)) = player_query.get_single() else {
        return;
    };
    if *player_state != PlayerState::Active {
        return;
    }
    let Ok(camera) = camera.get_single() else {
        return;
    };

    let is_sprinting = locomotion_opt.map(|l| l.is_sprinting).unwrap_or(false);
    let base_direction = *camera.forward();
    let final_direction = apply_throw_inaccuracy(base_direction, is_sprinting, time.elapsed_secs());

    let slot_index = slot.as_deref().map(|s| s.0).unwrap_or(1);
    let (color, effects) = selected_potion_effects(slot_index);
    let spawn_pos = camera.translation() + base_direction * 0.45 + Vec3::new(0.0, -0.10, 0.0);

    let impulse_dir = (final_direction + Vec3::new(0.0, 0.22, 0.0)).normalize();
    let speed = 16.0;
    let linear_vel = impulse_dir * speed;

    let flask_mesh = meshes.add(create_potion_flask_mesh(slot_index));
    let flask_mat = materials.add(StandardMaterial {
        base_color: color.with_alpha(0.85),
        emissive: color.to_linear() * 1.2,
        perceptual_roughness: 0.12,
        metallic: 0.05,
        reflectance: 0.85,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        Name::new("Potion_Flask_Projectile"),
        Mesh3d(flask_mesh),
        MeshMaterial3d(flask_mat),
        Transform::from_translation(spawn_pos).with_scale(Vec3::splat(1.0)),
        PotionProjectile {
            velocity: linear_vel,
            effects,
            lifetime: 8.0,
            color,
            slot: slot_index,
        },
    ));
}

pub fn update_potion_projectiles(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut projectiles: Query<(Entity, &mut Transform, &mut PotionProjectile)>,
    mut puddles: Query<(Entity, &mut PotionPuddle, &mut Transform, &MeshMaterial3d<StandardMaterial>), Without<PotionProjectile>>,
    player_entities: Query<Entity, With<Player>>,
    mut receivers: Query<(Entity, &Transform, &mut EffectReceiverComponent), (Without<PotionProjectile>, Without<PotionPuddle>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let gravity = Vec3::new(0.0, -9.8, 0.0);
    let mut excluded: Vec<Entity> = player_entities.iter().collect();

    for (entity, _, _) in projectiles.iter() {
        excluded.push(entity);
    }
    let filter = SpatialQueryFilter::default().with_excluded_entities(excluded.clone());

    for (entity, mut transform, mut projectile) in &mut projectiles {
        projectile.velocity += gravity * dt;
        let step = projectile.velocity * dt;
        let distance = step.length();
        let direction = projectile.velocity.normalize_or_zero();

        let hit = if distance > 1e-4 {
            spatial_query.cast_ray(
                transform.translation,
                Dir3::new(direction).unwrap_or(Dir3::NEG_Y),
                distance,
                true,
                &filter,
            )
        } else {
            None
        };

        if let Some(hit) = hit {
            if hit.distance > 0.001 && hit.entity != entity && !excluded.contains(&hit.entity) {
                let impact_point = transform.translation + direction * hit.distance;
                let impact_normal = hit.normal;

                let mut merged = false;
                let mut final_color = projectile.color;

                for (_p_ent, mut puddle, mut p_tf, mat_handle) in puddles.iter_mut() {
                    let dist = p_tf.translation.distance(impact_point);
                    if dist <= puddle.radius + 0.65 {
                        final_color = mix_potion_colors(puddle.color, projectile.color);
                        puddle.color = final_color;
                        merge_potion_effects(&mut puddle.effects, &projectile.effects);
                        puddle.radius = (puddle.radius + 0.38).min(2.8);
                        puddle.elapsed = (puddle.elapsed * 0.30).max(0.0);
                        if let Some(mat) = materials.get_mut(&mat_handle.0) {
                            mat.base_color = final_color.with_alpha(0.88);
                            mat.emissive = final_color.to_linear() * 1.9;
                        }
                        p_tf.scale = Vec3::new(puddle.radius, 1.0, puddle.radius);

                        for (r_entity, r_tf, mut receiver) in receivers.iter_mut() {
                            if r_entity == hit.entity || r_tf.translation.distance(p_tf.translation) <= puddle.radius * 1.6 {
                                for effect in &puddle.effects {
                                    receiver.apply_effect(effect.kind, effect.duration, effect.magnitude);
                                }
                            }
                        }

                        merged = true;
                        break;
                    }
                }

                if !merged {
                    for (r_entity, r_tf, mut receiver) in receivers.iter_mut() {
                        if r_entity == hit.entity || r_tf.translation.distance(impact_point) <= 2.8 {
                            for effect in &projectile.effects {
                                receiver.apply_effect(effect.kind, effect.duration, effect.magnitude);
                            }
                        }
                    }

                    let is_wall = impact_normal.y.abs() < 0.65;
                    let (puddle_mesh_handle, puddle_rot, puddle_pos) = if is_wall {
                        let grav = Vec3::NEG_Y;
                        let down_proj = (grav - impact_normal * grav.dot(impact_normal)).normalize_or_zero();
                        let down = if down_proj.length_squared() > 0.01 {
                            down_proj
                        } else {
                            Vec3::NEG_Y
                        };
                        let right = impact_normal.cross(down).normalize();
                        let rot = Quat::from_mat3(&Mat3::from_cols(right, impact_normal, -down));
                        let mesh = meshes.add(create_wall_splatter_mesh(1.10, 32));
                        (mesh, rot, impact_point + impact_normal * 0.038)
                    } else {
                        let rot = Quat::from_rotation_arc(Vec3::Y, impact_normal);
                        let mesh = meshes.add(create_irregular_puddle_mesh(1.15, 32));
                        (mesh, rot, impact_point + impact_normal * 0.055)
                    };

                    let puddle_mat = materials.add(StandardMaterial {
                        base_color: projectile.color.with_alpha(0.88),
                        emissive: projectile.color.to_linear() * 1.8,
                        perceptual_roughness: 0.04,
                        metallic: 0.15,
                        reflectance: 0.98,
                        cull_mode: None,
                        double_sided: true,
                        alpha_mode: AlphaMode::Blend,
                        depth_bias: 600.0,
                        ..default()
                    });

                    commands.spawn((
                        PotionPuddle {
                            color: projectile.color,
                            elapsed: 0.0,
                            max_lifetime: 75.0,
                            radius: 1.15,
                            effects: projectile.effects.clone(),
                            is_wall,
                        },
                        Mesh3d(puddle_mesh_handle),
                        MeshMaterial3d(puddle_mat),
                        Transform::from_translation(puddle_pos)
                            .with_rotation(puddle_rot)
                            .with_scale(Vec3::new(0.15, 1.0, 0.15)),
                    ));
                }

                commands.entity(entity).despawn();

                let is_wall = impact_normal.y.abs() < 0.65;
                let floor_y = if is_wall {
                    let down_ray = spatial_query.cast_ray(
                        impact_point + impact_normal * 0.15,
                        Dir3::NEG_Y,
                        25.0,
                        true,
                        &filter,
                    );
                    down_ray.map(|h| (impact_point.y - h.distance) + 0.048).unwrap_or(0.048)
                } else {
                    impact_point.y + 0.052
                };

                let tangent = if impact_normal.y.abs() < 0.9 {
                    impact_normal.cross(Vec3::Y).normalize()
                } else {
                    impact_normal.cross(Vec3::X).normalize()
                };
                let bitangent = impact_normal.cross(tangent).normalize();

                let c = final_color.to_srgba();
                let stain_base = Color::srgb(
                    c.red * 0.78 + 0.18,
                    c.green * 0.78 + 0.18,
                    c.blue * 0.78 + 0.18,
                );
                let stain_emissive = final_color.to_linear() * 1.6;

                for i in 0..16 {
                    let angle_noise = (i as f32 * 4.37).sin() * 0.35;
                    let phi = (i as f32) * (std::f32::consts::TAU / 16.0) + angle_noise;
                    let radial_speed = 0.8 + 1.4 * ((i as f32 * 2.3).sin().abs());
                    let normal_speed = 0.4 + 0.7 * ((i as f32 * 3.1).cos().abs());

                    let shard_vel = if is_wall {
                        let burst = impact_normal * (1.6 + 1.2 * ((i as f32 * 2.5).sin().abs()));
                        let lateral = tangent * (phi.cos() * 1.4);
                        let down_push = Vec3::NEG_Y * (0.6 + 0.8 * ((i as f32 * 3.1).cos().abs()));
                        burst + lateral + down_push
                    } else {
                        tangent * (phi.cos() * radial_speed)
                            + bitangent * (phi.sin() * radial_speed)
                            + impact_normal * normal_speed
                    };

                    let shard_scale = 0.08 + 0.09 * ((i as f32 * 1.7).sin().abs());
                    let shard_mesh = meshes.add(create_glass_shard_mesh(shard_scale, i));
                    let shard_mat = materials.add(StandardMaterial {
                        base_color: stain_base,
                        emissive: stain_emissive,
                        perceptual_roughness: 0.10,
                        metallic: 0.18,
                        reflectance: 0.98,
                        cull_mode: None,
                        double_sided: true,
                        alpha_mode: AlphaMode::Opaque,
                        depth_bias: 1000.0,
                        ..default()
                    });

                    let rot_x = (i as f32 * 1.9).sin() * 0.35;
                    let rot_y = phi + (i as f32 * 2.8).cos() * 1.2;
                    let rot_z = (i as f32 * 3.4).sin() * 0.35;
                    let shard_rot = Quat::from_euler(EulerRot::XYZ, rot_x, rot_y, rot_z);

                    commands.spawn((
                        GlassShardParticle {
                            velocity: shard_vel,
                            lifetime: 0.0,
                            max_lifetime: 60.0 + 15.0 * ((i as f32 * 1.9).sin().abs()),
                            ground_y: floor_y,
                            settled: false,
                            color: final_color,
                        },
                        Mesh3d(shard_mesh),
                        MeshMaterial3d(shard_mat),
                        Transform::from_translation(impact_point + impact_normal * 0.055)
                            .with_rotation(shard_rot),
                    ));
                }

                continue;
            }
        }

        transform.translation += step;
        transform.rotate_local_x(7.5 * dt);
        transform.rotate_local_y(4.2 * dt);
        projectile.lifetime -= dt;
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_potion_puddles(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut puddles: Query<(Entity, &mut PotionPuddle, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut receivers: Query<(&Transform, &mut EffectReceiverComponent), Without<PotionPuddle>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    let puddle_snapshots: Vec<(Entity, Vec3, f32, Color, Vec<ActiveEffect>)> = puddles
        .iter()
        .map(|(e, p, tf, _)| (e, tf.translation, p.radius, p.color, p.effects.clone()))
        .collect();

    let mut despawned = Vec::new();
    for i in 0..puddle_snapshots.len() {
        if despawned.contains(&puddle_snapshots[i].0) {
            continue;
        }
        for j in (i + 1)..puddle_snapshots.len() {
            if despawned.contains(&puddle_snapshots[j].0) {
                continue;
            }
            let dist = puddle_snapshots[i].1.distance(puddle_snapshots[j].1);
            if dist <= (puddle_snapshots[i].2 + puddle_snapshots[j].2) * 0.65 {
                let absorbed_id = puddle_snapshots[j].0;
                let absorbed_color = puddle_snapshots[j].3;
                let absorbed_effects = puddle_snapshots[j].4.clone();
                despawned.push(absorbed_id);
                commands.entity(absorbed_id).despawn();

                if let Ok((_, mut p_main, mut tf_main, mat_main)) = puddles.get_mut(puddle_snapshots[i].0) {
                    let mixed_color = mix_potion_colors(p_main.color, absorbed_color);
                    p_main.color = mixed_color;
                    merge_potion_effects(&mut p_main.effects, &absorbed_effects);
                    p_main.radius = (p_main.radius + 0.40).min(3.0);
                    p_main.elapsed = (p_main.elapsed * 0.5).max(0.0);
                    if let Some(mat) = materials.get_mut(&mat_main.0) {
                        mat.base_color = mixed_color.with_alpha(0.88);
                        mat.emissive = mixed_color.to_linear() * 1.9;
                    }
                    tf_main.scale = Vec3::new(p_main.radius, 1.0, p_main.radius);
                }
            }
        }
    }

    for (entity, mut puddle, mut transform, mat_handle) in puddles.iter_mut() {
        if despawned.contains(&entity) {
            continue;
        }

        puddle.elapsed += dt;
        if puddle.elapsed >= puddle.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        let fade_start = 50.0;
        let fade_duration = puddle.max_lifetime - fade_start;
        let alpha = if puddle.elapsed <= fade_start {
            0.88
        } else {
            let progress = (puddle.elapsed - fade_start) / fade_duration;
            (0.88 * (1.0 - progress)).max(0.0)
        };

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.base_color = puddle.color.with_alpha(alpha);
        }

        let spread_duration = 0.38;
        let scale_spread = if puddle.elapsed < spread_duration {
            let t = (puddle.elapsed / spread_duration).clamp(0.0, 1.0);
            (0.15 + 0.85 * (1.0 - (1.0 - t).powi(3))) * puddle.radius
        } else if puddle.elapsed > fade_start {
            let progress = (puddle.elapsed - fade_start) / fade_duration;
            (1.0 - progress * 0.12) * puddle.radius
        } else {
            puddle.radius
        };
        transform.scale = Vec3::new(scale_spread, 1.0, scale_spread);

        for (r_tf, mut receiver) in receivers.iter_mut() {
            if r_tf.translation.distance(transform.translation) <= puddle.radius * 1.25 {
                for effect in &puddle.effects {
                    receiver.apply_effect(effect.kind, effect.duration, effect.magnitude);
                }
            }
        }
    }
}

fn update_glass_shards(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shards: Query<(Entity, &mut GlassShardParticle, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let gravity = Vec3::new(0.0, -26.0, 0.0);
    for (entity, mut shard, mut transform, mat_handle) in shards.iter_mut() {
        shard.lifetime += dt;
        if shard.lifetime >= shard.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        if !shard.settled {
            shard.velocity += gravity * dt;
            transform.translation += shard.velocity * dt;

            let spin = Quat::from_euler(EulerRot::XYZ, dt * 4.0, dt * 6.0, dt * 3.0);
            transform.rotation = transform.rotation * spin;

            if transform.translation.y <= shard.ground_y && shard.velocity.y < 0.0 {
                transform.translation.y = shard.ground_y;
                shard.velocity.y = 0.0;
                shard.velocity.x *= 0.15;
                shard.velocity.z *= 0.15;
                shard.settled = true;
                let flat_rot = Quat::from_rotation_y(transform.rotation.to_euler(EulerRot::YXZ).0);
                transform.rotation = flat_rot;
            }
        } else {
            let friction = (1.0 - 14.0 * dt).max(0.0);
            shard.velocity.x *= friction;
            shard.velocity.z *= friction;
            transform.translation.x += shard.velocity.x * dt;
            transform.translation.z += shard.velocity.z * dt;
            transform.translation.y = shard.ground_y;
        }

        let fade_time = 3.0;
        if shard.max_lifetime - shard.lifetime < fade_time {
            let remaining = (shard.max_lifetime - shard.lifetime) / fade_time;
            if let Some(mat) = materials.get_mut(&mat_handle.0) {
                let c = shard.color.to_srgba();
                let stain_base = Color::srgba(
                    c.red * 0.78 + 0.18,
                    c.green * 0.78 + 0.18,
                    c.blue * 0.78 + 0.18,
                    0.95 * remaining.clamp(0.0, 1.0),
                );
                mat.base_color = stain_base;
                mat.emissive = shard.color.to_linear() * (1.6 * remaining.clamp(0.0, 1.0));
            }
        }
    }
}

pub fn create_lathe_mesh(profile: &[(f32, f32)], segments: usize) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for j in 0..profile.len() {
        let (r, y) = profile[j];
        for i in 0..segments {
            let theta = i as f32 / segments as f32 * std::f32::consts::TAU;
            let cos_t = theta.cos();
            let sin_t = theta.sin();
            positions.push([r * cos_t, y, r * sin_t]);
            normals.push([cos_t, 0.0, sin_t]);
        }
    }

    for j in 0..(profile.len() - 1) {
        let r0 = (j * segments) as u32;
        let r1 = ((j + 1) * segments) as u32;
        for i in 0..segments {
            let next_i = ((i + 1) % segments) as u32;
            let i = i as u32;
            let a = r0 + i;
            let b = r0 + next_i;
            let c = r1 + next_i;
            let d = r1 + i;
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_potion_flask_mesh(slot: usize) -> Mesh {
    match slot {
        2 => {
            let profile = [
                (0.01, -0.12),
                (0.14, -0.11),
                (0.11, -0.04),
                (0.06, 0.05),
                (0.04, 0.12),
                (0.055, 0.14),
                (0.035, 0.15),
                (0.01, 0.18),
            ];
            create_lathe_mesh(&profile, 14)
        }
        3 => {
            let profile = [
                (0.01, -0.12),
                (0.10, -0.11),
                (0.10, 0.04),
                (0.08, 0.08),
                (0.045, 0.12),
                (0.06, 0.14),
                (0.038, 0.15),
                (0.01, 0.18),
            ];
            create_lathe_mesh(&profile, 14)
        }
        4 => {
            let profile = [
                (0.01, -0.13),
                (0.065, -0.12),
                (0.065, 0.08),
                (0.04, 0.12),
                (0.055, 0.14),
                (0.035, 0.15),
                (0.01, 0.18),
            ];
            create_lathe_mesh(&profile, 14)
        }
        5 => {
            let profile = [
                (0.01, -0.15),
                (0.07, -0.05),
                (0.13, 0.02),
                (0.06, 0.09),
                (0.04, 0.12),
                (0.055, 0.14),
                (0.035, 0.15),
                (0.01, 0.18),
            ];
            create_lathe_mesh(&profile, 14)
        }
        _ => {
            let profile = [
                (0.01, -0.12),
                (0.07, -0.11),
                (0.12, -0.06),
                (0.13, 0.0),
                (0.11, 0.06),
                (0.045, 0.11),
                (0.06, 0.14),
                (0.038, 0.15),
                (0.01, 0.18),
            ];
            create_lathe_mesh(&profile, 14)
        }
    }
}

pub fn create_flat_circle_mesh(radius: f32, segments: usize) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(segments + 2);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(segments + 2);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 3);

    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);

    for i in 0..segments {
        let theta = i as f32 / segments as f32 * std::f32::consts::TAU;
        let x = radius * theta.cos();
        let z = radius * theta.sin();
        positions.push([x, 0.0, z]);
        normals.push([0.0, 1.0, 0.0]);
    }

    for i in 0..segments {
        let curr = (i + 1) as u32;
        let next = ((i + 1) % segments + 1) as u32;
        indices.extend_from_slice(&[0, curr, next]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_irregular_puddle_mesh(base_radius: f32, segments: usize) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(segments * 3 + 1);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(segments * 3 + 1);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(segments * 3 + 1);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 18);

    positions.push([0.0, 0.028, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let perturbation = 1.0
            + 0.22 * (theta * 3.0).sin()
            + 0.15 * (theta * 5.0 + 0.8).cos()
            + 0.09 * (theta * 7.0).sin();
        let r_mid = base_radius * perturbation * 0.62;
        let x = r_mid * theta.cos();
        let z = r_mid * theta.sin();
        positions.push([x, 0.022, z]);
        let norm = Vec3::new(theta.cos() * 0.12, 0.98, theta.sin() * 0.12).normalize();
        normals.push([norm.x, norm.y, norm.z]);
        uvs.push([0.5 + (x / (base_radius * 2.4)), 0.5 + (z / (base_radius * 2.4))]);
    }

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let perturbation = 1.0
            + 0.22 * (theta * 3.0).sin()
            + 0.15 * (theta * 5.0 + 0.8).cos()
            + 0.09 * (theta * 7.0).sin();
        let r_rim = base_radius * perturbation;
        let x = r_rim * theta.cos();
        let z = r_rim * theta.sin();
        positions.push([x, 0.012, z]);
        let norm = Vec3::new(theta.cos() * 0.25, 0.95, theta.sin() * 0.25).normalize();
        normals.push([norm.x, norm.y, norm.z]);
        uvs.push([0.5 + (x / (base_radius * 2.4)), 0.5 + (z / (base_radius * 2.4))]);
    }

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let perturbation = 1.0
            + 0.22 * (theta * 3.0).sin()
            + 0.15 * (theta * 5.0 + 0.8).cos()
            + 0.09 * (theta * 7.0).sin();
        let r_skirt = base_radius * perturbation * 1.02;
        let x = r_skirt * theta.cos();
        let z = r_skirt * theta.sin();
        positions.push([x, -0.065, z]);
        normals.push([theta.cos(), 0.0, theta.sin()]);
        uvs.push([0.5 + (x / (base_radius * 2.4)), 0.5 + (z / (base_radius * 2.4))]);
    }

    let mid_start = 1 as u32;
    let rim_start = (1 + segments) as u32;
    let skirt_start = (1 + segments * 2) as u32;
    let seg_u32 = segments as u32;

    for i in 0..seg_u32 {
        let next_i = (i + 1) % seg_u32;

        let curr_mid = mid_start + i;
        let next_mid = mid_start + next_i;
        indices.extend_from_slice(&[0, next_mid, curr_mid]);

        let curr_rim = rim_start + i;
        let next_rim = rim_start + next_i;
        indices.extend_from_slice(&[curr_mid, next_mid, next_rim]);
        indices.extend_from_slice(&[curr_mid, next_rim, curr_rim]);

        let curr_skirt = skirt_start + i;
        let next_skirt = skirt_start + next_i;
        indices.extend_from_slice(&[curr_rim, next_rim, next_skirt]);
        indices.extend_from_slice(&[curr_rim, next_skirt, curr_skirt]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_wall_splatter_mesh(base_radius: f32, segments: usize) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(segments * 3 + 1);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(segments * 3 + 1);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(segments * 3 + 1);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 18);

    positions.push([0.0, 0.022, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        let base_pert = 1.0 + 0.28 * (theta * 3.0).sin() + 0.18 * (theta * 5.0 + 0.5).cos();
        let pert = if sin_t < 0.0 {
            let drip = (-sin_t).powf(1.6) * (0.80 + 0.70 * ((theta * 8.0).sin().max(0.0)).powi(3));
            base_pert + drip
        } else {
            base_pert * 0.72
        };

        let r_mid = base_radius * pert * 0.60;
        let x = r_mid * cos_t;
        let z = r_mid * sin_t;
        let wrap = -0.050 * (x / base_radius).powi(2);
        let y = 0.016 + wrap;
        positions.push([x, y, z]);
        let norm = Vec3::new(cos_t * 0.15, 0.98, sin_t * 0.15).normalize();
        normals.push([norm.x, norm.y, norm.z]);
        uvs.push([0.5 + (x / (base_radius * 2.5)), 0.5 + (z / (base_radius * 2.5))]);
    }

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        let base_pert = 1.0 + 0.28 * (theta * 3.0).sin() + 0.18 * (theta * 5.0 + 0.5).cos();
        let pert = if sin_t < 0.0 {
            let drip = (-sin_t).powf(1.6) * (0.80 + 0.70 * ((theta * 8.0).sin().max(0.0)).powi(3));
            base_pert + drip
        } else {
            base_pert * 0.72
        };

        let r_rim = base_radius * pert;
        let x = r_rim * cos_t;
        let z = r_rim * sin_t;
        let wrap = -0.050 * (x / base_radius).powi(2);
        let y = 0.008 + wrap;
        positions.push([x, y, z]);
        let norm = Vec3::new(cos_t * 0.28, 0.94, sin_t * 0.28).normalize();
        normals.push([norm.x, norm.y, norm.z]);
        uvs.push([0.5 + (x / (base_radius * 2.5)), 0.5 + (z / (base_radius * 2.5))]);
    }

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        let base_pert = 1.0 + 0.28 * (theta * 3.0).sin() + 0.18 * (theta * 5.0 + 0.5).cos();
        let pert = if sin_t < 0.0 {
            let drip = (-sin_t).powf(1.6) * (0.80 + 0.70 * ((theta * 8.0).sin().max(0.0)).powi(3));
            base_pert + drip
        } else {
            base_pert * 0.72
        };

        let r_skirt = base_radius * pert * 1.03;
        let x = r_skirt * cos_t;
        let z = r_skirt * sin_t;
        let wrap = -0.050 * (x / base_radius).powi(2);
        let y = -0.055 + wrap;
        positions.push([x, y, z]);
        normals.push([cos_t, 0.0, sin_t]);
        uvs.push([0.5 + (x / (base_radius * 2.5)), 0.5 + (z / (base_radius * 2.5))]);
    }

    let mid_start = 1 as u32;
    let rim_start = (1 + segments) as u32;
    let skirt_start = (1 + segments * 2) as u32;
    let seg_u32 = segments as u32;

    for i in 0..seg_u32 {
        let next_i = (i + 1) % seg_u32;

        let curr_mid = mid_start + i;
        let next_mid = mid_start + next_i;
        indices.extend_from_slice(&[0, next_mid, curr_mid]);

        let curr_rim = rim_start + i;
        let next_rim = rim_start + next_i;
        indices.extend_from_slice(&[curr_mid, next_mid, next_rim]);
        indices.extend_from_slice(&[curr_mid, next_rim, curr_rim]);

        let curr_skirt = skirt_start + i;
        let next_skirt = skirt_start + next_i;
        indices.extend_from_slice(&[curr_rim, next_rim, next_skirt]);
        indices.extend_from_slice(&[curr_rim, next_skirt, curr_skirt]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_glass_shard_mesh(scale: f32, shard_type: usize) -> Mesh {
    match shard_type % 3 {
        0 => {
            let positions: Vec<[f32; 3]> = vec![
                [0.0, scale * 0.28, -scale * 0.85],
                [scale * 0.22, 0.0, -scale * 0.15],
                [scale * 0.08, 0.0, scale * 0.95],
                [-scale * 0.16, 0.0, scale * 0.35],
                [-scale * 0.24, 0.0, -scale * 0.40],
                [0.0, scale * 0.08, 0.0],
            ];
            let normals: Vec<[f32; 3]> = vec![
                [0.0, 0.95, -0.30],
                [0.85, 0.40, -0.20],
                [0.25, 0.80, 0.50],
                [-0.75, 0.50, 0.40],
                [-0.85, 0.40, -0.30],
                [0.0, 1.0, 0.0],
            ];
            let indices: Vec<u32> = vec![
                0, 1, 5,
                1, 2, 5,
                2, 3, 5,
                3, 4, 5,
                4, 0, 5,
                0, 5, 1,
                1, 5, 2,
                2, 5, 3,
                3, 5, 4,
                4, 5, 0,
            ];
            Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                .with_inserted_indices(Indices::U32(indices))
        }
        1 => {
            let positions: Vec<[f32; 3]> = vec![
                [-scale * 0.65, 0.0, -scale * 0.55],
                [scale * 0.70, 0.0, -scale * 0.35],
                [scale * 0.20, 0.0, scale * 0.75],
                [-scale * 0.40, 0.0, scale * 0.35],
                [0.0, scale * 0.18, 0.0],
            ];
            let normals: Vec<[f32; 3]> = vec![
                [-0.65, 0.55, -0.50],
                [0.70, 0.55, -0.35],
                [0.20, 0.80, 0.55],
                [-0.50, 0.70, 0.35],
                [0.0, 1.0, 0.0],
            ];
            let indices: Vec<u32> = vec![
                4, 0, 1,
                4, 1, 2,
                4, 2, 3,
                4, 3, 0,
                0, 2, 1,
                0, 3, 2,
            ];
            Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                .with_inserted_indices(Indices::U32(indices))
        }
        _ => {
            let positions: Vec<[f32; 3]> = vec![
                [-scale * 0.45, scale * 0.22, -scale * 0.35],
                [scale * 0.45, scale * 0.22, -scale * 0.35],
                [scale * 0.35, scale * 0.22, scale * 0.45],
                [-scale * 0.35, scale * 0.22, scale * 0.45],
                [-scale * 0.55, 0.0, -scale * 0.45],
                [scale * 0.55, 0.0, -scale * 0.45],
                [scale * 0.45, 0.0, scale * 0.55],
                [-scale * 0.45, 0.0, scale * 0.55],
            ];
            let normals: Vec<[f32; 3]> = vec![
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [-0.6, -0.5, -0.6],
                [0.6, -0.5, -0.6],
                [0.6, -0.5, 0.6],
                [-0.6, -0.5, 0.6],
            ];
            let indices: Vec<u32> = vec![
                0, 1, 2, 0, 2, 3,
                4, 6, 5, 4, 7, 6,
                0, 5, 1, 0, 4, 5,
                1, 6, 2, 1, 5, 6,
                2, 7, 3, 2, 6, 7,
                3, 4, 0, 3, 7, 4,
            ];
            Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                .with_inserted_indices(Indices::U32(indices))
        }
    }
}

fn update_active_effects(time: Res<Time>, mut receivers: Query<&mut EffectReceiverComponent>) {
    let dt = time.delta_secs();
    for mut receiver in receivers.iter_mut() {
        receiver.active_effects.retain_mut(|effect| {
            effect.elapsed += dt;
            effect.elapsed < effect.duration
        });
    }
}

fn update_thermal_geysers(
    time: Res<Time>,
    mut geysers: Query<(&mut ThermalGeyser, &Transform)>,
    mut receivers: Query<(&Transform, &mut EffectReceiverComponent)>,
) {
    let dt = time.delta_secs();
    for (mut geyser, g_tf) in geysers.iter_mut() {
        geyser.eruption_timer += dt;
        if geyser.eruption_timer >= geyser.eruption_interval {
            geyser.eruption_timer = 0.0;
            geyser.is_erupting = !geyser.is_erupting;
        }

        if geyser.is_erupting {
            let g_pos = g_tf.translation;
            let radius_sq = geyser.heat_radius * geyser.heat_radius;
            for (r_tf, mut receiver) in receivers.iter_mut() {
                if g_pos.distance_squared(r_tf.translation) <= radius_sq {
                    receiver.apply_effect(AlchemicalEffectKind::ThermalHeat, 1.5, 25.0);
                    receiver.apply_effect(AlchemicalEffectKind::Vaporize, 1.0, 10.0);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alchemy_plugin_build() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AlchemyPlugin);
        assert!(app.is_plugin_added::<AlchemyPlugin>());
    }

    #[test]
    fn test_effect_receiver_apply_and_query() {
        let mut receiver = EffectReceiverComponent::default();
        assert!(!receiver.has_effect(AlchemicalEffectKind::Frost));

        receiver.apply_effect(AlchemicalEffectKind::Frost, 3.0, 15.0);
        assert!(receiver.has_effect(AlchemicalEffectKind::Frost));
        assert_eq!(receiver.active_effects.len(), 1);

        receiver.apply_effect(AlchemicalEffectKind::Frost, 5.0, 25.0);
        assert_eq!(receiver.active_effects.len(), 1);
        assert_eq!(receiver.active_effects[0].duration, 5.0);
        assert_eq!(receiver.active_effects[0].magnitude, 25.0);
    }

    #[test]
    fn test_system_update_active_effects() {
        let mut world = World::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(200));
        world.insert_resource(time);

        let entity = world
            .spawn(EffectReceiverComponent {
                active_effects: vec![ActiveEffect {
                    kind: AlchemicalEffectKind::Vitality,
                    duration: 0.1,
                    elapsed: 0.0,
                    magnitude: 20.0,
                }],
            })
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_active_effects);
        schedule.run(&mut world);

        let receiver = world.get::<EffectReceiverComponent>(entity).unwrap();
        assert!(
            !receiver.has_effect(AlchemicalEffectKind::Vitality),
            "Effect must have expired and been retained out"
        );
    }

    #[test]
    fn test_system_update_thermal_geysers() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AlchemyPlugin);

        app.world_mut().spawn((
            ThermalGeyser {
                eruption_timer: 0.0,
                eruption_interval: 5.0,
                is_erupting: true,
                heat_radius: 6.0,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        let close_entity = app
            .world_mut()
            .spawn((
                EffectReceiverComponent::default(),
                Transform::from_xyz(2.0, 0.0, 2.0),
            ))
            .id();

        let far_entity = app
            .world_mut()
            .spawn((
                EffectReceiverComponent::default(),
                Transform::from_xyz(20.0, 0.0, 20.0),
            ))
            .id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(16));
        app.insert_resource(time);

        app.update();

        let close_recv = app
            .world()
            .get::<EffectReceiverComponent>(close_entity)
            .unwrap();
        assert!(close_recv.has_effect(AlchemicalEffectKind::ThermalHeat));
        assert!(close_recv.has_effect(AlchemicalEffectKind::Vaporize));

        let far_recv = app
            .world()
            .get::<EffectReceiverComponent>(far_entity)
            .unwrap();
        assert!(!far_recv.has_effect(AlchemicalEffectKind::ThermalHeat));
    }

    #[test]
    fn test_potion_selected_effects_all_slots_unique_and_colored() {
        let (c1, e1) = selected_potion_effects(1);
        let (c2, e2) = selected_potion_effects(2);
        let (c3, e3) = selected_potion_effects(3);
        let (c4, e4) = selected_potion_effects(4);
        let (c5, e5) = selected_potion_effects(5);

        assert_eq!(e1[0].kind, AlchemicalEffectKind::Swiftness);
        assert_eq!(e2[0].kind, AlchemicalEffectKind::ThermalHeat);
        assert_eq!(e3[0].kind, AlchemicalEffectKind::Vitality);
        assert_eq!(e4[0].kind, AlchemicalEffectKind::Frost);
        assert_eq!(e5[0].kind, AlchemicalEffectKind::Luminescence);

        assert_ne!(c1, c2);
        assert_ne!(c2, c3);
        assert_ne!(c3, c4);
        assert_ne!(c4, c5);
    }

    #[test]
    fn test_aim_trajectory_calculation_and_impact_ballistic_curve() {
        let origin = Vec3::new(0.0, 1.5, 0.0);
        let vel = Vec3::new(0.0, 5.0, -10.0);
        let (pts, hit) = calculate_aim_trajectory_and_impact(origin, vel, 9.8, 25, 0.045, None);
        assert!(pts.len() >= 20);
        assert_eq!(pts[0], origin);
        assert!(pts[pts.len() - 1].z < origin.z);
        assert!(hit.is_none());

        let mut max_y = origin.y;
        for p in &pts {
            if p.y > max_y {
                max_y = p.y;
            }
        }
        assert!(max_y > origin.y, "Parabolic arc must reach an apex higher than origin");
        assert!(pts[pts.len() - 1].y < max_y, "Arc must descend after apex");
    }

    #[test]
    fn test_aim_trajectory_calculation_generates_arc() {
        let origin = Vec3::new(0.0, 1.5, 0.0);
        let vel = Vec3::new(0.0, 5.0, -10.0);
        let pts = calculate_aim_trajectory(origin, vel, 9.8, 20, 0.05, None);
        assert!(pts.len() >= 15);
        assert_eq!(pts[0], origin);
        assert!(pts[pts.len() - 1].z < origin.z);
    }

    #[test]
    fn test_potion_aiming_spawns_indicators_and_cleans_up() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.add_plugins(avian3d::prelude::PhysicsPlugins::default());
        app.init_resource::<Assets<StandardMaterial>>();
        app.init_resource::<Assets<Mesh>>();

        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Right);
        app.insert_resource(mouse);

        app.world_mut().spawn((
            Player,
            PlayerState::Active,
        ));

        app.world_mut().spawn((
            PlayerCamera,
            Transform::from_xyz(0.0, 1.75, 0.0),
            GlobalTransform::from_translation(Vec3::new(0.0, 1.75, 0.0)),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_potion_aiming);
        schedule.run(app.world_mut());

        let mut indicator_query = app.world_mut().query::<&TrajectoryAimIndicator>();
        let count = indicator_query.iter(app.world()).count();
        assert!(count >= 15, "Aiming must spawn trajectory arc indicators");

        let mouse_released = ButtonInput::<MouseButton>::default();
        app.insert_resource(mouse_released);
        schedule.run(app.world_mut());

        let mut indicator_query_after = app.world_mut().query::<&TrajectoryAimIndicator>();
        let count_after = indicator_query_after.iter(app.world()).count();
        assert_eq!(count_after, 0, "Releasing aim must despawn all indicators");
    }

    #[test]
    fn test_potion_projectile_flight_lifecycle_no_premature_despawn() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.add_plugins(avian3d::prelude::PhysicsPlugins::default());
        app.init_resource::<Assets<StandardMaterial>>();
        app.init_resource::<Assets<Mesh>>();

        let initial_pos = Vec3::new(0.0, 5.0, 0.0);
        let initial_vel = Vec3::new(0.0, 2.0, -12.0);

        let proj_entity = app
            .world_mut()
            .spawn((
                Transform::from_translation(initial_pos),
                PotionProjectile {
                    velocity: initial_vel,
                    effects: vec![ActiveEffect {
                        kind: AlchemicalEffectKind::Swiftness,
                        duration: 5.0,
                        elapsed: 0.0,
                        magnitude: 20.0,
                    }],
                    lifetime: 8.0,
                    color: Color::srgb(0.12, 0.92, 0.42),
                    slot: 1,
                },
            ))
            .id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_potion_projectiles);

        for _ in 0..10 {
            schedule.run(app.world_mut());
        }

        let proj_opt = app.world().get::<PotionProjectile>(proj_entity);
        assert!(
            proj_opt.is_some(),
            "Projectile must remain active during in-flight propagation across multiple frames"
        );

        let tf = app.world().get::<Transform>(proj_entity).unwrap();
        assert!(
            tf.translation.z < -4.0,
            "Projectile must travel forward along Z axis"
        );
        let proj = proj_opt.unwrap();
        assert!(
            proj.lifetime < 7.6,
            "Projectile lifetime must decrease during flight"
        );
        assert!(
            proj.velocity.y < initial_vel.y,
            "Vertical velocity must decrease due to gravity"
        );
    }

    #[test]
    fn test_potion_projectile_impact_and_splash_lifecycle() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::transform::TransformPlugin,
            bevy::asset::AssetPlugin::default(),
            bevy::scene::ScenePlugin::default(),
            avian3d::prelude::PhysicsPlugins::default(),
        ));
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(50),
        ));
        app.init_resource::<Assets<StandardMaterial>>();
        app.init_resource::<Assets<Mesh>>();
        app.add_systems(Update, update_potion_projectiles);

        let target_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, -3.0),
                GlobalTransform::from_translation(Vec3::new(0.0, 0.0, -3.0)),
                RigidBody::Static,
                Collider::cuboid(4.0, 4.0, 0.5),
                EffectReceiverComponent::default(),
            ))
            .id();

        app.update();

        let proj_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, -1.0),
                PotionProjectile {
                    velocity: Vec3::new(0.0, 0.0, -50.0),
                    effects: vec![ActiveEffect {
                        kind: AlchemicalEffectKind::ThermalHeat,
                        duration: 6.0,
                        elapsed: 0.0,
                        magnitude: 35.0,
                    }],
                    lifetime: 8.0,
                    color: Color::srgb(0.95, 0.12, 0.16),
                    slot: 2,
                },
            ))
            .id();

        app.update();

        assert!(
            app.world().get_entity(proj_entity).is_err(),
            "Projectile must despawn upon impacting the target obstacle"
        );

        let target_recv = app.world().get::<EffectReceiverComponent>(target_entity).unwrap();
        assert!(
            target_recv.has_effect(AlchemicalEffectKind::ThermalHeat),
            "Receiver at impact location must receive the potion alchemical effect"
        );

        let mut puddle_query = app.world_mut().query::<&PotionPuddle>();
        let puddle_count = puddle_query.iter(app.world()).count();
        assert_eq!(puddle_count, 1, "Exactly one alchemical puddle must be spawned at impact");

        let mut shards_query = app.world_mut().query::<&GlassShardParticle>();
        let shards_count = shards_query.iter(app.world()).count();
        assert_eq!(shards_count, 16, "Exactly 16 glass shard particles must be spawned on shatter");
    }

    #[test]
    fn test_sprint_throw_inaccuracy_application() {
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let still_dir = apply_throw_inaccuracy(forward, false, 1.23);
        assert_eq!(still_dir, forward);

        let running_dir = apply_throw_inaccuracy(forward, true, 4.56);
        assert_ne!(running_dir, forward);
        let dot = running_dir.dot(forward);
        assert!(dot >= 0.75, "Direction must be within 20% variance window");
    }

    #[test]
    fn test_potion_puddle_lifetime_greater_than_60s_and_gradual_fade() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_resource::<Assets<StandardMaterial>>();

        let color = Color::srgb(0.95, 0.12, 0.16);
        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        let mat_handle = materials.add(StandardMaterial {
            base_color: color.with_alpha(0.88),
            ..default()
        });

        let puddle_entity = app
            .world_mut()
            .spawn((
                PotionPuddle {
                    color,
                    elapsed: 0.0,
                    max_lifetime: 75.0,
                    radius: 1.0,
                    effects: Vec::new(),
                    is_wall: false,
                },
                MeshMaterial3d(mat_handle.clone()),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        assert!(app.world().get::<PotionPuddle>(puddle_entity).unwrap().max_lifetime > 60.0);

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs(30));
        app.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_potion_puddles);
        schedule.run(app.world_mut());

        let mat_30s = app.world().resource::<Assets<StandardMaterial>>().get(&mat_handle).unwrap();
        assert_eq!(mat_30s.base_color.to_srgba().alpha, 0.88);

        let mut time2 = Time::<()>::default();
        time2.advance_by(std::time::Duration::from_secs(32));
        app.insert_resource(time2);
        schedule.run(app.world_mut());

        let mat_62s = app.world().resource::<Assets<StandardMaterial>>().get(&mat_handle).unwrap();
        assert!(mat_62s.base_color.to_srgba().alpha < 0.88);
        assert!(mat_62s.base_color.to_srgba().alpha > 0.0);

        let mut time3 = Time::<()>::default();
        time3.advance_by(std::time::Duration::from_secs(20));
        app.insert_resource(time3);
        schedule.run(app.world_mut());

        assert!(app.world().get_entity(puddle_entity).is_err());
    }

    #[test]
    fn test_create_potion_flask_mesh_has_all_slots() {
        for slot in 1..=6 {
            let mesh = create_potion_flask_mesh(slot);
            assert!(mesh.count_vertices() > 50);
        }
    }

    #[test]
    fn test_create_flat_circle_and_irregular_puddle_mesh() {
        let circle = create_flat_circle_mesh(0.38, 24);
        assert!(circle.count_vertices() >= 25);

        let puddle = create_irregular_puddle_mesh(0.72, 24);
        assert!(puddle.count_vertices() >= 25);
    }

    #[test]
    fn test_update_glass_shards_bounce_and_fade() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_resource::<Assets<StandardMaterial>>();

        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        let mat_handle = materials.add(StandardMaterial::default());

        let shard_entity = app
            .world_mut()
            .spawn((
                GlassShardParticle {
                    velocity: Vec3::new(0.3, -8.0, 0.0),
                    lifetime: 0.0,
                    max_lifetime: 65.0,
                    ground_y: 0.0,
                    settled: false,
                    color: Color::srgb(0.85, 0.95, 1.0),
                },
                MeshMaterial3d(mat_handle.clone()),
                Transform::from_xyz(0.0, -0.01, 0.0),
            ))
            .id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_glass_shards);
        schedule.run(app.world_mut());

        let shard = app.world().get::<GlassShardParticle>(shard_entity).unwrap();
        assert_eq!(shard.velocity.y, 0.0);
        assert!(shard.settled);

        let mut time2 = Time::<()>::default();
        time2.advance_by(std::time::Duration::from_secs(30));
        app.insert_resource(time2);
        schedule.run(app.world_mut());
        assert!(app.world().get_entity(shard_entity).is_ok());
    }

    #[test]
    fn test_potion_shards_persist_on_ground_longer_than_50s_and_puddle_spreads() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_resource::<Assets<StandardMaterial>>();

        let color = Color::srgb(0.2, 0.8, 0.9);
        let puddle_mat = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());

        let puddle_entity = app
            .world_mut()
            .spawn((
                PotionPuddle {
                    color,
                    elapsed: 0.0,
                    max_lifetime: 75.0,
                    radius: 1.0,
                    effects: Vec::new(),
                    is_wall: false,
                },
                MeshMaterial3d(puddle_mat),
                Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(0.15, 1.0, 0.15)),
            ))
            .id();

        let mut shard_materials = Vec::new();
        {
            let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
            for _ in 0..16 {
                shard_materials.push(materials.add(StandardMaterial::default()));
            }
        }

        let mut shard_entities = Vec::new();
        for (i, shard_mat) in shard_materials.into_iter().enumerate() {
            let entity = app
                .world_mut()
                .spawn((
                    GlassShardParticle {
                        velocity: Vec3::ZERO,
                        lifetime: 0.0,
                        max_lifetime: 60.0 + (i as f32),
                        ground_y: 0.0,
                        settled: true,
                        color: Color::srgb(0.85, 0.95, 1.0),
                    },
                    MeshMaterial3d(shard_mat),
                    Transform::from_xyz(0.1 * i as f32, 0.0, 0.1 * i as f32),
                ))
                .id();
            shard_entities.push(entity);
        }

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(400));
        app.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems((update_potion_puddles, update_glass_shards));
        schedule.run(app.world_mut());

        let tf = app.world().get::<Transform>(puddle_entity).unwrap();
        assert_eq!(tf.scale.x, 1.0);
        assert_eq!(tf.scale.z, 1.0);

        let mut time2 = Time::<()>::default();
        time2.advance_by(std::time::Duration::from_secs(50));
        app.insert_resource(time2);
        schedule.run(app.world_mut());

        for shard_id in shard_entities {
            assert!(app.world().get_entity(shard_id).is_ok());
            let shard = app.world().get::<GlassShardParticle>(shard_id).unwrap();
            assert!(shard.settled);
            assert_eq!(shard.color, Color::srgb(0.85, 0.95, 1.0));
        }
    }

    #[test]
    fn test_volumetric_puddle_mesh_skirt_and_normals() {
        let puddle = create_irregular_puddle_mesh(1.10, 32);
        assert!(puddle.count_vertices() >= 97);
        let positions = puddle.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        if let bevy::render::mesh::VertexAttributeValues::Float32x3(pts) = positions {
            assert!(pts[0][1] > 0.02);
            let has_skirt = pts.iter().any(|p| p[1] < -0.05);
            assert!(has_skirt);
        }
    }

    #[test]
    fn test_glass_shard_archetypes() {
        for shard_type in 0..3 {
            let shard = create_glass_shard_mesh(0.12, shard_type);
            assert!(shard.count_vertices() >= 5);
        }
    }

    #[test]
    fn test_mix_potion_colors_and_effects_reactions() {
        let red = Color::srgb(0.95, 0.12, 0.16);
        let blue = Color::srgb(0.12, 0.42, 0.95);
        let purple = mix_potion_colors(red, blue);
        let s = purple.to_srgba();
        assert!(s.red > 0.4 && s.blue > 0.4);

        let mut effects = vec![ActiveEffect {
            kind: AlchemicalEffectKind::ThermalHeat,
            duration: 6.0,
            elapsed: 0.0,
            magnitude: 30.0,
        }];
        let incoming = vec![ActiveEffect {
            kind: AlchemicalEffectKind::Frost,
            duration: 5.0,
            elapsed: 0.0,
            magnitude: 25.0,
        }];
        merge_potion_effects(&mut effects, &incoming);
        assert!(effects.iter().any(|e| e.kind == AlchemicalEffectKind::Vaporize));
        assert!(effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat));
        assert!(effects.iter().any(|e| e.kind == AlchemicalEffectKind::Frost));
    }

    #[test]
    fn test_create_wall_splatter_mesh_downward_drips_and_wrap() {
        let wall_mesh = create_wall_splatter_mesh(1.10, 32);
        assert!(wall_mesh.count_vertices() >= 97);
        let positions = wall_mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        if let bevy::render::mesh::VertexAttributeValues::Float32x3(pts) = positions {
            let has_downward_drip = pts.iter().any(|p| p[2] < -1.1);
            assert!(has_downward_drip);
            let has_conformal_wrap = pts.iter().any(|p| p[1] < 0.0);
            assert!(has_conformal_wrap);
        }
    }

    #[test]
    fn test_inter_puddle_merging_absorbs_and_expands() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_resource::<Assets<StandardMaterial>>();

        let red = Color::srgb(0.95, 0.12, 0.16);
        let blue = Color::srgb(0.12, 0.42, 0.95);

        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        let mat1 = materials.add(StandardMaterial::default());
        let mat2 = materials.add(StandardMaterial::default());

        let p1 = app
            .world_mut()
            .spawn((
                PotionPuddle {
                    color: red,
                    elapsed: 5.0,
                    max_lifetime: 75.0,
                    radius: 1.15,
                    effects: vec![ActiveEffect {
                        kind: AlchemicalEffectKind::ThermalHeat,
                        duration: 6.0,
                        elapsed: 0.0,
                        magnitude: 25.0,
                    }],
                    is_wall: false,
                },
                MeshMaterial3d(mat1),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        let p2 = app
            .world_mut()
            .spawn((
                PotionPuddle {
                    color: blue,
                    elapsed: 1.0,
                    max_lifetime: 75.0,
                    radius: 1.15,
                    effects: vec![ActiveEffect {
                        kind: AlchemicalEffectKind::Frost,
                        duration: 5.0,
                        elapsed: 0.0,
                        magnitude: 20.0,
                    }],
                    is_wall: false,
                },
                MeshMaterial3d(mat2),
                Transform::from_xyz(0.5, 0.0, 0.0),
            ))
            .id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_potion_puddles);
        schedule.run(app.world_mut());

        assert!(app.world().get_entity(p1).is_ok());
        assert!(app.world().get_entity(p2).is_err());

        let mixed_puddle = app.world().get::<PotionPuddle>(p1).unwrap();
        assert!(mixed_puddle.radius > 1.15);
        assert!(mixed_puddle.effects.iter().any(|e| e.kind == AlchemicalEffectKind::Vaporize));
    }
}


