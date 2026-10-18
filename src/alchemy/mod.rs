use crate::{
    player::camera::{LocomotionState, Player, PlayerCamera, PlayerState},
    ui::{hud::ActiveHotbarSlot, menu::GameState},
};
use avian3d::prelude::*;
use bevy::prelude::*;
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
}

#[derive(Component, Debug)]
pub struct TrajectoryAimIndicator;

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
                    .run_if(resource_exists::<SpatialQueryPipeline>),
                spawn_potion_projectile.run_if(resource_exists::<ButtonInput<MouseButton>>),
                update_potion_projectiles.run_if(resource_exists::<SpatialQueryPipeline>),
            ),
        );
    }
}

pub fn calculate_aim_trajectory(
    origin: Vec3,
    initial_velocity: Vec3,
    gravity: f32,
    steps: usize,
    step_time: f32,
    spatial_query: Option<&SpatialQuery>,
) -> Vec<Vec3> {
    let mut points = Vec::with_capacity(steps);
    let mut current_pos = origin;
    let mut current_vel = initial_velocity;
    points.push(current_pos);

    for _ in 0..steps {
        let next_vel = current_vel + Vec3::new(0.0, -gravity * step_time, 0.0);
        let displacement = (current_vel + next_vel) * 0.5 * step_time;
        let segment_len = displacement.length();

        if let Some(sq) = spatial_query {
            if segment_len > 1e-4 {
                let dir = displacement / segment_len;
                if let Some(hit) = sq.cast_ray(
                    current_pos,
                    Dir3::new(dir).unwrap_or(Dir3::NEG_Y),
                    segment_len,
                    true,
                    &SpatialQueryFilter::default(),
                ) {
                    current_pos += dir * hit.distance;
                    points.push(current_pos);
                    break;
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
    points
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

fn selected_potion_effects(slot: usize) -> (Color, Vec<ActiveEffect>) {
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

fn update_potion_aiming(
    game_state: Option<Res<State<GameState>>>,
    mouse: Res<ButtonInput<MouseButton>>,
    slot: Option<Res<ActiveHotbarSlot>>,
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    player: Query<&PlayerState>,
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
    let origin = camera_tf.translation() + forward * 0.45;
    let initial_velocity = forward * 16.0;
    let gravity = 9.8;
    let trajectory_points = calculate_aim_trajectory(
        origin,
        initial_velocity,
        gravity,
        45,
        0.045,
        Some(&spatial_query),
    );

    let sphere_mesh = meshes.add(Sphere::new(0.045).mesh().uv(8, 6));
    let line_mat = materials.add(StandardMaterial {
        base_color: color,
        emissive: color.to_linear() * 1.5,
        unlit: true,
        ..default()
    });

    for pt in trajectory_points.iter().step_by(2) {
        commands.spawn((
            TrajectoryAimIndicator,
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(line_mat.clone()),
            Transform::from_translation(*pt),
        ));
    }

    if let Some(&impact_pt) = trajectory_points.last() {
        let ring_mesh = meshes.add(Torus::new(0.06, 0.35).mesh());
        let arrow_mat = materials.add(StandardMaterial {
            base_color: color,
            emissive: color.to_linear() * 2.5,
            unlit: true,
            ..default()
        });
        commands.spawn((
            TrajectoryAimIndicator,
            Mesh3d(ring_mesh),
            MeshMaterial3d(arrow_mat),
            Transform::from_translation(impact_pt + Vec3::new(0.0, 0.03, 0.0)),
        ));
    }
}

fn spawn_potion_projectile(
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
    let origin = camera.translation() + base_direction * 0.55;
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.12).mesh().uv(12, 8))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            emissive: color.to_linear(),
            perceptual_roughness: 0.18,
            ..default()
        })),
        Transform::from_translation(origin),
        PotionProjectile {
            velocity: final_direction * 16.0,
            effects,
            lifetime: 4.5,
        },
    ));
}

fn update_potion_projectiles(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut projectiles: Query<(Entity, &mut Transform, &mut PotionProjectile)>,
    mut receivers: Query<&mut EffectReceiverComponent>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let gravity = Vec3::new(0.0, -9.8, 0.0);
    for (entity, mut transform, mut projectile) in &mut projectiles {
        projectile.velocity += gravity * dt;
        let distance = projectile.velocity.length() * dt;
        let direction = projectile.velocity.normalize_or_zero();
        let hit = spatial_query.cast_ray(
            transform.translation,
            Dir3::new(direction).unwrap_or(Dir3::NEG_Y),
            distance,
            true,
            &SpatialQueryFilter::default(),
        );
        if let Some(hit) = hit {
            if let Ok(mut receiver) = receivers.get_mut(hit.entity) {
                for effect in &projectile.effects {
                    receiver.apply_effect(effect.kind, effect.duration, effect.magnitude);
                }
            }
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation += projectile.velocity * dt;
        projectile.lifetime -= dt;
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
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
    fn test_aim_trajectory_calculation_generates_arc() {
        let origin = Vec3::new(0.0, 1.5, 0.0);
        let vel = Vec3::new(0.0, 5.0, -10.0);
        let pts = calculate_aim_trajectory(origin, vel, 9.8, 20, 0.05, None);
        assert!(pts.len() >= 15);
        assert_eq!(pts[0], origin);
        assert!(pts[pts.len() - 1].z < origin.z);
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
}
